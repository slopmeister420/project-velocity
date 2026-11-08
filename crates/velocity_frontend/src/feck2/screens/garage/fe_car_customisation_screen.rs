//! src\Feck2\Screens\Garage\FECarCustomisationScreen.cpp — `gpCarCustomisationScreen` (`dword_7680B0`, 0x9E3C
//! bytes; the object code is labelled FEPlayerGarageScreen 0x00247990..0x0024F680 and FECarCustomisationScreen
//! 0x0024F680..0x00254130 in the idb). The garage car (`dword_6F90C4`) on the modArea background (CUSTOMCAR spot),
//! browsed with the overlay's icon carousel:
//!
//! level 1 Performance / Styling / Colour -> level 2 groups -> level 3 items -> (level 4 paint brand) -> level 5
//! part list or colour swatches -> level 6 the buy / sell-back dialog. Browsing puts each part on the car at once;
//! cancel restores the setup copy (`+39804`), buying keeps it and Leave writes it back to the garage slot.
//!
//! Reached from the hangout ("THE SCENE", not ported) or `jumptoscreen CarCustomisationScreen` (byte_6F905A, the
//! original's debug route; with it, backing out of level 1 stays on the screen). Not ported: decals (styling group
//! 2, 0x00250C20..0x002513F0), the mix-your-own colour sliders
//! (0x002505F0 / 0x0024B3E0 / 0x0024B480), the debug LOD / colour tweak sliders (L2 / R2 / option 1, 0x002527A0,
//! 0x00252830, 0x002539A0..0x00253CD0), submission screenshots (0x002528D0), team discounts (`dword_6F71D0`),
//! the performance-kit price (`GetSetupCost`, 0x0018AB70) and wheel / tyre / brake models on the car.

use std::sync::{Arc, Mutex};

use velocity_physics::vehicle_database::vehicle_database::{CVehicleDatabase, VehicleSetup};

use super::fe_player_garage_screen::{camera_stick, car_camera, CAR_LOOK_AT};
use super::ov;
use crate::feck2::feck2_manager::{visos_colour, Feck2, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::feck2::screens::common::fe_colour_swatches::CColourSwatches;
use crate::osd::speedo::HudConfig;
use crate::string_manager::string_manager::string_crc;
use velocity_game::timeline_manager::race_timeline_sequencer::vs_string_hash;

const FE_VCD: &str = r"\FE\FE.vcd";
const CUSTOM_VCD: &str = r"\FE\FECustom.vcd";
const GENERAL_YES: u32 = -1829768882i32 as u32;
const GENERAL_NO: u32 = -668354537i32 as u32;
/// dialog title from Activate (0x0024B1AE) and the purchase dialog strings (0x0024F860)
const DIALOG_TITLE: u32 = -1908358031i32 as u32;
const DLG_SELL_BACK: u32 = 1678949290;
const DLG_CONFIRM: u32 = -819054681i32 as u32; // CARCUSTOMISATION_CONFIRM
const DLG_REPLACE: u32 = -950733114i32 as u32;
const DLG_BUY: u32 = -1683722865i32 as u32;
const ARG_PART_NAME: u32 = -1049120924i32 as u32; // idPartName
const ARG_REFUND: u32 = -1419631058i32 as u32;
const ARG_OLD_PART: u32 = -1059320120i32 as u32;
const ARG_CASH_DIFFERENCE: u32 = -522324960i32 as u32; // dCashDifference
const NOT_ENOUGH_CASH: u32 = -1860407723i32 as u32; // CARCUSTOMISATION_NOTENOUGHCASH

/// `off_6C1390` level-1 icons, `dword_768390` their strings (static init 0x0042E800)
const TOP_ICONS: [&str; 3] = ["mod-icon_performance", "mod-icon_styling", "mod-icon_colours"];
const TOP_STRINGS: [&str; 3] = ["CARCUSTOMISATION_CAT_PERFORMANCE", "CARCUSTOMISATION_CAT_STYLING", "CARCUSTOMISATION_CAT_COLOUR"];
/// level-2 strings: `dword_768354` performance, `dword_768368` styling, `dword_768340` colour
const PERF_GROUPS: [&str; 5] = [
    "CARCUSTOMISATION_PERF_ACCELERATION", "CARCUSTOMISATION_PERF_BRAKES", "CARCUSTOMISATION_PERF_TOPSPEED",
    "CARCUSTOMISATION_PERF_HANDLING", "CARCUSTOMISATION_PERF_KIT",
];
const STYLING_GROUPS: [&str; 5] = [
    "CARCUSTOMISATION_STYL_BODY", "CARCUSTOMISATION_STYL_WHEELS", "CUSTOM_STYL_BODY_DECAL", "CARCUSTOMISATION_STYL_GAUGES",
    "CARCUSTOMISATION_STYL_STEREO",
];
const COLOUR_GROUPS: [&str; 5] = [
    "CARCUSTOMISATION_STYL_COLOUR", "CUSTOM_STYL_BODY_NEON", "CUSTOM_PERF_BRAKES_PADS", "CARCUSTOMISATION_COLOUR_ALLOYS",
    "CARCUSTOMISATION_COLOUR_WINDOWS",
];
/// `dword_76837C` paint items, `off_6C13D8` their icons
const PAINT_ITEMS: [&str; 5] = ["CAR_COLOURS_MANUFACTURE", "CAR_COLOURS_GLOSS", "CAR_COLOURS_METALLIC", "CAR_COLOURS_PEARL", "CAR_COLOURS_CUSTOM"];
const PAINT_ICONS: [&str; 5] = ["mod-icon_paint", "mod-icon_gloss", "mod-icon_metallic", "mod-icon_pearlescent", "mod-icon_mix_your_own"];

/// part types beyond the 14 bodypart types
const TYPE_BODY_KIT: u32 = 15;
const TYPE_PERF_KIT: u32 = 16;
const TYPE_WHEEL: u32 = 17;
const TYPE_BRAKE: u32 = 18;
const TYPE_TYRE: u32 = 19;
/// the eight decal items' part types (0x1B..0x22)
const TYPE_DECAL_FIRST: u32 = 27;
const TYPE_DECAL_LAST: u32 = 34;

/// one 284-byte item record: icon position, string, count (0 = disabled), part type, icon name
#[derive(Clone, Debug, Default)]
pub struct CustomItem {
    pub pos: [f32; 2],  // +0
    pub string: u32,    // +16
    pub count: u32,     // +20
    pub part_type: u32, // +24
    pub icon: String,   // +28
}

/// `CFECarCustomisationScreen` (0x9E3C bytes)
pub struct CFECarCustomisationScreen {
    pub screen: ScreenId,             // +0
    pub layer: LayerId,               // +4
    pub part_cost: ElementId,         // +8 partcosttext
    pub locked_icon: ElementId,       // +12
    pub perf_back_current: ElementId, // +16
    pub perf_back_installed: ElementId, // +20
    pub perf_level_current: ElementId, // +24
    pub perf_level_installed: ElementId, // +28
    pub perf_info: ElementId,         // +32 performanceinfotext
    pub slider_highlight: ElementId,  // +36
    pub colour_slider: ElementId,     // +40
    pub colour_arrow: ElementId,      // +44
    pub gradient_slider: ElementId,   // +48
    pub gradient_arrow: ElementId,    // +52
    pub lod_label: ElementId,         // +56
    pub lod_string: ElementId,        // +60
    pub tweak_sliders: [ElementId; 6], // +64
    pub tweak_values: [ElementId; 6], // +88
    pub decal_status: ElementId,      // +216
    pub loaded: bool,                 // +220
    pub cam_eyes: [[f32; 3]; 24],     // +224 the car's camera table, y + 0.35
    pub cam_looks: [[f32; 3]; 24],    // +608
    pub mix_shade: f32,               // +1360 mix-your-own shade -255..255
    pub mix_hue: f32,                 // +1364 hue 0..1529
    pub mix_saved_shade: f32,         // +1368 as bought (0x002504A0)
    pub mix_saved_hue: f32,           // +1372
    pub colour_slider_mode: bool,     // +1264
    pub swatches_shown: bool,         // +1265
    pub decal_mode: bool,             // +1266
    pub brakepad_mode: bool,          // +1267
    pub gauges_mode: bool,            // +1268
    pub decal_count: i32,             // +1272 decals of the chosen kind
    pub decal_sel: i32,               // +1276 chosen decal (0 = off, else index + 1)
    pub decal_slot: [i32; 8],         // +40400 + 4 * item: the slot being edited per decal item
    pub gauge_selected: i32,          // +1376 gauges list selection
    pub gauge_hashes: Vec<u32>,       // +40504 VsStringAnsi::Hash of each listed hud
    pub icon_base: [f32; 2],          // +1344
    pub part_hash: u32,               // +1380 the part under the cursor
    pub perf: Vec<CustomItem>,        // +1384 5 groups x 3
    pub styling: Vec<CustomItem>,     // +5644 5 groups x 11
    pub colour: Vec<CustomItem>,      // +36884 5 groups x 2 (the paint group's 5 items run into the next two)
    pub perf_counts: [u32; 5],        // +39724
    pub styling_counts: [u32; 5],     // +39744
    pub colour_counts: [u32; 5],      // +39784
    pub setup_copy: VehicleSetup,     // +39804 the garage setup as bought
    pub bodypart_set: Option<usize>,  // +40364
    pub swatches: CColourSwatches,    // +40372
    pub sel: [i32; 7],                // +40376 selection per level: [1] top, [2] group, [3] item, [4] brand, [5] list, [6] list as bought
    pub slider_row: i32,              // +40404
    pub level: i32,                   // +40432
    pub sub_count: u32,               // +40440
    pub part_count: u32,              // +40444
    pub part_type: u32,               // +40448
    pub default_wheel: u32,           // +40452
    pub exit_countdown: i32,          // +40496
    pub exit_screen: u32,             // +40500
}

fn item(string: u32, count: u32, part_type: u32) -> CustomItem {
    CustomItem { string, count, part_type, ..Default::default() }
}

impl CFECarCustomisationScreen {
    fn group_items(&self, top: i32) -> (&[CustomItem], usize) {
        match top {
            0 => (&self.perf, 3),
            1 => (&self.styling, 11),
            _ => (&self.colour, 2),
        }
    }

    /// FECarCustomisationScreen.cpp 0x002526B0 — the item under the level-3 selection
    fn current_item(&self) -> CustomItem {
        let (items, stride) = self.group_items(self.sel[1]);
        items.get(stride * self.sel[2].max(0) as usize + self.sel[3].max(0) as usize).cloned().unwrap_or_default()
    }

    /// FEPlayerGarageScreen.cpp 0x0024DC90 — items in the current group
    fn group_count(&self) -> u32 {
        let g = self.sel[2].clamp(0, 4) as usize;
        match self.sel[1] {
            0 => self.perf_counts[g],
            1 => self.styling_counts[g],
            2 => self.colour_counts[g],
            _ => 0,
        }
    }

    fn top_is(&self, top: i32, groups: &[i32]) -> bool {
        self.sel[1] == top && groups.contains(&self.sel[2])
    }
}

/// FEPlayerGarageScreen.cpp 0x00247990 / 0x00247A80 / 0x0024A160 / 0x0024B3B0 — `FECK2CreateCarCustomisationScreen` (ctor, AssignEvents + the R3 functor; Init 0x00247BE0)
pub fn create_car_customisation_screen(fe: &mut Feck2) -> Arc<Mutex<CFECarCustomisationScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("CarCustomisationScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("CarCustomisationScreen_2DLayer"), ..Default::default() });
    let mut d = SElementDesc::default();
    let sprite = |fe: &mut Feck2, d: &mut SElementDesc, crc: u32, x: u32, y: u32, tex: &str, vcd: &str| {
        d.kind = ELEMENT_2D;
        d.crc = crc;
        d.position[0] = f32::from_bits(x);
        d.position[1] = f32::from_bits(y);
        d.texture = tex.into();
        d.vcd = vcd.into();
        fe.create_element(layer, d)
    };
    let locked_icon = sprite(fe, &mut d, string_crc("LockedIcon"), 1131413504, 1118044160, "FE_locked", FE_VCD);
    let perf_back_current = sprite(fe, &mut d, string_crc("PerfBackCurrent"), 1112014848, 1137180672, "attribute_bg", FE_VCD);
    let perf_level_current = sprite(fe, &mut d, string_crc("PerfLevelCurrent"), 1112014848, 1137311744, "attribute_fill", FE_VCD);
    let perf_back_installed = sprite(fe, &mut d, string_crc("PerfBackInstalled"), 1112014848, 1137180672, "attribute_bg", FE_VCD);
    let perf_level_installed = sprite(fe, &mut d, string_crc("PerfLevelInstalled"), 1112014848, 1137311744, "attribute_fill", FE_VCD);
    let text = |fe: &mut Feck2, d: &mut SElementDesc, crc: u32, font: u32, x: u32, y: u32, string: u32| {
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = font;
        d.crc = crc;
        d.position[0] = f32::from_bits(x);
        d.position[1] = f32::from_bits(y);
        d.string = string;
        fe.create_element(layer, d)
    };
    let perf_info = text(fe, &mut d, string_crc("performanceinfotext"), 0, 1112014848, 1136689152, 0);
    let slider_highlight = sprite(fe, &mut d, 323716913, 1128267776, 1120403456, "sliderhighlight", CUSTOM_VCD);
    let colour_arrow = sprite(fe, &mut d, -1828201335i32 as u32, 1127743488, 1118306304, "arrow", CUSTOM_VCD);
    let colour_slider = sprite(fe, &mut d, string_crc("ColourSliderSprite"), 1128267776, 1120403456, "colourslider", CUSTOM_VCD);
    let gradient_arrow = sprite(fe, &mut d, string_crc("ColourArrowSprite"), 1134297088, 1124597760, "arrow", CUSTOM_VCD);
    let gradient_slider = sprite(fe, &mut d, string_crc("ColourSliderSprite"), 1128267776, 1123024896, "gradientslider", CUSTOM_VCD);
    d.dynamic = false;
    let decal_status = text(fe, &mut d, string_crc("decalstatustext"), 1, 1132462080, 1120403456, string_crc("CUSTOM_STYL_BODY_DECAL_SLOT"));
    let part_cost = text(fe, &mut d, string_crc("partcosttext"), 0, 1132462080, 1120403456, string_crc("CARCUSTOMISATION_CURRENTPART_COST"));
    // FEHangoutProgression 0x00290DF0 / 0x00290E90: the swatches at (256, 80)
    let swatches = CColourSwatches::init(fe, layer, 256, 80);
    let lod_label = text(fe, &mut d, string_crc("lodlabel"), 0, 1112014848, 1135542272, 0);
    let lod_string = text(fe, &mut d, string_crc("lodstring"), 0, 1125515264, 1135542272, 0);
    let mut tweak_sliders = [0; 6];
    let mut tweak_values = [0; 6];
    for i in 0..6u32 {
        d.kind = ELEMENT_2D;
        d.crc = string_crc("ColourSliderSprite");
        d.position[0] = f32::from_bits(1123024896);
        d.position[1] = (20 * i + 70) as f32;
        d.texture = "gradientslider".into();
        d.vcd = CUSTOM_VCD.into();
        tweak_sliders[i as usize] = fe.create_element(layer, &d);
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = 0;
        d.crc = string_crc("tweakcolourval");
        d.position[0] = f32::from_bits(1128792064);
        d.position[1] = (20 * i + 62) as f32;
        d.string = 0;
        tweak_values[i as usize] = fe.create_element(layer, &d);
    }
    let obj = Arc::new(Mutex::new(CFECarCustomisationScreen {
        screen,
        layer,
        part_cost,
        locked_icon,
        perf_back_current,
        perf_back_installed,
        perf_level_current,
        perf_level_installed,
        perf_info,
        slider_highlight,
        colour_slider,
        colour_arrow,
        gradient_slider,
        gradient_arrow,
        lod_label,
        lod_string,
        tweak_sliders,
        tweak_values,
        decal_status,
        loaded: false,
        cam_eyes: [[0.0; 3]; 24],
        cam_looks: [[0.0; 3]; 24],
        mix_shade: 0.0,
        mix_hue: 0.0,
        mix_saved_shade: 0.0,
        mix_saved_hue: 0.0,
        colour_slider_mode: false,
        swatches_shown: false,
        decal_mode: false,
        decal_count: 0,
        decal_sel: 0,
        decal_slot: [0; 8],
        brakepad_mode: false,
        gauges_mode: false,
        gauge_selected: 0,
        gauge_hashes: Vec::new(),
        icon_base: [f32::from_bits(1111254630), f32::from_bits(1115789722)],
        part_hash: 0,
        perf: vec![CustomItem::default(); 15],
        styling: vec![CustomItem::default(); 55],
        colour: vec![CustomItem::default(); 10],
        perf_counts: [0; 5],
        styling_counts: [0; 5],
        colour_counts: [0; 5],
        setup_copy: VehicleSetup::new(),
        bodypart_set: None,
        swatches,
        sel: [0; 7],
        slider_row: 0,
        level: 1,
        sub_count: 4,
        part_count: 0,
        part_type: 0,
        default_wheel: 0,
        exit_countdown: -1,
        exit_screen: 0,
    }));
    // 0x0024A160 — AssignEvents
    fe.map(layer, functor(&obj, cc_enter), ev::ENTER);
    fe.map(layer, functor(&obj, cc_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, cc_start), ev::START);
    fe.map(layer, functor(&obj, cc_leave), ev::LEAVE);
    fe.map(layer, functor(&obj, |s, fe, _| cc_left_right(s, fe, 1)), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| cc_left_right(s, fe, -1)), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| mix_hold(s, fe, 1)), ev::RIGHT_HELD);
    fe.map(layer, functor(&obj, |s, fe, _| mix_hold(s, fe, -1)), ev::LEFT_HELD);
    fe.map(layer, functor(&obj, |s, fe, _| cc_up_down(s, fe, -1)), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UP_HELD);
    fe.map(layer, functor(&obj, |s, fe, _| cc_up_down(s, fe, 1)), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::DOWN_HELD);
    fe.map(layer, functor(&obj, cc_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, cc_cancel), ev::CANCEL_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::OPTION1_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| {
        // 0x0024D1E0: flip the decal's white / black (logo items only)
        if s.decal_mode {
            if s.sel[3] <= 1 {
                return false;
            }
            decal_step(s, fe, 0, true);
        }
        true
    }), ev::OPTION2_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, |s, fe, _| {
        // 0x0024B3B0: R3 moves the camera to the part
        camera_to_part(s, fe, true);
        true
    }), ev::R3_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::L2_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::R2_PRESS);
    fe.map(layer, functor(&obj, cc_update), ev::UPDATE);
    obj
}

/// FEPlayerGarageScreen.cpp 0x00248D90 — an item's icon name and position (x * 0.75, y) + the icon base
fn place_item(base: [f32; 2], it: &mut CustomItem, icon: &str, x: f32, y: f32) {
    it.icon = icon.into();
    it.pos = [x * 0.75 + base[0], y + base[1]];
}

/// FEPlayerGarageScreen.cpp 0x00248E60 — the item tables of the car's bodypart set
fn build_items(s: &mut CFECarCustomisationScreen, fe: &Feck2) {
    let Some(db) = fe.vehicles.as_ref() else { return };
    let counts = s.bodypart_set.map(|i| db.bodypart_sets[i].part_counts).unwrap_or_default();
    let b = s.icon_base;
    let c = |t: usize| counts[t] as u32;
    let mut p = |idx: usize, string: u32, count: u32, ty: u32, icon: &str, x: f32, y: f32| {
        s.perf[idx] = item(string, count, ty);
        place_item(b, &mut s.perf[idx], icon, x, y);
    };
    p(0, string_crc("CUSTOM_PERF_ACCEL_FILTERS"), c(6), 6, "mod-icon_air_filters", 60.0, 60.0);
    p(1, string_crc("CUSTOM_PERF_ACCEL_FLYWHEELS"), c(7), 7, "mod-icon_fly_wheels", 60.0, 60.0);
    p(2, string_crc("CUSTOM_PERF_ACCEL_NOS"), c(8), 8, "mod-icon_nos", 60.0, 60.0);
    p(3, string_crc("CUSTOM_PERF_BRAKES_DISCS"), db.count_brake_parts(0), TYPE_BRAKE, "mod-icon_brake_discs", 60.0, 60.0);
    p(4, string_crc("CUSTOM_PERF_BRAKES_PADS"), db.count_brake_parts(1), TYPE_BRAKE, "mod-icon_brake_pads", 60.0, 60.0);
    p(6, string_crc("CUSTOM_PERF_SPEED_ENGINE"), c(9), 9, "mod-icon_engine", 60.0, 60.0);
    p(7, string_crc("CUSTOM_PERF_SPEED_EXHAUST"), c(10), 10, "mod-icon_exhaust", 60.0, 60.0);
    p(8, string_crc("CUSTOM_PERF_SPEED_TURBO"), c(11), 11, "mod-icon_turbo", 60.0, 60.0);
    p(9, string_crc("CUSTOM_PERF_HANDLE_TIRES"), db.count_tyres(0), TYPE_TYRE, "mod-icon_tyres", 60.0, 60.0);
    p(10, 1665177115, c(12), 12, "mod-icon_suspension", 60.0, 60.0);
    let kits = db.count_setups(fe.garage.custom_vehicle, 16, false);
    p(12, 79852367, kits, TYPE_PERF_KIT, "mod-icon_performance_kit", 60.0, 60.0);
    s.perf_counts = [3, 2, 3, 2, 1];
    // styling: body, wheels, decals, gauges, stereo (groups of 11)
    let y = |bits: u32| f32::from_bits(bits);
    let mut st = |idx: usize, string: u32, count: u32, ty: u32, icon: &str, x: f32, yy: f32| {
        s.styling[idx] = item(string, count, ty);
        place_item(b, &mut s.styling[idx], icon, x, yy);
    };
    st(0, -1172363839i32 as u32, c(0), TYPE_BODY_KIT, "mod-icon_body_kits", 0.0, y(1122788966));
    st(1, 1818604040, c(0), 0, "mod-icon_front_bumper", 0.0, y(1122788966));
    st(2, -1802203687i32 as u32, c(1), 1, "mod-icon_rear_bumper", 153.0, y(1099536794));
    st(3, 1340717044, c(2), 2, "mod-icon_bonnet", 11.0, y(1118109696));
    st(4, -248971534i32 as u32, c(3), 3, "mod-icon_side_skirts", 45.2, y(1116327117));
    st(5, -261307960i32 as u32, c(4), 4, "mod-icon_spoiler", 177.8, 0.0);
    st(6, 1410799840, c(5), 5, "mod-icon_roof_scoop", 103.1, y(1112617779));
    st(11, string_crc("CARCUSTOMISATION_STYL_WHEELS_STYLE"), 0, TYPE_WHEEL, "mod-icon_wheel_style", 60.0, 60.0);
    st(12, string_crc("CARCUSTOMISATION_STYL_WHEELS_SIZE"), 0, TYPE_WHEEL, "mod-icon_wheel_size", 60.0, 60.0);
    for (i, (n, icon)) in [
        ("CUSTOM_STYL_BODY_DECAL_BONNET", "mod-icon_decal_bonnet"),
        ("CUSTOM_STYL_BODY_DECAL_SIDE", "mod-icon_side_decals"),
        ("CUSTOM_STYL_BODY_LOGO_SIDE", "mod-icon_side_logos"),
        ("CUSTOM_STYL_BODY_LOGO_F_BUMPER", "mod-icon_front_bumper_decal"),
        ("CUSTOM_STYL_BODY_LOGO_R_BUMPER", "mod-icon_rear_bumper_decal"),
        ("CUSTOM_STYL_BODY_LOGO_TRIM", "mod-icon_skirts_decal"),
        ("CUSTOM_STYL_BODY_LOGO_F_WINDSCREEN", "mod-icon_front_window_decal"),
        ("CUSTOM_STYL_BODY_LOGO_R_WINDSCREEN", "mod-icon_rear_window_decal"),
    ]
    .into_iter()
    .enumerate()
    {
        st(22 + i, string_crc(n), 1, 27 + i as u32, icon, 60.0, 60.0);
    }
    st(33, string_crc("CARCUSTOMISATION_STYL_GAUGES"), 1, 0, "mod-icon_gauges", 60.0, 60.0);
    st(44, string_crc("CARCUSTOMISATION_STYL_STEREO"), 0, 0, "mod-icon_stereo", 60.0, 60.0);
    s.styling_counts = [7, 2, 8, 1, 0];
    // colour: the neon and brake pad items are written first and then overwritten by paint items 2 and 4
    let mut co = |idx: usize, string: u32, count: u32, icon: &str| {
        s.colour[idx] = item(string, count, 15);
        place_item(b, &mut s.colour[idx], icon, 60.0, 60.0);
    };
    co(2, -1172363839i32 as u32, 0, "mod-icon_neon");
    co(4, -1172363839i32 as u32, 1, "mod-icon_colours_brakepads");
    for i in 0..5 {
        co(i, string_crc(PAINT_ITEMS[i]), 1, PAINT_ICONS[i]);
    }
    co(6, -1172363839i32 as u32, 2, "mod-icon_colour_alloys");
    co(8, -1172363839i32 as u32, 0, "mod-icon_colour_windows");
    s.colour_counts = [5, 1, 1, 1, 1];
}

/// FEPlayerGarageScreen.cpp 0x0024A8E0 — Enter
fn cc_enter(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    fe.foyer.snap_to_target();
    ov(fe, |o, fe| overlay::attach_to_screen(o, fe, Some(s.screen)));
    fe.foyer_vehicles.set_foyer("modArea");
    fe.foyer_vehicles.load("Vehicles", fe.garage.vehicle_flags == 8);
    if !s.loaded {
        s.loaded = true;
        fe.garage.vehicle_flags = 8; // byte_6F90D0
        fe.element_load(s.locked_icon);
        fe.e2d(s.locked_icon).set_size(64, 64);
        fe.element_load(s.part_cost);
        fe.text(s.part_cost).set_justification(2);
        for id in [s.perf_back_current, s.perf_back_installed, s.perf_level_current, s.perf_level_installed, s.perf_info] {
            fe.element_load(id);
        }
        // 0x002502D0: the colour sliders
        fe.element_load(s.slider_highlight);
        fe.e2d(s.slider_highlight).set_size(128, 16);
        fe.element_load(s.colour_arrow);
        fe.element_load(s.colour_slider);
        fe.e2d(s.colour_slider).set_size(128, 16);
        fe.element_load(s.gradient_arrow);
        fe.element_load(s.gradient_slider);
        fe.e2d(s.gradient_slider).set_size(128, 16);
        let orange = visos_colour(0x80, 0x78, 0x4D, 0x0E);
        fe.e2d(s.colour_arrow).set_colour(orange, false, COLOUR_WHITE);
        fe.e2d(s.colour_arrow).set_flags(2);
        fe.e2d(s.gradient_arrow).set_colour(orange, false, COLOUR_WHITE);
        fe.element_load(s.decal_status);
        fe.el(s.decal_status).set_visible(false);
        fe.text(s.decal_status).set_justification(2);
    }
    fe.element_load(s.lod_label);
    fe.element_load(s.lod_string);
    for i in 0..6 {
        fe.element_load(s.tweak_sliders[i]);
        fe.element_load(s.tweak_values[i]);
    }
    let slot = fe.garage.custom_slot;
    s.setup_copy = fe.garage.setup(slot).cloned().unwrap_or_default();
    s.swatches.set_vehicle(s.setup_copy.vehicle);
    s.swatches.load(fe);
    true
}

/// FEPlayerGarageScreen.cpp 0x0024B090 / 0x00248B10 — Activate: overlay as an icon carousel, the item tables, the car's camera
/// table, modArea background
fn cc_activate(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| {
        overlay::reset(o, fe);
        overlay::set_nav_text(o, fe, 0, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, 1, string_crc("GENERAL_BACK"));
        overlay::set_nav_text(o, fe, 3, string_crc("GENERAL_TOGGLECOLOUR"));
        overlay::show_nav(o, fe, 3, false);
        overlay::set_message(o, fe, NOT_ENOUGH_CASH);
        overlay::show_dialog(o, fe, 2, DIALOG_TITLE, &[GENERAL_YES, GENERAL_NO]);
        overlay::show_arrows(o, fe, 0);
        overlay::show_borders(o, fe);
        overlay::show_cash(o, fe);
        overlay::show_prestige(o, fe);
        menu::set_window(o, 5);
        menu::set_style(o, fe, 5);
        menu::set_horizontal(o, fe, true);
        overlay::set_icon(o, fe);
        overlay::enable(o);
    });
    // the bodypart set of the garage setup's first fitted part
    s.bodypart_set = s.setup_copy.fitted_bodyparts.iter().flatten().next().map(|p| p.set);
    build_items(s, fe);
    s.slider_row = 0;
    fe.element_set_position(s.slider_highlight, f32::from_bits(1128267776), (20 * s.slider_row) as f32 + 100.0, 0.0);
    // 0x00248B10: the camera table, raised 0.35 where set
    let setup = fe.garage.setup(fe.garage.custom_slot).cloned();
    for i in 0..24 {
        let (mut e, l) = car_camera(fe, setup.as_ref(), i);
        if e != [0.0; 3] {
            e[1] += f32::from_bits(0x3EB33333);
        }
        s.cam_eyes[i] = e;
        s.cam_looks[i] = l;
    }
    fe.foyer.show_background(true, true, 4);
    true
}

/// FEPlayerGarageScreen.cpp 0x0024AC20 / FE2DOverlay.cpp 0x0027D340 — Start: the car up, level 1, camera on entry 13
fn cc_start(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    for i in 0..6 {
        fe.el(s.tweak_sliders[i]).set_visible(false);
        fe.el(s.tweak_values[i]).set_visible(false);
    }
    fe.el(s.lod_label).set_visible(false);
    fe.el(s.lod_string).set_visible(false);
    if let Some(db) = fe.vehicles.take() {
        let sel = [fe.quick_race.selected_vehicle; 2];
        let garage = fe.quick_race.garage.clone();
        fe.foyer_vehicles.show(None, &db, sel, &garage, fe.quick_race.garage_selected as usize, &fe.garage);
        fe.vehicles = Some(db);
    }
    s.sel = [0; 7];
    s.level = 1;
    s.colour_slider_mode = false;
    s.decal_mode = false;
    s.brakepad_mode = false;
    s.gauges_mode = false;
    level1_menu(s, fe, true);
    fe.el(s.locked_icon).set_visible(false);
    perf_panel(s, fe, false);
    for id in [s.slider_highlight, s.colour_arrow, s.colour_slider, s.gradient_arrow, s.gradient_slider] {
        fe.el(id).set_visible(false);
    }
    s.swatches_shown = false;
    ov(fe, |o, fe| {
        overlay::set_dialog_visible(o, fe, false, 0);
        o.flag534 = true; // 0x0027D340
        overlay::show_message(o, fe, false);
    });
    fe.foyer.set_position_look_at(s.cam_eyes[13], CAR_LOOK_AT);
    fe.foyer.set_fov(f32::from_bits(1103626240));
    fe.foyer.set_foyer("modArea");
    fe.foyer.orbit_reinit = true;
    fe.foyer.orbit_lock = 3;
    fe.foyer.orbit_init();
    let stick = camera_stick(fe);
    fe.foyer.orbit(0.0, 0.0, stick);
    s.default_wheel = fe
        .vehicles
        .as_ref()
        .and_then(|db| db.find_vehicle(fe.garage.custom_vehicle).and_then(|v| db.vehicles[v].setup).and_then(|st| db.setups[st].wheel_hash.into()))
        .unwrap_or(0);
    s.exit_screen = 0;
    s.exit_countdown = -1;
    true
}

/// FEPlayerGarageScreen.cpp 0x0024B340 — Leave: the bought setup back into the garage slot
fn cc_leave(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| overlay::show_message(o, fe, false));
    let slot = fe.garage.custom_slot as usize;
    if let Some(g) = fe.garage.setups.get_mut(slot) {
        *g = s.setup_copy.clone();
    }
    true
}

/// FEPlayerGarageScreen.cpp 0x0024D260 — Update: the exit countdown (camera paths of 0x0024F2D0 are snapped)
fn cc_update(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.exit_countdown != -1 && s.exit_screen != 0 {
        if s.exit_countdown == 0 {
            s.exit_countdown = -1;
            if s.exit_screen == string_crc("PREVIOUS") {
                fe.goto_previous_screen(1, 0);
            } else if s.exit_screen == string_crc("HangoutScreen") {
                let crc = s.exit_screen;
                fe.goto_previous_screen(1, crc);
            } else {
                let crc = s.exit_screen;
                fe.goto_screen(crc);
            }
            return false;
        }
        s.exit_countdown -= 1;
    }
    true
}

fn shown_setup(fe: &mut Feck2) -> Option<&mut VehicleSetup> {
    fe.foyer_vehicles.slots.first_mut().and_then(|v| v.setup.as_mut())
}

/// FEPlayerGarageScreen.cpp 0x0024D8B0 — level 1: Performance / Styling / Colour
fn level1_menu(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, show: bool) {
    if show {
        let sel = s.sel[1];
        ov(fe, |o, fe| {
            menu::clear(o, fe);
            fe.text(o.menu_title).set_string(string_crc("HANGOUT_GARAGE"));
            for i in 0..3 {
                menu::add_icon_entry(o, fe, string_crc(TOP_STRINGS[i]), true, CUSTOM_VCD, TOP_ICONS[i], None);
            }
            menu::set_selection(o, fe, sel);
            menu::set_active(o, fe, true);
            menu::show(o, fe, true, false);
        });
        level2_menu(s, fe, false);
        level3_menu(s, fe, false);
        part_list(s, fe, false);
    } else {
        ov(fe, |o, fe| menu::show(o, fe, false, false));
    }
}

/// FEPlayerGarageScreen.cpp 0x0024D9E0 — level 2: the groups of the chosen category
fn level2_menu(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, show: bool) {
    if show {
        level1_menu(s, fe, false);
        level3_menu(s, fe, false);
        part_list(s, fe, false);
    }
    s.sub_count = 5;
    if !show {
        return;
    }
    let top = s.sel[1];
    let names: &[&str; 5] = match top {
        0 => &PERF_GROUPS,
        1 => &STYLING_GROUPS,
        _ => &COLOUR_GROUPS,
    };
    // colour group enables: alloys need an alloy wheel (+0x2B), brake pads a colourable pad (+0x1A)
    let (wheel_alloy, pads) = {
        let st = fe.foyer_vehicles.slots.first().and_then(|v| v.setup.as_ref());
        let db = fe.vehicles.as_ref();
        let w = st.and_then(|st| st.wheel).and_then(|w| db.map(|db| db.wheels[w].unk_2b != 0)).unwrap_or(false);
        let p = st.and_then(|st| st.brake_parts[1]).and_then(|b| db.map(|db| db.brake_parts[b].num_strings != 0)).unwrap_or(false);
        (w, p)
    };
    let counts = s.styling_counts;
    let sel = s.sel[2];
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        fe.text(o.menu_title).set_string(string_crc(TOP_STRINGS[top.clamp(0, 2) as usize]));
        for i in 0..5 {
            let mut enabled = true;
            if top == 1 && counts[i] == 0 {
                enabled = false;
            }
            if top == 2 && i == 3 {
                enabled = wheel_alloy;
            }
            if top == 2 && i == 2 && !pads {
                enabled = false;
            }
            menu::add_icon_entry(o, fe, string_crc(names[i]), enabled, CUSTOM_VCD, "", None);
        }
        menu::set_selection(o, fe, sel);
        menu::set_active(o, fe, true);
        menu::show(o, fe, true, false);
    });
}

/// FEPlayerGarageScreen.cpp 0x0024DD20 / FECarCustomisationScreen.cpp 0x002525C0 — level 3: the items of the group (colour groups 1..4 go straight to their
/// swatches or brake pad list)
fn level3_menu(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, show: bool) {
    if s.top_is(2, &[1, 3, 4]) {
        swatches(s, fe, show);
    } else if s.top_is(2, &[2]) {
        brakepad_menu(s, fe, show);
    } else {
        if s.top_is(1, &[1]) {
            // 0x002525C0: wheel style = category wheels + those of the car's size, size = the car's own wheels
            if let Some(db) = fe.vehicles.as_ref() {
                let cv = fe.garage.custom_vehicle;
                let w = s.setup_copy.wheel.map(|w| &db.wheels[w]);
                let size = w.map_or(0, |w| w.size as i64 as u32);
                s.styling[11].count = db.count_wheels_in_category(cv) + db.count_wheels_of_size(cv, size);
                let n = w.map_or(0, |w| db.count_vehicle_wheels(w.vehicle_hash, cv));
                s.styling[12].count = if n == 1 { 0 } else { n };
            }
        }
        s.sub_count = s.group_count();
        if show {
            // FE3DVehicles 0x0025E110: the car back to its spot's rotation (the port's cars do not spin)
            level1_menu(s, fe, false);
            level2_menu(s, fe, false);
            part_list(s, fe, false);
            let (items, stride) = s.group_items(s.sel[1]);
            let g = s.sel[2].max(0) as usize;
            let top = s.sel[1];
            let group_name = match top {
                0 => PERF_GROUPS[g.min(4)],
                1 => STYLING_GROUPS[g.min(4)],
                _ => COLOUR_GROUPS[g.min(4)],
            };
            let entries: Vec<(u32, bool, String)> = (0..s.sub_count as usize)
                .map(|i| {
                    let it = items.get(stride * g + i).cloned().unwrap_or_default();
                    let enabled = if top == 2 {
                        // paint: manufacture and custom always, the others need brands of their kind
                        let db = fe.vehicles.as_ref();
                        match i {
                            0 | 4 => true,
                            1 => db.is_some_and(|db| db.count_colour_brands(0, false) != 0),
                            2 => db.is_some_and(|db| db.count_colour_brands(2, false) != 0),
                            _ => db.is_some_and(|db| db.count_colour_brands(1, false) != 0),
                        }
                    } else {
                        it.count != 0
                    };
                    (it.string, enabled, it.icon)
                })
                .collect();
            let sel = s.sel[3];
            ov(fe, |o, fe| {
                menu::clear(o, fe);
                fe.text(o.menu_title).set_string(string_crc(group_name));
                for (string, enabled, icon) in &entries {
                    menu::add_icon_entry(o, fe, *string, *enabled, CUSTOM_VCD, icon, None);
                }
                menu::set_selection(o, fe, sel);
                menu::set_active(o, fe, true);
                menu::show(o, fe, true, false);
            });
        }
    }
    if s.top_is(1, &[2]) {
        ov(fe, |o, _| o.flag534 = true);
    }
}

/// FEPlayerGarageScreen.cpp 0x0024C080 — part cost of list entry `idx` of `ty`
fn part_cost(s: &CFECarCustomisationScreen, fe: &Feck2, ty: u32, idx: i32) -> i32 {
    let Some(db) = fe.vehicles.as_ref() else { return 0 };
    if ty < 14 {
        let Some(p) = s.bodypart_set.and_then(|set| db.bodypart_sets[set].parts[ty as usize].get(idx.max(0) as usize)) else { return 0 };
        // flag 8: free (VehicleDatabase 0x00106460)
        return if p.flags & 8 != 0 { 0 } else { p.cost as i32 };
    }
    match ty {
        TYPE_WHEEL => db.find_wheel(s.part_hash).map_or(0, |w| if db.wheels[w].fitted != 0 { 0 } else { db.wheels[w].cost as i32 }),
        TYPE_BRAKE => db.find_brake_part(s.part_hash).map_or(0, |b| if db.brake_parts[b].flags & 2 != 0 { 0 } else { db.brake_parts[b].cost as i32 }),
        TYPE_TYRE => db.find_tyre(s.part_hash).map_or(0, |t| if db.tyres[t].fitted != 0 { 0 } else { db.tyres[t].cost as i32 }),
        _ => 0, // kits: GetSetupCost not ported
    }
}

/// FEPlayerGarageScreen.cpp 0x0024C2E0 — the part under the cursor is locked
fn part_locked(s: &CFECarCustomisationScreen, fe: &Feck2) -> bool {
    let Some(db) = fe.vehicles.as_ref() else { return false };
    if s.part_type < 14 {
        return s
            .bodypart_set
            .and_then(|set| db.bodypart_sets[set].parts[s.part_type as usize].get(s.sel[5].max(0) as usize))
            .is_some_and(|p| p.flags & 0x10 != 0);
    }
    match s.part_type {
        TYPE_WHEEL => db.find_wheel(s.part_hash).is_some_and(|w| db.wheels[w].unk_29 != 0),
        TYPE_BRAKE => db.find_brake_part(s.part_hash).is_some_and(|b| db.brake_parts[b].flags & 1 != 0),
        TYPE_TYRE => db.find_tyre(s.part_hash).is_some_and(|t| db.tyres[t].unk_09 != 0),
        _ => false,
    }
}

fn set_cost_text(s: &CFECarCustomisationScreen, fe: &mut Feck2, cost: i32) {
    let Some(h) = fe.text(s.part_cost).string else { return };
    fe.strings.set_argument(h, string_crc("dPartCost"), cost as u32).ok();
}

/// FEPlayerGarageScreen.cpp 0x0024C670 — locked icon instead of the price, or the price
fn price_or_lock(s: &CFECarCustomisationScreen, fe: &mut Feck2) {
    if part_locked(s, fe) {
        fe.el(s.locked_icon).set_visible(true);
        fe.el(s.part_cost).set_visible(false);
    } else {
        fe.el(s.locked_icon).set_visible(false);
        let c = part_cost(s, fe, s.part_type, s.sel[5]);
        if c != 0 {
            fe.el(s.part_cost).set_visible(true);
            set_cost_text(s, fe, c);
        }
    }
}

/// FEPlayerGarageScreen.cpp 0x0024C400 — the performance bars on / off
fn perf_panel(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, show: bool) {
    if show {
        perf_bars(s, fe);
    }
    for id in [s.perf_back_current, s.perf_back_installed, s.perf_level_current, s.perf_level_installed, s.perf_info] {
        fe.el(id).set_visible(show);
    }
}

/// FEPlayerGarageScreen.cpp 0x0024C480 — bar widths (v + 1) * 8 for the setup as bought and as shown
fn perf_bars(s: &CFECarCustomisationScreen, fe: &mut Feck2) {
    if s.sel[1] != 0 {
        return;
    }
    let Some(db) = fe.vehicles.as_ref() else { return };
    let shown = fe.foyer_vehicles.slots.first().and_then(|v| v.setup.as_ref());
    let installed = fe.garage.setup(fe.garage.custom_slot);
    let value = |st: Option<&VehicleSetup>| -> f32 {
        let Some(st) = st.filter(|st| st.vehicle.is_some()) else { return 0.0 };
        match s.sel[3] {
            1 => st.get_acceleration(db) as f32,
            2 => st.get_top_speed(db) as f32,
            3 => st.get_handling_modifier(db),
            // VehicleDatabasePhysics 0x00189500
            _ => (6.0f64 - (3.0f32 * (((st.get_top_speed_modifier(db) as f64 + st.get_acceleration_modifier(db) as f64) as f32) * 0.5)) as f64) as f32,
        }
    };
    let cur = value(shown);
    let inst = value(installed);
    let w = |v: f32| ((v + 1.0) * 8.0) as i64 as u32;
    fe.e2d(s.perf_level_current).set_size(w(cur), 8);
    fe.e2d(s.perf_level_installed).set_size(w(inst), 8);
    fe.text(s.perf_info).set_string(string_crc(PERF_GROUPS[s.sel[2].clamp(0, 4) as usize]));
}

/// FECarCustomisationScreen.cpp 0x00252260 — put the part under the cursor on the shown car
fn apply_part(s: &CFECarCustomisationScreen, fe: &mut Feck2) {
    let Some(db) = fe.vehicles.take() else { return };
    let set = s.bodypart_set;
    let ty = s.part_type;
    let idx = s.sel[5].max(0) as usize;
    let hash = s.part_hash;
    let custom_vehicle = fe.garage.custom_vehicle;
    let mut bounce = false;
    if let Some(st) = shown_setup(fe) {
        if ty < 14 {
            if let Some(set) = set {
                if idx < db.bodypart_sets[set].parts[ty as usize].len() {
                    // FE3DVehicles 0x0025CB20 part 12 starts the wheel change bounce
                    bounce |= ty == 12;
                    st.fitted_bodyparts[ty as usize] = Some(velocity_physics::vehicle_database::vehicle_database::PartRef { set, part_type: ty as usize, index: idx });
                }
            }
        } else if ty == TYPE_WHEEL {
            if let Some(w) = db.find_wheel(hash) {
                st.wheel = Some(w);
                bounce = true; // FE3DVehicles 0x0025CF40 (case 0x11)
            }
        } else if ty == TYPE_BRAKE {
            if let Some(b) = db.find_brake_part(hash) {
                let k = (db.brake_parts[b].kind as usize).min(1);
                // FE3DVehicles 0x0025D020: reload the disc / pad models + DrawWheelsAndBrakes, no bounce (the foyer
                // respawn key holds the brake part names)
                st.brake_parts[k] = Some(b);
            }
        } else if ty == TYPE_TYRE {
            if let Some(t) = db.find_tyre(hash) {
                // FE3DVehicles 0x0025D110: the "_tyre" texture of the 4 wheels from TyresT.vcd (CarRender 0x00118910,
                // graphics request in docs/graphics.md), no bounce
                st.tyre = Some(t);
            }
        } else {
            let mask = if ty == TYPE_PERF_KIT { 16 } else { 8 };
            if let Some(p) = db.get_nth_setup(custom_vehicle, s.sel[5].max(0) as u32, mask, false) {
                db.apply_predef_setup(st, p);
            }
        }
    }
    if bounce {
        let now = fe.stopwatch;
        fe.foyer_vehicles.start_wheel_bounce(now);
    }
    fe.vehicles = Some(db);
}

/// FEPlayerGarageScreen.cpp 0x0024E590 — step the part list: cost, the part on the car, bars
fn part_step(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, d: i32) {
    s.sel[5] += d;
    let it = s.current_item();
    s.part_count = it.count;
    if s.sel[5] < 0 {
        s.sel[5] = 0;
    } else if s.sel[5] as u32 >= s.part_count {
        s.sel[5] = s.part_count as i32 - 1;
    }
    if s.part_count == 0 {
        return;
    }
    s.part_type = it.part_type;
    let mut cost = 0;
    {
        let db = fe.vehicles.as_ref();
        let i = s.sel[5].max(0) as u32;
        if s.part_type < 14 {
            cost = part_cost(s, fe, s.part_type, s.sel[5]);
            s.part_hash = s
                .bodypart_set
                .and_then(|set| db.and_then(|db| db.bodypart_sets[set].parts[s.part_type as usize].get(i as usize)))
                .map_or(0, |p| p.hash);
        } else if let Some(db) = db {
            match s.part_type {
                TYPE_WHEEL => {
                    // style list: the category's wheels then those of the car's size
                    if let Some(w) = wheel_list(fe, s).get(i as usize) {
                        s.part_hash = db.wheels[*w].hash;
                    } else if i == 0 {
                        s.part_hash = s.default_wheel;
                    }
                }
                TYPE_BRAKE => {
                    if let Some(b) = db.get_nth_brake_part((s.sel[3] != 0) as u32, i) {
                        s.part_hash = db.brake_parts[b].hash;
                    }
                }
                TYPE_TYRE => {
                    if let Some(t) = db.get_nth_tyre(0, i) {
                        s.part_hash = db.tyres[t].hash;
                    }
                }
                _ => {
                    let mask = if s.part_type == TYPE_PERF_KIT { 16 } else { 8 };
                    if let Some(p) = db.get_nth_setup(fe.garage.custom_vehicle, i, mask, false) {
                        s.part_hash = db.setups[p].hash;
                    }
                }
            }
            cost = part_cost(s, fe, s.part_type, s.sel[5]);
        }
    }
    if cost != 0 {
        fe.el(s.part_cost).set_visible(true);
        set_cost_text(s, fe, cost);
        let c = if part_locked(s, fe) { visos_colour(0x80, 0x28, 0x18, 0) } else { visos_colour(0x80, 0x78, 0x4D, 0x0E) };
        fe.text(s.part_cost).set_colour(c);
    } else {
        fe.el(s.part_cost).set_visible(false);
    }
    apply_part(s, fe);
    price_or_lock(s, fe);
    perf_bars(s, fe);
}

/// FEPlayerGarageScreen.cpp 0x0024EB00 — (style mode) wheels offered for the car: the custom vehicle's category, then those of its size
fn wheel_list(fe: &Feck2, s: &CFECarCustomisationScreen) -> Vec<usize> {
    let Some(db) = fe.vehicles.as_ref() else { return Vec::new() };
    let cat = fe.garage.custom_vehicle;
    let mut v: Vec<usize> = (0..db.count_wheels_in_category(cat)).filter_map(|i| db.get_nth_wheel_in_category(cat, i as i32 + 1)).collect();
    let size = s.setup_copy.wheel.map_or(0, |w| db.wheels[w].size as i64 as u32);
    v.extend((0..db.count_wheels_of_size(cat, size)).filter_map(|i| db.get_nth_wheel_of_size(cat, size, i as i32 + 1)));
    v
}

/// FEPlayerGarageScreen.cpp 0x0024E1B0 — level 5: the part list as an icon carousel, starting on the fitted part
fn part_list(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, show: bool) {
    if show {
        part_step(s, fe, 0);
        camera_to_part(s, fe, true);
    }
    if s.top_is(1, &[2]) {
        decal_menu(s, fe, show);
        return;
    }
    if show {
        s.sel[5] = 0;
        let it = s.current_item();
        s.part_type = it.part_type;
        let mut entries: Vec<(u32, bool, String, Option<String>)> = Vec::new();
        if let Some(db) = fe.vehicles.as_ref() {
            if s.part_type >= 14 {
                match s.part_type {
                    TYPE_WHEEL => {
                        for w in wheel_list(fe, s) {
                            entries.push((db.wheels[w].vehicle_hash, db.wheels[w].unk_29 != 1, "mod-icon_wheel_style".into(), None));
                        }
                    }
                    TYPE_BRAKE => {
                        let k = (s.sel[3] != 0) as u32;
                        for i in 0..db.count_brake_parts(k) {
                            if let Some(b) = db.get_nth_brake_part(k, i) {
                                entries.push((db.brake_parts[b].hash, db.brake_parts[b].flags & 1 == 0, String::new(), None));
                            }
                        }
                    }
                    TYPE_TYRE => {
                        for k in 0..2 {
                            for i in 0..db.count_tyres(k) {
                                if let Some(t) = db.get_nth_tyre(k, i) {
                                    entries.push((db.tyres[t].hash, db.tyres[t].unk_09 != 1, "mod-icon_tyres".into(), None));
                                }
                            }
                        }
                        // 0x0024F060 (the tyre list, 0x0024EF80): the front wheels turned 35 degrees
                        let now = fe.stopwatch;
                        fe.foyer_vehicles.set_steer(0, 35.0, now);
                    }
                    TYPE_BODY_KIT | TYPE_PERF_KIT => {
                        let (mask, icon) = if s.part_type == TYPE_PERF_KIT { (16, "mod-icon_performance_kit") } else { (8, "mod-icon_body_kits") };
                        for i in 0..it.count {
                            if let Some(p) = db.get_nth_setup(fe.garage.custom_vehicle, i, mask, false) {
                                entries.push((db.setups[p].hash, true, icon.into(), None));
                            }
                        }
                    }
                    _ => {}
                }
            } else if let Some(set) = s.bodypart_set {
                let fitted = s.setup_copy.fitted_bodyparts[s.part_type as usize];
                let mut found = false;
                for (i, p) in db.bodypart_sets[set].parts[s.part_type as usize].iter().enumerate() {
                    let is_fitted = fitted.is_some_and(|f| db.bodypart_sets[f.set].parts[f.part_type][f.index].hash == p.hash);
                    if !found && is_fitted {
                        s.sel[5] = i as i32;
                        s.part_hash = p.hash;
                        found = true;
                    }
                    entries.push((p.hash, p.flags & 0x10 == 0, it.icon.clone(), p.string_14.clone()));
                }
            }
        }
        if s.part_type >= 14 {
            s.sel[5] = 0;
            s.sel[6] = 0;
        }
        let sel = s.sel[5];
        ov(fe, |o, fe| {
            menu::clear(o, fe);
            fe.text(o.menu_title).set_string(it.string);
            for (string, enabled, icon, icon2) in &entries {
                menu::add_icon_entry(o, fe, *string, *enabled, CUSTOM_VCD, icon, icon2.as_deref());
            }
            menu::set_selection(o, fe, sel);
            menu::set_active(o, fe, true);
            menu::show(o, fe, true, false);
        });
        apply_part(s, fe);
        if s.part_type >= 14 {
            part_step(s, fe, 0);
        }
    }
    fe.el(s.part_cost).set_visible(false);
    fe.el(s.locked_icon).set_visible(false);
    if s.sel[1] == 0 {
        perf_panel(s, fe, show);
    }
}

/// FECarCustomisationScreen.cpp 0x00250C20 / 0x00250EE0 — the decal type of decal item `item` (bonnet 0, side 1, the
/// two screens 2, the logo items 3)
fn decal_kind(item: i32) -> u32 {
    match item {
        0 => 0,
        1 => 1,
        6 | 7 => 2,
        _ => 3,
    }
}

/// FECarCustomisationScreen.cpp 0x002509A0 — the decal the setup (as bought, or shown) has on the item's slot
fn shown_decal(s: &CFECarCustomisationScreen, fe: &mut Feck2, from_copy: bool) -> Option<usize> {
    let slot = s.decal_slot[s.sel[3].clamp(0, 7) as usize].max(0) as usize;
    let st = if from_copy { Some(&s.setup_copy) } else { shown_setup(fe).map(|st| &*st) }?;
    match s.sel[3] {
        0 => st.decal_0,
        1 => st.decal_1,
        2 => st.decals_3.get(slot).copied().flatten(),
        3 => st.decals_3_1.get(slot).copied().flatten(),
        4 => st.decals_3_2.get(slot).copied().flatten(),
        5 => st.decals_3_3.get(slot).copied().flatten(),
        6 => st.decal_2_4,
        7 => st.decal_2_5,
        _ => None,
    }
}

/// FECarCustomisationScreen.cpp 0x00250BB0 — the overlay entry of the slot's decal (0 = off)
fn decal_index(s: &CFECarCustomisationScreen, fe: &mut Feck2, from_copy: bool) -> i32 {
    match shown_decal(s, fe, from_copy) {
        Some(d) => fe.vehicles.as_ref().map_or(0, |db| db.get_decal_index(d, 0, 0) as i32 + 1),
        None => 0,
    }
}

/// FECarCustomisationScreen.cpp 0x002513F0 — the slot number text (logo items 2 and 4 only) and the up / down arrows
fn decal_status_update(s: &CFECarCustomisationScreen, fe: &mut Feck2) {
    if !s.decal_mode {
        return;
    }
    let slot = if s.sel[3] == 2 || s.sel[3] == 4 { s.decal_slot[s.sel[3] as usize] } else { -1 };
    if slot == -1 {
        fe.el(s.decal_status).set_visible(false);
    } else {
        if let Some(h) = fe.text(s.decal_status).string {
            fe.strings.set_argument(h, string_crc("dSlotIndex"), (slot + 1) as u32).ok();
        }
        fe.el(s.decal_status).set_visible(true);
        ov(fe, |o, fe| overlay::set_arrows_visible(o, fe, true, 0));
    }
}

/// FECarCustomisationScreen.cpp 0x00250EE0 — step the chosen decal by `delta` (`mirror` swaps the white / black flag
/// instead) and put it on the shown car's slot
fn decal_step(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, delta: i32, mirror: bool) {
    let item = s.sel[3].clamp(0, 7);
    let kind = decal_kind(item);
    let Some(db) = fe.vehicles.take() else { return };
    s.decal_count = db.count_decals(kind, 0, 0) as i32;
    s.decal_sel += delta;
    if s.decal_sel >= 0 {
        if s.decal_sel >= s.decal_count + 1 {
            s.decal_sel = s.decal_count;
        }
    } else {
        s.decal_sel = 0;
    }
    let decal = if s.decal_sel != 0 { db.get_nth_decal(kind, s.decal_sel as u32 - 1, 0, 0) } else { None };
    s.part_hash = decal.map_or(0, |d| db.decals[d].hash);
    let slot = s.decal_slot[item as usize];
    let u = slot.max(0) as usize;
    // the flags the shown setup has on that slot (SVehicleSetup +0x1C4 / +0x1F4 / +0x204 / +0x214 / +0x220 / +0x228)
    let mut flags = 0;
    let mut sub = 0;
    if item > 1 {
        sub = item as u32 - 2;
        if let Some(st) = shown_setup(fe) {
            flags = match item {
                2 => st.decals_3_flags.get(u).copied(),
                3 => st.decals_3_1_flags.get(u).copied(),
                4 => st.decals_3_2_flags.get(u).copied(),
                5 => st.decals_3_3_flags.get(u).copied(),
                6 => Some(st.decal_2_4_flags),
                _ => Some(st.decal_2_5_flags),
            }
            .unwrap_or(0);
        }
    }
    let mut set_flags = flags;
    if mirror {
        if flags == 1 {
            set_flags = 2;
        } else if flags == 2 {
            set_flags = 1;
        }
    }
    if let Some(st) = shown_setup(fe) {
        CVehicleDatabase::set_setup_decal(st, decal, kind, sub, slot as u32, set_flags);
    }
    fe.vehicles = Some(db);
    decal_status_update(s, fe);
}

/// FECarCustomisationScreen.cpp 0x002512E0 — up / down: the next slot of the item (the setup as shown becomes the one
/// as bought), the slot's decal as the overlay selection
fn decal_slot_step(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, delta: i32) {
    let item = s.sel[3];
    if delta != 0 {
        if let Some(st) = shown_setup(fe) {
            s.setup_copy = st.clone();
        }
    }
    let count = if item >= 2 { fe.foyer_vehicles.decal_slot_count(item as usize - 2) } else { 0 };
    if (2..8).contains(&item) {
        let slot = &mut s.decal_slot[item as usize];
        *slot += delta;
        if *slot >= 0 {
            if *slot >= count {
                *slot = 0;
            }
        } else {
            *slot = count - 1;
        }
    }
    s.decal_sel = decal_index(s, fe, true);
    let sel = s.decal_sel;
    ov(fe, |o, fe| menu::set_selection(o, fe, sel));
    decal_status_update(s, fe);
}

/// FECarCustomisationScreen.cpp 0x00250C20 — the decal list of the chosen item as the overlay menu ("off" then every decal
/// of the kind), the slot's decal chosen, the camera on the item; `show` false leaves decal mode
fn decal_menu(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, show: bool) {
    s.decal_mode = show;
    if !show {
        ov(fe, |o, fe| overlay::show_nav(o, fe, 3, false));
        fe.el(s.decal_status).set_visible(false);
        return;
    }
    let it = s.current_item();
    s.part_type = it.part_type;
    level2_menu(s, fe, false);
    s.decal_sel = decal_index(s, fe, false);
    let kind = decal_kind(s.sel[3]);
    let hashes: Vec<u32> = fe.vehicles.as_ref().map_or_else(Vec::new, |db| {
        (0..db.count_decals(kind, 0, 0)).filter_map(|n| db.get_nth_decal(kind, n, 0, 0)).map(|d| db.decals[d].hash).collect()
    });
    s.decal_count = hashes.len() as i32;
    let sel = s.decal_sel;
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        fe.text(o.menu_title).set_string(it.string);
        if kind == 2 || kind == 3 {
            overlay::show_nav(o, fe, 3, true);
        }
        menu::add_entry(o, fe, string_crc("GENERAL_OFF"), true, false, 0);
        for h in &hashes {
            menu::add_entry(o, fe, *h, true, false, 0);
        }
        menu::set_selection(o, fe, sel);
        menu::show(o, fe, true, false);
    });
    decal_step(s, fe, 0, false);
    camera_to_part(s, fe, true);
}

/// FEPlayerGarageScreen.cpp 0x0024F2D0 — camera to the part's table entry (bodyparts by type, wheels 22, brakes
/// and tyres 23, else 13); the orbit path the original takes for far moves (9 m+, 0x0024D260) is snapped
fn camera_to_part(s: &CFECarCustomisationScreen, fe: &mut Feck2, _move: bool) {
    let t = s.part_type as usize;
    let target = if s.decal_mode {
        s.cam_eyes[(s.sel[3].max(0) as usize + 14).min(23)]
    } else if t < 14 {
        s.cam_eyes[t]
    } else if t == TYPE_WHEEL as usize {
        s.cam_eyes[22]
    } else if t == TYPE_BRAKE as usize || t == TYPE_TYRE as usize {
        s.cam_eyes[23]
    } else {
        s.cam_eyes[13]
    };
    if target != [0.0; 3] {
        fe.foyer.set_position_look_at(target, CAR_LOOK_AT);
        fe.foyer.orbit_reinit = true;
    }
}

/// FEPlayerGarageScreen.cpp 0x0024CA10 / 0x0024C880 — the colour swatches for the colour group (paint by item / brand)
fn swatches(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, show: bool) {
    if show {
        // 0x0024C880: which colours
        let (use_vehicle, kind, brand) = match (s.sel[2], s.sel[3]) {
            (1, _) => (false, 3, 0),
            (3, _) => (false, 4, 0),
            (4, _) => (false, 5, 0),
            (_, 1) => (false, 0, s.sel[4].max(0) as u32),
            (_, 2) => (false, 2, s.sel[4].max(0) as u32),
            (_, 3) => (false, 1, s.sel[4].max(0) as u32),
            _ => (true, 0, 0),
        };
        s.swatches.fill(fe, use_vehicle, kind, brand);
    }
    s.swatches.show(fe, show);
    // start on the setup's colour of that slot
    let slot = match s.sel[2] {
        1 => 1,
        3 => 2,
        4 => 3,
        _ => 0,
    };
    let c = s.setup_copy.colours[slot].clone();
    s.swatches.select(fe, c.hash, c.brand);
    s.swatches_shown = show;
    ov(fe, |o, fe| {
        menu::show(o, fe, !show, false);
        menu::set_active(o, fe, !show);
        fe.text(o.menu_title).set_string(string_crc("CARCUSTOMISATION_STYL_COLOUR"));
        fe.el(o.menu_title).set_visible(true);
    });
    if show {
        swatch_apply(s, fe);
    }
}

/// FEPlayerGarageScreen.cpp 0x0024CD00 — the colour under the highlighter on the shown car's slot
fn swatch_apply(s: &mut CFECarCustomisationScreen, fe: &mut Feck2) {
    let Some(c) = s.swatches.current(fe) else { return };
    let Some(rec) = fe.vehicles.as_ref().map(|db| db.colours[c].clone()) else { return };
    let slot = match s.sel[2] {
        1 => 1,
        3 => 2,
        4 => 3,
        _ => 0,
    };
    if let Some(st) = shown_setup(fe) {
        st.colours[slot] = rec;
    }
}

/// FECarCustomisationScreen.cpp 0x00251E10 / 0x00251FA0 — brake pad colours: three fixed choices (the second picks them)
fn brakepad_menu(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, show: bool) {
    s.brakepad_mode = show;
    if show {
        s.sel[3] = 0;
        ov(fe, |o, fe| {
            menu::clear(o, fe);
            fe.text(o.menu_title).set_string(string_crc("CUSTOM_PERF_BRAKES_PADS"));
            for n in ["CARCUSTOMISATION_COLOUR_3", "CARCUSTOMISATION_COLOUR_1", "CARCUSTOMISATION_COLOUR_6"] {
                menu::add_icon_entry(o, fe, string_crc(n), true, CUSTOM_VCD, "mod-icon_colours_brakepads", None);
            }
            menu::set_selection(o, fe, 0);
            menu::show(o, fe, true, false);
        });
    }
}

/// FEPlayerGarageScreen.cpp 0x0024B550 / 0x0024B5D0 / 0x0024B8B0 — left / right (both call the last)
fn cc_left_right(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, d: i32) -> bool {
    if s.swatches_shown {
        s.swatches.move_col(fe, d);
        swatch_apply(s, fe);
    } else if s.decal_mode {
        decal_step(s, fe, d, false);
    } else if s.brakepad_mode {
        s.sel[3] = (s.sel[3] + d).clamp(0, 2);
    } else if s.level == 5 && !s.colour_slider_mode && !s.decal_mode && !s.gauges_mode {
        part_step(s, fe, d);
    }
    true
}

/// FEPlayerGarageScreen.cpp 0x0024B650 / 0x0024B6F0 / 0x0024BA20 — (both call the last) up / down: swatch rows
fn cc_up_down(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, d: i32) -> bool {
    if s.swatches_shown {
        s.swatches.move_row(fe, d);
        swatch_apply(s, fe);
    }
    if s.colour_slider_mode {
        // 0x0024BA20: the other row, highlighter to y = 100 + 20 * row
        s.slider_row += d;
        if s.slider_row < 0 {
            s.slider_row = 1;
        } else if s.slider_row > 1 {
            s.slider_row = 0;
        }
        fe.element_set_position(s.slider_highlight, f32::from_bits(1128267776), (20 * s.slider_row) as f32 + 100.0, 0.0);
    }
    if s.decal_mode {
        decal_slot_step(s, fe, d);
    }
    true
}

/// FEPlayerGarageScreen.cpp 0x0024B9C0 (held left / right) -> 0x0024B3E0 hue, 0x0024B480 shade
fn mix_hold(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, d: i32) -> bool {
    if s.colour_slider_mode {
        if s.slider_row == 0 {
            // 0x0024B3E0
            s.mix_hue += (4 * d) as f32;
            if s.mix_hue >= 0.0 {
                if s.mix_hue > 1529.0 {
                    s.mix_hue = 0.0;
                }
            } else {
                s.mix_hue = 1529.0;
            }
            mix_apply(s, fe, 0);
        } else if s.slider_row == 1 {
            mix_apply(s, fe, d);
        }
    }
    true
}

/// FEPlayerGarageScreen.cpp 0x0024B480 + 0x0024C750 — shade step, the mix colour (FE3DVehicles 0x0025E170 ->
/// VehicleDatabaseColours 0x00187BA0) onto the shown car, the two arrows
fn mix_apply(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, d: i32) {
    s.mix_shade = (s.mix_shade + (2 * d) as f32).clamp(-255.0, 255.0);
    let (hue, shade) = (s.mix_hue as i32, s.mix_shade as i32);
    if let Some(db) = fe.vehicles.take() {
        if let Some(st) = shown_setup(fe) {
            let c = &mut st.colours[0];
            c.hash = 0; // 0x0025E170: hash, kind 0, hue, shade
            c.unk_0c = 0;
            c.set_hue_shade(hue, shade);
            db.update_mix_colour(st);
            s.mix_shade = st.colours[0].shade() as f32; // the update clamps and writes the shade back
        }
        fe.vehicles = Some(db);
    }
    fe.element_set_position(s.colour_arrow, 0.083f32 * s.mix_hue + 184.0, f32::from_bits(1118306304), 0.0);
    fe.element_set_position(s.gradient_arrow, s.mix_shade / 4.0 + 248.0, f32::from_bits(1124597760), 0.0);
}

/// FECarCustomisationScreen.cpp 0x002505F0 — mix-your-own on / off (the two sliders, their arrows and the highlighter)
fn colour_mode(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, on: bool) {
    s.colour_slider_mode = on;
    for id in [s.colour_slider, s.colour_arrow, s.gradient_slider, s.gradient_arrow, s.slider_highlight] {
        fe.el(id).set_visible(on);
    }
    let copy = s.setup_copy.colours[0].clone();
    if on {
        ov(fe, |o, fe| overlay::set_nav_string(o, fe, 0, string_crc("CARCUSTOMISATION_APPLYPAINT")));
        if copy.hash != 0 {
            s.mix_hue = 0.0;
            s.mix_shade = 0.0;
        } else {
            s.mix_hue = copy.hue() as f32;
            s.mix_shade = copy.shade() as f32;
        }
    } else if copy.hash != 0 {
        if let Some(st) = shown_setup(fe) {
            st.colours[0] = copy;
        }
    } else {
        s.mix_shade = s.mix_saved_shade;
        s.mix_hue = s.mix_saved_hue;
        mix_apply(s, fe, 0);
        ov(fe, |o, fe| overlay::set_nav_string(o, fe, 0, string_crc("GENERAL_SELECT")));
    }
    ov(fe, |o, fe| {
        menu::show(o, fe, !on, false);
        menu::set_active(o, fe, !on);
        fe.text(o.menu_title).set_string(string_crc("CARCUSTOMISATION_STYL_COLOUR"));
        fe.el(o.menu_title).set_visible(true);
    });
}

/// FECarCustomisationScreen.cpp 0x00252100 — back to the setup as bought (not for paint / gauges / stereo)
fn restore_setup(s: &mut CFECarCustomisationScreen, fe: &mut Feck2) {
    if s.top_is(2, &[0]) || s.top_is(1, &[3, 4]) {
        return;
    }
    s.sel[5] = s.sel[6];
    let copy = s.setup_copy.clone();
    if let Some(st) = shown_setup(fe) {
        *st = copy;
    }
}

/// FECarCustomisationScreen.cpp 0x002520B0 — keep the shown setup as bought
fn keep_setup(s: &mut CFECarCustomisationScreen, fe: &mut Feck2) {
    if let Some(st) = fe.foyer_vehicles.slots.first().and_then(|v| v.setup.clone()) {
        s.setup_copy = st;
    }
    if s.sel[3] == 4 && s.top_is(2, &[0]) {
        // 0x002504A0: the mix colour as bought
        s.mix_saved_shade = s.mix_shade;
        s.mix_saved_hue = s.mix_hue;
    }
    s.sel[6] = s.sel[5];
}

/// FEPlayerGarageScreen.cpp 0x0024BB90 / FECarCustomisationScreen.cpp 0x00251C00 — Select: down a level, or into the part / colour / dialog step
fn cc_select(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let (sel, enabled, dialog) = fe.overlay.as_ref().map_or((0, false, false), |o| (o.menu_selected, menu::entry_enabled(o, o.menu_selected.max(0) as usize), o.dialog_active));
    if !enabled {
        return false;
    }
    let lv = s.level.clamp(0, 6) as usize;
    s.sel[lv] = sel;
    for k in lv + 1..6 {
        s.sel[k] = 0;
    }
    if s.level == 2 && s.sel[1] == 0 && s.sel[2] == 4 {
        s.level = 5;
        s.sel[3] = 0;
    } else if s.level == 5 && s.top_is(2, &[0]) {
        // paint swatch picked
        keep_setup(s, fe);
        swatches(s, fe, false);
        if s.sel[3] == 4 {
            colour_mode(s, fe, false);
        }
        s.level = 3;
    } else if s.level == 2 && s.top_is(1, &[3, 4]) {
        s.level = 5;
        s.sel[3] = 0;
    } else {
        s.level += 1;
        if s.level == 4 && !(s.top_is(2, &[0]) && s.sel[3] != 0 && s.sel[3] != 4) {
            s.level += 1;
        }
    }
    if dialog {
        purchase(s, fe, true);
    } else if s.level == 6 {
        purchase(s, fe, false);
    } else if s.level == 4 && s.sel[1] == 2 {
        brand_menu(s, fe);
    } else if s.level == 5 {
        ov(fe, |o, fe| menu::set_active(o, fe, false));
        if s.top_is(2, &[0]) {
            if s.sel[3] == 4 {
                colour_mode(s, fe, true);
            } else {
                swatches(s, fe, true);
            }
        } else if s.top_is(2, &[1, 3, 4]) {
            // 0x00251C00: the colour as bought, back to the groups
            keep_setup(s, fe);
            s.level = 2;
            level2_menu(s, fe, true);
        } else if s.top_is(2, &[2]) {
            keep_setup(s, fe);
            s.brakepad_mode = false;
            s.level = 2;
            level2_menu(s, fe, true);
        } else if s.top_is(1, &[3]) {
            gauges_list(s, fe, true);
        } else if s.current_item().count != 0 {
            level2_menu(s, fe, false);
            level3_menu(s, fe, false);
            part_list(s, fe, true);
            if s.sel[2] != 2 && s.sel[2] != 1 {
                let c = part_cost(s, fe, s.part_type, s.sel[5]);
                if c != 0 {
                    fe.el(s.part_cost).set_visible(true);
                    set_cost_text(s, fe, c);
                } else {
                    fe.el(s.part_cost).set_visible(false);
                }
                price_or_lock(s, fe);
                perf_bars(s, fe);
            }
        } else {
            s.level -= 1;
        }
    } else if s.level == 3 {
        level3_menu(s, fe, true);
    } else if s.level == 2 {
        level2_menu(s, fe, true);
    }
    true
}

/// FEPlayerGarageScreen.cpp 0x0024CBA0 — level 4: the paint kind's brands
fn brand_menu(s: &mut CFECarCustomisationScreen, fe: &mut Feck2) {
    let kind = match s.sel[3] {
        1 => 0,
        2 => 2,
        _ => 1,
    };
    let brands: Vec<u32> = fe
        .vehicles
        .as_ref()
        .map(|db| (0..db.count_colour_brands(kind, false)).map(|i| db.get_nth_colour_brand(kind, i, false)).collect())
        .unwrap_or_default();
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        fe.text(o.menu_title).set_string(string_crc("CARCUSTOMISATION_STYL_COLOUR"));
        for b in &brands {
            menu::add_icon_entry(o, fe, *b, true, CUSTOM_VCD, "mod-icon_paint", None);
        }
        menu::set_selection(o, fe, brands.len() as i32);
        menu::set_active(o, fe, true);
        menu::show(o, fe, true, false);
    });
}

/// FECarCustomisationScreen.cpp 0x00253140 / 0x00253770 / 0x00253370 / 0x0024F680 — installed part checks for the
/// dialog: is list entry `idx` the fitted one, is the fitted one free, what it sells back for, its name
fn fitted_part<'a>(s: &CFECarCustomisationScreen, db: &'a CVehicleDatabase) -> Option<&'a velocity_physics::vehicle_database::vehicle_database_upgrade_bin_loading::Bodypart> {
    let ty = s.part_type as usize;
    s.setup_copy.fitted_bodyparts.get(ty).copied().flatten().map(|p| &db.bodypart_sets[p.set].parts[p.part_type][p.index])
}

fn is_installed(s: &CFECarCustomisationScreen, fe: &mut Feck2, idx: i32) -> bool {
    if (TYPE_DECAL_FIRST..=TYPE_DECAL_LAST).contains(&s.part_type) {
        // 0x00253140: the chosen decal is the one on the slot as bought
        return s.decal_sel == decal_index(s, fe, true);
    }
    let Some(db) = fe.vehicles.as_ref() else { return false };
    if s.part_type < 14 {
        let cand = s.bodypart_set.and_then(|set| db.bodypart_sets[set].parts[s.part_type as usize].get(idx.max(0) as usize));
        return matches!((fitted_part(s, db), cand), (Some(f), Some(c)) if f.hash == c.hash);
    }
    match s.part_type {
        TYPE_WHEEL => db.find_wheel(s.part_hash).is_some_and(|w| db.wheels[w].fitted != 0),
        TYPE_BRAKE => db.find_brake_part(s.part_hash).is_some_and(|b| db.brake_parts[b].flags & 2 != 0),
        TYPE_TYRE => db.find_tyre(s.part_hash).is_some_and(|t| db.tyres[t].fitted != 0),
        _ => {
            let mask = if s.part_type == TYPE_PERF_KIT { 16 } else { 8 };
            db.get_nth_setup(fe.garage.custom_vehicle, idx.max(0) as u32, mask, false).is_some_and(|p| setup_matches(db, &s.setup_copy, p))
        }
    }
}

/// VehicleDatabaseSetups.cpp 0x0018AAC0 — every part the predefined setup fits is fitted
fn setup_matches(db: &CVehicleDatabase, setup: &VehicleSetup, predef: usize) -> bool {
    let p = &db.setups[predef];
    let mut want = 0;
    let mut have = 0;
    for i in 0..14 {
        if let (true, Some(pp)) = (p.fit_bodypart[i] != 0, p.setup_bodyparts[i]) {
            want += 1;
            let ph = db.bodypart_sets[pp.set].parts[pp.part_type][pp.index].hash;
            if setup.fitted_bodyparts[i].is_some_and(|f| db.bodypart_sets[f.set].parts[f.part_type][f.index].hash == ph) {
                have += 1;
            }
        }
    }
    have == want
}

fn installed_is_free(s: &CFECarCustomisationScreen, fe: &Feck2) -> bool {
    if (TYPE_DECAL_FIRST..=TYPE_DECAL_LAST).contains(&s.part_type) {
        return false; // 0x00253770
    }
    let Some(db) = fe.vehicles.as_ref() else { return true };
    let c = &s.setup_copy;
    match s.part_type {
        t if t < 14 => fitted_part(s, db).is_none_or(|p| p.cost == 0),
        TYPE_WHEEL => c.wheel.is_none_or(|w| db.wheels[w].cost == 0),
        TYPE_BRAKE => c.brake_parts[(s.sel[3] != 0) as usize].is_none_or(|b| db.brake_parts[b].cost == 0),
        TYPE_TYRE => c.tyre.is_none_or(|t| db.tyres[t].cost == 0),
        _ => {
            let mask = if s.part_type == TYPE_PERF_KIT { 16 } else { 8 };
            db.get_nth_setup(fe.garage.custom_vehicle, 0, mask, false).is_some_and(|p| setup_matches(db, c, p))
        }
    }
}

fn installed_value(s: &CFECarCustomisationScreen, fe: &mut Feck2) -> i32 {
    if (TYPE_DECAL_FIRST..=TYPE_DECAL_LAST).contains(&s.part_type) {
        // 0x00253370: the decal on the slot as bought, its price (the record's +0x0C word)
        let d = shown_decal(s, fe, true);
        return d.and_then(|d| fe.vehicles.as_ref().map(|db| db.decals[d].unk_0c as u16 as i16 as i32)).unwrap_or(0);
    }
    let Some(db) = fe.vehicles.as_ref() else { return 0 };
    let c = &s.setup_copy;
    match s.part_type {
        t if t < 14 => fitted_part(s, db).map_or(0, |p| p.cost as i32),
        TYPE_WHEEL => c.wheel.map_or(0, |w| db.wheels[w].cost as i32),
        TYPE_BRAKE => c.brake_parts[(s.sel[3] != 0) as usize].map_or(0, |b| db.brake_parts[b].cost as i32),
        TYPE_TYRE => c.tyre.map_or(0, |t| db.tyres[t].cost as i32),
        _ => 0, // kits: 0x0018AB70 with the sell-back multiplier, not ported
    }
}

fn installed_name(s: &CFECarCustomisationScreen, fe: &mut Feck2) -> u32 {
    if (TYPE_DECAL_FIRST..=TYPE_DECAL_LAST).contains(&s.part_type) {
        let d = shown_decal(s, fe, true); // 0x0024F680
        return d.and_then(|d| fe.vehicles.as_ref().map(|db| db.decals[d].hash)).unwrap_or(0);
    }
    let Some(db) = fe.vehicles.as_ref() else { return 0 };
    let c = &s.setup_copy;
    match s.part_type {
        t if t < 14 => fitted_part(s, db).map_or(0, |p| p.hash),
        TYPE_WHEEL => c.wheel.map_or(0, |w| db.wheels[w].hash),
        TYPE_BRAKE => c.brake_parts[(s.sel[3] != 0) as usize].map_or(0, |b| db.brake_parts[b].hash),
        TYPE_TYRE => c.tyre.map_or(0, |t| db.tyres[t].hash),
        _ => 0,
    }
}

/// FECarCustomisationScreen.cpp 0x0024F860 — the buy dialog (`answered` = false: ask; true: act on the answer)
/// FECarCustomisationScreen.cpp 0x002515D0 / 0x002514E0 / 0x00251550 — the gauges list: one icon entry per `\OSD\HUDS\*.vcd` that has a
/// config (text = the config's hud name, icon = `<stem>-thumb`), selection = the setup's current hud hash.
/// `show` false = just leave gauges mode (the cancel / purchase paths call it with 0).
fn gauges_list(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, show: bool) {
    s.gauges_mode = show;
    if !show {
        return;
    }
    let mut hashes = Vec::new();
    let mut entries: Vec<(String, String)> = Vec::new();
    s.gauge_selected = 0;
    for stem in HudConfig::list() {
        match HudConfig::load(&stem) {
            Ok(c) => {
                let h = vs_string_hash(&stem);
                if h == s.setup_copy.unk_188 {
                    s.gauge_selected = hashes.len() as i32;
                }
                hashes.push(h);
                entries.push((stem, c.name));
            }
            Err(_) => log::error!("ERROR: CFECarCustomisationScreen::GaugesOptionEnabled - Config file doesn't exist : {}\nIgnoring this hud setup", stem),
        }
    }
    s.gauge_hashes = hashes;
    let sel = s.gauge_selected;
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        fe.text(o.menu_title).set_string(string_crc("CARCUSTOMISATION_STYL_GAUGES"));
        for (stem, name) in &entries {
            let vcd = format!(r"\OSD\HUDS\{}.VCD", stem.to_ascii_uppercase());
            let i = menu::add_icon_entry(o, fe, string_crc("GENERAL_BLANK"), true, &vcd, &format!("{}-THUMB", stem.to_ascii_uppercase()), None);
            let t: Vec<u16> = name.encode_utf16().collect();
            let id = o.menu[i].text;
            fe.text(id).set_text(&t);
        }
        menu::set_selection(o, fe, sel);
        menu::show(o, fe, true, false);
    });
}

fn purchase(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, answered: bool) {
    if s.gauges_mode {
        // 0x002514E0: the highlighted hud goes into the setup as bought, back to the groups
        s.gauge_selected = fe.overlay.as_ref().map_or(0, |o| o.menu_selected);
        if let Some(h) = s.gauge_hashes.get(s.gauge_selected.max(0) as usize) {
            s.setup_copy.unk_188 = *h;
        }
        gauges_list(s, fe, false);
        s.level = 2;
        level2_menu(s, fe, true);
        return;
    }
    let mut cash = fe.garage.cash;
    let mut cost = 0;
    let mut refund = 0;
    let mode;
    let idx = s.sel[5];
    if is_installed(s, fe, idx) {
        refund = installed_value(s, fe);
        cash += refund;
        mode = 1; // selling back
    } else {
        cost = part_cost(s, fe, s.part_type, idx);
        cash -= cost;
        if installed_is_free(s, fe) {
            mode = 0;
        } else {
            refund = installed_value(s, fe);
            cash += refund;
            mode = 2; // replacing
        }
    }
    if !answered {
        if part_locked(s, fe) {
            part_list(s, fe, false);
            s.level -= 1;
            return;
        }
        let (title, args): (u32, Vec<(u32, u32)>) = if mode == 1 {
            (DLG_SELL_BACK, vec![(ARG_PART_NAME, s.part_hash), (ARG_REFUND, refund as u32)])
        } else if mode == 0 {
            (DLG_CONFIRM, vec![(ARG_PART_NAME, s.part_hash), (string_crc("dPartCost"), cost as u32)])
        } else {
            let old = installed_name(s, fe);
            if old != 0 {
                (DLG_REPLACE, vec![(ARG_PART_NAME, s.part_hash), (ARG_OLD_PART, old)])
            } else {
                (DLG_BUY, vec![(ARG_PART_NAME, s.part_hash)])
            }
        };
        let mut args = args;
        if mode != 1 {
            args.push((ARG_CASH_DIFFERENCE, (refund - cost) as u32));
        }
        ov(fe, |o, fe| {
            overlay::set_dialog_strings(o, fe, title, &[GENERAL_YES, GENERAL_NO]);
            if let Some(h) = fe.text(o.dialog_item).string {
                for (a, v) in &args {
                    fe.strings.set_argument(h, *a, *v).ok();
                }
            }
            overlay::set_dialog_visible(o, fe, true, 0);
        });
        part_list(s, fe, false);
        level3_menu(s, fe, false);
        s.level = 6;
        return;
    }
    let yes = fe.overlay.as_ref().is_some_and(|o| o.dialog_selection == 0);
    if yes {
        fe.garage.cash = cash;
        ov(fe, overlay::update_cash);
        if s.part_type < 14 {
            if let Some(db) = fe.vehicles.as_mut() {
                if let Some(p) = s.setup_copy.fitted_bodyparts[s.part_type as usize] {
                    db.bodypart_sets[p.set].parts[p.part_type][p.index].flags &= !8; // 0x00186870
                }
            }
            if mode == 1 {
                // sold back: the list's first part goes on
                let set = s.bodypart_set;
                let ty = s.part_type as usize;
                if let (Some(set), Some(st)) = (set, shown_setup(fe)) {
                    st.fitted_bodyparts[ty] = Some(velocity_physics::vehicle_database::vehicle_database::PartRef { set, part_type: ty, index: 0 });
                }
            }
        } else {
            if (s.part_type == TYPE_BODY_KIT || s.part_type == TYPE_PERF_KIT) && mode == 1 {
                s.sel[5] = 0;
                apply_part(s, fe);
            }
            let hash = s.part_hash;
            if let Some(db) = fe.vehicles.as_mut() {
                match s.part_type {
                    TYPE_WHEEL => {
                        if let Some(w) = db.find_wheel(hash) {
                            db.wheels[w].fitted = 0;
                        }
                    }
                    TYPE_BRAKE => {
                        if let Some(b) = db.find_brake_part(hash) {
                            db.brake_parts[b].flags &= !2;
                        }
                    }
                    TYPE_TYRE => {
                        if let Some(t) = db.find_tyre(hash) {
                            db.tyres[t].fitted = 0;
                        }
                    }
                    _ => {}
                }
            }
        }
        keep_setup(s, fe);
        fe.el(s.part_cost).set_visible(false);
        ov(fe, |o, fe| menu::set_active(o, fe, false));
        let item = s.sel[3];
        if s.part_type < 14 || s.part_type == TYPE_BODY_KIT || s.part_type == TYPE_BRAKE || s.part_type == TYPE_TYRE || s.decal_mode {
            s.level = 3;
            part_list(s, fe, false);
            level3_menu(s, fe, true);
            s.sel[3] = item;
        } else {
            let group = s.sel[2];
            s.level = 2;
            part_list(s, fe, false);
            level2_menu(s, fe, true);
            s.sel[2] = group;
        }
        s.decal_mode = false;
    } else {
        s.level = 5;
        part_list(s, fe, true);
        restore_setup(s, fe);
    }
    ov(fe, |o, fe| overlay::set_dialog_visible(o, fe, false, 0));
}

/// FEPlayerGarageScreen.cpp 0x0024CDE0 / FECarCustomisationScreen.cpp 0x00250530 / 0x00251C60 / 0x00251D60 — Cancel: up a level (restoring the shown setup), leave from level 1
fn cc_cancel(s: &mut CFECarCustomisationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if fe.overlay.as_ref().is_some_and(|o| o.dialog_active) {
        ov(fe, |o, fe| overlay::set_dialog_visible(o, fe, false, 0));
        s.level = 5;
        part_list(s, fe, true);
        return true;
    }
    if s.level == 5 && s.top_is(0, &[4]) {
        restore_setup(s, fe);
        s.level = 2;
    } else if s.level == 5 && s.top_is(1, &[3, 4]) {
        // 0x00251550: the setup as bought back on the car, the list gone
        gauges_list(s, fe, false);
        let copy = s.setup_copy.clone();
        if let Some(st) = shown_setup(fe) {
            *st = copy;
        }
        s.gauge_hashes.clear();
        s.level = 2;
    } else if s.top_is(2, &[0]) {
        // 0x00250530: paint
        s.level -= 1;
        if s.level == 4 && (s.sel[3] == 0 || s.sel[3] == 4) {
            s.level -= 1;
        }
        let copy = s.setup_copy.clone();
        if let Some(st) = shown_setup(fe) {
            *st = copy;
        }
        swatches(s, fe, false);
        if s.sel[3] == 4 {
            colour_mode(s, fe, false);
        }
    } else if s.top_is(1, &[2]) {
        // decals (0x002508F0)
        s.level -= 1;
        if s.level == 4 {
            s.level -= 1;
        }
    } else if s.level != 1 && s.top_is(2, &[1, 3, 4]) {
        // 0x00251C60
        swatches(s, fe, false);
        if s.level == 3 {
            s.level = 2;
        } else if s.level == 2 {
            s.level = 1;
        }
        let copy = s.setup_copy.clone();
        if let Some(st) = shown_setup(fe) {
            *st = copy;
        }
    } else if s.level != 1 && s.top_is(2, &[2]) {
        // 0x00251D60
        s.brakepad_mode = false;
        if s.level == 3 {
            s.level = 2;
        } else if s.level == 2 {
            s.level = 1;
        }
        let copy = s.setup_copy.clone();
        if let Some(st) = shown_setup(fe) {
            *st = copy;
        }
    } else {
        s.level -= 1;
        if s.level == 4 && s.sel[1] != 2 {
            s.level -= 1;
        }
    }
    if s.level != 5 {
        ov(fe, |o, fe| menu::set_active(o, fe, true));
    }
    match s.level {
        1 => level1_menu(s, fe, true),
        2 => level2_menu(s, fe, true),
        3 => {
            restore_setup(s, fe);
            part_list(s, fe, false);
            level3_menu(s, fe, true);
        }
        4 if s.sel[1] == 2 => brand_menu(s, fe),
        0 => {
            if fe.jump_to_car_customisation {
                s.level = 1;
                level1_menu(s, fe, true);
            } else {
                s.exit_screen = string_crc("PREVIOUS");
                ov(fe, overlay::start_transition);
                s.exit_countdown = 20;
            }
        }
        _ => {}
    }
    true
}
