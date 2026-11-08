//! src\Replays\Cameras\ReplayCamera_Orbit.cpp — a camera circling the car at 6 m, re-placed at random (a
//! quarter turn or more round) whenever the car is hidden from it.

use bevy::math::Vec3;
use visos::handy::crt_rand::CrtRand;

use super::ReplayWorld;
use crate::camera::vs_base_camera::yaw_pitch_roll;

/// `flt_687684` orbit radius
pub const RADIUS: f32 = 6.0;
/// `dword_687688` field of view (radians)
pub const FOV: f32 = f32::from_bits(0x3F33_3333);
/// elevation (0x3EDF66DB, 25 degrees)
pub const PHI: f32 = f32::from_bits(0x3EDF_66DB);
/// turn rate (0x3E860A92, 15 degrees a second)
pub const RATE: f32 = f32::from_bits(0x3E86_0A92);
const TWO_PI: f32 = 6.283_185_5;

/// `CReplayCameraOrbit` (96 bytes at the manager's +0xA4)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CReplayCameraOrbit {
    pub time: f32,       // +0x00
    pub center: Vec3,    // +0x04
    pub dir: Vec3,       // +0x14
    pub position: Vec3,  // +0x24
    pub up: Vec3,        // +0x34
    pub vehicle: u32,    // +0x44
    pub theta: f32,      // +0x48
    pub theta_rate: f32, // +0x4C
    pub phi: f32,        // +0x50
    pub unknown_54: f32, // +0x54
    pub initialised: bool, // +0x5C
    pub reposition: bool,  // +0x5D
    pub cut: bool,         // +0x5E
}

/// ReplayCamera_Orbit.cpp 0x00167710 — between wheels 3 and 1, half a metre up
pub fn car_center(w: &dyn ReplayWorld, vehicle: u32) -> Vec3 {
    let a = w.wheel_position(vehicle, 3) + Vec3::new(0.0, 0.5, 0.0);
    let b = w.wheel_position(vehicle, 1) + Vec3::new(0.0, 0.5, 0.0);
    (a + b) * 0.5
}

impl CReplayCameraOrbit {
    /// ReplayCamera_Manager.cpp 0x00166C30
    pub fn init(&mut self) {
        if !self.initialised {
            self.up = Vec3::Y;
            self.dir = Vec3::NEG_Z;
            self.time = 0.0;
            self.theta = 0.0;
            self.phi = PHI;
            self.initialised = true;
            self.reposition = true;
            self.cut = false;
        }
    }

    /// ReplayCamera_Manager.cpp 0x00166D30 / 0x00166D60
    pub fn reset_for(&mut self, vehicle: u32) {
        self.time = 0.0;
        self.reposition = true;
        self.cut = true;
        self.vehicle = vehicle;
    }

    /// ReplayCamera_Orbit.cpp 0x00167250
    fn new_angle(&mut self, rand: &mut CrtRand) {
        self.cut = true;
        let q = rand.rand_unit() as f32;
        self.theta += (((q * 2.9) as i32) as f32 + 1.0) * std::f32::consts::FRAC_PI_2;
        let lo = -0.392_699_1f32;
        self.theta += (rand.rand_unit() as f32) * (0.392_699_1 - lo) + lo;
        self.theta_rate = if (rand.rand_unit() as f32) <= 0.5 { 0.0 - 0.261_799_4 } else { RATE };
        self.unknown_54 = f32::from_bits(0x3E32_B8C2);
        self.phi = PHI;
    }

    /// ReplayCamera_Orbit.cpp 0x00167350 — can the camera see the car from where it would be now
    fn visible(&self, w: &dyn ReplayWorld) -> bool {
        if !(self.time >= 1.5 || self.reposition) {
            return true;
        }
        let car = w.car_position(self.vehicle);
        let dir = yaw_pitch_roll(self.phi, self.theta, 0.0) * Vec3::NEG_Z;
        let pos = car + dir * RADIUS;
        !w.blocked(pos, car, self.vehicle)
    }

    /// ReplayCamera_Orbit.cpp 0x001671F0 — up to 25 tries
    fn place(&mut self, w: &dyn ReplayWorld, rand: &mut CrtRand) {
        for _ in 0..25 {
            self.new_angle(rand);
            if self.visible(w) {
                break;
            }
        }
    }

    /// ReplayCamera_Orbit.cpp 0x00166D90
    pub fn update(&mut self, dt: f32, w: &dyn ReplayWorld, rand: &mut CrtRand) {
        if !self.initialised {
            return;
        }
        self.time += dt;
        if self.reposition {
            self.place(w, rand);
            self.time = 0.0;
            self.reposition = false;
        }
        if !self.visible(w) {
            self.place(w, rand);
            self.time = 0.0;
        }
        self.center = car_center(w, self.vehicle);
        if self.theta.abs() > TWO_PI {
            let n = (self.theta.abs() / TWO_PI) as i32;
            self.theta = if self.theta <= 0.0 {
                (n as f64 * TWO_PI as f64 + self.theta as f64) as f32
            } else {
                (self.theta as f64 - n as f64 * TWO_PI as f64) as f32
            };
        }
        if self.theta < 0.0 {
            self.theta += TWO_PI;
        }
        self.dir = yaw_pitch_roll(self.phi, self.theta, 0.0) * Vec3::NEG_Z;
        self.position = self.center + self.dir * RADIUS;
        self.theta += self.theta_rate * dt;
        self.phi = PHI;
    }
}
