//! RainSurface.cpp (object `unk_726388`): the wet / London road layer. CommonInit 0x00136C50 builds the "roaddetail"
//! shader the road primitives are redrawn with (TrackFX 0x0014D490 -> 0x0014D6C0 over draw list `dword_6D5328`,
//! see `model_builder::is_road_detail_shader`), and the render 0x00137090 (GameManager 0x000B9010 view flag 2,
//! after the bloom and the motion blur) finds the reflection horizon and runs ProjectedLightBeam 0x00162400
//! (`projected_light_beam_pc_xbox::render_reflection`).
//!
//! API: in game the init is `tyrespray::TyreSprayInit { wet, track }` (core writes it at InGameInit, the same
//! VsAudio::IsTrackWet and race track name); `RaceEffectsShutdown` turns it off. The gate `PuddleInput.
//! rain_surface_off` (two players or camera mode 3) is core's. The player's car = the first `EffectCars` car.

use bevy::image::ImageAddressMode;
use bevy::prelude::*;

use crate::car_render::{CarRender, CarShadow};
use crate::full_screen_effects_manager_pc_xbox::{FseSet, FullScreenEffects};
use crate::puddle_render::{PuddleInput, PuddleRender};
use crate::race_effects::EffectCars;
use crate::renderer::shader_manager::{d3dcmp, StageState, XboxStageMaterial, XboxStageParams, STENCIL_ROAD_WRITE};
use crate::track_render::TrackCamera;

/// render layer of the road primitives' "roaddetail" copies (drawn only by `RoadCamera(ROAD_SCENE_DETAIL)`)
pub const ROAD_DETAIL_LAYER: usize = 21;
/// render layer of the puddle quads (drawn only by `RoadCamera(ROAD_SCENE_PUDDLES)`)
pub const PUDDLE_LAYER: usize = 22;

/// `dword_686FC4`: the horizon point's distance along the flattened camera direction
pub const HORIZON_DISTANCE: f32 = 10000.0;
/// `dword_686FC8`: smoothing of the car's ground slope per rendered (60 Hz) frame
pub const SLOPE_SMOOTHING: f32 = f32::from_bits(0x3F66_6666);
/// `byte_686FCC` (1): the slope is the car's ground direction y (CarRender +932) when no ray found one
pub const SLOPE_FROM_CAR: bool = true;
/// `byte_7266D8` (bss, never written): the two ground rays 50 / 100 m ahead of the car (`flt_686FD4`, ±10 m
/// `dword_686FD0`) — not ported
pub const SLOPE_RAYS: bool = false;
/// `flt_7263DC` (bss, never written): added to the projected horizon
pub const HORIZON_OFFSET: f32 = 0.0;
/// the original render rate the slope smoothing steps at (TV refresh, `car_render::XBOX_FRAME_RATE`)
const FRAME_RATE: f32 = 60.0;

/// the object `unk_726388` (in-game half of 0x00136C50)
#[derive(Resource, Clone, Debug)]
pub struct RainSurface {
    /// +0 "Brightnes" (128): the paste colour VISOS(0x80, b, b, b)
    pub brightness: u8,
    /// +4 "BlurStep", +8 / +12 "Reflection Image Width / Height" (log2; nothing in the render reads them)
    pub blur_step: u32,
    pub image_width: u32,
    pub image_height: u32,
    /// +16 draw list `dword_6D5330` ("_reflect" shaders, mask 128) held more than its sentinel this frame: not
    /// modelled (the one such shader, London's dustbin, only draws where +19 is set anyway)
    pub reflect_list: bool,
    /// +17 front end (the "FErefl" branch: `FeReflection` / `fe_reflection`)
    pub front_end: bool,
    /// +18 the track is wet, +19 on ("Graphics.Reflection" On/off: wet or "LONDON" in the track name)
    pub wet: bool,
    pub on: bool,
    /// +20 smoothed ground slope, +24 the reflection horizon (0..1 down the screen)
    pub slope: f32,
    pub horizon: f32,
    /// +44 "roaddetail": `\VFX\misc.vcd` texture (mtest4 wet, mtest4_dark dry London, else black)
    pub road_detail: Option<&'static str>,
    material: Option<Handle<XboxStageMaterial>>,
    /// the texture `material` holds
    loaded: Option<&'static str>,
}

impl Default for RainSurface {
    fn default() -> Self {
        Self {
            brightness: 128,
            blur_step: 32,
            image_width: 7,
            image_height: 6,
            reflect_list: false,
            front_end: false,
            wet: false,
            on: false,
            slope: 0.0,
            horizon: 0.0,
            road_detail: None,
            material: None,
            loaded: None,
        }
    }
}

/// the "roaddetail" stage (0x00136C50): reset 0x002A0AA0, texture, SetBlendMode(4) (alpha only), SetZState(1, 0)
/// test without write, 0x002A1760(20) depth bias, SetAddressWrap(1, 1), stencil func ALWAYS (0x002A10C0(0)), ref 8
/// (0x002A1140), pass REPLACE (0x002A12C0(0)), write mask 8 (0x002A1240) — stage 0 MODULATE(diffuse, texture)
pub fn road_detail_state() -> StageState {
    let mut s = StageState::from_commands(&[0]);
    s.blend_mode = 4;
    s.z_enable = true;
    s.z_func = d3dcmp::LESSEQUAL;
    s.z_write = false;
    s.depth_bias = 20;
    s.address_u = 1;
    s.address_v = 1;
    s.stencil_road = STENCIL_ROAD_WRITE;
    s
}

impl RainSurface {
    /// RainSurface.cpp 0x00136C50 (in game, `a2` = 0): the flags, the default tweak values and the roaddetail
    /// texture; `track` = the race's track name (+0x78; matched as core matches it, upper-cased)
    pub fn init(&mut self, wet: bool, track: &str) {
        self.front_end = false;
        self.brightness = 128;
        self.road_detail = None;
        self.wet = false;
        self.on = false;
        let london = track.to_ascii_uppercase().contains("LONDON");
        let mut tex = None;
        if wet {
            self.on = true;
            self.brightness = 128;
            tex = Some("mtest4");
            self.wet = true;
        } else if london {
            self.on = true;
            self.brightness = 128;
            tex = Some("mtest4_dark");
        }
        self.slope = 0.0;
        self.blur_step = 32;
        self.image_width = 7;
        self.image_height = 6;
        if tex.is_none() {
            tex = Some("black");
            self.image_width = 8;
            self.blur_step = 16;
        }
        self.road_detail = tex;
    }

    /// the roaddetail material the track's road copies share (made once, retextured by `init`)
    pub fn material(&mut self, materials: &mut Assets<XboxStageMaterial>, images: &mut Assets<Image>) -> Handle<XboxStageMaterial> {
        if let Some(h) = &self.material {
            return h.clone();
        }
        let state = road_detail_state();
        let h = materials.add(XboxStageMaterial { params: XboxStageParams::new(&state), texture: None, stencil: None, spec_texture: None, state });
        self.material = Some(h.clone());
        self.update_material(materials, images);
        h
    }

    fn update_material(&mut self, materials: &mut Assets<XboxStageMaterial>, images: &mut Assets<Image>) {
        let (Some(h), Some(name)) = (self.material.clone(), self.road_detail) else { return };
        if self.loaded == Some(name) {
            return;
        }
        let tex = visos::file::Vcd::open("\\VFX\\misc.vcd")
            .ok()
            .and_then(|v| v.get(&format!("{name}.vtf")).and_then(|d| crate::renderer::texture_manager::decode_any_vtf(d).ok()));
        let Some(tex) = tex else {
            log::warn!("roaddetail texture {name} not found");
            return;
        };
        let img = images.add(crate::model_builder::make_image(&tex, ImageAddressMode::Repeat, ImageAddressMode::Repeat));
        if let Some(mut m) = materials.get_mut(&h) {
            m.texture = Some(img);
        }
        self.loaded = Some(name);
    }

    /// RainSurface.cpp 0x00137090 — the horizon: the camera direction with its y replaced by the smoothed ground
    /// slope (x, z scaled by (1 − y²) / (x² + z²): not a normalisation, original) 10 km from the car, projected
    /// (0x0015EC80 y). `slope` = the car's CarRender +932 y (`None` = no car); `k` = smoothing for this frame.
    pub fn update_horizon(&mut self, cam_fwd: Vec3, car_pos: Vec3, slope: Option<f32>, k: f32, clip_from_world: &Mat4) {
        let mut v64 = -1.0f32;
        if SLOPE_FROM_CAR && v64 == -1.0 {
            if let Some(s) = slope {
                v64 = s;
            }
        }
        if v64 > -1.0 {
            self.slope = k * self.slope + (1.0 - k) * v64;
        }
        let mut d = cam_fwd;
        d.y = self.slope;
        let s = 1.0 - self.slope * self.slope;
        let f = s / (d.x * d.x + d.z * d.z);
        d.x *= f;
        d.z *= f;
        let p = car_pos + d * HORIZON_DISTANCE;
        let c = *clip_from_world * p.extend(1.0);
        let y = ((0.0 - c.y) / c.w + 1.0) / 2.0;
        self.horizon = (y + HORIZON_OFFSET).clamp(0.0, 1.0);
    }
}

fn rain_surface_init(
    mut init: MessageReader<crate::tyrespray::TyreSprayInit>,
    mut shutdown: MessageReader<crate::race_effects::RaceEffectsShutdown>,
    mut rs: ResMut<RainSurface>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if shutdown.read().count() > 0 {
        rs.on = false;
        rs.wet = false;
    }
    if let Some(m) = init.read().last() {
        rs.init(m.wet, &m.track);
        rs.update_material(&mut materials, &mut images);
        log::info!("rain surface init: wet {} on {} roaddetail {:?}", rs.wet, rs.on, rs.road_detail);
    }
}

/// GameManager 0x000B9010 -> RainSurface 0x00137090, in pass order after the motion blur / slipstream distortion
#[allow(clippy::too_many_arguments)]
fn rain_surface_system(
    mut fse: ResMut<FullScreenEffects>,
    mut rs: ResMut<RainSurface>,
    input: Option<Res<PuddleInput>>,
    puddles: Option<Res<PuddleRender>>,
    cars: Option<Res<EffectCars>>,
    roots: Query<(&CarRender, &Transform, Option<&CarShadow>)>,
    cam: Query<(&Camera, &GlobalTransform), With<TrackCamera>>,
    time: Res<Time>,
    ab: Option<Res<crate::gfx_ab::GfxAb>>,
    split: Option<Res<crate::view_render::SplitScreen>>,
) {
    rs.reflect_list = false;
    // 0x00137090: `byte_6F7157 == 0` (no wet road layer or puddles for two players)
    if split.is_some_and(|s| s.on()) {
        return;
    }
    // debug: `VELOCITY_RAIN_SURFACE=0` turns the render off (A/B shots); `VELOCITY_GFX_AB` per frame
    static OFF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *OFF.get_or_init(|| std::env::var("VELOCITY_RAIN_SURFACE").is_ok_and(|v| v == "0")) || ab.is_some_and(|a| a.rain_surface_off()) {
        return;
    }
    if !(rs.on || rs.reflect_list || rs.wet) || !fse.active {
        return;
    }
    if input.as_ref().is_some_and(|i| i.rain_surface_off) {
        return;
    }
    let Some((camera, gt)) = cam.iter().next() else { return };
    let clip_from_world = camera.clip_from_view() * gt.to_matrix().inverse();
    let player = cars.as_ref().and_then(|c| c.cars.first()).and_then(|c| roots.iter().find(|(r, _, _)| r.request == c.vehicle as u64));
    let (car_pos, slope) = match player {
        Some((_, t, s)) => (t.translation, s.map(|s| s.normal.y)),
        None => (gt.translation(), None),
    };
    let k = SLOPE_SMOOTHING.powf(time.delta_secs() * FRAME_RATE);
    rs.update_horizon(gt.forward().as_vec3(), car_pos, slope, k, &clip_from_world);    let puddles = puddles.is_some_and(|p| p.puddles_on && !p.puddles.is_empty());
    crate::projected_light_beam_pc_xbox::render_reflection(&mut fse, rs.horizon, rs.brightness, rs.road_detail.is_some(), puddles);
}

// ---- the front-end floor reflection ("FErefl", RainSurface 0x001378E0 / 0x00137B40 / 0x00137C50) ----

/// front-end cars while the reflection is split out: drawn by the reflection and the post camera
pub const FE_CAR_LAYER: usize = 13;
/// front-end characters and effects while split: drawn by the post camera only (after the paste)
pub const FE_POST_LAYER: usize = 14;
/// the paste strip and the alpha restore quad
pub const FE_PASTE_LAYER: usize = 15;
/// front-end elements with FECK2ElementCamera's reflect flag (+292 + i): also drawn into the reflection when the
/// foyer's "Reflect Environment" is set (the frontend puts its reflected background meshes on it, besides 0)
pub const FE_REFL_ENV_LAYER: usize = 16;
/// camera orders: the reflection before the foyer camera (0), the paste and the post pass after it
pub const FE_REFL_ORDER: isize = -5;
pub const FE_PASTE_ORDER: isize = 1;
pub const FE_POST_ORDER: isize = 2;

/// `dword_686FB4` / `dword_686FB8`: the "FErefl" target is 1 << 8 square (fmt 30, linear X8R8G8B8)
pub const FE_REFL_SIZE: u32 = 1 << 8;
/// `flt_686FDC`: the mirror plane above car 0's wheel bottom
pub const FE_REFL_PLANE_OFFSET: f32 = f32::from_bits(0x3CA3_D70A);
/// `byte_686FC0` (1): reflect the camera vectors (else the view matrix is multiplied)
pub const FE_REFL_CAMERA: bool = true;
/// `dword_686FE4` / `dword_686FE8`: the paste strip is full strength down to 0.6 of the screen, black at 0.85
pub const FE_PASTE_FULL: f32 = f32::from_bits(0x3F19_999A);
pub const FE_PASTE_FADE: f32 = f32::from_bits(0x3F59_999A);
/// `dword_686FEC`: how much a downward camera dims the paste
pub const FE_PASTE_DIR_K: f32 = 1.0;
/// `dword_686FE0` = 3: the paste shader (+48 + 4·i; blend 0, 0 (+0x002A17D0(1, 0)), 6, 13)
pub const FE_PASTE_SHADER: usize = 3;
pub const FE_PASTE_BLENDS: [u32; 4] = [0, 0, 6, 13];

/// the front end's 3d camera (FECK2ElementCamera's view); the frontend inserts it on its foyer camera
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct FrontEndCamera;

/// front-end render switches (frontend writes them)
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct FrontEndRender {
    /// `dword_7065A4` != 0: the garage customisation mode — no floor reflection, no flares (0x002999B0)
    pub garage: bool,
    /// `byte_7065C0` (CarPalette 0x0010CE80): no front-end flares
    pub flares_off: bool,
}

/// RainSurface +68.. (the front-end half; `byte_7263CC`..): FECK2Element3D::LoadFoyerLights 0x00298CD0 sets it from
/// the foyer's Lights.txt ("REFLECTION" block after the lights and the optional fog: +69 Reflect Environment != 0,
/// +72 Blur (`_ftol`, read by nothing), +76 Intensity), else +68 = 0. Tweaks "Frontend.Reflection" 0x00137610.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct FeReflection {
    pub on: bool,
    pub environment: bool,
    pub blur: i32,
    pub intensity: f32,
}

impl FeReflection {
    /// the "REFLECTION" block of a foyer Lights.txt (the first three numbers after the token); none = off
    pub fn from_lights_txt(text: &str) -> Self {
        let Some(at) = text.find("\"REFLECTION\"") else { return Self::default() };
        let mut nums = text[at + 12..]
            .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .filter(|s| !s.is_empty() && s.parse::<f32>().is_ok())
            .map(|s| s.parse::<f32>().unwrap_or(0.0));
        let env = nums.next().unwrap_or(0.0);
        let blur = nums.next().unwrap_or(0.0);
        let intensity = nums.next().unwrap_or(0.0);
        Self { on: true, environment: env != 0.0, blur: blur as i32, intensity }
    }
}

/// the floor reflection is drawn this frame: front-end cars, characters and effects move to their own layers
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct FeReflectionSplit(pub bool);

/// a perspective projection that keeps the foyer camera's aspect on the square "FErefl" target (the original renders
/// the 4:3 view into 256², the paste stretches it back)
#[derive(Clone, Debug)]
struct FixedPerspective(PerspectiveProjection);

impl bevy::camera::CameraProjection for FixedPerspective {
    fn get_clip_from_view(&self) -> Mat4 {
        self.0.get_clip_from_view()
    }
    fn get_clip_from_view_for_sub(&self, sub: &bevy::camera::SubCameraView) -> Mat4 {
        self.0.get_clip_from_view_for_sub(sub)
    }
    fn update(&mut self, _width: f32, _height: f32) {}
    fn far(&self) -> f32 {
        self.0.far()
    }
    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [bevy::math::Vec3A; 8] {
        self.0.get_frustum_corners(z_near, z_far)
    }
}

#[derive(Component)]
struct FeReflectionView;

#[derive(Resource, Default)]
struct FeReflectionCameras {
    image: Option<Handle<Image>>,
    refl: Option<Entity>,
    paste: Option<Entity>,
    post: Option<Entity>,
    strip: Option<(Entity, Handle<Mesh>)>,
    restore: Option<Entity>,
    /// the projection last copied (fov, aspect, near, far)
    proj: [f32; 4],
}

/// RainSurface.cpp 0x001378E0 mirror: reflection about the plane y = `plane` (`Renderer 0x002ABDE0` / `0x002ABF70` on
/// the camera position and vectors, then VsBaseCamera 0x0005D2E0 rebuilds the view from them — a mirrored view).
/// Bevy keeps back-face culling for the view's handedness, so the mirrored view is taken with x flipped in view
/// space (a proper rotation; the paste flips u back): world_from_view = R · W · S_x.
pub fn fe_reflection_transform(camera: &Mat4, plane: f32) -> Mat4 {
    let r = Mat4::from_cols(Vec4::X, Vec4::NEG_Y, Vec4::Z, Vec4::new(0.0, 2.0 * plane, 0.0, 1.0));
    r * *camera * Mat4::from_scale(Vec3::new(-1.0, 1.0, 1.0))
}

/// RainSurface.cpp 0x00137C50: the 6-vertex paste strip in the 512² space — (0,0)..(512, 512·0.6) at VISOS(0x80, c, c,
/// c), then to (512, 512·0.85) at black; uv = the same fractions of the target (texels of the linear 256² target),
/// u mirrored for `fe_reflection_transform`'s flip. c = `_ftol`((1 − clamp(−dir.y, 0, 1))·k + (1 − k)) · +76) as a
/// byte. Returns (512-space position, uv, D3DCOLOR).
pub fn fe_paste_strip(cam_dir_y: f32, intensity: f32) -> [(Vec2, Vec2, u32); 6] {
    let down = (0.0 - cam_dir_y).clamp(0.0, 1.0);
    let k = (1.0 - down) * FE_PASTE_DIR_K + (1.0 - FE_PASTE_DIR_K);
    let c = ((k * intensity) as i32 & 0xFF) as u8;
    let lit = crate::light_streak::visos_colour([0x80, c, c, c]);
    let black = crate::light_streak::visos_colour([0x80, 0, 0, 0]);
    let row = |y: f32, col: u32| [(Vec2::new(0.0, 512.0 * y), Vec2::new(1.0, y), col), (Vec2::new(512.0, 512.0 * y), Vec2::new(0.0, y), col)];
    let [a, b] = row(0.0, lit);
    let [c2, d] = row(FE_PASTE_FULL, lit);
    let [e, f] = row(FE_PASTE_FADE, black);
    [a, b, c2, d, e, f]
}

pub(crate) fn argb_vec4(c: u32) -> [f32; 4] {
    let b = |s: u32| ((c >> s) & 0xFF) as f32 / 255.0;
    [b(16), b(8), b(0), b(24)]
}

/// a 512²-space strip as a mesh for the paste camera (orthographic, 512 wide, y up, looking down −Z) at depth `z`
pub(crate) fn screen_mesh(verts: &[(Vec2, Vec2, u32)], z: f32) -> Mesh {
    let pos: Vec<[f32; 3]> = verts.iter().map(|(p, _, _)| [p.x - 256.0, 256.0 - p.y, z]).collect();
    let uv: Vec<[f32; 2]> = verts.iter().map(|(_, t, _)| t.to_array()).collect();
    let col: Vec<[f32; 4]> = verts.iter().map(|(_, _, c)| argb_vec4(*c)).collect();
    let mut idx = Vec::new();
    for i in 0..verts.len().saturating_sub(2) as u32 {
        if i % 2 == 0 {
            idx.extend([i, i + 1, i + 2]);
        } else {
            idx.extend([i + 1, i, i + 2]);
        }
    }
    Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col)
        .with_inserted_indices(bevy::mesh::Indices::U32(idx))
}

fn paste_state(blend: u32) -> StageState {
    let mut s = StageState::from_commands(&[0]);
    s.blend_mode = blend;
    s.z_enable = false;
    s.z_write = false;
    s.two_sided = true;
    s.address_u = 3;
    s.address_v = 3;
    s
}

type MeshLayerQuery<'w, 's> = Query<'w, 's, Option<&'static bevy::camera::visibility::RenderLayers>, (With<Mesh3d>, Without<FeReflectionView>)>;

/// the front-end entities that move off the foyer camera while split: cars (`FE_CAR_LAYER`), characters
/// (`FE_POST_LAYER`); only meshes without a layer of their own are touched
fn fe_layer_entities(
    commands: &mut Commands,
    split: bool,
    cars: &Query<(Entity, &CarRender)>,
    chars: &Query<(Entity, &crate::skinned_object_render::SkinnedModelRender)>,
    children: &Query<&Children>,
    meshes: &MeshLayerQuery,
) {
    use bevy::camera::visibility::RenderLayers;
    let mut apply = |root: Entity, layer: usize| {
        let want = if split { RenderLayers::layer(layer) } else { RenderLayers::default() };
        for e in children.iter_descendants(root) {
            let Ok(cur) = meshes.get(e) else { continue };
            let is = cur.cloned().unwrap_or_default();
            let ours = is == RenderLayers::default() || is == RenderLayers::layer(layer);
            if ours && is != want {
                commands.entity(e).insert(want.clone());
            }
        }
    };
    for (root, r) in cars.iter() {
        if r.spec.frontend {
            apply(root, FE_CAR_LAYER);
        }
    }
    for (root, m) in chars.iter() {
        if m.pack == crate::skinned_object_render::CharPack::Chars {
            apply(root, FE_POST_LAYER);
        }
    }
}

/// FECK2ElementCamera.cpp 0x002999B0's reflection half. When the foyer's reflection is on (`FeReflection`), not in
/// the garage mode and a front-end car exists: RainSurface 0x001378E0 renders the front-end cars (CarRender 0x001190D0)
/// — plus the reflect-flagged elements when "Reflect Environment" — into "FErefl" from the camera mirrored about
/// y = car 0's y − +1816/2 + 0.02, cleared to colour 0; 0x00137B40 restores the camera; after the foyer's own elements
/// 0x00137C50 adds the strip (blend 13: rgb += texture · colour · dest alpha) before the characters, cars, shadows,
/// flares and particles. Here: the foyer camera (`FrontEndCamera`, marked `FE_ALPHA_VIEW` so its draws write the
/// alpha the paste reads) draws the rest, the paste camera adds the strip and puts alpha back to 1, the post camera
/// draws `FE_CAR_LAYER` + `FE_POST_LAYER` over it (shared colour and depth).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn fe_reflection(
    mut commands: Commands,
    refl: Option<Res<FeReflection>>,
    fe: Res<FrontEndRender>,
    mut split: ResMut<FeReflectionSplit>,
    mut st: ResMut<FeReflectionCameras>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    foyer: Query<
        (Entity, &Transform, &Projection, &Msaa, Option<&bevy::camera::RenderTarget>, Option<&bevy::core_pipeline::tonemapping::Tonemapping>),
        (With<FrontEndCamera>, Without<FeReflectionView>),
    >,
    mut views: Query<(&mut Camera, &mut Transform, &mut Projection, &mut Msaa, Option<&bevy::camera::RenderTarget>, &mut bevy::camera::visibility::RenderLayers), With<FeReflectionView>>,
    cars: Query<(Entity, &CarRender)>,
    shadows: Query<(&CarRender, &Transform, &CarShadow), Without<FeReflectionView>>,
    chars: Query<(Entity, &crate::skinned_object_render::SkinnedModelRender)>,
    children: Query<&Children>,
    mesh_layers: MeshLayerQuery,
    mut mesh3d: Query<&mut Mesh3d>,
) {
    use bevy::camera::visibility::RenderLayers;
    use bevy::camera::{Camera3dDepthLoadOp, CameraOutputMode, RenderTarget};
    use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
    let r = refl.map(|r| *r).unwrap_or_default();
    let cam = foyer.iter().next();
    // CarDamage 0x000761B0(…, 0): car 0 (the lowest request) of the front end
    let car0 = shadows.iter().filter(|(c, _, _)| c.spec.frontend).min_by_key(|(c, _, _)| c.request);
    let plane = car0.map(|(_, t, s)| t.translation.y - s.height / 2.0 + FE_REFL_PLANE_OFFSET);
    let on = r.on && !fe.garage && cam.is_some() && plane.is_some();
    if split.0 != on {
        split.0 = on;
    }
    fe_layer_entities(&mut commands, on, &cars, &chars, &children, &mesh_layers);
    if let Some((e, _, _, _, _, tm)) = cam {
        let want = if on { crate::renderer::shader_manager::FE_ALPHA_VIEW } else { Tonemapping::default() };
        if tm != Some(&want) {
            commands.entity(e).insert(want);
        }
    }
    for e in [st.refl, st.paste, st.post].into_iter().flatten() {
        if let Ok((mut c, ..)) = views.get_mut(e) {
            if c.is_active != on {
                c.is_active = on;
            }
        }
    }
    let (Some((_, ct, cproj, msaa, target, _)), Some(plane), true) = (cam, plane, on) else { return };
    let Projection::Perspective(pp) = cproj else { return };
    let image = st
        .image
        .get_or_insert_with(|| {
            let mut img = Image::new_target_texture(FE_REFL_SIZE, FE_REFL_SIZE, bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, None);
            img.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor::linear());
            images.add(img)
        })
        .clone();
    let target = target.cloned().unwrap_or_default();
    let replace = || CameraOutputMode::Write { blend_state: Some(bevy::render::render_resource::BlendState::REPLACE), clear_color: ClearColorConfig::None };
    // the three cameras
    let mirrored = Transform::from_matrix(fe_reflection_transform(&ct.to_matrix(), plane));
    let refl_layers = if r.environment { RenderLayers::from_layers(&[FE_CAR_LAYER, FE_REFL_ENV_LAYER]) } else { RenderLayers::layer(FE_CAR_LAYER) };
    let post_layers = RenderLayers::from_layers(&[FE_CAR_LAYER, FE_POST_LAYER]);
    let ortho = || {
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed { width: 512.0, height: 512.0 },
            ..OrthographicProjection::default_3d()
        })
    };
    if st.refl.is_none() {
        st.refl = Some(
            commands
                .spawn((
                    Camera3d::default(),
                    Camera { order: FE_REFL_ORDER, clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)), ..default() },
                    RenderTarget::Image(image.clone().into()),
                    Projection::custom(FixedPerspective(pp.clone())),
                    Msaa::Off,
                    Tonemapping::None,
                    DebandDither::Disabled,
                    mirrored,
                    refl_layers.clone(),
                    FeReflectionView,
                ))
                .id(),
        );
        st.proj = [pp.fov, pp.aspect_ratio, pp.near, pp.far];
    }
    if st.paste.is_none() {
        st.paste = Some(
            commands
                .spawn((
                    Camera3d { depth_load_op: Camera3dDepthLoadOp::Load, ..default() },
                    Camera { order: FE_PASTE_ORDER, clear_color: ClearColorConfig::None, output_mode: replace(), ..default() },
                    target.clone(),
                    ortho(),
                    *msaa,
                    Tonemapping::None,
                    DebandDither::Disabled,
                    Transform::default(),
                    RenderLayers::layer(FE_PASTE_LAYER),
                    FeReflectionView,
                ))
                .id(),
        );
    }
    if st.post.is_none() {
        st.post = Some(
            commands
                .spawn((
                    Camera3d { depth_load_op: Camera3dDepthLoadOp::Load, ..default() },
                    Camera { order: FE_POST_ORDER, clear_color: ClearColorConfig::None, output_mode: replace(), ..default() },
                    target.clone(),
                    cproj.clone(),
                    *msaa,
                    Tonemapping::None,
                    DebandDither::Disabled,
                    *ct,
                    post_layers.clone(),
                    FeReflectionView,
                ))
                .id(),
        );
    }
    // per frame: poses, projections, targets, msaa
    let same_target = |t: Option<&RenderTarget>| t.is_some_and(|t| format!("{t:?}") == format!("{target:?}"));
    if let Some(Ok((_, mut t, mut p, _, _, mut l))) = st.refl.map(|e| views.get_mut(e)) {
        *t = mirrored;
        let key = [pp.fov, pp.aspect_ratio, pp.near, pp.far];
        if st.proj != key {
            *p = Projection::custom(FixedPerspective(pp.clone()));
            st.proj = key;
        }
        if *l != refl_layers {
            *l = refl_layers;
        }
    }
    for (e, own_proj) in [(st.paste, false), (st.post, true)] {
        let Some(Ok((_, mut t, mut p, mut m, tg, _))) = e.map(|e| views.get_mut(e)) else { continue };
        if own_proj {
            *t = *ct;
            if let Projection::Perspective(q) = &*p {
                if q.fov != pp.fov || q.aspect_ratio != pp.aspect_ratio || q.near != pp.near || q.far != pp.far {
                    *p = cproj.clone();
                }
            }
        }
        if *m != *msaa {
            *m = *msaa;
        }
        if !same_target(tg) {
            commands.entity(e.unwrap()).insert(target.clone());
        }
    }
    // the paste strip and the alpha restore (blend 14 adds (0, 0, 0, 1): alpha saturates to 1 for the 2d cameras)
    let strip = fe_paste_strip(ct.forward().y, r.intensity);
    let mesh = screen_mesh(&strip, -2.0);
    match &st.strip {
        Some((e, h)) => {
            if crate::set_mesh(&mut meshes, h, mesh) {
                if let Ok(mut m) = mesh3d.get_mut(*e) {
                    m.set_changed();
                }
            }
        }
        None => {
            let state = paste_state(FE_PASTE_BLENDS[FE_PASTE_SHADER]);
            let mut params = XboxStageParams::new(&state);
            params.arg.w = 2.0;
            let mat = materials.add(XboxStageMaterial { params, texture: Some(image.clone()), stencil: None, spec_texture: None, state });
            let h = meshes.add(mesh);
            let e = commands
                .spawn((Mesh3d(h.clone()), MeshMaterial3d(mat), Transform::default(), Visibility::Visible, bevy::camera::visibility::NoFrustumCulling, RenderLayers::layer(FE_PASTE_LAYER)))
                .id();
            st.strip = Some((e, h));
        }
    }
    if st.restore.is_none() {
        let black = crate::light_streak::visos_colour([0x80, 0, 0, 0]);
        let quad = [
            (Vec2::ZERO, Vec2::ZERO, black),
            (Vec2::new(512.0, 0.0), Vec2::X, black),
            (Vec2::new(0.0, 512.0), Vec2::Y, black),
            (Vec2::splat(512.0), Vec2::ONE, black),
        ];
        let state = paste_state(14);
        let mat = materials.add(XboxStageMaterial { params: XboxStageParams::new(&state), texture: None, stencil: None, spec_texture: None, state });
        let e = commands
            .spawn((
                Mesh3d(meshes.add(screen_mesh(&quad, -1.0))),
                MeshMaterial3d(mat),
                Transform::default(),
                Visibility::Visible,
                bevy::camera::visibility::NoFrustumCulling,
                RenderLayers::layer(FE_PASTE_LAYER),
            ))
            .id();
        st.restore = Some(e);
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RainSurfaceSet;

pub struct RainSurfacePlugin;

impl Plugin for RainSurfacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FeReflectionSplit>()
            .init_resource::<FeReflectionCameras>()
            .init_resource::<FrontEndRender>()
            .add_systems(
                PostUpdate,
                fe_reflection.before(crate::effect_draw::EffectSet::Flush).before(bevy::transform::TransformSystems::Propagate),
            );
        app.init_resource::<RainSurface>().add_systems(Update, rain_surface_init).add_systems(
            PostUpdate,
            rain_surface_system
                .in_set(FseSet::Build)
                .in_set(RainSurfaceSet)
                .after(crate::motion_blur_pc_xbox::MotionBlurSet)
                .before(crate::danger_zone_render::DofSet),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0x00136C50: wet -> mtest4, dry London -> mtest4_dark (on), anything else -> black (off)
    #[test]
    fn init_flags() {
        let mut r = RainSurface::default();
        r.init(true, "LA/LADown");
        assert_eq!((r.on, r.wet, r.road_detail, r.blur_step, r.image_width), (true, true, Some("mtest4"), 32, 7));
        r.init(false, "London/London");
        assert_eq!((r.on, r.wet, r.road_detail), (true, false, Some("mtest4_dark")));
        r.init(false, "LA/LACoast");
        assert_eq!((r.on, r.wet, r.road_detail, r.blur_step, r.image_width), (false, false, Some("black"), 16, 8));
        let s = road_detail_state();
        assert_eq!((s.blend_mode, s.z_write, s.depth_bias, s.stencil_road), (4, false, 20, STENCIL_ROAD_WRITE));
    }

    /// LoadFoyerLights 0x00298CD0's REFLECTION block, as the shipped carSel / modArea lights files carry it
    #[test]
    fn fe_reflection_block() {
        let t = "\"FOG\"\n(1,2,3)\n\"REFLECTION\"\nReflect Environment : \t(1)\nBlur\t\t\t\t:\t(16) \nIntensity\t\t\t:\t(255)\n";
        assert_eq!(FeReflection::from_lights_txt(t), FeReflection { on: true, environment: true, blur: 16, intensity: 255.0 });
        assert!(!FeReflection::from_lights_txt("\"FOG\"\n(1,2,3)\n").on);
        for (foyer, env, intensity) in [("carSel", false, 64.0), ("modArea", true, 255.0)] {
            let Ok(s) = std::fs::read_to_string(visos::file::resolve(&format!("\\Foyers\\{foyer}\\lights.txt"))) else { continue };
            let r = FeReflection::from_lights_txt(&s);
            assert_eq!((r.on, r.environment, r.intensity), (true, env, intensity), "{foyer}");
        }
    }

    /// the mirror keeps a point on the plane fixed and is a proper rotation after the view-space x flip; the paste
    /// strip dims with a downward camera and mirrors u
    #[test]
    fn fe_reflection_geometry() {
        let cam = Mat4::from_rotation_translation(Quat::from_rotation_x(-0.3), Vec3::new(1.0, 2.0, 5.0));
        let m = fe_reflection_transform(&cam, 0.5);
        assert!((m.determinant() - 1.0).abs() < 1e-5);
        let p = m.transform_point3(Vec3::ZERO);
        assert!((p - Vec3::new(1.0, -1.0, 5.0)).length() < 1e-5, "{p}");
        let s = fe_paste_strip(0.0, 64.0);
        assert_eq!((s[0].1, s[1].1), (Vec2::new(1.0, 0.0), Vec2::new(0.0, 0.0)));
        assert_eq!(s[0].2, 0xFF80_8080);
        assert_eq!(s[4].2, 0xFF00_0000);
        assert!((s[3].0.y - 512.0 * FE_PASTE_FULL).abs() < 1e-3);
        // looking half down: c = _ftol(0.5 · 64) = 32 → doubled 0x40
        assert_eq!(fe_paste_strip(-0.5, 64.0)[0].2, 0xFF40_4040);
    }

    /// a level camera looking down -Z: the horizon sits at the screen centre; a car pointing uphill raises it
    #[test]
    fn horizon_projection() {
        let proj = Mat4::perspective_infinite_reverse_rh(0.8185, 4.0 / 3.0, 0.5);
        let view = Mat4::look_to_rh(Vec3::new(0.0, 2.0, 0.0), Vec3::NEG_Z, Vec3::Y);
        let cfw = proj * view;
        let mut r = RainSurface::default();
        r.update_horizon(Vec3::NEG_Z, Vec3::ZERO, Some(0.0), 0.0, &cfw);
        assert!((r.horizon - 0.5).abs() < 1e-3, "{}", r.horizon);
        r.update_horizon(Vec3::NEG_Z, Vec3::ZERO, Some(0.1), 0.0, &cfw);
        assert!(r.horizon < 0.5);
        // smoothing: k = 0.9 moves a tenth of the way
        r.slope = 0.0;
        r.update_horizon(Vec3::NEG_Z, Vec3::ZERO, Some(0.1), SLOPE_SMOOTHING, &cfw);
        assert!((r.slope - 0.01).abs() < 1e-6, "{}", r.slope);
        // no car: the slope stays
        r.update_horizon(Vec3::NEG_Z, Vec3::ZERO, None, 0.0, &cfw);
        assert!((r.slope - 0.01).abs() < 1e-6);
    }
}
