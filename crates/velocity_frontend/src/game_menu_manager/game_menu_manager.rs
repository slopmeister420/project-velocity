//! src\GameMenuManager\GameMenuManager.cpp — `CGameMenuManager` (`dword_6F4FC0`, a stack of up to 8 menus)
//! and `CGameMenu` / `CGameMenuItem`: the in-race menus (pause, audio, display, quit confirm).
//!
//! A framed menu draws through OSD items in the 1333x1000 OSD space: debug font (FontHelper id 12), a `>` cursor
//! right-justified at x + 10, item text at x + 60, rows 42 apart from the top (y up); the frame (`menuComponents`,
//! title bar 0x000AADF0, 9-slice 0x000AB230) and scrollbar (`sliders`, 0x000ABB60) are not ported. Every in-race
//! menu sets the no-frame flag (+593, 0x000AE0A0(1)), which skips all of that: those menus are drawn by their OSD
//! pause screens (`osd::osd_screens::osd_screens`, hooked in OnEnter / OnLeave).

use crate::feck2::feck2_manager::{action, FeInput};
use crate::font_helper::font_helper::{CFontHelper, FONT_DEBUG};
use crate::osd::osd_manager::{flag, COSDManager, ItemId};
use crate::osd::osd_screens::osd_screens::{COSDPauseScreens, PauseScreen};
use crate::render2d::TextureStore;
use crate::string_manager::string_manager::{string_crc, CStringManager};

/// `CGameMenuItem` (28 bytes, ctor 0x000AC7F0): +8 text buffer, +12 string id, +16 item id, +20 selected,
/// +21 disabled, +24 OSD item.
#[derive(Clone, Debug)]
pub struct CGameMenuItem {
    pub text: Option<String>,
    pub string: u32,
    pub id: i32,
    pub selected: bool,
    pub disabled: bool,
}

impl CGameMenuItem {
    /// GameMenuManager.cpp 0x000AC7F0
    pub fn new(string: u32, id: i32) -> Self {
        Self { text: None, string, id, selected: false, disabled: false }
    }
}

/// which concrete menu (the vtable of the `CGameMenu` subclass).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuKind {
    Pause,
    Audio,
    Display,
    Quit { restart: bool },
    /// `CRaceOverMenu` (GameTweaksMenu.cpp, vtable 0x0062F3E4): the post race menu GameManager::Update mode 2 enters
    RaceOver(RaceOverEvent),
}

/// what `CRaceOverMenu` reads of the event (`dword_6F9BE8`) and the game.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RaceOverEvent {
    /// `byte_6F7158` network game
    pub online: bool,
    /// event +0x15 & 2 (`EVENT_FLAG_COMPLETED`)
    pub completed: bool,
    /// event +0x15 & 4 (`EVENT_FLAG_FAILED`)
    pub failed: bool,
    /// current race type 5 (drag)
    pub drag: bool,
}

/// `CGameMenu` (1116+ bytes, ctor 0x000A9EF0)
pub struct CGameMenu {
    pub kind: MenuKind,
    pub name: String,          // +4 (32 chars)
    pub description: u32,      // +36
    pub items: Vec<CGameMenuItem>, // +40 (max 128), count +552
    pub selected: usize,       // +556
    pub x: f32,                // +564
    pub y: f32,                // +568
    pub width: f32,            // +572
    pub height: f32,           // +576
    pub first_visible: usize,  // +580
    pub visible: usize,        // +584
    pub repeat: f32,           // +588 auto repeat timer
    pub dirty: bool,           // +592
    pub no_frame: bool,        // +593
    pub osd_items: Vec<ItemId>, // +596 (count +1108)
    pub closing: bool,         // +1113
    pub rumble_item: Option<usize>, // +1116 (pause menu)
}

/// what the menus ask of the game (Bevy message `PauseMenuRequest` in the plugin).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseMenuRequest {
    Resume,            // GameManager 0x000BC2C0
    Restart,           // GameTweaksMenu 0x000B12A0
    QuitToFrontend,    // GameManager 0x000BBF90(0)
    OnlineOptions,     // CFrontendManager::Start("OnlineMenuScreen")
    RumbleChanged(bool),
    OptionsChanged,    // byte_6D45C1: audio settings dirty
    /// GameTweaksMenu 0x000B17A0: `byte_6F5D09` = quit_event, LeaveAll, SetGameMode(3) (continue / leave the event)
    EndOfGame { quit_event: bool },
}

pub use crate::game_options::GameOptions;

const VOLUME_ITEMS: [(&str, &str); 6] = [
    ("MENU_AUDIO_MASTER_VOL", "dMasterVol"),
    ("MENU_AUDIO_MUSIC_VOL", "dMusicVol"),
    ("MENU_AUDIO_ENGINE_VOL", "dEngineVol"),
    ("MENU_AUDIO_SFX_VOL", "dSFXVol"),
    ("MENU_AUDIO_SPEECH_VOL", "dSpeechVol"),
    ("MENU_AUDIO_TYRES_VOL", "dTyresVol"),
];
const DISPLAY_ITEMS: [(&str, &str); 6] = [
    ("OPTION_VIDEO_RATIO_SINGLE_LINE", "idScreenRatio"),
    ("OPTION_VIDEO_TFT_SINGLE_LINE", "idTFTDisplay"),
    ("OPTION_VIDEO_PLAYERNAME_SINGLE_LINE", "idNameDisplay"),
    ("OPTION_VIDEO_JUMPCAM_SINGLE_LINE", "idJumpDisplay"),
    ("OPTION_VIDEO_CRASHCAM_SINGLE_LINE", "idCrashDisplay"),
    ("OPTION_VIDEO_POLICECAM_SINGLE_LINE", "idPoliceDisplay"),
];

impl CGameMenu {
    /// GameMenuManager.cpp 0x000A9EF0
    fn new(kind: MenuKind, name: &str) -> Self {
        Self {
            kind,
            name: name.chars().take(31).collect(),
            description: 0,
            items: Vec::new(),
            selected: 0,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            first_visible: 0,
            visible: 0,
            repeat: 0.0,
            dirty: true,
            no_frame: false,
            osd_items: Vec::new(),
            closing: false,
            rumble_item: None,
        }
    }

    /// GameMenuManager.cpp 0x000AA160 — `AddItem` (128 max)
    fn add(&mut self, item: CGameMenuItem) {
        if self.items.len() <= 0x80 {
            self.items.push(item);
        }
    }

    /// GameMenuManager.cpp 0x000AD0E0 / 0x000AD110
    fn set_size(&mut self, w: f32, h: f32) {
        self.width = w;
        self.height = h;
        self.dirty = true;
    }

    /// GameMenuManager.cpp 0x000AF070 — `CGamePauseMenu` ctor
    pub fn pause(opts: &GameOptions, strings: &mut CStringManager) -> Self {
        let mut m = Self::new(MenuKind::Pause, "Pause");
        m.add(CGameMenuItem::new(string_crc("MENU_RESUME"), 1));
        m.add(CGameMenuItem::new(string_crc("MENU_AUDIO"), 2));
        m.add(CGameMenuItem::new(string_crc("OPTION_VIDEO_TITLE"), 3));
        m.rumble_item = Some(m.items.len());
        m.add(CGameMenuItem::new(string_crc("OPTION_RUMBLE"), 4));
        m.add(CGameMenuItem::new(string_crc("MENU_RESTART"), 5));
        m.add(CGameMenuItem::new(string_crc("MENU_QUIT_TO_FE"), 6));
        m.add(CGameMenuItem::new(string_crc("ONLINE_OPTIONS"), 7));
        m.refresh(opts, strings);
        m.set_size(800.0, 600.0);
        m.no_frame = true;
        m
    }

    /// GameMenuManager.cpp 0x000AD940 — `CGameAudioMenu`
    pub fn audio(opts: &GameOptions, strings: &mut CStringManager) -> Self {
        let mut m = Self::new(MenuKind::Audio, "Audio");
        for (i, (s, _)) in VOLUME_ITEMS.iter().enumerate() {
            m.add(CGameMenuItem::new(string_crc(s), i as i32 + 1));
        }
        m.add(CGameMenuItem::new(string_crc("MENU_AUDIO_AUDIO_MODE"), 7));
        m.add(CGameMenuItem::new(string_crc("MENU_DONE"), 8));
        m.refresh(opts, strings);
        m.set_size(800.0, 600.0);
        m.no_frame = true;
        m
    }

    /// GameMenuManager.cpp 0x000AE880 — `CGameDisplayMenu` (named "Audio" in the original too)
    pub fn display(opts: &GameOptions, strings: &mut CStringManager) -> Self {
        let mut m = Self::new(MenuKind::Display, "Audio");
        for (i, (s, _)) in DISPLAY_ITEMS.iter().enumerate() {
            m.add(CGameMenuItem::new(string_crc(s), i as i32));
        }
        m.add(CGameMenuItem::new(string_crc("MENU_DONE"), 6));
        m.set_size(800.0, 600.0);
        m.refresh(opts, strings);
        m.no_frame = true;
        m
    }

    /// GameTweaksMenu.cpp 0x000B0DB0 — `CQuitMenu` (event type 2 cup / 3 league texts not distinguished here)
    pub fn quit(restart: bool) -> Self {
        let mut m = Self::new(MenuKind::Quit { restart }, "Quit");
        m.description = string_crc(if restart { "CONFIRM_RESTART_TEXT" } else { "CONFIRM_QUIT_TEXT" });
        m.add(CGameMenuItem::new(string_crc("GENERAL_YES"), 1));
        m.add(CGameMenuItem::new(string_crc("GENERAL_NO"), 2));
        m.set_size(800.0, 600.0);
        m.no_frame = true;
        m
    }

    /// GameTweaksMenu.cpp 0x000B1450 — `CRaceOverMenu` ctor ("RaceOver"): restart (not online), return to the front
    /// end, continue (unless the event is completed / failed or a drag). The OSD mask part is in `enter_race_over`.
    pub fn race_over(ev: RaceOverEvent) -> Self {
        let mut m = Self::new(MenuKind::RaceOver(ev), "RaceOver");
        if !ev.online {
            m.add(CGameMenuItem::new(string_crc("MENU_RESTART_EVENT"), 1));
        }
        m.add(CGameMenuItem::new(string_crc("MENU_RETURN_TO_FE"), 2));
        if !(ev.completed || ev.failed || ev.drag) {
            m.add(CGameMenuItem::new(string_crc("MENU_CONTINUE_EVENT"), 3));
        }
        m.set_size(f32::from_bits(0x44480000), f32::from_bits(0x44160000));
        m.no_frame = true;
        m
    }

    /// GameMenuManager.cpp 0x000AA110 — `SetSelection`: ignored for 0 or past the end
    fn set_selection(&mut self, n: usize) {
        if n != 0 && n < self.items.len() {
            self.selected = n;
        }
    }

    /// GamePauseMenu.cpp 0x000AF740 / 0x000AD480 / 0x000AE310 — refresh the item string arguments.
    pub fn refresh(&mut self, o: &GameOptions, strings: &mut CStringManager) {
        let on_off = |b: bool| string_crc(if b { "OPTION_ON" } else { "OPTION_OFF" });
        match self.kind {
            MenuKind::Pause => {
                strings.set_argument(string_crc("OPTION_RUMBLE"), string_crc("idRumble"), on_off(o.rumble)).ok();
            }
            MenuKind::Audio => {
                for (i, (s, a)) in VOLUME_ITEMS.iter().enumerate() {
                    strings.set_argument(string_crc(s), string_crc(a), (o.volumes[i] * 100.0) as i32 as u32).ok();
                }
                let mode = match o.audio_mode {
                    0 => "AUDIO_MODE_STEREO",
                    1 => "AUDIO_MODE_SURROUND",
                    _ => "AUDIO_MODE_MONO",
                };
                strings.set_argument(string_crc("MENU_AUDIO_AUDIO_MODE"), string_crc("idAudioMode"), string_crc(mode)).ok();
            }
            MenuKind::Display => {
                let ratio = string_crc(if o.display[0] { "OPTION_VIDEO_RATIO_WIDE" } else { "OPTION_VIDEO_RATIO_STANDARD" });
                strings.set_argument(string_crc(DISPLAY_ITEMS[0].0), string_crc(DISPLAY_ITEMS[0].1), ratio).ok();
                for i in 1..6 {
                    strings.set_argument(string_crc(DISPLAY_ITEMS[i].0), string_crc(DISPLAY_ITEMS[i].1), on_off(o.display[i])).ok();
                }
            }
            MenuKind::Quit { .. } | MenuKind::RaceOver(_) => {}
        }
        self.dirty = true;
    }

    /// GameMenuManager.cpp 0x000AC640 — release the menu's OSD items.
    fn release(&mut self, osd: &mut COSDManager) {
        for id in self.osd_items.drain(..) {
            osd.release(id);
        }
    }

    /// GameMenuManager.cpp 0x000AAC30 / 0x000AC570 / 0x000AC860 — build the OSD items (+ AddCursor, item Build): cursor and the visible rows. Everything is inside the
    /// no-frame check, so the in-race menus (all no-frame) draw nothing here: their OSD pause screens do.
    fn build(&mut self, osd: &mut COSDManager, fonts: &CFontHelper, strings: &CStringManager, tex: &TextureStore) {
        self.release(osd);
        if self.no_frame {
            return;
        }
        // frame / title / scrollbar (0x000AADF0, 0x000AB230, 0x000ABB60) not ported
        let x = self.x + 10.0;
        let n = self.visible.min(self.items.len());
        for i in 0..n {
            let index = self.first_visible + i;
            let Some(item) = self.items.get(index) else { continue };
            let y = 1000.0 - (self.y + 10.0 + i as f32 * 40.0 + i as f32 + i as f32);
            if index == self.selected {
                // 0x000AC570: '>' right-justified
                let id = osd.create_text_item(fonts, tex, ">", FONT_DEBUG, x, y, 900, -1.0, flag::JUSTIFY_RIGHT, 4096);
                self.osd_items.push(id);
            }
            // 0x000AC860: item text, left-justified, 50 right of the cursor
            let id = match &item.text {
                Some(t) => Some(osd.create_text_item(fonts, tex, t, FONT_DEBUG, x + 50.0, y, 900, -1.0, flag::JUSTIFY_LEFT, 4096)),
                None => osd.create_string_item(fonts, strings, tex, item.string, FONT_DEBUG, x + 50.0, y, 900, -1.0, flag::JUSTIFY_LEFT, 4096),
            };
            if let Some(id) = id {
                self.osd_items.push(id);
            }
        }
    }
}

/// `CGameMenuManager`
#[derive(Default)]
pub struct CGameMenuManager {
    pub stack: Vec<CGameMenu>, // +4 (8 slots), count +36, top +40
    pub paused: bool,          // +52 (pause button state)
    pub options: GameOptions,
    pub requests: Vec<PauseMenuRequest>,
    /// `g_OSDPauseScreens` (the menus' OnEnter / OnLeave point them at the menu)
    pub screens: COSDPauseScreens,
    /// GameManager mode (`FCF0::VsAudioTrackList(&unk_6F5BC8)`, CQuitMenu's dtor 0x000B10B0 reads it), set by the hud
    pub game_mode: u8,
    /// `byte_6F7158` network game
    pub online: bool,
}

/// set / clear bits of the game OSD manager's mask (`VsCrashCamera__63030` / `VsAIRaceVehicle__23D00`)
fn set(osd: &mut COSDManager, m: u32) {
    osd.hidden_mask |= m;
}
fn clear(osd: &mut COSDManager, m: u32) {
    osd.hidden_mask &= !m;
}

impl CGameMenuManager {
    /// GameMenuManager.cpp 0x000A9A40 — `EnterMenu` (with the mask part of the menu's ctor, which runs right before)
    pub fn enter(&mut self, mut menu: CGameMenu, osd: &mut COSDManager) {
        match menu.kind {
            // CGamePauseMenu 0x000AF070, CGameAudioMenu 0x000AD940, CGameDisplayMenu 0x000AE880
            MenuKind::Pause => set(osd, 8),
            MenuKind::Audio | MenuKind::Display => set(osd, 4),
            // CQuitMenu 0x000B0DB0
            MenuKind::Quit { .. } => {
                set(osd, 4);
                set(osd, 8);
                set(osd, 256);
            }
            MenuKind::RaceOver(_) => {}
        }
        self.enter_menu_only(&mut menu, osd);
        self.stack.push(menu);
        self.on_enter(osd);
    }

    /// GameMenuManager.cpp 0x000AF4A0 / 0x000AEC40 / 0x000ADD90 / GameTweaksMenu.cpp 0x000B1120 / 0x000B16E0 — vtable
    /// +12 `OnEnter` of the new top menu (pause, display, audio, quit, race over after its selection)
    fn on_enter(&mut self, osd: &mut COSDManager) {
        let i = self.stack.len() - 1;
        let s = &mut self.screens.screens;
        match self.stack[i].kind {
            MenuKind::Pause => s[PauseScreen::Pause as usize].enter(i),
            MenuKind::Display => {
                s[PauseScreen::Display as usize].enter(i);
                clear(osd, 64);
                set(osd, 4);
            }
            MenuKind::Audio => {
                s[PauseScreen::Audio as usize].enter(i);
                clear(osd, 128);
                set(osd, 4);
            }
            MenuKind::Quit { .. } => {
                s[PauseScreen::Quit as usize].enter(i);
                clear(osd, 2048);
                set(osd, 4);
            }
            MenuKind::RaceOver(_) => s[PauseScreen::RaceOver as usize].enter(i),
        }
    }

    /// GameMenuManager.cpp 0x000A9C40 / 0x000AF4D0 / 0x000AEC90 / 0x000AEC00 / 0x000ADDE0 / 0x000ADD50 /
    /// GameTweaksMenu.cpp 0x000B1170 / 0x000B10B0 / 0x000B1770 — vtable +16 `OnLeave`, then the dtor's mask part (0x000A9C40 pops this way)
    fn on_leave(&mut self, kind: MenuKind, osd: &mut COSDManager) {
        let s = &mut self.screens.screens;
        match kind {
            // 0x000AF4D0
            MenuKind::Pause => {
                s[PauseScreen::Pause as usize].leave();
                clear(osd, 8);
            }
            // 0x000AEC90, dtor 0x000AEC00
            MenuKind::Display => {
                s[PauseScreen::Display as usize].leave();
                set(osd, 64);
                clear(osd, 4);
                clear(osd, 4);
            }
            // 0x000ADDE0, dtor 0x000ADD50
            MenuKind::Audio => {
                s[PauseScreen::Audio as usize].leave();
                set(osd, 128);
                clear(osd, 4);
                clear(osd, 4);
            }
            // 0x000B1170, dtor 0x000B10B0 (game mode 1 in race, 2 post race)
            MenuKind::Quit { .. } => {
                s[PauseScreen::Quit as usize].leave();
                set(osd, 2048);
                clear(osd, 4);
                match self.game_mode {
                    1 => clear(osd, 4),
                    2 => clear(osd, 256),
                    _ => {}
                }
            }
            // 0x000B1770
            MenuKind::RaceOver(_) => s[PauseScreen::RaceOver as usize].leave(),
        }
    }

    /// OSDScreens.cpp 0x000FCAF0 — "g_OSDPauseScreens.Update", once per game update
    pub fn update_screens(&mut self, osd: &mut COSDManager, fonts: &CFontHelper, strings: &CStringManager, tex: &TextureStore) {
        let online = self.online;
        self.screens.update(&mut self.stack, osd, fonts, strings, tex, online);
    }

    /// GameManager.cpp 0x000BB1C0 mode 2 (no menu open): `new CRaceOverMenu` (0x000B1450: OSD manager `dword_6FD200`
    /// mask |= 4 | 8, &= ~256) and EnterMenu, whose vtable +12 call (0x000B16E0) picks the selection: online 1,
    /// failed 0 (restart), not completed 2 (continue), else 1 (return to the front end), then shows screen 3.
    pub fn enter_race_over(&mut self, ev: RaceOverEvent, osd: &mut COSDManager) {
        let mut m = CGameMenu::race_over(ev);
        osd.hidden_mask |= 4;
        osd.hidden_mask |= 8;
        osd.hidden_mask &= !256;
        self.enter_menu_only(&mut m, osd);
        if !ev.online {
            if ev.failed {
                m.set_selection(0);
            } else if !ev.completed {
                m.set_selection(2);
            } else {
                m.set_selection(1);
            }
        } else {
            m.set_selection(1);
        }
        self.stack.push(m);
        self.on_enter(osd);
    }

    /// the part of `EnterMenu` before the new menu goes on the stack
    fn enter_menu_only(&mut self, menu: &mut CGameMenu, osd: &mut COSDManager) {
        assert!(self.stack.len() < 8, "too many menus");
        if let Some(top) = self.stack.last_mut() {
            top.release(osd);
            top.dirty = true;
        }
        menu.dirty = true;
    }

    /// GameMenuManager.cpp 0x000A9B70 — close the top menu; true when that was the last one.
    pub fn leave(&mut self, osd: &mut COSDManager) -> bool {
        if let Some(mut m) = self.stack.pop() {
            self.on_leave(m.kind, osd);
            m.release(osd);
        }
        if let Some(top) = self.stack.last_mut() {
            top.dirty = true;
        }
        self.stack.is_empty()
    }

    /// GameMenuManager.cpp 0x000A9BD0 — close every menu.
    pub fn leave_all(&mut self, osd: &mut COSDManager) {
        while !self.leave(osd) {}
    }

    pub fn is_open(&self) -> bool {
        !self.stack.is_empty()
    }

    /// GameMenuManager.cpp 0x000AA3E0 / 0x000A9DF0 — (manager Update ->) `CGameMenu::Update` for the top menu: layout, input, cursor.
    pub fn update(&mut self, input: &FeInput, dt: f32, osd: &mut COSDManager, fonts: &CFontHelper, strings: &mut CStringManager, tex: &TextureStore) {
        let Some(menu) = self.stack.last_mut() else { return };
        let mut changed = false;
        if menu.dirty {
            changed = true;
            menu.visible = menu.items.len().min(13);
            menu.height = menu.visible as f32 * 42.0 + 20.0;
            menu.x = (1333.0 - menu.width) * 0.5;
            menu.y = (1000.0 - menu.height) * 0.5;
            menu.dirty = false;
        }
        let pad = &input.pads[0];
        let pressed = |a: usize| pad[a].value != 0.0 && pad[a].changed;
        let held = |a: usize| pad[a].value != 0.0;
        let mut select = None;
        let mut cancel = false;
        let mut side = 0;
        if pressed(action::MENU_SELECT) {
            select = menu.items.get(menu.selected).filter(|i| !i.disabled).map(|i| i.id);
        } else if pressed(action::MENU_CANCEL) {
            cancel = true;
        } else if pressed(action::MENU_LEFT) || (pad[action::MENU_LEFT_RIGHT].value < 0.0 && pad[action::MENU_LEFT_RIGHT].changed) {
            side = -1;
        } else if pressed(action::MENU_RIGHT) || (pad[action::MENU_LEFT_RIGHT].value > 0.0 && pad[action::MENU_LEFT_RIGHT].changed) {
            side = 1;
        }
        // up / down with a 0.2 s auto repeat
        menu.repeat -= dt;
        let timed = menu.repeat <= 0.0;
        let mut up = held(action::MENU_UP) && (pressed(action::MENU_UP) || timed);
        let mut down = !up && held(action::MENU_DOWN) && (pressed(action::MENU_DOWN) || timed);
        if !up && !down && timed {
            let v = pad[action::MENU_UP_DOWN].value;
            if v > 0.1 {
                up = true;
            } else if v < -0.1 {
                down = true;
            }
        }
        if up || down {
            menu.repeat = 0.2;
            let n = menu.items.len();
            if n > 0 {
                if up {
                    if menu.selected != 0 {
                        menu.selected -= 1;
                        if menu.selected < menu.first_visible {
                            menu.first_visible -= 1;
                        }
                    } else {
                        menu.selected = n - 1;
                        menu.first_visible = n - menu.visible;
                    }
                } else if menu.selected >= n - 1 {
                    menu.selected = 0;
                    menu.first_visible = 0;
                } else {
                    menu.selected += 1;
                    if menu.selected >= menu.visible + menu.first_visible {
                        menu.first_visible += 1;
                    }
                }
                changed = true;
                log::debug!("game menu {}: selection {} (dt {dt:.3}, {})", menu.name, menu.selected, if pressed(action::MENU_UP) || pressed(action::MENU_DOWN) { "press" } else { "repeat" });
            }
        }
        if let Some(id) = select {
            log::debug!("game menu {}: select id {id}", menu.name);
        }
        for (i, it) in menu.items.iter_mut().enumerate() {
            it.selected = i == menu.selected;
        }
        let kind = menu.kind;
        if changed {
            menu.build(osd, fonts, strings, tex);
        }
        if let Some(id) = select {
            self.on_select(kind, id, osd, strings);
        } else if cancel {
            // 0x000AA3E0 cancel: leave; closing the last one resumes the race
            if self.leave(osd) {
                self.requests.push(PauseMenuRequest::Resume);
            }
        } else if side != 0 {
            self.on_side(kind, side, osd, strings);
        }
        if let Some(top) = self.stack.last_mut() {
            if top.dirty {
                top.visible = top.items.len().min(13);
                top.height = top.visible as f32 * 42.0 + 20.0;
                top.x = (1333.0 - top.width) * 0.5;
                top.y = (1000.0 - top.height) * 0.5;
                top.dirty = false;
                top.build(osd, fonts, strings, tex);
            }
        }
    }

    /// GamePauseMenu.cpp 0x000AF510 / GameMenuManager.cpp 0x000ADE30 / GameTweaksMenu.cpp 0x000B11C0 — vtable +24, item
    /// selected (pause, audio, display, quit)
    fn on_select(&mut self, kind: MenuKind, id: i32, osd: &mut COSDManager, strings: &mut CStringManager) {
        match kind {
            MenuKind::Pause => match id {
                1 => {
                    self.leave_all(osd);
                    self.requests.push(PauseMenuRequest::Resume);
                }
                2 => {
                    let m = CGameMenu::audio(&self.options, strings);
                    self.enter(m, osd);
                }
                3 => {
                    let m = CGameMenu::display(&self.options, strings);
                    self.enter(m, osd);
                }
                4 => {
                    self.options.rumble = !self.options.rumble;
                    self.requests.push(PauseMenuRequest::RumbleChanged(self.options.rumble));
                    let o = self.options.clone();
                    if let Some(m) = self.stack.last_mut() {
                        m.refresh(&o, strings);
                    }
                }
                5 => self.enter(CGameMenu::quit(true), osd),
                6 => self.enter(CGameMenu::quit(false), osd),
                7 => self.requests.push(PauseMenuRequest::OnlineOptions),
                _ => {}
            },
            MenuKind::Audio if id == 8 => {
                self.leave(osd);
            }
            MenuKind::Display if id == 6 => {
                self.leave(osd);
            }
            // GameTweaksMenu.cpp 0x000B17A0
            MenuKind::RaceOver(ev) => match id {
                1 => {
                    if ev.failed {
                        // 0x000B12A0 at once (the confirm menu it made first is dropped)
                        self.leave_all(osd);
                        self.requests.push(PauseMenuRequest::Restart);
                    } else {
                        self.enter(CGameMenu::quit(true), osd);
                    }
                }
                2 if !ev.failed && !ev.completed => self.enter(CGameMenu::quit(false), osd),
                _ => {
                    self.leave_all(osd);
                    self.requests.push(PauseMenuRequest::EndOfGame { quit_event: id == 2 && ev.failed });
                }
            },
            MenuKind::Quit { restart } => {
                if id == 1 {
                    self.leave_all(osd);
                    self.requests.push(if restart { PauseMenuRequest::Restart } else { PauseMenuRequest::QuitToFrontend });
                } else {
                    self.leave(osd);
                }
            }
            _ => {}
        }
    }

    /// GameMenuManager.cpp 0x000ADE80 / 0x000AED30 — vtable +28, left / right on an item (audio, display)
    fn on_side(&mut self, kind: MenuKind, dir: i32, _osd: &mut COSDManager, strings: &mut CStringManager) {
        let Some(id) = self.stack.last().and_then(|m| m.items.get(m.selected)).map(|i| i.id) else { return };
        match kind {
            MenuKind::Audio => {
                let step = if dir >= 0 { 0.05 } else { -0.05 };
                match id {
                    1..=6 => {
                        let v = &mut self.options.volumes[id as usize - 1];
                        *v = (*v + step).clamp(0.0, 1.0);
                    }
                    7 => self.options.audio_mode = (self.options.audio_mode + dir).clamp(0, 2),
                    _ => return,
                }
                self.requests.push(PauseMenuRequest::OptionsChanged);
            }
            MenuKind::Display if (0..6).contains(&id) => {
                let b = &mut self.options.display[id as usize];
                *b = !*b;
            }
            _ => return,
        }
        let o = self.options.clone();
        if let Some(m) = self.stack.last_mut() {
            m.refresh(&o, strings);
        }
    }
}
