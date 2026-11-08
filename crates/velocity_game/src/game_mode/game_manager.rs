//! src\GameMode\GameManager.cpp — the top-level mode machine (`GameManager+0x210` m_eGameMode) and the init
//! sequences each mode runs. Subsystem inits are not called directly: each original init step is announced as a
//! `LoadStepMessage` in the original order, and the owning crate's plugin handles the steps it owns (see
//! `game_data_loading::ELoadStep::owner`).

use bevy::prelude::*;

use crate::cli::debug_unlocks::DebugUnlocks;
use crate::cli::tweak::CTweakManager;
use crate::event_database::event_database_track_loading::{CEventDatabase, ERaceType};
use crate::game_data_loading::game_data_loading::{
    front_end_init_steps, in_game_init_steps, CEventDefinition, ELoadStep, RacePaths,
};
use crate::game_mode::game_player::CFEGameVariables;
use crate::game_mode::game_vars::{CGameJump, QuickRaceEvent, RaceSelection};
use visos::handy::crt_rand::CrtRand;
use crate::controller::user_input::{sample_devices, EAction, UserInput, ALL_DEVICES, NUM_DEVICES};
use crate::game_input::game_input::GameInput;
use crate::game_mode::race_session::{self, ESessionState, RaceSession, RaceSessionMessage};
use crate::main_cpp::CommandLine;
use crate::triggers::trigger_update::{self, CTriggers, TriggerMessage, TriggerVehicles};
use crate::timeline_manager::objects::timeline_object::{HandlerCtx, TimelineObjectMessage, TimelineWorld};
use crate::timeline_manager::race_timeline_sequencer::{
    update_race_timelines, RaceTimelineEvent, RaceTimelines, TimelineCameraView, TimelineObjectParams,
};
use crate::vs_common_main::{GameTick, InGameUpdateSet};
use velocity_audio::{AudioCommand, AudioUpdateGate, VsAudioInitInfo};

/// `m_eGameMode` (+0x210). `Boot` stands for `VsCommonMain` before `GameManager::Init`.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EGameMode {
    #[default]
    Boot,
    /// 0
    FrontEnd,
    /// 1
    InGame,
    /// 2: results / post race screens (tweak menu while drag rounds continue)
    PostRace,
    /// 3: free the game, then next race or back to the front end (0x000BE540)
    EndOfGame,
    /// 4: rebuild the front end (0x000BE730)
    ReturnToFrontEnd,
}

impl EGameMode {
    /// original mode number, `None` for Boot.
    pub fn index(self) -> Option<u32> {
        match self {
            EGameMode::Boot => None,
            EGameMode::FrontEnd => Some(0),
            EGameMode::InGame => Some(1),
            EGameMode::PostRace => Some(2),
            EGameMode::EndOfGame => Some(3),
            EGameMode::ReturnToFrontEnd => Some(4),
        }
    }
    pub fn from_index(i: u32) -> Option<Self> {
        Some(match i {
            0 => EGameMode::FrontEnd,
            1 => EGameMode::InGame,
            2 => EGameMode::PostRace,
            3 => EGameMode::EndOfGame,
            4 => EGameMode::ReturnToFrontEnd,
            _ => return None,
        })
    }
}

/// the race is loaded: modes 1 and 2. The original keeps the race through mode 2 (post race menus) and frees it
/// only in EndOfGame 0x000BE540, so race init / teardown hooks run on entering / leaving this, not `InGame`;
/// 2 -> 1 (post race restart, next drag round) is RestartRace on the loaded race.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct InRace;

impl ComputedStates for InRace {
    type SourceStates = EGameMode;
    /// computed states re-run OnExit / OnEnter on every source change by default (bevy_state 0.19), which made
    /// PostRace -> InGame (next drag round, post race restart) a full race reload; the race stays loaded
    const ALLOW_SAME_STATE_TRANSITIONS: bool = false;
    fn compute(s: EGameMode) -> Option<Self> {
        matches!(s, EGameMode::InGame | EGameMode::PostRace).then_some(InRace)
    }
}

/// `m_dNumGameModes` (+0x21C)
pub const NUM_GAME_MODES: u32 = 5;

/// one init step to run now.
#[derive(Message, Clone, Debug)]
pub struct LoadStepMessage {
    pub step: ELoadStep,
    /// race paths for in-game steps (`None` in the front end)
    pub paths: Option<RacePaths>,
}

/// mode-change side effects of `GameManager::SetGameMode` (0x000BBD30) and friends.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameModeMessage {
    /// GameManager.cpp 0x000B5CB0 FrontEndShutdown
    FrontEndShutdown,
    /// GameManager.cpp 0x000BE540 "Freed game"
    InGameShutdown,
    /// GameManager.cpp 0x000BE0F0 reset the race in place (start / restart)
    RaceRestart,
    /// tweak systems freed (0x000831A0) on most mode changes
    TweaksReset,
}

/// `GameManager` (the global at 0x006F5BC8), the parts the flow needs.
#[derive(Resource, Debug, Default)]
pub struct GameManager {
    /// +0x210 current mode index (0..4)
    pub mode: Option<u32>,
    /// +0x220 restart requested (0x000BE0D0), handled in the next update
    pub restart_pending: bool,
    /// +0x221 paused
    pub paused: bool,
    /// +0x222 the pause menu is the debug menu
    pub debug_menu: bool,
    /// +0x223 network race: paused for the menu without stopping the game
    pub network_paused: bool,
    /// +0x229 front end updates while in game
    pub front_end_in_game: bool,
    /// +0x214 / +0x218 deferred front end free
    pub free_front_end: bool,
    pub free_front_end_frames: u32,
    /// +0x140 race abandoned / quit flag (`m_bQuitRace`)
    pub quit_race: bool,
    /// +0x141 `byte_6F5D09` quit the event: written only by the post race menu (GameTweaksMenu 0x000B17A0: 1 for
    /// RETURN_TO_FE on a failed event, else 0), read by EndOfGame 0x000BE540
    pub quit_event: bool,
    /// +0x38 players
    pub num_players: u32,
    pub event_db: CEventDatabase,
    pub jump: CGameJump,
    pub race: Option<RaceSelection>,
    pub event_def: Option<CEventDefinition>,
    pub tweaks: CTweakManager,
    /// `g_FEGameVariables` (GamePlayer.cpp): career globals and the event being raced
    pub vars: CFEGameVariables,
    /// the CRT `rand()` state the event code draws from
    pub rand: CrtRand,
    /// quick race event to build when the pending jump resolves (`CFEQuickRaceStartScreen::OnSelect`)
    pub quick_race_event: Option<QuickRaceEvent>,
    /// a career event was started (CFEHangoutProgression 0x0028F430): the pending jump races the database event
    /// in `vars.event` instead of building a temporary one
    pub career_start: bool,
    /// +0x108C the trailblazer's OSD message string crc (0 = none): TrailBlazer 0x000C6200 writes it,
    /// COSDInGameScreen::Update reads it through 0x000FAEA0 (item +52); InGameInit / RestartRace zero it
    pub trail_blazer_message: u32,
}

impl GameManager {
    /// the event being raced (`dword_6F9BE8`); its current race's competitors are the race's cars
    /// (PhysicsManager.cpp 0x0019E080: humans where `CEventCompetitor::human` is set, ai opponents otherwise).
    pub fn current_event(&self) -> Option<&crate::event_database::event_database::CEvent> {
        self.vars.current_event(&self.event_db)
    }
}

impl GameManager {
    /// GameManager.cpp 0x000BBD30 — SetGameMode: returns the state to switch to (None = no change) and pushes
    /// the side-effect messages.
    pub fn set_game_mode(&mut self, new: u32, out: &mut Vec<GameModeMessage>) -> Option<EGameMode> {
        if new >= NUM_GAME_MODES || Some(new) == self.mode {
            return None;
        }
        let cur = self.mode;
        match cur {
            Some(0) => out.push(GameModeMessage::FrontEndShutdown),
            Some(_) => {}
            None => {}
        }
        if cur.is_some() && !(cur == Some(1) && new == 2) && !(cur == Some(2) && new == 1) {
            out.push(GameModeMessage::TweaksReset);
        }
        if cur == Some(3) && new == 1 {
            // InGameInit again, then restart
            self.mode = Some(1);
            self.restart_pending = true;
            return Some(EGameMode::InGame);
        }
        if cur == Some(2) && new == 1 {
            // mode 2 -> 1 keeps the race (InRace stays); the restart runs on the next InGame update
            self.mode = Some(1);
            self.restart_pending = true;
            return Some(EGameMode::InGame);
        }
        self.mode = Some(new);
        EGameMode::from_index(new)
    }

    /// GameManager.cpp 0x000BE540 — end of game: either go back to the front end (4) or start the next race (1).
    pub fn end_of_game(&mut self, back_to_front_end: bool, out: &mut Vec<GameModeMessage>) -> Option<EGameMode> {
        log::info!("End of Game UPDATE");
        out.push(GameModeMessage::InGameShutdown);
        if back_to_front_end {
            log::info!("Go back to the frontend");
            self.set_game_mode(4, out)
        } else {
            log::info!("Start the next race");
            self.set_game_mode(1, out)
        }
    }
}

/// `CGameMenuManager` calls game flow makes; the menus are the front end's (the binary forwards these).
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMenuMessage {
    /// `CGameMenuManager::EnterMenu` with a new `CGamePauseMenu` (debug: the debug menu, core's
    /// `game_menu_manager`; the front end opens only the pause menu)
    EnterPauseMenu { debug: bool },
    /// GameManager 0x000BC0C0 outside a race: `new` GameMenuManager 0x000ACDD0 "Front End" debug menu (core's)
    EnterFrontEndMenu,
    /// `CGameMenuManager::LeaveAll`
    LeaveAll,
}

/// a pause menu choice (`CGameMenu` OnSelect results), sent back by the front end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuChoice {
    Resume,
    Restart,
    QuitToFrontend,
}

/// GameManager.cpp 0x000BC0C0 — pause the race and enter the pause (or debug) menu
pub fn pause_game(gm: &mut GameManager, debug: bool, audio: &mut MessageWriter<AudioCommand>, menu: &mut MessageWriter<GameMenuMessage>) {
    if gm.vars.network_race {
        gm.network_paused = true;
    } else {
        gm.paused = true;
        gm.debug_menu = debug;
        audio.write(AudioCommand::Pause);
    }
    // mode 1: debug menu 0x000AE160 or pause menu 0x000AF070; else the front end menu 0x000ACDD0
    if gm.mode == Some(1) {
        menu.write(GameMenuMessage::EnterPauseMenu { debug });
    } else {
        menu.write(GameMenuMessage::EnterFrontEndMenu);
    }
}

/// GameManager.cpp 0x000BC2C0 — resume (the render / camera / ai / particle resumes are their owners')
pub fn unpause_game(gm: &mut GameManager, audio: &mut MessageWriter<AudioCommand>) {
    if gm.vars.network_race {
        gm.network_paused = false;
    } else {
        gm.paused = false;
        gm.debug_menu = false;
        audio.write(AudioCommand::Resume);
    }
}

/// GameManager.cpp 0x000BC3D0 — the pause button: leave the menus and resume, or pause.
/// Mode 1 pauses only once the cars are released (session state 4..6), or any time in a network race.
pub fn pause_button(
    gm: &mut GameManager,
    session: ESessionState,
    debug: bool,
    attract: bool,
    audio: &mut MessageWriter<AudioCommand>,
    menu: &mut MessageWriter<GameMenuMessage>,
) {
    // 0x000B36B0: never during attract mode (not ported: the point-of-interest / crash camera early-outs)
    if attract {
        return;
    }
    if gm.mode == Some(2) || gm.mode == Some(3) {
        return;
    }
    if gm.mode != Some(0) {
        let st = session as i32;
        if (0..=3).contains(&st) {
            if !gm.vars.network_race {
                return;
            }
        } else if st == 7 {
            return;
        }
    }
    if gm.paused || gm.network_paused {
        menu.write(GameMenuMessage::LeaveAll);
        unpause_game(gm, audio);
    } else {
        pause_game(gm, debug, audio, menu);
    }
}

/// GameManager.cpp 0x000BBF90(0) — quit the race from the pause menu: end of game, back to the front end.
/// `m_bQuitRace` (+0x140 = `byte_6F5D08`) is cleared when a race finishes (MonitorSessions 0x000BD58E) and by
/// FEDummyGameScreen 0x00209340.
pub fn quit_race(
    gm: &mut GameManager,
    audio: &mut MessageWriter<AudioCommand>,
    menu: &mut MessageWriter<GameMenuMessage>,
    out: &mut Vec<GameModeMessage>,
) -> Option<EGameMode> {
    menu.write(GameMenuMessage::LeaveAll);
    unpause_game(gm, audio);
    let s = gm.set_game_mode(3, out);
    gm.quit_race = true;
    s
}

/// GameManager.cpp 0x000BF270 — drag restart: the rounds start over, then RestartRace (deferred to the next
/// update here; the ai re-inits its own vehicles, 0x0002BDA0, on `RaceRestart`)
fn drag_restart(gm: &mut GameManager, session: &mut RaceSession) {
    session.drag_continue = false;
    session.drag_round = 0;
    session.drag_winner = None;
    session.drag_out = false;
    gm.restart_pending = true;
}

/// GameTweaksMenu.cpp 0x000B12A0 — restart confirmed in the pause / tweak menu.
/// Original quirk kept: in a race past round 0, SetGameMode(1) is a no-op, so only the event is reset.
pub fn restart_from_menu(
    gm: &mut GameManager,
    session: &mut RaceSession,
    audio: &mut MessageWriter<AudioCommand>,
    menu: &mut MessageWriter<GameMenuMessage>,
    out: &mut Vec<GameModeMessage>,
) -> Option<EGameMode> {
    let (round, drag) = gm
        .current_event()
        .map(|e| {
            let round = e.races.as_ref().map_or(0, |r| r.current_round);
            (round, e.current_race().is_some_and(|r| r.race_type == ERaceType::Drag as u32))
        })
        .unwrap_or((0, false));
    let reset_event = |gm: &mut GameManager| {
        let g = &mut *gm;
        if let Some(e) = g.vars.current_event_mut(&mut g.event_db) {
            e.reset(&mut g.rand);
        }
    };
    audio.write(AudioCommand::Resume);
    if gm.mode == Some(1) {
        if round != 0 {
            reset_event(gm);
            menu.write(GameMenuMessage::LeaveAll);
            log::info!("Restart EVENT");
            let s = gm.set_game_mode(1, out);
            unpause_game(gm, audio);
            s
        } else {
            menu.write(GameMenuMessage::LeaveAll);
            if drag {
                drag_restart(gm, session);
            } else {
                gm.restart_pending = true;
            }
            unpause_game(gm, audio);
            None
        }
    } else if round == 1 {
        reset_event(gm);
        menu.write(GameMenuMessage::LeaveAll);
        log::info!("Restart RACE");
        gm.set_game_mode(1, out)
    } else {
        reset_event(gm);
        menu.write(GameMenuMessage::LeaveAll);
        if drag {
            drag_restart(gm, session);
            None
        } else {
            gm.set_game_mode(3, out)
        }
    }
}

/// VsCommonMain.cpp 0x0016DD20 + GameManager.cpp 0x000B4D30 (databases) + 0x000BACC0 (GameManager::Init).
fn boot(
    mut gm: ResMut<GameManager>,
    cl: Res<CommandLine>,
    unlocks: Res<DebugUnlocks>,
    mut gi: ResMut<GameInput>,
    mut next: ResMut<NextState<EGameMode>>,
    mut career: ResMut<crate::game_mode::career::Career>,
    mut choice: ResMut<crate::game_mode::race_cars::PlayerCarChoice>,
    mut attract: ResMut<crate::font_helper::CAttractMode>,
    sys: Res<crate::font_helper::SystemTweaks>,
) {
    log::info!("Start...");
    // GameManager::Init 0x000BADE0 -> FontHelper 0x000B3800
    crate::font_helper::add_system_tweaks(&mut gm.tweaks, &sys);
    // GameManager::Init 0x000BAE37 -> net 0x000CDD70 ("Network", gone at the first TweaksReset)
    {
        let mut ct = crate::game_mode::core_tweaks::CoreTweaks::default();
        let mut r = crate::game_mode::core_tweaks::Reg::new(&mut gm.tweaks, &mut ct, None);
        crate::game_mode::core_tweaks::add_network_tweaks(&mut r);
    }
    gm.event_db.jump_mode = cl.jump_mode;
    // main.cpp: GamePlayer SetInitialVariables (0x000C1930), then GameManager::InitDatabases (0x000B4D30):
    // vehicles, then the event database (0x00090090) and the unlocked tier count (0x000C1DA0)
    gm.vars.set_initial_variables();
    let vehicles = velocity_physics::vehicle_database::vehicle_database::CVehicleDatabase::load()
        .map_err(|e| log::warn!("vehicle database: {e}"))
        .ok();
    let exists = |c: u32| vehicles.as_ref().is_some_and(|v| v.vehicles.iter().any(|v| v.hash == c));
    let g = &mut *gm;
    g.event_db.build(&exists, &g.vars);
    g.vars.count_unlocked_tiers(&g.event_db);
    if cl.jump_mode || unlocks.races {
        gm.event_db.set_all_unlocked(true);
    }
    if unlocks.events {
        gm.event_db.unlock_all_events();
    }
    // the garage setups of the initial garage (FECK2Manager 0x002022F0) and the current car (`dword_6F724C`)
    if let Some(db) = vehicles.as_ref() {
        gm.vars.init_garage_setups(db);
        if choice.setup.is_none() {
            if let Some(s) = gm.vars.garage_setup(gm.vars.garage_slot as usize) {
                *choice = crate::game_mode::career::car_choice(db, s);
            }
        }
    }
    career.vehicles = vehicles;
    // main.cpp 0x0016D228: `attractmode <track> <race>`
    if let Some((t, r)) = &cl.attract_mode {
        attract.set_race(t, r);
    }
    if let Some((t, e)) = &cl.game_jump {
        let (pro, wet) = (cl.professional.unwrap_or(false), cl.wet.unwrap_or(false));
        gm.jump.set(t, e, pro, wet, false);
        // the front end normally assigns pads on "press start"; a jump skips it, so player 1 gets every device
        gi.set_device_mask(0, ALL_DEVICES);
    }
    // GameManager::Init: mode starts at 0 unless already in game
    let mut msgs = Vec::new();
    if let Some(s) = gm.set_game_mode(0, &mut msgs) {
        next.set(s);
    }
}

fn announce(steps: &[ELoadStep], paths: Option<RacePaths>, w: &mut MessageWriter<LoadStepMessage>) {
    for &step in steps {
        log::debug!("Loading {}", step.label());
        w.write(LoadStepMessage { step, paths: paths.clone() });
    }
}

/// GameManager.cpp 0x000B5510 FrontEndInit
fn front_end_init(mut w: MessageWriter<LoadStepMessage>, mut audio: MessageWriter<AudioCommand>, gate: Option<ResMut<AudioUpdateGate>>) {
    announce(&front_end_init_steps(), None, &mut w);
    // CFrontendManager::Update 0x002016DD runs VsAudio::Update every front end update
    if let Some(mut g) = gate {
        g.0 = true;
    }
    // CommonInit "Audio Init" (VsAudio::Init 0x00047A00), then the front end's groups
    // (CFrontendManager::Initialise calls VsAudio 0x00049280)
    audio.write(AudioCommand::Init(VsAudioInitInfo { frontend: true, track_dir: None, race_mode: None }));
    audio.write(AudioCommand::LoadFrontEndGroups);
}

/// front end update: a pending game jump goes straight to the race (GameVars.cpp 0x000C2860 → LoadingScreen).
fn front_end_update(mut gm: ResMut<GameManager>, mut next: ResMut<NextState<EGameMode>>, mut mw: MessageWriter<GameModeMessage>) {
    if !gm.jump.active {
        return;
    }
    let db = std::mem::take(&mut gm.event_db);
    let sel = gm.jump.execute(&db);
    gm.event_db = db;
    match sel {
        Some(mut sel) => {
            // the event: the quick race start screen's (0x00221F00), else the jump's own (0x000C2C50)
            let gm = &mut *gm;
            if std::mem::take(&mut gm.career_start) {
                // the career event set up by StartEvent is raced as it is
            } else {
                // `dword_6F5C00`: MultiplayerScreen 0x00217D70 sets 2 with `byte_6F7157`, the main menu's quick
                // race 0x0023CE80 sets 1
                if let Some(q) = &gm.quick_race_event {
                    sel.num_players = if q.two_players { 2 } else { 1 };
                }
                let ev = match gm.quick_race_event.take() {
                    Some(q) => q.build(&gm.event_db, &mut gm.rand),
                    None => sel.build_jump_event(&gm.event_db, gm.jump.drag, &mut gm.rand),
                };
                match ev {
                    Some(e) => gm.vars.set_temp_event(e),
                    None => log::error!("no event built for the race"),
                }
            }
            gm.num_players = sel.num_players as u32;
            gm.race = Some(sel);
            let mut msgs = Vec::new();
            if let Some(s) = gm.set_game_mode(1, &mut msgs) {
                next.set(s);
            }
            for m in msgs {
                mw.write(m);
            }
        }
        None => {
            log::error!("game jump: race not found in TRACKS.INI");
            gm.jump.active = false;
        }
    }
}

/// GameManager.cpp 0x000B5F80 InGameInit
pub(crate) fn in_game_init(
    mut gm: ResMut<GameManager>,
    mut w: MessageWriter<LoadStepMessage>,
    mut audio: MessageWriter<AudioCommand>,
    mut session: ResMut<RaceSession>,
    mut bridge: ResMut<crate::game_bridge::game_bridge::GameBridge>,
    audio_load: (Option<ResMut<AudioUpdateGate>>, Option<ResMut<velocity_audio::input::GameAudioInput>>),
    mut clock: ResMut<crate::vs_common_main::GameClock>,
) {
    // InGameInit 0x000B6074: SetRate(24), +0x0C = 0 (0x000BF4B0), Reset 0x00057310: the first update after the
    // load is due at once and anchors the update phase (total_updates restarts at 0)
    let now = clock.now();
    clock.timer.set_rate(f32::from_bits(0x41C0_0000), 1_000_000_000);
    clock.timer.vblank_mode = 0;
    clock.timer.reset(now);
    // VsAudio::Update runs only from InGameUpdate 0x000B7EE7 (and the front end / pause), never during the load
    // steps: off until the first in-game update (`audio_update_on`); the last race's cars and camera are gone
    let (gate, input) = audio_load;
    if let Some(mut g) = gate {
        g.0 = false;
    }
    if let Some(mut i) = input {
        *i = Default::default();
    }
    let Some(race) = gm.race.clone() else {
        log::error!("InGameInit without a race selected");
        return;
    };
    let paths = RacePaths::new(&race.track_path, &race.race_path);
    // the osd map needs a race section; cruise never has one
    let has_osd_map = race.race_type != ERaceType::Cruise;
    let steps = in_game_init_steps(&race.race_path, false, has_osd_map);
    gm.event_def = CEventDefinition::load(&paths.race_dir, race.num_players);
    announce(&steps, Some(paths), &mut w);
    // CommonInit "Audio Init": VsAudio::Init with the track dir and the track's area (GameInfo+0x88)
    let area = race.event_race.map(|r| gm.event_db.tracks[gm.event_db.races[r].track].area);
    audio.write(AudioCommand::Init(VsAudioInitInfo {
        frontend: false,
        track_dir: Some(race.track_path.clone()),
        race_mode: area,
    }));
    *session = RaceSession::in_game_init();
    // InGameInit 0x000B72C4
    gm.trail_blazer_message = 0;
    // CarBridge.cpp 0x000972B0 (in game only)
    bridge.init(race.num_players.max(1) as u32, race.num_players.max(1) as u32, gm.vars.network_race);
    // the restart flag set by entering the race runs RestartRace on the next update
    gm.restart_pending = true;
}

/// GameManager.cpp 0x000B7EE7 (InGameUpdate's VsAudio::Update call): the race is loaded, audio updates again
fn audio_update_on(gate: Option<ResMut<AudioUpdateGate>>) {
    if let Some(mut g) = gate {
        if !g.0 {
            g.0 = true;
        }
    }
}

/// GameManager.cpp 0x000BB1C0 — `flt_6F6C58/5C/60` (three `VsRandRange(-10, 10)` per update) and the engine rpm
/// of each player's car (filled by `race_cars::car_outputs`), read by `CarRender::EngineShake` 0x0011ABB0
#[derive(Resource, Default, Clone, Debug)]
pub struct EngineShake {
    pub rand: [f32; 3],
    pub rpm: [i32; 4],
}

/// GameManager.cpp 0x000BB1C0 Update (mode dispatch parts the flow needs), once per game tick.
#[allow(clippy::too_many_arguments)]
fn update(
    mut shake: ResMut<EngineShake>,
    mut gm: ResMut<GameManager>,
    state: Res<State<EGameMode>>,
    mut next: ResMut<NextState<EGameMode>>,
    mut mw: MessageWriter<GameModeMessage>,
    mut session: ResMut<RaceSession>,
    mut audio: MessageWriter<AudioCommand>,
    ui: Res<UserInput>,
    gi: Res<GameInput>,
    progress: Res<race_session::RaceProgress>,
    mut bridge: ResMut<crate::game_bridge::game_bridge::GameBridge>,
    mut menu: MessageWriter<GameMenuMessage>,
    (attract, mut bars): (Res<crate::font_helper::CAttractMode>, MessageWriter<race_session::OsdCinematicBars>),
) {
    let mut msgs = Vec::new();
    // "Update Game Bridge" (GameBridge.cpp 0x00097A00)
    let paused = gm.paused;
    bridge.swap(paused);
    // 0x000BB242 / 0x000BB267 / 0x000BB28C
    for r in &mut shake.rand {
        *r = crate::timeline_manager::objects::timeline_camera::vs_rand_range(&mut gm.rand, -10.0, 10.0);
    }
    if gm.restart_pending {
        if *state.get() == EGameMode::InGame {
            msgs.push(GameModeMessage::RaceRestart);
            race_session::restart(&mut session, &mut audio);
            // RestartRace 0x000BE26D: OSDManager 0x000ED6E0(false) when the game OSD manager exists (it does in a race)
            bars.write(race_session::OsdCinematicBars(false));
            // RestartRace 0x000BE413
            gm.trail_blazer_message = 0;
        }
        gm.restart_pending = false;
    }
    match *state.get() {
        EGameMode::InGame => {
            // the pause button toggles the pause menu (the menu itself is the front end's GameMenuManager)
            // GameInput 0x000A4F10: Pause -> pause menu, else Online (BACK) -> debug menu (0x000BC3D0(1))
            let mask = gi.device_mask(0);
            if ui.pressed(EAction::Pause, mask) {
                let st = session.state;
                pause_button(&mut gm, st, false, attract.playing, &mut audio, &mut menu);
            } else if ui.pressed(EAction::Online, mask) {
                let st = session.state;
                pause_button(&mut gm, st, true, attract.playing, &mut audio, &mut menu);
            }
        }
        // GameInput 0x000A4F10 mode 0: MenuL2 (WHITE) held + MenuR2 (BLACK) pressed on any pad -> front end debug
        // menu; `dword_6F5C00` (players) 0 becomes 1 first
        EGameMode::FrontEnd => {
            if ui.value(EAction::MenuL2, 0xFFFF) != 0.0 && ui.pressed(EAction::MenuR2, 0xFFFF) {
                if gm.num_players == 0 {
                    gm.num_players = 1;
                }
                let st = session.state;
                pause_button(&mut gm, st, true, attract.playing, &mut audio, &mut menu);
            }
        }
        // GameManager.cpp 0x000BB1C0 mode 2: another drag round restarts the race (+0x142); otherwise the post race
        // menus run (the front end's GameMenuManager)
        EGameMode::PostRace => {
            let drag = gm.race.as_ref().is_some_and(|r| r.race_type == ERaceType::Drag);
            if drag && session.drag_round < session.drag_rounds && !session.drag_out {
                session.drag_continue = true;
                if let Some(s) = gm.set_game_mode(1, &mut msgs) {
                    next.set(s);
                }
            }
        }
        // GameManager.cpp 0x000BE540: back to the front end once the race (+0x140) or the event (`byte_6F5D09`)
        // was quit, else after the race records when the event is finished (+0x15 & 0x10), else the next race
        EGameMode::EndOfGame => {
            let back = if gm.quit_race || gm.quit_event {
                true
            } else {
                let finished = gm
                    .current_event()
                    .is_some_and(|e| e.flags & crate::event_database::event_database::EVENT_FLAG_FINISHED != 0);
                race_session::update_race_records(&mut gm, &progress);
                finished
            };
            if let Some(s) = gm.end_of_game(back, &mut msgs) {
                next.set(s);
            }
        }
        EGameMode::ReturnToFrontEnd => {
            // GameManager.cpp 0x000BE730: FrontEndReInit, then mode 0
            log::info!("Return to the FE");
            gm.mode = Some(4);
            if let Some(s) = gm.set_game_mode(0, &mut msgs) {
                next.set(s);
            }
        }
        _ => {}
    }
    for m in msgs {
        mw.write(m);
    }
}

/// race data loaded by the core-owned steps.
#[derive(Resource, Debug, Default)]
pub struct RaceData {
    /// LevelTriggers: track-wide then race triggers (TimelineTrigger.cpp 0x0017F980)
    pub triggers: [Option<Vec<crate::timeline_manager::objects::timeline_trigger::FileTrigger>>; 2],
    /// LoadTimelineData: `timeline.bun` (RaceTimelineSequencer.cpp 0x00170570)
    pub timeline: Option<visos::chunk::Bundle>,
    /// FlagGirlTimeline: the start intro (`g_GameManager.m_pFlagGirlTimeline`, GameDataLoading.cpp 0x0009CB30);
    /// its `in_game_time` / `allow_go_time` are the sequencer's +0xCD8 / +0xCDC countdown times
    pub flag_girl: Option<crate::timeline_manager::timeline_sequencer::FlagGirlTimeline>,
}

/// the core-owned load steps.
#[allow(clippy::too_many_arguments)]
fn core_load_steps(
    gm: Res<GameManager>,
    mut data: ResMut<RaceData>,
    mut rt: ResMut<RaceTimelines>,
    mut tp: TimelineObjectParams,
    mut fg: ResMut<race_session::FlagGirl>,
    progress: Res<race_session::RaceProgress>,
    mut r: MessageReader<LoadStepMessage>,
    mut ev: MessageWriter<RaceTimelineEvent>,
    mut ow: MessageWriter<TimelineObjectMessage>,
    mut sw: MessageWriter<RaceSessionMessage>,
) {
    for m in r.read() {
        let Some(p) = &m.paths else { continue };
        match m.step {
            ELoadStep::LevelTriggers => {
                data.triggers =
                    crate::timeline_manager::objects::timeline_trigger::load_level_triggers(&p.track_dir, &p.race_dir);
                *tp.triggers = CTriggers::from_lists(data.triggers[0].as_ref(), data.triggers[1].as_ref());
            }
            ELoadStep::FlagGirlTimeline => {
                data.flag_girl = crate::timeline_manager::timeline_sequencer::load_flag_girl_timeline(
                    "\\Timeline\\FlagGirl.vcd",
                    gm.vars.flag_girl,
                );
                // the peer (+0xC) and the engine sequencer (+0x1FC) bound to it
                let mut out = Vec::new();
                let place = race_session::flag_girl_placement(&gm, &progress);
                fg.0 = data.flag_girl.clone().map(|d| {
                    crate::timeline_manager::timeline_sequencer::FlagGirlIntro::new(d, place, &mut out)
                });
                for e in out {
                    sw.write(RaceSessionMessage::FlagGirl(e));
                }
            }
            ELoadStep::LoadTimelineData => {
                data.timeline = visos::file::platform::read(&p.timeline()).ok().and_then(|b| {
                    let mut env = visos::type_::vs_type::CEnvironment::with_builtins();
                    visos::chunk::Bundle::load(&b, &mut env).map_err(|e| log::warn!("timeline.bun: {e}")).ok()
                });
                let mut out = Vec::new();
                *rt = RaceTimelines::new(data.timeline.as_ref().map(|b| b.timelines.clone()).unwrap_or_default(), &mut out);
                // 0x00170570 "Anims" / "Models": both from `\ASP\ASP.vcd`
                let asp = visos::file::VsFileDirectoryDetails::open("\\ASP\\ASP.vcd")
                    .map_err(|_| log::warn!("Cannot open directory \\ASP\\ASP.vcd"))
                    .ok();
                rt.load_animations(asp.as_ref());
                let links = data.timeline.as_ref().and_then(|b| b.instances.as_ref()).map(|i| i.links.as_slice());
                rt.load_models(links.unwrap_or(&[]), asp.as_ref());
                let instances = data.timeline.as_ref().and_then(|b| b.instances.as_ref()).map(|i| i.instances.clone());
                let (mut msgs, mut audio, mut actions, mut triggered) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
                let mut ctx = HandlerCtx {
                    triggers: &mut tp.triggers,
                    wmm: &mut tp.wmm,
                    rand: &mut tp.rand,
                    world: &tp.world,
                    out: &mut msgs,
                    audio: &mut audio,
                    actions: &mut actions,
                    triggered: &mut triggered,
                };
                rt.bind_objects(instances.as_deref().unwrap_or(&[]), &mut ctx);
                for e in out {
                    ev.write(e);
                }
                for m in msgs {
                    ow.write(m);
                }
            }
            _ => {}
        }
    }
}

/// GameManager::CommonInit -> LicencePlateRender 0x0012DC60(`a2` = front end). Style / text are GamePlayer's
/// (`plate_style` / `plate_text`, set by PlateCustomizationScreen 0x0023AE10 / 0x0023AEF0, saved with the profile);
/// `career_requests` keeps graphics' `LicencePlate` in step with them.
fn licence_plate_common_init(p: Option<ResMut<visos_render::licence_plate_render::LicencePlate>>, front_end: bool) {
    if let Some(mut p) = p {
        p.front_end = front_end;
    }
}

/// mode-change side effects: tweak systems, the audio destructor-list shutdown and the race restart's resets
/// (GameManager.cpp 0x000BE0F0: race timelines 0x00171100, the flag girl's emitters 0x00175810, WMM 0x000C6810,
/// the post race sequence 0x000C3350).
#[allow(clippy::too_many_arguments)]
fn mode_messages(
    mut gm: ResMut<GameManager>,
    mut r: MessageReader<GameModeMessage>,
    mut audio: MessageWriter<AudioCommand>,
    mut rt: ResMut<RaceTimelines>,
    mut p: TimelineObjectParams,
    mut fg: ResMut<race_session::FlagGirl>,
    mut prs: ResMut<race_session::PostRaceSequenceRes>,
    mut ow: MessageWriter<TimelineObjectMessage>,
    sys: Res<crate::font_helper::SystemTweaks>,
) {
    for m in r.read() {
        if *m == GameModeMessage::RaceRestart {
            let (mut msgs, mut a, mut actions, mut triggered) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            let mut ctx = HandlerCtx {
                triggers: &mut p.triggers,
                wmm: &mut p.wmm,
                rand: &mut p.rand,
                world: &p.world,
                out: &mut msgs,
                audio: &mut a,
                actions: &mut actions,
                triggered: &mut triggered,
            };
            if let Some(f) = fg.0.as_mut().filter(|f| f.bound) {
                f.peer.reset_particles();
            }
            ctx.wmm.reset();
            prs.0.reset();
            rt.reset(&mut ctx);
            for m in msgs {
                ow.write(m);
            }
            for x in a {
                audio.write(x);
            }
        }
        if *m == GameModeMessage::InGameShutdown {
            // EndOfGame 0x000BE5CC → GameManager 0x000BAC30: the race timeline data unloaded
            rt.unload();
        }
        if *m == GameModeMessage::TweaksReset {
            // SetGameMode 0x000BBEFE: new root, then the "System" tweaks again (FontHelper 0x000B3800)
            gm.tweaks = CTweakManager::default();
            crate::font_helper::add_system_tweaks(&mut gm.tweaks, &sys);
        }
    }
}

/// VsAudio::ShutdownCallback (0x000488A0) sits on the destructor list CommonInit fills; FrontEndShutdown
/// (0x000B5CB0, inside SetGameMode) and the end of a game (0x000BE540) unwind it *before* the next mode's init
/// runs its `VsAudio::Init`, so this runs in `StateTransition` ahead of the `OnEnter` systems.
fn mode_audio_shutdown(mut r: MessageReader<GameModeMessage>, mut audio: MessageWriter<AudioCommand>) {
    for m in r.read() {
        if matches!(m, GameModeMessage::FrontEndShutdown | GameModeMessage::InGameShutdown) {
            audio.write(AudioCommand::Shutdown);
        }
    }
}

fn update_user_input(mut ui: ResMut<UserInput>) {
    ui.update();
}

fn not_paused(gm: Res<GameManager>) -> bool {
    !gm.paused
}

/// requests from the front end (the binary bridges `velocity_frontend::FrontendRequest` into these).
#[derive(Message, Clone, Debug)]
pub enum GameFlowRequest {
    /// `GameInput::SetDeviceMask` from the front end
    SetControllerMask { player: usize, mask: u32 },
    /// quick race start screen: race index into the event database, the chosen car (`PlayerCarChoice`).
    /// The original builds the event and goes through the LoadingScreen; the race selection it makes is the
    /// game jump's (GameVars.cpp 0x000C2860), which is what this reuses.
    StartQuickRace {
        race: usize,
        professional: bool,
        wet: bool,
        car: crate::game_mode::race_cars::PlayerCarChoice,
        event: QuickRaceEvent,
    },
    /// a pause menu choice from the front end's `CGameMenuManager`
    PauseMenu(PauseMenuChoice),
    /// profile select: new profile (FE 0x00234550)
    NewProfile { name: String },
    /// profile select: load the profile's "Primary" save (FE 0x002345E0, SaveFileData 0x001050C0)
    LoadProfile { name: String },
    /// save the current profile (FE 0x00234700, SaveFileData 0x00106040)
    SaveProfile,
    /// delete a profile's save (FE 0x002347F0)
    DeleteProfile { name: String },
    /// the hangout starts a career event (by event crc): CFEHangoutProgression 0x0028F430, then the race
    StartCareerEvent { event: u32 },
    /// race the current event's current race as it is (GameManager 0x000BBF60; DummyGameScreen "Play Race": the
    /// next race of an unfinished event, no reset). Answer: `CareerMessage::EventStarted`
    StartCurrentEvent,
}

/// read every frame (not per update: messages would expire between 24 Hz updates)
#[allow(clippy::too_many_arguments)]
fn game_flow_requests(
    mut gm: ResMut<GameManager>,
    mut gi: ResMut<GameInput>,
    mut choice: ResMut<crate::game_mode::race_cars::PlayerCarChoice>,
    mut r: MessageReader<GameFlowRequest>,
    mut session: ResMut<RaceSession>,
    mut audio: MessageWriter<AudioCommand>,
    mut menu: MessageWriter<GameMenuMessage>,
    mut mw: MessageWriter<GameModeMessage>,
    mut next: ResMut<NextState<EGameMode>>,
) {
    let mut msgs = Vec::new();
    for m in r.read() {
        match m {
            GameFlowRequest::PauseMenu(c) => {
                let s = match c {
                    PauseMenuChoice::Resume => {
                        unpause_game(&mut gm, &mut audio);
                        None
                    }
                    PauseMenuChoice::Restart => restart_from_menu(&mut gm, &mut session, &mut audio, &mut menu, &mut msgs),
                    PauseMenuChoice::QuitToFrontend => quit_race(&mut gm, &mut audio, &mut menu, &mut msgs),
                };
                if let Some(s) = s {
                    next.set(s);
                }
            }
            GameFlowRequest::SetControllerMask { player, mask } => gi.set_device_mask(*player, *mask),
            GameFlowRequest::NewProfile { .. }
            | GameFlowRequest::LoadProfile { .. }
            | GameFlowRequest::SaveProfile
            | GameFlowRequest::DeleteProfile { .. }
            | GameFlowRequest::StartCareerEvent { .. }
            | GameFlowRequest::StartCurrentEvent => {}
            GameFlowRequest::StartQuickRace { race, professional, wet, car, event } => {
                gm.quick_race_event = Some(event.clone());
                let Some(rc) = gm.event_db.races.get(*race) else {
                    log::error!("quick race: no race {race}");
                    continue;
                };
                let track = gm.event_db.tracks[rc.track].path.clone();
                let path = rc.path.clone();
                let drag = rc.race_type == ERaceType::Drag;
                gm.jump.set(&track, &path, *professional, *wet, drag);
                *choice = car.clone();
            }
        }
    }
    for m in msgs {
        mw.write(m);
    }
}

/// register the mode machine with a parsed command line.
pub fn build_with(app: &mut App, cl: CommandLine) {
    app.add_message::<AudioCommand>();
    crate::vs_common_main::build(app);
    crate::game_mode::race_cars::build(app);
    crate::game_mode::flag_girl_models::build(app);
    crate::game_mode::race_effects_bridge::build(app);
    crate::game_mode::physics_object_render::build(app);
    crate::camera::game_cameras::build(app);
    app.init_resource::<GameManager>()
        .init_resource::<EngineShake>()
        .init_resource::<crate::font_helper::CAttractMode>()
        .init_resource::<RaceData>()
        .init_resource::<RaceSession>()
        .init_resource::<GameInput>()
        .init_resource::<CTriggers>()
        .init_resource::<TriggerVehicles>()
        .init_resource::<RaceTimelines>()
        .init_resource::<crate::timeline_manager::race_timeline_sequencer::TrailBlazerTimelineCalls>()
        .init_resource::<crate::timeline_manager::race_timeline_sequencer::PhysicsObjectResets>()
        .init_resource::<TimelineWorld>()
        .init_resource::<TimelineCameraView>()
        .init_resource::<race_session::FlagGirl>()
        .init_resource::<race_session::IntroCameraView>()
        .init_resource::<race_session::RaceProgress>()
        .init_resource::<race_session::PostRaceSequenceRes>()
        .init_resource::<crate::game_bridge::game_bridge::GameBridge>()
        .add_message::<RaceTimelineEvent>()
        .add_message::<TimelineObjectMessage>()
        .add_message::<crate::message::message_manager::GameMessage>()
        .add_message::<TriggerMessage>()
        .add_message::<crate::game_mode::trail_blazer::TrailBlazerMessage>()
        .init_resource::<crate::game_mode::trail_blazer::TrailBlazerCalls>()
        .init_resource::<crate::career_teams::CareerTeams>()
        .init_resource::<crate::font_helper::SystemTweaks>()
        .init_resource::<crate::game_mode::core_tweaks::CoreTweaks>()
        .init_resource::<crate::game_mode::core_tweaks::ScoreTweaksRes>()
        .init_resource::<crate::game_menu_manager::game_menu_manager::CDebugMenuManager>()
        .add_message::<crate::career_teams::TeamRequest>()
        .insert_resource(UserInput::load(NUM_DEVICES))
        .insert_non_send(visos::input::input_manager::InputManager::new())
        .insert_resource(if cl.unlock { DebugUnlocks::all() } else { DebugUnlocks::default() })
        .insert_resource(cl)
        .init_state::<EGameMode>()
        .add_message::<LoadStepMessage>()
        .add_message::<GameModeMessage>()
        .add_message::<GameFlowRequest>()
        .add_message::<GameMenuMessage>()
        .add_message::<RaceSessionMessage>()
        .add_message::<race_session::OsdCinematicBars>()
        .add_systems(OnEnter(EGameMode::Boot), boot)
        .add_systems(OnEnter(EGameMode::FrontEnd), front_end_init)
        .add_computed_state::<InRace>()
        .add_systems(OnEnter(InRace), in_game_init)
        .add_systems(OnEnter(EGameMode::FrontEnd), |p: Option<ResMut<visos_render::licence_plate_render::LicencePlate>>| {
            licence_plate_common_init(p, true)
        })
        .add_systems(OnEnter(InRace), |p: Option<ResMut<visos_render::licence_plate_render::LicencePlate>>| {
            licence_plate_common_init(p, false)
        })
        .init_resource::<crate::game_mode::career::Career>()
        .init_resource::<crate::save_game::save_vars::SaveFlow>()
        .add_message::<crate::game_mode::career::CareerMessage>()
        .add_systems(OnEnter(EGameMode::FrontEnd), crate::game_mode::career::on_front_end)
        .add_systems(OnEnter(InRace), crate::game_mode::career::on_in_game)
        .add_systems(
            PreUpdate,
            (sample_devices, game_flow_requests, crate::game_mode::career::career_requests)
                .chain()
                .before(crate::vs_common_main::main_loop),
        )
        .add_systems(
            GameTick,
            (
                (update_user_input, update, front_end_update.run_if(in_state(EGameMode::FrontEnd)))
                    .chain()
                    .in_set(InGameUpdateSet::Input),
                // the debug menus (GameMenuManager / GameTweaksMenu), any mode, also while paused
                (
                    crate::font_helper::system_tweaks_in,
                    crate::game_mode::core_tweaks::core_tweaks_in,
                    crate::game_menu_manager::game_menu_manager::debug_menu_update,
                    crate::font_helper::system_tweaks_out,
                    crate::game_mode::core_tweaks::core_tweaks_out,
                )
                    .chain()
                    .after(update)
                    .in_set(InGameUpdateSet::Input),
                crate::game_mode::trail_blazer::apply_world_mode_stretch
                    .after(update)
                    .in_set(InGameUpdateSet::Input)
                    .run_if(in_state(EGameMode::InGame)),
                // before the physics frame (Ai-set messages; the Physics-set instance sits in race_cars' chain) and
                // after the ai's trigger pass
                crate::game_mode::trail_blazer::apply_trail_blazer_calls
                    .in_set(InGameUpdateSet::Cameras)
                    .run_if(in_state(EGameMode::InGame))
                    .run_if(not_paused),
                audio_update_on.in_set(InGameUpdateSet::Audio).run_if(in_state(InRace)),
                crate::font_helper::attract_game_update
                    .in_set(InGameUpdateSet::Osd)
                    .run_if(in_state(EGameMode::InGame))
                    .run_if(not_paused),
                race_session::monitor_sessions
                    .in_set(InGameUpdateSet::MonitorSessions)
                    .run_if(in_state(EGameMode::InGame)).run_if(not_paused),
                trigger_update::update_triggers
                    .in_set(InGameUpdateSet::Ai)
                    .run_if(in_state(EGameMode::InGame))
                    .run_if(not_paused),
                race_session::update_sessions
                    .in_set(InGameUpdateSet::UpdateSessions)
                    .run_if(in_state(EGameMode::InGame)).run_if(not_paused),
                race_session::update_post_race_sequence
                    .in_set(InGameUpdateSet::PostRaceSequence)
                    .run_if(in_state(EGameMode::InGame))
                    .run_if(not_paused),
                (
                    race_session::update_flag_girl,
                    // the trailblazer calls of the ai's trigger pass (the Ai-set ones ran before the physics frame)
                    crate::timeline_manager::race_timeline_sequencer::trail_blazer_timeline_calls,
                    update_race_timelines,
                )
                    .chain()
                    .in_set(InGameUpdateSet::Timeline)
                    .run_if(in_state(EGameMode::InGame))
                    .run_if(not_paused),
                crate::timeline_manager::race_timeline_sequencer::collect_object_resets
                    .after(InGameUpdateSet::Timeline)
                    .run_if(in_state(EGameMode::InGame)),
            ),
        )
        .add_systems(PostUpdate, (mode_messages, core_load_steps).chain())
        .add_systems(Last, crate::game_mode::core_tweaks::core_tweak_steps)
        .add_systems(
            StateTransition,
            mode_audio_shutdown.before(bevy::state::state::StateTransitionSystems::DependentTransitions),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// InGame <-> PostRace keeps the race: no OnEnter / OnExit(InRace); EndOfGame leaves it
    #[test]
    fn in_race_survives_post_race() {
        #[derive(Resource, Default)]
        struct N(u32, u32);
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<EGameMode>()
            .add_computed_state::<InRace>()
            .init_resource::<N>()
            .add_systems(OnEnter(InRace), |mut n: ResMut<N>| n.0 += 1)
            .add_systems(OnExit(InRace), |mut n: ResMut<N>| n.1 += 1);
        for (s, enter, exit) in [
            (EGameMode::InGame, 1, 0),
            (EGameMode::PostRace, 1, 0),
            (EGameMode::InGame, 1, 0),
            (EGameMode::EndOfGame, 1, 1),
            (EGameMode::InGame, 2, 1),
        ] {
            app.world_mut().resource_mut::<NextState<EGameMode>>().set(s);
            app.update();
            let n = app.world().resource::<N>();
            assert_eq!((n.0, n.1), (enter, exit), "{s:?}");
        }
    }

    #[test]
    fn set_mode_rules() {
        let mut gm = GameManager::default();
        let mut m = Vec::new();
        assert_eq!(gm.set_game_mode(0, &mut m), Some(EGameMode::FrontEnd));
        assert_eq!(gm.set_game_mode(0, &mut m), None);
        m.clear();
        assert_eq!(gm.set_game_mode(1, &mut m), Some(EGameMode::InGame));
        assert!(m.contains(&GameModeMessage::FrontEndShutdown));
        assert_eq!(gm.set_game_mode(2, &mut m), Some(EGameMode::PostRace));
        assert_eq!(gm.set_game_mode(1, &mut m), Some(EGameMode::InGame));
        assert!(gm.restart_pending);
        assert_eq!(InRace::compute(EGameMode::PostRace), Some(InRace));
        assert_eq!(InRace::compute(EGameMode::EndOfGame), None);
        assert_eq!(gm.set_game_mode(2, &mut m), Some(EGameMode::PostRace));
        assert_eq!(gm.set_game_mode(3, &mut m), Some(EGameMode::EndOfGame));
        assert_eq!(gm.end_of_game(false, &mut m), Some(EGameMode::InGame));
        assert_eq!(gm.set_game_mode(9, &mut m), None);
    }

    #[test]
    fn boots_into_jumped_race() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        build_with(&mut app, CommandLine::parse(&["jumptogame".into(), r"TRACKS\LACOAST\".into(), r"CIRCUIT\RACE01\".into()]));
        app.world_mut().resource_mut::<crate::vs_common_main::GameClock>().manual = Some(0);
        let tick = 1_000_000_000 / 24 + 1;
        let mut states = Vec::new();
        for i in 0..24 * 60u64 {
            app.world_mut().resource_mut::<crate::vs_common_main::GameClock>().manual = Some(i * tick);
            app.update();
            let s = app.world().resource::<RaceSession>();
            states.push(s.state);
            if s.state == race_session::ESessionState::Racing && s.race_time > 0.5 {
                break;
            }
        }
        assert_eq!(*app.world().resource::<State<EGameMode>>().get(), EGameMode::InGame);
        // reset, intro, the flag girl intro until its ALLOWGO, go, racing
        assert!(app.world().resource::<race_session::FlagGirl>().0.as_ref().is_some_and(|f| f.bound));
        let countdown = states.iter().filter(|&&s| s == race_session::ESessionState::Countdown).count();
        assert!(countdown > 24, "{countdown}");
        assert!(states.contains(&race_session::ESessionState::Go), "{states:?}");
        assert_eq!(*states.last().unwrap(), race_session::ESessionState::Racing);
        assert!(app.world().resource::<RaceSession>().race_time > 0.5);
        let mode = app.world().resource::<crate::camera::game_cameras::GameCameras>().manager.mode(0);
        assert_eq!(mode, crate::camera::vs_camera_manager::ECameraMode::Follow);
        let gm = app.world().resource::<GameManager>();
        assert_eq!(gm.race.as_ref().unwrap().track_path, "Tracks\\LACOAST\\");
        assert!(gm.event_def.is_some());
        let rd = app.world().resource::<RaceData>();
        assert!(rd.timeline.is_some());
    }
}
