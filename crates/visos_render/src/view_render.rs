//! Render side of GameManager.cpp's game views (the two-player split screen). Views (GameManager +588, 40 B: +0 x, +4
//! y, +8 w, +12 h over the 640x480 screen, +24 render flags, +28 player = camera index, +32 kind, +36 active) are set up
//! by 0x000B3EC0 / 0x000B3960 (core: `velocity_game::game_mode::game_views`); 0x000B9010 draws every active view in
//! turn: 0x000B3C30 sets the view's viewport on the back buffer (RenderTargetManager 0x002EEF10 stores it with the
//! target, 0x002EEE20 re-applies it whenever the target is set again) and its camera, then the whole world pass and
//! the view's full-screen effects run inside that viewport. Two players: view 1 = top half (player 0), view 2 = bottom
//! half (player 1), view 0 (full screen) inactive; `byte_6F9180` (side by side) is only ever written 0.
//!
//! Projection: VsBaseCamera 0x0005D1D0 builds the frustum from the fov and the aspect +144, which VsCameraManager
//! 0x0005F8F0 sets to 4/3 (16/9 widescreen) whatever the viewport — so each half shows the full-screen projection
//! squashed into 640x240 (the projection, VsRender 0x000C89F0, never reads the viewport). Two players also halve the
//! cameras' far plane (0x0005ECC0: 600 · 0.5, 200 · 0.5), which only the camera planes (culling) use.
//!
//! Here: `TrackCamera` stays the player-0 view (every single-view effect keeps using it); a world camera for player
//! p >= 1 carries `TrackView { player: p, .. }` and gets `SecondaryTrackCamera` (graphics strips `TrackCamera` from
//! it). Graphics sets each view camera's viewport from `TrackView.rect`, its render target / msaa / output / glow view
//! from the player-0 camera, copies the sky passes for it, and keeps the full-screen aspect (`fix_view_aspect`).

use bevy::camera::primitives::Frustum;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{CameraOutputMode, RenderTarget, Viewport};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;

use crate::renderer::shader_manager::XboxStageMaterial;
use crate::sky_render::{SkyBackground, SkyCamera, BACKGROUND_SPAN, SKY_PASS_BACKGROUND};
use crate::track_render::{TrackCamera, WORLD_CAMERA_ORDER};

/// one player's world view (GameManager view +28 player, +0..+12 rect as a fraction of the screen: x, y, w, h)
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct TrackView {
    pub player: usize,
    pub rect: Vec4,
}

/// the world camera of a view for player >= 1 (player 0's is the `TrackCamera`)
#[derive(Component)]
pub struct SecondaryTrackCamera;

/// a sky pass camera copied for a later player's view (the original draws the sky inside each view); `source` = the
/// player-0 sky camera it copies
#[derive(Component, Clone, Copy, Debug)]
pub struct ViewSky {
    pub player: usize,
    pub source: Entity,
}

/// the background quad of a copied sky (`CSkyRender::RenderBackground` 0x0013CF40 per view)
#[derive(Component)]
pub struct ViewSkyBackground(pub Handle<XboxStageMaterial>);

/// the player-0 background camera's own clear, kept while player 1's sky clears instead
#[derive(Component)]
pub struct SavedClear(pub ClearColorConfig);

/// the rear-view mirror view (GameManager view kind 1, one player only): a world camera that core places from the
/// camera bridge's mirror record ([1], `GameBridge::camera_view(p, lerp)[1]`). `rect` = the view's fraction of the
/// screen (0x000B3960 kind 1: x W/3, y 0, w W/3, h H/6); `on` = 0x000B88B0's +36 for the view (`byte_685808`, review
/// cameras != 2, the player's camera mode 1 with +524 == 0 and not looking back +585)
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct MirrorView {
    pub player: usize,
    pub rect: Vec4,
    pub on: bool,
}

/// `ViewSky.player` of the mirror's sky copies
pub const MIRROR_SKY: usize = 99;

/// this frame's world views, by player (`byte_6F7157` = more than one); `mirror` = the mirror view's rect while it
/// is drawn (never together with a split)
#[derive(Resource, Clone, Debug, Default)]
pub struct SplitScreen {
    pub views: Vec<(usize, Vec4)>,
    pub mirror: Option<Vec4>,
}

impl SplitScreen {
    /// two players (`byte_6F7157`)
    pub fn on(&self) -> bool {
        self.views.len() > 1
    }

    /// more than one view this frame (two players, or one player with the mirror): per-view layers in use
    pub fn per_view(&self) -> bool {
        self.on() || self.mirror.is_some()
    }

    /// player `p`'s view rect (the whole screen without a view)
    pub fn rect(&self, p: usize) -> [f32; 4] {
        self.views.iter().find(|(q, _)| *q == p).map_or(FULL, |(_, r)| r.to_array())
    }
}

const FULL: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

/// one world view as the camera-built effects see it (GameManager 0x000B9010's view loop: view camera `v77[60]`)
#[derive(Clone, Debug)]
pub struct ViewCam {
    pub player: usize,
    pub camera: Entity,
    pub transform: GlobalTransform,
    pub projection: Option<Projection>,
    pub frustum: Option<Frustum>,
    /// the view rect (fraction of the screen; the whole screen without a split)
    pub rect: [f32; 4],
    /// `EffectDraw::view` for this view's batches (None with one view: everything stays on layer 0)
    pub tag: Option<usize>,
}

type ViewCamItem = (Entity, &'static GlobalTransform, Option<&'static Projection>, Option<&'static Frustum>);

/// the world views in the original's loop order (player 0's view first, player 1's only while split). The mirror
/// (flags 0x08000620: no flares, particles, rain, zone FX or lens spray) is not listed, but while it is drawn
/// player 0's camera-built batches are tagged so the mirror does not see them; the few camera-built producers the
/// mirror runs (car shadows, barriers, start lights) draw once more with `mirror()`.
#[derive(bevy::ecs::system::SystemParam)]
pub struct ViewCams<'w, 's> {
    split: Res<'w, SplitScreen>,
    main: Query<'w, 's, ViewCamItem, (With<TrackCamera>, Without<SecondaryTrackCamera>)>,
    second: Query<'w, 's, (ViewCamItem, &'static TrackView), With<SecondaryTrackCamera>>,
    mirror_cam: Query<'w, 's, (ViewCamItem, &'static MirrorView), (Without<TrackCamera>, Without<SecondaryTrackCamera>)>,
}

/// `EffectDraw::view` tag of the mirror's own batches (`view_layer(1)` = 18: the mirror camera only)
pub const MIRROR_VIEW_TAG: usize = 1;

impl ViewCams<'_, '_> {
    pub fn on(&self) -> bool {
        self.split.on()
    }

    /// the mirror view while it is drawn (GameManager view kind 1, player 0, camera mode 2), tagged `MIRROR_VIEW_TAG`
    pub fn mirror(&self) -> Option<ViewCam> {
        let rect = self.split.mirror?;
        let ((e, t, p, f), _) = self.mirror_cam.iter().find(|(_, m)| m.on)?;
        Some(ViewCam { player: 0, camera: e, transform: *t, projection: p.cloned(), frustum: f.cloned(), rect: rect.to_array(), tag: Some(MIRROR_VIEW_TAG) })
    }

    pub fn list(&self) -> Vec<ViewCam> {
        let on = self.split.on();
        let tagged = self.split.per_view();
        let mut out = Vec::new();
        if let Some((e, t, p, f)) = self.main.iter().next() {
            let rect = if on { self.split.rect(0) } else { FULL };
            out.push(ViewCam { player: 0, camera: e, transform: *t, projection: p.cloned(), frustum: f.cloned(), rect, tag: tagged.then_some(0) });
        }
        if on && !out.is_empty() {
            let mut sec: Vec<_> = self.second.iter().collect();
            sec.sort_by_key(|(_, v)| v.player);
            for ((e, t, p, f), v) in sec {
                let rect = self.split.rect(v.player);
                out.push(ViewCam { player: v.player, camera: e, transform: *t, projection: p.cloned(), frustum: f.cloned(), rect, tag: Some(v.player) });
            }
        }
        out
    }

    /// the views' rects in loop order (the whole screen once without a camera: screen-only effects still run)
    pub fn rects(&self) -> Vec<[f32; 4]> {
        let r: Vec<_> = self.list().iter().map(|v| v.rect).collect();
        if r.is_empty() {
            vec![FULL]
        } else {
            r
        }
    }

    /// views drawn this frame (1 without a camera too: the per-view updates still tick once)
    pub fn count(&self) -> usize {
        self.list().len().max(1)
    }
}

/// the layer of a view-tagged effect batch while split (`VIEW_CAR_LAYERS`: player p's camera sees 17 + p)
pub fn view_layer(player: usize) -> usize {
    VIEW_CAR_LAYERS[player.min(1)]
}

/// render layers seen by one view only: the cars' per-view LOD groups (`car_render::update_car_lights`)
pub const VIEW_CAR_LAYERS: [usize; 2] = [17, 18];
/// the background quad of player 1's sky copy
pub const VIEW_SKY_BACKGROUND_LAYER: usize = 19;
/// player 1's view renders first: its sky passes at -70..-67 (the background clears the shared target), its world at
/// -66; player 0's sky (-44..-41, no clear while split) and world (-40) follow, then the FSE passes
pub const SECOND_VIEW_ORDER: isize = -70;
/// VsCameraManager 0x0005ECC0: `byte_6F7157` scales both cameras' far planes by 0.5 (`dword_6850A0` 600, `6850A8` 200)
pub const SPLIT_FAR_SCALE: f32 = 0.5;

/// GameManager 0x000B3C30 / RenderTargetManager 0x002EEF10: a view rect (fraction) on a target of `size` pixels
pub fn viewport_of(rect: Vec4, size: UVec2) -> Viewport {
    let s = size.as_vec2();
    let pos = (Vec2::new(rect.x, rect.y) * s).round().as_uvec2();
    let sz = (Vec2::new(rect.z, rect.w) * s).round().as_uvec2().max(UVec2::ONE);
    Viewport { physical_position: pos, physical_size: sz, ..default() }
}

fn same_viewport(a: &Option<Viewport>, b: &Option<Viewport>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a.physical_position == b.physical_position && a.physical_size == b.physical_size,
        _ => false,
    }
}

fn same_debug<T: std::fmt::Debug>(a: &T, b: &T) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

/// collect the views; a camera for player >= 1 leaves the `TrackCamera` role to player 0's
#[allow(clippy::type_complexity)]
pub fn update_views(
    mut commands: Commands,
    mut split: ResMut<SplitScreen>,
    cams: Query<(Entity, &TrackView, Has<TrackCamera>, Has<SecondaryTrackCamera>)>,
    mirrors: Query<&MirrorView>,
) {
    let mirror_cam = mirrors.iter().find(|m| m.on).map(|m| m.rect);
    let mut views: Vec<(usize, Vec4)> = Vec::new();
    for (e, v, track, second) in &cams {
        if v.player > 0 {
            if track {
                commands.entity(e).remove::<TrackCamera>();
            }
            if !second {
                commands.entity(e).insert(SecondaryTrackCamera);
            }
        } else if second {
            commands.entity(e).remove::<SecondaryTrackCamera>();
        }
        if !views.iter().any(|(p, _)| *p == v.player) {
            views.push((v.player, v.rect));
        }
    }
    views.sort_by_key(|(p, _)| *p);
    // the mirror view exists for one player only (0x000B3EC0)
    let mirror = mirror_cam.filter(|_| views.len() <= 1);
    if split.views != views || split.mirror != mirror {
        split.views = views;
        split.mirror = mirror;
    }
}

/// the player-0 camera's target, msaa, output, glow view and layers onto every other view's camera; viewports from
/// the view rects (`TrackCamera` too while split); player 1 draws first and clears (see `SECOND_VIEW_ORDER`)
#[allow(clippy::type_complexity)]
pub fn setup_view_cameras(
    mut commands: Commands,
    split: Res<SplitScreen>,
    clear_colour: Res<ClearColor>,
    sky: Query<(), (With<SkyCamera>, Without<ViewSky>)>,
    mut main: Query<
        (Entity, &mut Camera, Option<&TrackView>, Option<&RenderTarget>, Option<&Msaa>, Option<&RenderLayers>, Option<&bevy::light::ShadowFilteringMethod>),
        (With<TrackCamera>, Without<SecondaryTrackCamera>),
    >,
    mut second: Query<
        (Entity, &mut Camera, &TrackView, Option<&RenderTarget>, Option<&Msaa>, Option<&RenderLayers>, Option<&bevy::light::ShadowFilteringMethod>, Has<Tonemapping>),
        (With<SecondaryTrackCamera>, Without<TrackCamera>),
    >,
) {
    let Some((me, mut mcam, mview, mtarget, mmsaa, mlayers, mshadow)) = main.iter_mut().next() else { return };
    let size = mcam.physical_target_size();
    let on = split.on();
    let want = mview.filter(|_| on).and_then(|v| size.map(|s| viewport_of(v.rect, s)));
    if !same_viewport(&mcam.viewport, &want) {
        mcam.viewport = want;
    }
    let mut layers = mlayers.cloned().unwrap_or_default();
    let has_view_layer = layers.intersects(&RenderLayers::layer(VIEW_CAR_LAYERS[0]));
    let per_view = split.per_view();
    if per_view && !has_view_layer {
        layers = layers.with(VIEW_CAR_LAYERS[0]);
        commands.entity(me).insert(layers.clone());
    } else if !per_view && has_view_layer {
        commands.entity(me).insert(layers.clone().without(VIEW_CAR_LAYERS[0]));
    }
    let target = mtarget.cloned();
    let msaa = mmsaa.copied().unwrap_or_default();
    let output = mcam.output_mode;
    for (e, mut cam, v, t, m, l, sh, has_tm) in &mut second {
        let order = SECOND_VIEW_ORDER + 4;
        // no sky: the world camera of the view drawn first clears (alpha 0, as `setup_track_camera`)
        let clear = if sky.is_empty() { ClearColorConfig::Custom(clear_colour.0.with_alpha(0.0)) } else { ClearColorConfig::None };
        if cam.order != order {
            cam.order = order;
        }
        if std::mem::discriminant(&cam.clear_color) != std::mem::discriminant(&clear) {
            cam.clear_color = clear;
        }
        if !same_debug::<CameraOutputMode>(&cam.output_mode, &output) {
            cam.output_mode = output;
        }
        let want = size.map(|s| viewport_of(v.rect, s));
        if !same_viewport(&cam.viewport, &want) {
            cam.viewport = want;
        }
        let mut ec = commands.entity(e);
        if !has_tm {
            ec.insert(Tonemapping::None);
        }
        match (&target, t) {
            (Some(a), Some(b)) if same_debug(a, b) => {}
            (None, None) => {}
            (Some(a), _) => {
                ec.insert(a.clone());
            }
            (None, Some(_)) => {
                ec.remove::<RenderTarget>();
            }
        }
        if m.copied().unwrap_or_default() != msaa {
            ec.insert(msaa);
        }
        let want_layers = layers.clone().without(VIEW_CAR_LAYERS[0]).with(VIEW_CAR_LAYERS[v.player.min(1)]);
        if l != Some(&want_layers) {
            ec.insert(want_layers);
        }
        match (mshadow, sh) {
            (Some(a), Some(b)) if a == b => {}
            (Some(a), _) => {
                ec.insert(*a);
            }
            (None, Some(_)) => {
                ec.remove::<bevy::light::ShadowFilteringMethod>();
            }
            (None, None) => {}
        }
    }
}

/// the mirror's sky passes and world camera before the FSE gives them slots (`update_mirror_orders`): right after the
/// world camera; with the FSE they take the five slots `mirror_scene` queues after the lens spray
pub const MIRROR_FALLBACK_ORDER: isize = WORLD_CAMERA_ORDER + 1;
/// the mirror's FSE slots: 4 sky passes then the world (`ROAD_SCENE_MIRROR` + k)
pub const MIRROR_SLOTS: usize = 5;

/// GameManager 0x000B9010 for the mirror view (kind 1): 0x002EF0B0(0, 1.0, 0, 5) clears z and stencil only (still
/// the previous view's whole-screen viewport), the camera manager switches the player to the mirror camera (mode 2,
/// 0x0005FFE0 → 0x0005FB30), then 0x000B3C30 sets the mirror rect and the world pass runs with flags 0x08000620
/// (sky + sun/moon 0x20, cars 0x200, fog 0x400, barriers 0x08000000; sectors and characters always; no TrackFX,
/// skid marks, bloom, flares, particles, rain, zone FX or lens spray) — drawn after view 0's lens spray, before the
/// flash records and the fade. Here: the `MirrorView` camera takes player 0's target, msaa, output, glow view key
/// and layers (view layer 18 instead of 17: player 0's camera-built effects are tagged away from it, the cars pick a
/// LOD for it as for a second view), its rect as viewport, no colour clear, depth cleared.
#[allow(clippy::type_complexity)]
pub fn setup_mirror_camera(
    mut commands: Commands,
    split: Res<SplitScreen>,
    main: Query<
        (&Camera, Option<&RenderTarget>, Option<&Msaa>, Option<&RenderLayers>, Option<&bevy::light::ShadowFilteringMethod>),
        (With<TrackCamera>, Without<SecondaryTrackCamera>, Without<MirrorView>),
    >,
    mut mirror: Query<
        (
            Entity,
            &mut Camera,
            &mut Camera3d,
            &MirrorView,
            Option<&RenderTarget>,
            Option<&Msaa>,
            Option<&RenderLayers>,
            Option<&bevy::light::ShadowFilteringMethod>,
            Has<Tonemapping>,
        ),
        (Without<TrackCamera>, Without<SecondaryTrackCamera>),
    >,
) {
    let m0 = main.iter().next();
    for (e, mut cam, mut cam3d, mv, t, m, l, sh, has_tm) in &mut mirror {
        let active = mv.on && split.mirror.is_some() && m0.is_some() && !crate::golden::perf_off("mirror");
        if cam.is_active != active {
            cam.is_active = active;
        }
        let Some((mcam, mtarget, mmsaa, mlayers, mshadow)) = m0.filter(|_| active) else { continue };
        if !matches!(cam.clear_color, ClearColorConfig::None) {
            cam.clear_color = ClearColorConfig::None;
        }
        if !matches!(cam3d.depth_load_op, bevy::camera::Camera3dDepthLoadOp::Clear(_)) {
            cam3d.depth_load_op = bevy::camera::Camera3dDepthLoadOp::Clear(0.0);
        }
        if !same_debug::<CameraOutputMode>(&cam.output_mode, &mcam.output_mode) {
            cam.output_mode = mcam.output_mode;
        }
        let want = mcam.physical_target_size().map(|s| viewport_of(mv.rect, s));
        if !same_viewport(&cam.viewport, &want) {
            cam.viewport = want;
        }
        let mut ec = commands.entity(e);
        if !has_tm {
            ec.insert(Tonemapping::None);
        }
        match (mtarget, t) {
            (Some(a), Some(b)) if same_debug(a, b) => {}
            (None, None) => {}
            (Some(a), _) => {
                ec.insert(a.clone());
            }
            (None, Some(_)) => {
                ec.remove::<RenderTarget>();
            }
        }
        let msaa = mmsaa.copied().unwrap_or_default();
        if m.copied().unwrap_or_default() != msaa {
            ec.insert(msaa);
        }
        let want_layers = mlayers.cloned().unwrap_or_default().without(VIEW_CAR_LAYERS[0]).with(VIEW_CAR_LAYERS[1]);
        if l != Some(&want_layers) {
            ec.insert(want_layers);
        }
        match (mshadow, sh) {
            (Some(a), Some(b)) if a == b => {}
            (Some(a), _) => {
                ec.insert(*a);
            }
            (None, Some(_)) => {
                ec.remove::<bevy::light::ShadowFilteringMethod>();
            }
            (None, None) => {}
        }
    }
}

/// the mirror's slots in the FSE pass list: queued after the lens spray (0x0015FB30, the last effect of view 0),
/// before the fade
fn mirror_scene(split: Res<SplitScreen>, fse: Option<ResMut<crate::full_screen_effects_manager_pc_xbox::FullScreenEffects>>) {
    let Some(mut fse) = fse else { return };
    if split.mirror.is_none() {
        return;
    }
    let r = fse.viewport();
    fse.set_viewport(FULL);
    for k in 0..MIRROR_SLOTS {
        fse.road_scene(crate::full_screen_effects_manager_pc_xbox::ROAD_SCENE_MIRROR + k);
    }
    fse.set_viewport(r);
}

/// the mirror's sky copies and world camera at this frame's mirror slots (or right after the world camera when the
/// FSE is off); runs in `FseSet::Execute` after the passes (added by the FSE plugin)
#[allow(clippy::type_complexity)]
pub(crate) fn update_mirror_orders(
    fse: Option<Res<crate::full_screen_effects_manager_pc_xbox::FullScreenEffects>>,
    mut world: Query<&mut Camera, (With<MirrorView>, Without<SkyCamera>, Without<PostViewCamera>)>,
    mut skies: Query<(&mut Camera, &ViewSky), (With<SkyCamera>, Without<PostViewCamera>)>,
    source: Query<&Camera, (With<SkyCamera>, Without<ViewSky>, Without<MirrorView>, Without<PostViewCamera>)>,
    mut post: Query<&mut Camera, (With<PostViewCamera>, Without<MirrorView>, Without<SkyCamera>)>,
) {
    for mut c in &mut post {
        let o = fse
            .as_ref()
            .and_then(|f| f.scene_order(crate::full_screen_effects_manager_pc_xbox::ROAD_SCENE_POST))
            .unwrap_or(POST_FALLBACK_ORDER);
        if c.order != o {
            c.order = o;
        }
    }
    let slot = |k: usize| {
        fse.as_ref()
            .and_then(|f| f.scene_order(crate::full_screen_effects_manager_pc_xbox::ROAD_SCENE_MIRROR + k))
            .unwrap_or(MIRROR_FALLBACK_ORDER + k as isize)
    };
    for mut c in &mut world {
        let o = slot(MIRROR_SLOTS - 1);
        if c.order != o {
            c.order = o;
        }
    }
    for (mut c, v) in &mut skies {
        if v.player != MIRROR_SKY {
            continue;
        }
        let Ok(src) = source.get(v.source) else { continue };
        let pass = (src.order - (WORLD_CAMERA_ORDER - 4)).clamp(0, MIRROR_SLOTS as isize - 2) as usize;
        let o = slot(pass);
        if c.order != o {
            c.order = o;
        }
    }
}

/// a car's LOD picks per view (CarRender 0x00111920 inside each view): bit `p` of `lods[k]` = view p draws LOD k,
/// `lights` = the views drawing the lights (LOD ≠ 2, 0x00112A50)
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct CarViewLod {
    pub lods: Vec<u8>,
    pub lights: u8,
}

impl CarViewLod {
    /// `sel` = (view player, picked LOD) per view
    pub fn from_selection(sel: &[(usize, u8)], lods: usize) -> Self {
        let mut m = CarViewLod { lods: vec![0; lods], lights: 0 };
        for (p, l) in sel {
            let bit = 1u8 << (*p).min(1);
            if let Some(x) = m.lods.get_mut(*l as usize) {
                *x |= bit;
            }
            if *l != 2 {
                m.lights |= bit;
            }
        }
        m
    }
}

/// the meshes under `root` drawn by the views in `mask` only (`VIEW_CAR_LAYERS`); an empty mask leaves them (the
/// group is hidden)
pub fn set_view_layers(commands: &mut Commands, children: &Query<&Children>, meshes: &Query<(), With<Mesh3d>>, root: Entity, mask: u8) {
    if mask == 0 {
        return;
    }
    let mut layers = RenderLayers::none();
    for (p, l) in VIEW_CAR_LAYERS.iter().enumerate() {
        if mask >> p & 1 != 0 {
            layers = layers.with(*l);
        }
    }
    for e in std::iter::once(root).chain(children.iter_descendants(root)) {
        if meshes.contains(e) {
            commands.entity(e).insert(layers.clone());
        }
    }
}

/// undo `set_view_layers` (one view again)
pub fn clear_view_layers(commands: &mut Commands, children: &Query<&Children>, meshes: &Query<(), With<Mesh3d>>, root: Entity) {
    for e in std::iter::once(root).chain(children.iter_descendants(root)) {
        if meshes.contains(e) {
            commands.entity(e).remove::<RenderLayers>();
        }
    }
}

/// `CSkyRender::RenderBackground` 0x0013CF40: background v offset / span for a camera pitch and vertical fov
pub fn background_uv(forward_y: f32, fov: f32) -> (Vec2, Vec4) {
    let k = 1.0 / fov;
    let pitch = forward_y.clamp(-1.0, 1.0).asin();
    (Vec2::new(0.0, (0.0 - pitch) * k), Vec4::new(1.0, BACKGROUND_SPAN * k, 1.0, 1.0))
}

/// the sky passes inside each view: player 0's sky cameras take the `TrackCamera` viewport; for player 1 each sky
/// camera is copied once (orders from `SECOND_VIEW_ORDER`, background on `VIEW_SKY_BACKGROUND_LAYER` with its own
/// quad and material), following player 1's camera; player 1's background clears the target instead of player 0's.
/// The mirror view (flag 0x20: sky and sun/moon drawn) gets the same copies (`MIRROR_SKY`, no clear: 0x000B9010
/// clears only z and stencil for it, 0x002EF0B0 flags 5), ordered in its FSE slots (`update_mirror_orders`).
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn sync_view_sky(
    mut commands: Commands,
    split: Res<SplitScreen>,
    main: Query<&Camera, (With<TrackCamera>, Without<SkyCamera>, Without<SecondaryTrackCamera>, Without<MirrorView>)>,
    second: Query<(&Camera, &GlobalTransform, &Projection), (With<SecondaryTrackCamera>, Without<SkyCamera>, Without<MirrorView>)>,
    mirror: Query<(&Camera, &GlobalTransform, &Projection, &MirrorView), (Without<SkyCamera>, Without<TrackCamera>, Without<SecondaryTrackCamera>)>,
    mut skies: Query<
        (Entity, &mut Camera, &Projection, Option<&RenderLayers>, Option<&ChildOf>, Option<&ViewSky>, &mut Transform, Option<&SavedClear>),
        (With<SkyCamera>, Without<TrackCamera>, Without<SecondaryTrackCamera>, Without<MirrorView>),
    >,
    bg: Query<(&SkyBackground, &Mesh3d, &Transform, &ChildOf), (Without<SkyCamera>, Without<ViewSkyBackground>)>,
    view_bg: Query<&ViewSkyBackground>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
) {
    let main_vp = main.iter().next().and_then(|c| c.viewport.clone());
    let split_sec = second.iter().next().filter(|_| split.on());
    let mirror_sec = mirror.iter().find(|m| m.3.on).filter(|_| split.mirror.is_some()).map(|(c, t, p, _)| (c, t, p));
    let (sec, copy_player) = match (split_sec, mirror_sec) {
        (Some(s), _) => (Some(s), 1),
        (None, Some(m)) => (Some(m), MIRROR_SKY),
        _ => (None, 1),
    };
    let split_on = split_sec.is_some();
    // copies already made (of the wanted kind), by source
    let copies: Vec<(Entity, Entity)> =
        skies.iter().filter_map(|(e, _, _, _, _, v, _, _)| v.filter(|v| v.player == copy_player).map(|v| (v.source, e))).collect();
    let mut spawned = Vec::new();
    for (e, mut cam, proj, layers, parent, view, mut t, saved) in &mut skies {
        if let Some(v) = view {
            let Some((scam, st, _)) = sec.filter(|_| v.player == copy_player) else {
                commands.entity(e).despawn();
                continue;
            };
            if copy_player == MIRROR_SKY {
                // the mirror's sky follows its camera's activity (order: `update_mirror_orders`)
                if cam.is_active != scam.is_active {
                    cam.is_active = scam.is_active;
                }
            }
            t.rotation = st.compute_transform().rotation;
            t.translation = Vec3::ZERO;
            if !same_viewport(&cam.viewport, &scam.viewport) {
                cam.viewport = scam.viewport.clone();
            }
            continue;
        }
        let want = if split_on { main_vp.clone() } else { None };
        if !same_viewport(&cam.viewport, &want) {
            cam.viewport = want;
        }
        let clears = !matches!(cam.clear_color, ClearColorConfig::None) || saved.is_some();
        if split_on && clears && saved.is_none() {
            commands.entity(e).insert(SavedClear(cam.clear_color));
            cam.clear_color = ClearColorConfig::None;
        } else if !split_on {
            if let Some(s) = saved {
                cam.clear_color = s.0;
                commands.entity(e).remove::<SavedClear>();
            }
        }
        if sec.is_none() || copies.iter().any(|(s, _)| *s == e) {
            continue;
        }
        let layers = layers.cloned().unwrap_or_default();
        let is_bg = layers.intersects(&RenderLayers::layer(SKY_PASS_BACKGROUND));
        let clear = match saved {
            _ if !split_on => ClearColorConfig::None,
            Some(s) => s.0,
            None if clears => cam.clear_color,
            None => ClearColorConfig::None,
        };
        let pass = cam.order - (WORLD_CAMERA_ORDER - 4);
        let order = if split_on { pass + SECOND_VIEW_ORDER } else { MIRROR_FALLBACK_ORDER + pass };
        let mut c = commands.spawn((
            Camera3d::default(),
            Camera { order, clear_color: clear, viewport: sec.and_then(|(s, _, _)| s.viewport.clone()), ..default() },
            Tonemapping::None,
            proj.clone(),
            Transform::default(),
            // the shared sky camera (`sky_render::sky_one_camera`) keeps its blended pass layers
            if is_bg { layers.without(SKY_PASS_BACKGROUND).with(VIEW_SKY_BACKGROUND_LAYER) } else { layers },
            SkyCamera,
            ViewSky { player: copy_player, source: e },
        ));
        if let Some(p) = parent {
            c.insert(ChildOf(p.parent()));
        }
        let id = c.id();
        if is_bg {
            spawned.push((e, id));
        }
    }
    // the background quad: a copy under the new background camera with its own material
    for (src, cam) in spawned {
        for (b, mesh, t, parent) in &bg {
            if parent.parent() != src {
                continue;
            }
            let Some(m) = materials.get(&b.0).cloned() else { continue };
            let mat = materials.add(m);
            commands.spawn((
                Mesh3d(mesh.0.clone()),
                MeshMaterial3d(mat.clone()),
                *t,
                RenderLayers::layer(VIEW_SKY_BACKGROUND_LAYER),
                ViewSkyBackground(mat),
                ChildOf(cam),
            ));
        }
    }
    // background v range from player 1's pitch (as `update_sky` for player 0)
    if let Some((_, st, proj)) = sec {
        let fov = match proj {
            Projection::Perspective(p) => p.fov,
            _ => BACKGROUND_SPAN,
        };
        let (off, scale) = background_uv(st.forward().y, fov);
        for vb in &view_bg {
            if let Some(mut m) = materials.get_mut(&vb.0) {
                m.params.uv_offset = off;
                m.params.uv_scale = scale;
            }
        }
    }
}

/// VsCameraManager 0x0005F8F0: the projection keeps the full screen's aspect inside a view's viewport (bevy would
/// widen it to the viewport's); run after `camera_system`, before the frusta
#[allow(clippy::type_complexity)]
pub fn fix_view_aspect(
    split: Res<SplitScreen>,
    mut cams: Query<
        (&mut Camera, &mut Projection),
        Or<(
            With<TrackCamera>,
            With<SecondaryTrackCamera>,
            With<MirrorView>,
            With<SkyCamera>,
            With<crate::full_screen_effects_manager_pc_xbox::CarStencilCamera>,
        )>,
    >,
) {
    if !split.per_view() {
        return;
    }
    for (mut cam, mut proj) in &mut cams {
        if cam.viewport.is_none() {
            continue;
        }
        let Some(size) = cam.physical_target_size().filter(|s| s.x > 0 && s.y > 0) else { continue };
        let aspect = size.x as f32 / size.y as f32;
        let Projection::Perspective(p) = &*proj else { continue };
        if (p.aspect_ratio - aspect).abs() <= 1e-6 {
            continue;
        }
        let p = proj.bypass_change_detection();
        if let Projection::Perspective(pp) = p {
            pp.aspect_ratio = aspect;
        }
        cam.computed.clip_from_view = match &cam.sub_camera_view {
            Some(sub) => p.get_clip_from_view_for_sub(sub),
            None => p.get_clip_from_view(),
        };
    }
}

/// the after-the-loop layer: GameManager 0x000B9010 draws the flash records (0x00127080) after every view (and the
/// mirror) with 0x000B3C30 on view 0 = the whole screen and player 0's camera, before the fade and the HUD.
/// (was 23 = frontend's `HUD_3D_LAYER_P2`: the post camera drew player 1's 3D hud models into the top view)
pub const POST_VIEW_LAYER: usize = 11;
/// `EffectDraw::view` tag of batches drawn after the view loop (`POST_VIEW_LAYER`)
pub const POST_VIEW_TAG: usize = usize::MAX - 1;
/// the post camera's order without the FSE (after the mirror's fallback slots)
pub const POST_FALLBACK_ORDER: isize = MIRROR_FALLBACK_ORDER + MIRROR_SLOTS as isize;

/// the camera drawing `POST_VIEW_LAYER`: player 0's camera (transform, projection, target, msaa, output) over the
/// whole target, no colour clear, depth kept; ordered at the FSE slot `ROAD_SCENE_POST` (after the lens spray and
/// the mirror, before the fade)
#[derive(Component)]
pub struct PostViewCamera;

#[allow(clippy::type_complexity)]
fn setup_post_camera(
    mut commands: Commands,
    main: Query<
        (&Camera, &Transform, &Projection, Option<&RenderTarget>, Option<&Msaa>),
        (With<TrackCamera>, Without<SecondaryTrackCamera>, Without<PostViewCamera>),
    >,
    mut post: Query<
        (Entity, &mut Camera, &mut Transform, &mut Projection, Option<&RenderTarget>, Option<&Msaa>),
        (With<PostViewCamera>, Without<TrackCamera>),
    >,
    used: Option<Res<crate::effect_draw::PostViewUsed>>,
) {
    let m0 = main.iter().next();
    // perf: the camera runs only in frames with a flash record batch (0x00127080 draws nothing otherwise; an
    // empty pass left the target as it was); `VELOCITY_PERF_OFF=postopt` = always
    let drawn = used.is_none_or(|u| u.0) || crate::golden::perf_off("postopt");
    let Some((e, mut cam, mut t, mut p, target, msaa)) = post.iter_mut().next() else {
        if m0.is_some() {
            commands.spawn((
                Camera3d { depth_load_op: bevy::camera::Camera3dDepthLoadOp::Load, ..default() },
                Camera { order: POST_FALLBACK_ORDER, clear_color: ClearColorConfig::None, is_active: false, ..default() },
                Tonemapping::None,
                RenderLayers::layer(POST_VIEW_LAYER),
                PostViewCamera,
            ));
        }
        return;
    };
    let active = m0.is_some() && drawn;
    if cam.is_active != active {
        cam.is_active = active;
    }
    let Some((mcam, mt, mp, mtarget, mmsaa)) = m0 else { return };
    if *t != *mt {
        *t = *mt;
    }
    if !same_debug::<Projection>(&p, mp) {
        *p = mp.clone();
    }
    if !same_debug::<CameraOutputMode>(&cam.output_mode, &mcam.output_mode) {
        cam.output_mode = mcam.output_mode;
    }
    let mut ec = commands.entity(e);
    match (mtarget, target) {
        (Some(a), Some(b)) if same_debug(a, b) => {}
        (None, None) => {}
        (Some(a), _) => {
            ec.insert(a.clone());
        }
        (None, Some(_)) => {
            ec.remove::<RenderTarget>();
        }
    }
    let want = mmsaa.copied().unwrap_or_default();
    if msaa.copied().unwrap_or_default() != want {
        ec.insert(want);
    }
}

/// the post slot in the FSE pass list (after the mirror's slots, before the fade)
fn post_scene(main: Query<(), (With<TrackCamera>, Without<SecondaryTrackCamera>)>, fse: Option<ResMut<crate::full_screen_effects_manager_pc_xbox::FullScreenEffects>>) {
    let Some(mut fse) = fse else { return };
    if main.is_empty() {
        return;
    }
    let r = fse.viewport();
    fse.set_viewport(FULL);
    fse.road_scene(crate::full_screen_effects_manager_pc_xbox::ROAD_SCENE_POST);
    fse.set_viewport(r);
}

/// each frame's full-screen effects start in player 0's view (GameManager 0x000B3C30 on the view); the bloom loops
/// over the views itself, the fade (after the view loop, 0x000B3C30 on view 0 = whole screen) draws full screen
fn fse_view_viewport(split: Res<SplitScreen>, fse: Option<ResMut<crate::full_screen_effects_manager_pc_xbox::FullScreenEffects>>) {
    if let Some(mut f) = fse {
        let r = if split.on() { split.rect(0) } else { FULL };
        if f.viewport() != r {
            f.set_viewport(r);
        }
    }
}

pub struct ViewRenderPlugin;

impl Plugin for ViewRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SplitScreen>()
            .add_systems(
                PostUpdate,
                fix_view_aspect
                    .after(bevy::camera::CameraUpdateSystems)
                    .before(bevy::camera::visibility::VisibilitySystems::UpdateFrusta),
            )
            .add_systems(PostUpdate, fse_view_viewport.before(crate::full_screen_effects_manager_pc_xbox::FseSet::Build))
            .add_systems(
                PostUpdate,
                (mirror_scene, post_scene)
                    .chain()
                    .in_set(crate::full_screen_effects_manager_pc_xbox::FseSet::Build)
                    .after(crate::lens_spray_manager::LensSpraySet)
                    .before(crate::fade_render::FadeSet),
            )
            .add_systems(
                PostUpdate,
                setup_post_camera
                    .after(crate::effect_draw::EffectSet::Flush)
                    .before(bevy::camera::CameraUpdateSystems)
                    .before(bevy::transform::TransformSystems::Propagate),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// GameManager 0x000B3960 kinds 2 / 3 on a 1280x720 target: top and bottom halves
    #[test]
    fn view_viewports() {
        let top = viewport_of(Vec4::new(0.0, 0.0, 1.0, 0.5), UVec2::new(1280, 720));
        let bottom = viewport_of(Vec4::new(0.0, 0.5, 1.0, 0.5), UVec2::new(1280, 720));
        assert_eq!((top.physical_position, top.physical_size), (UVec2::ZERO, UVec2::new(1280, 360)));
        assert_eq!((bottom.physical_position, bottom.physical_size), (UVec2::new(0, 360), UVec2::new(1280, 360)));
        let s = SplitScreen { views: vec![(0, Vec4::new(0.0, 0.0, 1.0, 0.5)), (1, Vec4::new(0.0, 0.5, 1.0, 0.5))], mirror: None };
        assert!(s.on() && s.per_view());
        assert_eq!(s.rect(1), [0.0, 0.5, 1.0, 0.5]);
        assert_eq!(s.rect(3), FULL);
        assert!(!SplitScreen::default().on());
        // one player with the mirror: per-view layers, but not a split (no `byte_6F7157` gates)
        let m = SplitScreen { views: vec![(0, Vec4::new(0.0, 0.0, 1.0, 1.0))], mirror: Some(Vec4::new(213.0 / 640.0, 0.0, 213.0 / 640.0, 80.0 / 480.0)) };
        assert!(!m.on() && m.per_view());
        let vp = viewport_of(m.mirror.unwrap(), UVec2::new(640, 480));
        assert_eq!((vp.physical_position, vp.physical_size), (UVec2::new(213, 0), UVec2::new(213, 80)));
    }

    /// player 0 near (LOD 0), player 1 far (LOD 2): each group goes to one view, lights only to player 0's
    #[test]
    fn per_view_lod_masks() {
        let m = CarViewLod::from_selection(&[(0, 0), (1, 2)], 3);
        assert_eq!(m.lods, vec![1, 0, 2]);
        assert_eq!(m.lights, 1);
        let both = CarViewLod::from_selection(&[(0, 1), (1, 1)], 3);
        assert_eq!((both.lods.clone(), both.lights), (vec![0, 3, 0], 3));
    }

    /// the background span matches `update_sky` (level camera: offset 0)
    #[test]
    fn background_uv_level() {
        let (o, s) = background_uv(0.0, BACKGROUND_SPAN);
        assert_eq!(o, Vec2::ZERO);
        assert!((s.y - 1.0).abs() < 1e-6);
    }
}
