//! src\Feck2\Screens\Common\FETipsScreen.cpp — `TipsScreen` (`gpTipsScreen` `dword_767140`, 0x1C bytes; the
//! create / events sit at the end of the idb's FEMainMenuScreen.cpp range, 0x0023F110..0x00240590): the options'
//! TUTORIALS list (tips 1..8, only the ones whose movie was seen are enabled) and the after-race tip (the hangout
//! sets the tip, the screen shows its text for 5 frames, plays the movie once and goes back).
//!
//! Original data bug kept: the text prefix table `dword_688C90` misses two commas ("SLIPSTREAM" "DRAG"
//! "MANUALGEARS" is one string), so tip 5 reads `GAMETIPS_SLIPSTREAMDRAGMANUALGEARS_INFO_VIDEO_n` (NOT FOUND), tip 6
//! (DRAG) TRAILBLAZER's, and tips 7 / 8 strcat a null pointer (a crash in the original; `GAMETIPS__INFO_VIDEO_n` here).

use std::sync::{Arc, Mutex};

use crate::feck2::feck2_manager::{Feck2, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay::{self as overlay, CFE2DOverlay, B_MENU_HEADER};
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::feck2::screens::common::fe_loading_screen::TUTORIAL_TITLES;
use crate::feck2::screens::garage::ov;
use crate::feck2::screens::hangout::CareerRequest;
use crate::string_manager::string_manager::string_crc;

/// `dword_688C90`: the tip text prefixes (see the module note)
const TIP_TEXT: [Option<&str>; 9] =
    [None, Some("INITIAL"), Some("ZONE"), Some("NOS"), Some("DRIFT"), Some("SLIPSTREAMDRAGMANUALGEARS"), Some("TRAILBLAZER"), None, None];
/// `dword_688CB4`: the tips' movies under `\Movies\PAL\TIPS\` (INITIAL has none; no tblazer.bik is shipped)
const TIP_MOVIES: [Option<&str>; 9] =
    [None, None, Some("zone"), Some("nos"), Some("drift"), Some("slip"), Some("drag"), Some("mgears"), Some("tblazer")];
/// `dword_6C135C` / `dword_6C1364`: the two text lines
const TEXT_X: [u32; 2] = [30, 30];
const TEXT_Y: [u32; 2] = [75, 368];

/// `CFETipsScreen` (0x1C bytes)
pub struct CFETipsScreen {
    pub screen: ScreenId,       // +0
    pub layer: LayerId,         // +4
    pub texts: [ElementId; 2],  // +8
    pub tip: i32,               // +16 (-1 = show the list)
    pub loaded: bool,           // +20
    /// +22: go back to the previous screen after the movie (else back to the list)
    pub go_back: bool,
    pub countdown: i32,         // +24
}

/// FEMainMenuScreen.cpp 0x0023F110 / 0x0023F210 / 0x0023F280 / 0x0023F550 — `FECK2CreateTipsScreen` (ctor, Init, AssignEvents)
pub fn create_tips_screen(fe: &mut Feck2) -> Arc<Mutex<CFETipsScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("TipsScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("TipsScreen_Layer"), ..Default::default() });
    let mut d = SElementDesc::default();
    let mut texts = [0; 2];
    for i in 0..2 {
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = 0;
        d.crc = string_crc("LoadingTipInfo");
        d.position[0] = TEXT_X[i] as f32;
        d.position[1] = TEXT_Y[i] as f32;
        d.string = 0;
        texts[i] = fe.create_element(layer, &d);
    }
    let obj = Arc::new(Mutex::new(CFETipsScreen { screen, layer, texts, tip: -1, loaded: false, go_back: false, countdown: -1 }));
    // 0x0023F550 — AssignEvents
    fe.map(layer, functor(&obj, on_enter), ev::ENTER);
    fe.map(layer, functor(&obj, on_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEAVE);
    fe.map(layer, functor(&obj, on_start), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, on_update), ev::UPDATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, on_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, on_cancel), ev::CANCEL_PRESS);
    fe.tips_screen = Some(obj.clone());
    obj
}

/// FEMainMenuScreen.cpp 0x0023F1F0 — `gpTipsScreen` +16 = tip (-1 = the list)
pub fn set_tip(fe: &mut Feck2, tip: i32) {
    if let Some(t) = fe.tips_screen.clone() {
        t.lock().unwrap().tip = tip;
    }
}

/// FETipsScreen.cpp 0x00240310 — the tip's movie was seen (`dword_6F71D8` bit tip - 1)
pub fn tip_watched(fe: &Feck2, tip: i32) -> bool {
    match tip {
        1..=8 => fe.career.tips_watched & (1 << (tip - 1)) != 0,
        0 => false,
        _ => {
            log::error!("assert: 0 && \"Unhandled TIPS TYPE\" (FETipsScreen.cpp 474)");
            false
        }
    }
}

/// FE3DCamera.cpp 0x002798D0 / FE2DOverlay.cpp 0x00282420 — (idb names) the overlay's 2D background: blend flag (+13) and visibility
pub(crate) fn background(o: &mut CFE2DOverlay, fe: &mut Feck2, visible: bool, blend: bool) {
    fe.el(o.background_2d).base_mut().blend = blend;
    fe.el(o.background_2d).set_visible(visible);
}

/// FEMainMenuScreen.cpp 0x0023F900 — Enter
fn on_enter(s: &mut CFETipsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| overlay::attach_to_screen(o, fe, Some(s.screen)));
    // FE3DCamera 0x0026FB60(screen, 0) then 0x00271E80
    fe.foyer.snap_to_target();
    if !s.loaded {
        s.loaded = true;
        for t in s.texts {
            fe.element_load(t);
        }
    }
    for t in s.texts {
        fe.el(t).set_visible(false);
    }
    true
}

/// FEMainMenuScreen.cpp 0x0023F9E0 — Activate: the options look (title OPTION_TUTORIALS, style 1, 5 rows) over LOADSCR's background
fn on_activate(_s: &mut CFETipsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| {
        let bg = fe.el(o.background_2d).is_visible();
        overlay::reset(o, fe);
        fe.el(o.background_2d).set_visible(bg);
        overlay::set_hide_options(o, fe, true);
        overlay::set_nav_text(o, fe, overlay::NAV_SELECT, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, overlay::NAV_CANCEL, string_crc("GENERAL_BACK"));
        overlay::set_title(o, fe, string_crc("GENERAL_OPTIONS"));
        overlay::show_borders(o, fe);
        menu::set_style(o, fe, 1);
        menu::set_window(o, 5);
        fe.text(o.menu_title).set_string(string_crc("OPTION_TUTORIALS"));
        overlay::set_2d_background(o, fe, "loadingBackground", r"\FE\LOADSCR.vcd", false, COLOUR_WHITE);
        overlay::enable(o);
    });
    true
}

/// FEMainMenuScreen.cpp 0x0023FB90 — Start: the list (tip -1), or the tip set by the hangout
fn on_start(s: &mut CFETipsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.countdown = -1;
    if s.tip == -1 {
        s.tip = 0;
        let watched: Vec<bool> = (1..9).map(|i| tip_watched(fe, i)).collect();
        let sel = s.tip;
        ov(fe, |o, fe| {
            for i in 1..9 {
                menu::add_entry(o, fe, string_crc(TUTORIAL_TITLES[i]), watched[i - 1], false, 0);
            }
            menu::show(o, fe, true, false);
            menu::set_selection(o, fe, sel);
        });
        s.go_back = false;
        ov(fe, |o, fe| background(o, fe, false, true));
    } else if s.tip > 0 && s.tip < 9 {
        if tip_watched(fe, s.tip) {
            s.go_back = true;
            s.countdown = 0;
        } else {
            show_text(s, fe, s.tip);
            s.go_back = true;
            s.countdown = 5;
        }
    }
    true
}

/// FEMainMenuScreen.cpp 0x0023FCD0 / FEHangoutScreen.cpp 0x0021C890 — Update: after the countdown play the movie (always from the list, once from the hangout), then
/// back to the list or the previous screen
fn on_update(s: &mut CFETipsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.countdown > 0 {
        s.countdown -= 1;
    } else if s.countdown == 0 {
        s.countdown = -1;
        if s.tip > 0 && s.tip < 9 {
            if !s.go_back || !tip_watched(fe, s.tip) {
                play_movie(s, fe);
            }
            ov(fe, |o, fe| background(o, fe, false, true));
            if s.go_back {
                // FEHangoutScreen 0x0021C890: the overlay's next Activate is a first one
                ov(fe, |o, _| o.first_activate = true);
                fe.goto_previous_screen(1, 0);
            } else {
                back_to_list(s, fe);
            }
        }
    }
    true
}

/// FEMainMenuScreen.cpp 0x0023FDE0 — Select: the highlighted tip's text, its movie after the countdown
fn on_select(s: &mut CFETipsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let (shown, sel) = fe.overlay.as_ref().map(|o| (o.menu_shown, o.menu_selected)).unwrap_or((false, 0));
    if shown {
        s.countdown = 5;
        s.tip = sel + 1;
        show_text(s, fe, s.tip);
    }
    false
}

/// FEMainMenuScreen.cpp 0x0023FE40 — Cancel
fn on_cancel(_s: &mut CFETipsScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, _| o.first_activate = true);
    fe.goto_previous_screen(1, 0);
    false
}

/// FEMainMenuScreen.cpp 0x0023FE80 — the tip's title and its two `<PREFIX>_INFO_VIDEO_n` lines, menu hidden
fn show_text(s: &mut CFETipsScreen, fe: &mut Feck2, tip: i32) {
    let tip = tip as usize;
    ov(fe, |o, fe| {
        background(o, fe, true, true);
        overlay::set_title(o, fe, string_crc(TUTORIAL_TITLES[tip]));
    });
    // strcpy "GAMETIPS_", strcat the prefix (a null one would crash the original: "" here), strcat "_INFO_VIDEO"
    let base = format!("GAMETIPS_{}_INFO_VIDEO", TIP_TEXT[tip].unwrap_or(""));
    for i in 0..2 {
        let id = string_crc(&format!("{}_{}", base, i + 1)); // VsStringAnsi::Hash
        fe.text(s.texts[i]).set_string(id);
        fe.text(s.texts[i]).set_box(f32::from_bits(0x43E6_0000), f32::from_bits(0x42C8_0000));
        fe.el(s.texts[i]).set_visible(true);
    }
    ov(fe, |o, fe| {
        menu::show(o, fe, false, true);
        // element vtable +36 = Fade(-128, 0, 0): the menu header goes at once
        fe.element_fade(o.borders[B_MENU_HEADER], -128, 0, 0);
    });
}

/// FEMainMenuScreen.cpp 0x00240020 — back to the list
fn back_to_list(s: &mut CFETipsScreen, fe: &mut Feck2) {
    ov(fe, |o, fe| {
        background(o, fe, false, true);
        overlay::set_title(o, fe, string_crc("GENERAL_OPTIONS"));
        menu::show(o, fe, true, false);
    });
    for t in s.texts {
        fe.el(t).set_visible(false);
    }
}

/// FETipsScreen.cpp 0x002400D0 — `GameManager__BE9F0("TIPS\<movie>", 1, 0)` and mark the tip seen
fn play_movie(s: &mut CFETipsScreen, fe: &mut Feck2) {
    let tip = s.tip;
    let Some(name) = usize::try_from(tip).ok().and_then(|t| TIP_MOVIES.get(t).copied().flatten()) else { return };
    fe.movies.push(format!("TIPS\\{name}"));
    match tip {
        0 => {}
        1..=8 => {
            fe.career.tips_watched |= 1 << (tip - 1);
            let flags = fe.career.tips_watched;
            fe.career.requests.push(CareerRequest::SetTipsWatched { flags });
        }
        _ => log::error!("assert: 0 && \"Unhandled TIPS TYPE\" (FETipsScreen.cpp 419)"),
    }
}
