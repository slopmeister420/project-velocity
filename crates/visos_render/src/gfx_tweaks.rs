//! the graphics subsystem's `AddTweaks` registrations (21 functions, docs/graphics.md "Tweaks") as recorded
//! `CTweakManager` call lists, plus one value slot per tweak that mirrors the live render variable it edits.
//!
//! visos_render cannot see velocity_game's `CTweakManager`, so core replays these calls into it (`calls(origin)`
//! at the call sites of `PHASES`), keeps one `TweakCell` per slot and copies the cells to / from
//! `GraphicsTweaks.values` around the debug menu. `sync_graphics_tweaks` then pushes changed slots into the live
//! resources and pulls the live values back every frame (the original points the tweak at the variable itself).
//! Slots without a live variable in the port (debug displays, unported readers) are the variable themselves.

use bevy::prelude::*;

use crate::barrier_render::BarrierTweaks;
use crate::camera_rain::CameraRain;
use crate::car_render::CarRenderTweaks;
use crate::car_shader_manager_pc_xbox::DynamicEnvmap;
use crate::danger_zone_render::DepthOfField;
use crate::fade_render::FadeRender;
use crate::fog_render::FogRender;
use crate::fury_fx::FuryFx;
use crate::glow_pc_xbox::GlowTweaks;
use crate::motion_blur_pc_xbox::MotionBlur;
use crate::mtv_effect::MtvEffect;
use crate::projected_light_beam_pc_xbox::{BeamTweaks, ProjectedLightBeam};
use crate::race_effects::RaceEffects;
use crate::rain_surface::{FeReflection, RainSurface};
use crate::sky_render::SkySetupRes;
use crate::specular_chars::SpecularChars;
use crate::zone_render_pc_xbox::ZoneFx;

/// a tweak value (`TweakData` without strings)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GfxValue {
    Float(f32),
    Int(i32),
    Bool(bool),
}

impl GfxValue {
    pub fn f32(self) -> f32 {
        match self {
            GfxValue::Float(f) => f,
            GfxValue::Int(i) => i as f32,
            GfxValue::Bool(b) => b as i32 as f32,
        }
    }
    pub fn i32(self) -> i32 {
        match self {
            GfxValue::Float(f) => f as i32,
            GfxValue::Int(i) => i,
            GfxValue::Bool(b) => b as i32,
        }
    }
    pub fn bool(self) -> bool {
        self.i32() != 0
    }
    /// the same value in `like`'s variant
    fn conform(self, like: GfxValue) -> GfxValue {
        match like {
            GfxValue::Float(_) => GfxValue::Float(self.f32()),
            GfxValue::Int(_) => GfxValue::Int(self.i32()),
            GfxValue::Bool(_) => GfxValue::Bool(self.bool()),
        }
    }
}

impl From<f32> for GfxValue {
    fn from(v: f32) -> Self {
        GfxValue::Float(v)
    }
}
impl From<i32> for GfxValue {
    fn from(v: i32) -> Self {
        GfxValue::Int(v)
    }
}
impl From<u32> for GfxValue {
    fn from(v: u32) -> Self {
        GfxValue::Int(v as i32)
    }
}
impl From<u8> for GfxValue {
    fn from(v: u8) -> Self {
        GfxValue::Int(v as i32)
    }
}
impl From<usize> for GfxValue {
    fn from(v: usize) -> Self {
        GfxValue::Int(v as i32)
    }
}
impl From<bool> for GfxValue {
    fn from(v: bool) -> Self {
        GfxValue::Bool(v)
    }
}

/// a live variable's type from a tweak value (S32 tweaks on byte variables keep the low byte)
pub trait FromGfx {
    fn from_gfx(v: GfxValue) -> Self;
}
impl FromGfx for f32 {
    fn from_gfx(v: GfxValue) -> Self {
        v.f32()
    }
}
impl FromGfx for i32 {
    fn from_gfx(v: GfxValue) -> Self {
        v.i32()
    }
}
impl FromGfx for u32 {
    fn from_gfx(v: GfxValue) -> Self {
        v.i32() as u32
    }
}
impl FromGfx for u8 {
    fn from_gfx(v: GfxValue) -> Self {
        v.i32() as u8
    }
}
impl FromGfx for usize {
    fn from_gfx(v: GfxValue) -> Self {
        v.i32().max(0) as usize
    }
}
impl FromGfx for bool {
    fn from_gfx(v: GfxValue) -> Self {
        v.bool()
    }
}

/// one recorded `CTweakManager` call
#[derive(Clone, Debug, PartialEq)]
pub enum GfxTweakCall {
    /// createSystem (0x00086250) by dotted path below the root; the following tweaks go there
    System(&'static str),
    /// addTweak(float) 0x00085A90
    Float { name: &'static str, slot: usize, min: f32, max: f32, step: f32 },
    /// addTweak(S32) 0x00085B40 (step 0 is passed as is; the manager makes it 1)
    Int { name: &'static str, slot: usize, min: i32, max: i32, step: i32 },
    /// addTweak(enum) 0x00085BF0
    Enum { name: &'static str, slot: usize, elements: &'static str },
    /// addTweak(bool) 0x00085CA0
    Bool { name: &'static str, slot: usize },
}

impl GfxTweakCall {
    pub fn slot(&self) -> Option<usize> {
        match self {
            GfxTweakCall::System(_) => None,
            GfxTweakCall::Float { slot, .. }
            | GfxTweakCall::Int { slot, .. }
            | GfxTweakCall::Enum { slot, .. }
            | GfxTweakCall::Bool { slot, .. } => Some(*slot),
        }
    }
}

/// where the original calls a registration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GfxTweakPhase {
    /// GameManager::CommonInit 0x000B4F80 (boot; dropped by the first TweaksReset)
    CommonInit,
    /// GameManager::FrontEndInit 0x000B5510
    FrontEndInit,
    /// GameManager::InGameInit 0x000B5F80
    InGameInit,
    /// VsAIPoliceVehicle 0x00018000 (each police car VsRaceManager::CreateVehicles builds)
    PoliceVehicle,
}

/// (phase, call site in the phase function, registration origins in call order). Nested registrations follow their
/// caller: ZoneRender 0x00163770 → CarPalette 0x0010D7E0 (its own tweak, then 0x0010E3F0, then DangerZoneRender
/// 0x001214A0 → 0x00122AE0); FlareRender 0x00128100 → 0x00162B60 → tyrespray 0x00155070, then FuryFX 0x001287B0.
pub const PHASES: &[(GfxTweakPhase, u32, &[u32])] = &[
    (GfxTweakPhase::CommonInit, 0x000B503F, &[0x0015AF70]),
    (GfxTweakPhase::CommonInit, 0x000B50A2, &[0x00145F20]),
    (GfxTweakPhase::CommonInit, 0x000B51BC, &[0x0015F820]),
    (GfxTweakPhase::CommonInit, 0x000B51E5, &[0x0015A5C0]),
    (GfxTweakPhase::CommonInit, 0x000B5212, &[0x00148BA0]),
    (GfxTweakPhase::CommonInit, 0x000B5268, &[0x00123360]),
    (GfxTweakPhase::CommonInit, 0x000B5295, &[0x00137610]),
    (GfxTweakPhase::FrontEndInit, 0x000B5829, &[0x0010D7E0, 0x0010E3F0, 0x00122AE0]),
    (GfxTweakPhase::FrontEndInit, 0x000B5854, &[0x001212D0]),
    (GfxTweakPhase::FrontEndInit, 0x000B587D, &[0x00131130]),
    // 0x000B6247 = GameManager 0x000B4170, which calls it at 0x000B4BD1 among core's own registrations
    (GfxTweakPhase::InGameInit, 0x000B6247, &[0x00145F20]),
    (GfxTweakPhase::InGameInit, 0x000B6265, &[0x0015A5C0]),
    (GfxTweakPhase::InGameInit, 0x000B62C3, &[0x00161190]),
    (GfxTweakPhase::InGameInit, 0x000B657C, &[0x001411C0]),
    (GfxTweakPhase::InGameInit, 0x000B65A5, &[0x00135120]),
    (GfxTweakPhase::InGameInit, 0x000B65C3, &[0x001212D0]),
    (GfxTweakPhase::InGameInit, 0x000B65DF, &[0x0011E080]),
    (GfxTweakPhase::InGameInit, 0x000B660D, &[0x0013D320]),
    (GfxTweakPhase::InGameInit, 0x000B6D2E, &[0x00107560]),
    (GfxTweakPhase::InGameInit, 0x000B6D48, &[0x00107560]),
    (GfxTweakPhase::InGameInit, 0x000B6E52, &[0x0010D7E0, 0x0010E3F0, 0x00122AE0]),
    (GfxTweakPhase::InGameInit, 0x000B6E85, &[0x00155070, 0x001287B0]),
    (GfxTweakPhase::PoliceVehicle, 0x0001808D, &[0x0006B8B0]),
];

/// accessors of the live variable behind a slot (`get` None = no such resource now)
#[derive(Clone, Copy)]
pub struct Live {
    pub get: fn(&World) -> Option<GfxValue>,
    pub set: fn(&mut World, GfxValue),
}

/// one registration function
#[derive(Clone, Debug)]
pub struct GfxTweakFn {
    pub origin: u32,
    pub calls: Vec<GfxTweakCall>,
}

/// every graphics registration and the value slots their tweaks point at
#[derive(Resource)]
pub struct GraphicsTweaks {
    pub fns: Vec<GfxTweakFn>,
    /// the tweaked variables (core's cells copy in / out of these)
    pub values: Vec<GfxValue>,
    /// the values last synced with the live variables
    applied: Vec<GfxValue>,
    live: Vec<Option<Live>>,
}

impl GraphicsTweaks {
    /// the recorded calls of the registration at `origin`
    pub fn calls(&self, origin: u32) -> Option<&[GfxTweakCall]> {
        self.fns.iter().find(|f| f.origin == origin).map(|f| &f.calls[..])
    }

    /// the registration origins of one phase, in call order
    pub fn phase_origins(phase: GfxTweakPhase) -> Vec<u32> {
        PHASES.iter().filter(|p| p.0 == phase).flat_map(|p| p.2.iter().copied()).collect()
    }

    /// whether a slot reaches a live render variable
    pub fn is_live(&self, slot: usize) -> bool {
        self.live[slot].is_some()
    }
}

impl Default for GraphicsTweaks {
    fn default() -> Self {
        let mut r = Reg { t: GraphicsTweaks { fns: Vec::new(), values: Vec::new(), applied: Vec::new(), live: Vec::new() }, calls: Vec::new() };
        car_damage_6b8b0(&mut r);
        r.end(0x0006B8B0);
        barrier_render_107560(&mut r);
        r.end(0x00107560);
        car_palette_10d7e0(&mut r);
        r.end(0x0010D7E0);
        car_palette_10e3f0(&mut r);
        r.end(0x0010E3F0);
        cats_eye_render_11e080(&mut r);
        r.end(0x0011E080);
        danger_zone_render_1212d0(&mut r);
        r.end(0x001212D0);
        danger_zone_render_122ae0(&mut r);
        r.end(0x00122AE0);
        danger_zone_render_123360(&mut r);
        r.end(0x00123360);
        fury_fx_1287b0(&mut r);
        r.end(0x001287B0);
        light_streak_131130(&mut r);
        r.end(0x00131130);
        light_streak_135120(&mut r);
        r.end(0x00135120);
        rain_surface_137610(&mut r);
        r.end(0x00137610);
        sky_render_add_tweaks(&mut r);
        r.end(0x0013D320);
        slipstream_render_1411c0(&mut r);
        r.end(0x001411C0);
        specular_chars_145f20(&mut r);
        r.end(0x00145F20);
        start_line_effects_148ba0(&mut r);
        r.end(0x00148BA0);
        tyrespray_155070(&mut r);
        r.end(0x00155070);
        fog_render_add_tweaks(&mut r);
        r.end(0x0015A5C0);
        car_shader_manager_15af70(&mut r);
        r.end(0x0015AF70);
        game_shader_program_manager_15f820(&mut r);
        r.end(0x0015F820);
        game_shader_program_manager_161190(&mut r);
        r.end(0x00161190);
        let mut t = r.t;
        t.applied = t.values.clone();
        t
    }
}

/// registration recorder
struct Reg {
    t: GraphicsTweaks,
    calls: Vec<GfxTweakCall>,
}

impl Reg {
    fn end(&mut self, origin: u32) {
        let calls = std::mem::take(&mut self.calls);
        self.t.fns.push(GfxTweakFn { origin, calls });
    }
    fn slot(&mut self, v: GfxValue, live: Option<Live>) -> usize {
        self.t.values.push(v);
        self.t.live.push(live);
        self.t.values.len() - 1
    }
    fn sys(&mut self, path: &'static str) {
        self.calls.push(GfxTweakCall::System(path));
    }
    fn f(&mut self, name: &'static str, min: f32, max: f32, step: f32, init: f32, live: Option<Live>) {
        let slot = self.slot(GfxValue::Float(init), live);
        self.calls.push(GfxTweakCall::Float { name, slot, min, max, step });
    }
    fn i(&mut self, name: &'static str, min: i32, max: i32, step: i32, init: i32, live: Option<Live>) {
        let slot = self.slot(GfxValue::Int(init), live);
        self.calls.push(GfxTweakCall::Int { name, slot, min, max, step });
    }
    fn e(&mut self, name: &'static str, elements: &'static str, init: i32, live: Option<Live>) {
        let slot = self.slot(GfxValue::Int(init), live);
        self.calls.push(GfxTweakCall::Enum { name, slot, elements });
    }
    fn b(&mut self, name: &'static str, init: bool, live: Option<Live>) {
        let slot = self.slot(GfxValue::Bool(init), live);
        self.calls.push(GfxTweakCall::Bool { name, slot });
    }
}

/// a live field of a resource: `res!(R, |r| r.field)`
macro_rules! res {
    ($R:ty, |$r:ident| $place:expr) => {
        Some(Live {
            get: |w| w.get_resource::<$R>().map(|$r| GfxValue::from($place)),
            set: |w, v| {
                if let Some(mut $r) = w.get_resource_mut::<$R>() {
                    $place = FromGfx::from_gfx(v);
                }
            },
        })
    };
}

/// a sky cloud layer field (the tweaks exist for four layers; a layer the sky does not have is the slot itself)
macro_rules! cloud {
    ($i:expr, $field:ident) => {
        Some(Live {
            get: |w| w.get_resource::<SkySetupRes>().and_then(|r| r.0.clouds.get($i).map(|c| GfxValue::from(c.$field))),
            set: |w, v| {
                if let Some(mut r) = w.get_resource_mut::<SkySetupRes>() {
                    if let Some(c) = r.0.clouds.get_mut($i) {
                        c.$field = FromGfx::from_gfx(v);
                    }
                }
            },
        })
    };
}

/// `x * 0.02f` constants of the sky tweaks (SkyRender's cfg-to-world scale; double · float, passed as float)
fn sky(x: f64) -> f32 {
    (x * 0.02f32 as f64) as f32
}

const ON: bool = true;
const OFF: bool = false;

/// CarDamage.cpp 0x0006B8B0
fn car_damage_6b8b0(r: &mut Reg) {
    // CarDamage's debug draws (physics' car damage reads "Single Bone Damage"; nothing here is drawn by the port)
    r.sys("Graphics.Damage");
    r.b("Display Bones", OFF, None);
    r.i("Single Bone Damage", -1, 8, 1, -1, None);
    r.i("Show Impacts", 0, 56, 0, 0, None);
    r.i("Show Impact", -1, 55, 1, -1, None);
    r.b("Show Buckle", OFF, None);
    r.b("Show Hinges", OFF, None);
}

/// BarrierRender.cpp 0x00107560
fn barrier_render_107560(r: &mut Reg) {
    r.sys("Graphics.Spline Barriers");
    r.f("frequency", 1.0, 12.0, 0.1, 6.0, res!(BarrierTweaks, |t| t.frequency));
    r.f("speed", 0.0, 2.0, 0.05, f32::from_bits(0x3E99_999A), res!(BarrierTweaks, |t| t.speed));
    r.f("UV wobble", 0.0, 2048.0, 32.0, 256.0, res!(BarrierTweaks, |t| t.uv_wobble));
    r.f("duration", 0.5, 4.0, 0.1, 1.0, res!(BarrierTweaks, |t| t.duration));
    r.f("red", 0.0, 255.0, 1.0, 48.0, res!(BarrierTweaks, |t| t.red.x));
    r.f("green", 0.0, 255.0, 1.0, 0.0, res!(BarrierTweaks, |t| t.red.y));
    r.f("blue", 0.0, 255.0, 1.0, 0.0, res!(BarrierTweaks, |t| t.red.z));
}

/// CarPalette.cpp 0x0010D7E0
fn car_palette_10d7e0(r: &mut Reg) {
    // read by CarRender 0x001134A0 (wheel penetration fix), not modelled
    r.sys("Graphics.WheelPenetration");
    r.b("Fix Wheels", OFF, None);
}

/// CarPalette.cpp 0x0010E3F0 — mostly mirrors applied by the debug override CarRender 0x0011AD30 (not ported)
fn car_palette_10e3f0(r: &mut Reg) {
    r.sys("Graphics.Paint");
    r.i("Paint Type 0=Gloss,1=Pearl,2=Metallic", 0, 2, 1, 0, None);
    r.sys("Graphics.Paint.BaseColour");
    r.i("Red", 0, 255, 2, 0, None);
    r.i("Green", 0, 255, 2, 0, None);
    r.i("Blue", 0, 255, 2, 0, None);
    r.sys("Graphics.Paint.HilightColour");
    r.i("Red", 0, 255, 2, 0, None);
    r.i("Green", 0, 255, 2, 0, None);
    r.i("Blue", 0, 255, 2, 0, None);
    r.sys("Frontend");
    r.b("Black Magic", OFF, None);
    r.sys("Frontend.TwoTone");
    r.b("enable", OFF, None);
    r.i("split style", 0, 7, 1, 0, None);
    r.sys("Graphics.Envmap");
    r.f("On/Off", 0.0, 1.0, 1.0, 0.0, None);
    r.sys("Graphics.Specular");
    r.f("On/Off", 0.0, 1.0, 1.0, 0.0, None);
    r.f("Brightness", 0.0, 2.0, 0.01, 0.0, None);
    r.f("Size", 0.5, 6.0, 0.05, 2.5, None);
    r.f("Bloom", 0.0, 1.0, 0.05, 0.25, None);
    r.sys("Graphics.Lights");
    r.f("Ambient - Red", 0.0, 2.0, 0.025, 0.0, None);
    r.f("Ambient - Green", 0.0, 2.0, 0.025, 0.0, None);
    r.f("Ambient - Blue", 0.0, 2.0, 0.025, 0.0, None);
    r.f("Point Light Intensity", 0.0, 4.0, 0.1, 0.25, res!(CarRenderTweaks, |t| t.point_light_intensity));
    r.f("Metal Paint Brightness", 0.0, 10.0, 0.05, f32::from_bits(0x3EB3_3333), None);
    r.f("Pearl Paint Brightness", 0.0, 2.0, 0.025, f32::from_bits(0x3F4C_CCCD), None);
    r.f("Diffuse - Red", 0.0, 1.0, 0.025, 0.0, None);
    r.f("Diffuse - Green", 0.0, 1.0, 0.025, 0.0, None);
    r.f("Diffuse - Blue", 0.0, 1.0, 0.025, 0.0, None);
    // sqrt of the car LOD thresholds (400 / 1600)
    r.sys("Graphics.LOD");
    r.f("Medium LOD Front Range", 0.0, 200.0, 1.0, 20.0, None);
    r.f("Medium LOD Side Range", 0.0, 200.0, 1.0, 20.0, None);
    r.f("Low LOD Front Range", 0.0, 200.0, 1.0, 40.0, None);
    r.f("Low LOD Side Range", 0.0, 200.0, 1.0, 40.0, None);
    r.b("Force Car LOD", OFF, res!(CarRenderTweaks, |t| t.force_lod));
    r.i("Force LOD Level", 0, 2, 1, 0, res!(CarRenderTweaks, |t| t.force_lod_level));
    r.sys("Graphics.AlloyColour");
    r.b("Coloured Alloys", OFF, None);
    r.f("base Red", 0.0, 1.0, 0.01, 0.0, None);
    r.f("base Green", 0.0, 1.0, 0.01, 0.0, None);
    r.f("base Blue", 0.0, 1.0, 0.01, 0.0, None);
    r.f("spec Red", 0.0, 1.0, 0.01, 0.0, None);
    r.f("spec Green", 0.0, 1.0, 0.01, 0.0, None);
    r.f("spec Blue", 0.0, 1.0, 0.01, 0.0, None);
    r.f("bloom", 0.0, 1.0, 0.01, f32::from_bits(0x3DCC_CCCD), None);
    r.f("spec scale", 0.25, 1.5, 0.02, 1.0, None);
}

/// CatsEyeRender.cpp 0x0011E080 (debug displays, not drawn)
fn cats_eye_render_11e080(r: &mut Reg) {
    r.sys("Graphics.Display");
    r.b("Display material names", OFF, None);
    r.b("Collision Mesh Render", OFF, None);
    r.b("Collision Mesh Textured", OFF, None);
}

/// DangerZoneRender.cpp 0x001212D0
fn danger_zone_render_1212d0(r: &mut Reg) {
    r.sys("Graphics.DOF");
    r.b("Enabled", OFF, res!(DepthOfField, |d| d.enabled));
    r.b("Debug Display", OFF, res!(DepthOfField, |d| d.debug));
    r.f("DOF Factor", 0.0, 2.0, 0.01, 0.0, res!(DepthOfField, |d| d.dof_factor));
    r.f("Blur Factor", 0.0, 2.0, 0.1, 0.0, res!(DepthOfField, |d| d.blur_factor));
    r.i("Max Blur", 0, 32, 1, 0, res!(DepthOfField, |d| d.max_blur));
}

/// DangerZoneRender.cpp 0x00122AE0
fn danger_zone_render_122ae0(r: &mut Reg) {
    r.sys("Graphics.Dynamic Envmap");
    r.b("enabled", OFF, res!(DynamicEnvmap, |e| e.enabled));
    r.b("display", OFF, res!(DynamicEnvmap, |e| e.tweaks.display));
    r.b("hi-res", ON, res!(DynamicEnvmap, |e| e.tweaks.hi_res));
    r.b("hi-contrast", OFF, None);
    r.b("greyscale", OFF, None);
    r.i("blur", 0, 2, 1, 1, res!(DynamicEnvmap, |e| e.tweaks.blur));
    r.f("top bright", 0.0, 1.0, 0.01, 0.75, res!(DynamicEnvmap, |e| e.tweaks.top_bright));
    r.f("mid bright", 0.0, 1.0, 0.01, 0.5, res!(DynamicEnvmap, |e| e.tweaks.mid_bright));
    r.f("bot bright", 0.0, 1.0, 0.01, f32::from_bits(0x3DCC_CCCD), res!(DynamicEnvmap, |e| e.tweaks.bot_bright));
    r.f("global bright", 0.0, 2.0, 0.01, 1.0, res!(DynamicEnvmap, |e| e.tweaks.global_bright));
    r.i("subtract", 0, 128, 1, 0x20, res!(DynamicEnvmap, |e| e.tweaks.subtract));
    // S32 tweak on the byte `byte_686950`
    r.i("fresnel", 0, 128, 1, 0x70, res!(DynamicEnvmap, |e| e.tweaks.fresnel));
    r.f("desaturate", 0.0, 1.0, 0.01, 0.0, None);
    r.sys("Graphics.Dynamic Envmap.Specular");
    r.f("size", 0.0, 0.5, 0.01, 0.25, res!(DynamicEnvmap, |e| e.tweaks.spec_size));
    r.f("bright", 0.0, 1.0, 0.01, 0.75, res!(DynamicEnvmap, |e| e.tweaks.spec_bright));
    r.f("bloom", 0.0, 2.0, 0.01, f32::from_bits(0x3F99_999A), res!(DynamicEnvmap, |e| e.tweaks.spec_bloom));
    r.f("fresnel", 0.0, 1.0, 0.02, f32::from_bits(0x3F4C_CCCD), res!(DynamicEnvmap, |e| e.tweaks.spec_fresnel));
    r.b("extra bloom fresnel", ON, res!(DynamicEnvmap, |e| e.tweaks.extra_bloom_fresnel));
}

/// DangerZoneRender.cpp 0x00123360
fn danger_zone_render_123360(r: &mut Reg) {
    r.sys("Graphics.Fullscreen Blur");
    r.f("Blur", 0.0, 1.0, 0.01, 0.0, res!(FadeRender, |f| f.blur));
}

/// FuryFX.cpp 0x001287B0
fn fury_fx_1287b0(r: &mut Reg) {
    r.sys("Graphics.NOS FX");
    r.f("Fov Change", 0.0, 35.0, 1.0, 20.0, res!(FuryFx, |f| f.tweaks.fov_change));
    r.f("Max Nitro Speed", 0.0, 100.0, 1.0, 65.0, res!(FuryFx, |f| f.tweaks.max_nitro_speed));
}

/// LightStreak.cpp 0x00131130 — the front end's MTV effect (`mtv_effect`; "Enabled" = the object's +0 running byte)
fn light_streak_131130(r: &mut Reg) {
    r.sys("Frontend.MTV Effect");
    r.b("Enabled", OFF, res!(MtvEffect, |m| m.enabled));
    r.f("Line Freq", 0.0, 10.0, 0.1, 100.0, res!(MtvEffect, |m| m.tweaks.line_freq));
    r.f("Line Alpha", 0.0, 1.0, 0.05, 0.5, res!(MtvEffect, |m| m.tweaks.line_alpha));
    r.f("Line Speed", 0.0, 1024.0, 32.0, 512.0, res!(MtvEffect, |m| m.tweaks.line_speed));
    r.f("Line Width", 0.0, 5.0, 1.0, 2.0, res!(MtvEffect, |m| m.tweaks.line_width));
    r.f("Image Freq", 0.0, 10.0, 0.1, 100.0, res!(MtvEffect, |m| m.tweaks.image_freq));
    r.f("Image Alpha", 0.0, 1.0, 0.05, 1.0, res!(MtvEffect, |m| m.tweaks.image_alpha));
    r.f("Image Speed", 0.0, 30.0, 1.0, 15.0, res!(MtvEffect, |m| m.tweaks.image_speed));
    r.f("Tint Freq", 0.0, 10.0, 0.1, 100.0, res!(MtvEffect, |m| m.tweaks.tint_freq));
    r.f("Tint Intensity", 0.0, 1.0, 0.05, 0.5, res!(MtvEffect, |m| m.tweaks.tint_intensity));
    r.f("Desat Intensity", 0.0, 1.0, 0.05, 0.5, res!(MtvEffect, |m| m.tweaks.desat_intensity));
    r.f("Tint Time in", 0.0, 2.0, 0.05, f32::from_bits(0x3E4C_CCCD), res!(MtvEffect, |m| m.tweaks.tint_in));
    r.f("Tint Time out", 0.0, 2.0, 0.05, 0.5, res!(MtvEffect, |m| m.tweaks.tint_out));
}

/// LightStreak.cpp 0x00135120
fn light_streak_135120(r: &mut Reg) {
    r.sys("Graphics.Rain");
    r.b("Rain Enabled", OFF, res!(CameraRain, |c| c.rain));
    r.f("Rain Thickness", 0.0, 5.0, 0.05, 2.0, res!(CameraRain, |c| c.tweaks.thickness));
    r.f("Rain Alpha", 0.0, 1.0, 0.01, f32::from_bits(0x3DCC_CCCD), res!(CameraRain, |c| c.tweaks.alpha));
    r.f("Rain Length", 0.0, 1.0, 0.01, 0.25, res!(CameraRain, |c| c.tweaks.length));
    r.f("Rain Speed", 0.0, 4.0, 0.1, 1.0, res!(CameraRain, |c| c.tweaks.speed));
    r.f("Rain spawn rate", 0.0, 1000.0, 10.0, 250.0, res!(CameraRain, |c| c.tweaks.spawn_rate));
    r.f("Rain crisscrossing", 0.0, 50.0, 0.5, 16.0, res!(CameraRain, |c| c.tweaks.crisscross));
    r.b("Splashes enabled", OFF, res!(CameraRain, |c| c.splash));
    r.f("Splash Size (far)", 0.0, 256.0, 1.0, 46.0, res!(CameraRain, |c| c.tweaks.size_far));
    r.f("Splash Size (close)", 0.0, 256.0, 1.0, 70.0, res!(CameraRain, |c| c.tweaks.size_close));
    r.f("Splash width factor", 0.0, 16.0, 0.01, 2.0, res!(CameraRain, |c| c.tweaks.width_factor));
    r.f("Splash height factor", 0.0, 16.0, 0.01, 0.5, res!(CameraRain, |c| c.tweaks.height_factor));
    r.f("Splash start forward extra", -5.0, 5.0, 0.1, -1.5, res!(CameraRain, |c| c.tweaks.forward_extra));
    r.f("Splash close threshold", 0.0, 50.0, 1.0, 6.0, res!(CameraRain, |c| c.tweaks.close));
    r.f("Splash Y bias", -2.0, 2.0, 0.05, 0.25, res!(CameraRain, |c| c.tweaks.y_bias));
    r.i("Number of splashes", 0, 1024, 32, 1024, res!(CameraRain, |c| c.tweaks.count));
    r.b("Splash skewed distribution", ON, res!(CameraRain, |c| c.tweaks.skewed));
    r.f("Splash decay", 0.0, 1.0, 0.01, f32::from_bits(0x3DCC_CCCD), res!(CameraRain, |c| c.tweaks.decay));
    r.e("Splash_surface_type", "Horiz = 0, Plane = 1, Raceline = 2, Splines = 3, Rays = 4", 4, res!(CameraRain, |c| c.tweaks.surface));
    r.f("Splash Alpha", 0.0, 1.0, 0.01, f32::from_bits(0x3D8F_5C29), res!(CameraRain, |c| c.tweaks.splash_alpha));
}

/// RainSurface.cpp 0x00137610
fn rain_surface_137610(r: &mut Reg) {
    r.sys("Graphics.Reflection");
    r.b("On/off", OFF, res!(RainSurface, |s| s.on));
    r.i("Brightnes", 0, 255, 4, 128, res!(RainSurface, |s| s.brightness));
    r.i("BlurStep", 0, 32, 4, 0, res!(RainSurface, |s| s.blur_step));
    // `unk_686FB0` (1): read by nothing in this build
    r.b("Additive", ON, None);
    r.i("Reflection Image Width", 6, 8, 1, 0, res!(RainSurface, |s| s.image_width));
    r.i("Reflection Image Height", 5, 7, 1, 0, res!(RainSurface, |s| s.image_height));
    r.sys("Frontend.Reflection");
    r.b("On/off", OFF, res!(FeReflection, |f| f.on));
    r.i("Blur", 0, 32, 4, 0, res!(FeReflection, |f| f.blur));
    r.f("Intensity", 0.0, 255.0, 8.0, 0.0, res!(FeReflection, |f| f.intensity));
}

/// SkyRender.cpp 0x0013D320 `CSkyRender::AddTweaks`
fn sky_render_add_tweaks(r: &mut Reg) {
    r.sys("Graphics.Sky");
    r.f("Sky Height Offset", sky(-250.0), sky(250.0), sky(5.0), 0.0, res!(SkySetupRes, |s| s.0.height_offset));
    r.f("Sky FOV", 45.0, 90.0, 1.0, 70.0, res!(SkySetupRes, |s| s.0.fov));
    r.f("Sky Height Scale", sky(5.0), sky(20.0), sky(0.25), 0.0, res!(SkySetupRes, |s| s.0.cylinder_scale.y));
    r.f("Sky Cylinder Brightness", 0.0, 1.0, 0.05, 0.5, res!(SkySetupRes, |s| s.0.cylinder_brightness));
    r.f("Global Sky Brightness", 0.0, 1.0, 0.05, 1.0, res!(SkySetupRes, |s| s.0.global_brightness));
    // `0.1f * 0.02f`: float · float
    r.f("Buildings Scale", 0.0, sky(10.0), 0.1f32 * 0.02f32, 0.0, res!(SkySetupRes, |s| s.0.buildings_scale.y));
    r.f("Buildings Brightness", 0.0, 1.0, 0.01, f32::from_bits(0x3E19_999A), res!(SkySetupRes, |s| s.0.buildings_brightness));
    // +120 = the cfg's layer count; the port builds the layers at load, so an edit is not applied
    r.i(
        "Num Cloud Layers",
        0,
        4,
        1,
        0,
        Some(Live { get: |w| w.get_resource::<SkySetupRes>().map(|s| GfxValue::from(s.0.clouds.len())), set: |_, _| {} }),
    );
    // cfg save / load (the game dir is read-only): not ported
    r.b("Save Config File", OFF, None);
    r.b("Load Config File", OFF, None);
    r.sys("Graphics.Sky.Background Tint");
    r.f("Red", 0.0, 1.0, 0.01, 0.0, res!(SkySetupRes, |s| s.0.background_top.x));
    r.f("Green", 0.0, 1.0, 0.01, 0.0, res!(SkySetupRes, |s| s.0.background_top.y));
    r.f("Blue", 0.0, 1.0, 0.01, 0.0, res!(SkySetupRes, |s| s.0.background_top.z));
    r.sys("Graphics.Sky.Sun");
    r.f("Sun Width", 250.0, 4000.0, 50.0, 2250.0, res!(SkySetupRes, |s| s.0.sun_width));
    r.f("Sun Height", 250.0, 3000.0, 50.0, 1120.0, res!(SkySetupRes, |s| s.0.sun_height));
    r.f("Sun Alpha", 0.0, 1.0, 0.01, 0.5, res!(SkySetupRes, |s| s.0.sun_alpha));
    r.f("Sun Rotation", -180.0, 180.0, 5.0, -90.0, res!(SkySetupRes, |s| s.0.sun_rotation));
    r.f("Sun Elevation", -30.0, 90.0, 1.0, 0.0, res!(SkySetupRes, |s| s.0.sun_elevation));
    r.f("Sun RED", 0.0, 1.0, 0.01, 0.0, res!(SkySetupRes, |s| s.0.sun_rgb.x));
    r.f("Sun GREEN", 0.0, 1.0, 0.01, 0.0, res!(SkySetupRes, |s| s.0.sun_rgb.y));
    r.f("Sun BLUE", 0.0, 1.0, 0.01, 0.0, res!(SkySetupRes, |s| s.0.sun_rgb.z));
    r.sys("Graphics.Sky.Moon");
    r.f("Moon Size", 0.0, 1000.0, 10.0, 250.0, res!(SkySetupRes, |s| s.0.moon_size));
    r.f("Moon Rotation", -180.0, 180.0, 5.0, 30.0, res!(SkySetupRes, |s| s.0.moon_rotation));
    r.f("Moon Elevation", 0.0, 90.0, 1.0, 30.0, res!(SkySetupRes, |s| s.0.moon_elevation));
    r.f("R", 0.0, 1.0, 0.01, 0.0, res!(SkySetupRes, |s| s.0.moon_rgba.x));
    r.f("G", 0.0, 1.0, 0.01, 0.0, res!(SkySetupRes, |s| s.0.moon_rgba.y));
    r.f("B", 0.0, 1.0, 0.01, 0.0, res!(SkySetupRes, |s| s.0.moon_rgba.z));
    r.f("Alpha", 0.0, 1.0, 0.01, 0.0, res!(SkySetupRes, |s| s.0.moon_rgba.w));
    // layer 1's height range is in world units (· 0.02), layers 2-4 in cfg units (original inconsistency)
    r.sys("Graphics.Sky.Cloud Layer 1");
    r.f("Cloud Height", 0.0, sky(500.0), sky(5.0), 0.0, cloud!(0, height));
    r.f("Speed", -8.0, 8.0, 0.1, -0.25, cloud!(0, speed));
    r.f("Tiling", 1.0, 8.0, 0.1, 1.0, cloud!(0, tiling));
    r.f("Cloud Brightness", 0.0, 1.0, 0.05, 0.5, cloud!(0, brightness));
    r.f("Cloud Alpha", 0.0, 1.0, 0.05, 1.0, cloud!(0, alpha));
    r.i("Alpha Test", 0, 128, 1, 128, cloud!(0, alpha_test));
    r.sys("Graphics.Sky.Cloud Layer 2");
    r.f("Cloud Height", 0.0, 250.0, 5.0, 0.0, cloud!(1, height));
    r.f("Speed", -8.0, 8.0, 0.1, -1.0, cloud!(1, speed));
    r.f("Tiling", 1.0, 8.0, 0.1, 1.0, cloud!(1, tiling));
    r.f("Cloud Brightness", 0.0, 1.0, 0.05, 1.0, cloud!(1, brightness));
    r.f("Cloud Alpha", 0.0, 1.0, 0.05, 1.0, cloud!(1, alpha));
    r.i("Alpha Test", 0, 128, 1, 80, cloud!(1, alpha_test));
    r.sys("Graphics.Sky.Cloud Layer 3");
    r.f("Cloud Height", 0.0, 250.0, 5.0, 0.0, cloud!(2, height));
    r.f("Speed", -8.0, 8.0, 0.1, -1.0, cloud!(2, speed));
    r.f("Tiling", 1.0, 8.0, 0.1, 0.0, cloud!(2, tiling));
    r.f("Cloud Brightness", 0.0, 1.0, 0.05, 1.0, cloud!(2, brightness));
    r.f("Cloud Alpha", 0.0, 1.0, 0.05, 1.0, cloud!(2, alpha));
    r.i("Alpha Test", 0, 128, 1, 128, cloud!(2, alpha_test));
    r.sys("Graphics.Sky.Cloud Layer 4");
    r.f("Cloud Height", 0.0, 250.0, 5.0, 0.0, cloud!(3, height));
    r.f("Speed", -8.0, 8.0, 0.1, -1.0, cloud!(3, speed));
    r.f("Tiling", 1.0, 8.0, 0.1, 0.0, cloud!(3, tiling));
    r.f("Cloud Brightness", 0.0, 1.0, 0.05, 1.0, cloud!(3, brightness));
    r.f("Cloud Alpha", 0.0, 1.0, 0.05, 1.0, cloud!(3, alpha));
    r.i("Alpha Test", 0, 128, 1, 128, cloud!(3, alpha_test));
}

/// SlipstreamRender.cpp 0x001411C0
fn slipstream_render_1411c0(r: &mut Reg) {
    r.sys("Graphics.Slipstream");
    // S32 tweak on the byte `byte_6870C0`
    r.i("trail length", 2, 16, 1, 16, res!(RaceEffects, |f| f.slipstream.globals.trail_length));
    r.i("Test Vehicle", -1, 10, 1, -1, res!(RaceEffects, |f| f.slipstream.globals.test_vehicle));
    r.f("Distortion Speed", -4.0, 4.0, 0.05, -1.0, res!(RaceEffects, |f| f.slipstream.distortion_speed));
    r.f("Distortion Amplitude", 0.0, 1.0, 0.01, f32::from_bits(0x3E4C_CCCD), res!(RaceEffects, |f| f.slipstream.distortion_amplitude));
    r.f("Distortion Wave Step", 0.0, 5.0, 0.05, 1.0, res!(RaceEffects, |f| f.slipstream.distortion_step));
    r.f("Fullscreen Blur Min", 0.0, 1.0, 0.05, 0.25, res!(MotionBlur, |m| m.tweaks.slip_min));
    r.f("Fullscreen Blur Max", 0.0, 1.0, 0.05, 0.75, res!(MotionBlur, |m| m.tweaks.slip_max));
    r.sys("Graphics.Slipstream.Side Trails");
    r.f("Trail Width", 0.0, 1.0, 0.05, 0.5, res!(RaceEffects, |f| f.slipstream.width[0]));
    r.i("Trail Brightness", 0, 128, 1, 9, res!(RaceEffects, |f| f.slipstream.globals.brightness[0]));
    r.f("Distortion Alpha", 0.0, 252.0, 4.0, 188.0, res!(RaceEffects, |f| f.slipstream.globals.alpha[0]));
    r.sys("Graphics.Slipstream.Top Trails");
    r.f("Trail Width", 0.0, 1.0, 0.05, 0.25, res!(RaceEffects, |f| f.slipstream.width[1]));
    r.i("Trail Brightness", 0, 128, 1, 20, res!(RaceEffects, |f| f.slipstream.globals.brightness[1]));
    r.f("Distortion Alpha", 0.0, 252.0, 4.0, 100.0, res!(RaceEffects, |f| f.slipstream.globals.alpha[1]));
    r.f("Rise Height", 0.0, 4.0, 0.05, f32::from_bits(0x3FE0_0000), res!(RaceEffects, |f| f.slipstream.rise_height));
    r.f("Start Height", -1.0, 1.0, 0.025, f32::from_bits(0xBECC_CCCD), res!(RaceEffects, |f| f.slipstream.start_height));
}

/// SpecularChars.cpp 0x00145F20 — only "Enabled" has a reader in the port (the editing / display / file tools are
/// debug, see "Character specular")
fn specular_chars_145f20(r: &mut Reg) {
    r.sys("Frontend.CharacterSpecular");
    r.b("Enabled", ON, res!(SpecularChars, |s| s.enabled));
    r.b("Display", OFF, None);
    r.b("All-Skin", OFF, None);
    r.b("Show glossmap", OFF, None);
    r.b("Character Cam", OFF, None);
    r.sys("Frontend.CharacterSpecular.Edit Materials");
    r.i("Material", 0, 5, 1, 1, None);
    r.f("Power", 0.0, 4.0, 0.05, 1.0, None);
    r.f("Intensity", 0.0, 2.0, 0.05, 1.0, None);
    r.i("Texture", -1, 11, 1, 0, None);
    r.f("UVStretch", 0.0, 1.0, 0.02, 0.0, None);
    r.sys("Frontend.CharacterSpecular.Edit Light");
    r.f("Light Rotation", 0.0, 360.0, 4.0, 0.0, None);
    r.f("Light Elevation", -90.0, 90.0, 1.0, 45.0, None);
    r.f("Red", 0.0, 1.0, 0.01, 0.0, None);
    r.f("Green", 0.0, 1.0, 0.01, 0.0, None);
    r.f("Blue", 0.0, 1.0, 0.01, 0.0, None);
    r.f("Light Intensity", 0.0, 4.0, 0.05, 0.0, None);
    r.f("Bloom", 0.0, 1.0, 0.02, f32::from_bits(0x3DCC_CCCD), None);
    r.sys("Frontend.CharacterSpecular.Diffuse Lights");
    r.f("AmbientR", 0.0, 1.0, 0.01, 0.0, None);
    r.f("AmbientG", 0.0, 1.0, 0.01, 0.0, None);
    r.f("AmbientB", 0.0, 1.0, 0.01, 0.0, None);
    for sys in [
        "Frontend.CharacterSpecular.Diffuse Lights.Diffuse1",
        "Frontend.CharacterSpecular.Diffuse Lights.Diffuse2",
        "Frontend.CharacterSpecular.Diffuse Lights.Diffuse3",
    ] {
        r.sys(sys);
        r.f("R", 0.0, 1.0, 0.01, 0.0, None);
        r.f("G", 0.0, 1.0, 0.01, 0.0, None);
        r.f("B", 0.0, 1.0, 0.01, 0.0, None);
        r.f("Rotation", 0.0, 360.0, 4.0, 0.0, None);
        r.f("Elevation", -90.0, 90.0, 1.0, 0.0, None);
    }
    r.sys("Frontend.CharacterSpecular.Load File");
    r.b("Load", OFF, None);
    r.sys("Frontend.CharacterSpecular.Save File");
    r.b("Save", OFF, None);
}

/// StartLineEffects.cpp 0x00148BA0 — the detail road / stretch is not ported ("Detail Road" is set by the start line
/// init 0x00147DC0 and read by GameManager 0x000B9010)
fn start_line_effects_148ba0(r: &mut Reg) {
    r.sys("Graphics.TrackFX");
    r.b("Detail Road", OFF, None);
    r.b("Detail Stretch", ON, None);
    r.f("Stretch start speed", 0.0, 150.0, 1.0, 30.0, None);
    r.f("Stretch Scale", 0.0, 1.0, 0.01, f32::from_bits(0x3E4C_CCCD), None);
}

/// the ZoneData fields in tweak order (index into `ZoneData`)
const ZONE_FIELDS: [(&str, f32, f32, f32, usize); 12] = [
    ("WaveSpeed", -2.0, 2.0, 0.01, 0),
    ("WaveSize", 0.0, 1.0, 0.005, 1),
    ("Wave Frequency", 0.0, 5.0, 0.05, 2),
    ("Blur", 0.0, 0.05, 0.001, 3),
    ("GreyScale Width", 0.0, 1.0, 0.01, 4),
    ("GreyScale Height", 0.0, 1.0, 0.01, 5),
    ("Wobble Ring Size", 0.0, 1.0, 0.01, 6),
    ("Ripple Width", 0.01, 0.05, 0.001, 7),
    ("Brightness", 0.0, 1.0, 0.05, 11),
    ("R", 0.0, 1.99, 0.01, 8),
    ("G", 0.0, 1.99, 0.01, 9),
    ("B", 0.0, 1.99, 0.01, 10),
];

macro_rules! zone {
    ($zone:ident, $i:expr) => {
        Some(Live {
            get: |w| w.get_resource::<ZoneFx>().map(|z| GfxValue::from(z.tweaks.$zone[$i])),
            set: |w, v| {
                if let Some(mut z) = w.get_resource_mut::<ZoneFx>() {
                    z.tweaks.$zone[$i] = v.f32();
                }
            },
        })
    };
}

/// tyrespray.cpp 0x00155070
fn tyrespray_155070(r: &mut Reg) {
    use crate::zone_render_pc_xbox::{MAX_ZONE, MIN_ZONE};
    r.sys("Graphics.ZoneFX");
    r.b("On/off", OFF, res!(ZoneFx, |z| z.tweaks.on));
    r.f("FadeRateA", 0.0, 1.0, 0.01, 0.25, res!(ZoneFx, |z| z.tweaks.fade_a));
    r.f("FadeRateB", 0.0, 1.0, 0.01, 0.25, res!(ZoneFx, |z| z.tweaks.fade_b));
    r.b("Invert Colour", OFF, res!(ZoneFx, |z| z.tweaks.invert));
    r.f("Min Dilation", 0.0, 1.0, 0.01, f32::from_bits(0x3ECC_CCCD), res!(ZoneFx, |z| z.tweaks.min_dilation));
    r.f("Max Dilation Speed", 0.0, 125.0, 1.0, 75.0, res!(ZoneFx, |z| z.tweaks.max_dilation_speed));
    let min_live = [zone!(min_zone, 0), zone!(min_zone, 1), zone!(min_zone, 2), zone!(min_zone, 3), zone!(min_zone, 4), zone!(min_zone, 5),
        zone!(min_zone, 6), zone!(min_zone, 7), zone!(min_zone, 8), zone!(min_zone, 9), zone!(min_zone, 10), zone!(min_zone, 11)];
    let max_live = [zone!(max_zone, 0), zone!(max_zone, 1), zone!(max_zone, 2), zone!(max_zone, 3), zone!(max_zone, 4), zone!(max_zone, 5),
        zone!(max_zone, 6), zone!(max_zone, 7), zone!(max_zone, 8), zone!(max_zone, 9), zone!(max_zone, 10), zone!(max_zone, 11)];
    r.sys("Graphics.ZoneFX.MinZone");
    for (name, min, max, step, i) in ZONE_FIELDS {
        r.f(name, min, max, step, MIN_ZONE[i], min_live[i]);
    }
    r.sys("Graphics.ZoneFX.MaxZone");
    for (name, min, max, step, i) in ZONE_FIELDS {
        r.f(name, min, max, step, MAX_ZONE[i], max_live[i]);
    }
    r.sys("Graphics.ZoneFX.OneOff");
    r.f("One off speed", 0.0, 0.1, 0.001, 0.005, res!(ZoneFx, |z| z.tweaks.one_off_speed));
}

/// CarShaderManager_PC_XBOX.cpp 0x0015A5C0 `CFogRender::AddTweaks`
fn fog_render_add_tweaks(r: &mut Reg) {
    r.sys("Graphics.Fog");
    r.b("Enabled", OFF, res!(FogRender, |f| f.enabled));
    r.b("Fog Sky", OFF, res!(FogRender, |f| f.fog_sky));
    r.f("R", 0.0, 1.0, 0.01, 0.0, res!(FogRender, |f| f.colour.x));
    r.f("G", 0.0, 1.0, 0.01, 0.0, res!(FogRender, |f| f.colour.y));
    r.f("B", 0.0, 1.0, 0.01, 0.0, res!(FogRender, |f| f.colour.z));
    r.f("Alpha", 0.0, 1.0, 0.05, 0.0, res!(FogRender, |f| f.alpha));
    r.f("Fade (Std)", 0.0, 1.0, 0.05, 0.0, res!(FogRender, |f| f.fade_std));
    r.f("Distance", 0.0, 1.0, 0.05, 0.0, res!(FogRender, |f| f.distance));
    r.f("Intensity", 0.0, 1.0, 0.05, 0.0, res!(FogRender, |f| f.intensity));
    r.i("Type (Off/Std/Fade)", 0, 2, 1, 0, res!(FogRender, |f| f.fog_type));
    r.b("Rebloom (Fade)", OFF, res!(FogRender, |f| f.rebloom));
    r.i("Fade start (Fade)", 0, 255, 1, 0, res!(FogRender, |f| f.fade_start));
    r.i("Fade end (Fade)", 0, 255, 1, 0, res!(FogRender, |f| f.fade_end));
}

/// CarShaderManager_PC_XBOX.cpp 0x0015AF70 — the FSE manager's mip bias and shader reload (no reader in the port)
fn car_shader_manager_15af70(r: &mut Reg) {
    r.sys("Graphics.FullScreenEffects");
    r.f("BaseMipMapBias", -6.0, 0.0, 0.2, 0.0, None);
    r.f("MipMapBiasFOVScale", 0.0, 4.0, 0.2, 0.0, None);
    r.b("Reload Shaders", OFF, None);
}

/// GameShaderProgramManager_PC_XBOX.cpp 0x0015F820
fn game_shader_program_manager_15f820(r: &mut Reg) {
    r.sys("Graphics.Bloom");
    r.b("On/Off", OFF, res!(GlowTweaks, |g| g.on));
    r.i("Num Blur Passes", 0, 10, 1, 0, res!(GlowTweaks, |g| g.passes));
    r.f("Colour Intensity", 0.0, 1.0, 0.1, 0.0, res!(GlowTweaks, |g| g.intensity));
    r.b("Display blur On/Off", OFF, res!(GlowTweaks, |g| g.display_blur));
    r.b("Use stencil on read On/Off", OFF, res!(GlowTweaks, |g| g.stencil_on_read));
}

/// GameShaderProgramManager_PC_XBOX.cpp 0x00161190
fn game_shader_program_manager_161190(r: &mut Reg) {
    use crate::projected_light_beam_pc_xbox as b;
    r.sys("Graphics.3DLightBeams");
    r.b("On/off", ON, res!(ProjectedLightBeam, |p| p.on));
    r.f("MinOffset", 0.0, 1.0, 0.01, b::MIN_OFFSET, res!(BeamTweaks, |t| t.min_offset));
    r.f("MaxOffset", 0.0, 1.0, 0.01, b::MAX_OFFSET, res!(BeamTweaks, |t| t.max_offset));
    r.f("MinSigma", 0.0, 1.0, 0.005, b::MIN_SIGMA, res!(BeamTweaks, |t| t.min_sigma));
    r.f("MaxSigma", 0.0, 1.0, 0.01, b::MAX_SIGMA, res!(BeamTweaks, |t| t.max_sigma));
    r.f("FOV", 0.1, 3.0, 0.01, b::BEAM_FOV, res!(BeamTweaks, |t| t.fov));
    r.f("Extent", 10.0, 100.0, 1.0, b::BEAM_EXTENT, res!(BeamTweaks, |t| t.extent));
    r.f("DamagedFOV", 0.1, 3.0, 0.01, b::DAMAGED_FOV, res!(BeamTweaks, |t| t.damaged_fov));
    r.f("DamagedExtent", 10.0, 100.0, 1.0, b::DAMAGED_EXTENT, res!(BeamTweaks, |t| t.damaged_extent));
    r.f("FallOffStart", 0.0, 1.0, 0.05, b::FALL_OFF_START, res!(BeamTweaks, |t| t.fall_off_start));
    r.f("BeamDip", -1.0, 1.0, 0.05, b::BEAM_DIP, res!(BeamTweaks, |t| t.beam_dip));
    r.i("Intensity", 0, 255, 1, b::BEAM_INTENSITY, res!(BeamTweaks, |t| t.intensity));
    r.b("Redraw Texture", OFF, res!(BeamTweaks, |t| t.redraw));
}

/// each frame: a slot the menu changed (value ≠ last synced) is written to its live variable, every other slot
/// takes the live value (so the menu shows what the renderer uses)
pub fn sync_graphics_tweaks(world: &mut World) {
    world.resource_scope::<GraphicsTweaks, _>(|world, mut gt| {
        let gt = &mut *gt;
        for i in 0..gt.values.len() {
            let Some(live) = gt.live[i] else {
                gt.applied[i] = gt.values[i];
                continue;
            };
            if gt.values[i] != gt.applied[i] {
                (live.set)(world, gt.values[i]);
            } else if let Some(v) = (live.get)(world) {
                gt.values[i] = v.conform(gt.values[i]);
            }
            gt.applied[i] = gt.values[i];
        }
    });
}

/// `GraphicsTweaks` + the tweak resources it reaches that no other plugin creates
pub struct GraphicsTweaksPlugin;

impl Plugin for GraphicsTweaksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphicsTweaks>()
            .init_resource::<CarRenderTweaks>()
            .init_resource::<BeamTweaks>()
            .add_systems(First, sync_graphics_tweaks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// the TSV dump of the 21 registrations (`tools/ida/gfx_tweak_regs.py`): fn, kind, name, var, args, init
    fn dump() -> Option<String> {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../research/gfx/tweak_regs.tsv");
        std::fs::read_to_string(p).ok()
    }

    fn unquote(s: &str) -> String {
        let s = s.trim().trim_start_matches("(int)").trim();
        s.trim_matches('"').to_string()
    }

    /// a Hex-Rays constant: float literal (`0.1f`, `1.0`), int (`2u`, `0x20u`, `-1`), float bits pushed as an int,
    /// `COERCE_INT(x * 0.02f)` / `x * 0.02f`
    fn num_f(s: &str) -> f32 {
        let s = s.trim();
        if let Some(inner) = s.strip_prefix("COERCE_INT(").and_then(|x| x.strip_suffix(')')) {
            return num_f(inner);
        }
        if let Some((a, b)) = s.split_once(" * ") {
            if a.ends_with('f') {
                return num_f(a) * num_f(b);
            }
            return (a.parse::<f64>().unwrap() * num_f(b) as f64) as f32;
        }
        if s.contains('.') {
            return s.trim_end_matches('f').parse::<f32>().unwrap();
        }
        let i = num_i(s);
        if i.unsigned_abs() > 100_000 { f32::from_bits(i as u32) } else { i as f32 }
    }

    fn num_i(s: &str) -> i32 {
        let s = s.trim().trim_end_matches('u');
        if let Some(h) = s.strip_prefix("0x") {
            u32::from_str_radix(h, 16).unwrap() as i32
        } else {
            s.parse::<i64>().unwrap() as i32
        }
    }

    /// every registration matches the binary's calls: order, systems, kinds, names, ranges, steps, enum elements,
    /// and every .data-initialised variable's default
    #[test]
    fn registrations_match_the_binary() {
        let Some(tsv) = dump() else {
            eprintln!("research/gfx/tweak_regs.tsv missing: skipped");
            return;
        };
        let gt = GraphicsTweaks::default();
        assert_eq!(gt.fns.len(), 21);
        let mut n = 0;
        for f in &gt.fns {
            let rows: Vec<Vec<&str>> =
                tsv.lines().map(|l| l.split('\t').collect::<Vec<_>>()).filter(|c| u32::from_str_radix(c[0], 16).unwrap() == f.origin).collect();
            assert_eq!(rows.len(), f.calls.len(), "{:x}: call count", f.origin);
            for (c, row) in f.calls.iter().zip(&rows) {
                let name = unquote(row[2]);
                let args: Vec<&str> = if row.len() > 4 && !row[4].is_empty() { row[4].split('|').collect() } else { vec![] };
                match c {
                    GfxTweakCall::System(p) => assert_eq!((row[1], *p), ("S", name.as_str()), "{:x}", f.origin),
                    GfxTweakCall::Float { name: nm, slot, min, max, step } => {
                        assert_eq!((row[1], *nm), ("F", name.as_str()), "{:x}", f.origin);
                        let want = (num_f(args[0]), num_f(args[1]), num_f(args[2]));
                        assert_eq!((min.to_bits(), max.to_bits(), step.to_bits()), (want.0.to_bits(), want.1.to_bits(), want.2.to_bits()), "{nm}");
                        if row[5].len() == 8 {
                            assert_eq!(gt.values[*slot].f32().to_bits(), u32::from_str_radix(row[5], 16).unwrap(), "{nm} default");
                        }
                    }
                    GfxTweakCall::Int { name: nm, slot, min, max, step } => {
                        assert_eq!((row[1], *nm), ("I", name.as_str()), "{:x}", f.origin);
                        assert_eq!((*min, *max, *step), (num_i(args[0]), num_i(args[1]), num_i(args[2])), "{nm}");
                        if row[5].len() == 8 {
                            let raw = u32::from_str_radix(row[5], 16).unwrap();
                            // byte variables under an S32 tweak: the low byte is the variable
                            let v = gt.values[*slot].i32();
                            assert!(v == raw as i32 || v == (raw & 0xFF) as i32, "{nm} default {v} vs {raw:08X}");
                        }
                    }
                    GfxTweakCall::Enum { name: nm, slot, elements } => {
                        assert_eq!((row[1], *nm), ("E", name.as_str()));
                        assert_eq!(*elements, unquote(row[4]));
                        assert_eq!(gt.values[*slot].i32(), num_i(&format!("0x{}", row[5])));
                    }
                    GfxTweakCall::Bool { name: nm, slot } => {
                        assert_eq!((row[1], *nm), ("B", name.as_str()), "{:x}", f.origin);
                        if row[5].len() == 8 {
                            assert_eq!(gt.values[*slot].bool(), u32::from_str_radix(row[5], 16).unwrap() & 0xFF != 0, "{nm} default");
                        } else if row[5] == "bss" {
                            assert!(!gt.values[*slot].bool(), "{nm} bss default");
                        }
                    }
                }
                n += 1;
            }
        }
        assert_eq!(n, tsv.lines().count());
    }

    /// slots are unique and dense; the phase table names every registration
    #[test]
    fn slots_and_phases() {
        let gt = GraphicsTweaks::default();
        let slots: Vec<usize> = gt.fns.iter().flat_map(|f| f.calls.iter().filter_map(|c| c.slot())).collect();
        assert_eq!(slots, (0..gt.values.len()).collect::<Vec<_>>());
        assert_eq!(gt.values.len(), 297);
        for f in &gt.fns {
            assert!(PHASES.iter().any(|p| p.2.contains(&f.origin)), "{:x} has no phase", f.origin);
        }
        assert_eq!(GraphicsTweaks::phase_origins(GfxTweakPhase::FrontEndInit), [0x0010D7E0, 0x0010E3F0, 0x00122AE0, 0x001212D0, 0x00131130]);
    }

    /// a menu edit reaches the live variable, a live change reaches the slot
    #[test]
    fn sync_pushes_edits_and_pulls_live_values() {
        let mut world = World::new();
        world.init_resource::<GraphicsTweaks>();
        world.init_resource::<BeamTweaks>();
        world.insert_resource(GlowTweaks { on: true, passes: 3, intensity: 0.5, display_blur: false, stencil_on_read: true });
        let gt = world.resource::<GraphicsTweaks>();
        let find = |origin: u32, name: &str| {
            gt.calls(origin).unwrap().iter().find_map(|c| match c {
                GfxTweakCall::Float { name: n, slot, .. } | GfxTweakCall::Int { name: n, slot, .. } | GfxTweakCall::Bool { name: n, slot } if *n == name => Some(*slot),
                _ => None,
            })
        };
        let (extent, passes) = (find(0x00161190, "Extent").unwrap(), find(0x0015F820, "Num Blur Passes").unwrap());
        sync_graphics_tweaks(&mut world);
        assert_eq!(world.resource::<GraphicsTweaks>().values[passes], GfxValue::Int(3));
        world.resource_mut::<GraphicsTweaks>().values[extent] = GfxValue::Float(50.0);
        sync_graphics_tweaks(&mut world);
        assert_eq!(world.resource::<BeamTweaks>().extent, 50.0);
        world.resource_mut::<GlowTweaks>().passes = 7;
        sync_graphics_tweaks(&mut world);
        assert_eq!(world.resource::<GraphicsTweaks>().values[passes], GfxValue::Int(7));
        assert!(world.resource::<GraphicsTweaks>().is_live(extent));
    }
}
