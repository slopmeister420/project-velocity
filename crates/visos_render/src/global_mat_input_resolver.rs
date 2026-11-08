//! Render\GlobalMatInputResolver.cpp — the 30 global material inputs a shader program can name (matrices,
//! lights, texture sizes, effect parameters), resolved by name when a program is bound (ShaderManagerBase.cpp
//! 0x002A08B0: the shader's own inputs first, then this resolver).
//!
//! Per-object and per-view inputs (matrices, `cameraDir`) are evaluated on the gpu by the program wrapper
//! (`DynInput`); lights and texture sizes are filled per material by the caller; the rest hold values set
//! through `set_vector` / `set_texture` (0x0012D090 / 0x0012D130).

use bevy::prelude::*;

use crate::renderer::xgraphics_shader::DynInput;

/// `eMatInputDataType`: 1 vector, 3 matrix 4x3, 4 matrix, 6 vector array, 8/9 matrix arrays, 12/13 textures
pub mod mat_input_type {
    pub const VECTOR: u8 = 1;
    pub const MATRIX: u8 = 4;
    pub const TEXTURE: u8 = 12;
    pub const TEXTURE_VOLUME: u8 = 13;
}

/// global input ids (index into the resolver table, 0x0012C980)
pub mod gmi {
    pub const MAT_WORLD_VIEW_PROJ: usize = 0;
    pub const MAT_WORLD: usize = 1;
    pub const LIGHTBEAM_TEXTURE_TRANSFORM: usize = 2;
    pub const LIGHTING_DIRS: usize = 3;
    pub const LIGHTING_COLS: usize = 4;
    pub const LIGHTBEAM_TEXTURE: usize = 5;
    pub const MAT_WORLD_VIEW: usize = 6;
    pub const MAT_VIEW_PROJ: usize = 7;
    pub const MAT_VIEW: usize = 8;
    pub const MIN_TEXTURE_ALPHA: usize = 9;
    pub const COMPARTMENT_FADE_ALPHA: usize = 10;
    pub const SOURCE_TEXTURE: usize = 11;
    pub const BASE_TEXTURE: usize = 12;
    pub const ALPHA_TEXTURE: usize = 13;
    pub const INTENSITY: usize = 14;
    pub const FOCUS_POINT: usize = 15;
    pub const PROGRESS: usize = 16;
    pub const CONSTANT_COLOUR: usize = 17;
    pub const BLUR_AMOUNT: usize = 18;
    pub const SOURCE_TEXTURE_SIZE: usize = 19;
    pub const VIEWPORT_SIZE: usize = 20;
    pub const INVERSE_VIEWPORT_SIZE: usize = 21;
    pub const TEXTURE0_SIZE: usize = 22;
    pub const TEXTURE1_SIZE: usize = 23;
    pub const TEXTURE2_SIZE: usize = 24;
    pub const TEXTURE3_SIZE: usize = 25;
    pub const PAINT_COLOUR: usize = 26;
    pub const CAMERA_DIR: usize = 27;
    pub const TEX_COORD_SCALE_FACTOR: usize = 28;
    pub const NEON_UNDER_GLOW_SCALE: usize = 29;
    pub const COUNT: usize = 30;
}

/// GlobalMatInputResolver.cpp 0x0012C980 — name and data type of each global input, in table order.
pub const GLOBAL_INPUTS: [(&str, u8); gmi::COUNT] = {
    use mat_input_type::*;
    [
        ("matWorldViewProj", MATRIX),
        ("matWorld", MATRIX),
        ("lightbeamTextureTransform", MATRIX),
        ("lightingDirs", MATRIX),
        ("lightingCols", MATRIX),
        ("lightbeamTexture", TEXTURE_VOLUME),
        ("matWorldView", MATRIX),
        ("matViewProj", MATRIX),
        ("matView", MATRIX),
        ("minTextureAlpha", VECTOR),
        ("fCompartmentFadeAlpha", VECTOR),
        ("SourceTexture", TEXTURE),
        ("baseTexture", TEXTURE),
        ("AlphaTexture", TEXTURE),
        ("fIntensity", VECTOR),
        ("fFocusPoint", VECTOR),
        ("fProgress", VECTOR),
        ("ConstantColour", VECTOR),
        ("fBlurAmount", VECTOR),
        ("fSourceTextureSize", VECTOR),
        ("fViewportSize", VECTOR),
        ("fInverseViewportSize", VECTOR),
        ("texture0Size", VECTOR),
        ("texture1Size", VECTOR),
        ("texture2Size", VECTOR),
        ("texture3Size", VECTOR),
        ("fPaintColour", VECTOR),
        ("cameraDir", VECTOR),
        ("fTexCoordScaleFactor", VECTOR),
        ("fNeonUnderGlowScale", VECTOR),
    ]
};

/// global id of an input name (0x0012D200 compares names)
pub fn global_id(name: &str) -> Option<usize> {
    GLOBAL_INPUTS.iter().position(|(n, _)| *n == name)
}

/// globals whose resolver functions read the current transform / camera (GFXHelper.cpp 0x0012C130..0x0012C5B0)
pub fn dynamic_input(id: usize) -> Option<DynInput> {
    Some(match id {
        gmi::MAT_WORLD_VIEW_PROJ => DynInput::WorldViewProj,
        gmi::MAT_WORLD => DynInput::World,
        gmi::MAT_WORLD_VIEW => DynInput::WorldView,
        gmi::MAT_VIEW_PROJ => DynInput::ViewProj,
        gmi::MAT_VIEW => DynInput::View,
        gmi::CAMERA_DIR => DynInput::CameraDir,
        _ => return None,
    })
}

/// the resolver's stored vector values and textures (the parts not computed per object).
#[derive(Resource, Clone)]
pub struct GlobalMatInputResolver {
    pub vectors: [Vec4; gmi::COUNT],
    pub textures: [Option<Handle<Image>>; gmi::COUNT],
    /// `lightbeamTextureTransform` (GFXHelper 0x0012C910 = the beam object +4, uploaded as stored)
    pub beam_matrix: [Vec4; 4],
}

impl Default for GlobalMatInputResolver {
    /// 0x0012C980 builds the table (values left uninitialised), then GameShaderProgramManager_PC_XBOX.cpp
    /// 0x0015D250 / 0x0015DB90 set `fTexCoordScaleFactor` (8, 8, 8, 1) and `minTextureAlpha` (1, 1, 1, 0);
    /// 0x0015EF10 is the reset state of `fCompartmentFadeAlpha` (1, 1, 1, 1).
    fn default() -> Self {
        let mut s = Self { vectors: [Vec4::ZERO; gmi::COUNT], textures: Default::default(), beam_matrix: [Vec4::ZERO; 4] };
        s.set_vector(gmi::TEX_COORD_SCALE_FACTOR, Vec4::new(8.0, 8.0, 8.0, 1.0));
        s.set_vector(gmi::MIN_TEXTURE_ALPHA, Vec4::new(1.0, 1.0, 1.0, 0.0));
        s.set_vector(gmi::COMPARTMENT_FADE_ALPHA, Vec4::ONE);
        s
    }
}

impl GlobalMatInputResolver {
    /// GlobalMatInputResolver.cpp 0x0012D090 — only vector inputs take a value
    pub fn set_vector(&mut self, id: usize, v: Vec4) {
        if GLOBAL_INPUTS[id].1 == mat_input_type::VECTOR {
            self.vectors[id] = v;
        }
    }

    /// GlobalMatInputResolver.cpp 0x0012D130 — only texture inputs take a texture
    pub fn set_texture(&mut self, id: usize, t: Option<Handle<Image>>) {
        if GLOBAL_INPUTS[id].1 == mat_input_type::TEXTURE {
            self.textures[id] = t;
        }
    }
}
