//! src\Replays\Cameras\ReplayCamera_Manager.cpp — `g_ReplayCameraManager` (0x00749128): picks the TV camera that
//! shows the player's car once the race is over (or when review cameras are switched on). CAMERA_SWITCH
//! triggers name a fixed camera; without one the wheel camera, failing that the orbit camera.
//! (`DecideCameraShot`, 0x00165FF0, is compiled in but called from nowhere.)

use visos::handy::crt_rand::CrtRand;
use visos::handy::handy_crc;

use super::replay_camera_fixed::{parse_cameras, CReplayCameraFixed};
use super::replay_camera_orbit::{self, CReplayCameraOrbit};
use super::replay_camera_wheel::{self, CReplayCameraWheel};
use super::ReplayWorld;
use crate::game_bridge::camera_bridge::CameraBridgeData;
use crate::triggers::trigger_update::{contains, CTriggers};

/// `m_eCameraType`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EReplayCamera {
    #[default]
    None = 0,
    Orbit = 1,
    Fixed = 2,
    Wheel = 3,
}

/// `VsAIPoliceVehicle 0x00018920` (SetHumanControl) on player `player`'s race vehicle, asked by the game flow
/// (InGameUpdate's review switch 0x000B7D20); the ai world is the ai crate's, the binary applies it.
#[derive(bevy::prelude::Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerHumanControl {
    pub player: usize,
    pub human: bool,
}

/// what the manager wants done outside it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReplayEvent {
    /// message 3: the camera type changed (a cut)
    CameraChanged,
}

#[derive(Clone, Debug, Default)]
pub struct CReplayCameraManager {
    pub camera: EReplayCamera,   // +0x00
    pub vehicle: u32,            // +0x04
    pub unknown_08: bool,        // +0x08
    pub unknown_0c: u32,         // +0x0C
    /// +0x10 updating this update
    pub updating: bool,
    pub unknown_11: bool,        // +0x11
    pub last_time: f32,          // +0x14
    pub initialised: bool,       // +0x18
    /// +0x19 `<track><race>cameras.cam`
    pub path: String,
    /// +0x9C/+0xA0 the file read and handed to the fixed camera
    pub data: Option<Vec<u8>>,
    pub parsed: bool,
    pub orbit: CReplayCameraOrbit,  // +0xA4
    pub fixed: CReplayCameraFixed,  // +0x104
    pub wheel: CReplayCameraWheel,  // +0x16C
    pub time: f32,                  // +0x1B4
    pub has_file: bool,             // +0x1BC
    /// +0x1BD a new shot is wanted (DecideCameraShot)
    pub decide: bool,
    /// +0x1BE review mode (debug)
    pub review: bool,
    pub unknown_1bf: bool,
    /// +0x1C8 index of the CAMERA_SWITCH trigger the car is in
    pub last_trigger: Option<usize>,
    pub unknown_1cc: bool,
    /// `dword_749108` a camera lost the car
    pub lost: bool,
    /// `dword_749110` "Enable Cameras For Review" (debug tweak 0..2, ReplayCamera_Fixed 0x00165640): 1 = the ai
    /// drives player 0 and the replay cameras show it; 2 = fixed cameras only (the jump cam, VehicleEntity
    /// 0x001CCAD0, which also starts `review_timer`)
    pub review_cameras: u32,
    /// `dword_749114` last update's `review_cameras`
    pub review_cameras_prev: u32,
    /// `dword_749118` the jump cam countdown (InGameUpdate 0x000B8329), `dword_74911C` its start value
    pub review_timer: f32,
    pub review_timer_start: f32,
    /// `byte_75B278` the "Jump Cam" video option (the binary copies the front end's option in)
    pub jump_cam_option: bool,
    /// the race is LONDON2 circuit RACE05 (fixed cameras preferred there)
    pub london2_race05: bool,
    /// the view to apply
    pub bridge: CameraBridgeData,
    /// this update's camera cut wrote `bridge` into both camera bridge buffers (0x000969B0 swap, write, swap
    /// back), so the renderer does not lerp across the shot change; `write_camera_bridge` takes it
    pub bridge_both: bool,
}

impl CReplayCameraManager {
    /// ReplayCamera_Fixed.cpp 0x00165400 — "Replay Camera Manager Init": `track_path` / `race_path` as stored
    /// in the event database ("Tracks\LONDON\", "CIRCUIT\RACE01\")
    pub fn init(&mut self, track_path: &str, race_path: &str) {
        if self.initialised {
            return;
        }
        self.orbit.init();
        self.wheel.init();
        self.last_time = 0.0;
        self.time = 0.0;
        self.initialised = true;
        self.reset();
        self.path = format!("{track_path}{race_path}cameras.cam");
        self.london2_race05 = {
            let t = track_path.to_ascii_uppercase();
            let r = race_path.to_ascii_uppercase();
            t.contains("LONDON2") && r.contains("CIRCUIT") && r.contains("RACE05")
        };
        match visos::file::platform::read(&format!("\\{}", self.path)) {
            Ok(b) => {
                self.data = Some(b);
                self.has_file = true;
                self.parsed = false;
            }
            Err(_) => {
                self.has_file = false;
                log::warn!("No Cameras.cam found : ({})", self.path);
            }
        }
    }

    /// ReplayCamera_Fixed.cpp 0x00165730
    pub fn shutdown(&mut self) {
        if !self.initialised {
            return;
        }
        self.data = None;
        self.fixed.unload();
        self.initialised = false;
        self.parsed = false;
        self.updating = false;
        self.has_file = false;
        self.vehicle = 0;
        self.unknown_0c = 0;
    }

    /// ReplayCamera_Fixed.cpp 0x001658C0
    pub fn reset(&mut self) {
        self.time = 0.0;
        self.decide = true;
        self.unknown_1bf = true;
        self.vehicle = 0;
        self.unknown_08 = false;
        self.unknown_0c = 0;
        self.updating = false;
        self.unknown_1cc = false;
        self.orbit.reset_for(self.orbit.vehicle);
        self.fixed.reset();
        self.wheel.reset();
        self.last_trigger = None;
        self.camera = EReplayCamera::Wheel;
    }

    /// ReplayCamera_Fixed.cpp 0x001659D0 — "CheckForLoad": hand the file to the fixed camera once read
    pub fn check_for_load(&mut self) {
        if !self.parsed && self.has_file {
            if let Some(b) = self.data.as_ref() {
                match parse_cameras(b) {
                    Ok(f) => self.fixed.load(f),
                    Err(e) => log::warn!("{}: {e}", self.path),
                }
                self.parsed = true;
            }
        }
    }

    /// ReplayCamera_Manager.cpp 0x00166700 — choose the camera for `vehicle` (`camera_id` from a trigger)
    pub fn set_camera(&mut self, vehicle: u32, camera_id: u32, w: &dyn ReplayWorld, rand: &mut CrtRand, out: &mut Vec<ReplayEvent>) {
        if self.review_cameras == 2 {
            self.camera = EReplayCamera::Fixed;
            return;
        }
        if self.london2_race05 && self.fixed.choose(self.review, w) {
            self.camera = EReplayCamera::Fixed;
            return;
        }
        let prev = self.camera;
        self.vehicle = vehicle;
        if camera_id != 0 && self.has_file && {
            self.fixed.reset_for(vehicle);
            self.fixed.select_by_id(camera_id, w)
        } {
            self.camera = EReplayCamera::Fixed;
        } else {
            // ReplayCamera_Manager.cpp 0x00166690
            self.wheel.setup(self.vehicle, w, rand);
            if self.wheel.is_valid() {
                self.camera = EReplayCamera::Wheel;
            } else {
                self.orbit.reset_for(self.vehicle);
                self.camera = EReplayCamera::Orbit;
            }
        }
        if self.camera != prev {
            out.push(ReplayEvent::CameraChanged);
        }
    }

    /// ReplayCamera_Manager.cpp 0x00166910 — the CAMERA_SWITCH trigger the car enters names the camera
    pub fn check_triggers(&mut self, triggers: &CTriggers, w: &dyn ReplayWorld, rand: &mut CrtRand, out: &mut Vec<ReplayEvent>) {
        let pos = w.car_position(self.vehicle);
        let switch = handy_crc::crc32(b"CAMERA_SWITCH", 0);
        for (i, t) in triggers.list.iter().enumerate() {
            if t.file.type_crc != switch {
                continue;
            }
            if contains(&t.file, pos) {
                if self.last_trigger != Some(i) {
                    self.last_trigger = Some(i);
                    let id = t.file.target_crc;
                    self.set_camera(self.vehicle, id, w, rand, out);
                }
            } else if self.last_trigger == Some(i) {
                self.last_trigger = None;
            }
        }
    }

    /// ReplayCamera_Manager.cpp 0x00165A30 — per update (`session_state` = GameManager+0x24, from 5 on)
    pub fn update(
        &mut self,
        time: f32,
        session_state: u32,
        triggers: &CTriggers,
        w: &dyn ReplayWorld,
        rand: &mut CrtRand,
        out: &mut Vec<ReplayEvent>,
    ) {
        if !self.initialised {
            return;
        }
        self.updating = false;
        if !(self.parsed || !self.has_file) {
            return;
        }
        let mut dt = time - self.last_time;
        self.last_time = time;
        if dt < 0.0 {
            dt = 0.0;
        }
        if session_state < 5 || self.review {
            return;
        }
        if self.unknown_1cc {
            dt = 0.01;
        }
        self.time += dt;
        self.updating = true;
        self.check_triggers(triggers, w, rand, out);
        self.unknown_1bf = true;
        match self.camera {
            EReplayCamera::None => {}
            EReplayCamera::Orbit => self.orbit.update(dt, w, rand),
            EReplayCamera::Fixed => {
                if !self.fixed.update(dt, self.review, w) {
                    self.lost = true;
                }
            }
            EReplayCamera::Wheel => {
                if !self.wheel.update(w) {
                    self.lost = true;
                }
                self.unknown_1bf = false;
            }
        }
        if self.lost {
            self.lost = false;
            self.decide = true;
        }
        self.unknown_1cc = false;
    }

    /// ReplayCamera_Manager.cpp 0x00165DD0 — "ApplyCamera": the view for player 0's active camera
    pub fn apply(&mut self, dt: f32, session_state: u32, w: &dyn ReplayWorld) -> Option<CameraBridgeData> {
        if !self.initialised || session_state < 5 || self.review {
            return None;
        }
        let written = match self.camera {
            EReplayCamera::None => None,
            EReplayCamera::Orbit => self.orbit_bridge(),
            EReplayCamera::Fixed => self.fixed_bridge(dt, w),
            EReplayCamera::Wheel => self.wheel_bridge(),
        };
        let (rec, both) = written?;
        self.bridge = rec;
        self.bridge_both |= both;
        Some(self.bridge)
    }

    /// ReplayCamera_Orbit.cpp 0x001674E0 — view 0's bridge record (+0x24 eye, +0x04 centre, +0x34 up,
    /// `dword_687688` fov) when initialised (+0x5C); the cut (+0x5E) writes it into both buffers and clears it
    fn orbit_bridge(&mut self) -> Option<(CameraBridgeData, bool)> {
        if !self.orbit.initialised {
            return None;
        }
        let rec = CameraBridgeData { position: self.orbit.position, target: self.orbit.center, up: self.orbit.up, fov: replay_camera_orbit::FOV };
        let both = std::mem::take(&mut self.orbit.cut);
        Some((rec, both))
    }

    /// ReplayCamera_Fixed.cpp 0x00164D70 — loaded (+0x64) with a camera (+0x58) and 0x00163E80 still sees the car:
    /// view 0's bridge record (*(+0x58) eye, +0x34 target, +0x44 up, +0x54 fov); the cut (+0x0C) writes both buffers
    fn fixed_bridge(&mut self, dt: f32, w: &dyn ReplayWorld) -> Option<(CameraBridgeData, bool)> {
        if !(self.fixed.loaded && self.fixed.record.is_some()) {
            return None;
        }
        if !self.fixed.aim(dt, w) {
            return None;
        }
        let rec = CameraBridgeData { position: self.fixed.position, target: self.fixed.target, up: self.fixed.up, fov: self.fixed.fov };
        let both = std::mem::take(&mut self.fixed.cut);
        Some((rec, both))
    }

    /// ReplayCamera_Orbit.cpp 0x00167D60 — the wheel camera's bridge write (+0x40 initialised; +0x0C eye, +0x1C
    /// target, +0x2C up, `dword_68768C` fov); the cut (+0x09) writes both buffers
    fn wheel_bridge(&mut self) -> Option<(CameraBridgeData, bool)> {
        if !self.wheel.initialised {
            return None;
        }
        let rec = CameraBridgeData { position: self.wheel.position, target: self.wheel.target, up: self.wheel.up, fov: replay_camera_wheel::FOV };
        let both = std::mem::take(&mut self.wheel.cut);
        Some((rec, both))
    }

    /// ReplayCamera_Manager.cpp 0x00165D90 — a collision with car 0 while on the wheel camera wants a new shot
    pub fn on_collision(&mut self, id: i32, other: i32) {
        if self.camera == EReplayCamera::Wheel && (id == 0 || other == 0) {
            self.decide = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec3;

    struct W {
        blocked: bool,
    }
    impl ReplayWorld for W {
        fn car_position(&self, _: u32) -> Vec3 {
            Vec3::new(0.0, 0.0, 0.0)
        }
        fn wheel_position(&self, _: u32, w: u32) -> Vec3 {
            Vec3::new(if w == 0 || w == 1 { 1.5 } else { -1.5 }, 0.3, if w < 2 { 0.8 } else { -0.8 })
        }
        fn car_axes(&self, _: u32) -> (Vec3, Vec3, Vec3) {
            (Vec3::X, Vec3::Y, Vec3::Z)
        }
        fn blocked(&self, _: Vec3, _: Vec3, _: u32) -> bool {
            self.blocked
        }
    }

    #[test]
    fn wheel_then_orbit() {
        let mut m = CReplayCameraManager::default();
        m.init("Tracks\\LACOAST\\", "CIRCUIT\\RACE01\\");
        m.check_for_load();
        let mut rand = CrtRand::default();
        let mut out = Vec::new();
        m.set_camera(0, 0, &W { blocked: false }, &mut rand, &mut out);
        assert_eq!(m.camera, EReplayCamera::Wheel);
        m.set_camera(0, 0, &W { blocked: true }, &mut rand, &mut out);
        assert_eq!(m.camera, EReplayCamera::Orbit);
        m.update(1.0, 5, &CTriggers::default(), &W { blocked: false }, &mut rand, &mut out);
        let v = m.apply(1.0 / 24.0, 5, &W { blocked: false }).unwrap();
        assert!((v.position - v.target).length() > 5.0);
    }
}
