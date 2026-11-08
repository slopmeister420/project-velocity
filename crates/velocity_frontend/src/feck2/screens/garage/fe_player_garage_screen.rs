//! src\Feck2\Screens\Garage\FEPlayerGarageScreen.cpp — `gPlayerGarageScreen` (`dword_767E28`, 60 bytes; the
//! object code sits in the idb's FENetworkServerDetailsScreen range 0x00246170..0x00247990): one garage car on
//! the modArea background, USE CAR / SELL CAR toggled with left/right and confirmed in a yes/no dialog. Both
//! leave for `HangoutScreen`; back returns to the previous screen.
//!
//! The car comes from the garage car list (the PinkSlip screen object, 0x00245AA0, sets it with `SetCar`
//! 0x00246010 and FE3DVehicles' garage slot 0x00246040 before going here); `jumptoscreen PlayerGarageScreen`
//! starts with the current garage car instead. R2 shows the car statistics list (`+40`, `fe_car_stats_list`).

use std::sync::{Arc, Mutex};

use super::ov;
use crate::feck2::feck2_manager::{visos_colour, Feck2};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay::{self as overlay, B_MENU_HEADER, R_MENU_CONTENT};
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::feck2::screens::common::fe_car_stats_list::CCarStatsList;
use crate::string_manager::string_manager::string_crc;

/// `GENERAL_NO` / `GENERAL_YES` (dialog options 0 / 1)
const GENERAL_NO: u32 = -668354537i32 as u32;
const GENERAL_YES: u32 = -1829768882i32 as u32;
/// the look-at point of the garage cameras (0, 0.7, 0)
pub const CAR_LOOK_AT: [f32; 3] = [0.0, f32::from_bits(1060320051), 0.0];
/// the camera table entry framing the car (vehicle data +0x14 positions, +0x134 look-ats; 24 entries)
pub const CAR_CAMERA: usize = 13;

/// `CFEPlayerGarageScreen` (0x3C bytes)
pub struct CFEPlayerGarageScreen {
    pub screen: ScreenId,     // +0
    pub layer: LayerId,       // +4
    pub loaded: bool,         // +8
    pub sell: bool,           // +12 0 = use, 1 = sell
    pub garage_slot: u32,     // +16
    pub vehicle: u32,         // +20 its hash
    pub exit_countdown: i32,  // +24 frames before leaving, -1 = staying
    pub exit_screen: u32,     // +28 crc or PREVIOUS
    pub use_text: ElementId,  // +32
    pub sell_text: ElementId, // +36
    pub stats: Option<CCarStatsList>, // +40 the R2 statistics list
    pub stats_shown: bool,    // +44
    pub unk48: u32,           // +48
    pub unk52: u32,           // +52
    pub unk56: i32,           // +56
}

/// the car's camera-table entry `i` (eye, look-at) from the vehicle data (+0x38 data, +0x14 positions)
pub fn car_camera(fe: &Feck2, setup: Option<&velocity_physics::vehicle_database::vehicle_database::VehicleSetup>, i: usize) -> ([f32; 3], [f32; 3]) {
    let data = setup.and_then(|s| s.vehicle).and_then(|v| fe.vehicles.as_ref().map(|db| &db.vehicles[v].data.unk_014));
    match data {
        Some(c) => ([c[3 * i], c[3 * i + 1], c[3 * i + 2]], [c[72 + 3 * i], c[72 + 3 * i + 1], c[72 + 3 * i + 2]]),
        None => ([0.0; 3], [0.0; 3]),
    }
}

/// FENetworkServerDetailsScreen.cpp 0x00246170 / 0x002477E0 / 0x002462D0 / 0x00246670 — `FECK2CreatePlayerGarageScreen` (ctor, Init, AssignEvents)
pub fn create_player_garage_screen(fe: &mut Feck2) -> Arc<Mutex<CFEPlayerGarageScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("PlayerGarageScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("PlayerGarageScreen_2DLayer"), ..Default::default() });
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 0, crc: string_crc("usecar"), ..Default::default() };
    d.position[0] = f32::from_bits(1133903872);
    d.position[1] = f32::from_bits(1133903872);
    d.string = string_crc("CARSELECTION_SELECT");
    let use_text = fe.create_element(layer, &d);
    d.kind = ELEMENT_STATIC_TEXT;
    d.font = 0;
    d.crc = string_crc("sellcar");
    d.position[0] = f32::from_bits(1140457472);
    d.position[1] = f32::from_bits(1133903872);
    d.string = string_crc("CARSELECTION_SELL");
    let sell_text = fe.create_element(layer, &d);
    let obj = Arc::new(Mutex::new(CFEPlayerGarageScreen {
        screen,
        layer,
        loaded: false,
        sell: false,
        garage_slot: 0,
        vehicle: 0,
        exit_countdown: -1,
        exit_screen: 0,
        use_text,
        sell_text,
        // 0x002894A0 / 0x00289510 on the overlay's layer (0x002105B0)
        stats: Some(CCarStatsList::new(fe.overlay.as_ref().map_or(layer, |o| o.layer))),
        stats_shown: false,
        unk48: 0,
        unk52: 0,
        unk56: -1,
    }));
    // 0x00246670 — AssignEvents
    fe.map(layer, functor(&obj, pg_enter), ev::ENTER);
    fe.map(layer, functor(&obj, pg_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, pg_start), ev::START);
    fe.map(layer, functor(&obj, pg_toggle), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, pg_toggle), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, pg_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, pg_cancel), ev::CANCEL_PRESS);
    // 0x00247360 / 0x002473A0 / 0x002473E0 / 0x00247420: scroll the statistics list when shown
    for e in [ev::UP_PRESS, ev::UP_HELD] {
        fe.map(layer, functor(&obj, |s, fe, _| pg_scroll(s, fe, 1)), e);
    }
    for e in [ev::DOWN_PRESS, ev::DOWN_HELD] {
        fe.map(layer, functor(&obj, |s, fe, _| pg_scroll(s, fe, -1)), e);
    }
    fe.map(layer, functor(&obj, pg_stats), ev::R2_PRESS);
    fe.map(layer, functor(&obj, pg_update), ev::UPDATE);
    obj
}

impl CFEPlayerGarageScreen {
    /// FENetworkServerDetailsScreen.cpp 0x00246010 — the garage car shown (slot, vehicle hash)
    pub fn set_car(&mut self, slot: u32, vehicle: u32) {
        self.garage_slot = slot;
        self.vehicle = vehicle;
    }
}

/// FENetworkServerDetailsScreen.cpp 0x00246B30 / FE2DOverlay.cpp 0x0027D360 — Enter: camera on this screen, overlay attached, the garage foyer's
/// vehicle file (CARSLOT at the origin facing 270°).
fn pg_enter(s: &mut CFEPlayerGarageScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    // FE3DCamera 0x0026FB60: attached to the screen, up vectors reset, no move running
    fe.foyer.snap_to_target();
    ov(fe, |o, fe| overlay::attach_to_screen(o, fe, Some(s.screen)));
    if !s.loaded {
        fe.element_load(s.use_text);
        fe.element_load(s.sell_text);
        if let Some(mut st) = s.stats.take() {
            st.reset(fe); // 0x00289550
            s.stats = Some(st);
        }
        s.loaded = true;
    }
    fe.garage.vehicle_flags = 8; // byte_6F90D0
    fe.foyer_vehicles.set_foyer("garage");
    let garage = fe.garage.vehicle_flags == 8;
    fe.foyer_vehicles.load("Vehicles", garage);
    fe.foyer_vehicles.garage_index = s.garage_slot as usize;
    ov(fe, |o, _| o.no_up_down_sound = true); // 0x0027D360
    true
}

/// FENetworkServerDetailsScreen.cpp 0x00246D10 / FE3DCamera.cpp 0x00272920 / 0x00272970 — Activate: car up, overlay (garage heading, cash), camera on the
/// car's camera 13 looking at (0, 0.7, 0) with the stick orbit on, modArea background and lights.
fn pg_activate(s: &mut CFEPlayerGarageScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(db) = fe.vehicles.take() {
        let sel = [fe.quick_race.selected_vehicle, fe.quick_race.selected_vehicle];
        let garage = fe.quick_race.garage.clone();
        fe.foyer_vehicles.show(None, &db, sel, &garage, fe.quick_race.garage_selected as usize, &fe.garage);
        fe.vehicles = Some(db);
    }
    let name = fe
        .foyer_vehicles
        .slots
        .first()
        .and_then(|v| v.setup.as_ref())
        .and_then(|st| st.vehicle)
        .and_then(|v| fe.vehicles.as_ref().map(|db| db.vehicles[v].hash))
        .unwrap_or(0);
    ov(fe, |o, fe| {
        overlay::reset(o, fe);
        overlay::set_nav_text(o, fe, 0, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, 1, string_crc("GENERAL_BACK"));
        overlay::set_message(o, fe, 0);
        overlay::show_borders(o, fe);
        overlay::show_cash(o, fe);
        overlay::show_prestige(o, fe);
        overlay::set_heading(o, fe, string_crc("HANGOUT_GARAGE"));
        overlay::set_icon(o, fe);
        fe.text(o.menu_title).set_string(name);
        fe.el(o.menu_title).set_visible(true);
        overlay::enable(o);
    });
    let setup = fe.foyer_vehicles.slots.first().and_then(|v| v.setup.clone());
    let (eye, _) = car_camera(fe, setup.as_ref(), CAR_CAMERA);
    fe.foyer.set_position_look_at(eye, CAR_LOOK_AT);
    fe.foyer.set_fov(f32::from_bits(1103626240));
    fe.foyer.set_foyer("modArea");
    fe.foyer.orbit_reinit = true; // 0x00272920
    fe.foyer.orbit_lock = 3; // 0x00272970
    fe.foyer.orbit_init();
    let stick = camera_stick(fe);
    fe.foyer.orbit(0.0, 0.0, stick);
    fe.foyer.show_background(true, true, 4);
    s.sell = false;
    s.exit_countdown = -1;
    refresh_colours(s, fe);
    // the sell text is right-aligned on 500
    let w = fe.text_width(s.sell_text);
    fe.element_set_position(s.sell_text, (500i64 - w as i64) as f32, f32::from_bits(1133903872), 0.0);
    true
}

/// the `CameraRotateHoriz` / `Vert` values of the first controller (UserInput 0x001E6BC0 on GameInput's mask)
pub fn camera_stick(fe: &Feck2) -> (f32, f32) {
    use crate::feck2::feck2_manager::action;
    let p = &fe.input.pads[0];
    (p[action::CAMERA_ROTATE_HORIZ].value, p[action::CAMERA_ROTATE_VERT].value)
}

/// FENetworkServerDetailsScreen.cpp 0x00246C10 — Start
fn pg_start(s: &mut CFEPlayerGarageScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.exit_screen = 0;
    s.exit_countdown = -1;
    ov(fe, |o, fe| {
        overlay::show_message(o, fe, false);
        menu::set_style(o, fe, 2);
        fe.el(o.borders[R_MENU_CONTENT]).set_visible(false);
        overlay::fade_arrows(o, fe, -128, 1, 0);
        fe.el(o.borders[B_MENU_HEADER]).set_visible(true);
        fe.element_set_position(o.borders[B_MENU_HEADER], 256.0, f32::from_bits(1113325568), 0.0);
        menu::fade_title(o, fe, 128, 0);
        fe.el(o.menu_title).set_visible(true);
    });
    s.unk48 = 0;
    s.unk52 = 0;
    fe.foyer.show_background(true, true, 4);
    true
}

/// FENetworkServerDetailsScreen.cpp 0x00247060 / 0x002470C0 — left / right: use <-> sell when there is more than
/// one car and no dialog
fn pg_toggle(s: &mut CFEPlayerGarageScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let dialog = fe.overlay.as_ref().is_some_and(|o| o.dialog_active);
    if !dialog && fe.quick_race.garage.len() > 1 {
        s.sell = !s.sell;
    }
    refresh_colours(s, fe);
    true
}

/// FENetworkServerDetailsScreen.cpp 0x00247120 — Select: the yes/no dialog, then use or sell the car.
fn pg_select(s: &mut CFEPlayerGarageScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    hide_stats(s, fe);
    let dialog = fe.overlay.as_ref().is_some_and(|o| o.dialog_active);
    if dialog {
        let yes = fe.overlay.as_ref().is_some_and(|o| o.dialog_selection == 1);
        if yes {
            if s.sell {
                sell_car(s, fe);
            } else {
                fe.quick_race.garage_selected = s.garage_slot; // dword_6F71C4
                fe.garage.current_vehicle = s.vehicle; // dword_6F71C0
            }
            s.exit_screen = string_crc("HangoutScreen");
            ov(fe, overlay::start_transition);
            s.exit_countdown = 20;
        }
        ov(fe, |o, fe| overlay::set_dialog_visible(o, fe, false, 0));
    } else {
        let title = if s.sell { string_crc("CARSELECTION_SELL") } else { string_crc("CARSELECTION_SELECT") };
        ov(fe, |o, fe| {
            overlay::show_dialog(o, fe, 2, title, &[GENERAL_NO, GENERAL_YES]);
            overlay::set_dialog_visible(o, fe, true, 0);
        });
    }
    true
}

/// FENetworkServerDetailsScreen.cpp 0x002472D0 — Cancel: back to the previous screen after the transition
fn pg_cancel(s: &mut CFEPlayerGarageScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    hide_stats(s, fe);
    s.exit_screen = string_crc("PREVIOUS");
    ov(fe, overlay::start_transition);
    s.exit_countdown = 20;
    true
}

/// FENetworkServerDetailsScreen.cpp 0x00247360 / 0x002473A0 / 0x002473E0 / 0x00247420 — up / down (pressed or held)
/// scroll the shown statistics list
fn pg_scroll(s: &mut CFEPlayerGarageScreen, fe: &mut Feck2, dir: i32) -> bool {
    if s.stats_shown {
        if let Some(mut st) = s.stats.take() {
            st.scroll(fe, dir);
            s.stats = Some(st);
        }
    }
    true
}

/// `if (+44) { +44 = 0; 0x0028AD40(0) }` (Select 0x0024714E, Cancel 0x002472FC)
fn hide_stats(s: &mut CFEPlayerGarageScreen, fe: &mut Feck2) {
    if s.stats_shown {
        s.stats_shown = false;
        if let Some(mut st) = s.stats.take() {
            st.show(fe, false);
            s.stats = Some(st);
        }
    }
}

/// FENetworkServerDetailsScreen.cpp 0x00247460 — R2: statistics list off, or on with prices for the garage slot's
/// setup (no arrows: `Show(true)` is not called here, rows below 350 stay hidden by the scroll)
fn pg_stats(s: &mut CFEPlayerGarageScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.stats_shown {
        hide_stats(s, fe);
        return true;
    }
    s.stats_shown = true;
    let (Some(mut db), Some(mut st)) = (fe.vehicles.take(), s.stats.take()) else { return true };
    if let Some(setup) = fe.garage.setup(s.garage_slot).cloned() {
        st.set_setup(setup);
    }
    st.refill(fe, &mut db, true);
    st.scroll(fe, 0);
    s.stats = Some(st);
    fe.vehicles = Some(db);
    true
}

/// FENetworkServerDetailsScreen.cpp 0x002474E0 — Update: count the exit down, then leave
fn pg_update(s: &mut CFEPlayerGarageScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.exit_countdown == -1 || s.exit_screen == 0 {
        return true;
    }
    if s.exit_countdown != 0 {
        s.exit_countdown -= 1;
        return true;
    }
    s.exit_countdown = -1;
    if s.exit_screen == string_crc("PREVIOUS") {
        fe.goto_previous_screen(1, 0);
    } else {
        let crc = s.exit_screen;
        fe.goto_screen(crc);
    }
    false
}

/// FENetworkServerDetailsScreen.cpp 0x002475B0 — the chosen option bright, the other dark
fn refresh_colours(s: &CFEPlayerGarageScreen, fe: &mut Feck2) {
    let lit = visos_colour(0x80, 0x60, 0x60, 0x20);
    let dark = visos_colour(0x80, 0x20, 0x20, 0x20);
    let (u, l) = if s.sell { (dark, lit) } else { (lit, dark) };
    fe.text(s.use_text).set_colour(u);
    fe.text(s.sell_text).set_colour(l);
}

/// FEPlayerGarageScreen.cpp 0x00247670 — sell the car: its value and its parts' cost go to the cash, the garage
/// closes up, and the current car falls back to slot 0 when it was this one.
fn sell_car(s: &CFEPlayerGarageScreen, fe: &mut Feck2) {
    debug_assert!(fe.quick_race.garage.len() > 1, "Attempting to sell last car.");
    let idx = s.garage_slot as usize;
    if let (Some(mut db), Some(setup)) = (fe.vehicles.take(), fe.garage.setups.get(idx).cloned()) {
        fe.garage.cash += super::setup_parts_cost(&mut db, &setup, true);
        // + the vehicle entry's +0x10 value
        fe.garage.cash += setup.vehicle.map_or(0, |v| db.vehicles[v].unk_10 as i32);
        fe.vehicles = Some(db);
    }
    ov(fe, overlay::update_cash);
    let count = fe.quick_race.garage.len();
    for i in idx + 1..count {
        fe.garage.setups[i - 1] = fe.garage.setups[i].clone();
    }
    // only the setups close up; the hash list just loses its last entry (original bug, kept)
    fe.quick_race.garage.truncate(count.saturating_sub(1));
    if fe.quick_race.garage_selected == s.garage_slot {
        fe.quick_race.garage_selected = 0;
        fe.garage.current_vehicle = fe
            .garage
            .setups
            .first()
            .and_then(|st| st.vehicle)
            .and_then(|v| fe.vehicles.as_ref().map(|db| db.vehicles[v].hash))
            .unwrap_or(0);
    }
}
