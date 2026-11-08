//! Render\CarPalette.cpp — a car's paint from its setup colour (`SVehicleSetup` +0x88): the colour record's two
//! D3DCOLORs (filled from its float rgb at database load) and its paint kind, turned into the paint / pearl colours
//! and paint type the ShinyCar set takes (`car_palette_pc_xbox`).

use bevy::prelude::*;

/// VISOS_ColourXBOX.cpp 0x002B1370 — D3DCOLOR from 0..1 floats (each channel `_ftol(x * 255.0)`)
pub fn visos_colour(a: f32, r: f32, g: f32, b: f32) -> u32 {
    let c = |x: f32| ((x as f64 * 255.0) as i64 as u32) & 0xFF;
    (c(a) << 24) | (c(r) << 16) | (c(g) << 8) | c(b)
}

/// 0x00015ED0 — D3DCOLOR to an rgba vector (`byte * 0.003921569f`)
pub fn colour_vec(c: u32) -> Vec4 {
    let k = 0.003921569f32;
    Vec4::new(((c >> 16) & 0xFF) as f32 * k, ((c >> 8) & 0xFF) as f32 * k, (c & 0xFF) as f32 * k, (c >> 24) as f32 * k)
}

/// one paint kind (`unk_706260`, 32 bytes, static init 0x004245A0), copied to the car at +1752 by SetCarColour
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintKind {
    /// +0x00 xyz replaced by the pearl colour, w kept
    pub colour: Vec4,
    pub unk10: f32, // +0x10
    pub unk14: f32, // +0x14
    pub unk18: u32, // +0x18
    /// +0x1C tweak "Paint Type 0=Gloss,1=Pearl,2=Metallic"
    pub paint_type: u32,
}

pub const PAINT_KINDS: [PaintKind; 3] = [
    PaintKind { colour: Vec4::new(1.0, 1.0, 1.0, 0.12), unk10: 0.5, unk14: 0.3, unk18: 0, paint_type: 0 },
    PaintKind { colour: Vec4::new(1.0, 1.0, 1.0, 0.12), unk10: 0.53, unk14: 0.5, unk18: 2, paint_type: 1 },
    PaintKind { colour: Vec4::new(1.0, 1.0, 1.0, 0.12), unk10: 0.825, unk14: 1.2, unk18: 1, paint_type: 2 },
];

/// the parts of a 64-byte colour record (COLOURS.BIN, `VehicleDatabaseColours`) the renderer reads
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CarColour {
    /// +0x0C paint kind (index into `PAINT_KINDS`)
    pub kind: u32,
    /// +0x34 base colour, +0x38 pearl colour (D3DCOLOR)
    pub colour_a: u32,
    pub colour_b: u32,
}

impl CarColour {
    /// CVehicleDatabase::Init 0x00184860 loop — the record's D3DCOLORs from its float rgb (0..255 at +0x1C / +0x28)
    pub fn from_record(kind: u32, rgb_a: [f32; 3], rgb_b: [f32; 3]) -> Self {
        let d = |c: [f32; 3]| visos_colour(1.0, c[0] / 255.0, c[1] / 255.0, c[2] / 255.0);
        Self { kind, colour_a: d(rgb_a), colour_b: d(rgb_b) }
    }

    /// parse a raw 64-byte record
    pub fn parse(rec: &[u8]) -> Option<Self> {
        let u = |o: usize| rec.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
        let f = |o: usize| u(o).map(f32::from_bits);
        Some(Self::from_record(u(0x0C)?, [f(0x1C)?, f(0x20)?, f(0x24)?], [f(0x28)?, f(0x2C)?, f(0x30)?]))
    }

    /// BarrierRender.cpp-labelled 0x0010C960 (SetCarColour): the car's paint kind block (+1752, pearl colour with the
    /// kind's w), colours at +2236 / +2240, then CarPalette_PC_XBOX.cpp 0x00157F80 hands (+2236, +2240) to the ShinyCar
    /// set as paint and pearl. Returns (paint rgb, pearl rgb, paint type). Kinds past the table (3..5, not body colours)
    /// would read the globals after it in the original; here they read as kind 0.
    pub fn apply(&self) -> (Vec3, Vec3, u32) {
        let k = PAINT_KINDS.get(self.kind as usize).copied().unwrap_or(PAINT_KINDS[0]);
        (colour_vec(self.colour_a).truncate(), colour_vec(self.colour_b).truncate(), k.paint_type)
    }
}

/// all colour records of `\Vehicles\COLOURS.BIN` (f32 version 1.1, i16 count, 64-byte records); for viewers and tests,
/// the game reads it through the vehicle database.
pub fn load_colours_bin() -> Vec<CarColour> {
    let Ok(b) = visos::file::platform::read("\\Vehicles\\COLOURS.BIN") else { return Vec::new() };
    let n = b.get(4..6).map(|x| i16::from_le_bytes([x[0], x[1]]).max(0) as usize).unwrap_or(0);
    b.get(6..6 + n * 64).map(|d| d.chunks_exact(64).filter_map(CarColour::parse).collect()).unwrap_or_default()
}

/// 0x0010D7E0: depth bias of "carshadow" / "CarNeonDA" (`dword_686658` in game, `dword_68665C` in the showroom)
pub const SHADOW_DEPTH_BIAS: [i16; 2] = [100, 82];

/// CarPalette.cpp 0x0010D7E0 — "carshadow" (CarRender +16): `\VFX\misc.vcd` "shadow", one stage, blend 3, clamp,
/// z test without write, depth bias 100 (82 in the showroom, CarRender +49), stencil write mask 0 (not modelled)
pub fn carshadow_shader(showroom: bool) -> crate::effect_draw::EffectShader {
    use crate::effect_draw::{EffectShader, EffectTexture};
    EffectShader::new(EffectTexture::Misc("shadow".into()), 3)
        .wrap(false, false)
        .z(true, false)
        .bias(SHADOW_DEPTH_BIAS[showroom as usize])
}

/// CarPalette.cpp 0x0010D7E0 — "CarNeonDA" (CarRender +4): misc "neon1", blend 13 (rgb += src · dest alpha), z test
/// without write, same bias, stencil write mask 0; `ShaderManagerBase 0x002A17D0(1, 0)` (stage bytes +86/+87) is
/// unidentified. `VELOCITY_LIGHTREF_DEBUG=1` draws it with blend 12, as the other dest-alpha effects.
pub fn car_neon_shader(showroom: bool) -> crate::effect_draw::EffectShader {
    use crate::effect_draw::{EffectShader, EffectTexture};
    let blend = if std::env::var_os("VELOCITY_LIGHTREF_DEBUG").is_some() { 12 } else { 13 };
    EffectShader::new(EffectTexture::Misc("neon1".into()), blend).z(true, false).bias(SHADOW_DEPTH_BIAS[showroom as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_bytes_exact() {
        // every integral 0..255 survives the float round trip
        for x in 0..=255u32 {
            let c = CarColour::from_record(0, [x as f32, 0.0, 0.0], [0.0; 3]);
            assert_eq!((c.colour_a >> 16) & 0xFF, x);
        }
        assert_eq!(visos_colour(1.0, 1.0, 0.0, 0.0), 0xFFFF0000);
    }

    #[test]
    fn colours_bin() {
        let c = load_colours_bin();
        assert_eq!(c.len(), 203);
        // kinds 3..5 are the setup's other slots (rims, neons...: setup instance slot types 3, 4, 4), never body paint
        assert_eq!(c.iter().filter(|c| (c.kind as usize) < PAINT_KINDS.len()).count(), 175);
        // first record: (141, 33, 20) / (161, 33, 44), kind 2 = metallic
        assert_eq!(c[0].colour_a, 0xFF8D2114);
        assert_eq!(c[0].apply().2, 2);
    }
}
