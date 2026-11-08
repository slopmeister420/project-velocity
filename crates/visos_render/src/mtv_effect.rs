//! the front end's "MTV" glitch overlay (object `byte_720208`; its functions sit in LightStreak.cpp's range
//! 0x001300D0..0x001310E3, tweak system "Frontend.MTV Effect"): black scrolling bars, flashing "FE_Glitch_*" image
//! sequences and a timed orange multiply tint over the front-end view, run for a while when the front-end camera
//! moves (FE3DCamera 0x00275E50 / 0x00275F10) and when the quick-race car select changes car.
//!
//! API (frontend): `ResMut<MtvEffect>` — `start(seconds, no_glitch)` (0x00130310) and `stop(immediate)` (0x00130340).
//! Updated and drawn every rendered frame while a `FrontEndCamera` exists (FECK2ElementCamera 0x002999B0 draws it last),
//! with the real frame delta, in the 512² pretransformed space stretched over the front-end camera's target.

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{Camera3dDepthLoadOp, CameraOutputMode, RenderTarget};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::prelude::*;
use visos::handy::crt_rand::CrtRand;

use crate::rain_surface::{screen_mesh, FrontEndCamera};
use crate::renderer::shader_manager::{StageState, XboxStageMaterial, XboxStageParams};

/// the overlay camera's layer and order (after the foyer camera 0 and the reflection paste / post passes 1 / 2,
/// before the front end's 2d camera 10). was 17 = `view_render::VIEW_CAR_LAYERS[0]`: the overlay camera saw race car
/// parts and `mark_car_pass` treated the overlay quads as car parts (Transmissive3d)
pub const MTV_LAYER: usize = 10;
pub const MTV_ORDER: isize = 3;

/// `off_686C40`: the "FE_Glitch_%s" texture suffixes (+104 + 4·i)
pub const GLITCH_NAMES: [&str; 14] = ["A01L", "A02L", "A03L", "A01R", "A02R", "A03R", "B01", "B02", "B03", "B04", "C01", "C02", "C03", "C04"];
/// `dword_686C78`: size scale per glitch texture
pub const GLITCH_SCALE: [f32; 14] = [128.0, 128.0, 256.0, 256.0, 128.0, 128.0, 256.0, 256.0, 256.0, 256.0, 256.0, 256.0, 256.0, 256.0];

/// one 16-byte quad of a glitch frame (x, y, size, texture; −1 = none)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlitchQuad {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub tex: i32,
}

const fn q(x: f32, y: f32, size: f32, tex: i32) -> GlitchQuad {
    GlitchQuad { x, y, size, tex }
}

/// one 8-byte sequence of `0x00686E50` (frames ptr, u16 frame count, u16 quads per frame)
pub struct GlitchSeq {
    pub quads: &'static [GlitchQuad],
    pub frames: u16,
    pub per_frame: u16,
}

/// `0x00686E50`: the four sequences (quads at 0x00686CB0 / 0x00686D30 / 0x00686D70 / 0x00686DB0)
pub static GLITCH_SEQS: [GlitchSeq; 4] = [
    GlitchSeq {
        quads: &[
            q(0.0, 201.0, 133.0, 0),
            q(384.0, 160.0, 133.0, 3),
            q(-78.75, 171.0, 142.0, 1),
            q(393.75, 171.0, 142.0, 4),
            q(0.0, 261.0, 133.0, 2),
            q(384.0, 218.0, 133.0, 5),
            q(-41.25, 121.0, 142.0, 1),
            q(365.25, 121.0, 142.0, 4),
        ],
        frames: 4,
        per_frame: 2,
    },
    GlitchSeq {
        quads: &[q(22.5, 0.0, 122.0, 6), q(48.0, 0.0, 100.0, 7), q(33.75, 0.0, 100.0, 8), q(0.0, 54.0, 122.0, 9)],
        frames: 4,
        per_frame: 1,
    },
    GlitchSeq {
        quads: &[q(-150.0, 250.0, 122.0, 10), q(-150.0, 250.0, 100.0, 11), q(-140.0, 260.0, 100.0, 12), q(-130.0, 320.0, 122.0, 13)],
        frames: 4,
        per_frame: 1,
    },
    GlitchSeq {
        quads: &[
            q(426.0, 14.0, 100.0, 4),
            q(0.0, 0.0, 0.0, -1),
            q(340.0, 107.0, 100.0, 6),
            q(359.0, -138.0, 100.0, 12),
            q(441.0, 27.0, 100.0, 7),
            q(0.0, 0.0, 0.0, -1),
            q(417.0, 60.0, 100.0, 7),
            q(0.0, 0.0, 0.0, -1),
            q(344.0, 8.0, 100.0, 11),
            q(0.0, 0.0, 0.0, -1),
        ],
        frames: 5,
        per_frame: 2,
    },
];

/// `flt_686E78`: lower bound of a bar's random speed / alpha factor
pub const BAR_RAND_MIN: f32 = f32::from_bits(0x3EA8_F5C3);
/// `unk_720520` (static, 0x00130A9D): the tint colour before scaling, and the luminance weights it is scaled by
pub const TINT_COLOUR: Vec3 = Vec3::new(1.0, f32::from_bits(0x3EB3_3333), 0.0);
pub const TINT_LUMA: Vec3 = Vec3::new(f32::from_bits(0x3E99_999A), 0.5, f32::from_bits(0x3E4C_CCCD));
/// `dword_625378`
pub const TINT_MIN_LUMA: f32 = f32::from_bits(0x3DCC_CCCD);

/// the "Frontend.MTV Effect" tweaks (LightStreak.cpp 0x00131130; .data defaults)
#[derive(Clone, Copy, Debug)]
pub struct MtvTweaks {
    /// `dword_686E7C` "Line Freq"
    pub line_freq: f32,
    /// `dword_686E80` "Line Alpha"
    pub line_alpha: f32,
    /// `dword_686E70` "Line Speed"
    pub line_speed: f32,
    /// `dword_686E74` "Line Width"
    pub line_width: f32,
    /// `dword_686E84` "Image Freq"
    pub image_freq: f32,
    /// `flt_686E88` "Image Alpha"
    pub image_alpha: f32,
    /// `dword_686EA0` "Image Speed" (frames per second)
    pub image_speed: f32,
    /// `dword_686E8C` "Tint Freq"
    pub tint_freq: f32,
    /// `dword_686E90` "Tint Intensity"
    pub tint_intensity: f32,
    /// `dword_686E94` "Desat Intensity" (computed with flag 2 of `dword_686EA4`, never used: 0x00130C00 drops it)
    pub desat_intensity: f32,
    /// `dword_686E98` "Tint Time in"
    pub tint_in: f32,
    /// `dword_686E9C` "Tint Time out"
    pub tint_out: f32,
    /// `dword_686EA4` (3; no tweak)
    pub flags: u32,
}

impl Default for MtvTweaks {
    fn default() -> Self {
        Self {
            line_freq: 100.0,
            line_alpha: 0.5,
            line_speed: 512.0,
            line_width: 2.0,
            image_freq: 100.0,
            image_alpha: 1.0,
            image_speed: 15.0,
            tint_freq: 100.0,
            tint_intensity: 0.5,
            desat_intensity: 0.5,
            tint_in: f32::from_bits(0x3E4C_CCCD),
            tint_out: 0.5,
            flags: 3,
        }
    }
}

/// one scrolling bar (16 bytes at +4 + 16·i)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MtvBar {
    /// +0
    pub x: f32,
    /// +4 alpha (0..128 scale); > 0 = live
    pub alpha: f32,
    /// +8
    pub width: f32,
    /// +12 x per second
    pub speed: f32,
}

/// a quad to draw in the 512² space: texture (−1 = "MTVLines", else the glitch index), corners, D3DCOLOR, uvs
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MtvQuad {
    pub tex: i32,
    pub p0: Vec2,
    pub p1: Vec2,
    pub colour: u32,
    pub uv0: Vec2,
    pub uv1: Vec2,
}

/// what one frame draws: quads in order, then the tint (D3DCOLOR of the "MTVTint" full-screen multiply)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MtvFrame {
    pub quads: Vec<MtvQuad>,
    pub tint: Option<u32>,
}

/// the effect object (`byte_720208`)
#[derive(Resource, Clone, Debug)]
pub struct MtvEffect {
    /// +0 running ("Enabled" tweak)
    pub enabled: bool,
    /// +1 start flag: no new bars or images (tint only)
    pub no_glitch: bool,
    /// +4
    pub bars: [MtvBar; 5],
    /// +84 current frame (float)
    pub frame: f32,
    /// +88 running sequence (index into `GLITCH_SEQS`)
    pub seq: Option<usize>,
    /// +92 tint clock, −1 = off
    pub tint: f32,
    /// +96 seconds left to start new pieces
    pub time: f32,
    pub tweaks: MtvTweaks,
    /// VsRandUnit / rand (the original shares the CRT state)
    pub rng: CrtRand,
}

impl Default for MtvEffect {
    fn default() -> Self {
        let mut m = Self {
            enabled: false,
            no_glitch: false,
            bars: [MtvBar::default(); 5],
            frame: 0.0,
            seq: None,
            tint: -1.0,
            time: 0.0,
            tweaks: MtvTweaks::default(),
            rng: CrtRand::default(),
        };
        m.init();
        m
    }
}

impl MtvEffect {
    /// LightStreak.cpp 0x001300D0 — FrontEndInit: textures and shaders (`mtv_draw`), bars off, no sequence, tint off,
    /// not running; then the tweaks (0x00131130, `gfx_tweaks`)
    pub fn init(&mut self) {
        for b in &mut self.bars {
            b.alpha = 0.0;
        }
        self.seq = None;
        self.tint = -1.0;
        self.enabled = false;
        self.no_glitch = false;
    }

    /// LightStreak.cpp 0x00130310 — run for `seconds`; `no_glitch` starts no bars or images
    pub fn start(&mut self, seconds: f32, no_glitch: bool) {
        self.time = seconds;
        self.enabled = true;
        self.no_glitch = no_glitch;
    }

    /// LightStreak.cpp 0x00130340 — start nothing new; `immediate` also stops drawing what runs
    pub fn stop(&mut self, immediate: bool) {
        self.time = 0.0;
        if immediate {
            self.enabled = false;
        }
    }

    /// LightStreak.cpp 0x00130380 — per drawn frame (spawn chances are per frame, as the original)
    pub fn update(&mut self, dt: f32) {
        let t = self.tweaks;
        self.time -= dt;
        if self.time < 0.0 {
            self.time = 0.0;
        }
        let mut busy = false;
        for b in &mut self.bars {
            if b.alpha > 0.0 {
                busy = true;
                b.x = b.speed * dt + b.x;
                if b.x < 0.0 || b.x > 512.0 {
                    b.alpha = 0.0;
                }
            }
        }
        if self.tint >= 0.0 {
            busy = true;
            self.tint += dt;
            if self.tint >= t.tint_in + t.tint_out {
                self.tint = -1.0;
            }
        }
        if self.seq.is_some() {
            busy = true;
        }
        if self.time > 0.0 {
            if !self.no_glitch {
                for i in 0..5 {
                    if self.bars[i].alpha == 0.0 {
                        let chance = t.line_freq * 0.004f32;
                        if chance as f64 > self.rng.rand_unit() {
                            // the inner search finds a free slot but fills slot i (free as well)
                            if self.bars.iter().any(|b| b.alpha == 0.0) {
                                let b = &mut self.bars[i];
                                b.x = 512.0;
                                let r = (self.rng.rand_unit() * (1.0 - BAR_RAND_MIN as f64) + BAR_RAND_MIN as f64) as f32;
                                b.speed = (0.0 - t.line_speed) * r;
                                b.width = t.line_width;
                                b.alpha = (t.line_alpha * 128.0) * r;
                            }
                        }
                    }
                }
            }
            if !self.no_glitch && self.seq.is_none() {
                let chance = t.image_freq * 0.004f32;
                if chance as f64 > self.rng.rand_unit() {
                    self.seq = Some((self.rng.rand() % 4) as usize);
                    self.frame = 0.0;
                }
            }
            if self.tint < 0.0 {
                let chance = t.tint_freq * 0.004f32;
                if chance as f64 > self.rng.rand_unit() {
                    self.tint = 0.0;
                }
            }
        } else if !busy {
            self.enabled = false;
        }
    }

    /// LightStreak.cpp 0x001306E0 — the frame's draw (FECK2ElementCamera 0x002999B0): update, the bars (0x00130DC0 in
    /// "MTVLines"), the sequence frame's images (sequence 3 frame 2 mirrored), then the tint (0x00130C00)
    pub fn frame(&mut self, dt: f32) -> MtvFrame {
        let mut out = MtvFrame::default();
        if !self.enabled {
            return out;
        }
        self.update(dt);
        let t = self.tweaks;
        for b in &self.bars {
            if b.alpha > 0.0 {
                let c = crate::light_streak::visos_colour([b.alpha as i32 as u8, 0, 0, 0]);
                out.quads.push(MtvQuad { tex: -1, p0: Vec2::new(b.x, 0.0), p1: Vec2::new(b.x + b.width, 512.0), colour: c, uv0: Vec2::ZERO, uv1: Vec2::ONE });
            }
        }
        if let Some(s) = self.seq {
            let seq = &GLITCH_SEQS[s];
            let f = self.frame as i32;
            if f >= seq.frames as i32 {
                self.seq = None;
            } else {
                for k in 0..seq.per_frame as i32 {
                    let g = seq.quads[(k + f * seq.per_frame as i32) as usize];
                    if g.tex >= 0 {
                        let size = (g.size * 0.01f32) * GLITCH_SCALE[g.tex as usize];
                        let a = (128.0 * t.image_alpha) as i32 as u8;
                        let c = crate::light_streak::visos_colour([a, 0x80, 0x80, 0x80]);
                        let (uv0, uv1) = if s == 3 && f == 2 { (Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)) } else { (Vec2::ZERO, Vec2::ONE) };
                        let p1 = Vec2::new((size * 3.0) / 4.0 + g.x, g.y + size);
                        out.quads.push(MtvQuad { tex: g.tex, p0: Vec2::new(g.x, g.y), p1, colour: c, uv0, uv1 });
                    }
                }
                self.frame = dt * t.image_speed + self.frame;
            }
        }
        if self.tint >= 0.0 {
            let mut k = 0.0f32;
            if t.tint_in <= self.tint {
                if t.tint_in + t.tint_out > self.tint {
                    let d = self.tint - t.tint_in;
                    k = 1.0 - d / t.tint_out;
                }
            } else {
                k = self.tint / t.tint_in;
            }
            let grey = Vec4::splat(128.0);
            let mut tint = TINT_COLOUR.extend(1.0);
            let luma = tint.x * TINT_LUMA.x + tint.y * TINT_LUMA.y + tint.z * TINT_LUMA.z;
            if luma > TINT_MIN_LUMA {
                tint *= 128.0 / luma;
                let _desat = if t.flags & 2 != 0 { k * t.desat_intensity } else { 0.0 };
                out.tint = Some(tint_colour(tint, grey, t.tint_intensity * k));
            }
        }
        out
    }
}

/// LightStreak.cpp 0x00130C00 — after FuryInstance 0x0012A4F0 (a·k + b·(1 − k)): each channel / 128 capped at 1 into
/// VisosColour (0x002B1370); the second argument (desat) is unused
pub fn tint_colour(a: Vec4, b: Vec4, k: f32) -> u32 {
    let c = b * (1.0 - k) + a * k;
    let cap = |v: f32| {
        let v = v / 128.0;
        if v <= 1.0 {
            v
        } else {
            1.0
        }
    };
    crate::car_palette::visos_colour(cap(c.w), cap(c.x), cap(c.y), cap(c.z))
}

/// overlay camera, pooled quad entities and materials
#[derive(Default)]
pub(crate) struct MtvDraw {
    camera: Option<Entity>,
    /// "MTVLines" then the 14 glitch textures (shader +160: blend 2, z off) and the tint ("MTVTint" +164: blend 23)
    materials: Vec<Option<Handle<XboxStageMaterial>>>,
    tint: Option<Handle<XboxStageMaterial>>,
    pool: Vec<(Entity, Handle<Mesh>)>,
}

#[derive(Component)]
struct MtvView;

fn mtv_state(blend: u32) -> StageState {
    let mut s = StageState::from_commands(&[0]);
    s.blend_mode = blend;
    s.z_enable = false;
    s.z_write = false;
    s.two_sided = true;
    s
}

/// LightStreak.cpp 0x00130DC0 — the quads (x0, y0, x1, y1, texture, colour, u0, v0, u1, v1 as a 4-vertex strip through
/// Renderer 0x002A9B00, pretransformed) and the tint (FullScreenEffectsManager 0x0015C340 screen quad, oD0 = the
/// colour) of 0x001306E0, run each rendered frame as FECK2ElementCamera 0x002999B0 does
#[allow(clippy::too_many_arguments)]
fn mtv_draw(
    mut commands: Commands,
    time: Res<Time>,
    mut mtv: ResMut<MtvEffect>,
    mut st: Local<MtvDraw>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    mut images: ResMut<Assets<Image>>,
    fe: Query<(&Msaa, Option<&RenderTarget>), (With<FrontEndCamera>, Without<MtvView>)>,
    mut views: Query<(&mut Camera, &mut Msaa, Option<&RenderTarget>), With<MtvView>>,
    mut quads: Query<(&mut Visibility, &mut MeshMaterial3d<XboxStageMaterial>, &mut Mesh3d), Without<MtvView>>,
) {
    let fe = fe.iter().next();
    let frame = if fe.is_some() { mtv.frame(time.delta_secs()) } else { MtvFrame::default() };
    let draws = frame.quads.len() + frame.tint.is_some() as usize;
    if let Some(Ok((mut c, ..))) = st.camera.map(|e| views.get_mut(e)) {
        let on = draws > 0;
        if c.is_active != on {
            c.is_active = on;
        }
    }
    for (e, _) in &st.pool {
        if let Ok((mut v, _, _)) = quads.get_mut(*e) {
            if *v != Visibility::Hidden {
                *v = Visibility::Hidden;
            }
        }
    }
    let Some((msaa, target)) = fe else { return };
    if draws == 0 {
        return;
    }
    let target = target.cloned().unwrap_or_default();
    match st.camera.map(|e| views.get_mut(e)) {
        Some(Ok((_, mut m, t))) => {
            if *m != *msaa {
                *m = *msaa;
            }
            if t.is_none_or(|t| format!("{t:?}") != format!("{target:?}")) {
                commands.entity(st.camera.unwrap()).insert(target);
            }
        }
        _ => {
            let e = commands
                .spawn((
                    Camera3d { depth_load_op: Camera3dDepthLoadOp::Load, ..default() },
                    Camera {
                        order: MTV_ORDER,
                        clear_color: ClearColorConfig::None,
                        output_mode: CameraOutputMode::Write {
                            blend_state: Some(bevy::render::render_resource::BlendState::REPLACE),
                            clear_color: ClearColorConfig::None,
                        },
                        ..default()
                    },
                    target,
                    Projection::Orthographic(OrthographicProjection {
                        scaling_mode: bevy::camera::ScalingMode::Fixed { width: 512.0, height: 512.0 },
                        ..OrthographicProjection::default_3d()
                    }),
                    *msaa,
                    Tonemapping::None,
                    DebandDither::Disabled,
                    Transform::default(),
                    RenderLayers::layer(MTV_LAYER),
                    MtvView,
                ))
                .id();
            st.camera = Some(e);
        }
    }
    if st.materials.is_empty() {
        let mut tex = crate::model_builder::TexSet::default();
        crate::car_render::add_vcd_textures(&mut tex, "\\VFX\\misc.vcd");
        let state = mtv_state(2);
        let names = std::iter::once("mtvlines".to_string()).chain(GLITCH_NAMES.iter().map(|n| format!("fe_glitch_{}", n.to_ascii_lowercase())));
        for n in names {
            let image = tex.image(&n, state.address_u, state.address_v, &mut images);
            if image.is_none() {
                log::warn!("mtv: no texture {n} in \\VFX\\misc.vcd");
            }
            st.materials.push(image.map(|i| {
                materials.add(XboxStageMaterial { params: XboxStageParams::new(&state), texture: Some(i), stencil: None, spec_texture: None, state })
            }));
        }
        let ts = mtv_state(23);
        st.tint = Some(materials.add(XboxStageMaterial { params: XboxStageParams::new(&ts), texture: None, stencil: None, spec_texture: None, state: ts }));
    }
    // draw order = depth order of the blended pass: later draws nearer
    let mut list: Vec<(Vec<(Vec2, Vec2, u32)>, Option<Handle<XboxStageMaterial>>)> = frame
        .quads
        .iter()
        .map(|q| {
            let v = vec![
                (q.p0, q.uv0, q.colour),
                (Vec2::new(q.p1.x, q.p0.y), Vec2::new(q.uv1.x, q.uv0.y), q.colour),
                (Vec2::new(q.p0.x, q.p1.y), Vec2::new(q.uv0.x, q.uv1.y), q.colour),
                (q.p1, q.uv1, q.colour),
            ];
            (v, st.materials.get((q.tex + 1) as usize).cloned().flatten())
        })
        .collect();
    if let Some(c) = frame.tint {
        let v = vec![(Vec2::ZERO, Vec2::ZERO, c), (Vec2::new(512.0, 0.0), Vec2::X, c), (Vec2::new(0.0, 512.0), Vec2::Y, c), (Vec2::splat(512.0), Vec2::ONE, c)];
        list.push((v, st.tint.clone()));
    }
    let n = list.len();
    for (i, (verts, mat)) in list.into_iter().enumerate() {
        let Some(mat) = mat else { continue };
        let z = -1.0 - (n - i) as f32;
        let mesh = screen_mesh(&verts, z);
        if i < st.pool.len() {
            let (e, h) = st.pool[i].clone();
            let touch = crate::set_mesh(&mut meshes, &h, mesh);
            if let Ok((mut v, mut mm, mut m3)) = quads.get_mut(e) {
                if touch {
                    m3.set_changed();
                }
                *v = Visibility::Visible;
                if mm.0 != mat {
                    mm.0 = mat;
                }
            }
        } else {
            let h = meshes.add(mesh);
            let e = commands
                .spawn((
                    Mesh3d(h.clone()),
                    MeshMaterial3d(mat),
                    Transform::default(),
                    Visibility::Visible,
                    bevy::camera::visibility::NoFrustumCulling,
                    RenderLayers::layer(MTV_LAYER),
                ))
                .id();
            st.pool.push((e, h));
        }
    }
}

pub struct MtvEffectPlugin;

impl Plugin for MtvEffectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MtvEffect>().add_systems(PostUpdate, mtv_draw.before(bevy::camera::visibility::VisibilitySystems::CheckVisibility));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_runs_then_stops() {
        let mut m = MtvEffect::default();
        assert!(!m.enabled && m.frame(0.1).quads.is_empty());
        m.start(0.5, false);
        let mut drew = false;
        for _ in 0..10 {
            let f = m.frame(0.05);
            drew |= !f.quads.is_empty() || f.tint.is_some();
        }
        assert!(drew);
        // bars leave the screen, sequences end, the tint fades: the effect turns itself off
        for _ in 0..200 {
            m.frame(0.05);
        }
        assert!(!m.enabled);
    }

    #[test]
    fn no_glitch_only_tints() {
        let mut m = MtvEffect::default();
        m.start(0.1, true);
        for _ in 0..4 {
            let f = m.frame(0.02);
            assert!(f.quads.is_empty());
        }
        m.stop(true);
        assert!(!m.enabled);
    }

    #[test]
    fn tint_peak_colour() {
        // fully in: k = 0.5 → (284.4·0.5 + 64, 94.3·0.5·…): red capped at 1, blue 0.5
        let luma = TINT_COLOUR.x * TINT_LUMA.x + TINT_COLOUR.y * TINT_LUMA.y + TINT_COLOUR.z * TINT_LUMA.z;
        let tint = TINT_COLOUR.extend(1.0) * (128.0 / luma);
        let c = tint_colour(tint, Vec4::splat(128.0), 0.5);
        assert_eq!(c >> 16 & 0xFF, 0xFF);
        assert_eq!(c & 0xFF, 127);
        // k = 0: white (no change under the multiply)
        assert_eq!(tint_colour(tint, Vec4::splat(128.0), 0.0) & 0x00FF_FFFF, 0x00FF_FFFF);
    }
}
