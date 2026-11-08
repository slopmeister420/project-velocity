//! src\Feck2\Screens\Profile\FEProfileSelectScreen.cpp — `gpProfileSelectScreen` (`dword_7655CC`, 0x3C bytes; create
//! 0x00232C60, code 0x00232D60..0x00234EB0, the idb's FEOnlineSignInScreen range): the "Primary" saves as an overlay
//! list (5 slots, empty ones "PROFILE_EMPTY", then "DELETE PROFILE"), the highlighted profile's cash / prestige /
//! cars / progression, and new / load / save / delete.
//!
//! The memory unit layer (`XMem` status `dword_701404`, SAVE_GetFileListing_M) is core's: requests go out as
//! `CareerRequest`s (game flow's `NewProfile` / `LoadProfile` / `SaveProfile` / `DeleteProfile`) and the answers
//! (`CareerMessage`) take the place of the XMem "done" (3) / "failed" (-1) status polled in Update. The listing
//! is synchronous. A new profile is named on the on-screen keyboard (`gpCommonKeyboard`, state 2), pre-filled with
//! `unk_6F90DC` (empty: SetInitialVariables zeroes it and core keeps no name). Not ported: the XMem popups (`+40`,
//! 0x00234070). A new profile then goes to PlateCustomizationScreen; coming back (Start with +44) saves it.

use std::sync::{Arc, Mutex};

use crate::feck2::feck2_manager::Feck2;
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::feck2::screens::common::fe_keyboard_overlay::kb;
use crate::feck2::screens::garage::ov;
use crate::feck2::screens::hangout::{CareerMessage, CareerRequest};
use crate::string_manager::string_manager::string_crc;
use velocity_game::register_file_types::save_file_data;

/// SAVE_GetFileListing_M's limit (5 profiles)
pub const MAX_PROFILES: usize = 5;

/// one valid profile of the listing (0x00233EB0: `dword_7655D0` table, 12 bytes each, names `0x7659A8`)
#[derive(Clone, Debug, Default)]
pub struct ProfileInfo {
    pub name: String,
    pub cash: u32,      // save +0x1EA0
    pub prestige: u32,  // save +0x1EA4
    pub completed: u16, // save +0x3C34
    pub vehicles: u16,  // save +0x0004
}

/// `CFEProfileSelectScreen` (0x3C bytes)
pub struct CFEProfileSelectScreen {
    pub screen: ScreenId,     // +0
    pub layer: LayerId,       // +4
    pub title: ElementId,     // +8
    pub info: [ElementId; 4], // +12 cash, prestige, cars, progression
    pub done: bool,           // +28 a profile was made / loaded this visit
    pub loaded: bool,         // +29
    pub selection: i32,       // +32
    pub entries: u32,         // +36
    pub popup: i32,           // +40
    pub new_profile: bool,    // +44
    pub mode: u32,            // +48 0 load, 1 load (from options), 2 save, 3 delete (from load), 4 delete, 5 new
    pub state: u32,           // +52 0 listing, 1 menu, 2 keyboard, 3..7 waiting (4/6 save, 5 load, 7 delete), 8/9 leave, 10
    pub no_autoload: bool,    // +56
    pub profiles: Vec<ProfileInfo>,
    pub invalid: u32,         // dword_76598C
}

/// FEOnlineSignInScreen.cpp 0x00232C60 / 0x00232D60 / 0x002330D0 / 0x00233680 — create, Init, AssignEvents (+ Leave functor)
pub fn create_profile_select_screen(fe: &mut Feck2) -> Arc<Mutex<CFEProfileSelectScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("ProfileSelectScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 0, crc: string_crc("ProfileSelect_Layer"), ..Default::default() });
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 3, crc: string_crc("optionprofiletext"), ..Default::default() };
    d.position[0] = f32::from_bits(1119092736); // 90
    d.position[1] = f32::from_bits(1126825984); // 170
    d.string = string_crc("PROFILE_SELECT");
    let title = fe.create_element(layer, &d);
    let mut info = [0; 4];
    for (i, e) in info.iter_mut().enumerate() {
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = 1;
        d.crc = string_crc("optionprofiletext");
        d.position[0] = f32::from_bits(1124204544); // 130
        d.position[1] = (20 * i as u32 + 350) as f32;
        d.string = 0;
        *e = fe.create_element(layer, &d);
    }
    let obj = Arc::new(Mutex::new(CFEProfileSelectScreen {
        screen,
        layer,
        title,
        info,
        done: false,
        loaded: false,
        selection: 0,
        entries: 0,
        popup: -1,
        new_profile: false,
        mode: 0,
        state: 0,
        no_autoload: false,
        profiles: Vec::new(),
        invalid: 0,
    }));
    // 0x002330D0 — AssignEvents
    fe.map(layer, functor(&obj, ps_enter), ev::ENTER);
    fe.map(layer, functor(&obj, ps_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, fe, _| {
        // 0x00233680: the keyboard's name check cleared
        kb(fe, |k, _| k.validator = None);
        true
    }), ev::LEAVE);
    fe.map(layer, functor(&obj, ps_start), ev::START);
    fe.map(layer, functor(&obj, ps_update), ev::UPDATE);
    fe.map(layer, functor(&obj, |s, fe, _| ps_move(s, fe, -1)), ev::UP_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| ps_move(s, fe, 1)), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, ps_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, ps_cancel), ev::CANCEL_PRESS);
    obj
}

impl CFEProfileSelectScreen {
    /// FEOnlineSignInScreen.cpp 0x00232D40 / 0x002348B0 — what the next visit does (`mode`), and whether it
    /// may load straight away (`+56`)
    pub fn set_mode(&mut self, mode: u32, no_autoload: bool) {
        self.mode = mode;
        self.no_autoload = no_autoload;
    }
}

/// FEOnlineSignInScreen.cpp 0x00233430 — Enter: overlay and camera on this screen, texts loaded
fn ps_enter(s: &mut CFEProfileSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| overlay::attach_to_screen(o, fe, Some(s.screen)));
    fe.foyer.snap_to_target();
    if !s.loaded {
        s.loaded = true;
    }
    fe.element_load(s.title);
    for i in 0..4 {
        fe.element_load(s.info[i]);
    }
    true
}

/// FEOnlineSignInScreen.cpp 0x00233500 — Activate: overlay prompts, title, centred menu
fn ps_activate(s: &mut CFEProfileSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    let mode = s.mode;
    ov(fe, |o, fe| {
        let bg = fe.el(o.background_2d).is_visible();
        overlay::reset(o, fe);
        fe.el(o.background_2d).set_visible(bg);
        overlay::set_hide_options(o, fe, true);
        overlay::set_nav_text(o, fe, 0, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, 1, string_crc("GENERAL_CANCEL"));
        if mode == 0 {
            overlay::show_nav(o, fe, 1, false);
        }
        overlay::set_message(o, fe, 0);
        overlay::show_dialog(o, fe, 4, 0, &[]);
        overlay::set_heading(o, fe, string_crc("PROFILE_SCREEN_TITLE"));
        overlay::show_borders(o, fe);
        menu::set_style(o, fe, 3);
        overlay::set_message_timeout(o, -1);
        overlay::enable(o);
    });
    true
}

/// FEOnlineSignInScreen.cpp 0x002336B0 / 0x00234E40 — Start: a new profile coming back saves it, otherwise list the saves
fn ps_start(s: &mut CFEProfileSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    kb(fe, |k, _| k.validator = Some(valid_profile_name)); // 0x00234E40(0x00234BF0)
    // 0x0027D340: no left/right cursor sound; the title in the menu colours (0x00207CD0 / 0x00213C00)
    ov(fe, |o, fe| {
        o.flag534 = true;
        overlay::set_dialog_visible(o, fe, false, 0);
        overlay::show_message(o, fe, false);
    });
    s.popup = -1;
    if s.new_profile {
        s.new_profile = false;
        s.state = 4;
        fe.career.requests.push(CareerRequest::SaveProfile); // SaveFileData 0x00106040
    } else {
        s.new_profile = false;
        s.state = 0;
        list_profiles(s);
        s.entries = 5;
        if s.mode == 0 {
            s.entries += 1;
        }
        kb(fe, |k, fe| k.show(fe, false));
        ps_move(s, fe, 0);
    }
    true
}

/// FEOnlineSignInScreen.cpp 0x00233FB0 + 0x00233EB0 — the "Primary" saves: valid ones (version "Version_25")
/// counted with their cash / prestige / completed events / cars, the others counted as invalid
fn list_profiles(s: &mut CFEProfileSelectScreen) {
    s.profiles.clear();
    s.invalid = 0;
    for name in save_file_data::list_profiles().into_iter().take(MAX_PROFILES) {
        match save_file_data::load_primary(&name) {
            Some(p) if p.version == string_crc("Version_25") => s.profiles.push(ProfileInfo {
                name,
                cash: p.vars[0],
                prestige: p.vars[1],
                completed: p.num_completed as u16,
                vehicles: p.num_vehicles as u16,
            }),
            _ => s.invalid += 1,
        }
    }
}

/// FEOnlineSignInScreen.cpp 0x002337C0 — Update: the save layer's answer moves the state on, then leave / show
fn ps_update(s: &mut CFEProfileSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    // the listing finishes at once (XMem status 3 with state 0)
    if s.state == 0 {
        s.state = 1;
        build_menu(s, fe);
    }
    let msgs: Vec<CareerMessage> = std::mem::take(&mut fe.career.messages);
    let mut rest = Vec::new();
    for m in msgs {
        match (&m, s.state) {
            // status 3 (done)
            (CareerMessage::Loaded { ok: true, .. }, 5) => s.state = 9, // SaveFileData 0x001050C0 ran in core
            (CareerMessage::Saved { ok: true, .. }, 4 | 6) => s.state = 8,
            (CareerMessage::Deleted { ok: true, .. }, 7) => {
                if s.mode == 3 {
                    s.mode = 0;
                    s.state = 0;
                    list_profiles(s);
                } else {
                    s.state = 8;
                }
            }
            // status -1 (failed)
            (CareerMessage::Loaded { ok: false, .. } | CareerMessage::Saved { ok: false, .. } | CareerMessage::Deleted { ok: false, .. }, 4..=7) => {
                s.state = if s.state == 6 { 10 } else { 8 };
            }
            _ => rest.push(m),
        }
    }
    fe.career.messages = rest;
    if s.state == 0 {
        s.state = 1;
        build_menu(s, fe);
    }
    let mut show_info = false;
    match s.state {
        10 => s.state = 1,
        8 | 9 => {
            if s.new_profile {
                if fe.find_screen(string_crc("PlateCustomizationScreen")).is_some() {
                    fe.goto_screen_name("PlateCustomizationScreen");
                } else {
                    // no PlateCustomizationScreen made: save as its return (Start with +44) would
                    s.new_profile = false;
                    s.state = 4;
                    fe.career.requests.push(CareerRequest::SaveProfile);
                    return true;
                }
            } else if s.state == 8 {
                fe.goto_previous_screen(1, 0);
            } else {
                fe.goto_previous_screen(1, string_crc("MainMenuScreen"));
            }
            // byte_6F9059 = 0
        }
        1 => {
            if s.mode != 4 || !s.profiles.is_empty() {
                ov(fe, |o, fe| menu::show(o, fe, true, false));
                fe.el(s.title).set_visible(true);
                show_info = true; // dword_7013E4 (memory unit state) is always 0 here
            } else {
                s.state = 10;
            }
        }
        _ => {
            fe.el(s.title).set_visible(false);
            ov(fe, |o, fe| menu::show(o, fe, false, false));
        }
    }
    set_info_visible(s, fe, show_info);
    // the keyboard finished: its name becomes `unk_6F90DC` and the profile is made (0x00234550)
    let name = kb(fe, |k, _| (k.is_visible() && k.is_done()).then(|| k.text().to_vec())).flatten();
    if let Some(name) = name {
        kb(fe, |k, fe| k.show(fe, false));
        let name: String = name.iter().map(|&c| c as char).collect();
        new_profile(s, fe, name);
    }
    ps_move(s, fe, 0);
    true
}

/// FEOnlineSignInScreen.cpp 0x00234BF0 — the keyboard's name check: a name containing "empty" (any case) needs
/// some character above '!' before or after it
pub fn valid_profile_name(name: &[u8]) -> bool {
    let lower: Vec<u8> = name.iter().map(|c| c.to_ascii_lowercase()).collect();
    let Some(at) = lower.windows(5).position(|w| w == b"empty") else { return true };
    if lower.len() == 5 {
        return false;
    }
    // signed chars: bytes >= 0x80 are negative
    let above = |c: u8| (c as i8) > 33;
    lower[..at].iter().any(|&c| above(c)) || lower[at + 5..].iter().any(|&c| above(c))
}

/// FEOnlineSignInScreen.cpp 0x00234240 — the list: profiles, empty slots, "delete profile" (load mode)
fn build_menu(s: &mut CFEProfileSelectScreen, fe: &mut Feck2) {
    if s.invalid != 0 {
        let text = if (s.invalid as usize) < MAX_PROFILES {
            "WARNING ... SOME INVALID PROFILES FOUND ON MEMORY CARD!!!!"
        } else {
            "ERROR..... TOO MANY INVALID PROFILES FOUND.... LOAD AND SAVE WILL HAVE PROBLEMS!"
        };
        let t: Vec<u16> = text.encode_utf16().collect();
        ov(fe, |o, fe| {
            overlay::set_message_text(o, fe, &t);
            overlay::show_message(o, fe, true);
        });
    }
    let count = s.profiles.len();
    let title = match s.mode {
        3 | 4 => {
            s.entries = count as u32;
            Some("PROFILE_SELECT_DELETE")
        }
        2 => Some("PROFILE_SELECT_SAVE"),
        0 | 1 => Some("PROFILE_SELECT_LOAD"),
        _ => None,
    };
    if let Some(t) = title {
        fe.text(s.title).set_string(string_crc(t));
    }
    s.selection = 0;
    let names: Vec<String> = s.profiles.iter().map(|p| p.name.clone()).collect();
    let mode = s.mode;
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        menu::set_selection(o, fe, 0);
        for n in &names {
            menu::add_text_entry(o, fe, n, true);
        }
        if mode != 3 && mode != 4 {
            for _ in count..MAX_PROFILES {
                menu::add_entry(o, fe, string_crc("PROFILE_EMPTY"), true, false, 0);
            }
        }
        if mode == 0 && count != 0 {
            menu::add_entry(o, fe, string_crc("OPTION_DELETEPROFILE"), true, false, 0);
        }
    });
}

/// FEOnlineSignInScreen.cpp 0x00233E20 / 0x00233B80 / 0x00233BC0 — move the selection (clamped) and show the profile's details
fn ps_move(s: &mut CFEProfileSelectScreen, fe: &mut Feck2, d: i32) -> bool {
    if d != 0 && s.state != 1 {
        return true; // 0x00233B80 / 0x00233BC0: only while the list is up
    }
    let count = fe.overlay.as_ref().map(|o| o.menu_count).unwrap_or(0);
    s.selection += d;
    if s.selection < 0 {
        s.selection = 0;
    } else if s.selection >= count {
        s.selection = count - 1;
    }
    show_details(s, fe);
    if d != 0 {
        fe.el(s.title).set_visible(true);
        set_info_visible(s, fe, true);
    }
    true
}

/// FEOnlineSignInScreen.cpp 0x002348E0 — the highlighted profile's cash, prestige, cars and progression
/// (`100 * (completed / events)` in integers: 0 until every event is done, then 1 for any progress — kept)
fn show_details(s: &mut CFEProfileSelectScreen, fe: &mut Feck2) {
    let Some(p) = usize::try_from(s.selection).ok().and_then(|i| s.profiles.get(i)).cloned() else {
        set_info_visible(s, fe, false);
        return;
    };
    let total = (fe.career.events.len() as u32).max(1); // CompartmentManager 0x0008ECC0: the event count
    let mut percent = 100 * (p.completed as u32 / total);
    if percent == 0 {
        percent = (p.completed != 0) as u32;
    }
    let rows = [
        ("PROFILE_INFO_CASH", "dCash", p.cash),
        ("PROFILE_INFO_PRESTIGE", "dPrestige", p.prestige),
        ("PROFILE_INFO_PINKSLIPS", "dPinkSlips", p.vehicles as u32),
        ("PROFILE_INFO_PROGRESSION", "dPercent", percent),
    ];
    for (i, (id, arg, v)) in rows.into_iter().enumerate() {
        fe.text(s.info[i]).set_string(string_crc(id));
        if let Some(h) = fe.text(s.info[i]).string {
            fe.strings.set_argument(h, string_crc(arg), v).ok();
        }
    }
    set_info_visible(s, fe, true);
}

/// FEOnlineSignInScreen.cpp 0x00234B80 — the details only while the list is up
fn set_info_visible(s: &mut CFEProfileSelectScreen, fe: &mut Feck2, mut show: bool) {
    if s.state != 1 {
        show = false;
    }
    for i in 0..4 {
        fe.el(s.info[i]).set_visible(show);
    }
}

/// FEOnlineSignInScreen.cpp 0x00233C00 / 0x00234700 / 0x002347F0 / 0x002345E0 — Select (+ save / delete / load, I/O by core): an empty slot makes a profile, "delete profile" switches to
/// delete mode, otherwise load / save / delete the highlighted one
fn ps_select(s: &mut CFEProfileSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.state != 1 {
        return true;
    }
    let count = s.profiles.len() as i32;
    if s.selection >= count || s.mode == 5 {
        if s.mode != 0 || s.selection != 5 {
            // the keyboard asks for the name, pre-filled with `unk_6F90DC`
            kb(fe, |k, fe| {
                k.show(fe, true);
                k.set_string(fe, b"");
            });
            s.state = 2;
        } else {
            s.mode = 3;
            s.state = 1;
            build_menu(s, fe);
            ov(fe, |o, fe| overlay::show_nav(o, fe, 1, true));
        }
    } else if s.mode > 1 {
        let name = s.profiles[s.selection as usize].name.clone();
        if s.mode == 2 {
            // 0x00234700: save over the highlighted profile's name
            s.state = 6;
            fe.career.requests.push(CareerRequest::SaveProfile);
        } else if s.mode == 3 || s.mode == 4 {
            // 0x002347F0
            s.state = 7;
            fe.career.requests.push(CareerRequest::DeleteProfile { name });
        }
    } else {
        // 0x002345E0
        let name = s.profiles[s.selection as usize].name.clone();
        s.state = 5;
        s.done = true;
        fe.career.requests.push(CareerRequest::LoadProfile { name });
    }
    true
}

/// FEOnlineSignInScreen.cpp 0x00234550 — new profile (SetInitialVariables in core), then leave for the plate
fn new_profile(s: &mut CFEProfileSelectScreen, fe: &mut Feck2, name: String) {
    fe.career.requests.push(CareerRequest::NewProfile { name });
    s.new_profile = true;
    s.done = true;
    s.state = 8;
}

/// FEOnlineSignInScreen.cpp 0x00233D40 — Cancel: back out of delete mode, else leave (not from the first screen)
fn ps_cancel(s: &mut CFEProfileSelectScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if kb(fe, |k, _| k.is_visible()) == Some(true) {
        kb(fe, |k, fe| k.show(fe, false));
        s.state = 1;
        return true;
    }
    if !s.done {
        if s.mode == 4 {
            s.state = 1;
            s.mode = 1;
            build_menu(s, fe);
            return true;
        }
        if s.mode == 3 {
            s.state = 1;
            s.mode = 0;
            build_menu(s, fe);
            ov(fe, |o, fe| overlay::show_nav(o, fe, 1, false));
            return true;
        }
    }
    if fe.history_level != 0 {
        fe.goto_previous_screen(1, 0);
    }
    true
}
