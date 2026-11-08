//! FastCar `FCMesh` — the level collision mesh (`Gfx\arena.msh`), plus the triangle tests the
//! game runs against it (`FCIntersectRayTriangle`, `FCTriangleBoxOverlap`).
//!
//! File = the 32-byte object image followed by its arrays, all LE:
//! `u32 numVerts, u32 numFaces, u32 field_08, u32 shortFaces, ptr verts, ptr faces32, ptr faces16, ptr normals`
//! (pointers are stale exporter values; `normals != 0` means normals follow), then `f32[3] * numVerts`,
//! faces (`u16[4]` if shortFaces else `u32[4]`: three vertex indices + attribute), then `f32[3] * numFaces`.
//! Face attribute: low byte = surface type (`eSURF_*`), bits 8-15 = 1-based physics zone index.

use super::mem::{cross, sub, V3};
use super::vsmath::{dot87, fabs, sqrt};

#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    #[error("arena.msh truncated: need {need} bytes, have {have}")]
    Truncated { need: usize, have: usize },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Debug, Default)]
pub struct FcMesh {
    pub num_verts: u32,  // +0x00
    pub num_faces: u32,  // +0x04
    pub field_08: u32,   // +0x08 (never read)
    pub short_faces: u32, // +0x0C
    pub verts: Vec<V3>,  // +0x10
    pub faces32: Vec<[u32; 4]>, // +0x14
    pub faces16: Vec<[u16; 4]>, // +0x18
    pub normals: Vec<V3>, // +0x1C
}

fn rd_u32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn rd_f32(b: &[u8], o: usize) -> f32 {
    f32::from_bits(rd_u32(b, o))
}

/// sequential reader standing in for `FILE_Read(handle, dst, offset, size, &read)`
pub(crate) struct Reader<'a> {
    pub b: &'a [u8],
    pub pos: usize,
}
impl<'a> Reader<'a> {
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], MeshError> {
        if self.pos + n > self.b.len() {
            return Err(MeshError::Truncated { need: self.pos + n, have: self.b.len() });
        }
        let s = &self.b[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
}

fn read_v3s(s: &[u8]) -> Vec<V3> {
    s.chunks_exact(12).map(|c| [rd_f32(c, 0), rd_f32(c, 4), rd_f32(c, 8)]).collect()
}

impl FcMesh {
    /// (FastCar) 0x00014040
    pub fn new() -> Self {
        Self::default()
    }

    /// game path of a track's mesh (`<track dir>gfx\arena.msh`, built in `PhysicsManager::Init` 0x0019CA80)
    pub fn track_path(track: &str) -> String {
        format!(r"\Tracks\{track}\gfx\arena.msh")
    }

    /// load through the game-dir resolver (`\Tracks\<name>\Gfx\arena.msh`)
    pub fn open(game_path: &str) -> Result<Self, MeshError> {
        Self::load(&visos::file::platform::read(game_path)?)
    }

    /// (FastCar) 0x0019C460
    pub fn load(bytes: &[u8]) -> Result<Self, MeshError> {
        let mut r = Reader { b: bytes, pos: 0 };
        let h = r.take(32)?;
        let mut m = FcMesh {
            num_verts: rd_u32(h, 0),
            num_faces: rd_u32(h, 4),
            field_08: rd_u32(h, 8),
            short_faces: rd_u32(h, 12),
            ..Default::default()
        };
        let normals_stored = rd_u32(h, 28);
        m.verts = read_v3s(r.take(12 * m.num_verts as usize)?);
        if m.short_faces != 0 {
            let s = r.take(8 * m.num_faces as usize)?;
            m.faces16 = s
                .chunks_exact(8)
                .map(|c| std::array::from_fn(|k| u16::from_le_bytes([c[2 * k], c[2 * k + 1]])))
                .collect();
        } else {
            let s = r.take(16 * m.num_faces as usize)?;
            m.faces32 = s.chunks_exact(16).map(|c| std::array::from_fn(|k| rd_u32(c, 4 * k))).collect();
        }
        if normals_stored != 0 {
            m.normals = read_v3s(r.take(12 * m.num_faces as usize)?);
        } else {
            m.compute_normals();
        }
        Ok(m)
    }

    /// vertex index of corner `c` (0..3) of `face`
    #[inline]
    pub fn index(&self, face: u32, c: usize) -> u32 {
        if self.short_faces != 0 {
            self.faces16[face as usize][c] as u32
        } else {
            self.faces32[face as usize][c]
        }
    }

    /// FCMesh::GetVertex 0x0011F280 (CollisionMeshRender.cpp, inline accessor)
    #[inline]
    pub fn vertex(&self, face: u32, c: usize) -> V3 {
        self.verts[self.index(face, c) as usize]
    }

    /// the three corners of a face
    pub fn triangle(&self, face: u32) -> [V3; 3] {
        [self.vertex(face, 0), self.vertex(face, 1), self.vertex(face, 2)]
    }

    /// raw attribute word (+6 u16 / +12 u32), as `VehicleEntity::WheelCollision` reads it
    #[inline]
    pub fn attribute(&self, face: u32) -> u32 {
        if self.short_faces != 0 {
            self.faces16[face as usize][3] as u32
        } else {
            self.faces32[face as usize][3]
        }
    }

    /// surface type (`eSURF_*`), low byte of the attribute
    #[inline]
    pub fn surface(&self, face: u32) -> u32 {
        self.attribute(face) & 0xFF
    }

    /// 1-based physics zone index (0 = none), bits 8-15 of the attribute
    #[inline]
    pub fn zone(&self, face: u32) -> u32 {
        (self.attribute(face) & 0xFF00) >> 8
    }

    #[inline]
    pub fn normal(&self, face: u32) -> V3 {
        self.normals[face as usize]
    }

    /// (FastCar) 0x00356A90 — returns the number of degenerate faces (normal left unnormalised)
    pub fn compute_normals(&mut self) -> u32 {
        let mut degenerate = 0;
        self.normals = vec![[0.0; 3]; self.num_faces as usize];
        for f in 0..self.num_faces {
            let e1 = vec_sub(self.vertex(f, 1), self.vertex(f, 0));
            let e2 = vec_sub(self.vertex(f, 2), self.vertex(f, 0));
            let n = vec_cross(e1, e2);
            self.normals[f as usize] = n;
            if vec_length(n) >= 0.000001f32 as f64 {
                self.normals[f as usize] = vec_normalize(n);
            } else {
                degenerate += 1;
            }
        }
        degenerate
    }
}

/// (FastCar) 0x00356C20
#[inline]
pub fn vec_sub(a: V3, b: V3) -> V3 {
    sub(a, b)
}
/// (FastCar) 0x00356C70
#[inline]
pub fn vec_cross(a: V3, b: V3) -> V3 {
    cross(a, b)
}
/// (FastCar) 0x00356D30
#[inline]
pub fn vec_length_sq(a: V3) -> f64 {
    dot87(a, a)
}
/// (FastCar) 0x00356D00 — `VsSqrt` of the float-rounded square, left on st0
#[inline]
pub fn vec_length(a: V3) -> f64 {
    sqrt(vec_length_sq(a) as f32)
}
/// (FastCar) 0x00356DF0
#[inline]
pub fn vec_scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
/// (FastCar) 0x00356D60
pub fn vec_normalize(a: V3) -> V3 {
    let len = vec_length(a) as f32;
    if len == 0.0 {
        [1.0, 0.0, 0.0]
    } else {
        vec_scale(a, 1.0 / len)
    }
}

/// result of [`intersect_ray_triangle`]: `t` along `dir`, barycentrics `u` (v1) and `v` (v2)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
    pub t: f32,
    pub u: f32,
    pub v: f32,
}

/// FCIntersectRayTriangle (FastCar) 0x00383540 — Möller-Trumbore, two-sided, `dir` unnormalised.
/// The original writes `u` (and `v`) through its out pointers before rejecting; `partial` gets them.
pub fn intersect_ray_triangle_raw(v0: V3, v1: V3, v2: V3, orig: V3, dir: V3, partial: &mut RayHit) -> bool {
    let e1 = sub(v1, v0);
    let e2 = sub(v2, v0);
    let p = cross(dir, e2);
    let det = dot87(e1, p) as f32;
    if det > -0.000001f32 && 0.000001f32 > det {
        return false;
    }
    let s = sub(orig, v0);
    let inv = 1.0f32 / det;
    partial.u = (dot87(s, p) * inv as f64) as f32;
    if 0.0 > partial.u || partial.u > 1.0 {
        return false;
    }
    let q = cross(s, e1);
    partial.v = (dot87(dir, q) * inv as f64) as f32;
    if 0.0 > partial.v || partial.u + partial.v > 1.0 {
        return false;
    }
    partial.t = (dot87(e2, q) * inv as f64) as f32;
    true
}

/// [`intersect_ray_triangle_raw`] returning the hit
pub fn intersect_ray_triangle(v0: V3, v1: V3, v2: V3, orig: V3, dir: V3) -> Option<RayHit> {
    let mut h = RayHit { t: 0.0, u: 0.0, v: 0.0 };
    intersect_ray_triangle_raw(v0, v1, v2, orig, dir, &mut h).then_some(h)
}

/// FCPlaneBoxOverlap (FastCar) 0x003801F0 — plane `n.x + d = 0` vs box of half extents `h` at the origin
pub fn plane_box_overlap(n: V3, d: f32, h: V3) -> bool {
    let mut vmin = [0.0f32; 3];
    let mut vmax = [0.0f32; 3];
    for q in 0..3 {
        if n[q] > 0.0 {
            vmin[q] = 0.0 - h[q];
            vmax[q] = h[q];
        } else {
            vmin[q] = h[q];
            vmax[q] = 0.0 - h[q];
        }
    }
    if dot87(n, vmin) + d as f64 > 0.0 {
        return false;
    }
    dot87(n, vmax) + d as f64 > 0.0
}

#[inline]
fn min_max3(a: f32, b: f32, c: f32) -> (f32, f32) {
    let mut mx = a;
    let mut mn = a;
    if a > b {
        mn = b;
    }
    if b > mx {
        mx = b;
    }
    if mn > c {
        mn = c;
    }
    if c > mx {
        mx = c;
    }
    (mn, mx)
}

/// `p0 <= p1 ? (p0, p1) : (p1, p0)` as the separating-axis tests order them
#[inline]
fn min_max2(lo_if_le: f32, other: f32) -> (f32, f32) {
    if lo_if_le <= other {
        (lo_if_le, other)
    } else {
        (other, lo_if_le)
    }
}

#[inline]
fn axis_ok(mn: f32, mx: f32, rad: f32) -> bool {
    mn <= rad && 0.0 - rad <= mx
}

/// FCTriangleBoxOverlap (FastCar) 0x00380330 — Akenine-Möller triangle/AABB overlap, box at `c` with half extents `h`
pub fn triangle_box_overlap(a: V3, b: V3, cc: V3, c: V3, h: V3) -> bool {
    let v0 = sub(a, c);
    let v1 = sub(b, c);
    let v2 = sub(cc, c);
    for k in 0..3 {
        let (mn, mx) = min_max3(v0[k], v1[k], v2[k]);
        if !(mn <= h[k] && 0.0 - h[k] <= mx) {
            return false;
        }
    }
    let e0 = sub(v1, v0);
    let e1 = sub(v2, v1);
    let n = cross(e0, e1);
    let d = -dot87(n, v0) as f32;
    if !plane_box_overlap(n, d, h) {
        return false;
    }
    let e2 = sub(v0, v2);

    // edge 0
    let (fex, fey, fez) = (fabs(e0[0]) as f32, fabs(e0[1]) as f32, fabs(e0[2]) as f32);
    let (mn, mx) = min_max2(e0[2] * v2[1] - e0[1] * v2[2], e0[2] * v0[1] - e0[1] * v0[2]);
    if !axis_ok(mn, mx, fez * h[1] + fey * h[2]) {
        return false;
    }
    let (mn, mx) = min_max2((0.0 - e0[2]) * v2[0] + e0[0] * v2[2], (0.0 - e0[2]) * v0[0] + e0[0] * v0[2]);
    if !axis_ok(mn, mx, fez * h[0] + fex * h[2]) {
        return false;
    }
    let (mn, mx) = min_max2(e0[1] * v1[0] - e0[0] * v1[1], e0[1] * v2[0] - e0[0] * v2[1]);
    if !axis_ok(mn, mx, fey * h[0] + fex * h[1]) {
        return false;
    }

    // edge 1
    let (fex, fey, fez) = (fabs(e1[0]) as f32, fabs(e1[1]) as f32, fabs(e1[2]) as f32);
    let (mn, mx) = min_max2(e1[2] * v2[1] - e1[1] * v2[2], e1[2] * v0[1] - e1[1] * v0[2]);
    if !axis_ok(mn, mx, fez * h[1] + fey * h[2]) {
        return false;
    }
    let (mn, mx) = min_max2((0.0 - e1[2]) * v2[0] + e1[0] * v2[2], (0.0 - e1[2]) * v0[0] + e1[0] * v0[2]);
    if !axis_ok(mn, mx, fez * h[0] + fex * h[2]) {
        return false;
    }
    let (mn, mx) = min_max2(e1[1] * v1[0] - e1[0] * v1[1], e1[1] * v0[0] - e1[0] * v0[1]);
    if !axis_ok(mn, mx, fey * h[0] + fex * h[1]) {
        return false;
    }

    // edge 2
    let (fex, fey, fez) = (fabs(e2[0]) as f32, fabs(e2[1]) as f32, fabs(e2[2]) as f32);
    let (mn, mx) = min_max2(e2[2] * v1[1] - e2[1] * v1[2], e2[2] * v0[1] - e2[1] * v0[2]);
    if !axis_ok(mn, mx, fez * h[1] + fey * h[2]) {
        return false;
    }
    let (mn, mx) = min_max2((0.0 - e2[2]) * v1[0] + e2[0] * v1[2], (0.0 - e2[2]) * v0[0] + e2[0] * v0[2]);
    if !axis_ok(mn, mx, fez * h[0] + fex * h[2]) {
        return false;
    }
    let (mn, mx) = min_max2(e2[1] * v1[0] - e2[0] * v1[1], e2[1] * v2[0] - e2[0] * v2[1]);
    axis_ok(mn, mx, fey * h[0] + fex * h[1])
}
