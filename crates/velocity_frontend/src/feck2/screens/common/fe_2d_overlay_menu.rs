//! src\Feck2\Screens\Common\FE2DOverlay.cpp — the overlay's menu (`sMenuEntry` list at `+392`): screens add
//! entries, the `COMMONMENUOVERLAY` layer moves the selection and plays the cursor sounds, and the screen reads
//! the selection (`+412`) in its own select handler. The functions sit in the idb's FE3DCamera / FE2DOverlay
//! ranges (0x0027D220..0x00280EC0).
//!
//! styles (`+428`): 0 bottom list, 1 (mapped to 2), 2 right list, 3 centred list, 5 icon carousel.

use crate::feck2::feck2_manager::{visos_colour, Feck2, COLOUR_GREY, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay::{position_arrows, CFE2DOverlay, ICON, B_MENU_CONTENT, B_MENU_HEADER, R_MENU_CONTENT, TITLE_HIGHLIGHT};
use crate::string_manager::string_manager::string_crc;
use velocity_audio::GuiSound;

const FE_VCD: &str = "\\FE\\FE.vcd";
/// `dword_6C1768` / `dword_6C1764`: carousel slide and fade time
const ICON_MOVE_TIME: i32 = 25;
const ICON_FADE_TIME: i32 = 25;

/// `sMenuEntry` (44 bytes)
#[derive(Clone, Debug, Default)]
pub struct MenuEntry {
    pub text: ElementId,          // +0
    pub bg: [ElementId; 3],       // +4
    pub info: Option<ElementId>,  // +16
    pub icons: [Option<ElementId>; 2], // +20
    pub string: u32,              // +28
    pub enabled: bool,            // +32
    pub data: u32,                // +36
}

/// style metrics (0x0027D860): entry x, info x, layout x, layout y, background x, row spacing.
#[derive(Clone, Copy, Debug)]
pub struct MenuMetrics {
    pub x: f32,
    pub info_x: f32,
    pub layout_x: f32,
    pub layout_y: f32,
    pub bg_x: f32,
    pub spacing: f32,
}

/// FE2DOverlay.cpp 0x0027D860
pub fn metrics(o: &CFE2DOverlay) -> MenuMetrics {
    let f = f32::from_bits;
    let m = |x: f32, lx: u32, ly: u32, bg: f32, sp: u32| MenuMetrics { x, info_x: x + 140.0, layout_x: f(lx), layout_y: f(ly), bg_x: bg, spacing: f(sp) };
    match o.menu_style {
        0 => m(245.0, 1131151360, 1135214592, 236.0, 1101004800),
        1 => m(57.0, 1131151360, 1124597760, 48.0, 1103626240),
        2 => m(309.0, 1133903872, 1125515264, 300.0, 1101004800),
        3 => m(256.0, 1132462080, 1128792064, (256.0f32 - 64.0) - 16.0, 1103626240),
        5 => m(256.0, 1132462080, 1117782016, 256.0, 0),
        _ => MenuMetrics { x: 0.0, info_x: 0.0, layout_x: 0.0, layout_y: 0.0, bg_x: 0.0, spacing: 20.0 },
    }
}

/// FE2DOverlay.cpp 0x0027DB00 — the visible window (first, last) around the selection.
pub fn window(o: &CFE2DOverlay) -> (i32, i32) {
    if o.menu_window == -1 {
        return (0, 0);
    }
    let half = o.menu_window / 2;
    if o.menu_selected >= half {
        if o.menu_selected <= o.menu_count - half - 1 {
            (o.menu_selected - half, half + o.menu_selected)
        } else {
            (o.menu_count - o.menu_window, o.menu_count)
        }
    } else {
        (0, o.menu_count.min(o.menu_window) - 1)
    }
}

/// FE2DOverlay.cpp 0x0027DC10 — `AddMenuEntry(string, enabled, hasInfo, infoString)`
pub fn add_entry(o: &mut CFE2DOverlay, fe: &mut Feck2, string: u32, enabled: bool, has_info: bool, info: u32) -> usize {
    let m = metrics(o);
    let row = (o.menu_window / 2 + o.menu_count) as f32;
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: o.menu_font, crc: string_crc("MenuEntry"), ..Default::default() };
    d.position = [m.x, row * m.spacing + m.layout_y, 0.0, 1.0];
    d.string = string;
    let mut e = MenuEntry { text: fe.create_element(o.menu_layer, &d), ..Default::default() };
    if has_info {
        d.crc = string_crc("MenuEntryInfo");
        d.position = [m.info_x, row * m.spacing + m.layout_y, 0.0, 1.0];
        d.string = info;
        e.info = Some(fe.create_element(o.menu_layer, &d));
    }
    d.kind = ELEMENT_2D;
    d.crc = string_crc("MenuEntryBG");
    d.vcd = FE_VCD.into();
    let y = m.layout_y - 6.0 + row * m.spacing;
    for (i, (tex, x)) in [("FE_menu_VerNavLeft", m.bg_x), ("FE_menu_VerNavMid", m.bg_x + 64.0), ("FE_menu_VerNavRight", (m.bg_x + 64.0) + 32.0)].into_iter().enumerate() {
        d.texture = tex.into();
        d.position = [x, y, 0.0, 1.0];
        e.bg[i] = fe.create_element(o.menu_layer, &d);
    }
    e.enabled = enabled;
    e.string = string;
    fe.element_load(e.text);
    if let Some(i) = e.info {
        fe.element_load(i);
    }
    for i in 0..3 {
        fe.element_load(e.bg[i]);
        if i == 1 {
            fe.e2d(e.bg[i]).set_size(32, 32);
        }
        fe.el(e.bg[i]).set_visible(false);
    }
    let c = if enabled { COLOUR_WHITE } else { visos_colour(128, 68, 68, 68) };
    for t in std::iter::once(e.text).chain(e.info) {
        fe.text(t).set_colour(c);
        fe.text(t).set_colour2(c);
    }
    match o.menu_style {
        0 => fe.text(e.text).set_justification(1),
        3 | 5 => fe.text(e.text).set_justification(2),
        _ => {}
    }
    o.menu_count += 1;
    o.menu.push(e);
    o.menu.len() - 1
}

/// FE2DOverlay.cpp 0x0027E330 — `AddMenuEntry(text, enabled)`: an entry showing raw text (profile names)
pub fn add_text_entry(o: &mut CFE2DOverlay, fe: &mut Feck2, text: &str, enabled: bool) -> usize {
    let i = add_entry(o, fe, 0, enabled, false, 0);
    let t: Vec<u16> = text.encode_utf16().collect();
    let id = o.menu[i].text;
    fe.text(id).set_text(&t);
    i
}

/// FE2DOverlay.cpp 0x0027E9D0 — `AddMenuEntry` plus an "EntryIcon" (dynamic 2D) at the info column, centred on the
/// entry's y, tinted top / bottom (gradient) and sized `w` x `h`; the layout keeps it on the entry's row
#[allow(clippy::too_many_arguments)]
pub fn add_colour_entry(o: &mut CFE2DOverlay, fe: &mut Feck2, string: u32, enabled: bool, vcd: &str, texture: &str, top: u32, bottom: u32, w: u32, h: u32) -> usize {
    let m = metrics(o);
    let i = add_entry(o, fe, string, enabled, false, 0);
    // 0x002824A0: the entry text's y
    let y = fe.el(o.menu[i].text).base().position[1];
    let mut d = SElementDesc { kind: ELEMENT_2D, dynamic: true, crc: string_crc("EntryIcon"), ..Default::default() };
    d.position = [m.info_x, y - (h / 2) as f32, 0.0, 1.0];
    d.vcd = vcd.into();
    d.texture = texture.into();
    let id = fe.create_element(o.menu_layer, &d);
    fe.el(id).set_visible(false);
    fe.element_load(id);
    o.menu[i].icons[0] = Some(id);
    let e = fe.e2d(id);
    e.set_colour(top, true, bottom);
    e.set_size(w, h);
    i
}

/// FE2DOverlay.cpp 0x0027E3E0 — `AddMenuIconEntry(string, enabled, vcd, texture, texture2)`: an entry plus its
/// icon(s) laid out 90 apart, scaled to fit 86x64.
pub fn add_icon_entry(o: &mut CFE2DOverlay, fe: &mut Feck2, string: u32, enabled: bool, vcd: &str, texture: &str, texture2: Option<&str>) -> usize {
    let i = add_entry(o, fe, string, enabled, false, 0);
    let x = (256 + ((o.menu_count - 1) as f64 * 90.0).round() as i32 - 43) as f32;
    let mut d = SElementDesc { kind: ELEMENT_2D, dynamic: true, crc: string_crc("MenuEntryIcon"), ..Default::default() };
    d.position = [x, f32::from_bits(1123024896), 0.0, 1.0];
    d.vcd = vcd.into();
    if let Some(t2) = texture2.filter(|t| !t.eq_ignore_ascii_case("NONE")) {
        d.texture = t2.into();
        let id = fe.create_element(o.menu_layer, &d);
        fe.el(id).set_visible(false);
        fe.element_load(id);
        o.menu[i].icons[1] = Some(id);
    }
    d.texture = texture.into();
    let id = fe.create_element(o.menu_layer, &d);
    fe.el(id).set_visible(false);
    fe.element_load(id);
    o.menu[i].icons[0] = Some(id);
    let (w, h) = {
        let e = fe.e2d(id);
        (e.width, e.height)
    };
    if w as f64 > 86.0 || h as f64 > 64.0 {
        let s: f32 = if w as f64 > 86.0 && h as f64 > 64.0 {
            if 86.0 - w as f64 <= 64.0 - h as f64 { (64.0 / h as f64) as f32 } else { (86.0 / w as f64) as f32 }
        } else if w as f64 <= 86.0 {
            (64.0 / h as f64) as f32
        } else {
            (86.0 / w as f64) as f32
        };
        let nw = (w as f64 * s as f64).round() as u32;
        let nh = (h as f64 * s as f64).round() as u32;
        fe.e2d(id).set_size(nw, nh);
    }
    i
}

/// FE2DOverlay.cpp 0x0027EC70 — `ClearMenu`
pub fn clear(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    o.menu_shown = false;
    for e in std::mem::take(&mut o.menu) {
        fe.delete_element(e.text);
        if let Some(i) = e.info {
            fe.delete_element(i);
        }
        for b in e.bg {
            fe.delete_element(b);
        }
        for i in e.icons.into_iter().flatten() {
            fe.delete_element(i);
        }
    }
    o.menu_count = 0;
    o.menu_selected = 0;
    fe.text(o.menu_title).set_string(string_crc("GENERAL_BLANK"));
    o.menu_slide = 0;
}

/// FE2DOverlay.cpp 0x0027EE60 — `ShowMenu(show, animate)`
pub fn show(o: &mut CFE2DOverlay, fe: &mut Feck2, visible: bool, animate: bool) {
    // the backgrounds' flag is set for styles 1/3 and then cleared again
    let bg_visible = false;
    for e in o.menu.clone() {
        fe.el(e.text).set_visible(visible);
        if let Some(i) = e.info {
            fe.el(i).set_visible(visible);
        }
        for i in e.icons.into_iter().flatten() {
            fe.el(i).set_visible(visible);
        }
        for b in e.bg {
            fe.el(b).set_visible(bg_visible);
        }
    }
    fe.el(o.menu_indicator).set_visible(visible);
    fe.el(o.menu_indicator).set_visible(false);
    fe.el(o.menu_highlighter).set_visible(visible);
    fe.el(o.menu_title).set_visible(visible);
    if visible {
        move_selection(o, fe, 0);
        o.menu_shown = true;
        let b = o.borders;
        let vis = |fe: &mut Feck2, s: [bool; 4]| {
            fe.el(b[B_MENU_HEADER]).set_visible(s[0]);
            fe.el(b[B_MENU_CONTENT]).set_visible(s[1]);
            fe.el(b[R_MENU_CONTENT]).set_visible(s[2]);
            fe.el(b[ICON]).set_visible(s[3]);
        };
        match o.menu_style {
            0 => {
                vis(fe, [true, true, false, true]);
                fe.element_set_position(b[B_MENU_HEADER], 256.0, f32::from_bits(1134657536), 0.0);
            }
            2 => {
                vis(fe, [true, false, true, true]);
                fe.element_set_position(b[B_MENU_HEADER], 256.0, f32::from_bits(1113325568), 0.0);
            }
            1 => vis(fe, [true, false, false, true]),
            3 => vis(fe, [false, false, false, true]),
            5 => vis(fe, [false, false, false, false]),
            _ => {}
        }
        fe.el(b[ICON]).set_visible(false);
        layout(o, fe);
    } else {
        o.menu_shown = false;
    }
    if animate {
        fade(o, fe, -128, 0);
        fade_title(o, fe, -128, 0);
        o.menu_slide = 0;
        slide_border(o, fe, -1);
    } else if visible {
        fade(o, fe, 128, 0);
        fade_title(o, fe, 128, 0);
        o.menu_slide = 5;
        slide_border(o, fe, 1);
    }
    place(o, fe);
}

/// FE2DOverlay.cpp 0x0027F390 — `MoveSelection(delta)`: clamps at the ends; false when it could not move.
pub fn move_selection(o: &mut CFE2DOverlay, fe: &mut Feck2, delta: i32) -> bool {
    let mut moved = true;
    o.menu_selected += delta;
    if o.menu_selected >= 0 {
        if o.menu_selected >= o.menu_count {
            o.menu_selected = o.menu_count - 1;
            moved = false;
        }
    } else {
        o.menu_selected = 0;
        moved = false;
    }
    if o.menu_window == -1 {
        let y = (o.menu_selected as f32 * 20.0) + 154.0;
        fe.element_set_position(o.menu_indicator, f32::from_bits(1133445120), y, 0.0);
        fe.el(o.menu_highlighter).set_visible(false);
        fe.element_set_position(o.menu_title, f32::from_bits(1136656384), f32::from_bits(1119879168), 0.0);
    } else {
        let half = o.menu_window / 2;
        let mut row = half;
        if o.menu_selected >= half {
            if o.menu_selected > o.menu_window - half - 1 {
                row = o.menu_window - (o.menu_count - o.menu_selected);
                if o.menu_count < o.menu_window {
                    row -= 1;
                }
            }
        } else {
            row = o.menu_selected;
        }
        let y = (row as f64 * 20.0 + 344.0) as f32;
        fe.element_set_position(o.menu_indicator, f32::from_bits(1131806720), y, 0.0);
        fe.el(o.menu_highlighter).set_visible(false);
        match o.menu_style {
            0 => fe.element_set_position(o.menu_title, f32::from_bits(1136525312), f32::from_bits(1136033792), 0.0),
            2 => fe.element_set_position(o.menu_title, f32::from_bits(1136656384), f32::from_bits(1119879168), 0.0),
            5 => fe.element_set_position(o.menu_title, f32::from_bits(1132462080), f32::from_bits(1114636288), 0.0),
            _ => {}
        }
        if o.menu_style == 5 && moved {
            for (i, e) in o.menu.clone().into_iter().enumerate() {
                for icon in e.icons.into_iter().flatten() {
                    if delta != 0 {
                        let dx = (((delta as f32) * 90.0) * -1.0) as i32;
                        fe.el(icon).base_mut().move_by_x(dx, ICON_MOVE_TIME, 0);
                    }
                    let pct: f32 = match o.menu_selected - i as i32 {
                        0 => 100.0,
                        -1 | 1 => 40.0,
                        -2 | 2 => 30.0,
                        _ => 0.0,
                    };
                    fe.element_fade(icon, pct as i32, ICON_FADE_TIME, 0);
                }
            }
        }
    }
    layout(o, fe);
    moved
}

/// FE2DOverlay.cpp 0x0027F840 — place the visible entries and colour them by selection / enabled state.
pub fn layout(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    if o.menu_style == 5 {
        layout_icons(o, fe);
        return;
    }
    let m = metrics(o);
    let (first, last) = window(o);
    let mut row = 0;
    for (i, e) in o.menu.clone().into_iter().enumerate() {
        let i = i as i32;
        if i >= first && (i <= last || o.menu_window == -1) {
            if o.menu_window != -1 {
                let y = (row as f32 * 20.0) + m.layout_y;
                fe.el(e.text).set_visible(true);
                fe.element_set_position(e.text, m.layout_x, y, 0.0);
                if let Some(t) = e.info {
                    fe.el(t).set_visible(true);
                    fe.element_set_position(t, m.layout_x + 140.0, y, 0.0);
                }
                if o.menu_style != 5 {
                    for icon in e.icons.into_iter().flatten() {
                        fe.el(icon).set_visible(true);
                        fe.element_set_position(icon, m.info_x, y, 0.0);
                    }
                }
                row += 1;
            }
            let c = if i == o.menu_selected {
                if e.enabled { visos_colour(128, 120, 77, 14) } else { visos_colour(128, 40, 24, 0) }
            } else if e.enabled {
                COLOUR_GREY
            } else {
                visos_colour(128, 32, 32, 32)
            };
            fe.text(e.text).set_colour(c);
            if let Some(t) = e.info {
                fe.text(t).set_colour(c);
            }
        } else {
            fe.el(e.text).set_visible(false);
            if let Some(t) = e.info {
                fe.el(t).set_visible(false);
            }
            for b in e.bg {
                fe.el(b).set_visible(false);
            }
            for icon in e.icons.into_iter().flatten() {
                fe.el(icon).set_visible(false);
            }
        }
    }
}

/// FE2DOverlay.cpp 0x0027FD80 — the carousel: every icon shown, only the selected entry's text.
fn layout_icons(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    if o.menu_window == -1 {
        log::error!("assert: Invalid menu count icon display!!");
        return;
    }
    let (first, last) = window(o);
    for (i, e) in o.menu.clone().into_iter().enumerate() {
        let i = i as i32;
        for icon in e.icons.into_iter().flatten() {
            fe.el(icon).set_visible(true);
        }
        if i >= first && i <= last && i == o.menu_selected {
            let c = if e.enabled { visos_colour(128, 120, 77, 14) } else { visos_colour(128, 40, 24, 0) };
            fe.el(e.text).set_visible(true);
            fe.text(e.text).set_colour(c);
            if let Some(t) = e.info {
                fe.text(t).set_colour(c);
                fe.el(t).set_visible(true);
            }
        } else {
            fe.el(e.text).set_visible(false);
            if let Some(t) = e.info {
                fe.el(t).set_visible(false);
            }
            if !(i >= first && i <= last) {
                for b in e.bg {
                    fe.el(b).set_visible(false);
                }
            }
        }
    }
}

/// FE2DOverlay.cpp 0x00280230 — `SetMenuSelection`
pub fn set_selection(o: &mut CFE2DOverlay, fe: &mut Feck2, index: i32) {
    let old = o.menu_selected;
    if index >= 0 && index < o.menu_count {
        o.menu_selected = index;
        move_selection(o, fe, 0);
    }
    if o.menu_style == 5 {
        let dx = ((old - o.menu_selected) as f32 * 90.0) as i32;
        for e in o.menu.clone() {
            for icon in e.icons.into_iter().flatten() {
                fe.el(icon).base_mut().move_by_x(dx, 0, 0);
            }
        }
    }
}

/// FE2DOverlay.cpp 0x00280390 — `SetMenuActive`
pub fn set_active(o: &mut CFE2DOverlay, fe: &mut Feck2, active: bool) {
    o.menu_shown = active;
    if active {
        layout(o, fe);
    }
}

/// FE2DOverlay.cpp 0x002803D0 — `SetMenuHorizontal`: left/right move the selection instead of up/down.
pub fn set_horizontal(o: &mut CFE2DOverlay, fe: &mut Feck2, horizontal: bool) {
    o.menu_horizontal = horizontal;
    layout(o, fe);
}

/// FE2DOverlay.cpp 0x00280410 — `SetMenuEntryEnabled`
pub fn set_entry_enabled(o: &mut CFE2DOverlay, fe: &mut Feck2, index: usize, enabled: bool) {
    if let Some(e) = o.menu.get_mut(index) {
        e.enabled = enabled;
        let c = if enabled { COLOUR_WHITE } else { visos_colour(128, 68, 68, 68) };
        let (t, i) = (e.text, e.info);
        fe.text(t).set_colour(c);
        if let Some(i) = i {
            fe.text(i).set_colour(c);
        }
    }
}

/// FE2DOverlay.cpp 0x00280550 — `SetMenuEntryInfo`
pub fn set_entry_info(o: &mut CFE2DOverlay, fe: &mut Feck2, index: usize, string: u32) {
    if let Some(i) = o.menu.get(index).and_then(|e| e.info) {
        fe.text(i).set_string(string);
    }
}

/// FE2DOverlay.cpp 0x00280720 — `IsMenuEntryEnabled`
pub fn entry_enabled(o: &CFE2DOverlay, index: usize) -> bool {
    o.menu.get(index).map(|e| e.enabled).unwrap_or(false)
}

/// FE2DOverlay.cpp 0x002807F0 — `GetMenuEntryString`
pub fn entry_string(o: &CFE2DOverlay, index: usize) -> u32 {
    o.menu.get(index).map(|e| e.string).unwrap_or(0)
}

/// FE2DOverlay.cpp 0x002808C0 — `SetMenuWindow`: rows shown at once, -1 = all at their created place.
pub fn set_window(o: &mut CFE2DOverlay, rows: i32) {
    o.menu_window = rows;
}

/// FE2DOverlay.cpp 0x002808E0 — fade the entries (backgrounds timed), indicator, highlighter and icon border.
pub fn fade(o: &mut CFE2DOverlay, fe: &mut Feck2, pct: i32, time: i32) {
    for e in o.menu.clone() {
        fe.element_fade(e.text, pct, 0, 0);
        if let Some(i) = e.info {
            fe.element_fade(i, pct, 0, 0);
        }
        for b in e.bg {
            fe.element_fade(b, pct, time, 0);
        }
    }
    fe.element_fade(o.menu_indicator, pct, time, 0);
    fe.element_fade(o.menu_highlighter, pct, time, 0);
    fe.element_fade(o.borders[ICON], pct, time, 0);
}

/// FE2DOverlay.cpp 0x00280AF0 — fade the menu title and the title highlight.
pub fn fade_title(o: &mut CFE2DOverlay, fe: &mut Feck2, pct: i32, time: i32) {
    fe.element_fade(o.menu_title, pct, time, 0);
    fe.element_fade(o.borders[TITLE_HIGHLIGHT], pct, time, 0);
}

/// FE2DOverlay.cpp 0x00280B60 — step the menu border open (+1) or closed (-1); 5 = fully open.
pub fn slide_border(o: &mut CFE2DOverlay, fe: &mut Feck2, dir: i32) {
    o.menu_slide += if dir <= 0 { -1 } else { 1 };
    let b = o.borders;
    if o.menu_slide >= 0 {
        if o.menu_slide > 5 {
            o.menu_slide = 5;
        }
        match o.menu_style {
            2 => {
                fe.el(b[R_MENU_CONTENT]).set_visible(true);
                fe.el(b[B_MENU_HEADER]).set_visible(true);
                fe.el(b[ICON]).set_visible(true);
                let name = if o.menu_slide == 5 { "FE_Border_R_Menu_Content".to_string() } else { format!("FE_Border_RM{}", o.menu_slide + 1) };
                fe.change_texture(b[R_MENU_CONTENT], &name, false);
                fe.e2d(b[R_MENU_CONTENT]).set_size(256, 256);
            }
            0 => {
                fe.el(b[B_MENU_CONTENT]).set_visible(true);
                fe.el(b[B_MENU_HEADER]).set_visible(true);
                fe.el(b[ICON]).set_visible(true);
                let name = if o.menu_slide == 5 { "FE_Border_B_Menu_Content".to_string() } else { format!("FE_Border_RM{}", o.menu_slide + 1) };
                fe.change_texture(b[B_MENU_CONTENT], &name, false);
                let w = if o.menu_slide == 5 { 512 } else { 256 };
                fe.e2d(b[B_MENU_CONTENT]).set_size(w, 90);
            }
            1 => {
                fe.el(b[R_MENU_CONTENT]).set_visible(false);
                fe.el(b[B_MENU_CONTENT]).set_visible(false);
                fe.el(b[ICON]).set_visible(false);
            }
            _ => {}
        }
        fe.el(b[ICON]).set_visible(false);
    } else {
        o.menu_slide = -1;
        fe.el(b[R_MENU_CONTENT]).set_visible(false);
        fe.el(b[B_MENU_CONTENT]).set_visible(false);
        fe.el(b[ICON]).set_visible(false);
    }
}

/// FE2DOverlay.cpp 0x00280E60 — `SetMenuStyle` (1 is shown as 2)
pub fn set_style(o: &mut CFE2DOverlay, fe: &mut Feck2, style: i32) {
    o.menu_style = if style == 1 { 2 } else { style };
    place(o, fe);
}

/// FE2DOverlay.cpp 0x00280EC0 — place the icon border, the menu title and the arrows for the style.
pub fn place(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    let f = f32::from_bits;
    let icon = o.borders[ICON];
    match o.menu_style {
        0 => {
            fe.element_set_position(icon, f(3271557120), f(1135214592), 0.0);
            fe.e2d(icon).set_size(70, 70);
            fe.e2d(icon).set_flags(1);
            fe.element_set_position(o.menu_title, f(1136525312), f(1136033792), 0.0);
            position_arrows(o, fe, f(1133903872), f(1139802112), f(1136033792), 0);
            position_arrows(o, fe, f(1131544576), f(1135050752), f(1137180672), 1);
        }
        2 => {
            fe.element_set_position(icon, f(1134559232), f(1132593152), 0.0);
            fe.e2d(icon).set_size(70, 70);
            fe.e2d(icon).set_flags(1);
            fe.element_set_position(o.menu_title, f(1136656384), f(1119879168), 0.0);
            position_arrows(o, fe, f(1133379584), f(1139408896), f(1119223808), 0);
            position_arrows(o, fe, f(1139146752), f(1124073472), f(1131282432), 1);
        }
        1 => {
            fe.element_set_position(icon, f(1134559232), f(1132593152), 0.0);
            fe.e2d(icon).set_size(70, 70);
            fe.e2d(icon).set_flags(1);
            let x = (80.0f32 + 48.0) - 8.0;
            let y = ((25 * o.menu_count) as f32 + 136.0) + 5.0;
            position_arrows(o, fe, x, f(1120796672), y, 1);
            position_arrows(o, fe, f(1120403456), f(1136525312), f(1114374144), 0);
        }
        5 => position_arrows(o, fe, f(1120403456), f(1136525312), f(1117782016), 0),
        _ => {}
    }
}

// ------------------------------------------------------------------------------------------------------------
// layer functors

/// FE2DOverlay.cpp 0x00276C70 — the `COMMONMENUOVERLAY` layer functors (it sees events before the screen).
pub fn assign_menu_events(fe: &mut Feck2) {
    let Some(layer) = fe.overlay.as_ref().map(|o| o.menu_layer) else { return };
    let h = |f: fn(&mut CFE2DOverlay, &mut Feck2) -> bool| -> Handler {
        std::sync::Arc::new(move |fe: &mut Feck2, _e: &CEvent| with(fe, f))
    };
    fe.map(layer, h(menu_up), ev::UP_PRESS);
    fe.map(layer, h(menu_down), ev::DOWN_PRESS);
    fe.map(layer, h(menu_left), ev::LEFT_PRESS);
    fe.map(layer, h(menu_right), ev::RIGHT_PRESS);
    fe.map(layer, h(menu_select), ev::SELECT_PRESS);
    fe.map(layer, h(|_, _| true), ev::CANCEL_PRESS);
    fe.map(layer, h(menu_online), ev::ONLINE_PRESS);
}

fn with(fe: &mut Feck2, f: fn(&mut CFE2DOverlay, &mut Feck2) -> bool) -> bool {
    let Some(mut o) = fe.overlay.take() else { return true };
    let r = f(&mut o, fe);
    fe.overlay = Some(o);
    r
}

/// FE2DOverlay.cpp 0x00277DA0
fn menu_up(o: &mut CFE2DOverlay, fe: &mut Feck2) -> bool {
    if !o.dialog_active && o.menu_count > 0 && o.menu_shown && !o.menu_horizontal && move_selection(o, fe, -1) {
        fe.sounds.push(GuiSound::Up);
    }
    true
}

/// FE2DOverlay.cpp 0x00277EE0 (plays the up sound as well)
fn menu_down(o: &mut CFE2DOverlay, fe: &mut Feck2) -> bool {
    if !o.dialog_active && o.menu_count > 0 && o.menu_shown && !o.menu_horizontal && move_selection(o, fe, 1) {
        fe.sounds.push(GuiSound::Up);
    }
    true
}

/// FE2DOverlay.cpp 0x00278030
fn menu_left(o: &mut CFE2DOverlay, fe: &mut Feck2) -> bool {
    if o.menu_count > 0 && o.menu_shown && o.menu_horizontal && move_selection(o, fe, -1) {
        fe.sounds.push(GuiSound::Left);
    }
    true
}

/// FE2DOverlay.cpp 0x00278170
fn menu_right(o: &mut CFE2DOverlay, fe: &mut Feck2) -> bool {
    if o.menu_count > 0 && o.menu_shown && o.menu_horizontal && move_selection(o, fe, 1) {
        fe.sounds.push(GuiSound::Right);
    }
    true
}

/// FE2DOverlay.cpp 0x00278280 — a disabled entry swallows the select with the error sound.
fn menu_select(o: &mut CFE2DOverlay, fe: &mut Feck2) -> bool {
    if o.menu_shown && !o.dialog_active && o.menu_count > 0 {
        if let Some(e) = o.menu.get(o.menu_selected as usize) {
            if e.enabled {
                fe.sounds.push(GuiSound::Accept);
                return true;
            }
            fe.sounds.push(GuiSound::Error);
            return false;
        }
    }
    true
}

/// FE2DOverlay.cpp 0x002784A0 — the online button: sign-in screen (XOnline is not ported, so it stays here).
fn menu_online(o: &mut CFE2DOverlay, _fe: &mut Feck2) -> bool {
    if !o.hide_options {
        log::debug!("online button: OnlineSignInScreen not ported");
    }
    true
}

/// FE2DOverlay.cpp 0x00277D40 / 0x00277E80 / 0x00277FC0 / 0x00278100 / 0x00278390 / 0x00278400 — the
/// `COMMON2DOVERLAY` layer's cursor sounds and the options button.
pub fn assign_overlay_input_events(fe: &mut Feck2) {
    let Some(layer) = fe.overlay.as_ref().map(|o| o.layer) else { return };
    let h = |f: fn(&mut CFE2DOverlay, &mut Feck2) -> bool| -> Handler {
        std::sync::Arc::new(move |fe: &mut Feck2, _e: &CEvent| with(fe, f))
    };
    // 0x00277D40 / 0x00277E80: with the dialog up the d-pad moves its selection and goes no further
    fe.map(layer, h(|o, fe| {
        if !o.no_up_down_sound {
            fe.sounds.push(GuiSound::Up);
        }
        if o.dialog_active {
            crate::feck2::screens::common::fe_2d_overlay::move_dialog_selection(o, fe, -1);
        }
        !o.dialog_active
    }), ev::UP_PRESS);
    fe.map(layer, h(|o, _| !o.dialog_active), ev::UP_HELD);
    fe.map(layer, h(|o, _| !o.dialog_active), ev::UP_RELEASE);
    fe.map(layer, h(|o, fe| {
        if !o.no_up_down_sound {
            fe.sounds.push(GuiSound::Down);
        }
        if o.dialog_active {
            crate::feck2::screens::common::fe_2d_overlay::move_dialog_selection(o, fe, 1);
        }
        !o.dialog_active
    }), ev::DOWN_PRESS);
    fe.map(layer, h(|o, _| !o.dialog_active), ev::DOWN_HELD);
    fe.map(layer, h(|o, _| !o.dialog_active), ev::DOWN_RELEASE);
    fe.map(layer, h(|o, fe| {
        if o.dialog_active {
            return false;
        }
        if !o.flag534 {
            fe.sounds.push(GuiSound::Left);
        }
        true
    }), ev::LEFT_PRESS);
    fe.map(layer, h(|o, _| !o.dialog_active), ev::LEFT_HELD);
    fe.map(layer, h(|o, _| !o.dialog_active), ev::LEFT_RELEASE);
    fe.map(layer, h(|o, fe| {
        if o.dialog_active {
            return false;
        }
        if !o.flag534 {
            fe.sounds.push(GuiSound::Right);
        }
        true
    }), ev::RIGHT_PRESS);
    fe.map(layer, h(|o, _| !o.dialog_active), ev::RIGHT_HELD);
    fe.map(layer, h(|o, _| !o.dialog_active), ev::RIGHT_RELEASE);
    fe.map(layer, h(|_, fe| {
        fe.sounds.push(GuiSound::Accept);
        true
    }), ev::SELECT_PRESS);
    fe.map(layer, h(|o, fe| {
        if !o.no_back_sound {
            fe.sounds.push(GuiSound::Back);
        }
        true
    }), ev::CANCEL_PRESS);
    fe.map(layer, h(|o, fe| {
        if !o.hide_options {
            fe.goto_screen_name("OptionsScreen");
        }
        true
    }), ev::PAUSE_PRESS);
}
