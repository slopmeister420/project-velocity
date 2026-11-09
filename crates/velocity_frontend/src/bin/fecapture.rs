//! headless front end captures with the 3D foyer: `cargo run -p velocity_frontend --bin fecapture [-- --game <dir>]`.
//! Boots the frontend at the main menu, presses through to the options screens with a scripted pad (keyboard
//! device of the visos input layer) and writes `target/fe_captures/scene_*.png` (640x480), then exits.

use bevy::app::ScheduleRunnerPlugin;
use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use velocity_frontend::foyer::FoyerCamera;
use velocity_frontend::{FrontendCommand, FrontendView};
use visos::input::input_device::{key, NUM_KEYS};

#[derive(Component)]
struct Retargeted;

#[derive(Resource)]
struct Capture {
    target: Handle<Image>,
    frame: u32,
}

/// (frame, key pressed for 6 frames or None, screenshot name)
const SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (150, None, Some("scene_main_menu_press_start")),
    (160, Some(key::START), None),
    // the profile select: back to the main menu
    (230, Some(key::B), None),
    (300, None, Some("scene_main_menu")),
    (310, Some(key::DOWN), None),
    (330, Some(key::DOWN), None),
    (350, Some(key::DOWN), None),
    (380, Some(key::A), None),
    (500, None, Some("scene_options")),
    (510, Some(key::DOWN), None),
    (530, Some(key::DOWN), None),
    (560, Some(key::A), None),
    (680, None, Some("scene_options_video")),
    (690, Some(key::B), None),
    (800, Some(key::DOWN), None),
    (820, Some(key::DOWN), None),
    (840, Some(key::DOWN), None),
    (860, Some(key::A), None),
    (1000, None, Some("scene_options_controller")),
    (1010, Some(key::B), None),
    (1100, Some(key::B), None),
    (1200, Some(key::UP), None),
    (1220, Some(key::UP), None),
    (1240, Some(key::UP), None),
    (1290, None, Some("scene_hangout_nochars")),
    // the hangout screen (not ported) shows the characters: done by hand at HANGOUT_SHOW
    (1400, None, Some("scene_hangout_characters")),
    (1420, Some(key::A), None),
    (1600, Some(key::A), None),
    (1800, None, Some("scene_car_select_tier")),
    (1810, Some(key::A), None),
    (1950, None, Some("scene_car_select_car")),
    (1960, Some(key::A), None),
    (2200, None, Some("scene_car_select_colour_stage")),
    // R2 (MenuR2 = BLACK): the statistics list without prices, with the up / down arrows; option 1 hides it
    (2202, Some(key::BLACK), None),
    (2208, None, Some("scene_car_select_stats")),
    (2210, Some(key::X), None),
    (2300, None, Some("scene_car_select_swatches")),
    (2310, Some(key::RIGHT), None),
    (2330, Some(key::DOWN), None),
    (2450, None, Some("scene_car_select_swatch_colour")),
    (2470, None, Some("scene_car_select_spot")),
];
/// `fecapture options`: every options screen from the main menu's OPTIONS row (read-only: no left/right on rows that
/// change and save the options)
const OPTIONS_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (160, Some(key::START), None),
    (230, Some(key::B), None),
    (310, Some(key::DOWN), None),
    (330, Some(key::DOWN), None),
    (350, Some(key::DOWN), None),
    (380, Some(key::A), None),
    (500, None, Some("options")),
    (510, Some(key::DOWN), None),
    (540, Some(key::A), None),
    (660, None, Some("options_language")),
    (670, Some(key::B), None),
    // port: PC settings (row 3); windowed = resolution greyed, down skips it to the frame cap row
    (760, Some(key::DOWN), None),
    (780, Some(key::DOWN), None),
    (810, Some(key::A), None),
    (930, None, Some("options_pc")),
    (940, Some(key::DOWN), None),
    (960, None, Some("options_pc_skip")),
    (970, Some(key::B), None),
    (1060, Some(key::DOWN), None),
    (1090, Some(key::A), None),
    (1210, None, Some("options_audio")),
    (1220, Some(key::DOWN), None),
    (1240, Some(key::DOWN), None),
    (1280, None, Some("options_audio_music")),
    (1290, Some(key::B), None),
    (1380, Some(key::DOWN), None),
    (1410, Some(key::A), None),
    (1530, None, Some("options_controller")),
    (1540, Some(key::B), None),
    (1630, Some(key::UP), None),
    (1650, Some(key::UP), None),
    (1670, Some(key::UP), None),
    (1700, Some(key::A), None),
    (1820, None, Some("options_video")),
    (1830, Some(key::DOWN), None),
    (1850, Some(key::DOWN), None),
    (1880, None, Some("options_video_row2")),
    (1890, Some(key::B), None),
    (1980, Some(key::UP), None),
    (2000, Some(key::UP), None),
    (2030, Some(key::A), None),
    (2150, None, Some("options_profile")),
    (2160, Some(key::DOWN), None),
    (2180, Some(key::A), None),
    (2250, None, Some("options_profile_info")),
    (2260, Some(key::A), None),
    (2300, Some(key::DOWN), None),
    (2320, Some(key::DOWN), None),
    (2340, Some(key::DOWN), None),
    (2360, Some(key::A), None),
    (2420, None, Some("options_profile_new_dialog")),
    (2430, Some(key::DOWN), None),
    (2450, Some(key::A), None),
    (2500, Some(key::B), None),
    (2590, Some(key::B), None),
    (2690, None, Some("options_back_main_menu")),
];

/// `fecapture garage`: `jumptoscreen PlayerGarageScreen` (main menu -> garage), orbit with the right stick, the
/// use-car dialog. Each entry: (frame, key or None, screenshot name)
const GARAGE_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (200, None, Some("garage")),
    (210, Some(key::STICK_RX), None),
    (216, Some(key::STICK_RX), None),
    (222, Some(key::STICK_RX), None),
    (228, Some(key::STICK_RX), None),
    (300, None, Some("garage_orbit")),
    // R2 (MenuR2 = BLACK): the statistics list with prices, then scrolled up (down pressed), then off
    (310, Some(key::BLACK), None),
    (350, None, Some("garage_stats")),
    (360, Some(key::DOWN), None),
    (370, Some(key::DOWN), None),
    (400, None, Some("garage_stats_scrolled")),
    (410, Some(key::BLACK), None),
    (440, None, Some("garage_stats_off")),
    (450, Some(key::RIGHT), None),
    (480, Some(key::A), None),
    (540, None, Some("garage_sell_dialog")),
    (550, Some(key::B), None),
];

/// `fecapture typing`: profile select -> the on-screen keyboard, then a name typed as window key events (`KeyboardInput`
/// messages written by `script`, same path as the PC keyboard): "Kurwiszon_" (frame 340), backspace + left + 'X' (400)
const TYPING_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (160, Some(key::START), None),
    (270, Some(key::A), None),
    (330, None, Some("typing_empty")),
    (380, None, Some("typing_name")),
    (440, None, Some("typing_edited")),
    (480, None, Some("typing_done")),
];

/// the window key events `fecapture typing` sends: (frame, key, text)
fn typing_events(f: u32) -> Vec<(KeyCode, Key, Option<&'static str>)> {
    match f {
        340 => "Kurwiszon_".chars().map(|c| {
            let s: &'static str = Box::leak(c.to_string().into_boxed_str());
            (KeyCode::KeyA, Key::Character(s.into()), Some(s))
        }).collect(),
        400 => vec![(KeyCode::Backspace, Key::Backspace, None), (KeyCode::ArrowLeft, Key::ArrowLeft, None), (KeyCode::KeyX, Key::Character("X".into()), Some("X"))],
        460 => vec![(KeyCode::Enter, Key::Enter, Some("\r"))],
        _ => Vec::new(),
    }
}

/// `fecapture career`: main menu Start -> profile select, back, THE SCENE -> hangout, the garage character's menu,
/// Tory's event offer (events from a freshly built event database; no game flow, so nothing is started)
const CAREER_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (160, Some(key::START), None),
    (260, None, Some("career_profile_select")),
    // A on the empty slot: the keyboard; up, up (backspace, 'Q'), A A ("Qq"), down, down (DONE), A
    (270, Some(key::A), None),
    (330, None, Some("career_keyboard")),
    (340, Some(key::UP), None),
    (360, Some(key::UP), None),
    (380, Some(key::A), None),
    (400, Some(key::A), None),
    (420, None, Some("career_keyboard_typed")),
    (430, Some(key::DOWN), None),
    (450, Some(key::DOWN), None),
    (470, Some(key::A), None),
    (530, Some(key::B), None),
    (600, Some(key::DOWN), None),
    (640, Some(key::A), None),
    (780, None, Some("career_hangout")),
    (790, Some(key::RIGHT), None),
    (796, None, Some("career_hangout_flight")),
    (850, Some(key::A), None),
    (930, None, Some("career_hangout_menu")),
    (940, Some(key::B), None),
    (990, Some(key::LEFT), None),
    (1050, Some(key::A), None),
    (1130, None, Some("career_tory_offer")),
    (1140, Some(key::A), None),
    (1200, None, Some("career_tory_page2")),
    (1210, Some(key::A), None),
    (1270, None, Some("career_tory_page3")),
    (1280, Some(key::A), None),
    (1340, None, Some("career_tory_page4")),
    (1350, Some(key::B), None),
    (1380, Some(key::B), None),
    (1410, Some(key::B), None),
    (1440, Some(key::B), None),
    (1500, Some(key::B), None),
    (1570, None, Some("career_quit_dialog")),
];

/// `fecapture scene`: the career script into the hangout (THE SCENE), then right, right: the showroom character with
/// its prop (Chars.txt "Powerbook", FE3DCharacters 0x002609D0)
const SCENE_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (160, Some(key::START), None),
    (270, Some(key::A), None),
    (340, Some(key::UP), None),
    (360, Some(key::UP), None),
    (380, Some(key::A), None),
    (400, Some(key::A), None),
    (430, Some(key::DOWN), None),
    (450, Some(key::DOWN), None),
    (470, Some(key::A), None),
    (530, Some(key::B), None),
    (600, Some(key::DOWN), None),
    (640, Some(key::A), None),
    (780, None, Some("scene_hangout_tory")),
    (790, Some(key::RIGHT), None),
    (900, Some(key::RIGHT), None),
    (1020, None, Some("scene_hangout_showroom")),
];

/// `fecapture playlist`: the career script into the hangout, three characters right (the DJ), A (menu), A (the first
/// playlist -> AudioPlaylistScreen), the first menu, display tracks, add by id, the edit menu
const PLAYLIST_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (160, Some(key::START), None),
    (270, Some(key::A), None),
    (340, Some(key::UP), None),
    (360, Some(key::UP), None),
    (380, Some(key::A), None),
    (400, Some(key::A), None),
    (430, Some(key::DOWN), None),
    (450, Some(key::DOWN), None),
    (470, Some(key::A), None),
    (530, Some(key::B), None),
    (600, Some(key::DOWN), None),
    (640, Some(key::A), None),
    (790, Some(key::RIGHT), None),
    (900, Some(key::RIGHT), None),
    (1010, Some(key::RIGHT), None),
    (1140, None, Some("playlist_dj")),
    (1150, Some(key::A), None),
    (1240, None, Some("playlist_dj_menu")),
    (1250, Some(key::A), None),
    (1330, None, Some("playlist_menu")),
    (1340, Some(key::RIGHT), None),
    (1360, Some(key::DOWN), None),
    (1380, Some(key::DOWN), None),
    (1400, Some(key::A), None),
    (1440, None, Some("playlist_tracks")),
    (1450, Some(key::X), None),
    (1490, None, Some("playlist_add")),
    (1500, Some(key::A), None),
    (1520, Some(key::A), None),
    (1560, None, Some("playlist_edit")),
];

/// `fecapture teams`: the career script up to Tory's browser, then CHALENGES -> TEAM_INVITES (page 7, the
/// invite list), the invite question, YES -> joined message. Teams from `TEAMS.BIN` with the unlock teams of the
/// first two team-unlocking events unlocked (as a completed event would)
const TEAMS_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (160, Some(key::START), None),
    (270, Some(key::A), None),
    (340, Some(key::UP), None),
    (360, Some(key::UP), None),
    (380, Some(key::A), None),
    (400, Some(key::A), None),
    (430, Some(key::DOWN), None),
    (450, Some(key::DOWN), None),
    (470, Some(key::A), None),
    (530, Some(key::B), None),
    (600, Some(key::DOWN), None),
    (640, Some(key::A), None),
    (1050, Some(key::A), None),
    (1130, None, Some("teams_tory_offer")),
    (1140, Some(key::DOWN), None),
    (1160, Some(key::DOWN), None),
    (1180, Some(key::A), None),
    (1240, None, Some("teams_challenges")),
    (1250, Some(key::DOWN), None),
    (1270, Some(key::DOWN), None),
    (1290, Some(key::A), None),
    (1350, None, Some("teams_invites")),
    (1360, Some(key::A), None),
    (1420, None, Some("teams_invite_question")),
    (1430, Some(key::DOWN), None),
    (1450, Some(key::A), None),
    (1480, None, Some("teams_joined")),
];

/// the ai's team database as `CareerTeams` mirrors it (list order = reverse load order), with the unlock teams of
/// the first two events that unlock one unlocked / newly unlocked / inviting (TeamDatabase `unlock_team`)
fn career_teams(events: &[velocity_frontend::feck2::screens::hangout::CareerEventView]) -> Vec<velocity_frontend::feck2::screens::hangout::TeamView> {
    let Some(mut db) = velocity_ai::team_database::TeamDatabase::load() else { return Vec::new() };
    let mut unlock: Vec<u32> = events.iter().map(|e| e.unlock_team).filter(|&t| t != 0).collect();
    unlock.dedup();
    for &t in unlock.iter().take(2) {
        db.unlock_team(t);
    }
    db.teams
        .iter()
        .rev()
        .map(|t| velocity_frontend::feck2::screens::hangout::TeamView {
            crc: t.get_id(),
            name: t.name(),
            car: t.car(),
            status: t.status(),
            cash_bonus: t.cash_bonus(),
            flags: t.flags,
        })
        .collect()
}

/// which script runs: `garage`, `custom`, `career`, `teams` or the default foyer tour
fn mode() -> &'static str {
    let a: Vec<String> = std::env::args().collect();
    if a.iter().any(|x| x == "teams") {
        "teams"
    } else if a.iter().any(|x| x == "garage") {
        "garage"
    } else if a.iter().any(|x| x == "custom") {
        "custom"
    } else if a.iter().any(|x| x == "wheels") {
        "wheels"
    } else if a.iter().any(|x| x == "gauges") {
        "gauges"
    } else if a.iter().any(|x| x == "decals") {
        "decals"
    } else if a.iter().any(|x| x == "mix") {
        "mix"
    } else if a.iter().any(|x| x == "typing") {
        "typing"
    } else if a.iter().any(|x| x == "career") {
        "career"
    } else if a.iter().any(|x| x == "scene") {
        "scene"
    } else if a.iter().any(|x| x == "playlist") {
        "playlist"
    } else if a.iter().any(|x| x == "options") {
        "options"
    } else if a.iter().any(|x| x == "showroom") {
        "showroom"
    } else if a.iter().any(|x| x == "quickcar") {
        "quickcar"
    } else if a.iter().any(|x| x == "pinkslip") {
        "pinkslip"
    } else {
        "foyer"
    }
}

fn script_for(mode: &str) -> &'static [(u32, Option<usize>, Option<&'static str>)] {
    match mode {
        "garage" => GARAGE_SCRIPT,
        "custom" => CUSTOM_SCRIPT,
        "wheels" => WHEELS_SCRIPT,
        "gauges" => GAUGES_SCRIPT,
        "decals" => DECALS_SCRIPT,
        "mix" => MIX_SCRIPT,
        "career" => CAREER_SCRIPT,
        "typing" => TYPING_SCRIPT,
        "scene" => SCENE_SCRIPT,
        // FECAPTURE_LIST_ROW=n: the DJ's n-th list (DOWN n times), then DOWN DOWN A = "display tracks", one shot
        "playlist" if std::env::var("FECAPTURE_LIST_ROW").is_ok() => {
            let n: u32 = std::env::var("FECAPTURE_LIST_ROW").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
            let mut v: Vec<(u32, Option<usize>, Option<&'static str>)> = PLAYLIST_SCRIPT.iter().take_while(|s| s.0 < 1140).cloned().collect();
            v.push((1140, None, Some("playlist_dj")));
            for i in 0..n {
                v.push((1142 + 4 * i, Some(key::DOWN), None));
            }
            v.extend([(1160, Some(key::A), None), (1250, None, Some("playlist_row_menu")), (1300, Some(key::DOWN), None), (1330, Some(key::DOWN), None), (1360, Some(key::A), None), (1440, None, Some("playlist_row_tracks"))]);
            Box::leak(v.into_boxed_slice())
        }
        "playlist" => PLAYLIST_SCRIPT,
        "teams" => TEAMS_SCRIPT,
        "options" => OPTIONS_SCRIPT,
        "showroom" => SHOWROOM_SCRIPT,
        "pinkslip" => PINKSLIP_SCRIPT,
        "quickcar" => QUICKCAR_SCRIPT,
        _ => SCRIPT,
    }
}

/// the career events as game flow would show them (EventDatabase 0x00090090 with an empty career)
fn career_events() -> Vec<velocity_frontend::feck2::screens::hangout::CareerEventView> {
    use velocity_game::event_database::event_database::CareerQuery;
    struct NoCareer;
    impl CareerQuery for NoCareer {
        fn prestige(&self) -> i32 {
            0
        }
        fn owns_vehicle(&self, _: u32) -> bool {
            false
        }
    }
    let mut db = velocity_game::event_database::event_database_track_loading::CEventDatabase::default();
    db.build(&|_| true, &NoCareer);
    db.events.iter().map(|e| velocity_frontend::career_event_view(&db, e)).collect()
}

/// `fecapture custom`: `jumptoscreen CarCustomisationScreen`
const CUSTOM_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (200, None, Some("garage_custom")),
    (210, Some(key::RIGHT), None),
    (240, Some(key::A), None),
    (300, None, Some("garage_custom_styling")),
    (310, Some(key::A), None),
    (370, None, Some("garage_custom_body")),
    (380, Some(key::RIGHT), None),
    (410, Some(key::A), None),
    (470, None, Some("garage_custom_parts")),
    (480, Some(key::RIGHT), None),
    (500, Some(key::RIGHT), None),
    (540, None, Some("garage_custom_part2")),
    (550, Some(key::A), None),
    (600, None, Some("garage_custom_buy")),
    (610, Some(key::A), None),
    (660, None, Some("garage_custom_bought")),
    (670, Some(key::B), None),
    (700, Some(key::B), None),
    (730, Some(key::RIGHT), None),
    (760, Some(key::A), None),
    (790, Some(key::A), None),
    (820, Some(key::A), None),
    (880, None, Some("garage_custom_swatches")),
    (890, Some(key::RIGHT), None),
    (910, Some(key::DOWN), None),
    (950, None, Some("garage_custom_swatch2")),
];

/// `fecapture gauges`: Styling -> Gauges list (hud thumbnails), one step right, pick it, back in the groups
const GAUGES_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (200, None, Some("gauges_custom")),
    (210, Some(key::RIGHT), None),
    (240, Some(key::A), None),
    (270, Some(key::RIGHT), None),
    (290, Some(key::RIGHT), None),
    (310, Some(key::RIGHT), None),
    (340, None, Some("gauges_groups")),
    (350, Some(key::A), None),
    (410, None, Some("gauges_list")),
    (420, Some(key::RIGHT), None),
    (450, None, Some("gauges_list2")),
    (460, Some(key::A), None),
    (510, None, Some("gauges_picked")),
];

/// `fecapture decals`: Styling -> Decals -> side logos (item 2): the decal list, a decal picked, the next slot, the flip
const DECALS_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (200, None, Some("decals_custom")),
    (210, Some(key::RIGHT), None),
    (240, Some(key::A), None),
    (270, Some(key::RIGHT), None),
    (290, Some(key::RIGHT), None),
    (320, Some(key::A), None),
    (350, None, Some("decals_items")),
    (360, Some(key::RIGHT), None),
    (380, Some(key::RIGHT), None),
    (400, Some(key::A), None),
    (460, None, Some("decals_list")),
    (470, Some(key::RIGHT), None),
    (490, Some(key::RIGHT), None),
    (530, None, Some("decals_picked")),
    (540, Some(key::DOWN), None),
    (580, None, Some("decals_slot2")),
];

/// `fecapture mix`: Styling -> Paint -> mix your own (item 4): before, hue moved, other row, shade moved
const MIX_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (210, Some(key::RIGHT), None),
    (225, Some(key::RIGHT), None),
    (240, Some(key::A), None),
    (270, Some(key::A), None),
    (290, Some(key::RIGHT), None),
    (300, Some(key::RIGHT), None),
    (310, Some(key::RIGHT), None),
    (320, Some(key::RIGHT), None),
    (330, Some(key::RIGHT), None),
    (340, Some(key::A), None),
    (360, None, Some("mix_before")),
    (370, Some(key::RIGHT), None),
    (371, Some(key::RIGHT), None),
    (372, Some(key::RIGHT), None),
    (373, Some(key::RIGHT), None),
    (374, Some(key::RIGHT), None),
    (375, Some(key::RIGHT), None),
    (376, Some(key::RIGHT), None),
    (377, Some(key::RIGHT), None),
    (378, Some(key::RIGHT), None),
    (379, Some(key::RIGHT), None),
    (380, Some(key::RIGHT), None),
    (381, Some(key::RIGHT), None),
    (382, Some(key::RIGHT), None),
    (383, Some(key::RIGHT), None),
    (384, Some(key::RIGHT), None),
    (385, Some(key::RIGHT), None),
    (386, Some(key::RIGHT), None),
    (387, Some(key::RIGHT), None),
    (388, Some(key::RIGHT), None),
    (389, Some(key::RIGHT), None),
    (440, None, Some("mix_hue")),
    (450, Some(key::DOWN), None),
    (460, Some(key::LEFT), None),
    (461, Some(key::LEFT), None),
    (462, Some(key::LEFT), None),
    (463, Some(key::LEFT), None),
    (464, Some(key::LEFT), None),
    (465, Some(key::LEFT), None),
    (466, Some(key::LEFT), None),
    (467, Some(key::LEFT), None),
    (468, Some(key::LEFT), None),
    (469, Some(key::LEFT), None),
    (470, Some(key::LEFT), None),
    (471, Some(key::LEFT), None),
    (472, Some(key::LEFT), None),
    (473, Some(key::LEFT), None),
    (474, Some(key::LEFT), None),
    (520, None, Some("mix_shade")),
];

/// `fecapture wheels`: the customisation screen's wheel style list, then a wheel change (the bounce) and the steer
const WHEELS_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (200, None, Some("wheels_custom")),
    (210, Some(key::RIGHT), None),
    (240, Some(key::A), None),
    (300, None, Some("wheels_styling")),
    (310, Some(key::RIGHT), None),
    (330, Some(key::A), None),
    (350, None, Some("wheels_sub")),
    (352, Some(key::RIGHT), None), // Size (the style list has only "Stock" for the 300ZX)
    (385, Some(key::A), None),
    (410, None, Some("wheels_list")),
    (420, Some(key::RIGHT), None),
    (422, None, Some("wheels_bounce_a")),
    (425, None, Some("wheels_bounce_b")),
    (429, None, Some("wheels_bounce_c")),
    (520, None, Some("wheels_settled")),
];

/// `fecapture showroom`: `jumptoscreen CarShowroomScreen` — the list (tier 1, car 2, tier 2), A on a car (camera
/// flight to the spot), turntable with the right stick, doors, purchase dialog -> bought -> current car, colours,
/// back to the list
const SHOWROOM_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (200, None, Some("showroom_list")),
    (210, Some(key::DOWN), None),
    (240, None, Some("showroom_list_car2")),
    (250, Some(key::RIGHT), None),
    (290, None, Some("showroom_list_tier2")),
    (300, Some(key::LEFT), None),
    (340, Some(key::A), None),
    (380, None, Some("showroom_car_flight")),
    (480, None, Some("showroom_car")),
    (490, Some(key::STICK_RX), None),
    (496, Some(key::STICK_RX), None),
    (502, Some(key::STICK_RX), None),
    (560, None, Some("showroom_car_turned")),
    (570, Some(key::X), None),
    (620, None, Some("showroom_doors_closed")),
    (630, Some(key::A), None),
    (680, None, Some("showroom_buy_dialog")),
    (690, Some(key::A), None),
    (740, None, Some("showroom_bought")),
    (750, Some(key::A), None),
    (800, None, Some("showroom_customise_question")),
    (810, Some(key::B), None),
    (840, Some(key::DOWN), None),
    (860, Some(key::DOWN), None),
    (880, Some(key::A), None),
    (930, None, Some("showroom_colours")),
    (940, Some(key::RIGHT), None),
    (980, None, Some("showroom_colour2")),
    (990, Some(key::B), None),
    (1020, Some(key::B), None),
    (1160, None, Some("showroom_back_to_list")),
];

/// `fecapture pinkslip`: `jumptoscreen PinkSlipScreen` — the garage car list, second car, VIEW CAR, A -> the garage
/// screen on that car
/// `fecapture quickcar`: `jumptoscreen QuickRaceCarSelectScreen`, A on the first tier (camera move + MTV overlay)
const QUICKCAR_SCRIPT: &[(u32, Option<usize>, Option<&str>)] = &[
    (200, None, Some("quickcar_tiers")),
    (210, Some(key::A), None),
    (214, None, Some("quickcar_pick_early")),
    (222, None, Some("quickcar_pick_mid")),
    (260, None, Some("quickcar_pick")),
    (270, Some(key::RIGHT), None),
    (275, None, Some("quickcar_next")),
    (330, None, None),
];

const PINKSLIP_SCRIPT:&[(u32, Option<usize>, Option<&str>)] = &[
    (200, None, Some("pinkslip_list")),
    (210, Some(key::DOWN), None),
    (230, Some(key::RIGHT), None),
    (260, None, Some("pinkslip_view")),
    (270, Some(key::A), None),
    (420, None, Some("pinkslip_garage")),
];

/// frames during which the hangout characters are shown (FEHangoutScreen 0x00219300 / 0x00219920)
const HANGOUT_SHOW: (u32, u32) = (1300, 1410);

fn main() {
    if let Err(e) = visos::file::platform::init_game_dir_from_args() {
        eprintln!("{e}");
        std::process::exit(1);
    }
    // no boot movie screen: the capture starts the main menu itself
    // SAFETY: single-threaded, before the app starts
    unsafe { std::env::set_var("VELOCITY_NO_FRONTEND", "1") };
    let mut app = App::new();
    app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, ..default() })
                .disable::<WinitPlugin>(),
        )
        .add_plugins(ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 60.0)))
        .add_plugins((visos_render::RenderPlugin, velocity_frontend::FrontendPlugin))
        .add_plugins(velocity_audio::AudioPlugin)
        .add_systems(Startup, |mut a: MessageWriter<velocity_audio::AudioCommand>| {
            a.write(velocity_audio::AudioCommand::Init(velocity_audio::VsAudioInitInfo {
                frontend: true,
                track_dir: None,
                race_mode: None,
            }));
        })
        .add_systems(Startup, |mut a: MessageWriter<velocity_audio::AudioCommand>, mut gate: ResMut<velocity_audio::AudioUpdateGate>| {
            // FECAPTURE_AUDIO_UPDATE=1: VsAudio::Update runs every frame as in the game (music picks, track changes)
            if std::env::var_os("FECAPTURE_AUDIO_UPDATE").is_some() {
                gate.0 = true;
            }
            // FECAPTURE_LISTS=<a profile's "Primary" save>: its playlists as the profile load sends them (SaveFileData 0x00104F30)
            if let Some(p) = std::env::var_os("FECAPTURE_LISTS") {
                let b = std::fs::read(p).expect("FECAPTURE_LISTS");
                let o = 0x1EF4 + velocity_audio::vs_audio_track_list::SavedPlaylists::OPTIONS_OFFSET;
                a.write(velocity_audio::AudioCommand::SetLists(Box::new(velocity_audio::vs_audio_track_list::SavedPlaylists::from_bytes(&b[o..]))));
            }
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (retarget, script));
    if std::env::var_os("FECAPTURE_NO_3D").is_some() {
        app.add_systems(PostUpdate, no_3d.after(visos_render::full_screen_effects_manager_pc_xbox::FseSet::Execute));
    }
    let r = app.run();
    // a black 3D area (check_3d_area) exits non-zero
    if let AppExit::Error(code) = r {
        std::process::exit(code.get() as i32);
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut cmds: MessageWriter<FrontendCommand>) {
    let mut target = Image::new_target_texture(640, 480, TextureFormat::Rgba8UnormSrgb, None);
    target.asset_usage = RenderAssetUsages::default();
    let target = images.add(target);
    commands.insert_resource(Capture { target, frame: 0 });
    // the legal screen is the main menu's history entry, as on boot
    cmds.write(FrontendCommand::Start("LegalScreen".into()));
}

/// both frontend cameras draw into the capture image.
fn retarget(
    mut commands: Commands,
    cap: Res<Capture>,
    cams: Query<Entity, (With<Camera>, Or<(With<FoyerCamera>, With<FrontendView>)>, Without<Retargeted>)>,
) {
    for e in &cams {
        commands.entity(e).insert((RenderTarget::Image(cap.target.clone().into()), Retargeted));
    }
}

/// negative control for the black check: `FECAPTURE_NO_3D=1` keeps every 3D camera (foyer, FSE passes, car stencil)
/// from drawing and lets the 2D camera clear to black instead (else the target keeps stale frames); runs after the
/// FSE turns its pass cameras on.
fn no_3d(mut cams: Query<&mut Camera, With<Camera3d>>, mut fe: Query<&mut Camera, (With<FrontendView>, Without<Camera3d>)>) {
    for mut cam in &mut cams {
        cam.is_active = false;
    }
    for mut cam in &mut fe {
        cam.clear_color = ClearColorConfig::Custom(Color::BLACK);
    }
}

fn script(
    mut commands: Commands,
    mut cap: ResMut<Capture>,
    mut fe: ResMut<velocity_frontend::FrontendState>,
    mut exit: MessageWriter<AppExit>,
    images: Res<Assets<Image>>,
    mut kbd: MessageWriter<bevy::input::keyboard::KeyboardInput>,
) {
    let overlay = images.get(&fe.image).and_then(|i| i.data.as_deref());
    cap.frame += 1;
    let f = cap.frame;
    let mode = mode();
    if mode == "typing" {
        for (key_code, logical_key, text) in typing_events(f) {
            kbd.write(KeyboardInput {
                key_code,
                logical_key,
                state: ButtonState::Pressed,
                text: text.map(Into::into),
                repeat: false,
                window: Entity::PLACEHOLDER,
            });
        }
    }
    let script = script_for(mode);
    if f == 3 {
        if let Some(x) = fe.frontend.as_mut() {
            match mode {
                "garage" => x.fe.next_screen_crc = velocity_frontend::string_manager::string_manager::string_crc("PlayerGarageScreen"),
                "quickcar" => x.fe.next_screen_crc = velocity_frontend::string_manager::string_manager::string_crc("QuickRaceCarSelectScreen"),
                "pinkslip" => x.fe.next_screen_crc = velocity_frontend::string_manager::string_manager::string_crc("PinkSlipScreen"),
                "showroom" => {
                    x.fe.next_screen_crc = velocity_frontend::string_manager::string_manager::string_crc("CarShowroomScreen");
                    // as the hangout's showroom entry (0x0021C210) does
                    x.fe.garage.showroom_vehicle = x.fe.garage.current_vehicle;
                }
                "custom" | "wheels" | "gauges" | "decals" | "mix" => {
                    x.fe.next_screen_crc = velocity_frontend::string_manager::string_manager::string_crc("CarCustomisationScreen");
                    x.fe.jump_to_car_customisation = true;
                }
                "career" | "scene" => x.fe.career.events = career_events(),
                "playlist" => {
                    x.fe.career.events = career_events();
                }
                "teams" => {
                    // every event unlocked (as with `--unlock`) so the browser has its challenges page
                    x.fe.career.events = career_events();
                    for e in &mut x.fe.career.events {
                        e.flags &= !velocity_game::event_database::event_database::EVENT_FLAG_LOCKED;
                    }
                    x.fe.career.teams = career_teams(&x.fe.career.events);
                }
                _ => {}
            }
            x.fe.goto_screen_name("MainMenuScreen");
        }
    }
    // the 300ZX has only a stock style: fit a "size" wheel (other model) directly, as the part change does
    if mode == "wheels" && f == 450 {
        if let Some(x) = fe.frontend.as_mut() {
            let fe = &mut x.fe;
            let pick = fe.vehicles.as_ref().and_then(|db| db.wheels.iter().position(|w| w.string_1c.as_deref().is_some_and(|s| s.starts_with("wheel_001_size3_20"))));
            if let (Some(p), Some(v)) = (pick, fe.foyer_vehicles.slots.first_mut()) {
                if let Some(s) = v.setup.as_mut() {
                    s.wheel = Some(p);
                }
                let now = fe.stopwatch;
                fe.foyer_vehicles.start_wheel_bounce(now);
            }
        }
    }
    if mode != "foyer" {
        run_script(&mut commands, &cap, script, f, &mut exit, overlay);
        return;
    }
    // check shot: look at the car select's vehicle spot from 6 m away
    if f == 2460 {
        if let Some(x) = fe.frontend.as_mut() {
            if let Some(v) = x.fe.foyer_vehicles.slots.first() {
                let p = v.pos;
                x.fe.foyer.snap_to([p[0] - 4.0, p[1] + 2.0, p[2] - 4.5], p);
            }
        }
    }
    if f == HANGOUT_SHOW.0 || f == HANGOUT_SHOW.1 {
        if let Some(x) = fe.frontend.as_mut() {
            x.fe.characters.set_visible(f == HANGOUT_SHOW.0);
            // the first character's LOOKAT camera, as the hangout screen frames it
            let cam = x.fe.characters.chars.first().and_then(|c| c.cameras.first()).map(|c| (c.pos, c.look_at));
            match cam {
                Some((e, a)) if f == HANGOUT_SHOW.0 => x.fe.foyer.snap_to(e, a),
                _ => x.fe.foyer.custom = None,
            }
        }
    }
    run_script(&mut commands, &cap, SCRIPT, f, &mut exit, overlay);
}

/// press the script's keys and take its screenshots at frame `f`; exit 30 frames after the last entry
fn run_script(
    commands: &mut Commands,
    cap: &Capture,
    script: &[(u32, Option<usize>, Option<&str>)],
    f: u32,
    exit: &mut MessageWriter<AppExit>,
    overlay: Option<&[u8]>,
) {
    let mut keys = [0.0f32; NUM_KEYS];
    for (at, k, shot) in script {
        if let Some(k) = k {
            if f >= *at && f < at + 6 {
                keys[*k] = 1.0;
            }
        }
        if let Some(name) = shot {
            if f == *at {
                let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/fe_captures");
                let _ = std::fs::create_dir_all(&dir);
                let shot = name.to_string();
                let mask = overlay.map(|o| o.to_vec());
                commands
                    .spawn(Screenshot::image(cap.target.clone()))
                    .observe(save_to_disk(dir.join(format!("{name}.png"))))
                    .observe(move |c: On<ScreenshotCaptured>| check_3d_area(&shot, &c.image, mask.as_deref()));
            }
        }
    }
    visos::input::input_manager::set_keyboard(keys);
    if f > script.last().map(|s| s.0 + 30).unwrap_or(0) {
        let black = BLACK_SHOTS.lock().map(|b| b.clone()).unwrap_or_default();
        if black.is_empty() {
            exit.write(AppExit::Success);
        } else {
            error!("fecapture: 3D area black in {black:?}");
            exit.write(AppExit::from_code(2));
        }
    }
}

/// shots whose 3D area averaged ~0 rgb (graphics' request: catch a foyer that renders black)
static BLACK_SHOTS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// 3D area = the pixels where the frontend's 2D overlay (640x480 RGBA, `FrontendState.image`) is fully transparent,
/// i.e. only the foyer camera draws there. Black = fewer than 0.5% of them non-zero (the foyer clears to exact black;
/// the dark hangout has only ~1-2% above 4/255 but far more non-zero). Shots whose 2D covers everything are skipped.
fn check_3d_area(name: &str, img: &Image, overlay: Option<&[u8]>) {
    let (mut area, mut lit, mut nonzero, mut sum) = (0u32, 0u32, 0u32, 0.0f64);
    for y in 0..480u32 {
        for x in 0..640u32 {
            if overlay.is_some_and(|o| o.get(((y * 640 + x) * 4 + 3) as usize).is_some_and(|&a| a != 0)) {
                continue;
            }
            let Ok(c) = img.get_color_at(x, y) else { continue };
            let c = c.to_srgba();
            let m = c.red.max(c.green).max(c.blue) * 255.0;
            area += 1;
            sum += m as f64;
            if m > 4.0 {
                lit += 1;
            }
            if m >= 0.5 {
                nonzero += 1;
            }
        }
    }
    let pc = |n: u32| if area == 0 { 0.0 } else { n as f64 * 100.0 / area as f64 };
    let (lit_pc, nz_pc) = (pc(lit), pc(nonzero));
    info!(
        "fecapture: {name} 3D area {area} px, {nz_pc:.1}% non-zero, {lit_pc:.1}% above 4/255, mean max-rgb {:.2}",
        if area == 0 { 0.0 } else { sum / area as f64 }
    );
    if area > 0 && nz_pc < 0.5 {
        error!("fecapture: {name}: 3D area is black ({area} px, {nz_pc:.2}% non-zero)");
        if let Ok(mut b) = BLACK_SHOTS.lock() {
            b.push(name.to_string());
        }
    }
}
