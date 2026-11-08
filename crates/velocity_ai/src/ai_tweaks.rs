//! The ai's debug-menu tweaks on core's `CTweakManager` (`GameManager.tweaks`): VsAIWorld::AddTweaks ("AI.*", from
//! LoadWorldData) and the TFT manager's ("TFT", from the TFT Manager load step).
//!
//! the original points each tweak at its variable; here every tweak gets a `TweakCell` bound to the variable in the
//! `AiWorld` through an accessor. Cells of the static variables (the 0x006D3698 block, the slowdown statics, the TFT
//! manager at 0x006D26E0) outlive the race like the statics do and are pushed into each new world; the
//! PlayerAISpeedRC cells point into the per-race response curve and are reread at registration.

use bevy::prelude::*;
use velocity_game::cli::tweak::{CTweakManager, TweakCell, TweakData};

use crate::ai::ai_world::AiWorld;

/// the variable a tweak edits.
pub enum VarRef<'a> {
    F(&'a mut f32),
    B(&'a mut bool),
    U(&'a mut u32),
}

impl VarRef<'_> {
    fn read(&self) -> TweakData {
        match self {
            VarRef::F(v) => TweakData::Float(**v),
            VarRef::B(v) => TweakData::Bool(**v),
            VarRef::U(v) => TweakData::Int(**v as i32),
        }
    }
    fn write(&mut self, c: &TweakCell) {
        match self {
            VarRef::F(v) => **v = c.f32(),
            VarRef::B(v) => **v = c.bool(),
            VarRef::U(v) => **v = c.i32() as u32,
        }
    }
}

pub type Access = for<'a> fn(&'a mut AiWorld) -> Option<VarRef<'a>>;

pub struct Binding {
    /// "system path/tweak name"
    pub key: String,
    pub cell: TweakCell,
    access: Access,
    /// a static in the original (kept across races)
    persist: bool,
}

/// every ai tweak cell registered so far (resource; survives races and tweak manager resets).
#[derive(Resource, Default)]
pub struct AiTweaks {
    pub bindings: Vec<Binding>,
}

impl AiTweaks {
    pub fn cell(&self, key: &str) -> Option<&TweakCell> {
        self.bindings.iter().find(|b| b.key == key).map(|b| &b.cell)
    }

    /// the statics carry over into a new world (VsAIWorld::Init does not touch them)
    pub fn push_statics(&self, w: &mut AiWorld) {
        for b in self.bindings.iter().filter(|b| b.persist) {
            if let Some(mut v) = (b.access)(w) {
                v.write(&b.cell);
            }
        }
    }

    /// cells -> variables (before the ai update: picks up debug-menu edits)
    pub fn tweaks_out(&self, w: &mut AiWorld) {
        for b in &self.bindings {
            if let Some(mut v) = (b.access)(w) {
                v.write(&b.cell);
            }
        }
    }

    /// variables -> cells (after the ai update: the menu shows what the ai wrote)
    pub fn tweaks_in(&self, w: &mut AiWorld) {
        for b in &self.bindings {
            if let Some(v) = (b.access)(w) {
                let d = v.read();
                if b.cell.get() != d {
                    b.cell.set(d);
                }
            }
        }
    }

    fn bind(&mut self, key: String, access: Access, persist: bool, w: &mut AiWorld) -> TweakCell {
        if let Some(b) = self.bindings.iter().find(|b| b.key == key) {
            if !b.persist
                && let Some(v) = (b.access)(w)
            {
                b.cell.set(v.read());
            }
            return b.cell.clone();
        }
        let init = (access)(w).map(|v| v.read()).unwrap_or(TweakData::Float(0.0));
        let cell = TweakCell::new(init);
        self.bindings.push(Binding { key, cell: cell.clone(), access, persist });
        cell
    }
}

/// registration helper: the current system (`dword_6D7D64`) and its path
struct Reg<'a> {
    tm: &'a mut CTweakManager,
    t: &'a mut AiTweaks,
    w: &'a mut AiWorld,
    sys: usize,
    path: String,
}

impl Reg<'_> {
    fn sys(&mut self, path: &str) {
        self.sys = self.tm.create_system(path);
        self.path = path.to_string();
    }
    fn cell(&mut self, name: &str, a: Access, persist: bool) -> TweakCell {
        let key = format!("{}/{}", self.path, name);
        self.t.bind(key, a, persist, self.w)
    }
    fn f(&mut self, name: &str, a: Access, min: u32, max: u32, step: f32) {
        let c = self.cell(name, a, true);
        self.tm.add_tweak_f32(self.sys, name, c, f32::from_bits(min), f32::from_bits(max), step);
    }
    fn b(&mut self, name: &str, a: Access) {
        let c = self.cell(name, a, true);
        self.tm.add_tweak_bool(self.sys, name, c);
    }
    fn i(&mut self, name: &str, a: Access, min: i32, max: i32, step: i32) {
        let c = self.cell(name, a, true);
        self.tm.add_tweak_i32(self.sys, name, c, min, max, step);
    }
    fn e(&mut self, name: &str, a: Access, elements: &str) {
        let c = self.cell(name, a, true);
        self.tm.add_tweak_enum(self.sys, name, c, elements);
    }
    /// a PlayerAISpeedRC point (per-race curve, not a static)
    fn rc(&mut self, name: &str, a: Access) {
        let c = self.cell(name, a, false);
        self.tm.add_tweak_f32(self.sys, name, c, 0.0, f32::from_bits(0x3F80_0000), 0.001);
    }
}

macro_rules! g {
    ($k:ident, $($p:tt)+) => {
        (|w: &mut AiWorld| -> Option<VarRef<'_>> { Some(VarRef::$k(&mut w.world.globals.$($p)+)) }) as Access
    };
}

macro_rules! tft {
    ($k:ident, $f:ident) => {
        (|w: &mut AiWorld| -> Option<VarRef<'_>> { Some(VarRef::$k(&mut w.race.tft.$f)) }) as Access
    };
}

/// y of point `k` of race car 0's speed-ratio curve (`0x00015DB0(rm, 0)` -> +0 handling -> +4 curve -> +4 items,
/// `+4 + 8k`). Every race car shares the 350Z handling, so the edit reaches all of them.
fn speed_rc(w: &mut AiWorld, k: usize) -> Option<VarRef<'_>> {
    let h = w.race.players.first()?.base.handling?;
    let rc = w.world.handlings.get(h)?.speed_ratio_rc;
    let item = w.world.response_curves.get_mut(rc)?.items.get_mut(k)?;
    Some(VarRef::F(&mut item.1))
}

macro_rules! rc {
    ($k:literal) => {
        (|w: &mut AiWorld| -> Option<VarRef<'_>> { speed_rc(w, $k) }) as Access
    };
}

/// VsAIWorld.cpp 0x0002D360
pub fn add_tweaks(tm: &mut CTweakManager, t: &mut AiTweaks, w: &mut AiWorld) {
    log::debug!("Tweak Add AI");
    let mut r = Reg { tm, t, w, sys: CTweakManager::ROOT, path: String::new() };
    r.sys("AI");
    r.f("Difficulty Scale", g!(F, difficulty_scale), 0, 0x3F80_0000, 0.05);
    r.b("PlayerAIControl", g!(B, player_ai_control));
    r.b("TriggerFinishRace", g!(B, trigger_finish_race));
    r.b("ForceFollowRaceline", g!(B, force_follow_raceline));
    r.b("RacelineTrackCam", g!(B, raceline_track_cam));
    r.b("DrawVehicleDebug", g!(B, draw_vehicle_debug));
    r.b("DrawDriverDebug", g!(B, draw_driver_debug));
    r.b("DrawPoliceDebug", g!(B, draw_police_debug));
    r.b("DrawMistakeDebug", g!(B, draw_mistake_debug));
    r.b("DrawAvoidanceDebug", g!(B, draw_avoidance_debug));
    r.b("DrawResponseCurves", g!(B, draw_response_curves));
    r.b("DrawRaceline", g!(B, draw_raceline));
    r.b("DrawEdges", g!(B, draw_edges));
    r.b("DrawLanes", g!(B, draw_lanes));
    r.b("DrawTrafTrigSplines", g!(B, draw_traf_trig_splines));
    r.b("DrawTrailBlazerSplines", g!(B, draw_trail_blazer_splines));
    r.b("DrawTriggers", g!(B, draw_triggers));
    r.f("Judgment error (low skill)", g!(F, judgment_error[0]), 0, 0x3F80_0000, 0.01);
    r.f("Judgment error (med skill)", g!(F, judgment_error[1]), 0, 0x3F80_0000, 0.01);
    r.f("Judgment error (high skill)", g!(F, judgment_error[2]), 0, 0x3F80_0000, 0.01);
    r.f("catchup boost (low skill)", g!(F, catchup_boost[0]), 0x3F80_0000, 0x40A0_0000, 0.01);
    r.f("catchup boost (med skill)", g!(F, catchup_boost[1]), 0x3F80_0000, 0x40A0_0000, 0.01);
    r.f("catchup boost (high skill)", g!(F, catchup_boost[2]), 0x3F80_0000, 0x40A0_0000, 0.01);
    r.f("big collision threshold", g!(F, big_collision_threshold), 0x4170_0000, 0x428C_0000, 1.0);

    r.sys("AI.Mistakes");
    r.b("Make Mistakes", g!(B, make_mistakes));
    r.sys("AI.Mistakes.Skid out");
    r.f("Skid out time", g!(F, skid_out_time), 0, 0x40A0_0000, 0.1);
    r.f("Min skid out steering", g!(F, min_skid_out_steering), 0, 0x3F80_0000, 0.01);
    r.f("Skid out handbrake strength", g!(F, skid_out_handbrake_strength), 0, 0x3F80_0000, 0.05);
    r.sys("AI.Mistakes.Brake late");
    r.f("Brake late time", g!(F, brake_late_time), 0, 0x40A0_0000, 0.1);
    // max 0 in the original: the menu clamps it to 0
    r.f("Min brake late corner tightness", g!(F, min_brake_late_corner_tightness), 0, 0, 0.01);
    r.f("Brake late brake multiplier", g!(F, brake_late_brake_multiplier), 0, 0x3F80_0000, 0.05);
    r.f("Brake late steering multiplier", g!(F, brake_late_steering_multiplier), 0, 0x3F80_0000, 0.05);
    r.sys("AI.Mistakes.Misalign");
    r.f("Misalign time", g!(F, misalign_time), 0, 0x40A0_0000, 0.1);
    r.f("Max misalign road width", g!(F, max_misalign_road_width), 0, 0x4248_0000, 0.5);
    r.f("Misalign amount", g!(F, misalign_amount), 0, 0x4120_0000, 0.1);

    r.sys("AI.Nitrous");
    r.b("Use Nitrous", g!(B, use_nitrous));
    r.f("Ahead of player threshold", g!(F, nos_ahead_of_player_threshold), 0, 0x437A_0000, 1.0);
    r.f("EOR no conservation length", g!(F, nos_eor_no_conservation_length), 0, 0x43FA_0000, 10.0);
    r.f("Speed check distance", g!(F, nos_speed_check_distance), 0, 0x43FA_0000, 10.0);
    r.f("Speed check threshold", g!(F, nos_speed_check_threshold), 0, 0x42C8_0000, 1.0);
    macro_rules! aggression {
        ($sys:literal, $i:literal) => {
            r.sys($sys);
            r.f("Start straightness length", g!(F, nitrous_aggression[$i].start_straightness_length), 0x4120_0000, 0x43FA_0000, 1.0);
            r.f("Continue straightness length", g!(F, nitrous_aggression[$i].continue_straightness_length), 0x4120_0000, 0x43FA_0000, 1.0);
            r.f("Overtaken time window", g!(F, nitrous_aggression[$i].overtaken_time_window), 0, 0x4120_0000, 0.1);
            r.f("Overtaken length factor", g!(F, nitrous_aggression[$i].overtaken_length_factor), 0x3DCC_CCCD, 0x3F80_0000, 0.01);
            r.f("Minimum road straightness", g!(F, nitrous_aggression[$i].minimum_road_straightness), 0, 0x42B4_0000, 1.0);
            r.f("Max steering", g!(F, nitrous_aggression[$i].max_steering), 0, 0x3F80_0000, 0.01);
        };
    }
    aggression!("AI.Nitrous.Low Aggression", 0);
    aggression!("AI.Nitrous.Med Aggression", 1);
    aggression!("AI.Nitrous.High Aggression", 2);
    r.sys("AI.Nitrous.Low Skill");
    r.f("Conservation propensity", g!(F, nos_conservation_propensity[0]), 0, 0x4000_0000, 0.05);
    r.sys("AI.Nitrous.Med Skill");
    r.f("Conservation propensity", g!(F, nos_conservation_propensity[1]), 0, 0x4000_0000, 0.05);
    // lower-case "high" in the original
    r.sys("AI.Nitrous.high Skill");
    r.f("Conservation propensity", g!(F, nos_conservation_propensity[2]), 0, 0x4000_0000, 0.05);

    r.sys("AI.Slipstream");
    r.b("Use Slipstream", g!(B, use_slipstream));
    r.f("spd diff thresh/quick overtake", g!(F, spd_diff_thresh_quick_overtake), 0, 0x4248_0000, 1.0);
    macro_rules! slipstream {
        ($sys:literal, $i:literal) => {
            r.sys($sys);
            r.f("Min distance", g!(F, slipstream[$i].min_distance), 0, 0x4120_0000, 0.25);
            r.f("Min speed", g!(F, slipstream[$i].min_speed), 0, 0x42C8_0000, 1.0);
            r.f("Min speed diff (current)", g!(F, slipstream[$i].min_speed_diff_current), 0xC1A0_0000, 0x42C8_0000, 1.0);
            r.f("Min speed diff (new)", g!(F, slipstream[$i].min_speed_diff_new), 0xC1A0_0000, 0x42C8_0000, 1.0);
            r.f("Max speed difference", g!(F, slipstream[$i].max_speed_difference), 0, 0x42C8_0000, 1.0);
            r.f("Semicircle radius", g!(F, slipstream[$i].semicircle_radius), 0, 0x42C8_0000, 1.0);
            r.f("Semicircle angle", g!(F, slipstream[$i].semicircle_angle), 0, 0x42B4_0000, 1.0);
            r.f("Direction difference", g!(F, slipstream[$i].direction_difference), 0, 0x42B4_0000, 1.0);
            r.f("Steering smoothness", g!(F, slipstream[$i].steering_smoothness), 0, 0x3F80_0000, 0.01);
        };
    }
    slipstream!("AI.Slipstream.Low skill", 0);
    slipstream!("AI.Slipstream.Med skill", 1);
    slipstream!("AI.Slipstream.High skill", 2);

    r.sys("AI.Disorient");
    r.f("Min impact speed", g!(F, disorient_min_impact_speed), 0, 0x447A_0000, 10.0);
    r.f("Max impact speed", g!(F, disorient_max_impact_speed), 0, 0x447A_0000, 10.0);
    r.f("Max delay", g!(F, disorient_max_delay), 0, 0x4120_0000, 0.1);
    // created empty
    r.sys("AI.Data");

    r.sys("AI.PlayerAISpeedRC");
    r.rc("0.0", rc!(0));
    r.rc("0.125", rc!(1));
    r.rc("0.25", rc!(2));
    r.rc("0.375", rc!(3));
    r.rc("0.5", rc!(4));
    r.rc("0.625", rc!(5));
    r.rc("0.75", rc!(6));
    r.rc("0.875", rc!(7));
    r.rc("1.0", rc!(8));

    r.sys("AI.OpponentHandling");
    r.sys("AI.OpponentHandling.Global");
    r.b("Apply Slowdown", g!(B, apply_slowdown));
    r.f("Min Slowdown at Dist", g!(F, slowdown_min_dist), 0, 0x42C8_0000, 5.0);
    r.f("Max Slowdown at Dist", g!(F, slowdown_max_dist), 0, 0x4348_0000, 5.0);
    r.f("Max Slowdown Accel", g!(F, slowdown_max_accel), 0, 0x3F80_0000, 0.05);
    r.f("TightCornerThreshold", g!(F, tight_corner_threshold), 0, 0x4120_0000, 0.05);
    r.f("BrakeThrotScale", g!(F, brake_throt_scale), 0, 0x41A0_0000, 0.05);
    r.f("RacelineOffsetScale", g!(F, raceline_offset_scale), 0, 0x3F80_0000, 0.01);
    r.b("Force Speed Match", g!(B, force_speed_match));
    r.f("Force Speed", g!(F, force_speed), 0, 0x42C8_0000, 1.0);
    r.sys("AI.OpponentHandling.Individual");
    r.e("Base_Aggression", g!(U, base_aggression), "Low = 0, Medium = 1, High = 2");
    r.e("Skill", g!(U, skill), "Low = 0, Medium = 1, High = 2");
    r.f("BrakeFactor", g!(F, handling_override.brake_factor), 0, 0x4000_0000, 0.01);
    r.f("SteeringFactor", g!(F, handling_override.steering_factor), 0x3DCC_CCCD, 0x4170_0000, 0.5);
    r.f("ThrottleFactor", g!(F, handling_override.throttle_factor), 0, 0x4000_0000, 0.01);
    r.f("LookAheadMax", g!(F, handling_override.look_ahead_max), 0x40A0_0000, 0x42C8_0000, 1.0);
    r.f("LookAheadMin", g!(F, handling_override.look_ahead_min), 0x40A0_0000, 0x42C8_0000, 1.0);
    r.f("MaxSpeed", g!(F, handling_override.max_speed), 0x4120_0000, 0x42C8_0000, 1.0);
    r.f("SteeringDampingFactor", g!(F, handling_override.steering_damping_factor), 0x3F80_0000, 0x42C8_0000, 1.0);

    r.sys("AI.Shortcuts");
    r.f("Max distance to player", g!(F, shortcut_max_distance_to_player), 0x4120_0000, 0x447A_0000, 10.0);
    r.f("Max player prestige", g!(F, shortcut_max_player_prestige), 0x42C8_0000, 0x4743_5000, 100.0);
    r.f("Distance contribution", g!(F, shortcut_distance_contribution), 0, 0x3F80_0000, 0.001);
    r.f("Skill contribution", g!(F, shortcut_skill_contribution), 0, 0x3F80_0000, 0.001);
    r.f("Aggression contribution", g!(F, shortcut_aggression_contribution), 0, 0x3F80_0000, 0.001);
    r.f("Prestige contribution", g!(F, shortcut_prestige_contribution), 0, 0x3F80_0000, 0.001);

    r.sys("AI.Failures");
    r.b("Enabled", g!(B, failures_enabled));
    r.f("Min AI car health", g!(F, min_ai_car_health), 0x3F80_0000, 0x447A_0000, 10.0);
    r.f("Min player car health", g!(F, min_player_car_health), 0x3F80_0000, 0x447A_0000, 10.0);
    r.f("Max AI car health", g!(F, max_ai_car_health), 0x3F80_0000, 0x447A_0000, 10.0);
    r.f("Max player car health", g!(F, max_player_car_health), 0x3F80_0000, 0x447A_0000, 10.0);
    r.f("Max health at prestige", g!(F, max_health_at_prestige), 0, 0x47C3_5000, 100.0);
    r.f("AI damage multiplier from player", g!(F, ai_damage_multiplier_from_player), 0, 0x4080_0000, 0.1);
    r.f("Blown engine oil spill min delay", g!(F, oil_spill_min_delay), 0x3DCC_CCCD, 0x40A0_0000, 0.1);
    r.f("Blown engine oil spill max delay", g!(F, oil_spill_max_delay), 0x3DCC_CCCD, 0x40A0_0000, 0.1);
    r.f("Blown engine oil spill min radius", g!(F, oil_spill_min_radius), 0x3DCC_CCCD, 0x4000_0000, 0.1);
    r.f("Blown engine oil spill max radius", g!(F, oil_spill_max_radius), 0x3DCC_CCCD, 0x4000_0000, 0.1);
    r.f("Blown engine oil spill friction mod", g!(F, oil_spill_friction_mod), 0x3DCC_CCCD, 0x4120_0000, 0.05);
    r.f("Blown engine oil spill lifetime", g!(F, oil_spill_lifetime), 0x4120_0000, 0x447A_0000, 10.0);
    r.f("NOS explosion velocity increment", g!(F, nos_explosion_velocity_increment), 0, 0x42C8_0000, 5.0);
    r.f("Overrev damage scale", g!(F, overrev_damage_scale), 0, 0x42C8_0000, 1.0);
    r.e("Force_failure", g!(U, force_failure), "EngineBlown = 0, NOSExplosion = 1, TyrePuncture = 2");

    r.sys("AI.StartlineRevs");
    r.f("Min Time Multiplier", g!(F, revs_min_time_multiplier), 0x3F00_0000, 0x3FC0_0000, 0.05);
    r.f("Max Time Multiplier", g!(F, revs_max_time_multiplier), 0x3F00_0000, 0x3FC0_0000, 0.05);
    r.f("Min Target Revs", g!(F, revs_min_target), 0x459C_4000, 0x45FA_0000, 100.0);
    r.f("Max Target Revs", g!(F, revs_max_target), 0x459C_4000, 0x45FA_0000, 100.0);
    r.f("Min Ramp Time Start", g!(F, revs_min_ramp_time_start), 0x3F00_0000, 0x4080_0000, 0.1);
    r.f("Max Ramp Time Start", g!(F, revs_max_ramp_time_start), 0x3F00_0000, 0x4080_0000, 0.1);
    r.f("Min Hold Time Start", g!(F, revs_min_hold_time_start), 0x3F00_0000, 0x4080_0000, 0.1);
    r.f("Max Hold Time Start", g!(F, revs_max_hold_time_start), 0x3F00_0000, 0x4080_0000, 0.1);
    r.f("Blend Factor", g!(F, revs_blend_factor), 0x3DCC_CCCD, 0x3F66_6666, 0.05);
}

/// the TFTCharacter enum body: "<opponent name> = i, ..." over the race manager's ai cars (+0x38 count, +0x24
/// cars, car +0x234 driver, driver +4 opponent, opponent +0 name). A driver without an opponent gives "" (the
/// original strcats through NULL).
pub fn tft_character_enum(w: &AiWorld) -> String {
    let n = w.race.ai.len();
    let mut s = String::new();
    for i in 0..n {
        if let Some(o) = w.race.drivers.get(i).and_then(|d| d.opponent.as_ref()) {
            s.push_str(&o.name);
        }
        s.push_str(" = ");
        s.push_str(&i.to_string());
        if i + 1 < n {
            s.push_str(", ");
        }
    }
    s
}

/// VsAIPlayerTFTManager.cpp 0x00016C30
pub fn add_tft_tweaks(tm: &mut CTweakManager, t: &mut AiTweaks, w: &mut AiWorld) {
    let chars = tft_character_enum(w);
    log::debug!("Tweak Add TFT");
    let mut r = Reg { tm, t, w, sys: CTweakManager::ROOT, path: String::new() };
    r.sys("TFT");
    r.f("Time Between TFTs", tft!(F, time_between), 0x3F80_0000, 0x41F0_0000, 0.5);
    r.f("Time Between Same Char TFTs", tft!(F, time_between_same), 0x3F80_0000, 0x41F0_0000, 0.5);
    r.e("TFTCharacter", tft!(U, debug_character), &chars);
    r.e(
        "MovieType",
        tft!(U, debug_movie_type),
        "ChitChat = 0, GenericAnger = 1, BeingHitBig = 2, GenericJoy = 3, Winning = 4, StuntSuccess = 5, Lost = 6, BeingOvertaken = 7, MakeMistake = 8, Special = 9",
    );
    r.i("Type Choice", tft!(U, debug_type_choice), 0, 4, 1);
    r.b("Play Movie", tft!(B, debug_play));
    r.b("Movie On TFT", tft!(B, movie_on_tft));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::vs_ai_driver::VsAIDriver;
    use crate::ai::vs_ai_race_vehicle::VsAIRaceVehicle;
    use crate::ai::vs_ai_response_curve::VsAIResponseCurve;
    use crate::ai::vs_ai_world::VsAIHandling;
    use crate::opponent_database::OpponentEntry;
    use visos::type_::vs_type::CTypeKind;

    fn bits(d: &TweakData) -> String {
        match d {
            TweakData::Float(f) => format!("{:08X}", f.to_bits()),
            TweakData::Int(i) => i.to_string(),
            other => format!("{other:?}"),
        }
    }

    /// a world with player car 0 on a 9-point speed curve and two named opponents
    fn world() -> AiWorld {
        let mut w = AiWorld::default();
        w.world.response_curves.push(VsAIResponseCurve {
            items: (0..9).map(|k| (k as f32 * 0.125, k as f32 * 0.1)).collect(),
            ..Default::default()
        });
        w.world.handlings.push(VsAIHandling { speed_ratio_rc: 0, ..Default::default() });
        let mut p = VsAIRaceVehicle::default();
        p.base.handling = Some(0);
        w.race.players.push(p);
        for n in ["AL", "BUD"] {
            w.race.ai.push(VsAIRaceVehicle::default());
            let mut d = VsAIDriver::default();
            d.opponent = Some(OpponentEntry { name: n.into(), ..Default::default() });
            w.race.drivers.push(d);
        }
        w
    }

    fn register(tm: &mut CTweakManager, t: &mut AiTweaks, w: &mut AiWorld) {
        add_tweaks(tm, t, w);
        add_tft_tweaks(tm, t, w);
    }

    /// the tree (systems in creation order, kinds, ranges, steps) equals the decompiles of 0x0002D360 / 0x00016C30
    /// (`ai_tweaks_original.txt`, generated by tools/ai_tweak_table.py)
    #[test]
    fn registered_tree_matches_the_original() {
        let (mut tm, mut t, mut w) = (CTweakManager::default(), AiTweaks::default(), world());
        register(&mut tm, &mut t, &mut w);
        let mut got = Vec::new();
        for (i, s) in tm.systems.iter().enumerate().skip(1) {
            let mut path = vec![s.name.clone()];
            let mut p = s.parent;
            while let Some(q) = p.filter(|&q| q != CTweakManager::ROOT) {
                path.insert(0, tm.systems[q].name.clone());
                p = tm.systems[q].parent;
            }
            got.push(format!("S|{}", path.join(".")));
            for v in &tm.systems[i].tweaks {
                got.push(match v.kind {
                    CTypeKind::Float => format!("F|{}|{}|{}|{}", v.name, bits(&v.min), bits(&v.max), bits(&v.step)),
                    CTypeKind::Integer => format!("I|{}|{}|{}|{}", v.name, bits(&v.min), bits(&v.max), bits(&v.step)),
                    CTypeKind::Enum => format!("E|{}", v.name),
                    CTypeKind::Bool => format!("B|{}", v.name),
                    k => format!("?|{}|{k:?}", v.name),
                });
                assert!(!v.read_only, "{}", v.name);
            }
        }
        let want: Vec<&str> = include_str!("ai_tweaks_original.txt").lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(got.len(), want.len());
        for (g, w) in got.iter().zip(&want) {
            assert_eq!(g, w);
        }
        // head insertion: the newest top-level system comes first
        let root: Vec<&str> = tm.systems[0].children.iter().map(|&c| tm.systems[c].name.as_str()).collect();
        assert_eq!(root, ["TFT", "AI"]);
        // enum bodies
        let e = |p: &str, n: &str| {
            let s = tm.find_system(p).unwrap();
            tm.tweak(tm.find_tweak(s, n).unwrap()).enumerators.clone()
        };
        assert_eq!(e("TFT", "TFTCharacter"), [("AL".to_string(), 0), ("BUD".to_string(), 1)]);
        assert_eq!(e("TFT", "MovieType").len(), 10);
        assert_eq!(e("AI.Failures", "Force_failure")[2], ("TyrePuncture".to_string(), 2));
        assert_eq!(e("AI.OpponentHandling.Individual", "Skill").len(), 3);
    }

    /// defaults come from the live variables; menu edits reach them and survive into the next race's world
    #[test]
    fn tweaks_edit_the_live_values() {
        let (mut tm, mut t, mut w) = (CTweakManager::default(), AiTweaks::default(), world());
        register(&mut tm, &mut t, &mut w);
        let id = |tm: &CTweakManager, p: &str, n: &str| tm.find_tweak(tm.find_system(p).unwrap(), n).unwrap();
        let ds = id(&tm, "AI", "Difficulty Scale");
        assert_eq!(tm.tweak(ds).default, TweakData::Float(f32::from_bits(0x3F19_999A)));
        assert_eq!(tm.tweak(id(&tm, "AI.OpponentHandling.Global", "Min Slowdown at Dist")).default, TweakData::Float(75.0));
        assert_eq!(tm.tweak(id(&tm, "TFT", "Movie On TFT")).default, TweakData::Bool(true));
        assert_eq!(tm.tweak(id(&tm, "TFT", "Time Between TFTs")).default, TweakData::Float(10.0));
        assert_eq!(tm.tweak(id(&tm, "AI.PlayerAISpeedRC", "0.5")).default, TweakData::Float(0.4));

        // the menu's RIGHT press
        tm.tweak_mut(ds).increment(1.0);
        tm.tweak_mut(id(&tm, "AI.PlayerAISpeedRC", "1.0")).decrement(1.0);
        tm.tweak_mut(id(&tm, "AI.Failures", "Enabled")).increment(1.0);
        tm.tweak_mut(id(&tm, "TFT", "Type Choice")).increment(1.0);
        t.tweaks_out(&mut w);
        assert_eq!(w.world.globals.difficulty_scale, f32::from_bits(0x3F19_999A) + 0.05);
        assert_eq!(w.world.response_curves[0].items[8].1, 0.8 - 0.001);
        assert!(!w.world.globals.failures_enabled);
        assert_eq!(w.race.tft.debug_type_choice, 1);

        // the ai's own write shows in the cell
        w.world.globals.tight_corner_threshold = 1.25;
        t.tweaks_in(&mut w);
        assert_eq!(t.cell("AI.OpponentHandling.Global/TightCornerThreshold").unwrap().f32(), 1.25);

        // a new race: statics carry over, the speed curve is the file's again
        let mut w2 = world();
        t.push_statics(&mut w2);
        assert_eq!(w2.world.globals.difficulty_scale, w.world.globals.difficulty_scale);
        assert_eq!(w2.race.tft.debug_type_choice, 1);
        assert_eq!(w2.world.response_curves[0].items[8].1, 0.8);
        let mut tm2 = CTweakManager::default();
        register(&mut tm2, &mut t, &mut w2);
        assert_eq!(tm2.tweak(id(&tm2, "AI", "Difficulty Scale")).default, TweakData::Float(w.world.globals.difficulty_scale));
        assert_eq!(t.cell("AI.PlayerAISpeedRC/1.0").unwrap().f32(), 0.8);
    }
}
