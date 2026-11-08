//! the audio subsystem's `AddTweaks` registrations (7 functions, docs/audio.md "Debug-menu tweaks") as recorded
//! `CTweakManager` call lists, plus one value slot per tweak that mirrors the live variable it edits.
//!
//! velocity_game depends on velocity_audio, so this crate cannot see `CTweakManager`: core replays `calls(origin)`
//! for each of `ADD_TWEAKS_ORDER` at the in-game VsAudio::Init, keeps one `TweakCell` per slot and copies the cells
//! to / from `AudioTweaks.values` around the debug menu (graphics' `gfx_tweaks` scheme). `sync_audio_tweaks`
//! pushes changed slots into `VisosAudio.game` and pulls the live values back every frame (the original points the
//! tweak at the variable itself). The two Audio.Slowdown floats have no reader in the binary: the slot is the variable.

use bevy::prelude::*;

use crate::plugin::VisosAudio;
use crate::vs_audio::VsAudio;

/// a tweak value (`TweakData` without strings)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AudioTweakValue {
    Float(f32),
    Int(i32),
    Bool(bool),
}

impl AudioTweakValue {
    pub fn f32(self) -> f32 {
        match self {
            AudioTweakValue::Float(f) => f,
            AudioTweakValue::Int(i) => i as f32,
            AudioTweakValue::Bool(b) => b as i32 as f32,
        }
    }
    pub fn i32(self) -> i32 {
        match self {
            AudioTweakValue::Float(f) => f as i32,
            AudioTweakValue::Int(i) => i,
            AudioTweakValue::Bool(b) => b as i32,
        }
    }
    pub fn bool(self) -> bool {
        self.i32() != 0
    }
    /// the same value in `like`'s variant
    fn conform(self, like: AudioTweakValue) -> AudioTweakValue {
        match like {
            AudioTweakValue::Float(_) => AudioTweakValue::Float(self.f32()),
            AudioTweakValue::Int(_) => AudioTweakValue::Int(self.i32()),
            AudioTweakValue::Bool(_) => AudioTweakValue::Bool(self.bool()),
        }
    }
}

/// one recorded `CTweakManager` call (the audio registrations use no enum tweaks)
#[derive(Clone, Debug, PartialEq)]
pub enum AudioTweakCall {
    /// createSystem (0x00086250) by dotted path below the root; the following tweaks go there
    System(&'static str),
    /// addTweak(float) 0x00085A90
    Float { name: &'static str, slot: usize, min: f32, max: f32, step: f32 },
    /// addTweak(S32) 0x00085B40
    Int { name: &'static str, slot: usize, min: i32, max: i32, step: i32 },
    /// addTweak(bool) 0x00085CA0
    Bool { name: &'static str, slot: usize },
}

impl AudioTweakCall {
    pub fn slot(&self) -> Option<usize> {
        match self {
            AudioTweakCall::System(_) => None,
            AudioTweakCall::Float { slot, .. } | AudioTweakCall::Int { slot, .. } | AudioTweakCall::Bool { slot, .. } => Some(*slot),
        }
    }
}

/// GameManager::CommonInit 0x000B4F80 calls VsAudio::Init 0x00047A00 at this site (front end and in game)
pub const INIT_CALL_SITE: u32 = 0x000B5169;
/// VsAudio::Init calls VsAudio::AddTweaks 0x000484D0 here, after LoadEnvAudio; AddTweaks returns at once when
/// VsAudio +104 (front end) is set, so the tweaks exist only after an in-game Init
pub const ADD_TWEAKS_CALL_SITE: u32 = 0x00048362;
/// VsAudio::AddTweaks 0x000484D0: the registrations in call order (Impact, Debug, Music, Vehics, Engine,
/// Slowdown, EnvFx)
pub const ADD_TWEAKS_ORDER: [u32; 7] = [0x00048410, 0x00048530, 0x0004D7D0, 0x0004FD50, 0x0004CB10, 0x0004B380, 0x00048600];

/// accessors of the live variable behind a slot
#[derive(Clone, Copy)]
pub struct Live {
    pub get: fn(&VsAudio) -> AudioTweakValue,
    pub set: fn(&mut VsAudio, AudioTweakValue),
}

/// one registration function
#[derive(Clone, Debug)]
pub struct AudioTweakFn {
    pub origin: u32,
    pub calls: Vec<AudioTweakCall>,
}

/// every audio registration and the value slots their tweaks point at
#[derive(Resource)]
pub struct AudioTweaks {
    /// in address order; `ADD_TWEAKS_ORDER` is the call order
    pub fns: Vec<AudioTweakFn>,
    /// the tweaked variables (core's cells copy in / out of these)
    pub values: Vec<AudioTweakValue>,
    /// the values last synced with the live variables
    applied: Vec<AudioTweakValue>,
    live: Vec<Option<Live>>,
}

impl AudioTweaks {
    /// the recorded calls of the registration at `origin`
    pub fn calls(&self, origin: u32) -> Option<&[AudioTweakCall]> {
        self.fns.iter().find(|f| f.origin == origin).map(|f| &f.calls[..])
    }

    /// whether a slot reaches a live variable
    pub fn is_live(&self, slot: usize) -> bool {
        self.live[slot].is_some()
    }

    /// the slot of tweak `name` in the registration at `origin`
    pub fn find(&self, origin: u32, name: &str) -> Option<usize> {
        self.calls(origin)?.iter().find_map(|c| match c {
            AudioTweakCall::Float { name: n, slot, .. } | AudioTweakCall::Int { name: n, slot, .. } | AudioTweakCall::Bool { name: n, slot } if *n == name => Some(*slot),
            _ => None,
        })
    }
}

impl Default for AudioTweaks {
    fn default() -> Self {
        let mut r = Reg { t: AudioTweaks { fns: Vec::new(), values: Vec::new(), applied: Vec::new(), live: Vec::new() }, calls: Vec::new() };
        add_impact_tweaks(&mut r);
        r.end(0x00048410);
        add_debug_tweaks(&mut r);
        r.end(0x00048530);
        add_env_fx_tweaks(&mut r);
        r.end(0x00048600);
        add_slowdown_tweaks(&mut r);
        r.end(0x0004B380);
        add_engine_tweaks(&mut r);
        r.end(0x0004CB10);
        add_music_tweaks(&mut r);
        r.end(0x0004D7D0);
        vs_audio_vehics_add_tweaks(&mut r);
        r.end(0x0004FD50);
        let mut t = r.t;
        t.applied = t.values.clone();
        t
    }
}

/// registration recorder
struct Reg {
    t: AudioTweaks,
    calls: Vec<AudioTweakCall>,
}

impl Reg {
    fn end(&mut self, origin: u32) {
        let calls = std::mem::take(&mut self.calls);
        self.t.fns.push(AudioTweakFn { origin, calls });
    }
    fn slot(&mut self, v: AudioTweakValue, live: Option<Live>) -> usize {
        self.t.values.push(v);
        self.t.live.push(live);
        self.t.values.len() - 1
    }
    fn sys(&mut self, path: &'static str) {
        self.calls.push(AudioTweakCall::System(path));
    }
    fn f(&mut self, name: &'static str, min: f32, max: f32, step: f32, init: f32, live: Option<Live>) {
        let slot = self.slot(AudioTweakValue::Float(init), live);
        self.calls.push(AudioTweakCall::Float { name, slot, min, max, step });
    }
    fn i(&mut self, name: &'static str, min: i32, max: i32, step: i32, init: i32, live: Option<Live>) {
        let slot = self.slot(AudioTweakValue::Int(init), live);
        self.calls.push(AudioTweakCall::Int { name, slot, min, max, step });
    }
    fn b(&mut self, name: &'static str, init: bool, live: Option<Live>) {
        let slot = self.slot(AudioTweakValue::Bool(init), live);
        self.calls.push(AudioTweakCall::Bool { name, slot });
    }
}

/// a live f32 / i32 / bool field of `VsAudio`: `fld!(F, |a| a.field)`
macro_rules! fld {
    (F, |$a:ident| $place:expr) => {
        Some(Live { get: |$a| AudioTweakValue::Float($place), set: |$a, v| $place = v.f32() })
    };
    (I, |$a:ident| $place:expr) => {
        Some(Live { get: |$a| AudioTweakValue::Int($place), set: |$a, v| $place = v.i32() })
    };
    (B, |$a:ident| $place:expr) => {
        Some(Live { get: |$a| AudioTweakValue::Bool($place), set: |$a, v| $place = v.bool() })
    };
}

fn bits(b: u32) -> f32 {
    f32::from_bits(b)
}

/// VsAudio.cpp 0x00048410
fn add_impact_tweaks(r: &mut Reg) {
    r.sys("Audio.ImpactStuff");
    r.i("Impact Frame Count", 0, 30, 1, 15, fld!(I, |a| a.impact_frames));
}

/// VsAudio.cpp 0x00048530
fn add_debug_tweaks(r: &mut Reg) {
    // VsAudio +484, read by VsAudio::Update 0x00048F4C (debug text display, not drawn by the port)
    r.sys("Audio.Debug");
    r.f("Debug Txt on", 0.0, 1.0, 1.0, 0.0, fld!(F, |a| a.debug_text));
}

/// VsAudio.cpp 0x00048600
fn add_env_fx_tweaks(r: &mut Reg) {
    r.sys("Audio.ENVFX");
    r.b("On/Off", true, fld!(B, |a| a.envfx_on));
    r.f("DopplerFactor", 0.0, 10.0, bits(0x3DCC_CCCD), bits(0x3E99_999A), fld!(F, |a| a.doppler));
    r.f("SpeedOfSound", 0.0, 500.0, 1.0, bits(0x43AB_A666), fld!(F, |a| a.speed_of_sound));
}

/// VsAudio.cpp 0x0004B380 (unk_684F00 / unk_684F04: no reader in the binary)
fn add_slowdown_tweaks(r: &mut Reg) {
    r.sys("Audio.Slowdown");
    r.f("Slowdown Factor", 0.0, 200.0, 1.0, 25.0, None);
    r.f("Slowdown Target", -1200.0, 0.0, 10.0, -800.0, None);
}

/// VsAudioRevBands.cpp 0x0004CB10 (dword_684F94 = the options' engine volume, VsAudio::Update 0x00048CC1)
fn add_engine_tweaks(r: &mut Reg) {
    r.sys("Audio.Engine");
    r.f("Volume", 0.0, 1.0, bits(0x3C23_D70A), bits(0x3EE6_6666), fld!(F, |a| a.settings.engine));
}

/// VsAudioRevBands.cpp 0x0004D7D0 (dword_684FB4 = the options' music volume, VsAudio::Update 0x00048CA1)
fn add_music_tweaks(r: &mut Reg) {
    r.sys("Audio.Music");
    r.f("Volume", 0.0, 1.0, bits(0x3C23_D70A), 0.75, fld!(F, |a| a.settings.music));
    r.f("TFT_MusicMult", 0.0, 1.0, bits(0x3DCC_CCCD), bits(0x3F66_6666), fld!(F, |a| a.tft_music_mult));
}

/// VsAudioVehics.cpp 0x0004FD50
fn vs_audio_vehics_add_tweaks(r: &mut Reg) {
    let c = bits(0x3C23_D70A); // 0.01f
    let t = bits(0x3DCC_CCCD); // 0.1f
    r.sys("Audio.VolTweaks");
    r.f("PassBy", 0.0, 5.0, c, bits(0x3F4C_CCCD), fld!(F, |a| a.vehicle_tweaks.passby));
    r.sys("Audio.SirenTweaks");
    r.f("State 1 min time", 0.5, 30.0, t, 2.0, fld!(F, |a| a.vehicle_tweaks.siren_times[0].0));
    r.f("State 1 max time", 0.5, 30.0, t, 8.0, fld!(F, |a| a.vehicle_tweaks.siren_times[0].1));
    r.f("State 2 min time", 0.5, 30.0, t, 2.5, fld!(F, |a| a.vehicle_tweaks.siren_times[1].0));
    r.f("State 2 max time", 0.5, 30.0, t, 7.0, fld!(F, |a| a.vehicle_tweaks.siren_times[1].1));
    r.f("State 3 min time", 0.5, 30.0, t, 0.5, fld!(F, |a| a.vehicle_tweaks.siren_times[2].0));
    r.f("State 3 max time", 0.5, 30.0, t, 1.0, fld!(F, |a| a.vehicle_tweaks.siren_times[2].1));
    r.f("State 4 min time", 0.5, 30.0, t, 4.0, fld!(F, |a| a.vehicle_tweaks.siren_times[3].0));
    r.f("State 4 max time", 0.5, 30.0, t, 10.0, fld!(F, |a| a.vehicle_tweaks.siren_times[3].1));
    r.sys("Audio.Suspension");
    r.i("Wait Frame Count", 0, 30, 1, 5, fld!(I, |a| a.vehicle_tweaks.susp_wait_frames));
    r.f("Front Pressure Small Limit", 0.0, 3.0, c, bits(0x3D4C_CCCD), fld!(F, |a| a.vehicle_tweaks.front_pressure));
    r.f("Rear Pressure Small Limit", 0.0, 3.0, c, bits(0x3D4C_CCCD), fld!(F, |a| a.vehicle_tweaks.rear_pressure));
    r.sys("Audio.OtherCarImpacts");
    r.f("Impact Distance Limit", 0.0, 50.0, 1.0, 10.0, fld!(F, |a| a.vehicle_tweaks.impact_distance));
    r.sys("Audio.AIRevs");
    r.f("ai revs rate of change", 0.0, 1000.0, 10.0, 50.0, fld!(F, |a| a.vehicle_tweaks.ai_revs_rate));
    r.sys("Audio.PlayerPositionVols");
    let (near, far) = (bits(0x3E99_999A), bits(0xBE99_999A)); // 0.3, -0.3
    r.f("Front Normal Vol", -1.0, 1.0, c, 0.0, fld!(F, |a| a.vehicle_tweaks.front_vol[0]));
    r.f("Front Far Vol", -1.0, 1.0, c, far, fld!(F, |a| a.vehicle_tweaks.front_vol[1]));
    r.f("Front Bumper Vol", -1.0, 1.0, c, near, fld!(F, |a| a.vehicle_tweaks.front_vol[2]));
    r.f("Front Bonnet Vol", -1.0, 1.0, c, bits(0x3E4C_CCCD), fld!(F, |a| a.vehicle_tweaks.front_vol[3]));
    r.f("Wheel Normal Vol", -1.0, 1.0, c, 0.0, fld!(F, |a| a.vehicle_tweaks.wheel_vol[0]));
    r.f("Wheel Far Vol", -1.0, 1.0, c, far, fld!(F, |a| a.vehicle_tweaks.wheel_vol[1]));
    r.f("Wheel Bumper Vol", -1.0, 1.0, c, near, fld!(F, |a| a.vehicle_tweaks.wheel_vol[2]));
    r.f("Wheel Bonnet Vol", -1.0, 1.0, c, bits(0x3E4C_CCCD), fld!(F, |a| a.vehicle_tweaks.wheel_vol[3]));
    r.f("Rear Normal Vol", -1.0, 1.0, c, 0.0, fld!(F, |a| a.vehicle_tweaks.rear_vol[0]));
    r.f("Rear Far Vol", -1.0, 1.0, c, bits(0x3E33_3333), fld!(F, |a| a.vehicle_tweaks.rear_vol[1]));
    r.f("Rear Bumper Vol", -1.0, 1.0, c, far, fld!(F, |a| a.vehicle_tweaks.rear_vol[2]));
    r.f("Rear Bonnet Vol", -1.0, 1.0, c, far, fld!(F, |a| a.vehicle_tweaks.rear_vol[3]));
}

/// each frame: a slot the menu changed (value != last synced) is written to its live variable, every other slot
/// takes the live value (so the menu shows what the game uses)
pub fn sync_audio_tweaks(mut at: ResMut<AudioTweaks>, mut va: ResMut<VisosAudio>) {
    let at = &mut *at;
    let game = &mut va.game;
    for i in 0..at.values.len() {
        if let Some(live) = at.live[i] {
            if at.values[i] != at.applied[i] {
                (live.set)(game, at.values[i]);
            } else {
                at.values[i] = (live.get)(game).conform(at.values[i]);
            }
        }
        at.applied[i] = at.values[i];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::system::AudioSystem;

    /// the TSV dump of the 7 registrations (`tools/ida/audio_tweak_regs.py`): fn, kind, name, var, args, init
    fn dump() -> Option<String> {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../research/audio/tweak_regs.tsv");
        std::fs::read_to_string(p).ok()
    }

    fn unquote(s: &str) -> String {
        s.trim().trim_start_matches("(int)").trim().trim_matches('"').to_string()
    }

    /// a Hex-Rays constant: float literal (`0.1f`, `1.0`), int (`30`, `1u`), float bits pushed as an int
    fn num_f(s: &str) -> f32 {
        let s = s.trim();
        if s.contains('.') {
            return s.trim_end_matches('f').parse::<f32>().unwrap();
        }
        let i = num_i(s);
        if i.unsigned_abs() > 100_000 { f32::from_bits(i as u32) } else { i as f32 }
    }

    fn num_i(s: &str) -> i32 {
        s.trim().trim_end_matches('u').parse::<i64>().unwrap() as i32
    }

    /// every registration matches the binary: systems, kinds, names, ranges, steps and every .data default
    #[test]
    fn registrations_match_the_binary() {
        let Some(tsv) = dump() else {
            eprintln!("research/audio/tweak_regs.tsv missing: skipped");
            return;
        };
        let at = AudioTweaks::default();
        assert_eq!(at.fns.len(), 7);
        let mut n = 0;
        for f in &at.fns {
            let rows: Vec<Vec<&str>> =
                tsv.lines().map(|l| l.split('\t').collect::<Vec<_>>()).filter(|c| u32::from_str_radix(c[0], 16).unwrap() == f.origin).collect();
            assert_eq!(rows.len(), f.calls.len(), "{:x}: call count", f.origin);
            for (c, row) in f.calls.iter().zip(&rows) {
                let name = unquote(row[2]);
                let args: Vec<&str> = if row.len() > 4 && !row[4].is_empty() { row[4].split('|').collect() } else { vec![] };
                let init = row.get(5).copied().unwrap_or("");
                match c {
                    AudioTweakCall::System(p) => assert_eq!((row[1], *p), ("S", name.as_str()), "{:x}", f.origin),
                    AudioTweakCall::Float { name: nm, slot, min, max, step } => {
                        assert_eq!((row[1], *nm), ("F", name.as_str()), "{:x}", f.origin);
                        let want = (num_f(args[0]), num_f(args[1]), num_f(args[2]));
                        assert_eq!((min.to_bits(), max.to_bits(), step.to_bits()), (want.0.to_bits(), want.1.to_bits(), want.2.to_bits()), "{nm}");
                        let def = if init.len() == 8 { u32::from_str_radix(init, 16).unwrap() } else { 0 }; // "" = VsAudio +484 (bss)
                        assert_eq!(at.values[*slot].f32().to_bits(), def, "{nm} default");
                    }
                    AudioTweakCall::Int { name: nm, slot, min, max, step } => {
                        assert_eq!((row[1], *nm), ("I", name.as_str()), "{:x}", f.origin);
                        assert_eq!((*min, *max, *step), (num_i(args[0]), num_i(args[1]), num_i(args[2])), "{nm}");
                        assert_eq!(at.values[*slot].i32(), u32::from_str_radix(init, 16).unwrap() as i32, "{nm} default");
                    }
                    AudioTweakCall::Bool { name: nm, slot } => {
                        assert_eq!((row[1], *nm), ("B", name.as_str()), "{:x}", f.origin);
                        // byte variable: the low byte
                        assert_eq!(at.values[*slot].bool(), u32::from_str_radix(init, 16).unwrap() & 0xFF != 0, "{nm} default");
                    }
                }
                n += 1;
            }
        }
        assert_eq!(n, tsv.lines().count());
    }

    /// slots are unique and dense, the call order names every registration once, and a fresh `VsAudio` holds the
    /// registered defaults in every live variable
    #[test]
    fn slots_order_and_live_defaults() {
        let at = AudioTweaks::default();
        let slots: Vec<usize> = at.fns.iter().flat_map(|f| f.calls.iter().filter_map(|c| c.slot())).collect();
        assert_eq!(slots, (0..at.values.len()).collect::<Vec<_>>());
        assert_eq!(at.values.len(), 36);
        let mut order = ADD_TWEAKS_ORDER.to_vec();
        order.sort();
        assert_eq!(order, at.fns.iter().map(|f| f.origin).collect::<Vec<_>>());
        let (mut system, _mixer) = AudioSystem::with_default_mixer();
        let game = VsAudio::new(&mut system);
        for (i, live) in at.live.iter().enumerate() {
            if let Some(l) = live {
                assert_eq!((l.get)(&game).conform(at.values[i]), at.values[i], "slot {i}");
            }
        }
        assert!(!at.is_live(at.find(0x0004B380, "Slowdown Factor").unwrap()));
    }

    /// a menu edit reaches the live variable, a live change reaches the slot
    #[test]
    fn sync_pushes_edits_and_pulls_live_values() {
        let mut app = App::new();
        app.init_resource::<AudioTweaks>().init_resource::<VisosAudio>().add_systems(Update, sync_audio_tweaks);
        let at = app.world().resource::<AudioTweaks>();
        let (doppler, wait) = (at.find(0x00048600, "DopplerFactor").unwrap(), at.find(0x0004FD50, "Wait Frame Count").unwrap());
        app.world_mut().resource_mut::<AudioTweaks>().values[doppler] = AudioTweakValue::Float(1.5);
        app.update();
        assert_eq!(app.world().resource::<VisosAudio>().game.doppler, 1.5);
        app.world_mut().resource_mut::<VisosAudio>().game.vehicle_tweaks.susp_wait_frames = 9;
        app.update();
        assert_eq!(app.world().resource::<AudioTweaks>().values[wait], AudioTweakValue::Int(9));
    }
}
