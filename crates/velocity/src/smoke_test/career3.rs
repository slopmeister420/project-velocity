//! `SMOKE_CAREER=3`: three career events in a row on a fresh profile (frontend's check of the garage and the career
//! progression). The ai drives the player car (`PlayerAIControl`); the post race menu takes its default (A);
//! DummyGameScreen (later races of a multi-race event) gets the debug result 1ST. After event 1 the garage buys a
//! part (CarCustomisationScreen: front bumper style 2, shot `<prefix>_garage_buy`), after event 2 the profile is
//! reloaded from its autosave, and event 3 must show the part on the race car. Each step is logged as
//! `smoke-test: career3 ...`; the run ends once the hangout settles after event 3.

use super::{crc, press, shot, Script};
use bevy::prelude::*;
use velocity_frontend::frontend::Frontend;
use velocity_game::game_mode::career::CareerMessage;
use velocity_game::GameManager;
use visos::input::input_device::key;

pub fn enabled() -> bool {
    std::env::var("SMOKE_CAREER").is_ok_and(|v| v == "3")
}

#[derive(Default)]
pub struct C3 {
    /// EventStarted / ReturnedFromRace seen
    pub events: u32,
    pub returned: u32,
    /// garage: 0 not yet, 1 on the customisation screen, 2 bought, 3 back in the hangout
    pub bought: u8,
    pub buy_step: usize,
    pub cash_before: i32,
    /// 0, 1 reload asked (after event 2), 2 reloaded
    pub reload: u8,
    /// time the post race menu came up
    pub post_at: Option<f32>,
    pub dialog_step: u8,
    /// last logged hangout after-race step
    pub step: u32,
}

const POST: [&str; 4] = ["c3_post_race1", "c3_post_race2", "c3_post_race3", "c3_post_race4"];
const RACE: [&str; 4] = ["c3_race1", "c3_race2", "c3_race3", "c3_race4"];
const BACK: [&str; 4] = ["c3_hangout1", "c3_hangout2", "c3_hangout3", "c3_hangout4"];

fn bodyparts(s: Option<&velocity_physics::vehicle_database::vehicle_database::VehicleSetup>) -> String {
    s.map_or("none".into(), |s| {
        let v: Vec<String> =
            s.fitted_bodyparts.iter().enumerate().filter_map(|(t, p)| p.map(|p| format!("{t}:{}/{}", p.set, p.index))).collect();
        format!("vehicle {:?} parts [{}]", s.vehicle, v.join(" "))
    })
}

fn summary(gm: &GameManager) -> String {
    let done = gm.event_db.events.iter().filter(|e| e.flags & 2 != 0).count();
    let open = gm.event_db.events.iter().filter(|e| e.flags & 3 == 0).count();
    format!(
        "cash {} prestige {} team {:08x} completed {done} open {open} tiers {:?}",
        gm.vars.cash, gm.vars.prestige, gm.vars.team, gm.vars.unlocked_tiers
    )
}

/// career messages (from `run`'s reader)
pub fn on_message(s: &mut Script, m: &CareerMessage, gm: &GameManager, flow: &mut MessageWriter<velocity_game::GameFlowRequest>) {
    match m {
        CareerMessage::EventStarted { event } => {
            s.c3.events += 1;
            // keep the plain career smoke's reload / second race branches out of the way
            s.career = 20;
            s.race_t = None;
            s.intro_t = None;
            log::info!("smoke-test: career3 event {} started {event:08x}: {}", s.c3.events, summary(gm));
        }
        CareerMessage::ReturnedFromRace { saved } => {
            s.c3.returned += 1;
            s.c3.post_at = None;
            let ev = gm.vars.current_event(&gm.event_db).map(|e| (e.crc, e.flags));
            log::info!("smoke-test: career3 back from event {} saved={saved} event {ev:08x?}: {}", s.c3.returned, summary(gm));
            log::info!("smoke-test: career3 garage slot {} {}", gm.vars.garage_slot, bodyparts(gm.vars.garage_setup(gm.vars.garage_slot as usize)));
        }
        CareerMessage::Loaded { ok, .. } if s.c3.reload == 1 => {
            s.c3.reload = 2;
            log::info!("smoke-test: career3 reloaded ok={ok}: {}", summary(gm));
            log::info!("smoke-test: career3 reloaded garage slot {} {}", gm.vars.garage_slot, bodyparts(gm.vars.garage_setup(gm.vars.garage_slot as usize)));
        }
        _ => {}
    }
    // after event 2 (bought part, autosaved): reload the profile, the hangout comes back with the loaded garage
    if matches!(m, CareerMessage::ReturnedFromRace { .. }) && s.c3.returned == 2 && s.c3.reload == 0 {
        let name = s.profile.clone().unwrap_or_else(|| gm.vars.profile_name.clone());
        s.c3.reload = 1;
        flow.write(velocity_game::GameFlowRequest::LoadProfile { name });
    }
}

/// the ai takes the player car once the race runs (the review switch's `PlayerHumanControl`, applied on GameTick;
/// `PlayerAIControl` is only read at the race vehicle's reset, before the smoke can set it), again after a restart
pub fn ai_drive(
    mode: Res<State<velocity_game::EGameMode>>,
    session: Res<velocity_game::game_mode::race_session::RaceSession>,
    ai: Option<Res<velocity_ai::plugin::AiState>>,
    mut w: MessageWriter<velocity_game::replays::cameras::replay_camera_manager::PlayerHumanControl>,
) {
    use velocity_game::game_mode::race_session::ESessionState as S;
    if *mode.get() != velocity_game::EGameMode::InGame || !matches!(session.state, S::Countdown | S::Go | S::Racing) {
        return;
    }
    let human = ai.as_ref().and_then(|a| a.world.as_ref()).and_then(|w| w.race.players.first()).is_some_and(|p| p.human_control);
    if human {
        w.write(velocity_game::replays::cameras::replay_camera_manager::PlayerHumanControl { player: 0, human: false });
    }
}

/// in the race: the current car the race was built from, one shot 8 s in
pub fn in_race(commands: &mut Commands, s: &mut Script, choice: Option<&velocity_game::game_mode::race_cars::PlayerCarChoice>) {
    s.c3.post_at = None;
    let i = (s.c3.events.max(1) - 1).min(3) as usize;
    if s.race_t.is_some_and(|t| s.t - t > 8.0) && !s.shots.contains(&RACE[i]) {
        log::info!("smoke-test: career3 race {} car {}", i + 1, bodyparts(choice.and_then(|c| c.setup.as_ref())));
        shot(commands, s, RACE[i]);
    }
}

/// the post race menu (CRaceOverMenu): its default entry (restart a failed event, continue, return to the FE)
pub fn post_race(commands: &mut Commands, s: &mut Script, ready: bool) {
    let now = s.t;
    if s.c3.post_at.is_none() {
        log::info!("smoke-test: career3 post race at {now:.1}");
    }
    let at = *s.c3.post_at.get_or_insert(now);
    let i = (s.c3.events.max(1) - 1).min(3) as usize;
    if now - at > 2.0 {
        shot(commands, s, POST[i]);
    }
    if ready && now - at > 3.0 && now - s.last > 4.0 {
        log::info!("smoke-test: career3 post race menu A at {now:.1}");
        press(s, key::A);
    }
}

/// the front end screens of the walk; false = not ours (profile select, plate, main menu stay with `run`)
pub fn fe(commands: &mut Commands, s: &mut Script, f: &Frontend, cur: u32, ready: bool, gm: &GameManager) -> bool {
    let settled = ready && s.t - s.screen.1 > 1.5;
    if cur == crc("RaceCompleteScreen") || cur == crc("TipsScreen") || cur == crc("LoadingScreen") {
        return true; // they leave by themselves
    }
    if cur == crc("DummyGameScreen") {
        if settled {
            let sel = f.dummy_game.lock().map(|d| d.selection).unwrap_or(0);
            if sel == 0 {
                press(s, key::DOWN);
            } else {
                log::info!("smoke-test: career3 dummy result {sel}");
                press(s, key::A);
            }
        }
        return true;
    }
    if cur == crc("CarCustomisationScreen") {
        garage(commands, s, f, settled);
        return true;
    }
    if cur != crc("HangoutScreen") {
        return false;
    }
    if s.c3.bought == 2 {
        s.c3.bought = 3;
        log::info!(
            "smoke-test: career3 back from the garage: cash {} (fe {}), slot {} {}",
            gm.vars.cash,
            f.fe.garage.cash,
            gm.vars.garage_slot,
            bodyparts(gm.vars.garage_setup(gm.vars.garage_slot as usize))
        );
    }
    let Ok(h) = f.hangout.lock() else { return true };
    let p = &h.progression;
    let o = f.fe.overlay.as_ref();
    let dialog = o.is_some_and(|o| o.dialog_active);
    let message = o.is_some_and(|o| o.message_time >= 0.0);
    if p.step != s.c3.step {
        s.c3.step = p.step;
        log::info!("smoke-test: career3 hangout step {} message {message} dialog {dialog} page {}", p.step, p.page);
    }
    // leaving (an event started, a character's screen picked)
    if !settled || !h.started || h.move_timer > 0.0 || h.exit_screen != 0 {
        return true;
    }
    let returned = s.c3.returned.min(4) as usize;
    if returned > 0 {
        shot(commands, s, BACK[returned - 1]);
    }
    if dialog {
        // the team invite (after-race step 4 / browser): NO is first, take YES
        if p.page == 8 {
            shot(commands, s, "c3_team_invite");
            if s.c3.dialog_step == 0 {
                log::info!("smoke-test: career3 team invite {:08x?}: DOWN, A (yes)", p.invite_team);
                press(s, key::DOWN);
                s.c3.dialog_step = 1;
            } else {
                press(s, key::A);
                s.c3.dialog_step = 0;
            }
        } else {
            log::warn!("smoke-test: career3 unexpected dialog, B");
            press(s, key::B);
        }
        return true;
    }
    // after-race steps still running (m_pEvent set) or Tory talking
    if f.fe.career.event.is_some() && !p.active || message {
        return true;
    }
    if s.c3.returned >= 3 {
        if s.done_at.is_none() {
            log::info!("smoke-test: career3 done: {}", summary(gm));
            shot(commands, s, "c3_done");
            let now = s.t;
            s.done_at = Some(now + 2.0);
        }
        return true;
    }
    if s.c3.returned == 1 && s.c3.bought == 0 {
        // the garage character, its menu, GARAGE_CUSTOMISE
        if p.active {
            press(s, key::B);
        } else if h.character == 0 {
            press(s, key::RIGHT);
        } else if h.character != 1 {
            press(s, key::LEFT);
        } else {
            press(s, key::A);
        }
        return true;
    }
    if s.c3.reload == 1 {
        return true;
    }
    // Tory's browser: the first open event that is not completed
    if h.character != 0 {
        press(s, if h.menu_open { key::B } else { key::LEFT });
        return true;
    }
    if !p.active {
        log::info!("smoke-test: career3 open Tory's browser: {}", summary(gm));
        press(s, key::A);
        return true;
    }
    let sel = o.map_or(0, |o| o.menu_selected);
    let c = &f.fe.career;
    let open = |tier: u8, cat: u32| (0..64u32).map_while(|i| c.nth(tier, cat, i, 0, false)).position(|e| e.flags & 3 == 0);
    let want = match p.page {
        3 => (0..5u8).position(|t| open(t + 1, c.category).is_some()).map(|t| t as i32),
        4 => open(p.tier as u8 + 1, c.category).map(|i| i as i32),
        _ => Some(sel),
    };
    match want {
        Some(w) if w > sel => press(s, key::DOWN),
        Some(w) if w < sel => press(s, key::UP),
        Some(_) => {
            if p.page == 5 {
                log::info!("smoke-test: career3 start event {:08x?} (category {} tier {})", p.browse, c.category, p.tier + 1);
            }
            press(s, key::A);
        }
        None => {
            log::warn!("smoke-test: career3 nothing open on page {} (category {} tier {}), back", p.page, c.category, p.tier + 1);
            press(s, key::B);
        }
    }
    true
}

/// CarCustomisationScreen: RIGHT A A RIGHT A RIGHT RIGHT A (buy dialog) A (yes) — the fecapture `custom` walk — then
/// B until the screen is left
fn garage(commands: &mut Commands, s: &mut Script, f: &Frontend, settled: bool) {
    const BUY: [usize; 9] = [key::RIGHT, key::A, key::A, key::RIGHT, key::A, key::RIGHT, key::RIGHT, key::A, key::A];
    if s.c3.bought == 0 {
        s.c3.bought = 1;
        s.c3.buy_step = 0;
        s.c3.cash_before = f.fe.garage.cash;
        log::info!(
            "smoke-test: career3 garage: cash {} slot {} vehicle {:08x}",
            f.fe.garage.cash,
            f.fe.garage.custom_slot,
            f.fe.garage.custom_vehicle
        );
    }
    if !settled || s.t - s.last < 1.0 {
        return;
    }
    if s.c3.bought == 1 && s.c3.buy_step < BUY.len() {
        if s.c3.buy_step == BUY.len() - 1 {
            shot(commands, s, "c3_garage_dialog");
        }
        press(s, BUY[s.c3.buy_step]);
        s.c3.buy_step += 1;
        return;
    }
    if s.c3.bought == 1 {
        let shown = f.fe.foyer_vehicles.slots.first().and_then(|v| v.setup.as_ref());
        log::info!(
            "smoke-test: career3 bought: cash {} -> {} shown {}",
            s.c3.cash_before,
            f.fe.garage.cash,
            bodyparts(shown)
        );
        shot(commands, s, "garage_buy");
        s.c3.bought = 2;
        return;
    }
    press(s, key::B);
}
