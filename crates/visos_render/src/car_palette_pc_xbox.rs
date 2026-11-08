//! Render\Pc_xbox\CarPalette_PC_XBOX.cpp (+ the parts of CarRender.cpp / CarPalette.cpp they call) — the car shader
//! sets that replace a car model's own shaders: `CarShader` (0), `ShinyCar` (1, paint program ShinyCarPaint +
//! Default), `BrakeLight` (2), `ShineyWheel` (3) and `CarDecal` (4, Decals + Decals), and the per-frame values the
//! two programmable sets take (paint/pearl colours, envmap and pearlescent matrices, intensities).

use bevy::prelude::*;

use crate::renderer::shader_program_manager::{MatInput, MatValue, ProgramTexture};

/// shader set index (`CarPalette_PC_XBOX.cpp 0x001577F0` creation order)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CarShaderSet {
    CarShader = 0,
    ShinyCar = 1,
    BrakeLight = 2,
    ShineyWheel = 3,
    CarDecal = 4,
}

fn has(name: &str, s: &str) -> bool {
    name.contains(s)
}

/// GameDataLoading.cpp 0x0009EAF0 — car shader type from the material name (case sensitive `strstr`).
/// `initial` is the record's type before classification (0 for freshly loaded models).
pub fn car_shader_type(name: &str, initial: u32, frontend: bool) -> u32 {
    let mut t = initial;
    if has(name, "_matt") {
        t = if has(name, "paint_matt") { 1 } else { 3 };
    } else if has(name, "decal_1") {
        t = 4;
    } else if has(name, "_seams") {
        if has(name, "decal_seams") {
            t = match t {
                1 => 4,
                2 => 0,
                0 => 6,
                x => x,
            };
        } else if t == 1 {
            t = 6;
        }
    } else if has(name, "_tyre") {
        t = 3;
    }
    if has(name, "logo") {
        if has(name, "logofw") || has(name, "logorw") {
            if t == 1 {
                t = 5;
            }
        } else {
            t = 5;
        }
    }
    if has(name, "_carbon") && t == 0 {
        t = 8;
    }
    if t == 0 {
        t = if has(name, "_paint") || has(name, "decal_0") || has(name, "seams_2") || has(name, "car_seams_0") { 0 } else { 2 };
    }
    if !frontend && has(name, "_noreflect") {
        t = 10;
    }
    t
}

/// CarPalette_PC_XBOX.cpp 0x00157BA0 — which shader set replaces a car shader of this type and name
/// (`None`: wheel tyres keep their own shader)
pub fn shader_set_for(ty: u32, name: &str) -> Option<CarShaderSet> {
    Some(if ty == 0 {
        CarShaderSet::ShinyCar
    } else if matches!(ty, 4 | 5 | 7) {
        CarShaderSet::CarDecal
    } else if ty == 10 || has(name, "wheel") {
        if has(name, "tyre") {
            return None;
        }
        CarShaderSet::ShineyWheel
    } else if has(name, "_brake") {
        CarShaderSet::BrakeLight
    } else {
        CarShaderSet::CarShader
    })
}

/// fixed-function override of the CarShader (0x001564A0) / ShineyWheel (0x00156520) sets
#[derive(Clone, Debug, PartialEq)]
pub struct FixedCarSet {
    /// compiled stage records the port models (CompileStage 0x002A3320): only `eMS_AlphaTest` — reset 0x002A0AA0 then
    /// 0x002A0F40(2) = alpha test ALWAYS ref 0, z test LESSEQUAL; blend 0, clamp, z write and bias stay at the reset
    /// values, so the compiler emits nothing for them and the primitive's own records stand
    pub records: Vec<u8>,
    /// stencil ref 4 (0x002A1140), pass op REPLACE (0x002A12C0(0)); ShineyWheel write mask ~3 (0x002A1240(-4))
    pub stencil_ref: u8,
    pub stencil_write_mask: u32,
    /// ShineyWheel TFACTOR VISOS(1.0, 0.35, 0.35, 0.35) (0x002A1990): read only by the front-end pass
    pub tfactor: u32,
    /// stage flags (0x002A0E40): ShineyWheel 1
    pub stage_flags: u32,
    /// passes (0x002A1A00): ShineyWheel in the front end (CarShaderManager +12) adds pass 1 = 0x00156150: texture
    /// = the set's argument (0 from 0x001577F0: no texture), LERP(TFACTOR; TEXTURE, CURRENT), alpha DISABLE,
    /// 0x002A1DC0(5) — not drawn by the port
    pub passes: u32,
}

/// CarPalette_PC_XBOX.cpp 0x001564A0 / 0x00156520 — the two fixed-function sets. Their pass-0 texture is the
/// texture manager's default (0x0011F260), which SetShaderStage 0x002A4960 code 10 never applies to pass 0: the
/// primitive keeps its own texture (ShaderManager 0x002A4240 loads it from the own stage first).
pub fn fixed_car_set(set: CarShaderSet, frontend: bool) -> Option<FixedCarSet> {
    let records = crate::renderer::shader_manager::compile_alpha_test(2, 0, true, 2).map(|r| r.to_vec()).unwrap_or_default();
    match set {
        CarShaderSet::CarShader => {
            Some(FixedCarSet { records, stencil_ref: 4, stencil_write_mask: u32::MAX, tfactor: 0xFFFF_FFFF, stage_flags: 0, passes: 1 })
        }
        CarShaderSet::ShineyWheel => Some(FixedCarSet {
            records,
            stencil_ref: 4,
            stencil_write_mask: (-4i32) as u32,
            tfactor: crate::car_palette::visos_colour(1.0, 0.35, 0.35, 0.35),
            stage_flags: 1,
            passes: if frontend { 2 } else { 1 },
        }),
        _ => None,
    }
}

/// "CarColour" texture size (CTextureManager::Register 0x0029F2E0 in 0x001577F0: 32x32, format 6)
pub const CAR_COLOUR_SIZE: u32 = 32;
/// 0x001577F0's first fill: red, alpha 0, both ends
pub const CAR_COLOUR_INITIAL: (u32, u32) = (0x00FF_0000, 0x00FF_0000);

/// CarPalette_PC_XBOX.cpp 0x001561C0 — the 32x32 "CarColour" gradient: row r has t = 2·|0.5 − r/32|, every channel
/// `(u8)_ftol(a·t + b·(1 − t))` — `a` (the paint, +2236) at the top and bottom rows, `b` (the pearl, +2240) in the
/// middle row. 0x00157F80 refills it with alpha 0 on every colour change. Pixels are A8R8G8B8 (XGSwizzleRect'd into
/// the texture). Nothing samples it: no data stage names "CarColour", and ShinyCar's fixed-function fallback that
/// would bind it (0x00156F86) sits behind a constant `if (1)` program path.
pub fn car_colour_gradient(a: u32, b: u32) -> Vec<u32> {
    let ch = |c: u32, s: u32| ((c >> s) & 0xFF) as f64;
    let mut out = Vec::with_capacity(1024);
    for i in 0u32..0x400 {
        let row = (i >> 5) as u64;
        let mut t = (row as f64 / 32.0) as f32;
        let d = (0.5 - t).abs();
        t = d + d;
        let u = 1.0 - t;
        let px = |s: u32| -> u32 {
            let v = ch(a, s) * t as f64 + ch(b, s) * u as f64;
            (v as i64 as u8) as u32
        };
        out.push((px(24) << 24) | (px(16) << 16) | (px(8) << 8) | px(0));
    }
    out
}

/// CarPalette_PC_XBOX.cpp 0x00157F80 — the gradient's two ends from the car's colours (alpha cleared)
pub fn car_colour_ends(paint: u32, pearl: u32) -> (u32, u32) {
    (paint & 0x00FF_FFFF, pearl & 0x00FF_FFFF)
}

/// CarRender.cpp tables 0x006866D0 (showroom), 0x00686710, 0x006866F0 (default): per [type == 2, type != 2]
/// { pearl power, paint scale x128, pearl scale, pearl direction y offset }
pub const PAINT_TABLE_SHOWROOM: [[f32; 4]; 2] = [[10.0, 128.0, 4.0, 0.7], [4.0, 128.0, 2.0, 0.7]];
pub const PAINT_TABLE_ALT: [[f32; 4]; 2] = [[10.0, 64.0, 2.5, 0.7], [4.0, 92.0, 1.5, 0.7]];
pub const PAINT_TABLE_DEFAULT: [[f32; 4]; 2] = [[10.0, 92.0, 3.0, 0.7], [4.0, 128.0, 1.75, 0.7]];

/// CarRender.cpp 0x00118120 — paint parameters for a car (`paint_type` = car data +1780)
pub fn paint_params(showroom: bool, alt: bool, paint_type: u32) -> [f32; 4] {
    let i = (paint_type != 2) as usize;
    if showroom {
        PAINT_TABLE_SHOWROOM[i]
    } else if alt {
        PAINT_TABLE_ALT[i]
    } else {
        PAINT_TABLE_DEFAULT[i]
    }
}

/// CarPalette_PC_XBOX.cpp 0x001573E0 — paint and pearl colours (D3DCOLOR as 0..1 rgba) for the ShinyCar set
pub fn paint_colours(colour: Vec4, pearl: Vec4, paint_type: u32, params: [f32; 4]) -> (Vec4, Vec4) {
    let paint = colour * (params[1] / 128.0);
    let mut p = pearl;
    if paint_type != 3 {
        if paint_type != 0 {
            if paint_type == 2 || paint_type == 1 {
                p *= params[2];
                p += colour;
                let avg = (p.x + p.y + p.z) / 3.0;
                if avg > 1.0 {
                    let m = 0.0f32.max(p.x).max(p.y).max(p.z);
                    p *= 1.0 / m;
                }
            }
        } else {
            p = paint;
        }
    }
    (paint, p)
}

/// CarPalette_PC_XBOX.cpp 0x001570F0 (matrix part) — pearlescentMatrix rows: camera direction tilted by the
/// table's y offset (w = pearl power), then (2, 2, 1, 1) which the program uses as the noise uv scale.
pub fn pearlescent_matrix(camera_dir: Vec3, params: [f32; 4]) -> [Vec4; 4] {
    let mut d = camera_dir;
    d.y -= params[3];
    let d = normalise(d);
    [d.extend(params[0]), Vec4::new(2.0, 2.0, 1.0, 1.0), Vec4::Z, Vec4::W]
}

/// VsVector4::Normalise: zero below length 1e-6
fn normalise(v: Vec3) -> Vec3 {
    let l = v.length();
    if l <= 1e-6 { Vec3::ZERO } else { v / l }
}

/// envmap fade direction `unk_7065CC` (static init 0x00424A40)
pub const ENVMAP_UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
/// envmap scale `dword_686620`
pub const ENVMAP_SCALE: f32 = 0.9;

/// row-major d3d 4x4 product a*b
fn mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut r = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            r[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j];
        }
    }
    r
}

fn transpose(m: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut r = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            r[i][j] = m[j][i];
        }
    }
    r
}

/// CarRender.cpp 0x00114D80 + CarPalette.cpp 0x0010CE90 — envmapMatrix registers for a car: the car's rotation,
/// a basis around the half vector between `ENVMAP_UP` and the direction to the camera, scaled by 0.9 about 0.5,
/// transposed for upload. `car` is the car's world transform (game space), `camera` the camera position.
pub fn envmap_matrix(car: Mat4, camera: Vec3) -> [Vec4; 4] {
    // d3d row-vector matrix of the car with its translation row cleared
    let c = car.to_cols_array_2d();
    let mut a = c;
    a[3] = [0.0, 0.0, 0.0, 1.0];
    let car_pos = Vec3::new(c[3][0], c[3][1], c[3][2]);
    let to_cam = normalise(camera - car_pos);
    let h = normalise(ENVMAP_UP + to_cam);
    let v23 = h; // (h, 1)
    let v17 = normalise(v23.cross(Vec3::Z));
    let v29 = v23.cross(v17);
    let v23 = normalise(v23);
    let mut b = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    // rows 0..2 keep the vectors' w (cross products give w = 1 in VsVector4)
    b[1] = [v23.x, v23.y, v23.z, 1.0];
    b[2] = [v29.x, v29.y, v29.z, 1.0];
    b[0] = [v17.x, v17.y, v17.z, 1.0];
    let mut bt = transpose(&b);
    bt[3] = [0.0, 0.0, 0.0, 1.0];
    let s = ENVMAP_SCALE;
    let sm = [[s, 0.0, 0.0, 0.0], [0.0, s, 0.0, 0.0], [0.0, 0.0, s, 0.0], [0.5, 0.5, 0.0, 1.0]];
    let m = mul(&mul(&a, &bt), &sm);
    let t = transpose(&m);
    t.map(Vec4::from_array)
}

/// textures the car sets take: refmap (`EnvMapTest\envmap1`, CarShaderManager +44), glitter noise
/// (`Effect_glitter_002_128`, +16) and the dynamic envmap render target when it exists.
#[derive(Clone, Debug, Default)]
pub struct CarSetTextures {
    pub refmap: Option<ProgramTexture>,
    pub noise: Option<ProgramTexture>,
    /// DangerZoneRender "Envmap" target (0x00122AA0); `None` while the dynamic envmap is off or not rendered
    pub dynamic_envmap: Option<ProgramTexture>,
}

/// per-frame inputs of one car
#[derive(Clone, Debug)]
pub struct CarPaintState {
    /// CarShaderManager +12: shader manager initialised for the front end
    pub frontend: bool,
    pub paint_type: u32,
    pub paint: Vec4,
    pub pearl: Vec4,
    pub envmap: [Vec4; 4],
    pub pearlescent: [Vec4; 4],
}

/// CarPalette_PC_XBOX.cpp 0x00156C00 + 0x001570F0 — ShinyCar material inputs
pub fn shiny_car_inputs(s: &CarPaintState, t: &CarSetTextures) -> Vec<MatInput> {
    let (refmap, env, noise) = match (&t.dynamic_envmap, s.frontend) {
        (Some(d), false) => (Some(d.clone()), Vec4::new(0.6, 0.6, 0.6, 0.8), Vec4::ZERO),
        _ => {
            let n = if s.paint_type == 0 { 0.0 } else { 0.6 };
            (t.refmap.clone(), Vec4::new(0.2, 0.2, 0.2, 0.1), Vec4::splat(n))
        }
    };
    vec![
        MatInput { name: "paintColour", value: MatValue::Vector(s.paint) },
        MatInput { name: "pearlColour", value: MatValue::Vector(s.pearl) },
        MatInput { name: "noiseTexture", value: MatValue::Texture(t.noise.clone()) },
        MatInput { name: "refmapTexture", value: MatValue::Texture(refmap) },
        MatInput { name: "envmapIntensity", value: MatValue::Vector(env) },
        MatInput { name: "noiseIntensity", value: MatValue::Vector(noise) },
        MatInput { name: "envmapMatrix", value: MatValue::Matrix(s.envmap) },
        MatInput { name: "pearlescentMatrix", value: MatValue::Matrix(s.pearlescent) },
    ]
}

/// CarPalette_PC_XBOX.cpp 0x00156680 + 0x00156970 — CarDecal material inputs
pub fn car_decal_inputs(s: &CarPaintState, t: &CarSetTextures) -> Vec<MatInput> {
    let (refmap, env) = if s.frontend {
        (t.refmap.clone(), Vec4::new(0.2, 0.2, 0.2, 0.1))
    } else {
        // in game the decal set reads the dynamic envmap at full intensity (null while it is not rendered)
        (t.dynamic_envmap.clone(), Vec4::ONE)
    };
    vec![
        MatInput { name: "refmapTexture", value: MatValue::Texture(refmap) },
        MatInput { name: "envmapIntensity", value: MatValue::Vector(env) },
        MatInput { name: "envmapMatrix", value: MatValue::Matrix(s.envmap) },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_types() {
        assert_eq!(car_shader_type("350z_paint", 0, false), 0);
        assert_eq!(car_shader_type("350z_paint_matt", 0, false), 1);
        assert_eq!(car_shader_type("350z_tyre", 0, false), 3);
        assert_eq!(car_shader_type("decal_1", 0, false), 4);
        assert_eq!(car_shader_type("350z_logo", 0, false), 5);
        assert_eq!(car_shader_type("glass", 0, false), 2);
        // the primitive's +4 layer index is the starting type: decal seam layers 0/1/2 -> 6 / 4 / 0 (ShinyCar)
        assert_eq!(car_shader_type("side_decal_seams_0", 0, true), 6);
        assert_eq!(car_shader_type("side_decal_seams_1", 1, true), 4);
        assert_eq!(car_shader_type("side_decal_seams_2", 2, true), 0);
        assert_eq!(car_shader_type("logofw_alpha_1", 1, true), 5);
        assert_eq!(shader_set_for(0, "x_paint"), Some(CarShaderSet::ShinyCar));
        assert_eq!(shader_set_for(4, "decal_1"), Some(CarShaderSet::CarDecal));
    }

    #[test]
    fn car_colour_gradient_rows() {
        let (a, b) = car_colour_ends(0xFF10_2030, 0xFFF0_E0D0);
        let g = car_colour_gradient(a, b);
        assert_eq!(g.len(), 1024);
        assert_eq!(g[0], 0x0010_2030);
        assert_eq!(g[31], 0x0010_2030);
        assert_eq!(g[16 * 32], 0x00F0_E0D0);
        // row 8: t = 0.5 -> (0x10 + 0xF0) / 2 = 0x80 etc., truncated
        assert_eq!(g[8 * 32], 0x0080_8080);
        let (r0, r1) = CAR_COLOUR_INITIAL;
        assert!(car_colour_gradient(r0, r1).iter().all(|&p| p == 0x00FF_0000));
    }

    #[test]
    fn fixed_sets_override_alpha_test() {
        use crate::renderer::shader_manager::{d3dcmp, StageState};
        let cs = fixed_car_set(CarShaderSet::CarShader, false).unwrap();
        // own stage: alpha test GREATEREQUAL ref 0x40 (code 2), no z write (code 4), blend 3
        let v: u32 = 0x10000 | (2 << 17) | 1 | (6 << 1) | (0x40 << 4);
        let b = v.to_le_bytes();
        let own = [2u8, 8, 0, 0, b[0], b[1], b[2], b[3], 4, 4, 0, 0, 1, 4, 3, 0, 0];
        let mut s = StageState::from_commands(&own);
        assert!(s.alpha_test && s.alpha_ref == 0x40 && !s.z_write && s.blend_mode == 3);
        s.apply_commands(&cs.records);
        assert!(s.alpha_test && s.z_enable);
        assert_eq!((s.alpha_func, s.alpha_ref, s.z_func), (d3dcmp::ALWAYS, 0, d3dcmp::LESSEQUAL));
        assert!(!s.z_write && s.blend_mode == 3);
        let w = fixed_car_set(CarShaderSet::ShineyWheel, false).unwrap();
        assert_eq!((w.stencil_write_mask, w.stage_flags, w.passes, w.tfactor), (0xFFFF_FFFC, 1, 1, 0xFF59_5959));
        assert_eq!(fixed_car_set(CarShaderSet::ShineyWheel, true).unwrap().passes, 2);
        assert!(fixed_car_set(CarShaderSet::ShinyCar, false).is_none());
    }

    #[test]
    fn envmap_matrix_faces_camera() {
        // camera straight above: the half vector is +y, the normal +y maps to the centre of row 1
        let m = envmap_matrix(Mat4::IDENTITY, Vec3::new(0.0, 10.0, 0.0));
        let n = Vec4::new(0.0, 1.0, 0.0, 1.0);
        let uv = Vec2::new(n.dot(m[0]), n.dot(m[1]));
        assert!((uv.y - (0.9 + 0.5)).abs() < 1e-5, "{uv}");
        assert!((uv.x - 0.5).abs() < 1e-5, "{uv}");
    }
}
