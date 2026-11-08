# physics agent: assemble crates/velocity_physics/src/c_config.rs from the generated global field list
src = open(r"D:\projects\project poland rewrite\research\phys_dump\gen_global.rs").read()
fields, load = src.split("// ---- load\n")
head = r'''//! src/Physics/CConfig.cpp - the FastCar configuration structures. `CGlobalConfig` (`::GC`, one
//! instance, `dword_74B618`) and `CLocalConfig` (`VehicleEntity::LC`, one per car, 744 bytes) are
//! read from `Vehicles\cfgs\*.cfg` in a fixed parameter order (the original `CConfig.inl`); every
//! field keeps the name of its config parameter.
#![allow(non_snake_case)]

use crate::vehicle::vs_config_file_read::{ConfigReader, Defines, Expression, ReadError, TextFile};

/// `CGlobalConfig` (0x870 bytes). Field order and offsets as in the original.
#[derive(Clone, Debug, Default)]
pub struct CGlobalConfig {
'''
mid = r'''}

/// result of a `Load`: `Stopped` when the file only carried defines and ended with `stop_loading 1`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadResult {
    Stopped,
    Loaded,
}

impl CGlobalConfig {
    /// CConfig.cpp 0x0018F870 (CGlobalConfig::Load)
    pub fn load(&mut self, path: &str, defines: &mut Defines) -> Result<LoadResult, ReadError> {
        let file = TextFile::open(path).ok_or_else(|| ReadError { name: path.into(), line: String::new() })?;
        let mut r = ConfigReader::new(file, defines);
        self.load_from(&mut r)
    }

    pub fn load_from(&mut self, r: &mut ConfigReader) -> Result<LoadResult, ReadError> {
        let mut stop = 0i32;
        r.read_int(&mut stop, "stop_loading")?;
        if stop != 0 {
            return Ok(LoadResult::Stopped);
        }
'''
tail = r'''        Ok(LoadResult::Loaded)
    }
}
'''
out = head + fields + mid + load + tail
open(r"D:\projects\project poland rewrite\research\phys_dump\c_config_global.rs", "w", encoding="utf-8").write(out)
print("ok")
