//! Render\FlareRender.cpp (0x00123A20..0x001273B0; the idb labels it DangerZoneRender / FlareRender, the assert at
//! 0x001271B0 names the file) — the car light flares: a "LensFlareMiddle" glow on each headlight facing the camera
//! with screen-space lens ghosts and a horizontal streak, red "rearL" glows on the brake lights (white on the reverse
//! lights), police top lights, the flashing lights of emergency traffic, and the headlight streaks fed to a second
//! LightStreak manager (width 128). Object `unk_719450`, drawn by GameManager 0x000B9D04 after TrackFX's flares,
//! and in the front end by FECK2ElementCamera 0x00299C5A (front-end cars, `rain_surface::FrontEndCamera`). Also the
//! dummy police cars' top lights (CarRender 0x001192F0, `car_render::DummyPoliceCars`) and the flash records
//! 0x001270E0 / 0x001267C0 ("FS0"/"FS1", `FlareFlash`).

use bevy::math::{Mat4, Vec2, Vec3, Vec4};
use bevy::prelude::*;

use crate::effect_draw::{EffectDraw, EffectShader, EffectTexture, EffectVertex};
use crate::light_streak::{visos_colour, LightStreakManager, StreakLight, MAX_LIGHTSTREAK_LIGHTS};

/// `dword_686A08` / `dword_686A04`: flare range, full-size range (head flares)
pub const FLARE_RANGE: f32 = 200.0;
pub const FLARE_NEAR: f32 = 150.0;
/// `dword_6869FC` / `dword_686A00`: head flare size at `FLARE_NEAR` and beyond / at the camera
pub const HEAD_SIZE_FAR: f32 = 3.5;
pub const HEAD_SIZE_NEAR: f32 = f32::from_bits(0x3FA6_6666);
/// `flt_6869E8`: head flare alpha scale; `flt_686AD4`: width over height
pub const HEAD_ALPHA: f32 = f32::from_bits(0x3ECC_CCCD);
pub const HEAD_WIDTH: f32 = 2.0;
/// `flt_686ADC` / `flt_686AD8` / `flt_686AE0` / `flt_686AE4`: powers of the facing cosine (size, alpha, ghosts, streak)
pub const POW_SIZE: f32 = 2.0;
pub const POW_ALPHA: f32 = 4.0;
pub const POW_GHOST: f32 = 4.0;
pub const POW_STREAK: f32 = 8.0;
/// `byte_686A10` glare on, `dword_686A0C` glare range, `flt_686AC8` / `flt_686ACC` powers of its fade
pub const GLARE_ON: bool = true;
pub const GLARE_RANGE: f32 = 150.0;
pub const GLARE_POW_GHOST: f32 = 2.0;
pub const GLARE_POW_STREAK: f32 = 4.0;
/// `dword_6869E4`: ghost size in screen units (of 512)
pub const GHOST_SIZE: f32 = 192.0;
/// `dword_686A14`: 4:3 aspect (0x3FAAAAAB)
pub const ASPECT: f32 = f32::from_bits(0x3FAA_AAAB);
/// `dword_686B14` streak width scale, `dword_686B0C` / `dword_686B10` streak height range
pub const STREAK_WIDTH: f32 = 1.25;
pub const STREAK_H_MIN: f32 = 8.0;
pub const STREAK_H_MAX: f32 = 24.0;
/// `dword_686B18`: ghost fade radius from the screen centre
pub const GHOST_RADIUS: f32 = 0.5;
/// `flt_6869F4` / `dword_6869EC`: streak / ghost alpha in camera mode 3 (index `dword_7192A8`)
pub const MODE_SCALE: [f32; 2] = [1.0, 0.5];
/// `dword_686A24` rear range, `dword_686A18` size at range, `dword_686A1C` / `dword_686A20` base size (brake 0 / 1)
pub const REAR_RANGE: f32 = 200.0;
pub const REAR_SIZE_FAR: f32 = 7.0;
pub const REAR_SIZE: f32 = f32::from_bits(0x3F8C_CCCD);
pub const REAR_SIZE_BRAKE: f32 = 1.5;
/// `dword_686A2C` traffic alpha, `dword_686A30` / `dword_686A34` alpha (brake 0 / 1), `flt_686A28` width
pub const REAR_ALPHA_TRAFFIC: f32 = 0.75;
pub const REAR_ALPHA: f32 = 0.5;
pub const REAR_ALPHA_BRAKE: f32 = 1.0;
pub const REAR_WIDTH: f32 = 1.0;
/// `unk_719FE8` (local static): added to the l_brake locator
pub const REAR_OFFSET: Vec3 = Vec3::new(f32::from_bits(0x3DCC_CCCD), f32::from_bits(0xBD4C_CCCD), 0.0);
/// `flt_686AE8`: reverse flare size
pub const REVERSE_SIZE: f32 = 0.75;
/// `dword_686A70` range, `flt_686A6C` / `flt_686A68` size near / far, `flt_686A74` width, `dword_686A78` phase per
/// traffic index, `flt_686AEC` power, `flt_686A38` alpha, `dword_686A64` phase speed (cycles per second)
pub const TRAFFIC_RANGE: f32 = 200.0;
pub const TRAFFIC_SIZE: [f32; 2] = [2.0, 20.0];
pub const TRAFFIC_WIDTH: f32 = 1.5;
pub const TRAFFIC_INDEX_PHASE: f32 = 2.0;
pub const TRAFFIC_POW: f32 = 4.0;
pub const TRAFFIC_ALPHA: f32 = 1.5;
pub const TRAFFIC_SPEED: f32 = 2.0;
/// `dword_686A3C`: start phase of each traffic light
pub const TRAFFIC_PHASES: [u32; 10] = [
    0x0000_0000, 0x3FC9_0FDB, 0x4049_0FDB, 0x4096_CBE4, 0x4049_0FDB, 0x4049_0FDB, 0x4096_CBE4, 0x4096_CBE4, 0x0000_0000, 0x3FC9_0FDB,
];
/// static init 0x004253A3 (`unk_7193A0`): the ten traffic light positions (car space, z as stored)
pub const TRAFFIC_LIGHTS: [[u32; 3]; 10] = [
    [0xBF60_C49C, 0x4013_53F8, 0x404E_E979],
    [0x3F60_C49C, 0x4013_53F8, 0x404E_E979],
    [0xBF77_0A3D, 0x3F68_B439, 0x4071_1687],
    [0x3F77_0A3D, 0x3F68_B439, 0x4071_1687],
    [0xBFBD_70A4, 0x3FF2_0C4A, 0xBF20_8312],
    [0x3FBD_70A4, 0x3FF2_0C4A, 0xBF20_8312],
    [0xBFBD_70A4, 0x3FF2_0C4A, 0xC071_47AE],
    [0x3FBD_70A4, 0x3FF2_0C4A, 0xC071_47AE],
    [0xBF91_EB85, 0x3FC1_EB85, 0xC08F_9581],
    [0x3F91_EB85, 0x3FC1_EB85, 0xC08F_9581],
];
/// static init 0x004254B3 (`unk_7192B8`): their facing directions
pub const TRAFFIC_DIRS: [[f32; 3]; 10] = [
    [0.0, 0.0, 1.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.0, 1.0],
    [-1.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 0.0, -1.0],
    [0.0, 0.0, -1.0],
];
/// `dword_686B08` range, `flt_686B00` / `flt_686B04` size near / far, `flt_686AFC` height, `flt_686AF8` width,
/// `flt_686AF4` phase between the two lights, `dword_686AF0` alpha, `flt_686A80` phase speed (cycles per second)
pub const TOP_RANGE: f32 = 200.0;
pub const TOP_SIZE: [f32; 2] = [3.5, 25.0];
pub const TOP_HEIGHT: f32 = f32::from_bits(0x3EB3_3333);
pub const TOP_WIDTH: f32 = f32::from_bits(0x3F19_999A);
pub const TOP_LIGHT_PHASE: f32 = std::f32::consts::PI;
pub const TOP_ALPHA: f32 = 1.75;
pub const TOP_SPEED: f32 = 2.0;
/// `flt_686A7C`: headlight flash period; `unk_686A88`: size over the period (GFXHelper 0x0012BC40 points)
pub const FLASH_PERIOD: f32 = 1.75;
pub const FLASH_CURVE: [[f32; 2]; 8] = [[0.0, 1.0], [0.25, 1.0], [0.25, 1.75], [0.5, 1.75], [0.5, 1.0], [1.0, 1.0], [1.0, 1.75], [0.0, 0.0]];
/// DangerZoneRender 0x00123930: the flash timer a traffic car's "hazard" call sets (4.0)
pub const FLASH_TIME: f32 = 4.0;
/// `dword_6869D0` / 2: depth bias of the z-tested flare shaders
pub const FLARE_BIAS: i16 = 128;
/// static init 0x004251A3 (`dword_719358`): ghosts {alpha, position (0 = the light, 1 = mirrored), size, shader,
/// colour (a, r, g, b)}
pub const GHOSTS: [(u32, u32, u32, usize, [u8; 4]); 3] = [
    (0x3F28_F5C3, 0x0000_0000, 0x3F4C_CCCD, 2, [0x80, 0x80, 0x80, 0x80]),
    (0x3F19_999A, 0x3F33_3333, 0x3E99_999A, 1, [0x80, 0x1A, 0x19, 0x10]),
    (0x3F19_999A, 0x3F38_51EC, 0x3E80_0000, 1, [0x80, 0x0D, 0x17, 0x1C]),
];
/// static inits (a, r, g, b): head `unk_719440`, rear `unk_719394`, reverse `unk_719444`, streak `unk_719448`,
/// traffic `dword_719398`
pub const HEAD_COLOUR: [u8; 4] = [0x80, 0x80, 0x80, 0x80];
pub const REAR_COLOUR: [u8; 4] = [0x80, 0xC0, 0x00, 0x00];
pub const REVERSE_COLOUR: [u8; 4] = [0x80, 0x80, 0x80, 0x80];
pub const STREAK_COLOUR: [u8; 4] = [0x80, 0x10, 0x10, 0x20];
pub const TRAFFIC_COLOUR: [u8; 4] = [0x80, 0x00, 0x50, 0xB0];
/// `off_6869D4`: the four "flare" shader textures (misc.vcd)
pub const FLARE_TEXTURES: [&str; 4] = ["LensFlareMiddle", "LensFlareSecondary", "LensFlareSecondaryBlob", "LensFlareStreak"];
/// eye depth the screen-space (pretransformed, Renderer 0x002A9B00) quads are placed at; their shaders have no z test
pub const SCREEN_DEPTH: f32 = 1.0;
/// `eMaxStreakLightTypes * MAX_VEHICLE_COUNT` (light index = car + 22 · type)
pub const MAX_STREAK_LIGHTS: usize = 88;
pub const MAX_VEHICLE_COUNT: usize = 22;

/// CarRender data the flares read, z negated as CarRender 0x00110220 / 0x001179xx store them: +896 `l_brake`,
/// +784 / +800 `l_reverse` / `r_reverse` (zero when missing), +816.. `TopLight_1`, `TopLight_2` (count +1748)
#[derive(Component, Clone, Debug, Default)]
pub struct CarFlareLocators {
    pub brake: Vec3,
    pub reverse: [Vec3; 2],
    reverse_found: [bool; 2],
    pub top: Vec<Vec3>,
    /// `l_nos` / `r_nos` of the fitted EXHAUST part's LOD 0 model (part 10, car +1208 +440; FuryFX 0x00128A40), z
    /// negated like the others
    pub nos: [Option<Vec3>; 2],
}

fn locator(m: &crate::renderer::renderer_types::Model, n: &str) -> Option<Vec3> {
    m.locator(n).map(|x| Vec3::new(x[12], x[13], -x[14]))
}

impl CarFlareLocators {
    /// CarRender.cpp 0x00117770 / 0x00110220 — from the LOD 0 chassis model (`TopLight_%d` stops at the first missing)
    pub fn from_chassis(m: &crate::renderer::renderer_types::Model) -> Self {
        let r = [locator(m, "l_reverse"), locator(m, "r_reverse")];
        let mut top = Vec::new();
        for i in 0..2 {
            match locator(m, &format!("TopLight_{}", i + 1)) {
                Some(p) => top.push(p),
                None => break,
            }
        }
        Self {
            brake: locator(m, "l_brake").unwrap_or(Vec3::ZERO),
            reverse: r.map(|p| p.unwrap_or(Vec3::ZERO)),
            reverse_found: r.map(|p| p.is_some()),
            top,
            nos: [None; 2],
        }
    }

    /// the exhaust part's nitrous locators (FuryFX 0x00128A40: GameDataLoading 0x000A3CB0 "l_nos", "r_nos")
    pub fn nos_from_exhaust(&mut self, m: &crate::renderer::renderer_types::Model) {
        self.nos = [locator(m, "l_nos"), locator(m, "r_nos")];
    }

    /// part 0's model fills the reverse locators the chassis lacks
    pub fn reverse_from_part(&mut self, m: &crate::renderer::renderer_types::Model) {
        for (i, n) in ["l_reverse", "r_reverse"].iter().enumerate() {
            if !self.reverse_found[i] {
                if let Some(p) = locator(m, n) {
                    self.reverse[i] = p;
                    self.reverse_found[i] = true;
                }
            }
        }
    }
}

/// per-car flare state in CarRender data: +1824 headlight flash timer, +2332 top light on, +2344 top light colour
/// (D3DCOLOR), +2336 top light phase, and the emergency traffic flag (vehicle +5080 traffic index, VsAITrafficVehicle
/// 0x0002B5D0)
#[derive(Component, Clone, Debug, Default)]
pub struct CarFlareState {
    pub flash: f32,
    pub top_on: [bool; 2],
    pub top_colour: [u32; 2],
    pub phase: f32,
    pub traffic: Option<i32>,
}

/// calls into FlareRender from the game (by `CarRender.request`)
#[derive(Message, Clone, Debug)]
pub enum CarFlareMessage {
    /// DangerZoneRender 0x00123930 (VsAITrafficVehicle 0x0002B4B0, the hazard call): flash timer 4.0 / 0
    HeadlightFlash { request: u64, on: bool },
    /// 0x00123970 (VsAIPoliceVehicle::SetSiren 0x000179B0): top light `light` on/off with colour a, r, g, b
    TopLight { request: u64, light: usize, on: bool, a: u8, r: u8, g: u8, b: u8 },
    /// an emergency traffic car (`is_triggered_type`) with its traffic manager index, or `None`
    TrafficLights { request: u64, index: Option<i32> },
}

/// the FlareRender object
#[derive(Resource)]
pub struct FlareRender {
    /// +0 front end (CommonInit's argument)
    pub front_end: bool,
    /// +196 the streak manager (width 128)
    pub streaks: LightStreakManager,
    /// +2160 streak slot per light index
    pub slot_of: [i8; MAX_STREAK_LIGHTS],
    /// `dword_686A3C`
    pub traffic_phase: [f32; 10],
    /// `flt_7192B4`: accumulated top light phase (read by the dummy police car)
    pub phase_sum: f32,
    /// +52 four 36-byte flash records, +36 their timers (< 0 = free)
    pub flash: [FlashRecord; 4],
    pub flash_time: [f32; 4],
}

impl Default for FlareRender {
    /// FlareRender.cpp 0x00123A20
    fn default() -> Self {
        Self {
            front_end: false,
            streaks: LightStreakManager::new(false),
            slot_of: [-1; MAX_STREAK_LIGHTS],
            traffic_phase: TRAFFIC_PHASES.map(f32::from_bits),
            phase_sum: 0.0,
            flash: [FlashRecord::default(); 4],
            flash_time: [-1.0; 4],
        }
    }
}

/// a flash (FlareRender +52 + 36·i): +0 rise and +4 fall time scales, +8 intensity, +12 full screen, +16 world
/// position, +32 size (screen fraction)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FlashRecord {
    pub rise: f32,
    pub fall: f32,
    pub intensity: f32,
    pub screen: bool,
    pub pos: Vec3,
    pub size: f32,
}

/// FlareRender 0x001270E0 from VsAIRaceVehicle 0x00019740 (event 1: {0, 0.05, k, world, car light point, k/2}) and
/// TimelineCameraShaker 0x0017A490 ({item +4, item +8, 0.5, full screen})
#[derive(Message, Clone, Copy, Debug)]
pub struct FlareFlash(pub FlashRecord);

/// `dword_686B40` / `dword_686B34`: rise / fall time of the three flash layers (× the record's rise / fall)
pub const FLASH_RISE: [f32; 3] = [1.0, 1.0, 1.5];
pub const FLASH_FALL: [f32; 3] = [0.5, 2.0, 1.0];
/// `byte_686B30`: layers whose fall is squared
pub const FLASH_SQUARED: [bool; 3] = [true, false, true];
/// `dword_686B1C` (count `dword_686B28` = 3): the layers' shader (+28 "FS0" / +32 "FS1")
pub const FLASH_SHADER: [usize; 3] = [0, 1, 0];
/// `dword_686B2C`: the uv inset (u, v from 0.25 to 0.75 of Spec4)
pub const FLASH_UV_INSET: f32 = 0.25;
/// `flt_624A90`: 0x001270E0 scales the record's fall by it
pub const FLASH_FALL_SCALE: f32 = 10.0;
/// `dword_71A01C` VISOS(0x40, 0xFF, 0x40, 0x20) for shader 0, `dword_71A018` VISOS(0x80 ×4) for shader 1
pub const FLASH_COLOUR: [[u8; 4]; 2] = [[0x40, 0xFF, 0x40, 0x20], [0x80, 0x80, 0x80, 0x80]];

impl FlareRender {
    /// FlareRender.cpp 0x001270E0: the free-slot search is overwritten with 0 before use (original): every flash goes to slot 0
    pub fn add_flash(&mut self, r: FlashRecord) {
        let _free = (0..4).find(|&i| 0.0 > self.flash_time[i]);
        let i = 0;
        self.flash[i] = r;
        self.flash[i].fall *= FLASH_FALL_SCALE;
        self.flash_time[i] = 0.0;
    }
}

/// FlareRender.cpp 0x001267C0 alphas: per layer k, rise over `FLASH_RISE[k]·rise` then fall over `FLASH_FALL[k]·fall` (squared for
/// `FLASH_SQUARED`), × intensity, clamped
pub fn flash_alphas(r: &FlashRecord, t: f32) -> [f32; 3] {
    let mut a = [0f32; 3];
    for k in 0..3 {
        let mut t = t;
        let rise = FLASH_RISE[k] * r.rise;
        let fall = FLASH_FALL[k] * r.fall;
        let v = if rise <= t {
            t -= rise;
            if fall <= t {
                0.0
            } else {
                let v = 1.0 - t / fall;
                if FLASH_SQUARED[k] {
                    v * v
                } else {
                    v
                }
            }
        } else {
            t / rise
        };
        a[k] = (v * r.intensity).clamp(0.0, 1.0);
    }
    a
}

pub mod flash_shaders {
    use super::*;
    /// +28 "FS0" / +32 "FS1": Spec4, blend 12, no z, MODULATE4X
    pub fn flash(_i: usize) -> EffectShader {
        EffectShader::new(EffectTexture::Misc("Spec4".into()), 12).z(false, false).colour_op(5)
    }
}

/// FlareRender.cpp 0x001267C0 (GameManager 0x000BA275 → 0x00127080 per slot, frame ms · 0.001): advance the timer, free the slot when
/// all three layers are out, then draw the layers as one screen quad each — full screen, or `size` around the
/// projected point when 0x00125D90 passes (in front of the camera, and the camera → point ray with flags 0, which
/// tests nothing). `visible` = that test.
pub fn draw_flash(v: &FlareView, r: &FlashRecord, time: &mut f32, dt: f32, visible: &mut dyn FnMut(Vec3) -> bool, draw: &mut EffectDraw) {
    if 0.0 > *time {
        return;
    }
    *time += dt;
    let a = flash_alphas(r, *time);
    if *time > 0.0 && a[0] == 0.0 && a[1] == 0.0 && a[2] == 0.0 {
        *time = -1.0;
        return;
    }
    let (c0, c1) = (FLASH_UV_INSET, 1.0 - FLASH_UV_INSET);
    let uv = [Vec2::new(c0, c0), Vec2::new(c1, c0), Vec2::new(c0, c1), Vec2::new(c1, c1)];
    let pos = if r.screen {
        [Vec2::ZERO, Vec2::new(512.0, 0.0), Vec2::new(0.0, 512.0), Vec2::splat(512.0)]
    } else {
        if !visible(r.pos) {
            return;
        }
        let s = (v.project)(r.pos);
        let (x, y) = (s.x * 512.0, s.y * 512.0);
        let h = r.size * 0.5 * 512.0;
        [Vec2::new(x - h, y - h), Vec2::new(h + x, y - h), Vec2::new(x - h, h + y), Vec2::new(h + x, h + y)]
    };
    let pts: Vec<(Vec2, Vec2)> = pos.into_iter().zip(uv).collect();
    for (k, &sh) in FLASH_SHADER.iter().enumerate() {
        let colour = with_alpha(visos_colour(FLASH_COLOUR[(sh == 1) as usize]), a[k]);
        screen_strip(v, &pts, colour, &flash_shaders::flash(sh), draw);
    }
}

/// CarRender.cpp 0x001192F0 → 0x001257B0(a6 = 1): a dummy police car's top lights, queued on +2256 and drawn by
/// 0x00125720 with the same "rearL" shader — so drawn here after the cars. Phase = fmod(`flt_7192B4` + key, 2π)
/// (the original adds the object address / 64), then the usual step; light count = the police car's (+1748).
pub fn dummy_top_lights(v: &FlareView, matrix: &Mat4, key: u64, count: usize, phase_sum: f32, step: f32, draw: &mut EffectDraw) {
    let d = v.cam - matrix.w_axis.truncate();
    if d.length_squared() >= crate::car_render::DUMMY_LIGHT_RANGE_SQ {
        return;
    }
    let locs = CarFlareLocators { top: crate::car_render::DUMMY_TOP_LIGHTS[..count.min(2)].to_vec(), ..Default::default() };
    let mut st = CarFlareState {
        top_on: [true; 2],
        top_colour: crate::car_render::DUMMY_TOP_COLOURS,
        phase: ((phase_sum as f64 + key as f64) % std::f64::consts::TAU) as f32,
        ..Default::default()
    };
    top_lights(v, matrix, &locs, &mut st, d, step, draw);
}

/// the shaders of 0x00123A20 (blend 12, stencil write mask 0)
pub mod shaders {
    use super::*;

    /// +4.. +16 "flare": texture i; i = 0 z test, no write, bias 128; the others no z
    pub fn flare(i: usize) -> EffectShader {
        let s = EffectShader::new(EffectTexture::Misc(FLARE_TEXTURES[i].into()), 12);
        if i == 0 {
            s.z(true, false).bias(FLARE_BIAS)
        } else {
            s.z(false, false)
        }
    }

    /// +20 / +24 "rearL": LensFlareSecondaryBlob, z test no write, bias 128, MODULATE2X
    pub fn rear() -> EffectShader {
        EffectShader::new(EffectTexture::Misc(FLARE_TEXTURES[2].into()), 12).z(true, false).bias(FLARE_BIAS).colour_op(4)
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (b - a) * t + a
}

/// `_ftol(x)` stored into a byte (the alpha byte of a D3DCOLOR): truncated, wraps past 255 (original)
fn alpha_byte(x: f32) -> u32 {
    (x as i32 as u32) & 0xFF
}

fn with_alpha(c: u32, x: f32) -> u32 {
    (c & 0x00FF_FFFF) | (alpha_byte(x * 255.0) << 24)
}

/// GFXHelper.cpp 0x0012BC40: piecewise linear over (x, y) points
pub fn curve(x: f32, pts: &[[f32; 2]]) -> f32 {
    if pts[0][0] > x {
        return pts[0][1];
    }
    for i in 1..pts.len() {
        if pts[i][0] > x {
            return (pts[i][1] - pts[i - 1][1]) * ((x - pts[i - 1][0]) / (pts[i][0] - pts[i - 1][0])) + pts[i - 1][1];
        }
    }
    pts[pts.len() - 1][1]
}

/// the quad every world flare uses: c0 = p − (b − a)/2, c0 + b, c0 − a, c0 − a + b with the template uvs
/// (0,0) (1,0) (0,1) (1,1) of 0x00123A20
pub fn flare_quad(p: Vec3, a: Vec3, b: Vec3, colour: u32) -> [EffectVertex; 4] {
    let c0 = p - (b - a) * 0.5;
    let c2 = c0 - a;
    [
        EffectVertex::new(c0, colour, Vec2::ZERO),
        EffectVertex::new(c0 + b, colour, Vec2::X),
        EffectVertex::new(c2, colour, Vec2::Y),
        EffectVertex::new(c2 + b, colour, Vec2::ONE),
    ]
}

/// one car as the flares see it (CarRender data)
#[derive(Clone, Debug)]
pub struct FlareCar {
    /// +0 matrix
    pub matrix: Mat4,
    /// +768 `l_head`
    pub head: Vec3,
    pub locs: CarFlareLocators,
    /// +1784 != 0 (traffic)
    pub traffic: bool,
    /// +1828 brake, +1832 reverse level
    pub brake: f32,
    pub reverse: f32,
    /// +2184 / +2200: the car has reverse light prims on that side
    pub has_reverse: [bool; 2],
    /// +2250 light damage
    pub damage: u8,
}

/// screen point (0..1) and the camera of a frame
pub struct FlareView<'a> {
    /// VsBaseCamera +72 position, +88 `m_right_RENDER` (bevy's left), +104 up
    pub cam: Vec3,
    pub r88: Vec3,
    pub up: Vec3,
    /// camera mode 3 and not the front end (`dword_7192A8`)
    pub mode3: bool,
    /// 0x0015EC80
    pub project: &'a dyn Fn(Vec3) -> Vec4,
    /// a point of the 512² pretransformed space back to the world (at `SCREEN_DEPTH`)
    pub screen: &'a dyn Fn(Vec2) -> Vec3,
}

/// a pretransformed quad / strip (Renderer 0x002A9B00), z = 0
fn screen_strip(v: &FlareView, pts: &[(Vec2, Vec2)], colour: u32, sh: &EffectShader, draw: &mut EffectDraw) {
    let s: Vec<EffectVertex> = pts.iter().map(|(p, uv)| EffectVertex::new((v.screen)(*p), colour, *uv)).collect();
    draw.strip(sh, &s);
}

/// FlareRender.cpp 0x001265D0: a w × h quad (w × 4:3) centred on (x, y) of the screen in flare shader `shader`
fn ghost_quad(v: &FlareView, x: f32, y: f32, w: f32, h: f32, shader: usize, colour: u32, draw: &mut EffectDraw) {
    let w = w * ASPECT;
    let x0 = x * 512.0 - w * 0.5;
    let y0 = y * 512.0 - h * 0.5;
    let pts = [
        (Vec2::new(x0, y0), Vec2::ZERO),
        (Vec2::new(x0 + w, y0), Vec2::X),
        (Vec2::new(x0, y0 + h), Vec2::Y),
        (Vec2::new(x0 + w, y0 + h), Vec2::ONE),
    ];
    screen_strip(v, &pts, colour, &shaders::flare(shader), draw);
}

/// FlareRender.cpp 0x00126340: the three lens ghosts on the line from the light through the screen centre
pub fn ghosts(v: &FlareView, p: Vec3, a4: f32, a5: f32, draw: &mut EffectDraw) {
    let s = (v.project)(p);
    let (x, y) = (s.x, s.y);
    if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
        return;
    }
    let (cx, cy) = (0.5f32, 0.5f32);
    let r = ((x - cx) * (x - cx) + (y - cy) * (y - cy)).sqrt();
    let k = (GHOST_RADIUS - r) / GHOST_RADIUS;
    let k = k * k * a4;
    let (mx, my) = ((cx - x) + cx, (cy - y) + cy);
    for (alpha, at, size, shader, colour) in GHOSTS {
        let t = f32::from_bits(at);
        let gx = (1.0 - t) * x + t * mx;
        let gy = (1.0 - t) * y + t * my;
        let size = (f32::from_bits(size) * a5 * a4).min(a5);
        let al = (f32::from_bits(alpha) * k * MODE_SCALE[v.mode3 as usize]).min(1.0);
        ghost_quad(v, gx, gy, size, size, shader, with_alpha(visos_colour(colour), al), draw);
    }
}

/// FlareRender.cpp 0x00125E60: the horizontal "LensFlareStreak" through the light (6-vertex strip)
pub fn streak(v: &FlareView, p: Vec3, a4: f32, a5: f32, draw: &mut EffectDraw) {
    let s = (v.project)(p);
    if !(0.0..=1.0).contains(&s.x) || !(0.0..=1.0).contains(&s.y) {
        return;
    }
    let w = STREAK_WIDTH * GHOST_SIZE * ASPECT;
    let h = (STREAK_H_MAX - STREAK_H_MIN) * a5 + STREAK_H_MIN;
    let (x, y) = (s.x * 512.0, s.y * 512.0);
    let colour = with_alpha(visos_colour(STREAK_COLOUR), a4 * MODE_SCALE[v.mode3 as usize]);
    let pts = [
        (Vec2::new(x - w, y - h * 0.5), Vec2::X),
        (Vec2::new(x - w, y + h * 0.5), Vec2::ONE),
        (Vec2::new(x, y - h * 0.5), Vec2::ZERO),
        (Vec2::new(x, y + h * 0.5), Vec2::Y),
        (Vec2::new(x + w, y - h * 0.5), Vec2::X),
        (Vec2::new(x + w, y + h * 0.5), Vec2::ONE),
    ];
    screen_strip(v, &pts, colour, &shaders::flare(3), draw);
}

/// FlareRender.cpp 0x00124340: one headlight's flare at world `p` (car z axis `z`), size scale `flash`
pub fn head_flare(v: &FlareView, p: Vec3, z: Vec3, flash: f32, draw: &mut EffectDraw) {
    let d = v.cam - p;
    let l = d.length();
    if l > FLARE_RANGE {
        return;
    }
    let dir = d * (1.0 / l);
    let dot = dir.dot(z);
    let mut fade = 1.0;
    let mut size = if l <= FLARE_NEAR {
        (HEAD_SIZE_FAR - HEAD_SIZE_NEAR) * (l / FLARE_NEAR) + HEAD_SIZE_NEAR
    } else {
        fade = 1.0 - (l - FLARE_NEAR) / (FLARE_RANGE - FLARE_NEAR);
        HEAD_SIZE_FAR
    };
    size *= dot.powf(POW_SIZE);
    let alpha = dot.powf(POW_ALPHA) * HEAD_ALPHA * fade;
    let ghost = dot.powf(POW_GHOST);
    let glare = dot.powf(POW_STREAK);
    let colour = with_alpha(visos_colour(HEAD_COLOUR), alpha);
    size *= flash;
    let a = v.up * size;
    let b = v.r88 * size * HEAD_WIDTH;
    draw.strip(&shaders::flare(0), &flare_quad(p, a, b, colour));
    // byte_7192AC (the ray test of 0x00125D90) is never set
    if GLARE_ON && GLARE_RANGE > l {
        let g = 1.0 - l / GLARE_RANGE;
        let e1 = g.powf(GLARE_POW_GHOST);
        let e2 = g.powf(GLARE_POW_STREAK);
        ghosts(v, p, ghost * e1, e1 * GHOST_SIZE * flash, draw);
        streak(v, p, glare * e2, e2, draw);
    }
}

/// a light the frame wants streaked (0x001271B0's arguments)
#[derive(Clone, Copy, Debug)]
pub struct StreakCall {
    pub pos: Vec3,
    pub dist: f32,
    /// 0/1 head left/right, 2/3 rear left/right
    pub kind: usize,
    pub car: usize,
    pub intensity: f32,
}

/// FlareRender.cpp 0x001248F0: brake and reverse light flares of car `car` (index `i`), `cosf` = facing away cosine, `dist`
pub fn rear_flares(v: &FlareView, car: &FlareCar, i: usize, cosf: f32, dist: f32, front_end: bool, draw: &mut EffectDraw, streaks: &mut Vec<StreakCall>, max_depth: f32) {
    if dist > REAR_RANGE || (car.damage & 4 != 0 && car.damage & 8 != 0) {
        return;
    }
    let (alpha_k, base) = if car.traffic {
        (REAR_ALPHA_TRAFFIC, REAR_SIZE)
    } else {
        let a = (1.0 - car.brake) * REAR_ALPHA + REAR_ALPHA_BRAKE * car.brake;
        let s = if front_end { REAR_SIZE } else { (1.0 - car.brake) * REAR_SIZE + REAR_SIZE_BRAKE * car.brake };
        (a, s)
    };
    let t = dist / REAR_RANGE;
    let fade = 1.0 - t * t;
    let size = (REAR_SIZE_FAR - base) * t + base;
    let alpha = cosf * fade * alpha_k;
    let colour = with_alpha(visos_colour(REAR_COLOUR), alpha);
    let a = v.up * size;
    let b = v.r88 * size * REAR_WIDTH;
    let sh = shaders::rear();
    let mut base_pos = car.locs.brake;
    for side in 0..2 {
        if car.damage & (4 << side) == 0 {
            let p = car.matrix.transform_point3(base_pos + REAR_OFFSET);
            draw.strip(&sh, &flare_quad(p, a, b, colour));
            if max_depth > dist {
                streaks.push(StreakCall { pos: p, dist, kind: 2 + side, car: i, intensity: cosf });
            }
        }
        base_pos.x = -base_pos.x;
    }
    if car.reverse != 0.0 {
        let a = v.up * REVERSE_SIZE;
        let b = v.r88 * REVERSE_SIZE;
        let colour = with_alpha(visos_colour(REVERSE_COLOUR), cosf * car.reverse);
        for side in 0..2 {
            if car.damage & (4 << side) == 0 && car.has_reverse[side] {
                let p = car.matrix.transform_point3(car.locs.reverse[side]);
                draw.strip(&sh, &flare_quad(p, a, b, colour));
            }
        }
    }
}

/// FlareRender.cpp 0x00125190: the ten flashing lights of an emergency traffic car (traffic index `index`), `d` = camera − car
pub fn traffic_lights(v: &FlareView, matrix: &Mat4, index: i32, d: Vec3, phase: &[f32; 10], draw: &mut EffectDraw) {
    let l2 = d.length_squared();
    if !(l2 <= TRAFFIC_RANGE * TRAFFIC_RANGE && l2 >= 0.1) {
        return;
    }
    let l = l2.sqrt();
    let t = (l / TRAFFIC_RANGE).min(1.0);
    let k = 1.0 - t * t;
    let size = lerp(TRAFFIC_SIZE[0], TRAFFIC_SIZE[1], t);
    let a = v.up * size;
    let b = v.r88 * size * TRAFFIC_WIDTH;
    let base = visos_colour(TRAFFIC_COLOUR);
    for i in 0..10 {
        let p = matrix.transform_point3(Vec3::from_array(TRAFFIC_LIGHTS[i].map(f32::from_bits)));
        let dir = (v.cam - p).normalize_or_zero();
        let n = matrix.transform_vector3(Vec3::from_array(TRAFFIC_DIRS[i]));
        let dot = dir.dot(n);
        let c = (index as f32 * TRAFFIC_INDEX_PHASE + phase[i]).cos().max(0.0).powf(TRAFFIC_POW);
        let al = dot.max(0.0) * c * k * TRAFFIC_ALPHA;
        draw.strip(&shaders::rear(), &flare_quad(p, a, b, with_alpha(base, al)));
    }
}

/// FlareRender.cpp 0x001257B0 (a6 = 0): the police top lights, `d` = camera − car; advances the car's phase by `step`
pub fn top_lights(v: &FlareView, matrix: &Mat4, locs: &CarFlareLocators, st: &mut CarFlareState, d: Vec3, step: f32, draw: &mut EffectDraw) {
    st.phase += step;
    if st.phase >= std::f32::consts::TAU {
        st.phase -= std::f32::consts::TAU;
    }
    let l2 = d.length_squared();
    if !(l2 <= TOP_RANGE * TOP_RANGE && l2 >= 0.1) {
        return;
    }
    let l = l2.sqrt();
    let t = (l / TOP_RANGE).min(1.0);
    let k = 1.0 - t * t;
    let size = lerp(TOP_SIZE[0], TOP_SIZE[1], t);
    let a = v.up * size * TOP_HEIGHT;
    let b = v.r88 * size * TOP_WIDTH;
    for (i, p) in locs.top.iter().enumerate() {
        if !st.top_on[i] {
            continue;
        }
        let p = matrix.transform_point3(*p);
        // +2336 / +2340 (a copy of +2336)
        let c = (i as f32 * TOP_LIGHT_PHASE + st.phase).cos().max(0.0);
        let al = c * k * TOP_ALPHA;
        draw.strip(&shaders::rear(), &flare_quad(p, a, b, with_alpha(st.top_colour[i], al)));
    }
}

impl FlareRender {
    /// FlareRender.cpp 0x001271B0: a streak record for light `car + 22 · kind` unless it is the player's car
    pub fn add_streak(&mut self, records: &mut Vec<StreakLight>, c: &StreakCall, player: Option<usize>) {
        if self.front_end || Some(c.car) == player {
            return;
        }
        let k = if records.len() >= MAX_LIGHTSTREAK_LIGHTS {
            // 0x0012FA80: the farthest record when farther than this light; the streak range drops to it
            let (k, far) = records.iter().enumerate().fold((None, -9_999_999.0f32), |(k, m), (j, r)| if r.depth > m { (Some(j), r.depth) } else { (k, m) });
            self.streaks.max_depth = far;
            if far <= c.dist {
                None
            } else {
                k
            }
        } else {
            records.push(StreakLight::default());
            Some(records.len() - 1)
        };
        let Some(k) = k else { return };
        let light = c.car + MAX_VEHICLE_COUNT * c.kind;
        debug_assert!(light < MAX_STREAK_LIGHTS, "LightIndex < eMaxStreakLightTypes * MAX_VEHICLE_COUNT");
        records[k] = StreakLight { pos: c.pos, depth: c.dist, intensity: c.intensity, light: light as i16, colour: (c.kind / 2 + 2) as u8 };
    }
}

pub struct FlareRenderPlugin;

impl Plugin for FlareRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlareRender>()
            .add_message::<CarFlareMessage>()
            .add_message::<FlareFlash>()
            .init_resource::<crate::car_render::DummyPoliceCars>()
            .init_resource::<crate::rain_surface::FrontEndRender>()
            .add_systems(PostUpdate, (apply_flare_messages, draw_flares).chain().in_set(crate::effect_draw::EffectOrder::TrackFx).after(crate::track_fx::draw_track_fx));
    }
}

fn apply_flare_messages(
    mut msgs: MessageReader<CarFlareMessage>,
    mut flashes: MessageReader<FlareFlash>,
    mut fr: ResMut<FlareRender>,
    mut cars: Query<(&crate::car_render::CarRender, &mut CarFlareState)>,
) {
    for f in flashes.read() {
        fr.add_flash(f.0);
    }
    for m in msgs.read() {
        let req = match m {
            CarFlareMessage::HeadlightFlash { request, .. } | CarFlareMessage::TopLight { request, .. } | CarFlareMessage::TrafficLights { request, .. } => *request,
        };
        let Some((_, mut st)) = cars.iter_mut().find(|(r, _)| r.request == req) else { continue };
        match *m {
            CarFlareMessage::HeadlightFlash { on, .. } => st.flash = if on { FLASH_TIME } else { 0.0 },
            CarFlareMessage::TopLight { light, on, a, r, g, b, .. } => {
                if light < 2 {
                    st.top_on[light] = on;
                    st.top_colour[light] = u32::from_be_bytes([a, r, g, b]);
                }
            }
            CarFlareMessage::TrafficLights { index, .. } => st.traffic = index,
        }
    }
}

type FlareCarQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static crate::car_render::CarRender,
        &'static Transform,
        Option<&'static crate::track_fx::CarHeadBeams>,
        &'static CarFlareLocators,
        &'static mut CarFlareState,
        Option<&'static crate::car_render::CarLights>,
        Option<&'static crate::car_render::CarLightInput>,
        Option<&'static crate::car_damage_render::CarLightDamage>,
        &'static InheritedVisibility,
    ),
>;

type FlareCamQuery<'w, 's, F> = Query<'w, 's, (&'static GlobalTransform, &'static Projection, &'static bevy::camera::primitives::Frustum), F>;

/// GameManager 0x000B9D04 → 0x00123E60 (frame time), per rendered frame after TrackFX 0x0014A5B0; in the front end
/// FECK2ElementCamera 0x00299C5A (not in the garage mode `dword_7065A4`, and only while `byte_7065C0` is 0) on the
/// front-end cars, with +0 front end set (no streaks, no camera-mode halving). In game it runs inside the view loop
/// (flag 0x10000): once per view with that view's camera, each call stepping the phases by the whole frame time; the
/// flash records (0x00127080) follow the loop once, with player 0's camera.
#[allow(clippy::too_many_arguments)]
fn draw_flares(
    time: Res<Time>,
    clock: Res<crate::effect_draw::EffectClock>,
    mut fr: ResMut<FlareRender>,
    views: crate::view_render::ViewCams,
    fe_cam: FlareCamQuery<(With<crate::rain_surface::FrontEndCamera>, Without<crate::track_render::TrackCamera>)>,
    fe: Res<crate::rain_surface::FrontEndRender>,
    view: Res<crate::track_fx::HeadlightView>,
    others: Option<Res<crate::track_fx::HeadlightViews>>,
    planes: Option<Res<crate::full_screen_effects_manager_pc_xbox::FseCameraPlanes>>,
    dummies: Res<crate::car_render::DummyPoliceCars>,
    physics: Option<Res<velocity_physics::plugin::PhysicsWorldRes>>,
    mut draw: ResMut<EffectDraw>,
    mut cars: FlareCarQuery,
) {
    let others = others.map(|o| *o).unwrap_or_default();
    // per view: its camera mode and the car its camera sits in (CarRender 0x001127F0 with the view's player)
    let game: Vec<_> = views
        .list()
        .into_iter()
        .filter_map(|v| {
            let hv = others.for_view(&view, &v);
            Some((false, v.transform, v.projection?, v.frustum?, v.tag, hv))
        })
        .collect();
    let list = if !game.is_empty() {
        game
    } else {
        match fe_cam.iter().next() {
            Some((g, p, f)) if !fe.garage && !fe.flares_off => vec![(true, *g, p.clone(), f.clone(), None, *view)],
            _ => return,
        }
    };
    if std::env::var("VELOCITY_CAR_FLARES").is_ok_and(|v| v == "0") {
        return;
    }
    for (k, (front_end, g, proj, frustum, tag, view)) in list.into_iter().enumerate() {
    let (g, frustum) = (&g, &frustum);
    fr.front_end = front_end;
    let Projection::Perspective(pp) = &proj else { continue };
    draw.set_view(tag);
    let tan_y = (pp.fov * 0.5).tan();
    let tan_x = tan_y * pp.aspect_ratio;
    let (pos, fwd, right, up) = (g.translation(), g.forward().as_vec3(), g.right().as_vec3(), g.up().as_vec3());
    let near = planes.as_ref().map_or(pp.near, |p| p.near);
    let project = move |w: Vec3| {
        let d = w - pos;
        let ew = fwd.dot(d);
        let x = right.dot(d) / (ew * tan_x);
        let y = up.dot(d) / (ew * tan_y);
        Vec4::new((x + 1.0) / 2.0, (-y + 1.0) / 2.0, 1.0 - near / ew.max(near), ew)
    };
    let depth = SCREEN_DEPTH.max(pp.near * 2.0);
    let screen = move |p: Vec2| crate::light_streak::unproject(p, depth, pos, right, up, fwd, tan_x, tan_y);
    let front_end = fr.front_end;
    let v = FlareView { cam: pos, r88: -right, up, mode3: view.camera_mode == 3 && !front_end, project: &project, screen: &screen };
    // GameManager__BF060 (frame ms) · 0.001
    let dt = if clock.paused { 0.0 } else { time.delta_secs() * clock.scale };
    for p in fr.traffic_phase.iter_mut() {
        *p += dt * TRAFFIC_SPEED * std::f32::consts::TAU;
        if *p >= std::f32::consts::TAU {
            *p -= std::f32::consts::TAU;
        }
    }
    let top_step = dt * TOP_SPEED * std::f32::consts::TAU;
    let mut rows: Vec<_> = cars.iter_mut().filter(|r| r.0.spec.frontend == front_end).collect();
    rows.sort_by_key(|r| r.0.request);
    // dword_74BA70 +4: the player's car (the first race car)
    let player = rows.iter().position(|r| !r.0.spec.traffic);
    let mut calls = Vec::new();
    let max_depth = fr.streaks.max_depth;
    let phase = fr.traffic_phase;
    for (i, (r, t, hb, locs, st, lights, input, dmg, vis)) in rows.iter_mut().enumerate() {
        let matrix = t.to_matrix();
        // +2251 (CarRender 0x001127F0): hidden, outside the view or the camera in this car; +2304 taken as 0
        let in_view = hb.is_none_or(|hb| {
            let c = matrix.transform_point3(hb.sphere.truncate());
            frustum.half_spaces.iter().all(|h| h.normal_d().dot(c.extend(1.0)) + hb.sphere.w > 0.0)
        });
        if !vis.get() || !in_view || view.inside == Some(r.request) {
            continue;
        }
        let mut flash = 1.0;
        if st.flash > 0.0 {
            let m = st.flash % FLASH_PERIOD;
            flash = curve(FLASH_PERIOD - m, &FLASH_CURVE);
            st.flash -= dt;
        }
        let car_pos = matrix.w_axis.truncate();
        let d = pos - car_pos;
        let l2 = d.length_squared();
        if l2 >= FLARE_RANGE * FLARE_RANGE {
            continue;
        }
        let l = l2.sqrt();
        let z = matrix.z_axis.truncate();
        let dz = d.dot(z);
        let cosf = (0.0 - dz) / l;
        let damage = input.map_or(0, |x| x.damage) | dmg.map_or(0, |x| x.0);
        if dz <= 0.0 {
            let car = FlareCar {
                matrix,
                head: hb.map_or(Vec3::ZERO, |h| h.head),
                locs: (*locs).clone(),
                traffic: r.spec.traffic,
                brake: lights.map_or(0.0, |x| x.brake_level),
                reverse: lights.map_or(0.0, |x| x.reverse_level),
                has_reverse: lights.map_or([false; 2], |x| [x.reverse[0].is_some(), x.reverse[1].is_some()]),
                damage,
            };
            rear_flares(&v, &car, i, cosf, l, front_end, &mut draw, &mut calls, max_depth);
        } else if let Some(hb) = hb {
            let mut head = hb.head;
            for side in 0..2 {
                let p = matrix.transform_point3(head);
                if damage & (1 << side) == 0 {
                    head_flare(&v, p, z, flash, &mut draw);
                    if max_depth > l {
                        calls.push(StreakCall { pos: p, dist: l, kind: side, car: i, intensity: 0.0 - cosf });
                    }
                }
                head.x = -head.x;
            }
        }
        if let Some(index) = st.traffic {
            if index >= 0 {
                traffic_lights(&v, &matrix, index, d, &phase, &mut draw);
            }
        }
        top_lights(&v, &matrix, locs, st, d, top_step, &mut draw);
    }
    // the dummy police cars (drawn with the physics objects, before this pass: their deferred lights come first in
    // 0x00125720, with the phase sum before this frame's step)
    if !front_end {
        if let Some(count) = rows.iter().find(|r| r.0.spec.police).map(|r| r.3.top.len()) {
            for d in &dummies.0 {
                dummy_top_lights(&v, &d.matrix, d.key, count, fr.phase_sum, top_step, &mut draw);
            }
        }
    }
    // 0x00125720
    fr.phase_sum += top_step;
    if !front_end {
        let mut records = Vec::with_capacity(MAX_LIGHTSTREAK_LIGHTS);
        for c in &calls {
            fr.add_streak(&mut records, c, player);
        }
        let FlareRender { streaks, slot_of, .. } = &mut *fr;
        streaks.update(&records, slot_of, dt, project, &mut |s, w| {
            crate::light_streak::draw_slot(s, w, pos, right, up, fwd, tan_x, tan_y, &mut draw)
        });
    }
    // GameManager 0x000BA275 → 0x00127080 after the view loop (0x000B3C30 on view 0: player 0's camera): the four
    // flash slots, once a frame, drawn after every view and the mirror over the whole screen (`POST_VIEW_TAG`)
    if !front_end && k == 0 {
        draw.set_view(Some(crate::view_render::POST_VIEW_TAG));
        let mut world = physics.as_ref().map(|p| p.lock());
        let mut visible = |p: Vec3| {
            fwd.dot(p - pos) >= 0.0 && world.as_mut().and_then(|w| w.as_mut()).is_none_or(|w| w.ray_cast(pos.to_array(), p.to_array(), 0, -1).is_none())
        };
        let FlareRender { flash, flash_time, .. } = &mut *fr;
        for i in 0..4 {
            draw_flash(&v, &flash[i], &mut flash_time[i], dt, &mut visible, &mut draw);
        }
    }
    }
    draw.set_view(None);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view<'a>(project: &'a dyn Fn(Vec3) -> Vec4, screen: &'a dyn Fn(Vec2) -> Vec3) -> FlareView<'a> {
        FlareView { cam: Vec3::new(0.0, 0.0, 10.0), r88: Vec3::NEG_X, up: Vec3::Y, mode3: false, project, screen }
    }

    /// the flash curve: on at 1, 1.75 in the two blips, off for the last 0.75 s of each period
    #[test]
    fn flash_curve() {
        assert_eq!(curve(0.1, &FLASH_CURVE), 1.0);
        assert_eq!(curve(0.3, &FLASH_CURVE), 1.75);
        assert_eq!(curve(0.7, &FLASH_CURVE), 1.0);
        assert_eq!(curve(1.2, &FLASH_CURVE), 0.0);
        assert_eq!(curve(FLASH_PERIOD - 4.0 % FLASH_PERIOD, &FLASH_CURVE), 0.0);
    }

    /// a headlight facing the camera at 10 m: 1.3 + 2.2·10/150 tall, twice as wide, alpha 0.4, plus 3 ghosts and the
    /// 6-vertex streak; facing away nothing but a zero-size quad
    #[test]
    fn head_flare_quad() {
        let project = |_: Vec3| Vec4::new(0.5, 0.5, 0.5, 10.0);
        let screen = |p: Vec2| p.extend(0.0);
        let v = view(&project, &screen);
        let mut d = EffectDraw::default();
        head_flare(&v, Vec3::ZERO, Vec3::Z, 1.0, &mut d);
        let q = &d.batches[0].1;
        let h = (HEAD_SIZE_FAR - HEAD_SIZE_NEAR) * (10.0 / FLARE_NEAR) + HEAD_SIZE_NEAR;
        assert!((q[0].pos - Vec3::new(h, h * 0.5, 0.0)).length() < 1e-5, "{:?}", q[0].pos);
        assert!((q[1].pos.x - (h - 2.0 * h)).abs() < 1e-5);
        assert_eq!(q[0].argb >> 24, (0.4f32 * 255.0) as u32);
        // middle flare, ghosts (shader 2 then 1), streak
        assert_eq!(d.batches.len(), 4);
        assert_eq!(d.batches[3].1.len(), 12);
    }

    /// rear flares: brake raises size and alpha, broken lights drop out, traffic alpha is fixed
    #[test]
    fn rear_flare_levels() {
        let project = |_: Vec3| Vec4::ZERO;
        let screen = |p: Vec2| p.extend(0.0);
        let v = view(&project, &screen);
        let mut car = FlareCar {
            matrix: Mat4::IDENTITY,
            head: Vec3::ZERO,
            locs: CarFlareLocators { brake: Vec3::new(0.7, 0.8, -2.0), ..Default::default() },
            traffic: false,
            brake: 1.0,
            reverse: 0.0,
            has_reverse: [false; 2],
            damage: 4,
        };
        let mut d = EffectDraw::default();
        let mut calls = Vec::new();
        rear_flares(&v, &car, 1, 1.0, 0.0, false, &mut d, &mut calls, 150.0);
        // one light (left broken): the right one at x = -0.7 + 0.1
        assert_eq!(d.batches[0].1.len(), 6);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].kind, 3);
        assert!((calls[0].pos.x - (-0.7 + 0.1)).abs() < 1e-6);
        assert_eq!(d.batches[0].1[0].argb >> 24, 255);
        car.traffic = true;
        car.damage = 0;
        let mut d = EffectDraw::default();
        rear_flares(&v, &car, 1, 1.0, 0.0, false, &mut d, &mut calls, 150.0);
        assert_eq!(d.batches[0].1[0].argb >> 24, (0.75f32 * 255.0) as u32);
        assert_eq!(d.batches[0].1.len(), 12);
    }

    /// FlareRender.cpp 0x001270E0 always takes slot 0 and scales the fall by 10; 0x001267C0's three layers at t = 0.1 of a
    /// {0, 0.05, 1} record: falls 0.25 (squared), 1.0, 0.5 (squared)
    #[test]
    fn flash_records() {
        let mut fr = FlareRender::default();
        fr.flash_time[0] = 5.0;
        fr.add_flash(FlashRecord { rise: 0.0, fall: 0.05, intensity: 1.0, ..Default::default() });
        fr.add_flash(FlashRecord { rise: 0.0, fall: 0.05, intensity: 1.0, ..Default::default() });
        assert_eq!(fr.flash_time, [0.0, -1.0, -1.0, -1.0]);
        assert!((fr.flash[0].fall - 0.5).abs() < 1e-6);
        let a = flash_alphas(&fr.flash[0], 0.1);
        assert!((a[0] - 0.36).abs() < 1e-5 && (a[1] - 0.9).abs() < 1e-5 && (a[2] - 0.64).abs() < 1e-5, "{a:?}");
        // a full-screen flash: three quads, then freed once every layer is out
        let project = |_: Vec3| Vec4::ZERO;
        let screen = |p: Vec2| p.extend(0.0);
        let v = view(&project, &screen);
        let mut d = EffectDraw::default();
        let mut t = 0.0;
        let r = FlashRecord { screen: true, ..fr.flash[0] };
        draw_flash(&v, &r, &mut t, 0.1, &mut |_| true, &mut d);
        assert_eq!(d.batches.iter().map(|b| b.1.len()).sum::<usize>(), 18);
        draw_flash(&v, &r, &mut t, 5.0, &mut |_| true, &mut d);
        assert_eq!(t, -1.0);
    }

    /// the dummy's top lights: blue and red at the fixed locators, phase offset by the key
    #[test]
    fn dummy_lights() {
        let project = |_: Vec3| Vec4::ZERO;
        let screen = |p: Vec2| p.extend(0.0);
        let v = view(&project, &screen);
        let mut d = EffectDraw::default();
        dummy_top_lights(&v, &Mat4::IDENTITY, 0, 2, 0.0, 0.0, &mut d);
        // light 0 at phase 0 (cos 1), light 1 at π (cos −1 → 0 alpha)
        let q = &d.batches[0].1;
        assert_eq!(q.len(), 12);
        assert_eq!(q[0].argb & 0x00FF_FFFF, 0x0000_70FF);
        assert_eq!(q[6].argb >> 24, 0);
        // out of range: nothing
        let mut d = EffectDraw::default();
        dummy_top_lights(&v, &Mat4::from_translation(Vec3::new(0.0, 0.0, 300.0)), 0, 2, 0.0, 0.0, &mut d);
        assert!(d.batches.is_empty());
    }

    /// traffic flash alpha wraps past 255 like the original byte store
    #[test]
    fn alpha_wraps() {
        assert_eq!(alpha_byte(1.5 * 255.0), 382 & 0xFF);
        assert_eq!(with_alpha(0x00FF_FFFF, 0.5) >> 24, 127);
    }

    /// streak records: light index car + 22·kind, colour 2 (head) / 3 (rear), the player's car never streaks
    #[test]
    fn streak_records() {
        let mut fr = FlareRender::default();
        let mut rec = Vec::new();
        let c = StreakCall { pos: Vec3::ONE, dist: 20.0, kind: 3, car: 2, intensity: 0.5 };
        fr.add_streak(&mut rec, &c, Some(0));
        fr.add_streak(&mut rec, &StreakCall { car: 0, ..c }, Some(0));
        assert_eq!(rec.len(), 1);
        assert_eq!(rec[0].light, 68);
        assert_eq!(rec[0].colour, 3);
    }
}
