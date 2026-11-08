//! src\Replays\Cameras\ReplayCamera_Wheel.cpp — a camera low by a front wheel looking along the car. Its code
//! sits in the range the idb labels ReplayCamera_Orbit (0x00167860..0x00167E90).

use bevy::math::Vec3;
use visos::handy::crt_rand::CrtRand;

use super::ReplayWorld;

/// `dword_68768C` field of view (radians)
pub const FOV: f32 = 0.5;
/// offsets along the car: `flt_687698` forward, `flt_687694` up, `dword_687690` sideways
pub const FORWARD: f32 = -2.0;
pub const UP: f32 = f32::from_bits(0x3E4C_CCCD);
pub const SIDE: f32 = 0.5;

/// `CReplayCameraWheel` (at the manager's +0x16C)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CReplayCameraWheel {
    pub vehicle: u32,     // +0x00
    pub valid: bool,      // +0x08
    pub cut: bool,        // +0x09
    pub position: Vec3,   // +0x0C
    pub target: Vec3,     // +0x1C
    pub up: Vec3,         // +0x2C
    pub initialised: bool, // +0x40
    /// +0x44 0 or 1
    pub side: u32,
}

impl CReplayCameraWheel {
    /// ReplayCamera_Orbit.cpp 0x001678A0
    pub fn init(&mut self) {
        if !self.initialised {
            self.cut = false;
            self.valid = false;
            self.initialised = true;
        }
    }

    /// ReplayCamera_Orbit.cpp 0x001678E0 — empty in F3_D (clears nothing)
    pub fn reset(&mut self) {}

    /// ReplayCamera_Wheel.cpp 0x00167E90 — the wheel followed
    pub fn wheel(&self) -> u32 {
        match self.side {
            0 => 3,
            1 => 0,
            _ => {
                log::error!("assert: wheel cam is trying to follow an invalid wheel!!");
                3
            }
        }
    }

    /// ReplayCamera_Orbit.cpp 0x00167930
    fn place(&mut self, w: &dyn ReplayWorld) {
        let (wheel, sign) = if self.side == 1 { (0, 1.0) } else { (3, -1.0) };
        let wp = w.wheel_position(self.vehicle, wheel);
        let (fwd, up, side) = w.car_axes(self.vehicle);
        let off = side * (sign * SIDE) + up * UP + fwd * FORWARD;
        self.position = wp + off;
        self.target = self.position + fwd;
        self.up = up;
    }

    /// ReplayCamera_Orbit.cpp 0x00167C20
    fn visible(&self, w: &dyn ReplayWorld) -> bool {
        self.initialised && !w.blocked(self.position, w.car_position(self.vehicle), self.vehicle)
    }

    /// ReplayCamera_Orbit.cpp 0x00167B10 — pick a side at random, the other if that one is hidden
    pub fn setup(&mut self, vehicle: u32, w: &dyn ReplayWorld, rand: &mut CrtRand) {
        if !self.initialised {
            return;
        }
        self.vehicle = vehicle;
        self.side = if (rand.rand_unit() as f32) <= 0.5 { 0 } else { 1 };
        self.place(w);
        if self.visible(w) {
            self.valid = true;
        } else {
            self.side = if self.side != 0 { 0 } else { 1 };
            self.place(w);
            self.valid = self.visible(w);
        }
        if self.valid {
            self.cut = true;
        }
    }

    /// ReplayCamera_Orbit.cpp 0x00167CB0
    pub fn is_valid(&self) -> bool {
        self.initialised && self.valid
    }

    /// ReplayCamera_Orbit.cpp 0x00167CE0 — false = it lost the car
    pub fn update(&mut self, w: &dyn ReplayWorld) -> bool {
        if !self.initialised {
            return true;
        }
        self.place(w);
        self.valid = self.visible(w);
        self.valid
    }
}
