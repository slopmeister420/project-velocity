//! src\Feck2\Screens\Common\FETipsScreen.cpp object — `gpLoadingScreen` (`dword_7673C8`, 0x84 bytes; create
//! 0x002405C0, Init 0x002406A0): "LOADING" over `\FE\LOADSCR.VCD`'s background with a game tip. The event's tip
//! (`CEvent` +0x54) picks a tutorial tip, otherwise a random loading tip (the trailblazer one for LA circuit
//! RACE02). Ten frames after Enter it asks GameManager to start the game (`GameManager__BBF60` sets GM +532,
//! here `FrontendRequest::StartQuickRace`).
//!
//! Tutorial tips label the controller picture (0x00241670): each tip's actions name the pad input they are bound
//! to (0x00241840) and draw that input's line (0x00241940, `dword_76A518` points: placeholder values in this build).

use std::sync::{Arc, Mutex};

use crate::feck2::feck2_manager::{Feck2, FrontendRequest};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::string_manager::string_manager::string_crc;

const LOADSCR_VCD: &str = r"\FE\LOADSCR.VCD";
/// `dword_75C670` / `dword_75C694` tutorial tips (title, info), index = event +0x54 (static init 0x00428F80)
pub const TUTORIAL_TITLES: [&str; 9] = [
    "NONE", "GAMETIPS_INITIAL_TITLE", "GAMETIPS_ZONE_TITLE", "GAMETIPS_NOS_TITLE", "GAMETIPS_DRIFT_TITLE",
    "GAMETIPS_SLIPSTREAM_TITLE", "GAMETIPS_DRAG_TITLE", "GAMETIPS_MANUALGEARS_TITLE", "GAMETIPS_TRAILBLAZER_TITLE",
];
const TUTORIAL_INFO: [&str; 9] = [
    "NONE", "GAMETIPS_INITIAL_INFO", "GAMETIPS_ZONE_INFO", "GAMETIPS_NOS_INFO", "GAMETIPS_DRIFT_INFO",
    "GAMETIPS_SLIPSTREAM_INFO", "GAMETIPS_DRAG_INFO", "GAMETIPS_MANUALGEARS_INFO", "GAMETIPS_TRAILBLAZER_INFO",
];
/// `dword_75C6B8` / `dword_75C6D0` loading tips
const LOADING_TITLES: [&str; 6] = [
    "LOADINGINFO_NOS_TITLE", "LOADINGINFO_QUICKRACE_TITLE", "LOADINGINFO_SHOWROOM_TITLE", "LOADINGINFO_PRESTIGE_TITLE",
    "LOADINGINFO_ONLINE_TITLE", "LOADINGINFO_TRAILBLAZER_TITLE",
];
const LOADING_INFO: [&str; 6] = [
    "LOADINGINFO_NOS_DETAILS", "LOADINGINFO_QUICKRACE_DETAILS", "LOADINGINFO_SHOWROOM_DETAILS", "LOADINGINFO_PRESTIGE_DETAILS",
    "LOADINGINFO_ONLINE_DETAILS", "LOADINGINFO_TRAILBLAZER_DETAILS",
];
/// `dword_6C1800`: tip text offsets from (284, 196)
const TIP_TEXT_POS: [[f32; 2]; 10] = [
    [100.0, 100.0], [100.0, 100.0], [100.0, 100.0], [100.0, 100.0], [127.0, -9.0], [0.0, -9.0], [100.0, 100.0],
    [100.0, 100.0], [100.0, 100.0], [34.0, 170.0],
];
/// `dword_76A518`: three line points per pad input, offset by (284, 196) (static init 0x00431080; the same
/// placeholder hook for every input but A)
const TIP_LINE_POINTS: [[[f32; 2]; 3]; 10] = {
    let hook = [[10.0, 10.0], [40.0, 10.0], [40.0, 40.0]];
    [[[10.0, 10.0], [100.0, 10.0], [100.0, 100.0]], hook, hook, hook, hook, hook, hook, hook, hook, hook]
};

/// `CFELoadingScreen` (0x84 bytes)
pub struct CFELoadingScreen {
    pub screen: ScreenId,             // +0
    pub layer: LayerId,               // +4
    pub background: ElementId,        // +8 "bg"
    pub loading_text: ElementId,      // +12
    pub tip_title: ElementId,         // +16
    pub tip_info: ElementId,          // +20
    pub tip_controller: ElementId,    // +24
    pub tip_image: ElementId,         // +28
    pub tip_texts: [ElementId; 10],   // +32
    pub tip_lines: [ElementId; 10],   // +72
    pub loaded: bool,                 // +128
    /// event +0x54 tutorial tip of the race being loaded (0 = a loading tip) when no career event is pending (a
    /// pending career event's `CareerEventView.loading_tip` wins); quick races leave it 0
    pub tutorial_tip: u32,
    pub frames: u32,                  // dword_767650
}

/// FETipsScreen.cpp 0x002405C0 / 0x002406A0 / 0x00240DB0 — `FECK2CreateLoadingScreen` (Init, AssignEvents)
pub fn create_loading_screen(fe: &mut Feck2) -> Arc<Mutex<CFELoadingScreen>> {
    let screen = fe.create_screen(&SScreenDesc { crc: string_crc("LoadingScreen") });
    let layer = fe.create_layer(Some(screen), &SLayerDesc { priority: 1, crc: string_crc("LoadingScreen_Layer"), ..Default::default() });
    let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 4, crc: string_crc("LoadingText"), ..Default::default() };
    d.position[0] = f32::from_bits(1133084672);
    d.position[1] = f32::from_bits(1138163712);
    d.string = string_crc("GENERAL_LOADING");
    let loading_text = fe.create_element(layer, &d);
    d.kind = ELEMENT_STATIC_TEXT;
    d.font = 2;
    d.crc = string_crc("LoadingTipTitle");
    d.position[0] = f32::from_bits(1109917696);
    d.position[1] = f32::from_bits(1116209152);
    d.string = 0;
    let tip_title = fe.create_element(layer, &d);
    d.font = 1;
    d.crc = string_crc("LoadingTipInfo");
    d.position[1] = f32::from_bits(1121189888);
    let tip_info = fe.create_element(layer, &d);
    d.kind = ELEMENT_2D;
    d.crc = string_crc("TipImage");
    d.texture = "tipImage".into();
    d.vcd = LOADSCR_VCD.into();
    d.position[0] = f32::from_bits(1101529088);
    d.position[1] = f32::from_bits(1128529920);
    let tip_image = fe.create_element(layer, &d);
    let mut tip_texts = [0; 10];
    let mut tip_lines = [0; 10];
    for i in 0..10 {
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = 0;
        d.crc = string_crc("TipText");
        d.position[0] = TIP_TEXT_POS[i][0] + 284.0;
        d.position[1] = TIP_TEXT_POS[i][1] + 196.0;
        d.string = 0;
        tip_texts[i] = fe.create_element(layer, &d);
        d.kind = ELEMENT_2D_LINE;
        d.font = 0;
        d.crc = string_crc("TipLine");
        d.position[0] = 0.0;
        d.position[1] = 0.0;
        d.string = 0;
        tip_lines[i] = fe.create_element(layer, &d);
    }
    d.kind = ELEMENT_2D;
    d.crc = string_crc("TipController");
    d.texture = "ls_controller".into();
    d.vcd = LOADSCR_VCD.into();
    d.position[0] = f32::from_bits(1133379584);
    d.position[1] = f32::from_bits(1128529920);
    let tip_controller = fe.create_element(layer, &d);
    d.crc = string_crc("bg");
    d.texture = "loading".into();
    d.position[0] = 0.0;
    d.position[1] = 0.0;
    let background = fe.create_element(layer, &d);
    let obj = Arc::new(Mutex::new(CFELoadingScreen {
        screen,
        layer,
        background,
        loading_text,
        tip_title,
        tip_info,
        tip_controller,
        tip_image,
        tip_texts,
        tip_lines,
        loaded: false,
        tutorial_tip: 0,
        frames: 0,
    }));
    // 0x00240DB0 — AssignEvents
    fe.map(layer, functor(&obj, ls_enter), ev::ENTER);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::LEAVE);
    fe.map(layer, functor(&obj, |_, _, _| true), ev::EXIT_SCREEN);
    fe.map(layer, functor(&obj, ls_update), ev::UPDATE);
    obj
}

/// FETipsScreen.cpp 0x00240F60 / 0x002415A0 / 0x002415E0 / 0x002417A0 / 0x002417F0 / FE2DOverlay.cpp 0x0027D220 — Enter: overlay with the static logo, the tip
fn ls_enter(s: &mut CFELoadingScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    // dword_6F7244 / dword_6F7248 (event prestige / unlock message) are reset by game flow
    if let Some(mut o) = fe.overlay.take() {
        overlay::attach_to_screen(&mut o, fe, Some(s.screen));
        overlay::reset(&mut o, fe);
        fe.el(o.static_logo).set_visible(true); // 0x0027D220
        overlay::set_hide_options(&mut o, fe, true);
        fe.overlay = Some(o);
    }
    fe.foyer.snap_to_target(); // FE3DCamera 0x0026FB60
    if !s.loaded {
        fe.element_load(s.loading_text);
        fe.element_load(s.background);
        s.loaded = true;
    }
    for id in [s.tip_title, s.tip_info, s.tip_controller, s.tip_image] {
        fe.element_load(id);
    }
    for i in 0..10 {
        fe.element_load(s.tip_texts[i]);
        fe.element_load(s.tip_lines[i]);
    }
    for id in [s.tip_title, s.tip_info, s.tip_controller, s.tip_image] {
        fe.el(id).set_visible(false);
    }
    for i in 0..10 {
        fe.el(s.tip_texts[i]).set_visible(false);
        fe.el(s.tip_lines[i]).set_visible(false);
    }
    // `*(dword_6F9BE8 + 84)`: the event being started
    let pending = fe.career.start_pending.and_then(|crc| fe.career.find_event(crc)).map(|e| e.loading_tip);
    let tip = pending.unwrap_or(s.tutorial_tip) as usize;
    if tip != 0 {
        fe.text(s.tip_title).set_string(string_crc(TUTORIAL_TITLES[tip.min(8)]));
        fe.el(s.tip_title).set_visible(true);
        fe.text(s.tip_info).set_string(string_crc(TUTORIAL_INFO[tip.min(8)]));
        fe.text(s.tip_info).set_box(f32::from_bits(1138491392), f32::from_bits(1120403456));
        fe.el(s.tip_info).set_visible(true);
        // 0x002415A0: the controller picture for tips 0..7
        if tip < 8 {
            fe.change_texture(s.tip_controller, "ls_controller", false);
            fe.el(s.tip_controller).set_visible(true);
        }
        tutorial_labels(s, fe, tip as u32);
        // 0x002415E0: the tip's picture
        let image = match tip {
            0 | 1 => Some("initial"),
            2 => Some("zone"),
            3 => Some("nos"),
            4 => Some("drift"),
            5 => Some("slip"),
            6 => Some("drag"),
            7 => Some("mgears"),
            _ => None,
        };
        if let Some(image) = image {
            fe.change_texture(s.tip_image, image, false);
            fe.el(s.tip_image).set_visible(true);
        }
        fe.change_texture(s.background, "loadingBackground", false);
    } else {
        // VsRandRange(0, 6) until it is not 5; the trailblazer tip for LA circuit RACE02
        let mut r;
        loop {
            r = (fe.char_rand.rand_unit() * 6.0) as i32; // VsRandRange 0x000134F0
            if r != 5 {
                break;
            }
        }
        let race = fe.quick_race.races[0].and_then(|i| fe.events.races.get(i));
        if let Some(race) = race {
            let area = fe.events.tracks.get(race.track).map_or(0, |t| t.area);
            if area == 1 && race.race_type as u32 == 3 && race.name_crc == string_crc("RACE02") {
                r = 5;
            }
        }
        let r = r.clamp(0, 5) as usize;
        fe.change_texture(s.background, "loadingBackground", false);
        fe.el(s.background).set_visible(true);
        fe.text(s.tip_title).set_string(string_crc(LOADING_TITLES[r]));
        fe.el(s.tip_title).set_visible(true);
        fe.text(s.tip_info).set_string(string_crc(LOADING_INFO[r]));
        fe.text(s.tip_info).set_box(f32::from_bits(1138491392), f32::from_bits(1120403456));
        fe.el(s.tip_info).set_visible(true);
        // 0x002417A0 / 0x002417F0: trailblazer picture and icon
        if r == 5 {
            fe.change_texture(s.tip_image, "trailblazer_Image", false);
            fe.el(s.tip_image).set_visible(true);
            fe.change_texture(s.tip_controller, "trailblazer_Icon", false);
            fe.element_set_position(s.tip_controller, f32::from_bits(1134362624), f32::from_bits(1129840640), 0.0);
            fe.el(s.tip_controller).set_visible(true);
        }
    }
    s.frames = 0;
    true
}

/// FETipsScreen.cpp 0x00241670 — the actions a tutorial tip labels: steering (27) and accelerate / brake (30), plus
/// zone (39) for tip 2, nitrous (38) for 3, handbrake (33) for 4
fn tutorial_labels(s: &mut CFELoadingScreen, fe: &mut Feck2, tip: u32) {
    match tip {
        1 | 5 | 6 | 7 => {
            label_action(s, fe, 27);
            label_action(s, fe, 30);
        }
        2 => {
            label_action(s, fe, 27);
            label_action(s, fe, 30);
            label_action(s, fe, 39);
        }
        3 => {
            label_action(s, fe, 27);
            label_action(s, fe, 30);
            label_action(s, fe, 38);
        }
        4 => {
            label_action(s, fe, 27);
            label_action(s, fe, 30);
            label_action(s, fe, 33);
        }
        _ => {}
    }
}

/// FETipsScreen.cpp 0x00241840 — the first pad input (`off_6C1850`) pad 0's mapping binds `action` to gets the
/// action's description (UserInput 0x001E7360) and its line
fn label_action(s: &mut CFELoadingScreen, fe: &mut Feck2, action: usize) {
    use crate::feck2::screens::options::fe_config_options_screen::{action_button, action_string, BUTTONS};
    let config = fe.options.controller_config[0] as i32;
    for (i, button) in BUTTONS.iter().enumerate() {
        let want = string_crc(button);
        let desc = action_string(action);
        if desc != 0 && action_button(config, action) == want {
            fe.text(s.tip_texts[i]).set_string(desc);
            fe.el(s.tip_texts[i]).set_visible(true);
            tip_line(s, fe, i);
            return;
        }
    }
}

/// FETipsScreen.cpp 0x00241940 — line `i` through its three points, shown
fn tip_line(s: &mut CFELoadingScreen, fe: &mut Feck2, i: usize) {
    let id = s.tip_lines[i];
    if let Some(line) = fe.el(id).as_line() {
        line.clear_points();
        for p in TIP_LINE_POINTS[i] {
            line.add_point(p[0] + 284.0, p[1] + 196.0);
        }
    }
    fe.el(id).set_visible(true);
}

/// FETipsScreen.cpp 0x00241520 — Update: the overlay transition ends; on the tenth frame the game starts loading
fn ls_update(s: &mut CFELoadingScreen, fe: &mut Feck2, _e: &CEvent) -> bool {
    if let Some(mut o) = fe.overlay.take() {
        overlay::end_transition(&mut o, fe);
        fe.overlay = Some(o);
    }
    s.frames += 1;
    if s.frames == 10 {
        // the original then drops this update functor through the layer (vtable +12); `frames` stays past 10
        s.frames = 11;
        // GameManager__BBF60 starts the event `m_pEvent` points at: the hangout's career event, else the quick race's
        // (a game jump's event, `CGameJump::Execute`: core already races it, nothing to raise)
        if std::mem::take(&mut fe.attract.loading_jump) {
            return true;
        }
        match fe.career.start_pending.take() {
            Some(event) => fe.career.requests.push(crate::feck2::screens::hangout::CareerRequest::StartCareerEvent { event }),
            None if std::mem::take(&mut fe.career.continue_pending) => {
                fe.career.requests.push(crate::feck2::screens::hangout::CareerRequest::ContinueEvent)
            }
            None => fe.requests.push(FrontendRequest::StartQuickRace),
        }
    }
    true
}
