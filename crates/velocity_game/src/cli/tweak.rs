//! src\Cli\tweak.cpp — `CTweakManager` / `CTweakSystem` / `CTweakValue`: named, typed, ranged debug values grouped
//! in a dotted hierarchy under the root system `$`. The debug build has no console to read `tweak.set` lines back
//! (`autoexec.cli` is a leftover); values are edited from the in-game tweak menu and exported to `Tweaks.txt`.
//!
//! the original points each tweak at the game's own variable; here the variable is a shared `TweakCell` the owner
//! keeps and reads.

use std::sync::{Arc, Mutex};

use visos::handy::handy_crc;
use visos::type_::vs_type::{CEnvironment, CTypeKind};

/// default step when none is given (0x006853E0)
pub const DEFAULT_STEP: f32 = f32::from_bits(0x3DCC_CCCD);
/// `sizeof(name) - 1` for systems and values
pub const MAX_NAME: usize = 63;

/// the value a tweak edits.
#[derive(Clone, Debug, PartialEq)]
pub enum TweakData {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
}

/// shared storage standing in for the tweaked variable.
#[derive(Clone, Debug)]
pub struct TweakCell(Arc<Mutex<TweakData>>);

impl TweakCell {
    pub fn new(v: TweakData) -> Self {
        TweakCell(Arc::new(Mutex::new(v)))
    }
    pub fn get(&self) -> TweakData {
        self.0.lock().unwrap().clone()
    }
    pub fn set(&self, v: TweakData) {
        *self.0.lock().unwrap() = v;
    }
    pub fn f32(&self) -> f32 {
        match self.get() {
            TweakData::Float(f) => f,
            TweakData::Int(i) => i as f32,
            TweakData::Bool(b) => b as i32 as f32,
            TweakData::String(_) => 0.0,
        }
    }
    pub fn i32(&self) -> i32 {
        match self.get() {
            TweakData::Int(i) => i,
            TweakData::Float(f) => f as i32,
            TweakData::Bool(b) => b as i32,
            TweakData::String(_) => 0,
        }
    }
    pub fn bool(&self) -> bool {
        self.i32() != 0
    }
}

/// `CTweakValue` (0x70 bytes).
#[derive(Debug)]
pub struct CTweakValue {
    pub name: String, // +0x0C
    pub kind: CTypeKind,
    /// enum elements (name, value) for `CTypeKind::Enum`
    pub enumerators: Vec<(String, i32)>,
    pub cell: TweakCell,  // +0x50
    pub min: TweakData,   // +0x54
    pub max: TweakData,   // +0x58
    pub step: TweakData,  // +0x5C
    pub default: TweakData, // +0x60
    /// current enum element index (+0x64)
    pub enum_current: usize,
    /// HANDY_CRC32 (case-sensitive) of the name (+0x68)
    pub name_crc: u32,
    /// peeks are read-only (+0x6C)
    pub read_only: bool,
}

impl CTweakValue {
    fn new(name: &str, kind: CTypeKind, cell: TweakCell, read_only: bool) -> Self {
        let name: String = name.chars().take(MAX_NAME).collect();
        CTweakValue {
            name_crc: crc(&name),
            name,
            kind,
            enumerators: Vec::new(),
            default: cell.get(),
            cell,
            min: TweakData::Int(0),
            max: TweakData::Int(0),
            step: TweakData::Int(0),
            enum_current: 0,
            read_only,
        }
    }

    /// tweak.cpp 0x00084290 — clamp to [min, max] and snap to the step.
    pub fn set_value(&mut self, mut v: f32) {
        assert!(!self.read_only, "!m_bReadOnly");
        assert!(matches!(self.kind, CTypeKind::Float | CTypeKind::Integer), "Only valid for Float and Integer types");
        let min = as_f32(&self.min);
        let max = as_f32(&self.max);
        let mut step = as_f32(&self.step);
        if min > v {
            v = min;
        }
        if v > max {
            v = max;
        }
        // original bug: tests `!= 0` so any explicit step is replaced by 0.1 here
        if step != 0.0 {
            step = DEFAULT_STEP;
        }
        let d = v - min;
        match self.kind {
            CTypeKind::Float => self.cell.set(TweakData::Float(d / step * step + min)),
            CTypeKind::Integer => {
                step = step.ceil();
                self.cell.set(TweakData::Int(cvttss2si(d / step * step + min)));
            }
            _ => unreachable!("Unknown type of tweak value"),
        }
    }

    /// tweak.cpp 0x000846A0
    pub fn increment(&mut self, amount: f32) {
        self.step_by(amount, 1);
    }

    /// tweak.cpp 0x000848A0
    pub fn decrement(&mut self, amount: f32) {
        self.step_by(amount, -1);
    }

    fn step_by(&mut self, amount: f32, dir: i32) {
        assert!(!self.read_only, "!m_bReadOnly");
        match self.kind {
            CTypeKind::Float | CTypeKind::Integer => {
                let mut step = as_f32(&self.step);
                let cur = as_f32(&self.cell.get());
                if step == 0.0 {
                    step = DEFAULT_STEP;
                }
                let v = if dir > 0 { step * amount + cur } else { cur - step * amount };
                self.set_value(v);
            }
            CTypeKind::Enum => {
                let next = if dir > 0 { self.enum_current + 1 } else { self.enum_current.wrapping_sub(1) };
                if let Some((_, val)) = self.enumerators.get(next) {
                    self.enum_current = next;
                    self.cell.set(TweakData::Int(*val));
                }
            }
            CTypeKind::Bool => {
                let b = self.cell.bool();
                self.cell.set(TweakData::Bool(!b));
            }
            _ => {}
        }
    }

    /// tweak.cpp 0x000845C0 — the type's `toString` of the current value.
    pub fn to_string_value(&self) -> String {
        type_to_string(self.kind, &self.enumerators, &self.cell.get())
    }

    /// tweak.cpp 0x00084610 — the default value as a string (peeks show the live value).
    pub fn default_to_string(&self) -> String {
        if self.read_only {
            return self.to_string_value();
        }
        type_to_string(self.kind, &self.enumerators, &self.default)
    }
}

/// VSTYPE toString formats: int `%d`, float `%g`, bool `on`/`off`, string `"%s"`, enum element name.
fn type_to_string(kind: CTypeKind, enums: &[(String, i32)], v: &TweakData) -> String {
    match (kind, v) {
        (CTypeKind::Float, _) => fmt_g(as_f32(v)),
        (CTypeKind::Integer, _) => format!("{}", as_f32_i(v)),
        (CTypeKind::Bool, _) => if as_f32_i(v) != 0 { "on" } else { "off" }.into(),
        (CTypeKind::String, TweakData::String(s)) => format!("\"{s}\""),
        (CTypeKind::Enum, _) => {
            let i = as_f32_i(v);
            enums.iter().find(|(_, val)| *val == i).map(|(n, _)| n.clone()).unwrap_or_default()
        }
        _ => String::new(),
    }
}

fn as_f32(v: &TweakData) -> f32 {
    match v {
        TweakData::Float(f) => *f,
        TweakData::Int(i) => *i as f32,
        TweakData::Bool(b) => *b as i32 as f32,
        TweakData::String(_) => 0.0,
    }
}

fn as_f32_i(v: &TweakData) -> i32 {
    match v {
        TweakData::Float(f) => *f as i32,
        TweakData::Int(i) => *i,
        TweakData::Bool(b) => *b as i32,
        TweakData::String(_) => 0,
    }
}

/// x86 `cvttss2si`: truncation, NaN/out of range -> i32::MIN.
fn cvttss2si(f: f32) -> i32 {
    if f.is_nan() || f >= 2147483648.0 || f < -2147483648.0 { i32::MIN } else { f as i32 }
}

/// crt `%g` (6 significant digits, trailing zeros stripped).
pub fn fmt_g(f: f32) -> String {
    let f = f as f64;
    if f == 0.0 {
        return "0".into();
    }
    if !f.is_finite() {
        return if f.is_nan() { "-1.#IND".into() } else if f > 0.0 { "1.#INF".into() } else { "-1.#INF".into() };
    }
    let exp = f.abs().log10().floor() as i32;
    let s = if !(-4..6).contains(&exp) {
        let m = format!("{:.5e}", f);
        let (mant, e) = m.split_once('e').unwrap();
        let mant = strip_zeros(mant);
        let e: i32 = e.parse().unwrap();
        format!("{}e{}{:03}", mant, if e < 0 { '-' } else { '+' }, e.abs())
    } else {
        strip_zeros(&format!("{:.*}", (5 - exp).max(0) as usize, f))
    };
    s
}

fn strip_zeros(s: &str) -> String {
    if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s.to_string() }
}

/// VsStringAnsi.cpp 0x002B46B0 — `VsStringAnsi` crc: case-sensitive HANDY_CRC32, empty = 0.
fn crc(s: &str) -> u32 {
    if s.is_empty() { 0 } else { handy_crc::crc32(s.as_bytes(), 0) }
}

/// `CTweakSystem` (0x64 bytes): a named group of tweaks and child systems.
#[derive(Debug)]
pub struct CTweakSystem {
    pub name: String,           // +0x08
    pub parent: Option<usize>,  // +0x48
    pub children: Vec<usize>,   // +0x4C
    pub tweaks: Vec<CTweakValue>, // +0x58
}

/// handle to a tweak inside the manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TweakId {
    pub system: usize,
    pub index: usize,
}

/// `CTweakManager` (0x000830B0): owns every system; index 0 is the root `$`.
#[derive(Debug)]
pub struct CTweakManager {
    pub systems: Vec<CTweakSystem>,
    env: CEnvironment,
}

impl Default for CTweakManager {
    /// tweak.cpp 0x000830B0
    fn default() -> Self {
        CTweakManager {
            systems: vec![CTweakSystem { name: "$".into(), parent: None, children: Vec::new(), tweaks: Vec::new() }],
            env: CEnvironment::with_builtins(),
        }
    }
}

impl CTweakManager {
    pub const ROOT: usize = 0;

    /// tweak.cpp 0x000833F0 — find a system by dotted path below the root (case-insensitive).
    pub fn find_system(&self, path: &str) -> Option<usize> {
        let mut cur = Self::ROOT;
        let mut found = None;
        for tok in path.split('.').filter(|t| !t.is_empty()) {
            let next = self.systems[cur].children.iter().copied().find(|&c| self.systems[c].name.eq_ignore_ascii_case(tok))?;
            cur = next;
            found = Some(next);
        }
        found
    }

    /// tweak.cpp 0x000835C0 — find or create every system along the path.
    pub fn create_system(&mut self, path: &str) -> usize {
        let mut cur = Self::ROOT;
        for tok in path.split('.').filter(|t| !t.is_empty()) {
            let existing = self.systems[cur].children.iter().copied().find(|&c| self.systems[c].name.eq_ignore_ascii_case(tok));
            cur = match existing {
                Some(c) => c,
                None => {
                    let id = self.systems.len();
                    self.systems.push(CTweakSystem {
                        name: tok.chars().take(MAX_NAME).collect(),
                        parent: Some(cur),
                        children: Vec::new(),
                        tweaks: Vec::new(),
                    });
                    // tweak.cpp 0x000865B0 addChild: new systems at the head
                    self.systems[cur].children.insert(0, id);
                    id
                }
            };
        }
        cur
    }

    /// tweak.cpp 0x000859E0
    pub fn find_tweak(&self, system: usize, name: &str) -> Option<TweakId> {
        let c = crc(name);
        self.systems[system].tweaks.iter().position(|t| t.name_crc == c).map(|index| TweakId { system, index })
    }

    fn insert(&mut self, system: usize, v: CTweakValue) -> TweakId {
        if let Some(id) = self.find_tweak(system, &v.name) {
            return id;
        }
        let t = &mut self.systems[system].tweaks;
        t.push(v);
        TweakId { system, index: t.len() - 1 }
    }

    /// tweak.cpp 0x00085A90 — addTweak(float).
    pub fn add_tweak_f32(&mut self, system: usize, name: &str, cell: TweakCell, min: f32, max: f32, step: f32) -> TweakId {
        let mut v = CTweakValue::new(name, CTypeKind::Float, cell, false);
        v.min = TweakData::Float(min);
        v.max = TweakData::Float(max);
        v.step = TweakData::Float(if step == 0.0 { DEFAULT_STEP } else { step });
        self.insert(system, v)
    }

    /// tweak.cpp 0x00085B40 — addTweak(S32); a zero step becomes `(S32)ceil(0.1)` = 1.
    pub fn add_tweak_i32(&mut self, system: usize, name: &str, cell: TweakCell, min: i32, max: i32, step: i32) -> TweakId {
        let mut v = CTweakValue::new(name, CTypeKind::Integer, cell, false);
        v.min = TweakData::Int(min);
        v.max = TweakData::Int(max);
        v.step = TweakData::Int(if step == 0 { DEFAULT_STEP.ceil() as i32 } else { step });
        self.insert(system, v)
    }

    /// tweak.cpp 0x00085BF0 — addTweak(enum): `elements` is the enum body text (`A, B = 5, C`).
    pub fn add_tweak_enum(&mut self, system: usize, name: &str, cell: TweakCell, elements: &str) -> TweakId {
        let mut v = CTweakValue::new(name, CTypeKind::Enum, cell, false);
        let decl = format!("enum e{} {{ {} }};", v.name, elements);
        if let Ok((t, _)) = self.env.parse(&decl, true) {
            v.enumerators = t.enumerators.clone();
            self.env.add(t);
        }
        self.insert(system, v)
    }

    /// tweak.cpp 0x00085CA0 — addTweak(bool).
    pub fn add_tweak_bool(&mut self, system: usize, name: &str, cell: TweakCell) -> TweakId {
        let v = CTweakValue::new(name, CTypeKind::Bool, cell, false);
        self.insert(system, v)
    }

    /// addPeek(float / S32 / bool / string): read-only display of a live value.
    pub fn add_peek(&mut self, system: usize, name: &str, cell: TweakCell) -> TweakId {
        let kind = match cell.get() {
            TweakData::Float(_) => CTypeKind::Float,
            TweakData::Int(_) => CTypeKind::Integer,
            TweakData::Bool(_) => CTypeKind::Bool,
            TweakData::String(_) => CTypeKind::String,
        };
        let v = CTweakValue::new(name, kind, cell, true);
        self.insert(system, v)
    }

    pub fn tweak(&self, id: TweakId) -> &CTweakValue {
        &self.systems[id.system].tweaks[id.index]
    }

    pub fn tweak_mut(&mut self, id: TweakId) -> &mut CTweakValue {
        &mut self.systems[id.system].tweaks[id.index]
    }

    /// tweak.cpp 0x00085090 — dotted path of a system (the root contributes nothing).
    pub fn system_path(&self, system: usize) -> String {
        match self.systems[system].parent {
            None => String::new(),
            Some(p) => {
                let pp = self.system_path(p);
                if pp.is_empty() { self.systems[system].name.clone() } else { format!("{}.{}", pp, self.systems[system].name) }
            }
        }
    }

    /// tweak.cpp 0x000861C0 — `system.path.name`.
    pub fn full_name(&self, id: TweakId) -> String {
        format!("{}.{}", self.system_path(id.system), self.tweak(id).name)
    }

    /// tweak.cpp 0x00083830 — `tweak.set <system> <name> <value>` lines for one system.
    pub fn save_system(&self, system: usize, out: &mut String) {
        let path = self.system_path(system);
        for t in &self.systems[system].tweaks {
            out.push_str(&format!("tweak.set {}.{} {}\r\n", path, t.name, t.to_string_value()));
        }
    }

    /// tweak.cpp 0x000854A0 — the `Tweaks.txt` export (GameTweaksMenu.cpp 0x000AFFB0), recursive.
    pub fn export(&self, system: usize, out: &mut String) {
        if self.systems[system].parent.is_some() {
            out.push_str(&format!("========== {} ==========\r\n", self.system_path(system)));
        }
        for i in 0..self.systems[system].tweaks.len() {
            let id = TweakId { system, index: i };
            out.push_str(&format!("{} = {}\r\n", self.full_name(id), self.tweak(id).to_string_value()));
        }
        for &c in &self.systems[system].children {
            self.export(c, out);
        }
    }

    /// tweak.cpp 0x000851A0 — the TTY export (GameTweaksMenu.cpp 0x000AFF60), recursive.
    pub fn export_tty(&self, system: usize, out: &mut Vec<String>) {
        let path = self.system_path(system);
        let named = self.systems[system].parent.is_some();
        if named {
            out.push(format!("Begin Export: {path}"));
        }
        for i in 0..self.systems[system].tweaks.len() {
            let id = TweakId { system, index: i };
            out.push(format!("\t{} = {}", self.full_name(id), self.tweak(id).to_string_value()));
        }
        if named {
            out.push(format!("End Export: {path}"));
        }
        for &c in &self.systems[system].children {
            self.export_tty(c, out);
        }
    }

    /// not in F3_D (its console was compiled out): apply a `tweak.set a.b.name value` line as the save format
    /// implies. returns false if the line or tweak is unknown.
    pub fn apply_set_line(&mut self, line: &str) -> bool {
        let mut it = line.trim().splitn(3, char::is_whitespace);
        if !it.next().is_some_and(|c| c.eq_ignore_ascii_case("tweak.set")) {
            return false;
        }
        let (Some(path), Some(value)) = (it.next(), it.next()) else { return false };
        let Some((sys, name)) = path.rsplit_once('.') else { return false };
        let Some(s) = self.find_system(sys) else { return false };
        let Some(id) = self.find_tweak(s, name) else { return false };
        let value = value.trim();
        let t = self.tweak_mut(id);
        if t.read_only {
            return false;
        }
        match t.kind {
            CTypeKind::Float | CTypeKind::Integer => match value.parse::<f32>() {
                Ok(v) => t.set_value(v),
                Err(_) => return false,
            },
            CTypeKind::Bool => t.cell.set(TweakData::Bool(matches!(value, "on" | "true" | "1"))),
            CTypeKind::String => t.cell.set(TweakData::String(value.trim_matches('"').to_string())),
            CTypeKind::Enum => {
                let Some(i) = t.enumerators.iter().position(|(n, _)| n == value) else { return false };
                t.enum_current = i;
                let v = t.enumerators[i].1;
                t.cell.set(TweakData::Int(v));
            }
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systems_and_values() {
        let mut m = CTweakManager::default();
        let s = m.create_system("Physics.Car");
        assert_eq!(m.find_system("physics.CAR"), Some(s));
        assert_eq!(m.system_path(s), "Physics.Car");
        let grip = TweakCell::new(TweakData::Float(1.0));
        let id = m.add_tweak_f32(s, "Grip", grip.clone(), 0.0, 2.0, 0.0);
        assert_eq!(m.add_tweak_f32(s, "Grip", grip.clone(), 0.0, 2.0, 0.0), id);
        m.tweak_mut(id).set_value(5.0);
        assert_eq!(grip.f32(), 2.0);
        m.tweak_mut(id).decrement(1.0);
        assert!((grip.f32() - 1.9).abs() < 1e-6);
        let gear = TweakCell::new(TweakData::Int(3));
        let g = m.add_tweak_i32(s, "Gear", gear.clone(), 1, 6, 1);
        m.tweak_mut(g).increment(1.0);
        assert_eq!(gear.i32(), 4);
        let mode = TweakCell::new(TweakData::Int(0));
        let e = m.add_tweak_enum(s, "Mode", mode.clone(), "Off, On, Auto = 7");
        m.tweak_mut(e).increment(1.0);
        m.tweak_mut(e).increment(1.0);
        assert_eq!(mode.i32(), 7);
        assert_eq!(m.tweak(e).to_string_value(), "Auto");
        let mut out = String::new();
        m.save_system(s, &mut out);
        assert!(out.contains("tweak.set Physics.Car.Gear 4\r\n"));
        assert!(m.apply_set_line("tweak.set Physics.Car.Gear 2"));
        assert_eq!(gear.i32(), 2);
        let mut ex = String::new();
        m.export(CTweakManager::ROOT, &mut ex);
        assert!(ex.contains("========== Physics.Car ==========\r\n"));
        assert_eq!(fmt_g(1.9), "1.9");
        assert_eq!(fmt_g(1e7), "1e+007");
    }
}
