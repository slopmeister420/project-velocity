//! src\GameMenuManager\GameMenuManager.cpp — the debug menus of `CGameMenuManager` (`dword_6F4FC0`, up to 8
//! menus): the in-race "Debug" menu (0x000AE160, vtable 0x0062EC6C), the "Front End" menu (0x000ACDD0, vtable
//! 0x0062EA5C) and the tweak menus (`game_tweaks_menu`). Opened by GameInput 0x000A4F10 through GameManager
//! 0x000BC3D0(1) (see `game_manager::update`): BACK (Online) in a race, WHITE (MenuL2) held + BLACK (MenuR2) in the
//! front end. F3_D has no other gate (only attract mode / point of interest / crash camera block the button).
//!
//! Same `CGameMenu` update and build as the front end's port; these menus keep their frame (+593 = 0), so 0x000AAC30
//! builds the title, cursor and rows as OSD text items (debug font 12, mask 4096) into `osd`, which the binary
//! draws (`velocity/src/debug_menu.rs`). The frame / title bar / scrollbar graphics (`menuComponents`, `sliders`)
//! are not drawn.

use bevy::prelude::*;

use crate::cli::tweak::{CTweakManager, TweakId};
use crate::controller::user_input::{EAction, UserInput};
use crate::game_input::game_input::GameInput;
use crate::game_mode::game_manager::{unpause_game, GameManager, GameMenuMessage};
use crate::message::message_manager::GameMessage;
use crate::vs_common_main::GameClock;
use velocity_audio::AudioCommand;

use super::game_tweaks_menu::{build_tweak_item, trunc, update_tweak_item};

pub const MAX_MENUS: usize = 8;
/// the OSD mask of every menu item (R2 hides them: `VsCrashCamera__63030(dword_6FD1FC, 4096)`)
pub const MASK: u32 = 4096;
/// OSD 1333 x 1000 space, pixel conversions of OSDManager 0x000EF7E0 / 0x000EF810 (512 target)
const OSD_W: f32 = 1333.0;
const OSD_H: f32 = 1000.0;

/// one OSD text item (COSDManager::CreateTextItem 0x000EDF30 arguments): OSD space, y up
#[derive(Clone, Debug, PartialEq)]
pub struct DebugMenuText {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub layer: i16,
    pub flags: u32,
    pub mask: u32,
}

impl DebugMenuText {
    pub fn new(text: &str, x: f32, y: f32, layer: i16, flags: u32, mask: u32) -> Self {
        Self { text: text.to_string(), x, y, layer, flags, mask }
    }
}

/// `CGameMenuItem` (0x1C bytes, ctor 0x000AC7A0: +8 text, +16 id, +20 selected, +21 disabled); tweak items are
/// `CTweakableMenuItem` (+28 tweak, +48 value text)
#[derive(Clone, Debug)]
pub struct CGameMenuItem {
    pub text: String,
    pub id: i32,
    pub disabled: bool,
    pub tweak: Option<TweakId>,
    pub value_text: String,
    /// the value text's OSD item (+44) in the menu's `osd`
    pub value_osd: Option<usize>,
}

impl CGameMenuItem {
    /// GameMenuManager.cpp 0x000AC7A0
    pub fn new(text: &str, id: i32) -> Self {
        Self { text: text.to_string(), id, disabled: false, tweak: None, value_text: String::new(), value_osd: None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuKind {
    /// 0x000AE160 "Debug": Resume (3), Tweaks (2)
    Debug,
    /// 0x000ACDD0 "Front End": Tweaks (1), Resume (2), Profiler Mode (3), Music Manager (4)
    FrontEnd,
    /// GameTweaksMenu.cpp 0x000AFAC0
    TweakSystem { system: usize },
}

/// `CGameMenu` (ctor 0x000A9EF0)
#[derive(Clone, Debug)]
pub struct CGameMenu {
    pub kind: MenuKind,
    pub name: String,              // +4 char[32]
    pub items: Vec<CGameMenuItem>, // +40 (128), count +552
    pub selected: usize,           // +556
    pub x: f32,                    // +564
    pub y: f32,                    // +568
    pub width: f32,                // +572
    pub height: f32,               // +576
    pub first_visible: usize,      // +580
    pub visible: usize,            // +584
    pub repeat: f32,               // +588
    pub dirty: bool,               // +592
    pub closing: bool,             // +1113
    /// `CGameTweakSystemMenu` +1120: the tweak items' repeat timer
    pub tweak_repeat: f32,
    /// +596 the built OSD items
    pub osd: Vec<DebugMenuText>,
}

impl CGameMenu {
    /// GameMenuManager.cpp 0x000A9EF0
    pub fn new(kind: MenuKind, name: &str) -> Self {
        Self {
            kind,
            name: name.chars().take(31).collect(),
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
            closing: false,
            tweak_repeat: 0.0,
            osd: Vec::new(),
        }
    }

    /// GameMenuManager.cpp 0x000AA160 — `AddItem` (128 max)
    pub fn add(&mut self, item: CGameMenuItem) {
        if self.items.len() <= 0x80 {
            self.items.push(item);
        }
    }

    /// GameMenuManager.cpp 0x000AA360
    pub fn set_name(&mut self, name: &str) {
        self.name = name.chars().take(31).collect();
    }

    /// GameMenuManager.cpp 0x000AD0E0 / 0x000AD110
    pub fn set_size(&mut self, w: f32, h: f32) {
        self.width = w;
        self.height = h;
        self.dirty = true;
    }

    /// GameMenuManager.cpp 0x000AE160
    pub fn debug() -> Self {
        let mut m = CGameMenu::new(MenuKind::Debug, "");
        m.set_name("Debug");
        m.add(CGameMenuItem::new("Resume", 3));
        m.add(CGameMenuItem::new("Tweaks", 2));
        m.set_size(f32::from_bits(0x4448_0000), f32::from_bits(0x4416_0000));
        m
    }

    /// GameMenuManager.cpp 0x000ACDD0
    pub fn front_end() -> Self {
        let mut m = CGameMenu::new(MenuKind::FrontEnd, "");
        m.add(CGameMenuItem::new("Tweaks", 1));
        m.add(CGameMenuItem::new("Resume", 2));
        m.add(CGameMenuItem::new("Profiler Mode", 3));
        m.add(CGameMenuItem::new("Music Manager", 4));
        m.set_name("Front End");
        m.set_size(f32::from_bits(0x4448_0000), f32::from_bits(0x4416_0000));
        m
    }

    /// the dirty part of 0x000AA3E0: at most 13 rows, centred
    pub(crate) fn layout(&mut self) {
        self.visible = self.items.len().min(13);
        self.height = self.visible as f32 * 42.0 + 20.0;
        self.x = (1333.0 - self.width) * 0.5;
        self.y = (1000.0 - self.height) * 0.5;
        self.dirty = false;
    }

    /// GameMenuManager.cpp 0x000AAC30 — `Build`: title (0x000AADF0, text part), cursor (0x000AC570) and the visible
    /// rows (item vtable +4: 0x000AC860, tweak items 0x000B0270)
    pub fn build(&mut self, tm: &CTweakManager) {
        self.osd.clear();
        for it in &mut self.items {
            it.value_osd = None;
        }
        // 0x000AADF0: the title 16 px right of the frame, in the 32 px bar above it (flags 0x1000, no mask)
        if !self.name.is_empty() {
            let w16 = (16.0 / 512.0 * OSD_W) as i32 as i16 as f32;
            let h32 = (32.0 / 512.0 * OSD_H) as i32 as i16 as f32;
            let w5 = (5.0 / 512.0 * OSD_W) as i32 as i16 as f32;
            let tx = trunc(w16 + self.x);
            let ty = trunc(self.y + self.height + h32 - w5);
            self.osd.push(DebugMenuText::new(&self.name.clone(), tx, ty, 900, 0x1000, 0));
        }
        let x = self.x + 10.0;
        let w = self.width - 20.0 - 50.0;
        let n = self.visible.min(self.items.len());
        for i in 0..n {
            let index = self.first_visible + i;
            let fi = i as f64;
            let y = (1000.0 - ((self.y + 10.0) as f64 + fi * 40.0 + fi + fi)) as f32;
            if index == self.selected {
                // 0x000AC570: ">" right-justified
                self.osd.push(DebugMenuText::new(">", trunc(x), trunc(y), 900, 4, MASK));
            }
            let mut item = self.items[index].clone();
            if item.tweak.is_some() {
                build_tweak_item(tm, &mut item, x + 50.0, y, w, &mut self.osd);
            } else {
                // 0x000AC860
                self.osd.push(DebugMenuText::new(&item.text, trunc(x + 50.0), trunc(y), 900, 2, MASK));
            }
            self.items[index] = item;
        }
    }

    /// the value texts change without a rebuild (0x000B0A20 writes the item's text buffer, item +44's text)
    fn refresh_values(&mut self) {
        for it in &self.items {
            if let Some(v) = it.value_osd.and_then(|i| self.osd.get_mut(i)) {
                v.text = it.value_text.clone();
            }
        }
    }
}

/// what a debug menu asks of the game
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DebugMenuRequest {
    /// GameManager 0x000BC2C0
    Resume,
    /// 0x000ACFA0 case 3: GameManager 0x000BF0B0(0), LACOAST RACE1, `byte_6F70E8` / `byte_6F70D8`, LoadingScreen
    ProfilerMode,
    /// 0x000ACFA0 case 4: GameTweaksMenu 0x000B1950 (VsSoundtrackPlayer song list)
    MusicManager,
}

/// `CGameMenuManager` for the debug menus
#[derive(Resource, Debug)]
pub struct CDebugMenuManager {
    pub stack: Vec<CGameMenu>, // +4, count +36, top +40
    /// 0x000ACB70 / 0x000ACB90: menus hidden with R2
    pub hidden: bool,
    /// `gs_tweakStack` `dword_6F54E8[16]`
    pub tweak_stack: [i32; 16],
    /// `gs_tweakDepth` `dword_6F552C` (can become -1, see `enter_tweaks_menu`)
    pub tweak_depth: i32,
    /// `byte_6F5528`: set by OnLeaveAll so a closing stack keeps the path
    pub keep_tweak_path: bool,
    pub requests: Vec<DebugMenuRequest>,
    pub changed: Vec<TweakId>,
}

impl Default for CDebugMenuManager {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            hidden: false,
            tweak_stack: [0; 16],
            tweak_depth: 0,
            keep_tweak_path: false,
            requests: Vec::new(),
            changed: Vec::new(),
        }
    }
}

impl CDebugMenuManager {
    /// GameMenuManager.cpp 0x000A9A40 — `EnterMenu` (the old top's items are released, vtable +12 OnEnter)
    pub fn enter(&mut self, mut menu: CGameMenu) {
        if self.stack.len() >= MAX_MENUS {
            log::error!("ASSERT: m_nMenusInStack < nMAX_GAME_MENUS (GameMenuManager.cpp 94)");
            return;
        }
        if let Some(top) = self.stack.last_mut() {
            top.osd.clear();
        }
        // OnEnter: debug 0x000AC710 (+592 = 1), the others none; the ctor's size set is dirty already
        menu.dirty = true;
        self.stack.push(menu);
    }

    /// GameMenuManager.cpp 0x000A9B70 — mark the top closing; true when it is the last
    pub fn leave(&mut self) -> bool {
        let n = self.stack.len();
        let Some(top) = self.stack.last_mut() else { return true };
        top.closing = true;
        n <= 1
    }

    /// GameMenuManager.cpp 0x000A9BD0
    pub fn leave_all(&mut self) {
        for m in &mut self.stack {
            m.closing = true;
        }
    }

    pub fn is_open(&self) -> bool {
        !self.stack.is_empty()
    }

    /// GameMenuManager.cpp 0x000A9C40 — pop the closing menus: OnLeaveAll (+20) on the top when all close, then
    /// OnLeave (+16) on each
    fn pop_closing(&mut self) {
        if self.stack.is_empty() || !self.stack.iter().any(|m| m.closing) {
            return;
        }
        if self.stack.iter().all(|m| m.closing) {
            if let Some(MenuKind::TweakSystem { .. }) = self.stack.last().map(|m| m.kind) {
                // 0x000AFDE0
                self.keep_tweak_path = true;
            }
        }
        let mut i = self.stack.len();
        while i > 0 {
            i -= 1;
            if self.stack[i].closing {
                let m = self.stack.remove(i);
                if let MenuKind::TweakSystem { .. } = m.kind {
                    self.tweak_system_leave();
                }
            }
        }
        if let Some(top) = self.stack.last_mut() {
            top.dirty = true;
        }
    }

    #[cfg(test)]
    pub fn pop_closing_for_test(&mut self) {
        self.pop_closing();
    }

    /// `CGameMenuManager::Update`: pop closing menus, then the top menu's update (vtable +4)
    pub fn update(&mut self, ui: &UserInput, masks: &[u32], dt: f32, tm: &mut CTweakManager) {
        self.pop_closing();
        if self.stack.is_empty() {
            return;
        }
        let top = self.stack.len() - 1;
        let kind = self.stack[top].kind;
        let select = self.menu_update(top, ui, masks, dt, tm);
        if let MenuKind::TweakSystem { .. } = kind {
            // 0x000B0130: +1120 = max(+1120 - dt, 0) after CGameMenu::Update
            if let Some(m) = self.stack.get_mut(top) {
                m.tweak_repeat = (m.tweak_repeat - dt).max(0.0);
            }
        }
        if let Some(id) = select {
            self.handle_action(kind, id, tm);
        }
    }

    /// GameMenuManager.cpp 0x000AA3E0 — `CGameMenu::Update`: layout when dirty, then per player (`dword_6F5C00`,
    /// GameInput 0x000A59D0 masks) select / cancel / R2 hide / up-down with a 0.2 s repeat, the build on a change
    /// and the current item's update (vtable +8). Left / right go to vtable +28, empty for these menus.
    fn menu_update(&mut self, top: usize, ui: &UserInput, masks: &[u32], dt: f32, tm: &mut CTweakManager) -> Option<i32> {
        let mut changed = false;
        let mut select = None;
        {
            let menu = &mut self.stack[top];
            if menu.dirty {
                menu.layout();
                changed = true;
            }
        }
        let pressed = |a: EAction, m: u32| ui.value(a, m) != 0.0 && ui.changed(a, m);
        for &mask in masks {
            let menu = &mut self.stack[top];
            if pressed(EAction::MenuSelect, mask) {
                if let Some(it) = menu.items.get(menu.selected) {
                    if it.id != -1 && !it.disabled {
                        select = Some(it.id);
                    }
                }
            }
            if pressed(EAction::MenuCancel, mask) {
                self.hidden = false;
                if self.leave() {
                    self.requests.push(DebugMenuRequest::Resume);
                }
                return None;
            }
            // MenuR2 held toggles the menus' visibility every update
            if ui.value(EAction::MenuR2, mask) != 0.0 {
                self.hidden = !self.hidden;
            }
            let menu = &mut self.stack[top];
            menu.repeat -= dt;
            let timed = menu.repeat <= 0.0;
            let held = |a: EAction| ui.value(a, mask) != 0.0;
            let mut up = held(EAction::MenuUp) && (pressed(EAction::MenuUp, mask) || timed);
            let mut down = !up && held(EAction::MenuDown) && (pressed(EAction::MenuDown, mask) || timed);
            if !up && !down && timed {
                let v = ui.value(EAction::MenuUpDown, mask);
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
                }
            }
            if select.is_some() || up || down {
                break;
            }
        }
        let menu = &mut self.stack[top];
        if changed {
            menu.build(tm);
        }
        // the current item's update (+560 -> vtable +8): only tweak items do anything (0x000B07E0)
        let sel = menu.selected;
        if let Some(item) = menu.items.get_mut(sel) {
            if let Some(id) = update_tweak_item(tm, item, &mut menu.tweak_repeat, ui) {
                self.changed.push(id);
                menu.refresh_values();
            }
        }
        select
    }

    /// vtable +24 `HandleMenuAction`
    fn handle_action(&mut self, kind: MenuKind, id: i32, tm: &mut CTweakManager) {
        log::debug!("debug menu {kind:?}: select id {id}");
        match kind {
            // GameMenuManager.cpp 0x000AE240
            MenuKind::Debug => match id {
                2 => self.enter_tweaks_menu(tm),
                3 => {
                    self.leave_all();
                    self.requests.push(DebugMenuRequest::Resume);
                }
                _ => {}
            },
            // GameMenuManager.cpp 0x000ACFA0 (no case 2: "Resume" does nothing, original)
            MenuKind::FrontEnd => match id {
                1 => self.enter_tweaks_menu(tm),
                3 => {
                    self.requests.push(DebugMenuRequest::ProfilerMode);
                    self.leave();
                }
                4 => self.requests.push(DebugMenuRequest::MusicManager),
                _ => {}
            },
            MenuKind::TweakSystem { system } => self.tweak_system_action(tm, system, id),
        }
    }

    /// the draw list: the top menu's items unless hidden (mask 4096; the title has no mask)
    pub fn draw(&self) -> Vec<DebugMenuText> {
        let Some(top) = self.stack.last() else { return Vec::new() };
        top.osd.iter().filter(|t| !(self.hidden && t.mask & MASK != 0)).cloned().collect()
    }
}

/// GameInput / GameManager menu calls -> the debug menus, then one `CGameMenuManager::Update` per game update
/// (InGameUpdate 0x000B77F0 calls it with the update step; the front end's GameManager::Update likewise)
#[allow(clippy::too_many_arguments)]
pub fn debug_menu_update(
    mut dm: ResMut<CDebugMenuManager>,
    mut gm: ResMut<GameManager>,
    mut r: MessageReader<GameMenuMessage>,
    ui: Res<UserInput>,
    gi: Res<GameInput>,
    clock: Res<GameClock>,
    mut audio: MessageWriter<AudioCommand>,
    mut msg: MessageWriter<GameMessage>,
) {
    for m in r.read() {
        match m {
            GameMenuMessage::EnterPauseMenu { debug: true } => {
                log::info!("debug menu: Debug");
                dm.enter(CGameMenu::debug());
            }
            GameMenuMessage::EnterFrontEndMenu => {
                log::info!("debug menu: Front End");
                dm.enter(CGameMenu::front_end());
            }
            GameMenuMessage::LeaveAll => dm.leave_all(),
            GameMenuMessage::EnterPauseMenu { debug: false } => {}
        }
    }
    if !dm.is_open() {
        return;
    }
    let masks: Vec<u32> = (0..gm.num_players as usize).map(|p| gi.device_mask(p)).collect();
    let dt = clock.timer.update_dt();
    let dm = &mut *dm;
    dm.update(&ui, &masks, dt, &mut gm.tweaks);
    for id in dm.changed.drain(..) {
        // MessageManager 0x000CA2C0 channel 1 (the FECK2 callback 0x00202EA0 listens)
        let name = gm.tweaks.full_name(id);
        log::info!("debug menu: tweak {name} = {}", gm.tweaks.tweak(id).to_string_value());
        msg.write(GameMessage::TweakChanged(name));
    }
    for q in dm.requests.drain(..) {
        match q {
            DebugMenuRequest::Resume => unpause_game(&mut gm, &mut audio),
            other => log::warn!("debug menu: {other:?} not ported"),
        }
    }
}
