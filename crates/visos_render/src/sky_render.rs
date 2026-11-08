//! Render\SkyRender.cpp — `CSkyRender`: camera-centred sky (background, cloud layers, sky cylinder, distant
//! buildings) drawn with its own field of view before the world. Settings come from
//! `\scenery\skies\<name>\SkySetup.cfg`, models from `SKY.VCD`, textures from `SKYT.VCD`.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::renderer::renderer_types::Model;
use crate::renderer::shader_manager::{StageState, XboxStageMaterial};
use crate::renderer::texture_manager::decode_any_vtf;
use crate::model_builder::{Builder, SpawnOpts, TexSet};

pub const MAX_CLOUD_LAYERS: usize = 4;

/// tweak/config units: every distance in the cfg is stored scaled by this
pub const SKY_UNIT: f32 = 0.02;

#[derive(Clone, Copy, Debug)]
pub struct CloudLayer {
    pub height: f32,     // dword_730F04[i], cfg value * 0.02
    pub speed: f32,      // dword_687074[2i]
    pub tiling: f32,     // dword_687044[i]
    pub brightness: f32, // dword_687024[i]
    pub alpha: f32,      // dword_687034[i]
    pub alpha_test: i32, // dword_687090[i]
}

/// the sky globals (`dword_730E30`, `flt_687018`, `byte_730E20`, ...) as one struct.
#[derive(Clone, Debug)]
pub struct SkySetup {
    pub height_offset: f32,        // dword_730E30
    pub fov: f32,                  // flt_687018
    pub cylinder_scale: Vec3,      // byte_730E20, y from the cfg
    pub cylinder_brightness: f32,  // dword_68701C
    pub global_brightness: f32,    // dword_687020
    pub buildings_scale: Vec3,     // byte_730E34, y from the cfg
    pub buildings_brightness: f32, // flt_68700C
    pub background_top: Vec3,      // byte_730E10
    pub sun_rgb: Vec3,             // byte_730E58
    pub sun_width: f32,            // dword_687054
    pub sun_height: f32,           // dword_687058
    pub sun_alpha: f32,            // flt_68705C
    pub sun_rotation: f32,         // dword_687060, degrees
    pub sun_elevation: f32,        // dword_730DE0, degrees
    pub moon_size: f32,            // dword_68706C
    pub moon_rotation: f32,        // dword_687064
    pub moon_elevation: f32,       // dword_687068
    pub moon_rgba: Vec4,           // byte_730E00
    pub clouds: Vec<CloudLayer>,
    /// clouds model scale (`unk_730E44`): mirrored in y so the patch faces down
    pub cloud_scale: Vec3,
    pub buildings_offset: f32, // dword_730E54
}

impl Default for SkySetup {
    /// static initialisers at 0x00425F10.. and the .data defaults at 0x00687000.
    fn default() -> Self {
        Self {
            height_offset: -35.0 * SKY_UNIT,
            fov: 70.0,
            cylinder_scale: Vec3::new(22.0 * SKY_UNIT, 15.0 * SKY_UNIT, 22.0 * SKY_UNIT),
            cylinder_brightness: 1.0,
            global_brightness: 1.0,
            buildings_scale: Vec3::new(22.0 * SKY_UNIT, 4.0 * SKY_UNIT, 22.0 * SKY_UNIT),
            buildings_brightness: 0.5,
            background_top: Vec3::ONE,
            sun_rgb: Vec3::ONE,
            sun_width: 2250.0,
            sun_height: 1120.0,
            sun_alpha: 0.5,
            sun_rotation: -90.0,
            sun_elevation: 0.0,
            moon_size: 250.0,
            moon_rotation: 30.0,
            moon_elevation: 30.0,
            moon_rgba: Vec4::ONE,
            clouds: Vec::new(),
            cloud_scale: Vec3::new(50.0 * SKY_UNIT, -50.0 * SKY_UNIT, 50.0 * SKY_UNIT),
            buildings_offset: 0.0,
        }
    }
}

/// values of every `( a, b, ... )` group in order — the cfg reader (`VsStringAnsi` 0x002B3190 / 0x002B3520 /
/// 0x002B3730) pulls the next parenthesised number(s) regardless of the label in front of them.
fn paren_values(text: &str) -> Vec<Vec<f32>> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find('(') {
        let Some(j) = rest[i..].find(')') else { break };
        out.push(rest[i + 1..i + j].split(',').filter_map(|v| v.trim().parse::<f32>().ok()).collect());
        rest = &rest[i + j + 1..];
    }
    out
}

impl SkySetup {
    /// SkyRender.cpp 0x0013ED90 — load `SkySetup.cfg` (version 1). Missing file: defaults.
    pub fn load(sky: &str) -> Self {
        let mut s = Self::default();
        let path = visos::file::resolve(&format!("\\scenery\\skies\\{sky}\\SkySetup.cfg"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            log::warn!("Failed to Load {} - using default values", path.display());
            return s;
        };
        let g = paren_values(&text);
        let mut k = 0usize;
        let mut next = || {
            k += 1;
            g.get(k - 1).cloned().unwrap_or_default()
        };
        let f = |v: Vec<f32>| v.first().copied().unwrap_or(0.0);
        let v3 = |v: Vec<f32>| Vec3::new(v.first().copied().unwrap_or(0.0), v.get(1).copied().unwrap_or(0.0), v.get(2).copied().unwrap_or(0.0));
        if f(next()) as i32 != 1 {
            log::error!("ERROR: {} version number should be 1", path.display());
            return s;
        }
        s.height_offset = f(next()) * SKY_UNIT;
        s.fov = f(next());
        s.cylinder_scale.y = f(next()) * SKY_UNIT;
        s.cylinder_brightness = f(next());
        s.global_brightness = f(next());
        s.buildings_scale.y = f(next()) * SKY_UNIT;
        s.buildings_brightness = f(next());
        s.background_top = v3(next());
        s.sun_rgb = v3(next());
        s.sun_width = f(next());
        s.sun_height = f(next());
        s.sun_alpha = f(next());
        s.sun_rotation = f(next());
        s.sun_elevation = f(next());
        s.moon_size = f(next());
        s.moon_rotation = f(next());
        s.moon_elevation = f(next());
        let m = next();
        let c = |i: usize| m.get(i).copied().unwrap_or(1.0);
        s.moon_rgba = Vec4::new(c(0), c(1), c(2), c(3));
        let mut n = f(next()) as i32;
        if n > MAX_CLOUD_LAYERS as i32 {
            log::warn!("WARNING: Max cloud layers {n} exceeded");
            n = MAX_CLOUD_LAYERS as i32;
        }
        for _ in 0..n.max(0) {
            s.clouds.push(CloudLayer {
                height: f(next()) * SKY_UNIT,
                speed: f(next()),
                tiling: f(next()),
                brightness: f(next()),
                alpha: f(next()),
                alpha_test: f(next()) as i32,
            });
        }
        s
    }
}

/// the sky camera: same orientation as the main camera, at the origin, own projection.
#[derive(Component)]
pub struct SkyCamera;

/// sky passes in draw order (`CSkyRender::Render` 0x0013BF30): every sky shader has z test and z write off, so
/// the call order is the only ordering. Each pass keeps its render layer; one camera draws all four (the original
/// draws them as calls inside the view): the background quad is the only opaque item (opaque phase first), the
/// blended passes sort by `sky_pass_bias` (`VELOCITY_PERF_OFF=skyopt`: one camera per pass, the old path)
pub const SKY_PASS_BACKGROUND: usize = 1;
pub const SKY_PASS_CLOUDS: usize = 2;
pub const SKY_PASS_CYLINDER: usize = 3;
pub const SKY_PASS_BUILDINGS: usize = 4;

/// one camera for all sky passes (perf; see `SKY_PASS_BACKGROUND`)
pub fn sky_one_camera() -> bool {
    !crate::golden::perf_off("skyopt")
}

/// transparent sort bias of a pass in the shared sky camera: earlier passes sort farther (drawn first); steps far
/// beyond the sky's own depth range (far plane 2000), small enough to keep f32 distances exact to ~0.002
fn sky_pass_bias(pass: usize) -> i16 {
    if sky_one_camera() {
        (pass as i16 - SKY_PASS_BUILDINGS as i16) * 10000
    } else {
        0
    }
}

/// sky shader settings built in code (`CSkyRender::Load` 0x0013B5D0): blend mode, clamp, z test and write off.
fn sky_state(blend: u32, clamp: bool) -> StageState {
    let mut st = StageState::from_commands(&[0]);
    st.blend_mode = blend;
    st.z_enable = false;
    st.z_write = false;
    if clamp {
        st.address_u = 3;
        st.address_v = 3;
    }
    st
}

/// SkyRender.cpp 0x0013B5D0 (load: models, textures, code-built shaders) + 0x0013BF30 / 0x0013CB90 (matrices).
/// spawns one camera per pass (orders -3..0, sky fov, at the origin, following the main camera's rotation).
pub fn spawn_sky(
    commands: &mut Commands,
    b: &mut Builder,
    sky: &str,
    setup: &SkySetup,
    root: Entity,
) {
    let bgc = setup.background_top;
    let mut cams = Vec::new();
    let one = sky_one_camera();
    let passes: &[usize] = if one { &[SKY_PASS_BACKGROUND] } else { &[SKY_PASS_BACKGROUND, SKY_PASS_CLOUDS, SKY_PASS_CYLINDER, SKY_PASS_BUILDINGS] };
    for &pass in passes {
        let clear = if pass == SKY_PASS_BACKGROUND {
            ClearColorConfig::Custom(Color::srgb(bgc.x, bgc.y, bgc.z))
        } else {
            ClearColorConfig::None
        };
        let layers = if one {
            RenderLayers::from_layers(&[SKY_PASS_BACKGROUND, SKY_PASS_CLOUDS, SKY_PASS_CYLINDER, SKY_PASS_BUILDINGS])
        } else {
            RenderLayers::layer(pass)
        };
        let id = commands
            .spawn((
                Camera3d::default(),
                Camera { order: crate::track_render::WORLD_CAMERA_ORDER - 5 + pass as isize, clear_color: clear, ..default() },
                bevy::core_pipeline::tonemapping::Tonemapping::None,
                Projection::Perspective(PerspectiveProjection { fov: setup.fov.to_radians(), near: 0.05, far: 2000.0, ..default() }),
                Transform::default(),
                layers,
                SkyCamera,
                ChildOf(root),
            ))
            .id();
        cams.push(id);
    }
    let models = visos::file::Vcd::open(&format!("\\scenery\\skies\\{sky}\\SKY.VCD"));
    let textures = visos::file::Vcd::open(&format!("\\scenery\\skies\\{sky}\\SKYT.VCD"));
    let (Ok(models), Ok(textures)) = (models, textures) else {
        log::warn!("sky {sky}: missing SKY.VCD / SKYT.VCD");
        return;
    };
    let mut tex = TexSet::default();
    for e in textures.entries() {
        if let Some(t) = textures.data(&e).and_then(|d| decode_any_vtf(d).ok()) {
            tex.add(e.name, t);
        }
    }
    let sun_rot = 0.017453292f32 * setup.sun_rotation;
    let model = |name: &str| models.get(name).and_then(|d| Model::parse(d).ok());
    let mut clouds = Vec::new();
    // "bg": SkyGradient, blend 1, clamp, screen-filling quad (RenderBackground 0x0013CF40)
    {
        let d = 900.0f32;
        let hh = d * (setup.fov.to_radians() * 0.5).tan();
        // original blend 1 (rgba write); 0 (rgb only) here: the alpha-0 background would make bevy's camera
        // compositing drop the world drawn over it, and the frame buffer alpha is never displayed
        let st = sky_state(0, true);
        let texture = tex.image("skygradient", 3, 3, b.images);
        let mut params = crate::renderer::shader_manager::XboxStageParams::new(&st);
        // VISOS_ColourXBOX(0, r, g, b): the background colour is the vertex diffuse, alpha 0
        params.colour = Vec4::new(bgc.x, bgc.y, bgc.z, 0.0);
        let mat = b.materials.add(XboxStageMaterial { params, texture, stencil: None, spec_texture: None, state: st });
        // unit-aspect quad, stretched to the camera aspect each frame in `update_sky`
        let mut mesh = Mesh::from(Rectangle::new(2.0 * hh, 2.0 * hh));
        let n = mesh.count_vertices();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32; 4]; n]);
        commands.spawn((
            Mesh3d(b.meshes.add(mesh)),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(0.0, 0.0, -d),
            RenderLayers::layer(SKY_PASS_BACKGROUND),
            ChildOf(cams[0]),
            SkyBackground(mat),
        ));
    }
    // "cloud": Cloud_layer_0N, blend 3, colour = texture * TFACTOR (the global sky colour)
    if let Some(m) = model("sky_clouds.xmf") {
        for (i, c) in setup.clouds.iter().enumerate() {
            let xf = Mat4::from_translation(Vec3::new(0.0, c.height, 0.0))
                * Mat4::from_rotation_y(sun_rot + 3.14159)
                * Mat4::from_scale(setup.cloud_scale);
            let k = c.brightness * setup.global_brightness;
            let opts = SpawnOpts {
                layer: Some(RenderLayers::layer(SKY_PASS_CLOUDS)),
                parent: Some(root),
                colour: Vec4::new(k, k, k, c.alpha),
                tfactor: true,
                texture: Some(format!("cloud_layer_{:02}", i + 1)),
                state: Some(|s: &mut StageState| {
                    *s = sky_state(3, false);
                    s.depth_bias = sky_pass_bias(SKY_PASS_CLOUDS);
                }),
                ..default()
            };
            let mats = b.add_model(commands, &mut tex, &m, xf, &opts);
            clouds.push((mats, i));
        }
    }
    // "skycyl": Sky_cylinder, blend 3, clamp
    if let Some(m) = model("sky_cylinder.xmf") {
        let xf = Mat4::from_translation(Vec3::new(0.0, setup.height_offset, 0.0))
            * Mat4::from_rotation_y(sun_rot)
            * Mat4::from_scale(setup.cylinder_scale);
        let k = setup.global_brightness * setup.cylinder_brightness;
        let opts = SpawnOpts {
            layer: Some(RenderLayers::layer(SKY_PASS_CYLINDER)),
                parent: Some(root),
            colour: Vec4::new(k, k, k, 1.0),
            texture: Some("sky_cylinder".into()),
            state: Some(|s: &mut StageState| {
                *s = sky_state(3, true);
                s.depth_bias = sky_pass_bias(SKY_PASS_CYLINDER);
            }),
            ..default()
        };
        b.add_model(commands, &mut tex, &m, xf, &opts);
    }
    // buildings shader (+0x54): sky_cylinder_buildings, blend 3
    if let Some(m) = model("sky_buildings.xmf") {
        let xf = Mat4::from_translation(Vec3::new(0.0, setup.buildings_offset, 0.0)) * Mat4::from_scale(setup.buildings_scale);
        let k = setup.buildings_brightness;
        let opts = SpawnOpts {
            layer: Some(RenderLayers::layer(SKY_PASS_BUILDINGS)),
                parent: Some(root),
            colour: Vec4::new(k, k, k, 1.0),
            texture: Some("sky_cylinder_buildings".into()),
            state: Some(|s: &mut StageState| {
                *s = sky_state(3, false);
                s.depth_bias = sky_pass_bias(SKY_PASS_BUILDINGS);
            }),
            ..default()
        };
        b.add_model(commands, &mut tex, &m, xf, &opts);
    }
    // sun and moon billboards (`CSkyRender::RenderSunMoon` 0x0013C390, drawn by GameManager 0x000B9010 after the
    // world, z tested): "sunb" = sun_poly (VFX\misc.vcd), blend 14, no z write; "moon" = the sky's Moon, blend 3
    if let Ok(misc) = visos::file::Vcd::open(r"\VFX\misc.vcd") {
        if let Some(t) = misc.get("sun_poly.vtf").and_then(|d| decode_any_vtf(d).ok()) {
            tex.add("sun_poly.vtf", t);
        }
    }
    let mut billboard = |tex: &mut TexSet, key: &str, blend: u32, colour: Vec4, size: Vec2, moon: bool| {
        let texture = tex.image(key, 0, 0, b.images)?;
        let mut st = sky_state(blend, false);
        st.z_enable = true;
        let mut params = crate::renderer::shader_manager::XboxStageParams::new(&st);
        params.colour = colour;
        let mat = b.materials.add(XboxStageMaterial { params, texture: Some(texture), stencil: None, spec_texture: None, state: st });
        let mut mesh = Mesh::from(Rectangle::new(size.x, size.y));
        let n = mesh.count_vertices();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32; 4]; n]);
        Some(commands.spawn((
            Mesh3d(b.meshes.add(mesh)),
            MeshMaterial3d(mat),
            Transform::default(),
            bevy::camera::visibility::NoFrustumCulling,
            SunMoon { moon },
            ChildOf(root),
        )))
        .map(|_| ())
    };
    // `dword_6870A0 & 2` (0x3F: on); shader index `dword_6870AC` = 1 = "sunb"
    let s = setup.sun_rgb;
    billboard(&mut tex, "sun_poly", 14, Vec4::new(s.x, s.y, s.z, (255.0 * setup.sun_alpha).round() / 255.0), Vec2::new(setup.sun_width, setup.sun_height), false);
    billboard(&mut tex, "moon", 3, setup.moon_rgba, Vec2::splat(setup.moon_size), true);
    commands.insert_resource(SkySetupRes(setup.clone()));
    commands.insert_resource(SkyState {
        clouds: setup
            .clouds
            .iter()
            .zip(0..)
            .map(|(c, i)| CloudState {
                materials: clouds.iter().find(|(_, j)| *j == i).map(|(m, _)| m.clone()).unwrap_or_default(),
                speed_u: 0.0,
                speed_v: c.speed,
                tiling: c.tiling,
                u: 0.0,
                v: 0.0,
            })
            .collect(),
    });
}

/// sun / moon billboard (`CSkyRender::RenderSunMoon` 0x0013C390)
#[derive(Component)]
pub struct SunMoon {
    pub moon: bool,
}

/// `CSkyRender::DirectionFromAngles` 0x0013E380 (degrees)
pub fn direction_from_angles(rotation: f32, elevation: f32) -> Vec3 {
    let c = (0.017453292f32 * elevation).cos();
    Vec3::new((0.017453292f32 * rotation).sin() * c, (0.017453292f32 * elevation).sin(), (0.017453292f32 * rotation).cos() * c)
}

/// sun / moon placement each frame (0x0013C390): centre = camera + dir * 1000 (`flt_6870B0`), right = dir x up
/// (not normalised, so the width shrinks with elevation), the other edge along world up.
pub fn update_sun_moon(
    setup: Option<Res<SkySetupRes>>,
    main: Query<&GlobalTransform, (With<crate::track_render::TrackCamera>, Without<SunMoon>)>,
    mut q: Query<(&SunMoon, &mut Transform)>,
) {
    let (Some(setup), Some(cam)) = (setup, main.iter().next()) else { return };
    for (sm, mut t) in &mut q {
        let dir = if sm.moon {
            direction_from_angles(setup.0.moon_rotation, setup.0.moon_elevation)
        } else {
            direction_from_angles(setup.0.sun_rotation, setup.0.sun_elevation)
        };
        let right = dir.cross(Vec3::Y);
        let rn = right.normalize_or_zero();
        if rn == Vec3::ZERO {
            continue;
        }
        let z = rn.cross(Vec3::Y);
        t.translation = cam.translation() + dir * 1000.0;
        t.rotation = Quat::from_mat3(&Mat3::from_cols(rn, Vec3::Y, z));
        t.scale = Vec3::new(right.length(), 1.0, 1.0);
    }
}

/// the loaded sky's settings
#[derive(Resource, Clone)]
pub struct SkySetupRes(pub SkySetup);

/// background quad (`CSkyRender::RenderBackground` 0x13CF40)
#[derive(Component)]
pub struct SkyBackground(pub Handle<XboxStageMaterial>);

pub struct CloudState {
    pub materials: Vec<Handle<XboxStageMaterial>>,
    pub speed_u: f32, // dword_687070[2i] (0 in .data, not in the cfg)
    pub speed_v: f32, // dword_687074[2i], the cfg "Speed"
    pub tiling: f32,
    pub u: f32, // CSkyRender +0x58 + 4i
    pub v: f32, // CSkyRender +0x68 + 4i
}

#[derive(Resource)]
pub struct SkyState {
    pub clouds: Vec<CloudState>,
}

/// `dword_6870A4`: scroll scale per unit of frame time
pub const CLOUD_SCROLL_SCALE: f32 = 9.999_999_7e-6;
/// default vertical field of view of the game camera (`dword_6870B8`, the background gradient span)
pub const BACKGROUND_SPAN: f32 = f32::from_bits(0x3F51_89F2);

/// SkyRender.cpp 0x0013BE20 CSkyRender::UpdateClouds (frame time in milliseconds) + texture matrix of `RenderClouds` 0x0013CB90
/// (scale = tiling, offset = scroll) + background v range of `RenderBackground` 0x0013CF40
/// (v = -asin(forward.y) * k .. + 0.8185 * k, k = 1/vertical fov).
pub fn update_sky(
    time: Res<Time>,
    state: Option<ResMut<SkyState>>,
    mut mats: ResMut<Assets<XboxStageMaterial>>,
    main: Query<(&Transform, &Projection), (With<crate::track_render::TrackCamera>, Without<SkyCamera>)>,
    mut bg: Query<(&SkyBackground, &mut Transform), (Without<Camera3d>, Without<crate::track_render::TrackCamera>)>,
) {
    let Some(mut state) = state else { return };
    let dt_ms = time.delta_secs() * 1000.0;
    for c in &mut state.clouds {
        c.u += dt_ms * CLOUD_SCROLL_SCALE * c.speed_u;
        c.v += dt_ms * CLOUD_SCROLL_SCALE * c.speed_v;
        if c.v < 0.0 {
            c.v += 1.0;
        }
        if c.v > 1.0 {
            c.v -= 1.0;
        }
        for h in &c.materials {
            if let Some(mut m) = mats.get_mut(h) {
                m.params.uv_offset = Vec2::new(c.u, c.v);
                m.params.uv_scale = Vec4::new(c.tiling, c.tiling, 1.0, 1.0);
            }
        }
    }
    if let (Some((t, proj)), Ok((bg, mut bgt))) = (main.iter().next(), bg.single_mut()) {
        let (fov, aspect) = match proj {
            Projection::Perspective(p) => (p.fov, p.aspect_ratio),
            _ => (BACKGROUND_SPAN, 4.0 / 3.0),
        };
        bgt.scale = Vec3::new(aspect, 1.0, 1.0);
        let k = 1.0 / fov;
        let pitch = t.forward().y.clamp(-1.0, 1.0).asin();
        let top = (0.0 - pitch) * k;
        if let Some(mut m) = mats.get_mut(&bg.0) {
            m.params.uv_offset = Vec2::new(0.0, top);
            m.params.uv_scale = Vec4::new(1.0, BACKGROUND_SPAN * k, 1.0, 1.0);
        }
    }
}

/// keep the sky camera's rotation equal to the main camera's.
pub fn sync_sky_camera(
    main: Query<&GlobalTransform, (With<crate::track_render::TrackCamera>, Without<SkyCamera>)>,
    mut sky: Query<&mut Transform, (With<SkyCamera>, Without<crate::view_render::ViewSky>)>,
) {
    let Some(m) = main.iter().next() else { return };
    for mut t in &mut sky {
        t.rotation = m.compute_transform().rotation;
        t.translation = Vec3::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// every SkySetup.cfg in the install loads as version 1.
    #[test]
    fn load_every_sky() {
        let dir = visos::file::resolve("\\scenery\\skies");
        let mut n = 0;
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            if !e.path().join("SKYSETUP.CFG").exists() && !e.path().join("SkySetup.cfg").exists() {
                continue;
            }
            let name = e.file_name().to_string_lossy().into_owned();
            let s = SkySetup::load(&name);
            assert!(s.fov > 10.0 && s.fov < 120.0, "{name}: {s:?}");
            n += 1;
        }
        assert!(n >= 3);
        let la = SkySetup::load("LA");
        assert_eq!(la.clouds.len(), 2);
        assert_eq!(la.clouds[1].alpha_test, 128);
        assert!((la.buildings_scale.y - 2.8 * SKY_UNIT).abs() < 1e-6);
    }
}
