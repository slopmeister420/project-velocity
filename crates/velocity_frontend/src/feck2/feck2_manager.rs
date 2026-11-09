//! src\Feck2\FECK2Manager.cpp — `CFrontendManager`: screen list, history, global event mapper, input -> events,
//! per-frame update and render.
//!
//! events are dispatched in the order the original raises them. the one structural difference: an event posted
//! from inside a handler is queued and dispatched right after that handler returns instead of recursively (the
//! original's nested calls would re-enter the screen object that is currently locked). handlers in the
//! original always post as their last action, so the observable order is the same.

use std::collections::VecDeque;

use crate::feck2::feck2_object::*;
use crate::font_helper::font_helper::CFontHelper;
use crate::render2d::{Colour, DrawList, TextureStore};
use crate::string_manager::string_manager::{string_crc, CStringManager};

/// `SCREEN_HISTORY`
pub const SCREEN_HISTORY: usize = 20;

/// `byte_6C118C` "Frontend.Enable 2D" tweak (initial value 1; `fe_tweaks` edits it).
pub static ENABLE_2D: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// `byte_6C118C`
pub fn enable_2d() -> bool {
    ENABLE_2D.load(std::sync::atomic::Ordering::Relaxed)
}

/// `dword_77FD9C` = VISOS_Colour(128,128,128,128).
pub const COLOUR_WHITE: Colour = 0xFFFF_FFFF;
/// `dword_77FD50` = VISOS_Colour(128,0,0,0).
pub const COLOUR_BLACK: Colour = 0xFF00_0000;
/// `dword_77FD3C` = VISOS_Colour(128,64,64,64).
pub const COLOUR_GREY: Colour = 0xFF80_8080;
/// `dword_769EDC` = `dword_77FD50` (static init 0x00430250).
pub const TEXT_SHADOW_COLOUR: Colour = COLOUR_BLACK;

/// `VISOS_Colour::VISOS_Colour(a, r, g, b)` (0x002B11C0): ps2-style 0..128 channels doubled and clamped.
pub fn visos_colour(a: u32, r: u32, g: u32, b: u32) -> Colour {
    let c = |v: u32| (2 * v).min(255);
    (c(a) << 24) | (c(r) << 16) | (c(g) << 8) | c(b)
}

/// requests the frontend raises towards game flow (the core agent's GameManager consumes these).
#[derive(Debug, Clone, PartialEq)]
pub enum FrontendRequest {
    /// the movie screen's Activate call (0x0023EDF0 -> 0x000B4D30). The idb names 0x000B4D30
    /// `GameManager::InitDatabases` (vehicle / opponent / event / team databases): it plays no movie.
    PlayIntroMovies,
    /// main menu "Quick Race": `GameInput::SetDeviceMask(0, mask)` for the pad that pressed select.
    SetControllerMask { player: u32, mask: u32 },
    /// a screen asked to start the race described by the quick race selections.
    StartQuickRace,
    /// `CFrontendManager::Exit`
    Exit,
}

/// what a global event slot does (`CGlobalEventFunctor`s and the screen forwarder, Initialise 0x001FDC20).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Global {
    Screen,
    GlobalPreload,
    Exit,
    StartScreen,
    GotoScreen,
    GotoPreviousScreen,
    GotoNextScreen,
    None,
}

/// per-controller action state the update reads (`UserInput`): value, previous value, changed this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct ActionState {
    pub value: f32,
    pub previous: f32,
    pub changed: bool,
}

/// `UserInput` action indices (UserInput.cpp 0x001E7C20 registration order).
pub mod action {
    pub const MENU_LEFT: usize = 0;
    pub const MENU_RIGHT: usize = 1;
    pub const MENU_DOWN: usize = 2;
    pub const MENU_UP: usize = 3;
    pub const MENU_UP_DOWN: usize = 4;
    pub const MENU_LEFT_RIGHT: usize = 5;
    pub const MENU_SELECT: usize = 6;
    pub const MENU_CANCEL: usize = 7;
    pub const ALLOY_EDIT_HORIZ: usize = 8;
    pub const ALLOY_EDIT_VERT: usize = 9;
    pub const MOUSE_LEFT_RIGHT: usize = 10;
    pub const MOUSE_UP_DOWN: usize = 11;
    pub const MOUSE_SELECT: usize = 12;
    pub const PAUSE: usize = 13;
    pub const ONLINE: usize = 14;
    pub const SKIP: usize = 15;
    pub const MENU_OPTION1: usize = 16;
    pub const MENU_OPTION2: usize = 17;
    pub const MENU_L1: usize = 18;
    pub const MENU_L2: usize = 19;
    pub const MENU_L3: usize = 20;
    pub const MENU_R1: usize = 21;
    pub const MENU_R2: usize = 22;
    pub const MENU_R3: usize = 23;
    pub const CAMERA_ROTATE_HORIZ: usize = 24;
    pub const CAMERA_ROTATE_VERT: usize = 25;
    pub const COUNT: usize = 26;
}

/// one frame of frontend input for both controllers.
#[derive(Clone, Default)]
pub struct FeInput {
    pub pads: [[ActionState; action::COUNT]; 2],
    /// port: PC keyboard keys typed this frame (the on-screen keyboard takes them)
    pub typed: Vec<crate::feck2::screens::common::fe_keyboard_overlay::PcKey>,
}

/// `g_AttractMode` (`byte_6F5A28`, core's `CAttractMode`) and the game globals its front end calls touch. The plugin
/// copies core's resources in before the frontend update and back after it; `mode` is None without game flow.
#[derive(Default)]
pub struct AttractLink {
    pub mode: Option<velocity_game::font_helper::CAttractMode>,
    /// GameManager's CRT rand state (0x000B34D0 seeds and draws from it)
    pub rand: visos::handy::crt_rand::CrtRand,
    /// GameManager +0x190's timer (ms), the `srand` seed
    pub seed: u32,
    /// GameVars 0x000C2740: `g_GameJump` (`byte_6F9E60`) +0 active
    pub jump_active: bool,
    /// `CGameJump::Set` of 0x000B3550 (applied to core's jump by the plugin)
    pub jump: Option<velocity_game::font_helper::AttractRace>,
    /// `CGameJump::Execute` went to the LoadingScreen: core runs the jump itself, so its 10th frame raises nothing
    pub loading_jump: bool,
    /// 0x000B34D0 ran (`rand` was seeded and drawn from)
    pub started: bool,
}

/// the whole frontend: `CFrontendManager` plus the arenas the FECK2 objects live in and the resources they use.
pub struct Feck2 {
    pub screens: Vec<CScreen>,               // +148 (list)
    pub layers: Vec<CLayer>,
    pub elements: Vec<Option<Element>>,
    globals: [Global; EE_COUNT],             // +144 CFunctionMapper
    pub current: Option<ScreenId>,           // +152
    pub state: u32,                          // +156
    pub popup: Option<ScreenId>,             // +160
    pub pending: Option<ScreenId>,           // +168
    pub flag136: bool,                       // +136 first update after a screen change
    pub stopwatch: f32,                      // +120 CStopWatch seconds
    pub event_time: f32,                     // +124
    pub last_time: f32,                      // +128
    pub time: f32,                           // +132
    pub history: [u32; SCREEN_HISTORY + 1],  // dword_6F6F70
    pub history_level: usize,                // dword_6F6FC0
    pub last_screen_crc: u32,                // dword_6F6FCC
    pub next_screen_crc: u32,                // dword_6F6FC4 (debug boot screen from main)
    pub car_select_next: Option<(u32, u32)>, // pending FECarShowroomScreen 0x002224B0 `SetNextScreen(player, crc)` on gpQuickRaceCarSelectScreen
    pub input: FeInput,                      // +8 (104 bytes copied in Update)
    pub quick_race: crate::feck2::screens::quick_race::fe_quick_race_text_screen::QuickRaceSettings,
    pub events: velocity_game::event_database::event_database_track_loading::CEventDatabase, // gpEventDatabase tracks
    pub rand_state: u32,                     // CRT rand() seed (sub_3DE...)
    pub post_race_screen: u32,               // GameManager 0x000BE7C0 screen after the race
    pub return_screen: u32,                  // dword_6F6FC8 screen to return to after the race
    pub online_session: bool,                // byte_6F715A
    pub online_locked: bool,                 // byte_6F715D (input locked by the network game)
    pub vehicles: Option<velocity_physics::vehicle_database::vehicle_database::CVehicleDatabase>, // gpVehicleDatabase (dword_74B4A8)
    pub foyer: crate::feck2::screens::common::fe_3d_camera::FoyerState, // gpFE3DCamera (dword_7691B8)
    pub characters: crate::feck2::screens::common::fe_3d_characters::FE3DCharacters, // dword_768C30 character part
    pub foyer_vehicles: crate::feck2::screens::common::fe_3d_vehicles::FE3DVehicles, // dword_768978
    pub garage: crate::feck2::screens::garage::GarageVars, // g_FEGameVariables garage part (cash, setups)
    pub jump_to_car_customisation: bool,     // byte_6F905A (`jumptoscreen CarCustomisationScreen`)
    pub char_rand: visos::handy::crt_rand::CrtRand, // VsRandUnit state used by the character pickers
    queue: VecDeque<CEvent>,
    dispatching: bool,
    pub requests: Vec<FrontendRequest>,
    /// full-screen movies to play (`GameManager__BE9F0` 0x000BE9F0 calls), drained into `movie::MovieCommand`
    pub movies: Vec<String>,
    /// the career ("The Scene"): core's state as the screens read it, requests to core and core's answers
    pub career: crate::feck2::screens::hangout::CareerView,
    /// core's save flow (`XMem` 0x0030EF60 / 0x0030EF50), lent by the bevy side for the frame's update; None
    /// without game flow
    pub save_flow: Option<velocity_game::save_game::save_vars::SaveFlow>,
    /// `dword_7B2D50`: the last polled save status block (`dword_701404` points at it)
    pub save_status: velocity_game::save_game::save_vars::SaveStatus,
    /// `VsAudio::PlayGui*` calls made this frame (played by the bevy side)
    pub sounds: Vec<velocity_audio::GuiSound>,
    /// the player options (`GameVars` globals); `options_changed` asks the bevy side to apply and save them
    pub options: crate::game_options::GameOptions,
    pub options_changed: bool,
    /// port: monitor modes / window size for the PC settings page (filled by `pc_settings::pc_display_info`)
    pub pc_display: crate::pc_settings::PcDisplayInfo,
    /// the licence plate: style `dword_6F71DC` (GamePlayer +0x26C) and text `unk_6F71E0` (VsString); mirrored with
    /// graphics' `LicencePlate` by `lib.rs::plate_sync`, `plate_changed` = written by PlateCustomizationScreen
    pub plate: crate::feck2::screens::options::fe_plate_customization_screen::PlateVars,
    pub plate_changed: bool,
    pub textures: TextureStore,
    pub fonts: CFontHelper,
    pub strings: CStringManager,
    pub overlay: Option<crate::feck2::screens::common::fe_2d_overlay::CFE2DOverlay>,
    /// `gpCommonKeyboard` (`dword_769184`)
    pub keyboard: Option<crate::feck2::screens::common::fe_keyboard_overlay::KeyboardHook>,
    /// `gpTipsScreen` (`dword_767140`): FEMainMenuScreen 0x0023F1F0 sets its tip from other screens
    pub tips_screen: Option<std::sync::Arc<std::sync::Mutex<crate::feck2::screens::common::fe_tips_screen::CFETipsScreen>>>,
    pub draw: DrawList,
    /// `dword_6F6FD0`: the main menu state (0 press start .. 3 menu); a global in the original, so it survives screen
    /// changes and FrontEndReInit (`reset_screens` leaves it). Per frontend here, not per process (tests boot several)
    pub main_menu_state: u32,
    pub attract: AttractLink,
    /// CFrontendManager +113: set by the online results screen's leave (0x00206B10, not ported), cleared by Start
    /// (0x001FF1D5), DummyGameScreen Start and StartEvent; the hangout's Start shows nav 3 GENERAL_RESULTS on it
    pub results_prompt: bool,
    /// `VsAudio` music / ambience calls made by the screens this frame (sent by the bevy side)
    pub music: Vec<FeMusicCall>,
    /// the playlist mirror / requests of the DJ booth screens (`playlist_link`)
    pub playlist: crate::feck2::playlist_link::PlaylistLink,
}

/// the `VsAudio` calls of the hangout ("THE SCENE") Start / Leave / ChangeCharacter
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FeMusicCall {
    /// VsAudio.cpp 0x0004DA80 `FadeMusic(volume, reverb, time)`
    FadeMusic { to: f32, env: f32, time: f32 },
    /// VsAudio.cpp 0x0004A8D0
    StartLoopEvent210,
    /// VsAudio.cpp 0x0004A920
    StopLoopEvent210,
}

/// open a pack by xbox path through the visos file layer.
pub fn open_vcd(path: &str) -> Result<visos::file::Vcd, visos::file::VcdError> {
    visos::file::Vcd::open(path)
}

impl Default for Feck2 {
    fn default() -> Self {
        Self::new()
    }
}

impl Feck2 {
    /// FECK2Manager.cpp 0x000AD300 (ctor) + `Initialise` (0x001FDC20) global mapping.
    pub fn new() -> Self {
        let mut globals = [Global::None; EE_COUNT];
        for (i, g) in globals.iter_mut().enumerate() {
            *g = match i as u32 {
                ev::GLOBAL_PRELOAD => Global::GlobalPreload,
                ev::EXIT => Global::Exit,
                ev::START_SCREEN => Global::StartScreen,
                ev::GOTO_SCREEN => Global::GotoScreen,
                ev::GOTO_PREVIOUS_SCREEN => Global::GotoPreviousScreen,
                ev::GOTO_NEXT_SCREEN => Global::GotoNextScreen,
                _ => Global::Screen,
            };
        }
        Self {
            screens: Vec::new(),
            layers: Vec::new(),
            elements: Vec::new(),
            globals,
            current: None,
            state: 0,
            popup: None,
            pending: None,
            flag136: false,
            stopwatch: 0.0,
            event_time: 0.0,
            last_time: 0.0,
            time: 0.0,
            history: [0; SCREEN_HISTORY + 1],
            history_level: 0,
            last_screen_crc: 0,
            next_screen_crc: 0,
            car_select_next: None,
            input: FeInput::default(),
            quick_race: Default::default(),
            events: Default::default(),
            rand_state: 1,
            post_race_screen: 0,
            return_screen: 0,
            online_session: false,
            online_locked: false,
            vehicles: None,
            foyer: Default::default(),
            characters: Default::default(),
            foyer_vehicles: Default::default(),
            garage: Default::default(),
            jump_to_car_customisation: false,
            char_rand: Default::default(),
            queue: VecDeque::new(),
            dispatching: false,
            requests: Vec::new(),
            movies: Vec::new(),
            career: Default::default(),
            save_flow: None,
            save_status: Default::default(),
            sounds: Vec::new(),
            options: Default::default(),
            options_changed: false,
            pc_display: Default::default(),
            plate: Default::default(),
            plate_changed: false,
            textures: TextureStore::default(),
            fonts: CFontHelper::default(),
            strings: CStringManager::new(),
            overlay: None,
            keyboard: None,
            tips_screen: None,
            draw: DrawList::default(),
            main_menu_state: 0,
            attract: AttractLink::default(),
            results_prompt: false,
            music: Vec::new(),
            playlist: Default::default(),
        }
    }

    /// GameManager.cpp 0x000B5A80 — `FrontEndReInit`'s teardown before `CreateAllScreens` runs again: every
    /// screen, layer and element goes, and with them the common objects CreateAllScreens / CreateCommonScreens
    /// make (overlay 0x00282280, FE3DCamera 0x0026F3D0, FE3DVehicles 0x00258690, FE3DCharacters 0x0025ECD0).
    /// The history, the post race / return screens and the game variables (quick race, garage, options) stay.
    pub fn reset_screens(&mut self) {
        self.screens.clear();
        self.layers.clear();
        self.elements.clear();
        self.current = None;
        self.state = 0;
        self.popup = None;
        self.pending = None;
        self.flag136 = false;
        self.stopwatch = 0.0;
        self.event_time = 0.0;
        self.last_time = 0.0;
        self.time = 0.0;
        self.car_select_next = None;
        self.queue.clear();
        self.dispatching = false;
        self.overlay = None;
        self.keyboard = None;
        self.tips_screen = None;
        self.foyer = Default::default();
        self.characters = Default::default();
        self.foyer_vehicles = Default::default();
        self.draw = DrawList::default();
    }

    /// load the string index + the frontend sections and the frontend fonts.
    pub fn load_resources(&mut self, language: &str) {
        self.strings.set_language(language);
        if let Err(e) = self.strings.load_index_file("StrIndex.idx") {
            log::error!("string index: {:?}", e);
        }
        let sections: Vec<String> = self.strings.index.as_ref().map(|i| i.sections.iter().map(|s| s.name.clone()).collect()).unwrap_or_default();
        for s in sections {
            let stem = s.trim_end_matches(".sbk").to_string();
            if self.strings.load_section_name(&stem).is_err() {
                log::debug!("string section {} not loaded", s);
            }
        }
        self.fonts.load_frontend_fonts(&mut self.textures);
        // EventDatabaseTrackLoading.cpp 0x000948D0 from GameManager init: the frontend's track/race lists
        if !self.events.load_tracks(r"\FE") {
            log::warn!("event database: no tracks");
        }
        // the car select screen reads the vehicle database (game owns gpVehicleDatabase; the frontend keeps its own)
        match velocity_physics::vehicle_database::vehicle_database::CVehicleDatabase::load() {
            Ok(db) => self.vehicles = Some(db),
            Err(e) => log::warn!("vehicle database: {:?}", e),
        }
        // GamePlayer.cpp 0x000C1930 garage part, then GameManager::InitDatabases' 0x002022F0 setups
        self.garage.set_initial_variables(&mut self.quick_race.garage, &mut self.quick_race.garage_selected);
        if let Some(db) = self.vehicles.as_ref() {
            self.garage.init_setups(db, &mut self.quick_race.garage);
        }
    }

    // ---- arena helpers ----------------------------------------------------------------------------------------

    pub fn el(&mut self, id: ElementId) -> &mut Element {
        self.elements[id].as_mut().expect("element")
    }

    pub fn e2d(&mut self, id: ElementId) -> &mut crate::feck2::feck2_elements::feck2_element_2d::CElement2D {
        self.el(id).as_2d().expect("CElement2D")
    }

    pub fn text(&mut self, id: ElementId) -> &mut crate::feck2::feck2_elements::feck2_element_static_text::CElementStaticText {
        self.el(id).as_text().expect("CElementStaticText")
    }

    /// FECK2Manager.cpp 0x00202110 — `CreateScreen`
    pub fn create_screen(&mut self, d: &SScreenDesc) -> ScreenId {
        self.screens.push(CScreen { crc: d.crc, layers: Vec::new() });
        self.screens.len() - 1
    }

    /// FECK2Object.cpp 0x00284040 — `CScreen::CreateLayer`: init (fires the assign functor), then sorted insert.
    pub fn create_layer(&mut self, screen: Option<ScreenId>, d: &SLayerDesc) -> LayerId {
        let layer = CLayer::new(d);
        self.layers.push(layer);
        let id = self.layers.len() - 1;
        // CLayer::Init: event 1 then event 0 are processed straight away
        if d.on_assign.is_some() {
            self.layer_process_event(id, &CEvent::new(ev::ASSIGN_EVENTS, -1, 0, 0));
        }
        if d.on_none.is_some() {
            self.layer_process_event(id, &CEvent::new(ev::NONE, -1, 0, 0));
        }
        if let Some(s) = screen {
            self.add_layer(s, id);
        }
        id
    }

    /// FECK2Object.cpp 0x00284180 — `CScreen::AddLayer`: unique ascending priorities.
    fn add_layer(&mut self, s: ScreenId, id: LayerId) {
        let mut prio = self.layers[id].priority;
        let list = self.screens[s].layers.clone();
        let mut pos = list.len();
        for (i, &l) in list.iter().enumerate() {
            let p = self.layers[l].priority;
            if p == prio {
                prio += 1;
            } else if p > prio {
                pos = i;
                break;
            }
        }
        self.layers[id].priority = prio;
        self.layers[id].screen = Some(s);
        self.layers[id].flags = 3;
        self.screens[s].layers.insert(pos, id);
    }

    /// FECK2Object.cpp 0x002864E0 — `CLayer::CreateElement`
    pub fn create_element(&mut self, layer: LayerId, d: &SElementDesc) -> ElementId {
        let mut e = create_element(d.kind);
        match &mut e {
            Element::E2D(x) => x.init(d),
            Element::Line(x) => x.init(d),
            Element::Text(x) => x.init(d),
            Element::Stub(x) => x.base.init(d),
        }
        self.elements.push(Some(e));
        let id = self.elements.len() - 1;
        // CLayer::AddElement: prepend
        self.layers[layer].elements.insert(0, id);
        id
    }

    /// `CLayer::DeleteElement`: unlink from its layer and free it.
    pub fn delete_element(&mut self, id: ElementId) {
        for l in &mut self.layers {
            l.elements.retain(|&e| e != id);
        }
        self.elements[id] = None;
    }

    /// FECK2Object.cpp 0x00286800 — map a functor on a layer (`CLayer::MapEvent`).
    pub fn map(&mut self, layer: LayerId, h: Handler, event: u32) {
        self.layers[layer].map_event(h, event, false);
    }

    /// FECK2Object.cpp 0x002920F0 / 0x00286D30 / FECK2Element2DLine.cpp 0x00293690 — slot 3 `Load`
    pub fn element_load(&mut self, id: ElementId) -> bool {
        let Self { elements, textures, fonts, .. } = self;
        match elements[id].as_mut() {
            Some(Element::E2D(e)) => e.load(textures),
            Some(Element::Text(e)) => e.load(fonts),
            Some(Element::Line(_)) => true, // 0x00293690
            _ => false,
        }
    }

    pub fn element_set_position(&mut self, id: ElementId, x: f32, y: f32, z: f32) {
        self.el(id).base_mut().set_position(x, y, z);
    }

    /// slot 9 — `Fade`
    pub fn element_fade(&mut self, id: ElementId, pct: i32, time: i32, delay: i32) -> bool {
        match self.el(id) {
            Element::E2D(e) => e.fade(pct, time, delay),
            Element::Text(e) => e.fade(pct, time, delay),
            Element::Line(_) | Element::Stub(_) => false,
        }
    }

    /// FECK2Object.cpp 0x002866B0 — `CLayer::FadeAll`: with a time, start a timed fade on every element; with
    /// none, step each one and return true once none of them changed any more.
    pub fn layer_fade_all(&mut self, layer: LayerId, percent: i32, time: i32) -> bool {
        let mut done = true;
        let ids = self.layers[layer].elements.clone();
        for id in ids {
            if time != 0 {
                self.element_fade(id, percent, time, 0);
            } else if self.element_fade(id, percent, 0, 0) {
                done = false;
            }
        }
        done
    }

    /// `CElement2D::ChangeTexture` through the store.
    pub fn change_texture(&mut self, id: ElementId, name: &str, release: bool) {
        let Self { elements, textures, .. } = self;
        if let Some(Some(Element::E2D(e))) = elements.get_mut(id) {
            e.change_texture(name, release, textures);
        }
    }

    pub fn text_ctx(&self) -> crate::feck2::feck2_elements::feck2_element_static_text::TextCtx<'_> {
        crate::feck2::feck2_elements::feck2_element_static_text::TextCtx { fonts: &self.fonts, strings: &self.strings, tex: &self.textures }
    }

    /// `CElementStaticText::GetWidth` (extent of the current string)
    pub fn text_width(&mut self, id: ElementId) -> u32 {
        let e = match self.elements[id].as_ref() {
            Some(Element::Text(t)) => t.clone(),
            _ => return 0,
        };
        e.get_extent_width(&self.text_ctx())
    }

    /// FECK2Manager.cpp 0x00201870 — `GetPreviousScreenCRC`: `dword_6F6F6C[level]`, i.e. the last screen gone
    /// to (`history` is `dword_6F6F70`), 0 at level 0.
    pub fn previous_screen_crc(&self) -> u32 {
        if self.history_level != 0 { self.history[self.history_level - 1] } else { 0 }
    }

    /// `CElementStaticText::GetHeight`
    pub fn text_height(&mut self, id: ElementId) -> u32 {
        let e = match self.elements[id].as_ref() {
            Some(Element::Text(t)) => t.clone(),
            _ => return 0,
        };
        e.get_height(&self.text_ctx())
    }

    // ---- events -----------------------------------------------------------------------------------------------

    /// FECK2Manager.cpp 0x00201710 — `PostEvent` (global mapper dispatch); queued while a handler runs.
    pub fn post_event(&mut self, e: CEvent) {
        self.queue.push_back(e);
        if self.dispatching {
            return;
        }
        self.dispatching = true;
        while let Some(e) = self.queue.pop_front() {
            self.dispatch_global(&e);
        }
        self.dispatching = false;
    }

    /// FECK2Manager.cpp 0x00202A10 — `CFunctionMapper::Dispatch` (global)
    fn dispatch_global(&mut self, e: &CEvent) -> bool {
        match self.globals[e.event as usize] {
            Global::Screen => self.on_screen_event(e),
            Global::GlobalPreload => {
                self.on_global_preload();
                true
            }
            Global::Exit => {
                self.on_exit(e);
                true
            }
            Global::StartScreen => {
                self.on_start_screen(e);
                true
            }
            Global::GotoScreen => {
                self.on_goto_screen(e);
                true
            }
            Global::GotoPreviousScreen => {
                self.on_goto_previous_screen(e);
                true
            }
            Global::GotoNextScreen => {
                self.on_goto_next_screen(e);
                true
            }
            Global::None => true,
        }
    }

    /// FECK2Manager.cpp 0x002018B0 — `OnScreenEvent`
    fn on_screen_event(&mut self, e: &CEvent) -> bool {
        if let Some(c) = self.current {
            let r = self.screen_process_event(c, e);
            if !r {
                if let Some(p) = self.popup {
                    return self.screen_process_event(p, e);
                }
            }
            return r;
        }
        true
    }

    /// FECK2Object.cpp 0x00286930 — `CLayer::ProcessEvent`: the mapped functor's result, true if none.
    pub fn layer_process_event(&mut self, l: LayerId, e: &CEvent) -> bool {
        let slot = e.event as usize;
        let Some(h) = self.layers[l].handlers[slot].clone() else { return true };
        let was = std::mem::replace(&mut self.dispatching, true);
        let r = h(self, e);
        self.dispatching = was;
        r
    }

    /// FECK2Object.cpp 0x00283D00 / FE3DCharacters.cpp 0x00268160 — `CScreen::ProcessEvent` (+ the keyboard's part)
    pub fn screen_process_event(&mut self, s: ScreenId, e: &CEvent) -> bool {
        let mut r = true;
        let (pre, menu_layer, top_layer) = match &self.overlay {
            Some(o) => (o.pre_enabled, o.menu_layer, o.layer),
            None => (false, 0, 0),
        };
        if pre {
            r = self.layer_process_event(menu_layer, e);
        }
        if !r {
            return r;
        }
        // 0x00283D5F: the keyboard overlay (dword_769184 +5) — top, keys, backing (0x00268160)
        if let Some(k) = self.keyboard.as_ref().filter(|k| k.active).map(|k| k.layers) {
            r = self.layer_process_event(k[2], e);
            if r {
                r = self.layer_process_event(k[1], e);
                if r {
                    r = self.layer_process_event(k[0], e);
                }
            }
            if !r {
                return r;
            }
        }
        let layers = self.screens[s].layers.clone();
        for l in layers {
            if self.layers[l].is_active() {
                r = self.layer_process_event(l, e);
                if !r {
                    return r;
                }
            }
        }
        // the overlay flag is read again: a screen's Enter attaches the overlay
        let (pre, top_layer) = match &self.overlay {
            Some(o) => (o.pre_enabled, o.layer),
            None => (false, top_layer),
        };
        if pre {
            r = self.layer_process_event(top_layer, e);
        }
        r
    }

    /// FECK2Manager.cpp 0x00201940 — `OnStartScreen`
    fn on_start_screen(&mut self, e: &CEvent) {
        let new = e.param_b as ScreenId;
        self.pending = Some(new);
        let pending_state = 1;
        self.screen_process_event(new, &CEvent::new(ev::ENTER, -1, 0, 0));
        self.screen_process_event(new, &CEvent::new(ev::POST_ENTER, -1, 0, 0));
        self.flag136 = true;
        if let Some(c) = self.current {
            self.screen_process_event(c, &CEvent::new(ev::LEAVE, -1, 0, 0));
            self.state = 4;
            self.last_screen_crc = self.screens[c].crc;
        } else {
            self.state = 5;
        }
        self.screen_process_event(new, &CEvent::new(ev::ACTIVATE, -1, 0, 0));
        self.screen_process_event(new, &CEvent::new(ev::START, -1, 0, 0));
        self.current = Some(new);
        self.state = pending_state;
        self.state = 3;
        self.pending = None;
    }

    /// FECK2Manager.cpp 0x00201AD0 — `OnExit`
    fn on_exit(&mut self, e: &CEvent) {
        let s = e.param_b as ScreenId;
        if s < self.screens.len() {
            self.screen_process_event(s, &CEvent::new(ev::EXIT_SCREEN, -1, 0, 0));
        }
        self.requests.push(FrontendRequest::Exit);
    }

    /// FECK2Manager.cpp 0x001FF310 — `FindScreen`
    pub fn find_screen(&self, crc: u32) -> Option<ScreenId> {
        self.screens.iter().position(|s| s.crc == crc)
    }

    /// FECK2Manager.cpp 0x00201B40 — `OnGotoScreen`: push history, post StartScreen.
    fn on_goto_screen(&mut self, e: &CEvent) {
        let crc = e.param_b;
        let level = self.history_level;
        assert!(level < SCREEN_HISTORY, "level<SCREEN_HISTORY");
        match self.find_screen(crc) {
            Some(s) => {
                self.history[level] = self.screens[s].crc;
                self.history_level += 1;
                self.post_event(CEvent::new(ev::START_SCREEN, -1, 0, s as u32));
            }
            None => log::error!("ERROR - Screen not found  ... check case:  CRC({:08x})", crc),
        }
    }

    /// FECK2Manager.cpp 0x00201D50 — `OnGotoPreviousScreen`: back `param_a` levels (1 if 0), or to a named screen.
    fn on_goto_previous_screen(&mut self, e: &CEvent) {
        let mut back = e.param_a as usize;
        if back == 0 {
            back = 1;
        }
        let level = self.history_level;
        assert!(level <= SCREEN_HISTORY, "level<=SCREEN_HISTORY");
        if e.param_b != 0 {
            let crc = e.param_b;
            let mut i = self.history_level;
            while i > 0 {
                if self.history[i] == crc {
                    back = self.history_level - i - 1;
                }
                i -= 1;
            }
        }
        let idx = level.wrapping_sub(back + 1);
        let target = self.history.get(idx).copied().unwrap_or(0);
        self.history_level = idx;
        self.post_event(CEvent::new(ev::GOTO_SCREEN, -1, 0, target));
    }

    /// FECK2Manager.cpp 0x00201E90 — `OnGotoNextScreen`
    fn on_goto_next_screen(&mut self, e: &CEvent) {
        let crc = e.param_b;
        assert!(crc != 0, "screenCRC");
        let mut found = None;
        let mut i = self.history_level.wrapping_sub(1);
        while i < SCREEN_HISTORY && self.history[i] != 0 {
            if self.history[i] == crc {
                found = Some(i);
                break;
            }
            i += 1;
        }
        if let Some(i) = found {
            self.history_level = i;
        }
        self.post_event(CEvent::new(ev::GOTO_SCREEN, -1, 0, crc));
    }

    /// FECK2Manager.cpp 0x00202010 — `OnGlobalPreload`
    fn on_global_preload(&mut self) {
        for s in 0..self.screens.len() {
            self.screen_process_event(s, &CEvent::new(ev::PRELOAD, -1, 0, s as u32));
        }
    }

    /// FECK2Manager.cpp 0x00201750 — `GotoScreen`
    pub fn goto_screen(&mut self, crc: u32) {
        self.post_event(CEvent::new(ev::GOTO_SCREEN, -1, 0, crc));
    }

    pub fn goto_screen_name(&mut self, name: &str) {
        self.goto_screen(string_crc(name));
    }

    /// FECK2Manager.cpp 0x002017B0 — `GotoPreviousScreen`
    pub fn goto_previous_screen(&mut self, levels: u32, crc: u32) {
        self.post_event(CEvent::new(ev::GOTO_PREVIOUS_SCREEN, -1, levels, crc));
    }

    /// FECK2Manager.cpp 0x00201810 — `GotoNextScreen`
    pub fn goto_next_screen(&mut self, crc: u32) {
        self.post_event(CEvent::new(ev::GOTO_NEXT_SCREEN, -1, 0, crc));
    }

    /// FECK2Manager.cpp 0x001FEAF0 — `GetStartScreenCRC`: main menu once a profile is loaded, else language select.
    pub fn get_start_screen_crc(profile_loaded: bool) -> u32 {
        if profile_loaded { string_crc("MainMenuScreen") } else { string_crc("LanguageOptionsScreen") }
    }

    /// FECK2Manager.cpp 0x001FF190 — `Start`: resets and sends Preload + StartScreen to the named screen.
    /// GameManager.cpp 0x000B5A80 — the tail of `FrontEndReInit`: start on the post race screen and cut the
    /// history back to its first entry (the screens are kept here instead of being created again)
    pub fn return_from_race(&mut self) -> bool {
        debug_assert!(self.post_race_screen != 0, "m_dReturnFrontEndScreen!=0");
        let crc = self.post_race_screen;
        let started = self.start(crc);
        let level = self.history_level.min(SCREEN_HISTORY);
        for i in (1..=level).rev() {
            if self.history[i] == crc {
                self.history_level = i + 1;
            }
        }
        started
    }

    pub fn start(&mut self, crc: u32) -> bool {
        self.results_prompt = false; // +113 (0x001FF1D5)
        self.stopwatch = 0.0;
        self.event_time = 0.0;
        self.last_time = 0.0;
        self.time = 0.0;
        if let Some(c) = self.current.take() {
            let _ = c;
        }
        if let Some(_p) = self.popup.take() {}
        let mut started = false;
        for s in 0..self.screens.len() {
            if !started && self.screens[s].crc == crc {
                self.screen_process_event(s, &CEvent::new(ev::PRELOAD, -1, 0, s as u32));
                started = true;
                self.post_event(CEvent::new(ev::START_SCREEN, -1, 0, s as u32));
            }
        }
        started
    }

    // ---- per frame ----------------------------------------------------------------------------------------------

    fn digital(&mut self, pad: usize, a: usize, press: u32, held: u32, release: u32) {
        let s = self.input.pads[pad][a];
        if s.value != 0.0 && s.changed {
            self.post_event(CEvent::new(press, pad as i32, 0, 0));
        }
        if s.value != 0.0 && !s.changed {
            self.post_event(CEvent::new(held, pad as i32, 0, 0));
        }
        if s.value == 0.0 && s.changed {
            self.post_event(CEvent::new(release, pad as i32, 0, 0));
        }
    }

    fn axis(&mut self, pad: usize, a: usize, neg: [u32; 3], pos: [u32; 3]) {
        let s = self.input.pads[pad][a];
        if s.value < -0.1 && s.previous >= -0.1 {
            self.post_event(CEvent::new(neg[0], pad as i32, 0, 0));
        }
        if s.value < -0.1 && s.previous < -0.1 {
            self.post_event(CEvent::new(neg[1], pad as i32, 0, 0));
        }
        if s.value >= -0.1 && s.previous < -0.1 {
            self.post_event(CEvent::new(neg[2], pad as i32, 0, 0));
        }
        if s.value > 0.1 && s.previous <= 0.1 {
            self.post_event(CEvent::new(pos[0], pad as i32, 0, 0));
        }
        if s.value > 0.1 && s.previous > 0.1 {
            self.post_event(CEvent::new(pos[1], pad as i32, 0, 0));
        }
        if s.value <= 0.1 && s.previous > 0.1 {
            self.post_event(CEvent::new(pos[2], pad as i32, 0, 0));
        }
    }

    /// FECK2Manager.cpp 0x001FF370 — `Update`: input -> events for both pads, then the Update(dt) event.
    pub fn update(&mut self, input: &FeInput, now: f32) {
        self.input = input.clone();
        if self.current.is_some() || self.popup.is_some() {
            use action::*;
            for pad in 0..2 {
                self.digital(pad, PAUSE, 49, 50, 51);
                self.digital(pad, ONLINE, 52, 53, 54);
                self.digital(pad, MENU_CANCEL, 55, 56, 57);
                self.digital(pad, MENU_SELECT, 46, 47, 48);
                self.digital(pad, MENU_OPTION1, 58, 59, 60);
                self.digital(pad, MENU_OPTION2, 61, 62, 63);
                self.digital(pad, MENU_L1, 64, 65, 66);
                self.digital(pad, MENU_L2, 67, 68, 69);
                self.digital(pad, MENU_L3, 70, 71, 72);
                self.digital(pad, MENU_R1, 73, 74, 75);
                self.digital(pad, MENU_R2, 76, 77, 78);
                self.digital(pad, MENU_R3, 79, 80, 81);
                self.axis(pad, MENU_LEFT_RIGHT, [31, 32, 33], [25, 26, 27]);
                self.axis(pad, MENU_UP_DOWN, [28, 29, 30], [22, 23, 24]);
                self.digital(pad, MENU_LEFT, 31, 32, 33);
                self.digital(pad, MENU_RIGHT, 25, 26, 27);
                self.digital(pad, MENU_UP, 22, 23, 24);
                self.digital(pad, MENU_DOWN, 28, 29, 30);
                self.axis(pad, CAMERA_ROTATE_HORIZ, [34, 35, 36], [37, 38, 39]);
                self.axis(pad, CAMERA_ROTATE_VERT, [40, 41, 42], [43, 44, 45]);
            }
        }
        // port: PC keyboard typing into the on-screen keyboard
        if !input.typed.is_empty() {
            use crate::feck2::screens::common::fe_keyboard_overlay::kb;
            for &t in &input.typed {
                kb(self, |k, fe| k.active && k.pc_key(fe, t));
            }
        }
        self.stopwatch = now;
        let dt = self.stopwatch - self.event_time;
        if self.event_time != 0.0 {
            self.post_event(CEvent::new(ev::UPDATE, -1, dt.to_bits(), 0));
            if self.flag136 {
                self.flag136 = false;
            }
        }
        self.event_time = self.stopwatch;
    }

    /// FECK2Manager.cpp 0x00202090 — `Render`
    pub fn render(&mut self) {
        self.draw.clear();
        self.last_time = self.time;
        self.time = self.stopwatch;
        if let Some(c) = self.current {
            self.screen_render(c);
        }
        if let Some(p) = self.popup {
            self.screen_render(p);
        }
    }

    /// FECK2Object.cpp 0x00283EE0 / FE3DCharacters.cpp 0x00268200 — `CScreen::Render` (+ the keyboard's part)
    fn screen_render(&mut self, s: ScreenId) {
        let (vis, bg, layer, menu) = match &self.overlay {
            Some(o) => (o.enabled, o.bg_layer, o.layer, o.menu_layer),
            None => (false, 0, 0, 0),
        };
        if vis {
            self.layer_render(bg);
        }
        let layers = self.screens[s].layers.clone();
        for &l in layers.iter().rev() {
            if self.layers[l].is_visible() {
                self.layer_render(l);
            }
        }
        // 0x00283F98: the keyboard (+4) — backing, keys, top, each when visible (0x00268200)
        if let Some(k) = self.keyboard.as_ref().filter(|k| k.shown).map(|k| k.layers) {
            for l in k {
                if self.layers[l].is_visible() {
                    self.layer_render(l);
                }
            }
        }
        if vis {
            self.layer_render(layer);
            self.layer_render(menu);
        }
    }

    /// FECK2Object.cpp 0x002868B0 — `CLayer::Render`
    pub fn layer_render(&mut self, l: LayerId) {
        let dt = self.time - self.last_time;
        let ids = self.layers[l].elements.clone();
        let Self { elements, fonts, strings, textures, draw, .. } = self;
        for id in ids {
            let Some(e) = elements[id].as_mut() else { continue };
            if !e.is_visible() {
                continue;
            }
            match e {
                Element::E2D(x) => {
                    if !enable_2d() {
                        continue;
                    }
                    x.render(dt, draw);
                }
                Element::Line(x) => {
                    if !enable_2d() {
                        continue;
                    }
                    x.render(draw);
                }
                Element::Text(x) => {
                    let ctx = crate::feck2::feck2_elements::feck2_element_static_text::TextCtx { fonts, strings, tex: textures };
                    x.render(dt, &ctx, draw);
                }
                Element::Stub(_) => {}
            }
        }
    }
}
