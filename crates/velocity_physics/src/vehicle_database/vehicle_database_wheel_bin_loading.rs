//! src/VehicleDatabase/VehicleDatabaseWheelBinLoading.cpp — WHEELS.BIN (version 1.3): `i16` wheels
//! (72 bytes + 2 strings + `num_names` wide strings), `i16` brake parts (36 bytes + string +
//! `num_strings` strings), `i16` tyres (28 bytes + string); wheel and brake part lookups.

use super::vehicle_database::{BinReader, CVehicleDatabase, Result, bytes_at, f32_at, i16_at, string_hash, u8_at, u32_at};

/// a wheel rim (72 bytes).
#[derive(Debug, Clone, Default)]
pub struct Wheel {
    pub hash: u32,                  // +0x00
    pub path: String,               // +0x04 "\Vehicles\Wheels\"
    pub cost: i16,                  // +0x08
    pub unk_0a: [u8; 2],            // +0x0A
    pub unk_0c: f32,                // +0x0C
    pub size: f32,                  // +0x10 inches, compared against an integer size
    pub vehicle_hash: u32,          // +0x14
    pub unk_18: u32,                // +0x18
    pub string_1c: Option<String>,  // +0x1C
    pub string_20: Option<String>,  // +0x20
    pub names: Vec<Option<String>>, // +0x24 wide
    pub num_names: u8,              // +0x28
    pub unk_29: u8,                 // +0x29
    pub fitted: u8,                 // +0x2A
    pub unk_2b: u8,                 // +0x2B
    pub category: u32,              // +0x2C
    pub unk_30: u32,                // +0x30
    pub label: [u8; 16],            // +0x34 exporter name, not terminated
                                    // +0x44 next
}

/// a brake disc or pad (36 bytes).
#[derive(Debug, Clone, Default)]
pub struct BrakePart {
    pub hash: u32,                    // +0x00
    pub kind: u32,                    // +0x04
    pub cost: i16,                    // +0x08
    pub unk_0a: [u8; 2],              // +0x0A
    pub unk_0c: u32,                  // +0x0C
    pub name: Option<String>,         // +0x10
    pub path: String,                 // +0x14
    pub flags: u8,                    // +0x18 bit 1 = fitted
    pub level: u8,                    // +0x19 read by the brake modifier
    pub num_strings: u8,              // +0x1A
    pub unk_1b: u8,                   // +0x1B
    pub strings: Vec<Option<String>>, // +0x1C
                                      // +0x20 next
}

/// a tyre compound (28 bytes).
#[derive(Debug, Clone, Default)]
pub struct Tyre {
    pub hash: u32,            // +0x00
    pub unk_04: u32,          // +0x04
    pub fitted: u8,           // +0x08
    pub unk_09: u8,           // +0x09
    pub cost: i16,            // +0x0A
    pub kind: u32,            // +0x0C
    pub grip: u32,            // +0x10 0 (softest) .. 4
    pub name: Option<String>, // +0x14
                              // +0x18 next
}

impl Wheel {
    /// VehicleDatabaseWheelBinLoading.cpp 0x0018D440
    pub fn read(r: &mut BinReader, path: &str) -> Result<Self> {
        let b = r.read_data(72)?;
        let mut w = Self {
            hash: u32_at(b, 0x00),
            path: path.to_string(),
            cost: i16_at(b, 0x08),
            unk_0a: bytes_at(b, 0x0A),
            unk_0c: f32_at(b, 0x0C),
            size: f32_at(b, 0x10),
            vehicle_hash: u32_at(b, 0x14),
            unk_18: u32_at(b, 0x18),
            string_1c: None,
            string_20: None,
            names: Vec::new(),
            num_names: u8_at(b, 0x28),
            unk_29: u8_at(b, 0x29),
            fitted: u8_at(b, 0x2A),
            unk_2b: u8_at(b, 0x2B),
            category: u32_at(b, 0x2C),
            unk_30: u32_at(b, 0x30),
            label: bytes_at(b, 0x34),
        };
        w.string_1c = r.read_string()?;
        w.string_20 = r.read_string()?;
        for _ in 0..w.num_names {
            w.names.push(r.read_wide_string()?);
        }
        Ok(w)
    }
}

impl BrakePart {
    /// VehicleDatabaseWheelBinLoading.cpp 0x0018D540
    pub fn read(r: &mut BinReader, path: &str) -> Result<Self> {
        let b = r.read_data(36)?;
        let mut p = Self {
            hash: u32_at(b, 0x00),
            kind: u32_at(b, 0x04),
            cost: i16_at(b, 0x08),
            unk_0a: bytes_at(b, 0x0A),
            unk_0c: u32_at(b, 0x0C),
            name: None,
            path: path.to_string(),
            flags: u8_at(b, 0x18),
            level: u8_at(b, 0x19),
            num_strings: u8_at(b, 0x1A),
            unk_1b: u8_at(b, 0x1B),
            strings: Vec::new(),
        };
        p.name = r.read_string()?;
        for _ in 0..p.num_strings {
            p.strings.push(r.read_string()?);
        }
        Ok(p)
    }
}

impl Tyre {
    /// VehicleDatabaseWheelBinLoading.cpp 0x0018D630
    pub fn read(r: &mut BinReader) -> Result<Self> {
        let b = r.read_data(28)?;
        let mut t = Self {
            hash: u32_at(b, 0x00),
            unk_04: u32_at(b, 0x04),
            fitted: u8_at(b, 0x08),
            unk_09: u8_at(b, 0x09),
            cost: i16_at(b, 0x0A),
            kind: u32_at(b, 0x0C),
            grip: u32_at(b, 0x10),
            name: None,
        };
        t.name = r.read_string()?;
        Ok(t)
    }
}

impl CVehicleDatabase {
    /// VehicleDatabaseWheelBinLoading.cpp 0x0018D680
    pub fn load_wheels(&mut self, r: &mut BinReader, path: &str) -> Result<()> {
        r.read_version(1.3)?;
        self.num_wheels = r.read_i16()?;
        for _ in 0..self.num_wheels {
            let w = Wheel::read(r, path)?;
            self.wheels.push(w);
        }
        self.num_brake_parts = r.read_i16()?;
        for _ in 0..self.num_brake_parts {
            let p = BrakePart::read(r, path)?;
            self.brake_parts.push(p);
        }
        self.num_tyres = r.read_i16()?;
        for _ in 0..self.num_tyres {
            let t = Tyre::read(r)?;
            self.tyres.push(t);
        }
        log::debug!("Wheels, brake parts and tyres read from Binary");
        Ok(())
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018D9F0
    pub fn find_wheel(&self, hash: u32) -> Option<usize> {
        self.wheels.iter().position(|w| w.hash == hash)
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018DAA0
    pub fn find_wheel_by_name(&self, name: &str) -> Option<usize> {
        self.find_wheel(string_hash(name))
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018DAE0
    pub fn find_wheel_of_size(&self, vehicle: u32, category: u32, size: u32) -> Option<usize> {
        self.wheels
            .iter()
            .position(|w| w.vehicle_hash == vehicle && w.category == category && w.size as f64 == size as f64)
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018DBC0 — `n` counts from 1.
    pub fn get_nth_wheel_in_category(&self, category: u32, n: i32) -> Option<usize> {
        let mut i = 0;
        for (k, w) in self.wheels.iter().enumerate() {
            if w.category == category {
                i += 1;
                if i == n {
                    return Some(k);
                }
            }
        }
        None
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018DC90 — `n` counts from 1.
    pub fn get_nth_vehicle_wheel(&self, vehicle: u32, category: u32, n: i32) -> Option<usize> {
        let mut i = 0;
        for (k, w) in self.wheels.iter().enumerate() {
            if w.vehicle_hash == vehicle && w.category == category {
                i += 1;
                if i == n {
                    return Some(k);
                }
            }
        }
        None
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018DD60 — `n` counts from 1; the count check sits
    /// outside the match test, so `n == 0` returns the first wheel.
    pub fn get_nth_wheel_of_size(&self, category: u32, size: u32, n: i32) -> Option<usize> {
        let mut i = 0;
        for (k, w) in self.wheels.iter().enumerate() {
            if w.category == category && w.size as f64 == size as f64 {
                i += 1;
            }
            if i == n {
                return Some(k);
            }
        }
        None
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018DF90
    pub fn count_wheels_in_category(&self, category: u32) -> u32 {
        self.wheels.iter().filter(|w| w.category == category).count() as u32
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018E050
    pub fn count_wheels_of_size(&self, category: u32, size: u32) -> u32 {
        self.wheels.iter().filter(|w| w.category == category && w.size as f64 == size as f64).count() as u32
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018E130
    pub fn count_vehicle_wheels(&self, vehicle: u32, category: u32) -> u32 {
        self.wheels.iter().filter(|w| w.vehicle_hash == vehicle && w.category == category).count() as u32
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018E560 — rim size label hash to inches.
    pub fn get_wheel_size_inches(label: u32) -> u32 {
        if label == string_hash("SIZE1") {
            14
        } else if label == string_hash("SIZE2") {
            16
        } else if label == string_hash("SIZE3") {
            18
        } else if label == -2053124729i32 as u32 {
            19
        } else {
            14
        }
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018E6E0
    pub fn count_brake_parts(&self, kind: u32) -> u32 {
        self.brake_parts.iter().filter(|b| b.kind == kind).count() as u32
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018E7A0
    pub fn get_nth_brake_part(&self, kind: u32, n: u32) -> Option<usize> {
        let r = self.brake_parts.iter().enumerate().filter(|(_, b)| b.kind == kind).nth(n as usize).map(|x| x.0);
        if r.is_none() {
            log::warn!("Unable to find matching brake part (by index)");
        }
        r
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018E880
    pub fn find_brake_part(&self, hash: u32) -> Option<usize> {
        self.brake_parts.iter().position(|b| b.hash == hash)
    }

    /// VehicleDatabaseWheelBinLoading.cpp 0x0018E930
    pub fn find_brake_part_by_name(&self, name: &str) -> Option<usize> {
        self.find_brake_part(string_hash(name))
    }
}
