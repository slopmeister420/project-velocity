//! src\Feck2\Screens\Showroom\FECarShowroomScreen.cpp object region — `gpQuickRaceStartScreen` (create
//! 0x002203A0): the race summary before loading. Select builds the event and leaves for the loading screen.

use std::sync::{Arc, Mutex};

use crate::feck2::feck2_manager::{Feck2, COLOUR_GREY, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::string_manager::string_manager::string_crc;

/// `dword_75F968` / `dword_75F970` / `dword_75F978` / `dword_75F98C` / `dword_75F9A0` (inits 0x0042BC23..)
const CAR_INFO: [&str; 2] = ["PLAYERINFO_PLAYERNAME", "PLAYERINFO_PLAYERCAR"];
const WEATHER: [&str; 2] = ["QUICKRACE_WEATHER_FAIR", "QUICKRACE_WEATHER_RAIN"];
const DIFFICULTY: [&str; 3] = ["QUICKRACE_DIFFICULTY_EASY", "QUICKRACE_DIFFICULTY_NORMAL", "QUICKRACE_DIFFICULTY_HARD"];
const AREAS: [&str; 5] = ["LEVEL_AREA_UNKNOWN", "LEVEL_AREA_LA", "LEVEL_AREA_LONDON", "LEVEL_AREA_TOKYO", "LEVEL_AREA_MISC"];
const RACE_TYPES: [&str; 7] = [
    "RACECATEGORY_UNKNOWN", "RACECATEGORY_CRUISE", "RACECATEGORY_SPRINT", "RACECATEGORY_CIRCUIT",
    "RACECATEGORY_RELAY", "RACECATEGORY_DRAG", "RACECATEGORY_PURSUIT",
];

/// `CFEQuickRaceStartScreen` (1100 bytes)
pub struct CFEQuickRaceStartScreen {
    pub screen: ScreenId,             // +0
    pub layer: LayerId,               // +4
    pub info_background: ElementId,   // +8
    pub race_info: ElementId,         // +12
    pub options_info: ElementId,      // +16
    pub single_line: ElementId,       // +28
    pub highlighter: ElementId,       // +32
    pub car_labels: [ElementId; 2],   // +36
    pub player_lines: [ElementId; 2], // +44
    pub player_info: [[ElementId; 2]; 2], // +52
    pub exit: u32,                    // +1092 0 stay, 1 back, else screen crc
    pub loaded: bool,                 // +1096
}

/// FECarShowroomScreen.cpp 0x002203A0 / 0x00221CE0 / 0x00221DE0 / 0x00221EA0 — `FECK2CreateQuickRaceStartScreen` (+ functors)
pub fn create_quick_race_start_screen(fe: &mut Feck2) -> Arc<Mutex<CFEQuickRaceStartScreen>> {
    let obj = Arc::new(Mutex::new(CFEQuickRaceStartScreen {
        screen: 0,
        layer: 0,
        info_background: 0,
        race_info: 0,
        options_info: 0,
        single_line: 0,
        highlighter: 0,
        car_labels: [0; 2],
        player_lines: [0; 2],
        player_info: [[0; 2]; 2],
        exit: 0,
        loaded: false,
    }));
    // FECarShowroomScreen.cpp 0x002204E0 — Init
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("QuickRaceStartScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 2, crc: string_crc("QuickRaceStart_Layer"), ..Default::default() });
    let mut s = obj.lock().unwrap();
    s.screen = screen;
    s.layer = layer;
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, crc: string_crc("QuickRaceStart_TrackInfo"), font: 2, ..Default::default() };
    d.position[0] = f32::from_bits(1109917696);
    d.position[1] = f32::from_bits(1119748096);
    d.string = string_crc("QUICKRACE_RACEINFO_RACE");
    s.race_info = fe.create_element(layer, &d);
    d.font = 1;
    d.position[0] = f32::from_bits(1109917696);
    d.position[1] = f32::from_bits(1122369536);
    d.string = string_crc("QUICKRACE_RACEINFO_OPTIONS");
    s.options_info = fe.create_element(layer, &d);
    d.kind = ELEMENT_2D;
    d.crc = string_crc("QuickRaceStart_InfoBackground");
    d.texture = "FE_summary_title_highlight".into();
    d.vcd = "\\FE\\FE.vcd".into();
    d.position[0] = 0.0;
    d.position[1] = f32::from_bits(1119485952);
    d.f76 = f32::from_bits(1115684864);
    s.info_background = fe.create_element(layer, &d);
    for i in 0..2u32 {
        d.kind = ELEMENT_STATIC_TEXT;
        d.crc = string_crc("QuickRaceStart_CarInfo");
        d.font = 1;
        d.position[0] = (100 * i + 42) as f32;
        d.position[1] = f32::from_bits(1125056512);
        d.string = string_crc(CAR_INFO[i as usize]);
        s.car_labels[i as usize] = fe.create_element(layer, &d);
    }
    d.kind = ELEMENT_2D;
    d.crc = string_crc("BackgroundGraphic");
    d.texture = "FE_info_highlighter".into();
    d.vcd = "\\FE\\FE.vcd".into();
    d.position[0] = 0.0;
    d.position[1] = f32::from_bits(1124794368);
    d.f76 = f32::from_bits(1115684864);
    s.highlighter = fe.create_element(layer, &d);
    d.texture = "FE_summary_singleline".into();
    d.position[1] = f32::from_bits(1124597760);
    d.f76 = f32::from_bits(1115684864);
    s.single_line = fe.create_element(layer, &d);
    for row in 0..2u32 {
        for col in 0..2u32 {
            d.kind = ELEMENT_STATIC_TEXT;
            d.crc = string_crc("QuickRaceStart_CarInfo");
            d.font = 1;
            d.position[0] = (100 * col + 42) as f32;
            d.position[1] = (60 * row + 183) as f32;
            d.string = 0;
            s.player_info[row as usize][col as usize] = fe.create_element(layer, &d);
        }
        d.kind = ELEMENT_2D;
        d.crc = string_crc("BackgroundGraphic");
        d.texture = "FE_summary_singleline".into();
        d.vcd = "\\FE\\FE.vcd".into();
        d.position[0] = 0.0;
        d.position[1] = (60 * row + 176) as f32;
        d.f76 = f32::from_bits(1115684864);
        s.player_lines[row as usize] = fe.create_element(layer, &d);
        d.dynamic = false;
    }
    drop(s);
    // FECarShowroomScreen.cpp 0x00220CB0 — AssignEvents
    fe.map(layer, functor(&obj, qs_enter), ev::ENTER);
    fe.map(layer, functor(&obj, qs_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEAVE);
    fe.map(layer, functor(&obj, |s: &mut CFEQuickRaceStartScreen, _, _| {
        // 0x00221CE0
        s.exit = 0;
        true
    }), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, qs_update), ev::UPDATE);
    // 0x00221DE0..0x00221EA0: swallow the d-pad unless the overlay has a dialog up
    for e in [ev::UP_PRESS, ev::DOWN_PRESS, ev::LEFT_PRESS, ev::RIGHT_PRESS] {
        fe.map(layer, functor(&obj, |_, fe: &mut Feck2, _| !overlay_dialog_up(fe)), e);
    }
    fe.map(layer, functor(&obj, qs_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, qs_cancel), ev::CANCEL_PRESS);
    fe.map(layer, functor(&obj, qs_change_car), ev::OPTION1_PRESS);
    fe.map(layer, functor(&obj, qs_change_track), ev::OPTION2_PRESS);
    obj
}

/// FECarShowroomScreen.cpp 0x002211C0 — Enter: fill the summary arguments.
fn qs_enter(s: &mut CFEQuickRaceStartScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(mut o) = fe.overlay.take() {
        overlay::attach_to_screen(&mut o, fe, Some(s.screen));
        fe.overlay = Some(o);
    }
    // FE3DCamera: end any move, foyer "quickR", fov 50 (0x00271E80, 0x00272070, 0x00272140); the mode-2 timing
    // (0x002720D0(2, 0.25)) and the 5 s move duration (0x00272110 / 0x00272020) are not ported
    fe.foyer.snap_to_target();
    fe.foyer.set_foyer("quickR");
    fe.foyer.set_fov(50.0);
    // FEAlloyEditor_Basic 0x00259540 / 0x002595E0 / 0x00259D20: the spots are loaded again after FrontEndReInit
    if fe.foyer_vehicles.count == 0 {
        fe.foyer_vehicles.set_foyer("quickR");
        fe.foyer_vehicles.load("Vehicles", false);
        if let Some(db) = fe.vehicles.take() {
            let sel = [fe.quick_race.selected_vehicle, fe.quick_race.selected_vehicle];
            let garage = fe.quick_race.garage.clone();
            fe.foyer_vehicles.show(None, &db, sel, &garage, fe.quick_race.garage_selected as usize, &fe.garage);
            fe.vehicles = Some(db);
        }
    }
    // 0x00271D20 / 0x00271E00 (move): position and look-at entry 10 (dword_769280 / dword_769370)
    fe.foyer.move_to(10);
    // the white at 70% alpha for the lines
    let line = (COLOUR_WHITE & 0x00FF_FFFF) | (((((COLOUR_WHITE >> 24) as f32) * 0.7) as u32) << 24);
    if !s.loaded {
        fe.element_load(s.info_background);
        fe.element_load(s.race_info);
        fe.e2d(s.info_background).set_size(387, 39);
        fe.element_load(s.options_info);
        fe.element_load(s.highlighter);
        fe.element_load(s.single_line);
        fe.e2d(s.single_line).set_size(512, 32);
        fe.e2d(s.single_line).set_colour(line, false, COLOUR_WHITE);
        fe.e2d(s.highlighter).set_colour(COLOUR_GREY, false, COLOUR_WHITE);
        for i in 0..2 {
            fe.element_load(s.car_labels[i]);
        }
        for r in 0..2 {
            fe.element_load(s.player_lines[r]);
            fe.e2d(s.player_lines[r]).set_colour(line, false, COLOUR_WHITE);
            fe.e2d(s.player_lines[r]).set_size(512, 32);
            for c in 0..2 {
                fe.element_load(s.player_info[r][c]);
            }
        }
        s.loaded = true;
    }
    for id in [s.race_info, s.options_info, s.player_lines[0], s.player_info[0][0], s.player_info[0][1]] {
        fe.el(id).set_visible(true);
    }
    let two_players = fe.quick_race.two_players; // byte_6F7157
    fe.el(s.player_lines[1]).set_visible(two_players);
    for c in 0..2 {
        fe.el(s.player_info[1][c]).set_visible(two_players);
    }
    let q = fe.quick_race.clone();
    let area = q.track.map(|t| fe.events.tracks[t].area as usize).unwrap_or(0).min(4);
    let race = q.races[0].map(|r| fe.events.races[r].display_crc).unwrap_or(0);
    let info = string_crc("QUICKRACE_RACEINFO_RACE");
    fe.strings.set_argument(info, string_crc("idRaceArea"), string_crc(AREAS[area])).ok();
    fe.strings.set_argument(info, string_crc("idRaceType"), string_crc(RACE_TYPES[(q.race_type as usize).min(6)])).ok();
    fe.strings.set_argument(info, string_crc("idRaceName"), race).ok();
    let laps = if q.laps == 1 { "QUICKRACE_LAPS_SINGLE" } else { "QUICKRACE_LAPS_MULTIPLE" };
    let opps = if q.opponents == 1 { "QUICKRACE_OPPONENTS_SINGLE" } else { "QUICKRACE_OPPONENTS_MULTIPLE" };
    let opt = string_crc("QUICKRACE_RACEINFO_OPTIONS");
    fe.strings.set_argument(opt, string_crc("idOptionLaps"), string_crc(laps)).ok();
    fe.strings.set_argument(opt, string_crc("dLapsCount"), q.laps).ok();
    fe.strings.set_argument(opt, string_crc("idOptionOpponents"), string_crc(opps)).ok();
    fe.strings.set_argument(opt, string_crc("dOpponentsCount"), q.opponents).ok();
    fe.strings.set_argument(opt, string_crc("idOptionWeather"), string_crc(WEATHER[(q.weather as usize).min(1)])).ok();
    fe.strings.set_argument(opt, string_crc("idOptionDifficulty"), string_crc(DIFFICULTY[(q.difficulty as usize).min(2)])).ok();
    fe.text(s.player_info[0][0]).set_string(string_crc("GENERAL_PLAYER1"));
    // player car name: first dword of the player 1 vehicle setup (`dword_6F724C`), set by car select
    let car = fe.quick_race.car_name_crc;
    fe.text(s.player_info[0][1]).set_string(car);
    if fe.quick_race.two_players {
        fe.text(s.player_info[1][0]).set_string(string_crc("GENERAL_PLAYER2"));
        let car2 = fe.quick_race.car_name_crc2;
        fe.text(s.player_info[1][1]).set_string(car2);
    }
    // FE3DCharacters 0x00261220.. (hide the characters) and FE3DCamera +358: characters are created hidden
    true
}

/// FECarShowroomScreen.cpp 0x00221AD0 — Activate
fn qs_activate(s: &mut CFEQuickRaceStartScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    crate::feck2::screens::common::fe_3d_camera::show_hangout_and_track_area(fe);
    if let Some(mut o) = fe.overlay.take() {
        overlay::reset(&mut o, fe);
        overlay::set_nav_text(&mut o, fe, 0, string_crc("GENERAL_START"));
        let back = if fe.quick_race.skip_car_select { "GENERAL_MAINMENU" } else { "GENERAL_BACK" };
        overlay::set_nav_text(&mut o, fe, 1, string_crc(back));
        overlay::set_nav_text(&mut o, fe, 2, string_crc("GENERAL_CHANGECAR"));
        overlay::set_nav_text(&mut o, fe, 3, string_crc("GENERAL_CHANGETRACK"));
        overlay::show_arrows(&mut o, fe, 0);
        overlay::fade_arrows(&mut o, fe, -128, 0, 0);
        overlay::set_title(&mut o, fe, string_crc("MAINMENU_QUICKRACE"));
        overlay::show_borders(&mut o, fe);
        overlay::enable(&mut o);
        fe.overlay = Some(o);
    }
    let _ = s;
    true
}

/// FECarShowroomScreen.cpp 0x00221D30 — Update: once every element has faded out, leave. `exit` is only
/// cleared by Start, so the goto is re-posted each frame until the screen changes (as in the original).
fn qs_update(s: &mut CFEQuickRaceStartScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.exit != 0 {
        if fe.layer_fade_all(s.layer, -5, 0) || fe.online_session {
            if s.exit == 1 {
                fe.goto_previous_screen(1, 0);
            } else {
                let e = s.exit;
                fe.goto_screen(e);
            }
        } else if let Some(mut o) = fe.overlay.take() {
            overlay::fade_arrows(&mut o, fe, -5, 0, 0);
            fe.overlay = Some(o);
        }
    }
    // else: FE3DVehicles "allow spin" flag (dword_768978+15692), 3D foyer not ported
    true
}

fn overlay_dialog_up(fe: &Feck2) -> bool {
    // FEOnlinePlayerOptionsScreen__20B120(gpCommon2DOverlay): the overlay's popup dialog, not ported
    let _ = fe;
    false
}

/// FECarShowroomScreen.cpp 0x00222160 — Option1: back to car select.
fn qs_change_car(_s: &mut CFEQuickRaceStartScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if overlay_dialog_up(fe) {
        return false;
    }
    fe.quick_race.skip_car_select = true;
    fe.goto_previous_screen(1, string_crc("QuickRaceCarSelectScreen"));
    true
}

/// FECarShowroomScreen.cpp 0x002221F0 — Option2: back to the track/event screen.
fn qs_change_track(_s: &mut CFEQuickRaceStartScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if overlay_dialog_up(fe) {
        return false;
    }
    fe.quick_race.skip_car_select = true;
    fe.goto_previous_screen(1, string_crc("QuickRaceTextScreen"));
    true
}

/// FECarShowroomScreen.cpp 0x00221EE0 — Select: build the event, set the post race screen, go to the loading
/// screen (which starts the game). The event itself (`EventDatabaseEvents` 0x00093630 single race / 0x00093920
/// tournament, stored in `dword_6F9BE8`) is built by game flow on `FrontendRequest::StartQuickRace` from
/// `Feck2::quick_race`.
fn qs_select(s: &mut CFEQuickRaceStartScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    fe.quick_race.skip_car_select = true;
    s.exit = string_crc("LoadingScreen");
    fe.post_race_screen = string_crc("RaceCompleteScreen"); // GameManager 0x000BE7C0
    fe.return_screen = string_crc("QuickRaceStartScreen"); // dword_6F6FC8
    true
}

/// FECarShowroomScreen.cpp 0x002220D0 — Cancel
fn qs_cancel(_s: &mut CFEQuickRaceStartScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if fe.quick_race.skip_car_select {
        fe.goto_previous_screen(1, string_crc("MainMenuScreen"));
    } else {
        fe.goto_previous_screen(1, 0);
    }
    true
}
