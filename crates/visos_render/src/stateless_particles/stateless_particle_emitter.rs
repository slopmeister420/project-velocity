//! StatelessParticles\StatelessParticleEmitter.cpp — one live emitter (52 bytes in the original pool): a placement,
//! a seed that fixes its particles, an age and flags.

use bevy::math::Mat4;

/// +46 flags
pub mod flags {
    /// the clock runs (0x002D5800 sets it, 0x002D57D0 clears it)
    pub const RUNNING: u16 = 1;
    /// restarts when its time is up instead of dying
    pub const LOOPING: u16 = 2;
    /// removed on the next update
    pub const KILL: u16 = 4;
    pub const VISIBLE: u16 = 8;
}

/// `dword_6C1DC0`, the shared seed source (0x002D5BA0)
pub const SEED_START: u32 = 0x5FBE_76C9;

/// StatelessParticleEmitter.cpp 0x002D5BA0 — Schrage's form of the minimal standard generator without the negative fix-up, so it wraps
pub fn next_seed(s: &mut u32) -> u32 {
    *s = 48271u32.wrapping_mul(*s % 0xADC8).wrapping_sub(3399u32.wrapping_mul(*s / 0xADC8));
    *s
}

#[derive(Clone, Debug)]
pub struct CEmitter {
    /// +0 quaternion and +16 position in the original; kept as the matrix they rebuild (0x002D5830)
    pub matrix: Mat4,
    /// +32
    pub seed: u32,
    /// +36 age in seconds
    pub time: f32,
    /// +40 size scale
    pub scale: f32,
    /// +44 type index (-1 = none)
    pub type_index: i16,
    /// +46
    pub flags: u16,
    /// +48 time the emitter was told to stop (< 0 = not stopping)
    pub stop_time: f32,
}

impl CEmitter {
    /// StatelessParticleEmitter.cpp 0x002D5C00 (Init): new seed, scale 1, looping + visible, at the origin, clock reset
    pub fn init(seed_source: &mut u32) -> Self {
        let mut e = Self {
            matrix: Mat4::IDENTITY,
            seed: next_seed(seed_source),
            time: 0.0,
            scale: 1.0,
            type_index: -1,
            flags: flags::LOOPING | flags::VISIBLE,
            stop_time: -1.0,
        };
        e.reset();
        e
    }

    /// StatelessParticleEmitter.cpp 0x002D57D0
    pub fn reset(&mut self) {
        self.time = 0.0;
        self.stop_time = -1.0;
        self.flags &= !flags::RUNNING;
    }

    /// StatelessParticleEmitter.cpp 0x002D5800
    pub fn start(&mut self) {
        if self.flags & flags::RUNNING == 0 {
            self.reset();
            self.flags |= flags::RUNNING;
        }
    }

    /// StatelessParticleEmitter.cpp 0x002D5B70
    pub fn advance(&mut self, dt: f32) {
        if self.flags & flags::RUNNING != 0 {
            self.time += dt;
        }
    }

    /// StatelessParticleEmitter.cpp 0x002D59B0
    pub fn set_looping(&mut self, on: bool) {
        if on {
            self.flags |= flags::LOOPING;
        } else {
            self.flags &= !flags::LOOPING;
        }
    }

    /// StatelessParticleEmitter.cpp 0x002D5A10: stop emitting from now (particles already out live their life)
    pub fn stop(&mut self) {
        if self.stop_time < 0.0 {
            self.stop_time = self.time;
        }
    }

    /// StatelessParticleEmitter.cpp 0x002D5A80
    pub fn set_kill(&mut self, on: bool) {
        if on {
            self.flags |= flags::KILL;
        } else {
            self.flags &= !flags::KILL;
        }
    }
}
