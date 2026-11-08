//! `ConfigOptionsScreen` (`gpConfigOptionsScreen`, 68 bytes): the pad picture with what each button does, and
//! left/right through the pad mappings (`UserInput::SetDeviceMapping(0, n)`, 0x001E71F0). Mappings are the
//! `\Input\Xboxpad\Config<n>.ini` files; this install ships only `Config1.ini`, so only "A" exists.

use std::sync::{Arc, Mutex};

use super::{enter, ov, reset_keep_background};
use crate::feck2::feck2_manager::{visos_colour, Feck2, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::string_manager::string_manager::string_crc;
use velocity_game::controller::user_input::{parse_mapping_file, Expr, XBOXPAD_KEYS};

/// `dword_766698` (static init 0x0042DAB0)
pub const CONFIG_NAMES: [&str; 4] = ["OPTION_CONTROLLERCONFIG_A", "OPTION_CONTROLLERCONFIG_B", "OPTION_CONTROLLERCONFIG_C", "OPTION_CONTROLLERCONFIG_D"];
/// `off_6C1850`: the labelled pad inputs
pub const BUTTONS: [&str; 10] = ["A", "B", "X", "Y", "BLACK", "STICK_RY", "STICK_LX", "LEFT", "LTRIGGER", "RTRIGGER"];
/// `dword_6C17B0`: label positions
pub const BUTTON_POS: [[f32; 2]; 10] =
    [[395.0, 350.0], [395.0, 310.0], [395.0, 230.0], [395.0, 270.0], [395.0, 330.0], [315.0, 360.0], [55.0, 210.0], [55.0, 360.0], [125.0, 110.0], [325.0, 110.0]];
/// `dword_688B98`: (action, description) pairs the labels come from (static init 0x004288B3)
pub const ACTION_STRINGS: [(usize, &str); 15] = [
    (13, "CONFIGCONTROL_PAUSE"),
    (26, "CONFIGCONTROL_CHANGECAMERA"),
    (27, "CONFIGCONTROL_STEERING"),
    (28, "CONFIGCONTROL_STEERLEFT"),
    (29, "CONFIGCONTROL_STEERRIGHT"),
    (30, "CONFIGCONTROL_ACCELERATEBRAKE"),
    (31, "CONFIGCONTROL_ACCELERATE"),
    (32, "CONFIGCONTROL_BRAKE"),
    (33, "CONFIGCONTROL_HANDBRAKE"),
    (34, "CONFIGCONTROL_GEARUPDOWN"),
    (35, "CONFIGCONTROL_GEARUP"),
    (36, "CONFIGCONTROL_GEARDOWN"),
    (37, "CONFIGCONTROL_REARVIEW"),
    (38, "CONFIGCONTROL_NITROUSEMODE"),
    (39, "CONFIGCONTROL_ZONEMODE"),
];

/// UserInput.cpp 0x001E7090 — `GetNumDeviceMappings`: the consecutive `Config<n>.ini` files (at most 4 names).
pub fn mapping_count() -> u8 {
    let mut n = 0u8;
    while (n as usize) < CONFIG_NAMES.len() && visos::file::resolve(&format!(r"\Input\Xboxpad\Config{}.ini", n + 1)).exists() {
        n += 1;
    }
    n.max(1)
}

/// UserInput.cpp 0x001E7360 — the description of an action, 0 if it has none.
pub(crate) fn action_string(action: usize) -> u32 {
    ACTION_STRINGS.iter().find(|(a, _)| *a == action).map(|(_, s)| string_crc(s)).unwrap_or(0)
}

/// UserInput.cpp 0x001E6F80 — the name crc of the first input (InputManager 0x003D5720: first type-2 node) pad 0's
/// mapping `Config<config+1>.ini` binds to `action`, 0 if none. STAND-IN: the pad's device handle check
/// (0x001EA020) is taken as valid, as the config screen does.
pub(crate) fn action_button(config: i32, action: usize) -> u32 {
    let file = format!(r"\Input\Xboxpad\Config{}.ini", config + 1);
    let mapping = visos::file::platform::read(&file).map(|b| parse_mapping_file(&String::from_utf8_lossy(&b))).unwrap_or_default();
    mapping.iter().find(|(a, _)| *a == action).and_then(|(_, e)| first_key(e)).map_or(0, |k| string_crc(XBOXPAD_KEYS[k]))
}

/// the first input an expression reads.
fn first_key(e: &Expr) -> Option<usize> {
    match e {
        Expr::Key(k) => Some(*k),
        Expr::Digital(x) => first_key(x),
        Expr::DeadZone(x, d) => first_key(x).or_else(|| first_key(d)),
        Expr::Const(_) => None,
    }
}

pub struct CFEConfigOptionsScreen {
    pub screen: ScreenId,          // +0
    pub layer: LayerId,            // +4
    pub title: ElementId,          // +8
    pub layer_3d: LayerId,         // +12
    pub controller: ElementId,     // +16
    pub labels: [ElementId; 10],   // +20
    pub loaded: bool,              // +60
    pub config: i32,               // +64
}

/// FEOnlineSignInScreen.cpp 0x00238510 / 0x002385A0 / 0x00238A80 — `FECK2CreateConfigOptionsScreen` (Init, AssignEvents)
pub fn create_config_options_screen(fe: &mut Feck2) -> Arc<Mutex<CFEConfigOptionsScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("ConfigOptionsScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("ConfigOptions_Layer"), ..Default::default() });
    let layer_3d = fe.create_layer(Some(screen), &SLayerDesc { priority: 5, crc: string_crc("ConfigOptions_3DLayer"), ..Default::default() });
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 3, crc: string_crc("optionmenutext"), ..Default::default() };
    d.position = [f32::from_bits(1117782016), f32::from_bits(1117782016), 0.0, 1.0];
    d.string = string_crc(CONFIG_NAMES[0]);
    let title = fe.create_element(layer, &d);
    d.font = 0;
    d.crc = string_crc("ButtonName");
    let mut labels = [0; 10];
    for (i, l) in labels.iter_mut().enumerate() {
        d.position = [BUTTON_POS[i][0], BUTTON_POS[i][1], 0.0, 1.0];
        *l = fe.create_element(layer, &d);
    }
    d.kind = ELEMENT_2D;
    d.crc = string_crc("ConfigController");
    d.vcd = r"\FE\FE.vcd".into();
    d.texture = "controller".into();
    d.position = [f32::from_bits(1115684864), f32::from_bits(1115684864), 0.0, 1.0];
    let controller = fe.create_element(layer, &d);
    let obj = Arc::new(Mutex::new(CFEConfigOptionsScreen { screen, layer, title, layer_3d, controller, labels, loaded: false, config: 0 }));
    // 0x00238A80 — AssignEvents
    fe.map(layer, functor(&obj, on_enter), ev::ENTER);
    fe.map(layer, functor(&obj, on_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEAVE);
    fe.map(layer, functor(&obj, on_start), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, |s, fe, _| {
        fe.element_fade(s.title, 1, 0, 0);
        true
    }), ev::UPDATE);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| {
        cycle(s, fe, -1);
        false
    }), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| {
        cycle(s, fe, 1);
        false
    }), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, |_, fe, _| {
        fe.goto_previous_screen(1, 0);
        false
    }), ev::CANCEL_PRESS);
    obj
}

/// FEOnlineSignInScreen.cpp 0x00238EE0 — Enter
fn on_enter(s: &mut CFEConfigOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    enter(fe, s.screen);
    s.loaded = true;
    s.config = fe.options.controller_config[0] as i32;
    fe.element_load(s.title);
    fe.element_load(s.controller);
    fe.el(s.controller).set_visible(true);
    fe.e2d(s.controller).set_size(400, 400);
    for l in s.labels {
        fe.element_load(l);
        fe.el(l).set_visible(true);
    }
    true
}

/// FEOnlineSignInScreen.cpp 0x00239030 — Activate
fn on_activate(_s: &mut CFEConfigOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| {
        reset_keep_background(o, fe);
        overlay::set_nav_text(o, fe, overlay::NAV_SELECT, string_crc("GENERAL_ACCEPT"));
        overlay::set_nav_text(o, fe, overlay::NAV_CANCEL, string_crc("GENERAL_CANCEL"));
        overlay::show_arrows(o, fe, 0);
        overlay::show_borders(o, fe);
        overlay::enable(o);
    });
    true
}

/// FEOnlineSignInScreen.cpp 0x00239150 — Start: no up/down cursor sound, the title fades in.
fn on_start(s: &mut CFEConfigOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, _| o.no_up_down_sound = true);
    fe.text(s.title).set_colour(visos_colour(0, 128, 128, 128));
    fe.text(s.title).set_colour2(COLOUR_WHITE);
    cycle(s, fe, 0);
    true
}

/// FEOnlineSignInScreen.cpp 0x00239340 — step the mapping (wraps), apply it to pad 0, retitle and relabel.
fn cycle(s: &mut CFEConfigOptionsScreen, fe: &mut Feck2, dir: i32) {
    let count = mapping_count() as i32;
    s.config += dir;
    if s.config >= 0 {
        if s.config == count {
            s.config = 0;
        }
    } else {
        s.config = count - 1;
    }
    if fe.options.controller_config[0] as i32 != s.config {
        fe.options.controller_config[0] = s.config as u8;
        fe.options_changed = true;
    }
    fe.text(s.title).set_string(string_crc(CONFIG_NAMES[s.config as usize]));
    let w = (fe.text_width(s.title) + 80) as f32;
    ov(fe, |o, fe| overlay::position_arrows(o, fe, f32::from_bits(1115684864), w, f32::from_bits(1118175232), 0));
    relabel(s, fe);
}

/// FEOnlineSignInScreen.cpp 0x00239430 — each pad input gets the description of the first described action (13..39) it drives.
fn relabel(s: &mut CFEConfigOptionsScreen, fe: &mut Feck2) {
    let file = format!(r"\Input\Xboxpad\Config{}.ini", s.config + 1);
    let mapping = visos::file::platform::read(&file).map(|b| parse_mapping_file(&String::from_utf8_lossy(&b))).unwrap_or_default();
    for (i, button) in BUTTONS.iter().enumerate() {
        let want = string_crc(button);
        let mut label = 0;
        for action in 13..40 {
            let desc = action_string(action);
            if desc == 0 {
                continue;
            }
            let key = mapping.iter().find(|(a, _)| *a == action).and_then(|(_, e)| first_key(e));
            if key.map(|k| string_crc(XBOXPAD_KEYS[k])) == Some(want) {
                label = desc;
                break;
            }
        }
        let str = if label != 0 { label } else { string_crc("CONFIGCONTROL_NONE") };
        fe.text(s.labels[i]).set_string(str);
    }
}
