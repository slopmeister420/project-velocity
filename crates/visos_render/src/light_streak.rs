//! Render\LightStreak.cpp (0x0012E5C0..0x0012FA80; the idb labels the first functions LicencePlateRender) —
//! screen-space trails behind nearby lights: up to 16 slots, each a ring of 8 projected samples (one kept every
//! 0.1 s, the head follows the light every frame), drawn as a fading "spec2" ribbon. TrackFX owns one (width 192,
//! fed by its near flares, 0x0014A5B0); FlareRender owns the other (width 128, car lights, `flare_render`).

use bevy::math::{Vec2, Vec3, Vec4};

use crate::effect_draw::{EffectDraw, EffectShader, EffectTexture, EffectVertex};

pub const MAX_LIGHTSTREAK_LIGHTS: usize = 16;
pub const MAX_LIGHTSTREAK_SAMPLES: usize = 8;
/// `dword_686BFC`: ribbon half width in screen units at depth 1 (TrackFX, DangerZone)
pub const STREAK_WIDTH: [f32; 2] = [192.0, 128.0];
/// `dword_686C04`: least squared screen step between two drawn samples
pub const MIN_STEP_SQ: f32 = 0.0625;
/// `dword_686C08`: depth floor for the width
pub const MIN_DEPTH: f32 = 4.0;
/// `dword_686C0C`
pub const INTENSITY_SCALE: f32 = 1.0;
/// `dword_686C10` / `dword_686C14`: head-to-tail screen length where the ribbon starts / is fully on
pub const MIN_LENGTH: f32 = 8.0;
pub const FULL_LENGTH: f32 = 64.0;
/// `dword_686C18`: depth of a light that streaks at all (also the TrackFX near / far flare split)
pub const MAX_DEPTH: f32 = 150.0;
/// `dword_686C1C`: sample period
pub const SAMPLE_PERIOD: f32 = 0.1;
/// a new sample's alpha (0x3F333333)
pub const SAMPLE_ALPHA: f32 = f32::from_bits(0x3F33_3333);
/// screen space of the samples (projected 0..1 times 512)
pub const SCREEN: f32 = 512.0;

/// static init 0x004257E0 (`dword_71FF14` / `dword_71FF18`): (head, tail) colour per light colour index, as
/// VISOS_ColourXBOX (a, r, g, b) 0x80 = 1.0; index 4 reads past the table (zero here)
pub const STREAK_COLOURS: [([u8; 4], [u8; 4]); 5] = [
    ([0x80, 0x20, 0x18, 0x10], [0x00, 0x20, 0x08, 0x00]),
    ([0x80, 0x50, 0x50, 0xA0], [0x00, 0x00, 0x00, 0x80]),
    ([0x80, 0x40, 0x40, 0x80], [0x00, 0x00, 0x00, 0x60]),
    ([0x80, 0x40, 0x00, 0x00], [0x00, 0x20, 0x00, 0x00]),
    ([0; 4], [0; 4]),
];

/// VISOS_ColourXBOX 0x002B11C0 (a, r, g, b): doubled, clamped, D3DCOLOR
pub fn visos_colour(c: [u8; 4]) -> u32 {
    let d = |v: u8| (2 * v as u32).min(255);
    (d(c[0]) << 24) | (d(c[1]) << 16) | (d(c[2]) << 8) | d(c[3])
}

/// one light the owner wants streaked this frame (32-byte record of 0x0014A5B0)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StreakLight {
    /// +0 world position
    pub pos: Vec3,
    /// +12 view depth, +16 intensity
    pub depth: f32,
    pub intensity: f32,
    /// +20 light index
    pub light: i16,
    /// +28 colour (the light's flare id)
    pub colour: u8,
}

/// one slot, 120 bytes from manager +12
#[derive(Clone, Copy, Debug)]
pub struct LightStreakSlot {
    /// +0 samples: screen x, y (0..512), alpha
    pub samples: [[f32; 3]; MAX_LIGHTSTREAK_SAMPLES],
    /// +96 max(w, 4), +100 z / w of the newest projection
    pub w: f32,
    pub z: f32,
    /// +104 depth, +108 intensity (from the record)
    pub depth: f32,
    pub intensity: f32,
    /// +112 light, +114 head, +115 samples in use, +116 colour index
    pub light: i16,
    pub head: i8,
    pub count: u8,
    pub colour: u8,
    /// eye depth of the newest projection (the original draws at `z`; kept to place the ribbon in 3D)
    pub eye_depth: f32,
}

impl Default for LightStreakSlot {
    fn default() -> Self {
        Self {
            samples: [[0.0; 3]; MAX_LIGHTSTREAK_SAMPLES],
            w: 0.0,
            z: 0.0,
            depth: 0.0,
            intensity: 0.0,
            light: -1,
            head: -1,
            count: 0,
            colour: 0,
            eye_depth: 0.0,
        }
    }
}

impl LightStreakSlot {
    /// LightStreak.cpp 0x0012F070
    pub fn init(&mut self, light: i16, colour: u8) {
        self.light = light;
        self.colour = colour.min(4);
        self.head = -1;
        self.count = 0;
    }

    /// LightStreak.cpp 0x0012F0C0 — project the light (0x0015EC80: x, y in 0..1, z / w, w); behind the near plane (w < 1) fails
    pub fn sample(&mut self, p: Vec4) -> bool {
        if p.w < 1.0 {
            return false;
        }
        self.head += 1;
        if self.head >= 8 {
            self.head = 0;
        }
        if self.count < 8 {
            self.count += 1;
        }
        let h = self.head as usize;
        self.samples[h] = [p.x * SCREEN, p.y * SCREEN, SAMPLE_ALPHA];
        self.z = p.z;
        self.w = p.w.max(MIN_DEPTH);
        self.eye_depth = p.w;
        true
    }

    /// LightStreak.cpp 0x0012F250 — the ribbon as a strip in screen space (x, y, colour, uv); `None` under 4 vertices
    pub fn ribbon(&self, width: f32) -> Option<Vec<(Vec2, u32, Vec2)>> {
        let s = &self.samples;
        let mut head = self.head as i32;
        let mut next = head + 1;
        if next >= self.count as i32 {
            next = 0;
        }
        let dx = s[next as usize][0] - s[head as usize][0];
        let dy = s[next as usize][1] - s[head as usize][1];
        let len = (dx * dx + dy * dy).sqrt();
        if MIN_LENGTH > len {
            return None;
        }
        let mut intensity = (1.0 - self.depth / MAX_DEPTH) * INTENSITY_SCALE * self.intensity;
        if FULL_LENGTH > len {
            intensity *= (len - MIN_LENGTH) / (FULL_LENGTH - MIN_LENGTH);
        }
        let (head_col, tail_col) = STREAK_COLOURS[self.colour as usize];
        let (head_c, tail_c) = (visos_colour(head_col), visos_colour(tail_col));
        let w = width / self.w;
        let mut out: Vec<(Vec2, u32)> = Vec::new();
        let (mut px, mut py) = (0.0f32, 0.0f32);
        let mut prev = head;
        for _ in 0..self.count as i32 - 1 {
            let mut j = head - 1;
            if j < 0 {
                j += 8;
            }
            let ex = s[j as usize][0] - s[prev as usize][0];
            let ey = s[j as usize][1] - s[prev as usize][1];
            let mut d = ex * ex + ey * ey;
            if d >= MIN_STEP_SQ {
                d = d.sqrt();
                px = ey * (w / d);
                py = (0.0 - ex) * (w / d);
                let h = s[head as usize];
                if out.is_empty() {
                    // cap: the tail colour at full ribbon alpha, pushed forwards by the half width
                    let c = (tail_c & 0x00FF_FFFF) | ((ftol(intensity * 255.0) as u32 & 0xFF) << 24);
                    let mut v = Vec2::new(h[0] + px + py, h[1] + py - px);
                    out.push((v, c));
                    v -= Vec2::new(px * 2.0, py * 2.0);
                    out.push((v, c));
                }
                let mut v = Vec2::new(h[0] + px, h[1] + py);
                let a = s[j as usize][2] / 0.7;
                let k5 = ((a * a) * (a * a) * a * 128.0) as i32;
                let k1 = (a * 128.0 - k5 as f32) as i32;
                let mix = |sh: u32| (((k1 * ((tail_c >> sh) & 0xFF) as i32 + k5 * ((head_c >> sh) & 0xFF) as i32) >> 7) as u32 & 0xFF) << sh;
                let c = mix(16) | mix(8) | mix(0) | ((ftol(a * intensity * 255.0) as u32 & 0xFF) << 24);
                out.push((v, c));
                v -= Vec2::new(px * 2.0, py * 2.0);
                out.push((v, c));
            }
            prev = head;
            head = j;
        }
        let h = s[head as usize];
        let mut v = Vec2::new(h[0] + px, h[1] + py);
        out.push((v, 0));
        v -= Vec2::new(px * 2.0, py * 2.0);
        out.push((v, 0));
        if out.len() < 4 {
            return None;
        }
        // uvs of the vertex template (0x0012EDC0): u 0 on the first pair, 0.5 after, v 0 / 1 across
        Some(
            out.into_iter()
                .enumerate()
                .map(|(i, (p, c))| (p, c, Vec2::new(if i < 2 { 0.0 } else { 0.5 }, (i & 1) as f32)))
                .collect(),
        )
    }
}

/// `sub_3DDFA8` (_ftol): truncation
fn ftol(x: f32) -> i32 {
    x as i32
}

#[derive(Clone, Debug)]
pub struct LightStreakManager {
    /// +0 the farthest light that streaks (reset to `MAX_DEPTH` every update; TrackFX lowers it while its record
    /// list is full)
    pub max_depth: f32,
    /// +4 slots in use, +8 slots sampled this update
    pub used: u32,
    pub updated: u32,
    pub slots: [LightStreakSlot; MAX_LIGHTSTREAK_LIGHTS],
    /// +1944 width, +1948 sample timer
    pub width: f32,
    pub timer: f32,
}

impl LightStreakManager {
    /// LightStreak.cpp 0x0012E5C0 (the "spec2" shader is `shader()`); `track_fx` = the TrackFX instance
    pub fn new(track_fx: bool) -> Self {
        let mut m = Self {
            max_depth: MAX_DEPTH,
            used: 0,
            updated: 0,
            slots: [LightStreakSlot::default(); MAX_LIGHTSTREAK_LIGHTS],
            width: STREAK_WIDTH[(!track_fx) as usize],
            timer: 0.0,
        };
        m.reset();
        m
    }

    /// LightStreak.cpp 0x0012E770
    pub fn reset(&mut self) {
        for s in &mut self.slots {
            s.init(-1, 0);
        }
        self.used = 0;
        self.updated = 0;
        self.max_depth = MAX_DEPTH;
        self.timer = 0.0;
    }

    /// shader +1932: "spec2" (misc.vcd), blend 7, z test, no z write
    pub fn shader() -> EffectShader {
        EffectShader::new(EffectTexture::Misc("spec2".into()), 7).z(true, false)
    }

    /// LightStreak.cpp 0x0012E800 — `lights` = this frame's records, `slot_of[light]` = the light's slot byte (light info +30);
    /// `project` = 0x0015EC80. Draws (0x0012EDC0) through `draw` before the samples age.
    pub fn update(
        &mut self,
        lights: &[StreakLight],
        slot_of: &mut [i8],
        dt: f32,
        project: impl Fn(Vec3) -> Vec4,
        draw: &mut impl FnMut(&LightStreakSlot, f32),
    ) {
        self.max_depth = MAX_DEPTH;
        let mut pending: u32 = (1u32 << lights.len()) - 1;
        self.updated = 0;
        for (i, l) in lights.iter().enumerate() {
            let slot = slot_of.get(l.light as usize).copied().unwrap_or(-1);
            if slot == -1 {
                continue;
            }
            let n = slot as usize;
            if self.slots[n].light == l.light {
                if self.slots[n].sample(project(l.pos)) {
                    self.slots[n].depth = l.depth;
                    self.slots[n].intensity = l.intensity;
                    self.updated |= 1 << n;
                }
                pending &= !(1 << i);
            }
        }
        let mut from = 0;
        if pending != 0 {
            for (i, l) in lights.iter().enumerate() {
                if pending & (1 << i) == 0 {
                    continue;
                }
                let Some(n) = (from..MAX_LIGHTSTREAK_LIGHTS).find(|&n| self.used & (1 << n) == 0) else { continue };
                from = n;
                pending &= !(1 << i);
                let s = &mut self.slots[n];
                s.init(-1, l.colour);
                if s.sample(project(l.pos)) {
                    if let Some(b) = slot_of.get_mut(l.light as usize) {
                        *b = n as i8;
                    }
                    s.light = l.light;
                    s.depth = l.depth;
                    s.intensity = l.intensity;
                    self.used |= 1 << n;
                    self.updated |= 1 << n;
                }
            }
        }
        // 0x0012EDC0
        for (n, s) in self.slots.iter().enumerate() {
            if self.used & (1 << n) != 0 && s.count > 1 {
                draw(s, self.width);
            }
        }
        self.timer += dt;
        let tick = self.timer > SAMPLE_PERIOD;
        if tick {
            self.timer = 0.0;
        }
        for n in 0..MAX_LIGHTSTREAK_LIGHTS {
            let s = &mut self.slots[n];
            if tick {
                // lights not seen this update shrink, then free their slot
                if self.used & (1 << n) != 0 && self.updated & (1 << n) == 0 {
                    if s.count != 0 {
                        s.count -= 1;
                    } else {
                        s.count = 0;
                        s.light = -1;
                        self.used &= !(1 << n);
                    }
                }
            } else if self.updated & (1 << n) != 0 {
                // between ticks the new head replaces the previous one
                s.head -= 1;
                if s.head < 0 {
                    s.head = 7;
                }
                if s.count != 0 {
                    s.count -= 1;
                }
            }
        }
        for n in 0..MAX_LIGHTSTREAK_LIGHTS {
            if self.used & (1 << n) == 0 {
                continue;
            }
            let s = &mut self.slots[n];
            let mut k = s.head as i32;
            for _ in 0..s.count {
                let a = &mut s.samples[k as usize][2];
                *a -= dt;
                if *a < 0.0 {
                    *a = 0.0;
                }
                k -= 1;
                if k < 0 {
                    k += 8;
                }
            }
        }
    }
}

/// a screen point (0..512) at eye depth `d` back to world space: camera position, axes, tan of the half fovs
pub fn unproject(p: Vec2, d: f32, cam: Vec3, right: Vec3, up: Vec3, fwd: Vec3, tan_x: f32, tan_y: f32) -> Vec3 {
    let x = p.x / SCREEN * 2.0 - 1.0;
    let y = 1.0 - p.y / SCREEN * 2.0;
    cam + (fwd + right * (x * tan_x) + up * (y * tan_y)) * d
}

/// one ribbon into the effect batches, placed at the slot's eye depth
#[allow(clippy::too_many_arguments)]
pub fn draw_slot(s: &LightStreakSlot, width: f32, cam: Vec3, right: Vec3, up: Vec3, fwd: Vec3, tan_x: f32, tan_y: f32, out: &mut EffectDraw) {
    let Some(r) = s.ribbon(width) else { return };
    let v: Vec<EffectVertex> = r
        .into_iter()
        .map(|(p, c, uv)| EffectVertex::new(unproject(p, s.eye_depth, cam, right, up, fwd, tan_x, tan_y), c, uv))
        .collect();
    out.strip(&LightStreakManager::shader(), &v);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// a light sweeping across the screen grows a ribbon after a few sample periods and frees its slot once gone
    #[test]
    fn slot_lifecycle() {
        let mut m = LightStreakManager::new(true);
        let mut slot_of = vec![-1i8; 4];
        let l = StreakLight { pos: Vec3::ZERO, depth: 20.0, intensity: 1.0, light: 2, colour: 0 };
        let mut drawn = 0;
        for f in 0..60 {
            let x = 0.2 + f as f32 * 0.01;
            m.update(&[l], &mut slot_of, 1.0 / 30.0, |_| Vec4::new(x, 0.5, 0.5, 10.0), &mut |s, w| {
                if s.ribbon(w).is_some() {
                    drawn += 1;
                }
            });
        }
        assert_eq!(slot_of[2], 0);
        assert!(m.used & 1 != 0 && drawn > 0);
        for _ in 0..60 {
            m.update(&[], &mut slot_of, 1.0 / 30.0, |_| Vec4::ZERO, &mut |_, _| {});
        }
        assert_eq!(m.used, 0);
    }
}
