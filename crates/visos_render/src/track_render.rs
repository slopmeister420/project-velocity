//! track rendering for game flow (not an original module; the loaders it drives are cited per function):
//! `LoadTrackRender` builds a track's geometry, textures, lights, fog and sky under one `TrackRoot` entity,
//! `UnloadTrackRender` despawns it. Any 3D camera tagged `TrackCamera` becomes the world camera (sky passes
//! render under it and follow its rotation).

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;

use crate::compartment_manager::{sector_priorities, CompartmentIndex, CompartmentStreamer, StreamView};
use crate::compartment_sys::{Compartment, INSTANCE_VISUAL};
use crate::model_builder::{Builder, SpawnOpts, TexSet};
use crate::renderer::d3dx_tga;
use crate::renderer::renderer_types::Model;
use crate::renderer::shader_manager::XboxStageMaterial;
use crate::renderer::texture_manager::decode_any_vtf;
use crate::sky_render::SkyCamera;

/// load a track (directory name under `\Tracks`, e.g. "LACoast", "ExecTest"); replaces any loaded track.
#[derive(Message, Clone, Debug)]
pub struct LoadTrackRender {
    pub track: String,
    /// fixed-function fog (FogRender); on in game
    pub fog: bool,
}

impl LoadTrackRender {
    pub fn new(track: impl Into<String>) -> Self {
        Self { track: track.into(), fog: true }
    }
}

/// despawn the loaded track (geometry, sky passes).
#[derive(Message, Clone, Debug, Default)]
pub struct UnloadTrackRender;

/// root of everything a loaded track spawned; despawning it unloads the track.
#[derive(Component, Debug)]
pub struct TrackRoot {
    pub track: String,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub triangles: usize,
}

/// the loaded track's directional lights (cars are lit with them)
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct TrackLights {
    pub lights: crate::vs_render::VsLights,
    /// fog colour + density applied to world materials
    pub fog: Vec4,
}

/// the ambient of the global light set `unk_718218` while the world draws (CSkyRender::Render 0x0013BF30); compartment
/// instances (visual and physics objects) are lit with `TrackLights.lights.instance_light_set(cpt version, sky_ambient)`
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct TrackInstanceLights {
    pub sky_ambient: Vec4,
}

impl TrackInstanceLights {
    /// CSkyRender.cpp 0x0013BF30 — the last ambient write before the world: unk_730DE8 (1,1,1) x "Buildings
    /// Brightness" (`flt_68700C`) when the sky is loaded (+0x94), else the static zero
    pub fn sky_ambient(setup: Option<&crate::sky_render::SkySetup>) -> Vec4 {
        match setup {
            Some(s) => (Vec3::ONE * s.buildings_brightness).extend(1.0).clamp(Vec4::ZERO, Vec4::ONE),
            None => Vec4::ZERO,
        }
    }
}

/// the loaded track's point lights (`PtLights.txt`), used for object lighting (`VsLights::object_lights`)
#[derive(Resource, Clone, Debug, Default)]
pub struct TrackPointLights {
    pub points: Vec<crate::vs_render::PointLight>,
}

/// a track sector (`Sector_<index>_high.cpt`): visible while the compartment streamer holds it
#[derive(Component, Debug)]
pub struct TrackSector {
    pub index: usize,
}

/// sector streaming state of the loaded track (CompartmentManager.cpp)
#[derive(Resource, Debug)]
pub struct TrackStreaming {
    pub index: CompartmentIndex,
    pub streamer: CompartmentStreamer,
    pub sectors: Vec<(usize, Entity)>,
    filled: bool,
}

/// the cameras the streamer works around (one per player, as 0x000897C0's camera list), written by game flow
/// each update; when empty the `TrackCamera` is used (position, direction, frustum, no raceline).
#[derive(Resource, Clone, Debug, Default)]
pub struct TrackStreamViews {
    pub views: Vec<StreamView>,
    /// "Enable Cameras For Review" (`dword_749110`)
    pub review: bool,
}

/// marker for the game's world camera: gets order 1, no colour clear (the sky passes clear), no tonemapping.
#[derive(Component, Default)]
pub struct TrackCamera;

pub struct TrackRenderPlugin;

impl Plugin for TrackRenderPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::renderer::shader_manager::XboxStageMaterialPlugin>() {
            app.add_plugins(crate::renderer::shader_manager::XboxStageMaterialPlugin);
        }
        if !app.is_plugin_added::<crate::view_render::ViewRenderPlugin>() {
            app.add_plugins(crate::view_render::ViewRenderPlugin);
        }
        app.init_resource::<TrackStreamViews>().add_message::<LoadTrackRender>()
            .add_message::<UnloadTrackRender>()
            .add_systems(
                Update,
                (
                    process_track_messages,
                    crate::view_render::update_views,
                    setup_track_camera,
                    crate::view_render::setup_view_cameras,
                    crate::view_render::setup_mirror_camera,
                    crate::sky_render::sync_sky_camera,
                    crate::view_render::sync_view_sky,
                    crate::sky_render::update_sky,
                    crate::sky_render::update_sun_moon,
                    update_track_streaming,
                )
                    .chain(),
            );
    }
}

fn load_vcd(path: &str) -> Option<visos::file::Vcd> {
    match visos::file::Vcd::open(path) {
        Ok(v) => Some(v),
        Err(e) => {
            log::debug!("{path}: {e}");
            None
        }
    }
}

/// `"SKY"` section of the track's Setup.txt: the next quoted, non-comment line names the sky.
pub fn track_sky_name(base: &str) -> Option<String> {
    let text = std::fs::read_to_string(visos::file::resolve(&format!("{base}/Setup.txt"))).ok()?;
    let mut lines = text.lines().map(str::trim).skip_while(|l| *l != "\"SKY\"").skip(1);
    lines.find(|l| l.starts_with('"')).map(|l| l.trim_matches('"').to_string())
}


/// build a track under a new `TrackRoot`: single-arena layout (`ARENAM.VCD` + `ARENAT.VCD`) or compartments
/// (`CPTS.VCD`: every compartment's textures, `_high` sector models, visual instances), lights (`Lights.txt`),
/// fog (`Setup.txt`, FogRender) and the sky named by `Setup.txt`.
pub fn build_track(commands: &mut Commands, b: &mut Builder, track: &str, fog_on: bool) -> Entity {
    let root = commands.spawn((Transform::default(), Visibility::default())).id();
    let base = format!("\\Tracks\\{}", track);
    let mut tex = TexSet::default();
        let setup_fog = std::fs::read_to_string(visos::file::resolve(&format!("{base}/Setup.txt")))
        .map(|t| crate::fog_render::SetupFog::parse(&t))
        .unwrap_or_default();
    let fog = crate::fog_render::FogRender::init(&setup_fog);
    commands.insert_resource(fog.clone());
    let mut world_opts = SpawnOpts::world();
    world_opts.parent = Some(root);
    if fog_on {
        if let Some((c, d)) = fog.render_states() {
            world_opts.fog = c.extend(d);
        }
    }
    // no Lights.txt (Tokyo, Tokyo2): the scene keeps VsRender's start-up set 0x000583F0
    b.lights = std::fs::read_to_string(visos::file::resolve(&format!("{base}/Lights.txt")))
        .ok()
        .and_then(|t| crate::vs_render::VsLights::parse_lights_txt(&t))
        .unwrap_or_else(crate::vs_render::VsLights::vs_render_defaults);
    let sky = track_sky_name(&base);
    let sky_setup = sky.as_ref().map(|n| crate::sky_render::SkySetup::load(n));
    let sky_ambient = TrackInstanceLights::sky_ambient(sky_setup.as_ref());
    let arena = load_vcd(&format!("{base}\\ARENAM.VCD"));
    if let Some(arena) = arena {
        if let Some(t) = load_vcd(&format!("{base}\\ARENAT.VCD")) {
            for e in t.entries() {
                let d = t.data(&e).unwrap_or_default();
                let dec = if e.name.to_ascii_lowercase().ends_with(".tga") { d3dx_tga::decode_tga(d) } else { decode_any_vtf(d).ok() };
                if let Some(dec) = dec {
                    tex.add(e.name, dec);
                }
            }
        }
        for e in arena.entries() {
            if !e.name.to_ascii_lowercase().ends_with(".xmf") {
                continue;
            }
            match Model::parse(arena.data(&e).unwrap_or_default()) {
                Ok(m) => {
                    b.add_model(commands, &mut tex, &m, Mat4::IDENTITY, &world_opts);
                }
                Err(err) => log::warn!("{}: {err}", e.name),
            }
        }
    } else if let Some(cpts) = load_vcd(&format!("{base}\\CPTS.VCD")) {
        let mut parsed = Vec::new();
        for e in cpts.entries() {
            if e.name.to_ascii_lowercase().ends_with(".cpt") {
                parsed.push((e.name.to_ascii_lowercase(), cpts.data(&e).unwrap_or_default()));
            }
        }
        // textures first (common.cpt and every other compartment that carries some)
        for (_, d) in &parsed {
            let Ok(c) = Compartment::parse(d) else { continue };
            for r in c.textures() {
                if let (Some(n), Ok(t)) = (&r.name, decode_any_vtf(r.data)) {
                    tex.add(n, t);
                }
            }
        }
        // sector streaming (CompartmentManager.cpp): each `Sector_<n>_high.cpt` and its instances go under a
        // `TrackSector` entity that is shown while the streamer holds the sector; without a usable index every
        // sector stays visible
        let index = cpts.get("index.idx").and_then(CompartmentIndex::parse);
        let mut sectors = Vec::new();
        for (name, d) in &parsed {
            let Ok(c) = Compartment::parse(d) else { continue };
            let sector = name.strip_prefix("sector_").and_then(|r| r.strip_suffix("_high.cpt")).and_then(|n| n.parse::<usize>().ok());
            if name.starts_with("sector_") && sector.is_none() {
                continue; // `_low` sectors are never loaded
            }
            let mut opts = world_opts.clone();
            if let (Some(n), Some(_)) = (sector, &index) {
                let e = commands.spawn((Transform::default(), Visibility::Hidden, TrackSector { index: n }, ChildOf(root))).id();
                sectors.push((n, e));
                opts.parent = Some(e);
            }
            if sector.is_some() {
                for r in c.models() {
                    if let Ok(m) = Model::parse(r.data) {
                        b.add_model(commands, &mut tex, &m, Mat4::IDENTITY, &opts);
                    }
                }
            }
            // CompartmentManager.cpp 0x00087D00: instances draw with a light set (flag-1 stages get d3d lighting)
            let mut inst_opts = opts.clone();
            inst_opts.light_set = Some(b.lights.instance_light_set(c.header.version, sky_ambient));
            // F3CompartmentManager::AddInstance 0x0008AC20: the beam light override on every instance model
            inst_opts.beam_lights = true;
            for l in c.instance_lists().unwrap_or_default() {
                if l.kind != INSTANCE_VISUAL {
                    continue;
                }
                for i in &l.instances {
                    let Some(r) = c.resource(i.visual) else { continue };
                    let Ok(m) = Model::parse(r.data) else { continue };
                    let q = Quat::from_xyzw(i.rotation[0], i.rotation[1], i.rotation[2], i.rotation[3]);
                    let xf = Mat4::from_rotation_translation(q, Vec3::from_array(i.position));
                    b.add_model(commands, &mut tex, &m, xf, &inst_opts);
                }
            }
        }
        if let Some(index) = index {
            // pool taken by the resident packs first (0x000874A0 / 0x000875B0)
            let resident: u32 = cpts
                .entries()
                .filter(|e| matches!(e.name.to_ascii_lowercase().as_str(), "common.cpt" | "instance.cpt" | "uinstance.cpt"))
                .map(|e| e.size)
                .sum();
            let streamer = CompartmentStreamer::new(&index, resident);
            log::info!("track {track}: {} sectors, {} slots", index.sector_count(), streamer.slots.len());
            commands.insert_resource(TrackStreaming { index, streamer, sectors, filled: false });
        }
    } else {
        log::error!("no track data under {base}");
    }
    if let (Some(name), Some(setup)) = (&sky, &sky_setup) {
        crate::sky_render::spawn_sky(commands, b, name, setup, root);
    }
    log::info!("track {track}: {} textures, {} triangles, bounds {} .. {}", tex.by_name.len(), b.tris, b.bounds_min, b.bounds_max);
    commands.insert_resource(TrackLights { lights: b.lights, fog: world_opts.fog });
    commands.insert_resource(TrackInstanceLights { sky_ambient });
    let points = std::fs::read_to_string(visos::file::resolve(&format!("{base}/PtLights.txt")))
        .map(|t| crate::vs_render::parse_ptlights_txt(&t))
        .unwrap_or_default();
    commands.insert_resource(TrackPointLights { points });
    commands.entity(root).insert(TrackRoot {
        track: track.to_string(),
        bounds_min: b.bounds_min,
        bounds_max: b.bounds_max,
        triangles: b.tris,
    });
    root
}

fn process_track_messages(
    mut commands: Commands,
    mut load: MessageReader<LoadTrackRender>,
    mut unload: MessageReader<UnloadTrackRender>,
    roots: Query<Entity, With<TrackRoot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    mut images: ResMut<Assets<Image>>,
    rain: Option<ResMut<crate::rain_surface::RainSurface>>,
    programs: Option<ResMut<Assets<crate::renderer::shader_program_manager::ProgramMaterial>>>,
) {
    let unload_now = unload.read().count() > 0;
    let req = load.read().last().cloned();
    if unload_now || req.is_some() {
        for r in &roots {
            commands.entity(r).despawn();
        }
        commands.remove_resource::<crate::sky_render::SkyState>();
        commands.remove_resource::<TrackStreaming>();
        commands.remove_resource::<crate::projected_light_beam_pc_xbox::BeamPrograms>();
    }
    if let Some(req) = req {
        let road_detail = rain.map(|mut r| r.material(&mut materials, &mut images));
        let mut programs = programs;
        let mut b = Builder::new(&mut meshes, &mut materials, &mut images);
        b.road_detail = road_detail;
        b.beam = programs.as_deref_mut().map(crate::model_builder::BeamCtx::new);
        build_track(&mut commands, &mut b, &req.track, req.fog);
        if let Some(c) = b.beam.take() {
            log::info!("track {}: {} beam light programs", req.track, c.made.len());
            commands.insert_resource(crate::projected_light_beam_pc_xbox::BeamPrograms(c.made));
        }
    }
}

/// world camera order: low enough that the sky passes (4 below) and the full-screen effect passes (above, up to 9)
/// fit before the HUD cameras (10+)
pub const WORLD_CAMERA_ORDER: isize = -40;

/// world camera setup: drawn after the sky passes (`WORLD_CAMERA_ORDER`) without clearing colour, no tonemapping;
/// the sky cameras take its render target.
fn setup_track_camera(
    mut commands: Commands,
    cams: Query<(Entity, &Camera, Option<&bevy::camera::RenderTarget>), With<TrackCamera>>,
    sky: Query<(Entity, Option<&bevy::camera::RenderTarget>), With<SkyCamera>>,
    clear_colour: Res<ClearColor>,
    split: Res<crate::view_render::SplitScreen>,
) {
    let Some((e, cam, target)) = cams.iter().next() else { return };
    // a track without a sky (ExecTest) has no pass that clears, so the world camera clears itself — alpha 0, as the
    // original clears the stencil (frame-buffer alpha is the bloom glow mask); split, player 1's view clears first
    let clear = if sky.is_empty() && !split.on() { ClearColorConfig::Custom(clear_colour.0.with_alpha(0.0)) } else { ClearColorConfig::None };
    if cam.order != WORLD_CAMERA_ORDER || std::mem::discriminant(&cam.clear_color) != std::mem::discriminant(&clear) {
        commands.entity(e).insert((Camera { order: WORLD_CAMERA_ORDER, clear_color: clear, ..cam.clone() }, Tonemapping::None));
    }
    for (s, st) in &sky {
        let want = target.cloned();
        let differs = match (&want, st) {
            (Some(a), Some(b)) => format!("{a:?}") != format!("{b:?}"),
            (None, None) => false,
            _ => true,
        };
        if differs {
            match want {
                Some(t) => commands.entity(s).insert(t),
                None => commands.entity(s).remove::<bevy::camera::RenderTarget>(),
            };
        }
    }
}

/// CompartmentManager.cpp 0x0008A270 per frame: one sector swap; the first pass after a load fills every slot
/// (the loading screen's job in the original)
fn update_track_streaming(
    streaming: Option<ResMut<TrackStreaming>>,
    views: Res<TrackStreamViews>,
    cam: Query<(&GlobalTransform, Option<&bevy::camera::primitives::Frustum>), With<TrackCamera>>,
    mut vis: Query<&mut Visibility, With<TrackSector>>,
) {
    let Some(mut st) = streaming else { return };
    let fallback;
    let list = if views.views.is_empty() {
        let Some((g, f)) = cam.iter().next() else { return };
        fallback = [StreamView {
            position: g.translation(),
            forward: g.forward().as_vec3(),
            planes: f.map(|f| std::array::from_fn(|i| f.half_spaces[i].normal_d())),
            raceline: None,
        }];
        &fallback[..]
    } else {
        &views.views[..]
    };
    let pri = sector_priorities(&st.index, list, views.review);
    let st = &mut *st;
    if !st.filled {
        st.streamer.fill(pri.as_ref());
        st.filled = true;
    } else {
        st.streamer.update(pri.as_ref());
    }
    for (n, e) in &st.sectors {
        let want = if st.streamer.resident >> n & 1 != 0 { Visibility::Inherited } else { Visibility::Hidden };
        if let Ok(mut v) = vis.get_mut(*e) {
            v.set_if_neq(want);
        }
    }
}
