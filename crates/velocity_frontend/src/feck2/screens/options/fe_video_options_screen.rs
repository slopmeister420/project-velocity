//! `VideoOptionsScreen` (`gpVideoOptionsScreen`, 48 bytes): screen ratio, trash talk display and player names,
//! toggled with left/right. Screen adjust (row 0) has no handler in this build.
//!
//! port deviation (user request 2026-11-09): the Ratio row (`OPTION_VIDEO_RATIO`, 4:3 / 16:9) is removed, the UI
//! adapts to the window aspect by itself; the rows below move up one (TFT row 1, names row 2) and the value
//! elements take the original's y of the rows they now sit on (170 / 190).

use std::sync::{Arc, Mutex};

use super::{enter, ov, reset_keep_background};
use crate::feck2::feck2_manager::Feck2;
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::game_options::display;
use crate::string_manager::string_manager::string_crc;

/// `dword_766168` (static init 0x0042D840) without `OPTION_VIDEO_RATIO` (port: ratio row removed)
pub const VIDEO_ITEMS: [&str; 3] = ["OPTION_VIDEO_SCREENADJUST", "OPTION_VIDEO_TFTDISPLAY", "OPTION_VIDEO_PLAYERNAME"];

pub struct CFEVideoOptionsScreen {
    pub screen: ScreenId,       // +0
    pub layer: LayerId,         // +4
    pub values: [ElementId; 2], // +32 trash talk, +36 names (+28 ratio: removed)
    pub loaded: bool,           // +40
    pub selected: i32,          // +44
}

/// FEOnlineSignInScreen.cpp 0x00236290 / 0x002363C0 / 0x00236720 — `FECK2CreateVideoOptionsScreen` (Init, AssignEvents)
pub fn create_video_options_screen(fe: &mut Feck2) -> Arc<Mutex<CFEVideoOptionsScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("VideoOptionsScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("VideoOptions_Layer"), ..Default::default() });
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 1, crc: string_crc("optionmenutext"), string: 0, ..Default::default() };
    let mut values = [0; 2];
    // original y 170 / 190 / 210 for ratio / tft / names; the two kept rows now sit at 170 / 190
    for (i, y) in [1126825984u32, 1128136704].into_iter().enumerate() {
        d.position = [f32::from_bits(1138491392), f32::from_bits(y), 0.0, 1.0];
        values[i] = fe.create_element(layer, &d);
    }
    let obj = Arc::new(Mutex::new(CFEVideoOptionsScreen { screen, layer, values, loaded: false, selected: 0 }));
    // 0x00236720 — AssignEvents
    fe.map(layer, functor(&obj, on_enter), ev::ENTER);
    fe.map(layer, functor(&obj, on_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEAVE);
    fe.map(layer, functor(&obj, |_, fe, _| {
        super::ov(fe, |o, _| o.flag534 = true);
        true
    }), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UPDATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| {
        s.selected = fe.overlay.as_ref().map(|o| o.menu_selected).unwrap_or(0);
        refresh(s, fe, -1);
        true
    }), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| {
        s.selected = fe.overlay.as_ref().map(|o| o.menu_selected).unwrap_or(0);
        refresh(s, fe, 1);
        true
    }), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, |_, fe, _| {
        fe.goto_previous_screen(1, 0);
        false
    }), ev::CANCEL_PRESS);
    obj
}

/// FEOnlineSignInScreen.cpp 0x00236B80 — Enter
fn on_enter(s: &mut CFEVideoOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    enter(fe, s.screen);
    s.loaded = true;
    for v in s.values {
        fe.element_load(v);
    }
    refresh(s, fe, 0);
    true
}

/// FEOnlineSignInScreen.cpp 0x00236C50 — Activate
fn on_activate(s: &mut CFEVideoOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let sel = s.selected;
    ov(fe, |o, fe| {
        reset_keep_background(o, fe);
        overlay::set_nav_text(o, fe, overlay::NAV_SELECT, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, overlay::NAV_CANCEL, string_crc("GENERAL_BACK"));
        overlay::set_title(o, fe, string_crc("OPTION_VIDEO_TITLE"));
        overlay::show_borders(o, fe);
        overlay::enable(o);
        menu::set_style(o, fe, 1);
        for item in VIDEO_ITEMS {
            menu::add_entry(o, fe, string_crc(item), true, false, 0);
        }
        menu::show(o, fe, true, false);
        menu::set_selection(o, fe, sel);
    });
    true
}

/// FEOnlineSignInScreen.cpp 0x00236F80 — toggle the selected row's value (`dir` != 0) and show the values (all of them when `dir` == 0).
fn refresh(s: &mut CFEVideoOptionsScreen, fe: &mut Feck2, dir: i32) {
    let mut changed = false;
    // port: the ratio row (original selection 1) is gone
    if s.selected == 1 || dir == 0 {
        if dir != 0 {
            let v = &mut fe.options.display[display::TFT];
            *v = !*v;
            changed = true;
        }
        let str = if fe.options.display[display::TFT] { string_crc("GENERAL_ON") } else { string_crc("GENERAL_OFF") };
        fe.text(s.values[0]).set_string(str);
    }
    if s.selected == 2 || dir == 0 {
        if dir != 0 {
            let v = &mut fe.options.display[display::NAMES];
            *v = !*v;
            changed = true;
        }
        let str = if fe.options.display[display::NAMES] { string_crc("GENERAL_ON") } else { string_crc("GENERAL_OFF") };
        fe.text(s.values[1]).set_string(str);
    }
    if changed {
        fe.options.apply_globals();
        fe.options_changed = true;
    }
}
