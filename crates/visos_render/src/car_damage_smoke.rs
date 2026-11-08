//! CarDamage.cpp 0x000766B0..0x00076E80 — the bonnet smoke half of the CarDamageFX object `byte_6D6808`: two
//! elParticle systems "bonnetSmokeRight" / "bonnetSmokeLeft" either side of the player car's bonnet, re-tuned every
//! rendered frame (0x00076E80, from 0x000768D0 = GameManager 0x000B9037) by a speed level.
//!
//! **Never visible in the original**: nothing starts the systems (no caller of `elParticleSystem::start` 0x002CF100
//! in the CarDamage range; the "running" byte +156 that 0x00076870 checks before stopping them is only ever zeroed),
//! so they never emit. Kept that way; `VELOCITY_BONNET_SMOKE_FORCE=1` starts them (and counts four broken
//! headlights) to look at the tuning.
//!
//! Lifecycle here: `RaceEffectsInit` = 0x000766B0 (create, from CarDamage init 0x0006B8B0), `RaceEffectsReset` =
//! 0x000766F0 (RestartRace), `RaceEffectsShutdown` = 0x00076780.

use bevy::prelude::*;
use velocity_physics::car_damage::CarDamageManager;

use crate::car_render::CarRender;
use crate::effect_draw::{EffectShader, EffectTexture};
use crate::el_particle::el_particle_control_curve::{ColourCurve, ControlCurve};
use crate::el_particle::el_particle_system::{wxyz, ElParticleType, TrailParams};
use crate::el_particle::vs_pool_allocator::EmitterShape;
use crate::el_particle::{ElParticleSet, ElParticleSystemManager, SystemId, TypeId};
use crate::race_effects::{render_car_matrix, EffectCars, RaceEffectsInit, RaceEffectsReset, RaceEffectsShutdown};

/// `dword_6852E4`
const MAX_PARTICLES: i32 = 150;
/// `dword_6852E8` / the 0.5 jitter of 0x000769E0
const CREATE_RATE: f32 = 20.0;
const CREATE_JITTER: f32 = 0.5;
/// `dword_6852E0`: headlight count clamp and divisor
const COUNT_DIV: u32 = 4;
/// `flt_6852EC` (along model x), `dword_6852F0` (up), `dword_6D67AC` (bss: world z)
const SIDE: f32 = -0.5;
const LIFT: f32 = 0.5;
const Z_OFFSET: f32 = 0.0;
/// 0x00078040: min(`dword_6852DC` · mph/240 + `dword_6D67A8` (bss), 1) from 0.2 mph
const LEVEL_SCALE: f32 = 2.0;
const LEVEL_BASE: f32 = 0.0;
const MPH: f32 = 2.25;
const MPH_MIN: f32 = 0.2;
const MPH_RANGE: f32 = 240.0;
/// `dword_685318` / `dword_68531C` (u32): rate, `dword_685320` jitter
const RATE: [u32; 2] = [30, 50];
const RATE_JITTER: f32 = 0.2;
/// `flt_6852F4` / `flt_6852F8`: lifetime
const LIFE: [f32; 2] = [0.7, 1.2];
/// size keys: v0 `flt_6D67B0`/`B4` (bss), v1 `flt_6852FC`/`flt_685304`, v2 `flt_685300`/`flt_685308`, t1
/// `flt_68530C`/`flt_685310`; randomness `dword_685314`
const SIZE_V0: [f32; 2] = [0.0, 0.0];
const SIZE_V1: [f32; 2] = [1.0, 1.0];
const SIZE_V2: [f32; 2] = [1.0, 1.5];
const SIZE_T1: [f32; 2] = [0.7, 0.7];
const SIZE_RANDOM: f32 = 0.2;
/// rotation params (`dword_6D67B8` bss, `dword_685324`, `dword_685328`, `dword_68532C`)
const ROTATION: [f32; 4] = [0.0, 360.0, -45.0, 45.0];
/// static init 0x00421CE0..0x00421E51 (VsVector4 x, y, z): velocity `dword_6D6998` → `dword_6D67E0`, spread
/// `unk_6D6C78` → `unk_6D6988`, gravity `dword_6D67C0` → `dword_6D69A8`, colours `unk_6D67D0` / `unk_6D67F4`,
/// emitter box `unk_6D69F8`
const VEL: [Vec3; 2] = [Vec3::new(0.4, 0.0, -0.2), Vec3::new(0.4, 0.0, 0.0)];
const SPREAD: [Vec3; 2] = [Vec3::new(0.1, 0.1, 0.1), Vec3::new(0.2, 0.2, 0.4)];
const GRAVITY: [Vec3; 2] = [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -6.0)];
const COL0: Vec4 = Vec4::new(0.7, 0.7, 0.7, 0.5);
const COL1: Vec4 = Vec4::new(0.3, 0.3, 0.3, 0.0);
const BOX: Vec3 = Vec3::new(0.1, 0.0, 0.4);

/// 0x00077F30
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (b - a) * t + a
}

/// 0x00077F50
fn lerp3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    Vec3::new((b.x - a.x) * t + a.x, (b.y - a.y) * t + a.y, (b.z - a.z) * t + a.z)
}

#[derive(Clone, Copy, Debug)]
struct Side {
    ty: TypeId,
    sys: SystemId,
}

/// what 0x00076E80 reads of car 0
#[derive(Clone, Copy, Debug)]
pub struct SmokeCar {
    /// car data +128 / +320 translations: front wheels (FastCar 0 / 3), world
    pub front_wheels: [Vec3; 2],
    /// car data +0 row 0 = model x (lateral), world
    pub model_x: Vec3,
    /// VehicleEntity 0x001C66C0
    pub heading: Vec3,
    /// VsAIRaceVehicle 0x00024870 (forward speed; < 0 flips the z of the tuning vectors)
    pub forward_speed: f32,
    /// VsAIRaceVehicle 0x00024220 (m/s)
    pub speed: f32,
}

/// the bonnet smoke part of `byte_6D6808`: +0 created, +8 headlight count (physics' `CarDamageStats.headlights`),
/// +12/+16 right type/system, +20 right transform, +84/+88 left, +92 left transform, +156 running, +160 shader
#[derive(Resource, Debug, Default)]
pub struct CarDamageSmoke {
    pub created: bool,
    right: Option<Side>,
    left: Option<Side>,
    pub running: bool,
    /// +20 / +92 (the systems' transforms)
    pub transforms: [Mat4; 2],
}

fn shader() -> EffectShader {
    // 0x000769E0 "BonnetSmoke": PARTTEX enginesmoke, blend 2, SetZState(1, 0, 0), stencil write mask 0
    EffectShader::new(EffectTexture::Particle("enginesmoke".into()), 2).z(true, false)
}

fn create_side(mgr: &mut ElParticleSystemManager, name: &str) -> Side {
    let mut ty = ElParticleType::new(MAX_PARTICLES, 1, 0, name);
    ty.set_shader_code(shader());
    ty.set_trail(TrailParams { time: 0.0, size_end: 1.0, colour: wxyz(0.0, 0.0, 0.0, 0.0) });
    ty.set_emission_rate(ControlCurve::constant(CREATE_RATE), CREATE_JITTER);
    let (ty, _) = mgr.add_type(ty);
    let sys = mgr.add_system(name, ty, false).expect("type just added");
    Side { ty, sys }
}

impl CarDamageSmoke {
    /// 0x000766B0 → 0x00076730 + 0x000769E0
    pub fn create(&mut self, mgr: &mut ElParticleSystemManager) {
        if self.created {
            return;
        }
        self.running = false;
        self.right = Some(create_side(mgr, "bonnetSmokeRight"));
        self.left = Some(create_side(mgr, "bonnetSmokeLeft"));
        self.created = true;
    }

    /// 0x000766F0
    pub fn reset(&mut self, mgr: &mut ElParticleSystemManager) {
        if !self.created {
            self.create(mgr);
            return;
        }
        self.stop(mgr);
    }

    /// 0x00076870 (only while +156, which nothing sets)
    pub fn stop(&mut self, mgr: &mut ElParticleSystemManager) {
        if self.running {
            for s in [self.left, self.right].into_iter().flatten() {
                mgr.stop(s.sys);
            }
            self.running = false;
        }
    }

    /// debug: what the original never does
    pub fn force_start(&mut self, mgr: &mut ElParticleSystemManager) {
        for s in [self.right, self.left].into_iter().flatten() {
            mgr.start(s.sys);
        }
        self.running = true;
    }

    /// 0x00076780
    pub fn shutdown(&mut self, mgr: &mut ElParticleSystemManager) {
        if !self.created {
            return;
        }
        self.stop(mgr);
        for s in [self.left.take(), self.right.take()].into_iter().flatten() {
            mgr.remove_type(s.ty);
            mgr.remove_system(s.sys, true);
        }
        self.created = false;
    }

    /// 0x00078090 × the headlight count step: min(2·mph/240, 1) from 0.2 mph, times (count clamped 1..4) / 4 as an
    /// integer division (0 below four broken headlights; kept)
    pub fn level(speed: f32, headlights: u32) -> f32 {
        let mph = (speed as f64 * MPH as f64) as f32;
        let l = if mph >= MPH_MIN { (LEVEL_SCALE * (mph as f64 / MPH_RANGE as f64) as f32 + LEVEL_BASE).min(1.0) } else { 0.0 };
        let n = headlights.clamp(1, COUNT_DIV);
        (n / COUNT_DIV) as f32 * l
    }

    /// 0x00076E80, smoke part — per rendered frame for car 0
    pub fn update(&mut self, mgr: &mut ElParticleSystemManager, car: &SmokeCar, headlights: u32) {
        let (Some(right), Some(left)) = (self.right, self.left) else { return };
        let mid = (car.front_wheels[0] + car.front_wheels[1]) * 0.5;
        let mut p = mid + car.model_x * SIDE;
        p.y += LIFT;
        p.z += Z_OFFSET;
        // Renderer 0x002ABBC0: unit rows + translation
        self.transforms[0] = Mat4::from_translation(p);
        let mut p = mid + (-car.model_x) * SIDE;
        p.y += LIFT;
        p.z += Z_OFFSET;
        self.transforms[1] = Mat4::from_translation(p);
        let fwd = car.heading.normalize_or_zero();
        let x = fwd.cross(Vec3::Y).normalize_or_zero();
        let y = x.cross(fwd).normalize_or_zero();
        let frame = Mat4::from_cols(x.extend(0.0), y.extend(0.0), fwd.extend(0.0), Vec4::W);
        let level = Self::level(car.speed, headlights);
        let sign = if car.forward_speed < 0.0 { -1.0 } else { 1.0 };
        let zs = |v: Vec3| Vec3::new(v.x, v.y, sign * v.z);
        let flip = |v: Vec3| Vec3::new(0.0 - v.x, v.y, sign * v.z);
        // Renderer 0x002ABF70 = rotation only
        let vel_r = frame.transform_vector3(lerp3(zs(VEL[0]), zs(VEL[1]), level));
        let vel_l = frame.transform_vector3(lerp3(flip(VEL[0]), flip(VEL[1]), level));
        let spread = frame.transform_vector3(lerp3(SPREAD[0], SPREAD[1], level));
        let grav_r = frame.transform_vector3(lerp3(zs(GRAVITY[0]), zs(GRAVITY[1]), level));
        let grav_l = frame.transform_vector3(lerp3(flip(GRAVITY[0]), flip(GRAVITY[1]), level));
        for (side, t, v) in [(right, 0, vel_r), (left, 1, vel_l)] {
            if let Some(s) = mgr.system_mut(side.sys) {
                s.transform = Some(self.transforms[t]);
                s.velocity = v;
                s.shape = EmitterShape::new(3, frame, BOX);
            }
        }
        let rate = lerp(RATE[0] as f32, RATE[1] as f32, level);
        let life = lerp(LIFE[0], LIFE[1], level);
        let one = 1.0f32;
        let v0 = lerp(SIZE_V0[0], SIZE_V0[1], level);
        let v1 = lerp(SIZE_V1[0], SIZE_V1[1], level);
        let v2 = lerp(SIZE_V2[0], SIZE_V2[1], level);
        let t1 = lerp(SIZE_T1[0], SIZE_T1[1], level);
        for (side, v, g) in [(right, vel_r, grav_r), (left, vel_l, grav_l)] {
            let Some(t) = mgr.type_mut(side.ty) else { continue };
            t.set_velocity(v, spread);
            t.set_gravity(g);
            t.set_lifetime(life);
            t.set_size(ControlCurve::keys(v0 * one, t1 * one, v1 * one, life * one, v2 * one, false), SIZE_RANDOM * one);
            t.set_emission_rate(ControlCurve::constant(rate), RATE_JITTER);
            t.set_colour(ColourCurve::two(COL0, life, COL1, false), Vec4::ZERO);
            t.set_rotation_params(ROTATION);
        }
    }
}

pub struct CarDamageSmokePlugin;

impl Plugin for CarDamageSmokePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::el_particle::ElParticlePlugin>() {
            app.add_plugins(crate::el_particle::ElParticlePlugin);
        }
        app.init_resource::<CarDamageSmoke>()
            .add_systems(PostUpdate, car_damage_smoke_frame.in_set(crate::effect_draw::EffectOrder::ElDamage).before(ElParticleSet));
    }
}

fn force() -> bool {
    std::env::var("VELOCITY_BONNET_SMOKE_FORCE").is_ok_and(|v| v == "1")
}

#[allow(clippy::too_many_arguments)]
fn car_damage_smoke_frame(
    mut smoke: ResMut<CarDamageSmoke>,
    mut mgr: ResMut<ElParticleSystemManager>,
    mut init: MessageReader<RaceEffectsInit>,
    mut reset: MessageReader<RaceEffectsReset>,
    mut shutdown: MessageReader<RaceEffectsShutdown>,
    cars: Res<EffectCars>,
    damage: Option<Res<CarDamageManager>>,
    roots: Query<(&CarRender, &Transform)>,
) {
    let (smoke, mgr) = (&mut *smoke, &mut *mgr);
    if shutdown.read().count() > 0 {
        smoke.shutdown(mgr);
    }
    if init.read().count() > 0 {
        smoke.create(mgr);
        if force() {
            smoke.force_start(mgr);
        }
    }
    if reset.read().count() > 0 {
        smoke.reset(mgr);
        if force() {
            smoke.force_start(mgr);
        }
    }
    if !smoke.created {
        return;
    }
    let Some(c) = cars.cars.first() else { return };
    let model = crate::slipstream_render::model_matrix(c.matrix);
    let render = render_car_matrix(&roots, &cars, c.vehicle).unwrap_or(model);
    let inv = model.inverse();
    let at = |w: usize| render.transform_point3(inv.transform_point3(c.wheels[w].centre));
    let car = SmokeCar {
        front_wheels: [at(0), at(3)],
        model_x: render.x_axis.truncate(),
        heading: c.heading,
        forward_speed: c.velocity.dot(c.matrix.x_axis.truncate()),
        speed: c.speed,
    };
    let headlights = if force() { COUNT_DIV } else { damage.map_or(0, |d| d.stats.headlights) };
    smoke.update(mgr, &car, headlights);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_needs_four_headlights() {
        assert_eq!(CarDamageSmoke::level(50.0, 3), 0.0);
        assert_eq!(CarDamageSmoke::level(50.0, 4), (2.0 * (50.0f64 * 2.25 / 240.0) as f32).min(1.0));
        assert_eq!(CarDamageSmoke::level(200.0, 9), 1.0);
        assert_eq!(CarDamageSmoke::level(0.05, 4), 0.0);
    }

    #[test]
    fn systems_are_tuned_but_never_started() {
        let mut mgr = ElParticleSystemManager::default();
        let mut s = CarDamageSmoke::default();
        s.create(&mut mgr);
        let car = SmokeCar {
            front_wheels: [Vec3::new(-0.8, 0.3, 1.5), Vec3::new(0.8, 0.3, 1.5)],
            model_x: Vec3::X,
            heading: Vec3::Z,
            forward_speed: 1.0,
            speed: 100.0,
        };
        s.update(&mut mgr, &car, 4);
        // either side of the bonnet centre, 0.5 up
        assert_eq!(s.transforms[0].w_axis.truncate(), Vec3::new(-0.5, 0.8, 1.5));
        assert_eq!(s.transforms[1].w_axis.truncate(), Vec3::new(0.5, 0.8, 1.5));
        let r = mgr.type_ref(s.right.unwrap().ty).unwrap();
        // level 1: velocity (0.4, 0, 0) in the frame x = fwd × up = -X here, lifetime 1.2, rate 50, gravity back
        assert!((r.data.velocity - Vec3::new(-0.4, 0.0, 0.0)).length() < 1e-6);
        assert_eq!((r.data.lifetime, r.data.rate.v[0]), (1.2, 50.0));
        assert!((r.data.gravity - Vec3::new(0.0, 0.0, -6.0)).length() < 1e-6);
        let l = mgr.type_ref(s.left.unwrap().ty).unwrap();
        assert!((l.data.velocity - Vec3::new(0.4, 0.0, 0.0)).length() < 1e-6);
        for _ in 0..10 {
            mgr.tick(0.1);
        }
        assert_eq!(mgr.particle_count(), 0);
        s.force_start(&mut mgr);
        for _ in 0..10 {
            mgr.tick(0.1);
        }
        assert!(mgr.particle_count() > 0);
        s.shutdown(&mut mgr);
        assert!(!s.created);
    }
}
