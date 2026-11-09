//! src\CircuitEditor\CHUDMapConstructor.cpp + src\OSDMap\COSDMapRender.cpp — the minimap.
//!
//! `CHUDMapConstructor` (0x0007DEE0) rasterises the track's inside and outside edge polylines (x/z) into a
//! square 8-bit image: 255 on the edges, 0x80 for the one pixel outline around them, 0 elsewhere; the palette
//! (COSDMapRender 0x000E85A0) gives every index the hud's map colour with alpha `max(index, min alpha)`.
//! `COSDMapRender::Update` (0x000E8E70) then shows it centred on the player, rotated by the heading + pi, with
//! a window of `900 / span` of the map (`dword_685FE0` = 900), and places the opponent / police blips.
//!
//! In the original the map is the texture of the 3D `hud_map_screen` model; here the window is drawn as a 2D
//! panel at the hud's map position through the same PROVISIONAL hud plane mapping as the speedo.
//! Track edges and positions come from game flow (`MapInput`).

use crate::render2d::{Colour, DrawList, Quad, TextureId, TextureStore};
use crate::vtf::Rgba;

/// `CHUDMapConstructor` inputs (+0 inside edge, +4 outside edge, +12/+16 counts, +20 size, +22 border, +36 loop)
pub struct HudMapDesc<'a> {
    pub inside: &'a [[f32; 3]],
    pub outside: &'a [[f32; 3]],
    pub size: u16,
    pub border: u16,
    pub closed: bool,
}

/// the built map: indices plus the world window it covers.
pub struct HudMap {
    pub size: u16,
    pub pixels: Vec<u8>, // [x * size + z] as the original indexes it
    pub min: f32,        // +24
    pub max: f32,        // +28
    pub scale: f32,      // +32 pixels per world unit
}

impl HudMap {
    /// CHUDMapConstructor.cpp 0x0007DEE0 — `Construct`
    pub fn construct(d: &HudMapDesc) -> HudMap {
        assert!(!d.inside.is_empty(), "m_dNumLeftVerts != 0");
        assert!(!d.outside.is_empty(), "m_dNumRightVerts != 0");
        assert!(d.size != 0, "m_dTextureDimension != 0");
        let n = d.size as usize;
        // one bound over both axes and both edges (a square window)
        let mut min = d.inside[0][0];
        let mut max = min;
        for edge in [d.inside, d.outside] {
            for v in edge.iter().rev() {
                for c in [v[0], v[2]] {
                    if min <= c {
                        if c > max {
                            max = c;
                        }
                    } else {
                        min = c;
                    }
                }
            }
        }
        let mut span = max - min;
        let pad = d.border as f32 * (span / d.size as f32);
        min -= pad;
        max += pad;
        span += pad * 2.0;
        let mut m = HudMap { size: d.size, pixels: vec![0; n * n], min, max, scale: d.size as f32 / span };
        for edge in [d.inside, d.outside] {
            for i in (1..edge.len()).rev() {
                m.line(edge[i], edge[i - 1], 255);
            }
            if d.closed {
                m.line(edge[edge.len() - 1], edge[0], 255);
            }
        }
        m.line(d.outside[d.outside.len() - 1], d.inside[d.inside.len() - 1], 255);
        m.line(d.outside[0], d.inside[0], 255);
        m.outline();
        m
    }

    /// CHUDMapConstructor.cpp 0x0007E4B0 — Bresenham line in map pixels (x major index, as the original).
    fn line(&mut self, a: [f32; 3], b: [f32; 3], v: u8) {
        let px = |c: f32| ((c - self.min) * self.scale) as i32;
        let (mut x, mut z) = (px(a[0]), px(a[2]));
        let (x1, z1) = (px(b[0]), px(b[2]));
        let dx = (x1 - x).abs();
        let dz = (z1 - z).abs();
        let sx = if x <= x1 { 1 } else { -1 };
        let sz = if z <= z1 { 1 } else { -1 };
        let n = self.size as i32;
        let put = |x: i32, z: i32, p: &mut Vec<u8>| {
            // the original writes unchecked; keep the write inside the buffer
            if (0..n).contains(&x) && (0..n).contains(&z) {
                p[(x * n + z) as usize] = v;
            }
        };
        if dx < dz {
            let (inc_a, inc_b) = (2 * dx, 2 * dx - 2 * dz);
            let mut e = 2 * dx - dz;
            let mut left = dz;
            while left >= 0 {
                put(x, z, &mut self.pixels);
                if e <= 0 {
                    z += sz;
                    e += inc_a;
                } else {
                    x += sx;
                    z += sz;
                    e += inc_b;
                }
                left -= 1;
            }
        } else {
            let (inc_a, inc_b) = (2 * dz, 2 * dz - 2 * dx);
            let mut e = 2 * dz - dx;
            let mut left = dx;
            while left >= 0 {
                put(x, z, &mut self.pixels);
                x += sx;
                if e <= 0 {
                    e += inc_a;
                } else {
                    z += sz;
                    e += inc_b;
                }
                left -= 1;
            }
        }
    }

    /// CHUDMapConstructor.cpp 0x0007E6E0 — empty pixels touching an edge pixel become 0x80.
    fn outline(&mut self) {
        let n = self.size as usize;
        if n < 5 {
            return;
        }
        for i in (2..=n - 3).rev() {
            for j in (2..=n - 3).rev() {
                let o = i * n + j;
                if self.pixels[o] != 0 {
                    continue;
                }
                let p = &self.pixels;
                let hit = [o - n, o - n - 1, o - n + 1, o + n, o + n - 1, o + n + 1, o - 1, o + 1].iter().any(|&k| p[k] == 255);
                if hit {
                    self.pixels[o] = 0x80;
                }
            }
        }
    }

    /// palette of COSDMapRender 0x000E85A0: hud map colour, alpha `max(index, min_alpha)`.
    pub fn to_rgba(&self, colour: [u8; 3], min_alpha: u8) -> Rgba {
        let n = self.size as u32;
        let mut data = Vec::with_capacity((n * n * 4) as usize);
        // stored [x * n + z]; the image is drawn with x across, z down
        for z in 0..n {
            for x in 0..n {
                let idx = self.pixels[(x * n + z) as usize];
                data.extend_from_slice(&[colour[0], colour[1], colour[2], idx.max(min_alpha)]);
            }
        }
        Rgba { width: n, height: n, data }
    }

    /// COSDMapRender.cpp 0x000E8D70 — world x/z to map uv (`(p - min) * 1/span`).
    pub fn uv(&self, p: [f32; 3]) -> [f32; 2] {
        let inv = 1.0 / (self.max - self.min);
        [(p[0] - self.min) * inv, (p[2] - self.min) * inv]
    }
}

/// what the map shows each frame.
#[derive(Clone, Debug, Default)]
pub struct MapInput {
    pub player: [f32; 3],
    pub heading: [f32; 3], // the car's forward vector
    pub rivals: Vec<[f32; 3]>,
    pub police: Vec<([f32; 3], bool)>, // position, lights on
}

/// COSDMapRender.cpp 0x000E8E70 — the heading angle: `acos(X . fwd)`, negated when `(X x fwd).y < 0` (fwd.z > 0), plus pi
/// (the binary takes the acos of the x component as is, the vector being a unit body row)
pub fn heading_angle(f: [f32; 3]) -> f32 {
    let mut a = f[0].clamp(-1.0, 1.0).acos();
    if f[2] > 0.0 {
        a = -a;
    }
    a + std::f32::consts::PI
}

/// the binary's texture matrix mirrors u (`Renderer 0x2AB620(-1,1,1)`), which turns its rotation by `a` into the
/// panel rotation (y down, uv = player + R(angle) * corner) that puts the car's forward vector (u = x, v = z) at the
/// top of the panel: `3pi/2 - a` (= `atan2(fwd.z, fwd.x) + pi/2`). Measured by `map_forward_is_up`.
pub fn panel_angle(a: f32) -> f32 {
    3.0 * std::f32::consts::FRAC_PI_2 - a
}

/// `COSDMapRender`
pub struct COSDMapRender {
    pub map: HudMap,
    pub texture: TextureId,
    pub grid: TextureId,
    pub player_blip: TextureId,
    pub rival_blip: TextureId,
    pub police_on: TextureId,
    pub police_off: TextureId,
    pub centre: [f32; 3], // hud map position (plane units)
    pub size: f32,
    pub flash: bool,      // +900
    pub flash_time: f32,  // +896
}

/// `dword_685FE0` window (world units across)
const WINDOW: f32 = 900.0;
/// `dword_685FE4` blip flash period
const FLASH: f32 = 0.25;

impl COSDMapRender {
    /// COSDMapRender.cpp 0x000E85A0 — `Init`: build the map, load the blips from `OSD\OSDT.vcd`.
    pub fn new(map: HudMap, colour: [u8; 3], min_alpha: u8, centre: [f32; 3], tex: &mut TextureStore) -> Self {
        tex.load_vcd("\\OSD\\OSDT.vcd");
        let find = |tex: &TextureStore, n: &str| tex.find(visos::handy::handy_crc::name_hash(n));
        let texture = tex.add(visos::handy::handy_crc::name_hash("osd_hudmap_runtime"), map.to_rgba(colour, min_alpha));
        Self {
            texture,
            grid: find(tex, "nav_grid"),
            player_blip: find(tex, "plyrblip"),
            rival_blip: find(tex, "rvalblip"),
            police_on: find(tex, "blueblip"),
            police_off: find(tex, "redblip"),
            map,
            centre,
            size: 8.0,
            flash: true,
            flash_time: FLASH,
        }
    }

    /// COSDMapRender.cpp 0x000E8E70 / 0x000E9670 — `Update` + `Render`, into the map screen texture (`out` covers the
    /// `hud_map_screen` model's texture square; the 3D hud places the model).
    pub fn render(&mut self, input: &MapInput, dt: f32, out: &mut DrawList) {
        let p = self.map.uv(input.player);
        // heading: acos(x . fwd), signed by the cross product's y, plus pi
        let a = panel_angle(heading_angle(input.heading));
        let window = WINDOW / (self.map.max - self.map.min);
        let (s, c) = a.sin_cos();
        // panel corners (-0.5..0.5, y down), uv = player + R * (corner * window); the panel is the whole screen texture
        let corners = [[-0.5f32, -0.5f32], [0.5, -0.5], [-0.5, 0.5], [0.5, 0.5]];
        let uv = corners.map(|k| [p[0] + (k[0] * c - k[1] * s) * window, p[1] + (k[0] * s + k[1] * c) * window]);
        let pos = corners.map(|k| [k[0], k[1]]);
        out.push(screen_quad(self.grid, pos, corners.map(|k| [k[0] + 0.5, k[1] + 0.5]), 0xFFFF_FFFF));
        out.push(screen_quad(self.texture, pos, uv, 0xFFFF_FFFF));
        // blips: map-relative position rotated back into the panel
        let blip = |tex: TextureId, w: [f32; 3], size: f32, out: &mut DrawList| {
            let q = self.map.uv(w);
            let (dx, dz) = ((q[0] - p[0]) / window, (q[1] - p[1]) / window);
            let (bx, by) = (dx * c + dz * s, -dx * s + dz * c);
            if bx.abs() > 0.5 || by.abs() > 0.5 {
                return;
            }
            // blip size in hud units over the panel's
            let r = size * 0.5 / self.size;
            out.push(screen_quad(tex, [[bx - r, by - r], [bx + r, by - r], [bx - r, by + r], [bx + r, by + r]], [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]], 0xFFFF_FFFF));
        };
        for r in &input.rivals {
            blip(self.rival_blip, *r, 0.6, out);
        }
        for (pp, lights) in &input.police {
            blip(if *lights { self.police_on } else { self.police_off }, *pp, 0.6, out);
        }
        blip(self.player_blip, input.player, 0.8, out);
        // +896 / +900: blip flash timer
        self.flash_time -= dt;
        if self.flash_time < 0.0 {
            self.flash = !self.flash;
            self.flash_time = FLASH;
        }
    }
}

/// panel quad (-0.5..0.5, y down) to the map screen texture (the 512 virtual square, rasterised to
/// `osd_3d_interface::MAP_SCREEN_SIZE` and drawn on `hud_map_screen`)
fn screen_quad(texture: TextureId, pos: [[f32; 2]; 4], uv: [[f32; 2]; 4], colour: Colour) -> Quad {
    let map = |p: [f32; 2]| [(p[0] + 0.5) * 512.0, (p[1] + 0.5) * 512.0];
    Quad { texture, pos: pos.map(map), uv, colour: [colour; 4] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_angle_values() {
        use std::f32::consts::PI;
        // values of 0xE8E70 for the cardinal body-row-0 directions
        assert!((heading_angle([1.0, 0.0, 0.0]) - PI).abs() < 1e-5);
        assert!((heading_angle([0.0, 0.0, 1.0]) - PI / 2.0).abs() < 1e-5);
        assert!((heading_angle([0.0, 0.0, -1.0]) - 1.5 * PI).abs() < 1e-5);
        assert!((heading_angle([-1.0, 0.0, 0.0]) - 2.0 * PI).abs() < 1e-5);
    }

    /// builds a straight road (300 behind, 2000 ahead of the player) in direction `d`, renders the map at that heading
    /// and measures the screen angle of the road ahead of the marker (centre of the panel): it must point up.
    #[test]
    fn map_forward_is_up() {
        let mut log = String::new();
        for deg in [0.0f32, 37.0, 90.0, 135.0, 180.0, 225.0, 270.0, 322.0] {
            let t = deg.to_radians();
            let d = [t.cos(), 0.0, t.sin()]; // world heading (x, z)
            let n = [-d[2], 0.0, d[0]];
            let c = [3000.0f32, 0.0, 3000.0];
            let edge = |s: f32| (0..=46).map(|i| {
                let l = -300.0 + i as f32 * 50.0;
                [c[0] + d[0] * l + n[0] * s, 0.0, c[2] + d[2] * l + n[2] * s]
            }).collect::<Vec<_>>();
            let (i, o) = (edge(-25.0), edge(25.0));
            let map = HudMap::construct(&HudMapDesc { inside: &i, outside: &o, size: 256, border: 4, closed: false });
            let mut tex = TextureStore::default();
            let clear = Rgba { width: 1, height: 1, data: vec![0, 0, 0, 0] };
            let (grid, blip) = (tex.add(1, clear.clone()), tex.add(2, clear));
            let rgba = map.to_rgba([0, 255, 0], 0);
            let texture = tex.add(3, rgba);
            let mut r = COSDMapRender { map, texture, grid, player_blip: blip, rival_blip: blip, police_on: blip, police_off: blip, centre: [0.0; 3], size: 8.0, flash: true, flash_time: 1.0 };
            let mut dl = DrawList::default();
            r.render(&MapInput { player: c, heading: d, ..Default::default() }, 0.0, &mut dl);
            let img = dl.rasterise_overlay(&tex, 256, 256);
            let (mut sx, mut sy, mut cnt) = (0.0f64, 0.0f64, 0.0f64);
            for y in 0..256 {
                for x in 0..256 {
                    let px = &img.data[((y * 256 + x) * 4) as usize..][..4];
                    if px[3] > 128 && px[1] > 128 {
                        sx += x as f64 - 127.5;
                        sy += y as f64 - 127.5;
                        cnt += 1.0;
                    }
                }
            }
            assert!(cnt > 200.0, "heading {deg}: no road drawn");
            // screen angle of the road ahead, from straight up, clockwise positive
            let ang = (sx / cnt).atan2(-sy / cnt).to_degrees();
            log.push_str(&format!("heading x,z = ({:.3}, {:.3}) -> a = {:.1} deg, panel {:.1} deg; road ahead at {:.1} deg from screen-up ({} px)\n", d[0], d[2], heading_angle(d).to_degrees(), panel_angle(heading_angle(d)).to_degrees(), ang, cnt));
            assert!(ang.abs() < 5.0, "heading {deg}: road ahead points {ang} deg off up");
        }
        let _ = std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/map_heading_check.log"), log);
    }

    #[test]
    fn construct_ring() {
        // a ring track: inside radius 80, outside 100
        let ring = |r: f32| (0..64).map(|i| { let a = i as f32 / 64.0 * std::f32::consts::TAU; [a.cos() * r, 0.0, a.sin() * r] }).collect::<Vec<_>>();
        let (i, o) = (ring(80.0), ring(100.0));
        let m = HudMap::construct(&HudMapDesc { inside: &i, outside: &o, size: 128, border: 4, closed: true });
        assert!(m.pixels.iter().filter(|&&p| p == 255).count() > 300);
        assert!(m.pixels.iter().any(|&p| p == 0x80));
        assert!(m.min < -100.0 && m.max > 100.0);
    }
}
