//! Render\Pc_xbox\FullScreenEffectsManager_PC_XBOX.cpp — the full-screen pass layer (object `dword_738570`): its own
//! eight vertex / fragment programs (`Shaders\Copy|Blur|BloomStart|Bloom|Colour|PointBlur|PointBlurWithAlpha|
//! GrayScale`, 0x0015B2F0), the off-screen render targets (two half-size and two quarter-size "OffScreenBuffer"s,
//! linear A8R8G8B8, 0x0015BAE0) and the textured screen quad every effect draws through (0x0015C240 / 0x0015C3C0).
//!
//! The original draws into the back buffer and samples it as a texture (`RenderTargetManager` 0x002EF030). Here the
//! world camera (`TrackCamera`, and with it the sky passes) is redirected into a "BackBuffer" image of the final
//! target's size, every queued pass gets its own camera (orders 2..8) with a clip-space quad, and a last Copy pass
//! presents the back buffer to the final target at order 9 (the HUD cameras draw from order 10). Passes drawn into
//! the back buffer share the world camera's colour and depth textures (same target and msaa), which keeps the
//! original's z-tested composites.
//!
//! Images keep the Xbox's gamma-space values: targets are `Rgba8UnormSrgb` (bevy stores gamma), programs re-encode
//! what they sample from them (`tex[s].w`), as the stage shaders decode their output.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::camera::{Camera3dDepthLoadOp, RenderTarget};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ColorWrites, RenderPipelineDescriptor, SpecializedMeshPipelineError, TextureFormat};
use bevy::shader::Shader;
use bevy::window::PrimaryWindow;
use std::sync::OnceLock;

use crate::global_mat_input_resolver::gmi;
use crate::renderer::shader_manager::{apply_glow_stencil, apply_stage_pipeline_state, d3dcmp, get_blend_mode, is_glow_stencil_view, GLOW_STENCIL_VIEW};
use bevy::camera::CameraOutputMode;
use bevy::render::render_resource::{BlendComponent, BlendFactor, BlendOperation, BlendState};
use crate::renderer::shader_manager::CAR_STENCIL_VIEW;
use crate::renderer::shader_program_manager::{ProgramParams, VERTEX_WRAPPER};
use crate::renderer::xgraphics_shader::{self, COMMON_WGSL};
use crate::sky_render::SkyCamera;
use crate::track_render::TrackCamera;

/// `off_687590` — vertex program per FSE program index
pub const FSE_VERTEX_PROGRAMS: [&str; 8] = ["Copy", "Blur", "BloomStart", "Blur", "Copy", "PointBlur", "PointBlurWithAlpha", "GrayScale"];
/// `off_6875B0` — fragment program per FSE program index
pub const FSE_FRAGMENT_PROGRAMS: [&str; 8] = ["Copy", "Blur", "BloomStart", "Bloom", "Colour", "PointBlur", "PointBlurWithAlpha", "GrayScale"];

pub mod prog {
    pub const COPY: usize = 0;
    pub const BLUR: usize = 1;
    pub const BLOOM_START: usize = 2;
    pub const BLOOM: usize = 3;
    pub const COLOUR: usize = 4;
    pub const POINT_BLUR: usize = 5;
    pub const POINT_BLUR_WITH_ALPHA: usize = 6;
    pub const GRAY_SCALE: usize = 7;
    /// not an FSE pair: the game's "Danger" vertex / fragment programs (GameShaderProgramManager vp 7 / fp 5) that
    /// DangerZoneRender 0x001202F0 binds to its overlay shader (+12); run through the FSE pass layer here
    pub const DANGER: usize = 8;
}

/// file stem of `prog::DANGER` (both `.vxu` and `.pxu`)
pub const DANGER_PROGRAM: &str = "Danger";

/// one vertex of a screen strip drawn through GameShaderProgramManager_PC_XBOX 0x0015E990 / 0x0015E7B0: position in
/// 0..1 of the target (the original's 512² space), the stream's uv as written (texels for a linear source when the
/// program passes v9 through), D3DCOLOR
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FseVertex {
    pub pos: Vec2,
    pub uv: Vec2,
    pub colour: u32,
}

/// GameShaderProgramManager_PC_XBOX.cpp 0x0015EA60: the stream holds at most 1000 (0x3E8) vertices, the rest are
/// dropped
pub const MAX_STRIP_VERTICES: usize = 1000;

/// 0x0015B2F0 `CarShaderManager_PC_XBOX__15B160` calls: (global input, vertex constant register)
pub fn vertex_bindings(p: usize) -> &'static [(usize, i32)] {
    match p {
        0 | 7 => &[(gmi::SOURCE_TEXTURE_SIZE, 20)],
        1 | 3 => &[(gmi::BLUR_AMOUNT, 0), (gmi::SOURCE_TEXTURE_SIZE, 1)],
        // Danger.vxu: c0 matWorldViewProj (identity rows for the screen strip, `bind_pass`), c4 texture0Size
        8 => &[(gmi::SOURCE_TEXTURE_SIZE, 4)],
        5 | 6 => &[
            (gmi::VIEWPORT_SIZE, 0),
            (gmi::INVERSE_VIEWPORT_SIZE, 1),
            (gmi::INTENSITY, 2),
            (gmi::FOCUS_POINT, 3),
            (gmi::SOURCE_TEXTURE_SIZE, 20),
        ],
        _ => &[],
    }
}

/// 0x0015B2F0 `CarShaderManager_PC_XBOX__15B280` calls: (global input, texture stage / pixel constant)
pub fn fragment_bindings(p: usize) -> &'static [(usize, i32)] {
    match p {
        0 | 7 => &[(gmi::SOURCE_TEXTURE, 0), (gmi::INTENSITY, 0)],
        1 | 3 | 5 => &[(gmi::SOURCE_TEXTURE, 0), (gmi::SOURCE_TEXTURE, 1), (gmi::SOURCE_TEXTURE, 2), (gmi::SOURCE_TEXTURE, 3)],
        2 => &[(gmi::INTENSITY, 0)],
        4 => &[(gmi::CONSTANT_COLOUR, 0)],
        6 => &[
            (gmi::INTENSITY, 0),
            (gmi::SOURCE_TEXTURE, 0),
            (gmi::ALPHA_TEXTURE, 1),
            (gmi::SOURCE_TEXTURE, 2),
            (gmi::SOURCE_TEXTURE, 3),
        ],
        // Danger.pxu (fragment_inputs fp 5): c10 intensity, t1 = global 11 — DangerZoneRender points SourceTexture at
        // "spec4Inv" while stage 0 keeps the shader's texture (the back buffer); carried in the alpha slot here
        8 => &[(gmi::INTENSITY, 10), (gmi::ALPHA_TEXTURE, 1)],
        _ => &[],
    }
}

/// `dword_6875D4` — blur offset of the bloom ping-pong (0x0015C770)
pub const BLUR_STEP: f32 = f32::from_bits(0x3b656042);
/// `unk_687580` — default source / destination rectangle
pub const FULL_RECT: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

/// `dword_77FD9C` — the quad colour every effect passes (white)
pub const WHITE: u32 = 0xFFFF_FFFF;

pub const OFF_SCREEN_HALF: u32 = 1;
pub const OFF_SCREEN_QUARTER: u32 = 2;

/// what a pass reads or writes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FseImage {
    /// the redirected back buffer
    BackBuffer,
    /// buffer slot: 0/1 half size (+0xCC), 2/3 quarter size (+0xD0)
    OffScreen(usize),
    /// a render target of its own (RenderTargetManager 0x002EEA10, e.g. the dynamic "Envmap") or a plain texture
    /// read as a source (`FullScreenEffects::add_target`)
    Target(usize),
}

/// a registered target / texture: `srgb` = bevy srgb storage holding gamma values (the redirected buffers), else
/// the gamma values are stored as they are (`Rgba8Unorm`); `linear` = addressed in texels (an xbox linear texture,
/// every render target), else 0..1
#[derive(Clone, Debug)]
struct FseTarget {
    image: Handle<Image>,
    size: UVec2,
    srgb: bool,
    linear: bool,
}

/// fixed-function stage 0 of a pass drawn without a pixel program (`FseShader.ffp`)
pub const FFP_NONE: u8 = 0;
/// colour MODULATE(diffuse, texture), alpha SELECTARG1(diffuse)
pub const FFP_DIFFUSE_ALPHA: u8 = 1;
/// CreateShader's reset (ShaderManagerBase 0x002A0AA0): colour and alpha MODULATE(texture, diffuse)
pub const FFP_MODULATE: u8 = 2;
/// colour MODULATE2X(texture, diffuse) (SetColourOp 4 -> MapTextureOp 0x002A3230 = D3DTOP_MODULATE2X), alpha MODULATE
pub const FFP_MODULATE2X: u8 = 3;

/// the shader a pass is drawn with (only blend mode, z test and the stage ops matter for a screen quad)
#[derive(Clone, Copy, Debug)]
pub struct FseShader {
    pub blend: u32,
    pub z_test: bool,
    /// D3D z func of the test (`d3dcmp`; SetZState default LESSEQUAL)
    pub z_func: u32,
    /// drawn without programs (0x0015C3C0 straight after the shader, as DOF does): stage 0 = the shader's ops
    /// (`FFP_*`; `FFP_NONE` = the pass program's pixel shader)
    pub ffp: u8,
    /// stencil use (`FSE_STENCIL_*`): the car bit 4 lives in back-buffer alpha after the bloom (see `car_stencil`)
    pub stencil: u8,
}

/// no stencil test (CreateShader default: ALWAYS)
pub const FSE_STENCIL_NONE: u8 = 0;
/// alpha-only write of 0 over the target: the cleared stencil the car stencil camera replays into
pub const FSE_STENCIL_CLEAR: u8 = 1;
/// func NOTEQUAL ref 4 mask 4 ("ZoneWobble" 0x00162B60, "MotionBlurStencilToScreenBlit" 0x00160120): not over the
/// cars — the fragment is dropped where the back buffer's alpha holds the car bit
pub const FSE_STENCIL_NOT_CAR: u8 = 2;
/// func EQUAL ref 8 mask 8 ("RefPaste", ProjectedLightBeam 0x00162280): only over the road pixels RainSurface's road
/// camera marked (back-buffer alpha 1..254 / 255, `shader_manager::STENCIL_ROAD_WRITE`); colour only, the alpha
/// markers are kept
pub const FSE_STENCIL_ROAD: u8 = 3;

/// road camera slots (`FsePass.road_scene`): the "roaddetail" redraw (TrackFX 0x0014D490) and the puddles
/// (0x00131F50), each a camera of its own so the puddles can test the marks the first one left
pub const ROAD_SCENE_DETAIL: usize = 0;
pub const ROAD_SCENE_PUDDLES: usize = 1;
/// the rear-view mirror's slots (`view_render::MIRROR_SLOTS`: its 4 sky passes, then its world camera)
pub const ROAD_SCENE_MIRROR: usize = 2;
/// the after-the-loop slot (`view_render::PostViewCamera`: flash records over every view and the mirror)
pub const ROAD_SCENE_POST: usize = ROAD_SCENE_MIRROR + crate::view_render::MIRROR_SLOTS;
const SCENE_SLOTS: usize = ROAD_SCENE_POST + 1;

impl FseShader {
    /// "Default" (+272, 0x0015BD20): blend 1, z off
    pub const DEFAULT: Self = Self::new(1, false);

    pub const fn new(blend: u32, z_test: bool) -> Self {
        Self { blend, z_test, z_func: d3dcmp::LESSEQUAL, ffp: FFP_NONE, stencil: FSE_STENCIL_NONE }
    }
}

/// one `FullScreenEffectsManager_PC_XBOX__15C240` draw
#[derive(Clone, Debug)]
pub struct FsePass {
    pub target: FseImage,
    pub source: FseImage,
    pub shader: FseShader,
    pub program: usize,
    pub src_rect: [f32; 4],
    pub dst_rect: [f32; 4],
    /// D3D z of the quad (0x0015E740)
    pub z: f32,
    /// D3DCOLOR of the quad vertices
    pub colour: u32,
    /// global vectors at draw time (`GlobalMatInputResolver__12D090`)
    pub vectors: [Vec4; gmi::COUNT],
    /// `AlphaTexture` (global 13)
    pub alpha: Option<Handle<Image>>,
    /// vertex constants written straight to the device (0x00161090), register -> value
    pub vs_consts: Vec<(i32, Vec4)>,
    /// a screen strip instead of the quad (0x0015E7B0 draws)
    pub verts: Option<Vec<FseVertex>>,
    /// per strip vertex D3D z (0x0015E740 before each 0x0015E9E0 vertex); `None` = `z` for all
    pub vert_z: Option<Vec<f32>>,
    /// not a quad: the car stencil camera's slot (see `FullScreenEffects::car_stencil`)
    pub scene: bool,
    /// not a quad: a RainSurface road camera's slot (`ROAD_SCENE_*`, see `FullScreenEffects::road_scene`)
    pub road_scene: Option<usize>,
    /// drawn before the world camera (`FullScreenEffects::set_early`: the dynamic envmap, which the original renders
    /// at the start of the car pass, before any car is drawn)
    pub early: bool,
    /// the target's viewport (x, y, w, h fraction): the view's on the back buffer, the whole target otherwise
    pub viewport: [f32; 4],
}

#[derive(Clone, Debug)]
struct OffScreenBuffer {
    used: bool,
    image: Handle<Image>,
    size: UVec2,
}

/// the manager (`dword_738570`) and this frame's pass list
#[derive(Resource)]
pub struct FullScreenEffects {
    /// a world camera is redirected and the passes run
    pub active: bool,
    /// global input vectors as the effects set them (indexed by `gmi`)
    pub vectors: [Vec4; gmi::COUNT],
    /// vertex constants set through 0x00161090; persist like device registers
    pub vs_consts: Vec<(i32, Vec4)>,
    pub alpha_texture: Option<Handle<Image>>,
    passes: Vec<FsePass>,
    buffers: Vec<OffScreenBuffer>,
    back_buffer: Option<Handle<Image>>,
    size: UVec2,
    final_target: Option<RenderTarget>,
    /// world camera msaa and projection near plane (bevy reverse z)
    msaa: Msaa,
    near: f32,
    /// this frame: the car stencil is queued / the whole stencil was cleared before (danger overlay)
    stencil_built: bool,
    stencil_cleared: bool,
    /// camera order of this frame's car stencil slot
    stencil_order: Option<isize>,
    /// camera orders of this frame's road slots (`ROAD_SCENE_*`)
    road_orders: [Option<isize>; SCENE_SLOTS],
    /// registered render targets / textures (`FseImage::Target`)
    targets: Vec<FseTarget>,
    /// passes queued now go before the world camera
    early: bool,
    /// the back buffer's viewport (RenderTargetManager 0x002EEF10, set per view by GameManager 0x000B3C30)
    viewport: [f32; 4],
}

impl Default for FullScreenEffects {
    fn default() -> Self {
        Self {
            active: false,
            vectors: [Vec4::ZERO; gmi::COUNT],
            vs_consts: Vec::new(),
            alpha_texture: None,
            passes: Vec::new(),
            buffers: Vec::new(),
            back_buffer: None,
            size: UVec2::ZERO,
            final_target: None,
            msaa: Msaa::default(),
            near: 0.5,
            stencil_built: false,
            stencil_cleared: false,
            stencil_order: None,
            road_orders: [None; SCENE_SLOTS],
            targets: Vec::new(),
            early: false,
            viewport: FULL_RECT,
        }
    }
}

impl FullScreenEffects {
    /// back buffer size in pixels (0 when not redirected)
    pub fn size(&self) -> UVec2 {
        self.size
    }

    /// the image the world camera draws into while redirected
    pub fn back_buffer(&self) -> Option<Handle<Image>> {
        self.back_buffer.clone()
    }

    pub fn set_vector(&mut self, id: usize, v: Vec4) {
        self.vectors[id] = v;
    }

    /// GameShaderProgramManager_PC_XBOX.cpp 0x00161090 — `SetVertexShaderConstant(reg, v, 1)`
    pub fn set_vertex_constant(&mut self, reg: i32, v: Vec4) {
        self.vs_consts.retain(|(r, _)| *r != reg);
        self.vs_consts.push((reg, v));
    }

    /// FullScreenEffectsManager_PC_XBOX.cpp 0x0015C920 — first free buffer of a kind (1 half, 2 quarter)
    pub fn get_off_screen_buffer(&mut self, kind: u32) -> Option<FseImage> {
        if !self.active {
            return None;
        }
        let range = match kind {
            OFF_SCREEN_HALF => 0..2,
            OFF_SCREEN_QUARTER => 2..4,
            _ => return None,
        };
        for i in range {
            if let Some(b) = self.buffers.get_mut(i) {
                if !b.used {
                    b.used = true;
                    return Some(FseImage::OffScreen(i));
                }
            }
        }
        None
    }

    /// FullScreenEffectsManager_PC_XBOX.cpp 0x0015CA70
    pub fn release_off_screen_buffer(&mut self, b: FseImage) {
        if let FseImage::OffScreen(i) = b {
            if let Some(b) = self.buffers.get_mut(i) {
                b.used = false;
            }
        }
    }

    /// TextureManager.cpp 0x0029F7F0 on a render target's texture: its size in pixels (0 when missing)
    pub fn image_size(&self, i: FseImage) -> UVec2 {
        self.image(i).map(|(_, s)| s).unwrap_or(UVec2::ZERO)
    }

    fn image(&self, i: FseImage) -> Option<(Handle<Image>, UVec2)> {
        match i {
            FseImage::BackBuffer => self.back_buffer.clone().map(|h| (h, self.size)),
            FseImage::OffScreen(k) => self.buffers.get(k).map(|b| (b.image.clone(), b.size)),
            FseImage::Target(k) => self.targets.get(k).map(|t| (t.image.clone(), t.size)),
        }
    }

    /// (linear, srgb) of an image: the redirected buffers are linear render targets in srgb storage
    fn image_flags(&self, i: FseImage) -> (bool, bool) {
        match i {
            FseImage::Target(k) => self.targets.get(k).map_or((true, true), |t| (t.linear, t.srgb)),
            _ => (true, true),
        }
    }

    /// RenderTargetManager.cpp 0x002EEA10 / a texture handle for the passes: register `image` (see `FseTarget`)
    pub fn add_target(&mut self, image: Handle<Image>, size: UVec2, srgb: bool, linear: bool) -> FseImage {
        self.targets.push(FseTarget { image, size, srgb, linear });
        FseImage::Target(self.targets.len() - 1)
    }

    /// replace a registered target's image (resized source)
    pub fn set_target(&mut self, t: FseImage, image: Handle<Image>, size: UVec2) {
        if let FseImage::Target(k) = t {
            if let Some(e) = self.targets.get_mut(k) {
                e.image = image;
                e.size = size;
            }
        }
    }

    /// passes queued from now on run before the world camera (`FsePass.early`)
    pub fn set_early(&mut self, on: bool) {
        self.early = on;
    }

    /// RenderTargetManager.cpp 0x002EEF10 on the back buffer (GameManager 0x000B3C30 per view): later passes into the back
    /// buffer draw inside `rect` (x, y, w, h fraction); off-screen buffers keep their whole surface
    pub fn set_viewport(&mut self, rect: [f32; 4]) {
        self.viewport = rect;
    }

    pub fn viewport(&self) -> [f32; 4] {
        self.viewport
    }

    fn pass_viewport(&self, target: FseImage) -> [f32; 4] {
        if target == FseImage::BackBuffer {
            self.viewport
        } else {
            FULL_RECT
        }
    }

    /// FullScreenEffectsManager_PC_XBOX.cpp 0x0015C240 — draw `source` over `target` with an FSE program
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        target: FseImage,
        source: FseImage,
        shader: FseShader,
        program: usize,
        src_rect: Option<[f32; 4]>,
        dst_rect: Option<[f32; 4]>,
        z: f32,
        colour: u32,
    ) {
        if !self.active {
            return;
        }
        let pass = FsePass {
            target,
            source,
            shader,
            program,
            src_rect: src_rect.unwrap_or(FULL_RECT),
            dst_rect: dst_rect.unwrap_or(FULL_RECT),
            z,
            colour,
            vectors: self.vectors,
            alpha: self.alpha_texture.clone(),
            vs_consts: self.vs_consts.clone(),
            verts: None,
            vert_z: None,
            scene: false,
            road_scene: None,
            early: self.early,
            viewport: self.pass_viewport(target),
        };
        self.passes.push(pass);
    }

    /// the danger overlay's copy back (0x00120850: default stencil state ALWAYS / ZERO, whole screen) cleared the
    /// stencil of every pixel this frame
    pub fn clear_car_stencil(&mut self) {
        self.stencil_cleared = true;
    }

    /// the car stencil bit 4 for the following stencil-tested passes (`FSE_STENCIL_NOT_CAR`), queued once per frame
    /// in pass order: back-buffer alpha cleared (the bloom has read it), then the car stencil camera redraws the
    /// world's layers with alpha-only stencil pipelines (`shader_manager::apply_car_stencil`). The original's
    /// stencil buffer is written by the world pass itself; bevy's depth is Depth32Float (no stencil) and the
    /// frame-buffer alpha already carries the glow bit until the bloom.
    pub fn car_stencil(&mut self) {
        if !self.active || self.stencil_built {
            return;
        }
        self.stencil_built = true;
        let shader = FseShader { stencil: FSE_STENCIL_CLEAR, ..FseShader::DEFAULT };
        self.draw(FseImage::BackBuffer, FseImage::BackBuffer, shader, prog::COPY, None, None, 0.0, WHITE);
        if !self.stencil_cleared {
            self.draw(FseImage::BackBuffer, FseImage::BackBuffer, FseShader::DEFAULT, prog::COPY, None, None, 0.0, WHITE);
            if let Some(p) = self.passes.last_mut() {
                p.scene = true;
            }
        }
    }

    /// RainSurface's stencil bit 8 starts out clear where the road pass did not draw: back-buffer alpha 0 except the
    /// car bit (alpha 1) when this frame's car stencil is already in it (then every other pixel is 0 already)
    pub fn road_stencil_clear(&mut self) {
        if !self.active || self.stencil_built {
            return;
        }
        let shader = FseShader { stencil: FSE_STENCIL_CLEAR, ..FseShader::DEFAULT };
        self.draw(FseImage::BackBuffer, FseImage::BackBuffer, shader, prog::COPY, None, None, 0.0, WHITE);
    }

    /// this frame's camera order of scene slot `kind` (`ROAD_SCENE_*`; set in `FseSet::Execute`)
    pub fn scene_order(&self, kind: usize) -> Option<isize> {
        self.road_orders.get(kind).copied().flatten().filter(|_| self.active)
    }

    /// a road camera's slot in pass order (`ROAD_SCENE_*`): it redraws its layer (`rain_surface::ROAD_DETAIL_LAYER`
    /// / `PUDDLE_LAYER`) into the back buffer's shared colour and depth; the mirror's slots (`ROAD_SCENE_MIRROR`..)
    /// order its own cameras (`view_render::update_mirror_orders`)
    pub fn road_scene(&mut self, kind: usize) {
        if !self.active {
            return;
        }
        self.draw(FseImage::BackBuffer, FseImage::BackBuffer, FseShader::DEFAULT, prog::COPY, None, None, 0.0, WHITE);
        if let Some(p) = self.passes.last_mut() {
            p.road_scene = Some(kind);
        }
    }

    /// many consecutive 3-vertex 0x0015E7B0 draws with one shader and source (no state change between them) as one
    /// pass: the triangles joined by degenerate ones (culling is off), so the pass budget is not spent per triangle.
    /// Each original draw is far below the 1000-vertex stream limit, so nothing is dropped.
    pub fn draw_triangles(&mut self, target: FseImage, source: FseImage, shader: FseShader, program: usize, tris: &[[FseVertex; 3]]) {
        if !self.active || tris.is_empty() {
            return;
        }
        let mut verts = Vec::with_capacity(tris.len() * 5);
        for (i, t) in tris.iter().enumerate() {
            if i != 0 {
                let last = *verts.last().unwrap();
                verts.push(last);
                verts.push(t[0]);
                // keep every real triangle on an even strip index
                if verts.len() % 2 == 1 {
                    verts.push(t[0]);
                }
            }
            verts.extend_from_slice(t);
        }
        let pass = FsePass {
            target,
            source,
            shader,
            program,
            src_rect: FULL_RECT,
            dst_rect: FULL_RECT,
            z: 0.0,
            colour: WHITE,
            vectors: self.vectors,
            alpha: self.alpha_texture.clone(),
            vs_consts: self.vs_consts.clone(),
            verts: Some(verts),
            vert_z: None,
            scene: false,
            road_scene: None,
            early: self.early,
            viewport: self.pass_viewport(target),
        };
        self.passes.push(pass);
    }

    /// 0x0015E7B0 after 0x0015E9E0 vertices: a strip whose vertices carry their own D3D z (`z[i]`)
    pub fn draw_strip_z(&mut self, target: FseImage, source: FseImage, shader: FseShader, program: usize, verts: Vec<FseVertex>, mut z: Vec<f32>) {
        let n = self.passes.len();
        z.truncate(MAX_STRIP_VERTICES);
        self.draw_strip(target, source, shader, program, verts);
        if self.passes.len() > n {
            if let Some(p) = self.passes.last_mut() {
                p.vert_z = Some(z);
            }
        }
    }

    /// GameShaderProgramManager_PC_XBOX.cpp 0x0015E7B0 — draw the collected stream (0x0015E780 begin, 0x0015E990
    /// vertices) as one triangle strip with `shader`, its stage texture = `source`
    pub fn draw_strip(&mut self, target: FseImage, source: FseImage, shader: FseShader, program: usize, mut verts: Vec<FseVertex>) {
        if !self.active || verts.len() < 3 {
            return;
        }
        verts.truncate(MAX_STRIP_VERTICES);
        let pass = FsePass {
            target,
            source,
            shader,
            program,
            src_rect: FULL_RECT,
            dst_rect: FULL_RECT,
            z: 0.0,
            colour: WHITE,
            vectors: self.vectors,
            alpha: self.alpha_texture.clone(),
            vs_consts: self.vs_consts.clone(),
            verts: Some(verts),
            vert_z: None,
            scene: false,
            road_scene: None,
            early: self.early,
            viewport: self.pass_viewport(target),
        };
        self.passes.push(pass);
    }

    /// FullScreenEffectsManager_PC_XBOX.cpp 0x0015C770 — blur `source` into `target`: per pass, the source (the
    /// first time `source`, then `target`) -> a quarter buffer through "GlowBlur1" (+280: blend 1, z off) with
    /// program 3 Bloom (`bloom`) or 1 Blur and fBlurAmount (-step, step, 0, 0), then back into `target` through
    /// "GlowBlur2" (+284, same state) with Copy; no passes = one Copy with "Default". No free quarter buffer: nothing.
    pub fn blur_passes(&mut self, target: FseImage, source: FseImage, passes: u32, bloom: bool) {
        let glow_blur = FseShader::new(1, false);
        if passes == 0 {
            self.draw(target, source, FseShader::DEFAULT, prog::COPY, None, None, 0.0, WHITE);
            return;
        }
        let Some(quarter) = self.get_off_screen_buffer(OFF_SCREEN_QUARTER) else { return };
        for i in 0..passes {
            let src = if i != 0 { target } else { source };
            self.set_vector(gmi::BLUR_AMOUNT, Vec4::new(0.0 - BLUR_STEP, BLUR_STEP, 0.0, 0.0));
            let p = if bloom { prog::BLOOM } else { prog::BLUR };
            self.draw(quarter, src, glow_blur, p, None, None, 0.0, WHITE);
            self.draw(target, quarter, glow_blur, prog::COPY, None, None, 0.0, WHITE);
        }
        self.release_off_screen_buffer(quarter);
    }

    /// this frame's queued passes
    pub fn queued(&self) -> &[FsePass] {
        &self.passes
    }

    /// four placeholder off-screen buffers (tests without an image store)
    #[cfg(test)]
    pub fn test_buffers(&mut self) {
        self.size = UVec2::new(640, 480);
        self.back_buffer = Some(Handle::default());
        self.buffers = (0..4u32)
            .map(|i| {
                let s = if i < 2 { 1 } else { 2 };
                OffScreenBuffer { used: false, image: Handle::default(), size: UVec2::new(640 >> s, 480 >> s) }
            })
            .collect();
    }

    /// bevy reverse-z depth of a D3D z with the original camera's planes (only z-tested passes use it)
    fn device_depth(&self, z: f32, near: f32, far: f32) -> f32 {
        let view_z = near * far / (far - z * (far - near));
        (self.near / view_z.max(1e-6)).clamp(0.0, 1.0)
    }
}

/// the original camera's planes for z-tested screen quads (VsBaseCamera +0xEC / +0xF0, set by core)
#[derive(Resource, Clone, Copy, Debug)]
pub struct FseCameraPlanes {
    pub near: f32,
    pub far: f32,
}

impl Default for FseCameraPlanes {
    fn default() -> Self {
        Self { near: 0.5, far: 600.0 }
    }
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FseSet {
    /// effects queue their passes (GameManager 0x000B9010 order)
    Build,
    /// redirect, cameras, quads
    Execute,
}

// ---- programs ---------------------------------------------------------------------------------------------------

const FSE_FRAGMENT_WRAPPER: &str = r#"
struct ProgramParams {
    vs: array<vec4<f32>, 192>,
    ps: array<vec4<f32>, 16>,
    tex: array<vec4<f32>, 4>,
    fog: vec4<f32>,
    alpha: vec4<f32>,
};
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> prog: ProgramParams;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var tex0: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var samp0: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var tex1: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var samp1: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var tex2: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var samp2: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var tex3: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var samp3: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(9) var tex_stencil: texture_2d<f32>;

fn xps_c(i: i32) -> vec4<f32> { return prog.ps[i]; }
fn xps_uv(s: i32, c: vec4<f32>) -> vec2<f32> {
    let t = prog.tex[s];
    return select(c.xy, c.xy / max(t.xy, vec2<f32>(1.0)), t.z != 0.0);
}
fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}
// render targets hold gamma values in srgb storage: back to the values the nv2a sampled
fn fse_texel(s: i32, c: vec4<f32>) -> vec4<f32> {
    return select(c, vec4<f32>(linear_to_srgb(c.rgb), c.a), prog.tex[s].w != 0.0);
}
fn xps_tex0(c: vec4<f32>) -> vec4<f32> { return fse_texel(0, textureSample(tex0, samp0, xps_uv(0, c))); }
fn xps_tex1(c: vec4<f32>) -> vec4<f32> { return fse_texel(1, textureSample(tex1, samp1, xps_uv(1, c))); }
fn xps_tex2(c: vec4<f32>) -> vec4<f32> { return fse_texel(2, textureSample(tex2, samp2, xps_uv(2, c))); }
fn xps_tex3(c: vec4<f32>) -> vec4<f32> { return fse_texel(3, textureSample(tex3, samp3, xps_uv(3, c))); }

struct ProgramVaryings {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) d0: vec4<f32>,
    @location(2) d1: vec4<f32>,
    @location(3) t0: vec4<f32>,
    @location(4) t1: vec4<f32>,
    @location(5) t2: vec4<f32>,
    @location(6) t3: vec4<f32>,
};

@fragment
fn fragment(in: ProgramVaryings) -> @location(0) vec4<f32> {
    // stencil NOTEQUAL ref 4 mask 4: the car bit sits in the back buffer's alpha as 255 (FSE_STENCIL_NOT_CAR);
    // EQUAL ref 8 mask 8: road pixels hold 1..254 (FSE_STENCIL_ROAD)
    if (prog.alpha.y != 0.0) {
        let s = round(textureLoad(tex_stencil, vec2<i32>(in.position.xy), 0).a * 255.0);
        if (prog.alpha.y < 1.5 && s > 254.0) {
            discard;
        }
        if (prog.alpha.y > 1.5 && (s < 1.0 || s > 254.0)) {
            discard;
        }
    }
    var i: XpsIn;
    i.d0 = in.d0;
    i.d1 = in.d1;
    i.t0 = in.t0;
    i.t1 = in.t1;
    i.t2 = in.t2;
    i.t3 = in.t3;
    var c = xps_program(i);
    // fixed-function stage 0 (FFP_*)
    let ffp = i32(prog.alpha.x + 0.5);
    if (ffp != 0) {
        let t = xps_tex0(in.t0);
        if (ffp == 1) {
            // colour MODULATE(diffuse, texture), alpha SELECTARG1(diffuse)
            c = vec4<f32>(in.d0.rgb * t.rgb, in.d0.a);
        } else if (ffp == 2) {
            c = in.d0 * t;
        } else {
            c = vec4<f32>(clamp(2.0 * in.d0.rgb * t.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), in.d0.a * t.a);
        }
    }
    // a target in plain storage keeps the gamma values as written
    if (prog.alpha.z != 0.0) {
        return c;
    }
    return vec4<f32>(srgb_to_linear(c.rgb), c.a);
}
"#;

pub fn fse_vertex_module(source: &str) -> Result<String, xgraphics_shader::AsmError> {
    let p = xgraphics_shader::parse(source)?;
    Ok(format!("{COMMON_WGSL}{VERTEX_WRAPPER}{}", xgraphics_shader::vertex_wgsl(&p, &[])))
}

pub fn fse_fragment_module(source: &str) -> Result<String, xgraphics_shader::AsmError> {
    let p = xgraphics_shader::parse(source)?;
    Ok(format!("{COMMON_WGSL}{FSE_FRAGMENT_WRAPPER}{}", xgraphics_shader::pixel_wgsl(&p)))
}

/// `fse_chain` vertex wrapper: the material wrapper (`VERTEX_WRAPPER`) without bevy's mesh / view bindings (FSE
/// programs use no dynamic inputs), vertex buffer = position, uv, colour at locations 0..2
const CHAIN_VERTEX_WRAPPER: &str = r#"
struct ProgramParams {
    vs: array<vec4<f32>, 192>,
    ps: array<vec4<f32>, 16>,
    tex: array<vec4<f32>, 4>,
    fog: vec4<f32>,
    alpha: vec4<f32>,
};
@group(0) @binding(0) var<uniform> prog: ProgramParams;

var<private> dyn_wvp: mat4x4<f32>;
var<private> dyn_world: mat4x4<f32>;
var<private> dyn_wv: mat4x4<f32>;
var<private> dyn_vp: mat4x4<f32>;
var<private> dyn_view: mat4x4<f32>;
var<private> dyn_camera_dir: vec4<f32>;

fn xvs_c(i: i32) -> vec4<f32> { return prog.vs[i]; }
fn xvs_c_rel(i: i32) -> vec4<f32> { return prog.vs[clamp(i, 0, 191)]; }

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct ProgramVaryings {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) d0: vec4<f32>,
    @location(2) d1: vec4<f32>,
    @location(3) t0: vec4<f32>,
    @location(4) t1: vec4<f32>,
    @location(5) t2: vec4<f32>,
    @location(6) t3: vec4<f32>,
};

@vertex
fn vertex(vin: Vertex) -> ProgramVaryings {
    var v: array<vec4<f32>, 16>;
    for (var k = 0; k < 16; k++) { v[k] = vec4<f32>(0.0, 0.0, 0.0, 1.0); }
    v[0] = vec4<f32>(vin.position, 1.0);
    v[3] = vin.color;
    v[9] = vec4<f32>(vin.uv, 0.0, 1.0);
    let o = xvs_program(v);
    var out: ProgramVaryings;
    out.position = o.pos;
    out.world_position = vec4<f32>(vin.position, 1.0);
    out.d0 = o.d0;
    out.d1 = o.d1;
    out.t0 = o.t0;
    out.t1 = o.t1;
    out.t2 = o.t2;
    out.t3 = o.t3;
    return out;
}
"#;

#[derive(Default)]
struct FseRegistry {
    vertex: Vec<Option<Handle<Shader>>>,
    fragment: Vec<Option<Handle<Shader>>>,
    chain_vertex: Vec<Option<Handle<Shader>>>,
}

static FSE_REGISTRY: OnceLock<FseRegistry> = OnceLock::new();

/// (vertex, fragment) shaders of program `p` for `fse_chain`
pub(crate) fn chain_shaders(p: u8) -> Option<(Handle<Shader>, Handle<Shader>)> {
    let r = FSE_REGISTRY.get()?;
    Some((r.chain_vertex.get(p as usize)?.clone()?, r.fragment.get(p as usize)?.clone()?))
}

/// FullScreenEffectsManager_PC_XBOX.cpp 0x0015B2F0 — assemble the eight program pairs (a missing file asserts
/// "pData" in the original; the slot stays empty here)
fn load_fse_programs(shaders: &mut Assets<Shader>) -> FseRegistry {
    let mut r = FseRegistry::default();
    let load = |name: &str, ext: &str| visos::file::platform::read(&format!("\\Shaders\\{name}.{ext}")).ok().map(|b| String::from_utf8_lossy(&b).into_owned());
    for (k, name) in FSE_VERTEX_PROGRAMS.iter().enumerate() {
        r.vertex.push(match load(name, "vxu").map(|s| fse_vertex_module(&s)) {
            Some(Ok(m)) => Some(shaders.add(Shader::from_wgsl(m, format!("visos_render/fse/{k}_{name}.vxu.wgsl")))),
            other => {
                log::error!("pData: fse vertex program {name} ({:?})", other.map(|r| r.err()));
                None
            }
        });
    }
    for (k, name) in FSE_FRAGMENT_PROGRAMS.iter().enumerate() {
        r.fragment.push(match load(name, "pxu").map(|s| fse_fragment_module(&s)) {
            Some(Ok(m)) => Some(shaders.add(Shader::from_wgsl(m, format!("visos_render/fse/{k}_{name}.pxu.wgsl")))),
            other => {
                log::error!("pData: fse fragment program {name} ({:?})", other.map(|r| r.err()));
                None
            }
        });
    }
    // prog::DANGER: the game's pair (GameShaderProgramManager 0x0015D250 tables), used by the danger overlay
    let k = prog::DANGER;
    r.vertex.push(load(DANGER_PROGRAM, "vxu").and_then(|s| fse_vertex_module(&s).ok()).map(|m| shaders.add(Shader::from_wgsl(m, format!("visos_render/fse/{k}_{DANGER_PROGRAM}.vxu.wgsl")))));
    r.fragment.push(load(DANGER_PROGRAM, "pxu").and_then(|s| fse_fragment_module(&s).ok()).map(|m| shaders.add(Shader::from_wgsl(m, format!("visos_render/fse/{k}_{DANGER_PROGRAM}.pxu.wgsl")))));
    if r.vertex[k].is_none() || r.fragment[k].is_none() {
        log::error!("pData: program {DANGER_PROGRAM}");
    }
    // the same vertex programs for `fse_chain` (own vertex buffer and bind group 0, no mesh / view bindings)
    for (k, name) in FSE_VERTEX_PROGRAMS.iter().chain([&DANGER_PROGRAM]).enumerate() {
        let m = load(name, "vxu").and_then(|s| xgraphics_shader::parse(&s).ok()).map(|p| format!("{COMMON_WGSL}{CHAIN_VERTEX_WRAPPER}{}", xgraphics_shader::vertex_wgsl(&p, &[])));
        r.chain_vertex.push(m.map(|m| shaders.add(Shader::from_wgsl(m, format!("visos_render/fse/{k}_{name}.chain.vxu.wgsl")))));
    }
    r
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FseKey {
    program: u8,
    blend: u32,
    z_test: bool,
    z_func: u32,
    stencil: u8,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, PartialEq)]
#[bind_group_data(FseKey)]
pub struct FseMaterial {
    /// `tex[s]` = (w, h, linear, srgb render target)
    #[uniform(0)]
    pub params: ProgramParams,
    #[texture(1)]
    #[sampler(2)]
    pub t0: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    pub t1: Option<Handle<Image>>,
    #[texture(5)]
    #[sampler(6)]
    pub t2: Option<Handle<Image>>,
    #[texture(7)]
    #[sampler(8)]
    pub t3: Option<Handle<Image>>,
    /// the back buffer for `FSE_STENCIL_NOT_CAR` passes (`prog.alpha.y` = 1)
    #[texture(9)]
    pub stencil: Option<Handle<Image>>,
    pub program: u8,
    pub shader: FseShaderKey,
    /// Transparent3d sort bias: position of the pass in its camera's run (`fse_runs`), later passes draw later
    pub bias: f32,
    /// part of a multi-pass run: drawn in Transparent3d whatever its blend (the pipeline state is the pass's own)
    pub in_run: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FseShaderKey {
    pub blend: u32,
    pub z_test: bool,
    pub z_func: u32,
    pub stencil: u8,
}

impl From<&FseMaterial> for FseKey {
    fn from(m: &FseMaterial) -> Self {
        Self { program: m.program, blend: m.shader.blend, z_test: m.shader.z_test, z_func: m.shader.z_func, stencil: m.shader.stencil }
    }
}

impl Material for FseMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        if self.in_run {
            return AlphaMode::Blend;
        }
        match get_blend_mode(self.shader.blend) {
            Some(b) if b.enable => AlphaMode::Blend,
            _ => AlphaMode::Opaque,
        }
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn depth_bias(&self) -> f32 {
        self.bias
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let k = key.bind_group_data;
        if let Some(r) = FSE_REGISTRY.get() {
            if let Some(Some(vs)) = r.vertex.get(k.program as usize) {
                descriptor.vertex.shader = vs.clone();
            }
            if let (Some(Some(ps)), Some(f)) = (r.fragment.get(k.program as usize), descriptor.fragment.as_mut()) {
                f.shader = ps.clone();
            }
        }
        // SetZState(test, 0, 0): LESSEQUAL, no write
        apply_stage_pipeline_state(descriptor, k.blend, false, k.z_func, k.z_test);
        // screen strips are not culled; render targets keep the alpha the programs write
        descriptor.primitive.cull_mode = None;
        if let (Some(b), Some(f)) = (get_blend_mode(k.blend), descriptor.fragment.as_mut()) {
            if let Some(Some(t)) = f.targets.get_mut(0) {
                if b.write_mask & 0x0100_0000 != 0 {
                    t.write_mask |= ColorWrites::ALPHA;
                }
            }
        }
        // passes into the back buffer go through the same stencil rules as geometry
        if is_glow_stencil_view(key.mesh_key) {
            apply_glow_stencil(descriptor, k.blend, false);
        }
        if k.stencil == FSE_STENCIL_CLEAR {
            if let Some(Some(t)) = descriptor.fragment.as_mut().and_then(|f| f.targets.get_mut(0)) {
                let zero = BlendComponent { src_factor: BlendFactor::Zero, dst_factor: BlendFactor::Zero, operation: BlendOperation::Add };
                t.blend = Some(BlendState { color: zero, alpha: zero });
                t.write_mask = ColorWrites::ALPHA;
            }
        }
        if k.stencil == FSE_STENCIL_ROAD {
            if let Some(Some(t)) = descriptor.fragment.as_mut().and_then(|f| f.targets.get_mut(0)) {
                t.write_mask.remove(ColorWrites::ALPHA);
            }
        }
        Ok(())
    }
}

// ---- execution --------------------------------------------------------------------------------------------------

/// passes run at orders WORLD_CAMERA_ORDER + 1.. (-39..=-1), the present copy at 9 (HUD cameras from 10). A frame
/// with bloom (12 passes), motion blur (2) and the zone / DOF / danger effects stays well inside.
pub const MAX_PASSES: usize = 39;
const FIRST_ORDER: isize = crate::track_render::WORLD_CAMERA_ORDER + 1;
const PRESENT_ORDER: isize = 9;
const FIRST_LAYER: usize = 24;
/// early passes (`FsePass.early`) run at orders EARLY_FIRST_ORDER.. (-57..=-46), after the envmap source cameras
/// (`ENVMAP_SOURCE_ORDER`.., 5 cameras) and before the sky passes (-44..) and the world camera (-40); their slots
/// follow the present slot in the pool (layers 64..)
pub const EARLY_PASSES: usize = 12;
pub const EARLY_FIRST_ORDER: isize = crate::track_render::WORLD_CAMERA_ORDER - 5 - EARLY_PASSES as isize;
pub const ENVMAP_SOURCE_ORDER: isize = EARLY_FIRST_ORDER - 5;

/// cameras sharing the back buffer's main texture write it out unblended (bevy alpha-blends every camera after the
/// first on a target, which would make the frame-buffer alpha — the glow mask — an opacity)
fn replace_output() -> CameraOutputMode {
    CameraOutputMode::Write { blend_state: Some(BlendState::REPLACE), clear_color: ClearColorConfig::None }
}

fn is_replace_output(m: &CameraOutputMode) -> bool {
    matches!(m, CameraOutputMode::Write { blend_state: Some(b), .. } if *b == BlendState::REPLACE)
}

/// marks a camera this module put into the glow-stencil view state
#[derive(Component)]
struct FseGlowView;

#[derive(Component)]
pub(crate) struct FsePassCamera;

#[derive(Default, Resource)]
struct FsePool {
    /// (camera, quad, mesh, material) per pass slot; the last is the present pass
    slots: Vec<(Entity, Entity, Handle<Mesh>, Handle<FseMaterial>)>,
}

fn target_image(size: UVec2) -> Image {
    let mut img = Image::new_target_texture(size.x.max(1), size.y.max(1), TextureFormat::Rgba8UnormSrgb, None);
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::linear());
    img
}

/// FullScreenEffectsManager_PC_XBOX.cpp 0x0015BAE0 / 0x0015BBF0 — back buffer plus 2 half and 2 quarter buffers
fn ensure_targets(fse: &mut FullScreenEffects, images: &mut Assets<Image>, size: UVec2) {
    if fse.size == size && fse.back_buffer.is_some() && fse.buffers.len() == 4 {
        return;
    }
    fse.size = size;
    fse.back_buffer = Some(images.add(target_image(size)));
    fse.buffers.clear();
    for kind in [OFF_SCREEN_HALF, OFF_SCREEN_HALF, OFF_SCREEN_QUARTER, OFF_SCREEN_QUARTER] {
        // 0x0015BBF0: width/height >> 1 (kind 1) or >> 2 (kind 2)
        let s = UVec2::new(size.x >> kind, size.y >> kind).max(UVec2::ONE);
        fse.buffers.push(OffScreenBuffer { used: false, image: images.add(target_image(s)), size: s });
    }
}

fn same_target(a: &RenderTarget, b: &RenderTarget) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

/// redirect the world camera into the back buffer; size the targets after the final target
#[allow(clippy::type_complexity)]
fn redirect_world(
    mut commands: Commands,
    mut fse: ResMut<FullScreenEffects>,
    mut images: ResMut<Assets<Image>>,
    mut cams: Query<(Entity, Option<&RenderTarget>, Option<&Msaa>, Option<&Projection>, &mut Camera, Has<FseGlowView>), With<TrackCamera>>,
    mut sky: Query<(Entity, &mut Camera, Option<&Msaa>, Has<FseGlowView>), (With<SkyCamera>, Without<TrackCamera>)>,
    primary: Query<&Window, With<PrimaryWindow>>,
    windows: Query<&Window>,
) {
    let active_before = fse.active;
    redirect_world_inner(&mut commands, &mut fse, &mut images, &mut cams, &primary, &windows);
    // world and sky cameras share the back buffer's main texture: same msaa, unblended output, glow-stencil view
    let Some((we, _, wmsaa, _, mut wcam, wglow)) = cams.iter_mut().next() else { return };
    let msaa = wmsaa.copied().unwrap_or_default();
    let on = fse.active;
    let mut views: Vec<(Entity, Mut<Camera>, bool, bool)> = vec![(we, wcam.reborrow(), wglow, true)];
    for (e, cam, m, g) in sky.iter_mut() {
        let same = m.copied().unwrap_or_default() == msaa;
        views.push((e, cam, g, same));
    }
    for (e, mut cam, glow, same_msaa) in views {
        if on {
            if !is_replace_output(&cam.output_mode) {
                cam.output_mode = replace_output();
            }
            if !same_msaa {
                commands.entity(e).insert(msaa);
            }
            if !glow {
                commands.entity(e).insert((GLOW_STENCIL_VIEW, FseGlowView));
            }
        } else if glow || active_before {
            cam.output_mode = CameraOutputMode::default();
            commands.entity(e).remove::<(bevy::light::ShadowFilteringMethod, FseGlowView)>();
        }
    }
}

fn redirect_world_inner(
    commands: &mut Commands,
    fse: &mut FullScreenEffects,
    images: &mut Assets<Image>,
    cams: &mut Query<(Entity, Option<&RenderTarget>, Option<&Msaa>, Option<&Projection>, &mut Camera, Has<FseGlowView>), With<TrackCamera>>,
    primary: &Query<&Window, With<PrimaryWindow>>,
    windows: &Query<&Window>,
) {
    let Some((e, target, msaa, proj, _, _)) = cams.iter().next() else {
        fse.active = false;
        fse.final_target = None;
        return;
    };
    let current = target.cloned().unwrap_or_default();
    let redirected = fse.back_buffer.as_ref().is_some_and(|h| same_target(&current, &RenderTarget::Image(h.clone().into())));
    if !redirected {
        fse.final_target = Some(current);
    }
    let Some(fin) = fse.final_target.clone() else { return };
    let size = match &fin {
        RenderTarget::Window(bevy::window::WindowRef::Primary) => primary.iter().next().map(|w| w.physical_size()),
        RenderTarget::Window(bevy::window::WindowRef::Entity(w)) => windows.get(*w).ok().map(|w| w.physical_size()),
        RenderTarget::Image(i) => images.get(&i.handle).map(|i| i.size()),
        _ => None,
    };
    let Some(size) = size.filter(|s| s.x > 0 && s.y > 0) else {
        fse.active = false;
        return;
    };
    ensure_targets(fse, images, size);
    fse.msaa = msaa.copied().unwrap_or_default();
    if let Some(Projection::Perspective(p)) = proj {
        fse.near = p.near;
    }
    let bb = fse.back_buffer.clone().unwrap();
    if !redirected || fse.size != size {
        commands.entity(e).insert(RenderTarget::Image(bb.into()));
    }
    fse.active = true;
}

/// FullScreenEffectsManager_PC_XBOX.cpp 0x0015C3C0 — the screen quad: destination rectangle in clip space (the D3D half-pixel shift is left out, wgpu
/// samples texel centres), source rectangle in texels of a linear source
fn quad_mesh(pass: &FsePass, src_size: UVec2, depth: f32, vert_depth: impl Fn(f32) -> f32) -> Mesh {
    // the target's viewport: D3D maps clip space into it
    let vp = pass.viewport;
    let vx = |x: f32| vp[0] + x * vp[2];
    let vy = |y: f32| vp[1] + y * vp[3];
    if let Some(verts) = &pass.verts {
        let mut m = Mesh::new(PrimitiveTopology::TriangleStrip, RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD);
        let dz = |i: usize| match &pass.vert_z {
            Some(z) if pass.shader.z_test => z.get(i).map_or(depth, |z| vert_depth(*z)),
            _ => depth,
        };
        m.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            verts.iter().enumerate().map(|(i, v)| [2.0 * vx(v.pos.x) - 1.0, 1.0 - vy(v.pos.y) * 2.0, dz(i)]).collect::<Vec<_>>(),
        );
        m.insert_attribute(Mesh::ATTRIBUTE_UV_0, verts.iter().map(|v| [v.uv.x, v.uv.y]).collect::<Vec<_>>());
        m.insert_attribute(Mesh::ATTRIBUTE_COLOR, verts.iter().map(|v| crate::effect_draw::argb_to_rgba(v.colour)).collect::<Vec<_>>());
        m.insert_indices(Indices::U32((0..verts.len() as u32).collect()));
        return m;
    }
    let d = pass.dst_rect;
    let s = pass.src_rect;
    let x0 = 2.0 * vx(d[0]) - 1.0;
    let x1 = vx(d[0] + d[2]) * 2.0 - 1.0;
    let y0 = 1.0 - vy(d[1]) * 2.0;
    let y1 = 1.0 - vy(d[1] + d[3]) * 2.0;
    let (w, h) = (src_size.x as f32, src_size.y as f32);
    let u0 = w * s[0];
    let u1 = w * (s[0] + s[2]);
    let v0 = h * s[1];
    let v1 = h * (s[1] + s[3]);
    let c = crate::effect_draw::argb_to_rgba(pass.colour);
    let mut m = Mesh::new(PrimitiveTopology::TriangleStrip, RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD);
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[x0, y0, depth], [x1, y0, depth], [x0, y1, depth], [x1, y1, depth]]);
    m.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[u0, v0], [u1, v0], [u0, v1], [u1, v1]]);
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![c; 4]);
    m.insert_indices(Indices::U32(vec![0, 1, 2, 3]));
    m
}

/// ShaderProgramManager BindShader for an FSE pass: each binding of the pair resolved against the pass's globals
fn bind_pass(pass: &FsePass, m: &mut FseMaterial, src: &(Handle<Image>, UVec2), src_flags: (bool, bool), dst: (UVec2, bool), back_buffer: Option<Handle<Image>>) {
    let (dst_size, dst_srgb) = dst;
    let (src_linear, src_srgb) = src_flags;
    let mut p = ProgramParams::default();
    let sz = src.1.as_vec2();
    let dz = dst_size.as_vec2().max(Vec2::ONE);
    let value = |id: usize| -> Vec4 {
        match id {
            // GFXHelper.cpp 0x0012BD80: (w, h, 1/w, 1/h) of the source texture
            gmi::SOURCE_TEXTURE_SIZE => Vec4::new(sz.x, sz.y, 1.0 / sz.x.max(1.0), 1.0 / sz.y.max(1.0)),
            // 0x0012BE80 / 0x0012BF40: the current render target (x, y only)
            gmi::VIEWPORT_SIZE => Vec4::new(dz.x, dz.y, 0.0, 0.0),
            gmi::INVERSE_VIEWPORT_SIZE => Vec4::new(1.0 / dz.x, 1.0 / dz.y, 0.0, 0.0),
            _ => pass.vectors[id],
        }
    };
    // stage 0 keeps the shader's own texture: the source (0x0015C240)
    let mut stage: [Option<(Handle<Image>, UVec2, bool, bool)>; 4] = [Some((src.0.clone(), src.1, src_linear, src_srgb)), None, None, None];
    for &(id, reg) in fragment_bindings(pass.program) {
        match id {
            gmi::SOURCE_TEXTURE => stage[reg as usize] = Some((src.0.clone(), src.1, src_linear, src_srgb)),
            gmi::ALPHA_TEXTURE => stage[reg as usize] = pass.alpha.clone().map(|h| (h, UVec2::ONE, false, false)),
            _ => p.ps[reg as usize] = value(id),
        }
    }
    // device constants first: BindShader writes the program's bindings over them at draw time
    for &(reg, v) in &pass.vs_consts {
        if let Some(r) = p.vs.get_mut((reg + 96) as usize) {
            *r = v;
        }
    }
    for &(id, reg) in vertex_bindings(pass.program) {
        p.vs[(reg + 96) as usize] = value(id);
    }
    if pass.program == prog::DANGER {
        // matWorldViewProj of a pretransformed screen strip: positions are already in clip space here
        for k in 0..4 {
            p.vs[96 + k] = Vec4::from(Mat4::IDENTITY.row(k));
        }
    }
    for (k, s) in stage.iter().enumerate() {
        p.tex[k] = match s {
            Some((_, size, linear, srgb)) => Vec4::new(size.x as f32, size.y as f32, *linear as u32 as f32, *srgb as u32 as f32),
            None => Vec4::ZERO,
        };
    }
    let [a, b, c, d] = stage;
    m.params = p;
    m.t0 = a.map(|s| s.0);
    m.t1 = b.map(|s| s.0);
    m.t2 = c.map(|s| s.0);
    m.t3 = d.map(|s| s.0);
    m.program = pass.program as u8;
    m.shader = FseShaderKey { blend: pass.shader.blend, z_test: pass.shader.z_test, z_func: pass.shader.z_func, stencil: pass.shader.stencil };
    // fixed-function stage and stencil test (wrapper `fragment`); the back buffer image holds the previous camera's
    // output while this pass renders into the shared main texture
    let test = match pass.shader.stencil {
        FSE_STENCIL_NOT_CAR if back_buffer.is_some() => 1.0,
        FSE_STENCIL_ROAD if back_buffer.is_some() => 2.0,
        _ => 0.0,
    };
    m.stencil = if test != 0.0 { back_buffer } else { None };
    m.params.alpha = Vec4::new(pass.shader.ffp as f32, test, (!dst_srgb) as u32 as f32, 0.0);
}

fn spawn_slot(commands: &mut Commands, meshes: &mut Assets<Mesh>, mats: &mut Assets<FseMaterial>, k: usize) -> (Entity, Entity, Handle<Mesh>, Handle<FseMaterial>) {
    // early slots go past 63: `layer` is const-limited to one block
    let layer = RenderLayers::none().with(FIRST_LAYER + k);
    // a real (degenerate) strip: an empty mesh is never allocated but still uploaded (slab_allocator errors)
    let mut m = Mesh::new(PrimitiveTopology::TriangleStrip, RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD);
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; 4]);
    m.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32; 2]; 4]);
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32; 4]; 4]);
    m.insert_indices(Indices::U32(vec![0, 1, 2, 3]));
    let mesh = meshes.add(m);
    let mat = mats.add(blank_material());
    let cam = commands
        .spawn((
            Camera3d::default(),
            Camera { order: FIRST_ORDER + k as isize, is_active: false, clear_color: ClearColorConfig::None, output_mode: replace_output(), ..default() },
            Tonemapping::None,
            DebandDither::Disabled,
            Msaa::Off,
            layer.clone(),
            FsePassCamera,
        ))
        .id();
    let quad = commands
        .spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), Transform::default(), Visibility::Hidden, NoFrustumCulling, layer))
        .id();
    (cam, quad, mesh, mat)
}

/// the lead slot of each job: consecutive passes drawn by one camera (`VELOCITY_PERF_OFF=fseopt`: one camera each).
/// a pass joins the previous one's camera when both write the same target, neither is a scene / road slot and it
/// does not read the run's target (its image keeps the previous camera's output until the run's camera ends). a
/// run's passes all draw in Transparent3d (`FseMaterial::in_run`; Opaque3d has no queue order) sorted by `bias`
fn fse_runs(jobs: &[Option<FsePass>], chained: &[bool], back_buffer: bool, image: impl Fn(FseImage) -> Option<Handle<Image>>) -> Vec<usize> {
    let mut lead: Vec<usize> = (0..jobs.len()).collect();
    let slot = |p: &FsePass| p.scene || p.road_scene.is_some();
    for k in 1..jobs.len() {
        // the present slot and the first early slot start their own lists
        if k == MAX_PASSES || k == MAX_PASSES + 1 {
            continue;
        }
        let l = lead[k - 1];
        // passes drawn by a chain camera (`fse_chain`) are never part of a camera run
        if chained[k] || chained[l] {
            continue;
        }
        let (Some(a), Some(b)) = (&jobs[l], &jobs[k]) else { continue };
        if slot(a) || slot(b) || a.target != b.target {
            continue;
        }
        let t = a.target;
        let reads_stencil = t == FseImage::BackBuffer && back_buffer && matches!(b.shader.stencil, FSE_STENCIL_NOT_CAR | FSE_STENCIL_ROAD);
        let reads_alpha = b.alpha.is_some() && b.alpha == image(t);
        if b.source == t || reads_stencil || reads_alpha {
            continue;
        }
        lead[k] = l;
    }
    lead
}

/// passes `fse_chain` draws: into an off-screen image, no z test, no stencil, not a scene / road slot, and no camera
/// pass of the frame writes the same image (that camera's main texture would miss the chain's writes)
fn chain_mask(jobs: &[Option<FsePass>]) -> Vec<bool> {
    let ok = |k: usize, p: &FsePass| {
        k != MAX_PASSES
            && !p.scene
            && p.road_scene.is_none()
            && p.target != FseImage::BackBuffer
            && !p.shader.z_test
            && p.shader.stencil == FSE_STENCIL_NONE
    };
    let mut mask: Vec<bool> = jobs.iter().enumerate().map(|(k, j)| j.as_ref().is_some_and(|p| ok(k, p))).collect();
    let camera_targets: Vec<FseImage> = jobs.iter().zip(&mask).filter(|(_, m)| !**m).filter_map(|(j, _)| j.as_ref().map(|p| p.target)).collect();
    for (k, j) in jobs.iter().enumerate() {
        if j.as_ref().is_some_and(|p| camera_targets.contains(&p.target)) {
            mask[k] = false;
        }
    }
    mask
}

fn blank_material() -> FseMaterial {
    FseMaterial {
        params: ProgramParams::default(),
        t0: None,
        t1: None,
        t2: None,
        t3: None,
        stencil: None,
        program: 0,
        shader: FseShaderKey { blend: 1, z_test: false, z_func: d3dcmp::LESSEQUAL, stencil: FSE_STENCIL_NONE },
        bias: 0.0,
        in_run: false,
    }
}

/// one camera + quad per run of queued passes (`fse_runs`), then the present copy of the back buffer to the final target
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn execute_passes(
    mut commands: Commands,
    mut fse: ResMut<FullScreenEffects>,
    mut pool: ResMut<FsePool>,
    planes: Res<FseCameraPlanes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<FseMaterial>>,
    mut cams: Query<(&mut Camera, &mut Camera3d, Option<&RenderTarget>, &Msaa, Option<&mut Projection>, Has<FseGlowView>, Option<&RenderLayers>), With<FsePassCamera>>,
    mut vis: Query<&mut Visibility>,
    mut mesh3d: Query<&mut Mesh3d>,
    mut images: ResMut<Assets<Image>>,
    mut chain_pool: ResMut<crate::fse_chain::FseChainPool>,
    mut chain_cams: Query<&mut Camera, (With<crate::fse_chain::FseChainCamera>, Without<FsePassCamera>)>,
) {
    if crate::golden::DUMP_FSE.swap(false, std::sync::atomic::Ordering::Relaxed) {
        for (k, p) in fse.passes.iter().enumerate() {
            log::info!(
                "fse pass {k}: early {} scene {} road {:?} tgt {:?} src {:?} prog {} blend {} z {} stencil {} strip {}",
                p.early, p.scene, p.road_scene, p.target, p.source, p.program, p.shader.blend, p.shader.z_test, p.shader.stencil, p.verts.is_some()
            );
        }
    }
    let (mut early, mut passes): (Vec<FsePass>, Vec<FsePass>) = std::mem::take(&mut fse.passes).into_iter().partition(|p| p.early);
    fse.early = false;
    for b in fse.buffers.iter_mut() {
        b.used = false;
    }
    fse.stencil_built = false;
    fse.stencil_cleared = false;
    fse.stencil_order = None;
    fse.road_orders = [None; SCENE_SLOTS];
    fse.viewport = FULL_RECT;
    let back_buffer = fse.back_buffer.clone();
    if passes.len() > MAX_PASSES {
        log::warn!("full screen effects: {} passes, {} drawn", passes.len(), MAX_PASSES);
        passes.truncate(MAX_PASSES);
    }
    if early.len() > EARLY_PASSES {
        log::warn!("full screen effects: {} early passes, {} drawn", early.len(), EARLY_PASSES);
        early.truncate(EARLY_PASSES);
    }
    while pool.slots.len() < MAX_PASSES + 1 + EARLY_PASSES {
        let k = pool.slots.len();
        let s = spawn_slot(&mut commands, &mut meshes, &mut mats, k);
        pool.slots.push(s);
    }
    let mut jobs: Vec<Option<FsePass>> = passes.into_iter().map(Some).collect();
    jobs.resize(MAX_PASSES, None);
    // present: Copy of the back buffer, rgb only over an opaque clear (the back buffer's alpha is the glow mask; the
    // HUD cameras composite over this camera's alpha)
    jobs.push(fse.active.then(|| FsePass {
        target: FseImage::BackBuffer,
        source: FseImage::BackBuffer,
        shader: FseShader::new(0, false),
        program: prog::COPY,
        src_rect: FULL_RECT,
        dst_rect: FULL_RECT,
        z: 0.0,
        colour: 0xFFFF_FFFF,
        vectors: [Vec4::ZERO; gmi::COUNT],
        alpha: None,
        vs_consts: Vec::new(),
        verts: None,
        vert_z: None,
        scene: false,
        road_scene: None,
        early: false,
        viewport: FULL_RECT,
    }));
    jobs.extend(early.into_iter().map(Some));
    jobs.resize(MAX_PASSES + 1 + EARLY_PASSES, None);
    let chained: Vec<bool> = if fse.active && !crate::golden::perf_off("fsechain") { chain_mask(&jobs) } else { vec![false; jobs.len()] };
    // chain lead per pass (consecutive chained passes share one chain camera) and the chains' passes
    let mut chain_lead: Vec<usize> = (0..jobs.len()).collect();
    for k in 1..jobs.len() {
        if chained[k] && chained[k - 1] {
            chain_lead[k] = chain_lead[k - 1];
        }
    }
    let mut chains: Vec<(usize, Vec<crate::fse_chain::ChainPass>)> = Vec::new();
    let lead: Vec<usize> = if fse.active && !crate::golden::perf_off("fseopt") {
        fse_runs(&jobs, &chained, back_buffer.is_some(), |i| fse.image(i).map(|x| x.0))
    } else {
        (0..jobs.len()).collect()
    };
    // a run's camera sees the quad layers of all its passes
    let mut run_layers: Vec<Option<RenderLayers>> = vec![None; jobs.len()];
    for (k, &l) in lead.iter().enumerate() {
        if l != k {
            let r = run_layers[l].take().unwrap_or_else(|| RenderLayers::none().with(FIRST_LAYER + l));
            run_layers[l] = Some(r.with(FIRST_LAYER + k));
        }
    }
    for (k, job) in jobs.into_iter().enumerate() {
        let (cam_e, quad_e, mesh_h, mat_h) = pool.slots[k].clone();
        let present = k == MAX_PASSES;
        let early_slot = k > MAX_PASSES;
        let joined = lead[k] != k;
        let Ok((mut cam, mut cam3d, cur_target, cur_msaa, proj, cur_glow, cur_layers)) = cams.get_mut(cam_e) else { continue };
        let want_layers = run_layers[k].take().unwrap_or_else(|| RenderLayers::none().with(FIRST_LAYER + k));
        if cur_layers != Some(&want_layers) {
            commands.entity(cam_e).insert(want_layers);
        }
        let pass = job.filter(|_| fse.active);
        if pass.as_ref().is_some_and(|p| p.scene) {
            // this slot's order goes to the car stencil camera (`update_car_stencil_camera`)
            fse.stencil_order = Some(FIRST_ORDER + k as isize);
        }
        if let Some(r) = pass.as_ref().and_then(|p| p.road_scene) {
            // this slot's order goes to that road camera (`update_road_cameras`)
            if let Some(o) = fse.road_orders.get_mut(r) {
                *o = Some(FIRST_ORDER + k as isize);
            }
        }
        let Some(pass) = pass.filter(|p| !p.scene && p.road_scene.is_none()) else {
            if cam.is_active {
                cam.is_active = false;
            }
            if let Ok(mut v) = vis.get_mut(quad_e) {
                *v = Visibility::Hidden;
            }
            continue;
        };
        let (Some(src), Some((dst_img, dst_size))) = (fse.image(pass.source), fse.image(pass.target)) else { continue };
        if chained[k] {
            // drawn by a chain camera (`fse_chain`): same program bindings and strip, no camera / quad / material
            if cam.is_active {
                cam.is_active = false;
            }
            if let Ok(mut v) = vis.get_mut(quad_e) {
                if *v != Visibility::Hidden {
                    *v = Visibility::Hidden;
                }
            }
            let src_flags = fse.image_flags(pass.source);
            let uv_size = if src_flags.0 { src.1 } else { UVec2::ONE };
            let mesh = quad_mesh(&pass, uv_size, 0.5, |z| fse.device_depth(z, planes.near, planes.far));
            let mut m = blank_material();
            bind_pass(&pass, &mut m, &src, src_flags, (dst_size, fse.image_flags(pass.target).1), back_buffer.clone());
            let cp = crate::fse_chain::ChainPass {
                target: dst_img.id(),
                program: m.program,
                blend: m.shader.blend,
                params: m.params,
                tex: [m.t0.as_ref().map(|h| h.id()), m.t1.as_ref().map(|h| h.id()), m.t2.as_ref().map(|h| h.id()), m.t3.as_ref().map(|h| h.id())],
                verts: crate::fse_chain::mesh_vertices(&mesh),
            };
            let l = chain_lead[k];
            match chains.last_mut() {
                Some((cl, v)) if *cl == l => v.push(cp),
                _ => chains.push((l, vec![cp])),
            }
            continue;
        }
        if joined {
            // drawn by the run's lead camera
            if cam.is_active {
                cam.is_active = false;
            }
        } else {
            // the present pass writes the final target; back buffer passes join the world camera's textures
            let (target, msaa, load_depth) = if present {
                (fse.final_target.clone().unwrap_or_default(), fse.msaa, false)
            } else if pass.target == FseImage::BackBuffer {
                (RenderTarget::Image(dst_img.into()), fse.msaa, true)
            } else {
                (RenderTarget::Image(dst_img.into()), Msaa::Off, false)
            };
            if !cur_target.is_some_and(|t| same_target(t, &target)) {
                commands.entity(cam_e).insert(target);
            }
            // bevy's camera_system only recomputes the target size for new cameras, projection changes and image
            // asset events, not for a swapped `RenderTarget`: a pass camera moved onto an existing image (the final
            // target of a headless capture) kept no viewport and drew nothing. touch the projection until it matches.
            let want_size = if present { fse.size } else { dst_size };
            if cam.physical_viewport_size() != Some(want_size) {
                if let Some(mut p) = proj {
                    p.set_changed();
                }
            }
            if *cur_msaa != msaa {
                commands.entity(cam_e).insert(msaa);
            }
            let glow_view = !present && pass.target == FseImage::BackBuffer;
            if glow_view != cur_glow {
                if glow_view {
                    commands.entity(cam_e).insert((GLOW_STENCIL_VIEW, FseGlowView));
                } else {
                    commands.entity(cam_e).remove::<(bevy::light::ShadowFilteringMethod, FseGlowView)>();
                }
            }
            let clear = if present { ClearColorConfig::Custom(Color::BLACK) } else { ClearColorConfig::None };
            if std::mem::discriminant(&cam.clear_color) != std::mem::discriminant(&clear) {
                cam.clear_color = clear;
            }
            let want_load = if load_depth { Camera3dDepthLoadOp::Load } else { Camera3dDepthLoadOp::Clear(0.0) };
            if std::mem::discriminant(&cam3d.depth_load_op) != std::mem::discriminant(&want_load) {
                cam3d.depth_load_op = want_load;
            }
            let order = if present {
                PRESENT_ORDER
            } else if early_slot {
                EARLY_FIRST_ORDER + (k - MAX_PASSES - 1) as isize
            } else {
                FIRST_ORDER + k as isize
            };
            if cam.order != order || !cam.is_active {
                cam.order = order;
                cam.is_active = true;
            }
        }
        let depth = if pass.shader.z_test { fse.device_depth(pass.z, planes.near, planes.far) } else { 0.5 };
        let src_flags = fse.image_flags(pass.source);
        // 0x0015C3C0: uv in texels only for a linear source (TextureManager 0x0029FB50), else 0..1
        let uv_size = if src_flags.0 { src.1 } else { UVec2::ONE };
        if crate::set_mesh(&mut meshes, &mesh_h, quad_mesh(&pass, uv_size, depth, |z| fse.device_depth(z, planes.near, planes.far))) {
            if let Ok(mut m) = mesh3d.get_mut(quad_e) {
                m.set_changed();
            }
        }
        let dst_srgb = present || fse.image_flags(pass.target).1;
        // written every frame on purpose: skipping unchanged materials was tried (details "Performance stage 5")
        if let Some(mut m) = mats.get_mut(&mat_h) {
            bind_pass(&pass, &mut m, &src, src_flags, (dst_size, dst_srgb), back_buffer.clone());
            m.bias = (k - lead[k]) as f32 * 1000.0;
            m.in_run = lead[k] != k || lead.get(k + 1) == Some(&k);
        }
        if let Ok(mut v) = vis.get_mut(quad_e) {
            if *v != Visibility::Visible {
                *v = Visibility::Visible;
            }
        }
    }
    // a chain runs at its first pass's camera order
    let chains = chains
        .into_iter()
        .map(|(l, v)| (if l > MAX_PASSES { EARLY_FIRST_ORDER + (l - MAX_PASSES - 1) as isize } else { FIRST_ORDER + l as isize }, v))
        .collect();
    crate::fse_chain::assign_chain_cameras(&mut commands, &mut chain_pool, &mut images, &mut chain_cams, chains);
}

/// the camera that replays the world's stencil bit 4 into the back buffer's alpha (`FullScreenEffects::car_stencil`)
#[derive(Component)]
pub struct CarStencilCamera;

/// a child of the world camera (same view, so the same depths), drawing its render layers in the car stencil view
/// at the order of this frame's scene slot, into the back buffer's shared colour and depth textures
#[allow(clippy::type_complexity)]
fn update_car_stencil_camera(
    mut commands: Commands,
    fse: Res<FullScreenEffects>,
    world: Query<(Entity, &Projection, Option<&RenderLayers>, &Camera), (With<TrackCamera>, Without<FsePassCamera>, Without<CarStencilCamera>)>,
    mut cams: Query<(Entity, &mut Camera, &mut Projection, Option<&RenderTarget>, &Msaa, Option<&RenderLayers>), (With<CarStencilCamera>, Without<TrackCamera>, Without<FsePassCamera>)>,
) {
    let want = fse.stencil_order.filter(|_| fse.active && !crate::golden::perf_off("stencil")).zip(fse.back_buffer.clone());
    let Some((we, proj, layers, wcam)) = world.iter().next() else { return };
    let viewport = wcam.viewport.clone();
    let layers = layers.cloned().unwrap_or_default();
    let Some((e, mut cam, mut p, target, msaa, cur_layers)) = cams.iter_mut().next() else {
        if let Some((order, bb)) = want {
            commands.spawn((
                Camera3d { depth_load_op: Camera3dDepthLoadOp::Load, ..default() },
                Camera { order, is_active: true, clear_color: ClearColorConfig::None, output_mode: replace_output(), ..default() },
                Tonemapping::None,
                DebandDither::Disabled,
                fse.msaa,
                RenderTarget::Image(bb.into()),
                proj.clone(),
                layers,
                CAR_STENCIL_VIEW,
                CarStencilCamera,
                Transform::IDENTITY,
                ChildOf(we),
            ));
        }
        return;
    };
    let Some((order, bb)) = want else {
        if cam.is_active {
            cam.is_active = false;
        }
        return;
    };
    if cam.order != order || !cam.is_active {
        cam.order = order;
        cam.is_active = true;
    }
    // the world view's viewport (split screen: player 0's half)
    let same_vp = match (&cam.viewport, &viewport) {
        (None, None) => true,
        (Some(a), Some(b)) => a.physical_position == b.physical_position && a.physical_size == b.physical_size,
        _ => false,
    };
    if !same_vp {
        cam.viewport = viewport;
    }
    // copied every frame: also makes camera_system refresh the target size (see execute_passes)
    *p = proj.clone();
    let target_now = RenderTarget::Image(bb.into());
    if !target.is_some_and(|t| same_target(t, &target_now)) {
        commands.entity(e).insert(target_now);
    }
    if *msaa != fse.msaa {
        commands.entity(e).insert(fse.msaa);
    }
    if cur_layers != Some(&layers) {
        commands.entity(e).insert(layers);
    }
}

/// RainSurface's road cameras (`ROAD_SCENE_*`): children of the world camera (same view and depths) drawing only
/// their layer at the order of this frame's road slot, into the back buffer's shared colour and depth; plain views
/// (no glow or car stencil emulation), so the road materials write the bit 8 alpha themselves
#[derive(Component)]
pub struct RoadCamera(pub usize);

#[allow(clippy::type_complexity)]
fn update_road_cameras(
    mut commands: Commands,
    fse: Res<FullScreenEffects>,
    world: Query<(Entity, &Projection), (With<TrackCamera>, Without<FsePassCamera>, Without<RoadCamera>)>,
    mut cams: Query<(Entity, &RoadCamera, &mut Camera, &mut Projection, Option<&RenderTarget>, &Msaa), (Without<TrackCamera>, Without<FsePassCamera>)>,
) {
    let Some((we, proj)) = world.iter().next() else { return };
    let layer = |k: usize| RenderLayers::layer(if k == ROAD_SCENE_DETAIL { crate::rain_surface::ROAD_DETAIL_LAYER } else { crate::rain_surface::PUDDLE_LAYER });
    let mut have = [false; 2];
    for (e, rc, mut cam, mut p, target, msaa) in &mut cams {
        let k = rc.0.min(1);
        have[k] = true;
        let want = fse.road_orders[k].filter(|_| fse.active && !crate::golden::perf_off("road")).zip(fse.back_buffer.clone());
        let Some((order, bb)) = want else {
            if cam.is_active {
                cam.is_active = false;
            }
            continue;
        };
        if cam.order != order || !cam.is_active {
            cam.order = order;
            cam.is_active = true;
        }
        *p = proj.clone();
        let target_now = RenderTarget::Image(bb.into());
        if !target.is_some_and(|t| same_target(t, &target_now)) {
            commands.entity(e).insert(target_now);
        }
        if *msaa != fse.msaa {
            commands.entity(e).insert(fse.msaa);
        }
    }
    for k in 0..2 {
        if have[k] {
            continue;
        }
        let Some((order, bb)) = fse.road_orders[k].filter(|_| fse.active && !crate::golden::perf_off("road")).zip(fse.back_buffer.clone()) else { continue };
        commands.spawn((
            Camera3d { depth_load_op: Camera3dDepthLoadOp::Load, ..default() },
            Camera { order, is_active: true, clear_color: ClearColorConfig::None, output_mode: replace_output(), ..default() },
            Tonemapping::None,
            DebandDither::Disabled,
            fse.msaa,
            RenderTarget::Image(bb.into()),
            proj.clone(),
            layer(k),
            RoadCamera(k),
            Transform::IDENTITY,
            ChildOf(we),
        ));
    }
}

pub struct FullScreenEffectsPlugin;

impl Plugin for FullScreenEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MaterialPlugin::<FseMaterial>::default(), crate::fse_chain::FseChainPlugin))
            .init_resource::<FullScreenEffects>()
            .init_resource::<FseCameraPlanes>()
            .init_resource::<FsePool>()
            .configure_sets(
                PostUpdate,
                (FseSet::Build, FseSet::Execute).chain().after(bevy::transform::TransformSystems::Propagate),
            )
            .add_systems(PostUpdate, redirect_world.before(FseSet::Build))
            .add_systems(
                PostUpdate,
                (execute_passes, update_car_stencil_camera, update_road_cameras, crate::view_render::update_mirror_orders)
                    .chain()
                    .in_set(FseSet::Execute),
            );
    }

    fn finish(&self, app: &mut App) {
        if FSE_REGISTRY.get().is_some() {
            return;
        }
        if let Some(mut shaders) = app.world_mut().get_resource_mut::<Assets<Shader>>() {
            let r = load_fse_programs(&mut shaders);
            let _ = FSE_REGISTRY.set(r);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// every FSE program file assembles and translates (positive control: all 16 slots of 0x0015B2F0)
    #[test]
    fn assemble_fse_programs() {
        for name in FSE_VERTEX_PROGRAMS {
            let s = String::from_utf8_lossy(&visos::file::platform::read(&format!("\\Shaders\\{name}.vxu")).unwrap()).into_owned();
            fse_vertex_module(&s).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
        for name in FSE_FRAGMENT_PROGRAMS.iter().chain([&DANGER_PROGRAM]) {
            let s = String::from_utf8_lossy(&visos::file::platform::read(&format!("\\Shaders\\{name}.pxu")).unwrap()).into_owned();
            fse_fragment_module(&s).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
        let s = String::from_utf8_lossy(&visos::file::platform::read(&format!("\\Shaders\\{DANGER_PROGRAM}.vxu")).unwrap()).into_owned();
        fse_vertex_module(&s).unwrap_or_else(|e| panic!("{DANGER_PROGRAM}: {e}"));
    }

    /// D3D z 0.995 with the default planes lands in front of geometry farther than ~86 m
    #[test]
    fn device_depth_of_composite() {
        let fse = FullScreenEffects::default();
        let d = fse.device_depth(f32::from_bits(0x3f7eb852), 0.5, 600.0);
        let view_z = 0.5 / d;
        assert!((view_z - 85.8).abs() < 0.5, "{view_z}");
    }
}
