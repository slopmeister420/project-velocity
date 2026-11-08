//! the player options the frontend options screens and the in-race menus edit: the `GameVars` globals
//! `dword_6F9160..byte_6F9182` (defaults from GamePlayer.cpp `SetInitialVariables` 0x000C1930), the controller
//! mapping index per pad (`UserInput +0x14`) and the string manager language.
//!
//! the original keeps them in the profile on the memory unit; here they live in `options.ini` under the user
//! profile (`%APPDATA%\Velocity`), never in the game folder.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

/// `byte_6F915D` as read by `CElement2D::Render` (0x0029273D): 16:9 output.
static WIDESCREEN: AtomicBool = AtomicBool::new(false);

/// the 2D renderer's widescreen switch.
pub fn widescreen() -> bool {
    WIDESCREEN.load(Ordering::Relaxed)
}

/// display toggles (`display[]`)
pub mod display {
    pub const RATIO_WIDE: usize = 0; // byte_6F915D
    pub const TFT: usize = 1; // byte_6F915E
    pub const NAMES: usize = 2; // byte_6F915F
    pub const JUMP_CAM: usize = 3; // byte_75B278
    pub const CRASH_CAM: usize = 4; // byte_6D58A8
    pub const POLICE_CAM: usize = 5; // byte_6D60B9
}

#[derive(Clone, Debug, PartialEq, bevy::prelude::Resource)]
pub struct GameOptions {
    /// master 6F9164, music 6F9168, engine 6F916C, sfx 6F9178, speech 6F9174, tyres 6F9170 (pause menu order)
    pub volumes: [f32; 6],
    /// dword_6F917C
    pub audio_mode: i32,
    /// byte_6F9182
    pub rumble: bool,
    pub display: [bool; 6],
    /// dword_6F9160: soundtrack list picked in the audio options, -1 = random
    pub soundtrack: i32,
    /// UserInput +0x14: mapping index per pad
    pub controller_config: [u8; 4],
    /// CStringManager language folder (`English`, `US`, ...)
    pub language: String,
    /// port: the PC settings page (`pc_settings.rs`), not in the original
    pub pc: PcSettings,
}

/// port (not original): window / frame pacing / music order, edited on `PcOptionsScreen`, applied by
/// `pc_settings::apply_pc_settings`. Command-line `--fps` and `VELOCITY_WINDOW` override them.
#[derive(Clone, Debug, PartialEq)]
pub struct PcSettings {
    /// 0 windowed, 1 borderless (desktop resolution), 2 fullscreen (`resolution`)
    pub display_mode: u8,
    /// fullscreen resolution (physical px); None = the monitor's current mode
    pub resolution: Option<(u32, u32)>,
    /// render rate cap in fps, 0 = uncapped
    pub fps_cap: u32,
    pub vsync: bool,
    /// audio's per-launch music randomiser (false = the original's pick order)
    pub music_random: bool,
}

impl Default for PcSettings {
    fn default() -> Self {
        Self { display_mode: 0, resolution: None, fps_cap: 0, vsync: true, music_random: true }
    }
}

pub mod display_mode {
    pub const WINDOWED: u8 = 0;
    pub const BORDERLESS: u8 = 1;
    pub const FULLSCREEN: u8 = 2;
}

/// the frame rate cap choices (0 = uncapped)
pub const FPS_CAPS: [u32; 6] = [30, 60, 120, 144, 240, 0];

impl Default for GameOptions {
    /// GamePlayer.cpp 0x000C1930 (speech is written 0.2 then 0.8)
    fn default() -> Self {
        Self {
            volumes: [1.0, 0.2, 0.8, 0.8, 0.8, 0.8],
            audio_mode: 1,
            rumble: true,
            display: [false, true, false, false, false, false],
            soundtrack: -1,
            controller_config: [0; 4],
            language: "English".into(),
            pc: PcSettings::default(),
        }
    }
}

/// `%APPDATA%\Velocity` (falls back to the home folder, then the exe folder).
pub fn profile_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("APPDATA") {
        return PathBuf::from(d).join("Velocity");
    }
    if let Some(d) = std::env::var_os("HOME") {
        return PathBuf::from(d).join(".velocity");
    }
    std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.to_path_buf())).unwrap_or_default()
}

pub fn options_path() -> PathBuf {
    profile_dir().join("options.ini")
}

impl GameOptions {
    /// the saved options, defaults for anything missing.
    pub fn load() -> Self {
        let o = std::fs::read_to_string(options_path()).map(|t| Self::parse(&t)).unwrap_or_default();
        o.apply_globals();
        o
    }

    pub fn parse(text: &str) -> Self {
        let mut o = Self::default();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let (k, v) = (k.trim(), v.trim());
            let f = |d: f32| v.parse::<f32>().map(|x| x.clamp(0.0, 1.0)).unwrap_or(d);
            let b = |d: bool| v.parse::<i32>().map(|x| x != 0).unwrap_or(d);
            match k {
                "MasterVolume" => o.volumes[0] = f(o.volumes[0]),
                "MusicVolume" => o.volumes[1] = f(o.volumes[1]),
                "EngineVolume" => o.volumes[2] = f(o.volumes[2]),
                "SfxVolume" => o.volumes[3] = f(o.volumes[3]),
                "SpeechVolume" => o.volumes[4] = f(o.volumes[4]),
                "TyresVolume" => o.volumes[5] = f(o.volumes[5]),
                "AudioMode" => o.audio_mode = v.parse::<i32>().map(|x| x.clamp(0, 2)).unwrap_or(o.audio_mode),
                "Rumble" => o.rumble = b(o.rumble),
                // read but unused: the ratio row is gone, the UI adapts to the window aspect (docs/frontend.md)
                "Widescreen" => {}
                "DisplayMode" => o.pc.display_mode = v.parse::<u8>().map(|x| x.min(display_mode::FULLSCREEN)).unwrap_or(0),
                "Resolution" => {
                    o.pc.resolution = v
                        .split_once('x')
                        .and_then(|(w, h)| Some((w.trim().parse::<u32>().ok()?, h.trim().parse::<u32>().ok()?)))
                        .filter(|&(w, h)| w > 0 && h > 0)
                }
                "FpsCap" => o.pc.fps_cap = v.parse().unwrap_or(0),
                "VSync" => o.pc.vsync = b(true),
                "MusicOrder" => o.pc.music_random = !v.eq_ignore_ascii_case("original"),
                "TftDisplay" => o.display[display::TFT] = b(true),
                "PlayerNames" => o.display[display::NAMES] = b(false),
                "JumpCam" => o.display[display::JUMP_CAM] = b(false),
                "CrashCam" => o.display[display::CRASH_CAM] = b(false),
                "PoliceCam" => o.display[display::POLICE_CAM] = b(false),
                "Soundtrack" => o.soundtrack = v.parse().unwrap_or(-1),
                "Language" if !v.is_empty() => o.language = v.to_string(),
                _ => {
                    if let Some(i) = k.strip_prefix("ControllerConfig").and_then(|i| i.parse::<usize>().ok()) {
                        if i < 4 {
                            o.controller_config[i] = v.parse().unwrap_or(0);
                        }
                    }
                }
            }
        }
        o
    }

    pub fn to_text(&self) -> String {
        let mut s = String::from("; Project Velocity options\n");
        let names = ["MasterVolume", "MusicVolume", "EngineVolume", "SfxVolume", "SpeechVolume", "TyresVolume"];
        for (n, v) in names.iter().zip(self.volumes) {
            s += &format!("{n}={v}\n");
        }
        s += &format!("AudioMode={}\nRumble={}\n", self.audio_mode, self.rumble as i32);
        let d = ["Widescreen", "TftDisplay", "PlayerNames", "JumpCam", "CrashCam", "PoliceCam"];
        for (n, v) in d.iter().zip(self.display) {
            s += &format!("{n}={}\n", v as i32);
        }
        s += &format!("Soundtrack={}\n", self.soundtrack);
        for (i, c) in self.controller_config.iter().enumerate() {
            s += &format!("ControllerConfig{i}={c}\n");
        }
        s += &format!("Language={}\n", self.language);
        let p = &self.pc;
        let res = p.resolution.map_or(String::new(), |(w, h)| format!("{w}x{h}"));
        s += &format!("DisplayMode={}\nResolution={res}\nFpsCap={}\nVSync={}\n", p.display_mode, p.fps_cap, p.vsync as i32);
        s += &format!("MusicOrder={}\n", if p.music_random { "Random" } else { "Original" });
        s
    }

    /// write `options.ini`; failures are logged, never fatal.
    pub fn save(&self) {
        let p = options_path();
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        if let Err(e) = std::fs::write(&p, self.to_text()) {
            log::warn!("options: cannot write {}: {e}", p.display());
        }
    }

    /// the globals other code reads directly.
    pub fn apply_globals(&self) {
        // the 4:3 / 16:9 option is retired (no row, key ignored): always the 4:3 path, widening is automatic
        WIDESCREEN.store(false, Ordering::Relaxed);
    }

    /// `VsAudio::Update` reads the volume globals once its settings are dirty (`byte_6D45C1`).
    pub fn audio_settings(&self) -> velocity_audio::input::AudioSettings {
        velocity_audio::input::AudioSettings {
            master: self.volumes[0],
            music: self.volumes[1],
            engine: self.volumes[2],
            tyres: self.volumes[5],
            speech: self.volumes[4],
            sfx: self.volumes[3],
            output_mode: self.audio_mode as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut o = GameOptions::default();
        o.volumes[1] = 0.55;
        o.display[display::TFT] = false;
        o.soundtrack = 2;
        o.controller_config[1] = 3;
        o.language = "French".into();
        assert_eq!(GameOptions::parse(&o.to_text()), o);
        assert_eq!(GameOptions::parse(""), GameOptions::default());
    }

    #[test]
    fn pc_settings_round_trip() {
        let mut o = GameOptions::default();
        assert_eq!(o.pc, PcSettings { display_mode: 0, resolution: None, fps_cap: 0, vsync: true, music_random: true });
        assert_eq!(GameOptions::parse(&o.to_text()), o);
        o.pc = PcSettings { display_mode: display_mode::FULLSCREEN, resolution: Some((1920, 1080)), fps_cap: 144, vsync: false, music_random: false };
        let text = o.to_text();
        for k in ["DisplayMode=2", "Resolution=1920x1080", "FpsCap=144", "VSync=0", "MusicOrder=Original"] {
            assert!(text.lines().any(|l| l == k), "{k} missing in\n{text}");
        }
        assert_eq!(GameOptions::parse(&text), o);
        // the retired ratio key is still accepted but changes nothing
        let p = GameOptions::parse("Widescreen=1\nDisplayMode=9\nResolution=bad\nMusicOrder=Random\n");
        assert_eq!(p.display[display::RATIO_WIDE], false);
        assert_eq!((p.pc.display_mode, p.pc.resolution, p.pc.music_random), (display_mode::FULLSCREEN, None, true));
    }
}
