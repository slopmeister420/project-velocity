//! src\Feck2\Screens\QuickRace\FEQuickRaceTextScreen.cpp — `gpQuickRaceTextScreen`: the quick race event
//! setup (category, area, single/tournament, race type, races; laps, opponents, difficulty, weather; online
//! match criteria). The object code sits at 0x00226390..0x0022B470 (`objregions` labels part of it
//! FEQuickRaceCarSelectScreen.cpp).
//!
//! the 3D track previews (`FE3DCamera` screens 0x002274E0 / 0x00227680) and the online match hosting paths are
//! not ported; online stays off (`byte_6F7158` / `byte_6F715A` / `byte_6F90BF` = 0).

use std::sync::{Arc, Mutex};

use velocity_game::event_database::event_database_track_loading::{CEventDatabase, ERaceType};

use crate::feck2::feck2_manager::{visos_colour, Feck2, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::string_manager::string_manager::string_crc;

/// `dword_6C12F4..6C1320`: column x and section y positions.
const TITLE_X: [f32; 3] = [43.0, 43.0, 43.0];
const LABEL_X: [f32; 3] = [43.0, 43.0, 43.0];
const VALUE_X: [f32; 3] = [128.0, 128.0, 145.0];
const SECTION_Y: [f32; 3] = [98.0, 229.0, 312.0];

/// static string tables (initialisers 0x0042C283..0x0042CAAF, 0x004303CE..).
const MAIN_OPTIONS: [&str; 3] = ["QUICKRACE_MAINOPTION_TRACK", "QUICKRACE_MAINOPTION_OPTIONS", "QUICKRACE_MAINOPTION_ONLINE"];
const TRACK_LABELS: [&str; 7] = [
    "QUICKRACE_SELECTCATEGORY", "QUICKRACE_SELECTAREA", "QUICKRACE_SELECTEVENT", "QUICKRACE_SELECTTYPE",
    "QUICKRACE_SELECTRACE", "GENERAL_BLANK", "GENERAL_BLANK",
];
const OPTION_LABELS: [&str; 4] = ["QUICKRACE_LAPS", "QUICKRACE_OPPONENTS", "QUICKRACE_DIFFICULTY", "QUICKRACE_WEATHER"];
const ONLINE_LABELS: [&str; 4] = [
    "ONLINE_MATCHCRITERIA_COLLISIONS", "ONLINE_MATCHCRITERIA_FRIENDSSLOTS", "ONLINE_MATCHCRITERIA_RANKRANGE",
    "ONLINE_MATCHCRITERIA_VEHICLE_TIERS",
];
/// `dword_75FF7C`: info rows 1..5 (row 0 never created).
const INFO_STRINGS: [&str; 6] = ["", "QUICKRACE_BESTLAPTIME", "QUICKRACE_BESTPRESTIGE", "", "", ""];
const WEATHER: [&str; 2] = ["QUICKRACE_WEATHER_FAIR", "QUICKRACE_WEATHER_RAIN"];
const DIFFICULTY: [&str; 3] = ["QUICKRACE_DIFFICULTY_EASY", "QUICKRACE_DIFFICULTY_NORMAL", "QUICKRACE_DIFFICULTY_HARD"];
const CATEGORY: [&str; 2] = ["RACETYPE_UNDERGROUND", "RACETYPE_PROFESSIONAL"];
const AREAS: [&str; 5] = ["LEVEL_AREA_UNKNOWN", "LEVEL_AREA_LA", "LEVEL_AREA_LONDON", "LEVEL_AREA_TOKYO", "LEVEL_AREA_MISC"];
const RACE_TYPES: [&str; 7] = [
    "RACECATEGORY_UNKNOWN", "RACECATEGORY_CRUISE", "RACECATEGORY_SPRINT", "RACECATEGORY_CIRCUIT",
    "RACECATEGORY_RELAY", "RACECATEGORY_DRAG", "RACECATEGORY_PURSUIT",
];
const RANK_RANGE: [&str; 7] = [
    "ONLINE_MATCHCRITERIA_RANKRANGE_ANY", "ONLINE_MATCHCRITERIA_RANKRANGE_SAME", "ONLINE_MATCHCRITERIA_RANKRANGE_SAME_OR_LOWER",
    "ONLINE_MATCHCRITERIA_RANKRANGE_SAME_OR_HIGHER", "ONLINE_MATCHCRITERIA_RANKRANGE_2", "ONLINE_MATCHCRITERIA_RANKRANGE_4",
    "ONLINE_MATCHCRITERIA_RANKRANGE_6",
];
const VEHICLE_TIERS: [&str; 7] = [
    "ONLINE_MATCHCRITERIA_VEHICLE_TIER_ANY", "VEHICLE_TIER1", "VEHICLE_TIER2", "VEHICLE_TIER3", "VEHICLE_TIER4",
    "VEHICLE_TIER5", "VEHICLE_TIER6",
];

/// race flag "selectable" (`CEventRace+0x25 & 2`)
const RACE_SELECTABLE: u8 = 2;

/// the quick race selections kept between screens (`GameVars` globals 0x006F70E8..0x006F7169).
#[derive(Debug, Clone, Default)]
pub struct QuickRaceSettings {
    pub amateur: bool,               // byte_6F70E8 (DummyGameScreen 0x002092A0: !pro)
    pub pro: bool,                   // byte_6F70E9
    pub laps: u32,                   // dword_6F7134
    pub opponents: u32,              // dword_6F713C
    pub race_type: u32,              // dword_6F7140
    pub track: Option<usize>,        // dword_6F7144
    pub races: [Option<usize>; 3],   // dword_6F7148..6F7150
    pub weather: u32,                // byte_6F7154
    pub drag: bool,                  // byte_6F7155
    pub single: bool,                // byte_6F7156 (true = single race, false = tournament)
    pub difficulty: u32,             // dword_6F70EC
    pub selected_race_id: u32,       // dword_6F9BE0
    pub online: bool,                // byte_6F7158
    pub peer_to_peer: bool,          // byte_6F7159 (MultiplayerScreen option 1, debug transmission mode)
    pub skip_car_select: bool,       // byte_6F7131
    pub two_players: bool,           // byte_6F7157
    pub car_name_crc: u32,           // *dword_6F724C player 1 vehicle setup name string
    pub car_name_crc2: u32,          // *dword_6F7478 player 2's
    pub player_setups: [Option<velocity_physics::vehicle_database::vehicle_database::VehicleSetup>; 2], // dword_6F724C / dword_6F7478
    pub vehicle_tier: u32,           // dword_6F7138 (tier of the chosen car, passed to the event builder)
    pub automatic: bool,             // dword_768978+2220
    pub selected_vehicle: u32,       // dword_6F70DC
    pub garage: Vec<u32>,            // dword_6F71A4 count, dword_6F71A8 hashes (profile garage, empty without a profile)
    pub garage_selected: u32,        // dword_6F71C4
    pub progress: u32,               // career progress for vehicle locks (profile)
    pub all_unlocked: bool,          // byte_6877F0
}

// ---- EventDatabaseTrackLoading queries the screen uses (core owns the database; these walk it in list order) ----

/// EventDatabaseTrackLoading.cpp 0x00094DA0 — races of a type on a track in a category (0 = any) with a flag.
fn race_count(db: &CEventDatabase, track: usize, category: u32, race_type: u32, flag: u8) -> u32 {
    db.tracks[track].races[race_type as usize]
        .iter()
        .filter(|&&r| (db.races[r].category == category || category == 0) && flag & db.races[r].flags != 0)
        .count() as u32
}

/// EventDatabaseTrackLoading.cpp 0x00094E80 — the n-th such race.
fn nth_race(db: &CEventDatabase, track: usize, category: u32, race_type: u32, flag: u8, n: u32) -> Option<usize> {
    let mut i = 0;
    for &r in &db.tracks[track].races[race_type as usize] {
        if (db.races[r].category == category || category == 0) && flag & db.races[r].flags != 0 {
            if i == n {
                return Some(r);
            }
            i += 1;
        }
    }
    log::warn!("Unable to find Nth race of given type");
    None
}

/// EventDatabaseTrackLoading.cpp 0x00094D20 — any race with the flag.
fn track_has_races(db: &CEventDatabase, track: usize, flag: u8) -> bool {
    (0..7).map(|t| race_count(db, track, 0, t, flag)).sum::<u32>() != 0
}

/// EventDatabaseTrackLoading.cpp 0x00094B00 — tracks in an area (5 = all).
fn track_count(db: &CEventDatabase, area: u32, flag: u8) -> u32 {
    if area == 5 {
        return db.tracks.len() as u32;
    }
    (0..db.tracks.len()).filter(|&t| db.tracks[t].area == area && track_has_races(db, t, flag)).count() as u32
}

/// EventDatabaseTrackLoading.cpp 0x00095110 — first track in an area.
fn first_track(db: &CEventDatabase, area: u32, flag: u8) -> Option<usize> {
    if area == 5 && flag == 0 {
        return if db.tracks.is_empty() { None } else { Some(0) };
    }
    (0..db.tracks.len()).find(|&t| (db.tracks[t].area == area || area == 5) && track_has_races(db, t, flag))
}

/// EventDatabaseTrackLoading.cpp 0x000951F0 — last track in an area.
fn last_track(db: &CEventDatabase, area: u32, flag: u8) -> Option<usize> {
    if area == 5 && flag == 0 {
        return db.tracks.len().checked_sub(1);
    }
    (0..db.tracks.len()).rev().find(|&t| (db.tracks[t].area == area || area == 5) && track_has_races(db, t, flag))
}

/// EventDatabaseTrackLoading.cpp 0x000952D0 — next track in an area, wrapping.
fn next_track(db: &CEventDatabase, track: usize, area: u32, flag: u8) -> Option<usize> {
    let mut t = track + 1;
    while t < db.tracks.len() {
        if t == track {
            return Some(track);
        }
        if (db.tracks[t].area == area || area == 5) && track_has_races(db, t, flag) {
            return Some(t);
        }
        t += 1;
    }
    first_track(db, area, flag)
}

/// EventDatabaseTrackLoading.cpp 0x000953C0 — previous track in an area, wrapping.
fn prev_track(db: &CEventDatabase, track: usize, area: u32, flag: u8) -> Option<usize> {
    let mut t = match track.checked_sub(1) {
        Some(t) => t,
        None => return last_track(db, area, flag),
    };
    loop {
        if t == track {
            return Some(track);
        }
        if (db.tracks[t].area == area || area == 5) && track_has_races(db, t, flag) {
            return Some(t);
        }
        match t.checked_sub(1) {
            Some(n) => t = n,
            None => return last_track(db, area, flag),
        }
    }
}

/// EventDatabaseTrackLoading.cpp 0x00094BE0 — tracks of an area having races of a type in a category.
fn tracks_with_type(db: &CEventDatabase, category: u32, area: u32, race_type: u32, flag: u8) -> u32 {
    (0..db.tracks.len()).filter(|&t| db.tracks[t].area == area && race_count(db, t, category, race_type, flag) != 0).count() as u32
}

/// EventDatabaseTrackLoading.cpp 0x00094CB0 / 0x00094CE0
fn next_type(t: u32) -> u32 {
    if t + 1 >= 7 { 0 } else { t + 1 }
}
fn prev_type(t: u32) -> u32 {
    if t == 0 { 6 } else { t - 1 }
}

/// next race in its track/type list (`CEventRace+0x38`), previous (`+0x3C`).
fn race_next(db: &CEventDatabase, r: usize) -> Option<usize> {
    let race = &db.races[r];
    let list = &db.tracks[race.track].races[race.race_type as usize];
    list.iter().position(|&x| x == r).and_then(|i| list.get(i + 1).copied())
}
fn race_prev(db: &CEventDatabase, r: usize) -> Option<usize> {
    let race = &db.races[r];
    let list = &db.tracks[race.track].races[race.race_type as usize];
    list.iter().position(|&x| x == r).and_then(|i| i.checked_sub(1)).map(|i| list[i])
}

// ---- the screen ------------------------------------------------------------------------------------------

/// `CFEQuickRaceTextScreen` (1024 bytes)
pub struct CFEQuickRaceTextScreen {
    pub this: std::sync::Weak<Mutex<CFEQuickRaceTextScreen>>,
    pub screen: ScreenId,                 // +0
    pub layer: LayerId,                   // +4
    pub info_highlighter: ElementId,      // +8
    pub track_box: ElementId,             // +12
    pub stats_box: ElementId,             // +16
    pub titles: [ElementId; 3],           // +20
    pub track_values: [ElementId; 7],     // +32
    pub track_labels: [ElementId; 7],     // +60
    pub option_values: [ElementId; 4],    // +88
    pub option_labels: [ElementId; 4],    // +104
    pub online_values: [ElementId; 4],    // +120
    pub online_labels: [ElementId; 4],    // +136
    pub info: [ElementId; 6],             // +152 (0 unused)
    pub loaded: bool,                     // +944
    pub row: i32,                         // +948
    pub section: i32,                     // +952
    pub laps: u32,                        // +956
    pub max_laps: u32,                    // +960
    pub max_opponents: u32,               // +964
    pub opponents: u32,                   // +968
    pub weather: u32,                     // +972
    pub difficulty: u32,                  // +976
    pub modified: [bool; 4],              // +980
    pub area: i32,                        // +984 track area - 1
    pub area_shown: i32,                  // +988
    pub camera_pos: u32,                  // +992
    pub camera_prev: u32,                 // +996
    pub online_visible: bool,             // +1000
    pub pro: bool,                        // +1001
    pub next: u32,                        // +1004 0 none, 1 back, else screen crc
    pub countdown: i32,                   // +1008
    pub criteria: [i8; 4],                // +1016
    pub net_state: u32,                   // +1020
}

/// FEQuickRaceTextScreen.cpp 0x00226390 — `FECK2CreateQuickRaceTextScreen`
pub fn create_quick_race_text_screen(fe: &mut Feck2) -> Arc<Mutex<CFEQuickRaceTextScreen>> {
    let obj = Arc::new(Mutex::new(CFEQuickRaceTextScreen {
        this: std::sync::Weak::new(),
        screen: 0,
        layer: 0,
        info_highlighter: 0,
        track_box: 0,
        stats_box: 0,
        titles: [0; 3],
        track_values: [0; 7],
        track_labels: [0; 7],
        option_values: [0; 4],
        option_labels: [0; 4],
        online_values: [0; 4],
        online_labels: [0; 4],
        info: [0; 6],
        loaded: false,
        row: 0,
        section: 0,
        laps: 0,
        max_laps: 0,
        max_opponents: 0,
        opponents: 0,
        weather: 0,
        difficulty: 0,
        modified: [false; 4],
        area: 0,
        area_shown: 0,
        camera_pos: 0,
        camera_prev: 0,
        online_visible: false,
        pro: false,
        next: 0,
        countdown: 0,
        criteria: [0; 4],
        net_state: 0,
    }));
    obj.lock().unwrap().this = Arc::downgrade(&obj);
    init(&obj, fe);
    obj
}

/// FEQuickRaceTextScreen.cpp 0x002264D0 / 0x00228C60 / 0x00228CA0 / 0x00228CE0 / 0x00228D30 — `Init` (+ d-pad functors)
fn init(obj: &Arc<Mutex<CFEQuickRaceTextScreen>>, fe: &mut Feck2) {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("QuickRaceTextScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 2, crc: string_crc("QuickRace_Layer"), ..Default::default() });
    let mut s = obj.lock().unwrap();
    s.screen = screen;
    s.layer = layer;
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, crc: string_crc("QuickRaceTexttext"), ..Default::default() };
    // three sections: title, then value + label per row
    let rows = [7usize, 4, 4];
    for sec in 0..3 {
        d.font = 1;
        d.position[0] = TITLE_X[sec];
        d.position[1] = SECTION_Y[sec];
        d.string = string_crc(MAIN_OPTIONS[sec]);
        s.titles[sec] = fe.create_element(layer, &d);
        for i in 0..rows[sec] {
            d.position[1] += 16.0;
            d.font = 1;
            d.position[0] = VALUE_X[sec];
            d.string = 0;
            let v = fe.create_element(layer, &d);
            d.font = 1;
            d.crc = string_crc("QuickRaceTexttextTitle");
            d.position[0] = LABEL_X[sec];
            d.string = string_crc(match sec {
                0 => TRACK_LABELS[i],
                1 => OPTION_LABELS[i],
                _ => ONLINE_LABELS[i],
            });
            let l = fe.create_element(layer, &d);
            match sec {
                0 => {
                    s.track_values[i] = v;
                    s.track_labels[i] = l;
                }
                1 => {
                    s.option_values[i] = v;
                    s.option_labels[i] = l;
                }
                _ => {
                    s.online_values[i] = v;
                    s.online_labels[i] = l;
                }
            }
        }
    }
    for i in 1..6 {
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = 0;
        d.crc = string_crc("QuickRaceTextInfo");
        d.position[0] = 400.0;
        d.position[1] = (18 * i + 330) as f32;
        d.string = if INFO_STRINGS[i].is_empty() { 0 } else { string_crc(INFO_STRINGS[i]) };
        s.info[i] = fe.create_element(layer, &d);
    }
    d.kind = ELEMENT_2D;
    d.crc = string_crc("HighlighterGraphic");
    d.texture = "FE_track_stats_box".into();
    d.vcd = "\\FE\\FE.vcd".into();
    d.position[0] = 320.0;
    d.position[1] = f32::from_bits(1134723072);
    d.f76 = 64.0;
    s.stats_box = fe.create_element(layer, &d);
    d.texture = "FE_info_highlighter".into();
    d.position[0] = -150.0;
    d.position[1] = 0.0;
    s.info_highlighter = fe.create_element(layer, &d);
    d.texture = "FE_track_stats_box".into();
    d.position[0] = 0.0;
    d.position[1] = 90.0;
    s.track_box = fe.create_element(layer, &d);
    s.next = 0;
    s.countdown = 0;
    s.area = 0;
    s.area_shown = -1;
    s.pro = false;
    fe.quick_race.single = true;
    s.modified = [false; 4];
    s.laps = 1;
    s.camera_pos = 5;
    drop(s);
    // FEQuickRaceTextScreen.cpp 0x00226FD0 — AssignEvents
    fe.map(layer, functor(obj, qr_enter), ev::ENTER);
    fe.map(layer, functor(obj, qr_activate), ev::ACTIVATE);
    fe.map(layer, functor(obj, |_, _, _| true), ev::LEAVE);
    fe.map(layer, functor(obj, qr_start), ev::START);
    fe.map(layer, functor(obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(obj, qr_update), ev::UPDATE);
    fe.map(layer, functor(obj, |s, fe, _| {
        // 0x00228C60
        s.move_row(fe, -1);
        true
    }), ev::UP_PRESS);
    fe.map(layer, functor(obj, |s, fe, _| {
        // 0x00228CA0
        s.move_row(fe, 1);
        true
    }), ev::DOWN_PRESS);
    fe.map(layer, functor(obj, |s, fe, _| {
        // 0x00228CE0
        s.change_value(fe, -1);
        true
    }), ev::LEFT_PRESS);
    fe.map(layer, functor(obj, |s, fe, _| {
        // 0x00228D30
        s.change_value(fe, 1);
        true
    }), ev::RIGHT_PRESS);
    fe.map(layer, functor(obj, qr_select), ev::SELECT_PRESS);
    fe.map(layer, functor(obj, qr_cancel), ev::CANCEL_PRESS);
    fe.map(layer, functor(obj, qr_random), ev::OPTION1_PRESS);
    fe.map(layer, functor(obj, |_, _, _| true), ev::OPTION2_PRESS);
}

const YELLOW: (u32, u32, u32, u32) = (128, 255, 110, 0);
const DISABLED: (u32, u32, u32, u32) = (128, 68, 68, 68);

fn c(t: (u32, u32, u32, u32)) -> u32 {
    visos_colour(t.0, t.1, t.2, t.3)
}

/// FEQuickRaceTextScreen.cpp 0x00227FD0 — Enter
fn qr_enter(s: &mut CFEQuickRaceTextScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(mut o) = fe.overlay.take() {
        overlay::attach_to_screen(&mut o, fe, Some(s.screen));
        fe.overlay = Some(o);
    }
    // FE3DCamera.cpp 0x00272140
    fe.foyer.set_fov(50.0);
    s.camera_pos = s.camera_target();
    s.camera_prev = s.camera_pos;
    // 3D foyer "quickR", characters (FE3DCamera / FE3DCharacters, not ported)
    s.loaded = true;
    for i in 0..3 {
        fe.element_load(s.titles[i]);
        fe.text(s.titles[i]).set_colour(c(YELLOW));
        fe.text(s.titles[i]).set_colour2(c(YELLOW));
    }
    for i in 0..7 {
        fe.element_load(s.track_labels[i]);
        fe.element_load(s.track_values[i]);
        fe.text(s.track_values[i]).set_justification(0);
    }
    for i in 0..4 {
        fe.element_load(s.option_labels[i]);
        fe.element_load(s.option_values[i]);
        fe.text(s.option_values[i]).set_justification(0);
        fe.element_load(s.online_labels[i]);
        fe.element_load(s.online_values[i]);
        fe.text(s.online_values[i]).set_justification(0);
    }
    let online = fe.quick_race.online;
    s.set_online_visible(fe, online);
    for i in 1..6 {
        fe.element_load(s.info[i]);
        fe.text(s.info[i]).set_colour(c(YELLOW));
        fe.text(s.info[i]).set_justification(2);
    }
    fe.element_set_position(s.info[3], 400.0, 200.0, 0.0);
    fe.element_set_position(s.info[4], 400.0, 240.0, 0.0);
    fe.element_set_position(s.info[5], 400.0, 280.0, 0.0);
    fe.element_load(s.info_highlighter);
    fe.e2d(s.info_highlighter).set_colour(visos_colour(128, 120, 77, 14), false, COLOUR_WHITE);
    fe.element_load(s.track_box);
    fe.e2d(s.track_box).set_size(271, 331);
    fe.element_load(s.stats_box);
    fe.e2d(s.stats_box).set_size(215, 80);
    s.row = 0;
    s.section = 0;
    if let Some(t) = fe.quick_race.track {
        s.area = fe.events.tracks[t].area as i32 - 1;
    } else {
        // defaults: underground sprint, London "Race02"
        let q = &mut fe.quick_race;
        q.pro = false;
        q.race_type = 2;
        s.area = 1;
        q.track = fe.events.find_track(2, "LONDON");
        let r = q.track.and_then(|t| fe.events.find_race(t, 1, ERaceType::from_index(q.race_type), "Race02"));
        q.races = [r, r, r];
        q.laps = 1;
        q.opponents = 3;
        q.weather = 0;
        q.difficulty = 0;
    }
    s.pro = fe.quick_race.pro;
    s.refresh_options_defaults(fe);
    s.change_value(fe, 0);
    true
}

/// FEQuickRaceTextScreen.cpp 0x00228780 — Activate
fn qr_activate(s: &mut CFEQuickRaceTextScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    crate::feck2::screens::common::fe_3d_camera::show_hangout_and_track_area(fe);
    if let Some(mut o) = fe.overlay.take() {
        overlay::reset(&mut o, fe);
        overlay::set_nav_text(&mut o, fe, 0, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(&mut o, fe, 1, string_crc("GENERAL_BACK"));
        overlay::set_nav_text(&mut o, fe, 2, string_crc("GENERAL_RANDOM"));
        overlay::show_arrows(&mut o, fe, 0);
        overlay::set_title(&mut o, fe, string_crc("MAINMENU_QUICKRACE"));
        overlay::show_borders(&mut o, fe);
        overlay::enable(&mut o);
        fe.overlay = Some(o);
    }
    s.area_shown = -1;
    true
}

/// FEQuickRaceTextScreen.cpp 0x00228910 — Start
fn qr_start(s: &mut CFEQuickRaceTextScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    fe.layer_fade_all(s.layer, 100, 20);
    s.move_camera(false);
    fe.e2d(s.info_highlighter).pulse(60, 30, 0);
    s.next = 0;
    s.net_state = 2;
    true
}

/// FEQuickRaceTextScreen.cpp 0x002289D0 — Update: leave once the fade countdown runs out.
fn qr_update(s: &mut CFEQuickRaceTextScreen, fe: &mut Feck2, e: &CEvent) -> bool {
    if s.next != 0 {
        s.countdown = (s.countdown as f32 - e.dt()) as i32;
        if s.countdown <= 0 {
            if s.next == 1 {
                fe.goto_previous_screen(1, 0);
            } else {
                let n = s.next;
                fe.goto_next_screen(n);
            }
        }
    }
    true
}

/// FEQuickRaceTextScreen.cpp 0x00228D80 — Select: store the selections and leave for car select.
fn qr_select(s: &mut CFEQuickRaceTextScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let q = &mut fe.quick_race;
    q.selected_race_id = q.races[0].map(|r| fe.events.races[r].id_crc).unwrap_or(0);
    q.pro = s.pro;
    q.laps = s.laps;
    q.opponents = s.opponents;
    q.difficulty = s.difficulty;
    q.weather = s.weather;
    q.drag = q.race_type == 5;
    s.next = if q.skip_car_select { string_crc("QuickRaceStartScreen") } else { string_crc("QuickRaceCarSelectScreen") };
    s.countdown = 10;
    fe.e2d(s.info_highlighter).pulse(-1, s.countdown, 0);
    if let Some(mut o) = fe.overlay.take() {
        overlay::slide_out(&mut o, fe, s.countdown);
        fe.layer_fade_all(s.layer, 0, s.countdown);
        overlay::fade_arrows(&mut o, fe, 0, 0, s.countdown);
        fe.overlay = Some(o);
    }
    true
}

/// FEQuickRaceTextScreen.cpp 0x00229200 — Cancel
fn qr_cancel(s: &mut CFEQuickRaceTextScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(mut o) = fe.overlay.take() {
        overlay::slide_out(&mut o, fe, 5);
        fe.overlay = Some(o);
    }
    s.next = 1;
    true
}

/// FEQuickRaceTextScreen.cpp 0x00229310 — Option 1: random event.
fn qr_random(s: &mut CFEQuickRaceTextScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.random_event(fe);
    s.refresh_track(fe, 0);
    s.random_options(fe);
    s.refresh_options(fe, 0);
    true
}

/// `VsRandRange` stand-in: a small lcg (the original uses the crt rand()).
fn rand_range(fe: &mut Feck2, n: u32) -> u32 {
    fe.rand_state = fe.rand_state.wrapping_mul(214013).wrapping_add(2531011);
    let r = (fe.rand_state >> 16) & 0x7FFF;
    if n == 0 { 0 } else { r % n }
}

impl CFEQuickRaceTextScreen {
    /// FEQuickRaceTextScreen.cpp 0x0022B270 — camera position for the category.
    fn camera_target(&self) -> u32 {
        if self.pro { 6 } else { 5 }
    }

    /// FEQuickRaceTextScreen.cpp 0x0022B2A0 — `MoveCamera` (the 3D move itself is not ported)
    fn move_camera(&mut self, _snap: bool) {
        self.camera_prev = self.camera_pos;
        self.camera_pos = self.camera_target();
    }

    /// FEQuickRaceTextScreen.cpp 0x002286E0 — online section visibility.
    fn set_online_visible(&mut self, fe: &mut Feck2, v: bool) {
        self.online_visible = v;
        fe.el(self.titles[2]).set_visible(v);
        for i in 0..4 {
            fe.el(self.online_labels[i]).set_visible(v);
            fe.el(self.online_values[i]).set_visible(v);
        }
    }

    /// FEQuickRaceTextScreen.cpp 0x0022B0D0 — rows in a section.
    fn rows_in_section(&self, fe: &Feck2, section: i32) -> i32 {
        match section {
            0 => {
                if fe.quick_race.single { 5 } else { 7 }
            }
            1 | 2 => 4,
            _ => 0,
        }
    }

    /// FEQuickRaceTextScreen.cpp 0x0022B140 — `MoveRow`
    fn move_row(&mut self, fe: &mut Feck2, d: i32) {
        self.row += d;
        let rows = self.rows_in_section(fe, self.section);
        let last = if self.online_visible { 2 } else { 1 };
        if self.row >= 0 {
            if self.row >= rows {
                self.section += 1;
                if self.section > last {
                    self.section = 0;
                }
                self.row = 0;
            }
        } else {
            self.section -= 1;
            if self.section < 0 {
                self.section = last;
            }
            self.row = self.rows_in_section(fe, self.section) - 1;
        }
        self.change_value(fe, 0);
    }

    /// FEQuickRaceTextScreen.cpp 0x00229520 — `ChangeValue` on the highlighted row.
    fn change_value(&mut self, fe: &mut Feck2, d: i32) {
        if self.section == 0 {
            self.refresh_track(fe, d);
            if d != 0 {
                self.move_camera(true);
            }
            self.area_shown = self.area;
        }
        if self.section == 1 {
            self.refresh_options(fe, d);
        }
        if self.section == 2 {
            self.refresh_online(fe, d);
        }
    }

    /// FEQuickRaceTextScreen.cpp 0x002277A0 — cycle the area (then the first selectable race there).
    fn cycle_area(&mut self, fe: &mut Feck2, d: i32) {
        let pro = self.pro;
        let mut step = if fe.quick_race.track.is_none() { 0 } else { d };
        let category = if pro { 2 } else { 1 };
        let mut area;
        loop {
            self.area += step;
            if self.area >= 0 {
                if self.area > 3 {
                    self.area = 0;
                }
            } else {
                self.area = 3;
            }
            area = (self.area + 1) as u32;
            if track_count(&fe.events, area, RACE_SELECTABLE) != 0 {
                break;
            }
            if step == 0 {
                step = 1;
            }
        }
        let Some(start) = first_track(&fe.events, area, RACE_SELECTABLE) else { return };
        let mut t = start;
        loop {
            for rt in 0..7u32 {
                if race_count(&fe.events, t, category, rt, RACE_SELECTABLE) != 0 {
                    for &r in &fe.events.tracks[t].races[rt as usize] {
                        let race = &fe.events.races[r];
                        if race.category == category && race.flags & RACE_SELECTABLE != 0 {
                            let q = &mut fe.quick_race;
                            q.track = Some(t);
                            q.race_type = rt;
                            q.races = [Some(r); 3];
                            return;
                        }
                    }
                }
            }
            t = if step == 1 {
                next_track(&fe.events, t, area, RACE_SELECTABLE).unwrap_or(start)
            } else {
                prev_track(&fe.events, t, area, RACE_SELECTABLE).unwrap_or(start)
            };
            if t == start {
                break;
            }
        }
        // nothing in this category here: original recurses one area on
        let next = if d == 0 { 1 } else { d };
        self.cycle_area(fe, next);
    }

    /// FEQuickRaceTextScreen.cpp 0x00227A20 — cycle the race type.
    fn cycle_type(&mut self, fe: &mut Feck2, d: i32) {
        let Some(mut t) = fe.quick_race.track else { return };
        let category = if self.pro { 2 } else { 1 };
        let start = fe.quick_race.race_type;
        let mut rt = start;
        if d == 1 {
            rt = next_type(rt);
        } else if d == -1 {
            rt = prev_type(rt);
        }
        let area = fe.events.tracks[t].area;
        loop {
            if tracks_with_type(&fe.events, category, area, rt, RACE_SELECTABLE) != 0 {
                while race_count(&fe.events, t, category, rt, RACE_SELECTABLE) == 0 {
                    t = next_track(&fe.events, t, area, RACE_SELECTABLE).unwrap_or(t);
                }
                for &r in &fe.events.tracks[t].races[rt as usize] {
                    if fe.events.races[r].category == category {
                        let q = &mut fe.quick_race;
                        q.track = Some(t);
                        q.race_type = rt;
                        q.races = [Some(r); 3];
                        return;
                    }
                }
                t = next_track(&fe.events, t, area, RACE_SELECTABLE).unwrap_or(t);
            } else if d == -1 {
                rt = prev_type(rt);
            } else if d <= 1 {
                rt = next_type(rt);
            }
            if rt == start {
                break;
            }
        }
    }

    /// FEQuickRaceTextScreen.cpp 0x00227C40 — cycle the race in a slot (continues onto other tracks).
    fn cycle_race(&mut self, fe: &mut Feck2, d: i32, slot: usize) {
        let Some(mut t) = fe.quick_race.track else { return };
        let category = if self.pro { 2 } else { 1 };
        let rt = fe.quick_race.race_type;
        let area = fe.events.tracks[t].area;
        let Some(cur) = fe.quick_race.races[slot] else { return };
        if d == 0 {
            return;
        }
        let mut cand = if d == 1 { race_next(&fe.events, cur) } else { race_prev(&fe.events, cur) };
        for _ in 0..fe.events.races.len() + fe.events.tracks.len() + 1 {
            if cand.is_none() || race_count(&fe.events, t, category, rt, RACE_SELECTABLE) == 0 {
                loop {
                    t = if d == 1 {
                        next_track(&fe.events, t, area, RACE_SELECTABLE).unwrap_or(t)
                    } else {
                        prev_track(&fe.events, t, area, RACE_SELECTABLE).unwrap_or(t)
                    };
                    if race_count(&fe.events, t, category, rt, RACE_SELECTABLE) != 0 {
                        break;
                    }
                }
                cand = if race_count(&fe.events, t, category, rt, RACE_SELECTABLE) == 1 {
                    nth_race(&fe.events, t, category, rt, RACE_SELECTABLE, 0)
                } else {
                    let list = &fe.events.tracks[t].races[rt as usize];
                    let iter: Box<dyn Iterator<Item = &usize>> = if d == 1 { Box::new(list.iter()) } else { Box::new(list.iter().rev()) };
                    iter.copied().find(|&r| fe.events.races[r].category == category && fe.events.races[r].flags & RACE_SELECTABLE != 0)
                };
            }
            if let Some(r) = cand {
                let race = &fe.events.races[r];
                if race.category == category && race.flags & RACE_SELECTABLE != 0 {
                    fe.quick_race.races[slot] = Some(r);
                    fe.quick_race.track = Some(t);
                    return;
                }
                cand = if d == 1 { race_next(&fe.events, r) } else { race_prev(&fe.events, r) };
            }
        }
    }

    /// FEQuickRaceTextScreen.cpp 0x002295B0 — the track section rows.
    fn refresh_track(&mut self, fe: &mut Feck2, d: i32) {
        match self.row {
            0 => {
                if d != 0 {
                    self.pro = !self.pro;
                    self.cycle_area(fe, 0);
                    self.cycle_type(fe, 0);
                    self.cycle_race(fe, 0, 0);
                    let r0 = fe.quick_race.races[0];
                    fe.quick_race.races[1] = r0;
                    fe.quick_race.races[2] = r0;
                }
            }
            1 => {
                if d != 0 {
                    self.cycle_area(fe, if d <= 0 { -1 } else { 1 });
                }
            }
            2 => {
                if d != 0 {
                    fe.quick_race.single = !fe.quick_race.single;
                }
            }
            3 => {
                if d != 0 {
                    self.cycle_type(fe, if d <= 0 { -1 } else { 1 });
                }
            }
            4 | 5 | 6 => {
                if d != 0 {
                    self.cycle_race(fe, if d <= 0 { -1 } else { 1 }, (self.row - 4) as usize);
                }
            }
            _ => {}
        }
        if self.row == 0 && d == 0 {
            // first call: make sure a valid track/race is chosen
        }
        if let Some(t) = fe.quick_race.track {
            self.area = fe.events.tracks[t].area as i32 - 1;
        }
        let tv = self.track_values;
        fe.text(tv[0]).set_string(string_crc(CATEGORY[self.pro as usize]));
        fe.text(tv[1]).set_string(string_crc(AREAS[(self.area + 1).clamp(0, 4) as usize]));
        let single = fe.quick_race.single;
        fe.text(tv[2]).set_string(string_crc(if single { "EVENTTYPE_SINGLE" } else { "EVENTTYPE_TOURNAMENT" }));
        fe.el(tv[5]).set_visible(!single);
        fe.el(tv[6]).set_visible(!single);
        let rt = fe.quick_race.race_type as usize;
        fe.text(tv[3]).set_string(string_crc(RACE_TYPES[rt.min(6)]));
        for i in 0..3 {
            if let Some(r) = fe.quick_race.races[i] {
                let crc = fe.events.races[r].display_crc;
                fe.text(tv[4 + i]).set_string(crc);
            }
        }
        if self.section == 0 {
            self.place_highlight(fe, tv[self.row as usize], VALUE_X[0], SECTION_Y[0]);
        }
        self.refresh_info(fe);
        self.refresh_options_defaults(fe);
        self.refresh_options(fe, 0);
    }

    /// arrows + highlighter around a value (shared tail of 0x002295B0 / 0x00229E60 / 0x0022AAD0).
    fn place_highlight(&mut self, fe: &mut Feck2, value: ElementId, x: f32, y0: f32) {
        let w = {
            let t = match fe.elements[value].as_ref() {
                Some(Element::Text(t)) => t.clone(),
                _ => return,
            };
            t.get_extent_width(&fe.text_ctx())
        };
        let y = (16 * (self.row + 1)) as f32 + y0;
        if let Some(mut o) = fe.overlay.take() {
            overlay::position_arrows(&mut o, fe, x - 29.0, w as f32 + x, y + -7.0, 0);
            fe.overlay = Some(o);
        }
        fe.element_set_position(self.info_highlighter, -150.0, y - 3.0, 0.0);
    }

    /// FEQuickRaceTextScreen.cpp 0x00229BB0 — best lap / prestige of the first race.
    fn refresh_info(&mut self, fe: &mut Feck2) {
        if let Some(r) = fe.quick_race.races[0] {
            let lap = fe.events.races[r].unknown_28;
            fe.strings.set_argument(string_crc("QUICKRACE_BESTLAPTIME"), string_crc("fLapTime"), lap.to_bits()).ok();
            fe.strings.set_argument(string_crc("QUICKRACE_BESTPRESTIGE"), string_crc("dPrestigeScore"), 0).ok();
        }
        // debug race path rows are only shown with the event database debug flag (+53)
        for i in 3..6 {
            fe.el(self.info[i]).set_visible(false);
        }
    }

    /// FEQuickRaceTextScreen.cpp 0x002293B0 — option limits for the race type, defaults from the globals.
    fn refresh_options_defaults(&mut self, fe: &mut Feck2) {
        let q = &fe.quick_race;
        self.max_laps = if q.race_type == 2 || q.race_type == 6 || q.race_type == 5 { 1 } else { 10 };
        self.max_opponents = if q.online || self.pro {
            5
        } else if q.race_type == 6 {
            0
        } else {
            3
        };
        if !self.modified[0] {
            self.laps = q.laps;
        }
        if !self.modified[1] {
            self.opponents = q.opponents;
        }
        if !self.modified[3] {
            self.weather = q.weather;
        }
        if !self.modified[2] {
            self.difficulty = q.difficulty;
        }
    }

    /// FEQuickRaceTextScreen.cpp 0x00229E60 — the options section.
    fn refresh_options(&mut self, fe: &mut Feck2, d: i32) {
        if d != 0 {
            match self.row {
                1 => self.change_opponents(fe, d),
                2 => {}
                3 => self.change_weather(d),
                _ => self.change_laps(d),
            }
            self.modified[self.row as usize] = true;
        } else {
            self.change_laps(0);
            self.change_opponents(fe, 0);
        }
        if self.laps == 0 {
            self.laps = 1;
        }
        let ov = self.option_values;
        fe.text(ov[0]).set_colour(COLOUR_WHITE);
        fe.text(ov[1]).set_colour(COLOUR_WHITE);
        fe.text(ov[2]).set_colour(c(DISABLED));
        fe.text(ov[2]).set_string(string_crc(DIFFICULTY[self.difficulty.min(2) as usize]));
        fe.text(ov[3]).set_string(string_crc(WEATHER[self.weather.min(1) as usize]));
        match fe.quick_race.race_type {
            1 => {
                fe.text(ov[0]).set_colour(c(DISABLED));
                self.laps = 0;
                fe.text(ov[1]).set_colour(c(DISABLED));
                self.opponents = 0;
            }
            2 => {
                fe.text(ov[0]).set_colour(c(DISABLED));
                self.laps = 1;
            }
            6 => {
                fe.text(ov[0]).set_colour(c(DISABLED));
                self.laps = 1;
                fe.text(ov[1]).set_colour(c(DISABLED));
                self.opponents = 0;
            }
            _ => {}
        }
        fe.text(ov[0]).set_string(string_crc("QUICKRACE_LAPS_VALUE"));
        fe.strings.set_argument(string_crc("QUICKRACE_LAPS_VALUE"), string_crc("dLaps"), self.laps).ok();
        fe.text(ov[1]).set_string(string_crc("QUICKRACE_OPPONENTS_VALUE"));
        fe.strings.set_argument(string_crc("QUICKRACE_OPPONENTS_VALUE"), string_crc("dOpponents"), self.opponents).ok();
        if self.section == 1 {
            self.place_highlight(fe, ov[self.row as usize], VALUE_X[1], SECTION_Y[1]);
        }
    }

    /// FEQuickRaceTextScreen.cpp 0x0022A4F0 — laps 1..max, wrapping.
    fn change_laps(&mut self, d: i32) {
        if d != 0 {
            self.laps = (self.laps as i32 + d) as u32;
            if self.laps != 0 {
                if self.laps > self.max_laps {
                    self.laps = 1;
                }
            } else {
                self.laps = self.max_laps;
            }
        } else if self.modified[0] {
            if self.laps > self.max_laps {
                self.laps = self.max_laps;
            }
        } else if self.max_laps <= 3 {
            self.laps = self.max_laps;
        } else {
            self.laps = 3;
        }
    }

    /// FEQuickRaceTextScreen.cpp 0x0022A410 — opponents 1..max.
    fn change_opponents(&mut self, fe: &Feck2, mut d: i32) {
        if fe.quick_race.online || fe.quick_race.race_type == 5 {
            d = 0;
        }
        self.opponents = (self.opponents as i32 + d) as u32;
        if self.max_opponents != 0 {
            if self.opponents != 0 {
                if self.opponents > self.max_opponents {
                    self.opponents = if d != 0 { 1 } else { self.max_opponents };
                }
            } else {
                self.opponents = self.max_opponents;
            }
        } else {
            self.opponents = 0;
        }
    }

    /// FEQuickRaceTextScreen.cpp 0x0022A380 — weather fair / rain.
    fn change_weather(&mut self, d: i32) {
        let mut w = self.weather as i32 + d;
        if w >= 0 {
            if w > 1 {
                w = 0;
            }
        } else {
            w = 1;
        }
        self.weather = w as u32;
    }

    /// FEQuickRaceTextScreen.cpp 0x0022AAD0 — online criteria (greyed out offline).
    fn refresh_online(&mut self, fe: &mut Feck2, d: i32) {
        if d != 0 {
            match self.row {
                0 => self.criteria[0] = (self.criteria[0] == 0) as i8,
                3 => {
                    self.criteria[3] += d as i8;
                    if self.criteria[3] < 0 {
                        self.criteria[3] = 6;
                    } else if self.criteria[3] > 6 {
                        self.criteria[3] = 0;
                    }
                }
                _ => {}
            }
        }
        let ov = self.online_values;
        fe.text(ov[0]).set_string(string_crc(if self.criteria[0] != 0 {
            "ONLINE_MATCHCRITERIA_COLLISIONS_ON"
        } else {
            "ONLINE_MATCHCRITERIA_COLLISIONS_OFF"
        }));
        fe.text(ov[1]).set_string(string_crc("ONLINE_MATCHCRITERIA_FRIENDSSLOTS_VALUE"));
        fe.text(ov[2]).set_string(string_crc(RANK_RANGE[self.criteria[2].clamp(0, 6) as usize]));
        fe.text(ov[3]).set_string(string_crc(VEHICLE_TIERS[self.criteria[3].clamp(0, 6) as usize]));
        let grey = c(DISABLED);
        fe.text(ov[0]).set_colour(grey);
        fe.text(ov[1]).set_colour(grey);
        if self.section == 2 {
            self.place_highlight(fe, ov[self.row as usize], VALUE_X[2], SECTION_Y[2]);
        }
    }

    /// FEQuickRaceTextScreen.cpp 0x0022A5D0 — random area / track / type / races.
    fn random_event(&mut self, fe: &mut Feck2) {
        if fe.quick_race.track.is_none() {
            for area in 1..5 {
                fe.quick_race.track = first_track(&fe.events, area, RACE_SELECTABLE);
                if fe.quick_race.track.is_some() {
                    break;
                }
            }
        }
        let Some(mut t) = fe.quick_race.track else { return };
        let n = track_count(&fe.events, 5, RACE_SELECTABLE);
        let skip = rand_range(fe, n);
        for _ in 0..skip {
            t = next_track(&fe.events, t, 5, RACE_SELECTABLE).unwrap_or(t);
        }
        fe.quick_race.track = Some(t);
        self.area = fe.events.tracks[t].area as i32 - 1;
        let mut rt = rand_range(fe, 7);
        let mut tries = 0;
        while race_count(&fe.events, t, 0, rt, RACE_SELECTABLE) == 0 {
            rt += 1;
            if rt == 7 {
                rt = 0;
            }
            tries += 1;
            if tries == 7 {
                log::error!("Track found with no races!!!");
                return;
            }
        }
        fe.quick_race.race_type = rt;
        let count = race_count(&fe.events, t, 0, rt, RACE_SELECTABLE);
        for slot in 0..3 {
            if slot > 0 && fe.quick_race.single {
                break;
            }
            let pick = rand_range(fe, count);
            let r = fe.events.tracks[t].races[rt as usize].get(pick as usize).copied();
            if slot == 0 {
                fe.quick_race.races = [r; 3];
            } else {
                fe.quick_race.races[slot] = r;
            }
        }
        if let Some(r) = fe.quick_race.races[0] {
            self.pro = fe.events.races[r].category == 2;
        }
        self.move_camera(true);
    }

    /// FEQuickRaceTextScreen.cpp 0x0022AA10 — random options the player has not touched.
    fn random_options(&mut self, fe: &mut Feck2) {
        let rt = fe.quick_race.race_type;
        if rt != 1 && rt != 2 && !self.modified[0] {
            self.laps = rand_range(fe, self.max_laps) + 1;
        }
        if rt != 1 && !self.modified[1] {
            self.opponents = self.max_opponents;
        }
        if !self.modified[3] {
            self.weather = rand_range(fe, 2);
        }
    }
}
