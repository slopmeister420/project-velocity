//! src\Feck2\Screens\Hangout\ — "The Scene": FEHangoutScreen.cpp (`HangoutScreen`) and FEHangoutProgression.cpp
//! (`CFEHangoutProgression`, the career messages after a race). The career itself (`g_FEGameVariables`, the event
//! database, saves) is core's; the screens read [`CareerView`] (filled by the bevy side from `GameManager`) and
//! queue [`CareerRequest`]s that become `GameFlowRequest`s. Core's answers arrive in `CareerView::messages`.

pub mod fe_hangout_progression;
pub mod fe_hangout_screen;

pub use velocity_game::career_teams::{TeamRequest, TeamView};
pub use velocity_game::game_mode::career::CareerMessage;

/// one career event as the hangout reads it (`CEvent` fields)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CareerEventView {
    pub crc: u32,      // +0x10
    pub category: u32, // +0x00 1 underground, 2 professional, 3 team challenge, 4 opponent challenge
    pub kind: u32,     // +0x04 1 single, 2 cup, 3 series
    pub tier: u8,      // +0x14
    pub flags: u8,     // +0x15 1 locked, 2 completed, 4 failed, 8 temp
    pub area: u32,        // +0x0C
    pub unlock_team: u32, // +0x40
    /// +0x48: the cut scene (`off_685400` index) played when the hangout starts this event by itself (0x0021C430)
    pub unlock_message: u32,
    pub movie: u32,       // +0x4C
    pub tips: u32,        // +0x50
    /// +0x54: the loading screen's tutorial tip (0x00240F60, `dword_75C670` index)
    pub loading_tip: u32,
    /// round 0: its race count, first race's laps / competitors, that race's type (+0x28) and icon (+0x14)
    pub race_count: u32,
    pub laps: u32,
    pub opponents: u32,
    pub race_type: u32,
    pub track_icon: Option<String>,
    /// unlock criteria +0x08[0] / +0x0C[0]: the first event to complete and its tier
    pub unlock_event: Option<(u32, u8)>,
    pub result_position: u8, // +0x2C +0
    pub result_points: u32,  // +0x2C +4
    /// competitors' points (+0x24)
    pub points: Vec<u32>,
    /// the last round's first race, slot 0 finish position
    pub finish_position: u8,
    /// the current race (`+0x20 +0x0C` = VsAIPoliceVehicle 0x00017CB0), read by DummyGameScreen
    pub current_race: Option<CurrentRaceView>,
}

/// the event's current race as DummyGameScreen reads it
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurrentRaceView {
    /// +0x15 competitors
    pub competitors: u8,
    /// EventDatabaseBinLoading 0x00091B70: humans seated
    pub humans: u8,
    /// +0x10 the race record: its DisplayID crc (+0x04) and professional (+0x2C == 2); None = "not in build"
    pub details: Option<(u32, bool)>,
    /// per slot the competitor's opponent crc (0 / None = the player)
    pub entrants: Vec<Option<u32>>,
}

/// what the front end sends to game flow (core) for the career
#[derive(Clone, Debug, PartialEq)]
pub enum CareerRequest {
    NewProfile { name: String },
    LoadProfile { name: String },
    SaveProfile,
    DeleteProfile { name: String },
    StartCareerEvent { event: u32 },
    /// `dword_6F9BE8 = 0` (CFEHangoutProgression 0x0028F790 step 8)
    ClearEvent,
    /// `dword_6F7248 = 0` (0x0028FBD0 step 2)
    ClearUnlockMessage,
    /// GamePlayer 0x000C1DA0 (0x002900A0 step 6)
    CountUnlockedTiers,
    /// `dword_6F71D0 = team` (FEHangoutProgression 0x00290BC0)
    SetTeam { team: u32 },
    /// `++dword_6F71D4` teams joined with a bonus (0x00290BC0)
    CountTeamJoined,
    /// a team flag write for the ai's `CTeamDatabase` (0x00290D60 / 0x00290DC0)
    Team(TeamRequest),
    /// `dword_6F71D8 = flags`: the tips movies seen (FETipsScreen 0x002400D0)
    SetTipsWatched { flags: u32 },
    /// the hangout's DummyGameScreen route (0x00219F70): CFEHangoutProgression::StartEvent's game part (core's
    /// `career::start_event`) without starting the race
    SelectEvent { event: u32 },
    /// DummyGameScreen Select 0x00209340, result branch: `dword_6F7244 += prestige`, the current race's finish
    /// positions (`position` for the player, 0 = no human seated), race / round / event over, `byte_6F5D08 = 0`;
    /// then RaceCompleteScreen (whose Start saves)
    DummyResult { position: u32, prestige: u32 },
    /// DummyGameScreen Select, race branch: the LoadingScreen starts `m_pEvent` as it is (GameManager 0x000BBF60,
    /// no event reset)
    ContinueEvent,
    /// `qmemcpy(dword_6F724C, FE3DVehicles +1824)` (hangout Select 0x00219D9B / 0x00219EB6): the car on show
    /// (`foyer_vehicles.slots[0].setup`, the garage car) becomes the current car the next race uses
    CopyShownCar,
    /// `byte_6F9058 = 1` (FEHangoutScreen 0x0021C430, the tutorial start)
    SetCompletedAny,
    /// `CEventDatabase::EvaluateUnlocks(0)` (EventDatabase 0x0008F020) after the showroom bought a car (0x0021E2D0)
    EvaluateUnlocks,
}

/// core's career state for the screens, refreshed every front end frame
#[derive(Clone, Debug, Default)]
pub struct CareerView {
    /// game flow exists (false in the headless captures)
    pub present: bool,
    /// the database events in list order
    pub events: Vec<CareerEventView>,
    /// `dword_6F9BE8`: the current database event, if any
    pub event: Option<CareerEventView>,
    pub completed_any: bool,  // byte_6F9058
    pub profile_active: bool, // byte_6F9055
    /// `byte_6F9054` a profile is in use: RaceCompleteScreen saves it (core's `vars.save_enabled`)
    pub save_enabled: bool,
    /// `byte_6F5D08` = GameManager +0x140 `m_bQuitRace`
    pub quit_race: bool,
    /// the after-race save's XMem status (`dword_701404` +0): 3 done, -1 failed, 0 none attempted; set from core's
    /// `ReturnedFromRace { saved }` (core saves in `race_complete`), taken by RaceCompleteScreen's Update
    pub race_save_status: Option<i32>,
    /// queued for core
    pub requests: Vec<CareerRequest>,
    /// core's answers since the last frame
    pub messages: Vec<CareerMessage>,
    /// the hangout picked an event: the loading screen asks for it instead of a quick race (GameManager 0x000BBF60
    /// starts whatever `m_pEvent` is)
    pub start_pending: Option<u32>,
    /// DummyGameScreen's "play race": the loading screen starts the current event as it is
    pub continue_pending: bool,
    /// `byte_6F9BE4..` unlocked tiers per category (GamePlayer 0x000C1DA0)
    pub unlocked_tiers: [u8; 3],
    /// `dword_6F7248` movie of the event unlocked last
    pub unlock_message: u32,
    /// `dword_6F9BDC` the browser's category (front end global, kept across visits)
    pub category: u32,
    /// `dword_6F71D0` the player's team (0 none)
    pub team: u32,
    /// `dword_6F71D8` (GameVars +0x268, core's `unknown_264[1]`): bit `tip - 1` = that tip's movie was seen
    pub tips_watched: u32,
    /// the ai's `CTeamDatabase` in list order (`velocity_game::career_teams::CareerTeams`)
    pub teams: Vec<TeamView>,
}

impl CareerView {
    /// VsCommonMain 0x0016E610 — team by crc
    pub fn find_team(&self, crc: u32) -> Option<&TeamView> {
        self.teams.iter().find(|t| t.crc == crc)
    }

    /// VsCommonMain 0x0016E8B0 — teams offering invites other than `exclude`
    pub fn count_teams_offering_invites(&self, exclude: u32) -> usize {
        self.teams.iter().filter(|t| t.is_offering_invite() && exclude != t.crc).count()
    }

    /// a team flag write: applied to the local copy at once (the original edits the team in place) and queued for
    /// the ai
    pub fn team_request(&mut self, r: TeamRequest) {
        let (crc, set, clear) = match r {
            TeamRequest::ClearNewlyUnlocked(c) => (c, 0, 0x20),
            TeamRequest::SetJoinedOnce(c) => (c, 0x10, 0),
        };
        if let Some(t) = self.teams.iter_mut().find(|t| t.crc == crc) {
            t.flags = (t.flags | set) & !clear;
        }
        self.requests.push(CareerRequest::Team(r));
    }

    /// `EventDatabaseEvents 0x00093080` — event by crc
    pub fn find_event(&self, crc: u32) -> Option<&CareerEventView> {
        self.events.iter().find(|e| e.crc == crc)
    }

    /// CompartmentManager 0x0008EDB0 — events of a category in a tier (0 = any) with / without flag bits
    pub fn count(&self, tier: u8, category: u32, flags: u8, without: bool) -> u32 {
        self.events.iter().filter(|e| fe_hangout_progression::matches(e, tier, category, flags, without)).count() as u32
    }

    /// CompartmentManager 0x0008EED0 — the n-th such event in list order
    pub fn nth(&self, tier: u8, category: u32, index: u32, flags: u8, without: bool) -> Option<&CareerEventView> {
        self.events.iter().filter(|e| fe_hangout_progression::matches(e, tier, category, flags, without)).nth(index as usize)
    }
}
