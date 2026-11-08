//! src\Osd\OSD3dInterface.cpp — the in-race 3D hud (`COSD3DInterface`, `byte_6FC098`): the TFT (frame, hinge and
//! screen models from `\OSD\OSDM.vcd`, textures from `\OSD\OSDT.vcd`, the screen showing the 64x64 "movie_test"
//! texture with the "LCD_Movie" shader), the map frame / screen models, and the speedo, all drawn with the 3d
//! interface camera (VsCameraManager camera 5: eye (0, 0, 30) looking at the origin, vertical fov 45 degrees,
//! 4:3, near 1, far 600; GameManager 0x000B75D0, selected by the render 0x000BA112 before 0x000EA2B0).
//!
//! Model slots (96 bytes each, 0x000E9C30: model, render struct +4 with the world matrix pointer and the light set
//! +2824 at +16, matrix +32): +2344 `tft_frame`, +2440 `tft_hinge`, +2536 `tft_screen`, +2632 `hud_map_frame`,
//! +2728 `hud_map_screen` (names strcpy'd from `off_686004..686014` into the hud config at +832 / +896 / +960 /
//! +1280 / +1344 by OSD3dInterfaceConfiguration 0x000EC874 / 0x000ECB4F).
//!
//! Light set (+2824): the player car's VsAIRaceVehicle +1028 set, colours 0 / 1 x `flt_686000` 1.5, ambient lerped
//! to the police flash +2912 (`VsLights::hud_light_set`), bound to the five slot models and the map screen layers;
//! LCD_Movie (GFXHelper 0x0012AE10 arg 0) is unlit. STAND-IN: the map
//! screen's per-layer texture matrices (COSDMapRender::Render 0x000E9670: the map, grid, police and player blip
//! shaders each drawn over `hud_map_screen`) are composited on the cpu into one texture ([`MAP_SCREEN_SIZE`]).

use bevy::prelude::*;
use velocity_game::camera::vs_base_camera::yaw_pitch_roll;
use velocity_game::camera::vs_camera_manager::INTERFACE_FOV;

/// the 3d interface camera (GameManager.cpp 0x000B75D0): `look_at((0, 0, 30), 0, y)`
pub const INTERFACE_EYE: [f32; 3] = [0.0, 0.0, 30.0];
/// VsBaseCamera default aspect
pub const INTERFACE_ASPECT: f32 = 4.0 / 3.0;
/// VsCameraManager `dword_6850A4` / `dword_6850A0`
pub const INTERFACE_NEAR: f32 = 1.0;
pub const INTERFACE_FAR: f32 = 600.0;

/// the render layer and camera order of the 3D hud view (drawn over the world and the full-screen effects present
/// at order 9, under the 2D OSD overlay at 11)
pub const HUD_3D_LAYER: usize = 12;
pub const HUD_3D_ORDER: isize = 10;
/// player 1's 3D hud view in a split screen race (its own camera with the bottom / right viewport)
pub const HUD_3D_LAYER_P2: usize = 23;

/// side of the cpu-composited map screen texture
pub const MAP_SCREEN_SIZE: u32 = 256;

/// the model names (`off_686004` .. `off_686014`)
pub const TFT_FRAME: &str = "tft_frame";
pub const TFT_HINGE: &str = "tft_hinge";
pub const TFT_SCREEN: &str = "tft_screen";
pub const MAP_FRAME: &str = "hud_map_frame";
pub const MAP_SCREEN: &str = "hud_map_screen";

/// `dword_685FF8`: TFT flip step per render call (radians)
const FLIP_STEP: f32 = f32::from_bits(0x3DCC_CCCD);
/// `dword_685FF4`: TFT drop step per render call
const DROP_STEP: f32 = f32::from_bits(0x3E23_D70A);
/// `dword_685FFC`: TFT drop when closed (`dword_6FD1E0`'s first value)
pub const DROP_MAX: f32 = 3.0;
/// +2900 closed: `0xC0490FDB`
const FLIP_CLOSED: f32 = f32::from_bits(0xC049_0FDB);
/// 0x000EAD80: the movie shows once the frame has flipped past this
const FLIP_SHOW: f32 = -1.3;

/// a hud point (HUD units, y up) through the 3d interface camera onto the 512x512 2D target
pub fn project(p: [f32; 3]) -> [f32; 2] {
    let t = (INTERFACE_FOV * 0.5).tan();
    let d = INTERFACE_EYE[2] - p[2];
    let nx = p[0] / (d * t * INTERFACE_ASPECT);
    let ny = p[1] / (d * t);
    [(nx + 1.0) * 0.5 * crate::render2d::SCREEN_W as f32, (1.0 - ny) * 0.5 * crate::render2d::SCREEN_H as f32]
}

/// Renderer.cpp 0x002AB7E0 (rotation x / y / z) + VsTrafficManager 0x00042CD0 (row 3 = position, w 1)
pub fn model_matrix(rot: [f32; 3], pos: [f32; 3]) -> Mat4 {
    Mat4::from_mat3_translation(yaw_pitch_roll(rot[0], rot[1], rot[2]), Vec3::from_array(pos))
}

/// what the 3D hud draws this frame (the bevy side builds the models once and places them)
#[derive(Clone, Debug, Default)]
pub struct Osd3dFrame {
    /// inside 0x000EA2B0's OSD gate this frame
    pub drawn: bool,
    /// the TFT models were placed (never in a split screen race, `byte_6F7157`)
    pub tft: bool,
    pub tft_hinge: Mat4,
    pub tft_frame: Mat4,
    /// 0x000EAD80: the screen shows the movie ("LCD_Movie") instead of its own texture
    pub tft_movie: bool,
    pub map: Mat4,
    /// the map screen texture was drawn this frame (`hud_map_screen` hidden otherwise)
    pub map_screen: bool,
}

/// `COSD3DInterface`'s per-race TFT state
#[derive(Clone, Debug)]
pub struct COSD3DInterface {
    /// +2900 TFT flip angle, -pi (closed) .. 0 (open); -pi at load (0x000E9C90)
    pub flip: f32,
    /// hud config +800 / +816 single player TFT position / rotation
    pub tft: [[f32; 3]; 2],
    /// hud config +1248 / +1264 single player map position / rotation
    pub map: [[f32; 3]; 2],
    /// `byte_6F7157`: two players, no TFT; `map` is then player 0's split slot (+1248) and `map2` player 1's (+1796)
    pub split: bool,
    pub map2: [[f32; 3]; 2],
    pub out: Osd3dFrame,
    /// player 1's view (`a3` = 1): only its map frame / screen
    pub out2: Osd3dFrame,
}

impl COSD3DInterface {
    pub fn new(tft: [[f32; 3]; 2], map: [[f32; 3]; 2]) -> Self {
        Self { flip: FLIP_CLOSED, tft, map, split: false, map2: map, out: Osd3dFrame::default(), out2: Osd3dFrame::default() }
    }

    /// OSD3dInterface.cpp 0x000EA2B0 / 0x000EAD80 / OSD3dInterfaceConfiguration.cpp 0x000ECFD0 / 0x000ED080 — the TFT (+ LCD) / map part, after the speedo, for view 0 (and view 1 when split).
    /// `drop` is the static `dword_6FD1E0`, `slide` the hud slide (+2932), `wanted` = +2896, `frames_ok` = +2904 >= 2.
    pub fn render(&mut self, drop: &mut f32, slide: f32, wanted: bool, frames_ok: bool) {
        if self.split {
            // the TFT block is `if (!byte_6F7157)`; the map uses the view's config slot (0x000ECFD0 / 0x000ED080)
            self.out.tft = false;
            self.out.map = Self::map_matrix(self.map, slide);
            self.out.drawn = true;
            return;
        }
        self.out.tft = true;
        let mut pos = self.tft[0];
        let rot = self.tft[1];
        pos[1] += *drop;
        if wanted {
            if self.flip < 0.0 {
                self.flip += FLIP_STEP;
            }
            if self.flip > 0.0 {
                self.flip = 0.0;
            }
            if *drop > 0.0 {
                *drop -= DROP_STEP;
            }
            if *drop < 0.0 {
                *drop = 0.0;
            }
        } else {
            if self.flip > FLIP_CLOSED {
                self.flip -= FLIP_STEP;
            }
            if self.flip < FLIP_CLOSED {
                self.flip = FLIP_CLOSED;
            }
            if DROP_MAX > *drop {
                *drop += DROP_STEP;
            }
            if *drop > DROP_MAX {
                *drop = DROP_MAX;
            }
        }
        // +2472 hinge at the TFT rotation, +2376 frame flipped about x (the screen is drawn with the frame's matrix)
        self.out.tft_hinge = model_matrix(rot, pos);
        self.out.tft_frame = model_matrix([rot[0] + self.flip, rot[1], rot[2]], pos);
        // 0x000EAD80
        self.out.tft_movie = !(!wanted || !frames_ok || self.flip < FLIP_SHOW);
        self.out.map = Self::map_matrix(self.map, slide);
        self.out.drawn = true;
    }

    /// OSD3dInterface.cpp 0x000EA2B0 for view 1 (split screen only): player 1's map slot
    pub fn render_view1(&mut self, slide: f32) {
        self.out2.tft = false;
        self.out2.map = Self::map_matrix(self.map2, slide);
        self.out2.drawn = true;
    }

    /// +2664 / +2760 map frame and screen: position y += slide, x += slide
    fn map_matrix(map: [[f32; 3]; 2], slide: f32) -> Mat4 {
        let mut mpos = map[0];
        mpos[1] += slide;
        mpos[0] += slide;
        model_matrix(map[1], mpos)
    }
}

/// the 3d interface camera's projection: VsBaseCamera's aspect stays 4:3 whatever the viewport (VsCameraManager
/// 0x0005F8F0), so a split view shows the full-screen hud squashed into its half (bevy would refit the aspect)
#[derive(Clone, Debug)]
pub struct InterfaceProjection(pub PerspectiveProjection);

impl bevy::camera::CameraProjection for InterfaceProjection {
    fn get_clip_from_view(&self) -> Mat4 {
        self.0.get_clip_from_view()
    }
    fn get_clip_from_view_for_sub(&self, sub_view: &bevy::camera::SubCameraView) -> Mat4 {
        self.0.get_clip_from_view_for_sub(sub_view)
    }
    fn update(&mut self, _width: f32, _height: f32) {}
    fn far(&self) -> f32 {
        self.0.far
    }
    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [bevy::math::Vec3A; 8] {
        self.0.get_frustum_corners(z_near, z_far)
    }
}

/// the bevy side of the 3D hud: its camera and the model roots (hidden until a frame is drawn)
pub struct Osd3dView {
    pub camera: Entity,
    pub tft_hinge: Entity,
    pub tft_frame: Entity,
    /// `tft_screen` with its own shader (FuryInstance 0x0012A7D0 with the frame's render struct)
    pub tft_screen: Entity,
    /// `tft_screen` with "LCD_Movie" (GFXHelper 0x0012AE10: texture +2860, blend 1, z off)
    pub tft_movie: Entity,
    pub map_frame: Entity,
    pub map_screen: Entity,
    pub movie_materials: Vec<Handle<visos_render::renderer::shader_manager::XboxStageMaterial>>,
    /// the lit materials of the five slot models (render struct light set +2824)
    pub lit_materials: Vec<Handle<visos_render::renderer::shader_manager::XboxStageMaterial>>,
    pub map_image: Handle<Image>,
}

/// OSD3dInterface.cpp 0x000EA2B0 — the light set the slot models are drawn with this render: the player car's set
/// (`car`, the view player's; `VsLights::default()` without one) as `hud_light_set(flash)`, flash = +2912.
pub fn hud_lights(car: Option<&visos_render::vs_render::VsLights>, flash: [f32; 4]) -> visos_render::vs_render::VsLights {
    car.copied().unwrap_or_default().hud_light_set(Vec4::from_array(flash))
}

/// COSDMapRender.cpp 0x000E9C90 — `Register`: the five models from `\OSD\OSDM.vcd` (textures `\OSD\OSDT.vcd`), the
/// LCD_Movie shader, the 3d interface camera.
pub fn spawn_view(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<visos_render::renderer::shader_manager::XboxStageMaterial>,
    images: &mut Assets<Image>,
    layer: usize,
) -> Osd3dView {
    use bevy::asset::RenderAssetUsages;
    use bevy::camera::visibility::RenderLayers;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    use visos_render::model_builder::{Builder, SpawnOpts, TexSet};
    use visos_render::renderer::renderer_types::Model;
    let camera = commands
        .spawn((
            Camera3d::default(),
            Camera { order: HUD_3D_ORDER, clear_color: ClearColorConfig::None, is_active: false, ..default() },
            bevy::core_pipeline::tonemapping::Tonemapping::None,
            Projection::custom(InterfaceProjection(PerspectiveProjection {
                fov: INTERFACE_FOV,
                aspect_ratio: INTERFACE_ASPECT,
                near: INTERFACE_NEAR,
                far: INTERFACE_FAR,
                ..default()
            })),
            Transform::from_translation(Vec3::from_array(INTERFACE_EYE)).looking_at(Vec3::ZERO, Vec3::Y),
            RenderLayers::layer(layer),
        ))
        .id();
    let mut tex = TexSet::default();
    if let Ok(v) = visos::file::Vcd::open("\\OSD\\OSDT.vcd") {
        for en in v.entries() {
            if let Some(t) = v.data(&en).and_then(|d| visos_render::renderer::texture_manager::decode_any_vtf(d).ok()) {
                tex.add(en.name, t);
            }
        }
    }
    let osdm = visos::file::Vcd::open("\\OSD\\OSDM.vcd").map_err(|e| log::error!("\\OSD\\OSDM.vcd: {e:?}")).ok();
    let mut spawn = |commands: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<_>, images: &mut Assets<Image>, name: &str, lit: bool| {
        let root = commands.spawn((Transform::default(), Visibility::Hidden)).id();
        let mut mats = Vec::new();
        // LoadModelGeometry 0x0009FE10: "Failed to load model" leaves the slot empty (nothing drawn)
        if let Some(m) = osdm.as_ref().and_then(|v| v.get(&format!("{name}.xmf"))).and_then(|d| Model::parse(d).ok()) {
            let mut b = Builder::new(meshes, materials, images);
            let mut opts = SpawnOpts::world();
            opts.layer = Some(RenderLayers::layer(layer));
            opts.parent = Some(root);
            // the render struct's light set (+16 -> +2824); the builder returns lit material copies
            if lit {
                opts.light_set = Some(visos_render::vs_render::VsLights::default());
            }
            mats =b.add_model(commands, &mut tex, &m, Mat4::IDENTITY, &opts);
            log::debug!("3d hud {name}: {} triangles, bounds {:?}..{:?}", b.tris, b.bounds_min, b.bounds_max);
        } else {
            log::error!("Failed to load model \"{name}\"");
        }
        (root, mats)
    };
    let (tft_frame, m0) = spawn(commands, meshes, materials, images, TFT_FRAME, true);
    let (tft_hinge, m1) = spawn(commands, meshes, materials, images, TFT_HINGE, true);
    let (tft_screen, m2) = spawn(commands, meshes, materials, images, TFT_SCREEN, true);
    let (tft_movie, movie_materials) = spawn(commands, meshes, materials, images, TFT_SCREEN, false);
    let (map_frame, m3) = spawn(commands, meshes, materials, images, MAP_FRAME, true);
    let (map_screen, map_materials) = spawn(commands, meshes, materials, images, MAP_SCREEN, true);
    let lit_materials: Vec<_> = [m0, m1, m2, m3, map_materials.clone()].into_iter().flatten().collect();
    // "LCD_Movie": SetTexture(movie_test), SetBlendMode(1, 0), SetZState(0, 0, 0)
    for h in &movie_materials {
        if let Some(mut m) = materials.get_mut(h) {
            m.texture = None;
            m.state.blend_mode = 1;
            m.state.z_enable = false;
            m.state.z_write = false;
        }
    }
    let map_image = images.add(Image::new_fill(
        Extent3d { width: MAP_SCREEN_SIZE, height: MAP_SCREEN_SIZE, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    for h in &map_materials {
        if let Some(mut m) = materials.get_mut(h) {
            m.texture = Some(map_image.clone());
        }
    }
    Osd3dView { camera, tft_hinge, tft_frame, tft_screen, tft_movie, map_frame, map_screen, movie_materials, lit_materials, map_image }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// every OSDM.vcd model parses, and the five the 3D hud loads are there with their OSDT textures
    #[test]
    fn load_every_osdm_model() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let v = visos::file::Vcd::open("\\OSD\\OSDM.vcd").unwrap();
        let t = visos::file::Vcd::open("\\OSD\\OSDT.vcd").unwrap();
        let mut n = 0;
        for e in v.entries() {
            let d = v.data(&e).unwrap();
            let m = visos_render::renderer::renderer_types::Model::parse(d).unwrap_or_else(|err| panic!("{}: {err}", e.name));
            assert!(!m.objects.is_empty(), "{}", e.name);
            n += 1;
        }
        assert!(n >= 30, "{n}");
        for name in [TFT_FRAME, TFT_HINGE, TFT_SCREEN, MAP_FRAME, MAP_SCREEN] {
            let d = v.get(&format!("{name}.xmf")).unwrap_or_else(|| panic!("{name}"));
            let m = visos_render::renderer::renderer_types::Model::parse(d).unwrap();
            for s in &m.shaders {
                for st in &s.stages {
                    let d = t.get(&format!("{}.vtf", st.texture_name));
                    assert!(d.is_some_and(|d| visos_render::renderer::texture_manager::decode_any_vtf(d).is_ok()), "{name}: {}", st.texture_name);
                }
            }
        }
    }

    /// no police flash: white ambient (the OSDM prims show their textures untinted); a full red flash tints them red
    #[test]
    fn hud_lights_follow_the_police_flash() {
        let white = hud_lights(None, [0.0; 4]).light_fixed_function(None, Vec4::ONE);
        assert_eq!(white.truncate(), Vec3::ONE);
        let red = hud_lights(None, [1.0, 0.0, 0.0, 1.0]).light_fixed_function(None, Vec4::ONE);
        assert_eq!(red.truncate(), Vec3::X);
        let half = hud_lights(None, [0.0, 0.0, 1.0, 0.5]).ambient;
        assert_eq!(half.truncate(), Vec3::new(0.5, 0.5, 1.0));
    }

    #[test]
    fn tft_flips_open_and_closes() {
        let mut i = COSD3DInterface::new([[-11.7, 11.3, 1.0], [-0.1, 0.25, 0.0]], [[-10.75, -7.2, 1.0], [-0.35, 0.65, 0.1]]);
        let mut drop = DROP_MAX;
        i.render(&mut drop, 0.0, true, true);
        assert!(!i.out.tft_movie);
        for _ in 0..40 {
            i.render(&mut drop, 0.0, true, true);
        }
        assert_eq!(i.flip, 0.0);
        assert_eq!(drop, 0.0);
        assert!(i.out.tft_movie);
        for _ in 0..40 {
            i.render(&mut drop, 0.0, false, false);
        }
        assert_eq!(i.flip, FLIP_CLOSED);
        assert_eq!(drop, DROP_MAX);
        // the speedo at (12, -7, 0) lands right of centre, low
        let p = project([12.0, -7.0, 0.0]);
        assert!(p[0] > 400.0 && p[1] > 350.0, "{p:?}");
    }
}
