//! Render\tyrespray.cpp (0x00152860..0x00154BD0; the object `byte_73750C` and its init 0x001523D0, update
//! 0x001527F0 and shutdown 0x00152700 sit in TrailBlazerRender's range): spray thrown up by the wheels on wet
//! tracks and in puddles, built on elParticle. Per car four 108-byte wheel records; front wheels (FastCar 0 / 3)
//! own one "mistsprayfront" system that rides with the wheel, rear wheels (1 / 2) a "mistsprayback" system riding
//! with the wheel plus a "mistsprayworld" system left in the world.
//!
//! API (core): `TyreSprayInit { wet, track }` at InGameInit after the first `EffectCars` (0x001523D0: only when
//! the track is wet or its name contains "ladown"), `TyreSprayShutdown` (or `RaceEffectsShutdown`) at the end.
//! Puddle hits (`PuddleRender.wheel_wet`, 0x001326A0 → 0x00152860(car, wheel, 2.0)) are read by itself every frame;
//! `TyreSprayRender::set_strength` is the same call for other users.

use bevy::prelude::*;

use crate::car_render::CarRender;
use crate::effect_draw::{EffectClock,EffectShader, EffectTexture};
use crate::el_particle::el_particle_control_curve::{ColourCurve, ControlCurve};
use crate::el_particle::el_particle_system::{wxyz, ElParticleType, TrailParams};
use crate::el_particle::vs_pool_allocator::EmitterShape;
use crate::el_particle::{ElParticleSet, ElParticleSystemManager, SystemId, TypeId};
use crate::puddle_render::PuddleRender;
use crate::race_effects::{render_car_matrix, EffectCars, RaceEffectsShutdown};

/// 0x00152DE0 / 0x00153060 / 0x001532E0: `new elParticleType(150, 1, MODE_FREELY_ROTATING, name)`
pub const MAX_PARTICLES: i32 = 150;
/// rate jitter the systems are created with (0x3E99999A)
pub const CREATE_JITTER: f32 = f32::from_bits(0x3E99_999A);

/// front spray 0x00153630: size keys `dword_687428` 1 / `dword_687430` 0.2 s / `dword_68742C` 2 / `dword_687438`
/// 0.5 s / `dword_687434` 4 (all · speed), rate `dword_68743C` 20 · speed, jitter `dword_687440` 0.4, life and
/// colour time `dword_687438` 0.5, rotation 0..`dword_687444` 360°, spin `dword_687448` 15..`dword_68744C` 45°/s
pub const FRONT_SIZE: [f32; 5] = [1.0, f32::from_bits(0x3E4C_CCCD), 2.0, 0.5, 4.0];
pub const FRONT_RATE: f32 = 20.0;
pub const FRONT_JITTER: f32 = f32::from_bits(0x3ECC_CCCD);
pub const FRONT_LIFE: f32 = 0.5;
pub const FRONT_ROTATION: [f32; 3] = [360.0, 15.0, 45.0];
/// back spray 0x00153CB0: size `dword_687450` 0.4 / `dword_687458` 0.2 s / `dword_687454` 2.5 / `dword_68746C` 1 s /
/// `dword_68745C` 5, randomness `dword_687468` 0.3 (all · strength), rate `dword_687460` 20 (unscaled), jitter
/// `dword_687464` 0.2, life and colour time 1, rotation `dword_687470..78` 360 / 15 / 45
pub const BACK_SIZE: [f32; 5] = [f32::from_bits(0x3ECC_CCCD), f32::from_bits(0x3E4C_CCCD), 2.5, 1.0, 5.0];
pub const BACK_RATE: f32 = 20.0;
pub const BACK_JITTER: f32 = f32::from_bits(0x3E4C_CCCD);
pub const BACK_RANDOM: f32 = f32::from_bits(0x3E99_999A);
pub const BACK_LIFE: f32 = 1.0;
pub const BACK_ROTATION: [f32; 3] = [360.0, 15.0, 45.0];
/// world spray 0x00154310: size `dword_68747C` 1 / `dword_687484` 0.7 s / `dword_687480` 20 / `dword_68748C` 4 s /
/// `dword_687488` 20 (· speed), rate `dword_687490` 8 · speed, jitter `dword_687494` 0.2, randomness `flt_687498`
/// 0.3, life and colour time 4, rotation 0..`dword_68749C` 360, spin 0..`dword_6874A0` 15
pub const WORLD_SIZE: [f32; 5] = [1.0, f32::from_bits(0x3F33_3333), 20.0, 4.0, 20.0];
pub const WORLD_RATE: f32 = 8.0;
pub const WORLD_JITTER: f32 = f32::from_bits(0x3E4C_CCCD);
pub const WORLD_RANDOM: f32 = f32::from_bits(0x3E99_999A);
pub const WORLD_LIFE: f32 = 4.0;
pub const WORLD_ROTATION: [f32; 2] = [360.0, 15.0];
/// 0x00154820 / 0x00154870: speed factor min(`dword_6874A8` 1.75 · mph/240 + `dword_6874A4` 0.1, 1) from 0.1 mph
/// (`flt_62AE28` 2.25 · m/s, `dword_62AE24` 240); above strength 1: speed · (1 + `dword_6874B4` 0.33·(s − 1)),
/// size 0.5 + `dword_6874B8` 2·(s − 1)
pub const MPH: f32 = 2.25;
pub const SPEED_RANGE: f32 = 240.0;
pub const SPEED_SCALE: f32 = 1.75;
pub const SPEED_BASE: f32 = f32::from_bits(0x3DCC_CCCD);
pub const SPEED_BOOST: f32 = f32::from_bits(0x3EA8_F5C3);
pub const SIZE_BOOST: f32 = 2.0;
/// 0x00153560: strength · `dword_6874B0` 0.9 per rendered frame, below `dword_6874AC` 0.01 off
pub const DECAY: f32 = f32::from_bits(0x3F66_6666);
pub const MIN_STRENGTH: f32 = f32::from_bits(0x3C23_D70A);
/// static init 0x00426560..0x004268A9 (x, y, z[, w]): front velocity `unk_7375AC` (z along −fwd, y), spread
/// `unk_73754C`, box `unk_737890` (z = offset along fwd), colours `unk_73760C` → `unk_73759C`, gravity `dword_73753C`
pub const FRONT_VEL: Vec3 = Vec3::new(0.0, 0.0, 10.0);
pub const FRONT_SPREAD: Vec3 = Vec3::new(0.0, 0.0, 2.0);
pub const FRONT_BOX: Vec3 = Vec3::new(0.0, f32::from_bits(0x3DCC_CCCD), f32::from_bits(0x3DCC_CCCD));
pub const FRONT_COL: [Vec4; 2] = [Vec4::new(1.0, 1.0, 1.0, 0.25), Vec4::new(1.0, 1.0, 1.0, 0.0)];
pub const FRONT_GRAVITY: Vec3 = Vec3::ZERO;
/// back: velocity `unk_73752C`, spread `unk_7375DC`, box `unk_73755C`, colours `unk_73758C` → `unk_73757C`,
/// gravity `dword_7374EC`
pub const BACK_VEL: Vec3 = Vec3::new(0.0, 3.0, 7.0);
pub const BACK_SPREAD: Vec3 = Vec3::new(3.0, 0.0, 3.0);
pub const BACK_BOX: Vec3 = Vec3::splat(f32::from_bits(0x3DCC_CCCD));
pub const BACK_COL: [Vec4; 2] = [Vec4::new(1.0, 1.0, 1.0, f32::from_bits(0x3E99_999A)), Vec4::new(1.0, 1.0, 1.0, 0.0)];
pub const BACK_GRAVITY: Vec3 = Vec3::new(0.0, -10.0, 0.0);
/// world: box and offset `unk_7375BC`, velocity `unk_7374FC`, spread `unk_7375FC`, gravity `unk_7375CC`, colours
/// `unk_73756C` → `unk_7375EC`
pub const WORLD_BOX: Vec3 = Vec3::new(0.0, 0.0, 1.0);
pub const WORLD_VEL: Vec3 = Vec3::ZERO;
pub const WORLD_SPREAD: Vec3 = Vec3::ZERO;
pub const WORLD_GRAVITY: Vec3 = Vec3::ZERO;
pub const WORLD_COL: [Vec4; 2] = [Vec4::new(1.0, 1.0, 1.0, f32::from_bits(0x3E4C_CCCD)), Vec4::new(1.0, 1.0, 1.0, 0.0)];

/// one type + its system
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SprayPair {
    pub ty: TypeId,
    pub sys: SystemId,
}

/// one wheel (108 bytes)
#[derive(Clone, Debug, PartialEq)]
pub struct TyreSprayWheel {
    /// +0 strength (1 at creation, 2 from a puddle, · 0.9 per frame)
    pub strength: f32,
    /// +4 / +8 front or back spray, +12 / +16 world spray (rear wheels)
    pub near: Option<SprayPair>,
    pub world: Option<SprayPair>,
    /// +24 car (index into `EffectCars`), +28 wheel, +32 car index
    pub car: usize,
    pub wheel: usize,
    /// +36 the near system's transform (translation at the wheel)
    pub matrix: Mat4,
    /// +100 speed factor, +104 size factor
    pub speed: f32,
    pub size: f32,
}

/// the object `byte_73750C`
#[derive(Resource, Clone, Debug, Default)]
pub struct TyreSprayRender {
    /// +0 the track is wet (VsAudio::IsTrackWet): spray strength never drops below 1
    pub wet: bool,
    /// +12 records (4 per car), +16 race cars, +20 TFT cars, +24 total
    pub cars: Vec<[TyreSprayWheel; 4]>,
    /// +28
    pub initialised: bool,
}

/// what the update reads of one car (VehicleEntity / VsAIRaceVehicle)
#[derive(Clone, Copy, Debug)]
pub struct SprayCar {
    /// car data +128 + 64·wheel translation (the rendered wheel positions)
    pub wheels: [Vec3; 4],
    /// VehicleEntity 0x001C66C0 (+4788)
    pub heading: Vec3,
    /// VsAIRaceVehicle 0x00024220 (+20) m/s
    pub speed: f32,
    /// VsAIRaceVehicle 0x00024870: velocity · forward (only its sign is used)
    pub forward_speed: f32,
}

#[derive(Message, Clone, Debug)]
pub struct TyreSprayInit {
    /// VsAudio::IsTrackWet
    pub wet: bool,
    /// track directory name (VsAIPoliceVehicle 0x00017CB0), lower-cased by the check
    pub track: String,
}

#[derive(Message, Clone, Debug)]
pub struct TyreSprayShutdown;

fn spray_shader() -> EffectShader {
    // 0x00152B90 "mistspray": PARTTEX tyremist, blend 2, SetZState(1, 0, 0), stencil write mask 0
    EffectShader::new(EffectTexture::Particle("tyremist".into()), 2).z(true, false)
}

/// tyrespray.cpp 0x00152DE0 / 0x00153060 / 0x001532E0 — type "mistspray<kind><car>x<wheel>" with the spray shader, no trail,
/// a constant rate, registered and started
fn create(mgr: &mut ElParticleSystemManager, kind: &str, vehicle: usize, wheel: usize, rate: f32) -> SprayPair {
    let name = format!("mistspray{kind}{vehicle}x{wheel}");
    let mut ty = ElParticleType::new(MAX_PARTICLES, 1, 0, &name);
    ty.set_shader_code(spray_shader());
    ty.set_trail(TrailParams { time: 0.0, size_end: 1.0, colour: wxyz(0.0, 0.0, 0.0, 0.0) });
    ty.set_emission_rate(ControlCurve::constant(rate), CREATE_JITTER);
    let (ty, _) = mgr.add_type(ty);
    let sys = mgr.add_system(&name, ty, false).expect("type just added");
    mgr.start(sys);
    SprayPair { ty, sys }
}

/// the spray frame: x = normalise(fwd × up), y = normalise(x × fwd) (VsAITools 0x000293D0 = cross)
fn axes(fwd: Vec3) -> (Vec3, Vec3) {
    let x = fwd.cross(Vec3::Y).normalize_or_zero();
    let y = x.cross(fwd).normalize_or_zero();
    (x, y)
}

/// rows x, y, z (glam columns = D3D rows), translation `t`
fn rows(x: Vec3, y: Vec3, z: Vec3, t: Vec3) -> Mat4 {
    Mat4::from_cols(x.extend(0.0), y.extend(0.0), z.extend(0.0), t.extend(1.0))
}

impl TyreSprayWheel {
    /// tyrespray.cpp 0x00152B90 — record of `car`'s `wheel`: rear wheels (1, 2) get world + back spray, the front ones front spray
    fn new(mgr: &mut ElParticleSystemManager, car: usize, vehicle: usize, wheel: usize) -> Self {
        let mut w = Self { strength: 1.0, near: None, world: None, car, wheel, matrix: Mat4::IDENTITY, speed: 0.0, size: 0.0 };
        if wheel == 2 || wheel == 1 {
            w.world = Some(create(mgr, "world", vehicle, wheel, WORLD_RATE));
            w.near = Some(create(mgr, "back", vehicle, wheel, BACK_RATE));
        } else {
            w.near = Some(create(mgr, "front", vehicle, wheel, FRONT_RATE));
        }
        w
    }

    fn rear(&self) -> bool {
        self.wheel == 2 || self.wheel == 1
    }

    /// tyrespray.cpp 0x00152CF0
    fn shutdown(&mut self, mgr: &mut ElParticleSystemManager) {
        for p in [self.world.take(), self.near.take()].into_iter().flatten() {
            mgr.remove_type(p.ty);
            mgr.remove_system(p.sys, true);
        }
    }

    /// tyrespray.cpp 0x00154870 — speed factor (+100) and size factor (+104) from the car speed and the strength
    pub fn speed_factors(&mut self, speed: f32) {
        let v = (speed as f64 * MPH as f64) as f32;
        self.speed = v / SPEED_RANGE;
        if v >= SPEED_BASE {
            // 0x00154820
            self.speed = (SPEED_SCALE * self.speed + SPEED_BASE).min(1.0);
        } else {
            self.speed = 0.0;
        }
        if self.strength <= 1.0 {
            self.size = 0.5;
            self.speed *= self.strength;
        } else {
            let s = self.strength - 1.0;
            self.size = s * SIZE_BOOST + 0.5;
            self.speed = ((s * SPEED_BOOST + 1.0) * self.speed).min(1.0);
        }
    }

    /// tyrespray.cpp 0x00153560 — `frames` = rendered 60 Hz frames this step (the per-frame decay), 0 while paused
    pub fn update(&mut self, mgr: &mut ElParticleSystemManager, car: &SprayCar, wet: bool, frames: f32) {
        if frames > 0.0 {
            self.strength *= DECAY.powf(frames);
        }
        if wet {
            if self.strength < 1.0 {
                self.strength = 1.0;
            }
        } else if MIN_STRENGTH > self.strength {
            self.strength = 0.0;
            return;
        }
        self.speed_factors(car.speed);
        if self.rear() {
            self.update_world(mgr, car);
            self.update_back(mgr, car);
        } else {
            self.update_front(mgr, car);
        }
    }

    /// tyrespray.cpp 0x00153630 — front spray: rides with the wheel (system transform = translation 0.1 ahead of it), thrown
    /// backwards from a thin box across the wheel
    fn update_front(&mut self, mgr: &mut ElParticleSystemManager, car: &SprayCar) {
        let Some(p) = self.near else { return };
        let sp = self.speed;
        let wheel = car.wheels[self.wheel];
        let mut fwd = car.heading;
        let (x, y) = axes(fwd);
        let shape = rows(x, y, fwd, Vec3::ZERO);
        if car.forward_speed < 0.0 {
            fwd = -fwd;
        }
        let centre = wheel - (-fwd * FRONT_BOX.z);
        // Renderer 0x002ABBC0: unit rows + translation
        self.matrix = Mat4::from_translation(centre);
        let vel = Vec3::new(-fwd.x * FRONT_VEL.z, FRONT_VEL.y, -fwd.z * FRONT_VEL.z) * sp;
        let spread = Vec3::new(-fwd.x * FRONT_SPREAD.z, FRONT_SPREAD.y, -fwd.z * FRONT_SPREAD.z) * sp;
        if let Some(t) = mgr.type_mut(p.ty) {
            t.set_emission_rate(ControlCurve::constant(FRONT_RATE * sp), FRONT_JITTER);
            let s = FRONT_SIZE;
            t.set_size(ControlCurve::keys(s[0] * sp, s[1] * sp, s[2] * sp, s[3] * sp, s[4] * sp, false), 0.0 * sp);
            let c0 = Vec4::new(FRONT_COL[0].x, FRONT_COL[0].y, FRONT_COL[0].z, FRONT_COL[0].w * sp);
            t.set_colour(ColourCurve::two(c0, FRONT_LIFE, FRONT_COL[1], false), Vec4::ZERO);
            t.set_lifetime(FRONT_LIFE * sp);
            t.set_rotation_params([0.0, FRONT_ROTATION[0], FRONT_ROTATION[1], FRONT_ROTATION[2]]);
            t.set_velocity(vel, spread);
            t.set_gravity(FRONT_GRAVITY);
        }
        if let Some(s) = mgr.system_mut(p.sys) {
            s.transform = Some(self.matrix);
            s.shape = EmitterShape::new(3, shape, FRONT_BOX);
            s.velocity = Vec3::ZERO;
        }
    }

    /// tyrespray.cpp 0x00153CB0 — back spray of a rear wheel: like the front one with the size factor, a rising throw and gravity
    fn update_back(&mut self, mgr: &mut ElParticleSystemManager, car: &SprayCar) {
        let Some(p) = self.near else { return };
        let sp = (self.size * self.speed).min(1.0);
        let wheel = car.wheels[self.wheel];
        let mut fwd = car.heading;
        let (x, y) = axes(fwd);
        let shape = rows(x, y, fwd, Vec3::ZERO);
        if car.forward_speed < 0.0 {
            fwd = -fwd;
        }
        let centre = wheel - (-fwd * BACK_BOX.z);
        self.matrix = Mat4::from_translation(centre);
        let vel = Vec3::new(-fwd.x * BACK_VEL.z, BACK_VEL.y, -fwd.z * BACK_VEL.z) * sp;
        let spread = Vec3::new(-fwd.x * BACK_SPREAD.z, BACK_SPREAD.y, -fwd.z * BACK_SPREAD.z) * sp;
        if let Some(t) = mgr.type_mut(p.ty) {
            t.set_emission_rate(ControlCurve::constant(BACK_RATE), BACK_JITTER);
            let s = BACK_SIZE;
            t.set_size(ControlCurve::keys(s[0] * sp, s[1] * sp, s[2] * sp, s[3] * sp, s[4] * sp, false), BACK_RANDOM * sp);
            let c0 = Vec4::new(BACK_COL[0].x, BACK_COL[0].y, BACK_COL[0].z, BACK_COL[0].w * sp);
            t.set_colour(ColourCurve::two(c0, BACK_LIFE, BACK_COL[1], false), Vec4::ZERO);
            t.set_lifetime(BACK_LIFE * sp);
            t.set_rotation_params([0.0, BACK_ROTATION[0], BACK_ROTATION[1], BACK_ROTATION[2]]);
            t.set_velocity(vel, spread);
            t.set_gravity(BACK_GRAVITY);
        }
        if let Some(s) = mgr.system_mut(p.sys) {
            s.transform = Some(self.matrix);
            s.shape = EmitterShape::new(3, shape, BACK_BOX);
            s.velocity = Vec3::ZERO;
        }
    }

    /// tyrespray.cpp 0x00154310 — world spray of a rear wheel: a still, growing mist spawned along a 2 m box 2 m ahead of the
    /// wheel, left in the world
    fn update_world(&mut self, mgr: &mut ElParticleSystemManager, car: &SprayCar) {
        let Some(p) = self.world else { return };
        let sp = self.speed;
        let wheel = car.wheels[self.wheel];
        let fwd = car.heading;
        let x = fwd.cross(Vec3::Y).normalize_or_zero();
        let at = wheel - (-fwd * WORLD_BOX.z) * 2.0;
        let shape = rows(x, Vec3::Y, fwd, at);
        if let Some(t) = mgr.type_mut(p.ty) {
            t.set_emission_rate(ControlCurve::constant(WORLD_RATE * sp), WORLD_JITTER);
            let s = WORLD_SIZE;
            t.set_size(ControlCurve::keys(s[0] * sp, s[1] * sp, s[2] * sp, s[3] * sp, s[4] * sp, false), WORLD_RANDOM);
            t.set_colour(ColourCurve::two(WORLD_COL[0], WORLD_LIFE, WORLD_COL[1], false), Vec4::ZERO);
            t.set_lifetime(WORLD_LIFE * sp);
            t.set_rotation_params([0.0, WORLD_ROTATION[0], 0.0, WORLD_ROTATION[1]]);
            t.set_velocity(WORLD_VEL * sp, WORLD_SPREAD * sp);
            t.set_gravity(WORLD_GRAVITY * sp);
        }
        if let Some(s) = mgr.system_mut(p.sys) {
            s.shape = EmitterShape::new(3, shape, WORLD_BOX);
            s.velocity = Vec3::ZERO;
        }
    }
}

impl TyreSprayRender {
    /// tyrespray.cpp 0x001523D0 — `cars` = (car index, vehicle id) in race-manager order
    pub fn init(&mut self, mgr: &mut ElParticleSystemManager, wet: bool, track: &str, cars: &[usize]) {
        let ladown = track.to_ascii_lowercase().contains("ladown");
        self.wet = wet;
        if self.initialised || !(self.wet || ladown) {
            return;
        }
        self.cars = cars
            .iter()
            .enumerate()
            .map(|(i, &vehicle)| std::array::from_fn(|w| TyreSprayWheel::new(mgr, i, vehicle, w)))
            .collect();
        self.initialised = true;
    }

    /// tyrespray.cpp 0x00152700
    pub fn shutdown(&mut self, mgr: &mut ElParticleSystemManager) {
        if !self.initialised {
            return;
        }
        for c in &mut self.cars {
            for w in c.iter_mut() {
                w.shutdown(mgr);
            }
        }
        self.cars.clear();
        self.initialised = false;
    }

    /// tyrespray.cpp 0x00152860(car, wheel, strength) — car = race-manager index (drivers, then TFT cars)
    pub fn set_strength(&mut self, car: usize, wheel: usize, strength: f32) {
        if !self.initialised {
            return;
        }
        match self.cars.get_mut(car) {
            Some(c) => c[wheel & 3].strength = strength,
            None => log::warn!("dSprayCarIndex < m_dNumCars (tyrespray.cpp 273)"),
        }
    }

    /// tyrespray.cpp 0x001527F0
    pub fn update(&mut self, mgr: &mut ElParticleSystemManager, cars: &[Option<SprayCar>], frames: f32) {
        if !self.initialised {
            return;
        }
        let wet = self.wet;
        for (c, rec) in self.cars.iter_mut().enumerate() {
            let Some(Some(car)) = cars.get(c) else { continue };
            for w in rec.iter_mut() {
                w.update(mgr, car, wet, frames);
            }
        }
    }
}

pub struct TyreSprayPlugin;

impl Plugin for TyreSprayPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::el_particle::ElParticlePlugin>() {
            app.add_plugins(crate::el_particle::ElParticlePlugin);
        }
        app.init_resource::<TyreSprayRender>()
            .add_message::<TyreSprayInit>()
            .add_message::<TyreSprayShutdown>()
            .add_systems(PostUpdate, tyre_spray_frame.in_set(crate::effect_draw::EffectOrder::ElSpray).before(ElParticleSet));
    }
}

/// GameManager 0x000B9F57 (before the particle tick): puddle hits, then every wheel record
#[allow(clippy::too_many_arguments)]
fn tyre_spray_frame(
    mut spray: ResMut<TyreSprayRender>,
    mut mgr: ResMut<ElParticleSystemManager>,
    mut init: MessageReader<TyreSprayInit>,
    mut shutdown: MessageReader<TyreSprayShutdown>,
    mut race_shutdown: MessageReader<RaceEffectsShutdown>,
    cars: Res<EffectCars>,
    puddles: Option<Res<PuddleRender>>,
    roots: Query<(&CarRender, &Transform)>,
    time: Res<Time>,
    clock: Res<EffectClock>,
) {
    let (spray, mgr) = (&mut *spray, &mut *mgr);
    if shutdown.read().count() + race_shutdown.read().count() > 0 {
        spray.shutdown(mgr);
    }
    for i in init.read() {
        let ids: Vec<usize> = cars.cars.iter().map(|c| c.vehicle).collect();
        spray.init(mgr, i.wet, &i.track, &ids);
    }
    if !spray.initialised {
        return;
    }
    if let Some(p) = puddles {
        for (c, wheels) in p.wheel_wet.iter().enumerate() {
            for (w, &wet) in wheels.iter().enumerate() {
                if wet {
                    spray.set_strength(c, w, 2.0);
                }
            }
        }
    }
    let sc: Vec<Option<SprayCar>> = cars
        .cars
        .iter()
        .map(|c| {
            // the wheel centres at the rendered (lerped) car pose: model-space offsets from the 24 Hz pose
            let model = crate::slipstream_render::model_matrix(c.matrix);
            let render = render_car_matrix(&roots, &cars, c.vehicle).unwrap_or(model);
            let inv = model.inverse();
            let wheels = c.wheels.map(|w| render.transform_point3(inv.transform_point3(w.centre)));
            Some(SprayCar { wheels, heading: c.heading, speed: c.speed, forward_speed: c.velocity.dot(c.matrix.x_axis.truncate()) })
        })
        .collect();
    let frames = if clock.paused { 0.0 } else { time.delta_secs() * 60.0 };
    spray.update(mgr, &sc, frames);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn car(speed: f32) -> SprayCar {
        SprayCar {
            wheels: [Vec3::new(1.3, 0.3, -0.7), Vec3::new(-1.3, 0.3, -0.7), Vec3::new(-1.3, 0.3, 0.7), Vec3::new(1.3, 0.3, 0.7)],
            heading: Vec3::X,
            speed,
            forward_speed: speed,
        }
    }

    #[test]
    fn speed_factors() {
        let mut w = TyreSprayWheel { strength: 1.0, near: None, world: None, car: 0, wheel: 0, matrix: Mat4::IDENTITY, speed: 0.0, size: 0.0 };
        w.speed_factors(0.0);
        assert_eq!((w.speed, w.size), (0.0, 0.5));
        w.speed_factors(30.0);
        // 67.5 mph: 1.75 · 0.28125 + 0.1
        assert!((w.speed - 0.592_187_5).abs() < 1e-6);
        w.strength = 2.0;
        w.speed_factors(30.0);
        assert!((w.speed - 0.787_609_4).abs() < 1e-5 && w.size == 2.5);
    }

    #[test]
    fn wet_track_sprays() {
        let mut mgr = ElParticleSystemManager::default();
        let mut s = TyreSprayRender::default();
        s.init(&mut mgr, false, "Tokyo", &[0]);
        assert!(!s.initialised);
        s.init(&mut mgr, true, "Tokyo", &[3]);
        assert_eq!((s.cars.len(), mgr.systems.len()), (1, 6));
        assert_eq!(mgr.system(s.cars[0][1].world.unwrap().sys).unwrap().name, "mistsprayworld3x1");
        let c = [Some(car(30.0))];
        for _ in 0..60 {
            s.update(&mut mgr, &c, 1.0);
            mgr.tick(1.0 / 60.0);
        }
        // wet: strength held at 1, every system emitting
        assert_eq!(s.cars[0][0].strength, 1.0);
        for e in &mgr.systems {
            assert!(!e.sys.pool.is_empty(), "{}", e.sys.name);
        }
        // front spray follows the wheel: particles behind the wheel in its local frame
        let front = mgr.system(s.cars[0][0].near.unwrap().sys).unwrap();
        assert!(front.pool.items.iter().all(|p| p.nodes[0].pos.x <= 0.1));
        s.shutdown(&mut mgr);
        assert!(mgr.systems.is_empty() && mgr.types.iter().all(Option::is_none) && mgr.buffer.used == 0);
    }

    #[test]
    fn dry_strength_decays() {
        let mut mgr = ElParticleSystemManager::default();
        let mut s = TyreSprayRender::default();
        s.init(&mut mgr, false, "LADown", &[0]);
        assert!(s.initialised);
        s.set_strength(0, 1, 2.0);
        let c = [Some(car(20.0))];
        s.update(&mut mgr, &c, 1.0);
        assert!((s.cars[0][1].strength - 1.8).abs() < 1e-6);
        // 0.9^n below 0.01 after 44 frames: off
        for _ in 0..50 {
            s.update(&mut mgr, &c, 1.0);
        }
        assert_eq!(s.cars[0][1].strength, 0.0);
    }
}
