//! src\Osd\OSDManager.cpp — `COSDManager` (gpOSDManager `dword_6FD200`): a pool of OSD items, each a text
//! (char buffer or string-table handle) or a texture, laid out by anchor/justify flags in a 1333x1000 virtual
//! screen with **y pointing up**, and drawn with the game font set into the same 512x512 2D target the FE uses.
//!
//! Mapping (0x000EF6D0 / 0x000EF710): `sx = x / 1333 * 512`, `sy = 512 - y / 1000 * 512`; pixel sizes go back
//! with 0x000EF7E0 / 0x000EF810 (`w / 512 * 1333`, `h / 512 * 1000`). `flt_628E34` = 512 is the 2D target.
//! Graphic items (0x000EE2D0) are created and laid out but not drawn here.

use crate::font_helper::font_helper::CFontHelper;
use crate::fonts::font_manager::wide;
use crate::render2d::{Colour, DrawList, TextureStore};
use crate::string_manager::string_manager::CStringManager;

/// `flt_636BA4` / `flt_636BA8`
pub const OSD_W: f32 = 1333.0;
pub const OSD_H: f32 = 1000.0;
/// `flt_628E34` (Speedo 0x000F7A70 / 0x000F7A90)
pub const TARGET: f32 = 512.0;

/// OSDManager.cpp 0x000EF6D0 — OSD x to 2D target x
pub fn to_screen_x(x: f32) -> f32 {
    x / OSD_W * TARGET
}
/// OSDManager.cpp 0x000EF710 — OSD y (up) to 2D target y (down)
pub fn to_screen_y(y: f32) -> f32 {
    TARGET - y / OSD_H * TARGET
}
/// OSDManager.cpp 0x000EF7E0 — pixel width to OSD units
pub fn width_to_osd(w: f32) -> f32 {
    w / TARGET * OSD_W
}
/// OSDManager.cpp 0x000EF810 — pixel height to OSD units
pub fn height_to_osd(h: f32) -> f32 {
    h / TARGET * OSD_H
}

/// item flags (+4)
pub mod flag {
    pub const PERMANENT: u32 = 0x1; // life < 0
    pub const JUSTIFY_LEFT: u32 = 0x2;
    pub const JUSTIFY_RIGHT: u32 = 0x4;
    pub const JUSTIFY_CENTRE_X: u32 = 0x8;
    pub const JUSTIFY_TOP: u32 = 0x10; // just_y 0
    pub const JUSTIFY_BOTTOM: u32 = 0x20; // just_y -height
    pub const JUSTIFY_CENTRE_Y: u32 = 0x40;
    pub const ANCHOR_LEFT: u32 = 0x80;
    pub const ANCHOR_RIGHT: u32 = 0x100; // x = 1333
    pub const ANCHOR_CENTRE_X: u32 = 0x200;
    pub const ANCHOR_TOP: u32 = 0x400; // y = 0 (screen bottom: OSD y points up)
    pub const ANCHOR_BOTTOM: u32 = 0x800; // y = 1000 (screen top)
    pub const ANCHOR_CENTRE_Y: u32 = 0x1000;
    pub const SHADOW: u32 = 0x2000;
    pub const DISABLED: u32 = 0x4000;
    pub const HIDDEN: u32 = 0x8000;
    pub const ROTATED: u32 = 0x10000;
    pub const SCALED: u32 = 0x20000;
    pub const HALF_UP: u32 = 0x40000;
    pub const IN_3D: u32 = 0x80000;
}

/// `SOSDItem` (pool entry, ~100 bytes)
#[derive(Clone, Debug, Default)]
pub struct SOSDItem {
    pub layer: i16,             // +0
    pub flags: u32,             // +4
    pub text: Option<String>,   // +8 char buffer (owned by the screen in the original)
    pub string: Option<u32>,    // +12 string table handle (crc here)
    pub font: Option<usize>,    // +16 font id
    pub texture: Option<u32>,   // +20
    pub colour: Colour,         // +32
    pub x: f32,                 // +36
    pub y: f32,                 // +40
    pub z: f32,                 // +44
    pub just_x: f32,            // +48
    pub just_y: f32,            // +52
    pub unk_56: u32,            // +56
    pub unk_60: u32,            // +60
    pub scale_x: f32,           // +64
    pub scale_y: f32,           // +68
    pub width: f32,             // +72
    pub height: f32,            // +76
    pub rotation: f32,          // +80
    pub life: f32,              // +84 seconds, < 0 = permanent
    pub zoom_x: f32,            // +88
    pub zoom_y: f32,            // +92
    pub mask: u32,              // +96 screens the item is hidden on
    pub free: bool,             // back in the pool (0x000ED960)
    /// creation order (the list position among equal layers: 0x000EDB00 links a new item before older ones)
    pub seq: u32,
}

impl SOSDItem {
    /// OSDInGameScreen.cpp 0x000FAD20 — `SetVisible`
    pub fn set_visible(&mut self, v: bool) {
        if v {
            self.flags &= !flag::HIDDEN;
        } else {
            self.flags |= flag::HIDDEN;
        }
    }

    /// OSDManager.cpp 0x000EFB20 — `SetLife`
    pub fn set_life(&mut self, t: f32) {
        self.life = t;
        if self.life >= 0.0 {
            self.flags &= !flag::PERMANENT;
        } else {
            self.flags |= flag::PERMANENT;
        }
    }

    /// GameMenuManager.cpp 0x000ACC30 / 0x000ACCC0 — `SetScaleX/Y`
    fn update_scale_flag(&mut self) {
        let one = |v: f32| (0.9999..=1.0001).contains(&v);
        if one(self.scale_x) && one(self.scale_y) {
            self.flags &= !flag::SCALED;
        } else {
            self.flags |= flag::SCALED;
        }
    }
}

/// `COSDManager`
#[derive(Default)]
pub struct COSDManager {
    pub items: Vec<SOSDItem>, // +4 list / +16 pool
    pub last_time: f32,       // +32
    pub hidden_mask: u32,     // +36 items whose mask intersects are skipped
    pub disabled: bool,       // +29 bit 0
    /// items created so far (`SOSDItem::seq`)
    pub created: u32,
}

pub type ItemId = usize;

impl COSDManager {
    /// OSDManager.cpp 0x000EDD00 / 0x000EFE40 — `CreateItem` (+ the pool take)
    #[allow(clippy::too_many_arguments)]
    pub fn create_item(&mut self, x: f32, y: f32, layer: i16, life: f32, flags: u32, mask: u32) -> ItemId {
        let mut it = SOSDItem {
            layer,
            flags,
            colour: crate::feck2::feck2_manager::COLOUR_WHITE,
            x,
            y,
            scale_x: 1.0,
            scale_y: 1.0,
            zoom_x: 1.0,
            zoom_y: 1.0,
            mask,
            seq: self.created,
            ..Default::default()
        };
        self.created = self.created.wrapping_add(1);
        it.set_life(life);
        it.update_scale_flag();
        // OSDManager.cpp 0x000EDB00 links the item before the first one with a layer <= its own (list = layer
        // descending, newest first among equals); here ids are pool slots and `render` walks them in that
        // order. 0x000EFE40 takes a free pool entry.
        if let Some(i) = self.items.iter().position(|o| o.free) {
            self.items[i] = it;
            return i;
        }
        self.items.push(it);
        self.items.len() - 1
    }

    /// OSDManager.cpp 0x000ED960 — `ReleaseItem`: back to the pool.
    pub fn release(&mut self, id: ItemId) {
        if let Some(it) = self.items.get_mut(id) {
            *it = SOSDItem { free: true, ..Default::default() };
        }
    }

    /// OSDManager.cpp 0x000EDF30 — text item from a char buffer.
    #[allow(clippy::too_many_arguments)]
    pub fn create_text_item(&mut self, fonts: &CFontHelper, tex: &TextureStore, text: &str, font: usize, x: f32, y: f32, layer: i16, life: f32, flags: u32, mask: u32) -> ItemId {
        let id = self.create_item(x, y, layer, life, flags, mask);
        let it = self.item_mut(id);
        it.text = Some(text.to_string());
        it.font = Some(font);
        if let Some(f) = fonts.font(font) {
            it.width = width_to_osd(f.get_string_width(&wide(text), 0, tex) as f32);
            it.height = height_to_osd(f.line_height as f32);
        }
        id
    }

    /// OSDManager.cpp 0x000EE200 / 0x000EE060 — text item from the string table.
    #[allow(clippy::too_many_arguments)]
    pub fn create_string_item(&mut self, fonts: &CFontHelper, strings: &CStringManager, tex: &TextureStore, crc: u32, font: usize, x: f32, y: f32, layer: i16, life: f32, flags: u32, mask: u32) -> Option<ItemId> {
        // GetStringHandle: a crc the index does not know gives no handle
        if strings.index.as_ref().is_none_or(|i| i.find(crc).is_none()) {
            log::warn!("OSD: Unable to find message with crc [{}]", crc as i32);
            return None;
        }
        let id = self.create_item(x, y, layer, life, flags, mask);
        let it = self.item_mut(id);
        it.string = Some(crc);
        it.font = Some(font);
        Self::measure(it, fonts, strings, tex);
        Some(id)
    }

    /// OSDManager.cpp 0x000EE2D0 — graphic item.
    #[allow(clippy::too_many_arguments)]
    pub fn create_graphic_item(&mut self, texture: u32, x: f32, y: f32, layer: i16, w: f32, h: f32, life: f32, flags: u32, mask: u32) -> ItemId {
        let id = self.create_item(x, y, layer, life, flags, mask);
        let it = self.item_mut(id);
        it.texture = Some(texture);
        it.width = w;
        it.height = h;
        id
    }

    pub fn item_mut(&mut self, id: ItemId) -> &mut SOSDItem {
        &mut self.items[id]
    }

    fn measure(it: &mut SOSDItem, fonts: &CFontHelper, strings: &CStringManager, tex: &TextureStore) {
        let Some(f) = it.font.and_then(|i| fonts.font(i)) else { return };
        if let Some(t) = &it.text {
            it.width = width_to_osd(f.get_string_width(&wide(t), 0, tex) as f32);
            it.height = height_to_osd(f.line_height as f32);
        }
        if let Some(crc) = it.string {
            let s = strings.get_string(crc);
            let (w, h, ..) = f.get_string_extent(&s, 256.0, 256.0, 512, tex);
            it.width = width_to_osd(w);
            it.height = height_to_osd(h);
        }
    }

    /// OSDManager.cpp 0x000EE5F0 — `UpdateItem`: measure, then anchor and justify.
    pub fn layout_item(it: &mut SOSDItem, fonts: &CFontHelper, strings: &CStringManager, tex: &TextureStore) {
        Self::measure(it, fonts, strings, tex);
        let f = it.flags;
        if f & flag::ANCHOR_LEFT != 0 {
            it.x = 0.0;
        } else if f & flag::ANCHOR_RIGHT != 0 {
            it.x = OSD_W;
        } else if f & flag::ANCHOR_CENTRE_X != 0 {
            it.x = 666.0; // 0x44268000
        }
        if f & flag::ANCHOR_TOP != 0 {
            it.y = 0.0;
        } else if f & flag::ANCHOR_BOTTOM != 0 {
            it.y = OSD_H;
        } else if f & flag::ANCHOR_CENTRE_Y != 0 {
            it.y = OSD_H / 2.0;
        }
        if f & flag::JUSTIFY_LEFT != 0 {
            it.just_x = 0.0;
        } else if f & flag::JUSTIFY_RIGHT != 0 {
            it.just_x = -it.width;
        } else if f & flag::JUSTIFY_CENTRE_X != 0 {
            it.just_x = -(it.width / 2.0);
        }
        if f & flag::JUSTIFY_BOTTOM != 0 {
            it.just_y = -it.height;
        } else if f & flag::JUSTIFY_TOP != 0 {
            it.just_y = 0.0;
        } else if f & flag::JUSTIFY_CENTRE_Y != 0 {
            it.just_y = -(it.height / 2.0);
        }
        if f & flag::HALF_UP != 0 {
            it.just_y = it.just_y / 2.0;
        }
    }

    /// OSDManager.cpp 0x000EE910 — `Update`: lay out every live item and run down timed ones.
    pub fn update(&mut self, time: f32, fonts: &CFontHelper, strings: &CStringManager, tex: &TextureStore) {
        if self.last_time <= 0.00000001 {
            self.last_time = time;
        }
        let dt = time - self.last_time;
        let mask = self.hidden_mask;
        for it in self.items.iter_mut() {
            if it.free || it.flags & 0xC000 != 0 || mask & it.mask != 0 {
                continue;
            }
            Self::layout_item(it, fonts, strings, tex);
            if it.flags & flag::PERMANENT == 0 {
                let l = it.life - dt;
                it.set_life(l);
                if it.life < 0.0 {
                    // 0x000ED960: item expired -> released
                    it.flags |= flag::HIDDEN;
                }
            }
        }
        self.last_time = time;
    }

    /// OSDManager.cpp 0x000EEA70 — draw a text item (optional drop shadow one pixel down-right).
    fn render_text(it: &SOSDItem, fonts: &CFontHelper, strings: &CStringManager, tex: &TextureStore, out: &mut DrawList) {
        let Some(f) = it.font.and_then(|i| fonts.font(i)) else { return };
        let x = to_screen_x(it.zoom_x * it.just_x + it.x);
        let y = to_screen_y(it.zoom_y * it.just_y + it.y);
        let text: Option<Vec<u16>> = match (&it.text, it.string) {
            (Some(t), _) => Some(wide(t)),
            (None, Some(c)) => Some(strings.get_string(c)),
            _ => None,
        };
        let Some(s) = text else { return };
        // zoom about the item origin (Renderer 0x002AC450): applied to the emitted quads below
        let mut list = DrawList::default();
        if it.flags & flag::SHADOW != 0 {
            let shadow = (crate::feck2::feck2_manager::COLOUR_BLACK & 0x00FF_FFFF) | (it.colour & 0xFF00_0000);
            if it.text.is_some() {
                f.draw_string_at(x + 1.0, y + 1.0, &s, shadow, 512, tex, &mut list);
            } else {
                f.draw_string_rect(x + 1.0, y + 1.0, x + 257.0, y + 257.0, &s, shadow, 512, None, tex, &mut list);
            }
        }
        if it.text.is_some() {
            f.draw_string_at(x, y, &s, it.colour, 512, tex, &mut list);
        } else {
            f.draw_string_rect(x, y, x + 256.0, y + 256.0, &s, it.colour, 512, None, tex, &mut list);
        }
        let (zx, zy) = (it.zoom_x, it.zoom_y);
        for mut q in list.quads {
            for p in q.pos.iter_mut() {
                if zx != 1.0 || zy != 1.0 {
                    p[0] = (p[0] - x) * zx + x;
                    p[1] = (p[1] - y) * zy + y;
                }
            }
            out.quads.push(q);
        }
    }

    /// OSDManager.cpp 0x000EF4A0 / 0x000EEEC0 — `Render`: visible text items in layer order.
    pub fn render(&self, fonts: &CFontHelper, strings: &CStringManager, tex: &TextureStore, out: &mut DrawList) {
        if self.disabled {
            return;
        }
        // 0x000EF4A0 walks the list from the head: highest layer first, so low layers end up on top
        let mut order: Vec<usize> = (0..self.items.len()).collect();
        order.sort_by_key(|&i| (std::cmp::Reverse(self.items[i].layer), std::cmp::Reverse(self.items[i].seq)));
        for i in order {
            let it = &self.items[i];
            if it.free || it.flags & flag::HIDDEN != 0 || self.hidden_mask & it.mask != 0 {
                continue;
            }
            if it.flags & flag::IN_3D == 0 && it.font.is_some() && (it.text.is_some() || it.string.is_some()) {
                Self::render_text(it, fonts, strings, tex, out);
            } else if it.texture.is_some_and(|t| t != 0) {
                // 0x000EEEC0 (rotated) forwards to the same function
                Self::render_graphic(it, out);
            }
        }
    }

    /// OSDManager.cpp 0x000EF5F0 / 0x000EEE00 / FontManager.cpp 0x002AD5A0 / 0x002AD680 — the 3D pass (GameManager 0x000B9010): every shown text item with the in-3D flag
    /// is drawn at its world position (+36..+44) by 0x000EEE00 -> FontManager 0x002AD5A0 / 0x002AD680: projected
    /// (behind the camera: skipped), faded from `zoom_y` to 1e6 by clip w, on top when `zoom_x != 0`, centred when
    /// flag 0x100000. The manager mask is not checked here.
    pub fn render_3d(&self, fonts: &CFontHelper, tex: &TextureStore, view_proj: &[f32; 16], out: &mut DrawList) {
        if self.disabled {
            return;
        }
        let mut order: Vec<usize> = (0..self.items.len()).collect();
        order.sort_by_key(|&i| (std::cmp::Reverse(self.items[i].layer), std::cmp::Reverse(self.items[i].seq)));
        for i in order {
            let it = &self.items[i];
            if it.free || it.flags & flag::HIDDEN != 0 || it.flags & flag::IN_3D == 0 {
                continue;
            }
            let (Some(t), Some(f)) = (&it.text, it.font.and_then(|i| fonts.font(i))) else { continue };
            let m = view_proj;
            let c = |r: usize| m[r] * it.x + m[4 + r] * it.y + m[8 + r] * it.z + m[12 + r];
            let (cx, cy, w) = (c(0), c(1), c(3));
            // ShaderManager 0x002A7840: not visible behind the eye
            if w <= 0.0 {
                continue;
            }
            let (near, far) = (it.zoom_y, 1_000_000.0f32);
            let mut colour = it.colour;
            if w > near {
                if w > far {
                    continue;
                }
                let fade = 1.0 - (w - near) / (far - near);
                let a = ((colour >> 24) as f32 * fade) as i32 as u32 & 0xFF;
                colour = (colour & 0x00FF_FFFF) | (a << 24);
            }
            let x = (cx / w * 0.5 + 0.5) * TARGET;
            let y = (0.5 - cy / w * 0.5) * TARGET;
            let flags = (it.flags & 0x100000 != 0) as u32;
            f.draw_string_at(x, y, &wide(t), colour, flags, tex, out);
        }
    }

    /// OSDManager.cpp 0x000EEEF0 — `RenderGraphicItem`: a 4-vertex strip from the item's bottom-left corner up,
    /// uvs +56 / +60 origin and +64 / +68 size (stored as 4096 fixed point), item colour, alpha blend (mode 3).
    fn render_graphic(it: &SOSDItem, out: &mut DrawList) {
        let q = |v: f32| ((v * 4096.0) as i32 as i16) as f32 / 4096.0;
        let (u0, v0) = (f32::from_bits(it.unk_56), f32::from_bits(it.unk_60));
        let (u, v, us, vs) = (q(u0), q(v0), q(u0 + it.scale_x), q(v0 + it.scale_y));
        let x = to_screen_x(it.x + it.just_x);
        let y = to_screen_y(it.y + it.just_y);
        let w = it.width / OSD_W * TARGET;
        let h = it.height / OSD_H * TARGET;
        out.push(crate::render2d::Quad {
            texture: it.texture.unwrap_or(0),
            pos: [[x, y - h], [x + w, y - h], [x, y], [x + w, y]],
            uv: [[u, v], [us, v], [u, vs], [us, vs]],
            colour: [it.colour; 4],
        });
    }
}

/// OSDManager.cpp 0x000EF840 — `FormatTime`: `m:ss.hh`, or `m:ss` when `no_hundredths`.
pub fn format_time(t: f32, no_hundredths: bool) -> String {
    let mut out = String::new();
    let mut t = t;
    if t < 0.0 {
        t = 0.0 - t;
        out.push('-');
    }
    let mins = (t / 60.0) as i32;
    let secs = (t as i32) % 60;
    let hund = ((t - (t as i32) as f32) * 100.0) as i32;
    if no_hundredths {
        out += &format!("{}:{:02}", mins, secs);
    } else {
        out += &format!("{}:{:02}.{:02}", mins, secs, hund);
    }
    out
}
