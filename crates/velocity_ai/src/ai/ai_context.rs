//! What the ai needs from the rest of the game each frame.
//!
//! The original reaches these through globals (`g_GameManager` 0x006F5BC8, the event manager
//! 0x006F9BE8, the physics singleton 0x0074BA70, the frame timer 0x0074A430, the start grid
//! 0x006F5C0C...). The port passes them explicitly; owners of those systems implement the traits.

use super::vs_ai_tools::AiCollisionQuery;
use super::vs_ai_vehicle::AiPhysicsVehicle;
use super::vs_math::{VsRand, VsVector4};

/// the physics cars and the collision queries the ai uses.
pub trait AiPhysicsWorld: AiCollisionQuery {
    fn vehicle(&mut self, handle: usize) -> &mut dyn AiPhysicsVehicle;
    fn vehicle_ref(&self, handle: usize) -> &dyn AiPhysicsVehicle;
    /// the car whose slipstream `handle` is in (VehicleEntity.cpp 0x001BAEA0)
    fn slipstream_source(&self, _handle: usize) -> Option<usize> {
        None
    }
    /// steering angle that points the front wheels at `target` (`body 0x003717D0`)
    fn steer_angle_to(&self, handle: usize, target: &VsVector4) -> f32;
    /// glue: a physics world exists (the ai only touches cars once it does)
    fn ready(&self) -> bool {
        true
    }
    /// glue: a physics vehicle with this id exists
    fn has_vehicle(&self, _handle: usize) -> bool {
        true
    }
    /// glue, after `VsAIWorld::Update`: who drives each bound race car (pad or the controls block,
    /// VehicleManager__1DD9F0) and the controls blocks into the cars before the physics frame
    fn end_update(&mut self, _human_control: &[(usize, bool)]) {}
    /// glue, before the ai touches the cars: re-read every controls block (VE+0x9C), which the original
    /// ai addresses in place
    fn begin_access(&mut self) {}
    /// glue, after the ai touched the cars: the controls blocks it changed back into the cars
    fn end_access(&mut self) {}
    /// the physics globals TrailBlazer.cpp 0x000C4870 saves: `GC->visual_zone_velocity`,
    /// `zone_sensitivities_multiplier`, `SS_friction_limited_SA_multiplier`, `SS_steering_bias_when_steering`,
    /// `SS_steering_bias`, `dword_686B58`
    fn trail_blazer_globals(&self) -> [f32; 6] {
        [0.0; 6]
    }
    /// PhysicsObjectManager 0x001E18E0 (`VsAIRaceVehicle__24F90()+24`): ground ray at `pos`, on a hit a friction
    /// zone and a puddle (VsRandRange x2 on the shared `rand`); true if the ray hit
    #[allow(clippy::too_many_arguments)]
    fn friction_spot(&mut self, _pos: &VsVector4, _friction: f32, _radius: f32, _race_distance: f32, _ignore_car: i32, _expiry: f32, _rand: &mut VsRand) -> bool {
        false
    }
}

/// the event being raced (`EventDef`, reached through 0x006F9BE8 `+0x20 +0x0C`).
#[derive(Debug, Clone, Default)]
pub struct AiEventInfo {
    pub mode: u32,          // +0x04 1 time trial, 5 ..., 6 police chase
    pub num_laps: u8,       // +0x14
    pub num_opponents: u8,  // +0x15
    /// event `+0x10 -> +0x1C` / `+0x20`: copied into race stats 11 and 20 on reset
    pub stat_11_init: f32,
    pub stat_20_init: f32,
}

/// per-frame game state the ai reads.
#[derive(Debug, Clone, Default)]
pub struct AiGameState {
    /// `g_GameManager +0x24` (`dword_6F5BEC`): race phase, >= 4 once the race runs
    pub race_phase: u32,
    /// `g_GameManager +0x208`: race clock
    pub race_clock: f32,
    /// `byte_6F7158`: network race
    pub network: bool,
    /// `dword_6F71A0`: player prestige
    pub player_prestige: u32,
    /// `dword_6F70EC`
    pub difficulty_mode: u32,
    /// frame timer time (VsAIRaceVehicle.cpp 0x00024390 on 0x0074A430)
    pub timer_time: f32,
    /// countdown: `0x006F5BD4 +0xCD8` start of the green light and `+0xCDC` its end
    pub countdown_start: f32,
    pub countdown_end: f32,
    /// start/finish line points (`dword_6F5C44`, `dword_6F5C54`)
    pub start_line: Option<(VsVector4, VsVector4)>,
    /// the untouched start line copy the start detection uses (`dword_6F5C24`, `dword_6F5C34`)
    pub start_line_original: Option<(VsVector4, VsVector4)>,
    /// `byte_6F915E`: in-race speech on
    pub speech_enabled: bool,
    /// start grid (`dword_6F5C0C`, 16 bytes per slot) and grid heading (`dword_6F5C14`)
    pub grid: Vec<VsVector4>,
    pub grid_heading: VsVector4,
    /// finish point the 30m arrival test uses (`dword_6F5C64`)
    pub finish_point: VsVector4,
}

/// notifications the ai sends to render/audio/osd code; all optional.
pub trait AiHost {
    /// a driver says a TFT line (`VsAIPlayerTFTManager::PlayLine`); true when it played
    fn play_tft(&mut self, _name: &str, _player: i32, _volume: f32) -> bool {
        false
    }
    /// blown engine smoke on/off (CarRender.cpp 0x00116A10 / 0x00116A70)
    fn engine_smoke(&mut self, _vehicle_id: u32, _on: bool) {}
    /// oil spill dropped (PhysicsObjectManager.cpp 0x001E18E0)
    /// a traffic blocker's physics object stops (true) or starts colliding (`object +0x98` bit 2)
    fn traffic_blocker_object(&mut self, _object: u32, _disabled: bool) {}
    /// a traffic car's hazard warning on/off (DangerZoneRender.cpp 0x00123930, 4.0 or 0)
    fn traffic_hazard(&mut self, _vehicle_id: u32, _on: bool) {}
    /// a traffic car was placed: new paint and render setup (CarPalette.cpp 0x0010CCB0, BarrierRender 0x0010CBD0)
    fn traffic_spawned(&mut self, _vehicle_id: u32) {}
    /// per-frame hook of a trigger-spawned traffic car (TriggerUpdate.cpp 0x00181920)
    fn traffic_trigger_update(&mut self, _vehicle_id: u32) {}
    /// a police siren light (DangerZoneRender.cpp 0x00123970)
    #[allow(clippy::too_many_arguments)]
    fn police_light(&mut self, _vehicle_id: u32, _light: u32, _on: bool, _intensity: f32, _a: u8, _r: u8, _g: u8, _b: u8) {}
    /// the screen-edge police flash colour, zero = off (OSD3dInterface.cpp 0x000EAE90)
    fn police_flash(&mut self, _colour: &VsVector4) {}
    /// point the follow camera at a physics car (VsVehicleFollowCameraPointOfInterest.cpp 0x00069BE0)
    fn camera_point_of_interest(&mut self, _physics: usize, _weight: f32) {}
    /// the same call with no vehicle (`69BE0(&unk_6D60BC, 0, 0, priority)`): drops the point of interest
    fn camera_point_of_interest_clear(&mut self, _priority: f32) {}
    /// `g_CrashCameraManager` (0x006D58B0) reset for race car 0 after a respawn (VsCrashCameraManager.cpp
    /// 0x00063BA0: crash camera of the car's pad slot `+0x78` when < 4, VsCrashCamera 0x000623D0)
    fn crash_camera_reset(&mut self, _physics: usize) {}
    /// VsAIVehicle.cpp 0x0002C620 on `dword_6D55C8` (camera manager): player record `+44*id+40` = 1 (follow camera
    /// cut) and MessageManager message 3 (CameraCut); `id` = the race car's `+0x78`
    fn camera_cut(&mut self, _id: u32) {}
    /// a physics car is in view of the player camera (VsVehicleFollowCameraPointOfInterest.cpp 0x0006A4E0)
    fn in_camera_view(&self, _physics: usize) -> bool {
        false
    }
    /// a score manager's OSD message / audio cue / session best lap (ScoreManager 0x00169E30), by race car id
    fn score_event(&mut self, _vehicle_id: u32, _e: velocity_game::score_manager::score_manager::ScoreEvent) {}
    /// `g_FadeRender.Start(t)` (DangerZoneRender 0x00123070 on 0x007190FC), one global fade
    fn fade_start(&mut self, _t: f32) {}
    /// OSDManager 0x000ED6E0(show) on the game OSD manager (`dword_6FD200`): cinematic bars
    fn cinematic_bars(&mut self, _on: bool) {}
    /// 0x00023D00 on the game OSD manager: `+0x24 &= ~mask`
    fn osd_clear_flags(&mut self, _mask: u32) {}
    /// the camera manager's (`dword_6D55C8`) player record camera (Armco 0x00013F70 `+44*p+36`): position +212
    /// (VsAIRaceVehicle 0x00023E00) and direction +280 (0x00023EB0)
    fn camera(&self, _player: usize) -> Option<(VsVector4, VsVector4)> {
        None
    }
    /// translation of car data +64 of a physics car's render car (VsAIRaceVehicle 0x00023F40 / 0x00023DD0): the
    /// render matrix +0 copied with the engine shake added (CarRender 0x0011ABB0), world space
    fn car_render_point(&self, _physics: usize) -> Option<VsVector4> {
        None
    }
    /// FlareRender 0x001270E0 on `unk_719450`: a flash record
    fn flare_flash(&mut self, _rec: &AiFlashRecord) {}
}

/// the 36-byte flash record FlareRender 0x001270E0 copies (VsAIRaceVehicle 0x00023F70 builds it)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AiFlashRecord {
    pub rise: f32,      // +0
    pub fall: f32,      // +4
    pub intensity: f32, // +8
    pub screen: bool,   // +12
    pub pos: VsVector4, // +16
    pub size: f32,      // +32
}

/// a host that ignores everything.
#[derive(Debug, Default)]
pub struct NullHost;
impl AiHost for NullHost {}

/// everything a frame of ai update needs.
pub struct AiContext<'a> {
    pub physics: &'a mut dyn AiPhysicsWorld,
    pub host: &'a mut dyn AiHost,
    pub game: &'a AiGameState,
    pub event: &'a AiEventInfo,
    pub rand: &'a mut VsRand,
}
