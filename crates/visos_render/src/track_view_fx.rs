//! debug (not an original module): `trackview --effects skid,sparks,slip,start --fxat x,y,z` — two cars circling
//! a point on the road feed `EffectCars` the way core does in a race: game updates at 24 Hz of the real time,
//! car roots placed every rendered frame between the last two updates by the timer lerp (core's `place_cars`).

use bevy::prelude::*;

use crate::car_render::{CarRender, CarSpec, SpawnCarRender};
use crate::race_effects::{EffectCar, EffectCars, EffectWheel, RaceEffectsInit, SlipstreamStrength, SparkCollisionMessage};
use crate::spark_render::SparkCollision;
use crate::track_render::TrackRoot;
use crate::track_view::TrackViewConfig;

#[derive(Default)]
pub struct Demo {
    t: f32,
    spawned: bool,
    init: bool,
    next_spark: f32,
    /// real time not yet run as game updates
    acc: f32,
    /// the last two updates' car matrices
    prev: Vec<Mat4>,
}

const RADIUS: f32 = 6.0;
const SPEED: f32 = 12.0;
/// the game update (GameClock, 24 Hz)
const STEP: f32 = 1.0 / 24.0;
/// vehicle ids = render requests of the demo cars (below MAX_VEHICLES, clear of `--car`'s 1..)
pub const FIRST_VEHICLE: usize = 20;

fn demo_cars(c: Vec3, t: f32, on: &dyn Fn(&str) -> bool) -> Vec<EffectCar> {
    let w = SPEED / RADIUS;
    (0..2)
        .map(|i| {
            let a = t * w + i as f32 * 1.2;
            let pos = c + Vec3::new(a.cos(), 0.0, a.sin()) * RADIUS;
            let fwd = Vec3::new(-a.sin(), 0.0, a.cos());
            let side = fwd.cross(Vec3::Y);
            let m = Mat4::from_cols(fwd.extend(0.0), Vec3::Y.extend(0.0), side.extend(0.0), pos.extend(1.0));
            // FastCar order: 0 front left, 1 rear left, 2 rear right, 3 front right
            let local = [Vec3::new(1.3, 0.33, -0.75), Vec3::new(-1.3, 0.33, -0.75), Vec3::new(-1.3, 0.33, 0.75), Vec3::new(1.3, 0.33, 0.75)];
            let wheels = local.map(|l| EffectWheel {
                centre: m.transform_point3(l),
                radius: 0.33,
                contacts: 1,
                sliding: on("skid"),
                // `mist`: every wheel on wet surface 1 (VE +4868), the no-race-section branch of 0x001326A0
                surface: on("mist") as u8,
                no_marks: false,
            });
            EffectCar {
                vehicle: FIRST_VEHICLE + i,
                matrix: m,
                velocity: fwd * SPEED,
                speed: SPEED,
                heading: fwd,
                smoke_off: !on("smoke"),
                wheels,
                ground_plane: Some(Vec4::new(0.0, 1.0, 0.0, -c.y)),
            }
        })
        .collect()
}

/// `--effects blur[=ω]`: every car's wheel 0 spinning at ω rad/s (default the demo speed over a 0.33 m wheel)
pub fn wheel_spin_demo(cfg: Res<TrackViewConfig>, mut commands: Commands, q: Query<Entity, (With<CarRender>, Without<crate::car_render::CarWheelSpin>)>) {
    let which = cfg.effects.clone().unwrap_or_default();
    let Some(k) = which.split(',').find(|x| x.starts_with("blur")) else { return };
    let omega = k.strip_prefix("blur=").and_then(|v| v.parse().ok()).unwrap_or(SPEED / 0.33);
    for e in &q {
        commands.entity(e).insert(crate::car_render::CarWheelSpin { omega });
    }
}

/// `--effects rain[=level]`: camera rain on a wet track (InGameInit 0x00133AB0), the level held at `level` when given
/// (it starts at 0.25 and rises 0.01/s); car 0 = the first demo car
pub fn rain_demo(
    cfg: Res<TrackViewConfig>,
    roots: Query<&TrackRoot>,
    mut done: Local<bool>,
    mut init: MessageWriter<crate::camera_rain::CameraRainInit>,
    mut rain: ResMut<crate::camera_rain::CameraRain>,
    mut input: ResMut<crate::camera_rain::CameraRainInput>,
    cars: Res<EffectCars>,
) {
    let which = cfg.effects.clone().unwrap_or_default();
    let Some(k) = which.split(',').find(|x| x.starts_with("rain")) else { return };
    if roots.is_empty() {
        return;
    }
    if !*done {
        *done = true;
        init.write(crate::camera_rain::CameraRainInit { wet: true });
    }
    if let Some(l) = k.strip_prefix("rain=").and_then(|v| v.parse::<f32>().ok()) {
        rain.level = l;
    }
    input.car0 = cars.cars.first().map(|c| c.matrix.w_axis.truncate());
}

#[allow(clippy::too_many_arguments)]
pub fn effects_demo(
    cfg: Res<TrackViewConfig>,
    time: Res<Time>,
    mut demo: Local<Demo>,
    roots: Query<&TrackRoot>,
    mut cars: ResMut<EffectCars>,
    mut spawn: MessageWriter<SpawnCarRender>,
    mut q: Query<(&CarRender, &mut Transform), Without<crate::track_render::TrackCamera>>,
    mut cam: Query<&mut Transform, With<crate::track_render::TrackCamera>>,
    mut init: MessageWriter<RaceEffectsInit>,
    mut hits: MessageWriter<SparkCollisionMessage>,
    mut slip: MessageWriter<SlipstreamStrength>,
    mut fx: ResMut<crate::race_effects::RaceEffects>,
    mut particles: ResMut<crate::stateless_particles::Particles>,
    mut spray: MessageWriter<crate::tyrespray::TyreSprayInit>,
    mut puddles: Option<ResMut<crate::puddle_render::PuddleRender>>,
    mut lens: Option<ResMut<crate::lens_spray_manager::LensSpray>>,
) {
    if roots.is_empty() {
        return;
    }
    let which = cfg.effects.clone().unwrap_or_default();
    let on = |k: &str| which.split(',').any(|x| x == k || x == "all");
    let c = cfg.fx_at.unwrap_or(Vec3::ZERO);
    if !demo.spawned {
        demo.spawned = true;
        for (i, name) in ["LA/350z", "LA/RX8"].iter().enumerate() {
            let request = (FIRST_VEHICLE + i) as u64;
            spawn.write(SpawnCarRender { request, spec: CarSpec::new(*name, Vec3::new(0.7, 0.1, 0.1)), transform: Transform::default() });
        }
        cars.cars = demo_cars(c, 0.0, &on);
        demo.prev = cars.cars.iter().map(|c| c.matrix).collect();
    }
    demo.acc = (demo.acc + time.delta_secs()).min(STEP * 2.0);
    while demo.acc >= STEP {
        demo.acc -= STEP;
        demo.prev = cars.cars.iter().map(|c| c.matrix).collect();
        demo.t += STEP;
        let t = demo.t;
        cars.cars = demo_cars(c, t, &on);
        cars.dt = STEP;
        cars.time = t;
        cars.start_line = on("start").then(|| (c + Vec3::new(RADIUS - 3.0, 0.0, 0.0), c + Vec3::new(RADIUS + 3.0, 0.0, 0.0)));
        cars.professional = on("start");
        cars.go = t > 1.8;
        if !demo.init {
            demo.init = true;
            init.write(RaceEffectsInit);
            if on("spray") {
                // a wet track: every wheel sprays with the car's speed
                spray.write(crate::tyrespray::TyreSprayInit { wet: true, track: cfg.track.clone() });
            }
        }
        if on("slip") {
            slip.write(SlipstreamStrength { vehicle: FIRST_VEHICLE + 1, strength: 0.6 });
        }
        if on("sparks") && t >= demo.next_spark {
            demo.next_spark = t + 0.25;
            let car = &cars.cars[0];
            let side = car.matrix.z_axis.truncate();
            hits.write(SparkCollisionMessage(SparkCollision {
                car: 0,
                other: -1,
                point: car.matrix.w_axis.truncate() + side * 0.9 + Vec3::Y * 0.3,
                normal: side,
                impulse: 40.0,
                scale: 1.0,
                material: -1,
            }));
        }
        // InGameUpdateSet::Effects
        crate::race_effects::game_update(&mut fx, &mut particles, &cars);
    }
    if on("mist") {
        if let (Some(p), Some(l)) = (puddles.as_mut(), lens.as_mut()) {
            if p.generation == 0 {
                p.init(false, None, &mut visos::handy::crt_rand::CrtRand::default());
            }
            // the render path's puddle test (core's puddle_render_update), per rendered frame
            p.update(&crate::puddle_render::PuddleInput::default(), &cars, l);
        }
    }
    // render: position lerp / rotation slerp between the last two updates by the timer lerp
    let lerp = demo.acc / STEP;
    let pose = |k: usize| -> Option<(Vec3, Quat)> {
        let (a, b) = (demo.prev.get(k)?, cars.cars.get(k)?.matrix);
        let (qa, qb) = (Quat::from_mat4(a), Quat::from_mat4(&b));
        Some((a.w_axis.truncate().lerp(b.w_axis.truncate(), lerp), qa.slerp(qb, lerp)))
    };
    for (r, mut tr) in &mut q {
        if let Some((p, rot)) = (r.request as usize).checked_sub(FIRST_VEHICLE).and_then(pose) {
            tr.translation = p;
            // the matrix is a FastCar body (nose +X); models are nose +Z (core's MODEL_TO_BODY)
            tr.rotation = rot * Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        }
    }
    if let Some(k) = cfg.fx_follow {
        if let (Some((p, _)), Ok(mut t)) = (pose(k), cam.single_mut()) {
            *t = Transform::from_translation(p + Vec3::new(3.0, 2.0, 3.0)).looking_at(p + Vec3::Y * 0.5, Vec3::Y);
        }
    }
}
