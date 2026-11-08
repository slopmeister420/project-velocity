//! Render\CatsEyeRender.cpp — the road's cat's eyes: per race a `CatsEyes.ced` of point strings (only LADown Circuit
//! Race02 / Race09 ship one), drawn each view as camera-facing glints lit by the player car's headlights plus flat
//! glow quads on the road (object `unk_7184E8`; init CarRender.cpp 0x0011CB10, load 0x0011CDB0, draw 0x0011CF70).
//! The editor's `CatsEyes.txt` sources are not read by the game.

use bevy::camera::primitives::{Frustum, Sphere};
use bevy::prelude::*;

use crate::car_render::CarRender;
use crate::effect_draw::{engine_colour, EffectDraw,EffectShader, EffectTexture, EffectVertex};

/// `CATS_EYE_VERSION`
pub const CATS_EYE_VERSION: u16 = 1;
/// `dword_686858`: distance where the glints start to fade
pub const FADE_START: f32 = f32::from_bits(0x41A0_0000);
/// `dword_68685C`: draw distance (and the sector search margin)
pub const MAX_DIST: f32 = f32::from_bits(0x4270_0000);
/// `dword_686860`: glint size at the camera, and the size of a glow between the camera and the car
pub const SIZE_NEAR: f32 = f32::from_bits(0x3E99_999A);
/// `dword_686864`: glint size at `MAX_DIST`
pub const SIZE_FAR: f32 = f32::from_bits(0x3FB3_3333);
/// `dword_686868`: depth bias of the three shaders
pub const DEPTH_BIAS: i16 = 0x80;
/// `dword_68686C`: glint alpha at full strength
pub const ALPHA: u32 = 0xFF;
/// `flt_686870`: glint half width (× size, along the camera's right)
pub const HALF_WIDTH: f32 = f32::from_bits(0x3F00_0000);
/// `flt_686874`: glint half height (× size, along the camera's up)
pub const HALF_HEIGHT: f32 = f32::from_bits(0x3E4C_CCCD);
/// `dword_686878`: glow half size (× the glow's size)
pub const GLOW_SCALE: f32 = f32::from_bits(0x3E99_999A);
/// `dword_68687C`: bit 0 glows, bit 1 glints ("CatsEye Display")
pub const DISPLAY: u32 = 3;
/// `dword_686880`: a near glint this faint also gets a glow
pub const GLOW_FADE: f32 = f32::from_bits(0x3F00_0000);
/// `dword_686884`: ... when nearer than this
pub const GLOW_DIST: f32 = f32::from_bits(0x4170_0000);
/// `dword_624C6C`: faintest glint drawn
pub const MIN_FADE: f32 = f32::from_bits(0x3C23_D70A);
/// `dword_63B99C`: point delta unit (i16 millimetres)
pub const DELTA_UNIT: f32 = f32::from_bits(0x3A83_126F);
/// `kMAX_CATEYES`: glints + near glows per frame
pub const MAX_CATEYES: u32 = 0x100;

/// `dword_71851C` (static init 0x00424D40): engine colours of string kinds 0 red, 1 green (a 0x80); later kinds read
/// past the table (static zero here)
pub fn kind_colour(kind: u16) -> u32 {
    // OvColourXbox(a, r, g, b) -> bytes r, g, b, a
    let visos = |a: u32, r: u32, g: u32, b: u32| r | (g << 8) | (b << 16) | (a << 24);
    match kind {
        0 => visos(0x80, 0xFF, 0, 0),
        1 => visos(0x80, 0, 0xFF, 0),
        _ => 0,
    }
}

/// one string of cat's eyes (+0 kind, +2 n, +4 start xyz, +16 n × i16[3] deltas in mm): n + 1 points
#[derive(Clone, Debug, PartialEq)]
pub struct CatsEyeString {
    pub kind: u16,
    pub points: Vec<Vec3>,
}

/// a sector record (20 B: centre, radius, +16 string offset in dwords)
#[derive(Clone, Debug, PartialEq)]
pub struct CatsEyeSector {
    pub centre: Vec3,
    pub radius: f32,
    pub string: CatsEyeString,
}

/// a loaded `CatsEyes.ced`: "CatEye", +6 version, +8 sector count, +10 sector table offset in dwords
#[derive(Resource, Clone, Debug, Default)]
pub struct CatsEyeRender {
    pub sectors: Vec<CatsEyeSector>,
}

impl CatsEyeRender {
    /// CatsEyeRender.cpp 0x0011CDB0 — parse the file (the string walk of 0x0011D450 done once here). Strings are
    /// packed at 2-byte steps but addressed in dwords: one not on a dword boundary is read 2 bytes early (kind and
    /// count from the bytes before it, start point garbage far off the track), so it never draws (original bug, kept;
    /// both shipped files: 2 of 4 strings).
    pub fn parse(d: &[u8]) -> Option<Self> {
        let u16_at = |o: usize| d.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
        let i16_at = |o: usize| u16_at(o).map(|v| v as i16);
        let f32_at = |o: usize| d.get(o..o + 4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]));
        if u16_at(6)? != CATS_EYE_VERSION {
            log::error!("assert: m_pFileHeader->m_version == CATS_EYE_VERSION");
            return None;
        }
        let count = u16_at(8)? as usize;
        let table = 4 * u16_at(10)? as usize;
        let mut sectors = Vec::with_capacity(count);
        for i in 0..count {
            let r = table + 20 * i;
            let centre = Vec3::new(f32_at(r)?, f32_at(r + 4)?, f32_at(r + 8)?);
            let radius = f32_at(r + 12)?;
            let s = 4 * u16_at(r + 16)? as usize;
            let (kind, n) = (u16_at(s)?, u16_at(s + 2)?);
            let mut p = Vec3::new(f32_at(s + 4)?, f32_at(s + 8)?, f32_at(s + 12)?);
            let mut points = Vec::with_capacity(n as usize + 1);
            // n + 1 points; the delta after the last point is read and unused (the next string's header)
            for k in 0..=n as usize {
                points.push(p);
                if k == n as usize {
                    break;
                }
                let o = s + 16 + 6 * k;
                let delta = Vec3::new(i16_at(o)? as f32 * DELTA_UNIT, i16_at(o + 2)? as f32 * DELTA_UNIT, i16_at(o + 4)? as f32 * DELTA_UNIT);
                p += delta;
            }
            sectors.push(CatsEyeSector { centre, radius, string: CatsEyeString { kind, points } });
        }
        Some(Self { sectors })
    }
}

/// what one view sees the cat's eyes with
#[derive(Clone, Copy, Debug)]
pub struct CatsEyeView {
    /// VsBaseCamera +72
    pub cam_pos: Vec3,
    /// +88 `m_right_RENDER` (bevy's left) and +104 up
    pub right: Vec3,
    pub up: Vec3,
    /// plane 4 (+296 + 64): the near plane, n = view direction; depth = n·p + w
    pub plane: Vec4,
    /// the player car's matrix (car data +0: row 2 = nose, row 3 = position)
    pub car: Mat4,
}

/// one frame's quads: glows (flat on the road) and glints (camera facing)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CatsEyeQuads {
    /// (position, half size factor = the stack entry's w)
    pub glows: Vec<(Vec3, f32)>,
    /// (position, engine colour, size)
    pub eyes: Vec<(Vec3, u32, f32)>,
}

fn depth(plane: Vec4, p: Vec3) -> f32 {
    plane.truncate().dot(p) + plane.w
}

impl CatsEyeRender {
    /// CatsEyeRender.cpp 0x0011CF70 — the sectors within (radius + 60) of the camera and in the frustum, through
    /// 0x0011D450 each
    pub fn collect(&self, v: &CatsEyeView, visible: impl Fn(Vec3, f32) -> bool) -> CatsEyeQuads {
        let mut q = CatsEyeQuads::default();
        let mut count = 0u32;
        for s in &self.sectors {
            let r = s.radius + MAX_DIST;
            if r * r > (s.centre - v.cam_pos).length_squared() && visible(s.centre, s.radius) {
                Self::string(&s.string, v, &mut count, &mut q);
            }
        }
        q
    }

    /// CatsEyeRender.cpp 0x0011D450 + 0x0011DE10 + 0x0011DDF0 — one string: points 0 < depth < 60 m. Nearer than the player car: a glow of
    /// size 0.3. Beyond it: fade = c⁵ · (1 beyond 20 m fading to 0 at 60 m), c = car nose · normalise(p − car), size =
    /// lerp(0.3, 1.4, depth / 60); fade > 0.01 → a glint with alpha _ftol(255 · fade) (0x0011DB40); within 15 m and
    /// fade < 0.5 also a glow of that size. Glints and near glows count against kMAX_CATEYES.
    fn string(st: &CatsEyeString, v: &CatsEyeView, count: &mut u32, q: &mut CatsEyeQuads) {
        let base = kind_colour(st.kind);
        let car_pos = v.car.w_axis.truncate();
        let nose = v.car.z_axis.truncate();
        let car_depth = depth(v.plane, car_pos);
        for &p in &st.points {
            let d = depth(v.plane, p);
            if !(d > 0.0 && MAX_DIST > d) {
                continue;
            }
            if d <= car_depth {
                q.glows.push((p, SIZE_NEAR));
                continue;
            }
            let mut fade = if FADE_START > d { 1.0 } else { 1.0 - (d - FADE_START) / (MAX_DIST - FADE_START) };
            let mut to = p - car_pos;
            let l = to.length();
            if l.abs() > 1e-6 {
                to /= l;
            }
            let c = nose.dot(to);
            let mut c4 = c * c;
            c4 *= c4;
            fade = c * c4 * fade;
            let size = (SIZE_FAR - SIZE_NEAR) * (d / MAX_DIST) + SIZE_NEAR;
            if fade > MIN_FADE {
                if *count >= MAX_CATEYES {
                    log::error!("assert: false && \"CatsEyeRender:: exceeded kMAX_CATEYES\"");
                } else {
                    let a = (ALPHA as f64 * fade as f64) as i64 as u32 & 0xFF;
                    q.eyes.push((p, (base & 0x00FF_FFFF) | (a << 24), size));
                    *count += 1;
                }
            }
            if GLOW_DIST > d && GLOW_FADE > fade {
                if *count >= MAX_CATEYES {
                    log::error!("assert: false && \"CatsEyeRender:: exceeded kMAX_CATEYES\"");
                } else {
                    q.glows.push((p, size));
                    *count += 1;
                }
            }
        }
    }
}

/// uv template of the four strip vertices (OSDManager 0xEFD30 u / 0xEFD50 v in 4096ths)
const UV: [Vec2; 4] = [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0), Vec2::new(1.0, 1.0)];

/// CatsEyeRender.cpp 0x0011D960 + 0x0011DDC0 — a glow: flat quad on the xz plane, half size 0.3 · w, white
pub fn glow_quad(p: Vec3, w: f32) -> [EffectVertex; 4] {
    let z = Vec3::new(0.0, 0.0, GLOW_SCALE * w);
    let x = Vec3::new(GLOW_SCALE * w, 0.0, 0.0);
    let c = engine_colour(0x8080_8080);
    let pos = [p - x + z, p + x + z, p - x - z, p + x - z];
    std::array::from_fn(|i| EffectVertex::new(pos[i], c, UV[i]))
}

/// CatsEyeRender.cpp 0x0011DB40 / 0x0011DF60 / 0x0011DFF0 — a glint: camera-facing quad, right · size · 0.5 by up · size · 0.2, colour
/// through 0x0011DE40
pub fn eye_quad(p: Vec3, colour: u32, size: f32, right: Vec3, up: Vec3) -> [EffectVertex; 4] {
    let a = up * size * HALF_HEIGHT;
    let b = right * size * HALF_WIDTH;
    let c = engine_colour(colour);
    let pos = [p - b + a, p + b + a, p - b - a, p + b - a];
    std::array::from_fn(|i| EffectVertex::new(pos[i], c, UV[i]))
}

/// CarRender.cpp 0x0011CB10 — +12 "catseye": `spec_t_gk_002`, blend 7, z test without write, 0x002A15F0(1)
/// (unidentified, not modelled), bias 128. The draw picks +12 + 4 · `dword_7184E4` (static zero): this one; +16
/// "catseye2" (`spec3`) is never drawn.
pub fn eye_shader() -> EffectShader {
    EffectShader::new(EffectTexture::Misc("spec_t_gk_002".into()), 7).z(true, false).bias(DEPTH_BIAS)
}

/// CarRender.cpp 0x0011CB10 — +20 "catseye2": `catunlit`, blend 3, z test without write, bias 128
pub fn glow_shader() -> EffectShader {
    EffectShader::new(EffectTexture::Misc("catunlit".into()), 3).z(true, false).bias(DEPTH_BIAS)
}

/// GameDataLoading "Load catseyes" (0x0009D055): `<race dir>CatsEyes.ced` (race dir with its trailing `\`)
#[derive(Message, Clone, Debug)]
pub struct LoadCatsEyes {
    pub race_dir: String,
}

/// drop the loaded file (race shutdown)
#[derive(Message, Clone, Debug, Default)]
pub struct UnloadCatsEyes;

pub struct CatsEyeRenderPlugin;

impl Plugin for CatsEyeRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CatsEyeRender>()
            .add_message::<LoadCatsEyes>()
            .add_message::<UnloadCatsEyes>()
            .add_systems(Update, cats_eye_messages)
            .add_systems(PostUpdate, draw_cats_eyes.in_set(crate::effect_draw::EffectOrder::CatsEye));
    }
}

/// CatsEyeRender.cpp 0x0011CDB0 — FILE_LoadWholeFile of `%s%s.%s` (dir, "CatsEyes", "ced"); missing → none
fn cats_eye_messages(mut ce: ResMut<CatsEyeRender>, mut load: MessageReader<LoadCatsEyes>, mut unload: MessageReader<UnloadCatsEyes>) {
    if unload.read().count() > 0 {
        ce.sectors.clear();
    }
    for l in load.read() {
        let path = format!("{}CatsEyes.ced", l.race_dir);
        *ce = match std::fs::read(visos::file::resolve(&path)) {
            Ok(d) => CatsEyeRender::parse(&d).unwrap_or_default(),
            Err(_) => {
                log::info!("failed to load {path}");
                CatsEyeRender::default()
            }
        };
    }
}

/// GameManager 0x000B9010 → 0x0011CF70 in every view with render flag 0x100 (not the mirror): glows (bit 0) then
/// glints (bit 1). The car is the race's player vehicle (`VsAIRaceVehicle__24F90` +4) in every view.
fn draw_cats_eyes(
    ce: Res<CatsEyeRender>,
    mut draw: ResMut<EffectDraw>,
    views: crate::view_render::ViewCams,
    cars: Query<(&CarRender, &Transform)>,
) {
    if ce.sectors.is_empty() {
        return;
    }
    let car = cars
        .iter()
        .filter(|(c, _)| !c.spec.frontend && !c.spec.traffic && !c.spec.police)
        .min_by_key(|(c, _)| c.request)
        .map_or(Mat4::IDENTITY, |(_, t)| t.to_matrix());
    for v in views.list() {
        let cam_pos = v.transform.translation();
        let fwd = v.transform.forward().as_vec3();
        let near = match &v.projection {
            Some(Projection::Perspective(p)) => p.near,
            _ => 0.1,
        };
        let view = CatsEyeView {
            cam_pos,
            right: -v.transform.right().as_vec3(),
            up: v.transform.up().as_vec3(),
            plane: fwd.extend(0.0 - fwd.dot(cam_pos + fwd * near)),
            car,
        };
        let frustum: Option<&Frustum> = v.frustum.as_ref();
        let visible = |c: Vec3, r: f32| frustum.is_none_or(|f| f.intersects_sphere(&Sphere { center: c.into(), radius: r }, true));
        let q = ce.collect(&view, visible);
        log::trace!("cats eyes view {}: {} glows, {} glints (car {:?})", v.player, q.glows.len(), q.eyes.len(), car.w_axis);
        draw.set_view(v.tag);
        if !q.glows.is_empty() && DISPLAY & 1 != 0 {
            let sh = glow_shader();
            for (p, w) in &q.glows {
                draw.strip(&sh, &glow_quad(*p, *w));
            }
        }
        if !q.eyes.is_empty() && DISPLAY & 2 != 0 {
            let sh = eye_shader();
            for (p, c, s) in &q.eyes {
                draw.strip(&sh, &eye_quad(*p, *c, *s, view.right, view.up));
            }
        }
    }
    draw.set_view(None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cats_eye_file() {
        let root = visos::file::resolve("\\Tracks");
        let mut n = 0;
        for track in std::fs::read_dir(root).unwrap().flatten() {
            for kind in ["Circuit", "Sprint"] {
                let Ok(races) = std::fs::read_dir(track.path().join(kind)) else { continue };
                for race in races.flatten() {
                    let p = race.path().join("catseyes.ced");
                    let Ok(d) = std::fs::read(&p) else { continue };
                    let ce = CatsEyeRender::parse(&d).unwrap_or_else(|| panic!("{}", p.display()));
                    assert!(!ce.sectors.is_empty());
                    let mut good = 0;
                    for s in &ce.sectors {
                        // a string on a dword boundary lies inside its sector's sphere; the others are read 2 bytes
                        // early (the stored offset is bytes / 4 truncated) and lie out of reach: a drawn point is within 60 m of a
                        // camera within radius + 60 m of the centre
                        let far = s.string.points.iter().map(|q| (*q - s.centre).length()).fold(0.0, f32::max);
                        if far <= s.radius + 0.01 {
                            assert!(s.string.kind <= 1);
                            good += 1;
                        } else {
                            let near = s.string.points.iter().map(|q| (*q - s.centre).length()).fold(f32::INFINITY, f32::min);
                            assert!(!(near <= s.radius + 2.0 * MAX_DIST), "{} {near}", p.display());
                        }
                    }
                    assert_eq!(good, 2, "{}", p.display());
                    n += 1;
                }
            }
        }
        assert_eq!(n, 2);
    }

    /// the glint texture is dim (avg rgb 41, alpha 44): additive glints are faint in the original too
    #[test]
    #[ignore]
    fn dump_textures() {
        let v = visos::file::Vcd::open("\\VFX\\misc.vcd").unwrap();
        for n in ["spec_t_gk_002", "spec3", "catunlit"] {
            let t = crate::renderer::texture_manager::decode_any_vtf(v.get(&format!("{n}.vtf")).unwrap()).unwrap();
            let px = &t.levels[0];
            let k = px.len() / 4;
            let avg = |c: usize| px.chunks(4).map(|p| p[c] as u64).sum::<u64>() / k as u64;
            let max = |c: usize| px.chunks(4).map(|p| p[c]).max().unwrap();
            println!("{n} {}x{} avg rgba {} {} {} {} max a {}", t.width, t.height, avg(0), avg(1), avg(2), avg(3), max(3));
        }
    }

    #[test]
    fn glints_need_the_headlights() {
        let st = CatsEyeString { kind: 1, points: vec![Vec3::new(0.0, 0.0, 30.0), Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -30.0)] };
        let ce = CatsEyeRender { sectors: vec![CatsEyeSector { centre: Vec3::ZERO, radius: 40.0, string: st }] };
        // camera behind the car looking +z, car at z 10 facing +z
        let v = CatsEyeView {
            cam_pos: Vec3::ZERO,
            right: Vec3::NEG_X,
            up: Vec3::Y,
            plane: Vec4::new(0.0, 0.0, 1.0, -0.1),
            car: Mat4::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        };
        let q = ce.collect(&v, |_, _| true);
        // z 30: depth 29.9 (fade 1 − 9.9 / 40, c = 1) → glint; z 5: before the car → glow 0.3; z −30: behind the camera
        assert_eq!(q.glows, vec![(Vec3::new(0.0, 0.0, 5.0), SIZE_NEAR)]);
        assert_eq!(q.eyes.len(), 1);
        let d = 30.0f32 + -0.1f32;
        let fade = 1.0 - (d - 20.0) / 40.0;
        assert_eq!(q.eyes[0].1 >> 24, (255.0 * fade as f64) as u32);
        assert_eq!(q.eyes[0].1 & 0xFF_FFFF, 0x00_FF00);
        assert_eq!(kind_colour(0), 0x8000_00FF);
    }
}
