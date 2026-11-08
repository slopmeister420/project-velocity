//! src\EventDatabase\EventDatabaseEvents.cpp — per-event state: locking, the event list, reset and round set-up,
//! competitor lookups and the temporary events built for quick races and jumps.

use visos::handy::crt_rand::CrtRand;
use visos::handy::handy_crc;

use super::event_database::{
    CEvent, CEventCompetitor, CEventRaceEntry, CEventRaces, CEventResult, CEventRound, EVENT_FLAG_COMPLETED,
    EVENT_FLAG_FAILED, EVENT_FLAG_FINISHED, EVENT_FLAG_LOCKED, EVENT_FLAG_TEMP, EVENT_KIND_CUP, EVENT_KIND_SERIES,
    EVENT_KIND_SINGLE,
};
use super::event_database_track_loading::CEventDatabase;

/// opponents of temporary events, in turn (EventDatabaseEvents.cpp 0x00093160 switch, upper-cased by the lookup
/// OpponentDatabase.cpp 0x000E7D60)
pub const TEMP_EVENT_OPPONENTS: [&str; 5] = ["AL", "BUD", "JULIO", "ROBYN", "JAVIER"];

/// `VsStringAnsi::Hash` of an upper-cased name (OpponentDatabase.cpp 0x000E7D60)
fn name_crc(s: &str) -> u32 {
    let u = s.to_ascii_uppercase();
    if u.is_empty() { 0 } else { handy_crc::crc32(u.as_bytes(), 0) }
}

impl CEvent {
    /// EventDatabaseEvents.cpp 0x00092CD0 — lock / unlock, returns whether the state changed.
    pub fn set_unlocked(&mut self, unlocked: bool) -> bool {
        if (self.flags & EVENT_FLAG_LOCKED != 0) == !unlocked {
            return false;
        }
        if unlocked {
            self.flags &= !EVENT_FLAG_LOCKED;
        } else {
            self.flags |= EVENT_FLAG_LOCKED;
        }
        true
    }

    /// EventDatabaseEvents.cpp 0x000920E0 — back to the start of round 0.
    pub fn reset(&mut self, rand: &mut CrtRand) {
        self.flags &= !EVENT_FLAG_FAILED;
        self.flags &= !EVENT_FLAG_FINISHED;
        let Some(races) = self.races.as_mut() else { return };
        races.current_round = 0;
        races.previous_round = None;
        races.last_race = None;
        for round in &mut races.rounds {
            round.reset();
        }
        if let Some(r) = self.current_race_mut() {
            r.randomise_flags(rand);
        }
        if let Some(c) = self.competitors.as_mut() {
            for c in c {
                c.reset();
            }
        }
        self.change_and_set_up_round(0, rand);
    }

    /// EventDatabaseEvents.cpp 0x00092230 — the competitor with an opponent crc.
    pub fn find_competitor(&self, crc: u32) -> Option<usize> {
        let r = self.competitors.as_ref().and_then(|c| c.iter().position(|c| c.opponent_crc == crc));
        if r.is_none() {
            log::warn!("WARNING: Unable to match competitor CRC to event competitor list");
        }
        r
    }

    /// EventDatabaseEvents.cpp 0x000922B0 — "ChangeAndSetUpRound": fill the round's race slots (everyone for
    /// single / series events; the file's lists for a cup's first round, then the winners of the round before)
    /// and make it current. returns true when no human takes part in the round (the event is then failed).
    pub fn change_and_set_up_round(&mut self, round: u8, rand: &mut CrtRand) -> bool {
        let n_comp = self.competitor_count();
        let kind = self.kind;
        let per_cup = self.competitors_per_cup_race;
        let round_lists = self.round_competitors.clone();
        let comp_crcs: Vec<u32> = self.competitors.as_ref().map(|c| c.iter().map(|c| c.opponent_crc).collect()).unwrap_or_default();
        let Some(races) = self.races.as_mut() else { return false };
        if round as usize >= races.rounds.len() {
            log::error!("assert: u8RoundNumber < m_pEventRaces->m_u8NumRounds");
            return false;
        }
        let prev = races.current_round;
        let num_rounds = races.rounds.len();
        races.rounds[round as usize].reset();
        match kind {
            EVENT_KIND_SINGLE | EVENT_KIND_SERIES => {
                let r = &mut races.rounds[round as usize];
                if r.races.len() != 1 {
                    log::error!("assert: pNewRound->m_u8NumRacesInRound == 1");
                }
                if let Some(race) = r.races.first_mut() {
                    for i in 0..n_comp as usize {
                        if let Some(s) = race.competitors.get_mut(i) {
                            *s = Some(i);
                        }
                    }
                }
            }
            EVENT_KIND_CUP if round == 0 => {
                let mut count = 0u32;
                let r = &mut races.rounds[0];
                for (b, race) in r.races.iter_mut().enumerate() {
                    for l in 0..race.num_competitors as usize {
                        let crc = round_lists.as_ref().and_then(|v| v.get(b)).and_then(|v| v.get(l)).copied().unwrap_or(0);
                        let found = comp_crcs.iter().position(|&c| c == crc);
                        if found.is_none() {
                            log::warn!("WARNING: Unable to match competitor CRC to event competitor list");
                        }
                        race.competitors[l] = found;
                        count += 1;
                        if count > n_comp as u32 {
                            log::error!("assert: u8CompetitorCount <= m_pEventCompetitors->m_u8NumCompetitors");
                        }
                    }
                }
            }
            EVENT_KIND_CUP => {
                let winners = |rnd: &CEventRound, race: usize, pos: u8| rnd.races.get(race).and_then(|r| r.competitor_in_position(pos));
                let p = races.rounds[prev as usize].clone();
                let new = &mut races.rounds[round as usize];
                let mut set = |race: usize, slot: usize, v: Option<usize>| {
                    if let Some(s) = new.races.get_mut(race).and_then(|r| r.competitors.get_mut(slot)) {
                        *s = v;
                    }
                };
                // the two winners of races a and b meet in race `into`
                let mut pair = |into: usize, a: usize, b: usize| {
                    set(into, 0, winners(&p, a, 1));
                    set(into, 1, winners(&p, b, 1));
                };
                match num_rounds {
                    2 => {
                        if round == 1 && per_cup == 2 {
                            pair(0, 0, 1);
                        } else if round == 1 && per_cup == 4 {
                            set(0, 0, winners(&p, 0, 1));
                            set(0, 1, winners(&p, 0, 2));
                            set(0, 2, winners(&p, 1, 1));
                            set(0, 3, winners(&p, 1, 2));
                            log::error!("assert: EventDatabaseEvents.cpp 586");
                        } else {
                            log::error!("assert: EventDatabaseEvents.cpp 591");
                        }
                    }
                    3 => {
                        if per_cup != 2 {
                            log::error!("assert: m_u8NumCompetitorsPerCupRace == 2");
                        }
                        match round {
                            2 => pair(0, 0, 1),
                            1 => {
                                pair(0, 0, 1);
                                pair(1, 2, 3);
                            }
                            _ => log::error!("assert: EventDatabaseEvents.cpp 611"),
                        }
                    }
                    4 => {
                        if per_cup != 2 {
                            log::error!("assert: m_u8NumCompetitorsPerCupRace == 2");
                        }
                        match round {
                            3 => pair(0, 0, 1),
                            2 => {
                                pair(0, 0, 1);
                                pair(1, 2, 3);
                            }
                            1 => {
                                pair(0, 0, 1);
                                pair(1, 2, 3);
                                pair(2, 4, 5);
                                pair(3, 6, 7);
                                log::error!("assert: EventDatabaseEvents.cpp 645");
                            }
                            _ => log::error!("assert: EventDatabaseEvents.cpp 645"),
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                log::error!("ChangeAndSetUpRound encountered unrecognised event");
            }
        }
        races.current_round = round;
        races.previous_round = Some(prev);
        let has_human = {
            let humans: Vec<bool> = self.competitors.as_ref().map(|c| c.iter().map(|c| c.human != 0).collect()).unwrap_or_default();
            let races = self.races.as_mut().unwrap();
            let r = &mut races.rounds[round as usize];
            let cur = r.current_race as usize;
            if let Some(race) = r.races.get_mut(cur) {
                race.randomise_flags(rand);
            }
            r.has_human(&humans)
        };
        if has_human {
            return false;
        }
        self.flags |= EVENT_FLAG_FAILED;
        true
    }

    /// EventDatabaseEvents.cpp 0x00092D40 — completed: unlock what depends on it (announcing the unlock
    /// message). the original then unhides vehicles (CVehicleDatabase::UnhideVehicles 0x00184E60).
    pub fn complete(db: &mut CEventDatabase, event: usize, career: &dyn super::event_database::CareerQuery) -> Option<u32> {
        db.events[event].flags |= EVENT_FLAG_COMPLETED;
        db.evaluate_unlocks(career, true)
    }
}

impl CEventRound {
    /// EventDatabaseEvents.cpp 0x00091FC0 — first race current, positions and slots cleared.
    pub fn reset(&mut self) {
        self.current_race = 0;
        for r in &mut self.races {
            r.finish_positions.iter_mut().for_each(|p| *p = 0);
            r.competitors.iter_mut().for_each(|c| *c = None);
        }
    }

    /// EventDatabaseEvents.cpp 0x00092B50 — does any race of the round seat a human.
    pub fn has_human(&self, humans: &[bool]) -> bool {
        self.races
            .iter()
            .any(|r| r.competitors.iter().any(|c| c.is_some_and(|i| humans.get(i).copied().unwrap_or(false))))
    }
}

impl CEventRaceEntry {
    /// EventDatabaseEvents.cpp 0x00091CC0 — the competitor that finished in `pos` (1-based).
    pub fn competitor_in_position(&self, pos: u8) -> Option<usize> {
        if pos == 0 || pos as usize >= self.num_competitors as usize + 1 {
            log::error!("assert: u8FinishPos < m_u8NumCompetitorsInRace + 1");
        }
        let i = self.finish_positions.iter().position(|&p| p == pos)?;
        self.competitors.get(i).copied().flatten()
    }

    /// EventDatabaseBinLoading.cpp 0x000913B0 — flag 4 picks bit 0 at random.
    pub fn randomise_flags(&mut self, rand: &mut CrtRand) {
        if self.flags & super::event_database::RACE_ENTRY_FLAG_RANDOM_BIT0 != 0 {
            // VsRandUnit is a float return: rounded to f32 before the compare
            if (rand.rand_unit() as f32) <= 0.5 {
                self.flags &= !1;
            } else {
                self.flags |= 1;
            }
        }
    }

    /// EventDatabaseBinLoading.cpp 0x00091B70 — humans seated in this race.
    pub fn human_count(&self, comps: &[CEventCompetitor]) -> u8 {
        self.competitors.iter().filter(|c| c.is_some_and(|i| comps.get(i).is_some_and(|c| c.human != 0))).count() as u8
    }
}

impl CEventCompetitor {
    /// EventDatabaseEvents.cpp 0x00092070 — points cleared; the opponent is looked up again by crc (the ai
    /// crate's database: `opponent_found` is left to it).
    pub fn reset(&mut self) {
        self.points = 0;
        if self.opponent_crc == 0 {
            self.opponent_found = false;
        }
    }
}

impl CEventDatabase {
    /// EventDatabaseEvents.cpp 0x00092E80 — insert before the first event that sorts at or after it by
    /// (category, tier, order), else at the end.
    pub fn insert_event(&mut self, e: CEvent) {
        let at = self
            .events
            .iter()
            .position(|o| e.category <= o.category && e.tier <= o.tier && e.order <= o.order);
        match at {
            Some(i) => self.events.insert(i, e),
            None => self.events.push(e),
        }
        self.num_events = self.num_events.wrapping_add(1);
    }

    /// EventDatabaseEvents.cpp 0x00092D90 (and 0x00093080) — event by crc.
    pub fn find_event(&self, crc: u32) -> Option<usize> {
        self.events.iter().position(|e| e.crc == crc)
    }

    /// EventDatabaseEvents.cpp 0x00092E40 — event by name (upper-cased, hashed).
    pub fn find_event_by_name(&self, name: &str) -> Option<usize> {
        self.find_event(name_crc(name))
    }
}

/// race of a temporary event: the database race and its track (`CEventRace+0x34`).
#[derive(Clone, Copy, Debug)]
pub struct TempRace {
    pub race: usize,
    pub track: usize,
}

/// EventDatabaseEvents.cpp 0x00093160 — the shell of a temporary event: humans first, then opponents named in
/// turn from `TEMP_EVENT_OPPONENTS`. (the original also flags a random opponent of groups 0..9 per ai slot in
/// the opponent database before overwriting the pick with the named one; that pass is the ai crate's data and
/// changes nothing in the event)
fn build_temp_event(underground: bool, humans: u8, opponents: u8) -> CEvent {
    let mut e = CEvent { flags: EVENT_FLAG_TEMP, category: if underground { 1 } else { 2 }, ..Default::default() };
    let count = humans.wrapping_add(opponents);
    let mut list = Vec::with_capacity(count as usize);
    let mut k = 0usize;
    for i in 0..count {
        if i >= humans {
            let crc = name_crc(TEMP_EVENT_OPPONENTS[k]);
            k = (k + 1) % 5;
            list.push(CEventCompetitor { opponent_crc: crc, ..Default::default() });
        } else {
            list.push(CEventCompetitor { human: 1, ..Default::default() });
        }
    }
    e.competitors = Some(list);
    // 0x00093F30 then position 1
    e.result = Some(CEventResult { position: 1, points: 0xFFFF_FFFF });
    e
}

/// builder argument 12 / 13 to the event tier (0x00093630 switch)
fn temp_tier(t: u32) -> u8 {
    match t {
        6 => 0,
        7 | 8 => 2,
        _ => t as u8,
    }
}

fn temp_race(r: TempRace, race_type: u32, laps: u8, n: u8, flag1: bool, flag2: bool) -> CEventRaceEntry {
    let mut flags = 0;
    if flag1 {
        flags |= 1;
    }
    if flag2 {
        flags |= 2;
    }
    CEventRaceEntry {
        race_type,
        track: Some(r.track),
        race: Some(r.race),
        laps,
        num_competitors: n,
        flags,
        finish_positions: vec![0; n as usize],
        competitors: vec![None; n as usize],
        ..Default::default()
    }
}

/// arguments of the temporary event builders (`CFEQuickRaceStartScreen::OnSelect`, `CGameJump::Execute`,
/// `NetworkGame` use the `GameVars` quick race globals).
#[derive(Clone, Debug, Default)]
pub struct TempEventArgs {
    /// `byte_6F70E9 == 0`
    pub underground: bool,
    pub area: u32,
    /// `dword_6F7140`
    pub race_type: u32,
    pub laps: u8,
    pub humans: u8,
    pub opponents: u8,
    /// race entry flag bits 0 / 1 (`byte_6F7154 != 0`, `byte_6F7155`)
    pub flag1: bool,
    pub flag2: bool,
    /// `dword_6F7138` (the jump passes 1)
    pub tier: u32,
}

/// EventDatabaseEvents.cpp 0x00093630 — "BuildTempSingleRaceEvent": one race.
pub fn build_temp_single_race_event(a: &TempEventArgs, race: TempRace, rand: &mut CrtRand) -> CEvent {
    let mut e = build_temp_event(a.underground, a.humans, a.opponents);
    e.kind = EVENT_KIND_SINGLE;
    e.unknown_08 = 0;
    e.area = a.area;
    e.tier = temp_tier(a.tier);
    let n = e.competitor_count();
    e.races = Some(CEventRaces {
        num_competitor_rounds: 1,
        current_round: 0,
        rounds: vec![CEventRound { current_race: 0, races: vec![temp_race(race, a.race_type, a.laps, n, a.flag1, a.flag2)] }],
        ..Default::default()
    });
    e.reset(rand);
    e
}

/// EventDatabaseEvents.cpp 0x00093920 — a series of `rounds` single-race rounds (the quick race tournament).
/// (the original never sets a round's race count before sizing it; one race per round is what it then writes)
pub fn build_temp_series_event(a: &TempEventArgs, races: &[TempRace], rounds: u8, rand: &mut CrtRand) -> CEvent {
    let mut e = build_temp_event(a.underground, a.humans, a.opponents);
    e.kind = EVENT_KIND_SERIES;
    e.unknown_08 = 0;
    e.area = a.area;
    e.tier = temp_tier(a.tier);
    let n = e.competitor_count();
    let mut r = CEventRaces { num_competitor_rounds: rounds, current_round: 0, ..Default::default() };
    for i in 0..rounds as usize {
        let Some(&tr) = races.get(i) else {
            log::error!("temp series event: no race for round {i}");
            break;
        };
        r.rounds.push(CEventRound { current_race: 0, races: vec![temp_race(tr, a.race_type, a.laps, n, a.flag1, a.flag2)] });
    }
    e.races = Some(r);
    e.reset(rand);
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_event_competitors() {
        let a = TempEventArgs { underground: true, race_type: 3, laps: 3, humans: 1, opponents: 3, tier: 1, ..Default::default() };
        let mut rand = CrtRand::default();
        let e = build_temp_single_race_event(&a, TempRace { race: 0, track: 0 }, &mut rand);
        let c = e.competitors.as_ref().unwrap();
        assert_eq!(c.len(), 4);
        assert_eq!(c[0].human, 1);
        assert_eq!(c[1].opponent_crc, handy_crc::crc32(b"AL", 0));
        assert_eq!(c[3].opponent_crc, handy_crc::crc32(b"JULIO", 0));
        let race = e.current_race().unwrap();
        assert_eq!(race.competitors, vec![Some(0), Some(1), Some(2), Some(3)]);
        assert_eq!(e.flags & EVENT_FLAG_FAILED, 0);
        let s = build_temp_series_event(&a, &[TempRace { race: 0, track: 0 }; 3], 3, &mut rand);
        assert_eq!(s.races.as_ref().unwrap().rounds.len(), 3);
        let _ = (EVENT_KIND_CUP, EVENT_FLAG_COMPLETED);
    }
}
