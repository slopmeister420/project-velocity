//! Puddle render (object `unk_720840`, "Puddle Render Init"; functions 0x00131690..0x00133970 sit in LightStreak.cpp's
//! range, the file name is not in the binary): random puddles along the race line of wet London races, and the
//! per-frame wheel test that starts tyre spray and the camera car's lens splash (`LensSpray.splash`).
//!
//! API (core): at InGameInit `PuddleRender::init(london && wet, lines, rng)` (0x00131690 / 0x00131AC0, `lines` =
//! race section 0's race line and its two edge lines through `PuddleRoadLines`), then once per game update before
//! `LensSpray::update`: `PuddleRender::update(&PuddleInput, &EffectCars, &mut LensSpray)` (0x001326A0). The puddle
//! quads (0x00131F50) are drawn every frame through `EffectDraw`. `wheel_wet` feeds the tyre spray (`tyrespray`,
//! 0x00152860(car, wheel, 2.0)); the puddle mist (`PuddleMist`, elParticle "puddle%dx%d", build 0x00132D20,
//! emission 0x001331D0) applies the test's hits each rendered frame. Runtime puddles (physics' friction spots):
//! `PuddleAdd` per `PhysicsPuddleMessage` (0x00133750), `PuddleRestart` at RestartRace (0x00131EC0).

use bevy::prelude::*;
use visos::handy::crt_rand::CrtRand;

use crate::effect_draw::{engine_colour, EffectDraw,EffectShader, EffectTexture, EffectVertex};
use crate::lens_spray_manager::LensSpray;
use crate::race_effects::EffectCars;
use crate::track_render::TrackCamera;

/// at most 100 puddles (0x00131AC0)
pub const MAX_PUDDLES: usize = 100;
/// `flt_686F04` / `flt_686F08`: spacing along the race line
pub const SPACING: [f32; 2] = [25.0, 50.0];
/// `flt_686EF0`: chance of a gap, `flt_686EF4` / `flt_686EF8` its length
pub const GAP_CHANCE: f32 = f32::from_bits(0x3DCC_CCCD);
pub const GAP: [f32; 2] = [200.0, 400.0];
/// `flt_686EFC` / `flt_686F00`: puddle side lengths
pub const SIZE: [f32; 2] = [15.0, 25.0];
/// `flt_686F10`: extra reach of the car test
pub const CAR_REACH: f32 = 2.0;
/// `dword_686F0C`: weight of the view-angle alpha of type 2 puddles
pub const FRESNEL: f32 = 1.0;
/// `byte_721B34` = VISOS_ColourXBOX(0x80, 0, 0, 0) (static init 0x00425B70): the puddles' vertex colour
pub const COLOUR: u32 = 0x8000_0000;
/// 0x00131F50: u of the four texture quarters in 1/4096
pub const U_STEP: u16 = 1024;

/// a straight stand-in for race section 0 (viewer and tests): `length` m from `start` along `dir`, edges at
/// ±`half_width` to the side
pub struct StraightLine {
    pub start: Vec3,
    pub dir: Vec3,
    pub length: f32,
    pub half_width: f32,
}

impl PuddleRoadLines for StraightLine {
    fn raceline_length(&mut self) -> f32 {
        self.length
    }
    fn raceline_point(&mut self, d: f32) -> Vec3 {
        self.start + self.dir * d
    }
    fn edge_point(&mut self, edge: usize, p: Vec3) -> Vec3 {
        let side = self.dir.cross(Vec3::Y).normalize_or_zero() * self.half_width;
        if edge == 0 {
            p - side
        } else {
            p + side
        }
    }
}

/// one 48-byte record (+20 in the object)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Puddle {
    /// +0 position, +12 (w) radius
    pub pos: Vec3,
    pub radius: f32,
    /// +16 / +20 direction in x / z (normalised)
    pub dir: Vec2,
    /// +24 race line distance
    pub distance: f32,
    /// +28 `rand() & 1` (the draw's fresnel type 2 never occurs)
    pub kind: u32,
    /// +32 / +36 side lengths
    pub size: [f32; 2],
    /// +40 life (480 for a friction-spot puddle, 0 generated; nothing in the original reads it)
    pub life: i32,
    /// +44 flags: bit 1 = added at runtime (removed by the restart 0x00131EC0); 0 generated
    pub flags: u32,
}

/// a friction-spot puddle from physics (PhysicsObjectManager 0x001E1A20) as the 48-byte record it built
impl From<&velocity_physics::physics_objects::PhysicsPuddle> for Puddle {
    fn from(p: &velocity_physics::physics_objects::PhysicsPuddle) -> Self {
        Self {
            pos: Vec3::from_array(p.position),
            radius: p.radius,
            dir: Vec2::from_array(p.direction),
            distance: p.race_distance,
            kind: p.kind as u32,
            size: p.size,
            life: p.life,
            flags: p.flags,
        }
    }
}

/// core: forward each `velocity_physics::plugin::PhysicsPuddleMessage` as `PuddleAdd((&msg.0).into())`
/// (LightStreak__133750 on `unk_720840`)
#[derive(Message, Clone, Copy, Debug)]
pub struct PuddleAdd(pub Puddle);

/// core: at `GameManager::RestartRace` (0x000BE20C → 0x00131EC0) — drops the runtime puddles again
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct PuddleRestart;

/// LightStreak.cpp 0x00133800, the qsort comparator on +24 (-1 / 0 / 1)
fn compare_distance(a: &Puddle, b: &Puddle) -> i32 {
    if b.distance <= a.distance {
        (a.distance > b.distance) as i32
    } else {
        -1
    }
}

/// race section 0's lines as VsAIRoadLine offers them (velocity_ai; core implements this)
pub trait PuddleRoadLines {
    /// `VsAIRoadLine::GetLength` of the race line
    fn raceline_length(&mut self) -> f32;
    /// `VsAIRoadLine::GetPointAtDistance(raceline, d, 0)`
    fn raceline_point(&mut self, d: f32) -> Vec3;
    /// edge 0 = race section +8, 1 = +12: `FindClosestSegment(p)`, `GetDistanceAlongLine`, `GetPointAtDistance`
    fn edge_point(&mut self, edge: usize, p: Vec3) -> Vec3;
}

/// what 0x001326A0 reads besides the cars
#[derive(Resource, Clone, Debug, Default)]
pub struct PuddleInput {
    /// `VsAIWorld::GetFirstRaceSection` is set
    pub race_section: bool,
    /// race distance of each `EffectCars` car's driver (VsAIDriver +680, 0x00015CA0); `None` = no driver
    pub distance: Vec<Option<f32>>,
    /// `EffectCars` index of the camera's car (camera manager +0x60, 0x0012DC20)
    pub camera_car: Option<usize>,
    /// RainSurface 0x00137090 skips its render (and so the puddles 0x00131F50) when `byte_6F7157` is set or view 0's
    /// camera mode (VsCameraManager 0x0005FFB0) is 3
    pub rain_surface_off: bool,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct PuddleRender {
    /// +12 puddles on (set to 1 unconditionally), +13 wheel test on (+12 or "ladown")
    pub puddles_on: bool,
    pub test_on: bool,
    /// +16 count, +20 records
    pub puddles: Vec<Puddle>,
    /// `dword_686EA8`: puddle the last tested car is in (-1 none)
    pub last_hit: i32,
    /// VehicleEntity +4872 + wheel per `EffectCars` car: the wheel is in a puddle / on a wet surface this frame
    pub wheel_wet: Vec<[bool; 4]>,
    /// the 0x001331D0 calls of the last test, in call order (applied by `puddle_mist_frame` on the rendered pose)
    pub mist_hits: Vec<MistHit>,
    /// 0x001326A0 ran since the mist last applied it (its rate reset happens only then)
    pub tested: bool,
    /// bumped by every `init` (0x00131690 builds the mist systems again)
    pub generation: u32,
}

/// one `LightStreak__1331D0(car, wheel, system, type, fresnel)` call
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MistHit {
    /// `EffectCars` index
    pub car: usize,
    pub wheel: usize,
    /// puddle +28 == 2 (never generated)
    pub fresnel: bool,
}

fn rand_range(rng: &mut CrtRand, lo: f32, hi: f32) -> f32 {
    // VsRandRange 0x000134F0: VsRandUnit() * (hi - lo) + lo on the x87 stack, stored as float
    (rng.rand_unit() * (hi - lo) as f64 + lo as f64) as f32
}

impl PuddleRender {
    /// LightStreak.cpp 0x00131690 — "puddle1" / "tyremist" shaders (not drawn here); +12 = 1, +13 = +12 || "ladown" (so both are
    /// always on); puddles only when the track is wet and its name contains "london" (`london_wet`)
    pub fn init(&mut self, london_wet: bool, lines: Option<&mut dyn PuddleRoadLines>, rng: &mut CrtRand) {
        *self = Self { puddles_on: true, test_on: true, last_hit: -1, generation: self.generation.wrapping_add(1), ..default() };
        if london_wet {
            if let Some(l) = lines {
                self.generate(l, rng);
            }
        }
    }

    /// LightStreak.cpp 0x00131AC0 — puddles every 25..50 m of the race line (10% chance of another 200..400 m), each at a random
    /// point between the two edge lines, random direction, sides 15..25, radius = (a + b) / 4
    pub fn generate(&mut self, lines: &mut dyn PuddleRoadLines, rng: &mut CrtRand) {
        self.puddles.clear();
        let len = lines.raceline_length();
        let mut d = rand_range(rng, SPACING[0], SPACING[1]);
        while len > d && self.puddles.len() < MAX_PUDDLES {
            let p = lines.raceline_point(d);
            let a = lines.edge_point(0, p);
            let b = lines.edge_point(1, p);
            let t = rand_range(rng, 0.0, 1.0);
            let pos = a * t + b * (1.0 - t);
            let mut dir = Vec2::new(rand_range(rng, -1.0, 1.0), rand_range(rng, -1.0, 1.0));
            let n = (dir.x * dir.x + dir.y * dir.y).sqrt();
            dir = Vec2::new(dir.x / n, dir.y / n);
            let s0 = rand_range(rng, SIZE[0], SIZE[1]);
            let s1 = rand_range(rng, SIZE[0], SIZE[1]);
            let kind = (rng.rand() & 1) as u32;
            self.puddles.push(Puddle { pos, radius: (s0 + s1) * 0.25, dir, distance: d, kind, size: [s0, s1], life: 0, flags: 0 });
            d += rand_range(rng, SPACING[0], SPACING[1]);
            if (GAP_CHANCE as f64) > rng.rand_unit() {
                d += rand_range(rng, GAP[0], GAP[1]);
            }
        }
    }

    /// LightStreak__133750 — append the record (+16 count, +20 + 48·n; no bound check, the Vec just grows), qsort
    /// it by race distance (0x001337C0 / 0x00133800); 0x00133A20 (RainSurface `byte_726388` +18 = 1, turning its
    /// render — and so the puddle draw — on for the rest of the race) is `RainSurface.wet = true` in
    /// `puddle_messages`. The sort is a stable insertion sort; the CRT qsort may order equal distances differently.
    pub fn add(&mut self, p: Puddle) {
        self.puddles.push(p);
        let mut i = self.puddles.len() - 1;
        while i > 0 && compare_distance(&self.puddles[i - 1], &self.puddles[i]) > 0 {
            self.puddles.swap(i - 1, i);
            i -= 1;
        }
    }

    /// LightStreak.cpp 0x00131EC0 (GameManager::RestartRace 0x000BE20C) — from the last record down, while it has flag bit 1:
    /// clear the bit and drop it; stops at the first record without (original: runtime puddles sorted in front of
    /// a generated one stay)
    pub fn restart(&mut self) {
        while let Some(p) = self.puddles.last_mut() {
            if p.flags & 1 != 1 {
                break;
            }
            p.flags &= !1;
            self.puddles.pop();
        }
    }

    /// LightStreak.cpp 0x001326A0 (from the puddle draw 0x00131F50 when +13) — per car: with a race section the puddle test
    /// 0x00132A00 at its driver's race distance, else every wheel on surface 1 (VE +4868) sprays. A spraying wheel
    /// of the camera's car sets the lens splash to 1 (`LightStreak__133910(1.0)`).
    pub fn update(&mut self, input: &PuddleInput, cars: &EffectCars, lens: &mut LensSpray) {
        if !self.test_on {
            return;
        }
        self.wheel_wet.resize(cars.cars.len(), [false; 4]);
        // the mist types' rates are zeroed first; only this frame's hits set them again
        self.mist_hits.clear();
        self.tested = true;
        for (i, car) in cars.cars.iter().enumerate() {
            if self.puddles_on && input.race_section {
                if let Some(d) = input.distance.get(i).copied().flatten() {
                    self.puddle_test(input, cars, i, d, lens);
                }
            } else {
                for w in 0..4 {
                    let wet = car.wheels[w].surface == 1;
                    if wet && input.camera_car == Some(i) {
                        lens.splash = 1.0;
                    }
                    if wet {
                        self.mist_hits.push(MistHit { car: i, wheel: w, fresnel: false });
                    }
                    self.wheel_wet[i][w] = wet;
                }
            }
        }
    }

    /// LightStreak.cpp 0x00132A00 — the last puddle whose race distance (`this` +44 at 0x132A69 = record +24) is below the car's
    /// (0 if none), or the next (wrapping) when the car is not within radius + 2 of it; then each wheel in
    /// contact within the radius sprays
    fn puddle_test(&mut self, input: &PuddleInput, cars: &EffectCars, i: usize, d: f32, lens: &mut LensSpray) {
        self.last_hit = -1;
        let n = self.puddles.len() as i32;
        let rec = |k: i32| -> Puddle { if k >= 0 && k < n { self.puddles[k as usize] } else { Puddle::default() } };
        let mut found = -1;
        let mut j = 0;
        let mut k = n - 1;
        while k >= 0 {
            if d > rec(k).distance {
                j = k;
                break;
            }
            k -= 1;
        }
        let car = &cars.cars[i];
        let pos = car.matrix.w_axis.truncate();
        let r = rec(j).radius + CAR_REACH;
        if r * r <= (pos - rec(j).pos).length_squared() {
            j += 1;
            if j >= n {
                j = 0;
            }
            let r = rec(j).radius + CAR_REACH;
            if r * r > (pos - rec(j).pos).length_squared() {
                found = j;
            }
        } else {
            found = j;
        }
        if found == -1 {
            return;
        }
        let p = rec(found);
        for w in 0..4 {
            self.wheel_wet[i][w] = false;
            let wheel = &car.wheels[w];
            if wheel.contacts > 0 {
                let hit = p.radius * p.radius > (wheel.centre - p.pos).length_squared();
                if hit {
                    self.last_hit = found;
                    if input.camera_car == Some(i) {
                        if lens.splash <= 0.0 {
                            info!("lens splash: car {i} wheel {w} in puddle {found} at race distance {d}");
                        }
                        lens.splash = 1.0;
                    }
                    self.mist_hits.push(MistHit { car: i, wheel: w, fresnel: p.kind == 2 });
                }
                self.wheel_wet[i][w] = hit;
            }
        }
    }
}

impl PuddleRender {
    /// the shader the draw uses (0x00131690, `*(this + 4 * (byte_72639B == 0))`; `byte_72639B` is never written,
    /// so +4): `\Particle\PARTTEX.VCD` "puddle1", blend 2, SetZState(0, 0), 0x002A0F40(6) / 0x002A0FC0(1) alpha
    /// test, clamp, stencil EQUAL ref 8 pass KEEP (bit 8 = RainSurface's "roaddetail" road layer, 0x00136C50):
    /// drawn untested by the puddle road camera, kept to the road pixels the roaddetail pass marked
    pub fn shader() -> EffectShader {
        EffectShader::new(EffectTexture::Particle("puddle1".into()), 2).z(false, false).wrap(false, false).alpha_test(6, 1).road_stencil()
    }

    /// LightStreak.cpp-range 0x00131F50 (from RainSurface 0x00137090 -> 0x00162400 before the road reflection):
    /// one quad per puddle in view (sphere pos / radius, 0x0005E070 — culling is left to the gpu here): corner =
    /// pos − A/2 − B/2 with A = (dir.x, 0, dir.z)·side[1], B = (−dir.z, 0, dir.x)·side[0], strip corner, +A, +B,
    /// +B+A; u = texture quarter `kind & 3` (1024/4096 wide), v 0..1; colour `byte_721B34`, type 2 (never generated)
    /// alpha = _ftol(((1 − dy/|d|)²·w + (1 − w))·128) of the camera offset d
    pub fn draw(&self, cam: Vec3, draw: &mut EffectDraw) {
        if !self.puddles_on || self.puddles.is_empty() {
            return;
        }
        let sh = Self::shader();
        for p in &self.puddles {
            let a = Vec3::new(p.dir.x * p.size[1], 0.0, p.dir.y * p.size[1]);
            let b = Vec3::new((0.0 - p.dir.y) * p.size[0], 0.0, p.dir.x * p.size[0]);
            let v0 = p.pos - a * 0.5 - b * 0.5;
            let v1 = v0 + a;
            let v2 = v0 + b;
            let v3 = v2 + a;
            let k = (p.kind & 3) as u16;
            let u0 = (k * U_STEP) as f32 / 4096.0;
            let u1 = (k * U_STEP + U_STEP) as f32 / 4096.0;
            let mut colour = COLOUR;
            if p.kind == 2 {
                let d = cam - p.pos;
                let f = 1.0 - d.y / d.length();
                let f = f * f * FRESNEL + (1.0 - FRESNEL);
                colour = ((f * 128.0) as i32 as u32 & 0xFF) << 24 | (COLOUR & 0xFF_FFFF);
            }
            let c = engine_colour(colour);
            draw.strip(
                &sh,
                &[
                    EffectVertex::new(v0, c, Vec2::new(u0, 0.0)),
                    EffectVertex::new(v1, c, Vec2::new(u1, 0.0)),
                    EffectVertex::new(v2, c, Vec2::new(u0, 1.0)),
                    EffectVertex::new(v3, c, Vec2::new(u1, 1.0)),
                ],
            );
        }
    }
}

/// `dword_686EC0`: lifetime at build and the lifetime per unit of upward speed
pub const MIST_LIFE: f32 = f32::from_bits(0x3E61_47AE);
/// `dword_686EC4`: no mist below this speed (m/s)
pub const MIST_MIN_SPEED: f32 = 0.5;
/// `dword_686EE4`: emitter drop below the wheel centre; `dword_686EE0`: disc radius
pub const MIST_DROP: f32 = 0.5;
pub const MIST_DISC: f32 = f32::from_bits(0x3D4C_CCCD);
/// `flt_686F14`: system velocity = car velocity × this
pub const MIST_SYSTEM_VEL: f32 = 2.0;
/// static init 0x00425AE0..0x00425B57 (x, y, z): base `byte_721B38`, per m/s `byte_721B48`, cap `byte_721B24` of the
/// particle velocity (x sideways, y up, z along the car velocity)
pub const MIST_VEL_BASE: Vec3 = Vec3::new(0.0, 1.5, 0.0);
pub const MIST_VEL_SLOPE: Vec3 = Vec3::new(f32::from_bits(0x3E19_999A), f32::from_bits(0x3E4C_CCCD), f32::from_bits(0x3DCC_CCCD));
pub const MIST_VEL_MAX: Vec3 = Vec3::new(5.0, 5.0, 100.0);
/// spread scale min(speed·a + b, c): `dword_686ECC`/`686EC8`/`flt_686ED0`, fresnel puddles `686ED8`/`686ED4`/`686EDC`
pub const MIST_SCALE: [[f32; 3]; 2] = [
    [f32::from_bits(0x3CA3_D70A), f32::from_bits(0x3D4C_CCCD), 1.0],
    [f32::from_bits(0x3CF5_C28F), f32::from_bits(0x3DCC_CCCD), 1.25],
];
/// `unk_72082C` (static init 0x00425A33): velocity spread before the scale
pub const MIST_SPREAD: Vec3 = Vec3::new(2.0, 2.0, 2.0);
/// `dword_686EAC`: emission rate while a wheel is wet
pub const MIST_RATE: f32 = 50.0;
/// size keys v0 `dword_686EB0`, v1 `686EB4`, v2 `686EB8` (× scale), t1 = life × `686EBC`; randomness `flt_7207C8`
/// (bss, never written)
pub const MIST_SIZE: [f32; 3] = [1.0, 3.0, 4.0];
pub const MIST_SIZE_T1: f32 = 0.5;
pub const MIST_SIZE_RANDOM: f32 = 0.0;
/// trail `dword_686EEC` / `686EE8` (one node: unused)
pub const MIST_TRAIL: [f32; 2] = [0.5, 1.0];
/// colours (static init, `CarDamageFX__78FE0` w, x, y, z): `dword_7207CC` (0.3; 1, 1, 1), fresnel `dword_721B14`
/// (0.75; 1, 1, 1), end `dword_72081C` (0; 0.8, 0.8, 0.8)
pub const MIST_COL0: [f32; 4] = [0.3, 1.0, 1.0, 1.0];
pub const MIST_COL0_FRESNEL: [f32; 4] = [0.75, 1.0, 1.0, 1.0];
pub const MIST_COL1: [f32; 4] = [0.0, 0.8, 0.8, 0.8];
/// build values (0x00132D20): velocity (0, 2, 0) spread (1, 0.5, 1), gravity (0, −10, 0), size 0.1, colour to 0.3 s
const BUILD_VEL: Vec3 = Vec3::new(0.0, 2.0, 0.0);
const BUILD_SPREAD: Vec3 = Vec3::new(1.0, 0.5, 1.0);
const BUILD_GRAVITY: Vec3 = Vec3::new(0.0, -10.0, 0.0);
const BUILD_SIZE: f32 = f32::from_bits(0x3DCC_CCCD);
const BUILD_COLOUR_T: f32 = f32::from_bits(0x3ECC_CCCD);
const MIST_MAX_PARTICLES: i32 = 100;

fn c4(c: [f32; 4]) -> Vec4 {
    crate::el_particle::el_particle_system::wxyz(c[0], c[1], c[2], c[3])
}

/// what 0x001331D0 reads of the car (car data / VehicleEntity), on the rendered pose
#[derive(Clone, Copy, Debug)]
pub struct MistCar {
    /// car data +128 + 64·wheel translations (wheel render matrices, FastCar order), world
    pub wheels: [Vec3; 4],
    /// car data +0 row 0 (model x, lateral)
    pub model_x: Vec3,
    /// VehicleEntity 0x001C6480
    pub velocity: Vec3,
}

/// the puddle mist systems of `unk_720840`: `dword_7207FC` types / `unk_7207DC` systems, 2 × 4 slots at 2·wheel + k
/// (k = 0 car 0, 1 every other car — they share one system per wheel)
#[derive(Resource, Debug, Default)]
pub struct PuddleMist {
    pub slots: [Option<(crate::el_particle::TypeId, crate::el_particle::SystemId)>; 8],
    pub generation: u32,
}

impl PuddleMist {
    /// "splash" (0x00131690 +8): PARTTEX `tyremist`, blend 2, SetZState(1, 0, 0), alpha 0x002A0F40(7) /
    /// 0x002A0FC0(1), stencil write mask 0
    pub fn shader() -> EffectShader {
        EffectShader::new(EffectTexture::Particle("tyremist".into()), 2).z(true, false).alpha_test(7, 1)
    }

    /// LightStreak.cpp 0x00132D20 (from 0x00131690 when +13): "puddle%dx%d" (k, wheel) types (100, 1, 0) and their systems, started
    pub fn build(&mut self, mgr: &mut crate::el_particle::ElParticleSystemManager) {
        use crate::el_particle::el_particle_control_curve::{ColourCurve, ControlCurve};
        use crate::el_particle::el_particle_system::ElParticleType;
        for k in 0..2 {
            for w in 0..4 {
                let name = format!("puddle{k}x{w}");
                let mut ty = ElParticleType::new(MIST_MAX_PARTICLES, 1, 0, &name);
                ty.set_velocity(BUILD_VEL, BUILD_SPREAD);
                ty.set_gravity(BUILD_GRAVITY);
                ty.set_shader_code(Self::shader());
                ty.set_emission_rate(ControlCurve::constant(0.0), 0.0);
                ty.set_size(ControlCurve::constant(BUILD_SIZE), 0.0);
                ty.set_colour(ColourCurve::two(c4(MIST_COL0), BUILD_COLOUR_T, c4(MIST_COL1), false), Vec4::ZERO);
                ty.set_lifetime(MIST_LIFE);
                let (ty, _) = mgr.add_type(ty);
                let Some(sys) = mgr.add_system(&name, ty, false) else { continue };
                mgr.start(sys);
                self.slots[2 * w + k] = Some((ty, sys));
            }
        }
    }

    /// LightStreak.cpp 0x001330D0 (destructor list 0x00131AA0): systems removed at once, types removed
    pub fn shutdown(&mut self, mgr: &mut crate::el_particle::ElParticleSystemManager) {
        for s in self.slots.iter_mut() {
            if let Some((ty, sys)) = s.take() {
                mgr.remove_system(sys, true);
                mgr.remove_type(ty);
            }
        }
    }

    /// head of 0x001326A0: every type's rate to a constant 0
    pub fn reset_rates(&mut self, mgr: &mut crate::el_particle::ElParticleSystemManager) {
        use crate::el_particle::el_particle_control_curve::ControlCurve;
        for (ty, _) in self.slots.iter().flatten() {
            if let Some(t) = mgr.type_mut(*ty) {
                t.set_emission_rate(ControlCurve::constant(0.0), 0.0);
            }
        }
    }

    /// LightStreak.cpp 0x001331D0(car, wheel, slot, fresnel): from 0.5 m/s, a disc 0.5 under the wheel throws mist up and sideways
    /// (odd wheels to −x), faster cars throw it higher, further and bigger
    pub fn emit(&mut self, mgr: &mut crate::el_particle::ElParticleSystemManager, car_index: usize, wheel: usize, car: &MistCar, fresnel: bool) {
        use crate::el_particle::el_particle_control_curve::{ColourCurve, ControlCurve};
        use crate::el_particle::el_particle_system::{wxyz, TrailParams};
        use crate::el_particle::vs_pool_allocator::EmitterShape;
        let Some((ty, sys)) = self.slots[2 * wheel + (car_index != 0) as usize] else { return };
        let mut pos = car.wheels[wheel];
        let speed = car.velocity.length();
        if MIST_MIN_SPEED > speed {
            return;
        }
        pos.y -= MIST_DROP;
        if let Some(s) = mgr.system_mut(sys) {
            // 0x001339B0(2, pos, (r, 0, 0)): identity + translation, disc
            s.shape = EmitterShape::new(2, Mat4::from_translation(pos), Vec3::new(MIST_DISC, 0.0, 0.0));
            s.velocity = car.velocity * MIST_SYSTEM_VEL;
        }
        let y = (speed * MIST_VEL_SLOPE.y + MIST_VEL_BASE.y).min(MIST_VEL_MAX.y);
        let x = (speed * MIST_VEL_SLOPE.x + MIST_VEL_BASE.x).min(MIST_VEL_MAX.x);
        let z = (speed * MIST_VEL_SLOPE.z + MIST_VEL_BASE.z).min(MIST_VEL_MAX.z);
        let mut v = if wheel & 1 != 0 { car.model_x * (0.0 - x) } else { car.model_x * x };
        v.y = y;
        v += car.velocity * (z / speed);
        let [a, b, c] = MIST_SCALE[fresnel as usize];
        let scale = (speed * a + b).min(c);
        let life = y * MIST_LIFE;
        let Some(t) = mgr.type_mut(ty) else { return };
        t.set_velocity(v, MIST_SPREAD * scale);
        t.set_emission_rate(ControlCurve::constant(MIST_RATE), 0.0);
        t.set_trail(TrailParams { time: MIST_TRAIL[0], size_end: MIST_TRAIL[1], colour: wxyz(0.0, 0.0, 0.0, 0.0) });
        t.set_size(
            ControlCurve::keys(MIST_SIZE[0] * scale, life * MIST_SIZE_T1, MIST_SIZE[1] * scale, life, MIST_SIZE[2] * scale, false),
            MIST_SIZE_RANDOM,
        );
        t.set_lifetime(life);
        let c0 = if fresnel { MIST_COL0_FRESNEL } else { MIST_COL0 };
        t.set_colour(ColourCurve::two(c4(c0), life, c4(MIST_COL1), false), Vec4::ZERO);
    }
}

/// GameManager 0x000B9010 order: the puddle test (0x00131F50 → 0x001326A0, core's `PuddleRender::update`) runs in
/// the render path before the particle tick; its 0x001331D0 calls are applied here on the rendered car pose
#[allow(clippy::too_many_arguments)]
fn puddle_mist_frame(
    mut mist: ResMut<PuddleMist>,
    mut puddles: ResMut<PuddleRender>,
    mut mgr: ResMut<crate::el_particle::ElParticleSystemManager>,
    cars: Option<Res<EffectCars>>,
    roots: Query<(&crate::car_render::CarRender, &Transform)>,
) {
    let (mist, mgr) = (&mut *mist, &mut *mgr);
    let Some(cars) = cars else { return };
    if mist.generation != puddles.generation {
        mist.shutdown(mgr);
        mist.generation = puddles.generation;
        if puddles.test_on {
            mist.build(mgr);
        }
    }
    if !puddles.tested {
        return;
    }
    puddles.tested = false;
    mist.reset_rates(mgr);
    for h in puddles.mist_hits.clone() {
        let Some(c) = cars.cars.get(h.car) else { continue };
        // wheel centres moved onto the rendered (lerped) car pose, as the tyre spray does
        let model = crate::slipstream_render::model_matrix(c.matrix);
        let render = crate::race_effects::render_car_matrix(&roots, &cars, c.vehicle).unwrap_or(model);
        let inv = model.inverse();
        let wheels = c.wheels.map(|w| render.transform_point3(inv.transform_point3(w.centre)));
        let mc = MistCar { wheels, model_x: render.x_axis.truncate(), velocity: c.velocity };
        mist.emit(mgr, h.car, h.wheel, &mc, h.fresnel);
    }
}

fn draw_puddles(r: Res<PuddleRender>, input: Res<PuddleInput>, cam: Query<&GlobalTransform, With<TrackCamera>>, mut draw: ResMut<EffectDraw>) {
    if input.rain_surface_off {
        return;
    }
    let Some(cam) = cam.iter().next() else { return };
    r.draw(cam.translation(), &mut draw);
}

/// `PuddleRestart` then each `PuddleAdd` (0x00131EC0 / 0x00133750); an add sets RainSurface +18 (0x00133A20)
fn puddle_messages(
    mut restarts: MessageReader<PuddleRestart>,
    mut adds: MessageReader<PuddleAdd>,
    mut r: ResMut<PuddleRender>,
    mut rs: Option<ResMut<crate::rain_surface::RainSurface>>,
) {
    if restarts.read().count() > 0 {
        r.restart();
    }
    for a in adds.read() {
        r.add(a.0);
        if let Some(rs) = rs.as_mut() {
            rs.wet = true;
        }
        info!("puddle added at {:?} race distance {} ({} puddles)", a.0.pos, a.0.distance, r.puddles.len());
    }
}

pub struct PuddleRenderPlugin;

impl Plugin for PuddleRenderPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::el_particle::ElParticlePlugin>() {
            app.add_plugins(crate::el_particle::ElParticlePlugin);
        }
        app.init_resource::<PuddleRender>()
            .init_resource::<PuddleInput>()
            .init_resource::<PuddleMist>()
            .add_message::<PuddleAdd>()
            .add_message::<PuddleRestart>()
            .add_systems(Update, puddle_messages)
            .add_systems(PostUpdate, (
                    draw_puddles.in_set(crate::effect_draw::EffectOrder::Puddles),
                    puddle_mist_frame.in_set(crate::effect_draw::EffectOrder::ElMist).before(crate::el_particle::ElParticleSet),
                ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::race_effects::{EffectCar, EffectWheel};

    struct Straight;
    impl PuddleRoadLines for Straight {
        fn raceline_length(&mut self) -> f32 {
            1000.0
        }
        fn raceline_point(&mut self, d: f32) -> Vec3 {
            Vec3::new(0.0, 0.0, d)
        }
        fn edge_point(&mut self, edge: usize, p: Vec3) -> Vec3 {
            p + Vec3::X * if edge == 0 { -5.0 } else { 5.0 }
        }
    }

    fn car_at(p: Vec3) -> EffectCar {
        let wheel = EffectWheel { centre: p, radius: 0.33, contacts: 1, sliding: false, surface: 0, no_marks: false };
        EffectCar {
            vehicle: 0,
            matrix: Mat4::from_translation(p),
            velocity: Vec3::ZERO,
            speed: 0.0,
            heading: Vec3::Z,
            smoke_off: false,
            wheels: [wheel; 4],
            ground_plane: None,
        }
    }

    #[test]
    fn runtime_add_and_restart() {
        let generated = |d: f32| Puddle { distance: d, ..default() };
        let mut r = PuddleRender { puddles: vec![generated(10.0), generated(40.0), generated(90.0)], ..default() };
        let phys = velocity_physics::physics_objects::PhysicsPuddle {
            position: [1.0, 2.0, 3.0],
            radius: 15.0,
            direction: [0.0, 1.0],
            race_distance: 120.0,
            kind: 2,
            size: [30.0, 30.0],
            life: 480,
            flags: 0xCCCC_CCCD,
        };
        r.add((&phys).into());
        r.add(Puddle { distance: 50.0, flags: 1, ..default() });
        let d: Vec<f32> = r.puddles.iter().map(|p| p.distance).collect();
        assert_eq!(d, [10.0, 40.0, 50.0, 90.0, 120.0]);
        assert_eq!((r.puddles[4].kind, r.puddles[4].radius, r.puddles[4].life), (2, 15.0, 480));
        // the restart drops trailing flagged records only: the one sorted before 90 stays (original)
        r.restart();
        let d: Vec<f32> = r.puddles.iter().map(|p| p.distance).collect();
        assert_eq!(d, [10.0, 40.0, 50.0, 90.0]);
        // and the message path sets RainSurface +18
        let mut app = App::new();
        app.add_message::<PuddleAdd>().add_message::<PuddleRestart>().init_resource::<PuddleRender>();
        app.insert_resource(crate::rain_surface::RainSurface::default()).add_systems(Update, puddle_messages);
        app.world_mut().write_message(PuddleAdd((&phys).into()));
        app.update();
        assert!(app.world().resource::<crate::rain_surface::RainSurface>().wet);
        assert_eq!(app.world().resource::<PuddleRender>().puddles.len(), 1);
        app.world_mut().write_message(PuddleRestart);
        app.update();
        assert!(app.world().resource::<PuddleRender>().puddles.is_empty());
    }

    #[test]
    fn puddle_quads() {
        let mut r = PuddleRender { puddles_on: true, ..default() };
        r.puddles.push(Puddle { pos: Vec3::new(1.0, 2.0, 3.0), radius: 5.0, dir: Vec2::X, distance: 0.0, kind: 1, size: [4.0, 10.0], ..default() });
        let mut d = EffectDraw::default();
        r.draw(Vec3::ZERO, &mut d);
        let v = &d.batches[0].1;
        // strip of 4 = two triangles: corner, +A (x 10), +B (z 4), +B+A
        assert_eq!(v.len(), 6);
        assert_eq!((v[0].pos, v[1].pos, v[2].pos), (Vec3::new(-4.0, 2.0, 1.0), Vec3::new(6.0, 2.0, 1.0), Vec3::new(-4.0, 2.0, 5.0)));
        assert_eq!(v[5].pos, Vec3::new(6.0, 2.0, 5.0));
        assert_eq!((v[0].uv, v[5].uv), (Vec2::new(0.25, 0.0), Vec2::new(0.5, 1.0)));
        assert_eq!(v[0].argb >> 24, 0xFF);
    }

    #[test]
    fn puddles_and_splash() {
        let mut r = PuddleRender::default();
        r.init(true, Some(&mut Straight), &mut CrtRand::default());
        assert!(!r.puddles.is_empty() && r.puddles.len() <= MAX_PUDDLES);
        for p in &r.puddles {
            assert!(p.pos.x.abs() <= 5.0 && (7.5..=12.5).contains(&p.radius));
        }
        // the puddle chosen is the last one behind the car's race distance (record +24)
        assert!(r.puddles.len() >= 3);
        let mid = r.puddles[1];
        let mut cars = EffectCars::default();
        cars.cars.push(car_at(mid.pos));
        let input = PuddleInput { race_section: true, distance: vec![Some(mid.distance + 0.5)], camera_car: Some(0), ..default() };
        let mut lens = LensSpray::default();
        r.update(&input, &cars, &mut lens);
        assert_eq!(lens.splash, 1.0);
        assert_eq!(r.last_hit, 1);
        // just before it: puddle 0 is chosen, out of reach, so the next one (1) is tested and hit
        let mut lens = LensSpray::default();
        r.update(&PuddleInput { distance: vec![Some(mid.distance - 0.5)], ..input.clone() }, &cars, &mut lens);
        assert_eq!((lens.splash, r.last_hit), (1.0, 1));
        // no race section: surface 1 wheels
        let mut lens = LensSpray::default();
        cars.cars[0] = car_at(Vec3::splat(5000.0));
        cars.cars[0].wheels[2].surface = 1;
        r.update(&PuddleInput { race_section: false, distance: vec![], camera_car: Some(0), ..default() }, &cars, &mut lens);
        assert_eq!((lens.splash, r.wheel_wet[0]), (1.0, [false, false, true, false]));
        // without puddles (dry or not london) nothing is in reach of the zeroed record
        let mut dry = PuddleRender::default();
        dry.init(false, None, &mut CrtRand::default());
        let mut lens = LensSpray::default();
        dry.update(&input, &cars, &mut lens);
        assert_eq!(lens.splash, 0.0);
    }

    #[test]
    fn puddle_mist() {
        use crate::el_particle::ElParticleSystemManager;
        let mut mgr = ElParticleSystemManager::default();
        let mut m = PuddleMist::default();
        m.build(&mut mgr);
        assert_eq!(mgr.systems.len(), 8);
        let (_, sys) = m.slots[2 * 3 + 1].unwrap();
        assert_eq!(mgr.system(sys).unwrap().name, "puddle1x3");
        // built silent: rate 0
        for _ in 0..30 {
            mgr.tick(1.0 / 60.0);
        }
        assert!(mgr.systems.iter().all(|e| e.sys.pool.is_empty()));
        // car 0 wheel 1 at 20 m/s along +z, model x = +x: thrown to −x, up, forward
        let car = MistCar { wheels: [Vec3::new(0.0, 0.3, 0.0); 4], model_x: Vec3::X, velocity: Vec3::new(0.0, 0.0, 20.0) };
        m.reset_rates(&mut mgr);
        m.emit(&mut mgr, 0, 1, &car, false);
        let (ty, sys) = m.slots[2].unwrap();
        let t = mgr.type_mut(ty).unwrap();
        // y = min(20·0.2 + 1.5, 5) = 5, x = 3, z = 2: v = (−3, 5, 0 + 20·2/20)
        assert!((t.data.velocity - Vec3::new(-3.0, 5.0, 2.0)).length() < 1e-5, "{:?}", t.data.velocity);
        assert!((t.data.lifetime - 5.0 * MIST_LIFE).abs() < 1e-6);
        // spread scale min(20·0.02 + 0.05, 1) = 0.45
        assert!((t.data.velocity_spread - Vec3::splat(0.9)).length() < 1e-5);
        for _ in 0..30 {
            mgr.tick(1.0 / 60.0);
        }
        assert!(!mgr.system(sys).unwrap().pool.is_empty());
        // too slow: untouched (the reset rate stays 0)
        m.reset_rates(&mut mgr);
        let slow = MistCar { velocity: Vec3::new(0.0, 0.0, 0.4), ..car };
        m.emit(&mut mgr, 2, 0, &slow, false);
        let (ty, _) = m.slots[1].unwrap();
        assert_eq!(mgr.type_mut(ty).unwrap().data.velocity, BUILD_VEL);
        m.shutdown(&mut mgr);
        assert!(mgr.systems.is_empty() && mgr.types.iter().all(Option::is_none));
    }
}
