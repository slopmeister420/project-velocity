//! src\Feck2\Screens\QuickRace\FEQuickRaceCarSelectScreen.cpp — `gpQuickRaceCarSelectScreen` (create 0x002223D0,
//! code in the FECarShowroomScreen.cpp object region up to 0x00225A80). Stage 0 picks a tier, 1 a car, 2 the
//! colour, 3 commits the setup and leaves for the next screen.
//!
//! The 3D set is the "quickR" foyer: the camera moves between table entries 7 / 8 / 9 per stage, the selected
//! car is put up on the PLAYER1CAR spot after the stage-2 countdown, option 1 opens the colour swatches whose
//! colour goes straight onto that car (and into the chosen setup), and the car info panel (+5444) slides in with
//! the car list; the area model's logo wall shows the tier's logos / polaroids (`FoyerState::set_material_texture`).
//! Not ported: automod (option 2).

use std::sync::{Arc, Mutex};

use velocity_physics::vehicle_database::vehicle_database::VehicleSetup;

use crate::feck2::feck2_manager::{visos_colour, Feck2};
use crate::feck2::screens::common::fe_3d_camera::MtvCall;
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_car_info_panel::CCarInfoPanel;
use crate::feck2::screens::common::fe_car_stats_list::CCarStatsList;
use crate::feck2::screens::common::fe_colour_swatches::CColourSwatches;
use crate::string_manager::string_manager::string_crc;

/// `dword_6C12B0` x/y per player, `dword_6C12C0` arrow y offset, `dword_6C1290` loading text base per player.
const TIER_POS: [[f32; 2]; 2] = [[42.0, 125.0], [280.0, 125.0]];
const ARROW_Y: f32 = -5.0;
const LOADING_POS: [[f32; 2]; 2] = [[22.0, 225.0], [260.0, 225.0]];
/// `dword_75FC48` / `dword_75FC50` (inits 0x0042C134..)
const CAR_LABEL: [&str; 2] = ["GENERAL_BLANK", "GENERAL_BLANK"];
const CAR_NAME: [&str; 2] = ["SPLITSCREEN_PLAYER1_CAR", "SPLITSCREEN_PLAYER2_CAR"];
/// tier 8 is the player's garage (`VEHICLE_GARAGE`-style string 0x2A40E7DA)
const GARAGE_TIER: i32 = 8;
const GARAGE_STRING: u32 = 708888794;

/// the background holding the logo wall: `dword_6F7144` +136 (the selected track's area), 1 without a track
fn logo_wall_area(fe: &Feck2) -> usize {
    fe.quick_race.track.and_then(|t| fe.events.tracks.get(t)).map_or(1, |t| t.area as usize)
}

/// vehicle +0 path -> `<path>\VEHICLEF.VCD`, vehicle +4 name
fn logo_names(db: &velocity_physics::vehicle_database::vehicle_database::CVehicleDatabase, v: usize) -> (String, String) {
    let e = &db.vehicles[v];
    (format!("{}\\VEHICLEF.VCD", e.path), e.name.clone().unwrap_or_default())
}

/// `CFEQuickRaceCarSelectScreen` (5508 bytes)
pub struct CFEQuickRaceCarSelectScreen {
    pub screen: ScreenId,          // +0
    pub fg_layer: LayerId,         // +4
    pub bg_layer: LayerId,         // +8
    pub player_text: ElementId,    // +12
    pub player_bg: ElementId,      // +16
    pub tier_text: ElementId,      // +20
    pub title_bg: ElementId,       // +24
    pub title_highlight: ElementId, // +28
    pub car_label: ElementId,      // +32
    pub car_name: ElementId,       // +36
    pub loading: ElementId,        // +40
    pub inited: bool,              // +44
    pub car_index: i32,            // +5424
    pub vehicle_hash: u32,         // +5428
    pub transmission: u32,         // +5432 (1 = manual)
    pub tier: i32,                 // +5436 0..7 tiers, 8 garage
    pub stage: u32,                // +5440
    pub info: Option<CCarInfoPanel>, // +5444
    pub stats: Option<CCarStatsList>, // +5448 the R2 statistics list
    pub stats_shown: bool,         // +5452
    pub swatches: Option<CColourSwatches>, // +5456 mpColourSwatches
    pub camera: usize,             // +5460
    pub prev_camera: usize,        // +5464
    pub tier_count: u32,           // +5468
    pub waiting: bool,             // +5472
    pub countdown: i32,            // +5476
    pub players: u8,               // +5480 0 / 1 = one player, 2 = both
    pub exit: u32,                 // +5484 0 stay, 1 back, else next screen crc
    pub player: u32,               // +5488
    pub next: u32,                 // +5492
    pub reset: bool,               // +5496
    pub from_garage: bool,         // +5497
}

/// FECarShowroomScreen.cpp 0x002223D0 / 0x002224E0 — `FECK2CreateQuickRaceCarSelectScreen` (ctor; Init 0x00222620)
pub fn create_quick_race_car_select_screen(fe: &mut Feck2) -> Arc<Mutex<CFEQuickRaceCarSelectScreen>> {
    let obj = Arc::new(Mutex::new(CFEQuickRaceCarSelectScreen {
        screen: 0,
        fg_layer: 0,
        bg_layer: 0,
        player_text: 0,
        player_bg: 0,
        tier_text: 0,
        title_bg: 0,
        title_highlight: 0,
        car_label: 0,
        car_name: 0,
        loading: 0,
        inited: false,
        car_index: 0,
        vehicle_hash: 0,
        transmission: 0,
        tier: 0,
        stage: 0,
        info: None,
        stats: None,
        stats_shown: false,
        swatches: None,
        camera: 7,
        prev_camera: 7,
        tier_count: 0,
        waiting: false,
        countdown: 0,
        players: 0,
        exit: 0,
        player: 0,
        next: 0,
        reset: true,
        from_garage: false,
    }));
    init(&obj, fe);
    obj
}

/// FECarShowroomScreen.cpp 0x00222620 / 0x00224DC0 / 0x00224DF0 — Init (+ the camera held functors)
fn init(obj: &Arc<Mutex<CFEQuickRaceCarSelectScreen>>, fe: &mut Feck2) {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("QuickRaceCarSelectScreen") });
    let bg = fe.create_layer(Some(screen), &SLayerDesc { priority: 6, crc: string_crc("QuickRaceCarSelect_BGLayer"), ..Default::default() });
    let fg = fe.create_layer(Some(screen), &SLayerDesc { priority: 2, crc: string_crc("QuickRaceCarSelect_FGLayer"), ..Default::default() });
    let mut s = obj.lock().unwrap();
    s.screen = screen;
    s.bg_layer = bg;
    s.fg_layer = fg;
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 2, crc: string_crc("PlayerIndexText"), ..Default::default() };
    d.position[0] = f32::from_bits(1109917696);
    d.position[1] = f32::from_bits(1118568448);
    d.string = string_crc("QUICKRACE_CARSELECT_PLAYERCAR");
    s.player_text = fe.create_element(fg, &d);
    d.kind = ELEMENT_2D;
    d.crc = string_crc("PlayerIndexBackground");
    d.position[0] = 0.0;
    d.position[1] = f32::from_bits(1117913088);
    d.texture = "FE_summary_title_highlight".into();
    d.vcd = "\\FE\\FE.vcd".into();
    s.player_bg = fe.create_element(fg, &d);
    d.kind = ELEMENT_STATIC_TEXT;
    d.font = 2;
    d.crc = string_crc("TierSelectionText");
    d.position[0] = TIER_POS[0][0];
    d.position[1] = TIER_POS[0][1];
    d.string = 0;
    s.tier_text = fe.create_element(fg, &d);
    d.kind = ELEMENT_2D;
    d.position[0] = 0.0;
    d.position[1] = TIER_POS[0][1] - 1.0;
    d.texture = "FE_menu_TitleHighlight".into();
    s.title_highlight = fe.create_element(fg, &d);
    d.position[1] = TIER_POS[0][1] - 11.0;
    d.texture = "FE_menu_TitleBackground".into();
    s.title_bg = fe.create_element(fg, &d);
    d.dynamic = false;
    d.kind = ELEMENT_STATIC_TEXT;
    d.crc = string_crc("Car1Name");
    d.font = 2;
    d.position[0] = TIER_POS[0][0];
    d.position[1] = TIER_POS[0][1];
    d.string = string_crc(CAR_LABEL[0]);
    s.car_label = fe.create_element(fg, &d);
    d.font = 2;
    d.string = string_crc(CAR_NAME[0]);
    s.car_name = fe.create_element(fg, &d);
    d.dynamic = false;
    d.crc = string_crc("Car1Name");
    d.font = 3;
    d.position[0] = LOADING_POS[0][0] + 64.0;
    d.position[1] = LOADING_POS[0][1] + 70.0;
    d.string = string_crc("GENERAL_LOADING");
    s.loading = fe.create_element(fg, &d);
    s.next = 0;
    s.stage = 0;
    // car info panel (0x00287C00), the R2 stats list (0x002894A0 / 0x00289510 on the overlay's layer, 0x002105B0)
    s.info = Some(CCarInfoPanel::create(fe, fg));
    let layer = fe.overlay.as_ref().map_or(fg, |o| o.layer);
    s.stats = Some(CCarStatsList::new(layer));
    s.stats_shown = false;
    drop(s);
    // FECarShowroomScreen.cpp 0x00222DF0 — AssignEvents (FG layer)
    fe.map(fg, functor(obj, cs_enter), ev::ENTER);
    fe.map(fg, functor(obj, cs_activate), ev::ACTIVATE);
    fe.map(fg, functor(obj, |_, _, _| true), ev::LEAVE);
    fe.map(fg, functor(obj, cs_start), ev::START);
    fe.map(fg, functor(obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(fg, functor(obj, cs_update), ev::UPDATE);
    // up/down (0x00223FE0 / 0x002240C0): stats list or colour swatch rows; held (0x00224070 / 0x00224150) only
    // scrolls the stats list
    fe.map(fg, functor(obj, |s, fe, _| cs_up_down(s, fe, -1)), ev::UP_PRESS);
    fe.map(fg, functor(obj, |s, fe, _| cs_up_down(s, fe, 1)), ev::DOWN_PRESS);
    fe.map(fg, functor(obj, |s, fe, _| cs_held(s, fe, 1)), ev::UP_HELD);
    fe.map(fg, functor(obj, |s, fe, _| cs_held(s, fe, -1)), ev::DOWN_HELD);
    fe.map(fg, functor(obj, |s, fe, _| cs_left_right(s, fe, -1)), ev::LEFT_PRESS);
    fe.map(fg, functor(obj, |s, fe, _| cs_left_right(s, fe, 1)), ev::RIGHT_PRESS);
    fe.map(fg, functor(obj, cs_select), ev::SELECT_PRESS);
    fe.map(fg, functor(obj, cs_option1), ev::OPTION1_PRESS);
    fe.map(fg, functor(obj, cs_option2), ev::OPTION2_PRESS);
    fe.map(fg, functor(obj, cs_stats), ev::R2_PRESS);
    fe.map(fg, functor(obj, cs_cancel), ev::CANCEL_PRESS);
    // 0x00224DC0 / 0x00224DF0: camera left/right held swallowed unless locked
    fe.map(fg, functor(obj, |_, fe: &mut Feck2, _| !fe.online_locked), ev::CAM_LEFT_PRESS + 1);
    fe.map(fg, functor(obj, |_, fe: &mut Feck2, _| !fe.online_locked), ev::CAM_RIGHT_PRESS + 1);
}

/// FECarShowroomScreen.cpp 0x002224B0 / 0x00226170 / 0x00226220 — `SetNextScreen(player, crc)`
pub fn set_next_screen(s: &mut CFEQuickRaceCarSelectScreen, player: u32, next: u32) {
    debug_assert!(s.player <= 1, "Invalid player index!!");
    s.player = player;
    s.reset = true;
    s.next = next;
}

/// FECarShowroomScreen.cpp 0x00223510 — Enter
fn cs_enter(s: &mut CFEQuickRaceCarSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some((player, next)) = fe.car_select_next.take() {
        set_next_screen(s, player, next);
    }
    if let Some(mut o) = fe.overlay.take() {
        overlay::attach_to_screen(&mut o, fe, Some(s.screen));
        fe.overlay = Some(o);
    }
    // FE3DCamera: end any move, foyer "quickR", fov 50
    fe.foyer.snap_to_target();
    fe.foyer.set_foyer("quickR");
    fe.foyer.set_fov(50.0);
    if s.reset || fe.quick_race.skip_car_select || fe.quick_race.online {
        s.stage = 0;
    }
    s.reset = false;
    s.camera = s.camera_index();
    s.prev_camera = s.camera;
    fe.foyer.move_to(s.camera);
    // 0x00223695
    fe.foyer.shake(-1.0, f32::from_bits(0x3DCC_CCCD), -1.0);
    fe.foyer_vehicles.set_foyer("quickR");
    if !s.inited {
        s.inited = true;
        s.transmission = 0;
        s.stage = 0;
        s.tier = GARAGE_TIER;
        s.car_index = fe.quick_race.garage_selected as i32; // dword_6F71C4
        let mut sw = CColourSwatches::init(fe, s.fg_layer, 140, 345);
        sw.load(fe);
        s.swatches = Some(sw);
        if let Some(mut i) = s.info.take() {
            i.load(fe);
            s.info = Some(i);
        }
        // 0x00289550
        if let Some(mut st) = s.stats.take() {
            st.reset(fe);
            s.stats = Some(st);
        }
    }
    s.from_garage = false;
    // FEAlloyEditor_Basic.cpp 0x002595E0: the foyer's spots; one car shown (two from the garage)
    fe.foyer_vehicles.load("Vehicles", false);
    fe.foyer_vehicles.count = if s.from_garage { 2 } else { 1 };
    fe.element_load(s.player_text);
    fe.element_load(s.player_bg);
    let arg = string_crc("QUICKRACE_CARSELECT_PLAYERCAR");
    fe.strings.set_argument(arg, string_crc("dPlayerNum"), s.player + 1).ok();
    for id in [s.tier_text, s.title_bg, s.title_highlight, s.car_label, s.car_name, s.loading] {
        fe.element_load(id);
    }
    fe.el(s.loading).set_visible(false);
    fe.text(s.loading).set_colour(visos_colour(128, 124, 75, 3));
    s.exit = 0;
    true
}

/// FECarShowroomScreen.cpp 0x002239F0 / FECK2ElementStaticText.cpp 0x00287BA0 — Activate
fn cs_activate(s: &mut CFEQuickRaceCarSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    crate::feck2::screens::common::fe_3d_camera::show_hangout_and_track_area(fe);
    if let Some(mut o) = fe.overlay.take() {
        overlay::reset(&mut o, fe);
        overlay::set_nav_text(&mut o, fe, 0, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(&mut o, fe, 1, string_crc("GENERAL_BACK"));
        overlay::set_nav_text(&mut o, fe, 2, string_crc("GENERAL_TRANSMISSION"));
        overlay::set_nav_text(&mut o, fe, 3, string_crc("GENERAL_AUTOMOD"));
        overlay::set_title(&mut o, fe, string_crc("MAINMENU_QUICKRACE"));
        overlay::show_borders(&mut o, fe);
        overlay::show_arrows(&mut o, fe, 0);
        overlay::fade_arrows(&mut o, fe, -128, 0, 0);
        overlay::enable(&mut o);
        // 0x00287BA0: the car info panel's R2 prompt
        overlay::set_nav_text(&mut o, fe, 13, string_crc("GENERAL_CARSTATS"));
        fe.overlay = Some(o);
    }
    let _ = s;
    true
}

/// FECarShowroomScreen.cpp 0x00223BE0 — Start
fn cs_start(s: &mut CFEQuickRaceCarSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.countdown = 50;
    s.waiting = true;
    s.change_tier(fe, 0, 0);
    s.change_tier(fe, 0, 1);
    s.change_car(fe, 0, 0);
    s.place_highlight(fe, 0);
    if s.stage == 3 {
        s.stage = 2;
    }
    if s.stage != 0 {
        s.show_tier(fe, false);
        s.show_cars(fe, true, 0);
    } else {
        s.show_tier(fe, true);
        s.show_cars(fe, false, 0);
    }
    s.players = if s.from_garage { 2 } else { 0 };
    s.refresh_nav(fe);
    fe.layer_fade_all(s.fg_layer, 128, 0);
    fe.layer_fade_all(s.bg_layer, 128, 0);
    if let Some(mut o) = fe.overlay.take() {
        overlay::fade_arrows(&mut o, fe, 128, 0, 0);
        fe.overlay = Some(o);
    }
    fe.e2d(s.title_highlight).pulse(60, 30, 0);
    true
}

/// FECarShowroomScreen.cpp 0x00223D80 — Update
fn cs_update(s: &mut CFEQuickRaceCarSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    // 0x00225F90 online bail-out / 0x00226070 network start: offline only here
    if s.exit != 0 {
        if fe.layer_fade_all(s.fg_layer, -5, 0) && fe.layer_fade_all(s.bg_layer, -5, 0) {
            if s.exit == 1 {
                fe.goto_previous_screen(1, 0);
            } else {
                let e = s.exit;
                fe.goto_next_screen(e);
            }
        } else if let Some(mut o) = fe.overlay.take() {
            overlay::fade_arrows(&mut o, fe, -5, 0, 0);
            fe.overlay = Some(o);
        }
        return true;
    }
    // the bars slide in once the 3D camera has arrived; without the foyer it is there at once
    if let Some(mut o) = fe.overlay.take() {
        if !o.slid_in {
            overlay::slide_in(&mut o, fe, 10);
        }
        fe.overlay = Some(o);
    }
    if s.countdown == 1 {
        fe.foyer_vehicles.set_visible(0, false);
        fe.foyer_vehicles.set_visible(1, false);
    }
    if s.countdown == -1 {
        if s.waiting {
            if s.players == 2 {
                s.show_vehicle(fe, 0);
                s.show_vehicle(fe, 1);
            } else {
                let p = s.players as usize;
                s.show_vehicle(fe, p);
            }
            s.waiting = false;
            s.update_camera(fe);
            s.refresh_nav(fe);
            fe.el(s.loading).set_visible(false);
        }
    } else if s.stage == 2 {
        s.waiting = true;
        s.countdown -= 1;
    }
    true
}

/// FECarShowroomScreen.cpp 0x002241A0 / 0x00224230 — left / right: previous / next tier or car.
fn cs_left_right(s: &mut CFEQuickRaceCarSelectScreen, fe: &mut Feck2, d: i32) -> bool {
    if fe.online_locked {
        return false;
    }
    if !s.stats_shown {
        if s.swatches.as_ref().is_some_and(|w| w.shown) {
            if let Some(mut w) = s.swatches.take() {
                w.move_col(fe, d);
                s.swatches = Some(w);
            }
            s.apply_colour(fe);
        } else {
            s.change_tier(fe, d, 0);
        }
    }
    true
}

/// FECarShowroomScreen.cpp 0x00223FE0 / 0x002240C0 - up / down: colour swatch row
fn cs_up_down(s: &mut CFEQuickRaceCarSelectScreen, fe: &mut Feck2, d: i32) -> bool {
    if fe.online_locked {
        return false;
    }
    if s.stats_shown {
        // up scrolls the list down (+1)
        s.scroll_stats(fe, -d);
    } else if s.swatches.as_ref().is_some_and(|w| w.shown) {
        if let Some(mut w) = s.swatches.take() {
            w.move_row(fe, d);
            s.swatches = Some(w);
        }
        s.apply_colour(fe);
    }
    true
}

/// FECarShowroomScreen.cpp 0x002242C0 — Select
fn cs_select(s: &mut CFEQuickRaceCarSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if fe.online_locked {
        return false;
    }
    s.hide_stats(fe);
    if s.swatches.as_ref().is_some_and(|w| w.shown) {
        if let Some(mut w) = s.swatches.take() {
            w.show(fe, false);
            s.swatches = Some(w);
        }
        return true;
    }
    match s.stage {
        0 => {
            s.stage += 1;
            s.show_tier(fe, false);
            s.show_cars(fe, true, 0);
        }
        1 => {
            s.stage += 1;
            s.countdown = 10;
            fe.el(s.loading).set_visible(true);
        }
        2 => s.stage += 1,
        _ => {}
    }
    if s.stage == 3 {
        // dword_768978+2220 automatic gearbox, then the setup into the player's slot (dword_6F724C[139 * player])
        fe.quick_race.automatic = s.transmission != 1;
        // the car on the spot (with its colour) if one is up, else the default setup
        let mut setup = VehicleSetup::new();
        let mut tier = 0;
        if let Some(db) = fe.vehicles.as_ref() {
            match fe.foyer_vehicles.slots.first().and_then(|v| v.setup.clone()) {
                Some(x) => setup = x,
                None => {
                    db.setup_default_vehicle(&mut setup, s.vehicle_hash);
                }
            }
            if let Some(v) = setup.vehicle {
                tier = db.vehicles[v].data.tier as u32;
            }
        }
        fe.quick_race.vehicle_tier = tier; // dword_6F7138
        // 0x0022446D: dword_768978+2220 = the FE working setup (+0x720) +0x18C, 1 = automatic
        setup.unk_18c = (s.transmission != 1) as u32;
        if s.player == 0 {
            fe.quick_race.car_name_crc = s.vehicle_hash;
        } else {
            fe.quick_race.car_name_crc2 = s.vehicle_hash;
        }
        fe.quick_race.player_setups[s.player as usize & 1] = Some(setup);
        if s.next != 0 && !fe.quick_race.skip_car_select {
            s.exit = s.next;
            if s.next == string_crc("QuickRaceCarSelectScreen") {
                set_next_screen(s, 1, string_crc("QuickRaceStartScreen"));
            }
        } else {
            s.exit = string_crc("QuickRaceStartScreen");
        }
        if let Some(mut o) = fe.overlay.take() {
            overlay::slide_out(&mut o, fe, 5);
            fe.overlay = Some(o);
        }
    }
    if s.stage != 2 {
        s.update_camera(fe);
        s.refresh_nav(fe);
    }
    true
}

/// FECarShowroomScreen.cpp 0x00224790 — Cancel
fn cs_cancel(s: &mut CFEQuickRaceCarSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if fe.online_locked {
        return false;
    }
    s.hide_stats(fe);
    if s.swatches.as_ref().is_some_and(|w| w.shown) {
        if let Some(mut w) = s.swatches.take() {
            w.show(fe, false);
            s.swatches = Some(w);
        }
        return true;
    }
    match s.stage {
        1 => {
            s.stage -= 1;
            s.show_cars(fe, false, 0);
            s.show_tier(fe, true);
        }
        2 => s.stage -= 1,
        3 => {
            s.stage -= 1;
            return false;
        }
        _ => {
            s.from_garage = false;
            s.exit = 1;
            if let Some(mut o) = fe.overlay.take() {
                overlay::slide_out(&mut o, fe, 5);
                fe.overlay = Some(o);
            }
            if s.player == 1 && fe.previous_screen_crc() == string_crc("QuickRaceCarSelectScreen") {
                set_next_screen(s, 0, string_crc("QuickRaceCarSelectScreen"));
            }
        }
    }
    s.update_camera(fe);
    s.refresh_nav(fe);
    true
}

/// FECarShowroomScreen.cpp 0x002249C0 — Option 1: transmission (stage 1); stage 2 opens the colour swatches.
/// Returns false like the original, so the overlay sees the press too.
fn cs_option1(s: &mut CFEQuickRaceCarSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if fe.online_locked {
        return false;
    }
    s.hide_stats(fe);
    if s.stage == 1 {
        s.toggle_transmission(fe, true);
    } else if s.stage == 2 && !s.swatches.as_ref().is_some_and(|w| w.shown) {
        // the swatches of the car on the spot, starting on its colour
        let (vehicle, hash, brand) = match fe.foyer_vehicles.slots.first().and_then(|v| v.setup.as_ref()) {
            Some(x) => (x.vehicle, x.colours[0].hash, x.colours[0].brand),
            None => (None, 0, 0),
        };
        if let Some(mut w) = s.swatches.take() {
            w.set_vehicle(vehicle);
            w.fill(fe, true, 0, 0);
            w.select(fe, hash, brand);
            w.show(fe, true);
            s.swatches = Some(w);
        }
    }
    false
}

/// FECarShowroomScreen.cpp 0x00224AE0 — Option 2: automod (needs the 3D car, not ported) on race vehicles.
fn cs_option2(s: &mut CFEQuickRaceCarSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if fe.online_locked {
        return false;
    }
    let race = fe.vehicles.as_ref().is_some_and(|db| db.is_race_vehicle(s.vehicle_hash));
    race
}

/// FECarShowroomScreen.cpp 0x00224070 / 0x00224150 — up / down held: scroll the shown statistics list
fn cs_held(s: &mut CFEQuickRaceCarSelectScreen, fe: &mut Feck2, dir: i32) -> bool {
    if fe.online_locked {
        return false;
    }
    if s.stats_shown {
        s.scroll_stats(fe, dir);
    }
    true
}

/// FECarShowroomScreen.cpp 0x00224BE0 — R2 (stages 1 and 2): the statistics list on / off for the car on the spot
/// (else the garage car, else the tier car's default setup), colour swatches closed
fn cs_stats(s: &mut CFEQuickRaceCarSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if fe.online_locked {
        return false;
    }
    if s.stage == 1 || s.stage == 2 {
        if s.swatches.as_ref().is_some_and(|w| w.shown) {
            if let Some(mut w) = s.swatches.take() {
                w.show(fe, false);
                s.swatches = Some(w);
            }
        }
        let Some(mut db) = fe.vehicles.take() else { return true };
        let mut setup = VehicleSetup::new();
        match fe.foyer_vehicles.slots.first().and_then(|v| v.setup.clone()).filter(|x| x.vehicle.is_some()) {
            Some(x) => setup = x,
            None if s.tier == GARAGE_TIER => {
                if let Some(g) = fe.garage.setup(s.car_index as u32) {
                    setup = g.clone();
                }
            }
            None => {
                db.setup_default_vehicle(&mut setup, s.vehicle_hash);
            }
        }
        s.stats_shown = !s.stats_shown;
        if let Some(mut st) = s.stats.take() {
            st.set_setup(setup);
            st.refill(fe, &mut db, false);
            st.scroll(fe, 0);
            st.show(fe, s.stats_shown);
            s.stats = Some(st);
        }
        fe.vehicles = Some(db);
    }
    true
}

impl CFEQuickRaceCarSelectScreen {
    /// `if (+5452) { +5452 = 0; 0x0028AD40(0) }` (Select 0x0022430D, Cancel 0x002247DB, option 1 0x002249F1)
    fn hide_stats(&mut self, fe: &mut Feck2) {
        if self.stats_shown {
            self.stats_shown = false;
            if let Some(mut st) = self.stats.take() {
                st.show(fe, false);
                self.stats = Some(st);
            }
        }
    }

    /// 0x0028AEC0 on the shown list
    fn scroll_stats(&mut self, fe: &mut Feck2, dir: i32) {
        if let Some(mut st) = self.stats.take() {
            st.scroll(fe, dir);
            self.stats = Some(st);
        }
    }

    /// FECarShowroomScreen.cpp 0x002252D0 — stage 0 steps the tier (skipping empty ones), stage 1 the car.
    fn change_tier(&mut self, fe: &mut Feck2, d: i32, player: u32) {
        if self.stage == 3 {
            return;
        }
        if self.stage == 1 {
            self.players = player as u8;
            self.change_car(fe, d, player);
            self.place_highlight(fe, player);
            return;
        }
        if self.stage != 0 {
            return;
        }
        let mut count = 0;
        let mut name = 0;
        // the original loops until a non-empty tier; with d == 0 on an empty tier that never ends (an
        // empty garage at tier 8 would hang it) — step forward instead after one full turn
        let mut tries = 0;
        let mut step = d;
        while count == 0 {
            self.tier += step;
            if self.tier >= 9 {
                self.tier = 0;
            } else if self.tier < 0 {
                self.tier = 8;
            }
            if self.tier == GARAGE_TIER {
                name = GARAGE_STRING;
                count = fe.quick_race.garage.len() as u32; // dword_6F71A4
            } else {
                name = string_crc(&format!("VEHICLE_TIER{}", self.tier + 1));
                count = fe.vehicles.as_ref().map_or(0, |db| {
                    db.count_available_vehicles_in_tier(self.tier as u8 + 1, fe.quick_race.progress, fe.quick_race.all_unlocked)
                });
            }
            tries += 1;
            if count == 0 && step == 0 {
                step = 1;
            }
            if tries > 9 {
                break;
            }
        }
        self.tier_count = count;
        fe.text(self.tier_text).set_string(name);
        self.show_tier_logos(fe);
        self.place_highlight(fe, player);
    }

    /// FEQuickRaceCarSelectScreen.cpp 0x00225A80 — in stage 0 the area model's logo wall shows the tier's cars:
    /// `carlogo_0<n>` = `DisplayLogo_<name>` and `polaroid_0<n>` = `Polaroid_<name>` from the car's VEHICLEF.VCD,
    /// `polaroid_bg_0<n>` = FE.vcd's `polaroid_bg`; the unused slots up to 6 get `general_OFF`
    fn show_tier_logos(&mut self, fe: &mut Feck2) {
        if self.stage != 0 {
            return;
        }
        let count = self.tier_count;
        if count > 6 {
            log::error!("0 && \"MAXIMUM Vehicles in tier exceeded!!!\"");
        }
        let area = logo_wall_area(fe);
        for i in 0..count {
            let Some(db) = fe.vehicles.as_ref() else { break };
            let v = if self.tier == GARAGE_TIER {
                fe.quick_race.garage.get(i as usize).and_then(|&h| db.find_vehicle(h)) // dword_6F71A8
            } else {
                db.get_nth_available_vehicle_in_tier(i, self.tier as u8 + 1, fe.quick_race.progress, fe.quick_race.all_unlocked)
            };
            let Some(v) = v else { continue };
            let (vcd, name) = logo_names(db, v);
            let n = i + 1;
            fe.foyer.set_material_texture(area, &format!("carlogo_0{n}"), &format!("DisplayLogo_{name}"), &vcd);
            fe.foyer.set_material_texture(area, &format!("polaroid_0{n}"), &format!("Polaroid_{name}"), &vcd);
            fe.foyer.set_material_texture(area, &format!("polaroid_bg_0{n}"), "polaroid_bg", r"\FE\FE.vcd");
        }
        for i in count..6 {
            let n = i + 1;
            fe.foyer.set_material_texture(area, &format!("polaroid_0{n}"), "general_OFF", r"\FE\FE.vcd");
            fe.foyer.set_material_texture(area, &format!("polaroid_bg_0{n}"), "general_OFF", r"\FE\FE.vcd");
            fe.foyer.set_material_texture(area, &format!("carlogo_0{n}"), "general_OFF", r"\FE\FE.vcd");
        }
        // 0x00225E84: MTV tint only
        fe.foyer.mtv.push(MtvCall::Start { seconds: f32::from_bits(0x3DCC_CCCD), no_glitch: true });
    }

    /// FECarShowroomScreen.cpp 0x002254D0 — step the car within the tier and show its name.
    fn change_car(&mut self, fe: &mut Feck2, d: i32, player: u32) {
        if self.stage == 0 {
            return;
        }
        self.car_index += d;
        if self.car_index < 0 {
            self.car_index = self.tier_count as i32 - 1;
        }
        if self.car_index > self.tier_count as i32 - 1 {
            self.car_index = 0;
        }
        let Some(db) = fe.vehicles.as_ref() else { return };
        let vehicle = if self.tier == GARAGE_TIER {
            fe.quick_race.garage_selected = self.car_index as u32;
            fe.quick_race.garage.get(self.car_index as usize).and_then(|&h| db.find_vehicle(h))
        } else {
            db.get_nth_available_vehicle_in_tier(self.car_index as u32, self.tier as u8 + 1, fe.quick_race.progress, fe.quick_race.all_unlocked)
        };
        let mut setup = VehicleSetup::new();
        if let Some(v) = vehicle {
            db.setup_default_vehicle(&mut setup, db.vehicles[v].hash);
        }
        if let Some(v) = setup.vehicle {
            self.vehicle_hash = db.vehicles[v].hash;
            fe.quick_race.selected_vehicle = self.vehicle_hash; // dword_6F70DC
            fe.text(self.car_name).set_string(self.vehicle_hash);
            // 0x00289220 car info panel
            if let Some(mut i) = self.info.take() {
                if let Some(mut db) = fe.vehicles.take() {
                    i.fill(fe, &mut db, &setup);
                    fe.vehicles = Some(db);
                }
                self.info = Some(i);
            }
            self.toggle_transmission(fe, false);
            // +4653 / +5165 / +4909: the chosen car on the wall's carlogo_current / polaroid_current
            if let Some(db) = fe.vehicles.as_ref() {
                let (vcd, name) = logo_names(db, v);
                let area = logo_wall_area(fe);
                fe.foyer.set_material_texture(area, "carlogo_current", &format!("DisplayLogo_{name}"), &vcd);
                fe.foyer.set_material_texture(area, "polaroid_current", &format!("Polaroid_{name}"), &vcd);
            }
            // 0x00225820: MTV tint only
            fe.foyer.mtv.push(MtvCall::Start { seconds: f32::from_bits(0x3DCC_CCCD), no_glitch: true });
            let _ = player;
        }
    }

    /// FECarShowroomScreen.cpp 0x002258F0 — drag races are always manual.
    fn toggle_transmission(&mut self, fe: &mut Feck2, toggle: bool) {
        if fe.quick_race.race_type == 5 {
            self.transmission = 1;
        } else if toggle {
            self.transmission = (self.transmission != 1) as u32;
        }
        if let Some(mut i) = self.info.take() {
            i.set_transmission(fe, self.transmission == 0);
            self.info = Some(i);
        }
    }

    /// FECarShowroomScreen.cpp 0x00225170 — arrows and the title bar around the tier / car name.
    fn place_highlight(&mut self, fe: &mut Feck2, player: u32) {
        if player == 1 {
            return;
        }
        let text = if self.stage != 0 { self.car_name } else { self.tier_text };
        let w = fe.text_width(text) as f32;
        let [x, y] = TIER_POS[player as usize];
        if let Some(mut o) = fe.overlay.take() {
            overlay::position_arrows(&mut o, fe, x - 20.0, x + w, y + ARROW_Y, 0);
            fe.overlay = Some(o);
        }
        let width = (w + x + 60.0) as u32;
        fe.e2d(self.title_bg).width = width;
        fe.e2d(self.title_highlight).width = width;
    }

    /// FECarShowroomScreen.cpp 0x00224E80 - camera table entry per stage
    fn camera_index(&self) -> usize {
        match self.stage {
            0 => 7,
            1 => 8,
            _ => 9,
        }
    }

    /// FECarShowroomScreen.cpp 0x00224EC0 - move the camera to the stage's entry
    fn update_camera(&mut self, fe: &mut Feck2) {
        self.prev_camera = self.camera;
        self.camera = self.camera_index();
        fe.foyer.snap(self.prev_camera);
        fe.foyer.move_to(self.camera);
        // 0x00224FE8
        fe.foyer.shake(-1.0, f32::from_bits(0x3CA3_D70A), -1.0);
    }

    /// FECarShowroomScreen.cpp 0x00225890 - put the player's car up (`FEAlloyEditor_Basic` 0x00259D20)
    fn show_vehicle(&mut self, fe: &mut Feck2, player: usize) {
        fe.foyer_vehicles.count = 1;
        if player != 1 || self.from_garage {
            let Some(db) = fe.vehicles.take() else { return };
            let sel = [fe.quick_race.selected_vehicle, fe.quick_race.selected_vehicle];
            let garage = fe.quick_race.garage.clone();
            let from_garage = self.tier == GARAGE_TIER;
            fe.foyer_vehicles.slots.iter_mut().for_each(|v| v.garage = from_garage);
            fe.foyer_vehicles.show(Some(player), &db, sel, &garage, self.car_index.max(0) as usize, &fe.garage);
            fe.vehicles = Some(db);
        }
    }

    /// FECarShowroomScreen.cpp 0x00224E20 - the swatch under the highlighter onto the car
    fn apply_colour(&mut self, fe: &mut Feck2) {
        let Some(mut w) = self.swatches.take() else { return };
        let c = w.current(fe);
        self.swatches = Some(w);
        if let Some(db) = fe.vehicles.take() {
            fe.foyer_vehicles.set_colour(0, &db, c);
            fe.vehicles = Some(db);
        }
    }

    /// FECarShowroomScreen.cpp 0x00225A30
    fn show_tier(&mut self, fe: &mut Feck2, show: bool) {
        fe.el(self.tier_text).set_visible(show);
        if show {
            self.show_tier_logos(fe);
        }
    }

    /// FECarShowroomScreen.cpp 0x00225980
    fn show_cars(&mut self, fe: &mut Feck2, show: bool, player: u32) {
        fe.el(self.car_label).set_visible(show);
        fe.el(self.car_name).set_visible(show);
        if let Some(mut i) = self.info.take() {
            i.show(fe, show);
            self.info = Some(i);
        }
        if show {
            self.change_tier(fe, 0, player);
            self.change_car(fe, 0, player);
        }
    }

    /// FECarShowroomScreen.cpp 0x00225000 — nav prompts per stage.
    fn refresh_nav(&mut self, fe: &mut Feck2) {
        let race = fe.vehicles.as_ref().is_some_and(|db| db.is_race_vehicle(fe.quick_race.selected_vehicle));
        let Some(mut o) = fe.overlay.take() else { return };
        match self.stage {
            0 => {
                overlay::show_nav(&mut o, fe, 2, false);
                overlay::show_nav(&mut o, fe, 3, false);
                overlay::set_arrows_visible(&mut o, fe, true, 0);
            }
            1 => {
                overlay::set_nav_string(&mut o, fe, 2, string_crc("GENERAL_TRANSMISSION"));
                overlay::show_nav(&mut o, fe, 2, true);
                overlay::show_nav(&mut o, fe, 3, false);
                overlay::set_arrows_visible(&mut o, fe, true, 0);
            }
            _ => {
                overlay::set_nav_string(&mut o, fe, 2, string_crc("GENERAL_COLOUR"));
                overlay::show_nav(&mut o, fe, 2, true);
                overlay::show_nav(&mut o, fe, 3, race);
                overlay::set_arrows_visible(&mut o, fe, false, 0);
            }
        }
        fe.overlay = Some(o);
        self.place_highlight(fe, 0);
    }
}

