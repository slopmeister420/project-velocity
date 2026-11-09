//! glue (not an original module): the race effects and particles draw through `VertexStream` + `Renderer::Draw`
//! (0x002B8C70 / 0x002A9A80) with shaders built in code (`CShaderManagerBase::CreateShader` 0x002A04C0). Here every
//! frame's vertices are gathered per shader into `EffectDraw` and written into one dynamic mesh per shader.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::image::ImageAddressMode;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::renderer::shader_manager::{d3dcmp, StageState, XboxStageMaterial, XboxStageParams};
use crate::renderer::texture_manager::decode_any_vtf;

/// where a code-built shader's texture comes from: `dword_6F4AC8` = `\VFX\misc.vcd`, `dword_6F4AC4` =
/// `\Particle\PARTTEX.VCD` (GameDataLoading.cpp 0x0009D7B0); `White` = the default texture (0x0011F260)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EffectTexture {
    White,
    Misc(String),
    Particle(String),
    /// `\Scenery\SCENERYT.VCD` (Armco.cpp 0x00011C50 "ArmcoShader")
    Scenery(String),
}

/// the stage settings a code-built shader uses (ShaderManagerBase setters, compiled by
/// `CShaderManager::CompileStage` 0x002A3320 into the same `eMS_*` records the data uses)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EffectShader {
    pub texture: EffectTexture,
    /// SetBlendMode (stage +16)
    pub blend: u32,
    /// SetZState (stage +56 test, +57 write)
    pub z_test: bool,
    pub z_write: bool,
    /// 0x002A1760 (stage +84): `eMS_DepthBias`
    pub depth_bias: i16,
    /// SetAddressWrap (stage +72 / +73, false = clamp)
    pub wrap_u: bool,
    pub wrap_v: bool,
    /// 0x002A0F40 alpha test function + 1 (0 = off) and 0x002A0FC0 reference
    pub alpha_func: u32,
    pub alpha_ref: u8,
    /// stencil EQUAL ref 8 (0x002A10C0(4) + 0x002A1140(8)): drawn by RainSurface's puddle camera on
    /// `rain_surface::PUDDLE_LAYER`, kept to the road pixels the "roaddetail" pass marked
    pub road_stencil: bool,
    /// SetColourOp (`MapTextureOp` 0x002A3230): 0 = the reset MODULATE, 4 MODULATE2X, 5 MODULATE4X (rgb scaled and
    /// saturated, alpha MODULATE)
    pub colour_op: u8,
}

impl EffectShader {
    /// a shader with the manager's defaults (wrap, z test and write on, no alpha test)
    pub fn new(texture: EffectTexture, blend: u32) -> Self {
        Self {
            texture,
            blend,
            z_test: true,
            z_write: true,
            depth_bias: 0,
            wrap_u: true,
            wrap_v: true,
            alpha_func: 0,
            alpha_ref: 0,
            road_stencil: false,
            colour_op: 0,
        }
    }

    /// SetColourOp(op) on stage 0
    pub fn colour_op(mut self, op: u8) -> Self {
        self.colour_op = op;
        self
    }

    /// the stage params of the colour op (`model_builder::STAGE_MODE_SCALED`: rgb × 2 / × 4, saturated)
    fn params(&self, state: &StageState) -> XboxStageParams {
        let mut p = XboxStageParams::new(state);
        let s = match self.colour_op {
            4 => 2.0,
            5 => 4.0,
            _ => return p,
        };
        p.colour = Vec4::new(s, s, s, 1.0);
        p.arg.y = crate::model_builder::STAGE_MODE_SCALED;
        p
    }

    /// stencil func EQUAL ref 8 (RainSurface bit 8)
    pub fn road_stencil(mut self) -> Self {
        self.road_stencil = true;
        self
    }

    /// SetZState(test, write)
    pub fn z(mut self, test: bool, write: bool) -> Self {
        self.z_test = test;
        self.z_write = write;
        self
    }

    pub fn bias(mut self, b: i16) -> Self {
        self.depth_bias = b;
        self
    }

    pub fn wrap(mut self, u: bool, v: bool) -> Self {
        self.wrap_u = u;
        self.wrap_v = v;
        self
    }

    pub fn alpha_test(mut self, func: u32, r: u8) -> Self {
        self.alpha_func = func;
        self.alpha_ref = r;
        self
    }

    fn stage_state(&self) -> StageState {
        let mut s = StageState::from_commands(&[0]);
        s.blend_mode = self.blend;
        s.z_enable = true;
        s.z_func = if self.z_test { d3dcmp::LESSEQUAL } else { d3dcmp::ALWAYS };
        s.z_write = self.z_write;
        s.depth_bias = self.depth_bias;
        s.address_u = if self.wrap_u { 1 } else { 3 };
        s.address_v = if self.wrap_v { 1 } else { 3 };
        if self.alpha_func != 0 {
            s.alpha_test = true;
            s.alpha_func = crate::renderer::shader_manager::ALPHA_FUNC_TABLE[((self.alpha_func - 1) & 7) as usize];
            s.alpha_ref = self.alpha_ref as u32;
        }
        s.two_sided = true;
        if self.road_stencil {
            s.stencil_road = crate::renderer::shader_manager::STENCIL_ROAD_TEST;
        }
        s
    }
}

/// one vertex: position (game space), D3DCOLOR (0xAARRGGBB), uv
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EffectVertex {
    pub pos: Vec3,
    pub argb: u32,
    pub uv: Vec2,
}

impl EffectVertex {
    pub fn new(pos: Vec3, argb: u32, uv: Vec2) -> Self {
        Self { pos, argb, uv }
    }
}

/// effect time: particles, sparks and slipstreams stop while paused (`GameBridge__97E90`); `scale` = time stretch
#[derive(Resource, Clone, Copy, Debug)]
pub struct EffectClock {
    pub paused: bool,
    pub scale: f32,
}

impl Default for EffectClock {
    fn default() -> Self {
        Self { paused: false, scale: 1.0 }
    }
}

/// `EffectDraw::view` of a world-space batch every view draws, the mirror too (GameManager 0x000B9010 calls it
/// ungated, and nothing in it depends on the camera: zone trails, slipstream ribbons)
pub const ALL_VIEWS: usize = usize::MAX;

/// this frame's effect triangles, per shader, in submission order
#[derive(Resource, Default)]
pub struct EffectDraw {
    pub batches: Vec<(EffectShader, Vec<EffectVertex>)>,
    /// the view of each batch (parallel to `batches`): Some(p) = built with player p's camera while split, drawn
    /// in that view only (`view_render::view_layer`); None = world-space, seen by every view
    pub views: Vec<Option<usize>>,
    /// the view producers are building for (set around a camera-built pass with `set_view`)
    pub view: Option<usize>,
}

impl EffectDraw {
    /// camera-built batches from here on belong to `view` (`ViewCam::tag`); None = back to world-space
    pub fn set_view(&mut self, view: Option<usize>) {
        self.view = view;
    }

    fn batch(&mut self, sh: &EffectShader) -> &mut Vec<EffectVertex> {
        if self.views.len() != self.batches.len() {
            self.views.resize(self.batches.len(), None);
        }
        let v = self.view;
        let i = match self.batches.iter().zip(&self.views).position(|((k, _), w)| k == sh && *w == v) {
            Some(i) => i,
            None => {
                self.batches.push((sh.clone(), Vec::new()));
                self.views.push(v);
                self.batches.len() - 1
            }
        };
        &mut self.batches[i].1
    }

    /// triangle list
    pub fn triangles(&mut self, sh: &EffectShader, v: impl IntoIterator<Item = EffectVertex>) {
        self.batch(sh).extend(v);
    }

    /// triangle strip (VertexStream strips, primitive 14)
    pub fn strip(&mut self, sh: &EffectShader, v: &[EffectVertex]) {
        let b = self.batch(sh);
        for i in 2..v.len() {
            b.extend([v[i - 2], v[i - 1], v[i]]);
        }
    }

    /// quads given as 4 corners each (VertexStream quad lists)
    pub fn quad(&mut self, sh: &EffectShader, q: [EffectVertex; 4]) {
        self.batch(sh).extend([q[0], q[1], q[2], q[0], q[2], q[3]]);
    }
}

/// this frame's flush put a batch on `view_render::POST_VIEW_LAYER` (the post camera runs only then)
#[derive(Resource, Default)]
pub struct PostViewUsed(pub bool);

#[derive(Resource, Default)]
struct EffectRenderCache {
    materials: HashMap<EffectShader, Handle<XboxStageMaterial>>,
    textures: HashMap<(EffectTexture, bool, bool), Handle<Image>>,
    /// mesh slots: entity, mesh, render layer (0, the puddle camera's, the front-end post-reflection camera's)
    meshes: Vec<(Entity, Handle<Mesh>, usize)>,
    misc: Option<Option<visos::file::Vcd>>,
    part: Option<Option<visos::file::Vcd>>,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectSet {
    /// producers write `EffectDraw`
    Build,
    /// meshes are rewritten
    Flush,
}

/// the producers' fixed order inside `EffectSet::Build` (chained). Batches draw in submission order and producers
/// sharing `EffectDraw` / `ElParticleSystemManager` / `Particles` otherwise run in scheduler order, which changed
/// with unrelated systems (london_wet_cams_06 sparks). Order of GameManager 0x000B9010: skid marks 0x00139410,
/// cat's eyes 0x0011CF70, trail blazer 0x0014E710, sparks 0x00142120, barriers 0x00109080, flares 0x00128590 /
/// TrackFX 0x0014A5B0, particles 0x002D3460, CarDamageFX 0x00079730 + elParticle 0x002C5B70; the front end's
/// shadows come first (FECK2ElementCamera 0x002999B0: shadows, flares, particles)
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectOrder {
    Shadows,
    RaceFx,
    CatsEye,
    Trail,
    Barriers,
    TrackFx,
    Puddles,
    Particles,
    ElRain,
    ElDamage,
    ElMist,
    ElSpray,
    ElParticle,
}

pub struct EffectDrawPlugin;

impl Plugin for EffectDrawPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EffectDraw>()
            .init_resource::<EffectClock>()
            .init_resource::<EffectRenderCache>()
            .init_resource::<PostViewUsed>()
            .configure_sets(PostUpdate, (EffectSet::Build, EffectSet::Flush).chain().before(bevy::transform::TransformSystems::Propagate))
            .configure_sets(
                PostUpdate,
                {
                    use EffectOrder::*;
                    (Shadows, RaceFx, CatsEye, Trail, Barriers, TrackFx, Puddles, Particles, ElRain, ElDamage, ElMist, ElSpray, ElParticle)
                        .chain()
                        .in_set(EffectSet::Build)
                },
            )
            .add_systems(PostUpdate, flush_effects.in_set(EffectSet::Flush));
    }
}

fn texture_image(cache: &mut EffectRenderCache, images: &mut Assets<Image>, t: &EffectTexture, wu: bool, wv: bool) -> Handle<Image> {
    if let Some(h) = cache.textures.get(&(t.clone(), wu, wv)) {
        return h.clone();
    }
    let decoded = match t {
        EffectTexture::White => None,
        EffectTexture::Misc(n) => cache
            .misc
            .get_or_insert_with(|| visos::file::Vcd::open("\\VFX\\misc.vcd").ok())
            .as_ref()
            .and_then(|v| v.get(&format!("{n}.vtf")))
            .and_then(|d| decode_any_vtf(d).ok()),
        EffectTexture::Scenery(n) => visos::file::Vcd::open("\\Scenery\\SCENERYT.VCD")
            .ok()
            .and_then(|v| v.get(&format!("{n}.vtf")).and_then(|d| decode_any_vtf(d).ok())),
        EffectTexture::Particle(n) => cache
            .part
            .get_or_insert_with(|| visos::file::Vcd::open("\\Particle\\PARTTEX.VCD").ok())
            .as_ref()
            .and_then(|v| v.get(&format!("{n}.vtf")))
            .and_then(|d| decode_any_vtf(d).ok()),
    };
    let decoded = decoded.unwrap_or_else(|| {
        if *t != EffectTexture::White {
            log::warn!("effect texture {t:?} not found");
        }
        crate::renderer::texture_manager::DecodedTexture { width: 1, height: 1, levels: vec![vec![255; 4]] }
    });
    let mode = |w: bool| if w { ImageAddressMode::Repeat } else { ImageAddressMode::ClampToEdge };
    let h = images.add(crate::model_builder::make_image(&decoded, mode(wu), mode(wv)));
    cache.textures.insert((t.clone(), wu, wv), h.clone());
    h
}

/// write each batch into its mesh; unused meshes are hidden
#[allow(clippy::too_many_arguments)]
fn flush_effects(
    mut commands: Commands,
    mut draw: ResMut<EffectDraw>,
    mut cache: ResMut<EffectRenderCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    mut images: ResMut<Assets<Image>>,
    lights: Option<Res<crate::track_render::TrackLights>>,
    fse: Option<Res<crate::full_screen_effects_manager_pc_xbox::FullScreenEffects>>,
    mut vis: Query<(&mut Visibility, &mut MeshMaterial3d<XboxStageMaterial>, &mut Mesh3d)>,
    ab: Option<Res<crate::gfx_ab::GfxAb>>,
    fe_split: Option<Res<crate::rain_surface::FeReflectionSplit>>,
    split: Option<Res<crate::view_render::SplitScreen>>,
    mut post_used: ResMut<PostViewUsed>,
) {
    let mut any_post = false;
    let fog = lights.map(|l| l.fog).unwrap_or(Vec4::ZERO);
    let back_buffer = fse.and_then(|f| f.back_buffer());
    let split_on = split.as_ref().is_some_and(|s| s.per_view());
    // the mirror (flags 0x08000620) draws only its own batches (`MIRROR_VIEW_TAG`) and the `ALL_VIEWS` ones
    let mirror_on = split.as_ref().is_some_and(|s| s.mirror.is_some());
    let batches = std::mem::take(&mut draw.batches);
    let mut views = std::mem::take(&mut draw.views);
    views.resize(batches.len(), None);
    draw.view = None;
    let mut used = 0;
    for ((sh, verts), mut view) in batches.into_iter().zip(views) {
        // world-space batches the mirror draws as well (`ALL_VIEWS`) and the after-the-loop ones (`POST_VIEW_TAG`)
        let all = view == Some(ALL_VIEWS);
        let post = view == Some(crate::view_render::POST_VIEW_TAG);
        if all || post {
            view = None;
        }
        if verts.is_empty() || ab.as_ref().is_some_and(|a| a.effect_off(&sh.texture)) {
            continue;
        }
        let mat = match cache.materials.get(&sh) {
            Some(m) => m.clone(),
            None => {
                let image = texture_image(&mut cache, &mut images, &sh.texture, sh.wrap_u, sh.wrap_v);
                let state = sh.stage_state();
                let m = materials.add(XboxStageMaterial { params: sh.params(&state), texture: Some(image), stencil: None, spec_texture: None, state });
                cache.materials.insert(sh.clone(), m.clone());
                m
            }
        };
        if let Some(m) = materials.get(&mat) {
            let stencil = if sh.road_stencil { back_buffer.clone() } else { None };
            if m.params.fog != fog || m.stencil != stencil {
                if let Some(mut m) = materials.get_mut(&mat) {
                    m.params.fog = fog;
                    m.stencil = stencil;
                }
            }
        }
        let mesh = build_mesh(&verts);
        // in the front end with the floor reflection split, the effects draw after the paste (FECK2ElementCamera
        // 0x002999B0 draws shadows, flares and particles after RainSurface 0x00137C50)
        let layer = if sh.road_stencil {
            crate::rain_surface::PUDDLE_LAYER
        } else if post {
            any_post = true;
            crate::view_render::POST_VIEW_LAYER
        } else if let Some(p) = view.filter(|_| split_on) {
            // GameManager 0x000B9010 draws a camera-built effect inside its own view only
            crate::view_render::view_layer(p)
        } else if mirror_on && !all {
            crate::view_render::view_layer(0)
        } else if fe_split.as_ref().is_some_and(|s| s.0) {
            crate::rain_surface::FE_POST_LAYER
        } else {
            0
        };
        if used == cache.meshes.len() {
            let h = meshes.add(mesh);
            let e = commands
                .spawn((Mesh3d(h.clone()), MeshMaterial3d(mat.clone()), Transform::default(), Visibility::Inherited, NoFrustumCulling, RenderLayers::layer(layer)))
                .id();
            cache.meshes.push((e, h, layer));
        } else {
            let (e, h, old) = cache.meshes[used].clone();
            // unchanged meshes are skipped (`VELOCITY_PERF_OFF=effmesh` = always written); a written one re-extracts
            // the entity (details "Performance stage 10")
            let touch = if effmesh_opt() {
                crate::set_mesh(&mut meshes, &h, mesh)
            } else {
                if let Some(mut m) = meshes.get_mut(&h) {
                    *m = mesh;
                }
                false
            };
            if old != layer {
                commands.entity(e).insert(RenderLayers::layer(layer));
                cache.meshes[used].2 = layer;
            }
            if let Ok((mut v, mut mm, mut m3)) = vis.get_mut(e) {
                if touch {
                    m3.set_changed();
                }
                *v = Visibility::Inherited;
                if mm.0 != mat {
                    mm.0 = mat.clone();
                }
            }
        }
        used += 1;
    }
    if post_used.0 != any_post {
        post_used.0 = any_post;
    }
    for (e, _, _) in cache.meshes.iter().skip(used) {
        if let Ok((mut v, _, _)) = vis.get_mut(*e) {
            if *v != Visibility::Hidden {
                *v = Visibility::Hidden;
            }
        }
    }
}

fn effmesh_opt() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| !crate::golden::perf_off("effmesh"))
}

fn build_mesh(v: &[EffectVertex]) -> Mesh {
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD);
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, v.iter().map(|x| x.pos.to_array()).collect::<Vec<_>>());
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; v.len()]);
    m.insert_attribute(Mesh::ATTRIBUTE_UV_0, v.iter().map(|x| x.uv.to_array()).collect::<Vec<_>>());
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, v.iter().map(|x| argb_to_rgba(x.argb)).collect::<Vec<_>>());
    m.insert_indices(Indices::U32((0..v.len() as u32).collect()));
    m
}

/// D3DCOLOR to the raw (gamma) rgba the stage shader takes
pub fn argb_to_rgba(c: u32) -> [f32; 4] {
    let b = |s: u32| ((c >> s) & 0xFF) as f32 / 255.0;
    [b(16), b(8), b(0), b(24)]
}

/// CatsEyeRender.cpp 0x0011DE40 — an engine colour (bytes r, g, b, a with 0x80 = 1.0, alpha 0x40 = 1.0) to D3DCOLOR
pub fn engine_colour(c: u32) -> u32 {
    let r = c & 0xFF;
    let g = (c >> 8) & 0xFF;
    let b = (c >> 16) & 0xFF;
    let a = c >> 24;
    let d = |v: u32| if v <= 0x7F { 2 * v } else { 0xFF };
    let a = if a <= 0x3F { 4 * a } else { 0xFF };
    d(b) | (d(g) << 8) | (d(r) << 16) | (a << 24)
}
