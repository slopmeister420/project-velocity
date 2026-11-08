//! Platform layer for `full_screen_effects_manager_pc_xbox`: consecutive full-screen passes into off-screen images
//! in one graph run. The original draws every FSE quad on the one device; bevy needs a camera (a whole Core3d run)
//! per target switch, so the bloom ping-pong alone cost 11 runs. A chain camera (`FseChainView`, render graph
//! `FseChainGraph`, no Core3d, no main texture, output skipped) draws its passes in order straight into their
//! target images: load, the pass's own pipeline state (`apply_stage_pipeline_state`), the same programs and
//! uniforms as `FseMaterial`. `VELOCITY_PERF_OFF=fsechain` = one camera per pass as before.

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{CameraOutputMode, RenderTarget};
use bevy::ecs::schedule::{ScheduleBuildSettings, ScheduleLabel};
use bevy::mesh::{VertexAttributeValues, VertexBufferLayout};
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::binding_types::{sampler, texture_2d, uniform_buffer};
use bevy::render::render_resource::encase::UniformBuffer;
use bevy::render::render_resource::{
    BindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries, Buffer, BufferInitDescriptor, BufferUsages,
    CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, LoadOp, MultisampleState, Operations, PipelineCache,
    PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, SamplerBindingType,
    ShaderStages, SpecializedRenderPipeline, SpecializedRenderPipelines, StoreOp, TextureFormat, TextureSampleType, VertexFormat, VertexState, VertexStepMode,
};
use bevy::render::renderer::{RenderContext, RenderDevice, ViewQuery};
use bevy::render::texture::{FallbackImage, GpuImage};
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};
use bevy::shader::ShaderDefVal;
use std::ops::Range;

use crate::renderer::shader_manager::{apply_stage_pipeline_state, d3dcmp, get_blend_mode};
use crate::renderer::shader_program_manager::ProgramParams;

/// one pass of a chain: what `bind_pass` / `quad_mesh` made of it
#[derive(Clone)]
pub(crate) struct ChainPass {
    pub target: AssetId<Image>,
    pub program: u8,
    pub blend: u32,
    pub params: ProgramParams,
    /// stage textures (None = bevy's fallback, as `AsBindGroup` binds it)
    pub tex: [Option<AssetId<Image>>; 4],
    /// triangle strip: position, uv, colour
    pub verts: Vec<[f32; 9]>,
}

/// the passes a chain camera draws this frame
#[derive(Component, Clone, Default, ExtractComponent)]
pub(crate) struct FseChainView(pub Vec<ChainPass>);

#[derive(Component)]
pub(crate) struct FseChainCamera;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct FseChainGraph;

#[derive(Default, Resource)]
pub(crate) struct FseChainPool {
    cams: Vec<Entity>,
    dummy: Option<Handle<Image>>,
}

/// chain cameras spawned with the FSE slots (a race frame has 1-2 chains)
const PRESPAWN: usize = 4;

/// strip vertices of a `quad_mesh` (sequential indices: drawn unindexed)
pub(crate) fn mesh_vertices(m: &Mesh) -> Vec<[f32; 9]> {
    let (Some(VertexAttributeValues::Float32x3(p)), Some(VertexAttributeValues::Float32x2(uv)), Some(VertexAttributeValues::Float32x4(c))) =
        (m.attribute(Mesh::ATTRIBUTE_POSITION), m.attribute(Mesh::ATTRIBUTE_UV_0), m.attribute(Mesh::ATTRIBUTE_COLOR))
    else {
        return Vec::new();
    };
    p.iter().zip(uv).zip(c).map(|((p, u), c)| [p[0], p[1], p[2], u[0], u[1], c[0], c[1], c[2], c[3]]).collect()
}

/// one active chain camera per chain (`order`, passes), the rest inactive
pub(crate) fn assign_chain_cameras(
    commands: &mut Commands,
    pool: &mut FseChainPool,
    images: &mut Assets<Image>,
    cams: &mut Query<&mut Camera, (With<FseChainCamera>, Without<crate::full_screen_effects_manager_pc_xbox::FsePassCamera>)>,
    chains: Vec<(isize, Vec<ChainPass>)>,
) {
    let n = chains.len();
    // spawned up front (inactive) even with `fsechain` off, so on / off runs allocate the same entities (the race
    // particle golden shot follows entity order)
    while pool.cams.len() < n.max(PRESPAWN) {
        // the camera only needs a target with a size: a 1x1 image it never writes
        let dummy = pool.dummy.get_or_insert_with(|| images.add(Image::new_target_texture(1, 1, TextureFormat::Rgba8UnormSrgb, None))).clone();
        let e = commands
            .spawn((
                Camera { order: 0, is_active: false, clear_color: ClearColorConfig::None, output_mode: CameraOutputMode::Skip, ..default() },
                CameraRenderGraph::new(FseChainGraph),
                Projection::default(),
                RenderTarget::Image(dummy.into()),
                RenderLayers::none(),
                FseChainCamera,
            ))
            .id();
        pool.cams.push(e);
    }
    for (i, (order, passes)) in chains.into_iter().enumerate() {
        let e = pool.cams[i];
        match cams.get_mut(e) {
            Ok(mut cam) => {
                if cam.order != order || !cam.is_active {
                    cam.order = order;
                    cam.is_active = true;
                }
            }
            // spawned this frame: activated by the command
            Err(_) => {
                commands.entity(e).insert(Camera { order, is_active: true, clear_color: ClearColorConfig::None, output_mode: CameraOutputMode::Skip, ..default() });
            }
        }
        commands.entity(e).insert(FseChainView(passes));
    }
    for &e in pool.cams.iter().skip(n) {
        if let Ok(mut cam) = cams.get_mut(e) {
            if cam.is_active {
                cam.is_active = false;
                commands.entity(e).remove::<FseChainView>();
            }
        }
    }
}

// ---- render world -----------------------------------------------------------------------------------------------

#[derive(Resource)]
struct FseChainPipeline {
    layout: BindGroupLayoutDescriptor,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ChainKey {
    program: u8,
    blend: u32,
    format: TextureFormat,
}

impl SpecializedRenderPipeline for FseChainPipeline {
    type Key = ChainKey;

    fn specialize(&self, key: ChainKey) -> RenderPipelineDescriptor {
        let (vs, fs) = crate::full_screen_effects_manager_pc_xbox::chain_shaders(key.program).expect("checked in prepare");
        let mut d = RenderPipelineDescriptor {
            label: Some("fse_chain".into()),
            layout: vec![self.layout.clone()],
            vertex: VertexState {
                shader: vs,
                shader_defs: Vec::new(),
                entry_point: Some("vertex".into()),
                buffers: vec![VertexBufferLayout::from_vertex_formats(
                    VertexStepMode::Vertex,
                    [VertexFormat::Float32x3, VertexFormat::Float32x2, VertexFormat::Float32x4],
                )],
            },
            fragment: Some(FragmentState {
                shader: fs,
                shader_defs: vec![ShaderDefVal::UInt("MATERIAL_BIND_GROUP".into(), 0)],
                entry_point: Some("fragment".into()),
                // the opaque material pipeline's target before the stage state
                targets: vec![Some(ColorTargetState { format: key.format, blend: None, write_mask: ColorWrites::ALL })],
            }),
            primitive: PrimitiveState { topology: PrimitiveTopology::TriangleStrip, cull_mode: None, ..default() },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            ..default()
        };
        // FseMaterial::specialize: SetZState(0, ..) and the blend; alpha written when the mode writes it
        apply_stage_pipeline_state(&mut d, key.blend, false, d3dcmp::LESSEQUAL, false);
        if let (Some(b), Some(f)) = (get_blend_mode(key.blend), d.fragment.as_mut()) {
            if let Some(Some(t)) = f.targets.get_mut(0) {
                if b.write_mask & 0x0100_0000 != 0 {
                    t.write_mask |= ColorWrites::ALPHA;
                }
            }
        }
        d
    }
}

fn init_chain_pipeline(mut commands: Commands) {
    let tex = || texture_2d(TextureSampleType::Float { filterable: true });
    let smp = || sampler(SamplerBindingType::Filtering);
    let layout = BindGroupLayoutDescriptor::new(
        "fse_chain",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (uniform_buffer::<ProgramParams>(false), tex(), smp(), tex(), smp(), tex(), smp(), tex(), smp(), tex()),
        ),
    );
    commands.insert_resource(FseChainPipeline { layout });
}

struct ChainDraw {
    pipeline: CachedRenderPipelineId,
    bind: BindGroup,
    target: AssetId<Image>,
    verts: Range<u32>,
}

#[derive(Component)]
struct FseChainPrepared {
    vbuf: Option<Buffer>,
    draws: Vec<ChainDraw>,
}

#[allow(clippy::too_many_arguments)]
fn prepare_fse_chain(
    mut commands: Commands,
    views: Query<(Entity, &FseChainView)>,
    stale: Query<Entity, (With<FseChainPrepared>, Without<FseChainView>)>,
    pipeline: Res<FseChainPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<FseChainPipeline>>,
    cache: Res<PipelineCache>,
    images: Res<RenderAssets<GpuImage>>,
    fallback: Res<FallbackImage>,
    device: Res<RenderDevice>,
) {
    for e in &stale {
        commands.entity(e).remove::<FseChainPrepared>();
    }
    let layout = cache.get_bind_group_layout(&pipeline.layout);
    let fb = (&fallback.d2.texture_view, &fallback.d2.sampler);
    for (e, chain) in &views {
        let mut verts: Vec<[f32; 9]> = Vec::new();
        let mut draws = Vec::new();
        for p in &chain.0 {
            // a pass whose images are not uploaded yet is skipped, as an unprepared material is
            let Some(dst) = images.get(p.target) else { continue };
            if p.verts.is_empty() || crate::full_screen_effects_manager_pc_xbox::chain_shaders(p.program).is_none() {
                continue;
            }
            let tex = p.tex.map(|t| match t {
                None => Some(fb),
                Some(id) => images.get(id).map(|g| (&g.texture_view, &g.sampler)),
            });
            let [Some(a), Some(b), Some(c), Some(d)] = tex else { continue };
            let id = pipelines.specialize(&cache, &pipeline, ChainKey { program: p.program, blend: p.blend, format: dst.texture_descriptor.format });
            let mut ub = UniformBuffer::new(Vec::<u8>::new());
            if ub.write(&p.params).is_err() {
                continue;
            }
            let buf = device.create_buffer_with_data(&BufferInitDescriptor { label: Some("fse_chain_params"), contents: ub.as_ref(), usage: BufferUsages::UNIFORM });
            let bind = device.create_bind_group(
                "fse_chain",
                &layout,
                &BindGroupEntries::sequential((buf.as_entire_binding(), a.0, a.1, b.0, b.1, c.0, c.1, d.0, d.1, fb.0)),
            );
            let start = verts.len() as u32;
            verts.extend_from_slice(&p.verts);
            draws.push(ChainDraw { pipeline: id, bind, target: p.target, verts: start..verts.len() as u32 });
        }
        let vbuf = (!verts.is_empty()).then(|| {
            let bytes: Vec<u8> = verts.iter().flatten().flat_map(|f| f.to_le_bytes()).collect();
            device.create_buffer_with_data(&BufferInitDescriptor { label: Some("fse_chain_verts"), contents: &bytes, usage: BufferUsages::VERTEX })
        });
        commands.entity(e).insert(FseChainPrepared { vbuf, draws });
    }
}

/// the chain camera's whole graph: each pass in its own render pass on its target image (load / store)
fn run_fse_chain(view: ViewQuery<&FseChainPrepared>, cache: Res<PipelineCache>, images: Res<RenderAssets<GpuImage>>, mut ctx: RenderContext) {
    let prepared = view.into_inner();
    let Some(vbuf) = &prepared.vbuf else { return };
    for d in &prepared.draws {
        let (Some(pipe), Some(dst)) = (cache.get_render_pipeline(d.pipeline), images.get(d.target)) else { continue };
        let mut pass = ctx.command_encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("fse_chain"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &dst.texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations { load: LoadOp::Load, store: StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipe);
        pass.set_bind_group(0, &*d.bind, &[]);
        pass.set_vertex_buffer(0, *vbuf.slice(..));
        pass.draw(d.verts.clone(), 0..1);
    }
}

pub struct FseChainPlugin;

impl Plugin for FseChainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FseChainPool>().add_plugins(ExtractComponentPlugin::<FseChainView>::default());
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return };
        let mut schedule = Schedule::new(FseChainGraph);
        schedule.set_build_settings(ScheduleBuildSettings { auto_insert_apply_deferred: false, ..default() });
        render_app
            .add_schedule(schedule)
            .init_resource::<SpecializedRenderPipelines<FseChainPipeline>>()
            .add_systems(RenderStartup, init_chain_pipeline)
            .add_systems(Render, prepare_fse_chain.in_set(RenderSystems::PrepareBindGroups))
            .add_systems(FseChainGraph, run_fse_chain);
    }
}
