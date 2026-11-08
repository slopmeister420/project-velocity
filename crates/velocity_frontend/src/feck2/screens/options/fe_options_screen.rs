//! `OptionsScreen` (`gpOptionsScreen`, 16 bytes): the options list on the overlay menu.

use std::sync::{Arc, Mutex};

use super::{enter, ov, reset_keep_background};
use crate::feck2::feck2_manager::Feck2;
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::string_manager::string_manager::string_crc;

/// `dword_766940` (static init 0x0042DD03), plus the port's `OPTION_PC` (PC settings page) after VIDEO
pub const OPTIONS_ITEMS: [&str; 8] = [
    "OPTION_PROFILE", "OPTION_LANGUAGE", "OPTION_VIDEO", "OPTION_PC", "OPTION_SOUND", "OPTION_CONTROLLERCONFIG", "OPTION_TUTORIALS",
    "OPTION_TWEAK",
];

pub struct CFEOptionsScreen {
    pub screen: ScreenId, // +0
    pub layer: LayerId,   // +4
    pub loaded: bool,     // +8
    pub selected: i32,    // +12
}

/// FEOnlineSignInScreen.cpp 0x002396B0 / 0x00239790 / 0x002397E0 / 0x00239980 / 0x0023A160 — `FECK2CreateOptionsScreen` (ctor, Init, AssignEvents, right functor)
pub fn create_options_screen(fe: &mut Feck2) -> Arc<Mutex<CFEOptionsScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("OptionsScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: 1498348315, ..Default::default() });
    let obj = Arc::new(Mutex::new(CFEOptionsScreen { screen, layer, loaded: false, selected: 0 }));
    // 0x00239980 — AssignEvents
    fe.map(layer, functor(&obj, on_enter), ev::ENTER);
    fe.map(layer, functor(&obj, on_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEAVE);
    fe.map(layer, functor(&obj, on_start), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UPDATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEFT_PRESS);
    // 0x0023A160 calls an empty function
    fe.map(layer, functor(&obj, |_, _, _| true), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, on_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, |_, fe, _| {
        fe.goto_previous_screen(1, 0);
        false
    }), ev::CANCEL_PRESS);
    fe.map(layer, functor(&obj, |_, fe, _| {
        fe.goto_previous_screen(1, 0);
        true
    }), ev::PAUSE_PRESS);
    obj
}

/// FEOnlineSignInScreen.cpp 0x00239E40 — Enter
fn on_enter(s: &mut CFEOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    enter(fe, s.screen);
    s.loaded = true;
    true
}

/// FEOnlineSignInScreen.cpp 0x00239EB0 — Activate
fn on_activate(s: &mut CFEOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let sel = s.selected;
    ov(fe, |o, fe| {
        reset_keep_background(o, fe);
        overlay::set_nav_text(o, fe, overlay::NAV_SELECT, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, overlay::NAV_CANCEL, string_crc("GENERAL_BACK"));
        overlay::set_title(o, fe, string_crc("GENERAL_OPTIONS"));
        overlay::show_borders(o, fe);
        // 0x0027AA90 ("FE_Icon_Hangout"): only hides the icon border
        fe.el(o.borders[overlay::ICON]).set_visible(false);
        menu::set_style(o, fe, 1);
        menu::set_window(o, 5);
        for item in OPTIONS_ITEMS {
            menu::add_entry(o, fe, string_crc(item), true, false, 0);
        }
        menu::show(o, fe, true, false);
        menu::set_selection(o, fe, sel);
        overlay::enable(o);
    });
    true
}

/// FEOnlineSignInScreen.cpp 0x0023A080 — Start: no left/right cursor sound on this screen.
fn on_start(_s: &mut CFEOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, _| o.flag534 = true);
    true
}

/// FEOnlineSignInScreen.cpp 0x0023A190 — Select
fn on_select(s: &mut CFEOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.selected = fe.overlay.as_ref().map(|o| o.menu_selected).unwrap_or(0);
    // port: rows from 3 on are one lower than the original's (OPTION_PC inserted at 3)
    match s.selected {
        2 => fe.goto_screen_name("VideoOptionsScreen"),
        1 => fe.goto_screen_name("LanguageOptionsScreen"),
        0 => fe.goto_screen_name("ProfileOptionsScreen"),
        3 => fe.goto_screen_name(super::fe_pc_options_screen::SCREEN),
        5 => fe.goto_screen_name("ConfigOptionsScreen"),
        // FEHangoutScreen 0x0021C890 and FEMainMenuScreen 0x0023F1F0(-1) prepare the tips run first
        6 => {
            crate::feck2::screens::garage::ov(fe, |o, _| o.first_activate = true);
            crate::feck2::screens::common::fe_tips_screen::set_tip(fe, -1);
            fe.goto_screen_name("TipsScreen");
        }
        4 => fe.goto_screen_name("AudioOptionsScreen"),
        // GameManager 0x000BC3D0: the in-game tweak menu, nothing in the front end
        _ => {}
    }
    false
}
