//! The radial motion blur of a race view (functions 0x00160120..0x00161050, after GameShaderProgramManager in the
//! Render\Pc_xbox object order; file name not in the binary — shader names "MotionBlurBlit" etc.), plus its amount
//! (0x0012FB60, in the LightStreak.cpp range, tweak block "Graphics.Motion Blur").
//!
//! Per frame GameManager 0x000B9010 asks 0x0012FB60 for this view's amount (speed ramp, slipstream boost, faded out
//! by Fury and the zone) and, when non-zero, 0x00160FF0 draws it: the back buffer into a half-size buffer with
//! PointBlur (four taps towards the focus point), then back over the screen with PointBlurWithAlpha — alpha =
//! (1 - "spec2".a) · intensity, z test at D3D z 0.995 so near geometry (the player's car) stays sharp.

use bevy::prelude::*;

use crate::full_screen_effects_manager_pc_xbox::{prog, FseImage, FseSet, FseShader, FullScreenEffects, FSE_STENCIL_NOT_CAR, OFF_SCREEN_HALF};
use crate::global_mat_input_resolver::gmi;
use crate::track_render::TrackCamera;

/// "Graphics.Motion Blur" tweaks (0x00686C20..0x00686C38) and the debug force flag `byte_71FF38`
#[derive(Clone, Copy, Debug)]
pub struct MotionBlurTweaks {
    pub on: bool,                 // byte_686C20
    pub force: bool,              // byte_71FF38
    pub min_speed: f32,           // 0x00686C24 mph: blur starts
    pub full_speed: f32,          // 0x00686C28 mph: factor 1
    pub max_blur_speed: f32,      // 0x00686C2C "Max Blur Speed"
    pub max_blur_amount: f32,     // 0x00686C30 "Max Blur Amount"
    pub blur_depth: f32,          // 0x00686C34 "Blur Depth"
    pub slipstream_extra: f32,    // 0x00686C38 "Slipstream Extra Blur Depth"
    pub slip_min: f32,            // 0x006870D4
    pub slip_max: f32,            // 0x006870D8
}

impl Default for MotionBlurTweaks {
    fn default() -> Self {
        Self {
            on: true,
            force: false,
            min_speed: f32::from_bits(0x42820000),
            full_speed: f32::from_bits(0x42f00000),
            max_blur_speed: f32::from_bits(0x43020000),
            max_blur_amount: f32::from_bits(0x3bf5c28f),
            blur_depth: f32::from_bits(0x41a00000),
            slipstream_extra: f32::from_bits(0x42480000),
            slip_min: f32::from_bits(0x3e800000),
            slip_max: f32::from_bits(0x3f400000),
        }
    }
}

/// what core supplies per update for view 0 (GameManager +588 view, vehicle `+28`)
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct MotionBlurInput {
    /// GameManager 0x000B9010 gate: render flag 0x10, `byte_6F7157 == 0`, no replay (`FontHelper__B36B0`),
    /// `dword_749110 == 0`
    pub active: bool,
    /// VsAIRaceVehicle 0x00024870 speed (m/s; ×2.232 = mph)
    pub speed: f32,
    /// VehicleEntity +0x44 (PhysicsManager vehicle array, `LightStreak__130070`)
    pub ve_44: f32,
    /// FuryFX 0x00128680 for the player
    pub fury: f32,
    /// ZoneRender 0x00154FC0 for the player (zone amount · 0.5)
    pub zone: f32,
    /// PhysicsCarInput 0x001BA270 position and VehicleEntity 0x001C66C0 velocity
    pub position: Vec3,
    pub velocity: Vec3,
    /// all four `LightStreak__133970` wheel tests passed
    pub wheels_down: bool,
}

/// the view's blur fields (view +20 factor, +24 amount, +28 depth, +44 velocity, +60 focus)
#[derive(Resource, Debug)]
pub struct MotionBlur {
    pub tweaks: MotionBlurTweaks,
    pub factor: f32,
    pub amount: f32,
    pub depth: f32,
    pub velocity: Vec3,
    pub focus: Vec4,
    /// "spec2" from `\VFX\misc.vcd` (0x00160BA0 loads it into view +80)
    spec2: Option<Handle<Image>>,
}

impl Default for MotionBlur {
    fn default() -> Self {
        let mut tweaks = MotionBlurTweaks::default();
        // debug: the tweak's force flag from the environment (verification shots)
        tweaks.force = std::env::var_os("VELOCITY_MOTION_BLUR_FORCE").is_some();
        Self { tweaks, factor: 0.0, amount: 0.0, depth: 0.0, velocity: Vec3::ZERO, focus: Vec4::new(0.5, 0.5, 0.0, 1.0), spec2: None }
    }
}

impl MotionBlur {
    /// GameShaderProgramManager_PC_XBOX.cpp 0x00160BA0 loads "spec2" on every draw; once here
    pub fn load_spec2(&mut self, images: &mut Assets<Image>) {
        if self.spec2.is_none() {
            self.spec2 = spec2_image(images);
        }
    }

    /// LightStreak.cpp range 0x0012FB60 — this view's blur amount; true when it should be drawn
    pub fn update_amount(&mut self, inp: &MotionBlurInput) -> bool {
        let t = self.tweaks;
        self.factor = 0.0;
        if !t.on {
            return false;
        }
        self.amount = 0.0;
        if t.force {
            self.amount = t.max_blur_amount;
            self.factor = 1.0;
        } else {
            let mph = inp.speed * 2.232;
            self.factor = if t.min_speed > mph {
                0.0
            } else if t.full_speed <= mph {
                1.0
            } else {
                (mph - t.min_speed) / (t.full_speed - t.min_speed)
            };
            self.amount = if t.max_blur_speed <= mph { t.max_blur_amount } else { mph / t.max_blur_speed * t.max_blur_amount };
        }
        self.depth = t.blur_depth;
        let s = inp.ve_44;
        if s > t.slip_min {
            let g = if s <= t.slip_max { (s - t.slip_min) / (t.slip_max - t.slip_min) } else { 1.0 };
            self.factor = g.max(self.factor);
            self.depth = g * t.slipstream_extra + self.depth;
        }
        if self.factor > 0.0 {
            let fury = inp.fury + inp.fury;
            let zone = inp.zone + inp.zone;
            self.factor = (1.0 - fury.max(zone)) * self.factor;
        }
        self.factor > 0.0
    }

    /// GameShaderProgramManager_PC_XBOX.cpp 0x00160920 — the focus: car position plus (last grounded) velocity · 1000, projected; x and y clamped to
    /// [-0.5, 1.5] (`flt_687618`)
    pub fn update_focus(&mut self, inp: &MotionBlurInput, clip_from_world: &Mat4) {
        if inp.wheels_down {
            self.velocity = inp.velocity;
        }
        let p = inp.position + self.velocity * f32::from_bits(0x447a0000);
        let c = *clip_from_world * p.extend(1.0);
        // 0x0015EC80: x/w and -y/w to 0..1, z/w
        let mut f = Vec4::new((c.x / c.w + 1.0) / 2.0, (-c.y / c.w + 1.0) / 2.0, c.z / c.w, c.w);
        let lim = f32::from_bits(0x3fc00000);
        if f.x > lim {
            f.x = lim;
        } else if 1.0 - lim > f.x {
            f.x = (0.0 - lim) + 1.0;
        }
        if f.y > lim {
            f.y = lim;
        } else if 1.0 - lim > f.y {
            f.y = (0.0 - lim) + 1.0;
        }
        self.focus = f;
    }
}

/// GameShaderProgramManager_PC_XBOX.cpp 0x00160BA0 — draw the blur with amount `a5`; flag 8 picks the z-tested composite ("MotionBlurStencilToScreenBlit":
/// blend 2, z test, not over the cars' stencil bit 4), else "MotionBlurToScreenBlit" (blend 2, no z)
pub fn render(fse: &mut FullScreenEffects, mb: &MotionBlur, flags: u32, a5: f32) -> Option<FseImage> {
    render_from(fse, mb, FseImage::BackBuffer, flags, a5)
}

/// GameShaderProgramManager_PC_XBOX.cpp 0x00160BA0 proper: the source and composite target is the current render target (`RenderTargetManager__2EEFE0`)
/// — the back buffer for the race view, the zone FX's grey half buffer for 0x00162D10. Returns the released half
/// buffer that holds the blurred image (`dword_748430`, read by the slipstream distortion 0x001629B0).
pub fn render_from(fse: &mut FullScreenEffects, mb: &MotionBlur, current: FseImage, flags: u32, a5: f32) -> Option<FseImage> {
    let half = fse.get_off_screen_buffer(OFF_SCREEN_HALF)?;
    let k = f32::from_bits(0x3f99999a); // flt_687628
    let mut i = a5 * 120.0;
    i = (1.0 - (mb.focus.x - 0.5).abs() * k) * i;
    i = if i <= 0.0 { 0.0 } else { i };
    i = (1.0 - (mb.focus.y - 0.5).abs() * k) * i;
    i = if i <= 0.0 { 0.0 } else { i };
    fse.set_vector(gmi::INTENSITY, Vec4::splat(i));
    fse.set_vector(gmi::FOCUS_POINT, Vec4::new(mb.focus.x, mb.focus.y, 0.0, 0.0));
    let step = f32::from_bits(0xbca3d70a); // dword_687624
    fse.set_vertex_constant(4, Vec4::new(-0.5, step, step * 2.0, step * 3.0));
    fse.set_vertex_constant(5, Vec4::new(1.0, -1.0, 0.0, 0.0));
    // the view rectangle over the back buffer: the whole screen for one player
    let rect = [0.0, 0.0, 1.0, 1.0];
    // "MotionBlurBlit": blend 1, clamp, z off
    fse.draw(half, current, FseShader::new(1, false), prog::POINT_BLUR, Some(rect), None, 0.0, 0xFFFF_FFFF);
    fse.alpha_texture = mb.spec2.clone();
    let shader = if flags & 8 != 0 {
        // "MotionBlurStencilToScreenBlit": NOTEQUAL ref 4 mask 4, stencil write mask 0
        fse.car_stencil();
        FseShader { stencil: FSE_STENCIL_NOT_CAR, ..FseShader::new(2, true) }
    } else {
        FseShader::new(2, false)
    };
    fse.draw(current, half, shader, prog::POINT_BLUR_WITH_ALPHA, None, None, f32::from_bits(0x3f7eb852), 0xFFFF_FFFF);
    fse.alpha_texture = None;
    fse.release_off_screen_buffer(half);
    Some(half)
}

fn spec2_image(images: &mut Assets<Image>) -> Option<Handle<Image>> {
    let vcd = visos::file::Vcd::open("\\VFX\\misc.vcd").ok()?;
    let d = crate::renderer::texture_manager::decode_any_vtf(vcd.get("spec2.vtf")?).ok()?;
    let mode = bevy::image::ImageAddressMode::ClampToEdge;
    Some(images.add(crate::model_builder::make_image(&d, mode, mode)))
}

/// GameManager 0x000B9010: `if (LightStreak__12FB60(view)) 0x00160FF0(view)` — 0x00160FF0 = the slipstream ribbons
/// 0x0013FDB0 (drawn by race_effects), blur with flags 8 and the view's amount, then the slipstream distortion
/// 0x001629B0 over the blurred half buffer (`dword_748430`)
#[allow(clippy::too_many_arguments)]
fn motion_blur_system(
    mut fse: ResMut<FullScreenEffects>,
    mut mb: ResMut<MotionBlur>,
    input: Option<Res<MotionBlurInput>>,
    mut images: ResMut<Assets<Image>>,
    cam: Query<(&Camera, &GlobalTransform, Option<&bevy::camera::primitives::Frustum>), With<TrackCamera>>,
    effects: Option<Res<crate::race_effects::RaceEffects>>,
    planes: Option<Res<crate::full_screen_effects_manager_pc_xbox::FseCameraPlanes>>,
    split: Option<Res<crate::view_render::SplitScreen>>,
    fury: Option<Res<crate::fury_fx::FuryFx>>,
) {
    // render flag 0x10 && `byte_6F7157 == 0`: no motion blur (nor its forced flag, read inside 0x0012FB60) for two
    // players; the slipstream ribbons 0x0013FDB0 draw instead (race_effects)
    if !fse.active || split.is_some_and(|s| s.on()) {
        return;
    }
    let mut inp = input.map(|i| *i).unwrap_or_default();
    // 0x0012FB60 → FuryFX 0x00128680(view +28): this frame's fury state (the original reads last frame's, its
    // update 0x00128E40 runs after the blur)
    if let Some(f) = fury.as_deref().filter(|f| f.initialised) {
        inp.fury = f.view_amount(0);
    }
    if mb.tweaks.force && !inp.active {
        inp.active = true;
    }
    if !inp.active || !mb.update_amount(&inp) {
        return;
    }
    mb.load_spec2(&mut images);
    let Some((camera, gt, frustum)) = cam.iter().next() else { return };
    let clip_from_world = camera.clip_from_view() * gt.to_matrix().inverse();
    if inp.position != Vec3::ZERO || inp.velocity != Vec3::ZERO {
        mb.update_focus(&inp, &clip_from_world);
    }
    let amount = mb.amount;
    let Some(half) = render(&mut fse, &mb, 8, amount) else { return };
    let Some(fx) = effects.filter(|f| f.active) else { return };
    // the original's five camera planes (VsBaseCamera +296): bevy's sides + near (the far plane is left out)
    let sides: Vec<(Vec3, f32)> = frustum
        .map(|f| f.half_spaces[..5].iter().map(|h| (h.normal().into(), h.d())).collect())
        .unwrap_or_default();
    let (near, far) = planes.map_or((0.5, 600.0), |p| (p.near, p.far));
    let view = crate::slipstream_render::DistortView {
        clip_from_world,
        cam_pos: gt.translation(),
        cam_fwd: gt.forward().as_vec3(),
        near,
        far,
        planes: &sides,
    };
    fx.slipstream.render_distortion(&mut fse, half, &view);
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MotionBlurSet;

pub struct MotionBlurPlugin;

impl Plugin for MotionBlurPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MotionBlur>().add_systems(PostUpdate, motion_blur_system.in_set(FseSet::Build).in_set(MotionBlurSet).after(crate::glow_pc_xbox::GlowSet));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// speed ramp of 0x0012FB60: no blur under 65 mph, full factor from 120 mph, amount capped at 130 mph
    #[test]
    fn amount_ramp() {
        let mut mb = MotionBlur::default();
        mb.tweaks = MotionBlurTweaks::default();
        let mph = |m: f32| MotionBlurInput { active: true, speed: m / 2.232, ..Default::default() };
        assert!(!mb.update_amount(&mph(60.0)));
        assert!(mb.update_amount(&mph(92.5)));
        assert!((mb.factor - 0.5).abs() < 1e-4);
        assert!(mb.update_amount(&mph(200.0)));
        assert_eq!(mb.amount, f32::from_bits(0x3bf5c28f));
        // fury fades it out
        let mut i = mph(200.0);
        i.fury = 0.5;
        assert!(!mb.update_amount(&i));
    }
}
