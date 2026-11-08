//! src\Feck2\Screens\Common\FE2DOverlay.cpp — `CFE2DOverlay` (`gpCommon2DOverlay`, 0x007696B0): the borders,
//! title, nav button prompts, message box, dialog, transition and 2D background shared by every screen.
//!
//! three layers drawn around the active screen's layers: `COMMON2DBACKGROUND` under them, `COMMON2DOVERLAY` and
//! `COMMONMENUOVERLAY` over them; the menu layer also sees events first. The menu list and both layers' input
//! functors are in `fe_2d_overlay_menu`.

use crate::feck2::feck2_manager::{Feck2, COLOUR_BLACK, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::string_manager::string_manager::string_crc;

const FE_VCD: &str = "\\FE\\FE.vcd";

/// `dword_6C1638`: nav prompt positions (sprite; text at +32, +10).
pub const NAV_POS: [[f32; 2]; 15] = [
    [366.1, 445.0], [256.0, 445.0], [37.5, 445.0], [148.0, 445.0], [280.0, 445.0], [38.5, 445.0], [38.5, 417.0],
    [35.0, 445.0], [35.0, 445.0], [35.0, 445.0], [35.0, 445.0], [35.0, 445.0], [35.0, 445.0], [35.0, 445.0], [35.0, 445.0],
];
/// `off_6C16B0`: nav sprite texture, sprite name, text name.
pub const NAV_NAMES: [[&str; 3]; 15] = [
    ["select_button", "SelectSprite", "SelectText"],
    ["cancel_button", "CancelSprite", "CancelText"],
    ["option1_button", "Option1Sprite", "Option1Text"],
    ["option2_button", "Option2Sprite", "Option2Text"],
    ["dpad", "DPadSprite", "DPadText"],
    ["GE_FE_buttonStart", "OptionsSprite", "OptionsText"],
    ["GE_FE_buttonSelect", "OnlineSprite", "OnlineText"],
    ["analog", "LeftAnalogSprite", "LeftAnalogText"],
    ["analog", "RightAnalogSprite", "RightAnalogText"],
    ["L1Button", "L1Sprite", "L1Text"],
    ["L2Button", "L2Sprite", "L2Text"],
    ["L3Button", "L3Sprite", "L3Text"],
    ["R1Button", "R1Sprite", "R1Text"],
    ["R2Button", "R2Sprite", "R2Text"],
    ["R3Button", "R3Sprite", "R3Text"],
];

pub const NAV_SELECT: usize = 0;
pub const NAV_CANCEL: usize = 1;

/// `CFE2DOverlay` (656 bytes). field comments give the original offsets.
#[derive(Default)]
pub struct CFE2DOverlay {
    pub enabled: bool,            // +624 draw the overlay layers
    pub pre_enabled: bool,        // +625 overlay layers see events
    pub layer: LayerId,           // +364 COMMON2DOVERLAY
    pub bg_layer: LayerId,        // +368 COMMON2DBACKGROUND
    pub menu_layer: LayerId,      // +372 COMMONMENUOVERLAY
    pub screen: Option<ScreenId>, // +360
    pub screen_crc: u32,          // +620
    pub background_2d: ElementId, // +8
    pub transition_bg: ElementId, // +12
    pub transition_text: ElementId, // +16
    pub transition_img: [ElementId; 2], // +20
    pub borders: [ElementId; 9],  // +28 .. +60
    pub nav_sprite: [ElementId; 15], // +64
    pub nav_text: [ElementId; 15],   // +124
    pub message_text: ElementId,  // +184
    pub message_bg: ElementId,    // +188
    pub message_y: f32,           // +196
    pub dialog_item: ElementId,   // +200
    pub dialog_option: [ElementId; 8], // +204
    pub dialog_bg: ElementId,     // +236
    pub dialog_indicator: ElementId, // +240
    pub background: ElementId,    // +256
    pub arrows: [ElementId; 4],   // +284
    pub game_logo: ElementId,     // +308
    pub static_logo: ElementId,   // +332
    pub menu_indicator: ElementId, // +336
    pub menu_highlighter: ElementId, // +340
    pub menu_title: ElementId,    // +344
    pub online_message: ElementId, // +348
    pub countdown_text: ElementId, // +376
    pub loading_text: ElementId,  // +436
    pub cash_icon: ElementId,     // +440
    pub prestige_icon: ElementId, // +444
    pub total_cash: ElementId,    // +448
    pub prestige_points: ElementId, // +452
    pub title_text: ElementId,    // +540
    pub heading_text: ElementId,  // +544
    pub heading_bar: ElementId,   // +548
    pub nav_shown: [bool; 15],    // +636
    pub hide_options: bool,       // +533
    pub flag534: bool,            // +534 no left/right cursor sound
    pub assets_loaded: bool,      // +538
    pub first_activate: bool,     // +4
    pub slid_in: bool,            // +532
    pub transition_state: i32,    // +528
    pub transition_timer: i32,    // +652
    pub message_time: f32,        // +192
    pub arrow_timer: i32,         // +300
    pub arrow_dir: i32,           // +304
    pub dialog_active: bool,      // +252
    pub dialog_selection: i32,    // +244
    pub dialog_count: u32,        // +248
    pub no_up_down_sound: bool,   // +535
    pub no_back_sound: bool,      // +536
    pub menu: Vec<crate::feck2::screens::common::fe_2d_overlay_menu::MenuEntry>, // +392
    pub menu_count: i32,          // +408
    pub menu_selected: i32,       // +412
    pub menu_slide: i32,          // +416
    pub menu_shown: bool,         // +420
    pub menu_horizontal: bool,    // +421
    pub menu_window: i32,         // +424
    pub menu_style: i32,          // +428
    pub menu_font: u32,           // +432
}

// border slots (+28 + 4*i)
pub const B_MENU_HEADER: usize = 0;
pub const B_MENU_CONTENT: usize = 1;
pub const R_MENU_CONTENT: usize = 2;
pub const INFO: usize = 3;
pub const TOP: usize = 4;
pub const NAV: usize = 5;
pub const OPTIONS_BG: usize = 6;
pub const ICON: usize = 7;
pub const TITLE_HIGHLIGHT: usize = 8;

fn desc(kind: u32, name: &str, x: f32, y: f32) -> SElementDesc {
    SElementDesc { kind, crc: string_crc(name), position: [x, y, 0.0, 1.0], ..Default::default() }
}

/// FE2DOverlay.cpp 0x00282280 / 0x00276560 — `FECK2CreateCommonOverlay`: ctor, then `Create` 0x00281B50.
pub fn create_common_overlay(fe: &mut Feck2) {
    let mut o = CFE2DOverlay::default();
    // 0x00276560: three layers; the first two carry functors in the original (menu system, not ported)
    o.layer = fe.create_layer(None, &SLayerDesc { priority: 1, crc: string_crc("COMMON2DOVERLAY"), ..Default::default() });
    o.menu_layer = fe.create_layer(None, &SLayerDesc { priority: 1, crc: string_crc("COMMONMENUOVERLAY"), ..Default::default() });
    o.bg_layer = fe.create_layer(None, &SLayerDesc { priority: 6, crc: string_crc("COMMON2DBACKGROUND"), ..Default::default() });
    o.message_time = -1.0;
    o.menu_window = -1;
    o.menu_style = 2;
    o.menu_font = 1;
    create(&mut o, fe);
    fe.overlay = Some(o);
    assign_overlay_events(fe);
    crate::feck2::screens::common::fe_2d_overlay_menu::assign_menu_events(fe);
    crate::feck2::screens::common::fe_2d_overlay_menu::assign_overlay_input_events(fe);
}

/// FE2DOverlay.cpp 0x00281B50 / 0x002792A0 / 0x00279460 / 0x0027B010 / 0x0027B520 / 0x0027BCE0 / 0x0027C040 / 0x0027CE50 / 0x0027D110 / 0x0027C7E0 / 0x0027C970 / 0x0027CB60 / 0x0027D660 / 0x00281250 / 0x00281800 / 0x00279980 / 0x002796C0 — `Create` and its per-part creates
fn create(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    // 0x002792A0 title
    let mut d = desc(ELEMENT_STATIC_TEXT, "TitleText", 32.0, 40.0);
    d.font = 4;
    d.string = 0;
    o.title_text = fe.create_element(o.layer, &d);
    // 0x00279460 heading
    let mut d = desc(ELEMENT_STATIC_TEXT, "HeadingText", f32::from_bits(1138556928), f32::from_bits(1114636288));
    d.font = 3;
    d.string = 0;
    o.heading_text = fe.create_element(o.layer, &d);
    d.kind = ELEMENT_2D;
    d.crc = string_crc("HeadingBar");
    d.position = [f32::from_bits(1136918528), f32::from_bits(1103626240), 0.0, 1.0];
    d.texture = "FE_Border_R_Menu_Header".into();
    d.vcd = FE_VCD.into();
    o.heading_bar = fe.create_element(o.layer, &d);
    // 0x0027B010 message
    let mut d = desc(ELEMENT_STATIC_TEXT, "MessageText", 256.0, f32::from_bits(1132068864));
    d.font = 1;
    d.string = 0;
    o.message_text = fe.create_element(o.menu_layer, &d);
    d.kind = ELEMENT_2D;
    d.crc = string_crc("Message_BACKGROUND");
    d.position = [f32::from_bits(1106771968), f32::from_bits(1130758144), 0.0, 1.0];
    d.texture = "menubackground".into();
    d.vcd = FE_VCD.into();
    o.message_bg = fe.create_element(o.menu_layer, &d);
    o.message_y = f32::from_bits(1114636288);
    // 0x0027B520 dialog
    let mut d = desc(ELEMENT_STATIC_TEXT, "DIALOG_ITEM", f32::from_bits(1119092736), f32::from_bits(1121714176));
    d.font = 1;
    d.string = 0;
    o.dialog_item = fe.create_element(o.menu_layer, &d);
    for i in 0..8u32 {
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = 1;
        d.crc = string_crc("DialogOption1");
        d.position = [f32::from_bits(1127481344), (25 * i + 190) as f32, 0.0, 1.0];
        d.string = 0;
        o.dialog_option[i as usize] = fe.create_element(o.menu_layer, &d);
    }
    d.kind = ELEMENT_2D;
    d.crc = string_crc("Dialog_Indicator");
    d.position = [f32::from_bits(1120403456), f32::from_bits(1130430464), 0.0, 1.0];
    d.texture = "white".into();
    d.vcd = FE_VCD.into();
    o.dialog_indicator = fe.create_element(o.menu_layer, &d);
    d.crc = string_crc("DialogBackground");
    d.position = [f32::from_bits(1084227584), f32::from_bits(1117519872), 0.0, 1.0];
    d.texture = "dialogbacking".into();
    o.dialog_bg = fe.create_element(o.menu_layer, &d);
    // 0x0027BCE0 background
    let mut d = desc(ELEMENT_2D, "Background", 0.0, 0.0);
    d.texture = "menuBackground".into();
    d.vcd = FE_VCD.into();
    o.background = fe.create_element(o.bg_layer, &d);
    // 0x0027C040 arrows (desc +80 dynamic set first)
    let mut d = SElementDesc { dynamic: true, ..Default::default() };
    for (i, (name, x, y, tex)) in [
        ("ArrowLeftSprite", 1120403456u32, 1120403456u32, "FE_Arrow_16"),
        ("ArrowRightSprite", 1128792064, 1120403456, "FE_Arrow_16"),
        ("ArrowUpSprite", 1106247680, 1116471296, "FE_Arrow_Vert"),
        ("ArrowDownSprite", 1106247680, 1120403456, "FE_Arrow_Vert"),
    ]
    .into_iter()
    .enumerate()
    {
        d.kind = ELEMENT_2D;
        d.crc = string_crc(name);
        d.position = [f32::from_bits(x), f32::from_bits(y), 0.0, 1.0];
        d.texture = tex.into();
        d.vcd = FE_VCD.into();
        o.arrows[i] = fe.create_element(o.layer, &d);
    }
    // 0x0027CE50 / 0x0027D110 logos
    let mut d = desc(ELEMENT_2D, "GameLogo", 0.0, f32::from_bits(1119748096));
    d.texture = "FE_mm_velocityTitle".into();
    d.vcd = FE_VCD.into();
    o.game_logo = fe.create_element(o.layer, &d);
    let mut d = desc(ELEMENT_2D, "StaticLogo", f32::from_bits(1135607808), 42.0);
    d.texture = "FE_gameLogo".into();
    d.vcd = FE_VCD.into();
    o.static_logo = fe.create_element(o.layer, &d);
    // 0x0027C7E0 loading
    let mut d = desc(ELEMENT_STATIC_TEXT, "LoadingText", 256.0, f32::from_bits(1138819072));
    d.font = 3;
    d.string = string_crc("GENERAL_LOADING");
    o.loading_text = fe.create_element(o.layer, &d);
    // 0x0027C970 / 0x0027CB60 cash and prestige
    let mut d = desc(ELEMENT_STATIC_TEXT, "TotalCash", f32::from_bits(1134559232), f32::from_bits(1136689152));
    d.font = 1;
    d.string = string_crc("PLAYER_CASH_TOTAL");
    o.total_cash = fe.create_element(o.layer, &d);
    d.kind = ELEMENT_2D;
    d.crc = string_crc("CashIcon");
    d.texture = "FE_menu_IconCash".into();
    d.vcd = FE_VCD.into();
    o.cash_icon = fe.create_element(o.layer, &d);
    let mut d = desc(ELEMENT_STATIC_TEXT, "PrestigePoints", f32::from_bits(1134559232), f32::from_bits(1137278976));
    d.font = 1;
    d.string = string_crc("PLAYER_PRESTIGE_TOTAL");
    o.prestige_points = fe.create_element(o.layer, &d);
    d.kind = ELEMENT_2D;
    d.crc = string_crc("PrestigeIcon");
    d.position = [f32::from_bits(1134559232), f32::from_bits(1136689152), 0.0, 1.0];
    d.texture = "FE_menu_IconPrestige".into();
    d.vcd = FE_VCD.into();
    o.prestige_icon = fe.create_element(o.layer, &d);
    // 0x0027D660 menu indicator / highlighter / title
    let mut d = desc(ELEMENT_2D, "MenuIndicatorSprite", f32::from_bits(1109393408), f32::from_bits(1125515264));
    d.texture = "white".into();
    d.vcd = FE_VCD.into();
    o.menu_indicator = fe.create_element(o.menu_layer, &d);
    d.crc = string_crc("MenuHighlighterSprite");
    d.position = [f32::from_bits(1108869120), f32::from_bits(1125187584), 0.0, 1.0];
    o.menu_highlighter = fe.create_element(o.menu_layer, &d);
    d.kind = ELEMENT_STATIC_TEXT;
    d.font = 2;
    d.crc = string_crc("MenuTitle");
    d.position = [256.0, f32::from_bits(1115815936), 0.0, 1.0];
    d.string = 0;
    o.menu_title = fe.create_element(o.layer, &d);
    // 0x00281250 online message
    let mut d = desc(ELEMENT_2D, "OnlineMessageSprite", 0.0, 0.0);
    d.texture = "invitation_Received".into();
    d.vcd = "\\FE\\FEOnline.vcd".into();
    o.online_message = fe.create_element(o.menu_layer, &d);
    // 0x00281800 countdown
    let mut d = desc(ELEMENT_STATIC_TEXT, "CountdownText", 256.0, 0.0);
    d.font = 3;
    d.string = 0;
    o.countdown_text = fe.create_element(o.layer, &d);
    // 0x00279980 transition
    let mut d = desc(ELEMENT_STATIC_TEXT, "TransitionText", 256.0, f32::from_bits(1137180672));
    d.font = 4;
    d.string = string_crc("GENERAL_LOADING");
    o.transition_text = fe.create_element(o.menu_layer, &d);
    let orange = crate::feck2::feck2_manager::visos_colour(128, 120, 77, 14);
    {
        let t = fe.text(o.transition_text);
        t.set_colour2(orange);
        t.set_justification(2);
        t.set_colour(orange & 0x00FF_FFFF);
    }
    d.kind = ELEMENT_2D;
    d.crc = string_crc("TransitionBackground");
    d.position = [f32::from_bits(1124728832), f32::from_bits(1126170624), 0.0, 1.0];
    d.texture = "FE_transition_img".into();
    d.vcd = FE_VCD.into();
    o.transition_img[0] = fe.create_element(o.menu_layer, &d);
    d.position = [f32::from_bits(1124073472), f32::from_bits(1124204544), 0.0, 1.0];
    d.texture = "FE_transition_img2".into();
    o.transition_img[1] = fe.create_element(o.menu_layer, &d);
    for i in 0..2 {
        let id = o.transition_img[i];
        fe.e2d(id).set_colour(COLOUR_WHITE & 0x00FF_FFFF, false, COLOUR_WHITE);
    }
    d.position = [0.0, 0.0, 0.0, 1.0];
    d.texture = "white".into();
    o.transition_bg = fe.create_element(o.menu_layer, &d);
    {
        let e = fe.e2d(o.transition_bg);
        e.set_size(512, 512);
        e.set_colour(COLOUR_BLACK & 0x00FF_FFFF, false, COLOUR_WHITE);
    }
    // 0x0027AAE0 nav prompts
    for i in 0..15 {
        create_nav(o, fe, i);
    }
    // 0x0027A3E0 borders
    create_borders(o, fe);
    // 0x002796C0 2D background (dynamic names, empty)
    let mut d = desc(ELEMENT_2D, "2DBackground", 0.0, 0.0);
    d.dynamic = true;
    o.background_2d = fe.create_element(o.bg_layer, &d);
    o.enabled = false;
    o.pre_enabled = false;
}

/// FE2DOverlay.cpp 0x0027AAE0 — `CreateNavIcon`
fn create_nav(o: &mut CFE2DOverlay, fe: &mut Feck2, i: usize) {
    let mut d = desc(ELEMENT_2D, NAV_NAMES[i][1], NAV_POS[i][0], NAV_POS[i][1]);
    d.font = 2;
    d.texture = NAV_NAMES[i][0].into();
    d.vcd = FE_VCD.into();
    o.nav_sprite[i] = fe.create_element(o.menu_layer, &d);
    d.kind = ELEMENT_STATIC_TEXT;
    d.font = 0;
    d.crc = string_crc(NAV_NAMES[i][2]);
    d.position = [NAV_POS[i][0] + 32.0, NAV_POS[i][1] + 10.0, 0.0, 1.0];
    d.string = 0;
    o.nav_text[i] = fe.create_element(o.menu_layer, &d);
    show_nav(o, fe, i, false);
}

/// FE2DOverlay.cpp 0x0027A3E0 — `CreateBorders`
fn create_borders(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    let mut d = desc(ELEMENT_2D, "FE_Icon", f32::from_bits(1110966272), 0.0);
    o.borders[ICON] = fe.create_element(o.bg_layer, &d);
    d.crc = string_crc("FE_Border_Options");
    d.texture = "FE_title_Highlight".into();
    d.position = [0.0, f32::from_bits(1105723392), 0.0, 1.0];
    d.vcd = FE_VCD.into();
    o.borders[TITLE_HIGHLIGHT] = fe.create_element(o.bg_layer, &d);
    fe.e2d(o.borders[TITLE_HIGHLIGHT]).set_size(503, 69);
    for (slot, name, x, y) in [
        (B_MENU_HEADER, "FE_Border_B_Menu_Header", 1132462080u32, 1134657536u32),
        (B_MENU_CONTENT, "FE_Border_B_Menu_Content", 0, 1134657536),
        (R_MENU_CONTENT, "FE_Border_R_Menu_Content", 1132462080, 1124073472),
        (INFO, "FE_Border_INFO", 1133248512, 1135706112),
        (TOP, "FE_Border_TOP", 0, 0),
        (NAV, "FE_Border_NAV", 0, 1136984064),
    ] {
        d.crc = string_crc(name);
        d.position = [f32::from_bits(x), f32::from_bits(y), 0.0, 1.0];
        d.texture = name.into();
        d.vcd = FE_VCD.into();
        o.borders[slot] = fe.create_element(o.bg_layer, &d);
    }
    d.crc = string_crc("FE_Border_Options");
    d.position = [f32::from_bits(1103626240), f32::from_bits(1137573888), 0.0, 1.0];
    d.texture = "menubackground".into();
    o.borders[OPTIONS_BG] = fe.create_element(o.bg_layer, &d);
}

/// FE2DOverlay.cpp 0x0027AD30 — `ShowNavIcon`
pub fn show_nav(o: &mut CFE2DOverlay, fe: &mut Feck2, i: usize, show: bool) {
    fe.el(o.nav_sprite[i]).set_visible(show);
    fe.el(o.nav_text[i]).set_visible(show);
    o.nav_shown[i] = show;
    if show {
        fe.element_fade(o.nav_text[i], 100, 1, 0);
    } else {
        fe.element_fade(o.nav_text[i], 0, 1, 0);
    }
}

/// FE2DOverlay.cpp 0x0027D3D0 — `PositionNavIcon`: sprite at (x, y), text after it centred on its height
pub fn position_nav(o: &mut CFE2DOverlay, fe: &mut Feck2, i: usize, x: f32, y: f32) {
    fe.element_set_position(o.nav_sprite[i], x, y, 0.0);
    fe.element_load(o.nav_sprite[i]);
    let (w, h) = {
        let s = fe.e2d(o.nav_sprite[i]);
        (s.width, s.height)
    };
    let th = fe.text_height(o.nav_text[i]);
    let dy = (h as f32 - th as f32) * 0.5;
    fe.element_set_position(o.nav_text[i], w as f32 + x, y + dy, 0.0);
}

/// FE2DOverlay.cpp 0x0027ACB0 — `SetNavText`
pub fn set_nav_text(o: &mut CFE2DOverlay, fe: &mut Feck2, i: usize, crc: u32) {
    {
        let t = fe.text(o.nav_text[i]);
        t.set_string(crc);
        t.set_justification(0);
        t.set_colour(COLOUR_WHITE);
    }
    show_nav(o, fe, i, true);
}

/// FE2DOverlay.cpp 0x0027D250 — `HideOptionsIcons`
pub fn set_hide_options(o: &mut CFE2DOverlay, fe: &mut Feck2, hide: bool) {
    o.hide_options = hide;
    for id in [o.nav_sprite[5], o.nav_text[5], o.nav_sprite[6], o.nav_text[6], o.borders[OPTIONS_BG]] {
        fe.el(id).set_visible(!hide);
    }
}

/// FE2DOverlay.cpp 0x0027A970 — `ShowBorders` (top and nav bar only)
pub fn show_borders(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    for (slot, v) in [(B_MENU_CONTENT, false), (R_MENU_CONTENT, false), (B_MENU_HEADER, false), (ICON, false), (INFO, false), (TOP, true), (NAV, true), (OPTIONS_BG, false)] {
        fe.el(o.borders[slot]).set_visible(v);
    }
}

/// FE2DOverlay.cpp 0x002820B0 — `Enable`
pub fn enable(o: &mut CFE2DOverlay) {
    o.enabled = true;
}

/// FE2DOverlay.cpp 0x002820D0 — `AttachToScreen`: positions the nav prompts, sets the start/back texts.
pub fn attach_to_screen(o: &mut CFE2DOverlay, fe: &mut Feck2, screen: Option<ScreenId>) {
    o.screen = screen;
    match screen {
        Some(s) => {
            o.screen_crc = fe.screens[s].crc;
            o.enabled = true;
            o.pre_enabled = true;
            for i in 0..15 {
                fe.element_set_position(o.nav_sprite[i], NAV_POS[i][0], NAV_POS[i][1], 0.0);
                fe.element_set_position(o.nav_text[i], NAV_POS[i][0] + 32.0, NAV_POS[i][1] + 10.0, 0.0);
            }
            fe.text(o.nav_text[5]).set_string(string_crc("GENERAL_OPTIONS"));
            fe.text(o.nav_text[6]).set_string(string_crc("GENERAL_ONLINE"));
        }
        None => {
            o.enabled = false;
            o.pre_enabled = false;
        }
    }
}

/// FE2DOverlay.cpp 0x00281C40 / 0x00279FC0 — `Reset`: hide everything screen specific and fade the persistent parts in.
pub fn reset(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    fe.el(o.background_2d).set_visible(false);
    for i in 0..9 {
        fe.el(o.borders[i]).set_visible(false);
    }
    // 0x00279FC0: park the top and nav borders off screen for the slide in
    fe.element_set_position(o.borders[TOP], -512.0, 0.0, 0.0);
    fe.element_set_position(o.borders[NAV], -512.0, f32::from_bits(1136984064), 0.0);
    for i in 0..15 {
        fe.el(o.nav_sprite[i]).set_visible(false);
        fe.el(o.nav_text[i]).set_visible(false);
        o.nav_shown[i] = false;
    }
    for id in [o.message_text, o.message_bg, o.dialog_item] {
        fe.el(id).set_visible(false);
    }
    for i in 0..8 {
        fe.el(o.dialog_option[i]).set_visible(false);
    }
    for id in [
        o.dialog_bg, o.dialog_indicator, o.background, o.arrows[0], o.arrows[1], o.arrows[2], o.arrows[3], o.game_logo,
        o.static_logo, o.loading_text, o.total_cash, o.prestige_points, o.title_text, o.heading_text, o.heading_bar,
    ] {
        fe.el(id).set_visible(false);
    }
    crate::feck2::screens::common::fe_2d_overlay_menu::set_active(o, fe, false);
    crate::feck2::screens::common::fe_2d_overlay_menu::clear(o, fe);
    o.menu_window = -1;
    o.menu_font = 1;
    o.hide_options = false;
    o.dialog_active = false;
    o.no_up_down_sound = false;
    o.flag534 = false;
    o.no_back_sound = false;
    show_nav(o, fe, 5, true);
    show_nav(o, fe, 6, true);
    o.message_time = -1.0;
    o.message_y = f32::from_bits(1114636288);
    o.menu_style = 2;
    o.menu_slide = 0;
    o.menu_horizontal = false;
    // fades to 100% over 25: arrows, menu entries, dialog, menu title, borders, title/heading, cash/prestige
    for id in o.arrows {
        fe.element_fade(id, 100, 25, 0);
    }
    for id in [o.dialog_item, o.menu_title] {
        fe.element_fade(id, 100, 25, 0);
    }
    for i in 0..9 {
        fe.element_fade(o.borders[i], 100, 25, 0);
    }
    for id in [o.title_text, o.heading_text, o.heading_bar, o.total_cash, o.prestige_points] {
        fe.element_fade(id, 100, 25, 0);
    }
}

/// FE2DOverlay.cpp 0x00279800 — `Set2DBackground`
pub fn set_2d_background(o: &mut CFE2DOverlay, fe: &mut Feck2, texture: &str, vcd: &str, full: bool, colour: u32) {
    let id = o.background_2d;
    {
        let e = fe.e2d(id);
        if full {
            e.set_size(512, 512);
        }
        e.set_colour(colour, false, COLOUR_WHITE);
        e.change_texture_vcd(vcd);
    }
    fe.change_texture(id, texture, true);
    fe.el(id).set_visible(true);
    fe.element_fade(id, 100, 0, 0);
}

/// FE2DOverlay.cpp 0x00279910 — `Fade2DBackground(percent, time)`
pub fn fade_2d_background(o: &mut CFE2DOverlay, fe: &mut Feck2, percent: i32, time: i32) {
    fe.element_fade(o.background_2d, percent, time, 0);
}

/// run `f` on the overlay taken out of the frontend (the overlay's functors are members of the overlay object).
fn with_overlay(fe: &mut Feck2, f: impl FnOnce(&mut CFE2DOverlay, &mut Feck2) -> bool) -> bool {
    let Some(mut o) = fe.overlay.take() else { return true };
    let r = f(&mut o, fe);
    fe.overlay = Some(o);
    r
}

/// FE2DOverlay.cpp 0x00276F10 — the `COMMON2DOVERLAY` layer functors (the ones the ported screens rely on).
pub fn assign_overlay_events(fe: &mut Feck2) {
    let Some(layer) = fe.overlay.as_ref().map(|o| o.layer) else { return };
    fe.map(layer, std::sync::Arc::new(|fe: &mut Feck2, _e: &CEvent| with_overlay(fe, on_enter)), ev::ENTER);
    fe.map(layer, std::sync::Arc::new(|fe: &mut Feck2, _e: &CEvent| with_overlay(fe, on_activate)), ev::ACTIVATE);
    fe.map(layer, std::sync::Arc::new(|fe: &mut Feck2, e: &CEvent| { let e = *e; with_overlay(fe, |o, fe| on_update(o, fe, &e)) }), ev::UPDATE);
}

/// FE2DOverlay.cpp 0x00277910 / 0x00281370 — Enter: nav option texts, then load the assets once.
fn on_enter(o: &mut CFE2DOverlay, fe: &mut Feck2) -> bool {
    if !o.assets_loaded {
        set_nav_text(o, fe, 5, string_crc("GENERAL_OPTIONS"));
        set_nav_text(o, fe, 6, string_crc("GENERAL_ONLINE"));
        // 0x00281370: online message sprite
        fe.element_load(o.online_message);
        fe.el(o.online_message).set_visible(false);
        fe.e2d(o.online_message).set_colour(COLOUR_WHITE, false, COLOUR_WHITE);
    }
    load_assets(o, fe);
    true
}

/// FE2DOverlay.cpp 0x002786B0 — `LoadAssets`
fn load_assets(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    fe.element_load(o.title_text);
    fe.element_load(o.heading_text);
    fe.text(o.heading_text).set_justification(1);
    fe.element_load(o.heading_bar);
    fe.element_load(o.background_2d);
    fe.element_load(o.transition_bg);
    fe.element_load(o.transition_text);
    for id in o.transition_img {
        fe.element_load(id);
    }
    for id in o.borders {
        fe.element_load(id);
    }
    fe.e2d(o.borders[OPTIONS_BG]).set_size(110, 30);
    for i in 0..15 {
        fe.element_load(o.nav_sprite[i]);
        fe.element_load(o.nav_text[i]);
        fe.text(o.nav_text[i]).set_colour2(COLOUR_WHITE);
    }
    fe.element_load(o.message_text);
    fe.text(o.message_text).set_justification(2);
    fe.el(o.message_text).set_visible(false);
    fe.text(o.message_text).set_box(f32::from_bits(1138819072), f32::from_bits(1125515264));
    fe.element_load(o.message_bg);
    fe.el(o.message_bg).set_visible(false);
    fe.element_load(o.dialog_item);
    fe.text(o.dialog_item).set_box(f32::from_bits(1134559232), 256.0);
    for id in o.dialog_option {
        fe.element_load(id);
    }
    fe.element_load(o.dialog_bg);
    fe.element_load(o.dialog_indicator);
    let orange = crate::feck2::feck2_manager::visos_colour(128, 120, 77, 14);
    {
        let e = fe.e2d(o.dialog_indicator);
        e.set_colour(orange, false, COLOUR_WHITE);
        e.set_size(6, 10);
    }
    let arrow = crate::feck2::feck2_manager::visos_colour(128, 255, 110, 0);
    fe.element_load(o.arrows[0]);
    fe.e2d(o.arrows[0]).set_flags(crate::feck2::feck2_elements::feck2_element_2d::FLAG_FLIP_U);
    fe.e2d(o.arrows[0]).set_colour(arrow, false, COLOUR_WHITE);
    fe.element_load(o.arrows[1]);
    fe.e2d(o.arrows[1]).set_colour(arrow, false, COLOUR_WHITE);
    fe.element_load(o.arrows[3]);
    fe.e2d(o.arrows[3]).set_flags(crate::feck2::feck2_elements::feck2_element_2d::FLAG_FLIP_V);
    fe.e2d(o.arrows[3]).set_colour(arrow, false, COLOUR_WHITE);
    fe.element_load(o.arrows[2]);
    fe.e2d(o.arrows[2]).set_colour(arrow, false, COLOUR_WHITE);
    fe.element_load(o.countdown_text);
    fe.el(o.countdown_text).set_visible(false);
    fe.text(o.countdown_text).set_justification(2);
    fe.element_load(o.loading_text);
    fe.element_load(o.total_cash);
    fe.element_load(o.cash_icon);
    fe.el(o.cash_icon).set_visible(false);
    fe.element_load(o.prestige_points);
    fe.element_load(o.prestige_icon);
    fe.el(o.prestige_icon).set_visible(false);
    fe.element_load(o.game_logo);
    fe.el(o.game_logo).set_visible(false);
    fe.element_load(o.static_logo);
    fe.el(o.static_logo).set_visible(false);
    fe.element_load(o.background);
    fe.element_load(o.menu_indicator);
    {
        let e = fe.e2d(o.menu_indicator);
        e.set_colour(orange, false, COLOUR_WHITE);
        e.set_size(6, 10);
    }
    fe.el(o.menu_indicator).set_visible(false);
    fe.element_load(o.menu_highlighter);
    {
        let e = fe.e2d(o.menu_highlighter);
        e.set_colour(crate::feck2::feck2_manager::visos_colour(48, 32, 32, 32), false, COLOUR_WHITE);
        e.set_size(128, 24);
    }
    fe.el(o.menu_highlighter).set_visible(false);
    fe.element_load(o.menu_title);
    {
        let t = fe.text(o.menu_title);
        t.set_justification(2);
        t.set_colour(orange);
        t.set_colour2(orange);
    }
    fe.el(o.menu_title).set_visible(false);
    o.assets_loaded = true;
}

/// FE2DOverlay.cpp 0x002779F0 / 0x0027A090 — Activate: options/online prompts, slide the top and nav bars in.
fn on_activate(o: &mut CFE2DOverlay, fe: &mut Feck2) -> bool {
    fe.el(o.nav_sprite[5]).set_visible(false);
    fe.el(o.nav_text[5]).set_visible(false);
    fe.el(o.nav_sprite[6]).set_visible(!o.hide_options);
    fe.el(o.nav_text[6]).set_visible(!o.hide_options);
    o.nav_shown[5] = false;
    o.nav_shown[6] = !o.hide_options;
    // 0x0027A090
    o.transition_state = -1;
    o.transition_timer = 40;
    let t = if o.first_activate { 0 } else { 10 };
    slide_in(o, fe, t);
    o.first_activate = false;
    true
}

/// FE2DOverlay.cpp 0x00279D40 — `SlideIn`
pub fn slide_in(o: &mut CFE2DOverlay, fe: &mut Feck2, time: i32) {
    for id in [o.borders[TOP], o.borders[NAV]] {
        fe.el(id).base_mut().move_by_x(512, time, 0);
        fe.element_fade(id, 100, time, 0);
    }
    for i in 0..15 {
        fe.element_fade(o.nav_text[i], 100, time / 2, 0);
        fe.element_fade(o.nav_sprite[i], 100, time / 2, 0);
    }
    fe.element_fade(o.title_text, 100, time / 2, 0);
    o.slid_in = true;
}

/// FE2DOverlay.cpp 0x00277BE0 — Update (message timeout part).
fn on_update(o: &mut CFE2DOverlay, fe: &mut Feck2, _e: &CEvent) -> bool {
    bob_arrows(o, fe, 0);
    bob_arrows(o, fe, 1);
    if o.message_time > -1.0 {
        o.message_time += 1.0;
        if o.message_time >= o.message_y && o.message_y != -1.0 {
            fe.el(o.message_text).set_visible(false);
            fe.el(o.message_bg).set_visible(false);
            o.message_time = -1.0;
        }
    }
    true
}

/// FE2DOverlay.cpp 0x002793A0 / 0x002793F0 / 0x00279420 — `SetTitle` (string, show)
pub fn set_title(o: &mut CFE2DOverlay, fe: &mut Feck2, crc: u32) {
    fe.text(o.title_text).set_string(crc);
    fe.el(o.title_text).set_visible(true);
}

/// FE2DOverlay.cpp 0x0027C320 — `ShowArrows`: 0 = left/right pair, 1 = up/down pair.
pub fn show_arrows(o: &mut CFE2DOverlay, fe: &mut Feck2, pair: u32) {
    let (a, b) = if pair != 0 { (o.arrows[2], o.arrows[3]) } else { (o.arrows[0], o.arrows[1]) };
    fe.el(a).set_visible(true);
    fe.el(b).set_visible(true);
}

/// FE2DOverlay.cpp 0x0027C3A0 — `SetArrowsVisible(visible, pair)`
pub fn set_arrows_visible(o: &mut CFE2DOverlay, fe: &mut Feck2, visible: bool, pair: u32) {
    let (a, b) = match pair {
        0 => (o.arrows[0], o.arrows[1]),
        1 => (o.arrows[2], o.arrows[3]),
        _ => return,
    };
    fe.el(a).set_visible(visible);
    fe.el(b).set_visible(visible);
}

/// FE2DOverlay.cpp 0x0027ADE0 — `SetNavString`: only the text, visibility untouched.
pub fn set_nav_string(o: &mut CFE2DOverlay, fe: &mut Feck2, i: usize, crc: u32) {
    fe.text(o.nav_text[i]).set_string(crc);
}

/// FE2DOverlay.cpp 0x0027C420 — `PositionArrows`
pub fn position_arrows(o: &mut CFE2DOverlay, fe: &mut Feck2, a: f32, b: f32, c: f32, pair: u32) {
    if pair == 0 {
        fe.element_set_position(o.arrows[0], a, c, 0.0);
        fe.element_set_position(o.arrows[1], b, c, 0.0);
    } else if pair == 1 {
        fe.element_set_position(o.arrows[2], a, b, 0.0);
        fe.element_set_position(o.arrows[3], a, c, 0.0);
    }
}

/// FE2DOverlay.cpp 0x0027C6A0 — `FadeArrows`
pub fn fade_arrows(o: &mut CFE2DOverlay, fe: &mut Feck2, pct: i32, pair: u32, time: i32) {
    let (a, b) = if pair != 0 { (o.arrows[2], o.arrows[3]) } else { (o.arrows[0], o.arrows[1]) };
    fe.element_fade(a, pct, time, 0);
    fe.element_fade(b, pct, time, 0);
}

/// FE2DOverlay.cpp 0x00279E80 — `SlideOut`: top/nav bars move off, prompts and title fade.
pub fn slide_out(o: &mut CFE2DOverlay, fe: &mut Feck2, time: i32) {
    for id in [o.borders[TOP], o.borders[NAV]] {
        fe.el(id).base_mut().move_by_x(512, time, 0);
        fe.element_fade(id, 0, time, 0);
    }
    for i in 0..15 {
        fe.element_fade(o.nav_text[i], 0, time / 2, 0);
        fe.element_fade(o.nav_sprite[i], 0, time / 2, 0);
    }
    fe.element_fade(o.title_text, 0, time / 2, 0);
    o.slid_in = false;
}

/// FE2DOverlay.cpp 0x0027C500 — `BobArrows`: 15 frame period, moves for the first 5.
fn bob_arrows(o: &mut CFE2DOverlay, fe: &mut Feck2, pair: u32) {
    if o.arrow_timer >= 15 {
        o.arrow_dir = if o.arrow_dir == -1 { 1 } else { -1 };
        o.arrow_timer = 0;
    } else {
        o.arrow_timer += 1;
    }
    if o.arrow_timer < 5 {
        if pair == 0 {
            fe.el(o.arrows[0]).base_mut().move_by_x(o.arrow_dir, 0, 0);
            fe.el(o.arrows[1]).base_mut().move_by_x(-o.arrow_dir, 0, 0);
        } else {
            fe.el(o.arrows[2]).base_mut().move_by_y(o.arrow_dir, 0, 0);
            fe.el(o.arrows[3]).base_mut().move_by_y(-o.arrow_dir, 0, 0);
        }
    }
}

/// FE2DOverlay.cpp 0x0027B1A0 — `SetMessage(string)`
pub fn set_message(o: &mut CFE2DOverlay, fe: &mut Feck2, crc: u32) {
    fe.text(o.message_text).set_string(crc);
    fe.el(o.message_text).set_visible(true);
}

/// FE2DOverlay.cpp 0x00279140 — everything fades out over 10 frames (arrows, menu, title, heading, cash,
/// prestige); the nav prompt / border parts (0x0027AEA0 / 0x0027AF80) are not ported
pub fn fade_out_all(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    fade_arrows(o, fe, 0, 0, 10);
    fade_arrows(o, fe, 0, 1, 10);
    crate::feck2::screens::common::fe_2d_overlay_menu::fade(o, fe, 0, 10);
    crate::feck2::screens::common::fe_2d_overlay_menu::fade_title(o, fe, 0, 10);
    for id in [o.title_text, o.heading_text, o.heading_bar, o.total_cash, o.prestige_points] {
        fe.element_fade(id, 0, 10, 0);
    }
}

/// FE2DOverlay.cpp 0x0027B200 — `SetMessage(string id)`: the backing (460 x height + 50) and the text centred
pub fn set_message_string(o: &mut CFE2DOverlay, fe: &mut Feck2, crc: u32) {
    fe.text(o.message_text).set_string(crc);
    let h = fe.text_height(o.message_text);
    fe.e2d(o.message_bg).set_size(460, h + 50);
    fe.element_set_position(o.message_text, f32::from_bits(1132462080), (256.0 - (h >> 1) as f64) as f32, 0.0);
    fe.element_set_position(o.message_bg, f32::from_bits(1106771968), (256.0 - (h >> 1) as f64 - 20.0) as f32, 0.0);
}

/// FE2DOverlay.cpp 0x0027B320 — `SetMessageText(text)`: the backing is sized and both are centred on the text.
pub fn set_message_text(o: &mut CFE2DOverlay, fe: &mut Feck2, text: &[u16]) {
    fe.text(o.message_text).set_text(text);
    let h = fe.text_height(o.message_text);
    fe.e2d(o.message_bg).set_size(460, h + 50);
    fe.element_set_position(o.message_text, 256.0, (h >> 1) as f32 + 256.0, 0.0);
    fe.element_set_position(o.message_bg, f32::from_bits(1106771968), ((h >> 1) as f64 + 256.0 - 20.0) as f32, 0.0);
}

/// FE2DOverlay.cpp 0x0027B440 — `ShowMessage`: starts the timer when shown.
pub fn show_message(o: &mut CFE2DOverlay, fe: &mut Feck2, show: bool) {
    o.message_time = if show { 0.0 } else { -1.0 };
    fe.el(o.message_text).set_visible(show);
    fe.el(o.message_bg).set_visible(show);
}

/// FE2DOverlay.cpp 0x0027B4C0 — `SetMessageTimeout` (frames, -1 = until hidden)
pub fn set_message_timeout(o: &mut CFE2DOverlay, frames: i32) {
    o.message_y = frames as f32;
}

/// FE2DOverlay.cpp 0x0027B7F0 — `ShowDialog(count, title, option...)`: strings set, unused options hidden (the
/// dialog itself appears with `SetDialogVisible`).
pub fn show_dialog(o: &mut CFE2DOverlay, fe: &mut Feck2, count: u32, title: u32, options: &[u32]) {
    o.dialog_count = count;
    fe.text(o.dialog_item).set_string(title);
    for i in 0..count as usize {
        fe.text(o.dialog_option[i]).set_string(options.get(i).copied().unwrap_or(0));
    }
    for i in count as usize..8 {
        fe.el(o.dialog_option[i]).set_visible(false);
    }
}

/// FE2DOverlay.cpp 0x0027B980 — `SetDialogCount(n)`: options past `n` hidden, newly counted ones shown, the
/// indicator shown while there is any option
pub fn set_dialog_count(o: &mut CFE2DOverlay, fe: &mut Feck2, n: u32) {
    if n < o.dialog_count {
        for j in n..o.dialog_count {
            fe.el(o.dialog_option[j as usize]).set_visible(false);
        }
    } else if n > o.dialog_count {
        let mut i = n - 1;
        while i > o.dialog_count.wrapping_sub(1) {
            fe.el(o.dialog_option[i as usize]).set_visible(true);
            i -= 1;
        }
    }
    fe.el(o.dialog_indicator).set_visible(n != 0);
    o.dialog_count = n;
}

/// FE2DOverlay.cpp 0x0027B8E0 — `SetDialogStrings(title, option...)`
pub fn set_dialog_strings(o: &mut CFE2DOverlay, fe: &mut Feck2, title: u32, options: &[u32]) {
    fe.text(o.dialog_item).set_string(title);
    for i in 0..o.dialog_count as usize {
        fe.text(o.dialog_option[i]).set_string(options.get(i).copied().unwrap_or(0));
    }
}

/// FE2DOverlay.cpp 0x0027BA70 — `SetDialogVisible(show, selection)`: the indicator sits at the selected option.
pub fn set_dialog_visible(o: &mut CFE2DOverlay, fe: &mut Feck2, show: bool, selection: i32) {
    o.dialog_active = show;
    if show {
        o.dialog_selection = selection;
    }
    fe.element_set_position(o.dialog_indicator, f32::from_bits(1126825984), (25 * o.dialog_selection + 194) as f32, 0.0);
    fe.el(o.dialog_item).set_visible(show);
    for i in 0..o.dialog_count as usize {
        fe.el(o.dialog_option[i]).set_visible(show);
    }
    fe.el(o.dialog_bg).set_visible(show);
    fe.el(o.dialog_indicator).set_visible(show);
}

/// FE2DOverlay.cpp 0x0027BB70 / 0x0027BC20 — move the dialog selection (-1 / +1), wrapping
pub fn move_dialog_selection(o: &mut CFE2DOverlay, fe: &mut Feck2, d: i32) {
    o.dialog_selection += d;
    if o.dialog_selection < 0 {
        o.dialog_selection = o.dialog_count as i32 - 1;
    } else if o.dialog_selection >= o.dialog_count as i32 {
        o.dialog_selection = 0;
    }
    fe.element_set_position(o.dialog_indicator, f32::from_bits(1126825984), (25 * o.dialog_selection + 194) as f32, 0.0);
}

/// FE2DOverlay.cpp 0x002795E0 — `SetHeading(string)`: heading text shown, its bar hidden
pub fn set_heading(o: &mut CFE2DOverlay, fe: &mut Feck2, crc: u32) {
    fe.text(o.heading_text).set_string(crc);
    fe.el(o.heading_text).set_visible(true);
    fe.el(o.heading_bar).set_visible(false);
}

/// FE2DOverlay.cpp 0x0027AA90 — `SetIcon(name, vcd, ...)`: only hides the icon border in this build
pub fn set_icon(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    fe.el(o.borders[ICON]).set_visible(false);
}

/// FE2DOverlay.cpp 0x0027CB10 / 0x0027CD00 — show the cash / prestige readout with the info border
pub fn show_cash(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    fe.el(o.borders[INFO]).set_visible(true);
    fe.el(o.total_cash).set_visible(true);
}

pub fn show_prestige(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    fe.el(o.prestige_points).set_visible(true);
    fe.el(o.borders[INFO]).set_visible(true);
}

/// FE2DOverlay.cpp 0x0027CDD0 — the cash readout's `dCash` argument from the player's cash
pub fn update_cash(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    let Some(s) = fe.text(o.total_cash).string else { return };
    let cash = fe.garage.cash as u32;
    fe.strings.set_argument(s, string_crc("dCash"), cash).ok();
}

/// FE2DOverlay.cpp 0x0027A030 — start the screen transition (the overlay's update plays it)
pub fn start_transition(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    o.transition_state = 1;
    o.transition_timer = 0;
    fe.element_set_position(o.transition_img[0], f32::from_bits(1124728832), f32::from_bits(1126170624), 0.0);
}

/// FE2DOverlay.cpp 0x0027A0C0 — end the screen transition: background faded out, pictures hidden
pub fn end_transition(o: &mut CFE2DOverlay, fe: &mut Feck2) {
    o.transition_state = 0;
    fe.element_fade(o.transition_bg, -128, 0, 0);
    for i in 0..2 {
        fe.el(o.transition_img[i]).set_visible(false);
    }
}
