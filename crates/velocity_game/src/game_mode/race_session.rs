//! src\GameMode\GameManager.cpp — the race session machine: "MonitorSessions" (0x000BC5F0) moves `+0x24`
//! through the race phases, "UpdateSessions" (0x000BD7A0) does the per-phase work. Both run every in-game update.
//! The flag girl intro (`FlagGirlIntro`, GameManager +0xC / +0x1FC) starts the race: its "ALLOWGO" event sets
//! `+0x15` (started). At the end the event progression runs (0x000BDB50) and the post race sequence starts.
//!
//! Subsystems hear about phase changes through `RaceSessionMessage`; audio gets its `AudioCommand`s here at the
//! original call sites.

use bevy::prelude::*;
use velocity_audio::AudioCommand;
use visos_render::fade_render::FadeRender;

use crate::camera::game_cameras::GameCameras;
use crate::controller::user_input::{EAction, UserInput};
use crate::event_database::event_database_bin_loading::EventOutcome;
use crate::event_database::event_database_track_loading::ERaceType;
use crate::game_input::game_input::GameInput;
use crate::game_mode::game_manager::{GameManager, GameModeMessage};
use crate::game_mode::post_race_sequence::{PostRaceContext, PostRaceEvent, PostRaceSequence};
use crate::font_helper::CAttractMode;
use crate::timeline_manager::timeline_sequencer::{intro_camera_view, FlagGirlEvent, FlagGirlIntro, FlagGirlPlacement};
use crate::vs_common_main::GameClock;

/// `+0x24` session phase
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ESessionState {
    /// 0: waiting (network sync in mp; the restart puts single player straight into Reset)
    #[default]
    Init,
    /// 1: grid reset, OSD reset
    Reset,
    /// 2: optional intro camera fly-in, then the flag girl (or straight to the start)
    Intro,
    /// 3: the flag girl intro runs until it allows the go
    Countdown,
    /// 4: cars released
    Go,
    /// 5: racing (race clock running)
    Racing,
    /// 6: race finished: event progression
    Finished,
    /// 7: post race sequence and results
    PostRace,
}

/// OSD manager flags (`dword_6FD200 +0x24`): 1 / 2 / 0x2000 as the session sets and clears them
pub mod osd_flags {
    pub const HUD: u32 = 1;
    pub const COUNTDOWN: u32 = 2;
    pub const RESULTS: u32 = 0x2000;
}

/// OSDManager 0x000ED6E0(show) on the game OSD manager (`dword_6FD200`): the in-game OSD's cinematic bars, in call
/// order within the game tick (crash camera, point of interest, ai hand-back, timeline, RestartRace). The binary
/// forwards it as the front end's `HudCommand::CinematicBars`.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OsdCinematicBars(pub bool);

/// phase changes other crates react to.
#[derive(Message, Clone, Debug, PartialEq)]
pub enum RaceSessionMessage {
    /// VsRaceManager reset to the grid (`VsRaceManager__3C5D0(rm, pursuit)`), police reset in pursuits
    ResetToGrid { pursuit: bool },
    /// zero the player vehicles' inputs (0x000BE840: steer 0, brake 0, handbrake 1), every update before the start
    ClearVehicleInputs,
    /// release the cars: vehicles leave the grid state (0x000BD9B0)
    ReleaseCars,
    /// the race manager reported the end of the race
    RaceFinished,
    /// post race sequence starts (0x000C3560)
    StartPostRace,
    /// 0x000BD8B0 (attract mode or replay): every race car back under ai control
    AiTakesRacers,
    /// OSD manager flags set / cleared (`VsCrashCamera 0x00063030` / `VsAIRaceVehicle 0x00023D00` on 0x006FD200)
    OsdFlags { set: u32, clear: u32 },
    /// OSD3dInterface 0x000EAF00 hide (with the fade) / 0x000EAF60 show
    Hud(bool),
    /// every player's camera cut and put in a mode (VsCameraManager 0x0005FFE0: 1 follow, 3 cine)
    CameraMode { mode: u32 },
    /// the flag girl intro's events (models, cameras, particles) for the renderer and cameras
    FlagGirl(FlagGirlEvent),
    /// drag OSD text (`+0x1088` string id: OSD_DRAG_ELIMINATED / WINNER / QUALIFIED / FAILURE)
    DragResult(u32),
    /// VehicleDatabase 0x001850B0: the player's car covered `laps` of the raceline
    Mileage { laps: u8 },
    /// the event is over (EventDatabaseBinLoading 0x00091890)
    EventOver(EventOutcome),
    /// what the post race sequence does to the cars
    PostRace(PostRaceEvent),
}

/// what the race manager reports (filled by the ai)
#[derive(Resource, Clone, Debug, Default)]
pub struct RaceProgress {
    /// event competitor slot of the car in each finishing place (VsAIWorld::GetVehicleInPosition, +0xF0)
    pub order: Vec<Option<usize>>,
    /// race car ids in race order (the post race line up)
    pub order_ids: Vec<u32>,
    /// player 0's place (1-based) and its race vehicle state (0x00023CE0; 6 = finished)
    pub player_position: u32,
    pub player_state: u32,
    /// per player index (split screen): RaceInfo.dPlayerPos (1-based) and the race vehicle state (0x00023CE0);
    /// [0] = `player_position` / `player_state`
    pub player_positions: Vec<u32>,
    pub player_states: Vec<u32>,
    /// race length in cars (`VsAIWorld__37D80`) and racers still in (`GetRaceLength`)
    pub racers: u32,
    pub active_racers: u32,
    /// `VsRaceManager 0x0003DB70` is human driven (0x00015D30)
    pub leader_human: bool,
    /// OSDManager 0x000F35C0 not done yet: the results screen step is still showing (false = done)
    pub results_busy: bool,
    /// CarDamage 0x000761B0 player car +0x8CC (flag girl camera side)
    pub car_flags: u32,
    /// each race car's record block (+0x1CC): time (+0x2C) and record (+0x50)
    pub racer_records: Vec<(f32, u32)>,
}

/// GameManager.cpp 0x000BE430 — the event race's track race keeps the best time and record of the race cars
pub fn update_race_records(gm: &mut GameManager, progress: &RaceProgress) {
    let g = &mut *gm;
    let Some(r) = g.vars.current_event(&g.event_db).and_then(|e| e.current_race()).and_then(|r| r.race) else { return };
    let Some(rec) = g.event_db.races.get_mut(r) else { return };
    for &(time, record) in &progress.racer_records {
        if rec.unknown_28 > time {
            rec.unknown_28 = time;
        }
        if record > rec.best_record {
            rec.best_record = record;
        }
    }
}

/// the GameManager fields the session machine uses.
#[derive(Resource, Debug, Default)]
pub struct RaceSession {
    pub state: ESessionState,   // +0x24
    /// +0x14 racing (clock and HUD)
    pub racing: bool,
    /// +0x15 the cars may go (flag girl "ALLOWGO", or no intro)
    pub started: bool,
    /// +0x17 skip pressed during intro/countdown
    pub skip: bool,
    /// +0x10 (-1 after a reset)
    pub reset_marker: i32,
    pub intro_camera: bool,     // +0x1040 an intro fly-in was set up (0x000BF160)
    pub intro_t: f32,           // +0x1044
    pub race_time: f32,         // +0x208
    /// +0x142 the next drag round follows (set by the PostRace mode, keeps the round count over the restart)
    pub drag_continue: bool,
    /// drag rounds (+0x144 count, 3 from InGameInit), +0x148 current, +0x14C round winner (car id), +0x150 out
    pub drag_rounds: u32,
    pub drag_round: u32,
    pub drag_winner: Option<u32>,
    pub drag_out: bool,
    /// +0x1048 "%d/%d" drag round text, +0x1088 drag result string id
    pub drag_text: String,
    pub drag_result: u32,
    /// set by whoever owns the race manager when every racer has finished (VsRaceManager+4)
    pub race_finished: bool,
    /// post race display step (0x000BDFE0, +0x2C) and +0x30
    pub post_race_step: u32,
    pub post_race_t: f32,
    /// +0x34 a replay is playing back (the replay manager, not ported)
    pub replay: bool,
    /// `byte_685E99` (1): the intro fly-in ends the intro when done
    pub intro_ends: bool,
}

/// GameManager.cpp 0x000BE0F0 RestartRace (the session part): back to Reset (and the audio leaves the old race).
pub fn restart(session: &mut RaceSession, audio: &mut MessageWriter<AudioCommand>) {
    audio.write(AudioCommand::StopGameAudio);
    session.state = ESessionState::Reset;
    session.race_finished = false;
    session.post_race_step = 0;
    session.intro_ends = true;
    if !session.drag_continue {
        session.drag_round = 0;
        session.drag_out = false;
    }
    session.drag_continue = false;
    session.drag_winner = None;
    session.intro_camera = false;
}

impl RaceSession {
    /// GameManager.cpp 0x000B5F80 InGameInit (the session fields)
    pub fn in_game_init() -> Self {
        RaceSession { drag_rounds: 3, intro_ends: true, ..Default::default() }
    }
}

/// `UserInput 0x001E6BC0 != 0 && 0x001E6DF0` on action 15 (Skip) for each player
fn skip_pressed(ui: &UserInput, gi: &GameInput, players: u32) -> bool {
    (0..players.max(1) as usize).any(|p| ui.pressed(EAction::Skip, gi.device_mask(p)))
}

/// the flag girl intro (`GameManager+0xC` / `+0x1FC`)
#[derive(Resource, Default)]
pub struct FlagGirl(pub Option<FlagGirlIntro>);

/// the player's grid slot and heading the intro is placed at
pub fn flag_girl_placement(gm: &GameManager, progress: &RaceProgress) -> FlagGirlPlacement {
    let d = gm.event_def.as_ref();
    let grid = d.and_then(|d| d.start_positions.first()).map(|p| Vec3::new(p[0], p[1], p[2])).unwrap_or(Vec3::ZERO);
    // dword_6F5C10 = VsCameraManager 0x000613D0(start direction)
    let heading = d.map(|d| (d.start_direction as f64 * f32::from_bits(0x3C8E_FA35) as f64) as f32).unwrap_or(0.0);
    FlagGirlPlacement { grid, heading, car_flags: progress.car_flags }
}

/// GameManager.cpp 0x000BC5F0 MonitorSessions
#[allow(clippy::too_many_arguments)]
pub fn monitor_sessions(
    mut s: ResMut<RaceSession>,
    mut gm: ResMut<GameManager>,
    mut attract: ResMut<CAttractMode>,
    ui: Res<UserInput>,
    gi: Res<GameInput>,
    progress: Res<RaceProgress>,
    mut fg: ResMut<FlagGirl>,
    mut out: MessageWriter<RaceSessionMessage>,
    mut audio: MessageWriter<AudioCommand>,
) {
    let race_type = gm.race.as_ref().map(|r| r.race_type).unwrap_or_default();
    let players = gm.num_players.max(1);
    let network = gm.vars.network_race;
    let mut fge = Vec::new();
    match s.state {
        ESessionState::Init => {
            if let Some(f) = fg.0.as_mut() {
                f.peer.show(false, &mut fge);
            }
            // the network game's state (0x000BFE60) moves it on; single player restarts straight into Reset
            out.write(RaceSessionMessage::Hud(false));
            s.race_time = 0.0;
        }
        ESessionState::Reset => {
            out.write(RaceSessionMessage::ResetToGrid { pursuit: race_type == ERaceType::Pursuit });
            if race_type == ERaceType::Drag {
                s.drag_text = format!("{}/{}", s.drag_round + 1, s.drag_rounds);
                s.drag_result = 0;
            } else {
                // GameManager.cpp 0x000BF230
                s.drag_text.clear();
                s.drag_result = 0;
            }
            s.started = false;
            s.skip = false;
            s.reset_marker = -1;
            out.write(RaceSessionMessage::ClearVehicleInputs);
            s.race_time = 0.0;
            out.write(RaceSessionMessage::Hud(false));
            out.write(RaceSessionMessage::OsdFlags { set: osd_flags::COUNTDOWN, clear: osd_flags::HUD | osd_flags::RESULTS });
            audio.write(AudioCommand::RemoveShopAlarm);
            if let Some(f) = fg.0.as_mut() {
                f.peer.show(false, &mut fge);
            }
            s.intro_t = 0.0;
            s.state = ESessionState::Intro;
        }
        ESessionState::Intro => {
            s.race_time = 0.0;
            s.skip |= skip_pressed(&ui, &gi, players);
            let mut go = !s.intro_camera || s.skip;
            if !go {
                // the fly-in camera itself is the camera module's (reads intro_t)
                s.intro_t += 0.015;
                go = s.intro_t >= 1.0 && s.intro_ends;
            }
            if go {
                let intro = fg.0.as_mut().filter(|f| f.bound);
                match intro {
                    Some(f) if race_type != ERaceType::Pursuit => {
                        let place = flag_girl_placement(&gm, &progress);
                        f.start(place, &mut fge);
                        f.cine = true;
                        out.write(RaceSessionMessage::CameraMode { mode: 3 });
                        f.peer.show(true, &mut fge);
                    }
                    _ => {
                        out.write(RaceSessionMessage::CameraMode { mode: 1 });
                        s.started = true;
                    }
                }
                s.state = ESessionState::Countdown;
            }
        }
        ESessionState::Countdown => {
            s.race_time = 0.0;
            s.skip |= skip_pressed(&ui, &gi, players);
            if network {
                s.skip = false;
            }
            // a skip jumps the intro to its "InGame" camera change
            if s.skip {
                if let Some(f) = fg.0.as_mut() {
                    if let Some(t) = f.data.in_game_time.filter(|&t| t >= 0.0 && t > f.time()) {
                        log::debug!("flag girl skipped to {t:.2}");
                        f.peer.seek = t;
                        s.skip = false;
                    }
                }
            }
            if s.started {
                s.state = ESessionState::Go;
                s.racing = true;
                // 0x000BD25C
                if attract.playing {
                    attract.start_timer();
                    log::info!("attract mode: race timer {} s", attract.timer);
                }
            } else {
                // VsAudioTrackList 0x00052B40(3.0)
                audio.write(AudioCommand::Countdown);
            }
        }
        ESessionState::Go => {
            s.drag_text.clear();
            s.drag_result = 0;
            s.race_time = 0.0;
            s.state = ESessionState::Racing;
            s.racing = false;
        }
        ESessionState::Racing => {
            if s.race_finished {
                out.write(RaceSessionMessage::OsdFlags { set: 0, clear: osd_flags::RESULTS });
                if race_type == ERaceType::Drag {
                    drag_round_over(&mut s, &progress);
                    out.write(RaceSessionMessage::DragResult(s.drag_result));
                }
                out.write(RaceSessionMessage::RaceFinished);
                s.state = ESessionState::Finished;
                // 0x000BD58E: `byte_6F5D08` = GameManager +0x140 `m_bQuitRace`
                gm.quit_race = false;
            } else if race_type == ERaceType::Drag && progress.player_state != 6 {
                s.drag_winner = None;
                s.drag_out = true;
            }
        }
        ESessionState::Finished => {
            s.state = ESessionState::PostRace;
            s.post_race_step = 0;
            s.post_race_t = 0.0;
            out.write(RaceSessionMessage::StartPostRace);
        }
        ESessionState::PostRace => {}
    }
    for e in fge {
        out.write(RaceSessionMessage::FlagGirl(e));
    }
}

/// GameManager.cpp 0x000BC5F0 state 5, drag: who goes through to the next round
fn drag_round_over(s: &mut RaceSession, p: &RaceProgress) {
    let crc = |n: &str| visos::handy::handy_crc::crc32(n.as_bytes(), 0);
    s.drag_round += 1;
    if p.player_position <= p.racers.wrapping_sub(s.drag_round) && p.player_state == 6 {
        if p.leader_human {
            s.drag_winner = None;
            s.drag_out = true;
            s.drag_result = crc("OSD_DRAG_ELIMINATED");
        } else {
            if p.active_racers < 2 {
                log::error!("assert: VsAIWorld::GetInstance()->GetNumActiveRacers() >= 2");
            }
            s.drag_result = if p.active_racers == 2 { crc("OSD_DRAG_WINNER") } else { crc("OSD_DRAG_QUALIFIED") };
        }
    } else {
        s.drag_winner = None;
        s.drag_out = true;
        s.drag_result = if p.player_state == 6 { crc("OSD_DRAG_ELIMINATED") } else { crc("OSD_DRAG_FAILURE") };
    }
}

/// GameManager.cpp 0x000BD7A0 UpdateSessions
#[allow(clippy::too_many_arguments)]
pub fn update_sessions(
    mut s: ResMut<RaceSession>,
    clock: Res<GameClock>,
    attract: Res<CAttractMode>,
    progress: Res<RaceProgress>,
    mut out: MessageWriter<RaceSessionMessage>,
    mut audio: MessageWriter<AudioCommand>,
    mut gm: ResMut<GameManager>,
    mut prs: ResMut<PostRaceSequenceRes>,
    mut next: ResMut<NextState<crate::game_mode::game_manager::EGameMode>>,
    mut mw: MessageWriter<GameModeMessage>,
    mut fade: Option<ResMut<FadeRender>>,
) {
    match s.state {
        // 0x000BE840
        ESessionState::Init => {
            out.write(RaceSessionMessage::ClearVehicleInputs);
        }
        // 0x000BD8B0
        ESessionState::Countdown => {
            out.write(RaceSessionMessage::ClearVehicleInputs);
            if s.replay || attract.playing {
                out.write(RaceSessionMessage::AiTakesRacers);
            }
        }
        // 0x000BD9B0
        ESessionState::Go => {
            out.write(RaceSessionMessage::ReleaseCars);
            audio.write(AudioCommand::RaceStarted);
            if attract.playing {
                out.write(RaceSessionMessage::OsdFlags { set: osd_flags::RESULTS, clear: 0 });
            } else {
                out.write(RaceSessionMessage::OsdFlags { set: 0, clear: osd_flags::COUNTDOWN });
            }
        }
        // GameManager.cpp 0x000BDB10: dt * time scale (0x000BF670)
        ESessionState::Racing => {
            s.race_time += clock.timer.scaled_dt();
        }
        // 0x000BDB50
        ESessionState::Finished => {
            for m in race_over(&mut gm, &progress) {
                out.write(m);
            }
        }
        // GameManager.cpp 0x000BDFE0: results OSD steps, then mode 2 (PostRace)
        ESessionState::PostRace => {
            let done = !progress.results_busy;
            match s.post_race_step {
                0 => {
                    out.write(RaceSessionMessage::OsdFlags { set: 0, clear: osd_flags::RESULTS });
                    s.post_race_step = 1;
                }
                1 if done => s.post_race_step = 2,
                2 => {
                    out.write(RaceSessionMessage::OsdFlags { set: osd_flags::RESULTS, clear: 0 });
                    s.post_race_step = 3;
                }
                3 if done => {
                    log::info!("End of Game UPDATE");
                    // GameVars 0x000C3590: the post race sequence back to idle
                    let mut ev = Vec::new();
                    let markers = crate::controller::vibrate_effects::CMarkers::default();
                    let ctx = PostRaceContext {
                        markers: &markers,
                        ground_y: &|_| None,
                        split_screen: false,
                        player: 0,
                        player_position: progress.player_position,
                        order: Vec::new(),
                    };
                    prs.0.set_state(0, &ctx, &mut ev);
                    for e in ev {
                        apply_post_race_fade(&e, fade.as_deref_mut());
                        out.write(RaceSessionMessage::PostRace(e));
                    }
                    let mut msgs = Vec::new();
                    if let Some(st) = gm.set_game_mode(2, &mut msgs) {
                        next.set(st);
                    }
                    for m in msgs {
                        mw.write(m);
                    }
                    s.post_race_step = 4;
                }
                _ => {}
            }
        }
        _ => {}
    }
}

/// GameManager.cpp 0x000BDB50 — finish positions into the event's current race, then the progression:
/// race over (EventDatabaseBinLoading 0x00091620), round over (0x00091810), event over (0x00091890)
pub fn race_over(gm: &mut GameManager, progress: &RaceProgress) -> Vec<RaceSessionMessage> {
    let mut out = Vec::new();
    let network = gm.vars.network_race;
    let g = &mut *gm;
    let Some(ev) = g.vars.current_event_mut(&mut g.event_db) else { return out };
    if let Some(race) = ev.current_race_mut() {
        let n = race.num_competitors as usize;
        for i in 1..=n {
            let car = progress.order.get(i - 1).copied().flatten();
            let mut found = false;
            for j in 0..n {
                if car.is_some() && race.competitors.get(j).copied().flatten() == car {
                    if let Some(f) = race.finish_positions.get_mut(j) {
                        *f = i as u8;
                    }
                    found = true;
                }
            }
            if !network && !found {
                log::error!("assert: finishing car not in the event race");
            }
        }
    }
    let (round_over, laps) = ev.race_finished(&mut g.rand);
    out.push(RaceSessionMessage::Mileage { laps });
    if round_over && ev.round_finished(&mut g.rand) {
        let o = ev.update_at_end_of_event();
        apply_event_outcome(g, &o);
        out.push(RaceSessionMessage::EventOver(o));
    }
    out
}

/// EventDatabaseBinLoading.cpp 0x00091890 tail — the outcome into `g_FEGameVariables`: on success the reward
/// (`dword_6F71C8 += (float)+0x18`) and the completion (0x00092D40: completed flag, unlock pass announcing into
/// `dword_6F7248`; its CVehicleDatabase::UnhideVehicles runs on the next load / save pass here), then the prize
/// and the wager into the cash (`dword_6F719C`). The team unlock stays in `EventOver` for the ai's TeamDatabase.
pub fn apply_event_outcome(g: &mut GameManager, o: &EventOutcome) {
    use crate::event_database::event_database::{CEvent, EVENT_FLAG_COMPLETED};
    use crate::game_mode::game_player::EventRef;
    if o.success {
        g.vars.reward += o.prestige_reward;
        let msg = match g.vars.event.as_mut() {
            Some(EventRef::Database(i)) => {
                let i = *i;
                CEvent::complete(&mut g.event_db, i, &g.vars)
            }
            Some(EventRef::Temp(e)) => {
                e.flags |= EVENT_FLAG_COMPLETED;
                g.event_db.evaluate_unlocks(&g.vars, true)
            }
            None => None,
        };
        if let Some(m) = msg {
            g.vars.unlock_message = m;
        }
    }
    g.vars.cash = g.vars.cash.wrapping_add(o.cash);
}

/// `g_PostRaceSequence` (0x006FA070)
#[derive(Resource, Default)]
pub struct PostRaceSequenceRes(pub PostRaceSequence);

/// "g_PostRaceSequence.Update" (InGameUpdateSet::PostRaceSequence), started from MonitorSessions
pub fn update_post_race_sequence(
    mut prs: ResMut<PostRaceSequenceRes>,
    mut rw: ParamSet<(MessageReader<RaceSessionMessage>, MessageWriter<RaceSessionMessage>)>,
    progress: Res<RaceProgress>,
    mut cams: ResMut<GameCameras>,
    gm: Res<GameManager>,
    clock: Res<GameClock>,
    mut fade: Option<ResMut<FadeRender>>,
) {
    let ctx = PostRaceContext {
        markers: &cams.markers,
        ground_y: &|_| None,
        split_screen: gm.race.as_ref().is_some_and(|r| r.splitscreen),
        player: 0,
        player_position: progress.player_position,
        order: progress.order_ids.clone(),
    };
    let mut ev = Vec::new();
    let start = rw.p0().read().any(|m| *m == RaceSessionMessage::StartPostRace);
    if start {
        prs.0.init();
        prs.0.start(&ctx, &mut ev);
    }
    // FontHelper 0x000B3710 on g_FadeRender (0 without the renderer, as in game flow tests)
    let level = fade.as_deref().map_or(0.0, FadeRender::alpha);
    prs.0.update(clock.timer.scaled_dt(), level, &ctx, &mut ev);
    let mut out = rw.p1();
    for e in ev {
        apply_post_race_fade(&e, fade.as_deref_mut());
        if e == PostRaceEvent::CameraCut {
            // 0x000C4460: 0x0002C620(0) and 0x00060380(1)
            cams.manager.render_cut = true;
            if let Some(pc) = cams.manager.players.get_mut(0) {
                pc.cut = true;
            }
        }
        out.write(RaceSessionMessage::PostRace(e));
    }
}

/// PostRaceSequence 0x000C37C0 (start(1.0), fade out) / 0x000C38E0 (start(-1.0), fade back in) on g_FadeRender;
/// 0x000C37C0's vibration stop (0x001EDD20) has no consumer
fn apply_post_race_fade(e: &PostRaceEvent, fade: Option<&mut FadeRender>) {
    if let (PostRaceEvent::Fade(t), Some(f)) = (e, fade) {
        f.start(*t);
    }
}

/// GameManager.cpp InGameUpdate tail (0x000B77F0): seek after a skip, then step the intro
pub fn update_flag_girl(
    mut fg: ResMut<FlagGirl>,
    mut s: ResMut<RaceSession>,
    gm: Res<GameManager>,
    progress: Res<RaceProgress>,
    clock: Res<GameClock>,
    mut view: ResMut<IntroCameraView>,
    mut out: MessageWriter<RaceSessionMessage>,
) {
    let Some(f) = fg.0.as_mut() else {
        view.0 = None;
        return;
    };
    let place = flag_girl_placement(&gm, &progress);
    let mut ev = Vec::new();
    f.update(clock.timer.scaled_dt(), place, &mut ev);
    for e in ev {
        if matches!(e, FlagGirlEvent::AllowGo | FlagGirlEvent::InGameCameras | FlagGirlEvent::Camera(_)) {
            log::debug!("flag girl {e:?} at {:.2}", f.time());
        }
        if e == FlagGirlEvent::AllowGo {
            s.started = true;
        }
        if e == FlagGirlEvent::InGameCameras {
            f.cine = false;
            out.write(RaceSessionMessage::CameraMode { mode: 1 });
        }
        if e == FlagGirlEvent::ShowHud {
            out.write(RaceSessionMessage::Hud(true));
        }
        out.write(RaceSessionMessage::FlagGirl(e));
    }
    view.0 = if f.cine { intro_camera_view(&f.peer) } else { None };
}

/// the flag girl intro's camera while the players' cameras show it (player 0's cine camera)
#[derive(Resource, Default, Clone, Debug)]
pub struct IntroCameraView(pub Option<crate::game_bridge::camera_bridge::CameraBridgeData>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_rounds() {
        let mut s = RaceSession { drag_rounds: 3, ..Default::default() };
        let p = RaceProgress { player_position: 1, player_state: 6, racers: 4, active_racers: 3, ..Default::default() };
        drag_round_over(&mut s, &p);
        assert_eq!(s.drag_round, 1);
        assert_eq!(s.drag_result, visos::handy::handy_crc::crc32(b"OSD_DRAG_QUALIFIED", 0));
        let p = RaceProgress { player_position: 4, player_state: 6, racers: 4, active_racers: 3, ..Default::default() };
        drag_round_over(&mut s, &p);
        assert!(s.drag_out);
        assert_eq!(s.drag_result, visos::handy::handy_crc::crc32(b"OSD_DRAG_ELIMINATED", 0));
    }
}
