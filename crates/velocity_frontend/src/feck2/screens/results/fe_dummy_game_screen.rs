//! `gpDummyGameScreen` (`dword_75D510`, 0x48 bytes): src\Feck2\Screens\Results\FEDummyGameScreen.cpp (create 0x00207E20,
//! Init 0x00207F50, AssignEvents 0x00208560 — shifted into the idb's FECK2Object range; the rest 0x00208D60..0x002096D0).
//! The between-races screen of a career event: RaceCompleteScreen goes here instead of the hangout while the event
//! is not over (flag 0x10), and the hangout does when the event's race has no record ("not in build"). Up / down pick
//! "play race" or a finish position, left / right held change the prestige gained by 10, Select either races (the
//! loading screen) or fills in the result (core's event progression, then RaceCompleteScreen).
//! IDA dumps: `research/fe_dummy.c`, `research/fe_dummy2.c`.

use std::sync::{Arc, Mutex};

use crate::feck2::feck2_manager::{visos_colour, Feck2, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::hangout::CareerRequest;
use crate::string_manager::string_manager::string_crc;

/// `dword_75D798` (static init 0x00429D80): the rows
const ROWS: [&str; 7] = [
    "DUMMYGAME_PLAYRACE",
    "DUMMYGAME_1ST",
    "DUMMYGAME_2ND",
    "DUMMYGAME_3RD",
    "DUMMYGAME_4TH",
    "DUMMYGAME_5TH",
    "DUMMYGAME_6TH",
];

/// `CFEDummyGameScreen` (0x48 bytes)
pub struct CFEDummyGameScreen {
    pub screen: ScreenId,     // +0
    pub layer: LayerId,       // +4
    pub indicator: ElementId, // +8 "MenuIndicatorSprite"
    pub select: ElementId,    // +12 DUMMYGAME_SELECT_RESULT
    pub rows: [ElementId; 7], // +16
    pub event: ElementId,     // +44 CURRENT_EVENT
    pub race: ElementId,      // +48 CURRENT_RACE
    pub prestige_text: ElementId, // +52 PRESTIGE_GAINED
    pub loaded: bool,         // +56
    pub valid: bool,          // +57 the race exists in this build
    pub selection: i32,       // +60 0 play the race, n finish n-th
    pub competitors: i32,     // +64
    pub prestige: u32,        // +68
}

/// FECK2Object.cpp 0x00207E20 / 0x00207F50 / 0x00208560 / 0x002095A0 — `FECK2CreateDummyGameScreen` + Init, AssignEvents (+ cancel functor)
pub fn create_dummy_game_screen(fe: &mut Feck2) -> Arc<Mutex<CFEDummyGameScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("DummyGameScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("DummyGame_Layer"), ..Default::default() });
    let mut d = SElementDesc::default();
    d.kind = ELEMENT_2D;
    d.crc = string_crc("MenuIndicatorSprite");
    d.position[0] = 75.0;
    d.position[1] = 240.0;
    d.texture = "FE_Arrow_16".into();
    d.vcd = r"\FE\FE.vcd".into();
    let indicator = fe.create_element(layer, &d);
    let text = |fe: &mut Feck2, d: &mut SElementDesc, font: u32, name: &str, x: f32, y: f32, s: u32| {
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = font;
        d.crc = string_crc(name);
        d.position[0] = x;
        d.position[1] = y;
        d.string = s;
        fe.create_element(layer, d)
    };
    let event = text(fe, &mut d, 3, "DummyGameEvent", 60.0, 100.0, string_crc("CURRENT_EVENT"));
    let race = text(fe, &mut d, 2, "DummyGameRace", 60.0, 140.0, string_crc("CURRENT_RACE"));
    let prestige_text = text(fe, &mut d, 1, "DummyGameRace", 60.0, 170.0, string_crc("PRESTIGE_GAINED"));
    let select = text(fe, &mut d, 2, "DummyGametext", 80.0, 210.0, string_crc("DUMMYGAME_SELECT_RESULT"));
    let mut rows = [0; 7];
    for (i, r) in rows.iter_mut().enumerate() {
        *r = text(fe, &mut d, 2, "DummyGametext", 100.0, (30 * i as u32 + 240) as f32, string_crc(ROWS[i]));
    }
    let obj = Arc::new(Mutex::new(CFEDummyGameScreen {
        screen,
        layer,
        indicator,
        select,
        rows,
        event,
        race,
        prestige_text,
        loaded: false,
        valid: false,
        selection: 0,
        competitors: 0,
        prestige: 0,
    }));
    // 0x00208560 — AssignEvents
    fe.map(layer, functor(&obj, dg_enter), ev::ENTER);
    fe.map(layer, functor(&obj, dg_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::LEAVE);
    fe.map(layer, functor(&obj, dg_start), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UPDATE);
    fe.map(layer, functor(&obj, |s, fe, _| { move_selection(s, fe, -1); false }), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| { move_selection(s, fe, 1); false }), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, dg_less_prestige), ev::LEFT_HELD);
    fe.map(layer, functor(&obj, dg_more_prestige), ev::RIGHT_HELD);
    fe.map(layer, functor(&obj, dg_select), ev::SELECT_PRESS);
    // 0x002095A0
    fe.map(layer, functor(&obj, |_, fe, _| { fe.goto_previous_screen(1, 0); false }), ev::CANCEL_PRESS);
    obj
}

/// `VISOS_ColourXBOX(0x80, 0x78, 0x4D, 0x0E)`: the indicator (0x00209730 with `dword_77FD9C`'s colour unused)
fn indicator_colour() -> u32 {
    visos_colour(0x80, 0x78, 0x4D, 0x0E)
}

/// FECK2Object.cpp 0x002089C0 — Enter: overlay attached; once: the prestige text orange; rows up to the race's
/// competitor count shown
fn dg_enter(s: &mut CFEDummyGameScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(mut o) = fe.overlay.take() {
        overlay::attach_to_screen(&mut o, fe, Some(s.screen));
        fe.overlay = Some(o);
    }
    if !s.loaded {
        s.loaded = true;
        // element vtable +12 = Load
        for el in [s.indicator, s.select].into_iter().chain(s.rows).chain([s.event, s.race, s.prestige_text]) {
            fe.element_load(el);
        }
        fe.text(s.prestige_text).set_colour(visos_colour(0x80, 0xFF, 0x6E, 0));
        let c = indicator_colour();
        fe.e2d(s.indicator).set_colour(c, false, COLOUR_WHITE);
    }
    s.competitors = current_race(fe).map_or(0, |r| r.competitors as i32);
    fe.el(s.select).set_visible(true);
    for i in 0..7 {
        fe.el(s.rows[i]).set_visible((i as i32) < s.competitors + 1);
    }
    true
}

/// FECK2Object.cpp 0x00208BA0 — Activate: SELECT / BACK, options background, logo, left/right arrows, the entrants
/// to the debug log
fn dg_activate(s: &mut CFEDummyGameScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(mut o) = fe.overlay.take() {
        overlay::reset(&mut o, fe);
        overlay::set_nav_text(&mut o, fe, 0, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(&mut o, fe, 1, string_crc("GENERAL_BACK"));
        overlay::set_2d_background(&mut o, fe, "optionsBG", r"\FE\LOADSCR.vcd", false, COLOUR_WHITE);
        fe.el(o.static_logo).set_visible(true); // 0x0027D220
        overlay::show_arrows(&mut o, fe, 0);
        overlay::position_arrows(&mut o, fe, 30.0, 260.0, 160.0, 0);
        overlay::enable(&mut o);
        fe.overlay = Some(o);
    }
    let _ = s;
    log::debug!("========= Race Entrants =========");
    for e in current_race(fe).map(|r| r.entrants.clone()).unwrap_or_default() {
        match e {
            Some(c) if c != 0 => log::debug!("{c:#010x}"),
            _ => log::debug!("Player"),
        }
    }
    true
}

fn current_race(fe: &Feck2) -> Option<&crate::feck2::screens::hangout::CurrentRaceView> {
    fe.career.event.as_ref()?.current_race.as_ref()
}

fn set_arg(fe: &mut Feck2, el: ElementId, arg: &str, v: u32) {
    let h = fe.text(el).string.unwrap_or(0);
    fe.strings.set_argument(h, string_crc(arg), v).ok();
}

/// FEDummyGameScreen.cpp 0x00208D60 — Start: event / race names, "not in build" when the race has no record
fn dg_start(s: &mut CFEDummyGameScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    fe.results_prompt = false; // CFrontendManager +113
    s.valid = true;
    let Some(ev) = fe.career.event.clone() else {
        log::error!("ERROR: EVENT not found");
        log::error!("assert: 0 (FEDummyGameScreen.cpp 342)");
        // the original then reads the null event (+0x10)
        return false;
    };
    set_arg(fe, s.event, "idCurrentEvent", ev.crc);
    let details = match &ev.current_race {
        None => {
            log::error!("ERROR: Event CURRENT RACE not found");
            s.valid = false;
            None
        }
        Some(r) => {
            if r.details.is_none() {
                log::error!("ERROR: Event RACE DETAILS not found");
                s.valid = false;
            }
            r.details
        }
    };
    if s.valid {
        fe.text(s.rows[0]).set_colour(COLOUR_WHITE);
        fe.text(s.race).set_string(string_crc("CURRENT_RACE"));
        set_arg(fe, s.race, "idCurrentRace", details.map_or(0, |d| d.0));
    } else {
        let t: Vec<u16> = "RACE CURRENTLY NOT AVAILABLE IN BUILD".encode_utf16().collect();
        fe.text(s.race).set_text(&t);
        fe.text(s.rows[0]).set_colour(visos_colour(0x80, 0x44, 0x44, 0x44));
    }
    s.prestige = 0;
    set_arg(fe, s.prestige_text, "dPrestige", s.prestige);
    false
}

/// FEDummyGameScreen.cpp 0x002095D0 — move the selection (wraps over `competitors + 1` rows)
fn move_selection(s: &mut CFEDummyGameScreen, fe: &mut Feck2, d: i32) {
    s.selection += d;
    if s.selection >= 0 {
        if s.selection == s.competitors + 1 {
            s.selection = 0;
        }
    } else {
        s.selection = s.competitors;
    }
    fe.element_set_position(s.indicator, 75.0, (30 * s.selection + 236) as f32, 0.0);
    let c = indicator_colour();
    fe.e2d(s.indicator).set_colour(c, false, COLOUR_WHITE);
}

/// FEDummyGameScreen.cpp 0x00209180 — left held: prestige - 10 (not below 0)
fn dg_less_prestige(s: &mut CFEDummyGameScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.prestige >= 10 {
        s.prestige -= 10;
    }
    set_arg(fe, s.prestige_text, "dPrestige", s.prestige);
    false
}

/// FEDummyGameScreen.cpp 0x00209210 — right held: prestige + 10
fn dg_more_prestige(s: &mut CFEDummyGameScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.prestige = s.prestige.wrapping_add(10);
    set_arg(fe, s.prestige_text, "dPrestige", s.prestige);
    false
}

/// FEDummyGameScreen.cpp 0x002092A0 — the race record's class into the quick race globals; false without a record
pub fn set_race_class(fe: &mut Feck2) -> bool {
    let Some((_, pro)) = current_race(fe).and_then(|r| r.details) else { return false };
    fe.quick_race.pro = pro;
    fe.quick_race.amateur = !pro;
    true
}

/// FEDummyGameScreen.cpp 0x00209340 — Select: a position (or a race nobody human sits in) is the result: core fills
/// it in and runs the event's progression (`CareerRequest::DummyResult`), then RaceCompleteScreen; "play race" on a
/// race in this build goes to the loading screen
fn dg_select(s: &mut CFEDummyGameScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let humans = current_race(fe).map_or(0, |r| r.humans);
    if s.selection != 0 || humans == 0 {
        fe.career.requests.push(CareerRequest::DummyResult { position: s.selection as u32, prestige: s.prestige });
        fe.career.quit_race = false;
        fe.goto_screen(string_crc("RaceCompleteScreen"));
    } else if s.valid {
        set_race_class(fe);
        fe.career.continue_pending = true;
        fe.goto_screen(string_crc("LoadingScreen"));
    }
    false
}
