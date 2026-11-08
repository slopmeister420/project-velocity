//! `LanguageOptionsScreen` (`gpLanguageOptionsScreen`, 16 bytes): the flag carousel. Select shows the language's
//! "loading" message, switches the string manager (`ChangeLanguage`, 0x002EBD70) and returns once it is done (to
//! the movie screen when it was the boot screen).

use std::sync::{Arc, Mutex};

use super::{enter, ov, reset_keep_background};
use crate::feck2::feck2_manager::{Feck2, COLOUR_BLACK};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::string_manager::string_manager::string_crc;

/// `dword_7609F0` language folders, `dword_760A0C` names, `dword_760A28` loading messages, `off_6C1330` flags
pub const LANGUAGES: [(&str, &str, &str, &str); 7] = [
    ("English", "LANGUAGE_ENGLISH", "LANGUAGE_ENGLISH_LOADING", "flag_English"),
    ("US", "LANGUAGE_US", "LANGUAGE_US_LOADING", "flag_US"),
    ("French", "LANGUAGE_FRENCH", "LANGUAGE_FRENCH_LOADING", "flag_French"),
    ("German", "LANGUAGE_GERMAN", "LANGUAGE_GERMAN_LOADING", "flag_German"),
    ("Spanish", "LANGUAGE_SPANISH", "LANGUAGE_SPANISH_LOADING", "flag_Spanish"),
    ("Italian", "LANGUAGE_ITALIAN", "LANGUAGE_ITALIAN_LOADING", "flag_Italian"),
    ("Japanese", "LANGUAGE_JAPANESE", "LANGUAGE_JAPANESE_LOADING", "flag_Japanese"),
];

pub struct CFELanguageOptionsScreen {
    pub screen: ScreenId,  // +0
    pub layer: LayerId,    // +4
    pub loaded: bool,      // +8
    pub changing: bool,    // +9
    pub from_boot: bool,   // +10 no previous screen: the language select at start-up
    pub selected: i32,     // +12
}

/// FEOnlineSignInScreen.cpp 0x00230AC0 / 0x00230C00 / 0x00230DD0 — `FECK2CreateLanguageOptionsScreen` (Init, AssignEvents)
pub fn create_language_options_screen(fe: &mut Feck2) -> Arc<Mutex<CFELanguageOptionsScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("LanguageOptionsScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("LanguageOptions_Layer"), ..Default::default() });
    let obj = Arc::new(Mutex::new(CFELanguageOptionsScreen { screen, layer, loaded: false, changing: false, from_boot: false, selected: 0 }));
    // 0x00230DD0 — AssignEvents
    fe.map(layer, functor(&obj, on_enter), ev::ENTER);
    fe.map(layer, functor(&obj, on_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEAVE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, on_update), ev::UPDATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, on_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| {
        if !s.from_boot {
            fe.goto_previous_screen(1, 0);
        }
        true
    }), ev::CANCEL_PRESS);
    obj
}

/// FEOnlineSignInScreen.cpp 0x00231230 — Enter
fn on_enter(s: &mut CFELanguageOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    enter(fe, s.screen);
    s.from_boot = fe.previous_screen_crc() == 0;
    s.loaded = true;
    s.changing = false;
    s.selected = 0;
    true
}

/// FEOnlineSignInScreen.cpp 0x002312D0 — Activate: the carousel (window 5, style 5, horizontal).
fn on_activate(s: &mut CFELanguageOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let (from_boot, sel) = (s.from_boot, s.selected);
    ov(fe, |o, fe| {
        reset_keep_background(o, fe);
        overlay::set_message(o, fe, string_crc("GENERAL_LOADING"));
        overlay::show_message(o, fe, false);
        overlay::show_arrows(o, fe, 0);
        if from_boot {
            overlay::set_2d_background(o, fe, "white", r"FE\FE.vcd", true, COLOUR_BLACK);
        } else {
            overlay::set_nav_text(o, fe, overlay::NAV_SELECT, string_crc("GENERAL_ACCEPT"));
            overlay::set_nav_text(o, fe, overlay::NAV_CANCEL, string_crc("GENERAL_CANCEL"));
            overlay::show_borders(o, fe);
        }
        overlay::enable(o);
        menu::set_window(o, 5);
        menu::set_style(o, fe, 5);
        menu::set_horizontal(o, fe, true);
        menu::clear(o, fe);
        for (_, name, _, flag) in LANGUAGES {
            menu::add_icon_entry(o, fe, string_crc(name), true, r"\FE\FEFlags.vcd", flag, None);
        }
        menu::show(o, fe, true, false);
        menu::set_selection(o, fe, sel);
    });
    true
}

/// FEOnlineSignInScreen.cpp 0x00231580 — Update: hide the message once the strings are in, then leave.
fn on_update(s: &mut CFELanguageOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    // CStringManager +124: files loading (never, the loads here are synchronous)
    ov(fe, |o, fe| overlay::show_message(o, fe, false));
    if s.changing {
        s.changing = false;
        if s.from_boot {
            fe.goto_screen_name("MovieScreen");
        } else {
            fe.goto_previous_screen(1, 0);
        }
    }
    true
}

/// FEOnlineSignInScreen.cpp 0x002316C0 — Select: show the loading message and change the language.
fn on_select(s: &mut CFELanguageOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.changing {
        return true;
    }
    s.selected = fe.overlay.as_ref().map(|o| o.menu_selected).unwrap_or(0);
    let (folder, _, loading, _) = LANGUAGES[s.selected as usize];
    let text = fe.strings.get_string(string_crc(loading));
    ov(fe, |o, fe| {
        overlay::set_message_text(o, fe, &text);
        overlay::set_message_timeout(o, -1);
        overlay::show_message(o, fe, true);
        overlay::show_nav(o, fe, 0, false);
        overlay::show_nav(o, fe, 1, false);
    });
    if fe.strings.change_language(string_crc(folder)).is_ok() && fe.options.language != folder {
        fe.options.language = folder.to_string();
        fe.options_changed = true;
    }
    ov(fe, |o, fe| {
        menu::show(o, fe, false, false);
        overlay::set_arrows_visible(o, fe, false, 0);
    });
    s.changing = true;
    true
}
