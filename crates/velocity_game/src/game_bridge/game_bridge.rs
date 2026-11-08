//! src\GameBridge\GameBridge.cpp (+ CarBridge.cpp, OSDBridge.cpp, TimelineBridge.cpp) — the update writes what
//! the renderer needs into double buffers; between updates the renderer interpolates the last two by the frame
//! timer's fraction. The idb's labels are shuffled here: the car bridge's code is labelled CameraBridge
//! (0x00096D20..0x00097130), the camera bridge's EventDatabaseTrackLoading (0x000968F0..0x00096A20).
//!
//! `GameBridge` (0x006F5BC8 + 0x220 area): +0 car bridge, +4 camera bridge, +8 anim bridge (empty), +0xC OSD
//! bridge, +0x10 timeline bridge, +0x14 "updated" flag.

use bevy::math::{Quat, Vec3};

use super::camera_bridge::CameraBridgeData;

/// `MAX_VEHICLE_COUNT`
pub const MAX_CARS: usize = 22;
/// `CAMERA_VIEWS`
pub const CAMERA_VIEWS: usize = 2;
/// `MAX_OSDS`
pub const MAX_OSDS: usize = 2;
/// `MAX_TIMELINE_MODELS`
pub const MAX_TIMELINE_MODELS: usize = 5;

/// two copies written alternately; `write` is the one the update fills
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DoubleBuffer<T> {
    pub buf: [T; 2],
    pub write: usize,
}

impl<T> DoubleBuffer<T> {
    pub fn current(&self) -> &T {
        &self.buf[self.write]
    }
    pub fn current_mut(&mut self) -> &mut T {
        &mut self.buf[self.write]
    }
    pub fn previous(&self) -> &T {
        &self.buf[(self.write + 1) & 1]
    }
    /// CameraBridge.cpp 0x00097130 / 0x000969B0 / OSDBridge.cpp 0x00098400 / 0x00098BF0 — the bridge swaps
    pub fn swap(&mut self) {
        self.write = (self.write + 1) & 1;
    }
}

/// CarDamage 0x000762F0 — `a + (b - a) * t`
pub fn lerp3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t
}

/// one car (168 bytes): body and wheels
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarBridgeData {
    pub position: Vec3,           // +0x00
    pub rotation: Quat,           // +0x10
    pub wheel_position: [Vec3; 4], // +0x20
    pub wheel_rotation: [Quat; 4], // +0x60
    pub valid: bool,              // +0xA4
}

impl Default for CarBridgeData {
    fn default() -> Self {
        CarBridgeData {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            wheel_position: [Vec3::ZERO; 4],
            wheel_rotation: [Quat::IDENTITY; 4],
            valid: false,
        }
    }
}

/// the interpolated pose a car is drawn with
pub type CarPose = CarBridgeData;

/// one OSD (108 bytes)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsdBridgeData {
    pub a: f32,         // +0x00
    /// +0x04 printed with "%d" into `text`
    pub value: f32,
    pub text: Option<String>, // +0x08
    /// +0x0C positions of the other racers (network name tags)
    pub racers: [Vec3; 6],
}

#[derive(Clone, Debug, Default, bevy::prelude::Resource)]
pub struct GameBridge {
    pub cars: Vec<DoubleBuffer<CarBridgeData>>,
    /// per view the player camera and the mirror camera
    pub cameras: [DoubleBuffer<[CameraBridgeData; 2]>; CAMERA_VIEWS],
    pub osd: [DoubleBuffer<OsdBridgeData>; MAX_OSDS],
    pub timeline_models: [DoubleBuffer<bevy::math::Mat4>; MAX_TIMELINE_MODELS],
    /// +0x294 the timeline model was written this update
    pub timeline_written: [bool; MAX_TIMELINE_MODELS],
    /// OSD bridge +0x290 players, +0x294 racers, +0x298 network game
    pub osd_players: u32,
    pub osd_racers: u32,
    pub osd_network: bool,
    /// +0x14 camera cut: set by VsCameraManager 0x00060380 (view changes, camera switches), cleared by the swap;
    /// while set the render draws the latest update (`render_lerp`)
    pub cut: bool,
    pub initialised: bool,
}

impl GameBridge {
    /// CarBridge.cpp 0x000972B0 — in game only (mode 1)
    pub fn init(&mut self, players: u32, racers: u32, network: bool) {
        self.cars = vec![DoubleBuffer::default(); MAX_CARS];
        self.cameras = Default::default();
        self.osd = Default::default();
        self.osd_players = players;
        self.osd_racers = racers;
        self.osd_network = network;
        if network && racers > 6 {
            log::error!("assert: m_iNumRacers <= CNetworkGame::MAX_NET_PLAYERS");
        }
        self.timeline_models = Default::default();
        self.timeline_written = [false; MAX_TIMELINE_MODELS];
        self.initialised = true;
    }

    /// GameBridge.cpp 0x000975A0
    pub fn shutdown(&mut self) {
        *self = GameBridge::default();
    }

    /// CarBridge.cpp 0x000971A0 — the record the update writes for car `n`
    pub fn car(&mut self, n: usize) -> &mut CarBridgeData {
        if n >= MAX_CARS {
            log::error!("assert: (nCarNo >= 0) && (nCarNo < MAX_VEHICLE_COUNT)");
        }
        self.cars[n].current_mut()
    }

    /// CameraBridge.cpp 0x00096C70 — the record the update writes for view `n`
    pub fn camera(&mut self, n: usize) -> &mut [CameraBridgeData; 2] {
        if n >= CAMERA_VIEWS {
            log::error!("assert: (nView >= 0) && (nView < CAMERA_VIEWS)");
        }
        self.cameras[n].current_mut()
    }

    /// CameraBridge.cpp 0x000969B0 + 0x00096C70 + 0x000969B0 — view `n`'s record in the other buffer (swap,
    /// write, swap back: the replay cameras' cut)
    pub fn camera_previous(&mut self, n: usize) -> &mut [CameraBridgeData; 2] {
        let c = &mut self.cameras[n];
        let prev = (c.write + 1) & 1;
        &mut c.buf[prev]
    }

    /// TimelineBridge.cpp 0x00098C60
    pub fn timeline_model(&mut self, n: usize) -> &mut bevy::math::Mat4 {
        if n >= MAX_TIMELINE_MODELS {
            log::error!("assert: (nModel >= 0) && (nModel < MAX_TIMELINE_MODELS)");
        }
        self.timeline_written[n] = true;
        self.timeline_models[n].current_mut()
    }

    /// GameBridge.cpp 0x00097A00 — flip the buffers. Run it before the update writes, so that while rendering
    /// `write` holds the newest update and the other buffer the one before (what the interpolation reads)
    pub fn swap(&mut self, paused: bool) {
        if paused || !self.initialised {
            return;
        }
        for c in &mut self.cars {
            c.swap();
        }
        for c in &mut self.cameras {
            c.swap();
        }
        for o in &mut self.osd {
            o.swap();
        }
        for m in &mut self.timeline_models {
            m.swap();
        }
        self.cut = false;
    }

    /// GameManager::Render 0x000BBB00 / GameManager 0x000B9010 (physics objects): the bridge lerp is 1 while the
    /// cut flag (0x000BFAB0 = +0x14) is set, else the frame timer's
    pub fn render_lerp(&self, lerp: f32) -> f32 {
        if self.cut { 1.0 } else { lerp }
    }

    /// CarBridge 0x00096E20 — a car between the last two updates: interpolated when both are valid and it
    /// moved less than 50 m (a teleport snaps), else the latest
    pub fn car_pose(&self, n: usize, t: f32) -> CarPose {
        let c = &self.cars[n];
        let now = c.buf[c.write];
        let before = c.buf[(c.write + 1) & 1];
        if !now.valid {
            return now;
        }
        if before.valid && now.position.distance_squared(before.position) < 2500.0 {
            let mut p = now;
            p.position = lerp3(before.position, now.position, t);
            p.rotation = before.rotation.slerp(now.rotation, t);
            for j in 0..4 {
                p.wheel_position[j] = lerp3(before.wheel_position[j], now.wheel_position[j], t);
                p.wheel_rotation[j] = before.wheel_rotation[j].slerp(now.wheel_rotation[j], t);
            }
            p
        } else {
            now
        }
    }

    /// CameraBridge 0x00096A20 — view `n` (main and mirror) between the last two updates
    pub fn camera_view(&self, n: usize, t: f32) -> [CameraBridgeData; 2] {
        let c = &self.cameras[n];
        let now = c.buf[c.write];
        let before = c.buf[(c.write + 1) & 1];
        let mix = |a: &CameraBridgeData, b: &CameraBridgeData| CameraBridgeData {
            position: lerp3(a.position, b.position, t),
            target: lerp3(a.target, b.target, t),
            up: lerp3(a.up, b.up, t),
            fov: (b.fov - a.fov) * t + a.fov,
        };
        [mix(&before[0], &now[0]), mix(&before[1], &now[1])]
    }

    /// OSDBridge 0x00098470 — the OSD values between updates (network name tags of the other racers too)
    pub fn osd_view(&self, n: usize, t: f32) -> OsdBridgeData {
        let c = &self.osd[n];
        let now = &c.buf[c.write];
        let before = &c.buf[(c.write + 1) & 1];
        let mut o = OsdBridgeData {
            a: (now.a - before.a) * t + before.a,
            value: (now.value - before.value) * t + before.value,
            text: None,
            racers: now.racers,
        };
        if now.text.is_some() {
            o.text = Some(format!("{}", o.value as i32));
        }
        if self.osd_network && (n as u32) < self.osd_players {
            for k in self.osd_players as usize..(self.osd_racers as usize).min(6) {
                o.racers[k] = lerp3(before.racers[k], now.racers[k], t);
            }
        }
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn car_interpolation_and_teleport() {
        let mut b = GameBridge::default();
        b.init(1, 1, false);
        *b.car(0) = CarBridgeData { position: Vec3::ZERO, valid: true, ..Default::default() };
        b.swap(false);
        *b.car(0) = CarBridgeData { position: Vec3::X * 10.0, valid: true, ..Default::default() };
        let p = b.car_pose(0, 0.5);
        assert!((p.position.x - 5.0).abs() < 1e-5);
        *b.car(0) = CarBridgeData { position: Vec3::X * 100.0, valid: true, ..Default::default() };
        assert_eq!(b.car_pose(0, 0.5).position.x, 100.0);
    }
}
