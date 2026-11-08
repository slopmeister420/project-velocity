//! `PlateCustomizationScreen` (`gpPlateCustomizationScreen` `dword_766960`, 0x20 bytes; code 0x0023A540..0x0023B370
//! in the idb's FEOnlineSignInScreen range, tag `FECK2CreatePlateCustomizationScreen` in CreateAllScreens 0x001FED04):
//! the licence plate. Stage 0 picks the style from the overlay menu (AMERICAN / BRITISH / JAPANESE, `dword_766BE8`),
//! stage 1 types the text on the common keyboard (8 characters, pre-filled with the current text); DONE stores it and
//! goes back. Reached from ProfileOptionsScreen (row 0) and a new profile (ProfileSelectScreen states 8 / 9).
//! IDA dump: `research/fe_plate.c`.

use std::sync::{Arc, Mutex};

use super::{enter, ov, reset_keep_background};
use crate::feck2::feck2_manager::Feck2;
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::feck2::screens::common::fe_keyboard_overlay::kb;
use crate::string_manager::string_manager::string_crc;

/// `dword_766BE8` (static init 0x0042DE90)
pub const PLATE_STYLES: [&str; 3] = ["OPTION_PLATE_STYLE_AMERICAN", "OPTION_PLATE_STYLE_BRITISH", "OPTION_PLATE_STYLE_JAPANESE"];
/// FEOnlineSignInScreen.cpp 0x0023AFA0: `SetMaxStringSize(8)`
pub const PLATE_TEXT_SIZE: usize = 8;

/// the game variables the screen edits: `dword_6F71DC` (style, GamePlayer +0x26C) and `unk_6F71E0` (text)
#[derive(Clone, Debug, PartialEq)]
pub struct PlateVars {
    pub style: u32,
    pub text: String,
}

impl Default for PlateVars {
    /// GamePlayer 0x000C1930 (0, "OCTANE")
    fn default() -> Self {
        Self { style: 0, text: "OCTANE".into() }
    }
}

/// `CFEPlateCustomizationScreen` (ctor 0x0023A620)
pub struct CFEPlateCustomizationScreen {
    pub screen: ScreenId, // +0
    pub layer: LayerId,   // +4
    pub loaded: bool,     // +20
    pub stage: i32,       // +24 0 style, 1 text, 2 done (-1 back)
    pub style: i32,       // +28
}

/// FEOnlineSignInScreen.cpp 0x0023A540 / 0x0023A620 / 0x0023A670 / 0x0023A840 / 0x0023B0B0 / 0x0023ADB0 / 0x0023B090 — `FECK2CreatePlateCustomizationScreen` (ctor, Init, AssignEvents + its empty functors)
pub fn create_plate_customization_screen(fe: &mut Feck2) -> Arc<Mutex<CFEPlateCustomizationScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("PlateCustomizationScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("PlateCustomizationScreen_Layer"), ..Default::default() });
    let obj = Arc::new(Mutex::new(CFEPlateCustomizationScreen { screen, layer, loaded: false, stage: 0, style: 0 }));
    fe.map(layer, functor(&obj, |_, _, _| true), ev::PRELOAD); // 0x0023B0B0
    fe.map(layer, functor(&obj, pc_enter), ev::ENTER);
    // 0x0023AC10 maps 0x0023B0D0 to UPDATE again (the same functor)
    fe.map(layer, functor(&obj, pc_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEAVE); // 0x0023ADB0
    fe.map(layer, functor(&obj, pc_start), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN); // 0x0023B090
    fe.map(layer, functor(&obj, pc_update), ev::UPDATE);
    fe.map(layer, functor(&obj, pc_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, pc_cancel), ev::CANCEL_PRESS);
    obj
}

/// FEOnlineSignInScreen.cpp 0x0023ABA0 — Enter: overlay and camera here
fn pc_enter(s: &mut CFEPlateCustomizationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    enter(fe, s.screen);
    if !s.loaded {
        s.loaded = true;
    }
    true
}

/// FEOnlineSignInScreen.cpp 0x0023AC10 — Activate: the options look, SELECT / BACK prompts, title CUSTOMIZE_PLATE_HEADING, menu style 1
fn pc_activate(_s: &mut CFEPlateCustomizationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| {
        reset_keep_background(o, fe);
        overlay::set_nav_text(o, fe, overlay::NAV_SELECT, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, overlay::NAV_CANCEL, string_crc("GENERAL_BACK"));
        overlay::set_title(o, fe, string_crc("CUSTOMIZE_PLATE_HEADING"));
        overlay::show_borders(o, fe);
        menu::set_style(o, fe, 1);
        overlay::enable(o);
        menu::set_style(o, fe, 1);
    });
    true
}

/// FEOnlineSignInScreen.cpp 0x0023ADD0 — Start: stage 0
fn pc_start(s: &mut CFEPlateCustomizationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.stage = 0;
    step(s, fe, 0);
    true
}

/// FEOnlineSignInScreen.cpp 0x0023AE10 — move the stage by `delta`: -1 back, 0 the style menu, 1 the keyboard (the menu selection becomes
/// the style), 2 back
fn step(s: &mut CFEPlateCustomizationScreen, fe: &mut Feck2, delta: i32) {
    if delta != 0 {
        s.stage += delta;
    }
    if s.stage == -1 {
        fe.goto_previous_screen(1, 0);
    }
    match s.stage {
        0 => {
            show_keyboard(fe, false);
            show_menu(s, fe, true);
        }
        1 => {
            s.style = fe.overlay.as_ref().map_or(0, |o| o.menu_selected);
            fe.plate.style = s.style as u32;
            fe.plate_changed = true;
            show_menu(s, fe, false);
            show_keyboard(fe, true);
        }
        2 => {
            show_menu(s, fe, false);
            show_keyboard(fe, false);
            fe.goto_previous_screen(1, 0);
        }
        _ => {}
    }
}

/// FEOnlineSignInScreen.cpp 0x0023AEF0 — the style menu (selection = `dword_6F71DC`), or hide it
fn show_menu(s: &mut CFEPlateCustomizationScreen, fe: &mut Feck2, show: bool) {
    if !show {
        ov(fe, |o, fe| menu::show(o, fe, false, false));
        return;
    }
    s.style = fe.plate.style as i32;
    let sel = s.style;
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        for st in PLATE_STYLES {
            menu::add_entry(o, fe, string_crc(st), true, false, 0);
        }
        menu::show(o, fe, true, false);
        menu::set_selection(o, fe, sel);
    });
}

/// FEOnlineSignInScreen.cpp 0x0023AFA0 — the keyboard: shown with the case locked (+74), 8 characters, the current text
fn show_keyboard(fe: &mut Feck2, show: bool) {
    if show {
        let text = fe.plate.text.clone().into_bytes();
        kb(fe, |k, fe| {
            k.show(fe, true);
            k.case_locked = true;
            k.set_max_string_size(PLATE_TEXT_SIZE);
            k.set_string(fe, &text);
        });
    } else {
        kb(fe, |k, fe| k.show(fe, false));
    }
}

/// FEOnlineSignInScreen.cpp 0x0023B0D0 — Update: DONE on the keyboard stores the text (`unk_766BF4` -> `unk_6F71E0`) and goes back
fn pc_update(_s: &mut CFEPlateCustomizationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let text = kb(fe, |k, _| (k.is_done() && k.is_visible()).then(|| k.text().to_vec())).flatten();
    if let Some(t) = text {
        kb(fe, |k, fe| k.show(fe, false));
        fe.plate.text = String::from_utf8_lossy(&t).into_owned();
        fe.plate_changed = true;
        fe.goto_previous_screen(1, 0);
    }
    true
}

/// FEOnlineSignInScreen.cpp 0x0023B170 — Select: left to the keyboard while it is up and not done, else the next stage
fn pc_select(s: &mut CFEPlateCustomizationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if kb(fe, |k, _| k.is_visible() && !k.is_done()) == Some(true) {
        return false;
    }
    step(s, fe, 1);
    true
}

/// FEOnlineSignInScreen.cpp 0x0023B1D0 — Cancel: the previous stage
fn pc_cancel(s: &mut CFEPlateCustomizationScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    step(s, fe, -1);
    true
}
