//! `gpMultiplayerScreen` (`dword_75EB18`, 0x1C bytes; tag `FECK2CreateMultiplayerScreen`, create 0x00217230, Init
//! 0x00217360, AssignEvents 0x002175F0 — the start of the idb's FEOnlineMatchInfoScreen.cpp range, no `__FILE__`).
//! The main menu's MULTIPLAYER: Xbox Live (-> OnlineSignInScreen / OnlineMatchOptionsScreen), System Link
//! (-> NetworkMenuScreen) and Split Screen (two players through the quick race screens). Option 1 toggles the
//! debug transmission mode (peer to peer / client server) on nav prompt 2. The online / system link screens are not
//! ported (no Xbox Live / network layer): going there logs "Screen not found" and stays, as an unregistered CRC
//! does in the original.
//! IDA dump: `research/fe_mpmenu.c`.

use std::sync::{Arc, Mutex};

use crate::feck2::feck2_manager::Feck2;
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::string_manager::string_manager::string_crc;

/// `dword_75EDA0` (static init 0x0042B4F0..)
const ENTRIES: [&str; 3] = ["MULTIPLAYER_OPTION_ONLINE", "MULTIPLAYER_OPTION_SYSTEMLINK", "MULTIPLAYER_OPTION_SPLITSCREEN"];

/// `CFEMultiplayerScreen` (0x1C bytes)
pub struct CFEMultiplayerScreen {
    pub screen: ScreenId,       // +0
    pub layer: LayerId,         // +4
    pub loaded: bool,           // +8
    pub unused_12: i32,         // +12 (-1 on Start, never read)
    pub selection: i32,         // +16 the menu entry chosen last
    pub transmission: ElementId, // +20 "DebugTransmissionText"
    /// `byte_6F90BF`: Xbox Live (1) / System Link (0) chosen
    pub live: bool,
    /// `byte_6F90C2`: the sign-in screen was entered from here
    pub sign_in_from_menu: bool,
}

/// FEOnlineMatchInfoScreen.cpp 0x00217230 / 0x00217360 / 0x002175F0 / 0x00217F40 / 0x00217F70 — create, Init, AssignEvents (+ its cancel / option 1 functors)
pub fn create_multiplayer_screen(fe: &mut Feck2) -> Arc<Mutex<CFEMultiplayerScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("MultiplayerScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("MultiplayerScreen_Layer"), ..Default::default() });
    let mut d = SElementDesc::default();
    d.kind = ELEMENT_STATIC_TEXT;
    d.font = 0;
    d.position = [100.0, 300.0, 0.0, 1.0];
    d.crc = string_crc("DebugTransmissionText");
    d.string = 0;
    let transmission = fe.create_element(layer, &d);
    fe.text(transmission).set_string(string_crc("MULTIPLAYER_TOGGLE_TRANSMISSION"));
    let obj = Arc::new(Mutex::new(CFEMultiplayerScreen {
        screen,
        layer,
        loaded: false,
        unused_12: 0,
        selection: 0,
        transmission,
        live: false,
        sign_in_from_menu: false,
    }));
    fe.map(layer, functor(&obj, mp_enter), ev::ENTER);
    fe.map(layer, functor(&obj, mp_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEAVE);
    fe.map(layer, functor(&obj, |s, _, _| { s.unused_12 = -1; true }), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UPDATE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, mp_select), ev::SELECT_PRESS);
    // 0x00217F40
    fe.map(layer, functor(&obj, |_, fe, _| { fe.goto_previous_screen(1, 0); true }), ev::CANCEL_PRESS);
    // 0x00217F70
    fe.map(layer, functor(&obj, |_, fe, _| { fe.quick_race.peer_to_peer = !fe.quick_race.peer_to_peer; transmission_prompt(fe); true }), ev::OPTION1_PRESS);
    obj
}

/// FEOnlineMatchInfoScreen.cpp 0x00217AB0 — Enter: overlay, camera snapped, the debug text loaded
fn mp_enter(s: &mut CFEMultiplayerScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(mut o) = fe.overlay.take() {
        overlay::attach_to_screen(&mut o, fe, Some(s.screen));
        fe.overlay = Some(o);
    }
    // FE3DCamera 0x0026FB60(screen, 0) then 0x00271E80
    fe.foyer.snap_to_target();
    s.loaded = true;
    fe.element_load(s.transmission);
    true
}

/// FEOnlineMatchInfoScreen.cpp 0x00217FB0 — nav prompt 2 = the transmission mode (`byte_6F7159`)
fn transmission_prompt(fe: &mut Feck2) {
    let crc = string_crc(if fe.quick_race.peer_to_peer { "MULTIPLAYER_PEER_TO_PEER" } else { "MULTIPLAYER_CLIENT_SERVER" });
    if let Some(mut o) = fe.overlay.take() {
        overlay::set_nav_string(&mut o, fe, 2, crc); // 0x0027ADE0
        fe.overlay = Some(o);
    }
}

/// FEOnlineMatchInfoScreen.cpp 0x00217B30 — Activate: SELECT / BACK, the transmission prompt, borders, the three entries (style 1)
fn mp_activate(s: &mut CFEMultiplayerScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(mut o) = fe.overlay.take() {
        overlay::reset(&mut o, fe);
        overlay::set_nav_text(&mut o, fe, overlay::NAV_SELECT, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(&mut o, fe, overlay::NAV_CANCEL, string_crc("GENERAL_BACK"));
        overlay::set_nav_text(&mut o, fe, 2, 0);
        fe.overlay = Some(o);
    }
    transmission_prompt(fe);
    if let Some(mut o) = fe.overlay.take() {
        overlay::show_borders(&mut o, fe);
        overlay::enable(&mut o);
        menu::set_style(&mut o, fe, 1);
        for e in ENTRIES {
            menu::add_entry(&mut o, fe, string_crc(e), true, false, 0);
        }
        menu::show(&mut o, fe, true, false);
        menu::set_selection(&mut o, fe, s.selection);
        fe.overlay = Some(o);
    }
    true
}

/// FEOnlineMatchInfoScreen.cpp 0x00217D70 — Select
fn mp_select(s: &mut CFEMultiplayerScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.selection = fe.overlay.as_ref().map_or(0, |o| o.menu_selected);
    match s.selection {
        0 => {
            s.live = true;
            // NetworkGame state 11 (signed in) -> OnlineMatchOptionsScreen; there is no network game here
            s.sign_in_from_menu = true;
            fe.goto_screen_name("OnlineSignInScreen");
        }
        1 => {
            s.live = false;
            fe.goto_screen_name("NetworkMenuScreen");
        }
        2 => {
            fe.quick_race.two_players = true;
            // byte_6F70D8 = 1 (also set by the main menu's quick race; no reader found in the FE) and
            // dword_6F5C00 = 2 (GameManager player count): game flow takes it from `two_players` (request in core.md)
            fe.car_select_next = Some((0, string_crc("QuickRaceCarSelectScreen")));
            fe.goto_screen_name("QuickRaceTextScreen");
        }
        _ => {}
    }
    true
}
