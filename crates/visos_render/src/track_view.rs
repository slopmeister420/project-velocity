//! debug track viewer (not an original module): loads a track's geometry the way the game's loaders lay it out
//! (single-arena `ARENAM.VCD`/`ARENAT.VCD`, or streamed `CPTS.VCD` compartments), builds bevy meshes and either
//! flies a camera or renders one frame offscreen to a PNG.
//!
//! args: `--track <dir under Tracks>` `[--shot <png>]` `[--cam x,y,z,yaw,pitch]` (game coordinates, degrees).

use std::path::PathBuf;

use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::camera::RenderTarget;

use crate::model_builder::to_bevy;
use crate::car_render::{CarSpec, SpawnCarRender};
use crate::track_render::{LoadTrackRender, TrackCamera, TrackRoot};

#[derive(Resource, Clone, Debug, Default)]
pub struct TrackViewConfig {
    pub track: String,
    pub shot: Option<PathBuf>,
    /// x, y, z in game space, yaw / pitch in degrees (yaw 0 looks down -z)
    pub cam: Option<[f32; 5]>,
    /// `--split x,y,z,yaw,pitch`: forced two-player views (GameManager 0x000B3EC0 case 2): `--cam` = player 0 in the
    /// top half, this = player 1's camera in the bottom half (`view_render`)
    pub split: Option<[f32; 5]>,
    /// `--mirror x,y,z,yaw,pitch`: the one-player rear-view mirror view (0x000B3960 kind 1) with this camera
    pub mirror: Option<[f32; 5]>,
    pub width: u32,
    pub height: u32,
    pub no_fog: bool,
    /// `--car LA/350z[,LA/rx8...]`: spawn those cars in a row in front of the camera
    pub car: Option<String>,
    /// `--colours 3,40,...`: COLOURS.BIN record per car (default: body colours spread over the file)
    pub colours: Vec<usize>,
    /// `--neon 30,31`: COLOURS.BIN record per car fitted as its neon (the setup's colour slot 1; +0x38 = the neon)
    pub neon: Vec<usize>,
    /// `--chars fg:lon_flaggirl_a:Start,ch:dj:dj_idle1`: skinned models (pack fg / ch, model, anim) in a row
    pub chars: Vec<(crate::skinned_object_render::CharPack, String, Option<String>)>,
    /// `--foyer hangout` (instead of `--track`): the foyer model with its Crowd.txt / Dance.txt characters
    pub foyer: Option<String>,
    /// `--particles [Circuit\Race01\]`: load the area's particle definitions and placed emitters
    pub particles: Option<String>,
    /// `--effects skid,sparks,slip,start --fxat x,y,z`: race effects demo, two cars circling the point (y = road)
    pub effects: Option<String>,
    pub fx_at: Option<Vec3>,
    /// `--fxfollow k`: the camera stays 4 m off car k, looking at it
    pub fx_follow: Option<usize>,
    /// `--damage`: scripted CarDamage hits on car 0 (`track_view_damage`)
    pub damage: bool,
    /// `--decals n`: every car gets usable DECALS.BIN decals in every slot (car k starts at n + k)
    pub decals: Option<u32>,
    /// `--plate TEXT[,style]`: car 0's licence plate (text, style 0 LA / 1 London / 2 Tokyo)
    pub plate: Option<String>,
    /// `--caryaw deg`: the viewer cars' turn relative to the camera (default 34.4 = 0.6 rad, front three-quarter)
    pub car_yaw: Option<f32>,
    /// `--barriers Circuit\Race01 [--barrierhit x,y,z]`: the race's spline barriers (Barier01.br / NoEntr01.br),
    /// optionally hit by the player at a point once placed
    pub barriers: Option<String>,
    pub barrier_hit: Option<Vec3>,
    /// `--catseyes Circuit\Race02`: the race's `CatsEyes.ced` (CatsEyeRender, lit by the `--car` car)
    pub cats_eyes: Option<String>,
    /// `--lens n`: n rain drops and n/4 splash drops on the lens (held, no update)
    pub lens: Option<u32>,
    /// `--fade a`: the screen fade held at alpha a
    pub fade: Option<f32>,
    /// `--puddles x,y,z,yawdeg[,len,halfwidth]`: wet-London puddles generated (0x00131AC0) along a straight stand-in
    /// race line from the point (default 300 m, edges ±6 m)
    pub puddles: Option<Vec<f32>>,
    /// `--addpuddle x,y,z[,distance]`: one friction-spot puddle as physics adds it at runtime (0x001E1A20 →
    /// `PuddleAdd`, kind 2, 30 x 30, radius 15); without `--puddles` RainSurface and PuddleRender are inited dry
    /// first (TyreSprayInit / InGameInit)
    pub add_puddle: Option<Vec<f32>>,
    /// `--zonetrail <race dir>[,intensity[,section]]`: the race's trailblazer zone trails, the section's one on
    pub zone_trail: Option<String>,
    /// `--lightref`: load the track's physics collision (as PhysicsManager::Init) so TrackFX can drop its lights
    /// onto the road for the reflections (0x00149020)
    pub light_ref: bool,
    /// `--noenvmap`: no dynamic envmap (on by default, as in a single-player race)
    pub no_envmap: bool,
    /// `--reload n`: 60 frames after placing, tear the scene down like the game's race shutdown (camera despawned,
    /// track unloaded, cars removed) and load it again, n times (the second race load check); the shot is of the last
    pub reload: u32,
    /// `--frontend`: build the `--car` cars as front-end cars (`_high` models, showroom shading, front-end shadow)
    pub frontend: bool,
    /// `--fecam`: the camera is the front end's (`rain_surface::FrontEndCamera`, no `TrackCamera`): front-end
    /// flares and, with `--foyer`, the foyer's floor reflection (its Lights.txt "REFLECTION" block)
    pub fe_cam: bool,
    /// `--mtv`: keep the front end's MTV glitch overlay running (`mtv_effect`, needs `--fecam`)
    pub mtv: bool,
}

impl TrackViewConfig {
    /// parse `--track`, `--shot`, `--cam`, `--size WxH` from the process args.
    pub fn from_args() -> Option<Self> {
        let a: Vec<String> = std::env::args().collect();
        let get = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
        let foyer = get("--foyer");
        let track = get("--track").or_else(|| foyer.clone())?;
        let five = |k: &str| {
            get(k).and_then(|s| {
                let v: Vec<f32> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                (v.len() == 5).then(|| [v[0], v[1], v[2], v[3], v[4]])
            })
        };
        let cam = five("--cam");
        let split = five("--split");
        let mirror = five("--mirror");
        let (width, height) = get("--size")
            .and_then(|s| s.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?))))
            .unwrap_or((1280, 720));
        Some(Self { track, shot: get("--shot").map(PathBuf::from), cam, split, mirror, width, height, no_fog: a.iter().any(|x| x == "--nofog"), car: get("--car"),
            foyer,
            particles: a.iter().position(|x| x == "--particles").map(|i| a.get(i + 1).filter(|v| !v.starts_with("--")).cloned().unwrap_or_default()),
            effects: get("--effects"),
            damage: a.iter().any(|x| x == "--damage"),
            decals: get("--decals").and_then(|v| v.parse().ok()),
            plate: get("--plate"),
            car_yaw: get("--caryaw").and_then(|v| v.parse::<f32>().ok()).map(f32::to_radians),
            fx_follow: get("--fxfollow").and_then(|v| v.parse().ok()),
            barriers: get("--barriers"),
            cats_eyes: get("--catseyes"),
            lens: get("--lens").and_then(|v| v.parse().ok()),
            fade: get("--fade").and_then(|v| v.parse().ok()),
            zone_trail: get("--zonetrail"),
            light_ref: a.iter().any(|x| x == "--lightref"),
            no_envmap: a.iter().any(|x| x == "--noenvmap"),
            reload: get("--reload").and_then(|v| v.parse().ok()).unwrap_or(0),
            frontend: a.iter().any(|x| x == "--frontend"),
            fe_cam: a.iter().any(|x| x == "--fecam"),
            mtv: a.iter().any(|x| x == "--mtv"),
            puddles: get("--puddles").map(|s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect::<Vec<f32>>()).filter(|v| v.len() >= 4),
            add_puddle: get("--addpuddle").map(|s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect::<Vec<f32>>()).filter(|v| v.len() >= 3),
            barrier_hit: get("--barrierhit").and_then(|s| {
                let v: Vec<f32> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                (v.len() == 3).then(|| Vec3::new(v[0], v[1], v[2]))
            }),
            fx_at: get("--fxat").and_then(|s| {
                let v: Vec<f32> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                (v.len() == 3).then(|| Vec3::new(v[0], v[1], v[2]))
            }),
            chars: get("--chars")
                .map(|s| {
                    s.split(',')
                        .filter_map(|c| {
                            let mut p = c.split(':');
                            let pack = match p.next()? {
                                "fg" => crate::skinned_object_render::CharPack::FlagGirl,
                                _ => crate::skinned_object_render::CharPack::Chars,
                            };
                            Some((pack, p.next()?.to_string(), p.next().map(str::to_string)))
                        })
                        .collect()
                })
                .unwrap_or_default(),
            colours: get("--colours").map(|s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default(),
            neon: get("--neon").map(|s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default(),
        })
    }
}

pub struct TrackViewPlugin(pub TrackViewConfig);

impl Plugin for TrackViewPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::track_render::TrackRenderPlugin>() {
            app.add_plugins(crate::golden::GoldenPlugin);
            app.add_plugins((
                crate::track_render::TrackRenderPlugin,
                crate::car_render::CarRenderPlugin,
                crate::skinned_object_render::SkinnedObjectRenderPlugin,
                crate::stateless_particles::ParticlePlugin,
                crate::race_effects::RaceEffectsPlugin,
                crate::track_fx::TrackFxPlugin,
                crate::full_screen_effects_manager_pc_xbox::FullScreenEffectsPlugin,
                crate::glow_pc_xbox::GlowPlugin,
                crate::motion_blur_pc_xbox::MotionBlurPlugin,
                crate::danger_zone_render::DepthOfFieldPlugin,
                crate::zone_render_pc_xbox::ZoneFxPlugin,
                crate::danger_zone_render::DangerZonePlugin,
                crate::barrier_render::BarrierRenderPlugin,
                crate::lens_spray_manager::LensSprayPlugin,
                crate::fade_render::FadeRenderPlugin,
            ));
            app.add_plugins((crate::cats_eye_render::CatsEyeRenderPlugin, crate::mtv_effect::MtvEffectPlugin));
            app.add_plugins((crate::puddle_render::PuddleRenderPlugin, crate::tyrespray::TyreSprayPlugin, crate::trail_blazer_render::TrailBlazerRenderPlugin, crate::rain_surface::RainSurfacePlugin));
            app.add_plugins((crate::camera_rain::CameraRainPlugin, crate::car_shader_manager_pc_xbox::DynamicEnvmapPlugin));
            app.add_plugins(crate::projected_light_beam_pc_xbox::ProjectedLightBeamPlugin);
            crate::add_camera_target_fixes(app);
        }
        // the in-game car renderer init (CarPalette 0x0010D7E0 -> DangerZoneRender 0x001214A0): single player
        if !self.0.no_envmap && self.0.foyer.is_none() {
            // `--split`: two players (`byte_6F7157`), which turns the envmap off
            let two_players = self.0.split.is_some();
            app.add_systems(Startup, move |mut w: MessageWriter<crate::car_shader_manager_pc_xbox::DynamicEnvmapInit>| {
                w.write(crate::car_shader_manager_pc_xbox::DynamicEnvmapInit { front_end: false, two_players });
            });
        }
        if self.0.light_ref {
            let track = self.0.track.clone();
            let world = velocity_physics::c_config::load_global_config_with_fallback(0).and_then(|gc| {
                let t = velocity_physics::vehicle::vehicle_entity::cx::TrackWorld::load(&track).ok()?;
                Some(velocity_physics::vehicle::vehicle_entity::sim::PhysicsWorld::new(gc, Some(t)))
            });
            if world.is_none() {
                log::error!("lightref: no physics collision for {track}");
            }
            app.insert_resource(velocity_physics::plugin::PhysicsWorldRes(std::sync::Arc::new(std::sync::Mutex::new(world))));
        }
        if self.0.mtv {
            app.add_systems(Update, |mut m: ResMut<crate::mtv_effect::MtvEffect>| {
                if !m.enabled {
                    m.start(1000.0, false);
                }
            });
        }
        if cfg_has_trail(&self.0) {
            app.init_resource::<crate::track_view_trail::TrailDemo>().add_systems(Update, crate::track_view_trail::trail_demo);
        }
        let cfg = self.0.clone();
        if let Some(p) = cfg.puddles.clone() {
            let track = cfg.track.clone();
            app.add_systems(Startup, move |mut r: ResMut<crate::puddle_render::PuddleRender>, mut rs: ResMut<crate::rain_surface::RainSurface>| {
                // a wet track: RainSurface's roaddetail layer and road reflection (0x00136C50)
                rs.init(true, &track);
                let mut line = crate::puddle_render::StraightLine {
                    start: Vec3::new(p[0], p[1], p[2]),
                    dir: Quat::from_rotation_y(p[3].to_radians()) * Vec3::NEG_Z,
                    length: p.get(4).copied().unwrap_or(300.0),
                    half_width: p.get(5).copied().unwrap_or(6.0),
                };
                r.init(true, Some(&mut line), &mut visos::handy::crt_rand::CrtRand::default());
                log::info!("puddles: {} (first {:?})", r.puddles.len(), r.puddles.first().map(|p| (p.pos, p.size)));
            });
        }
        if let Some(p) = cfg.add_puddle.clone() {
            let (track, dry) = (cfg.track.clone(), cfg.puddles.is_none());
            app.add_systems(Startup, move |mut w: MessageWriter<crate::puddle_render::PuddleAdd>, mut rs: ResMut<crate::rain_surface::RainSurface>, mut r: ResMut<crate::puddle_render::PuddleRender>| {
                // InGameInit always runs both (0x00131690 with no generated puddles off wet London)
                if dry {
                    rs.init(false, &track);
                    r.init(false, None, &mut visos::handy::crt_rand::CrtRand::default());
                }
                let phys = velocity_physics::physics_objects::PhysicsPuddle {
                    position: [p[0], p[1], p[2]],
                    radius: 15.0,
                    direction: [1.0, 0.0],
                    race_distance: p.get(3).copied().unwrap_or(0.0),
                    kind: 2,
                    size: [30.0, 30.0],
                    life: 480,
                    flags: 0xCCCC_CCCD,
                };
                w.write(crate::puddle_render::PuddleAdd((&phys).into()));
            });
        }
        if cfg.lens.is_some() || cfg.fade.is_some() {
            app.add_systems(Startup, viewer_lens_fade);
        }
        app.insert_resource(cfg.clone()).init_resource::<ViewerState>().add_systems(Startup, setup_viewer).add_systems(Update, (viewer_reload, place_viewer).chain());
        if cfg.shot.is_some() {
            app.add_systems(Update, shot_countdown);
        } else {
            app.add_systems(Update, fly_camera);
        }
        if cfg.effects.is_some() {
            // effects_demo runs race_effects_update's body once per 24 Hz update itself
            app.add_systems(Update, (crate::track_view_fx::effects_demo, crate::track_view_fx::wheel_spin_demo, crate::track_view_fx::rain_demo));
        }
        if cfg.barriers.is_some() {
            app.add_systems(Update, viewer_barriers);
        }
        if let Some(race) = cfg.cats_eyes.clone() {
            let dir = format!("\\Tracks\\{}\\{}\\", cfg.track, race.trim_matches('\\'));
            app.add_systems(Startup, move |mut w: MessageWriter<crate::cats_eye_render::LoadCatsEyes>| {
                w.write(crate::cats_eye_render::LoadCatsEyes { race_dir: dir.clone() });
            });
        }
        if cfg.damage {
            app.init_resource::<crate::track_view_damage::DamageDemo>().add_systems(Update, crate::track_view_damage::damage_demo);
        }
    }
}

fn cfg_has_trail(c: &TrackViewConfig) -> bool {
    c.zone_trail.is_some()
}

/// `--lens n` / `--fade a`: drops placed once (never updated, so they stay), fade alpha held
fn viewer_lens_fade(cfg: Res<TrackViewConfig>, mut lens: ResMut<crate::lens_spray_manager::LensSpray>, mut fade: ResMut<crate::fade_render::FadeRender>) {
    if let Some(n) = cfg.lens {
        lens.init();
        for _ in 0..n {
            lens.spawn(false);
        }
        for _ in 0..n / 4 {
            lens.spawn(true);
        }
    }
    if let Some(a) = cfg.fade {
        fade.init();
        fade.set(a);
    }
}

/// the viewer's state: camera placed (cars / characters spawned), frames since, the offscreen target
#[derive(Resource, Default)]
struct ViewerState {
    placed: bool,
    frames: u32,
    target: Option<Handle<Image>>,
    yaw: f32,
    pitch: f32,
    /// `--reload`: loads torn down so far, frames since the teardown (`Some` until the reload)
    reloads: u32,
    unloaded: Option<u32>,
}

#[derive(Component)]
struct ViewerCamera;

/// the `--split` / `--mirror` view cameras (despawned with the viewer camera)
#[derive(Component)]
struct ViewerExtraCamera;

/// the viewer camera; a headless shot renders into one offscreen image kept over reloads (the game's window)
fn spawn_viewer_camera(commands: &mut Commands, cfg: &TrackViewConfig, state: &mut ViewerState, images: &mut Assets<Image>) {
    let projection = Projection::Perspective(PerspectiveProjection {
        fov: crate::sky_render::BACKGROUND_SPAN,
        near: 0.5,
        far: 20000.0,
        ..default()
    });
    let mut cam = commands.spawn((Camera3d::default(), projection.clone(), Transform::default(), ViewerCamera));
    if cfg.fe_cam {
        cam.insert(crate::rain_surface::FrontEndCamera);
    } else {
        cam.insert(TrackCamera);
    }
    let target = cfg.shot.is_some().then(|| {
        state
            .target
            .get_or_insert_with(|| {
                let mut target = Image::new_target_texture(cfg.width, cfg.height, TextureFormat::Rgba8UnormSrgb, None);
                target.asset_usage = RenderAssetUsages::default();
                images.add(target)
            })
            .clone()
    });
    if let Some(t) = &target {
        cam.insert(RenderTarget::Image(t.clone().into()));
    }
    // forced split screen: GameManager 0x000B3960 kinds 2 (top, player 0) and 3 (bottom, player 1)
    if let Some([x, y, z, yaw, pitch]) = cfg.split {
        cam.insert(crate::view_render::TrackView { player: 0, rect: Vec4::new(0.0, 0.0, 1.0, 0.5) });
        let t = Transform::from_translation(to_bevy([x, y, z]))
            .with_rotation(Quat::from_rotation_y(yaw.to_radians()) * Quat::from_rotation_x(pitch.to_radians()));
        // spawned as the game does (a `TrackCamera` per player); view_render keeps player 0's as the TrackCamera
        let mut second = commands.spawn((
            Camera3d::default(),
            projection,
            t,
            TrackCamera,
            crate::view_render::TrackView { player: 1, rect: Vec4::new(0.0, 0.5, 1.0, 0.5) },
            ViewerExtraCamera,
        ));
        if let Some(t) = target.clone() {
            second.insert(RenderTarget::Image(t.into()));
        }
    }
    // the rear-view mirror (kind 1: x W/3, y 0, w W/3, h H/6 of 640x480), as core places it from the bridge's [1]
    if let Some([x, y, z, yaw, pitch]) = cfg.mirror.filter(|_| cfg.split.is_none()) {
        let t = Transform::from_translation(to_bevy([x, y, z]))
            .with_rotation(Quat::from_rotation_y(yaw.to_radians()) * Quat::from_rotation_x(pitch.to_radians()));
        let rect = Vec4::new(213.0 / 640.0, 0.0, 213.0 / 640.0, 80.0 / 480.0);
        let mut m = commands.spawn((
            Camera3d::default(),
            Camera { is_active: false, ..default() },
            Projection::Perspective(PerspectiveProjection { fov: crate::sky_render::BACKGROUND_SPAN, near: 0.5, far: 200.0, ..default() }),
            t,
            crate::view_render::MirrorView { player: 0, rect, on: true },
            ViewerExtraCamera,
        ));
        if let Some(t) = target {
            m.insert(RenderTarget::Image(t.into()));
        }
    }
}

/// `--reload n`: race shutdown (race_cars `despawn_race`: cars, camera, track) 60 frames after placing, the next
/// load 10 frames later
#[allow(clippy::too_many_arguments)]
fn viewer_reload(
    mut commands: Commands,
    cfg: Res<TrackViewConfig>,
    mut state: ResMut<ViewerState>,
    mut images: ResMut<Assets<Image>>,
    cams: Query<Entity, Or<(With<ViewerCamera>, With<ViewerExtraCamera>)>>,
    mut unload: MessageWriter<crate::track_render::UnloadTrackRender>,
    mut despawn: MessageWriter<crate::car_render::DespawnCarRender>,
    mut load: MessageWriter<LoadTrackRender>,
    mut glow: ResMut<crate::glow_pc_xbox::GlowTweaks>,
) {
    if let Some(n) = state.unloaded {
        if n < 10 {
            state.unloaded = Some(n + 1);
            return;
        }
        state.unloaded = None;
        // a different FSE pass layout than the last load ended with (the game's pause menu, fades, zone passes):
        // pass slots that last drew a glow buffer now draw the back buffer — the second-race crash
        glow.passes = if state.reloads % 2 == 1 { 4 } else { 5 };
        log::info!("reload {}: loading {} again (glow passes {})", state.reloads, cfg.track, glow.passes);
        spawn_viewer_camera(&mut commands, &cfg, &mut state, &mut images);
        load.write(LoadTrackRender { track: cfg.track.clone(), fog: !cfg.no_fog });
        return;
    }
    if !state.placed || state.frames != 60 || state.reloads >= cfg.reload || cfg.foyer.is_some() {
        return;
    }
    state.reloads += 1;
    log::info!("reload {}: unloading {}", state.reloads, cfg.track);
    for e in &cams {
        commands.entity(e).despawn();
    }
    let n = cfg.car.as_ref().map_or(0, |l| l.split(',').count().max(cfg.colours.len()));
    for k in 0..n {
        despawn.write(crate::car_render::DespawnCarRender { request: k as u64 + 1 });
    }
    unload.write(crate::track_render::UnloadTrackRender);
    state.placed = false;
    state.frames = 0;
    state.unloaded = Some(0);
}

/// viewer camera (a `TrackCamera`) and the load request; the camera is placed once the track exists.
#[allow(clippy::too_many_arguments)]
fn setup_viewer(
    mut commands: Commands,
    cfg: Res<TrackViewConfig>,
    mut state: ResMut<ViewerState>,
    mut images: ResMut<Assets<Image>>,
    mut load: MessageWriter<LoadTrackRender>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<crate::renderer::shader_manager::XboxStageMaterial>>,
    mut chars: MessageWriter<crate::skinned_object_render::SpawnSkinnedModel>,
    mut particles: MessageWriter<crate::stateless_particles::LoadParticles>,
) {
    if let Some(p) = &cfg.plate {
        let (text, style) = p.split_once(',').map_or((p.as_str(), 0), |(t, s)| (t, s.parse().unwrap_or(0)));
        commands.insert_resource(crate::licence_plate_render::LicencePlate { style, text: text.into(), front_end: false });
    }
    let area = match &cfg.foyer {
        Some(f) => {
            load_foyer(&mut commands, f, &mut meshes, &mut materials, &mut images, &mut chars);
            format!("\\Foyers\\{f}\\")
        }
        None => {
            load.write(LoadTrackRender { track: cfg.track.clone(), fog: !cfg.no_fog });
            format!("\\Tracks\\{}\\", cfg.track)
        }
    };
    if let Some(sub) = &cfg.particles {
        particles.write(crate::stateless_particles::LoadParticles { area, sub: (!sub.is_empty()).then(|| sub.clone()) });
    }
    spawn_viewer_camera(&mut commands, &cfg, &mut state, &mut images);
}

/// a foyer (`--foyer hangout`): `\Foyers\<f>\Model\<f>.xmf` with the pack's textures, lit by its Lights.txt, and
/// the Crowd.txt / Dance.txt characters standing in their STANDANIM
fn load_foyer(
    commands: &mut Commands,
    foyer: &str,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<crate::renderer::shader_manager::XboxStageMaterial>,
    images: &mut Assets<Image>,
    chars: &mut MessageWriter<crate::skinned_object_render::SpawnSkinnedModel>,
) {
    use crate::model_builder::{Builder, SpawnOpts, TexSet};
    let dir = format!("\\Foyers\\{foyer}");
    let lights = std::fs::read_to_string(visos::file::resolve(&format!("{dir}\\Lights.txt")))
        .ok()
        .and_then(|t| crate::vs_render::VsLights::parse_lights_txt(&t))
        .unwrap_or_else(crate::vs_render::VsLights::vs_render_defaults);
    commands.insert_resource(crate::track_render::TrackLights { lights, fog: Vec4::ZERO });
    let mut tex = TexSet::default();
    crate::car_render::add_vcd_textures(&mut tex, &format!("{dir}\\Model\\{}T.VCD", foyer.to_ascii_uppercase()));
    let root = commands.spawn((Transform::default(), Visibility::default())).id();
    let mut b = Builder::new(meshes, materials, images);
    b.lights = lights;
    let opts = SpawnOpts { parent: Some(root), ..SpawnOpts::world() };
    match std::fs::read(visos::file::resolve(&format!("{dir}\\Model\\{foyer}.xmf"))) {
        Ok(d) => match crate::renderer::renderer_types::Model::parse(&d) {
            Ok(m) => {
                b.add_model(commands, &mut tex, &m, Mat4::IDENTITY, &opts);
            }
            Err(e) => log::warn!("foyer {foyer}: {e}"),
        },
        Err(e) => log::warn!("foyer {foyer}: {e}"),
    }
    let (lo, hi) = if b.bounds_min.x <= b.bounds_max.x { (b.bounds_min, b.bounds_max) } else { (Vec3::splat(-10.0), Vec3::splat(10.0)) };
    commands.entity(root).insert(TrackRoot { track: foyer.to_string(), bounds_min: lo, bounds_max: hi, triangles: b.tris });
    // FECK2Element3D::LoadFoyerLights 0x00298CD0: the REFLECTION block (used with `--fecam`)
    let lights = std::fs::read_to_string(visos::file::resolve(&format!("{dir}\\Lights.txt"))).unwrap_or_default();
    commands.insert_resource(crate::rain_surface::FeReflection::from_lights_txt(&lights));
    // GDL_LoadPointLightsTxt 0x0009DE90 on the foyer (none: VsRender 0x00058FF0 frees them); ClosestLights follows
    // with `--fecam` (car_render::fe_closest_lights)
    let points = std::fs::read_to_string(visos::file::resolve(&format!("{dir}\\PtLights.txt")))
        .map(|t| crate::vs_render::parse_ptlights_txt(&t))
        .unwrap_or_default();
    commands.insert_resource(crate::track_render::TrackPointLights { points });
    let mut k = 0;
    for file in ["Crowd.txt", "Dance.txt"] {
        let Ok(text) = std::fs::read_to_string(visos::file::resolve(&format!("{dir}\\{file}"))) else { continue };
        for c in parse_foyer_chars(&text) {
            chars.write(crate::skinned_object_render::SpawnSkinnedModel {
                request: 500 + k,
                pack: crate::skinned_object_render::CharPack::Chars,
                model: c.name,
                anim: c.stand,
                transform: Transform::from_translation(c.pos).with_rotation(Quat::from_rotation_y(c.rotation.to_radians())),
            });
            k += 1;
        }
    }
}

struct FoyerChar {
    name: String,
    pos: Vec3,
    rotation: f32,
    stand: Option<String>,
}

/// Crowd.txt / Dance.txt records: `Name : "x"`, `CharPos : (x, y, z)`, `Rotation : (deg)`, `STANDANIM : "a"`
fn parse_foyer_chars(text: &str) -> Vec<FoyerChar> {
    let mut out: Vec<FoyerChar> = Vec::new();
    let value = |l: &str| l.split_once(['(', '"']).map(|(_, r)| r.trim_end_matches([')', '"', ' ', '\t', '\r']).trim().to_string());
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with("//") {
            continue;
        }
        let key = l.split([':', ' ', '\t']).next().unwrap_or("");
        match key {
            "Name" => {
                if let Some(v) = value(l) {
                    out.push(FoyerChar { name: v.trim_matches('"').to_string(), pos: Vec3::ZERO, rotation: 0.0, stand: None });
                }
            }
            "CharPos" => {
                if let (Some(c), Some(v)) = (out.last_mut(), value(l)) {
                    let f: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                    if f.len() == 3 {
                        c.pos = Vec3::new(f[0], f[1], f[2]);
                    }
                }
            }
            "Rotation" => {
                if let (Some(c), Some(v)) = (out.last_mut(), value(l)) {
                    c.rotation = v.parse().unwrap_or(0.0);
                }
            }
            "STANDANIM" => {
                if let (Some(c), Some(v)) = (out.last_mut(), value(l)) {
                    let v = v.trim_matches('"').to_string();
                    c.stand = (v != "NONE").then_some(v);
                }
            }
            _ => {}
        }
    }
    out
}

/// once the track exists: place the camera (`--cam`, else an overview of the bounds), then the cars and
/// characters in front of it
#[allow(clippy::too_many_arguments)]
fn place_viewer(
    cfg: Res<TrackViewConfig>,
    mut state: ResMut<ViewerState>,
    roots: Query<&TrackRoot>,
    mut cam: Query<&mut Transform, With<ViewerCamera>>,
    mut cars: MessageWriter<SpawnCarRender>,
    mut skinned: MessageWriter<crate::skinned_object_render::SpawnSkinnedModel>,
) {
    if state.placed {
        state.frames += 1;
        return;
    }
    let Some(root) = roots.iter().next() else { return };
    let Ok(mut t) = cam.single_mut() else { return };
    state.placed = true;
    let (pos, yaw, pitch) = match cfg.cam {
        Some([x, y, z, yaw, pitch]) => (to_bevy([x, y, z]), yaw.to_radians(), pitch.to_radians()),
        None => {
            let c = (root.bounds_min + root.bounds_max) * 0.5;
            let ext = (root.bounds_max - root.bounds_min).max_element().max(1.0);
            (c + Vec3::new(0.0, ext * 0.35, ext * 0.5), 0.0, -35f32.to_radians())
        }
    };
    log::info!("{}: {} triangles, bounds {:?}..{:?}", root.track, root.triangles, root.bounds_min, root.bounds_max);
    *t = Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch));
    (state.yaw, state.pitch) = (yaw, pitch);
    if let Some(list) = &cfg.car {
        // cars 6 m ahead on the camera's level, side by side, rotated to show their sides
        let fwd = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
        let right = Vec3::new(fwd.z, 0.0, -fwd.x);
        let names: Vec<&str> = list.split(',').collect();
        let colours = crate::car_palette::load_colours_bin();
        let body: Vec<usize> = (0..colours.len()).filter(|&i| colours[i].kind < 3).collect();
        let n = names.len().max(cfg.colours.len()).max(1);
        for k in 0..n {
            let name = names[k % names.len()];
            let at = pos + fwd * (6.0 + 3.0 * n as f32) - Vec3::Y * 0.7 + right * ((k as f32 - (n - 1) as f32 * 0.5) * 4.5);
            let ci = cfg.colours.get(k).copied().or_else(|| body.get(k * body.len().max(1) / n).copied());
            // `traffic:car01` = a TrafficM.vcd model
            let mut spec = match name.strip_prefix("traffic:") {
                Some(t) => CarSpec::traffic(t, Vec3::new(0.8, 0.1, 0.05)),
                None => CarSpec::new(name, Vec3::new(0.8, 0.1, 0.05)),
            };
            if let Some(c) = ci.and_then(|i| colours.get(i)) {
                spec = spec.with_colour(c);
            }
            if let Some(d) = cfg.decals {
                spec.decals = viewer_decals(d + k as u32);
            }
            spec.licence_plate = k == 0 && cfg.plate.is_some();
            spec.neon = cfg.neon.get(k).and_then(|&i| colours.get(i)).map(|c| c.colour_b);
            spec.frontend = cfg.frontend;
            // debug: `VELOCITY_VIEW_PARTS=b,f,r,e,s,t,sc` part indices (`-` = none), `VELOCITY_VIEW_CARDROP=m` lowers the cars
            if let Ok(p) = std::env::var("VELOCITY_VIEW_PARTS") {
                for (i, v) in p.split(',').enumerate().take(7) {
                    spec.parts[i] = v.trim().parse().ok();
                }
            }
            // debug: `VELOCITY_VIEW_BRAKES=disk001_1,caliper001_1` brake disc / pad models (front-end cars)
            if let Ok(p) = std::env::var("VELOCITY_VIEW_BRAKES") {
                for (i, v) in p.split(',').enumerate().take(2) {
                    spec.brakes[i] = Some(v.trim().to_string());
                }
            }
            let drop = std::env::var("VELOCITY_VIEW_CARDROP").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
            cars.write(SpawnCarRender {
                request: k as u64 + 1,
                spec,
                transform: Transform::from_translation(at - Vec3::Y * drop).with_rotation(Quat::from_rotation_y(yaw + cfg.car_yaw.unwrap_or(0.6))),
            });
        }
    }
    // characters 4 m ahead, 1.6 m below the camera, side by side, facing it
    let n = cfg.chars.len();
    let fwd = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
    let right = Vec3::new(fwd.z, 0.0, -fwd.x);
    for (k, (pack, model, anim)) in cfg.chars.iter().enumerate() {
        let at = pos + fwd * 4.0 - Vec3::Y * 1.6 - right * ((k as f32 - (n as f32 - 1.0) * 0.5) * 1.2);
        skinned.write(crate::skinned_object_render::SpawnSkinnedModel {
            request: 100 + k as u64,
            pack: *pack,
            model: model.clone(),
            anim: anim.clone(),
            transform: Transform::from_translation(at).with_rotation(Quat::from_rotation_y(yaw)),
        });
    }
}

/// window mode: right mouse look, WASD / QE move, shift fast
fn fly_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    mut state: ResMut<ViewerState>,
    mut cam: Query<&mut Transform, With<ViewerCamera>>,
) {
    let Ok(mut t) = cam.single_mut() else { return };
    if buttons.pressed(MouseButton::Right) {
        state.yaw -= motion.delta.x * 0.003;
        state.pitch = (state.pitch - motion.delta.y * 0.003).clamp(-1.55, 1.55);
        t.rotation = Quat::from_rotation_y(state.yaw) * Quat::from_rotation_x(state.pitch);
    }
    let mut d = Vec3::ZERO;
    for (k, v) in [(KeyCode::KeyW, Vec3::NEG_Z), (KeyCode::KeyS, Vec3::Z), (KeyCode::KeyA, Vec3::NEG_X), (KeyCode::KeyD, Vec3::X), (KeyCode::KeyE, Vec3::Y), (KeyCode::KeyQ, Vec3::NEG_Y)] {
        if keys.pressed(k) {
            d += v;
        }
    }
    let speed = if keys.pressed(KeyCode::ShiftLeft) { 80.0 } else { 15.0 };
    let r = t.rotation;
    t.translation += r * d * speed * time.delta_secs();
}

/// headless: 150 frames after the track is placed, save the offscreen image and exit
fn shot_countdown(
    mut commands: Commands,
    cfg: Res<TrackViewConfig>,
    state: Res<ViewerState>,
    fse: Option<Res<crate::full_screen_effects_manager_pc_xbox::FullScreenEffects>>,
    mut exit: MessageWriter<AppExit>,
    golden_requests: Option<ResMut<crate::golden::GoldenRequests>>,
) {
    if !state.placed {
        return;
    }
    // golden mode: the harness settles, captures (to its own dir) and exits
    if crate::golden::enabled() {
        if state.frames == 150 {
            if let Some(mut r) = golden_requests {
                r.0.push(crate::golden::GoldenRequest { image: state.target.clone(), exit: true });
            }
        }
        return;
    }
    if state.frames == 150 {
        if let (Some(target), Some(path)) = (&state.target, &cfg.shot) {
            commands.spawn(Screenshot::image(target.clone())).observe(save_to_disk(path.clone()));
            // VELOCITY_FSE_DEBUG=1: also save the back buffer the world camera draws into
            if std::env::var("VELOCITY_FSE_DEBUG").is_ok_and(|v| v == "1") {
                if let Some(bb) = fse.as_ref().and_then(|f| f.back_buffer()) {
                    commands.spawn(Screenshot::image(bb)).observe(save_to_disk(path.with_extension("bb.png")));
                }
            }
        }
    }
    if state.frames >= 170 {
        exit.write(AppExit::Success);
    }
}

/// viewer-only `.br` reader (the game's is `velocity_ai`'s VsAIBarriers::parse, VsAIWorld.cpp 0x00033FC0): count at
/// +4, lines from +16, each a 16-byte header (+8 point count) and vec4 points
fn read_br(d: &[u8]) -> Vec<Vec<Vec3>> {
    let rd = |p: usize| d.get(p..p + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
    let f = |p: usize| rd(p).map(f32::from_bits).unwrap_or(0.0);
    let mut out = Vec::new();
    let mut p = 16;
    for _ in 0..rd(4).unwrap_or(0) {
        let Some(n) = rd(p + 8) else { break };
        p += 16;
        if d.len() < p + 16 * n as usize {
            break;
        }
        out.push((0..n as usize).map(|i| Vec3::new(f(p + 16 * i), f(p + 16 * i + 4), f(p + 16 * i + 8))).collect());
        p += 16 * n as usize;
    }
    out
}

/// `--barriers <race dir>`: load the race's barriers once the viewer is placed; `--barrierhit` hits the nearest
/// drawn barrier at frame 140 (the wobble is still running at the frame-150 shot)
fn viewer_barriers(
    cfg: Res<TrackViewConfig>,
    state: Res<ViewerState>,
    mut done: Local<u8>,
    mut load: MessageWriter<crate::barrier_render::LoadBarrierRender>,
    mut hit: MessageWriter<crate::barrier_render::BarrierHit>,
) {
    if !state.placed || *done >= 2 {
        return;
    }
    if *done == 0 {
        let dir = format!("\\Tracks\\{}\\{}", cfg.track, cfg.barriers.as_deref().unwrap_or_default().trim_matches('\\'));
        let read = |f: &str| std::fs::read(visos::file::resolve(&format!("{dir}\\{f}"))).map(|d| read_br(&d)).unwrap_or_default();
        let (arrows, no_entry) = (read("Barier01.br"), read("NoEntr01.br"));
        log::info!("trackview: {} arrow and {} no-entry barriers from {dir}", arrows.len(), no_entry.len());
        load.write(crate::barrier_render::LoadBarrierRender { arrows, no_entry, render: true });
        *done = 1;
    } else if state.frames >= 140 {
        if let Some(p) = cfg.barrier_hit {
            hit.write(crate::barrier_render::BarrierHit { car_a: 0, material: 21, point: p });
        }
        *done = 2;
    }
}

/// `--decals n`: per DECALS.BIN kind (0 bonnet, 1 side, 2 screens, 3 logos) the decals whose texture is in
/// DECALST.VCD (plain, else `_white`, else `_black`), the (n + slot)-th of them in every setup slot
fn viewer_decals(n: u32) -> crate::car_render::CarDecals {
    use velocity_physics::vehicle_database::{CVehicleDatabase, VehicleSetup};
    static DB: std::sync::OnceLock<Option<(CVehicleDatabase, Vec<Vec<(usize, u32)>>)>> = std::sync::OnceLock::new();
    let Some((db, usable)) = DB.get_or_init(|| {
        let db = CVehicleDatabase::load().map_err(|e| log::warn!("decals: {e}")).ok()?;
        let v = visos::file::Vcd::open("\\Vehicles\\DECALST.VCD").ok()?;
        let names: std::collections::HashSet<String> =
            v.entries().map(|e| e.name.to_ascii_lowercase().trim_end_matches(".vtf").to_string()).collect();
        let mut usable = vec![Vec::new(); 4];
        for (i, d) in db.decals.iter().enumerate() {
            let Some(name) = &d.name else { continue };
            if let Some(f) = (0..3).find(|&f| names.contains(&crate::car_render::decal_texture(name, f))) {
                if let Some(u) = usable.get_mut(d.kind as usize) {
                    u.push((i, f));
                }
            }
        }
        Some((db, usable))
    }) else {
        return default();
    };
    let nth = |kind: usize, i: u32| {
        let u = &usable[kind];
        (!u.is_empty()).then(|| u[((n + i) as usize) % u.len()])
    };
    let mut s = VehicleSetup::default();
    s.decal_0 = nth(0, 0).map(|x| x.0);
    s.decal_1 = nth(1, 0).map(|x| x.0);
    let set = |slot: &mut Option<usize>, flags: &mut u32, v: Option<(usize, u32)>| {
        *slot = v.map(|x| x.0);
        *flags = v.map_or(0, |x| x.1);
    };
    for i in 0..10 {
        set(&mut s.decals_3[i], &mut s.decals_3_flags[i], nth(3, i as u32));
    }
    for i in 0..2 {
        set(&mut s.decals_3_1[i], &mut s.decals_3_1_flags[i], nth(3, 10 + i as u32));
        set(&mut s.decals_3_2[i], &mut s.decals_3_2_flags[i], nth(3, 12 + i as u32));
        set(&mut s.decals_3_3[i], &mut s.decals_3_3_flags[i], nth(3, 14 + i as u32));
    }
    set(&mut s.decal_2_4, &mut s.decal_2_4_flags, nth(2, 0));
    set(&mut s.decal_2_5, &mut s.decal_2_5_flags, nth(2, 1));
    crate::car_render::CarDecals::from_setup(&s, db)
}
