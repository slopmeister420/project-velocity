//! src/VehicleDatabase/VehicleDatabaseColours.cpp — COLOURS.BIN (version 1.1): `i16 count` then
//! `count` raw 64-byte colours; colour lookups by id / brand / kind.

use super::vehicle_database::{BinReader, CVehicleDatabase, Result, VehicleSetup, bytes_at, f32s_at, u8_at, u32_at};

/// one paint colour (64 bytes, copied verbatim from the file).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Colour {
    pub hash: u32,         // +0x00 colour id
    pub brand: u32,        // +0x04 brand id
    pub available: u8,     // +0x08
    pub unk_09: [u8; 3],   // +0x09
    pub unk_0c: u32,       // +0x0C slot type (setup instance: 0, 3, 4, 4)
    pub kind: u32,         // +0x10 matched by the brand queries (3, 5 used by the randomiser)
    /// +0x14/+0x18 = setup +0x9C / +0xA0 on the body colour: mix-your-own hue (0..1529) and shade (-255..255, i32)
    pub unk_14: [u32; 2],
    pub colour_a: [f32; 3], // +0x1C 0..255
    pub colour_b: [f32; 3], // +0x28 0..255
    /// +0x34 / +0x38 = setup +0xBC / +0xC0 packed colours (b | g << 8 | r << 16 | a << 24); mix-your-own writes them
    pub unk_34: u32,
    pub unk_38: u32,
    pub unk_3c: u32,       // +0x3C
}

impl Colour {
    pub fn parse(b: &[u8]) -> Self {
        Self {
            hash: u32_at(b, 0x00),
            brand: u32_at(b, 0x04),
            available: u8_at(b, 0x08),
            unk_09: bytes_at(b, 0x09),
            unk_0c: u32_at(b, 0x0C),
            kind: u32_at(b, 0x10),
            unk_14: [u32_at(b, 0x14), u32_at(b, 0x18)],
            colour_a: f32s_at(b, 0x1C),
            colour_b: f32s_at(b, 0x28),
            unk_34: u32_at(b, 0x34),
            unk_38: u32_at(b, 0x38),
            unk_3c: u32_at(b, 0x3C),
        }
    }
}

impl Colour {
    /// setup +0x9C (body colour): mix-your-own hue
    pub fn hue(&self) -> i32 {
        self.unk_14[0] as i32
    }
    /// setup +0xA0 (body colour): mix-your-own shade
    pub fn shade(&self) -> i32 {
        self.unk_14[1] as i32
    }
    pub fn set_hue_shade(&mut self, hue: i32, shade: i32) {
        self.unk_14 = [hue as u32, shade as u32];
    }
}

/// Armco.h 0x00013940 (pack) via CarDamage 0x000763F0 (v / 255): xyz = r,g,b, alpha = 255
fn pack_unit(v: [f32; 3]) -> u32 {
    let d = |x: f32| (x / 255.0f32) as f64;
    let b = |x: f64| (x * 255.0) as i32 as u8 as u32;
    255 << 24 | b(d(v[0]) as f32 as f64) << 16 | b(d(v[1]) as f32 as f64) << 8 | b(d(v[2]) as f32 as f64)
}

/// ColourToVec 0x00015ED0 xyz, then `* 255` (VsVector4::operator* 0x00024020)
fn unpack_255(c: u32) -> [f32; 3] {
    let k = 0.003921568859368563f64 as f32;
    let f = |b: u32| ((b & 255) as f32 * k) * 255.0f32;
    [f(c >> 16), f(c >> 8), f(c)]
}

/// VsClamp 0x000141E0
fn vs_clamp(v: f32, lo: f32, hi: f32) -> f32 {
    if v > lo { if v > hi { hi } else { v } } else { lo }
}

/// x87 luminance chain of 0x00187BA0 (partials stored as f32)
fn luma(v: &[f32; 3]) -> f32 {
    let p1 = (v[0] as f64 * 0.3f32 as f64) as f32;
    let p2 = (v[1] as f64 * 0.5 + p1 as f64) as f32;
    (v[2] as f64 * f32::from_bits(1043140182) as f64 + p2 as f64) as f32
}

impl CVehicleDatabase {
    /// VehicleDatabaseColours.cpp 0x00187880 — hue wheel (index 0..1529, wraps once) to a packed colour
    pub fn hue_wheel_colour(&self, hue: i32) -> u32 {
        let mut a = hue;
        if a < 0 {
            a += 1530;
        } else if a >= 1530 {
            a -= 1530;
        }
        let f = |i: i32| i as f32;
        let v = match a {
            0..=254 => [255.0, 0.0, f(a)],
            255..=509 => [f(255 - (a - 255)), 0.0, 255.0],
            510..=764 => [0.0, f(a - 510), 255.0],
            765..=1019 => [0.0, 255.0, f(255 - (a - 765))],
            1020..=1274 => [f(a - 1020), 255.0, 0.0],
            1275..=1529 => [255.0, f(255 - (a - 1275)), 0.0],
            _ => {
                debug_assert!(false, "0 && \"Invalid colour index!\"");
                [0.0; 3]
            }
        };
        pack_unit(v)
    }

    /// VehicleDatabaseColours.cpp 0x00187BA0 — mix-your-own: body colour hue (+0x9C) and shade (+0xA0)
    /// -> the two packed colours at setup +0xBC / +0xC0 (shade is clamped to -255..255 and written back)
    pub fn update_mix_colour(&self, setup: &mut VehicleSetup) {
        let c = &mut setup.colours[0];
        let (hue, shade) = (c.hue(), c.shade());
        let mut h0 = hue + 5; // dword_687974 = 5.0
        let mut h1 = hue; // dword_68796C = 0.0
        if h0 > 1529 {
            h0 -= 1529;
        }
        if h1 < 0 {
            h1 += 1529;
        }
        let mut a = unpack_255(self.hue_wheel_colour(h0));
        let mut b = unpack_255(self.hue_wheel_colour(h1));
        for v in a.iter_mut().chain(b.iter_mut()) {
            *v = vs_clamp((*v as f64 - shade as f64) as f32, 0.0, 255.0);
        }
        c.unk_14[1] = vs_clamp(shade as f32, -255.0, 255.0) as i32 as u32;
        let la = luma(&a);
        let lb = luma(&b);
        if lb > la {
            let s = la / lb;
            for v in &mut b {
                *v *= s;
            }
        }
        c.unk_34 = pack_unit(a);
        c.unk_38 = pack_unit(b);
    }

    /// VehicleDatabaseColours.cpp 0x001886C0
    pub fn load_colours(&mut self, r: &mut BinReader) -> Result<()> {
        r.read_version(1.1)?;
        self.total_colours = r.read_i16()?;
        let n = self.total_colours.max(0) as usize;
        let b = r.read_data(n << 6)?;
        self.colours = b.chunks_exact(64).map(Colour::parse).collect();
        log::debug!("Colour list read from Binary");
        Ok(())
    }

    /// VehicleDatabaseColours.cpp 0x001888C0 (VehicleDatabase.h inline)
    pub fn get_colour(&self, index: i16) -> Option<usize> {
        debug_assert!(index < self.total_colours, "s16Index < m_s16TotalColours");
        (index >= 0 && index < self.total_colours).then_some(index as usize)
    }

    /// VehicleDatabaseColours.cpp 0x00188060 — copy a colour into the setup's body colour slot.
    pub fn set_setup_colour(&self, setup: &mut VehicleSetup, colour: Option<usize>) {
        if let Some(c) = colour {
            setup.colours[0] = self.colours[c].clone();
        }
    }

    /// VehicleDatabaseColours.cpp 0x00188120 — the vehicle's first colour flagged as default.
    pub fn get_default_vehicle_colour(&self, vehicle: Option<usize>) -> Option<usize> {
        let v = &self.vehicles[vehicle?];
        if v.data.num_colours == 0 {
            return None;
        }
        v.data.colours.iter().take(v.data.num_colours as usize).find(|c| c.is_default != 0).and_then(|c| c.colour)
    }

    /// VehicleDatabaseColours.cpp 0x001882A0
    pub fn get_vehicle_colour(&self, vehicle: Option<usize>, index: i16) -> Option<usize> {
        match vehicle {
            Some(v) if self.vehicles[v].data.num_colours != 0 => {
                let d = &self.vehicles[v].data;
                if (index as i32) < d.num_colours as i32 { d.colours[index as usize].colour } else { None }
            }
            _ => self.get_colour(index),
        }
    }

    /// VehicleDatabaseColours.cpp 0x00188330
    pub fn find_colour(&self, brand: u32, hash: u32) -> Option<usize> {
        let r = self.colours.iter().position(|c| c.brand == brand && c.hash == hash);
        if r.is_none() {
            log::warn!("WARNING: Unable to match IDs to a colour");
        }
        r
    }

    /// VehicleDatabaseColours.cpp 0x001883D0 — brands of `kind` (consecutive runs counted once).
    pub fn count_colour_brands(&self, kind: u32, include_unavailable: bool) -> u32 {
        let mut last = 0;
        let mut n = 0;
        for c in &self.colours {
            if c.brand != last && c.kind == kind && (include_unavailable || c.available != 0) {
                n += 1;
                last = c.brand;
            }
        }
        n
    }

    /// VehicleDatabaseColours.cpp 0x00188490
    pub fn get_nth_colour_brand(&self, kind: u32, n: u32, include_unavailable: bool) -> u32 {
        let mut last = 0;
        let mut i = 0;
        for c in &self.colours {
            if c.brand != last && c.kind == kind && (include_unavailable || c.available != 0) {
                if i == n {
                    return c.brand;
                }
                i += 1;
                last = c.brand;
            }
        }
        0
    }

    /// VehicleDatabaseColours.cpp 0x00188570
    pub fn count_matching_colour_brand(&self, brand: u32, kind: u32) -> u32 {
        self.colours.iter().filter(|c| c.brand == brand && c.kind == kind).count() as u32
    }

    /// VehicleDatabaseColours.cpp 0x00188600
    pub fn get_nth_matching_colour_brand(&self, brand: u32, kind: u32, n: u32) -> Option<usize> {
        let r = self.colours.iter().enumerate().filter(|(_, c)| c.brand == brand && c.kind == kind).nth(n as usize);
        if r.is_none() {
            log::warn!("WARNING: No matching colour found in GetNthMatchingColourBrand");
        }
        r.map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_colour_wheel() {
        let db = CVehicleDatabase::default();
        // wheel corners: red, magenta-ish end of segment 0 stays full red/blue mix, wrap at 1530
        assert_eq!(db.hue_wheel_colour(0) >> 24, 255);
        assert_eq!(db.hue_wheel_colour(1530), db.hue_wheel_colour(0));
        assert_eq!(db.hue_wheel_colour(-1530 + 1530), db.hue_wheel_colour(0));
        let mut s = VehicleSetup::new();
        s.colours[0].set_hue_shade(300, 400);
        db.update_mix_colour(&mut s);
        assert_eq!(s.colours[0].shade(), 255); // clamped and written back
        assert_eq!(s.colours[0].unk_34 & 0xFFFFFF, 0); // shade 255 turns every channel to 0
        s.colours[0].set_hue_shade(0, 0);
        db.update_mix_colour(&mut s);
        assert_eq!(s.colours[0].unk_34 >> 24, 255);
        assert_ne!(s.colours[0].unk_34, s.colours[0].unk_38);
    }
}
