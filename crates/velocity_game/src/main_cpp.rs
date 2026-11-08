//! src\main.cpp (as `main_cpp`: a src/main.rs would become a cargo bin) — command line handling before
//! `VsCommonMain` (0x0016D120). The xbox build gets its arguments from the launch data; here they come from the
//! process arguments (`--game` and other `--` options are skipped, with their value).

use bevy::prelude::Resource;

/// options set by `main()` (globals in the original).
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct CommandLine {
    /// `attractmode <a> <b>`
    pub attract_mode: Option<(String, String)>,
    /// `profiler` (0) / `profiler_loop` (1)
    pub profiler: Option<bool>,
    /// `jumptogame [TRACK EVENT]` / `profiler*`: GameManager+0x35, enables every race (0x000BE7E0)
    pub jump_mode: bool,
    /// set when a jump target was given or defaulted (`GameVars` GameJump active)
    pub game_jump: Option<(String, String)>,
    /// `skipInitialScreens`, also implied by jumps (byte_6F905B)
    pub skip_initial_screens: bool,
    /// `jumptoscreen <crc of name>`
    pub jump_to_screen: Option<u32>,
    /// jumptoscreen CarCustomisationScreen (byte_6F905A)
    pub jump_to_car_customisation: bool,
    /// `setCar <name>` crc
    pub set_car: Option<u32>,
    pub splitscreen: bool,
    /// `professional` / `underground`
    pub professional: Option<bool>,
    /// `wet` / `dry`
    pub wet: Option<bool>,
    /// `hud <type>` crc (dword_6FD1F8)
    pub hud: Option<u32>,
    /// `setflaggirl <name>`
    pub flag_girl: Option<String>,
    pub no_vibrate: bool,
    pub no_audio: bool,
    pub no_music: bool,
    pub no_barriers: bool,
    /// port option `-u` / `--unlock` (not in the original): boot with the debug build's unlock tweaks on,
    /// see `cli::debug_unlocks`
    pub unlock: bool,
}

fn crc(s: &str) -> u32 {
    if s.is_empty() { 0 } else { visos::handy::handy_crc::crc32(s.as_bytes(), 0) }
}

impl CommandLine {
    pub fn from_env() -> Self {
        let mut v = Vec::new();
        let mut unlock = false;
        let mut it = std::env::args().skip(1);
        while let Some(a) = it.next() {
            if a == "-u" || a == "--unlock" {
                unlock = true;
                continue;
            }
            // the scripted smoke test picks a shipped-unavailable cruise, so it runs unlocked; takes no value
            if a == "--slice-test" || a == "--smoke-test" {
                unlock = true;
                continue;
            }
            if a.starts_with("--") {
                if !a.contains('=') {
                    it.next();
                }
                continue;
            }
            v.push(a);
        }
        Self { unlock, ..Self::parse(&v) }
    }

    /// main.cpp 0x0016D120 — every argument is tested against every keyword (parameters included).
    pub fn parse(args: &[String]) -> Self {
        let mut c = CommandLine::default();
        let eq = |a: &str, k: &str| a.eq_ignore_ascii_case(k);
        for (i, a) in args.iter().enumerate() {
            let rest = args.len() - (i + 1);
            if eq(a, "attractmode") && rest >= 2 {
                c.attract_mode = Some((args[i + 1].clone(), args[i + 2].clone()));
            }
            if eq(a, "profiler") || eq(a, "profiler_loop") || eq(a, "jumptogame") {
                if eq(a, "profiler") {
                    c.profiler = Some(false);
                } else if eq(a, "profiler_loop") {
                    c.profiler = Some(true);
                }
                c.skip_initial_screens = true;
                c.jump_mode = true;
                if rest >= 2 {
                    c.game_jump = Some((args[i + 1].clone(), args[i + 2].clone()));
                } else if c.profiler.is_some() {
                    c.game_jump = Some(("TRACKS\\LACOAST\\".into(), "CIRCUIT\\RACE01\\".into()));
                } else {
                    c.game_jump = None;
                    log::error!("jumptogame\tcommand parameter badly formed");
                    log::error!("USAGE:\njumptogame <TRACK> <EVENT>\nEXAMPLE: jumptogame TRACKS\\LACOAST\\ CIRCUIT\\RACE01\\");
                }
            } else if eq(a, "skipInitialScreens") {
                c.skip_initial_screens = true;
            } else if eq(a, "jumptoscreen") {
                let name = args.get(i + 1).cloned().unwrap_or_default();
                c.jump_to_screen = Some(crc(&name));
                if crc(&name) == crc("CarCustomisationScreen") {
                    c.jump_to_car_customisation = true;
                }
                c.skip_initial_screens = true;
            } else if eq(a, "setCar") {
                c.set_car = Some(crc(args.get(i + 1).map(String::as_str).unwrap_or("")));
            } else if eq(a, "splitscreen") {
                c.splitscreen = true;
            } else if eq(a, "underground") {
                c.professional = Some(false);
            } else if eq(a, "professional") {
                c.professional = Some(true);
            } else if eq(a, "wet") {
                c.wet = Some(true);
            } else if eq(a, "dry") {
                c.wet = Some(false);
            } else if eq(a, "scantracks") {
                log::info!("Track directory scanning enabled");
                c.jump_mode = true;
            } else if eq(a, "hud") {
                // the original advances past the parameter but hashes the keyword itself (kept)
                if rest == 0 {
                    log::warn!("OSD type missing, ignored");
                } else {
                    c.hud = Some(crc(a));
                }
            } else if eq(a, "setflaggirl") {
                c.flag_girl = args.get(i + 1).cloned();
            } else if eq(a, "novibrate") {
                c.no_vibrate = true;
            } else if eq(a, "noaudio") {
                c.no_audio = true;
                c.no_music = true;
            } else if eq(a, "nomusic") {
                c.no_music = true;
            } else if eq(a, "nobarriers") {
                c.no_barriers = true;
            }
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jump() {
        let a: Vec<String> = ["jumptogame", "TRACKS\\LADOWN\\", "CIRCUIT\\RACE02\\", "professional", "wet"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let c = CommandLine::parse(&a);
        assert_eq!(c.game_jump, Some(("TRACKS\\LADOWN\\".into(), "CIRCUIT\\RACE02\\".into())));
        assert!(c.jump_mode && c.skip_initial_screens);
        assert_eq!((c.professional, c.wet), (Some(true), Some(true)));
        let p = CommandLine::parse(&["profiler".to_string()]);
        assert_eq!(p.game_jump.unwrap().0, "TRACKS\\LACOAST\\");
    }
}
