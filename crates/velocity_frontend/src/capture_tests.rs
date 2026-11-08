//! headless captures written to `target/fe_captures/*.png` for visual checks.

use crate::font_helper::font_helper::*;
use crate::fonts::font_manager::*;
use crate::render2d::*;

pub fn out_dir() -> std::path::PathBuf {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/fe_captures");
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn capture_fonts() {
    if !visos::file::game_dir().exists() {
        return;
    }
    let mut tex = TextureStore::default();
    let mut fh = CFontHelper::default();
    assert!(fh.load_frontend_fonts(&mut tex));
    let mut dl = DrawList::default();
    let mut y = 10.0;
    for id in [FONT_VELOCITY10, FONT_FE11, FONT_FE12, FONT_FE14, FONT_FE_TITLE, FONT_DEBUG] {
        let f = fh.font(id).unwrap();
        let s = wide("The quick brown fox jumps over the lazy dog 0123456789 AVAWAT");
        f.draw_string_rect(10.0, y, 630.0, y + 60.0, &s, WHITE, FONT_WORDWRAP, None, &tex, &mut dl);
        y += f.line_height as f32 * 2.0 + 8.0;
    }
    let f = fh.font(FONT_FE14).unwrap();
    f.draw_string_rect(320.0, 300.0, 620.0, 470.0, &wide("Centred wrapped text in a 300 pixel wide box, checking word wrap and centring."), 0xFFFFC000, FONT_WORDWRAP | FONT_CENTRE_X | FONT_CENTRE_Y, None, &tex, &mut dl);
    let img = dl.rasterise(&tex, 640, 480);
    crate::vtf::write_png(&out_dir().join("fonts.png"), &img).unwrap();
}

use crate::feck2::feck2_manager::{action, FeInput};
use crate::string_manager::string_manager::string_crc;

fn press(a: usize) -> FeInput {
    let mut i = FeInput::default();
    i.pads[0][a].value = 1.0;
    i.pads[0][a].changed = true;
    i
}

fn shot(f: &crate::frontend::Frontend, name: &str) {
    let img = f.fe.draw.rasterise(&f.fe.textures, 640, 480);
    crate::vtf::write_png(&out_dir().join(name), &img).unwrap();
}

fn current_name(f: &crate::frontend::Frontend) -> u32 {
    f.fe.current.map(|c| f.fe.screens[c].crc).unwrap_or(0)
}

#[test]
fn capture_startup_flow() {
    if !visos::file::game_dir().exists() {
        return;
    }
    let mut f = crate::frontend::Frontend::new(Default::default());
    assert!(f.fe.start(string_crc("MovieScreen")));
    let idle = FeInput::default();
    let dt = 1.0 / 60.0;
    for _ in 0..3 {
        f.frame(&idle, dt);
    }
    assert_eq!(current_name(&f), string_crc("LegalScreen"));
    assert!(f.fe.requests.contains(&crate::feck2::feck2_manager::FrontendRequest::PlayIntroMovies));
    for _ in 0..10 {
        f.frame(&idle, dt);
    }
    shot(&f, "legal_page1.png");
    // 7.0 s per page
    for _ in 0..(7 * 60 + 5) {
        f.frame(&idle, dt);
    }
    shot(&f, "legal_page2.png");
    for _ in 0..(7 * 60 + 5) {
        f.frame(&idle, dt);
    }
    shot(&f, "legal_page3.png");
    for _ in 0..(7 * 60 + 5) {
        f.frame(&idle, dt);
    }
    assert_eq!(current_name(&f), string_crc("MainMenuScreen"));
    for _ in 0..30 {
        f.frame(&idle, dt);
    }
    shot(&f, "main_menu_press_start.png");
    f.frame(&press(action::PAUSE), dt);
    for _ in 0..40 {
        f.frame(&idle, dt);
    }
    shot(&f, "main_menu_after_start.png");
}

fn run(f: &mut crate::frontend::Frontend, n: usize) {
    let idle = FeInput::default();
    for _ in 0..n {
        f.frame(&idle, 1.0 / 60.0);
    }
}


fn tap(f: &mut crate::frontend::Frontend, a: usize, settle: usize) {
    f.frame(&press(a), 1.0 / 60.0);
    run(f, settle);
}

#[test]
fn capture_quick_race_flow() {
    if !visos::file::game_dir().exists() {
        return;
    }
    let mut f = crate::frontend::Frontend::new(Default::default());
    // the legal screen reaches the main menu with GotoScreen, which is what puts it in the history
    assert!(f.fe.start(string_crc("LegalScreen")));
    run(&mut f, 2);
    f.fe.goto_screen(string_crc("MainMenuScreen"));
    run(&mut f, 30);
    // Start -> profile select -> B back with the menu open
    tap(&mut f, action::PAUSE, 60);
    assert_eq!(current_name(&f), string_crc("ProfileSelectScreen"));
    shot(&f, "profile_select.png");
    tap(&mut f, action::MENU_CANCEL, 60);
    assert_eq!(current_name(&f), string_crc("MainMenuScreen"));
    shot(&f, "main_menu.png");
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("QuickRaceTextScreen"));
    shot(&f, "quick_race_text.png");
    tap(&mut f, action::MENU_RIGHT, 10);
    shot(&f, "quick_race_text_right.png");
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("QuickRaceCarSelectScreen"));
    shot(&f, "car_select_tier.png");
    tap(&mut f, action::MENU_RIGHT, 10);
    tap(&mut f, action::MENU_SELECT, 30);
    shot(&f, "car_select_car.png");
    tap(&mut f, action::MENU_RIGHT, 10);
    shot(&f, "car_select_car_next.png");
    tap(&mut f, action::MENU_SELECT, 30);
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("QuickRaceStartScreen"));
    assert!(f.fe.quick_race.player_setups[0].is_some());
    shot(&f, "quick_race_start.png");
    // select goes to the loading screen, which asks game flow to start the race on its tenth frame
    let start = crate::feck2::feck2_manager::FrontendRequest::StartQuickRace;
    f.fe.requests.clear();
    tap(&mut f, action::MENU_SELECT, 0);
    for _ in 0..120 {
        if current_name(&f) == string_crc("LoadingScreen") {
            break;
        }
        run(&mut f, 1);
    }
    assert_eq!(current_name(&f), string_crc("LoadingScreen"));
    assert!(!f.fe.requests.contains(&start));
    run(&mut f, 30);
    shot(&f, "loading_screen.png");
    assert_eq!(f.fe.requests.iter().filter(|r| **r == start).count(), 1);
    // back from the race: RaceCompleteScreen, which returns to the quick race start screen
    assert!(f.fe.return_from_race());
    run(&mut f, 1);
    assert_eq!(current_name(&f), string_crc("RaceCompleteScreen"));
    run(&mut f, 60);
    assert_eq!(current_name(&f), string_crc("QuickRaceStartScreen"));
}

/// RaceCompleteScreen Update 0x00205900 polls core's save flow: Saving / SaveComplete popups, then back; a failed
/// write asks ErrorSaving's question, whose OK ends the flow (user exit) and goes back one screen
#[test]
fn capture_race_complete_save_popups() {
    use velocity_game::save_game::save_vars::SaveFlow;
    if !visos::file::game_dir().exists() {
        return;
    }
    let dir = std::env::temp_dir().join(format!("velocity_fe_save_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blocker = dir.join("file");
    std::fs::write(&blocker, b"x").unwrap();
    for (ok, path) in [(true, dir.join("Primary")), (false, blocker.join("Primary"))] {
        let mut f = crate::frontend::Frontend::new(Default::default());
        assert!(f.fe.start(string_crc("LegalScreen")));
        run(&mut f, 2);
        f.fe.goto_screen(string_crc("MainMenuScreen"));
        run(&mut f, 30);
        // the screen the race started from is in the history (GotoPrevious finds it there)
        f.fe.goto_screen(string_crc("QuickRaceTextScreen"));
        run(&mut f, 30);
        f.fe.post_race_screen = string_crc("RaceCompleteScreen");
        f.fe.return_screen = string_crc("QuickRaceTextScreen");
        f.fe.career.save_enabled = true;
        let mut flow = SaveFlow::default();
        flow.start_save(path, vec![1, 2, 3], true);
        assert_eq!(flow.written(), Some(ok));
        f.fe.save_flow = Some(flow);
        assert!(f.fe.return_from_race());
        run(&mut f, 30);
        assert_eq!(current_name(&f), string_crc("RaceCompleteScreen"));
        let (msg, dialog) = {
            let o = f.fe.overlay.as_ref().unwrap();
            (o.message_text, o.dialog_active)
        };
        if ok {
            assert!(f.fe.el(msg).is_visible() && !dialog);
            shot(&f, "race_complete_saving.png");
            run(&mut f, 200);
            shot(&f, "race_complete_saved.png");
            for _ in 0..400 {
                if current_name(&f) != string_crc("RaceCompleteScreen") {
                    break;
                }
                run(&mut f, 1);
            }
            assert_eq!(current_name(&f), string_crc("QuickRaceTextScreen"));
        } else {
            // the failed write leaves Saving at once, ErrorSaving waits out Saving's 3 s
            assert!(!dialog);
            for _ in 0..240 {
                if f.fe.overlay.as_ref().unwrap().dialog_active {
                    break;
                }
                run(&mut f, 1);
            }
            let o = f.fe.overlay.as_ref().unwrap();
            let (dialog, count) = (o.dialog_active, o.dialog_count);
            assert!(dialog && count == 1);
            shot(&f, "race_complete_save_error.png");
            tap(&mut f, action::MENU_SELECT, 5);
            assert!(!f.fe.save_flow.as_ref().unwrap().in_progress());
            assert_ne!(current_name(&f), string_crc("RaceCompleteScreen"));
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// DummyGameScreen (FEDummyGameScreen.cpp): rows per competitor, held right = +10 prestige per update, a position
/// asks core for the result and goes to RaceCompleteScreen; a race without a record reads "not in build"
#[test]
fn capture_dummy_game_screen() {
    use crate::feck2::screens::hangout::{CareerEventView, CareerRequest, CurrentRaceView};
    if !visos::file::game_dir().exists() {
        return;
    }
    let mut f = crate::frontend::Frontend::new(Default::default());
    assert!(f.fe.start(string_crc("LegalScreen")));
    run(&mut f, 2);
    f.fe.goto_screen(string_crc("MainMenuScreen"));
    run(&mut f, 30);
    let race = CurrentRaceView { competitors: 4, humans: 1, details: Some((string_crc("LADOWN_RACE01"), false)), entrants: vec![None; 4] };
    f.fe.career.event = Some(CareerEventView { crc: string_crc("UT1EVENT1"), current_race: Some(race), ..Default::default() });
    f.fe.goto_screen(string_crc("DummyGameScreen"));
    run(&mut f, 30);
    assert_eq!(current_name(&f), string_crc("DummyGameScreen"));
    let rows = f.dummy_game.lock().unwrap().rows;
    assert!(f.fe.el(rows[4]).is_visible() && !f.fe.el(rows[5]).is_visible());
    tap(&mut f, action::MENU_DOWN, 2);
    tap(&mut f, action::MENU_DOWN, 2);
    let mut held = press(action::MENU_RIGHT);
    f.frame(&held, 1.0 / 60.0);
    held.pads[0][action::MENU_RIGHT].changed = false;
    f.frame(&held, 1.0 / 60.0);
    run(&mut f, 2);
    shot(&f, "dummy_game.png");
    let (sel, prestige) = { let s = f.dummy_game.lock().unwrap(); (s.selection, s.prestige) };
    assert_eq!(sel, 2);
    assert!(prestige >= 10 && prestige % 10 == 0);
    f.fe.career.requests.clear();
    // a profile in use: RaceCompleteScreen waits for the save (none without game flow) instead of going back
    f.fe.career.save_enabled = true;
    tap(&mut f, action::MENU_SELECT, 5);
    assert_eq!(f.fe.career.requests, vec![CareerRequest::DummyResult { position: 2, prestige }]);
    assert_eq!(current_name(&f), string_crc("RaceCompleteScreen"));
    // no record: "not in build", play race does nothing
    f.fe.career.event.as_mut().unwrap().current_race.as_mut().unwrap().details = None;
    f.fe.goto_screen(string_crc("DummyGameScreen"));
    run(&mut f, 30);
    shot(&f, "dummy_game_not_in_build.png");
    assert!(!f.dummy_game.lock().unwrap().valid);
}

/// main menu MULTIPLAYER -> MultiplayerScreen; Split Screen -> track -> car select for player 1, then player 2 ->
/// the start screen with both player rows; Xbox Live / System Link go to unported screens (stays)
#[test]
fn capture_split_screen_flow() {
    if !visos::file::game_dir().exists() {
        return;
    }
    let mut f = crate::frontend::Frontend::new(Default::default());
    assert!(f.fe.start(string_crc("LegalScreen")));
    run(&mut f, 2);
    f.fe.goto_screen(string_crc("MainMenuScreen"));
    run(&mut f, 30);
    tap(&mut f, action::PAUSE, 60);
    tap(&mut f, action::MENU_CANCEL, 60);
    assert_eq!(current_name(&f), string_crc("MainMenuScreen"));
    tap(&mut f, action::MENU_DOWN, 5);
    tap(&mut f, action::MENU_DOWN, 5);
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("MultiplayerScreen"));
    shot(&f, "multiplayer.png");
    // Xbox Live: OnlineSignInScreen is not made, the screen stays
    tap(&mut f, action::MENU_SELECT, 30);
    assert_eq!(current_name(&f), string_crc("MultiplayerScreen"));
    tap(&mut f, action::MENU_DOWN, 5);
    tap(&mut f, action::MENU_DOWN, 5);
    tap(&mut f, action::MENU_SELECT, 60);
    assert!(f.fe.quick_race.two_players);
    assert_eq!(current_name(&f), string_crc("QuickRaceTextScreen"));
    tap(&mut f, action::MENU_SELECT, 60);
    for player in 0..2 {
        assert_eq!(current_name(&f), string_crc("QuickRaceCarSelectScreen"), "player {player}");
        assert_eq!(f.quick_race_car_select.lock().unwrap().player, player);
        tap(&mut f, action::MENU_RIGHT, 10);
        tap(&mut f, action::MENU_SELECT, 30);
        tap(&mut f, action::MENU_RIGHT, 10);
        tap(&mut f, action::MENU_SELECT, 30);
        tap(&mut f, action::MENU_SELECT, 60);
    }
    assert_eq!(current_name(&f), string_crc("QuickRaceStartScreen"));
    assert!(f.fe.quick_race.player_setups[1].is_some());
    let rows = f.quick_race_start.lock().unwrap().player_info;
    assert!(f.fe.el(rows[1][0]).is_visible() && f.fe.el(rows[1][1]).is_visible());
    shot(&f, "split_screen_start.png");
}

/// events whose current (first) race has no record: the hangout sends them to DummyGameScreen (0x00219F70 with
/// 0x002092A0 false)
#[test]
fn shipped_event_races_have_records() {
    use velocity_game::event_database::event_database::CareerQuery;
    use velocity_game::event_database::event_database_track_loading::CEventDatabase;
    if !visos::file::game_dir().exists() {
        return;
    }
    struct NoCareer;
    impl CareerQuery for NoCareer {
        fn prestige(&self) -> i32 {
            0
        }
        fn owns_vehicle(&self, _: u32) -> bool {
            false
        }
    }
    let mut db = CEventDatabase::default();
    db.build(&|_| true, &NoCareer);
    assert!(!db.events.is_empty());
    let missing: Vec<u32> = db
        .events
        .iter()
        .map(|e| crate::career_event_view(&db, e))
        .filter(|v| v.current_race.as_ref().and_then(|r| r.details).is_none())
        .map(|v| v.crc)
        .collect();
    // shipped data: 18 events' first races name no race record ("RACE CURRENTLY NOT AVAILABLE IN BUILD")
    assert_eq!(missing.len(), 18, "events without a race record: {missing:08x?}");
    if std::env::var_os("FE_DUMP_EVENTS").is_some() {
        for e in &db.events {
            let v = crate::career_event_view(&db, e);
            eprintln!(
                "ev {:08x} cat {} kind {} tier {} flags {:x} order {} area {} races {} laps {} type {} opp {} rec {} unlock {:?} team {:08x} prestige {} prize {}",
                v.crc, v.category, v.kind, v.tier, v.flags, e.order, v.area, v.race_count, v.laps, v.race_type, v.opponents,
                v.current_race.as_ref().and_then(|r| r.details).is_some(), v.unlock_event, v.unlock_team, e.prestige_reward, e.prize
            );
        }
    }
}

#[test]
fn capture_osd_in_game() {
    if !visos::file::game_dir().exists() {
        return;
    }
    use crate::osd::osd_screens::osd_in_game_screen as ig;
    let mut tex = TextureStore::default();
    let mut fonts = CFontHelper::default();
    assert!(fonts.load_game_fonts(&mut tex));
    let mut strings = crate::string_manager::string_manager::CStringManager::new();
    strings.set_language("English");
    strings.load_index_file("StrIndex.idx").unwrap();
    for s in ["Common", "Game"] {
        let _ = strings.load_section_name(s);
    }
    let mut osd = crate::osd::osd_manager::COSDManager::default();
    let mut screen = ig::create(&mut osd, &fonts, &strings, &tex, false, 0, None);
    let st = ig::OsdRaceStatus { racing: true, race_length: 4, position: 2, lap: 2, laps: 3, lap_time: 83.47, points: 1250, combo: 3, multiplier: 2, wrong_way: true, ..Default::default() };
    ig::update(&mut screen, &mut osd, &st);
    osd.update(1.0, &fonts, &strings, &tex);
    let mut dl = DrawList::default();
    osd.render(&fonts, &strings, &tex, &mut dl);
    assert!(!dl.quads.is_empty());
    let cfg = crate::osd::speedo::HudConfig::load("digione").unwrap();
    let speedo = crate::osd::speedo::Speedo::new(cfg, &mut tex);
    let input = crate::osd::speedo::SpeedoInput { speed: 42.0, rpm: 4300.0, gear: 3, nos: 0.6, zone: 0.4, turbo: 0.5 };
    speedo.render(&input, &mut dl);
    // minimap from a synthetic oval track (the real edges come from the track data via game flow)
    use crate::osd::osd_map::*;
    let oval = |r: f32| (0..96).map(|i| { let a = i as f32 / 96.0 * std::f32::consts::TAU; [a.cos() * r * 1.8, 0.0, a.sin() * r] }).collect::<Vec<_>>();
    let (inside, outside) = (oval(300.0), oval(330.0));
    let map = HudMap::construct(&HudMapDesc { inside: &inside, outside: &outside, size: 128, border: 4, closed: true });
    let mut mr = COSDMapRender::new(map, [255, 208, 48], 0, [-10.75, -7.2, 1.0], &mut tex);
    let mi = MapInput { player: [0.0, 0.0, 315.0], heading: [1.0, 0.0, 0.0], rivals: vec![[90.0, 0.0, 312.0]], police: vec![] };
    // the map screen texture of the 3D hud's hud_map_screen model
    let mut ml = DrawList::default();
    mr.render(&mi, 1.0 / 60.0, &mut ml);
    let img = ml.rasterise(&tex, crate::osd::osd_3d_interface::MAP_SCREEN_SIZE, crate::osd::osd_3d_interface::MAP_SCREEN_SIZE);
    crate::vtf::write_png(&out_dir().join("osd_map_screen.png"), &img).unwrap();
    let img = dl.rasterise(&tex, 640, 480);
    crate::vtf::write_png(&out_dir().join("osd_in_game.png"), &img).unwrap();
}

#[test]
fn dump_hud_textures() {
    if !visos::file::game_dir().exists() {
        return;
    }
    let v = visos::file::Vcd::open(r"\OSD\HUDS\digione.vcd").unwrap();
    let names: Vec<String> = v.entries().map(|e| e.name.to_string()).collect();
    for name in names {
        if let Some(img) = v.get(&name).and_then(crate::vtf::decode) {
            crate::vtf::write_png(&out_dir().join(format!("hud_{}.png", name.trim_end_matches(".vtf"))), &img).unwrap();
        }
    }
}

#[test]
fn capture_pause_menu() {
    if !visos::file::game_dir().exists() {
        return;
    }
    use crate::game_menu_manager::game_menu_manager::*;
    let mut tex = TextureStore::default();
    let mut fonts = CFontHelper::default();
    assert!(fonts.load_game_fonts(&mut tex));
    let mut strings = crate::string_manager::string_manager::CStringManager::new();
    strings.set_language("English");
    strings.load_index_file("StrIndex.idx").unwrap();
    let sections: Vec<String> = strings.index.as_ref().unwrap().sections.iter().map(|s| s.name.trim_end_matches(".sbk").to_string()).collect();
    for s in sections {
        let _ = strings.load_section_name(&s);
    }
    let mut osd = crate::osd::osd_manager::COSDManager::default();
    tex.load_vcd("\\OSD\\OSDT.vcd");
    let mut gm = CGameMenuManager { screens: crate::osd::osd_screens::osd_screens::COSDPauseScreens::new(&tex), game_mode: 1, ..Default::default() };
    assert_ne!(gm.screens.texture, 0, "pauseScreen");
    let pause = CGameMenu::pause(&gm.options, &mut strings);
    gm.enter(pause, &mut osd);
    let idle = FeInput::default();
    // one game update per frame here
    let frame = |gm: &mut CGameMenuManager, osd: &mut crate::osd::osd_manager::COSDManager, strings: &mut crate::string_manager::string_manager::CStringManager, i: &FeInput| {
        gm.update_screens(osd, &fonts, strings, &tex);
        gm.update(i, 1.0 / 60.0, osd, &fonts, strings, &tex);
        osd.update(1.0, &fonts, strings, &tex);
    };
    for _ in 0..8 {
        frame(&mut gm, &mut osd, &mut strings, &idle);
    }
    assert_eq!(gm.screens.screens[0].state, 2);
    assert_eq!(gm.screens.fade, 1.0);
    frame(&mut gm, &mut osd, &mut strings, &press(action::MENU_DOWN));
    let mut dl = DrawList::default();
    osd.render(&fonts, &strings, &tex, &mut dl);
    crate::vtf::write_png(&out_dir().join("pause_menu.png"), &dl.rasterise(&tex, 640, 480)).unwrap();
    // into the audio menu, nudge the master volume, back out, then quit -> confirm
    frame(&mut gm, &mut osd, &mut strings, &press(action::MENU_SELECT));
    assert_eq!(gm.stack.last().unwrap().kind, MenuKind::Audio);
    frame(&mut gm, &mut osd, &mut strings, &press(action::MENU_LEFT));
    assert!((gm.options.volumes[0] - 0.95).abs() < 1e-5);
    frame(&mut gm, &mut osd, &mut strings, &idle);
    frame(&mut gm, &mut osd, &mut strings, &idle);
    // audio screen up, pause items masked (4)
    assert_eq!(gm.screens.screens[2].state, 2);
    assert_ne!(osd.hidden_mask & 4, 0);
    let mut dl = DrawList::default();
    osd.render(&fonts, &strings, &tex, &mut dl);
    crate::vtf::write_png(&out_dir().join("pause_audio.png"), &dl.rasterise(&tex, 640, 480)).unwrap();
    frame(&mut gm, &mut osd, &mut strings, &press(action::MENU_CANCEL));
    assert_eq!(gm.stack.last().unwrap().kind, MenuKind::Pause);
    assert_eq!(osd.hidden_mask & (4 | 128), 128);
    for _ in 0..4 {
        frame(&mut gm, &mut osd, &mut strings, &press(action::MENU_DOWN));
        frame(&mut gm, &mut osd, &mut strings, &idle);
    }
    frame(&mut gm, &mut osd, &mut strings, &press(action::MENU_SELECT));
    assert_eq!(gm.stack.last().unwrap().kind, MenuKind::Quit { restart: false });
    frame(&mut gm, &mut osd, &mut strings, &idle);
    frame(&mut gm, &mut osd, &mut strings, &idle);
    frame(&mut gm, &mut osd, &mut strings, &idle);
    let mut dl = DrawList::default();
    osd.render(&fonts, &strings, &tex, &mut dl);
    crate::vtf::write_png(&out_dir().join("pause_quit.png"), &dl.rasterise(&tex, 640, 480)).unwrap();
    frame(&mut gm, &mut osd, &mut strings, &press(action::MENU_SELECT));
    assert_eq!(gm.requests.last(), Some(&PauseMenuRequest::QuitToFrontend));
    assert!(!gm.is_open());
    // the screens close: the pause one fades out, then every item goes back to the pool
    for _ in 0..8 {
        frame(&mut gm, &mut osd, &mut strings, &idle);
    }
    assert!(gm.screens.screens.iter().all(|s| s.flags == 0));
    assert!(osd.items.iter().all(|i| i.free));
}

#[test]
fn capture_race_hud() {
    if !visos::file::game_dir().exists() {
        return;
    }
    use crate::hud::*;
    let oval = |r: f32| (0..96).map(|i| { let a = i as f32 / 96.0 * std::f32::consts::TAU; [a.cos() * r * 1.8, 0.0, a.sin() * r] }).collect::<Vec<_>>();
    let mut h = HudResources::new("digione", false, &oval(300.0), &oval(330.0), true);
    let mut input = RaceHudInput::default();
    input.status = crate::osd::osd_screens::osd_in_game_screen::OsdRaceStatus { racing: true, race_length: 6, position: 1, lap: 1, laps: 3, lap_time: 12.3, ..Default::default() };
    input.speedo = crate::osd::speedo::SpeedoInput { speed: 30.0, rpm: 5600.0, gear: 4, nos: 1.0, zone: 0.2, turbo: 0.0 };
    input.map.player = [0.0, 0.0, 315.0];
    input.map.heading = [1.0, 0.0, 0.0];
    h.frame(&input,1.0 / 60.0);
    let dl = h.frame(&input,1.0 / 60.0);
    crate::vtf::write_png(&out_dir().join("race_hud.png"), &dl.rasterise_overlay(&h.textures, 640, 480)).unwrap();
    // name tags (Player names on): one rival 10 m ahead (full alpha = 128), one 30 m (half), one 50 m (hidden)
    {
        use bevy::math::{Mat4, Vec3};
        use crate::osd::osd_screens::osd_in_game_screen::{NameTagVehicle, NAME_TAG_FIRST};
        h.menus.options.display[crate::game_options::display::NAMES] = true;
        let view = Mat4::look_at_rh(Vec3::new(0.0, 1.5, 4.0), Vec3::new(0.0, 1.0, -10.0), Vec3::Y);
        let proj = Mat4::perspective_rh(50f32.to_radians(), 4.0 / 3.0, 1.0, 300.0);
        input.name_tags.view_proj = Some((proj * view).to_cols_array());
        let car = |slot: usize, z: f32, name: &str| NameTagVehicle { slot, pos: [1.5 * slot as f32 - 2.0, 0.0, -z], name: name.into(), colour: 0xFF20_A0FF };
        input.name_tags.vehicles = vec![car(1, 10.0, "Rival Near"), car(2, 30.0, "Rival Mid"), car(3, 50.0, "Rival Far")];
        let dl = h.frame(&input,1.0 / 60.0);
        let item = |s: usize| &h.osd.items[h.screen.items[NAME_TAG_FIRST + s].unwrap()];
        assert_eq!(item(1).colour >> 24, 128);
        // 0x000F9340 takes VsAIDriver 0x00015F80's D3DCOLOR as is, only the alpha replaced
        assert_eq!(item(1).colour & 0x00FF_FFFF, 0x0020_A0FF);
        // 30.02 m (1 m to the side): a = 0.499 -> 63
        assert_eq!(item(2).colour >> 24, 63);
        assert!(item(3).flags & crate::osd::osd_manager::flag::HIDDEN != 0);
        crate::vtf::write_png(&out_dir().join("race_hud_names.png"), &dl.rasterise_overlay(&h.textures, 640, 480)).unwrap();
        input.name_tags = Default::default();
    }
    // OSDManager 0x000ED6E0 -> 0x000F9980: the cinematic bars (menu_top / menu_bottom) on, then off again
    {
        use crate::osd::osd_screens::osd_in_game_screen::set_cinematic_bars;
        set_cinematic_bars(&h.screen, &mut h.osd, true);
        let dl = h.frame(&input, 1.0 / 60.0);
        for s in [23, 24] {
            assert!(h.osd.items[h.screen.items[s].unwrap()].flags & crate::osd::osd_manager::flag::HIDDEN == 0);
        }
        crate::vtf::write_png(&out_dir().join("race_hud_bars.png"), &dl.rasterise_overlay(&h.textures, 640, 480)).unwrap();
        set_cinematic_bars(&h.screen, &mut h.osd, false);
    }
    h.open_pause();
    h.frame(&input,1.0 / 60.0);
    h.game_updates(&input, 6);
    let dl = h.frame(&input,1.0 / 60.0);
    crate::vtf::write_png(&out_dir().join("race_hud_paused.png"), &dl.rasterise(&h.textures, 640, 480)).unwrap();
}

/// two players, top / bottom (GameManager views kinds 2 / 3): one OSD screen per player with the split layouts
/// (`unk_6861C8` / `unk_6862A0`), each speedo squashed into its half; side by side (`unk_686378` / `unk_686450`)
#[test]
fn capture_split_hud() {
    if !visos::file::game_dir().exists() {
        return;
    }
    use crate::hud::*;
    use crate::osd::osd_screens::osd_in_game_screen as ig;
    for side in [false, true] {
        let mut h = HudResources::new_split("digione", false, &[], &[], true, Some(side));
        let mut input = RaceHudInput::default();
        input.status = ig::OsdRaceStatus { racing: true, race_length: 6, position: 2, lap: 1, laps: 3, lap_time: 12.3, ..Default::default() };
        input.speedo = crate::osd::speedo::SpeedoInput { speed: 30.0, rpm: 5600.0, gear: 4, ..Default::default() };
        let rects = if side { [[0.0, 0.0, 0.5, 1.0], [0.5, 0.0, 0.5, 1.0]] } else { [[0.0, 0.0, 1.0, 0.5], [0.0, 0.5, 1.0, 0.5]] };
        let mut st2 = input.status.clone();
        st2.position = 5;
        st2.lap_time = 9.87;
        let sp2 = crate::osd::speedo::SpeedoInput { speed: 12.0, rpm: 2500.0, gear: 2, ..Default::default() };
        input.split = Some(SplitHudInput { side_by_side: side, rects, status: st2, speedo: sp2 });
        h.frame(&input, 1.0 / 60.0);
        let dl = h.frame(&input, 1.0 / 60.0);
        let s2 = h.screen2.as_ref().unwrap();
        assert_eq!(s2.player, 1);
        // player 1: no attract / logo / bars / network items; its position item from its own layout
        assert!(s2.items[20..27].iter().all(|i| i.is_none()));
        let pos = &h.osd.items[s2.items[0].unwrap()];
        assert_eq!((pos.x, pos.y), if side { (845.0, 925.0) } else { (235.0, 475.0) });
        let pos0 = &h.osd.items[h.screen.items[0].unwrap()];
        assert_eq!((pos0.x, pos0.y), (235.0, 925.0));
        assert_eq!(pos.string, Some(crate::string_manager::string_manager::string_crc("OSD_5TH")));
        // both speedos land inside their halves of the 512 target
        let (mut a, mut b) = (0, 0);
        for q in &dl.quads {
            let c = q.pos.iter().fold([0.0f32; 2], |s, p| [s[0] + p[0] / 4.0, s[1] + p[1] / 4.0]);
            let k = if side { c[0] } else { c[1] };
            if q.texture == 0 {
                continue;
            }
            if k < 256.0 { a += 1 } else { b += 1 }
        }
        assert!(a > 0 && b > 0, "{a} {b}");
        let name = if side { "split_hud_side.png" } else { "split_hud.png" };
        crate::vtf::write_png(&out_dir().join(name), &dl.rasterise_overlay(&h.textures, 640, 480)).unwrap();
        // GameManager 0x000B9010: the 3D OSD pass (name tags) once per player view with that view's camera, cut to
        // the view; both screens own tag items, so each view draws both sets
        {
            use bevy::math::{Mat4, Vec3};
            use ig::{NameTagVehicle, NAME_TAG_FIRST};
            let proj = Mat4::perspective_rh(50f32.to_radians(), 4.0 / 3.0, 1.0, 300.0);
            let v0 = Mat4::look_at_rh(Vec3::new(0.0, 1.5, 4.0), Vec3::new(0.0, 1.0, -10.0), Vec3::Y);
            let v1 = Mat4::look_at_rh(Vec3::new(3.0, 1.5, 4.0), Vec3::new(3.0, 1.0, -10.0), Vec3::Y);
            input.name_tags.view_proj = Some((proj * v0).to_cols_array());
            input.name_tags.view_proj1 = Some((proj * v1).to_cols_array());
            input.name_tags.vehicles = vec![NameTagVehicle { slot: 2, pos: [1.0, 0.0, -10.0], name: "Rival Near".into(), colour: 0xFF20_A0FF }];
            h.menus.options.display[crate::game_options::display::NAMES] = false;
            let off = h.frame(&input, 1.0 / 60.0).quads.len();
            h.menus.options.display[crate::game_options::display::NAMES] = true;
            let dl = h.frame(&input, 1.0 / 60.0);
            let s2 = h.screen2.as_ref().unwrap();
            assert!(s2.items[NAME_TAG_FIRST + 2].is_some() && h.screen.items[NAME_TAG_FIRST + 2].is_some());
            let tags = &dl.quads[..dl.quads.len() - off];
            assert!(!tags.is_empty());
            let half = |q: &crate::render2d::Quad| {
                let k = |p: &[f32; 2]| if side { p[0] } else { p[1] };
                if q.pos.iter().all(|p| k(p) <= 256.0) { 0 } else if q.pos.iter().all(|p| k(p) >= 256.0) { 1 } else { 9 }
            };
            let n0 = tags.iter().filter(|q| half(q) == 0).count();
            let n1 = tags.iter().filter(|q| half(q) == 1).count();
            assert!(n0 > 0 && n0 == n1 && n0 + n1 == tags.len(), "{n0} {n1} {}", tags.len());
            let name = if side { "split_hud_side_names.png" } else { "split_hud_names.png" };
            crate::vtf::write_png(&out_dir().join(name), &dl.rasterise_overlay(&h.textures, 640, 480)).unwrap();
        }
    }
}

#[test]
fn capture_race_over_menu_and_attract_hud() {
    if !visos::file::game_dir().exists() {
        return;
    }
    use crate::game_menu_manager::game_menu_manager::{MenuKind, PauseMenuRequest, RaceOverEvent};
    use crate::hud::*;
    let mut input = RaceHudInput::default();
    input.status = crate::osd::osd_screens::osd_in_game_screen::OsdRaceStatus { racing: true, race_length: 4, position: 2, lap: 3, laps: 3, lap_time: 61.5, session_state: Some(7), ..Default::default() };
    // a quick race just finished (not completed: continue is offered and selected, 0x000B16E0)
    let mut h = HudResources::new("digione", false, &[], &[], true);
    h.menus.enter_race_over(RaceOverEvent::default(), &mut h.osd);
    h.frame(&input,1.0 / 60.0);
    let m = h.menus.stack.last().unwrap();
    assert_eq!(m.items.len(), 3);
    assert_eq!(m.selected, 2);
    h.game_updates(&input, 3);
    let dl = h.frame(&input,1.0 / 60.0);
    crate::vtf::write_png(&out_dir().join("race_over.png"), &dl.rasterise(&h.textures, 640, 480)).unwrap();
    h.game_update(&input, &press(action::MENU_SELECT), 1.0 / 60.0);
    assert_eq!(h.menus.requests.last(), Some(&PauseMenuRequest::EndOfGame { quit_event: false }));
    assert!(!h.menus.is_open());
    // return to the front end asks first; restart asks first
    h.menus.enter_race_over(RaceOverEvent::default(), &mut h.osd);
    h.game_update(&input, &press(action::MENU_UP), 1.0 / 60.0);
    h.game_update(&input, &press(action::MENU_SELECT), 1.0 / 60.0);
    assert_eq!(h.menus.stack.last().unwrap().kind, MenuKind::Quit { restart: false });
    h.menus.leave_all(&mut h.osd);
    // completed: no continue, return selected and left at once; failed: restart selected, no confirm
    let done = RaceOverEvent { completed: true, ..Default::default() };
    h.menus.enter_race_over(done, &mut h.osd);
    assert_eq!((h.menus.stack.last().unwrap().items.len(), h.menus.stack.last().unwrap().selected), (2, 1));
    h.game_update(&input, &press(action::MENU_SELECT), 1.0 / 60.0);
    assert_eq!(h.menus.requests.last(), Some(&PauseMenuRequest::EndOfGame { quit_event: false }));
    h.menus.enter_race_over(RaceOverEvent { failed: true, ..Default::default() }, &mut h.osd);
    h.game_update(&input, &press(action::MENU_SELECT), 1.0 / 60.0);
    assert_eq!(h.menus.requests.last(), Some(&PauseMenuRequest::Restart));
    // CGameMenu::Update 0x000AA3E0 runs per game update: a held down moves once, then again after 0.2 s of updates
    // (6th update at 1/24); the sweep's smoke lost the quit to this when its 3-frame holds outlasted 0.2 s
    h.menus.enter_race_over(RaceOverEvent { completed: true, ..Default::default() }, &mut h.osd);
    let mut held = press(action::MENU_DOWN);
    let sel = |h: &HudResources| h.menus.stack.last().unwrap().selected;
    for n in 1..=6 {
        h.game_update(&input, &held, 1.0 / 24.0);
        held.pads[0][action::MENU_DOWN].changed = false;
        assert_eq!(sel(&h), if n < 6 { 0 } else { 1 }, "update {n}");
    }
    h.menus.leave_all(&mut h.osd);
    // the attract race's OSD (session state 3): race hud masked, logo + "Press Start", which fades and swaps
    let mut h = HudResources::new("digione", false, &[], &[], true);
    input.status = crate::osd::osd_screens::osd_in_game_screen::OsdRaceStatus { racing: true, laps: 3, lap: 1, lap_time: 0.5, attract: true, session_state: Some(3), ..Default::default() };
    h.frame(&input,1.0 / 60.0);
    assert_eq!(h.osd.hidden_mask & 33, 1);
    let dl = h.frame(&input,1.0 / 60.0);
    crate::vtf::write_png(&out_dir().join("attract_hud.png"), &dl.rasterise(&h.textures, 640, 480)).unwrap();
    input.status.lap_time = 2.5;
    h.game_updates(&input, 1);
    h.game_updates(&input, 65);
    let s20 = h.screen.items[20].unwrap();
    let s21 = h.screen.items[21].unwrap();
    assert!(h.osd.items[s20].flags & crate::osd::osd_manager::flag::HIDDEN != 0);
    assert!(h.osd.items[s21].flags & crate::osd::osd_manager::flag::HIDDEN == 0);
    assert_eq!(h.screen.fade.alpha, 0);
}

#[test]
fn capture_main_menu_attract_start() {
    if !visos::file::game_dir().exists() {
        return;
    }
    let mut f = crate::frontend::Frontend::new(Default::default());
    let mut a = velocity_game::font_helper::CAttractMode::default();
    a.idle_time = 0.5;
    a.movie_chance = -1.0;
    f.fe.attract.mode = Some(a);
    f.fe.start(string_crc("MainMenuScreen"));
    run(&mut f, 20);
    assert!(f.fe.attract.mode.as_ref().unwrap().timer_on);
    // input keeps it idle
    tap(&mut f, action::MENU_DOWN, 20);
    assert!(f.fe.attract.jump.is_none());
    run(&mut f, 30);
    let cur = f.fe.current.map(|c| f.fe.screens[c].crc);
    assert_eq!(cur, Some(string_crc("LoadingScreen")));
    assert!(f.fe.attract.jump.is_some() && f.fe.attract.mode.as_ref().unwrap().playing);
    assert_eq!(f.fe.post_race_screen, string_crc("MainMenuScreen"));
    // Leave stopped the idle timer
    assert!(!f.fe.attract.mode.as_ref().unwrap().timer_on);
}

#[test]
fn capture_options_flow() {
    if !visos::file::game_dir().exists() {
        return;
    }
    let mut f = crate::frontend::Frontend::new(Default::default());
    assert!(f.fe.start(string_crc("LegalScreen")));
    run(&mut f, 2);
    f.fe.goto_screen(string_crc("MainMenuScreen"));
    run(&mut f, 30);
    tap(&mut f, action::PAUSE, 60);
    tap(&mut f, action::MENU_CANCEL, 60); // profile select -> back
    for _ in 0..3 {
        tap(&mut f, action::MENU_DOWN, 5);
    }
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("OptionsScreen"));
    shot(&f, "options.png");
    // video: down to the trash talk row and toggle it
    tap(&mut f, action::MENU_DOWN, 5);
    tap(&mut f, action::MENU_DOWN, 5);
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("VideoOptionsScreen"));
    shot(&f, "options_video.png");
    // the ratio row is gone (port): TFT is row 1
    let tft = f.fe.options.display[crate::game_options::display::TFT];
    tap(&mut f, action::MENU_DOWN, 5);
    tap(&mut f, action::MENU_RIGHT, 10);
    assert_eq!(f.fe.options.display[crate::game_options::display::TFT], !tft);
    assert!(f.fe.options_changed);
    shot(&f, "options_video_toggled.png");
    tap(&mut f, action::MENU_CANCEL, 60);
    assert_eq!(current_name(&f), string_crc("OptionsScreen"));
    // PC settings (port): windowed -> the resolution row is disabled and skipped
    tap(&mut f, action::MENU_DOWN, 5);
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("PcOptionsScreen"));
    let sel = |f: &crate::frontend::Frontend| f.fe.overlay.as_ref().unwrap().menu_selected;
    assert_eq!(f.fe.options.pc.display_mode, 0);
    tap(&mut f, action::MENU_DOWN, 5);
    assert_eq!(sel(&f), 2);
    shot(&f, "options_pc_windowed.png");
    tap(&mut f, action::MENU_RIGHT, 10);
    assert_eq!(f.fe.options.pc.fps_cap, 30);
    tap(&mut f, action::MENU_UP, 5);
    assert_eq!(sel(&f), 0);
    tap(&mut f, action::MENU_RIGHT, 10);
    tap(&mut f, action::MENU_RIGHT, 10);
    assert_eq!(f.fe.options.pc.display_mode, 2);
    tap(&mut f, action::MENU_DOWN, 5);
    assert_eq!(sel(&f), 1);
    shot(&f, "options_pc_fullscreen.png");
    f.fe.options.pc = Default::default();
    tap(&mut f, action::MENU_CANCEL, 60);
    assert_eq!(current_name(&f), string_crc("OptionsScreen"));
    // sound: the music row opens the soundtrack picker
    tap(&mut f, action::MENU_DOWN, 5);
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("AudioOptionsScreen"));
    shot(&f, "options_audio.png");
    tap(&mut f, action::MENU_DOWN, 5);
    tap(&mut f, action::MENU_SELECT, 10);
    shot(&f, "options_audio_soundtrack.png");
    tap(&mut f, action::MENU_CANCEL, 10);
    tap(&mut f, action::MENU_CANCEL, 60);
    assert_eq!(current_name(&f), string_crc("OptionsScreen"));
    // controller
    tap(&mut f, action::MENU_DOWN, 5);
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("ConfigOptionsScreen"));
    run(&mut f, 120);
    shot(&f, "options_controller.png");
    tap(&mut f, action::MENU_CANCEL, 60);
    // language: right to US and select (OPTION_PC sits between video and sound)
    for _ in 0..4 {
        tap(&mut f, action::MENU_UP, 5);
    }
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("LanguageOptionsScreen"));
    shot(&f, "options_language.png");
    tap(&mut f, action::MENU_RIGHT, 40);
    shot(&f, "options_language_us.png");
    tap(&mut f, action::MENU_SELECT, 10);
    run(&mut f, 60);
    assert_eq!(f.fe.strings.language, "US");
    assert_eq!(f.fe.options.language, "US");
    assert_eq!(current_name(&f), string_crc("OptionsScreen"));
    shot(&f, "options_after_language.png");
}

/// OptionsScreen PROFILE -> EDIT PLATE -> PlateCustomizationScreen: the style menu, BRITISH, the keyboard with the
/// plate text, L1 / R1 / backspace, DONE -> back to the profile options with style 1 / "OCTAN"
#[test]
fn capture_plate_flow() {
    if !visos::file::game_dir().exists() {
        return;
    }
    let mut f = crate::frontend::Frontend::new(Default::default());
    assert!(f.fe.start(string_crc("LegalScreen")));
    run(&mut f, 2);
    f.fe.goto_screen(string_crc("MainMenuScreen"));
    run(&mut f, 30);
    tap(&mut f, action::PAUSE, 60);
    tap(&mut f, action::MENU_CANCEL, 60);
    for _ in 0..3 {
        tap(&mut f, action::MENU_DOWN, 5);
    }
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("OptionsScreen"));
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("ProfileOptionsScreen"));
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("PlateCustomizationScreen"));
    shot(&f, "plate_style.png");
    tap(&mut f, action::MENU_DOWN, 10);
    tap(&mut f, action::MENU_SELECT, 30);
    assert_eq!(f.fe.plate.style, 1);
    assert!(crate::feck2::screens::common::fe_keyboard_overlay::kb(&mut f.fe, |k, _| k.is_visible()).unwrap());
    // the first key of a pre-filled string clears it (clear-on-first); a cursor move first keeps it
    tap(&mut f, action::MENU_L1, 10);
    tap(&mut f, action::MENU_R1, 10);
    tap(&mut f, action::MENU_OPTION1, 10);
    shot(&f, "plate_keyboard.png");
    tap(&mut f, action::PAUSE, 60);
    assert_eq!(f.fe.plate.text, "OCTAN");
    assert_eq!(current_name(&f), string_crc("ProfileOptionsScreen"));
    shot(&f, "plate_back.png");
}

/// OptionsScreen TUTORIALS -> TipsScreen: the list (only seen tips enabled), a tip's text, its movie after 5 frames,
/// back to the list; then the hangout's path (a set tip goes back to the previous screen after its movie)
#[test]
fn capture_loading_tutorial_labels() {
    if !visos::file::game_dir().exists() {
        return;
    }
    let mut f = crate::frontend::Frontend::new(Default::default());
    assert!(f.fe.start(string_crc("LegalScreen")));
    run(&mut f, 2);
    // tip 2 (zone): steering, accelerate / brake and zone labels on the controller picture
    f.loading.lock().unwrap().tutorial_tip = 2;
    f.fe.goto_screen(string_crc("LoadingScreen"));
    run(&mut f, 5);
    assert_eq!(current_name(&f), string_crc("LoadingScreen"));
    let (texts, lines) = {
        let s = f.loading.lock().unwrap();
        (s.tip_texts, s.tip_lines)
    };
    let shown: Vec<usize> = (0..10).filter(|&i| f.fe.el(texts[i]).is_visible()).collect();
    log::info!("tutorial labels on inputs {:?}", shown);
    assert_eq!(shown.len(), 3);
    for i in 0..10 {
        assert_eq!(f.fe.el(lines[i]).is_visible(), shown.contains(&i));
    }
    shot(&f, "loading_tutorial.png");
}

#[test]
fn capture_tips_flow() {
    if !visos::file::game_dir().exists() {
        return;
    }
    let mut f = crate::frontend::Frontend::new(Default::default());
    assert!(f.fe.start(string_crc("LegalScreen")));
    run(&mut f, 2);
    f.fe.goto_screen(string_crc("MainMenuScreen"));
    run(&mut f, 30);
    tap(&mut f, action::PAUSE, 60);
    tap(&mut f, action::MENU_CANCEL, 60);
    for _ in 0..3 {
        tap(&mut f, action::MENU_DOWN, 5);
    }
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("OptionsScreen"));
    // tips 2..8 seen (dword_6F71D8), INITIAL has no movie so it stays disabled
    f.fe.career.tips_watched = 0xFE;
    // TUTORIALS is row 6 (the port's OPTION_PC at row 3)
    for _ in 0..6 {
        tap(&mut f, action::MENU_DOWN, 5);
    }
    tap(&mut f, action::MENU_SELECT, 60);
    assert_eq!(current_name(&f), string_crc("TipsScreen"));
    shot(&f, "tips_list.png");
    tap(&mut f, action::MENU_DOWN, 5);
    f.fe.movies.clear();
    f.frame(&press(action::MENU_SELECT), 1.0 / 60.0);
    run(&mut f, 2);
    shot(&f, "tips_text.png");
    run(&mut f, 6);
    assert_eq!(f.fe.movies, vec!["TIPS\\zone".to_string()]);
    assert_eq!(current_name(&f), string_crc("TipsScreen"));
    run(&mut f, 30);
    shot(&f, "tips_back.png");
    // the hangout's tip 3 (NOS), not seen yet: text, movie, flag, back to OptionsScreen (the previous screen here)
    tap(&mut f, action::MENU_CANCEL, 60);
    assert_eq!(current_name(&f), string_crc("OptionsScreen"));
    f.fe.career.tips_watched = 0;
    f.fe.movies.clear();
    crate::feck2::screens::common::fe_tips_screen::set_tip(&mut f.fe, 3);
    f.fe.goto_screen_name("TipsScreen");
    run(&mut f, 60);
    assert_eq!(f.fe.movies, vec!["TIPS\\nos".to_string()]);
    assert_eq!(f.fe.career.tips_watched, 4);
    assert_eq!(current_name(&f), string_crc("OptionsScreen"));
}

/// AudioPlaylistScreen (DJ booth) on the real `.mus` lists: kind / shuffle steps, build + add by id + remove +
/// commit through the playlist mirror, the play request, leaving saves the lists
#[test]
fn capture_audio_playlist() {
    if !visos::file::game_dir().exists() {
        return;
    }
    let Some(tracks) = velocity_audio::vs_audio_track_list::VsAudioTrackList::load() else { return };
    let mut f = crate::frontend::Frontend::new(Default::default());
    f.fe.playlist.tracks = tracks;
    assert!(f.fe.start(string_crc("LegalScreen")));
    run(&mut f, 2);
    f.fe.goto_screen(string_crc("MainMenuScreen"));
    run(&mut f, 30);
    f.fe.playlist.list = 0;
    let before = f.fe.playlist.tracks.lists[0].tracks.len();
    let (kind, sh) = (f.fe.playlist.list_type(0), f.fe.playlist.list_shuffle(0));
    f.fe.goto_screen(string_crc("AudioPlaylistScreen"));
    run(&mut f, 30);
    assert_eq!(current_name(&f), string_crc("AudioPlaylistScreen"));
    // row 0 right: kind + 1 (wraps 7 -> 0), row 1: shuffle toggles
    tap(&mut f, action::MENU_RIGHT, 2);
    assert_eq!(f.fe.playlist.list_type(0), (kind + 1) % 7);
    tap(&mut f, action::MENU_DOWN, 2);
    tap(&mut f, action::MENU_RIGHT, 2);
    assert_eq!(f.fe.playlist.list_shuffle(0), (sh + 1) % 2);
    shot(&f, "audio_playlist_menu.png");
    // display tracks -> the working list; option 1 -> master tracks; select adds one after the node
    tap(&mut f, action::MENU_DOWN, 2);
    tap(&mut f, action::MENU_SELECT, 2);
    assert_eq!(f.audio_playlist.lock().unwrap().state, 1);
    tap(&mut f, action::MENU_OPTION1, 2);
    assert_eq!(f.audio_playlist.lock().unwrap().state, 2);
    shot(&f, "audio_playlist_add.png");
    tap(&mut f, action::MENU_SELECT, 2);
    assert_eq!(f.audio_playlist.lock().unwrap().state, 1);
    assert_eq!(f.fe.playlist.tracks.edit[0].as_ref().unwrap().order().len(), before + 1);
    // select -> edit menu: play requests the node's song; select again on "remove" removes it
    tap(&mut f, action::MENU_SELECT, 2);
    assert_eq!(f.audio_playlist.lock().unwrap().state, 3);
    shot(&f, "audio_playlist_edit.png");
    tap(&mut f, action::MENU_DOWN, 2);
    assert_eq!(f.audio_playlist.lock().unwrap().edit_sel, 1);
    tap(&mut f, action::MENU_SELECT, 2);
    assert_eq!(f.fe.playlist.tracks.edit[0].as_ref().unwrap().order().len(), before);
    tap(&mut f, action::MENU_SELECT, 2);
    tap(&mut f, action::MENU_SELECT, 2);
    assert!(matches!(f.fe.playlist.requests.last(), Some(crate::feck2::playlist_link::PlaylistRequest::PlayTrack { .. })));
    // cancel: edit menu -> list -> first menu (commits), then leaves saving the lists
    tap(&mut f, action::MENU_CANCEL, 2);
    tap(&mut f, action::MENU_CANCEL, 2);
    assert_eq!(f.audio_playlist.lock().unwrap().state, 0);
    assert!(f.fe.playlist.tracks.edit[0].is_none());
    assert_eq!(f.fe.playlist.tracks.lists[0].tracks.len(), before);
    tap(&mut f, action::MENU_CANCEL, 2);
    assert!(f.fe.playlist.requests.contains(&crate::feck2::playlist_link::PlaylistRequest::SaveLists));
    run(&mut f, 60);
    assert_ne!(current_name(&f), string_crc("AudioPlaylistScreen"));
}


