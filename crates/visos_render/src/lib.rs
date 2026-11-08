//! ViSOS renderer: textures, meshes, shaders, track and car rendering (visos/Renderer, src/Render).

use bevy::prelude::*;

pub mod bone_anim;
pub mod bone_anim_blend_tree;
pub mod bone_skeleton_model;
pub mod barrier_render;
pub mod cats_eye_render;
pub mod car_damage_fx;
pub mod car_damage_render;
pub mod car_damage_smoke;
pub mod car_palette;
pub mod car_palette_pc_xbox;
pub mod car_render;
pub mod car_shader_manager_pc_xbox;
pub mod compartment_manager;
pub mod compartment_sys;
pub mod danger_zone_render;
pub mod flare_render;
pub mod effect_draw;
pub mod el_particle;
pub mod fade_render;
pub mod fog_render;
pub mod full_screen_effects_manager_pc_xbox;
pub mod fse_chain;
pub mod fury_fx;
pub mod fury_instance;
pub mod game_shader_program_manager;
pub mod gfx_ab;
pub mod golden;
pub mod global_mat_input_resolver;
pub mod glow_pc_xbox;
pub mod gfx_tweaks;
pub mod lens_spray_manager;
pub mod licence_plate_render;
pub mod model_builder;
pub mod motion_blur_pc_xbox;
pub mod mtv_effect;
pub mod particle_manager;
pub mod puddle_render;
pub mod projected_light_beam_pc_xbox;
pub mod rain_surface;
pub mod camera_rain;
pub mod race_effects;
pub mod renderer;
pub mod skinned_object_render;
pub mod skid_mark_render;
pub mod specular_chars;
pub mod vertex_anim;
pub mod light_streak;
pub mod skidsmoke;
pub mod sky_render;
pub mod slipstream_render;
pub mod spark_render;
pub mod start_line_effects;
pub mod stateless_particles;
pub mod swapchain_guard;
pub mod vs_render;
pub mod zone_render_pc_xbox;
pub mod track_render;
pub mod track_view;
pub mod track_fx;
pub mod track_view_fx;
pub mod track_view_damage;
pub mod track_view_trail;
pub mod trail_blazer_render;
pub mod tyrespray;
pub mod view_render;

#[cfg(test)]
pub(crate) mod test_util {
    /// every `.vcd` under the game dir.
    pub fn all_vcds() -> Vec<std::path::PathBuf> {
        fn walk(p: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(p).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("vcd")) {
                    out.push(p);
                }
            }
        }
        let mut v = Vec::new();
        walk(visos::file::game_dir(), &mut v);
        v
    }
}

pub struct RenderPlugin;

/// bevy workaround: `camera_system` recomputes a camera's target size only for new cameras, projection changes and
/// image asset events — not when the `RenderTarget` component is swapped. A camera spawned on the (absent) window
/// and moved onto an existing image a frame later (fecapture's foyer camera) kept no viewport and drew nothing:
/// touch the projection of every camera whose target changed or that still has no target size on an image.
pub fn refresh_retargeted_cameras(
    mut cams: Query<(&Camera, Ref<bevy::camera::RenderTarget>, &mut Projection)>,
    images: Res<Assets<Image>>,
) {
    for (cam, target, mut proj) in &mut cams {
        let unsized_image = match &*target {
            bevy::camera::RenderTarget::Image(i) => cam.physical_target_size().is_none() && images.contains(&i.handle),
            _ => false,
        };
        if (target.is_changed() && !target.is_added()) || unsized_image {
            proj.set_changed();
        }
    }
}

/// bevy workaround, second half: the FSE pass / stencil / road cameras are retargeted in PostUpdate after
/// `camera_system`, so for that frame they extract with the old target's size. bevy shares one depth texture per
/// (target, msaa) and one main texture set per (target, usage, hdr, msaa), each sized by whichever camera comes first:
/// a pass camera moved from a 320x240 glow buffer onto the 640x480 back buffer gave the back buffer's cameras a
/// 320x240 depth next to 640x480 colour — a validation error that drops the frame's submission, after which bevy_pbr's
/// gpu cluster readback reads garbage and panics (`next_power_of_two` overflow, cluster/gpu.rs 1169). the first race
/// spawns fresh slots; every later load reuses slots still on the previous race's last targets, hence "second race
/// crashes". runs in `Last` (all retarget commands applied) and resizes any camera whose target size is stale.
pub fn sync_camera_target_sizes(
    mut cams: Query<(&mut Camera, &bevy::camera::RenderTarget, &mut Projection)>,
    images: Res<Assets<Image>>,
    primary: Query<&Window, With<bevy::window::PrimaryWindow>>,
    windows: Query<&Window>,
) {
    use bevy::camera::{RenderTarget, RenderTargetInfo};
    for (mut cam, target, mut proj) in &mut cams {
        if cam.viewport.is_some() {
            continue;
        }
        let info = match target {
            RenderTarget::Image(i) => images.get(&i.handle).map(|img| RenderTargetInfo { physical_size: img.size(), scale_factor: i.scale_factor }),
            RenderTarget::Window(bevy::window::WindowRef::Primary) => {
                primary.iter().next().map(|w| RenderTargetInfo { physical_size: w.physical_size(), scale_factor: w.scale_factor() })
            }
            RenderTarget::Window(bevy::window::WindowRef::Entity(e)) => {
                windows.get(*e).ok().map(|w| RenderTargetInfo { physical_size: w.physical_size(), scale_factor: w.scale_factor() })
            }
            _ => None,
        };
        let Some(info) = info.filter(|i| i.physical_size.x > 0 && i.physical_size.y > 0) else { continue };
        // an unsized camera is not extracted (camera_system sizes it next frame); sizing it here would extract a
        // view whose clusters were never assigned (zero-sized "clustering dummy texture")
        if cam.physical_target_size().is_none_or(|s| s == info.physical_size) {
            continue;
        }
        log::debug!("camera order {} retargeted: size {:?} -> {}", cam.order, cam.physical_target_size(), info.physical_size);
        cam.computed.target_info = Some(info);
        if let Some(size) = cam.logical_viewport_size().filter(|s| s.x != 0.0 && s.y != 0.0) {
            // bypass change detection: this is camera_system's own update, done a frame early
            let p = proj.bypass_change_detection();
            p.update(size.x, size.y);
            cam.computed.clip_from_view = match &cam.sub_camera_view {
                Some(sub) => p.get_clip_from_view_for_sub(sub),
                None => p.get_clip_from_view(),
            };
        }
    }
}

/// both camera target workarounds (`RenderPlugin`, and the track viewer when it builds the plugins itself)
pub fn add_camera_target_fixes(app: &mut App) {
    app.add_systems(PostUpdate, refresh_retargeted_cameras.before(bevy::camera::CameraUpdateSystems));
    app.add_systems(Last, sync_camera_target_sizes);
    app.add_systems(Update, no_cluster_on_new_cameras).add_systems(Startup, cpu_clustering);
}

/// perf: bevy's GPU light clustering runs a compute + raster pass per view (`cluster_on_gpu`, ~23 views a frame in a
/// race) and `prepare_clusters_for_gpu_clustering`, for no lights; CPU clustering of one cluster and no lights is
/// free. `GlobalClusterSettings` is inserted by `PbrPlugin::finish`, so before Startup and before any pipeline.
/// `VELOCITY_PERF_OFF=gpucluster` = keep bevy's GPU clustering
fn cpu_clustering(settings: Option<ResMut<bevy::light::cluster::GlobalClusterSettings>>) {
    if golden::perf_off("gpucluster") {
        return;
    }
    if let Some(mut s) = settings {
        s.gpu_clustering = None;
    }
}

/// perf: no bevy lights exist (the original's light sets are in the shaders), so skip clustering on every 3d view
fn no_cluster_on_new_cameras(mut commands: Commands, cams: Query<Entity, Added<Camera3d>>) {
    if std::env::var_os("VELOCITY_PERF_OFF").is_some_and(|v| v.to_string_lossy().contains("clusteropt")) {
        return;
    }
    for e in &cams {
        // not `None`: bevy 0.19 gpu clustering then creates a zero-size dummy texture (validation error)
        commands.entity(e).insert(bevy::light::cluster::ClusterConfig::Single);
    }
}

/// perf: replace a per-frame rebuilt mesh only when it changed (every `get_mut` re-uploads the mesh: bevy's
/// `allocate_and_free_meshes` frees and reallocates its slabs each frame); `VELOCITY_PERF_OFF=meshopt` = always
/// Returns true when written: the caller then marks the entity's `Mesh3d` changed (bevy's gpu mesh instances keep
/// the vertex offset of the extraction; a re-uploaded mesh moves in its slab, so the entity must be re-extracted)
pub fn set_mesh(meshes: &mut Assets<Mesh>, h: &Handle<Mesh>, mesh: Mesh) -> bool {
    if !mesh_opt() {
        if let Some(mut m) = meshes.get_mut(h) {
            *m = mesh;
        }
        return false;
    }
    if meshes.get(h).is_some_and(|m| *m == mesh) {
        return false;
    }
    if let Some(mut m) = meshes.get_mut(h) {
        *m = mesh;
        return true;
    }
    false
}

/// `set_mesh` skips unchanged meshes (off: `VELOCITY_PERF_OFF=meshopt`)
pub fn mesh_opt() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| !golden::perf_off("meshopt"))
}

#[cfg(test)]
mod retarget_tests {
    use bevy::asset::RenderAssetUsages;
    use bevy::camera::{CameraUpdateSystems, RenderTarget};
    use bevy::render::render_resource::TextureFormat;

    use super::*;

    /// a camera spawned on the primary window (none, headless) then moved onto an image gets a target size only with
    /// the workaround (positive control: without it bevy leaves it unsized — the black fecapture hangout).
    fn retarget_sized(fix: bool) -> Option<UVec2> {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default(), bevy::image::ImagePlugin::default()));
        app.add_message::<bevy::window::WindowResized>()
            .add_message::<bevy::window::WindowCreated>()
            .add_message::<bevy::window::WindowScaleFactorChanged>()
            .init_resource::<bevy::render::texture::ManualTextureViews>()
            .add_systems(PostUpdate, bevy::render::camera::camera_system.in_set(CameraUpdateSystems));
        if fix {
            app.add_systems(PostUpdate, refresh_retargeted_cameras.before(CameraUpdateSystems));
        }
        let mut img = Image::new_target_texture(64, 48, TextureFormat::Rgba8UnormSrgb, None);
        img.asset_usage = RenderAssetUsages::default();
        let h = app.world_mut().resource_mut::<Assets<Image>>().add(img);
        app.update();
        app.update();
        let cam = app.world_mut().spawn((Camera::default(), Projection::default())).id();
        app.update();
        app.world_mut().entity_mut(cam).insert(RenderTarget::Image(h.into()));
        app.update();
        app.update();
        app.world().get::<Camera>(cam).and_then(|c| c.physical_target_size())
    }

    /// a camera retargeted in PostUpdate after camera_system (the FSE pass slots) from a 32x24 buffer onto a 64x48 one:
    /// the size it would extract with that frame
    fn late_retarget_size(fix: bool) -> Option<UVec2> {
        #[derive(Resource)]
        struct To(Option<Handle<Image>>);
        fn retarget(mut to: ResMut<To>, mut commands: Commands, cams: Query<Entity, With<Camera>>) {
            if let Some(h) = to.0.take() {
                for e in &cams {
                    commands.entity(e).insert(RenderTarget::Image(h.clone().into()));
                }
            }
        }
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default(), bevy::image::ImagePlugin::default()));
        app.add_message::<bevy::window::WindowResized>()
            .add_message::<bevy::window::WindowCreated>()
            .add_message::<bevy::window::WindowScaleFactorChanged>()
            .init_resource::<bevy::render::texture::ManualTextureViews>()
            .insert_resource(To(None))
            .add_systems(PostUpdate, bevy::render::camera::camera_system.in_set(CameraUpdateSystems))
            .add_systems(PostUpdate, (refresh_retargeted_cameras.before(CameraUpdateSystems), retarget.after(CameraUpdateSystems)));
        if fix {
            app.add_systems(Last, sync_camera_target_sizes);
        }
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        let mut small = Image::new_target_texture(32, 24, TextureFormat::Rgba8UnormSrgb, None);
        small.asset_usage = RenderAssetUsages::default();
        let small = images.add(small);
        let mut big = Image::new_target_texture(64, 48, TextureFormat::Rgba8UnormSrgb, None);
        big.asset_usage = RenderAssetUsages::default();
        let big = images.add(big);
        let cam = app.world_mut().spawn((Camera::default(), Projection::default(), RenderTarget::Image(small.into()))).id();
        app.update();
        app.update();
        assert_eq!(app.world().get::<Camera>(cam).and_then(|c| c.physical_target_size()), Some(UVec2::new(32, 24)));
        app.world_mut().resource_mut::<To>().0 = Some(big);
        app.update();
        app.world().get::<Camera>(cam).and_then(|c| c.physical_target_size())
    }

    #[test]
    fn late_retarget_extracts_the_new_size() {
        assert_eq!(late_retarget_size(false), Some(UVec2::new(32, 24)), "bevy now sizes late retargets: the sync can go");
        assert_eq!(late_retarget_size(true), Some(UVec2::new(64, 48)));
    }

    #[test]
    fn retargeted_camera_gets_a_viewport() {
        assert_eq!(retarget_sized(false), None, "bevy now handles retargets: the workaround can go");
        assert_eq!(retarget_sized(true), Some(UVec2::new(64, 48)));
    }
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            track_render::TrackRenderPlugin,
            car_render::CarRenderPlugin,
            skinned_object_render::SkinnedObjectRenderPlugin,
            stateless_particles::ParticlePlugin,
            race_effects::RaceEffectsPlugin,
            track_fx::TrackFxPlugin,
            full_screen_effects_manager_pc_xbox::FullScreenEffectsPlugin,
            glow_pc_xbox::GlowPlugin,
            motion_blur_pc_xbox::MotionBlurPlugin,
            danger_zone_render::DepthOfFieldPlugin,
            zone_render_pc_xbox::ZoneFxPlugin,
            danger_zone_render::DangerZonePlugin,
            barrier_render::BarrierRenderPlugin,
            lens_spray_manager::LensSprayPlugin,
            fade_render::FadeRenderPlugin,
        ));
        app.add_plugins((puddle_render::PuddleRenderPlugin, tyrespray::TyreSprayPlugin, trail_blazer_render::TrailBlazerRenderPlugin, rain_surface::RainSurfacePlugin));
        app.add_plugins((cats_eye_render::CatsEyeRenderPlugin, mtv_effect::MtvEffectPlugin));
        app.add_plugins((car_damage_smoke::CarDamageSmokePlugin, camera_rain::CameraRainPlugin, car_shader_manager_pc_xbox::DynamicEnvmapPlugin));
        app.add_plugins((gfx_ab::GfxAbPlugin, projected_light_beam_pc_xbox::ProjectedLightBeamPlugin, fury_fx::FuryFxPlugin));
        app.add_plugins((gfx_tweaks::GraphicsTweaksPlugin, swapchain_guard::SwapchainGuardPlugin, golden::GoldenPlugin));
        add_camera_target_fixes(app);
        // debug: `--track <name>` turns the app into the track viewer
        if let Some(cfg) = track_view::TrackViewConfig::from_args() {
            app.add_plugins(track_view::TrackViewPlugin(cfg));
        }
    }
}
