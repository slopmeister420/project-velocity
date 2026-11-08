//! Render\FuryFX.cpp (0x001273B0..0x00128E40; the idb labels most of it FlareRender__ — the asserts at 0x00128680
//! and the "FuryFX::InitParticleFX" message name the file) — the nitrous ("fury") effects object `byte_71A0D0`:
//! textures +0 spec3 / +4 spec4Inv / +8 spec4, shaders +12 "bits" (unused) / +20 "beams" / +28 "streak" / +32
//! "mb_mask", then six `FuryInstance`s at +36 (one per vehicle index), plus the 40 light beam records
//! `dword_71CEB0` (two per car) and the beam styles `dword_71A078` / `71A0B0` / `71A094`.
//! GameManager 0x000B9010 per frame: after RainSurface, 0x00128E40 + 0x00129090 for each human player's vehicle,
//! then per view 0x001293E0 (sound, on/off, car beams, streaks) before TrackFX; late (0x000BA023, after the second
//! bloom, before the lens spray) 0x00129580 = a motion blur pass while the view's instance is on.

use std::collections::HashMap;

use bevy::math::{Mat4, Vec2, Vec3, Vec4};
use bevy::prelude::*;
use visos::handy::crt_rand::CrtRand;

use crate::effect_draw::{engine_colour, EffectClock, EffectDraw,EffectShader, EffectTexture, EffectVertex};
use crate::full_screen_effects_manager_pc_xbox::{FullScreenEffects, FseSet};
use crate::fury_instance::{streak_shader, FuryInstance, FuryVehicle, NosSound, BLUR_SCALE};
use crate::motion_blur_pc_xbox::{MotionBlur, MotionBlurInput};
use crate::zone_render_pc_xbox::{MotionBlurSecondInput, ZoneFx};

/// `NUM_FURYRENDER_INSTANCES`
pub const NUM_INSTANCES: usize = 6;
/// `MAX_CAR_BEAMS` (`dword_71CEB0`: two 156-byte records per car)
pub const MAX_CAR_BEAMS: usize = 40;
/// +152 of a beam record
pub const BEAM_POINTS: usize = 8;
/// `flt_686B64`: a race car's stored (car-relative) beam points move by this share of its travel per point
pub const BEAM_DRIFT: f32 = -0.25;

/// "Graphics.NOS FX" tweaks (0x001287B0)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FuryTweaks {
    /// "Fov Change" `dword_686B5C` (0..40): the static camera's fov added at full fury (read by the camera)
    pub fov_change: f32,
    /// "Max Nitro Speed" `dword_686B60` (0..100 mph): speed of a full speed factor
    pub max_nitro_speed: f32,
}

impl Default for FuryTweaks {
    fn default() -> Self {
        Self { fov_change: f32::from_bits(0x41A0_0000), max_nitro_speed: f32::from_bits(0x4282_0000) }
    }
}

/// a light beam record (156 bytes; reset 0x001274F0)
#[derive(Clone, Debug, PartialEq)]
pub struct FuryBeam {
    /// +0 points (world, or relative to `base` for race cars)
    pub points: [Vec3; BEAM_POINTS],
    /// +128 the car position of the last relative add
    pub base: Vec3,
    /// +144 next slot, +148 count, +152 capacity
    pub head: usize,
    pub count: usize,
    pub max: usize,
}

impl Default for FuryBeam {
    fn default() -> Self {
        Self { points: [Vec3::ZERO; BEAM_POINTS], base: Vec3::ZERO, head: 0, count: 0, max: BEAM_POINTS }
    }
}

/// a beam style (`dword_71A078` = the only one `dword_71A0CC` ever points at; 0x00128100 fills all three)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeamStyle {
    /// +0 (unread)
    pub colour_a: u32,
    /// +4 engine colour (CatsEyeRender 0x0011DE40: low byte red)
    pub colour: u32,
    /// +16 (unread)
    pub kind: u32,
    /// +20 half width
    pub width: f32,
}

pub const BEAM_STYLES: [BeamStyle; 3] = [
    BeamStyle { colour_a: 0, colour: 48, kind: 2, width: f32::from_bits(0x3E4C_CCCD) },
    BeamStyle { colour_a: 0x00FF_4040, colour: 0, kind: 2, width: 1.0 },
    BeamStyle { colour_a: 255, colour: 0, kind: 2, width: 0.5 },
];

/// VsVector4::Normalise 0x000135B0
fn normalise(v: Vec3) -> Vec3 {
    let l = v.length();
    if l.abs() <= 0.000001 {
        return Vec3::ZERO;
    }
    Vec3::new(v.x / l, v.y / l, v.z / l)
}

impl FuryBeam {
    /// FuryFX.cpp 0x001274F0
    pub fn reset(&mut self) {
        self.head = 0;
        self.count = 0;
        self.max = BEAM_POINTS;
        self.base = Vec3::ZERO;
    }

    /// FuryFX.cpp 0x00127560 — a world point (traffic cars)
    pub fn add(&mut self, p: Vec3) {
        self.points[self.head] = p;
        self.head += 1;
        if self.head >= self.max {
            self.head = 0;
        }
        if self.count < self.max {
            self.count += 1;
        }
    }

    /// FuryFX.cpp 0x00127600 — a point relative to the car at `pos`; the stored points first move by
    /// (pos − base) · `BEAM_DRIFT`
    pub fn add_relative(&mut self, p: Vec3, pos: Vec3) {
        let d = (pos - self.base) * BEAM_DRIFT;
        for k in 0..self.count {
            self.points[k] += d;
        }
        self.points[self.head] = p;
        self.head += 1;
        if self.head >= self.max {
            self.head = 0;
        }
        if self.count < self.max {
            self.count += 1;
        }
        self.base = pos;
    }

    /// FuryFX.cpp 0x00127740 — the ribbon (oldest point first, camera-facing, capped both ends): 2·count + 2
    /// vertices, alpha rising with the square of the vertex index to `a4`; nothing under two points
    pub fn strip(&self, style: &BeamStyle, cam: Vec3, a4: f32) -> Vec<EffectVertex> {
        let count = self.count;
        if count < 2 {
            return Vec::new();
        }
        let n = 2 * count + 2;
        let step = 1.0 / (n - 1) as f32;
        let mut out = vec![EffectVertex::default(); n];
        for (i, v) in out.iter_mut().enumerate() {
            let t = i as f32 * step;
            let t = t * t;
            let a = ((128.0 * a4) * t) as i32;
            v.argb = engine_colour(((a as u32) << 24) | (style.colour & 0x00FF_FFFF));
        }
        out[0].uv = Vec2::new(1.0, 0.0);
        out[n - 1].uv = Vec2::new(0.0, 1.0);
        let mut i = 1;
        while i < n - 1 {
            out[i].uv = Vec2::ZERO;
            out[i + 1].uv = Vec2::ONE;
            i += 2;
        }
        let w = style.width;
        let mut k = 0;
        let mut idx = self.head;
        if idx >= count {
            idx -= count;
        }
        let mut side = Vec3::ZERO;
        for i in 0..count {
            let mut p = self.points[idx];
            let next = if idx >= count - 1 { self.points[0] } else { self.points[idx + 1] };
            let d = p - next;
            p += self.base;
            let v = p - cam;
            if i != count - 1 {
                side = normalise(d.cross(v)) * w;
            }
            if i == 0 {
                out[k].pos = p + normalise(v.cross(side)) * w;
                k += 1;
            }
            out[k].pos = p + side;
            out[k + 1].pos = p - side;
            k += 2;
            if i == count - 1 {
                out[k].pos = p + normalise(v.cross(side)) * (0.0 - w);
                k += 1;
            }
            idx += 1;
            if idx >= count {
                idx = 0;
            }
        }
        out
    }
}

/// "beams" (+20, 0x00128100): spec3, blend 12, clamp, no alpha test, z test no write, stencil write mask 0
pub fn beam_shader() -> EffectShader {
    EffectShader::new(EffectTexture::Misc("spec3".into()), 12).wrap(false, false).z(true, false)
}

/// VsAudio::StartNos / StopNos from 0x001293E0 (core forwards them as `AudioCommand::StartNos` / `StopNos`;
/// StopNos repeats every unpaused frame while the player's nitrous is off, as the original)
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FuryNosAudio {
    pub start: bool,
}

/// the object `byte_71A0D0`
#[derive(Resource)]
pub struct FuryFx {
    pub tweaks: FuryTweaks,
    /// +36 the six instances (index = vehicle index)
    pub instances: Vec<FuryInstance>,
    /// `dword_71CEB0`
    pub beams: Vec<FuryBeam>,
    /// VsRandRange's CRT stream (own here)
    pub rng: CrtRand,
    /// `dword_6D4558 & 0x20`: the nos sound is playing (mirrored)
    pub nos_playing: bool,
    /// the vehicle index of each player view's instance (view +28), this frame
    pub view_vehicle: [Option<usize>; 2],
    /// 0x00128100 ran (InGameInit), cleared at shutdown
    pub initialised: bool,
    /// 0x00128A40 waits for the cars (CarRender data) to exist
    pending_particles: bool,
    /// 60 Hz frames not yet given a beam point
    beam_clock: f32,
    /// each car's root pose at the previous frame (beam points of the 60 Hz steps in between)
    last_pose: HashMap<u64, Transform>,
}

impl Default for FuryFx {
    fn default() -> Self {
        Self {
            tweaks: FuryTweaks::default(),
            instances: vec![FuryInstance::default(); NUM_INSTANCES],
            beams: vec![FuryBeam::default(); MAX_CAR_BEAMS],
            rng: CrtRand::default(),
            nos_playing: false,
            view_vehicle: [None; 2],
            initialised: false,
            pending_particles: false,
            beam_clock: 0.0,
            last_pose: HashMap::new(),
        }
    }
}

impl FuryFx {
    /// FuryFX.cpp 0x00128680 — vehicle `v`'s fury amount (camera fov / kite, motion blur fade, zone gate)
    pub fn amount(&self, v: usize) -> f32 {
        if v >= NUM_INSTANCES {
            log::error!("assert: nPlayer >=0 && nPlayer < NUM_FURYRENDER_INSTANCES");
            return 0.0;
        }
        self.instances[v].amount()
    }

    /// 0x00128680 for player view `p`'s vehicle (0 without one)
    pub fn view_amount(&self, p: usize) -> f32 {
        self.view_vehicle.get(p).copied().flatten().map_or(0.0, |v| self.amount(v))
    }

    /// FuryFX.cpp 0x00128100 (InGameInit): instances 0..5 for vehicles 0..5 (`views` = `dword_74A434` +292); the
    /// beam styles are constants here and ZoneRender 0x00162B60 / the tweaks 0x001287B0 are the zone's / ours
    pub fn init(&mut self, views: i32) {
        for (i, inst) in self.instances.iter_mut().enumerate() {
            inst.init(i, views);
        }
        self.initialised = true;
        self.pending_particles = true;
    }
}

type FuryCarQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static crate::car_render::CarRender,
        &'static Transform,
        Option<&'static crate::track_fx::CarHeadBeams>,
        &'static crate::flare_render::CarFlareLocators,
        Option<&'static crate::car_render::CarLightInput>,
        Option<&'static crate::car_damage_render::CarLightDamage>,
        &'static InheritedVisibility,
    ),
>;

/// InGameInit 0x000B6E85 → 0x00128100, RestartRace 0x000BE3A1 → 0x00128530 (each instance's 0x00128A40 again,
/// the old emitters left running — original), shutdown 0x00128DC0
fn fury_messages(
    mut fx: ResMut<FuryFx>,
    mut particles: Option<ResMut<crate::stateless_particles::Particles>>,
    split: Option<Res<crate::view_render::SplitScreen>>,
    mut init: MessageReader<crate::race_effects::RaceEffectsInit>,
    mut reset: MessageReader<crate::race_effects::RaceEffectsReset>,
    mut shutdown: MessageReader<crate::race_effects::RaceEffectsShutdown>,
) {
    if init.read().count() > 0 {
        let views = if split.is_some_and(|s| s.on()) { 2 } else { 1 };
        fx.init(views);
    }
    if reset.read().count() > 0 && fx.initialised {
        fx.pending_particles = true;
    }
    if shutdown.read().count() > 0 {
        if let Some(p) = particles.as_deref_mut() {
            for inst in &mut fx.instances {
                inst.release_particles(&mut p.stateless);
            }
        }
        fx.initialised = false;
        fx.pending_particles = false;
        fx.view_vehicle = [None; 2];
    }
}

/// a beam point of car `m` (0x00129910: race cars car-relative through 0x00127600, others world through 0x00127560)
fn add_beam_point(b: &mut FuryBeam, race: bool, m: &Mat4, off: Vec3) {
    if race {
        b.add_relative(m.transform_vector3(off), m.w_axis.truncate());
    } else {
        b.add(m.transform_point3(off));
    }
}

/// the car data 0x00129910 reads (CarRender order = request order)
struct BeamCar {
    request: u64,
    /// +0 matrix (the lerped root)
    pose: Transform,
    /// the chassis sphere of the +2251 view test
    sphere: Option<Vec4>,
    /// +896 `l_brake`
    brake: Vec3,
    /// +2250
    damage: u8,
    visible: bool,
    /// +1784: 2 traffic, 1 + `0x4000` police
    traffic: bool,
    police: bool,
}

/// FuryInstance.cpp 0x00129910 — every car's two brake-light beams at amount `a2` (`k` = the view instance's +1728):
/// skipped when culled (+2251: hidden, outside this view, the camera inside it), +2304 (taken as 0), police cars
/// (type 1 with +2328 & 0x4000; the type-1 instance branch has no other car) and broken lights (+2250 bits 4/8).
/// Points are added only by vehicle 0's instance unpaused (`add` = 60 Hz steps this frame); with no step due the
/// ribbon is drawn as if one were added now (the original adds one per 60 Hz render frame).
#[allow(clippy::too_many_arguments)]
fn draw_car_beams(
    beams: &mut [FuryBeam],
    cars: &[BeamCar],
    a2: f32,
    k: f32,
    add: Option<usize>,
    last: &HashMap<u64, Transform>,
    cam: Vec3,
    frustum: Option<&bevy::camera::primitives::Frustum>,
    inside: Option<u64>,
    draw: &mut EffectDraw,
) {
    let sh = beam_shader();
    let style = &BEAM_STYLES[0];
    for (i, c) in cars.iter().enumerate() {
        let matrix = c.pose.to_matrix();
        let in_view = match (c.sphere, frustum) {
            (Some(s), Some(f)) => {
                let p = matrix.transform_point3(s.truncate());
                f.half_spaces.iter().all(|h| h.normal_d().dot(p.extend(1.0)) + s.w > 0.0)
            }
            _ => true,
        };
        if !c.visible || !in_view || inside == Some(c.request) || c.police {
            continue;
        }
        if a2 <= 0.0 {
            continue;
        }
        if 2 * i >= MAX_CAR_BEAMS {
            log::error!("assert: (i*2) < MAX_CAR_BEAMS");
            break;
        }
        // traffic: VsAIPoliceVehicle 0x00017CF0 = the traffic vehicle's +32 (active; taken as set) and its +33
        // (beams reset, flag cleared) are not surfaced
        let race = !c.traffic;
        for side in 0..2 {
            if (side == 0 && c.damage & 4 != 0) || (side == 1 && c.damage & 8 != 0) {
                continue;
            }
            let b = &mut beams[2 * i + side];
            let mut off = c.brake;
            if side != 0 {
                off.x = -off.x;
            }
            let verts = match add {
                Some(n) if n > 0 => {
                    let prev = last.get(&c.request).copied().unwrap_or(c.pose);
                    for s in 1..=n {
                        let f = s as f32 / n as f32;
                        let pose = Transform {
                            translation: prev.translation.lerp(c.pose.translation, f),
                            rotation: prev.rotation.slerp(c.pose.rotation, f),
                            scale: c.pose.scale,
                        };
                        add_beam_point(b, race, &pose.to_matrix(), off);
                    }
                    b.strip(style, cam, k * a2)
                }
                Some(_) => {
                    let mut t = b.clone();
                    add_beam_point(&mut t, race, &matrix, off);
                    t.strip(style, cam, k * a2)
                }
                None => b.strip(style, cam, k * a2),
            };
            draw.strip(&sh, &verts);
        }
    }
}

/// GameManager 0x000B9010: 0x00128E40 + 0x00129090 per human player, then 0x001293E0 per view (with that view's
/// camera). `VELOCITY_FURY_FORCE=<fade>` forces player 0's VE+88 on with that nitrous fade (verification shots).
#[allow(clippy::too_many_arguments)]
fn fury_system(
    time: Res<Time>,
    clock: Res<EffectClock>,
    mut fx: ResMut<FuryFx>,
    mut zone: Option<ResMut<ZoneFx>>,
    mut particles: Option<ResMut<crate::stateless_particles::Particles>>,
    views: crate::view_render::ViewCams,
    hv: (Option<Res<crate::track_fx::HeadlightView>>, Option<Res<crate::track_fx::HeadlightViews>>, Option<Res<crate::full_screen_effects_manager_pc_xbox::FseCameraPlanes>>),
    physics: Option<Res<velocity_physics::plugin::PhysicsWorldRes>>,
    cars: FuryCarQuery,
    mut draw: ResMut<EffectDraw>,
    mut audio: MessageWriter<FuryNosAudio>,
) {
    let fx = &mut *fx;
    if !fx.initialised {
        fx.view_vehicle = [None; 2];
        return;
    }
    let force = std::env::var("VELOCITY_FURY_FORCE").ok().and_then(|v| v.parse::<f32>().ok());
    let paused = clock.paused;
    let frames = if paused { 0.0 } else { time.delta_secs() * clock.scale * crate::car_render::XBOX_FRAME_RATE };
    let mut rows: Vec<_> = cars.iter().filter(|r| !r.0.spec.frontend).collect();
    rows.sort_by_key(|r| r.0.request);
    if fx.pending_particles && !rows.is_empty() {
        if let Some(p) = particles.as_deref_mut() {
            for i in 0..NUM_INSTANCES {
                // CarRender car i (+1800: set up) → its exhaust's l_nos / r_nos
                let nos = rows.get(i).map(|r| r.3.nos);
                fx.instances[i].init_particles(&mut p.stateless, nos);
            }
        }
        fx.pending_particles = false;
    }
    let cars_b: Vec<BeamCar> = rows
        .iter()
        .map(|r| BeamCar {
            request: r.0.request,
            pose: *r.1,
            sphere: r.2.map(|h| h.sphere),
            brake: r.3.brake,
            damage: r.4.map_or(0, |x| x.damage) | r.5.map_or(0, |x| x.0),
            visible: r.6.get(),
            traffic: r.0.spec.traffic,
            police: r.0.spec.police,
        })
        .collect();
    // the human players' vehicles (view +28): the race cars in request order
    let players: Vec<u64> = rows.iter().filter(|r| !r.0.spec.traffic && !r.0.spec.police).map(|r| r.0.request).collect();
    let list = views.list();
    let guard = physics.as_ref().map(|p| p.lock());
    let world = guard.as_deref().and_then(Option::as_ref);
    let read = |id: usize| -> FuryVehicle {
        let Some(w) = world.filter(|w| w.vehs.get(id).is_some_and(|v| v.is_some())) else { return FuryVehicle::default() };
        let s = w.slot(id);
        let row = Vec3::from(s.car.m.v3(112));
        FuryVehicle {
            speed: row.dot(Vec3::from(w.velocity(id))),
            nos: s.ve.m.b(88) != 0,
            fade: f32::from_bits(w.glob.w.get(&0x75B27C).copied().unwrap_or(0)),
        }
    };
    let mut inputs: Vec<(usize, usize, FuryVehicle, Mat4)> = Vec::new();
    for v in &list {
        if inputs.iter().any(|x| x.0 == v.player) {
            continue;
        }
        let Some(&req) = players.get(v.player) else { continue };
        let veh = req as usize;
        if veh >= NUM_INSTANCES {
            log::error!("assert: nView < NUM_FURYRENDER_INSTANCES");
            continue;
        }
        let mut inp = read(veh);
        if let (Some(f), 0) = (force, v.player) {
            inp.nos = true;
            inp.fade = f;
        }
        let m = cars_b.iter().find(|c| c.request == req).map_or(Mat4::IDENTITY, |c| c.pose.to_matrix());
        inputs.push((v.player, veh, inp, m));
    }
    drop(guard);
    fx.view_vehicle = [None; 2];
    // 0x000B9BF9 / 0x000B9C0A (race player), 0x000B9C82 / 0x000B9C93 (TFT player)
    for (p, veh, inp, m) in &inputs {
        fx.view_vehicle[(*p).min(1)] = Some(*veh);
        let z = zone.as_ref().map_or(0.0, |z| z.amount(*veh));
        let FuryFx { instances, beams, tweaks, .. } = &mut *fx;
        instances[*veh].update(inp, z, paused, tweaks, beams);
        if let Some(pt) = particles.as_deref_mut() {
            instances[*veh].update_particles(&mut pt.stateless, m);
        }
    }
    // 60 Hz beam steps due this frame
    fx.beam_clock += frames;
    let steps = fx.beam_clock.floor();
    fx.beam_clock -= steps;
    let steps = (steps as usize).min(BEAM_POINTS);
    let (view, others, planes) = hv;
    let view = view.map(|v| *v).unwrap_or_default();
    let others = others.map(|o| *o).unwrap_or_default();
    let near = planes.map_or(0.5, |p| p.near);
    let mut amount_global = zone.as_ref().map_or(0.0, |z| z.amount_global);
    let mut sounds = Vec::new();
    // 0x000B9CBE: per view (render flag 1)
    for v in &list {
        let Some(&(_, veh, inp, car_m)) = inputs.iter().find(|x| x.0 == v.player) else { continue };
        let hview = others.for_view(&view, v);
        let z = zone.as_ref().map_or(0.0, |z| z.amount(veh));
        let FuryFx { instances, beams, nos_playing, rng, last_pose, .. } = &mut *fx;
        let inst = &mut instances[veh];
        let v8 = inst.render_state(inp.nos, z, paused, nos_playing, beams, &mut amount_global, &mut sounds);
        draw.set_view(v.tag);
        let cam = v.transform.translation();
        let add = (!paused && veh == 0).then_some(steps);
        draw_car_beams(beams, &cars_b, v8, inst.nos, add, last_pose, cam, v.frustum.as_ref(), hview.inside, &mut draw);
        if inst.active && v8 > 0.0 {
            inst.set_streak_count();
            if !paused {
                // camera plane 4 (VsBaseCamera +296 + 64): the near plane, normal = the view direction
                let fwd = v.transform.forward().as_vec3();
                let plane = fwd.extend(0.0 - fwd.dot(cam + fwd * near));
                inst.update_streaks(frames, &car_m, plane, rng);
            }
            let sh = streak_shader();
            for s in inst.streak_strips(v8, car_m.w_axis.truncate()) {
                draw.strip(&sh, &s);
            }
        }
    }
    draw.set_view(None);
    if let Some(z) = zone.as_deref_mut() {
        z.amount_global = amount_global;
    }
    for s in sounds {
        audio.write(FuryNosAudio { start: s == NosSound::Start });
    }
    fx.last_pose = cars_b.iter().map(|c| (c.request, c.pose)).collect();
}

/// GameManager 0x000BA023 → 0x001285E0 → FuryInstance.cpp 0x00129580 per view (render flag 1): while the view's
/// instance is on with an amount, motion blur 0x00160BA0 flags 4 (no z) at amount · 0.0075 with the view's focus
/// (0x00160920(view +28)). The "mb_mask" shader it binds first is replaced by the blur's own, its colour
/// VISOS(min(_ftol(256a), 128), 0x80…) is never used and `byte_71FF60` (blur object +32) has no reader.
#[allow(clippy::too_many_arguments)]
fn fury_blur_system(
    mut fse: ResMut<FullScreenEffects>,
    fx: Res<FuryFx>,
    mut mb: ResMut<MotionBlur>,
    mb_input: Option<Res<MotionBlurInput>>,
    mb_second: Option<Res<MotionBlurSecondInput>>,
    mut images: ResMut<Assets<Image>>,
    views: crate::view_render::ViewCams,
    cams: Query<&Camera>,
) {
    if !fse.active || !fx.initialised {
        return;
    }
    let restore = fse.viewport();
    for v in views.list() {
        let Some(veh) = fx.view_vehicle.get(v.player).copied().flatten() else { continue };
        let inst = &fx.instances[veh];
        let a = inst.nos * inst.speed;
        if !inst.active || a <= 0.0 {
            continue;
        }
        fse.set_viewport(v.rect);
        mb.load_spec2(&mut images);
        let car = if v.player == 0 { mb_input.as_deref().copied() } else { mb_second.as_ref().and_then(|s| s.0) };
        if let (Some(m), Ok(camera)) = (car, cams.get(v.camera)) {
            if m.position != Vec3::ZERO || m.velocity != Vec3::ZERO {
                mb.update_focus(&m, &(camera.clip_from_view() * v.transform.to_matrix().inverse()));
            }
        }
        crate::motion_blur_pc_xbox::render(&mut fse, &mb, 4, a * BLUR_SCALE);
    }
    fse.set_viewport(restore);
}

pub struct FuryFxPlugin;

impl Plugin for FuryFxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FuryFx>()
            .add_message::<FuryNosAudio>()
            .add_systems(PostUpdate, (fury_messages, fury_system).chain().in_set(crate::effect_draw::EffectOrder::TrackFx).before(crate::track_fx::draw_track_fx))
            .add_systems(
                PostUpdate,
                fury_blur_system
                    .in_set(FseSet::Build)
                    .after(crate::danger_zone_render::DangerWarpSet)
                    .before(crate::lens_spray_manager::LensSpraySet),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0x00127600: a race car's points are stored relative and trail by a quarter of its travel per point
    #[test]
    fn relative_beam_trails() {
        let mut b = FuryBeam::default();
        b.add_relative(Vec3::new(0.0, 0.0, -2.0), Vec3::ZERO);
        b.add_relative(Vec3::new(0.0, 0.0, -2.0), Vec3::new(0.0, 0.0, 4.0));
        assert_eq!(b.count, 2);
        assert_eq!(b.points[0], Vec3::new(0.0, 0.0, -3.0));
        assert_eq!(b.base, Vec3::new(0.0, 0.0, 4.0));
        for _ in 0..10 {
            b.add(Vec3::ONE);
        }
        assert_eq!((b.count, b.head), (8, 4));
    }

    /// 0x00127740: 2n + 2 vertices, caps at both ends, alpha ∝ index², uv corners
    #[test]
    fn beam_strip() {
        let mut b = FuryBeam::default();
        for z in [0.0, -1.0, -2.0] {
            b.add(Vec3::new(0.0, 0.0, z));
        }
        let v = b.strip(&BEAM_STYLES[0], Vec3::new(0.0, 10.0, 0.0), 1.0);
        assert_eq!(v.len(), 8);
        // first vertex alpha 0, last 128 → ×4 saturates; red 0x30 doubled
        assert_eq!(v[0].argb, 0x0060_0000);
        assert_eq!(v[7].argb, 0xFF60_0000);
        assert_eq!((v[0].uv, v[7].uv, v[1].uv, v[2].uv), (Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0), Vec2::ZERO, Vec2::ONE));
        // sides ±0.2 across the ribbon, the caps along it
        assert!((v[1].pos - v[2].pos).length() - 0.4 < 1e-5);
        assert!((v[0].pos.z - 0.2).abs() < 1e-5 && v[0].pos.x.abs() < 1e-5);
        // the end cap leans with the view ray to the last point (0, −10, −2)
        assert!((v[7].pos.z + 2.0 + 0.2 * 2.0 / 4.16f32.sqrt()).abs() < 1e-5);
    }
}
