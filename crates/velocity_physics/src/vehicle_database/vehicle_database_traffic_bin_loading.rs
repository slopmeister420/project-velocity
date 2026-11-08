//! src/VehicleDatabase/VehicleDatabaseTrafficBinLoading.cpp — TRAFFIC.BIN (version 1.1): `i16 count`,
//! per traffic vehicle 76 raw bytes + name string. `pTrafficSetup` of the physics.

use super::vehicle_database::{BinReader, CVehicleDatabase, Result, f32_at, f32s_at, u32_at};

/// a traffic vehicle (76 bytes).
#[derive(Debug, Clone, Default)]
pub struct TrafficSetup {
    pub hash: u32,                  // +0x00 physics' vehicle name hash
    pub name: Option<String>,       // +0x04
    pub unk_08: u32,                // +0x08
    pub kind: u32,                  // +0x0C
    pub unk_10: u32,                // +0x10 area the traffic car drives in (0x0018AEF0 key)
    pub unk_14: [f32; 8],           // +0x14
    pub half_wheel_span_front_back: f32, // +0x34 physics doubles it to the front-back wheel span
    pub unk_38: f32,                // +0x38
    pub half_track: f32,            // +0x3C physics doubles it to the left-right span
    pub unk_40: f32,                // +0x40
    pub wheel_radius: f32,          // +0x44 copied raw to the vehicle (+4096)
                                    // +0x48 next
}

impl TrafficSetup {
    /// VehicleDatabaseTrafficBinLoading.cpp 0x0018B1F0
    pub fn read(r: &mut BinReader) -> Result<Self> {
        let b = r.read_data(76)?;
        let mut t = Self {
            hash: u32_at(b, 0x00),
            name: None,
            unk_08: u32_at(b, 0x08),
            kind: u32_at(b, 0x0C),
            unk_10: u32_at(b, 0x10),
            unk_14: f32s_at(b, 0x14),
            half_wheel_span_front_back: f32_at(b, 0x34),
            unk_38: f32_at(b, 0x38),
            half_track: f32_at(b, 0x3C),
            unk_40: f32_at(b, 0x40),
            wheel_radius: f32_at(b, 0x44),
        };
        t.name = r.read_string()?;
        Ok(t)
    }
}

impl CVehicleDatabase {
    /// VehicleDatabaseTrafficBinLoading.cpp 0x0018B240
    pub fn load_traffic(&mut self, r: &mut BinReader) -> Result<()> {
        r.read_version(1.1)?;
        self.num_traffic = r.read_i16()?;
        for _ in 0..self.num_traffic {
            let t = TrafficSetup::read(r)?;
            self.traffic.push(t);
        }
        log::debug!("Traffic read from Binary");
        Ok(())
    }

    /// traffic setup by name hash (list +0x6C, link +0x48).
    pub fn find_traffic(&self, hash: u32) -> Option<usize> {
        self.traffic.iter().position(|t| t.hash == hash)
    }

    pub fn find_traffic_by_name(&self, name: &str) -> Option<usize> {
        self.find_traffic(super::vehicle_database::string_hash(name))
    }

    /// VehicleDatabaseTrafficBinLoading.cpp 0x0018B510 — tyre by hash (tyre list +0x64).
    pub fn find_tyre(&self, hash: u32) -> Option<usize> {
        self.tyres.iter().position(|t| t.hash == hash)
    }

    /// VehicleDatabaseTrafficBinLoading.cpp 0x0018B5C0
    pub fn find_tyre_by_name(&self, name: &str) -> Option<usize> {
        self.find_tyre(super::vehicle_database::string_hash(name))
    }

    /// VehicleDatabaseTrafficBinLoading.cpp 0x0018B600
    pub fn count_tyres(&self, kind: u32) -> u32 {
        self.tyres.iter().filter(|t| t.kind == kind).count() as u32
    }

    /// VehicleDatabaseTrafficBinLoading.cpp 0x0018B6C0
    pub fn get_nth_tyre(&self, kind: u32, n: u32) -> Option<usize> {
        self.tyres.iter().enumerate().filter(|(_, t)| t.kind == kind).nth(n as usize).map(|x| x.0)
    }

    /// VehicleDatabaseSetups.cpp 0x0018AEF0 — traffic setups of `area` (+0x10) whose kind (+0x0C) shares
    /// a bit with `kind_mask` (0 = any kind)
    pub fn count_traffic(&self, area: u32, kind_mask: u32) -> u32 {
        self.traffic.iter().filter(|t| (kind_mask == 0 || kind_mask & t.kind != 0) && t.unk_10 == area).count() as u32
    }

    /// VehicleDatabaseSetups.cpp 0x0018AE10 — the `n`th of those (`None` past the end)
    pub fn get_nth_traffic(&self, area: u32, kind_mask: u32, n: u32) -> Option<usize> {
        self.traffic
            .iter()
            .enumerate()
            .filter(|(_, t)| (kind_mask == 0 || kind_mask & t.kind != 0) && t.unk_10 == area)
            .nth(n as usize)
            .map(|x| x.0)
    }
}
