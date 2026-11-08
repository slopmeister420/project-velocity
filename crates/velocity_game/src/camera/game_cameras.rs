//! glue (not an original file): the camera objects GameManager owns or reaches through globals
//! (`g_CameraMan`, `g_CrashCameraManager`, the point of interest, `g_ReplayCameraManager`, `g_Markers`) as one
//! resource, set up by the core-owned load steps of InGameInit (GameManager.cpp 0x000B5F80):
//! "Markers Init", "Camera Manager", "Free Cameras", "3d Interface", "Crash Cam", "Fly Cameras",
//! "Replay Camera Manager". Per update (InGameUpdate order): crash cameras, point of interest, replay cameras
//! (attract / review only), the post race camera, then the camera manager with the player cars' follow cameras.

use bevy::prelude::*;

use super::vs_camera_manager::{ECameraMode, VsCameraManager};
use super::vs_crash_camera_manager::VsCrashCameraManager;
use super::vs_vehicle_follow_camera_point_of_interest::{PoiEvent, PointOfInterestTweaks, VsVehicleFollowCameraPointOfInterest};
use crate::game_mode::race_session::OsdCinematicBars;
use crate::controller::vibrate_effects::CMarkers;
use crate::game_data_loading::game_data_loading::ELoadStep;
use crate::game_mode::game_manager::{GameManager, LoadStepMessage};
use crate::game_mode::update_thread::CWorldModeManager;
use crate::replays::cameras::replay_camera_manager::{CReplayCameraManager, PlayerHumanControl, ReplayEvent};
use crate::game_mode::race_cars::{PlayerCarChoice, RaceCar};
use crate::game_mode::race_session::{PostRaceSequenceRes, RaceSession, RaceSessionMessage};
use crate::replays::cameras::ReplayWorld;
use crate::camera::vs_crash_camera::{CrashCameraEvent, CrashCameraWorld};
use crate::camera::vs_vehicle_follow_camera::camera_point::{camera_point, CameraPointIn, Cvi, CviState};
use crate::camera::vs_vehicle_follow_camera::{
    FollowExtras, FollowTarget, ImpactShake, PoiBlend, VsVehicleFollowCamera, PAD_LOOK_BACK,
};
use crate::controller::user_input::{EAction, UserInput};
use crate::game_bridge::camera_bridge::CameraBridgeData;
use crate::game_input::game_input::GameInput;
use crate::triggers::trigger_update::CTriggers;
use crate::vs_common_main::GameClock;
use velocity_physics::plugin::{PhysicsCar, PhysicsCarInput, PhysicsCarState, PhysicsWorldRes};
use velocity_physics::vehicle::vehicle_entity::sim::{RAY_CARS, RAY_FIRST_HIT, RAY_NORMAL, RAY_OBJECTS, RAY_TRACK};
use std::cell::RefCell;
use visos_render::compartment_manager::StreamView;
use visos_render::track_render::TrackStreamViews;

#[derive(Resource, Default)]
pub struct GameCameras {
    pub manager: VsCameraManager,
    pub crash: VsCrashCameraManager,
    pub poi: VsVehicleFollowCameraPointOfInterest,
    pub poi_tweaks: PointOfInterestTweaks,
    pub replay: CReplayCameraManager,
    pub markers: CMarkers,
}

/// the camera load steps, in the order InGameInit runs them
fn camera_load_steps(
    mut r: MessageReader<LoadStepMessage>,
    mut cams: ResMut<GameCameras>,
    gm: Res<GameManager>,
    choice: Res<PlayerCarChoice>,
    career: Option<Res<crate::game_mode::career::Career>>,
) {
    for m in r.read() {
        let Some(p) = &m.paths else { continue };
        let race = gm.race.as_ref();
        let players = race.map(|r| r.num_players.max(1) as usize).unwrap_or(1);
        // `byte_6F7157` (two players), not the dead `splitscreen` command line flag (GameVars 0x000C2EC0 clears it)
        let split = players > 1;
        let network = gm.vars.network_race;
        match m.step {
            // VibrateEffects.cpp 0x001EDFE0
            ELoadStep::MarkersInit => cams.markers.load(&p.race_dir),
            // VsCameraManager.cpp 0x0005ECC0: each player's car view table
            ELoadStep::CameraManager => {
                let table: Vec<[f32; 20]> = if choice.camera_views.is_empty() {
                    choice.camera_view.into_iter().collect()
                } else {
                    choice.camera_views.clone()
                };
                // 0x0005F0F0 on player p's vehicle (`dword_6F724C[139 * p]`): data +8 list a with one player, else +12
                // list b side by side (`byte_6F9180`) / +16 list c top and bottom
                let vertical = career.as_ref().is_some_and(|c| c.options.time_format != 0);
                let tables = (0..players)
                    .map(|p| {
                        let setup = if p == 1 { choice.setup2.as_ref().or(choice.setup.as_ref()) } else { choice.setup.as_ref() };
                        let v = career.as_ref().and_then(|c| c.vehicles.as_ref()).zip(setup.and_then(|s| s.vehicle)).and_then(|(db, v)| db.vehicles.get(v));
                        let list = match v {
                            Some(v) if split => if vertical { v.data.list_b.clone() } else { v.data.list_c.clone() },
                            Some(v) if p > 0 => v.data.list_a.clone(),
                            _ => Vec::new(),
                        };
                        if list.is_empty() { table.clone() } else { list }
                    })
                    .collect();
                cams.manager.init(tables, split);
                // 0x0005ECC0 then VsFlyCamera 0x00065610 with the vehicle database's GLOBALS.BIN record when loaded
                // (shipped: 120 / 90, the .data values)
                if let Some(g) = career.as_ref().and_then(|c| c.vehicles.as_ref()).and_then(|db| db.globals.as_ref()) {
                    cams.manager.globals.set_turn_rates(&g.values);
                }
            }
            // GameManager.cpp 0x000B7730 / 0x000B75D0: the roaming and 3d interface cameras become the view
            // in turn while they are set up
            ELoadStep::FreeCameras => {
                cams.manager.create_debug_cameras();
                for i in 0..players {
                    cams.manager.set_mode(ECameraMode::Roaming, i);
                }
            }
            ELoadStep::Interface3dCamera => {
                for i in 0..players {
                    cams.manager.set_mode(ECameraMode::Interface3d, i);
                }
            }
            // VsCrashCamera.cpp 0x000630E0 with the players' fly cameras (0x00063580)
            ELoadStep::CrashCam => {
                let n = players as u32;
                cams.crash.init(n);
            }
            // GameManager.cpp: network races start on the follow cameras, others on the intro (0x000B73D0)
            ELoadStep::FlyCameras => {
                if network {
                    for i in 0..players {
                        cams.manager.set_mode(ECameraMode::Follow, i);
                    }
                } else {
                    let grid: Vec<Vec3> = gm
                        .event_def
                        .as_ref()
                        .map(|d| d.start_positions.iter().map(|q| Vec3::new(q[0], q[1], q[2])).collect())
                        .unwrap_or_default();
                    cams.manager.start_intro(&grid, split);
                }
            }
            // ReplayCamera_Fixed.cpp 0x00165400: the current race's track and race paths
            ELoadStep::ReplayCameraManager => {
                if let Some(r) = race {
                    cams.replay.init(&r.track_path, &r.race_path);
                }
            }
            _ => {}
        }
    }
}

/// a view that replaces the camera manager's this update (replay cameras, the post race camera)
#[derive(Resource, Default, Clone, Debug)]
pub struct OverrideView(pub Option<CameraBridgeData>);

/// the cars and track as the camera code sees them
struct CamWorld<'a> {
    cars: Vec<(u32, PhysicsCarState)>,
    physics: RefCell<&'a mut PhysicsWorldRes>,
    race_time: f32,
    camera: Vec3,
}

impl CamWorld<'_> {
    fn car(&self, v: u32) -> Option<&PhysicsCarState> {
        self.cars.iter().find(|(s, _)| *s == v).map(|(_, c)| c)
    }
    fn hit(&self, from: Vec3, to: Vec3) -> Option<f32> {
        self.physics.borrow_mut().ray_cast(from, to, RAY_TRACK | RAY_FIRST_HIT, -1).map(|h| h.t)
    }
}

impl ReplayWorld for CamWorld<'_> {
    fn car_position(&self, v: u32) -> Vec3 {
        self.car(v).map(|c| c.position).unwrap_or_default()
    }
    fn wheel_position(&self, v: u32, w: u32) -> Vec3 {
        self.car(v).map(|c| c.wheel_matrix(w as usize).w_axis.truncate()).unwrap_or_default()
    }
    fn car_axes(&self, v: u32) -> (Vec3, Vec3, Vec3) {
        let r = self.car(v).map(|c| c.rotation).unwrap_or_default();
        (r * Vec3::X, r * Vec3::Y, r * Vec3::Z)
    }
    /// ReplayCamera_Orbit 0x00167680: MaterialProperties 0x0019D350(.., 1, vehicle) -> RayCast flags 23, the car
    /// itself ignored
    fn blocked(&self, from: Vec3, to: Vec3, v: u32) -> bool {
        self.physics.borrow_mut().ray_cast(from, to, RAY_TRACK | RAY_OBJECTS | RAY_CARS | RAY_NORMAL, v as i32).is_some()
    }
}

impl CrashCameraWorld for CamWorld<'_> {
    fn race_time(&self) -> f32 {
        self.race_time
    }
    fn camera_position(&self, _p: u32) -> Vec3 {
        self.camera
    }
    fn ray_fraction(&self, from: Vec3, to: Vec3, _v: u32) -> Option<f32> {
        self.hit(from, to)
    }
}

/// VsCrashCamera.cpp 0x00062D90's VsAIRaceVehicle 0x000208D0(crashed car, `flt_6850F0`): written in the CrashCam set
/// (and from the ai's crash camera reset call); the binary forwards it as the ai's `AiRespawnOnRaceline` before the Ai set
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct CrashCameraRespawn {
    pub vehicle: u32,
    pub speed: f32,
}

/// the crash cameras' calls out: camera mode + cuts, the respawn, and the cinematic bars (VsCrashCamera 0x00062C10 / 0x00062D90 ->
/// OSDManager 0x000ED6E0)
fn apply_crash_events(
    c: &mut GameCameras,
    ev: Vec<CrashCameraEvent>,
    bars: &mut MessageWriter<OsdCinematicBars>,
    respawn: &mut MessageWriter<CrashCameraRespawn>,
) {
    for e in ev {
        match e {
            CrashCameraEvent::Respawn { vehicle, kick_speed } => {
                respawn.write(CrashCameraRespawn { vehicle, speed: kick_speed });
            }
            CrashCameraEvent::SetCameraMode { player, mode } => {
                let m = if mode == 3 { ECameraMode::Cine } else { ECameraMode::Follow };
                c.manager.set_mode(m, player as usize);
                // VsCrashCamera 0x00062C10 / 0x00062D90: then 0x00060380(1) and 0x0002C620(0)
                c.manager.render_cut = true;
                if let Some(pc) = c.manager.players.get_mut(0) {
                    pc.cut = true;
                }
            }
            CrashCameraEvent::CinematicBars(on) => {
                bars.write(OsdCinematicBars(on));
            }
        }
    }
}

/// the point of interest's calls out: cinematic bars (0x0006A720 / 0x0006A7B0 -> OSDManager 0x000ED6E0)
fn apply_poi_events(pe: Vec<PoiEvent>, bars: &mut MessageWriter<OsdCinematicBars>) {
    for e in pe {
        if let PoiEvent::CinematicBars(on) = e {
            bars.write(OsdCinematicBars(on));
        }
    }
}

/// "g_CrashCameraManager.Update" (CrashCam set), point of interest (Ai set)
fn crash_and_poi(
    mut cams: ResMut<GameCameras>,
    mut wmm: ResMut<CWorldModeManager>,
    mut physics: ResMut<PhysicsWorldRes>,
    session: Res<RaceSession>,
    clock: Res<GameClock>,
    cars: Query<(&RaceCar, &PhysicsCarState)>,
    mut bars: MessageWriter<OsdCinematicBars>,
    mut respawn: MessageWriter<CrashCameraRespawn>,
) {
    let w = CamWorld {
        cars: cars.iter().map(|(r, s)| (r.slot as u32, s.clone())).collect(),
        physics: RefCell::new(&mut physics),
        race_time: session.race_time,
        camera: cams.manager.view(0).map(|v| v.position).unwrap_or_default(),
    };
    let c = &mut *cams;
    let mut ev = Vec::new();
    c.crash.update(&w, &mut wmm, &mut ev);
    apply_crash_events(c, ev, &mut bars, &mut respawn);
    let mut pe = Vec::new();
    let tw = c.poi_tweaks.clone();
    c.poi.update(clock.timer.scaled_dt(), &tw, &mut wmm, session.race_time, &mut pe);
    apply_poi_events(pe, &mut bars);
}

/// camera calls the ai makes inside its update (`AiHost`; VsAITrafficVehicle 0x0002B800 / 0x0002BB30 on a respawn),
/// in call order. The binary installs the ai host that records them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AiCameraCall {
    /// VsAIVehicle 0x0002C620 on `dword_6D55C8`: player record `+44*id+40` = 1 (follow camera cut); `id` = race car +0x78
    Cut { id: u32 },
    /// VsCrashCameraManager 0x00063BA0 for the physics car's pad slot (race car +0x78 when < 4)
    CrashCameraReset { physics: usize },
    /// VsVehicleFollowCameraPointOfInterest 0x00069BE0 with no vehicle
    PointOfInterestClear { priority: f32 },
}

/// shared with the binary's `AiHost` (which runs inside the ai's systems)
#[derive(Resource, Clone, Default)]
pub struct AiCameraCalls(pub std::sync::Arc<std::sync::Mutex<Vec<AiCameraCall>>>);

/// applies the ai update's camera calls right after the Ai set, before the physics frame (the original makes them
/// inside the ai update)
#[allow(clippy::too_many_arguments)]
fn apply_ai_camera_calls(
    calls: Res<AiCameraCalls>,
    mut cams: ResMut<GameCameras>,
    mut wmm: ResMut<CWorldModeManager>,
    mut physics: ResMut<PhysicsWorldRes>,
    session: Res<RaceSession>,
    attract: Res<crate::font_helper::CAttractMode>,
    cars: Query<(&RaceCar, &PhysicsCarState, Option<&PhysicsCar>)>,
    mut bars: MessageWriter<OsdCinematicBars>,
    mut respawn: MessageWriter<CrashCameraRespawn>,
) {
    let list: Vec<AiCameraCall> = match calls.0.lock() {
        Ok(mut v) => v.drain(..).collect(),
        Err(_) => return,
    };
    if list.is_empty() {
        return;
    }
    let slot_of = |physics: usize| cars.iter().find(|(_, _, pc)| pc.is_some_and(|p| p.id == physics)).map(|(r, _, _)| r.slot as u32);
    let w = CamWorld {
        cars: cars.iter().map(|(r, s, _)| (r.slot as u32, s.clone())).collect(),
        physics: RefCell::new(&mut physics),
        race_time: session.race_time,
        camera: cams.manager.view(0).map(|v| v.position).unwrap_or_default(),
    };
    let c = &mut *cams;
    for call in list {
        match call {
            AiCameraCall::Cut { id } => {
                // the original writes the record of any id (an opponent's lands past the players' records); only the
                // players' records exist here
                if let Some(pc) = c.manager.players.get_mut(id as usize) {
                    pc.cut = true;
                }
            }
            AiCameraCall::CrashCameraReset { physics } => {
                if let Some(slot) = slot_of(physics).filter(|s| *s < 4) {
                    let mut ev = Vec::new();
                    c.crash.skip(slot, &w, &mut wmm, &mut ev);
                    apply_crash_events(c, ev, &mut bars, &mut respawn);
                }
            }
            AiCameraCall::PointOfInterestClear { priority } => {
                let tw = c.poi_tweaks.clone();
                let mut pe = Vec::new();
                c.poi.set(None, 0.0, priority, &tw, attract.playing, &mut wmm, session.race_time, &mut pe);
                apply_poi_events(pe, &mut bars);
            }
        }
    }
}

/// replay cameras (attract mode / review only), the post race camera, then "g_CameraMan.Update" with the
/// players' follow cameras on their cars
#[allow(clippy::too_many_arguments)]
fn update_cameras(
    mut cams: ResMut<GameCameras>,
    mut physics: ResMut<PhysicsWorldRes>,
    mut view: ResMut<OverrideView>,
    mut rand: Local<visos::handy::crt_rand::CrtRand>,
    session: Res<RaceSession>,
    prs: Res<PostRaceSequenceRes>,
    triggers: Res<CTriggers>,
    clock: Res<GameClock>,
    mut attract_ai: (Res<crate::font_helper::CAttractMode>, MessageWriter<PlayerHumanControl>),
    ui: Res<UserInput>,
    gi: Res<GameInput>,
    cars: Query<(&RaceCar, &PhysicsCarState)>,
    (fg, gm): (Res<crate::game_mode::race_session::FlagGirl>, Res<GameManager>),
    inputs: Query<(&RaceCar, &PhysicsCar, Option<&PhysicsCarInput>)>,
    (fury_fx, trails): (Option<Res<visos_render::fury_fx::FuryFx>>, Res<crate::game_mode::trail_blazer::PlayerTrailBlazers>),
    mut wmm: ResMut<CWorldModeManager>,
) {
    let dt = clock.timer.scaled_dt();
    // the flag girl intro's current camera drives the players' fly cameras (VsFlyCamera mode 1)
    let intro = fg.0.as_ref().filter(|f| f.cine).and_then(|f| {
        let c = &f.peer.cameras[f.peer.current_camera?];
        Some(crate::camera::vs_fly_camera::TimelineCameraOutput {
            position: c.position,
            target: c.target,
            up: c.up,
            fov_deg: c.fov,
            cut: f.peer.camera_cut,
        })
    });
    let w = CamWorld {
        cars: cars.iter().map(|(r, s)| (r.slot as u32, s.clone())).collect(),
        physics: RefCell::new(&mut physics),
        race_time: session.race_time,
        camera: Vec3::ZERO,
    };
    let c = &mut *cams;
    view.0 = None;
    let state = session.state as u32;
    // InGameUpdate 0x000B7D20: a change of "Enable Cameras For Review" (`dword_749110` against `dword_749114`)
    // hands player 0's race vehicle to the ai (0 -> 1) or back (from 1), then cuts player 0's camera (0x0002C620(0))
    let (attract, ai_control) = (attract_ai.0.playing, &mut attract_ai.1);
    if c.replay.review_cameras != c.replay.review_cameras_prev {
        if c.replay.review_cameras_prev == 1 {
            ai_control.write(PlayerHumanControl { player: 0, human: true });
        }
        if c.replay.review_cameras_prev == 0 && c.replay.review_cameras == 1 {
            ai_control.write(PlayerHumanControl { player: 0, human: false });
        }
        if let Some(pc) = c.manager.players.get_mut(0) {
            pc.cut = true;
        }
    }
    c.replay.review_cameras_prev = c.replay.review_cameras;
    if attract || c.replay.review_cameras != 0 || c.replay.review {
        // 0x00165A30 is passed the update time (0x000BF690), 0x00165DD0 the update dt (0x000BF670)
        let mut ev = Vec::new();
        c.replay.update(clock.timer.update_time(), state, &triggers, &w, &mut rand, &mut ev);
        view.0 = c.replay.apply(dt, state, &w);
        for e in ev {
            match e {
                // MessageManager message 3 (CameraCut; its listeners are not ported)
                ReplayEvent::CameraChanged => log::debug!("replay camera: {:?} on car {}", c.replay.camera, c.replay.vehicle),
            }
        }
    } else {
        // 0x00166630(0)
        c.replay.updating = false;
    }
    // "g_PostRaceSequence.ApplyCurrentCamera"
    if let Some(v) = prs.0.camera_view() {
        view.0 = Some(v);
    }
    // CameraChange cycles the car views (0x0005F140)
    for p in 0..c.manager.players.len() {
        if ui.pressed(EAction::CameraChange, gi.device_mask(p)) {
            c.manager.change_view(p, -1);
        }
    }
    // VsFlyCamera 0x00064090: VsAITools::LineCheck at the camera's x/z from grid slot 0's y (`dword_6F5C0C`) + 5
    // to - 5 (a 10 m segment; was a 1000 m cast down from the camera's relative y: no hit and 27 s calls)
    let grid_y = gm.event_def.as_ref().and_then(|d| d.start_positions.first()).map_or(0.0, |q| q[1]);
    let ground = |p: Vec3| {
        let (a, b) = (Vec3::new(p.x, grid_y + 5.0, p.z), Vec3::new(p.x, grid_y - 5.0, p.z));
        w.physics.borrow_mut().ray_cast(a, b, RAY_TRACK | RAY_NORMAL, -1).map(|h| h.point[1])
    };
    let roam = |a: EAction| ui.value(a, gi.device_mask(0));
    let mut timeline: Vec<_> = (0..c.manager.players.len()).map(|_| intro.clone()).collect();
    c.manager.update(&mut timeline, &ground, &roam);
    // the follow cameras: VehicleEntity 0x001C5CF0 per player car, skipped while the replay cameras update
    // (ReplayCamera_Manager 0x00166650 = +0x10), review cameras are on (`dword_749110`) or the post race camera
    // runs (PostRaceSequence 0x000C4650)
    if c.replay.updating || c.replay.review_cameras != 0 || prs.0.camera_active() {
        return;
    }
    let res = w.physics.borrow_mut();
    let mut lock = res.lock();
    let Some(world) = lock.as_mut() else { return };
    // 0x00066940: VsAIDriver__15D50(&unk_6F5BC8) = GameManager +0x208, the race time
    let game_time = session.race_time;
    for p in 0..c.manager.players.len() {
        let Some((_, car, input)) = inputs.iter().find(|(r, _, _)| r.slot == p && r.human) else { continue };
        let id = car.id;
        if world.vehs.get(id).is_none_or(|v| v.is_none()) {
            continue;
        }
        let mut pad = input.map(|i| i.pad()).unwrap_or_default();
        if ui.value(EAction::CarRearView, gi.device_mask(p)) > 0.5 {
            pad.buttons |= PAD_LOOK_BACK;
        }
        // FuryFX 0x00128680(camera +580 = p) on the render's instance state (`byte_71A0D0`), and its "Fov Change"
        // tweak `dword_686B5C` (0x000690B0)
        let fury = fury_fx.as_deref().map_or(0.0, |f| f.amount(p));
        let fury_fov = fury_fx.as_deref().map_or(crate::camera::vs_vehicle_follow_camera::k::FOV_FURY, |f| f.tweaks.fov_change);
        let poi = c.poi.object.and_then(|o| {
            let o = o as usize;
            world.vehs.get(o).and_then(|v| v.as_ref())?;
            let d = c.poi_tweaks.extra_fov.then_some([
                c.poi_tweaks.min_fov_distance,
                c.poi_tweaks.max_fov_distance,
                c.poi_tweaks.extra_fov_at_max,
            ]);
            Some(PoiBlend { target: Vec3::from(world.position(o)), t: c.poi.t, out: c.poi.out, fov_distances: d })
        });
        let in_car = c.manager.is_in_car_view(p);
        let tweaks = c.manager.camera_tweaks;
        let g = c.manager.globals;
        // a8: this update's car bridge record (written by `car_outputs` in the Physics set), model +Z = body +X
        let bridge_forward = cars.iter().find(|(r, _)| r.slot == p && r.human).map(|(_, s)| s.rotation * Vec3::X);
        let pc = &mut c.manager.players[p];
        let Some(f) = pc.follow.as_mut() else { continue };
        let (mut t, x) = follow_inputs(world, id, f, game_time);
        t.pad = pad;
        t.fury = fury;
        t.fury_fov = fury_fov;
        t.in_zone = pc.in_zone;
        t.look_around = ui.value(EAction::CameraRotateHoriz, 0xFFFF);
        t.poi = poi;
        t.in_car_view = in_car;
        t.one_off = trails.0.get(p).copied().flatten();
        if let Some(b) = bridge_forward {
            t.bridge_forward = b;
        }
        let x = FollowExtras { poi_active: c.poi.is_active(), tweaks, ..x };
        let mut hit = |a: Vec3, b: Vec3| world.line_check(a.to_array(), b.to_array(), false, -1).map(|h| h.0);
        let out = f.update_follow(dt, &g, &t, &x, &mut pc.cut, &mut hit, &mut |lo, hi| {
            crate::timeline_manager::objects::timeline_camera::vs_rand_range(&mut rand, lo, hi)
        });
        // VehicleEntity 0x001C5CF0 then runs the mirror camera (VE+4936) on the same inputs; it reads the VE+5008
        // the follow camera just wrote
        let mut outs = vec![out];
        if let Some(m) = pc.mirror.as_mut() {
            let mut xm = x;
            if let (Some(i), Some(a)) = (xm.impact.as_mut(), out.impact_amount) {
                i.amount = a;
            }
            outs.push(m.update_follow(dt, &g, &t, &xm, &mut pc.cut, &mut hit, &mut |lo, hi| {
                crate::timeline_manager::objects::timeline_camera::vs_rand_range(&mut rand, lo, hi)
            }));
        }
        for out in outs {
            if let Some(a) = out.impact_amount {
                if let Some(v) = world.vehs[id].as_mut() {
                    v.ve.m.set_f(5008, a);
                }
            }
            if out.poi_done {
                let tw = c.poi_tweaks.clone();
                let mut ev = Vec::new();
                c.poi.set(None, 0.0, 0.5, &tw, false, &mut wmm, session.race_time, &mut ev);
            }
        }
    }
}

/// the follow camera's inputs from the physics images, as 0x001C5CF0 / 0x00067510 / 0x00066940 read them; writes
/// the vertical inertia state back to VE+4436 / +4440 / +4444
fn follow_inputs(
    world: &mut velocity_physics::vehicle::vehicle_entity::sim::PhysicsWorld,
    id: usize,
    f: &VsVehicleFollowCamera,
    game_time: f32,
) -> (FollowTarget, FollowExtras) {
    let gc = &world.gc;
    let cvi = Cvi {
        inertia_reference_frame: gc.CVI_inertia_reference_frame,
        tau_up: gc.CVI_tau_car_up,
        tau_down: gc.CVI_tau_car_down,
        scale_up: gc.CVI_motion_scale_car_up,
        scale_down: gc.CVI_motion_scale_car_down,
        max_up: gc.CVI_max_motion_car_up,
        max_down: gc.CVI_max_motion_car_down,
        stop_decay: gc.CVI_motion_stop_decay_const,
        landing_tau: gc.PL_landing_camera_tau_multiplier,
        landing_limits: gc.PL_landing_camera_limits_multiplier,
    };
    let shift_forward = gc.camera_shift_forward;
    let up_offset = world.camera_up_offset(id);
    let position = Vec3::from(world.position(id));
    let velocity = Vec3::from(world.velocity(id));
    let s = world.slot(id);
    let ve = &s.ve.m;
    let row = |o: usize| Vec3::new(ve.f(o), ve.f(o + 4), ve.f(o + 8));
    let pin = CameraPointIn {
        position,
        heading: row(4788),
        up: row(4804),
        shift_forward,
        skid: ve.f(4456),
        skid_look: ve.f(4468),
        skid_right: ve.f(4476),
        up_offset,
        offset: (ve.i(4408) != 0).then(|| row(4396)),
        landing: ve.i(3840) != 0,
        lc0: s.ve.lc.f(0),
        ve_276: ve.f(276),
    };
    let mut st = CviState { rate: ve.f(4436), height: ve.f(4440), frame: ve.f(4444) };
    let mut t = FollowTarget::new(Vec3::ZERO, row(4788), velocity, ve.f(20));
    t.car_up = row(4804);
    t.extra_yaw = ve.f(4480);
    t.lag = ve.f(4484);
    t.trail_offset = ve.f(4504);
    t.pitch_offset = ve.f(4516);
    let car = &s.car.m;
    // 0x00364B20: any wheel's 0x00364A90 == 1 (as `race_effects_bridge::effect_car`)
    let sliding = (0..4).any(|i| {
        let contacts = car.i(1088 + 4 * i);
        (0..contacts.max(0) as usize).any(|k| car.i(676 + 136 * (i + k)) != 0)
    });
    let impact = Some(ImpactShake {
        amount: ve.f(5008),
        base: ve.f(5012),
        start: ve.f(5016),
        end: ve.f(5020),
        up: row(4804),
        side: row(4820),
    });
    let x = FollowExtras { sliding, game_time, impact, poi_active: false, tweaks: Default::default() };
    t.position = camera_point(&pin, &cvi, &mut st, f.look, f.right, f.view_index);
    if let Some(v) = world.vehs[id].as_mut() {
        v.ve.m.set_f(4436, st.rate);
        v.ve.m.set_f(4440, st.height);
        v.ve.m.set_f(4444, st.frame);
    }
    (t, x)
}

/// `RaceSessionMessage::CameraMode`: every player's camera (1 follow, 3 cine)
fn camera_modes(mut cams: ResMut<GameCameras>, mut r: MessageReader<RaceSessionMessage>) {
    for m in r.read() {
        if let RaceSessionMessage::CameraMode { mode } = m {
            let mode = if *mode == 3 { ECameraMode::Cine } else { ECameraMode::Follow };
            for p in 0..cams.manager.players.len() {
                cams.manager.set_mode(mode, p);
            }
        }
    }
}

/// the camera bridge records of this update (CameraBridge 0x00096C70): in the original every camera writes its
/// view into the bridge as it updates (VsVehicleFollowCamera 0x00066940, VsFlyCamera 0x00064790, the replay /
/// post race cameras, TimelineCamera 0x00178950 last), so the last writer of the update wins; the renderer then
/// draws the bridge interpolated between the last two updates (CameraBridge 0x00096A20). Here the same priority:
/// cut-scene camera, replay / post race view (player 0), else the manager's current camera; [1] = the mirror
/// camera's own record (0x00066940 writes +52 when +496 and +497). The renderer cut requests of this update
/// (VsCameraManager 0x00060380(1): view change 0x0005F140, the follow / mirror / fly cameras, TimelineCamera
/// 0x001781C0 begin and 0x001783C0 end, which also cuts player 0's next follow update via 0x0002C620(0)) are
/// moved to the bridge's cut flag here.
pub fn write_camera_bridge(
    mut cams: ResMut<GameCameras>,
    cut: Res<crate::timeline_manager::race_timeline_sequencer::TimelineCameraView>,
    other: Res<OverrideView>,
    mut timeline: MessageReader<crate::timeline_manager::objects::timeline_object::TimelineObjectMessage>,
    mut bridge: ResMut<crate::game_bridge::game_bridge::GameBridge>,
) {
    use crate::timeline_manager::objects::timeline_object::TimelineObjectMessage;
    let mut render_cut = std::mem::take(&mut cams.manager.render_cut);
    for pc in &mut cams.manager.players {
        for f in pc.follow.iter_mut().chain(pc.mirror.iter_mut()) {
            render_cut |= std::mem::take(&mut f.render_cut);
        }
    }
    for m in timeline.read() {
        if let TimelineObjectMessage::CutSceneCamera { active } = m {
            render_cut = true;
            if !*active {
                if let Some(pc) = cams.manager.players.get_mut(0) {
                    pc.cut = true;
                }
            }
        }
    }
    let replay_both = std::mem::take(&mut cams.replay.bridge_both);
    if !bridge.initialised {
        return;
    }
    if render_cut {
        bridge.cut = true;
    }
    // the replay cameras' cut (0x00164D70 / 0x001674E0 / 0x00167D60): their record also goes into the other
    // buffer (0x000969B0 swap, write, swap back), so view 0 does not lerp across the shot change; later writers
    // of this update (post race, timeline camera) overwrite only the current buffer
    if replay_both {
        let rec = cams.replay.bridge;
        bridge.camera_previous(0)[0] = rec;
    }
    let n = cams.manager.players.len().min(crate::game_bridge::game_bridge::CAMERA_VIEWS);
    for p in 0..n {
        let main = if p == 0 { cut.0.or(other.0) } else { None }.or_else(|| cams.manager.view(p));
        let mirror = cams.manager.players[p].mirror.as_ref().and_then(|f| f.bridge);
        let rec = bridge.camera(p);
        if let Some(v) = main {
            rec[0] = v;
        }
        if let Some(v) = mirror {
            rec[1] = v;
        }
    }
}

/// TimelineCameraShaker 0x0017A5F0 (`F3EventPlayerCameraShake` begin) → VehicleEntity 0x001D1610 on vehicle array
/// entry 0 with (Intensity, event length); the start time is the race time the follow camera decays against
fn player_camera_shake(
    mut timeline: MessageReader<crate::timeline_manager::objects::timeline_object::TimelineObjectMessage>,
    physics: Option<Res<PhysicsWorldRes>>,
    session: Res<RaceSession>,
) {
    use crate::timeline_manager::objects::timeline_object::TimelineObjectMessage;
    for m in timeline.read() {
        let TimelineObjectMessage::PlayerCameraShake { intensity, length } = *m else { continue };
        let Some(physics) = physics.as_ref() else { continue };
        let mut lock = physics.lock();
        let Some(world) = lock.as_mut() else { continue };
        if world.vehs.first().is_some_and(|v| v.is_some()) {
            world.set_camera_shake(0, intensity, length, session.race_time);
        }
    }
}

/// per rendered frame: what TrackFX 0x0014B030 reads about view 0 — its camera mode (0x0005FFB0) and the car the
/// camera sits in (CarRender 0x001127F0: the race car whose mode 1 follow camera (VehicleEntity 0x001C5780) or
/// mode 2 mirror camera (0x001C57C0) is view 0's and is not an outside view, `!VsVehicleFollowCamera 0x00069880` =
/// +528 == 0 || (in-car view 0x00060260 && point of interest running 0x00069810))
fn headlight_view(
    cams: Res<GameCameras>,
    cars: Query<(&RaceCar, &PhysicsCar)>,
    mirrors: Query<&visos_render::view_render::MirrorView>,
    mut view: ResMut<visos_render::track_fx::HeadlightView>,
    mut views: ResMut<visos_render::track_fx::HeadlightViews>,
) {
    let m = &cams.manager;
    let v = view_headlight(&cams, &cars, 0, m.mode(0));
    if view.camera_mode != v.camera_mode || view.inside != v.inside {
        *view = v;
    }
    // GameManager 0x000B9010: views after 0 run 0x001127F0 with their player; player 1 only while split
    // (`byte_6F7157`); the mirror view switches its player to mode 2 first (0x0005FFE0(2, p))
    let player1 = (m.players.len() > 1).then(|| view_headlight(&cams, &cars, 1, m.mode(1)));
    let mirror = mirrors.iter().any(|mv| mv.player == 0 && mv.on).then(|| view_headlight(&cams, &cars, 0, ECameraMode::Mirror));
    let set = |a: Option<visos_render::track_fx::HeadlightView>, b: Option<visos_render::track_fx::HeadlightView>| match (a, b) {
        (Some(a), Some(b)) => a.camera_mode == b.camera_mode && a.inside == b.inside,
        (None, None) => true,
        _ => false,
    };
    if !set(views.player1, player1) || !set(views.mirror, mirror) {
        views.player1 = player1;
        views.mirror = mirror;
    }
}

/// CarRender.cpp 0x001127F0 for view player `p` drawn in camera mode `mode`: the human race car of slot p whose
/// follow (mode 1, VehicleEntity 0x001C5780) / mirror (mode 2, 0x001C57C0) camera belongs to p (+580) and is not an
/// outside view (`!VsVehicleFollowCamera 0x00069880`; 0x00069810 = in-car view 0x00060260 of +580 && poi running)
fn view_headlight(
    cams: &GameCameras,
    cars: &Query<(&RaceCar, &PhysicsCar)>,
    p: usize,
    mode: ECameraMode,
) -> visos_render::track_fx::HeadlightView {
    let m = &cams.manager;
    let pc = m.players.get(p);
    let cam = match mode {
        ECameraMode::Follow => pc.and_then(|c| c.follow.as_ref()),
        ECameraMode::Mirror => pc.and_then(|c| c.mirror.as_ref()),
        _ => None,
    };
    let inside_view = cam.is_some_and(|f| {
        f.player as usize == p && !(!(m.is_in_car_view(p) && cams.poi.is_active()) && f.view.unknown_528 != 0.0)
    });
    let car = cars.iter().find(|(r, _)| r.slot == p && r.human && r.kind == crate::game_mode::race_cars::CarKind::Race);
    visos_render::track_fx::HeadlightView {
        camera_mode: mode as i32,
        inside: car.filter(|_| inside_view).map(|(_, c)| c.id as u64),
    }
}

fn reset_headlight_view(
    mut view: ResMut<visos_render::track_fx::HeadlightView>,
    mut views: ResMut<visos_render::track_fx::HeadlightViews>,
) {
    *view = Default::default();
    *views = Default::default();
}

/// "g_ReplayCameraManager.CheckForLoad" (GameManager InGameUpdate) — the file is read synchronously here, so
/// it parses on the first update
fn replay_check_for_load(mut cams: ResMut<GameCameras>) {
    cams.replay.check_for_load();
}

/// GameManager 0x000B4170 adds the 0x000C0A90 listener (`dword_6F5B98`) to MessageManager channel 1 in game,
/// 0x000B8550 removes it: every `TweakChanged` re-applies the clip globals. Runs right after the debug menu
/// (the original calls listeners synchronously); messages outside the game are consumed and dropped.
pub fn clip_tweaks_changed(
    mut cursor: Local<bevy::ecs::message::MessageCursor<crate::message::message_manager::GameMessage>>,
    messages: Option<Res<bevy::ecs::message::Messages<crate::message::message_manager::GameMessage>>>,
    mode: Option<Res<State<crate::EGameMode>>>,
    mut cams: ResMut<GameCameras>,
) {
    let Some(messages) = messages else { return };
    let n = cursor
        .read(&messages)
        .filter(|m| matches!(m, crate::message::message_manager::GameMessage::TweakChanged(_)))
        .count();
    if mode.is_some_and(|m| *m.get() == crate::EGameMode::InGame) {
        for _ in 0..n {
            cams.manager.clip_tweak_changed();
        }
    }
}

pub fn build(app: &mut App) {
    app.init_resource::<GameCameras>()
        .add_message::<PlayerHumanControl>()
        .add_message::<CrashCameraRespawn>()
        // InGameInit 0x000B6055: `dword_749114` = 0, `dword_749110` = it
        .add_systems(OnEnter(crate::InRace), |mut c: ResMut<GameCameras>| {
            c.replay.review_cameras_prev = 0;
            c.replay.review_cameras = 0;
        })
        .init_resource::<PlayerCarChoice>()
        .init_resource::<crate::game_mode::trail_blazer::PlayerTrailBlazers>()
        .init_resource::<visos_render::track_fx::HeadlightView>()
        .init_resource::<visos_render::track_fx::HeadlightViews>()
        .add_systems(
            Update,
            headlight_view.after(crate::game_mode::race_cars::place_mirror).run_if(in_state(crate::EGameMode::InGame)),
        )
        .add_systems(OnExit(crate::InRace), reset_headlight_view)
        .insert_resource(CWorldModeManager::default())
        .init_resource::<OverrideView>()
        .init_resource::<AiCameraCalls>()
        .add_message::<OsdCinematicBars>()
        .add_systems(Last, camera_load_steps.run_if(in_state(crate::EGameMode::InGame)))
        .add_systems(
            crate::vs_common_main::GameTick,
            clip_tweaks_changed
                .after(crate::game_mode::core_tweaks::core_tweaks_out)
                .in_set(crate::vs_common_main::InGameUpdateSet::Input),
        )
        .add_systems(
            crate::vs_common_main::GameTick,
            replay_check_for_load
                .in_set(crate::vs_common_main::InGameUpdateSet::Cameras)
                .run_if(in_state(crate::EGameMode::InGame)),
        )
        .add_systems(
            crate::vs_common_main::GameTick,
            (
                crash_and_poi.in_set(crate::vs_common_main::InGameUpdateSet::CrashCam),
                apply_ai_camera_calls
                    .after(crate::vs_common_main::InGameUpdateSet::Ai)
                    .before(crate::vs_common_main::InGameUpdateSet::Physics),
                update_cameras.after(replay_check_for_load).in_set(crate::vs_common_main::InGameUpdateSet::Cameras),
            )
                .run_if(in_state(crate::EGameMode::InGame))
                .run_if(physics_ready),
        )
        .add_systems(
            crate::vs_common_main::GameTick,
            // end of the update: modes asked for anywhere in it (messages do not live from one 24 Hz update to
            // the next when frames run faster)
            (player_camera_shake, write_camera_bridge, camera_modes)
                .chain()
                .after(crate::vs_common_main::InGameUpdateSet::Timeline)
                .run_if(in_state(crate::EGameMode::InGame)),
        )
        .init_resource::<TrackStreamViews>()
        .add_systems(
            crate::vs_common_main::GameTick,
            stream_views.in_set(crate::vs_common_main::InGameUpdateSet::World).run_if(in_state(crate::EGameMode::InGame)),
        )
        .init_resource::<StreamViewPlayers>()
        .add_systems(
            crate::vs_common_main::GameTick,
            audio_camera.in_set(crate::vs_common_main::InGameUpdateSet::Audio).run_if(in_state(crate::EGameMode::InGame)),
        )
        .add_systems(OnExit(crate::InRace), clear_stream_views)
        .add_systems(OnExit(crate::InRace), |c: Res<AiCameraCalls>| {
            if let Ok(mut v) = c.0.lock() {
                v.clear();
            }
        });
}

/// a camera frame as the streamer reads it: position, -right / up / dir rows, vertical fov, aspect, near, far
struct StreamFrame {
    pos: Vec3,
    right: Vec3,
    up: Vec3,
    dir: Vec3,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

/// the player's current camera planes (VsBaseCamera +0xEC near, +0xF0 far; follow/mirror: the manager's)
pub fn current_planes(cams: &VsCameraManager, p: usize) -> Option<(f32, f32)> {
    current_frame(cams, p).map(|f| (f.near, f.far))
}

/// VsRender 0x0005ACD0 -> VsAIRaceVehicle 0x00023E00: the position of the camera manager's current player's
/// (+0x60) current camera
pub fn current_camera_position(cams: &VsCameraManager) -> Option<Vec3> {
    current_frame(cams, cams.unknown_60 as usize).map(|f| f.pos)
}

/// VsRender 0x0005ACD0 -> VsAIRaceVehicle 0x00023EB0: the direction of the same camera
pub fn current_camera_dir(cams: &VsCameraManager) -> Option<Vec3> {
    current_frame(cams, cams.unknown_60 as usize).map(|f| f.dir)
}

/// `Armco::13F70(g_CameraMan, p)` (manager +0x24 + 44p): the player's current camera
fn current_frame(cams: &VsCameraManager, p: usize) -> Option<StreamFrame> {
    use super::vs_base_camera::VsBaseCamera;
    let pc = cams.players.get(p)?;
    let base = |b: &VsBaseCamera| StreamFrame {
        pos: b.pos_render,
        right: b.right_render,
        up: b.up_render,
        dir: b.dir_render,
        fov: b.fov_render,
        aspect: b.aspect,
        near: b.near,
        far: b.far,
    };
    let follow = |f: &super::vs_vehicle_follow_camera::VsVehicleFollowCamera, near: f32, far: f32| StreamFrame {
        pos: f.eye,
        right: f.right,
        up: f.up,
        dir: f.look,
        fov: f.fov_rad,
        aspect: 4.0 / 3.0,
        near,
        far,
    };
    match pc.mode {
        ECameraMode::Follow => pc.follow.as_ref().map(|f| follow(f, pc.follow_near, pc.follow_far)),
        ECameraMode::Mirror => pc.mirror.as_ref().map(|f| follow(f, pc.mirror_near, pc.mirror_far)),
        ECameraMode::Cine => pc.cine.as_ref().map(|c| base(&c.base)),
        ECameraMode::Roaming => pc.roaming.as_ref().map(|c| base(&c.base)),
        ECameraMode::Interface3d => pc.interface3d.as_ref().map(base),
        ECameraMode::None => None,
    }
}

/// VsRender.cpp 0x0005C9C0 — the camera's six planes (+0x128), inside = n.p + w >= 0: left, right, bottom, top
/// (local normals (∓1, 0, a·tan) / (0, ∓1, tan) through the rows -right, up, dir), near, far
fn frustum_planes(f: &StreamFrame) -> [Vec4; 6] {
    let t = (f.fov / 2.0).tan();
    let at = f.aspect * t;
    let rows = [-f.right, f.up, f.dir];
    let side = |l: Vec3| {
        let n = l.normalize_or_zero();
        let n = rows[0] * n.x + rows[1] * n.y + rows[2] * n.z;
        n.extend(f.pos.dot(n) * -1.0)
    };
    let near_pt = f.pos + f.dir * f.near;
    let n_near = (near_pt - f.pos).normalize_or_zero();
    let far_pt = f.pos + f.dir * f.far;
    let n_far = -(far_pt - f.pos).normalize_or_zero();
    [
        side(Vec3::new(-1.0, 0.0, at)),
        side(Vec3::new(1.0, 0.0, at)),
        side(Vec3::new(0.0, -1.0, t)),
        side(Vec3::new(0.0, 1.0, t)),
        n_near.extend(n_near.dot(near_pt) * -1.0),
        n_far.extend(n_far.dot(far_pt) * -1.0),
    ]
}

/// CompartmentManager.cpp 0x000897C0's camera list: per player its current camera (position, direction, planes);
/// the binary adds the player's raceline look-ahead (`VsAIRaceVehicle::214B0`, ai crate). review = `dword_749110`.
/// The caller (0x0008A270) passes 1 player, 2 with `byte_6F7157` (= the manager's players); a player without a
/// current camera adds no view, so `StreamViewPlayers` keeps each view's player for its raceline (`v24`).
pub fn stream_views(cams: Res<GameCameras>, mut views: ResMut<TrackStreamViews>, mut players: ResMut<StreamViewPlayers>) {
    let n = cams.manager.players.len();
    players.0.clear();
    views.views.clear();
    for p in 0..n {
        let Some(f) = current_frame(&cams.manager, p) else { continue };
        players.0.push(p);
        views.views.push(StreamView { position: f.pos, forward: f.dir, planes: Some(frustum_planes(&f)), raceline: None });
    }
    views.review = cams.replay.review_cameras != 0;
}

/// the player index of each `TrackStreamViews` entry (CompartmentManager 0x000897C0 loop index)
#[derive(Resource, Default, Debug)]
pub struct StreamViewPlayers(pub Vec<usize>);

fn clear_stream_views(mut views: ResMut<TrackStreamViews>, mut players: ResMut<StreamViewPlayers>) {
    views.views.clear();
    views.review = false;
    players.0.clear();
}

/// VsAudio::UpdateListener 0x0004B180 / VsVehicleAudio::UpdateEmitters 0x00050770 camera inputs: **player 0 only**
/// in split screen too — listener = `Armco::13F70(g_CameraMan, 0)` position (GetPositionInline) and direction
/// (0x00023EB0), velocity = vehicle array [0] (0x001C6480) when mode(0) == 1 else zero (`unk_7805C0`), volume
/// offsets by `VsCameraManager 0x0005F430(0)` = player 0's view index (+4). VsAudio::Update runs after the cameras.
fn audio_camera(cams: Res<GameCameras>, audio: Option<ResMut<velocity_audio::input::GameAudioInput>>) {
    let Some(mut audio) = audio else { return };
    audio.camera = current_frame(&cams.manager, 0).map(|f| velocity_audio::input::CameraAudioInput {
        position: f.pos,
        forward: f.dir,
        up: f.up,
        follows_player: cams.manager.mode(0) == ECameraMode::Follow,
        view: cams.manager.players[0].view as i8,
    });
}

fn physics_ready(p: Option<Res<PhysicsWorldRes>>) -> bool {
    p.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_planes_inside_ahead() {
        let f = StreamFrame {
            pos: Vec3::new(10.0, 2.0, 5.0),
            right: Vec3::NEG_X,
            up: Vec3::Y,
            dir: Vec3::NEG_Z,
            fov: 1.0,
            aspect: 4.0 / 3.0,
            near: 0.5,
            far: 600.0,
        };
        let pl = frustum_planes(&f);
        let inside = |p: Vec3| pl.iter().all(|q| q.truncate().dot(p) + q.w >= 0.0);
        assert!(inside(f.pos + Vec3::new(0.0, 0.0, -50.0)));
        assert!(!inside(f.pos + Vec3::new(0.0, 0.0, 50.0)));
        assert!(!inside(f.pos + Vec3::new(0.0, 0.0, -700.0)));
        assert!(!inside(f.pos + Vec3::new(0.0, 100.0, -50.0)));
        assert!(!inside(f.pos + Vec3::new(100.0, 0.0, -50.0)));
        assert!(inside(f.pos + Vec3::new(30.0, 0.0, -50.0)));
    }
}
