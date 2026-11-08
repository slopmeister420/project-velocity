//! Renderer\source\Xbox\ShaderProgramManager.cpp + RenderShaderProgramSupport.cpp — programmable vertex/fragment
//! shaders. Programs are assembled from `Shaders\*.vxu|*.pxu` at startup (`NewShaderVersion` 0x002DEBB0, here the
//! `xgraphics_shader` translation into WGSL) and bound per draw: `BindShader` 0x002DF5A0 resolves every input of
//! the program's table (0x002DF7F0 vertex / 0x002DFBB0 fragment) against the shader's own inputs, then the global
//! resolver, and writes vertex constants (register + 96), pixel constants and stage textures.
//!
//! `ProgramMaterial` is the D3D replacement: constants in one uniform, textures t0..t3, the stage's blend/z state
//! as pipeline state, and the program pair selected in `specialize`.

use std::sync::{OnceLock, RwLock};

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError};
use bevy::shader::Shader;

use crate::game_shader_program_manager::{fragment_inputs, vertex_inputs, ProgramInput, FRAGMENT_PROGRAMS, VERTEX_PROGRAMS};
use crate::global_mat_input_resolver::{dynamic_input, global_id, gmi, mat_input_type, GlobalMatInputResolver};
use crate::renderer::shader_manager::{apply_stage_pipeline_state, StageState};
use crate::renderer::xgraphics_shader::{self, DynInput, COMMON_WGSL};
use crate::vs_render::VsLights;

/// a texture bound to a program stage. `linear`: an xbox linear (non-swizzled) texture, addressed in texels.
#[derive(Clone, Debug)]
pub struct ProgramTexture {
    pub image: Handle<Image>,
    pub size: UVec2,
    pub linear: bool,
}

#[derive(Clone, Debug)]
pub enum MatValue {
    Vector(Vec4),
    /// four registers as uploaded
    Matrix([Vec4; 4]),
    Texture(Option<ProgramTexture>),
}

/// a shader's own material input (`ShaderManagerBase` +0x460 list), matched by name
#[derive(Clone, Debug)]
pub struct MatInput {
    pub name: &'static str,
    pub value: MatValue,
}

/// per-object values the global resolver computes on the cpu: lights (GFXHelper.cpp 0x0012C670 / 0x0012C770)
/// and the stage-0 texture (`baseTexture`, 0x0012BF90).
#[derive(Clone, Debug, Default)]
pub struct ObjectInputs {
    pub lights: VsLights,
    pub base: Option<ProgramTexture>,
}

#[derive(Clone, Copy, Debug, PartialEq, ShaderType)]
pub struct ProgramParams {
    /// vertex constants c-96..c95
    pub vs: [Vec4; 192],
    /// pixel constants c0..c15
    pub ps: [Vec4; 16],
    /// per stage: width, height, linear (1/0), 0
    pub tex: [Vec4; 4],
    /// fixed-function fog: rgb colour, a = D3DFOG_EXP density (0 = off)
    pub fog: Vec4,
    /// x alpha ref (0..255), y alpha func (D3DCMP_*)
    pub alpha: Vec4,
}

impl Default for ProgramParams {
    fn default() -> Self {
        Self { vs: [Vec4::ZERO; 192], ps: [Vec4::ZERO; 16], tex: [Vec4::ZERO; 4], fog: Vec4::ZERO, alpha: Vec4::new(0.0, 0x207 as f32, 0.0, 0.0) }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProgramKey {
    pub vertex: u8,
    pub fragment: u8,
    pub blend_mode: u32,
    pub z_write: bool,
    pub z_func: u32,
    pub z_enable: bool,
    pub stencil: bool,
    pub stencil_car: bool,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
#[bind_group_data(ProgramKey)]
pub struct ProgramMaterial {
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
    /// stage 1 as a volume texture (`eMatInputDataType` 13: `lightbeamTexture`); `params.tex[1].w` = 1 selects it
    #[texture(9, dimension = "3d")]
    #[sampler(10)]
    pub t1_volume: Option<Handle<Image>>,
    pub vertex: u8,
    pub fragment: u8,
    pub state: StageState,
}

/// `ProgramMaterial`'s per-frame fields
#[derive(Clone)]
pub struct ProgramBinding {
    pub params: ProgramParams,
    pub t0: Option<Handle<Image>>,
    pub t1: Option<Handle<Image>>,
    pub t2: Option<Handle<Image>>,
    pub t3: Option<Handle<Image>>,
    pub t1_volume: Option<Handle<Image>>,
}

impl From<&ProgramMaterial> for ProgramKey {
    fn from(m: &ProgramMaterial) -> Self {
        Self {
            vertex: m.vertex,
            fragment: m.fragment,
            blend_mode: m.state.blend_mode,
            z_write: m.state.z_write,
            z_func: m.state.z_func,
            z_enable: m.state.z_enable,
            stencil: m.state.stencil,
            stencil_car: m.state.stencil_car,
        }
    }
}

impl ProgramMaterial {
    pub fn new(vertex: usize, fragment: usize, state: StageState) -> Self {
        let mut params = ProgramParams::default();
        if state.alpha_test {
            params.alpha = Vec4::new(state.alpha_ref as f32, state.alpha_func as f32, 0.0, 0.0);
        }
        Self { params, t0: None, t1: None, t2: None, t3: None, t1_volume: None, vertex: vertex as u8, fragment: fragment as u8, state }
    }

    /// perf: apply `f` (a per-frame `bind_inputs`) and write the asset only when constants or textures changed: every
    /// `get_mut` re-prepares the material (3.4 KB uniform + bind group; ~2 ms a frame in London for car + beam
    /// programs). `VELOCITY_PERF_OFF=bindopt` = always write
    pub fn rebind(mats: &mut Assets<ProgramMaterial>, h: &Handle<ProgramMaterial>, f: impl FnOnce(&mut ProgramMaterial)) {
        if crate::golden::perf_off("bindopt") {
            if let Some(mut m) = mats.get_mut(h) {
                f(&mut m);
            }
            return;
        }
        let Some(cur) = mats.get(h) else { return };
        let mut n = cur.clone();
        f(&mut n);
        let same = n.params == cur.params
            && n.t0 == cur.t0
            && n.t1 == cur.t1
            && n.t2 == cur.t2
            && n.t3 == cur.t3
            && n.t1_volume == cur.t1_volume;
        if !same {
            if let Some(mut m) = mats.get_mut(h) {
                *m = n;
            }
        }
    }

    /// the per-frame part (constants + textures) for a deferred write (`car_render::flush_car_programs`)
    pub fn binding(&self) -> ProgramBinding {
        ProgramBinding { params: self.params, t0: self.t0.clone(), t1: self.t1.clone(), t2: self.t2.clone(), t3: self.t3.clone(), t1_volume: self.t1_volume.clone() }
    }

    pub fn same_binding(&self, b: &ProgramBinding) -> bool {
        self.params == b.params && self.t0 == b.t0 && self.t1 == b.t1 && self.t2 == b.t2 && self.t3 == b.t3 && self.t1_volume == b.t1_volume
    }

    pub fn apply_binding(&mut self, b: ProgramBinding) {
        self.params = b.params;
        self.t0 = b.t0;
        self.t1 = b.t1;
        self.t2 = b.t2;
        self.t3 = b.t3;
        self.t1_volume = b.t1_volume;
    }

    /// ShaderProgramManager.cpp 0x002DF7F0 + 0x002DFBB0 — resolve both programs' inputs and write constants and
    /// textures. Fragment inputs go first so `textureNSize` sees this draw's textures.
    pub fn bind_inputs(&mut self, material: &[MatInput], obj: &ObjectInputs, globals: &GlobalMatInputResolver) {
        let fog = self.params.fog;
        let alpha = self.params.alpha;
        self.params = ProgramParams { fog, alpha, ..Default::default() };
        let mut stage: [Option<ProgramTexture>; 4] = Default::default();
        let mut volume = [false; 4];
        for inp in fragment_inputs(self.fragment as usize) {
            match resolve(&inp, material, obj, globals, &stage) {
                Some(MatValue::Texture(t)) => {
                    if let Some(s) = stage.get_mut(inp.reg as usize) {
                        *s = t;
                        volume[inp.reg as usize] = inp.ty == mat_input_type::TEXTURE_VOLUME;
                    }
                }
                Some(MatValue::Vector(v)) => write(&mut self.params.ps, inp.reg as usize, &[v]),
                Some(MatValue::Matrix(m)) => write(&mut self.params.ps, inp.reg as usize, &m),
                None => {}
            }
        }
        for inp in vertex_inputs(self.vertex as usize) {
            let base = inp.reg as usize + 96;
            match resolve(&inp, material, obj, globals, &stage) {
                Some(MatValue::Vector(v)) => write(&mut self.params.vs, base, &[v]),
                Some(MatValue::Matrix(m)) => write(&mut self.params.vs, base, &m),
                _ => {}
            }
        }
        for (k, s) in stage.iter().enumerate() {
            self.params.tex[k] = match s {
                Some(t) => Vec4::new(t.size.x as f32, t.size.y as f32, t.linear as u32 as f32, 0.0),
                None => Vec4::ZERO,
            };
        }
        let [a, b, c, d] = stage;
        self.t0 = a.map(|t| t.image);
        // only stage 1 is ever a volume (BeamLights.pxu t1)
        if volume[1] {
            self.params.tex[1] = Vec4::new(1.0, 1.0, 0.0, 1.0);
            self.t1_volume = b.map(|t| t.image);
            self.t1 = None;
        } else {
            self.t1 = b.map(|t| t.image);
            self.t1_volume = None;
        }
        self.t2 = c.map(|t| t.image);
        self.t3 = d.map(|t| t.image);
    }
}

fn write(dst: &mut [Vec4], at: usize, regs: &[Vec4]) {
    for (k, r) in regs.iter().enumerate() {
        if let Some(d) = dst.get_mut(at + k) {
            *d = *r;
        }
    }
}

/// ShaderManagerBase.cpp 0x002A08B0 — the shader's input of that name, else the global resolver
/// (GlobalMatInputResolver.cpp 0x0012D200 calls the input's resolver function), else the table's own value.
fn resolve(inp: &ProgramInput, material: &[MatInput], obj: &ObjectInputs, g: &GlobalMatInputResolver, stage: &[Option<ProgramTexture>; 4]) -> Option<MatValue> {
    if let Some(m) = material.iter().find(|m| m.name == inp.name) {
        return Some(m.value.clone());
    }
    if let Some(id) = global_id(inp.name) {
        if dynamic_input(id).is_some() {
            return None; // evaluated per vertex from the draw's transform and view
        }
        return Some(match id {
            gmi::LIGHTING_DIRS => {
                // GFXHelper.cpp 0x0012C670: D3DLIGHT8 direction rows, w = Range (0 for directional lights)
                let l = &obj.lights;
                MatValue::Matrix([l.dir[0].extend(0.0), l.dir[1].extend(0.0), l.dir[2].extend(0.0), Vec4::W])
            }
            gmi::LIGHTING_COLS => {
                // GFXHelper.cpp 0x0012C770: diffuse colours, row 3 = D3DRS_AMBIENT with alpha 1
                let l = &obj.lights;
                MatValue::Matrix([l.colour[0], l.colour[1], l.colour[2], l.ambient.truncate().extend(1.0)])
            }
            gmi::BASE_TEXTURE => MatValue::Texture(obj.base.clone()),
            // GFXHelper.cpp 0x0012C910: the beam object's +4, already transposed (0x001618D0)
            gmi::LIGHTBEAM_TEXTURE_TRANSFORM => MatValue::Matrix(g.beam_matrix),
            gmi::TEXTURE0_SIZE | gmi::TEXTURE1_SIZE | gmi::TEXTURE2_SIZE | gmi::TEXTURE3_SIZE => {
                // GFXHelper.cpp 0x0012BFB0: (w, h, 0, 1), 1x1 without a texture
                let s = stage[id - gmi::TEXTURE0_SIZE].as_ref().map(|t| t.size).unwrap_or(UVec2::ONE);
                MatValue::Vector(Vec4::new(s.x as f32, s.y as f32, 0.0, 1.0))
            }
            _ if inp.ty == mat_input_type::TEXTURE || inp.ty == mat_input_type::TEXTURE_VOLUME => {
                MatValue::Texture(g.textures[id].as_ref().map(|h| ProgramTexture { image: h.clone(), size: UVec2::ONE, linear: false }))
            }
            _ => MatValue::Vector(g.vectors[id]),
        });
    }
    inp.value.map(|v| MatValue::Vector(Vec4::from_array(v)))
}

/// assembled programs by game program index (ShaderProgramManager's handle table)
#[derive(Default)]
struct Registry {
    vertex: Vec<Option<Handle<Shader>>>,
    fragment: Vec<Option<Handle<Shader>>>,
}

static REGISTRY: OnceLock<RwLock<Registry>> = OnceLock::new();

/// the bevy-side wrapper around a translated vertex program: game constants come from the material uniform, the
/// matrices and camera direction from bevy's mesh and view (same values the game's resolvers produce).
pub(crate) const VERTEX_WRAPPER: &str = r#"
#import bevy_pbr::mesh_functions
#import bevy_pbr::mesh_view_bindings::view

struct ProgramParams {
    vs: array<vec4<f32>, 192>,
    ps: array<vec4<f32>, 16>,
    tex: array<vec4<f32>, 4>,
    fog: vec4<f32>,
    alpha: vec4<f32>,
};
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> prog: ProgramParams;

var<private> dyn_wvp: mat4x4<f32>;
var<private> dyn_world: mat4x4<f32>;
var<private> dyn_wv: mat4x4<f32>;
var<private> dyn_vp: mat4x4<f32>;
var<private> dyn_view: mat4x4<f32>;
var<private> dyn_camera_dir: vec4<f32>;

fn xvs_c(i: i32) -> vec4<f32> { return prog.vs[i]; }
fn xvs_c_rel(i: i32) -> vec4<f32> { return prog.vs[clamp(i, 0, 191)]; }

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
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
    let world = mesh_functions::get_world_from_local(vin.instance_index);
    dyn_world = world;
    dyn_view = view.view_from_world;
    dyn_vp = view.clip_from_world;
    dyn_wv = view.view_from_world * world;
    dyn_wvp = view.clip_from_world * world;
    dyn_camera_dir = vec4<f32>(-view.world_from_view[2].xyz, 0.0);
    var v: array<vec4<f32>, 16>;
    for (var k = 0; k < 16; k++) { v[k] = vec4<f32>(0.0, 0.0, 0.0, 1.0); }
    v[0] = vec4<f32>(vin.position, 1.0);
#ifdef VERTEX_NORMALS
    v[2] = vec4<f32>(vin.normal, 1.0);
#endif
#ifdef VERTEX_COLORS
    v[3] = vin.color;
#endif
#ifdef VERTEX_UVS_A
    v[9] = vec4<f32>(vin.uv, 0.0, 1.0);
#endif
    let o = xvs_program(v);
    var out: ProgramVaryings;
    out.position = o.pos;
    out.world_position = world * vec4<f32>(vin.position, 1.0);
    out.d0 = o.d0;
    out.d1 = o.d1;
    out.t0 = o.t0;
    out.t1 = o.t1;
    out.t2 = o.t2;
    out.t3 = o.t3;
    return out;
}
"#;

const FRAGMENT_WRAPPER: &str = r#"
#import bevy_pbr::mesh_view_bindings::view

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
@group(#{MATERIAL_BIND_GROUP}) @binding(9) var tex1v: texture_3d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(10) var samp1v: sampler;

fn xps_c(i: i32) -> vec4<f32> { return prog.ps[i]; }
// linear textures are addressed in texels on the nv2a
fn xps_uv(s: i32, c: vec4<f32>) -> vec2<f32> {
    let t = prog.tex[s];
    return select(c.xy, c.xy / max(t.xy, vec2<f32>(1.0)), t.z != 0.0);
}
fn xps_tex0(c: vec4<f32>) -> vec4<f32> { return textureSample(tex0, samp0, xps_uv(0, c)); }
// a volume texture on stage 1 is sampled with (s, t, r)
fn xps_tex1(c: vec4<f32>) -> vec4<f32> {
    let a = textureSample(tex1, samp1, xps_uv(1, c));
    let b = textureSample(tex1v, samp1v, c.xyz);
    return select(a, b, prog.tex[1].w != 0.0);
}
fn xps_tex2(c: vec4<f32>) -> vec4<f32> { return textureSample(tex2, samp2, xps_uv(2, c)); }
fn xps_tex3(c: vec4<f32>) -> vec4<f32> { return textureSample(tex3, samp3, xps_uv(3, c)); }

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

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

fn alpha_pass(a: f32) -> bool {
    let v = round(clamp(a, 0.0, 1.0) * 255.0);
    let r = prog.alpha.x;
    switch u32(prog.alpha.y) {
        case 0x200u: { return false; }
        case 0x201u: { return v < r; }
        case 0x202u: { return v == r; }
        case 0x203u: { return v <= r; }
        case 0x204u: { return v > r; }
        case 0x205u: { return v != r; }
        case 0x206u: { return v >= r; }
        default: { return true; }
    }
}

@fragment
fn fragment(in: ProgramVaryings) -> @location(0) vec4<f32> {
    var i: XpsIn;
    i.d0 = in.d0;
    i.d1 = in.d1;
    i.t0 = in.t0;
    i.t1 = in.t1;
    i.t2 = in.t2;
    i.t3 = in.t3;
    let c = xps_program(i);
    if (!alpha_pass(c.a)) {
        discard;
    }
#ifdef CAR_STENCIL_WRITE
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
#endif
    var rgb = c.rgb;
    // D3DFOG_EXP table fog after the final combiner, as the stage materials do
    if (prog.fog.a > 0.0) {
        let depth = -(view.view_from_world * vec4<f32>(in.world_position.xyz, 1.0)).z;
        rgb = mix(prog.fog.rgb, rgb, clamp(exp(-prog.fog.a * depth), 0.0, 1.0));
    }
    return vec4<f32>(srgb_to_linear(rgb), c.a);
}
"#;

/// full WGSL module of a vertex program (game convention constants + bevy wrapper)
pub fn vertex_module(index: usize, source: &str) -> Result<String, xgraphics_shader::AsmError> {
    let p = xgraphics_shader::parse(source)?;
    let dynamic: Vec<(i32, DynInput)> = vertex_inputs(index)
        .iter()
        .filter_map(|i| global_id(i.name).and_then(dynamic_input).map(|d| (i.reg as i32, d)))
        .collect();
    Ok(format!("{COMMON_WGSL}{VERTEX_WRAPPER}{}", xgraphics_shader::vertex_wgsl(&p, &dynamic)))
}

pub fn fragment_module(source: &str) -> Result<String, xgraphics_shader::AsmError> {
    let p = xgraphics_shader::parse(source)?;
    Ok(format!("{COMMON_WGSL}{FRAGMENT_WRAPPER}{}", xgraphics_shader::pixel_wgsl(&p)))
}

/// GameShaderProgramManager_PC_XBOX.cpp 0x0015D250 / 0x0015DB90 — load and assemble every game program from
/// `Shaders\`. A missing or broken file leaves its slot empty (the original asserts).
fn load_programs(shaders: &mut Assets<Shader>) -> Registry {
    let mut r = Registry::default();
    for (k, name) in VERTEX_PROGRAMS.iter().enumerate() {
        let src = visos::file::platform::read(&format!("\\Shaders\\{name}.vxu")).ok().map(|b| String::from_utf8_lossy(&b).into_owned());
        r.vertex.push(match src.as_deref().map(|s| vertex_module(k, s)) {
            Some(Ok(m)) => Some(shaders.add(Shader::from_wgsl(m, format!("visos_render/program/{name}.vxu.wgsl")))),
            Some(Err(e)) => {
                log::error!("false && \"Failed to load vertex shader file!\" {name}: {e}");
                None
            }
            None => {
                log::error!("false && \"Failed to load vertex shader file!\" {name}");
                None
            }
        });
    }
    for name in FRAGMENT_PROGRAMS {
        let src = visos::file::platform::read(&format!("\\Shaders\\{name}.pxu")).ok().map(|b| String::from_utf8_lossy(&b).into_owned());
        r.fragment.push(match src.as_deref().map(fragment_module) {
            Some(Ok(m)) => Some(shaders.add(Shader::from_wgsl(m, format!("visos_render/program/{name}.pxu.wgsl")))),
            other => {
                log::error!("nResult == ERR_OK: fragment program {name} ({:?})", other.map(|r| r.err()));
                None
            }
        });
    }
    r
}

fn program_shaders(v: u8, f: u8) -> (Option<Handle<Shader>>, Option<Handle<Shader>>) {
    match REGISTRY.get().and_then(|r| r.read().ok()) {
        Some(r) => (
            r.vertex.get(v as usize).cloned().flatten(),
            r.fragment.get(f as usize).cloned().flatten(),
        ),
        None => (None, None),
    }
}

impl Material for ProgramMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        match crate::renderer::shader_manager::get_blend_mode(self.state.blend_mode) {
            Some(b) if b.enable => AlphaMode::Blend,
            _ if self.state.alpha_test => AlphaMode::Mask(0.0),
            _ => AlphaMode::Opaque,
        }
    }

    fn reads_view_transmission_texture(&self) -> bool {
        self.state.car_pass
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let k = key.bind_group_data;
        let (vs, ps) = program_shaders(k.vertex, k.fragment);
        if let Some(vs) = vs {
            descriptor.vertex.shader = vs;
        }
        if let (Some(ps), Some(f)) = (ps, descriptor.fragment.as_mut()) {
            f.shader = ps;
        }
        apply_stage_pipeline_state(descriptor, k.blend_mode, k.z_write, k.z_func, k.z_enable);
        if crate::renderer::shader_manager::is_glow_stencil_view(key.mesh_key) {
            crate::renderer::shader_manager::apply_glow_stencil(descriptor, k.blend_mode, k.stencil);
        }
        if crate::renderer::shader_manager::is_car_stencil_view(key.mesh_key) {
            crate::renderer::shader_manager::apply_car_stencil(descriptor, k.blend_mode, k.stencil, k.stencil_car);
        }
        if crate::renderer::shader_manager::is_alpha_write_view(key.mesh_key) {
            crate::renderer::shader_manager::apply_alpha_write(descriptor, k.blend_mode);
        }
        Ok(())
    }
}

pub struct ProgramMaterialPlugin;

impl Plugin for ProgramMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ProgramMaterial>::default());
        if !app.world().contains_resource::<GlobalMatInputResolver>() {
            app.init_resource::<GlobalMatInputResolver>();
        }
    }

    fn finish(&self, app: &mut App) {
        if REGISTRY.get().is_some() {
            return;
        }
        if let Some(mut shaders) = app.world_mut().get_resource_mut::<Assets<Shader>>() {
            let r = load_programs(&mut shaders);
            let _ = REGISTRY.set(RwLock::new(r));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// every game program assembles and its full module (wrapper + translation) is accepted by bevy's
    /// preprocessor-free parse of the program part.
    #[test]
    fn assemble_game_programs() {
        for (k, name) in VERTEX_PROGRAMS.iter().enumerate() {
            let s = String::from_utf8_lossy(&visos::file::platform::read(&format!("\\Shaders\\{name}.vxu")).unwrap()).into_owned();
            vertex_module(k, &s).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
        for name in FRAGMENT_PROGRAMS {
            let s = String::from_utf8_lossy(&visos::file::platform::read(&format!("\\Shaders\\{name}.pxu")).unwrap()).into_owned();
            fragment_module(&s).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }
}
