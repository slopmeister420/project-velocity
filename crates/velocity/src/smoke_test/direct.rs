//! `SMOKE_DIRECT=1`: boot straight into the race chosen by the usual SMOKE_AREA / SMOKE_TRACK / SMOKE_RACE_TYPE /
//! SMOKE_RACE / SMOKE_RACE_PATH / SMOKE_WET variables, without the menu presses. The front end's own quick race
//! screen is entered once (so its option defaults are computed by the screen, not copied here), the selections are
//! stored the way `qr_select` / the car select screen store them, and `FrontendRequest::StartQuickRace` is pushed,
//! which `bridge::frontend_requests` turns into the same `GameFlowRequest::StartQuickRace` the loading screen
//! causes. After the race the pause-menu walk of the normal smoke test still runs; the run ends 4 s after the
//! front end is back (the quick race start screen is never visited in this mode).

use bevy::prelude::*;
use velocity_frontend::{FrontendRequest, FrontendState};
use velocity_game::event_database::event_database_track_loading::CEventDatabase;
use velocity_game::EGameMode;
use velocity_physics::vehicle_database::vehicle_database::VehicleSetup;

use super::{crc, Script};

pub fn enabled() -> bool {
    std::env::var_os("SMOKE_DIRECT").is_some() || split()
}

/// `SMOKE_SPLIT=1` (implies SMOKE_DIRECT): a two-player split-screen race (MultiplayerScreen 0x00217D70 entry 2:
/// `two_players`, both players on the same car), player 0 on pad 0, player 1 on pad 1 (scripted port 1)
pub fn split() -> bool {
    std::env::var_os("SMOKE_SPLIT").is_some()
}

fn race_key(ev: &CEventDatabase, r: usize) -> String {
    let race = &ev.races[r];
    format!("{}\\{}", ev.tracks[race.track].name, race.path).to_ascii_uppercase().trim_end_matches('\\').to_string()
}

/// the race the plain smoke test would step to (see the QuickRaceTextScreen branch of `run`)
fn pick_race(ev: &CEventDatabase) -> Result<usize, String> {
    if let Ok(p) = std::env::var("SMOKE_RACE_PATH") {
        let p = p.replace('/', "\\").to_ascii_uppercase().trim_matches('\\').to_string();
        return (0..ev.races.len()).find(|&r| race_key(ev, r) == p).ok_or(format!("SMOKE_RACE_PATH {p} not in the event database"));
    }
    let ty: usize = std::env::var("SMOKE_RACE_TYPE").ok().and_then(|v| v.parse().ok()).unwrap_or(3);
    let area: u32 = std::env::var("SMOKE_AREA").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let ladown = || ev.tracks.iter().position(|t| t.name.eq_ignore_ascii_case("LADOWN"));
    let track = match std::env::var("SMOKE_TRACK") {
        Ok(n) => ev.tracks.iter().position(|t| t.name.eq_ignore_ascii_case(&n)),
        Err(_) if area == 1 => ladown(),
        Err(_) => ev.tracks.iter().position(|t| t.area == area && t.races.get(ty).is_some_and(|r| !r.is_empty())).or_else(ladown),
    }
    .ok_or("no such track")?;
    let list = ev.tracks[track].races.get(ty).ok_or("bad SMOKE_RACE_TYPE")?;
    let want = std::env::var("SMOKE_RACE").ok().and_then(|v| v.parse::<u32>().ok());
    let hit = match want {
        Some(n) => list.iter().copied().find(|&r| ev.races[r].path.to_ascii_uppercase().contains(&format!("RACE{n:02}"))),
        None => list.iter().copied().find(|&r| ev.races[r].category == 1 && ev.races[r].flags & 2 != 0).or(list.first().copied()),
    };
    hit.ok_or_else(|| format!("track {} has no such race", ev.tracks[track].name))
}

/// SMOKE_SPLIT: pad 1 (player 1): full throttle from 1 s after Go to SMOKE_DRIVE (default 12), a little left steer
/// 3..5 s so the two cars part; neutral outside the race. Logs both players' HUD status every 2 s, shot `1c_split`.
#[allow(clippy::too_many_arguments)]
pub fn split_pad(
    mut commands: Commands,
    mut s: ResMut<Script>,
    clock: Res<velocity_game::vs_common_main::GameClock>,
    mode: Res<State<EGameMode>>,
    session: Res<velocity_game::game_mode::race_session::RaceSession>,
    status: Res<velocity_game::game_mode::race_cars::PlayerRaceStatus>,
    cars: Query<(&velocity_game::game_mode::race_cars::RaceCar, &velocity_physics::plugin::PhysicsCarState)>,
    mut exit: MessageReader<AppExit>,
    mut last: Local<i32>,
) {
    use velocity_game::game_mode::race_session::ESessionState;
    use visos::input::input_device::{key, NUM_KEYS};
    let mut k = [0.0f32; NUM_KEYS];
    let racing = *mode.get() == EGameMode::InGame && matches!(session.state, ESessionState::Go | ESessionState::Racing);
    if s.race_t.is_some() && racing {
        let dt = (clock.timer.total_updates - s.race_u) as f32 * clock.timer.dt;
        let drive: f32 = std::env::var("SMOKE_DRIVE").ok().and_then(|v| v.parse().ok()).unwrap_or(12.0);
        if dt > 1.0 && dt < drive {
            k[key::RTRIGGER] = 1.0;
        }
        if (3.0..5.0).contains(&dt) {
            k[key::STICK_LX] = -0.3;
        }
        if dt > 4.0 {
            super::shot(&mut commands, &mut s, "1c_split");
        }
        if (dt * 0.5) as i32 != *last {
            *last = (dt * 0.5) as i32;
            let pos = |p: usize| cars.iter().find(|(c, _)| c.slot == p && c.human).map(|(_, st)| st.position);
            for (p, st) in status.0.iter().enumerate() {
                log::info!(
                    "smoke-test: split {dt:.0} s player {p} at {:?} position {}/{} lap {}/{} speed {:.1} rpm {:.0} gear {}",
                    pos(p),
                    st.position,
                    st.race_length,
                    st.lap,
                    st.laps,
                    st.speed,
                    st.rpm,
                    st.gear
                );
            }
        }
    }
    let done = exit.read().count() > 0;
    visos::input::input_manager::set_port_script(1, (!done).then_some(k));
}

#[derive(Default)]
pub struct Phase {
    step: u32,
    frames: u32,
    race: usize,
    back_at: Option<f32>,
}

pub fn drive(
    mut fe: ResMut<FrontendState>,
    mode: Res<State<EGameMode>>,
    mut s: ResMut<Script>,
    mut exit: MessageWriter<AppExit>,
    mut p: Local<Phase>,
) {
    // after the race: the front end is back (or the quit never got there) -> done
    if s.quit.is_some() && *mode.get() != EGameMode::InGame {
        let now = s.t;
        let at = *p.back_at.get_or_insert(now);
        if now - at > 4.0 && s.done_at.is_none() {
            s.done_at = Some(now);
        }
        return;
    }
    if p.step >= 3 || *mode.get() == EGameMode::InGame {
        return;
    }
    let Some(f) = fe.frontend.as_mut() else { return };
    if f.fe.current.is_none() {
        return;
    }
    p.frames += 1;
    let text = f.quick_race_text.clone();
    match p.step {
        0 if p.frames > 30 => {
            let r = match pick_race(&f.fe.events) {
                Ok(r) => r,
                Err(e) => {
                    log::error!("smoke-test: direct: {e}");
                    exit.write(AppExit::error());
                    return;
                }
            };
            let race = &f.fe.events.races[r];
            let q = &mut f.fe.quick_race;
            q.track = Some(race.track);
            q.race_type = race.race_type as u32;
            q.races = [Some(r); 3];
            q.pro = race.category == 2;
            q.weather = std::env::var_os("SMOKE_WET").is_some() as u32;
            p.race = r;
            p.frames = 0;
            p.step = 1;
            f.fe.goto_screen(crc("QuickRaceTextScreen"));
        }
        1 => {
            let cur = f.fe.current.map(|c| f.fe.screens[c].crc).unwrap_or(0);
            if cur == crc("QuickRaceTextScreen") && p.frames > 10 {
                p.step = 2;
            } else if p.frames > 600 {
                log::error!("smoke-test: direct: quick race screen not reached");
                exit.write(AppExit::error());
            }
        }
        2 => {
            // qr_select (0x00228D80)
            let (laps, opponents, difficulty, weather, pro) = match text.lock() {
                Ok(t) => (t.laps, t.opponents, t.difficulty, t.weather, t.pro),
                Err(_) => return,
            };
            let r = p.race;
            let id = f.fe.events.races[r].id_crc;
            let q = &mut f.fe.quick_race;
            q.selected_race_id = id;
            q.pro = pro;
            q.laps = laps;
            q.opponents = opponents;
            q.difficulty = difficulty;
            q.weather = weather;
            q.drag = q.race_type == 5;
            // the car select screen's choice (SMOKE_CAR=<vehicle path>, default the 300ZX), default setup, automatic
            let mut setup = VehicleSetup::new();
            let mut tier = 0;
            let (progress, all) = (q.progress, q.all_unlocked);
            let mut hash = 0;
            if let Some(db) = f.fe.vehicles.as_ref() {
                // the plain smoke ends up on the Tokyo 300ZX (the car on the spot in the car select foyer)
                let want = std::env::var("SMOKE_CAR").unwrap_or_else(|_| "\\VEHICLES\\TOKYO\\300ZX".into());
                let named = db.vehicles.iter().position(|v| v.path.eq_ignore_ascii_case(&want));
                if let Some(v) = named.or_else(|| db.get_nth_available_vehicle_in_tier(0, 1, progress, all)) {
                    hash = db.vehicles[v].hash;
                    db.setup_default_vehicle(&mut setup, hash);
                }
                if let Some(v) = setup.vehicle {
                    tier = db.vehicles[v].data.tier as u32;
                }
            }
            setup.unk_18c = 1;
            let q = &mut f.fe.quick_race;
            q.selected_vehicle = hash;
            q.car_name_crc = hash;
            q.vehicle_tier = tier;
            q.automatic = true;
            q.player_setups[0] = Some(setup.clone());
            let two = split();
            if two {
                q.two_players = true;
                q.car_name_crc2 = hash;
                q.player_setups[1] = Some(setup);
            }
            log::info!(
                "smoke-test: direct race {} laps {laps} opponents {opponents} weather {weather} car {hash:08x} (quick race {})",
                race_key(&f.fe.events, r),
                r
            );
            // the main menu screen's init (fe_main_menu_screen.rs:247) gives pad mask 0xFFFF to player 0
            f.fe.post_race_screen = crc("RaceCompleteScreen"); // fe_quick_race_start_screen.rs:296
            // split: the pad that pressed select (port 0) is player 0's, every other device player 1's (0x0023CE80's
            // masks, as the split-screen path keeps them)
            let (m0, m1) = if two { (1, !1u32) } else { (0xFFFF, 0) };
            f.fe.requests.push(FrontendRequest::SetControllerMask { player: 0, mask: m0 });
            f.fe.requests.push(FrontendRequest::SetControllerMask { player: 1, mask: m1 });
            f.fe.requests.push(FrontendRequest::StartQuickRace);
            p.step = 3;
        }
        _ => {}
    }
}
