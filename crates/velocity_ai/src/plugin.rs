//! Bevy side of the AI: builds the `AiWorld` on the original load steps, runs `VsAIWorld::Update` in the
//! game tick's AI phase, and turns race-session and collision messages into AI calls.
//!
//! Physics cars are reached through the `AiPhysics` resource, backed by `velocity_physics`'s shared world
//! ([`PhysicsAi`]) whenever the physics plugin is in the app. Race cars are bound to physics vehicles as
//! PhysicsManager 0x0019E080 numbers them (players, then opponents) once those exist; until then the AI
//! loads its data but does not drive.

use bevy::prelude::*;
use velocity_game::game_data_loading::game_data_loading::ELoadStep;
use velocity_game::game_mode::race_session::{ESessionState, RaceProgress, RaceSession, RaceSessionMessage};
use velocity_game::vs_common_main::{GameTick, InGameUpdateSet};
use velocity_game::{EGameMode, GameManager, GameModeMessage, LoadStepMessage};

use crate::ai::ai_context::{AiContext, AiEventInfo, AiFlashRecord, AiGameState, AiHost, AiPhysicsWorld, NullHost};
use visos_render::flare_render::{FlareFlash, FlashRecord};
use crate::ai::ai_world::{AiWorld, AiWorldFlags};
use crate::ai::vs_ai_player_tft_manager::{NullTft, TftPlayer};
use crate::ai::vs_ai_race_vehicle::{NoRespawns, RespawnPoint, RespawnSource};
use crate::team_database::TeamDatabase;
use velocity_game::game_mode::career::CareerMessage;
use velocity_game::career_teams::{CareerTeams, TeamRequest, TeamView};
use velocity_game::game_mode::race_effects_bridge::FadeStart;
use velocity_game::message::message_manager::GameMessage;
use velocity_game::game_mode::trail_blazer::TrailBlazerMessage;
use velocity_game::triggers::trigger_update::CTriggers;
use crate::ai::vs_math::{vs_quat_axis_angle, VsRand, VsVector4};
use crate::ai::vs_race_manager::RaceSetup;
use crate::opponent_database::OpponentDatabase;
use crate::physics_ai::PhysicsAi;
use crate::ai::vs_ai_race_vehicle::VehicleRef;
use velocity_game::game_mode::game_manager::RaceData;
use velocity_game::main_cpp::CommandLine;
use velocity_game::triggers::trigger_update::TriggerMessage;
use velocity_game::timeline_manager::objects::timeline_object::TimelineObjectMessage;
use velocity_physics::plugin::{PhysicsCollision, PhysicsWorldRes, SetPhysicsWalls};

/// the physics cars; inserted by the physics crate.
#[derive(Resource)]
pub struct AiPhysics(pub Box<dyn AiPhysicsWorld + Send + Sync>);

/// render/audio/osd notifications; optional (defaults to ignoring them).
#[derive(Resource)]
pub struct AiHostResource(pub Box<dyn AiHost + Send + Sync>);

/// the TFT movie player; optional.
#[derive(Resource)]
pub struct AiTftPlayer(pub Box<dyn TftPlayer + Send + Sync>);

/// level respawn triggers (`TriggerUpdate`); optional.
#[derive(Resource)]
pub struct AiRespawns(pub Box<dyn RespawnSource + Send + Sync>);

/// a physics collision between two cars (ids; -1 = scenery), the world's message 0 (VsAIDriver.cpp 0x000160B0).
#[derive(Message, Clone, Copy, Debug)]
pub struct AiCollisionMessage {
    pub a: i32,
    pub b: i32,
    pub impact: f32,
}

/// VsCrashCamera.cpp 0x00062D90 → VsAIRaceVehicle 0x000208D0: the crash camera's end puts the crashed car (race car
/// `+0x78` id) back on its raceline at `speed` (`flt_6850F0`). Written by core from `CrashCameraEvent::Respawn`
/// (CrashCam set, before the Ai set); applied at the start of the Ai set.
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct AiRespawnOnRaceline {
    pub vehicle: u32,
    pub speed: f32,
}

/// everything the AI keeps between frames.
#[derive(Resource, Default)]
pub struct AiState {
    pub world: Option<AiWorld>,
    pub game: AiGameState,
    pub event: AiEventInfo,
    pub rand: VsRand,
    /// game ticks since the race was set up, as the frame timer time
    pub ticks: u64,
    /// a grid reset that came before the race cars were bound (replayed once they are)
    pub pending_reset: Option<bool>,
    /// wall meshes built at load, sent once the physics world exists
    pub pending_walls: Vec<SetPhysicsWalls>,
    /// the first race section's raceline for physics' race distance hook (Some(None): no race section), installed
    /// with the walls
    pub pending_distance_line: Option<Option<crate::ai::vs_ai_road_line::VsAIRoadLine>>,
    /// `g_FadeRender.Start` calls of this update, written as `FadeStart` by the Ai-set update
    pub fades: Vec<f32>,
    /// 0x0002C620 calls (race car `+0x78` ids), written as `AiCameraCut` + `GameMessage::CameraCut`
    pub cuts: Vec<u32>,
    /// the last `police_flash` colour of this update (copied into `PoliceFlash`)
    pub flash: Option<[f32; 4]>,
    /// OSD manager calls of this update, written as `AiCinematicBars` / `RaceSessionMessage::OsdFlags`
    pub osd: Vec<AiOsd>,
    /// FlareRender 0x001270E0 calls of this pass, written as `FlareFlash`
    pub flashes: Vec<AiFlashRecord>,
    /// cameras / render cars read by the ai (filled by `view_inputs`)
    pub view: AiViewInputs,
    /// GameManager +0x14C: the drag round's eliminated ai car (VsRaceManager 0x0003DB70 when the player qualified),
    /// deactivated by the next RestartRace
    pub drag_eliminated: Option<VehicleRef>,
}

/// OSD3dInterface.cpp 0x000EAE90's value: the 3d hud's +2912 tint (rgb, w = weight) last set by
/// `VsPoliceManager::flash` (0x0003A1C0); zero by default and from each race's AI init.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct PoliceFlash(pub [f32; 4]);

/// VsAIVehicle.cpp 0x0002C620 from a respawn (0x0002B800): the camera manager's player record `id` cuts
/// (`+44*id+40` = 1); the plugin also sends MessageManager message 3 (`GameMessage::CameraCut`).
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AiCameraCut {
    pub id: u32,
}

/// forwards to the installed host and records the fade starts (core applies them as `FadeStart`) and camera cuts.
struct FadeHost<'a> {
    inner: &'a mut dyn AiHost,
    fades: &'a mut Vec<f32>,
    cuts: &'a mut Vec<u32>,
    flash: &'a mut Option<[f32; 4]>,
    osd: &'a mut Vec<AiOsd>,
    view: &'a AiViewInputs,
    flashes: &'a mut Vec<AiFlashRecord>,
}

/// game OSD manager calls of an ai update, in call order
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiOsd {
    CinematicBars(bool),
    ClearFlags(u32),
}

/// OSDManager 0x000ED6E0(show) from the ai (VsAIRaceVehicle 0x00018BD7: `false` when a respawned car is handed back to
/// its human); core forwards it as the frontend's `HudCommand::CinematicBars`.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AiCinematicBars(pub bool);

impl AiHost for FadeHost<'_> {
    fn play_tft(&mut self, name: &str, player: i32, volume: f32) -> bool {
        self.inner.play_tft(name, player, volume)
    }
    fn engine_smoke(&mut self, vehicle_id: u32, on: bool) {
        self.inner.engine_smoke(vehicle_id, on)
    }
    fn traffic_blocker_object(&mut self, object: u32, disabled: bool) {
        self.inner.traffic_blocker_object(object, disabled)
    }
    fn traffic_hazard(&mut self, vehicle_id: u32, on: bool) {
        self.inner.traffic_hazard(vehicle_id, on)
    }
    fn traffic_spawned(&mut self, vehicle_id: u32) {
        self.inner.traffic_spawned(vehicle_id)
    }
    fn traffic_trigger_update(&mut self, vehicle_id: u32) {
        self.inner.traffic_trigger_update(vehicle_id)
    }
    #[allow(clippy::too_many_arguments)]
    fn police_light(&mut self, vehicle_id: u32, light: u32, on: bool, intensity: f32, a: u8, r: u8, g: u8, b: u8) {
        self.inner.police_light(vehicle_id, light, on, intensity, a, r, g, b)
    }
    fn police_flash(&mut self, colour: &VsVector4) {
        *self.flash = Some([colour.x, colour.y, colour.z, colour.w]);
        self.inner.police_flash(colour)
    }
    fn camera_point_of_interest(&mut self, physics: usize, weight: f32) {
        self.inner.camera_point_of_interest(physics, weight)
    }
    fn camera_point_of_interest_clear(&mut self, priority: f32) {
        self.inner.camera_point_of_interest_clear(priority)
    }
    fn crash_camera_reset(&mut self, physics: usize) {
        self.inner.crash_camera_reset(physics)
    }
    fn camera_cut(&mut self, id: u32) {
        self.cuts.push(id);
        self.inner.camera_cut(id)
    }
    fn in_camera_view(&self, physics: usize) -> bool {
        self.inner.in_camera_view(physics)
    }
    fn score_event(&mut self, vehicle_id: u32, e: velocity_game::score_manager::score_manager::ScoreEvent) {
        self.inner.score_event(vehicle_id, e)
    }
    fn fade_start(&mut self, t: f32) {
        self.fades.push(t);
        self.inner.fade_start(t)
    }
    fn cinematic_bars(&mut self, on: bool) {
        self.osd.push(AiOsd::CinematicBars(on));
        self.inner.cinematic_bars(on)
    }
    fn osd_clear_flags(&mut self, mask: u32) {
        self.osd.push(AiOsd::ClearFlags(mask));
        self.inner.osd_clear_flags(mask)
    }
    fn camera(&self, player: usize) -> Option<(VsVector4, VsVector4)> {
        self.view.cameras.get(player).copied().flatten().or_else(|| self.inner.camera(player))
    }
    fn car_render_point(&self, physics: usize) -> Option<VsVector4> {
        self.view.car_points.iter().find(|(h, _)| *h == physics).map(|(_, p)| *p).or_else(|| self.inner.car_render_point(physics))
    }
    fn flare_flash(&mut self, rec: &AiFlashRecord) {
        self.flashes.push(*rec);
        self.inner.flare_flash(rec)
    }
}

/// what the ai reads from the cameras and the render cars, gathered before each Ai-set pass (`view_inputs`)
#[derive(Clone, Debug, Default)]
pub struct AiViewInputs {
    /// per player: camera position (+212) and direction (+280)
    pub cameras: Vec<Option<(VsVector4, VsVector4)>>,
    /// (physics id, car data +64 translation)
    pub car_points: Vec<(usize, VsVector4)>,
}

/// the cameras (VsCameraManager's current view per player) and the render cars' root translations (car data +0;
/// the engine shake CarRender 0x0011ABB0 adds for +64 is not modelled).
fn view_inputs(
    mut st: ResMut<AiState>,
    cams: Option<Res<velocity_game::camera::game_cameras::GameCameras>>,
    cars: Query<(&visos_render::car_render::CarRender, &Transform)>,
) {
    let s = &mut *st;
    s.view.cameras.clear();
    if let Some(c) = cams.as_ref() {
        for p in 0..c.manager.players.len() {
            s.view.cameras.push(c.manager.view(p).map(|v| {
                let mut dir = VsVector4::new(v.target.x - v.position.x, v.target.y - v.position.y, v.target.z - v.position.z, 1.0);
                dir.normalise();
                (VsVector4::new(v.position.x, v.position.y, v.position.z, 1.0), dir)
            }));
        }
    }
    s.view.car_points.clear();
    for (r, t) in &cars {
        s.view.car_points.push((r.request as usize, VsVector4::new(t.translation.x, t.translation.y, t.translation.z, 1.0)));
    }
}

/// FlareRender 0x001270E0 calls of an ai pass, as graphics' `FlareFlash`
fn write_flashes(st: &mut AiState, out: &mut MessageWriter<FlareFlash>) {
    for r in st.flashes.drain(..) {
        out.write(FlareFlash(FlashRecord {
            rise: r.rise,
            fall: r.fall,
            intensity: r.intensity,
            screen: r.screen,
            pos: Vec3::new(r.pos.x, r.pos.y, r.pos.z),
            size: r.size,
        }));
    }
}

/// the career's team database (VsCommonMain `+0x04`, `CTeamDatabase`) with its runtime lock / invite flags
#[derive(Resource, Default)]
pub struct AiTeams(pub Option<TeamDatabase>);

/// 1/24 s, the game tick
const TICK: f32 = 1.0 / 24.0;

pub(crate) fn build(app: &mut App) {
    // the opponents' / teams' race cars for game flow (PhysicsManager 0x0019E080)
    if !app.world().contains_resource::<velocity_game::game_mode::race_cars::RaceCarSourceRes>() {
        app.insert_resource(velocity_game::game_mode::race_cars::RaceCarSourceRes(Box::new(crate::race_car_source::AiRaceCarSource::load())));
    }
    if !app.world().contains_resource::<AiTeams>() {
        app.insert_resource(AiTeams(TeamDatabase::load()));
    }
    app.init_resource::<AiState>()
        .init_resource::<crate::ai_tweaks::AiTweaks>()
        .init_resource::<PoliceFlash>()
        .add_message::<RaceSessionMessage>()
        .add_message::<CareerMessage>()
        .init_resource::<CareerTeams>()
        .add_message::<TeamRequest>()
        .add_systems(Last, career_teams)
        .add_message::<AiCollisionMessage>()
        .add_message::<PhysicsCollision>()
        .add_message::<SetPhysicsWalls>()
        .add_message::<TriggerMessage>()
        .add_message::<TrailBlazerMessage>()
        .add_message::<FadeStart>()
        .add_message::<FlareFlash>()
        .add_message::<AiCameraCut>()
        .add_message::<AiCinematicBars>()
        .add_message::<GameMessage>()
        .add_message::<TimelineObjectMessage>()
        .add_message::<AiRespawnOnRaceline>()
        .add_systems(Last, (install_physics, load_steps, mode_messages).chain())
        .add_systems(
            GameTick,
            (tweaks_out, send_walls, session_messages, crash_respawns, view_inputs, update, tweaks_in)
                .chain()
                .in_set(InGameUpdateSet::Ai)
                .run_if(in_state(EGameMode::InGame))
                .run_if(|gm: Res<GameManager>| !gm.paused),
        )
        // the physics frame's messages reach the ai in the same tick (MessageManager delivers them at once; a
        // message written in one GameTick can expire before the next)
        .add_systems(
            GameTick,
            (physics_collisions, collisions, trigger_hints)
                .chain()
                .after(InGameUpdateSet::Physics)
                .before(InGameUpdateSet::Cameras)
                .run_if(in_state(EGameMode::InGame))
                .run_if(|gm: Res<GameManager>| !gm.paused),
        )
        // TimelineAI.cpp handlers: called from the race timeline sequencer (Timeline set), same tick
        .add_systems(
            GameTick,
            timeline_objects
                .after(InGameUpdateSet::Timeline)
                .run_if(in_state(EGameMode::InGame))
                .run_if(|gm: Res<GameManager>| !gm.paused),
        );
}

/// debug-menu edits (GameTick Input set, also while paused) reach the ai variables before its update
fn tweaks_out(mut st: ResMut<AiState>, t: Res<crate::ai_tweaks::AiTweaks>) {
    if let Some(w) = st.world.as_mut() {
        t.tweaks_out(w);
    }
}

/// what the ai wrote (e.g. TightCornerThreshold every update) shows in the menu
fn tweaks_in(mut st: ResMut<AiState>, t: Res<crate::ai_tweaks::AiTweaks>) {
    if let Some(w) = st.world.as_mut() {
        t.tweaks_in(w);
    }
}

fn v4(p: [f32; 3]) -> VsVector4 {
    VsVector4::new(p[0], p[1], p[2], 1.0)
}

/// the race inputs from the event definition (GameDataLoading.cpp 0x000A2EF0 globals 0x006F5C0C..0x006F5C64).
fn race_state(gm: &GameManager) -> (AiGameState, AiEventInfo, RaceSetup) {
    let mut g = AiGameState::default();
    let mut e = AiEventInfo::default();
    let race = gm.race.clone().unwrap_or_default();
    if let Some(d) = gm.event_def.as_ref() {
        g.grid = d.start_positions.iter().map(|&p| v4(p)).collect();
        // ((0 - dir) + 90) degrees about +y (0x000A3460)
        let ang = ((0.0 - d.start_direction) + 90.0) * f32::from_bits(0x3C8E_FA35);
        g.grid_heading = vs_quat_axis_angle(ang, &VsVector4::new(0.0, 1.0, 0.0, 1.0));
        // placeholder eventdefs (every LACoast race: no grid, both lines at the origin) break into the debugger
        // in the original (ProjectOntoLine "VecA != VecB" from LoadEventDefinition and SetStartPoint); here the
        // degenerate line is left out, so no start snap and no lap counting
        let line = |l: &[[f32; 3]; 2]| (l[0] != l[1]).then(|| (v4(l[0]), v4(l[1])));
        g.start_line_original = line(&d.start_line);
        g.start_line = line(&d.finish_line);
        if g.start_line.is_none() || g.start_line_original.is_none() {
            log::warn!("ai: event definition has a degenerate start/finish line (placeholder data)");
        }
        g.finish_point = v4(d.unknown_line_point);
        e.num_laps = d.num_laps.max(0) as u8;
        e.num_opponents = d.num_opponents.max(0) as u8;
    }
    e.mode = race.race_type as u32;
    let mut humans = race.num_players.max(1) as u8;
    // the current event's race (`dword_6F9BE8`): type +0x04, laps +0x14, competitors +0x15
    if let Some(ev) = gm.current_event()
        && let Some(r) = ev.current_race()
    {
        e.mode = r.race_type;
        e.num_laps = r.laps;
        e.num_opponents = r.num_competitors;
        humans = r.human_count(ev.competitors.as_deref().unwrap_or(&[]));
    }
    g.player_prestige = 0;
    g.network = gm.vars.network_race;
    g.speech_enabled = gm.vars.speech;
    // VsRaceManager.cpp 0x0003AD10: competitors minus the humans seated in the race, none in split screen
    let num_ai = if race.splitscreen { 0 } else { e.num_opponents.saturating_sub(humans) as u32 };
    let setup = RaceSetup { network: false, network_players: 0, num_ai, num_players: race.num_players.max(1) as u32 };
    (g, e, setup)
}

/// the AI's view of the physics cars: the shared physics world, once the physics plugin is in the app.
fn install_physics(mut commands: Commands, res: Option<Res<PhysicsWorldRes>>, physics: Option<Res<AiPhysics>>) {
    if physics.is_none()
        && let Some(r) = res
    {
        commands.insert_resource(AiPhysics(Box::new(PhysicsAi::new(r.0.clone()))));
    }
}

/// PhysicsManager 0x0019E080: each race car gets the physics vehicle with its id (players first, then the
/// opponents) once that exists. Players: `SetPlayer`, `SetHumanControl(1)`, control mode 0; opponents: base
/// +0x20 set, control mode 1 (4 in a network race). Returns whether the players are bound (opponent cars the
/// game did not create stay unbound and inactive).
fn bind_race_cars(w: &mut AiWorld, phys: &mut dyn AiPhysicsWorld, prestige: u32, network: bool) -> bool {
    let np = w.race.players.len();
    let refs: Vec<(VehicleRef, usize)> =
        (0..np).map(|i| (VehicleRef::Player(i), i)).chain((0..w.race.ai.len()).map(|i| (VehicleRef::Ai(i), np + i))).collect();
    let mut all = true;
    for (r, id) in refs {
        if w.race.vehicle(r).unwrap().base.physics.is_some() {
            continue;
        }
        if !phys.has_vehicle(id) {
            all &= !matches!(r, VehicleRef::Player(_));
            continue;
        }
        let g = w.world.globals.clone();
        let time = w.world.time;
        let v = w.race.vehicle_mut(r).unwrap();
        v.base.set_physics_vehicle(id);
        if let VehicleRef::Player(_) = r {
            v.set_player(true, &g, prestige);
            v.set_human_control_on(true, &w.world, phys.vehicle(id), time);
            phys.vehicle(id).set_control_mode(0);
        } else {
            v.base.active = true;
            phys.vehicle(id).set_control_mode(if network { 4 } else { 1 });
            if network {
                v.set_human_control_on(true, &w.world, phys.vehicle(id), time);
            }
        }
        log::info!("AI: race car {r:?} on physics vehicle {id}");
    }
    if !all {
        return false;
    }
    // "Initialising Traffic Vehicles": control mode 2, VsAIVehicle 0x0002BD00, VE+0x13D8 = traffic index
    if let Some(t) = w.traffic.as_mut() {
        for (i, v) in t.vehicles.iter_mut().enumerate().filter(|(_, v)| v.base.physics.is_none()) {
            let id = v.id as usize;
            if phys.has_vehicle(id) {
                phys.vehicle(id).set_control_mode(2);
                v.base.set_physics_vehicle(id);
                phys.vehicle(id).set_traffic_index(i as i32);
            }
        }
    }
    // "Initialising Police Vehicles": control mode 1, VsAIPoliceVehicle 0x000177E0 binds both parts, then the
    // car's handling is scaled (VE+3220 x2, LC +264/+92 x2, +80/+76 x1.5)
    if let Some(p) = w.police.as_mut() {
        for v in p.vehicles.iter_mut().filter(|v| v.race.base.physics.is_none()) {
            let id = v.race.id as usize;
            if phys.has_vehicle(id) {
                phys.vehicle(id).set_control_mode(1);
                v.set_physics_vehicle(id);
                phys.vehicle(id).police_handling();
            }
        }
    }
    // the original creates traffic and police cars with the race cars; managers whose cars the game did not
    // create would drive vehicles that are not there
    let unbound = |t: &crate::ai::vs_traffic_manager::VsTrafficManager| t.vehicles.iter().any(|v| v.base.physics.is_none());
    if w.traffic.as_ref().is_some_and(unbound) {
        log::warn!("AI: no physics cars for traffic, traffic off");
        w.traffic = None;
        w.world.traffic_enabled = false;
    }
    if w.police.as_ref().is_some_and(|p| p.vehicles.iter().any(|v| v.race.base.physics.is_none())) {
        log::warn!("AI: no physics cars for police, police off");
        w.police = None;
        w.world.police_enabled = false;
    }
    all
}

/// "Ai World", "AI World Data", "TFT Manager" (GameManager.cpp 0x000B5F80).
#[allow(clippy::too_many_arguments)]
fn load_steps(
    mut steps: MessageReader<LoadStepMessage>,
    mut gm: ResMut<GameManager>,
    mut st: ResMut<AiState>,
    mut physics: Option<ResMut<AiPhysics>>,
    mut host: Option<ResMut<AiHostResource>>,
    cl: Option<Res<CommandLine>>,
    mut tweaks: ResMut<crate::ai_tweaks::AiTweaks>,
) {
    if steps.is_empty() {
        return;
    }
    rand_in(&mut st, Some(&*gm));
    for m in steps.read() {
        match m.step {
            ELoadStep::AiWorldInit => {
                let Some(dir) = crate::game_dir() else {
                    log::error!("AI: no game directory");
                    continue;
                };
                let (game, event, setup) = race_state(&gm);
                let race = gm.race.clone().unwrap_or_default();
                let flags = AiWorldFlags::for_race(race.traffic, gm.vars.network_race, race.splitscreen, race.professional, event.mode);
                st.game = game;
                st.event = event;
                st.ticks = 0;
                let s = &mut *st;
                let mut null_phys = NullPhysics;
                let mut null_host = NullHost;
                let phys: &mut dyn AiPhysicsWorld = match physics.as_deref_mut() {
                    Some(p) if p.0.ready() => &mut *p.0,
                    _ => &mut null_phys,
                };
                let host: &mut dyn AiHost = match host.as_deref_mut() {
                    Some(h) => &mut *h.0,
                    None => &mut null_host,
                };
                let mut ctx = AiContext { physics: phys, host, game: &s.game, event: &s.event, rand: &mut s.rand };
                let statics = &*tweaks;
                let mut w = AiWorld::init_with(crate::resolve(&dir, "AI"), &mut ctx, flags, &setup, |a| statics.push_statics(a));
                // a new race starts without a police tint (PoliceFlash, written by the first update)
                s.flash = Some([0.0; 4]);
                s.drag_eliminated = None;
                // PhysicsManager.cpp 0x0019E080: the current race's competitor slots in order, the non-human
                // ones by opponent crc (database order when no event is set)
                if let Some(db) = OpponentDatabase::load() {
                    let crcs: Vec<u32> = gm
                        .current_event()
                        .and_then(|e| {
                            let comp = e.competitors.as_ref()?;
                            let race = e.current_race()?;
                            Some(race.competitors.iter().flatten().filter_map(|&c| comp.get(c)).filter(|c| c.human == 0).map(|c| c.opponent_crc).collect())
                        })
                        .unwrap_or_default();
                    for (i, d) in w.race.drivers.iter_mut().enumerate() {
                        let o = if crcs.is_empty() { db.opponents.get(i) } else { crcs.get(i).and_then(|&c| db.get_opponent_by_crc(c)) };
                        match o {
                            Some(o) => d.set_opponent(o.clone()),
                            None => log::warn!("AI: no opponent for driver {i}"),
                        }
                    }
                }
                s.world = Some(w);
            }
            ELoadStep::AiWorldData => {
                let (Some(paths), Some(dir)) = (m.paths.as_ref(), crate::game_dir()) else { continue };
                let level = crate::resolve(&dir, &paths.race_dir);
                let start = st.game.start_line;
                let s = &mut *st;
                if let Some(w) = s.world.as_mut() {
                    w.load_world_data(&level, start);
                    // ProcessSplineData 0x00031040: a circular route copies the finish line over the start line
                    // (dword_6F5C24/34 = dword_6F5C44/54)
                    if start.is_some() && w.world.race_sections().first().is_some_and(|r| r.raceline.is_circular()) {
                        s.game.start_line_original = start;
                    }
                    // Armco::BuildMesh (0x00012440) runs inside the armco load; the barrier mesh is "Barrier Render
                    // Init" (GameManager InGameInit -> BarrierRender 0x00107560 -> 0x00108030) unless `nobarriers`
                    // (main 0x0016D806 clears flag 4)
                    s.pending_walls.clear();
                    if let Some(a) = w.armco.as_ref() {
                        s.pending_walls.push(SetPhysicsWalls { armco: true, vertices: a.mesh.vertices.clone(), faces: a.mesh.faces.clone() });
                    }
                    if !cl.as_ref().is_some_and(|c| c.no_barriers)
                        && let Some(m) = crate::barrier_render::build_collision_mesh(&w.world)
                    {
                        s.pending_walls.push(SetPhysicsWalls { armco: false, vertices: m.vertices, faces: m.faces });
                    }
                    log::info!("AI: {} wall meshes for physics", s.pending_walls.len());
                    s.pending_distance_line = Some(w.world.race_sections().first().map(|r| r.raceline.clone()));
                    // LoadWorldData 0x00030348
                    crate::ai_tweaks::add_tweaks(&mut gm.tweaks, &mut tweaks, w);
                }
            }
            ELoadStep::TftManager => {
                // VsAIDriver.cpp 0x00016820: tweaks plus 0x00016850
                if let Some(w) = st.world.as_mut() {
                    crate::ai_tweaks::add_tft_tweaks(&mut gm.tweaks, &mut tweaks, w);
                    w.race.tft.last_time = -1.0;
                    w.race.tft.last_volume = 0.0;
                }
            }
            _ => {}
        }
    }
    rand_out(&st, Some(&mut *gm));
}

/// the wall meshes reach the physics world in the same game tick (its `set_walls` runs at the start of the
/// physics frame), once that world exists.
fn send_walls(mut st: ResMut<AiState>, res: Option<Res<PhysicsWorldRes>>, mut out: MessageWriter<SetPhysicsWalls>) {
    let Some(res) = res.filter(|r| r.lock().is_some()) else { return };
    // the hydrant hit's race distance (PhysicsObjectManager 0x001E1F80 case 0x17): VsAIWorld::GetFirstRaceSection
    // 0x00024300 -> GetRaceline -> VsAIRoadLine::FindClosestSegment 0x000262D0 -> GetDistanceAlongLine 0x00025E00.
    // without a race section (cruise) the original reads through NULL; the hook is left out (distance 0)
    if let Some(line) = st.pending_distance_line.take()
        && let Some(w) = res.lock().as_mut()
    {
        let hook: Option<std::sync::Arc<std::sync::Mutex<velocity_physics::vehicle::vehicle_entity::cx::RaceDistanceFn>>> =
            line.map(|line| {
                let f = move |p: [f32; 3]| {
                    let pos = VsVector4::new(p[0], p[1], p[2], 1.0);
                    let seg = line.find_closest_segment(&pos);
                    line.get_distance_along_line(&pos, seg)
                };
                std::sync::Arc::new(std::sync::Mutex::new(f)) as std::sync::Arc<std::sync::Mutex<velocity_physics::vehicle::vehicle_entity::cx::RaceDistanceFn>>
            });
        w.set_race_distance_hook(hook);
    }
    if st.pending_walls.is_empty() {
        return;
    }
    for m in st.pending_walls.drain(..) {
        log::info!("AI: {} walls, {} faces", if m.armco { "armco" } else { "barrier" }, m.faces.len());
        out.write(m);
    }
}

/// ai-only triggers: `VsAIRaceVehicle::SetCheckpoint` (0x00021980) on the race clock, and the trailblazer start
/// of a player car (TriggerUpdate 0x00180310).
fn trigger_hints(
    mut msgs: MessageReader<TriggerMessage>,
    mut st: ResMut<AiState>,
    physics: Option<Res<AiPhysics>>,
    mut out: MessageWriter<TrailBlazerMessage>,
) {
    let clock = st.game.race_clock;
    let Some(w) = st.world.as_mut() else { return };
    for m in msgs.read() {
        match *m {
            TriggerMessage::AiHint { vehicle, hint } => {
                let r = w.race.find_vehicle_by_id(vehicle);
                if let Some(v) = w.race.vehicle_mut(r) {
                    v.set_checkpoint(hint, clock);
                }
            }
            TriggerMessage::Trailblazer { vehicle, spline, max_distance, success_timeline, respawn_trigger, flag } => {
                let saved = physics.as_ref().map_or([0.0; 6], |p| p.0.trail_blazer_globals());
                let r = w.race.find_vehicle_by_id(vehicle);
                if let Some(v) = w.race.vehicle_mut(r) {
                    v.start_trail_blazer(&mut w.world, spline, max_distance, success_timeline, respawn_trigger, flag, saved);
                }
            }
            _ => {}
        }
    }
    send_trail_events(w, &mut out);
}

/// the `F3AIController` timeline events (TimelineAI.cpp, run by the race timeline sequencer at the end of the tick)
fn timeline_objects(
    mut msgs: MessageReader<TimelineObjectMessage>,
    mut st: ResMut<AiState>,
    mut physics: Option<ResMut<AiPhysics>>,
    mut host: Option<ResMut<AiHostResource>>,
    mut tft: Option<ResMut<AiTftPlayer>>,
    mut gm: Option<ResMut<GameManager>>,
) {
    rand_in(&mut st, gm.as_deref());
    for m in msgs.read() {
        match m {
            TimelineObjectMessage::EnableRaceSection { raceline, enable } => {
                if let Some(w) = st.world.as_mut() {
                    w.enable_race_section(*raceline, *enable);
                }
            }
            TimelineObjectMessage::RespawnRacersOnRaceline { value } => {
                with_ctx(&mut st, &mut physics, &mut host, &mut tft, |w, ctx, out| {
                    w.race.respawn_players_on_raceline(ctx, &mut w.world, out, *value)
                });
            }
            TimelineObjectMessage::PlayerHumanControl { human } => {
                with_ctx(&mut st, &mut physics, &mut host, &mut tft, |w, ctx, out| {
                    w.race.player_human_control(ctx, &mut w.world, out, *human)
                });
            }
            TimelineObjectMessage::Respawn { inside, points, speed } => {
                let pts: Vec<Option<(VsVector4, VsVector4)>> = points
                    .iter()
                    .map(|p| p.map(|(pos, q)| (VsVector4::new(pos.x, pos.y, pos.z, 1.0), VsVector4::new(q.x, q.y, q.z, q.w))))
                    .collect();
                with_ctx(&mut st, &mut physics, &mut host, &mut tft, |w, ctx, out| {
                    w.race.timeline_respawn(ctx, &mut w.world, out, *inside, &pts, *speed)
                });
            }
            _ => {}
        }
    }
    rand_out(&st, gm.as_deref_mut());
}

/// every race car's trailblazer events as `TrailBlazerMessage`s (read by core in the same tick)
fn send_trail_events(w: &mut AiWorld, out: &mut MessageWriter<TrailBlazerMessage>) {
    for v in w.race.players.iter_mut().chain(w.race.ai.iter_mut()) {
        let vehicle = v.id;
        for event in v.trail_events.drain(..) {
            out.write(TrailBlazerMessage { vehicle, event });
        }
    }
}

/// the level triggers as respawn points (`TriggerUpdate` 0x00181AB0 by name crc; record +0x00 position,
/// +0x10 rotation, +0x40.. params)
struct TriggerRespawns<'a>(&'a CTriggers);

impl RespawnSource for TriggerRespawns<'_> {
    fn respawn_point(&mut self, crc: u32) -> Option<RespawnPoint> {
        let t = &self.0.find(crc)?.file;
        let p = t.position;
        let q = t.rotation;
        Some(RespawnPoint {
            pos: VsVector4::new(p[0], p[1], p[2], p[3]),
            orientation: VsVector4::new(q[0], q[1], q[2], q[3]),
            delay_ms: t.params[0],
            speed: t.params[1],
            param_2: t.params[2],
        })
    }
}

/// career team flags: the unlock of a successful event (EventDatabaseBinLoading 0x00091890) and the lock refresh
/// after a profile load (SaveFileData 0x001050C0) or new profile (SetInitialVariables 0x000C1930), VsCommonMain
/// 0x0016E790
fn career_teams(
    mut session: MessageReader<RaceSessionMessage>,
    mut career: MessageReader<CareerMessage>,
    mut requests: MessageReader<TeamRequest>,
    mut teams: ResMut<AiTeams>,
    mut view: ResMut<CareerTeams>,
    gm: Option<Res<GameManager>>,
) {
    use velocity_game::event_database::event_database::EVENT_FLAG_COMPLETED;
    let Some(db) = teams.0.as_mut() else {
        session.clear();
        career.clear();
        requests.clear();
        return;
    };
    let mut changed = view.teams.len() != db.teams.len();
    for m in session.read() {
        if let RaceSessionMessage::EventOver(o) = m
            && o.success
            && let Some(t) = o.unlock_team
        {
            db.unlock_team(t);
            changed = true;
        }
    }
    let refresh = career.read().any(|m| matches!(m, CareerMessage::Created { .. } | CareerMessage::Loaded { ok: true, .. }));
    if refresh && let Some(gm) = gm {
        db.refresh_all_teams_lock_status(gm.event_db.events.iter().map(|e| (e.flags & EVENT_FLAG_COMPLETED != 0, e.unlock_team)));
        changed = true;
    }
    for r in requests.read() {
        changed |= apply_team_request(db, *r);
    }
    if changed {
        mirror_teams(db, &mut view);
    }
}

/// a front-end team change on the team FindTeam (0x0016E610) returns; false when the crc is unknown
fn apply_team_request(db: &mut TeamDatabase, r: TeamRequest) -> bool {
    let (crc, f): (u32, fn(&mut crate::team_database::Team)) = match r {
        TeamRequest::ClearNewlyUnlocked(c) => (c, crate::team_database::Team::clear_newly_unlocked),
        TeamRequest::SetJoinedOnce(c) => (c, crate::team_database::Team::set_joined_once),
    };
    match db.find_team_mut(crc) {
        Some(t) => {
            f(t);
            true
        }
        None => {
            log::warn!("team request {r:?}: unknown team crc {crc:#x}");
            false
        }
    }
}

/// `CareerTeams` = the database in the original list order (the reverse of load order)
fn mirror_teams(db: &TeamDatabase, view: &mut CareerTeams) {
    view.teams = db
        .teams
        .iter()
        .rev()
        .map(|t| TeamView { crc: t.get_id(), name: t.name(), car: t.car(), status: t.status(), cash_bonus: t.cash_bonus(), flags: t.flags })
        .collect();
}

fn mode_messages(mut msgs: MessageReader<GameModeMessage>, mut st: ResMut<AiState>) {
    for m in msgs.read() {
        if *m == GameModeMessage::InGameShutdown
            && let Some(w) = st.world.as_mut()
        {
            w.shutdown();
            st.world = None;
        }
    }
}

/// runs `f` with an `AiContext` over whatever resources exist; skipped without physics.
/// VsRandRange 0x000134F0 / VsRandUnit 0x00013510 draw from CRT `rand` 0x003DE030, the one state the whole game
/// shares (core's `GameManager.rand`, handed to physics around its tick): the ai's `VsRand` runs on it.
fn rand_in(st: &mut AiState, gm: Option<&GameManager>) {
    if let Some(gm) = gm {
        st.rand.seed = gm.rand.state;
    }
}
fn rand_out(st: &AiState, gm: Option<&mut GameManager>) {
    if let Some(gm) = gm {
        gm.rand.state = st.rand.seed;
    }
}

fn with_ctx(
    st: &mut AiState,
    physics: &mut Option<ResMut<AiPhysics>>,
    host: &mut Option<ResMut<AiHostResource>>,
    tft: &mut Option<ResMut<AiTftPlayer>>,
    f: impl FnOnce(&mut AiWorld, &mut AiContext, &mut dyn TftPlayer),
) {
    let Some(p) = physics.as_deref_mut().filter(|p| p.0.ready()) else { return };
    let s = &mut *st;
    let Some(w) = s.world.as_mut() else { return };
    p.0.begin_access();
    // the original creates every race car before the ai's first update
    if !bind_race_cars(w, &mut *p.0, s.game.player_prestige, s.game.network) {
        return;
    }
    let mut null_host = NullHost;
    let inner: &mut dyn AiHost = match host.as_deref_mut() {
        Some(h) => &mut *h.0,
        None => &mut null_host,
    };
    let mut fade_host =
        FadeHost { inner, fades: &mut s.fades, cuts: &mut s.cuts, flash: &mut s.flash, osd: &mut s.osd, view: &s.view, flashes: &mut s.flashes };
    let host: &mut dyn AiHost = &mut fade_host;
    let mut null_tft = NullTft;
    let out: &mut dyn TftPlayer = match tft.as_deref_mut() {
        Some(t) => &mut *t.0,
        None => &mut null_tft,
    };
    let mut ctx = AiContext { physics: &mut *p.0, host, game: &s.game, event: &s.event, rand: &mut s.rand };
    // a grid reset waits for the race cars' physics vehicles (the original creates them first)
    if let Some(pursuit) = s.pending_reset.take() {
        w.reset_to_grid(&mut ctx, out, pursuit);
    }
    f(w, &mut ctx, out);
    ctx.physics.end_access();
}

/// grid reset (VsRaceManager 0x0003C5D0), restarts and the drag rounds' cars.
#[allow(clippy::too_many_arguments)]
fn session_messages(
    mut msgs: MessageReader<RaceSessionMessage>,
    mut modes: MessageReader<GameModeMessage>,
    session: Res<RaceSession>,
    mut st: ResMut<AiState>,
    mut physics: Option<ResMut<AiPhysics>>,
    mut host: Option<ResMut<AiHostResource>>,
    mut tft: Option<ResMut<AiTftPlayer>>,
    mut gm: Option<ResMut<GameManager>>,
) {
    rand_in(&mut st, gm.as_deref());
    // RestartRace is written by GameManager::Update (Input set) of this tick, before MonitorSessions' grid reset
    if modes.read().any(|m| *m == GameModeMessage::RaceRestart) {
        let eliminated = st.drag_eliminated.take();
        let drag = st.event.mode == 5;
        // the next round keeps its round count (+0x142); any other restart of a drag race is 0x000BF270's
        let next_round = session.drag_round > 0;
        let mut done = false;
        with_ctx(&mut st, &mut physics, &mut host, &mut tft, |w, ctx, _| {
            if drag && !next_round {
                w.drag_reactivate(ctx);
            }
            // GameManager::RestartRace 0x000BE0F0: VsAIWorld::Reset, the score reset 0x0003DA60, later the +0x14C car out
            w.reset(ctx);
            w.race.reset_scores();
            if drag
                && next_round
                && let Some(r) = eliminated
            {
                w.drag_deactivate(ctx, r);
            }
            done = true;
        });
        // the race's first RestartRace (InGameInit) comes before the cars are bound: no physics car to touch
        let s = &mut *st;
        if !done && let Some(w) = s.world.as_mut() {
            let mut null_phys = NullPhysics;
            let mut null_host = NullHost;
            let mut ctx = AiContext { physics: &mut null_phys, host: &mut null_host, game: &s.game, event: &s.event, rand: &mut s.rand };
            w.reset(&mut ctx);
            w.race.reset_scores();
        }
    }
    for m in msgs.read() {
        match *m {
            RaceSessionMessage::ResetToGrid { pursuit } => {
                st.pending_reset = Some(pursuit);
                with_ctx(&mut st, &mut physics, &mut host, &mut tft, |_, _, _| {});
            }
            // GameManager 0x000BC5F0 state 5, drag: a qualified player puts the last active car out (+0x14C; a
            // human there is the ELIMINATED branch, so these two results always name an ai car)
            RaceSessionMessage::DragResult(r)
                if r == crate::crc::string_crc("OSD_DRAG_WINNER") || r == crate::crc::string_crc("OSD_DRAG_QUALIFIED") =>
            {
                st.drag_eliminated = st.world.as_ref().and_then(|w| w.drag_eliminated_car());
            }
            _ => {}
        }
    }
    rand_out(&st, gm.as_deref_mut());
}

/// VsCrashCamera.cpp 0x00062D90's respawn of the crashed car.
fn crash_respawns(
    mut msgs: MessageReader<AiRespawnOnRaceline>,
    mut st: ResMut<AiState>,
    mut physics: Option<ResMut<AiPhysics>>,
    mut host: Option<ResMut<AiHostResource>>,
    mut tft: Option<ResMut<AiTftPlayer>>,
    mut gm: Option<ResMut<GameManager>>,
) {
    if msgs.is_empty() {
        return;
    }
    rand_in(&mut st, gm.as_deref());
    for m in msgs.read() {
        with_ctx(&mut st, &mut physics, &mut host, &mut tft, |w, ctx, out| {
            w.race.crash_camera_respawn(ctx, &mut w.world, out, m.vehicle, m.speed)
        });
    }
    rand_out(&st, gm.as_deref_mut());
}

/// the physics frame's collision messages (MessageManager message 0) as the AI's.
fn physics_collisions(mut msgs: MessageReader<PhysicsCollision>, mut out: MessageWriter<AiCollisionMessage>) {
    for m in msgs.read() {
        out.write(AiCollisionMessage { a: m.0.a, b: m.0.b, impact: m.0.impact });
    }
}

/// VsAIDriver.cpp 0x000160B0
fn collisions(
    mut msgs: MessageReader<AiCollisionMessage>,
    mut st: ResMut<AiState>,
    mut physics: Option<ResMut<AiPhysics>>,
    mut host: Option<ResMut<AiHostResource>>,
    mut tft: Option<ResMut<AiTftPlayer>>,
    mut flare_out: MessageWriter<FlareFlash>,
    mut gm: Option<ResMut<GameManager>>,
) {
    rand_in(&mut st, gm.as_deref());
    for m in msgs.read() {
        with_ctx(&mut st, &mut physics, &mut host, &mut tft, |w, ctx, out| w.on_collision(ctx, out, m.a, m.b, m.impact));
    }
    rand_out(&st, gm.as_deref_mut());
    // damage failures (0x00019740 event 1) in the collision handler
    write_flashes(&mut st, &mut flare_out);
}

/// "VsAIWorld->Update" (VsAIWorld.cpp 0x00034280) in the game tick's AI phase.
#[allow(clippy::too_many_arguments)]
fn update(
    mut st: ResMut<AiState>,
    mut session: ResMut<RaceSession>,
    mut physics: Option<ResMut<AiPhysics>>,
    mut host: Option<ResMut<AiHostResource>>,
    mut tft: Option<ResMut<AiTftPlayer>>,
    mut respawns: Option<ResMut<AiRespawns>>,
    mut gm: ResMut<GameManager>,
    data: Option<Res<RaceData>>,
    triggers: Option<Res<CTriggers>>,
    mut trail_out: MessageWriter<TrailBlazerMessage>,
    (mut fade_out, mut flare_out): (MessageWriter<FadeStart>, MessageWriter<FlareFlash>),
    (mut cut_out, mut msg_out): (MessageWriter<AiCameraCut>, MessageWriter<GameMessage>),
    clock: Option<Res<velocity_game::vs_common_main::GameClock>>,
    (progress, mut flash): (Option<ResMut<RaceProgress>>, ResMut<PoliceFlash>),
    (mut bars_out, mut session_out): (MessageWriter<AiCinematicBars>, MessageWriter<RaceSessionMessage>),
) {
    // InGameUpdate 0x000B7C15 passes the stretched step (GameManager 0x000BF670)
    let dt = clock.map_or(TICK, |c| c.timer.scaled_dt());
    st.ticks += 1;
    // the flag girl sequencer's +0xCD8 / +0xCDC (countdown start, ALLOWGO) and the speech option
    if let Some(fg) = data.as_ref().and_then(|d| d.flag_girl.as_ref()) {
        st.game.countdown_start = fg.in_game_time.unwrap_or(0.0);
        st.game.countdown_end = fg.allow_go_time.unwrap_or(0.0);
    }
    st.game.speech_enabled = gm.vars.speech;
    st.game.network = gm.vars.network_race;
    st.game.timer_time = st.ticks as f32 * TICK;
    st.game.race_clock = session.race_time;
    st.game.race_phase = match session.state {
        ESessionState::Init => 0,
        ESessionState::Reset => 1,
        ESessionState::Intro => 2,
        ESessionState::Countdown => 3,
        ESessionState::Go => 4,
        ESessionState::Racing => 5,
        ESessionState::Finished => 6,
        ESessionState::PostRace => 7,
    };
    let mut no_respawns = NoRespawns;
    let mut trigger_respawns;
    let rs: &mut dyn RespawnSource = match (respawns.as_deref_mut(), triggers.as_deref()) {
        (Some(r), _) => &mut *r.0,
        (None, Some(t)) => {
            trigger_respawns = TriggerRespawns(t);
            &mut trigger_respawns
        }
        (None, None) => &mut no_respawns,
    };
    rand_in(&mut st, Some(&*gm));
    with_ctx(&mut st, &mut physics, &mut host, &mut tft, |w, ctx, out| {
        w.update(ctx, out, rs, dt);
        // VehicleManager__1DD9F0 reads the pad or the controls block by this flag
        let human: Vec<(usize, bool)> =
            w.race.players.iter().chain(w.race.ai.iter()).filter_map(|v| v.base.physics.map(|h| (h, v.human_control))).collect();
        ctx.physics.end_update(&human);
    });
    rand_out(&st, Some(&mut *gm));
    // applied by core's Effects set before g_FadeRender.Update in this tick
    write_flashes(&mut st, &mut flare_out);
    for t in st.fades.drain(..) {
        fade_out.write(FadeStart(t));
    }
    for id in st.cuts.drain(..) {
        cut_out.write(AiCameraCut { id });
        msg_out.write(GameMessage::CameraCut);
    }
    if let Some(c) = st.flash.take() {
        flash.0 = c;
    }
    for o in st.osd.drain(..) {
        match o {
            AiOsd::CinematicBars(on) => {
                bars_out.write(AiCinematicBars(on));
            }
            AiOsd::ClearFlags(clear) => {
                session_out.write(RaceSessionMessage::OsdFlags { set: 0, clear });
            }
        }
    }
    if let Some(w) = st.world.as_mut() {
        session.race_finished = w.race.race_finished;
        send_trail_events(w, &mut trail_out);
        if let Some(mut p) = progress {
            fill_progress(w, &gm, &mut p);
        }
    }
    if let Some(period) = trace_period()
        && st.ticks % ((period * 24.0) as u64).max(1) == 0
        && let Some(w) = st.world.as_ref()
    {
        trace(w, st.ticks as f32 * TICK, session.race_time);
    }
}

/// what game code reads from the race manager (GameManager 0x000BDB50 / 0x000BC5F0 / 0x000BE430, post race).
/// Leaves `results_busy` / `car_flags` (OSD, CarDamage) alone.
fn fill_progress(w: &AiWorld, gm: &GameManager, p: &mut RaceProgress) {
    let rm = &w.race;
    // VsAIRaceVehicle +0xF0 (0x00019350 in PhysicsManager 0x0019E080): the race's competitor slots in order, the
    // human ones to the players, the rest to the ai cars
    let (mut humans, mut others) = (Vec::new(), Vec::new());
    if let Some(ev) = gm.current_event()
        && let Some(r) = ev.current_race()
    {
        let comps = ev.competitors.as_deref().unwrap_or(&[]);
        for &c in r.competitors.iter().take(r.num_competitors as usize) {
            if c.and_then(|i| comps.get(i)).is_some_and(|x| x.human != 0) {
                humans.push(c);
            } else {
                others.push(c);
            }
        }
    }
    let slot = |r: VehicleRef| match r {
        VehicleRef::Player(i) => humans.get(i).copied().flatten(),
        VehicleRef::Ai(i) => others.get(i).copied().flatten(),
        _ => None,
    };
    // VsAIWorld::GetVehicleInPosition 0x00034770: RaceInfo.pVehicles[place - 1]
    p.order = rm.race_order.iter().map(|&r| slot(r)).collect();
    p.order_ids = rm.race_order.iter().filter_map(|&r| rm.vehicle(r).map(|v| v.id)).collect();
    p.player_position = rm.player_positions.first().copied().unwrap_or(0);
    // VsAIRaceVehicle.cpp 0x00023CE0: +0x260 (6 = no failure)
    p.player_state = rm.players.first().map_or(crate::ai::vs_ai_race_vehicle::FAILURE_NONE, |v| v.failure);
    // VsAIWorld__37D80 (+0x40 racing count) and GetRaceLength 0x000348C0 (VsRaceManager 0x0003DCA0)
    p.racers = rm.num_racing();
    p.active_racers = rm.get_race_length_count();
    // VsRaceManager 0x0003DB70 (last active car in the order), 0x00015D30 (+0x34C)
    p.leader_human = rm.race_order.iter().rev().filter_map(|&r| rm.vehicle(r)).find(|v| v.base.active).is_some_and(|v| v.is_player);
    // 0x000BE430: the players' record blocks 0x000244E0, +0x2C and +0x50
    p.racer_records = rm
        .players
        .iter()
        .map(|v| (crate::ai::vs_ai_race_vehicle::stat_f(&v.stats, 11), v.stats[20]))
        .collect();
}

/// `VELOCITY_AI_TRACE=<s>`: one log block per period with every car's ai state (review aid, not original).
fn trace_period() -> Option<f32> {
    static P: std::sync::OnceLock<Option<f32>> = std::sync::OnceLock::new();
    *P.get_or_init(|| std::env::var("VELOCITY_AI_TRACE").ok().and_then(|v| v.parse().ok()).filter(|p: &f32| *p > 0.0))
}

fn trace(w: &AiWorld, t: f32, race_t: f32) {
    let order: Vec<String> = w.race.race_order.iter().map(|r| format!("{r:?}")).collect();
    log::info!("aitrace t {t:.1} race {race_t:.1} order [{}]", order.join(" "));
    for (k, v) in w.race.players.iter().map(|v| ("P", v)).chain(w.race.ai.iter().map(|v| ("A", v))) {
        let p = &v.base.position;
        log::info!(
            "aitrace {k}{} {:?} pos {:.0},{:.0} spd {:.1} dist {:.0} lap {} rp {} lat {:.1} sec {:?} fail {} stuck {:.1} off {}/{:.1}/{:.1} side {} slip {:?} rival {} dmg {:.0} hc {}",
            v.id, v.state, p.x, p.z, v.base.speed, v.race_distance, v.lap, crate::ai::vs_ai_race_vehicle::stat_position(&v.stats), v.lateral, v.race_section,
            v.failure, v.stuck_time, v.offset_active as u8, v.offset_current, v.offset_target, v.overtake_side, v.slipstream_target, v.rival_state,
            v.base.damage, v.human_control
        );
    }
    if let Some(tm) = w.traffic.as_ref() {
        let active = tm.vehicles.iter().filter(|v| v.base.active).count();
        let blocked: String =
            w.world.spline.as_ref().map_or(String::new(), |s| s.traffic_blockers.iter().map(|b| if b.flag != 0 { '1' } else { '0' }).collect());
        log::info!("aitrace traffic active {active}/{} (num_active {}) blockers on {blocked}", tm.vehicles.len(), tm.num_active);
    }
    if let Some(pm) = w.police.as_ref() {
        log::info!("aitrace police {:?} target {:?} wanted {:.2} active {}", pm.state, pm.target, pm.wanted, pm.num_active);
        for v in &pm.vehicles {
            let b = if v.mode == crate::ai::vs_ai_police_vehicle::PoliceVehicleMode::Race { &v.race.base } else { &v.traffic.base };
            log::info!(
                "aitrace cop {} {:?} act {} {:?} pos {:.0},{:.0} spd {:.1} siren {}",
                v.race.id, v.mode, b.active, v.race.state, b.position.x, b.position.z, b.speed, v.siren
            );
        }
    }
}

/// no cars: lets the AI build its managers before the physics crate provides any.
struct NullPhysics;

impl crate::ai::vs_ai_tools::AiCollisionQuery for NullPhysics {
    fn line_check(&self, _s: &VsVector4, _e: &VsVector4) -> Option<f32> {
        None
    }
    fn line_check_normal(&self, _s: &VsVector4, _e: &VsVector4, _mask: u32) -> Option<(f32, VsVector4)> {
        None
    }
}

impl AiPhysicsWorld for NullPhysics {
    fn vehicle(&mut self, _h: usize) -> &mut dyn crate::ai::vs_ai_vehicle::AiPhysicsVehicle {
        unreachable!("no physics cars yet")
    }
    fn vehicle_ref(&self, _h: usize) -> &dyn crate::ai::vs_ai_vehicle::AiPhysicsVehicle {
        unreachable!("no physics cars yet")
    }
    fn steer_angle_to(&self, _h: usize, _t: &VsVector4) -> f32 {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use velocity_game::main_cpp::CommandLine;

    /// jumping into a race builds the ai world from the core's load steps.
    #[test]
    fn jumped_race_loads_ai_world() {
        if crate::game_dir().is_none() {
            return;
        }
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        velocity_game::game_mode::game_manager::build_with(
            &mut app,
            CommandLine::parse(&["jumptogame".into(), r"TRACKS\LADOWN\".into(), r"CIRCUIT\RACE01\".into()]),
        );
        build(&mut app);
        let tick = 1_000_000_000 / 24 + 1;
        for i in 0..10u64 {
            app.world_mut().resource_mut::<velocity_game::vs_common_main::GameClock>().manual = Some(i * tick);
            app.update();
        }
        let st = app.world().resource::<AiState>();
        let w = st.world.as_ref().expect("ai world");
        assert!(w.world.get_num_race_sections() > 0);
        assert!(!w.world.handlings.is_empty());
        assert_eq!(w.traffic.is_some(), w.world.traffic_enabled);
        // the event race's competitors (+0x15) are the humans plus the ai cars (VsRaceManager 0x0003AD10)
        assert_eq!((w.race.ai.len() + w.race.players.len()) as u8, st.event.num_opponents);
        assert!(!st.game.grid.is_empty());
        eprintln!(
            "ai world: {} race sections, {} traffic sections, traffic {}, police {}, {} ai cars, ticks {}",
            w.world.get_num_race_sections(),
            w.world.get_num_traffic_sections(),
            w.traffic.is_some(),
            w.police.is_some(),
            w.race.ai.len(),
            st.ticks
        );
    }

    /// with the physics plugin: the race's physics cars are bound to the ai's race cars and the ai drives
    /// (or hands the pad to) them; the player's car moves once the race runs.
    #[test]
    fn jumped_race_binds_physics_cars() {
        if crate::game_dir().is_none() {
            return;
        }
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin, velocity_physics::plugin::PhysicsPlugin));
        velocity_game::game_mode::game_manager::build_with(
            &mut app,
            CommandLine::parse(&["jumptogame".into(), r"TRACKS\LONDON\".into(), r"CIRCUIT\RACE01\".into()]),
        );
        build(&mut app);
        let tick = 1_000_000_000 / 24 + 1;
        let mut bound_at = None;
        for i in 0..1200u64 {
            app.world_mut().resource_mut::<velocity_game::vs_common_main::GameClock>().manual = Some(i * tick);
            app.update();
            let st = app.world().resource::<AiState>();
            if bound_at.is_none() && st.world.as_ref().is_some_and(|w| w.race.players.first().is_some_and(|p| p.base.physics.is_some())) {
                bound_at = Some(i);
            }
        }
        let pos = app.world().resource::<PhysicsWorldRes>().lock().as_mut().map(|p| (p.position(0), p.ai_speed(0)));
        let st = app.world().resource::<AiState>();
        let w = st.world.as_ref().expect("ai world");
        let session = app.world().resource::<RaceSession>();
        eprintln!("player bound at update {bound_at:?}, session {:?}, pending reset {:?}, active {}, human {}",
            session.state, st.pending_reset, w.race.players[0].base.active, w.race.players[0].human_control);
        eprintln!("player car {pos:?}, ai state {:?}, race distance {}", w.race.players[0].state, w.race.players[0].race_distance);
        assert_eq!(w.race.players[0].base.physics, Some(0));
        // the barrier walls went to the physics world once it existed
        assert!(st.pending_walls.is_empty(), "walls not sent");
        assert!(app.world().resource::<PhysicsWorldRes>().lock().as_ref().is_some_and(|w| w.walls.barrier.is_some()));
        assert!(app.world().get_resource::<AiPhysics>().is_some_and(|p| p.0.ready()));
    }

    #[test]
    fn team_requests_reach_the_view() {
        use crate::team_database::Team;
        let mut db = TeamDatabase::default();
        for crc in [10, 20] {
            db.teams.push(Team { crc, vehicle_crc: crc + 1, flags: 0x20, ..Default::default() });
        }
        let mut view = CareerTeams::default();
        mirror_teams(&db, &mut view);
        // original list order = reverse of load order
        assert_eq!(view.teams.iter().map(|t| t.crc).collect::<Vec<_>>(), [20, 10]);
        assert!(apply_team_request(&mut db, TeamRequest::ClearNewlyUnlocked(10)));
        assert!(apply_team_request(&mut db, TeamRequest::SetJoinedOnce(20)));
        assert!(!apply_team_request(&mut db, TeamRequest::SetJoinedOnce(99)));
        mirror_teams(&db, &mut view);
        assert!(!view.find_team(10).unwrap().is_newly_unlocked());
        assert!(view.find_team(20).unwrap().joined_once_already() && view.find_team(20).unwrap().is_newly_unlocked());
        assert_eq!(view.find_team(20).unwrap().car, 21);
    }
}
