//! src/Audio/VsAudio.cpp — `g_AudioSystem` (0x006D4558): the game's audio manager. owns the shared emitters,
//! the per-car audio, environment sounds, music, and the game-mode effects (nos, slipstream, zone, ...).
//! (the module begins at the constructor 0x00047910; the assert-based region map starts it at 0x00049280.)

use bevy::math::Vec3;

use crate::audio::system::{AudioSystem, EmitterId, EventSlot, ListenerId};
use crate::input::{AudioSettings, CarAudioInput, GameAudioInput};
use crate::vs_audio_events::{crc, VsAudioEvents};
use crate::vs_audio_track_list::{SavedPlaylists, VsAudioTrackList, FIXED_LISTS};
use crate::vs_audio_vehics::{self as vehics, ImpactState, SuspensionState, VehicleTweaks, VsVehicleAudio};
use crate::vs_soundtrack::{MusicSource, PlayerState, VsSoundtrackPlayer};

/// `VsAudio` flags (+0)
pub mod flags {
    pub const INITIALISED: u32 = 1;
    pub const ENABLED: u32 = 2;
    pub const PAUSED: u32 = 4;
    pub const SOUNDTRACK: u32 = 8;
    pub const ZONE: u32 = 0x10;
    pub const NOS: u32 = 0x20;
    pub const FLAG40: u32 = 0x40;
    pub const MUSIC: u32 = 0x80;
}

/// the render rate the original's per-frame audio steps were tuned at (TV refresh, NTSC; core.md
/// "Original render rate"). VsAudio::Update runs once per rendered frame (GameManager::InGameUpdate 0x000B7EE7).
pub const REFERENCE_RATE: f32 = 60.0;

/// a per-frame exponential factor `alpha` (tuned at 60 Hz) for a frame of `scale` reference frames; exact at 1.
pub fn per_frame(alpha: f32, scale: f32) -> f32 {
    if scale == 1.0 {
        alpha
    } else if (0.0..1.0).contains(&alpha) {
        1.0 - (1.0 - alpha).powf(scale)
    } else {
        alpha * scale
    }
}

/// the per-frame leaky accumulator `acc = (acc + v) * k` (tuned at 60 Hz) over `scale` reference frames: same
/// steady state `v·k/(1-k)` and time constant at any frame rate; the original expression at 1.
pub fn leaky(acc: f32, v: f32, k: f32, scale: f32) -> f32 {
    if scale == 1.0 {
        (acc + v) * k
    } else {
        let ks = k.powf(scale);
        acc * ks + v * k * (1.0 - ks) / (1.0 - k)
    }
}

/// CarDamageFX 0x00078320 glass impact event (995820418)
pub const GLASS_IMPACT: u32 = 0x3B5B_1A82;
const HWE_HOP_ALARM: u32 = 0x4E7E_C896; // 1316931734
const RAIN_LEFT: u32 = 0x0C41_795E; // 205617502 amb_rain_light_left
const RAIN_RIGHT: u32 = 0x926A_0956; // -1838544554 amb_rain_light_right
const START_GIRL_READY: u32 = 0x8F0D_3E23; // -1894957533
const START_GIRL_GO: u32 = 0xBF83_28C4; // -1081923388
const START_AIRHORN: u32 = 0x36D5_0B2A; // 919931690
const ZONE_HEARTBEAT: u32 = 0x678C_58E1; // 1737251041
const LOOP_EVENT_210: u32 = 210; // numbered id, not in this build's data

/// VsAudio.cpp 0x0004A340 / 0x0004A390 / 0x0004A3E0 / 0x0004A430 / 0x0004A480 / 0x0004A4D0 / 0x0004A520 / 0x0004A570 / 0x0004A5C0 / 0x0004A610 / 0x0004A660 PlayGui*: front-end sounds, all on the sfx emitter
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GuiSound {
    Up,
    Down,
    Left,
    Right,
    Accept,
    Back,
    WooshLeft,
    WooshRight,
    Error,
    WooshOut,
    WooshIn,
}

impl GuiSound {
    pub fn event(self) -> u32 {
        match self {
            GuiSound::Up | GuiSound::Left => 0x1EFF_B41F,    // gui_cursor_upleft (520074271)
            GuiSound::Down | GuiSound::Right => 0xBD8F_5885, // gui_cursor_downright (-1114679163)
            GuiSound::Accept => 0x55F6_D369,
            GuiSound::Back => 0xBABE_FDC5,
            GuiSound::WooshLeft => 0x5ADE_053F,
            GuiSound::WooshRight => 0xA889_C7E4,
            GuiSound::Error => 0xE7D5_9038,
            GuiSound::WooshOut => 0x60E7_F9EA,
            GuiSound::WooshIn => 0x8B48_51A1,
        }
    }
}

/// an environment sound from `envaudio.aee` or added at runtime (`VsEnvSound`, 80 bytes)
#[derive(Clone, Debug, Default)]
pub struct VsEnvSound {
    pub position: Vec3,   // +0
    pub direction: Vec3,  // +16
    pub volume: f32,      // +32
    pub min_distance: f32, // +36
    pub unk40: f32,       // +40
    pub max_distance: f32, // +44
    pub oneshot: (f32, f32), // +48/+52
    pub timers: (f32, f32), // +56/+60
    pub emitter: Option<EmitterId>, // +64
    pub event: u32,       // +68
    pub global: bool,     // +72
    pub is_oneshot: bool, // +73
    pub dynamic: bool,    // +74
}

/// one `envaudio.aee` record (76 bytes after the 12-byte `AED1` header)
#[derive(Clone, Debug, PartialEq)]
pub struct AeeRecord {
    pub name: String,
    pub position: Vec3,
    pub direction: Vec3,
    pub min_distance: f32,
    pub volume: f32,
    pub max_distance: f32,
    pub unk40: f32,
    pub event: u32,
}

/// parse `envaudio.aee`: `'AED1' u32 count u32 recordSize(76)` then records
/// `char name[32]; f32 pos[3]; f32 dir[3]; f32 min; f32 vol; f32 max; f32 unk; u32 event`.
pub fn parse_aee(b: &[u8]) -> Option<Vec<AeeRecord>> {
    if b.get(..4)? != b"AED1" {
        return None;
    }
    let count = u32::from_le_bytes(b.get(4..8)?.try_into().ok()?) as usize;
    let f = |o: usize| -> Option<f32> { Some(f32::from_le_bytes(b.get(o..o + 4)?.try_into().ok()?)) };
    let mut v = Vec::with_capacity(count);
    for i in 0..count {
        let o = 12 + 76 * i;
        let name = b.get(o..o + 32)?;
        let end = name.iter().position(|&c| c == 0).unwrap_or(32);
        v.push(AeeRecord {
            name: String::from_utf8_lossy(&name[..end]).into_owned(),
            position: Vec3::new(f(o + 32)?, f(o + 36)?, f(o + 40)?),
            direction: Vec3::new(f(o + 44)?, f(o + 48)?, f(o + 52)?),
            min_distance: f(o + 56)?,
            volume: f(o + 60)?,
            max_distance: f(o + 64)?,
            unk40: f(o + 68)?,
            event: u32::from_le_bytes(b.get(o + 72..o + 76)?.try_into().ok()?),
        });
    }
    Some(v)
}

/// what `VsAudio::Init` learns from the game (front end or race, the track).
#[derive(Clone, Debug, Default)]
pub struct VsAudioInitInfo {
    pub frontend: bool,
    /// the track's data directory as the game info holds it (`Tracks\London\`)
    pub track_dir: Option<String>,
    /// `GameInfo +136`, the race mode the playlist choice depends on
    pub race_mode: Option<u32>,
}

pub struct VsAudio {
    pub flags: u32,                   // +0
    pub saved_master: f32,            // +4
    pub player_emitter: Option<EmitterId>, // +8
    pub race_count: usize,            // +12
    pub tft_count: usize,             // +16
    pub total: usize,                 // +20
    pub traffic_count: usize,         // +24
    pub police_count: usize,          // +28
    pub player_cars: Vec<VsVehicleAudio>, // +32
    pub ai_cars: Vec<VsVehicleAudio>,     // +36
    pub traffic_cars: Vec<VsVehicleAudio>, // +40
    pub police_cars: Vec<VsVehicleAudio>,  // +44
    pub music_emitter: Option<EmitterId>,  // +48
    pub speech_emitter: Option<EmitterId>, // +52
    pub sfx_emitter: Option<EmitterId>,    // +56
    pub ambience_emitter: Option<EmitterId>, // +60
    pub gamefx_emitter: Option<EmitterId>, // +64
    pub impact_emitters: [Option<EmitterId>; 6], // +68
    pub shop_alarm: EventSlot,        // +92
    pub env_alarm: EventSlot,         // +96
    pub shop_env: Option<VsEnvSound>, // +100
    pub frontend: bool,               // +104
    pub settings_dirty: bool,         // +105
    pub engine_restart: bool,         // +106
    pub soundtrack: VsSoundtrackPlayer, // +108
    pub listener: Option<ListenerId>, // +464
    pub fade_rate: f32,               // +468
    pub fade_current: f32,            // +472
    pub fade_target: f32,             // +476
    pub fading: bool,                 // +480
    pub debug_text: f32,              // +484
    pub env_sounds: Vec<VsEnvSound>,  // +488 (head first)
    pub tft_paused: bool,             // +504
    pub tracks: VsAudioTrackList,     // +548..+632
    /// the save's playlist block (global `dword_6F9184`, options +0x28); applied by every InitMusic
    pub saved_lists: SavedPlaylists,
    /// port: this frame's length in reference frames (1 at 60 Hz) for the original's per-frame steps
    pub frame_scale: f32,

    // VsAudio.cpp / VsAudioVehics.cpp statics
    pub rain_emitter: Option<EmitterId>, // dword_6D4540
    pub rain: [EventSlot; 2],           // dword_6D4544/48
    pub city_ambience: EventSlot,       // dword_6D454C
    pub saved_music: f32,               // dword_6D4534
    pub env_restart: bool,              // byte_6D453C
    pub one_shot: EventSlot,            // dword_6D4A48
    pub loop210: EventSlot,             // dword_6D4538
    pub slipstream: [EventSlot; 2],     // dword_6D4A50/54
    pub slipstream_on: bool,            // byte_6D4A58
    pub wind: [EventSlot; 2],           // dword_6D4A5C/60
    pub tb_on: bool,                    // byte_6D4A64
    pub tb_enter: EventSlot,            // dword_6D4A68
    pub tb_atmos: EventSlot,            // dword_6D4A6C
    pub tb_volume: f32,                 // flt_6D4AA4
    pub nos_start: EventSlot,           // dword_6D4A74
    pub nos_loop: EventSlot,            // dword_6D4A80
    pub zone_on: bool,                  // byte_6D4A94
    pub zone: EventSlot,                // dword_6D4A90
    pub mode3_on: bool,                 // byte_6D4AA0
    pub mode3: EventSlot,               // dword_6D4A9C
    pub flag40_ended: bool,             // byte_6D4AA1
    pub under_cover: bool,              // byte_6D5051
    pub wind_event: EventSlot,          // dword_6D504C
    pub wind_keep: bool,                // byte_6D5050
    pub music_event: EventSlot,         // dword_6D5038
    pub traffic_muted: bool,            // byte_6D5054
    pub countdown_started: bool,        // byte_6D5055
    pub countdown_time: f32,            // dword_6D5058
    pub countdown_girl: bool,           // byte_6D505C
    pub countdown_master_off: bool,     // byte_6D5052
    pub start_master_fade: bool,        // byte_6D5053

    // the Audio.* tweak globals (0x00684EC4..)
    pub settings: AudioSettings,
    pub doppler: f32,          // 0x684EC4
    pub speed_of_sound: f32,   // 0x684EC8
    pub last_doppler: (f32, f32), // 0x684ECC/0x684ED0
    pub tft_music_mult: f32,   // 0x684ED4
    pub impact_frames: i32,    // 0x684ED8
    pub envfx_on: bool,        // byte_684EF0
    pub rain_start: bool,      // byte_684EF1
    pub slipstream_param: f32, // 0x684EF8
    pub nos_param: f32,        // 0x684EFC
    pub tb_fade: f32,          // 0x684F08
    pub vehicle_tweaks: VehicleTweaks,

    pub events: VsAudioEvents,
    pub suspension: SuspensionState,
    pub impacts: ImpactState,
    pub race_mode: Option<u32>,
    /// the race definition's wet flag (`this+0x2C78` -> 0x00017CB0 +22 & 1), latched from `LoadLevelSounds`
    pub track_wet: bool,
    /// TimelineSound objects' 2D emitters (object +0x18), by object name crc
    pub timeline_emitters: Vec<(u32, EmitterId)>,
}

impl VsAudio {
    /// VsAudio.cpp 0x0004B0A0 (`GameAudioInput::track_wet` or the flag `LoadLevelSounds` was given)
    pub fn is_track_wet(&self, input: &GameAudioInput) -> bool {
        input.track_wet || self.track_wet
    }

    /// VsAudio.cpp 0x00047910
    pub fn new(audio: &mut AudioSystem) -> Self {
        let s = |a: &mut AudioSystem| a.new_slot();
        Self {
            flags: flags::ENABLED | flags::SOUNDTRACK,
            saved_master: 1.0,
            player_emitter: None,
            race_count: 0,
            tft_count: 0,
            total: 0,
            traffic_count: 0,
            police_count: 0,
            player_cars: Vec::new(),
            ai_cars: Vec::new(),
            traffic_cars: Vec::new(),
            police_cars: Vec::new(),
            music_emitter: None,
            speech_emitter: None,
            sfx_emitter: None,
            ambience_emitter: None,
            gamefx_emitter: None,
            impact_emitters: [None; 6],
            shop_alarm: s(audio),
            env_alarm: s(audio),
            shop_env: None,
            frontend: true,
            settings_dirty: false,
            engine_restart: true,
            soundtrack: VsSoundtrackPlayer::default(),
            listener: None,
            fade_rate: 0.0,
            fade_current: 1.0,
            fade_target: 1.0,
            fading: false,
            debug_text: 0.0,
            env_sounds: Vec::new(),
            tft_paused: false,
            tracks: VsAudioTrackList::default(),
            saved_lists: SavedPlaylists::default(),
            frame_scale: 1.0,
            rain_emitter: None,
            rain: [s(audio), s(audio)],
            city_ambience: s(audio),
            saved_music: 0.0,
            env_restart: false,
            one_shot: s(audio),
            loop210: s(audio),
            slipstream: [s(audio), s(audio)],
            slipstream_on: false,
            wind: [s(audio), s(audio)],
            tb_on: false,
            tb_enter: s(audio),
            tb_atmos: s(audio),
            tb_volume: 0.0,
            nos_start: s(audio),
            nos_loop: s(audio),
            zone_on: false,
            zone: s(audio),
            mode3_on: false,
            mode3: s(audio),
            flag40_ended: false,
            under_cover: false,
            wind_event: s(audio),
            wind_keep: false,
            music_event: s(audio),
            traffic_muted: false,
            countdown_started: false,
            countdown_time: 0.0,
            countdown_girl: false,
            countdown_master_off: false,
            start_master_fade: false,
            settings: AudioSettings::default(),
            doppler: f32::from_bits(0x3E99_999A),
            speed_of_sound: f32::from_bits(0x43AB_A666),
            last_doppler: (f32::from_bits(0x3E99_999A), f32::from_bits(0x43AB_A666)),
            tft_music_mult: f32::from_bits(0x3F66_6666),
            impact_frames: 15,
            envfx_on: true,
            rain_start: true,
            slipstream_param: f32::from_bits(0x3DCC_CCCD),
            nos_param: f32::from_bits(0x3DCC_CCCD),
            tb_fade: f32::from_bits(0x3C08_8723),
            vehicle_tweaks: VehicleTweaks::default(),
            events: VsAudioEvents::default(),
            suspension: SuspensionState::default(),
            impacts: ImpactState::default(),
            race_mode: None,
            track_wet: false,
            timeline_emitters: Vec::new(),
        }
    }

    fn enabled(&self) -> bool {
        self.flags & flags::ENABLED != 0
    }

    fn create(audio: &mut AudioSystem, f: u32) -> Option<EmitterId> {
        audio.create_emitter(f).ok()
    }

    /// VsAudio.cpp 0x00047A00
    pub fn init(&mut self, audio: &mut AudioSystem, info: &VsAudioInitInfo) -> i32 {
        if !self.enabled() {
            return 0;
        }
        self.frontend = info.frontend;
        self.race_mode = info.race_mode;
        self.track_wet = false;
        self.settings_dirty = false;
        if self.frontend {
            audio.set_default_emitter_distances(1.0, 25.0);
            self.flags |= flags::MUSIC;
        } else {
            audio.set_default_emitter_distances(10.0, 200.0);
        }
        self.init_music(audio);
        if self.frontend {
            (self.race_count, self.tft_count, self.total, self.traffic_count, self.police_count) = (0, 0, 0, 0, 0);
        } else {
            self.race_count = 1;
            self.tft_count = 6;
            self.total = self.race_count + self.tft_count;
            self.traffic_count = 20;
            self.police_count = 3;
            self.player_cars = (0..self.race_count).map(|_| VsVehicleAudio::new(audio)).collect();
            self.ai_cars = (0..self.total).map(|_| VsVehicleAudio::new(audio)).collect();
            self.traffic_cars = (0..self.traffic_count).map(|_| VsVehicleAudio::new(audio)).collect();
            self.police_cars = (0..self.police_count).map(|_| VsVehicleAudio::new(audio)).collect();
            for p in [1, 3, 16, 2, 4, 17, 5] {
                audio.set_control_param(p, 1.0);
            }
        }
        self.fade_rate = 0.0;
        self.fade_current = 1.0;
        self.fade_target = 1.0;
        self.fading = false;
        match audio.create_listener(0x4000_0000) {
            Ok(l) => self.listener = Some(l),
            Err(e) => {
                log::warn!("Error creating Audio emitters");
                return e;
            }
        }
        for i in 0..self.total {
            if i != 0 {
                let r = audio.rand();
                let va = &mut self.ai_cars[i];
                va.active = false;
                va.siren_state = (r % 3 + 2) as i8;
                va.horn = u32::MAX;
                va.emitters[0] = Self::create(audio, 4);
                va.emitters[3] = None;
            } else {
                let r = audio.rand();
                let va = &mut self.player_cars[0];
                for e in va.emitters.iter_mut() {
                    *e = Self::create(audio, 0x4000_0004);
                }
                va.active = false;
                va.siren_state = (r % 3 + 2) as i8;
                va.horn = u32::MAX;
                va.surface = [u32::MAX, u32::MAX];
            }
        }
        for (k, va) in self.traffic_cars.iter_mut().enumerate() {
            va.active = false;
            va.horn = vehics::HORNS[k % 3];
        }
        self.music_emitter = Self::create(audio, 0x4000_0001);
        self.speech_emitter = Self::create(audio, 0x4000_0001);
        self.sfx_emitter = Self::create(audio, 0x4000_0001);
        self.ambience_emitter = Self::create(audio, 0x4000_0001);
        self.gamefx_emitter = Self::create(audio, 0x4000_0001);
        for e in self.impact_emitters.iter_mut() {
            *e = Self::create(audio, 0x4000_0004);
        }
        self.player_emitter = Self::create(audio, 0x4000_0001);
        self.rain_emitter = Self::create(audio, 0x4000_0001);
        if audio.load_volume_presets(r"\Audio\velocity.vol") != 0 {
            log::warn!("Error Loading volume presets");
        }
        audio.set_category_volume(u32::MAX, 1.0);
        audio.set_master_volume(if self.frontend { 1.0 } else { 0.0 });
        audio.apply_volume_preset(0);
        if !self.frontend {
            if let Some(dir) = info.track_dir.clone() {
                self.load_env_audio(audio, &dir);
            }
        }
        self.flags |= flags::INITIALISED;
        audio.set_doppler_params(self.doppler, self.speed_of_sound);
        audio.set_environment(6, f32::from_bits(0x3C23_D70A));
        self.impacts.history = [0; 10];
        self.tft_paused = false;
        if let Some(e) = self.music_emitter {
            self.soundtrack.init(audio, e);
        }
        0
    }

    /// GameDataLoading.cpp 0x000A1190 ("Load Level Sounds", load step `LevelSounds`): the race's groups.
    /// `track_dir` as in `VsAudioInitInfo` (`Tracks\LADOWN\`), `wet` = `VsAudio::IsTrackWet`. The "Audio SVG
    /// preload" of `\Audio\allSVG.vcd` before it is a read-ahead cache and not needed here.
    pub fn load_level_sounds(&mut self, audio: &mut AudioSystem, track_dir: &str, wet: bool) {
        if !self.enabled() {
            return;
        }
        self.track_wet = wet;
        let load = |audio: &mut AudioSystem, path: &str, name: &str| {
            if audio.load_group(path, None).is_err() {
                log::warn!("Failed to load audio group '{name}'");
            }
        };
        if wet {
            load(audio, r"\Audio\surface\tarmac\sftarmw", "sftarmw");
            load(audio, r"\Audio\envfx\enrain", "enrain");
            load(audio, r"\Audio\passbys\tarmac\pbtarmw", "imcar");
        } else {
            load(audio, r"\Audio\surface\tarmac\sftarm", "sftarm");
            load(audio, r"\Audio\surface\dirt\sfdirt", "sfdirt");
            // the sfgravl failure only logs ("Fritz got fed up and disabled the assert")
            load(audio, r"\Audio\surface\gravel\sfgravl", "sfdirt");
            load(audio, r"\Audio\passbys\tarmac\pbtarm", "imcar");
        }
        for (p, n) in [
            (r"\Audio\Impact\imcar", "imcar"),
            (r"\Audio\Impact\imfoliag", "imfoliag"),
            (r"\Audio\Impact\imglass", "imglass"),
            (r"\Audio\Impact\immetal", "immetal"),
            (r"\Audio\Impact\implasti", "imfoliag"),
            (r"\Audio\Impact\imstone", "imstone"),
            (r"\Audio\Impact\imwood", "imstone"),
            (r"\Audio\vehicle\fx\npc_fx", "Horn"),
            (r"\Audio\fx\mddrift", "mddrift"),
            (r"\Audio\fx\mdslipst", "mdslipst"),
            (r"\Audio\fx\mdzone", "mdzone"),
            (r"\Audio\engupgr\nos\unos__01", "nos"),
            (r"\Audio\misc\misc", "misc"),
            (r"\Audio\vehicle\npc\vpoliclo", "vpoliclo"),
        ] {
            load(audio, p, n);
        }
        // EnvFX: "\<track dir>Audio\" + "en" + the track name cut to 6 chars (or at its '\') + 'm' / 's'
        let base = format!(r"\{track_dir}Audio\");
        let mut en: Vec<u8> = b"en".to_vec();
        en.extend_from_slice(track_dir.as_bytes().get(7..).unwrap_or(&[]));
        let cut = if en.len() > 6 { 7 } else { en.iter().position(|&c| c == b'\\').unwrap_or(en.len()) };
        en.truncate(cut);
        let en = String::from_utf8_lossy(&en).into_owned();
        load(audio, &format!("{base}{en}m"), "EnvFX");
        load(audio, &format!("{base}{en}s"), "EnvFX");
        // special event groups (0x000A21A6): "\Tracks\LON..." but not "...2" = London's petrol station; else
        // "\Tracks\LAD..." = LADown's (aschina/aschinam are not shipped), wet tarmac skids and the trailblazer
        let b = base.as_bytes();
        if b.get(8) == Some(&b'L') && b.get(9) == Some(&b'O') && b.get(10) == Some(&b'N') && b.get(14) != Some(&b'2') {
            for n in ["asPetrol", "asPetros", "ashop"] {
                load(audio, &format!("{base}{n}"), "PetrlStn");
            }
        } else if b.get(8) == Some(&b'L') && b.get(9) == Some(&b'A') && b.get(10) == Some(&b'D') {
            for n in ["aschina", "aschinam", "asbilbrd", "ascarpk", "ascarpkm", "ascarpks", "ashydrnt"] {
                load(audio, &format!("{base}{n}"), "PetrlStn");
            }
            load(audio, r"\Audio\surface\tarmac\sftarmw", "sftarmw");
            load(audio, r"\Audio\fx\mdtrail", "mdtrail");
            load(audio, r"\Audio\fx\mdtrails", "mdtrail");
        }
        load(audio, r"\Audio\FX\racestar", r"EnvFX\racestar");
    }

    /// VsAudio.cpp 0x00049280: the front end's groups.
    pub fn load_front_end_groups(&mut self, audio: &mut AudioSystem) -> i32 {
        for (g, n) in [(r"\Audio\GUI\FE_GUI", "FEGUI"), (r"\Audio\liptest\liptest", "liptest"), (r"\Audio\Envfx\enshowro", "enshowro")] {
            if audio.load_group(g, None).is_err() {
                log::warn!("Failed to load audio group '{n}'");
            }
        }
        // fe_guifx holds the woosh events PlayGuiWoosh* use; the front end loads it with its screens
        let _ = audio.load_group(r"\Audio\GUI\FE_GUIFX", None);
        0
    }

    /// VsAudio.cpp 0x000488A0
    pub fn shutdown(&mut self, audio: &mut AudioSystem) -> i32 {
        self.flags &= !flags::MUSIC;
        if !self.enabled() {
            return 0;
        }
        if !self.frontend && self.events.stop_player_engine(audio) {
            self.engine_restart = true;
        }
        audio.stop_event(None);
        audio.stop_all_emitter_samples();
        // the timeline objects (and their emitters) go with the race
        for (_, e) in self.timeline_emitters.drain(..) {
            audio.destroy_emitter(e);
        }
        self.env_sounds.clear();
        audio.unload_group(None, true);
        self.soundtrack.stop(audio);
        audio.reset();
        for va in self.player_cars.iter_mut().chain(&mut self.ai_cars).chain(&mut self.traffic_cars).chain(&mut self.police_cars) {
            va.release(audio);
        }
        // VsAudio.cpp 0x00049420 FreeVehicleAudio
        self.player_cars.clear();
        self.ai_cars.clear();
        self.traffic_cars.clear();
        self.police_cars.clear();
        0
    }

    /// VsAudio.cpp 0x00049090
    pub fn pause(&mut self, audio: &mut AudioSystem, input: &GameAudioInput) {
        if self.enabled() {
            self.flags |= flags::PAUSED;
            self.saved_master = audio.master_volume;
            audio.set_master_volume(0.0);
            audio.pause_all();
            self.update(audio, 0.0, input);
            if self.flags & flags::SOUNDTRACK != 0 {
                self.soundtrack.pause(audio);
            }
        }
    }

    /// VsAudio.cpp 0x00049110
    pub fn resume(&mut self, audio: &mut AudioSystem, input: &GameAudioInput) {
        if self.enabled() {
            self.flags &= !flags::PAUSED;
            audio.set_master_volume(self.saved_master);
            audio.resume_all();
            self.update(audio, 0.0, input);
            if self.flags & flags::SOUNDTRACK != 0 && self.soundtrack.state == PlayerState::Paused {
                self.soundtrack.play(audio, &self.tracks);
            }
        }
    }

    /// VsAudio.cpp 0x00049190
    pub fn start_master_fade(&mut self, audio: &AudioSystem, target: f32, time: f32) {
        self.fade_rate = (target - audio.master_volume) / time;
        self.fading = true;
        self.fade_target = target;
        self.fade_current = audio.master_volume;
        log::debug!("Starting fade - delta = {}", self.fade_rate);
    }

    /// the options screen changed the volumes (`+105`).
    pub fn apply_settings(&mut self, s: AudioSettings) {
        self.settings = s;
        self.settings_dirty = true;
    }

    /// VsAudio.cpp 0x00048BC0: the game's audio tick.
    pub fn update(&mut self, audio: &mut AudioSystem, dt: f32, input: &GameAudioInput) -> i32 {
        audio.stop_slot(self.one_shot);
        if !self.enabled() {
            return 0;
        }
        let r = audio.update(dt);
        if r != 0 {
            return r;
        }
        // AUDIO_Update's clamped frame time, in reference frames
        self.frame_scale = audio.frame_time * REFERENCE_RATE;
        if self.flags & flags::PAUSED != 0 || self.flags & flags::INITIALISED == 0 {
            return 0;
        }
        if self.settings_dirty {
            let was_zero = self.settings.master == 0.0;
            self.saved_music = self.settings.music;
            self.settings_dirty = false;
            audio.set_master_volume(self.settings.master);
            if self.settings.music == 0.0 || self.settings.master == 0.0 {
                self.stop_soundtrack(audio);
            }
            if self.settings.master == 0.0 {
                if self.events.stop_player_engine(audio) {
                    self.engine_restart = true;
                }
                if !self.env_sounds.is_empty() {
                    self.env_restart = true;
                    for env in &mut self.env_sounds {
                        if let Some(e) = env.emitter.take() {
                            audio.stop_emitter_samples(e);
                            audio.destroy_emitter(e);
                        }
                    }
                }
                audio.stop_all_emitter_samples();
                return 0;
            }
            if was_zero {
                if let Some(e) = self.player_cars.first().and_then(|p| p.emitters[0]) {
                    self.events.start_player_engine(e);
                }
            }
        }
        let s = self.settings.clone();
        for (e, v) in [(self.speech_emitter, s.speech), (self.sfx_emitter, s.sfx), (self.gamefx_emitter, s.sfx)] {
            if let Some(e) = e {
                audio.set_emitter_volume(e, v);
            }
        }
        // VsAudio.cpp 0x0004A710 SetOutputMode
        audio.set_output_mode(s.output_mode + 1);
        if self.flags & flags::SOUNDTRACK != 0 {
            self.soundtrack.update(audio);
            self.update_music(audio);
        }
        if self.frontend {
            return 0;
        }
        if s.master == 0.0 {
            return 0;
        }
        if self.flags & flags::NOS != 0 {
            self.update_nos(audio, input.player.as_ref());
        }
        self.update_tb(audio);
        self.age_impact_history();
        self.update_slipstream(audio);
        let r = self.update_listener(audio, input);
        if r != 0 {
            return r;
        }
        self.update_env_sounds(audio, input);
        if self.is_track_wet(input) {
            self.start_rain(audio);
        }
        self.update_vehicles(audio, input);
        self.update_wind(audio, input);
        if self.fading {
            let step = if dt >= 0.1 { 0.1 } else { dt };
            self.fade_current += self.fade_rate * step;
            if (self.fade_rate < 0.0 && self.fade_target >= self.fade_current)
                || (self.fade_rate > 0.0 && self.fade_current >= self.fade_target)
            {
                self.fade_current = self.fade_target;
                self.fading = false;
            }
            audio.set_master_volume(self.fade_current);
        }
        0
    }

    /// VsAudio.cpp 0x0004B180: listener at the camera; facing with x/z flipped from the camera matrix.
    pub fn update_listener(&mut self, audio: &mut AudioSystem, input: &GameAudioInput) -> i32 {
        let Some(cam) = &input.camera else { return -1 };
        let front = Vec3::new(-cam.forward.x, cam.forward.y, -cam.forward.z);
        let up = cam.up;
        if front.length_squared() < 0.01 || up.length_squared() < 0.01 {
            return 2;
        }
        let Some(l) = self.listener else { return 0 };
        let r = audio.set_listener_position(l, cam.position);
        if r != 0 {
            return r;
        }
        let v = if cam.follows_player { input.player.as_ref().map(|p| p.velocity).unwrap_or(Vec3::ZERO) } else { Vec3::ZERO };
        let r = audio.set_listener_velocity(l, v);
        if r != 0 {
            return r;
        }
        audio.set_listener_direction(l, front, up)
    }

    fn listener_position(&self, audio: &AudioSystem) -> Vec3 {
        self.listener.and_then(|l| audio.listeners.get(l)).map(|l| l.position).unwrap_or(Vec3::ZERO)
    }

    // ---- environment ---------------------------------------------------------------------------

    /// VsAudio.cpp 0x00049570: `\<track dir>envaudio.aee`.
    pub fn load_env_audio(&mut self, audio: &mut AudioSystem, track_dir: &str) -> bool {
        let _ = audio;
        let Ok(b) = visos::file::platform::read(&format!("\\{track_dir}envaudio.aee")) else { return false };
        self.env_restart = true;
        let Some(recs) = parse_aee(&b) else { return false };
        for r in recs {
            let mut env = VsEnvSound::default();
            if r.name.contains("GLOBAL_") {
                env.global = true;
            }
            if let Some(i) = r.name.find("ONESHOT_") {
                let t = &r.name[i + 8..];
                let (a, b) = t.split_once(':').unwrap_or((t, "0"));
                env.oneshot = (atof(a), atof(b));
                env.is_oneshot = true;
            }
            env.position = Vec3::new(r.position.x, r.position.y, 0.0 - r.position.z);
            env.direction = r.direction;
            env.event = r.event;
            env.min_distance = r.min_distance;
            env.volume = r.volume;
            env.max_distance = r.max_distance;
            env.unk40 = r.unk40;
            if r.event == HWE_HOP_ALARM {
                self.shop_env = Some(env);
            } else {
                self.env_sounds.insert(0, env);
            }
        }
        true
    }

    /// VsAudio.cpp 0x000499F0: a dynamic positional loop (physics objects).
    pub fn add_env_sound(&mut self, event: u32, pos: Vec3, min: f32, vol: f32, max: f32, unk40: f32) {
        self.env_sounds.insert(
            0,
            VsEnvSound {
                position: pos,
                volume: vol,
                min_distance: min,
                max_distance: max,
                unk40,
                event,
                global: true,
                dynamic: true,
                ..Default::default()
            },
        );
    }

    /// VsAudio.cpp 0x00049AC0
    fn restart_city_ambience(&mut self, audio: &mut AudioSystem) {
        if self.enabled() && audio.slot(self.city_ambience).is_none() && self.envfx_on {
            if let Some(e) = self.ambience_emitter {
                audio.play_event(e, crc("CITY_AMBIENCE"), None, 0, Some(self.city_ambience));
                if let Some(ps) = audio.slot_sample(self.city_ambience) {
                    audio.set_sample_volume(ps, f32::from_bits(0x3EE6_6666)); // 0.45
                }
                audio.set_emitter_volume(e, self.settings.sfx);
            }
        }
    }

    /// VsAudio.cpp 0x00049B80
    pub fn update_env_sounds(&mut self, audio: &mut AudioSystem, input: &GameAudioInput) {
        if self.doppler != self.last_doppler.0 || self.speed_of_sound != self.last_doppler.1 {
            audio.set_doppler_params(self.doppler, self.speed_of_sound);
            self.last_doppler = (self.doppler, self.speed_of_sound);
        }
        if self.env_restart {
            self.restart_city_ambience(audio);
            self.env_restart = false;
        } else if !self.envfx_on {
            self.env_restart = true;
            if let Some(e) = audio.slot(self.city_ambience) {
                audio.stop_event(Some(e));
            }
        }
        if audio.slot(self.city_ambience).is_some() {
            if input.tft_active {
                if !self.tft_paused {
                    if let Some(ps) = audio.slot_sample(self.city_ambience) {
                        audio.pause_sample(ps);
                    }
                    self.saved_music = self.settings.music;
                    self.tft_paused = true;
                }
                self.settings.music *= self.tft_music_mult;
            } else if self.tft_paused {
                if let Some(ps) = audio.slot_sample(self.city_ambience) {
                    audio.resume_sample(ps);
                }
                self.settings.music = self.saved_music;
                self.tft_paused = false;
            }
            if let Some(e) = self.ambience_emitter {
                audio.set_emitter_volume(e, self.settings.sfx);
            }
        }
        let lp = self.listener_position(audio);
        let sfx = self.settings.sfx;
        for env in self.env_sounds.iter_mut() {
            let d2 = env.position.distance_squared(lp);
            let r = (env.max_distance + 10.0) * (env.max_distance + 10.0);
            if r >= d2 && self.envfx_on {
                if env.emitter.is_none() {
                    if let Ok(e) = audio.create_emitter(0x4000_0004) {
                        env.emitter = Some(e);
                        audio.set_emitter_position(e, env.position);
                        audio.set_emitter_velocity(e, Vec3::ZERO);
                        audio.set_emitter_direction(e, env.direction, Vec3::Y);
                        audio.set_emitter_distances(e, env.min_distance, env.max_distance);
                        // aee volumes are percentages, so this call fails its range check like the original
                        audio.set_emitter_volume(e, env.volume * sfx);
                        if let Some(em) = audio.emitters.get_mut(e) {
                            em.distance_gain = 0.0;
                        }
                        let out = if env.event == HWE_HOP_ALARM { Some(self.env_alarm) } else { None };
                        audio.play_event(e, env.event, None, 0, out);
                    }
                }
            } else if let Some(e) = env.emitter.take() {
                audio.stop_emitter_samples(e);
                audio.destroy_emitter(e);
            }
        }
    }

    /// VsAudio.cpp 0x00049F10
    pub fn enter_cover(&mut self, audio: &mut AudioSystem, wet: bool) {
        self.under_cover = true;
        if wet {
            for s in self.rain {
                if let Some(e) = audio.slot(s) {
                    audio.stop_event(Some(e));
                }
            }
        }
        audio.set_environment(3, f32::from_bits(0x3F66_6666));
    }

    /// VsAudio.cpp 0x00049F80
    pub fn exit_cover(&mut self, audio: &mut AudioSystem, wet: bool) {
        self.under_cover = false;
        if wet {
            self.start_rain_pair(audio);
        }
        audio.set_environment(6, f32::from_bits(0x3C23_D70A));
    }

    fn start_rain_pair(&mut self, audio: &mut AudioSystem) {
        let Some(e) = self.rain_emitter else { return };
        for (slot, ev, pan) in [(self.rain[0], RAIN_LEFT, -1.0), (self.rain[1], RAIN_RIGHT, 1.0)] {
            if audio.slot(slot).is_none() {
                audio.play_event(e, ev, None, 0, Some(slot));
                if let Some(ps) = audio.slot_sample(slot) {
                    audio.samples.get_mut(ps).unwrap().flags |= 2;
                    audio.set_sample_pan(ps, pan);
                }
            }
        }
    }

    /// VsAudio.cpp 0x0004A740: rain follows the listener.
    pub fn start_rain(&mut self, audio: &mut AudioSystem) {
        let lp = self.listener_position(audio);
        if let Some(e) = self.rain_emitter {
            audio.set_emitter_position(e, lp);
            if self.rain_start {
                for (slot, ev, pan) in [(self.rain[0], RAIN_LEFT, -1.0), (self.rain[1], RAIN_RIGHT, 1.0)] {
                    audio.play_event(e, ev, None, 0, Some(slot));
                    if let Some(ps) = audio.slot_sample(slot) {
                        audio.samples.get_mut(ps).unwrap().flags |= 2;
                        audio.set_sample_pan(ps, pan);
                    }
                }
                self.rain_start = false;
            }
            audio.set_emitter_volume(e, self.settings.sfx);
        }
        audio.set_control_param(8, f32::from_bits(0x3F4C_CCCD));
    }

    /// VsAudio.cpp 0x0004A0B0
    pub fn duck_city_ambience(&mut self, audio: &mut AudioSystem) {
        if let Some(ps) = audio.slot_sample(self.city_ambience) {
            audio.set_sample_volume(ps, f32::from_bits(0x3E19_999A));
        }
    }

    /// VsAudio.cpp 0x0004A0F0
    pub fn unduck_city_ambience(&mut self, audio: &mut AudioSystem) {
        if let Some(ps) = audio.slot_sample(self.city_ambience) {
            audio.set_sample_volume(ps, f32::from_bits(0x3EE6_6666));
        }
    }

    /// VsAudio.cpp 0x0004A130: drive into the petrol station shop.
    pub fn enter_shop(&mut self, audio: &mut AudioSystem, wet: bool) {
        self.duck_city_ambience(audio);
        if let Some(e) = self.sfx_emitter {
            audio.play_event(e, HWE_HOP_ALARM, None, 0, Some(self.shop_alarm));
        }
        if !self.env_sounds.iter().any(|e| e.event == HWE_HOP_ALARM) {
            if let Some(env) = self.shop_env.clone() {
                self.env_sounds.insert(0, env);
            }
        }
        self.enter_cover(audio, wet);
    }

    /// VsAudio.cpp 0x0004A210
    pub fn exit_shop(&mut self, audio: &mut AudioSystem, wet: bool) {
        self.unduck_city_ambience(audio);
        audio.stop_slot(self.shop_alarm);
        self.exit_cover(audio, wet);
    }

    /// VsAudio.cpp 0x0004A260 (the env sound's emitter is dropped without being destroyed)
    pub fn remove_shop_alarm(&mut self, audio: &mut AudioSystem) {
        audio.stop_slot(self.shop_alarm);
        if let Some(i) = self.env_sounds.iter().position(|e| e.event == HWE_HOP_ALARM) {
            self.env_sounds.remove(i);
            if let Some(e) = audio.slot(self.env_alarm) {
                audio.stop_event(Some(e));
            }
        }
    }

    /// VsAudio.cpp 0x0004AD60
    pub fn start_city_ambience(&mut self, audio: &mut AudioSystem) {
        if self.enabled() && audio.slot(self.city_ambience).is_none() && self.envfx_on {
            if let Some(e) = self.ambience_emitter {
                audio.play_event(e, crc("CITY_AMBIENCE"), None, 0, Some(self.city_ambience));
                if let Some(ps) = audio.slot_sample(self.city_ambience) {
                    audio.set_sample_volume(ps, f32::from_bits(0x3F11_EB85)); // 0.57
                }
            }
        }
    }

    /// VsAudio.cpp 0x0004AE10
    pub fn stop_city_ambience(&mut self, audio: &mut AudioSystem) {
        if self.enabled() {
            audio.stop_slot(self.city_ambience);
        }
    }

    // ---- timeline sounds (TimelineSound.cpp; the 3D path never runs, see velocity_game timeline_sound) ----

    /// TimelineSound.cpp 0x0017EA00: the object's 2D emitter (0x0017E7F0, flags 0x40000001, made on first use),
    /// volume 1, then the event by crc.
    pub fn play_timeline_sound(&mut self, audio: &mut AudioSystem, object: u32, event: u32) -> i32 {
        let e = match self.timeline_emitters.iter().find(|(o, _)| *o == object) {
            Some(&(_, e)) => e,
            None => match audio.create_emitter(0x4000_0001) {
                Ok(e) => {
                    self.timeline_emitters.push((object, e));
                    e
                }
                Err(r) => return r,
            },
        };
        audio.set_emitter_volume(e, 1.0);
        audio.play_event(e, event, None, 0, None)
    }

    /// TimelineSound.cpp 0x0017EBD0 (`Some`, AUDIO 0x00304F30) / 0x0017EB80 (`None`, AUDIO_StopEmitterSamples)
    pub fn stop_timeline_sound(&mut self, audio: &mut AudioSystem, object: u32, event: Option<u32>) {
        let Some(&(_, e)) = self.timeline_emitters.iter().find(|(o, _)| *o == object) else { return };
        match event {
            Some(h) => audio.stop_emitter_event(e, h),
            None => audio.stop_emitter_samples(e),
        };
    }

    // ---- front end -----------------------------------------------------------------------------

    /// VsAudio::PlayGui* (0x0004A340..0x0004A660)
    pub fn play_gui(&mut self, audio: &mut AudioSystem, s: GuiSound) -> i32 {
        if !self.enabled() {
            return 0;
        }
        match self.sfx_emitter {
            Some(e) => audio.play_event(e, s.event(), None, 0, None),
            None => 0,
        }
    }

    /// VsAudio.cpp 0x0004AAB0
    pub fn play_sfx_by_name(&mut self, audio: &mut AudioSystem, name: &str) -> i32 {
        match self.sfx_emitter {
            Some(e) => audio.play_event(e, crc(name), None, 0, None),
            None => 0,
        }
    }

    /// VsAudio.cpp 0x0004AB00: boot, two doors and the bonnet open in the showroom.
    pub fn play_car_open(&mut self, audio: &mut AudioSystem) {
        if let Some(e) = self.sfx_emitter {
            for n in ["CAR_BOOT_OPEN", "CAR_DOOR_OPEN", "CAR_DOOR_OPEN", "CAR_HOOD_OPEN"] {
                audio.play_event(e, crc(n), None, 0, None);
            }
        }
    }

    /// VsAudio.cpp 0x0004AC30
    pub fn play_car_close(&mut self, audio: &mut AudioSystem) {
        if let Some(e) = self.sfx_emitter {
            for n in ["CAR_BOOT_CLOSE", "CAR_DOOR_CLOSE", "CAR_DOOR_CLOSE", "CAR_HOOD_CLOSE"] {
                audio.play_event(e, crc(n), None, 0, None);
            }
        }
    }

    /// VsAudio.cpp 0x0004A8D0
    pub fn start_loop_event_210(&mut self, audio: &mut AudioSystem) {
        if audio.slot(self.loop210).is_none() {
            if let Some(e) = self.player_emitter {
                audio.play_event(e, LOOP_EVENT_210, None, 0, Some(self.loop210));
            }
        }
    }

    /// VsAudio.cpp 0x0004A920
    pub fn stop_loop_event_210(&mut self, audio: &mut AudioSystem) {
        if let Some(e) = audio.slot(self.loop210) {
            audio.stop_event(Some(e));
        }
    }

    /// VsAudio.cpp 0x0004A960: leaving a race.
    pub fn stop_game_audio(&mut self, audio: &mut AudioSystem) {
        self.stop_soundtrack(audio);
        self.flags &= !flags::MUSIC;
        self.events.stop_player_engine(audio);
        self.engine_restart = true;
        if !self.env_sounds.is_empty() {
            self.env_restart = true;
            self.env_sounds.retain_mut(|env| {
                if let Some(e) = env.emitter.take() {
                    audio.stop_emitter_samples(e);
                    audio.destroy_emitter(e);
                }
                !env.dynamic
            });
        }
        audio.stop_event(None);
        audio.stop_all_emitter_samples();
        self.tft_paused = false;
    }

    /// VsAudio.cpp 0x0004AF30
    pub fn set_tunnel_reverb(&mut self, audio: &mut AudioSystem) {
        if self.enabled() {
            audio.set_environment(3, f32::from_bits(0x3F66_6666));
        }
    }

    /// VsAudio.cpp 0x0004AF70
    pub fn set_default_reverb(&mut self, audio: &mut AudioSystem) {
        if self.enabled() {
            audio.set_environment(6, f32::from_bits(0x3C23_D70A));
        }
    }

    /// VsAudio.cpp 0x0004AFB0
    /// (`impact_frames` rendered frames at the reference rate; counted in reference frames for any frame rate)
    fn age_impact_history(&mut self) {
        self.impacts.age += self.frame_scale;
        if self.impacts.age >= self.impact_frames as f32 {
            self.impacts.history = [0; 10];
            self.impacts.age = 0.0;
        }
    }

    // ---- game modes ----------------------------------------------------------------------------

    fn rear_emitter(&self) -> Option<EmitterId> {
        self.player_cars.first().and_then(|p| p.emitters[9])
    }

    fn play_panned(&self, audio: &mut AudioSystem, e: Option<EmitterId>, slot: EventSlot, ev: u32, pan: f32) {
        if audio.slot(slot).is_none() {
            if let Some(e) = e {
                audio.play_event(e, ev, None, 0, Some(slot));
                if let Some(ps) = audio.slot_sample(slot) {
                    audio.samples.get_mut(ps).unwrap().flags |= 2;
                    audio.set_sample_pan(ps, pan);
                }
            }
        }
    }

    /// VsAudio.cpp 0x0004B480
    pub fn start_nos(&mut self, audio: &mut AudioSystem) {
        if !self.enabled() {
            return;
        }
        audio.fade_to_volume_preset(4, f32::from_bits(0x3E4C_CCCD));
        self.flags |= flags::NOS;
        if self.zone_on {
            self.stop_zone(audio);
        }
        if let Some(e) = audio.slot(self.nos_start) {
            audio.stop_event(Some(e));
        }
        if let Some(e) = self.rear_emitter() {
            audio.play_event(e, crc("CAR_NOS_START"), None, 0, Some(self.nos_start));
        }
        self.nos_param = 0.0;
        audio.set_control_param(12, 0.0);
        self.play_panned(audio, self.gamefx_emitter, self.wind[0], crc("WINDNOISE_01_LEFT"), -1.0);
        self.play_panned(audio, self.gamefx_emitter, self.wind[1], crc("WINDNOISE_01_RIGHT"), 1.0);
    }

    /// VsAudio.cpp 0x0004B6A0
    pub fn update_nos(&mut self, audio: &mut AudioSystem, player: Option<&CarAudioInput>) {
        if self.flags & flags::NOS == 0 {
            return;
        }
        if audio.slot(self.nos_start).is_none() && audio.slot(self.nos_loop).is_none() {
            if let Some(e) = self.rear_emitter() {
                audio.play_event(e, crc("CAR_NOS_LOOP"), None, 0, Some(self.nos_loop));
            }
        }
        let speed = player.map(|p| p.speed).unwrap_or(0.0);
        let mut t = speed * f32::from_bits(0x4066_6666) * 0.625 / 150.0;
        if t > 1.0 {
            t = 1.0;
        }
        // steps per reference frame
        if t <= self.nos_param {
            if self.nos_param > t {
                self.nos_param -= f32::from_bits(0x3CA3_D70A) * self.frame_scale;
            }
        } else {
            self.nos_param += f32::from_bits(0x3C23_D70A) * self.frame_scale;
        }
        if self.nos_param < 0.0 {
            self.nos_param = 0.0;
        }
        audio.set_control_param(12, self.nos_param);
        for s in self.wind {
            if let Some(ps) = audio.slot_sample(s) {
                audio.set_sample_volume(ps, self.nos_param);
            }
        }
    }

    /// VsAudio.cpp 0x0004B850
    pub fn stop_nos(&mut self, audio: &mut AudioSystem) {
        if self.flags & flags::NOS == 0 {
            return;
        }
        self.flags &= !flags::NOS;
        audio.fade_to_volume_preset(0, f32::from_bits(0x3E4C_CCCD));
        for s in [self.nos_start, self.nos_loop, self.wind[0], self.wind[1]] {
            if let Some(e) = audio.slot(s) {
                audio.stop_event(Some(e));
            }
        }
        if let Some(e) = self.rear_emitter() {
            audio.play_event(e, crc("CAR_NOS_STOP"), None, 0, None);
        }
    }

    /// VsAudio.cpp 0x0004B950
    pub fn start_zone(&mut self, audio: &mut AudioSystem) {
        if !self.zone_on && !self.tb_on {
            self.zone_on = true;
            audio.fade_to_volume_preset(6, f32::from_bits(0x3E4C_CCCD));
            if let Some(e) = self.gamefx_emitter {
                audio.play_event(e, ZONE_HEARTBEAT, None, 0, Some(self.zone));
            }
        }
    }

    /// VsAudio.cpp 0x0004B9E0
    pub fn stop_zone(&mut self, audio: &mut AudioSystem) {
        if self.zone_on {
            self.zone_on = false;
            audio.fade_to_volume_preset(0, f32::from_bits(0x3E4C_CCCD));
            audio.stop_slot(self.zone);
        }
    }

    /// VsAudio.cpp 0x0004BA40
    pub fn start_slipstream(&mut self, audio: &mut AudioSystem) {
        if self.slipstream_on {
            return;
        }
        self.slipstream_on = true;
        audio.fade_to_volume_preset(5, f32::from_bits(0x3E4C_CCCD));
        self.slipstream_param = f32::from_bits(0x3DCC_CCCD);
        audio.set_control_param(12, self.slipstream_param);
        self.play_panned(audio, self.gamefx_emitter, self.slipstream[0], crc("MODE_SLIPSTREAM_LEFT"), -1.0);
        self.play_panned(audio, self.gamefx_emitter, self.slipstream[1], crc("MODE_SLIPSTREAM_RIGHT"), 1.0);
    }

    /// VsAudio.cpp 0x0004BBE0
    pub fn update_slipstream(&mut self, audio: &mut AudioSystem) {
        if self.slipstream_on && self.slipstream_param < f32::from_bits(0x3F4C_CCCD) {
            // 0.01 per reference frame
            self.slipstream_param += f32::from_bits(0x3C23_D70A) * self.frame_scale;
            audio.set_control_param(12, self.slipstream_param);
        }
    }

    /// VsAudio.cpp 0x0004BC50
    pub fn stop_slipstream(&mut self, audio: &mut AudioSystem) {
        if self.slipstream_on {
            audio.fade_to_volume_preset(0, f32::from_bits(0x3E4C_CCCD));
            self.slipstream_param = 0.0;
            audio.set_control_param(12, 0.0);
            self.slipstream_on = false;
            audio.stop_slot(self.slipstream[0]);
            audio.stop_slot(self.slipstream[1]);
        }
    }

    /// VsAudio.cpp 0x0004BD00
    pub fn start_mode3(&mut self, audio: &mut AudioSystem) {
        if !self.mode3_on {
            audio.fade_to_volume_preset(3, f32::from_bits(0x3E4C_CCCD));
            self.mode3_on = true;
            audio.stop_slot(self.mode3);
        }
    }

    /// VsAudio.cpp 0x0004BD80
    pub fn stop_mode3(&mut self, audio: &mut AudioSystem) {
        if self.mode3_on {
            self.mode3_on = false;
            audio.fade_to_volume_preset(0, f32::from_bits(0x3E4C_CCCD));
            if let Some(ps) = audio.slot_sample(self.mode3) {
                audio.set_sample_volume(ps, 0.0);
            }
            if audio.slot(self.mode3).is_some() {
                audio.stop_slot(self.mode3);
                if let Some(e) = self.sfx_emitter {
                    audio.stop_emitter_samples(e);
                }
            }
        }
    }

    /// VsAudio.cpp 0x0004BEA0
    pub fn start_tb(&mut self, audio: &mut AudioSystem) {
        if self.tb_on {
            return;
        }
        self.tb_on = true;
        self.flags |= flags::FLAG40;
        audio.apply_volume_preset(14);
        if let Some(e) = self.sfx_emitter {
            audio.play_event(e, crc("TB_ENTER"), None, 0, Some(self.tb_enter));
        }
        self.tb_fade = 0.0;
    }

    /// VsAudio.cpp 0x0004BF40
    pub fn update_tb(&mut self, audio: &mut AudioSystem) {
        if !self.tb_on {
            return;
        }
        if audio.slot(self.tb_atmos).is_some() {
            if self.tb_volume < 1.0 {
                self.tb_volume += self.tb_fade * self.frame_scale; // per reference frame
                if let Some(ps) = audio.slot_sample(self.tb_atmos) {
                    audio.set_sample_volume(ps, self.tb_volume);
                }
            }
        } else {
            self.stop_soundtrack(audio);
            if let Some(e) = self.sfx_emitter {
                audio.play_event(e, crc("TB_ATMOS"), None, 0, Some(self.tb_atmos));
            }
            if let Some(ps) = audio.slot_sample(self.tb_atmos) {
                audio.set_sample_volume(ps, 0.0);
            }
            self.tb_volume = 0.0;
            audio.fade_to_volume_preset(15, 1.0);
        }
    }

    /// VsAudio.cpp 0x0004C050
    pub fn set_tb_fade_in(&mut self, v: i32) {
        if v == 0 {
            self.tb_fade = f32::from_bits(0x3D08_882F); // 1/30 (the .data initial 0x684F08 is 1/120)
        }
    }

    /// VsAudio.cpp 0x0004C080
    pub fn stop_tb(&mut self, audio: &mut AudioSystem) {
        if self.tb_on {
            self.tb_on = false;
            if let Some(e) = audio.slot(self.tb_atmos) {
                audio.stop_event(Some(e));
            }
            audio.set_slot(self.tb_atmos, None);
            self.flags |= flags::MUSIC;
            self.next_track(audio);
            audio.fade_to_volume_preset(0, 1.0);
            self.end_flag40();
            self.tb_fade = 0.0;
        }
    }

    /// VsAudio.cpp 0x0004BE60
    fn end_flag40(&mut self) {
        if self.flags & flags::FLAG40 != 0 && !self.flag40_ended {
            self.flag40_ended = true;
        }
    }

    /// VsAudio.cpp 0x0004C110
    pub fn reset_modes(&mut self, audio: &mut AudioSystem) {
        audio.fade_to_volume_preset(0, 0.5);
        audio.set_environment(6, f32::from_bits(0x3C23_D70A));
        self.end_flag40();
    }

    // ---- race flow -----------------------------------------------------------------------------

    /// VsAudio.cpp 0x00052B40: the pre-race countdown.
    pub fn update_countdown(&mut self, audio: &mut AudioSystem, dt: f32, input: &GameAudioInput) {
        if !self.enabled() {
            return;
        }
        self.countdown_started = true;
        self.stop_soundtrack(audio);
        if self.flags & flags::PAUSED != 0 {
            self.resume(audio, input);
        }
        if !self.countdown_master_off {
            audio.set_master_volume(0.0);
            self.reset_modes(audio);
            self.countdown_master_off = true;
            self.start_master_fade = true;
        }
        self.countdown_time += dt;
        if self.countdown_time >= 3.0 && !self.countdown_girl {
            self.countdown_girl = true;
            if let Some(e) = self.speech_emitter {
                audio.play_event(e, START_GIRL_READY, None, 0, None);
            }
        }
    }

    /// VsAudio.cpp 0x00052C20
    pub fn race_started(&mut self, audio: &mut AudioSystem) {
        if !self.enabled() {
            return;
        }
        if self.countdown_started {
            self.countdown_time = 0.0;
            self.countdown_started = false;
            self.countdown_girl = false;
            self.countdown_master_off = false;
            self.start_master_fade = false;
            if let Some(e) = self.speech_emitter {
                audio.play_event(e, START_GIRL_GO, None, 0, None);
                audio.play_event(e, START_AIRHORN, None, 0, None);
            }
            self.flags |= flags::MUSIC;
            self.next_track(audio);
        } else if audio.master_volume == 0.0 {
            audio.set_master_volume(self.settings.master);
        }
    }

    /// VsAudio.cpp 0x00052600
    pub fn set_traffic_muted(&mut self, audio: &mut AudioSystem, muted: bool) {
        self.traffic_muted = muted;
        if muted {
            for va in &mut self.traffic_cars {
                va.release(audio);
            }
        }
    }

    /// VsAudio.cpp 0x00052DB0: wind speed parameter while the wind event runs.
    pub fn update_wind(&mut self, audio: &mut AudioSystem, input: &GameAudioInput) {
        if self.wind_keep || audio.slot(self.wind_event).is_none() {
            if audio.slot(self.wind_event).is_some() {
                let speed = input.player.as_ref().map(|p| p.speed).unwrap_or(0.0);
                audio.set_control_param(13, (f32::from_bits(0x400F_29D3) * speed / 100.0).clamp(0.0, 1.0));
            }
        } else {
            audio.stop_slot(self.wind_event);
        }
        self.wind_keep = false;
    }

    /// VsAudio.cpp 0x000524B0: player car, ai racers, traffic and police.
    pub fn update_vehicles(&mut self, audio: &mut AudioSystem, input: &GameAudioInput) {
        if self.start_master_fade {
            let m = self.settings.master;
            self.start_master_fade(audio, m, 0.5);
            self.start_master_fade = false;
        }
        let view = input.camera.as_ref().map(|c| c.view).unwrap_or(-1);
        let (engine, tyres, sfx) = (self.settings.engine, self.settings.tyres, self.settings.sfx);
        let track_wet = self.is_track_wet(input);
        let wet = track_wet && !self.under_cover;
        let tw = self.vehicle_tweaks.clone();
        if !self.engine_restart && self.events.engine_running {
            if let (Some(car), Some(va)) = (input.player.as_ref(), self.player_cars.first_mut()) {
                // VsAudio.cpp 0x000523F0 VsVehicleAudio::UpdateRpm
                va.frame_scale = self.frame_scale;
                self.events.rev_bands.frame_scale = self.frame_scale;
                va.rpm = (car.engine_rpm as f32 * 1.25).min(10000.0);
                self.events.set_engine_param(17, va.rpm as i32);
                self.events.set_engine_param(26, va.rpm as i32);
                let turbo = va.emitters[1];
                self.events.update_player_engine(audio, car, turbo);
                va.surface = car.surface;
                va.build_surface_events(audio, 0, wet);
                va.build_surface_events(audio, 1, wet);
                va.update_drift(audio, car, 3, 0, &tw);
                va.update_brakes(audio, car, 3, 0, tyres, &tw);
                va.update_drift(audio, car, 0, 1, &tw);
                va.update_brakes(audio, car, 0, 1, tyres, &tw);
                va.update_emitters(audio, car, view, engine, tyres, &tw);
                vehics::update_suspension(audio, &mut self.suspension, va.emitters[2], car, &tw, self.frame_scale);
            }
        } else {
            if let Some(e) = self.player_cars.first().and_then(|p| p.emitters[0]) {
                self.events.start_player_engine(e);
            }
            self.engine_restart = false;
        }
        let player_pos = input.player.as_ref().map(|p| p.position).unwrap_or(Vec3::ZERO);
        // VsAudio.cpp 0x00053290 UpdateAiCars (index 0 is the player). the counts are re-read from the race manager
        // every update: +12 players (+60, 0x00015C20; the port's input has one player car), +16 ai race vehicles
        // (+56, 0x000173B0), +20 the sum, +24 the traffic manager's count (0x00024B80, 0 without one).
        if input.player.is_some() {
            self.race_count = 1;
            self.tft_count = input.ai.len();
            self.total = self.race_count + self.tft_count;
            self.traffic_count = input.traffic.len();
        }
        // slots past Init's 1 + 6 would overrun the original's +36 array; the port stops there
        for i in 1..self.total.min(self.ai_cars.len()) {
            // i < players: a second human (only its emitters are updated); not reachable with one player car
            if let Some(ai) = i.checked_sub(self.race_count).and_then(|k| input.ai.get(k)) {
                let maxr = self.events.ai_max_rpm;
                vehics::update_ai_car(audio, &mut self.ai_cars[i], ai, maxr, sfx, &tw, view, engine, tyres);
            }
        }
        // VsAudio.cpp 0x00053660 UpdateTraffic
        for i in 0..self.traffic_count.min(self.traffic_cars.len()) {
            match input.traffic.get(i) {
                Some(t) => vehics::update_traffic_car(
                    audio,
                    &mut self.traffic_cars[i],
                    t,
                    player_pos,
                    track_wet,
                    self.traffic_muted,
                    sfx,
                    &tw,
                    view,
                    engine,
                    tyres,
                ),
                None => self.traffic_cars[i].release(audio),
            }
        }
        // VsAudio.cpp 0x00054060 UpdatePolice
        debug_assert!(input.police.len() <= self.police_count, "numPolice <= m_nPoliceVehicles");
        for i in 0..input.police.len().min(self.police_cars.len()) {
            vehics::update_police_car(
                audio,
                &mut self.police_cars[i],
                &input.police[i],
                player_pos,
                track_wet,
                self.traffic_muted,
                sfx,
                &tw,
                view,
                engine,
                tyres,
            );
        }
    }

    /// VsAudio.cpp 0x00052E90 PlayPhysicsImpact
    pub fn play_physics_impact(&mut self, audio: &mut AudioSystem, input: &GameAudioInput, hash: u32, pos: Vec3, strength: f32, car_layer_only: bool) {
        let pp = input.player.as_ref().map(|p| p.position).unwrap_or(Vec3::ZERO);
        vehics::play_physics_impact(audio, &mut self.impacts, &self.impact_emitters, pp, hash, pos, strength, car_layer_only, self.settings.sfx);
    }

    /// CarDamageFX.cpp 0x00078320 (the audio half): glass particles fired for a light/window of the car at `pos`.
    pub fn play_glass_impact(&mut self, audio: &mut AudioSystem, input: &GameAudioInput, pos: Vec3) {
        let strength = (audio.rand() % 700) as f32 + 100.0;
        self.play_physics_impact(audio, input, GLASS_IMPACT, pos, strength, false);
    }

    // ---- music ---------------------------------------------------------------------------------

    /// VsAudio.cpp 0x0004DB90
    pub fn init_music(&mut self, audio: &mut AudioSystem) {
        if let Some(t) = VsAudioTrackList::load() {
            self.tracks = t;
        }
        if self.soundtrack.source == MusicSource::Bank {
            // the port's song source: the music banks carry the same tracks under the MusLists names
            for g in [r"\Audio\Music\fe\music_fe", r"\Audio\Music\ingame\music"] {
                if let Err(e) = audio.load_group(g, None) {
                    log::warn!("audio: music bank {g}: {e:#x}");
                }
            }
        }
        if self.saved_lists.lists[0][0] != 0 {
            let s = self.saved_lists.clone();
            self.apply_lists(audio, &s);
        }
        self.pick_list(audio);
    }

    /// VsAudioTrackList.cpp 0x0004F1A0 PickList with the soundtrack lists' real types.
    fn pick_list(&mut self, audio: &mut AudioSystem) {
        let n = self.soundtrack.soundtracks.len().max(1);
        let mode = if self.frontend { None } else { Some(self.race_mode.unwrap_or(0)) };
        let sts = &self.soundtrack.soundtracks;
        let mut rng = || audio.music_rand();
        self.tracks.pick_list(n, |j| sts.get(j).map_or(6, |s| s.kind), mode, &mut rng);
    }

    /// SaveFileData 0x00104F30: the loaded save's playlists (also kept for every later InitMusic).
    pub fn set_lists(&mut self, audio: &mut AudioSystem, s: &SavedPlaylists) {
        // port: a save whose playlist block was never written (all zero: first list unnamed, as `init_music` also
        // treats it) keeps the MusLists defaults instead of wiping every list (empty DJ booth, no music)
        if s.lists[0][0] == 0 {
            return;
        }
        self.saved_lists = s.clone();
        self.apply_lists(audio, s);
    }

    /// VsAudioTrackList.cpp 0x0004E2A0 SetLists
    fn apply_lists(&mut self, audio: &mut AudioSystem, s: &SavedPlaylists) {
        self.tracks.set_lists(s);
        let n = self.tracks.list_count(self.soundtrack.soundtracks.len().max(1));
        for (k, i) in (FIXED_LISTS..n).enumerate() {
            // VsAudioTrackList.cpp 0x0004FAD0 SetListType / SetListShuffle: soundtrack i - 2
            if let (Some(st), Some(x)) = (self.soundtrack.soundtracks.get_mut(i - (FIXED_LISTS - 1)), s.extra.get(k)) {
                st.kind = x[0];
                st.shuffle = x[1];
            }
        }
        if self.soundtrack.state == PlayerState::Playing {
            self.stop_soundtrack(audio);
            self.pick_list(audio);
            self.next_track(audio);
        }
    }

    /// SaveFileData 0x00105E10: the playlists for the save (also updates the kept copy, as the original
    /// writes them into the global block).
    pub fn get_lists(&mut self) -> SavedPlaylists {
        let sts = &self.soundtrack.soundtracks;
        let s = self.tracks.get_lists(sts.len().max(1), |j| sts.get(j).map_or((6, 0), |s| (s.kind, s.shuffle)));
        self.saved_lists = s.clone();
        s
    }

    /// VsAudio.cpp 0x0004DA30
    pub fn stop_soundtrack(&mut self, audio: &mut AudioSystem) {
        if matches!(self.soundtrack.state, PlayerState::Paused | PlayerState::Playing) {
            self.soundtrack.stop(audio);
        }
    }

    /// VsAudio.cpp 0x0004D950
    pub fn stop_music_event(&mut self, audio: &mut AudioSystem) {
        if self.enabled() && self.flags & flags::SOUNDTRACK != 0 {
            audio.stop_slot(self.music_event);
        }
    }

    /// VsAudio.cpp 0x0004DA80: crossfade the old music event and pick a reverb.
    pub fn fade_music(&mut self, audio: &mut AudioSystem, to: f32, env: f32, time: f32) {
        if let Some(ps) = audio.slot_sample(self.music_event) {
            let v = audio.samples.get(ps).unwrap().volume;
            audio.fade_sample_volume(ps, v, to, time);
        }
        if env == f32::from_bits(0x3DCC_CCCD) {
            audio.set_environment(6, f32::from_bits(0x3C23_D70A));
        } else {
            audio.set_environment(3, env);
        }
    }

    /// VsAudio.cpp 0x0004F4B0
    pub fn next_track(&mut self, audio: &mut AudioSystem) {
        if self.flags & flags::SOUNDTRACK != 0
            && self.flags & flags::MUSIC != 0
            && self.settings.music != 0.0
            && self.settings.master != 0.0
        {
            self.play_track(audio, false, None);
        }
    }

    /// VsAudio.cpp 0x0004F540. `force` plays (soundtrack, song) right away (the playlist screen).
    pub fn play_track(&mut self, audio: &mut AudioSystem, force: bool, pick: Option<(usize, usize)>) {
        if force {
            if self.soundtrack.state != PlayerState::Stopped {
                self.soundtrack.stop(audio);
            }
            let (st, song) = pick.unwrap_or((0, 0));
            self.soundtrack.set_soundtrack(st);
            self.soundtrack.set_song(song);
            self.soundtrack.play(audio, &self.tracks);
            let (m, master) = (self.settings.music, self.settings.master);
            self.soundtrack.set_volume(audio, m, master);
            return;
        }
        let list = self.tracks.current_list as usize;
        let fixed = list <= FIXED_LISTS;
        if fixed && self.tracks.lists.get(list).is_none_or(|l| l.tracks.is_empty()) {
            return;
        }
        if self.soundtrack.state != PlayerState::Stopped || self.settings.music <= 0.0 || self.settings.master <= 0.0 {
            return;
        }
        if fixed {
            let l = &self.tracks.lists[list];
            let n = l.tracks.len() as i32;
            if l.shuffle == 1 {
                let prev = self.tracks.current_track;
                self.tracks.current_track = (audio.music_rand() as u32 % n as u32) as i32;
                // port: per-launch random mode never repeats the previous track
                if audio.music_random && n > 1 && (0..n).contains(&prev) {
                    let r = audio.music_rand() % (n - 1);
                    self.tracks.current_track = if r >= prev { r + 1 } else { r };
                }
            } else if audio.music_random && self.tracks.current_track < 0 {
                // port: sequential list starts at a random track on the first pick
                self.tracks.current_track = audio.music_rand() % n;
            } else {
                self.tracks.current_track += 1;
                if self.tracks.current_track == n {
                    self.tracks.current_track = 0;
                }
            }
            let e = &self.tracks.lists[list].tracks[self.tracks.current_track as usize];
            let (song, st) = (e.master as usize, e.soundtrack as usize);
            self.soundtrack.set_soundtrack(st);
            self.soundtrack.set_song(song);
        } else {
            let st = list - (FIXED_LISTS - 1);
            let songs = self.soundtrack.soundtracks.get(st).map(|s| s.songs.len()).unwrap_or(0) as i32;
            let sts = &self.soundtrack.soundtracks;
            if self.tracks.list_shuffle(list, |j| sts.get(j).map_or(0, |s| s.shuffle)) == 1 && songs > 0 {
                let prev = self.tracks.current_track;
                self.tracks.current_track = audio.music_rand() % songs;
                if audio.music_random && songs > 1 && (0..songs).contains(&prev) {
                    let r = audio.music_rand() % (songs - 1);
                    self.tracks.current_track = if r >= prev { r + 1 } else { r };
                }
            } else if audio.music_random && self.tracks.current_track < 0 && songs > 0 {
                self.tracks.current_track = audio.music_rand() % songs;
            } else {
                self.tracks.current_track += 1;
            }
            self.soundtrack.set_soundtrack(st);
            self.soundtrack.set_song(self.tracks.current_track as usize);
        }
        self.soundtrack.play(audio, &self.tracks);
        let (m, master) = (self.settings.music, self.settings.master);
        self.soundtrack.set_volume(audio, m, master);
    }

    /// VsAudio.cpp 0x0004F8E0
    pub fn update_music(&mut self, audio: &mut AudioSystem) {
        if self.flags & flags::SOUNDTRACK != 0 && self.flags & flags::MUSIC != 0 {
            if self.soundtrack.state == PlayerState::Stopped && self.settings.music > 0.0 && self.settings.master > 0.0 {
                self.next_track(audio);
            }
            let (m, master) = (self.settings.music, self.settings.master);
            self.soundtrack.set_volume(audio, m, master);
        }
    }

    /// VsAudio.cpp 0x0004AE60
    pub fn pause_music(&mut self, audio: &mut AudioSystem) {
        if self.enabled() {
            if let Some(ps) = self.player_emitter.and_then(|e| audio.emitters.get(e)).and_then(|e| e.samples) {
                audio.pause_sample(ps);
            }
            if self.flags & flags::SOUNDTRACK != 0 {
                self.soundtrack.pause(audio);
            }
        }
    }

    /// VsAudio.cpp 0x0004AEC0
    pub fn resume_music(&mut self, audio: &mut AudioSystem) {
        if self.enabled() {
            if let Some(ps) = self.player_emitter.and_then(|e| audio.emitters.get(e)).and_then(|e| e.samples) {
                audio.resume_sample(ps);
            }
            if self.flags & flags::SOUNDTRACK != 0 && self.soundtrack.state == PlayerState::Paused {
                self.soundtrack.play(audio, &self.tracks);
            }
        }
    }
}

fn atof(s: &str) -> f32 {
    let t: String = s.trim().chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect();
    t.parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_aee_parses() {
        let tracks = visos::file::resolve(r"\Tracks");
        let mut n = 0;
        for e in std::fs::read_dir(tracks).unwrap().flatten() {
            let p = e.path().join("envaudio.aee");
            let Ok(b) = std::fs::read(&p) else { continue };
            let recs = parse_aee(&b).unwrap();
            assert_eq!(b.len(), 12 + 76 * recs.len());
            // LACoast AudioEmitter_5 ships with max distance 0 (a data quirk the game passes on as is)
            assert!(recs.iter().filter(|r| r.max_distance <= 0.0).count() <= 1, "{}", p.display());
            assert!(recs.iter().all(|r| r.event != 0 && r.min_distance > 0.0));
            n += 1;
        }
        assert_eq!(n, 4); // LACoast, LADown, London, London2
    }

    #[test]
    fn gui_hashes_are_the_fe_events() {
        for (s, n) in [(GuiSound::Up, "GUI_CURSOR_UPLEFT"), (GuiSound::Down, "GUI_CURSOR_DOWNRIGHT"), (GuiSound::Accept, "GUI_ACCEPT"),
                       (GuiSound::Back, "GUI_BACK"), (GuiSound::Error, "GUI_ERROR"), (GuiSound::WooshIn, "GUI_WOOSHIN"),
                       (GuiSound::WooshOut, "GUI_WOOSHOUT"), (GuiSound::WooshLeft, "GUI_WOOSHLEFT"), (GuiSound::WooshRight, "GUI_WOOSHRIGHT")] {
            assert_eq!(s.event(), crc(n), "{n}");
        }
        for (h, n) in [(HWE_HOP_ALARM, "HWE_HOP_ALARM"), (RAIN_LEFT, "AMB_RAIN_LIGHT_LEFT"), (RAIN_RIGHT, "AMB_RAIN_LIGHT_RIGHT"),
                       (START_GIRL_READY, "START_FLG_GIRL_READY"), (START_GIRL_GO, "START_FLG_GIRL_GO"),
                       (START_AIRHORN, "START_AIRHORN"), (ZONE_HEARTBEAT, "MODE_ZONE_HEARTBEAT")] {
            assert_eq!(h, crc(n), "{n}");
        }
    }

    /// port: an unwritten (all zero) playlist block of a save keeps the default lists; a written one replaces them.
    #[test]
    fn zero_saved_lists_keep_defaults() {
        let (mut audio, _mixer) = AudioSystem::with_default_mixer();
        let mut vs = VsAudio::new(&mut audio);
        vs.init(&mut audio, &VsAudioInitInfo { frontend: true, ..Default::default() });
        let before = vs.tracks.clone();
        assert!(!before.lists[0].tracks.is_empty() && before.lists[0].name != 0);
        vs.set_lists(&mut audio, &SavedPlaylists::default());
        assert_eq!(vs.tracks.lists, before.lists);
        let mut s = vs.get_lists();
        s.lists[0][1] = 0; // a written block with an emptied first list is the user's choice
        vs.set_lists(&mut audio, &s);
        assert!(vs.tracks.lists[0].tracks.is_empty());
    }

    /// a front-end session: groups, gui sounds and the front-end playlist from the music banks.
    #[test]
    fn front_end_session() {
        let (mut audio, mixer) = AudioSystem::with_default_mixer();
        let mut vs = VsAudio::new(&mut audio);
        assert_eq!(vs.init(&mut audio, &VsAudioInitInfo { frontend: true, ..Default::default() }), 0);
        vs.load_front_end_groups(&mut audio);
        assert_eq!(vs.play_gui(&mut audio, GuiSound::Accept), 0);
        let input = GameAudioInput::default();
        let mut buf = vec![0.0f32; 1600];
        let mut energy = 0.0;
        for _ in 0..120 {
            vs.update(&mut audio, 1.0 / 60.0, &input);
            mixer.lock().unwrap().render(&mut buf);
            energy += buf.iter().map(|v| v * v).sum::<f32>();
        }
        assert!(energy > 0.0);
        if vs.soundtrack.source == MusicSource::Bank {
            assert_eq!(vs.soundtrack.state, PlayerState::Playing, "front-end music starts");
            let ev = vs.soundtrack.event.and_then(|s| audio.slot(s));
            assert!(ev.is_some(), "music event running");
        }
        vs.shutdown(&mut audio);
    }

    /// the front-end mix level does not depend on the update rate (mix rendered in real time between updates).
    #[test]
    fn front_end_mix_at_any_rate() {
        let mut db = Vec::new();
        for hz in [30.0f64, 60.0, 240.0, 1000.0] {
            let (mut audio, mixer) = AudioSystem::with_default_mixer();
            let mut vs = VsAudio::new(&mut audio);
            vs.init(&mut audio, &VsAudioInitInfo { frontend: true, ..Default::default() });
            vs.load_front_end_groups(&mut audio);
            let input = GameAudioInput::default();
            let (mut done, mut ss, mut n) = (0u64, 0.0f64, 0u64);
            let mut buf = Vec::new();
            let mut per_sec = Vec::new();
            for f in 1..=(hz * 4.0) as u64 {
                vs.update(&mut audio, (1.0 / hz) as f32, &input);
                let due = (f as f64 / hz * 48000.0) as u64;
                buf.resize(((due - done) * 2) as usize, 0.0);
                mixer.lock().unwrap().render(&mut buf);
                done = due;
                ss += buf.iter().map(|&v| (v as f64) * (v as f64)).sum::<f64>();
                n += buf.len() as u64;
                if f % hz as u64 == 0 {
                    per_sec.push(10.0 * (ss / n.max(1) as f64).max(1e-30).log10());
                    ss = 0.0;
                    n = 0;
                }
            }
            eprintln!("fe mix {hz} Hz: {per_sec:.1?}");
            db.push(per_sec);
            vs.shutdown(&mut audio);
        }
        for d in &db[1..] {
            for (a, b) in d.iter().zip(&db[0]) {
                assert!((a - b).abs() < 3.0, "{db:?}");
            }
        }
    }

    /// front end -> shutdown -> race (LADown, the smoke run's race): level sounds and the wrx rev bands load,
    /// the engine plays and its pitch follows rpm the same at 60 and 144 Hz.
    #[test]
    fn race_session_engine_at_any_rate() {
        let mut pitch = Vec::new();
        for hz in [60.0f32, 144.0] {
            let (mut audio, _mixer) = AudioSystem::with_default_mixer();
            let mut vs = VsAudio::new(&mut audio);
            vs.init(&mut audio, &VsAudioInitInfo { frontend: true, ..Default::default() });
            vs.shutdown(&mut audio);
            let info = VsAudioInitInfo { frontend: false, track_dir: Some(r"Tracks\LADOWN\".into()), race_mode: None };
            assert_eq!(vs.init(&mut audio, &info), 0, "race init after the front end's shutdown");
            vs.load_level_sounds(&mut audio, r"Tracks\LADOWN\", false);
            for h in ["IMPACT_CAR_PHYSICS", "START_FLG_GIRL_READY", "RN_TARMAC_PASSBY_DRY_LIGHT", "TB_ENTER", "ASP_CARPARK_EXPLOSION_INSIDE"] {
                assert!(audio.find_event(crc(h)) >= 0, "{h}");
            }
            // TB_ATMOS is in no shipped group (UpdateTb retries it every frame, as the original)
            assert!(audio.find_event(crc("TB_ATMOS")) < 0);
            assert!(vs.events.rev_bands.load(&mut audio, r"tier4\wrx\vimprrb0"));
            vs.race_started(&mut audio);
            let mut input = GameAudioInput::default();
            let mut car = CarAudioInput { engine_rpm: 1000, gear: 1, active: true, ..Default::default() };
            car.forward = Vec3::Z;
            input.player = Some(car);
            input.camera = Some(crate::input::CameraAudioInput {
                position: Vec3::new(0.0, 1.5, -5.0),
                forward: Vec3::Z,
                up: Vec3::Y,
                follows_player: true,
                view: 0,
            });
            let frames = (hz * 2.0) as usize;
            for f in 0..frames {
                // rpm ramps 1000 -> 6000 over the first second
                let t = (f as f32 / hz).min(1.0);
                input.player.as_mut().unwrap().engine_rpm = (1000.0 + 5000.0 * t) as i32;
                vs.update(&mut audio, 1.0 / hz, &input);
            }
            assert!(vs.events.engine_running);
            assert!(vs.events.rev_bands.is_active());
            pitch.push(vs.events.rev_bands.smooth);
        }
        assert!(pitch[0] > 0.5, "{pitch:?}");
        assert!((pitch[0] - pitch[1]).abs() < 0.02, "{pitch:?}");
    }

    /// LADown trailblazer: StartTb plays TB_ENTER (mdtrail, LADown's level block), AudioFadeIn sets the 1/30 step
    /// (0x0004C050; the .data default is 1/120), TB_ATMOS is not shipped so UpdateTb keeps retrying it, StopTb.
    #[test]
    fn trailblazer_enter_and_fade_step() {
        let (mut audio, _mixer) = AudioSystem::with_default_mixer();
        let mut vs = VsAudio::new(&mut audio);
        assert_eq!(vs.tb_fade.to_bits(), 0x3C08_8723);
        let info = VsAudioInitInfo { frontend: false, track_dir: Some(r"Tracks\LADOWN\".into()), race_mode: None };
        assert_eq!(vs.init(&mut audio, &info), 0);
        vs.load_level_sounds(&mut audio, r"Tracks\LADOWN\", false);
        vs.race_started(&mut audio);
        let input = GameAudioInput::default();
        vs.start_tb(&mut audio);
        assert!(audio.slot(vs.tb_enter).is_some(), "TB_ENTER playing");
        assert_eq!(vs.tb_fade, 0.0);
        vs.set_tb_fade_in(0);
        assert_eq!(vs.tb_fade, f32::from_bits(0x3D08_882F));
        vs.update(&mut audio, 1.0 / 60.0, &input);
        assert!(audio.slot(vs.tb_atmos).is_none());
        vs.stop_tb(&mut audio);
        assert!(!vs.tb_on);
    }
}
