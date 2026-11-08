//! port (not original, user request 2026-11-09): `PcOptionsScreen`, the PC settings page under OPTIONS (row 3,
//! `OPTION_PC`). Built exactly like `VideoOptionsScreen` (overlay menu style 1, one value text per row, left/right
//! changes the selected row): display mode, resolution, frame rate cap, v-sync, music order (`GameOptions.pc`,
//! applied live by `pc_settings::apply_pc_settings`). The resolution row is only enabled in fullscreen: in
//! windowed / borderless it is drawn with the menu's disabled colours and up/down skip it.

use std::sync::{Arc, Mutex};

use super::{enter, ov, reset_keep_background};
use crate::feck2::feck2_manager::{visos_colour, Feck2, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::game_options::{display_mode, FPS_CAPS};
use crate::string_manager::string_manager::string_crc;

pub const SCREEN: &str = "PcOptionsScreen";

pub const PC_ITEMS: [&str; 5] = ["OPTION_PC_DISPLAYMODE", "OPTION_PC_RESOLUTION", "OPTION_PC_FPSCAP", "OPTION_PC_VSYNC", "OPTION_PC_MUSIC"];
pub const ROW_MODE: i32 = 0;
pub const ROW_RESOLUTION: i32 = 1;
pub const ROW_FPS: i32 = 2;
pub const ROW_VSYNC: i32 = 3;
pub const ROW_MUSIC: i32 = 4;

const MODE_STRINGS: [&str; 3] = ["OPTION_PC_DISPLAYMODE_WINDOWED", "OPTION_PC_DISPLAYMODE_BORDERLESS", "OPTION_PC_DISPLAYMODE_FULLSCREEN"];

pub struct CFEPcOptionsScreen {
    pub screen: ScreenId,
    pub layer: LayerId,
    pub values: [ElementId; 5],
    pub loaded: bool,
    pub selected: i32,
}

/// port — create the screen (the video options' Init / AssignEvents pattern, FEOnlineSignInScreen.cpp 0x00236290)
pub fn create_pc_options_screen(fe: &mut Feck2) -> Arc<Mutex<CFEPcOptionsScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc(SCREEN) });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("PcOptions_Layer"), ..Default::default() });
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 1, crc: string_crc("optionmenutext"), string: 0, ..Default::default() };
    let mut values = [0; 5];
    for (i, v) in values.iter_mut().enumerate() {
        // the video page's rows: y 150 + 20 * row; values right-aligned at x 500 (the labels here are longer)
        d.position = [500.0, 150.0 + 20.0 * i as f32, 0.0, 1.0];
        *v = fe.create_element(layer, &d);
        fe.text(*v).set_justification(1);
    }
    let obj = Arc::new(Mutex::new(CFEPcOptionsScreen { screen, layer, values, loaded: false, selected: 0 }));
    fe.map(layer, functor(&obj, on_enter), ev::ENTER);
    fe.map(layer, functor(&obj, on_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEAVE);
    fe.map(layer, functor(&obj, |_, fe, _| {
        ov(fe, |o, _| o.flag534 = true);
        true
    }), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UPDATE);
    // the menu layer has moved the selection already; step over a disabled row
    fe.map(layer, functor(&obj, |_, fe, _| skip_disabled(fe, -1)), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |_, fe, _| skip_disabled(fe, 1)), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| {
        s.selected = fe.overlay.as_ref().map(|o| o.menu_selected).unwrap_or(0);
        change(s, fe, -1);
        true
    }), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| {
        s.selected = fe.overlay.as_ref().map(|o| o.menu_selected).unwrap_or(0);
        change(s, fe, 1);
        true
    }), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, |_, fe, _| {
        fe.goto_previous_screen(1, 0);
        false
    }), ev::CANCEL_PRESS);
    obj
}

fn on_enter(s: &mut CFEPcOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    enter(fe, s.screen);
    s.loaded = true;
    for v in s.values {
        fe.element_load(v);
    }
    show_values(s, fe);
    true
}

fn on_activate(s: &mut CFEPcOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let fullscreen = fe.options.pc.display_mode == display_mode::FULLSCREEN;
    let mut sel = s.selected;
    if sel == ROW_RESOLUTION && !fullscreen {
        sel = ROW_MODE;
    }
    ov(fe, |o, fe| {
        reset_keep_background(o, fe);
        overlay::set_nav_text(o, fe, overlay::NAV_SELECT, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, overlay::NAV_CANCEL, string_crc("GENERAL_BACK"));
        overlay::set_title(o, fe, string_crc("OPTION_PC_TITLE"));
        overlay::show_borders(o, fe);
        overlay::enable(o);
        menu::set_style(o, fe, 1);
        for (i, item) in PC_ITEMS.into_iter().enumerate() {
            menu::add_entry(o, fe, string_crc(item), i as i32 != ROW_RESOLUTION || fullscreen, false, 0);
        }
        menu::show(o, fe, true, false);
        menu::set_selection(o, fe, sel);
    });
    show_values(s, fe);
    true
}

fn skip_disabled(fe: &mut Feck2, dir: i32) -> bool {
    ov(fe, |o, fe| {
        if o.menu_count > 0 && !menu::entry_enabled(o, o.menu_selected as usize) {
            if !menu::move_selection(o, fe, dir) || !menu::entry_enabled(o, o.menu_selected as usize) {
                // nothing enabled past it: back to where the press came from
                menu::move_selection(o, fe, -dir);
            }
        }
    });
    true
}

fn cycle(i: usize, n: usize, dir: i32) -> usize {
    ((i as i32 + dir).rem_euclid(n as i32)) as usize
}

/// left / right on the selected row.
fn change(s: &mut CFEPcOptionsScreen, fe: &mut Feck2, dir: i32) {
    let p = &mut fe.options.pc;
    match s.selected {
        ROW_MODE => {
            p.display_mode = cycle(p.display_mode.min(2) as usize, 3, dir) as u8;
            let full = p.display_mode == display_mode::FULLSCREEN;
            ov(fe, |o, fe| {
                menu::set_entry_enabled(o, fe, ROW_RESOLUTION as usize, full);
                menu::layout(o, fe);
            });
        }
        ROW_RESOLUTION => {
            if p.display_mode != display_mode::FULLSCREEN {
                return;
            }
            let cur = fe.pc_display.shown(p);
            match fe.pc_display.step(cur, dir) {
                Some(r) if Some(r) != p.resolution => p.resolution = Some(r),
                _ => return,
            }
        }
        ROW_FPS => {
            let i = FPS_CAPS.iter().position(|&f| f == p.fps_cap).unwrap_or(FPS_CAPS.len() - 1);
            p.fps_cap = FPS_CAPS[cycle(i, FPS_CAPS.len(), dir)];
        }
        ROW_VSYNC => p.vsync = !p.vsync,
        ROW_MUSIC => p.music_random = !p.music_random,
        _ => return,
    }
    fe.options_changed = true;
    show_values(s, fe);
}

fn utf16(t: &str) -> Vec<u16> {
    t.encode_utf16().collect()
}

/// every row's value text (and the resolution value's enabled colour).
fn show_values(s: &mut CFEPcOptionsScreen, fe: &mut Feck2) {
    let p = fe.options.pc.clone();
    fe.text(s.values[0]).set_string(string_crc(MODE_STRINGS[p.display_mode.min(2) as usize]));
    let (w, h) = fe.pc_display.shown(&p);
    let res = if w == 0 { utf16("-") } else { utf16(&format!("{w} x {h}")) };
    let full = p.display_mode == display_mode::FULLSCREEN;
    let t = fe.text(s.values[1]);
    t.set_text(&res);
    // FE2DOverlay.cpp 0x00280410's disabled entry colour
    t.set_colour(if full { COLOUR_WHITE } else { visos_colour(128, 68, 68, 68) });
    if p.fps_cap == 0 {
        fe.text(s.values[2]).set_string(string_crc("OPTION_PC_FPSCAP_UNCAPPED"));
    } else {
        fe.text(s.values[2]).set_text(&utf16(&format!("{} FPS", p.fps_cap)));
    }
    fe.text(s.values[3]).set_string(string_crc(if p.vsync { "GENERAL_ON" } else { "GENERAL_OFF" }));
    fe.text(s.values[4]).set_string(string_crc(if p.music_random { "OPTION_PC_MUSIC_RANDOM" } else { "OPTION_PC_MUSIC_ORIGINAL" }));
}
