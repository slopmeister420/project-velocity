//! Render\DangerZoneRender.cpp — the depth-of-field part (object `dword_718BA8`, 0x00120B20..0x001212D0) and its
//! draw (0x00158F20..0x00159860, in the CarShaderManager_PC_XBOX.cpp range, shaders "DOF*" from 0x00158F90).
//!
//! The back buffer is blurred once into a half buffer; up to three planes in front of the focus distance and three
//! behind it are drawn from it as z-tested screen quads at their view depth: the near ones with GREATEREQUAL (blur
//! what lies in front of the plane), the far ones with LESSEQUAL, each with its own alpha. No time dependency.
//! Also the danger zone (object `unk_718AC0`, TrailBlazer's effect): the "Danger" overlay 0x00120850 and the record
//! lerp / motion blur / ring warp 0x001205E0 → 0x0011F860. The screen fade 0x00123250 is `fade_render`. 0x00123E60.. is
//! FlareRender.cpp (`flare_render`).

use bevy::prelude::*;

use crate::full_screen_effects_manager_pc_xbox::{prog, FseCameraPlanes, FseImage, FseSet, FseShader, FseVertex, FullScreenEffects, OFF_SCREEN_HALF, WHITE};
use crate::global_mat_input_resolver::gmi;
use crate::light_streak::visos_colour;
use crate::renderer::shader_manager::d3dcmp;
use crate::track_render::TrackCamera;

/// `dword_6868B8` — "DOF Factor" per mode (0 game, 1/2 = 0x00120BF0 without / with blend)
pub const DOF_FACTOR: [f32; 3] = [f32::from_bits(0x3f000000), f32::from_bits(0x3e99999a), f32::from_bits(0x3f000000)];
/// `dword_6868C4` — "Blur Factor"
pub const BLUR_FACTOR: [f32; 3] = [f32::from_bits(0x3f800000), f32::from_bits(0x3f800000), f32::from_bits(0x3eb33333)];
/// `dword_6868D0` — focus offset (m)
pub const FOCUS_OFFSET: [f32; 3] = [f32::from_bits(0x3f800000), f32::from_bits(0x3fb33333), f32::from_bits(0x3f800000)];
/// `dword_6868DC` — "Max Blur" (read by nothing here)
pub const MAX_BLUR: [i32; 3] = [16, 16, 0];
/// `dword_6868E8` — minimum focus distance (mm)
pub const MIN_FOCUS_MM: f32 = f32::from_bits(0x44960000);
/// `dword_6868EC` — plane alpha per layer
pub const LAYER_ALPHA: [i32; 3] = [128, 80, 48];
/// `dword_6868F8`
pub const LAYER_SPREAD: f32 = f32::from_bits(0x3f000000);
/// `dword_6868FC` / `dword_686900` — circle-of-confusion apertures (×1000) of the far / near planes
pub const FAR_APERTURE: f32 = f32::from_bits(0x437a0000);
pub const NEAR_APERTURE: f32 = f32::from_bits(0x40000000);
/// `dword_686904` / `dword_686908` — replay camera type 3: dof factor and focus (mm)
pub const REPLAY_DOF_FACTOR: f32 = f32::from_bits(0x3de147ae);
pub const REPLAY_FOCUS_MM: f32 = f32::from_bits(0x458ca000);
/// `unk_68690C` (game) / `aFff` 0x00686918 (cut scenes) — plane spacing per layer
pub const LAYER_SCALE_GAME: [f32; 3] = [f32::from_bits(0x3f400000), f32::from_bits(0x3f800000), f32::from_bits(0x3fb33333)];
pub const LAYER_SCALE_CUT: [f32; 3] = [f32::from_bits(0x3f666666), f32::from_bits(0x3f800000), f32::from_bits(0x3f933333)];

/// what 0x00120D20 / 0x00120E80 read from the game (core fills it each frame; default = in race, no replay)
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct DepthOfFieldInput {
    /// replay playing (`FontHelper__B36B0(byte_6F5A28)`)
    pub replay: bool,
    /// replay camera type (`ReplayCamera_Manager__1664F0`)
    pub replay_camera_type: u32,
    /// the other draw conditions of 0x00120D20: camera mode 3 (`VsCameraManager__5FFB0`), `dword_749110 == 2`,
    /// `byte_718E70`
    pub other: bool,
    /// the player's race vehicle position (focus on the car, +3)
    pub car_position: Option<Vec3>,
}

/// the DOF object (`dword_718BA8`)
#[derive(Resource, Clone, Debug)]
pub struct DepthOfField {
    pub active: bool,           // +0
    pub enabled: bool,          // +1 tweak "Enabled"
    pub debug: bool,            // +2 tweak "Debug Display"
    pub car_focus: bool,        // +3
    pub cut_scene: bool,        // +4
    pub force: bool,            // +5
    pub dof_factor: f32,        // +8
    pub blur_factor: f32,       // +12
    pub focus_offset: f32,      // +16
    pub max_blur: i32,          // +20
    pub far_dist: [f32; 3],     // +24 (m)
    pub near_dist: [f32; 3],    // +36 (m)
    pub focus: f32,             // +48 (m)
    pub blend: f32,             // +52
    pub blend_from: f32,        // +56
    pub blend_to: f32,          // +60
    pub far_alpha: [i32; 3],    // +64
    pub near_alpha: [i32; 3],   // +76
}

impl DepthOfField {
    /// DangerZoneRender.cpp 0x00120B20 (the game object: `cut_scene` false)
    pub fn new(cut_scene: bool) -> Self {
        let k = cut_scene as usize;
        Self {
            active: !cut_scene,
            enabled: true,
            debug: false,
            car_focus: !cut_scene,
            cut_scene,
            force: false,
            dof_factor: DOF_FACTOR[k],
            blur_factor: BLUR_FACTOR[k],
            focus_offset: FOCUS_OFFSET[k],
            max_blur: MAX_BLUR[k],
            far_dist: [0.0; 3],
            near_dist: [0.0; 3],
            focus: 10.0,
            blend: 0.0,
            blend_from: 0.0,
            blend_to: 0.0,
            far_alpha: [0; 3],
            near_alpha: [0; 3],
        }
    }

    /// DangerZoneRender.cpp 0x00120BF0 — focus at `focus` m (≤ 0 = off), `blend` picks table 2 and the blended
    /// focus (+60·+52 + (1 − +52)·+56), `off` turns it off
    pub fn set_focus(&mut self, focus: f32, blend: bool, off: bool) {
        self.car_focus = false;
        self.focus = focus;
        self.active = self.focus > 0.0;
        if off {
            self.active = false;
        }
        let k = blend as usize + 1;
        self.dof_factor = DOF_FACTOR[k];
        self.blur_factor = BLUR_FACTOR[k];
        self.focus_offset = FOCUS_OFFSET[k];
        self.max_blur = MAX_BLUR[k];
        if blend {
            self.focus = self.blend_to * self.blend + (1.0 - self.blend) * self.blend_from;
        }
    }

    /// DangerZoneRender.cpp 0x00120CF0
    pub fn set(&mut self, focus: f32, dof_factor: f32) {
        self.focus = focus;
        self.dof_factor = dof_factor;
    }

    /// DangerZoneRender.cpp 0x00120D20 — the draw gate (true = update and draw)
    pub fn wants_draw(&mut self, inp: &DepthOfFieldInput) -> bool {
        if !self.enabled || !self.active {
            return false;
        }
        if !(self.force || self.cut_scene || inp.replay || inp.other) {
            return false;
        }
        if inp.replay && !self.cut_scene {
            self.car_focus = inp.replay_camera_type != 3;
        }
        true
    }

    /// DangerZoneRender.cpp 0x00120E80 — plane distances and alphas for the camera at `cam_pos` with vertical fov
    /// `fov` (radians); false = no plane
    pub fn update(&mut self, inp: &DepthOfFieldInput, cam_pos: Vec3, fov: f32) -> bool {
        let scale = if self.cut_scene { LAYER_SCALE_CUT } else { LAYER_SCALE_GAME };
        let mut factor = self.dof_factor;
        if !self.cut_scene {
            self.max_blur = MAX_BLUR[0];
        }
        let mut d;
        if self.car_focus {
            let car = inp.car_position.unwrap_or(cam_pos);
            d = ((car - cam_pos).length() - self.focus_offset) * 1000.0;
        } else if inp.replay && !self.cut_scene && inp.replay_camera_type == 3 {
            d = REPLAY_FOCUS_MM;
            factor = REPLAY_DOF_FACTOR;
            self.max_blur = 32;
        } else {
            d = (self.focus - self.focus_offset) * 1000.0;
        }
        if MIN_FOCUS_MM > d {
            d = MIN_FOCUS_MM;
        }
        // focal length (mm) from the fov (CarRender 0x0011B700, VsBaseCamera 0x0005EAE0 = tan)
        let f = 90.0 / fov.tan();
        let mut near = coc(NEAR_APERTURE * 1000.0, d, f) * self.blur_factor * 100.0;
        let mut far = coc(FAR_APERTURE * 1000.0, d, f) * self.blur_factor * 100.0;
        if near > 1.0 {
            near = 1.0;
        }
        if far > 1.0 {
            far = 1.0;
        }
        let near_a = near.sqrt().clamp(0.0, 1.0);
        let far_a = far.sqrt().clamp(0.0, 1.0);
        let mut any = false;
        for i in 0..3 {
            let s = (near * LAYER_SPREAD + 1.0 - LAYER_SPREAD) * scale[i] * factor;
            let h = f * f * s;
            let den = (d - f) + h;
            if den <= 0.0 {
                self.near_alpha[i] = 0;
            } else {
                self.near_dist[i] = h * d / den * 0.001;
                self.near_alpha[i] = (LAYER_ALPHA[i] as f32 * near_a) as i32;
                any = true;
            }
            let s = (far * LAYER_SPREAD + 1.0 - LAYER_SPREAD) * scale[i] * factor;
            let h = f * f * s;
            let den = h - (d - f);
            if den <= 0.0 {
                self.far_alpha[i] = 0;
            } else {
                self.far_dist[i] = h * d / den * 0.001;
                self.far_alpha[i] = (LAYER_ALPHA[i] as f32 * far_a) as i32;
                any = true;
            }
        }
        any
    }
}

impl Default for DepthOfField {
    fn default() -> Self {
        Self::new(false)
    }
}

/// DangerZoneRender.cpp 0x00120E30 — circle of confusion |(d / aperture) · f² / (d − f)|
pub fn coc(aperture: f32, d: f32, f: f32) -> f32 {
    (((d / aperture) * f) * f / (d - f)).abs()
}

/// "DOFFar" (blend 2, z test LESSEQUAL no write) / "DOFNear" (same, z func 3 = GREATEREQUAL); stage 0 colour
/// MODULATE(diffuse, texture), alpha SELECTARG1 diffuse (0x00158F90)
fn plane_shader(near: bool) -> FseShader {
    let mut s = FseShader::new(2, true);
    s.ffp = crate::full_screen_effects_manager_pc_xbox::FFP_DIFFUSE_ALPHA;
    s.z_func = if near { d3dcmp::GREATEREQUAL } else { d3dcmp::LESSEQUAL };
    s
}

/// D3D projected z of a view depth with the camera planes (0x00159550: the projection matrix applied to (0,0,z,1))
fn d3d_z(view: f32, p: &FseCameraPlanes) -> f32 {
    p.far / (p.far - p.near) * (1.0 - p.near / view)
}

/// CarShaderManager_PC_XBOX.cpp 0x00159550 — one plane: the blurred half buffer at view depth `dist`
fn draw_plane(fse: &mut FullScreenEffects, half: FseImage, planes: &FseCameraPlanes, dist: f32, colour: u32, near: bool) {
    let z = d3d_z(dist, planes);
    // outside the clip volume the original's quad is clipped away
    if !(0.0..=1.0).contains(&z) {
        return;
    }
    fse.draw(FseImage::BackBuffer, half, plane_shader(near), prog::COPY, None, None, z, colour);
}

/// CarShaderManager_PC_XBOX.cpp 0x00158F20 — blur the back buffer into a half buffer (0x0015C770, one pass, Blur),
/// then 0x00159720: the planes before the focus (DOFNear), then those behind it (DOFFar)
pub fn render(fse: &mut FullScreenEffects, dof: &DepthOfField, planes: &FseCameraPlanes) {
    let Some(half) = fse.get_off_screen_buffer(OFF_SCREEN_HALF) else { return };
    fse.blur_passes(half, FseImage::BackBuffer, 1, false);
    let g: u8 = if dof.debug { 64 } else { 0x80 };
    // byte_687550 / byte_687551 (both 1)
    for i in 0..3 {
        if dof.near_alpha[i] != 0 {
            let c = visos_colour([(dof.near_alpha[i] / 2) as u8, 0x80, g, 0x80]);
            draw_plane(fse, half, planes, dof.near_dist[i], c, true);
        }
    }
    for i in 0..3 {
        if dof.far_alpha[i] != 0 {
            let c = visos_colour([(dof.far_alpha[i] / 2) as u8, g, 0x80, g]);
            draw_plane(fse, half, planes, dof.far_dist[i], c, false);
        }
    }
    fse.release_off_screen_buffer(half);
}

/// GameManager 0x000B9010: `DangerZoneRender__120D20(&dword_718BA8)` after the light streaks (after motion blur),
/// render flag 0x20000000 — in every player view (not the mirror) with that view's camera (0x0005ACD0) and viewport;
/// the focus car stays the player car (`VsAIRaceVehicle__24F90() + 4`)
fn dof_system(
    mut fse: ResMut<FullScreenEffects>,
    mut dof: ResMut<DepthOfField>,
    input: Option<Res<DepthOfFieldInput>>,
    planes: Res<FseCameraPlanes>,
    views: crate::view_render::ViewCams,
) {
    if !fse.active {
        return;
    }
    let inp = input.map(|i| *i).unwrap_or_default();
    let restore = fse.viewport();
    for v in views.list() {
        if !dof.wants_draw(&inp) {
            break;
        }
        let Some(Projection::Perspective(p)) = v.projection.as_ref() else { continue };
        if dof.update(&inp, v.transform.translation(), p.fov) {
            fse.set_viewport(v.rect);
            render(&mut fse, &dof, &planes);
        }
    }
    fse.set_viewport(restore);
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DofSet;

pub struct DepthOfFieldPlugin;

impl Plugin for DepthOfFieldPlugin {
    fn build(&self, app: &mut App) {
        let mut d = DepthOfField::default();
        // VELOCITY_DOF_FORCE=<focus m>: the +5 force flag with a fixed focus (viewer checks)
        if let Some(f) = std::env::var("VELOCITY_DOF_FORCE").ok().and_then(|v| v.parse::<f32>().ok()) {
            d.force = true;
            d.car_focus = false;
            d.focus = f;
        }
        app.insert_resource(d)
            .init_resource::<DepthOfFieldInput>()
            .add_systems(PostUpdate, dof_system.in_set(FseSet::Build).in_set(DofSet).after(crate::motion_blur_pc_xbox::MotionBlurSet));
    }
}

// ---- the danger zone (object `unk_718AC0`, TrailBlazer's screen effect) ----------------------------------------

/// DangerZoneRender.cpp 0x0011F590 — the parameter block at +72 (`a3` of 0x0011F860 / 0x00120100); pairs are lerped
/// by the danger amount (+44)
#[derive(Clone, Copy, Debug)]
pub struct DangerParams {
    pub segments: i32,          // +0 points per ring line (30)
    pub radius: [f32; 2],       // +4/+8 inner radius (0 -> 0.35)
    pub success_radius: f32,    // +12 inner radius lerp start on success (0.6 -> +8)
    pub width_ramp: f32,        // +16 width scaled by amount/ramp below it (0.1)
    pub width: [f32; 2],        // +20/+24 ring band width (0.1 -> 0.3; success: +24)
    pub displace: [f32; 2],     // +28/+32 uv push (10 -> 6)
    pub spin: f32,              // +36 clock A speed (-20/s) ...
    pub spin_from: f32,         // +40 ... from this amount on (0.9)
    pub frequency: [f32; 2],    // +44/+48 radial wave count (1 -> 2.4)
    pub wave_count: f32,        // +52 waves along the ring (2.5)
    pub wave_size: f32,         // +56 radial wave size (0.4)
    pub wave_shift: f32,        // +60 wave phase scale (1)
    pub wave_speed: f32,        // +64 clock B speed (0.5/s)
    pub tint: [f32; 3],         // +68/+72/+76 r/g/b removed (0.8, 1, 0.8)
    pub tint_scale: f32,        // +80 (0.25: ×128)
    pub warp_on: bool,          // +84 (1)
    pub blur: [f32; 2],         // +88/+92 motion blur amount by +36 (0 -> 0.012)
    pub blur_passes: i32,       // +96 (3)
    pub rest: [f32; 8],         // +100..+128 (2, 2, 0.5, 0.7, 0.5, 0.6086, 50, 0.3; not read by the ported paths)
    pub hue: f32,               // +132 overlay hue (200°, +180)
    pub saturation: f32,        // +136 (0.45)
    pub value: f32,             // +140 (0.15)
    pub hue_low: f32,           // +144 hue near the end (25°, +180)
    pub saturation_low: f32,    // +148 (0.55)
    pub value_low: f32,         // +152 (0.3)
    pub low_band: f32,          // +156 (0.3)
}

impl Default for DangerParams {
    fn default() -> Self {
        let f = f32::from_bits;
        Self {
            segments: 30,
            radius: [0.0, f(0x3EB33333)],
            success_radius: f(0x3F19999A),
            width_ramp: f(0x3DCCCCCD),
            width: [f(0x3DCCCCCD), f(0x3E99999A)],
            displace: [f(0x41200000), f(0x40C00000)],
            spin: f(0xC1A00000),
            spin_from: f(0x3F666666),
            frequency: [f(0x3F800000), f(0x4019999A)],
            wave_count: f(0x40200000),
            wave_size: f(0x3ECCCCCD),
            wave_shift: f(0x3F800000),
            wave_speed: f(0x3F000000),
            tint: [f(0x3F4CCCCD), f(0x3F800000), f(0x3F4CCCCD)],
            tint_scale: f(0x3E800000),
            warp_on: true,
            blur: [0.0, f(0x3C449BA6)],
            blur_passes: 3,
            rest: [f(0x40000000), f(0x40000000), f(0x3F000000), f(0x3F333333), f(0x3F000000), f(1058810102), f(0x42480000), f(0x3E99999A)],
            hue: f(0x43480000),
            saturation: f(0x3EE66666),
            value: f(0x3E19999A),
            hue_low: f(0x41C80000),
            saturation_low: f(0x3F0CCCCD),
            value_low: f(0x3E99999A),
            low_band: f(0x3E99999A),
        }
    }
}

/// what core supplies: TrailBlazer 0x000C5F90 each update (`RenderParams`: on, the +96 record = previous and current
/// five values, the +92 success flag), the render lerp (`dword_74A430` +32) and the pause flag
#[derive(Resource, Clone, Copy, Debug)]
pub struct DangerZoneInput {
    /// +29 (1 at init; TrailBlazer 0x000C6080 / 0x000C6120 write it at start / stop)
    pub enabled: bool,
    pub on: bool,
    pub prev: [f32; 5],
    pub cur: [f32; 5],
    pub success: bool,
    pub lerp: f32,
    pub paused: bool,
}

impl Default for DangerZoneInput {
    fn default() -> Self {
        Self { enabled: true, on: false, prev: [0.0; 5], cur: [0.0; 5], success: false, lerp: 1.0, paused: false }
    }
}

/// the danger object `unk_718AC0` (0x001202F0)
#[derive(Resource, Clone, Debug)]
pub struct DangerZone {
    pub enabled: bool,    // +29
    pub active: bool,     // +28
    pub overlay: f32,     // +32 overlay intensity
    pub fade: f32,        // +36
    pub level: f32,       // +40
    pub amount: f32,      // +44
    pub success: bool,    // +48
    pub fade_inv: f32,    // +52
    pub clock_a: f32,     // +56
    pub clock_b: f32,     // +60
    /// +68 overlay vertex colour (D3DCOLOR)
    pub colour: u32,
    pub params: DangerParams,
    /// `dword_6868AC` (7): 1 nothing, 2 motion blur, 4 warp
    pub flags: u32,
    /// `dword_6868B0` (-1): amount override when >= 0
    pub amount_override: f32,
    /// `byte_718AB8` (bss 0): debug stripes
    pub stripes: bool,
    /// "spec4Inv" (`\VFX\misc.vcd`, +0)
    spec4inv: Option<Handle<Image>>,
}

impl Default for DangerZone {
    fn default() -> Self {
        Self {
            enabled: true,
            active: false,
            overlay: f32::from_bits(0x3F400000),
            fade: 1.0,
            level: 0.0,
            amount: 0.0,
            success: false,
            fade_inv: 1.0,
            clock_a: 0.0,
            clock_b: 0.0,
            // VISOS_ColourXBOX 0x002B1370(a 0, r 0, g 0, b 1)
            colour: 0x0000_00FF,
            params: DangerParams::default(),
            flags: 7,
            amount_override: -1.0,
            stripes: false,
            spec4inv: None,
        }
    }
}

/// "Danger" warp shader (+8): blend 1, z off, LinearCopy vertex program, stage 0 MODULATE (= the Copy pair with
/// texel uvs)
pub const DANGER_WARP: FseShader = FseShader::new(1, false);
/// "Danger" overlay shader (+12): blend 1, z off, Danger vertex / fragment programs
pub const DANGER_OVERLAY: FseShader = FseShader::new(1, false);

/// DangerZoneRender.cpp 0x0011F3A0 — hsv (hue in degrees) to rgb, w = 1
pub fn hsv(h: f32, s: f32, v: f32) -> Vec4 {
    let x = h / 60.0;
    let i = x.floor();
    let f = x - i;
    let p = (1.0 - s) * v;
    let q = (1.0 - s * f) * v;
    let t = (1.0 - (1.0 - f) * s) * v;
    match i as i32 {
        0 | 6 => Vec4::new(v, t, p, 1.0),
        1 => Vec4::new(q, v, p, 1.0),
        2 => Vec4::new(p, v, t, 1.0),
        3 => Vec4::new(p, q, v, 1.0),
        4 => Vec4::new(t, p, v, 1.0),
        5 => Vec4::new(v, p, q, 1.0),
        _ => {
            log::error!("assert: false (DangerZoneRender.cpp 80)");
            Vec4::ZERO
        }
    }
}

/// `sub_3DDFA8` (_ftol)
fn ftol(v: f32) -> i32 {
    v as i32
}

/// DangerZoneRender.cpp 0x00120A60 — `sin(a·b + c) · sin(a/2)²`
fn ring_wave(a: f32, b: f32, c: f32) -> f32 {
    let s = (a * b + c).sin();
    let h = (a * 0.5).sin();
    h * h * s
}

impl DangerZone {
    /// TrailBlazer.cpp 0x000C5F90 (the writes into this object)
    pub fn apply_trail(&mut self, on: bool, success: bool) {
        if on {
            self.active = true;
            self.success = success;
        } else {
            self.active = false;
            self.fade_inv = (1.0f32 - 0.0).clamp(0.0, 1.0);
            self.fade = 0.0;
            self.overlay = 0.0;
            self.amount = 0.0;
            self.success = false;
        }
    }

    /// DangerZoneRender.cpp 0x001205E0 first half — the record lerped by the render lerp
    pub fn lerp_record(&mut self, prev: &[f32; 5], cur: &[f32; 5], t: f32) {
        let l = |k: usize| (cur[k] - prev[k]) * t + prev[k];
        let l3 = l(3);
        self.fade_inv = (1.0 - l3).clamp(0.0, 1.0);
        self.fade = l3;
        self.level = l(1);
        self.overlay = l(2);
        self.amount = l(4);
    }

    /// DangerZoneRender.cpp 0x00120100 — the overlay tint from the level: hue 200°+180 fading towards 25°+180 in
    /// the last `low_band`
    pub fn overlay_colour(&mut self) {
        let a = &self.params;
        let (mut hue, sat, val);
        if self.level < 1.0 - a.low_band {
            hue = a.hue_low + 180.0;
            let k = (((1.0 - a.low_band) - self.level) / a.low_band).clamp(0.0, 1.0);
            sat = a.saturation_low * k;
            val = (a.value_low - a.value) * k + a.value;
        } else {
            hue = a.hue + 180.0;
            sat = (1.0 - ((1.0 - self.level) / a.low_band).clamp(0.0, 1.0)) * a.saturation;
            val = a.value;
        }
        if hue > 360.0 {
            hue -= 360.0;
        }
        let mut c = hsv(hue, sat, val);
        c.w = 1.0;
        // Armco 0x00013940: VISOS_Colour from a vector, each channel `_ftol(x·255)` (asserts <= 1)
        let b = |x: f32| (ftol(x * 255.0) as u32) & 0xFF;
        self.colour = b(c.w) << 24 | b(c.x) << 16 | b(c.y) << 8 | b(c.z);
    }

    /// DangerZoneRender.cpp 0x0011F860 — the clocks (`dt` seconds, 0 while paused) and the warp strip: 16 rings of
    /// `segments` + 1 point pairs, uvs pushed outwards by a wave, rgb reduced by the tint; uvs in texels of `tex`
    pub fn warp_vertices(&mut self, dt: f32, paused: bool, tex: Vec2) -> Vec<FseVertex> {
        let a = self.params;
        let mut t = self.amount;
        if self.amount_override >= 0.0 {
            t = self.amount_override;
        }
        let lerp = |x: f32, y: f32| (y - x) * t + x;
        let (start, mut width);
        if self.success {
            start = lerp(a.success_radius, a.radius[1]);
            width = a.width[1];
        } else {
            start = lerp(a.radius[0], a.radius[1]);
            width = lerp(a.width[0], a.width[1]);
            if a.width_ramp > t {
                width = t / a.width_ramp * width;
            }
        }
        let displace = lerp(a.displace[0], a.displace[1]);
        let freq = lerp(a.frequency[0], a.frequency[1]);
        let spin = if a.spin_from < 1.0 { ((t - a.spin_from) / (1.0 - a.spin_from)).clamp(0.0, 1.0) * a.spin } else { a.spin };
        if !paused {
            self.clock_a = spin * dt + self.clock_a;
            self.clock_b = a.wave_speed * dt + self.clock_b;
        }
        let mut r1 = start;
        let step = width / 16.0;
        let tint = 128.0 * a.tint_scale;
        let n = a.segments;
        let tau = f32::from_bits(0x40C90FDB);
        let white = crate::light_streak::visos_colour([0x80; 4]);
        let mut out = Vec::with_capacity(16 * (n.max(0) as usize + 1) * 2);
        for i in 0..16 {
            let mut ca = white;
            let mut cb = white;
            let f0 = i as f32 / 17.0;
            let f1 = ((i + 1) as f32 / 17.0).min(1.0);
            let w0 = ring_wave(f0 * tau, freq, self.clock_a);
            let w1 = ring_wave(f1 * tau, freq, self.clock_a);
            let d0 = w0 * step * displace;
            let d1 = w1 * step * displace;
            let r0 = r1;
            r1 += step;
            let sub = |c: &mut u32, w: f32| {
                let mut b = c.to_le_bytes();
                b[2] = b[2].wrapping_sub(ftol(tint * a.tint[0] * w) as u8);
                b[1] = b[1].wrapping_sub(ftol(tint * a.tint[1] * w) as u8);
                b[0] = b[0].wrapping_sub(ftol(tint * a.tint[2] * w) as u8);
                *c = u32::from_le_bytes(b);
            };
            if self.stripes {
                let r = if i & 1 != 0 { 0 } else { 0x80 };
                ca = (ca & 0xFF00_FFFF) | r << 16;
                cb = (cb & 0xFF00_FFFF) | r << 16;
            } else {
                if w0 > 0.0 {
                    sub(&mut ca, w0);
                }
                if w1 > 0.0 {
                    sub(&mut cb, w1);
                }
            }
            for j in 0..=n {
                let f = j as f32 / n as f32;
                let s0 = (f * a.wave_count * tau).sin();
                let g = ((f * a.wave_shift + self.clock_b) * tau).sin() * s0;
                let g = g * g;
                let ra = f0 * g * a.wave_size * self.amount + r0;
                let rb = f1 * g * a.wave_size * self.amount + r1;
                let sn = (f * tau).sin();
                let cs = (f * tau).cos();
                let p = Vec2::new(sn * ra + 0.5, cs * ra + 0.5);
                let uv = Vec2::new(d0 * sn + p.x, d0 * cs + p.y);
                out.push(FseVertex { pos: p, uv: uv * tex, colour: ca });
                let p = Vec2::new(sn * rb + 0.5, cs * rb + 0.5);
                let uv = Vec2::new(d1 * sn + p.x, d1 * cs + p.y);
                out.push(FseVertex { pos: p, uv: uv * tex, colour: cb });
            }
        }
        out
    }
}

/// DangerZoneRender.cpp 0x0011F860 — copy the back buffer into a half buffer, draw the warp strip from it over the
/// screen (blend 1)
pub fn render_warp(fse: &mut FullScreenEffects, dz: &mut DangerZone, dt: f32, paused: bool) {
    if !dz.params.warp_on {
        return;
    }
    let Some(half) = fse.get_off_screen_buffer(OFF_SCREEN_HALF) else {
        log::error!("assert: pOffScreenBuffer != NULL (DangerZoneRender.cpp 195)");
        return;
    };
    fse.draw(half, FseImage::BackBuffer, FseShader::DEFAULT, prog::COPY, None, None, 0.0, WHITE);
    let tex = fse.image_size(half).as_vec2();
    let verts = dz.warp_vertices(dt, paused, tex);
    fse.draw_strip(FseImage::BackBuffer, half, DANGER_WARP, prog::COPY, verts);
    fse.release_off_screen_buffer(half);
}

/// DangerZoneRender.cpp 0x00120850 — the "Danger" overlay: the screen through Danger.pxu (grey by the intensity,
/// "spec4Inv" tinted and subtracted) into a half buffer, copied back over the screen
pub fn render_overlay(fse: &mut FullScreenEffects, dz: &mut DangerZone) {
    if !(dz.active && dz.overlay > 0.0) {
        return;
    }
    dz.overlay_colour();
    let Some(half) = fse.get_off_screen_buffer(OFF_SCREEN_HALF) else { return };
    fse.set_vector(gmi::INTENSITY, Vec4::splat(dz.overlay));
    let saved = fse.alpha_texture.take();
    fse.alpha_texture = dz.spec4inv.clone();
    let c = dz.colour;
    let v = |x: f32, y: f32| FseVertex { pos: Vec2::new(x, y), uv: Vec2::new(x, y), colour: c };
    fse.draw_strip(half, FseImage::BackBuffer, DANGER_OVERLAY, prog::DANGER, vec![v(0.0, 0.0), v(1.0, 0.0), v(0.0, 1.0), v(1.0, 1.0)]);
    fse.alpha_texture = saved;
    fse.draw(FseImage::BackBuffer, half, FseShader::DEFAULT, prog::COPY, None, None, 0.0, WHITE);
    // "Default" keeps CreateShader's stencil state (ALWAYS, pass ZERO, write mask all): the whole stencil is zero
    fse.clear_car_stencil();
    fse.release_off_screen_buffer(half);
}

/// debug: `VELOCITY_DANGER_FORCE=<amount>` = a running trail with every record value at that amount
fn forced_input(inp: &DangerZoneInput) -> DangerZoneInput {
    match std::env::var("VELOCITY_DANGER_FORCE").ok().and_then(|v| v.parse::<f32>().ok()) {
        Some(a) => DangerZoneInput { on: true, prev: [a; 5], cur: [a; 5], lerp: 1.0, ..*inp },
        None => *inp,
    }
}

fn spec4inv_image(images: &mut Assets<Image>) -> Option<Handle<Image>> {
    let vcd = visos::file::Vcd::open("\\VFX\\misc.vcd").ok()?;
    let d = crate::renderer::texture_manager::decode_any_vtf(vcd.get("spec4inv.vtf")?).ok()?;
    let mode = bevy::image::ImageAddressMode::ClampToEdge;
    Some(images.add(crate::model_builder::make_image(&d, mode, mode)))
}

/// GameManager 0x000B9010: `DangerZoneRender__120850` early in the view (before the bloom), render flag 0x4000000 —
/// in every player view (not the mirror), inside that view's viewport (the view argument is unused; the copy takes
/// the whole back buffer each time)
fn danger_overlay_system(
    mut fse: ResMut<FullScreenEffects>,
    mut dz: ResMut<DangerZone>,
    input: Res<DangerZoneInput>,
    mut images: ResMut<Assets<Image>>,
    views: crate::view_render::ViewCams,
) {
    if !fse.active {
        return;
    }
    let inp = forced_input(&input);
    dz.enabled = inp.enabled;
    dz.apply_trail(inp.on, inp.success);
    if dz.spec4inv.is_none() && dz.active {
        dz.spec4inv = spec4inv_image(&mut images);
    }
    let restore = fse.viewport();
    for rect in views.rects() {
        fse.set_viewport(rect);
        render_overlay(&mut fse, &mut dz);
    }
    fse.set_viewport(restore);
}

/// the danger blur / warp 0x001205E0 (GameManager 0x000B9E77)
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DangerWarpSet;

/// GameManager 0x000B9010: `DangerZoneRender__1205E0` after the zone FX — record lerp, motion blur (flags 4), warp;
/// in every player view (render flag 0x4000000, not the mirror) inside its viewport, the warp stepping by the whole
/// frame time each call
#[allow(clippy::too_many_arguments)]
fn danger_zone_system(
    time: Res<Time>,
    mut fse: ResMut<FullScreenEffects>,
    mut dz: ResMut<DangerZone>,
    mut mb: ResMut<crate::motion_blur_pc_xbox::MotionBlur>,
    input: Res<DangerZoneInput>,
    mut images: ResMut<Assets<Image>>,
    views: crate::view_render::ViewCams,
) {
    if !fse.active {
        return;
    }
    let inp = forced_input(&input);
    let restore = fse.viewport();
    for rect in views.rects() {
        if !(dz.enabled && dz.active) {
            break;
        }
        fse.set_viewport(rect);
        if inp.on {
            dz.lerp_record(&inp.prev, &inp.cur, inp.lerp);
        }
        let p = dz.params;
        let blur = ((p.blur[1] - p.blur[0]) * dz.fade + p.blur[0]).max(f32::from_bits(0x3727C5AC));
        if dz.flags & 2 != 0 {
            mb.load_spec2(&mut images);
            crate::motion_blur_pc_xbox::render(&mut fse, &mb, 4, blur);
        }
        if dz.flags & 4 != 0 {
            // GameManager 0x000BF060 · 0.001: the frame time, 0 while paused
            let dt = if inp.paused { 0.0 } else { time.delta_secs() };
            render_warp(&mut fse, &mut dz, dt, inp.paused);
        }
    }
    fse.set_viewport(restore);
}

/// `off_6869A4`: the five danger particle types (`LADown\partdef.vcd`)
pub const DANGER_PARTICLES: [&str; 5] = ["chase_debris", "chase_explosion", "chase_sparks", "chase_fire_column_L", "chase_fire_column_R"];
/// `dword_719118` (static init 0x004250F0): offsets in camera space, x / y scaled by the level lerps below
const DANGER_OFFSETS: [Vec3; 5] = [Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, -1.0, 0.0), Vec3::new(-1.0, 0.0, 2.0), Vec3::new(1.0, 0.0, 2.0)];
/// `flt_6869B8..C4`: x scale lerp(7, 3.25, level), y scale lerp(7.5, 3, level); `flt_6869C8` level approach rate
const DANGER_X: [f32; 2] = [7.0, 3.25];
const DANGER_Y: [f32; 2] = [7.5, 3.0];
const DANGER_RATE: f32 = 0.25;
/// `dword_719110`: z override when non-zero (bss, only read: 0)
const DANGER_Z: f32 = 0.0;

/// calls into the danger particle object `byte_719168` (core forwards them)
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub enum DangerParticleCall {
    /// TrailBlazer 0x000C63E0 (+29): false at the intro (0x000C6080), true from the "TriggerBigParticles" trigger
    /// (TriggerUpdate 0x00180640) or a timeline item (0x0017A750, its byte)
    Enable(bool),
    /// TrailBlazer 0x000C4FF0 per update → 0x001234A0(level)
    Level(f32),
    /// TrailBlazer 0x000C60E0 → 0x001238A0
    Stop,
}

/// the danger particle object `byte_719168` (ctor 0x00123450): +0 level, +4 (0), +8 five emitter handles, +28
/// active, +29 enabled
#[derive(Resource, Clone, Debug, Default)]
pub struct DangerParticles {
    pub level: f32,
    pub emitters: [Option<usize>; 5],
    pub active: bool,
    pub enabled: bool,
}

/// CRim 0x000452C0
fn crim_lerp(a: f32, b: f32, t: f32) -> f32 {
    (b - a) * t + a
}

impl DangerParticles {
    /// DangerZoneRender.cpp 0x00123510 — five looping emitters (a failed add returns 385, which the original then
    /// indexes past the pool; kept as no emitter here)
    pub fn start(&mut self, mgr: &mut crate::stateless_particles::stateless_particle_mgr::CStatelessParticleMgr) {
        for (i, name) in DANGER_PARTICLES.iter().enumerate() {
            let h = mgr.add_emitter(visos::handy::handy_crc::crc32(name.as_bytes(), 0));
            self.emitters[i] = (h != crate::stateless_particles::stateless_particle_mgr::NO_EMITTER).then_some(h);
            if let Some(h) = self.emitters[i] {
                let e = &mut mgr.emitters[h];
                e.start();
                e.set_looping(true);
            }
        }
        self.active = true;
        self.level = 0.0;
    }

    /// DangerZoneRender.cpp 0x001234A0 — per game update while the trailblazer runs
    pub fn set_level(&mut self, level: f32, mgr: &mut crate::stateless_particles::stateless_particle_mgr::CStatelessParticleMgr) {
        if level > 0.0 && self.enabled {
            if !self.active {
                self.start(mgr);
            }
            self.level = crim_lerp(self.level, level.sqrt(), DANGER_RATE);
        }
    }

    /// DangerZoneRender.cpp 0x001238A0 — running emitters stop emitting (handles are kept: a freed slot that is
    /// reused keeps being moved by `place` while running, as in the original)
    pub fn stop(&mut self, mgr: &mut crate::stateless_particles::stateless_particle_mgr::CStatelessParticleMgr) {
        for h in self.emitters.iter().flatten() {
            let e = &mut mgr.emitters[*h];
            if e.flags & crate::stateless_particles::stateless_particle_emitter::flags::RUNNING != 0 {
                e.stop();
            }
        }
    }

    pub fn apply(&mut self, c: DangerParticleCall, mgr: &mut crate::stateless_particles::stateless_particle_mgr::CStatelessParticleMgr) {
        match c {
            DangerParticleCall::Enable(on) => self.enabled = on,
            DangerParticleCall::Level(l) => self.set_level(l, mgr),
            DangerParticleCall::Stop => self.stop(mgr),
        }
    }

    /// DangerZoneRender.cpp 0x001235F0 (GameManager 0x000B9E24, every rendered frame before the particle update):
    /// running emitters ride on the camera at their offsets; `active` drops when none runs. Camera rows: +88
    /// right = up × dir (VsBaseCamera; bevy's left), +104 up, +120 dir, +72 position
    pub fn place(&mut self, cam: &GlobalTransform, mgr: &mut crate::stateless_particles::stateless_particle_mgr::CStatelessParticleMgr) {
        if !self.active {
            return;
        }
        let (r, u, f, p) = (cam.left().as_vec3(), cam.up().as_vec3(), cam.forward().as_vec3(), cam.translation());
        let m = Mat4::from_cols(r.extend(0.0), u.extend(0.0), f.extend(0.0), p.extend(1.0));
        let xs = crim_lerp(DANGER_X[0], DANGER_X[1], self.level);
        let ys = crim_lerp(DANGER_Y[0], DANGER_Y[1], self.level);
        let mut any = false;
        for (i, h) in self.emitters.iter().enumerate() {
            let Some(h) = *h else { continue };
            let e = &mut mgr.emitters[h];
            if e.flags & crate::stateless_particles::stateless_particle_emitter::flags::RUNNING == 0 {
                continue;
            }
            let mut o = DANGER_OFFSETS[i];
            o.x *= xs;
            o.y *= ys;
            if DANGER_Z != 0.0 {
                o.z = DANGER_Z;
            }
            // Renderer 0x002ABDE0 (offset · camera matrix, w 1); 0x002D5960 keeps rotation + position
            let pos = m.transform_point3(o);
            e.matrix = Mat4::from_cols(m.x_axis, m.y_axis, m.z_axis, pos.extend(1.0));
            any = true;
        }
        self.active = any;
    }
}

fn danger_particles_system(
    mut calls: MessageReader<DangerParticleCall>,
    mut dp: ResMut<DangerParticles>,
    parts: Option<ResMut<crate::stateless_particles::Particles>>,
    cam: Query<&GlobalTransform, With<TrackCamera>>,
) {
    let Some(mut parts) = parts else { return };
    let mgr = &mut parts.stateless;
    for c in calls.read() {
        dp.apply(*c, mgr);
    }
    if let Some(cam) = cam.iter().next() {
        dp.place(cam, mgr);
    }
}

pub struct DangerZonePlugin;

impl Plugin for DangerZonePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DangerZone>()
            .init_resource::<DangerParticles>()
            .add_message::<DangerParticleCall>()
            .add_systems(Update, danger_particles_system.before(crate::stateless_particles::update_particles).after(crate::stateless_particles::particle_messages))
            .init_resource::<DangerZoneInput>()
            .add_systems(PostUpdate, danger_overlay_system.in_set(FseSet::Build).before(crate::glow_pc_xbox::GlowSet))
            .add_systems(
                PostUpdate,
                danger_zone_system
                    .in_set(FseSet::Build)
                    .in_set(DangerWarpSet)
                    .after(crate::zone_render_pc_xbox::ZoneFxSet)
                    .before(crate::lens_spray_manager::LensSpraySet),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// the five chase types exist in LADown's partdef; start, level approach, camera placement, stop
    #[test]
    fn danger_particles_on_the_camera() {
        let mut p = crate::stateless_particles::Particles::default();
        p.load("\\Tracks\\LADown\\", None);
        let mgr = &mut p.stateless;
        let mut dp = DangerParticles::default();
        dp.apply(DangerParticleCall::Level(1.0), mgr);
        assert!(!dp.active, "gated by +29");
        dp.apply(DangerParticleCall::Enable(true), mgr);
        dp.apply(DangerParticleCall::Level(0.64), mgr);
        assert!(dp.active && dp.emitters.iter().all(Option::is_some), "{:?}", dp.emitters);
        assert_eq!(dp.level, 0.2);
        let cam = GlobalTransform::from(Transform::from_xyz(10.0, 2.0, 5.0));
        dp.place(&cam, mgr);
        let xs = crim_lerp(7.0, 3.25, 0.2);
        // camera looks down -Z: +88 right = up × dir = -X, so fire column L (x -1) sits at +X, 2 m ahead
        let l = mgr.emitters[dp.emitters[3].unwrap()].matrix.w_axis.truncate();
        assert!((l - Vec3::new(10.0 + xs, 2.0, 3.0)).length() < 1e-4, "{l}");
        let d = mgr.emitters[dp.emitters[0].unwrap()].matrix.w_axis.truncate();
        assert!((d - Vec3::new(10.0, 2.0 - crim_lerp(7.5, 3.0, 0.2), 5.0)).length() < 1e-4, "{d}");
        dp.apply(DangerParticleCall::Stop, mgr);
        assert!(dp.emitters.iter().flatten().all(|&h| mgr.emitters[h].stop_time >= 0.0));
    }

    /// 0x0011F3A0 is the textbook hsv conversion
    #[test]
    fn hsv_sectors() {
        assert_eq!(hsv(0.0, 1.0, 1.0), Vec4::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(hsv(120.0, 1.0, 1.0), Vec4::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(hsv(240.0, 1.0, 1.0), Vec4::new(0.0, 0.0, 1.0, 1.0));
        assert_eq!(hsv(360.0, 0.0, 0.5), Vec4::new(0.5, 0.5, 0.5, 1.0));
    }

    /// warp: 16 rings × 31 pairs, inside the stream limit; the clocks move by time, not by frames
    #[test]
    fn warp_strip_and_clocks() {
        let mut a = DangerZone { active: true, amount: 1.0, ..Default::default() };
        let v = a.warp_vertices(0.5, false, Vec2::ONE);
        assert_eq!(v.len(), 16 * 31 * 2);
        assert!(v.len() <= crate::full_screen_effects_manager_pc_xbox::MAX_STRIP_VERTICES);
        let mut b = DangerZone { active: true, amount: 1.0, ..Default::default() };
        for _ in 0..10 {
            b.warp_vertices(0.05, false, Vec2::ONE);
        }
        assert!((a.clock_a - b.clock_a).abs() < 1e-4 && (a.clock_b - b.clock_b).abs() < 1e-4);
        assert_eq!(a.clock_a, -10.0);
        // paused: frozen
        let c0 = b.clock_b;
        b.warp_vertices(0.0, true, Vec2::ONE);
        assert_eq!(b.clock_b, c0);
    }

    /// TrailBlazer off resets what the overlay reads
    #[test]
    fn trail_off_resets() {
        let mut d = DangerZone::default();
        d.apply_trail(true, false);
        d.lerp_record(&[0.0; 5], &[1.0, 0.5, 0.8, 0.6, 0.9], 0.5);
        assert_eq!(d.overlay, 0.4);
        assert_eq!(d.amount, 0.45);
        d.apply_trail(false, false);
        assert!(!d.active && d.overlay == 0.0 && d.fade_inv == 1.0);
    }

    /// in a race without replay / cut scene the game object draws nothing
    #[test]
    fn gate() {
        let mut d = DepthOfField::new(false);
        assert!(!d.wants_draw(&DepthOfFieldInput::default()));
        assert!(d.wants_draw(&DepthOfFieldInput { replay: true, replay_camera_type: 1, ..default() }));
        assert!(d.car_focus);
    }

    /// planes bracket the focus: near ones in front, far ones behind, alphas within the layer table
    #[test]
    fn planes_bracket_focus() {
        let mut d = DepthOfField::new(false);
        d.set_focus(20.0, false, false);
        let ok = d.update(&DepthOfFieldInput::default(), Vec3::ZERO, 1.0);
        assert!(ok);
        let focus = d.focus - d.focus_offset;
        for i in 0..3 {
            if d.near_alpha[i] != 0 {
                assert!(d.near_dist[i] < focus, "{:?}", d.near_dist);
            }
            if d.far_alpha[i] != 0 {
                assert!(d.far_dist[i] > focus, "{:?}", d.far_dist);
            }
            assert!(d.near_alpha[i] <= LAYER_ALPHA[i] && d.far_alpha[i] <= LAYER_ALPHA[i]);
        }
    }
}

/// DangerZoneRender.cpp 0x001214A0..0x00122FA9 — the dynamic envmap object (`byte_709F00` = CarRender +56: +0 render
/// target, +4 saved target, +8..+28 six shaders, +32 "Spec2", +36 "lightglow_white", +48 "fresnel4", +52 shader
/// index 1, +56 hi-res state, +57 enabled, +58 drawn this car pass). Drawn by CarShaderManager_PC_XBOX.cpp
/// (`car_shader_manager_pc_xbox`).
pub mod dynamic_envmap {
    use bevy::prelude::*;

    use crate::full_screen_effects_manager_pc_xbox::FseVertex;
    use crate::light_streak::visos_colour;
    use crate::vs_render::PointLight;

    /// `dword_686924` / `dword_686928` at boot: the target is first made 512² (0x001214A0)
    pub const SIZE_LOG2_BOOT: u32 = 9;
    /// 0x001598A0: 128² normally, 256² in camera mode 3 or with the "hi-res" tweak (on)
    pub const SIZE_LOG2_LOW: u32 = 7;
    pub const SIZE_LOG2_HIGH: u32 = 8;
    /// `dword_686948` = 0x7F: 1 clear (0x002EED60 is empty in this build), 8 specular spots, 0x10 subtract, 0x20 fresnel
    pub const FLAGS: u32 = 0x7F;
    pub const FLAG_SPOTS: u32 = 8;
    pub const FLAG_SUBTRACT: u32 = 0x10;
    pub const FLAG_FRESNEL: u32 = 0x20;
    /// `byte_687570`: the band is squashed towards the projected horizon
    pub const HORIZON: bool = true;
    /// `dword_687568` / `dword_68756C`: the inner disc and the outer band are drawn
    pub const RING_DISC: bool = true;
    pub const RING_BAND: bool = true;
    /// `dword_687560` segments, `dword_687564` disc radius / band inner radius, `dword_68755C` band outer radius
    pub const RING_SEGMENTS: u32 = 32;
    pub const RING_MID: f32 = f32::from_bits(0x3E99_999A);
    pub const RING_OUTER: f32 = f32::from_bits(0x3F07_AE14);
    /// `dword_68699C`: v radius of the band's outer edge in the source
    pub const BAND_V: f32 = f32::from_bits(0x3E4C_CCCD);
    /// `byte_686998` brightness ramp by height, `byte_686999` horizon squash
    pub const RAMP: bool = true;
    pub const SQUASH: bool = true;
    /// `dword_686930`: the squash factor is at least 1 / 2.5
    pub const SQUASH_MIN_DIV: f32 = 2.5;
    /// `dword_686968`: the lookat point is the car position lowered by 0.16
    pub const LOOK_DROP: f32 = f32::from_bits(0xBE23_D70A);
    /// `dword_68696C` / `dword_686970` / `dword_686974` / `dword_686978`: sphere map scale / offset
    pub const MAP_SCALE: f32 = 0.5;
    pub const MAP_OFFSET: f32 = 0.5;
    /// `byte_68697C`: the player car's normals are bent towards the radial direction once (0x00121BB0)
    pub const BEND: bool = true;
    /// `flt_686980`
    pub const BEND_AMOUNT: f32 = f32::from_bits(0x3E19_999A);
    /// specular spots (0x00121ED0): `flt_686994` min height above the car, `dword_686990` range (≠ 0, so the lights'
    /// radii only gate), `dword_68698C` height scale, `dword_686988` fade scale, `dword_686984` size falloff
    pub const SPOT_MIN_HEIGHT: f32 = 1.0;
    pub const SPOT_RANGE: f32 = 60.0;
    pub const SPOT_Y_SCALE: f32 = 1.5;
    pub const SPOT_FADE: f32 = 3.0;
    pub const SPOT_FALLOFF: f32 = f32::from_bits(0x3C4C_CCCD);
    pub const MAX_SPOTS: usize = 32;
    /// 0x0015A090: `dword_687574` / `dword_687578` stretch of spots below the horizon
    pub const SPOT_STRETCH_W: f32 = 2.5;
    pub const SPOT_STRETCH_H: f32 = 0.25;

    /// "Graphics.Dynamic Envmap" / ".Specular" tweaks (0x00122AE0) at their static values
    #[derive(Clone, Debug)]
    pub struct EnvmapTweaks {
        /// +57 "enabled" is the object's own flag; `byte_718E78` "display" (bss: off)
        pub display: bool,
        /// `byte_686944` "hi-res" = 1
        pub hi_res: bool,
        /// `dword_686954` "blur" = 1 (the blur pass count passed is always 1)
        pub blur: u32,
        /// `flt_686938` / `flt_68693C` / `flt_686940` "top / mid / bot bright", `flt_686934` "global bright"
        pub top_bright: f32,
        pub mid_bright: f32,
        pub bot_bright: f32,
        pub global_bright: f32,
        /// `dword_68694C` "subtract" = 0x20, `byte_686950` "fresnel" = 0x70
        pub subtract: u32,
        pub fresnel: u8,
        /// ".Specular": `dword_68692C` size 0.25, `dword_686958` bright 0.75, `dword_686964` bloom 1.2, `flt_68695C`
        /// fresnel 0.8, `byte_686960` extra bloom fresnel 1. ("hi-contrast", "greyscale", "desaturate" have no reader.)
        pub spec_size: f32,
        pub spec_bright: f32,
        pub spec_bloom: f32,
        pub spec_fresnel: f32,
        pub extra_bloom_fresnel: bool,
    }

    impl Default for EnvmapTweaks {
        fn default() -> Self {
            Self {
                display: false,
                hi_res: true,
                blur: 1,
                top_bright: 0.75,
                mid_bright: 0.5,
                bot_bright: f32::from_bits(0x3DCC_CCCD),
                global_bright: 1.0,
                subtract: 0x20,
                fresnel: 0x70,
                spec_size: 0.25,
                spec_bright: 0.75,
                spec_bloom: f32::from_bits(0x3F99_999A),
                spec_fresnel: f32::from_bits(0x3F4C_CCCD),
                extra_bloom_fresnel: true,
            }
        }
    }

    /// CRim 0x000452C0 — (b − a)·t + a
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        (b - a) * t + a
    }

    /// VsVector4::Normalise (zero below 1e-6)
    fn normalise(v: Vec3) -> Vec3 {
        let l = v.length();
        if l <= 1e-6 { Vec3::ZERO } else { v / l }
    }

    /// `_ftol` to a colour byte
    fn ftol_u8(v: f32) -> u8 {
        (v as i32) as u8
    }

    /// DangerZoneRender.cpp 0x00122400 — one ring of the envmap as a strip of `segments + 1` (outer, inner) pairs:
    /// positions on circles of radius `outer` (clamped to 0..1) / `inner` round the centre, uv in texels of the
    /// source (`size`) — the disc (`band` false) maps the screen circle of radius 0.5 onto itself with the centre
    /// at the screen centre, the band maps the inner circle's x to the outer edge; `horizon` (0..1 screen y of the
    /// horizon) squashes v about 0.5 (`SQUASH`); vertex colours VISOS(alpha, b, b, b) with b the brightness ramp
    /// by the vertex height (`RAMP`), alpha = global bright − 1
    pub fn ring_strip(t: &EnvmapTweaks, segments: u32, inner: f32, outer: f32, band: bool, horizon: f32, size: Vec2) -> Vec<FseVertex> {
        let step = 6.2831855f32 / segments as f32;
        let mut a = 0.0f32;
        let (sx, sy) = (1.0f32, 1.0f32);
        // VsFrameTimer 0x00057F90 = max
        let h = (horizon * 2.0).max(1.0 / SQUASH_MIN_DIV);
        let mut v = Vec::with_capacity(2 * segments as usize + 2);
        for _ in 0..=segments {
            let c = a.cos();
            let s = a.sin();
            let mut ox = (0.5 - s * outer).clamp(0.0, 1.0);
            let mut oy = (0.5 - c * outer).clamp(0.0, 1.0);
            let mut ix = 0.5 - s * inner;
            let mut iy = 0.5 - c * inner;
            // (u0, v0) of the outer vertex, (u1, v1) of the inner one
            let (u0, mut v0, u1, mut v1) = if band {
                (ix, 0.5 - c * BAND_V, ox, 0.5 - c * 0.5)
            } else {
                (0.5 - s * 0.5, 0.5 - c * 0.5, 0.5, 0.5)
            };
            if SQUASH {
                if oy >= 0.5 {
                    v0 = v0 * (2.0 - h) - (1.0 - h);
                    v1 = v1 * (2.0 - h) - (1.0 - h);
                } else {
                    v0 *= h;
                    v1 *= h;
                }
            }
            let g = t.global_bright.clamp(0.0, 1.0);
            let alpha = ftol_u8((t.global_bright - 1.0).clamp(0.0, 1.0) * 255.0);
            let mut c0 = visos_colour([alpha, 0x80, 0x80, 0x80]);
            let mut c1 = c0;
            if RAMP {
                let ramp = |y: f32| {
                    if y >= 0.5 { lerp(t.mid_bright, t.bot_bright, y * 2.0 - 1.0) } else { lerp(t.top_bright, t.mid_bright, y * 2.0) }
                };
                let b = ftol_u8(ramp(oy) * 255.0 * g);
                c0 = visos_colour([alpha, b, b, b]);
                let b = ftol_u8(ramp(iy) * 255.0 * g);
                c1 = visos_colour([alpha, b, b, b]);
            }
            ox *= sx;
            oy *= sy;
            ix = (ix * sx).clamp(0.0, sx);
            iy = (iy * sy).clamp(0.0, sy);
            v.push(FseVertex { pos: Vec2::new(ox, oy), uv: Vec2::new(u0 * size.x, v0 * size.y), colour: c0 });
            v.push(FseVertex { pos: Vec2::new(ix, iy), uv: Vec2::new(u1 * size.x, v1 * size.y), colour: c1 });
            a += step;
        }
        v
    }

    /// one specular spot of 0x00121ED0 (24-byte record): sphere-map position (u, v, s), D3DCOLOR, size
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct EnvSpot {
        pub pos: Vec3,
        pub colour: u32,
        pub size: f32,
    }

    /// DangerZoneRender.cpp 0x00121ED0 — the point lights (`dword_74A434` +296, +292 count) as sphere-map spots
    /// round the car: lights more than 1 m above `car` and within 60 m (xz) of it; direction with y × 1.5 into view
    /// space (`view` = D3DTS_VIEW rows 0..2 — x right, y up, z forward — GetTransform 0 via 0x0012D2C0), z negated,
    /// sphere map u = x / 2s, v = y / 2s, s = √((z + 1)/2); colour = light colour · fade · lerp(1, 1 − s, fresnel) ·
    /// bright, alpha = fade · bloom · (1 − s) (² with "extra bloom fresnel"); at most 32
    pub fn specular_spots(t: &EnvmapTweaks, lights: &[PointLight], car: Vec3, view: [Vec3; 3]) -> Vec<EnvSpot> {
        let mut out = Vec::new();
        for l in lights {
            if l.radius == 0.0 {
                continue;
            }
            let mut d = l.pos - car;
            if d.y <= SPOT_MIN_HEIGHT {
                continue;
            }
            let x2 = d.x * d.x;
            let dxz = d.z * d.z + x2;
            let r = if SPOT_RANGE == 0.0 { l.radius } else { SPOT_RANGE };
            if r * r <= dxz {
                continue;
            }
            d.y *= SPOT_Y_SCALE;
            let dist = dxz.sqrt();
            let d = normalise(d);
            let fade = ((1.0 - dist / r) * SPOT_FADE).min(1.0);
            let size = 1.0 / (dist * SPOT_FALLOFF + 1.0);
            // Renderer 0x002ABF70: row-vector 3x3
            let mut v = view[0] * d.x + view[1] * d.y + view[2] * d.z;
            v.z = 0.0 - v.z;
            let s = ((v.z + 1.0) * 0.5).sqrt();
            let u = v.x / (2.0 * s);
            let w = v.y / (2.0 * s);
            let fres = 1.0 - s.abs();
            let f = lerp(1.0, fres, t.spec_fresnel);
            let c = l.colour.truncate() * (fade * f * t.spec_bright);
            let cr = (c.x * 255.0).min(255.0);
            let cg = (c.y * 255.0).min(255.0);
            let cb = (c.z * 255.0).min(255.0);
            let mut a = fade * t.spec_bloom * fres;
            if t.extra_bloom_fresnel {
                a *= fres;
            }
            let colour = visos_colour([ftol_u8((a * 255.0).min(255.0)), ftol_u8(cr), ftol_u8(cg), ftol_u8(cb)]);
            out.push(EnvSpot { pos: Vec3::new(u, w, s), colour, size });
            if out.len() >= MAX_SPOTS {
                break;
            }
        }
        out
    }

    /// CarShaderManager_PC_XBOX.cpp 0x0015A090 — a spot's quad (0..1 of the envmap, uv 0/1): size · spec size,
    /// v negated, below the centre wider and flatter
    pub fn spot_quad(t: &EnvmapTweaks, s: &EnvSpot) -> [FseVertex; 4] {
        let sz = t.spec_size * s.size;
        let (mut w, mut h) = (sz, sz);
        let y = 0.0 - s.pos.y;
        if y < 0.0 {
            let y2 = y * y;
            w *= y2 * SPOT_STRETCH_W + 1.0;
            h /= y2 * SPOT_STRETCH_H + 1.0;
        }
        let x0 = s.pos.x * 0.5 + 0.5 - w * 0.5;
        let y0 = y * 0.5 + 0.5 - h * 0.5;
        let v = |x: f32, y: f32, u: f32, vv: f32| FseVertex { pos: Vec2::new(x, y), uv: Vec2::new(u, vv), colour: s.colour };
        [v(x0, y0, 0.0, 0.0), v(x0 + w, y0, 1.0, 0.0), v(x0, y0 + h, 0.0, 1.0), v(x0 + w, y0 + h, 1.0, 1.0)]
    }

    /// row-major d3d 4x4 product a·b
    fn mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
        let mut r = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                r[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j];
            }
        }
        r
    }

    /// D3DXMatrixLookAtLH (sub_48A0F9): z = |at − eye|, x = |up × z|, y = z × x
    pub fn look_at_lh(eye: Vec3, at: Vec3, up: Vec3) -> [[f32; 4]; 4] {
        let z = normalise(at - eye);
        let x = normalise(up.cross(z));
        let y = z.cross(x);
        [
            [x.x, y.x, z.x, 0.0],
            [x.y, y.y, z.y, 0.0],
            [x.z, y.z, z.z, 0.0],
            [0.0 - x.dot(eye), 0.0 - y.dot(eye), 0.0 - z.dot(eye), 1.0],
        ]
    }

    /// DangerZoneRender.cpp 0x001218A0 + CarPalette.cpp 0x0010CE90 — envmapMatrix registers while the dynamic
    /// envmap is on: car rotation · lookat from the camera to the car position lowered by 0.16 (camera up), the
    /// translation row cleared, then x/y scaled by −0.5 about 0.5 (z dropped), transposed for upload. `car` = the
    /// car's world transform (car data +0), `eye` / `up` = camera +72 / +104. (`byte_7190F0` — car · view instead of
    /// the lookat — is bss, off.)
    pub fn envmap_matrix(car: Mat4, eye: Vec3, up: Vec3) -> [Vec4; 4] {
        let c = car.to_cols_array_2d();
        let mut at = Vec3::new(c[3][0], c[3][1], c[3][2]);
        at.y += LOOK_DROP;
        let l = look_at_lh(eye, at, up);
        let mut m45 = mul(&c, &l);
        m45[3] = [0.0, 0.0, 0.0, 1.0];
        let m27 = [
            [0.0 - MAP_SCALE, 0.0, 0.0, 0.0],
            [0.0, 0.0 - MAP_SCALE, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [MAP_OFFSET, MAP_OFFSET, 0.0, 1.0],
        ];
        let m = mul(&m45, &m27);
        // Renderer 0x002ABD60 = D3DXMatrixTranspose: register k = column k
        [0, 1, 2, 3].map(|k| Vec4::new(m[0][k], m[1][k], m[2][k], m[3][k]))
    }

    /// DangerZoneRender.cpp 0x00121C80 — one vertex of the normal bend (0x00121BB0, once, the player car's chassis
    /// and fitted parts at every LOD): the normal moved 0.15 of the way to the direction of the (model-space)
    /// position, normalised
    pub fn bend_normal(pos: Vec3, n: Vec3) -> Vec3 {
        let d = normalise(pos);
        normalise((d - n) * BEND_AMOUNT + n)
    }

    /// CarShaderManager_PC_XBOX.cpp 0x00159D00 — the screen height of the horizon behind the car: the camera
    /// direction with y = the car's ground slope (+932 y; x, z scaled by (1 − y²)/(x² + z²), not its root —
    /// original), 10 000 m out from the car, projected (`project_y` = 0x0015EC80 y, 0..1 down), clamped
    pub fn horizon(car: Vec3, cam_fwd: Vec3, ground: Vec3, project_y: impl Fn(Vec3) -> f32) -> f32 {
        let mut d = cam_fwd;
        d.y = ground.y;
        let k = (1.0 - ground.y * ground.y) / (d.z * d.z + d.x * d.x);
        d.x *= k;
        d.z *= k;
        project_y(car + d * 10000.0).clamp(0.0, 1.0)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn rings() {
            let t = EnvmapTweaks::default();
            let disc = ring_strip(&t, RING_SEGMENTS, 0.0, RING_MID, false, 0.5, Vec2::new(320.0, 240.0));
            assert_eq!(disc.len(), 66);
            // segment 0 (a = 0): outer vertex straight up at 0.5 − 0.3, sampling the top of the screen circle
            assert!((disc[0].pos - Vec2::new(0.5, 0.2)).length() < 1e-6, "{:?}", disc[0]);
            assert!((disc[0].uv - Vec2::new(160.0, 0.0)).length() < 1e-3, "{:?}", disc[0]);
            assert_eq!(disc[1].pos, Vec2::new(0.5, 0.5));
            assert_eq!(disc[1].uv, Vec2::new(160.0, 120.0));
            // top bright 0.75 · 255 = 191 doubled saturates; alpha = global bright − 1 = 0
            assert_eq!(disc[0].colour, 0x00FF_FFFF);
            // half way round (a = π): the bottom, bot..mid ramp at y = 0.8
            let b = &disc[32];
            assert!((b.pos.y - 0.8).abs() < 1e-5);
            let band = ring_strip(&t, RING_SEGMENTS, RING_MID, RING_OUTER, true, 0.5, Vec2::ONE);
            // 0.5 − 0.53 clamps to the top edge
            assert_eq!(band[0].pos.y, 0.0);
            assert!((band[0].uv.y - (0.5 - BAND_V)).abs() < 1e-6);
            assert!((band[1].uv.y - 0.0).abs() < 1e-6);
        }

        #[test]
        fn spots() {
            let t = EnvmapTweaks::default();
            let view = [Vec3::X, Vec3::Y, Vec3::Z];
            let lights = [
                PointLight { pos: Vec3::new(0.0, 0.5, 10.0), radius: 10.0, colour: Vec4::ONE },
                PointLight { pos: Vec3::new(0.0, 10.0, 0.0), radius: 0.0, colour: Vec4::ONE },
                PointLight { pos: Vec3::new(0.0, 10.0, 70.0), radius: 10.0, colour: Vec4::ONE },
                PointLight { pos: Vec3::new(0.0, 10.0, 0.0), radius: 10.0, colour: Vec4::ONE },
            ];
            let s = specular_spots(&t, &lights, Vec3::ZERO, view);
            assert_eq!(s.len(), 1);
            // straight up: view z 0 → s = √0.5, u 0, v = 1/(2s)
            let sp = s[0];
            assert!((sp.pos.z - 0.5f32.sqrt()).abs() < 1e-6);
            assert!(sp.pos.x.abs() < 1e-6 && (sp.pos.y - 1.0 / (2.0 * 0.5f32.sqrt())).abs() < 1e-6);
            assert_eq!(sp.size, 1.0);
            let q = spot_quad(&t, &sp);
            assert!((q[1].pos.x - q[0].pos.x - 0.25 * (sp.pos.y * sp.pos.y * 2.5 + 1.0)).abs() < 1e-5);
        }

        #[test]
        fn matrix_maps_camera_facing_normal_to_centre() {
            // camera 10 m behind the car on -z: a normal facing the camera samples the centre
            let car = Mat4::from_translation(Vec3::new(0.0, 0.16, 0.0));
            let m = envmap_matrix(car, Vec3::new(0.0, 0.0, -10.0), Vec3::Y);
            let n = Vec4::new(0.0, 0.0, -1.0, 1.0);
            assert!((n.dot(m[0]) - 0.5).abs() < 1e-5 && (n.dot(m[1]) - 0.5).abs() < 1e-5);
            // up → top (v < 0.5)
            let up = Vec4::new(0.0, 1.0, 0.0, 1.0);
            assert!((up.dot(m[1]) - 0.0).abs() < 1e-5, "{}", up.dot(m[1]));
        }

        #[test]
        fn bend() {
            let n = bend_normal(Vec3::new(2.0, 0.0, 0.0), Vec3::Y);
            assert!((n - Vec3::new(0.15, 0.85, 0.0).normalize()).length() < 1e-6);
        }
    }
}
