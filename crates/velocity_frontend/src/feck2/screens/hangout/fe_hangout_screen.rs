//! src\Feck2\Screens\Hangout\FEHangoutScreen.cpp — `gpHangoutScreen` ("THE SCENE"; create 0x00218390, code
//! 0x002181A0..0x0021FA60, the idb's FEOnlineMatchInfoScreen / FEHangoutScreen ranges): the hangout's characters,
//! walked through with left/right. Character 0 (Tory) offers the career events (`CFEHangoutProgression`, +456);
//! 1 the garage (customise / pink slips / alloy editor), 2 the showroom (buy), 3 the DJ (playlists). Cancel asks
//! to quit to the main menu. A new profile that has completed nothing starts the tutorial event "UT0EVENT1" on
//! the first update (0x0021C430).
//!
//! Camera: every change flies along Catmull-Rom splines through the character cameras (0x0021B0D0 left / right
//! through the MOVERIGHT cameras, 0x0021AE70 from one camera kind to another), sampled into the point tables
//! `dword_75EDD8` / `dword_75EFB8` (0x0021AD30) and walked one FE3DCamera move of `+436` (0.06 s, linear) per point
//! (0x0021A310); the up vectors (+396 / +412) are not carried (the FE camera keeps +Y).
//!
//! Start (0x00219300) / Leave (0x00219920) show and hide the characters and their props, fade the music to the
//! character's Chars.txt volume / reverb and run the hangout ambience (`Feck2.music`, `VsAudio` loop event 210).
//!
//! Not ported: character anims on select (0x00262DB0), the loading screen
//! snapshot of the car (0x0021A310 `byte_75EDB4`), the timeline camera fraction (0x0021C7E0), the event detail texts
//! (0x0021BBF0, the elements are created and kept hidden), the
//! online results prompt (option 2 -> StatsScreen), the DJ's playlist entries (0x0021C2A0).

use std::sync::{Arc, Mutex};

use crate::feck2::feck2_manager::{visos_colour, FeMusicCall, Feck2};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::feck2::screens::garage::ov;
use crate::feck2::screens::common::fe_spline::CCatMullRom;
use crate::feck2::screens::hangout::fe_hangout_progression::{self as progression, CFEHangoutProgression};
use crate::feck2::screens::hangout::CareerRequest;
use crate::string_manager::string_manager::string_crc;
use velocity_audio::GuiSound;

/// `dword_6C1288`: frames of the screen transition before leaving
pub const EXIT_FRAMES: i32 = 20;
/// the quit dialog's title / options (0x0021A100: `27B8E0(0x34FE3B70, GENERAL_YES, GENERAL_NO)`)
const QUIT_TITLE: u32 = 889056496;
const GENERAL_YES: u32 = -1829768882i32 as u32;
const GENERAL_NO: u32 = -668354537i32 as u32;

/// `CFEHangoutScreen`
pub struct CFEHangoutScreen {
    pub screen: ScreenId,          // +0
    pub layer: LayerId,            // +4
    pub info: [ElementId; 6],      // +8 "infotext"
    pub headings: [ElementId; 3],  // +32 RACEINFO_TYPE / CATEGORY / EVENT
    pub race_info: [ElementId; 3], // +44
    pub background: ElementId,     // +152 GameTagBackgroundSprite
    pub headings2: [ElementId; 5], // +156
    pub unk_272: i32,              // +272 zeroed by Start (no other access seen)
    pub character: i32,            // +276 (0 only at create: Start keeps the character)
    pub previous: i32,             // +280
    pub selection: i32,            // +284
    /// +292 / +308 the camera Leave left (the character's `cam_kind` camera), +340 / +356 the copies Start puts
    /// the FE3DCamera on (ChangeCharacter rewrites them)
    pub cam_pos: [f32; 4],
    pub cam_look: [f32; 4],
    pub cam_pos2: [f32; 4],
    pub cam_look2: [f32; 4],
    pub first_activate: bool,      // +428
    pub chars_shown: bool,         // +429
    pub chars_created: bool,       // +430
    pub loaded: bool,              // +431
    pub move_timer: f32,           // +432
    pub move_time: f32,            // +436 0.06
    pub unk_440: i32,              // +440 -1 at create, 1 by Start
    pub slide: i32,                // +444
    pub slide_count: i32,          // +448
    pub menu_open: bool,           // +452
    pub rebuild_menu: bool,        // +453
    pub progression: CFEHangoutProgression, // +456
    pub camera: u32,               // +460 0 left, 1 right, 2 menu open, 3 menu closed, 4 leaving, 5 none
    pub cam_kind: u32,             // +464 camera kind of a change in place (2 with the menu open)
    pub entries: u32,              // +468
    /// `dword_75EDD8` / `dword_75EFB8` flight points (position, look-at), `dword_75EDBC` the current one
    pub flight: (Vec<[f32; 4]>, Vec<[f32; 4]>),
    pub flight_point: usize,
    /// +324 the flight's current up, +372 its copy for the next move, +396 / +412 the start / end cameras' up
    pub up: [f32; 4],
    pub up_prev: [f32; 4],
    pub up_from: [f32; 4],
    pub up_to: [f32; 4],
    pub quit_dialog: bool,         // +472
    pub started: bool,             // +473
    pub exit_camera: bool,         // +474
    pub exit_slide: bool,          // +475
    pub exit_countdown: i32,       // +476
    pub exit_screen: u32,          // +480
}

/// FEOnlineMatchInfoScreen.cpp 0x00218390 / 0x00218AF0 / FEHangoutScreen.cpp 0x0021A260 / 0x0021A2F0 — `FECK2CreateHangoutScreen` (AssignEvents + two functors)
pub fn create_hangout_screen(fe: &mut Feck2) -> Arc<Mutex<CFEHangoutScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("HangoutScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 2, crc: string_crc("HangoutScreen 2D Overlay layer"), ..Default::default() });
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 0, crc: string_crc("infotext"), ..Default::default() };
    d.position[0] = f32::from_bits(1135869952); // 360
    let mut info = [0; 6];
    for (i, e) in info.iter_mut().enumerate() {
        d.position[1] = (16 * i as u32 + 280) as f32;
        d.string = 0;
        *e = fe.create_element(layer, &d);
    }
    let raceinfo = ["RACEINFO_TYPE", "RACEINFO_CATEGORY", "RACEINFO_EVENT"]; // dword_75F43C
    let mut headings = [0; 3];
    let mut race_info = [0; 3];
    for i in 0..3 {
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = 0;
        d.crc = string_crc("currentraceheadings");
        d.position[0] = f32::from_bits(1135542272); // 350
        d.position[1] = (18 * i as u32 + 120) as f32;
        d.string = string_crc(raceinfo[i]);
        headings[i] = fe.create_element(layer, &d);
        d.crc = string_crc("currentraceinfo");
        d.position[0] = f32::from_bits(1137508352); // 410
        d.string = 0;
        race_info[i] = fe.create_element(layer, &d);
    }
    let mut headings2 = [0; 5];
    for (i, e) in headings2.iter_mut().enumerate() {
        d.crc = string_crc("currentraceheadings");
        d.position[0] = f32::from_bits(1132068864); // 250
        d.position[1] = (18 * i as u32 + 390) as f32;
        d.string = 0;
        *e = fe.create_element(layer, &d);
    }
    d.kind = ELEMENT_2D;
    d.crc = string_crc("GameTagBackgroundSprite");
    d.position[0] = f32::from_bits(1131413504); // 240
    d.position[1] = f32::from_bits(1136525312); // 380
    d.texture = "menubackground".into();
    d.vcd = "\\FE\\FE.vcd".into();
    let background = fe.create_element(layer, &d);
    // 0x0028B8A0 / 0x0028B900 on the screen's layer
    let progression = progression::create(fe, layer);
    let obj = Arc::new(Mutex::new(CFEHangoutScreen {
        screen,
        layer,
        info,
        headings,
        race_info,
        background,
        headings2,
        unk_272: 0,
        character: 0,
        previous: 0,
        selection: 0,
        cam_pos: [0.0; 4],
        cam_look: [0.0; 4],
        cam_pos2: [0.0; 4],
        cam_look2: [0.0; 4],
        first_activate: true,
        chars_shown: true,
        chars_created: false,
        loaded: false,
        move_timer: 0.0,
        move_time: f32::from_bits(1035489772),
        unk_440: -1,
        slide: 0,
        slide_count: 0,
        menu_open: false,
        rebuild_menu: false,
        progression,
        camera: 0,
        cam_kind: 0,
        entries: 0,
        flight: (Vec::new(), Vec::new()),
        flight_point: 0,
        // 0x00219690 first activate: VsVector4::Set(0, 1, 0, 1)
        up: [0.0, 1.0, 0.0, 1.0],
        up_prev: [0.0, 1.0, 0.0, 1.0],
        up_from: [0.0, 1.0, 0.0, 1.0],
        up_to: [0.0, 1.0, 0.0, 1.0],
        quit_dialog: false,
        started: false,
        exit_camera: false,
        exit_slide: false,
        exit_countdown: -1,
        exit_screen: 0,
    }));
    // 0x00218AF0 — AssignEvents
    fe.map(layer, functor(&obj, ho_enter), ev::ENTER);
    fe.map(layer, functor(&obj, ho_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, ho_start), ev::START);
    fe.map(layer, functor(&obj, ho_leave), ev::LEAVE);
    // 0x0021A8C0: mapped by Start and dropped by Leave in the original (`started` gates it here)
    fe.map(layer, functor(&obj, ho_update), ev::UPDATE);
    fe.map(layer, functor(&obj, |s, fe, _| ho_left_right(s, fe, 1)), ev::RIGHT_PRESS);
    fe.map(layer, functor(&obj, |s, fe, _| ho_left_right(s, fe, -1)), ev::LEFT_PRESS);
    fe.map(layer, functor(&obj, ho_up_down), ev::UP_PRESS);
    fe.map(layer, functor(&obj, ho_up_down), ev::DOWN_PRESS);
    fe.map(layer, functor(&obj, ho_select), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, ho_cancel), ev::CANCEL_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::OPTION2_PRESS); // 0x0021A260: online results only
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN); // 0x0021A2F0
    obj
}

/// FEOnlineMatchInfoScreen.cpp 0x00218F50 / FE2DOverlay.cpp 0x0027D340 — Enter: overlay / camera on the screen, the hangout's characters and
/// cars, the texts loaded and hidden the first time
fn ho_enter(s: &mut CFEHangoutScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.started = false;
    ov(fe, |o, fe| overlay::attach_to_screen(o, fe, Some(s.screen)));
    fe.foyer.snap_to_target();
    fe.garage.vehicle_flags = 8; // byte_6F90D0
    s.chars_created = false;
    if !fe.characters.loaded {
        fe.characters.loaded = true;
        fe.characters.foyer = "hangout".into();
        fe.characters.load_chars();
        fe.characters.load_crowd();
        fe.characters.load_dance();
    }
    if !fe.characters.created {
        fe.characters.create_elements();
        s.chars_created = true;
    }
    if s.chars_shown || s.chars_created {
        // 0x002609D0 the props, 0x0025F2A0 their world matrices
        fe.characters.create_props();
        fe.characters.place_props();
    }
    if !s.loaded {
        s.loaded = true;
        let name = fe.characters.chars.first().map_or(0, |c| c.name_id);
        ov(fe, |o, fe| fe.text(o.menu_title).set_string(name));
        let orange = visos_colour(128, 124, 75, 3);
        for i in 0..6 {
            fe.element_load(s.info[i]);
            fe.text(s.info[i]).set_colour(orange);
            fe.el(s.info[i]).set_visible(false);
        }
        for i in 0..3 {
            for id in [s.headings[i], s.race_info[i]] {
                fe.element_load(id);
                fe.el(id).set_visible(false);
            }
        }
        for i in 0..5 {
            fe.element_load(s.headings2[i]);
            fe.el(s.headings2[i]).set_visible(false);
        }
        fe.element_load(s.background);
        fe.el(s.background).set_visible(false);
        let mut p = std::mem::take(&mut s.progression);
        p.load(fe); // 0x0028C610
        s.progression = p;
    }
    fe.foyer_vehicles.set_foyer("hangout");
    fe.foyer_vehicles.load("Vehicles", false);
    crate::feck2::screens::startup::fe_main_menu_screen::show_hangout_cars(fe);
    s.progression.reset_steps();
    ov(fe, |o, _| o.flag534 = true); // 0x0027D340
    s.move_timer = s.move_time;
    true
}

/// FEOnlineMatchInfoScreen.cpp 0x00219690 — Activate: overlay (story heading, select / back, arrows, cash and
/// prestige), list style 1 with 5 rows, the hangout lights
fn ho_activate(s: &mut CFEHangoutScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    ov(fe, |o, fe| {
        overlay::reset(o, fe);
        overlay::set_nav_text(o, fe, overlay::NAV_SELECT, string_crc("GENERAL_SELECT"));
        overlay::set_nav_text(o, fe, overlay::NAV_CANCEL, string_crc("GENERAL_BACK"));
        overlay::show_arrows(o, fe, 0);
        overlay::show_arrows(o, fe, 1);
        overlay::show_borders(o, fe);
        overlay::show_cash(o, fe);
        overlay::show_prestige(o, fe);
        overlay::show_dialog(o, fe, 2, 0, &[]);
        overlay::set_heading(o, fe, string_crc("MAINMENU_STORY"));
        menu::set_style(o, fe, 1);
        menu::set_window(o, 5);
        fe.el(o.borders[overlay::ICON]).set_visible(false); // 0x0027AA90 ("FE_Icon_Hangout")
        overlay::enable(o);
    });
    s.previous = s.character;
    s.first_activate = false;
    fe.foyer.set_foyer("hangout");
    fe.foyer.show_background(true, true, 0);
    // 0x00219690 tail: Show(all) again — Activate runs after the previous screen's Leave, so the car on show
    // (FE3DVehicles +1824, copied into the current car by Select) carries what CarCustomisationScreen's Leave
    // (0x0024B340) just wrote into the garage slot. Then 0x0025E040(0, 28.0): slot 0's front wheels steered; the
    // 0x0025E7D0(0) height / wheel-change bounce is `show`'s y (the bounce, started by the unported part loading, is 0)
    crate::feck2::screens::startup::fe_main_menu_screen::show_hangout_cars(fe);
    let now = fe.stopwatch;
    fe.foyer_vehicles.set_steer(0, 28.0, now);
    true
}

/// FEOnlineMatchInfoScreen.cpp 0x00219300 / FE3DCamera.cpp 0x002728E0 / 0x00271F10 / 0x00271FA0 — Start: characters shown, camera on the current character, the
/// character's menu, the dialog hidden
fn ho_start(s: &mut CFEHangoutScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.move_time = f32::from_bits(1035489772);
    s.move_timer = s.move_time;
    // the update functor (0x0021A8C0) is mapped here in the original (`started` below)
    fe.foyer.show_background(true, true, 0); // 0x00273620(1, 1, 0)
    // chars shown / created: 0x002739F0 re-places the backgrounds (areas at `byte_769380`, zero) and 0x00273990 puts
    // each prop element on the camera's list (+65, the reflected one) — `foyer.rs` draws props on those layers
    // 0x002728E0(0): linear moves of +436, ending once less than a frame is left (+347)
    fe.foyer.move_mode = 1;
    fe.foyer.move_time = s.move_time;
    fe.foyer.end_early = true;
    fe.foyer.set_fov(f32::from_bits(1112014848)); // 50
    fe.characters.set_visible(true); // 0x00261510 / 0x002615F0 / 0x002616A0
    s.selection = 0; // +284
    s.entries = 6; // +468
    s.unk_272 = 0;
    s.unk_440 = 1;
    // 0x00271CC0 / 0x00271CF0 (+340 / +356), 0x00271E80 / 0x00271F10 / 0x00271FA0: the camera Leave left, at once
    // (ChangeCharacter(0) below snaps it on again)
    fe.foyer.snap_to([s.cam_pos2[0], s.cam_pos2[1], s.cam_pos2[2]], [s.cam_look2[0], s.cam_look2[1], s.cam_look2[2]]);
    change_character(s, fe, 0); // 0x0021B0D0(0): the character is kept (+276)
    build_menu(s, fe, false);
    s.rebuild_menu = false;
    if fe.results_prompt {
        ov(fe, |o, fe| overlay::set_nav_text(o, fe, 3, string_crc("GENERAL_RESULTS")));
    }
    ov(fe, |o, fe| overlay::set_dialog_visible(o, fe, false, 0));
    s.exit_countdown = -1;
    s.exit_screen = 0;
    s.camera = 5;
    s.slide = 0;
    ov(fe, |o, fe| {
        fe.el(o.borders[overlay::R_MENU_CONTENT]).set_visible(false); // overlay +36
        overlay::fade_arrows(o, fe, -128, 1, 0);
    });
    fe.music.push(FeMusicCall::StartLoopEvent210);
    s.started = true;
    s.chars_shown = true;
    if let Some(c) = fe.characters.chars.get(s.character.max(0) as usize) {
        fe.music.push(FeMusicCall::FadeMusic { to: c.volume, env: c.reverb, time: 1.0 });
    }
    if s.menu_open {
        open_menu(s, fe, true); // 0x0021B7D0(1), then 0x00271FA0 (the element takes the camera)
    } else {
        s.move_timer = s.move_time;
    }
    true
}

/// FEOnlineMatchInfoScreen.cpp 0x00219920 — Leave: the characters go when the screen is left for good
fn ho_leave(s: &mut CFEHangoutScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    // 0x00262380(+276, +464, 0) -> +292 / +308 / +324, copied to +340 / +356 / +372
    let (pos, look, up) = character_camera(fe, s.character, s.cam_kind, 0);
    s.cam_pos = pos;
    s.cam_look = look;
    s.up = up;
    s.cam_pos2 = s.cam_pos;
    s.cam_look2 = s.cam_look;
    s.up_prev = s.up;
    ov(fe, |o, _| o.flag534 = false); // 0x0027D340(0)
    if s.exit_screen != 0 {
        fe.music.push(FeMusicCall::FadeMusic { to: 1.0, env: f32::from_bits(0x3DCCCCCD), time: 1.0 });
    } else {
        s.chars_shown = false;
    }
    if s.chars_shown {
        fe.characters.set_visible(false);
        fe.characters.drop_props(); // 0x00261300
    }
    fe.music.push(FeMusicCall::StopLoopEvent210);
    // the update functor is unmapped (vtable +12 of event 21)
    s.started = false;
    true
}

/// FEHangoutScreen.cpp 0x0021A8C0 — Update: the tutorial start, the leave countdown, Tory's steps
pub fn ho_update(s: &mut CFEHangoutScreen, fe: &mut Feck2, e: &CEvent) -> bool {
    if !s.started {
        return true;
    }
    let dt = f32::from_bits(e.param_a);
    if !fe.career.completed_any && fe.career.profile_active && s.exit_screen == 0 {
        start_tutorial(s, fe);
    }
    s.move_timer -= dt;
    if s.move_timer > 0.0 {
        update_flight(s, fe);
    }
    if s.exit_countdown != -1 && s.exit_screen != 0 && s.move_timer <= 0.0 {
        if s.exit_countdown == 0 {
            s.exit_countdown = -1;
            let to = s.exit_screen;
            if to == string_crc("MainMenuScreen") {
                fe.goto_previous_screen(0, to);
            } else if fe.find_screen(to).is_some() {
                fe.goto_screen(to);
            } else {
                log::warn!("hangout: screen {to:08x} is not ported");
                s.exit_screen = 0;
            }
            return false;
        }
        s.exit_countdown -= 1;
    }
    if s.entries != 0 && s.slide != 0 && s.camera != 4 {
        // the menu slides with the arrows while it opens / closes
        let mut v = 2 * s.slide;
        if s.exit_screen != 0 {
            v = 4 * s.slide;
        }
        ov(fe, |o, fe| {
            overlay::fade_arrows(o, fe, -v, 0, 0);
            overlay::fade_arrows(o, fe, 3 * v, 1, 0);
            menu::fade(o, fe, v, 0);
        });
        s.slide_count += s.slide;
        if s.slide_count == 0 {
            let d = s.slide;
            ov(fe, |o, fe| menu::slide_border(o, fe, d));
            s.slide_count = -s.slide;
        }
    }
    // 0x0021AB15: FE3DVehicles 0x0025E040(0, 28.0) steer, 0x0025E7D0(0) height (no bounce running), +15692 = 1
    let now = fe.stopwatch;
    fe.foyer_vehicles.set_steer(0, 28.0, now);
    let mut p = std::mem::take(&mut s.progression);
    let r = if p.active {
        p.update_exit(fe)
    } else {
        if s.exit_screen == 0 {
            p.update(fe);
        }
        true
    };
    s.progression = p;
    r
}

/// FEHangoutScreen.cpp 0x0021A310 — the flight: the next point once the camera reached the last one, the menu
/// and arrows swapped halfway through a character change, the leave set up during the exit move
fn update_flight(s: &mut CFEHangoutScreen, fe: &mut Feck2) {
    let n = s.flight.0.len();
    if fe.foyer.move_left() > 0.0 {
        // +292 / +308: the character camera of the current kind (only read by the next flight's start)
    } else {
        s.flight_point += 1;
        if s.flight_point >= n.saturating_sub(1) {
            if s.camera != 4 {
                s.move_timer = 0.0;
            }
        } else {
            // CarDamage 0x000762F0: up = to + (from - to) * the flight time left (not a 0..1 fraction: original),
            // the camera's target up (0x00271DA0); the start up below gets the same value (0x00271DD0)
            let t = s.move_timer;
            let (a, b) = (s.up_to, s.up_from);
            s.up = std::array::from_fn(|k| (b[k] - a[k]) * t + a[k]);
            let v = |p: [f32; 4]| [p[0], p[1], p[2]];
            fe.foyer.set_target_up(v(s.up));
            s.up_prev = s.up;
            let i = s.flight_point;
            let (p, l) = (&s.flight.0, &s.flight.1);
            let to_l = l.get(i + 1).copied().unwrap_or(l[l.len() - 1]);
            fe.foyer.set_up(v(s.up_prev));
            fe.foyer.start_vector_move((v(p[i]), v(l[i.min(l.len() - 1)])), (v(p[i + 1]), v(to_l)));
        }
    }
    if s.camera > 1 {
        if s.camera == 4 {
            if s.exit_screen == string_crc("LoadingScreen") {
                s.exit_countdown = 1;
            } else {
                ov(fe, |o, fe| overlay::start_transition(o, fe));
            }
        }
    } else {
        let frac = s.flight_point as f64 / (n as f64 - 1.0);
        if 0.5 <= frac {
            ov(fe, |o, fe| {
                overlay::fade_arrows(o, fe, 4, 0, 0);
                menu::fade_title(o, fe, 4, 0);
            });
            if s.rebuild_menu {
                build_menu(s, fe, false);
                s.rebuild_menu = false;
            }
            s.cam_kind = 0;
        } else {
            ov(fe, |o, fe| {
                overlay::fade_arrows(o, fe, -4, 0, 0);
                menu::fade_title(o, fe, -4, 0);
            });
        }
    }
}

/// FEHangoutScreen.cpp 0x0021AD30 + the tail of 0x0021B0D0 / 0x0021AE70 — the splines sampled into the point
/// tables, the move timer = (points - 1) * `+436`, the camera free for the first step
fn start_flight(s: &mut CFEHangoutScreen, fe: &mut Feck2, pos: CCatMullRom, look: CCatMullRom, up: ([f32; 4], [f32; 4])) {
    s.up_prev = s.up;
    s.up_from = up.0;
    s.up_to = up.1;
    s.flight = (pos.out, look.out);
    s.flight_point = 0;
    s.move_timer = (s.flight.0.len() as f32 - 1.0) * s.move_time;
    fe.foyer.vmove = None;
}

/// FE3DCharacters.cpp 0x00262380 — the character's `n`-th camera of a kind (LOOKAT 0, MOVERIGHT 1, SELECT 2,
/// EXIT 3, ENTRY 4) as (position, look-at, up = CamAxis); the original asserts when there is none
pub(crate) fn character_camera(fe: &Feck2, character: i32, kind: u32, n: usize) -> ([f32; 4], [f32; 4], [f32; 4]) {
    let cams = fe.characters.chars.get(character.max(0) as usize).map(|c| c.cameras.as_slice()).unwrap_or(&[]);
    match cams.iter().filter(|k| k.kind == kind).nth(n) {
        Some(k) => (
            [k.pos[0], k.pos[1], k.pos[2], 1.0],
            [k.look_at[0], k.look_at[1], k.look_at[2], 1.0],
            [k.axis[0], k.axis[1], k.axis[2], 1.0],
        ),
        None => {
            log::error!("assert: 0 (FE3DCharacters.cpp 1130): character {character} has no camera kind {kind}");
            ([0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0])
        }
    }
}

/// FE3DCharacters.cpp 0x00262610 / 0x002626A0 — count of a camera kind, the first one's spline step (0.2 if none)
fn camera_count_step(fe: &Feck2, character: i32, kind: u32) -> (usize, f32) {
    let cams = fe.characters.chars.get(character.max(0) as usize).map(|c| c.cameras.as_slice()).unwrap_or(&[]);
    let n = cams.iter().filter(|k| k.kind == kind).count();
    let step = cams.iter().find(|k| k.kind == kind).map_or(0.2, |k| k.timer);
    (n, step)
}

/// FEHangoutScreen.cpp 0x0021AE70 — fly from one camera kind of the current character to another
/// (`dword_6C128C` = 0.15 spline step)
fn fly_between(s: &mut CFEHangoutScreen, fe: &mut Feck2, from: u32, to: u32, camera: u32) {
    s.camera = camera;
    let a = character_camera(fe, s.character, from, 0);
    let b = character_camera(fe, s.character, to, 0);
    let step = f32::from_bits(0x3E19999A);
    let (mut pos, mut look) = (CCatMullRom::new(step), CCatMullRom::new(step));
    pos.add(a.0);
    pos.add(b.0);
    pos.build();
    look.add(a.1);
    look.add(b.1);
    look.build();
    start_flight(s, fe, pos, look, (a.2, b.2));
}

/// FEHangoutScreen.cpp 0x0021C430 — the first event of a new career ("UT0EVENT1") starts by itself
fn start_tutorial(s: &mut CFEHangoutScreen, fe: &mut Feck2) {
    let crc = string_crc("UT0EVENT1");
    let Some(e) = fe.career.find_event(crc).cloned() else {
        log::error!("assert: g_FEGameVariables.m_pEvent != NULL && \"Unable to find event to start.\"");
        fe.career.completed_any = true; // asserted once
        return;
    };
    // event +0x48: its cut scene (off_685400[0] is "NONE", non-null, so it is queued too and fails to open)
    progression::play_cut_movie(fe, e.unlock_message);
    let mut p = std::mem::take(&mut s.progression);
    p.start_event(fe, crc);
    s.progression = p;
    s.exit_screen = string_crc("LoadingScreen");
    s.exit_countdown = 0;
    // byte_6F9058 = 1 (0x0021C618): core's flag, else `career_in` reads it back false and the tutorial restarts
    // after the race
    fe.career.completed_any = true;
    fe.career.requests.push(CareerRequest::SetCompletedAny);
}

/// FEHangoutScreen.cpp 0x0021B0D0 / FE2DOverlay.cpp 0x0027C670 — `ChangeCharacter(dir)`: the next / previous character (wrapping), camera on it
fn change_character(s: &mut CFEHangoutScreen, fe: &mut Feck2, dir: i32) {
    hide_info(s, fe);
    let count = fe.characters.chars.len() as i32;
    s.previous = s.character;
    s.character += dir;
    if s.character >= count {
        s.character = 0;
    } else if s.character < 0 {
        s.character = count - 1;
    }
    if s.previous == s.character {
        s.cam_pos2 = s.cam_pos;
        s.cam_look2 = s.cam_look;
        s.up_prev = s.up;
    } else {
        if let Some(c) = fe.characters.chars.get(s.character.max(0) as usize) {
            fe.music.push(FeMusicCall::FadeMusic { to: c.volume, env: c.reverb, time: 1.0 });
        }
        // 0x0027C670(0) arrows; +340 / +356 = the previous character's LOOKAT camera
        let (pos, look, _) = character_camera(fe, s.previous, 0, 0);
        s.cam_pos2 = pos;
        s.cam_look2 = look;
        fe.sounds.push(if dir >= 0 { GuiSound::WooshRight } else { GuiSound::WooshLeft });
    }
    s.rebuild_menu = true;
    s.selection = 0;
    // the character whose MOVERIGHT cameras the flight passes: the new one going left, the old one going right
    let via = match dir {
        -1 => {
            s.camera = 0;
            s.character
        }
        1 => {
            s.camera = 1;
            s.previous
        }
        _ => {
            s.camera = 5;
            s.character
        }
    };
    let (from, to) = if dir != 0 {
        (character_camera(fe, s.previous, 0, 0), character_camera(fe, s.character, 0, 0))
    } else {
        (character_camera(fe, s.character, s.cam_kind, 0), character_camera(fe, s.character, s.cam_kind, 0))
    };
    let (count, step) = camera_count_step(fe, via, 1);
    let (mut pos, mut look) = (CCatMullRom::new(step), CCatMullRom::new(step));
    pos.add(from.0);
    look.add(from.1);
    let movers: Vec<usize> = match s.camera {
        1 => (0..count).collect(),
        0 => (0..count).rev().collect(),
        _ => Vec::new(),
    };
    for i in movers {
        let c = character_camera(fe, via, 1, i);
        pos.add(c.0);
        look.add(c.1);
    }
    pos.add(to.0);
    look.add(to.1);
    pos.build();
    look.build();
    start_flight(s, fe, pos, look, (from.2, to.2));
    if dir == 0 {
        s.move_timer = 0.0;
        fe.foyer.snap_to([to.0[0], to.0[1], to.0[2]], [to.1[0], to.1[1], to.1[2]]);
    }
    s.slide = 0;
}

/// FEHangoutScreen.cpp 0x0021B750 — the info texts hidden (the selection kept)
fn hide_info(s: &mut CFEHangoutScreen, fe: &mut Feck2) {
    if !s.menu_open {
        s.selection = fe.overlay.as_ref().map_or(0, |o| o.menu_selected);
        for i in 0..6 {
            fe.el(s.info[i]).set_visible(false);
        }
    }
}

/// FEHangoutScreen.cpp 0x0021ABB0 / 0x0021BF00 / 0x0021C1A0 — the character's name as the menu title and, open, its entries
fn build_menu(s: &mut CFEHangoutScreen, fe: &mut Feck2, open: bool) {
    let name = fe.characters.chars.get(s.character.max(0) as usize).map_or(0, |c| c.name_id);
    ov(fe, |o, fe| {
        menu::clear(o, fe);
        fe.text(o.menu_title).set_string(name);
        fe.el(o.menu_title).set_visible(true);
    });
    if open {
        match s.character {
            1 => {
                // 0x0021BF00: customise only for a race car
                s.entries = 3;
                ov(fe, |o, fe| {
                    for id in ["GARAGE_CUSTOMISE", "GARAGE_PINKSLIPS", "EDITOR_ALLOYEDITOR"] {
                        menu::add_entry(o, fe, string_crc(id), true, false, 0);
                    }
                });
            }
            2 => {
                // 0x0021C1A0
                s.entries = 1;
                ov(fe, |o, fe| {
                    menu::add_entry(o, fe, string_crc("CARSHOWROOM_BUY"), true, false, 0);
                });
            }
            3 => {
                // 0x0021C2A0: one entry per playlist (fixed lists by name, soundtrack lists by their name text)
                s.entries = fe.playlist.list_count() as u32;
                let names: Vec<_> = (0..fe.playlist.list_count()).map(|i| fe.playlist.list_name(i)).collect();
                ov(fe, |o, fe| {
                    for n in names {
                        match n {
                            crate::feck2::playlist_link::ListName::Hash(h) => {
                                menu::add_entry(o, fe, h, true, false, 0);
                            }
                            crate::feck2::playlist_link::ListName::Text(t) => {
                                menu::add_text_entry(o, fe, &t, true);
                            }
                        }
                    }
                });
            }
            _ => {}
        }
    }
    let entries = s.entries;
    ov(fe, |o, fe| {
        if entries != 0 {
            menu::show(o, fe, true, !open);
        } else {
            menu::show(o, fe, false, false);
        }
    });
}

/// FEHangoutScreen.cpp 0x0021B7D0 — open / close the character's menu
fn open_menu(s: &mut CFEHangoutScreen, fe: &mut Feck2, open: bool) {
    let changed = s.menu_open != open;
    s.menu_open = open;
    build_menu(s, fe, open);
    if open {
        if changed {
            fly_between(s, fe, 0, 2, 2);
        }
        s.cam_kind = 2;
        s.slide = 1;
        s.slide_count = -1;
        ov(fe, |o, _| o.menu_slide = 0);
        fe.sounds.push(GuiSound::WooshIn);
    } else {
        if changed {
            fly_between(s, fe, 2, 0, 3);
        }
        s.cam_kind = 0;
        s.slide = -1;
        s.slide_count = 1;
        ov(fe, |o, _| o.menu_slide = 6);
        fe.sounds.push(GuiSound::WooshOut);
    }
}

/// FEOnlineMatchInfoScreen.cpp 0x00219B70 — up / down: the browser's highlighted entry, else the info texts
fn ho_up_down(s: &mut CFEHangoutScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if fe.overlay.as_ref().is_some_and(|o| o.dialog_active) {
        return true;
    }
    if s.progression.active {
        let mut p = std::mem::take(&mut s.progression);
        let r = p.selection_changed(fe);
        s.progression = p;
        return r;
    }
    // byte_7691BF is never set
    if !s.menu_open {
        return false;
    }
    hide_info(s, fe);
    true
}

/// FEOnlineMatchInfoScreen.cpp 0x00219AC0 / FEHangoutProgression.cpp 0x0028F410 — left / right: the next character when nothing is open
fn ho_left_right(s: &mut CFEHangoutScreen, fe: &mut Feck2, dir: i32) -> bool {
    if fe.overlay.as_ref().is_some_and(|o| o.dialog_active) {
        return false;
    }
    if s.progression.active {
        return false; // 0x0028F410: the offer's own left/right
    }
    if s.menu_open || s.move_timer > 0.0 || !s.started {
        return false;
    }
    change_character(s, fe, dir);
    true
}

/// FEOnlineMatchInfoScreen.cpp 0x00219CC0 — Select
fn ho_select(s: &mut CFEHangoutScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.camera == 4 || s.move_timer > 0.0 || !s.started {
        return false;
    }
    if s.quit_dialog || s.character != 0 {
        if s.quit_dialog {
            s.quit_dialog = false;
            let yes = fe.overlay.as_ref().is_some_and(|o| o.dialog_selection == 0);
            if yes {
                s.character = 0;
                s.previous = 0;
                s.first_activate = true;
                s.exit_screen = string_crc("MainMenuScreen");
                s.chars_shown = false;
                ov(fe, |o, fe| overlay::start_transition(o, fe));
                s.exit_countdown = EXIT_FRAMES;
            }
            ov(fe, |o, fe| overlay::set_dialog_visible(o, fe, false, 0));
            return true;
        }
        // 0x00219DAB: dword_6F724C = the car on show
        fe.career.requests.push(CareerRequest::CopyShownCar);
        if s.menu_open {
            let enabled = fe.overlay.as_ref().is_some_and(|o| o.menu_selected >= 0 && menu::entry_enabled(o, o.menu_selected as usize));
            if s.move_timer <= 0.0 && enabled {
                menu_select(s, fe);
            }
            return true;
        }
        fe.sounds.push(GuiSound::Accept);
        open_menu(s, fe, true);
        return false;
    }
    if s.progression.active {
        let mut p = std::mem::take(&mut s.progression);
        p.select(fe);
        let started = p.started;
        s.progression = p;
        if started {
            go_to_race(s, fe);
        }
    } else {
        fly_between(s, fe, 0, 2, 2);
        // 0x00262DB0: Tory's select anim (not ported)
        let mut p = std::mem::take(&mut s.progression);
        if p.invite_waiting() {
            p.answer_invite(fe);
        } else {
            p.open(fe, true);
            // 0x00219EC6: dword_6F724C = the car on show
            fe.career.requests.push(CareerRequest::CopyShownCar);
        }
        s.progression = p;
    }
    true
}

/// `byte_75EDB4` "Frontend.Display cheat screen" (bss 0; `fe_tweaks` edits it)
pub static DISPLAY_CHEAT_SCREEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// FEHangoutScreen.cpp 0x00219F70 — an event was started: the camera's exit move, then the loading screen; a race
/// without a record (DummyGameScreen 0x002092A0 false) goes to DummyGameScreen, and so does every event while the
/// "Display cheat screen" tweak (`byte_75EDB4`) is on (0x00219F8C, GotoScreen at once; here through the same
/// SelectEvent route). `dword_6F9BE8` is the pending event here (core sets `m_pEvent` only when the loading screen
/// asks for the race).
fn go_to_race(s: &mut CFEHangoutScreen, fe: &mut Feck2) {
    let dummy = string_crc("DummyGameScreen");
    let cheat = DISPLAY_CHEAT_SCREEN.load(std::sync::atomic::Ordering::Relaxed);
    if !cheat && fe.find_screen(dummy).is_none() {
        log::error!("assert: ds (FEHangoutScreen.cpp 898)");
    }
    let details = fe.career.start_pending.and_then(|c| fe.career.find_event(c)).and_then(|e| e.current_race.as_ref()?.details);
    let details = if cheat { None } else { details };
    let Some((_, pro)) = details else {
        // StartEvent's game part (core's `start_event`: m_pEvent, reset) now, as the loading screen won't ask for it;
        // the screen reads the event this frame, before core's view comes back
        if let Some(c) = fe.career.start_pending.take() {
            fe.career.event = fe.career.find_event(c).cloned();
            fe.career.requests.push(crate::feck2::screens::hangout::CareerRequest::SelectEvent { event: c });
        }
        fe.goto_screen(dummy);
        return;
    };
    fe.quick_race.pro = pro;
    fe.quick_race.amateur = !pro;
    fly_between(s, fe, 2, 3, 4);
    s.slide = -1;
    s.exit_screen = string_crc("LoadingScreen");
    ov(fe, |o, fe| overlay::fade_out_all(o, fe)); // 0x00279140
}

/// FEHangoutScreen.cpp 0x0021B910 / 0x0021C010 / 0x0021C210 / 0x0021C370 — a menu entry: the character's screen
fn menu_select(s: &mut CFEHangoutScreen, fe: &mut Feck2) {
    s.selection = fe.overlay.as_ref().map_or(0, |o| o.menu_selected);
    if (s.character, s.selection) == (1, 0) {
        // 0x0021C010: dword_6F90C4 = dword_6F71C4, dword_6F90C8 = dword_6F71C0
        fe.garage.custom_slot = fe.quick_race.garage_selected;
        fe.garage.custom_vehicle = fe.garage.current_vehicle;
    }
    if s.character == 2 {
        // 0x0021C210: byte_6F90D0 = 0, dword_6F90CC = dword_6F71C0 (the showroom starts on the current car)
        fe.garage.vehicle_flags = 0;
        fe.garage.showroom_vehicle = fe.garage.current_vehicle;
    }
    if s.character == 3 {
        fe.playlist.list = s.selection.max(0) as usize; // 0x0021C370 -> SetList (0x0022BB20)
    }
    let (screen, countdown, keep_chars) = match (s.character, s.selection) {
        (1, 0) => ("CarCustomisationScreen", EXIT_FRAMES, true), // 0x0021C010 (dword_6F90C4/C8 = current car)
        (1, 1) => ("PinkSlipScreen", EXIT_FRAMES, false),
        (1, 2) => ("AlloyEditorBasic", 1, true),
        (2, _) => ("CarShowroomScreen", EXIT_FRAMES, true), // 0x0021C210
        (3, _) => ("AudioPlaylistScreen", 1, false),        // 0x0021C370
        _ => return,
    };
    s.exit_screen = string_crc(screen);
    s.exit_countdown = countdown;
    s.chars_shown = keep_chars;
    if countdown == EXIT_FRAMES {
        ov(fe, |o, fe| overlay::start_transition(o, fe));
    }
    // +474 / +475 (set by the character's entry above): the exit move and the menu slide
    if keep_chars {
        fly_between(s, fe, 2, 3, 4);
        s.slide = -1;
    }
}

/// FEHangoutScreen.cpp 0x0021A100 — Cancel: close the menu / the offer, else the quit dialog
fn ho_cancel(s: &mut CFEHangoutScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.camera == 4 {
        return false;
    }
    if s.character != 0 || !s.progression.active {
        if s.quit_dialog || s.character != 0 || s.progression.active || !s.progression.invite_waiting() {
            if s.menu_open {
                open_menu(s, fe, false);
            } else if s.quit_dialog {
                ov(fe, |o, fe| overlay::set_dialog_visible(o, fe, false, 0));
                s.quit_dialog = false;
            } else {
                s.quit_dialog = true;
                ov(fe, |o, fe| {
                    overlay::show_dialog(o, fe, 2, QUIT_TITLE, &[GENERAL_YES, GENERAL_NO]);
                    overlay::set_dialog_visible(o, fe, true, 0);
                });
            }
        } else {
            let mut p = std::mem::take(&mut s.progression);
            p.cancel(fe);
            s.progression = p;
        }
    } else {
        let mut p = std::mem::take(&mut s.progression);
        p.cancel(fe);
        let active = p.active;
        s.progression = p;
        if !active {
            fly_between(s, fe, 2, 0, 3);
        }
    }
    true
}
