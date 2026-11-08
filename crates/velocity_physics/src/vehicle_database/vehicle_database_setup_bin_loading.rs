//! src/VehicleDatabase/VehicleDatabaseSetupBinLoading.cpp — `<car>setup.bin` (version 1.0): `i16 count`,
//! per predefined setup 216 raw bytes + `num_names` wide strings.

use visos::file::Vcd;

use super::vehicle_database::{
    BinReader, CVehicleDatabase, NUM_BODYPART_TYPES, PartRef, Result, VehicleDatabaseError, bytes_at, u8_at, u32_at,
    u32s_at,
};

/// a predefined setup (216 bytes).
#[derive(Debug, Clone)]
pub struct Setup {
    pub hash: u32,                                               // +0x00
    pub vehicle_hash: u32,                                       // +0x04
    pub bodypart_set_hash: u32,                                  // +0x08
    pub num_names: u8,                                           // +0x0C
    pub unk_0d: [u8; 3],                                         // +0x0D
    pub names: Vec<Option<String>>,                              // +0x10 wide
    pub fit_bodypart: [u32; NUM_BODYPART_TYPES],                 // +0x14 nonzero = slot used
    pub part_hashes: [u32; NUM_BODYPART_TYPES],                  // +0x4C
    pub setup_bodyparts: [Option<PartRef>; NUM_BODYPART_TYPES], // +0x84 m_pSetupBodypartsA
    pub wheel_hash: u32,                                         // +0xBC
    pub flags: u32,                                              // +0xC0 4 = has wheel, 0x80 = hidden
    pub label: [u8; 16],                                         // +0xC4 exporter name, not terminated
                                                                 // +0xD4 next
}

impl Setup {
    /// VehicleDatabaseSetupBinLoading.cpp 0x00189B00
    pub fn read(r: &mut BinReader) -> Result<Self> {
        let b = r.read_data(216)?;
        let mut s = Self {
            hash: u32_at(b, 0x00),
            vehicle_hash: u32_at(b, 0x04),
            bodypart_set_hash: u32_at(b, 0x08),
            num_names: u8_at(b, 0x0C),
            unk_0d: bytes_at(b, 0x0D),
            names: Vec::new(),
            fit_bodypart: u32s_at(b, 0x14),
            part_hashes: u32s_at(b, 0x4C),
            setup_bodyparts: [None; NUM_BODYPART_TYPES],
            wheel_hash: u32_at(b, 0xBC),
            flags: u32_at(b, 0xC0),
            label: bytes_at(b, 0xC4),
        };
        for _ in 0..s.num_names {
            s.names.push(r.read_wide_string()?);
        }
        Ok(s)
    }
}

impl CVehicleDatabase {
    /// VehicleDatabaseSetupBinLoading.cpp 0x00189BB0
    pub fn load_setup_bin(&mut self, vcd: &Vcd, file: &str, _path: &str) -> Result<()> {
        let Some(data) = vcd.get(file) else {
            log::error!("ERROR: Unable to load vehicle file [{file}]");
            return Err(VehicleDatabaseError::NotFound(file.to_string()));
        };
        let mut r = BinReader::new(file, data);
        r.read_version(1.0)?;
        let n = r.read_i16()?;
        self.num_setups += n;
        for _ in 0..n {
            let s = Setup::read(&mut r)?;
            self.setups.push(s);
        }
        Ok(())
    }
}
