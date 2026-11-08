//! src\EventDatabase\EventDatabaseBinLoading.cpp — besides the file reader (`event_database::read_event_file`,
//! 0x00090E60) this object holds the race-to-race progression of an event: points, advancing races and rounds,
//! and the end-of-event result with its prize money.

use visos::handy::crt_rand::CrtRand;

use super::event_database::{
    CEvent, EVENT_FLAG_FAILED, EVENT_FLAG_FINISHED, EVENT_KIND_CUP, EVENT_KIND_SERIES, EVENT_KIND_SINGLE,
};

/// CompartmentManager__8E9C0 (idb name; EventDatabase code) — series points for a finish position.
pub fn points_for_position(pos: u8) -> u32 {
    match pos {
        1 => 10,
        2 => 6,
        3 => 5,
        4 => 4,
        5 => 3,
        6 => 2,
        _ => 0,
    }
}

/// CompartmentManager__8EA60 (idb name) — share of the prize pool for a 0-based place among 4 or 6
/// competitors; returned on the x87 stack as `pool * share`.
pub fn prize_money(pool: f32, place: u8, competitors: u8, category: u32) -> f64 {
    const FOUR: [u32; 4] = [0x3ECC_CCCD, 0x3E99_999A, 0x3E4C_CCCD, 0x3DCC_CCCD];
    const SIX: [u32; 6] = [0x3EB3_3333, 0x3E80_0000, 0x3DF5_C28F, 0x3DA3_D70A, 0x3D4C_CCCD, 0];
    if place >= competitors {
        log::warn!("Finish pos higher than max competitors when calculating cash prize. Setting to 0");
        return 0.0;
    }
    let share = if category == 0 || category > 3 {
        log::warn!("Unknown event catagory when calculating prize winnings");
        0.0
    } else if competitors == 4 {
        f32::from_bits(FOUR[place as usize])
    } else if competitors == 6 {
        f32::from_bits(SIX[place as usize])
    } else {
        log::warn!("Unsupported number of competitors when calculating prize winnings");
        0.0
    };
    pool as f64 * share as f64
}

/// what UpdateAtEndOfEvent hands to the player's career (globals of `g_FEGameVariables` in the original).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EventOutcome {
    pub success: bool,
    /// 0-based place used for the prize
    pub place: u8,
    /// added to the cash (`dword_6F719C`): `(int)prize` then +/- the wager
    pub cash: i32,
    /// added to `dword_6F71C8` on success
    pub prestige_reward: f32,
    /// team to unlock on success (TeamDatabase: set flags 1, 0x20, 2 when still locked)
    pub unlock_team: Option<u32>,
}

impl CEvent {
    /// EventDatabaseBinLoading.cpp 0x00091410 — single / series: add points by finish, next race of the round.
    /// returns true when the round is over.
    fn series_race_finished(&mut self, rand: &mut CrtRand) -> bool {
        let pos: Vec<u8> = self.current_race().map(|r| r.finish_positions.clone()).unwrap_or_default();
        if let Some(c) = self.competitors.as_mut() {
            for (i, c) in c.iter_mut().enumerate() {
                c.points = c.points.wrapping_add(points_for_position(pos.get(i).copied().unwrap_or(0)));
            }
        }
        self.next_race_in_round(rand)
    }

    fn next_race_in_round(&mut self, rand: &mut CrtRand) -> bool {
        let Some(races) = self.races.as_mut() else { return true };
        let round = &mut races.rounds[races.current_round as usize];
        round.current_race = round.current_race.wrapping_add(1);
        if (round.current_race as usize) < round.races.len() {
            let i = round.current_race as usize;
            round.races[i].randomise_flags(rand);
            false
        } else {
            true
        }
    }

    /// EventDatabaseBinLoading.cpp 0x00091520 — cup: next race of the round; races without a human are
    /// decided at once in slot order. returns true when the round is over.
    fn cup_race_finished(&mut self, rand: &mut CrtRand) -> bool {
        let comps = self.competitors.clone().unwrap_or_default();
        loop {
            if self.next_race_in_round(rand) {
                return true;
            }
            let race = self.current_race_mut().unwrap();
            if race.human_count(&comps) != 0 {
                return false;
            }
            for k in 0..race.num_competitors as usize {
                race.finish_positions[k] = (k + 1) as u8;
            }
        }
    }

    /// EventDatabaseBinLoading.cpp 0x00091620 — the current race is over (finish positions filled in): returns
    /// whether its round is over. the original then adds `raceline length * laps` to the player vehicle's
    /// mileage (VehicleDatabase 0x001850B0), the caller's job: `laps` of the finished race is returned with it.
    pub fn race_finished(&mut self, rand: &mut CrtRand) -> (bool, u8) {
        let cur = self.races.as_ref().map(|r| (r.current_round, r.rounds[r.current_round as usize].current_race));
        let laps = self.current_race().map(|r| r.laps).unwrap_or(0);
        let done = match self.kind {
            EVENT_KIND_SINGLE | EVENT_KIND_SERIES => self.series_race_finished(rand),
            EVENT_KIND_CUP => self.cup_race_finished(rand),
            _ => {
                log::warn!("Unsupported Event type (for now)");
                false
            }
        };
        if let Some(r) = self.races.as_mut() {
            r.last_race = cur;
        }
        (done, laps)
    }

    /// EventDatabaseBinLoading.cpp 0x00091810 (0x00091730 / 0x000917A0) — next round: returns true when the
    /// event is over (no more rounds, or no human left in the new one).
    pub fn round_finished(&mut self, rand: &mut CrtRand) -> bool {
        if !matches!(self.kind, EVENT_KIND_SINGLE | EVENT_KIND_SERIES | EVENT_KIND_CUP) {
            log::warn!("Unsupported Event type (for now)");
            return false;
        }
        let Some(r) = self.races.as_mut() else { return true };
        let next = r.current_round as usize + 1;
        if next < r.rounds.len() {
            self.change_and_set_up_round(next as u8, rand)
        } else {
            // past the last round
            r.current_round = r.current_round.wrapping_add(1);
            true
        }
    }

    /// EventDatabaseBinLoading.cpp 0x00091890 — "UpdateAtEndOfEvent": did the player (competitor 0) meet the
    /// result, and the prize. the event's completion (0x00092D40) is the caller's when `success`.
    pub fn update_at_end_of_event(&mut self) -> EventOutcome {
        let mut success = true;
        let mut place: u8 = 0;
        self.flags |= EVENT_FLAG_FINISHED;
        let req = self.result.unwrap_or_default();
        let comps = self.competitors.clone().unwrap_or_default();
        let last_round = self.races.as_ref().and_then(|r| r.rounds.last()).cloned().unwrap_or_default();
        let first_pos = last_round.races.first().and_then(|r| r.finish_positions.first().copied()).unwrap_or(0);
        match self.kind {
            k if k == EVENT_KIND_SINGLE || k == EVENT_KIND_CUP => {
                let need = if req.position == 0 { 1 } else { req.position };
                let humans: Vec<bool> = comps.iter().map(|c| c.human != 0).collect();
                if last_round.has_human(&humans) {
                    if first_pos > need {
                        success = false;
                    }
                } else {
                    success = false;
                }
                place = first_pos.wrapping_sub(1);
            }
            EVENT_KIND_SERIES => {
                let lead = comps.first().map(|c| c.points).unwrap_or(0);
                if req.points != 0 && req.points > lead {
                    success = false;
                }
                for c in comps.iter().skip(1) {
                    if c.points > lead {
                        place = place.wrapping_add(1);
                    }
                }
                if req.position != 0 && (req.position as i8 as i32) <= place as i32 {
                    success = false;
                }
            }
            _ => {
                log::error!("UpdateAtEndOfEvent encountered unrecognised event");
            }
        }
        let mut out = EventOutcome { success, place, ..Default::default() };
        if success {
            if self.unlock_team != 0 {
                out.unlock_team = Some(self.unlock_team);
            }
            out.prestige_reward = self.prestige_reward as f32;
        } else {
            self.flags |= EVENT_FLAG_FAILED;
        }
        // the prize is paid whatever the result (ftol of the x87 product)
        out.cash = prize_money(self.prize, place, comps.len() as u8, self.category) as i32;
        if let Some(w) = comps.first().map(|c| c.wager).filter(|&w| w != 0) {
            if first_pos == 1 {
                out.cash = out.cash.wrapping_add(w);
            } else {
                out.cash = out.cash.wrapping_sub(w);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_database::event_database_events::{build_temp_series_event, TempEventArgs, TempRace};

    #[test]
    fn series_points_and_rounds() {
        let a = TempEventArgs { underground: true, race_type: 3, laps: 2, humans: 1, opponents: 3, tier: 1, ..Default::default() };
        let mut rand = CrtRand::default();
        let mut e = build_temp_series_event(&a, &[TempRace { race: 0, track: 0 }; 3], 3, &mut rand);
        for round in 0..3 {
            e.current_race_mut().unwrap().finish_positions = vec![1, 2, 3, 4];
            let (round_done, laps) = e.race_finished(&mut rand);
            assert!(round_done);
            assert_eq!(laps, 2);
            assert_eq!(e.round_finished(&mut rand), round == 2);
        }
        assert_eq!(e.competitors.as_ref().unwrap()[0].points, 30);
        // temp events require points <= 0xFFFFFFFF (0x00093F30), so a series always reads as failed (kept)
        let o = e.update_at_end_of_event();
        assert!(!o.success);
        assert_eq!(o.place, 0);
        assert_eq!(prize_money(1000.0, 0, 4, 1) as i32, 400);
    }
}
