//! Render\Pc_xbox\ZoneRender_PC_XBOX.cpp — the zone screen effect (object `flt_737AB0`). The idb's region labels
//! are shifted here: init 0x00162B60 and the per-view draw 0x00162D10 sit in the ProjectedLightBeam range, the zone
//! level / tweaks 0x00154BD0..0x00155070 in the tyrespray range, only the ring 0x00162FF0 opens the real region.
//!
//! While a player is "in the zone" a per-player level (0..2) rises; above 0 the screen is copied grey into a half
//! buffer ("GrayScale"), motion-blurred there, and drawn back over the screen through a ring of wobbling strips
//! ("ZoneWobble", blend 2) whose alpha ramps up outwards — a sharp colour centre, a grey rippling rim. Level 1..2
//! interpolates the MinZone -> MaxZone tweak blocks by the car's speed.
//!
//! Frame rate: the original steps the level fades (0.25) and the ring clock (WaveSpeed) once per rendered frame at
//! the Xbox's 60 Hz (`car_render::XBOX_FRAME_RATE`); both are scaled by `dt · 60` here. The one-off clock is in ms.

use bevy::prelude::*;

use crate::full_screen_effects_manager_pc_xbox::{prog, FseImage, FseSet, FseShader, FseVertex, FullScreenEffects, FSE_STENCIL_NOT_CAR, OFF_SCREEN_HALF};
use crate::motion_blur_pc_xbox::{MotionBlur, MotionBlurInput};

/// one ZoneData block (13 floats, "ZoneData" local of 0x00162D10)
pub type ZoneData = [f32; 13];

/// ZoneData fields
pub mod zd {
    pub const WAVE_SPEED: usize = 0;
    pub const WAVE_SIZE: usize = 1;
    pub const WAVE_FREQUENCY: usize = 2;
    pub const BLUR: usize = 3;
    /// "GreyScale Width/Height", "R/G/B", "Brightness": tweakable, read by nothing in this build
    pub const GREY_WIDTH: usize = 4;
    pub const GREY_HEIGHT: usize = 5;
    pub const WOBBLE_RING_SIZE: usize = 6;
    pub const RIPPLE_WIDTH: usize = 7;
    pub const R: usize = 8;
    pub const G: usize = 9;
    pub const B: usize = 10;
    pub const BRIGHTNESS: usize = 11;
    /// the level when below 1 (fades the ring in), else 1
    pub const LEVEL: usize = 12;
}

const fn data(b: [u32; 13]) -> ZoneData {
    let mut d = [0.0f32; 13];
    let mut i = 0;
    while i < 13 {
        d[i] = f32::from_bits(b[i]);
        i += 1;
    }
    d
}

/// `unk_6874F0` "Graphics.ZoneFX.MinZone"
pub const MIN_ZONE: ZoneData = data([
    0xBF4CCCCD, 0x3D4CCCCD, 0x3FA33333, 0x3C75C28F, 0, 0, 0x3F000000, 0x3CB851EC, 0x3FE66666, 0x3FF33333, 0x3FFEB852, 0,
    0x3F800000,
]);
/// `unk_6874BC` "Graphics.ZoneFX.MaxZone"
pub const MAX_ZONE: ZoneData = data([
    0xBE800000, 0x3CA3D70A, 0x3FA33333, 0x3C75C28F, 0x3F0CCCCD, 0x3F0CCCCD, 0x3E800000, 0x3CB851EC, 0x3FE66666, 0x3FF33333,
    0x3FFEB852, 0x3E4CCCCD, 0x3F800000,
]);

/// `dword_687670` segments per ring
pub const RING_SEGMENTS: i32 = 16;
/// `dword_687678` alpha step per ring
pub const ALPHA_STEP: u32 = 32;
/// `dword_687674` fade-in steepness while the level is below 1
pub const FADE_SCALE: f32 = f32::from_bits(0x40800000);
/// 0x00162FF0 outer radius (1060655596) and `dword_687668` (clamped last ring)
pub const OUTER_RADIUS: f32 = f32::from_bits(0x3F333333);
pub const LAST_RING_RADIUS: f32 = f32::from_bits(0x3F333333);

/// "Graphics.ZoneFX" tweaks (0x00155070) and the module's debug bytes
#[derive(Clone, Copy, Debug)]
pub struct ZoneTweaks {
    /// +20 "On/off" (not read by the draw path)
    pub on: bool,
    /// +40 "Invert Colour" (not read by the xbox renderer)
    pub invert: bool,
    /// +48 "One off speed" (0.005 per ms)
    pub one_off_speed: f32,
    /// `dword_687524` "FadeRateA" (level below 1, or leaving the zone), per frame
    pub fade_a: f32,
    /// `dword_68752C` "FadeRateB" (level 1..2 towards the speed target), per frame
    pub fade_b: f32,
    /// `dword_687534` "Max Dilation Speed"
    pub max_dilation_speed: f32,
    /// `dword_687538` "Min Dilation"
    pub min_dilation: f32,
    pub min_zone: ZoneData,
    pub max_zone: ZoneData,
    /// `byte_68766C` (1): the innermost edge is not wobbled
    pub flat_inner: bool,
    /// `byte_748C00` (bss, 0): the last ring's outer edge sits at `LAST_RING_RADIUS`
    pub clamp_last: bool,
}

impl Default for ZoneTweaks {
    fn default() -> Self {
        Self {
            on: false,
            invert: false,
            one_off_speed: f32::from_bits(0x3BA3D70A),
            fade_a: f32::from_bits(0x3E800000),
            fade_b: f32::from_bits(0x3E800000),
            max_dilation_speed: f32::from_bits(0x42960000),
            min_dilation: f32::from_bits(0x3ECCCCCD),
            min_zone: MIN_ZONE,
            max_zone: MAX_ZONE,
            flat_inner: true,
            clamp_last: false,
        }
    }
}

/// what core supplies for the race view each frame
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct ZoneFxInput {
    /// GameManager 0x000B9010 gate: the view's render flag 0x4000000
    pub active: bool,
    /// the view's player (`v77[60]`; asserts `nPlayerID < 2`)
    pub player: usize,
    /// GameBridge +545 (paused: no clocks, no fades)
    pub paused: bool,
    /// VsAIRaceVehicle 0x00024220 (abs taken)
    pub speed: f32,
    /// VehicleManager 0x001DDCC0 vehicle +89 (0x00155990): zone on
    pub zone_on: bool,
    /// FuryFX 0x00128680 for the player: the zone only counts while it is <= 0
    pub fury: f32,
    /// VsAIRaceVehicle 0x00023310 (+968 set and its 0x00024F10 test): the one-off pulse
    pub one_off: bool,
}

/// VsAudio StartZone / StopZone requests of 0x00162D10 (StopZone repeats every frame while fading out)
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoneFxAudio {
    pub player: usize,
    pub start: bool,
}

/// the zone object `flt_737AB0`
#[derive(Resource, Clone, Debug)]
pub struct ZoneFx {
    pub tweaks: ZoneTweaks,
    /// +24 level per player (0..2)
    pub level: [f32; 2],
    /// +32 ring clock per player
    pub clock: [f32; 2],
    /// +41 one-off pulse running
    pub one_off: bool,
    /// +44 one-off clock in ms (< 0 off)
    pub one_off_clock: f32,
    /// result of 0x00154BD0 per player, written to the camera manager (`dword_6D55C8` + 44p + 41, 0x00163740)
    pub in_zone: [bool; 2],
    /// `dword_6F5B70` = 0x00154FC0 of the view player before its update
    pub amount_global: f32,
    /// `dword_6865F0`: bit p cleared while player p is in the zone at level 0; -1 when out
    pub zone_mask: i32,
}

impl Default for ZoneFx {
    /// ZoneRender_PC_XBOX.cpp 0x00162B60 (+41 0, +44 -1, +48 0.005; the shaders are `ZONE_WOBBLE` / `GRAY_SCALE`)
    fn default() -> Self {
        Self {
            tweaks: ZoneTweaks::default(),
            level: [0.0; 2],
            clock: [0.0; 2],
            one_off: false,
            one_off_clock: -1.0,
            in_zone: [false; 2],
            amount_global: 0.0,
            // CarPalette 0x0010E1A1 / CarRender 0x00110098 (car init) set it to -1 before any zone update
            zone_mask: -1,
        }
    }
}

/// "GrayScale" (0x00162B60): blend 1, clamp, z off
pub const GRAY_SCALE: FseShader = FseShader::new(1, false);
/// "ZoneWobble" (0x00162B60): blend 2, clamp, z off, stage 0 MODULATE colour and alpha (= the Copy pair), stencil
/// func 7 NOTEQUAL ref 4 mask 4 (not over the cars' bit 4; its ZERO pass op is read by nothing later)
pub const ZONE_WOBBLE: FseShader = FseShader { stencil: FSE_STENCIL_NOT_CAR, ..FseShader::new(2, false) };

/// tyrespray.cpp 0x00154F00 — per field `b·t + (1−t)·a`, equal fields copied
pub fn lerp_data(a: &ZoneData, b: &ZoneData, t: f32) -> ZoneData {
    let mut o = [0.0; 13];
    for i in 0..13 {
        o[i] = if a[i] == b[i] { a[i] } else { b[i] * t + (1.0 - t) * a[i] };
    }
    o
}

/// `sub_3DDFA8` (_ftol): truncation
fn ftol(v: f64) -> i32 {
    v as i32
}

impl ZoneFx {
    /// tyrespray.cpp range 0x00154FC0 — the zone amount others read (motion blur, fury, light streaks)
    pub fn amount(&self, player: usize) -> f32 {
        if player < 2 {
            self.level[player] * 0.5
        } else {
            0.0
        }
    }

    /// tyrespray.cpp range 0x00154BD0 — step player `p`'s level (`frames` = rendered 60 Hz frames this frame) and
    /// fill the ZoneData; returns (in zone, data — None when a finished one-off reset the level)
    pub fn update_level(&mut self, p: usize, inp: &ZoneFxInput, frames: f32) -> (bool, Option<ZoneData>) {
        let t = self.tweaks;
        let mut v18 = self.level[p];
        let v16 = inp.speed.abs();
        let mut v15 = false;
        if inp.zone_on {
            v15 = inp.fury <= 0.0;
        }
        if inp.one_off {
            if !self.one_off {
                self.one_off_clock = 0.0;
                self.one_off = true;
            }
        } else if self.one_off {
            self.level[p] = 0.0;
            self.one_off = false;
            return (false, None);
        }
        let d = if v18 >= 1.0 {
            lerp_data(&t.min_zone, &t.max_zone, v18 - 1.0)
        } else {
            let mut m = t.min_zone;
            m[zd::LEVEL] = v18;
            m
        };
        // the original steps once per 60 Hz frame; longer frames run whole steps so a fade never overshoots
        let mut left = if inp.paused { 0.0 } else { frames };
        let mut stepped = false;
        while left > 0.0 {
            let f = left.min(1.0);
            left -= f;
            v18 = self.step_level(v18, v15, v16, f);
            stepped = true;
        }
        if !stepped {
            v18 = self.step_level(v18, v15, v16, 0.0);
        }
        self.level[p] = v18;
        (v15, Some(d))
    }

    /// one frame of 0x00154BD0's level step (`f` = fraction of a 60 Hz frame)
    fn step_level(&mut self, mut v18: f32, v15: bool, v16: f32, frames: f32) -> f32 {
        let t = self.tweaks;
        let paused = frames == 0.0;
        if v18 >= 1.0 {
            if !paused {
                if v15 {
                    let mut v13 = v16 / (t.max_dilation_speed * 0.448) * (1.0 - t.min_dilation) + t.min_dilation + 1.0;
                    if v13 > 2.0 {
                        v13 = 2.0;
                    }
                    if v13 <= v18 {
                        if v18 > v13 {
                            v18 -= t.fade_b * frames;
                            if v13 > v18 {
                                v18 = v13;
                            }
                        }
                    } else {
                        v18 += t.fade_b * frames;
                        if v18 > v13 {
                            v18 = v13;
                        }
                    }
                } else {
                    v18 -= t.fade_a * frames;
                }
            }
        } else if !paused {
            // `dword_687524[!v15]`: +FadeRateA in the zone, its negation (0x00687528) out of it
            v18 += if v15 { t.fade_a } else { 0.0 - t.fade_a } * frames;
        }
        if v18 <= 0.0 {
            v18 = 0.0;
            self.one_off_clock = -1.0;
        }
        v18
    }
}

/// ZoneRender_PC_XBOX.cpp 0x00162FF0 — the wobble ring strip for a ZoneData at ring clock `clock`; uvs in texels of
/// the `tex`-sized source (the shader has no vertex program)
pub fn ring_vertices(clock: f32, src: &ZoneData, tex: Vec2, t: &ZoneTweaks) -> Vec<FseVertex> {
    let mut d = *src;
    let mut outer = OUTER_RADIUS;
    let mut fade = false;
    if d[zd::LEVEL] < 1.0 {
        outer *= d[zd::LEVEL];
        let l2 = d[zd::LEVEL] * d[zd::LEVEL];
        d[zd::WOBBLE_RING_SIZE] = d[zd::WOBBLE_RING_SIZE] * l2 + (1.0 - l2) * 0.5;
        fade = true;
    }
    let inner = 0.5 - d[zd::WOBBLE_RING_SIZE];
    let n = ((outer - inner) / d[zd::RIPPLE_WIDTH] + 0.5) as i32;
    let mut radius = inner;
    let mut phase = inner / d[zd::RIPPLE_WIDTH] * d[zd::WAVE_FREQUENCY] + clock;
    // the per-ring alphas (`_alloca(n + 1)`)
    let mut alpha = vec![0u8; (n.max(0) + 1) as usize];
    let mut acc: u32 = 0;
    for i in 0..=n.max(0) {
        let k = i as usize;
        let f = 1.0 - i as f32 / n as f32;
        alpha[k] = acc as u8;
        acc += ALPHA_STEP;
        if acc > 0xFF {
            acc = 255;
        }
        if fade {
            let mut s = f * FADE_SCALE;
            if f * FADE_SCALE > 1.0 {
                s = 1.0;
            }
            alpha[k] = ftol(alpha[k] as f64 * s as f64) as u8;
        }
    }
    let step = std::f32::consts::TAU / RING_SEGMENTS as f32;
    let mut out = Vec::with_capacity((n.max(0) as usize) * (RING_SEGMENTS as usize + 1) * 2);
    for r in 0..n {
        let ri = r as usize;
        for j in 0..=RING_SEGMENTS {
            let a = j as f32 * step;
            let s = a.sin();
            let c = a.cos();
            let pos = Vec2::new(s * radius + 0.5, c * radius + 0.5);
            let w = phase.sin() * d[zd::WAVE_SIZE] + a;
            let mut uv = Vec2::new(w.sin() * radius + 0.5, w.cos() * radius + 0.5);
            if r == 0 && t.flat_inner {
                uv = pos;
            }
            out.push(FseVertex { pos, uv: uv * tex, colour: (alpha[ri] as u32) << 24 | 0x00FF_FFFF });
            let r_out = radius + d[zd::RIPPLE_WIDTH];
            let ph_out = phase + d[zd::WAVE_FREQUENCY];
            let pos = if r == n - 1 && t.clamp_last {
                Vec2::new(s * LAST_RING_RADIUS + 0.5, c * LAST_RING_RADIUS + 0.5)
            } else {
                Vec2::new(s * r_out + 0.5, c * r_out + 0.5)
            };
            let w = ph_out.sin() * d[zd::WAVE_SIZE] + a;
            let uv = Vec2::new(w.sin() * r_out + 0.5, w.cos() * r_out + 0.5);
            out.push(FseVertex { pos, uv: uv * tex, colour: (alpha[ri + 1] as u32) << 24 | 0x00FF_FFFF });
        }
        radius += d[zd::RIPPLE_WIDTH];
        phase += d[zd::WAVE_FREQUENCY];
    }
    out
}

/// ZoneRender_PC_XBOX.cpp 0x00162D10 for view player `p`: clocks, level, audio, then (amount > 0) grey copy, motion blur (flags 0, Blur/2)
/// and the ring. `frame_ms` = GameManager 0x000BF060 (0 while paused). `focus` = the view player's car and the view's
/// clip-from-world for 0x00160920 (None keeps the last focus).
#[allow(clippy::too_many_arguments)]
pub fn zone_fx(
    zone: &mut ZoneFx,
    fse: &mut FullScreenEffects,
    mb: &mut MotionBlur,
    inp: &ZoneFxInput,
    focus: Option<(&MotionBlurInput, Mat4)>,
    frame_ms: f32,
    audio: &mut Vec<ZoneFxAudio>,
) {
    let p = inp.player;
    if p >= 2 {
        log::error!("assert: nPlayerID < 2");
        return;
    }
    if zone.one_off_clock >= 0.0 {
        zone.one_off_clock += frame_ms;
    }
    zone.amount_global = zone.amount(p);
    let frames = frame_ms * 0.001 * crate::car_render::XBOX_FRAME_RATE;
    let (in_zone, data) = zone.update_level(p, inp, frames);
    zone.in_zone[p] = in_zone;
    let mut amount = zone.level[p];
    if amount <= 0.0 {
        if !in_zone {
            zone.zone_mask = -1;
            return;
        }
        zone.zone_mask &= !(1 << p);
        if !zone.one_off {
            audio.push(ZoneFxAudio { player: p, start: true });
        }
    }
    if !in_zone {
        audio.push(ZoneFxAudio { player: p, start: false });
    }
    if zone.one_off {
        if zone.one_off_clock < 0.0 {
            amount = 0.0;
        } else {
            amount = (zone.one_off_clock * zone.tweaks.one_off_speed).sin();
            if amount < 0.0 {
                amount = 0.0;
                zone.one_off_clock = -1.0;
            }
        }
    }
    let Some(data) = data else { return };
    if amount <= 0.0 {
        return;
    }
    let Some(half) = fse.get_off_screen_buffer(OFF_SCREEN_HALF) else { return };
    fse.draw(half, FseImage::BackBuffer, GRAY_SCALE, prog::GRAY_SCALE, None, None, 0.0, crate::full_screen_effects_manager_pc_xbox::WHITE);
    // 0x00160BA0 → 0x00160920(view +28): only here, i.e. only when the zone draws (the +44 velocity latch is shared)
    if let Some((m, clip_from_world)) = focus {
        mb.update_focus(m, &clip_from_world);
    }
    crate::motion_blur_pc_xbox::render_from(fse, mb, half, 0, data[zd::BLUR] / 2.0);
    // 0x00162FF0: the clock steps per draw unless paused
    if !inp.paused {
        zone.clock[p] += data[zd::WAVE_SPEED] * frames;
    }
    let tex = fse.image_size(half).as_vec2();
    let verts = ring_vertices(zone.clock[p], &data, tex, &zone.tweaks);
    fse.draw_strip(FseImage::BackBuffer, half, ZONE_WOBBLE, prog::COPY, verts);
    fse.release_off_screen_buffer(half);
}

/// player 1's `ZoneFxInput` in a two-player race (`player` = 1; core fills it like `ZoneFxInput` for player 0)
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct ZoneFxSecondInput(pub Option<ZoneFxInput>);

/// player 1's car for the zone blur focus 0x00160920 in a two-player race (`position`, `velocity`, `wheels_down` as
/// in `MotionBlurInput`; core fills it from the slot-1 race car). None: player 1's view keeps the last focus.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct MotionBlurSecondInput(pub Option<MotionBlurInput>);

/// GameManager 0x000B9010: 0x00162D10(view, player) inside the view loop — once per view, in that view's viewport,
/// with that view's player's level / clocks (each call steps the shared clocks by the whole frame time)
#[allow(clippy::too_many_arguments)]
fn zone_fx_system(
    time: Res<Time>,
    mut fse: ResMut<FullScreenEffects>,
    mut zone: ResMut<ZoneFx>,
    mut mb: ResMut<MotionBlur>,
    input: Res<ZoneFxInput>,
    second: Option<Res<ZoneFxSecondInput>>,
    mb_input: Option<Res<MotionBlurInput>>,
    mb_second: Option<Res<MotionBlurSecondInput>>,
    mut images: ResMut<Assets<Image>>,
    views: crate::view_render::ViewCams,
    cams: Query<&Camera>,
    mut audio_out: MessageWriter<ZoneFxAudio>,
    fury: Option<Res<crate::fury_fx::FuryFx>>,
) {
    if !fse.active {
        return;
    }
    let force = std::env::var("VELOCITY_ZONE_FORCE").ok().and_then(|v| v.parse::<f32>().ok());
    let list = views.list();
    let restore = fse.viewport();
    for v in &list {
        let mut inp = if v.player == 0 { *input } else { second.as_ref().and_then(|s| s.0).unwrap_or_default() };
        inp.player = v.player;
        // 0x00154BD0 → FuryFX 0x00128680 of the view's vehicle (graphics' own state; core's field stays 0)
        if let Some(f) = fury.as_deref().filter(|f| f.initialised) {
            inp.fury = f.view_amount(v.player);
        }
        if let Some(speed) = force {
            // debug: in the zone at this speed (m/s)
            inp = ZoneFxInput { active: true, zone_on: true, speed, ..inp };
        }
        if !inp.active {
            continue;
        }
        mb.load_spec2(&mut images);
        // 0x00160920(view +28): VehicleManager 0x001DDCC0 = the view player's car, projected with the view's camera
        // (0x0015EC80 reads the device view / projection)
        let car = if v.player == 0 { mb_input.as_deref().copied() } else { mb_second.as_ref().and_then(|s| s.0) };
        let focus = match (car, cams.get(v.camera)) {
            (Some(m), Ok(camera)) if m.position != Vec3::ZERO || m.velocity != Vec3::ZERO => {
                Some((m, camera.clip_from_view() * v.transform.to_matrix().inverse()))
            }
            _ => None,
        };
        let frame_ms = if inp.paused { 0.0 } else { time.delta_secs() * 1000.0 };
        let mut audio = Vec::new();
        fse.set_viewport(v.rect);
        zone_fx(&mut zone, &mut fse, &mut mb, &inp, focus.as_ref().map(|(m, c)| (m, *c)), frame_ms, &mut audio);
        for a in audio {
            audio_out.write(a);
        }
    }
    fse.set_viewport(restore);
}

/// GameManager 0x000B9010: 0x00162D10 runs after the DOF (`byte_685E8C` = 1), before the danger warp
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ZoneFxSet;

pub struct ZoneFxPlugin;

impl Plugin for ZoneFxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoneFx>()
            .init_resource::<ZoneFxInput>()
            .init_resource::<ZoneFxSecondInput>()
            .init_resource::<MotionBlurSecondInput>()
            .add_message::<ZoneFxAudio>()
            .add_systems(PostUpdate, zone_fx_system.in_set(FseSet::Build).in_set(ZoneFxSet).after(crate::danger_zone_render::DofSet));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// level rises by FadeRateA per 60 Hz frame in the zone, then settles on the speed target; frame-rate independent
    #[test]
    fn level_fade_any_rate() {
        let inp = ZoneFxInput { active: true, zone_on: true, speed: 0.0, ..Default::default() };
        for fps in [30.0f32, 60.0, 144.0] {
            let mut z = ZoneFx::default();
            let frames = 60.0 / fps;
            let n = (fps * 2.0) as usize;
            for _ in 0..n {
                z.update_level(0, &inp, frames);
            }
            // speed 0: target = min dilation + 1
            assert!((z.level[0] - (1.0 + z.tweaks.min_dilation)).abs() < 1e-4, "{fps}: {}", z.level[0]);
        }
        let mut z = ZoneFx::default();
        z.update_level(0, &inp, 1.0);
        assert_eq!(z.level[0], 0.25);
        // out of the zone it falls back to 0 and the one-off clock goes off
        let out = ZoneFxInput { zone_on: false, ..inp };
        for _ in 0..4 {
            z.update_level(0, &out, 1.0);
        }
        assert_eq!(z.level[0], 0.0);
        assert_eq!(z.one_off_clock, -1.0);
    }

    /// full level: MinZone gives 31 rings of 17 segments (1054 vertices, the stream keeps 1000), alphas ramp by 32
    #[test]
    fn ring_shape() {
        let t = ZoneTweaks::default();
        let d = MIN_ZONE;
        let v = ring_vertices(0.0, &d, Vec2::ONE, &t);
        assert_eq!(v.len(), 31 * 17 * 2);
        // innermost edge: radius 0, alpha 0, unwobbled
        assert_eq!(v[0].pos, Vec2::splat(0.5));
        assert_eq!(v[0].colour >> 24, 0);
        assert_eq!(v[1].colour >> 24, 32);
        assert_eq!(v.last().unwrap().colour >> 24, 255);
        // fading in: level 0.5 shrinks the ring and the alphas
        let mut h = MIN_ZONE;
        h[zd::LEVEL] = 0.5;
        let f = ring_vertices(0.0, &h, Vec2::ONE, &t);
        assert!(f.len() < v.len());
    }
}
