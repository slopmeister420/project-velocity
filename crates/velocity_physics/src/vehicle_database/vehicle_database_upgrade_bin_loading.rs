//! src/VehicleDatabase/VehicleDatabaseUpgradeBinLoading.cpp — `<car>upgrades.bin` (version 1.2):
//! `i16 count` bodypart sets; a set is 116 raw bytes + `num_names` wide strings + for each of the 14
//! bodypart types `part_counts[t]` parts (36 raw bytes + `num_names` wide strings + 0..2 strings).

use visos::file::Vcd;

use super::vehicle_database::{
    BinReader, CVehicleDatabase, NUM_BODYPART_TYPES, PartRef, Result, VehicleSetup, bytes_at, i16_at, u8_at, u32_at,
};

/// one upgrade part (36 bytes).
#[derive(Debug, Clone, Default)]
pub struct Bodypart {
    pub hash: u32,                  // +0x00
    pub set: usize,                 // +0x04 m_pSet
    pub unk_08: u32,                // +0x08
    pub names: Vec<Option<String>>, // +0x0C wide, one per set name
    pub string_10: Option<String>,  // +0x10 present when the file word is nonzero
    pub string_14: Option<String>,  // +0x14 present when the file word is nonzero
    pub brand_hash: u32,            // +0x18
    pub unk_1c: u32,                // +0x1C
    pub flags: u8,                  // +0x20 bit 3 cleared when bought
    pub level: u8,                  // +0x21 upgrade level read by the performance modifiers
    pub cost: i16,                  // +0x22
}

impl Bodypart {
    /// VehicleDatabaseUpgradeBinLoading.cpp 0x0018B8D0
    pub fn read(r: &mut BinReader, num_names: u8, set: usize) -> Result<Self> {
        let b = r.read_data(36)?;
        let mut p = Self {
            hash: u32_at(b, 0x00),
            set,
            unk_08: u32_at(b, 0x08),
            names: Vec::new(),
            string_10: None,
            string_14: None,
            brand_hash: u32_at(b, 0x18),
            unk_1c: u32_at(b, 0x1C),
            flags: u8_at(b, 0x20),
            level: u8_at(b, 0x21),
            cost: i16_at(b, 0x22),
        };
        for _ in 0..num_names {
            p.names.push(r.read_wide_string()?);
        }
        if u32_at(b, 0x10) != 0 {
            p.string_10 = r.read_string()?;
        }
        if u32_at(b, 0x14) != 0 {
            p.string_14 = r.read_string()?;
        }
        Ok(p)
    }
}

/// a vehicle's bodypart set (116 bytes).
#[derive(Debug, Clone, Default)]
pub struct BodypartSet {
    pub hash: u32,                                   // +0x00
    pub vehicle_hash: u32,                           // +0x04
    pub vehicle: Option<usize>,                      // +0x08 set by LinkBodypartSets
    pub path: String,                                // +0x0C vehicle folder
    pub names: Vec<Option<String>>,                  // +0x10 wide
    pub part_counts: [u8; NUM_BODYPART_TYPES],       // +0x14
    pub unk_22: [u8; 2],                             // +0x22
    pub parts: [Vec<Bodypart>; NUM_BODYPART_TYPES],  // +0x24
    pub num_names: u8,                               // +0x5C
    pub unk_5d: [u8; 3],                             // +0x5D
    pub label: [u8; 16],                             // +0x60 exporter name, not terminated
                                                     // +0x70 next
}

impl BodypartSet {
    /// VehicleDatabaseUpgradeBinLoading.cpp 0x0018B9C0 — `index` is this set's position in the list.
    pub fn read(r: &mut BinReader, path: &str, index: usize) -> Result<Self> {
        let b = r.read_data(116)?;
        let mut s = Self {
            hash: u32_at(b, 0x00),
            vehicle_hash: u32_at(b, 0x04),
            vehicle: None,
            path: path.to_string(),
            names: Vec::new(),
            part_counts: bytes_at(b, 0x14),
            unk_22: bytes_at(b, 0x22),
            parts: Default::default(),
            num_names: u8_at(b, 0x5C),
            unk_5d: bytes_at(b, 0x5D),
            label: bytes_at(b, 0x60),
        };
        for _ in 0..s.num_names {
            s.names.push(r.read_wide_string()?);
        }
        for t in 0..NUM_BODYPART_TYPES {
            for _ in 0..s.part_counts[t] {
                let p = Bodypart::read(r, s.num_names, index)?;
                s.parts[t].push(p);
            }
        }
        Ok(s)
    }
}

impl CVehicleDatabase {
    /// VehicleDatabaseUpgradeBinLoading.cpp 0x0018BB50 — false when the file is missing.
    pub fn load_upgrade_bin(&mut self, vcd: &Vcd, file: &str, path: &str) -> Result<bool> {
        let Some(data) = vcd.get(file) else {
            log::warn!("WARNING: Unable to load vehicle file [{file}]");
            return Ok(false);
        };
        let mut r = BinReader::new(file, data);
        r.read_version(1.2)?;
        let n = r.read_i16()?;
        self.num_bodypart_sets += n;
        for _ in 0..n {
            let s = BodypartSet::read(&mut r, path, self.bodypart_sets.len())?;
            self.bodypart_sets.push(s);
            log::debug!("[{file}] bodypart set info read from binary file");
        }
        Ok(true)
    }

    /// VehicleDatabaseUpgradeBinLoading.cpp 0x0018BE60
    pub fn find_bodypart_set(&self, hash: u32) -> Option<usize> {
        self.bodypart_sets.iter().position(|s| s.hash == hash)
    }

    /// VehicleDatabaseUpgradeBinLoading.cpp 0x0018BF10
    pub fn find_bodypart_set_by_name(&self, name: &str) -> Option<usize> {
        self.find_bodypart_set(super::vehicle_database::string_hash(name))
    }

    /// VehicleDatabaseUpgradeBinLoading.cpp 0x0018BF50
    pub fn find_bodypart(&self, set: Option<usize>, part_type: usize, hash: u32) -> Option<PartRef> {
        let Some(si) = set else {
            log::warn!("Invalid set passed to FindBodypart!");
            return None;
        };
        let s = &self.bodypart_sets[si];
        for i in 0..s.part_counts[part_type] as usize {
            if s.parts[part_type][i].hash == hash {
                return Some(PartRef { set: si, part_type, index: i });
            }
        }
        log::warn!("Unable to match part name to actual part");
        None
    }

    /// VehicleDatabaseUpgradeBinLoading.cpp 0x0018C170 — cost of the fitted bodyparts; `mark_bought`
    /// clears flag bit 3 on each.
    pub fn get_bodyparts_cost(&mut self, setup: &VehicleSetup, mark_bought: bool) -> i32 {
        let mut cost = 0i32;
        for p in setup.fitted_bodyparts.iter().flatten() {
            let part = &mut self.bodypart_sets[p.set].parts[p.part_type][p.index];
            cost += part.cost as i32;
            if mark_bought {
                part.flags &= !8;
            }
        }
        cost
    }
}
