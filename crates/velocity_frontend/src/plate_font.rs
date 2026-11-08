//! the font calls of Render\LicencePlateRender.cpp 0x0012DF40, answered by the frontend's fonts: graphics'
//! `PlateFont` hook (`visos_render::licence_plate_render`). Font helper id 2 is the front end set's FEFont12
//! (`LoadFrontendFonts`), id 6 the game set's FEFont11 (`LoadGameFonts`); each set is a `CFontHelper` of its own.

use visos_render::licence_plate_render::{PlateFont, PlateFontHook, PlateGlyph};
use visos_render::renderer::texture_manager::DecodedTexture;

use crate::font_helper::font_helper::{CFontHelper, FONT_FE12, FONT_GAME_FE11};
use crate::fonts::font_manager::CBaseFont;
use crate::render2d::{DrawList, TextureStore};

struct Set {
    helper: CFontHelper,
    tex: TextureStore,
    page: Option<DecodedTexture>,
}

impl Set {
    fn load(id: usize, game: bool) -> Self {
        let mut s = Set { helper: CFontHelper::default(), tex: TextureStore::default(), page: None };
        let ok = if game { s.helper.load_game_fonts(&mut s.tex) } else { s.helper.load_frontend_fonts(&mut s.tex) };
        if ok {
            // only the id the plate uses; an unmapped id would fall back to the debug font
            if let Some(f) = s.helper.ids[id].and_then(|i| s.helper.manager.get(i)) {
                s.page = s.tex.get(f.texture).map(|p| DecodedTexture { width: p.width, height: p.height, levels: vec![p.data.clone()] });
            }
        }
        s
    }

    fn font(&self) -> Option<&CBaseFont> {
        self.page.as_ref()?;
        self.helper.font(if self.helper.set == crate::font_helper::font_helper::FontSet::Game { FONT_GAME_FE11 } else { FONT_FE12 })
    }
}

/// `PlateFont` over the two font sets
pub struct PlateFonts {
    front_end: Set,
    game: Set,
}

impl PlateFonts {
    pub fn load() -> Self {
        Self { front_end: Set::load(FONT_FE12, false), game: Set::load(FONT_GAME_FE11, true) }
    }

    fn set(&self, font: u32) -> Option<&Set> {
        match font as usize {
            FONT_FE12 => Some(&self.front_end),
            FONT_GAME_FE11 => Some(&self.game),
            _ => None,
        }
    }

    pub fn hook() -> PlateFontHook {
        PlateFontHook(Box::new(Self::load()))
    }
}

impl PlateFont for PlateFonts {
    fn scale(&self, font: u32) -> Option<f32> {
        Some(self.set(font)?.font()?.scale)
    }

    fn string_width(&self, font: u32, text: &[u16]) -> i32 {
        let Some(s) = self.set(font) else { return 0 };
        s.font().map(|f| f.get_string_width(text, 0, &s.tex)).unwrap_or(0)
    }

    fn draw_string_at(&self, font: u32, scale: f32, x: f32, y: f32, text: &[u16], colour: u32) -> Vec<PlateGlyph> {
        let Some(s) = self.set(font) else { return Vec::new() };
        let Some(f) = s.font() else { return Vec::new() };
        // FontHelper 0x000B2F60 SetScale before the draw (the original leaves it set; the next plate sets it again)
        let mut f = f.clone();
        f.scale = scale;
        let mut out = DrawList::default();
        f.draw_string_at(x, y, text, colour, 0, &s.tex, &mut out);
        out.quads
            .iter()
            .map(|q| PlateGlyph {
                rect: [q.pos[0][0], q.pos[0][1], q.pos[3][0], q.pos[3][1]],
                uv: [q.uv[0][0], q.uv[0][1], q.uv[3][0], q.uv[3][1]],
                colour: q.colour[0],
            })
            .collect()
    }

    fn page(&self, font: u32) -> Option<&DecodedTexture> {
        self.set(font)?.page.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// "OCTANE" (GamePlayer.cpp 0x000C1930 default) measures and draws in both sets
    #[test]
    fn plate_fonts_draw_octane() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let p = PlateFonts::load();
        let text: Vec<u16> = "OCTANE".encode_utf16().collect();
        for id in [2u32, 6] {
            assert!(p.scale(id).is_some(), "font {id}");
            assert!(p.string_width(id, &text) > 0);
            assert_eq!(p.draw_string_at(id, 1.0, 0.0, 11.0, &text, 0xFF00_0000).len(), 6);
            assert!(p.page(id).is_some());
        }
    }
}
