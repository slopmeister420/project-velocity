//! `AudioOptionsScreen` (`gpAudioOptionsScreen`, 368 bytes): sound type, music, sfx. Type and sfx are disabled in
//! this build; music opens the soundtrack picker (left/right through the playlists, `RANDOM` = -1, saved in
//! `dword_6F9160`). The playlist accessors it calls (0x0004D9B0..0x0004DA10) are stubs returning 0, so the picker
//! only ever offers `RANDOM` and lists no songs. Volumes are set from the in-race audio menu.

use std::sync::{Arc, Mutex};

use super::{enter, ov, reset_keep_background};
use crate::feck2::feck2_manager::Feck2;
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::string_manager::string_manager::string_crc;

/// `dword_766400` (static init 0x0042D98A)
pub const AUDIO_ITEMS: [&str; 3] = ["OPTION_AUDIO_TYPE", "OPTION_AUDIO_MUSIC", "OPTION_AUDIO_SFX"];
/// song rows (`+80`, 68 of them)
pub const SONG_ROWS: usize = 68;

/// soundtrack playlist count / song count / song name (0x0004D9B0 / 0x0004D9D0 / 0x0004DA10): stubs in F3_D.
fn playlist_count() -> i32 {
    0
}
fn playlist_songs(_list: i32) -> usize {
    0
}

pub struct CFEAudioOptionsScreen {
    pub screen: ScreenId,          // +0
    pub layer: LayerId,            // +4
    pub heading: ElementId,        // +8
    pub value: ElementId,          // +12
    pub songs: Vec<ElementId>,     // +80
    pub loaded: bool,              // +352
    pub selected: i32,             // +356
    pub picking: i32,              // +360 1 while the soundtrack picker is open
    pub list: i32,                 // +364
}

/// FEOnlineSignInScreen.cpp 0x002372C0 / 0x00237400 / 0x002377E0 — `FECK2CreateAudioOptionsScreen` (Init, AssignEvents)
pub fn create_audio_options_screen(fe: &mut Feck2) -> Arc<Mutex<CFEAudioOptionsScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("AudioOptionsScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("AudioOptions_Layer"), ..Default::default() });
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 3, crc: string_crc("tracklistheading"), ..Default::default() };
    d.position = [f32::from_bits(1120403456), f32::from_bits(1125515264), 0.0, 1.0];
    d.string = string_crc("DJBOOTH_SOUNDTRACK_HEADING");
    let heading = fe.create_element(layer, &d);
    d.font = 1;
    d.crc = string_crc("tracklistvalue");
    d.position = [f32::from_bits(1132068864), f32::from_bits(1125515264), 0.0, 1.0];
    d.string = 0;
    let value = fe.create_element(layer, &d);
    d.font = 2;
    d.crc = string_crc("tracksongs");
    let songs = (0..SONG_ROWS as u32)
        .map(|i| {
            d.position = [f32::from_bits(1124204544), (20 * i + 200) as f32, 0.0, 1.0];
            d.string = 0;
            fe.create_element(layer, &d)
        })
        .collect();
    let obj = Arc::new(Mutex::new(CFEAudioOptionsScreen { screen, layer, heading, value, songs, loaded: false, selected: 0, picking: 0, list: -1 }));
    // 0x002377E0 — AssignEvents
    fe.map(layer, functor(&obj, on_enter), ev::ENTER);
    fe.map(layer, functor(&obj, on_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, on_leave), ev::LEAVE);
    fe.map(layer, functor(&obj, on_start), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UPDATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| {
        if s.picking != 0 {
            cycle(s, fe, -1);
        }
        false
    }), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| {
        if s.picking != 0 {
            cycle(s, fe, 1);
        }
        false
    }), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, on_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, on_cancel), ev::CANCEL_PRESS);
    obj
}

/// FEOnlineSignInScreen.cpp 0x00237C40 — Enter
fn on_enter(s: &mut CFEAudioOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    enter(fe, s.screen);
    s.loaded = true;
    fe.element_load(s.heading);
    fe.element_load(s.value);
    for &e in &s.songs {
        fe.element_load(e);
    }
    true
}

/// FEOnlineSignInScreen.cpp 0x00237D30 — Activate
fn on_activate(s: &mut CFEAudioOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let sel = s.selected;
    ov(fe, |o, fe| {
        reset_keep_background(o, fe);
        overlay::set_hide_options(o, fe, true);
        overlay::set_nav_text(o, fe, overlay::NAV_SELECT, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, overlay::NAV_CANCEL, string_crc("GENERAL_BACK"));
        overlay::set_title(o, fe, string_crc("OPTION_AUDIO_TITLE"));
        overlay::show_borders(o, fe);
        menu::set_style(o, fe, 1);
        for item in AUDIO_ITEMS {
            menu::add_entry(o, fe, string_crc(item), true, false, 0);
        }
        menu::set_entry_enabled(o, fe, 0, false);
        menu::set_entry_enabled(o, fe, 2, false);
        menu::show(o, fe, true, false);
        menu::set_selection(o, fe, sel);
        overlay::enable(o);
    });
    true
}

/// FEOnlineSignInScreen.cpp 0x00237EF0 — Leave: keep the picked playlist.
fn on_leave(s: &mut CFEAudioOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if fe.options.soundtrack != s.list {
        fe.options.soundtrack = s.list;
        fe.options_changed = true;
    }
    true
}

/// FEOnlineSignInScreen.cpp 0x00237F20 — Start
fn on_start(s: &mut CFEAudioOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, _| o.flag534 = true);
    s.picking = 0;
    s.list = fe.options.soundtrack;
    show_picker(s, fe, false);
    true
}

/// FEOnlineSignInScreen.cpp 0x00238080 — Select: music opens the picker.
fn on_select(s: &mut CFEAudioOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.selected = fe.overlay.as_ref().map(|o| o.menu_selected).unwrap_or(0);
    if s.selected == 1 && s.picking == 0 {
        s.picking += 1;
        show_picker(s, fe, true);
    }
    false
}

/// FEOnlineSignInScreen.cpp 0x002380F0 — Cancel: close the picker, else back.
fn on_cancel(s: &mut CFEAudioOptionsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.selected == 1 && s.picking == 1 {
        s.picking -= 1;
        show_picker(s, fe, false);
    } else if s.picking == 0 {
        fe.goto_previous_screen(1, 0);
    }
    false
}

/// FEOnlineSignInScreen.cpp 0x00238170 — step the playlist (-1 = random) and list its songs.
fn cycle(s: &mut CFEAudioOptionsScreen, fe: &mut Feck2, dir: i32) {
    if s.selected != 1 {
        return;
    }
    s.list += dir;
    if s.list >= -1 {
        if s.list >= playlist_count() {
            s.list = -1;
        }
    } else {
        s.list = playlist_count() - 1;
    }
    let songs = if s.list == -1 {
        fe.text(s.value).set_text(&"RANDOM".encode_utf16().collect::<Vec<_>>());
        0
    } else {
        playlist_songs(s.list)
    };
    for (i, &e) in s.songs.iter().enumerate() {
        fe.el(e).set_visible(i < songs);
    }
}

/// FEOnlineSignInScreen.cpp 0x00238330 — show / hide the picker.
fn show_picker(s: &mut CFEAudioOptionsScreen, fe: &mut Feck2, show: bool) {
    if show {
        cycle(s, fe, 0);
    } else {
        for &e in &s.songs {
            fe.el(e).set_visible(false);
        }
    }
    fe.el(s.heading).set_visible(show);
    fe.el(s.value).set_visible(show);
}
