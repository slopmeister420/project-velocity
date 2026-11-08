//! Opponent database: `OPPONENT.BIN` (src/OpponentDatabase/OpponentDatabase.cpp).
//!
//! The game reads it from `\VEHICLES\XMLBIN.VCD`; a loose copy also ships as `AI/OPPONENT.BIN`.
//! Layout: `f32 version (1.5) | i16 count | count x entry`, where an entry is the raw 136-byte
//! `OpponentEntry` image, then `numSetups x 52-byte VehicleSetup`, then for each of the 12
//! string lists `count x (u32 len, len bytes)`.

use crate::crc::string_crc;
use velocity_physics::vehicle_database::vehicle_database::{CVehicleDatabase, VehicleSetup};

pub const OPPONENT_DATABASE_VERSION: f32 = 1.5;

#[derive(Debug, thiserror::Error)]
pub enum OpponentDbError {
    #[error("Opponent database version wrong: expecting 1.50 got {0:.2}")]
    Version(f32),
    #[error("truncated opponent database")]
    Truncated,
}

/// one car an opponent can drive (`m_pVehicleSetupsA[n]`, 52 bytes).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpponentVehicleSetup {
    pub vehicle: u32,         // +0x00 vehicle crc
    pub body_setup: u32,      // +0x04
    pub engine_setup: u32,    // +0x08
    pub runtime_setup: u32,   // +0x0C VehicleSetup* at runtime
    pub paint_colour: [u32; 2], // +0x10 colour group / colour crcs, "NONE" = keep default
    pub neon_colour: [u32; 2],  // +0x18
    pub wheels: u32,          // +0x20
    pub alloy_colour: [u32; 2], // +0x24
    pub decals: [u32; 2],     // +0x2C
}

/// one opponent (136 bytes in the file, linked list at runtime via `+0x84`).
#[derive(Debug, Clone, Default)]
pub struct OpponentEntry {
    pub name: String,         // +0x00 char[16]
    pub short_name: String,   // +0x10 char[16]
    pub name_crc: u32,        // +0x20
    pub group: u32,           // +0x24 filter key of GetOpponentCount/GetOpponent
    pub flags: u32,           // +0x28 filter mask
    pub unk_2c: u32,          // +0x2C
    pub unk_30: u32,          // +0x30
    pub unk_35: [u8; 3],      // +0x35
    pub base_aggression: u32, // +0x48 eAIA level (0 low, 1 med, 2 high)
    pub skill: u32,           // +0x4C
    pub level_50: u32,        // +0x50 third eAIA level (VsAIDriver.cpp 0x00015370)
    pub vehicle_setups: Vec<OpponentVehicleSetup>, // +0x34 count, +0x44 pointer
    pub string_lists: [Vec<String>; 12],           // +0x38 counts, +0x54 pointers
}

#[derive(Debug, Clone, Default)]
pub struct OpponentDatabase {
    pub opponents: Vec<OpponentEntry>,
}

struct Rd<'a> {
    d: &'a [u8],
    p: usize,
}

impl Rd<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], OpponentDbError> {
        let s = self.d.get(self.p..self.p + n).ok_or(OpponentDbError::Truncated)?;
        self.p += n;
        Ok(s)
    }
}

fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

fn cstr(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).into_owned()
}

impl OpponentDatabase {
    /// reads `OPPONENT.BIN` out of `\VEHICLES\XMLBIN.VCD` (or its loose folder) in the game dir.
    /// OpponentDatabase.cpp 0x000E78E0
    pub fn load() -> Option<Self> {
        let dir = visos::file::vs_file_directory::VsFileDirectoryDetails::open("\\VEHICLES\\XMLBIN.VCD").ok()?;
        let data = dir.load_file("OPPONENT.BIN").ok()?;
        Self::read_from_binary(&data).ok()
    }

    /// OpponentDatabase.cpp 0x000E7EC0
    pub fn read_from_binary(data: &[u8]) -> Result<Self, OpponentDbError> {
        let mut r = Rd { d: data, p: 0 };
        let version = f32::from_le_bytes(r.take(4)?.try_into().unwrap());
        if version != OPPONENT_DATABASE_VERSION {
            log::error!(
                "ERROR: Opponent Database data exported with out-of-date exporter.  Synch to Tools\\Bin and re-export"
            );
            return Err(OpponentDbError::Version(version));
        }
        let count = i16::from_le_bytes(r.take(2)?.try_into().unwrap());
        let mut db = Self::default();
        for _ in 0..count.max(0) {
            db.opponents.push(Self::read_entry(&mut r)?);
        }
        log::info!("Opponents read from Binary");
        Ok(db)
    }

    /// OpponentDatabase.cpp 0x000E7DA0
    fn read_entry(r: &mut Rd) -> Result<OpponentEntry, OpponentDbError> {
        let e = r.take(136)?.to_vec();
        let num_setups = e[52] as usize;
        let mut ent = OpponentEntry {
            name: cstr(&e[0..16]),
            short_name: cstr(&e[16..32]),
            name_crc: u32_at(&e, 0x20),
            group: u32_at(&e, 0x24),
            flags: u32_at(&e, 0x28),
            unk_2c: u32_at(&e, 0x2C),
            unk_30: u32_at(&e, 0x30),
            unk_35: [e[0x35], e[0x36], e[0x37]],
            base_aggression: u32_at(&e, 0x48),
            skill: u32_at(&e, 0x4C),
            level_50: u32_at(&e, 0x50),
            ..Default::default()
        };
        let setups = r.take(52 * num_setups)?;
        for s in setups.chunks_exact(52) {
            let w = |i: usize| u32_at(s, i * 4);
            ent.vehicle_setups.push(OpponentVehicleSetup {
                vehicle: w(0),
                body_setup: w(1),
                engine_setup: w(2),
                runtime_setup: w(3),
                paint_colour: [w(4), w(5)],
                neon_colour: [w(6), w(7)],
                wheels: w(8),
                alloy_colour: [w(9), w(10)],
                decals: [w(11), w(12)],
            });
        }
        for (i, list) in ent.string_lists.iter_mut().enumerate() {
            for _ in 0..e[56 + i] {
                list.push(read_string(r)?);
            }
        }
        Ok(ent)
    }

    /// OpponentDatabase.cpp 0x000E7AA0
    pub fn get_opponent_by_crc(&self, crc: u32) -> Option<&OpponentEntry> {
        self.opponents.iter().find(|o| o.name_crc == crc)
    }

    /// OpponentDatabase.cpp 0x000E7D60
    pub fn get_opponent_by_name(&self, name: &str) -> Option<&OpponentEntry> {
        self.get_opponent_by_crc(string_crc(name))
    }

    /// OpponentDatabase.cpp 0x000E7B50
    pub fn get_opponent_count(&self, group: u32, flags: u32) -> i16 {
        let mut n: i16 = 0;
        for o in &self.opponents {
            if (group == 0 || group == o.group) && (flags == 0 || (flags & o.flags) != 0) {
                n = n.wrapping_add(1);
            }
        }
        n
    }

    /// OpponentDatabase.cpp 0x000E7C20
    pub fn get_opponent(&self, index: i16, group: u32, flags: u32) -> Option<&OpponentEntry> {
        let mut n: i16 = 0;
        for o in &self.opponents {
            if (group == 0 || group == o.group) && (flags == 0 || (flags & o.flags) != 0) {
                if n == index {
                    return Some(o);
                }
                n = n.wrapping_add(1);
            }
        }
        None
    }
}

/// u32 length + bytes (no terminator in the file).
/// TriggerUpdate.cpp 0x00182380
fn read_string(r: &mut Rd) -> Result<String, OpponentDbError> {
    let n = u32::from_le_bytes(r.take(4)?.try_into().unwrap()) as usize;
    Ok(String::from_utf8_lossy(r.take(n)?).into_owned())
}

/// `CCRCChecked("NONE", 0x68413267)`: the paint colour crc that keeps the default
const NONE_CRC: u32 = 1749103207;

impl OpponentDatabase {
    /// the `index`th car of opponent `crc` as a vehicle setup, with the opponent's short name: body set-up,
    /// engine set-up (240SX defaults when missing), paint unless "NONE", neon and alloy colours, wheels
    /// (fallback: nth wheel in "category" = the vehicle hash, as the original passes it, then the first
    /// wheel), two decals.
    /// OpponentDatabase.cpp 0x000E72C0 (one setup)
    pub fn build_vehicle_setup(&self, db: &CVehicleDatabase, crc: u32, index: usize) -> Option<(VehicleSetup, String)> {
        let o = self.get_opponent_by_crc(crc)?;
        let v = o.vehicle_setups.get(index)?;
        let mut s = VehicleSetup::new();
        match db.find_setup(v.body_setup) {
            Some(p) => db.apply_predef_setup(&mut s, p),
            None => {
                log::warn!("WARNING: Opponent with no matching vehicle Body set-up [{}], using a default", o.name);
                db.apply_predef_setup(&mut s, db.find_setup_by_name("240SXBODYKITDEFAULT")?);
            }
        }
        match db.find_setup(v.engine_setup) {
            Some(p) => db.apply_predef_setup(&mut s, p),
            None => {
                log::warn!("WARNING: Opponent with no matching vehicle Engine set-up [{}], using a default", o.name);
                db.apply_predef_setup(&mut s, db.find_setup_by_name("240SXENGINEDEFAULT")?);
            }
        }
        if v.paint_colour[1] != NONE_CRC {
            match db.find_colour(v.paint_colour[0], v.paint_colour[1]) {
                Some(c) => s.colours[0] = db.colours[c].clone(),
                None => log::warn!("WARNING: Opponent with no matching paint colour info [{}]", o.name),
            }
        }
        for (slot, pair) in [(1, v.neon_colour), (2, v.alloy_colour)] {
            if pair[0] != 0 && pair[1] != 0 {
                match db.find_colour(pair[0], pair[1]) {
                    Some(c) => s.colours[slot] = db.colours[c].clone(),
                    None => log::warn!("WARNING: Opponent with no matching colour info [{}]", o.name),
                }
            }
        }
        s.wheel = if v.wheels != 0 { db.find_wheel(v.wheels) } else { None };
        if s.wheel.is_none() {
            log::warn!("Warning: Unable to find wheels for opponent [{}]. Using a default.", o.name);
            let hash = s.vehicle.and_then(|i| db.vehicles.get(i)).map(|e| e.hash).unwrap_or(0);
            s.wheel = db.get_nth_wheel_in_category(hash, 1).or((!db.wheels.is_empty()).then_some(0));
        }
        s.decal_0 = if v.decals[0] != 0 { db.find_decal(v.decals[0]) } else { None };
        s.decal_1 = if v.decals[1] != 0 { db.find_decal(v.decals[1]) } else { None };
        Some((s, o.short_name.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_game_opponent_bin() {
        let Some(dir) = crate::game_dir() else { return };
        let data = std::fs::read(dir.join("AI").join("OPPONENT.BIN")).unwrap();
        let db = OpponentDatabase::read_from_binary(&data).unwrap();
        assert_eq!(db.opponents.len(), 31);
        for o in &db.opponents {
            assert_eq!(o.name_crc, string_crc(&o.name), "{}", o.name);
        }
        assert!(db.get_opponent_by_name("TO_COP2").is_some());
        assert_eq!(db.get_opponent_count(0, 0), 31);
        let vcd = OpponentDatabase::load().expect("OPPONENT.BIN in XMLBIN.VCD");
        assert!(!vcd.opponents.is_empty());
        for o in &vcd.opponents {
            assert_eq!(o.name_crc, string_crc(&o.name), "{}", o.name);
        }
    }
}
