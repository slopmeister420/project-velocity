//! src\GameMode\GameVars.cpp — `CGameJump` (debug "jump straight into a race") and the race selection it builds.

use bevy::prelude::Resource;

use crate::event_database::event_database::CEvent;
use crate::event_database::event_database_events::{
    build_temp_series_event, build_temp_single_race_event, TempEventArgs, TempRace,
};
use crate::event_database::event_database_track_loading::{CEventDatabase, ERaceType};
use visos::handy::crt_rand::CrtRand;

/// `CGameJump` (0x205 bytes at byte_6F9E60)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CGameJump {
    pub active: bool,       // +0x000
    pub track: String,      // +0x001 "TRACKS\LACOAST\"
    pub event: String,      // +0x101 "CIRCUIT\RACE01\"
    pub professional: bool, // +0x201
    pub wet: bool,          // +0x202
    pub drag: bool,         // +0x203
    pub splitscreen: bool,  // +0x204
}

/// the race the game is about to load (what `g_FEGameVariables.m_pEvent` points at for the loaders).
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct RaceSelection {
    pub track: String,      // "LACOAST"
    pub race_type: ERaceType,
    pub race: String,       // "RACE01"
    /// CEventTrack+0x78 "Tracks\LACOAST\"
    pub track_path: String,
    /// CEventRace+0x10 "CIRCUIT\RACE01\"
    pub race_path: String,
    pub professional: bool,
    pub wet: bool,
    pub splitscreen: bool,
    /// players (dword_6F5C00)
    pub num_players: i32,
    /// traffic enabled (byte_6F70E8)
    pub traffic: bool,
    /// event build arguments of 0x00093630
    pub laps_flag: bool,
    pub opponents_mode: u8,
    /// index into `CEventDatabase::races`
    pub event_race: Option<usize>,
}

impl CGameJump {
    /// GameVars.cpp 0x000C2780 — Set
    pub fn set(&mut self, track: &str, event: &str, professional: bool, wet: bool, drag: bool) {
        self.active = true;
        self.track = track.to_ascii_lowercase(); // _strlwr (0x003E60E0)
        self.event = event.to_ascii_lowercase();
        self.professional = professional;
        self.wet = wet;
        self.drag = drag;
        // GameVars.cpp 0x000C2EC0 clears split screen whatever is passed, so `splitscreen` never takes (kept)
        self.splitscreen = false;
    }

    /// GameVars.cpp 0x000C2860 — resolve the jump against the event database. the original then builds a
    /// one-race event (EventDatabaseEvents.cpp 0x00093630) and goes to the "LoadingScreen" front end screen.
    pub fn execute(&mut self, db: &CEventDatabase) -> Option<RaceSelection> {
        let path = format!("\\{}{}", self.track, self.event);
        let parts: Vec<String> = path.split('\\').filter(|s| !s.is_empty()).map(|s| s.to_ascii_uppercase()).collect();
        // token 0 is "TRACKS"
        let track = parts.get(1).cloned().unwrap_or_default();
        let race_type = parts.get(2).cloned().unwrap_or_default();
        let race = parts.get(3).cloned().unwrap_or_default();

        let mut sel = RaceSelection {
            professional: self.professional,
            wet: self.wet,
            splitscreen: self.splitscreen,
            ..Default::default()
        };
        let mut laps_flag = false;
        let mut opponents_mode: u8;
        if self.splitscreen {
            sel.num_players = 2;
            opponents_mode = 1;
        } else {
            sel.num_players = 1;
            opponents_mode = 3;
        }
        if self.professional {
            sel.traffic = false;
        } else {
            sel.traffic = true;
            let l = self.track.to_ascii_lowercase();
            if !l.contains("la") && !l.contains("london") && !l.contains("tokyo") {
                sel.traffic = false;
                log::warn!("TRAFFIC DISABLED - NO VALID TRACK SELECTED!");
            }
        }
        let rt = CEventDatabase::race_type_from_name(&race_type);
        if matches!(rt, ERaceType::Sprint | ERaceType::Pursuit) {
            laps_flag = true;
        }
        if matches!(rt, ERaceType::Cruise | ERaceType::Pursuit) {
            opponents_mode = 0;
        }
        if rt == ERaceType::Drag {
            laps_flag = true;
            opponents_mode = 3;
            self.drag = true;
        }
        log::info!(
            "Jumping to {}{} {} {} {}",
            self.track,
            self.event,
            if self.splitscreen { "SplitScreen" } else { "" },
            if self.wet { "Wet" } else { "Fair" },
            if self.professional { "Professional" } else { "Underground" }
        );
        let t = db.find_track(0, &track)?;
        let r = db.find_race(t, 0, rt, &race)?;
        sel.track = track;
        sel.race_type = rt;
        sel.race = race;
        sel.track_path = db.tracks[t].path.clone();
        sel.race_path = db.races[r].path.clone();
        sel.laps_flag = laps_flag;
        sel.opponents_mode = opponents_mode;
        sel.event_race = Some(r);
        // GameVars.cpp 0x000C27F0
        self.active = false;
        Some(sel)
    }
}

impl RaceSelection {
    /// CGameJump::Execute 0x000C2C50 — the one-race event it builds (EventDatabaseEvents.cpp 0x00093630).
    pub fn build_jump_event(&self, db: &CEventDatabase, drag: bool, rand: &mut CrtRand) -> Option<CEvent> {
        let r = self.event_race?;
        let a = TempEventArgs {
            underground: !self.professional,
            area: 0,
            race_type: self.race_type as u32,
            laps: if self.laps_flag { 1 } else { 3 },
            humans: 1,
            opponents: self.opponents_mode,
            flag1: self.wet,
            flag2: drag,
            tier: 1,
        };
        Some(build_temp_single_race_event(&a, TempRace { race: r, track: db.races[r].track }, rand))
    }
}

/// the event part of `CFEQuickRaceStartScreen::OnSelect` (0x00221F00): the quick race globals the front end
/// holds, turned into a temporary event.
#[derive(Clone, Debug, Default)]
pub struct QuickRaceEvent {
    /// `byte_6F7156`: one race (else a three-round series)
    pub single: bool,
    /// `dword_6F7148[3]` races (indices into `CEventDatabase::races`)
    pub races: [Option<usize>; 3],
    pub professional: bool,
    pub race_type: u32,
    /// `dword_6F7134`
    pub laps: u8,
    /// `dword_6F713C`
    pub opponents: u8,
    /// `byte_6F7157` two players
    pub two_players: bool,
    /// `byte_6F7154 != 0`, `byte_6F7155`
    pub flag1: bool,
    pub flag2: bool,
    /// `dword_6F7138`
    pub tier: u32,
}

impl QuickRaceEvent {
    pub fn build(&self, db: &CEventDatabase, rand: &mut CrtRand) -> Option<CEvent> {
        let tr = |r: Option<usize>| r.map(|r| TempRace { race: r, track: db.races[r].track });
        let mut a = TempEventArgs {
            underground: !self.professional,
            area: 0,
            race_type: self.race_type,
            laps: self.laps,
            humans: 1,
            opponents: self.opponents,
            flag1: self.flag1,
            flag2: self.flag2,
            tier: self.tier,
        };
        if self.single {
            a.humans = self.two_players as u8 + 1;
            Some(build_temp_single_race_event(&a, tr(self.races[0])?, rand))
        } else {
            let races: Vec<TempRace> = self.races.iter().map(|&r| tr(r)).collect::<Option<_>>()?;
            Some(build_temp_series_event(&a, &races, 3, rand))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jump_to_lacoast() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let mut db = CEventDatabase::default();
        db.load_tracks("\\FE");
        let mut j = CGameJump::default();
        j.set("TRACKS\\LACOAST\\", "CIRCUIT\\RACE01\\", false, false, false);
        let s = j.execute(&db).unwrap();
        assert_eq!(s.track_path, "Tracks\\LACOAST\\");
        assert_eq!(s.race_path, "CIRCUIT\\RACE01\\");
        assert!(s.traffic);
        assert!(!j.active);
        // one human, three named opponents, one 3-lap race
        let e = s.build_jump_event(&db, j.drag, &mut CrtRand::default()).unwrap();
        assert_eq!(e.competitor_count(), 4);
        assert_eq!(e.current_race().unwrap().laps, 3);
        assert_eq!(e.current_race().unwrap().race, s.event_race);
    }
}
