//! src\Feck2\Screens\Showroom\FECarShowroomScreen.cpp — `gCarShowroomScreen` (`dword_75F448`, 4276 bytes; the code
//! sits in the idb's FEHangoutScreen range 0x0021C9C0..0x002201D0). Reached from the hangout's showroom character
//! (CARSHOWROOM_BUY, 0x0021C210).
//!
//! Two modes on the carSel foyer: the **list** (tier with left / right, car with up / down: the overlay menu of the
//! tier's cars, the car's logo and its `Showroom_<car>` picture as the 2D background, unlock criteria for locked /
//! hidden cars) and the **car** (A on a car: the camera flies to the SHOWROOM_SINGLE spot's camera, the car stands on
//! the turntable, its doors open; menu PURCHASE / TESTDRIVE (always disabled) / CHANGECOLOUR). Purchase asks, checks
//! the cash and the six garage slots, buys (`AddGarageVehicle`, unlock evaluation), then offers to make it the current
//! car and, for race cars, to customise it now. Not ported: the "Enable (Hidden) Showroom screenshots" tweak mode
//! (0x0021FFB0, debug), the door part matrices (graphics, see FE3DVehicles 0x0025BF80).

use std::sync::{Arc, Mutex};

use velocity_physics::vehicle_database::vehicle_database_vehicle_bin_loading::VehicleLockState;

use crate::feck2::feck2_manager::{Feck2, COLOUR_BLACK, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::feck2::screens::common::fe_colour_swatches::CColourSwatches;
use crate::feck2::screens::common::fe_tips_screen::background;
use crate::feck2::screens::garage::{ov, GarageVars};
use crate::feck2::screens::hangout::CareerRequest;
use crate::string_manager::string_manager::string_crc;

const GENERAL_YES: u32 = 0x92EF_F14E;
const GENERAL_NO: u32 = 0xD829_B817;
/// dialog titles: buy the car (idCarName, dPrice), the same when already owned, after buying ("make it your
/// current car?"), for race cars ("customise it now?")
const BUY_TITLE: u32 = 0x632C_0669;
const BUY_TITLE_OWNED: u32 = 0xB18E_B24C;
const BOUGHT_TITLE: u32 = -1159746430i32 as u32;
const CUSTOMISE_TITLE: u32 = 0x12C5_0AF2;
/// Activate's ShowDialog option 0 (string not identified)
const ACTIVATE_DIALOG_OPTION: u32 = 0x5147_684E;
/// `dword_75F6D0` (static init 0x0042BA40): the car menu
const CAR_MENU: [&str; 3] = ["CARSHOWROOM_OPTION_PURCHASE", "CARSHOWROOM_OPTION_TESTDRIVE", "CARSHOWROOM_OPTION_CHANGECOLOUR"];
/// the list camera (Activate 0x0021DD54..: +3888 / +3904 / +3920)
const LIST_CAM_POS: [f32; 3] = [f32::from_bits(0xC0A91687), f32::from_bits(0x3FCE147B), f32::from_bits(0xBF000000)];
const LIST_CAM_LOOK_AT: [f32; 3] = [f32::from_bits(0xC0E91687), f32::from_bits(0x3ECCCCCD), f32::from_bits(0xC06D6042)];
const LIST_CAM_UP: [f32; 3] = [0.0, 1.0, 0.0];
/// cars per tier the name tables hold (assert "MAXIMUM Vehicles in tier exceeded!!!")
const MAX_CARS: usize = 10;
/// the carSel background of the FE3DCamera
const CARSEL_BACKGROUND: usize = 5;

/// `CFECarShowroomScreen`
pub struct CFECarShowroomScreen {
    pub screen: ScreenId,                      // +0
    pub layer: LayerId,                        // +4
    pub car_graphic: ElementId,                // +8 "CarGraphic" the car's logo
    pub unlock_text: ElementId,                // +12 "UnlockText"
    pub car_option: i32,                       // +16 the car menu's selection at Select
    pub loaded: bool,                          // +20
    pub swatches: Option<CColourSwatches>,     // +24 mpColourSwatches
    pub owned: bool,                           // +28 the car is in the garage
    pub camera_held: bool,                     // +29 a camera left / right press is held
    pub car_mode: bool,                        // +30 a car is shown (else the list)
    pub menu_built: bool,                      // +31
    pub buy_stage: u8,                         // +32 0 ask, 1 bought, 2 customise?
    pub bought_slot: u32,                      // +33 garage slot of the bought car
    pub doors_open: bool,                      // +34
    pub colour_mode: bool,                     // +36 the swatches are up
    pub tier: i32,                             // +40 0-based
    pub car_index: i32,                        // +44 in the tier
    pub vcds: [String; MAX_CARS],              // +48 `<vehicle path>\VEHICLEF.VCD`
    pub logos: [String; MAX_CARS],             // +2608 `DisplayLogo_<name>`
    pub pictures: [String; MAX_CARS],          // +3248 `Showroom_<name>[_hidden]`
    pub list_cam: [[f32; 3]; 3],               // +3888 position, look-at, up
    pub car_cam: [[f32; 3]; 3],                // +3936 the SHOWROOM_SINGLE spot's camera
    pub camera_moving: bool,                   // +3984
    pub turntable: bool,                       // +3985
    pub saved_rotation: f32,                   // +3988 the car's angle when the list was asked for
    pub default_colour: Option<usize>,         // +3996
    pub exit_countdown: i32,                   // +4268
    pub exit_screen: u32,                      // +4272 crc or PREVIOUS
}

/// FEHangoutScreen.cpp 0x0021C9C0 / 0x0021CAB0 / 0x0021CB70 / 0x0021D010 — `FECK2CreateCarShowroomScreen` (ctor, Init with the
/// "Frontend.Vehicles" tweaks (registered by `fe_tweaks`), AssignEvents)
pub fn create_car_showroom_screen(fe: &mut Feck2) -> Arc<Mutex<CFECarShowroomScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("CarShowroomScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("CarShowroomScreen_2DLayer"), ..Default::default() });
    let mut d = SElementDesc { kind: ELEMENT_2D, dynamic: true, crc: string_crc("CarGraphic"), ..Default::default() };
    d.position[0] = f32::from_bits(1114636288);
    d.position[1] = f32::from_bits(1135214592);
    let car_graphic = fe.create_element(layer, &d);
    d.dynamic = false;
    d.kind = ELEMENT_STATIC_TEXT;
    d.font = 2;
    d.crc = string_crc("UnlockText");
    d.position[0] = f32::from_bits(1116471296);
    d.position[1] = f32::from_bits(1133903872);
    d.string = 0;
    let unlock_text = fe.create_element(layer, &d);
    let obj = Arc::new(Mutex::new(CFECarShowroomScreen {
        screen,
        layer,
        car_graphic,
        unlock_text,
        car_option: 0,
        loaded: false,
        swatches: None,
        owned: false,
        camera_held: false,
        car_mode: false,
        menu_built: false,
        buy_stage: 0,
        bought_slot: 0,
        doors_open: false,
        colour_mode: false,
        tier: 0,
        car_index: 0,
        vcds: Default::default(),
        logos: Default::default(),
        pictures: Default::default(),
        list_cam: [[0.0; 3]; 3],
        car_cam: [[0.0; 3]; 3],
        camera_moving: false,
        turntable: false,
        saved_rotation: 0.0,
        default_colour: None,
        exit_countdown: -1,
        exit_screen: 0,
    }));
    // 0x0021D010 — AssignEvents
    fe.map(layer, functor(&obj, sr_enter), ev::ENTER);
    fe.map(layer, functor(&obj, sr_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, sr_start), ev::START);
    fe.map(layer, functor(&obj, sr_leave), ev::LEAVE);
    fe.map(layer, functor(&obj, |s, fe, _| sr_side(s, fe, 1)), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| sr_side(s, fe, -1)), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| sr_up_down(s, fe, -1)), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| sr_up_down(s, fe, 1)), ev::DOWN_PRESS);
    // 0x0021E000 / 0x0021E020 / 0x0021E040 / 0x0021E060: camera left / right press and release (34 / 37, 36 / 39)
    fe.map(layer, functor(&obj, |s, _, _| {
        s.camera_held = true;
        false
    }), ev::CAM_LEFT_PRESS);
    fe.map(layer, functor(&obj, |s, _, _| {
        s.camera_held = true;
        false
    }), ev::CAM_RIGHT_PRESS);
    fe.map(layer, functor(&obj, |s, _, _| {
        s.camera_held = false;
        false
    }), ev::CAM_LEFT_PRESS + 2);
    fe.map(layer, functor(&obj, |s, _, _| {
        s.camera_held = false;
        false
    }), ev::CAM_RIGHT_PRESS + 2);
    fe.map(layer, functor(&obj, sr_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, sr_cancel), ev::CANCEL_PRESS);
    fe.map(layer, functor(&obj, sr_option1), ev::OPTION1_PRESS);
    // 0x0021EBD0
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    // Start (0x0021D7C0) maps the Update functor 0x0021EF50 each time; mapped once here
    fe.map(layer, functor(&obj, sr_update), ev::UPDATE);
    obj
}

/// the tier's n-th car (`CVehicleDatabase::GetNthVehicleInTier(n, tier + 1)`) and its lock state
fn nth_car(fe: &Feck2, n: i32, tier: i32) -> Option<(usize, VehicleLockState)> {
    let db = fe.vehicles.as_ref()?;
    let v = db.get_nth_vehicle_in_tier(n.max(0) as u32, (tier + 1) as u8)?;
    Some((v, db.get_lock_state(db.vehicles[v].hash, fe.quick_race.progress, fe.quick_race.all_unlocked)))
}

fn count_in_tier(fe: &Feck2, tier: i32) -> u32 {
    fe.vehicles.as_ref().map_or(0, |db| db.count_vehicles_in_tier((tier + 1) as u8))
}

/// FE3DVehicles 0x0025E210 / 0x0025E250 on slot 0
fn set_car_colour(fe: &mut Feck2, colour: Option<usize>) {
    if let Some(db) = fe.vehicles.take() {
        fe.foyer_vehicles.set_colour(0, &db, colour);
        fe.vehicles = Some(db);
    }
}

/// `FE3DVehicles::Show(slot or all)`
fn show_cars(fe: &mut Feck2, slot: Option<usize>) {
    if let Some(db) = fe.vehicles.take() {
        let sel = [fe.quick_race.selected_vehicle, fe.quick_race.selected_vehicle];
        let garage = fe.quick_race.garage.clone();
        fe.foyer_vehicles.show(slot, &db, sel, &garage, fe.quick_race.garage_selected as usize, &fe.garage);
        fe.vehicles = Some(db);
    }
}

/// the setup shown on slot 0 (FE3DVehicles +1824)
fn shown_setup(fe: &Feck2) -> Option<velocity_physics::vehicle_database::vehicle_database::VehicleSetup> {
    fe.foyer_vehicles.slots.first().and_then(|v| v.setup.clone())
}

/// FEHangoutScreen.cpp 0x0021D5F0 — Enter
fn sr_enter(s: &mut CFECarShowroomScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.car_mode {
        fe.foyer_vehicles.set_doors_allowed(true); // 0x00259580(1, 1)
    }
    s.default_colour = None;
    // FE3DCamera 0x0026FB60(screen, 1): ups reset, +349 / +357 / +358 off
    fe.foyer.set_up([0.0, 1.0, 0.0]);
    fe.foyer.set_target_up([0.0, 1.0, 0.0]);
    fe.foyer.snap_to_target();
    ov(fe, |o, fe| overlay::attach_to_screen(o, fe, Some(s.screen)));
    fe.foyer_vehicles.set_foyer("carSel");
    if !s.loaded {
        s.loaded = true;
        fe.element_load(s.car_graphic);
        fe.el(s.car_graphic).set_visible(false);
        fe.element_load(s.unlock_text);
        fe.el(s.unlock_text).set_visible(false);
        fe.text(s.unlock_text).set_colour(COLOUR_BLACK);
        let mut sw = CColourSwatches::init(fe, s.layer, 140, 345);
        sw.load(fe);
        s.swatches = Some(sw);
        s.camera_held = false;
        s.colour_mode = false;
        s.car_mode = false;
        s.tier = 0;
        s.car_index = 0;
        s.doors_open = false;
    }
    let garage = fe.garage.vehicle_flags == 8;
    fe.foyer_vehicles.load("Vehicles", garage);
    // FE3DCamera 0x002739F0: the backgrounds re-placed (areas at `byte_769380`)
    fe.foyer_vehicles.showroom = true; // 0x00220210
    true
}

/// FEHangoutScreen.cpp 0x0021DAE0 / FE3DCamera.cpp 0x002728E0 / 0x00272110 / 0x002729D0 — Activate: every spot's car, the overlay (select / back / close
/// doors, cash, the showroom icon, both arrow pairs), the spot's camera as the car camera, the carSel lights, linear
/// 1.2 s camera moves, fov 50, the camera stick pitching between 0 and 20°
fn sr_activate(s: &mut CFECarShowroomScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    show_cars(fe, None);
    ov(fe, |o, fe| {
        overlay::reset(o, fe);
        overlay::set_nav_text(o, fe, 0, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, 1, string_crc("GENERAL_BACK"));
        overlay::set_nav_text(o, fe, 2, string_crc("CARSHOWROOM_OPTION_CLOSEDOORS"));
        overlay::set_message(o, fe, 0);
        overlay::show_dialog(o, fe, 2, BUY_TITLE, &[ACTIVATE_DIALOG_OPTION, GENERAL_NO]);
        overlay::show_borders(o, fe);
        overlay::show_cash(o, fe);
        overlay::show_prestige(o, fe); // 0x0027CD00
        menu::set_style(o, fe, 2);
        overlay::set_icon(o, fe); // "FE_Icon_Showroom", \FE\FE.vcd
        overlay::show_arrows(o, fe, 0);
        overlay::show_arrows(o, fe, 1);
        overlay::enable(o);
        overlay::set_dialog_visible(o, fe, false, 0);
        overlay::show_message(o, fe, false);
        fe.el(o.total_cash).set_visible(true);
    });
    if let Some(v) = fe.foyer_vehicles.slots.first() {
        s.car_cam = [v.cam_pos, v.cam_look_at, v.cam_up];
    }
    fe.foyer.set_foyer("carSel");
    fe.foyer.move_mode = 1; // 0x002728E0(0)
    fe.foyer.move_time = f32::from_bits(0x3F99999A); // 0x00272110(1.2)
    fe.foyer.end_early = false; // +347
    fe.foyer.set_fov(f32::from_bits(0x42480000));
    // +357 = 0, 0x002729B0(1) (+349)
    fe.foyer.orbit_lock = 2; // 0x00272970(2): the stick pitches only (left / right turn the car)
    fe.foyer.set_pitch_limits(0.0, 20.0f32 * f32::from_bits(0x3C8EFA35)); // VsCameraManager 0x000613D0
    s.list_cam = [LIST_CAM_POS, LIST_CAM_LOOK_AT, LIST_CAM_UP];
    true
}

/// FEHangoutScreen.cpp 0x0021D7C0 — Start: the carSel background, swatches off; back from the customisation screen
/// the car mode is restored on the spot's camera with the doors open, else the list
fn sr_start(s: &mut CFECarShowroomScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| overlay::show_message(o, fe, false));
    fe.foyer.show_background(true, true, CARSEL_BACKGROUND);
    colour_mode(s, fe, false);
    if s.car_mode {
        set_list(s, fe, false);
        let [pos, at, up] = s.car_cam;
        fe.foyer.set_position_look_at(pos, at);
        fe.foyer.set_target_up(up);
        fe.foyer.orbit_from(pos, at);
        fe.foyer_vehicles.set_doors_allowed(true);
        prepare_car(s, fe);
        build_car_menu(s, fe);
        s.doors_open = true;
        set_doors(s, fe, s.doors_open);
        fe.foyer_vehicles.set_doors(0, 0.0);
    } else {
        start_camera_move(s, fe);
        set_list(s, fe, true);
        let [pos, at, up] = s.list_cam;
        fe.foyer.set_position_look_at(pos, at);
        fe.foyer.set_up(up);
        fe.foyer.set_target_up(up);
        fill_names(s, fe);
        change_tier(s, fe, 0);
    }
    fe.foyer.snap_to_target();
    // FE3DCamera 0x00272990(0): +348 off
    ov(fe, |o, fe| menu::set_active(o, fe, true));
    s.camera_moving = false;
    s.turntable = true;
    s.menu_built = false;
    s.buy_stage = 0;
    s.exit_screen = 0;
    s.exit_countdown = -1;
    if !s.car_mode {
        ov(fe, |o, fe| background(o, fe, true, false));
    }
    true
}

/// FEHangoutScreen.cpp 0x0021DDD0 — Leave: going back resets the tier and the car
fn sr_leave(s: &mut CFECarShowroomScreen, _fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.exit_screen != 0 && s.exit_screen == string_crc("PREVIOUS") {
        s.car_index = 0;
        s.tier = 0;
    }
    true
}

/// FEHangoutScreen.cpp 0x0021DE60 / 0x0021DEC0 — left / right: swatch column in the car mode, else the tier
fn sr_side(s: &mut CFECarShowroomScreen, fe: &mut Feck2, dir: i32) -> bool {
    if s.car_mode {
        if s.swatches.as_ref().is_some_and(|w| w.shown) {
            if let Some(mut w) = s.swatches.take() {
                w.move_col(fe, dir);
                s.swatches = Some(w);
            }
            apply_swatch(s, fe);
        }
        false
    } else {
        change_tier(s, fe, dir);
        true
    }
}

/// FEHangoutScreen.cpp 0x0021DF20 / 0x0021DF90 — up / down (not while the camera moves): swatch row, else the car
fn sr_up_down(s: &mut CFECarShowroomScreen, fe: &mut Feck2, dir: i32) -> bool {
    if s.camera_moving {
        return false;
    }
    if s.car_mode {
        if s.swatches.as_ref().is_some_and(|w| w.shown) {
            if let Some(mut w) = s.swatches.take() {
                w.move_row(fe, dir);
                s.swatches = Some(w);
            }
            apply_swatch(s, fe);
        }
    } else {
        change_car(s, fe, dir);
    }
    true
}

/// FEHangoutScreen.cpp 0x0021E080 / 0x0021E9C0 / 0x0021EA20 — Select: in the car mode the car menu (purchase, test drive = the shown car becomes
/// the current one, colour); in the list the highlighted car (unless hidden) goes on the spot and the camera flies
/// to it
fn sr_select(s: &mut CFECarShowroomScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.camera_moving || s.exit_screen != 0 {
        return false;
    }
    s.menu_built = false;
    if s.car_mode {
        let setup = shown_setup(fe);
        s.car_option = fe.overlay.as_ref().map_or(0, |o| o.menu_selected);
        match s.car_option {
            0 => {
                if let Some(setup) = setup {
                    buy(s, fe, setup);
                }
                prepare_car(s, fe);
                build_car_menu(s, fe);
            }
            // 0x0021E9C0: dword_6F724C = FE3DVehicles +1824 unless a car is loading (never here)
            1 => fe.career.requests.push(CareerRequest::CopyShownCar),
            2 => colour_mode(s, fe, !s.colour_mode),
            _ => {}
        }
    } else if let Some((v, lock)) = nth_car(fe, s.car_index, s.tier) {
        if lock != VehicleLockState::Hidden {
            fe.garage.showroom_vehicle = fe.vehicles.as_ref().map_or(0, |db| db.vehicles[v].hash);
            set_list(s, fe, false);
            fe.e2d(s.car_graphic).change_texture_vcd("\\FE\\FE.vcd");
            fe.change_texture(s.car_graphic, "white", true);
            fe.foyer_vehicles.set_doors_allowed(true);
            show_cars(fe, Some(0));
            start_camera_move(s, fe);
            prepare_car(s, fe);
        }
    }
    false
}

/// FEHangoutScreen.cpp 0x0021E2D0 — the purchase dialog: ask (car name and price); yes -> not enough cash / garage
/// full messages, or buy (cash, garage slot, unlocks) and ask about the current car; yes there -> current car; race
/// cars then ask about customising it (yes -> CarCustomisationScreen)
fn buy(s: &mut CFECarShowroomScreen, fe: &mut Feck2, setup: velocity_physics::vehicle_database::vehicle_database::VehicleSetup) {
    let Some(v) = setup.vehicle else { return };
    let (hash, price, race) = match fe.vehicles.as_ref() {
        Some(db) => (db.vehicles[v].hash, db.vehicles[v].unk_10, db.vehicles[v].is_race_vehicle()),
        None => return,
    };
    let (active, yes) = fe.overlay.as_ref().map_or((false, false), |o| (o.dialog_active, o.dialog_selection == 0));
    if !active {
        let owned = s.owned;
        ov(fe, |o, fe| {
            overlay::set_dialog_count(o, fe, 2);
            let title = if owned { BUY_TITLE_OWNED } else { BUY_TITLE };
            overlay::set_dialog_strings(o, fe, title, &[GENERAL_YES, GENERAL_NO]);
            if let Some(h) = fe.text(o.dialog_item).string {
                fe.strings.set_argument(h, string_crc("idCarName"), hash).ok();
                fe.strings.set_argument(h, string_crc("dPrice"), price).ok();
            }
            overlay::set_dialog_visible(o, fe, true, 0);
        });
        return;
    }
    match s.buy_stage {
        0 => {
            if !yes {
                s.buy_stage = 0;
                ov(fe, |o, fe| {
                    overlay::show_message(o, fe, false);
                    overlay::set_dialog_visible(o, fe, false, 0);
                });
            } else if (fe.garage.cash as u32) < price {
                ov(fe, |o, fe| {
                    overlay::set_message_string(o, fe, string_crc("CARSHOWROOM_INSUFFICIENTCASH"));
                    overlay::show_message(o, fe, true);
                    overlay::set_dialog_visible(o, fe, false, 0);
                });
                s.buy_stage = 0;
            } else if fe.quick_race.garage.len() >= 6 {
                ov(fe, |o, fe| {
                    overlay::set_message_string(o, fe, string_crc("CARSHOWROOM_SLOTSFULL"));
                    overlay::show_message(o, fe, true);
                    overlay::set_dialog_visible(o, fe, false, 0);
                });
                s.buy_stage = 0;
            } else {
                fe.garage.cash = (fe.garage.cash as u32).wrapping_sub(price) as i32;
                let mut garage = std::mem::take(&mut fe.quick_race.garage);
                s.bought_slot = fe.garage.add_garage_vehicle(&mut garage, hash, setup) & 0xFF;
                fe.quick_race.garage = garage;
                ov(fe, overlay::update_cash);
                // CEventDatabase::EvaluateUnlocks(0)
                fe.career.requests.push(CareerRequest::EvaluateUnlocks);
                s.owned = true;
                s.buy_stage += 1;
                ov(fe, |o, fe| {
                    overlay::set_dialog_strings(o, fe, BOUGHT_TITLE, &[GENERAL_YES, GENERAL_NO]);
                    overlay::set_dialog_visible(o, fe, true, 0);
                });
            }
        }
        1 => {
            if yes {
                fe.quick_race.garage_selected = s.bought_slot; // dword_6F71C4
                fe.garage.current_vehicle = hash; // dword_6F71C0
                fe.career.requests.push(CareerRequest::CopyShownCar); // dword_6F724C = the setup
            }
            if race {
                s.buy_stage += 1;
                ov(fe, |o, fe| {
                    overlay::set_dialog_strings(o, fe, CUSTOMISE_TITLE, &[GENERAL_YES, GENERAL_NO]);
                    overlay::set_dialog_visible(o, fe, true, 0);
                });
            } else {
                s.buy_stage = 0;
                ov(fe, |o, fe| overlay::set_dialog_visible(o, fe, false, 0));
            }
        }
        2 => {
            ov(fe, |o, fe| overlay::set_dialog_visible(o, fe, false, 0));
            s.buy_stage = 0;
            if yes {
                go_customise(s, fe);
            }
        }
        _ => {}
    }
}

/// FEHangoutScreen.cpp 0x0021EAE0 — to the customisation screen with the bought car's slot
fn go_customise(s: &mut CFECarShowroomScreen, fe: &mut Feck2) {
    s.exit_screen = string_crc("CarCustomisationScreen");
    ov(fe, overlay::start_transition);
    // dword_6F90C8 = the vehicle of FE3DVehicles +1268, dword_6F90C4 = the bought slot
    fe.garage.custom_vehicle = shown_setup(fe)
        .and_then(|st| st.vehicle)
        .and_then(|v| fe.vehicles.as_ref().map(|db| db.vehicles[v].hash))
        .unwrap_or(0);
    fe.garage.custom_slot = s.bought_slot;
    s.exit_countdown = 20;
}

/// FEHangoutScreen.cpp 0x0021E6C0 — Cancel: colour mode -> the default colour back; else close the dialog, or back to
/// the list (the logo back on), or leave for the previous screen
fn sr_cancel(s: &mut CFECarShowroomScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.camera_moving {
        return false;
    }
    if s.colour_mode {
        set_car_colour(fe, s.default_colour);
        colour_mode(s, fe, false);
        return false;
    }
    s.menu_built = false;
    if fe.overlay.as_ref().is_some_and(|o| o.dialog_active) {
        ov(fe, |o, fe| overlay::set_dialog_visible(o, fe, false, 0));
        s.buy_stage = 0;
    } else if s.car_mode {
        let i = s.car_index.clamp(0, MAX_CARS as i32 - 1) as usize;
        fe.e2d(s.car_graphic).change_texture_vcd(&s.vcds[i]);
        let logo = s.logos[i].clone();
        fe.change_texture(s.car_graphic, &logo, true);
        fe.e2d(s.car_graphic).set_size(256, 64);
        set_list(s, fe, true);
    } else {
        fe.el(s.car_graphic).set_visible(false);
        fe.e2d(s.car_graphic).change_texture_vcd("\\FE\\FE.vcd");
        fe.change_texture(s.car_graphic, "white", true);
        s.exit_screen = string_crc("PREVIOUS");
        ov(fe, overlay::start_transition);
        s.exit_countdown = 20;
    }
    true
}

/// FEHangoutScreen.cpp 0x0021E960 — option 1 in the car mode: open / close the doors
fn sr_option1(s: &mut CFECarShowroomScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if !s.car_mode {
        return false;
    }
    s.doors_open = !s.doors_open;
    set_doors(s, fe, s.doors_open);
    true
}

/// FEHangoutScreen.cpp 0x0021E8A0 — the doors prompt and FE3DVehicles 0x0025D9A0 on slot 0
fn set_doors(_s: &mut CFECarShowroomScreen, fe: &mut Feck2, open: bool) {
    let crc = if open { string_crc("CARSHOWROOM_OPTION_CLOSEDOORS") } else { string_crc("CARSHOWROOM_OPTION_OPENDOORS") };
    ov(fe, |o, fe| overlay::set_nav_string(o, fe, 2, crc));
    let now = fe.stopwatch;
    fe.foyer_vehicles.open_doors(0, open, now);
}

/// FEHangoutScreen.cpp 0x0021EF50 — Update: the exit countdown, the car up once loaded (turntable on, the 2D picture
/// off), the camera step, then the turntable or the ease back to the spot's angle
fn sr_update(s: &mut CFECarShowroomScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.exit_countdown != -1 && s.exit_screen != 0 {
        if s.exit_countdown == 0 {
            fe.foyer_vehicles.set_doors_allowed(false); // 0x00259580(0, 0)
            s.exit_countdown = -1;
            if s.exit_screen == string_crc("PREVIOUS") {
                fe.goto_previous_screen(1, 0);
                s.car_mode = false;
            } else {
                let crc = s.exit_screen;
                fe.goto_screen(crc);
            }
            return false;
        }
        s.exit_countdown -= 1;
    }
    if s.car_mode {
        // FE3DVehicles +15692 = 1
        if fe.foyer_vehicles.slots.first().is_some_and(|v| v.loaded) && !s.menu_built {
            fe.foyer_vehicles.set_visible(0, true);
            s.turntable = true;
            ov(fe, |o, fe| background(o, fe, false, false));
        }
    }
    camera_step(s, fe);
    if s.turntable {
        fe.foyer_vehicles.set_turntable(true);
    } else {
        fe.foyer_vehicles.set_turntable(false);
        let base = fe.foyer_vehicles.slots.first().map_or(0.0, |v| v.base_rotation);
        let mut d = base - s.saved_rotation;
        if d < -180.0 {
            d += 360.0;
        } else if d > 180.0 {
            d -= 360.0;
        }
        // sin(1 - left * 90°): the camera move's time left in seconds, as the original
        let k = ((1.0 - fe.foyer.move_left() * 90.0) * f32::from_bits(0x3C8EFA35)).sin();
        let now = fe.stopwatch;
        fe.foyer_vehicles.set_rotation(0, d * k + base, now);
    }
    true
}

/// FEHangoutScreen.cpp 0x0021EBF0 — while the camera moves: the move between the list and car cameras (started when
/// none runs), the end within 0.1² of the target (car: doors open, orbit; list: the 2D picture, the car hidden), and
/// the menu built and slid in after half the move, out before
fn camera_step(s: &mut CFECarShowroomScreen, fe: &mut Feck2) {
    if !s.camera_moving {
        return;
    }
    let (from, to) = if s.car_mode { (s.list_cam, s.car_cam) } else { (s.car_cam, s.list_cam) };
    fe.foyer.set_up(from[2]);
    fe.foyer.set_target_up(to[2]);
    if s.car_mode {
        fe.foyer.orbit_from(to[0], to[1]);
    } else {
        s.saved_rotation = fe.foyer_vehicles.rotation(0);
        s.turntable = false;
        let now = fe.stopwatch;
        fe.foyer_vehicles.open_doors(0, false, now);
    }
    let eye = fe.foyer.eye().0;
    let d = [eye[0] - to[0][0], eye[1] - to[0][1], eye[2] - to[0][2]];
    if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] < f32::from_bits(0x3DCCCCCD) {
        s.camera_moving = false;
        ov(fe, |o, fe| menu::set_active(o, fe, true));
        if s.car_mode {
            s.doors_open = true;
            set_doors(s, fe, true);
            let stick = crate::feck2::screens::garage::fe_player_garage_screen::camera_stick(fe);
            fe.foyer.orbit(0.0, 0.0, stick);
        } else {
            ov(fe, |o, fe| background(o, fe, true, false));
            fe.foyer_vehicles.set_visible(0, false);
        }
    }
    if fe.foyer.move_left() <= 0.0 && s.camera_moving {
        fe.foyer.start_vector_move((from[0], from[1]), (to[0], to[1]));
    }
    if fe.foyer.move_left() < fe.foyer.move_time / 2.0 {
        if !s.menu_built {
            if s.car_mode {
                build_car_menu(s, fe);
            } else {
                build_list_menu(s, fe);
            }
        }
        ov(fe, |o, fe| {
            menu::slide_border(o, fe, 4);
            menu::fade(o, fe, 4, 0);
        });
    } else {
        ov(fe, |o, fe| {
            menu::slide_border(o, fe, -4);
            menu::fade(o, fe, -4, 0);
        });
    }
}

/// FEHangoutScreen.cpp 0x0021F290 — the shown car: its default colour, the swatches' vehicle, owned (logo off)
fn prepare_car(s: &mut CFECarShowroomScreen, fe: &mut Feck2) {
    let setup = shown_setup(fe);
    let vehicle = setup.as_ref().and_then(|st| st.vehicle);
    s.default_colour = fe.vehicles.as_ref().and_then(|db| db.get_default_vehicle_colour(vehicle));
    if let Some(w) = s.swatches.as_mut() {
        w.set_vehicle(vehicle);
    }
    let hash = vehicle.and_then(|v| fe.vehicles.as_ref().map(|db| db.vehicles[v].hash)).unwrap_or(0);
    if GarageVars::in_garage(&fe.quick_race.garage, hash) {
        s.owned = true;
        fe.el(s.car_graphic).set_visible(false);
    } else {
        s.owned = false;
    }
    s.car_option = 0;
}

/// FEHangoutScreen.cpp 0x0021F340 — a camera move pending, the menu title shown
fn start_camera_move(s: &mut CFECarShowroomScreen, fe: &mut Feck2) {
    s.camera_moving = true;
    ov(fe, |o, fe| menu::fade_title(o, fe, 128, 0));
}

/// FEHangoutScreen.cpp 0x0021F390 — the colour swatches on (starting on the car's colour) or off; the menu the other
/// way; the title CHANGECOLOUR
fn colour_mode(s: &mut CFECarShowroomScreen, fe: &mut Feck2, on: bool) {
    s.colour_mode = on;
    if let Some(mut w) = s.swatches.take() {
        if on {
            w.fill(fe, true, 0, 0);
            w.show(fe, true);
            if let Some(st) = shown_setup(fe) {
                w.select(fe, st.colours[0].hash, st.colours[0].brand);
            }
        } else {
            w.show(fe, false);
        }
        s.swatches = Some(w);
    }
    ov(fe, |o, fe| {
        menu::show(o, fe, !on, false);
        menu::set_active(o, fe, !on);
        fe.text(o.menu_title).set_string(string_crc("CARSHOWROOM_OPTION_CHANGECOLOUR"));
        fe.el(o.menu_title).set_visible(true);
    });
}

/// FEHangoutScreen.cpp 0x0021F500 — the highlighted swatch onto the car
fn apply_swatch(s: &mut CFECarShowroomScreen, fe: &mut Feck2) {
    let Some(mut w) = s.swatches.take() else { return };
    let c = w.current(fe);
    s.swatches = Some(w);
    set_car_colour(fe, c);
}

/// FEHangoutScreen.cpp 0x0021F560 — `SetMode(list)`: the camera moves; the list (left / right arrows, the tier's
/// names, the default colour back, no doors prompt) or the car (doors shut, prompt shown)
fn set_list(s: &mut CFECarShowroomScreen, fe: &mut Feck2, list: bool) {
    s.car_mode = !list;
    start_camera_move(s, fe);
    ov(fe, |o, fe| {
        overlay::set_arrows_visible(o, fe, list, 0);
        overlay::show_message(o, fe, false);
    });
    if list {
        change_tier(s, fe, 0);
        fill_names(s, fe);
        set_car_colour(fe, s.default_colour);
        ov(fe, |o, fe| overlay::show_nav(o, fe, 2, false));
    } else {
        prepare_car(s, fe);
        s.doors_open = false;
        set_doors(s, fe, false);
        ov(fe, |o, fe| overlay::show_nav(o, fe, 2, true));
        fe.foyer_vehicles.set_doors(0, 0.0);
    }
}

/// FEHangoutScreen.cpp 0x0021F650 — next tier with cars (all of a tier count when "Unlock ALL Vehicles"), clamped to
/// 0..7 (a clamp keeps the old tier); a change starts on its first car and shows the picture
fn change_tier(s: &mut CFECarShowroomScreen, fe: &mut Feck2, dir: i32) {
    let mut going = true;
    let mut count = 0;
    let old = s.tier;
    while count == 0 && going {
        s.tier += dir;
        if s.tier < 0 {
            s.tier = 0;
            going = false;
        } else if s.tier >= 8 {
            s.tier = 7;
            going = false;
        }
        count = fe.vehicles.as_ref().map_or(0, |db| {
            if fe.quick_race.all_unlocked {
                db.count_vehicles_in_tier((s.tier + 1) as u8)
            } else {
                db.count_available_vehicles_in_tier((s.tier + 1) as u8, fe.quick_race.progress, false)
            }
        });
        // with dir 0 an empty tier loops forever in the original
        if dir == 0 {
            break;
        }
    }
    if going {
        if dir != 0 {
            s.car_index = 0;
        }
    } else {
        s.tier = old;
    }
    fill_names(s, fe);
    change_car(s, fe, 0);
    build_list_menu(s, fe);
    if dir != 0 {
        ov(fe, |o, fe| background(o, fe, true, false));
    }
}

/// FEHangoutScreen.cpp 0x0021F780 — next car in the tier (clamped): its logo, the unlock criteria for locked (prestige)
/// and hidden (half the prestige) cars, its picture as the 2D background (shown on a move only)
fn change_car(s: &mut CFECarShowroomScreen, fe: &mut Feck2, dir: i32) {
    s.car_index += dir;
    let n = count_in_tier(fe, s.tier) as i32;
    if s.car_index < 0 {
        s.car_index = 0;
    } else if s.car_index > n - 1 {
        s.car_index = n - 1;
    }
    let i = s.car_index.clamp(0, MAX_CARS as i32 - 1) as usize;
    fe.e2d(s.car_graphic).change_texture_vcd(&s.vcds[i]);
    let logo = s.logos[i].clone();
    fe.change_texture(s.car_graphic, &logo, true);
    fe.e2d(s.car_graphic).set_size(256, 64);
    let car = nth_car(fe, s.car_index, s.tier);
    let unlock = car.and_then(|(v, _)| fe.vehicles.as_ref().map(|db| db.vehicles[v].unlock_value)).unwrap_or(0);
    let lock = car.map(|c| c.1);
    let set_prestige = |fe: &mut Feck2, id: ElementId, p: u32| {
        fe.text(id).set_string(string_crc("CARSHOWROOM_UNLOCKCRITERIA_PRESTIGE"));
        if let Some(h) = fe.text(id).string {
            fe.strings.set_argument(h, string_crc("dPrestige"), p).ok();
        }
    };
    if lock == Some(VehicleLockState::Hidden) {
        fe.el(s.car_graphic).set_visible(false);
        fe.el(s.unlock_text).set_visible(true);
        set_prestige(fe, s.unlock_text, unlock >> 1);
    } else {
        fe.el(s.car_graphic).set_visible(true);
        if lock == Some(VehicleLockState::Locked) {
            fe.el(s.unlock_text).set_visible(true);
            set_prestige(fe, s.unlock_text, unlock);
        } else {
            fe.el(s.unlock_text).set_visible(false);
        }
    }
    let (pic, vcd) = (s.pictures[i].clone(), s.vcds[i].clone());
    ov(fe, |o, fe| {
        overlay::set_2d_background(o, fe, &pic, &vcd, false, COLOUR_WHITE);
        if dir == 0 {
            background(o, fe, false, false);
        }
    });
}

/// FECarShowroomScreen.cpp 0x0021FA60 — per car of the tier: `<path>\VEHICLEF.VCD`, `DisplayLogo_<name>`,
/// `Showroom_<name>` (+ `_hidden` for hidden cars)
fn fill_names(s: &mut CFECarShowroomScreen, fe: &mut Feck2) {
    let n = count_in_tier(fe, s.tier);
    for k in 0..n {
        debug_assert!(n <= MAX_CARS as u32, "0 && \"MAXIMUM Vehicles in tier exceeded!!!\"");
        let Some((v, lock)) = nth_car(fe, k as i32, s.tier) else { continue };
        let Some(db) = fe.vehicles.as_ref() else { return };
        let e = &db.vehicles[v];
        let name = e.name.clone().unwrap_or_default();
        let k = (k as usize).min(MAX_CARS - 1);
        s.vcds[k] = format!("{}\\VEHICLEF.VCD", e.path);
        s.logos[k] = format!("DisplayLogo_{name}");
        s.pictures[k] = format!("Showroom_{name}");
        if lock == VehicleLockState::Hidden {
            s.pictures[k].push_str("_hidden");
        }
    }
}

/// FECarShowroomScreen.cpp 0x0021FC30 — the list menu: the tier's cars (hidden ones GENERAL_HIDDEN, disabled), on
/// the current car, title VEHICLE_TIER<n>; active unless the camera moves
fn build_list_menu(s: &mut CFECarShowroomScreen, fe: &mut Feck2) {
    s.menu_built = true;
    let n = count_in_tier(fe, s.tier);
    let entries: Vec<(u32, bool)> = (0..n)
        .map(|k| match nth_car(fe, k as i32, s.tier) {
            Some((_, VehicleLockState::Hidden)) | None => (string_crc("GENERAL_HIDDEN"), false),
            Some((v, _)) => (fe.vehicles.as_ref().map_or(0, |db| db.vehicles[v].hash), true),
        })
        .collect();
    let (sel, moving, tier) = (s.car_index, s.camera_moving, s.tier);
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        menu::set_style(o, fe, 2);
        menu::set_window(o, -1);
        for (crc, enabled) in entries {
            menu::add_entry(o, fe, crc, enabled, false, 0);
        }
        menu::set_selection(o, fe, sel);
        menu::show(o, fe, true, false);
        menu::set_active(o, fe, !moving);
        // VsStringAnsi::Hash("VEHICLE_TIER%d")
        fe.text(o.menu_title).set_string(string_crc(&format!("VEHICLE_TIER{}", tier + 1)));
    });
}

/// FECarShowroomScreen.cpp 0x0021FE30 — the car menu: PURCHASE (disabled while locked), TESTDRIVE (always disabled),
/// CHANGECOLOUR (disabled for tier 7, the team cars); the logo off; title = the car's name
fn build_car_menu(s: &mut CFECarShowroomScreen, fe: &mut Feck2) {
    s.menu_built = true;
    let car = nth_car(fe, s.car_index, s.tier);
    let locked = car.is_some_and(|c| c.1 == VehicleLockState::Locked);
    let team = car.is_some_and(|(v, _)| fe.vehicles.as_ref().is_some_and(|db| db.vehicles[v].data.tier == 7));
    let name = shown_setup(fe)
        .and_then(|st| st.vehicle)
        .and_then(|v| fe.vehicles.as_ref().map(|db| db.vehicles[v].hash))
        .unwrap_or(0);
    let moving = s.camera_moving;
    let graphic = s.car_graphic;
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        menu::set_style(o, fe, 0);
        menu::set_window(o, 5);
        for (i, e) in CAR_MENU.iter().enumerate() {
            menu::add_entry(o, fe, string_crc(e), i != 1, false, 0);
        }
        if locked {
            menu::set_entry_enabled(o, fe, 0, false);
        }
        if team {
            menu::set_entry_enabled(o, fe, 2, false);
        }
        fe.el(graphic).set_visible(false);
        menu::show(o, fe, true, false);
        menu::set_active(o, fe, !moving);
        fe.text(o.menu_title).set_string(name);
    });
}
