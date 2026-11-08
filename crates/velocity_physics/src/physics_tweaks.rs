//! the physics subsystem's three `AddTweaks` registrations (docs/physics.md "Debug-menu tweaks") as recorded
//! `CTweakManager` call lists, plus one value slot per tweak that mirrors the executable global it edits.
//!
//! velocity_physics cannot see velocity_game's `CTweakManager` (velocity_game depends on this crate), so core
//! replays these calls (`calls(origin)` at the call sites in `PHASES`), keeps one `TweakCell` per slot and copies the
//! cells to / from `PhysicsTweaks.values` around the debug menu, as for `visos_render::gfx_tweaks`.
//! `sync_physics_tweaks` then writes changed slots into the globals and reads game-side writes back every frame.
//!
//! Every tweaked variable is an executable global. Process-wide values live in [`TWEAKED`] (the original's .data
//! survives races): a new `PhysicsWorld` is seeded from it (its `PhysGlobals` start from the xbe image), and the
//! hand-ported readers outside the world (`SVehicleSetup` modifiers) read it through [`global`].

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use bevy::prelude::*;

use crate::material_properties::{DEFAULT_MATERIALS, SURF_NAMES, SURF_TYPE_MAX_TYPES};

/// a tweak value (`TweakData` without strings)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PhysTweakValue {
    Float(f32),
    Int(i32),
    Bool(bool),
}

impl PhysTweakValue {
    pub fn f32(self) -> f32 {
        match self {
            PhysTweakValue::Float(f) => f,
            PhysTweakValue::Int(i) => i as f32,
            PhysTweakValue::Bool(b) => b as i32 as f32,
        }
    }
    pub fn i32(self) -> i32 {
        match self {
            PhysTweakValue::Float(f) => f as i32,
            PhysTweakValue::Int(i) => i,
            PhysTweakValue::Bool(b) => b as i32,
        }
    }
    pub fn bool(self) -> bool {
        self.i32() != 0
    }
    /// the value as the global's word (`old` = the word now; a bool is the low byte of it)
    fn to_word(self, old: u32) -> u32 {
        match self {
            PhysTweakValue::Float(f) => f.to_bits(),
            PhysTweakValue::Int(i) => i as u32,
            PhysTweakValue::Bool(b) => (old & !0xFF) | b as u32,
        }
    }
    /// the global's word read as `like`'s variant
    fn from_word(like: PhysTweakValue, w: u32) -> PhysTweakValue {
        match like {
            PhysTweakValue::Float(_) => PhysTweakValue::Float(f32::from_bits(w)),
            PhysTweakValue::Int(_) => PhysTweakValue::Int(w as i32),
            PhysTweakValue::Bool(_) => PhysTweakValue::Bool(w & 0xFF != 0),
        }
    }
}

/// one recorded `CTweakManager` call
#[derive(Clone, Debug, PartialEq)]
pub enum PhysTweakCall {
    /// createSystem (0x00086250) by dotted path below the root; the following tweaks go there
    System(String),
    /// addTweak(float) 0x00085A90
    Float { name: &'static str, slot: usize, min: f32, max: f32, step: f32 },
    /// addTweak(S32) 0x00085B40
    Int { name: &'static str, slot: usize, min: i32, max: i32, step: i32 },
    /// addTweak(bool) 0x00085CA0
    Bool { name: &'static str, slot: usize },
}

impl PhysTweakCall {
    pub fn slot(&self) -> Option<usize> {
        match self {
            PhysTweakCall::System(_) => None,
            PhysTweakCall::Float { slot, .. } | PhysTweakCall::Int { slot, .. } | PhysTweakCall::Bool { slot, .. } => Some(*slot),
        }
    }
}

/// where the original calls a registration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysTweakPhase {
    /// GameManager::InGameInit 0x000B5F80
    InGameInit,
}

/// (phase, call site in the phase function, registration origins in call order). 0x000B679E = PhysicsManager::Init
/// 0x0019CB78 -> PhysicsManager 0x0019E080 0x0019EFF8 -> VehicleManager 0x001DC650, which calls PhysicsCarInput
/// 0x001B8A00 first and then adds "Debug Cameras" itself.
pub const PHASES: &[(PhysTweakPhase, u32, &[u32])] = &[
    (PhysTweakPhase::InGameInit, 0x000B6251, &[0x0019BEC0]),
    (PhysTweakPhase::InGameInit, 0x000B679E, &[0x001B8A00, 0x001DC650]),
];

/// slots whose global is not written into the world: setting them makes VehicleEntity 0x001CB530 reload the car's
/// config through CConfig 0x00199DC0, which the translation does not model (it would panic); the menu still edits
/// the slot. `Config version` 0x0075B28C, `Reload Handling.cfg` 0x0075B288.
pub const NOT_PUSHED: &[u32] = &[0x0075_B28C, 0x0075_B288];

/// one registration function
#[derive(Clone, Debug)]
pub struct PhysTweakFn {
    pub origin: u32,
    pub calls: Vec<PhysTweakCall>,
}

/// process-wide words of the tweaked globals once synced (address -> word); seeds every new physics world
pub static TWEAKED: LazyLock<Mutex<HashMap<i32, u32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// the synced word of a tweaked global (None = still its .data value)
pub fn global(addr: u32) -> Option<u32> {
    TWEAKED.lock().unwrap_or_else(|e| e.into_inner()).get(&(addr as i32)).copied()
}

/// a tweaked float global, or `default` (its .data value) when never synced
pub fn global_f32(addr: u32, default: f32) -> f32 {
    global(addr).map_or(default, f32::from_bits)
}

/// copy the synced tweak globals into a new world's globals (`PhysicsWorld::new`)
pub fn seed_globals(w: &mut HashMap<i32, u32>) {
    for (a, v) in TWEAKED.lock().unwrap_or_else(|e| e.into_inner()).iter() {
        if !NOT_PUSHED.contains(&(*a as u32)) {
            w.insert(*a, *v);
        }
    }
}

/// every physics registration and the value slots their tweaks point at
#[derive(Resource, Clone, Debug)]
pub struct PhysicsTweaks {
    pub fns: Vec<PhysTweakFn>,
    /// the tweaked variables (core's cells copy in / out of these)
    pub values: Vec<PhysTweakValue>,
    /// the executable global behind each slot
    pub addrs: Vec<u32>,
    /// the values last synced with the globals
    applied: Vec<PhysTweakValue>,
}

impl PhysicsTweaks {
    /// the recorded calls of the registration at `origin`
    pub fn calls(&self, origin: u32) -> Option<&[PhysTweakCall]> {
        self.fns.iter().find(|f| f.origin == origin).map(|f| &f.calls[..])
    }

    /// the registration origins of one phase, in call order
    pub fn phase_origins(phase: PhysTweakPhase) -> Vec<u32> {
        PHASES.iter().filter(|p| p.0 == phase).flat_map(|p| p.2.iter().copied()).collect()
    }

    /// the slot of the tweak at a global
    pub fn slot_of(&self, addr: u32) -> Option<usize> {
        self.addrs.iter().position(|a| *a == addr)
    }

    /// one sync against the globals: slots the menu changed are written to `overlay` and `world` (when present);
    /// otherwise a global the game wrote (seen in `world`) is read back into the slot
    pub fn sync(&mut self, overlay: &mut HashMap<i32, u32>, mut world: Option<&mut HashMap<i32, u32>>, word: &dyn Fn(u32) -> u32) {
        for s in 0..self.values.len() {
            let a = self.addrs[s];
            if self.values[s] != self.applied[s] {
                let old = overlay.get(&(a as i32)).copied().unwrap_or_else(|| word(a));
                let w = self.values[s].to_word(old);
                overlay.insert(a as i32, w);
                if let Some(g) = world.as_deref_mut() {
                    if !NOT_PUSHED.contains(&a) {
                        g.insert(a as i32, w);
                    }
                }
                self.applied[s] = self.values[s];
            } else if world.is_some() && !NOT_PUSHED.contains(&a) {
                let v = PhysTweakValue::from_word(self.values[s], word(a));
                if v != self.values[s] {
                    overlay.insert(a as i32, word(a));
                    self.values[s] = v;
                    self.applied[s] = v;
                }
            }
        }
    }
}

impl Default for PhysicsTweaks {
    fn default() -> Self {
        let mut r = Reg { t: PhysicsTweaks { fns: Vec::new(), values: Vec::new(), addrs: Vec::new(), applied: Vec::new() }, calls: Vec::new() };
        c_config_add_material_tweaks(&mut r);
        r.end(0x0019BEC0);
        physics_car_input_1b8a00(&mut r);
        r.end(0x001B8A00);
        vehicle_manager_1dc650(&mut r);
        r.end(0x001DC650);
        let mut t = r.t;
        t.applied = t.values.clone();
        t
    }
}

/// registration recorder
struct Reg {
    t: PhysicsTweaks,
    calls: Vec<PhysTweakCall>,
}

impl Reg {
    fn end(&mut self, origin: u32) {
        let calls = std::mem::take(&mut self.calls);
        self.t.fns.push(PhysTweakFn { origin, calls });
    }
    fn slot(&mut self, v: PhysTweakValue, addr: u32) -> usize {
        self.t.values.push(v);
        self.t.addrs.push(addr);
        self.t.values.len() - 1
    }
    fn sys(&mut self, path: &str) {
        self.calls.push(PhysTweakCall::System(path.to_string()));
    }
    /// float tweak: min / max / step / .data value as the bits the original pushes
    fn f(&mut self, name: &'static str, addr: u32, min: u32, max: u32, step: u32, init: u32) {
        let slot = self.slot(PhysTweakValue::Float(f32::from_bits(init)), addr);
        let (min, max, step) = (f32::from_bits(min), f32::from_bits(max), f32::from_bits(step));
        self.calls.push(PhysTweakCall::Float { name, slot, min, max, step });
    }
    fn i(&mut self, name: &'static str, addr: u32, min: i32, max: i32, step: i32, init: i32) {
        let slot = self.slot(PhysTweakValue::Int(init), addr);
        self.calls.push(PhysTweakCall::Int { name, slot, min, max, step });
    }
    fn b(&mut self, name: &'static str, addr: u32, init: bool) {
        let slot = self.slot(PhysTweakValue::Bool(init), addr);
        self.calls.push(PhysTweakCall::Bool { name, slot });
    }
}

/// `g_materialProperties` 0x006879C0 (12 bytes per eSURF type)
const MATERIALS: u32 = 0x006879C0;

/// CConfig.cpp 0x0019BEC0
fn c_config_add_material_tweaks(r: &mut Reg) {
    r.sys("Materials");
    // eSURF 0 (DEFAULT) has no tweaks
    for i in 1..SURF_TYPE_MAX_TYPES {
        let m = &DEFAULT_MATERIALS[i];
        let a = MATERIALS + 12 * i as u32;
        r.sys(&format!("Materials.{}", SURF_NAMES[i]));
        r.f("Friction multiplier", a, 0, 0x3F800000, 0x3D4CCCCD, m.friction_multiplier.to_bits());
        r.f("Drag", a + 4, 0, 0x3E4CCCCD, 0x3A83126F, m.drag.to_bits());
        r.f("Bump map height", a + 8, 0, 0x3E4CCCCD, 0x3BA3D70A, m.bump_map_height.to_bits());
    }
}

/// PhysicsCarInput.cpp 0x001B8A00 (also clears `dword_75B280`, already 0 in a new world)
fn physics_car_input_1b8a00(r: &mut Reg) {
    r.sys("JumpCam");
    r.f("JumpUpdateRate", 0x6889F8, 0x3F800000, 0x41C00000, 0x3E4CCCCD, 0x41900000);
    r.f("PreLandingCutTime", 0x6889FC, 0, 0x3F800000, 0x3C23D70A, 0x3DCCCCCD);
    r.f("JumpCamSlowdownMax", 0x688A00, 0, 0x3F800000, 0x3C23D70A, 0x3F333333);
    r.f("JumpUpdateStartRate", 0x688A04, 0, 0x41C00000, 0x3F000000, 0x40800000);
    r.f("SlipstreamEffect", 0x688A08, 0, 0x40A00000, 0x3DCCCCCD, 0x3F800000);
    r.f("AddSlipstreamAtTimerPercent", 0x688A0C, 0, 0x3F800000, 0x3D4CCCCD, 0x3E4CCCCD);
    // read by the net code (not ported) and VehicleEntity 0x001C1FF0 / 0x001C2EF0 / 0x001C51D0 (network cars)
    r.sys("Network");
    r.b("StopTransmitting", 0x75B285, false);
    r.b("DisableExtrapolationDelay", 0x75B286, false);
    r.b("SmoothPrediction", 0x688A24, true);
    r.f("SmoothAmount", 0x688A28, 0, 0x3F800000, 0x3DCCCCCD, 0x3F000000);
    r.b("DebugOutput", 0x75B287, false);
    r.f("SmoothValue", 0x688A2C, 0, 0x40000000, 0x3D4CCCCD, 0x3ECCCCCD);
    r.f("SmoothValue2", 0x688A30, 0, 0x40000000, 0x3D4CCCCD, 0x3F333333);
    r.f("SmoothValue3", 0x688A34, 0, 0x40000000, 0x3D4CCCCD, 0x3F666666);
    r.f("MaxTimeBeforeInvalidPrediction", 0x685F68, 0, 0x40A00000, 0x3DCCCCCD, 0x40000000);
    r.f("MaxDistBeforeWarpingSq", 0x688A38, 0, 0x43C80000, 0x40A00000, 0x41C80000);
    r.sys("Handling");
    r.b("Reload Handling.cfg", 0x75B288, false);
    r.i("Config version (0-standard, other-alternatives)", 0x75B28C, 0, 2, 1, 0);
    r.sys("Handling.Performance");
    r.f("Engine/Turbo Accel Influence", 0x688A74, 0, 0x3F800000, 0x3D4CCCCD, 0x3F333333);
    r.f("Engine/Turbo TopSpeed Influence", 0x688A78, 0, 0x3F800000, 0x3D4CCCCD, 0x3F333333);
    r.sys("Handling.Tyres");
    r.i("Tyre type (soft-1,med.-2,hard-3)", 0x688A7C, 1, 3, 1, 2);
    r.sys("EngineSequencer");
    r.i("RPM Curve Exponent", 0x688A6C, 2, 10, 1, 2);
    r.f("RPM Curve", 0x688A68, 0, 0x3F800000, 0x3D4CCCCD, 0x3ECCCCCD);
    r.f("Non Skid RPM Blend", 0x688A70, 0, 0x3F800000, 0x3D4CCCCD, 0x3F000000);
    // read by PhysicsObjectManager 0x001E0710 / 0x001E0EE0 (render passes, not ported)
    r.sys("PhysicsObjects");
    r.b("Use Axis Sort List for Rendering", 0x688B90, true);
    r.sys("Turbo");
    r.f("Rise rate", 0x688A80, 0, 0x3F800000, 0x3D4CCCCD, 0x3E19999A);
    r.f("Fall rate", 0x688A84, 0, 0x3F800000, 0x3D4CCCCD, 0x3F000000);
    r.f("Throttle threshold", 0x688A88, 0, 0x3F800000, 0x3D4CCCCD, 0x3F666666);
    r.f("Max revs", 0x688A8C, 0, 0x461C4000, 0x42C80000, 0x459C4000);
    r.f("Min revs", 0x688A90, 0, 0x461C4000, 0x42C80000, 0x44BB8000);
    r.f("Torque effect", 0x688A94, 0, 0x41200000, 0x3C23D70A, 0x3E800000);
    r.f("Effect start", 0x688A98, 0, 0x3F800000, 0x3C23D70A, 0x3F59999A);
    r.sys("StartLineBehaviour");
    r.f("Turn Rate", 0x688A10, 0x3A83126F, 0x3DCCCCCD, 0x3A83126F, 0x3BA3D70A);
    r.f("Maximum Turn", 0x688A14, 0x3D4CCCCD, 0x3F000000, 0x3D4CCCCD, 0x3DCCCCCD);
    r.f("Oscillation Rate", 0x688A18, 0x3A03126F, 0x3D4CCCCD, 0x3A03126F, 0x3B03126F);
    r.f("Max Oscillation", 0x688A1C, 0x3B03126F, 0x3E4CCCCD, 0x3B03126F, 0x3CA3D70A);
    r.sys("SkidVibrate");
    r.f("Time to max intensity", 0x688A9C, 0x3DCCCCCD, 0x40A00000, 0x3DCCCCCD, 0x3FC00000);
}

/// VehicleManager.cpp 0x001DC650 (the tweak part; it calls 0x001B8A00 first). `dword_688B80` is read by SyncModels
/// 0x001DDAE0 / 0x001DDBC0 (which car the debug camera follows; core's camera)
fn vehicle_manager_1dc650(r: &mut Reg) {
    r.sys("Debug Cameras");
    r.i("Follow vehicle (-1 defaults to player)", 0x688B80, -1, 6, 1, -1);
}

/// First: push slots the debug menu changed into the tweaked globals (process-wide and the current world) and pull
/// back what the game wrote
#[cfg(feature = "tr")]
pub fn sync_physics_tweaks(mut pt: ResMut<PhysicsTweaks>, world: Res<crate::plugin::PhysicsWorldRes>) {
    let mut overlay = TWEAKED.lock().unwrap_or_else(|e| e.into_inner());
    let mut w = world.lock();
    match w.as_mut() {
        Some(pw) => {
            let snap: HashMap<u32, u32> = pt.addrs.iter().map(|a| (*a, pw.glob.word(*a as i32))).collect();
            pt.sync(&mut overlay, Some(&mut pw.glob.w), &|a| snap.get(&a).copied().unwrap_or(0));
        }
        None => {
            let snap = overlay.clone();
            pt.sync(&mut overlay, None, &|a| snap.get(&(a as i32)).copied().unwrap_or(0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `research/phys_tweak_regs.tsv` (`tools/ida/phys_tweak_tsv.py`): fn, call ea, kind, name, var, a0, a1, a2, init
    fn dump() -> Option<String> {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../research/phys_tweak_regs.tsv");
        std::fs::read_to_string(p).ok()
    }

    /// every registration matches the pushes of the binary's calls: order, systems, kinds, names, ranges, steps
    /// (bit-exact) and every global's .data default (the material loop is unrolled against the table at 0x006879C0)
    #[test]
    fn registrations_match_the_binary() {
        let Some(tsv) = dump() else {
            eprintln!("research/phys_tweak_regs.tsv missing: skipped");
            return;
        };
        let pt = PhysicsTweaks::default();
        assert_eq!(pt.fns.len(), 3);
        let mut n = 0;
        for f in &pt.fns {
            let rows: Vec<Vec<&str>> = tsv
                .lines()
                .map(|l| l.split('\t').collect::<Vec<_>>())
                .filter(|c| u32::from_str_radix(c[0], 16).unwrap() == f.origin)
                .collect();
            // 0x0019BEC0: "Materials", then the loop body (system + 3 floats) for eSURF 1..25
            let rows: Vec<Vec<String>> = if f.origin == 0x0019BEC0 {
                let mut v = vec![rows[0].iter().map(|s| s.to_string()).collect::<Vec<_>>()];
                for i in 1..SURF_TYPE_MAX_TYPES {
                    v.push(vec![String::new(), String::new(), "S".into(), format!("\"Materials.{}\"", SURF_NAMES[i])]);
                    for (k, row) in rows[2..5].iter().enumerate() {
                        let mut r: Vec<String> = row.iter().map(|s| s.to_string()).collect();
                        let a = MATERIALS + 12 * i as u32 + 4 * k as u32;
                        r[4] = format!("{a:X}");
                        r[8] = format!("{:08X}", [DEFAULT_MATERIALS[i].friction_multiplier, DEFAULT_MATERIALS[i].drag, DEFAULT_MATERIALS[i].bump_map_height][k].to_bits());
                        v.push(r);
                    }
                }
                v
            } else {
                rows.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect()
            };
            assert_eq!(rows.len(), f.calls.len(), "{:x}: call count", f.origin);
            for (c, row) in f.calls.iter().zip(&rows) {
                let name = row[3].trim_matches('"');
                let hex = |s: &str| u32::from_str_radix(s, 16).unwrap();
                let init = |s: &str| if s == "bss" { 0 } else { hex(s) };
                match c {
                    PhysTweakCall::System(p) => assert_eq!((row[2].as_str(), p.as_str()), ("S", name)),
                    PhysTweakCall::Float { name: nm, slot, min, max, step } => {
                        assert_eq!((row[2].as_str(), *nm), ("F", name));
                        assert_eq!((min.to_bits(), max.to_bits(), step.to_bits()), (hex(&row[5]), hex(&row[6]), hex(&row[7])), "{nm}");
                        assert_eq!(pt.addrs[*slot], hex(&row[4]), "{nm} variable");
                        assert_eq!(pt.values[*slot].f32().to_bits(), init(&row[8]), "{nm} default");
                    }
                    PhysTweakCall::Int { name: nm, slot, min, max, step } => {
                        assert_eq!((row[2].as_str(), *nm), ("I", name));
                        assert_eq!((*min, *max, *step), (hex(&row[5]) as i32, hex(&row[6]) as i32, hex(&row[7]) as i32), "{nm}");
                        assert_eq!(pt.addrs[*slot], hex(&row[4]), "{nm} variable");
                        assert_eq!(pt.values[*slot].i32(), init(&row[8]) as i32, "{nm} default");
                    }
                    PhysTweakCall::Bool { name: nm, slot } => {
                        assert_eq!((row[2].as_str(), *nm), ("B", name));
                        assert_eq!(pt.addrs[*slot], hex(&row[4]), "{nm} variable");
                        assert_eq!(pt.values[*slot].bool(), init(&row[8]) & 0xFF != 0, "{nm} default");
                    }
                }
                n += 1;
            }
        }
        // 1 + 25 * 4 material calls, 47 in 0x001B8A00 (10 systems), 2 in 0x001DC650
        assert_eq!(n, 101 + 47 + 2);
        assert_eq!(PhysicsTweaks::phase_origins(PhysTweakPhase::InGameInit), vec![0x0019BEC0, 0x001B8A00, 0x001DC650]);
    }

    /// a menu edit reaches the global (overlay + world), a game write comes back, the reload pair stays out of the world
    #[test]
    fn sync_pushes_and_pulls_globals() {
        let mut pt = PhysicsTweaks::default();
        let mut overlay = HashMap::new();
        let mut world: HashMap<i32, u32> = HashMap::new();
        // the world's word: written, else the .data value (= the registration default)
        let defaults = PhysicsTweaks::default();
        let def = move |a: u32| defaults.values[defaults.slot_of(a).unwrap()].to_word(0);
        let drag = pt.slot_of(MATERIALS + 12 * 11 + 4).unwrap();
        pt.values[drag] = PhysTweakValue::Float(0.1);
        let reload = pt.slot_of(0x75B288).unwrap();
        pt.values[reload] = PhysTweakValue::Bool(true);
        let snap = world.clone();
        pt.sync(&mut overlay, Some(&mut world), &|a| snap.get(&(a as i32)).copied().unwrap_or_else(|| def(a)));
        assert_eq!(world.get(&((MATERIALS + 12 * 11 + 4) as i32)), Some(&0.1f32.to_bits()));
        assert_eq!(overlay.get(&((MATERIALS + 12 * 11 + 4) as i32)), Some(&0.1f32.to_bits()));
        assert!(!world.contains_key(&0x75B288));
        assert_eq!(world.len(), 1);
        // the game writes "Tyre type" = 3
        world.insert(0x688A7C, 3);
        let snap = world.clone();
        pt.sync(&mut overlay, Some(&mut world), &|a| snap.get(&(a as i32)).copied().unwrap_or_else(|| def(a)));
        let tyre = pt.slot_of(0x688A7C).unwrap();
        assert_eq!(pt.values[tyre], PhysTweakValue::Int(3));
        assert_eq!(pt.values[drag], PhysTweakValue::Float(0.1));
        assert_eq!(overlay.get(&0x688A7C), Some(&3));
    }
}
