//! `gPinkSlipScreen` (0x20 bytes; FEPlayerGarageScreen.cpp, the code sits in the idb's
//! FENetworkServerDetailsScreen range 0x00244EA0..0x00246060): the garage's car list (hangout garage character,
//! GARAGE_PINKSLIPS). One row per owned car with a two-tone swatch of its body colour; USE CAR / VIEW CAR toggled
//! with left / right; A uses the car (current garage slot, back to the hangout) or opens it in PlayerGarageScreen.

use std::sync::{Arc, Mutex};

use super::fe_player_garage_screen::CFEPlayerGarageScreen;
use super::ov;
use crate::feck2::feck2_manager::{visos_colour, Feck2};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::string_manager::string_manager::string_crc;

/// `CFEPinkSlipScreen`
pub struct CFEPinkSlipScreen {
    pub screen: ScreenId,      // +0
    pub layer: LayerId,        // +4
    pub loaded: bool,          // +8
    pub exit_countdown: i32,   // +12 frames before leaving, -1 = staying
    pub exit_screen: u32,      // +16 crc or PREVIOUS
    pub view: bool,            // +20 0 = use, 1 = view
    pub use_text: ElementId,   // +24 "usecar"
    pub view_text: ElementId,  // +28 "viewcar"
    pub garage: Arc<Mutex<CFEPlayerGarageScreen>>,
}

/// FENetworkServerDetailsScreen.cpp 0x00244EA0 / 0x00245F80 / 0x00244F90 / 0x002452C0 — `FECK2CreatePinkSlipScreen` (ctor, Init, AssignEvents)
pub fn create_pink_slip_screen(fe: &mut Feck2, garage: Arc<Mutex<CFEPlayerGarageScreen>>) -> Arc<Mutex<CFEPinkSlipScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("PinkSlipScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("PinkSlipScreen_2DLayer"), ..Default::default() });
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 0, crc: string_crc("usecar"), ..Default::default() };
    d.position[0] = f32::from_bits(1133903872);
    d.position[1] = f32::from_bits(1133903872);
    d.string = string_crc("CARSELECTION_SELECT");
    let use_text = fe.create_element(layer, &d);
    d.kind = ELEMENT_STATIC_TEXT;
    d.font = 0;
    d.crc = string_crc("viewcar");
    d.position[0] = f32::from_bits(1140457472);
    d.position[1] = f32::from_bits(1133903872);
    d.string = string_crc("CARSELECTION_VIEW");
    let view_text = fe.create_element(layer, &d);
    let obj = Arc::new(Mutex::new(CFEPinkSlipScreen {
        screen,
        layer,
        loaded: false,
        exit_countdown: -1,
        exit_screen: 0,
        view: false,
        use_text,
        view_text,
        garage,
    }));
    // 0x002452C0 — AssignEvents
    fe.map(layer, functor(&obj, ps_enter), ev::ENTER);
    fe.map(layer, functor(&obj, ps_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, ps_start), ev::START);
    fe.map(layer, functor(&obj, ps_toggle), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, ps_toggle), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, ps_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, ps_cancel), ev::CANCEL_PRESS);
    fe.map(layer, functor(&obj, ps_update), ev::UPDATE);
    obj
}

/// FENetworkServerDetailsScreen.cpp 0x002455C0 — Enter: camera on this screen (no move left), overlay attached, the
/// hangout's vehicle file, every car up
fn ps_enter(s: &mut CFEPinkSlipScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    // FE3DCamera 0x0026FB60(screen, 1): ups reset; +224 = 0
    fe.foyer.set_up([0.0, 1.0, 0.0]);
    fe.foyer.set_target_up([0.0, 1.0, 0.0]);
    fe.foyer.vmove = None;
    ov(fe, |o, fe| overlay::attach_to_screen(o, fe, Some(s.screen)));
    fe.foyer_vehicles.set_foyer("hangout");
    let garage = fe.garage.vehicle_flags == 8;
    fe.foyer_vehicles.load("Vehicles", garage);
    show_cars(fe);
    if !s.loaded {
        fe.element_load(s.use_text);
        fe.element_load(s.view_text);
        s.loaded = true;
    }
    true
}

fn show_cars(fe: &mut Feck2) {
    if let Some(db) = fe.vehicles.take() {
        let sel = [fe.quick_race.selected_vehicle, fe.quick_race.selected_vehicle];
        let garage = fe.quick_race.garage.clone();
        fe.foyer_vehicles.show(None, &db, sel, &garage, fe.quick_race.garage_selected as usize, &fe.garage);
        fe.vehicles = Some(db);
    }
}

/// FENetworkServerDetailsScreen.cpp 0x002456B0 / FE3DCharacters.cpp 0x00262380 — Start: the hangout background, the garage character's SELECT camera at once, fov 50, the
/// characters and their props shown
fn ps_start(_s: &mut CFEPinkSlipScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| overlay::show_message(o, fe, false));
    fe.foyer.show_background(true, true, 0);
    let (pos, at, up) = crate::feck2::screens::hangout::fe_hangout_screen::character_camera(fe, 1, 2, 0);
    fe.foyer.snap_to([pos[0], pos[1], pos[2]], [at[0], at[1], at[2]]);
    fe.foyer.set_up([up[0], up[1], up[2]]);
    fe.foyer.set_target_up([up[0], up[1], up[2]]);
    fe.foyer.snap_to_target();
    // +357 = 1
    fe.foyer.set_fov(f32::from_bits(1112014848));
    fe.characters.set_visible(true); // 0x00261510 / 0x002615F0 / 0x002616A0
    fe.characters.drop_props();
    fe.characters.create_props();
    fe.characters.place_props();
    // 0x002739F0 re-places the backgrounds, 0x00273990 puts each prop on the camera's list (foyer.rs draws them)
    true
}

/// FENetworkServerDetailsScreen.cpp 0x00245850 — Activate: the overlay (select / back, cash, prestige, heading
/// HANGOUT_GARAGE, garage icon), every car up, the list, USE CAR chosen, VIEW CAR right-aligned on 500
fn ps_activate(s: &mut CFEPinkSlipScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| {
        overlay::reset(o, fe);
        overlay::set_nav_text(o, fe, 0, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, 1, string_crc("GENERAL_BACK"));
        overlay::set_message(o, fe, 0);
        overlay::show_borders(o, fe);
        overlay::show_cash(o, fe);
        overlay::show_prestige(o, fe);
        overlay::set_heading(o, fe, string_crc("HANGOUT_GARAGE"));
        overlay::set_icon(o, fe); // "FE_Icon_Garage", \FE\FE.vcd
        overlay::enable(o);
    });
    show_cars(fe);
    // FE3DVehicles 0x0025E040(0, 28.0) steer / 0x0025E7D0(0) height (no bounce running)
    let now = fe.stopwatch;
    fe.foyer_vehicles.set_steer(0, 28.0, now);
    build_list(s, fe);
    s.view = false;
    let w = fe.text_width(s.view_text);
    fe.element_set_position(s.view_text, (500i64 - w as i64) as f32, f32::from_bits(1133903872), 0.0);
    refresh_colours(s, fe);
    true
}

/// FENetworkServerDetailsScreen.cpp 0x00245A20 / 0x00245A60 — left / right: use <-> view
fn ps_toggle(s: &mut CFEPinkSlipScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.view = !s.view;
    refresh_colours(s, fe);
    true
}

/// FENetworkServerDetailsScreen.cpp 0x00245AA0 / 0x00246040 / 0x00246010 — Select: view -> the car in PlayerGarageScreen (FE3DVehicles' garage slot,
/// `SetCar`); use -> it becomes the current car (`dword_6F71C4` / `dword_6F71C0`), back to the hangout
fn ps_select(s: &mut CFEPinkSlipScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let slot = fe.overlay.as_ref().map_or(0, |o| o.menu_selected).max(0) as u32;
    // GameVars 0x000C1F90: the slot's setup, its vehicle hash
    let hash = fe
        .garage
        .setup(slot)
        .and_then(|st| st.vehicle)
        .and_then(|v| fe.vehicles.as_ref().map(|db| db.vehicles[v].hash))
        .unwrap_or(0);
    if s.view {
        fe.foyer_vehicles.garage_index = slot as usize;
        s.exit_screen = string_crc("PlayerGarageScreen");
        ov(fe, overlay::start_transition);
        s.exit_countdown = 20;
        s.garage.lock().unwrap().set_car(slot, hash);
    } else {
        fe.quick_race.garage_selected = slot;
        fe.garage.current_vehicle = hash;
        s.exit_screen = string_crc("PREVIOUS");
        ov(fe, overlay::start_transition);
        s.exit_countdown = 20;
    }
    true
}

/// FENetworkServerDetailsScreen.cpp 0x00245BD0 / FEHangoutScreen.cpp 0x0021C890 — Cancel: back next update; the overlay's next Activate is a first one
fn ps_cancel(s: &mut CFEPinkSlipScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.exit_screen = string_crc("PREVIOUS");
    ov(fe, |o, _| o.first_activate = true);
    s.exit_countdown = 1;
    true
}

/// FENetworkServerDetailsScreen.cpp 0x00245C40 — Update: count the exit down, then back, or hide the characters
/// and go to the chosen screen
fn ps_update(s: &mut CFEPinkSlipScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
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
        fe.characters.set_visible(false);
        let crc = s.exit_screen;
        fe.goto_screen(crc);
    }
    false
}

/// FENetworkServerDetailsScreen.cpp 0x00245D40 — the chosen option bright, the other dark
fn refresh_colours(s: &CFEPinkSlipScreen, fe: &mut Feck2) {
    let lit = visos_colour(0x80, 0x60, 0x60, 0x20);
    let dark = visos_colour(0x80, 0x20, 0x20, 0x20);
    let (u, v) = if s.view { (dark, lit) } else { (lit, dark) };
    fe.text(s.use_text).set_colour(u);
    fe.text(s.view_text).set_colour(v);
}

/// FENetworkServerDetailsScreen.cpp 0x00245E00 / GameVars.cpp 0x000C2400 — the list: one entry per owned car (its name) with a 16x16 "white" swatch
/// tinted with the body colour's two D3DCOLORs (setup +188 / +192), window 5, title GARAGE_PINKSLIPS
fn build_list(_s: &mut CFEPinkSlipScreen, fe: &mut Feck2) {
    let n = fe.quick_race.garage.len().min(super::GARAGE_SLOTS);
    let rows: Vec<(u32, u32, u32)> = (0..n)
        .filter_map(|i| {
            let st = fe.garage.setup(i as u32)?;
            let db = fe.vehicles.as_ref()?;
            let v = st.vehicle?;
            let c = &st.colours[0];
            let k = visos_render::car_palette::CarColour::from_record(c.unk_0c, c.colour_a, c.colour_b);
            Some((db.vehicles[v].hash, k.colour_a, k.colour_b))
        })
        .collect();
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        menu::set_window(o, 5);
        fe.text(o.menu_title).set_string(string_crc("GARAGE_PINKSLIPS"));
        for (hash, top, bottom) in rows {
            menu::add_colour_entry(o, fe, hash, true, "\\FE\\FE.vcd", "white", top, bottom, 16, 16);
        }
        menu::show(o, fe, true, false);
        menu::set_active(o, fe, true);
    });
}
