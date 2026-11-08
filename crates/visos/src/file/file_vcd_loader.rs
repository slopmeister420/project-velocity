//! File\FILE_VCDLoader.cpp — `VsVCDLoader`: whole-pack loads kept resident, files fetched by name.

use std::collections::HashMap;
use std::sync::Arc;

use crate::file::vs_file_directory::{Vcd, VcdError};
use crate::handy::handy_crc;

/// default `m_dMaxVCDCount` (Init, 0x003054F0)
pub const MAX_VCD_COUNT: usize = 40;

/// `VsLoadedVCD` — a resident pack plus crc map and "touched" bookkeeping.
pub struct VsLoadedVCD {
    pub name_hash: u32,
    pub vcd: Arc<Vcd>,
    crc_map: HashMap<u32, usize>,
    touched: Vec<bool>,
}

impl VsLoadedVCD {
    /// FILE_VCDLoader.cpp 0x00305070 — later duplicates overwrite earlier ones in the map.
    fn build(name: &str, vcd: Vcd) -> Self {
        let mut crc_map = HashMap::new();
        for (i, e) in vcd.crc_entries.iter().enumerate() {
            crc_map.insert(e.crc, i);
        }
        let touched = vec![false; vcd.len()];
        Self { name_hash: handy_crc::name_hash(name), vcd: Arc::new(vcd), crc_map, touched }
    }

    /// FILE_VCDLoader.cpp 0x003053C0 — `None` is the original's -2.
    pub fn get_file(&mut self, name: &str) -> Option<&[u8]> {
        let i = *self.crc_map.get(&handy_crc::name_hash(name))?;
        self.touched[i] = true;
        let e = self.vcd.entries().nth(i)?;
        self.vcd.data(&e)
    }

    /// read-only lookup that does not mark the entry touched.
    pub fn peek_file(&self, name: &str) -> Option<&[u8]> {
        let i = *self.crc_map.get(&handy_crc::name_hash(name))?;
        let e = self.vcd.entries().nth(i)?;
        self.vcd.data(&e)
    }

    /// FILE_VCDLoader.cpp 0x00305270
    pub fn reset_touched(&mut self) {
        self.touched.iter_mut().for_each(|t| *t = false);
    }

    /// FILE_VCDLoader.cpp 0x00305310 — names never fetched since the last reset.
    pub fn report_untouched(&self) -> Vec<&str> {
        log::debug!("******** {} ********", self.vcd.path);
        let v: Vec<&str> = self
            .touched
            .iter()
            .zip(&self.vcd.file_entries)
            .filter(|(t, _)| !**t)
            .map(|(_, e)| e.name.as_str())
            .collect();
        for n in &v {
            log::debug!("{} not touched", n);
        }
        log::debug!("******** {} end ********", self.vcd.path);
        v
    }
}

/// `VsVCDLoader` — table of loaded packs keyed by the hash of the path they were loaded from.
pub struct VsVCDLoader {
    max_vcd_count: usize,
    loaded: HashMap<u32, VsLoadedVCD>,
}

impl Default for VsVCDLoader {
    /// FILE_VCDLoader.cpp 0x00305C80
    fn default() -> Self {
        Self { max_vcd_count: MAX_VCD_COUNT, loaded: HashMap::new() }
    }
}

impl VsVCDLoader {
    /// FILE_VCDLoader.cpp 0x00305750 — loads the whole pack; returns its key.
    pub fn load_vcd(&mut self, path: &str) -> Result<u32, VcdError> {
        assert!(self.loaded.len() < self.max_vcd_count, "MAXIMUM NUMBER OF LOADED VCDS REACHED.");
        let vcd = Vcd::open(path)?;
        let l = VsLoadedVCD::build(path, vcd);
        let key = l.name_hash;
        self.loaded.insert(key, l);
        Ok(key)
    }

    /// FILE_VCDLoader.cpp 0x00305CF0
    pub fn unload_vcd(&mut self, key: u32) {
        let l = self.loaded.remove(&key);
        assert!(l.is_some(), "pLoadedVCD not in map");
        if let Some(l) = l {
            l.report_untouched();
        }
    }

    pub fn get(&mut self, key: u32) -> Option<&mut VsLoadedVCD> {
        self.loaded.get_mut(&key)
    }

    /// loaded pack by the path it was loaded from.
    pub fn by_path(&mut self, path: &str) -> Option<&mut VsLoadedVCD> {
        self.loaded.get_mut(&handy_crc::name_hash(path))
    }

    pub fn count(&self) -> usize {
        self.loaded.len()
    }
}
