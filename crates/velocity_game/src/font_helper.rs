//! src\FontHelper\FontHelper.cpp — besides the font helpers (the front end's) this object range holds the attract
//! mode object `g_AttractMode` (`byte_6F5A28`, 0x4C bytes; ctor 0x000B3000): after 300 s idle on the main menu it
//! jumps into a random London race (or, one time in ten, the intro movies) with every racer under ai control and
//! the replay cameras on, and leaves it after 45 s or on Select / Start with a 0.5 s fade.

use bevy::prelude::*;
use visos::handy::crt_rand::CrtRand;

/// the race an attract start jumps to (the `CGameJump::Set` arguments)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttractRace {
    pub track: String,
    pub race: String,
    pub professional: bool,
    pub wet: bool,
    pub drag: bool,
    pub splitscreen: bool,
}

/// `aTracksLondon` 0x006856A8: 5 rows of 68 bytes (track[32], race[32], professional, wet, drag, splitscreen);
/// 0x000B3550 indexes it with `ftol(VsRandUnit() * 5)`
pub const ATTRACT_RACES: [(&str, &str, [bool; 4]); 5] = [
    ("tracks\\london\\", "circuit\\race01\\", [false, false, false, false]),
    ("tracks\\london\\", "circuit\\race02\\", [false, true, false, false]),
    ("tracks\\london\\", "circuit\\race05\\", [false, false, false, false]),
    ("tracks\\london\\", "circuit\\race06\\", [false, true, false, false]),
    ("tracks\\london\\", "sprint\\race01\\", [false, true, false, false]),
];

/// what starting the attract mode asks of the game
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttractStart {
    /// 0x000B3620: `GameManager__BE9F0` "In001", "In002", "In003" (Bink is not played here)
    Movies,
    /// 0x000B3550 → GameVars 0x000C2810: `CGameJump::Set` + `Execute(return screen)`
    Race(AttractRace),
}

/// what the in-race update (0x000B3170) asks of the game
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AttractGameOut {
    /// FadeRender 0x00123070 start(t)
    pub fade_start: Option<f32>,
    /// FadeRender 0x001230C0 set(alpha)
    pub fade_set: Option<f32>,
    /// `byte_6F5D08` (= GameManager +0x140 `m_bQuitRace`) = 1, then SetGameMode(3) (0x000B3680)
    pub end: bool,
}

/// `g_AttractMode` (`byte_6F5A28`)
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct CAttractMode {
    /// +0x00 an attract race is playing (0x000B36B0)
    pub playing: bool,
    /// +0x01 the race was given on the command line (`attractmode <track> <race>`)
    pub fixed_race: bool,
    pub track: String, // +0x02 char[32]
    pub race: String,  // +0x22 char[32]
    /// +0x42 / +0x44 the countdown (front end idle, then the race length)
    pub timer_on: bool,
    pub timer: f32,
    /// +0x48 leaving: fading out (0x000B36F0)
    pub quitting: bool,
    /// `dword_685698` front end idle time (300 s; the command line sets 1 s)
    pub idle_time: f32,
    /// `dword_68569C` attract race length (45 s; command line 300 s)
    pub race_length: f32,
    /// `dword_6856A0` chance of the movies instead of a race (0.1; command line 0)
    pub movie_chance: f32,
}

impl Default for CAttractMode {
    /// FontHelper.cpp 0x000B3000 + the .data values of `dword_685698` / `dword_68569C` / `dword_6856A0`
    fn default() -> Self {
        CAttractMode {
            playing: false,
            fixed_race: false,
            track: String::new(),
            race: String::new(),
            timer_on: false,
            timer: 0.0,
            quitting: false,
            idle_time: f32::from_bits(0x4396_0000),
            race_length: f32::from_bits(0x4234_0000),
            movie_chance: f32::from_bits(0x3DCC_CCCD),
        }
    }
}

impl CAttractMode {
    /// FontHelper.cpp 0x000B3060 — `attractmode <track> <race>` (main.cpp): that race, idle 1 s, 300 s long, no
    /// movies
    pub fn set_race(&mut self, track: &str, race: &str) {
        // sub_3DE3D0 = strcpy into char[32]
        self.track = track.chars().take(31).collect();
        self.race = race.chars().take(31).collect();
        self.idle_time = f32::from_bits(0x3F80_0000);
        self.race_length = f32::from_bits(0x4396_0000);
        self.movie_chance = 0.0;
        self.fixed_race = true;
    }

    /// FontHelper.cpp 0x000B30E0 — the main menu's update (FEMainMenuScreen 0x0023C400, every front end frame):
    /// arms the idle countdown on the first call, starts the attract mode when it runs out. `seed` = the
    /// GameManager +0x190 timer the start seeds `srand` with.
    pub fn front_end_update(&mut self, dt: f32, seed: u32, rand: &mut CrtRand) -> Option<AttractStart> {
        if self.timer_on {
            self.timer -= dt;
            if self.timer <= 0.0 {
                let s = self.start(seed, rand);
                self.timer = 0.0;
                self.timer_on = false;
                return Some(s);
            }
        } else {
            self.timer_on = true;
            self.reset_timer();
        }
        None
    }

    /// FontHelper.cpp 0x000B3430 — the race countdown, from the race start (MonitorSessions 0x000BC5F0 state 3)
    pub fn start_timer(&mut self) {
        if !self.timer_on {
            self.timer_on = true;
            self.timer = self.race_length;
        }
    }

    /// FontHelper.cpp 0x000B3470 — main menu Leave (0x0023C1A0)
    pub fn stop_timer(&mut self) {
        self.timer_on = false;
        self.timer = 0.0;
    }

    /// FontHelper.cpp 0x000B34A0 — main menu Up / Down / Select (0x0023C8B0 / 0x0023C8F0 / 0x0023CAC0): idle again
    pub fn reset_timer(&mut self) {
        if self.timer_on {
            self.timer = self.idle_time;
        }
    }

    /// FontHelper.cpp 0x000B34D0 — `srand(seed)`, then the movies (VsRandUnit ≤ chance) or a race
    pub fn start(&mut self, seed: u32, rand: &mut CrtRand) -> AttractStart {
        rand.seed(seed);
        let r = rand.rand_unit() as f32;
        if r <= self.movie_chance {
            return AttractStart::Movies;
        }
        self.playing = true;
        AttractStart::Race(self.pick_race(rand))
    }

    /// FontHelper.cpp 0x000B3550 — the command line's race, else a random table row
    pub fn pick_race(&self, rand: &mut CrtRand) -> AttractRace {
        if self.fixed_race {
            return AttractRace {
                track: self.track.clone(),
                race: self.race.clone(),
                professional: false,
                wet: false,
                drag: false,
                splitscreen: false,
            };
        }
        // ftol(VsRandUnit() * 5): rand() == RAND_MAX gives 5, one past the table (the original reads the next
        // .data bytes); clamped here
        let i = (rand.rand_unit() * 5.0) as usize;
        let (track, race, f) = ATTRACT_RACES[i.min(ATTRACT_RACES.len() - 1)];
        AttractRace {
            track: track.into(),
            race: race.into(),
            professional: f[0],
            wet: f[1],
            drag: f[2],
            splitscreen: f[3],
        }
    }

    /// FontHelper.cpp 0x000B3170 — "g_AttractMode.GameUpdate" (InGameUpdate, unpaused, while playing): the race
    /// countdown, Select (action 6) / Start (action 13) on any pad (`value != 0 || changed`) or the countdown
    /// running out fade out over 0.5 s; once the fade is (almost) black the attract mode ends.
    pub fn game_update(&mut self, dt: f32, select: bool, start: bool, fade_alpha: f32) -> AttractGameOut {
        let mut out = AttractGameOut::default();
        if self.timer_on {
            self.timer -= dt;
        }
        let quit = select || start;
        if self.quitting {
            if fade_alpha > 0.999 {
                out.fade_set = Some(1.0);
                self.timer_on = false;
                self.playing = false;
                self.quitting = false;
                out.end = true;
            }
        } else if quit || (self.timer_on && self.timer <= 0.0) {
            self.quitting = true;
            self.timer_on = false;
            out.fade_start = Some(0.5);
        }
        out
    }
}

/// InGameUpdate 0x000B7A28 "g_AttractMode.GameUpdate" (GameTick Osd set, unpaused, while playing; dt = 0x000BF670)
#[allow(clippy::too_many_arguments)]
pub fn attract_game_update(
    mut attract: ResMut<CAttractMode>,
    mut gm: ResMut<crate::GameManager>,
    clock: Res<crate::vs_common_main::GameClock>,
    ui: Res<crate::controller::user_input::UserInput>,
    fade: Option<ResMut<visos_render::fade_render::FadeRender>>,
    mut next: ResMut<NextState<crate::EGameMode>>,
    mut mw: MessageWriter<crate::GameModeMessage>,
) {
    use crate::controller::user_input::EAction;
    if !attract.playing {
        return;
    }
    // UserInput 0x001E6BC0 / 0x001E6DF0 with mask 0xFFFF: the three tests add up to `value != 0 || changed`
    let any = |a: EAction| ui.value(a, 0xFFFF) != 0.0 || ui.changed(a, 0xFFFF);
    let mut fade = fade;
    let alpha = fade.as_ref().map(|f| f.alpha()).unwrap_or(1.0);
    let o = attract.game_update(clock.timer.scaled_dt(), any(EAction::MenuSelect), any(EAction::Pause), alpha);
    if let Some(f) = fade.as_mut() {
        if let Some(t) = o.fade_start {
            log::info!("attract mode: leaving (fade {t} s)");
            f.start(t);
        }
        if let Some(a) = o.fade_set {
            f.set(a);
        }
    }
    if o.end {
        log::info!("attract mode: over");
        gm.quit_race = true;
        let mut msgs = Vec::new();
        if let Some(s) = gm.set_game_mode(3, &mut msgs) {
            next.set(s);
        }
        for m in msgs {
            mw.write(m);
        }
    }
}

/// the variables of the "System" tweaks (the cells stand in for the globals they point at)
#[derive(Resource, Clone, Debug)]
pub struct SystemTweaks {
    /// "Enum_Memory_Dump" `dword_6F5B68` (No_Dump .. Sorted_Map_Dump; read by the memory manager only)
    pub memory_dump: TweakCell,
    /// "System Update Rate" `dword_68580C` = `jump_cam::SystemUpdateRate`
    pub update_rate: TweakCell,
    /// "Scan Track Folder" `byte_6F5B75` = `DebugUnlocks.races`
    pub scan_track_folder: TweakCell,
    /// "Texture Viewer" `byte_6F5B65` (the debug texture viewer, not ported)
    pub texture_viewer: TweakCell,
}

impl Default for SystemTweaks {
    fn default() -> Self {
        Self {
            memory_dump: TweakCell::new(TweakData::Int(0)),
            update_rate: TweakCell::new(TweakData::Float(24.0)),
            scan_track_folder: TweakCell::new(TweakData::Bool(false)),
            texture_viewer: TweakCell::new(TweakData::Bool(false)),
        }
    }
}

use crate::cli::tweak::{CTweakManager, TweakCell, TweakData};

/// FontHelper.cpp 0x000B3800
pub fn add_system_tweaks(tm: &mut CTweakManager, t: &SystemTweaks) {
    log::debug!("Tweak Add System");
    let s = tm.create_system("System");
    tm.add_tweak_enum(s, "Enum_Memory_Dump", t.memory_dump.clone(), "No_Dump = 0, Basic_Dump = 1, Verbose_Dump = 2, Map_Dump = 3, Sorted_Map_Dump = 4");
    tm.add_tweak_f32(s, "System Update Rate", t.update_rate.clone(), f32::from_bits(0x3F80_0000), f32::from_bits(0x4270_0000), 1.0);
    tm.add_tweak_bool(s, "Scan Track Folder", t.scan_track_folder.clone());
    tm.add_tweak_bool(s, "Texture Viewer", t.texture_viewer.clone());
}

/// the cells mirror their game variables: copied in before the debug menu runs, back out after it
pub fn system_tweaks_in(t: Res<SystemTweaks>, rate: Option<Res<crate::game_mode::jump_cam::SystemUpdateRate>>, unlocks: Res<crate::cli::debug_unlocks::DebugUnlocks>) {
    if let Some(r) = rate {
        t.update_rate.set(TweakData::Float(r.0));
    }
    t.scan_track_folder.set(TweakData::Bool(unlocks.races));
}

pub fn system_tweaks_out(t: Res<SystemTweaks>, rate: Option<ResMut<crate::game_mode::jump_cam::SystemUpdateRate>>, mut unlocks: ResMut<crate::cli::debug_unlocks::DebugUnlocks>) {
    if let Some(mut r) = rate {
        let v = t.update_rate.f32();
        if r.0 != v {
            r.0 = v;
        }
    }
    let b = t.scan_track_folder.bool();
    if unlocks.races != b {
        unlocks.races = b;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_then_race_then_fade_out() {
        let mut a = CAttractMode::default();
        assert_eq!(a.idle_time, 300.0);
        let mut rand = CrtRand::default();
        // first call arms the idle countdown
        assert_eq!(a.front_end_update(0.1, 7, &mut rand), None);
        assert!(a.timer_on && a.timer == 300.0);
        a.reset_timer();
        let mut started = None;
        for _ in 0..3100 {
            if let Some(s) = a.front_end_update(0.1, 7, &mut rand) {
                started = Some(s);
                break;
            }
        }
        let s = started.expect("attract started");
        match s {
            AttractStart::Race(r) => {
                assert!(a.playing);
                assert!(ATTRACT_RACES.iter().any(|(t, n, _)| *t == r.track && *n == r.race));
            }
            AttractStart::Movies => assert!(!a.playing),
        }
        assert!(!a.timer_on);
        a.playing = true;
        a.start_timer();
        assert_eq!(a.timer, 45.0);
        let mut t = 0.0;
        let mut fade = 0.0f32;
        loop {
            let o = a.game_update(1.0 / 24.0, false, false, fade);
            if o.fade_start.is_some() {
                assert!((t - 45.0f32).abs() < 0.1, "fade at {t}");
                fade = 1.0;
            }
            if o.end {
                assert_eq!(o.fade_set, Some(1.0));
                break;
            }
            t += 1.0 / 24.0;
            assert!(t < 50.0);
        }
        assert!(!a.playing && !a.quitting);
    }

    #[test]
    fn command_line_race() {
        let mut a = CAttractMode::default();
        a.set_race("tracks\\ladown\\", "circuit\\race01\\");
        let mut rand = CrtRand::default();
        assert_eq!(
            a.start(1, &mut rand),
            AttractStart::Race(AttractRace {
                track: "tracks\\ladown\\".into(),
                race: "circuit\\race01\\".into(),
                professional: false,
                wet: false,
                drag: false,
                splitscreen: false
            })
        );
        assert_eq!((a.idle_time, a.race_length, a.movie_chance), (1.0, 300.0, 0.0));
    }

    #[test]
    fn select_quits() {
        let mut a = CAttractMode { playing: true, ..Default::default() };
        a.start_timer();
        let o = a.game_update(1.0 / 24.0, true, false, 0.0);
        assert_eq!(o.fade_start, Some(0.5));
        assert!(a.quitting);
    }
}
