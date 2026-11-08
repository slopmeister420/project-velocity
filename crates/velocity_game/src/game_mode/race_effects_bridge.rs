//! glue from game flow and physics to the effect crates (not an original module; each piece cites the call site
//! it stands for): the race's particle definitions, `EffectCars` + the per-update race effects, the physics
//! objects' particles / sounds / timelines (PhysicsObjectManager 0x001E1F80, PhysicsObjectParticleTracker
//! 0x001E4C30, the immovable pass 0x001A89F0) and the flag girl's start line fire.

use bevy::prelude::*;
use velocity_audio::AudioCommand;
use velocity_physics::physics_objects::PhysicsObjectEvent;
use velocity_physics::plugin::{PhysicsCar, PhysicsCarState, PhysicsCollision, PhysicsObjectMessage, PhysicsSlipstream, PhysicsWorldRes};
use visos_render::effect_draw::EffectClock;
use visos_render::full_screen_effects_manager_pc_xbox::FseCameraPlanes;
use visos_render::motion_blur_pc_xbox::MotionBlurInput;
use visos_render::race_effects::{
    race_effects_update, EffectCar, EffectCars, EffectWheel, RaceEffects, RaceEffectsInit, RaceEffectsReset,
    RaceEffectsShutdown, SlipstreamStrength, SparkCollisionMessage, StartLineFire,
};
use visos_render::spark_render::SparkCollision;
use visos_render::barrier_render::{BarrierHit, UnloadBarrierRender};
use visos_render::danger_zone_render::{DangerParticleCall, DangerZoneInput, DepthOfField, DepthOfFieldInput};
use visos_render::trail_blazer_render::{trail_blazer_render_update, TrailBlazerRender, TrailBlazerRenderCall};
use visos_render::zone_render_pc_xbox::{ZoneFx, ZoneFxAudio, ZoneFxInput, ZoneFxSecondInput};
use visos_render::car_render::CarDriverInput;
use visos_render::fade_render::FadeRender;
use visos_render::lens_spray_manager::{LensSpray, LensSprayInput};
use visos_render::camera_rain::{CameraRainInit, CameraRainInput, CameraRainReset, CameraRainShutdown};

use crate::camera::game_cameras::{current_camera_position, current_planes, GameCameras};
use crate::camera::vs_camera_manager::ECameraMode;
use crate::game_bridge::game_bridge::GameBridge;
use crate::game_mode::trail_blazer::{TrailBlazerMessage, TrailEvent};
use crate::timeline_manager::objects::timeline_object::{TimelineObjectMessage, TimelineWorld};
use visos_render::stateless_particles::{LoadParticles, ObjectCollisionParticles, ParticleGroup, ParticleTrail, UnloadParticles};

use crate::game_data_loading::game_data_loading::ELoadStep;
use crate::game_mode::game_manager::{EGameMode, GameManager, GameModeMessage, LoadStepMessage};
use crate::game_mode::race_cars::{CarKind, RaceCar};
use crate::game_mode::race_session::{RaceSession, RaceSessionMessage};
use crate::timeline_manager::timeline_sequencer::FlagGirlEvent;
use crate::triggers::trigger_update::TriggerMessage;
use crate::vs_common_main::{GameClock, GameTick, InGameUpdateSet};

/// `VsAudio::AddEnvSound` event of the fire hydrant spray (PhysicsObjectManager 0x001E1F80 case 0x17)
const HYDRANT_SOUND: u32 = -33188540i32 as u32;

/// RaceEffectsInit was sent for this race (GameManager::InGameInit runs the effect inits once the cars exist)
#[derive(Resource, Default)]
struct EffectsStarted(bool);

/// GameManager::InGameInit 0x000B63FA "Particle Init": PartFX.bin, generic + track + race particle definitions
fn particle_load_steps(mut r: MessageReader<LoadStepMessage>, gm: Res<GameManager>, mut load: MessageWriter<LoadParticles>) {
    for m in r.read() {
        if m.step != ELoadStep::ParticleManagerInit {
            continue;
        }
        let Some(race) = &gm.race else { continue };
        let paths = crate::game_data_loading::game_data_loading::RacePaths::new(&race.track_path, &race.race_path);
        let (area, dir) = (&paths.track_dir, &paths.race_dir);
        let area = if area.ends_with('\\') { area.clone() } else { format!("{area}\\") };
        // `\Tracks\LACOAST\` + `CIRCUIT\RACE01\`
        let sub = dir.get(area.len()..).filter(|_| dir.len() >= area.len() && dir[..area.len()].eq_ignore_ascii_case(&area));
        load.write(LoadParticles { area: area.clone(), sub: sub.map(str::to_string) });
    }
}

/// `g_FadeRender.Start(t)` (DangerZoneRender 0x00123070) from outside core: the ai's respawn (VsAIRaceVehicle
/// 0x00022C90 start(delay), 0x00018A40 start(-delay)). Applied in the GameTick Effects set before the fade update,
/// so write it in the same tick (Ai set).
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct FadeStart(pub f32);

/// CommonInit "FadeRender Init" (0x00122FB0, front end and in game) and InGameInit "Lens Spray Render Init"
/// (0x0012D400)
fn render_load_steps(mut r: MessageReader<LoadStepMessage>, fade: Option<ResMut<FadeRender>>, lens: Option<ResMut<LensSpray>>) {
    let (mut fade, mut lens) = (fade, lens);
    for m in r.read() {
        match m.step {
            ELoadStep::FadeRenderInit => {
                if let Some(f) = fade.as_deref_mut() {
                    f.init();
                    log::info!("fade render init");
                }
            }
            ELoadStep::LensSprayRender => {
                if let Some(l) = lens.as_deref_mut() {
                    l.init();
                    log::info!("lens spray init");
                }
            }
            _ => {}
        }
    }
}

/// GameManager::InGameUpdate "g_LensSprayManager.Update" (0x0012D520): the camera manager's current camera, the
/// forward speed of that player's vehicle (0x0012DC20 = manager +0x60 -> VsAIDriver 0x00015DB0 -> VsAIRaceVehicle
/// 0x00024870 = FastCar +32 velocity . +112 row 0), `VsAudio::IsTrackWet` = the race's wet flag
fn lens_spray_update(
    q: Query<(&RaceCar, &PhysicsCar)>,
    physics: Option<Res<PhysicsWorldRes>>,
    gm: Res<GameManager>,
    cams: Res<GameCameras>,
    lens: Option<ResMut<LensSpray>>,
    inp: Option<ResMut<LensSprayInput>>,
) {
    let Some(mut lens) = lens else { return };
    // players hold the first race slots
    let player = cams.manager.unknown_60 as usize;
    let id = q.iter().find(|(c, _)| c.kind == CarKind::Race && c.human && c.slot == player).map(|(_, pc)| pc.id);
    let mut forward_speed = 0.0;
    if let (Some(id), Some(g)) = (id, physics.as_deref().map(|p| p.lock())) {
        if let Some(w) = g.as_ref().filter(|w| w.vehicle_ids().contains(&id)) {
            let (row, vel) = (w.slot(id).car.m.v3(112), w.velocity(id));
            forward_speed = row[0] * vel[0] + row[1] * vel[1] + row[2] * vel[2];
        }
    }
    let input = LensSprayInput {
        camera: current_camera_position(&cams.manager),
        forward_speed,
        wet: gm.race.as_ref().is_some_and(|r| r.wet),
    };
    lens.update(&input);
    if let Some(mut i) = inp {
        *i = input;
    }
}

/// the ai's fade starts of this update (FadeStart), then "g_FadeRender.Update" (0x00123100 with GameManager
/// 0x000BF690's clock; the paused branch of InGameUpdate does not call it)
fn fade_update(mut r: MessageReader<FadeStart>, gm: Res<GameManager>, clock: Res<GameClock>, fade: Option<ResMut<FadeRender>>) {
    let Some(mut fade) = fade else {
        r.clear();
        return;
    };
    for m in r.read() {
        fade.start(m.0);
    }
    fade.update(clock.timer.update_time(), gm.paused);
}

/// InGameShutdown / RestartRace side effects of the effect managers
#[allow(clippy::too_many_arguments)]
fn mode_messages(
    mut r: MessageReader<GameModeMessage>,
    mut started: ResMut<EffectsStarted>,
    mut unload: MessageWriter<UnloadParticles>,
    mut shutdown: MessageWriter<RaceEffectsShutdown>,
    mut reset: MessageWriter<RaceEffectsReset>,
    mut barriers: MessageWriter<UnloadBarrierRender>,
    mut fade: Option<ResMut<FadeRender>>,
    mut trails: Option<ResMut<TrailBlazerRender>>,
) {
    for m in r.read() {
        match m {
            GameModeMessage::InGameShutdown => {
                // BarrierRender 0x00107CE0 (the load, 0x00107560, needs the ai world: binary `barriers.rs`)
                barriers.write(UnloadBarrierRender);
                shutdown.write(RaceEffectsShutdown);
                unload.write(UnloadParticles);
                started.0 = false;
                // zone trails 0x0014E360: the original frees them only at the next Init (0x0014DF70 → 0x0014E280)
                // or exit, but draws them only in the race; cleared here so the front end draws none
                if let Some(t) = trails.as_deref_mut() {
                    t.shutdown();
                }
            }
            // GameManager::RestartRace: start line effects, sparks, skid mark start line records
            GameModeMessage::RaceRestart => {
                reset.write(RaceEffectsReset);
                // 0x000BE234: g_FadeRender reset (0x00123030)
                if let Some(f) = fade.as_deref_mut() {
                    f.reset();
                }
                // 0x000BE2BA: CZoneTrailRender reset (0x0014E2A0)
                if let Some(t) = trails.as_deref_mut() {
                    t.reset();
                }
            }
            _ => {}
        }
    }
}

/// physics' puddles (PhysicsObjectManager 0x001E1F80 case 0x17 -> LightStreak__133750 on `unk_720840`), read in the
/// same GameTick right after the physics step
pub(crate) fn forward_puddles(
    mut r: MessageReader<velocity_physics::plugin::PhysicsPuddleMessage>,
    mut w: MessageWriter<visos_render::puddle_render::PuddleAdd>,
) {
    for m in r.read() {
        w.write(visos_render::puddle_render::PuddleAdd((&m.0).into()));
    }
}

/// GameManager::RestartRace 0x000BE20C -> 0x00131EC0: the runtime puddles go again
fn puddle_restart(mut modes: MessageReader<GameModeMessage>, mut w: MessageWriter<visos_render::puddle_render::PuddleRestart>) {
    for m in modes.read() {
        if *m == GameModeMessage::RaceRestart {
            w.write(visos_render::puddle_render::PuddleRestart);
        }
    }
}

/// camera rain (`unk_721DD0`): InGameInit "Rain Render Init" (0x000B65A5 → 0x00133AB0, wet = `VsAudio::IsTrackWet`),
/// RestartRace 0x000BE2E2 → 0x00133CF0, shutdown 0x00133CD0 (registered by the init on the VsAIWorld destructor
/// list, so it runs with the race's shutdown)
fn camera_rain_steps(
    mut steps: MessageReader<LoadStepMessage>,
    mut modes: MessageReader<GameModeMessage>,
    gm: Res<GameManager>,
    mut init: MessageWriter<CameraRainInit>,
    mut reset: MessageWriter<CameraRainReset>,
    mut shutdown: MessageWriter<CameraRainShutdown>,
) {
    // a mode's shutdown precedes the next mode's init steps (SetGameMode 0x000BBD30)
    for m in modes.read() {
        match m {
            GameModeMessage::RaceRestart => {
                reset.write(CameraRainReset);
            }
            GameModeMessage::InGameShutdown => {
                shutdown.write(CameraRainShutdown);
            }
            _ => {}
        }
    }
    for m in steps.read() {
        if m.step == ELoadStep::RainRender {
            let wet = gm.race.as_ref().is_some_and(|r| r.wet);
            log::info!("camera rain init: wet {wet}");
            init.write(CameraRainInit { wet });
        }
    }
}

/// "Car Render Init" (FrontEndInit 0x000B5829 push 1 / InGameInit 0x000B6E52 push 0 → ZoneRender 0x00163770 →
/// CarPalette 0x0010D7E0 → DangerZoneRender 0x001214A0, `byte_6F7157`); 0x00163770 puts 0x001637B0 on the
/// VsAIWorld destructor list, which a mode's shutdown runs
fn dynamic_envmap_steps(
    mut steps: MessageReader<LoadStepMessage>,
    mut modes: MessageReader<GameModeMessage>,
    gm: Res<GameManager>,
    mut init: MessageWriter<visos_render::car_shader_manager_pc_xbox::DynamicEnvmapInit>,
    mut shutdown: MessageWriter<visos_render::car_shader_manager_pc_xbox::DynamicEnvmapShutdown>,
) {
    for m in modes.read() {
        if matches!(m, GameModeMessage::FrontEndShutdown | GameModeMessage::InGameShutdown) {
            shutdown.write(visos_render::car_shader_manager_pc_xbox::DynamicEnvmapShutdown);
        }
    }
    for m in steps.read() {
        let front_end = match m.step {
            ELoadStep::FeCarRender => true,
            ELoadStep::CarRender => false,
            _ => continue,
        };
        let two_players = gm.race.as_ref().is_some_and(|r| r.num_players > 1);
        log::info!("dynamic envmap init: front end {front_end}, two players {two_players}");
        init.write(visos_render::car_shader_manager_pc_xbox::DynamicEnvmapInit { front_end, two_players });
    }
}

/// per rendered frame (GameManager 0x000B9F43, the rain update): VsCameraManager 0x0005FFB0(0) == follow, and the
/// race manager's driver 0 position (RainSurface 0x00135FB0 +20, read raw: the latest update's)
fn camera_rain_input(
    q: Query<(&RaceCar, &PhysicsCarState)>,
    cams: Res<GameCameras>,
    mut input: ResMut<CameraRainInput>,
) {
    let car0 = q.iter().find(|(c, _)| c.kind == CarKind::Race && c.slot == 0).map(|(_, st)| st.position);
    let v = CameraRainInput { follow: cams.manager.mode(0) == ECameraMode::Follow, car0 };
    if input.follow != v.follow || input.car0 != v.car0 {
        *input = v;
    }
}

fn clear_camera_rain_input(mut input: ResMut<CameraRainInput>) {
    *input = CameraRainInput::default();
}

/// the physics objects' events where the original raises them: debris (ParticleManager 0x000FFC30), trackers'
/// trails (0x001000E0), VsAudio::PlayPhysicsImpact, RaceTimelineSequencer::PlayTimeline (0x001711E0, read by the
/// Timeline set later in this update) and the hydrant spray sound (`VsAudio::AddEnvSound(-33188540, pos, 10,
/// 100, 50)`; the spray object itself, 0x001E18E0, is not simulated)
fn forward_physics_objects(
    mut r: MessageReader<PhysicsObjectMessage>,
    mut debris: MessageWriter<ObjectCollisionParticles>,
    mut trail: MessageWriter<ParticleTrail>,
    mut audio: MessageWriter<AudioCommand>,
    mut trig: MessageWriter<TriggerMessage>,
) {
    for PhysicsObjectMessage(e) in r.read() {
        match *e {
            PhysicsObjectEvent::CollisionParticles { kind, object, point, impulse } => {
                debris.write(ObjectCollisionParticles { kind, object: Mat4::from_cols_array(&object), point: Vec3::from(point), impulse });
            }
            PhysicsObjectEvent::Trail { from, to, event } => {
                trail.write(ParticleTrail { from: Vec3::from(from), to: Vec3::from(to), event });
            }
            PhysicsObjectEvent::Impact { sound, mapped, point, impulse, again } => {
                let event = if mapped { sound } else { velocity_audio::vs_audio_events::map_old_id(sound as i32) };
                audio.write(AudioCommand::Impact { event, position: Vec3::from(point), strength: impulse, car_layer_only: again });
            }
            PhysicsObjectEvent::PlayTimeline { timeline } => {
                trig.write(TriggerMessage::Timeline { name_crc: timeline });
            }
            PhysicsObjectEvent::Explosion { point } => {
                audio.write(AudioCommand::AddEnvSound { event: HYDRANT_SOUND, position: Vec3::from(point), min: 10.0, volume: 100.0, max: 50.0 });
            }
        }
    }
}

/// `dword_68676C`: spark height above the car for car-car and wall hits
const SPARK_LIFT: f32 = f32::from_bits(0x3E99_999A);

/// collision message listener CarPalette 0x0010D5F0: a hit against another car or a wall (`b != -1`) moves the
/// point to the first car's height + 0.3 (in the message itself), then SparkRender 0x00141980, the empty audio
/// call 0x00052D10 and BarrierRender 0x0010B590 (which returns at once when `byte_6F70E9` professional is set; the
/// car 0 / material 21 checks are the renderer's); `SlipstreamRender::SetStrength` 0x0013FB30 1:1
#[allow(clippy::too_many_arguments)]
fn forward_collisions(
    mut hits: MessageReader<PhysicsCollision>,
    mut slip: MessageReader<PhysicsSlipstream>,
    physics: Option<Res<PhysicsWorldRes>>,
    gm: Res<GameManager>,
    mut sparks: MessageWriter<SparkCollisionMessage>,
    mut barrier: MessageWriter<BarrierHit>,
    mut strength: MessageWriter<SlipstreamStrength>,
) {
    let professional = gm.race.as_ref().is_some_and(|r| r.professional);
    let g = physics.as_deref().map(|p| p.lock());
    let world = g.as_ref().and_then(|g| g.as_ref());
    for PhysicsCollision(c) in hits.read() {
        let mut point = Vec3::from(c.point);
        if c.b != -1 {
            if let Some(w) = world.filter(|w| c.a >= 0 && w.vehicle_ids().contains(&(c.a as usize))) {
                point.y = w.position(c.a as usize)[1] + SPARK_LIFT;
            }
        }
        sparks.write(SparkCollisionMessage(SparkCollision {
            car: c.a as usize,
            other: c.b,
            point,
            normal: Vec3::from(c.normal),
            impulse: c.impact,
            scale: c.scale,
            material: c.material,
        }));
        if !professional {
            barrier.write(BarrierHit { car_a: c.a, material: c.material, point });
        }
    }
    for s in slip.read() {
        // the original's index is unsigned (`nVehicleIndex < MAX_VEHICLE_COUNT` assert)
        strength.write(SlipstreamStrength { vehicle: s.vehicle as usize, strength: s.strength });
    }
}

/// GameManager 0x000B9010's motion blur inputs for view 0 (the player's car, slot 0): gate (view flag 0x10 is set
/// on race views, `byte_6F7157` two players, attract mode, `dword_749110`), VsAIRaceVehicle 0x00024870 speed
/// (FastCar +32 velocity . +112 row 0), VE+0x44 (0x00130070), wheel contacts (0x00133970 -> car +1088 + 4i > 0),
/// position 0x001BA270, velocity 0x001C66C0. FuryFX 0x00128680 is not ported (0); the zone amount 0x00154FC0 is
/// written per frame by `fill_render_inputs`.
/// The original reads these raw at render time, i.e. the latest update's values, as here.
fn fill_motion_blur(
    q: Query<(&RaceCar, &PhysicsCar)>,
    physics: Option<Res<PhysicsWorldRes>>,
    gm: Res<GameManager>,
    cams: Res<GameCameras>,
    mut inp: ResMut<MotionBlurInput>,
    mut second: ResMut<visos_render::zone_render_pc_xbox::MotionBlurSecondInput>,
    mut planes: ResMut<FseCameraPlanes>,
    attract: Res<crate::font_helper::CAttractMode>,
) {
    let mut out = MotionBlurInput::default();
    let mut out1 = None;
    let two_players = gm.race.as_ref().is_some_and(|r| r.num_players > 1);
    // 0x000B96B9: `FontHelper__B36B0(byte_6F5A28) == 0 && dword_749110 == 0`
    let gate = !two_players && !attract.playing && cams.replay.review_cameras == 0;
    if let Some(g) = physics.as_deref().map(|p| p.lock()) {
        if let Some(w) = g.as_ref() {
            let of = |slot: usize, zone: f32| {
                let player = q.iter().find(|(c, _)| c.kind == CarKind::Race && c.slot == slot).map(|(_, pc)| pc.id);
                player.filter(|id| w.vehicle_ids().contains(id)).map(|id| {
                    let s = w.slot(id);
                    let (car, ve) = (&s.car.m, &s.ve.m);
                    let (row, vel) = (car.v3(112), w.velocity(id));
                    MotionBlurInput {
                        active: gate,
                        speed: row[0] * vel[0] + row[1] * vel[1] + row[2] * vel[2],
                        ve_44: ve.f(0x44),
                        fury: 0.0,
                        // ZoneRender 0x00154FC0, set per rendered frame by `fill_render_inputs`
                        zone,
                        position: Vec3::from(w.position(id)),
                        velocity: Vec3::from(vel),
                        wheels_down: (0..4).all(|i| car.i(1088 + 4 * i) > 0),
                    }
                })
            };
            if let Some(o) = of(0, inp.zone) {
                out = o;
            }
            // the zone FX focus of player 1's view (0x00160920(view +28) -> VehicleManager 0x001DDCC0(1))
            if two_players {
                out1 = of(1, 0.0);
            }
        }
    }
    *inp = out;
    if second.0.is_some() || out1.is_some() {
        second.0 = out1;
    }
    if let Some((near, far)) = current_planes(&cams.manager, 0) {
        if planes.near != near || planes.far != far {
            *planes = FseCameraPlanes { near, far };
        }
    }
}

fn clear_motion_blur(
    mut inp: ResMut<MotionBlurInput>,
    mut second: ResMut<visos_render::zone_render_pc_xbox::MotionBlurSecondInput>,
    mut feed: ResMut<RenderFeed>,
    mut zone: Option<ResMut<ZoneFxInput>>,
    mut dof: Option<ResMut<DepthOfFieldInput>>,
    mut danger: Option<ResMut<DangerZoneInput>>,
    mut commands: Commands,
) {
    // absent = plain racing (front end car draws pass their own flag)
    commands.remove_resource::<CarDriverInput>();
    *inp = MotionBlurInput::default();
    second.0 = None;
    *feed = RenderFeed::default();
    if let Some(z) = zone.as_deref_mut() {
        *z = ZoneFxInput::default();
    }
    commands.insert_resource(ZoneFxSecondInput(None));
    if let Some(d) = dof.as_deref_mut() {
        *d = DepthOfFieldInput::default();
    }
    if let Some(d) = danger.as_deref_mut() {
        *d = DangerZoneInput::default();
    }
}

/// what the render-side effect inputs read from the last two updates (the view player's car = race slot 0)
#[derive(Resource, Default, Clone, Debug)]
struct RenderFeed {
    /// physics id of the player's car
    player: Option<usize>,
    /// VE+20 speed (`VsAIRaceVehicle__24220` is called on the VehicleEntity), [before, latest]
    speed: [f32; 2],
    /// linear velocity (VehicleEntity 0x001C6480 / VsAIRaceVehicle 0x00024200), [before, latest]
    velocity: [Vec3; 2],
    /// VE+89 zone on (tyrespray 0x00155990)
    zone_on: bool,
    /// 0x00023310: player 0's race vehicle runs a trailblazer (`PlayerTrailBlazers`)
    one_off: bool,
    /// DangerZoneRender `unk_718AC0` as TrailBlazer 0x000C5F90 / 0x000C6080 / 0x000C6120 leave it (+29 is 1 after
    /// init 0x001202F0, so this holds its negation)
    danger_disabled: bool,
    danger_on: bool,
    danger_prev: [f32; 5],
    danger_cur: [f32; 5],
    danger_success: bool,
    /// player 1's car in a two-player race (its view's 0x00162D10 call)
    second: Option<ZoneFeedPlayer>,
}

/// one more view player's zone FX values (same sources as player 0's fields)
#[derive(Clone, Copy, Debug, Default)]
struct ZoneFeedPlayer {
    id: usize,
    speed: [f32; 2],
    zone_on: bool,
    one_off: bool,
}

/// GameTick Effects set: the player's car values the zone FX reads (VehicleManager 0x001DDCC0 → VE+20 / +89)
fn fill_render_feed(
    q: Query<(&RaceCar, &PhysicsCar)>,
    physics: Option<Res<PhysicsWorldRes>>,
    trails: Option<Res<crate::game_mode::trail_blazer::PlayerTrailBlazers>>,
    mut feed: ResMut<RenderFeed>,
) {
    let player = q.iter().find(|(c, _)| c.kind == CarKind::Race && c.slot == 0).map(|(_, pc)| pc.id);
    let g = physics.as_deref().map(|p| p.lock());
    let world = g.as_ref().and_then(|g| g.as_ref());
    match (player, world) {
        (Some(id), Some(w)) if w.vehicle_ids().contains(&id) => {
            let ve = &w.slot(id).ve.m;
            let speed = ve.f(20);
            // the first update with the car has no earlier value
            let before = if feed.player == Some(id) { feed.speed[1] } else { speed };
            let vel = Vec3::from(w.velocity(id));
            let vel_before = if feed.player == Some(id) { feed.velocity[1] } else { vel };
            feed.player = Some(id);
            feed.speed = [before, speed];
            feed.velocity = [vel_before, vel];
            let on = ve.b(89) != 0;
            if on != feed.zone_on {
                log::info!("zone {} (VE+104 meter {:.1}, speed {speed:.1})", if on { "on" } else { "off" }, ve.f(104));
            }
            feed.zone_on = on;
            feed.one_off = trails.as_ref().is_some_and(|t| t.0[0].is_some());
        }
        _ => {
            feed.player = None;
            feed.zone_on = false;
            feed.one_off = false;
        }
    }
    // player 1 (humans are race slots 0..n-1)
    let second = q.iter().find(|(c, _)| c.kind == CarKind::Race && c.human && c.slot == 1).map(|(_, pc)| pc.id);
    feed.second = match (second, world) {
        (Some(id), Some(w)) if w.vehicle_ids().contains(&id) => {
            let ve = &w.slot(id).ve.m;
            let speed = ve.f(20);
            let before = feed.second.filter(|s| s.id == id).map_or(speed, |s| s.speed[1]);
            Some(ZoneFeedPlayer {
                id,
                speed: [before, speed],
                zone_on: ve.b(89) != 0,
                one_off: trails.as_ref().is_some_and(|t| t.0.get(1).is_some_and(|b| b.is_some())),
            })
        }
        _ => None,
    };
}

/// TrailBlazer 0x000C5F90 (each trailblazer's update writes the one global danger object, last writer wins),
/// 0x000C6080 / 0x000C6120 (+29); read in the tick the ai wrote them
fn danger_zone_record(mut r: MessageReader<TrailBlazerMessage>, mut feed: ResMut<RenderFeed>) {
    for m in r.read() {
        match m.event {
            TrailEvent::RenderParams { on, prev, cur, success } => {
                feed.danger_on = on;
                feed.danger_prev = prev;
                feed.danger_cur = cur;
                feed.danger_success = success;
            }
            TrailEvent::DangerEnabled(e) => feed.danger_disabled = !e,
            _ => {}
        }
    }
}

/// the trailblazer's immediate render calls, in message order (read in the tick the ai wrote them, before
/// `trail_blazer_render_update`): TrailBlazerRender 0x001521E0 on `dword_7374CC` (0x000C4870 on, 0x000C4E50 off),
/// its +8 (0x000C6240), the danger particles `byte_719168` 0x001234A0 (0x000C4FF0) / 0x001238A0 (0x000C60E0) and
/// the intro's +29 = 0 (0x000C6080 → 0x000C63E0(0), sent with `DangerEnabled(true)`)
fn trail_render_calls(mut r: MessageReader<TrailBlazerMessage>, mut tr: MessageWriter<TrailBlazerRenderCall>, mut dp: MessageWriter<DangerParticleCall>) {
    for m in r.read() {
        match m.event {
            TrailEvent::Render { spline } => {
                tr.write(TrailBlazerRenderCall::Render { vehicle: m.vehicle as usize, spline });
            }
            TrailEvent::Intensity(v) => {
                tr.write(TrailBlazerRenderCall::Intensity(v));
            }
            TrailEvent::DangerZone(Some(l)) => {
                dp.write(DangerParticleCall::Level(l));
            }
            TrailEvent::DangerZone(None) => {
                dp.write(DangerParticleCall::Stop);
            }
            TrailEvent::DangerEnabled(true) => {
                dp.write(DangerParticleCall::Enable(false));
            }
            _ => {}
        }
    }
}

/// danger particles on (TrailBlazer 0x000C63E0(1)): the "TriggerBigParticles" TIMELINE trigger (TriggerUpdate
/// 0x00180310) and F3EventTrailBlazerFirewall (TimelineCameraShaker 0x0017A750, its `On` byte)
fn danger_particle_enables(
    mut trig: MessageReader<TriggerMessage>,
    mut tl: MessageReader<TimelineObjectMessage>,
    mut dp: MessageWriter<DangerParticleCall>,
) {
    for m in trig.read() {
        if let TriggerMessage::BigParticles = m {
            log::info!("TriggerBigParticles: danger particles on");
            dp.write(DangerParticleCall::Enable(true));
        }
    }
    for m in tl.read() {
        if let TimelineObjectMessage::TrailBlazerFirewall(on) = m {
            log::info!("TrailBlazerFirewall: danger particles {on}");
            dp.write(DangerParticleCall::Enable(*on));
        }
    }
}

/// F3ParticleGroupController (TimelineParticleGroup 0x0017C390 / unbind 0x0017C260): ParticleManager StartGroup
/// 0x000FF900 / StopGroup 0x000FFA70 on the race's placed particle groups
fn timeline_particle_systems(mut tl: MessageReader<TimelineObjectMessage>, mut group: MessageWriter<ParticleGroup>) {
    for m in tl.read() {
        if let TimelineObjectMessage::ParticleSystem { system, active } = *m {
            group.write(ParticleGroup { group: system, start: active });
        }
    }
}

/// the timeline audio objects' calls: TimelineSound 0x0017EA00 (F3EventSound / F3Event2DSound, always 2D: the
/// emitter's +0x10 instance is never set), 0x0017EBD0 / 0x0017EB80 (SoundStop), AUDIO_FadeToVolumePreset and
/// VsAudio::StartCityAmbience 0x0004AD60 / StopCityAmbience 0x0004AE10, in the tick the Timeline set wrote them
/// (also the cinematic bars: TimelineCameraShaker 0x0017A540 / 0x0017A580 -> OSDManager 0x000ED6E0, `On` ignored)
fn timeline_audio(
    mut tl: MessageReader<TimelineObjectMessage>,
    mut audio: MessageWriter<AudioCommand>,
    mut bars: MessageWriter<crate::game_mode::race_session::OsdCinematicBars>,
) {
    for m in tl.read() {
        match *m {
            TimelineObjectMessage::CinematicBars(on) => {
                bars.write(crate::game_mode::race_session::OsdCinematicBars(on));
            }
            TimelineObjectMessage::Sound { object, event, .. } => {
                audio.write(AudioCommand::TimelineSound { object, event });
            }
            TimelineObjectMessage::SoundStop { object, event } => {
                audio.write(AudioCommand::TimelineSoundStop { object, event });
            }
            TimelineObjectMessage::FadeToVolumePreset { preset, time } => {
                audio.write(AudioCommand::FadeToVolumePreset { preset, time });
            }
            TimelineObjectMessage::CityAmbience(on) => {
                audio.write(AudioCommand::CityAmbience(on));
            }
            _ => {}
        }
    }
}

/// TimelineCameraShaker.cpp 0x0017A490
fn timeline_screen_flash(mut tl: MessageReader<TimelineObjectMessage>, mut flash: MessageWriter<visos_render::flare_render::FlareFlash>) {
    for m in tl.read() {
        // FlareRender 0x001270E0 with {item +4 TimeIn, item +8 TimeOut, 0.5, full screen}
        if let TimelineObjectMessage::ScreenFlash { time_in, time_out, intensity } = *m {
            log::info!("screen flash: rise {time_in} fall {time_out}");
            flash.write(visos_render::flare_render::FlareFlash(visos_render::flare_render::FlashRecord {
                rise: time_in,
                fall: time_out,
                intensity,
                screen: true,
                ..Default::default()
            }));
        }
    }
}

/// the depth of field object's factor / focus for the timeline handlers (`DangerZoneRender +0x08 / +0x30`)
fn dof_world(dof: Option<Res<DepthOfField>>, mut world: ResMut<TimelineWorld>) {
    if let Some(d) = dof {
        world.dof = (d.dof_factor, d.focus);
    }
}

/// the timeline camera's calls on `dword_718BA8` (TimelineCamera 0x00178740 / 0x00179480 / 0x001783C0), in the
/// tick the Timeline set wrote them
fn dof_timeline(mut r: MessageReader<TimelineObjectMessage>, dof: Option<ResMut<DepthOfField>>) {
    let Some(mut dof) = dof else {
        r.clear();
        return;
    };
    for m in r.read() {
        match *m {
            TimelineObjectMessage::DepthOfFieldSetup { enabled, near, far } => {
                // 0x00179AE0 / 0x000BFF70 / 0x00179B00 / 0x00179A30 (set_focus between them reads none of these)
                dof.force = true;
                dof.enabled = enabled;
                dof.blend_from = near;
                dof.blend_to = far;
                dof.car_focus = false;
            }
            TimelineObjectMessage::DepthOfFieldFocus { focus } => dof.set_focus(focus, false, false),
            TimelineObjectMessage::DepthOfField { focus, factor } => dof.set(focus, factor),
            _ => {}
        }
    }
}

/// per rendered frame (GameManager 0x000B9010 for view 0, the race view: render flags 0x4000000 / 0x20000000
/// assumed set as for motion blur): the zone FX, danger zone and depth of field inputs. 24 Hz values are
/// interpolated with the timer lerp (speed, car position; the danger record is lerped by the renderer).
#[allow(clippy::too_many_arguments)]
fn fill_render_inputs(
    clock: Res<GameClock>,
    gm: Res<GameManager>,
    bridge: Res<GameBridge>,
    feed: Res<RenderFeed>,
    mut cams: ResMut<GameCameras>,
    mut mb: ResMut<MotionBlurInput>,
    zone: Option<Res<ZoneFx>>,
    zone_in: Option<ResMut<ZoneFxInput>>,
    zone_second: Option<ResMut<ZoneFxSecondInput>>,
    dof_in: Option<ResMut<DepthOfFieldInput>>,
    danger_in: Option<ResMut<DangerZoneInput>>,
    driver_in: Option<ResMut<CarDriverInput>>,
    mut commands: Commands,
    attract: Res<crate::font_helper::CAttractMode>,
) {
    let t = clock.timer.lerp.clamp(0.0, 1.0);
    let paused = gm.paused;
    // ZoneRender 0x00163800 -> CarPalette 0x0010D580 (`byte_7065A9`; the race renderer's front end flag +49 is 0)
    // and car 0's velocity for CarRender 0x00118DD0, lerped; the render path does not run while paused
    if !paused {
        let two_players = gm.race.as_ref().is_some_and(|r| r.num_players > 1);
        let cine = (!two_players && cams.manager.mode(0) == ECameraMode::Cine) || cams.replay.review_cameras == 2;
        let velocity = if feed.player.is_some() { feed.velocity[0].lerp(feed.velocity[1], t) } else { Vec3::ZERO };
        let v = CarDriverInput { cine, velocity };
        match driver_in {
            Some(mut d) => {
                if d.cine != v.cine || d.velocity != v.velocity {
                    *d = v;
                }
            }
            None => commands.insert_resource(v),
        }
    }
    if let Some(mut z) = zone_in {
        let speed = (feed.speed[1] - feed.speed[0]) * t + feed.speed[0];
        let v = ZoneFxInput {
            active: feed.player.is_some(),
            player: 0,
            paused,
            speed,
            zone_on: feed.zone_on,
            // FuryFX 0x00128680 is not ported (the fury render instances never activate): 0
            fury: 0.0,
            one_off: feed.one_off,
        };
        *z = v;
    }
    // GameManager 0x000B9010 calls 0x00162D10 per view with the view's player: player 1's view in two-player races
    {
        let two_players = gm.race.as_ref().is_some_and(|r| r.num_players > 1);
        let v = feed.second.filter(|_| two_players).map(|s| ZoneFxInput {
            active: true,
            player: 1,
            paused,
            speed: (s.speed[1] - s.speed[0]) * t + s.speed[0],
            zone_on: s.zone_on,
            fury: 0.0,
            one_off: s.one_off,
        });
        match zone_second {
            Some(mut z2) => z2.0 = v,
            None => commands.insert_resource(ZoneFxSecondInput(v)),
        }
    }
    if let Some(z) = zone.as_deref() {
        // 0x00154FC0 read by the motion blur 0x0012FB60 (the level before this frame's zone step)
        let a = z.amount(0);
        if mb.zone != a {
            mb.zone = a;
        }
        // 0x00163740: the in-zone flag on the camera manager (+44p+41)
        for (p, pc) in cams.manager.players.iter_mut().enumerate().take(2) {
            if pc.in_zone != z.in_zone[p] {
                pc.in_zone = z.in_zone[p];
            }
        }
    }
    if let Some(mut d) = dof_in {
        // 0x00120D20 / 0x00120E80: "replay" = the attract mode playing (`byte_6F5A28` +0, 0x000B36B0; F3_D records
        // no replays); `byte_718E70` is only read (bss 0)
        let car_position = feed.player.filter(|&id| id < bridge.cars.len()).map(|id| bridge.car_pose(id, bridge.render_lerp(t))).filter(|p| p.valid).map(|p| p.position);
        let v = DepthOfFieldInput {
            replay: attract.playing,
            replay_camera_type: cams.replay.camera as u32,
            other: cams.manager.mode(0) == ECameraMode::Cine || cams.replay.review_cameras == 2,
            car_position,
        };
        if d.replay != v.replay || d.replay_camera_type != v.replay_camera_type || d.other != v.other || d.car_position != v.car_position {
            *d = v;
        }
    }
    if let Some(mut d) = danger_in {
        let v = DangerZoneInput {
            enabled: !feed.danger_disabled,
            on: feed.danger_on,
            prev: feed.danger_prev,
            cur: feed.danger_cur,
            success: feed.danger_success,
            lerp: t,
            paused,
        };
        *d = v;
    }
}

/// VsAudio StartZone / StopZone of ZoneRender 0x00162D10
fn zone_audio(mut r: MessageReader<ZoneFxAudio>, mut audio: MessageWriter<AudioCommand>) {
    for m in r.read() {
        audio.write(if m.start { AudioCommand::StartZone } else { AudioCommand::StopZone });
    }
}

/// VsAudio StartNos / StopNos of FuryFX 0x001293E0
fn nos_audio(
    mut r: MessageReader<visos_render::fury_fx::FuryNosAudio>,
    mut audio: MessageWriter<AudioCommand>,
    mut last: Local<Option<bool>>,
) {
    for m in r.read() {
        if *last != Some(m.start) {
            log::info!("nos audio: {}", if m.start { "StartNos" } else { "StopNos" });
            *last = Some(m.start);
        }
        audio.write(if m.start { AudioCommand::StartNos } else { AudioCommand::StopNos });
    }
}

/// `EffectCars` for this update: the race manager's cars (race drivers in slot order) as the effect code reads
/// them off VehicleEntity / FastCar, plus the race globals
#[allow(clippy::too_many_arguments)]
fn fill_effect_cars(
    q: Query<(&RaceCar, &PhysicsCar, &PhysicsCarState)>,
    physics: Option<Res<PhysicsWorldRes>>,
    gm: Res<GameManager>,
    session: Res<RaceSession>,
    clock: Res<GameClock>,
    mut fx: ResMut<EffectCars>,
    mut started: ResMut<EffectsStarted>,
    mut init: MessageWriter<RaceEffectsInit>,
    mut spray_init: MessageWriter<visos_render::tyrespray::TyreSprayInit>,
) {
    let mut list: Vec<_> = q.iter().filter(|(c, _, _)| c.kind == CarKind::Race).collect();
    list.sort_by_key(|(c, _, _)| c.slot);
    let mut cars = Vec::with_capacity(list.len());
    if let Some(w) = physics.as_deref() {
        let g = w.lock();
        if let Some(world) = g.as_ref() {
            let ids = world.vehicle_ids();
            for (_, pc, st) in list {
                if !ids.contains(&pc.id) {
                    continue;
                }
                cars.push(effect_car(world, pc.id, st));
            }
        }
    }
    fx.cars = cars;
    fx.dt = clock.timer.scaled_dt();
    fx.time = clock.timer.game_time();
    fx.start_line = gm.event_def.as_ref().map(|d| (Vec3::from(d.start_line[0]), Vec3::from(d.start_line[1])));
    // byte_6F70E9 / byte_6F5BDD (set by the flag girl's ALLOWGO, TimelineSequencer 0x00175270)
    fx.professional = gm.race.as_ref().is_some_and(|r| r.professional);
    fx.go = session.started;
    if !started.0 && !fx.cars.is_empty() {
        started.0 = true;
        init.write(RaceEffectsInit);
        // "Tyre Spray Render Init" (0x001523D0): VsAudio::IsTrackWet and the track (+0x78, VsAIPoliceVehicle
        // 0x00017CB0); its shutdown is `RaceEffectsShutdown` (InGameShutdown above)
        if let Some(r) = gm.race.as_ref() {
            spray_init.write(visos_render::tyrespray::TyreSprayInit { wet: r.wet, track: r.track_path.clone() });
        }
    }
}

/// one car: FastCar +32 velocity, VE+20 speed, VE+4788 heading (0x001BB200(0)), VE+3691 smoke off, wheels
/// (0x00361400 centre, +1804 radius, +1088 contacts, 0x00364A90 sliding, VE+4868 surface, VE+24 no marks) and
/// the ground plane (0x001BAF60)
fn effect_car(world: &velocity_physics::vehicle::vehicle_entity::sim::PhysicsWorld, id: usize, st: &PhysicsCarState) -> EffectCar {
    let s = world.slot(id);
    let (car, ve) = (&s.car.m, &s.ve.m);
    // 0x00361400 / 0x00361390: wheel centre (+1968 + 16i when +2240 is set, else the wheel matrix origin)
    let centre = |i: usize| Vec3::from(if car.i(2240) != 0 { car.v3(1968 + 16 * i) } else { car.v3(1596 + 64 * i) });
    let wheels = std::array::from_fn(|i| {
        let contacts = car.i(1088 + 4 * i);
        // 0x00364A90 == 1: any record set (the original indexes record wheel + k)
        let sliding = (0..contacts.max(0) as usize).any(|k| car.i(676 + 136 * (i + k)) != 0);
        EffectWheel {
            centre: centre(i),
            radius: car.f(1804 + 4 * i),
            contacts,
            sliding,
            surface: ve.b(4868 + i),
            no_marks: ve.b(24 + i) != 0,
        }
    });
    // 0x001BAF60: three or more wheels touching -> plane through wheel centres 0, 1, 2 (0x001DA190)
    let touching = (0..4).filter(|&i| car.i(1088 + 4 * i) > 0).count();
    let ground_plane = (touching >= 3).then(|| {
        let (p0, p1, p2) = (centre(0), centre(1), centre(2));
        let n = (p2 - p1).cross(p0 - p1).normalize_or_zero();
        n.extend(-n.dot(p0))
    });
    EffectCar {
        vehicle: id,
        matrix: Mat4::from_cols_array(&st.body_matrix),
        velocity: Vec3::from(world.velocity(id)),
        speed: ve.f(20),
        heading: Vec3::new(ve.f(4788), ve.f(4792), ve.f(4796)),
        smoke_off: ve.b(3691) != 0,
        wheels,
        ground_plane,
    }
}

/// effect time stops while paused (`GameBridge__97E90`) and follows the time stretch
fn effect_clock(gm: Res<GameManager>, clock: Res<GameClock>, mut ec: ResMut<EffectClock>) {
    let (paused, scale) = (gm.paused, clock.timer.time_stretch);
    if ec.paused != paused || ec.scale != scale {
        ec.paused = paused;
        ec.scale = scale;
    }
}

/// TimelineSequencer 0x00174A70 "startlinefireeffect": StartLineEffects 0x001472F0 at the emitter
fn start_line_fire(mut r: MessageReader<RaceSessionMessage>, mut fire: MessageWriter<StartLineFire>) {
    for m in r.read() {
        if let RaceSessionMessage::FlagGirl(FlagGirlEvent::StartLineFire(p)) = m {
            fire.write(StartLineFire(*p));
        }
    }
}

fn not_paused(gm: Res<GameManager>) -> bool {
    !gm.paused
}

pub(crate) fn build(app: &mut App) {
    // registered in every app: the ai writes it
    app.add_message::<FadeStart>();
    // only with the render and physics plugins in the app (game flow tests run without them)
    if !app.world().contains_resource::<RaceEffects>() || !app.world().contains_resource::<Messages<PhysicsObjectMessage>>() {
        return;
    }
    app.add_systems(PostUpdate, render_load_steps)
        .add_systems(
            GameTick,
            (lens_spray_update, fade_update)
                .chain()
                .in_set(InGameUpdateSet::Effects)
                .after(fill_motion_blur)
                .run_if(in_state(EGameMode::InGame))
                .run_if(not_paused),
        );
    app.init_resource::<EffectsStarted>()
        .init_resource::<MotionBlurInput>()
        .init_resource::<visos_render::zone_render_pc_xbox::MotionBlurSecondInput>()
        .init_resource::<RenderFeed>()
        .add_systems(OnExit(crate::InRace), clear_motion_blur)
        .add_systems(PostUpdate, particle_load_steps.run_if(in_state(EGameMode::InGame)))
        // effect_clock / mode_messages feed race_effects_frame / update_particles (EffectClock, RaceEffectsReset/Shutdown)
        .add_systems(Update, (mode_messages, effect_clock).before(visos_render::race_effects::RaceEffectsMessages))
        .add_systems(Update, start_line_fire)
        .add_systems(Update, fill_render_inputs.run_if(in_state(EGameMode::InGame)))
        .add_systems(Update, zone_audio.run_if(resource_exists::<Messages<ZoneFxAudio>>))
        .add_systems(Update, nos_audio.run_if(resource_exists::<Messages<visos_render::fury_fx::FuryNosAudio>>))
        .add_systems(
            GameTick,
            (danger_zone_record, fill_render_feed, dof_world)
                .chain()
                .in_set(InGameUpdateSet::Effects)
                .run_if(in_state(EGameMode::InGame))
                .run_if(not_paused),
        )
        .add_systems(GameTick, (dof_timeline, timeline_particle_systems, timeline_audio).after(InGameUpdateSet::Timeline).run_if(in_state(EGameMode::InGame)))
        .add_systems(
            GameTick,
            (forward_physics_objects, forward_collisions).in_set(InGameUpdateSet::World).run_if(in_state(EGameMode::InGame)).run_if(not_paused),
        )
        .add_systems(
            GameTick,
            (fill_effect_cars, race_effects_update, fill_motion_blur)
                .chain()
                .in_set(InGameUpdateSet::Effects)
                .run_if(in_state(EGameMode::InGame))
                .run_if(not_paused),
        );
    // puddles (graphics' PuddleRender); `forward_puddles` runs in race_cars' physics chain
    if !app.world().contains_resource::<Messages<visos_render::puddle_render::PuddleAdd>>() {
        app.add_message::<visos_render::puddle_render::PuddleAdd>()
            .add_message::<visos_render::puddle_render::PuddleRestart>();
    }
    app.add_systems(PostUpdate, puddle_restart);
    // camera rain (graphics' CameraRainPlugin)
    if app.world().contains_resource::<Messages<CameraRainInit>>() {
        app.add_systems(PostUpdate, camera_rain_steps.before(visos_render::effect_draw::EffectSet::Build))
            .add_systems(Update, camera_rain_input.run_if(in_state(EGameMode::InGame)))
            .add_systems(OnExit(crate::InRace), clear_camera_rain_input);
    }
    // timeline screen flash (graphics' FlareRender)
    if app.world().contains_resource::<Messages<visos_render::flare_render::FlareFlash>>() {
        app.add_systems(GameTick, timeline_screen_flash.after(InGameUpdateSet::Timeline).run_if(in_state(EGameMode::InGame)));
    }
    // dynamic envmap (graphics' car shader manager)
    if app.world().contains_resource::<Messages<visos_render::car_shader_manager_pc_xbox::DynamicEnvmapInit>>() {
        app.add_systems(PostUpdate, dynamic_envmap_steps);
    }
    // zone trails + danger particles (graphics' TrailBlazerRenderPlugin / DangerZonePlugin)
    if app.world().contains_resource::<TrailBlazerRender>() {
        // "CZoneTrailRender.Update" (InGameUpdate 0x000B82B3, after the effects; 0x0014E530 skips itself while paused)
        app.add_systems(
            GameTick,
            (trail_render_calls, trail_blazer_render_update)
                .chain()
                .in_set(InGameUpdateSet::Effects)
                .after(race_effects_update)
                .run_if(in_state(EGameMode::InGame)),
        )
        .add_systems(GameTick, danger_particle_enables.after(InGameUpdateSet::Timeline).run_if(in_state(EGameMode::InGame)));
    }
}
