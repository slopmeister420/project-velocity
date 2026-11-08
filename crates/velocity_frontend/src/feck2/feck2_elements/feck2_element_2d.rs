//! src\Feck2\FECK2Elements\FECK2Element2D.cpp — `CElement2D`: a textured, vertex-coloured sprite.

use crate::feck2::feck2_object::{CElementBase, SElementDesc};
use crate::render2d::{Colour, DrawList, Quad, TextureId, WHITE};

/// `CElement2D::m_dFlags` bits.
pub const FLAG_FLIP_U: u32 = 0x1;
pub const FLAG_FLIP_V: u32 = 0x2;
/// pre-scale x by 4:3 (set by Init, undoes the 1/aspect of the sprite matrix).
pub const FLAG_ASPECT_4_3: u32 = 0x4;
/// pre-scale x by 16:9.
pub const FLAG_ASPECT_16_9: u32 = 0x8;
/// no aspect correction at all.
pub const FLAG_NO_ASPECT: u32 = 0x10;

/// `0x3FAAA0BE` / `0x3FE374BC`: the pre-scale factors for flags 4 / 8.
pub const PRESCALE_4_3: f32 = f32::from_bits(0x3FAA_A0BE);
pub const PRESCALE_16_9: f32 = f32::from_bits(0x3FE3_74BC);
/// `0x3FAAAAAB` / `0x3FE38E39`: screen aspect divided out of x (`byte_6F915D` selects widescreen).
pub const ASPECT_4_3: f32 = f32::from_bits(0x3FAA_AAAB);
pub const ASPECT_16_9: f32 = f32::from_bits(0x3FE3_8E39);


#[derive(Clone, Debug)]
pub struct CElement2D {
    pub base: CElementBase,
    pub alpha_target: i32,   // +84
    pub alpha_start: i32,    // +88
    pub fade_time: i32,      // +92
    pub fade_left: i32,      // +96
    pub fade_delay: i32,     // +100
    pub pulse: i32,          // +104
    pub texture_name: String,// +108
    pub vcd: String,         // +112
    pub texture: TextureId,  // +116
    pub width: u32,          // +120
    pub height: u32,         // +124
    pub tex_width: u32,      // +128
    pub tex_height: u32,     // +132
    pub flags: u32,          // +136
    pub colour_top: Colour,  // +140
    pub colour_bottom: Colour, // +144
    pub dynamic: bool,       // +148
    pub scale: f32,          // +152
    pub rotation: f32,       // +156
    pub pivot: [f32; 4],     // +160
}

impl Default for CElement2D {
    /// FECK2Element2D.cpp 0x0029AC10
    fn default() -> Self {
        Self {
            base: CElementBase::default(),
            alpha_target: 0,
            alpha_start: 0,
            fade_time: 0,
            fade_left: 0,
            fade_delay: 0,
            pulse: 0,
            texture_name: String::new(),
            vcd: String::new(),
            texture: 0,
            width: 0,
            height: 0,
            tex_width: 0,
            tex_height: 0,
            flags: 0,
            colour_top: WHITE,
            colour_bottom: WHITE,
            dynamic: false,
            scale: 1.0,
            rotation: 0.0,
            pivot: [-0.5, -0.5, 0.0, 1.0],
        }
    }
}

fn alpha(c: Colour) -> u8 {
    (c >> 24) as u8
}
fn with_alpha(c: Colour, a: u8) -> Colour {
    (c & 0x00FF_FFFF) | ((a as u32) << 24)
}

/// `VsClamp` (0x000141E0)
pub fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    if v < lo { lo } else if v > hi { hi } else { v }
}

impl CElement2D {
    /// FECK2Element2D.cpp 0x00291F60 — `Init`
    pub fn init(&mut self, d: &SElementDesc) {
        self.base.init(d);
        self.base.position = d.position;
        self.base.target = self.base.position;
        self.base.move_time = 0;
        self.base.move_left = 0;
        self.texture_name = d.texture.clone();
        self.vcd = d.vcd.clone();
        self.texture = 0;
        self.width = 0;
        self.height = 0;
        self.tex_width = 0;
        self.tex_height = 0;
        self.colour_top = d.colour;
        self.colour_bottom = d.colour;
        self.alpha_target = alpha(d.colour) as i32;
        self.alpha_start = alpha(d.colour) as i32;
        self.fade_time = 0;
        self.fade_left = 0;
        self.fade_delay = 0;
        self.pulse = 0;
        self.dynamic = d.dynamic;
        self.pivot = [-0.5, -0.5, 0.0, 1.0];
        self.rotation = 0.0;
        self.scale = 1.0;
        self.flags = 0;
        self.set_flags(FLAG_ASPECT_4_3);
    }

    /// FECK2Element2D.cpp 0x00213B30 — `SetFlags` (or-in)
    pub fn set_flags(&mut self, f: u32) {
        self.flags |= f;
    }

    /// FECK2Element2D.cpp 0x00293380 — `HasFlags`
    pub fn has_flags(&self, f: u32) -> bool {
        f == (f & self.flags)
    }

    /// FECK2Element2D.cpp 0x00213B60 — `SetSize`
    pub fn set_size(&mut self, w: u32, h: u32) {
        self.width = w;
        self.height = h;
    }

    /// FECK2Element2D.cpp 0x00209730 — `SetColour(top, gradient, bottom)`
    pub fn set_colour(&mut self, top: Colour, gradient: bool, bottom: Colour) {
        self.colour_top = top;
        self.colour_bottom = if gradient { bottom } else { top };
        self.alpha_target = alpha(top) as i32;
    }

    /// FECK2Element2D.cpp 0x002920F0 — `Load`: loads the texture from the element's pack on first use.
    pub fn load(&mut self, tex: &mut crate::render2d::TextureStore) -> bool {
        if self.texture != 0 {
            return true;
        }
        self.texture = load_texture(tex, &self.vcd, &self.texture_name);
        let (w, h) = tex.size(self.texture);
        self.tex_height = h;
        if self.height == 0 {
            self.height = self.tex_height;
        }
        self.tex_width = w;
        if self.width == 0 {
            self.width = self.tex_width;
        }
        true
    }

    /// FECK2Element2D.cpp 0x00292DF0 — `ChangeTexture`: the size follows the new texture.
    pub fn change_texture(&mut self, name: &str, _release_old: bool, tex: &mut crate::render2d::TextureStore) {
        if self.dynamic {
            self.texture_name = name.to_string();
        }
        let id = tex.find(visos::handy::handy_crc::name_hash(name));
        if id == self.texture && self.texture != 0 {
            return;
        }
        let id = if id != 0 { id } else { load_texture(tex, &self.vcd, name) };
        self.texture = id;
        let (w, h) = tex.size(id);
        self.tex_height = h;
        self.height = h;
        self.tex_width = w;
        self.width = w;
    }

    /// FECK2Element2D.cpp 0x00292FF0 — `ChangeTextureVCD`
    pub fn change_texture_vcd(&mut self, vcd: &str) {
        if self.dynamic {
            self.vcd = vcd.to_string();
        } else {
            log::error!("CElement2D::ChangeTextureVCD() - Trying to change TextureVCD of CElement object with static strings!");
        }
    }

    /// FECK2Element2D.cpp 0x00293120 — `Fade`: to `percent` over `time`, or a pulse step when `time` is 0.
    pub fn fade(&mut self, percent: i32, time: i32, delay: i32) -> bool {
        self.fade_delay = delay;
        if time != 0 {
            if self.fade_left != 0 {
                let a = self.alpha_target as u8;
                self.colour_top = with_alpha(self.colour_top, a);
                self.colour_bottom = with_alpha(self.colour_bottom, a);
            }
            self.alpha_start = alpha(self.colour_top) as i32;
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
            self.fade_time = 0;
            self.fade_left = 0;
            let a = alpha(self.colour_top) as i32;
            if step <= 0 {
                if a == 0 {
                    return false;
                }
                let n = if a >= -step { (a + step) as u8 } else { 0 };
                self.colour_top = with_alpha(self.colour_top, n);
            } else {
                let max = (crate::feck2::feck2_manager::COLOUR_WHITE >> 24) as i32;
                if a == max {
                    return false;
                }
                let v = (step + a) as u32;
                let n = if v <= max as u32 { v as u8 } else { max as u8 };
                self.colour_top = with_alpha(self.colour_top, n);
            }
            self.colour_bottom = with_alpha(self.colour_bottom, alpha(self.colour_top));
            true
        }
    }

    /// FECK2Element2D.cpp 0x002930A0 — `Pulse`
    pub fn pulse(&mut self, percent: i32, time: i32, delay: i32) -> bool {
        if percent == -1 {
            self.pulse = 0;
            self.fade_time = 0;
            self.fade_left = 0;
        } else {
            self.pulse = percent;
            self.fade(100 - percent, time, delay);
        }
        true
    }

    /// FECK2Element2D.cpp 0x002922A0 — `Update` (fade step)
    pub fn update(&mut self, frame_time: f32) {
        if self.fade_left == 0 {
            return;
        }
        if self.fade_delay <= 0 {
            self.fade_delay = 0;
            self.fade_left = (self.fade_left as f32 - frame_time) as i32;
            if self.fade_left > 0 {
                let d = self.alpha_target - self.alpha_start;
                let a = (1.0 - (self.fade_left as f32 * 1.0) / (self.fade_time as f32 * 1.0)) * 90.0;
                let s = (a * 0.017_453_292_f32).sin();
                let v = clamp((self.alpha_start as f32 + d as f32 * s) as i32 as f32, 0.0, 255.0) as u8;
                self.colour_top = with_alpha(self.colour_top, v);
                self.colour_bottom = with_alpha(self.colour_bottom, v);
            } else if self.pulse != 0 {
                self.fade_left = self.fade_time;
                self.alpha_start = self.alpha_target;
                let a = self.alpha_target as u8;
                self.colour_top = with_alpha(self.colour_top, a);
                self.colour_bottom = with_alpha(self.colour_bottom, a);
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
                self.colour_top = with_alpha(self.colour_top, a);
                self.colour_bottom = with_alpha(self.colour_bottom, a);
                self.fade_left = 0;
                self.fade_time = 0;
            }
        } else {
            self.fade_delay = (self.fade_delay as f32 - frame_time) as i32;
        }
    }

    /// FECK2Element2D.cpp 0x00292510 — `Render`
    pub fn render(&mut self, frame_time: f32, out: &mut DrawList) {
        if self.base.move_left != 0 {
            self.base.update_move(frame_time);
        }
        if self.fade_left != 0 {
            self.update(frame_time);
        }
        let (u0, u1) = if self.has_flags(FLAG_FLIP_U) { (4096u16, 0u16) } else { (0, 4096) };
        let (v0, v1) = if self.has_flags(FLAG_FLIP_V) { (4096u16, 0u16) } else { (0, 4096) };
        let mut aspect = 1.0f32;
        let mut prescale = 1.0f32;
        let mut has_prescale = false;
        if !self.has_flags(FLAG_NO_ASPECT) {
            aspect = if crate::game_options::widescreen() { ASPECT_16_9 } else { ASPECT_4_3 };
            if self.has_flags(FLAG_ASPECT_4_3) {
                prescale = PRESCALE_4_3;
                has_prescale = true;
            } else if self.has_flags(FLAG_ASPECT_16_9) {
                prescale = PRESCALE_16_9;
                has_prescale = true;
            }
        }
        let h = self.height as f32;
        let w = self.width as f32;
        let y0 = (-self.pivot[1] - 0.5) * h;
        let x0 = (-self.pivot[0] - 0.5) * w;
        let mut c = [[x0, y0], [x0 + w, y0], [x0, y0 + h], [x0 + w, y0 + h]];
        if has_prescale {
            for p in &mut c {
                p[0] *= self.scale * prescale;
                p[1] *= self.scale;
            }
        }
        let (sn, cs) = self.rotation.sin_cos();
        let sx = self.scale / aspect;
        let sy = self.scale;
        let pos = self.base.position;
        let mut q = Quad {
            texture: self.texture,
            pos: [[0.0; 2]; 4],
            uv: [
                [u0 as f32 / 4096.0, v0 as f32 / 4096.0],
                [u1 as f32 / 4096.0, v0 as f32 / 4096.0],
                [u0 as f32 / 4096.0, v1 as f32 / 4096.0],
                [u1 as f32 / 4096.0, v1 as f32 / 4096.0],
            ],
            colour: [self.colour_top, self.colour_top, self.colour_bottom, self.colour_bottom],
        };
        for (i, p) in c.iter().enumerate() {
            let x = p[0] * sx;
            let y = p[1] * sy;
            q.pos[i] = [x * cs - y * sn + pos[0], x * sn + y * cs + pos[1]];
        }
        out.push(q);
    }
}

/// `TextureManager` load through the pack the element names (`\FE\FE.vcd`): find by name, else load the `.vtf`.
pub fn load_texture(tex: &mut crate::render2d::TextureStore, vcd: &str, name: &str) -> TextureId {
    if name.is_empty() {
        return 0;
    }
    let crc = visos::handy::handy_crc::name_hash(name);
    let id = tex.find(crc);
    if id != 0 {
        return id;
    }
    if vcd.is_empty() {
        return 0;
    }
    let Ok(v) = crate::feck2::feck2_manager::open_vcd(vcd) else {
        log::warn!("CElement2D: cannot open {}", vcd);
        return 0;
    };
    match v.get(&format!("{}.vtf", name)).and_then(crate::vtf::decode) {
        Some(img) => tex.add(crc, img),
        None => {
            log::warn!("CElement2D: texture {} not in {}", name, vcd);
            0
        }
    }
}
