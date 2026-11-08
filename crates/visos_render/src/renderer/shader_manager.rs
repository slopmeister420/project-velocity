//! Renderer\source\Xbox\ShaderManager.cpp — `CShaderManager`: interprets a shader stage's command stream into
//! D3D render / texture stage states. The D3D device is replaced by `StageState`, which the bevy material
//! (`XboxStageMaterial`) turns into pipeline state and shader uniforms.

use bevy::asset::{embedded_asset, embedded_path};
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, ColorWrites, CompareFunction,
    RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

/// xbox `D3DCMP_*`
pub mod d3dcmp {
    pub const NEVER: u32 = 0x200;
    pub const LESS: u32 = 0x201;
    pub const EQUAL: u32 = 0x202;
    pub const LESSEQUAL: u32 = 0x203;
    pub const GREATER: u32 = 0x204;
    pub const NOTEQUAL: u32 = 0x205;
    pub const GREATEREQUAL: u32 = 0x206;
    pub const ALWAYS: u32 = 0x207;
}

/// xbox `D3DBLEND_*`
pub mod d3dblend {
    pub const ZERO: u32 = 0;
    pub const ONE: u32 = 1;
    pub const SRCCOLOR: u32 = 0x300;
    pub const INVSRCCOLOR: u32 = 0x301;
    pub const SRCALPHA: u32 = 0x302;
    pub const INVSRCALPHA: u32 = 0x303;
    pub const DESTALPHA: u32 = 0x304;
    pub const INVDESTALPHA: u32 = 0x305;
    pub const DESTCOLOR: u32 = 0x306;
}

/// `D3DBLENDOP_ADD` / `SUBTRACT` / `REVSUBTRACT`
pub const D3DBLENDOP_ADD: u32 = 0x8006;
pub const D3DBLENDOP_SUBTRACT: u32 = 0x800A;
pub const D3DBLENDOP_REVSUBTRACT: u32 = 0x800B;

/// ALPHAFUNC table (0x0065CE30), indexed by bits 1..3 of an `eMS_AlphaTest` record.
pub const ALPHA_FUNC_TABLE: [u32; 8] = [
    d3dcmp::NEVER,
    d3dcmp::ALWAYS,
    d3dcmp::LESS,
    d3dcmp::LESSEQUAL,
    d3dcmp::EQUAL,
    d3dcmp::GREATEREQUAL,
    d3dcmp::GREATER,
    d3dcmp::NOTEQUAL,
];
/// ZFUNC table (0x0065CE50), indexed by bits 17..18.
pub const Z_FUNC_TABLE: [u32; 4] = [d3dcmp::NEVER, d3dcmp::ALWAYS, d3dcmp::LESSEQUAL, d3dcmp::GREATEREQUAL];

/// blend mode parameters (ShaderManager.cpp 0x002A67C0): blend enable, src, dst, op, colour write mask
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlendMode {
    pub enable: bool,
    pub src: u32,
    pub dst: u32,
    pub op: u32,
    pub write_mask: u32,
}

/// D3DCOLORWRITEENABLE: 0x00010101 rgb, 0x01010101 rgba, 0x01000000 alpha
const WRITE_RGB: u32 = 0x0001_0101;
const WRITE_RGBA: u32 = 0x0101_0101;
const WRITE_A: u32 = 0x0100_0000;

/// ShaderManager.cpp 0x002A67C0 — `GetBlendModeParams`
pub fn get_blend_mode(mode: u32) -> Option<BlendMode> {
    use d3dblend::*;
    let b = |enable, op, write_mask, src, dst| Some(BlendMode { enable, src, dst, op, write_mask });
    match mode {
        0 => b(false, D3DBLENDOP_ADD, WRITE_RGB, ONE, ZERO),
        1 => b(false, D3DBLENDOP_ADD, WRITE_RGBA, ONE, ZERO),
        2 => b(true, D3DBLENDOP_ADD, WRITE_RGBA, SRCALPHA, INVSRCALPHA),
        3 => b(true, D3DBLENDOP_ADD, WRITE_RGB, SRCALPHA, INVSRCALPHA),
        4 => b(false, D3DBLENDOP_ADD, WRITE_A, ONE, ZERO),
        5 => b(true, D3DBLENDOP_ADD, WRITE_RGBA, DESTALPHA, INVDESTALPHA),
        6 => b(true, D3DBLENDOP_ADD, WRITE_RGB, DESTALPHA, INVDESTALPHA),
        7 => b(true, D3DBLENDOP_ADD, WRITE_RGBA, SRCALPHA, ONE),
        8 => b(true, D3DBLENDOP_ADD, WRITE_RGBA, DESTALPHA, ONE),
        9 => b(true, D3DBLENDOP_REVSUBTRACT, WRITE_RGBA, SRCALPHA, ONE),
        10 => b(true, D3DBLENDOP_SUBTRACT, WRITE_RGBA, DESTALPHA, ONE),
        11 => {
            log::error!("0 && \"Not implemented\"");
            None
        }
        12 => b(true, D3DBLENDOP_ADD, WRITE_RGB, SRCALPHA, ONE),
        13 => b(true, D3DBLENDOP_ADD, WRITE_RGB, DESTALPHA, ONE),
        14 => b(true, D3DBLENDOP_ADD, WRITE_RGBA, ONE, ONE),
        17 => b(true, D3DBLENDOP_ADD, WRITE_RGBA, ZERO, SRCALPHA),
        19 => b(true, D3DBLENDOP_ADD, WRITE_RGBA, SRCALPHA, ZERO),
        21 => b(true, D3DBLENDOP_ADD, WRITE_RGBA, ZERO, INVSRCALPHA),
        22 => b(true, D3DBLENDOP_ADD, WRITE_RGBA, DESTCOLOR, ZERO),
        23 => b(true, D3DBLENDOP_ADD, WRITE_RGB, DESTCOLOR, ZERO),
        // unlisted modes below 24 leave the outputs untouched (original returns 0 without writing)
        _ => None,
    }
}

/// `eMS_*` stage command codes seen in the data
pub mod ems {
    pub const END: u8 = 0;
    pub const BLEND_MODE: u8 = 1;
    pub const ALPHA_TEST: u8 = 2;
    pub const ADDRESS: u8 = 3;
    pub const NO_ZWRITE: u8 = 4;
    pub const DEPTH_BIAS: u8 = 5;
    pub const STENCIL: u8 = 6;
    pub const UV_ANIM: u8 = 22;
    pub const LAST_ONE: u8 = 28;
}

/// render state after a stage's commands, starting from the reset state the manager restores between shaders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageState {
    pub blend_mode: u32,
    pub z_enable: bool,
    pub z_func: u32,
    pub alpha_test: bool,
    pub alpha_func: u32,
    pub alpha_ref: u32,
    pub z_write: bool,
    /// D3DTADDRESS per axis: 1 wrap, 3 clamp
    pub address_u: u32,
    pub address_v: u32,
    /// value of the `eMS_DepthBias` record (projection z shifted by -value/1024)
    pub depth_bias: i16,
    pub stencil: bool,
    /// stencil REPLACE with bit 4 in the ref: the car sets (ShinyCar / BrakeLight ref 7, CarShader / ShineyWheel
    /// ref 4) — what the zone ring and the motion blur composite test (NOTEQUAL ref 4 mask 4)
    pub stencil_car: bool,
    pub uv_anim: bool,
    /// no back-face culling (effects drawn through vertex streams)
    pub two_sided: bool,
    /// RainSurface stencil bit 8 (`STENCIL_ROAD_*`), emulated in back-buffer alpha by the road cameras
    pub stencil_road: u8,
    /// an in-game car's opaque part (`car_shader_manager_pc_xbox::mark_car_pass`): drawn in bevy's transmissive phase,
    /// after the opaque world, so the dynamic envmap can snapshot the frame before the cars (CarRender 0x00119900)
    pub car_pass: bool,
}

/// no bit 8 use
pub const STENCIL_ROAD_NONE: u8 = 0;
/// "roaddetail" (RainSurface 0x00136C50: ALWAYS ref 8 REPLACE write mask 8, blend 4): the alpha write is kept and
/// clamped to 1..254 / 255 — a drawn road pixel; 0 = no bit, 255 = the car bit 4 (`FullScreenEffects::car_stencil`)
pub const STENCIL_ROAD_WRITE: u8 = 1;
/// puddles (0x00131690: EQUAL ref 8, KEEP): dropped where the back buffer (`XboxStageMaterial.stencil`) holds no
/// road alpha
pub const STENCIL_ROAD_TEST: u8 = 2;
/// shader defs of the two
pub const ROAD_STENCIL_WRITE: &str = "ROAD_STENCIL_WRITE";
pub const ROAD_STENCIL_TEST: &str = "ROAD_STENCIL_TEST";

impl Default for StageState {
    /// ShaderManager.cpp 0x002A4960 (reset half): blend 0, ZENABLE 1, ZFUNC LESSEQUAL, alpha test off,
    /// ZWRITE 1, addressing wrap.
    fn default() -> Self {
        Self {
            blend_mode: 0,
            z_enable: true,
            z_func: d3dcmp::LESSEQUAL,
            alpha_test: false,
            alpha_func: d3dcmp::ALWAYS,
            alpha_ref: 0,
            z_write: true,
            address_u: 1,
            address_v: 1,
            depth_bias: 0,
            stencil: false,
            stencil_car: false,
            uv_anim: false,
            two_sided: false,
            stencil_road: STENCIL_ROAD_NONE,
            car_pass: false,
        }
    }
}

/// ShaderManagerBase.cpp 0x002A2BE0's walk: does the command stream hold a record with this code
pub fn has_command(cmds: &[u8], code: u8) -> bool {
    let mut p = 0usize;
    while let Some(&c) = cmds.get(p) {
        if c == ems::END {
            return false;
        }
        if c == code {
            return true;
        }
        let size = cmds.get(p + 1).copied().unwrap_or(0) as usize;
        if size == 0 {
            return false;
        }
        p += size;
    }
    false
}

impl StageState {
    /// ShaderManager.cpp 0x002A4960 — `CShaderManager::SetShaderStage`, the records that touch state the port
    /// models. Program selection (13/14) and the texture-matrix records (9, 22-26) are not used by track data
    /// except `eMS_UvAnim` (22), which is only flagged here.
    pub fn from_commands(cmds: &[u8]) -> Self {
        let mut s = Self::default();
        s.apply_commands(cmds);
        s
    }


    /// ShaderManager.cpp 0x002A4240 — a shader with an override (shader +0, the car sets) runs SetShaderStage on its
    /// own stage without the closing reset, then on the override's stage: the override's records land on top.
    pub fn apply_commands(&mut self, cmds: &[u8]) {
        let s = self;
        let mut p = 0usize;
        while let Some(&code) = cmds.get(p) {
            if code == ems::END {
                break;
            }
            if code >= ems::LAST_ONE {
                log::error!("Code < eMS_LastOne");
            }
            let size = cmds.get(p + 1).copied().unwrap_or(0) as usize;
            let rec = &cmds[p..(p + size).min(cmds.len())];
            let byte = |i: usize| rec.get(i).copied().unwrap_or(0);
            let dword = |i: usize| rec.get(i..i + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).unwrap_or(0);
            match code {
                ems::BLEND_MODE => s.blend_mode = byte(2) as u32,
                ems::ALPHA_TEST => {
                    let v = dword(4);
                    s.z_enable = v & 0x10000 != 0;
                    if (v >> 16) & 1 != 0 {
                        s.z_func = Z_FUNC_TABLE[((v >> 17) & 3) as usize];
                    }
                    s.alpha_test = v & 1 != 0;
                    if s.alpha_test {
                        s.alpha_func = ALPHA_FUNC_TABLE[((v >> 1) & 7) as usize];
                        s.alpha_ref = (v >> 4) & 0xFF;
                    }
                }
                ems::ADDRESS => {
                    s.address_u = if byte(2) == 0 { 1 } else { 3 };
                    s.address_v = if byte(3) == 0 { 1 } else { 3 };
                }
                ems::NO_ZWRITE => s.z_write = false,
                ems::DEPTH_BIAS => s.depth_bias = i16::from_le_bytes([byte(2), byte(3)]),
                ems::STENCIL => s.stencil = true,
                ems::UV_ANIM => s.uv_anim = true,
                _ => {}
            }
            if size == 0 {
                break;
            }
            p += size;
        }
    }
}

/// ShaderManager.cpp 0x002A3320 (CShaderManager::CompileStage) — the `eMS_AlphaTest` (code 2) record of a code-built
/// stage, emitted when z is off, stage byte +58 is set, an alpha func is set or the z func is not 2. `alpha_func` /
/// `z_func` are the setter values (0x002A0F40: 0 = no test, else table index + 1; 0x002A1340: table index); stage
/// bytes +58/+59 and the dword at +28 are zero in every car set.
pub fn compile_alpha_test(alpha_func: u32, alpha_ref: u8, z_enable: bool, z_func: u32) -> Option<[u8; 8]> {
    if z_enable && alpha_func == 0 && z_func == 2 {
        return None;
    }
    let zf = if z_enable { z_func } else { 1 };
    let v = (zf << 17) | ((alpha_ref as u32) << 4) | (alpha_func != 0) as u32 | (alpha_func.saturating_sub(1) << 1) | 0x10000;
    let b = v.to_le_bytes();
    Some([ems::ALPHA_TEST, 8, 0, 0, b[0], b[1], b[2], b[3]])
}

/// pipeline key: everything that becomes fixed-function pipeline state.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct XboxStageKey {
    pub blend_mode: u32,
    pub z_write: bool,
    pub z_func: u32,
    pub z_enable: bool,
    pub two_sided: bool,
    pub stencil: bool,
    pub stencil_car: bool,
    pub stencil_road: u8,
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct XboxStageParams {
    pub alpha_ref: f32,
    pub alpha_func: u32,
    pub uv_offset: Vec2,
    /// material colour multiplied into the stage result (the sky's global colour, `dword_718230`); white otherwise
    pub colour: Vec4,
    /// fixed-function fog: rgb colour, a = D3DFOG_EXP density (0 = off)
    pub fog: Vec4,
    /// texture matrix scale (xy); uv' = uv * scale + uv_offset
    pub uv_scale: Vec4,
    /// x: 1 = COLORARG1 is the vertex diffuse (default), 0 = TFACTOR (`colour` alone);
    /// y: stage mode (`model_builder::STAGE_MODE_*`: 1 BrakeLight, 2 revlight; `colour` = TFACTOR);
    /// z: 1 = d3d lighting on a primitive without normals (diffuse rgb = `lit`, `XboxStageMaterial::set_lit`)
    pub arg: Vec4,
    /// fixed-function lighting result when `arg.z` = 1 (`VsLights::light_fixed_function(None, _)`)
    pub lit: Vec4,
    /// `arg.w` = 1: Characters.pxu c10 "SpecularLightColour" (`specular_chars`), t1 = `spec_texture` at uv 1
    pub spec: Vec4,
}

impl XboxStageParams {
    pub fn new(state: &StageState) -> Self {
        Self {
            alpha_ref: state.alpha_ref as f32,
            alpha_func: if state.alpha_test { state.alpha_func } else { d3dcmp::ALWAYS },
            uv_offset: Vec2::ZERO,
            colour: Vec4::ONE,
            fog: Vec4::ZERO,
            uv_scale: Vec4::ONE,
            arg: Vec4::new(1.0, 0.0, 0.0, 0.0),
            lit: Vec4::ONE,
            spec: Vec4::ZERO,
        }
    }
}

impl XboxStageMaterial {
    /// Renderer.cpp 0x002A8FD0: a stage with flags bit 0 drawn with a light set gets d3d lighting (0x002A8BE0);
    /// for primitives without a normal stream that is the set's ambient (`lights = None` turns lighting off).
    pub fn set_lit(&mut self, lights: Option<&crate::vs_render::VsLights>) {
        match lights {
            Some(l) => {
                self.params.arg.z = 1.0;
                self.params.lit = l.light_fixed_function(None, Vec4::ONE);
            }
            None => self.params.arg.z = 0.0,
        }
    }
}

/// fixed-function stage 0 as the shader manager leaves it for world geometry: COLOROP and ALPHAOP MODULATE
/// with ARG1 DIFFUSE and ARG2 TEXTURE (0x002A7120 reset), plus the stage's alpha test and blend state.
/// colour maths runs on gamma-space values, as on the xbox.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
#[bind_group_data(XboxStageKey)]
pub struct XboxStageMaterial {
    #[uniform(0)]
    pub params: XboxStageParams,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
    /// `STENCIL_ROAD_TEST`: the back buffer holding the road cameras' bit 8 alpha
    #[texture(3)]
    pub stencil: Option<Handle<Image>>,
    /// Characters.pxu t1 "SpecularTexture" (`params.arg.w` = 1)
    #[texture(4)]
    #[sampler(5)]
    pub spec_texture: Option<Handle<Image>>,
    pub state: StageState,
}

impl From<&XboxStageMaterial> for XboxStageKey {
    fn from(m: &XboxStageMaterial) -> Self {
        Self {
            blend_mode: m.state.blend_mode,
            z_write: m.state.z_write,
            z_func: m.state.z_func,
            z_enable: m.state.z_enable,
            two_sided: m.state.two_sided,
            stencil: m.state.stencil,
            stencil_car: m.state.stencil_car,
            stencil_road: m.state.stencil_road,
        }
    }
}

fn wgpu_factor(f: u32) -> BlendFactor {
    use d3dblend::*;
    match f {
        ZERO => BlendFactor::Zero,
        ONE => BlendFactor::One,
        SRCCOLOR => BlendFactor::Src,
        INVSRCCOLOR => BlendFactor::OneMinusSrc,
        SRCALPHA => BlendFactor::SrcAlpha,
        INVSRCALPHA => BlendFactor::OneMinusSrcAlpha,
        DESTALPHA => BlendFactor::DstAlpha,
        INVDESTALPHA => BlendFactor::OneMinusDstAlpha,
        DESTCOLOR => BlendFactor::Dst,
        _ => BlendFactor::One,
    }
}

/// d3d compare (near = small z) to bevy's reverse-z compare.
fn wgpu_compare(f: u32) -> CompareFunction {
    match f {
        d3dcmp::NEVER => CompareFunction::Never,
        d3dcmp::LESS => CompareFunction::Greater,
        d3dcmp::EQUAL => CompareFunction::Equal,
        d3dcmp::LESSEQUAL => CompareFunction::GreaterEqual,
        d3dcmp::GREATER => CompareFunction::Less,
        d3dcmp::NOTEQUAL => CompareFunction::NotEqual,
        d3dcmp::GREATEREQUAL => CompareFunction::LessEqual,
        _ => CompareFunction::Always,
    }
}

/// fixed-function pipeline state of a stage: culling, z test/write, blend factors and write mask.
pub fn apply_stage_pipeline_state(descriptor: &mut RenderPipelineDescriptor, blend_mode: u32, z_write: bool, z_func: u32, z_enable: bool) {
    // D3DRS_CULLMODE (xbox 0x93) is only ever set to D3DCULL_NONE: Renderer init 0x002A83CF, every shader
    // (0x002A4240, 0x002A4488), the reset 0x002A65CC, 0x002AA978. The data's strips mix windings (hangout.xmf).
    descriptor.primitive.cull_mode = None;
    if let Some(ds) = descriptor.depth_stencil.as_mut() {
        ds.depth_write_enabled = Some(z_write && z_enable);
        ds.depth_compare = Some(if z_enable { wgpu_compare(z_func) } else { CompareFunction::Always });
    }
    if let (Some(frag), Some(b)) = (descriptor.fragment.as_mut(), get_blend_mode(blend_mode)) {
        if let Some(Some(t)) = frag.targets.get_mut(0) {
            let op = match b.op {
                D3DBLENDOP_SUBTRACT => BlendOperation::Subtract,
                D3DBLENDOP_REVSUBTRACT => BlendOperation::ReverseSubtract,
                _ => BlendOperation::Add,
            };
            let comp = BlendComponent { src_factor: wgpu_factor(b.src), dst_factor: wgpu_factor(b.dst), operation: op };
            t.blend = b.enable.then_some(BlendState { color: comp, alpha: comp });
            let mut w = ColorWrites::empty();
            if b.write_mask & 0x0001_0000 != 0 {
                w |= ColorWrites::RED;
            }
            if b.write_mask & 0x0000_0100 != 0 {
                w |= ColorWrites::GREEN;
            }
            if b.write_mask & 0x0000_0001 != 0 {
                w |= ColorWrites::BLUE;
            }
            // framebuffer alpha only fed the original's bloom; an unblended alpha write (car paint writes its envmap
            // alpha) would make bevy's camera compositing show the sky cameras through opaque geometry
            if b.write_mask & 0x0100_0000 != 0 && b.enable {
                w |= ColorWrites::ALPHA;
            }
            t.write_mask = w;
        }
    }
}

/// view marker of the cameras that draw into the full-screen effects back buffer (world, sky and back-buffer passes):
/// their pipelines emulate the glow stencil in frame-buffer alpha (`apply_glow_stencil`). Bevy's view key has no
/// free bit; this renderer draws no shadows, so the shadow filter method carries it.
pub const GLOW_STENCIL_VIEW: bevy::light::ShadowFilteringMethod = bevy::light::ShadowFilteringMethod::Temporal;

pub fn is_glow_stencil_view(key: bevy::pbr::MeshPipelineKey) -> bool {
    use bevy::pbr::MeshPipelineKey as K;
    key.intersection(K::SHADOW_FILTER_METHOD_RESERVED_BITS) == K::SHADOW_FILTER_METHOD_TEMPORAL
}

/// ShaderManager.cpp 0x002A4960 stencil half, emulated in alpha. The original keeps STENCILPASS = ZERO (xbox op 0,
/// `0x002A6FA0(1)` on reset), so every draw clears the stencil it covers; `eMS_Stencil` records (all 607 in the
/// data carry the flag: ref |= 3, REPLACE) and the code-built ShinyCar / BrakeLight sets (0x00156C00 / 0x00156A60:
/// ref 7, REPLACE) mark bit 1 = glow; a blend 2 draw without a stencil record keeps it and writes rgb only (0x10101);
/// a blend 0 draw with one becomes blend 1. Bloom's "GlowStencil" pass (0x0015EF70: NOTEQUAL ref 2 mask 2, blend 4,
/// colour 0) then clears the alpha of every pixel without bit 1, so the bloom source is the alpha of the last
/// glow draw where no later draw cleared the bit. Here every non-glow draw writes alpha 0 instead (alpha factors
/// zero), which leaves the same alpha the GlowStencil pass would — without a stencil buffer (bevy's depth is
/// Depth32Float). Only for `is_glow_stencil_view` views; other cameras keep the plain blend state.
pub fn apply_glow_stencil(descriptor: &mut RenderPipelineDescriptor, blend_mode: u32, stencil: bool) {
    let Some(Some(t)) = descriptor.fragment.as_mut().and_then(|f| f.targets.get_mut(0)) else { return };
    let mode = if stencil && blend_mode == 0 { 1 } else { blend_mode };
    let Some(b) = get_blend_mode(mode) else { return };
    if stencil {
        if mode != blend_mode {
            // re-derive the colour state of blend 1 (rgba, no blending)
            t.blend = None;
            t.write_mask = ColorWrites::ALL;
        } else if b.write_mask & 0x0100_0000 != 0 {
            t.write_mask |= ColorWrites::ALPHA;
        }
    } else if blend_mode == 2 {
        t.write_mask &= !ColorWrites::ALPHA;
    } else {
        let zero = BlendComponent { src_factor: BlendFactor::Zero, dst_factor: BlendFactor::Zero, operation: BlendOperation::Add };
        let colour = t.blend.map(|s| s.color).unwrap_or(BlendComponent::REPLACE);
        t.blend = Some(BlendState { color: colour, alpha: zero });
        t.write_mask |= ColorWrites::ALPHA;
    }
}

/// view marker of the car stencil camera (`full_screen_effects_manager_pc_xbox`): it redraws the world into the back
/// buffer's alpha (after the bloom, which is the alpha's only other reader) as the original's stencil bit 4.
/// Must not be bevy's default (`Gaussian`, also used for cameras without the component): every plain camera
/// (the frontend's foyer camera, a world camera with the FSE off) would draw no colour — the black hangout.
pub const CAR_STENCIL_VIEW: bevy::light::ShadowFilteringMethod = bevy::light::ShadowFilteringMethod::Hardware2x2;
/// shader def of the pipelines that write the car bit (fragment returns alpha 1 after the alpha test)
pub const CAR_STENCIL_WRITE: &str = "CAR_STENCIL_WRITE";

pub fn is_car_stencil_view(key: bevy::pbr::MeshPipelineKey) -> bool {
    use bevy::pbr::MeshPipelineKey as K;
    key.intersection(K::SHADOW_FILTER_METHOD_RESERVED_BITS) == K::SHADOW_FILTER_METHOD_HARDWARE_2X2
}

/// view marker of a front-end camera whose frame-buffer alpha is read back (RainSurface 0x00137C50's FErefl paste,
/// blend 13 = rgb += src · dest alpha): there every draw writes alpha as the original's RGBA write masks do, instead
/// of the compositing-safe mask of `apply_stage_pipeline_state`. No shadow-filter value is left; this renderer's
/// materials never tonemap in the shader, so the tonemapping method carries it (`rain_surface::FeReflectionCameras`).
pub const FE_ALPHA_VIEW: bevy::core_pipeline::tonemapping::Tonemapping = bevy::core_pipeline::tonemapping::Tonemapping::Reinhard;

pub fn is_alpha_write_view(key: bevy::pbr::MeshPipelineKey) -> bool {
    use bevy::pbr::MeshPipelineKey as K;
    key.intersection(K::TONEMAP_METHOD_RESERVED_BITS) == K::TONEMAP_METHOD_REINHARD
}

/// on `FE_ALPHA_VIEW` views: the blend table's alpha write (0x0100_0000) also for unblended draws
pub fn apply_alpha_write(descriptor: &mut RenderPipelineDescriptor, blend_mode: u32) {
    let Some(b) = get_blend_mode(blend_mode) else { return };
    if b.write_mask & 0x0100_0000 == 0 {
        return;
    }
    if let Some(Some(t)) = descriptor.fragment.as_mut().and_then(|f| f.targets.get_mut(0).map(|t| t.as_mut())) {
        t.write_mask |= ColorWrites::ALPHA;
    }
}

#[cfg(test)]
mod view_marker_tests {
    use super::*;

    /// a camera without a `ShadowFilteringMethod` gets bevy's default in its view key: that must be a plain view,
    /// neither the glow nor the car stencil emulation (regression: the foyer camera drew nothing).
    #[test]
    fn default_view_is_plain() {
        use bevy::light::ShadowFilteringMethod as M;
        use bevy::pbr::MeshPipelineKey as K;
        let key = |m: M| match m {
            M::Hardware2x2 => K::SHADOW_FILTER_METHOD_HARDWARE_2X2,
            M::Gaussian => K::SHADOW_FILTER_METHOD_GAUSSIAN,
            M::Temporal => K::SHADOW_FILTER_METHOD_TEMPORAL,
        };
        let d = key(M::default());
        assert!(!is_car_stencil_view(d) && !is_glow_stencil_view(d));
        assert!(is_car_stencil_view(key(CAR_STENCIL_VIEW)) && !is_glow_stencil_view(key(CAR_STENCIL_VIEW)));
        assert!(is_glow_stencil_view(key(GLOW_STENCIL_VIEW)) && !is_car_stencil_view(key(GLOW_STENCIL_VIEW)));
    }
}

/// ShaderManager.cpp 0x002A4960 stencil half for bit 4, replayed in alpha: no colour or z writes, the draw's own z
/// test against the frame's depth (same pipeline vertex path, so equal depths pass LESSEQUAL again). Car sets
/// (REPLACE, ref with bit 4) write 1; other draws clear it (pass op ZERO) — alpha factors zero; a blend 2 draw
/// without a stencil record keeps it (KEEP, 0x002A6FA0(2)): no write at all.
pub fn apply_car_stencil(descriptor: &mut RenderPipelineDescriptor, blend_mode: u32, stencil: bool, stencil_car: bool) {
    if let Some(ds) = descriptor.depth_stencil.as_mut() {
        ds.depth_write_enabled = Some(false);
    }
    let Some(frag) = descriptor.fragment.as_mut() else { return };
    if stencil_car {
        frag.shader_defs.push(CAR_STENCIL_WRITE.into());
    }
    let Some(Some(t)) = frag.targets.get_mut(0) else { return };
    if stencil_car {
        t.blend = None;
        t.write_mask = ColorWrites::ALPHA;
    } else if !stencil && blend_mode == 2 {
        t.write_mask = ColorWrites::empty();
    } else {
        let zero = BlendComponent { src_factor: BlendFactor::Zero, dst_factor: BlendFactor::Zero, operation: BlendOperation::Add };
        t.blend = Some(BlendState { color: zero, alpha: zero });
        t.write_mask = ColorWrites::ALPHA;
    }
}

/// stencil bit 8 of RainSurface, emulated in back-buffer alpha (`STENCIL_ROAD_*`; only the road cameras draw these
/// materials): the writer keeps the blend table's alpha write (blend 4 = alpha only), the reader tests the back buffer
pub fn apply_road_stencil(descriptor: &mut RenderPipelineDescriptor, blend_mode: u32, stencil_road: u8) {
    let Some(frag) = descriptor.fragment.as_mut() else { return };
    match stencil_road {
        STENCIL_ROAD_WRITE => {
            frag.shader_defs.push(ROAD_STENCIL_WRITE.into());
            if let (Some(Some(t)), Some(b)) = (frag.targets.get_mut(0), get_blend_mode(blend_mode)) {
                if b.write_mask & WRITE_A != 0 {
                    t.write_mask |= ColorWrites::ALPHA;
                }
            }
        }
        STENCIL_ROAD_TEST => frag.shader_defs.push(ROAD_STENCIL_TEST.into()),
        _ => {}
    }
}

impl Material for XboxStageMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(bevy::asset::AssetPath::from_path_buf(embedded_path!("xbox_stage.wgsl")).with_source("embedded"))
    }

    fn alpha_mode(&self) -> AlphaMode {
        match get_blend_mode(self.state.blend_mode) {
            Some(b) if b.enable => AlphaMode::Blend,
            _ if self.state.alpha_test => AlphaMode::Mask(0.0),
            _ => AlphaMode::Opaque,
        }
    }

    fn reads_view_transmission_texture(&self) -> bool {
        self.state.car_pass
    }

    fn depth_bias(&self) -> f32 {
        self.state.depth_bias as f32
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let k = key.bind_group_data;
        apply_stage_pipeline_state(descriptor, k.blend_mode, k.z_write, k.z_func, k.z_enable);
        if is_glow_stencil_view(key.mesh_key) {
            apply_glow_stencil(descriptor, k.blend_mode, k.stencil);
        }
        if is_car_stencil_view(key.mesh_key) {
            apply_car_stencil(descriptor, k.blend_mode, k.stencil, k.stencil_car);
        }
        if is_alpha_write_view(key.mesh_key) {
            apply_alpha_write(descriptor, k.blend_mode);
        }
        if k.two_sided {
            descriptor.primitive.cull_mode = None;
        }
        apply_road_stencil(descriptor, k.blend_mode, k.stencil_road);
        Ok(())
    }
}

pub struct XboxStageMaterialPlugin;

impl Plugin for XboxStageMaterialPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "xbox_stage.wgsl");
        app.add_plugins(MaterialPlugin::<XboxStageMaterial>::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_typical_streams() {
        // Standard_0: alpha test GEQUAL 0, zfunc LESSEQUAL
        let s = StageState::from_commands(&[0x02, 0x08, 0, 0, 0x0b, 0x00, 0x05, 0x00, 0]);
        assert!(s.alpha_test && s.z_enable);
        assert_eq!((s.alpha_func, s.alpha_ref, s.z_func), (d3dcmp::GREATEREQUAL, 0, d3dcmp::LESSEQUAL));
        // Overlay: blend 3, no zwrite
        let s = StageState::from_commands(&[0x01, 0x04, 0x03, 0x00, 0x04, 0x04, 0, 0, 0]);
        assert_eq!(s.blend_mode, 3);
        assert!(!s.z_write);
    }
}
