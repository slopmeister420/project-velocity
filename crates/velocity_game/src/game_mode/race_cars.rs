//! glue (not an original file): the race's cars on the physics and render crates' public APIs, at the original
//! call sites — track collision with the "Physics Manager" load step, the race cars (PhysicsManager 0x0019E080:
//! players then the event's opponents on the eventdef grid), `PhysicsManager->Update` in InGameUpdate (the
//! player's pad record from UserInput), and the per-update outputs: car bridge, triggers, audio, timeline world,
//! HUD. Rendering draws the car bridge interpolated by the frame timer.

use bevy::prelude::*;
use velocity_audio::input::{CarAudioInput, GameAudioInput};
use velocity_audio::AudioCommand;
use velocity_physics::plugin::{
    run_physics_tick, LoadPhysicsRaceObjects, LoadPhysicsTrack, PhysicsCar, PhysicsCarDamage, PhysicsCarInput, RepairCarDamage, SetupCarDamage, PhysicsCarSpawn, PhysicsCarState, PhysicsTimestep,
    PhysicsWorldRes, UnloadPhysicsTrack,
};
use velocity_physics::vehicle::vehicle_entity::sim::SpawnParams;
use velocity_physics::vehicle_database::vehicle_database::{string_hash, VehicleSetup};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;
use velocity_physics::vehicle_database::vehicle_database::CVehicleDatabase;
use visos_render::car_palette::CarColour;
use visos_render::car_render::{CarLightInput, CarRender, CarWheelSpin, CarRenderParts, CarSpec, DespawnCarRender, SpawnCarRender, PART_SLOTS};
use visos_render::cats_eye_render::{LoadCatsEyes, UnloadCatsEyes};
use visos_render::track_render::{LoadTrackRender, TrackCamera, UnloadTrackRender};

use crate::camera::game_cameras::GameCameras;
use crate::controller::user_input::{EAction, UserInput};
use crate::event_database::event_database::EVENT_FLAG_TEMP;
use crate::game_bridge::game_bridge::{CarBridgeData, GameBridge};
use crate::game_data_loading::game_data_loading::ELoadStep;
use crate::game_input::game_input::GameInput;
use crate::game_mode::car_damage_bridge;
use crate::game_mode::game_manager::{EGameMode, EngineShake, GameManager, LoadStepMessage};
use crate::game_mode::race_session::{ESessionState, RaceSession, RaceSessionMessage};
use crate::timeline_manager::objects::timeline_object::TimelineWorld;
use crate::timeline_manager::race_timeline_sequencer::TimelineCameraView;
use crate::triggers::trigger_update::{TriggerVehicle, TriggerVehicles};
use crate::vs_common_main::{GameClock, GameTick, InGameUpdateSet};

/// the car the player picked (vehicle folder like `\VEHICLES\LA SX`, its setup, camera view records of the car).
#[derive(Resource, Clone, Debug, Default)]
pub struct PlayerCarChoice {
    pub vehicle_path: Option<String>,
    /// `dword_6F724C[139 * player]`: the player's vehicle setup (parts, paint) handed to the physics
    pub setup: Option<VehicleSetup>,
    /// `dword_6F7478` (= `dword_6F724C[139]`): player 2's setup in a split-screen race
    pub setup2: Option<VehicleSetup>,
    pub camera_view: Option<[f32; 20]>,
    /// the car's whole view table (`VehicleData` list a) for the camera manager
    pub camera_views: Vec<[f32; 20]>,
}

/// the opponent and team databases are the ai crate's (it depends on this one): the binary installs this so
/// PhysicsManager 0x0019E080 can pick the opponents' cars.
pub trait RaceCarSource: Send + Sync {
    /// opponent (by name crc) vehicle setup `index` (`+0x44[index].+0x0C`, built by OpponentDatabase.cpp
    /// 0x000E72C0) and its short name (`+0x10`, the `ehicles\opponent\%s	ier%d.vcd` folder)
    fn opponent_car(&self, db: &CVehicleDatabase, crc: u32, index: usize) -> Option<(VehicleSetup, String)>;
    /// the team car of the opponent's team (`+0x24`, VsCommonMain 0x0016E610 then the team's `+0x00`)
    fn team_car(&self, db: &CVehicleDatabase, crc: u32) -> Option<VehicleSetup>;
    /// the car of team `team` (TeamDatabase by crc, VsCommonMain 0x0016E610, then the team's car
    /// FEHangoutProgression 0x00290CD0 set up by CVehicleDatabase::SetupDefaultVehicle)
    fn team_car_by_team(&self, _db: &CVehicleDatabase, _team: u32) -> Option<VehicleSetup> {
        None
    }
    /// "Police Car setup": police car `n` of the area's police opponents (group crc of "LA" / "LONDON" /
    /// "TOKYO", flag 2), their first car
    fn police_car(&self, _db: &CVehicleDatabase, _group: u32, _n: usize) -> Option<(VehicleSetup, String)> {
        None
    }
}

#[derive(Resource)]
pub struct RaceCarSourceRes(pub Box<dyn RaceCarSource>);

/// `VsAIRoadLine::GetLength` of the first race section's raceline (`VsAIWorld::GetFirstRaceSection` 0x00024300);
/// the ai world is the ai crate's, so the binary fills this
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct RacelineLength(pub Option<f32>);

/// what the in-race HUD shows (the frontend's `OsdRaceStatus` fields game flow can fill). `racing`, `lap`,
/// `position` and `race_length` are the ai race manager's (filled by the binary, `hud.rs`); the rest by
/// `car_outputs` from player car 0.
#[derive(Resource, Clone, Debug, Default)]
pub struct RaceStatus {
    /// player vehicle +0xF4: crossed the start line (never set in a cruise)
    pub racing: bool,
    /// player vehicle +0x2FC (VsAIPoliceVehicle 0x00017E00)
    pub lap: u32,
    pub laps: u32,
    pub lap_time: f32,
    /// RaceInfo.dPlayerPos[player] (VsAIWorld::GetPlayerVehicle 0x000346C0), 1-based
    pub position: u32,
    /// VsAIWorld::GetRaceLength 0x000348C0 -> VsRaceManager 0x0003DCA0: players + active ai cars
    pub race_length: u32,
    pub speed: f32,
    /// engine rpm and gear (0 neutral, < 0 reverse) for the speedo
    pub rpm: f32,
    pub gear: i32,
    /// race vehicle record +84 (`VsAIRaceVehicle 0x000244E0` +0x54, the score manager's score; COSDInGameScreen
    /// 0x000F99E0 through the score manager's +188 vehicle)
    pub points: u32,
    /// score manager (VsRaceManager 0x0003DF30 = race vehicle +0x1C8) +152 combo score, +156 combo count
    /// (multiplier), +168 wrong-way time (the OSD shows it while > 1.0)
    pub combo: u32,
    pub multiplier: u32,
    pub wrong_way_time: f32,
    /// Speedo::GetRatioForGauge 0x000F4F40 on physics vehicle [p]: nos 0x00024720 (VE +96 / +100), zone 0x000F78D0
    /// (VE +104 / +108), turbo = 1.0 without one (0x000F7970 VE +144 == 0) else 0x000F79A0 (VE +148)
    pub nos: f32,
    pub zone: f32,
    pub turbo: f32,
}

/// race vehicle record +84 (points) of each racer by place (1-based place = index + 1, `VsAIWorld::
/// GetVehicleInPosition`), for the results table OSDManager 0x000F09C0; the binary fills it from the ai
#[derive(Resource, Clone, Debug, Default)]
pub struct RacerPoints(pub Vec<u32>);

impl RacerPoints {
    /// race vehicle +84 of the racer in 1-based `place` (OSDManager 0x000F09C0's per-row prestige), 0 when no racer
    pub fn points_in_place(&self, place: usize) -> u32 {
        place.checked_sub(1).and_then(|i| self.0.get(i)).copied().unwrap_or(0)
    }
}

/// `RaceStatus` per player (one COSDInGameScreen per player, its player index = screen +0x14, COSDInGameScreen::Update
/// 0x000F99E0): [0] is a copy of `RaceStatus`, [p] filled the same way from player p's race / physics vehicle
#[derive(Resource, Clone, Debug, Default)]
pub struct PlayerRaceStatus(pub Vec<RaceStatus>);

/// what a car of the race is
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarKind {
    #[default]
    Race,
    Traffic,
    Police,
}

/// a car of the race: its slot (vehicle id = bridge index = render request), whether a human drives it
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct RaceCar {
    pub slot: usize,
    pub human: bool,
    pub kind: CarKind,
    /// the physics car is active (`VE+28`; traffic and police start parked until the ai places them)
    pub active: bool,
}

/// the race cars and the world camera
#[derive(Resource, Default)]
pub struct RaceCars {
    pub cars: Vec<Entity>,
    pub camera: Option<Entity>,
    /// split screen: the world cameras of players 1.. (player 0's is `camera`)
    pub player_cameras: Vec<Entity>,
    /// one player: the rear-view mirror's camera (view 1, kind 1)
    pub mirror_camera: Option<Entity>,
    /// the physics track still to load (written into the next physics tick so the message cannot expire)
    pub pending_track: Option<String>,
    /// the race directory whose `RACE.VCD` objects physics registers right after the track
    pub pending_race: Option<String>,
    /// render wheel slot for each FastCar wheel, per car (matched by position once the model is built)
    pub wheel_map: Vec<Option<[usize; 4]>>,
    /// per opponent physics id: the vehicle entry's ai engine group (+0x40 +4), 0x0019E080 `LoadAiEngine` path
    pub ai_engines: Vec<(usize, String)>,
}

/// `MAX_TRAFFIC_VEHICLES`
pub const MAX_TRAFFIC: usize = 14;
/// VsPoliceManager.cpp 0x00038A50 police car count
pub const MAX_POLICE: usize = 2;

/// a race car waiting for the track collision; created in vehicle id order (`VehicleManager__1DC650`) so the
/// ai's ids (players, then opponents) are the physics vehicle ids
#[derive(Component, Clone, Debug)]
pub struct PendingCar(pub PhysicsCarSpawn);

/// model space (nose +Z) to the physics body frame (nose +X)
pub(crate) const MODEL_TO_BODY: Quat = Quat::from_xyzw(0.0, std::f32::consts::FRAC_1_SQRT_2, 0.0, std::f32::consts::FRAC_1_SQRT_2);

/// `\VEHICLES\LA\240SX` -> ("LA/240SX", "240SX")
fn vehicle_dir(path: &str) -> (String, String) {
    let p = path.trim_matches('\\');
    let rest = p.split_once('\\').map(|(_, r)| r).unwrap_or(p);
    let name = rest.rsplit('\\').next().unwrap_or(rest).to_string();
    (rest.replace('\\', "/"), name)
}

/// the eventdef grid slot: position, and the heading the AI uses (`axis_angle((90 - dir) deg, +y)` applied
/// through its conjugate) as a transform rotation (car forward = model +X)
fn grid_transform(gm: &GameManager, slot: usize) -> Transform {
    let d = gm.event_def.as_ref();
    let p = d.and_then(|d| d.start_positions.get(slot)).copied().unwrap_or([0.0, 0.0, 0.0]);
    let dir = d.map(|d| d.start_direction).unwrap_or(90.0);
    Transform::from_translation(Vec3::new(p[0], p[1], p[2])).with_rotation(Quat::from_rotation_y((dir - 90.0).to_radians()))
}

/// the world camera player `p` is drawn with (one per player view, GameManager 0x000B3EC0)
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerView(pub usize);

fn spawn_world_camera(commands: &mut Commands, p: usize, players: usize) -> Entity {
    // VsCameraManager 0x0005ECC0: `byte_6F7157` (two players) halves the cameras' far planes (culling only here)
    let far = if players > 1 { 20000.0 * visos_render::view_render::SPLIT_FAR_SCALE } else { 20000.0 };
    commands
        .spawn((
            Camera3d::default(),
            // graphics' world order (after the sky passes, before the FSE passes and the HUD at 10+); a later
            // player's view draws into its own viewport without clearing the other's
            Camera {
                order: visos_render::track_render::WORLD_CAMERA_ORDER,
                clear_color: if p == 0 { ClearColorConfig::default() } else { ClearColorConfig::None },
                ..default()
            },
            TrackCamera,
            PlayerView(p),
            Projection::Perspective(PerspectiveProjection {
                fov: visos_render::sky_render::BACKGROUND_SPAN,
                near: 0.5,
                far,
                ..default()
            }),
            Transform::default(),
        ))
        .id()
}

/// "Init Game Views" (InGameInit 0x000B6498 -> GameManager 0x000B3EC0) for the race's players (`byte_6F9180` =
/// options +0x24 picks a side-by-side split); every player after 0 gets its own world camera
fn init_game_views(
    mut r: MessageReader<LoadStepMessage>,
    mut commands: Commands,
    gm: Res<GameManager>,
    career: Option<Res<crate::game_mode::career::Career>>,
    mut views: ResMut<crate::game_mode::game_views::GameViews>,
    mut cars: ResMut<RaceCars>,
) {
    for m in r.read() {
        if m.step != ELoadStep::InitGameViews {
            continue;
        }
        let players = gm.race.as_ref().map_or(1, |r| r.num_players.max(1)) as u32;
        let vertical = career.as_ref().is_some_and(|c| c.options.time_format != 0);
        views.init(players, vertical);
        for p in cars.player_cameras.len() + 1..players as usize {
            let e = spawn_world_camera(&mut commands, p, players as usize);
            cars.player_cameras.push(e);
        }
        // view 1 of a one-player race is the rear-view mirror (kind 1); graphics sets its viewport, order, target
        if players == 1 && views.views[1].kind == 1 && cars.mirror_camera.is_none() {
            let rect = crate::game_mode::game_views::GameViews::fraction(&views.views[1]);
            let e = commands
                .spawn((
                    Camera3d::default(),
                    Camera { order: visos_render::track_render::WORLD_CAMERA_ORDER, is_active: false, ..default() },
                    visos_render::view_render::MirrorView { player: 0, rect, on: false },
                    Projection::Perspective(PerspectiveProjection {
                        fov: visos_render::sky_render::BACKGROUND_SPAN,
                        near: 0.5,
                        far: 20000.0,
                        ..default()
                    }),
                    Transform::default(),
                ))
                .id();
            cars.mirror_camera = Some(e);
        }
        log::info!("game views: {players} player(s), vertical {vertical}");
    }
}

/// each player's world camera tells graphics its view (GameManager 0x000B3C30 per view of 0x000B9010): player and
/// rect (fraction of the screen; whole screen until the views exist). Graphics' `view_render` sets the viewports,
/// orders and targets from it and keeps player 0's camera as the `TrackCamera`.
fn track_views(
    mut commands: Commands,
    views: Res<crate::game_mode::game_views::GameViews>,
    cams: Query<(Entity, &PlayerView, Option<&visos_render::view_render::TrackView>)>,
) {
    for (e, pv, tv) in &cams {
        let rect = views.player_view(pv.0).map(crate::game_mode::game_views::GameViews::fraction).unwrap_or(Vec4::new(0.0, 0.0, 1.0, 1.0));
        let want = visos_render::view_render::TrackView { player: pv.0, rect };
        if tv != Some(&want) {
            commands.entity(e).insert(want);
        }
    }
}

/// the load steps: track render and collision, then the race cars (PhysicsManager 0x0019E080)
#[allow(clippy::too_many_arguments)]
fn race_load_steps(
    mut r: MessageReader<LoadStepMessage>,
    mut commands: Commands,
    mut gm: ResMut<GameManager>,
    choice: Res<PlayerCarChoice>,
    source: Option<Res<RaceCarSourceRes>>,
    attract: Res<crate::font_helper::CAttractMode>,
    mut cars: ResMut<RaceCars>,
    mut load: MessageWriter<LoadTrackRender>,
    mut spawn: MessageWriter<SpawnCarRender>,
    mut audio: MessageWriter<AudioCommand>,
    mut cats_eyes: MessageWriter<LoadCatsEyes>,
) {
    for m in r.read() {
        let Some(race) = &gm.race else { continue };
        let track = race.track_path.trim_matches('\\').rsplit('\\').next().unwrap_or(&race.track).to_string();
        match m.step {
            // GameDataLoading 0x000A1190 "Load Level Sounds" (audio's): read here so it keeps its place in the step
            // order, before the player's rev bands (Physics Manager step)
            ELoadStep::LevelSounds => {
                audio.write(AudioCommand::LoadLevelSounds { track_dir: race.track_path.clone(), wet: race.wet });
            }
            ELoadStep::LevelArena => {
                load.write(LoadTrackRender::new(track.clone()));
                if cars.camera.is_none() {
                    cars.camera = Some(spawn_world_camera(&mut commands, 0, race.num_players.max(1) as usize));
                }
            }
            ELoadStep::PhysicsManagerInit => {
                cars.pending_track = Some(track.clone());
                cars.pending_race = Some(crate::game_data_loading::game_data_loading::RacePaths::new(&race.track_path, &race.race_path).race_dir);
                // PhysicsManager 0x0019E30E / 0x0019E53A: RandomiseSetup per race car while the attract mode plays,
                // on the game's VsRandUnit state
                let mut rand = gm.rand.clone();
                let r = attract.playing.then_some(&mut rand);
                spawn_race_cars(&mut commands, &gm, &choice, source.as_deref().map(|s| &*s.0), r, &mut cars, &mut spawn, &mut audio);
                gm.rand = rand;
            }
            // GameDataLoading 0x0009D055 "Load catseyes" → CatsEyeRender 0x0011CDB0(race dir, trailing `\`)
            ELoadStep::CatsEye => {
                let dir = m.paths.as_ref().map(|p| p.race_dir.clone());
                let dir = dir.unwrap_or_else(|| crate::game_data_loading::game_data_loading::RacePaths::new(&race.track_path, &race.race_path).race_dir);
                log::info!("Load catseyes: {dir}CatsEyes.ced");
                cats_eyes.write(LoadCatsEyes { race_dir: dir });
            }
            _ => {}
        }
    }
}

/// a car PhysicsManager 0x0019E080 creates: vehicle id, eventdef grid slot (= competitor slot), driver, setup
#[derive(Clone, Debug)]
pub struct RaceCarPlan {
    pub id: usize,
    pub grid: usize,
    pub human: bool,
    pub setup: VehicleSetup,
}

/// PhysicsManager 0x0019E080 "Race Competitors" — two passes over the current race's competitor slots:
/// humans (player n = vehicle id n, the player's setup `dword_6F724C[139 * n]`), then the opponents (ids
/// after the players): the team car in career categories 2/3 (`(flags & 8) == 0`), else the opponent's setup
/// for the event tier (`tier - 1`, 0 for tier 0). Grid position = the competitor slot. Without an event the
/// players take the first slots. `rand` = Some while the attract mode plays (`byte_6F5A28`): each car's setup goes
/// through `RandomiseSetup` (0x0019E30E players, 0x0019E53A opponents, in pass order). The original randomises the
/// garage / opponent database setup in place; here the race's copy (also for the drag `+0x18C` clear).
pub fn plan_race_cars(
    gm: &GameManager,
    db: &CVehicleDatabase,
    player: &VehicleSetup,
    player2: Option<&VehicleSetup>,
    source: Option<&dyn RaceCarSource>,
    mut rand: Option<&mut visos::handy::crt_rand::CrtRand>,
) -> Vec<RaceCarPlan> {
    // human n's setup `dword_6F724C[139 * n]` (player 2's falls back to player 1's when none was chosen)
    let setup_of = |n: usize| if n == 1 { player2.unwrap_or(player) } else { player };
    let players = gm.race.as_ref().map(|r| r.num_players.max(1) as usize).unwrap_or(1);
    let mut out = Vec::new();
    let event = gm.current_event();
    let slots = event.and_then(|e| Some((e, e.current_race()?, e.competitors.as_ref()?)));
    let Some((e, race, comps)) = slots else {
        for n in 0..players {
            out.push(RaceCarPlan { id: n, grid: n, human: true, setup: setup_of(n).clone() });
        }
        if let Some(r) = rand.as_deref_mut() {
            for p in &mut out {
                db.randomise_setup(&mut p.setup, true, r);
            }
        }
        return out;
    };
    let comp = |i: usize| race.competitors.get(i).copied().flatten().and_then(|c| comps.get(c));
    let n_slots = race.num_competitors as usize;
    // PhysicsManager 0x001B4010: the current race entry's flag 2 (the builders' second flag, drag) clears setup
    // +0x18C before the randomise (0x0019E2EC / 0x0019E518)
    let clear_18c = race.flags & 2 != 0;
    let mut np = 0;
    for i in 0..n_slots {
        if comp(i).is_some_and(|c| c.human != 0) {
            let mut setup = setup_of(np).clone();
            if clear_18c {
                setup.unk_18c = 0;
            }
            if let Some(r) = rand.as_deref_mut() {
                db.randomise_setup(&mut setup, true, r);
            }
            out.push(RaceCarPlan { id: np, grid: i, human: true, setup });
            np += 1;
        }
    }
    let mut k = 0;
    for i in 0..n_slots {
        let Some(c) = comp(i).filter(|c| c.human == 0) else { continue };
        let team = (e.category == 2 || e.category == 3) && e.flags & EVENT_FLAG_TEMP == 0;
        let mut setup = if team { source.and_then(|s| s.team_car(db, c.opponent_crc)) } else { None };
        if setup.is_none() {
            let tier = e.tier.saturating_sub(1) as usize;
            match source.and_then(|s| s.opponent_car(db, c.opponent_crc, tier)) {
                Some((st, short)) => {
                    log::info!("race car {}: opponent {short} tier{}", np + k, tier + 1);
                    setup = Some(st);
                }
                None => log::warn!("race car {}: opponent {:08X} has no car", np + k, c.opponent_crc),
            }
        }
        if let Some(mut setup) = setup {
            if clear_18c {
                setup.unk_18c = 0;
            }
            if let Some(r) = rand.as_deref_mut() {
                db.randomise_setup(&mut setup, true, r);
            }
            out.push(RaceCarPlan { id: np + k, grid: i, human: false, setup });
        }
        // the ai's ids advance per opponent slot whether or not its car exists
        k += 1;
    }
    out
}

/// render spec of a setup: the vehicle folder, the body colour record (`+0x88`: kind `+0x0C`, float colours
/// `+0x1C`/`+0x28`, CarPalette via `with_colour`) and the fitted body parts — VehicleEntity.cpp 0x001C08B0 loads
/// each fitted part's model by its name (`+0x10`, e.g. "f_bumper3"), so the name gives the render slot and index
fn car_spec(db: &CVehicleDatabase, setup: &VehicleSetup, fallback: &str) -> CarSpec {
    let path = setup.vehicle.and_then(|v| db.vehicles.get(v)).map(|v| v.path.clone()).unwrap_or_else(|| fallback.into());
    let (dir, _) = vehicle_dir(&path);
    let c = &setup.colours[0];
    let colour = CarColour::from_record(c.unk_0c, c.colour_a, c.colour_b);
    let mut spec = CarSpec::new(dir, Vec3::ZERO).with_colour(&colour);
    let names: Vec<&str> = setup.fitted_bodyparts.iter().flatten().filter_map(|r| db.part(*r).string_10.as_deref()).collect();
    if !names.is_empty() {
        spec.parts = [None; 7];
        for n in names {
            let stem = n.trim_end_matches(|c: char| c.is_ascii_digit());
            let slot = PART_SLOTS.iter().position(|s| s.eq_ignore_ascii_case(stem));
            match (slot, n[stem.len()..].parse::<u32>()) {
                (Some(k), Ok(i)) => spec.parts[k] = Some(i),
                _ => log::warn!("car {}: part model {n} has no render slot", spec.dir),
            }
        }
    }
    // the setup's decal slots (none of the shipped setups carry any)
    spec.decals = visos_render::car_render::CarDecals::from_setup(setup, db);
    // CarRender 0x00110220 / 0x00115920: shadow size from the vehicle data, neon from colour slot 1
    spec.with_setup_underbody(setup, db)
}

/// PhysicsManager 0x0019E080 — the race cars on their eventdef grid slots (VehicleManager__1DC650 creates them
/// once the collision exists)
fn spawn_race_cars(
    commands: &mut Commands,
    gm: &GameManager,
    choice: &PlayerCarChoice,
    source: Option<&dyn RaceCarSource>,
    rand: Option<&mut visos::handy::crt_rand::CrtRand>,
    cars: &mut RaceCars,
    spawn: &mut MessageWriter<SpawnCarRender>,
    audio: &mut MessageWriter<AudioCommand>,
) {
    let Some(race) = &gm.race else { return };
    let Ok(db) = CVehicleDatabase::load() else {
        log::error!("race cars: no vehicle database");
        return;
    };
    let player_path = choice.vehicle_path.clone().unwrap_or_else(|| r"\VEHICLES\LA\240SX".into());
    let player = choice.setup.clone().unwrap_or_else(|| {
        let (_, name) = vehicle_dir(&player_path);
        let mut s = VehicleSetup::new();
        db.setup_default_vehicle(&mut s, string_hash(&name.to_ascii_uppercase()));
        s
    });
    let grid = gm.event_def.as_ref().map(|d| d.start_positions.len()).unwrap_or(1).max(1);
    for plan in plan_race_cars(gm, &db, &player, choice.setup2.as_ref(), source, rand) {
        if plan.grid >= grid {
            log::warn!("race car {}: no grid slot {}", plan.id, plan.grid);
            continue;
        }
        let tf = grid_transform(gm, plan.grid);
        let mut inputs = LocalConfigInputs::from_setup(&db, &plan.setup);
        if inputs.name.is_empty() {
            inputs = LocalConfigInputs::default_spans(&vehicle_dir(&player_path).1);
        }
        inputs.is_player = plan.human;
        inputs.professional_race = race.professional;
        let pad_slot = if plan.human { plan.id as i32 } else { -1 };
        let mut e = commands.spawn((tf, PendingCar(PhysicsCarSpawn { inputs, pad_slot }), RaceCar { slot: plan.id, human: plan.human, kind: CarKind::Race, active: true }));
        if plan.human {
            e.insert(PhysicsCarInput::default());
            // 0x0019F120: competitor slot 0 (human) only -> VsAudioRevBands::Load(vehicle entry +0x40 +0 model)
            if plan.grid == 0 {
                match plan.setup.vehicle.and_then(|v| db.vehicles.get(v)).and_then(|v| v.audio.model.clone()) {
                    Some(name) => {
                        audio.write(AudioCommand::LoadRevBands { name });
                    }
                    None => log::warn!("race car {}: no rev band name", plan.id),
                }
            }
        } else if let Some(path) = plan.setup.vehicle.and_then(|v| db.vehicles.get(v)).and_then(|v| v.audio.audio.clone()) {
            // 0x0019E080 opponents: VsAudioVehics::LoadAiEngine(entry +0x40 +4) at 0x0019F2B0, ai +356 = hash of its base name
            audio.write(AudioCommand::LoadAiEngine { path: path.clone() });
            cars.ai_engines.push((plan.id, path));
        }
        cars.cars.push(e.id());
        if cars.wheel_map.len() <= plan.id {
            cars.wheel_map.resize(plan.id + 1, None);
        }
        let spec = car_spec(&db, &plan.setup, &player_path);
        let rtf = Transform { rotation: tf.rotation * MODEL_TO_BODY, ..tf };
        log::info!("race car {}: {} grid {} at {:?}{}", plan.id, spec.dir, plan.grid, tf.translation, if plan.human { " (player)" } else { "" });
        spawn.write(SpawnCarRender { request: plan.id as u64, spec, transform: rtf });
    }
    spawn_traffic_and_police(commands, gm, &db, source, cars, spawn);
}

/// the race's traffic area (`dword_6F9BE8` race entry -> track +0x84: 1 LA, 2 LONDON, 3 TOKYO), and its type
pub(crate) fn race_area_and_type(gm: &GameManager) -> (u32, u32) {
    let race = gm.race.as_ref();
    let mut ty = race.map(|r| r.race_type as u32).unwrap_or(0);
    let mut track = race.and_then(|r| gm.event_db.tracks.iter().position(|t| t.name.eq_ignore_ascii_case(&r.track)));
    if let Some(r) = gm.current_event().and_then(|e| e.current_race()) {
        ty = r.race_type;
        track = r.track.or(track);
    }
    (track.and_then(|t| gm.event_db.tracks.get(t)).map(|t| t.traffic_area).unwrap_or(0), ty)
}

/// PhysicsManager 0x0019E080 "Traffic Car setup" / "Police Car setup": 14 traffic cars (the first three the
/// area's kind 2 set when it has one, else kind 1; the rest kind 1; `n % count`), then the police manager's cars
/// (2) from the area's police opponents. Ids follow the ai's numbering: traffic from the race's competitor count,
/// police 14 later. Both are created parked at the origin (identity orientation) and placed by the ai.
fn spawn_traffic_and_police(
    commands: &mut Commands,
    gm: &GameManager,
    db: &CVehicleDatabase,
    source: Option<&dyn RaceCarSource>,
    cars: &mut RaceCars,
    spawn: &mut MessageWriter<SpawnCarRender>,
) {
    let Some(race) = &gm.race else { return };
    let (area, ty) = race_area_and_type(gm);
    // VsAIWorld::Init flags (GameManager.cpp 0x000B5F80)
    let traffic = race.traffic && !gm.vars.network_race;
    let police = (traffic || ty == 6) && ty != 5;
    let first = match gm.current_event().and_then(|e| e.current_race()) {
        Some(r) => r.num_competitors as usize,
        None => gm.event_def.as_ref().map(|d| d.num_opponents.max(0) as usize).unwrap_or(1),
    };
    let add = |commands: &mut Commands, cars: &mut RaceCars, slot: usize, kind: CarKind, inputs: LocalConfigInputs| {
        let e = commands.spawn((Transform::default(), PendingCar(PhysicsCarSpawn { inputs, pad_slot: -1 }), RaceCar { slot, human: false, kind, active: false }));
        cars.cars.push(e.id());
    };
    if traffic {
        for i in 0..MAX_TRAFFIC {
            let kind = if i >= 3 || db.count_traffic(area, 2) == 0 { 1 } else { 2 };
            let count = db.count_traffic(area, kind);
            let Some(t) = (count != 0).then(|| db.get_nth_traffic(area, kind, i as u32 % count)).flatten() else {
                log::error!("traffic car {i}: no traffic setups for area {area}");
                break;
            };
            let mut inputs = LocalConfigInputs::from_traffic(db, t);
            inputs.professional_race = race.professional;
            log::info!("traffic car {}: {}", first + i, inputs.name);
            // VehicleEntity 0x001C0D60 no-setup branch: TrafficM/T.vcd `<name>` + four `<name>_wheel`; no setup, so
            // no colour record (the paint colour is not set by the loader; white here)
            let spec = visos_render::car_render::CarSpec::traffic(inputs.name.clone(), Vec3::ONE);
            add(commands, cars, first + i, CarKind::Traffic, inputs);
            spawn.write(SpawnCarRender { request: (first + i) as u64, spec, transform: Transform::default() });
        }
    }
    if police {
        let group = match area {
            1 => "LA",
            2 => "LONDON",
            3 => "TOKYO",
            _ => {
                log::error!("assert: police area {area}");
                ""
            }
        };
        if !traffic {
            log::warn!("police without traffic: physics ids {first}.. but the ai numbers police from {}", first + MAX_TRAFFIC);
        }
        for n in 0..MAX_POLICE {
            let Some((setup, short)) = source.and_then(|s| s.police_car(db, string_hash(group), n)) else {
                log::warn!("police car {n}: no opponent car");
                break;
            };
            let mut inputs = LocalConfigInputs::from_setup(db, &setup);
            inputs.professional_race = race.professional;
            log::info!("police car {}: {short}", first + MAX_TRAFFIC + n);
            add(commands, cars, first + MAX_TRAFFIC + n, CarKind::Police, inputs);
            let mut spec = car_spec(db, &setup, "");
            // CarRender 0x00110220: +2328 |= 0x4000 for a car index past the race + traffic cars
            spec.police = true;
            spawn.write(SpawnCarRender { request: (first + MAX_TRAFFIC + n) as u64, spec, transform: Transform::default() });
        }
    }
}

/// EventDatabase.cpp 0x00091620 tail — a finished race adds `(int)raceline length * laps` metres to player 0's
/// car (`GamePlayer__C1E70(0)` = `dword_6F724C`, the player's setup); nothing without an ai raceline
fn race_mileage(mut r: MessageReader<RaceSessionMessage>, len: Res<RacelineLength>, mut choice: ResMut<PlayerCarChoice>) {
    for m in r.read() {
        let RaceSessionMessage::Mileage { laps } = m else { continue };
        let (Some(l), Some(setup)) = (len.0, choice.setup.as_mut()) else { continue };
        let metres = setup.add_mileage(VehicleSetup::race_mileage(l, *laps));
        log::info!("mileage +{metres} m -> {}", setup.mileage);
    }
}

/// GameManager.cpp 0x000BE840 ClearVehicleInputs
/// the players' cars before and at Go: ClearVehicleInputs (steer 0, handbrake 1, brake 0, wheels locked by body 0x0035DCC0), ReleaseCars = 0x000BD9B0 (wheels released, controls +0x16 = 1).
/// the ai's grid state locks the same wheels (0x0001B4F0) and a human car never leaves it, so only Go frees them.
/// start boost `VehicleEntity__1D1270` per car after the release (gauge constant 0.0, see below); the +0x16 byte is not
/// ported (nothing in the physics reads it)
/// GameManager.cpp 0x000BD9B0 ReleaseCars
fn race_release(
    mut r: MessageReader<RaceSessionMessage>,
    cars: Query<(&RaceCar, &PhysicsCar)>,
    physics: Option<Res<velocity_physics::plugin::PhysicsWorldRes>>,
) {
    let Some(w) = physics.as_deref() else {
        r.clear();
        return;
    };
    for m in r.read() {
        let mut g = w.lock();
        let Some(world) = g.as_mut() else { return };
        for (c, p) in &cars {
            if !c.human || !world.vehicle_ids().contains(&p.id) {
                continue;
            }
            match m {
                RaceSessionMessage::ClearVehicleInputs => {
                    let mut k = world.controls(p.id);
                    k.steer = 0.0;
                    k.handbrake = 1.0;
                    world.set_wheel_lock(p.id, true);
                    k.brake = 0.0;
                    world.set_controls(p.id, &k);
                }
                RaceSessionMessage::ReleaseCars => {
                    world.set_wheel_lock(p.id, false);
                    // OSD3dInterface +2936 (unk_6FC098) is only ever written by the COSDMapRender ctor 0x000E9A80 (0.0)
                    world.launch_boost(p.id, 0.0);
                    log::info!("launch result {} (car {})", world.launch_result(p.id), p.id);
                }
                _ => {}
            }
        }
    }
}

/// mode shutdown: cars, renders, track
fn despawn_race(
    mut commands: Commands,
    mut cars: ResMut<RaceCars>,
    slots: Query<&RaceCar>,
    mut unload: MessageWriter<UnloadTrackRender>,
    mut unload_cats_eyes: MessageWriter<UnloadCatsEyes>,
    mut unload_phys: MessageWriter<UnloadPhysicsTrack>,
    mut despawn: MessageWriter<DespawnCarRender>,
    physics: Option<Res<PhysicsWorldRes>>,
    mut spins: ResMut<WheelSpins>,
    mut views: ResMut<crate::game_mode::game_views::GameViews>,
    mut statuses: ResMut<PlayerRaceStatus>,
) {
    spins.0.clear();
    // the next race's cameras start full screen until its "Init Game Views" step
    *views = default();
    statuses.0.clear();
    for e in cars.cars.drain(..) {
        if let Ok(c) = slots.get(e) {
            despawn.write(DespawnCarRender { request: c.slot as u64 });
        }
        commands.entity(e).despawn();
    }
    if let Some(c) = cars.camera.take() {
        commands.entity(c).despawn();
    }
    for c in cars.player_cameras.drain(..) {
        commands.entity(c).despawn();
    }
    if let Some(c) = cars.mirror_camera.take() {
        commands.entity(c).despawn();
    }
    cars.wheel_map.clear();
    cars.ai_engines.clear();
    unload.write(UnloadTrackRender);
    // graphics' request: the original has no unload (the next race's 0x0011CDB0 replaces the file pointer)
    unload_cats_eyes.write(UnloadCatsEyes);
    // PhysicsTick does not run in the front end, so the message alone expires unread and the old world (with its
    // cars) lived into the next race, where the ai bound its cars to it before the new track replaced it
    if let Some(p) = physics {
        *p.lock() = None;
    }
    unload_phys.write(UnloadPhysicsTrack);
}

/// before `PhysicsManager->Update`: the frame time, a pending track load, the players' pad records
/// (VehicleEntity 0x001CB530 reads them); cars are held on the grid until released (0x000BD9B0)
fn car_inputs(
    clock: Res<GameClock>,
    ui: Res<UserInput>,
    gi: Res<GameInput>,
    session: Res<RaceSession>,
    mut cars: ResMut<RaceCars>,
    mut step: ResMut<PhysicsTimestep>,
    mut load: MessageWriter<LoadPhysicsTrack>,
    mut load_objects: MessageWriter<LoadPhysicsRaceObjects>,
    mut q: Query<(&RaceCar, &mut PhysicsCarInput)>,
    pending: Query<(Entity, &PendingCar, &RaceCar, &Transform)>,
    physics: Option<Res<velocity_physics::plugin::PhysicsWorldRes>>,
    mut commands: Commands,
    attract: Res<crate::font_helper::CAttractMode>,
    cameras: Option<ResMut<GameCameras>>,
) {
    step.0 = clock.timer.scaled_dt();
    // VehicleManager__1DC650: every car at once, in vehicle id order, at its eventdef grid position
    if let Some(w) = physics.as_deref() {
        let mut g = w.lock();
        if let Some(world) = g.as_mut() {
            let mut list: Vec<_> = pending.iter().collect();
            list.sort_by_key(|(_, _, c, _)| c.slot);
            let spawned = !list.is_empty();
            for (e, p, c, tf) in list {
                // `Create` stores the conjugate of the VS quaternion it is given (the car faces rotate(q, +X))
                let id = world.spawn(&SpawnParams {
                    inputs: p.0.inputs.clone(),
                    position: tf.translation.to_array(),
                    orientation: tf.rotation.conjugate().to_array(),
                    slot: p.0.pad_slot,
                });
                if id != c.slot {
                    log::error!("race car {}: physics vehicle id {id}", c.slot);
                }
                // traffic and police wait parked (inactive) until the ai places them
                if c.kind != CarKind::Race {
                    world.deactivate(id);
                }
                // VsAudioEvents::StartPlayerEngine 0x0004C3A0: dword_6D4D98 = 6000 (audio's top rpm, the scale of
                // 0x001B6F10); the port's audio starts the engine on its own, so the physics global is set as the
                // player car exists, before the first engine_rpm
                if c.human && c.slot == 0 {
                    world.set_audio_max_rpm(6000);
                }
                commands.entity(e).remove::<PendingCar>().insert((PhysicsCar { id, pad_slot: p.0.pad_slot }, PhysicsCarState::default()));
            }
            // VehicleEntity 0x001C9070 (run by each spawn) writes the Handling.cfg camera globals that are non-zero;
            // applied once per batch of spawns so debug-menu edits survive
            if spawned {
                if let Some(mut cams) = cameras {
                    let o = world.handling_camera_overrides();
                    let m = &mut cams.manager;
                    if let Some(v) = o.far_clip_distance { m.clip.far = v; }
                    if let Some(v) = o.yaw_rate_per_degree { m.globals.yaw_gain = v; }
                    if let Some(v) = o.yaw_relax_angle { m.globals.yaw_full = v; }
                    if let Some(v) = o.yaw_direction_blend { m.globals.heading_weight = v; }
                }
            }
        }
    }
    if let Some(track) = cars.pending_track.take() {
        load.write(LoadPhysicsTrack { track });
        // the race's placed objects (RACE.VCD instance / uinstance), registered into the new world
        if let Some(race) = cars.pending_race.take() {
            load_objects.write(LoadPhysicsRaceObjects { race });
        }
    }
    let held = !matches!(session.state, ESessionState::Go | ESessionState::Racing | ESessionState::Finished | ESessionState::PostRace);
    for (car, mut inp) in &mut q {
        let mask = gi.device_mask(car.slot);
        if held || !car.human {
            *inp = PhysicsCarInput { handbrake: true, ..Default::default() };
            continue;
        }
        // GameInput 0x000A4F10: the pad record is zeroed while the attract mode plays (the ai drives)
        if attract.playing {
            *inp = PhysicsCarInput::default();
            continue;
        }
        let v = |a: EAction| ui.value(a, mask);
        *inp = PhysicsCarInput {
            axis: v(EAction::CarAccelerateBrake),
            accelerator: v(EAction::CarAccelerate),
            brake: v(EAction::CarBrake),
            steer: (v(EAction::CarSteering) + v(EAction::CarSteerRight) - v(EAction::CarSteerLeft)).clamp(-1.0, 1.0),
            handbrake: v(EAction::CarHandbrake) > 0.5,
            gear_up: ui.pressed(EAction::CarGearUp, mask),
            gear_down: ui.pressed(EAction::CarGearDown, mask),
            boost: [v(EAction::CarNitrous) > 0.5, v(EAction::CarZoneMode) > 0.5],
        };

    }
}

/// what VsAudio::UpdateVehicles (0x000524B0) reads off physics vehicle 0 (VE, car = VE+0xF0):
/// UpdateSuspension 0x00051570 contact 0x00364BB0 (car +1088 contacts > 0) and compression 0x003649D0 (car +2268),
/// UpdateBrakes 0x00051B00 slip = 0x0036CCB0 (car +160, one vector for every wheel; y is dropped by the reader),
/// reversing VE+0xC90 < 0 (int); UpdatePlayerEngine 0x0004C470 throttle and UpdateSuspension 0x000516F4 front pair
/// from the newest state history entry (VE+0xDE8, entries in the VE+3560 heap block, 0x001B6E80(0)) +0x24 / +0x2C. The gear (UpdatePlayerEngine
/// 0x0004C44E) is the caller's `PhysicsWorld::shift_gear`.
fn player_audio_wheels(w: &velocity_physics::vehicle::vehicle_entity::sim::PhysicsWorld, id: usize, input: &mut CarAudioInput) {
    let s = w.slot(id);
    let (car, ve) = (&s.car, &s.ve);
    let slip = Vec3::new(car.m.f(160), car.m.f(164), car.m.f(168));
    for (w, wheel) in input.wheels.iter_mut().enumerate() {
        wheel.contact = car.wheel_touches(w);
        wheel.compression = car.m.f(2268 + 4 * w);
        wheel.slip = slip;
    }
    input.reversing = ve.m.i(0xC90) < 0;
    // 0x001B6E80(0): k = 0 is never > count, so the entry is the head's
    let head = ve.m.i(0xDE8 + 4);
    let e = (velocity_physics::vehicle::vehicle_entity::cx::HIST_ENTRY * head) as usize;
    // the history is a heap block at VE+3560 (`ve.hist` stays zero): read it as the physics code does
    if let Some(hp) = ve.ptrs.get(&3560).filter(|p| !p.is_null()) {
        let (m, o) = w.glob.heap.block(hp.off + e as i32);
        input.throttle = m.f(o + 0x24);
        input.front_suspension = m.i(o + 0x2C) != 0;
    }
}

/// the VsVehicleAudio view of a physics car (VE position 0x001BA270, velocity 0x001C6480, up 0x001C6630,
/// forward 0x001C66C0, speed VE+20, rpm 0x001B6F10, `active` = VE+28); the player adds wheels / gear itself.
pub fn car_audio_input(st: &PhysicsCarState, active: bool) -> CarAudioInput {
    let forward = st.rotation * Vec3::X;
    CarAudioInput {
        position: st.position,
        velocity: st.velocity,
        up: st.rotation * Vec3::Y,
        forward,
        speed: st.speed(),
        // 0x001B6F10 smooths VE+4068 on every call: the caller fills it with `PhysicsWorld::engine_rpm` where the
        // original calls it (UpdateRpm 0x00052401, UpdateAiCars 0x000534EC, UpdateTraffic 0x00053B36)
        engine_rpm: 0,
        gear: st.gear,
        throttle: 0.0,
        reversing: st.velocity.dot(forward) < 0.0,
        // VsVehicleAudio +40/+44: only VsAudio::Init (-1) and FreeVehicleAudio (0) write them in F3_D, so
        // SurfacePrefix always sees -1 (tarmac)
        surface: [u32::MAX, u32::MAX],
        active,
        ..default()
    }
}

/// after the physics frame: car bridge, triggers, audio, timeline world and HUD from the cars' state
#[allow(clippy::too_many_arguments)]
fn car_outputs(
    mut q: Query<(&mut RaceCar, &PhysicsCar, &PhysicsCarState)>,
    physics: Option<Res<velocity_physics::plugin::PhysicsWorldRes>>,
    session: Res<RaceSession>,
    gm: Res<GameManager>,
    cams: Res<GameCameras>,
    mut bridge: ResMut<GameBridge>,
    mut trig: ResMut<TriggerVehicles>,
    mut audio: ResMut<GameAudioInput>,
    mut tw: ResMut<TimelineWorld>,
    mut hud: ResMut<RaceStatus>,
    mut statuses: ResMut<PlayerRaceStatus>,
    mut shake: ResMut<EngineShake>,
) {
    trig.0.clear();
    let mut racers = 0;
    let players = gm.race.as_ref().map_or(1, |r| r.num_players.max(1) as usize);
    if statuses.0.len() != players {
        statuses.0.resize(players, RaceStatus::default());
    }
    let laps = gm.event_def.as_ref().map(|e| e.num_laps.max(0) as u32).unwrap_or(0);
    let mut world = physics.as_deref().map(|p| p.lock());
    for (mut car, pc, st) in &mut q {
        car.active = world.as_ref().and_then(|w| w.as_ref()).is_some_and(|w| w.is_active(pc.id));
        if car.kind == CarKind::Race {
            racers += 1;
        } else if !car.active {
            if bridge.initialised && car.slot < bridge.cars.len() {
                bridge.car(car.slot).valid = false;
            }
            continue;
        }
        if bridge.initialised && car.slot < bridge.cars.len() {
            *bridge.car(car.slot) = CarBridgeData {
                position: st.position,
                rotation: st.rotation,
                wheel_position: std::array::from_fn(|i| st.wheel_matrix(i).w_axis.truncate()),
                wheel_rotation: std::array::from_fn(|i| Quat::from_mat4(&st.wheel_matrix(i))),
                valid: true,
            };
        }
        trig.0.push(TriggerVehicle { id: car.slot as u32, position: st.position, player: car.human, traffic: car.kind == CarKind::Traffic });
        // the speedo gauges of player p (Speedo::GetRatioForGauge 0x000F4F40, physics vehicle [p])
        if car.human && car.kind == CarKind::Race && car.slot < players {
            let gauges = world.as_ref().and_then(|w| w.as_ref()).filter(|w| w.vehicle_ids().contains(&pc.id)).map(|w| {
                // 0x000F4F40 case 3: `0.0 == 0x000F7970` -> 1.0, else 0x000F79A0
                let turbo = if 0.0 == w.has_turbo(pc.id) { 1.0 } else { w.turbo_ratio(pc.id) as f32 };
                (w.nos_ratio(pc.id) as f32, w.zone_ratio(pc.id) as f32, turbo)
            });
            let (nos, zone, turbo) = gauges.unwrap_or((0.0, 0.0, 1.0));
            let s = if car.slot == 0 { &mut *hud } else { &mut statuses.0[car.slot] };
            (s.nos, s.zone, s.turbo) = (nos, zone, turbo);
        }
        // player p >= 1's COSDInGameScreen (0x000F99E0 with its player index): speed of VehicleManager 0x001DDCC0(p),
        // OSDBridge +0 = (float)(int)0x001B6F10 on the same vehicle (one smoothing call per screen)
        if car.human && car.kind == CarKind::Race && car.slot > 0 && car.slot < players {
            let s = &mut statuses.0[car.slot];
            if let Some(w) = world.as_mut().and_then(|w| w.as_mut()).filter(|w| w.vehicle_ids().contains(&pc.id)) {
                s.rpm = w.engine_rpm(pc.id) as f32;
                if let Some(r) = shake.rpm.get_mut(car.slot) {
                    *r = s.rpm as i32;
                }
            }
            s.laps = laps;
            s.lap_time = session.race_time;
            s.speed = st.speed();
            s.gear = st.gear as i32;
        }
        if car.slot != 0 {
            continue;
        }
        let speed = st.speed();
        let mut input = car_audio_input(st, true);
        if let Some(w) = world.as_mut().and_then(|w| w.as_mut()).filter(|w| w.vehicle_ids().contains(&pc.id)) {
            player_audio_wheels(w, pc.id, &mut input);
            // VsVehicleAudio::UpdateRpm 0x00052401 first (0x001B6F10 itself steps the held gear via 0x001B74A0),
            // then UpdatePlayerEngine 0x0004C44E: 0x001B74A0 on VE 0 (>= 100 while the shift holds), then
            // COSDInGameScreen::Update 0x000F9D72 (next update's Osd set, before its physics frame, same state);
            // OSDBridge +0 = (float)(int)
            input.engine_rpm = w.engine_rpm(pc.id);
            shake.rpm[0] = input.engine_rpm;
            input.gear = w.shift_gear(pc.id);
            hud.rpm = w.engine_rpm(pc.id) as f32;
        }
        audio.player = Some(input);
        if clock_tick(&session) {
            log::debug!("car 0 at {:?} speed {:.1} gear {} cam {:?} view {:?}", st.position, speed, st.gear, cams.manager.mode(0), cams.manager.view(0).map(|v| v.position));
        }
        tw.player = st.position;
        // racing / lap / position / race length come from the ai race manager (binary `hud.rs`)
        hud.laps = laps;
        hud.lap_time = session.race_time;
        hud.speed = speed;
        hud.gear = st.gear as i32;
    }
    tw.racers = racers;
    statuses.0[0] = hud.clone();
}

fn clock_tick(s: &RaceSession) -> bool {
    (s.race_time * 24.0) as i32 % 24 == 0
}

/// render: the car bridge between the last two updates on each car's render root and wheels
#[allow(clippy::too_many_arguments)]
fn place_cars(
    clock: Res<GameClock>,
    bridge: Res<GameBridge>,
    mut cars: ResMut<RaceCars>,
    mut roots: Query<(Entity, &CarRender, &CarRenderParts, &mut Visibility)>,
    mut tfs: Query<&mut Transform>,
    gm: Res<GameManager>,
    kinds: Query<&RaceCar>,
    shake: Res<EngineShake>,
) {
    let players = gm.race.as_ref().map_or(1, |r| r.num_players.max(1) as usize);
    // race cars keep their visibility until their first pose; traffic and police (ids after them) are hidden
    let race_slots = kinds.iter().filter(|c| c.kind == CarKind::Race).count();
    // GameBridge 0x00097C40: skipped while paused
    if !bridge.initialised || gm.paused {
        return;
    }
    let t = bridge.render_lerp(clock.timer.lerp.clamp(0.0, 1.0));
    for (e, cr, parts, mut vis) in &mut roots {
        let slot = cr.request as usize;
        if slot >= bridge.cars.len() {
            continue;
        }
        let pose = bridge.car_pose(slot, t);
        // parked police cars (inactive) are not drawn
        let shown = if pose.valid { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != shown && (pose.valid || slot >= race_slots) {
            *vis = shown;
        }
        if !pose.valid {
            continue;
        }
        // the car models are authored nose +Z, FastCar's body frame is nose +X
        let root = Transform::from_translation(pose.position).with_rotation(pose.rotation * MODEL_TO_BODY);
        // FastCar wheel i -> the render wheel whose built placement is nearest (once)
        if cars.wheel_map.get(slot).is_some_and(|m| m.is_none()) {
            let built: Vec<Vec3> = parts.wheels.iter().map(|w| tfs.get(*w).map(|t| t.translation).unwrap_or_default()).collect();
            let inv = root.compute_affine().inverse();
            let map = std::array::from_fn(|i| {
                let local = inv.transform_point3(pose.wheel_position[i]);
                (0..4).min_by(|&a, &b| built[a].distance_squared(local).total_cmp(&built[b].distance_squared(local))).unwrap_or(i)
            });
            cars.wheel_map[slot] = Some(map);
        }
        let inv = root.compute_affine().inverse();
        let inv_rot = root.rotation.inverse();
        if let Some(map) = cars.wheel_map.get(slot).copied().flatten() {
            for i in 0..4 {
                if let Ok(mut wt) = tfs.get_mut(parts.wheels[map[i]]) {
                    wt.translation = inv.transform_point3(pose.wheel_position[i]);
                    wt.rotation = inv_rot * pose.wheel_rotation[i] * MODEL_TO_BODY;
                }
            }
        }
        if let Ok(mut rt) = tfs.get_mut(e) {
            // CCarRender::EngineShake 0x0011ABB0 (car followed by a player camera): the render matrix's translation
            // moves by +-5 mm / +-1 cm * min(rpm / 8000, 1); flt_624A90-range rands are per update (`EngineShake`)
            let mut shaken = root.translation;
            if slot < players.min(shake.rpm.len()) {
                let k = (shake.rpm[slot] as f32 / 8000.0).min(1.0);
                shaken += Vec3::new(shake.rand[0] * 5.0e-4 * k, shake.rand[1] * 1.0e-3 * k, shake.rand[2] * 5.0e-4 * k);
            }
            rt.translation = shaken;
            rt.rotation = root.rotation;
        }
    }
}

/// CarRender.cpp 0x00112A50 (vehicle inputs): brake input VE+168 > 0, reversing = gear < 0. the original calls
/// 0x001B74A0 per frame (it mutates the shift hold timers), so the port reads its stored gear VE+3216 (car+2976)
/// instead; it differs from the displayed gear only mid-shift, never in sign. light damage +2250 is the renderer's
/// `CarLightDamage` (fed by `car_damage_bridge::forward_car_damage`), so `damage` stays 0 here.
fn car_light_inputs(
    mut commands: Commands,
    physics: Option<Res<PhysicsWorldRes>>,
    roots: Query<(Entity, &CarRender, Option<&CarLightInput>)>,
) {
    let Some(w) = physics.as_deref() else { return };
    let g = w.lock();
    let Some(world) = g.as_ref() else { return };
    let ids = world.vehicle_ids();
    for (e, cr, old) in &roots {
        let id = cr.request as usize;
        if !ids.contains(&id) {
            continue;
        }
        let s = world.slot(id);
        let input = CarLightInput { braking: s.ve.m.f(168) > 0.0, reversing: s.ve.m.i(3216) < 0, damage: 0 };
        if old.is_none_or(|o| o.braking != input.braking || o.reversing != input.reversing || o.damage != input.damage) {
            commands.entity(e).insert(input);
        }
    }
}

/// FastCar +1432 (`WHEEL_OMEGA[0]`) per physics id after the last two updates, [before, latest] (render lerp)
#[derive(Resource, Default, Debug)]
pub struct WheelSpins(pub Vec<Option<[f32; 2]>>);

/// GameTick, after the physics frame: wheel 0's spin of every live vehicle
fn record_wheel_spins(physics: Option<Res<PhysicsWorldRes>>, mut spins: ResMut<WheelSpins>) {
    let Some(w) = physics.as_deref() else { return };
    let g = w.lock();
    let Some(world) = g.as_ref() else { return };
    let ids = world.vehicle_ids();
    let n = ids.iter().copied().max().map_or(0, |m| m + 1);
    if spins.0.len() < n {
        spins.0.resize(n, None);
    }
    for (id, s) in spins.0.iter_mut().enumerate() {
        if !ids.contains(&id) {
            *s = None;
            continue;
        }
        let omega = world.slot(id).car.m.f(velocity_physics::fastcar::car::off::WHEEL_OMEGA);
        // the first update with the car has no earlier value
        *s = Some([s.map_or(omega, |v| v[1]), omega]);
    }
}

/// CarRender 0x00116E30 (`AUDIO_Resample__361500(VE+240, 0)` = FastCar +1432, read every rendered frame): wheel 0's
/// spin on each car root, lerped by `timer.lerp` between the last two updates (the original reads the 24 Hz value
/// raw); frozen while paused like the rest of the render path
fn car_wheel_spins(
    mut commands: Commands,
    clock: Res<GameClock>,
    gm: Res<GameManager>,
    spins: Res<WheelSpins>,
    roots: Query<(Entity, &CarRender, Option<&CarWheelSpin>)>,
) {
    if gm.paused {
        return;
    }
    let t = clock.timer.lerp.clamp(0.0, 1.0);
    for (e, cr, old) in &roots {
        let Some([a, b]) = spins.0.get(cr.request as usize).copied().flatten() else { continue };
        let omega = (b - a) * t + a;
        if old.is_none_or(|o| o.omega != omega) {
            commands.entity(e).insert(CarWheelSpin { omega });
        }
    }
}

/// render: CameraBridge 0x00096A20 — player 0's camera between the last two updates (position, look-at point,
/// up and fov lerped), `look_at_render` (0x0005D540) + fov (0x0005D740) on the camera, then its matrix. Before
/// the bridge exists (or before its second update) the update's own view is drawn.
fn place_camera(
    cars: Res<RaceCars>,
    cams: Res<GameCameras>,
    cut: Res<TimelineCameraView>,
    other: Res<crate::camera::game_cameras::OverrideView>,
    clock: Res<GameClock>,
    bridge: Res<GameBridge>,
    gm: Res<GameManager>,
    mut q: Query<(&mut Transform, &mut Projection)>,
) {
    // GameBridge 0x00097C40: no render interpolation while paused (the last frame stays)
    if gm.paused {
        return;
    }
    // GameBridge 0x00096A20: per player view
    let players = cars.camera.into_iter().chain(cars.player_cameras.iter().copied()).enumerate();
    for (n, c) in players.take(crate::game_bridge::game_bridge::CAMERA_VIEWS) {
        let Ok((mut t, mut p)) = q.get_mut(c) else { continue };
        let latest = if n == 0 { cut.0.or(other.0) } else { None }.or_else(|| cams.manager.view(n));
        let view = if bridge.initialised {
            let cb = &bridge.cameras[n];
            // a record never written (fov 0) is not interpolated from
            if cb.buf[cb.write][0].fov == 0.0 {
                latest
            } else if cb.buf[(cb.write + 1) & 1][0].fov == 0.0 {
                Some(cb.buf[cb.write][0])
            } else {
                Some(bridge.camera_view(n, bridge.render_lerp(clock.timer.lerp.clamp(0.0, 1.0)))[0])
            }
        } else {
            latest
        };
        let Some(v) = view else { continue };
        if v.target != v.position {
            let mut cam = crate::camera::vs_base_camera::VsBaseCamera::default();
            cam.look_at_render(v.position, v.target, v.up);
            *t = cam.render_transform();
        }
        if let Projection::Perspective(pp) = &mut *p {
            pp.fov = v.fov;
        }
    }
}

/// GameManager 0x000B88B0 (view +36 of the mirror view, player 0; dump `research/core_mirror_b88b0.c`): drawn while
/// `byte_685808` && review cameras != 2 && the player's camera mode is follow (1) and its current camera is the
/// static one (+524 == 0) not looking back (+585); placed like `place_camera` from CameraBridge record [1]
/// (0x00096A20, the mirror camera 0x0005F5D0) with the manager's mirror near / far.
pub(crate) fn place_mirror(
    cars: Res<RaceCars>,
    cams: Res<GameCameras>,
    clock: Res<GameClock>,
    bridge: Res<GameBridge>,
    gm: Res<GameManager>,
    mut q: Query<(&mut Transform, &mut Projection, &mut visos_render::view_render::MirrorView)>,
) {
    let Some(e) = cars.mirror_camera else { return };
    let Ok((mut t, mut p, mut mv)) = q.get_mut(e) else { return };
    let pc = cams.manager.players.first();
    // `byte_685808` = the "Graphics.Camera" "Enable Mirror" tweak (.data 1)
    let on = cams.manager.clip.mirror
        && cams.replay.review_cameras != 2
        && pc.is_some_and(|pc| {
            pc.mode == crate::camera::vs_camera_manager::ECameraMode::Follow && pc.follow.as_ref().is_some_and(|f| f.view.chase == 0.0 && !f.unknown_585)
        });
    if mv.on != on {
        mv.on = on;
    }
    if gm.paused || !bridge.initialised {
        return;
    }
    let cb = &bridge.cameras[0];
    let view = if cb.buf[cb.write][1].fov == 0.0 {
        return;
    } else if cb.buf[(cb.write + 1) & 1][1].fov == 0.0 {
        cb.buf[cb.write][1]
    } else {
        bridge.camera_view(0, bridge.render_lerp(clock.timer.lerp.clamp(0.0, 1.0)))[1]
    };
    if view.target != view.position {
        let mut cam = crate::camera::vs_base_camera::VsBaseCamera::default();
        cam.look_at_render(view.position, view.target, view.up);
        *t = cam.render_transform();
    }
    if let Projection::Perspective(pp) = &mut *p {
        pp.fov = view.fov;
        if let Some(pc) = pc {
            pp.near = pc.mirror_near;
            pp.far = pc.mirror_far;
        }
    }
}

/// "CPhysicsManager->Update" (only with the physics plugin in the app)
fn physics_tick(world: &mut World) {
    if world.contains_resource::<velocity_physics::plugin::PhysicsWorldRes>() {
        // one CRT rand stream (0x003DE030): physics draws from the game's state inside the step and hands it back
        // (only when the world existed before the step: a world loaded inside it starts its own seed)
        let state = world.get_resource::<GameManager>().map(|gm| gm.rand.state);
        let mut handed = false;
        if let Some(s) = state {
            if let Some(w) = world.resource::<velocity_physics::plugin::PhysicsWorldRes>().lock().as_mut() {
                w.set_rand_state(s);
                handed = true;
            }
        }
        run_physics_tick(world);
        if handed {
            let after = world.resource::<velocity_physics::plugin::PhysicsWorldRes>().lock().as_ref().map(|w| w.rand_state());
            if let (Some(s), Some(mut gm)) = (after, world.get_resource_mut::<GameManager>()) {
                gm.rand.state = s;
            }
        }
    }
}

/// ZoneRender `flt_737AB0` +24 + 4·pad (graphics' `ZoneFx.level`, pads 0 / 1) mirrored into the physics globals
/// before the step: VsConfigFileRead 0x001B82A0 clears the pad's boost while it is non-zero
fn zone_fx_levels(zone: Option<Res<visos_render::zone_render_pc_xbox::ZoneFx>>, physics: Option<Res<velocity_physics::plugin::PhysicsWorldRes>>) {
    let Some(p) = physics.as_deref() else { return };
    let levels = zone.as_ref().map(|z| z.level).unwrap_or([0.0; 2]);
    // `dword_6F5B70`: the zone amount written per rendered view by ZoneRender 0x00162D64 (0x00154FC0 of the view
    // player) and zeroed by FuryFX 0x001297F0 / 0x001298E0 (both graphics' `ZoneFx.amount_global`); read by the
    // physics frame (PhysicsManager 0x0019F920, VsConfigFileRead 0x001B82A0)
    let global = zone.as_ref().map_or(0.0, |z| z.amount_global);
    if let Some(w) = p.lock().as_mut() {
        for (pad, l) in levels.iter().enumerate() {
            w.set_zone_fx_level(pad, *l);
        }
        w.glob.w.insert(0x006F_5B70, global.to_bits());
    }
}

fn not_paused(gm: Res<GameManager>) -> bool {
    !gm.paused
}

pub(crate) fn build(app: &mut App) {
    if !app.world().contains_resource::<GameAudioInput>() {
        app.init_resource::<GameAudioInput>();
    }
    if !app.world().contains_resource::<Messages<LoadTrackRender>>() {
        app.add_message::<LoadTrackRender>().add_message::<UnloadTrackRender>();
    }
    if !app.world().contains_resource::<Messages<LoadCatsEyes>>() {
        app.add_message::<LoadCatsEyes>().add_message::<UnloadCatsEyes>();
    }
    if !app.world().contains_resource::<Messages<SpawnCarRender>>() {
        app.add_message::<SpawnCarRender>().add_message::<DespawnCarRender>();
    }
    if !app.world().contains_resource::<Messages<LoadPhysicsTrack>>() {
        app.add_message::<LoadPhysicsTrack>().add_message::<UnloadPhysicsTrack>();
    }
    if !app.world().contains_resource::<Messages<LoadPhysicsRaceObjects>>() {
        app.add_message::<LoadPhysicsRaceObjects>();
    }
    if !app.world().contains_resource::<Messages<velocity_physics::plugin::ResetPhysicsObject>>() {
        app.add_message::<velocity_physics::plugin::ResetPhysicsObject>()
            .add_message::<velocity_physics::plugin::PhysicsObjectNoticeMessage>();
    }
    if !app.world().contains_resource::<Messages<velocity_physics::plugin::PhysicsExplosion>>() {
        app.add_message::<velocity_physics::plugin::PhysicsExplosion>();
    }
    if !app.world().contains_resource::<Messages<velocity_physics::plugin::RestartPhysicsWorld>>() {
        app.add_message::<velocity_physics::plugin::RestartPhysicsWorld>();
    }
    if !app.world().contains_resource::<Messages<velocity_physics::plugin::PhysicsPuddleMessage>>() {
        app.add_message::<velocity_physics::plugin::PhysicsPuddleMessage>();
    }
    if !app.world().contains_resource::<Messages<visos_render::puddle_render::PuddleAdd>>() {
        app.add_message::<visos_render::puddle_render::PuddleAdd>();
    }
    if !app.world().contains_resource::<Messages<velocity_physics::plugin::ResetAllPhysicsObjects>>() {
        app.add_message::<velocity_physics::plugin::ResetAllPhysicsObjects>()
            .add_message::<velocity_physics::plugin::PhysicsInstanceMove>();
    }
    if !app.world().contains_resource::<PhysicsTimestep>() {
        app.init_resource::<PhysicsTimestep>();
    }
    crate::controller::vibrate_effects::add_vibration(app);
    if !app.world().contains_resource::<Messages<SetupCarDamage>>() {
        app.add_message::<SetupCarDamage>().add_message::<RepairCarDamage>().add_message::<PhysicsCarDamage>();
    }
    if !app.world().contains_resource::<Messages<visos_render::car_damage_fx::GlassFxFired>>() {
        app.add_message::<visos_render::car_damage_fx::GlassFxFired>();
    }
    if !app.world().contains_resource::<velocity_physics::car_damage::CarDamageManager>() {
        app.init_resource::<velocity_physics::car_damage::CarDamageManager>();
    }
    if !app.world().contains_resource::<Messages<visos_render::car_damage_render::CarDamageRenderEvent>>() {
        app.add_message::<visos_render::car_damage_render::CarDamageRenderEvent>();
    }
    if !app.world().contains_resource::<Messages<crate::game_mode::trail_blazer::TrailBlazerMessage>>() {
        app.add_message::<crate::game_mode::trail_blazer::TrailBlazerMessage>();
    }
    app.init_resource::<crate::game_mode::trail_blazer::TrailBlazerCalls>()
        .init_resource::<PlayerCarChoice>()
        .init_resource::<RaceStatus>()
        .init_resource::<RaceCars>()
        .init_resource::<RacelineLength>()
        .add_systems(PostUpdate, race_load_steps.run_if(in_state(EGameMode::InGame)))
        .add_systems(PostUpdate, car_damage_bridge::forward_glass.run_if(in_state(EGameMode::InGame)))
        .add_systems(OnExit(crate::InRace), despawn_race)
        .add_systems(
            GameTick,
            (
                race_mileage,
                race_release,
                car_inputs,
                car_damage_bridge::setup_car_damage,
                car_damage_bridge::repair_on_restart,
                crate::game_mode::trail_blazer::apply_trail_blazer_calls,
                crate::timeline_manager::race_timeline_sequencer::trail_blazer_timeline_calls,
                crate::timeline_manager::race_timeline_sequencer::send_object_resets,
                zone_fx_levels,
                crate::game_mode::jump_cam::jump_cam_pre,
                physics_tick,
                crate::game_mode::jump_cam::jump_cam_post,
                crate::timeline_manager::race_timeline_sequencer::record_object_notices,
                crate::game_mode::race_effects_bridge::forward_puddles,
                car_outputs,
                record_wheel_spins,
                car_damage_bridge::forward_car_damage,
            )
                .chain()
                .in_set(InGameUpdateSet::Physics)
                .run_if(in_state(EGameMode::InGame))
                .run_if(not_paused),
        )
        .add_systems(
            GameTick,
            crate::game_mode::jump_cam::review_timer_countdown
                .in_set(InGameUpdateSet::Effects)
                .run_if(in_state(EGameMode::InGame))
                .run_if(not_paused),
        )
        .init_resource::<crate::game_mode::jump_cam::JumpCam>()
        .init_resource::<crate::game_mode::jump_cam::SystemUpdateRate>()
        .init_resource::<crate::game_mode::game_views::GameViews>()
        .add_systems(PostUpdate, init_game_views.run_if(in_state(EGameMode::InGame)))
        .init_resource::<PlayerRaceStatus>()
        .init_resource::<RacerPoints>()
        .add_systems(OnExit(crate::InRace), |mut p: ResMut<RacerPoints>| p.0.clear())
        .add_systems(Update, track_views.run_if(in_state(crate::InRace)))
        .add_systems(OnEnter(crate::InRace), crate::game_mode::jump_cam::reset_system_update_rate)
        .init_resource::<WheelSpins>()
        .add_systems(Update, (place_cars, place_camera, place_mirror, car_light_inputs, car_wheel_spins).run_if(in_state(EGameMode::InGame)));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// the player's LADown circuit grid slot at the eventdef height (~3 m above the collision, as the original
    /// spawns it): the car drops, settles and stays there
    #[test]
    fn ladown_grid_car_settles() {
        if visos::file::platform::try_game_dir().is_err() {
            return;
        }
        use velocity_physics::plugin::PhysicsPlugin;
        let mut app = App::new();
        app.add_plugins(PhysicsPlugin);
        app.world_mut().write_message(LoadPhysicsTrack { track: "LADOWN".into() });
        run_physics_tick(app.world_mut());
        let p = Vec3::new(395.8287, 9.4137, 268.6233);
        let mut inputs = LocalConfigInputs::default_spans("240SX");
        inputs.is_player = true;
        let e = app.world_mut().spawn((
            Transform::from_translation(p).with_rotation(Quat::from_rotation_y((-87.23545f32 - 90.0).to_radians())),
            PhysicsCarSpawn { inputs, pad_slot: 0 },
            PhysicsCarInput { handbrake: true, ..Default::default() },
        )).id();
        for _ in 0..72 {
            run_physics_tick(app.world_mut());
        }
        let end = app.world().get::<PhysicsCarState>(e).unwrap().position;
        assert!(end.is_finite() && Vec2::new(end.x - p.x, end.z - p.z).length() < 1.0 && end.y < p.y, "the car settles on its grid slot: {end:?}");
    }

    /// a jumped LADown circuit race: the players then the temp event's opponents, on distinct grid slots, ids in order
    #[test]
    fn plan_has_opponents() {
        if visos::file::platform::try_game_dir().is_err() {
            return;
        }
        struct Src;
        impl RaceCarSource for Src {
            fn opponent_car(&self, db: &CVehicleDatabase, _crc: u32, _i: usize) -> Option<(VehicleSetup, String)> {
                let mut s = VehicleSetup::new();
                db.setup_default_vehicle(&mut s, string_hash("350Z")).then_some((s, "X".into()))
            }
            fn team_car(&self, _db: &CVehicleDatabase, _crc: u32) -> Option<VehicleSetup> {
                None
            }
        }
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        crate::game_mode::game_manager::build_with(
            &mut app,
            crate::main_cpp::CommandLine::parse(&["jumptogame".into(), r"TRACKS\LADOWN\".into(), r"CIRCUIT\RACE01\".into()]),
        );
        let tick = 1_000_000_000 / 24 + 1;
        for i in 0..6u64 {
            app.world_mut().resource_mut::<crate::vs_common_main::GameClock>().manual = Some(i * tick);
            app.update();
        }
        let gm = app.world().resource::<GameManager>();
        let db = CVehicleDatabase::load().unwrap();
        let plan = plan_race_cars(gm, &db, &VehicleSetup::new(), None, Some(&Src), None);
        eprintln!("{:?}", plan.iter().map(|p| (p.id, p.grid, p.human)).collect::<Vec<_>>());
        assert!(plan.len() > 1 && plan[0].human && plan.iter().skip(1).all(|p| !p.human));
        assert!(plan.iter().enumerate().all(|(i, p)| p.id == i));
        let mut g: Vec<_> = plan.iter().map(|p| p.grid).collect();
        g.dedup();
        assert_eq!(g.len(), plan.len());
    }

    /// every vehicle's default setup: its fitted parts all map to render slots, and the paint is not black
    #[test]
    fn default_setup_specs() {
        if visos::file::platform::try_game_dir().is_err() {
            return;
        }
        let db = CVehicleDatabase::load().unwrap();
        let mut n = 0;
        for v in &db.vehicles {
            let mut s = VehicleSetup::new();
            if !db.setup_default_vehicle(&mut s, v.hash) {
                continue;
            }
            let spec = car_spec(&db, &s, "");
            let names: Vec<_> = s.fitted_bodyparts.iter().flatten().filter_map(|r| db.part(*r).string_10.clone()).collect();
            assert_eq!(spec.parts.iter().flatten().count(), names.len(), "{} {names:?}", spec.dir);
            assert!(spec.colour != Vec3::ZERO, "{}", spec.dir);
            n += 1;
        }
        assert!(n > 0);
    }

    #[test]
    fn vehicle_dirs() {
        assert_eq!(vehicle_dir(r"\VEHICLES\LA\240SX"), ("LA/240SX".to_string(), "240SX".to_string()));
    }
}
