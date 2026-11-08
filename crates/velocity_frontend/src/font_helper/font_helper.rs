//! src\FontHelper\FontHelper.cpp — `CFontHelper`: maps the game's font ids (`eFONT_ID`) to loaded fonts.
//!
//! two sets: frontend (`LoadFrontendFonts`, `\fonts\fontsF.vcd` else `\fonts\fonts.vcd`) and in-game
//! (`LoadGameFonts`, `\fonts\fontsg.vcd` else `\fonts\fonts.vcd`). ids 0..4 are drawn at 0.75 horizontal scale.

use crate::fonts::font_manager::CFontManager;
use crate::render2d::TextureStore;
use crate::string_manager::string_manager::string_crc;

/// `eFONT_ID_MAX`
pub const FONT_ID_MAX: usize = 13;

/// `eFONT_ID` values used by the frontend/osd code.
pub const FONT_VELOCITY10: usize = 0;
pub const FONT_FE11: usize = 1;
pub const FONT_FE12: usize = 2;
pub const FONT_FE14: usize = 3;
pub const FONT_FE_TITLE: usize = 4;
pub const FONT_GAME_FE11: usize = 6;
pub const FONT_OSD_SMALLEST: usize = 7;
pub const FONT_OSD_SMALL: usize = 8;
pub const FONT_OSD_MEDIUM: usize = 9;
pub const FONT_OSD_LARGE: usize = 10;
pub const FONT_VELOCITY12: usize = 11;
pub const FONT_DEBUG: usize = 12;

/// `0x3F400000` passed to `CBaseFont::SetScale` for ids 0..4.
pub const FE_FONT_SCALE: f32 = 0.75;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontSet {
    #[default]
    None,
    Frontend = 1,
    Game = 2,
}

/// `CFontHelper`
#[derive(Default)]
pub struct CFontHelper {
    pub set: FontSet,                         // +0x00
    pub ids: [Option<usize>; FONT_ID_MAX],    // +0x04 font manager index per id (-1 = none)
    pub manager: CFontManager,
}

impl CFontHelper {
    fn open_pack(primary: &str) -> Option<visos::file::Vcd> {
        visos::file::Vcd::open(primary).ok().or_else(|| visos::file::Vcd::open("\\fonts\\fonts.vcd").ok())
    }

    fn load(&mut self, vcd: &visos::file::Vcd, file: &str, crc_name: &str, tex: &mut TextureStore) -> Option<usize> {
        // FontHelper.cpp 0x000B2D10 — `LoadFont`: an already loaded crc is reused
        if let Some(i) = self.manager.find_font(string_crc(crc_name)) {
            return Some(i);
        }
        self.manager.load_font(vcd, file, string_crc(crc_name), tex)
    }

    /// FontHelper.cpp 0x000B2E40 — `UnloadFonts`
    pub fn unload(&mut self) {
        self.ids = [None; FONT_ID_MAX];
        self.manager.fonts.clear();
    }

    /// FontHelper.cpp 0x000B2370 — `LoadFrontendFonts`
    pub fn load_frontend_fonts(&mut self, tex: &mut TextureStore) -> bool {
        if self.set == FontSet::Frontend {
            return true;
        }
        self.set = FontSet::Frontend;
        self.unload();
        let Some(vcd) = Self::open_pack("\\fonts\\fontsF.vcd") else { return false };
        // no compiled debug font in this build: the "debug" id falls back to Tahoma12Bold
        self.ids[FONT_DEBUG] = self.load(&vcd, "Tahoma12Bold", "debug", tex);
        self.ids[FONT_VELOCITY10] = self.load(&vcd, "Velocity10", "Velocity10", tex);
        self.ids[FONT_FE11] = self.load(&vcd, "FEFont11", "fefont11", tex);
        self.ids[FONT_FE12] = self.load(&vcd, "FEFont12", "fefont12", tex);
        self.ids[FONT_FE14] = self.load(&vcd, "FEFont14", "fefont14", tex);
        self.ids[FONT_FE_TITLE] = self.load(&vcd, "FEFontTitle", "FEFontTitle", tex);
        for id in 0..5 {
            self.set_scale(id, FE_FONT_SCALE);
        }
        true
    }

    /// FontHelper.cpp 0x000B26D0 — `LoadGameFonts`
    pub fn load_game_fonts(&mut self, tex: &mut TextureStore) -> bool {
        if self.set == FontSet::Game {
            return true;
        }
        self.set = FontSet::Game;
        self.unload();
        let Some(vcd) = Self::open_pack("\\fonts\\fontsg.vcd") else { return false };
        self.ids[FONT_DEBUG] = self.load(&vcd, "Tahoma12Bold", "debug", tex);
        self.ids[FONT_FE12] = self.ids[FONT_DEBUG];
        self.ids[FONT_GAME_FE11] = self.load(&vcd, "FEFont11", "fefont11", tex);
        self.ids[FONT_OSD_LARGE] = self.load(&vcd, "Velocity26", "font_osd_large", tex);
        self.ids[FONT_OSD_MEDIUM] = self.load(&vcd, "Velocity15", "font_osd_medium", tex);
        self.ids[FONT_OSD_SMALL] = self.load(&vcd, "Arial11", "font_osd_small", tex);
        self.ids[FONT_OSD_SMALLEST] = self.load(&vcd, "Arial09", "font_osd_smallest", tex);
        self.ids[FONT_VELOCITY12] = self.load(&vcd, "Velocity12", "Velocity12", tex);
        let Ok(base) = visos::file::Vcd::open("\\fonts\\fonts.vcd") else { return false };
        self.ids[FONT_VELOCITY10] = self.load(&base, "Velocity10", "Velocity10", tex);
        self.ids[FONT_FE11] = self.load(&base, "FEFont11", "fefont11", tex);
        self.ids[FONT_FE12] = self.load(&base, "FEFont12", "fefont12", tex);
        self.ids[FONT_FE14] = self.load(&base, "FEFont14", "fefont14", tex);
        self.ids[FONT_FE_TITLE] = self.load(&base, "FEFontTitle", "FEFontTitle", tex);
        for id in 0..5 {
            self.set_scale(id, FE_FONT_SCALE);
        }
        true
    }

    /// FontHelper.cpp 0x000B2C60 / 0x000B2F60 — + `CBaseFont::SetScale`
    fn set_scale(&mut self, id: usize, s: f32) {
        if let Some(i) = self.ids[id] {
            if let Some(f) = self.manager.get_mut(i) {
                f.scale = s;
            }
        }
    }

    /// FontHelper.cpp 0x000B2BD0 / 0x000B2FB0 — font for an id (the debug font for an unmapped id).
    pub fn font(&self, id: usize) -> Option<&crate::fonts::font_manager::CBaseFont> {
        assert!(id < FONT_ID_MAX, "nFont < eFONT_ID_MAX");
        let i = self.ids[id].or(self.ids[FONT_DEBUG])?;
        self.manager.get(i)
    }
}
