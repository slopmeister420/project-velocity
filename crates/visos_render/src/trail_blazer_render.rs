//! TrailBlazerRender.cpp (0x0014DF70..0x00152380; the idb labels 0x0014DF70..0x00150140 "TrackFX__*"): the zone
//! trails, two glowing ribbons laid on the road along every lane of every trailblazer section. Object
//! `dword_7374CC` (24 B, ctor 0x0014DEF0, singleton 0x000C04F0): +0 trails (16 × 36 B), +4 count, +8 intensity,
//! +12/+16 shaders "zonetrailone"/"zonetrailtwo", +20 enabled (1). Trail 36 B: +0 geometry blocks, +4/+8 sampled
//! spline points, +12 road line, +16 state (0 off, 1 growing, 2 shown, 3 fading), +20 next state, +24 lerped
//! timer, +28 timer, +32 previous timer (−1 = none).
//!
//! API (core): `TrailBlazerRenderCall::Init(lines)` at InGameInit (0x0014DF70 → 0x0014E480: every lane line of
//! every trailblazer section, `ZoneTrailLine`), `Reset` at RestartRace (0x0014E2A0), `Shutdown`; the trailblazer's
//! calls `Render { vehicle, spline }` (0x001521E0) and `Intensity(level)` (TrailBlazer 0x000C6240). Run
//! [`trail_blazer_render_update`] once per game update in `InGameUpdateSet::Effects` (InGameUpdate 0x000B82B3 →
//! 0x0014E530; it applies the queued calls first). The ribbons are drawn every frame (GameManager 0x000B9721 →
//! 0x0014E710) through `EffectDraw` with the timers lerped by `DangerZoneInput.lerp` (`dword_74A430` +32).

use bevy::prelude::*;
use visos::handy::crt_rand::CrtRand;

use crate::danger_zone_render::DangerZoneInput;
use crate::effect_draw::{EffectClock, EffectDraw,EffectShader, EffectTexture, EffectVertex};
use crate::race_effects::EffectCars;

/// `*(this + 1) >= 0x10` in 0x00151DC0
pub const MAX_ZONE_TRAILS: usize = 16;
/// segments per 0x3FC-byte geometry block (+964 count)
const BLOCK_SEGMENTS: usize = 16;

// module constants 0x006873D4..0x00687424 (bit-exact)
/// `dword_6873D4` spline sample spacing (m)
const SPACING: f32 = 2.0;
/// `dword_6873D8` half distance between the two ribbons
const HALF_WIDTH: f32 = f32::from_bits(0x3F59_999A);
/// `flt_6873DC` half width of one ribbon
const RIBBON_HALF: f32 = f32::from_bits(0x3ECC_CCCD);
/// `flt_6873E0` side ground probe half length
const PROBE_SIDE: f32 = 2.0;
/// `flt_6873E4` centre ground probe half length
const PROBE_CENTRE: f32 = 2.0;
/// `flt_6873E8` lift along the ground normal
const LIFT: f32 = f32::from_bits(0x3E23_D70A);
/// `dword_6873EC` / `dword_6873F0` uv scroll speeds of the two shaders (u per second)
const SCROLL_ONE: f32 = 0.25;
const SCROLL_TWO: f32 = f32::from_bits(0xBEF0_A3D7);
/// `dword_6873F4` / `dword_6873F8` points faded in at the start / out at the end
const FADE_IN: u32 = 3;
const FADE_OUT: u32 = 3;
/// `dword_6873FC` half period of the u wave (points)
const U_PERIOD: u32 = 10;
/// `dword_687400` length of the growing head (m)
const HEAD_LENGTH: f32 = 100.0;
/// `dword_687404` head peak position
const HEAD_PEAK: f32 = f32::from_bits(0x3F4C_CCCD);
/// `dword_687408` fading spread, `dword_68740C` growing spread
const FADE_SPREAD: f32 = 10.0;
const GROW_SPREAD: f32 = 10.0;
/// `dword_687410` / `dword_687414` growing / fading time (s)
const GROW_TIME: f32 = 0.5;
const FADE_TIME: f32 = 3.0;
/// `dword_687418` / `dword_68741C` random alpha range
const ALPHA_LO: f32 = 190.0;
const ALPHA_HI: f32 = 255.0;
/// `dword_687424` alpha boost at the head
const HEAD_BOOST: f32 = 2.0;
/// `flt_7374C0` head rise: bss, only read (0)
const HEAD_RISE: f32 = 0.0;
/// `dword_7374C4` = VISOS_ColourXBOX(0, 0x10, 0x20, 0x80) (static init 0x00426540), `unk_7374C8` = (0, 0x80, 0, 0)
pub const COLOUR_COLD: u32 = 0x0020_40FF;
pub const COLOUR_HOT: u32 = 0x00FF_0000;

/// one lane line of a trailblazer section as the AI world holds it (VsAIRoadLine: points, `GetLength`)
#[derive(Clone, Debug, Default)]
pub struct ZoneTrailLine {
    /// trailblazer section index and lane within it (the trail's +12 is the lane's line pointer)
    pub section: u32,
    pub lane: u32,
    /// the line's lane id (road line header +0xC)
    pub id: u32,
    pub points: Vec<Vec3>,
    pub length: f32,
}

/// calls into the zone trail object (core forwards them)
#[derive(Message, Clone, Debug)]
pub enum TrailBlazerRenderCall {
    /// GameManager::InGameInit → 0x0014DF70
    Init(Vec<ZoneTrailLine>),
    /// GameManager::RestartRace → 0x0014E2A0
    Reset,
    /// destructor list → 0x0014E280
    Shutdown,
    /// TrailBlazer 0x000C4870 (`Some(spline crc)`: 0x001521E0(line, 1, 0)) / 0x000C4E50 (`None`: the vehicle's last
    /// line, 0x001521E0(line, 3, 1))
    Render { vehicle: usize, spline: Option<u32> },
    /// TrailBlazer 0x000C6240 (+8)
    Intensity(f32),
}

/// one segment of a geometry block: two ribbons, each an edge pair
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TrailSegment {
    /// +0 / +12 left ribbon edges, +384 colour, +448 u (4096ths)
    pub a: [Vec3; 2],
    pub col_a: u32,
    pub u_a: u16,
    /// +480 / +492 right ribbon edges, +864 colour, +928 u
    pub b: [Vec3; 2],
    pub col_b: u32,
    pub u_b: u16,
}

/// a 0x3FC-byte geometry block (ctor 0x00152290); +968/+984 bounds (FLT_MAX / −FLT_MAX) and +1000 are never read
#[derive(Clone, Debug, Default)]
pub struct TrailBlock {
    pub segs: Vec<TrailSegment>,
    /// +960 strip breaks (bit k ends a strip after segment k; never set)
    pub breaks: u32,
}

/// a zone trail (36 B)
#[derive(Clone, Debug, Default)]
pub struct ZoneTrail {
    pub blocks: Vec<TrailBlock>,
    pub points: Vec<Vec3>,
    pub line: usize,
    pub state: u32,
    pub next: u32,
    pub lerped: f32,
    pub timer: f32,
    pub prev: f32,
}

/// the zone trail object `dword_7374CC`
#[derive(Resource, Clone, Debug)]
pub struct TrailBlazerRender {
    pub trails: Vec<ZoneTrail>,
    pub lines: Vec<ZoneTrailLine>,
    /// +8
    pub intensity: f32,
    /// +20
    pub enabled: bool,
    /// the line each vehicle's trailblazer last turned on (TrailBlazer +0x14)
    pub last_spline: Vec<(usize, u32)>,
    /// uv anim clock (s) of the two shaders' `eMS_UvAnim` records
    pub anim_time: f32,
    /// `byte_7187E0` (CatsEyeRender object +0x20; only read): lifts the ribbons by 0.2
    pub lift: bool,
    /// VsRandUnit stream used for the per-point alphas
    pub rng: CrtRand,
}

impl Default for TrailBlazerRender {
    fn default() -> Self {
        Self { trails: Vec::new(), lines: Vec::new(), intensity: 0.0, enabled: true, last_spline: Vec::new(), anim_time: 0.0, lift: false, rng: CrtRand::default() }
    }
}

/// a ground probe: `(from, to)` → fraction and surface normal (VsAITools 0x00028460 → PhysicsManager::RayCast
/// 0x0019D3E0 with flags 17 = track + normal, no car ignored)
pub type GroundRay<'a> = dyn FnMut(Vec3, Vec3) -> Option<(f32, Vec3)> + 'a;

/// TrailBlazerRender.cpp 0x00151ED0: the state a trail moves to when its timer runs out
fn next_state(s: u32) -> u32 {
    match s {
        1 => 2,
        3 => 0,
        s => s,
    }
}

/// TrailBlazerRender.cpp 0x00151F10
fn is_settled(s: u32) -> bool {
    s == 0 || s == 2
}

/// VsCameraManager 0x000613E0
fn lerp3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    Vec3::new((b.x - a.x) * t + a.x, (b.y - a.y) * t + a.y, (b.z - a.z) * t + a.z)
}

/// VsVector4::Normalise 0x000135B0 (|len| ≤ 1e-6 → the zero vector, Armco 0x00013670)
fn normalise(v: Vec3) -> Vec3 {
    let l = v.length();
    if l.abs() <= 0.000001 {
        return Vec3::ZERO;
    }
    Vec3::new(v.x / l, v.y / l, v.z / l)
}

/// VsRandRange 0x000134F0
fn rand_range(rng: &mut CrtRand, lo: f32, hi: f32) -> f32 {
    (rng.rand_unit() * (hi - lo) as f64 + lo as f64) as f32
}

/// `_ftol(ceil(x))` (tweak 0x000863B0 + 0x003DDFA8)
fn ceil_ftol(x: f32) -> u32 {
    (x as f64).ceil() as i64 as u32
}

/// FuryInstance.cpp 0x0012A680: bytewise lerp of two colours by `t` in 128ths
pub fn colour_lerp(a: u32, b: u32, t: f32) -> u32 {
    let w = ((t as f64 * 128.0).clamp(0.0, 128.0)) as i64 as u32;
    let mut out = 0u32;
    for s in [24, 16, 8, 0] {
        let (x, y) = ((a >> s) & 0xFF, (b >> s) & 0xFF);
        out |= (((128 - w) * x + w * y) >> 7 & 0xFF) << s;
    }
    out
}

/// the three-ray ground probe of 0x001501E0 around `c` (`top` = c + up·len, `bot` = c − up·len): ground point
/// and normal, or `None` (then the caller uses `bot`, its fallback normal and clears the alpha)
fn probe(ray: &mut GroundRay, c: Vec3, top: Vec3, bot: Vec3) -> Option<(Vec3, Vec3)> {
    let mut up = ray(c, top);
    let down = ray(c, bot);
    if up.is_none() && down.is_none() {
        // the full top→bottom ray stores t = 0 (not its own t): the ground is the probe centre (bug kept)
        if let Some((_, n)) = ray(top, bot) {
            up = Some((0.0, n));
        }
    }
    match (up, down) {
        (Some((t1, n1)), d) => {
            let mut g = (lerp3(c, top, t1), n1);
            if let Some((t2, n2)) = d {
                if t1 > t2 {
                    g = (lerp3(c, bot, t2), n2);
                }
            }
            Some(g)
        }
        (None, Some((t2, n2))) => Some((lerp3(c, bot, t2), n2)),
        _ => None,
    }
}

impl ZoneTrail {
    /// TrailBlazerRender.cpp 0x00151880 — append a segment; a full block continues in a new one that starts with
    /// a copy of the last segment (strips stay joined)
    pub fn push_segment(&mut self, s: TrailSegment) {
        if self.blocks.last().is_none_or(|b| b.segs.len() == BLOCK_SEGMENTS) {
            let carry = self.blocks.last().and_then(|b| (b.breaks & (1 << (b.segs.len() - 1)) == 0).then(|| *b.segs.last().unwrap()));
            let mut nb = TrailBlock::default();
            nb.segs.extend(carry);
            self.blocks.push(nb);
        }
        self.blocks.last_mut().unwrap().segs.push(s);
    }

    /// TrailBlazerRender.cpp 0x001501E0 — sample the line every ~2 m, probe the ground at the centre and at both
    /// sides, and lay two ribbons 1.7 apart, 0.8 wide, lifted 0.16 along the ground normals
    pub fn build(&mut self, line: &ZoneTrailLine, ray: &mut GroundRay, rng: &mut CrtRand) {
        let pts = &line.points;
        let n = pts.len();
        if n == 0 {
            return;
        }
        let length = line.length;
        let nseg = ceil_ftol(length / SPACING);
        let step = (length as f64 / nseg as f64) as f32;
        let mut spl = vec![Vec3::ZERO; nseg as usize + 1];
        let (mut acc, mut idx, mut o) = (0.0f32, 0usize, 0u32);
        let (mut seg_len, mut d) = (-1.0f32, 0.0f32);
        while o < nseg {
            while d >= acc {
                idx += 1;
                if idx >= n {
                    seg_len = -1.0;
                    break;
                }
                seg_len = pts[idx - 1].distance(pts[idx]);
                acc += seg_len;
            }
            let f = if seg_len <= 0.0 { 1.0 } else { (1.0 - (acc - d) / seg_len).clamp(0.0, 1.0) };
            // past the end the original reads beyond the point array; clamped here
            spl[o as usize] = lerp3(pts[(idx - 1).min(n - 1)], pts[idx.min(n - 1)], f);
            o += 1;
            d += step;
        }
        spl[nseg as usize] = pts[n - 1];
        let np = spl.len() as u32;
        self.points = spl;
        if np < 2 {
            return;
        }
        for i in 0..np {
            let p = self.points[i as usize];
            let dir = if i >= np - 1 { p - self.points[i as usize - 1] } else { self.points[i as usize + 1] - p };
            let mut fade = 1.0f32;
            if i >= FADE_IN {
                let k = np.wrapping_sub(1).wrapping_sub(FADE_OUT);
                if i > k {
                    fade = (1.0 - (i - k) as f64 / (np - 1 - k) as f64) as f32;
                }
            } else {
                fade = (i as f64 / FADE_IN as f64) as f32;
            }
            let r = rand_range(rng, 0.0, 1.0);
            let a1 = ((ALPHA_HI - ALPHA_LO) * r + ALPHA_LO) * fade;
            let r = rand_range(rng, 0.0, 1.0);
            let a2 = ((ALPHA_HI - ALPHA_LO) * r + ALPHA_LO) * fade;
            let mut col_l = COLOUR_COLD | (a1 as i64 as u32) << 24;
            let mut col_r = COLOUR_COLD | (a2 as i64 as u32) << 24;
            // centre: only the normal is used
            let up = Vec3::Y * PROBE_CENTRE;
            let n_c = match probe(ray, p, p + up, p - up) {
                Some((_, n)) => n,
                None => {
                    col_l &= 0xFF_FFFF;
                    col_r &= 0xFF_FFFF;
                    Vec3::Y
                }
            };
            let side = normalise(dir.cross(n_c)) * HALF_WIDTH;
            let up2 = normalise(side.cross(dir));
            let c = p - side;
            let (top, bot) = (c + up2 * PROBE_SIDE, c - up2 * PROBE_SIDE);
            let (l, n_l) = probe(ray, c, top, bot).unwrap_or_else(|| {
                col_l &= 0xFF_FFFF;
                (bot, up2)
            });
            let c = p + side;
            let (top, bot) = (c + up2 * PROBE_SIDE, c - up2 * PROBE_SIDE);
            let (r, n_r) = probe(ray, c, top, bot).unwrap_or_else(|| {
                col_r &= 0xFF_FFFF;
                (bot, up2)
            });
            // squeeze the two ground points to 1.7 apart about the centre's projection onto their plane
            let span = r - l;
            let k = (HALF_WIDTH as f64 * 2.0 / span.length() as f64) as f32;
            let m = normalise(span.cross(dir));
            let mut w = p - l;
            w -= m * m.dot(w);
            let mid = l + w;
            let l = mid + (l - mid) * k + n_l * LIFT;
            let r = mid + (r - mid) * k + n_r * LIFT;
            let ph = i % (2 * U_PERIOD);
            let u = if ph >= U_PERIOD { 1.0 - (ph - U_PERIOD) as f64 / U_PERIOD as f64 } else { ph as f64 / U_PERIOD as f64 } as f32;
            let u = (u * 4096.0) as i32 as u16;
            let s_l = normalise(dir.cross(n_l)) * RIBBON_HALF;
            let s_r = normalise(dir.cross(n_r)) * RIBBON_HALF;
            self.push_segment(TrailSegment { a: [l - s_l, l + s_l], col_a: col_l, u_a: u, b: [r - s_r, r + s_r], col_b: col_r, u_b: u });
        }
    }
}

/// the strips of one draw pass: (vertex, ...) per strip, uv in texture units
type Strips = Vec<Vec<EffectVertex>>;

fn vtx(p: Vec3, argb: u32, u: u16, top: bool) -> EffectVertex {
    EffectVertex::new(p, argb, Vec2::new(u as f32 / 4096.0, if top { 1.0 } else { 0.0 }))
}

/// segment `k` of `b` ends a strip
fn strip_end(b: &TrailBlock, k: usize) -> bool {
    k + 1 == b.segs.len() || b.breaks & (1 << k) != 0
}

impl TrailBlazerRender {
    /// TrailBlazerRender.cpp 0x0014DF70 (shaders are built in `draw`) → 0x0014E480: one trail per lane line
    pub fn init(&mut self, lines: Vec<ZoneTrailLine>, ray: &mut GroundRay) {
        self.shutdown();
        self.lines = lines;
        for i in 0..self.lines.len() {
            self.add_trail(i, ray);
        }
        self.intensity = 0.0;
    }

    /// TrailBlazerRender.cpp 0x00151DC0
    fn add_trail(&mut self, line: usize, ray: &mut GroundRay) -> i32 {
        if self.lines[line].points.len() <= 1 || self.trails.len() >= MAX_ZONE_TRAILS {
            return -1;
        }
        let mut t = ZoneTrail { line, state: 0, next: next_state(0), lerped: 0.0, timer: 0.0, prev: -1.0, ..Default::default() };
        let l = self.lines[line].clone();
        t.build(&l, ray, &mut self.rng);
        self.trails.push(t);
        self.trails.len() as i32 - 1
    }

    /// TrailBlazerRender.cpp 0x0014E2A0 (RestartRace)
    pub fn reset(&mut self) {
        for t in &mut self.trails {
            t.state = 0;
            t.next = next_state(0);
            t.lerped = 0.0;
            t.timer = 0.0;
            t.prev = -1.0;
        }
        self.intensity = 0.0;
    }

    /// TrailBlazerRender.cpp 0x0014E360
    pub fn shutdown(&mut self) {
        self.trails.clear();
        self.lines.clear();
        self.last_spline.clear();
    }

    /// TrailBlazerRender.cpp 0x00151F50 — `state` now, its successor next
    pub fn set_state(&mut self, i: usize, state: u32, timer: f32) {
        assert!(i < self.trails.len(), "(TrailIndex >= 0) && (TrailIndex < m_nNumZoneTrails)");
        let t = &mut self.trails[i];
        t.prev = if t.state == state { t.timer } else { -1.0 };
        t.state = state;
        t.lerped = timer;
        t.timer = timer;
        t.next = next_state(state);
    }

    /// TrailBlazerRender.cpp 0x00152100 — `queue`: only set the next state, unless the trail is settled
    pub fn change_state(&mut self, i: usize, state: u32, mut queue: bool) {
        assert!(i < self.trails.len(), "(TrailIndex >= 0) && (TrailIndex < m_nNumZoneTrails)");
        if is_settled(self.trails[i].state) {
            queue = false;
        }
        if queue {
            self.trails[i].next = state;
        } else {
            self.set_state(i, state, 0.0);
        }
    }

    /// TrailBlazerRender.cpp 0x001521E0 — by road line (here: its index in `lines`)
    pub fn change_line_state(&mut self, line: usize, state: u32, queue: bool) {
        if let Some(i) = self.trails.iter().position(|t| t.line == line) {
            self.change_state(i, state, queue);
        }
    }

    /// the line the trailblazer follows for spline `id`: the first lane of the first section having a lane with
    /// that id (VsAIWorld 0x00034F70 / 0x00034E80, `section + 8`)
    pub fn line_for_spline(&self, id: u32) -> Option<usize> {
        let sec = self.lines.iter().find(|l| l.id == id)?.section;
        self.lines.iter().position(|l| l.section == sec && l.lane == 0)
    }

    pub fn apply(&mut self, call: &TrailBlazerRenderCall, ray: &mut GroundRay) {
        match call {
            TrailBlazerRenderCall::Init(lines) => self.init(lines.clone(), ray),
            TrailBlazerRenderCall::Reset => self.reset(),
            TrailBlazerRenderCall::Shutdown => self.shutdown(),
            TrailBlazerRenderCall::Intensity(v) => self.intensity = *v,
            TrailBlazerRenderCall::Render { vehicle, spline: Some(id) } => {
                self.last_spline.retain(|(v, _)| v != vehicle);
                self.last_spline.push((*vehicle, *id));
                if let Some(l) = self.line_for_spline(*id) {
                    self.change_line_state(l, 1, false);
                }
            }
            TrailBlazerRenderCall::Render { vehicle, spline: None } => {
                if let Some(&(_, id)) = self.last_spline.iter().find(|(v, _)| v == vehicle) {
                    if let Some(l) = self.line_for_spline(id) {
                        self.change_line_state(l, 3, true);
                    }
                }
            }
        }
    }

    /// TrailBlazerRender.cpp 0x0014E530 — per game update (`dt` = timer dt × scale, GameManager 0x000BF670)
    pub fn update(&mut self, dt: f32, paused: bool) {
        if paused || !self.enabled {
            return;
        }
        for j in 0..self.trails.len() {
            let t = &mut self.trails[j];
            t.prev = t.timer;
            match t.state {
                1 => t.timer += dt / GROW_TIME,
                3 => t.timer += dt / FADE_TIME,
                _ => {}
            }
            if t.timer >= 1.0 {
                let next = t.next;
                self.change_state(j, next, false);
            }
        }
    }

    /// TrailBlazerRender.cpp 0x0014E710 — the strips of every visible trail (one stream, drawn with both shaders)
    pub fn strips(&mut self, lerp: f32, paused: bool) -> Strips {
        let base = colour_lerp(COLOUR_COLD, COLOUR_HOT, self.intensity) & 0xFF_FFFF;
        let yoff = if self.lift { 0.2 } else { 0.0 };
        let mut out = Strips::new();
        for t in &mut self.trails {
            if t.state == 0 {
                continue;
            }
            if !paused {
                t.lerped = if t.prev == -1.0 { t.timer } else { (t.timer - t.prev) * lerp + t.prev };
            }
            match t.state {
                1 => growing(t, base, yoff, &mut out),
                3 => fading(t, base, yoff, &mut out),
                _ => shown(t, base, yoff, &mut out),
            }
        }
        out
    }

    /// 0x0014DF70: "zonetrailone" / "zonetrailtwo" = `streek01` (found among the loaded textures: `\textures.vcd`
    /// does not exist, misc.vcd has it), stage flags 4, blend 7, 0x002A15F0(1), wrap u / clamp v, z test no write,
    /// an `eMS_UvAnim` record (type 1 = u, speed 0.25 / −0.47, 0x003016E0) each
    pub fn shaders() -> [(EffectShader, f32); 2] {
        let sh = EffectShader::new(EffectTexture::Misc("streek01".into()), 7).z(true, false).wrap(true, false);
        [(sh.clone(), SCROLL_ONE), (sh, SCROLL_TWO)]
    }

    pub fn draw(&mut self, lerp: f32, paused: bool, draw: &mut EffectDraw) {
        let strips = self.strips(lerp, paused);
        if strips.is_empty() {
            return;
        }
        for (sh, speed) in Self::shaders() {
            // 0x003015F0: offset = seconds · speed; the u wrap makes only the fraction matter
            let off = (self.anim_time as f64 * speed as f64).rem_euclid(1.0) as f32;
            for s in &strips {
                let v: Vec<EffectVertex> = s.iter().map(|v| EffectVertex { uv: Vec2::new(v.uv.x + off, v.uv.y), ..*v }).collect();
                draw.strip(&sh, &v);
            }
        }
    }
}

/// TrailBlazerRender.cpp 0x0014F3A0 — state 2: the ribbons as built
fn shown(t: &ZoneTrail, base: u32, yoff: f32, out: &mut Strips) {
    for b in &t.blocks {
        for right in [false, true] {
            let mut k = 0;
            while k < b.segs.len() {
                let mut s = Vec::new();
                loop {
                    let g = &b.segs[k];
                    let (e, col, u) = if right { (g.b, g.col_b, g.u_b) } else { (g.a, g.col_a, g.u_a) };
                    let c = base | col & 0xFF00_0000;
                    s.push(vtx(Vec3::new(e[0].x, e[0].y + yoff, e[0].z), c, u, false));
                    s.push(vtx(Vec3::new(e[1].x, e[1].y + yoff, e[1].z), c, u, true));
                    let end = strip_end(b, k);
                    k += 1;
                    if end {
                        break;
                    }
                }
                if s.len() > 3 {
                    out.push(s);
                }
            }
        }
    }
}

/// TrailBlazerRender.cpp 0x0014F960 — state 3: each ribbon spreads to 10× its width (edges crossing) and fades
fn fading(t: &ZoneTrail, base: u32, yoff: f32, out: &mut Strips) {
    let x = 1.0 - t.lerped;
    let x = 1.0 - x * x;
    let sub = (x * ALPHA_HI) as i64 as i32;
    let s = (FADE_SPREAD - 1.0) * t.lerped + 1.0;
    for b in &t.blocks {
        for right in [false, true] {
            let mut k = 0;
            while k < b.segs.len() {
                let mut st = Vec::new();
                loop {
                    let g = &b.segs[k];
                    let (e, col, u) = if right { (g.b, g.col_b, g.u_b) } else { (g.a, g.col_a, g.u_a) };
                    let a = ((col >> 24) as i32 - sub).max(0) as u32;
                    let c = base | a << 24;
                    let p0 = lerp3(e[0], e[1], s);
                    let p1 = lerp3(e[1], e[0], s);
                    st.push(vtx(Vec3::new(p0.x, p0.y + yoff, p0.z), c, u, false));
                    st.push(vtx(Vec3::new(p1.x, p1.y + yoff, p1.z), c, u, true));
                    let end = strip_end(b, k);
                    k += 1;
                    if end {
                        break;
                    }
                }
                if st.len() > 3 {
                    out.push(st);
                }
            }
        }
    }
}

/// TrailBlazerRender.cpp 0x0014E940 — state 1: a 100 m head runs along the trail; ahead of it nothing, inside it
/// the ribbons narrow from 10× and brighten (×2 at 0.8 of the head), behind it they are as built
fn growing(t: &ZoneTrail, base: u32, yoff: f32, out: &mut Strips) {
    let np = t.points.len() as u32;
    let head_n = ceil_ftol(HEAD_LENGTH / SPACING);
    let total = head_n + np;
    let last = (total as f64 * t.lerped as f64) as i64 as u32;
    let first = if last <= head_n { 0 } else { last - head_n };
    let head = ((np.wrapping_sub(1) as f64 * SPACING as f64 + HEAD_LENGTH as f64) * t.lerped as f64) as f32;
    let tail = head - HEAD_LENGTH;
    // the index steps back at every block (its first segment repeats the previous block's last) but is not
    // advanced at a block's end, so later blocks run one behind (original)
    let mut idx: u32 = 1;
    let mut dist = 0.0f32;
    let mut done = false;
    for b in &t.blocks {
        if done {
            break;
        }
        idx = idx.wrapping_sub(1);
        let mut k = 0;
        while k < b.segs.len() && !done {
            let (mut sa, mut sb) = (Vec::new(), Vec::new());
            loop {
                let g = &b.segs[k];
                let mut ca = base | g.col_a & 0xFF00_0000;
                let mut cb = base | g.col_b & 0xFF00_0000;
                let (mut h, mut s) = (0.0f32, 1.0f32);
                if idx >= first {
                    let f = (dist - tail) / (head - tail);
                    let f = if f <= 1.0 {
                        if f < 0.0 {
                            0.0
                        } else {
                            f
                        }
                    } else {
                        1.0
                    };
                    let af = if HEAD_PEAK <= f { (1.0 - (f - HEAD_PEAK) / (1.0 - HEAD_PEAK)) * HEAD_BOOST } else { (HEAD_BOOST - 1.0) * (f / HEAD_PEAK) + 1.0 };
                    let boost = |c: u32| ((((c >> 24) as f64 * af as f64) as i64 as i32).clamp(0, 255) as u32) << 24;
                    ca = base | boost(ca);
                    cb = base | boost(cb);
                    h = ((((1.0 - f) * std::f32::consts::PI).cos() * 0.5 + 0.5) as f32) * HEAD_RISE;
                    s = (GROW_SPREAD - 1.0) * f + 1.0;
                }
                let y = |p: Vec3| p.y + h + yoff;
                let p0 = lerp3(g.a[0], g.a[1], s);
                let p1 = lerp3(g.a[1], g.a[0], s);
                sa.push(vtx(Vec3::new(p0.x, y(p0), p0.z), ca, g.u_a, false));
                sa.push(vtx(Vec3::new(p1.x, y(p1), p1.z), ca, g.u_a, true));
                let p0 = lerp3(g.b[0], g.b[1], s);
                let p1 = lerp3(g.b[1], g.b[0], s);
                sb.push(vtx(Vec3::new(p0.x, y(p0), p0.z), cb, g.u_b, false));
                sb.push(vtx(Vec3::new(p1.x, y(p1), p1.z), cb, g.u_b, true));
                let end = strip_end(b, k) || idx == last;
                if end {
                    if idx == last {
                        done = true;
                    }
                } else {
                    idx += 1;
                    dist += SPACING;
                }
                k += 1;
                if end {
                    break;
                }
            }
            if sa.len() > 3 {
                out.push(sa);
                out.push(sb);
            }
        }
    }
}

/// run once per game update (core: `InGameUpdateSet::Effects`): the queued calls, then 0x0014E530
pub fn trail_blazer_render_update(
    mut calls: MessageReader<TrailBlazerRenderCall>,
    mut r: ResMut<TrailBlazerRender>,
    fx: Res<EffectCars>,
    clock: Res<EffectClock>,
    physics: Option<Res<velocity_physics::plugin::PhysicsWorldRes>>,
) {
    let mut ray = |a: Vec3, b: Vec3| -> Option<(f32, Vec3)> {
        use velocity_physics::vehicle::vehicle_entity::sim::{RAY_NORMAL, RAY_TRACK};
        let h = physics.as_ref()?.lock().as_mut()?.ray_cast(a.to_array(), b.to_array(), RAY_TRACK | RAY_NORMAL, -1)?;
        Some((h.t, Vec3::from(h.normal)))
    };
    for c in calls.read() {
        r.apply(c, &mut ray);
    }
    r.update(fx.dt, clock.paused);
}

fn draw_trail_blazer(mut r: ResMut<TrailBlazerRender>, input: Option<Res<DangerZoneInput>>, time: Res<Time>, mut draw: ResMut<EffectDraw>) {
    if r.trails.is_empty() {
        return;
    }
    let (lerp, paused) = input.map_or((1.0, false), |i| (i.lerp, i.paused));
    if !paused {
        r.anim_time += time.delta_secs();
    }
    // GameManager 0x000B9010 calls it ungated in every view, the mirror's too: world space, one build for all
    draw.set_view(Some(crate::effect_draw::ALL_VIEWS));
    r.draw(lerp, paused, &mut draw);
    draw.set_view(None);
}

pub struct TrailBlazerRenderPlugin;

impl Plugin for TrailBlazerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrailBlazerRender>().add_message::<TrailBlazerRenderCall>().add_systems(PostUpdate, draw_trail_blazer.in_set(crate::effect_draw::EffectOrder::Trail));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// flat ground at y = 0 (normal up)
    fn flat(a: Vec3, b: Vec3) -> Option<(f32, Vec3)> {
        if (a.y >= 0.0) == (b.y >= 0.0) && a.y != 0.0 {
            return None;
        }
        let t = if a.y == b.y { 0.0 } else { a.y / (a.y - b.y) };
        Some((t, Vec3::Y))
    }

    fn line(len: f32) -> ZoneTrailLine {
        ZoneTrailLine { section: 0, lane: 0, id: 0x3E7E_D980, points: vec![Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 0.5, -len)], length: len }
    }

    fn built(len: f32) -> TrailBlazerRender {
        let mut r = TrailBlazerRender::default();
        r.init(vec![line(len), ZoneTrailLine { lane: 1, id: 7, ..line(len) }], &mut flat);
        r
    }

    #[test]
    fn geometry_on_flat_ground() {
        let r = built(61.0);
        let t = &r.trails[0];
        // ceil(61/2) = 31 segments, 32 points, blocks of 16 with a carried first segment
        assert_eq!(t.points.len(), 32);
        assert_eq!(t.blocks.iter().map(|b| b.segs.len()).collect::<Vec<_>>(), vec![16, 16, 2]);
        assert_eq!(t.blocks[1].segs[0], t.blocks[0].segs[15]);
        let g = &t.blocks[0].segs[5];
        // ribbons 1.7 apart, each 0.8 wide, 0.16 above the ground
        let (ca, cb) = ((g.a[0] + g.a[1]) * 0.5, (g.b[0] + g.b[1]) * 0.5);
        assert!((ca.distance(cb) - 1.7).abs() < 1e-4, "{}", ca.distance(cb));
        assert!((g.a[0].distance(g.a[1]) - 0.8).abs() < 1e-4);
        assert!((g.a[0].y - 0.16).abs() < 1e-5 && (g.b[1].y - 0.16).abs() < 1e-5);
        // u triangle wave over 20 points, alphas 190..255 except the 3 faded ends
        assert_eq!(t.blocks[0].segs.iter().map(|s| s.u_a).take(12).collect::<Vec<_>>(), [0, 409, 819, 1228, 1638, 2048, 2457, 2867, 3276, 3686, 4096, 3686]);
        assert_eq!(t.blocks[0].segs[0].col_a >> 24, 0);
        assert!((190..=255).contains(&(g.col_a >> 24)) && (190..=255).contains(&(g.col_b >> 24)));
        assert_eq!(g.col_a & 0xFF_FFFF, COLOUR_COLD);
    }

    #[test]
    fn no_ground_hides_the_point() {
        let mut r = TrailBlazerRender::default();
        r.init(vec![line(20.0)], &mut |_, _| None);
        assert!(r.trails[0].blocks[0].segs.iter().all(|s| s.col_a >> 24 == 0 && s.col_b >> 24 == 0));
    }

    #[test]
    fn states_and_timers() {
        let mut r = built(40.0);
        let mut ray = flat;
        r.apply(&TrailBlazerRenderCall::Render { vehicle: 0, spline: Some(7) }, &mut ray);
        // lane 1's id finds its section's lane 0 (the trailblazer follows section + 8)
        assert_eq!((r.trails[0].state, r.trails[0].next, r.trails[1].state), (1, 2, 0));
        // 0.5 s growing at 24 Hz (12 steps sum to just under 1 in f32), then shown
        for _ in 0..12 {
            r.update(1.0 / 24.0, false);
        }
        assert_eq!(r.trails[0].state, 1);
        r.update(1.0 / 24.0, false);
        assert_eq!(r.trails[0].state, 2);
        // off while shown: fades at once; 3 s later off
        r.apply(&TrailBlazerRenderCall::Render { vehicle: 0, spline: None }, &mut ray);
        assert_eq!(r.trails[0].state, 3);
        for _ in 0..73 {
            r.update(1.0 / 24.0, false);
        }
        assert_eq!(r.trails[0].state, 0);
        // off while growing only queues the fade
        r.apply(&TrailBlazerRenderCall::Render { vehicle: 0, spline: Some(0x3E7E_D980) }, &mut ray);
        r.update(1.0 / 24.0, false);
        r.apply(&TrailBlazerRenderCall::Render { vehicle: 0, spline: None }, &mut ray);
        assert_eq!((r.trails[0].state, r.trails[0].next), (1, 3));
        for _ in 0..13 {
            r.update(1.0 / 24.0, false);
        }
        assert_eq!(r.trails[0].state, 3);
        r.reset();
        assert_eq!((r.trails[0].state, r.trails[0].prev), (0, -1.0));
    }

    #[test]
    fn draw_passes() {
        let mut r = built(61.0);
        let mut ray = flat;
        assert!(r.strips(1.0, false).is_empty());
        r.apply(&TrailBlazerRenderCall::Render { vehicle: 0, spline: Some(0x3E7E_D980) }, &mut ray);
        // growing at t = 0: nothing past the first point
        assert!(r.strips(1.0, false).is_empty());
        r.trails[0].timer = 0.5;
        let g = r.strips(1.0, false);
        assert!(!g.is_empty() && g.len() % 2 == 0);
        r.set_state(0, 2, 0.0);
        // shown: two ribbons per block
        let s = r.strips(1.0, false);
        assert_eq!(s.len(), 6);
        assert_eq!(s[0].len(), 32);
        r.set_state(0, 3, 0.5);
        let f = r.strips(1.0, false);
        // fading at 0.5: alpha − _ftol(0.75·255), edges spread by 5.5 each way (crossed: (2·5.5 − 1)·0.8 apart)
        let a = (r.trails[0].blocks[0].segs[5].col_a >> 24) as i32;
        assert_eq!((f[0][10].argb >> 24) as i32, (a - 191).max(0));
        assert!((f[0][10].pos.distance(f[0][11].pos) - 8.0).abs() < 1e-3);
    }

    #[test]
    fn colour_by_intensity() {
        assert_eq!(colour_lerp(COLOUR_COLD, COLOUR_HOT, 0.0), COLOUR_COLD);
        assert_eq!(colour_lerp(COLOUR_COLD, COLOUR_HOT, 1.0), COLOUR_HOT);
        assert_eq!(colour_lerp(COLOUR_COLD, COLOUR_HOT, 0.5), 0x008F_207F);
    }
}
