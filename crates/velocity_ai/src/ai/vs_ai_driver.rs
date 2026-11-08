//! The personality behind an ai car: aggression, mistakes, collision reactions and TFT lines
//! (src/Ai/VsAIDriver.cpp; constructor/reset inlines sit in Armco.obj).
//!
//! Not ported: the debug text (0x00015A00). The aggression colour gradient is `name_colour` (name tags).

use super::vs_ai_globals::VsAIGlobals;
use super::vs_ai_player_tft_manager::{TftPlayer, VsAIPlayerTFTManager, tft};
use super::vs_ai_race_vehicle::{AiScene, VehicleRef, VsAIRaceVehicle, stat_position};
use super::vs_math::{VsRand, vs_abs, vs_fge, vs_fle, vs_min};
use crate::opponent_database::OpponentEntry;

/// which race manager driver array a driver lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DriverRef {
    Ai(usize),
    Player(usize),
    Police(usize),
}

/// eAIA levels: 0 low, 1 medium, 2 high, (3 for cops), 4 = not set.
pub const AIA_NOT_SET: u32 = 4;

/// eMistake (`+0x50`): 0 brake late, 1 skid out, 2 misalign, 3 none.
pub const MISTAKE_NONE: u32 = 3;

/// what a driver update needs besides its car.
pub struct DriverFrame<'a> {
    pub time: f32,          // VsAIWorld::GetTime
    pub race_clock: f32,    // g_GameManager +0x208
    pub speech_enabled: bool, // byte_6F915E
    pub num_players: u32,   // VsRaceManager +0x3C
    pub globals: &'a VsAIGlobals,
    pub rand: &'a mut VsRand,
    pub tft: &'a mut VsAIPlayerTFTManager,
    pub tft_out: &'a mut dyn TftPlayer,
}

/// `VsAIDriver` (120 bytes).
#[derive(Debug, Clone)]
pub struct VsAIDriver {
    pub vehicle: VehicleRef,          // +0x00 m_pVehicle
    pub opponent: Option<OpponentEntry>, // +0x04
    pub last_line_time: f32,          // +0x08
    pub last_line: [u32; 12],         // +0x0C per tft event
    pub temperament: u32,             // +0x3C aggression level 0..3
    pub aggression: f32,              // +0x40 0..100
    pub mistake: u32,                 // +0x50
    pub mistake_start: f32,           // +0x54
    pub mistake_end: f32,             // +0x58
    pub last_position: u32,           // +0x5C
    pub collided_with: VehicleRef,    // +0x60
    pub collided_from_behind: bool,   // +0x64
    pub collision_time: f32,          // +0x68
    pub impact_speed: f32,            // +0x6C
    pub overtaken_time: f32,          // +0x70
    pub disoriented_until: f32,       // +0x74
}

impl Default for VsAIDriver {
    /// Armco.cpp 0x00014350
    fn default() -> Self {
        Self {
            vehicle: VehicleRef::None,
            opponent: None,
            last_line_time: 0.0,
            last_line: [0; 12],
            temperament: 0,
            aggression: 0.0,
            mistake: MISTAKE_NONE,
            mistake_start: 0.0,
            mistake_end: 0.0,
            last_position: 0,
            collided_with: VehicleRef::None,
            collided_from_behind: false,
            collision_time: -1.0,
            impact_speed: 0.0,
            overtaken_time: -1.0,
            disoriented_until: 0.0,
        }
    }
}

/// the driver's colour gradient (+0x44, set by the ctor 0x000143F0 through 0x00015470): green 0x77FD8C, yellow
/// 0x77FD84, red 0x77FDA8 (static VISOS_ColourXBOX(0x80, ..) ctors 0x431AF0/0x431BD0/0x431AD0: each byte min(2x, 255));
/// positions i / (n - 1), the last exactly 1.0
const AGGRESSION_COLOURS: [(u32, f32); 3] = [(0xFF00_FF00, 0.0), (0xFFFF_FF00, 0.5), (0xFFFF_0000, 1.0)];

/// VsAIDriver.cpp 0x00015ED0: D3DCOLOR -> (r, g, b, a) floats (x = byte 2, y = byte 1, z = byte 0, w = byte 3)
fn colour_to_vec(c: u32) -> [f32; 4] {
    let k = f32::from_bits(0x3B80_8081); // 0.003921569
    let b = c.to_le_bytes();
    [b[2] as f32 * k, b[1] as f32 * k, b[0] as f32 * k, b[3] as f32 * k]
}

/// colour at `t` (VsAIDriver.cpp 0x000156C0): the last colour at the last entry, an exact position match its colour,
/// between two positions `a - (a - b) * f` per channel (x87, stored as float), then 0x00013940: each channel
/// `_ftol(c * 255.0)` into A/R/G/B
fn colour_gradient(g: &[(u32, f32)], t: f32) -> u32 {
    for i in 0..g.len() {
        if i == g.len() - 1 {
            return g[i].0;
        }
        if t == g[i].1 {
            return g[i].0;
        }
        if t > g[i].1 && g[i + 1].1 > t {
            let f = (t - g[i].1) / (g[i + 1].1 - g[i].1);
            let (a, b) = (colour_to_vec(g[i].0), colour_to_vec(g[i + 1].0));
            let ch = |k: usize| (a[k] as f64 - (a[k] as f64 - b[k] as f64) * f as f64) as f32;
            let byte = |v: f32| ((v as f64 * 255.0) as i32) as u8 as u32;
            let (r, gr, bl, al) = (ch(0), ch(1), ch(2), ch(3));
            return (byte(al) << 24) | (byte(r) << 16) | (byte(gr) << 8) | byte(bl);
        }
    }
    // "0" assert (VsAIDriver.cpp line 621), then grey 0x77FD9C; unreachable with entries
    0xFFFF_FFFF
}

impl VsAIDriver {
    /// Armco.cpp 0x000143F0
    pub fn init(&mut self, tft: &VsAIPlayerTFTManager) {
        self.last_line = [0; 12];
        self.reset(tft);
    }

    /// Armco.cpp 0x00014450
    pub fn reset(&mut self, tft: &VsAIPlayerTFTManager) {
        self.aggression = 0.0;
        self.last_position = 0;
        self.mistake = MISTAKE_NONE;
        self.collided_with = VehicleRef::None;
        self.collision_time = -1.0;
        self.impact_speed = 0.0;
        self.overtaken_time = -1.0;
        self.last_line_time = -tft.get_time_between_same();
        self.disoriented_until = 0.0;
    }

    /// VsAIDriver.cpp 0x000147E0
    pub fn update(&mut self, me: &VsAIRaceVehicle, scene: &AiScene, f: &mut DriverFrame, dt: f32) {
        debug_assert!(self.vehicle != VehicleRef::None, "m_pVehicle");
        debug_assert!(
            self.get_base_aggression_level(me) != AIA_NOT_SET && self.get_skill_level(me) != AIA_NOT_SET,
            "GetBaseAggressionLevel() != eAIA_NotSet && GetSkillLevel() != eAIA_NotSet"
        );
        self.update_aggression(me, scene, f, dt);
        // VsAIDriver.cpp 0x00014C00 is empty
        self.update_mistakes(me, f);
        self.update_speech(me, scene, f);
        self.last_position = stat_position(&me.stats) as u32;
    }

    /// aggression builds up at a temperament rate scaled by skill, ten times faster (twice for ai)
    /// with a car right behind.
    /// VsAIDriver.cpp 0x00014900
    fn update_aggression(&mut self, me: &VsAIRaceVehicle, scene: &AiScene, f: &DriverFrame, dt: f32) {
        if f.race_clock < 10.0 {
            return;
        }
        let g = f.globals;
        let mut rate = match self.temperament {
            0 => g.unk_1c4[0],
            1 => g.unk_1c4[1],
            2 => g.unk_1c4[2],
            3 => g.unk_1c4[3],
            _ => {
                debug_assert!(false);
                0.0
            }
        };
        let skill_scale = match self.get_skill_level(me) {
            0 => g.unk_1c4[4],
            1 => g.unk_1c4[5],
            2 => g.unk_1c4[6],
            _ => {
                debug_assert!(false);
                0.0
            }
        };
        let mut r = rate * skill_scale;
        if let Some(behind) = scene.get(me.vehicle_behind)
            && behind.race_section == me.race_section
            && behind.speed >= me.base.speed - 5.0
        {
            let gap = me.dist_along - behind.dist_along;
            if gap > 0.0 && gap < 15.0 {
                if behind.is_player {
                    r *= 10.0;
                } else {
                    r *= 2.0;
                }
            }
        }
        rate = r;
        self.aggression += rate * dt;
        if self.aggression > 100.0 {
            self.aggression = 100.0;
        }
        if self.aggression < 0.0 {
            self.aggression = 0.0;
        }
    }

    /// picks a mistake that suits the road; writes how long it lasts.
    /// VsAIDriver.cpp 0x00014C20
    fn choose_mistake(&self, me: &VsAIRaceVehicle, g: &VsAIGlobals, duration: &mut f32) -> u32 {
        if g.max_misalign_road_width * g.max_misalign_road_width <= me.road_width_sq {
            if vs_abs(me.corner_tightness) <= g.min_brake_late_corner_tightness {
                if vs_abs(me.steer_angle) <= g.min_skid_out_steering {
                    *duration = 0.0;
                    MISTAKE_NONE
                } else {
                    *duration = g.skid_out_time;
                    1
                }
            } else {
                *duration = g.brake_late_time;
                0
            }
        } else {
            *duration = g.misalign_time;
            2
        }
    }

    /// VsAIDriver.cpp 0x00014D00
    fn end_mistake(&mut self) {
        self.mistake = MISTAKE_NONE;
        self.mistake_start = 0.0;
        self.mistake_end = 0.0;
        self.aggression *= 0.2;
    }

    /// a full aggression meter makes the driver slip up.
    /// VsAIDriver.cpp 0x00014D60
    fn update_mistakes(&mut self, me: &VsAIRaceVehicle, f: &mut DriverFrame) {
        if !f.globals.make_mistakes {
            return;
        }
        if !vs_fge(self.aggression, 100.0, 0.000001) {
            return;
        }
        if self.mistake == MISTAKE_NONE {
            let mut d = 0.0;
            self.mistake = self.choose_mistake(me, f.globals, &mut d);
            self.mistake_start = f.time;
            self.mistake_end = self.mistake_start + d;
        }
        if self.mistake != MISTAKE_NONE {
            self.speak(tft::MAKE_MISTAKE, me, f);
            if f.time > self.mistake_end {
                self.end_mistake();
            }
        }
    }

    /// VsAIDriver.cpp 0x00014E60
    fn update_speech(&mut self, me: &VsAIRaceVehicle, scene: &AiScene, f: &mut DriverFrame) {
        if !f.speech_enabled || !me.started {
            return;
        }
        let mut gained = false;
        if self.check_overtake(me, scene, f.time, &mut gained) {
            self.speak(if gained { tft::GENERIC_JOY } else { tft::BEING_OVERTAKEN }, me, f);
        }
        let mut from_player = false;
        if self.consume_collision(scene, &mut from_player) {
            if from_player {
                self.speak(tft::GENERIC_JOY, me, f);
            } else if f.globals.big_collision_threshold <= self.impact_speed {
                self.speak(tft::BEING_HIT_BIG, me, f);
            } else {
                self.speak(tft::GENERIC_ANGER, me, f);
            }
        }
    }

    /// passing / being passed by the player since the last update.
    /// VsAIDriver.cpp 0x00014F90
    fn check_overtake(&mut self, me: &VsAIRaceVehicle, scene: &AiScene, time: f32, gained: &mut bool) -> bool {
        let pos = stat_position(&me.stats) as u32;
        let p0 = scene.player0().map_or(0, |p| p.race_position as u32);
        if self.last_position == 0 {
            self.last_position = pos;
        }
        let behind = scene.get(me.vehicle_behind);
        let ahead = scene.get(me.vehicle_ahead);
        if behind.is_some_and(|b| b.is_player) && pos < self.last_position && p0 == pos + 1 {
            *gained = true;
            self.overtaken_time = -1.0;
            true
        } else if ahead.is_some_and(|a| a.is_player) && pos > self.last_position && p0 == pos.wrapping_sub(1) {
            *gained = false;
            self.overtaken_time = time;
            true
        } else {
            false
        }
    }

    /// records a hit; a hard one from behind leaves the driver disoriented.
    /// VsAIDriver.cpp 0x000150B0
    pub fn on_collision(&mut self, other: VehicleRef, from_behind: bool, impact: f32, g: &VsAIGlobals, time: f32) {
        self.collided_with = other;
        self.collided_from_behind = from_behind;
        self.impact_speed = impact;
        if other != VehicleRef::None {
            self.collision_time = time;
        }
        if from_behind && impact > g.disorient_min_impact_speed {
            let ratio = (vs_min(impact, g.disorient_max_impact_speed) - g.disorient_min_impact_speed)
                / (g.disorient_max_impact_speed - g.disorient_min_impact_speed);
            debug_assert!(
                vs_fge(ratio, 0.0, 0.000001) && vs_fle(ratio, 1.0, 0.000001),
                "VsFge(ratio, 0.0f) && VsFle(ratio, 1.0f)"
            );
            self.disorient(ratio * g.disorient_max_delay, time);
        }
    }

    /// VsAIDriver.cpp 0x00015200
    pub fn get_collision(&self) -> (VehicleRef, f32, f32) {
        (self.collided_with, self.collision_time, self.impact_speed)
    }

    /// VsAIDriver.cpp 0x00015240
    pub fn disorient(&mut self, t: f32, time: f32) {
        let until = time + t;
        if until > self.disoriented_until {
            self.disoriented_until = until;
        }
    }

    /// VsAIRaceVehicle.h 0x00024EB0
    pub fn is_disoriented(&self, time: f32) -> bool {
        self.disoriented_until > time
    }

    /// only a hit by a player is reported, and then always as `from_player`.
    /// VsAIDriver.cpp 0x000152A0
    fn consume_collision(&mut self, scene: &AiScene, from_player: &mut bool) -> bool {
        if self.collided_with == VehicleRef::None || !scene.get(self.collided_with).is_some_and(|v| v.is_player) {
            return false;
        }
        *from_player = true;
        self.collided_with = VehicleRef::None;
        true
    }

    /// VsAIDriver.cpp 0x000152F0
    pub fn get_base_aggression_level(&self, me: &VsAIRaceVehicle) -> u32 {
        if me.is_player { 1 } else { self.opponent.as_ref().map_or(AIA_NOT_SET, |o| o.base_aggression) }
    }
    /// VsAIDriver.cpp 0x00015330
    pub fn get_skill_level(&self, me: &VsAIRaceVehicle) -> u32 {
        if me.is_player { 1 } else { self.opponent.as_ref().map_or(AIA_NOT_SET, |o| o.skill) }
    }
    /// VsAIDriver.cpp 0x00015370
    pub fn get_level_50(&self, me: &VsAIRaceVehicle) -> u32 {
        if me.is_player { 1 } else { self.opponent.as_ref().map_or(AIA_NOT_SET, |o| o.level_50) }
    }
    /// VsAIDriver.cpp 0x000153B0
    pub fn get_name<'a>(&'a self, me: &VsAIRaceVehicle) -> Option<&'a str> {
        if me.is_player { None } else { self.opponent.as_ref().map(|o| o.name.as_str()) }
    }
    /// aggression colour (name tags, debug text): the +0x44 gradient at aggression * 0.01, as a D3DCOLOR.
    /// VsAIDriver.cpp 0x00015F80
    pub fn name_colour(&self) -> u32 {
        colour_gradient(&AGGRESSION_COLOURS, self.aggression * 0.01)
    }
    /// VsAIDriver.cpp 0x00015F60
    pub fn get_temperament(&self) -> u32 {
        self.temperament
    }
    /// the event's competitor; temperament starts at its base aggression. Called from the
    /// physics manager's race setup (PhysicsManager.cpp 0x0019E080), compiled into its region.
    /// VsAIDriver.cpp 0x001B4120
    pub fn set_opponent(&mut self, o: crate::opponent_database::OpponentEntry) {
        self.temperament = o.base_aggression;
        self.opponent = Some(o);
    }
    /// VsAIDriver.cpp 0x00015D90
    pub fn get_aggression(&self) -> f32 {
        self.aggression
    }

    /// says a line of the given tft event when the per-driver cooldown allows.
    /// Armco.cpp 0x00014510
    pub fn speak(&mut self, event: u32, me: &VsAIRaceVehicle, f: &mut DriverFrame) {
        let Some(op) = &self.opponent else { return };
        let lines = &op.string_lists[event as usize];
        if lines.is_empty() {
            return;
        }
        let cooldown = f.tft.get_time_between_same();
        if f.time <= cooldown + self.last_line_time {
            return;
        }
        let idx = self.pick_line(event, f.rand);
        let line = &lines[idx as usize];
        let name = if f.tft.has_vcds() { line.clone() } else { format!("TFT/{}/{}", op.name, line) };
        let slot = me.id as i32 - f.num_players as i32;
        if f.tft.play_line(&name, slot, 0.5, f.time, f.speech_enabled, f.tft_out) {
            self.last_line_time = f.time;
            self.last_line[event as usize] = idx;
        }
    }

    /// random line, avoiding the previous one when there is a choice.
    /// Armco.cpp 0x000146E0
    fn pick_line(&self, event: u32, rand: &mut VsRand) -> u32 {
        let n = self.opponent.as_ref().map_or(0, |o| o.string_lists[event as usize].len()) as u32;
        let mut idx = 0;
        if n > 1 {
            let mut tries = 5;
            let mut ok = false;
            while !ok && tries > 0 {
                let hi = (n - 1) as f32;
                let r = (rand.unit() as f64 * n as f64) as f32;
                idx = vs_min(r, hi) as i32 as u32;
                if idx == self.last_line[event as usize] {
                    tries -= 1;
                } else {
                    ok = true;
                }
            }
            if !ok {
                return (self.last_line[event as usize] + 1) % n;
            }
        }
        idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_colour_runs_green_yellow_red() {
        let c = |a: f32| colour_gradient(&AGGRESSION_COLOURS, a * 0.01);
        assert_eq!(c(0.0), 0xFF00_FF00);
        assert_eq!(c(25.0), 0xFF7F_FF00);
        assert_eq!(c(50.0), 0xFFFF_FF00);
        assert_eq!(c(100.0), 0xFFFF_0000);
        assert_eq!(c(150.0), 0xFFFF_0000);
        assert_eq!(c(-5.0), 0xFFFF_0000);
    }
}
