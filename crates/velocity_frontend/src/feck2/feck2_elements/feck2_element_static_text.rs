//! src\Feck2\FECK2Elements\FECK2ElementStaticText.cpp — `CElementStaticText`: a string drawn in a box.

use crate::feck2::feck2_elements::feck2_element_2d::clamp;
use crate::feck2::feck2_object::{CElementBase, SElementDesc};
use crate::fonts::font_manager::{FONT_CENTRE_X, FONT_RIGHT, FONT_WORDWRAP};
use crate::render2d::{Colour, DrawList, TextureStore, WHITE};

/// `unk_769EE0`: shadow offset (1, 1, 0).
pub const SHADOW_OFFSET: [f32; 2] = [1.0, 1.0];

/// `0x44000000`
pub const DEFAULT_BOX: f32 = 512.0;

#[derive(Clone, Debug)]
pub struct CElementStaticText {
    pub base: CElementBase,
    pub alpha_target: i32,   // +84
    pub alpha_start: i32,    // +88
    pub fade_time: i32,      // +92
    pub fade_left: i32,      // +96
    pub fade_delay: i32,     // +100
    pub pulse: i32,          // +104
    pub font: Option<usize>, // +108 resolved font (manager index)
    pub font_id: u32,        // +112
    pub justification: u32,  // +116
    pub colour: Colour,      // +120
    pub colour2: Colour, // +124
    pub flags: u32,          // +128
    pub string: Option<u32>, // +132 string handle (crc of the id), None = raw text
    pub text: Vec<u16>,      // +136 raw text when no handle
    pub shadow: bool,        // +144
    pub has_string: bool,    // +145
    pub box_w: f32,          // +148
    pub box_h: f32,          // +152
}

impl Default for CElementStaticText {
    /// FECK2ElementStaticText.cpp 0x0029B060
    fn default() -> Self {
        Self {
            base: CElementBase::default(),
            alpha_target: 0,
            alpha_start: 0,
            fade_time: 0,
            fade_left: 0,
            fade_delay: 0,
            pulse: 0,
            font: None,
            font_id: 0,
            justification: 0,
            colour: WHITE,
            colour2: WHITE,
            flags: 0,
            string: None,
            text: vec![0],
            shadow: false,
            has_string: false,
            box_w: DEFAULT_BOX,
            box_h: DEFAULT_BOX,
        }
    }
}

fn alpha(c: Colour) -> u8 {
    (c >> 24) as u8
}
fn with_alpha(c: Colour, a: u8) -> Colour {
    (c & 0x00FF_FFFF) | ((a as u32) << 24)
}

impl CElementStaticText {
    /// FECK2ElementStaticText.cpp 0x00286B80 — `Init`
    pub fn init(&mut self, d: &SElementDesc) {
        self.base.init(d);
        self.base.position = d.position;
        self.base.target = self.base.position;
        self.base.move_time = 0;
        self.base.move_left = 0;
        self.font = None;
        if d.string == 0xFFFF {
            self.text = vec![0];
            self.has_string = false;
            self.string = None;
        } else {
            self.string = Some(d.string);
            self.has_string = true;
        }
        self.font_id = d.font;
        self.justification = 0;
        self.colour = WHITE;
        self.colour2 = WHITE;
        self.alpha_target = alpha(self.colour) as i32;
        self.alpha_start = alpha(self.colour) as i32;
        self.fade_time = 0;
        self.fade_left = 0;
        self.fade_delay = 0;
        self.pulse = 0;
        self.shadow = false;
        self.box_w = DEFAULT_BOX;
        self.box_h = DEFAULT_BOX;
        self.flags = FONT_WORDWRAP;
    }

    /// FECK2ElementStaticText.cpp 0x00286D30 — `Load`: resolve the font id through the font helper.
    pub fn load(&mut self, fonts: &crate::font_helper::font_helper::CFontHelper) -> bool {
        match fonts.ids.get(self.font_id as usize).copied().flatten() {
            Some(i) => {
                self.font = Some(i);
                true
            }
            None => {
                self.font = fonts.ids[crate::font_helper::font_helper::FONT_DEBUG];
                self.font.is_some()
            }
        }
    }

    /// FECK2ElementStaticText.cpp 0x002870E0 — `SetString`
    pub fn set_string(&mut self, crc: u32) {
        self.string = Some(crc);
    }

    /// FECK2ElementStaticText.cpp 0x00287120 — `SetText`: raw text, drops the handle.
    pub fn set_text(&mut self, text: &[u16]) {
        self.text = text.to_vec();
        if self.text.last() != Some(&0) {
            self.text.push(0);
        }
        self.string = None;
    }

    /// FECK2ElementStaticText.cpp 0x002871C0 — `SetJustification` (original never clears the old bit)
    pub fn set_justification(&mut self, j: u32) {
        self.justification = j;
        if self.justification == 2 {
            self.flags |= FONT_CENTRE_X;
        }
        if self.justification == 1 {
            self.flags |= FONT_RIGHT;
        }
    }

    /// FECK2ElementStaticText.cpp 0x00287240 — `SetBox`
    pub fn set_box(&mut self, w: f32, h: f32) {
        self.box_w = w;
        self.box_h = h;
    }

    /// FECK2Object.cpp 0x00207CD0 — `SetColour`
    pub fn set_colour(&mut self, c: Colour) {
        self.colour = c;
    }

    /// FECK2Object.cpp 0x00213C00 — `SetColour2`
    pub fn set_colour2(&mut self, c: Colour) {
        self.colour2 = c;
    }

    fn resolve<'a>(&'a self, strings: &crate::string_manager::string_manager::CStringManager) -> Vec<u16> {
        if let Some(crc) = self.string {
            let mut s = strings.get_string(crc);
            s.push(0);
            s
        } else {
            self.text.clone()
        }
    }

    /// FECK2ElementStaticText.cpp 0x00286E40 — `GetHeight` (string extent in the box, truncated)
    pub fn get_height(&self, ctx: &TextCtx) -> u32 {
        let Some(f) = self.font.and_then(|i| ctx.fonts.manager.get(i)) else { return 0 };
        let s = self.resolve(ctx.strings);
        let (_, h, ..) = f.get_string_extent(&s, self.box_w, self.box_h, FONT_WORDWRAP, ctx.tex);
        h as u32
    }

    /// FECK2ElementStaticText.cpp 0x00286F90 — `GetWidth` (string extent in the box, truncated)
    pub fn get_extent_width(&self, ctx: &TextCtx) -> u32 {
        let Some(f) = self.font.and_then(|i| ctx.fonts.manager.get(i)) else { return 0 };
        let s = self.resolve(ctx.strings);
        let (w, ..) = f.get_string_extent(&s, self.box_w, self.box_h, FONT_WORDWRAP, ctx.tex);
        w as u32
    }

    /// FECK2ElementStaticText.cpp 0x00287880 — `Fade`
    pub fn fade(&mut self, percent: i32, time: i32, delay: i32) -> bool {
        self.fade_delay = delay;
        if time != 0 {
            if self.fade_left != 0 {
                let a = self.alpha_target as u8;
                self.colour2 = with_alpha(self.colour2, a);
                self.colour = with_alpha(self.colour, a);
            }
            self.alpha_start = alpha(self.colour2) as i32;
            self.alpha_target = ((percent as f32 * 0.01) * 255.0) as i32;
            self.alpha_target = clamp(self.alpha_target as f32, 0.0, 255.0) as i32;
            self.fade_time = time;
            self.fade_left = time;
            false
        } else {
            let step = 2 * percent;
            if step == 0 {
                return false;
            }
            let a = alpha(self.colour) as i32;
            if step <= 0 {
                if a == 0 {
                    return false;
                }
                let n = if a >= -step { (a + step) as u8 } else { 0 };
                self.colour = with_alpha(self.colour, n);
            } else {
                let max = alpha(self.colour2) as i32;
                if a == max {
                    return false;
                }
                let v = (step + a) as u32;
                let n = if v <= max as u32 { v as u8 } else { max as u8 };
                self.colour = with_alpha(self.colour, n);
            }
            true
        }
    }

    /// FECK2ElementStaticText.cpp 0x00287830 — `Pulse`
    pub fn pulse(&mut self, percent: i32, time: i32, delay: i32) -> bool {
        self.pulse = percent;
        self.fade(100 - percent, time, delay);
        true
    }

    /// FECK2ElementStaticText.cpp 0x00287500 — `Update` (fade step)
    pub fn update(&mut self, frame_time: f32) {
        if self.fade_delay <= 0 {
            self.fade_left = (self.fade_left as f32 - frame_time) as i32;
            self.fade_delay = 0;
            if self.fade_left > 0 {
                let d = self.alpha_target - self.alpha_start;
                let a = (1.0 - (self.fade_left as f32 * 1.0) / (self.fade_time as f32 * 1.0)) * 90.0;
                let s = (a * 0.017_453_292_f32).sin();
                let v = clamp((self.alpha_start as f32 + d as f32 * s) as i32 as f32, 0.0, 255.0) as u8;
                self.colour2 = with_alpha(self.colour2, v);
                self.colour = with_alpha(self.colour, v);
            } else if self.pulse != 0 {
                self.fade_left = self.fade_time;
                self.alpha_start = self.alpha_target;
                let a = self.alpha_target as u8;
                self.colour2 = with_alpha(self.colour2, a);
                self.colour = with_alpha(self.colour, a);
                let v = if self.pulse <= 0 {
                    (self.pulse as f32 * 0.01) * 255.0 + 255.0
                } else {
                    (self.pulse as f32 * 0.01) * 255.0
                };
                self.alpha_target = v as i32;
                self.pulse = -self.pulse;
            } else {
                self.alpha_start = self.alpha_target;
                let a = self.alpha_target as u8;
                self.colour2 = with_alpha(self.colour2, a);
                self.colour = with_alpha(self.colour, a);
                self.fade_left = 0;
                self.fade_time = 0;
            }
        } else {
            self.fade_delay = (self.fade_delay as f32 - frame_time) as i32;
        }
    }

    /// FECK2ElementStaticText.cpp 0x00287280 — `DrawText`
    pub fn draw_text(&self, s: &[u16], ctx: &TextCtx, out: &mut DrawList) {
        if s.is_empty() {
            return;
        }
        let Some(font) = self.font.and_then(|i| ctx.fonts.manager.get(i)) else { return };
        let mut x = self.base.position[0];
        if self.justification == 1 {
            x -= (self.box_w as u32) as f32;
        } else if self.justification == 2 {
            x -= ((self.box_w / 2.0) as u32) as f32;
        }
        let y = self.base.position[1];
        if self.shadow {
            let sx = SHADOW_OFFSET[0] + x;
            let sy = SHADOW_OFFSET[1] + y;
            let shadow = with_alpha(crate::feck2::feck2_manager::TEXT_SHADOW_COLOUR, alpha(self.colour) / 2);
            font.draw_string_rect(sx, sy, sx + self.box_w, sy + self.box_h, s, shadow, self.flags, None, ctx.tex, out);
        }
        font.draw_string_rect(x, y, x + self.box_w, y + self.box_h, s, self.colour, self.flags, None, ctx.tex, out);
    }

    /// FECK2ElementStaticText.cpp 0x00287750 — `Render`
    pub fn render(&mut self, frame_time: f32, ctx: &TextCtx, out: &mut DrawList) {
        if !crate::feck2::feck2_manager::enable_2d() {
            return;
        }
        if self.base.move_left != 0 {
            self.base.update_move(frame_time);
        }
        if self.fade_left != 0 {
            self.update(frame_time);
        }
        let s = self.resolve(ctx.strings);
        self.draw_text(&s, ctx, out);
    }
}

/// what text drawing needs from the frontend.
pub struct TextCtx<'a> {
    pub fonts: &'a crate::font_helper::font_helper::CFontHelper,
    pub strings: &'a crate::string_manager::string_manager::CStringManager,
    pub tex: &'a TextureStore,
}
