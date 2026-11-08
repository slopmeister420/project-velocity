//! An ai (or player-tracked) racing car: route following, lookahead, steering, throttle/brake,
//! avoidance and overtaking, nitrous, slipstream, mistakes, failures and race progress
//! (src/Ai/VsAIRaceVehicle.cpp, VsAIRaceVehicle.h).
//!
//! The original keeps raw pointers to other cars; the port reads them through an
//! [`AiScene`] snapshot (refreshed after every car update, so ordering effects match) and writes
//! them through indices owned by `VsRaceManager`.
//!
//! Not ported (render/debug only): the debug overlays drawn when `DrawVehicleDebug`/
//! `DrawAvoidanceDebug` are on, the danger-zone particles of a trailblazer respawn (0x00123070), the danger-zone
//! explosion effect of the NOS failure, replay-camera notifications of `Update`, the score manager's OSD
//! gap items (0x00169940).

use super::ai_context::{AiContext, AiEventInfo, AiFlashRecord, AiHost};
use super::vs_ai_driver::{DriverFrame, DriverRef, VsAIDriver};
use super::vs_ai_globals::VsAIGlobals;
use super::vs_ai_player_tft_manager::{TftPlayer, VsAIPlayerTFTManager, tft};
use super::vs_ai_road_line::VsAIRoadLine;
use super::vs_ai_road_section::VsAIRaceSection;
use super::vs_ai_startline_revs::VsAIStartlineRevs;
use super::vs_ai_tools::VsAITools;
use super::vs_ai_vehicle::{AiPhysicsVehicle, VehicleControls, VsAIVehicle};
use super::vs_ai_world::{VsAIHandling, VsAIWorld};
use super::vs_math::*;
use velocity_game::game_mode::trail_blazer::{TrailBlazer, TrailCar, TrailEvent, TrailSpline};
use velocity_game::score_manager::score_manager::{ScoreInputs, ScoreManager};

/// file statics of VsAIRaceVehicle.cpp at 0x006D2CAC (VsAIRaceVehicle.cpp 0x000250A0).
pub mod params {
    pub const FOLLOW_DISTANCE: f32 = f32::from_bits(0x4100_0000); // 0x006D2CAC 8.0
    pub const UNK_2CB0: f32 = f32::from_bits(0x4040_0000); // 0x006D2CB0 3.0
    pub const FOLLOW_LATERAL: f32 = f32::from_bits(0x4020_0000); // 0x006D2CB4 2.5
    pub const OVERTAKE_LATERAL: f32 = f32::from_bits(0x40A0_0000); // 0x006D2CB8 5.0
    pub const OVERTAKE_LOOKAHEAD: f32 = f32::from_bits(0x41A0_0000); // 0x006D2CBC 20.0
    pub const UNK_2CC0: f32 = f32::from_bits(0x4000_0000); // 0x006D2CC0 2.0
    pub const SPEED_MATCH_GAIN: f32 = f32::from_bits(0x4120_0000); // 0x006D2CC4 10.0
    /// VsAIDriver.cpp-side globals at 0x00684E58.. (initialised data)
    pub const SLOWDOWN_MIN_DIST: f32 = f32::from_bits(0x4296_0000); // 0x00684E58 75
    pub const SLOWDOWN_MAX_DIST: f32 = f32::from_bits(0x4348_0000); // 0x00684E5C 200
    pub const SLOWDOWN_MAX_ACCEL: f32 = f32::from_bits(0x3F00_0000); // 0x00684E60 0.5
    pub const SUPERCAR_GRIP_SCALE: f32 = f32::from_bits(0x3CA3_D70A); // 0x00684E64 0.02
    pub const EDGE_MARGIN: f32 = f32::from_bits(0x4000_0000); // 0x00684E70 2.0
    pub const SPIN_IMPULSE: f32 = f32::from_bits(0x469C_4000); // 0x00684E74 20000 (sign flips per use)
    pub const WHEEL_DROP: f32 = f32::from_bits(0x3DCC_CCCD); // 0x00684E78 0.1
    pub const BURNOUT_SPEED: f32 = f32::from_bits(0x41F0_0000); // 0x00684E7C 30
    /// prestige needed per shortcut level (0x00624ED0)
    pub const SHORTCUT_PRESTIGE: [u32; 4] = [0, 1000, 5000, 15000];
    /// highest shortcut level by [skill][aggression] (0x00624EE0)
    pub const SHORTCUT_LEVEL: [[u32; 4]; 4] = [[1, 1, 2, 2], [1, 2, 3, 3], [2, 3, 4, 4], [3, 4, 4, 4]];
}

/// eRS states (`m_eRaceState` +0x254); enter/update/exit per state at +0x168/+0x188/+0x1A8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum RaceState {
    Grid = 0,
    Launch = 1,
    PlayerAssist = 2,
    #[default]
    Racing = 3,
    Chase = 4,
    Recover = 5,
    Idle6 = 6,
    Finished = 7,
}

impl RaceState {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0 => Self::Grid,
            1 => Self::Launch,
            2 => Self::PlayerAssist,
            3 => Self::Racing,
            4 => Self::Chase,
            5 => Self::Recover,
            6 => Self::Idle6,
            _ => Self::Finished,
        }
    }
}

/// eFT failures (`m_eFailure` +0x260); 6 = none.
pub const FAILURE_NONE: u32 = 6;

/// another car as the ai sees it this frame.
#[derive(Debug, Clone, Default)]
pub struct AiVehicleView {
    pub r: VehicleRef,
    pub active: bool,
    pub is_traffic: bool,          // +0x07C
    pub is_player: bool,           // +0x34C
    pub is_police: bool,           // +0x230 bit 1
    pub physics: Option<usize>,
    pub id: u32,                   // +0x078
    pub position: VsVector4,       // +0x098
    pub velocity: VsVector4,       // +0x080
    pub forward: VsVector4,        // +0x0C8
    pub speed: f32,                // +0x090
    pub race_section: Option<usize>, // +0x258
    pub dist_along: f32,           // +0x2A8
    pub lateral: f32,              // +0x2AC
    pub race_distance: f32,        // +0x2F0
    pub traffic_dist: f32,         // +0x0EC traffic: distance along the asking car's raceline
    pub started: bool,             // +0x0F4
    pub race_position: u8,         // stats +0x59
    pub driver: Option<DriverRef>,
    pub lap: u32,                  // +0x2FC
    /// race statistics (+0x1CC); zero for traffic
    pub stats: RaceStats,
}

/// which array a car lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VehicleRef {
    #[default]
    None,
    Ai(usize),
    Player(usize),
    Traffic(usize),
    /// a police car (its race part in race mode, its traffic part otherwise)
    Police(usize),
}

/// all cars, for lookups by the car being updated.
#[derive(Debug, Clone, Default)]
pub struct AiScene {
    pub ai: Vec<AiVehicleView>,
    pub players: Vec<AiVehicleView>,
    pub traffic: Vec<AiVehicleView>,
    /// police cars as (active race-or-traffic part)
    pub police: Vec<AiVehicleView>,
}

impl AiScene {
    pub fn get(&self, r: VehicleRef) -> Option<&AiVehicleView> {
        match r {
            VehicleRef::None => None,
            VehicleRef::Ai(i) => self.ai.get(i),
            VehicleRef::Player(i) => self.players.get(i),
            VehicleRef::Traffic(i) => self.traffic.get(i),
            VehicleRef::Police(i) => self.police.get(i),
        }
    }
    /// VsRaceManager.h 0x00015DB0 with index 0
    pub fn player0(&self) -> Option<&AiVehicleView> {
        self.players.first()
    }
}

/// one lookahead point (20 bytes at +0x268).
#[derive(Debug, Clone, Copy, Default)]
pub struct LookAheadPoint {
    pub pos: VsVector4,
    pub dist: f32,
}

/// one avoidance record (40 bytes in the +0x24C array).
#[derive(Debug, Clone, Copy, Default)]
pub struct AvoidInfo {
    pub pos: VsVector4,      // +0x00 point beside the other car
    pub vehicle: VehicleRef, // +0x10
    pub is_traffic: bool,    // +0x14
    pub close: bool,         // +0x18 laterally within the avoid radius
    pub offset: f32,         // +0x1C lateral offset of the avoid point
    pub dist: f32,           // +0x20 distance along the raceline of the avoid point
    pub time: f32,           // +0x24 time to reach the other car
}

/// the race statistics block (+0x1CC, 25 dwords cleared on reset; `VsAIRaceVehicle.cpp 0x000244E0`).
/// [0] top speed, [1] finish time, [2] lap start, [3 + n] checkpoint n time, [11] best lap,
/// [19] [20] lap records, [21] prestige earned, byte 1 of [22] race position.
pub type RaceStats = [u32; 25];

pub fn stat_f(s: &RaceStats, i: usize) -> f32 {
    f32::from_bits(s[i])
}
pub fn set_stat_f(s: &mut RaceStats, i: usize, v: f32) {
    s[i] = v.to_bits();
}
/// `stats +0x59`
pub fn stat_position(s: &RaceStats) -> u8 {
    (s[22] >> 8) as u8
}

#[derive(Debug, Clone)]
pub struct VsAIRaceVehicle {
    pub base: VsAIVehicle,
    pub corner_tightness: f32,       // +0x008
    pub id: u32,                     // +0x078 player/opponent index
    pub is_traffic: bool,            // +0x07C
    pub speed_match: bool,           // +0x00C (base flag)
    pub target_speed: f32,           // +0x010
    pub traffic_dist: f32,           // +0x0EC
    pub lane_ptr: u32,               // +0x0F0
    pub started: bool,               // +0x0F4
    pub finished: bool,              // +0x0F5
    pub line_state: u32,             // +0x0F8 start line crossing state 0/1/2
    pub flag_0fc: bool,              // +0x0FC
    pub finished_flag: bool,         // +0x0FD
    pub draft_on_player: bool,       // +0x0FE
    pub startline_revs: VsAIStartlineRevs, // +0x100
    pub score: Option<ScoreManager>, // +0x1C8 m_pScoreManager
    pub stats: RaceStats,            // +0x1CC
    pub driver: Option<DriverRef>,   // +0x234
    pub vehicle_ahead: VehicleRef,   // +0x238 next car up the race order
    pub vehicle_behind: VehicleRef,  // +0x23C
    pub chase_target: VehicleRef,    // +0x240 (police)
    pub avoid: Vec<VehicleRef>,      // +0x244 / +0x248, at most 20
    pub avoid_info: Vec<AvoidInfo>,  // +0x24C, 22 slots
    pub flags_230: u32,              // +0x230 bit0 set at init, bit1 police
    pub prev_state: RaceState,       // +0x250
    pub state: RaceState,            // +0x254
    pub race_section: Option<usize>, // +0x258
    pub checkpoint: u32,             // +0x25C
    pub failure: u32,                // +0x260 eFT
    pub failure_time: f32,           // +0x264
    pub look_ahead: [LookAheadPoint; 3], // +0x268
    pub steer_angle: f32,            // +0x2A4
    pub dist_along: f32,             // +0x2A8 along the current raceline
    pub lateral: f32,                // +0x2AC signed offset from the raceline, incl. +0x39C
    pub raceline_seg: u32,           // +0x2B0
    pub unk_2b4: u32,                // +0x2B4
    pub dist_left: f32,              // +0x2B8 along the left edge
    pub dist_right: f32,             // +0x2BC along the right edge
    pub edge_left_sq: f32,           // +0x2C0 squared xz distance to the left edge
    pub edge_right_sq: f32,          // +0x2C4
    pub road_width_sq: f32,          // +0x2C8
    pub left_seg: u32,               // +0x2CC
    pub right_seg: u32,              // +0x2D0
    pub next_sub_point: Option<usize>, // +0x2D4
    pub offset_active: bool,         // +0x2D8
    pub offset_rate: f32,            // +0x2E0
    pub offset_current: f32,         // +0x2E4
    pub offset_target: f32,          // +0x2E8
    pub near_edge: bool,             // +0x2EC
    pub race_distance: f32,          // +0x2F0
    pub prev_race_distance: f32,     // +0x2F4
    pub track_distance: f32,         // +0x2F8
    pub lap: u32,                    // +0x2FC
    pub on_route: bool,              // +0x300
    pub unk_304: f32,                // +0x304 -1
    pub left_of_left_edge: bool,     // +0x308
    pub right_of_right_edge: bool,   // +0x309
    pub flag_30a: bool,              // +0x30A
    pub stuck: bool,                 // +0x30B
    pub stuck_time: f32,             // +0x30C
    pub stuck_pos: VsVector4,        // +0x310
    pub grace_until: f32,            // +0x320 -1
    pub upside_down: bool,           // +0x324
    pub upside_down_time: f32,       // +0x328
    pub start_pos: VsVector4,        // +0x32C grid slot
    pub start_orientation: VsVector4, // +0x33C
    pub is_player: bool,             // +0x34C
    pub human_control: bool,         // +0x34D the player drives; the ai only tracks progress
    pub nos_desire: f32,             // +0x350 0..1
    pub nos_start: f32,              // +0x354 -1 when off
    pub nos_stop: f32,               // +0x358 -FLT_MAX
    pub slipstream_target: VehicleRef, // +0x35C
    pub overtake_target: VehicleRef, // +0x360
    pub rival_state: u32,            // +0x364 0 none, 1 weaving
    pub rival_next: f32,             // +0x368
    pub rival_offset: f32,           // +0x36C
    pub slip_lock_until: f32,        // +0x370 -1
    pub overtake_side: u32,          // +0x374
    pub police_mode: u32,            // +0x378 2 at reset
    pub avoiding: bool,              // +0x380
    pub aggression_scale: f32,       // +0x384
    pub saved_grip: [f32; 3],        // +0x388 +0x38C +0x390 handling values restored on exit
    pub launch_time: f32,            // +0x394
    pub launch_first: bool,          // +0x398
    pub lane_offset: f32,            // +0x39C
    pub respawn_trigger_pending: f32, // +0x3B8 time the respawn crash ends
    pub respawn_trigger: u32,        // +0x3BC
    pub respawn_time: f32,           // +0x3C0
    pub oil_spill_time: f32,         // +0x3C4
    /// +0x3C8 players only (0x00023250)
    pub trail_blazer: Option<TrailBlazer>,
    /// port: the trailblazer's events since the plugin last sent them as `TrailBlazerMessage`s
    pub trail_events: Vec<TrailEvent>,
    pub scheduled_failure_time: f32, // +0x3CC
    pub scheduled_failure: u32,      // +0x3D0 eFT
    pub post_race_state: u32,        // +0x3D4 ePR
}

impl Default for VsAIRaceVehicle {
    /// VsAIPoliceVehicle.cpp 0x00017E40 (the VsAIRaceVehicle constructor sits in that object)
    fn default() -> Self {
        Self {
            base: VsAIVehicle::default(),
            corner_tightness: 0.0,
            id: 0,
            is_traffic: false,
            speed_match: false,
            target_speed: 0.0,
            traffic_dist: 0.0,
            lane_ptr: 0,
            started: false,
            finished: false,
            line_state: 0,
            flag_0fc: false,
            finished_flag: false,
            draft_on_player: false,
            startline_revs: VsAIStartlineRevs::default(),
            stats: [0; 25],
            driver: None,
            vehicle_behind: VehicleRef::None,
            vehicle_ahead: VehicleRef::None,
            chase_target: VehicleRef::None,
            avoid: Vec::new(),
            avoid_info: vec![AvoidInfo::default(); 22],
            flags_230: 0,
            prev_state: RaceState::Grid,
            state: RaceState::Racing,
            race_section: None,
            checkpoint: 0,
            failure: FAILURE_NONE,
            failure_time: 0.0,
            look_ahead: [LookAheadPoint::default(); 3],
            steer_angle: 0.0,
            dist_along: 0.0,
            lateral: 0.0,
            raceline_seg: 0,
            unk_2b4: 0,
            dist_left: 0.0,
            dist_right: 0.0,
            edge_left_sq: 0.0,
            edge_right_sq: 0.0,
            road_width_sq: 0.0,
            left_seg: 0,
            right_seg: 0,
            next_sub_point: None,
            offset_active: false,
            offset_rate: 0.0,
            offset_current: 0.0,
            offset_target: 0.0,
            near_edge: false,
            race_distance: 0.0,
            prev_race_distance: 0.0,
            track_distance: 0.0,
            lap: 0,
            on_route: true,
            unk_304: -1.0,
            left_of_left_edge: true,
            right_of_right_edge: true,
            flag_30a: true,
            stuck: false,
            stuck_time: 0.0,
            stuck_pos: VsVector4::default(),
            grace_until: -1.0,
            upside_down: false,
            upside_down_time: 0.0,
            start_pos: VsVector4::default(),
            start_orientation: VsVector4::default(),
            is_player: false,
            human_control: false,
            nos_desire: 0.0,
            nos_start: -1.0,
            nos_stop: f32::from_bits(0xFF7F_FFFF),
            slipstream_target: VehicleRef::None,
            overtake_target: VehicleRef::None,
            rival_state: 0,
            rival_next: 0.0,
            rival_offset: 0.0,
            slip_lock_until: -1.0,
            overtake_side: 0,
            police_mode: 2,
            avoiding: false,
            aggression_scale: 1.0,
            saved_grip: [0.0; 3],
            launch_time: 0.0,
            launch_first: true,
            lane_offset: 0.0,
            respawn_trigger_pending: 0.0,
            respawn_trigger: 0,
            respawn_time: 0.0,
            oil_spill_time: -1.0,
            trail_blazer: None,
            trail_events: Vec::new(),
            scheduled_failure_time: 0.0,
            scheduled_failure: FAILURE_NONE,
            post_race_state: 0,
            score: None,
        }
    }
}

/// everything a race vehicle update reads besides itself.
pub struct RaceFrame<'a, 'c> {
    pub ctx: &'a mut AiContext<'c>,
    pub world: &'a mut VsAIWorld,
    pub scene: &'a AiScene,
    pub tft: &'a mut VsAIPlayerTFTManager,
    pub tft_out: &'a mut dyn TftPlayer,
    pub speech_enabled: bool,
    pub num_players: u32,
    pub time: f32,
}

impl RaceFrame<'_, '_> {
    pub fn driver_frame(&mut self) -> DriverFrame<'_> {
        DriverFrame {
            time: self.time,
            race_clock: self.ctx.game.race_clock,
            speech_enabled: self.speech_enabled,
            num_players: self.num_players,
            globals: &self.world.globals,
            rand: self.ctx.rand,
            tft: self.tft,
            tft_out: self.tft_out,
        }
    }
    fn lap_length(&self) -> f32 {
        self.world.race_sections()[0].raceline.get_length()
    }
}

impl VsAIRaceVehicle {
    fn phys<'p>(&self, f: &'p mut RaceFrame) -> &'p mut dyn AiPhysicsVehicle {
        f.ctx.physics.vehicle(self.base.physics.expect("m_pPhysicsVehicle"))
    }
    fn phys_ref<'p>(&self, f: &'p RaceFrame) -> &'p dyn AiPhysicsVehicle {
        f.ctx.physics.vehicle_ref(self.base.physics.expect("m_pPhysicsVehicle"))
    }
    fn controls<'p>(&self, f: &'p mut RaceFrame) -> &'p mut VehicleControls {
        self.phys(f).controls()
    }
    fn handling<'w>(&self, w: &'w VsAIWorld) -> &'w VsAIHandling {
        &w.handlings[self.base.handling.expect("m_pHandling")]
    }
    fn rc(&self, w: &VsAIWorld, idx: usize, x: f32) -> f32 {
        w.response_curves[idx].get_value(x)
    }
    fn section<'w>(&self, w: &'w VsAIWorld) -> &'w VsAIRaceSection {
        &w.race_sections()[self.race_section.expect("m_pCurrentRaceSection")]
    }
    fn raceline<'w>(&self, w: &'w VsAIWorld) -> &'w VsAIRoadLine {
        &self.section(w).raceline
    }
    /// point on the current raceline; the road line caches the interpolated point, so it is `&mut`.
    fn raceline_point(&self, w: &mut VsAIWorld, dist: f32, seg: u32) -> VsVector4 {
        let si = self.race_section.expect("m_pCurrentRaceSection");
        let mut idx = 0;
        w.race_sections_mut()[si].raceline.get_point_at_distance(dist, seg, &mut idx)
    }
    /// VsAIRaceVehicle.h 0x00015CA0
    pub fn get_dist_along(&self) -> f32 {
        debug_assert!(self.race_section.is_some(), "m_pCurrentRaceSection");
        self.dist_along
    }
    /// VsAIRaceVehicle.h 0x00023DB0
    pub fn get_race_distance(&self) -> f32 {
        self.race_distance
    }
    /// VsAIRaceVehicle.h 0x00018A10
    pub fn is_police(&self) -> bool {
        (self.flags_230 & 2) != 0
    }
    /// VsAIRaceVehicle.h 0x00023C20
    pub fn get_look_ahead(&self, i: u32) -> &LookAheadPoint {
        debug_assert!(i < 3, "dIdx < NUM_LOOKAHEAD_POINTS");
        &self.look_ahead[i as usize]
    }

    /// snapshot others read from this car.
    pub fn view(&self, r: VehicleRef) -> AiVehicleView {
        AiVehicleView {
            r,
            active: self.base.active,
            is_traffic: self.is_traffic,
            is_player: self.is_player,
            is_police: self.is_police(),
            physics: self.base.physics,
            id: self.id,
            position: self.base.position,
            velocity: self.base.velocity,
            forward: self.base.forward,
            speed: self.base.speed,
            race_section: self.race_section,
            dist_along: self.dist_along,
            lateral: self.lateral,
            race_distance: self.race_distance,
            traffic_dist: self.traffic_dist,
            started: self.started,
            race_position: stat_position(&self.stats),
            driver: self.driver,
            lap: self.lap,
            stats: self.stats,
        }
    }

    /// VsAIPoliceVehicle.cpp 0x00018000 (`local`: the score manager feeds the local player's OSD)
    pub fn init(&mut self, id: u32, g: &VsAIGlobals, event: &AiEventInfo, rand: &mut VsRand, prestige: u32, local: bool) {
        self.id = id;
        self.driver = None;
        self.base.physics = None;
        self.is_player = false;
        self.human_control = false;
        self.flags_230 = 0;
        self.flags_230 |= 1;
        self.avoid = Vec::with_capacity(22);
        self.avoid_info = vec![AvoidInfo::default(); 22];
        self.failure = FAILURE_NONE;
        self.failure_time = 0.0;
        // 0x00017F60 (0x00167F30 sets up the local OSD items, not ported)
        self.score = Some(ScoreManager::new(local));
        self.startline_revs.init();
        self.base.deactivate(None);
        self.reset(g, event, rand, prestige);
    }

    /// VsAIPoliceVehicle.cpp 0x00018190
    pub fn reset(&mut self, g: &VsAIGlobals, event: &AiEventInfo, rand: &mut VsRand, prestige: u32) {
        self.base.reset_base();
        self.race_distance = 0.0;
        self.prev_race_distance = 0.0;
        self.track_distance = 0.0;
        self.edge_left_sq = 0.0;
        self.edge_right_sq = 0.0;
        self.road_width_sq = 0.0;
        self.left_seg = 0;
        self.right_seg = 0;
        self.raceline_seg = 0;
        self.dist_along = 0.0;
        self.unk_2b4 = 0;
        self.near_edge = false;
        self.base.prev_speed = 0.0;
        self.base.speed = 0.0;
        self.on_route = true;
        self.unk_304 = -1.0;
        self.left_of_left_edge = true;
        self.right_of_right_edge = true;
        self.speed_match = false;
        self.base.flag_004 = false;
        self.base.debug_selected = false;
        self.flag_0fc = false;
        self.line_state = 0;
        self.started = false;
        self.finished = false;
        self.finished_flag = false;
        self.flag_30a = true;
        self.offset_active = false;
        self.offset_target = 0.0;
        self.stuck = false;
        self.upside_down = false;
        self.grace_until = -1.0;
        self.avoid.clear();
        self.lap = 0;
        self.draft_on_player = false;
        if self.is_player && !g.player_ai_control {
            self.human_control = true;
        }
        self.clear_failure_state(true);
        self.stats = [0; 25];
        set_stat_f(&mut self.stats, 11, event.stat_11_init);
        set_stat_f(&mut self.stats, 20, event.stat_20_init);
        self.nos_desire = 0.0;
        self.nos_start = -1.0;
        self.nos_stop = f32::from_bits(0xFF7F_FFFF);
        self.slipstream_target = VehicleRef::None;
        self.overtake_target = VehicleRef::None;
        self.slip_lock_until = -1.0;
        self.rival_state = 0;
        self.rival_next = 0.0;
        self.rival_offset = 0.0;
        self.race_section = None;
        self.next_sub_point = None;
        self.checkpoint = 0;
        self.vehicle_ahead = VehicleRef::None;
        self.vehicle_behind = VehicleRef::None;
        self.chase_target = VehicleRef::None;
        self.overtake_side = 0;
        self.police_mode = 2;
        self.saved_grip = [0.0; 3];
        self.launch_time = 0.0;
        self.launch_first = true;
        self.lane_offset = 0.0;
        self.setup_max_damage(g, prestige);
        self.setup_trail_blazer();
        self.respawn_trigger_pending = 0.0;
        self.respawn_trigger = 0;
        self.respawn_time = 0.0;
        self.state = RaceState::Racing;
        self.startline_revs.reset(g, rand);
        self.scheduled_failure_time = 0.0;
        self.scheduled_failure = FAILURE_NONE;
        self.post_race_state = 0;
        if let Some(s) = self.score.as_mut() {
            s.reset();
        }
        self.stats_to_score();
    }

    /// the score manager writes this car's record block (0x000244E0 = +0x1CC) in place; the port keeps two copies
    /// (`stats`, `ScoreManager.stats`) and makes them one around every score manager call. Index = offset / 4:
    /// +0x0C lap time [3], +0x24 gap to the car ahead [9], +0x28 gap of the car behind [10], +0x2C fastest lap [11],
    /// +0x44 lead time [17], +0x48 overtakes [18], +0x4C lap score [19], +0x50 best lap score [20], +0x54 score
    /// [21], +0x5C / +0x60 car ahead / behind [23]/[24] (race vehicle pointers; the port stores id + 1, 0 = none).
    pub fn stats_to_score(&mut self) {
        let st = &self.stats;
        let Some(s) = self.score.as_mut() else { return };
        let car = |v: u32| v.checked_sub(1);
        s.stats.lap_time = stat_f(st, 3);
        s.stats.gap_ahead = stat_f(st, 9);
        s.stats.gap_behind = stat_f(st, 10);
        s.stats.fastest_lap = stat_f(st, 11);
        s.stats.lead_time = stat_f(st, 17);
        s.stats.overtakes = st[18];
        s.stats.lap_score = st[19];
        s.stats.best_lap_score = st[20];
        s.stats.score = st[21];
        s.stats.car_ahead = car(st[23]);
        s.stats.car_behind = car(st[24]);
    }

    /// the score manager's writes back into the record block (see `stats_to_score`).
    pub fn stats_from_score(&mut self) {
        let Some(s) = self.score.as_ref() else { return };
        let st = &mut self.stats;
        set_stat_f(st, 3, s.stats.lap_time);
        set_stat_f(st, 9, s.stats.gap_ahead);
        set_stat_f(st, 10, s.stats.gap_behind);
        set_stat_f(st, 11, s.stats.fastest_lap);
        set_stat_f(st, 17, s.stats.lead_time);
        st[18] = s.stats.overtakes;
        st[19] = s.stats.lap_score;
        st[20] = s.stats.best_lap_score;
        st[21] = s.stats.score;
        st[23] = s.stats.car_ahead.map_or(0, |id| id + 1);
        st[24] = s.stats.car_behind.map_or(0, |id| id + 1);
    }

    /// the parts of `ClearFailure` that need no host.
    fn clear_failure_state(&mut self, reset_damage: bool) {
        if self.failure == 0 {
            self.oil_spill_time = -1.0;
        }
        self.failure = FAILURE_NONE;
        if reset_damage {
            self.base.damage = 0.0;
        }
    }

    /// VsAIRaceVehicle.cpp 0x00019D40
    pub fn clear_failure(&mut self, reset_damage: bool, host: &mut dyn AiHost) {
        if self.failure == 0 {
            host.engine_smoke(self.id, false);
        }
        self.clear_failure_state(reset_damage);
    }

    /// VsAIRaceVehicle.cpp 0x00023150
    pub fn setup_max_damage(&mut self, g: &VsAIGlobals, prestige: u32) {
        let lo = if self.is_player { g.min_player_car_health } else { g.min_ai_car_health };
        let hi = if self.is_player { g.max_player_car_health } else { g.max_ai_car_health };
        let r = vs_clamp((prestige as f64 / g.max_health_at_prestige as f64) as f32, 0.0, 1.0);
        self.base.set_max_damage((1.0 - r) * lo + hi * r);
    }

    /// VsAIRaceVehicle.cpp 0x00023110
    pub fn set_player(&mut self, b: bool, g: &VsAIGlobals, prestige: u32) {
        self.is_player = b;
        self.setup_max_damage(g, prestige);
        self.setup_trail_blazer();
    }

    /// VsAIRaceVehicle.cpp 0x00023250
    pub fn setup_trail_blazer(&mut self) {
        // the old one is deleted (0x000235E0, empty destructor 0x000C4850)
        self.trail_blazer = None;
        if self.is_player {
            self.trail_blazer = Some(TrailBlazer::new(self.id));
        }
    }

    /// the car as TrailBlazer.cpp reads it: VsAIDriver 0x00016690 (+0x98), 0x00015C60 (+0x90), 0x000245B0 (+0xC8)
    fn trail_car(&self) -> TrailCar {
        let b = &self.base;
        TrailCar {
            position: bevy::math::Vec3::new(b.position.x, b.position.y, b.position.z),
            speed: b.speed,
            heading: bevy::math::Vec3::new(b.forward.x, b.forward.y, b.forward.z),
        }
    }

    /// the trailblazer branch of TriggerUpdate.cpp 0x00180310: the trigger's spline (lane id `spline`) is looked up
    /// in the trailblazer sections (VsAIWorld 0x00034F70); its first lane's line is followed (TrailBlazer 0x000C4870).
    /// `saved` = the physics globals TrailBlazer.cpp saves. Returns false if the spline does not exist.
    #[allow(clippy::too_many_arguments)]
    pub fn start_trail_blazer(
        &mut self,
        world: &mut VsAIWorld,
        spline: u32,
        max_distance: f32,
        success_timeline: u32,
        respawn_trigger: u32,
        flag: bool,
        saved: [f32; 6],
    ) -> bool {
        let Some(line) = world.find_trail_blazer_line_mut(spline) else {
            log::warn!("Trail blazer spline {spline:x} not found");
            return false;
        };
        let car = self.trail_car();
        if let Some(tb) = self.trail_blazer.as_mut() {
            let s = TrailLine(std::cell::RefCell::new(line));
            tb.start(spline, &s, max_distance, success_timeline, respawn_trigger, flag, &car, saved, &mut self.trail_events);
        }
        true
    }

    /// `TrailBlazer::Update` (0x000C4FF0) from `Update` (0x00018A40); a failure respawns the car through
    /// 0x00022C90 with the respawn trigger's param 2 / 1000 as the delay (TrailBlazer.cpp 0x000C4C90)
    fn update_trail_blazer(&mut self, f: &mut RaceFrame, drv: Option<&mut VsAIDriver>, respawns: &mut dyn RespawnSource, dt: f32) {
        let Some(mut tb) = self.trail_blazer.take() else { return };
        let delay = match tb.respawn_trigger {
            0 => 0.0,
            t => match respawns.respawn_point(t) {
                Some(r) => (r.param_2 as u64 as f64 / 1000.0) as f32,
                None => {
                    log::error!("assert: respawnTrigger");
                    0.0
                }
            },
        };
        let car = self.trail_car();
        let at = self.trail_events.len();
        {
            let line = tb.spline.and_then(|crc| f.world.find_trail_blazer_line_mut(crc));
            let s = line.map(|l| TrailLine(std::cell::RefCell::new(l)));
            tb.update(dt, s.as_ref().map(|s| s as &dyn TrailSpline), &car, delay, &mut self.trail_events);
        }
        self.trail_blazer = Some(tb);
        let respawn = self.trail_events[at..].iter().find_map(|e| match *e {
            TrailEvent::Respawn { trigger, speed } => Some((trigger, speed)),
            _ => None,
        });
        if let Some((trigger, delay)) = respawn {
            self.schedule_respawn(f, drv, trigger, delay);
        }
    }

    /// VsAIPoliceVehicle.cpp 0x00018920
    pub fn set_human_control(&mut self, on: bool, f: &mut RaceFrame) {
        let h = self.base.physics.expect("m_pPhysicsVehicle");
        self.set_human_control_on(on, f.world, f.ctx.physics.vehicle(h), f.time);
    }

    /// [`Self::set_human_control`] with its inputs spelt out (PhysicsManager 0x0019E080 calls it outside an update)
    pub fn set_human_control_on(&mut self, on: bool, world: &VsAIWorld, phys: &mut dyn AiPhysicsVehicle, time: f32) {
        self.human_control = on;
        if let Some(si) = self.race_section {
            let s = &world.race_sections()[si];
            self.next_sub_point = s.get_sub_route_point((s.sub_route_points.len() as u32).wrapping_sub(1));
        }
        if self.human_control {
            self.base.active = true;
            phys.set_direct_steer(false);
            self.stuck = false;
            self.grace_until = time + 4.0;
        }
    }

    /// VsAIRaceVehicle.cpp 0x000234F0
    pub fn set_post_race_state(&mut self, s: u32) {
        debug_assert!(s < 4, "(_state >= 0) && (_state < ePR_NumItems)");
        self.post_race_state = s;
    }

    /// arms a failure to trigger after `delay` when `driver_name_crc` matches (optionally only on `lap`).
    /// VsAIRaceVehicle.cpp 0x00023370
    pub fn schedule_failure(&mut self, f: &mut RaceFrame, driver_name: Option<&str>, driver_name_crc: u32, failure: u32, lap: Option<u32>, delay: f32) {
        match lap {
            Some(l) => {
                if l != self.lap {
                    return;
                }
            }
            None => {
                if f.ctx.rand.unit() < 0.5 {
                    return;
                }
            }
        }
        debug_assert!(self.driver.is_some(), "pDriver");
        if crate::crc::string_crc(driver_name.unwrap_or("")) == driver_name_crc {
            self.scheduled_failure_time = f.time + delay;
            self.scheduled_failure = failure;
        }
    }

    /// keep the ai driving after an accident: blown engine, nos explosion or puncture.
    /// VsAIRaceVehicle.cpp 0x00019740
    pub fn trigger_failure(&mut self, f: &mut RaceFrame, drv: Option<&mut VsAIDriver>, failure: u32) {
        debug_assert!(failure != FAILURE_NONE, "eFailure != eFT_None");
        if self.is_player {
            return;
        }
        self.failure = failure;
        self.failure_time = f.time;
        match self.failure {
            0 => {
                f.ctx.host.engine_smoke(self.id, true);
                let g = &f.world.globals;
                let (lo, hi) = (g.oil_spill_min_delay, g.oil_spill_max_delay);
                self.oil_spill_time = f.ctx.rand.range(lo, hi) + f.time;
            }
            1 => {
                self.nos_explosion_flash(f);
                let mut v = self.base.velocity;
                let len = v.length() + f.world.globals.nos_explosion_velocity_increment;
                v.normalise();
                v *= len;
                self.phys(f).set_linear_velocity(&v);
            }
            2..=5 => {
                self.phys(f).puncture(f32::from_bits(0x3D4C_CCCD), f32::from_bits(0x40A0_0000));
                if self.failure == 2 || self.failure == 4 {
                    f.world.spin_impulse = 0.0 - f.world.spin_impulse;
                }
                let imp = VsVector4::new(0.0, f.world.spin_impulse, 0.0, 0.0);
                self.phys(f).apply_angular_impulse(&imp);
            }
            _ => {}
        }
        if !self.is_player
            && let Some(d) = drv
        {
            let mut df = f.driver_frame();
            d.speak(tft::MAKE_MISTAKE, self, &mut df);
        }
    }

    /// failure 1's flash: fades with the distance to player 0 and the angle off camera 0's direction. The render car
    /// point (already world space) goes through the vehicle matrix a second time (original bug, kept).
    /// VsAIRaceVehicle.cpp 0x00019740
    fn nos_explosion_flash(&mut self, f: &mut RaceFrame) {
        // VsRaceManager player 0 (assert "pPlayer", line 880)
        let Some(player) = f.scene.player0().map(|p| p.position) else { return };
        let me = self.base.position;
        let dist = (player - me).length();
        let k = vs_clamp(0.5 / (dist * f32::from_bits(0x3D4C_CCCD)), 0.0, 1.0);
        let Some((cam_pos, cam_dir)) = f.ctx.host.camera(0) else { return };
        let mut to = me - cam_pos;
        to.normalise();
        let d = vs_dot3(&cam_dir, &to);
        if d <= 0.0 {
            return;
        }
        let s = (d + 0.5) * f32::from_bits(0x3F2A_7EFA);
        let a = vs_clamp(k * s, 0.0, 1.0);
        if a <= 0.0 {
            return;
        }
        let Some(h) = self.base.physics else { return };
        // car data +64 translation of this car's render car; the 24 Hz body position without one
        let p = f.ctx.host.car_render_point(h).unwrap_or(me);
        let (r0, r1, r2, r3) = {
            let v = self.phys_ref(f);
            (v.axis_right(), v.axis_up(), v.axis_forward(), v.position())
        };
        // D3DXVec3Transform with the rows 0x001C65A0 / 0x001C6630 / 0x001C66C0 / PhysicsCarInput 0x001BA270
        let pos = VsVector4::new(
            p.x * r0.x + p.y * r1.x + p.z * r2.x + r3.x,
            p.x * r0.y + p.y * r1.y + p.z * r2.y + r3.y,
            p.x * r0.z + p.y * r1.z + p.z * r2.z + r3.z,
            1.0,
        );
        let rec = AiFlashRecord { rise: 0.0, fall: f32::from_bits(0x3D4C_CCCD), intensity: a, screen: false, pos, size: k * 0.5 };
        f.ctx.host.flare_flash(&rec);
    }

    /// a blown engine drags the car to the roadside dropping oil.
    /// VsAIRaceVehicle.cpp 0x00019DD0
    fn update_oil_spill(&mut self, f: &mut RaceFrame) {
        if self.failure != 0 {
            return;
        }
        let right = self.edge_right_sq > self.edge_left_sq;
        let c = self.controls(f);
        c.throttle = 0.0;
        c.brake = f32::from_bits(0x3E4C_CCCD);
        c.steer = if right { f32::from_bits(0x3E4C_CCCD) } else { f32::from_bits(0xBE4C_CCCD) };
        if f.time >= self.oil_spill_time {
            let g = &f.world.globals;
            let (rlo, rhi, life, fric, dlo, dhi) =
                (g.oil_spill_min_radius, g.oil_spill_max_radius, g.oil_spill_lifetime, g.oil_spill_friction_mod, g.oil_spill_min_delay, g.oil_spill_max_delay);
            let radius = f.ctx.rand.range(rlo, rhi);
            let until = f.time + life;
            let along = self.get_dist_along();
            // +0xE0/+0xE4 radius (0x006D3778/7C), +0xE8 friction (0x006D3780), +0xEC lifetime (0x006D3784);
            // ignore car = +0x78 (0x00015B50)
            f.ctx.physics.friction_spot(&self.base.position, fric, radius, along, self.id as i32, until, f.ctx.rand);
            self.oil_spill_time = f.ctx.rand.range(dlo, dhi) + f.time;
        }
    }

    /// refreshes the cached physics state and the position on the route.
    /// VsAIRaceVehicle.cpp 0x00019F40
    pub fn update_state(&mut self, f: &mut RaceFrame, force_closest: bool) {
        let (fwd, pos, ori, vel, spd) = {
            let v = self.phys_ref(f);
            (v.forward(), v.position(), v.orientation(), v.velocity(), v.speed())
        };
        self.base.forward.x = fwd.x;
        self.base.forward.y = fwd.y;
        self.base.forward.z = fwd.z;
        self.base.position = pos;
        self.base.orientation = ori;
        self.controls(f).nitrous = false;
        if self.base.speed > stat_f(&self.stats, 0) {
            set_stat_f(&mut self.stats, 0, self.base.speed);
        }
        self.base.velocity.x = vel.x;
        self.base.velocity.y = vel.y;
        self.base.velocity.z = vel.z;
        self.base.prev_speed = self.base.speed;
        self.base.speed = spd;
        if self.base.forward.dot3(&self.base.velocity) < 0.0 {
            self.base.speed = 0.0 - self.base.speed;
        }
        let Some(si) = self.race_section else { return };
        let p = self.base.position;
        {
            let w = &mut *f.world;
            if force_closest {
                if self.is_player {
                    // VsAIRaceVehicle.cpp 0x000241C0: the player found this route
                    w.race_sections_mut()[si].raceline.flag_0c = true;
                }
                self.raceline_seg = w.race_sections()[si].raceline.find_closest_segment(&p);
            } else {
                self.raceline_seg = w.race_sections()[si].raceline.find_segment(&p, self.raceline_seg);
            }
            let s = &w.race_sections()[si];
            self.dist_along = s.raceline.get_distance_along_line(&p, self.raceline_seg);
            self.lateral = self.lateral_offset(&p, self.raceline_seg, &s.raceline) + self.lane_offset;
            self.prev_race_distance = self.race_distance;
            if s.tracked {
                self.track_distance = self.section_track_distance(w);
                self.race_distance = self.track_race_distance(w);
            }
            if force_closest {
                self.left_seg = s.left.find_closest_segment(&p);
                self.right_seg = s.right.find_closest_segment(&p);
            } else {
                self.left_seg = s.left.find_segment(&p, self.left_seg);
                self.right_seg = s.right.find_segment(&p, self.right_seg);
            }
            self.dist_left = s.left.get_distance_along_line(&p, self.left_seg);
            self.dist_right = s.right.get_distance_along_line(&p, self.right_seg);
        }
        self.check_on_route(f.world);
        let shift = self.phys_ref(f).axis_right() * self.lane_offset;
        let (mut lp, mut rp) = {
            let s = &mut f.world.race_sections_mut()[si];
            let mut i = 0;
            let l = s.left.get_point_at_distance(self.dist_left, self.left_seg, &mut i);
            let r = s.right.get_point_at_distance(self.dist_right, self.right_seg, &mut i);
            (l - shift, r - shift)
        };
        lp.y = 0.0;
        rp.y = 0.0;
        let mut me = self.base.position;
        me.y = 0.0;
        self.edge_left_sq = vs_distance_squared(&lp, &me);
        self.edge_right_sq = vs_distance_squared(&rp, &me);
        self.road_width_sq = vs_distance_squared(&lp, &rp);
        self.check_near_finish(f.ctx.game);
        self.check_upside_down(f);
    }

    /// next branch/join ahead of the car on its section.
    /// VsAIRaceVehicle.cpp 0x0001A5A0
    pub fn find_next_sub_route_point(&mut self, w: &VsAIWorld) {
        let si = self.race_section.expect("m_pCurrentRaceSection");
        let s = &w.race_sections()[si];
        match s.sub_route_points.len() {
            0 => self.next_sub_point = None,
            1 => self.next_sub_point = s.get_sub_route_point(0),
            n => {
                for i in 0..n as u32 {
                    let pi = s.get_sub_route_point(i).unwrap();
                    let p = &w.sub_route_points[pi];
                    if p.from == si && p.dist > self.dist_along + 20.0 {
                        self.next_sub_point = Some(pi);
                        return;
                    }
                }
            }
        }
    }

    /// takes or skips the next branch once within 40m of it.
    /// VsAIRaceVehicle.cpp 0x0001A6A0
    fn update_route(&mut self, f: &mut RaceFrame, drv: Option<&VsAIDriver>) {
        let Some(pi) = self.next_sub_point else { return };
        let p = f.world.sub_route_points[pi].clone();
        let d = vs_distance_squared(&self.base.position, &p.pos);
        if d < 1600.0 && Some(p.from) == self.race_section {
            if self.should_take_sub_route(f, drv, pi) {
                self.set_race_section(f, p.to, false);
            } else {
                self.next_sub_point = p.next;
            }
        }
    }

    /// shortcuts unlock by player prestige and suit only bold enough drivers.
    /// VsAIRaceVehicle.cpp 0x0001A7D0
    fn can_take_shortcut(&self, f: &RaceFrame, drv: Option<&VsAIDriver>, shortcut: &VsAIRoadLine) -> bool {
        debug_assert!(drv.is_some(), "pDriver");
        let Some(d) = drv else { return false };
        let flags = shortcut.header.flags[1];
        // VsAIRaceVehicle.cpp 0x00024270 / 0x000242A0
        if (flags & 0x80) != 0 && !shortcut.flag_0c {
            return false;
        }
        let level = (flags & 7) as usize;
        let prestige = self.stats[21].wrapping_add(f.ctx.game.player_prestige);
        if prestige < params::SHORTCUT_PRESTIGE[level.min(3)] {
            return false;
        }
        let skill = d.get_skill_level(self) as usize;
        let temp = d.get_temperament() as usize;
        params::SHORTCUT_LEVEL[skill.min(3)][temp.min(3)] >= level as u32
    }

    /// VsAIRaceVehicle.cpp 0x0001A930
    fn should_take_sub_route(&self, f: &mut RaceFrame, drv: Option<&VsAIDriver>, pi: usize) -> bool {
        let p = f.world.sub_route_points[pi].clone();
        if !f.world.race_sections()[p.to].flag_25 {
            return false;
        }
        if self.is_player {
            return true;
        }
        if p.next.is_none() && self.race_section != Some(0) {
            return true;
        }
        let Some(p0) = f.scene.player0().cloned() else {
            debug_assert!(false, "pPlayerVehicle");
            return false;
        };
        if self.is_police() {
            if p0.race_section == Some(p.to) {
                log::info!("** Police car {} following player on route {}", self.id, p.to);
                return true;
            }
            return false;
        }
        let to_line = f.world.race_sections()[p.to].raceline.clone();
        if !self.can_take_shortcut(f, drv, &to_line) {
            return false;
        }
        let g = &f.world.globals;
        let gap = vs_clamp(p0.race_distance - self.race_distance, 0.0 - g.shortcut_max_distance_to_player, g.shortcut_max_distance_to_player);
        let d = drv.unwrap();
        let mut prestige = self.stats[21].wrapping_add(f.ctx.game.player_prestige);
        let cap = g.shortcut_max_player_prestige as i32 as u32;
        if prestige > cap {
            prestige = cap;
        }
        let dist_c = gap * g.shortcut_distance_contribution;
        let skill_c = d.get_skill_level(self) as f32 * g.shortcut_skill_contribution;
        let aggr_c = d.get_temperament() as f32 * g.shortcut_aggression_contribution;
        let prestige_c = (prestige as f64 * g.shortcut_prestige_contribution as f64) as f32;
        let chance = vs_clamp(((dist_c + 0.5) + skill_c) + aggr_c + prestige_c, 0.0, 1.0);
        chance >= f.ctx.rand.unit()
    }

    /// a human-driven car: find which section's road it is on.
    /// VsAIRaceVehicle.cpp 0x0001AC70
    fn find_current_section(&mut self, f: &mut RaceFrame) {
        if self.race_section.is_none() || self.on_route {
            return;
        }
        let p = self.base.position;
        let n = f.world.get_num_race_sections() as usize;
        for i in 0..n {
            if Some(i) == self.race_section {
                continue;
            }
            let s = &f.world.race_sections()[i];
            let seg = s.left.find_closest_segment(&p);
            let (a, b) = s.left.get_segment(seg);
            if VsAITools::is_left_of_line(&a, &b, &p) {
                let t = VsAITools::line_parameter(&a, &b, &p);
                if t >= 0.0 && t <= 1.0 {
                    let seg = s.right.find_closest_segment(&p);
                    let (c, d) = s.right.get_segment(seg);
                    if !VsAITools::is_left_of_line(&c, &d, &p) {
                        let t = VsAITools::line_parameter(&c, &d, &p);
                        if t >= 0.0 && t <= 1.0 {
                            self.race_section = Some(i);
                            self.update_state(f, true);
                            break;
                        }
                    }
                }
            }
        }
    }

    /// moves the car onto section `si` and picks its next sub-route point.
    /// VsAIRaceVehicle.cpp 0x0001AEA0
    pub fn set_race_section(&mut self, f: &mut RaceFrame, si: usize, force: bool) {
        // cruise has no race sections: the original's grid reset gets GetFirstRaceSection() == NULL here and
        // reads through it (GetSubRoutePoint 0x00023AE0); the port stores the null section and sub point instead
        let Some(s) = f.world.race_sections().get(si) else {
            self.next_sub_point = None;
            self.race_section = None;
            return;
        };
        let cur_join = self.next_sub_point.is_some_and(|p| f.world.sub_route_points[p].join);
        let mut p = None;
        if force || !cur_join {
            p = s.get_sub_route_point(0);
        } else {
            let cur_dist = f.world.sub_route_points[self.next_sub_point.unwrap()].dist;
            for i in 0..s.sub_route_points.len() as u32 {
                let q = s.get_sub_route_point(i).unwrap();
                let qp = &f.world.sub_route_points[q];
                if !qp.join && qp.dist > cur_dist {
                    p = Some(q);
                    break;
                }
            }
            if p.is_none() {
                p = if !s.raceline.is_circular() {
                    s.get_sub_route_point((s.sub_route_points.len() as u32).wrapping_sub(1))
                } else {
                    s.get_sub_route_point(0)
                };
            }
        }
        self.next_sub_point = p;
        self.race_section = Some(si);
        self.update_state(f, true);
    }

    /// race-state transitions.
    /// VsAIRaceVehicle.cpp 0x0001AFE0
    fn update_race_state(&mut self, f: &mut RaceFrame, drv: Option<&mut VsAIDriver>) {
        match self.state {
            RaceState::Grid => {
                self.phys(f).set_launch_control(true);
                if f.ctx.game.race_phase >= 4 {
                    self.grace_until = f.time + 2.0;
                    self.flag_30a = true;
                    self.set_state(f, drv, RaceState::Launch);
                    self.phys(f).set_launch_control(false);
                }
            }
            RaceState::Launch => {
                let mut elapsed = 0.0;
                if self.launch_first {
                    self.launch_first = false;
                    self.launch_time = f.ctx.game.timer_time;
                } else {
                    elapsed = f.ctx.game.timer_time - self.launch_time;
                }
                if elapsed > 3.0 {
                    self.set_state(f, drv, RaceState::Racing);
                }
            }
            RaceState::PlayerAssist => {}
            RaceState::Racing | RaceState::Chase => {
                if self.finished_flag {
                    self.set_state(f, drv, RaceState::Finished);
                } else if self.is_stuck(f.time) {
                    self.set_state(f, drv, RaceState::Recover);
                }
            }
            RaceState::Recover => {
                if self.can_leave_recover(f.time) {
                    let prev = self.prev_state;
                    self.set_state(f, drv, prev);
                } else if self.recover_timed_out(f.time) {
                    self.respawn_on_raceline(f, 0.0);
                }
            }
            RaceState::Idle6 => debug_assert!(false),
            RaceState::Finished => {}
        }
    }

    /// VsAIRaceVehicle.cpp 0x0001B2F0
    pub fn set_state(&mut self, f: &mut RaceFrame, mut drv: Option<&mut VsAIDriver>, new: RaceState) {
        debug_assert!((new as u32) < 8, "eNewState >= 0 && eNewState < eRS_NumStates");
        self.state_exit(f, drv.as_deref());
        if self.state != RaceState::Recover {
            self.prev_state = self.state;
        }
        self.state = new;
        self.state_enter(f, drv.as_deref_mut());
    }

    fn state_enter(&mut self, f: &mut RaceFrame, drv: Option<&mut VsAIDriver>) {
        match self.state {
            // VsAIRaceVehicle.cpp 0x0001B4F0
            RaceState::Grid => {
                self.phys(f).set_direct_steer(true);
                let c = self.controls(f);
                c.throttle = 1.0;
                c.steer = 0.0;
                c.brake = 0.0;
                c.handbrake = 1.0;
                self.startline_revs.has_vehicle = true;
            }
            // VsAIRaceVehicle.cpp 0x0001B5F0
            RaceState::Launch => {
                let base = drv.as_deref().map_or(1, |d| d.get_base_aggression_level(self));
                self.aggression_scale = ((base + 1).min(3) as f64 * (1.0 / 3.0)) as f32;
                self.aggression_scale = self.aggression_scale * f32::from_bits(0x3ECC_CCCD) + f32::from_bits(0x3F19_999A);
                if base == 3 {
                    let p = self.phys_ref(f).launch_params();
                    self.saved_grip = [p[2], p[0], p[1]];
                    self.phys(f).set_launch_params([0.0, 0.0, p[2] * params::SUPERCAR_GRIP_SCALE]);
                }
                self.launch_first = true;
                self.launch_time = 0.0;
            }
            // VsAIRaceVehicle.cpp 0x0001B950
            RaceState::PlayerAssist => {
                if self.is_player {
                    self.set_human_control(false, f);
                }
            }
            // VsAIRaceVehicle.cpp 0x0001BC00
            RaceState::Chase => {
                debug_assert!(self.chase_target != VehicleRef::None, "m_pChaseTargetVehicle != NULL");
                debug_assert!(self.race_section.is_some(), "m_pCurrentRaceSection != NULL");
            }
            // VsAIRaceVehicle.cpp 0x0001BF40
            RaceState::Finished => {
                if self.failure != FAILURE_NONE {
                    self.clear_failure(true, f.ctx.host);
                }
                if !self.is_player
                    && let Some(d) = drv
                {
                    let ev = if stat_position(&self.stats) == 1 { tft::WINNING } else { tft::LOST };
                    let mut df = f.driver_frame();
                    d.speak(ev, self, &mut df);
                }
            }
            _ => {}
        }
    }

    fn state_exit(&mut self, f: &mut RaceFrame, drv: Option<&VsAIDriver>) {
        match self.state {
            // VsAIRaceVehicle.cpp 0x0001B5A0
            RaceState::Grid => {
                self.phys(f).set_direct_steer(false);
                let c = self.controls(f);
                c.unk_18 = false;
                c.handbrake = 0.0;
            }
            // VsAIRaceVehicle.cpp 0x0001B8D0
            RaceState::Launch => {
                if drv.map_or(1, |d| d.get_base_aggression_level(self)) == 3 {
                    let s = self.saved_grip;
                    self.phys(f).set_launch_params([s[1], s[2], s[0]]);
                }
            }
            // VsAIRaceVehicle.cpp 0x0001BA60
            RaceState::PlayerAssist => {
                if self.is_player {
                    self.set_human_control(true, f);
                }
            }
            _ => {}
        }
    }

    fn state_update(&mut self, f: &mut RaceFrame, drv: Option<&mut VsAIDriver>) {
        match self.state {
            RaceState::Grid => {}
            // VsAIRaceVehicle.cpp 0x0001B790
            RaceState::Launch => {
                if !self.is_disoriented(drv.as_deref(), f.time) {
                    self.update_route(f, drv.as_deref());
                    self.drive_common(f, drv, VehicleRef::None, true, true);
                    let s = self.aggression_scale;
                    self.controls(f).throttle *= s;
                }
            }
            // VsAIRaceVehicle.cpp 0x0001B990
            RaceState::PlayerAssist => {
                self.compute_look_ahead_for_speed(f);
                self.avoiding = self.avoidance(f);
                if !self.avoiding {
                    self.follow_and_overtake(f);
                }
                self.apply_lateral_offset(f);
                self.update_steering(f, drv.as_deref());
                self.update_throttle_brake(f, drv.as_deref());
            }
            // VsAIRaceVehicle.cpp 0x0001BAC0
            RaceState::Racing => {
                if !self.is_disoriented(drv.as_deref(), f.time) {
                    self.update_route(f, drv.as_deref());
                    self.drive_common(f, drv, VehicleRef::None, true, true);
                }
            }
            // VsAIRaceVehicle.cpp 0x0001BCE0
            RaceState::Chase => {
                if !self.is_disoriented(drv.as_deref(), f.time) {
                    debug_assert!(self.chase_target != VehicleRef::None, "m_pChaseTargetVehicle != NULL");
                    self.update_route(f, drv.as_deref());
                    let t = self.chase_target;
                    self.drive_common(f, drv, t, false, false);
                }
            }
            // VsAIRaceVehicle.cpp 0x0001BE90
            RaceState::Recover => self.reverse_out(f),
            RaceState::Idle6 => {}
            // VsAIRaceVehicle.cpp 0x0001BFC0
            RaceState::Finished => self.finished_drive(f),
        }
    }

    /// shared body of the driving states (lookahead, avoidance, following, offsets, controls).
    fn drive_common(&mut self, f: &mut RaceFrame, drv: Option<&mut VsAIDriver>, target: VehicleRef, target_only_when_free: bool, stop_check: bool) {
        self.compute_look_ahead_for_speed(f);
        self.avoiding = self.avoidance(f);
        if !self.avoiding {
            self.follow_and_overtake(f);
            if target_only_when_free {
                self.target_behaviour(f, drv.as_deref(), target);
            }
        }
        if !target_only_when_free {
            self.target_behaviour(f, drv.as_deref(), target);
        }
        self.apply_lateral_offset(f);
        self.update_steering(f, drv.as_deref());
        if !self.is_police() {
            self.update_nitrous(f, drv.as_deref());
            self.update_slipstream(f, drv.as_deref());
            if stop_check {
                self.time_trial_stop(f);
            }
        }
        self.update_throttle_brake(f, drv.as_deref());
    }

    /// lookahead distance from the speed-ratio curve.
    fn compute_look_ahead_for_speed(&mut self, f: &mut RaceFrame) {
        let h = self.handling(f.world).clone();
        let v = self.rc(f.world, h.speed_ratio_rc, self.base.speed / h.max_speed);
        self.compute_look_ahead(f.world, (h.look_ahead_max - h.look_ahead_min) * v + h.look_ahead_min);
    }

    /// wheel angle to the first lookahead point; corner tightness from the other two.
    /// VsAIRaceVehicle.cpp 0x0001C010
    fn update_steering(&mut self, f: &mut RaceFrame, drv: Option<&VsAIDriver>) {
        self.controls(f).horn = false;
        self.phys(f).set_direct_steer(false);
        let skill = drv.map_or(1, |d| d.get_skill_level(self)) as f32;
        f.world.globals.tight_corner_threshold = skill * 0.5 + f.world.globals.difficulty_scale;
        self.steer_angle = self.base.get_steer_towards(&self.look_ahead[0].pos, 1.0);
        let a = self.steer_angle;
        self.phys(f).set_steer_angle(a);
        let v8 = self.base.get_steer_towards(&self.look_ahead[1].pos, 1.0);
        self.corner_tightness = self.base.get_steer_towards(&self.look_ahead[2].pos, 1.0) + v8;
        self.corner_tightness = (self.corner_tightness * self.corner_tightness) * self.corner_tightness;
    }

    /// throttle/brake from corner tightness (or speed matching), plus ai gear changes.
    /// VsAIRaceVehicle.cpp 0x0001C1C0
    fn update_throttle_brake(&mut self, f: &mut RaceFrame, drv: Option<&VsAIDriver>) {
        let mut throttle: f32;
        let mut brake: f32;
        let g = &f.world.globals;
        if self.base.flag_004 {
            throttle = 0.0;
            brake = 0.5;
        } else if self.speed_match || g.force_speed_match {
            let (diff, ratio) = if g.force_speed_match {
                (self.base.speed - g.force_speed, self.base.speed / g.force_speed)
            } else {
                (self.base.speed - self.target_speed, self.base.speed / self.target_speed)
            };
            let k = params::SPEED_MATCH_GAIN;
            if diff <= 0.0 {
                throttle = 1.0;
                brake = 0.0;
                if ratio < 1.0 && ratio >= 1.0 - k * 0.01 {
                    throttle = (1.0 - ratio) * k;
                }
            } else {
                throttle = 0.0;
                brake = 1.0;
                if ratio > 1.0 && k * 0.01 + 1.0 >= ratio {
                    brake = (ratio - 1.0) * k;
                }
            }
        } else {
            let h = self.handling(f.world);
            if vs_abs(self.corner_tightness) < g.tight_corner_threshold {
                throttle = self.rc(f.world, h.throttle_rc, 0.0);
                brake = self.rc(f.world, h.brake_rc, 0.0);
            } else {
                let c = self.corner_tightness;
                let sr = self.rc(f.world, h.speed_ratio_rc, self.base.speed / h.max_speed);
                let x = vs_abs(c) * sr * h.throttle_factor * 2.0;
                throttle = self.rc(f.world, h.throttle_rc, x);
                let y = vs_abs(c) * sr * h.brake_factor * 2.0;
                brake = self.rc(f.world, h.brake_rc, y);
            }
        }
        let (up_ok, down_ok) = self.phys_ref(f).shift_flags();
        if up_ok || down_ok {
            let gear = self.phys_ref(f).gear();
            if gear < 100 {
                let je = self.judgment_error(drv, &f.world.globals);
                let es = self.phys_ref(f).engine_speed();
                if throttle <= 0.0 {
                    if brake > 0.0 && down_ok && gear > 0 && es - 450.0 >= je {
                        self.phys(f).shift(-1);
                    }
                } else if up_ok {
                    let n = self.phys_ref(f).gear_count();
                    if gear < n - 1 && es - 950.0 >= je {
                        self.phys(f).shift(1);
                    }
                }
            }
        }
        let c = self.controls(f);
        c.throttle = throttle;
        c.brake = brake;
    }

    /// how far off a driver of this skill can misjudge things right now.
    /// VsAIRaceVehicle.cpp 0x0001C600
    fn judgment_error(&self, drv: Option<&VsAIDriver>, g: &VsAIGlobals) -> f32 {
        let Some(d) = drv else { return 0.0 };
        let a = d.get_aggression();
        match d.get_skill_level(self) {
            0 => g.judgment_error[0] * a,
            1 => g.judgment_error[1] * a,
            2 | 3 => g.judgment_error[2] * a,
            _ => {
                debug_assert!(false);
                0.0
            }
        }
    }

    /// VsAIRaceVehicle.cpp 0x0001C790
    fn apply_judgment(&self, drv: Option<&VsAIDriver>, g: &VsAIGlobals, time: f32, x: f32) -> f32 {
        // VsAIRaceVehicle.cpp 0x0001C740
        let wobble = vs_sin(time * 0.1);
        (self.judgment_error(drv, g) * wobble + 1.0) * x
    }

    /// smallest heading change (as a dot product) along the lookahead within `len`; 0 if any bend
    /// exceeds 22.5 degrees.
    /// VsAIRaceVehicle.cpp 0x0001C7E0
    fn straightness_ahead(&self, len: f32) -> f32 {
        let mut best = 1.0f32;
        let mut pos = self.base.position;
        let mut dir = self.base.forward;
        let l = dir.length_squared() - 1.0;
        debug_assert!(vs_abs(l) < 0.0001, "VsAbs(direction.LengthSquared() - 1.0f) < 0.0001f");
        let mut remaining = self.get_dist_along() + len;
        for i in 0..3 {
            let p = &self.look_ahead[i];
            let mut d = p.pos - pos;
            d.normalise();
            let dot = dir.dot3(&d);
            if dot < f32::from_bits(0x3F6C_8B44) {
                return 0.0;
            }
            if best > dot {
                best = dot;
            }
            if p.dist >= remaining {
                break;
            }
            remaining -= p.dist;
            pos = p.pos;
            dir = d;
        }
        best
    }

    /// aggression tier: 3 counts as 2.
    fn tier(drv: Option<&VsAIDriver>) -> usize {
        let t = drv.map_or(0, |d| d.get_temperament());
        if t == 3 { 2 } else { t as usize }
    }

    /// VsAIRaceVehicle.cpp 0x0001CA60
    fn nos_straight_length(&self, drv: Option<&VsAIDriver>, g: &VsAIGlobals, time: f32) -> f32 {
        let on = self.nos_start > 0.0;
        let n = &g.nitrous_aggression[Self::tier(drv)];
        let mut len = if on { n.continue_straightness_length } else { n.start_straightness_length };
        // VsAIRaceVehicle.cpp 0x00024670
        let overtaken = drv.map_or(-1.0, |d| d.overtaken_time);
        if overtaken >= 0.0 {
            let dt = time - overtaken;
            debug_assert!(vs_fge(dt, 0.0, 0.000001), "VsFge(dt, 0.0f)");
            if n.overtaken_time_window > dt {
                len *= n.overtaken_length_factor;
            }
        }
        len
    }

    /// VsAIRaceVehicle.cpp 0x0001CC80
    fn nos_max_steering(&self, drv: Option<&VsAIDriver>, g: &VsAIGlobals) -> f32 {
        let t = drv.map_or(0, |d| d.get_temperament());
        match t {
            0 => g.nitrous_aggression[0].max_steering,
            1 => g.nitrous_aggression[1].max_steering,
            2 | 3 => g.nitrous_aggression[2].max_steering,
            _ => 1.0,
        }
    }

    /// VsAIRaceVehicle.cpp 0x0001CD50
    fn nos_min_straightness(&self, drv: Option<&VsAIDriver>, g: &VsAIGlobals) -> f32 {
        let t = drv.map_or(0, |d| d.get_temperament());
        let deg = match t {
            0 => g.nitrous_aggression[0].minimum_road_straightness,
            1 => g.nitrous_aggression[1].minimum_road_straightness,
            2 | 3 => g.nitrous_aggression[2].minimum_road_straightness,
            _ => 0.0,
        };
        vs_cos(f32::from_bits(0x3C8E_FA35) * deg)
    }

    /// VsAIRaceVehicle.cpp 0x0001CE60
    fn nos_conservation(&self, drv: Option<&VsAIDriver>, g: &VsAIGlobals) -> f32 {
        match drv.map_or(0, |d| d.get_skill_level(self)) {
            0 => g.nos_conservation_propensity[0],
            1 => g.nos_conservation_propensity[1],
            2 | 3 => g.nos_conservation_propensity[2],
            _ => 1.0,
        }
    }

    /// keep nitrous back while plenty of race remains.
    /// VsAIRaceVehicle.cpp 0x0001CF30
    fn should_conserve_nos(&self, f: &RaceFrame, drv: Option<&VsAIDriver>) -> bool {
        let laps = f.ctx.event.num_laps as i32;
        debug_assert!(laps != 0, "NumLaps > 0");
        let lap_len = f.lap_length();
        debug_assert!(lap_len > 0.0, "LapLength > 0.0f");
        let total = laps as f32 * lap_len;
        let g = &f.world.globals;
        if self.race_distance + g.nos_eor_no_conservation_length > total {
            return false;
        }
        let remaining = 1.0 - self.race_distance / total;
        let v = self.phys_ref(f);
        // VsAIRaceVehicle.cpp 0x00024720
        let frac = v.nos_remaining() / v.nos_capacity();
        let x = vs_clamp(self.apply_judgment(drv, g, f.time, frac), 0.0, 1.0);
        x <= self.nos_conservation(drv, g) * remaining && self.nos_start < 0.0
    }

    /// 1 when everything says "boost now".
    /// VsAIRaceVehicle.cpp 0x0001D160
    fn want_nitrous(&self, f: &RaceFrame, drv: Option<&VsAIDriver>) -> f32 {
        if !self.started {
            return 0.0;
        }
        let v = self.phys_ref(f);
        // VsAIRaceVehicle.cpp 0x000248F0
        if vs_fle(v.nos_remaining() / 24.0, 0.0, 0.000001) {
            return 0.0;
        }
        if v.drive_state() != 4 || v.is_reversing() || v.is_airborne() || v.forward_speed() < 10.0 {
            return 0.0;
        }
        let c = *f.ctx.physics.vehicle_ref(self.base.physics.unwrap()).controls_ref();
        if c.brake > 0.0 || c.throttle < f32::from_bits(0x3F73_3333) {
            return 0.0;
        }
        let g = &f.world.globals;
        let p0 = f.scene.player0();
        if let Some(p0) = p0
            && self.race_distance - g.nos_ahead_of_player_threshold > p0.race_distance
        {
            return 0.0;
        }
        let Some(p0) = p0 else { return 0.0 };
        let gap = p0.race_distance - self.race_distance;
        if !(g.nos_speed_check_distance <= vs_abs(gap) || self.base.speed - p0.speed <= g.nos_speed_check_threshold) {
            return 0.0;
        }
        if vs_abs(c.steer) > self.nos_max_steering(drv, g) {
            return 0.0;
        }
        let len = self.apply_judgment(drv, g, f.time, self.nos_straight_length(drv, g, f.time));
        let s = 1.0 - self.apply_judgment(drv, g, f.time, 1.0 - self.straightness_ahead(len));
        if self.nos_min_straightness(drv, g) > s {
            return 0.0;
        }
        for o in f.scene.players.iter().chain(f.scene.ai.iter()) {
            if o.id == self.id && o.is_player == self.is_player {
                continue;
            }
            let mut d = o.position - self.base.position;
            let dsq = d.length_squared();
            d.normalise();
            let dot = d.dot3(&self.base.forward);
            if dot < f32::from_bits(0x3F78_51EC) && dsq < 100.0 {
                return 0.0;
            }
        }
        if self.should_conserve_nos(f, drv) { 0.0 } else { 1.0 }
    }

    /// VsAIRaceVehicle.cpp 0x0001D690
    fn update_nitrous(&mut self, f: &mut RaceFrame, drv: Option<&VsAIDriver>) {
        if self.is_player {
            return;
        }
        let want = self.want_nitrous(f, drv);
        self.nos_desire = if want == 0.0 { self.nos_desire - 1.0 } else { want * f32::from_bits(0x3DCC_CCCD) + self.nos_desire };
        self.nos_desire = vs_clamp(self.nos_desire, 0.0, 1.0);
        let on = f.world.globals.use_nitrous && self.nos_desire > 0.5;
        self.controls(f).nitrous = on;
        if (self.nos_start > 0.0) != on {
            if on {
                self.nos_start = f.time;
                log::debug!("Car {} starts using nitrous", self.id);
            } else {
                self.nos_start = -1.0;
                self.nos_stop = f.time;
                log::debug!("Car {} stops using nitrous", self.id);
            }
        }
    }

    /// 1 when `o` is a good car to tuck in behind.
    /// VsAIRaceVehicle.cpp 0x0001D8A0
    fn slipstream_score(&self, f: &RaceFrame, drv: Option<&VsAIDriver>, o: &AiVehicleView) -> f32 {
        let Some(oh) = o.physics else { return 0.0 };
        let ov = f.ctx.physics.vehicle_ref(oh);
        if ov.is_airborne() || ov.is_reversing() {
            return 0.0;
        }
        let tier = {
            // police drivers (no opponent): no-driver default, as in catchup_boost
            let s = drv.map_or(1, |d| d.get_skill_level(self));
            if s == super::vs_ai_driver::AIA_NOT_SET { 1 } else if s == 3 { 2 } else { s as usize }
        };
        let p = &f.world.globals.slipstream[tier];
        let same = self.slipstream_target == o.r;
        if !(same || p.min_speed <= self.base.speed) {
            return 0.0;
        }
        let diff = o.speed - self.base.speed;
        let lo = if same { p.min_speed_diff_current } else { p.min_speed_diff_new };
        if !(lo <= diff && diff <= p.max_speed_difference) {
            return 0.0;
        }
        let mut d = o.position;
        d -= self.base.position;
        let dsq = d.length_squared();
        if p.min_distance * p.min_distance > dsq {
            return 0.0;
        }
        if Some(oh) != f.ctx.physics.slipstream_source(self.base.physics.unwrap()) {
            if dsq > p.semicircle_radius * p.semicircle_radius {
                return 0.0;
            }
            let dot = self.base.forward.dot3(&d);
            if vs_cos(f32::from_bits(0x3C8E_FA35) * p.semicircle_angle) > dot {
                return 0.0;
            }
        }
        let mut to = self.look_ahead[0].pos;
        to -= self.base.position;
        to.normalise();
        let dot = to.dot3(&o.forward);
        if vs_cos(f32::from_bits(0x3C8E_FA35) * p.direction_difference) <= dot { 1.0 } else { 0.0 }
    }

    /// VsAIRaceVehicle.cpp 0x0001DD00
    fn find_slipstream_target(&self, f: &RaceFrame, drv: Option<&VsAIDriver>) -> VehicleRef {
        if !f.world.globals.use_slipstream || self.slip_lock_until > f.time {
            return VehicleRef::None;
        }
        if self.phys_ref(f).is_airborne() {
            return VehicleRef::None;
        }
        if self.phys_ref(f).controls_ref().brake > 0.0 {
            return VehicleRef::None;
        }
        let mut best = 0.0f32;
        let mut target = VehicleRef::None;
        let src = f.ctx.physics.slipstream_source(self.base.physics.unwrap());
        for o in f.scene.players.iter().chain(f.scene.ai.iter()) {
            if o.id == self.id && o.is_player == self.is_player {
                continue;
            }
            let mut s = self.slipstream_score(f, drv, o);
            if !vs_feq(s, 0.0, 0.000001) {
                if src == o.physics {
                    s += f32::from_bits(0x3DCC_CCCD);
                }
                if s > best {
                    best = s;
                    target = o.r;
                }
            }
        }
        target
    }

    /// VsAIRaceVehicle.cpp 0x0001DFF0
    fn update_slipstream(&mut self, f: &mut RaceFrame, drv: Option<&VsAIDriver>) {
        if self.is_player {
            return;
        }
        self.slipstream_target = self.find_slipstream_target(f, drv);
        if let Some(t) = f.scene.get(self.slipstream_target).cloned() {
            let la = self.look_ahead[0].pos;
            let sm = match drv.map_or(1, |d| d.get_skill_level(self)) {
                0 => f.world.globals.slipstream[0].steering_smoothness,
                1 => f.world.globals.slipstream[1].steering_smoothness,
                _ => f.world.globals.slipstream[2].steering_smoothness,
            };
            let h = self.base.physics.unwrap();
            let a1 = f.ctx.physics.steer_angle_to(h, &t.position);
            let a2 = f.ctx.physics.steer_angle_to(h, &la);
            self.steer_angle = a1 * sm + (1.0 - sm) * a2;
            debug_assert!(self.steer_angle >= -3.141_592_7 && self.steer_angle <= 3.141_592_7, "m_ftSteerAngle >= -VS_PI && m_ftSteerAngle <= VS_PI");
            let a = self.steer_angle;
            self.phys(f).set_steer_angle(a);
        }
        if let Some(t) = f.scene.get(self.overtake_target).cloned()
            && f.time > self.slip_lock_until
            && self.base.speed > t.speed
            && t.race_distance > self.race_distance
        {
            let d = t.position - self.base.position;
            if params::FOLLOW_DISTANCE * params::FOLLOW_DISTANCE >= d.length_squared() {
                // VsAIRaceVehicle.cpp 0x0001DFD0 always says no
                if f.world.globals.spd_diff_thresh_quick_overtake <= self.base.speed - t.speed {
                    self.slip_lock_until = f.time + 3.0;
                    self.follow_and_overtake(f);
                } else {
                    self.speed_match = true;
                    self.target_speed = t.speed;
                }
            }
        }
    }

    /// time-trial style events stop at the end of the distance.
    /// VsAIRaceVehicle.cpp 0x0001E400
    fn time_trial_stop(&mut self, f: &mut RaceFrame) {
        if f.ctx.event.mode == 5 && self.started {
            let stop = (self.base.speed * self.base.speed) / (2.0 * 30.0);
            let total = f.ctx.event.num_laps as f32 * f.lap_length();
            if stop > (total + 3.0) - self.race_distance {
                self.speed_match = true;
                self.target_speed = 0.0;
            }
        }
    }

    /// back out of a stuck position.
    /// VsAIRaceVehicle.cpp 0x0001E510
    fn reverse_out(&mut self, f: &mut RaceFrame) {
        self.controls(f).horn = true;
        let a = self.base.get_steer_towards(&self.look_ahead[0].pos, 1.0);
        self.phys(f).set_steer_angle(0.0 - a);
        self.phys(f).set_direct_steer(false);
        let c = self.controls(f);
        c.throttle = 0.0;
        c.brake = 1.0;
    }
}

impl VsAIRaceVehicle {
    /// VsAIRaceVehicle.cpp 0x00022B80
    fn is_disoriented(&self, drv: Option<&VsAIDriver>, time: f32) -> bool {
        drv.is_some_and(|d| d.is_disoriented(time))
    }

    /// avoid points around the cars ahead; steers the lateral offset to the most urgent one.
    /// VsAIRaceVehicle.cpp 0x0001E5C0
    #[allow(unused_assignments)] // the original breaks out of its best-time search at the first hit
    fn avoidance(&mut self, f: &mut RaceFrame) -> bool {
        let radius = 5.0f32;
        self.build_avoid_list(f);
        let si = self.race_section.expect("m_pCurrentRaceSection");
        for i in 0..self.avoid.len() {
            let Some(o) = f.scene.get(self.avoid[i]).cloned() else { continue };
            let mut opos = o.position;
            self.avoid_info[i].vehicle = o.r;
            let my_lat = self.lateral;
            let (seg, olat) = if o.is_traffic {
                let rl = self.raceline(f.world);
                let s = rl.find_closest_segment(&opos);
                (s, self.lateral_offset(&opos, s, rl))
            } else {
                (self.raceline_seg, o.lateral)
            };
            let dlat = olat - my_lat;
            let odist = if o.is_traffic { self.raceline(f.world).get_distance_along_line(&opos, seg) } else { o.dist_along };
            let rlp = self.raceline_point(f.world, odist, seg);
            opos.y = rlp.y;
            let mut dir = if (dlat <= 0.0 || olat <= 0.0) && (dlat >= 0.0 || olat >= 0.0) { opos - rlp } else { rlp - opos };
            dir.normalise();
            dir *= radius;
            let mut ap = opos + dir;
            self.avoid_info[i].close = radius > vs_abs(dlat);
            self.avoid_info[i].dist = self.raceline(f.world).get_distance_along_line(&ap, seg);
            let side = self.classify_side(f.world, si, &mut ap, false);
            if side == -1 || side == 1 {
                ap -= dir * 2.0;
            }
            let rl = self.raceline(f.world);
            let s2 = rl.find_closest_segment(&ap);
            let off = self.lateral_offset(&ap, s2, rl);
            let mut mv = self.base.velocity;
            mv.normalise();
            let mut ov = o.velocity;
            ov.normalise();
            let c = vs_dot3(&mv, &ov);
            let mut ang = vs_acos(c);
            if ang > 1.570_796_4 {
                ang = 3.141_592_7 - ang;
            }
            let k = vs_cos(ang);
            let mut closing = o.velocity.length() * k;
            if c < 0.0 {
                closing *= -1.0;
            }
            let rel = self.base.speed - closing;
            let a = &mut self.avoid_info[i];
            a.time = (odist - self.dist_along) / rel;
            a.pos = ap;
            a.offset = off;
            a.is_traffic = o.is_traffic;
        }
        let mut best_t = 1_000_000.0f32;
        let any_traffic = self.avoid_info[..self.avoid.len()].iter().any(|a| a.is_traffic);
        let horizon = if any_traffic { 5.0 } else { 3.0 };
        let mut target = None;
        for i in 0..self.avoid.len() {
            let a = &self.avoid_info[i];
            if a.close && best_t > a.time && horizon > a.time && a.time >= 0.0 {
                best_t = a.time;
                target = Some(i);
                break;
            }
        }
        if let Some(t) = target {
            let a = self.avoid_info[t];
            let diff = vs_abs(self.lateral - a.offset);
            let mut urgency = horizon - a.time;
            if urgency < 1.0 {
                urgency = 1.0;
            }
            self.set_offset_target(a.offset, urgency * diff);
        } else {
            // VsAIRaceVehicle.cpp 0x00024A80
            self.offset_active = false;
        }
        target.is_some()
    }

    /// within 50m and in front.
    /// VsAIRaceVehicle.cpp 0x0001F070
    fn is_ahead_within_50(&self, p: &VsVector4) -> bool {
        if 50.0 * 50.0 <= vs_distance_squared(&self.base.position, p) {
            return false;
        }
        let mut d = *p;
        d -= self.base.position;
        vs_dot3(&self.base.forward, &d) > 0.0
    }

    /// cars, traffic and police ahead on this section, nearest along the raceline first.
    /// VsAIRaceVehicle.cpp 0x0001F150
    fn build_avoid_list(&mut self, f: &mut RaceFrame) {
        self.avoid.clear();
        let police = self.is_police();
        let push = |me: &mut Self, r: VehicleRef| {
            debug_assert!(me.avoid.len() < 20, "m_ppVehiclesToAvoid != NULL && m_dVehiclesToAvoidCount < (MAX_TRAFFIC_VEHICLES + MAX_RACE_VEHICLES)");
            // VsAIRaceVehicle.cpp 0x000203D0
            if me.avoid.len() < 20 {
                me.avoid.push(r);
            }
        };
        for o in f.scene.ai.iter().chain(f.scene.players.iter()) {
            if o.race_section == self.race_section && (!police || o.r != self.chase_target) && self.is_ahead_within_50(&o.position) {
                push(self, o.r);
            }
        }
        if f.world.traffic_enabled {
            for o in &f.scene.traffic {
                if o.active && self.is_ahead_within_50(&o.position) {
                    push(self, o.r);
                }
            }
        }
        if f.world.police_enabled {
            for o in &f.scene.police {
                if o.active && self.is_ahead_within_50(&o.position) {
                    push(self, o.r);
                }
            }
        }
        // distance of traffic along this car's raceline (the original stores it in the traffic car, +0xEC)
        let mut key: Vec<f32> = Vec::with_capacity(self.avoid.len());
        for &r in &self.avoid {
            let o = f.scene.get(r).unwrap();
            key.push(if o.is_traffic {
                self.raceline(f.world).get_distance_along_line(&o.position, self.raceline_seg)
            } else {
                o.dist_along
            });
        }
        let mut swapped = 1;
        while swapped != 0 && !self.avoid.is_empty() {
            swapped = 0;
            for j in 0..self.avoid.len() - 1 {
                if key[j] > key[j + 1] {
                    self.avoid.swap(j, j + 1);
                    key.swap(j, j + 1);
                    swapped += 1;
                }
            }
        }
    }

    /// distance along the raceline between this car and `o` (wrapping a lap when `o` is behind).
    fn gap_ahead(&self, f: &RaceFrame, o: &AiVehicleView) -> f32 {
        if self.get_dist_along() <= o.dist_along {
            o.dist_along - self.get_dist_along()
        } else {
            f.lap_length() - self.get_dist_along() + o.dist_along
        }
    }

    /// follow the car ahead, pull out to pass it, or move aside for the car behind.
    /// VsAIRaceVehicle.cpp 0x0001F6D0
    fn follow_and_overtake(&mut self, f: &mut RaceFrame) {
        let mut handled = false;
        if let Some(a) = f.scene.get(self.vehicle_ahead).cloned()
            && a.race_section == self.race_section
            && !self.is_police()
            && self.chase_target != self.vehicle_ahead
        {
            let gap = self.gap_ahead(f, &a);
            let dlat = a.lateral - self.lateral;
            if params::FOLLOW_LATERAL > vs_abs(dlat) && self.base.speed > a.speed && params::FOLLOW_DISTANCE > gap {
                self.speed_match = true;
                self.target_speed = a.speed;
            }
            let mut sp = a.speed;
            if sp < 0.001 {
                sp = 0.001;
            }
            let ratio = self.base.speed / sp;
            let mut look = params::OVERTAKE_LOOKAHEAD * ratio;
            if params::OVERTAKE_LOOKAHEAD * ratio > 50.0 {
                look = 50.0;
            }
            // handling +0x14: the "RC_sp3" slot (a copy of the brake curve, see VsAIWorld::load_handling)
            let rc = self.handling(f.world).sp3_rc;
            let rate = self.rc(f.world, rc, gap / look) * 5.0;
            if params::OVERTAKE_LATERAL > vs_abs(dlat) && look > gap && look > 0.0 {
                self.offset_target = if dlat <= 0.0 { a.lateral + params::OVERTAKE_LATERAL } else { a.lateral - params::OVERTAKE_LATERAL };
                let t = self.offset_target;
                self.set_offset_target(t, rate);
                handled = true;
            }
        }
        if let Some(b) = f.scene.get(self.vehicle_behind).cloned()
            && b.race_section == self.race_section
            && !handled
            && !self.is_police()
            && self.chase_target != self.vehicle_behind
        {
            let gap = if b.dist_along <= self.get_dist_along() {
                self.get_dist_along() - b.dist_along
            } else {
                f.lap_length() - b.dist_along + self.get_dist_along()
            };
            let dlat = b.lateral - self.lateral;
            if params::OVERTAKE_LATERAL > vs_abs(dlat) && gap < 8.0 {
                self.offset_target = if dlat <= 0.0 { b.lateral + params::OVERTAKE_LATERAL } else { b.lateral - params::OVERTAKE_LATERAL };
                let t = self.offset_target;
                self.set_offset_target(t, 5.0);
                handled = true;
            }
        }
        if !handled {
            self.offset_active = false;
        }
    }

    /// side-by-side with a rival: hold the line, then bump it.
    /// VsAIRaceVehicle.cpp 0x0001FCF0
    fn rival_behaviour(&mut self, f: &RaceFrame, drv: Option<&VsAIDriver>, t: &AiVehicleView) -> u32 {
        debug_assert!(drv.is_some(), "GetDriver()");
        if !(vs_abs(self.get_dist_along() - t.dist_along) <= 4.0 && t.race_section == self.race_section) {
            return 0;
        }
        let mut s = self.overtake_side;
        let ram_check = |me: &mut Self, s: &mut u32| {
            let (coll, ctime, _) = drv.map_or((VehicleRef::None, -1.0, 0.0), |d| d.get_collision());
            if coll != t.r || me.is_police() || t.is_police {
                *s = 0;
            } else {
                let dt = f.time - ctime;
                debug_assert!(vs_fge(dt, 0.0, 0.000001), "VsFge(dt, 0.0f)");
                if dt >= 0.25 {
                    me.set_offset_target(t.lateral, 100.0);
                } else {
                    *s = if t.lateral <= me.lateral { 2 } else { 1 };
                }
            }
        };
        match s {
            0 => {
                s = 3;
                ram_check(self, &mut s);
            }
            1 | 2 => {
                if vs_abs(t.lateral - self.lateral) <= 2.0 {
                    let off = if self.overtake_side == 1 { t.lateral + 2.5 } else { t.lateral - 2.5 };
                    self.set_offset_target(off, 100.0);
                } else {
                    s = 3;
                }
            }
            3 => ram_check(self, &mut s),
            _ => debug_assert!(false),
        }
        s
    }

    /// police: close in on the target; others: weave or block the car behind.
    /// VsAIRaceVehicle.cpp 0x00020080
    fn target_behaviour(&mut self, f: &mut RaceFrame, drv: Option<&VsAIDriver>, police_target: VehicleRef) {
        let target;
        if self.is_police() {
            debug_assert!(police_target != VehicleRef::None, "pTargetVehicle != NULL");
            target = f.scene.get(police_target).cloned();
            let Some(t) = target.clone() else { return };
            let gap = self.race_distance - t.race_distance;
            if self.police_mode == 0 {
                if gap <= 5.0 {
                    if gap > -30.0 {
                        if !self.avoiding {
                            self.set_offset_target(t.lateral, 100.0);
                        }
                        self.speed_match = true;
                        self.target_speed = t.speed + 100.0;
                    }
                } else {
                    if !self.avoiding {
                        // VsAIRaceVehicle.cpp 0x00024C90
                        let off = if t.lateral >= 0.0 { t.lateral - 8.0 } else { t.lateral + 8.0 };
                        self.set_offset_target(off, 50.0);
                    }
                    self.speed_match = true;
                    self.target_speed = 5.0;
                }
                return;
            }
            if self.police_mode == 1 {
                if gap <= 100.0 {
                    if gap > 5.0 {
                        self.speed_match = true;
                        self.target_speed = t.speed - 5.0;
                        if self.target_speed < 5.0 {
                            self.target_speed = 5.0;
                        }
                    }
                } else {
                    self.speed_match = true;
                    self.target_speed = 5.0;
                }
            }
        } else {
            target = f.scene.get(self.vehicle_behind).cloned();
        }
        let Some(t) = target else { return };
        self.draft_on_player = false;
        let mut s = 0;
        let gap = self.race_distance - t.race_distance;
        if self.rival_state == 1 {
            let o = self.rival_offset;
            self.set_offset_target(o, 20.0);
        } else {
            let temp = drv.map_or(0, |d| d.get_temperament());
            if (temp == 1 || temp == 3) && gap > 5.0 && gap < 20.0 {
                if t.is_player {
                    self.draft_on_player = true;
                }
                self.set_offset_target(t.lateral, 40.0);
            }
        }
        if drv.map_or(0, |d| d.get_temperament()) >= 1 {
            s = self.rival_behaviour(f, drv, &t);
        }
        self.overtake_side = s;
    }

    /// VsAIRaceVehicle.cpp 0x00020490
    fn is_stuck(&mut self, time: f32) -> bool {
        if self.failure != FAILURE_NONE {
            return false;
        }
        let st = self.state as u32;
        if st == 0 || st >= 7 {
            return false;
        }
        if !self.stuck && self.base.speed < 1.5 && self.base.speed > -1.5 && time > self.grace_until {
            self.stuck = true;
            self.stuck_time = time;
        }
        if !self.stuck || time - self.stuck_time <= 2.0 {
            return false;
        }
        self.stuck_pos = self.base.position;
        true
    }

    /// VsAIRaceVehicle.cpp 0x000205A0
    fn can_leave_recover(&mut self, time: f32) -> bool {
        let mut d = self.look_ahead[0].pos - self.base.position;
        d.normalise();
        let dot = self.base.forward.dot3(&d);
        if dot <= 0.9 || (self.base.speed <= 5.0 && self.base.speed >= -5.0) {
            return false;
        }
        self.stuck = false;
        self.grace_until = time + 4.0;
        true
    }

    /// VsAIRaceVehicle.cpp 0x000206A0
    fn recover_timed_out(&self, time: f32) -> bool {
        self.stuck && time - self.stuck_time > 9.0
    }

    /// falling fast or on the roof for three seconds: respawn.
    /// VsAIRaceVehicle.cpp 0x00020700
    fn check_upside_down(&mut self, f: &mut RaceFrame) {
        if self.phys_ref(f).upside_down_exempt() {
            self.upside_down = false;
            return;
        }
        let falling = self.base.velocity.y < -20.0;
        let up = self.phys_ref(f).body_up();
        let up_ok = VsVector4::xyz(up.x, up.y, up.z).dot3(&VsVector4::new(0.0, 1.0, 0.0, 0.0)) > f32::from_bits(0x3E99_999A);
        if self.upside_down {
            if falling || !up_ok {
                if f.time > self.upside_down_time {
                    self.respawn_on_raceline(f, 0.0);
                }
            } else {
                self.upside_down = false;
            }
        } else if falling || !up_ok {
            self.upside_down = true;
            self.upside_down_time = f.time + 3.0;
        }
    }

    /// puts the car back on the ground at its raceline point, facing along it.
    /// VsAIRaceVehicle.cpp 0x000208D0
    pub fn respawn_on_raceline(&mut self, f: &mut RaceFrame, speed: f32) {
        log::info!("{}: respawning on raceline", self.id);
        self.clear_failure(true, f.ctx.host);
        let have = self.race_section.is_some_and(|si| !f.world.race_sections()[si].raceline.points.is_empty());
        if have {
            let up = VsVector4::new(0.0, 1.0, 0.0, 1.0);
            let rl = self.raceline(f.world);
            let next = if self.raceline_seg == rl.get_num_points() - 1 { 0 } else { self.raceline_seg + 1 };
            let mut dir = *rl.get_point(next) - *rl.get_point(self.raceline_seg);
            dir.y = 0.0;
            dir.normalise();
            let z = VsVector4::new(0.0, 0.0, 1.0, 1.0);
            let mut heading = VsAITools::angle_between(&z, &dir);
            heading = (0.0 - heading) + 1.570_796_4;
            self.stuck = false;
            self.grace_until = f.time + 2.0;
            let base = self.get_raceline_point(f.world);
            let mut pos = VsVector4::default();
            let mut found = false;
            for k in 1..=5 {
                let s = k as f32;
                let below = base - (VS_UNIT_Y * 3.0) * s;
                let above = base + (VS_UNIT_Y * 3.0) * s;
                if let Some(hit) = VsAITools::line_check(&*f.ctx.physics, &above, &below) {
                    pos = hit;
                    // VE+0x810 - (body 0x0035FB40(0) - flt_684E78)
                    let (wheel, rest) = self.phys_ref(f).respawn_wheel_heights();
                    let drop = wheel - (rest - f32::from_bits(0x3DCC_CCCD));
                    log::debug!("wheel dy: {drop:.4}");
                    pos.y -= drop;
                    found = true;
                    break;
                }
            }
            if !found {
                log::error!("Error finding AI Race Vehicle spawn position");
            }
            self.teleport_heading(f, &pos, &up, heading, speed);
        } else {
            let up = VsVector4::new(0.0, 1.0, 0.0, 1.0);
            let grid = f.ctx.game.grid.first().copied().unwrap_or_default();
            self.teleport_heading(f, &grid, &up, 90.0, 0.0);
        }
        self.respawn_trigger_pending = 0.0;
        self.respawn_trigger = 0;
        self.respawn_time = 0.0;
    }

    /// back to the grid slot (race restart); with a speed the player keeps ai assistance.
    /// VsAIRaceVehicle.cpp 0x00020DF0
    pub fn reset_to_grid(&mut self, f: &mut RaceFrame, drv: Option<&mut VsAIDriver>, speed: f32) {
        let p = self.start_pos;
        let mut below = p;
        below.y -= 5.0;
        let mut above = p;
        above.y += 5.0;
        let pos = match VsAITools::line_check(&*f.ctx.physics, &above, &below) {
            Some(h) => h,
            None => {
                log::error!("Error finding AI Race Vehicle spawn position");
                VsVector4::new(0.0, 0.0, 0.0, 1.0)
            }
        };
        let o = self.start_orientation;
        self.respawn(f, &pos, &o, speed);
        if speed <= 0.0 {
            self.set_state(f, drv, RaceState::Grid);
        } else {
            self.set_state(f, drv, RaceState::PlayerAssist);
        }
        self.prev_state = self.state;
        self.base.active = true;
    }

    /// VsAITrafficVehicle.cpp 0x0002BAA0
    fn teleport_heading(&mut self, f: &mut RaceFrame, pos: &VsVector4, axis: &VsVector4, angle: f32, speed: f32) {
        let q = vs_quat_axis_angle(angle, axis);
        self.respawn(f, pos, &q, speed);
    }

    /// VsAITrafficVehicle.cpp 0x0002BB30
    pub fn respawn(&mut self, f: &mut RaceFrame, pos: &VsVector4, orientation: &VsVector4, speed: f32) {
        // VsAITrafficVehicle.cpp 0x0002B800
        self.phys(f).reset_dynamics();
        // 0x0002B800 tail: `if (id != -1) 0x0002C620(dword_6D55C8, id)`
        f.ctx.host.camera_cut(self.id);
        let v = self.phys(f);
        let zero = VsVector4::default();
        v.teleport(pos, orientation, &zero);
        // race manager car 0 (0x00015DB0(rm, 0)): crash camera and point of interest reset
        if self.id == 0 && !self.is_traffic {
            let h = self.base.physics.expect("physics vehicle");
            f.ctx.host.crash_camera_reset(h);
            f.ctx.host.camera_point_of_interest_clear(0.5);
        }
        if speed != 0.0 {
            let mut fwd = vs_quat_rotate(orientation, &VS_UNIT_X);
            fwd.normalise();
            fwd *= speed;
            let v = self.phys(f);
            v.set_linear_velocity(&fwd);
            v.set_angular_velocity(&zero);
            v.commit_body();
            v.reset_state();
        }
        // VehicleEntity.cpp 0x001D1810
        self.phys(f).reset_history_scale();
        if !self.is_traffic {
            self.set_race_section(f, 0, true);
            self.update_state(f, true);
        }
    }

    /// one car of a timeline respawn: the respawn point (trigger position, rotation x,y,z,w) probed 10 m up/down for
    /// the ground, placed 0.75 up, heading = euler y of the rotation + pi, then raced on from the first section.
    /// TimelineAI.cpp 0x00177150 (inner loop body)
    pub fn timeline_respawn_at(&mut self, f: &mut RaceFrame, point: &VsVector4, rot: &VsVector4, speed: f32) {
        let probe = VS_UNIT_Y * 10.0;
        let below = *point - probe;
        let above = *point + probe;
        let Some(hit) = VsAITools::line_check(&*f.ctx.physics, &above, &below) else {
            log::error!("Error finding AI Race Vehicle spawn position");
            return;
        };
        // TimelineSequencer.cpp 0x002B7A30: euler y = -(acos(-2 (x z - y w)) - pi/2)
        let (x, y, z, w) = (rot.x, rot.y, rot.z, rot.w);
        let e = (x * z - y * w) * -2.0;
        let ey = -(vs_acos(e) - 1.570_796_4);
        let angle = ey + f32::from_bits(0x4049_0FDB);
        let pos = hit + VS_UNIT_Y * 0.75;
        self.teleport_heading(f, &pos, &VS_UNIT_Y, angle, speed);
        self.set_race_section(f, 0, true);
        self.update_state(f, true);
    }

    /// edge tests: is the car between the road edges of its section.
    /// VsAIRaceVehicle.cpp 0x00021000
    fn check_on_route(&mut self, w: &VsAIWorld) {
        let si = self.race_section.unwrap();
        let s = &w.race_sections()[si];
        if s.left.points.is_empty() || s.right.points.is_empty() {
            return;
        }
        let p = self.base.position;
        let mut outside = false;
        let next = if self.left_seg == s.left.get_num_points() - 1 { 0 } else { self.left_seg + 1 };
        let (a, b) = (*s.left.get_point(self.left_seg), *s.left.get_point(next));
        self.left_of_left_edge = VsAITools::is_left_of_line(&a, &b, &p);
        let t = VsAITools::line_parameter(&a, &b, &p);
        if t < 0.0 || t > 1.0 {
            outside = true;
        }
        let next = if self.right_seg == s.right.get_num_points() - 1 { 0 } else { self.right_seg + 1 };
        let (a, b) = (*s.right.get_point(self.right_seg), *s.right.get_point(next));
        self.right_of_right_edge = !VsAITools::is_left_of_line(&a, &b, &p);
        let t = VsAITools::line_parameter(&a, &b, &p);
        if t < 0.0 || t > 1.0 {
            outside = true;
        }
        if si == 0 {
            outside = false;
        }
        self.on_route = self.left_of_left_edge && self.right_of_right_edge && !outside;
    }

    /// whole-race distance: completed laps plus this lap's main-route distance.
    /// VsAIRaceVehicle.cpp 0x000212C0
    fn track_race_distance(&self, w: &VsAIWorld) -> f32 {
        if self.race_section.is_none() || self.lap == 0 {
            return 0.0;
        }
        // VsAIRaceVehicle.cpp 0x00024DA0
        let v5 = self.track_distance;
        w.race_sections()[0].raceline.get_length() * (self.lap - 1) as f32 + v5
    }

    /// distance on the main route equivalent to the position on a side route.
    /// VsAIRaceVehicle.cpp 0x00021350
    fn section_track_distance(&self, w: &VsAIWorld) -> f32 {
        let Some(si) = self.race_section else { return 0.0 };
        if self.lap == 0 {
            return 0.0;
        }
        if si == 0 {
            return self.dist_along;
        }
        let s = &w.race_sections()[si];
        let r = self.dist_along / s.raceline.get_length();
        r * (s.track_end - s.track_start) + s.track_start
    }

    /// VsAIRaceVehicle.cpp 0x00021430
    fn get_raceline_point(&self, w: &mut VsAIWorld) -> VsVector4 {
        self.raceline_point(w, self.dist_along, self.raceline_seg)
    }

    /// point `dist` ahead along the route, following joins into later sections.
    /// VsAIRaceVehicle.cpp 0x000214B0
    pub fn point_ahead(&self, w: &mut VsAIWorld, dist: f32) -> VsVector4 {
        let mut d = self.dist_along + dist;
        let mut s = self.race_section.unwrap();
        if s != 0 {
            loop {
                let len = w.race_sections()[s].raceline.get_length();
                if d <= len {
                    break;
                }
                let rem = d - len;
                let sec = &w.race_sections()[s];
                debug_assert!(!sec.sub_route_points.is_empty(), "pTestSection->m_dNumSubRoutePoints > 0");
                let Some(pi) = sec.get_sub_route_point(sec.sub_route_points.len() as u32 - 1) else {
                    log::error!("*** ERROR! Couldn't find sub-route point ***");
                    break;
                };
                let p = &w.sub_route_points[pi];
                s = p.to;
                d = p.dist + rem;
                if s == 0 {
                    d = w.race_sections()[0].raceline.wrap_distance(d);
                }
            }
        }
        let mut idx = 0;
        w.race_sections_mut()[s].raceline.get_point_at_distance(d, 0, &mut idx)
    }

    /// signed xz distance of `p` from segment `seg` of `line` (+ left); -1 past an open end.
    /// VsAIRaceVehicle.cpp 0x00021740
    fn lateral_offset(&self, p: &VsVector4, seg: u32, line: &VsAIRoadLine) -> f32 {
        let (mut a, mut b) = if seg == line.get_num_points() - 1 {
            if !line.is_circular() {
                return -1.0;
            }
            (*line.get_point(seg), *line.get_point(0))
        } else {
            (*line.get_point(seg), *line.get_point(seg + 1))
        };
        let mut q = *p;
        q.y = 0.0;
        b.y = 0.0;
        a.y = 0.0;
        let d = VsAITools::distance_to_line(&a, &b, &q);
        if VsAITools::is_left_of_line(&a, &b, &q) { d } else { -d }
    }

    /// VsAIRaceVehicle.cpp 0x000218F0
    fn check_near_finish(&mut self, g: &super::ai_context::AiGameState) {
        if vs_distance_squared(&self.base.position, &g.finish_point) < 900.0 {
            self.flag_0fc = true;
        }
    }

    /// VsAIRaceVehicle.cpp 0x00021940
    pub fn reset_dist(&mut self) {
        self.raceline_seg = 0;
        self.dist_along = 0.0;
    }

    /// records the race clock at checkpoint `n`.
    /// VsAIRaceVehicle.cpp 0x00021980
    pub fn set_checkpoint(&mut self, n: u32, race_clock: f32) {
        self.checkpoint = n;
        if n != 0 {
            set_stat_f(&mut self.stats, 3 + n as usize, race_clock);
        }
    }

    /// after the line: a slow handbrake doughnut, else brake to a stop.
    /// VsAIRaceVehicle.cpp 0x00021A30
    fn finished_drive(&mut self, f: &mut RaceFrame) {
        if self.base.speed <= params::BURNOUT_SPEED {
            self.phys(f).set_steer_angle(1.0);
            let c = self.controls(f);
            c.handbrake = 1.0;
            c.throttle = 0.0;
            c.brake = 0.0;
        } else {
            let c = self.controls(f);
            c.throttle = 0.0;
            c.brake = 1.0;
            c.handbrake = 1.0;
            self.phys(f).set_steer_angle(f32::from_bits(0xBC23_D70A));
        }
    }

    /// -1 left of the left edge, 1 right of the right edge, else 0; optionally clamps onto the road.
    /// VsAIRaceVehicle.cpp 0x00021B00
    fn classify_side(&self, w: &VsAIWorld, si: usize, p: &mut VsVector4, clamp: bool) -> i32 {
        let s = &w.race_sections()[si];
        let ls = s.left.find_segment(p, self.left_seg);
        let rs = s.right.find_segment(p, self.right_seg);
        let (a, b) = s.left.get_segment(ls);
        let (c, d) = s.right.get_segment(rs);
        let mut beyond_left = false;
        let mut beyond_right = false;
        let r = if VsAITools::is_left_of_line(&a, &b, p) {
            if VsAITools::is_left_of_line(&c, &d, p) {
                beyond_right = true;
                1
            } else {
                0
            }
        } else {
            beyond_left = true;
            -1
        };
        if clamp {
            if beyond_left {
                *p = VsAITools::project_onto_line(&a, &b, p);
            } else if beyond_right {
                *p = VsAITools::project_onto_line(&c, &d, p);
            }
        }
        r
    }

    /// the three raceline points the car aims at.
    /// VsAIRaceVehicle.cpp 0x00021DD0
    pub fn compute_look_ahead(&mut self, w: &mut VsAIWorld, dist: f32) {
        let si = self.race_section.expect("m_pCurrentRaceSection");
        for i in 0..3u32 {
            let scale = if i != 0 {
                (i + 1) as f32 + 1.0
            } else if dist <= 25.0 {
                1.0
            } else {
                25.0 / dist
            };
            let mut d = self.get_dist_along() + dist * scale;
            let rl = &w.race_sections()[si].raceline;
            if rl.is_circular() {
                if d > rl.get_length() {
                    d -= rl.get_length();
                }
                if d < 0.0 {
                    d += rl.get_length();
                }
            } else {
                if d > rl.get_length() {
                    d = rl.get_length();
                }
                if d < 0.0 {
                    d = 0.0;
                }
            }
            self.look_ahead[i as usize].dist = d;
            let mut idx = 0;
            self.look_ahead[i as usize].pos = w.race_sections_mut()[si].raceline.get_point_at_distance(d, self.raceline_seg, &mut idx);
        }
    }

    /// VsAIRaceVehicle.cpp 0x00022000
    fn set_offset_target(&mut self, target: f32, rate: f32) {
        if !self.offset_active {
            self.offset_active = true;
            self.offset_current = self.lateral;
        }
        self.offset_rate = rate;
        self.offset_target = target;
    }

    /// VsAIRaceVehicle.cpp 0x00022070
    fn update_offset(&mut self, dt: f32) {
        if !self.offset_active {
            return;
        }
        let step = dt * self.offset_rate;
        if self.offset_current <= self.offset_target {
            if self.offset_target > self.offset_current {
                self.offset_current += step;
                if self.offset_current > self.offset_target {
                    self.offset_current = self.offset_target;
                }
            }
        } else {
            self.offset_current -= step;
            if self.offset_target > self.offset_current {
                self.offset_current = self.offset_target;
            }
        }
    }

    /// shifts the lookahead points sideways by the wanted offset, kept 2m inside the road edges.
    /// VsAIRaceVehicle.cpp 0x00022190
    fn apply_lateral_offset(&mut self, f: &mut RaceFrame) {
        let dt = f.world.time_step;
        self.update_offset(dt);
        if f.world.globals.force_follow_raceline {
            f.world.globals.raceline_offset_scale = 0.0;
        }
        let m = params::EDGE_MARGIN;
        self.near_edge = false;
        if self.lateral <= 0.0 {
            if m * m >= self.edge_left_sq {
                self.near_edge = true;
            }
        } else if m * m >= self.edge_right_sq {
            self.near_edge = true;
        }
        let si = self.race_section.unwrap();
        for i in 0..3 {
            let la = self.look_ahead[i].pos;
            let (seg, a, b) = {
                let rl = &f.world.race_sections()[si].raceline;
                let seg = rl.find_segment(&la, self.raceline_seg);
                let (a, b) = rl.get_segment(seg);
                (seg, a, b)
            };
            let mut dir = b - a;
            dir.normalise();
            let mut off = if self.offset_active { self.offset_current } else { self.lateral * f.world.globals.raceline_offset_scale };
            let perp = if off >= 0.0 {
                VsVector4::new(0.0 - dir.z, dir.y, dir.x, 0.0)
            } else {
                VsVector4::new(dir.z, dir.y, 0.0 - dir.x, 0.0)
            };
            off = vs_abs(off);
            let mut p = perp;
            p *= off;
            p += la;
            let left_side = VsAITools::is_left_of_line(&a, &b, &p);
            let (eseg_start, use_right) = if left_side { (self.right_seg, true) } else { (self.left_seg, false) };
            let (eseg, c, d) = {
                let s = &f.world.race_sections()[si];
                let edge = if use_right { &s.right } else { &s.left };
                let eseg = edge.find_segment(&p, eseg_start);
                let (c, d) = edge.get_segment(eseg);
                (eseg, c, d)
            };
            let e_left = VsAITools::is_left_of_line(&c, &d, &p);
            if (left_side && e_left) || (!left_side && !e_left) || m > VsAITools::distance_to_line(&c, &d, &p) {
                let (rd, ed) = {
                    let s = &f.world.race_sections()[si];
                    let edge = if use_right { &s.right } else { &s.left };
                    (s.raceline.get_distance_along_line(&p, seg), edge.get_distance_along_line(&la, eseg))
                };
                let s = &mut f.world.race_sections_mut()[si];
                let mut idx = 0;
                let rp = s.raceline.get_point_at_distance(rd, seg, &mut idx);
                let edge = if use_right { &mut s.right } else { &mut s.left };
                let ep = edge.get_point_at_distance(ed, eseg, &mut idx);
                off = (ep - rp).length() - m;
                if off < 0.0 {
                    off = 0.0;
                }
                p = perp;
                p *= off;
                p += la;
            }
            self.look_ahead[i].pos = p;
        }
    }

    /// a fast car slipstreaming close behind may make an aggressive driver start weaving.
    /// VsAIRaceVehicle.cpp 0x00022890
    fn is_rival_candidate(&self, f: &RaceFrame, drv: Option<&VsAIDriver>) -> bool {
        if self.phys_ref(f).is_airborne() {
            return false;
        }
        if drv.map_or(0.0, |d| d.get_aggression()) < f32::from_bits(0x3F59_999A) {
            return false;
        }
        let Some(b) = f.scene.get(self.vehicle_behind) else { return false };
        let Some(bh) = b.physics else { return false };
        if f.ctx.physics.slipstream_source(bh) != self.base.physics {
            return false;
        }
        let gap = self.get_dist_along() - b.dist_along;
        gap >= 0.0 && gap <= 15.0
    }

    /// VsAIRaceVehicle.cpp 0x00022970
    fn update_rival(&mut self, f: &mut RaceFrame, drv: Option<&VsAIDriver>) {
        if self.is_player {
            return;
        }
        if self.is_rival_candidate(f, drv) {
            match self.rival_state {
                0 => {
                    let r = f.ctx.rand.unit();
                    self.rival_next = r + r + f.time;
                    self.rival_state = 1;
                }
                1 => {
                    // original: the offset is re-picked while the timer is still running
                    if self.rival_next >= f.time {
                        let l = vs_sqrt(self.edge_left_sq);
                        let r = vs_sqrt(self.edge_right_sq);
                        let w = l + r;
                        let lat = self.lateral;
                        for _ in 0..16 {
                            self.rival_offset = f.ctx.rand.unit() * w - l;
                            if w < 12.0 || vs_abs(lat - self.rival_offset) > 8.0 {
                                break;
                            }
                        }
                        self.rival_next = f.ctx.rand.unit() * 5.0 + f.time;
                    }
                }
                _ => debug_assert!(false),
            }
        } else {
            self.rival_state = 0;
            self.rival_offset = 0.0;
        }
    }

    /// VsAIRaceVehicle.cpp 0x00022C00
    fn choose_failure(&self, f: &mut RaceFrame) -> u32 {
        let r = f.ctx.rand.unit();
        let dt = f.time - self.nos_stop;
        if (self.phys_ref(f).nos_burning() || dt < 5.0) && r < f32::from_bits(0x3F8C_CCCD) {
            1
        } else {
            ((f.ctx.rand.rand() >> 8) & 3) as u32 + 2
        }
    }

    /// a respawn trigger was hit: crash out now, respawn after `delay`.
    /// VsAIRaceVehicle.cpp 0x00022C90
    pub fn schedule_respawn(&mut self, f: &mut RaceFrame, drv: Option<&mut VsAIDriver>, trigger: u32, delay: f32) {
        debug_assert!(trigger != 0, "respawnTriggerCRC");
        debug_assert!(delay >= 0.0, "delay >= 0.0f");
        let t = if self.steer_angle <= 0.0 { 2 } else { 5 };
        self.respawn_trigger = trigger;
        self.respawn_trigger_pending = f.time + delay;
        // DangerZoneRender 0x00123070 on g_FadeRender (flt_7190FC)
        f.ctx.host.fade_start(delay);
        self.trigger_failure(f, drv, t);
    }

    /// damage that crosses the car's limit makes something break.
    /// VsAIRaceVehicle.cpp 0x00022FE0
    pub fn apply_damage(&mut self, f: &mut RaceFrame, drv: Option<&mut VsAIDriver>, amount: f32, failure: u32) -> bool {
        if f.ctx.game.network {
            return false;
        }
        if self.is_player && f.ctx.event.mode != 5 {
            return false;
        }
        if !self.base.apply_damage(amount) || self.finished_flag || !f.world.globals.failures_enabled {
            return false;
        }
        let t = if failure == FAILURE_NONE { self.choose_failure(f) } else { failure };
        self.trigger_failure(f, drv, t);
        log::info!("{}: chose failure {}", self.id, t);
        true
    }

    /// the post-race and normal per-frame update.
    /// VsAIRaceVehicle.cpp 0x00018A40
    pub fn update(&mut self, f: &mut RaceFrame, mut drv: Option<&mut VsAIDriver>, respawns: &mut dyn RespawnSource, dt: f32) {
        match self.post_race_state {
            0 => {}
            1 => {
                let c = self.controls(f);
                c.brake = 0.0;
                c.throttle = 0.0;
                self.phys(f).set_steer_angle(0.0);
                self.phys(f).lock_wheels(1);
                return;
            }
            2 => return,
            3 => {
                let h = self.base.physics.unwrap();
                self.base.deactivate(Some(f.ctx.physics.vehicle(h)));
                return;
            }
            _ => debug_assert!(false, "ERROR: VsAIRaceVehicle::Update - invalid state for m_ePostRaceState"),
        }
        if self.trail_blazer.is_some() {
            self.update_trail_blazer(f, drv.as_deref_mut(), respawns, dt);
        }
        if !self.is_player && !self.is_police() {
            let rpm = self.phys_ref(f).engine_rpm();
            let g = f.world.globals.clone();
            self.startline_revs.update(&g, f.ctx.game.timer_time, f.ctx.game.countdown_end, rpm);
        }
        let time = f.time;
        if self.respawn_time > 0.0 && time >= self.respawn_time {
            if !self.human_control {
                self.set_human_control(true, f);
                // OSDManager 0x000ED6E0(0) on dword_6FD200, then its +0x24 mask &= ~1 (0x00023D00)
                f.ctx.host.cinematic_bars(false);
                f.ctx.host.osd_clear_flags(1);
            }
            self.respawn_time = 0.0;
        }
        if self.respawn_trigger_pending <= 0.0 || time < self.respawn_trigger_pending {
            if self.scheduled_failure != FAILURE_NONE && time > self.scheduled_failure_time {
                let s = self.scheduled_failure;
                self.trigger_failure(f, drv.as_deref_mut(), s);
                self.scheduled_failure = FAILURE_NONE;
                self.scheduled_failure_time = 0.0;
            }
            let forced = f.world.globals.force_failure;
            if !self.is_player && forced != FAILURE_NONE && self.failure == FAILURE_NONE {
                self.trigger_failure(f, drv.as_deref_mut(), forced);
            }
            self.base.debug_selected = f.world.debug_vehicle_id == self.id;
            self.update_score(f);
            self.update_oil_spill(f);
            // VsAIRaceVehicle.cpp 0x00022FA0
            if self.failure != FAILURE_NONE {
                let c = self.controls(f);
                c.throttle = 0.0;
                c.brake = 0.0;
                return;
            }
            if self.human_control || self.phys_ref(f).control_mode() == 4 {
                if self.failure == 0 {
                    self.controls(f).throttle = 0.0;
                }
                self.find_current_section(f);
                return;
            }
            self.update_race_state(f, drv.as_deref_mut());
            self.state_update(f, drv.as_deref_mut());
            if f.scene.player0().is_some_and(|p| p.started) {
                if f.world.globals.apply_slowdown && f.ctx.game.difficulty_mode != 2 {
                    self.apply_slowdown(f);
                }
                self.catchup_boost(f, drv.as_deref());
            }
            if f.ctx.game.race_phase >= 4 {
                self.update_rival(f, drv.as_deref());
                if f.world.globals.make_mistakes {
                    self.apply_mistake(f, drv.as_deref());
                }
            } else {
                self.controls(f).handbrake = 1.0;
            }
            self.speed_match = false;
        } else {
            self.respawn_time = 0.0;
            log::info!("{}: Scheduled respawn with full health", self.id);
            self.respawn_trigger_pending = 0.0;
            if self.respawn_trigger != 0 {
                if let Some(r) = respawns.respawn_point(self.respawn_trigger) {
                    let delay = r.delay_ms as f64 / 1000.0;
                    let mut speed = r.speed as f32;
                    if speed == 0.0 {
                        speed = 50.0;
                    }
                    if vs_fge(delay as f32, 0.0, 0.000001) {
                        self.set_human_control(false, f);
                        self.respawn_time = time + delay as f32;
                        f.ctx.host.fade_start(0.0 - delay as f32);
                    }
                    self.respawn(f, &r.pos, &r.orientation, speed);
                } else {
                    debug_assert!(false, "trigger");
                }
                self.respawn_trigger = 0;
            }
            self.clear_failure(true, f.ctx.host);
        }
    }

    /// the score manager's part of `Update` (0x00018A40): in race phases 4/5 ScoreManager 0x00169E30 runs on the
    /// race clock (never "in the zone" from here); 6/7 only fade its OSD gap items (0x00169940, not ported). The
    /// physics car reads the record's zone / nitrous gains (+0xA0/+0xA4) in `ComputeForcesAndTorques`.
    fn update_score(&mut self, f: &mut RaceFrame) {
        debug_assert!(self.score.is_some(), "m_pScoreManager");
        let phase = f.ctx.game.race_phase;
        if (phase == 4 || phase == 5) && self.score.is_some() {
            let i = self.score_inputs(f);
            let mut out = Vec::new();
            self.stats_to_score();
            self.score.as_mut().unwrap().update(&i, false, &mut out);
            self.stats_from_score();
            for e in out {
                f.ctx.host.score_event(self.id, e);
            }
            // 0x00169E30 clears the scored gear-switch result (0x0016AFE0) inside its "racing" branch
            if i.racing && i.race_switch != 0 {
                let h = self.base.physics.expect("m_pPhysicsVehicle");
                f.ctx.physics.vehicle(h).clear_shift_result();
            }
        }
        // VsRaceManager__3D640 finds only race manager cars: the police race part has no record
        if let (Some(s), Some(h), false) = (self.score.as_ref(), self.base.physics, self.is_police()) {
            let gains = (s.zone, s.nos);
            f.ctx.physics.vehicle(h).set_score_gains(Some(gains));
        }
    }

    /// what ScoreManager reads from this car and the race (see `ScoreInputs` for each field's origin).
    fn score_inputs(&self, f: &RaceFrame) -> ScoreInputs {
        use bevy::math::Vec3;
        let v3 = |v: &VsVector4| Vec3::new(v.x, v.y, v.z);
        let h = self.base.physics.expect("m_pPhysicsVehicle");
        let phys = f.ctx.physics.vehicle_ref(h);
        let raceline_dir = self.race_section.and_then(|si| f.world.race_sections().get(si)).map(|s| {
            let (a, b) = s.raceline.get_segment(self.raceline_seg);
            Vec3::new(b.x - a.x, b.y - a.y, b.z - a.z)
        });
        let split = self.checkpoint;
        let ahead = f.scene.get(self.vehicle_ahead).map(|o| {
            let t = if (1..=5).contains(&split) { stat_f(&o.stats, 3 + split as usize) } else { 0.0 };
            (o.id, t)
        });
        let behind = f.scene.get(self.vehicle_behind).map(|o| (o.id, o.race_distance));
        ScoreInputs {
            time: f.ctx.game.race_clock,
            racing: self.started,
            position: stat_position(&self.stats),
            lap: self.lap as u8,
            speed: self.base.speed,
            velocity: v3(&self.base.velocity),
            forward: v3(&self.base.forward),
            wheel_normal: v3(&phys.wheel_contact_normal(1)),
            handbrake: phys.controls_ref().handbrake,
            // LightStreak 0x00130070 (render side) is not exposed: no slipstream score
            slipstream: 0.0,
            raceline_offset: self.lateral,
            raceline_dir,
            // ScoreManagerChecks 0x0016CAB0 / 0x0016CBD0 -> 0x0016CD50 / 0x0016CD70 on the car's physics vehicle
            start_switch: phys.launch_result() as u32,
            race_switch: phys.shift_result() as u32,
            // the lap records of VsRaceManager's lap end (stats [3] = the lap just run, [11] = best)
            last_lap_time: stat_f(&self.stats, 3),
            best_lap_time: stat_f(&self.stats, 11),
            position_vec: v3(&self.base.position),
            traffic: f.scene.traffic.iter().filter(|t| t.active).map(|t| (t.id, v3(&t.position))).collect(),
            // the police manager's target is not in the scene snapshot
            chased_by_police: false,
            police: f.scene.police.iter().filter(|p| p.active).map(|p| v3(&p.position)).collect(),
            split,
            on_section: self.race_section.is_some(),
            is_player: self.is_player,
            ahead,
            behind,
            distance: self.race_distance,
        }
    }

    /// leaders ease off when far ahead of the player.
    /// VsAIRaceVehicle.cpp 0x00019370
    fn apply_slowdown(&mut self, f: &mut RaceFrame) {
        let Some(p0) = f.scene.player0() else { return };
        let gap = self.race_distance - p0.race_distance;
        let (min_dist, max_dist, max_accel) =
            (f.world.globals.slowdown_min_dist, f.world.globals.slowdown_max_dist, f.world.globals.slowdown_max_accel);
        if gap > min_dist {
            let mut t = (gap - min_dist) / (max_dist - min_dist);
            if t > 1.0 {
                t = 1.0;
            }
            let mut s = t * max_accel + (1.0 - t);
            if max_accel > s {
                s = max_accel;
            }
            self.controls(f).throttle *= s;
        }
    }

    /// stragglers more than 25m behind the player get more throttle.
    /// VsAIRaceVehicle.cpp 0x00019480
    fn catchup_boost(&mut self, f: &mut RaceFrame, drv: Option<&VsAIDriver>) {
        let Some(p0) = f.scene.player0() else { return };
        let gap = p0.race_distance - self.race_distance;
        if gap > 25.0 {
            let mut skill = drv.map_or(1, |d| d.get_skill_level(self));
            // police drivers have no opponent: the original reads an unset pointer (+0x04, never written by
            // VsPoliceManager 0x00038C10/0x00038D30); they get the no-driver default instead of indexing past the table
            if skill == super::vs_ai_driver::AIA_NOT_SET {
                skill = 1;
            }
            if skill == 3 {
                skill = 2;
            }
            let b = f.world.globals.catchup_boost[skill as usize];
            self.controls(f).throttle *= b;
        }
    }

    /// VsAIRaceVehicle.cpp 0x00019540
    fn apply_mistake(&mut self, f: &mut RaceFrame, drv: Option<&VsAIDriver>) {
        let g = f.world.globals.clone();
        match drv.map_or(super::vs_ai_driver::MISTAKE_NONE, |d| d.mistake) {
            0 => {
                self.steer_angle *= g.brake_late_steering_multiplier;
                let a = self.steer_angle;
                self.phys(f).set_steer_angle(a);
                self.controls(f).brake *= g.brake_late_brake_multiplier;
            }
            1 => {
                let c = self.controls(f);
                c.handbrake = g.skid_out_handbrake_strength;
                c.steer = if c.steer >= 0.0 { 1.0 } else { -1.0 };
                self.phys(f).set_grip_scale(f32::from_bits(0x3E80_0000));
            }
            2 => {
                self.lane_offset = if self.edge_right_sq <= self.edge_left_sq { g.misalign_amount } else { 0.0 - g.misalign_amount };
            }
            3 => {
                self.controls(f).handbrake = 0.0;
                self.lane_offset = 0.0;
                self.phys(f).set_grip_scale(1.0);
            }
            _ => debug_assert!(false),
        }
    }
}

/// a respawn point from the level's triggers (TriggerUpdate.cpp 0x00181AB0).
#[derive(Debug, Clone, Copy, Default)]
pub struct RespawnPoint {
    pub pos: VsVector4,         // +0x00
    pub orientation: VsVector4, // +0x10
    pub delay_ms: u32,          // +0x40
    pub speed: u32,             // +0x44
    /// +0x48 the trailblazer's respawn delay in ms (TrailBlazer.cpp 0x000C6220)
    pub param_2: u32,
}

/// a trailblazer road line as `TrailSpline` (the line caches its last point, hence the cell)
pub struct TrailLine<'a>(pub std::cell::RefCell<&'a mut VsAIRoadLine>);

fn trail_v4(p: bevy::math::Vec3) -> VsVector4 {
    VsVector4::new(p.x, p.y, p.z, 1.0)
}

impl TrailSpline for TrailLine<'_> {
    fn find_closest_segment(&self, p: bevy::math::Vec3) -> u32 {
        self.0.borrow().find_closest_segment(&trail_v4(p))
    }
    fn find_segment(&self, p: bevy::math::Vec3, hint: u32) -> u32 {
        self.0.borrow().find_segment(&trail_v4(p), hint)
    }
    fn distance_along(&self, p: bevy::math::Vec3, seg: u32) -> f32 {
        self.0.borrow().get_distance_along_line(&trail_v4(p), seg)
    }
    fn length(&self) -> f32 {
        self.0.borrow().get_length()
    }
    fn point_at(&self, d: f32, seg: u32) -> bevy::math::Vec3 {
        let mut o = 0;
        let v = self.0.borrow_mut().get_point_at_distance(d, seg, &mut o);
        bevy::math::Vec3::new(v.x, v.y, v.z)
    }
}

/// looks up respawn triggers by crc (owned by velocity_game).
pub trait RespawnSource {
    fn respawn_point(&mut self, crc: u32) -> Option<RespawnPoint>;
}

/// no level triggers.
pub struct NoRespawns;
impl RespawnSource for NoRespawns {
    fn respawn_point(&mut self, _crc: u32) -> Option<RespawnPoint> {
        None
    }
}
