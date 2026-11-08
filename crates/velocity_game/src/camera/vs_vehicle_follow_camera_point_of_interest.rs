//! src\Camera\VsVehicleFollowCameraPointOfInterest.cpp — the follow camera's point of interest: something worth
//! looking at (a police car joining, a traffic event) turns the chase camera towards it for a while. The
//! object is passed by an id here; the follow camera asks for its position through the caller.

use bevy::math::Vec3;

use crate::game_mode::update_thread::CWorldModeManager;

/// WMM mode of a point of interest
pub const WMM_POINT_OF_INTEREST: i32 = 1;

/// "Camera.PointOfInterest" tweaks (0x0006A930); values as shipped
#[derive(Clone, Debug, PartialEq)]
pub struct PointOfInterestTweaks {
    pub cinematic_bars: bool,        // byte_685230 1
    /// byte_6D60B8 (bss): the ai drives the player's car while it runs
    pub ai_control: bool,
    pub in_time_player: f32,         // dword_685234 0.375
    pub out_time_player: f32,        // dword_685238 0.375
    pub in_time_target: f32,         // dword_68523C 0.375
    pub out_time_target: f32,        // dword_685240 0.375
    pub speed_rate: f32,             // flt_685244 0.4
    pub default_length: f32,         // dword_685248 2.0
    pub start_delay: f32,            // dword_68524C 0.5
    pub ai_control_extra_time: f32,  // dword_685250 1.0
    pub extra_fov: bool,             // byte_685254 1
    pub min_fov_distance: f32,       // dword_685258 4
    pub max_fov_distance: f32,       // dword_68525C 35
    pub extra_fov_at_max: f32,       // dword_685260 0.1
    /// byte_6D60B9 (bss): points of interest are accepted at all
    pub enabled: bool,
}

impl Default for PointOfInterestTweaks {
    fn default() -> Self {
        PointOfInterestTweaks {
            cinematic_bars: true,
            ai_control: false,
            in_time_player: 0.375,
            out_time_player: 0.375,
            in_time_target: 0.375,
            out_time_target: 0.375,
            speed_rate: f32::from_bits(0x3ECC_CCCD),
            default_length: 2.0,
            start_delay: 0.5,
            ai_control_extra_time: 1.0,
            extra_fov: true,
            min_fov_distance: 4.0,
            max_fov_distance: 35.0,
            extra_fov_at_max: f32::from_bits(0x3DCC_CCCD),
            enabled: false,
        }
    }
}

/// side effects for the OSD and the ai.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PoiEvent {
    CinematicBars(bool),
    /// VsAIPoliceVehicle.cpp 0x00018920 on the player car: `true` gives control back
    PlayerControl(bool),
}

/// `VsVehicleFollowCameraPointOfInterest` (`VsAIVehicle__2C7A0` returns the global)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VsVehicleFollowCameraPointOfInterest {
    /// +0x00 the object looked at (an entity id)
    pub object: Option<u32>,
    pub priority: f32,    // +0x04
    pub length: f32,      // +0x08
    pub length_left: f32, // +0x0C
    /// +0x10 blend towards the object, 0..1 (eased)
    pub t: f32,
    /// +0x18 the player-side half (fading out)
    pub out: bool,
    /// +0x1C ai control extra time left
    pub extra_left: f32,
    /// +0x20 start delay left
    pub delay_left: f32,
    /// +0x24 running (camera turned)
    pub active: bool,
}

impl VsVehicleFollowCameraPointOfInterest {
    /// VsVehicleFollowCameraPointOfInterest.cpp 0x00069BE0 — `None` clears it; a lower priority than the
    /// running one is refused unless it has run out. `in_tweak_menu` = FontHelper 0x000B36B0.
    #[allow(clippy::too_many_arguments)]
    pub fn set(
        &mut self,
        object: Option<u32>,
        mut length: f32,
        priority: f32,
        tw: &PointOfInterestTweaks,
        in_tweak_menu: bool,
        wmm: &mut CWorldModeManager,
        game_time: f32,
        out: &mut Vec<PoiEvent>,
    ) {
        if in_tweak_menu || !tw.enabled {
            return;
        }
        if object.is_some() && !wmm.request(WMM_POINT_OF_INTEREST, "Point of interest", game_time) {
            return;
        }
        if length < 0.0 {
            log::error!("assert: VsFge(length, 0.0f)");
        }
        if !(0.0..=1.0).contains(&priority) {
            log::error!("assert: VsFge(priority, 0.0f) && VsFle(priority, 1.0f)");
        }
        if length.abs() <= 1e-6 {
            length = tw.default_length;
        }
        if !(self.priority < priority || self.length_left.abs() <= 1e-6) {
            return;
        }
        if self.object.is_none() && object.is_some() {
            self.begin(tw, out);
        }
        if self.object.is_some() && object.is_none() {
            self.end(tw, wmm, game_time, out);
        }
        self.object = object;
        if object.is_some() {
            self.priority = priority;
            self.length_left = length;
            self.length = length;
        } else {
            self.priority = 0.0;
            self.length_left = 0.0;
            self.length = 0.0;
        }
        self.t = 0.0;
        self.extra_left = 0.0;
        self.out = false;
    }

    /// VsVehicleFollowCameraPointOfInterest.cpp 0x0006A850 — starts after the delay (or at once without one)
    fn begin(&mut self, tw: &PointOfInterestTweaks, out: &mut Vec<PoiEvent>) {
        if tw.start_delay.abs() <= 1e-6 {
            self.start(tw, out);
        } else {
            self.delay_left = tw.start_delay;
        }
    }

    /// VsVehicleFollowCameraPointOfInterest.cpp 0x0006A720
    fn start(&mut self, tw: &PointOfInterestTweaks, out: &mut Vec<PoiEvent>) {
        if tw.cinematic_bars {
            out.push(PoiEvent::CinematicBars(true));
        }
        if tw.ai_control {
            out.push(PoiEvent::PlayerControl(false));
        }
        self.active = true;
    }

    /// VsVehicleFollowCameraPointOfInterest.cpp 0x0006A8B0
    fn end(&mut self, tw: &PointOfInterestTweaks, wmm: &mut CWorldModeManager, game_time: f32, out: &mut Vec<PoiEvent>) {
        self.active = false;
        if tw.ai_control_extra_time.abs() <= 1e-6 {
            self.finish(tw, wmm, game_time, out);
        } else {
            self.extra_left = tw.ai_control_extra_time;
        }
    }

    /// VsVehicleFollowCameraPointOfInterest.cpp 0x0006A7B0
    fn finish(&mut self, tw: &PointOfInterestTweaks, wmm: &mut CWorldModeManager, game_time: f32, out: &mut Vec<PoiEvent>) {
        if tw.cinematic_bars {
            out.push(PoiEvent::CinematicBars(false));
        }
        if tw.ai_control {
            out.push(PoiEvent::PlayerControl(true));
        }
        wmm.leave(WMM_POINT_OF_INTEREST, 0.0, game_time);
        self.object = None;
    }

    /// VsVehicleFollowCameraPointOfInterest.cpp 0x00069E80 — per update
    pub fn update(&mut self, dt: f32, tw: &PointOfInterestTweaks, wmm: &mut CWorldModeManager, game_time: f32, out: &mut Vec<PoiEvent>) {
        if self.delay_left > 0.0 {
            self.delay_left -= dt;
            if self.delay_left >= 0.0 {
                return;
            }
            self.delay_left = 0.0;
            self.start(tw, out);
        }
        if self.extra_left > 0.0 {
            self.extra_left -= dt;
            if self.extra_left < 0.0 {
                self.extra_left = 0.0;
                self.finish(tw, wmm, game_time, out);
                return;
            }
        }
        if self.length_left <= 0.0 {
            self.t = 0.0;
            self.out = true;
            return;
        }
        self.length_left -= dt;
        if self.length_left <= 0.0 {
            self.length_left = 0.0;
            self.t = 0.0;
            return;
        }
        let (ip, op, it, ot) = (tw.in_time_player, tw.out_time_player, tw.in_time_target, tw.out_time_target);
        let left = self.length_left;
        if left.abs() <= 1e-6 {
            self.t = 0.0;
            self.out = true;
        } else if op + ot <= left {
            self.out = false;
            if left <= self.length - (ip + it) {
                self.t = 1.0;
            } else if left <= self.length - ip {
                // inT - slow
                self.t = (((self.length - ip) - left) / it) * 0.5 + 0.5;
            } else {
                // inP - fast
                self.t = ((self.length - left) / ip) * 0.5;
            }
        } else {
            self.out = true;
            self.t = if op <= left {
                // outT - slow
                ((left - op) / ot) * 0.5 + 0.5
            } else {
                // outP - fast
                (left / op) * 0.5
            };
        }
        // eased: (sin(t * pi - pi / 2) + 1) / 2
        let s = (self.t * std::f32::consts::PI - std::f32::consts::FRAC_PI_2).sin();
        self.t = (s + 1.0) * 0.5;
    }

    /// VsVehicleFollowCameraPointOfInterest.cpp 0x0006A910
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// VsVehicleFollowCameraPointOfInterest.cpp 0x0006A6E0 — game speed while looking (slows down with `t`)
    pub fn speed_rate(&self, tw: &PointOfInterestTweaks) -> f32 {
        if self.object.is_some() { 1.0 - (1.0 - tw.speed_rate) * self.t } else { 1.0 }
    }

    /// VsVehicleFollowCameraPointOfInterest.cpp 0x0006A570 — extra fov fraction for an object `dist` away from the camera
    pub fn extra_fov(&self, object_pos: Vec3, camera_pos: Vec3, tw: &PointOfInterestTweaks) -> Option<f32> {
        self.object?;
        if !tw.extra_fov {
            return None;
        }
        let d = (object_pos - camera_pos).length();
        Some(if tw.min_fov_distance <= d {
            if d <= tw.max_fov_distance {
                1.0 - ((1.0 - tw.extra_fov_at_max) * (d - tw.min_fov_distance)) / (tw.max_fov_distance - tw.min_fov_distance)
            } else {
                tw.extra_fov_at_max
            }
        } else {
            1.0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blends_in_and_out() {
        let tw = PointOfInterestTweaks { enabled: true, ..Default::default() };
        let mut p = VsVehicleFollowCameraPointOfInterest::default();
        let mut wmm = CWorldModeManager::default();
        let mut out = Vec::new();
        p.set(Some(7), 0.0, 0.5, &tw, false, &mut wmm, 0.0, &mut out);
        assert_eq!(p.length, 2.0);
        let mut peak = 0.0f32;
        // the 0.5 s start delay runs out on the 13th update
        for _ in 0..13 {
            p.update(1.0 / 24.0, &tw, &mut wmm, 0.0, &mut out);
        }
        assert!(p.is_active());
        for _ in 0..40 {
            p.update(1.0 / 24.0, &tw, &mut wmm, 0.0, &mut out);
            peak = peak.max(p.t);
        }
        assert!((peak - 1.0).abs() < 1e-6);
        assert_eq!(out.first(), Some(&PoiEvent::CinematicBars(true)));
    }
}
