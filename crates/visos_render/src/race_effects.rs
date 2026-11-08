//! glue (not an original module): the race effects of GameManager's update and render passes — skid marks and
//! tyre smoke per game update, sparks, slipstream trails and start lights per rendered frame — fed by the
//! per-car state physics / core publish in `EffectCars`.
//!
//! API (core): fill `EffectCars` every game update before running `race_effects_update` (add it to
//! `InGameUpdateSet::Effects`); write `RaceEffectsInit` once the race and its cars exist (InGameInit),
//! `RaceEffectsReset` on RestartRace, `SparkCollision` per collision message, `SlipstreamStrength` where physics
//! calls `SlipstreamRender::SetStrength`, `StartLineFire` with the flag girl's "startlinefireeffect" position.

use bevy::prelude::*;

use crate::car_render::CarRender;
use crate::effect_draw::{EffectClock, EffectDraw};
use crate::skid_mark_render::SkidMarkRender;
use crate::skidsmoke::SkidSmoke;
use crate::slipstream_render::SlipstreamRender;
use crate::spark_render::{SparkCollision, SparkRender};
use crate::start_line_effects::StartLineEffects;
use crate::stateless_particles::Particles;

/// one wheel as the effects read it from FastCar / VehicleEntity
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EffectWheel {
    /// wheel centre (FastCar 0x00361400: +1968 + 16i when +2240 is set, else the wheel matrix origin)
    pub centre: Vec3,
    /// radius (FastCar +1804 + 4i, 0x0035FB40)
    pub radius: f32,
    /// contacts (FastCar +1088 + 4i)
    pub contacts: i32,
    /// any contact sliding (FastCar 0x00364A90 == 1)
    pub sliding: bool,
    /// `eSURF_*` under the wheel (VehicleEntity +4868 + i)
    pub surface: u8,
    /// VehicleEntity +24 + i: no marks or smoke from this wheel
    pub no_marks: bool,
}

/// one car, in the race manager's order (race drivers, then TFT players)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EffectCar {
    /// physics vehicle index (VehicleEntity +0x13E0, the slipstream key)
    pub vehicle: usize,
    /// render matrix (the car bridge pose, `VsAIRaceVehicle 0x00023F40`), game space
    pub matrix: Mat4,
    /// FastCar +32 velocity
    pub velocity: Vec3,
    /// VehicleEntity +20 speed (m/s)
    pub speed: f32,
    /// VehicleEntity +4788 (0x001BB200(0)): smoke direction when nearly stopped
    pub heading: Vec3,
    /// VehicleEntity +3691: no tyre smoke
    pub smoke_off: bool,
    /// FastCar wheel order: 0 / 3 front, 1 / 2 back
    pub wheels: [EffectWheel; 4],
    /// VehicleEntity 0x001BAF60: plane through three wheel contacts (normal up, d), when three or more touch
    pub ground_plane: Option<Vec4>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct EffectCars {
    pub cars: Vec<EffectCar>,
    /// this update's scaled length (GameManager 0x000BF670) and the game time (VsFrameTimer, 0x00024390)
    pub dt: f32,
    pub time: f32,
    /// start line ends (`dword_6F5C24` / `dword_6F5C34`)
    pub start_line: Option<(Vec3, Vec3)>,
    /// professional race (`byte_6F70E9`) and the go signal (`byte_6F5BDD`): start lights
    pub professional: bool,
    pub go: bool,
}

#[derive(Resource, Default)]
pub struct RaceEffects {
    pub skid: SkidMarkRender,
    pub smoke: SkidSmoke,
    pub sparks: SparkRender,
    pub slipstream: SlipstreamRender,
    pub start_line: StartLineEffects,
    /// set by `RaceEffectsInit`, cleared by `RaceEffectsShutdown`
    pub active: bool,
}

#[derive(Message, Clone, Debug)]
pub struct RaceEffectsInit;

#[derive(Message, Clone, Debug)]
pub struct RaceEffectsReset;

#[derive(Message, Clone, Debug)]
pub struct RaceEffectsShutdown;

#[derive(Message, Clone, Copy, Debug)]
pub struct SparkCollisionMessage(pub SparkCollision);

#[derive(Message, Clone, Copy, Debug)]
pub struct SlipstreamStrength {
    pub vehicle: usize,
    pub strength: f32,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct StartLineFire(pub Vec3);

/// `race_effect_messages` (collision debris + sparks spawn, Update): `Particles` writers order against it
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RaceEffectsMessages;

pub struct RaceEffectsPlugin;

impl Plugin for RaceEffectsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::stateless_particles::ParticlePlugin>() {
            app.add_plugins(crate::stateless_particles::ParticlePlugin);
        }
        app.init_resource::<EffectCars>()
            .init_resource::<RaceEffects>()
            .add_message::<RaceEffectsInit>()
            .add_message::<RaceEffectsReset>()
            .add_message::<RaceEffectsShutdown>()
            .add_message::<SparkCollisionMessage>()
            .add_message::<SlipstreamStrength>()
            .add_message::<StartLineFire>()
            .add_systems(
                Update,
                (
                    race_effect_messages.in_set(RaceEffectsMessages).before(crate::stateless_particles::particle_messages),
                    race_effects_frame,
                )
                    .chain(),
            )
            .add_systems(PostUpdate, draw_race_effects.in_set(crate::effect_draw::EffectOrder::RaceFx));
    }
}

fn player_pos(cars: &EffectCars) -> Vec3 {
    cars.cars.first().map(|c| c.matrix.w_axis.truncate()).unwrap_or(Vec3::ZERO)
}

#[allow(clippy::too_many_arguments)]
fn race_effect_messages(
    mut fx: ResMut<RaceEffects>,
    mut particles: ResMut<Particles>,
    cars: Res<EffectCars>,
    mut init: MessageReader<RaceEffectsInit>,
    mut reset: MessageReader<RaceEffectsReset>,
    mut shutdown: MessageReader<RaceEffectsShutdown>,
    mut hits: MessageReader<SparkCollisionMessage>,
    mut slip: MessageReader<SlipstreamStrength>,
    mut fire: MessageReader<StartLineFire>,
) {
    let fx = &mut *fx;
    for _ in init.read() {
        // GameManager::InGameInit: SkidMarkRender (0x001382F0), sparks (0x00141730), slipstream (0x0013F380),
        // skid smoke, start line effects (0x00146CC0)
        fx.skid = SkidMarkRender::default();
        fx.smoke = SkidSmoke::default();
        fx.sparks = SparkRender::default();
        let globals = fx.slipstream.globals;
        fx.slipstream = SlipstreamRender::default();
        fx.slipstream.globals = globals;
        fx.start_line = StartLineEffects::default();
        if let (Some(line), Some(c)) = (cars.start_line, cars.cars.first()) {
            fx.start_line.setup(line, c.matrix.w_axis.truncate(), c.matrix.x_axis.truncate());
        }
        fx.active = true;
    }
    for _ in reset.read() {
        // GameManager::RestartRace: start line effects, sparks (0x001418D0), start line records (0x001398A0)
        fx.start_line.reset();
        fx.sparks = SparkRender::default();
        fx.skid.reset_start_line();
    }
    for _ in shutdown.read() {
        *fx = RaceEffects::default();
    }
    for h in hits.read() {
        let c = &h.0;
        let (vel, ground) = cars.cars.get(c.car).map(|x| (x.velocity, x.ground_plane)).unwrap_or((Vec3::ZERO, None));
        // PhysicsManager 0x001A3A30: surface debris (ParticleManager 0x001005C0) before the message is posted
        let p = &mut *particles;
        p.placed.collision_particles(&mut p.stateless, c.material as u8, c.point, c.impulse);
        fx.sparks.on_collision(c, vel, ground, player_pos(&cars));
    }
    for s in slip.read() {
        fx.slipstream.set_strength(s.vehicle, s.strength);
    }
    for f in fire.read() {
        fx.start_line.fire_moved(f.0, &mut particles.stateless);
    }
}

/// GameManager::InGameUpdate: `g_SkidMarkRender.Update` (0x00138500), `g_SkidSmokeManager.Update`
/// (0x00139BD0), `g_StartLineEffects.Update` (0x00147280). Core runs it in `InGameUpdateSet::Effects` after
/// filling `EffectCars`.
pub fn race_effects_update(mut fx: ResMut<RaceEffects>, mut particles: ResMut<Particles>, cars: Res<EffectCars>) {
    game_update(&mut fx, &mut particles, &cars);
}

/// the body of `race_effects_update`
pub fn game_update(fx: &mut RaceEffects, particles: &mut Particles, cars: &EffectCars) {
    if !fx.active {
        return;
    }
    fx.skid.update(&cars);
    let flames: Vec<[f32; 4]> = fx.skid.start_line.iter().map(|r| [r[0].timer, r[1].timer, r[2].timer, r[3].timer]).collect();
    fx.smoke.update(&cars, &flames, &mut particles.stateless);
    fx.start_line.update(cars.professional, cars.go);
}

/// per rendered frame while not paused: sparks (0x00142120 -> 0x00141F80, frame ms * 0.001)
fn race_effects_frame(time: Res<Time>, clock: Res<EffectClock>, mut fx: ResMut<RaceEffects>, cars: Res<EffectCars>) {
    if !fx.active || clock.paused {
        return;
    }
    let dt = time.delta_secs() * clock.scale;
    let ground = cars.cars.first().and_then(|c| c.ground_plane);
    fx.sparks.update(dt, ground);
}

/// `VsAIRaceVehicle__23F40` = car data +0 of a vehicle: the render matrix the car bridge 0x00096E20 writes each
/// rendered frame from the last two updates by `timer.lerp` (model space, nose +Z). That is the root `Transform` of
/// its `CarRender` (request = physics vehicle id, core's `place_cars`); without a root, the 24 Hz `EffectCars` pose.
pub fn render_car_matrix(roots: &Query<(&CarRender, &Transform)>, cars: &EffectCars, v: usize) -> Option<Mat4> {
    roots
        .iter()
        .find(|(r, _)| r.request == v as u64)
        .map(|(_, t)| t.to_matrix())
        .or_else(|| cars.cars.iter().find(|c| c.vehicle == v).map(|c| crate::slipstream_render::model_matrix(c.matrix)))
}

/// GameManager 0x000B9010 draw calls: skid marks (0x00139410 x2), sparks (0x00142120), slipstream
/// (0x0013FDB0 -> update 0x0013F770, then the ribbons), start lights (0x001475B0)
#[allow(clippy::too_many_arguments)]
fn draw_race_effects(
    mut fx: ResMut<RaceEffects>,
    cars: Res<EffectCars>,
    time: Res<Time>,
    clock: Res<EffectClock>,
    roots: Query<(&CarRender, &Transform)>,
    cam: Query<&GlobalTransform, With<crate::track_render::TrackCamera>>,
    views: crate::view_render::ViewCams,
    mut draw: ResMut<EffectDraw>,
) {
    if !fx.active {
        return;
    }
    let car_matrix = |v: usize| render_car_matrix(&roots, &cars, v);
    if !clock.paused {
        // 0x0013F770 steps once per rendered TV frame: run as 60 Hz frames of the real delta, sampling the
        // lerped render matrices (so the trails do not step at the 24 Hz game rate)
        fx.slipstream.update_dt(time.delta_secs(), car_matrix);
    }
    let Some(cam) = cam.iter().next() else { return };
    let mirror = views.mirror();
    let fx = &mut *fx;
    fx.skid.draw(false, &mut draw);
    fx.skid.draw(true, &mut draw);
    // origin = player 0's car, set before drawing (0x00142120)
    fx.sparks.origin = player_pos(&cars);
    fx.sparks.draw(cam.translation(), &mut draw);
    // 0x0013FDB0 runs ungated in every view (world space: one build for all of them); the mirror view also belongs to
    // player 0 (`*(view + 28) == 0`), so it steps 0x0013F770 a second time and draws its own ribbons (original)
    draw.set_view(Some(if mirror.is_some() { 0 } else { crate::effect_draw::ALL_VIEWS }));
    fx.slipstream.draw(car_matrix, &mut draw);
    draw.set_view(None);
    fx.start_line.draw(cam.translation(), cam.right().as_vec3(), cam.up().as_vec3(), &mut draw);
    // the mirror view (flags 0x08000620): 0x0013FDB0 and StartLineEffects 0x001475B0 run ungated with its camera
    if let Some(m) = mirror {
        if !clock.paused {
            fx.slipstream.update_dt(time.delta_secs(), car_matrix);
        }
        draw.set_view(m.tag);
        fx.slipstream.draw(car_matrix, &mut draw);
        let t = m.transform;
        fx.start_line.draw(t.translation(), t.right().as_vec3(), t.up().as_vec3(), &mut draw);
        draw.set_view(None);
    }
}
