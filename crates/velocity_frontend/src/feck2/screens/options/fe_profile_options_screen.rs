//! `ProfileOptionsScreen` (`gpProfileOptionsScreen` `dword_765C30`, 36 bytes; code 0x00234FE0..0x00236290 in the
//! idb's FEOnlineSignInScreen range): the options' Profile row. Edit plate, profile info (cash / prestige / pink
//! slips / progression of the current game variables), save, load, new (confirm dialog) and delete go to the
//! ProfileSelectScreen in the matching mode. `dword_7013E4` (memory unit state, set to 4 for save / 0 for load) is
//! not kept: the profile select reads it as 0 (see there).

use std::sync::{Arc, Mutex};

use super::{enter, ov, reset_keep_background};
use crate::feck2::feck2_manager::Feck2;
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::feck2::screens::startup::fe_profile_select_screen::CFEProfileSelectScreen;
use crate::string_manager::string_manager::string_crc;

/// `dword_765EC8` (static init 0x0042D590)
pub const PROFILE_OPTIONS_ITEMS: [&str; 6] =
    ["OPTION_PROFILE_EDIT_PLATE", "OPTION_PROFILE_INFO", "OPTION_SAVE", "OPTION_LOAD", "OPTION_NEWPROFILE", "OPTION_DELETEPROFILE"];
/// `dword_765EB8` (static init 0x0042D6C0)
pub const PROFILE_INFO_ITEMS: [&str; 4] = ["PROFILE_INFO_CASH", "PROFILE_INFO_PRESTIGE", "PROFILE_INFO_PINKSLIPS", "PROFILE_INFO_PROGRESSION"];

/// `CFEProfileOptionsScreen` (0x24 bytes, ctor 0x002350C0)
pub struct CFEProfileOptionsScreen {
    pub screen: ScreenId,       // +0
    pub layer: LayerId,         // +4
    pub loaded: bool,           // +8
    pub info: [ElementId; 4],   // +12..+24
    pub selected: i32,          // +28
    pub info_shown: bool,       // +32
    /// `gpProfileSelectScreen` (a global in the original)
    pub profile_select: Arc<Mutex<CFEProfileSelectScreen>>,
}

/// FEOnlineSignInScreen.cpp 0x00234FE0 / 0x002350C0 / 0x00235110 / 0x002353D0 / 0x00235AB0 — `FECK2CreateProfileOptionsScreen` (ctor, Init, AssignEvents). `profile_select` stands for the global
/// `gpProfileSelectScreen` its Select sets the mode of (0x00232D40).
pub fn create_profile_options_screen(fe: &mut Feck2, profile_select: Arc<Mutex<CFEProfileSelectScreen>>) -> Arc<Mutex<CFEProfileOptionsScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("ProfileOptionsScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("ProfileOptions_Layer"), ..Default::default() });
    let mut d = SElementDesc::default();
    let mut info = [0; 4];
    for (i, id) in PROFILE_INFO_ITEMS.iter().enumerate() {
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = 1;
        d.crc = string_crc("profileProgress");
        d.position[0] = f32::from_bits(1123024896);
        d.position[1] = (25 * i as u32 + 150) as f32;
        d.string = string_crc(id);
        info[i] = fe.create_element(layer, &d);
    }
    let obj = Arc::new(Mutex::new(CFEProfileOptionsScreen { screen, layer, loaded: false, info, selected: 0, info_shown: false, profile_select }));
    // 0x002353D0 — AssignEvents
    fe.map(layer, functor(&obj, po_enter), ev::ENTER);
    fe.map(layer, functor(&obj, po_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEAVE);
    // 0x00235AB0: no left/right cursor sound (0x0027D340)
    fe.map(layer, functor(&obj, |_, fe, _| {
        ov(fe, |o, _| o.flag534 = true);
        true
    }), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UPDATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, po_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, po_cancel), ev::CANCEL_PRESS);
    obj
}

/// FEOnlineSignInScreen.cpp 0x00235830 — Enter: overlay and camera here, the info texts loaded once, the menu shown
fn po_enter(s: &mut CFEProfileOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    enter(fe, s.screen);
    if !s.loaded {
        s.loaded = true;
        for i in s.info {
            fe.element_load(i);
        }
    }
    show_info(s, fe, false);
    true
}

/// FEOnlineSignInScreen.cpp 0x002358F0 — Activate
fn po_activate(s: &mut CFEProfileOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let sel = s.selected;
    ov(fe, |o, fe| {
        reset_keep_background(o, fe);
        overlay::set_nav_text(o, fe, overlay::NAV_SELECT, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, overlay::NAV_CANCEL, string_crc("GENERAL_BACK"));
        overlay::show_borders(o, fe);
        overlay::enable(o);
        menu::set_style(o, fe, 1);
        for item in PROFILE_OPTIONS_ITEMS {
            menu::add_entry(o, fe, string_crc(item), true, false, 0);
        }
        fe.text(o.menu_title).set_string(string_crc("OPTION_PROFILE"));
        menu::show(o, fe, true, false);
        menu::set_selection(o, fe, sel);
    });
    true
}

/// FEOnlineSignInScreen.cpp 0x00235E70 — the info page instead of the menu (`show`), or back: the menu title reads OPTION_PROFILE_INFO with
/// cash (`dword_6F719C`), prestige (`dword_6F71A0`), pink slips (0) and progression (`100 * (completed / events)` in
/// integers, then 1 for any progress — the profile select's bug too)
fn show_info(s: &mut CFEProfileOptionsScreen, fe: &mut Feck2, show: bool) {
    s.info_shown = show;
    ov(fe, |o, fe| {
        menu::show(o, fe, !show, true);
        for i in s.info {
            fe.el(i).set_visible(show);
        }
        fe.el(o.menu_title).set_visible(true);
        menu::fade_title(o, fe, 128, 0);
        if show {
            fe.text(o.menu_title).set_string(string_crc("OPTION_PROFILE_INFO"));
        } else {
            fe.text(o.menu_title).set_string(string_crc("OPTION_PROFILE"));
        }
    });
    if !show {
        return;
    }
    let completed = fe.career.events.iter().filter(|e| e.flags & 2 != 0).count() as u32;
    // CompartmentManager 0x0008ECC0(db, 0, 0): every event
    let total = (fe.career.events.len() as u32).max(1);
    let mut percent = 100 * (completed / total);
    if percent == 0 {
        percent = (completed != 0) as u32;
    }
    let rows = [("dCash", fe.garage.cash as u32), ("dPrestige", fe.garage.prestige as u32), ("dPinkSlips", 0), ("dPercent", percent)];
    for (i, (arg, v)) in rows.into_iter().enumerate() {
        if let Some(h) = fe.text(s.info[i]).string {
            fe.strings.set_argument(h, string_crc(arg), v).ok();
        }
    }
}

/// FEOnlineSignInScreen.cpp 0x00235BA0 — Select
fn po_select(s: &mut CFEProfileOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let ps = s.profile_select.clone();
    if s.info_shown {
        show_info(s, fe, false);
        return false;
    }
    if fe.overlay.as_ref().is_some_and(|o| o.dialog_active) {
        // the new profile confirm: SaveGame_Yes (0) -> ProfileSelectScreen in new mode
        if s.selected == 4 {
            if fe.overlay.as_ref().is_some_and(|o| o.dialog_selection == 0) {
                ps.lock().unwrap().set_mode(5, false);
                fe.goto_screen_name("ProfileSelectScreen");
            }
            ov(fe, |o, fe| overlay::set_dialog_visible(o, fe, false, 0));
        }
        return true;
    }
    s.selected = fe.overlay.as_ref().map(|o| o.menu_selected).unwrap_or(0);
    match s.selected {
        0 => fe.goto_screen_name("PlateCustomizationScreen"),
        1 => show_info(s, fe, true),
        2 => {
            ps.lock().unwrap().set_mode(2, false);
            fe.goto_screen_name("ProfileSelectScreen");
        }
        3 => {
            ps.lock().unwrap().set_mode(1, false);
            fe.goto_screen_name("ProfileSelectScreen");
        }
        4 => ov(fe, |o, fe| {
            overlay::show_dialog(o, fe, 2, string_crc("SaveGame_New_Confirm"), &[string_crc("SaveGame_Yes"), string_crc("SaveGame_No")]);
            overlay::set_dialog_visible(o, fe, true, 0);
        }),
        5 => {
            ps.lock().unwrap().set_mode(4, false);
            fe.goto_screen_name("ProfileSelectScreen");
        }
        _ => {}
    }
    true
}

/// FEOnlineSignInScreen.cpp 0x00235E20 — Cancel: the info page closes, and back either way
fn po_cancel(s: &mut CFEProfileOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.info_shown {
        show_info(s, fe, false);
    }
    fe.goto_previous_screen(1, 0);
    true
}
