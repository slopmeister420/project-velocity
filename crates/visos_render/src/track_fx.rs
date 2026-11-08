//! Render\TrackFX.cpp (lights part; the idb labels 0x00147DC0..0x00149970 StartLineEffects) — lens flares on the
//! track's point lights and the light streaks of the near ones. Per light a flare id and a road reflection id come
//! from its PtLights name (`..._<n>_<flare>_<reflection>`, 0x0009D860 / 0x0014AEC0); the textures and flare sizes
//! from the "FLARES" block of Setup.txt (0x0014D140). Each frame (0x0014A5B0): lights in view beyond the streak
//! depth get a camera-facing flare quad, nearer ones become light streak records (at most 16, the farthest
//! replaced), streaked (LightStreak.cpp) and get the near flare pair (0x00149970). Road reflections 0x0014A010
//! (ground ray 0x00149020, which also stores the ground height in each point light's colour w for VsRender's
//! overlap reduction 0x0005A040) and the car headlights 0x0014B030 (reflections, beams) follow.

use bevy::math::{Vec2, Vec3, Vec4};

use crate::effect_draw::{EffectDraw, EffectShader, EffectTexture, EffectVertex};
use crate::fog_render::{setup_tokens, SetupToken};
use crate::light_streak::{visos_colour, LightStreakManager, StreakLight, MAX_LIGHTSTREAK_LIGHTS};

/// `MAX_LIGHT_FLARE_TEXTURES` / `MAX_LIGHT_REFLECTION_TEXTURES`
pub const MAX_FLARE_TEXTURES: usize = 10;
/// `dword_6872E4`: radius of a light's visibility sphere
pub const FLARE_CULL_RADIUS: f32 = 3.0;
/// `flt_6872B8`: flares are pulled this far towards the camera
pub const FLARE_PULL: f32 = f32::from_bits(0x3E4C_CCCD);
/// `byte_6872BC`: flare alpha
pub const FLARE_ALPHA: u8 = 0x80;
/// `dword_6872E0`: depth bias of the "lightflare" shaders
pub const FLARE_DEPTH_BIAS: i16 = 0x20;
/// `byte_6872A8` / `byte_6872A9`: grey of the first / second near flare
pub const NEAR_FLARE_GREY: [u8; 2] = [0x40, 0x80];
/// `flt_6872B4` / `flt_6872AC`: near flare sizes at the screen centre line (first / second), `flt_6872B0` at the side
pub const NEAR_FLARE_SIZE: [f32; 2] = [f32::from_bits(0x4120_0000), f32::from_bits(0x40A0_0000)];
pub const NEAR_FLARE_SIZE_SIDE: f32 = f32::from_bits(0x4110_0000);

/// `dword_687288` / `dword_68728C`: road reflections fade out from 50 to 100 m
pub const REFL_FAR: f32 = f32::from_bits(0x42C8_0000);
pub const REFL_NEAR: f32 = f32::from_bits(0x4248_0000);
/// `dword_687290` / `dword_687294` (game) and `dword_687298` / `dword_68729C` (front end): camera height over a
/// ground reflection where it fades out (+2216) / starts fading (+2220)
pub const REFL_HEIGHT: [f32; 2] = [f32::from_bits(0x4120_0000), f32::from_bits(0x40A0_0000)];
pub const REFL_HEIGHT_FE: [f32; 2] = [f32::from_bits(0x4080_0000), f32::from_bits(0x3F80_0000)];
/// `dword_6872C4` / `dword_6872C8`: half widths of the reflection strip at the light / camera end
pub const REFL_WIDTH: [f32; 2] = [f32::from_bits(0x4040_0000), f32::from_bits(0x4040_0000)];
/// `dword_687330`: width scale per reflection texture (slot 0 = `dword_687358` 0.25 in the front end)
pub const REFL_WIDTH_SCALE: [f32; 4] = [1.0; 4];
pub const REFL_WIDTH_SCALE_FE0: f32 = f32::from_bits(0x3E80_0000);
/// `dword_687378`: length of the ground ray under a light
pub const REFL_RAY: f32 = f32::from_bits(0x4248_0000);
/// `dword_687374`: drop of a light whose ray misses
pub const REFL_MISS_DROP: f32 = f32::from_bits(0x4120_0000);
/// `flt_68737C`: the strip's camera end stays this far below the camera
pub const REFL_CAMERA_DROP: f32 = f32::from_bits(0x3F80_0000);
/// `dword_6872A0`: intensity -> engine colour scale
pub const REFL_COLOUR_SCALE: f32 = f32::from_bits(0x4300_0000);

/// CRim 0x000452C0
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (b - a) * t + a
}

/// TrackFX.cpp 0x00149970 — the near flare pair of a light inside the streak depth: two quads at `p` turned along the
/// light's screen direction (x = `r88` · d, y = `up` · d), the second mirrored left / right. `r88` = VsBaseCamera
/// +88 (up × dir = bevy's left); view -> world goes through the transposed view matrix (camera +8, 0x002ABD60 /
/// 0x002ABF70), whose x row is −(+88), so the first quad points to the mirrored side (original, kept).
/// Returns the two 4-vertex strips (empty when the light is on the view axis: 1/0 in the original).
pub fn near_flare(p: Vec3, d: Vec3, r88: Vec3, up: Vec3, fwd: Vec3, uvs: [Vec2; 4]) -> Vec<[EffectVertex; 4]> {
    let x = r88.dot(d);
    let y = up.dot(d);
    let inv = 1.0 / (x * x + y * y).sqrt();
    if !inv.is_finite() {
        return Vec::new();
    }
    let mut n = Vec2::new(x * inv, y * inv);
    let mut s = Vec2::new(-n.y, n.x);
    let t = n.x.abs();
    let sizes = [lerp(NEAR_FLARE_SIZE[0], NEAR_FLARE_SIZE_SIDE, t), lerp(NEAR_FLARE_SIZE[1], NEAR_FLARE_SIZE_SIDE, t)];
    let world = |v: Vec2| -r88 * v.x + up * v.y;
    let mut out = Vec::with_capacity(2);
    for k in 0..2 {
        if k == 1 {
            n.x = -n.x;
            s.y = -s.y;
        }
        let a = world(n) * sizes[k];
        let b = world(s) * sizes[k];
        let g = NEAR_FLARE_GREY[k];
        let colour = visos_colour([FLARE_ALPHA, g, g, g]);
        let c = p - (b - a) * 0.5 - fwd * FLARE_PULL;
        let q = [c, c + b, c - a, c - a + b];
        out.push([0, 1, 2, 3].map(|i| EffectVertex::new(q[i], colour, uvs[i])));
    }
    out
}

/// the "FLARES" block (0x0014D140); without one, the init defaults (0x00147DC0 / `off_6872F4`, `off_6872FC`)
#[derive(Clone, Debug, PartialEq)]
pub struct TrackFxSetup {
    /// `byte_6872A4` = clamp(Flare Brightness * 128, 0, 255)
    pub brightness: u8,
    /// flare textures and their sizes (`flt_687308`)
    pub flares: Vec<(String, f32)>,
    pub reflections: Vec<String>,
}

impl Default for TrackFxSetup {
    fn default() -> Self {
        Self {
            brightness: 0x80,
            flares: ["lightflare_orange", "lightflare_blue", "lightflare_orange_ray01"].map(|n| (n.to_string(), 10.0)).to_vec(),
            reflections: vec!["spec3".into(), "glowtest".into()],
        }
    }
}

impl TrackFxSetup {
    /// Setup.txt: `"FLARES"`, brightness, texture count, (name, size) each, reflection count, names
    pub fn parse(text: &str) -> Self {
        let toks = setup_tokens(text);
        let Some(i) = toks.iter().position(|t| matches!(t, SetupToken::Str(s) if s == "FLARES")) else {
            return Self::default();
        };
        let mut it = toks[i + 1..].iter();
        let mut num = || match it.next() {
            Some(SetupToken::Values(v)) => v.first().copied().unwrap_or(0.0),
            _ => 0.0,
        };
        let brightness = (num() * 128.0).clamp(0.0, 255.0) as u8;
        let rest = &toks[i + 2..];
        let mut k = 0;
        let mut next = || {
            k += 1;
            rest.get(k - 1)
        };
        let read_num =|t: Option<&SetupToken>| match t {
            Some(SetupToken::Values(v)) => v.first().copied().unwrap_or(0.0),
            _ => 0.0,
        };
        let read_str = |t: Option<&SetupToken>| match t {
            Some(SetupToken::Str(s)) => s.clone(),
            _ => String::new(),
        };
        let nf = (read_num(next()) as usize).min(MAX_FLARE_TEXTURES - 1);
        let flares = (0..nf).map(|_| (read_str(next()), read_num(next()))).collect();
        let nr = (read_num(next()) as usize).min(MAX_FLARE_TEXTURES - 1);
        let reflections = (0..nr).map(|_| read_str(next())).collect();
        Self { brightness, flares, reflections }
    }
}

/// GameDataLoading.cpp 0x0009D860 — flare and reflection numbers from a light name: from the second-last `_`,
/// the first number after a `_` is the flare, the second the reflection (`w` instead = 99, no reflection)
pub fn light_name_ids(name: &str) -> (i32, i32) {
    let b = name.as_bytes();
    let (mut flare, mut refl) = (0, 0);
    let mut i = b.len();
    let mut seen = 0;
    while i > 0 {
        if b.get(i) == Some(&b'_') {
            seen += 1;
            if seen == 2 {
                break;
            }
        }
        i -= 1;
    }
    let atoi = |s: &[u8]| s.iter().take_while(|c| c.is_ascii_digit()).fold(0i32, |a, &c| a.wrapping_mul(10).wrapping_add((c - b'0') as i32));
    let (mut state, mut next) = (0, 1);
    let mut j = i;
    while j < b.len() && j < 256 {
        let c = b[j];
        match state {
            0 => {
                let n = b.get(j + 1).copied().unwrap_or(0);
                if c == b'_' && (n.is_ascii_digit() || n == b'W' || n == b'w') {
                    state = next;
                }
            }
            1 => {
                if c.is_ascii_digit() {
                    flare = atoi(&b[j..]);
                    state = 0;
                    next = 2;
                }
            }
            _ => {
                if c.is_ascii_digit() {
                    refl = atoi(&b[j..]);
                    return (flare, refl);
                }
                if c == b'w' || c == b'W' {
                    return (flare, 99);
                }
            }
        }
        j += 1;
    }
    (flare, refl)
}

/// TrackFX.cpp 0x00149480 — one road reflection strip: from the reflection point `p` towards the camera on the plane of `p`
/// (at most `REFL_CAMERA_DROP` below the camera), half widths `w` at the two ends; colour = light colour ·
/// intensity · 128 in engine format (CatsEyeRender 0x0011DE40)
pub fn reflection_strip(p: Vec3, cam: Vec3, intensity: f32, colour: Vec3, w: [f32; 2]) -> [EffectVertex; 4] {
    let s = intensity * REFL_COLOUR_SCALE;
    let r = (colour.x * s) as i32 as u32;
    let g = (colour.y * s) as i32 as u32;
    let b = (colour.z * s) as i32 as u32;
    let argb = crate::effect_draw::engine_colour((b << 16) | r | (g << 8) | 0xFF00_0000);
    let mut c = cam;
    c.y = p.y;
    if c.y > cam.y - REFL_CAMERA_DROP {
        c.y = cam.y - REFL_CAMERA_DROP;
    }
    let dir = (c - p).normalize();
    let side = Vec3::Y.cross(dir);
    [
        EffectVertex::new(p - side * w[0], argb, Vec2::ZERO),
        EffectVertex::new(p + side * w[0], argb, Vec2::X),
        EffectVertex::new(c - side * w[1], argb, Vec2::Y),
        EffectVertex::new(c + side * w[1], argb, Vec2::ONE),
    ]
}

/// `dword_6872CC` / `dword_6872D0`: headlight road reflections of a car facing the camera fade from 40 to 80
/// (depth beyond the near plane); `dword_6872D4` / `dword_6872DC` from 15 to 50 for a car seen from behind
pub const HEADREF_FRONT: [f32; 2] = [f32::from_bits(0x42A0_0000), f32::from_bits(0x4220_0000)];
pub const HEADREF_REAR: [f32; 2] = [f32::from_bits(0x4248_0000), f32::from_bits(0x4170_0000)];
/// `dword_6872D8`: brake level scale of the rear reflections
pub const HEADREF_BRAKE: f32 = f32::from_bits(0x4000_0000);
/// `dword_687394`: view alignment where the reflection starts
pub const HEADREF_DOT: f32 = f32::from_bits(0x3F00_0000);
/// `dword_687390`: the reflection point sits this far below the head locator
pub const HEADREF_DROP: f32 = f32::from_bits(0x3F19_999A);
/// `flt_6873A8`: the camera end stays this far below the camera
pub const HEADREF_CAMERA_DROP: f32 = f32::from_bits(0x3E80_0000);
/// `dword_6873A4` / `dword_6873A0`: strip length = distance · intensity + 1, at least 2
pub const HEADREF_LEN_ADD: f32 = f32::from_bits(0x3F80_0000);
pub const HEADREF_LEN_MIN: f32 = f32::from_bits(0x4000_0000);
/// `flt_687398` / `flt_68739C`: half widths at the light / camera end
pub const HEADREF_WIDTH: [f32; 2] = [f32::from_bits(0x3E99_999A), f32::from_bits(0x3E99_999A)];
/// `unk_7374AC` (255, 255, 232) headlights, `unk_73749C` (255, 0, 0) rear (static locals of 0x0014BE40)
pub const HEADREF_COLOUR: [f32; 3] = [f32::from_bits(0x437F_0000), f32::from_bits(0x437F_0000), f32::from_bits(0x4368_0000)];
pub const HEADREF_COLOUR_REAR: [f32; 3] = [f32::from_bits(0x437F_0000), 0.0, 0.0];
/// `flt_68738C` / `dword_687388` (both beams), `flt_687384` / `dword_687380` (one beam): push along the car's z
/// for an in-car view, drop below the locator
pub const BEAM_INCAR_PUSH: [f32; 2] = [f32::from_bits(0x3F00_0000), f32::from_bits(0x3F00_0000)];
pub const BEAM_DROP: [f32; 2] = [f32::from_bits(0x3EB3_3333), f32::from_bits(0x3EB3_3333)];
/// `dword_6873B4` / `dword_6873B8`: beam length outside / inside the car
pub const BEAM_LEN: [f32; 2] = [f32::from_bits(0x4110_0000), f32::from_bits(0x4140_0000)];
/// `flt_6873AC` / `flt_6873B0`: beam half widths at the car / far end
pub const BEAM_WIDTH: [f32; 2] = [f32::from_bits(0x4090_0000), f32::from_bits(0x4090_0000)];
/// `dword_687284`: depth bias of hbZT / hb2zt / hb2
pub const BEAM_DEPTH_BIAS: i16 = 100;

/// `_ftol` of a double product, low byte (sub_3DDFA8 into a byte)
fn ftol_u8(v: f64) -> u8 {
    v as i32 as u8
}

/// TrackFX's view of a car (CarRender data, 2352 bytes per car) for 0x0014B030
#[derive(Clone, Copy, Debug)]
pub struct BeamCar {
    /// +0 the car matrix (model space: x lateral, y up, z nose)
    pub matrix: bevy::math::Mat4,
    /// +768 the chassis' l_head locator (z negated)
    pub head: Vec3,
    /// +1828 brake light level
    pub brake: f32,
    /// +1820 headlight level (1 from the car init; only the neon draw 0x00111B20 moves it)
    pub level: f32,
    /// +2250 light damage (1/2 left/right headlight, 4/8 left/right brake)
    pub damage: u8,
    /// +2251 not drawn this frame (CarRender 0x001127F0: no active vehicle, or the sphere is outside the view)
    pub culled: bool,
    /// +2328 bit 4: the camera sits in this car (0x001127F0, camera modes 1/2)
    pub inside: bool,
}

/// TrackFX.cpp 0x0014BE40 — headlight / tail light road reflection: a strip from `b` towards the camera on the plane of `b`
/// (at most 0.25 below the camera), length distance · `a4` + 1 (≥ 2), light colour · `a4` at `b` (`a6` = 0
/// headlights, else red · min(a4 · a6, 1)), black (bss `unk_737498`) at the far end
pub fn headlight_reflection(b: Vec3, cam: Vec3, a4: f32, a6: f32) -> [EffectVertex; 4] {
    let mut c = cam;
    c.y = b.y;
    let lim = cam.y - HEADREF_CAMERA_DROP;
    if c.y > lim {
        c.y = lim;
    }
    let mut v = c - b;
    let len = v.length();
    // CSpoke 0x00047840: v /= len
    v = Vec3::new(v.x / len, v.y / len, v.z / len);
    let mut k = len * a4 + HEADREF_LEN_ADD;
    if HEADREF_LEN_MIN > k {
        k = HEADREF_LEN_MIN;
    }
    let c = b + v * k;
    let side = Vec3::Y.cross(v);
    let (r, g, bl) = if a6 == 0.0 {
        let a = a4 as f64;
        (ftol_u8(HEADREF_COLOUR[0] as f64 * a), ftol_u8(HEADREF_COLOUR[1] as f64 * a), ftol_u8(HEADREF_COLOUR[2] as f64 * a))
    } else {
        let a = (a4 * a6).min(1.0) as f64;
        (ftol_u8(HEADREF_COLOUR_REAR[0] as f64 * a), ftol_u8(HEADREF_COLOUR_REAR[1] as f64 * a), ftol_u8(HEADREF_COLOUR_REAR[2] as f64 * a))
    };
    let col = crate::effect_draw::engine_colour(((bl as u32) << 16) | ((g as u32) << 8) | r as u32 | 0x8000_0000);
    let far = crate::effect_draw::engine_colour(0);
    [
        EffectVertex::new(b - side * HEADREF_WIDTH[0], col, Vec2::ZERO),
        EffectVertex::new(b + side * HEADREF_WIDTH[0], col, Vec2::X),
        EffectVertex::new(c - side * HEADREF_WIDTH[1], far, Vec2::Y),
        EffectVertex::new(c + side * HEADREF_WIDTH[1], far, Vec2::ONE),
    ]
}

/// TrackFX.cpp 0x0014C430 — one flat beam quad from `p` along `dir` (the car's z), only while the headlight level +1820 > 0:
/// 9 long, 4.5 half wide, VISOS(0x20, 48, 56, 56)·level fading to black; in-car 12 long, VISOS(0x80, 112, 128,
/// 128)·level to VISOS(0x30, 48, 48, 48)·level. `flip` mirrors u (one headlight: the left one is broken)
pub fn headlight_beam(p: Vec3, dir: Vec3, car: &BeamCar, flip: bool) -> Option<[EffectVertex; 4]> {
    if !(car.level > 0.0) {
        return None;
    }
    let k = car.level as f64;
    let (len, near, far) = if car.inside {
        (
            BEAM_LEN[1],
            visos_colour([0x80, ftol_u8(112.0 * k), ftol_u8(128.0 * k), ftol_u8(128.0 * k)]),
            visos_colour([0x30, ftol_u8(48.0 * k), ftol_u8(48.0 * k), ftol_u8(48.0 * k)]),
        )
    } else {
        (BEAM_LEN[0], visos_colour([0x20, ftol_u8(48.0 * k), ftol_u8(56.0 * k), ftol_u8(56.0 * k)]), visos_colour([0, 0, 0, 0]))
    };
    let e = p + dir * len;
    let side = Vec3::Y.cross(dir);
    let uv = if flip { [Vec2::X, Vec2::ZERO, Vec2::ONE, Vec2::Y] } else { [Vec2::ZERO, Vec2::X, Vec2::Y, Vec2::ONE] };
    Some([
        EffectVertex::new(p - side * BEAM_WIDTH[0], near, uv[0]),
        EffectVertex::new(p + side * BEAM_WIDTH[0], near, uv[1]),
        EffectVertex::new(e - side * BEAM_WIDTH[1], far, uv[2]),
        EffectVertex::new(e + side * BEAM_WIDTH[1], far, uv[3]),
    ])
}

/// the headlight shaders of the init 0x00147DC0 (`byte_687283` = 1: blend 13 = rgb += src · dest alpha with stencil
/// EQUAL ref 1 mask 1, i.e. only over glow-stencil roads, see `reflection_shader`)
pub mod beam_shader {
    use super::*;
    /// blend 13; `VELOCITY_LIGHTREF_DEBUG=1` draws these with blend 12 (no glow mask), as the road reflections
    fn b13() -> u32 {
        if std::env::var_os("VELOCITY_LIGHTREF_DEBUG").is_some() { 12 } else { 13 }
    }
    /// +2128 "hb": `Hdlights2`, blend 13, no z
    pub fn hb() -> EffectShader {
        EffectShader::new(EffectTexture::Misc("Hdlights2".into()), b13()).z(false, false)
    }
    /// +2132 "hb2": blend 12, no z, bias 100
    pub fn hb2() -> EffectShader {
        EffectShader::new(EffectTexture::Misc("Hdlights2".into()), 12).z(false, false).bias(BEAM_DEPTH_BIAS)
    }
    /// +2136 "hbZT": blend 13, z test, bias 100
    pub fn hb_zt() -> EffectShader {
        EffectShader::new(EffectTexture::Misc("Hdlights2".into()), b13()).z(true, false).bias(BEAM_DEPTH_BIAS)
    }
    /// +2140 "hb2zt": blend 12, z test, bias 100
    pub fn hb2_zt() -> EffectShader {
        EffectShader::new(EffectTexture::Misc("Hdlights2".into()), 12).z(true, false).bias(BEAM_DEPTH_BIAS)
    }
    /// +2148 "headlightRef": `headlight02`, blend 13, no z
    pub fn headlight_ref() -> EffectShader {
        EffectShader::new(EffectTexture::Misc("headlight02".into()), b13()).z(false, false)
    }
    /// +2156 "OneHeadlight": `OneHdLgt`, blend 13, no z
    pub fn one_headlight() -> EffectShader {
        EffectShader::new(EffectTexture::Misc("OneHdLgt".into()), b13()).z(false, false)
    }
}

/// TrackFX.cpp 0x0014B030 (GameManager 0x000B93C1 after the road reflections, render flag bit 2) — per car in CarRender order:
/// headlight / tail light road reflections ("headlightRef"), both beams of cars with intact headlights (car 0
/// through "hb" + "hb2" without z, the rest "hbZT" + "hb2zt"; skipped for cars whose `zone_mask` bit is clear),
/// the single beam of cars with one broken headlight ("OneHeadlight"). `near_plane` = camera +236 (plane 4 of
/// 0x0005C9C0), `camera_mode` = view 0's mode (3 shows the beams of culled cars). The neon pass 4 is
/// `render_neons`.
#[allow(clippy::too_many_arguments)]
pub fn render_headlights(
    cars: &[BeamCar],
    cam: Vec3,
    fwd: Vec3,
    near_plane: f32,
    camera_mode: i32,
    zone_mask: i32,
    front_end: bool,
    draw: &mut EffectDraw,
) {
    // plane 4: normal = forward, through the near point
    let plane_w = -fwd.dot(cam + fwd * near_plane);
    if cars.iter().any(|c| !c.culled) {
        let sh = beam_shader::headlight_ref();
        for c in cars.iter().filter(|c| !c.culled) {
            let pos = c.matrix.w_axis.truncate();
            let d = fwd.dot(pos) + plane_w;
            if !(HEADREF_FRONT[0] > d) {
                continue;
            }
            let z = c.matrix.z_axis.truncate();
            let dir = (pos - cam).normalize();
            let (mut left, mut right) = (true, true);
            let mut dot = -z.dot(dir);
            let (far, near, mut rear);
            if dot <= 0.0 {
                (far, near, rear) = (HEADREF_REAR[0], HEADREF_REAR[1], 1.0f32);
                dot = 0.0 - dot;
                left &= c.damage & 4 == 0;
                right &= c.damage & 8 == 0;
            } else {
                (far, near, rear) = (HEADREF_FRONT[0], HEADREF_FRONT[1], 0.0f32);
                left &= c.damage & 1 == 0;
                right &= c.damage & 2 == 0;
            }
            let mut fade = 1.0;
            if d > near {
                fade = 1.0 - (d - near) / (far - near);
            }
            let mut a = (dot - HEADREF_DOT) / (1.0 - HEADREF_DOT);
            let mut u = dir.dot(Vec3::Y) + 1.0;
            u *= u;
            u *= u;
            a *= u;
            a *= fade;
            if a > 0.0 {
                if a > 1.0 {
                    a = 1.0;
                }
                // the tail lights use the head locator mirrored in z (original)
                let mut loc = c.head;
                if rear != 0.0 {
                    loc.z = -loc.z;
                    rear = c.brake * HEADREF_BRAKE + rear;
                }
                loc.y -= HEADREF_DROP;
                let p = c.matrix.transform_point3(loc);
                if left {
                    draw.strip(&sh, &headlight_reflection(p, cam, a, rear));
                }
                loc.x = -loc.x;
                let p = c.matrix.transform_point3(loc);
                if right {
                    draw.strip(&sh, &headlight_reflection(p, cam, a, rear));
                }
            }
        }
    }
    let mode3 = camera_mode == 3;
    let shown = |c: &BeamCar| !c.culled || c.inside || mode3;
    let (mut one, mut two) = (0, 0);
    for c in cars.iter().filter(|c| shown(c)) {
        match (c.damage & 1 != 0, c.damage & 2 != 0) {
            (true, false) | (false, true) => one += 1,
            (false, false) => two += 1,
            _ => {}
        }
    }
    let beam_at = |c: &BeamCar, k: usize| {
        let pos = c.matrix.w_axis.truncate();
        let z = c.matrix.z_axis.truncate();
        let mut p = if c.inside { pos + z * (c.head.z + BEAM_INCAR_PUSH[k]) } else { pos + z * c.head.z };
        p.y -= BEAM_DROP[k];
        (p, z)
    };
    if two > 0 {
        let flush_at: i32 = if front_end { -1 } else { 0 };
        let mut stream: Vec<[EffectVertex; 4]> = Vec::new();
        let flush = |stream: &mut Vec<[EffectVertex; 4]>, a: EffectShader, b: EffectShader, draw: &mut EffectDraw| {
            for sh in [a, b] {
                for q in stream.iter() {
                    draw.strip(&sh, q);
                }
            }
            stream.clear();
        };
        for (i, c) in cars.iter().enumerate() {
            if i >= 32 || zone_mask & (1 << i) == 0 {
                continue;
            }
            if shown(c) {
                if c.damage & 3 != 0 {
                    continue;
                }
                let (p, z) = beam_at(c, 0);
                stream.extend(headlight_beam(p, z, c, false));
            }
            if i as i32 == flush_at {
                flush(&mut stream, beam_shader::hb(), beam_shader::hb2(), draw);
            }
        }
        flush(&mut stream, beam_shader::hb_zt(), beam_shader::hb2_zt(), draw);
    }
    if one > 0 {
        let sh = beam_shader::one_headlight();
        for c in cars {
            let (l, r) = (c.damage & 1 != 0, c.damage & 2 != 0);
            if (!c.culled || c.inside) && (!l || !r) && (l || r) {
                let (p, z) = beam_at(c, 1);
                if let Some(q) = headlight_beam(p, z, c, l) {
                    draw.strip(&sh, &q);
                }
            }
        }
    }
}

/// what the game tells TrackFX about the view (core fills it; default = outside every car, camera mode 0)
#[derive(bevy::prelude::Resource, Clone, Copy, Debug, Default)]
pub struct HeadlightView {
    /// view 0's camera mode (VsCameraManager 0x0005FFB0)
    pub camera_mode: i32,
    /// the `CarRender.request` of the car the camera sits in (CarRender 0x001127F0 sets its +2328 bit 4)
    pub inside: Option<u64>,
}

/// `HeadlightView` of the views after view 0 (CarRender 0x001127F0 runs per view with that view's player and camera
/// mode): `player1` = player 1's view while split, `mirror` = the mirror view (player 0, mode 2: the race car's
/// mirror camera VehicleEntity 0x001C57C0). Core fills them; None = `HeadlightViews::for_view`'s stand-ins.
#[derive(bevy::prelude::Resource, Clone, Copy, Debug, Default)]
pub struct HeadlightViews {
    pub player1: Option<HeadlightView>,
    pub mirror: Option<HeadlightView>,
}

impl HeadlightViews {
    /// the view state of `v` (`view` = view 0's): player 1 without data = outside every car (mode 0); the mirror
    /// without data = mode 2 inside view 0's car (it is only drawn in the static bumper / in-car views, where its
    /// camera sits in player 0's car)
    pub fn for_view(&self, view: &HeadlightView, v: &crate::view_render::ViewCam) -> HeadlightView {
        // the mirror is the only player-0 view tagged `MIRROR_VIEW_TAG` (player 1's view carries the same tag value)
        if v.player == 0 && v.tag == Some(crate::view_render::MIRROR_VIEW_TAG) {
            return self.mirror_view(view);
        }
        if v.player == 0 {
            *view
        } else {
            self.player1_view()
        }
    }

    pub fn mirror_view(&self, view: &HeadlightView) -> HeadlightView {
        self.mirror.unwrap_or(HeadlightView { camera_mode: 2, inside: view.inside })
    }

    pub fn player1_view(&self) -> HeadlightView {
        self.player1.unwrap_or_default()
    }
}

/// TrackFX data on a car root (CarRender data +768 and the LOD 0 chassis sphere for the cull of 0x001127F0);
/// inserted by `spawn_car` when the chassis has its head locators. The level +1820 is `CarShadow.level`.
#[derive(bevy::prelude::Component, Clone, Copy, Debug)]
pub struct CarHeadBeams {
    pub head: Vec3,
    /// model-space bounding sphere (xyz centre, w radius)
    pub sphere: Vec4,
}

impl CarHeadBeams {
    pub fn new(head: Vec3, sphere: Vec4) -> Self {
        Self { head, sphere }
    }
}

/// light info, 32 bytes at TrackFX +2204
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TrackFxLight {
    /// +0 the reflection point (0x00149020: on the ground, mirrored below it for reflection ids > 0)
    pub pos: Vec3,
    /// +28 reflection id, +29 flare id (-1 none), +30 light streak slot, +31 flags (2 = reflects)
    pub reflection: i8,
    pub flare: i8,
    pub streak_slot: i8,
    pub flags: u8,
}

#[derive(Clone, Debug)]
pub struct TrackFx {
    /// +0 the light streak manager (its +0 is the near / far flare split)
    pub streaks: LightStreakManager,
    pub setup: TrackFxSetup,
    /// +2204 / +2240
    pub lights: Vec<TrackFxLight>,
    /// +2212: reflection points set up (0x00149020)
    pub reflections_set_up: bool,
    /// +2216 / +2220: camera heights where a ground reflection is gone / starts fading
    pub refl_height: [f32; 2],
    /// `dword_687330`
    pub refl_width_scale: [f32; 4],
}

impl TrackFx {
    /// TrackFX.cpp 0x00148CF0 (AllocPointLights) + 0x0014AEC0 per light, `names` = the second PtLights name of each light
    pub fn new(setup: TrackFxSetup, names: &[String]) -> Self {
        let nf = setup.flares.len() as i32;
        let nr = setup.reflections.len() as i32;
        let lights = names
            .iter()
            .map(|n| {
                let (mut flare, mut refl) = light_name_ids(n);
                let mut l = TrackFxLight { pos: Vec3::ZERO, reflection: 0, flare: 0, streak_slot: -1, flags: 0 };
                if flare > 10 {
                    flare = 1;
                }
                if refl == 99 {
                    refl = 0;
                    l.flags &= !2;
                } else if refl > 10 {
                    refl = 1;
                }
                l.flare = (flare - 1) as i8;
                if l.flare as i32 >= nf {
                    log::warn!("Attempting to set LightFlareID greater than number of LightFlare textures");
                    l.flare = (nf - 1) as i8;
                }
                l.reflection = (refl - 1) as i8;
                if l.reflection as i32 >= nr {
                    log::warn!("Attempting to set LightRefTex greater than number of LightRef textures");
                    l.reflection = (nr - 1) as i8;
                }
                l
            })
            .collect();
        Self {
            streaks: LightStreakManager::new(true),
            setup,
            lights,
            reflections_set_up: false,
            refl_height: REFL_HEIGHT,
            refl_width_scale: REFL_WIDTH_SCALE,
        }
    }

    /// TrackFX.cpp 0x00149020 (PhysicsManager::Init, else the first 0x0014A010) — reflection points of the lights `points`.
    /// In game every light is dropped onto the track by a 50 m ray (MaterialProperties 0x0019D310 → RayCast flags
    /// 17; `ray(from, to)` = hit fraction); the front end (+2232) takes the lights as they are. In game the
    /// caller then writes each light's resulting y into the point light's colour w (0x0014DEC0, `ground_heights`)
    /// and runs `vs_render::reduce_light_overlaps` (0x0005A040).
    pub fn setup_reflections(&mut self, points: &[Vec3], front_end: bool, ray: &mut dyn FnMut(Vec3, Vec3) -> Option<f32>) {
        self.reflections_set_up = true;
        let nf = self.setup.flares.len() as i32;
        if front_end {
            self.refl_height = REFL_HEIGHT_FE;
            self.refl_width_scale[0] = REFL_WIDTH_SCALE_FE0;
            for (l, &p) in self.lights.iter_mut().zip(points) {
                l.flags = (2 * (l.reflection >= 0) as u8) | (l.flags & 0xFD);
                l.pos = p;
            }
            return;
        }
        self.refl_height = REFL_HEIGHT;
        self.refl_width_scale[0] = 1.0;
        for (l, &p) in self.lights.iter_mut().zip(points) {
            if l.flare as i32 >= nf {
                l.flare = 0;
            }
            l.pos = p;
            let end = p - Vec3::Y * REFL_RAY;
            match ray(p, end) {
                Some(t) => {
                    l.pos.y -= t * REFL_RAY;
                    l.flags |= 2;
                    if l.reflection > 0 {
                        l.pos.y -= t * REFL_RAY;
                    }
                }
                None => {
                    l.reflection = -1;
                    l.pos.y -= REFL_MISS_DROP;
                }
            }
            if l.reflection == -1 {
                l.flags &= !2;
            }
        }
    }

    /// TrackFX.cpp 0x00149020 game branch: 0x0014DEC0 sets point light i's colour w to light info i's y, then VsRender
    /// 0x0005A040 shrinks overlapping radii (that distance uses the colour w as height)
    pub fn ground_heights(&self, points: &mut [crate::vs_render::PointLight]) {
        for (p, l) in points.iter_mut().zip(&self.lights) {
            p.colour.w = l.pos.y;
        }
        crate::vs_render::reduce_light_overlaps(points);
    }

    /// shader +2004 + 4 * reflection ("lightref", 0x00147DC0; `byte_687283` = 1): blend 13 (rgb += src · dest
    /// alpha), no z test or write, stencil EQUAL ref 1 mask 1 write mask 0 = only over glow-stencil surfaces
    /// (roads with an `eMS_Stencil` record). The stencil needs no state here: on glow views those surfaces are the
    /// only ones with dest alpha, which blend 13 multiplies by. 0x002A1680(−1) / 0x002A0DD0(1) / 0x002A17D0(1)
    /// unidentified.
    pub fn reflection_shader(&self, refl: usize) -> EffectShader {
        let tex = self.setup.reflections.get(refl).cloned().unwrap_or_default();
        // VELOCITY_LIGHTREF_DEBUG=1: plain additive (blend 12, the flares'), shows the strips without the glow mask
        let blend = if std::env::var_os("VELOCITY_LIGHTREF_DEBUG").is_some() { 12 } else { 13 };
        EffectShader::new(EffectTexture::Misc(tex), blend).z(false, false)
    }

    /// TrackFX.cpp 0x0014A010 — road reflections (GameManager 0x000B93A9, after the cars); `colours` = the point lights'
    /// colours, `cam` / `fwd` = camera +72 / +120
    pub fn render_reflections(&self, colours: &[Vec3], cam: Vec3, fwd: Vec3, draw: &mut EffectDraw) {
        let far2 = REFL_FAR * REFL_FAR;
        let near2 = REFL_NEAR * REFL_NEAR;
        for (i, l) in self.lights.iter().enumerate() {
            if l.flags & 2 == 0 {
                continue;
            }
            let d = l.pos - cam;
            let mut d2 = d.length_squared();
            if far2 <= d2 || fwd.dot(d) < 0.0 {
                continue;
            }
            let mut intensity = if near2 <= d2 {
                d2 = d2.sqrt();
                1.0 - (d2 - REFL_NEAR) / (REFL_FAR - REFL_NEAR)
            } else {
                1.0
            };
            let h = if l.reflection == 0 { cam.y - l.pos.y } else { 0.0 };
            let [max_h, min_h] = self.refl_height;
            if max_h <= h {
                continue;
            }
            if h > min_h {
                intensity *= 1.0 - (h - min_h) / (max_h - min_h);
            }
            if l.reflection < 0 || l.reflection as usize >= self.setup.reflections.len() {
                log::warn!("LightInfo->m_LightReflectionID >=0 && LightInfo->m_LightReflectionID < m_nLightRefTex");
                continue;
            }
            let k = l.reflection as usize;
            let scale = self.refl_width_scale.get(k).copied().unwrap_or(0.0);
            let w = [REFL_WIDTH[0] * scale, REFL_WIDTH[1] * scale];
            let colour = colours.get(i).copied().unwrap_or(Vec3::ZERO);
            draw.strip(&self.reflection_shader(k), &reflection_strip(l.pos, cam, intensity, colour, w));
        }
    }

    /// shader +2084 + 4 * flare ("lightflare", 0x00147DC0): blend 12, z test, no z write, depth bias 0x20
    pub fn flare_shader(&self, flare: usize) -> EffectShader {
        let tex = self.setup.flares.get(flare).map(|f| f.0.clone()).unwrap_or_default();
        EffectShader::new(EffectTexture::Misc(tex), 12).z(true, false).bias(FLARE_DEPTH_BIAS)
    }

    /// TrackFX.cpp 0x0014A5B0 — flares and light streaks for the point lights `points`; `visible` = the sphere test of the
    /// camera (0x0005E070), `project` = 0x0015EC80, `streak` draws a streak slot. `r88` = VsBaseCamera +88
    /// (`m_right_RENDER` = up × dir, bevy's left), `up` +104, `fwd` +120.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        points: &[Vec3],
        cam: Vec3,
        fwd: Vec3,
        r88: Vec3,
        up: Vec3,
        dt: f32,
        visible: impl Fn(Vec3, f32) -> bool,
        project: impl Fn(Vec3) -> Vec4,
        draw: &mut EffectDraw,
        mut streak: impl FnMut(&crate::light_streak::LightStreakSlot, f32, &mut EffectDraw),
    ) {
        let grey = self.setup.brightness;
        let colour = visos_colour([FLARE_ALPHA, grey, grey, grey]);
        let mut records: Vec<StreakLight> = Vec::with_capacity(MAX_LIGHTSTREAK_LIGHTS);
        for (i, info) in self.lights.iter().enumerate() {
            if info.flare < 0 {
                continue;
            }
            let Some(&p) = points.get(i) else { break };
            if !visible(p, FLARE_CULL_RADIUS) {
                continue;
            }
            let size = self.setup.flares.get(info.flare as usize).map(|f| f.1).unwrap_or(0.0);
            let a = r88 * size;
            let b = up * size;
            let depth = fwd.dot(p - cam);
            let shader = self.flare_shader(info.flare as usize);
            if self.streaks.max_depth <= depth {
                let c = p - (a - b) * 0.5 - fwd * FLARE_PULL;
                let q = [(c, Vec2::ZERO), (c + a, Vec2::X), (c - b, Vec2::Y), (c - b + a, Vec2::ONE)];
                let v = q.map(|(p, uv)| EffectVertex::new(p, colour, uv));
                draw.strip(&shader, &v);
            } else {
                let slot = if records.len() >= MAX_LIGHTSTREAK_LIGHTS {
                    // 0x0012FA80: the farthest record, when farther than this light (the split drops to it)
                    let (k, far) = records.iter().enumerate().fold((None, -9_999_999.0f32), |(k, m), (j, r)| {
                        if r.depth > m { (Some(j), r.depth) } else { (k, m) }
                    });
                    self.streaks.max_depth = far;
                    if far <= depth { None } else { k }
                } else {
                    records.push(StreakLight::default());
                    Some(records.len() - 1)
                };
                if let Some(k) = slot {
                    records[k] = StreakLight { pos: p, depth, intensity: 1.0, light: i as i16, colour: info.flare as u8 };
                }
                // drawn into the same stream, i.e. with this light's flare shader
                for q in near_flare(p, p - cam, r88, up, fwd, [Vec2::ZERO, Vec2::X, Vec2::Y, Vec2::ONE]) {
                    draw.strip(&shader, &q);
                }
            }
        }
        let mut slot_of: Vec<i8> = self.lights.iter().map(|l| l.streak_slot).collect();
        self.streaks.update(&records, &mut slot_of, dt, project, &mut |s, w| streak(s, w, draw));
        for (l, s) in self.lights.iter_mut().zip(slot_of) {
            l.streak_slot = s;
        }
    }
}

/// the second quoted name of every light in PtLights.txt
pub fn ptlights_names(text: &str) -> Vec<String> {
    setup_tokens(text)
        .into_iter()
        .filter_map(|t| match t {
            SetupToken::Str(s) => Some(s),
            _ => None,
        })
        .skip(1)
        .step_by(2)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_ids() {
        assert_eq!(light_name_ids("LGT_Flare_Reflect_Light_Orange_000_1_1"), (1, 1));
        assert_eq!(light_name_ids("LGT_Reflect_Light_003_0_1"), (0, 1));
        assert_eq!(light_name_ids("LGT_Light_007_2_w"), (2, 99));
    }

    /// lights dropped by the ground ray (hit, mirrored, miss) and the fade / gating of 0x0014A010
    #[test]
    fn road_reflections() {
        let setup = TrackFxSetup::default();
        let names: Vec<String> = ["L_000_1_1", "L_001_1_2", "L_002_1_1", "L_003_1_w"].map(String::from).to_vec();
        let mut fx = TrackFx::new(setup, &names);
        let pts = [Vec3::new(0.0, 6.0, -20.0), Vec3::new(2.0, 6.0, -20.0), Vec3::new(0.0, 6.0, 20.0), Vec3::new(4.0, 6.0, -20.0)];
        // ground at y = 1: t = 5/50; light 2 misses
        let mut ray = |a: Vec3, _b: Vec3| (a.z < 0.0).then_some(0.1);
        fx.setup_reflections(&pts, false, &mut ray);
        assert!((fx.lights[0].pos.y - 1.0).abs() < 1e-5 && fx.lights[0].flags & 2 != 0);
        assert!((fx.lights[1].pos.y + 4.0).abs() < 1e-5, "reflection id 1: mirrored below the ground");
        assert_eq!((fx.lights[2].reflection, fx.lights[2].flags & 2), (-1, 0));
        assert!((fx.lights[2].pos.y + 4.0).abs() < 1e-5);
        assert_eq!(fx.lights[3].flags & 2, 0, "w = no reflection");
        let mut draw = EffectDraw::default();
        let cols = [Vec3::ONE; 4];
        // camera 3 m above the ground point (between 5 and 10: no fade), looking down -z
        fx.render_reflections(&cols, Vec3::new(0.0, 4.0, 0.0), Vec3::NEG_Z, &mut draw);
        assert_eq!(draw.batches.iter().map(|b| b.1.len()).sum::<usize>(), 2 * 6);
        let s = reflection_strip(fx.lights[0].pos, Vec3::new(0.0, 4.0, 0.0), 1.0, Vec3::ONE, [3.0, 3.0]);
        assert_eq!(s[0].argb, 0xFFFF_FFFF);
        assert_eq!(s[0].pos, Vec3::new(-3.0, 1.0, -20.0));
        assert_eq!(s[3].pos, Vec3::new(3.0, 1.0, 0.0));
        // 75 m away: half intensity
        let s = reflection_strip(Vec3::ZERO, Vec3::new(0.0, 4.0, 75.0), 0.5, Vec3::new(1.0, 0.5, 0.25), [3.0, 3.0]);
        assert_eq!(s[0].argb, 0xFF80_4020);
    }

    fn car_at(x: f32, damage: u8) -> BeamCar {
        // nose +z towards the camera at z = 20, 10 m ahead of the near plane region
        BeamCar {
            matrix: bevy::math::Mat4::from_translation(Vec3::new(x, 0.0, 0.0)),
            head: Vec3::new(0.7, 0.7, 2.0),
            brake: 0.0,
            level: 1.0,
            damage,
            culled: false,
            inside: false,
        }
    }

    /// TrackFX.cpp 0x0014BE40: strip from the light towards the camera, length d·a + 1, colours
    #[test]
    fn headlight_reflection_strip() {
        let b = Vec3::new(0.0, 0.1, 0.0);
        let cam = Vec3::new(0.0, 2.0, 10.0);
        let s = headlight_reflection(b, cam, 0.5, 0.0);
        // camera end on the plane of b: 10 m away, length 10·0.5 + 1 = 6
        assert!((s[2].pos - Vec3::new(-0.3, 0.1, 6.0)).length() < 1e-5, "{:?}", s[2].pos);
        assert!((s[0].pos - Vec3::new(-0.3, 0.1, 0.0)).length() < 1e-5);
        // 255·0.5 = 127 doubled = 254, 232·0.5 = 116 -> 232, alpha 0x80 -> 0xFF
        assert_eq!(s[0].argb, 0xFFFE_FEE8);
        assert_eq!(s[3].argb, 0);
        // rear: red · min(a·a6, 1)
        assert_eq!(headlight_reflection(b, cam, 0.5, 3.0)[1].argb, 0xFFFF_0000);
        assert_eq!(headlight_reflection(b, cam, 0.25, 1.0)[1].argb, 0xFF7E_0000);
        // short: at least 2 long
        let s = headlight_reflection(b, cam, 0.01, 0.0);
        assert!((s[2].pos.z - 2.0).abs() < 1e-5);
    }

    /// TrackFX.cpp 0x0014B030: shader routing of car 0 / others / one broken headlight / zone mask
    #[test]
    fn headlight_beams() {
        let cam = Vec3::new(0.0, 1.5, 20.0);
        let fwd = Vec3::NEG_Z;
        let cars = [car_at(0.0, 0), car_at(5.0, 0), car_at(-5.0, 1)];
        let mut draw = EffectDraw::default();
        render_headlights(&cars, cam, fwd, 0.5, 0, -1, false, &mut draw);
        let n = |sh: EffectShader| draw.batches.iter().find(|b| b.0 == sh).map_or(0, |b| b.1.len() / 6);
        assert_eq!(n(beam_shader::hb()), 1);
        assert_eq!(n(beam_shader::hb2()), 1);
        assert_eq!(n(beam_shader::hb_zt()), 1);
        assert_eq!(n(beam_shader::hb2_zt()), 1);
        assert_eq!(n(beam_shader::one_headlight()), 1);
        // 2 + 2 + 1 headlight reflections (the broken left one has none)
        assert_eq!(n(beam_shader::headlight_ref()), 5);
        // beam: 9 long along +z from z = 2, 4.5 half wide, 0.35 below the locator
        let b = draw.batches.iter().find(|b| b.0 == beam_shader::hb()).unwrap();
        let ys: Vec<f32> = b.1.iter().map(|v| v.pos.y).collect();
        assert!(ys.iter().all(|y| (y + 0.35).abs() < 1e-5), "{ys:?}");
        assert!(b.1.iter().any(|v| (v.pos - Vec3::new(4.5, -0.35, 11.0)).length() < 1e-4));
        assert_eq!(b.1[0].argb, visos_colour([0x20, 48, 56, 56]));
        // one headlight: left broken -> u mirrored
        let o = draw.batches.iter().find(|b| b.0 == beam_shader::one_headlight()).unwrap();
        assert_eq!(o.1[0].uv, Vec2::X);
        // zone mask bit 0 clear: car 0's beams skip the flush, the others still draw z tested
        let mut draw = EffectDraw::default();
        render_headlights(&cars, cam, fwd, 0.5, 0, !1, false, &mut draw);
        assert!(draw.batches.iter().all(|b| b.0 != beam_shader::hb()));
        assert_eq!(draw.batches.iter().find(|b| b.0 == beam_shader::hb_zt()).map_or(0, |b| b.1.len() / 6), 1);
        // past 80 m: no reflections
        let mut draw = EffectDraw::default();
        render_headlights(&cars, Vec3::new(0.0, 1.5, 100.0), fwd, 0.5, 0, -1, false, &mut draw);
        assert!(draw.batches.iter().all(|b| b.0 != beam_shader::headlight_ref()));
    }

    /// light up-right of the view centre, camera looking down -z: sizes and the mirrored first quad
    #[test]
    fn near_flare_pair() {
        let (r88, up, fwd) = (Vec3::NEG_X, Vec3::Y, Vec3::NEG_Z);
        let p = Vec3::new(3.0, 4.0, -10.0);
        let uv = [Vec2::ZERO, Vec2::X, Vec2::Y, Vec2::ONE];
        let q = near_flare(p, p, r88, up, fwd, uv);
        assert_eq!(q.len(), 2);
        // x = r88 . d = -3 -> n = (-0.6, 0.8), t = 0.6: sizes 10 - 0.6 = 9.4 and 5 + 2.4 = 7.4
        let a0 = q[0][0].pos - q[0][2].pos;
        assert!((a0.length() - 9.4).abs() < 1e-4);
        // view x row = -(+88) = +x: the first quad points to the mirrored (left) side
        assert!(a0.x < 0.0 && a0.y > 0.0);
        let a1 = q[1][0].pos - q[1][2].pos;
        assert!((a1.length() - 7.4).abs() < 1e-4 && a1.x > 0.0 && a1.y > 0.0);
        assert_eq!(q[0][0].argb, 0xFF80_8080);
        assert_eq!(q[1][0].argb, 0xFFFF_FFFF);
        // centred on the light (pulled 0.2 towards the camera)
        let c = (q[1][0].pos + q[1][3].pos) * 0.5;
        assert!((c - (p - fwd * FLARE_PULL)).length() < 1e-4);
        assert!(near_flare(p, Vec3::new(0.0, 0.0, -5.0), r88, up, fwd, uv).is_empty());
    }

    /// every track's Setup.txt FLARES block and PtLights names (positive control: counts match the lights)
    #[test]
    fn every_track_flares() {
        let dir = visos::file::game_dir().join("Tracks");
        let mut n = 0;
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let (Ok(s), Ok(p)) = (std::fs::read_to_string(e.path().join("Setup.txt")), std::fs::read_to_string(e.path().join("PtLights.txt"))) else {
                continue;
            };
            let setup = TrackFxSetup::parse(&s);
            assert!(!setup.flares.is_empty() && setup.flares.iter().all(|f| !f.0.is_empty() && f.1 > 0.0), "{:?}", e.path());
            let names = ptlights_names(&p);
            assert_eq!(names.len(), crate::vs_render::parse_ptlights_txt(&p).len(), "{:?}", e.path());
            let fx = TrackFx::new(setup, &names);
            assert!(fx.lights.iter().all(|l| (l.flare as i32) < fx.setup.flares.len() as i32));
            n += 1;
        }
        assert!(n > 0);
    }
}

/// the loaded track's TrackFX (rebuilt with each `TrackRoot`)
#[derive(bevy::prelude::Resource, Default)]
pub struct TrackFxState(pub Option<TrackFx>);

pub struct TrackFxPlugin;

impl bevy::prelude::Plugin for TrackFxPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        use bevy::prelude::*;
        app.init_resource::<TrackFxState>()
            .init_resource::<HeadlightView>()
            .init_resource::<HeadlightViews>()
            .add_systems(Update, load_track_fx)
            .add_systems(PostUpdate, draw_track_fx.in_set(crate::effect_draw::EffectOrder::TrackFx))
            .add_plugins(crate::flare_render::FlareRenderPlugin);
    }
}

/// GDL_LoadTrackSetup 0x0009E9E0 (FLARES) + GDL_LoadPointLightsTxt 0x0009E08F, at track load
fn load_track_fx(
    roots: bevy::prelude::Query<&crate::track_render::TrackRoot, bevy::prelude::Added<crate::track_render::TrackRoot>>,
    all: bevy::prelude::Query<(), bevy::prelude::With<crate::track_render::TrackRoot>>,
    mut state: bevy::prelude::ResMut<TrackFxState>,
) {
    for r in &roots {
        let base = format!("\\Tracks\\{}", r.track);
        let read = |f: &str| std::fs::read_to_string(visos::file::resolve(&format!("{base}/{f}"))).unwrap_or_default();
        let setup = TrackFxSetup::parse(&read("Setup.txt"));
        let names = ptlights_names(&read("PtLights.txt"));
        state.0 = Some(TrackFx::new(setup, &names));
    }
    if all.is_empty() && state.0.is_some() {
        state.0 = None;
    }
}

type BeamCarQuery<'w, 's> = bevy::prelude::Query<
    'w,
    's,
    (
        &'static crate::car_render::CarRender,
        &'static bevy::prelude::Transform,
        Option<&'static CarHeadBeams>,
        Option<&'static mut crate::car_render::CarShadow>,
        Option<&'static crate::car_render::CarLights>,
        Option<&'static crate::car_render::CarLightInput>,
        Option<&'static crate::car_damage_render::CarLightDamage>,
        &'static bevy::prelude::InheritedVisibility,
    ),
>;

/// `dword_737490` VISOS(0x60, 0, 0, 0): the black "carshadow" quad under a neon
pub const NEON_SHADOW_COLOUR: [u8; 4] = [0x60, 0, 0, 0];

/// TrackFX.cpp 0x0014B030 pass 4 (`byte_687280` = 1): per car with a neon (+2328 bit 1) and not culled, CarRender 0x00111B20
/// twice — the black full-size "carshadow" quad, then the neon colour through "CarNeonDA" at 0.8 / 0.9 size (each
/// call also moves the headlight level). `cars[i]` = (matrix, shadow, vehicle, culled).
pub fn render_neons(
    cars: &mut [(bevy::math::Mat4, &mut crate::car_render::CarShadow, Option<crate::car_render::ShadowVehicle>, bool)],
    step: f32,
    front_end: bool,
    ray: &mut dyn FnMut(Vec3, Vec3) -> Option<f32>,
    draw: &mut EffectDraw,
) {
    let (shadow, neon_sh) = (crate::car_palette::carshadow_shader(front_end), crate::car_palette::car_neon_shader(front_end));
    let black = visos_colour(NEON_SHADOW_COLOUR);
    for (m, sh, ve, culled) in cars.iter_mut() {
        let Some(neon) = sh.neon else { continue };
        if *culled {
            continue;
        }
        if let Some(q) = crate::car_render::shadow_quad(*m, sh, ve.as_ref(), true, black, step, ray) {
            draw.strip(&shadow, &q);
        }
        if let Some(q) = crate::car_render::shadow_quad(*m, sh, ve.as_ref(), false, neon, step, ray) {
            draw.strip(&neon_sh, &q);
        }
    }
}

/// what 0x00111B20 reads from the physics for car `id` (None without a vehicle: the no-vehicle branch)
fn shadow_vehicle(world: &velocity_physics::vehicle::vehicle_entity::sim::PhysicsWorld, id: usize, race_car: bool) -> Option<crate::car_render::ShadowVehicle> {
    if !world.vehicle_ids().contains(&id) {
        return None;
    }
    let s = world.slot(id);
    let (car, ve) = (&s.car.m, &s.ve.m);
    // 0x001BB200(1): body row 1 (up)
    let up = Vec3::new(ve.f(4804), ve.f(4808), ve.f(4812));
    // 0x00361390: +1968 + 16i when +2240 is set, else the wheel matrix origin +1596 + 64i
    let centre = |i: usize| Vec3::from(if car.i(2240) != 0 { car.v3(1968 + 16 * i) } else { car.v3(1596 + 64 * i) });
    Some(crate::car_render::ShadowVehicle {
        ground: std::array::from_fn(|i| centre(i) - up * car.f(1804 + 4 * i)),
        touches: std::array::from_fn(|i| car.i(1088 + 4 * i) > 0),
        race_car,
        flag_4388: ve.i(4388) != 0,
    })
}

/// the car parts of a frame: TrackFX 0x0014B030 (headlight passes 1-3 from the levels of the last frame, neon pass
/// 4) and CarRender's shadow pass 0x0011A6E0 (GameManager 0x000B975D, in the car draw after TrackFX); cars in
/// CarRender order (request order; race car 0 first)
#[allow(clippy::too_many_arguments)]
fn car_passes(
    g: &bevy::prelude::GlobalTransform,
    frustum: &bevy::camera::primitives::Frustum,
    time: &bevy::prelude::Time,
    clock: &crate::effect_draw::EffectClock,
    physics: Option<&velocity_physics::plugin::PhysicsWorldRes>,
    cars: &mut BeamCarQuery,
    view: &HeadlightView,
    zone: Option<&crate::zone_render_pc_xbox::ZoneFx>,
    planes: Option<&crate::full_screen_effects_manager_pc_xbox::FseCameraPlanes>,
    shadows_only: bool,
    draw: &mut EffectDraw,
) {
    let mut rows: Vec<_> = cars.iter_mut().collect();
    rows.sort_by_key(|r| r.0.request);
    let mut beam_cars = Vec::new();
    // (matrix, culled, inside, race car) per row
    let mut info = Vec::with_capacity(rows.len());
    for (r, t, hb, sh, lights, input, dmg, vis) in rows.iter() {
        let matrix = t.to_matrix();
        // 0x001127F0: sphere centre through the car matrix, radius as stored
        let in_view = hb.is_none_or(|hb| {
            let c = matrix.transform_point3(hb.sphere.truncate());
            frustum.half_spaces.iter().all(|h| h.normal_d().dot(c.extend(1.0)) + hb.sphere.w > 0.0)
        });
        let inside = view.inside == Some(r.request);
        let culled = !vis.get() || !in_view || inside;
        info.push((matrix, culled, inside, !r.spec.traffic, r.request));
        if let Some(hb) = hb {
            beam_cars.push(BeamCar {
                matrix,
                head: hb.head,
                brake: lights.map_or(0.0, |l| l.brake_level),
                level: sh.as_ref().map_or(1.0, |s| s.level),
                damage: input.map_or(0, |i| i.damage) | dmg.map_or(0, |d| d.0),
                culled,
                inside,
            });
        }
    }
    let near = planes.map_or(f32::from_bits(0x3F00_0000), |p| p.near);
    let zone_mask = zone.map_or(-1, |z| z.zone_mask);
    if !shadows_only {
        render_headlights(&beam_cars, g.translation(), g.forward().as_vec3(), near, view.camera_mode, zone_mask, false, draw);
    }
    // per rendered frame in the original: scaled to the real frame delta
    let dt = if clock.paused { 0.0 } else { time.delta_secs() };
    let step = crate::car_render::HEADLIGHT_STEP * dt * crate::car_render::XBOX_FRAME_RATE;
    let mut guard = physics.map(|p| p.lock());
    let mut world = guard.as_mut().and_then(|g| g.as_mut());
    let vehicles: Vec<Option<crate::car_render::ShadowVehicle>> =
        info.iter().map(|i| world.as_deref().and_then(|w| shadow_vehicle(w, i.4 as usize, i.3))).collect();
    let mut ray = |a: Vec3, b: Vec3| {
        use velocity_physics::vehicle::vehicle_entity::sim::{RAY_NORMAL, RAY_TRACK};
        // MaterialProperties 0x0019D350 (flags 17)
        world.as_deref_mut().and_then(|w| w.ray_cast(a.to_array(), b.to_array(), RAY_TRACK | RAY_NORMAL, -1)).map(|h| h.t)
    };
    let mut neons: Vec<_> = rows
        .iter_mut()
        .zip(info.iter().zip(vehicles.iter()))
        .filter(|(row, _)| !row.0.spec.frontend)
        .filter_map(|(row, (i, ve))| row.3.as_deref_mut().map(|sh| (i.0, sh, *ve, i.1)))
        .collect();
    if !shadows_only {
        render_neons(&mut neons, step, false, &mut ray, draw);
    }
    drop(neons);
    // CarRender 0x0011A6E0, car index = CarRender slot
    let sh = crate::car_palette::carshadow_shader(false);
    let colour = visos_colour(crate::car_render::SHADOW_COLOUR[0]);
    for (k, (row, (i, ve))) in rows.iter_mut().zip(info.iter().zip(vehicles.iter())).enumerate() {
        // front-end cars: `car_render::draw_frontend_shadows`
        if row.0.spec.frontend {
            continue;
        }
        let Some(s) = row.3.as_deref_mut() else { continue };
        if let Some(q) = crate::car_render::shadow_pass(k, i.0, s, ve.as_ref(), i.1, i.2, zone_mask, colour, step, &mut ray) {
            draw.strip(&sh, &q);
        }
    }
}

/// GameManager 0x000B9010 -> TrackFX 0x0014A010 / 0x0014B030 / 0x0014A5B0 (frame time), per rendered frame inside the
/// view loop: once per view with that view's camera (each call steps the timers by the whole frame time, so twice a
/// frame for two players, as the original)
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_track_fx(
    time: bevy::prelude::Res<bevy::prelude::Time>,
    clock: bevy::prelude::Res<crate::effect_draw::EffectClock>,
    mut state: bevy::prelude::ResMut<TrackFxState>,
    mut points: Option<bevy::prelude::ResMut<crate::track_render::TrackPointLights>>,
    views: crate::view_render::ViewCams,
    mut draw: bevy::prelude::ResMut<EffectDraw>,
    physics: Option<bevy::prelude::Res<velocity_physics::plugin::PhysicsWorldRes>>,
    mut cars: BeamCarQuery,
    view: bevy::prelude::Res<HeadlightView>,
    others: Option<bevy::prelude::Res<HeadlightViews>>,
    zone: Option<bevy::prelude::Res<crate::zone_render_pc_xbox::ZoneFx>>,
    planes: Option<bevy::prelude::Res<crate::full_screen_effects_manager_pc_xbox::FseCameraPlanes>>,
) {
    let others = others.map(|o| *o).unwrap_or_default();
    for v in views.list() {
        let (Some(proj), Some(frustum)) = (v.projection.as_ref(), v.frustum.as_ref()) else { continue };
        draw.set_view(v.tag);
        let hv = others.for_view(&view, &v);
        track_fx_view(
            &v.transform,
            proj,
            frustum,
            &time,
            &clock,
            &mut state,
            points.as_deref_mut(),
            &mut draw,
            physics.as_deref(),
            &mut cars,
            &hv,
            zone.as_deref(),
            planes.as_deref(),
        );
    }
    // the mirror view (flags 0x08000620: no TrackFX) still runs the car draw 0x00119900 → shadows 0x0011A6E0 with its
    // camera (cull and in-car per CarRender 0x001127F0 in camera mode 2)
    if let Some(m) = views.mirror() {
        if let Some(frustum) = m.frustum.as_ref() {
            draw.set_view(m.tag);
            let hv = others.for_view(&view, &m);
            car_passes(&m.transform, frustum, &time, &clock, physics.as_deref(), &mut cars, &hv, zone.as_deref(), planes.as_deref(), true, &mut draw);
        }
    }
    draw.set_view(None);
}

#[allow(clippy::too_many_arguments)]
fn track_fx_view(
    g: &bevy::prelude::GlobalTransform,
    proj: &bevy::prelude::Projection,
    frustum: &bevy::camera::primitives::Frustum,
    time: &bevy::prelude::Time,
    clock: &crate::effect_draw::EffectClock,
    state: &mut TrackFxState,
    points: Option<&mut crate::track_render::TrackPointLights>,
    draw: &mut EffectDraw,
    physics: Option<&velocity_physics::plugin::PhysicsWorldRes>,
    cars: &mut BeamCarQuery,
    view: &HeadlightView,
    zone: Option<&crate::zone_render_pc_xbox::ZoneFx>,
    planes: Option<&crate::full_screen_effects_manager_pc_xbox::FseCameraPlanes>,
) {
    car_passes(g, frustum, time, clock, physics, cars, view, zone, planes, false, draw);
    let (Some(fx), Some(points)) = (state.0.as_mut(), points) else { return };
    // 0x00149020 runs at PhysicsManager::Init: here once the track collision is there (no world, no reflections)
    if !fx.reflections_set_up {
        if let Some(phys) = physics.as_ref() {
            let mut world = phys.lock();
            if let Some(w) = world.as_mut() {
                use velocity_physics::vehicle::vehicle_entity::sim::{RAY_NORMAL, RAY_TRACK};
                let lights: Vec<Vec3> = points.points.iter().map(|l| l.pos).collect();
                let mut ray = |a: Vec3, b: Vec3| w.ray_cast(a.to_array(), b.to_array(), RAY_TRACK | RAY_NORMAL, -1).map(|h| h.t);
                fx.setup_reflections(&lights, false, &mut ray);
                let before: f32 = points.points.iter().map(|l| l.radius).sum();
                fx.ground_heights(&mut points.points);
                let after: f32 = points.points.iter().map(|l| l.radius).sum();
                let n = fx.lights.iter().filter(|l| l.flags & 2 != 0).count();
                log::info!("track fx: {n} of {} lights reflect; light radii sum {before} -> {after}", fx.lights.len());
            }
        }
    }
    if fx.reflections_set_up {
        let colours: Vec<Vec3> = points.points.iter().map(|l| l.colour.truncate()).collect();
        fx.render_reflections(&colours, g.translation(), g.forward().as_vec3(), draw);
    }
    let bevy::prelude::Projection::Perspective(p) = proj else { return };
    let tan_y = (p.fov * 0.5).tan();
    let tan_x = tan_y * p.aspect_ratio;
    let (pos, fwd, right, up) = (g.translation(), g.forward().as_vec3(), g.right().as_vec3(), g.up().as_vec3());
    let dt = if clock.paused { 0.0 } else { time.delta_secs() * clock.scale };
    let lights: Vec<Vec3> = points.points.iter().map(|l| l.pos).collect();
    // 0x0015EC80: view then projection, x / w and -y / w to 0..1, z / w
    let project = |w: Vec3| {
        let d = w - pos;
        let ew = fwd.dot(d);
        let x = right.dot(d) / (ew * tan_x);
        let y = up.dot(d) / (ew * tan_y);
        Vec4::new((x + 1.0) / 2.0, (-y + 1.0) / 2.0, 1.0 - p.near / ew.max(p.near), ew)
    };
    let visible = |c: Vec3, r: f32| frustum.half_spaces.iter().all(|h| h.normal_d().dot(c.extend(1.0)) + r > 0.0);
    fx.render(&lights, pos, fwd, -right, up, dt, visible, project, draw, |s, w, d| {
        crate::light_streak::draw_slot(s, w, pos, right, up, fwd, tan_x, tan_y, d)
    });
}
