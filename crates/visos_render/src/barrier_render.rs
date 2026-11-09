//! src\Render\BarrierRender.cpp — the drawn spline barriers (object `dword_705F48`): arrow barriers (AI barrier kind
//! 0) as scrolling "BarrierArrows" strips 4 m high that wobble red from the point the player hit them, no-entry
//! barriers (kind 1) as "UndergroundDash" strips with a `barrierStopMainOn` sign in the middle. The collision side
//! (0x00108030) lives in `velocity_ai::barrier_render`.
//!
//! API: [`LoadBarrierRender`] at race load (GameManager::InGameInit -> 0x00107560) with the AI world's barrier
//! lines, [`UnloadBarrierRender`] at shutdown (0x00107CE0), [`BarrierHit`] per collision message (CarPalette
//! 0x0010D5F0 -> 0x0010B590). Drawn every frame (GameManager 0x000B9010 -> 0x00109080) through `EffectDraw`.

use bevy::camera::primitives::{Frustum, Sphere};
use bevy::prelude::*;

use crate::effect_draw::{EffectClock, EffectDraw, EffectShader, EffectTexture, EffectVertex};
use crate::track_render::TrackCamera;

/// `dword_68656C`: barrier height
pub const BARRIER_HEIGHT: f32 = 4.0;
/// `dword_624A84` (2.0) x `dword_68656C`: metres per texture repeat; `dword_705F84` = 1 / this
const U_PER_METRE: f32 = 1.0 / (BARRIER_HEIGHT * 2.0);
/// `dword_6865B8`: the texture length is rounded to thirds
const U_ROUND: i32 = 3;
/// `flt_6865BC`: the end caps sit this far in from the ends
const CAP_LENGTH: f32 = 3.0;
/// `dword_686570` / `dword_686574`: squared distances where the fade out starts / the barrier is gone
const FADE_START: f32 = 40000.0;
const FADE_END: f32 = 62500.0;
/// `dword_686598` / `dword_68659C`: closer than this the alpha drops towards `NEAR_ALPHA`
const NEAR_DIST2: f32 = 2500.0;
const NEAR_ALPHA: f32 = 92.0;
/// `dword_686594`: full alpha (VISOS 0x80 = 1.0)
const FULL_ALPHA: f32 = 128.0;
/// `dword_686578`: texture repeats per batch, in 4096ths
const BATCH_REPEATS: i32 = 7;
/// `dword_68657C`: no-entry strips repeat the texture twice as often
const DASH_U_SCALE: f32 = 2.0;
/// `dword_686580` / `dword_686584`: no-entry strip top / bottom height
const DASH_TOP: f32 = 4.0;
const DASH_BOTTOM: f32 = 0.5;
/// `dword_686588` / `dword_68658C`: the arrows scroll in sixths, by this per millisecond
const SCROLL_STEPS: i32 = 6;
const SCROLL_RATE: f32 = f32::from_bits(0xBA2E_C33E);
/// `byte_686590`: strip grey (VISOS 0x80 = white)
const GREY: u8 = 0x80;
/// `dword_6865C0`: half the gap the stop sign leaves in a no-entry barrier
const SIGN_GAP: f32 = 2.5;
/// `dword_6865C4`: wobble strips are cut every 1.5 m (1..24 pieces per segment, Speedo 0x000F7A20)
const WOBBLE_STEP: f32 = 1.5;
/// `MAX_VERTS_PER_BATCH`
const MAX_VERTS_PER_BATCH: usize = 50;

/// the "Graphics.Spline Barriers" tweaks (0x00107560, defaults in the data segment)
#[derive(Resource, Clone, Debug)]
pub struct BarrierTweaks {
    /// `dword_6865A0`
    pub frequency: f32,
    /// `dword_6865A4`: how far the wobble turns the strip red
    pub colour: f32,
    /// `flt_6865A8`
    pub amplitude: f32,
    /// `dword_6865AC`
    pub speed: f32,
    /// `dword_6865B0`: in wobble cycles
    pub duration: f32,
    /// `dword_6865B4`: u shift at full wobble, in 4096ths
    pub uv_wobble: f32,
    /// `byte_705F88` red / green / blue (static init 0x004244B0)
    pub red: Vec3,
}

impl Default for BarrierTweaks {
    fn default() -> Self {
        Self { frequency: 6.0, colour: 1.0, amplitude: 2.0, speed: f32::from_bits(0x3E99_999A), duration: 1.0, uv_wobble: 256.0, red: Vec3::new(48.0, 0.0, 0.0) }
    }
}

/// the AI world's barrier lines (`VsAIWorld::GetBarrier(kind, i)` points, xyz): kind 0 arrows, kind 1 no-entry
#[derive(Message, Clone, Debug, Default)]
pub struct LoadBarrierRender {
    pub arrows: Vec<Vec<Vec3>>,
    pub no_entry: Vec<Vec<Vec3>>,
    /// 0x00107560 argument (flag bit 1): draw at all
    pub render: bool,
    /// the loaded `armco01.ar` lines (professional races, Armco.cpp 0x00011090); drawn whatever `render` says
    pub armco: Vec<ArmcoLineDraw>,
}

/// one armco line: kind 2 / 3 get a 1.2 m rail strip (winding +1 / -1), every point with w == 1 a post
#[derive(Clone, Debug, Default)]
pub struct ArmcoLineDraw {
    pub kind: u16,
    pub closed: bool,
    /// xyz + w (1 = a post stands here)
    pub points: Vec<Vec4>,
}

/// Armco.cpp 0x00011C50: rail height, metres per texture repeat, `Armco05`, light directions / colours / ambient
const ARMCO_HEIGHT: f32 = 1.2;
const ARMCO_REPEAT: f32 = 32.0;
const ARMCO_LIGHT_DIRS: [[f32; 3]; 3] = [[0.0, 66.0, -115.0], [100.0, 66.0, 57.0], [-100.0, 66.0, 57.0]];
const ARMCO_LIGHT_COLOURS: [f32; 3] = [0.7, 0.3, 0.2];
const ARMCO_AMBIENT: f32 = 0.1;
/// segments per drawn chunk (the original culls per primitive chunk by a sphere)
const ARMCO_CHUNK: usize = 8;

/// a culled piece of an armco rail strip
pub struct ArmcoChunk {
    centre: Vec3,
    radius: f32,
    verts: Vec<EffectVertex>,
}

/// CStripFromSplines 0x000810F0 + 0x00081C80 + 0x00081780 + 0x00081940: the rail as a triangle strip of
/// (base, top) pairs (kind 2) or (top, base) (kind 3), v along the line in whole 32 m repeats, vertices lit by
/// the three armco lights plus ambient
pub fn armco_strip(l: &ArmcoLineDraw) -> Vec<EffectVertex> {
    if l.points.len() < 2 || !(l.kind == 2 || l.kind == 3) {
        return Vec::new();
    }
    let mut base: Vec<Vec3> = l.points.iter().map(|p| p.truncate()).collect();
    if l.closed {
        base.push(base[0]);
    }
    let up = Vec3::new(0.0, ARMCO_HEIGHT, 0.0);
    let mut pos = Vec::with_capacity(base.len() * 2);
    for b in &base {
        if l.kind == 2 {
            pos.extend([*b, *b + up]);
        } else {
            pos.extend([*b + up, *b]);
        }
    }
    let n = pos.len();
    // 0x00081C80: side value across the rail (kind 3 flips it, but not for the first pair), distance along it
    let mut uv = vec![Vec2::ZERO; n];
    for k in 0..2usize {
        uv[k] = Vec2::new(k as f32, 0.0);
        let mut acc = 0.0f32;
        let mut j = k;
        while j + 2 < n {
            acc += pos[j].distance(pos[j + 2]);
            uv[j + 2] = Vec2::new(if l.kind == 3 { (1 - k) as f32 } else { k as f32 }, acc);
            j += 2;
        }
        let repeats = (acc / ARMCO_REPEAT + 0.5) as i32 as f32;
        let ratio = repeats / acc;
        let mut j = k;
        while j + 2 < n {
            uv[j + 2].y *= ratio;
            j += 2;
        }
    }
    // 0x00081780: face normals added to their three vertices, each renormalised at once
    let mut nrm = vec![Vec3::ZERO; n];
    for i in 0..n.saturating_sub(2) {
        let e1 = pos[i + 1] - pos[i];
        let e2 = pos[i + 2] - pos[i];
        let f = if i % 2 == 0 { e1.cross(e2) } else { e2.cross(e1) }.normalize_or_zero();
        for j in i..i + 3 {
            nrm[j] = (nrm[j] + f).normalize_or_zero();
        }
    }
    let dirs = ARMCO_LIGHT_DIRS.map(|d| Vec3::from(d).normalize());
    (0..n)
        .map(|i| {
            let mut c = Vec3::splat(ARMCO_AMBIENT);
            for (d, col) in dirs.iter().zip(ARMCO_LIGHT_COLOURS) {
                let dot = nrm[i].dot(*d);
                if dot > 0.0 {
                    c += Vec3::splat(col * dot);
                }
            }
            let c = c.min(Vec3::ONE);
            let b = |x: f32| (x * 128.0).min(255.0) as u32;
            EffectVertex::new(pos[i], visos_argb(b(1.0), b(c.x), b(c.y), b(c.z)), uv[i])
        })
        .collect()
}

/// the strip cut into `ARMCO_CHUNK`-segment pieces (even vertex starts keep the strip parity) with bounding spheres
fn armco_chunks(strip: &[EffectVertex]) -> Vec<ArmcoChunk> {
    let mut out = Vec::new();
    let mut s = 0;
    while s + 2 < strip.len() {
        let e = (s + 2 * ARMCO_CHUNK + 2).min(strip.len());
        let v = strip[s..e].to_vec();
        let (lo, hi) = v.iter().fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(lo, hi), x| {
            let p = x.pos;
            (lo.min(p), hi.max(p))
        });
        let centre = (lo + hi) * 0.5;
        out.push(ArmcoChunk { centre, radius: (hi - centre).length(), verts: v });
        s += 2 * ARMCO_CHUNK;
    }
    out
}

#[derive(Message, Clone, Copy, Debug, Default)]
pub struct UnloadBarrierRender;

/// a collision message for 0x0010B590: car a (+0), material (+76), world point (+8). Core sends it only while
/// `byte_6F70E9 == 0`.
#[derive(Message, Clone, Copy, Debug)]
pub struct BarrierHit {
    pub car_a: i32,
    pub material: i8,
    pub point: Vec3,
}

/// one barrier's render data (72 B records at +12, filled by 0x00106B30 / 0x00107100)
#[derive(Clone, Debug, Default)]
pub struct BarrierRenderData {
    /// +0 bounding sphere
    pub centre: Vec3,
    pub radius: f32,
    /// +16 the road line
    pub points: Vec<Vec3>,
    /// +20 texture u per point in 4096ths
    pub u: Vec<i32>,
    /// +24 squared distance to the nearer end this frame
    pub dist2: f32,
    /// +28 barrier kind
    pub kind: u16,
    /// +30 / +31 start / end cap used
    pub start_cap: bool,
    pub end_cap: bool,
    /// +32 length
    pub length: f32,
    /// +36 where the player hit it (0..1), < 0 none
    pub hit: f32,
    /// +40 / +56 the cap points, w = their u
    pub cap0: Vec4,
    pub cap1: Vec4,
}

impl BarrierRenderData {
    /// BarrierRender.cpp 0x00106B30 — bounds and texture u along the line
    pub fn new(points: Vec<Vec3>, kind: u16) -> Option<Self> {
        let n = points.len();
        if n < 2 {
            return None;
        }
        let (mut lo, mut hi) = (Vec3::splat(999999.0), Vec3::splat(0.0 - 999999.0));
        for p in &points {
            lo = lo.min(*p);
            hi = hi.max(*p);
        }
        let centre = (lo + hi) / 2.0;
        let radius = (hi - centre).length();
        let mut u = vec![0i32; n];
        let mut length = 0.0f32;
        let mut acc = 0.0f32;
        for i in 1..n {
            let seg = (points[i - 1] - points[i]).length();
            length += seg;
            acc += seg * U_PER_METRE;
            u[i] = (acc * 4096.0) as i32;
        }
        // CStripFromSplines 0x000825C0: whole thirds of a repeat (truncated)
        let whole = (U_ROUND as f32 * acc) as i32 as f32;
        let ratio = whole / (U_ROUND as f32 * acc);
        for v in u.iter_mut().skip(1) {
            *v = (*v as f32 * ratio) as i32;
        }
        Some(Self { centre, radius, points, u, dist2: 0.0, kind, length, hit: -1.0, ..default() })
    }

    /// BarrierRender.cpp 0x00107100 — the end caps of an arrow barrier, `CAP_LENGTH` in from each end
    pub fn init_caps(&mut self) {
        let n = self.points.len();
        let scale = (self.u[n - 1] - self.u[0]) as f32 / self.length;
        let d = self.points[1] - self.points[0];
        let len = d.length();
        self.start_cap = len > CAP_LENGTH;
        if self.start_cap {
            let p = self.points[0] + d * CAP_LENGTH / len;
            self.cap0 = p.extend(((scale * CAP_LENGTH) as i32 + self.u[0]) as f32);
        }
        let d = self.points[n - 2] - self.points[n - 1];
        let len = d.length();
        self.end_cap = len > CAP_LENGTH;
        if self.end_cap {
            let p = self.points[n - 1] + d * CAP_LENGTH / len;
            self.cap1 = p.extend((self.u[n - 1] - (scale * CAP_LENGTH) as i32) as f32);
        }
    }
}

/// the BarrierRender object `dword_705F48`
#[derive(Resource, Default)]
pub struct BarrierRender {
    /// flag bit 0 (initialised) and bit 1 (render)
    pub initialised: bool,
    pub render: bool,
    pub data: Vec<BarrierRenderData>,
    /// +44 scroll clock, +48 its quantised value
    pub scroll_clock: f32,
    pub scroll: f32,
    /// +52 the nearest drawn barrier (index into `data`)
    pub current: Option<usize>,
    /// +56 wobble clock, < 0 = no wobble running
    pub wobble: f32,
    /// the no-entry signs (`barrierStopMainOn` instances) per barrier
    signs: Vec<(usize, Entity)>,
    root: Option<Entity>,
    armco: Vec<ArmcoChunk>,
}

/// VISOS_ColourXBOX.cpp 0x002B11C0 to D3DCOLOR (each byte doubled, saturated)
fn visos_argb(a: u32, r: u32, g: u32, b: u32) -> u32 {
    let d = |x: u32| (2 * (x & 0xFF)).min(255);
    d(a) << 24 | d(r) << 16 | d(g) << 8 | d(b)
}

/// BarrierRender.cpp 0x00109C70 / 0x0010A9B0 — one strip vertex: u in 4096ths (u16, read back signed), top = v 1
fn strip_vertex(p: Vec3, u: i32, top: bool, bottom_y: f32, top_y: f32, argb: u32) -> EffectVertex {
    let y = p.y + if top { top_y } else { bottom_y };
    EffectVertex::new(Vec3::new(p.x, y, p.z), argb, Vec2::new(u as u16 as i16 as f32 / 4096.0, if top { 1.0 } else { 0.0 }))
}

/// first point index past `BATCH_REPEATS` texture repeats from `k` (inner loop of 0x00109080 / 0x0010A610)
fn batch_points(u: &[i32], k: usize, max: usize) -> (usize, i32) {
    let mut n = max;
    let base = u[k];
    let vbase = (BATCH_REPEATS << 12) + base - base % 4096;
    for i in 1..=n {
        if u[i + k] - vbase > BATCH_REPEATS << 12 {
            n = if i <= 1 { i } else { i - 1 };
            break;
        }
    }
    (n, vbase)
}

impl BarrierRender {
    /// BarrierRender.cpp 0x00107560 (render data 0x00107DA0, caps 0x00107100 for the arrow barriers)
    pub fn init(&mut self, load: &LoadBarrierRender) {
        self.render = load.render;
        self.data.clear();
        for (kind, lines) in [(0u16, &load.arrows), (1, &load.no_entry)] {
            for l in lines {
                if let Some(d) = BarrierRenderData::new(l.clone(), kind) {
                    self.data.push(d);
                }
            }
        }
        for d in self.data.iter_mut().filter(|d| d.kind == 0) {
            d.init_caps();
        }
        self.armco = load.armco.iter().flat_map(|l| armco_chunks(&armco_strip(l))).collect();
        self.initialised = true;
        self.scroll_clock = 0.0;
        self.scroll = 0.0;
        self.wobble = -1.0;
        self.current = None;
    }

    /// BarrierRender.cpp 0x0010B590 — the player's car into a barrier wall (material 21) starts a wobble from the
    /// hit point along the current barrier, unless one is running
    pub fn hit(&mut self, h: &BarrierHit) {
        if h.car_a != 0 || h.material != 21 {
            return;
        }
        let Some(c) = self.current else { return };
        let wobble = self.wobble;
        let Some(d) = self.data.get_mut(c) else { return };
        if d.kind != 0 || !(wobble < 0.0 || d.hit < 0.0) || d.points.len() <= 1 {
            return;
        }
        let p0 = d.points[0];
        let mut dir = d.points[d.points.len() - 1] - p0;
        let inv = 1.0 / dir.length();
        dir *= inv;
        d.hit = (dir.dot(h.point - p0) * inv).clamp(0.0, 1.0);
        self.wobble = 0.0;
        log::debug!("barrier render: barrier {c} hit at {}", d.hit);
    }

    /// BarrierRender.cpp 0x00109080 clocks (skipped while paused): `dt_ms` = frame time in milliseconds
    pub fn advance(&mut self, dt_ms: f32, tweaks: &BarrierTweaks) {
        if self.wobble >= 0.0 {
            self.wobble += tweaks.speed * dt_ms * 0.025;
        }
        if SCROLL_RATE != 0.0 {
            self.scroll_clock += dt_ms * SCROLL_RATE;
            let k = (SCROLL_STEPS as f32 * self.scroll_clock) as i32 % SCROLL_STEPS;
            self.scroll = k as f32 / SCROLL_STEPS as f32;
        }
    }

    /// BarrierRender.cpp 0x00109080 — visible barriers within 250 m (0x00108E60), far to near, faded by distance.
    /// Returns the drawn barriers' indices (no-entry signs to show).
    pub fn draw(&mut self, cam: Vec3, visible: impl Fn(Vec3, f32) -> bool, tweaks: &BarrierTweaks, draw: &mut EffectDraw) -> Vec<usize> {
        let mut shown = Vec::new();
        if !self.initialised || !self.render {
            return shown;
        }
        // 0x00108E60: in-view barriers, squared distance to the nearer end, sorted (0x0010B840) far first
        let mut list: Vec<usize> = (0..self.data.len()).filter(|&i| visible(self.data[i].centre, self.data[i].radius)).collect();
        for &i in &list {
            let d = &mut self.data[i];
            let a = (cam - d.points[0]).length_squared();
            let b = (cam - d.points[d.points.len() - 1]).length_squared();
            d.dist2 = if b <= a { b } else { a };
        }
        list.sort_by(|&a, &b| self.data[b].dist2.total_cmp(&self.data[a].dist2));
        self.current = list.last().copied();
        let arrows = EffectShader::new(EffectTexture::Misc("BarrierArrows".into()), 2).z(true, false).bias(100);
        let dash = EffectShader::new(EffectTexture::Misc("UndergroundDash".into()), 2).z(true, false).bias(100);
        for &i in &list {
            let d = &self.data[i];
            if d.points.len() < 2 || d.dist2 >= FADE_END {
                continue;
            }
            let mut a = FULL_ALPHA as i32;
            if d.dist2 <= FADE_START {
                if NEAR_DIST2 > d.dist2 {
                    a = ((FULL_ALPHA - NEAR_ALPHA) * (d.dist2 / NEAR_DIST2) + NEAR_ALPHA) as i32;
                }
            } else {
                let f = (d.dist2 - FADE_START) * (1.0 / (FADE_END - FADE_START));
                a = ((1.0 - f) * FULL_ALPHA) as i32;
            }
            if d.kind == 1 {
                self.draw_no_entry(i, a, &dash, draw);
                shown.push(i);
            } else if d.hit < 0.0 {
                self.draw_arrows(i, a, &arrows, draw);
            } else {
                self.draw_wobble(i, a as f32, tweaks, &arrows, draw);
            }
        }
        shown
    }

    /// Armco.cpp 0x00012160: the rail strips (ArmcoShader: blend 3, wrap, z test + write) whose chunks are in view
    pub fn draw_armco(&self, visible: &impl Fn(Vec3, f32) -> bool, draw: &mut EffectDraw) {
        if self.armco.is_empty() {
            return;
        }
        let sh = EffectShader::new(EffectTexture::Scenery("Armco05".into()), 3);
        for c in &self.armco {
            if visible(c.centre, c.radius) {
                draw.strip(&sh, &c.verts);
            }
        }
    }

    /// the arrow strip of 0x00109080: batches of at most 43 segments / 7 texture repeats, end points transparent
    fn draw_arrows(&self, i: usize, a: i32, sh: &EffectShader, draw: &mut EffectDraw) {
        let d = &self.data[i];
        let np = d.points.len();
        let c = visos_argb(a as u32, GREY as u32, GREY as u32, GREY as u32);
        let c0 = visos_argb(0, GREY as u32, GREY as u32, GREY as u32);
        let mut remaining = np - 1;
        let mut k = 0usize;
        let mut first = true;
        let mut top = false;
        let v = |p: Vec3, u: i32, top: bool, c: u32| strip_vertex(p, u, top, 0.0, BARRIER_HEIGHT, c);
        while remaining != 0 {
            let (n, mut vbase) = batch_points(&d.u, k, remaining.min(43));
            let count = n + 3;
            vbase -= (4096.0 * self.scroll) as i32;
            let mut verts = Vec::with_capacity(MAX_VERTS_PER_BATCH);
            for j in 0..count {
                if j == 2 && d.start_cap && first {
                    let u = d.cap0.w as i32 - (vbase as u16 as i32);
                    verts.push(v(d.cap0.truncate(), u, top, c));
                    verts.push(v(d.cap0.truncate(), u, !top, c));
                }
                if j == count - 2 && d.end_cap && n == remaining {
                    let u = d.cap1.w as i32 - (vbase as u16 as i32);
                    verts.push(v(d.cap1.truncate(), u, top, c));
                    verts.push(v(d.cap1.truncate(), u, !top, c));
                }
                let u = d.u[k] - vbase;
                let col = if k != 0 && k != np - 1 { c } else { c0 };
                verts.push(v(d.points[k], u, top, col));
                top = !top;
                if j > 0 && j < count - 2 {
                    k += 1;
                }
            }
            remaining -= n;
            draw.strip(sh, &verts);
            first = false;
        }
    }

    /// BarrierRender.cpp 0x0010AAC0 — the hit barrier: cut into 1.5 m pieces, a decaying sine travels out from the
    /// hit point turning the strip red and shifting its u; when it has passed everywhere the hit is cleared
    fn draw_wobble(&mut self, i: usize, a4: f32, tw: &BarrierTweaks, sh: &EffectShader, draw: &mut EffectDraw) {
        let (scroll, timer) = (self.scroll, self.wobble);
        let d = &self.data[i];
        let np = d.points.len();
        let scale = d.length / (d.u[np - 1] - d.u[0]) as f32;
        let inv_len = 1.0 / d.length;
        let mut dist_acc = 0.0f32;
        let hit = d.hit;
        let dur = tw.duration * 6.283_185_5;
        let mut verts: Vec<EffectVertex> = Vec::with_capacity(MAX_VERTS_PER_BATCH);
        let mut start = 0usize;
        let mut active = false;
        let scroll_u = (4096.0 * scroll) as i32 - (BATCH_REPEATS << 12);
        for s in 0..np - 1 {
            let (p0, p1) = (d.points[s], d.points[s + 1]);
            let (u0, u1) = (d.u[s], d.u[s + 1]);
            let seg = (u1 - u0) as f32 * scale;
            let steps = (((WOBBLE_STEP * 0.5 + seg) / WOBBLE_STEP) as i32).clamp(1, 24) as usize;
            let count = steps + 1;
            if verts.len() + 2 * count > MAX_VERTS_PER_BATCH {
                draw.strip(sh, &verts);
                verts.clear();
                start = 0;
            }
            let dir = p1 - p0;
            for j in start..count {
                let f = j as f32 / steps as f32;
                let p = p0 + dir * f;
                let mut u = u0 + ((u1 - u0) as f32 * f) as i32;
                u += scroll_u;
                let dist = (j as f32 / (count - 1) as f32) * seg + dist_acc;
                let x = (dist * inv_len - hit).abs();
                let phase = timer - (x + x) * tw.frequency;
                let mut w = 0.0f32;
                if phase > 0.0 && dur > phase {
                    let fade = 1.0 - phase / dur;
                    w = (phase.sin() * fade * tw.amplitude).clamp(-1.0, 1.0);
                    active = true;
                }
                let (mut r, mut g, mut b) = (GREY as u32, GREY as u32, GREY as u32);
                let mut uu = u as u16;
                if tw.uv_wobble != 0.0 {
                    uu = uu.wrapping_add((tw.uv_wobble * w) as i32 as u16);
                }
                if tw.colour != 0.0 {
                    let k = (w * tw.colour).abs();
                    let base = (GREY as f32 * (1.0 - k)) as i32 as u8;
                    r = (tw.red.x * k + base as f32) as i32 as u8 as u32;
                    g = (tw.red.y * k + base as f32) as i32 as u8 as u32;
                    b = (tw.red.z * k + base as f32) as i32 as u8 as u32;
                }
                let mut alpha = a4 as i32 as u8 as u32;
                if s == 0 {
                    if j == 0 {
                        alpha = 0;
                    } else if j == 1 {
                        alpha = (a4 * 0.5) as i32 as u8 as u32;
                    }
                }
                if s == np - 2 {
                    if j == count - 1 {
                        alpha = 0;
                    } else if j == count - 2 {
                        alpha = (a4 * 0.5) as i32 as u8 as u32;
                    }
                }
                let c = visos_argb(alpha, r, g, b);
                verts.push(strip_vertex(p, uu as i32, false, 0.0, BARRIER_HEIGHT, c));
                verts.push(strip_vertex(p, uu as i32, true, 0.0, BARRIER_HEIGHT, c));
            }
            start = 1;
            dist_acc += seg;
        }
        draw.strip(sh, &verts);
        if !active {
            self.data[i].hit = -1.0;
            self.wobble = -1.0;
        }
    }

    /// BarrierRender.cpp 0x00109D70 — a no-entry barrier: dash strips from both ends up to `SIGN_GAP` short of the
    /// middle (0x0010A610); the sign between them is the entity `stop_sign_matrix` places
    fn draw_no_entry(&self, i: usize, a: i32, sh: &EffectShader, draw: &mut EffectDraw) {
        let (first, second) = no_entry_halves(&self.data[i]);
        if let Some((p, u)) = first {
            self.draw_dash(&p, &u, true, a, sh, draw);
        }
        if let Some((p, u)) = second {
            self.draw_dash(&p, &u, false, a, sh, draw);
        }
    }

    /// BarrierRender.cpp 0x0010A610 — one dash strip (batches of 47 segments); the top/bottom toggle starts from the
    /// debug build's 0xCC stack fill, i.e. on top
    fn draw_dash(&self, pts: &[Vec3], us: &[i32], reverse: bool, a: i32, sh: &EffectShader, draw: &mut EffectDraw) {
        let c = visos_argb(a as u32, GREY as u32, GREY as u32, GREY as u32);
        let count = pts.len();
        if count < 2 {
            return;
        }
        let mut remaining = count - 1;
        let mut k = 0usize;
        let mut top = true;
        let shift = (4096.0 * self.scroll) as i32;
        while remaining != 0 {
            let (n, mut vbase) = batch_points(us, k, remaining.min(47));
            let cnt = n + 3;
            if reverse {
                vbase -= shift;
            } else {
                vbase += shift;
            }
            let mut verts = Vec::with_capacity(cnt);
            for j in 0..cnt {
                let mut u = us[k] - vbase;
                if reverse {
                    u = -u;
                }
                verts.push(strip_vertex(pts[k], u, top, DASH_BOTTOM, DASH_TOP, c));
                top = !top;
                if j > 0 && j < cnt - 2 {
                    k += 1;
                }
            }
            remaining -= n;
            draw.strip(sh, &verts);
        }
    }
}

type DashHalf = Option<(Vec<Vec3>, Vec<i32>)>;

/// 0x00109D70's two halves: points + u (×2) from the start, and from the cut to the end, each cut `SIGN_GAP` short of
/// the middle (the original reuses one buffer: the first half is drawn before the second is written)
fn no_entry_halves(d: &BarrierRenderData) -> (DashHalf, DashHalf) {
    let np = d.points.len();
    let mut pts = vec![Vec3::ZERO; np + 1];
    let mut us = vec![0i32; np + 1];
    let half = d.length / 2.0 - SIGN_GAP;
    let scale = d.length / (d.u[np - 1] - d.u[0]) as f32;
    let mut acc = 0.0f32;
    let mut cnt = 0usize;
    let mut n0 = None;
    for i in 0..np - 1 {
        let p = d.points[i];
        let ua = d.u[i] as f32;
        let ub = d.u[i + 1] as f32;
        let seg = (ub - ua) * scale;
        acc += seg;
        pts[cnt] = p;
        us[cnt] = (ua * DASH_U_SCALE) as i32;
        cnt += 1;
        if acc > half {
            let f = 1.0 - (acc - half) / seg;
            us[cnt] = (((1.0 - f) * ua + ub * f) * DASH_U_SCALE) as i32;
            pts[cnt] = d.points[i + 1] * f + p * (1.0 - f);
            cnt += 1;
            n0 = Some((pts[..cnt].to_vec(), us[..cnt].to_vec()));
            break;
        }
    }
    // the second half is written backwards from the end of the buffer (index np = last + 1 slot)
    let mut back_pts = vec![Vec3::ZERO; np + 1];
    let mut back_us = vec![0i32; np + 1];
    let last = np - 1;
    acc = 0.0;
    cnt = 0;
    let mut n1 = None;
    for i in (1..np).rev() {
        let p = d.points[i];
        let ub = d.u[i] as f32;
        let ua = d.u[i - 1] as f32;
        let seg = (ub - ua) * scale;
        acc += seg;
        back_pts[last - cnt] = p;
        back_us[last - cnt] = (ub * DASH_U_SCALE) as i32;
        cnt += 1;
        if acc > half {
            let f = 1.0 - (acc - half) / seg;
            let k = last - cnt;
            back_us[k] = (((1.0 - f) * ub + ua * f) * DASH_U_SCALE) as i32;
            back_pts[k] = d.points[i - 1] * f + p * (1.0 - f);
            n1 = Some((back_pts[k..=last].to_vec(), back_us[k..=last].to_vec()));
            break;
        }
    }
    (n0, n1)
}

/// 0x00109D70 tail: the stop sign between the two cut points — rows x = along the line, y = up, z = x × up,
/// translation = the middle (FuryInstance 0x0012A7D0 draws the model's objects under it)
pub fn stop_sign_matrix(d: &BarrierRenderData) -> Option<Mat4> {
    let (first, second) = no_entry_halves(d);
    let (a, b) = (*first?.0.last()?, *second?.0.first()?);
    let centre = (b + a) * 0.5;
    let x = (b - a).normalize_or_zero();
    let up = Vec3::Y;
    let z = x.cross(up);
    Some(Mat4::from_cols(x.extend(0.0), up.extend(0.0), z.extend(0.0), centre.extend(1.0)))
}

pub struct BarrierRenderPlugin;

impl Plugin for BarrierRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BarrierRender>()
            .init_resource::<BarrierTweaks>()
            .add_message::<LoadBarrierRender>()
            .add_message::<UnloadBarrierRender>()
            .add_message::<BarrierHit>()
            .add_systems(Update, barrier_messages)
            .add_systems(PostUpdate, draw_barriers.in_set(crate::effect_draw::EffectOrder::Barriers));
    }
}

#[allow(clippy::too_many_arguments)]
fn barrier_messages(
    mut commands: Commands,
    mut br: ResMut<BarrierRender>,
    mut load: MessageReader<LoadBarrierRender>,
    mut unload: MessageReader<UnloadBarrierRender>,
    mut hits: MessageReader<BarrierHit>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<crate::renderer::shader_manager::XboxStageMaterial>>,
    mut images: ResMut<Assets<Image>>,
    lights: Option<Res<crate::track_render::TrackLights>>,
) {
    let drop_all =|br: &mut BarrierRender, commands: &mut Commands| {
        if let Some(r) = br.root.take() {
            commands.entity(r).despawn();
        }
        br.signs.clear();
        br.armco.clear();
        br.data.clear();
        br.initialised = false;
        br.current = None;
        br.wobble = -1.0;
    };
    if unload.read().count() > 0 {
        drop_all(&mut br, &mut commands);
    }
    for l in load.read() {
        drop_all(&mut br, &mut commands);
        br.init(l);
        // 0x00107560: barrierStopMainOn / Off from SCENERYM.VCD with SCENERYT.VCD textures (Off is never drawn)
        let bytes = visos::file::Vcd::open("\\Scenery\\SCENERYM.VCD").ok().and_then(|v| v.get("barrierStopMainOn.xmf").map(|d| d.to_vec()));
        let model = bytes.as_deref().and_then(|d| crate::renderer::renderer_types::Model::parse(d).ok());
        let root = commands.spawn((Transform::default(), Visibility::default(), Name::new("BarrierRender"))).id();
        br.root = Some(root);
        // Armco.cpp 0x00012290: one `armco_post` instance at every point with w == 1
        let post_bytes = visos::file::Vcd::open("\\Scenery\\SCENERYM.VCD").ok().and_then(|v| v.get("armco_post.xmf").map(|d| d.to_vec()));
        let post_model = post_bytes.as_deref().and_then(|d| crate::renderer::renderer_types::Model::parse(d).ok());
        if let Some(pm) = post_model.filter(|_| l.armco.iter().any(|a| !a.points.is_empty())) {
            let mut tex = crate::model_builder::TexSet::default();
            crate::car_render::add_vcd_textures(&mut tex, "\\Scenery\\SCENERYT.VCD");
            let mut b = crate::model_builder::Builder::new(&mut meshes, &mut materials, &mut images);
            if let Some(li) = &lights {
                b.lights = li.lights.clone();
            }
            let mut n = 0;
            for p in l.armco.iter().flat_map(|a| a.points.iter()).filter(|p| p.w == 1.0) {
                let opts = crate::model_builder::SpawnOpts { parent: Some(root), ..crate::model_builder::SpawnOpts::world() };
                b.add_model(&mut commands, &mut tex, &pm, Mat4::from_translation(p.truncate()), &opts);
                n += 1;
            }
            log::info!("armco: {} rail chunks, {n} posts", br.armco.len());
        }
        let Some(model) = model else {
            log::warn!("barrier render: barrierStopMainOn.xmf not loaded");
            continue;
        };
        let mut tex = crate::model_builder::TexSet::default();
        crate::car_render::add_vcd_textures(&mut tex, "\\Scenery\\SCENERYT.VCD");
        let mut b = crate::model_builder::Builder::new(&mut meshes, &mut materials, &mut images);
        if let Some(l) = &lights {
            b.lights = l.lights.clone();
        }
        let mut signs = Vec::new();
        for (i, d) in br.data.iter().enumerate().filter(|(_, d)| d.kind == 1) {
            let Some(m) = stop_sign_matrix(d) else { continue };
            let e = commands.spawn((Transform::default(), Visibility::Hidden, ChildOf(root))).id();
            let opts = crate::model_builder::SpawnOpts { parent: Some(e), ..crate::model_builder::SpawnOpts::world() };
            b.add_model(&mut commands, &mut tex, &model, m, &opts);
            signs.push((i, e));
        }
        br.signs = signs;
    }
    for h in hits.read() {
        log::debug!("barrier render: {h:?}, current {:?}", br.current);
        br.hit(h);
    }
}

fn draw_barriers(
    time: Res<Time>,
    clock: Res<EffectClock>,
    tweaks: Res<BarrierTweaks>,
    mut br: ResMut<BarrierRender>,
    mut draw: ResMut<EffectDraw>,
    cam: Query<(&GlobalTransform, Option<&Frustum>), With<TrackCamera>>,
    views: crate::view_render::ViewCams,
    mut vis: Query<&mut Visibility>,
) {
    if !br.initialised {
        return;
    }
    let Some((g, frustum)) = cam.iter().next() else {
        if !clock.paused {
            br.advance(time.delta_secs() * 1000.0 * clock.scale, &tweaks);
        }
        return;
    };
    // GameManager 0x000B9010: 0x00109080(mirror) in every view with render flag 0x08000000 — the views' cameras, then
    // the mirror's; each non-mirror call steps the wobble / scroll by the whole frame time (twice a frame for two
    // players), the mirror's (a3 = 1) does not
    let mut list = views.list();
    if list.is_empty() {
        list.push(crate::view_render::ViewCam { player: 0, camera: Entity::PLACEHOLDER, transform: *g, projection: None, frustum: frustum.cloned(), rect: [0.0, 0.0, 1.0, 1.0], tag: None });
    }
    let mirror = views.mirror();
    let mut shown = Vec::new();
    for (k, v) in list.iter().chain(mirror.iter()).enumerate() {
        let is_mirror = k == list.len();
        if !is_mirror && !clock.paused {
            br.advance(time.delta_secs() * 1000.0 * clock.scale, &tweaks);
        }
        let visible = |c: Vec3, r: f32| v.frustum.as_ref().is_none_or(|f| f.intersects_sphere(&Sphere { center: c.into(), radius: r }, true));
        draw.set_view(v.tag);
        br.draw_armco(&visible, &mut draw);
        let s = br.draw(v.transform.translation(), visible, &tweaks, &mut draw);
        // the no-entry sign models follow player 0's view
        if k == 0 {
            shown = s;
        }
    }
    draw.set_view(None);
    let signs = br.signs.clone();
    for (i, e) in signs {
        let want = if shown.contains(&i) { Visibility::Inherited } else { Visibility::Hidden };
        if let Ok(mut v) = vis.get_mut(e) {
            if *v != want {
                *v = want;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(n: usize, step: f32) -> Vec<Vec3> {
        (0..n).map(|i| Vec3::new(i as f32 * step, 0.0, 0.0)).collect()
    }

    #[test]
    fn armco_strip_pairs_and_repeats() {
        let l = ArmcoLineDraw { kind: 2, closed: false, points: (0..5).map(|i| Vec4::new(i as f32 * 16.0, 0.0, 0.0, 1.0)).collect() };
        let s = armco_strip(&l);
        assert_eq!(s.len(), 10);
        assert_eq!(s[1].pos.y, ARMCO_HEIGHT);
        // 64 m = 2 repeats
        assert!((s[8].uv.y - 2.0).abs() < 1e-4);
        assert!(armco_strip(&ArmcoLineDraw { kind: 1, ..l }).is_empty());
    }

    #[test]
    fn u_is_whole_thirds_of_a_repeat() {
        let d = BarrierRenderData::new(line(5, 2.9), 0).unwrap();
        // 11.6 m = 1.45 repeats -> 1.333.. (4 thirds)
        assert_eq!(d.u[0], 0);
        assert!((d.u[4] as f32 / 4096.0 - 4.0 / 3.0).abs() < 0.01, "{}", d.u[4]);
    }

    #[test]
    fn hit_position_along_the_current_barrier() {
        let mut br = BarrierRender::default();
        br.init(&LoadBarrierRender { arrows: vec![line(3, 10.0)], no_entry: vec![], render: true, armco: vec![] });
        br.current = Some(0);
        br.hit(&BarrierHit { car_a: 0, material: 21, point: Vec3::new(5.0, 1.0, 3.0) });
        assert!((br.data[0].hit - 0.25).abs() < 1e-6);
        assert_eq!(br.wobble, 0.0);
        // other cars and materials do nothing
        br.data[0].hit = -1.0;
        br.wobble = -1.0;
        br.hit(&BarrierHit { car_a: 1, material: 21, point: Vec3::ZERO });
        assert!(br.data[0].hit < 0.0);
    }

    #[test]
    fn wobble_runs_out_and_clears_the_hit() {
        let mut br = BarrierRender::default();
        let tw = BarrierTweaks::default();
        br.init(&LoadBarrierRender { arrows: vec![line(4, 5.0)], no_entry: vec![], render: true, armco: vec![] });
        br.current = Some(0);
        br.hit(&BarrierHit { car_a: 0, material: 21, point: Vec3::new(7.5, 0.0, 0.0) });
        let mut draw = EffectDraw::default();
        for _ in 0..2000 {
            br.advance(16.0, &tw);
            br.draw(Vec3::new(7.5, 2.0, 10.0), |_, _| true, &tw, &mut draw);
            if br.data[0].hit < 0.0 {
                break;
            }
        }
        assert!(br.data[0].hit < 0.0 && br.wobble < 0.0);
        assert!(!draw.batches.is_empty());
    }

    #[test]
    fn no_entry_leaves_a_gap_for_the_sign() {
        let d = BarrierRenderData::new(line(5, 5.0), 1).unwrap();
        let m = stop_sign_matrix(&d).unwrap();
        // distances come back from the 4096ths u table
        assert!(m.w_axis.truncate().abs_diff_eq(Vec3::new(10.0, 0.0, 0.0), 1e-2), "{}", m.w_axis);
        let (first, second) = no_entry_halves(&d);
        assert!((first.unwrap().0.last().unwrap().x - 7.5).abs() < 1e-2);
        assert!((second.unwrap().0[0].x - 12.5).abs() < 1e-2);
    }
}
