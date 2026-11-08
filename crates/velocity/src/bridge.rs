//! glue between crates that cannot see each other (the frontend depends on game flow, so game flow cannot read
//! frontend messages itself): `FrontendRequest` -> `GameFlowRequest`, and the frontend shown / hidden with the
//! game mode (`CFrontendManager::Start` / `Stop` at the original's mode changes).

use bevy::prelude::*;
use velocity_frontend::{FrontendCommand, FrontendRequest, FrontendRequestMsg, FrontendState};
use velocity_game::cli::debug_unlocks::DebugUnlocks;
use velocity_game::game_mode::game_vars::QuickRaceEvent;
use velocity_game::game_mode::race_cars::PlayerCarChoice;
use velocity_frontend::game_options::{display, GameOptions};
use velocity_game::camera::game_cameras::GameCameras;
use velocity_game::controller::user_input::UserInput;
use velocity_frontend::string_manager::string_manager::string_crc;
use velocity_game::game_mode::career::{Career, CareerMessage};
use velocity_game::register_file_types::save_file_data::OptionsBlock;
use velocity_game::{EGameMode, GameFlowRequest, GameManager};

fn frontend_requests(
    mut r: MessageReader<FrontendRequestMsg>,
    state: Res<FrontendState>,
    mut out: MessageWriter<GameFlowRequest>,
    mut fe_cmd: MessageWriter<FrontendCommand>,
) {
    for FrontendRequestMsg(req) in r.read() {
        match req {
            FrontendRequest::SetControllerMask { player, mask } => {
                out.write(GameFlowRequest::SetControllerMask { player: *player as usize, mask: *mask });
            }
            FrontendRequest::StartQuickRace => {
                let Some(f) = &state.frontend else { continue };
                let q = &f.fe.quick_race;
                let Some(race) = q.races[0] else {
                    log::error!("StartQuickRace without a race");
                    continue;
                };
                // the chosen car: vehicle folder and its first single-player camera view
                let mut car = PlayerCarChoice::default();
                if let (Some(setup), Some(db)) = (&q.player_setups[0], &f.fe.vehicles) {
                    if let Some(v) = setup.vehicle.and_then(|i| db.vehicles.get(i)) {
                        car.vehicle_path = Some(v.path.clone());
                        car.setup = Some(setup.clone());
                        car.camera_view = v.data.list_a.first().copied();
                        car.camera_views = v.data.list_a.clone();
                    }
                }
                // split screen: player 2's setup (`dword_6F7478`)
                if q.two_players {
                    car.setup2 = q.player_setups[1].clone();
                }
                log::info!("quick race: race {race}, car {:?}, two players {}", car.vehicle_path, q.two_players);
                // the globals CFEQuickRaceStartScreen::OnSelect (0x00221F00) builds the event from
                let event = QuickRaceEvent {
                    single: q.single,
                    races: q.races,
                    professional: q.pro,
                    race_type: q.race_type,
                    laps: q.laps as u8,
                    opponents: q.opponents as u8,
                    two_players: q.two_players,
                    flag1: q.weather != 0,
                    flag2: q.drag,
                    tier: q.vehicle_tier,
                };
                out.write(GameFlowRequest::StartQuickRace { race, professional: q.pro, wet: q.weather == 1, car, event });
                fe_cmd.write(FrontendCommand::Stop);
            }
            FrontendRequest::Exit => {}
            FrontendRequest::PlayIntroMovies => {}
        }
    }
}

/// the debug unlock tweaks (`velocity_game::cli::debug_unlocks`) applied to the front end's own databases once
/// it exists: vehicles through `byte_6877F0`, races like the 0x00202EA0 callback does for the game's copy.
fn frontend_unlocks(unlocks: Res<DebugUnlocks>, mut state: ResMut<FrontendState>, mut done: Local<bool>) {
    if *done {
        return;
    }
    let Some(f) = state.frontend.as_mut() else { return };
    *done = true;
    f.fe.quick_race.all_unlocked = unlocks.vehicles;
    if unlocks.races {
        f.fe.events.set_all_unlocked(true);
    }
}

/// `GameOptions.volumes` (pause menu order: master, music, engine, sfx, speech, tyres) -> the options block's
/// address order (master +8, music +C, engine +10, tyres +14, speech +18, sfx +1C)
const VOLUME_SLOT: [usize; 6] = [0, 1, 2, 5, 4, 3];

/// the profile's option globals (`byte_6F915C..`, core's `Career.options`) as the front end's `GameOptions`
fn block_to_options(b: &OptionsBlock, o: &mut GameOptions) {
    for (i, &s) in VOLUME_SLOT.iter().enumerate() {
        o.volumes[i] = b.volumes[s];
    }
    o.soundtrack = b.soundtrack;
    o.audio_mode = b.audio_mode;
    o.rumble = b.rumble;
    o.display[display::RATIO_WIDE] = b.widescreen;
    o.display[display::TFT] = b.speech;
    o.display[display::NAMES] = b.player_names;
    o.controller_config[0] = b.controller_config;
}

fn options_to_block(o: &GameOptions, b: &mut OptionsBlock) {
    for (i, &s) in VOLUME_SLOT.iter().enumerate() {
        b.volumes[s] = o.volumes[i];
    }
    b.soundtrack = o.soundtrack;
    b.audio_mode = o.audio_mode;
    b.rumble = o.rumble;
    b.widescreen = o.display[display::RATIO_WIDE];
    b.speech = o.display[display::TFT];
    b.player_names = o.display[display::NAMES];
}

/// a new profile (SetInitialVariables) or a loaded one (0x001050C0 copies the saved block over `byte_6F915C`)
/// replaces the option globals: push them into the front end's `GameOptions` (its change detection re-applies
/// audio and display) before `apply_options` reads them back
fn profile_options(
    mut msgs: MessageReader<CareerMessage>,
    career: Res<Career>,
    options: Option<ResMut<GameOptions>>,
) {
    let push = msgs
        .read()
        .any(|m| matches!(m, CareerMessage::Created { .. } | CareerMessage::Loaded { ok: true, .. }));
    let Some(mut o) = options else { return };
    if push {
        let mut n = o.clone();
        block_to_options(&career.options, &mut n);
        if n != *o {
            *o = n;
        }
    }
}

/// the player options the frontend edits, applied where game flow reads them: controller mapping per pad
/// (UserInput +0x14), 16:9 (`byte_6F915D`, VsCameraManager 0x0005F8F0), trash talk (`byte_6F915E`) and
/// player names (`byte_6F915F`), and the profile's option globals (`Career.options`, saved with the profile).
/// Rumble has no consumer yet (no vibration output).
fn apply_options(
    options: Option<Res<GameOptions>>,
    mut ui: ResMut<UserInput>,
    mut cams: ResMut<GameCameras>,
    mut gm: ResMut<GameManager>,
    mut career: ResMut<Career>,
    mut applied: Local<Option<[u8; 4]>>,
) {
    let Some(o) = options else { return };
    let mut b = career.options;
    options_to_block(&o, &mut b);
    if b != career.options {
        career.options = b;
    }
    // the globals are plain copies (SetInitialVariables may rewrite the game's own on a new game)
    // the 4:3 / 16:9 option is retired (frontend 2026-11-09: no options row, the UI adapts to the window aspect),
    // so the camera's 16:9 flag stays off whatever an old options.ini / save holds
    let (wide, tft, names) = (false, o.display[display::TFT], o.display[display::NAMES]);
    if cams.manager.widescreen != wide {
        cams.manager.widescreen = wide;
    }
    // byte_75B278 "Jump Cam" (read by the physics jump cam through `jump_cam_pre`)
    if cams.replay.jump_cam_option != o.display[display::JUMP_CAM] {
        cams.replay.jump_cam_option = o.display[display::JUMP_CAM];
    }
    if gm.vars.speech != tft || gm.vars.player_names != names {
        gm.vars.speech = tft;
        gm.vars.player_names = names;
    }
    let cfg = o.controller_config;
    if applied.as_ref() != Some(&cfg) {
        for (pad, &c) in cfg.iter().enumerate() {
            if applied.is_none_or(|a| a[pad] != c) {
                ui.set_config(pad, c);
            }
        }
        *applied = Some(cfg);
    }
}

/// the career variables live in core (`g_FEGameVariables`); the front end reads / writes `gm.vars` around its own
/// update (`run_frontend`), so nothing is copied here. A started career event points the front end's post race /
/// return screens at RaceCompleteScreen / HangoutScreen (MainMenuScreen fallback) and hides it.
fn career_sync(mut msgs: MessageReader<CareerMessage>, mut state: ResMut<FrontendState>, mut fe_cmd: MessageWriter<FrontendCommand>) {
    for m in msgs.read() {
        match m {
            CareerMessage::EventStarted { .. } => {
                if let Some(f) = state.frontend.as_mut() {
                    f.fe.post_race_screen = string_crc("RaceCompleteScreen");
                    let hangout = string_crc("HangoutScreen");
                    f.fe.return_screen =
                        if f.fe.find_screen(hangout).is_some() { hangout } else { string_crc("MainMenuScreen") };
                }
                fe_cmd.write(FrontendCommand::Stop);
            }
            _ => {}
        }
    }
}

pub fn build(app: &mut App) {
    app.add_systems(Update, (frontend_unlocks, frontend_requests, profile_options, apply_options, career_sync).chain());
    let _ = EGameMode::FrontEnd;
}
