//! src\Feck2\Screens\Startup\FEMainMenuScreen.cpp — the start-up screens in this object file: main menu
//! (`gpMainMenuScreen`), legal (`gpLegalScreen`) and the intro movie screen (`gpMovieScreen`).
//!
//! the main menu sits on the `hangout` foyer (FE3DCamera state in `fe_3d_camera`, drawn by `foyer.rs`); the
//! character / vehicle calls into FE3DVehicles / FE3DCharacters are listed where they happen.

use std::sync::{Arc, Mutex};

use crate::feck2::feck2_manager::{visos_colour, Feck2, FrontendRequest, COLOUR_BLACK, COLOUR_GREY, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::string_manager::string_manager::string_crc;

// ------------------------------------------------------------------------------------------------------------
// main menu

/// `dword_766E90` (static init 0x0042DFD3).
pub const MAIN_MENU_ITEMS: [&str; 4] = ["MAINMENU_QUICKRACE", "MAINMENU_STORY", "MAINMENU_MULTIPLAYER", "GENERAL_OPTIONS"];

/// `dword_6C134C`: pulse step of "press start", flips sign at each end.
static PRESS_START_STEP: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1);

/// `CFEMainMenuScreen` (56 bytes)
pub struct CFEMainMenuScreen {
    pub mode: u32,                  // +0
    pub screen: ScreenId,           // +12
    pub layer: LayerId,             // +16
    pub items: [ElementId; 4],      // +20
    pub start_text: ElementId,      // +36
    pub loaded: bool,               // +40
    pub quick_race_ok: bool,        // +41
    pub selected: i32,              // +44
    pub countdown: i32,             // +48
    pub state: u32,                 // +52 0 press start, 1 profile, 2 sign in, 3 menu
}

// the main menu state `dword_6F6FD0` survives screen changes: `Feck2.main_menu_state`

/// FEMainMenuScreen.cpp 0x0023B370 — `FECK2CreateMainMenuScreen`
pub fn create_main_menu_screen(fe: &mut Feck2) -> Arc<Mutex<CFEMainMenuScreen>> {
    let obj = Arc::new(Mutex::new(CFEMainMenuScreen {
        mode: 0,
        screen: 0,
        layer: 0,
        items: [0; 4],
        start_text: 0,
        loaded: false,
        quick_race_ok: false,
        selected: 0,
        countdown: 0,
        state: 0,
    }));
    init_main_menu(&obj, fe);
    obj
}

/// FEMainMenuScreen.cpp 0x0023B4A0 — `Init`
fn init_main_menu(obj: &Arc<Mutex<CFEMainMenuScreen>>, fe: &mut Feck2) {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("MainMenuScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 2, crc: string_crc("MainMenu_FGLayer"), ..Default::default() });
    obj.lock().unwrap().layer = layer;
    bind_main_menu(obj, fe);
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 2, crc: string_crc("mainmenutext"), ..Default::default() };
    d.position[0] = 256.0;
    let mut items = [0; 4];
    for (i, item) in items.iter_mut().enumerate() {
        d.position[1] = (20 * i as u32 + 230) as f32;
        d.string = string_crc(MAIN_MENU_ITEMS[i]);
        *item = fe.create_element(layer, &d);
    }
    d.kind = ELEMENT_STATIC_TEXT;
    d.font = 4;
    d.crc = string_crc("startText");
    d.position[0] = 256.0;
    d.position[1] = 260.0;
    d.string = string_crc("STARTSCREEN_PRESSTOSTART");
    let start_text = fe.create_element(layer, &d);
    let mut s = obj.lock().unwrap();
    s.screen = screen;
    s.layer = layer;
    s.items = items;
    s.start_text = start_text;
    s.selected = 0;
    s.state = 0;
    s.mode = 0;
}

/// FEMainMenuScreen.cpp 0x0023BA10 — `AssignEvents` (the layer's `SLayerDesc::+20` functor)
pub fn bind_main_menu(obj: &Arc<Mutex<CFEMainMenuScreen>>, fe: &mut Feck2) {
    let layer = obj.lock().unwrap().layer;
    fe.map(layer, functor(obj, mm_enter), ev::ENTER);
    fe.map(layer, functor(obj, mm_activate), ev::ACTIVATE);
    fe.map(layer, functor(obj, mm_leave), ev::LEAVE);
    fe.map(layer, functor(obj, mm_start), ev::START);
    fe.map(layer, functor(obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(obj, mm_update), ev::UPDATE);
    fe.map(layer, functor(obj, mm_up), ev::UP_PRESS);
    fe.map(layer, functor(obj, mm_down), ev::DOWN_PRESS);
    fe.map(layer, functor(obj, |_, _, _| false), ev::LEFT_PRESS);
    fe.map(layer, functor(obj, |_, _, _| false), ev::RIGHT_PRESS);
    fe.map(layer, functor(obj, mm_select), ev::SELECT_PRESS);
    fe.map(layer, functor(obj, |_, _, _| false), ev::CANCEL_PRESS);
    fe.map(layer, functor(obj, mm_pause), ev::PAUSE_PRESS);
}

/// FEMainMenuScreen.cpp 0x0023BED0 / FE3DVehicles.cpp 0x0025F3E0 — Enter
fn mm_enter(s: &mut CFEMainMenuScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    s.countdown = -1;
    // FE3DCamera: SetScreen(screen, 1), fade 1.0, enable characters / vehicles / lights (3D foyer, not ported)
    if let Some(mut o) = fe.overlay.take() {
        overlay::attach_to_screen(&mut o, fe, Some(s.screen));
        fe.overlay = Some(o);
    }
    if !s.loaded {
        s.loaded = true;
        for i in 0..4 {
            fe.element_load(s.items[i]);
            fe.text(s.items[i]).set_justification(2);
        }
        fe.element_load(s.start_text);
        fe.text(s.start_text).set_justification(2);
        let orange = visos_colour(128, 120, 77, 14);
        fe.text(s.start_text).set_colour(orange);
        fe.text(s.start_text).set_colour2(orange);
        s.mode = 3;
    }
    // hangout vehicles and characters (0x0025F3E0, 0x0025F440 / 0x0025FD60 / 0x00260250, then the hidden
    // character elements 0x00260710.. and the cars 0x0025F0C0)
    if !fe.characters.loaded {
        fe.characters.loaded = true;
        fe.foyer_vehicles.set_foyer("hangout");
        fe.foyer_vehicles.load("Vehicles", false);
        fe.characters.foyer = "hangout".into();
        fe.characters.load_chars();
        fe.characters.load_crowd();
        fe.characters.load_dance();
    }
    if !fe.characters.created {
        fe.characters.create_elements();
        show_hangout_cars(fe);
    }
    // the car select repoints the vehicle file at its own foyer; read the hangout's again on the way back
    if fe.foyer_vehicles.foyer != "hangout" {
        fe.foyer_vehicles.set_foyer("hangout");
        fe.foyer_vehicles.load("Vehicles", false);
        show_hangout_cars(fe);
    }
    true
}

/// FE3DVehicles.cpp 0x0025F0C0 — put the hangout's cars up
pub fn show_hangout_cars(fe: &mut Feck2) {
    let Some(db) = fe.vehicles.take() else { return };
    let sel = [fe.quick_race.selected_vehicle, fe.quick_race.selected_vehicle];
    let garage = fe.quick_race.garage.clone();
    fe.foyer_vehicles.show(None, &db, sel, &garage, fe.quick_race.garage_selected as usize, &fe.garage);
    fe.vehicles = Some(db);
}

/// FEMainMenuScreen.cpp 0x0023C0D0 — Activate
fn mm_activate(s: &mut CFEMainMenuScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(mut o) = fe.overlay.take() {
        overlay::reset(&mut o, fe);
        overlay::set_hide_options(&mut o, fe, true);
        overlay::set_nav_text(&mut o, fe, overlay::NAV_SELECT, string_crc("GENERAL_SELECT"));
        overlay::show_borders(&mut o, fe);
        if s.state == 0 {
            overlay::set_2d_background(&mut o, fe, "white", "\\FE\\FE.vcd", true, COLOUR_BLACK);
        }
        overlay::enable(&mut o);
        fe.overlay = Some(o);
    }
    fe.foyer.set_foyer("hangout");
    true
}

/// FEMainMenuScreen.cpp 0x0023C1A0 — Leave
fn mm_leave(_s: &mut CFEMainMenuScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    // FontHelper 0x000B3470: g_AttractMode idle timer off
    if let Some(a) = fe.attract.mode.as_mut() {
        a.stop_timer();
    }
    true
}

/// FontHelper 0x000B34A0 on `g_AttractMode` (Up / Down / Select): idle again
fn attract_reset_timer(fe: &mut Feck2) {
    if let Some(a) = fe.attract.mode.as_mut() {
        a.reset_timer();
    }
}

/// FontHelper 0x000B30E0 (`g_AttractMode` front end update); a start runs 0x000B3550 -> GameVars 0x000C2810
/// (`CGameJump::Set` + `Execute(screen)`). The movies (0x000B3620, In001..In003) go to `fe.movies`.
fn attract_front_end_update(fe: &mut Feck2, dt: f32, screen: u32) {
    let link = &mut fe.attract;
    let Some(a) = link.mode.as_mut() else { return };
    let start = a.front_end_update(dt, link.seed, &mut link.rand);
    link.started |= start.is_some();
    match start {
        // 0x000B3620: GameManager__BE9F0 "In001", "In002", "In003"
        Some(velocity_game::font_helper::AttractStart::Movies) => {
            log::info!("attract mode: intro movies In001..In003");
            fe.movies.extend(["In001", "In002", "In003"].map(String::from));
        }
        Some(velocity_game::font_helper::AttractStart::Race(r)) => {
            log::info!("attract mode: {}{} (wet {})", r.track, r.race, r.wet);
            link.jump = Some(r);
            link.jump_active = true;
            game_jump_execute(fe, screen);
        }
        None => {}
    }
}

/// GameVars.cpp 0x000C2860 — `CGameJump::Execute(screen)`, front end part: the event itself is core's (the plugin
/// hands the jump over), then GotoScreen("LoadingScreen") and GameManager 0x000BE7C0 (the screen after the race)
fn game_jump_execute(fe: &mut Feck2, screen: u32) {
    fe.attract.jump_active = false;
    fe.attract.loading_jump = true;
    fe.goto_screen_name("LoadingScreen");
    fe.post_race_screen = screen;
}

/// FEMainMenuScreen.cpp 0x0023C1D0 — Start
fn mm_start(s: &mut CFEMainMenuScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    // 0x0023C1FD: the hangout background alone
    fe.foyer.show_background(true, true, 0);
    s.countdown = -1;
    s.state = fe.main_menu_state;
    // camera: snap to the far end on the first visit, then glide to the state's position
    if s.state == 0 {
        fe.foyer.snap(0);
    }
    let cam = match s.state {
        0 => 1,
        1 | 2 => 2,
        _ => 3,
    };
    fe.foyer.move_to(cam);
    if let Some(mut o) = fe.overlay.take() {
        overlay::fade_2d_background(&mut o, fe, 0, 75);
        fe.overlay = Some(o);
    }
    main_menu_move_selection(s, fe, 0);
    s.quick_race_ok = true;
    fe.requests.push(FrontendRequest::SetControllerMask { player: 0, mask: 0xFFFF });
    fe.requests.push(FrontendRequest::SetControllerMask { player: 1, mask: 0 });
    if s.state == 2 {
        s.countdown = 1;
    }
    true
}

/// FEMainMenuScreen.cpp 0x0023C400 — Update
fn mm_update(s: &mut CFEMainMenuScreen, fe: &mut Feck2, e: &CEvent) -> bool {
    // security file check (\F3_SEC.SEC / SECUR427) only runs when byte_74B60C is set: not in this build's flow
    if fe.next_screen_crc != 0 {
        let crc = fe.next_screen_crc;
        fe.next_screen_crc = 0;
        s.state = 3;
        fe.main_menu_state = s.state;
        fe.goto_screen(crc);
        return true;
    }
    let in_menu = s.state == 3;
    for i in 0..4 {
        fe.el(s.items[i]).set_visible(in_menu);
    }
    fe.el(s.start_text).set_visible(!in_menu);
    if !in_menu {
        let step = PRESS_START_STEP.load(std::sync::atomic::Ordering::Relaxed);
        if !fe.element_fade(s.start_text, 2 * step, 0, 0) {
            PRESS_START_STEP.store(-step, std::sync::atomic::Ordering::Relaxed);
        }
    }
    if s.state == 2 {
        fe.el(s.start_text).set_visible(false);
    }
    // 0x0023C663: g_AttractMode front end update (0x000B30E0), "MainMenuScreen" the screen after its race
    attract_front_end_update(fe, f32::from_bits(e.param_a), string_crc("MainMenuScreen"));
    if fe.attract.jump_active {
        // GameVars 0x000C2740 -> CGameJump::Execute("MainMenuScreen")
        game_jump_execute(fe, string_crc("MainMenuScreen"));
        return true;
    }
    if s.countdown == -1 {
        main_menu_move_selection(s, fe, 0);
    } else {
        s.countdown -= 1;
        if s.countdown == 0 {
            if s.state == 1 {
                s.state += 1;
                fe.main_menu_state = s.state;
                fe.goto_screen_name("ProfileSelectScreen");
            } else if s.state == 2 {
                // online sign-in check (XOnline, not ported): stays offline
                s.state += 1;
                fe.main_menu_state = s.state;
            } else if s.selected == 1 {
                fe.goto_screen_name("HangoutScreen");
            }
        }
    }
    true
}

/// FEMainMenuScreen.cpp 0x0023C8B0 — Up
fn mm_up(s: &mut CFEMainMenuScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    attract_reset_timer(fe);
    main_menu_move_selection(s, fe, -1);
    true
}

/// FEMainMenuScreen.cpp 0x0023C8F0 — Down
fn mm_down(s: &mut CFEMainMenuScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    attract_reset_timer(fe);
    main_menu_move_selection(s, fe, 1);
    true
}

/// FEMainMenuScreen.cpp 0x0023C970 — Start button
fn mm_pause(s: &mut CFEMainMenuScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.state != 3 {
        for i in 0..4 {
            fe.el(s.items[i]).set_visible(true);
        }
        fe.el(s.start_text).set_visible(false);
        s.state += 1;
        fe.main_menu_state = s.state;
        if fe.next_screen_crc != 0 {
            s.state = 3;
            fe.main_menu_state = 3;
            let crc = fe.next_screen_crc;
            fe.goto_screen(crc);
            return true;
        }
        if s.state == 1 {
            s.state += 1;
            fe.main_menu_state = s.state;
            fe.goto_screen_name("ProfileSelectScreen");
        }
    }
    false
}

/// FEMainMenuScreen.cpp 0x0023CAC0 — Select
fn mm_select(s: &mut CFEMainMenuScreen, fe: &mut Feck2, e: &CEvent) -> bool {
    attract_reset_timer(fe);
    if s.state == 3 {
        if s.countdown != -1 {
            return false;
        }
        match s.selected {
            0 => {
                if s.quick_race_ok {
                    main_menu_quick_race(s, fe, e);
                }
            }
            1 => {
                // FE2DOverlay transition start (0x0027A030) then a 20 frame countdown to the hangout
                if let Some(mut o) = fe.overlay.take() {
                    o.transition_state = 1;
                    o.transition_timer = 0;
                    fe.element_set_position(o.transition_img[0], f32::from_bits(1124728832), f32::from_bits(1126170624), 0.0);
                    fe.overlay = Some(o);
                }
                s.countdown = 20;
                for i in 0..4 {
                    fe.element_fade(s.items[i], 0, s.countdown, 0);
                }
            }
            2 => fe.goto_screen_name("MultiplayerScreen"),
            3 => fe.goto_screen_name("OptionsScreen"),
            _ => {}
        }
        return true;
    }
    for i in 0..4 {
        fe.el(s.items[i]).set_visible(true);
    }
    if fe.next_screen_crc != 0 {
        s.state = 3;
        fe.main_menu_state = 3;
        let crc = fe.next_screen_crc;
        fe.goto_screen(crc);
        return true;
    }
    fe.el(s.start_text).set_visible(false);
    s.state += 1;
    fe.main_menu_state = s.state;
    s.countdown = 20;
    // camera moves to the profile position (FE3DCamera, not ported)
    false
}

/// FEMainMenuScreen.cpp 0x0023CD30 — `MoveSelection`: wraps 0..3, colours the items.
fn main_menu_move_selection(s: &mut CFEMainMenuScreen, fe: &mut Feck2, delta: i32) {
    s.selected += delta;
    if s.selected >= 0 {
        if s.selected == 4 {
            s.selected = 0;
        }
    } else {
        s.selected = 3;
    }
    for i in 0..4 {
        let enabled = if !s.quick_race_ok { i != 0 } else { true };
        let c = if i as i32 == s.selected {
            if enabled { visos_colour(128, 120, 77, 14) } else { visos_colour(128, 40, 24, 0) }
        } else if enabled {
            COLOUR_GREY
        } else {
            visos_colour(128, 32, 32, 32)
        };
        fe.text(s.items[i]).set_colour(c);
    }
}

/// FEMainMenuScreen.cpp 0x0023CE80 — `StartQuickRace`: bind the pad that pressed select, go to the text screen.
fn main_menu_quick_race(_s: &mut CFEMainMenuScreen, fe: &mut Feck2, e: &CEvent) {
    // UserInput::GetPressingDevice(MenuSelect): the controller that raised the event
    let mask = if e.controller >= 0 { 1u32 << e.controller } else { 0xFFFF };
    fe.requests.push(FrontendRequest::SetControllerMask { player: 0, mask });
    fe.requests.push(FrontendRequest::SetControllerMask { player: 1, mask: !mask });
    // byte_6F7157 = 0 (one player), byte_6F70D8 = 1, dword_6F5C00 = 1 (core's player count, from `two_players`)
    fe.quick_race.two_players = false;
    // FECarShowroomScreen::SetNextScreen(0, "QuickRaceStartScreen")
    fe.car_select_next = Some((0, string_crc("QuickRaceStartScreen")));
    fe.goto_screen_name("QuickRaceTextScreen");
}

// ------------------------------------------------------------------------------------------------------------
// legal

/// `off_6C1350`
pub const LEGAL_TEXTURES: [&str; 3] = ["Legal1", "Legal2", "Legal3"];

/// `CFELegalScreen` (100 bytes)
pub struct CFELegalScreen {
    pub this: std::sync::Weak<Mutex<CFELegalScreen>>,
    pub title: ElementId,          // +0
    pub text: [ElementId; 16],     // +4
    pub screen: ScreenId,          // +68
    pub layer: LayerId,            // +72
    pub bg: ElementId,             // +76
    pub page: u32,                 // +84
    pub loaded: bool,              // +88
    pub page_time: f32,            // +92
    pub total_time: f32,           // +96
}

/// FEMainMenuScreen.cpp 0x0023D1D0 — `FECK2CreateLegalScreen`
pub fn create_legal_screen(fe: &mut Feck2) -> Arc<Mutex<CFELegalScreen>> {
    let obj = Arc::new(Mutex::new(CFELegalScreen {
        this: std::sync::Weak::new(),
        title: 0,
        text: [0; 16],
        screen: 0,
        layer: 0,
        bg: 0,
        page: 0,
        loaded: false,
        page_time: 0.0,
        total_time: 0.0,
    }));
    // FEMainMenuScreen.cpp 0x0023D340 — Init
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("LegalScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("LegalScreen_Layer"), ..Default::default() });
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, crc: string_crc("LegalTitle"), font: 3, ..Default::default() };
    d.position[0] = 0.0;
    d.position[1] = 0.0;
    d.string = string_crc("GENERAL_BLANK");
    let title = fe.create_element(layer, &d);
    let mut text = [0; 16];
    for t in text.iter_mut() {
        d.kind = ELEMENT_STATIC_TEXT;
        d.crc = string_crc("LegalText");
        d.font = 1;
        d.position[0] = 0.0;
        d.position[1] = 0.0;
        d.string = string_crc("GENERAL_BLANK");
        *t = fe.create_element(layer, &d);
    }
    d.kind = ELEMENT_2D;
    d.crc = string_crc("bg");
    d.position = [0.0, 0.0, 64000.0, d.position[3]];
    d.f76 = 64.0;
    d.dynamic = true;
    d.texture = LEGAL_TEXTURES[0].into();
    d.vcd = "\\FE\\LOADSCR.VCD".into();
    let bg = fe.create_element(layer, &d);
    {
        let mut s = obj.lock().unwrap();
        s.screen = screen;
        s.layer = layer;
        s.title = title;
        s.text = text;
        s.bg = bg;
        s.page = 0;
        s.this = Arc::downgrade(&obj);
    }
    // FEMainMenuScreen.cpp 0x0023D7A0 — AssignEvents
    fe.map(layer, functor(&obj, |_, _, _| false), ev::PRELOAD);
    fe.map(layer, functor(&obj, legal_enter), ev::ENTER);
    fe.map(layer, functor(&obj, legal_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, legal_leave), ev::LEAVE);
    fe.map(layer, functor(&obj, legal_start), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, legal_skip), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::CANCEL_PRESS);
    obj
}

/// FEMainMenuScreen.cpp 0x0023DB00 — Enter
fn legal_enter(s: &mut CFELegalScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(mut o) = fe.overlay.take() {
        overlay::attach_to_screen(&mut o, fe, None);
        fe.overlay = Some(o);
    }
    if !s.loaded {
        fe.element_load(s.bg);
        for i in 0..16 {
            fe.element_load(s.text[i]);
            fe.text(s.text[i]).set_box(450.0, 512.0);
        }
        fe.element_load(s.title);
        s.loaded = true;
    }
    s.total_time = 0.0;
    false
}

/// FEMainMenuScreen.cpp 0x0023DBF0 — Activate: show the background and start the update functor.
fn legal_activate(s: &mut CFELegalScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    fe.el(s.bg).set_visible(true);
    if let Some(obj) = s.this.upgrade() {
        fe.map(s.layer, functor(&obj, legal_update), ev::UPDATE);
    }
    false
}

/// FEMainMenuScreen.cpp 0x0023DC90 — Leave
fn legal_leave(s: &mut CFELegalScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    fe.e2d(s.bg).change_texture_vcd("\\FE\\FE.vcd");
    fe.change_texture(s.bg, "white", true);
    false
}

/// FEMainMenuScreen.cpp 0x0023DD00 — Start: black 512x512 backdrop, first page.
fn legal_start(s: &mut CFELegalScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    fe.e2d(s.bg).change_texture_vcd("\\FE\\FE.vcd");
    fe.change_texture(s.bg, "white", true);
    {
        let b = fe.e2d(s.bg);
        b.set_size(512, 512);
        b.set_colour(COLOUR_BLACK, false, COLOUR_WHITE);
    }
    legal_show_page(s, fe);
    false
}

/// FEMainMenuScreen.cpp 0x0023DDE0 — Update: 7 seconds per page, three pages, then the main menu.
fn legal_update(s: &mut CFELegalScreen, fe: &mut Feck2, e: &CEvent) -> bool {
    let dt = e.dt();
    s.page_time += dt;
    s.total_time += dt;
    if s.page_time > 7.0 {
        if s.page >= 3 {
            fe.goto_screen_name("MainMenuScreen");
            return true;
        }
        legal_show_page(s, fe);
        s.total_time = 0.0;
        s.page_time = 0.0;
    }
    true
}

/// FEMainMenuScreen.cpp 0x0023DEC0 — Select: skips a page after 5 seconds.
fn legal_skip(s: &mut CFELegalScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if s.total_time < 5.0 {
        return true;
    }
    if s.page + 1 < 3 {
        s.page += 1;
        fe.change_texture(s.bg, LEGAL_TEXTURES[s.page as usize], true);
        s.total_time = 0.0;
        s.page_time = 0.0;
        true
    } else {
        fe.goto_screen_name("MainMenuScreen");
        false
    }
}

/// FEMainMenuScreen.cpp 0x0023DFD0 — `ShowPage`
fn legal_show_page(s: &mut CFELegalScreen, fe: &mut Feck2) {
    fe.el(s.title).set_visible(false);
    for i in 0..16 {
        fe.el(s.text[i]).set_visible(false);
    }
    match s.page {
        0 => legal_page_info(s, fe),
        1 => legal_page_manufacturers(s, fe),
        2 => legal_page_safety(s, fe),
        _ => {}
    }
    s.page += 1;
}

/// FEMainMenuScreen.cpp 0x0023E080 — page 1: LEGAL_INFO_1..4 and PLATFORM_LEGALINFO_1..2.
fn legal_page_info(s: &mut CFELegalScreen, fe: &mut Feck2) {
    let mut y = 70.0f32;
    let mut n = 0usize;
    for i in 0..4 {
        let id = s.text[n];
        fe.text(id).set_string(string_crc(&format!("LEGAL_INFO_{}", i + 1)));
        fe.element_set_position(id, 42.0, y, 0.0);
        fe.el(id).set_visible(true);
        let h = fe.text_height(id) as f32;
        y = (y + h) + 5.0;
        n += 1;
    }
    y += 20.0;
    for i in 0..2 {
        let id = s.text[n];
        fe.text(id).set_string(string_crc(&format!("PLATFORM_LEGALINFO_{}", i + 1)));
        fe.element_set_position(id, 42.0, y, 0.0);
        fe.el(id).set_visible(true);
        // original bug: the height is read from text[i], not the element just placed
        let h = fe.text_height(s.text[i]) as f32;
        y = (y + h) + 5.0;
        n += 1;
    }
}

/// FEMainMenuScreen.cpp 0x0023E300 — page 2: LEGAL_MANUFACTURER_1..16.
fn legal_page_manufacturers(s: &mut CFELegalScreen, fe: &mut Feck2) {
    let mut y = 70.0f32;
    for i in 0..16 {
        let id = s.text[i];
        fe.text(id).set_string(string_crc(&format!("LEGAL_MANUFACTURER_{}", i + 1)));
        let h = fe.text_height(id) as f32;
        fe.element_set_position(id, 42.0, y, 0.0);
        y += h;
        fe.el(id).set_visible(true);
    }
}

/// FEMainMenuScreen.cpp 0x0023E450 — page 3: LEGAL_SAFETY_TITLE and LEGAL_SAFETY_1..3.
fn legal_page_safety(s: &mut CFELegalScreen, fe: &mut Feck2) {
    let mut y = 70.0f32;
    fe.text(s.title).set_string(string_crc("LEGAL_SAFETY_TITLE"));
    fe.element_set_position(s.title, 42.0, y, 0.0);
    fe.el(s.title).set_visible(true);
    y += 50.0;
    for i in 0..3 {
        let id = s.text[i];
        fe.text(id).set_string(string_crc(&format!("LEGAL_SAFETY_{}", i + 1)));
        let h = fe.text_height(id) as f32;
        fe.element_set_position(id, 42.0, y, 0.0);
        y += h;
        fe.el(id).set_visible(true);
    }
}

// ------------------------------------------------------------------------------------------------------------
// movie

/// `CFEMovieScreen` (16 bytes)
pub struct CFEMovieScreen {
    pub screen: ScreenId, // +0
    pub layer: LayerId,   // +4
    pub flag: bool,       // +12
}

/// FEMainMenuScreen.cpp 0x0023E760 / 0x0023EDC0 — `FECK2CreateMovieScreen` (+ the Enter functor)
pub fn create_movie_screen(fe: &mut Feck2) -> Arc<Mutex<CFEMovieScreen>> {
    let obj = Arc::new(Mutex::new(CFEMovieScreen { screen: 0, layer: 0, flag: false }));
    // FEMainMenuScreen.cpp 0x0023E890 — Init
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("MovieScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 4, crc: string_crc("MovieScreen_Layer"), ..Default::default() });
    {
        let mut s = obj.lock().unwrap();
        s.screen = screen;
        s.layer = layer;
    }
    // FEMainMenuScreen.cpp 0x0023EA60 — AssignEvents
    fe.map(layer, functor(&obj, |_, _, _| false), ev::PRELOAD);
    fe.map(layer, functor(&obj, |s: &mut CFEMovieScreen, _, _| {
        // 0x0023EDC0
        if !s.flag {
            s.flag = true;
        }
        false
    }), ev::ENTER);
    fe.map(layer, functor(&obj, movie_activate), ev::ACTIVATE);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::LEAVE);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::START);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::SELECT_PRESS);
    fe.map(layer, functor(&obj, |_, _, _| false), ev::CANCEL_PRESS);
    obj
}

/// FEMainMenuScreen.cpp 0x0023EDF0 / 0x0023EF10 — Activate (+ the Update functor it maps): play the intro movies, then the legal screen next update.
fn movie_activate(s: &mut CFEMovieScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    fe.requests.push(FrontendRequest::PlayIntroMovies);
    let layer = s.layer;
    // 0x0023EF10 — Update: GotoScreen("LegalScreen")
    fe.layers[layer].map_event(
        Arc::new(|fe: &mut Feck2, _e: &CEvent| {
            fe.goto_screen_name("LegalScreen");
            true
        }),
        ev::UPDATE,
        false,
    );
    false
}
