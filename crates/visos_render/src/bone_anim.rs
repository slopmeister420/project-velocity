//! BoneAnimation\BoneAnim.cpp — `.gaf` bone animations ('GAF0': u32 0, u32 file size, u32 0, u32 count, then
//! `count` absolute offsets of 196-byte infos; every offset in an info is absolute). Keys are per-bone 16-byte
//! records {u16 euler x, y, z (65536 = 2pi), u16 position index, u16 scale index, u16 start frame, u16 run, pad};
//! compressed anims hold one record per run and interpolate to the next one.

use bevy::prelude::*;

use crate::renderer::renderer_types::{cstr, rd16, rd32, rdf, ModelError};

/// 65536 / 2pi
const ANGLE_SCALE: f32 = 10430.378;

#[derive(Clone, Debug)]
pub struct BoneTrack {
    pub name: String, // +0x00
    pub active: bool, // +0x04
    /// +0x08 records (compressed: runs, else one per frame)
    pub keys: Vec<[u16; 8]>,
}

#[derive(Clone, Debug)]
pub struct BoneAnim {
    pub name: String,      // +0x00 (128 bytes)
    pub checksum: u32,     // +0x80 skeleton checksum
    pub fps: f32,          // +0x84
    pub duration: f32,     // +0x88 seconds
    pub num_frames: u32,   // +0x8C
    pub offset: Vec3,      // +0x90
    pub looping: u32,      // +0x9C
    pub compressed: bool,  // +0xA0
    pub bones: Vec<BoneTrack>, // +0xA4 count, +0xA8 infos {name, active, key count}, +0xB0 key pointers
    pub positions: Vec<Vec4>,  // +0xB4 count, +0xB8
    pub scales: Vec<Vec4>,     // +0xBC count, +0xC0
}

/// sampled bone: rotation, translation (w = 1), scale (BoneAnimBlendTree QTS)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoneQts {
    pub rotation: Quat,
    pub scale: Vec4,
    pub translation: Vec4,
}

fn vec4(b: &[u8], o: usize) -> Result<Vec4, ModelError> {
    Ok(Vec4::new(rdf(b, o)?, rdf(b, o + 4)?, rdf(b, o + 8)?, rdf(b, o + 12)?))
}

impl BoneAnim {
    /// every anim of a `.gaf` file
    pub fn parse_file(b: &[u8]) -> Result<Vec<Self>, ModelError> {
        if b.get(0..4) != Some(b"0FAG") {
            return Err(ModelError::BadId(rd32(b, 0)?));
        }
        let n = rd32(b, 16)? as usize;
        (0..n).map(|i| Self::parse(b, rd32(b, 20 + 4 * i)? as usize)).collect()
    }

    fn parse(b: &[u8], o: usize) -> Result<Self, ModelError> {
        let nb = rd32(b, o + 164)? as usize;
        let (infos, keys) = (rd32(b, o + 168)? as usize, rd32(b, o + 176)? as usize);
        let mut bones = Vec::with_capacity(nb);
        for i in 0..nb {
            let bi = infos + 12 * i;
            let name_off = rd32(b, bi)? as usize;
            let nk = rd32(b, bi + 8)? as usize;
            let kp = rd32(b, keys + 4 * i)? as usize;
            let mut k = Vec::with_capacity(nk);
            if kp != 0 {
                for j in 0..nk {
                    let mut r = [0u16; 8];
                    for (t, v) in r.iter_mut().enumerate() {
                        *v = rd16(b, kp + 16 * j + 2 * t)?;
                    }
                    k.push(r);
                }
            }
            bones.push(BoneTrack { name: if name_off != 0 { cstr(b, name_off)? } else { String::new() }, active: rd32(b, bi + 4)? != 0, keys: k });
        }
        let table = |cnt: usize, at: usize| -> Result<Vec<Vec4>, ModelError> { (0..cnt).map(|i| vec4(b, at + 16 * i)).collect() };
        Ok(Self {
            name: cstr(b, o)?,
            checksum: rd32(b, o + 128)?,
            fps: rdf(b, o + 132)?,
            duration: rdf(b, o + 136)?,
            num_frames: rd32(b, o + 140)?,
            offset: Vec3::new(rdf(b, o + 144)?, rdf(b, o + 148)?, rdf(b, o + 152)?),
            looping: rd32(b, o + 156)?,
            compressed: rd32(b, o + 160)? != 0,
            bones,
            positions: table(rd32(b, o + 180)? as usize, rd32(b, o + 184)? as usize)?,
            scales: table(rd32(b, o + 188)? as usize, rd32(b, o + 192)? as usize)?,
        })
    }

    /// BoneAnim.cpp 0x002E0FF0 — normalised frame increment for `dt` seconds (fractional part of dt / duration)
    pub fn frame_inc(&self, dt: f32) -> f32 {
        let v = dt as f64 / self.duration as f64;
        let v5 = v as f32;
        (v5 as f64 - (v as i64) as f64) as f32
    }

    /// BoneAnim.cpp 0x002E1710 — the run holding `frame` (binary search over start / run length)
    fn find_run(keys: &[[u16; 8]], frame: i32) -> usize {
        let (mut lo, mut hi) = (0i32, keys.len() as i32 - 1);
        loop {
            let mid = lo + ((hi - lo) >> 1);
            let k = &keys[mid as usize];
            if frame >= k[5] as i32 {
                if frame <= k[6] as i32 + k[5] as i32 {
                    return mid as usize;
                }
                lo = mid + 1;
            } else {
                hi = mid;
            }
            if lo > hi || lo as usize >= keys.len() {
                return mid.max(0) as usize; // past the last run (data never does this)
            }
        }
    }

    /// BoneAnim.cpp 0x002E17A0 — euler angles of a run at `frame` (signed lerp towards the next record)
    fn run_angles(keys: &[[u16; 8]], i: usize, frame: i32) -> Vec3 {
        let a = &keys[i];
        let s = |v: u16| v as i16 as f64;
        if frame == a[5] as i32 {
            return Self::angles(a[0], a[1], a[2]);
        }
        let n = keys.get(i + 1).unwrap_or(a);
        let t = ((frame - a[5] as i32) as u32 as f64 / (a[6] as f64 + 1.0)) as f32;
        let l = |x: u16, y: u16| (s(x) + (s(y) - s(x)) * t as f64) as f32;
        Vec3::new(l(a[0], n[0]) / ANGLE_SCALE, l(a[1], n[1]) / ANGLE_SCALE, l(a[2], n[2]) / ANGLE_SCALE)
    }

    /// BoneAnim.cpp 0x002E1910
    fn angles(x: u16, y: u16, z: u16) -> Vec3 {
        Vec3::new(x as f32 / ANGLE_SCALE, y as f32 / ANGLE_SCALE, z as f32 / ANGLE_SCALE)
    }

    /// BoneAnim.cpp 0x002E11C0 — bone `bone` at normalised time `t` (0..1)
    pub fn sample(&self, bone: usize, t: f32) -> BoneQts {
        let tr = &self.bones[bone];
        let n = self.num_frames as i32;
        let f = if self.looping == 1 { n as f32 * t.min(0.99999) } else { (n - 1) as f32 * t.min(0.99999) };
        let f0 = f as i32;
        let mut f1 = f0 + 1;
        let mut s = f - f0 as f32;
        if f1 >= n {
            if self.looping == 1 {
                f1 = 0;
            } else {
                f1 = f0;
                s = 0.0;
            }
        }
        let k = &tr.keys;
        let (r0, r1, a, b) = if self.compressed {
            let (i0, i1) = (Self::find_run(k, f0), Self::find_run(k, f1));
            (i0, i1, Self::run_angles(k, i0, f0), Self::run_angles(k, i1, f1))
        } else {
            let (i0, i1) = (f0 as usize, f1 as usize);
            (i0, i1, Self::angles(k[i0][0], k[i0][1], k[i0][2]), Self::angles(k[i1][0], k[i1][1], k[i1][2]))
        };
        let p0 = self.positions[k[r0][3] as usize];
        let p1 = self.positions[k[r1][3] as usize];
        let c0 = self.scales[k[r0][4] as usize];
        let c1 = self.scales[k[r1][4] as usize];
        let mut translation = (p1 - p0) * s + p0;
        translation.w = 1.0;
        let scale = (c1 - c0) * s + c0;
        let q = quat_slerp(s, euler_quat(a.x, a.y, a.z), euler_quat(b.x, b.y, b.z));
        BoneQts { rotation: q, scale, translation }
    }
}

/// TimelineSequencer 0x002B7100 — quaternion from the anim's euler angles
pub fn euler_quat(x: f32, y: f32, z: f32) -> Quat {
    let (sy, cy) = (y * 0.5).sin_cos();
    let (sx, cx) = (x * 0.5).sin_cos();
    let (sz, cz) = (z * 0.5).sin_cos();
    let (v14, v9) = (cx * cz, sx * sz);
    Quat::from_xyzw(sy * v14 - cy * v9, cy * sx * cz + sy * cx * sz, cy * cx * sz - sy * sx * cz, cy * v14 + sy * v9)
}

/// TimelineSequencer 0x002B7530 — slerp (shortest arc, linear below 0.001)
pub fn quat_slerp(t: f32, a: Quat, b: Quat) -> Quat {
    let mut d = a.dot(b);
    let neg = d < 0.0;
    if neg {
        d = -d;
    }
    let (k0, mut k1);
    if 1.0 - d >= 0.001 {
        let th = d.acos();
        let s = th.sin();
        k0 = ((1.0 - t) * th).sin() / s;
        k1 = (t * th).sin() / s;
    } else {
        k0 = 1.0 - t;
        k1 = t;
    }
    if neg {
        k1 = -k1;
    }
    Quat::from_xyzw(k0 * a.x + k1 * b.x, k0 * a.y + k1 * b.y, k0 * a.z + k1 * b.z, k0 * a.w + k1 * b.w)
}
