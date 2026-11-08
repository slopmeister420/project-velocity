//! the visos audio globals (`g_AUDIO_*`, 0x00782848..0x00782A0C) and the system-wide entry points:
//! AUDIO_Init.cpp (init, reset, update) plus the small global setters that live in their own one-function
//! visos files (volumes, presets, control parameters, environment).

use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};

use bevy::math::Vec3;

use super::mixer::{Mixer, SampleData, SharedMixer};
use super::pool::{Id, Pool, StackPool};
use super::xva::XvaSampleHeader;
use super::xvg::{XvgChain, XvgSound};
use super::xbox::voice::{Voice, VoiceSystem};

/// visos audio result codes (`0x09000000 | n`), 0 = ok.
pub mod err {
    pub const ALREADY_INIT: i32 = 0x0900_0000;
    pub const NOT_INIT: i32 = 0x0900_0001;
    pub const BAD_PARAM: i32 = 0x0900_0002;
    pub const FAILED: i32 = 0x0900_0003;
    pub const NO_EMITTER: i32 = 0x0900_0005;
    pub const NO_LISTENER: i32 = 0x0900_0006;
    pub const NO_SAMPLE: i32 = 0x0900_0007;
    pub const NO_EVENT: i32 = 0x0900_000B;
    pub const NO_STREAM: i32 = 0x0900_000C;
    pub const VERSION: i32 = 0x0900_000D;
    pub const READ: i32 = 0x0900_000E;
    pub const NO_MEMORY: i32 = 0x0900_000F;
    pub const BAD_FILE: i32 = 0x0900_0011;
    pub const BAD_BANK: i32 = 0x0900_0013;
    pub const BAD_SAMPLE_INDEX: i32 = 0x0900_0014;
    pub const BAD_EMITTER: i32 = 0x0900_0015;
    pub const BAD_LISTENER: i32 = 0x0900_0016;
    pub const BAD_PLAYING_SAMPLE: i32 = 0x0900_0017;
    pub const UNKNOWN_EVENT: i32 = 0x0900_0019;
    pub const NOT_PAUSED: i32 = 0x0900_001B;
    pub const NOT_PLAYING: i32 = 0x0900_001B;
    pub const FADING: i32 = 0x0900_001C;
    pub const EMPTY_SAMPLE: i32 = 0x0900_001E;
    pub const DSOUND: i32 = 0x0900_0030;
    /// AUDIO_PlayEvent: the caller's handle already holds an event
    pub const ALREADY_PLAYING: i32 = 123456;
}

/// `AUDIO_InitParams` (88 bytes, defaults from its constructor 0x002BB210).
#[derive(Clone, Debug)]
pub struct AudioInitParams {
    pub unk8: u32,                // +8
    pub emitters: u32,            // +12
    pub listeners: u32,           // +16
    pub playing_samples: u32,     // +20
    pub streams: u32,             // +24
    pub unk28: u32,               // +28
    pub stream_extra: u32,        // +32
    pub stream_queue: u32,        // +36 max queued stream samples
    pub external_samples: u32,    // +40
    pub patch_buffers: u32,       // +44
    pub codecs: u32,              // +48
    pub unk52: u32,               // +52
    pub banks: u32,               // +56
    pub groups: u32,              // +60
    pub events: u32,              // +64 event look-up table
    pub active_events: u32,       // +68
    pub control_params: u32,      // +72
    pub emitter_min_distance: f32, // +76
    pub emitter_max_distance: f32, // +80
    pub doppler_factor: f32,      // +84
    pub dsp_image: String,        // +88
}

impl Default for AudioInitParams {
    /// AUDIO_InitParams.cpp 0x002BB210
    fn default() -> Self {
        Self {
            unk8: 0,
            emitters: 64,
            listeners: 1,
            playing_samples: 40,
            streams: 4,
            unk28: 2,
            stream_extra: 0,
            stream_queue: 0,
            external_samples: 0,
            patch_buffers: 0,
            codecs: 0,
            unk52: 512,
            banks: 40,
            groups: 64,
            events: 300,
            active_events: 64,
            control_params: 64,
            emitter_min_distance: 1.0,
            emitter_max_distance: 50.0,
            doppler_factor: 1.0,
            dsp_image: r"d:\audio\voice.bin".into(),
        }
    }
}

pub type BankId = Id<Bank>;
pub type GroupId = Id<Group>;
pub type EmitterId = Id<Emitter>;
pub type ListenerId = Id<Listener>;
pub type SampleId = Id<PlayingSample>;
pub type EventId = Id<ActiveEvent>;

/// `AUDIO_Bank` (40 bytes).
pub struct Bank {
    pub flags: u32,                     // +4
    pub refs: u32,                      // +8
    pub headers: Vec<XvaSampleHeader>,  // +12 count, +16 headers
    pub resident: Vec<Arc<SampleData>>, // +24 data block, split per sample
    pub streamed: Vec<Weak<SampleData>>, // streamed samples read on demand
    pub file: PathBuf,                  // +24 handle for streamed banks
    pub name_hash: u32,                 // +28
    pub(crate) header_at: u32,
}

/// `AUDIO_GroupSample` (20 bytes) with its bank resolved.
#[derive(Clone, Debug)]
pub struct GSample {
    pub weight: u32,
    pub volume: f32,
    pub bank: Option<BankId>,
    pub sample: u32,
}

/// `AUDIO_GroupSound` (84 bytes).
#[derive(Clone, Debug)]
pub struct GSound {
    pub def: XvgSound,
    pub samples: Vec<GSample>,
    pub chains: Vec<XvgChain>,
    pub last_sample: i32, // +68
    pub last_chain: i32,  // +72
}

/// `AUDIO_GroupEvent` (32 bytes).
#[derive(Clone, Debug)]
pub struct GEvent {
    pub hash: u32,  // +4
    pub unk8: u32,  // +8
    pub flags: u32, // +12
    pub unk16: u32, // +16
    pub sounds: Vec<GSound>,
    pub last: i32, // +24
}

/// `AUDIO_Group` (40 bytes).
pub struct Group {
    pub flags: u32,           // +4
    pub refs: u32,            // +8
    pub category_mask: u32,   // +12
    pub volume: f32,          // +16
    pub events: Vec<GEvent>,  // +20/+28
    pub banks: Vec<Option<BankId>>, // +24/+32
    pub name_hash: u32,       // +36
    pub(crate) allocs: GroupAllocs,
}

#[derive(Default, Clone)]
pub(crate) struct GroupAllocs {
    pub banks: Option<u32>,
    pub events: Option<u32>,
    pub sounds: Vec<u32>,
    pub samples: Vec<u32>,
    pub chains: Vec<u32>,
}

/// (group, event index): a pointer into a group's event array.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventRef {
    pub group: GroupId,
    pub event: usize,
}

/// 12-byte entry of the sorted event look-up table (`dword_7829DC`).
#[derive(Clone, Copy, Debug)]
pub struct EventLookup {
    pub hash: u32,
    pub group: Option<GroupId>,
    pub event: EventRef,
}

/// `AUDIO_Emitter` (120 bytes).
#[derive(Clone, Debug)]
pub struct Emitter {
    pub flags: u32,                // +4
    pub samples: Option<SampleId>, // +8 playing samples
    pub volume: f32,               // +24
    pub distance_gain: f32,        // +28
    pub position: Vec3,            // +32
    pub front: Vec3,               // +48
    pub top: Vec3,                 // +64
    pub velocity: Vec3,            // +80
    pub min_distance: f32,         // +96
    pub max_distance: f32,         // +100
    pub distance: f32,             // +104
    pub angle: f32,                // +108
    pub doppler: f32,              // +112 cents
}

/// emitter flags (+4)
pub mod emitter_flags {
    /// head-relative / non-positional: no 3d voice
    pub const FLAT: u32 = 1;
    /// doppler from velocity
    pub const MOVING: u32 = 2;
    pub const PAN: u32 = 8;
    pub const SOFTWARE: u32 = 0x10;
    pub const HAS_SAMPLES: u32 = 0x20;
    /// destroy once its samples end
    pub const AUTO_DESTROY: u32 = 0x40;
    pub const VOICE_3D: u32 = 0x1000;
    pub const VOLUME_DIRTY: u32 = 0x10_0000;
    pub const POSITION_DIRTY: u32 = 0x20_0000;
    pub const PITCH_DIRTY: u32 = 0x80_0000;
    pub const ALIVE: u32 = 0x4000_0000;
}

/// `AUDIO_Listener` (84 bytes).
#[derive(Clone, Debug)]
pub struct Listener {
    pub flags: u32,     // +4
    pub position: Vec3, // +8
    pub front: Vec3,    // +24
    pub top: Vec3,      // +40
    pub velocity: Vec3, // +56
    pub front_x: f32,   // +72
    pub front_z: f32,   // +76
}

/// sample completion callback (`+68`/`+72`); the only one the engine installs is the event's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleCallback {
    /// `_AUDIO_EventSampleCallback` 0x00342E10
    Event(EventId),
}

/// `AUDIO_PlayingSample` (96 bytes).
#[derive(Clone, Debug)]
pub struct PlayingSample {
    pub next: Option<SampleId>,       // +0 emitter list
    pub flags: u32,                   // +4
    pub queue_next: Option<SampleId>, // +8
    pub queue_index: u32,             // +12
    pub emitter: EmitterId,           // +16
    pub bank: BankId,                 // +20 sample header
    pub index: usize,
    pub stream: Option<StreamBuf>,    // +32
    pub resample_pos: f32,            // +40
    pub freq_ratio: f32,              // +44
    pub category_mask: u32,           // +48
    pub volume: f32,                  // +52
    pub fade_rate: f32,               // +56
    pub fade_time: f32,               // +60
    pub pitch: f32,                   // +64 cents
    pub callback: Option<SampleCallback>, // +68/+72
    pub voice: Option<Id<Voice>>,     // +76
    pub pan: f32,                     // +92
}

/// playing-sample flags (+4)
pub mod sample_flags {
    pub const LOOP: u32 = 1;
    pub const PAN: u32 = 2;
    pub const DONE: u32 = 0x10;
    pub const FINISHED: u32 = 0x20;
    pub const STOPPABLE: u32 = 0x80;
    pub const CHAIN_DATA: u32 = 0x100;
    pub const GLOBAL_PITCH: u32 = 0x200;
    pub const STARTED: u32 = 0x400;
    pub const PAUSED: u32 = 0x800;
    pub const QUEUED: u32 = 0x1000;
    pub const VOICE_DONE: u32 = 0x4000;
    pub const STOPPED_CB: u32 = 0x8000;
    pub const VOLUME_DIRTY: u32 = 0x10_0000;
    pub const POSITION_DIRTY: u32 = 0x20_0000;
    pub const FADING: u32 = 0x40_0000;
    pub const PITCH_DIRTY: u32 = 0x80_0000;
    pub const ALIVE: u32 = 0x4000_0000;
}

/// stream buffer (`AUDIO_StreamBuffer`, 44 bytes); the port streams from memory so only the
/// pool accounting survives.
#[derive(Clone, Copy, Debug, Default)]
pub struct StreamBuf;

/// event callback (`+64`/`+68`): code 1 = finished, 2 = killed.
pub type EventCallback = fn(code: i32, data: u32);

/// `AUDIO_ActiveEvent` (72 bytes).
#[derive(Clone, Debug)]
pub struct ActiveEvent {
    pub next: Option<EventId>, // +0
    pub flags: u32,            // +4
    pub prev: Option<EventId>, // +8
    pub event: EventRef,       // +12/+20
    pub sound: usize,          // +24
    pub sample: Option<usize>, // +28
    pub delay: f32,            // +32
    pub parent: Option<EventId>,      // +36
    pub child: Option<EventId>,       // +40
    pub next_sibling: Option<EventId>, // +44
    pub prev_sibling: Option<EventId>, // +48
    pub emitter: EmitterId,           // +52
    pub playing: Option<SampleId>,    // +56
    pub owner: Option<EventSlot>,     // +60
    pub callback: Option<EventCallback>, // +64
    pub data: u32,                    // +68
}

/// active event flags (+4)
pub mod event_flags {
    pub const PLAYING: u32 = 1;
    pub const DELAYED_CHAIN: u32 = 2;
    pub const DONE: u32 = 4;
    pub const CHAIN_ON_END: u32 = 0x100;
    pub const ALIVE: u32 = 0x4000_0000;
}

/// the caller's `AUDIO_ActiveEvent*` variable that `AUDIO_PlayEvent` fills and the engine clears when the
/// event dies (`+60` holds its address in the original).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EventSlot(pub u32);

/// one 16-byte control parameter (`dword_7829E4`): a value or a watched source mapped through min/max.
#[derive(Clone, Copy, Debug, Default)]
pub struct ControlParam {
    pub value: f32,
    pub source: Option<f32>,
    pub min: f32,
    pub max: f32,
}

/// all visos audio state.
pub struct AudioSystem {
    pub initialised: bool,       // dword_782848
    pub params: AudioInitParams,
    pub(crate) list_changed: bool, // dword_782850
    pub paused: bool,            // dword_782854

    pub banks: Pool<Bank>,
    pub bank_list: Vec<BankId>, // dword_782878, most recent first
    pub(crate) sample_headers: StackPool,
    pub groups: Pool<Group>,
    pub group_list: Vec<GroupId>, // dword_782950
    pub(crate) group_banks: StackPool,
    pub(crate) group_events: StackPool,
    pub(crate) group_sounds: StackPool,
    pub(crate) group_samples: StackPool,
    pub(crate) group_chains: StackPool,
    pub event_table: Vec<EventLookup>,
    pub(crate) last_search: i32, // dword_6C18E8

    pub emitters: Pool<Emitter>,
    pub emitter_list: Vec<EmitterId>, // dword_7828A4
    pub listeners: Pool<Listener>,
    pub listener_list: Vec<ListenerId>, // dword_7828BC
    pub samples: Pool<PlayingSample>,
    pub(crate) streams: Pool<StreamBuf>,
    pub events: Pool<ActiveEvent>,
    pub event_head: Option<EventId>, // dword_7829D4
    pub active_event_count: u32,     // dword_7829D8
    pub(crate) slots: Vec<Option<EventId>>,

    pub control: Vec<ControlParam>,
    pub category_volume: [f32; 32],       // flt_7816C0
    pub presets: [[f32; 32]; 32],         // dword_781740 (velocity.vol)
    pub(crate) fade_rate: [f32; 32],      // flt_7827C8
    pub(crate) fade_time: [f32; 32],      // flt_782740
    pub(crate) fading: u32,               // dword_7829F0
    pub(crate) category_dirty: u32,       // dword_7829EC
    pub(crate) all_dirty: bool,           // dword_7829E8
    pub(crate) pitch_all_dirty: bool,     // dword_7829F8
    pub master_volume: f32,               // dword_6C18EC
    pub global_pitch: f32,                // dword_7829F4
    pub frame_time: f32,                  // dword_6C18E4
    pub mix_frames: u32,                  // dword_7816B8
    pub environment: u32,                 // dword_7829FC
    pub environment_level: f32,           // dword_782EA4
    pub doppler_factor: f32,              // dword_6C18F0
    pub speed_of_sound: f32,              // dword_6C18F4
    pub default_min_distance: f32,        // dword_7827C0
    pub default_max_distance: f32,        // dword_7827C4
    pub output_mode: u32,                 // dword_6C18F8
    pub(crate) rng: u32,
    /// port: the music's own rng (list/track picks), so the effect rng stays deterministic
    pub(crate) music_rng: u32,
    /// port: per-launch seeded music picks (no immediate repeats); false = the original's deterministic picks
    pub music_random: bool,

    pub voices: VoiceSystem,
    /// port diagnostics (`VELOCITY_AUDIO_TRACE`): (hash, result) of every PlayEvent since the last drain
    pub trace_plays: Option<Vec<(u32, i32)>>,
    /// port diagnostics: names of the `AudioCommand`s applied since the last drain
    pub trace_cmds: Option<Vec<String>>,
}

pub(crate) fn not_init() -> i32 {
    log::trace!("audio: not initialised");
    err::NOT_INIT
}

impl AudioSystem {
    /// AUDIO_Init.cpp 0x002BA220 (+ `_AUDIO_InitM` 0x003134B0 for the device side).
    pub fn new(params: AudioInitParams, mixer: SharedMixer) -> Self {
        let p = &params;
        let banks = p.banks;
        let groups = p.groups;
        let events = p.events;
        let mut s = Self {
            initialised: false,
            list_changed: false,
            paused: false,
            banks: Pool::new(banks as usize),
            bank_list: Vec::new(),
            // _AUDIO_Init : sample header struct pool = banks * 25
            sample_headers: StackPool::new((banks as f64 * 25.0) as u32),
            groups: Pool::new(groups as usize),
            group_list: Vec::new(),
            group_banks: StackPool::new((groups as f64 * 1.4f32 as f64) as u32),
            group_events: StackPool::new(events),
            group_sounds: StackPool::new((events as f64 * 1.1f32 as f64) as u32),
            group_samples: StackPool::new((events as f64 * 4.0) as u32),
            group_chains: StackPool::new((events as f64 * 0.5) as u32),
            event_table: Vec::with_capacity(events as usize),
            last_search: -1,
            emitters: Pool::new(p.emitters as usize),
            emitter_list: Vec::new(),
            listeners: Pool::new(p.listeners as usize),
            listener_list: Vec::new(),
            samples: Pool::new(p.playing_samples as usize),
            streams: Pool::new((p.streams + p.stream_extra * p.stream_queue.saturating_sub(1)) as usize),
            events: Pool::new(p.active_events as usize),
            event_head: None,
            active_event_count: 0,
            slots: Vec::new(),
            control: vec![ControlParam::default(); p.control_params as usize],
            category_volume: [1.0; 32],
            presets: [[1.0; 32]; 32],
            fade_rate: [0.0; 32],
            fade_time: [0.0; 32],
            fading: 0,
            category_dirty: 0,
            all_dirty: false,
            pitch_all_dirty: false,
            master_volume: 1.0,
            global_pitch: 0.0,
            frame_time: f32::from_bits(0x3DCC_CCCD), // 0.1
            mix_frames: 0,
            environment: 0,
            environment_level: 0.0,
            doppler_factor: p.doppler_factor,
            speed_of_sound: f32::from_bits(0x43AB_A666), // 343.3
            default_min_distance: p.emitter_min_distance,
            default_max_distance: p.emitter_max_distance,
            output_mode: 2,
            rng: 1,
            music_rng: 1,
            music_random: false,
            trace_plays: None,
            trace_cmds: None,
            voices: VoiceSystem::new(mixer),
            params,
        };
        s.initialised = true;
        s
    }

    /// convenience: a system with its own mixer and default parameters.
    pub fn with_default_mixer() -> (Self, SharedMixer) {
        let mixer = Arc::new(Mutex::new(Mixer::new(super::xbox::voice::MIXER_VOICES)));
        (Self::new(AudioInitParams::default(), mixer.clone()), mixer)
    }

    // ---- caller handle slots -------------------------------------------------------------------

    /// a caller-owned `AUDIO_ActiveEvent*` variable.
    pub fn new_slot(&mut self) -> EventSlot {
        self.slots.push(None);
        EventSlot(self.slots.len() as u32 - 1)
    }
    pub fn slot(&self, s: EventSlot) -> Option<EventId> {
        self.slots.get(s.0 as usize).copied().flatten()
    }
    pub fn set_slot(&mut self, s: EventSlot, v: Option<EventId>) {
        if let Some(x) = self.slots.get_mut(s.0 as usize) {
            *x = v;
        }
    }

    /// crt `rand()` (0x003DE030), used by `_AUDIO_RandomFloat` and the game.
    pub fn rand(&mut self) -> i32 {
        self.rng = self.rng.wrapping_mul(214013).wrapping_add(2531011);
        ((self.rng >> 16) & 0x7FFF) as i32
    }

    /// port: crt-style rand() on the music's own stream (seed 1 = the original's sequence).
    pub fn music_rand(&mut self) -> i32 {
        self.music_rng = self.music_rng.wrapping_mul(214013).wrapping_add(2531011);
        ((self.music_rng >> 16) & 0x7FFF) as i32
    }

    /// port: seed the music rng from `seed` and switch to no-repeat picks.
    pub fn randomize_music(&mut self, seed: u32) {
        self.music_rng = seed;
        self.music_random = true;
    }

    /// port: seed from the system time unless hidden (`VELOCITY_HIDDEN`) or golden (`VELOCITY_GOLDEN`) runs.
    pub fn randomize_music_per_launch(&mut self) {
        let hidden = std::env::var_os("VELOCITY_HIDDEN").is_some_and(|v| !v.is_empty() && v != "0");
        if hidden || std::env::var_os("VELOCITY_GOLDEN").is_some() {
            return;
        }
        let ns = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos());
        self.randomize_music((ns ^ (ns >> 32) ^ (std::process::id() as u128) << 7) as u32);
    }

    /// _AUDIO_RandomFloat.cpp 0x002BB910 (-> Armco 0x00013510): rand() / 32767.
    pub fn random_float(&mut self) -> f32 {
        self.rand() as f32 / 32767.0
    }

    // ---- global volumes ------------------------------------------------------------------------

    /// AUDIO_SetMasterVolume.cpp 0x002B9BC0
    pub fn set_master_volume(&mut self, v: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        if !(0.0..=1.0).contains(&v) {
            return err::BAD_PARAM;
        }
        if self.master_volume != v {
            self.master_volume = v;
            self.all_dirty = true;
        }
        0
    }

    /// AUDIO_SetCategoryVolume.cpp 0x002B9C50
    pub fn set_category_volume(&mut self, mask: u32, v: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        self.category_dirty |= mask;
        for i in 0..32 {
            if mask & (1 << i) != 0 {
                if self.category_volume[i] == v {
                    self.category_dirty &= !(1 << i);
                } else {
                    self.category_volume[i] = v;
                }
            }
        }
        0
    }

    /// AUDIO_ApplyVolumePreset.cpp 0x002B9AF0: jump straight to a preset (negative entries are "keep").
    pub fn apply_volume_preset(&mut self, preset: usize) -> i32 {
        if !self.initialised {
            return not_init();
        }
        for i in 0..32 {
            let p = self.presets[preset][i];
            if self.category_volume[i] != p && p >= 0.0 {
                self.category_volume[i] = p;
                self.category_dirty |= 1 << i;
            }
        }
        self.all_dirty = true;
        0
    }

    /// AUDIO_FadeToVolumePreset.cpp 0x002BF0D0
    pub fn fade_to_volume_preset(&mut self, preset: usize, time: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        for i in 0..32 {
            let p = self.presets[preset][i];
            if self.category_volume[i] != p && p >= 0.0 {
                self.fade_rate[i] = (p - self.category_volume[i]) / time;
                self.fade_time[i] = time;
                self.fading |= 1 << i;
                self.category_dirty |= 1 << i;
            }
        }
        self.all_dirty = true;
        0
    }

    /// AUDIO_LoadVolumePresets.cpp 0x002B9D00: 4096 bytes = 32 presets x 32 category floats.
    pub fn load_volume_presets(&mut self, path: &str) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Ok(b) = visos::file::platform::read(path) else { return err::BAD_FILE };
        if b.len() < 4096 {
            return err::READ;
        }
        for (i, c) in b[..4096].chunks_exact(4).enumerate() {
            self.presets[i / 32][i % 32] = f32::from_le_bytes(c.try_into().unwrap());
        }
        0
    }

    /// AUDIO_SetControlParam.cpp 0x002BA0F0 (the original accepts `index == count`, writing one past the end).
    pub fn set_control_param(&mut self, index: usize, value: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        if index > self.control.len() {
            return err::BAD_PARAM;
        }
        let Some(c) = self.control.get_mut(index) else { return 0 };
        c.value = value.clamp(0.0, 1.0);
        c.source = None;
        0
    }

    /// AUDIO_GetControlParam.cpp 0x003147F0: the 0..1 value an event parameter selector reads.
    pub fn control_param(&self, index: usize) -> f32 {
        if !self.initialised || index >= self.control.len() {
            return -1.0;
        }
        let c = &self.control[index];
        match c.source {
            Some(src) => ((src - c.min) / (c.max - c.min)).clamp(0.0, 1.0),
            None => c.value,
        }
    }

    /// AUDIO_SetEnvironment.cpp 0x002B9A00: i3dl2 reverb preset (index 0..7) and level. the dsp effects
    /// image (`voice.bin`) is not emulated; the selection is kept for the mix.
    pub fn set_environment(&mut self, env: u32, level: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        if env > 7 || !(0.0..=1.0).contains(&level) {
            return err::BAD_PARAM;
        }
        self.environment_level = level;
        self.environment = env;
        0
    }

    /// AUDIO_SetDopplerParams.cpp 0x002B9A90
    pub fn set_doppler_params(&mut self, factor: f32, speed_of_sound: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        self.doppler_factor = factor;
        self.speed_of_sound = speed_of_sound;
        0
    }

    /// AUDIO_SetDefaultEmitterDistances.cpp 0x002BA1D0
    pub fn set_default_emitter_distances(&mut self, min: f32, max: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        self.default_min_distance = min;
        self.default_max_distance = max;
        0
    }

    /// AUDIO_SetOutputMode.cpp 0x002BEBF0 (mono/stereo/surround; the mix is always stereo here)
    pub fn set_output_mode(&mut self, mode: u32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        self.output_mode = mode;
        0
    }

    pub fn active_event_count(&self) -> u32 {
        self.active_event_count
    }
    pub fn max_playing_samples(&self) -> usize {
        self.samples.capacity()
    }
    pub fn free_playing_samples(&self) -> usize {
        self.samples.free_count()
    }
    pub fn alloc_playing_samples(&self) -> usize {
        self.samples.alloc_count()
    }

    /// AUDIO_PauseAll.cpp 0x002BD590
    pub fn pause_all(&mut self) -> i32 {
        if !self.initialised {
            return not_init();
        }
        self.paused = true;
        for e in self.emitter_list.clone() {
            if self.emitters.get(e).is_some_and(|e| e.flags & emitter_flags::ALIVE != 0) {
                let r = self.pause_emitter_samples(e);
                if r != 0 {
                    return r;
                }
            }
        }
        0
    }

    /// AUDIO_ResumeAll.cpp 0x002BD610
    pub fn resume_all(&mut self) -> i32 {
        if !self.initialised {
            return not_init();
        }
        self.paused = false;
        for e in self.emitter_list.clone() {
            if self.emitters.get(e).is_some_and(|e| e.flags & emitter_flags::ALIVE != 0) {
                let r = self.resume_emitter_samples(e);
                if r != 0 {
                    return r;
                }
            }
        }
        0
    }

    /// AUDIO_Update.cpp 0x002BCFE0: one engine tick. `dt` in seconds.
    pub fn update(&mut self, mut dt: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let mut result = 0;
        if dt <= 0.0 {
            dt = f32::from_bits(0x3D08_8509); // 0.0333
        }
        if dt > f32::from_bits(0x3D4C_CCCD) {
            dt = f32::from_bits(0x3D4C_CCCD); // 0.05
        }
        self.frame_time = dt;
        self.mix_frames = (dt * 48000.0) as u32;
        for i in 0..32 {
            if self.fading & (1 << i) != 0 {
                if self.fade_time[i] <= 0.0 {
                    // original bug kept: a non-positive time steps by one whole rate
                    self.category_volume[i] -= self.fade_rate[i];
                    self.fade_time[i] += 1.0;
                    self.category_dirty |= 1 << i;
                    if self.fade_time[i] >= 0.0 {
                        self.fading &= !(1 << i);
                    }
                } else {
                    self.category_volume[i] += self.frame_time * self.fade_rate[i];
                    self.fade_time[i] -= self.frame_time;
                    self.category_dirty |= 1 << i;
                    if self.fade_time[i] <= 0.0 {
                        self.fading &= !(1 << i);
                    }
                }
                self.category_volume[i] = self.category_volume[i].clamp(0.0, 1.0);
            }
        }
        // listeners (_AUDIO_UpdateListener -1 / each / -2)
        self.voices.listener_begin();
        for l in self.listener_list.clone() {
            let r = self.update_listener(l);
            if r != 0 {
                result = r;
            }
        }
        self.voices.listener_end();
        // emitters
        for e in self.emitter_list.clone() {
            if !self.emitters.contains(e) {
                continue;
            }
            self.update_emitter_3d(e);
            let r = self.update_emitter_samples(e);
            if r != 0 {
                result = r;
            }
            if let Some(em) = self.emitters.get(e) {
                if em.flags & emitter_flags::AUTO_DESTROY != 0 && em.flags & emitter_flags::HAS_SAMPLES == 0 {
                    self.destroy_emitter(e);
                }
            }
        }
        // active events
        self.active_event_count = 0;
        let mut ev = self.event_head;
        while let Some(e) = ev {
            let next = self.events.get(e).and_then(|a| a.next);
            let r = self.update_active_event(e);
            if r != 0 {
                result = r;
            }
            ev = next;
        }
        // _AUDIO_CommitM(-2): CommitDeferredSettings
        self.commit_voices_m();
        self.all_dirty = false;
        self.pitch_all_dirty = false;
        self.category_dirty = 0;
        // free playing samples no emitter list reaches any more
        for id in self.samples.ids() {
            let ps = self.samples.get(id).unwrap();
            let mut reached = false;
            let mut k = self.emitters.get(ps.emitter).and_then(|e| e.samples);
            while let Some(s) = k {
                if s == id {
                    reached = true;
                    break;
                }
                let mut q = self.samples.get(s).and_then(|x| x.queue_next);
                while let Some(qq) = q {
                    if qq == id {
                        reached = true;
                        break;
                    }
                    q = self.samples.get(qq).and_then(|x| x.queue_next);
                }
                if reached {
                    break;
                }
                k = self.samples.get(s).and_then(|x| x.next);
            }
            if !reached {
                self.stop_voice_m(id);
                self.free_playing_sample(id);
            }
        }
        result
    }

    /// AUDIO_Reset.cpp 0x002BC420: unload everything and rebuild the pools.
    pub fn reset(&mut self) -> i32 {
        if !self.initialised {
            return not_init();
        }
        while let Some(&g) = self.group_list.first() {
            self.unload_group(Some(g), true);
        }
        while let Some(&b) = self.bank_list.first() {
            self.unload_bank(b, true);
        }
        for id in self.samples.ids() {
            self.stop_voice_m(id);
        }
        self.emitters.reset();
        self.listeners.reset();
        self.samples.reset();
        self.streams.reset();
        self.events.reset();
        self.groups.reset();
        self.banks.reset();
        self.bank_list.clear();
        self.group_list.clear();
        self.emitter_list.clear();
        self.listener_list.clear();
        self.event_head = None;
        self.active_event_count = 0;
        self.sample_headers.reset();
        self.group_banks.reset();
        self.group_events.reset();
        self.group_sounds.reset();
        self.group_samples.reset();
        self.group_chains.reset();
        self.event_table.clear();
        for s in &mut self.slots {
            *s = None;
        }
        self.voices.reset();
        0
    }
}

#[cfg(test)]
mod music_rng_tests {
    use super::*;

    #[test]
    fn music_rng_is_separate_and_seedable() {
        let (mut a, _m) = AudioSystem::with_default_mixer();
        let (mut b, _m2) = AudioSystem::with_default_mixer();
        b.randomize_music(0x1234_5678);
        let first: Vec<i32> = (0..8).map(|_| a.music_rand()).collect();
        let other: Vec<i32> = (0..8).map(|_| b.music_rand()).collect();
        assert_ne!(first, other);
        assert!(b.music_random && !a.music_random);
        // the effect stream is untouched by music draws
        let (mut c, _m3) = AudioSystem::with_default_mixer();
        assert_eq!(a.rand(), c.rand());
    }
}
