//! CarShaderManager_PC_XBOX.cpp — the dynamic envmap draw (0x001598A0 from CarRender 0x00119900 before any car is
//! drawn, 0x00159D00 rings, 0x0015A090 specular spots, 0x00159C10 end of the car pass). The object and its maths
//! are DangerZoneRender.cpp's (`danger_zone_render::dynamic_envmap`).
//!
//! Bevy side: the original warps the back buffer as it stands before the cars (world + sky) into the "Envmap"
//! target, no extra scene draw. Here in-game car parts (`CAR_LAYER`) draw in bevy's transmissive phase
//! (`StageState.car_pass`), after the opaque world in the world camera's own pass; between the two the main
//! texture is copied into the envmap source (`capture_envmap_source`). The envmap passes are early FSE passes
//! (`FsePass.early`) before the world camera, so they warp the previous frame's copy (one frame late; the original
//! is same-frame).

use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::core_3d::main_opaque_pass_3d;
use bevy::core_pipeline::schedule::Core3d;
use bevy::core_pipeline::Core3dSystems;
use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::pbr::{main_transmissive_pass_3d, ScreenSpaceTransmission};
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::render::renderer::{RenderContext, ViewQuery};
use bevy::render::texture::GpuImage;
use bevy::render::view::ViewTarget;
use bevy::render::RenderApp;

use crate::car_render::{CarBodyMeshes, CarRender, CarShadow};
use crate::danger_zone_render::dynamic_envmap::{self as de, EnvmapTweaks};
use crate::full_screen_effects_manager_pc_xbox::{
    prog, FseImage, FseSet, FseShader, FseVertex, FullScreenEffects, FFP_MODULATE, FFP_MODULATE2X, WHITE,
};
use crate::light_streak::visos_colour;
use crate::renderer::shader_manager::XboxStageMaterial;
use crate::renderer::shader_program_manager::{ProgramMaterial, ProgramTexture};
use crate::track_render::{TrackCamera, TrackPointLights};

/// render layer of in-game car meshes while the dynamic envmap runs (kept out of the envmap source view)
pub const CAR_LAYER: usize = 20;

/// CarPalette.cpp 0x0010D7E0 -> DangerZoneRender.cpp 0x001214A0 (CarRender init, GameManager::CommonInit): `front_end`
/// = CommonInit's argument, `two_players` = `byte_6F7157` (enabled = single player)
#[derive(Message, Clone, Copy, Debug)]
pub struct DynamicEnvmapInit {
    pub front_end: bool,
    pub two_players: bool,
}

/// DangerZoneRender.cpp 0x00121840 / 0x00121860 (destructor list): the target is released
#[derive(Message, Clone, Copy, Debug)]
pub struct DynamicEnvmapShutdown;

/// the car the envmap is drawn round (CarRender 0x00119900: `VsAIRaceVehicle__24F90()` +4, the player's car index);
/// `None` = the lowest `CarRender.request` (race car 0)
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct DynamicEnvmapInput {
    pub player: Option<u64>,
}

/// the object `byte_709F00` (see `danger_zone_render::dynamic_envmap`)
#[derive(Resource, Debug)]
pub struct DynamicEnvmap {
    /// +0 the "Envmap" render target (fmt 18 = linear A8R8G8B8; gamma values in `Rgba8Unorm` here)
    pub target: Option<FseImage>,
    pub image: Option<Handle<Image>>,
    /// `dword_686924` (= `dword_686928`)
    pub size_log2: u32,
    /// +56 the target is the hi-res one
    pub hi_res: bool,
    /// +57 "enabled"
    pub enabled: bool,
    /// +58 drawn in this car pass (set by the init, so the first car pass after it draws nothing)
    pub drawn: bool,
    /// `byte_687558` (static 1): the normal bend is still to do — once per run, not reset by an init
    pub bend_pending: bool,
    pub tweaks: EnvmapTweaks,
    /// +36 "lightglow_white" (the spot shader's texture, `dword_68757C` = 1), +48 "fresnel4" (`dword_687554` = 4)
    lightglow: Option<FseImage>,
    fresnel: Option<FseImage>,
    /// the back buffer without cars (see the module doc)
    source: Option<FseImage>,
    source_image: Option<Handle<Image>>,
    source_size: UVec2,
}

impl Default for DynamicEnvmap {
    fn default() -> Self {
        Self {
            target: None,
            image: None,
            size_log2: de::SIZE_LOG2_BOOT,
            hi_res: false,
            enabled: false,
            drawn: false,
            bend_pending: de::BEND,
            // `VELOCITY_ENVMAP_DISPLAY=1` = the "display" tweak
            tweaks: EnvmapTweaks { display: std::env::var("VELOCITY_ENVMAP_DISPLAY").is_ok_and(|v| v == "1"), ..default() },
            lightglow: None,
            fresnel: None,
            source: None,
            source_image: None,
            source_size: UVec2::ZERO,
        }
    }
}

impl DynamicEnvmap {
    /// DangerZoneRender.cpp 0x00122AA0 — the target's texture for the car sets while enabled (+57)
    pub fn program_texture(&self) -> Option<ProgramTexture> {
        if !self.enabled {
            return None;
        }
        let image = self.image.clone()?;
        let s = 1u32 << self.size_log2;
        Some(ProgramTexture { image, size: UVec2::splat(s), linear: true })
    }
}

/// RenderTargetManager 0x002EEA10 ("Envmap", 1 << n, 1 << n, fmt 18): a target holding the gamma values as written
fn envmap_image(size: u32) -> Image {
    let mut img = Image::new_target_texture(size, size, TextureFormat::Rgba8Unorm, None);
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::linear());
    img
}

fn source_image(size: UVec2) -> Image {
    let mut img = Image::new_target_texture(size.x.max(1), size.y.max(1), TextureFormat::Rgba8UnormSrgb, None);
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::linear());
    img
}

/// a misc.vcd texture as an FSE source (a swizzled texture: uv 0..1)
fn misc_texture(fse: &mut FullScreenEffects, images: &mut Assets<Image>, name: &str) -> Option<FseImage> {
    let mut tex = crate::model_builder::TexSet::default();
    crate::car_render::add_vcd_textures(&mut tex, "\\VFX\\misc.vcd");
    let key = name.to_ascii_lowercase();
    let (w, h) = tex.by_name.get(&key).map(|t| (t.width, t.height))?;
    let image = tex.image(&key, 3, 3, images)?;
    Some(fse.add_target(image, UVec2::new(w, h), false, false))
}

/// DangerZoneRender.cpp 0x001214A0 / 0x00121860
fn envmap_messages(
    mut init: MessageReader<DynamicEnvmapInit>,
    mut shutdown: MessageReader<DynamicEnvmapShutdown>,
    mut env: ResMut<DynamicEnvmap>,
    mut fse: ResMut<FullScreenEffects>,
    mut images: ResMut<Assets<Image>>,
) {
    for _ in shutdown.read() {
        env.target = None;
        env.image = None;
        env.enabled = false;
    }
    for m in init.read() {
        if m.front_end {
            env.drawn = false;
            env.enabled = false;
            continue;
        }
        env.enabled = !m.two_players;
        env.drawn = true;
        env.size_log2 = de::SIZE_LOG2_BOOT;
        let s = 1u32 << env.size_log2;
        let image = images.add(envmap_image(s));
        match env.target {
            Some(t) => fse.set_target(t, image.clone(), UVec2::splat(s)),
            None => env.target = Some(fse.add_target(image.clone(), UVec2::splat(s), false, true)),
        }
        env.image = Some(image);
        if env.lightglow.is_none() {
            env.lightglow = misc_texture(&mut fse, &mut images, "lightglow_white");
            env.fresnel = misc_texture(&mut fse, &mut images, "fresnel4");
        }
        env.hi_res = false;
        log::info!("dynamic envmap: init (enabled {})", env.enabled);
    }
}

/// in-game cars draw on `CAR_LAYER` while the envmap is on; the world camera sees the layer too
fn layer_cars(
    mut commands: Commands,
    env: Res<DynamicEnvmap>,
    cars: Query<(Entity, &CarRender)>,
    children: Query<&Children>,
    unlayered: Query<(), (With<Mesh3d>, Without<RenderLayers>)>,
    cams: Query<(Entity, Option<&RenderLayers>), With<TrackCamera>>,
) {
    if !env.enabled {
        return;
    }
    for (root, car) in &cars {
        if car.spec.frontend {
            continue;
        }
        for e in children.iter_descendants(root) {
            if unlayered.contains(e) {
                commands.entity(e).insert(RenderLayers::layer(CAR_LAYER));
            }
        }
    }
    for (e, l) in &cams {
        let l = l.cloned().unwrap_or_default();
        if !l.intersects(&RenderLayers::layer(CAR_LAYER)) {
            commands.entity(e).insert(l.with(CAR_LAYER));
        }
    }
}

/// on the world camera while the envmap runs: its main texture is copied into `image` after the opaque pass, i.e.
/// the back buffer as it stands before the cars (`StageState.car_pass` parts draw in the transmissive phase)
#[derive(Component, Clone, PartialEq, ExtractComponent)]
pub struct EnvmapCaptureView {
    image: AssetId<Image>,
}

/// the envmap source: the back buffer's size (0x00159D00 reads the back buffer itself, `dword_7A5780`), filled by
/// `capture_envmap_source` in the world camera's own pass — no second scene view
fn update_envmap_source(
    mut commands: Commands,
    mut env: ResMut<DynamicEnvmap>,
    mut fse: ResMut<FullScreenEffects>,
    mut images: ResMut<Assets<Image>>,
    world: Query<(Entity, Option<&EnvmapCaptureView>), With<TrackCamera>>,
) {
    let want = env.enabled && fse.active && env.target.is_some() && !crate::golden::perf_off("envmap");
    let size = fse.size().max(UVec2::ONE);
    if want && (env.source.is_none() || env.source_size != size) {
        let image = images.add(source_image(size));
        match env.source {
            Some(t) => fse.set_target(t, image.clone(), size),
            None => env.source = Some(fse.add_target(image.clone(), size, true, true)),
        }
        env.source_image = Some(image);
        env.source_size = size;
    }
    let cap = env.source_image.as_ref().filter(|_| want).map(|h| EnvmapCaptureView { image: h.id() });
    // the first world camera (player 0; two players have no envmap)
    let mut first = true;
    for (e, have) in &world {
        let mine = if first { cap.as_ref() } else { None };
        first = false;
        match (mine, have) {
            (Some(c), h) if h != Some(c) => {
                commands.entity(e).insert(c.clone());
            }
            (None, Some(_)) => {
                commands.entity(e).remove::<EnvmapCaptureView>();
            }
            _ => {}
        }
    }
}

/// render world, the world camera's Core3d schedule between the opaque and the transmissive pass: main texture ->
/// envmap source (read by the next frame's early envmap passes: one frame late, see "Performance" in the docs)
fn capture_envmap_source(
    view: ViewQuery<(&ViewTarget, &EnvmapCaptureView)>,
    images: Res<RenderAssets<GpuImage>>,
    mut ctx: RenderContext,
) {
    let (target, cap) = view.into_inner();
    let Some(dst) = images.get(cap.image) else { return };
    let src = target.main_texture();
    if src.format().remove_srgb_suffix() != dst.texture_descriptor.format.remove_srgb_suffix() || src.sample_count() != 1 {
        return;
    }
    let (s, d) = (src.size(), dst.texture_descriptor.size);
    let extent = Extent3d { width: s.width.min(d.width), height: s.height.min(d.height), depth_or_array_layers: 1 };
    ctx.command_encoder().copy_texture_to_texture(src.as_image_copy(), dst.texture.as_image_copy(), extent);
}

/// no material reads bevy's transmission texture: cameras skip its copy (the car parts only use the phase's order)
fn no_transmission_copies(mut cams: Query<&mut ScreenSpaceTransmission, Added<Camera3d>>) {
    for mut t in &mut cams {
        t.steps = 0;
    }
}

/// in-game car parts (`CAR_LAYER` or a view car layer) draw after the opaque world: `StageState.car_pass`
#[allow(clippy::type_complexity)]
fn mark_car_pass(
    env: Res<DynamicEnvmap>,
    stage: Query<(&RenderLayers, &MeshMaterial3d<XboxStageMaterial>), Or<(Changed<RenderLayers>, Changed<MeshMaterial3d<XboxStageMaterial>>)>>,
    program: Query<(&RenderLayers, &MeshMaterial3d<ProgramMaterial>), Or<(Changed<RenderLayers>, Changed<MeshMaterial3d<ProgramMaterial>>)>>,
    mut stage_mats: ResMut<Assets<XboxStageMaterial>>,
    mut program_mats: ResMut<Assets<ProgramMaterial>>,
) {
    if !env.enabled {
        return;
    }
    let [v0, v1] = crate::view_render::VIEW_CAR_LAYERS;
    let car = RenderLayers::from_layers(&[CAR_LAYER, v0, v1]);
    for (l, m) in &stage {
        if l.intersects(&car) && stage_mats.get(&m.0).is_some_and(|m| !m.state.car_pass) {
            if let Some(mut m) = stage_mats.get_mut(&m.0) {
                m.state.car_pass = true;
            }
        }
    }
    for (l, m) in &program {
        if l.intersects(&car) && program_mats.get(&m.0).is_some_and(|m| !m.state.car_pass) {
            if let Some(mut m) = program_mats.get_mut(&m.0) {
                m.state.car_pass = true;
            }
        }
    }
}

/// what one car pass of the envmap reads: the camera (position, basis, vertical fov), the player car (world matrix,
/// +932 ground slope) and the point lights
pub struct EnvmapView<'a> {
    pub cam_pos: Vec3,
    pub cam_right: Vec3,
    pub cam_up: Vec3,
    pub cam_fwd: Vec3,
    pub tan_half_fov: f32,
    pub car: Vec3,
    pub ground: Vec3,
    pub lights: &'a [crate::vs_render::PointLight],
}

impl EnvmapView<'_> {
    /// 0x0015EC80 y: 0..1 down the screen
    fn project_y(&self, p: Vec3) -> f32 {
        let d = p - self.cam_pos;
        let w = self.cam_fwd.dot(d);
        let y = self.cam_up.dot(d) / (w * self.tan_half_fov);
        ((0.0 - y) + 1.0) / 2.0
    }

    /// D3DTS_VIEW rows 0..2 (x right, y up, z forward)
    fn view_rows(&self) -> [Vec3; 3] {
        let (r, u, f) = (self.cam_right, self.cam_up, self.cam_fwd);
        [Vec3::new(r.x, u.x, f.x), Vec3::new(r.y, u.y, f.y), Vec3::new(r.z, u.z, f.z)]
    }
}

/// CarShaderManager_PC_XBOX.cpp 0x00159D00 + 0x001598A0's passes + 0x0015A090 — queued as early FSE passes into
/// the envmap target: clear (0x002EF0B0 colour 0), the disc and the band warped from `source` (shader +28: blend 1, z
/// off, MODULATE2X), one blur pass (0x0015C770), subtract (`dword_686948` & 0x10: 0x0015C340 Colour quad VISOS(subtract/2)
/// through +24 blend 9 — its default z test has no depth buffer to fail against), fresnel (& 0x20: "fresnel4"
/// through +20 blend 21 with Copy, VISOS(0x70)), the spots (& 8: "lightglow_white" quads through +16 blend 14, z off;
/// one 0x0015E7B0 draw each, joined into one strip here)
pub fn render_envmap(fse: &mut FullScreenEffects, t: &EnvmapTweaks, env: FseImage, source: FseImage, glow: Option<FseImage>, fresnel: Option<FseImage>, view: &EnvmapView) {
    fse.set_early(true);
    fse.draw(env, source, FseShader::new(1, false), prog::COLOUR, None, None, 0.0, 0x0000_0000);
    let h = if de::HORIZON { de::horizon(view.car, view.cam_fwd, view.ground, |p| view.project_y(p)) } else { 0.5 };
    let size = fse.image_size(source).as_vec2();
    let ring = FseShader { ffp: FFP_MODULATE2X, ..FseShader::new(1, false) };
    if de::RING_DISC {
        fse.draw_strip(env, source, ring, prog::COPY, de::ring_strip(t, de::RING_SEGMENTS, 0.0, de::RING_MID, false, h, size));
    }
    if de::RING_BAND {
        fse.draw_strip(env, source, ring, prog::COPY, de::ring_strip(t, de::RING_SEGMENTS, de::RING_MID, de::RING_OUTER, true, h, size));
    }
    if t.blur != 0 {
        fse.blur_passes(env, env, 1, false);
    }
    if de::FLAGS & de::FLAG_SUBTRACT != 0 {
        let c = visos_colour([(t.subtract / 2) as u8, 0x80, 0x80, 0x80]);
        fse.draw(env, source, FseShader::new(9, false), prog::COLOUR, None, None, 0.0, c);
    }
    if de::FLAGS & de::FLAG_FRESNEL != 0 {
        if let Some(f) = fresnel {
            let c = visos_colour([t.fresnel, 0x80, 0x80, 0x80]);
            fse.draw(env, f, FseShader::new(21, false), prog::COPY, None, None, 0.0, c);
        }
    }
    if de::FLAGS & de::FLAG_SPOTS != 0 {
        if let Some(g) = glow {
            let spots = de::specular_spots(t, view.lights, view.car, view.view_rows());
            let mut verts: Vec<FseVertex> = Vec::new();
            for s in &spots {
                let q = de::spot_quad(t, s);
                if let Some(&last) = verts.last() {
                    verts.push(last);
                    verts.push(q[0]);
                }
                verts.extend_from_slice(&q);
            }
            fse.draw_strip(env, g, FseShader { ffp: FFP_MODULATE, ..FseShader::new(14, false) }, prog::COPY, verts);
        }
    }
    fse.set_early(false);
}

/// DangerZoneRender.cpp 0x00121BB0 / 0x00121C80 on a car's meshes (positions are in car model space here)
fn bend_meshes(meshes: &mut Assets<Mesh>, list: &CarBodyMeshes) -> usize {
    let mut n = 0;
    for h in &list.0 {
        let Some(mut m) = meshes.get_mut(h) else { continue };
        let Some(pos) = m.attribute(Mesh::ATTRIBUTE_POSITION).and_then(|a| a.as_float3()).map(<[[f32; 3]]>::to_vec) else { continue };
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(nrm)) = m.attribute_mut(Mesh::ATTRIBUTE_NORMAL) else { continue };
        for (p, nn) in pos.iter().zip(nrm.iter_mut()) {
            *nn = de::bend_normal(Vec3::from_array(*p), Vec3::from_array(*nn)).to_array();
        }
        n += 1;
    }
    n
}

/// CarRender.cpp 0x00119900's envmap calls per rendered frame: 0x001598A0 (when +57 and the main view) then, after
/// the cars, 0x00159C10 (+58 cleared; the "display" overlay is off)
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn draw_dynamic_envmap(
    mut fse: ResMut<FullScreenEffects>,
    mut env: ResMut<DynamicEnvmap>,
    input: Res<DynamicEnvmapInput>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    points: Option<Res<TrackPointLights>>,
    camera: Query<(&GlobalTransform, &Projection), With<TrackCamera>>,
    cars: Query<(&GlobalTransform, &CarRender, Option<&CarShadow>, Option<&CarBodyMeshes>)>,
) {
    if env.enabled && fse.active {
        let player = input.player.or_else(|| cars.iter().filter(|c| !c.1.spec.frontend).map(|c| c.1.request).min());
        let car = cars.iter().find(|c| Some(c.1.request) == player);
        if env.bend_pending {
            if let Some(m) = car.and_then(|c| c.3) {
                let n = bend_meshes(&mut meshes, m);
                log::debug!("dynamic envmap: bent the normals of {n} player car meshes");
            }
            env.bend_pending = false;
        }
        if !env.drawn {
            env.drawn = true;
            // camera mode 3 or "hi-res": 256², else 128²
            let hi = env.tweaks.hi_res;
            if hi != env.hi_res {
                env.size_log2 = if hi { de::SIZE_LOG2_HIGH } else { de::SIZE_LOG2_LOW };
                let s = 1u32 << env.size_log2;
                let image = images.add(envmap_image(s));
                if let Some(t) = env.target {
                    fse.set_target(t, image.clone(), UVec2::splat(s));
                }
                env.image = Some(image);
                env.hi_res = hi;
            }
            if let (Some(target), Some(source), Some((g, proj))) = (env.target, env.source, camera.iter().next()) {
                let tan = match proj {
                    Projection::Perspective(p) => (p.fov * 0.5).tan(),
                    _ => 1.0,
                };
                let (car_pos, ground) = car.map(|c| (c.0.translation(), c.2.map(|s| s.normal).unwrap_or(Vec3::ZERO))).unwrap_or_default();
                let lights = points.as_ref().map(|p| p.points.as_slice()).unwrap_or(&[]);
                let view = EnvmapView {
                    cam_pos: g.translation(),
                    cam_right: g.right().as_vec3(),
                    cam_up: g.up().as_vec3(),
                    cam_fwd: g.forward().as_vec3(),
                    tan_half_fov: tan,
                    car: car_pos,
                    ground,
                    lights,
                };
                let t = env.tweaks.clone();
                render_envmap(&mut fse, &t, target, source, env.lightglow, env.fresnel, &view);
            }
        }
    }
    // 0x00159C10: the "display" tweak copies the envmap into the top-left 0.3 of the screen (Default shader, Copy)
    if env.drawn {
        env.drawn = false;
        if let (true, Some(t)) = (env.tweaks.display, env.target) {
            fse.draw(FseImage::BackBuffer, t, FseShader::DEFAULT, prog::COPY, None, Some([0.0, 0.0, 0.3, 0.3]), 0.0, WHITE);
        }
    }
}

pub struct DynamicEnvmapPlugin;

impl Plugin for DynamicEnvmapPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DynamicEnvmapInit>()
            .add_message::<DynamicEnvmapShutdown>()
            .init_resource::<DynamicEnvmap>()
            .init_resource::<DynamicEnvmapInput>()
            .add_plugins(ExtractComponentPlugin::<EnvmapCaptureView>::default())
            .add_systems(Update, (envmap_messages, layer_cars, mark_car_pass).chain())
            .add_systems(PostUpdate, no_transmission_copies)
            .add_systems(PostUpdate, (update_envmap_source, draw_dynamic_envmap).chain().in_set(FseSet::Build));
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(
                Core3d,
                capture_envmap_source.after(main_opaque_pass_3d).before(main_transmissive_pass_3d).in_set(Core3dSystems::MainPass),
            );
        }
    }
}
