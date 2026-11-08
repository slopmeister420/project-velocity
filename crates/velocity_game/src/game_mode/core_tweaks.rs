//! core's AddTweaks registrations (VsCrashCamera, VsFlyCamera, VsVehicleFollowCameraPointOfInterest,
//! CompartmentManager, GameManager, StringManager (heat haze), net, ReplayCamera_Fixed, ScoreManager,
//! VibrateEffects) and the replay of graphics' recorded registrations (`visos_render::gfx_tweaks`) at the
//! original call sites. Names, ranges, steps and defaults are copied from the calls (`research/core_tweaks.tsv`,
//! `tools/ida/core_tweak_regs.py`).
//!
//! The original points each tweak at its global; here every tweak gets a `TweakCell` kept in [`CoreTweaks`],
//! optionally bound to the live port variable ([`Live`]): `core_tweaks_in` copies the live values into the cells
//! before the debug menu runs, `core_tweaks_out` writes edited cells back after it. Unbound cells are the variable
//! themselves (the port has no consumer, or the consumer is a constant: see docs/core.md "Debug menu").

use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;
use visos_render::gfx_tweaks::{GfxTweakCall, GfxValue, GraphicsTweaks, PHASES};

use crate::camera::game_cameras::GameCameras;
use crate::cli::tweak::{CTweakManager, TweakCell, TweakData};
use crate::game_data_loading::game_data_loading::ELoadStep;
use crate::game_mode::game_manager::{GameManager, GameModeMessage, LoadStepMessage};
use crate::score_manager::score_manager::ScoreTweaks;

/// accessors of the live variable behind a cell (`get` None = no such resource now)
#[derive(Clone, Copy)]
pub struct Live {
    pub get: fn(&World) -> Option<TweakData>,
    pub set: fn(&mut World, &TweakData),
}

/// the shared "Game.ScoreManager" globals (`dword_68769C..687768`); the binary copies them into every ai
/// vehicle's `ScoreManager.tw` when they change
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct ScoreTweaksRes(pub ScoreTweaks);

/// one registration call as made (for the test against the original's calls)
#[derive(Clone, Debug, PartialEq)]
pub enum Rec {
    S(String),
    /// name, min bits, max bits, step bits, default bits
    F(String, u32, u32, u32, u32),
    /// name, min, max, step, default
    I(String, i32, i32, i32, i32),
    B(String, bool),
    E(String, String, i32),
}

/// every core tweak cell and the graphics slots registered so far (cleared by `TweaksReset`)
#[derive(Resource, Default)]
pub struct CoreTweaks {
    pub cells: Vec<(TweakCell, Option<Live>)>,
    /// (crate, value slot, cell) of the replayed graphics / audio / physics registrations
    pub ext: Vec<(ExtSrc, usize, TweakCell)>,
}

pub trait TweakVal: Copy + PartialEq {
    fn to_tweak(self) -> TweakData;
    fn from_tweak(v: &TweakData) -> Self;
}
impl TweakVal for f32 {
    fn to_tweak(self) -> TweakData {
        TweakData::Float(self)
    }
    fn from_tweak(v: &TweakData) -> Self {
        match *v {
            TweakData::Float(f) => f,
            TweakData::Int(i) => i as f32,
            TweakData::Bool(b) => b as i32 as f32,
            TweakData::String(_) => 0.0,
        }
    }
}
impl TweakVal for i32 {
    fn to_tweak(self) -> TweakData {
        TweakData::Int(self)
    }
    fn from_tweak(v: &TweakData) -> Self {
        match *v {
            TweakData::Float(f) => f as i32,
            TweakData::Int(i) => i,
            TweakData::Bool(b) => b as i32,
            TweakData::String(_) => 0,
        }
    }
}
impl TweakVal for u32 {
    fn to_tweak(self) -> TweakData {
        TweakData::Int(self as i32)
    }
    fn from_tweak(v: &TweakData) -> Self {
        i32::from_tweak(v) as u32
    }
}
impl TweakVal for bool {
    fn to_tweak(self) -> TweakData {
        TweakData::Bool(self)
    }
    fn from_tweak(v: &TweakData) -> Self {
        i32::from_tweak(v) != 0
    }
}
pub fn like<T: TweakVal>(_: &T, v: &TweakData) -> T {
    T::from_tweak(v)
}

/// `live!(Resource, r => r.path.to.field)`: a [`Live`] on one field of a resource
macro_rules! live {
    ($res:ty, $r:ident => $($p:tt)+) => {
        Some(Live {
            get: |w: &World| w.get_resource::<$res>().map(|$r| TweakVal::to_tweak($($p)+)),
            set: |w: &mut World, v: &TweakData| {
                if let Some(mut $r) = w.get_resource_mut::<$res>() {
                    let nv = like(&$($p)+, v);
                    if $($p)+ != nv {
                        $($p)+ = nv;
                    }
                }
            },
        })
    };
}

/// the registration context: the tree, the cells, the world (live defaults; None in tests) and the call log
pub struct Reg<'a> {
    pub tm: &'a mut CTweakManager,
    pub ct: &'a mut CoreTweaks,
    pub w: Option<&'a World>,
    pub s: usize,
    pub log: Vec<(u32, Rec)>,
    pub origin: u32,
}

impl<'a> Reg<'a> {
    pub fn new(tm: &'a mut CTweakManager, ct: &'a mut CoreTweaks, w: Option<&'a World>) -> Self {
        Reg { tm, ct, w, s: CTweakManager::ROOT, log: Vec::new(), origin: 0 }
    }
    fn rec(&mut self, r: Rec) {
        self.log.push((self.origin, r));
    }
    fn cell(&mut self, d: TweakData, live: Option<Live>) -> TweakCell {
        // the original's default is the variable's value at registration
        let v = live.and_then(|l| self.w.and_then(|w| (l.get)(w))).unwrap_or(d);
        let c = TweakCell::new(v);
        self.ct.cells.push((c.clone(), live));
        c
    }
    /// createSystem (tweak 0x00086250)
    pub fn sys(&mut self, path: &str) {
        self.s = self.tm.create_system(path);
        self.rec(Rec::S(path.into()));
    }
    /// addTweak float (0x00085A90): min / max as passed (float bits), step, .data default bits (0 = bss)
    pub fn f(&mut self, name: &str, min: i32, max: i32, step: f32, def: u32, live: Option<Live>) {
        let (lo, hi) = (f32::from_bits(min as u32), f32::from_bits(max as u32));
        let c = self.cell(TweakData::Float(f32::from_bits(def)), live);
        self.tm.add_tweak_f32(self.s, name, c, lo, hi, step);
        self.rec(Rec::F(name.into(), min as u32, max as u32, step.to_bits(), def));
    }
    /// addTweak S32 (0x00085B40)
    pub fn i(&mut self, name: &str, min: i32, max: i32, step: i32, def: i32, live: Option<Live>) {
        let c = self.cell(TweakData::Int(def), live);
        self.tm.add_tweak_i32(self.s, name, c, min, max, step);
        self.rec(Rec::I(name.into(), min, max, step, def));
    }
    /// addTweak enum (0x00085BF0)
    pub fn e(&mut self, name: &str, elements: &str, def: i32, live: Option<Live>) {
        let c = self.cell(TweakData::Int(def), live);
        self.tm.add_tweak_enum(self.s, name, c, elements);
        self.rec(Rec::E(name.into(), elements.into(), def));
    }
    /// addTweak bool (0x00085CA0)
    pub fn b(&mut self, name: &str, def: bool, live: Option<Live>) {
        let c = self.cell(TweakData::Bool(def), live);
        self.tm.add_tweak_bool(self.s, name, c);
        self.rec(Rec::B(name.into(), def));
    }
    /// a bool on a cell another registration already owns (the same global)
    fn b_shared(&mut self, name: &str, cell: TweakCell) {
        let def = cell.bool();
        self.tm.add_tweak_bool(self.s, name, cell);
        self.rec(Rec::B(name.into(), def));
    }

    /// graphics' recorded registration `origin`, replayed
    pub fn gfx(&mut self, origin: u32) {
        self.ext(ExtSrc::Gfx, origin);
    }

    /// another crate's recorded registration `origin`, replayed (cells start at the slot's current value)
    pub fn ext(&mut self, src: ExtSrc, origin: u32) {
        let Some(w) = self.w else { return };
        let Some(calls) = ext_calls(w, src, origin) else { return };
        let mut s = CTweakManager::ROOT;
        for c in calls {
            let cell = |r: &mut Self, slot: usize| {
                let cell = TweakCell::new(ext_get(w, src, slot).unwrap_or(TweakData::Int(0)));
                r.ct.ext.push((src, slot, cell.clone()));
                cell
            };
            match c {
                XCall::Sys(p) => s = self.tm.create_system(&p),
                XCall::F(name, slot, min, max, step) => {
                    let c = cell(self, slot);
                    self.tm.add_tweak_f32(s, name, c, min, max, step);
                }
                XCall::I(name, slot, min, max, step) => {
                    let c = cell(self, slot);
                    self.tm.add_tweak_i32(s, name, c, min, max, step);
                }
                XCall::E(name, slot, elements) => {
                    let c = cell(self, slot);
                    self.tm.add_tweak_enum(s, name, c, elements);
                }
                XCall::B(name, slot) => {
                    let c = cell(self, slot);
                    self.tm.add_tweak_bool(s, name, c);
                }
            }
        }
        self.s = s;
    }
}

/// whose recorded registrations (crates below velocity_game: graphics, audio, physics)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtSrc {
    Gfx,
    Audio,
    Phys,
}

/// one recorded call, owned (the three crates' call types have the same shape)
enum XCall {
    Sys(String),
    F(&'static str, usize, f32, f32, f32),
    I(&'static str, usize, i32, i32, i32),
    E(&'static str, usize, &'static str),
    B(&'static str, usize),
}

fn ext_calls(w: &World, src: ExtSrc, origin: u32) -> Option<Vec<XCall>> {
    use velocity_audio::audio_tweaks::{AudioTweakCall as A, AudioTweaks};
    use velocity_physics::physics_tweaks::{PhysTweakCall as P, PhysicsTweaks};
    Some(match src {
        ExtSrc::Gfx => w
            .get_resource::<GraphicsTweaks>()?
            .calls(origin)?
            .iter()
            .map(|c| match *c {
                GfxTweakCall::System(p) => XCall::Sys(p.into()),
                GfxTweakCall::Float { name, slot, min, max, step } => XCall::F(name, slot, min, max, step),
                GfxTweakCall::Int { name, slot, min, max, step } => XCall::I(name, slot, min, max, step),
                GfxTweakCall::Enum { name, slot, elements } => XCall::E(name, slot, elements),
                GfxTweakCall::Bool { name, slot } => XCall::B(name, slot),
            })
            .collect(),
        ExtSrc::Audio => w
            .get_resource::<AudioTweaks>()?
            .calls(origin)?
            .iter()
            .map(|c| match *c {
                A::System(p) => XCall::Sys(p.into()),
                A::Float { name, slot, min, max, step } => XCall::F(name, slot, min, max, step),
                A::Int { name, slot, min, max, step } => XCall::I(name, slot, min, max, step),
                A::Bool { name, slot } => XCall::B(name, slot),
            })
            .collect(),
        ExtSrc::Phys => w
            .get_resource::<PhysicsTweaks>()?
            .calls(origin)?
            .iter()
            .map(|c| match c {
                P::System(p) => XCall::Sys(p.clone()),
                P::Float { name, slot, min, max, step } => XCall::F(name, *slot, *min, *max, *step),
                P::Int { name, slot, min, max, step } => XCall::I(name, *slot, *min, *max, *step),
                P::Bool { name, slot } => XCall::B(name, *slot),
            })
            .collect(),
    })
}

/// the slot's value as tweak data
fn ext_get(w: &World, src: ExtSrc, slot: usize) -> Option<TweakData> {
    use velocity_audio::audio_tweaks::{AudioTweakValue as A, AudioTweaks};
    use velocity_physics::physics_tweaks::{PhysTweakValue as P, PhysicsTweaks};
    Some(match src {
        ExtSrc::Gfx => match *w.get_resource::<GraphicsTweaks>()?.values.get(slot)? {
            GfxValue::Float(f) => TweakData::Float(f),
            GfxValue::Int(i) => TweakData::Int(i),
            GfxValue::Bool(b) => TweakData::Bool(b),
        },
        ExtSrc::Audio => match *w.get_resource::<AudioTweaks>()?.values.get(slot)? {
            A::Float(f) => TweakData::Float(f),
            A::Int(i) => TweakData::Int(i),
            A::Bool(b) => TweakData::Bool(b),
        },
        ExtSrc::Phys => match *w.get_resource::<PhysicsTweaks>()?.values.get(slot)? {
            P::Float(f) => TweakData::Float(f),
            P::Int(i) => TweakData::Int(i),
            P::Bool(b) => TweakData::Bool(b),
        },
    })
}

/// the cell into the slot (kept in the slot's type; untouched when equal)
fn ext_put(w: &mut World, src: ExtSrc, slot: usize, c: &TweakCell) {
    use velocity_audio::audio_tweaks::{AudioTweakValue as A, AudioTweaks};
    use velocity_physics::physics_tweaks::{PhysTweakValue as P, PhysicsTweaks};
    macro_rules! put {
        ($res:ty, $v:ident) => {
            if let Some(mut r) = w.get_resource_mut::<$res>() {
                let Some(&old) = r.values.get(slot) else { return };
                let new = match old {
                    $v::Float(_) => $v::Float(c.f32()),
                    $v::Int(_) => $v::Int(c.i32()),
                    $v::Bool(_) => $v::Bool(c.bool()),
                };
                if old != new {
                    r.values[slot] = new;
                }
            }
        };
    }
    match src {
        ExtSrc::Gfx => put!(GraphicsTweaks, GfxValue),
        ExtSrc::Audio => put!(AudioTweaks, A),
        ExtSrc::Phys => put!(PhysicsTweaks, P),
    }
}

/// VsCrashCamera.cpp 0x00061D30 — "Camera.Crash" (the camera)
pub fn add_crash_camera_tweaks(r: &mut Reg) {
    r.origin = 0x61D30;
    r.sys("Camera.Crash");
    r.f("Duration", 1036831949, 1092616192, 0.1, 0x40000000, live!(GameCameras, c => c.crash.tweaks.duration));
    r.f("Duration (failure)", 1036831949, 1092616192, 0.1, 0x40A00000, live!(GameCameras, c => c.crash.tweaks.duration_failure));
    r.f("Speed rate", 1028443341, 1065353216, 0.05, 0x3E4CCCCD, live!(GameCameras, c => c.crash.tweaks.speed_rate));
    r.f("Recover time", 0, 1106247680, 1.0, 0x40200000, live!(GameCameras, c => c.crash.tweaks.recover_time));
    // `s` (5.0)
    r.f("Camera distance", 0, 1112014848, 0.5, 0x40A00000, live!(GameCameras, c => c.crash.tweaks.camera_distance));
    r.f("Camera start angle", 0, 1119092736, 1.0, 0x41F00000, live!(GameCameras, c => c.crash.tweaks.start_angle));
    r.f("Camera angle span", 0, 1135869952, 5.0, 0x42B40000, live!(GameCameras, c => c.crash.tweaks.angle_span));
    r.f("Height variation speed", 0, 1120403456, 1.0, 0x40000000, live!(GameCameras, c => c.crash.tweaks.height_variation_speed));
    r.f("Height variation low", 0, 1092616192, 0.1, 0x40000000, live!(GameCameras, c => c.crash.tweaks.height_low));
    r.f("Height variation high", 0, 1092616192, 0.1, 0x4101999A, live!(GameCameras, c => c.crash.tweaks.height_high));
    r.f("Target direction offset", -1063256064, 1092616192, 0.2, 0x40200000, live!(GameCameras, c => c.crash.tweaks.target_direction_offset));
    r.f("Target speed offset", 0, 1065353216, 0.001, 0x3C23D70A, live!(GameCameras, c => c.crash.tweaks.target_speed_offset));
    r.b("Use raceline for position", false, live!(GameCameras, c => c.crash.tweaks.use_raceline));
    r.b("Use rays to detect obstructions", false, live!(GameCameras, c => c.crash.tweaks.use_rays));
    r.b("Respawn car on raceline", true, live!(GameCameras, c => c.crash.tweaks.respawn_on_raceline));
    r.f("Respawn max speed", 0, 1120403456, 1.0, 0x41A00000, live!(GameCameras, c => c.crash.tweaks.respawn_max_speed));
    r.f("Respawn kick speed", 0, 1120403456, 1.0, 0x42200000, live!(GameCameras, c => c.crash.tweaks.respawn_kick_speed));
}

/// VsCrashCamera.cpp 0x00063160 — "Camera.Crash" (the trigger)
pub fn add_crash_trigger_tweaks(r: &mut Reg) {
    r.origin = 0x63160;
    r.sys("Camera.Crash");
    r.b("enabled", false, live!(GameCameras, c => c.crash.trigger.enabled));
    r.f("Ignore first seconds of race", 0, 1106247680, 1.0, 0x40000000, live!(GameCameras, c => c.crash.trigger.ignore_first_seconds));
    r.f("Min impact speed with car", 0, 1148846080, 1.0, 0x42040000, live!(GameCameras, c => c.crash.trigger.min_impact_speed_car));
    r.f("Min impact speed with level", 0, 1148846080, 1.0, 0x42040000, live!(GameCameras, c => c.crash.trigger.min_impact_speed_level));
    r.f("Min X axis angle (in air)", 0, 1127481344, 1.0, 0x41700000, live!(GameCameras, c => c.crash.trigger.min_x_angle_in_air));
    r.f("Min Z axis angle (in air)", 0, 1127481344, 1.0, 0x41700000, live!(GameCameras, c => c.crash.trigger.min_z_angle_in_air));
    r.b("car/car collisions", true, live!(GameCameras, c => c.crash.trigger.car_car));
    r.b("car/level collisions", true, live!(GameCameras, c => c.crash.trigger.car_level));
    r.b("All wheels on ground", true, live!(GameCameras, c => c.crash.trigger.all_wheels_on_ground));
    r.f("Min vehicle speed", 0, 1120403456, 1.0, 0x41F00000, live!(GameCameras, c => c.crash.trigger.min_vehicle_speed));
    r.f("Max forward angle", 0, 1127481344, 1.0, 0x41F00000, live!(GameCameras, c => c.crash.trigger.max_forward_angle));
}

/// `live!` on one follow camera global (`VsCameraManager.globals`)
macro_rules! glob {
    ($f:ident) => {
        live!(GameCameras, c => c.manager.globals.$f)
    };
}

/// VsFlyCamera.cpp 0x00065640 — "Camera.TurnRates" (follow camera globals) and "Camera" (player 0's view)
pub fn add_fly_camera_tweaks(r: &mut Reg) {
    r.origin = 0x65640;
    r.sys("Camera.TurnRates");
    r.f("Yaw slew rate", 1065353216, 1153138688, 5.0, 0x42F00000, glob!(yaw_slew_rate));
    r.f("Car->Cam max angle", 1065353216, 1119092736, 1.0, 0x42B40000, glob!(max_angle));
    r.sys("Camera");
    r.f("Height", -1054867456, 1092616192, 0.1, 0x3FE66666, live!(GameCameras, c => c.manager.camera_tweaks.height));
    r.f("InterestHeight", -1054867456, 1092616192, 0.1, 0x3FE66666, live!(GameCameras, c => c.manager.camera_tweaks.look_height));
    r.f("RearOffset", -1054867456, 1092616192, 0.05, 0x41100000, live!(GameCameras, c => c.manager.camera_tweaks.dist_a));
    r.f("BumperOffset", -1054867456, 1092616192, 0.05, 0x41100000, live!(GameCameras, c => c.manager.camera_tweaks.dist_b));
    r.f("FOVMin", 1008981770, 1120403456, 0.1, 0x42200000, live!(GameCameras, c => c.manager.camera_tweaks.fov));
    r.f("ZRotation", -1027080192, 1120403456, 1.0, 0, glob!(z_rotation));
}

/// VsFlyCamera.cpp 0x00065920 — "Camera.Globals.*" (VsVehicleFollowCamera globals, `VsCameraManager.globals`)
pub fn add_follow_camera_globals_tweaks(r: &mut Reg) {
    r.origin = 0x65920;
    r.sys("Camera.Globals.Banking");
    r.b("Banking enabled", true, glob!(banking));
    r.f("bankingSpeedLow", 0, 1128792064, 1.0, 0x41700000, glob!(bank_speed_min));
    r.f("bankingSpeedHigh", 0, 1128792064, 1.0, 0x41F00000, glob!(bank_speed_max));
    r.f("bankingMaxAngle", 0, 1128792064, 1.0, 0x41000000, glob!(bank_max_angle));
    r.f("bankingNosFactor", 0, 1065353216, 0.01, 0x3F000000, glob!(bank_nos_factor));
    r.f("bankingLag", 0, 1065353216, 0.01, 0x3D75C28F, glob!(bank_lag));
    r.sys("Camera.Globals.FOV");
    r.f("fovMinOveride", 0, 1123024896, 1.0, 0x42700000, glob!(fov_min));
    r.f("fovMaxOveride", 0, 1123024896, 1.0, 0x42A00000, glob!(fov_max));
    r.f("fovSpeedLow", 0, 1128792064, 1.0, 0x41700000, glob!(fov_speed_min));
    r.f("fovSpeedHigh", 0, 1128792064, 1.0, 0x42200000, glob!(fov_speed_max));
    r.f("fovChangeRate", 0, 1065353216, 0.01, 0x3E19999A, glob!(fov_rate_up));
    r.f("fovChangeRateRelease", 0, 1065353216, 0.01, 0x3D4CCCCD, glob!(fov_rate_down));
    r.f("fovBoost", 0, 1112014848, 1.0, 0x41A00000, glob!(fov_boost));
    r.f("cameraTrailMin", 0, 1101004800, 0.1, 0x40833333, glob!(dist_near_fov));
    r.f("cameraTrailMax", 0, 1101004800, 0.1, 0x40666666, glob!(dist_far_fov));
    r.sys("Camera.Globals.Kite");
    r.f("globalHeightDiff", -1073741824, 0x40000000, 0.01, 0xBEE66666, glob!(eye_y));
    r.f("kiteMaxPitchAngle", 0, 1097859072, 0.5, 0x40800000, glob!(kite_max_pitch));
    r.f("kiteMaxReceedDist", 0, 0x40000000, 0.05, 0x3F400000, glob!(kite_recede));
    r.f("kiteMaxGetCloserDist", 0, 0x40000000, 0.05, 0x3F19999A, glob!(kite_closer));
    r.f("kite accel factor", 0, 1065353216, 0.01, 0x3F800000, glob!(kite_accel_factor));
    r.f("kite brake factor", 0, 1065353216, 0.01, 0x3F800000, glob!(kite_brake_factor));
    r.f("kite speed factor", 0, 1065353216, 0.01, 0x3F000000, glob!(kite_speed_factor));
    r.f("accel rise time", 1036831949, 1092616192, 0.1, 0x3F333333, glob!(kite_accel_rise));
    r.f("brake rise time", 1036831949, 1092616192, 0.1, 0x3F8CCCCD, glob!(kite_brake_rise));
    r.f("kite accel decay factor", 1036831949, 1065353216, 0.005, 0x3F733333, glob!(kite_accel_decay));
    r.f("kite brake decay factor", 1036831949, 1065353216, 0.005, 0x3F7851EC, glob!(kite_brake_decay));
    r.f("speedDamping", 0, 1065353216, 0.01, 0x3D75C28F, glob!(pitch_speed_rate));
    r.f("trailSpeedLow", 0, 1128792064, 1.0, 0x40A00000, glob!(pitch_speed_min));
    r.f("trailSpeedHigh", 0, 1128792064, 1.0, 0x41200000, glob!(pitch_speed_max));
    r.f("pivotPoint", -1073741824, 1084227584, 0.1, 0x3F8CCCCD, glob!(pivot));
    r.f("farCamTrailMultiplier", 0, 0x40000000, 0.01, 0x3F99999A, glob!(far_cam_trail));
    r.sys("Camera.Globals.TurnRate");
    r.f("direction blend", 0, 1065353216, 0.05, 0x3F19999A, glob!(heading_weight));
    r.f("yawRatePerDegree", 0, 1133903872, 1.0, 0x43480000, glob!(yaw_gain));
    r.f("yawRelaxAngle", 0, 1110704128, 1.0, 0x41700000, glob!(yaw_full));
    r.f("yawDeadZoneAngle", 0, 1084227584, 0.1, 0x3F000000, glob!(yaw_dead));
    r.f("use dir below speed", 0, 1112014848, 0.1, 0x40000000, glob!(vel_speed_min));
    r.f("use vel above speed", 0, 1112014848, 0.1, 0x40C00000, glob!(vel_speed_max));
    r.sys("Camera.Globals.Bounce");
    r.f("bounce rate", 1036831949, 1084227584, 0.1, 0x40400000, glob!(bounce_rate));
    r.f("bounce damping", 1036831949, 0x40000000, 0.01, 0x3ECCCCCD, glob!(bounce_damping));
    r.f("bounce amount", 0, 1077936128, 0.01, 0x3E99999A, glob!(bounce_amount));
    r.b("pitch camera", true, glob!(pitch_camera));
    r.sys("Camera.Globals.Shake");
    r.f("vertical", 0, 1036831949, 0.001, 0x3C03126F, glob!(shake_v));
    r.f("horizontal", 0, 1036831949, 0.001, 0x3A83126F, glob!(shake_h));
    r.f("min speed", 0, 1128792064, 1.0, 0x41C80000, glob!(shake_speed_min));
    r.f("max speed", 0, 1128792064, 1.0, 0x42700000, glob!(shake_speed_max));
    r.f("drift factor", 0, 1092616192, 0.1, 0, glob!(shake_drift));
    r.f("3rd person scale", 0, 1065353216, 0.01, 0x3F000000, glob!(shake_3rd_person));
    r.sys("Camera.Globals.Zone");
    r.b("use trail", true, glob!(zone_use_trail));
    r.b("use ramp curve", true, glob!(zone_ramp_curve));
    r.f("duration", 0, 1084227584, 0.1, 0x3F000000, glob!(zone_duration));
    r.f("fov change", 0, 1112014848, 1.0, 0, glob!(zone_fov_change));
    r.f("pitch change", -1035468800, 1112014848, 1.0, 0xC1C80000, glob!(zone_pitch_change));
    r.f("trail change", 0, 1084227584, 0.1, 0, glob!(zone_trail_change));
}

/// VsVehicleFollowCameraPointOfInterest.cpp 0x0006A930 — "Camera.PointOfInterest"
pub fn add_point_of_interest_tweaks(r: &mut Reg) {
    r.origin = 0x6A930;
    r.sys("Camera.PointOfInterest");
    r.b("Use cinematic bars", true, live!(GameCameras, c => c.poi_tweaks.cinematic_bars));
    r.b("AI control", false, live!(GameCameras, c => c.poi_tweaks.ai_control));
    r.f("In time, player", 0, 0x40000000, 0.025, 0x3EC00000, live!(GameCameras, c => c.poi_tweaks.in_time_player));
    r.f("Out time, player", 0, 0x40000000, 0.025, 0x3EC00000, live!(GameCameras, c => c.poi_tweaks.out_time_player));
    r.f("In time, target", 0, 0x40000000, 0.025, 0x3EC00000, live!(GameCameras, c => c.poi_tweaks.in_time_target));
    r.f("Out time, target", 0, 0x40000000, 0.025, 0x3EC00000, live!(GameCameras, c => c.poi_tweaks.out_time_target));
    r.f("Speed rate", 1008981770, 0x40000000, 0.01, 0x3ECCCCCD, live!(GameCameras, c => c.poi_tweaks.speed_rate));
    r.f("Default length", 1020054733, 1084227584, 0.025, 0x40000000, live!(GameCameras, c => c.poi_tweaks.default_length));
    r.f("AI control extra time", 0, 1084227584, 0.05, 0x3F800000, live!(GameCameras, c => c.poi_tweaks.ai_control_extra_time));
    r.f("Start delay", 0, 1084227584, 0.05, 0x3F000000, live!(GameCameras, c => c.poi_tweaks.start_delay));
    r.b("Enable extra FOV", true, live!(GameCameras, c => c.poi_tweaks.extra_fov));
    r.f("Min FOV distance", 0, 1120403456, 1.0, 0x40800000, live!(GameCameras, c => c.poi_tweaks.min_fov_distance));
    r.f("Max FOV distance", 0, 1140457472, 5.0, 0x420C0000, live!(GameCameras, c => c.poi_tweaks.max_fov_distance));
    r.f("Extra FOV at max distance", 0, 1065353216, 0.001, 0x3DCCCCCD, live!(GameCameras, c => c.poi_tweaks.extra_fov_at_max));
}

/// CompartmentManager.cpp 0x00087010 — "Compartments" (debug displays of the streaming manager, not ported)
pub fn add_compartment_tweaks(r: &mut Reg) {
    r.origin = 0x87010;
    r.sys("Compartments");
    r.b("Render Cpt Instances", true, None);
    // manager +49 / +50 / +48 / +52 / +51 (cleared by the constructor)
    r.b("Display load status", false, None);
    r.b("Display debug status", false, None);
    r.b("Lock Compartment Choice", false, None);
    r.b("Show compartment numbers", false, None);
    r.b("Show load times", false, None);
}

/// GameManager.cpp 0x000B4170 — InGameInit's own tweaks (clears GameManager +1..+6, sets +8 = 1); calls
/// SpecularChars 0x00145F20 (graphics) and VibrateEffects 0x001EDA50 at the end
pub fn add_game_manager_tweaks(r: &mut Reg) {
    r.origin = 0xB4170;
    r.sys("Debug Cameras");
    r.b("Fly'n'Dump Camera", false, None);
    r.b("Debug Camera", false, None);
    r.b("FreeFly Camera", false, None);
    r.sys("Physics");
    r.b("Show Friction Values", false, None);
    r.sys("Graphics.Display");
    r.b("OutputFpsInfo", false, None);
    r.b("HideOSD", false, None);
    r.b("Widescreen", false, live!(GameCameras, c => c.manager.widescreen));
    r.b("RenderSky", true, None);
    r.b("RenderAlpha", true, None);
    r.b("RenderSolid", true, None);
    r.b("RenderCars", true, None);
    r.b("Chosen Lights Display", false, None);
    // VsBaseCamera 0x0005EC60 result +300
    r.b("Average Light Sources", false, None);
    // `byte_6F5B65`, the same global as FontHelper's "System" tweak
    match r.w.and_then(|w| w.get_resource::<crate::font_helper::SystemTweaks>()).map(|t| t.texture_viewer.clone()) {
        Some(c) => r.b_shared("Texture Viewer", c),
        None => r.b("Texture Viewer", false, None),
    }
    r.b("Display Debug Graph", false, None);
    r.b("Render Timings Graph", false, None);
    r.b("Display Poly Counts", false, None);
    r.sys("Graphics.Camera");
    r.f("FarClipDistance", 1092616192, 1161527296, 10.0, 0x44160000, live!(GameCameras, c => c.manager.clip.far));
    r.f("NearClipDistance", 1008981770, 1091567616, 0.1, 0x3F800000, live!(GameCameras, c => c.manager.clip.near));
    r.b("Enable Mirror", true, live!(GameCameras, c => c.manager.clip.mirror));
    r.f("Mirror FarClipDistance", 1092616192, 1161527296, 10.0, 0x43480000, live!(GameCameras, c => c.manager.clip.mirror_far));
    r.sys("Graphics.Shadow");
    r.b("Silhouette", false, None);
    r.sys("Game");
    r.b("ShowCarPosition", false, None);
    r.f("TrailBlazer Start Speed", 0, 1125515264, 1.0, 0x42200000, None);
    r.f("TrailBlazer End Speed", 0, 1132068864, 1.0, 0x42DC0000, None);
    // `byte_6F5C78` (StartLineEffects' bss colour object): +132, +112 xyz, +128, +96 xyz, +136
    r.sys("Graphics.Contrast");
    r.f("Brightness ( Multiply - free)", 0, 1073657938, 0.02, 0, None);
    r.f("\t\t-Red", 0, 1065353216, 0.01, 0, None);
    r.f("\t\t-Green", 0, 1065353216, 0.01, 0, None);
    r.f("\t\t-Blue", 0, 1065353216, 0.01, 0, None);
    r.f("Dark ( Subtract - not free)", 0, 1065353216, 0.01, 0, None);
    r.f("\t sub-Red", 0, 1065353216, 0.01, 0, None);
    r.f("\t sub-Green", 0, 1065353216, 0.01, 0, None);
    r.f("\t sub-Blue", 0, 1065353216, 0.01, 0, None);
    r.f("Add ( free )", 0, 1065353216, 0.01, 0, None);
    // 0x000B4BD1 SpecularChars (graphics), then VibrateEffects
    r.gfx(0x145F20);
    add_vibrate_tweaks(r);
}

/// VibrateEffects.cpp `off_688C10`: name, motor, intensity bits, decay bits
pub const VIBRATE_EFFECTS: [(&str, i32, u32, u32); 8] = [
    ("SMALL_COLLISION", 0, 0x3E99999A, 0x3E4CCCCD),
    ("MEDIUM_COLLISION", 1, 0x3ECCCCCD, 0x3ECCCCCD),
    ("LARGE_COLLISION", 1, 0x3F19999A, 0x3F19999A),
    ("SLIPSTREAM", 1, 0x3F19999A, 0x3DCCCCCD),
    ("NITROUS", 0, 0x3F333333, 0x3DCCCCCD),
    ("ZONE", 1, 0x3F800000, 0x3F000000),
    ("SKID", 1, 0x3F000000, 0x3F000000),
    ("TRAILBLAZER", 1, 0x3F800000, 0x3F000000),
];

/// VibrateEffects.cpp 0x001EDA50 — "controller.vibrate.<effect>", live on `VibratePresets` row `$i`
pub fn add_vibrate_tweaks(r: &mut Reg) {
    use crate::controller::vibrate_effects::VibratePresets as VP;
    r.origin = 0x1EDA50;
    macro_rules! row {
        ($i:literal) => {{
            let (name, motor, intensity, decay) = VIBRATE_EFFECTS[$i];
            r.sys(&format!("controller.vibrate.{name}"));
            r.e("motor", "Light = 0, Heavy = 1", motor, live!(VP, p => p.0[$i].motor));
            r.f("intensity", 0, 1065353216, 0.1, intensity, live!(VP, p => p.0[$i].intensity));
            r.f("decay", 0, 1092616192, 0.1, decay, live!(VP, p => p.0[$i].decay));
        }};
    }
    row!(0);
    row!(1);
    row!(2);
    row!(3);
    row!(4);
    row!(5);
    row!(6);
    row!(7);
}

/// StringManager.cpp 0x000C9450 — "Graphics.HeatHaze" (the heat haze globals; graphics has no consumer yet)
pub fn add_heat_haze_tweaks(r: &mut Reg) {
    r.origin = 0xC9450;
    r.sys("Graphics.HeatHaze");
    r.f("Size of UV Movement in X", 0, 1045220557, 0.001, 0x3BA3D70A, None);
    r.f("Size of UV Movement in Y", 0, 1045220557, 0.001, 0x3BA3D70A, None);
    r.f("Frequency X ", 0, 1124073472, 1.0, 0x41800000, None);
    r.f("Frequency Y ", 0, 1124073472, 1.0, 0x40800000, None);
    r.f("Grid Size ", 1017370378, 1050253722, 0.005, 0x3D4CCCCD, None);
}

/// net.cpp 0x000CDD70 — "Network" (GameManager::Init; no network game in the port)
pub fn add_network_tweaks(r: &mut Reg) {
    r.origin = 0xCDD70;
    r.sys("Network");
    r.b("Selective_Update", false, None);
    r.b("Compress_Packets", false, None);
    r.f("Ticks_Between_Peer_Updates", 1065353216, 1103101952, 1.0, 0x41C00000, None);
    r.sys("Network.Server");
    r.f("Broadcast_Interval", 1036831949, 1084227584, 0.1, 0x3F800000, None);
    r.f("Timesync_Interval", 1036831949, 1084227584, 0.1, 0x3DCCCCCD, None);
    r.f("PlayerDetails_Interval", 1036831949, 1084227584, 0.1, 0x3F800000, None);
    r.sys("Network.Client");
    r.f("Timesync_Interval", 1036831949, 1084227584, 0.1, 0x3DCCCCCD, None);
    r.f("PlayerDetails_Interval", 1036831949, 1084227584, 0.1, 0x3F800000, None);
}

/// net.cpp 0x000CF200 — "Network.Timings" (only from the network session code 0x000CF840 / 0x000CFA50: never
/// called in the port); calls 0x000CDD70 first
pub fn add_network_timing_tweaks(r: &mut Reg) {
    add_network_tweaks(r);
    r.origin = 0xCF200;
    r.sys("Network.Timings");
    r.f("Added Latency (s)", 0, 1065353216, 0.025, 0, None);
    r.b("Disable Timeouts", false, None);
    r.f("Lobby Timeout (s)", 0, 1112014848, 1.0, 0x41F00000, None);
    r.f("WaitingForStart Timeout (s)", 0, 1112014848, 1.0, 0x42700000, None);
    r.f("InGame Timeout (s)", 0, 1112014848, 1.0, 0x41F00000, None);
    r.f("Timesync Interval (s)", 0, 1084227584, 0.1, 0x3E800000, None);
    r.f("Server Broadcast Interval (s)", 0, 1084227584, 0.25, 0x3E800000, None);
    r.f("Server GameDetails Interval (s)", 0, 1084227584, 0.25, 0x3F800000, None);
    r.f("Client PlayerDetails Interval (s)", 0, 1092616192, 0.5, 0x3F800000, None);
    r.f("Client Ready Interval (s)", 0, 1092616192, 0.5, 0x3F800000, None);
    r.f("Client Data Sync Interval (s)", 0, 1092616192, 0.5, 0x3F000000, None);
}

/// ReplayCamera_Fixed.cpp 0x00165640 — "Replay Cameras"
pub fn add_replay_camera_tweaks(r: &mut Reg) {
    r.origin = 0x165640;
    r.sys("Replay Cameras");
    // `byte_749120`: ReplayCamera_Manager 0x00165A30 reloads cameras.cam (not ported)
    r.b("Reload Replay Camera File", false, None);
    r.i("Enable Cameras For Review", 0, 2, 0, 0, live!(GameCameras, c => c.replay.review_cameras));
}

/// ScoreManager (ReplayCamera_Wheel.cpp-labelled) 0x00167F30 — "Game.ScoreManager.*", from 0x00017F60 for a local
/// vehicle's score manager
pub fn add_score_manager_tweaks(r: &mut Reg) {
    r.origin = 0x167F30;
    r.sys("Game.ScoreManager");
    r.f("Multiplier Reset Time", 0, 1092616192, 0.1, 0x40400000, live!(ScoreTweaksRes, s => s.0.multiplier_reset_time));
    r.f("Zone Multiplier", 0, 1092616192, 0.1, 0x40000000, live!(ScoreTweaksRes, s => s.0.zone_multiplier));
    r.sys("Game.ScoreManager.ZoneSettings");
    r.b("Infinite Zone", false, live!(ScoreTweaksRes, s => s.0.infinite_zone));
    r.f("Max Zone Time", 0, 1101004800, 0.5, 0x41200000, live!(ScoreTweaksRes, s => s.0.max_zone_time));
    r.f("Zone Incremement Per Score", 0, 1092616192, 0.001, 0x3F2AACDA, live!(ScoreTweaksRes, s => s.0.zone_per_score));
    r.f("Zone Decremement Per Crash", 0, 1092616192, 0.1, 0x3F000000, live!(ScoreTweaksRes, s => s.0.zone_per_crash));
    r.f("InTheZone Increment Multiplier", 0, 1092616192, 0.1, 0x3FC00000, live!(ScoreTweaksRes, s => s.0.in_zone_increment));
    r.sys("Game.ScoreManager.NosSettings");
    r.f("Nos Incremement Per Score", 0, 1092616192, 0.001, 0x3F000000, live!(ScoreTweaksRes, s => s.0.nos_per_score));
    r.sys("Game.ScoreManager.Overtake");
    r.i("Overtake Score", 0, 100, 1, 20, live!(ScoreTweaksRes, s => s.0.overtake));
    r.i("Clean Overtake Score", 0, 100, 1, 30, live!(ScoreTweaksRes, s => s.0.clean_overtake));
    r.sys("Game.ScoreManager.TopSpeed");
    r.i("Top Speed Score", 0, 100, 1, 10, live!(ScoreTweaksRes, s => s.0.top_speed));
    r.sys("Game.ScoreManager.Drift");
    r.i("Drift Score", 0, 100, 1, 20, live!(ScoreTweaksRes, s => s.0.drift));
    r.i("Handbrake Turn Score", 0, 100, 1, 15, live!(ScoreTweaksRes, s => s.0.handbrake_turn));
    r.f("360 Spin time", 0, 1092616192, 0.1, 0x40000000, live!(ScoreTweaksRes, s => s.0.spin_time));
    r.sys("Game.ScoreManager.Lap");
    r.i("Lead Entire Lap Score", 0, 100, 1, 20, live!(ScoreTweaksRes, s => s.0.lead_lap));
    r.i("Clean Lap Score", 0, 100, 1, 20, live!(ScoreTweaksRes, s => s.0.clean_lap));
    r.i("Fastest Lap Score", 0, 100, 1, 10, live!(ScoreTweaksRes, s => s.0.fastest_lap));
    r.sys("Game.ScoreManager.CleanStart");
    r.i("Clean Start Score", 0, 100, 1, 10, live!(ScoreTweaksRes, s => s.0.clean_start));
    r.f("Clean Start Time", 0, 1106247680, 0.1, 0x40400000, live!(ScoreTweaksRes, s => s.0.clean_start_time));
    r.f("Clean Start Min Speed", 0, 1132068864, 5.0, 0x41F00000, live!(ScoreTweaksRes, s => s.0.clean_start_min_speed));
    r.sys("Game.ScoreManager.SlipStream");
    r.i("Entered Slipstream Score", 0, 100, 1, 20, live!(ScoreTweaksRes, s => s.0.slipstream_enter));
    r.f("Min Time in Slipstream for Score", 0, 1092616192, 0.1, 0x40000000, live!(ScoreTweaksRes, s => s.0.slipstream_min_time));
    r.i("Continuous Slipstream Score", 0, 100, 1, 1, live!(ScoreTweaksRes, s => s.0.slipstream_continuous));
    r.f("Continous Slipstream Interval", 0, 1092616192, 0.1, 0x3F800000, live!(ScoreTweaksRes, s => s.0.slipstream_interval));
    r.sys("Game.ScoreManager.RacingLine");
    r.i("Started on RacingLine Score", 0, 100, 1, 30, live!(ScoreTweaksRes, s => s.0.racing_line_start));
    r.f("Max Dist from Racing Line", 0, 1084227584, 0.01, 0x3F800000, live!(ScoreTweaksRes, s => s.0.racing_line_max_dist));
    r.f("Min Speed", 0, 1128792064, 1.0, 0x42200000, live!(ScoreTweaksRes, s => s.0.racing_line_min_speed));
    r.f("Min Time on Racing Line for Score", 0, 1092616192, 0.1, 0x40000000, live!(ScoreTweaksRes, s => s.0.racing_line_min_time));
    r.i("Continuous RacingLine Score", 0, 100, 1, 1, live!(ScoreTweaksRes, s => s.0.racing_line_continuous));
    r.f("Continous RacingLine Interval", 0, 1092616192, 0.1, 0x3F800000, live!(ScoreTweaksRes, s => s.0.racing_line_interval));
    r.sys("Game.ScoreManager.FinishPositions");
    r.i("First Place Score", 0, 200, 1, 75, live!(ScoreTweaksRes, s => s.0.place[0]));
    r.i("Second Place Score", 0, 200, 1, 50, live!(ScoreTweaksRes, s => s.0.place[1]));
    r.i("Third Place Score", 0, 200, 1, 25, live!(ScoreTweaksRes, s => s.0.place[2]));
    r.sys("Game.ScoreManager.NearMiss");
    r.i("Near Miss Score", 0, 200, 1, 25, live!(ScoreTweaksRes, s => s.0.near_miss));
    r.f("Min Near Miss Speed", 0, 1092616192, 0.1, 0x42700000, live!(ScoreTweaksRes, s => s.0.near_miss_min_speed));
    r.f("Near Miss Distance", 0, 1092616192, 0.1, 0x40B00000, live!(ScoreTweaksRes, s => s.0.near_miss_distance));
    r.sys("Game.ScoreManager.Police");
    r.f("Min Dist For Escaping Score", 0, 1132068864, 1.0, 0x42480000, live!(ScoreTweaksRes, s => s.0.escape_min_dist));
    r.f("Max Dist For Escaping Score", 0, 1132068864, 1.0, 0x42C80000, live!(ScoreTweaksRes, s => s.0.escape_max_dist));
    r.i("Escaped Police Score", 0, 200, 1, 250, live!(ScoreTweaksRes, s => s.0.escaped_police));
    r.i("Start Escaping Police Score", 0, 200, 1, 25, live!(ScoreTweaksRes, s => s.0.start_escaping));
    r.i("Continous Escaping Police Score", 0, 200, 1, 5, live!(ScoreTweaksRes, s => s.0.escaping));
    r.f("Time Between Escaping Scores", 0, 1092616192, 0.05, 0x3F800000, live!(ScoreTweaksRes, s => s.0.escaping_interval));
    r.sys("Game.ScoreManager.GearSwitches");
    r.f("Start line bad switch", 0, 1128792064, 1.0, 0, live!(ScoreTweaksRes, s => s.0.start_switch[0]));
    r.f("Start line good switch", 0, 1128792064, 1.0, 0x42480000, live!(ScoreTweaksRes, s => s.0.start_switch[1]));
    r.f("Start line perfect switch", 0, 1128792064, 1.0, 0x42C80000, live!(ScoreTweaksRes, s => s.0.start_switch[2]));
    r.f("Start line skid switch", 0, 1128792064, 1.0, 0, live!(ScoreTweaksRes, s => s.0.start_switch[3]));
    r.f("In race bad switch", 0, 1128792064, 1.0, 0, live!(ScoreTweaksRes, s => s.0.race_switch[0]));
    r.f("In race good switch", 0, 1128792064, 1.0, 0x42480000, live!(ScoreTweaksRes, s => s.0.race_switch[1]));
    r.f("In race perfect switch", 0, 1128792064, 1.0, 0x42C80000, live!(ScoreTweaksRes, s => s.0.race_switch[2]));
}

/// graphics' call sites (`gfx_tweaks::PHASES`) mapped onto the load step that runs there. 0x000B6247 is
/// GameManager 0x000B4170, which calls SpecularChars itself ([`add_game_manager_tweaks`]).
pub const GFX_SITES: &[(u32, ELoadStep)] = &[
    (0x000B503F, ELoadStep::FullScreenEffects),
    (0x000B50A2, ELoadStep::AdditionalShaderData),
    (0x000B51BC, ELoadStep::GlowInit),
    (0x000B51E5, ELoadStep::FogInit),
    (0x000B5212, ELoadStep::TrackFxInit),
    (0x000B5268, ELoadStep::FadeRenderInit),
    (0x000B5295, ELoadStep::ReflectionInit),
    (0x000B5829, ELoadStep::FeCarRender),
    (0x000B5854, ELoadStep::FeDofRender),
    (0x000B587D, ELoadStep::MtvEffects),
    (0x000B6265, ELoadStep::Tweaks),
    (0x000B62C3, ELoadStep::ProjectedLightBeams),
    (0x000B657C, ELoadStep::SlipStreamRender),
    (0x000B65A5, ELoadStep::RainRender),
    (0x000B65C3, ELoadStep::DofRenderInit),
    (0x000B65DF, ELoadStep::CMeshRender),
    (0x000B660D, ELoadStep::SkyRender),
    (0x000B6D2E, ELoadStep::Barriers),
    (0x000B6D48, ELoadStep::Barriers),
    (0x000B6E52, ELoadStep::CarRender),
    (0x000B6E85, ELoadStep::FuryFx),
];

fn gfx_at_step(r: &mut Reg, step: ELoadStep) {
    for &(site, st) in GFX_SITES {
        if st != step {
            continue;
        }
        for p in PHASES.iter().filter(|p| p.1 == site) {
            for &o in p.2 {
                r.gfx(o);
            }
        }
    }
}

/// the registrations at one load step, in the original's order
pub fn register_step(r: &mut Reg, step: ELoadStep, police: bool, in_game: bool) {
    match step {
        // InGameInit 0x000B6247 / 0x000B624C / 0x000B6251 (physics CConfig::AddMaterialTweaks) / 0x000B6256 /
        // 0x000B625B, then the fog at 0x000B6265
        ELoadStep::Tweaks => {
            add_game_manager_tweaks(r);
            add_point_of_interest_tweaks(r);
            r.ext(ExtSrc::Phys, 0x0019BEC0);
            add_crash_trigger_tweaks(r);
            add_crash_camera_tweaks(r);
        }
        // CommonInit 0x000B5169 -> VsAudio::Init -> AddTweaks 0x000484D0 (returns at once for the front end's Init)
        ELoadStep::AudioInit if in_game => {
            for o in velocity_audio::audio_tweaks::ADD_TWEAKS_ORDER {
                r.ext(ExtSrc::Audio, o);
            }
        }
        // PhysicsManager::Init (0x000B679E): PhysicsCarInput 0x001B8A00, then VehicleManager 0x001DC650
        ELoadStep::PhysicsManagerInit => {
            r.ext(ExtSrc::Phys, 0x001B8A00);
            r.ext(ExtSrc::Phys, 0x001DC650);
        }
        // CompartmentManager's init (vtable 0x0062BB7C slot 0)
        ELoadStep::CompartmentPreInit => add_compartment_tweaks(r),
        // VsCameraManager 0x0005ECC0: 0x0005EDBE, then 0x0005F04D
        ELoadStep::CameraManager => {
            add_follow_camera_globals_tweaks(r);
            add_fly_camera_tweaks(r);
        }
        // 0x00017F60 per vehicle (the local ones register; insert dedups the rest), police cars' CarDamage
        ELoadStep::AiWorldData => {
            add_score_manager_tweaks(r);
            if police {
                for p in PHASES.iter().filter(|p| p.1 == 0x0001808D) {
                    for &o in p.2 {
                        r.gfx(o);
                    }
                }
            }
        }
        ELoadStep::ReplayCameraManager => add_replay_camera_tweaks(r),
        // StringManager 0x000C90A0 (heat haze load)
        ELoadStep::HeatHazeInit => add_heat_haze_tweaks(r),
        _ => {}
    }
    gfx_at_step(r, step);
}

/// police cars exist (VsAIWorld::Init flags, as `race_cars::spawn_traffic_and_police`)
fn race_has_police(gm: &GameManager) -> bool {
    let Some(race) = &gm.race else { return false };
    let ty = crate::game_mode::race_cars::race_area_and_type(gm).1;
    let traffic = race.traffic && !gm.vars.network_race;
    (traffic || ty == 6) && ty != 5
}

/// registrations at their load steps (Last, after `mode_messages` rebuilt the tree on `TweaksReset`)
pub fn core_tweak_steps(
    world: &mut World,
    mut steps: Local<MessageCursor<LoadStepMessage>>,
    mut modes: Local<MessageCursor<GameModeMessage>>,
) {
    let reset = world
        .get_resource::<Messages<GameModeMessage>>()
        .is_some_and(|m| modes.read(m).any(|m| *m == GameModeMessage::TweaksReset));
    if reset {
        let mut ct = world.resource_mut::<CoreTweaks>();
        ct.cells.clear();
        ct.ext.clear();
    }
    let list: Vec<ELoadStep> = match world.get_resource::<Messages<LoadStepMessage>>() {
        Some(m) => steps.read(m).map(|m| m.step).collect(),
        None => return,
    };
    if list.is_empty() {
        return;
    }
    world.resource_scope(|world, mut gm: Mut<GameManager>| {
        world.resource_scope(|world, mut ct: Mut<CoreTweaks>| {
            let police = race_has_police(&gm);
            let in_game = gm.mode == Some(1);
            let mut r = Reg::new(&mut gm.tweaks, &mut ct, Some(world));
            for s in list {
                register_step(&mut r, s, police, in_game);
            }
        });
    });
}

/// live variables -> cells (before the debug menu)
pub fn core_tweaks_in(world: &mut World) {
    world.resource_scope(|world, ct: Mut<CoreTweaks>| {
        for (cell, live) in &ct.cells {
            if let Some(v) = live.and_then(|l| (l.get)(world)) {
                if cell.get() != v {
                    cell.set(v);
                }
            }
        }
        for (src, slot, cell) in &ct.ext {
            if let Some(v) = ext_get(world, *src, *slot) {
                if cell.get() != v {
                    cell.set(v);
                }
            }
        }
    });
}

/// cells -> live variables (after the debug menu)
pub fn core_tweaks_out(world: &mut World) {
    world.resource_scope(|world, ct: Mut<CoreTweaks>| {
        for (cell, live) in &ct.cells {
            if let Some(l) = live {
                (l.set)(world, &cell.get());
            }
        }
        for (src, slot, cell) in &ct.ext {
            ext_put(world, *src, *slot, cell);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unquote(s: &str) -> String {
        let s = s.trim_start_matches("(int)").trim();
        s.trim_matches('"').replace("\\t", "\t")
    }
    fn int(s: &str) -> i64 {
        let s = s.trim().trim_end_matches('u');
        match s.strip_prefix("0x") {
            Some(h) => i64::from_str_radix(h, 16).unwrap(),
            None => s.parse().unwrap(),
        }
    }
    fn step(s: &str) -> u32 {
        s.trim().trim_end_matches('f').parse::<f32>().unwrap().to_bits()
    }

    /// every registration against the original's calls (`research/core_tweaks.tsv`): order, names, ranges, steps
    /// and the .data defaults
    #[test]
    fn registrations_match_the_original_calls() {
        let tsv = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/core_tweaks.tsv"));
        let Ok(tsv) = tsv else { return };
        let mut tm = CTweakManager::default();
        let mut ct = CoreTweaks::default();
        let mut r = Reg::new(&mut tm, &mut ct, None);
        for f in [
            add_crash_camera_tweaks,
            add_crash_trigger_tweaks,
            add_fly_camera_tweaks,
            add_follow_camera_globals_tweaks,
            add_point_of_interest_tweaks,
            add_compartment_tweaks,
            add_game_manager_tweaks,
            add_heat_haze_tweaks,
            add_network_tweaks,
            add_replay_camera_tweaks,
            add_score_manager_tweaks,
        ] {
            f(&mut r);
        }
        let mut ours = std::mem::take(&mut r.log);
        add_network_timing_tweaks(&mut r);
        ours.extend(r.log.drain(..).filter(|(o, _)| *o == 0xCF200));
        let mut checked = 0;
        for origin in [0x61D30u32, 0x63160, 0x65640, 0x65920, 0x6A930, 0x87010, 0xB4170, 0xC9450, 0xCDD70, 0xCF200, 0x165640, 0x167F30, 0x1EDA50] {
            let mut want: Vec<Vec<&str>> = tsv
                .lines()
                .map(|l| l.split('\t').collect::<Vec<_>>())
                .filter(|c| u32::from_str_radix(c[0], 16).unwrap() == origin)
                .collect();
            if origin == 0x1EDA50 {
                want = (0..8).flat_map(|_| want.clone()).collect();
            }
            let got: Vec<&Rec> = ours.iter().filter(|(o, _)| *o == origin).map(|(_, r)| r).collect();
            assert_eq!(got.len(), want.len(), "{origin:x}: call count");
            for (i, (g, w)) in got.iter().zip(&want).enumerate() {
                let known = w.get(5).is_some_and(|d| d.len() == 8) && !(origin == 0x1EDA50 && i >= 4);
                let def = w.get(5).and_then(|d| u32::from_str_radix(d, 16).ok()).unwrap_or(0);
                match (w[1], g) {
                    ("S", Rec::S(p)) => {
                        if origin == 0x1EDA50 {
                            assert!(p.starts_with("controller.vibrate."));
                        } else {
                            assert_eq!(*p, unquote(w[2]), "{origin:x}");
                        }
                    }
                    ("F", Rec::F(n, lo, hi, st, d)) => {
                        let a: Vec<&str> = w[4].split('|').collect();
                        assert_eq!((n.as_str(), *lo, *hi, *st), (unquote(w[2]).as_str(), int(a[0]) as u32, int(a[1]) as u32, step(a[2])), "{origin:x}");
                        if known {
                            assert_eq!(*d, def, "{origin:x} {n} default");
                        }
                    }
                    ("I", Rec::I(n, lo, hi, st, d)) => {
                        let a: Vec<&str> = w[4].split('|').collect();
                        assert_eq!((n.clone(), *lo as i64, *hi as i64, *st as i64), (unquote(w[2]), int(a[0]), int(a[1]), int(a[2])), "{origin:x}");
                        if known {
                            assert_eq!(*d as u32, def, "{origin:x} {n} default");
                        }
                    }
                    ("B", Rec::B(n, d)) => {
                        assert_eq!(*n, unquote(w[2]), "{origin:x}");
                        if known {
                            assert_eq!(*d, def & 0xFF != 0, "{origin:x} {n} default");
                        }
                    }
                    ("E", Rec::E(n, el, d)) => {
                        assert_eq!((n.clone(), el.clone()), (unquote(w[2]), unquote(w[4])), "{origin:x}");
                        if known {
                            assert_eq!(*d as u32, def, "{origin:x} {n} default");
                        }
                    }
                    (k, g) => panic!("{origin:x} #{i}: original {k} {:?}, port {g:?}", w),
                }
                checked += 1;
            }
        }
        assert!(checked > 280, "{checked}");
        // the tree: unique systems, values readable through find_tweak
        let s = tm.find_system("Game.ScoreManager.Police").unwrap();
        let id = tm.find_tweak(s, "Escaped Police Score").unwrap();
        assert_eq!(tm.tweak(id).cell.i32(), 250);
    }

    #[test]
    fn live_binding_round_trips() {
        let mut w = World::new();
        w.insert_resource(ScoreTweaksRes::default());
        w.insert_resource(CoreTweaks::default());
        w.insert_resource(GameCameras::default());
        let mut tm = CTweakManager::default();
        let mut ct = CoreTweaks::default();
        {
            let mut r = Reg::new(&mut tm, &mut ct, Some(&w));
            add_score_manager_tweaks(&mut r);
            add_replay_camera_tweaks(&mut r);
        }
        w.insert_resource(ct);
        let s = tm.find_system("Game.ScoreManager.Overtake").unwrap();
        let id = tm.find_tweak(s, "Overtake Score").unwrap();
        tm.tweak_mut(id).increment(1.0);
        core_tweaks_out(&mut w);
        assert_eq!(w.resource::<ScoreTweaksRes>().0.overtake, 21);
        w.resource_mut::<GameCameras>().replay.review_cameras = 2;
        core_tweaks_in(&mut w);
        let s = tm.find_system("Replay Cameras").unwrap();
        let id = tm.find_tweak(s, "Enable Cameras For Review").unwrap();
        assert_eq!(tm.tweak(id).cell.i32(), 2);
    }
}
