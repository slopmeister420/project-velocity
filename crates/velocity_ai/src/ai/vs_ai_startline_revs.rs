//! Engine revving on the start line during the countdown (src/Ai/VsAIStartlineRevs.cpp).
//!
//! Four states: follow a response curve (`AI/ResCurve/Revs<n>.txt`) over time, ramp to a random
//! target, hold it, then blend into the real engine rpm once the lights go green.

use std::path::Path;

use super::vs_ai_globals::VsAIGlobals;
use super::vs_ai_response_curve::VsAIResponseCurve;
use super::vs_math::{VsRand, vs_abs};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum RevState {
    #[default]
    ResponseCurve = 0,
    Ramp = 1,
    Hold = 2,
    Release = 3,
}

/// `VsAIStartlineRevs` (104 bytes, at VsAIRaceVehicle +0x100).
#[derive(Debug, Clone, Default)]
pub struct VsAIStartlineRevs {
    pub state: RevState,       // +0x00
    pub active: bool,          // +0x04 m_bActive
    pub has_vehicle: bool,     // +0x08 m_pPhysicsVehicle
    pub rc: Option<VsAIResponseCurve>, // +0x0C m_pRC_StartlineRevs
    pub time_left: f32,        // +0x10 seconds until the lights end
    pub time_offset: f32,      // +0x14
    pub time_multiplier: f32,  // +0x18
    pub target_revs: f32,      // +0x1C
    pub ramp_revs: f32,        // +0x20
    pub ramp_time_start: f32,  // +0x24
    pub hold_revs: f32,        // +0x28
    pub hold_time_start: f32,  // +0x2C
    pub revs: f32,             // +0x30 output
}

impl VsAIStartlineRevs {
    /// creates the curve and binds the state table.
    /// VsAIStartlineRevs.cpp 0x000270F0
    pub fn init(&mut self) {
        debug_assert!(self.rc.is_none(), "m_pRC_StartlineRevs == NULL");
        self.rc = Some(VsAIResponseCurve::new());
    }

    /// VsAIStartlineRevs.cpp 0x00027330
    pub fn load(&mut self, ai_dir: &Path, name: &str) -> bool {
        debug_assert!(self.rc.is_some(), "pszFileName && m_pRC_StartlineRevs");
        self.rc.as_mut().is_some_and(|rc| rc.load(ai_dir, name))
    }

    /// VsAIStartlineRevs.cpp 0x000273D0
    pub fn shutdown(&mut self) {
        self.rc = None;
    }

    /// VsAIStartlineRevs.cpp 0x00027240
    pub fn reset(&mut self, g: &VsAIGlobals, rand: &mut VsRand) {
        self.state = RevState::ResponseCurve;
        self.active = true;
        self.randomise(g, rand);
    }

    /// VsAIStartlineRevs.cpp 0x00027280
    pub fn randomise(&mut self, g: &VsAIGlobals, rand: &mut VsRand) {
        self.time_offset = rand.range(0.0, 2.0);
        self.time_multiplier = rand.range(g.revs_min_time_multiplier, g.revs_max_time_multiplier);
        self.ramp_time_start = rand.range(g.revs_min_ramp_time_start, g.revs_max_ramp_time_start);
        self.target_revs = rand.range(g.revs_min_target, g.revs_max_target);
        self.hold_time_start = rand.range(g.revs_min_hold_time_start, g.revs_max_hold_time_start);
    }

    /// `timer_time` is the frame timer, `countdown_end` the end of the countdown, `engine_rpm` the car's.
    /// VsAIStartlineRevs.cpp 0x00027440
    pub fn update(&mut self, g: &VsAIGlobals, timer_time: f32, countdown_end: f32, engine_rpm: i32) {
        if !self.active {
            return;
        }
        self.time_left = countdown_end - timer_time;
        self.update_state(engine_rpm);
        match self.state {
            // VsAIStartlineRevs.cpp 0x000277E0
            RevState::ResponseCurve => {
                let x = (timer_time + self.time_offset) * self.time_multiplier;
                if let Some(rc) = &self.rc {
                    self.revs = rc.get_value(x);
                }
            }
            // VsAIStartlineRevs.cpp 0x00027870
            RevState::Ramp => {
                self.revs = (self.target_revs - self.ramp_revs)
                    * (1.0 - (self.time_left - self.hold_time_start) / (self.ramp_time_start - self.hold_time_start))
                    + self.ramp_revs;
            }
            // VsAIStartlineRevs.cpp 0x00027930
            RevState::Hold => self.revs = self.hold_revs,
            // VsAIStartlineRevs.cpp 0x00027A00
            RevState::Release => {
                self.revs = self.revs - (self.revs - engine_rpm as f32) * g.revs_blend_factor;
            }
        }
    }

    /// VsAIStartlineRevs.cpp 0x00027520
    fn update_state(&mut self, engine_rpm: i32) {
        match self.state {
            RevState::ResponseCurve => {
                if self.time_left >= 0.0 && self.ramp_time_start > self.time_left {
                    self.set_state(RevState::Ramp);
                }
            }
            RevState::Ramp => {
                if self.time_left >= 0.0 && self.hold_time_start > self.time_left {
                    self.set_state(RevState::Hold);
                }
            }
            RevState::Hold => {
                if self.time_left <= 0.0 {
                    self.set_state(RevState::Release);
                }
            }
            RevState::Release => {
                if vs_abs(self.revs - engine_rpm as f32) < 50.0 {
                    self.active = false;
                }
            }
        }
    }

    /// exit of the old state is a no-op for all four; entries copy the current output.
    /// VsAIStartlineRevs.cpp 0x00027630
    fn set_state(&mut self, s: RevState) {
        self.state = s;
        match s {
            RevState::ResponseCurve => {}
            // VsAIStartlineRevs.cpp 0x00027850
            RevState::Ramp => self.ramp_revs = self.revs,
            // VsAIStartlineRevs.cpp 0x00027910
            RevState::Hold => self.hold_revs = self.revs,
            // VsAIStartlineRevs.cpp 0x00027970
            RevState::Release => debug_assert!(self.has_vehicle, "m_pPhysicsVehicle != NULL"),
        }
    }

    /// VsAIStartlineRevs.h 0x00023B90
    pub fn get_revs(&self) -> f32 {
        debug_assert!(self.active, "m_bActive");
        self.revs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revs_curves_drive_the_states() {
        let Some(dir) = crate::game_dir() else { return };
        let g = VsAIGlobals::default();
        let mut rand = VsRand::default();
        let mut r = VsAIStartlineRevs::default();
        r.init();
        assert!(r.load(&dir.join("AI"), "Revs1"));
        r.has_vehicle = true;
        r.reset(&g, &mut rand);
        let mut t = 0.0;
        let end = 6.0;
        let mut seen = [false; 4];
        while r.active && t < 20.0 {
            r.update(&g, t, end, 1500);
            seen[r.state as usize] = true;
            t += 1.0 / 60.0;
        }
        assert!(seen.iter().all(|&s| s), "{seen:?}");
        assert!(!r.active);
    }
}
