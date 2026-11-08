//! src\EventDatabase\EventDatabase.cpp — the career events of `CEventDatabase` (`dword_6F0D74`): their records,
//! the binary reader (`\EVENTS\EVENTS.VCD`, one `*.bin` per event, format 1.5), the link passes and the unlock
//! evaluation. Pointers of the original become indices: tracks / races into `CEventDatabase`, events into
//! `CEventDatabase::events`, competitors into the event's own competitor list.
//!
//! Event record, 96 bytes followed by its sub-records in this order (each only when its pointer field is
//! non-zero in the file): unlock criteria (24 + 4*events + 4*vehicles), races (24, then per round 8 + 36 per
//! race), competitors (8 + 24 each), result (8), then per-round competitor crcs (4 * per-cup-race count, one
//! list per round of `races+0`).

use visos::file::vs_file_directory::VsFileDirectoryDetails;

use super::event_database_track_loading::{CEventDatabase, ERaceType};

/// `CEvent+0x15` flags
pub const EVENT_FLAG_LOCKED: u8 = 1;
pub const EVENT_FLAG_COMPLETED: u8 = 2;
/// the player failed (eliminated, wrong position)
pub const EVENT_FLAG_FAILED: u8 = 4;
/// built at runtime (quick race, jump), not part of the database
pub const EVENT_FLAG_TEMP: u8 = 8;
/// UpdateAtEndOfEvent ran
pub const EVENT_FLAG_FINISHED: u8 = 0x10;

/// `CEventRaceEntry+0x16` flags: 1 and 2 come from the builders' last two flags, 4 randomises bit 0 on each
/// round start (EventDatabaseBinLoading.cpp 0x000913B0)
pub const RACE_ENTRY_FLAG_RANDOM_BIT0: u8 = 4;

/// event kinds (`CEvent+0x04`)
pub const EVENT_KIND_SINGLE: u32 = 1;
/// knockout cup: rounds of 2- or 4-car races, winners go through
pub const EVENT_KIND_CUP: u32 = 2;
/// series: every competitor in every round, points per finish
pub const EVENT_KIND_SERIES: u32 = 3;

/// file format version (EventDatabaseBinLoading.cpp 0x00090E60)
pub const EVENT_FILE_VERSION: f32 = 1.5;

/// `CEventUnlockCriteria` (24 bytes)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CEventUnlockCriteria {
    /// +0x00 needed player prestige (`dword_6F71A0`), ignored when <= 0
    pub prestige: f32,
    /// +0x04 completed events needed in the event's own tier
    pub completed_in_tier: u8,
    /// +0x05 count, +0x08 crcs of events that must be completed
    pub event_crcs: Vec<u32>,
    /// +0x0C resolved events (EventDatabase.cpp 0x0008F450)
    pub events: Vec<Option<usize>>,
    /// +0x06 count, +0x10 crcs of vehicles the player must own
    pub vehicle_crcs: Vec<u32>,
    /// +0x14 found in the vehicle database
    pub vehicles_found: Vec<bool>,
}

/// `CEventRaceEntry` (36 bytes)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CEventRaceEntry {
    pub track_crc: u32,             // +0x00
    pub race_type: u32,             // +0x04 ERaceType
    pub race_crc: u32,              // +0x08
    pub track: Option<usize>,       // +0x0C
    pub race: Option<usize>,        // +0x10
    pub laps: u8,                   // +0x14
    pub num_competitors: u8,        // +0x15
    pub flags: u8,                  // +0x16
    pub unknown_17: u8,             // +0x17
    pub unknown_18: u32,            // +0x18
    /// +0x1C finish position per competitor slot (0 = not finished)
    pub finish_positions: Vec<u8>,
    /// +0x20 competitor per slot (index into the event's competitors)
    pub competitors: Vec<Option<usize>>,
}

/// `CEventRound` (8 bytes)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CEventRound {
    pub current_race: u8,              // +0x01
    pub races: Vec<CEventRaceEntry>,   // +0x00 count, +0x04
}

/// `CEventRaces` (24 bytes)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CEventRaces {
    /// +0x00 rounds with per-round competitor lists (`CEvent+0x28`)
    pub num_competitor_rounds: u8,
    pub current_round: u8,           // +0x02
    pub rounds: Vec<CEventRound>,    // +0x01 count, +0x04
    /// +0x10 the round before the current one (`+0x08` is the current round, `+0x0C` its current race)
    pub previous_round: Option<u8>,
    /// +0x14 (round, race) finished last (EventDatabaseBinLoading.cpp 0x00091620); runtime only
    pub last_race: Option<(u8, u8)>,
}

/// `CEventCompetitor` (24 bytes)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CEventCompetitor {
    /// +0x00 opponent crc (`OpponentDatabase` name crc), 0 for human slots
    pub opponent_crc: u32,
    /// +0x04 resolved opponent (EventDatabaseEvents.cpp 0x00092070); the opponent database is the ai crate's,
    /// so consumers look `opponent_crc` up themselves
    pub opponent_found: bool,
    pub points: u32,       // +0x08
    pub unknown_0c: u32,   // +0x0C
    /// +0x10 money staked on the event (won if the first competitor wins the last race)
    pub wager: i32,
    /// +0x14 1 = a human player
    pub human: u32,
}

/// `CEventResult` (8 bytes)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CEventResult {
    /// +0x00 required finish position (0 = any)
    pub position: u8,
    /// +0x04 points the leader must not exceed (series)
    pub points: u32,
}

/// `CEvent` (96 bytes)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CEvent {
    pub category: u32,                          // +0x00 1 underground, 2 professional
    pub kind: u32,                              // +0x04 EVENT_KIND_*
    pub unknown_08: u32,                        // +0x08
    pub area: u32,                              // +0x0C 1 LA, 2 London, 3 Tokyo
    pub crc: u32,                               // +0x10
    pub tier: u8,                               // +0x14
    pub flags: u8,                              // +0x15 EVENT_FLAG_*
    pub order: u8,                              // +0x16 sort key
    pub competitors_per_cup_race: u8,           // +0x17
    pub prestige_reward: u8,                    // +0x18 added to `dword_6F71C8`
    pub unlock: Option<CEventUnlockCriteria>,   // +0x1C
    pub races: Option<CEventRaces>,             // +0x20
    pub competitors: Option<Vec<CEventCompetitor>>, // +0x24 (u8 count, ptr)
    /// +0x28 per competitor-round crcs of the competitors in each race (cups)
    pub round_competitors: Option<Vec<Vec<u32>>>,
    pub result: Option<CEventResult>,           // +0x2C
    pub prize: f32,                             // +0x30 prize pool
    pub unknown_34: [u32; 3],                   // +0x34..+0x3F
    pub unlock_team: u32,                       // +0x40 team crc unlocked on completion
    pub unknown_44: u32,                        // +0x44
    pub unlock_message: u32,                    // +0x48 shown when the event unlocks (`dword_6F7248`)
    pub unknown_4c: [u32; 3],                   // +0x4C..+0x57 (+0x58/+0x5C list links)
}

#[derive(Debug, thiserror::Error)]
pub enum EventReadError {
    #[error("event file truncated at {0:#x}")]
    Truncated(usize),
    #[error("Event database version wrong expecting {expected:.2} got {got:.2}")]
    Version { got: f32, expected: f32 },
}

struct Rd<'a> {
    b: &'a [u8],
    p: usize,
}

impl Rd<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], EventReadError> {
        let s = self.b.get(self.p..self.p + n).ok_or(EventReadError::Truncated(self.p))?;
        self.p += n;
        Ok(s)
    }
}

fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

fn u32s(b: &[u8]) -> Vec<u32> {
    b.chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect()
}

impl CEventUnlockCriteria {
    /// EventDatabase.cpp 0x00090920
    fn read(r: &mut Rd) -> Result<Self, EventReadError> {
        let h = r.take(24)?.to_vec();
        let mut c = CEventUnlockCriteria {
            prestige: f32::from_bits(u32_at(&h, 0)),
            completed_in_tier: h[4],
            ..Default::default()
        };
        if h[5] != 0 {
            c.event_crcs = u32s(r.take(4 * h[5] as usize)?);
        }
        if h[6] != 0 {
            c.vehicle_crcs = u32s(r.take(4 * h[6] as usize)?);
        }
        Ok(c)
    }
}

impl CEventRaceEntry {
    /// EventDatabase.cpp 0x00090A20
    fn read(r: &mut Rd) -> Result<Self, EventReadError> {
        let h = r.take(36)?;
        Ok(CEventRaceEntry {
            track_crc: u32_at(h, 0),
            race_type: u32_at(h, 4),
            race_crc: u32_at(h, 8),
            track: None,
            race: None,
            laps: h[20],
            num_competitors: h[21],
            flags: h[22],
            unknown_17: h[23],
            unknown_18: u32_at(h, 24),
            finish_positions: Vec::new(),
            competitors: Vec::new(),
        })
    }
}

impl CEventRaces {
    /// EventDatabase.cpp 0x00090B00 (rounds 0x00090A60)
    fn read(r: &mut Rd) -> Result<Self, EventReadError> {
        let h = r.take(24)?.to_vec();
        let mut v = CEventRaces {
            num_competitor_rounds: h[0],
            current_round: h[2],
            ..Default::default()
        };
        for _ in 0..h[1] {
            let rh = r.take(8)?.to_vec();
            let mut round = CEventRound { current_race: rh[1], races: Vec::new() };
            for _ in 0..rh[0] {
                round.races.push(CEventRaceEntry::read(r)?);
            }
            v.rounds.push(round);
        }
        Ok(v)
    }
}

impl CEvent {
    /// EventDatabase.cpp 0x00090CC0 — one event and its sub-records.
    fn read(r: &mut Rd) -> Result<Self, EventReadError> {
        let h = r.take(96)?.to_vec();
        let w = |o: usize| u32_at(&h, o);
        let mut e = CEvent {
            category: w(0),
            kind: w(4),
            unknown_08: w(8),
            area: w(12),
            crc: w(16),
            tier: h[20],
            flags: h[21],
            order: h[22],
            competitors_per_cup_race: h[23],
            prestige_reward: h[24],
            prize: f32::from_bits(w(0x30)),
            unknown_34: [w(0x34), w(0x38), w(0x3C)],
            unlock_team: w(0x40),
            unknown_44: w(0x44),
            unlock_message: w(0x48),
            unknown_4c: [w(0x4C), w(0x50), w(0x54)],
            ..Default::default()
        };
        if w(0x1C) != 0 {
            e.unlock = Some(CEventUnlockCriteria::read(r)?);
        }
        if w(0x20) != 0 {
            e.races = Some(CEventRaces::read(r)?);
        }
        if w(0x24) != 0 {
            // EventDatabase.cpp 0x00090BE0 / 0x00090BA0
            let n = r.take(8)?[0];
            let mut list = Vec::new();
            for _ in 0..n {
                let c = r.take(24)?;
                list.push(CEventCompetitor {
                    opponent_crc: u32_at(c, 0),
                    opponent_found: false,
                    points: u32_at(c, 8),
                    unknown_0c: u32_at(c, 12),
                    wager: u32_at(c, 16) as i32,
                    human: u32_at(c, 20),
                });
            }
            e.competitors = Some(list);
        }
        if w(0x2C) != 0 {
            // EventDatabase.cpp 0x00090C80
            let c = r.take(8)?;
            e.result = Some(CEventResult { position: c[0], points: u32_at(c, 4) });
        }
        if w(0x28) != 0 {
            // one list per round counted by races+0, each `competitors_per_cup_race` long (a null races record
            // crashes the original here)
            let rounds = e.races.as_ref().map(|r| r.num_competitor_rounds).unwrap_or(0);
            let mut v = Vec::new();
            for _ in 0..rounds {
                v.push(u32s(r.take(4 * e.competitors_per_cup_race as usize)?));
            }
            e.round_competitors = Some(v);
        }
        Ok(e)
    }

    /// the race being run: races +0x0C (`VsAIPoliceVehicle__17CB0` in the idb).
    pub fn current_race(&self) -> Option<&CEventRaceEntry> {
        let r = self.races.as_ref()?;
        let round = r.rounds.get(r.current_round as usize)?;
        round.races.get(round.current_race as usize)
    }

    pub fn current_race_mut(&mut self) -> Option<&mut CEventRaceEntry> {
        let r = self.races.as_mut()?;
        let round = r.rounds.get_mut(r.current_round as usize)?;
        round.races.get_mut(round.current_race as usize)
    }

    pub fn competitor_count(&self) -> u8 {
        self.competitors.as_ref().map(|c| c.len() as u8).unwrap_or(0)
    }
}

/// EventDatabaseBinLoading.cpp 0x00090E60 — read one `*.bin`: f32 version, u16 count, then the events.
pub fn read_event_file(data: &[u8]) -> Result<Vec<CEvent>, EventReadError> {
    let mut r = Rd { b: data, p: 0 };
    let version = f32::from_le_bytes(r.take(4)?.try_into().unwrap());
    if version != EVENT_FILE_VERSION {
        log::error!("ERROR: Event Database event data exported with out-of-date exporter.  Synch to Tools\\Bin and re-export");
        return Err(EventReadError::Version { got: version, expected: EVENT_FILE_VERSION });
    }
    let n = u16::from_le_bytes(r.take(2)?.try_into().unwrap());
    let mut v = Vec::new();
    for _ in 0..n {
        v.push(CEvent::read(&mut r)?);
    }
    Ok(v)
}

/// the player career data the unlock evaluation reads (`g_FEGameVariables` player 0).
pub trait CareerQuery {
    /// `dword_6F71A0` prestige
    fn prestige(&self) -> i32;
    /// CompartmentManager__8E930 (idb name; GamePlayer data): is the vehicle in the garage
    fn owns_vehicle(&self, crc: u32) -> bool;
}

impl CEventDatabase {
    /// EventDatabase.cpp 0x00090090 — "Building Event Database": tracks, then `\EVENTS\EVENTS.VCD`, then the
    /// link passes and the unlock evaluation. `vehicle_exists` is `CVehicleDatabase::FindVehicle`.
    pub fn build(&mut self, vehicle_exists: &dyn Fn(u32) -> bool, career: &dyn CareerQuery) {
        log::info!("Building Event Database...");
        if !self.load_tracks("\\FE") {
            log::warn!("No track information found");
        }
        self.load_events("\\EVENTS\\EVENTS.VCD");
        self.link_unlock_criteria(vehicle_exists);
        self.allocate_race_slots();
        self.link_races();
        self.evaluate_unlocks(career, false);
        log::info!("Event Database built");
    }

    /// EventDatabase.cpp 0x0008F9A0 — every file of the pack in toc order.
    pub fn load_events(&mut self, dir: &str) {
        let Ok(d) = VsFileDirectoryDetails::open(dir) else {
            log::warn!("Event Database: unable to open {dir}");
            return;
        };
        for name in d.file_names() {
            let Ok(b) = d.load_file(&name) else { continue };
            match read_event_file(&b) {
                Ok(evs) => {
                    // 0x00090E60 adds the file's count, then each insert adds one more (kept)
                    self.num_events = self.num_events.wrapping_add(evs.len() as u16);
                    for e in evs {
                        self.insert_event(e);
                    }
                }
                Err(e) => log::error!("{name}: {e}"),
            }
        }
    }

    /// EventDatabase.cpp 0x0008F450 — resolve the unlock criteria's vehicles and events. (the original allocates
    /// the vehicle array into +0x0C and writes the lookups through the file's +0x14 value; harmless as shipped)
    pub fn link_unlock_criteria(&mut self, vehicle_exists: &dyn Fn(u32) -> bool) {
        let crcs: Vec<u32> = self.events.iter().map(|e| e.crc).collect();
        for e in &mut self.events {
            let Some(u) = e.unlock.as_mut() else { continue };
            u.vehicles_found = u.vehicle_crcs.iter().map(|&c| vehicle_exists(c)).collect();
            if u.vehicles_found.iter().any(|f| !f) {
                log::warn!("Event Database: Unable to find vehicle with matching CRC. This could cause problems");
            }
            u.events = u.event_crcs.iter().map(|&c| crcs.iter().position(|&x| x == c)).collect();
            if u.events.iter().any(Option::is_none) {
                log::warn!("Event Database: Unable to find event with matching CRC.  This could cause problems");
            }
        }
    }

    /// EventDatabase.cpp 0x0008F620 — size every race's slots: all competitors, or for cups
    /// `competitors * 0.5^(rounds - 1)`.
    pub fn allocate_race_slots(&mut self) {
        for e in &mut self.events {
            let n = e.competitor_count();
            let kind = e.kind;
            let Some(races) = e.races.as_mut() else { continue };
            if n == 0 {
                continue;
            }
            let rounds = races.rounds.len();
            for round in &mut races.rounds {
                for race in &mut round.races {
                    race.num_competitors = if kind == EVENT_KIND_CUP {
                        // VsPow (x87) then ftol
                        let p = (0.5f32 as f64).powf((rounds as f32 - 1.0) as f64);
                        (p * n as f64) as i32 as u8
                    } else {
                        n
                    };
                    race.finish_positions = vec![0; race.num_competitors as usize];
                    race.competitors = vec![None; race.num_competitors as usize];
                }
            }
        }
    }

    /// EventDatabase.cpp 0x0008F800 — resolve each race entry's track (by the event's area) and race.
    pub fn link_races(&mut self) {
        let mut events = std::mem::take(&mut self.events);
        for e in &mut events {
            let (area, cat) = (e.area, e.category);
            let Some(races) = e.races.as_mut() else { continue };
            for round in &mut races.rounds {
                for race in &mut round.races {
                    race.track = self.find_track_crc(area, race.track_crc);
                    let Some(t) = race.track else {
                        log::warn!("Event in database without matching track");
                        break;
                    };
                    race.race = self.find_race_crc(t, cat, ERaceType::from_index(race.race_type), race.race_crc);
                    if race.race.is_none() {
                        log::warn!("Event in database without matching race");
                        break;
                    }
                }
            }
        }
        self.events = events;
    }

    /// EventDatabase.cpp 0x0008F020 — unlock every event whose criteria hold, lock the rest; repeats while
    /// anything changes (50 passes at most). `announce` = completion path: returns the unlock message of the
    /// last event it unlocked (`dword_6F7248`).
    pub fn evaluate_unlocks(&mut self, career: &dyn CareerQuery, announce: bool) -> Option<u32> {
        let mut message = None;
        let mut passes = 50;
        loop {
            let mut changed = false;
            let mut per_tier = [0u8; 10];
            let mut total = 0u8;
            for e in &self.events {
                if (e.category == 1 || e.category == 2) && e.flags & EVENT_FLAG_COMPLETED != 0 {
                    if total >= 250 {
                        log::error!("assert: completed event count");
                    }
                    if e.tier >= 10 {
                        log::error!("assert: event tier < 10");
                        continue;
                    }
                    per_tier[e.tier as usize] += 1;
                    total = total.wrapping_add(1);
                }
            }
            for i in 0..self.events.len() {
                let e = &self.events[i];
                let Some(u) = e.unlock.as_ref() else { continue };
                let events_ok = u
                    .events
                    .iter()
                    .all(|x| x.is_some_and(|j| self.events[j].flags & EVENT_FLAG_COMPLETED != 0));
                let vehicles_ok = u
                    .vehicle_crcs
                    .iter()
                    .zip(&u.vehicles_found)
                    .all(|(&c, &found)| found && career.owns_vehicle(c));
                let prestige_ok = !(u.prestige > 0.0 && career.prestige() < u.prestige as i32);
                let tier_ok = u.completed_in_tier <= per_tier.get(e.tier as usize).copied().unwrap_or(0);
                let msg = e.unlock_message;
                if events_ok && vehicles_ok && prestige_ok && tier_ok {
                    if self.events[i].set_unlocked(true) {
                        changed = true;
                        if announce && msg != 0 {
                            message = Some(msg);
                        }
                    }
                } else if self.events[i].set_unlocked(false) {
                    changed = true;
                }
            }
            passes -= 1;
            if passes == 0 {
                log::error!("ERROR: Looks like an infinite loop in Event Unlocking.  Check for closed cycle of event dependencies");
                break;
            }
            if !changed {
                break;
            }
        }
        message
    }

    /// FECK2Manager.cpp 0x00202EA0 ("Unlock ALL Events" tweak on) — every event unlocked.
    pub fn unlock_all_events(&mut self) {
        for e in &mut self.events {
            e.set_unlocked(true);
        }
    }

    /// CompartmentManager__8EDB0 (idb name, EventDatabase code): events of a category in a tier (0 = any),
    /// optionally only those with (`set`) or without some flag bits.
    pub fn count_events(&self, category: u32, tier: u8, flags: u8, without: bool) -> u32 {
        self.events
            .iter()
            .filter(|e| category == e.category && (tier == 0 || tier == e.tier))
            .filter(|e| flags == 0 || if without { flags & e.flags == 0 } else { flags & e.flags != 0 })
            .count() as u32
    }

    /// CompartmentManager__8EC20 (idb name) — reset every event (GamePlayer SetInitialVariables).
    pub fn reset_events(&mut self, rand: &mut visos::handy::crt_rand::CrtRand) {
        for e in &mut self.events {
            e.reset(rand);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoCareer;
    impl CareerQuery for NoCareer {
        fn prestige(&self) -> i32 {
            0
        }
        fn owns_vehicle(&self, _: u32) -> bool {
            false
        }
    }

    /// every shipped event file parses to its exact length and links to tracks and races.
    #[test]
    fn events_vcd_all() {
        if visos::file::platform::try_game_dir().is_err() {
            return;
        }
        let d = VsFileDirectoryDetails::open("\\EVENTS\\EVENTS.VCD").unwrap();
        let names = d.file_names();
        assert_eq!(names.len(), 97);
        for n in &names {
            let b = d.load_file(n).unwrap();
            let mut r = Rd { b: &b, p: 0 };
            r.take(6).unwrap();
            let e = CEvent::read(&mut r).unwrap();
            assert_eq!(r.p, b.len(), "{n} not consumed");
            assert!(e.races.is_some() && e.competitors.is_some(), "{n}");
        }
        let mut db = CEventDatabase::default();
        db.build(&|_| true, &NoCareer);
        assert_eq!(db.events.len(), 97);
        let mut unresolved = Vec::new();
        let mut dangling = 0;
        for e in &db.events {
            for round in &e.races.as_ref().unwrap().rounds {
                for race in &round.races {
                    if race.track.is_none() || race.race.is_none() {
                        unresolved.push(e.crc);
                    }
                }
            }
            if let Some(u) = &e.unlock {
                dangling += u.events.iter().filter(|x| x.is_none()).count();
            }
        }
        // shipped data: 22 race entries name races this build's TRACKS.INI lacks (or lists under the other
        // category), one prerequisite a missing event; the original warns the same way
        assert_eq!((unresolved.len(), dangling), (22, 1));
        let locked = db.events.iter().filter(|e| e.flags & EVENT_FLAG_LOCKED != 0).count();
        assert!(locked > 0 && locked < db.events.len());
        db.unlock_all_events();
        assert!(db.events.iter().all(|e| e.flags & EVENT_FLAG_LOCKED == 0));
    }
}
