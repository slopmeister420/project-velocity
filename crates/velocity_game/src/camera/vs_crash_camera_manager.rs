//! src\Camera\VsCrashCameraManager.cpp — `g_CrashCameraManager` (0x006D58B0): one crash camera per player, fed by
//! the collision message; it decides whether an impact deserves the crash camera.

use bevy::math::Vec3;
use visos::handy::crt_rand::CrtRand;

use super::vs_crash_camera::{CrashCameraEvent, CrashCameraTweaks, CrashCameraWorld, CrashVehicle, VsCrashCamera};
use crate::game_mode::update_thread::CWorldModeManager;

/// "Camera.Crash" trigger tweaks (VsCrashCamera.cpp 0x00063160); values as shipped
#[derive(Clone, Debug, PartialEq)]
pub struct CrashTriggerTweaks {
    /// byte_6D58A8 (bss): the crash camera is off unless switched on in the debug menu
    pub enabled: bool,
    pub ignore_first_seconds: f32,   // flt_6850F4 2.0
    pub min_impact_speed_car: f32,   // dword_6850FC 33
    pub min_impact_speed_level: f32, // dword_685100 33
    pub min_x_angle_in_air: f32,     // flt_685108 15 degrees
    pub min_z_angle_in_air: f32,     // flt_68510C 15 degrees
    pub car_car: bool,               // byte_6850F8 1
    pub car_level: bool,             // byte_6850F9 1
    pub all_wheels_on_ground: bool,  // byte_685104 1
    pub min_vehicle_speed: f32,      // flt_685110 30
    pub max_forward_angle: f32,      // flt_685114 30 degrees
}

impl Default for CrashTriggerTweaks {
    fn default() -> Self {
        CrashTriggerTweaks {
            enabled: false,
            ignore_first_seconds: 2.0,
            min_impact_speed_car: 33.0,
            min_impact_speed_level: 33.0,
            min_x_angle_in_air: 15.0,
            min_z_angle_in_air: 15.0,
            car_car: true,
            car_level: true,
            all_wheels_on_ground: true,
            min_vehicle_speed: 30.0,
            max_forward_angle: 30.0,
        }
    }
}

/// a race car as the trigger test reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CrashCandidate {
    pub vehicle: CrashVehicle,
    /// +0x34C (VsAIDriver 0x00015D30): driven by a player
    pub human: bool,
    /// +0xFD finished the race
    pub finished: bool,
    /// physics wheels touching the ground (0x000247C0)
    pub wheels_on_ground: i32,
    /// 0x00024820
    pub in_air: bool,
    /// car matrix rows (VehicleEntity 0x001C66C0 / 0x001C6630)
    pub forward: Vec3,
    pub up: Vec3,
}

/// the collision message (`MessageManager` type 0, handled at 0x00063DD0)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollisionInfo {
    /// car id, or < 0 for none
    pub id: i32,
    /// the other car's id; -1 / -2 = scenery
    pub other: i32,
    pub point: Vec3,
    pub normal: Vec3,
    pub speed: f32,
}

#[derive(Clone, Debug, Default)]
pub struct VsCrashCameraManager {
    /// +0x18 four crash cameras (32 bytes each)
    pub cams: [VsCrashCamera; 4],
    pub tweaks: CrashCameraTweaks,
    pub trigger: CrashTriggerTweaks,
}

/// VsCrashCameraManager.cpp 0x000639C0 — angle between two directions under `deg`
fn within(a: Vec3, b: Vec3, deg: f32) -> bool {
    let a = super::vs_base_camera::normalise(a);
    let b = super::vs_base_camera::normalise(b);
    a.dot(b) as f64 > (deg.to_radians()).cos() as f64
}

impl VsCrashCameraManager {
    /// VsCrashCamera.cpp 0x000630E0 (players' fly cameras assigned by 0x00063580) / 0x00063520
    pub fn init(&mut self, players: u32) {
        for (i, c) in self.cams.iter_mut().enumerate() {
            c.camera = (i < players as usize).then_some(i as u32);
            c.reset(&self.tweaks);
        }
    }

    /// VsCrashCameraManager.cpp 0x00063D70 — any crash camera running
    pub fn any_active(&self) -> bool {
        self.cams.iter().any(VsCrashCamera::is_active)
    }

    /// VsCrashCameraManager.cpp 0x00063C50 — both cars of an impact are considered.
    #[allow(clippy::too_many_arguments)]
    pub fn on_collision(
        &mut self,
        c: &CollisionInfo,
        network: bool,
        find: &dyn Fn(i32) -> Option<CrashCandidate>,
        w: &dyn CrashCameraWorld,
        wmm: &mut CWorldModeManager,
        rand: &mut CrtRand,
        out: &mut Vec<CrashCameraEvent>,
    ) {
        if !self.trigger.enabled || network {
            return;
        }
        if self.trigger.ignore_first_seconds >= w.race_time() {
            log::info!("Early collision ignored");
            return;
        }
        self.consider(c.id, c.other, c, find, w, wmm, rand, out);
        self.consider(c.other, c.id, c, find, w, wmm, rand, out);
    }

    /// VsCrashCameraManager.cpp 0x00063620
    #[allow(clippy::too_many_arguments)]
    fn consider(
        &mut self,
        id: i32,
        other: i32,
        c: &CollisionInfo,
        find: &dyn Fn(i32) -> Option<CrashCandidate>,
        w: &dyn CrashCameraWorld,
        wmm: &mut CWorldModeManager,
        rand: &mut CrtRand,
        out: &mut Vec<CrashCameraEvent>,
    ) -> bool {
        let t = &self.trigger;
        if id < 0 {
            return false;
        }
        if !(other >= 0 || (t.car_level && t.min_impact_speed_level <= c.speed)) {
            return false;
        }
        if !(other == -1 || other == -2 || (t.car_car && t.min_impact_speed_car <= c.speed)) {
            return false;
        }
        let Some(v) = find(id) else { return false };
        if !v.human || v.finished || t.min_vehicle_speed > v.vehicle.speed {
            return false;
        }
        if t.all_wheels_on_ground && v.wheels_on_ground < 4 {
            return false;
        }
        let head_on = if v.in_air {
            // in the air only a car tipped past both limits qualifies
            if within(v.up, Vec3::Y, t.min_z_angle_in_air) {
                return false;
            }
            let flat = Vec3::new(v.forward.x, 0.0, v.forward.z);
            if within(flat, v.forward, t.min_x_angle_in_air) {
                return false;
            }
            within(v.forward, -c.normal, t.max_forward_angle)
        } else {
            within(v.forward, -c.normal, t.max_forward_angle)
        };
        if !head_on || id as usize >= 4 {
            return false;
        }
        // kind: 1 for scenery hits (-1), 0 otherwise; only kind 2 would be a failure (kept)
        let kind = (other == -1) as u32;
        let tw = self.tweaks.clone();
        self.cams[id as usize].start(v.vehicle, c.point, kind, &tw, w, wmm, rand, out)
    }

    /// VsCrashCameraManager.cpp 0x00063B00 — per update; all cameras stop when disabled
    pub fn update(&mut self, w: &dyn CrashCameraWorld, wmm: &mut CWorldModeManager, out: &mut Vec<CrashCameraEvent>) {
        let tw = self.tweaks.clone();
        for c in &mut self.cams {
            if self.trigger.enabled {
                c.update(&tw, w, wmm, out);
            } else {
                c.cancel(&tw, w, wmm, out);
            }
        }
    }

    /// VsCrashCameraManager.cpp 0x00063BA0 — a player skipped it (pause button held)
    pub fn skip(&mut self, player: u32, w: &dyn CrashCameraWorld, wmm: &mut CWorldModeManager, out: &mut Vec<CrashCameraEvent>) {
        let tw = self.tweaks.clone();
        if let Some(c) = self.cams.get_mut(player as usize) {
            c.cancel(&tw, w, wmm, out);
        }
    }

    /// the slowest stretch of the running cameras (VsCrashCamera.cpp 0x00062410, read by the frame timer)
    pub fn speed_rate(&self) -> f32 {
        self.cams.iter().map(|c| c.speed_rate(&self.tweaks)).fold(1.0, f32::min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct W;
    impl CrashCameraWorld for W {
        fn race_time(&self) -> f32 {
            30.0
        }
        fn camera_position(&self, _: u32) -> Vec3 {
            Vec3::ZERO
        }
        fn ray_fraction(&self, _: Vec3, _: Vec3, _: u32) -> Option<f32> {
            None
        }
    }

    #[test]
    fn head_on_hit_starts_it() {
        let mut m = VsCrashCameraManager::default();
        m.trigger.enabled = true;
        m.init(1);
        let car = CrashCandidate {
            vehicle: CrashVehicle { id: 0, speed: 40.0, side: Vec3::Z, ..Default::default() },
            human: true,
            wheels_on_ground: 4,
            forward: Vec3::X,
            up: Vec3::Y,
            ..Default::default()
        };
        let find = |id: i32| (id == 0).then_some(car);
        let (mut wmm, mut rand, mut out) = (CWorldModeManager::default(), CrtRand::default(), Vec::new());
        let side_hit = CollisionInfo { id: 0, other: -1, point: Vec3::ZERO, normal: Vec3::Z, speed: 40.0 };
        m.on_collision(&side_hit, false, &find, &W, &mut wmm, &mut rand, &mut out);
        assert!(!m.any_active());
        let wall = CollisionInfo { normal: Vec3::NEG_X, ..side_hit };
        m.on_collision(&wall, false, &find, &W, &mut wmm, &mut rand, &mut out);
        assert!(m.any_active());
        assert!((m.speed_rate() - 0.2).abs() < 1e-6);
    }
}
