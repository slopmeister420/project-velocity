//! `gpRaceCompleteScreen` (`dword_75CF98`, 16 bytes; create 0x00204E40, Init 0x00204F70 in the idb's FECK2Object
//! range, next to src\Feck2\Screens\Results\FEDummyGameScreen.cpp): the screen the front end starts on after a
//! race. It has no elements of its own: the overlay over `optionsBG`; with a profile in use (`byte_6F9054`) Start
//! saves the career (0x001061D0, "Primary") and Update waits for the save layer's status (`XMem` 0x0030EF60) before
//! going back to the screen the race was started from (`dword_6F6FC8`, `Feck2::return_screen`).
//!
//! Game flow's side is not here: the race log, freeing a temporary event (`GameVars` 0x000C2EE0), the save itself
//! and the event's prestige (`dword_6F7244`) are core's `race_complete`, which starts the save flow; Update polls
//! core's `SaveFlow` (= XMem 0x0030EF60, lent through `Feck2.save_flow`) and shows its Saving / SaveComplete popups
//! and error questions on the overlay (0x00205BE0). Back to the hangout with the event not over = `DummyGameScreen`.

use std::sync::{Arc, Mutex};

use crate::feck2::feck2_manager::{Feck2, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::string_manager::string_manager::string_crc;
use velocity_game::save_game::save_vars::{status, SaveStatus};

/// `CFERaceCompleteScreen` (16 bytes)
pub struct CFERaceCompleteScreen {
    pub screen: ScreenId,    // +0
    pub layer: LayerId,      // +4
    pub loaded: bool,        // +8
    pub ready: bool,         // +9 leave on the next update
    pub dialog_answer: i32,  // +12 the XMem question's answer (0 / 1), -1 none
}

/// FECK2Object.cpp 0x00204E40 / 0x00204F70 / 0x00205130 — `FECK2CreateRaceCompleteScreen` (Init, AssignEvents)
pub fn create_race_complete_screen(fe: &mut Feck2) -> Arc<Mutex<CFERaceCompleteScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("RaceCompleteScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("RaceCompleteScreen_Layer"), ..Default::default() });
    let obj = Arc::new(Mutex::new(CFERaceCompleteScreen { screen, layer, loaded: false, ready: false, dialog_answer: -1 }));
    fe.map(layer, functor(&obj, rc_enter), ev::ENTER);
    fe.map(layer, functor(&obj, rc_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEAVE);
    fe.map(layer, functor(&obj, rc_start), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, rc_update), ev::UPDATE);
    fe.map(layer, functor(&obj, rc_select), ev::SELECT_PRESS);
    for e in [ev::CANCEL_PRESS, ev::LEFT_PRESS, ev::RIGHT_PRESS, ev::L1_PRESS, ev::R1_PRESS] {
        fe.map(layer, functor(&obj, |_, _, _| true), e);
    }
    obj
}

/// FECK2Object.cpp 0x00205590 — Enter
fn rc_enter(s: &mut CFERaceCompleteScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(mut o) = fe.overlay.take() {
        overlay::attach_to_screen(&mut o, fe, Some(s.screen));
        fe.overlay = Some(o);
    }
    s.loaded = true;
    true
}

/// FECK2Object.cpp 0x002055F0 — Activate: CONTINUE prompt over the options background, logo, an empty message
/// and four empty dialog options
fn rc_activate(s: &mut CFERaceCompleteScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(mut o) = fe.overlay.take() {
        overlay::reset(&mut o, fe);
        overlay::set_nav_text(&mut o, fe, 0, string_crc("GENERAL_CONTINUE"));
        overlay::show_borders(&mut o, fe);
        overlay::set_hide_options(&mut o, fe, true);
        overlay::set_2d_background(&mut o, fe, "optionsBG", r"\FE\LOADSCR.vcd", false, COLOUR_WHITE);
        fe.el(o.static_logo).set_visible(true); // 0x0027D220
        overlay::set_message(&mut o, fe, 0);
        overlay::show_dialog(&mut o, fe, 4, 0, &[0, 0, 0, 0]);
        overlay::set_message_timeout(&mut o, -1);
        overlay::enable(&mut o);
        fe.overlay = Some(o);
    }
    let _ = s;
    true
}

/// FECK2Object.cpp 0x00205830 — Start: message and dialog hidden; leave at once after an online race
/// (`byte_6F5D08`) or without a profile, else save the profile (0x001061D0, done by core) and wait
fn rc_start(s: &mut CFERaceCompleteScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.dialog_answer = -1;
    if let Some(mut o) = fe.overlay.take() {
        overlay::show_message(&mut o, fe, false);
        overlay::set_dialog_visible(&mut o, fe, false, 0);
        fe.overlay = Some(o);
    }
    if fe.career.quit_race {
        s.ready = true;
    } else if fe.career.save_enabled {
        s.ready = false;
    } else {
        s.ready = true;
    }
    // dword_6F71A0 += dword_6F7244 (non-temporary event): core's race_complete
    true
}

/// FECK2Object.cpp 0x00205900 — Update: back to the screen the race started from once ready, then poll the save
fn rc_update(s: &mut CFERaceCompleteScreen, fe: &mut Feck2, e: &CEvent) -> bool {
    let mut target = fe.return_screen;
    if s.ready {
        if !fe.career.quit_race {
            let unfinished = fe.career.event.as_ref().is_some_and(|e| e.flags & 0x10 == 0);
            let dummy = string_crc("DummyGameScreen");
            if target == string_crc("HangoutScreen") && unfinished && fe.find_screen(dummy).is_some() {
                target = dummy;
            }
        }
        fe.goto_previous_screen(0, target);
    }
    // dword_701404 = XMem 0x0030EF60 (core's `SaveFlow`, polled with the update's dt). STAND-IN: a save core did not
    // start (its view of quit / profile differed from Start's) arrives as `race_save_status` 0 = ready / -1 = failed,
    // so the screen cannot stall (the original's Start always starts the save it waits for)
    let st = match fe.save_flow.as_mut() {
        Some(f) if f.in_progress() => f.poll(e.dt()),
        _ => match fe.career.race_save_status.take() {
            Some(0) => {
                s.ready = true;
                return true;
            }
            Some(v) => SaveStatus { status: v, ..Default::default() },
            None => return true,
        },
    };
    fe.save_status = st.clone();
    match st.status + 2 {
        // -2 / -1 failed / user exit: byte_6F7165 = 0 (profile select's memory unit flag, not kept), nav icon 1, back
        // one screen
        0 | 1 => {
            if let Some(mut o) = fe.overlay.take() {
                overlay::show_nav(&mut o, fe, 1, true); // 0x0027AD30
                fe.overlay = Some(o);
            }
            fe.goto_previous_screen(1, 0);
        }
        // 1 question: Select's answer (`+12`) goes back to the flow (0x0030EF50) once, then the dialog as for 2
        3 => {
            if s.dialog_answer != -1 {
                if s.dialog_answer == 0 {
                    respond(fe, st.responses[0]);
                }
                if s.dialog_answer == 1 {
                    respond(fe, st.responses[1]);
                }
                s.dialog_answer = -1;
            }
            show_save_status(fe, &st);
        }
        // 2 popup
        4 => show_save_status(fe, &st),
        // 3 done
        5 => s.ready = true,
        // 4: answer +32 (no node returns it)
        6 => respond(fe, st.responses[0]),
        _ => {}
    }
    true
}

/// XMem.cpp 0x0030EF50
fn respond(fe: &mut Feck2, crc: u32) {
    if let Some(f) = fe.save_flow.as_mut() {
        f.respond(crc);
    }
}

/// FECK2Object.cpp 0x00205BE0 — the status block on the overlay: with options (`(+4 & 0x70) >> 4`) a dialog (title =
/// the text +12, options +16.., selection `(+4 & 0xE00) >> 9`) unless one is up, else the message +12
fn show_save_status(fe: &mut Feck2, st: &SaveStatus) {
    let Some(mut o) = fe.overlay.take() else { return };
    let n = (st.flags & 0x70) >> 4;
    if n != 0 {
        if !o.dialog_active {
            overlay::show_message(&mut o, fe, false);
            overlay::set_dialog_count(&mut o, fe, n);
            overlay::set_dialog_strings(&mut o, fe, st.text, &st.options);
            overlay::set_dialog_visible(&mut o, fe, true, ((st.flags & 0xE00) >> 9) as i32);
        }
    } else {
        if o.dialog_active {
            overlay::set_dialog_visible(&mut o, fe, false, 0);
        }
        overlay::set_message_string(&mut o, fe, st.text);
        overlay::show_message(&mut o, fe, true);
    }
    fe.overlay = Some(o);
}

/// FECK2Object.cpp 0x00205B00 — Select: while the last status was a question (1) its answer is the overlay dialog's
/// selection (`+244`)
fn rc_select(s: &mut CFERaceCompleteScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if fe.save_status.status == status::QUESTION {
        if let Some(o) = fe.overlay.as_ref() {
            s.dialog_answer = o.dialog_selection;
        }
    }
    true
}
