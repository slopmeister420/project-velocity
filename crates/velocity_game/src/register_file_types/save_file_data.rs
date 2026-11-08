//! src\RegisterFileTypes\SaveFileData.cpp (code 0x001044D0..0x00106B30; the idb labels 0x001044D0..0x00106300
//! "ParticleManager") — the career profile: the 18656-byte "Primary" save block (`dword_701428`), filled from
//! `g_FEGameVariables`, the vehicle and event databases before a save and spread back over them after a load.
//! The Xbox save containers (`SAVE_Flow_Load/Save/Delete_M`, one container per profile name) are replaced by one
//! file per profile under the user profile (`%APPDATA%\Velocity\Saves\<name>\Primary`), else under the exe dir;
//! never the game dir.

use std::path::{Path, PathBuf};

use velocity_physics::vehicle_database::vehicle_database::{CVehicleDatabase, VehicleSetup, NUM_BODYPART_TYPES};

use crate::event_database::event_database::{EVENT_FLAG_COMPLETED, EVENT_FLAG_LOCKED};
use crate::event_database::event_database_track_loading::{CEventDatabase, RACE_FLAG_UNLOCKED};
use crate::game_mode::game_player::{CFEGameVariables, GARAGE_SLOTS};

/// sizeof(`dword_701428`) — the "Primary" save buffer
pub const PRIMARY_SIZE: usize = 18656;
/// `CCRCChecked("Version_25")` = 0xA248A95E, the first word of a valid save
pub const VERSION_25: u32 = 0xA248_A95E;
/// limits of the save's tables
pub const MAX_WHEELS: usize = 30;
pub const MAX_BODYPARTS: usize = 500;
pub const MAX_RACES: usize = 300;
pub const MAX_EVENTS: usize = 150;
pub const NUM_ALLOYS: usize = 10;
/// `byte_6F915C` options block copied whole (2688 bytes): `OptionsBlock` (+0x00..+0x28), then the audio playlists
/// (+0x28 3 x 16 list headers, +0x58 150 x 12 tracks, +0x760 100 x 8 list type / shuffle; 0x00105E10 / 0x00104F30)
pub const OPTIONS_SIZE: usize = 2688;
pub const OPT_WIDESCREEN: usize = 0x01; // byte_6F915D
pub const OPT_SPEECH: usize = 0x02; // byte_6F915E
pub const OPT_PLAYER_NAMES: usize = 0x03; // byte_6F915F
pub const OPT_CONTROLLER_CONFIG: usize = 0x25; // byte_6F9181 (pad 0's Config<n+1>.ini)
/// end of the option globals, start of the playlists
pub const OPT_PLAYLISTS: usize = 0x28;

/// the option globals at the head of the options block (`byte_6F915C..byte_6F9182`), as the save copies them.
/// Defaults = GamePlayer.cpp SetInitialVariables 0x000C1930 (which leaves `controller_config` alone).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OptionsBlock {
    pub unk_00: u8,             // +0x00 byte_6F915C (only written: 0 at init, save copies)
    pub widescreen: bool,       // +0x01 byte_6F915D
    pub speech: bool,           // +0x02 byte_6F915E (trash talk / TFT)
    pub player_names: bool,     // +0x03 byte_6F915F
    pub soundtrack: i32,        // +0x04 dword_6F9160 (-1 random)
    /// +0x08 master, +0x0C music, +0x10 engine, +0x14 tyres, +0x18 speech, +0x1C sfx (address order)
    pub volumes: [f32; 6],
    pub audio_mode: i32,        // +0x20 dword_6F917C
    pub time_format: u8,        // +0x24 byte_6F9180 (COSDManager::FormatTime flag, "--:--" vs "--:--.--")
    pub controller_config: u8,  // +0x25 byte_6F9181 (pad 0's UserInput +0x14, written just before a save)
    pub rumble: bool,           // +0x26 byte_6F9182
    pub unk_27: u8,             // +0x27 padding
}

impl Default for OptionsBlock {
    /// GamePlayer.cpp 0x000C1930 (speech volume is written 0.2 then 0.8)
    fn default() -> Self {
        Self {
            unk_00: 0,
            widescreen: false,
            speech: true,
            player_names: false,
            soundtrack: -1,
            volumes: [1.0, 0.2, 0.8, 0.8, 0.8, 0.8],
            audio_mode: 1,
            time_format: 0,
            controller_config: 0,
            rumble: true,
            unk_27: 0,
        }
    }
}

impl OptionsBlock {
    /// the first 0x28 bytes of the options block
    pub fn read(b: &[u8]) -> Self {
        let d = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        let mut volumes = [0.0; 6];
        for (i, v) in volumes.iter_mut().enumerate() {
            *v = f32::from_bits(d(0x08 + 4 * i));
        }
        Self {
            unk_00: b[0x00],
            widescreen: b[0x01] != 0,
            speech: b[0x02] != 0,
            player_names: b[0x03] != 0,
            soundtrack: d(0x04) as i32,
            volumes,
            audio_mode: d(0x20) as i32,
            time_format: b[0x24],
            controller_config: b[0x25],
            rumble: b[0x26] != 0,
            unk_27: b[0x27],
        }
    }

    /// writes the first 0x28 bytes (the playlists after them are left as they are)
    pub fn write(&self, b: &mut [u8]) {
        b[0x00] = self.unk_00;
        b[0x01] = self.widescreen as u8;
        b[0x02] = self.speech as u8;
        b[0x03] = self.player_names as u8;
        b[0x04..0x08].copy_from_slice(&(self.soundtrack as u32).to_le_bytes());
        for (i, v) in self.volumes.iter().enumerate() {
            b[0x08 + 4 * i..0x0C + 4 * i].copy_from_slice(&v.to_bits().to_le_bytes());
        }
        b[0x20..0x24].copy_from_slice(&(self.audio_mode as u32).to_le_bytes());
        b[0x24] = self.time_format;
        b[0x25] = self.controller_config;
        b[0x26] = self.rumble as u8;
        b[0x27] = self.unk_27;
    }
}

/// one saved bodypart (`dword_701B58`, 12 bytes)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SavedBodypart {
    pub set: u32,
    pub part: u32,
    pub kind: u8,
}

/// one saved event (`dword_704BAC`, 8 bytes)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SavedEvent {
    pub crc: u32,
    pub completed: bool, // +4
    pub locked: bool,    // +5
}

/// the "Primary" block, field for field (offsets from `dword_701428`); garage arrays are per slot.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimarySave {
    pub version: u32,                           // +0x0000
    pub num_vehicles: u32,                      // +0x0004
    pub vehicle: [u32; 6],                      // +0x0008
    pub mileage: [u32; 6],                      // +0x0020
    pub wheel: [u32; 6],                        // +0x0038
    pub tyre: [u32; 6],                         // +0x0050
    pub brakes: [[u32; 2]; 6],                  // +0x0068
    pub unk_084: [u32; 6],                      // +0x0098 setup +0x84
    pub colour0: [[u32; 2]; 6],                 // +0x00B0 brand, hash
    pub colour0_18: [u32; 6],                   // +0x00E0 setup +0xA0 (used when the colour is gone)
    pub colour0_14: [u32; 6],                   // +0x00F8 setup +0x9C
    pub colour1: [[u32; 2]; 6],                 // +0x0110
    pub colour2: [[u32; 2]; 6],                 // +0x0140
    pub colour3: [[u32; 2]; 6],                 // +0x0170
    pub has_18c: [u8; 8],                       // +0x01A0 setup +0x18C != 0 (6 used)
    pub unk_188: [u32; 6],                      // +0x01A8
    pub bodypart_set: [[u32; 14]; 6],           // +0x01C0
    pub bodypart: [[u32; 14]; 6],               // +0x0310
    pub decals: [[u32; 20]; 6],                 // +0x0460
    pub wheels: Vec<[u32; 2]>,                  // +0x0640 owned wheels: hash, category (30)
    pub bodyparts: Vec<SavedBodypart>,          // +0x0730 bought bodyparts (500)
    pub vars: [u32; 21],                        // +0x1EA0 copy of `dword_6F719C`..
    pub options: Vec<u8>,                       // +0x1EF4 copy of `byte_6F915C` (2688)
    pub races: Vec<[u32; 3]>,                   // +0x2974 race id crc, best record, best time bits (300)
    pub events: Vec<SavedEvent>,                // +0x3784 (150)
    pub num_completed: u32,                     // +0x3C34
    pub alloys: Vec<u8>,                        // +0x3C38 10 x 324 custom alloys (`unk_6F83AC`)
}

/// indices into `PrimarySave::vars` (`dword_6F719C` + 4 i)
pub mod var {
    pub const CASH: usize = 0;
    pub const PRESTIGE: usize = 1;
    pub const NUM_VEHICLES: usize = 2;
    pub const GARAGE: usize = 3; // 6 crcs
    pub const VEHICLE: usize = 9;
    pub const SLOT: usize = 10;
    pub const REWARD: usize = 11;
    pub const UNKNOWN_25C: usize = 12;
    pub const TEAM: usize = 13;
    pub const REST: usize = 14; // 7 words
}

impl Default for PrimarySave {
    fn default() -> Self {
        Self {
            version: 0,
            num_vehicles: 0,
            vehicle: [0; 6],
            mileage: [0; 6],
            wheel: [0; 6],
            tyre: [0; 6],
            brakes: [[0; 2]; 6],
            unk_084: [0; 6],
            colour0: [[0; 2]; 6],
            colour0_18: [0; 6],
            colour0_14: [0; 6],
            colour1: [[0; 2]; 6],
            colour2: [[0; 2]; 6],
            colour3: [[0; 2]; 6],
            has_18c: [0; 8],
            unk_188: [0; 6],
            bodypart_set: [[0; 14]; 6],
            bodypart: [[0; 14]; 6],
            decals: [[0; 20]; 6],
            wheels: vec![[0; 2]; MAX_WHEELS],
            bodyparts: vec![SavedBodypart::default(); MAX_BODYPARTS],
            vars: [0; 21],
            options: vec![0; OPTIONS_SIZE],
            races: vec![[0; 3]; MAX_RACES],
            events: vec![SavedEvent::default(); MAX_EVENTS],
            num_completed: 0,
            alloys: vec![0; NUM_ALLOYS * 324],
        }
    }
}

/// one pass over the buffer in field order, reading or writing
struct Rw<'a> {
    buf: &'a mut [u8],
    pos: usize,
    write: bool,
}

impl Rw<'_> {
    fn u32(&mut self, v: &mut u32) {
        let r = &mut self.buf[self.pos..self.pos + 4];
        if self.write {
            r.copy_from_slice(&v.to_le_bytes());
        } else {
            *v = u32::from_le_bytes([r[0], r[1], r[2], r[3]]);
        }
        self.pos += 4;
    }
    fn u32s(&mut self, v: &mut [u32]) {
        v.iter_mut().for_each(|x| self.u32(x));
    }
    fn bytes(&mut self, v: &mut [u8]) {
        let r = &mut self.buf[self.pos..self.pos + v.len()];
        if self.write {
            r.copy_from_slice(v);
        } else {
            v.copy_from_slice(r);
        }
        self.pos += v.len();
    }
    fn pairs(&mut self, v: &mut [[u32; 2]]) {
        v.iter_mut().for_each(|p| self.u32s(p));
    }
}

impl PrimarySave {
    fn io(&mut self, rw: &mut Rw) {
        rw.u32(&mut self.version);
        rw.u32(&mut self.num_vehicles);
        rw.u32s(&mut self.vehicle);
        rw.u32s(&mut self.mileage);
        rw.u32s(&mut self.wheel);
        rw.u32s(&mut self.tyre);
        rw.pairs(&mut self.brakes);
        rw.u32s(&mut self.unk_084);
        rw.pairs(&mut self.colour0);
        rw.u32s(&mut self.colour0_18);
        rw.u32s(&mut self.colour0_14);
        rw.pairs(&mut self.colour1);
        rw.pairs(&mut self.colour2);
        rw.pairs(&mut self.colour3);
        rw.bytes(&mut self.has_18c);
        rw.u32s(&mut self.unk_188);
        self.bodypart_set.iter_mut().for_each(|a| rw.u32s(a));
        self.bodypart.iter_mut().for_each(|a| rw.u32s(a));
        self.decals.iter_mut().for_each(|a| rw.u32s(a));
        self.wheels.resize(MAX_WHEELS, [0; 2]);
        rw.pairs(&mut self.wheels);
        self.bodyparts.resize(MAX_BODYPARTS, SavedBodypart::default());
        for b in &mut self.bodyparts {
            rw.u32(&mut b.set);
            rw.u32(&mut b.part);
            let mut k = [b.kind, 0, 0, 0];
            rw.bytes(&mut k);
            b.kind = k[0];
        }
        rw.u32s(&mut self.vars);
        self.options.resize(OPTIONS_SIZE, 0);
        rw.bytes(&mut self.options);
        self.races.resize(MAX_RACES, [0; 3]);
        self.races.iter_mut().for_each(|r| rw.u32s(r));
        self.events.resize(MAX_EVENTS, SavedEvent::default());
        for e in &mut self.events {
            rw.u32(&mut e.crc);
            let mut f = [e.completed as u8, e.locked as u8, 0, 0];
            rw.bytes(&mut f);
            e.completed = f[0] != 0;
            e.locked = f[1] != 0;
        }
        rw.u32(&mut self.num_completed);
        self.alloys.resize(NUM_ALLOYS * 324, 0);
        rw.bytes(&mut self.alloys);
        debug_assert_eq!(rw.pos, PRIMARY_SIZE);
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![0u8; PRIMARY_SIZE];
        let mut c = self.clone();
        c.io(&mut Rw { buf: &mut buf, pos: 0, write: true });
        buf
    }

    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != PRIMARY_SIZE {
            return None;
        }
        let mut buf = b.to_vec();
        let mut s = Self::default();
        s.io(&mut Rw { buf: &mut buf, pos: 0, write: false });
        Some(s)
    }

    /// SaveFileData 0x00106040 / 0x001061D0 body — gather everything into the block (the options words game flow
    /// owns are written over the previous block's bytes; the audio playlists 0x00105E10 and alloys stay as they
    /// were, their owners are the audio and front end crates).
    /// `options` = the option globals (`controller_config` = pad 0's config, `byte_6F9181 = UserInput 0x1E7140(0)`);
    /// speech / names are taken from `vars`, where game flow keeps them.
    pub fn gather(&mut self, vars: &CFEGameVariables, events: &CEventDatabase, vdb: &CVehicleDatabase, options: &OptionsBlock) {
        self.version = VERSION_25;
        self.vars = vars_block(vars);
        self.options.resize(OPTIONS_SIZE, 0);
        let mut o = *options;
        o.speech = vars.speech;
        o.player_names = vars.player_names;
        o.write(&mut self.options);
        self.store_garage(vars, vdb);
        self.store_wheels(vdb);
        self.store_bodyparts(vdb);
        self.store_races(events);
        self.store_events(events);
    }

    /// SaveFileData 0x00105370 — the garage cars' setups by hash
    fn store_garage(&mut self, vars: &CFEGameVariables, vdb: &CVehicleDatabase) {
        let n = vars.garage.len().min(GARAGE_SLOTS);
        self.num_vehicles = n as u32;
        for i in 0..n {
            let Some(s) = vars.garage_setup(i) else { continue };
            self.vehicle[i] = s.vehicle.and_then(|v| vdb.vehicles.get(v)).map_or(0, |v| v.hash);
            // VehicleDatabase 0x00185090: ftol of the float mileage
            self.mileage[i] = s.mileage as i64 as u32;
            self.wheel[i] = s.wheel.and_then(|w| vdb.wheels.get(w)).map_or(0, |w| w.hash);
            self.tyre[i] = s.tyre.and_then(|t| vdb.tyres.get(t)).map_or(0, |t| t.hash);
            for j in 0..2 {
                self.brakes[i][j] = s.brake_parts[j].and_then(|b| vdb.brake_parts.get(b)).map_or(0, |b| b.hash);
            }
            self.unk_084[i] = s.unk_084;
            self.colour0[i] = [s.colours[0].brand, s.colours[0].hash];
            self.colour0_18[i] = s.colours[0].unk_14[1];
            self.colour0_14[i] = s.colours[0].unk_14[0];
            self.colour1[i] = [s.colours[1].brand, s.colours[1].hash];
            self.colour2[i] = [s.colours[2].brand, s.colours[2].hash];
            self.colour3[i] = [s.colours[3].brand, s.colours[3].hash];
            self.has_18c[i] = (s.unk_18c != 0) as u8;
            self.unk_188[i] = s.unk_188;
            for k in 0..NUM_BODYPART_TYPES {
                let (set, part) = match s.fitted_bodyparts[k] {
                    Some(p) => {
                        let set = &vdb.bodypart_sets[p.set];
                        (set.hash, set.parts[p.part_type][p.index].hash)
                    }
                    None => (0, 0),
                };
                self.bodypart_set[i][k] = set;
                self.bodypart[i][k] = part;
            }
            self.decals[i] = [0; 20];
            for (m, d) in decal_slots(s).into_iter().enumerate() {
                if let Some(d) = d.and_then(|d| vdb.decals.get(d)) {
                    self.decals[i][m] = d.hash;
                }
            }
        }
    }

    /// SaveFileData 0x00105890 — wheels fitted (`+0x2A == 1`, "owned"), with their category
    fn store_wheels(&mut self, vdb: &CVehicleDatabase) {
        self.wheels = vdb.wheels.iter().filter(|w| w.fitted == 1).map(|w| [w.hash, w.category]).take(MAX_WHEELS).collect();
        self.wheels.resize(MAX_WHEELS, [0; 2]);
    }

    /// SaveFileData 0x00105960 — bodyparts with flag 8 (SaveFileData 0x00106460)
    fn store_bodyparts(&mut self, vdb: &CVehicleDatabase) {
        let mut out = Vec::new();
        for set in &vdb.bodypart_sets {
            for i in 0..NUM_BODYPART_TYPES {
                for j in 0..set.part_counts[i] as usize {
                    if let Some(p) = set.parts[i].get(j).filter(|p| p.flags & 8 == 8) {
                        out.push(SavedBodypart { set: set.hash, part: p.hash, kind: i as u8 });
                    }
                }
            }
        }
        out.truncate(MAX_BODYPARTS);
        out.resize(MAX_BODYPARTS, SavedBodypart::default());
        self.bodyparts = out;
    }

    /// SaveFileData 0x00105B10 — race records of every race on the tracks with an unlocked race
    /// (EventDatabaseTrackLoading 0x00095110 / 0x00094D20 filter 2), per type list
    fn store_races(&mut self, db: &CEventDatabase) {
        let mut out = Vec::new();
        for t in &db.tracks {
            let open = t.races.iter().flatten().filter(|&&r| db.races[r].flags & RACE_FLAG_UNLOCKED != 0).count();
            if open == 0 {
                continue;
            }
            for list in &t.races {
                for &r in list {
                    let r = &db.races[r];
                    out.push([r.id_crc, r.best_record, r.unknown_28.to_bits()]);
                }
            }
        }
        // the original writes past the 300 entries if there are more (none in the shipped data)
        out.truncate(MAX_RACES);
        out.resize(MAX_RACES, [0; 3]);
        self.races = out;
    }

    /// SaveFileData 0x00105CB0 — every event's completed / locked flags in list order
    fn store_events(&mut self, db: &CEventDatabase) {
        let mut done = 0;
        let mut out: Vec<SavedEvent> = db
            .events
            .iter()
            .map(|e| {
                let c = e.flags & EVENT_FLAG_COMPLETED != 0;
                done += c as u32;
                SavedEvent { crc: e.crc, completed: c, locked: e.flags & EVENT_FLAG_LOCKED != 0 }
            })
            .take(MAX_EVENTS)
            .collect();
        out.resize(MAX_EVENTS, SavedEvent::default());
        self.events = out;
        self.num_completed = done;
    }

    /// SaveFileData 0x001050C0 — a valid block (version check) spread over the game: the vars block, garage
    /// (0x001044D0), wheels (0x00104C40), bodyparts (0x00104CC0), race records (0x00104D80), events (0x00104E30).
    /// returns the current garage car's setup (`dword_6F724C`) and pad 0's controller config, None when the
    /// version does not match.
    pub fn apply(&self, vars: &mut CFEGameVariables, events: &mut CEventDatabase, vdb: &mut CVehicleDatabase) -> Option<Applied> {
        if self.version != VERSION_25 {
            return None;
        }
        apply_vars_block(vars, &self.vars);
        vars.speech = self.options[OPT_SPEECH] != 0;
        vars.player_names = self.options[OPT_PLAYER_NAMES] != 0;
        let current = self.load_garage(vars, vdb);
        for &[hash, _] in &self.wheels {
            if hash != 0 {
                if let Some(w) = vdb.find_wheel(hash) {
                    vdb.wheels[w].fitted = 1;
                }
            }
        }
        for b in &self.bodyparts {
            if let Some(set) = vdb.find_bodypart_set(b.set) {
                if let Some(p) = vdb.find_bodypart(Some(set), b.kind as usize, b.part) {
                    // SaveFileData 0x00105230: flag 8, +0x1C = 0
                    let part = &mut vdb.bodypart_sets[p.set].parts[p.part_type][p.index];
                    part.flags |= 8;
                    part.unk_1c = 0;
                }
            }
        }
        for &[id, record, time] in &self.races {
            if id != 0 {
                // EventDatabaseTrackLoading 0x00095650: by +0x08 id crc
                if let Some(r) = events.races.iter_mut().find(|r| r.id_crc == id) {
                    r.id_crc = id;
                    r.best_record = record;
                    r.unknown_28 = f32::from_bits(time);
                }
            }
        }
        for e in &self.events {
            if e.crc != 0 {
                if let Some(i) = events.find_event(e.crc) {
                    let f = &mut events.events[i].flags;
                    if e.locked { *f |= EVENT_FLAG_LOCKED } else { *f &= !EVENT_FLAG_LOCKED }
                    if e.completed { *f |= EVENT_FLAG_COMPLETED } else { *f &= !EVENT_FLAG_COMPLETED }
                }
            }
        }
        // CompartmentManager__8ECC0(db, 2, 0): events with the completed flag
        vars.completed_any = events.events.iter().any(|e| e.flags & EVENT_FLAG_COMPLETED != 0);
        // 0x00104F30 audio playlists, VsCommonMain 0x0016E790 team lock refresh: the audio / ai crates' (not here)
        vdb.unhide_vehicles(vars.prestige as u32);
        let options = OptionsBlock::read(&self.options);
        Some(Applied { current_setup: current, controller_config: options.controller_config, options })
    }

    /// SaveFileData 0x001044D0 — rebuild the garage from the saved hashes
    fn load_garage(&self, vars: &mut CFEGameVariables, vdb: &mut CVehicleDatabase) -> Option<VehicleSetup> {
        vars.garage.clear();
        let mut current = None;
        for i in 0..(self.num_vehicles as usize).min(GARAGE_SLOTS) {
            let mut s = VehicleSetup::new();
            if !vdb.setup_default_vehicle(&mut s, self.vehicle[i]) {
                continue;
            }
            s.mileage = self.mileage[i] as f32;
            s.wheel = vdb.find_wheel(self.wheel[i]);
            s.tyre = vdb.find_tyre(self.tyre[i]);
            for j in 0..2 {
                s.brake_parts[j] = vdb.find_brake_part(self.brakes[i][j]);
            }
            s.unk_084 = self.unk_084[i];
            match vdb.find_colour(self.colour0[i][0], self.colour0[i][1]) {
                Some(c) => s.colours[0] = vdb.colours[c].clone(),
                None => {
                    s.colours[0].hash = 0;
                    s.colours[0].brand = 0;
                    s.colours[0].unk_14[1] = self.colour0_18[i];
                    s.colours[0].unk_14[0] = self.colour0_14[i];
                }
            }
            for (k, c) in [self.colour1[i], self.colour2[i], self.colour3[i]].into_iter().enumerate() {
                if let Some(c) = vdb.find_colour(c[0], c[1]) {
                    s.colours[k + 1] = vdb.colours[c].clone();
                }
            }
            s.unk_18c = self.has_18c[i] as u32;
            s.unk_188 = self.unk_188[i];
            for k in 0..NUM_BODYPART_TYPES {
                if let Some(set) = vdb.find_bodypart_set(self.bodypart_set[i][k]) {
                    if let Some(p) = vdb.find_bodypart(Some(set), k, self.bodypart[i][k]) {
                        s.fitted_bodyparts[k] = Some(p);
                    }
                }
            }
            let d: Vec<Option<usize>> = self.decals[i].iter().map(|&h| vdb.find_decal(h)).collect();
            set_decal_slots(&mut s, &d);
            // (the random colour pass 0x00187BA0 is guarded by a flag the loader never sets)
            if self.vars[var::SLOT] as usize == i {
                current = Some(s.clone());
            }
            vars.add_garage_vehicle(self.vehicle[i], s);
        }
        vdb.unhide_vehicles(vars.prestige as u32);
        for st in 0..4 {
            vdb.update_lock_count(st, vars.prestige as u32, false);
        }
        current
    }
}

/// what a load hands back to game flow
#[derive(Clone, Debug)]
pub struct Applied {
    pub current_setup: Option<VehicleSetup>,
    pub controller_config: u8,
    /// the option globals as loaded (0x001050C0 copies the whole block over `byte_6F915C`)
    pub options: OptionsBlock,
}

/// the setup's 20 decal slots in save order (+0x194, +0x198, +0x19C[10], +0x1EC[2], +0x1FC[2], +0x20C[2], +0x21C,
/// +0x224)
fn decal_slots(s: &VehicleSetup) -> Vec<Option<usize>> {
    let mut v = vec![s.decal_0, s.decal_1];
    v.extend(s.decals_3);
    v.extend(s.decals_3_1);
    v.extend(s.decals_3_2);
    v.extend(s.decals_3_3);
    v.push(s.decal_2_4);
    v.push(s.decal_2_5);
    v
}

fn set_decal_slots(s: &mut VehicleSetup, d: &[Option<usize>]) {
    s.decal_0 = d[0];
    s.decal_1 = d[1];
    s.decals_3.copy_from_slice(&d[2..12]);
    s.decals_3_1.copy_from_slice(&d[12..14]);
    s.decals_3_2.copy_from_slice(&d[14..16]);
    s.decals_3_3.copy_from_slice(&d[16..18]);
    s.decal_2_4 = d[18];
    s.decal_2_5 = d[19];
}

/// `dword_6F719C`..`dword_6F71F0` as the save copies it
fn vars_block(v: &CFEGameVariables) -> [u32; 21] {
    let mut b = [0u32; 21];
    b[var::CASH] = v.cash as u32;
    b[var::PRESTIGE] = v.prestige as u32;
    b[var::NUM_VEHICLES] = v.garage.len() as u32;
    for (i, &c) in v.garage.iter().take(GARAGE_SLOTS).enumerate() {
        b[var::GARAGE + i] = c;
    }
    b[var::VEHICLE] = v.garage_vehicle;
    b[var::SLOT] = v.garage_slot;
    b[var::REWARD] = v.reward.to_bits();
    b[var::UNKNOWN_25C] = v.unknown_25c as u32;
    b[var::TEAM] = v.team;
    b[var::REST..].copy_from_slice(&v.unknown_264);
    b
}

fn apply_vars_block(v: &mut CFEGameVariables, b: &[u32; 21]) {
    v.cash = b[var::CASH] as i32;
    v.prestige = b[var::PRESTIGE] as i32;
    v.garage = b[var::GARAGE..var::GARAGE + (b[var::NUM_VEHICLES] as usize).min(GARAGE_SLOTS)].to_vec();
    v.garage_vehicle = b[var::VEHICLE];
    v.garage_slot = b[var::SLOT];
    v.reward = f32::from_bits(b[var::REWARD]);
    v.unknown_25c = b[var::UNKNOWN_25C] as i32;
    v.team = b[var::TEAM];
    v.unknown_264.copy_from_slice(&b[var::REST..]);
}

/// where profiles live: `VELOCITY_SAVE_DIR`, else `%APPDATA%\Velocity\Saves`, else `<exe dir>\saves`
pub fn save_root() -> PathBuf {
    if let Some(d) = std::env::var_os("VELOCITY_SAVE_DIR") {
        return PathBuf::from(d);
    }
    if let Some(d) = std::env::var_os("APPDATA") {
        return PathBuf::from(d).join("Velocity").join("Saves");
    }
    std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.join("saves")))
        .unwrap_or_else(|| PathBuf::from("saves"))
}

/// the profile name as a folder name (the Xbox container name is the profile name, 7-bit, < 128 chars)
fn folder(name: &str) -> String {
    name.chars()
        .take(127)
        .map(|c| if c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// the "Primary" file of profile `name`
pub fn primary_path(name: &str) -> PathBuf {
    primary_path_in(&save_root(), name)
}

fn primary_path_in(root: &Path, name: &str) -> PathBuf {
    root.join(folder(name)).join("Primary")
}

/// `SAVE_Flow_Save_M("Primary", name, buf, 18656)` (0x00300680)
pub fn save_primary(name: &str, s: &PrimarySave) -> std::io::Result<()> {
    save_primary_in(&save_root(), name, s)
}

pub fn save_primary_in(root: &Path, name: &str, s: &PrimarySave) -> std::io::Result<()> {
    write_atomic(&primary_path_in(root, name), &s.to_bytes())
}

/// create the container folder, write then rename so a crash never leaves half a profile
pub fn write_atomic(p: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d)?;
    }
    let tmp = p.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, p)
}

/// `SAVE_Flow_Load_M("Primary", name, buf, 18656)` (0x002FE790); None if missing or the wrong size
pub fn load_primary(name: &str) -> Option<PrimarySave> {
    load_primary_in(&save_root(), name)
}

pub fn load_primary_in(root: &Path, name: &str) -> Option<PrimarySave> {
    let b = std::fs::read(primary_path_in(root, name)).ok()?;
    PrimarySave::from_bytes(&b)
}

/// `SAVE_Flow_Delete_M("Primary", name)` (0x00300F30)
pub fn delete_primary(name: &str) -> std::io::Result<()> {
    delete_primary_in(&save_root(), name)
}

pub fn delete_primary_in(root: &Path, name: &str) -> std::io::Result<()> {
    let p = primary_path_in(root, name);
    std::fs::remove_file(&p)?;
    if let Some(d) = p.parent() {
        let _ = std::fs::remove_dir(d);
    }
    Ok(())
}

/// the profiles on disk (the profile select's container listing, SAVE_GetFileListing_M)
pub fn list_profiles() -> Vec<String> {
    list_profiles_in(&save_root())
}

pub fn list_profiles_in(root: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root)
        .map(|r| {
            r.flatten()
                .filter(|e| e.path().join("Primary").is_file())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_crc() {
        assert_eq!(visos::handy::handy_crc::crc32(b"Version_25", 0), VERSION_25);
    }

    /// the plate style / text (+0x26C / +0x270) travel in the saved vars block
    #[test]
    fn plate_in_vars_block() {
        let mut v = CFEGameVariables::default();
        v.set_plate_text("VELO 05");
        *v.plate_style_mut() = 3;
        let b = vars_block(&v);
        let mut w = CFEGameVariables::default();
        apply_vars_block(&mut w, &b);
        assert_eq!((w.plate_style(), w.plate_text().as_str()), (3, "VELO 05"));
    }

    /// gather from a played career, save to disk, load into fresh databases: the same block comes back
    #[test]
    fn career_save_load_round_trip() {
        if visos::file::platform::try_game_dir().is_err() {
            return;
        }
        let mut vdb = CVehicleDatabase::load().unwrap();
        let mut vars = CFEGameVariables::default();
        let mut db = CEventDatabase::default();
        db.build(&|c| vdb.vehicles.iter().any(|v| v.hash == c), &vars);
        vars.init_garage_setups(&vdb);
        vars.cash = 123_456;
        vars.prestige = 77;
        vars.reward = 2.5;
        vars.garage_slot = 1;
        vars.garage_setups[1].mileage = 4321.0;
        db.events[0].flags |= EVENT_FLAG_COMPLETED;
        db.races[0].best_record = 3;
        vdb.wheels[0].fitted = 1;
        let mut p = PrimarySave::default();
        let opts = OptionsBlock {
            widescreen: true,
            soundtrack: 2,
            controller_config: 1,
            speech: vars.speech,
            player_names: vars.player_names,
            ..Default::default()
        };
        p.gather(&vars, &db, &vdb, &opts);
        assert!(db.events.len() > 50 && p.events[0].completed && p.num_vehicles == 3);
        let dir = std::env::temp_dir().join(format!("velocity_save_test_{}", std::process::id()));
        save_primary_in(&dir, "TEST", &p).unwrap();
        assert!(list_profiles_in(&dir).contains(&"TEST".to_string()));
        let q = load_primary_in(&dir, "TEST").unwrap();
        assert_eq!(p, q);

        let mut vdb2 = CVehicleDatabase::load().unwrap();
        let mut vars2 = CFEGameVariables::default();
        let mut db2 = CEventDatabase::default();
        db2.build(&|c| vdb2.vehicles.iter().any(|v| v.hash == c), &vars2);
        let a = q.apply(&mut vars2, &mut db2, &mut vdb2).unwrap();
        assert_eq!((vars2.cash, vars2.prestige, vars2.reward), (123_456, 77, 2.5));
        assert_eq!(vars2.garage, vars.garage);
        assert_eq!(a.current_setup.unwrap().mileage, 4321.0);
        assert!(vars2.completed_any && db2.events[0].flags & EVENT_FLAG_COMPLETED != 0);
        assert_eq!(db2.races[0].best_record, 3);
        assert_eq!(vdb2.wheels[0].fitted, 1);
        let mut r = PrimarySave::default();
        assert_eq!((a.options, a.controller_config), (opts, 1));
        r.gather(&vars2, &db2, &vdb2, &a.options);
        assert_eq!(r, q);
        delete_primary_in(&dir, "TEST").unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// option globals at their `byte_6F915C` offsets; the playlists after +0x28 untouched
    #[test]
    fn options_block_layout() {
        let mut b = vec![0xABu8; OPTIONS_SIZE];
        let o = OptionsBlock { volumes: [1.0, 0.25, 0.5, 0.75, 0.125, 0.0625], time_format: 1, controller_config: 3, ..Default::default() };
        o.write(&mut b);
        assert_eq!(&b[0..4], &[0, 0, 1, 0]);
        assert_eq!(&b[4..8], &(-1i32).to_le_bytes());
        assert_eq!(&b[0x14..0x18], &0.75f32.to_le_bytes());
        assert_eq!(&b[0x20..0x24], &1u32.to_le_bytes());
        assert_eq!(&b[0x24..0x28], &[1, 3, 1, 0]);
        assert_eq!(b[OPT_PLAYLISTS], 0xAB);
        assert_eq!(OptionsBlock::read(&b), o);
    }

    #[test]
    fn layout_round_trip() {
        let mut s = PrimarySave { version: VERSION_25, num_vehicles: 2, ..Default::default() };
        s.vehicle[1] = 0x1234;
        s.decals[5][19] = 7;
        s.wheels[29] = [1, 2];
        s.bodyparts[499] = SavedBodypart { set: 3, part: 4, kind: 5 };
        s.vars[20] = 9;
        s.options[OPTIONS_SIZE - 1] = 1;
        s.races[299] = [1, 2, 3];
        s.events[149] = SavedEvent { crc: 8, completed: true, locked: false };
        s.num_completed = 1;
        s.alloys[3239] = 6;
        let b = s.to_bytes();
        assert_eq!(b.len(), PRIMARY_SIZE);
        // field offsets of the original block
        assert_eq!(&b[0x0C..0x10], &0x1234u32.to_le_bytes());
        assert_eq!(&b[0x0640 + 29 * 8..0x0640 + 30 * 8], &[1, 0, 0, 0, 2, 0, 0, 0]);
        assert_eq!(b[0x1EA0 - 4], 5);
        assert_eq!(b[0x1EF4 - 4], 9);
        assert_eq!(b[0x2974 - 1], 1);
        assert_eq!(&b[0x3C34..0x3C38], &1u32.to_le_bytes());
        assert_eq!(b[PRIMARY_SIZE - 1], 6);
        assert_eq!(PrimarySave::from_bytes(&b).unwrap(), s);
    }
}
