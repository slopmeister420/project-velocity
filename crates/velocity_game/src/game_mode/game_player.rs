//! src\GameMode\GamePlayer.cpp — `g_FEGameVariables` (0x006F6F70): the career and race-setup globals the front end
//! and the race share, set up by `SetInitialVariables` (0x000C1930). Only the fields game flow uses are kept;
//! the quick race selections live in the front end's copy (`QuickRaceSettings`).

use velocity_physics::vehicle_database::vehicle_database::{CVehicleDatabase, VehicleSetup};
use visos::handy::handy_crc;

use crate::event_database::event_database::{CEvent, CareerQuery, EVENT_FLAG_LOCKED, EVENT_FLAG_TEMP};
use crate::event_database::event_database_track_loading::CEventDatabase;

/// what `dword_6F9BE8` points at: a database event, or a temporary one (`CEvent+0x15 & 8`) the game owns.
#[derive(Clone, Debug, PartialEq)]
pub enum EventRef {
    Database(usize),
    Temp(Box<CEvent>),
}

/// garage slots per player (`m_PlayerInfoA[].mNumCarSlots` < 6)
pub const GARAGE_SLOTS: usize = 6;

fn crc(s: &str) -> u32 {
    handy_crc::crc32(s.as_bytes(), 0)
}

#[derive(Clone, Debug)]
pub struct CFEGameVariables {
    /// +0x16C `dword_6F70DC` selected vehicle crc
    pub selected_vehicle: u32,
    /// +0x178 `byte_6F70E8` traffic
    pub traffic: bool,
    /// +0x179 `byte_6F70E9` professional
    pub professional: bool,
    /// +0x180 `byte_6F70F0` flag girl intro index (`\Timeline\FlagGirl.vcd` entry, modulo the count)
    pub flag_girl: u8,
    /// +0x1E8 `byte_6F7158` network race (set by the online session, NetworkLogger 0x000DE560)
    pub network_race: bool,
    /// +0x22C `dword_6F719C` cash
    pub cash: i32,
    /// +0x230 `dword_6F71A0` prestige
    pub prestige: i32,
    /// +0x234 count, +0x238 `dword_6F71A8` garage vehicle crcs
    pub garage: Vec<u32>,
    /// +0x250 `dword_6F71C0` current garage vehicle, +0x254 `dword_6F71C4` its slot
    pub garage_vehicle: u32,
    pub garage_slot: u32,
    /// +0x258 `dword_6F71C8` reputation earned from events
    pub reward: f32,
    /// +0x25C `dword_6F71CC`
    pub unknown_25c: i32,
    /// +0x260 `dword_6F71D0` the player's team (professional events race its car, FEHangoutProgression 0x0028F430)
    pub team: u32,
    /// +0x264..+0x27F rest of the 84-byte block the save copies (`dword_6F719C`..): [0] = +0x264 `dword_6F71D4`
    /// `g_TeamsJoinedCount` (use `teams_joined` / `teams_joined_mut`), [1..] not decoded
    pub unknown_264: [u32; 7],
    /// +0x204 `unk_6F7174` profile name (15 chars + nul)
    pub profile_name: String,
    /// +0x734 `dword_6F76A4` one 556-byte setup per garage slot (GameVars.cpp 0x000C1F90 / 0x000C20F0)
    pub garage_setups: Vec<VehicleSetup>,
    /// +0x2D4 `dword_6F7244` cleared on every event reset
    pub event_state: u32,
    /// +0x2D8 `dword_6F7248` message of the event unlocked last
    pub unlock_message: u32,
    /// +0x21EE `byte_6F915E` in-race speech
    pub speech: bool,
    /// +0x21EF `byte_6F915F` opponent name tags over the cars
    pub player_names: bool,
    /// +0x20E4 `byte_6F9054` a profile is in use: RaceCompleteScreen saves it (0x00205830)
    pub save_enabled: bool,
    /// +0x20E5 `byte_6F9055` profile created / loaded this session (read by the hangout)
    pub profile_active: bool,
    /// +0x20E8 `byte_6F9058` at least one event completed (SaveFileData 0x00104E30, hangout tutorial)
    pub completed_any: bool,
    /// +0x2C74 `byte_6F9BE4..` unlocked tier count per category 1..3 (0x000C1DA0)
    pub unlocked_tiers: [u8; 3],
    /// +0x2C78 `dword_6F9BE8` the event being raced
    pub event: Option<EventRef>,
}

impl Default for CFEGameVariables {
    /// the global as the game has it after boot: constructed, then SetInitialVariables
    fn default() -> Self {
        let mut v = Self::new();
        v.set_initial_variables();
        v
    }
}

impl CFEGameVariables {
    /// GamePlayer.cpp 0x000C1480
    /// the constructor: every field zeroed (the kept ones here); its default-constructed setups at +0x2DC / +0x734
    /// are `garage_setups`, filled by FECK2Manager 0x002022F0
    pub fn new() -> Self {
        Self {
            selected_vehicle: 0,
            traffic: false,
            professional: false,
            flag_girl: 0,
            network_race: false,
            cash: 0,
            prestige: 0,
            garage: Vec::new(),
            garage_vehicle: 0,
            garage_slot: 0,
            reward: 0.0,
            unknown_25c: 0,
            team: 0,
            unknown_264: [0; 7],
            profile_name: String::new(),
            garage_setups: Vec::new(),
            save_enabled: false,
            profile_active: false,
            completed_any: false,
            event_state: 0,
            unlock_message: 0,
            speech: false,
            player_names: false,
            unlocked_tiers: [0; 3],
            event: None,
        }
    }
}

impl CFEGameVariables {
    /// +0x264 `dword_6F71D4` (`g_TeamsJoinedCount`): teams joined with the bonus, `++` by FEHangoutProgression
    /// 0x00290BC0's first join, zeroed by SetInitialVariables, otherwise only saved
    pub fn teams_joined(&self) -> u32 {
        self.unknown_264[0]
    }

    pub fn teams_joined_mut(&mut self) -> &mut u32 {
        &mut self.unknown_264[0]
    }

    /// +0x268 `dword_6F71D8`: TipsScreen "movie seen" bits (bit `tip - 1`; FETipsScreen 0x002400D0 / 0x00240310)
    pub fn tips_watched(&self) -> u32 {
        self.unknown_264[1]
    }

    pub fn tips_watched_mut(&mut self) -> &mut u32 {
        &mut self.unknown_264[1]
    }

    /// +0x26C `dword_6F71DC`: licence plate style (PlateCustomizationScreen, LicencePlateRender 0x0012DC60)
    pub fn plate_style(&self) -> u32 {
        self.unknown_264[2]
    }

    pub fn plate_style_mut(&mut self) -> &mut u32 {
        &mut self.unknown_264[2]
    }

    /// +0x270 `unk_6F71E0`: licence plate text, an inline wide string (MEM_Alloc 0x002C1850 widens each byte) in
    /// the last 16 bytes of the saved vars block, so it is saved with the profile. 8 characters fill it and the
    /// nul lands on `dword_6F71F0` (outside the block, original overflow); here it simply ends at 8.
    pub fn plate_text(&self) -> String {
        let mut s = String::new();
        for w in &self.unknown_264[3..7] {
            for c in [*w as u16, (*w >> 16) as u16] {
                if c == 0 {
                    return s;
                }
                s.push(char::from_u32(c as u32).unwrap_or('?'));
            }
        }
        s
    }

    /// MEM_Alloc 0x002C1850 into `unk_6F71E0`: each byte widened (`(char)` sign extension kept), at most 8 here
    pub fn set_plate_text(&mut self, text: &str) {
        let mut w = [0u16; 8];
        for (d, b) in w.iter_mut().zip(text.bytes()) {
            *d = b as i8 as i16 as u16;
        }
        for (i, dst) in self.unknown_264[3..7].iter_mut().enumerate() {
            *dst = w[2 * i] as u32 | (w[2 * i + 1] as u32) << 16;
        }
    }

    /// GamePlayer.cpp 0x000C1930 — "SetInitialVariables" (the event database reset it ends with is
    /// `CEventDatabase::reset_events`).
    pub fn set_initial_variables(&mut self) {
        self.selected_vehicle = crc("350Z");
        self.traffic = false;
        self.professional = false;
        self.flag_girl = 0;
        self.network_race = false;
        self.cash = 0x3D090;
        self.prestige = 0;
        self.unknown_25c = -1;
        self.team = 0;
        *self.teams_joined_mut() = 0;
        *self.tips_watched_mut() = 0;
        *self.plate_style_mut() = 0;
        self.reward = 0.0;
        self.garage = vec![crc("300ZX"), crc("RX8"), crc("NSX")];
        self.garage_slot = 0;
        self.garage_vehicle = self.garage[0];
        // dword_6F71DC = 0 is written again here in the original, then the text
        self.set_plate_text("OCTANE");
        self.event_state = 0;
        self.unlock_message = 0;
        self.speech = true;
    }

    /// FECK2Manager.cpp 0x002022F0 (from `GameManager::InitDatabases`) — a default setup per garage car; the free
    /// slots up to 6 get a 350Z.
    pub fn init_garage_setups(&mut self, db: &CVehicleDatabase) {
        self.garage_setups = (0..GARAGE_SLOTS)
            .map(|i| {
                let mut s = VehicleSetup::new();
                db.setup_default_vehicle(&mut s, self.garage.get(i).copied().unwrap_or_else(|| crc("350Z")));
                s
            })
            .collect();
    }

    /// GameVars.cpp 0x000C20F0 — add a car to the garage (at most 6); returns the slot it went to.
    pub fn add_garage_vehicle(&mut self, vehicle: u32, setup: VehicleSetup) -> usize {
        let n = self.garage.len();
        if n < GARAGE_SLOTS {
            self.garage.push(vehicle);
            if self.garage_setups.len() <= n {
                self.garage_setups.resize_with(n + 1, VehicleSetup::new);
            }
            self.garage_setups[n] = setup;
        }
        n
    }

    /// GameVars.cpp 0x000C1F90 — the setup of a garage slot
    pub fn garage_setup(&self, slot: usize) -> Option<&VehicleSetup> {
        if slot >= self.garage.len() {
            log::error!("assert: vehicleIndex<m_PlayerInfoA[playerIndex].mNumCarSlots");
        }
        self.garage_setups.get(slot)
    }

    /// GamePlayer.cpp 0x000C1DA0 — per category, the tiers 1..5 that hold an unlocked event.
    pub fn count_unlocked_tiers(&mut self, db: &CEventDatabase) {
        for i in 0..3u32 {
            let mut n = 0u8;
            for j in 1..=5u8 {
                if db.count_events(i + 1, j, 0, false) > db.count_events(i + 1, j, EVENT_FLAG_LOCKED, false) {
                    n += 1;
                }
            }
            self.unlocked_tiers[i as usize] = n;
        }
    }

    /// the event being raced (`dword_6F9BE8`).
    pub fn current_event<'a>(&'a self, db: &'a CEventDatabase) -> Option<&'a CEvent> {
        match self.event.as_ref()? {
            EventRef::Database(i) => db.events.get(*i),
            EventRef::Temp(e) => Some(e),
        }
    }

    pub fn current_event_mut<'a>(&'a mut self, db: &'a mut CEventDatabase) -> Option<&'a mut CEvent> {
        match self.event.as_mut()? {
            EventRef::Database(i) => db.events.get_mut(*i),
            EventRef::Temp(e) => Some(e),
        }
    }

    /// GameVars.cpp 0x000C2EE0 path of the builders: a temporary event is freed before a new one is made.
    pub fn set_temp_event(&mut self, e: CEvent) {
        debug_assert!(e.flags & EVENT_FLAG_TEMP != 0);
        self.event = Some(EventRef::Temp(Box::new(e)));
    }
}

impl CareerQuery for CFEGameVariables {
    fn prestige(&self) -> i32 {
        self.prestige
    }
    /// CompartmentManager__8E930 (idb name): looks through player 0's garage whatever player is asked (kept)
    fn owns_vehicle(&self, c: u32) -> bool {
        self.garage.contains(&c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plate_text_lives_in_the_saved_block() {
        let mut v = CFEGameVariables::default();
        assert_eq!(v.plate_text(), "OCTANE");
        assert_eq!(v.unknown_264[3], 'O' as u32 | ('C' as u32) << 16);
        v.set_plate_text("ABCDEFGHIJ");
        assert_eq!(v.plate_text(), "ABCDEFGH");
        v.set_plate_text("Q");
        assert_eq!(v.plate_text(), "Q");
        assert_eq!(v.unknown_264[4..7], [0, 0, 0]);
    }
}
