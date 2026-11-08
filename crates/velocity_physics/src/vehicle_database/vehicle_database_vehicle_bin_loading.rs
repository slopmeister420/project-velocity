//! src/VehicleDatabase/VehicleDatabaseVehicleBinLoading.cpp — `<car>vehicle.bin` (version 1.5):
//! `i16 count` vehicles; a vehicle is 72 raw bytes + name string + `num_names` wide strings, then
//! the 612-byte vehicle data block with its three 80-byte record lists and 16-byte colour list,
//! then the 12-byte model/audio block (+2 strings) and the 12-byte stats block.
//! also the vehicle lookups (by hash, by tier) and lock state.

use visos::file::Vcd;

use super::vehicle_database::{
    BinReader, CVehicleDatabase, Result, VehicleDatabaseError, bytes_at, f32_at, f32s_at, string_hash, u8_at, u32_at,
    u32s_at,
};

/// a colour choice of a vehicle (16 bytes, data +0x25C).
#[derive(Debug, Clone, Default)]
pub struct VehicleColour {
    pub hash: u32,              // +0x00 colour id
    pub brand: u32,             // +0x04 brand id
    pub colour: Option<usize>,  // +0x08 resolved into the colour list
    pub is_default: u8,         // +0x0C
    pub unk_0d: [u8; 3],        // +0x0D
}

/// the 612-byte vehicle data block (entry +0x38).
#[derive(Debug, Clone)]
pub struct VehicleData {
    pub num_list_a: u8,              // +0x000
    pub num_list_b: u8,              // +0x001
    pub num_list_c: u8,              // +0x002
    pub tier: u8,                    // +0x003 1..8, 100 for cops
    pub num_colours: u8,             // +0x004
    pub unk_005: [u8; 3],            // +0x005
    pub list_a: Vec<[f32; 20]>,      // +0x008 80-byte records
    pub list_b: Vec<[f32; 20]>,      // +0x00C
    pub list_c: Vec<[f32; 20]>,      // +0x010
    pub unk_014: [f32; 144],         // +0x014
    pub unk_254: [f32; 2],           // +0x254
    pub colours: Vec<VehicleColour>, // +0x25C
    pub unk_260: u32,                // +0x260
}

/// model / audio names (12 bytes, entry +0x40).
#[derive(Debug, Clone, Default)]
pub struct VehicleAudio {
    pub model: Option<String>, // +0x00
    pub audio: Option<String>, // +0x04
    pub unk_08: u32,           // +0x08
}

/// 12 bytes at entry +0x3C; `values[0]` is the base the top speed / acceleration getters scale.
#[derive(Debug, Clone, Default)]
pub struct VehicleStats {
    pub values: [u32; 3], // +0x00
}

/// a vehicle (72 bytes).
#[derive(Debug, Clone)]
pub struct VehicleEntry {
    pub hash: u32,                  // +0x00 name hash (physics `***a6`)
    pub name: Option<String>,       // +0x04
    pub unk_08: u32,                // +0x08
    pub unk_0c: u32,                // +0x0C
    pub unk_10: u32,                // +0x10
    pub setup_hash: u32,            // +0x14
    pub setup_hash_2: u32,          // +0x18
    pub setup: Option<usize>,       // +0x1C
    pub setup_2: Option<usize>,     // +0x20
    pub path: String,               // +0x24 vehicle folder
    pub unlock_value: u32,          // +0x28 compared against progress
    pub num_names: u8,              // +0x2C
    pub unk_2d: u8,                 // +0x2D
    pub hidden: u8,                 // +0x2E
    pub unk_2f: u8,                 // +0x2F
    pub names: Vec<Option<String>>, // +0x30 wide
    // +0x34 config: a fresh VsConfigFileRead, filled elsewhere
    pub data: VehicleData,          // +0x38
    pub stats: VehicleStats,        // +0x3C
    pub audio: VehicleAudio,        // +0x40
                                    // +0x44 next
}

/// lock state returned by GetLockState.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleLockState {
    NotFound = 0,
    Unlocked = 1,
    Locked = 2,
    Hidden = 3,
}

impl VehicleData {
    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018C3E0
    pub fn read(r: &mut BinReader) -> Result<Self> {
        let b = r.read_data(612)?;
        let mut d = Self {
            num_list_a: u8_at(b, 0),
            num_list_b: u8_at(b, 1),
            num_list_c: u8_at(b, 2),
            tier: u8_at(b, 3),
            num_colours: u8_at(b, 4),
            unk_005: bytes_at(b, 5),
            list_a: Vec::new(),
            list_b: Vec::new(),
            list_c: Vec::new(),
            unk_014: f32s_at(b, 0x14),
            unk_254: [f32_at(b, 0x254), f32_at(b, 0x258)],
            colours: Vec::new(),
            unk_260: u32_at(b, 0x260),
        };
        let list = |r: &mut BinReader, n: u8| -> Result<Vec<[f32; 20]>> {
            let b = r.read_data(80 * n as usize)?;
            Ok(b.chunks_exact(80).map(|c| f32s_at(c, 0)).collect())
        };
        d.list_a = list(r, d.num_list_a)?;
        d.list_b = list(r, d.num_list_b)?;
        d.list_c = list(r, d.num_list_c)?;
        if d.num_colours != 0 {
            let b = r.read_data(16 * d.num_colours as usize)?;
            d.colours = b
                .chunks_exact(16)
                .map(|c| VehicleColour {
                    hash: u32_at(c, 0),
                    brand: u32_at(c, 4),
                    colour: None,
                    is_default: u8_at(c, 12),
                    unk_0d: bytes_at(c, 13),
                })
                .collect();
        }
        Ok(d)
    }
}

impl VehicleStats {
    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018C590
    pub fn read(r: &mut BinReader) -> Result<Self> {
        Ok(Self { values: u32s_at(r.read_data(12)?, 0) })
    }
}

impl VehicleAudio {
    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018C5D0
    pub fn read(r: &mut BinReader) -> Result<Self> {
        let b = r.read_data(12)?;
        let unk_08 = u32_at(b, 8);
        let model = r.read_string()?;
        let audio = r.read_string()?;
        Ok(Self { model, audio, unk_08 })
    }
}

impl VehicleEntry {
    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018C660
    pub fn read(r: &mut BinReader, path: &str) -> Result<Self> {
        let b = r.read_data(72)?;
        let name = r.read_string()?;
        let num_names = u8_at(b, 0x2C);
        let mut names = Vec::new();
        for _ in 0..num_names {
            names.push(r.read_wide_string()?);
        }
        let data = VehicleData::read(r)?;
        let audio = VehicleAudio::read(r)?;
        let stats = VehicleStats::read(r)?;
        Ok(Self {
            hash: u32_at(b, 0x00),
            name,
            unk_08: u32_at(b, 0x08),
            unk_0c: u32_at(b, 0x0C),
            unk_10: u32_at(b, 0x10),
            setup_hash: u32_at(b, 0x14),
            setup_hash_2: u32_at(b, 0x18),
            setup: None,
            setup_2: None,
            path: path.to_string(),
            unlock_value: u32_at(b, 0x28),
            num_names,
            unk_2d: u8_at(b, 0x2D),
            hidden: u8_at(b, 0x2E),
            unk_2f: u8_at(b, 0x2F),
            names,
            data,
            stats,
            audio,
        })
    }

    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018D2A0 — `all_unlocked` is `byte_6877F0`,
    /// `progress` is `dword_6F71A0`.
    pub fn lock_state(&self, progress: u32, all_unlocked: bool) -> VehicleLockState {
        if all_unlocked {
            return VehicleLockState::Unlocked;
        }
        if self.hidden != 0 {
            VehicleLockState::Hidden
        } else if self.unlock_value > progress {
            VehicleLockState::Locked
        } else {
            VehicleLockState::Unlocked
        }
    }

    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018D390 — not tiers 6, 7, 8 (special, teams, hin).
    pub fn is_race_vehicle(&self) -> bool {
        let t = self.data.tier.wrapping_sub(1);
        t != 5 && t != 6 && t != 7
    }
}

impl CVehicleDatabase {
    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018C7F0
    pub fn load_vehicle_bin(&mut self, vcd: &Vcd, file: &str, path: &str) -> Result<()> {
        let Some(data) = vcd.get(file) else {
            log::error!("Error: Unable to open vehicle file [{file}]");
            return Err(VehicleDatabaseError::NotFound(file.to_string()));
        };
        let mut r = BinReader::new(file, data);
        r.read_version(1.5)?;
        let n = r.read_i16()?;
        self.num_vehicles += n;
        for _ in 0..n {
            let v = VehicleEntry::read(&mut r, path)?;
            self.vehicles.push(v);
            log::debug!("[{file}] info read from binary file");
        }
        Ok(())
    }

    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018CB90
    pub fn find_vehicle(&self, hash: u32) -> Option<usize> {
        self.vehicles.iter().position(|v| v.hash == hash)
    }

    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018CC40
    pub fn find_vehicle_by_name(&self, name: &str) -> Option<usize> {
        self.find_vehicle(string_hash(name))
    }

    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018CD60
    pub fn get_nth_vehicle_in_tier(&self, n: u32, tier: u8) -> Option<usize> {
        self.vehicles.iter().enumerate().filter(|(_, v)| v.data.tier == tier).nth(n as usize).map(|x| x.0)
    }

    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018CE30 — skipping hidden vehicles.
    pub fn get_nth_available_vehicle_in_tier(&self, n: u32, tier: u8, progress: u32, all_unlocked: bool) -> Option<usize> {
        self.vehicles
            .iter()
            .enumerate()
            .filter(|(_, v)| v.data.tier == tier && self.get_lock_state(v.hash, progress, all_unlocked) != VehicleLockState::Hidden)
            .nth(n as usize)
            .map(|x| x.0)
    }

    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018D0C0
    pub fn count_vehicles_in_tier(&self, tier: u8) -> u32 {
        self.vehicles.iter().filter(|v| v.data.tier == tier).count() as u32
    }

    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018D180
    pub fn count_available_vehicles_in_tier(&self, tier: u8, progress: u32, all_unlocked: bool) -> u32 {
        self.vehicles
            .iter()
            .filter(|v| v.data.tier == tier && self.get_lock_state(v.hash, progress, all_unlocked) != VehicleLockState::Hidden)
            .count() as u32
    }

    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018D250
    pub fn get_lock_state(&self, hash: u32, progress: u32, all_unlocked: bool) -> VehicleLockState {
        if all_unlocked {
            return VehicleLockState::Unlocked;
        }
        match self.find_vehicle(hash) {
            Some(v) => self.vehicles[v].lock_state(progress, all_unlocked),
            None => {
                log::error!("Error: VEHICLE ENTRY NOT FOUND");
                VehicleLockState::NotFound
            }
        }
    }

    /// VehicleDatabaseVehicleBinLoading.cpp 0x0018D340
    pub fn is_race_vehicle(&self, hash: u32) -> bool {
        self.find_vehicle(hash).is_some_and(|v| self.vehicles[v].is_race_vehicle())
    }
}
