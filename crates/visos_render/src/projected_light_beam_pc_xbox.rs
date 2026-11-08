//! ProjectedLightBeam_PC_XBOX.cpp, the wet-road reflection half (object `dword_74897C`: +0 the buffer, +4 "RefCopy",
//! +8 "RefPaste", +12 `byte_748988`). The slipstream distortion 0x001629B0 (`slipstream_render`) and the zone FX
//! 0x00162B60 / 0x00162D10 (`zone_render_pc_xbox`) live in the same file. The beam lights half (object `unk_7486B0`,
//! "Graphics.3DLightBeams"): the player's headlight pool, a volume texture projected onto the instance primitives that
//! CompartmentManager 0x0008ABB0 gives the "HeadLightBeam(Lit)" override (`model_builder::Builder::beam`).

use bevy::prelude::*;

use crate::full_screen_effects_manager_pc_xbox::{
    prog, FseImage, FseShader, FseVertex, FullScreenEffects, FFP_MODULATE, FSE_STENCIL_ROAD, OFF_SCREEN_QUARTER, ROAD_SCENE_DETAIL,
    ROAD_SCENE_PUDDLES, WHITE,
};
use crate::global_mat_input_resolver::{gmi, GlobalMatInputResolver};
use crate::light_streak::visos_colour;
use crate::renderer::shader_program_manager::{ObjectInputs, ProgramMaterial};

/// `dword_687658`: the copied band starts at this fraction of the horizon height
pub const BAND_TOP: f32 = f32::from_bits(0x3EB3_3333);
/// `dword_68765C`: height of the fade below the horizon, as a fraction of the space under it
pub const FADE: f32 = f32::from_bits(0x3CCC_CCCD);
/// `dword_687660` (3): bit 1 the mirrored image, bit 2 the fade strip
pub const STRIPS: u32 = 3;
/// `dword_687664`: fBlurAmount for the copy
pub const COPY_BLUR_AMOUNT: f32 = 1.0;
/// `byte_748988` (bss, never written): paste with RefCopy instead of RefPaste
pub const PASTE_WITH_COPY: bool = false;

/// ProjectedLightBeam_PC_XBOX.cpp 0x00162280 "RefCopy": reset, blend 1, SetZState(0, 0) (0x002A1BE0(1, 1, s) on
/// stages 0..3 not identified), texture = the default
pub fn ref_copy_shader() -> FseShader {
    FseShader::new(1, false)
}

/// 0x00162280 "RefPaste": blend 8 (src · dest alpha + dest), z off, stencil EQUAL (0x002A10C0(4)) ref 8 mask 8;
/// drawn without a program (0x0015E7B0): stage 0 = the reset's MODULATE(texture, diffuse)
pub fn ref_paste_shader() -> FseShader {
    FseShader { ffp: FFP_MODULATE, stencil: FSE_STENCIL_ROAD, ..FseShader::new(8, false) }
}

/// 0x00162400 paste stream: with `h` the horizon and `k` = (1 − h)·FADE, bit 1 = the band copy mirrored from
/// y = h + k down to the bottom (v from (1 − FADE)·H to 0, colour VISOS(0x80, b, b, b)), bit 2 = the fade from
/// black at y = h (v = H) to that colour at h + k; one strip of all vertices (so 2 → 3 also joins, original).
/// uv in texels of the buffer (`size`).
pub fn paste_strip(h: f32, size: Vec2, brightness: u8) -> Vec<FseVertex> {
    let colour = visos_colour([0x80, brightness, brightness, brightness]);
    let black = visos_colour([0x80, 0, 0, 0]);
    let k = (1.0 - h) * FADE;
    let fade = FADE;
    let (w, ht) = (size.x, size.y);
    let v = |x: f32, y: f32, u: f32, vv: f32, c: u32| FseVertex { pos: Vec2::new(x, y), uv: Vec2::new(u, vv), colour: c };
    let mut s = Vec::with_capacity(8);
    if STRIPS & 1 != 0 {
        s.push(v(0.0, h + k, w * 0.0, (1.0 - fade) * ht, colour));
        s.push(v(1.0, h + k, w * 1.0, (1.0 - fade) * ht, colour));
        s.push(v(0.0, 1.0, w * 0.0, ht * 0.0, colour));
        s.push(v(1.0, 1.0, w * 1.0, ht * 0.0, colour));
    }
    if STRIPS & 2 != 0 {
        s.push(v(0.0, h, w * 0.0, ht * 1.0, black));
        s.push(v(1.0, h, w * 1.0, ht * 1.0, black));
        s.push(v(0.0, h + k, w * 0.0, (1.0 - fade) * ht, colour));
        s.push(v(1.0, h + k, w * 1.0, (1.0 - fade) * ht, colour));
    }
    s
}

/// ProjectedLightBeam_PC_XBOX.cpp 0x00162400 (RainSurface 0x00137090, `horizon` = +24, `brightness` = +0): with a
/// quarter buffer (0x00162920 = 0x0015C920(2)): the roaddetail redraw (TrackFX 0x0014D490 when +44 is set; its bit 8
/// starts clear where it does not draw) and the puddles 0x00131F50, then the band 0.35·h .. h of the back buffer ->
/// the buffer (RefCopy, Copy program, fBlurAmount 1), one blur pass (0x0015C770) and the mirrored paste over the
/// road pixels; the buffer is released (0x00162970)
pub fn render_reflection(fse: &mut FullScreenEffects, horizon: f32, brightness: u8, road_detail: bool, puddles: bool) {
    let Some(q) = fse.get_off_screen_buffer(OFF_SCREEN_QUARTER) else { return };
    if road_detail {
        fse.road_stencil_clear();
        fse.road_scene(ROAD_SCENE_DETAIL);
    }
    if puddles {
        fse.road_scene(ROAD_SCENE_PUDDLES);
    }
    let top = BAND_TOP * horizon;
    fse.set_vector(gmi::BLUR_AMOUNT, Vec4::new(COPY_BLUR_AMOUNT, 0.0, 0.0, 0.0));
    fse.draw(q, FseImage::BackBuffer, ref_copy_shader(), prog::COPY, Some([0.0, top, 1.0, horizon - top]), None, 0.0, WHITE);
    fse.blur_passes(q, q, 1, false);
    let shader = if PASTE_WITH_COPY { ref_copy_shader() } else { ref_paste_shader() };
    let size = fse.image_size(q).as_vec2();
    fse.draw_strip(FseImage::BackBuffer, q, shader, prog::COPY, paste_strip(horizon, size, brightness));
    fse.release_off_screen_buffer(q);
}

// ---- beam lights: object `unk_7486B0` (+0 texture, +4 transform, +68 initialised, +69 "On/off", never read) ----

/// `dword_68762C` "MinOffset" (ring radius of the first slice, in half texture widths)
pub const MIN_OFFSET: f32 = f32::from_bits(0x3CF5_C28F);
/// `dword_687630` "MaxOffset"
pub const MAX_OFFSET: f32 = f32::from_bits(0x3E99_999A);
/// `dword_687634` "MinSigma"
pub const MIN_SIGMA: f32 = f32::from_bits(0x3CA3_D70A);
/// `dword_687638` "MaxSigma"
pub const MAX_SIGMA: f32 = f32::from_bits(0x3F0F_5C29);
/// `dword_68763C` "BeamDip"
pub const BEAM_DIP: f32 = f32::from_bits(0x3EB3_3333);
/// `dword_687640` "FOV"
pub const BEAM_FOV: f32 = f32::from_bits(0x3FB1_EB85);
/// `dword_687644` "Extent"
pub const BEAM_EXTENT: f32 = f32::from_bits(0x422C_0000);
/// `dword_687648` "DamagedFOV"
pub const DAMAGED_FOV: f32 = f32::from_bits(0x3F75_C28F);
/// `dword_68764C` "DamagedExtent"
pub const DAMAGED_EXTENT: f32 = f32::from_bits(0x41F0_0000);
/// `dword_7486A8` "FallOffStart" (bss, only the tweak writes it)
pub const FALL_OFF_START: f32 = 0.0;
/// `dword_687650` "Intensity" (int)
pub const BEAM_INTENSITY: i32 = 0x8C;
/// `flt_687654`: extra distance along the car when the camera is in it
pub const IN_CAR_OFFSET: f32 = f32::from_bits(0x3F00_0000);
/// `unk_748968` (guarded static, 0x001618D0): added to the look-at target
pub const TARGET_OFFSET: Vec3 = Vec3::new(0.0, f32::from_bits(0xBE4C_CCCD), 0.0);
/// 0x00161C40: TextureManager 0x0029EFF0(64, 64, 32) = D3DDevice_CreateVolumeTexture(64, 64, 32, 1 level, format 0
/// = D3DFMT_L8)
pub const BEAM_TEXTURE_SIZE: UVec3 = UVec3::new(64, 64, 32);

/// the "Graphics.3DLightBeams" tweak variables (0x00161190; the consts above are their .data values). +69 "On/off"
/// lives on `ProjectedLightBeam.on`.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct BeamTweaks {
    pub min_offset: f32,
    pub max_offset: f32,
    pub min_sigma: f32,
    pub max_sigma: f32,
    pub fov: f32,
    pub extent: f32,
    pub damaged_fov: f32,
    pub damaged_extent: f32,
    pub fall_off_start: f32,
    pub beam_dip: f32,
    pub intensity: i32,
    /// `byte_7486AC` "Redraw Texture": 0x001618D0 rebuilds the texture (0x00161520) and clears it
    pub redraw: bool,
}

impl Default for BeamTweaks {
    fn default() -> Self {
        Self {
            min_offset: MIN_OFFSET,
            max_offset: MAX_OFFSET,
            min_sigma: MIN_SIGMA,
            max_sigma: MAX_SIGMA,
            fov: BEAM_FOV,
            extent: BEAM_EXTENT,
            damaged_fov: DAMAGED_FOV,
            damaged_extent: DAMAGED_EXTENT,
            fall_off_start: FALL_OFF_START,
            beam_dip: BEAM_DIP,
            intensity: BEAM_INTENSITY,
            redraw: false,
        }
    }
}

/// VsExp (float in, the CRT exp, float out)
fn vs_exp(x: f32) -> f32 {
    (x as f64).exp() as f32
}

/// GameShaderProgramManager_PC_XBOX.cpp 0x00161520 — the L8 volume texture, linear order before XGSwizzleBox
/// (0x001610C0): per slice t = i/31 a ring at radius o = lerp(MinOffset, MaxOffset, t) with sigma s = lerp(MinSigma,
/// MaxSigma, t): g(x) = exp(−(x − o)²/(2s²)), value = clamp((g(r) − g(1)) / (g(0) − g(1))), r = |(2x/63 − 1, 2y/63 − 1)|,
/// × (1 − (t − F)/(1 − F)) past FallOffStart F, byte = _ftol(Intensity · value)
pub fn beam_texture(tw: &BeamTweaks) -> Vec<u8> {
    let (w, h, d) = (BEAM_TEXTURE_SIZE.x, BEAM_TEXTURE_SIZE.y, BEAM_TEXTURE_SIZE.z);
    let mut out = Vec::with_capacity((w * h * d) as usize);
    let fall = tw.fall_off_start;
    for i in 0..d {
        let t = (i as f64 * 1.0 / 31.0) as f32;
        let o = (tw.max_offset - tw.min_offset) * t + tw.min_offset;
        let s = (tw.max_sigma - tw.min_sigma) * t + tw.min_sigma;
        let g1 = vs_exp(((0.0 - (1.0 - o)) * (1.0 - o)) / ((2.0 * s) * s));
        let g0 = vs_exp((0.0 - o * o) / ((2.0 * s) * s));
        let range = (g0 as f64 - g1 as f64) as f32;
        let inv = (1.0 / range as f64) as f32;
        for y in 0..h {
            for x in 0..w {
                let u = ((x as f64 + x as f64) / 63.0 - 1.0) as f32;
                let v = ((y as f64 + y as f64) / 63.0 - 1.0) as f32;
                let r = (u * u + v * v).sqrt();
                let g = vs_exp(((0.0 - (r - o)) * (r - o)) / ((2.0 * s) * s));
                let mut val = ((g as f64 - g1 as f64) * inv as f64) as f32;
                if val > 1.0 {
                    val = 1.0;
                }
                if val < 0.0 {
                    val = 0.0;
                }
                if t > fall {
                    val *= 1.0 - (t - fall) / (1.0 - fall);
                }
                out.push((tw.intensity as f64 * val as f64) as i32 as u8);
            }
        }
    }
    out
}

/// GameShaderProgramManager_PC_XBOX.cpp 0x001618D0 — `lightbeamTextureTransform` (+4): D3DXMatrixLookAtLH(pos, pos +
/// dir + `unk_748968`, (0,1,0)) · scaling(1/c, 1/c, 1/s) · translation(0.5, 0.5, 0) with c = cos((π − fov)/2)·extent,
/// s = sin(..)·extent, transposed — so the uploaded registers c10..c13 are the columns of the d3d matrix (rows of
/// the glam one): uvw = (lateral/c + 0.5, vertical/c + 0.5, depth/s)
pub fn beam_transform(pos: Vec3, dir: Vec3, fov: f32, extent: f32) -> [Vec4; 4] {
    let at = pos + dir + TARGET_OFFSET;
    // glam's look_at_lh = D3DXMatrixLookAtLH in memory order (columns = d3d rows)
    let view = Mat4::look_at_lh(pos, at, Vec3::Y);
    let half = (std::f32::consts::PI - fov) / 2.0;
    let c = half.cos() * extent;
    let s = half.sin() * extent;
    let sc = Mat4::from_scale(Vec3::new((1.0 / c as f64) as f32, (1.0 / c as f64) as f32, (1.0 / s as f64) as f32));
    let tr = Mat4::from_translation(Vec3::new(0.5, 0.5, 0.0));
    let m = tr * sc * view;
    [m.row(0), m.row(1), m.row(2), m.row(3)]
}

/// the player car as 0x00161F10 reads it (car render data from CarDamage 0x000761B0(0))
#[derive(Clone, Copy, Debug)]
pub struct BeamSource {
    /// +0 the car matrix (x lateral, y up, z nose; translation = VsAIRaceVehicle 0x00023DD0)
    pub matrix: Mat4,
    /// +768 l_head z
    pub head_z: f32,
    /// +2328 bit 4: the camera sits in the car
    pub inside: bool,
    /// +2250 bits 1/2: left/right headlight broken
    pub damage: u8,
    /// +1808 width
    pub width: f32,
}

/// ProjectedLightBeam_PC_XBOX.cpp 0x00161F10 — beam origin, direction, fov and extent (GameManager 0x000B904E, every
/// render frame before the world): pos = car + z·(head.z [+0.5 in car]), y −= BeamDip; a broken light uses the damaged
/// fov/extent and moves the origin by ∓x·width/5.5; both broken: y −100 and dir (0,−1,0)
pub fn beam_pose(c: &BeamSource, tw: &BeamTweaks) -> (Vec3, Vec3, f32, f32) {
    let z = c.matrix.z_axis.truncate();
    let x = c.matrix.x_axis.truncate();
    let car = c.matrix.w_axis.truncate();
    let mut dir = z;
    let along = if c.inside { c.head_z + IN_CAR_OFFSET } else { c.head_z };
    let mut pos = car + z * along;
    pos.y -= tw.beam_dip;
    let (mut fov, mut extent) = (tw.fov, tw.extent);
    if c.damage & 3 != 0 {
        fov = tw.damaged_fov;
        extent = tw.damaged_extent;
    }
    let shift = (c.width as f64 / 5.5) as f32;
    if c.damage & 1 != 0 && c.damage & 2 != 0 {
        pos.y += -100.0;
        dir = Vec3::NEG_Y;
    } else if c.damage & 1 != 0 {
        pos -= x * shift;
    } else if c.damage & 2 != 0 {
        pos += x * shift;
    }
    (pos, dir, fov, extent)
}

/// the beam object: the volume texture (black placeholder until the in-game init, as the null texture of the front
/// end) and the transform, both handed to the global resolver (GFXHelper 0x0012C8F0 / 0x0012C910)
#[derive(Resource)]
pub struct ProjectedLightBeam {
    pub texture: Option<Handle<Image>>,
    pub black: Handle<Image>,
    pub matrix: [Vec4; 4],
    /// +69 "On/off" (1; never read)
    pub on: bool,
}

/// program materials of instance primitives drawn with the "HeadLightBeam(Lit)" override (rebound every frame)
#[derive(Resource, Default)]
pub struct BeamPrograms(pub Vec<crate::model_builder::BeamProgram>);

/// an instance primitive drawn with the beam override; `fixed` = its plain stage material (A/B harness only)
#[derive(Component, Clone, Debug)]
pub struct BeamPrim {
    pub program: Handle<ProgramMaterial>,
    pub fixed: Handle<crate::renderer::shader_manager::XboxStageMaterial>,
}

fn volume_image(size: UVec3, l8: &[u8]) -> Image {
    use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let data: Vec<u8> = l8.iter().flat_map(|&l| [l, l, l, 255]).collect();
    let mut img = Image::new(
        Extent3d { width: size.x, height: size.y, depth_or_array_layers: size.z },
        TextureDimension::D3,
        data,
        TextureFormat::Rgba8Unorm,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    // 0x001065A0: pass 1 clamped in u, v, w (0x002A1CC0(3, 3, 3)); the reset's linear filtering
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        address_mode_w: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..Default::default()
    });
    img
}

fn beam_startup(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut globals: ResMut<GlobalMatInputResolver>) {
    let black = images.add(volume_image(UVec3::ONE, &[0]));
    globals.textures[gmi::LIGHTBEAM_TEXTURE] = Some(black.clone());
    commands.insert_resource(ProjectedLightBeam { texture: None, black, matrix: [Vec4::ZERO; 4], on: true });
    commands.init_resource::<BeamTweaks>();
}

type BeamCarQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static crate::car_render::CarRender,
        &'static Transform,
        &'static crate::track_fx::CarHeadBeams,
        Option<&'static crate::car_render::CarShadow>,
        Option<&'static crate::car_render::CarLightInput>,
        Option<&'static crate::car_damage_render::CarLightDamage>,
    ),
>;

/// ProjectedLightBeam_PC_XBOX.cpp 0x00161C40 (GameManager::InGameInit 0x000B62C3: the texture once a game car exists) and 0x00161F10 per frame for
/// the player car (CarDamage 0x000761B0(0) = race car 0: the lowest request of the in-game, non-traffic cars)
fn beam_update(
    mut beam: ResMut<ProjectedLightBeam>,
    mut images: ResMut<Assets<Image>>,
    mut globals: ResMut<GlobalMatInputResolver>,
    cars: BeamCarQuery,
    view: Option<Res<crate::track_fx::HeadlightView>>,
    ab: Option<Res<crate::gfx_ab::GfxAb>>,
    tweaks: Option<ResMut<BeamTweaks>>,
) {
    let mut tw = tweaks.map(|t| t.into_inner());
    let cur = tw.as_deref().copied().unwrap_or_default();
    let player = cars.iter().filter(|c| !c.0.spec.frontend && !c.0.spec.traffic).min_by_key(|c| c.0.request);
    if let Some((r, t, hb, sh, input, dmg)) = player {
        if beam.texture.is_none() {
            beam.texture = Some(images.add(volume_image(BEAM_TEXTURE_SIZE, &beam_texture(&cur))));
        }
        // 0x001618D0: "Redraw Texture" rebuilds the volume and clears itself
        if let Some(tw) = tw.as_deref_mut().filter(|t| t.redraw) {
            beam.texture = Some(images.add(volume_image(BEAM_TEXTURE_SIZE, &beam_texture(tw))));
            tw.redraw = false;
        }
        let src = BeamSource {
            matrix: t.to_matrix(),
            head_z: hb.head.z,
            inside: view.as_ref().is_some_and(|v| v.inside == Some(r.request)),
            damage: input.map_or(0, |i| i.damage) | dmg.map_or(0, |d| d.0),
            width: sh.map_or(crate::car_render::SHADOW_SIZE_DEFAULT[0], |s| s.size[0]),
        };
        let (pos, dir, fov, extent) = beam_pose(&src, &cur);
        beam.matrix = beam_transform(pos, dir, fov, extent);
    }
    // debug A/B: `VELOCITY_BEAM_LIGHTS=0` keeps the black placeholder (no pool)
    let off = ab.is_some_and(|a| a.beam_texture_off()) || std::env::var("VELOCITY_BEAM_LIGHTS").is_ok_and(|v| v == "0");
    globals.textures[gmi::LIGHTBEAM_TEXTURE] = if off { Some(beam.black.clone()) } else { beam.texture.clone().or(Some(beam.black.clone())) };
    globals.beam_matrix = beam.matrix;
    // debug: `VELOCITY_BEAM_LIGHTS=all` maps every point to the centre of slice 0 (each beam prim gets the full pool)
    if std::env::var("VELOCITY_BEAM_LIGHTS").is_ok_and(|v| v == "all") {
        globals.beam_matrix = [Vec4::new(0.0, 0.0, 0.0, 0.5), Vec4::new(0.0, 0.0, 0.0, 0.5), Vec4::new(0.0, 0.0, 0.0, 0.01), Vec4::W];
    }
}

/// ShaderProgramManager BindShader 0x002DF5A0 for every beam program material: the lights are the device's current
/// D3D lights (GFXHelper 0x0012C670 reads them back): the instance set (count 0) enables none and sets none, so they
/// hold the last SetLight of the previous frame — taken as the player car's 3d hud set (OSD3dInterface 0x000EA2B0,
/// drawn after the world; unverified); ambient = D3DRS_AMBIENT = the instance set's (`TrackInstanceLights`)
fn bind_beam_programs(
    progs: Option<Res<BeamPrograms>>,
    mut materials: ResMut<Assets<ProgramMaterial>>,
    globals: Res<GlobalMatInputResolver>,
    sets: Option<Res<crate::car_render::CarLightSets>>,
    inst: Option<Res<crate::track_render::TrackInstanceLights>>,
    cars: BeamCarQuery,
) {
    let Some(progs) = progs else { return };
    let player = cars.iter().filter(|c| !c.0.spec.frontend && !c.0.spec.traffic).map(|c| c.0.request).min();
    let mut lights = match (player, sets.as_ref()) {
        (Some(p), Some(s)) => s.0.get(&p).map(|l| l.hud_light_set(Vec4::ZERO)).unwrap_or_default(),
        _ => crate::vs_render::VsLights { colour: [Vec4::ZERO; 3], ..Default::default() },
    };
    lights.ambient = inst.map_or(Vec4::ZERO, |i| i.sky_ambient.clamp(Vec4::ZERO, Vec4::ONE));
    for p in &progs.0 {
        ProgramMaterial::rebind(&mut materials, &p.material, |m| {
            m.params.fog = p.fog;
            m.bind_inputs(&[], &ObjectInputs { lights, base: p.base.clone() }, &globals);
        });
    }
}

/// A/B harness: the beam primitives back on their plain stage materials while the phase asks for it
fn ab_beam_override(
    mut commands: Commands,
    ab: Option<Res<crate::gfx_ab::GfxAb>>,
    prims: Query<(Entity, &BeamPrim, Has<MeshMaterial3d<crate::renderer::shader_manager::XboxStageMaterial>>)>,
) {
    let Some(ab) = ab.filter(|a| a.enabled()) else { return };
    let off = ab.beam_override_off();
    for (e, p, fixed) in &prims {
        if off && !fixed {
            commands.entity(e).remove::<MeshMaterial3d<ProgramMaterial>>().insert(MeshMaterial3d(p.fixed.clone()));
        } else if !off && fixed {
            commands.entity(e).remove::<MeshMaterial3d<crate::renderer::shader_manager::XboxStageMaterial>>().insert(MeshMaterial3d(p.program.clone()));
        }
    }
}

/// "Graphics.3DLightBeams": the beam object and the per-frame binding of the instance override programs
pub struct ProjectedLightBeamPlugin;

impl Plugin for ProjectedLightBeamPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::renderer::shader_program_manager::ProgramMaterialPlugin>() {
            app.add_plugins(crate::renderer::shader_program_manager::ProgramMaterialPlugin);
        }
        app.add_systems(Startup, beam_startup).add_systems(
            PostUpdate,
            (beam_update, bind_beam_programs, ab_beam_override).chain().before(bevy::transform::TransformSystems::Propagate),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0x00161520: 64 × 64 × 32 bytes; the first slice is a bright dot (o 0.03, s 0.02), corners dark, the last
    /// slice faded out (FallOffStart 0), nothing above Intensity
    #[test]
    fn beam_texture_profile() {
        let t = beam_texture(&BeamTweaks::default());
        assert_eq!(t.len(), 64 * 64 * 32);
        let at = |s: usize, y: usize, x: usize| t[s * 4096 + y * 64 + x];
        assert_eq!(at(0, 31, 31), 140);
        assert_eq!((at(0, 0, 0), at(16, 0, 0)), (0, 0));
        assert!(t[31 * 4096..].iter().all(|&b| b == 0));
        assert!(t.iter().all(|&b| b <= 140));
        // a ring in the middle slices: brighter at the offset radius than on the axis
        let ring = (MAX_OFFSET - MIN_OFFSET) * (16.0 / 31.0) + MIN_OFFSET;
        let x = ((ring + 1.0) * 63.0 / 2.0).round() as usize;
        assert!(at(16, 31, x) > 0);
        assert!((0..64).map(|y| at(16, 31, y)).max().unwrap() > 0);
    }

    /// 0x001618D0: the look axis (dir + (0, −0.2, 0)) maps to the texture centre, depth s = sin((π − fov)/2)·extent
    /// to w = 1, lateral c = cos(..)·extent to u = 1.5
    #[test]
    fn beam_transform_axis() {
        let r = beam_transform(Vec3::new(1.0, 2.0, 3.0), Vec3::Z, BEAM_FOV, BEAM_EXTENT);
        let uvw = |p: Vec3| Vec3::new(r[0].dot(p.extend(1.0)), r[1].dot(p.extend(1.0)), r[2].dot(p.extend(1.0)));
        let half = (std::f32::consts::PI - BEAM_FOV) / 2.0;
        let (c, s) = (half.cos() * BEAM_EXTENT, half.sin() * BEAM_EXTENT);
        let f = (Vec3::Z + TARGET_OFFSET).normalize();
        let a = uvw(Vec3::new(1.0, 2.0, 3.0) + f * s);
        assert!((a - Vec3::new(0.5, 0.5, 1.0)).length() < 1e-4, "{a}");
        let side = Vec3::Y.cross(f).normalize();
        let b = uvw(Vec3::new(1.0, 2.0, 3.0) + side * c);
        assert!((b - Vec3::new(1.5, 0.5, 0.0)).length() < 1e-4, "{b}");
    }

    /// 0x00161F10: dip, in-car offset, damaged fov/extent and shifts, both broken
    #[test]
    fn beam_pose_damage() {
        let m = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let mut c = BeamSource { matrix: m, head_z: 2.0, inside: false, damage: 0, width: 2.75 };
        let tw = BeamTweaks::default();
        let (p, d, fov, ext) = beam_pose(&c, &tw);
        assert_eq!((p, d, fov, ext), (Vec3::new(10.0, -BEAM_DIP, 2.0), Vec3::Z, BEAM_FOV, BEAM_EXTENT));
        c.inside = true;
        assert_eq!(beam_pose(&c, &tw).0.z, 2.5);
        c.inside = false;
        c.damage = 1;
        let (p, _, fov, ext) = beam_pose(&c, &tw);
        assert_eq!((p.x, fov, ext), (9.5, DAMAGED_FOV, DAMAGED_EXTENT));
        c.damage = 2;
        assert_eq!(beam_pose(&c, &tw).0.x, 10.5);
        c.damage = 3;
        let (p, d, _, _) = beam_pose(&c, &tw);
        assert_eq!((p.y, d), (-BEAM_DIP - 100.0, Vec3::NEG_Y));
    }

    /// the eight stream vertices of 0x00162400 for h = 0.4 on a 160x120 buffer
    #[test]
    fn paste_stream() {
        let s = paste_strip(0.4, Vec2::new(160.0, 120.0), 128);
        assert_eq!(s.len(), 8);
        let k = 0.6 * FADE;
        assert!((s[0].pos.y - (0.4 + k)).abs() < 1e-6 && (s[0].uv.y - (1.0 - FADE) * 120.0).abs() < 1e-4);
        assert_eq!((s[3].pos, s[3].uv), (Vec2::new(1.0, 1.0), Vec2::new(160.0, 0.0)));
        assert_eq!((s[4].pos.y, s[4].uv.y, s[4].colour), (0.4, 120.0, 0xFF00_0000));
        assert_eq!(s[1].colour, 0xFFFF_FFFF);
    }

    /// queued passes: alpha clear, the two road slots, copy, blur (2), paste; the buffer comes back
    #[test]
    fn reflection_passes() {
        let mut fse = FullScreenEffects::default();
        fse.active = true;
        fse.test_buffers();
        render_reflection(&mut fse, 0.45, 128, true, true);
        let p = fse.queued();
        assert_eq!(p.len(), 7);
        assert_eq!(p[0].shader.stencil, crate::full_screen_effects_manager_pc_xbox::FSE_STENCIL_CLEAR);
        assert_eq!((p[1].road_scene, p[2].road_scene), (Some(ROAD_SCENE_DETAIL), Some(ROAD_SCENE_PUDDLES)));
        assert!((p[3].src_rect[1] - BAND_TOP * 0.45).abs() < 1e-6 && p[3].target == FseImage::OffScreen(2));
        assert_eq!((p[6].shader.blend, p[6].shader.stencil, p[6].source), (8, FSE_STENCIL_ROAD, FseImage::OffScreen(2)));
        assert_eq!(fse.get_off_screen_buffer(OFF_SCREEN_QUARTER), Some(FseImage::OffScreen(2)));
    }
}
