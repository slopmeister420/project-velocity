//! in-race HUD: game flow drives the frontend's `HudPlugin` (OSD in-game screen, speedo, pause menu):
//! `HudCommand::Start` / `Stop` with the race (COSD3DInterface::OnGameInit at InGameInit), `RaceHudInput` from
//! `RaceStatus` every frame, and the pause menu bridged both ways (`GameMenuMessage` <-> `PauseMenuMsg`).

use bevy::prelude::*;
use velocity_frontend::game_menu_manager::game_menu_manager::PauseMenuRequest;
use velocity_frontend::hud::{HudCommand, PauseMenuMsg, RaceHudInput};
use velocity_frontend::osd::osd_screens::osd_in_game_screen::NameTagVehicle;
use velocity_ai::plugin::AiState;
use velocity_game::event_database::event_database_track_loading::ERaceType;
use velocity_game::game_bridge::game_bridge::GameBridge;
use velocity_game::game_data_loading::game_data_loading::ELoadStep;
use velocity_game::game_mode::race_cars::{PlayerRaceStatus, RaceStatus, RacerPoints};
use velocity_game::game_mode::race_session::{OsdCinematicBars, RaceSessionMessage};
use velocity_game::vs_common_main::{GameClock, GameTick, InGameUpdateSet};
use velocity_game::{EGameMode, GameFlowRequest, GameManager, GameMenuMessage, LoadStepMessage, PauseMenuChoice};

/// COSD3DInterface::OnGameInit + COSDMapRender::Init (InGameInit 0x000B6874..0x000B6A1E, "Generating OSD Map"):
/// no map in a cruise (event race type 1) or without a race section; else the first race section's shorter edge
/// (+0x0C when its length <= +0x08's) is the inside, the other the outside, closed = +0x08 `IsCircular`.
/// Runs in `First` on the steps (the ai's "AI World Data" step ran in the previous `Last`): the map step when the
/// race has one, else "Load 3d Interface". No hud pack choice yet: the digital fallback.
fn start(mut steps: MessageReader<LoadStepMessage>, gm: Res<GameManager>, ai: Option<Res<AiState>>, mut cmd: MessageWriter<HudCommand>) {
    for m in steps.read() {
        let Some(race) = gm.race.as_ref() else { continue };
        let map_step = race.race_type != ERaceType::Cruise;
        if !(m.step == ELoadStep::OsdMap || (m.step == ELoadStep::Load3dInterface && !map_step)) {
            continue;
        }
        let drag = race.race_type == ERaceType::Drag;
        let section = ai.as_ref().and_then(|a| a.world.as_ref()).and_then(|w| w.world.race_sections().first());
        let (mut inside, mut outside, mut closed) = (Vec::new(), Vec::new(), true);
        if let Some(s) = section.filter(|_| map_step) {
            let pts = |l: &velocity_ai::ai::vs_ai_road_line::VsAIRoadLine| l.points.iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>();
            let (a, b) = if s.right.get_length() <= s.left.get_length() { (&s.right, &s.left) } else { (&s.left, &s.right) };
            inside = pts(a);
            outside = pts(b);
            closed = s.left.is_circular();
        }
        log::info!("hud start: map edges {} / {} points, closed {closed}", inside.len(), outside.len());
        cmd.write(HudCommand::Start { hud: "digione".into(), drag, inside_edge: inside, outside_edge: outside, closed });
    }
}

/// OSD3dInterface 0x000EAF00 (MonitorSessions 0x000BC68A Init / 0x000BC79A Reset) and 0x000EAF60 (TimelineSequencer
/// 0x00175270: "InGame" camera change, SHOWHUD), read in the tick that wrote them
fn hud_slide(mut r: MessageReader<RaceSessionMessage>, mut cmd: MessageWriter<HudCommand>) {
    for m in r.read() {
        if let RaceSessionMessage::Hud(show) = m {
            cmd.write(if *show { HudCommand::HudSlideIn } else { HudCommand::HudSlideReset });
        }
    }
}

/// COSDInGameScreen 0x000F9340's inputs: driver 0's position (0x00016690) and per ai race vehicle (race manager
/// +0x24, count +0x38) its slot (+0x78), the car's render position (bridge pose, as the body is drawn), the driver
/// profile's name (0x000153B0) and colour (0x00015F80, the ai's `name_colour`).
fn name_tags(
    ai: Option<Res<AiState>>,
    bridge: Res<GameBridge>,
    clock: Res<GameClock>,
    mut input: ResMut<RaceHudInput>,
) {
    let tags = &mut input.name_tags;
    tags.vehicles.clear();
    let Some(w) = ai.as_ref().and_then(|a| a.world.as_ref()) else { return };
    if !bridge.initialised {
        return;
    }
    let t = bridge.render_lerp(clock.timer.lerp.clamp(0.0, 1.0));
    let pos = |id: Option<usize>| {
        id.filter(|i| *i < bridge.cars.len()).map(|i| bridge.car_pose(i, t)).filter(|p| p.valid).map(|p| p.position.to_array())
    };
    // COSDMapRender 0x000E8E70: the player car's position and its body row 0 (forward, VehicleEntity 0x001C66C0)
    let mut map = None;
    if let Some(id) = w.race.players.first().and_then(|v| v.base.physics).filter(|i| *i < bridge.cars.len()) {
        let pose = bridge.car_pose(id, t);
        if pose.valid {
            tags.player = pose.position.to_array();
            map = Some((pose.position.to_array(), (pose.rotation * Vec3::X).to_array()));
        }
    }
    let mut rivals = Vec::new();
    for v in &w.race.ai {
        let Some(p) = pos(v.base.physics) else { continue };
        let driver = match v.driver {
            Some(velocity_ai::ai::vs_ai_driver::DriverRef::Ai(i)) => w.race.drivers.get(i),
            _ => None,
        };
        let name = driver.and_then(|d| d.get_name(v)).unwrap_or("");
        // 0x00015F80; a vehicle without a driver keeps the item colour (+2236), white here
        let colour = driver.map_or(0xFFFF_FFFF, |d| d.name_colour());
        tags.vehicles.push(NameTagVehicle { slot: v.id as usize, pos: p, name: name.to_string(), colour });
        rivals.push(p);
    }
    if let Some((player, heading)) = map {
        input.map.player = player;
        input.map.heading = heading;
        input.map.rivals = rivals;
    }
}

fn clear_name_tags(mut input: ResMut<RaceHudInput>) {
    input.name_tags.vehicles.clear();
}

fn stop(mut cmd: MessageWriter<HudCommand>) {
    cmd.write(HudCommand::Stop);
}

/// the race manager values COSDInGameScreen::Update (0x000F99E0) reads for its player (0): player vehicle +0xF4
/// (crossed the start line; UpdateLapCrossing is skipped in a cruise), `*GetPlayerVehicle(p)` 0x000346C0 =
/// RaceInfo.dPlayerPos[p], player vehicle +0x2FC (0x00017E00) the lap, `GetRaceLength` 0x000348C0 -> 0x0003DCA0
fn race_manager_status(
    ai: Option<Res<AiState>>,
    mut st: ResMut<RaceStatus>,
    mut players: ResMut<PlayerRaceStatus>,
    mut racer_points: ResMut<RacerPoints>,
) {
    let rm = ai.as_ref().and_then(|a| a.world.as_ref()).map(|w| &w.race);
    // COSDInGameScreen 0x000F99E0: score manager (0x0003DF30) +152 / +156 / +168 and its vehicle's record +84
    let score_of = |p: usize| {
        rm.and_then(|rm| rm.players.get(p)).and_then(|v| v.score.as_ref()).map_or((0, 0, 0, 0.0), |s| {
            (s.stats.score, s.combo_score, s.combo_count, s.wrong_way_time)
        })
    };
    for p in 0..players.0.len().max(1) {
        let sc = score_of(p);
        let s = if p == 0 { &*st } else { &players.0[p] };
        if (s.points, s.combo, s.multiplier, s.wrong_way_time) != sc {
            let s = if p == 0 { &mut *st } else { &mut players.0[p] };
            (s.points, s.combo, s.multiplier, s.wrong_way_time) = sc;
        }
    }
    // OSDManager 0x000F09C0: GetVehicleInPosition(place) -> 0x000244E0 +84 per place
    let by_place: Vec<u32> = rm.map_or(Vec::new(), |rm| {
        rm.race_order.iter().map(|&r| rm.vehicle(r).and_then(|v| v.score.as_ref()).map_or(0, |s| s.stats.score)).collect()
    });
    if racer_points.0 != by_place {
        racer_points.0 = by_place;
    }
    // each player's screen reads its own player index (screen +0x14) through the same calls
    let of = |p: usize| match rm.and_then(|rm| rm.players.get(p).map(|v| (rm, v))) {
        Some((rm, v)) => (v.started, rm.player_positions.get(p).copied().unwrap_or(0), v.lap, rm.get_race_length_count()),
        None => (false, 0, 0, 0),
    };
    let (racing, position, lap, len) = of(0);
    if (st.racing, st.position, st.lap, st.race_length) != (racing, position, lap, len) {
        st.racing = racing;
        st.position = position;
        st.lap = lap;
        st.race_length = len;
    }
    // player p >= 1 (`car_outputs` copies player 0's after the physics frame)
    for p in 1..players.0.len() {
        let (racing, position, lap, len) = of(p);
        let s = &players.0[p];
        if (s.racing, s.position, s.lap, s.race_length) != (racing, position, lap, len) {
            let s = &mut players.0[p];
            s.racing = racing;
            s.position = position;
            s.lap = lap;
            s.race_length = len;
        }
    }
}

fn feed(st: Res<RaceStatus>, mut input: ResMut<RaceHudInput>) {
    input.status.racing = st.racing;
    input.status.race_length = st.race_length;
    input.status.position = st.position;
    input.status.lap = st.lap;
    input.status.laps = st.laps;
    input.status.lap_time = st.lap_time;
    input.status.points = st.points as i32;
    input.status.combo = st.combo;
    input.status.multiplier = st.multiplier;
    input.status.wrong_way = st.wrong_way_time > 1.0;
    input.speedo.speed = st.speed;
    input.speedo.rpm = st.rpm;
    input.speedo.gear = st.gear;
    input.speedo.nos = st.nos;
    input.speedo.zone = st.zone;
    input.speedo.turbo = st.turbo;
}

/// OSDManager 0x000ED6E0(show) on the game OSD manager -> the in-game OSD's bars (items 23 / 24), in tick order
fn cinematic_bars(mut r: MessageReader<OsdCinematicBars>, mut cmd: MessageWriter<HudCommand>) {
    for OsdCinematicBars(on) in r.read() {
        log::debug!("cinematic bars {on}");
        cmd.write(HudCommand::CinematicBars(*on));
    }
}

/// game flow's menu calls -> the frontend's `CGameMenuManager`
fn menus_to_frontend(mut r: MessageReader<GameMenuMessage>, mut cmd: MessageWriter<HudCommand>) {
    for m in r.read() {
        match m {
            // the debug / front end menus are core's (`velocity_game::game_menu_manager`, drawn by `debug_menu.rs`)
            GameMenuMessage::EnterPauseMenu { debug: false } => {
                cmd.write(HudCommand::OpenPauseMenu);
            }
            GameMenuMessage::EnterPauseMenu { debug: true } | GameMenuMessage::EnterFrontEndMenu => {}
            GameMenuMessage::LeaveAll => {
                cmd.write(HudCommand::LeaveAllMenus);
            }
        }
    }
}

/// pause menu choices -> game flow (options / rumble are applied through `GameOptions` by `bridge`)
fn menus_to_game(mut r: MessageReader<PauseMenuMsg>, mut out: MessageWriter<GameFlowRequest>) {
    for PauseMenuMsg(q) in r.read() {
        let c = match q {
            PauseMenuRequest::Resume => PauseMenuChoice::Resume,
            PauseMenuRequest::Restart => PauseMenuChoice::Restart,
            PauseMenuRequest::QuitToFrontend => PauseMenuChoice::QuitToFrontend,
            // GameTweaksMenu 0x000B17A0: the front end's `hud::end_of_game` writes `gm.quit_event` (`byte_6F5D09`)
            // itself before its SetGameMode(3)
            _ => continue,
        };
        log::info!("pause menu: {c:?}");
        out.write(GameFlowRequest::PauseMenu(c));
    }
}

pub fn build(app: &mut App) {
    // COSD3DInterface::OnGameInit at InGameInit; the hud lives until the race is freed (mode 2 keeps it)
    app.add_systems(First, start)
        .add_systems(OnExit(velocity_game::InRace), (stop, clear_name_tags))
        .add_systems(
            Update,
            ((feed, name_tags).run_if(in_state(EGameMode::InGame)), menus_to_frontend, menus_to_game, cinematic_bars),
        )
        .add_systems(GameTick, race_manager_status.after(InGameUpdateSet::Ai).run_if(in_state(EGameMode::InGame)))
        .add_systems(GameTick, hud_slide.after(InGameUpdateSet::Timeline).run_if(in_state(EGameMode::InGame)));
}
