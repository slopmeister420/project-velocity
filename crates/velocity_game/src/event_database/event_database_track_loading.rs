//! src\EventDatabase\EventDatabaseTrackLoading.cpp — the track/race catalogue built from `\FE\TRACKS.INI` and each
//! track's `\Tracks\<TRACK>\setup.ini`.
//!
//! `TRACKS.INI` is a list of 9-line records `Key : "VALUE"` (Area, Track, Type, Race, DisplayID, Category, Length,
//! Icon, Available); `setup.ini` holds `Name : "LA 1"` and `Traffic_Area : "LA"`.

use visos::file::vs_file_directory::VsFileDirectoryDetails;
use visos::handy::handy_crc;
use visos::handy::vs_text_file::VsTextFile;

/// race type names (`off_685434`), indexed by `ERaceType`.
pub const RACE_TYPE_NAMES: [&str; 7] = ["UNKNOWN", "CRUISE", "SPRINT", "CIRCUIT", "RELAY", "DRAG", "PURSUIT"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum ERaceType {
    #[default]
    Unknown = 0,
    Cruise = 1,
    Sprint = 2,
    Circuit = 3,
    Relay = 4,
    Drag = 5,
    Pursuit = 6,
}

impl ERaceType {
    pub fn from_index(i: u32) -> Self {
        match i {
            1 => Self::Cruise,
            2 => Self::Sprint,
            3 => Self::Circuit,
            4 => Self::Relay,
            5 => Self::Drag,
            6 => Self::Pursuit,
            _ => Self::Unknown,
        }
    }
}

/// race available flags (+0x25)
pub const RACE_FLAG_AVAILABLE: u8 = 1;
pub const RACE_FLAG_UNLOCKED: u8 = 2;

/// `CEventRace` (0x40 bytes)
#[derive(Clone, Debug, Default)]
pub struct CEventRace {
    pub name_crc: u32,      // +0x00 crc of upper-case race dir (RACE01)
    pub display_crc: u32,   // +0x04 DisplayID crc
    pub id_crc: u32,        // +0x08 crc of "<TRACK>_<TYPE>_RACE_%02d"
    pub name: String,       // +0x0C
    pub path: String,       // +0x10 "CIRCUIT\RACE01\"
    pub icon: String,       // +0x14
    pub index: u32,         // +0x18 index within its type on the track
    /// +0x1C best race time (900 s until beaten; GameManager.cpp 0x000BE430)
    pub unknown_28: f32,
    /// +0x20 best racer record (the race vehicles' +0x21C, GameManager.cpp 0x000BE430)
    pub best_record: u32,
    pub race_type: ERaceType, // +0x28
    pub category: u32,      // +0x2C 1 underground, 2 professional, 3 challenge
    pub length: u32,        // +0x30 1 short, 2 medium, 3 long
    pub flags: u8,          // +0x25
    pub track: usize,       // +0x34
}

/// `CEventTrack` (0x94 bytes)
#[derive(Clone, Debug, Default)]
pub struct CEventTrack {
    /// per race type lists (+0x00, 7 x 12 bytes) of indices into `CEventDatabase::races`
    pub races: [Vec<usize>; 7],
    pub name: String,         // +0x70 "LACOAST"
    pub display_name: String, // +0x74 setup.ini Name
    pub path: String,         // +0x78 "Tracks\LACOAST\"
    pub name_crc: u32,        // +0x7C
    pub index: u32,           // +0x80 TRACK_%02d
    pub traffic_area: u32,    // +0x84 1 LA, 2 LONDON, 3 TOKYO
    pub area: u32,            // +0x88 1 LA, 2 London, 3 Tokyo, 4 other
}

/// the track side of `CEventDatabase`.
#[derive(Clone, Debug, Default)]
pub struct CEventDatabase {
    pub tracks: Vec<CEventTrack>,
    pub races: Vec<CEventRace>,
    /// `GameManager+0x35` jump/scan mode: every race becomes selectable
    pub jump_mode: bool,
    /// +0x0C career events in list order (EventDatabase.cpp)
    pub events: Vec<super::event_database::CEvent>,
    /// +0x18 u16 event count (counts every event twice, see `load_events`)
    pub num_events: u16,
}

fn crc_upper(s: &str) -> u32 {
    let u = s.to_ascii_uppercase();
    if u.is_empty() { 0 } else { handy_crc::crc32(u.as_bytes(), 0) }
}

impl CEventDatabase {
    /// EventDatabaseTrackLoading.cpp 0x000948D0 — load from `\FE` (or another directory).
    pub fn load_tracks(&mut self, dir: &str) -> bool {
        let Ok(d) = VsFileDirectoryDetails::open(dir) else {
            log::warn!("No track information found");
            return false;
        };
        self.load_tracks_ini(&d)
    }

    /// EventDatabaseTrackLoading.cpp 0x00094240
    pub fn load_tracks_ini(&mut self, dir: &VsFileDirectoryDetails) -> bool {
        let mut f = VsTextFile::default();
        if !f.load(Some(dir), "TRACKS.INI") {
            log::warn!("Unable to open Tracks.ini file");
            return false;
        }
        loop {
            let Some((_, area)) = f.read_key_value() else { break };
            let mut next = || f.read_key_value().map(|kv| kv.1).unwrap_or_default();
            let track = next();
            let race_type = next();
            let race = next();
            let display_id = next();
            let category = next();
            let length = next();
            let icon = next();
            let available = next();

            let area_id = if area.eq_ignore_ascii_case("LA") {
                1
            } else if area.eq_ignore_ascii_case("London") {
                2
            } else if area.eq_ignore_ascii_case("Tokyo") {
                3
            } else {
                4
            };
            let t = match self.find_track(area_id, &track) {
                Some(t) => Some(t),
                None => self.add_track(area_id, &track),
            };
            let rt = Self::race_type_from_name(&race_type);
            let cat = if category.eq_ignore_ascii_case("P") {
                2
            } else if category.eq_ignore_ascii_case("U") {
                1
            } else if category.eq_ignore_ascii_case("C") {
                3
            } else {
                debug_assert!(false, "EVENT Category not set!!!!!!!");
                0
            };
            let len = if length.eq_ignore_ascii_case("SHORT") {
                1
            } else if length.eq_ignore_ascii_case("MEDIUM") {
                2
            } else if length.eq_ignore_ascii_case("LONG") {
                3
            } else {
                0
            };
            let Some(t) = t else { continue };
            if self.find_race(t, cat, rt, &race).is_none() {
                let flags = if available.eq_ignore_ascii_case("YES") {
                    3
                } else if self.jump_mode {
                    2
                } else {
                    0
                };
                let dcrc = if display_id.is_empty() { 0 } else { handy_crc::crc32(display_id.as_bytes(), 0) };
                self.add_race(t, cat, rt, len, &race, &icon, dcrc, flags);
            }
        }
        true
    }

    /// EventDatabaseTrackLoading.cpp 0x00095B70 — case-insensitive lookup in `RACE_TYPE_NAMES`.
    pub fn race_type_from_name(name: &str) -> ERaceType {
        let u = name.to_ascii_uppercase();
        match RACE_TYPE_NAMES.iter().position(|n| *n == u) {
            Some(i) => ERaceType::from_index(i as u32),
            None => {
                log::warn!("WARNING: Race category [{}] not found", u);
                ERaceType::Unknown
            }
        }
    }

    /// EventDatabaseTrackLoading.cpp 0x00096250 — by area (0 = any) and name.
    pub fn find_track(&self, area: u32, name: &str) -> Option<usize> {
        self.find_track_crc(area, crc_upper(name))
    }

    /// EventDatabaseTrackLoading.cpp 0x00096190 — by area (0 = any) and name crc.
    pub fn find_track_crc(&self, area: u32, crc: u32) -> Option<usize> {
        self.tracks.iter().position(|t| (area == 0 || t.area == area) && t.name_crc == crc)
    }

    /// EventDatabaseTrackLoading.cpp 0x00095C50
    pub fn add_track(&mut self, area: u32, name: &str) -> Option<usize> {
        if name.is_empty() {
            log::warn!("Error: Invalid track name for Event Database");
            return None;
        }
        let upper = name.to_ascii_uppercase();
        if self.find_track(area, &upper).is_some() {
            log::warn!("Error: Attempt to add track with same name as existing one [{}]", upper);
            return None;
        }
        let mut t = CEventTrack { area, name_crc: crc_upper(&upper), name: upper.clone(), ..Default::default() };
        let mut f = VsTextFile::default();
        let dir = VsFileDirectoryDetails::open(&format!("\\Tracks\\{}", upper)).ok();
        if dir.is_some() && f.load(dir.as_ref(), "setup.INI") {
            t.display_name = f.read_key_value().map(|kv| kv.1).unwrap_or_default();
            let ta = f.read_key_value().map(|kv| kv.1.to_ascii_uppercase()).unwrap_or_default();
            match ta.as_str() {
                "LA" => t.traffic_area = 1,
                "LONDON" => t.traffic_area = 2,
                "TOKYO" => t.traffic_area = 3,
                _ => log::warn!("WARNING: Unknown area traffic type [{}]", ta),
            }
        } else {
            log::warn!("Unable to open setup.ini file in [Tracks\\{}]. Just using default name", upper);
            t.display_name = upper.clone();
            t.traffic_area = 1;
        }
        t.path = format!("Tracks\\{}\\", upper);
        t.index = self.tracks.len() as u32;
        self.tracks.push(t);
        Some(self.tracks.len() - 1)
    }

    /// EventDatabaseTrackLoading.cpp 0x000955A0 — race by name on a track; categories 3/4 search as 1.
    pub fn find_race(&self, track: usize, category: u32, rt: ERaceType, name: &str) -> Option<usize> {
        self.find_race_crc(track, category, rt, crc_upper(name))
    }

    /// EventDatabaseTrackLoading.cpp 0x000954C0 — the same by name crc.
    pub fn find_race_crc(&self, track: usize, mut category: u32, rt: ERaceType, c: u32) -> Option<usize> {
        for &r in &self.tracks[track].races[rt as usize] {
            if category == 3 || category == 4 {
                category = 1;
                log::warn!("Warning... making this challenge an Underground race for now !!!");
            }
            let race = &self.races[r];
            if (category == 0 || race.category == category) && race.name_crc == c {
                return Some(r);
            }
        }
        None
    }

    /// EventDatabaseTrackLoading.cpp 0x00095770
    #[allow(clippy::too_many_arguments)]
    pub fn add_race(
        &mut self,
        track: usize,
        category: u32,
        rt: ERaceType,
        length: u32,
        name: &str,
        icon: &str,
        display_crc: u32,
        mut flags: u8,
    ) -> Option<usize> {
        if name.is_empty() {
            log::warn!("Error: Invalid race name for Event Database");
            return None;
        }
        let upper = name.to_ascii_uppercase();
        if self.find_race(track, category, rt, &upper).is_some() {
            log::warn!("Error: Attempt to add race with same name as existing one [{}]", upper);
            return None;
        }
        let index = self.tracks[track].races[rt as usize].len() as u32;
        let id = format!("{}_{}_RACE_{:02}", self.tracks[track].name, RACE_TYPE_NAMES[rt as usize], index + 1);
        let none = handy_crc::crc32(b"NONE", 0);
        let display_crc = if display_crc == 0 || display_crc == none { crc_upper(&id) } else { display_crc };
        if flags & RACE_FLAG_AVAILABLE != 0 {
            flags |= RACE_FLAG_UNLOCKED;
        }
        self.races.push(CEventRace {
            name_crc: crc_upper(&upper),
            display_crc,
            id_crc: crc_upper(&id),
            name: upper.clone(),
            path: format!("{}\\{}\\", RACE_TYPE_NAMES[rt as usize], upper),
            icon: icon.to_string(),
            index,
            unknown_28: f32::from_bits(0x4461_0000),
            best_record: 0,
            race_type: rt,
            category,
            length,
            flags,
            track,
        });
        let r = self.races.len() - 1;
        self.tracks[track].races[rt as usize].push(r);
        Some(r)
    }

    /// EventDatabaseTrackLoading.cpp 0x00094990 — jump/scan mode marks every race unlocked.
    pub fn set_all_unlocked(&mut self, on: bool) {
        for r in &mut self.races {
            if on {
                r.flags |= RACE_FLAG_UNLOCKED;
            } else if r.flags & RACE_FLAG_AVAILABLE == 0 {
                r.flags &= !RACE_FLAG_UNLOCKED;
            }
        }
    }

    /// the game directory of a race: `\Tracks\<TRACK>\<TYPE>\<RACE>\`.
    pub fn race_dir(&self, race: usize) -> String {
        let r = &self.races[race];
        format!("\\{}{}", self.tracks[r.track].path, r.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_ini() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let mut db = CEventDatabase::default();
        assert!(db.load_tracks("\\FE"));
        assert_eq!(db.races.len(), 99);
        let la = db.find_track(1, "lacoast").unwrap();
        assert_eq!(db.tracks[la].display_name, "LA 1");
        assert_eq!(db.tracks[la].traffic_area, 1);
        let r = db.find_race(la, 0, ERaceType::Circuit, "Race01").unwrap();
        assert_eq!(db.race_dir(r), "\\Tracks\\LACOAST\\CIRCUIT\\RACE01\\");
        // not every catalogued race ships in this build; count the ones whose directory exists
        let present: Vec<String> = (0..db.races.len()).map(|i| db.race_dir(i)).filter(|d| visos::file::platform::resolve(d).is_dir()).collect();
        eprintln!("{} of {} races present", present.len(), db.races.len());
        assert!(present.len() >= 40);
    }
}
