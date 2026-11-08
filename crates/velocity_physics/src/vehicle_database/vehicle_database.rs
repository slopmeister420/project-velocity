//! src/VehicleDatabase/VehicleDatabase.cpp — CVehicleDatabase: build order, stream helpers, globals,
//! cross-linking of vehicles / bodypart sets / setups, lock state counts, the setup instance.
//!
//! the original keeps every record in an intrusive singly linked list (append at tail) inside two
//! bump buffers; here each list is a `Vec` in the same order and record pointers are indices.

use visos::file::{Vcd, VsFileDirectoryDetails, platform};

use super::vehicle_database_colours::Colour;
use super::vehicle_database_decal_loading::Decal;
use super::vehicle_database_setup_bin_loading::Setup;
use super::vehicle_database_traffic_bin_loading::TrafficSetup;
use super::vehicle_database_upgrade_bin_loading::BodypartSet;
use super::vehicle_database_vehicle_bin_loading::VehicleEntry;
use super::vehicle_database_wheel_bin_loading::{BrakePart, Tyre, Wheel};

#[derive(Debug, thiserror::Error)]
pub enum VehicleDatabaseError {
    #[error("{file}: truncated at {pos} (wanted {want} bytes of {len})")]
    Truncated { file: String, pos: usize, want: usize, len: usize },
    #[error("{file}: version wrong, expecting {expected} got {got}")]
    Version { file: String, expected: f32, got: f32 },
    #[error("{0}: not found")]
    NotFound(String),
    #[error("vcd: {0}")]
    Vcd(#[from] visos::file::VcdError),
    #[error("{file}: {what}")]
    Bad { file: String, what: String },
}

pub type Result<T> = std::result::Result<T, VehicleDatabaseError>;

/// `ePART_COUNT` of the bodypart slots in a set / setup (FRONTBUMPER..MISC).
pub const NUM_BODYPART_TYPES: usize = 14;

/// bodypart type labels (table at 0x6877F4, first 14 of the 22 part type names).
pub const BODYPART_TYPE_NAMES: [&str; NUM_BODYPART_TYPES] = [
    "FRONTBUMPER", "REARBUMPER", "BONNET", "SKIRT", "SPOILER", "ROOFSCOOP", "AIRFILTER", "FLYWHEEL", "NOS", "ENGINE",
    "EXHAUST", "TURBO", "SUSPENSION", "MISC",
];
pub const PART_AIRFILTER: usize = 6;
pub const PART_FLYWHEEL: usize = 7;
pub const PART_NOS: usize = 8;
pub const PART_ENGINE: usize = 9;
pub const PART_EXHAUST: usize = 10;
pub const PART_TURBO: usize = 11;
pub const PART_SUSPENSION: usize = 12;

/// read cursor over a loaded bin; `name` is only for errors.
pub struct BinReader<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
    pub pos: usize,
}

impl<'a> BinReader<'a> {
    pub fn new(name: &'a str, data: &'a [u8]) -> Self {
        Self { name, data, pos: 0 }
    }

    /// VehicleDatabase.cpp 0x00186800
    pub fn read_data(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).filter(|&e| e <= self.data.len()).ok_or_else(|| {
            VehicleDatabaseError::Truncated { file: self.name.to_string(), pos: self.pos, want: n, len: self.data.len() }
        })?;
        let r = &self.data[self.pos..end];
        self.pos = end;
        Ok(r)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_data(4)?.try_into().unwrap()))
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    pub fn read_i16(&mut self) -> Result<i16> {
        Ok(i16::from_le_bytes(self.read_data(2)?.try_into().unwrap()))
    }

    /// the leading `float` version every vehicle database bin starts with.
    pub fn read_version(&mut self, expected: f32) -> Result<f32> {
        let got = self.read_f32()?;
        if got != expected {
            return Err(VehicleDatabaseError::Version { file: self.name.to_string(), expected, got });
        }
        Ok(got)
    }

    /// VehicleDatabase.cpp 0x00182380 — `u32 len` + `len` chars (string buffer copy, nul added).
    pub fn read_string(&mut self) -> Result<Option<String>> {
        let len = self.read_u32()? as usize;
        if len == 0 {
            return Ok(None);
        }
        // the string buffer allocator asserts u32Size<256
        debug_assert!(len + 1 < 256, "u32Size<256");
        let b = self.read_data(len)?;
        Ok(Some(b.iter().map(|&c| c as char).collect()))
    }

    /// VehicleDatabase.cpp 0x00182450 — `u32 len` + `len` utf-16 chars.
    pub fn read_wide_string(&mut self) -> Result<Option<String>> {
        let len = self.read_u32()? as usize;
        if len == 0 {
            return Ok(None);
        }
        debug_assert!(2 * len + 2 < 256, "u32Size<256");
        let b = self.read_data(2 * len)?;
        let w: Vec<u16> = b.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        Ok(Some(String::from_utf16_lossy(&w)))
    }

    pub fn at_end(&self) -> bool {
        self.pos == self.data.len()
    }
}

pub(crate) fn u8_at(b: &[u8], o: usize) -> u8 {
    b[o]
}
pub(crate) fn i16_at(b: &[u8], o: usize) -> i16 {
    i16::from_le_bytes([b[o], b[o + 1]])
}
pub(crate) fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
pub(crate) fn i32_at(b: &[u8], o: usize) -> i32 {
    u32_at(b, o) as i32
}
pub(crate) fn f32_at(b: &[u8], o: usize) -> f32 {
    f32::from_bits(u32_at(b, o))
}
pub(crate) fn f32s_at<const N: usize>(b: &[u8], o: usize) -> [f32; N] {
    std::array::from_fn(|i| f32_at(b, o + 4 * i))
}
pub(crate) fn u32s_at<const N: usize>(b: &[u8], o: usize) -> [u32; N] {
    std::array::from_fn(|i| u32_at(b, o + 4 * i))
}
pub(crate) fn bytes_at<const N: usize>(b: &[u8], o: usize) -> [u8; N] {
    b[o..o + N].try_into().unwrap()
}

/// VsStringAnsi 0x002B46B0 (after the in-place `strupr` at 0x0041E950 every `Find*ByName` does):
/// case-sensitive `HANDY_CRC32(s, strlen, 0)` of the upper-cased name, 0 for "".
/// not `handy_crc::name_hash`, which lower-cases ("LON_BUS01" -> 527812301 here).
pub fn string_hash(name: &str) -> u32 {
    let up: Vec<u8> = name.bytes().map(|c| c.to_ascii_uppercase()).collect();
    if up.is_empty() {
        return 0;
    }
    visos::handy::handy_crc::crc32(&up, 0)
}

/// GLOBALS.BIN — global vehicle camera settings (44 bytes after the version).
#[derive(Debug, Clone, Default)]
pub struct VehicleGlobals {
    pub values: [f32; 11], // +0x00
}

/// reference to one bodypart inside `bodypart_sets[set].parts[part_type][index]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartRef {
    pub set: usize,
    pub part_type: usize,
    pub index: usize,
}

/// `CVehicleDatabase` (`dword_74B4A8`).
#[derive(Debug, Default)]
pub struct CVehicleDatabase {
    pub initialised: bool,              // +0x00
    pub num_vehicles: i16,              // +0x04
    pub num_bodypart_sets: i16,         // +0x06
    pub num_setups: i16,                // +0x08
    pub num_wheels: i16,                // +0x0A
    pub num_decals: i16,                // +0x0C
    pub num_tyres: i16,                 // +0x0E
    pub num_traffic: i16,               // +0x10
    pub num_brake_parts: i16,           // +0x12
    pub total_colours: i16,             // +0x14 m_s16TotalColours
    pub prev_lock_counts: [u32; 4],     // +0x18
    pub lock_counts: [u32; 4],          // +0x28
    pub random_decals: bool,            // +0x38
    pub vehicles: Vec<VehicleEntry>,    // +0x3C
    pub bodypart_sets: Vec<BodypartSet>, // +0x44
    pub setups: Vec<Setup>,             // +0x4C
    pub wheels: Vec<Wheel>,             // +0x54
    pub decals: Vec<Decal>,             // +0x5C
    pub tyres: Vec<Tyre>,               // +0x64
    pub traffic: Vec<TrafficSetup>,     // +0x6C
    pub brake_parts: Vec<BrakePart>,    // +0x74
    pub colours: Vec<Colour>,           // +0x7C
    pub globals: Option<VehicleGlobals>, // +0x80
}

impl CVehicleDatabase {
    /// build the database from the install (`Init`).
    pub fn load() -> Result<Self> {
        let mut db = Self::default();
        db.init()?;
        Ok(db)
    }

    /// VehicleDatabase.cpp 0x00182530
    pub fn load_globals(&mut self, r: &mut BinReader) -> Result<()> {
        r.read_version(1.0)?;
        let b = r.read_data(44)?;
        self.globals = Some(VehicleGlobals { values: f32s_at(b, 0) });
        log::debug!("Global camera settings loaded");
        Ok(())
    }

    /// VehicleDatabase.cpp 0x001841E0 — `<dir>\<name>`: upgrades, then (if present) setup and vehicle.
    pub fn load_vehicle(&mut self, vcd: &Vcd, path: &str) -> Result<()> {
        let name = match path.rfind('\\') {
            Some(i) if i > 0 => &path[i + 1..],
            _ => path,
        };
        if self.load_upgrade_bin(vcd, &format!("{name}upgrades.bin"), path)? {
            self.load_setup_bin(vcd, &format!("{name}setup.bin"), path)?;
            self.load_vehicle_bin(vcd, &format!("{name}vehicle.bin"), path)?;
        }
        Ok(())
    }

    /// VehicleDatabase.cpp 0x001843B0 — every entry of `<dir>\*` without a '.' is a vehicle folder.
    /// FindFirstFile order is the FATX directory order; the host listing is sorted case-insensitively.
    pub fn load_vehicle_directory(&mut self, vcd: &Vcd, dir: &str) -> Result<()> {
        let Ok(rd) = std::fs::read_dir(platform::resolve(dir)) else {
            return Ok(());
        };
        let mut names: Vec<String> = rd.flatten().filter_map(|e| e.file_name().into_string().ok()).collect();
        names.sort_by_key(|n| n.to_ascii_lowercase());
        for n in names {
            if !n.contains('.') {
                self.load_vehicle(vcd, &format!("{dir}\\{n}"))?;
            }
        }
        Ok(())
    }

    /// VehicleDatabase.cpp 0x00184590
    pub fn init(&mut self) -> Result<()> {
        if self.initialised {
            log::warn!("Vehicle database already initialised...");
            return Ok(());
        }
        let vcd = Vcd::open(r"\vehicles\vehdb.vcd")?;
        for dir in [r"\VEHICLES\LA", r"\VEHICLES\LONDON", r"\VEHICLES\TOKYO", r"\VEHICLES\TEAMS", r"\VEHICLES\HIN"] {
            self.load_vehicle_directory(&vcd, dir)?;
        }
        let xml = VsFileDirectoryDetails::open(r"\VEHICLES\XMLBIN.VCD")?;
        let get = |n: &str| xml.load_file(n).map_err(|_| VehicleDatabaseError::NotFound(n.to_string()));
        let b = get("wheels.bin")?;
        self.load_wheels(&mut BinReader::new("wheels.bin", &b), r"\Vehicles\Wheels\")?;
        let b = get("Traffic.BIN")?;
        self.load_traffic(&mut BinReader::new("Traffic.BIN", &b))?;
        let b = get("COLOURS.BIN")?;
        self.load_colours(&mut BinReader::new("COLOURS.BIN", &b))?;
        let b = get("GLOBALS.BIN")?;
        self.load_globals(&mut BinReader::new("GLOBALS.BIN", &b))?;
        let b = get("DECALS.BIN")?;
        self.load_decals(&mut BinReader::new("DECALS.BIN", &b))?;
        // RegisterStrings (0x00183BE0) hands the wide names to the string manager: frontend's job
        self.link_bodypart_sets();
        self.resolve_vehicle_setups();
        self.resolve_setup_bodyparts();
        self.initialised = true;
        self.random_decals = false;
        for state in 0..4 {
            self.update_lock_count(state, 0, false);
        }
        Ok(())
    }

    /// VehicleDatabase.cpp 0x001836F0 — set->m_pVehicle from the set's vehicle hash.
    pub fn link_bodypart_sets(&mut self) {
        for set in &mut self.bodypart_sets {
            match self.vehicles.iter().position(|v| v.hash == set.vehicle_hash) {
                Some(i) => set.vehicle = Some(i),
                None => log::warn!("Unmatched Bodypart Set []"),
            }
        }
    }

    /// VehicleDatabase.cpp 0x00183810 — vehicle setup pointers and colour pointers.
    pub fn resolve_vehicle_setups(&mut self) {
        for vi in 0..self.vehicles.len() {
            let (h0, h1) = (self.vehicles[vi].setup_hash, self.vehicles[vi].setup_hash_2);
            if h0 != 0 {
                self.vehicles[vi].setup = self.find_setup(h0);
            }
            if h1 != 0 {
                self.vehicles[vi].setup_2 = self.find_setup(h1);
            }
            let colours = &self.colours;
            for c in &mut self.vehicles[vi].data.colours {
                c.colour = None;
                let mut found = false;
                for (j, col) in colours.iter().enumerate().take(self.total_colours.max(0) as usize) {
                    // no break: the last match wins
                    if c.hash == col.hash && c.brand == col.brand {
                        c.colour = Some(j);
                        found = true;
                    }
                }
                if !found {
                    log::debug!("No matching colour found in vehicle set-up. Setting to default.");
                    c.colour = Some(0);
                }
            }
        }
    }

    /// VehicleDatabase.cpp 0x00183A20 — pPredefSetup->m_pSetupBodypartsA from the part name hashes.
    pub fn resolve_setup_bodyparts(&mut self) {
        for si in 0..self.setups.len() {
            for t in 0..NUM_BODYPART_TYPES {
                let part_hash = self.setups[si].part_hashes[t];
                if part_hash == 0 {
                    continue;
                }
                let set = self.find_bodypart_set(self.setups[si].bodypart_set_hash);
                debug_assert!(set.is_some(), "pSet");
                let p = self.find_bodypart(set, t, part_hash);
                debug_assert!(p.is_some(), "pPredefSetup->m_pSetupBodypartsA[u8Count]");
                self.setups[si].setup_bodyparts[t] = p;
            }
        }
    }

    /// VehicleDatabase.cpp 0x00184E60 — hidden vehicles become visible at half their unlock value.
    pub fn unhide_vehicles(&mut self, progress: u32) {
        for v in &mut self.vehicles {
            if v.hidden != 0 {
                let half = v.unlock_value >> 1;
                if half != 0 && half <= progress {
                    v.hidden = 0;
                }
            }
        }
    }

    /// VehicleDatabase.cpp 0x00184F30 — count vehicles in lock state `state`, keeping the old count.
    pub fn update_lock_count(&mut self, state: usize, progress: u32, all_unlocked: bool) {
        self.prev_lock_counts[state] = self.lock_counts[state];
        let n = self.vehicles.iter().filter(|v| v.lock_state(progress, all_unlocked) as usize == state).count();
        self.lock_counts[state] = n as u32;
    }

    /// VehicleDatabase.cpp 0x00185010
    pub fn get_lock_count_change(&self, state: usize) -> u32 {
        let d = self.prev_lock_counts[state].wrapping_sub(self.lock_counts[state]) as i32;
        if d >= 0 { d as u32 } else { 0 }
    }

    pub fn part(&self, r: PartRef) -> &super::vehicle_database_upgrade_bin_loading::Bodypart {
        &self.bodypart_sets[r.set].parts[r.part_type][r.index]
    }

    /// VehicleDatabase.cpp 0x001826D0 — may random decal `n` of `kind` be used (every fifth unless +0x38 is 0)
    pub fn random_decal_allowed(&self, kind: i32, n: i32) -> bool {
        !self.random_decals || n.wrapping_add(kind) % 5 == 0
    }

    /// VehicleDatabase.cpp 0x00182720 — `RandomiseSetup(db, setup, setup, vehicle_colours)` (attract mode,
    /// PhysicsManager 0x0019E30E / 0x0019E53A pass 1): a random flag-8 predef setup for the 16 licensed cars,
    /// random decals, random kind-3 / kind-5 brand colours, then a random colour of the vehicle's own list.
    /// Returns the predef setup's cost (eax; edx is the leftover `v41`). `rand` = the game's `VsRandUnit` state.
    pub fn randomise_setup(&self, setup: &mut VehicleSetup, vehicle_colours: bool, rand: &mut visos::handy::crt_rand::CrtRand) -> i32 {
        // CCRCChecked: RX7 RX8 CELICA MR2 240SX 300ZX 350Z ECLIPSE S2000 SKYLINE SUPRA VIPER IMPREZA MINI CIVIC SILVIA
        const LICENSED: [u32; 16] = [
            0xA5E7888D, 0x3558951C, 0x6DD993F1, 0x381870C5, 0x74E5B6D1, 0x1725B37F, 0x94C6FE2D, 0x1F7C15DB,
            0xF86566FE, 0x84642261, 0x668A2AEB, 0x359888F4, 0x0C9CAA09, 0x77CFAD10, 0x5F10834A, 0x466A70AC,
        ];
        // the record address the original stores into the kind-2 flags slots (never 1 / 2: plain texture)
        const DECAL_PTR_STANDIN: u32 = 0x0100_0000;
        // VsRandUnit stored to a float, times the count in double, _ftol (0x003DDFA8)
        let pick = |rand: &mut visos::handy::crt_rand::CrtRand, count: u32| -> u32 {
            let r = rand.rand_unit() as f32;
            ((count as f64) * (r as f64)) as i64 as u32
        };
        let mut cost = 0i32;
        let vehicle_hash = setup.vehicle.map_or(0, |v| self.vehicles[v].hash);
        let saved = setup.colours[0].clone();
        let (saved_14, saved_18) = (saved.unk_14[0], saved.unk_14[1]);
        let num_setups = self.count_setups(vehicle_hash, 8, false);
        if LICENSED.contains(&vehicle_hash) {
            let n = pick(rand, num_setups);
            // the original applies / prices a null setup when the pick runs off the end (r == 1.0)
            if let Some(p) = self.get_nth_setup(vehicle_hash, n, 8, false) {
                self.apply_predef_setup(setup, p);
                cost = cost.wrapping_add(self.get_setup_cost(p));
            }
        }
        let count = self.count_decals(0, 0, 0);
        let n = pick(rand, count);
        if self.random_decal_allowed(0, n as i32) {
            setup.decal_0 = self.get_nth_decal(0, n, 0, 0);
        }
        if self.random_decal_allowed(1, n as i32) {
            setup.decal_1 = self.get_nth_decal(1, n, 0, 0);
        }
        let count = self.count_decals(2, 0, 0);
        let as_flags = |d: Option<usize>| d.map_or(0, |i| DECAL_PTR_STANDIN + 4 * i as u32);
        let n = pick(rand, count);
        if self.random_decal_allowed(2, n as i32) {
            setup.decal_2_4 = self.get_nth_decal(2, n, 0, 0);
        }
        // original bug: setup +0x220 / +0x228 are the flags of the two kind-2 slots, written with a decal pointer
        if self.random_decal_allowed(2, n as i32) {
            setup.decal_2_4_flags = as_flags(self.get_nth_decal(2, n, 0, 0));
        }
        let n = pick(rand, count);
        if self.random_decal_allowed(2, n as i32) {
            setup.decal_2_5 = self.get_nth_decal(2, n, 0, 0);
        }
        if self.random_decal_allowed(2, n as i32) {
            setup.decal_2_5_flags = as_flags(self.get_nth_decal(2, n, 0, 0));
        }
        let count = self.count_decals(3, 0, 0);
        for i in 0..10 {
            let n = pick(rand, count);
            if self.random_decal_allowed(3, n as i32) {
                setup.decals_3[i] = self.get_nth_decal(3, n, 0, 0);
            }
        }
        for slots in [0usize, 1, 2] {
            let count = self.count_decals(3, 0, 0);
            for i in 0..2 {
                let n = pick(rand, count);
                if self.random_decal_allowed(3, n as i32) {
                    let d = self.get_nth_decal(3, n, 0, 0);
                    match slots {
                        0 => setup.decals_3_1[i] = d,
                        1 => setup.decals_3_2[i] = d,
                        _ => setup.decals_3_3[i] = d,
                    }
                }
            }
        }
        for (kind, slot) in [(3u32, 1usize), (5, 3)] {
            let brand = self.get_nth_colour_brand(kind, 0, false);
            let count = self.count_matching_colour_brand(brand, kind);
            let n = pick(rand, count);
            if let Some(c) = self.get_nth_matching_colour_brand(brand, kind, n) {
                setup.colours[slot] = self.colours[c].clone();
            }
        }
        if vehicle_colours {
            if let Some(v) = setup.vehicle {
                let d = &self.vehicles[v].data;
                if d.num_colours != 0 {
                    let r = rand.rand_unit() as f32;
                    let n = ((d.num_colours as f32) * r) as i32;
                    // n == num_colours (r == 1.0) reads past the list in the original
                    if let Some(c) = d.colours.get(n as usize) {
                        self.set_setup_colour(setup, c.colour);
                    }
                }
            }
        } else if setup.colours[0].unk_14[0] != saved_14 && setup.colours[0].unk_14[1] != saved_18 {
            setup.colours[0] = saved;
            setup.colours[0].unk_14 = [saved_14, saved_18];
            self.update_mix_colour(setup); // VehicleDatabaseColours.cpp 0x00187BA0
        }
        cost
    }
}

/// a 64-byte colour slot of the setup instance: copy of a `Colour`.
pub type SetupColour = Colour;

/// `_setup` — a vehicle setup instance (0x22C bytes) as handed to the physics (`VehicleEntity::Create`
/// a6[0]) and the renderer.
#[derive(Debug, Clone)]
pub struct VehicleSetup {
    pub vehicle: Option<usize>,                                  // +0x000
    pub fitted_bodyparts: [Option<PartRef>; NUM_BODYPART_TYPES], // +0x004 m_pFittedBodypartsA
    pub unk_03c: [u32; NUM_BODYPART_TYPES],                      // +0x03C
    pub wheel: Option<usize>,                                    // +0x074
    pub tyre: Option<usize>,                                     // +0x078
    pub brake_parts: [Option<usize>; 2],                         // +0x07C disc, pad
    pub unk_084: u32,                                            // +0x084
    pub colours: [SetupColour; 4],                               // +0x088 body, +0x0C8, +0x108, +0x148
    pub unk_188: u32,                                            // +0x188
    /// automatic gearbox (1 = auto, ctor default; car select 0x0022446D/0x0022447F write it through
    /// dword_768978+0x720 +0x18C); `VehicleEntity::Create` 0x001CABF0 turns 0 into manual up-shifts
    pub unk_18c: u32,                                            // +0x18C
    pub mileage: f32,                                            // +0x190 metres raced
    pub decal_0: Option<usize>,                                  // +0x194
    pub decal_1: Option<usize>,                                  // +0x198
    pub decals_3: [Option<usize>; 10],                           // +0x19C
    pub decals_3_flags: [u32; 10],                               // +0x1C4
    pub decals_3_1: [Option<usize>; 2],                          // +0x1EC
    pub decals_3_1_flags: [u32; 2],                              // +0x1F4
    pub decals_3_2: [Option<usize>; 2],                          // +0x1FC
    pub decals_3_2_flags: [u32; 2],                              // +0x204
    pub decals_3_3: [Option<usize>; 2],                          // +0x20C
    pub decals_3_3_flags: [u32; 2],                              // +0x214
    pub decal_2_4: Option<usize>,                                // +0x21C
    pub decal_2_4_flags: u32,                                    // +0x220
    pub decal_2_5: Option<usize>,                                // +0x224
    pub decal_2_5_flags: u32,                                    // +0x228
}

impl Default for VehicleSetup {
    fn default() -> Self {
        Self::new()
    }
}

impl VehicleSetup {
    /// VehicleDatabase.cpp 0x001850B0 — after a race (EventDatabase 0x00091620) the player's car gets
    /// `(int)raceline length * laps` metres; [`race_mileage`](Self::race_mileage) computes that
    pub fn add_mileage(&mut self, metres: u32) -> u32 {
        self.mileage = (metres as f64 + self.mileage as f64) as f32;
        metres
    }

    /// EventDatabase.cpp 0x00091620: the metres a finished race adds (first race section's raceline length,
    /// truncated, times the race's laps)
    pub fn race_mileage(raceline_length: f32, laps: u8) -> u32 {
        (raceline_length as i32 as u32).wrapping_mul(laps as u32)
    }

    /// VehicleDatabase.cpp 0x001833D0
    pub fn new() -> Self {
        // colour +0x34/+0x38 are set from dword_77FD9C (a shared colour object), left zero here
        let mut colours: [SetupColour; 4] = Default::default();
        colours[1].unk_0c = 3;
        colours[2].unk_0c = 4;
        colours[3].unk_0c = 4;
        Self {
            vehicle: None,
            fitted_bodyparts: [None; NUM_BODYPART_TYPES],
            unk_03c: [0; NUM_BODYPART_TYPES],
            wheel: None,
            tyre: None,
            brake_parts: [None; 2],
            unk_084: 0,
            colours,
            unk_188: 0,
            unk_18c: 1,
            mileage: 0.0,
            decal_0: None,
            decal_1: None,
            decals_3: [None; 10],
            decals_3_flags: [0; 10],
            decals_3_1: [None; 2],
            decals_3_1_flags: [0; 2],
            decals_3_2: [None; 2],
            decals_3_2_flags: [0; 2],
            decals_3_3: [None; 2],
            decals_3_3_flags: [0; 2],
            decal_2_4: None,
            decal_2_4_flags: 0,
            decal_2_5: None,
            decal_2_5_flags: 0,
        }
    }
}
