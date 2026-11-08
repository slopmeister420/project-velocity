//! src\Osd\OSDScreens\OSDScreens.cpp — `g_OSDPauseScreens` (`dword_6FDC70`): five 52-byte `COSDScreen`s that
//! draw the in-race menus as OSD items (the menus themselves set no-frame, so `CGameMenu::Build` draws nothing).
//! Slots: 0 pause, 1 display, 2 audio, 3 race over, 4 quit confirm. A menu's OnEnter (vtable +12) points its
//! screen at it (0x000AE100), OnLeave (+16) asks it to close (0x000AE140); the screens run once per game update
//! (GameManager::InGameUpdate "g_OSDPauseScreens.Update", mode 2 after the menu manager).
//! The callbacks of slots 0 / 3 / 4 sit in OSDInGameScreen.cpp, slot 2's in the Speedo.cpp range (object files are
//! shifted against objregions.tsv).

use crate::feck2::feck2_manager::{visos_colour, COLOUR_WHITE};
use crate::render2d::Colour;
use crate::font_helper::font_helper::{CFontHelper, FONT_GAME_FE11};
use crate::game_menu_manager::game_menu_manager::CGameMenu;
use crate::osd::osd_manager::{COSDManager, ItemId};
use crate::render2d::TextureStore;
use crate::string_manager::string_manager::{string_crc, CStringManager};

/// slot in `g_OSDPauseScreens`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseScreen {
    Pause = 0,
    Display = 1,
    Audio = 2,
    RaceOver = 3,
    Quit = 4,
}

/// `COSDScreen` (52 bytes, ctor 0x000FD090)
#[derive(Clone, Debug, Default)]
pub struct COSDScreen {
    pub items: Vec<Option<ItemId>>, // +0, count +8
    pub flags: u8,                  // +11: 1 active, 2 items made
    pub state: u32,                 // +12
    /// +16 the menu (here its index in the menu manager's stack)
    pub menu: Option<usize>,
}

impl COSDScreen {
    /// GameMenuManager.cpp 0x000AE100 — show for a menu
    pub fn enter(&mut self, menu: usize) {
        self.flags |= 1;
        self.state = 0;
        self.menu = Some(menu);
    }

    /// GameMenuManager.cpp 0x000AE140 — close (fade out / release)
    pub fn leave(&mut self) {
        self.state = 4;
    }
}

/// `g_OSDPauseScreens`
#[derive(Clone, Debug, Default)]
pub struct COSDPauseScreens {
    pub screens: [COSDScreen; 5],
    /// slot 0 +24 textures[0] = "pauseScreen" (osdT.vcd), shared by slots 3 / 4
    pub texture: u32,
    /// `flt_6FD9F8` the pause screen's fade
    pub fade: f32,
}

/// `dword_686528` fade step per update
const FADE_STEP: f32 = 0.25;
/// `byte_68652C` disabled item alpha
const DISABLED_ALPHA: u32 = 0x40;
/// `dword_637274` item x (`dword_686538` / 540 / 550 / 55C / 6860BC are overwritten with it before use)
const ITEM_X: f32 = 640.0;

/// VISOS_ColourXBOX(0x80, 0x7C, 0x4B, 3): the selected item
fn selected() -> Colour {
    visos_colour(0x80, 0x7C, 0x4B, 3)
}

fn with_alpha(c: Colour, a: u32) -> Colour {
    (c & 0x00FF_FFFF) | ((a & 0xFF) << 24)
}

impl COSDPauseScreens {
    /// OSDInGameScreen.cpp 0x000FC9E0 / 0x000FBB90 — "Pause Screens Init" (the second loads "pauseScreen" from \osd\osdT.vcd)
    pub fn new(tex: &TextureStore) -> Self {
        let texture = tex.find(visos::handy::handy_crc::name_hash("pauseScreen"));
        if texture == 0 {
            log::warn!("pauseScreen texture file not found!");
        }
        Self { texture, ..Default::default() }
    }

    /// OSDScreens.cpp 0x000FCAF0 — update every screen
    pub fn update(&mut self, menus: &mut [CGameMenu], osd: &mut COSDManager, fonts: &CFontHelper, strings: &CStringManager, tex: &TextureStore, online: bool) {
        for slot in 0..5 {
            self.update_screen(slot, menus, osd, fonts, strings, tex, online);
        }
    }

    /// OSDInGameScreen.cpp 0x000FC7D0 / 0x000FB6D0 / 0x000FC470 / 0x000FC000 / OSDScreens.cpp 0x000FD520 / Speedo.cpp 0x000F80A0 —
    /// one screen's state machine (+ the five release callbacks, state 5)
    #[allow(clippy::too_many_arguments)]
    fn update_screen(&mut self, slot: usize, menus: &mut [CGameMenu], osd: &mut COSDManager, fonts: &CFontHelper, strings: &CStringManager, tex: &TextureStore, online: bool) {
        let s = &self.screens[slot];
        if s.flags & 1 == 0 {
            return;
        }
        let m = s.menu.and_then(|i| menus.get_mut(i));
        match self.screens[slot].state {
            0 => {
                if let Some(m) = m {
                    self.init(slot, m, osd, fonts, strings, tex, online);
                }
                let s = &mut self.screens[slot];
                s.state = 1;
                s.flags |= 2;
            }
            1 => {
                // only the pause screen fades in (0x000FB7A0)
                let done = slot != 0 || m.map(|m| self.fade_in(m, osd)).unwrap_or(true);
                if done {
                    self.screens[slot].state = 2;
                }
            }
            3 => self.screens[slot].state = 4,
            4 => {
                let done = slot != 0 || self.fade_out(osd);
                if done {
                    self.screens[slot].state = 5;
                }
            }
            5 => {
                // 0x000FB6D0 / 0x000FD520 / 0x000F80A0 / 0x000FC470 / 0x000FC000
                let s = &mut self.screens[slot];
                for id in s.items.drain(..).flatten() {
                    osd.release(id);
                }
                s.flags &= !1;
                s.flags &= !2;
            }
            _ => {
                if let Some(m) = m {
                    self.update_colours(slot, m, osd);
                }
            }
        }
    }

    /// OSDInGameScreen.cpp 0x000FB3A0 / 0x000FBD40 / 0x000FC210 / OSDScreens.cpp 0x000FD310 / Speedo.cpp 0x000F7E90 — the
    /// init callbacks (pause, quit, race over, display, audio). State 0: make the items once (`(+11 & 2) == 0`)
    #[allow(clippy::too_many_arguments)]
    fn init(&mut self, slot: usize, m: &mut CGameMenu, osd: &mut COSDManager, fonts: &CFontHelper, strings: &CStringManager, tex: &TextureStore, online: bool) {
        if self.screens[slot].flags & 2 != 0 {
            return;
        }
        let n = m.items.len();
        // (first slot of the menu items, y of the first item, row step, mask)
        let (first, y0, dy, mask): (usize, f32, f32, u32) = match slot {
            // 0x000FB3A0: dword_686530 / dword_686534
            0 => (1, 650.0, 55.0, 4),
            // 0x000FD310: dword_686554 / dword_686558
            1 => (0, 650.0, 55.0, 64),
            // 0x000F7E90: dword_6860B4 / dword_6860B8
            2 => (0, 650.0, 45.0, 128),
            // 0x000FC210: dword_686548 / dword_68654C
            3 => (1, 575.0, 75.0, 256),
            // 0x000FBD40: dword_686544 / dword_68653C
            _ => (2, 525.0, 75.0, 2048),
        };
        let texture = self.texture;
        let s = &mut self.screens[slot];
        if s.items.is_empty() {
            s.items = vec![None; n + first];
        }
        // CFontHelper::GetFontIndex(6)
        let font = FONT_GAME_FE11;
        // 0x000FBD40: the confirm text above the items
        if slot == 4 && s.items[1].is_none() {
            let id = osd.create_string_item(fonts, strings, tex, m.description, font, ITEM_X, y0 + 160.0, 110, -1.0, 0x48, mask);
            if let Some(id) = id {
                osd.item_mut(id).colour = COLOUR_WHITE;
            }
            s.items[1] = id;
        }
        for (i, item) in m.items.iter().enumerate() {
            if s.items[i + first].is_some() {
                continue;
            }
            let colour = match (slot, i) {
                // the pause screen starts transparent and fades in
                (0, 0) => visos_colour(0, 0x7C, 0x4B, 3),
                (0, _) => with_alpha(COLOUR_WHITE, 0),
                (_, 0) => selected(),
                _ => COLOUR_WHITE,
            };
            let y = y0 - i as f32 * dy;
            let x = ITEM_X - i as f32 * 0.0;
            let id = osd.create_string_item(fonts, strings, tex, item.string, font, x, y, 110, -1.0, 0x28, mask);
            if let Some(id) = id {
                osd.item_mut(id).colour = colour;
            }
            s.items[i + first] = id;
        }
        // the panel: 1600 x 600 centred on (640, 500), layer 1000 (drawn under the text)
        if matches!(slot, 0 | 3 | 4) && s.items[0].is_none() {
            let mask = if slot == 0 { 0 } else { mask };
            let id = osd.create_graphic_item(texture, 640.0, 500.0, 1000, 1600.0, 600.0, -1.0, 0x48, mask);
            osd.item_mut(id).colour = if slot == 0 { with_alpha(COLOUR_WHITE, 0) } else { COLOUR_WHITE };
            s.items[0] = Some(id);
        }
        // 0x000FB3A0 online: MENU_RESTART is disabled
        if slot == 0 && online {
            let restart = string_crc("MENU_RESTART");
            for it in m.items.iter_mut() {
                if it.string == restart {
                    it.disabled = true;
                }
            }
        }
    }

    /// OSDInGameScreen.cpp 0x000FB7A0 — pause screen state 1: everything to `fade * 255`, disabled items to
    /// `0x40 * fade`; done once the fade passes 1
    fn fade_in(&mut self, m: &CGameMenu, osd: &mut COSDManager) -> bool {
        let fade = self.fade;
        let s = &self.screens[0];
        for id in s.items.iter().flatten() {
            let c = osd.item_mut(*id).colour;
            osd.item_mut(*id).colour = with_alpha(c, (fade * 255.0) as i32 as u32);
        }
        for (i, it) in m.items.iter().enumerate() {
            if it.disabled {
                if let Some(Some(id)) = s.items.get(i + 1) {
                    let c = osd.item_mut(*id).colour;
                    osd.item_mut(*id).colour = with_alpha(c, (DISABLED_ALPHA as f32 * fade) as i32 as u32);
                }
            }
        }
        self.fade += FADE_STEP;
        if self.fade <= 1.0 {
            false
        } else {
            self.fade = 1.0;
            true
        }
    }

    /// OSDInGameScreen.cpp 0x000FB940 — pause screen state 4
    fn fade_out(&mut self, osd: &mut COSDManager) -> bool {
        let fade = self.fade;
        for id in self.screens[0].items.iter().flatten() {
            let c = osd.item_mut(*id).colour;
            osd.item_mut(*id).colour = with_alpha(c, (fade * 255.0) as i32 as u32);
        }
        self.fade -= FADE_STEP;
        if self.fade <= 0.0 {
            self.fade = 0.0;
            true
        } else {
            false
        }
    }

    /// OSDInGameScreen.cpp 0x000FBA30 / 0x000FC540 / 0x000FC0D0 / OSDScreens.cpp 0x000FD5F0 / Speedo.cpp 0x000F8170 — the
    /// default state (2): selected orange, others white (pause also dims disabled items; display, audio, race over, quit)
    fn update_colours(&mut self, slot: usize, m: &CGameMenu, osd: &mut COSDManager) {
        let first = match slot {
            0 | 3 => 1,
            4 => 2,
            _ => 0,
        };
        let s = &self.screens[slot];
        for (i, it) in m.items.iter().enumerate() {
            let c = match (it.selected, slot == 0 && it.disabled) {
                (true, false) => selected(),
                (true, true) => visos_colour(DISABLED_ALPHA, 0x7C, 0x4B, 3),
                (false, false) => COLOUR_WHITE,
                // VISOS_ColourXBOX(0x40, white's r, g, b)
                (false, true) => visos_colour(DISABLED_ALPHA, 0xFF, 0xFF, 0xFF),
            };
            if let Some(Some(id)) = s.items.get(i + first) {
                osd.item_mut(*id).colour = c;
            }
        }
    }
}
