//! src\ScoreManager\ScoreManager.cpp — per player car style scoring: clean start, overtakes, top speed, drifts,
//! handbrake turns, 360s, slipstream, racing line, near misses, start / in-race gear switches, laps. Points
//! pile up in a combo that is banked after a quiet spell or lost on a crash; they also fill the zone and
//! nitrous meters. The object hangs off the ai race car (`VsAIRaceVehicle+0x1C8`), which updates it while
//! the session is in Go / Racing. The idb labels most of it ReplayCamera_Wheel (0x00168F70..0x00169E30).
//!
//! The car is read through `ScoreInputs`, filled by the caller each update; the race statistics it writes
//! (`VsAIRaceVehicle` 0x000244E0 block) are kept here as `RaceStats`.

use bevy::math::Vec3;

/// "Game.ScoreManager" tweaks (0x00167F30); values as shipped (ints are points)
#[derive(Clone, Debug, PartialEq)]
pub struct ScoreTweaks {
    pub multiplier_reset_time: f32,  // dword_6876A0 3
    pub zone_multiplier: f32,        // dword_68769C 2
    pub infinite_zone: bool,         // byte_749CCC (bss)
    pub max_zone_time: f32,          // unk_68774C 10
    pub zone_per_score: f32,         // flt_687750 0.6667
    pub zone_per_crash: f32,         // dword_687754 0.5
    pub in_zone_increment: f32,      // flt_687758 1.5
    pub nos_per_score: f32,          // flt_68775C 0.5
    pub overtake: i32,               // dword_6876B4 20
    pub clean_overtake: i32,         // dword_6876B0 30
    pub overtake_time: f32,          // dword_6876B8 3
    pub top_speed: i32,              // dword_6876BC 10
    pub drift: i32,                  // dword_6876C0 20
    pub handbrake_turn: i32,         // dword_6876C4 15
    pub spin_time: f32,              // dword_6876C8 2 ("360 Spin time")
    pub spin: i32,                   // dword_6876CC 25
    pub place: [i32; 3],             // unk_6876D0.. 75 50 25
    pub lead_lap: i32,               // dword_6876DC 20
    pub clean_lap: i32,              // dword_6876E0 20
    pub fastest_lap: i32,            // dword_6876E4 10
    pub clean_start: i32,            // dword_6876A4 10
    pub clean_start_time: f32,       // dword_6876A8 3
    pub clean_start_min_speed: f32,  // dword_6876AC 30
    pub slipstream_enter: i32,       // dword_6876E8 20
    pub slipstream_min_time: f32,    // dword_6876EC 2
    pub slipstream_continuous: i32,  // dword_6876F0 1
    pub slipstream_interval: f32,    // dword_6876F4 1
    pub racing_line_start: i32,      // dword_6876F8 30
    pub racing_line_max_dist: f32,   // flt_6876FC 1
    pub racing_line_min_speed: f32,  // dword_687700 40
    pub racing_line_min_time: f32,   // dword_687704 2
    pub racing_line_continuous: i32, // dword_687708 1
    pub racing_line_interval: f32,   // dword_68770C 1
    pub near_miss: i32,              // dword_687718 25
    pub near_miss_min_speed: f32,    // dword_68771C 60
    pub near_miss_distance: f32,     // flt_687720 5.5
    pub escape_min_dist: f32,        // dword_687724 50
    pub escape_max_dist: f32,        // dword_687728 100
    pub escaped_police: i32,         // dword_68772C 250
    pub start_escaping: i32,         // dword_687730 25
    pub escaping: i32,               // dword_687734 5
    pub escaping_interval: f32,      // dword_687738 1
    /// start line switch scores bad / good / perfect / skid (bad and skid are bss: 0)
    pub start_switch: [f32; 4],
    /// in race switch scores bad / good / perfect
    pub race_switch: [f32; 3],
    /// dword_687768 -0.707: heading against the raceline below this dot = wrong way
    pub wrong_way_dot: f32,
    /// "Game.OSD" Split Time Display Duration `dword_6860EC` 2.0 (the front end's tweak; the binary copies it in)
    pub split_time_duration: f32,
}

impl Default for ScoreTweaks {
    fn default() -> Self {
        ScoreTweaks {
            multiplier_reset_time: 3.0,
            zone_multiplier: 2.0,
            infinite_zone: false,
            max_zone_time: 10.0,
            zone_per_score: f32::from_bits(0x3F2A_ACDA),
            zone_per_crash: 0.5,
            in_zone_increment: 1.5,
            nos_per_score: 0.5,
            overtake: 20,
            clean_overtake: 30,
            overtake_time: 3.0,
            top_speed: 10,
            drift: 20,
            handbrake_turn: 15,
            spin_time: 2.0,
            spin: 25,
            place: [75, 50, 25],
            lead_lap: 20,
            clean_lap: 20,
            fastest_lap: 10,
            clean_start: 10,
            clean_start_time: 3.0,
            clean_start_min_speed: 30.0,
            slipstream_enter: 20,
            slipstream_min_time: 2.0,
            slipstream_continuous: 1,
            slipstream_interval: 1.0,
            racing_line_start: 30,
            racing_line_max_dist: 1.0,
            racing_line_min_speed: 40.0,
            racing_line_min_time: 2.0,
            racing_line_continuous: 1,
            racing_line_interval: 1.0,
            near_miss: 25,
            near_miss_min_speed: 60.0,
            near_miss_distance: 5.5,
            escape_min_dist: 50.0,
            escape_max_dist: 100.0,
            escaped_police: 250,
            start_escaping: 25,
            escaping: 5,
            escaping_interval: 1.0,
            start_switch: [0.0, 50.0, 100.0, 0.0],
            race_switch: [0.0, 50.0, 100.0],
            wrong_way_dot: f32::from_bits(0xBF34_FDF4),
            split_time_duration: 2.0,
        }
    }
}

/// flags (+0xB8)
pub mod flag {
    pub const STARTED: u32 = 1;
    pub const CRASHED: u32 = 2;
    pub const CLEAN_START_DONE: u32 = 4;
    /// a place was gained and is being held
    pub const GAINED_PLACE: u32 = 8;
    pub const BELOW_TOP_SPEED: u32 = 0x10;
    pub const CRASHED_THIS_LAP: u32 = 0x20;
    pub const PLACE_CHANGED: u32 = 0x40;
    /// falling or not upright: no scores
    pub const NOT_UPRIGHT: u32 = 0x80;
    pub const IN_SLIPSTREAM: u32 = 0x100;
    pub const WRONG_WAY: u32 = 0x200;
    pub const LAP_POS: [u32; 5] = [0x400, 0x800, 0x1000, 0x2000, 0x4000];
    pub const ON_RACING_LINE: u32 = 0x8000;
    pub const RACING_LINE_SCORED: u32 = 0x10000;
    pub const POLICE_NEAR: u32 = 0x20000;
    pub const ESCAPING: u32 = 0x40000;
    pub const START_SWITCH_DONE: u32 = 0x80000;
}

/// the race statistics the score manager writes (`VsAIRaceVehicle` 0x000244E0 block)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RaceStats {
    pub lap_time: f32,        // +0x0C
    pub fastest_lap: f32,     // +0x2C
    pub lead_time: f32,       // +0x44
    pub overtakes: u32,       // +0x48
    pub lap_score: u32,       // +0x4C
    pub best_lap_score: u32,  // +0x50
    pub score: u32,           // +0x54
    /// +0x24 time behind the car ahead / +0x28 time the car behind is behind; +0x5C / +0x60 those cars
    pub gap_behind: f32,
    pub gap_ahead: f32,
    pub car_behind: Option<u32>,
    pub car_ahead: Option<u32>,
}

/// what the score manager reads from the car this update.
#[derive(Clone, Debug, Default)]
pub struct ScoreInputs {
    /// race clock (GameManager+0x208)
    pub time: f32,
    /// counting (`VsAIRaceVehicle+0xF4`)
    pub racing: bool,
    pub position: u8,
    pub lap: u8,
    /// m/s (0x00024220)
    pub speed: f32,
    pub velocity: Vec3,
    pub forward: Vec3,
    /// contact normal of wheel 1 (0x000248C0)
    pub wheel_normal: Vec3,
    /// handbrake input (0x0002C7B0 + 0x10)
    pub handbrake: f32,
    /// slipstream strength (LightStreak 0x00130070)
    pub slipstream: f32,
    /// lateral distance from the raceline (0x00024A60)
    pub raceline_offset: f32,
    /// direction of the raceline segment the car is on (None off any race section)
    pub raceline_dir: Option<Vec3>,
    /// start line / in race gear switch results 0 none, 1 bad, 2 good, 3 perfect, 4 skid (vehicle +0x1338/+0x133C)
    pub start_switch: u32,
    pub race_switch: u32,
    /// last lap time (OSDInGameScreen 0x000FAEC0), the session's best (GameManager+0x20C)
    pub last_lap_time: f32,
    pub best_lap_time: f32,
    pub position_vec: Vec3,
    /// traffic car positions
    pub traffic: Vec<(u32, Vec3)>,
    /// police cars chasing this one (police manager target == this car, count > 0) and their positions
    pub chased_by_police: bool,
    pub police: Vec<Vec3>,
    /// lap split index of this car (`ScoreManagerChecks` 0x0016CCF0, race car +0x25C)
    pub split: u32,
    /// on a race section (0x00015C80) and the player's car (0x00023CC0)
    pub on_section: bool,
    pub is_player: bool,
    /// the car ahead in the race order and its time at this split (`VsAIRaceVehicle` 0x000244E0 +0x0C + 4n)
    pub ahead: Option<(u32, f32)>,
    /// the car behind and the distances along the race (0x00023DB0) of this car and that one
    pub behind: Option<(u32, f32)>,
    pub distance: f32,
}

/// what the score manager wants shown / played.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScoreEvent {
    /// an OSD message (string crc of the OSD_SCORE_* id)
    Message(u32),
    SlipstreamAudio(bool),
    Mode3Audio(bool),
    /// a new session best lap (GameManager 0x000BFFC0)
    BestLap(f32),
}

fn crc(s: &str) -> u32 {
    visos::handy::handy_crc::crc32(s.as_bytes(), 0)
}

#[derive(Clone, Debug)]
pub struct ScoreManager {
    pub last_time: f32,        // +0x00
    pub run_time: f32,         // +0x04
    /// +0x08 speed in mph (km/h * 0.625)
    pub speed: f32,
    pub last_lap: u8,          // +0x0C
    pub last_position: u8,     // +0x0D
    pub overtake_time: f32,    // +0x10
    pub clean_time: f32,       // +0x14
    pub combo_time: f32,       // +0x18
    pub handbrake_time: f32,   // +0x1C
    pub drift_time: f32,       // +0x20
    pub slip_time: f32,        // +0x24
    pub out_of_slip_time: f32, // +0x28
    pub line_time: f32,        // +0x2C
    pub line_scored_time: f32, // +0x30
    pub escape_time: f32,      // +0x34
    pub near_miss_car: Option<u32>, // +0x48
    pub near_miss_time: f32,   // +0x4C
    pub dt: f32,               // +0x50
    /// +0x54..+0x60 time since the car last faced -x, +x, -z, +z
    pub spin: [f32; 4],
    /// +0x74 the local player's (OSD and audio)
    pub local: bool,
    pub combo_score: u32,      // +0x98
    pub combo_count: u32,      // +0x9C
    pub zone: f32,             // +0xA0
    pub nos: f32,              // +0xA4
    pub wrong_way_time: f32,   // +0xA8
    pub lap_pos_time: f32,     // +0xAC
    pub ahead_shown: f32,      // +0xB0
    pub behind_shown: f32,     // +0xB4
    pub flags: u32,            // +0xB8
    pub stats: RaceStats,
    pub tw: ScoreTweaks,
}

impl Default for ScoreManager {
    fn default() -> Self {
        let mut s = ScoreManager {
            last_time: 0.0,
            run_time: 0.0,
            speed: 0.0,
            last_lap: 1,
            last_position: 0,
            overtake_time: 0.0,
            clean_time: 0.0,
            combo_time: 0.0,
            handbrake_time: 0.0,
            drift_time: 0.0,
            slip_time: 0.0,
            out_of_slip_time: 0.0,
            line_time: 0.0,
            line_scored_time: 0.0,
            escape_time: 0.0,
            near_miss_car: None,
            near_miss_time: 0.0,
            dt: 0.0,
            spin: [10000.0; 4],
            local: false,
            combo_score: 0,
            combo_count: 0,
            zone: 0.0,
            nos: 0.0,
            wrong_way_time: 0.0,
            lap_pos_time: 0.0,
            ahead_shown: 0.0,
            behind_shown: 0.0,
            flags: 0,
            stats: RaceStats::default(),
            tw: ScoreTweaks::default(),
        };
        s.reset();
        s
    }
}

fn flat_norm(v: Vec3) -> Vec3 {
    crate::camera::vs_base_camera::normalise(Vec3::new(v.x, 0.0, v.z))
}

impl ScoreManager {
    /// ReplayCamera_Wheel 0x001691B0 (constructor) — `local` = the player's own OSD
    pub fn new(local: bool) -> Self {
        ScoreManager { local, ..Default::default() }
    }

    /// ReplayCamera_Wheel 0x00168F70
    pub fn reset(&mut self) {
        let (tw, local) = (std::mem::take(&mut self.tw), self.local);
        *self = ScoreManager {
            last_time: 0.0,
            run_time: 0.0,
            speed: 0.0,
            last_lap: 1,
            last_position: 0,
            overtake_time: 0.0,
            clean_time: 0.0,
            combo_time: 0.0,
            handbrake_time: 0.0,
            drift_time: 0.0,
            slip_time: 0.0,
            out_of_slip_time: 0.0,
            line_time: 0.0,
            line_scored_time: 0.0,
            escape_time: 0.0,
            near_miss_car: None,
            near_miss_time: 0.0,
            dt: 0.0,
            spin: [10000.0; 4],
            local,
            combo_score: 0,
            combo_count: 0,
            zone: 0.0,
            nos: 0.0,
            wrong_way_time: 0.0,
            lap_pos_time: 0.0,
            behind_shown: 0.0,
            ahead_shown: 0.0,
            flags: 0,
            stats: self.stats.clone(),
            tw,
        };
    }

    /// ReplayCamera_Wheel 0x00169240 — a collision involving this car (after the start) is a crash
    pub fn on_collision(&mut self) {
        if self.flags & flag::STARTED != 0 {
            self.flags |= flag::CRASHED;
        }
    }

    /// ReplayCamera_Wheel 0x00169350 — timers, the crash penalty, banking the combo
    fn tick(&mut self, i: &ScoreInputs, in_zone: bool) {
        self.zone_tick_reset();
        if self.last_time <= 1e-8 {
            self.last_time = i.time;
            self.flags |= flag::STARTED;
        }
        self.dt = i.time - self.last_time;
        self.run_time += self.dt;
        if self.last_position == 1 {
            self.stats.lead_time += self.dt;
        }
        if i.racing {
            self.stats.lap_time += self.dt;
        }
        if self.flags & flag::CRASHED != 0 {
            self.clean_time = 0.0;
            self.flags &= !flag::CRASHED;
            self.flags |= flag::CRASHED_THIS_LAP;
            if self.combo_count > 1 {
                self.stats.score = self.stats.score.wrapping_add(self.combo_score);
                self.stats.lap_score = self.stats.lap_score.wrapping_add(self.combo_score);
            }
            self.combo_count = 0;
            self.combo_score = 0;
            if in_zone && self.zone > 0.0 {
                self.zone -= self.tw.zone_per_crash;
            }
        } else {
            self.clean_time += self.dt;
        }
        self.speed = (i.speed * 3.6) * 0.625;
        if i.handbrake == 0.0 {
            self.handbrake_time += self.dt;
        } else {
            self.handbrake_time = 0.0;
        }
        if self.flags & flag::IN_SLIPSTREAM != 0 {
            self.slip_time += self.dt;
        } else {
            self.out_of_slip_time += self.dt;
        }
        if self.flags & flag::WRONG_WAY != 0 {
            self.wrong_way_time += self.dt;
        } else {
            self.wrong_way_time = 0.0;
        }
        if self.flags & flag::ON_RACING_LINE != 0 && self.flags & flag::RACING_LINE_SCORED != 0 {
            self.line_scored_time += self.dt;
        } else if self.flags & flag::ON_RACING_LINE != 0 {
            self.line_time += self.dt;
        }
        if self.flags & flag::ESCAPING != 0 {
            self.escape_time += self.dt;
        }
        self.combo_time += self.dt;
        self.lap_pos_time += self.dt;
        if self.combo_time > self.tw.multiplier_reset_time {
            // the combo is banked times its length
            let banked = self.combo_count.wrapping_mul(self.combo_score);
            self.stats.score = self.stats.score.wrapping_add(banked);
            self.stats.lap_score = self.stats.lap_score.wrapping_add(banked);
            let n = self.combo_count as f64;
            self.zone = if in_zone {
                (n * self.tw.zone_per_score as f64 * self.tw.in_zone_increment as f64 + self.zone as f64) as f32
            } else {
                (n * self.tw.zone_per_score as f64 + self.zone as f64) as f32
            };
            self.nos = (self.combo_score as f64 * self.tw.nos_per_score as f64 * self.combo_count as f64) as f32;
            self.combo_count = 0;
            self.combo_score = 0;
            self.combo_time = 0.0;
        }
        if in_zone {
            self.zone -= self.dt;
        }
        if self.zone < 0.0 {
            self.zone = 0.0;
        }
    }

    /// +0xA0 / +0xA4 are cleared at the top of each update (0x00169350): they are the zone / nitrous gains of
    /// this update, read by the car afterwards
    fn zone_tick_reset(&mut self) {
        self.zone = 0.0;
        self.nos = 0.0;
    }

    fn score(&mut self, id: &str, points: f32, mult: f32, out: &mut Vec<ScoreEvent>) {
        if self.local {
            out.push(ScoreEvent::Message(crc(id)));
        }
        self.combo_score = self.combo_score.wrapping_add((points * mult) as i32 as u32);
    }

    /// ScoreManager.cpp 0x00169E30 — per update while racing (`in_zone` is always false from the ai car)
    pub fn update(&mut self, i: &ScoreInputs, in_zone: bool, out: &mut Vec<ScoreEvent>) {
        let mut mult = 1.0f32;
        if !i.racing {
            return;
        }
        self.check_upright(i);
        self.tick(i, in_zone);
        if in_zone {
            mult = self.tw.zone_multiplier;
        }
        if self.clean_start() {
            self.score("OSD_SCORE_CLEAN_START", self.tw.clean_start as f32, mult, out);
            self.combo_time = 0.0;
            self.combo_count += 1;
        }
        if let Some(clean) = self.overtake(i) {
            if clean {
                self.score("OSD_SCORE_CLEAN_OVERTAKE", self.tw.clean_overtake as f32, mult, out);
            } else {
                self.score("OSD_SCORE_OVERTAKE", self.tw.overtake as f32, mult, out);
            }
            self.combo_time = 0.0;
            self.combo_count += 1;
        }
        if self.top_speed() {
            self.score("OSD_SCORE_TOP_SPEED", self.tw.top_speed as f32, mult, out);
            self.combo_time = 0.0;
            self.combo_count += 1;
        }
        if let Some(handbrake) = self.drift(i) {
            if self.local {
                out.push(ScoreEvent::Mode3Audio(true));
            }
            if handbrake {
                self.score("OSD_SCORE_HANDBRAKE_TURN", self.tw.handbrake_turn as f32, mult, out);
            } else {
                self.score("OSD_SCORE_DRIFT_BONUS", self.tw.drift as f32, mult, out);
            }
            self.combo_time = 0.0;
            self.combo_count += 1;
        } else if self.drift_time > 1.0 && self.local {
            out.push(ScoreEvent::Mode3Audio(false));
        }
        if self.spin_360(i) {
            self.score("OSD_SCORE_360_TURN", self.tw.spin as f32, mult, out);
            self.combo_time = 0.0;
            self.combo_count += 1;
        }
        if let Some(entered) = self.slipstream(i) {
            if entered {
                self.score("OSD_SCORE_ENTERED_SLIPSTREAM", self.tw.slipstream_enter as f32, mult, out);
                self.combo_count += 1;
            } else {
                self.score("OSD_SCORE_SLIPSTREAM_BONUS", self.tw.slipstream_continuous as f32, mult, out);
            }
            if self.local {
                out.push(ScoreEvent::SlipstreamAudio(true));
            }
            self.combo_time = 0.0;
        } else if i.slipstream <= f32::from_bits(0x3DCC_CCCD) && self.local {
            out.push(ScoreEvent::SlipstreamAudio(false));
        }
        if let Some(started) = self.racing_line(i) {
            if started {
                self.score("OSD_SCORE_STARTED_ON_RACING_LINE", self.tw.racing_line_start as f32, mult, out);
                self.combo_count += 1;
            } else {
                self.score("OSD_SCORE_RACING_LINE_BONUS", self.tw.racing_line_continuous as f32, mult, out);
            }
            self.combo_time = 0.0;
        }
        if self.near_miss(i) {
            self.score("OSD_SCORE_NEAR_MISS", self.tw.near_miss as f32, mult, out);
            self.combo_count += 1;
        }
        // police escaping: 0x0016C920 never reports, so only its "else" side can run (original bug kept)
        self.police_escaping(i);
        if self.flags & flag::POLICE_NEAR != 0 {
            self.score("OSD_SCORE_POLICE_ESCAPED", self.tw.escaped_police as f32, mult, out);
            self.combo_count += 1;
        }
        self.flags &= !flag::POLICE_NEAR;
        self.flags &= !flag::ESCAPING;
        if self.flags & flag::START_SWITCH_DONE == 0 && i.start_switch != 0 {
            let (id, pts) = match i.start_switch {
                1 => ("OSD_SCORE_BAD_START", self.tw.start_switch[0]),
                2 => ("OSD_SCORE_GOOD_START", self.tw.start_switch[1]),
                3 => ("OSD_SCORE_PERFECT_START", self.tw.start_switch[2]),
                4 => ("OSD_SCORE_SKID_START", self.tw.start_switch[3]),
                _ => {
                    log::error!("assert: ScoreManager.cpp 859");
                    ("", 0.0)
                }
            };
            if !id.is_empty() {
                self.score(id, pts, mult, out);
            }
            self.combo_count += 1;
            self.flags |= flag::START_SWITCH_DONE;
        }
        if i.race_switch != 0 {
            let (id, pts) = match i.race_switch {
                1 => ("OSD_SCORE_BAD_SWITCH", self.tw.race_switch[0]),
                2 => ("OSD_SCORE_GOOD_SWITCH", self.tw.race_switch[1]),
                3 => ("OSD_SCORE_PERFECT_SWITCH", self.tw.race_switch[2]),
                _ => {
                    // the switch's default (an unknown value), ScoreManager.cpp 888
                    log::error!("assert: ScoreManager.cpp 888");
                    ("", 0.0)
                }
            };
            if !id.is_empty() {
                self.score(id, pts, mult, out);
            }
            self.combo_count += 1;
        }
        self.wheel_check_and_wrong_way(i);
        if self.last_lap >= i.lap {
            self.lap_positions(i);
        } else {
            if let Some(t) = self.fastest_lap(i, out) {
                let _ = t;
                self.score("OSD_SCORE_NEW_FASTEST_LAP", self.tw.fastest_lap as f32, mult, out);
                self.combo_time = 0.0;
                self.combo_count += 1;
            }
            if self.flags & flag::CRASHED_THIS_LAP == 0 {
                self.score("OSD_SCORE_CLEAN_LAP", self.tw.clean_lap as f32, mult, out);
                self.combo_time = 0.0;
                self.combo_count += 1;
            }
            if self.last_position == 1 && self.flags & flag::PLACE_CHANGED == 0 {
                self.score("OSD_SCORE_FULL_LAP_IN_LEAD", self.tw.lead_lap as f32, mult, out);
                self.combo_time = 0.0;
                self.combo_count += 1;
            }
            self.last_lap = i.lap;
            self.stats.lap_time = 0.0;
            if self.stats.lap_score > self.stats.best_lap_score {
                self.stats.best_lap_score = self.stats.lap_score;
            }
            self.stats.lap_score = 0;
            self.flags &= !(flag::PLACE_CHANGED | flag::CRASHED_THIS_LAP | 0x7C00);
        }
        if self.tw.infinite_zone {
            self.zone = 10.0;
        }
        self.last_time = i.time;
    }

    /// ScoreManager.cpp 0x0016B000 — falling fast, or the car not on its wheels: no scores
    fn check_upright(&mut self, i: &ScoreInputs) {
        let falling = i.velocity.y < -10.0;
        let upright = i.wheel_normal.dot(Vec3::Y) > f32::from_bits(0x3E99_999A);
        self.flags &= !flag::NOT_UPRIGHT;
        if falling || !upright {
            self.flags |= flag::NOT_UPRIGHT;
        }
    }

    /// ScoreManager.cpp 0x0016B180
    fn clean_start(&mut self) -> bool {
        if self.flags & flag::CLEAN_START_DONE != 0 || self.tw.clean_start_time > self.run_time {
            return false;
        }
        self.flags |= flag::CLEAN_START_DONE;
        self.clean_time >= self.tw.clean_start_time && self.speed > self.tw.clean_start_min_speed
    }

    /// ScoreManager.cpp 0x0016B200 — a place gained and held for 3 s; Some(clean)
    fn overtake(&mut self, i: &ScoreInputs) -> Option<bool> {
        let pos = i.position;
        if self.last_position == 0 {
            self.last_position = pos;
            self.flags &= !flag::GAINED_PLACE;
            return None;
        }
        if pos != self.last_position {
            self.flags |= flag::PLACE_CHANGED;
        }
        if pos < self.last_position {
            let mut r = None;
            if self.overtake_time > 1e-8 {
                r = Some(self.clean_time >= self.overtake_time + 0.5);
            }
            self.overtake_time = 0.0;
            self.last_position = pos;
            self.flags |= flag::GAINED_PLACE;
            return r;
        }
        if pos > self.last_position {
            self.overtake_time = 0.0;
            self.last_position = pos;
            self.flags &= !flag::GAINED_PLACE;
            return None;
        }
        if self.flags & flag::GAINED_PLACE != 0 && pos == self.last_position {
            self.overtake_time += self.dt;
            if self.overtake_time <= self.tw.overtake_time {
                return None;
            }
            self.last_position = pos;
            let clean = self.clean_time >= self.overtake_time + 0.5;
            self.stats.overtakes += 1;
            self.overtake_time = 0.0;
            self.flags &= !flag::GAINED_PLACE;
            return Some(clean);
        }
        self.last_position = pos;
        self.flags &= !flag::GAINED_PLACE;
        None
    }

    /// ScoreManager.cpp 0x0016B460 — passing 100 mph after dropping under 90
    fn top_speed(&mut self) -> bool {
        if (100.0f32 * 0.9) > self.speed {
            self.flags |= flag::BELOW_TOP_SPEED;
        }
        if self.flags & flag::NOT_UPRIGHT != 0 {
            return false;
        }
        if self.flags & flag::BELOW_TOP_SPEED == 0 || self.speed <= 100.0 {
            return false;
        }
        self.flags &= !flag::BELOW_TOP_SPEED;
        true
    }

    /// ScoreManager.cpp 0x0016B500 — sliding (velocity 10..170 degrees off the heading) above 30 mph, once a
    /// second; Some(handbrake turn)
    fn drift(&mut self, i: &ScoreInputs) -> Option<bool> {
        self.drift_time += self.dt;
        if self.flags & flag::NOT_UPRIGHT != 0 || self.drift_time < 1.0 || self.clean_time < 2.0 {
            return None;
        }
        let v = flat_norm(i.velocity);
        let f = flat_norm(i.forward);
        let d = v.dot(f);
        if self.speed <= 30.0 || d >= f32::from_bits(0x3F7C_28F6) || d <= -f32::from_bits(0x3F7C_28F6) {
            return None;
        }
        self.drift_time = 0.0;
        Some(self.handbrake_time < 1.0)
    }

    /// ScoreManager.cpp 0x0016B6B0 — the heading passed all four quarters within the spin time
    fn spin_360(&mut self, i: &ScoreInputs) -> bool {
        let f = Vec3::new(i.forward.x, 0.0, i.forward.z);
        let (a, b) = (0.05f32, 1.0 - 0.05f32);
        for s in &mut self.spin {
            *s += self.dt;
        }
        if self.flags & flag::NOT_UPRIGHT != 0 {
            return false;
        }
        let dx = Vec3::X.dot(f);
        let dz = Vec3::Z.dot(f);
        if a <= dx || dx <= 0.0 - a {
            if a > dz && dz > 0.0 - a {
                if (0.0 - b) <= dx {
                    if dx > b {
                        self.spin[3] = 0.0;
                    }
                } else {
                    self.spin[2] = 0.0;
                }
            }
        } else if (0.0 - b) <= dz {
            if dz > b {
                self.spin[1] = 0.0;
            }
        } else {
            self.spin[0] = 0.0;
        }
        let t = self.tw.spin_time;
        if self.spin.iter().any(|&s| t <= s) {
            return false;
        }
        self.spin = [10000.0; 4];
        t <= self.clean_time
    }

    /// ScoreManager.cpp 0x0016B9F0 — Some(entered)
    fn slipstream(&mut self, i: &ScoreInputs) -> Option<bool> {
        let s = i.slipstream;
        if s <= 0.0 {
            self.flags &= !flag::IN_SLIPSTREAM;
            return None;
        }
        if s <= 0.5 {
            return None;
        }
        if self.flags & flag::IN_SLIPSTREAM == 0 && self.out_of_slip_time > self.tw.slipstream_min_time {
            self.flags |= flag::IN_SLIPSTREAM;
            self.out_of_slip_time = 0.0;
            self.slip_time = 0.0;
            return Some(true);
        }
        if self.slip_time <= self.tw.slipstream_interval {
            return None;
        }
        self.slip_time = 0.0;
        Some(false)
    }

    /// ScoreManager.cpp 0x0016C160 — Some(started)
    fn racing_line(&mut self, i: &ScoreInputs) -> Option<bool> {
        if self.tw.racing_line_min_speed > self.speed || i.raceline_offset.abs() > self.tw.racing_line_max_dist {
            self.flags &= !(flag::ON_RACING_LINE | flag::RACING_LINE_SCORED);
            return None;
        }
        if self.flags & flag::ON_RACING_LINE == 0 {
            self.flags |= flag::ON_RACING_LINE;
            self.line_time = 0.0;
            self.line_scored_time = 0.0;
            return None;
        }
        if self.flags & flag::RACING_LINE_SCORED != 0 {
            if self.line_scored_time > self.tw.racing_line_interval {
                self.line_scored_time = 0.0;
                return Some(false);
            }
        } else if self.line_time > self.tw.racing_line_min_time {
            self.flags |= flag::RACING_LINE_SCORED;
            self.line_scored_time = 0.0;
            self.line_time = 0.0;
            return Some(true);
        }
        None
    }

    /// ScoreManager.cpp 0x0016C7A0 — passing a traffic car within 5.5 m above 60 mph (then 0.5 s quiet)
    fn near_miss(&mut self, i: &ScoreInputs) -> bool {
        if self.near_miss_car.is_some() {
            self.near_miss_time += self.dt;
            if self.near_miss_time > 0.5 {
                self.near_miss_car = None;
            }
            return false;
        }
        if self.tw.near_miss_min_speed > self.speed {
            return false;
        }
        for &(id, p) in &i.traffic {
            if (self.tw.near_miss_distance as f64) > (p - i.position_vec).length() as f64 {
                self.near_miss_car = Some(id);
                self.near_miss_time = 0.0;
                return true;
            }
        }
        false
    }

    /// ScoreManager.cpp 0x0016C920 — its result flag is never set (original bug kept): always None; it only
    /// clears the escape timer
    fn police_escaping(&mut self, i: &ScoreInputs) {
        let (mut close, mut mid) = (false, false);
        if i.chased_by_police {
            for p in &i.police {
                let d = (*p - i.position_vec).length();
                if self.tw.escape_min_dist > d {
                    close = true;
                } else if self.tw.escape_max_dist > d {
                    mid = true;
                }
            }
        }
        if close || !mid {
            self.escape_time = 0.0;
        }
    }

    /// ScoreManager.cpp 0x0016C2E0 (wheel heights, not kept) and 0x0016BC70 — heading against the raceline
    fn wheel_check_and_wrong_way(&mut self, i: &ScoreInputs) {
        let Some(dir) = i.raceline_dir else { return };
        let d = flat_norm(dir).dot(flat_norm(i.forward));
        if self.tw.wrong_way_dot <= d {
            self.flags &= !flag::WRONG_WAY;
        } else {
            self.flags |= flag::WRONG_WAY;
        }
    }

    /// ScoreManager.cpp 0x0016BB00 — Some(time) on a new session best lap
    fn fastest_lap(&mut self, i: &ScoreInputs, out: &mut Vec<ScoreEvent>) -> Option<f32> {
        let t = i.last_lap_time;
        if self.stats.fastest_lap < 0.001 || self.stats.fastest_lap > t {
            self.stats.fastest_lap = t;
        }
        if i.best_lap_time <= t {
            return None;
        }
        out.push(ScoreEvent::BestLap(t));
        Some(t)
    }

    /// ScoreManager.cpp 0x0016BE50 — on passing a new split of the lap: the gap to the car ahead (time) and
    /// to the car behind (distance over its average speed), shown for 2 s (`dword_6860EC`)
    fn lap_positions(&mut self, i: &ScoreInputs) {
        if !i.on_section || !i.racing {
            return;
        }
        let mut passed = false;
        if (1..=5).contains(&i.split) {
            let b = flag::LAP_POS[i.split as usize - 1];
            if self.flags & b == 0 {
                self.flags |= b;
                passed = true;
            }
        }
        if passed {
            self.lap_pos_time = 0.0;
        }
        if !i.is_player || !passed {
            return;
        }
        if let Some((id, t)) = i.ahead {
            self.stats.gap_ahead = i.time - t;
            self.stats.car_ahead = Some(id);
            self.ahead_shown = self.tw.split_time_duration;
        }
        if let Some((id, d)) = i.behind {
            let gap = i.distance - d;
            let speed = d / i.time;
            if speed > 0.0 && i.time > gap / speed {
                self.stats.gap_behind = gap / speed;
                self.stats.car_behind = Some(id);
                self.behind_shown = self.tw.split_time_duration;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overtake_and_combo() {
        let mut s = ScoreManager::new(true);
        let mut out = Vec::new();
        let mut i = ScoreInputs { racing: true, position: 3, lap: 1, wheel_normal: Vec3::Y, forward: Vec3::Z, ..Default::default() };
        let mut t = 1.0;
        for _ in 0..10 {
            i.time = t;
            s.update(&i, false, &mut out);
            t += 0.1;
        }
        i.position = 2;
        for _ in 0..40 {
            i.time = t;
            s.update(&i, false, &mut out);
            t += 0.1;
        }
        assert_eq!(s.stats.overtakes, 1);
        assert!(out.contains(&ScoreEvent::Message(crc("OSD_SCORE_CLEAN_OVERTAKE"))));
        for _ in 0..40 {
            i.time = t;
            s.update(&i, false, &mut out);
            t += 0.1;
        }
        assert!(s.stats.score > 0, "combo banked");
    }
}
