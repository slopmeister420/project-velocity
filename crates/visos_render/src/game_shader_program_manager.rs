//! Render\Pc_xbox\GameShaderProgramManager_PC_XBOX.cpp — the game's vertex (8) and fragment (6) programs: which
//! `Shaders\*.vxu|*.pxu` file each one is, and the input table each is registered with (register, data type,
//! name or global input). Programs are assembled by `ShaderProgramManager` from the raw files at startup.

use crate::global_mat_input_resolver::{gmi, mat_input_type, GLOBAL_INPUTS};

/// `eGameVertexShaderProgram` names (0x006875D8) -> `Shaders\<name>.vxu`
pub const VERTEX_PROGRAMS: [&str; 8] =
    ["Default", "LinearCopy", "default_packed", "ShinyCarPaint", "Decals", "BeamLights", "BeamLights_Lit", "Danger"];
/// `eGameFragmentShaderProgram` names (0x006875F8) -> `Shaders\<name>.pxu`
pub const FRAGMENT_PROGRAMS: [&str; 6] = ["Default", "TrafficCar", "Characters", "Decals", "BeamLights", "Danger"];

pub mod vp {
    pub const DEFAULT: usize = 0;
    pub const LINEAR_COPY: usize = 1;
    pub const DEFAULT_PACKED: usize = 2;
    pub const SHINY_CAR_PAINT: usize = 3;
    pub const DECALS: usize = 4;
    pub const BEAM_LIGHTS: usize = 5;
    pub const BEAM_LIGHTS_LIT: usize = 6;
    pub const DANGER: usize = 7;
}

pub mod fp {
    pub const DEFAULT: usize = 0;
    pub const TRAFFIC_CAR: usize = 1;
    pub const CHARACTERS: usize = 2;
    pub const DECALS: usize = 3;
    pub const BEAM_LIGHTS: usize = 4;
    pub const DANGER: usize = 5;
}

/// one 32-byte material input description (name, data type +8, register +12, value +16)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgramInput {
    pub name: &'static str,
    pub ty: u8,
    pub reg: u32,
    /// the program's own value (used when nothing resolves the name)
    pub value: Option<[f32; 4]>,
}

/// GlobalMatInputResolver.cpp 0x0012D1A0 — an entry copying a global input's name and type
const fn global(id: usize, reg: u32) -> ProgramInput {
    ProgramInput { name: GLOBAL_INPUTS[id].0, ty: GLOBAL_INPUTS[id].1, reg, value: None }
}

const fn named(name: &'static str, ty: u8, reg: u32) -> ProgramInput {
    ProgramInput { name, ty, reg, value: None }
}

/// GameShaderProgramManager_PC_XBOX.cpp 0x0015D250 — vertex program input tables
pub fn vertex_inputs(program: usize) -> Vec<ProgramInput> {
    use mat_input_type::*;
    let beam = vec![
        global(gmi::MAT_WORLD_VIEW_PROJ, 0),
        global(gmi::MAT_WORLD, 4),
        global(gmi::LIGHTBEAM_TEXTURE_TRANSFORM, 10),
        global(gmi::COMPARTMENT_FADE_ALPHA, 14),
        ProgramInput { name: "UsefulConstants", ty: VECTOR, reg: 15, value: Some([1.0, 0.5, 0.0, 1.0]) },
    ];
    match program {
        vp::DEFAULT => vec![global(gmi::MAT_WORLD_VIEW_PROJ, 0)],
        vp::LINEAR_COPY => vec![global(gmi::MAT_WORLD_VIEW_PROJ, 0), global(gmi::TEXTURE0_SIZE, 4)],
        vp::DEFAULT_PACKED => vec![global(gmi::MAT_WORLD_VIEW_PROJ, 0), global(gmi::TEX_COORD_SCALE_FACTOR, 4)],
        vp::SHINY_CAR_PAINT => vec![
            global(gmi::MAT_WORLD_VIEW_PROJ, 0),
            global(gmi::MAT_WORLD, 4),
            global(gmi::TEXTURE1_SIZE, 8),
            named("envmapMatrix", MATRIX, 9),
            global(gmi::MAT_WORLD, 13),
            global(gmi::CAMERA_DIR, 17),
            named("pearlescentMatrix", MATRIX, 18),
            global(gmi::LIGHTING_DIRS, 22),
            global(gmi::LIGHTING_COLS, 26),
            global(gmi::NEON_UNDER_GLOW_SCALE, 30),
        ],
        vp::DECALS => vec![
            global(gmi::MAT_WORLD_VIEW_PROJ, 0),
            global(gmi::MAT_WORLD, 4),
            global(gmi::TEXTURE1_SIZE, 8),
            named("envmapMatrix", MATRIX, 9),
            global(gmi::CAMERA_DIR, 17),
            global(gmi::LIGHTING_DIRS, 22),
            global(gmi::LIGHTING_COLS, 26),
        ],
        vp::BEAM_LIGHTS => beam,
        vp::BEAM_LIGHTS_LIT => {
            let mut v = beam;
            v.push(global(gmi::LIGHTING_DIRS, 20));
            v.push(global(gmi::LIGHTING_COLS, 24));
            v
        }
        vp::DANGER => vec![global(gmi::MAT_WORLD_VIEW_PROJ, 0), global(gmi::TEXTURE0_SIZE, 4)],
        _ => Vec::new(),
    }
}

/// GameShaderProgramManager_PC_XBOX.cpp 0x0015DB90 — fragment program input tables
pub fn fragment_inputs(program: usize) -> Vec<ProgramInput> {
    use mat_input_type::*;
    match program {
        fp::DEFAULT => vec![
            named("paintColour", VECTOR, 10),
            named("pearlColour", VECTOR, 11),
            named("baseTexture", TEXTURE, 0),
            named("refmapTexture", TEXTURE, 1),
            named("noiseTexture", TEXTURE, 3),
            named("envmapIntensity", VECTOR, 12),
            named("noiseIntensity", VECTOR, 13),
        ],
        fp::TRAFFIC_CAR => vec![named("fPaintColour", VECTOR, 10)],
        fp::CHARACTERS => vec![named("SpecularTexture", TEXTURE, 1), named("SpecularLightColour", VECTOR, 10)],
        fp::DECALS => vec![
            named("baseTexture", TEXTURE, 0),
            named("refmapTexture", TEXTURE, 1),
            named("envmapIntensity", VECTOR, 12),
        ],
        fp::BEAM_LIGHTS => vec![
            named("baseTexture", TEXTURE, 0),
            global(gmi::LIGHTBEAM_TEXTURE, 1),
            global(gmi::MIN_TEXTURE_ALPHA, 10),
        ],
        fp::DANGER => vec![
            global(gmi::INTENSITY, 10),
            global(gmi::SOURCE_TEXTURE, 1),
            global(gmi::BASE_TEXTURE, 0),
        ],
        _ => Vec::new(),
    }
}
