//! src/VehicleDatabase/VehicleDatabaseDecalLoading.cpp — DECALS.BIN (version 1.1): `i16 count`, then
//! per decal 32 raw bytes + name string; decal lookups and setup decal slots.

use super::vehicle_database::{BinReader, CVehicleDatabase, Result, VehicleSetup, i32_at, u32_at};

#[derive(Debug, Clone, Default)]
pub struct Decal {
    pub id_a: u32,            // +0x00
    pub id_b: u32,            // +0x04
    pub hash: u32,            // +0x08
    pub unk_0c: u32,          // +0x0C
    pub unk_10: i32,          // +0x10
    pub kind: u32,            // +0x14 0..3
    pub name: Option<String>, // +0x18
                              // +0x1C next
}

impl Decal {
    /// VehicleDatabaseDecalLoading.cpp 0x00188B30
    pub fn read(r: &mut BinReader) -> Result<Self> {
        let b = r.read_data(32)?;
        let mut d = Self {
            id_a: u32_at(b, 0x00),
            id_b: u32_at(b, 0x04),
            hash: u32_at(b, 0x08),
            unk_0c: u32_at(b, 0x0C),
            unk_10: i32_at(b, 0x10),
            kind: u32_at(b, 0x14),
            name: None,
        };
        d.name = r.read_string()?;
        Ok(d)
    }
}

impl CVehicleDatabase {
    /// VehicleDatabaseDecalLoading.cpp 0x00188960
    pub fn load_decals(&mut self, r: &mut BinReader) -> Result<()> {
        r.read_version(1.1)?;
        self.num_decals = r.read_i16()?;
        for _ in 0..self.num_decals {
            let d = Decal::read(r)?;
            self.decals.push(d);
        }
        log::debug!("Decals  read from Binary");
        Ok(())
    }

    /// VehicleDatabaseDecalLoading.cpp 0x00188BE0
    pub fn find_decal(&self, hash: u32) -> Option<usize> {
        self.decals.iter().position(|d| d.hash == hash)
    }

    /// VehicleDatabaseDecalLoading.cpp 0x00188C90
    pub fn find_decal_by_name(&self, name: &str) -> Option<usize> {
        self.find_decal(super::vehicle_database::string_hash(name))
    }

    fn decal_matches(d: &Decal, kind: u32, id_a: u32, id_b: u32) -> bool {
        d.kind == kind && (id_a == 0 || d.id_a == id_a) && (id_b == 0 || d.id_b == id_b)
    }

    /// VehicleDatabaseDecalLoading.cpp 0x00188CD0
    pub fn get_nth_decal(&self, kind: u32, n: u32, id_a: u32, id_b: u32) -> Option<usize> {
        self.decals.iter().enumerate().filter(|(_, d)| Self::decal_matches(d, kind, id_a, id_b)).nth(n as usize).map(|x| x.0)
    }

    /// VehicleDatabaseDecalLoading.cpp 0x00188DC0 — index of `decal` among the matching decals of its kind.
    pub fn get_decal_index(&self, decal: usize, id_a: u32, id_b: u32) -> u32 {
        let target = &self.decals[decal];
        let mut n = 0;
        for d in &self.decals {
            if Self::decal_matches(d, target.kind, id_a, id_b) {
                if d.hash == target.hash {
                    break;
                }
                n += 1;
            }
        }
        n
    }

    /// VehicleDatabaseDecalLoading.cpp 0x001890A0
    pub fn count_decals(&self, kind: u32, id_a: u32, id_b: u32) -> u32 {
        self.decals.iter().filter(|d| Self::decal_matches(d, kind, id_a, id_b)).count() as u32
    }

    /// VehicleDatabaseDecalLoading.cpp 0x00189280
    pub fn set_setup_decal(setup: &mut VehicleSetup, decal: Option<usize>, kind: u32, sub: u32, slot: u32, flags: u32) {
        let s = slot as usize;
        match kind {
            0 => setup.decal_0 = decal,
            1 => setup.decal_1 = decal,
            3 => {
                if sub == 0 && slot < 10 {
                    setup.decals_3[s] = decal;
                    setup.decals_3_flags[s] = flags;
                } else if sub == 1 && slot < 2 {
                    setup.decals_3_1[s] = decal;
                    setup.decals_3_1_flags[s] = flags;
                } else if sub == 2 && slot < 2 {
                    setup.decals_3_2[s] = decal;
                    setup.decals_3_2_flags[s] = flags;
                } else if sub == 3 && slot < 2 {
                    setup.decals_3_3[s] = decal;
                    setup.decals_3_3_flags[s] = flags;
                }
            }
            2 => {
                if sub == 4 && slot == 0 {
                    setup.decal_2_4 = decal;
                    setup.decal_2_4_flags = flags;
                } else if sub == 5 && slot == 0 {
                    setup.decal_2_5 = decal;
                    setup.decal_2_5_flags = flags;
                }
            }
            _ => {}
        }
    }
}
