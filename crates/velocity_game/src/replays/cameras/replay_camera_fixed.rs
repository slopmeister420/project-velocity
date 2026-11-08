//! src\Replays\Cameras\ReplayCamera_Fixed.cpp — fixed (and dolly) trackside cameras from the race's
//! `cameras.cam`. Most of the object's code sits in the range the idb labels ZoneRender_PC_XBOX (0x00163930..).
//!
//! `cameras.cam`, little endian: 16-byte header `"FXDCAMPOS"`, +9 u8 version (4; 3 is warned about and read the
//! same), +12 u32 camera count; 64-byte cameras {vec4 position, quat xyzw, f32 max turn angle (deg), f32 fov at
//! the closest / furthest distance (deg), u32 lock height, i32 dolly track, i32 look track, u32 id, pad}; then
//! u32 track count + 12 pad, per track u32 point count + 12 pad and vec4 points.

use bevy::math::{Mat3, Quat, Vec3};

use super::ReplayWorld;
use crate::camera::vs_base_camera::{normalise, quat_matrix};

/// `flt_748E94` (= 250, from 0x0063EDF4) furthest distance a camera may see the car from
pub const MAX_DISTANCE: f32 = 250.0;
/// `dword_68767C`
pub const MIN_DISTANCE: f32 = 10.0;
/// seconds before another camera may be chosen
pub const MIN_SHOT_TIME: f32 = 1.5;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FixedCamera {
    pub position: Vec3,
    pub rotation: Quat,
    pub max_angle: f32,
    pub fov_near: f32,
    pub fov_far: f32,
    pub lock_height: bool,
    pub dolly: Option<usize>,
    pub look: Option<usize>,
    pub id: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CamerasFile {
    pub version: u8,
    pub cameras: Vec<FixedCamera>,
    pub tracks: Vec<Vec<Vec3>>,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum CamerasError {
    #[error("not a fixed camera file")]
    Magic,
    #[error("truncated at {0:#x}")]
    Truncated(usize),
}

fn f(b: &[u8], o: usize) -> f32 {
    f32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn u(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

/// ZoneRender_PC_XBOX 0x00164620 (cameras) and 0x00164410 "ProcessDollyTrackData" (tracks); a file with no
/// track block (a 32-byte empty one) has no tracks
pub fn parse_cameras(b: &[u8]) -> Result<CamerasFile, CamerasError> {
    if b.len() < 16 || &b[0..9] != b"FXDCAMPOS" {
        return Err(CamerasError::Magic);
    }
    let version = b[9];
    if version != 4 {
        log::warn!("Fixed camera file is not current version.  This might go bad");
    }
    let n = u(b, 12) as usize;
    let mut p = 16;
    let mut out = CamerasFile { version, ..Default::default() };
    for _ in 0..n {
        let r = b.get(p..p + 64).ok_or(CamerasError::Truncated(p))?;
        let idx = |o: usize| {
            let v = u(r, o) as i32;
            (v >= 0).then_some(v as usize)
        };
        out.cameras.push(FixedCamera {
            position: Vec3::new(f(r, 0), f(r, 4), f(r, 8)),
            rotation: Quat::from_xyzw(f(r, 16), f(r, 20), f(r, 24), f(r, 28)),
            max_angle: f(r, 32),
            fov_near: f(r, 36),
            fov_far: f(r, 40),
            lock_height: u(r, 44) != 0,
            dolly: idx(48),
            look: idx(52),
            id: u(r, 56),
        });
        p += 64;
    }
    if p + 16 > b.len() {
        if n != 0 {
            return Err(CamerasError::Truncated(p));
        }
        return Ok(out);
    }
    let nt = u(b, p) as usize;
    p += 16;
    for _ in 0..nt {
        let k = u(b, b.get(p..p + 4).map(|_| p).ok_or(CamerasError::Truncated(p))?) as usize;
        p += 16;
        let mut pts = Vec::with_capacity(k);
        for _ in 0..k {
            let r = b.get(p..p + 16).ok_or(CamerasError::Truncated(p))?;
            pts.push(Vec3::new(f(r, 0), f(r, 4), f(r, 8)));
            p += 16;
        }
        out.tracks.push(pts);
    }
    Ok(out)
}

/// ZoneRender_PC_XBOX 0x00163CD0 — point at `t` (0..1, clamped) along a track, linear between points
pub fn track_point(pts: &[Vec3], t: f32) -> Vec3 {
    let t = t.clamp(0.0, 1.0);
    let seg = 1.0 / (pts.len() as u32 - 1) as f64 as f32;
    let i = (t / seg) as u32 as usize;
    let rem = t - i as f32 * seg;
    let mut p = pts[i];
    if i != pts.len() - 1 {
        p += (pts[i + 1] - pts[i]) * (rem / seg);
    }
    p
}

/// ZoneRender_PC_XBOX 0x00163B20 — where along a track `point` is closest (0..1), and that closest point
pub fn track_param(pts: &[Vec3], point: Vec3) -> (f32, Vec3) {
    if pts.is_empty() {
        return (0.0, Vec3::ZERO);
    }
    let mut best = f32::from_bits(0x47C3_4FF3);
    let mut best_i: Option<usize> = None;
    let mut best_t = 0.0f32;
    let mut best_p = pts[0];
    for i in 0..pts.len() - 1 {
        // 0x00163930: closest point on the segment
        let (a, b) = (pts[i], pts[i + 1]);
        let ab = b - a;
        let t = (ab.dot(point - a) / ab.dot(ab)).clamp(0.0, 1.0);
        let c = a + ab * t;
        let d = (point - c).length_squared();
        if best > d {
            best = d;
            best_i = Some(i);
            best_t = t;
            best_p = c;
        }
    }
    match best_i {
        None => (0.0, pts[0]),
        Some(i) => {
            let s = (1.0 / (pts.len() as u32 - 1) as f64) as f32;
            ((i as f64 * s as f64 + (best_t * s) as f64) as f32, best_p)
        }
    }
}

/// `CReplayCameraFixed` (104 bytes at the manager's +0x104)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CReplayCameraFixed {
    pub vehicle: u32,          // +0x00
    pub time: f32,             // +0x04 on the current camera
    pub current: Option<usize>, // +0x08
    pub cut: bool,             // +0x0C
    pub rechoose: bool,        // +0x0D
    pub file: Option<CamerasFile>, // +0x10..+0x20
    pub position: Vec3,        // +0x24
    /// +0x34 the car, then the point looked at
    pub target: Vec3,
    pub up: Vec3,              // +0x44
    pub fov: f32,              // +0x54
    /// +0x58 the camera in use (index into the file)
    pub record: Option<usize>,
    pub loaded: bool,          // +0x64
}

impl CReplayCameraFixed {
    /// ZoneRender_PC_XBOX 0x001645D0 / 0x00164620
    pub fn load(&mut self, file: CamerasFile) {
        if !self.loaded {
            self.loaded = true;
            self.up = Vec3::Y;
            self.file = Some(file);
            self.reset();
        }
    }

    /// ZoneRender_PC_XBOX 0x001647F0
    pub fn unload(&mut self) {
        if self.loaded {
            self.file = None;
            self.loaded = false;
        }
    }

    /// ZoneRender_PC_XBOX 0x001648A0
    pub fn reset(&mut self) {
        self.time = 0.0;
        self.current = None;
        self.rechoose = true;
        self.record = None;
    }

    /// ZoneRender_PC_XBOX 0x001648F0
    pub fn reset_for(&mut self, vehicle: u32) {
        self.reset();
        self.vehicle = vehicle;
    }

    pub(crate) fn count(&self) -> usize {
        self.file.as_ref().map(|f| f.cameras.len()).unwrap_or(0)
    }

    /// ReplayCamera_Fixed.cpp 0x00164EC0 — nearest camera in range that sees the point (any distance and no
    /// line of sight test with `any`)
    pub fn nearest(&self, point: Vec3, any: bool, w: &dyn ReplayWorld) -> (usize, f32) {
        let mut best = (0usize, 1.0e10f32);
        let Some(file) = &self.file else { return best };
        for (i, c) in file.cameras.iter().enumerate() {
            let d = c.position.distance_squared(point);
            if any {
                if best.1 > d {
                    best = (i, d);
                }
            } else if d <= MAX_DISTANCE * MAX_DISTANCE && best.1 > d && !w.blocked(c.position, self.target, self.vehicle) {
                best = (i, d);
            }
        }
        best
    }

    /// ZoneRender_PC_XBOX 0x001649A0 — follow the car, picking the nearest seeing camera every 1.5 s (or when
    /// told); false when there is none
    pub fn choose(&mut self, review: bool, w: &dyn ReplayWorld) -> bool {
        if self.count() == 0 {
            return false;
        }
        self.target = w.car_position(self.vehicle);
        if !(self.time >= MIN_SHOT_TIME || self.rechoose || review) {
            return true;
        }
        self.rechoose = false;
        let (i, d) = self.nearest(self.target, false, w);
        if MAX_DISTANCE * MAX_DISTANCE <= d {
            return false;
        }
        self.position = self.file.as_ref().unwrap().cameras[i].position;
        self.record = Some(i);
        if Some(i) != self.current {
            self.time = 0.0;
            self.cut = true;
            self.current = Some(i);
        }
        true
    }

    /// ReplayCamera_Fixed.cpp 0x00164B80 — use the camera with this id (from a CAMERA_SWITCH trigger)
    pub fn select_by_id(&mut self, id: u32, w: &dyn ReplayWorld) -> bool {
        let Some(i) = self.file.as_ref().and_then(|f| f.cameras.iter().position(|c| c.id == id)) else { return false };
        if self.count() == 0 {
            return false;
        }
        self.target = w.car_position(self.vehicle);
        self.rechoose = false;
        self.record = Some(i);
        self.position = self.file.as_ref().unwrap().cameras[i].position;
        if Some(i) != self.current {
            self.time = 0.0;
            self.cut = true;
            self.current = Some(i);
        }
        true
    }

    /// ZoneRender_PC_XBOX 0x00164920 — per update; false = the camera lost the car
    pub fn update(&mut self, dt: f32, review: bool, w: &dyn ReplayWorld) -> bool {
        if !self.loaded {
            return true;
        }
        self.time += dt;
        self.choose(review, w)
    }

    /// ZoneRender_PC_XBOX 0x00163E80 — slide along the dolly, turn towards the car within the camera's limit
    /// and zoom with the distance
    pub fn aim(&mut self, dt: f32, w: &dyn ReplayWorld) -> bool {
        let Some(ri) = self.record else { return false };
        let file = self.file.as_mut().unwrap();
        let cam = file.cameras[ri].clone();
        let track = cam.look.or(cam.dolly);
        let mut t = 0.0;
        if let Some(tr) = track.and_then(|i| file.tracks.get(i)) {
            t = track_param(tr, self.target).0;
        }
        if let Some(d) = cam.dolly.and_then(|i| file.tracks.get(i)) {
            let p = track_point(d, t);
            if self.cut {
                file.cameras[ri].position = p;
            } else {
                let mut k = 0.9 / (dt / 0.02);
                if k > 1.0 {
                    k = 1.0;
                }
                file.cameras[ri].position = p * (1.0 - k) + file.cameras[ri].position * k;
            }
        }
        let pos = file.cameras[ri].position;
        let base = normalise(quat_matrix(cam.rotation) * Vec3::NEG_Z);
        let to = normalise(self.target - pos);
        let mut ang = base.dot(to).clamp(-1.0, 1.0).acos();
        let lim = f32::from_bits(0x3C8E_FA35) * cam.max_angle;
        if ang > lim {
            ang = lim;
        }
        let axis = normalise(normalise(base - to).cross(base));
        let mut look = Mat3::from_axis_angle(axis, ang) * base;
        if !look.is_finite() {
            look = base;
        }
        if cam.lock_height {
            look.y = base.y;
        }
        self.target = pos + look;
        self.position = pos;
        let car = w.car_position(self.vehicle);
        let d = (car - pos).length().min(MAX_DISTANCE).max(MIN_DISTANCE);
        let x = (d - MIN_DISTANCE) / (MAX_DISTANCE - MIN_DISTANCE);
        let s = (x * x) * 3.5 - ((x * x) * x) * 2.52;
        let near = cam.fov_near * f32::from_bits(0x3C8E_FA35);
        let far = cam.fov_far * f32::from_bits(0x3C8E_FA35);
        self.fov = (1.0 - s) * (far - near) + near;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cameras_cam_all() {
        if visos::file::platform::try_game_dir().is_err() {
            return;
        }
        let root = visos::file::platform::resolve("\\Tracks");
        let mut files = Vec::new();
        fn walk(d: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(d).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("cameras.cam")) {
                    out.push(p);
                }
            }
        }
        walk(&root, &mut files);
        assert_eq!(files.len(), 65);
        let mut ok = 0;
        let mut bad = Vec::new();
        for f in &files {
            match parse_cameras(&std::fs::read(f).unwrap()) {
                Ok(c) => {
                    ok += 1;
                    for cam in &c.cameras {
                        assert!(cam.dolly.is_none_or(|i| i < c.tracks.len()), "{}", f.display());
                        assert!(cam.look.is_none_or(|i| i < c.tracks.len()), "{}", f.display());
                    }
                }
                Err(e) => bad.push((f.display().to_string(), e)),
            }
        }
        // two shipped files are cut short (London Circuit Race04, London Drag Race01)
        assert_eq!(bad.len(), 2, "{bad:?}");
        assert_eq!(ok, 63);
    }

    #[test]
    fn track_projection() {
        let pts = [Vec3::ZERO, Vec3::X * 10.0, Vec3::new(10.0, 0.0, 10.0)];
        let (t, p) = track_param(&pts, Vec3::new(5.0, 3.0, 0.0));
        assert!((t - 0.25).abs() < 1e-6 && (p - Vec3::X * 5.0).length() < 1e-6);
        assert!((track_point(&pts, 0.75) - Vec3::new(10.0, 0.0, 5.0)).length() < 1e-5);
    }
}
