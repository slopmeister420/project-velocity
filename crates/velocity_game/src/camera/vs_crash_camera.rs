//! src\Camera\VsCrashCamera.cpp — the crash camera: when a player car hits something head-on it swings round the
//! car on a Catmull-Rom spline for a couple of seconds (with the time slowed down), then hands the view back.
//! It drives the player's fly camera (`VsFlyCamera`, manual mode), the camera manager switches to it.

use bevy::math::{Mat3, Vec3};
use visos::handy::crt_rand::CrtRand;

use crate::game_mode::update_thread::CWorldModeManager;

/// WMM mode of the crash camera
pub const WMM_CRASH_CAM: i32 = 2;

/// "Camera.Crash" tweaks (VsCrashCamera.cpp 0x00061D30); values as shipped
#[derive(Clone, Debug, PartialEq)]
pub struct CrashCameraTweaks {
    pub duration: f32,              // dword_6850B4 2.0
    pub duration_failure: f32,      // dword_6850B8 5.0
    pub speed_rate: f32,            // flt_6850C4 0.2 (time stretch while running)
    pub recover_time: f32,          // dword_6850BC 2.5
    pub camera_distance: f32,       // s 5.0
    pub start_angle: f32,           // dword_6850CC 30 degrees
    pub angle_span: f32,            // flt_6850D0 90 degrees
    pub height_variation_speed: f32, // flt_6850D4 2.0 (spline points per radian of span)
    pub height_low: f32,            // flt_6850D8 2.0
    pub height_high: f32,           // flt_6850DC 8.1
    pub target_direction_offset: f32, // flt_6850E0 2.5
    pub target_speed_offset: f32,   // flt_6850E4 0.01
    pub use_raceline: bool,         // byte_6D58A0 (bss)
    pub use_rays: bool,             // byte_6D58A1 (bss)
    pub respawn_on_raceline: bool,  // byte_6850E8 1
    pub respawn_max_speed: f32,     // flt_6850EC 20
    pub respawn_kick_speed: f32,    // flt_6850F0 40
    /// byte_6850C0 1: cinematic bars on the OSD while it runs
    pub cinematic_bars: bool,
}

impl Default for CrashCameraTweaks {
    fn default() -> Self {
        CrashCameraTweaks {
            duration: 2.0,
            duration_failure: 5.0,
            speed_rate: f32::from_bits(0x3E4C_CCCD),
            recover_time: 2.5,
            camera_distance: 5.0,
            start_angle: 30.0,
            angle_span: 90.0,
            height_variation_speed: 2.0,
            height_low: 2.0,
            height_high: f32::from_bits(0x4101_999A),
            target_direction_offset: 2.5,
            target_speed_offset: f32::from_bits(0x3C23_D70A),
            use_raceline: false,
            use_rays: false,
            respawn_on_raceline: true,
            respawn_max_speed: 20.0,
            respawn_kick_speed: 40.0,
            cinematic_bars: true,
        }
    }
}

/// `VsCatmullRom` (VsCrashCamera.cpp 0x000614F0, 8 bytes): points, end tangents mirrored.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VsCatmullRom {
    pub pts: Vec<Vec3>,
}

impl VsCatmullRom {
    /// VsCrashCamera.cpp 0x000614F0
    pub fn new(n: usize) -> Self {
        if n <= 1 {
            log::error!("assert: m_npts > 1");
        }
        VsCatmullRom { pts: vec![Vec3::ZERO; n] }
    }

    /// VsCrashCamera.cpp 0x00061620
    pub fn set(&mut self, i: usize, p: Vec3) {
        self.pts[i] = p;
    }

    /// VsCrashCamera.cpp 0x00061710 — point at `t` in 0..1
    pub fn eval(&self, t: f32) -> Vec3 {
        let n = self.pts.len();
        let eq = |a: f32, b: f32| (a - b).abs() <= 1e-6;
        if eq(t, 0.0) {
            return self.pts[0];
        }
        if eq(t, 1.0) {
            return self.pts[n - 1];
        }
        let start = ((n as i32 - 1) as f32 * t) as i32;
        if start < 0 || start >= n as i32 - 1 {
            log::error!("assert: startidx >= 0 && startidx < m_npts-1");
        }
        let s = start as usize;
        let p0 = if s != 0 { self.pts[s - 1] } else { self.pts[0] * 2.0 - self.pts[1] };
        let p1 = self.pts[s];
        let p2 = self.pts[s + 1];
        // original bug kept: the fourth control point is pts[s + 1] again (its mirror branch for
        // startidx == npts-1 cannot run)
        let p3 = if s == n - 1 { self.pts[n - 1] * 2.0 - self.pts[n - 1] } else { self.pts[s + 1] };
        // note: the original passes the whole-curve t, not the segment's local parameter (kept)
        let t2 = t * t;
        let t3 = t2 * t;
        let b0 = ((-0.5 * t3) + (1.0 * t2)) - (0.5 * t) + 0.0;
        let b1 = ((1.5 * t3) - (2.5 * t2)) + (0.0 * t) + 1.0;
        let b2 = ((-1.5 * t3) + (2.0 * t2)) + (0.5 * t) + 0.0;
        let b3 = ((0.5 * t3) - (0.5 * t2)) + (0.0 * t) + 0.0;
        p0 * b0 + p1 * b1 + p2 * b2 + p3 * b3
    }
}

/// the crashed car as the crash camera reads it (VsAIRaceVehicle / VehicleEntity fields).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CrashVehicle {
    pub id: u32,          // +0x78
    pub velocity: Vec3,   // +0x80
    pub speed: f32,       // +0x90
    pub position: Vec3,   // +0x98
    /// +0xC8 side vector (points to the car's left)
    pub side: Vec3,
    /// +0x2C0 / +0x2C4 lateral raceline values compared for the side
    pub raceline_a: f32,
    pub raceline_b: f32,
}

/// what the crash camera asks of the game while running.
pub trait CrashCameraWorld {
    /// the race clock (`GameManager+0x208`)
    fn race_time(&self) -> f32;
    /// the fly camera's current position for `player` (its `VsBaseCamera::pos`)
    fn camera_position(&self, player: u32) -> Vec3;
    /// MaterialProperties.cpp 0x0019D350: first hit between two points, as a fraction
    fn ray_fraction(&self, from: Vec3, to: Vec3, vehicle: u32) -> Option<f32>;
}

/// side effects for the camera manager / OSD / physics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrashCameraEvent {
    /// the player's view switches to the fly camera (mode 3) and back (mode 1); `cut` = render cut
    SetCameraMode { player: u32, mode: u32 },
    CinematicBars(bool),
    /// VsAIRaceVehicle.cpp 0x000208D0: put the car back on the raceline at this speed
    Respawn { vehicle: u32, kick_speed: f32 },
}

/// `VsCrashCamera` (32 bytes)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VsCrashCamera {
    /// +0x00 the fly camera's player (None until assigned)
    pub camera: Option<u32>,
    /// +0x04 the car being watched
    pub vehicle: Option<CrashVehicle>,
    pub start_time: f32,   // +0x08
    pub end_time: f32,     // +0x0C
    /// +0x10 no new crash camera before this time
    pub recover_until: f32,
    /// +0x14 started as a failure (5 s)
    pub failure: bool,
    /// +0x18 the camera swings round on the car's left
    pub left_side: bool,
    pub spline: Option<VsCatmullRom>, // +0x1C
    /// fly camera target (+0x1F0) and position (+0xD4) this camera writes
    pub cam_target: Vec3,
    pub cam_pos: Vec3,
    /// fly camera mode (+0x208) this camera set
    pub cam_mode: u32,
}

impl VsCrashCamera {
    /// VsCrashCamera.cpp 0x00062190
    pub fn reset(&mut self, tw: &CrashCameraTweaks) {
        self.vehicle = None;
        self.start_time = 0.0;
        self.end_time = 0.0;
        self.recover_until = tw.recover_time;
        self.failure = false;
        self.spline = None;
    }

    /// VsCrashCameraManager.cpp 0x00063F00
    pub fn is_active(&self) -> bool {
        self.vehicle.is_some()
    }

    /// VsCrashCamera.cpp 0x00062410 — time stretch
    pub fn speed_rate(&self, tw: &CrashCameraTweaks) -> f32 {
        if self.vehicle.is_some() && self.camera.is_some() { tw.speed_rate } else { 1.0 }
    }

    /// VsCrashCamera.cpp 0x00062260 — start on `v` (`point` = impact point, `kind` 2 = failure)
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &mut self,
        v: CrashVehicle,
        point: Vec3,
        kind: u32,
        tw: &CrashCameraTweaks,
        w: &dyn CrashCameraWorld,
        wmm: &mut CWorldModeManager,
        rand: &mut CrtRand,
        out: &mut Vec<CrashCameraEvent>,
    ) -> bool {
        if self.vehicle.is_some() {
            return false;
        }
        let now = w.race_time();
        if self.recover_until > now {
            return false;
        }
        if !wmm.request(WMM_CRASH_CAM, "Crash camera", now) {
            return false;
        }
        self.left_side = side_of(&v, point, tw);
        self.vehicle = Some(v);
        self.failure = kind == 2;
        let d = if kind == 2 { tw.duration_failure } else { tw.duration };
        self.begin(d, tw, w, rand, out);
        true
    }

    /// VsCrashCamera.cpp 0x00062C10
    fn begin(&mut self, duration: f32, tw: &CrashCameraTweaks, w: &dyn CrashCameraWorld, rand: &mut CrtRand, out: &mut Vec<CrashCameraEvent>) {
        log::info!("Crash cam {:?} starts", self.camera);
        self.start_time = w.race_time();
        self.end_time = self.start_time + duration;
        let Some(player) = self.camera else { return };
        let v = self.vehicle.unwrap();
        out.push(CrashCameraEvent::SetCameraMode { player, mode: 3 });
        if tw.cinematic_bars {
            out.push(CrashCameraEvent::CinematicBars(true));
        }
        self.spline = Some(make_spline(&v, self.left_side, tw, rand));
        self.cam_target = v.position + v.side * tw.target_direction_offset;
        self.cam_pos = w.camera_position(v.id);
    }

    /// VsCrashCamera.cpp 0x00062870 — per update
    pub fn update(&mut self, tw: &CrashCameraTweaks, w: &dyn CrashCameraWorld, wmm: &mut CWorldModeManager, out: &mut Vec<CrashCameraEvent>) {
        let Some(v) = self.vehicle else { return };
        let now = w.race_time();
        if now >= self.end_time {
            self.stop(tw, w, wmm, out);
            return;
        }
        if self.camera.is_none() {
            return;
        }
        self.cam_mode = 0;
        self.cam_target = v.position + v.side * tw.target_direction_offset + v.velocity * tw.target_speed_offset;
        let t = (now - self.start_time) / (self.end_time - self.start_time);
        let p = self.spline.as_ref().map(|s| s.eval(t)).unwrap_or(Vec3::ZERO);
        let mut pos = self.cam_target + p;
        pos = pos * 0.5 + self.cam_pos * 0.5;
        if tw.use_rays {
            if let Some(f) = w.ray_fraction(self.cam_target, pos, v.id) {
                pos = self.cam_target + (pos - self.cam_target) * f;
            }
        }
        self.cam_pos = pos;
    }

    /// VsCrashCamera.cpp 0x00062D90
    pub fn stop(&mut self, tw: &CrashCameraTweaks, w: &dyn CrashCameraWorld, wmm: &mut CWorldModeManager, out: &mut Vec<CrashCameraEvent>) {
        log::info!("Crash cam {:?} stops", self.camera);
        let now = w.race_time();
        if let Some(player) = self.camera {
            out.push(CrashCameraEvent::SetCameraMode { player, mode: 1 });
            if tw.cinematic_bars {
                out.push(CrashCameraEvent::CinematicBars(false));
            }
        }
        self.recover_until = now + tw.recover_time;
        let v = self.vehicle.take();
        self.spline = None;
        if let Some(v) = v {
            if self.failure || (tw.respawn_on_raceline && tw.respawn_max_speed >= v.speed) {
                out.push(CrashCameraEvent::Respawn { vehicle: v.id, kick_speed: tw.respawn_kick_speed });
            }
        }
        wmm.leave(WMM_CRASH_CAM, tw.recover_time, now);
    }

    /// VsCrashCamera.cpp 0x000623D0 — stop if running
    pub fn cancel(&mut self, tw: &CrashCameraTweaks, w: &dyn CrashCameraWorld, wmm: &mut CWorldModeManager, out: &mut Vec<CrashCameraEvent>) {
        if self.vehicle.is_some() {
            self.stop(tw, w, wmm, out);
        }
        self.vehicle = None;
    }
}

/// VsCrashCamera.cpp 0x00062450 — which side the camera swings round
fn side_of(v: &CrashVehicle, point: Vec3, tw: &CrashCameraTweaks) -> bool {
    if tw.use_raceline {
        v.raceline_b > v.raceline_a
    } else {
        crate::camera::vs_crash_camera::is_left_of_line(v.position, v.position + v.side, point)
    }
}

/// VsAITools::IsLeftOfLine (xz plane)
pub fn is_left_of_line(a: Vec3, b: Vec3, p: Vec3) -> bool {
    (b.x - a.x) * (p.z - a.z) - (b.z - a.z) * (p.x - a.x) > 0.0
}

/// VsCrashCamera.cpp 0x00062590 — the spline round the car (relative to the target)
fn make_spline(v: &CrashVehicle, left: bool, tw: &CrashCameraTweaks, rand: &mut CrtRand) -> VsCatmullRom {
    let mut n = ((tw.angle_span * std::f32::consts::PI / 180.0) as f64 * tw.height_variation_speed as f64) as i32 + 1;
    if n < 2 {
        n = 2;
    }
    log::info!("Crash camera spline has {n} points");
    let mut s = VsCatmullRom::new(n as usize);
    let sign = if left { 1.0 } else { -1.0 };
    for i in 0..n {
        let t = (i + 1) as f32 / n as f32;
        let ang = -(((tw.angle_span * t) + tw.start_angle) * sign).to_radians();
        let p = Mat3::from_rotation_y(ang) * v.side;
        let mut pt = -p * tw.camera_distance;
        let h = rand.rand_unit() as f32 * (tw.height_high - tw.height_low) + tw.height_low;
        pt += Vec3::Y * h;
        s.set(i as usize, pt);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    struct W(f32);
    impl CrashCameraWorld for W {
        fn race_time(&self) -> f32 {
            self.0
        }
        fn camera_position(&self, _: u32) -> Vec3 {
            Vec3::new(0.0, 2.0, -5.0)
        }
        fn ray_fraction(&self, _: Vec3, _: Vec3, _: u32) -> Option<f32> {
            None
        }
    }

    #[test]
    fn crash_camera_runs_and_stops() {
        let tw = CrashCameraTweaks::default();
        let mut c = VsCrashCamera { camera: Some(0), ..Default::default() };
        c.reset(&tw);
        let mut wmm = CWorldModeManager::default();
        let mut rand = CrtRand::default();
        let mut out = Vec::new();
        let v = CrashVehicle { side: Vec3::X, speed: 35.0, ..Default::default() };
        assert!(!c.start(v, Vec3::Z, 1, &tw, &W(1.0), &mut wmm, &mut rand, &mut out), "recover time");
        assert!(c.start(v, Vec3::Z, 1, &tw, &W(10.0), &mut wmm, &mut rand, &mut out));
        assert_eq!(c.spline.as_ref().unwrap().pts.len(), 4);
        c.update(&tw, &W(11.0), &mut wmm, &mut out);
        assert!(c.is_active() && c.cam_pos.y > 0.0);
        c.update(&tw, &W(12.5), &mut wmm, &mut out);
        assert!(!c.is_active());
        assert_eq!(out.first(), Some(&CrashCameraEvent::SetCameraMode { player: 0, mode: 3 }));
        assert!(out.contains(&CrashCameraEvent::SetCameraMode { player: 0, mode: 1 }));
        assert_eq!(wmm.mode, 0);
    }
}
