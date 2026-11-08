//! platform layer for the frontend's 2D drawing: replaces the xbox `VertexStream` / `Renderer::DrawPrimitive`
//! path used by FontManager and the FECK2 2D elements. Everything above this (layout, metrics, uv maths) is the
//! original code; this layer only collects textured, vertex-coloured quads and rasterises them.
//!
//! screen space is the original 640x480 virtual space; the same draw list is rasterised on the cpu for headless
//! captures and uploaded to a bevy image for display.

use std::collections::HashMap;

use crate::vtf::Rgba;

/// virtual screen the frontend lays out in: 512x512 stretched over the 640x480 display (FE fonts are drawn at
/// 0.75 horizontal scale, which with the 1.25 x stretch gives square glyphs again).
pub const SCREEN_W: u32 = 512;
pub const SCREEN_H: u32 = 512;

/// texture handle (`TextureManager` id; 0 = none).
pub type TextureId = u32;

/// `VISOS_Colour` (argb, alpha in the top byte).
pub type Colour = u32;

pub const WHITE: Colour = 0xFFFF_FFFF;

/// one textured, vertex-coloured quad. corners are tl, tr, bl, br as the original vertex order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub texture: TextureId,
    pub pos: [[f32; 2]; 4],
    pub uv: [[f32; 2]; 4],
    pub colour: [Colour; 4],
}

/// textures loaded by name (`TextureManager` replacement): id = index + 1.
#[derive(Default)]
pub struct TextureStore {
    pub images: Vec<Rgba>,
    pub names: HashMap<u32, TextureId>,
}

impl TextureStore {
    /// register an image under the crc of its name (`crc32_no_case`), returning its id.
    pub fn add(&mut self, name_crc: u32, img: Rgba) -> TextureId {
        if let Some(&id) = self.names.get(&name_crc) {
            self.images[id as usize - 1] = img;
            return id;
        }
        self.images.push(img);
        let id = self.images.len() as TextureId;
        self.names.insert(name_crc, id);
        id
    }

    /// `TextureManager::FindTexture` (0x0029F650): 0 if not loaded.
    pub fn find(&self, name_crc: u32) -> TextureId {
        self.names.get(&name_crc).copied().unwrap_or(0)
    }

    /// `TextureManager::GetSize` (0x0029F7F0)
    pub fn size(&self, id: TextureId) -> (u32, u32) {
        if id == 0 {
            return (0, 0);
        }
        self.images.get(id as usize - 1).map(|i| (i.width, i.height)).unwrap_or((0, 0))
    }

    pub fn get(&self, id: TextureId) -> Option<&Rgba> {
        if id == 0 { None } else { self.images.get(id as usize - 1) }
    }

    /// load every `.vtf` of a pack (`TextureManager::LoadVCD`).
    pub fn load_vcd(&mut self, game_path: &str) -> usize {
        let Ok(v) = visos::file::Vcd::open(game_path) else {
            log::warn!("TextureStore: cannot open {}", game_path);
            return 0;
        };
        let mut n = 0;
        for e in v.entries() {
            let lower = e.name.to_ascii_lowercase();
            let Some(stem) = lower.strip_suffix(".vtf") else { continue };
            if let Some(img) = v.data(&e).and_then(crate::vtf::decode) {
                self.add(visos::handy::handy_crc::name_hash(stem), img);
                n += 1;
            }
        }
        n
    }
}

/// widescreen layout for a window of this aspect: (image width in pixels at the 4:3 layout's pixels per unit, extra
/// virtual units). Narrower than 4:3: unchanged.
pub fn wide_size(aspect: f32) -> (u32, f32) {
    let k = aspect / (4.0 / 3.0);
    if k <= 1.001 {
        return (640, 0.0);
    }
    ((640.0 * k).round() as u32, SCREEN_W as f32 * (k - 1.0))
}

/// the frame's quads in submission order.
#[derive(Default)]
pub struct DrawList {
    pub quads: Vec<Quad>,
}

impl DrawList {
    pub fn clear(&mut self) {
        self.quads.clear();
    }

    /// axis aligned quad.
    #[allow(clippy::too_many_arguments)]
    pub fn rect(&mut self, texture: TextureId, x0: f32, y0: f32, x1: f32, y1: f32, u0: f32, v0: f32, u1: f32, v1: f32, colour: Colour) {
        self.quads.push(Quad {
            texture,
            pos: [[x0, y0], [x1, y0], [x0, y1], [x1, y1]],
            uv: [[u0, v0], [u1, v0], [u0, v1], [u1, v1]],
            colour: [colour; 4],
        });
    }

    pub fn push(&mut self, q: Quad) {
        self.quads.push(q);
    }

    /// Widescreen (a deliberate deviation from the 4:3 original): the quads from `from` on were laid out in the
    /// 512-unit virtual screen; make room for `extra` more units across. A full-width bar (under a quarter of the
    /// height) repeats sideways over the whole width, any other full-width quad stretches to it, and the rest keep their
    /// size and move by their anchor: `anchor(centre x)` = 0 left, 0.5 centre, 1 right.
    pub fn widen(&mut self, from: usize, extra: f32, anchor: impl Fn(f32) -> f32) {
        if extra <= 0.0 {
            return;
        }
        let full = SCREEN_W as f32;
        let total = full + extra;
        let bbox = |q: &Quad| {
            let xs = q.pos.iter().map(|p| p[0]);
            let ys = q.pos.iter().map(|p| p[1]);
            [xs.clone().fold(f32::MAX, f32::min), ys.clone().fold(f32::MAX, f32::min), xs.fold(f32::MIN, f32::max), ys.fold(f32::MIN, f32::max)]
        };
        let is_wide = |b: &[f32; 4]| b[0] <= 1.0 && b[2] >= full - 1.0;
        // consecutive quads that touch (within a glyph gap) move together: a string / a gauge keeps its shape
        const GAP: f32 = 12.0;
        // a label and its value on one line are further apart than glyphs
        const GAP_X: f32 = 48.0;
        let mut group_of = Vec::with_capacity(self.quads.len() - from);
        let mut groups: Vec<[f32; 4]> = Vec::new();
        for q in &self.quads[from..] {
            let b = bbox(q);
            let join = !is_wide(&b)
                && groups.last().is_some_and(|g| b[0] <= g[2] + GAP_X && b[2] >= g[0] - GAP_X && b[1] <= g[3] + GAP && b[3] >= g[1] - GAP)
                && group_of.last().is_some_and(|&i: &usize| i == groups.len() - 1);
            if join {
                let g = groups.last_mut().unwrap();
                *g = [g[0].min(b[0]), g[1].min(b[1]), g[2].max(b[2]), g[3].max(b[3])];
                group_of.push(groups.len() - 1);
            } else {
                groups.push(b);
                group_of.push(groups.len() - 1);
            }
        }
        let mut out: Vec<Quad> = Vec::with_capacity(group_of.len());
        for (n, q) in self.quads[from..].iter().enumerate() {
            let b = bbox(q);
            if is_wide(&b) {
                if b[3] - b[1] < full * 0.25 {
                    // bar: copies from the centred original outwards
                    let base = extra * 0.5;
                    let mut k = -((base / full).ceil() as i32);
                    while (k as f32) * full + base < total {
                        let d = k as f32 * full + base;
                        if d + full > 0.0 {
                            let mut c = *q;
                            for p in &mut c.pos {
                                p[0] += d;
                            }
                            out.push(c);
                        }
                        k += 1;
                    }
                } else {
                    let mut c = *q;
                    for p in &mut c.pos {
                        p[0] *= total / full;
                    }
                    out.push(c);
                }
                continue;
            }
            let g = groups[group_of[n]];
            let d = extra * anchor((g[0] + g[2]) * 0.5);
            let mut c = *q;
            for p in &mut c.pos {
                p[0] += d;
            }
            out.push(c);
        }
        self.quads.truncate(from);
        self.quads.extend(out);
    }

    /// rasterise into an rgba frame of `w` x `h` (scaled from the virtual screen), black background.
    pub fn rasterise(&self, tex: &TextureStore, w: u32, h: u32) -> Rgba {
        let fb = self.raster_fb(tex, w, h);
        let data = fb.chunks_exact(4).flat_map(|c| [to8(c[0]), to8(c[1]), to8(c[2]), 255]).collect();
        Rgba { width: w, height: h, data }
    }

    fn raster_fb(&self, tex: &TextureStore, w: u32, h: u32) -> Vec<f32> {
        self.raster_fb_virt(tex, w, h, SCREEN_W as f32)
    }

    /// `rasterise_overlay` with a virtual screen `virt_w` wide (widescreen: more than `SCREEN_W` units across, the same
    /// pixels per unit as the 4:3 layout)
    pub fn rasterise_overlay_virt(&self, tex: &TextureStore, w: u32, h: u32, virt_w: f32) -> Rgba {
        let fb = self.raster_fb_virt(tex, w, h, virt_w);
        let data = fb
            .chunks_exact(4)
            .flat_map(|c| {
                let a = c[3];
                let un = |v: f32| if a > 0.0 { v / a } else { 0.0 };
                [to8(un(c[0])), to8(un(c[1])), to8(un(c[2])), to8(a)]
            })
            .collect();
        Rgba { width: w, height: h, data }
    }

    fn raster_fb_virt(&self, tex: &TextureStore, w: u32, h: u32, virt_w: f32) -> Vec<f32> {
        let mut fb = vec![0f32; (w * h * 4) as usize];
        let sx = w as f32 / virt_w;
        let sy = h as f32 / SCREEN_H as f32;
        for q in &self.quads {
            // two triangles: tl tr bl, tr br bl
            for tri in [[0usize, 1, 2], [1, 3, 2]] {
                raster_tri(&mut fb, w, h, sx, sy, q, tri, tex.get(q.texture));
            }
        }
        fb
    }

    /// rasterise onto a transparent frame (straight alpha), for an overlay drawn over the 3D view.
    pub fn rasterise_overlay(&self, tex: &TextureStore, w: u32, h: u32) -> Rgba {
        let fb = self.raster_fb(tex, w, h);
        // fb holds premultiplied colour + coverage; un-premultiply for the image
        let data = fb
            .chunks_exact(4)
            .flat_map(|c| {
                let a = c[3];
                let un = |v: f32| if a > 0.0 { v / a } else { 0.0 };
                [to8(un(c[0])), to8(un(c[1])), to8(un(c[2])), to8(a)]
            })
            .collect();
        Rgba { width: w, height: h, data }
    }
}

fn to8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

fn unpack(c: Colour) -> [f32; 4] {
    [
        ((c >> 16) & 255) as f32 / 255.0,
        ((c >> 8) & 255) as f32 / 255.0,
        (c & 255) as f32 / 255.0,
        (c >> 24) as f32 / 255.0,
    ]
}

/// bilinear fetch, clamp addressing.
fn sample(img: &Rgba, u: f32, v: f32) -> [f32; 4] {
    let x = u * img.width as f32 - 0.5;
    let y = v * img.height as f32 - 0.5;
    let x0 = x.floor();
    let y0 = y.floor();
    let fx = x - x0;
    let fy = y - y0;
    let px = |xi: i32, yi: i32| {
        let xi = xi.clamp(0, img.width as i32 - 1) as u32;
        let yi = yi.clamp(0, img.height as i32 - 1) as u32;
        let o = ((yi * img.width + xi) * 4) as usize;
        let d = &img.data[o..o + 4];
        [d[0] as f32 / 255.0, d[1] as f32 / 255.0, d[2] as f32 / 255.0, d[3] as f32 / 255.0]
    };
    let (a, b, c, d) = (px(x0 as i32, y0 as i32), px(x0 as i32 + 1, y0 as i32), px(x0 as i32, y0 as i32 + 1), px(x0 as i32 + 1, y0 as i32 + 1));
    let mut out = [0.0; 4];
    for k in 0..4 {
        out[k] = (a[k] * (1.0 - fx) + b[k] * fx) * (1.0 - fy) + (c[k] * (1.0 - fx) + d[k] * fx) * fy;
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn raster_tri(fb: &mut [f32], w: u32, h: u32, sx: f32, sy: f32, q: &Quad, t: [usize; 3], img: Option<&Rgba>) {
    let p: Vec<[f32; 2]> = t.iter().map(|&i| [q.pos[i][0] * sx, q.pos[i][1] * sy]).collect();
    let area = (p[1][0] - p[0][0]) * (p[2][1] - p[0][1]) - (p[2][0] - p[0][0]) * (p[1][1] - p[0][1]);
    if area.abs() < 1e-6 {
        return;
    }
    let minx = p.iter().map(|v| v[0]).fold(f32::MAX, f32::min).floor().max(0.0) as i32;
    let maxx = p.iter().map(|v| v[0]).fold(f32::MIN, f32::max).ceil().min(w as f32) as i32;
    let miny = p.iter().map(|v| v[1]).fold(f32::MAX, f32::min).floor().max(0.0) as i32;
    let maxy = p.iter().map(|v| v[1]).fold(f32::MIN, f32::max).ceil().min(h as f32) as i32;
    let cols: Vec<[f32; 4]> = t.iter().map(|&i| unpack(q.colour[i])).collect();
    for y in miny..maxy {
        for x in minx..maxx {
            let cx = x as f32 + 0.5;
            let cy = y as f32 + 0.5;
            let w0 = ((p[1][0] - cx) * (p[2][1] - cy) - (p[2][0] - cx) * (p[1][1] - cy)) / area;
            let w1 = ((p[2][0] - cx) * (p[0][1] - cy) - (p[0][0] - cx) * (p[2][1] - cy)) / area;
            let w2 = 1.0 - w0 - w1;
            // top-left style fill: include edges on one side only
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }
            let mut c = [0f32; 4];
            for k in 0..4 {
                c[k] = cols[0][k] * w0 + cols[1][k] * w1 + cols[2][k] * w2;
            }
            if let Some(img) = img {
                let u = q.uv[t[0]][0] * w0 + q.uv[t[1]][0] * w1 + q.uv[t[2]][0] * w2;
                let v = q.uv[t[0]][1] * w0 + q.uv[t[1]][1] * w1 + q.uv[t[2]][1] * w2;
                let s = sample(img, u, v);
                for k in 0..4 {
                    c[k] *= s[k];
                }
            }
            let o = ((y as u32 * w + x as u32) * 4) as usize;
            let a = c[3];
            for k in 0..3 {
                fb[o + k] = fb[o + k] * (1.0 - a) + c[k] * a;
            }
            fb[o + 3] = fb[o + 3] * (1.0 - a) + a;
        }
    }
}
