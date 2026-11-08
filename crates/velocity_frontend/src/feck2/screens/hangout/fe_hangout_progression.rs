//! src\Feck2\Screens\Hangout\FEHangoutProgression.cpp — `CFEHangoutProgression` (`mpHangoutProgression`, 0x100
//! bytes at HangoutScreen +456; code 0x0028B8A0..0x00292FF0, the idb's FECK2ElementStaticText / FEHangoutProgression
//! ranges): Tory's side of the hangout — the event browser (category -> tier -> event list -> event details ->
//! start), starting an event (0x0028F430) and the messages after a race (0x0028F790 steps).
//!
//! Pages (`+8`): 0 categories, 1 challenges, 2 (passes on to 3), 3 tiers, 4 events, 5 event details, 6 start,
//! 7 team invites, 8 invite answer. The top level (`+4`, set on open from the tier 1 unlocked counts): 0 a single
//! underground event (straight to its details), 1 underground only (its tier 1 events), 2 underground and
//! challenges, 3 everything. `dword_6F9BDC` (category) lives in `CareerView::category`, the browsed
//! `dword_6F9BE8` in `browse`.
//!
//! Teams come from the ai's `CTeamDatabase` through `CareerView::teams` (`velocity_game::career_teams`). Not ported:
//! StartEvent's team / opponent car (0x0028F430 categories 2..4). Steps 1/2 play the cut scenes `CUTS\<off_685400>`,
//! step 3 sets the tip and goes to TipsScreen.

use crate::feck2::feck2_manager::{visos_colour, Feck2};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::feck2::screens::common::fe_2d_overlay_menu as menu;
use crate::feck2::screens::garage::ov;
use crate::feck2::screens::hangout::{CareerEventView, CareerRequest, TeamRequest};
use crate::string_manager::string_manager::string_crc;
use velocity_game::event_database::event_database::{EVENT_FLAG_COMPLETED, EVENT_FLAG_FAILED, EVENT_FLAG_LOCKED};

/// `dword_76A4A0` / `dword_76A4A8` / `dword_76A4C0` / `dword_76A4CC` / `dword_76A4B4` / `dword_76A4D8` (static init
/// 0x00430C5A..0x0043105B)
/// `off_685400`: the cut scenes by event +0x48 / +0x4C / `dword_6F7248`, played as `CUTS\<name>` (13 entries; the
/// next table, the race type names, follows directly)
pub const CUT_MOVIES: [&str; 13] = [
    "NONE", "intro", "firstRaceEnd", "t1Team", "t1Story1Pre", "t2Story1Pre", "t2Story1Post", "t3Story1", "t4Story1",
    "t4DriverX", "t5Story1", "finalPre", "finalPost",
];

/// `GameManager__BE9F0("CUTS\" + off_685400[index], 1, 0)` (0x0021C430 / 0x0028FBD0)
pub fn play_cut_movie(fe: &mut Feck2, index: u32) {
    match CUT_MOVIES.get(index as usize) {
        Some(name) => fe.movies.push(format!("CUTS\\{name}")),
        None => log::warn!("hangout: cut scene {index} is past off_685400"),
    }
}

const DETAIL_LABELS: [&str; 2] = ["EVENT_DETAILS_OPPONENTS", "EVENT_DETAILS_LAPS"];
const TIER_LABELS: [&str; 3] = ["TIER_UNLOCKED", "TIER_LOCKED", "TIER_COMPLETED"];
const TIER_VALUES: [&str; 3] = ["TIER_UNLOCKED_VALUE", "TIER_LOCKED_VALUE", "TIER_COMPLETED_VALUE"];
const CATEGORY_NAMES: [&str; 3] = ["RACETYPE_UNDERGROUND", "RACETYPE_PROFESSIONAL", "RACETYPE_CHALENGES"];
const TEAM_LABELS: [&str; 5] = ["TEAM_INVITE_TEAM", "TEAM_INVITE_BENEFIT", "TEAM_INVITE_BONUS", "TEAM_INVITE_CAR", "TEAM_INVITE_STATUS"];
/// `dword_6F0DA8` race type names (static init 0x004222C3..; 5 and 6 copy the last one)
const RACE_TYPE_NAMES: [&str; 7] = [
    "RACECATEGORY_UNKNOWN",
    "RACECATEGORY_CRUISE",
    "RACECATEGORY_SPRINT",
    "RACECATEGORY_CIRCUIT",
    "RACECATEGORY_RELAY",
    "RACECATEGORY_RELAY",
    "RACECATEGORY_RELAY",
];
/// `dword_6C128C`... message time of the steps (`SetMessageTimeout(200)`)
const MESSAGE_FRAMES: i32 = 200;
/// the team dialogs' options (`27B7F0(2, title, GENERAL_NO, GENERAL_YES)`): selection 1 = yes
const GENERAL_NO: u32 = -668354537i32 as u32;
const GENERAL_YES: u32 = -1829768882i32 as u32;
/// team strings (string crcs as pushed in 0x0028FE70 / 0x0028EE90 / 0x00290AF0 / 0x002909B0)
const TEAM_UNLOCKED: u32 = 718512284; // idTeamName, idCashBonus
const TEAM_JOIN: u32 = -42298232i32 as u32; // idJoinTeamName (joined once before)
const TEAM_JOIN_BONUS: u32 = -304982335i32 as u32; // idJoinTeamName, idCashBonus
const TEAM_SWAP: u32 = -1826607064i32 as u32; // idNewTeamName, idCurrentTeamName
const TEAM_JOINED_BONUS: u32 = 1595163490; // idJoinedTeamName, idCashBonus
const TEAM_JOINED: u32 = 752791814; // idJoinedTeamName

/// `CFEHangoutProgression`
#[derive(Default)]
pub struct CFEHangoutProgression {
    pub active: bool,                // +0 the browser is up
    pub started: bool,               // +1 StartEvent ran
    pub top: u32,                    // +4
    pub page: u32,                   // +8
    pub step: u32,                   // +12 0x0028F790 step
    pub layer: LayerId,              // +16
    pub name: ElementId,             // +20 CurrentEventText
    pub info: ElementId,             // +24 CurrentEventInfo
    pub details: [ElementId; 2],     // +28 EventInfotext (opponents, laps)
    pub background: ElementId,       // +36 EventTrackSprite (dynamic: page backgrounds)
    pub track: ElementId,            // +40 EventTrackSprite
    pub start_text: ElementId,       // +44
    pub start_button: ElementId,     // +48
    pub tier_labels: [ElementId; 3], // +52
    pub tier_values: [ElementId; 3], // +64
    pub locked_text: ElementId,      // +76
    pub tier_heading: ElementId,     // +80
    pub tier_status: ElementId,      // +84
    pub criteria_title: ElementId,   // +88
    pub criteria_info: ElementId,    // +92
    pub completed_text: ElementId,   // +96
    pub invite_sel: i32,             // +100
    pub invite_count: i32,           // +104
    pub invite_cache: [u32; 10],     // +108 the inviting teams (crcs for the original's team pointers)
    pub team_labels: [ElementId; 5], // +180
    pub team_texts: [ElementId; 5],  // +200
    /// +220 `mpTeamExtendingInvite`
    pub invite_team: Option<u32>,
    /// +224 the invite dialog: 1 unlocked team (after a race), 2 its swap prompt, 3 browser invite, 4 its swap prompt
    pub invite: u32,
    pub exit_screen: u32,            // +228
    pub exit_countdown: i32,         // +232
    pub last_dir: i32,               // +236 1 forward, -1 back
    pub category_sel: i32,           // +240
    pub tier: i32,                   // +244
    pub event_sel: i32,              // +248
    pub challenge_sel: i32,          // +252
    /// `dword_6F9BE8` while browsing
    pub browse: Option<u32>,
}

/// FEHangoutProgression.cpp 0x0028B8A0 + 0x0028B900 — ctor and Init on the hangout's layer
pub fn create(fe: &mut Feck2, layer: LayerId) -> CFEHangoutProgression {
    let mut p = CFEHangoutProgression { layer, ..Default::default() };
    p.reset(fe);
    p.create_elements(fe);
    p.invite_sel = -1;
    p.show_details(fe, false);
    p.show_events(fe, false);
    p.show_tiers(fe, false);
    p.show_categories(fe, false);
    p.show_challenges(fe, false);
    p.show_invites(fe, false);
    p.show_invite(fe, -1);
    p
}

impl CFEHangoutProgression {
    /// FEHangoutProgression.cpp 0x0028B9B0
    fn reset(&mut self, fe: &mut Feck2) {
        self.exit_screen = 0;
        self.exit_countdown = 0;
        self.last_dir = 0;
        self.top = 0;
        self.page = 0;
        self.step = 0;
        self.category_sel = 0;
        self.tier = 0;
        self.event_sel = 0;
        self.challenge_sel = 0;
        self.started = false;
        if fe.career.category == 0 {
            fe.career.category = 1;
        }
    }

    /// FEHangoutProgression.cpp 0x0028BA60 — the browser's texts and sprites (the desc carries over between them)
    fn create_elements(&mut self, fe: &mut Feck2) {
        let l = self.layer;
        let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 2, crc: string_crc("CurrentEventText"), ..Default::default() };
        d.position[0] = 50.0;
        d.position[1] = 100.0;
        d.string = string_crc("EVENT_DETAILS_NAME");
        self.name = fe.create_element(l, &d);
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = 2;
        d.crc = string_crc("CurrentEventInfo");
        d.position = [50.0, 300.0, d.position[2], d.position[3]];
        d.string = string_crc("EVENT_DETAILS_INFORACES_SINGLE");
        self.info = fe.create_element(l, &d);
        d.crc = string_crc("StartText");
        d.position[0] = 310.0;
        d.position[1] = 340.0;
        d.string = string_crc("ENTRY_START");
        self.start_text = fe.create_element(l, &d);
        d.kind = ELEMENT_2D;
        d.crc = string_crc("StartRaceButton");
        d.position[0] = 290.0;
        d.position[1] = 334.0;
        d.texture = "select_button".into();
        d.vcd = "\\FE\\FE.vcd".into();
        self.start_button = fe.create_element(l, &d);
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = 0;
        d.crc = string_crc("EventInfotext");
        d.position[1] = 330.0;
        for i in 0..2 {
            d.position[0] = (60 * i) as f32 + 50.0;
            d.string = string_crc(DETAIL_LABELS[i]);
            self.details[i] = fe.create_element(l, &d);
        }
        d.font = 1;
        d.crc = string_crc("TierInfoText");
        for i in 0..3 {
            d.position[1] = (20 * i + 170) as f32;
            d.position[0] = 70.0;
            d.string = string_crc(TIER_LABELS[i]);
            self.tier_labels[i] = fe.create_element(l, &d);
            d.position[0] = 220.0;
            d.string = string_crc(TIER_VALUES[i]);
            self.tier_values[i] = fe.create_element(l, &d);
        }
        d.position[1] = 330.0;
        d.position[0] = 50.0;
        d.string = string_crc("STATUS_LOCKED");
        self.locked_text = fe.create_element(l, &d);
        d.font = 4;
        d.crc = string_crc("CurrentEventText");
        d.position[0] = 50.0;
        d.position[1] = 100.0;
        d.string = string_crc("EVENT_DETAILS_TIER");
        self.tier_heading = fe.create_element(l, &d);
        d.font = 2;
        d.position[0] = 70.0;
        d.position[1] = 140.0;
        d.string = string_crc("EVENT_TIER_STATUS");
        self.tier_status = fe.create_element(l, &d);
        d.crc = string_crc("OpeningCriteriaTitle");
        d.position[0] = 50.0;
        d.position[1] = 330.0;
        d.string = string_crc("TIEROPENING_TITLE");
        self.criteria_title = fe.create_element(l, &d);
        d.font = 1;
        d.crc = string_crc("OpeningCriteriaInfo");
        d.position[0] = 70.0;
        d.position[1] = (30 * (0 + 1) + 330) as f32;
        d.string = 0;
        self.criteria_info = fe.create_element(l, &d);
        d.font = 2;
        d.crc = string_crc("EventCompletedText");
        d.position[0] = 50.0;
        d.position[1] = 370.0;
        d.string = string_crc("EVENT_COMPLETED");
        self.completed_text = fe.create_element(l, &d);
        d.kind = ELEMENT_2D;
        d.crc = string_crc("EventTrackSprite");
        d.position[0] = 50.0;
        d.position[1] = 100.0;
        d.texture = "track001".into();
        d.vcd = "\\FE\\FETracks.vcd".into();
        self.track = fe.create_element(l, &d);
        d.dynamic = true;
        d.kind = ELEMENT_2D;
        d.position[0] = 35.0;
        d.position[1] = 85.0;
        d.texture = String::new();
        d.vcd = String::new();
        self.background = fe.create_element(l, &d);
        d.dynamic = false;
        for i in 0..5 {
            d.kind = ELEMENT_STATIC_TEXT;
            d.font = 1;
            d.crc = string_crc("TeamInfoLabel");
            d.position[0] = 300.0;
            d.position[1] = (20 * i + 250) as f32;
            d.string = string_crc(TEAM_LABELS[i]);
            self.team_labels[i] = fe.create_element(l, &d);
            d.kind = ELEMENT_STATIC_TEXT;
            d.font = 1;
            d.crc = string_crc("TeamInfoText");
            d.position[0] = 380.0;
            d.position[1] = (20 * i + 250) as f32;
            d.string = string_crc("TeamInviteName");
            self.team_texts[i] = fe.create_element(l, &d);
        }
        self.invite_sel = -1;
    }

    /// FEHangoutProgression.cpp 0x0028C610 — Load (from the hangout's first Enter)
    pub fn load(&mut self, fe: &mut Feck2) {
        let orange = visos_colour(0x80, 0x78, 0x4D, 0x0E);
        fe.element_load(self.name);
        fe.element_load(self.info);
        for i in 0..2 {
            fe.element_load(self.details[i]);
            fe.text(self.details[i]).set_colour(visos_colour(0x80, 0xFF, 0x6E, 0));
        }
        fe.element_load(self.start_text);
        fe.text(self.start_text).set_colour(orange);
        fe.element_load(self.start_button);
        fe.element_load(self.track);
        fe.element_load(self.background);
        for i in 0..3 {
            fe.element_load(self.tier_labels[i]);
            fe.element_load(self.tier_values[i]);
        }
        fe.element_load(self.tier_heading);
        fe.text(self.tier_heading).set_colour(orange);
        fe.element_load(self.tier_status);
        fe.element_load(self.criteria_title);
        fe.text(self.criteria_title).set_colour(orange);
        fe.element_load(self.criteria_info);
        fe.text(self.criteria_info).set_box(150.0, 40.0);
        fe.element_load(self.locked_text);
        fe.text(self.locked_text).set_colour(orange);
        fe.element_load(self.completed_text);
        for i in 0..5 {
            fe.element_load(self.team_labels[i]);
            fe.element_load(self.team_texts[i]);
        }
    }

    /// FEHangoutProgression.cpp 0x0028F750 — a new visit restarts the steps unless a message step is running
    pub fn reset_steps(&mut self) {
        if !(1..=3).contains(&self.step) {
            self.step = 0;
        }
    }

    /// FEHangoutScreen.cpp 0x0021C790 — a team invite is waiting for an answer
    pub fn invite_waiting(&self) -> bool {
        self.page == 8 && (1..=3).contains(&self.invite)
    }

    // ---- browser -----------------------------------------------------------------------------------------------

    /// FEHangoutProgression.cpp 0x0028C9C0 — open (the top level from the tier 1 counts) / close the browser
    pub fn open(&mut self, fe: &mut Feck2, open: bool) {
        self.active = open;
        if open {
            self.reset(fe);
        }
        if self.active {
            ov(fe, |o, fe| overlay::show_message(o, fe, false));
            self.step = 0;
            // dword_6F90D8 (-1, 0 or 1 here: no online team session)
            let c = &fe.career;
            let (u1, u2, u3) = (c.count(1, 1, EVENT_FLAG_LOCKED, true), c.count(1, 2, EVENT_FLAG_LOCKED, true), c.count(1, 3, EVENT_FLAG_LOCKED, true));
            if u1 == 1 && u2 == 0 && u3 == 0 {
                self.top = 0;
                self.page = 5;
                fe.career.category = 1;
                self.browse = fe.career.nth(1, 1, 0, 0, false).map(|e| e.crc);
            } else if u1 != 0 && u2 == 0 && u3 == 0 {
                self.top = 1;
                self.page = 4;
                fe.career.category = 1;
                self.tier = 0;
                self.event_sel = fe.career.count((self.tier as u8) + 1, 1, EVENT_FLAG_COMPLETED, false) as i32;
            } else if u1 != 0 && u2 == 0 && u3 != 0 {
                self.event_sel = 0;
                self.top = 2;
                self.page = 0;
            } else {
                self.event_sel = 0;
                self.top = 3;
                self.page = 0;
            }
            self.show_page(fe);
            ov(fe, |o, fe| overlay::fade_arrows(o, fe, -128, 0, 0));
        } else {
            self.show_details(fe, false);
            self.show_events(fe, false);
            self.show_tiers(fe, false);
            self.show_categories(fe, false);
            ov(fe, |o, fe| {
                menu::show(o, fe, false, true);
                menu::set_active(o, fe, false);
                overlay::fade_arrows(o, fe, 128, 0, 0);
                menu::fade_title(o, fe, 128, 0);
                fe.el(o.menu_title).set_visible(true);
                fe.el(o.title_text).set_visible(false);
            });
        }
    }

    /// FEHangoutProgression.cpp 0x0028CC70 — every page hidden, then the current one shown
    fn show_page(&mut self, fe: &mut Feck2) {
        self.show_details(fe, false);
        self.show_events(fe, false);
        self.show_tiers(fe, false);
        self.show_categories(fe, false);
        self.show_challenges(fe, false);
        self.show_invites(fe, false);
        match self.page {
            5 => self.show_details(fe, true),
            4 => self.show_events(fe, true),
            3 => self.show_tiers(fe, true),
            1 => self.show_challenges(fe, true),
            7 => self.show_invites(fe, true),
            8 => {}
            _ => self.show_categories(fe, true),
        }
    }

    fn selected(fe: &Feck2) -> i32 {
        fe.overlay.as_ref().map_or(0, |o| o.menu_selected)
    }

    fn set_arg(fe: &mut Feck2, id: ElementId, arg: &str, v: u32) {
        if let Some(h) = fe.text(id).string {
            fe.strings.set_argument(h, string_crc(arg), v).ok();
        }
    }

    fn title_to_menu(fe: &mut Feck2) {
        let name = fe.characters.chars.first().map_or(0, |c| c.name_id);
        ov(fe, |o, fe| {
            fe.text(o.menu_title).set_string(name);
            menu::fade_title(o, fe, 128, 0);
        });
    }

    /// FEHangoutProgression.cpp 0x0028CD50 — the browsed event's texts and track picture
    fn fill_details(&mut self, fe: &mut Feck2) {
        let Some(e) = self.browse.and_then(|c| fe.career.find_event(c)).cloned() else {
            log::warn!("No Event set");
            return;
        };
        if let Some(icon) = e.track_icon.as_deref() {
            fe.change_texture(self.track, icon, false);
        }
        let area = match e.area {
            1 => "LEVEL_AREA_LA",
            2 => "LEVEL_AREA_LONDON",
            3 => "LEVEL_AREA_TOKYO",
            _ => "LEVEL_AREA_UNKNOWN",
        };
        let kind = match e.kind {
            1 => "EVENT_DETAILS_TYPE_SINGLE_RACE",
            2 => "EVENT_DETAILS_TYPE_CUP",
            3 => "EVENT_DETAILS_TYPE_LEAGUE",
            _ => "EVENT_DETAILS_TYPE_ERROR",
        };
        if e.race_type == 2 || e.race_type == 6 {
            fe.text(self.details[0]).set_string(string_crc("EVENT_DETAILS_ATOB"));
        } else {
            fe.text(self.details[0]).set_string(string_crc("EVENT_DETAILS_LAPS"));
            Self::set_arg(fe, self.details[0], "dLaps", e.laps);
        }
        if e.opponents == 1 {
            fe.text(self.details[1]).set_string(string_crc("EVENT_DETAILS_OPPONENTS_SINGLE"));
        } else {
            fe.text(self.details[1]).set_string(string_crc("EVENT_DETAILS_OPPONENTS_MULTIPLE"));
            Self::set_arg(fe, self.details[1], "dOpponents", e.opponents);
        }
        Self::set_arg(fe, self.name, "idEventLocation", string_crc(area));
        Self::set_arg(fe, self.name, "idEventName", e.crc);
        let race_type = string_crc(RACE_TYPE_NAMES[(e.race_type as usize).min(6)]);
        if e.race_count == 1 {
            fe.text(self.info).set_string(string_crc("EVENT_DETAILS_INFORACES_SINGLE"));
        } else {
            fe.text(self.info).set_string(string_crc("EVENT_DETAILS_INFORACES_MULTIPLE"));
            Self::set_arg(fe, self.info, "dRaceCount", e.race_count);
        }
        Self::set_arg(fe, self.info, "idEventType", string_crc(kind));
        Self::set_arg(fe, self.info, "idRaceType", race_type);
    }

    /// FEHangoutProgression.cpp 0x0028D3A0 — page 5: the event's details
    fn show_details(&mut self, fe: &mut Feck2, show: bool) {
        if show {
            self.fill_details(fe);
            fe.change_texture(self.background, "FE_selectevent_background", false);
        }
        for id in [self.name, self.info, self.details[0], self.details[1], self.background, self.track, self.start_text, self.start_button] {
            fe.el(id).set_visible(show);
        }
        if show {
            Self::title_to_menu(fe);
            ov(fe, |o, fe| fe.el(o.menu_title).set_visible(true));
        }
    }

    /// FEHangoutProgression.cpp 0x0028D4D0 — the tier's events (locked ones disabled)
    fn list_events(&mut self, fe: &mut Feck2) {
        if self.last_dir == 1 {
            self.event_sel = 0;
        }
        let (tier, cat) = ((self.tier as u8) + 1, fe.career.category);
        let n = fe.career.count(tier, cat, 0, false);
        let evs: Vec<(u32, bool)> = (0..n).filter_map(|i| fe.career.nth(tier, cat, i, 0, false)).map(|e| (e.crc, e.flags & EVENT_FLAG_LOCKED == 0)).collect();
        let sel = self.event_sel;
        ov(fe, |o, fe| {
            menu::clear(o, fe);
            for (crc, enabled) in evs {
                menu::add_entry(o, fe, crc, enabled, false, 0);
            }
            menu::set_selection(o, fe, sel);
        });
    }

    /// FEHangoutProgression.cpp 0x0028D5E0 — page 4: the event list with the highlighted event's summary
    fn show_events(&mut self, fe: &mut Feck2, show: bool) {
        if show {
            self.list_events(fe);
            fe.change_texture(self.background, "FE_trackinfo_background", false);
        }
        ov(fe, |o, fe| menu::show(o, fe, show, false));
        for id in [self.completed_text, self.locked_text, self.background, self.track] {
            fe.el(id).set_visible(show);
        }
        for i in 0..2 {
            fe.el(self.details[i]).set_visible(false);
        }
        fe.el(self.name).set_visible(show);
        fe.el(self.info).set_visible(show);
        if show {
            self.event_selected(fe);
            Self::title_to_menu(fe);
            let category = match fe.career.category {
                1 => "RACETYPE_UNDERGROUND_TITLE",
                2 => "RACETYPE_PROFESSIONAL_TITLE",
                _ => "RACETYPE_CHALENGES_TITLE",
            };
            let tier = self.tier as u32 + 1;
            ov(fe, |o, fe| {
                overlay::set_title(o, fe, string_crc("EVENT_DETAILS_INFOTIER"));
                Self::set_arg(fe, o.title_text, "idCategoryType", string_crc(category));
                Self::set_arg(fe, o.title_text, "dTier", tier);
            });
        }
    }

    /// FEHangoutProgression.cpp 0x0028D8A0 — the category's tiers; false when there is only one (passed through)
    fn list_tiers(&mut self, fe: &mut Feck2) -> bool {
        let cat = fe.career.category;
        let mut n = 0;
        if self.last_dir == 1 {
            self.tier = 0;
        }
        let mut entries = Vec::new();
        for t in 0..5u8 {
            if fe.career.count(t + 1, cat, 0, false) == 0 {
                break;
            }
            n += 1;
            let unlocked = fe.career.count(t + 1, cat, EVENT_FLAG_LOCKED, true);
            let completed = fe.career.count(t + 1, cat, EVENT_FLAG_COMPLETED, false);
            entries.push((string_crc(&format!("EVENT_TIER_{}", t + 1)), unlocked != 0, unlocked != 0 && unlocked != completed));
        }
        let sel = self.tier;
        ov(fe, |o, fe| {
            menu::clear(o, fe);
            for (crc, enabled, icon) in entries {
                if icon {
                    menu::add_icon_entry(o, fe, crc, true, "\\FE\\FE.vcd", "FE_Icon_RaceAvailable", None);
                } else {
                    menu::add_entry(o, fe, crc, enabled, false, 0);
                }
            }
            menu::set_selection(o, fe, sel);
        });
        if n == 1 {
            self.tier = 0;
            if self.last_dir == 1 {
                self.select(fe);
            } else if self.last_dir == -1 {
                self.cancel(fe);
            }
            return false;
        }
        true
    }

    /// FEHangoutProgression.cpp 0x0028DAD0 — page 3: tiers with the highlighted tier's counts
    fn show_tiers(&mut self, fe: &mut Feck2, show: bool) {
        if show {
            if !self.list_tiers(fe) {
                return;
            }
            fe.change_texture(self.background, "FE_trackinfo_background", false);
        }
        ov(fe, |o, fe| menu::show(o, fe, show, false));
        for i in 0..3 {
            fe.el(self.tier_labels[i]).set_visible(show);
            fe.el(self.tier_values[i]).set_visible(show);
        }
        for id in [self.tier_heading, self.tier_status, self.criteria_title, self.criteria_info, self.background] {
            fe.el(id).set_visible(show);
        }
        if show {
            self.tier_selected(fe);
            Self::title_to_menu(fe);
            let title = match fe.career.category {
                1 => Some("RACETYPE_UNDERGROUND_TITLE"),
                2 => Some("RACETYPE_PROFESSIONAL_TITLE"),
                3 => Some("RACETYPE_CHALENGES_TITLE"),
                4 => Some("RACETYPE_STORY_TITLE"),
                _ => None,
            };
            if let Some(t) = title {
                ov(fe, |o, fe| overlay::set_title(o, fe, string_crc(t)));
            }
        }
    }

    /// FEHangoutProgression.cpp 0x0028DD40 — one category entry (marked while it has open events)
    fn category_entry(&self, fe: &mut Feck2, i: usize) {
        let tier = (self.tier as u8) + 1;
        let cat = [1, 2, 3][i];
        let c = &fe.career;
        let mut unlocked = c.count(tier, cat, EVENT_FLAG_LOCKED, true);
        let mut completed = c.count(tier, cat, EVENT_FLAG_COMPLETED, false);
        if cat == 3 {
            unlocked += c.count(tier, 4, EVENT_FLAG_LOCKED, true);
            completed += c.count(tier, 4, EVENT_FLAG_COMPLETED, false);
        }
        let s = string_crc(CATEGORY_NAMES[i]);
        ov(fe, |o, fe| {
            if unlocked == completed {
                menu::add_entry(o, fe, s, true, false, 0);
            } else {
                menu::add_icon_entry(o, fe, s, true, "\\FE\\FE.vcd", "FE_Icon_RaceAvailable", None);
            }
        });
    }

    /// FEHangoutProgression.cpp 0x0028DE90 — the categories of this top level
    fn list_categories(&mut self, fe: &mut Feck2) {
        ov(fe, |o, fe| menu::clear(o, fe));
        // byte_6F0D70 (all events, debug) is never set
        if self.top == 2 {
            self.category_entry(fe, 0);
            self.category_entry(fe, 2);
            if self.last_dir != -1 {
                let s = self.category_sel;
                ov(fe, |o, fe| menu::set_selection(o, fe, s));
            }
        } else if self.top == 3 {
            self.category_entry(fe, 0);
            self.category_entry(fe, 1);
            self.category_entry(fe, 2);
            let done = fe.career.count((self.tier as u8) + 1, 2, EVENT_FLAG_COMPLETED, false);
            if self.last_dir != -1 {
                self.category_sel = (done == 0) as i32;
            }
            let s = self.category_sel;
            ov(fe, |o, fe| menu::set_selection(o, fe, s));
        }
        self.page = 0;
    }

    /// FEHangoutProgression.cpp 0x0028DFF0 — page 0: categories
    fn show_categories(&mut self, fe: &mut Feck2, show: bool) {
        if show {
            self.list_categories(fe);
        }
        Self::title_to_menu(fe);
        ov(fe, |o, fe| {
            menu::show(o, fe, show, false);
            menu::set_active(o, fe, show);
        });
        if show {
            Self::title_to_menu(fe);
            ov(fe, |o, fe| fe.el(o.title_text).set_visible(false));
        }
    }

    /// FEHangoutProgression.cpp 0x0028E0C0 — page 1: team / story challenges and team invites
    fn show_challenges(&mut self, fe: &mut Feck2, show: bool) {
        if show {
            let c = &fe.career;
            let (mut teams, mut teams_done) = (0, 0);
            let n = c.count(0, 3, EVENT_FLAG_LOCKED, true);
            for i in 0..n {
                if let Some(e) = c.nth(0, 3, i, EVENT_FLAG_LOCKED, true) {
                    if e.unlock_team != 0 {
                        teams += 1;
                        if e.flags & EVENT_FLAG_COMPLETED != 0 {
                            teams_done += 1;
                        }
                    }
                }
            }
            let story = c.count(0, 4, EVENT_FLAG_LOCKED, true);
            let story_done = c.count(0, 4, EVENT_FLAG_COMPLETED, false);
            let invites = c.count_teams_offering_invites(c.team) > 0;
            if self.last_dir == 1 {
                self.challenge_sel = 0;
            }
            let sel = self.challenge_sel;
            ov(fe, |o, fe| {
                menu::clear(o, fe);
                let entry = |o: &mut overlay::CFE2DOverlay, fe: &mut Feck2, id: &str, n: u32, done: u32| {
                    let s = string_crc(id);
                    if n == 0 {
                        menu::add_entry(o, fe, s, false, false, 0);
                    } else if n == done {
                        menu::add_entry(o, fe, s, true, false, 0);
                    } else {
                        menu::add_icon_entry(o, fe, s, true, "\\FE\\FE.vcd", "FE_Icon_RaceAvailable", None);
                    }
                };
                entry(o, fe, "TEAM_CHALLENGE", teams, teams_done);
                entry(o, fe, "STORY_CHALLENGE", story, story_done);
                menu::add_entry(o, fe, string_crc("TEAM_INVITES"), invites, false, 0);
                menu::show(o, fe, show, false);
                menu::set_active(o, fe, show);
                menu::set_selection(o, fe, sel);
                overlay::set_title(o, fe, string_crc("RACETYPE_CHALENGES_TITLE"));
            });
        }
        ov(fe, |o, fe| {
            menu::show(o, fe, show, false);
            menu::set_active(o, fe, show);
        });
    }

    /// FEHangoutProgression.cpp 0x0028E460 — page 7: team invites
    fn show_invites(&mut self, fe: &mut Feck2, show: bool) {
        ov(fe, |o, fe| {
            menu::show(o, fe, show, false);
            menu::set_active(o, fe, show);
        });
        let sel = self.invite_sel;
        self.show_invite(fe, sel);
    }

    /// FEHangoutProgression.cpp 0x0028E4B0 — page 7's menu: the teams offering invites (`GetTeamsOfferingInvites`
    /// 0x0016E980, cache of 10), selection 0
    fn fill_invites(&mut self, fe: &mut Feck2) {
        ov(fe, |o, fe| menu::clear(o, fe));
        let team = fe.career.team;
        let mut n = 0usize;
        for t in fe.career.teams.iter().filter(|t| t.is_offering_invite() && team != t.crc) {
            if n >= self.invite_cache.len() {
                log::warn!("WARNING: Team id cache appears to be too small when calling CTeamDatabase::GetTeamsOfferingInvites");
            } else {
                self.invite_cache[n] = t.crc;
                n += 1;
            }
        }
        self.invite_count = n as i32;
        if n > 0 {
            let names: Vec<u32> =
                self.invite_cache[..n].iter().map(|&c| fe.career.find_team(c).map_or(0, |t| t.name)).collect();
            ov(fe, |o, fe| {
                for &name in &names {
                    menu::add_entry(o, fe, name, true, false, 0);
                }
                menu::show(o, fe, true, false);
                menu::set_active(o, fe, true);
                menu::set_selection(o, fe, 0);
            });
        }
        self.invite_sel = 0;
    }

    /// FEHangoutProgression.cpp 0x0028E5F0 — the highlighted invite's team: name, bonus "%d", car, status "%d"
    /// (BENEFIT +204 is never set)
    fn show_invite(&mut self, fe: &mut Feck2, sel: i32) {
        for i in 0..5 {
            fe.el(self.team_labels[i]).set_visible(sel >= 0);
            fe.el(self.team_texts[i]).set_visible(sel >= 0);
        }
        if sel < 0 {
            return;
        }
        // the cache slot may be stale (count 0): the original reads the old team pointer
        let Some(t) = self.invite_cache.get(sel as usize).and_then(|&c| fe.career.find_team(c)).copied() else { return };
        let utf16 = |v: u32| format!("{}", v as i32).encode_utf16().collect::<Vec<u16>>();
        fe.text(self.team_texts[2]).set_text(&utf16(t.cash_bonus));
        fe.text(self.team_texts[4]).set_text(&utf16(t.status));
        fe.text(self.team_texts[0]).set_string(t.name);
        fe.text(self.team_texts[3]).set_string(t.car);
    }

    /// FEHangoutProgression.cpp 0x0028E4B0 + 0x0028E5F0 + 0x0028CC70 — back on page 7 with a fresh list
    fn back_to_invites(&mut self, fe: &mut Feck2) {
        self.page = 7;
        self.fill_invites(fe);
        self.invite_sel = 0;
        self.show_invite(fe, 0);
        self.show_page(fe);
    }

    /// FEHangoutProgression.cpp 0x00290BC0 — join a team: `dword_6F71D0`; the first time, its cash bonus
    /// (+ `0x0027CDD0`, `++dword_6F71D4`, joined-once flag). Returns the bonus paid.
    fn join_team(&mut self, fe: &mut Feck2, team: Option<u32>) -> u32 {
        let Some(t) = team.and_then(|c| fe.career.find_team(c)).copied() else {
            log::error!("assert 0 (FEHangoutProgression.cpp 2605): join without a team");
            fe.career.team = 0;
            fe.career.requests.push(CareerRequest::SetTeam { team: 0 });
            return 0;
        };
        fe.career.team = t.crc;
        fe.career.requests.push(CareerRequest::SetTeam { team: t.crc });
        let mut bonus = 0;
        if !t.joined_once_already() {
            bonus = t.cash_bonus;
            fe.garage.cash = fe.garage.cash.wrapping_add(bonus as i32);
            ov(fe, |o, fe| overlay::update_cash(o, fe));
            fe.career.requests.push(CareerRequest::CountTeamJoined);
            fe.career.team_request(TeamRequest::SetJoinedOnce(t.crc));
        }
        bonus
    }

    /// FEHangoutProgression.cpp 0x002909B0 — "joined <team>" (with the bonus when one was paid), 200 frames
    fn joined_message(&mut self, fe: &mut Feck2, team: Option<u32>, bonus: u32) {
        let name = team.and_then(|c| fe.career.find_team(c)).map_or(0, |t| t.name);
        let cash = team.and_then(|c| fe.career.find_team(c)).map_or(0, |t| t.cash_bonus);
        let s = if bonus != 0 {
            fe.strings.set_argument(TEAM_JOINED_BONUS, string_crc("idJoinedTeamName"), name).ok();
            fe.strings.set_argument(TEAM_JOINED_BONUS, string_crc("idCashBonus"), cash).ok();
            TEAM_JOINED_BONUS
        } else {
            fe.strings.set_argument(TEAM_JOINED, string_crc("idJoinedTeamName"), name).ok();
            TEAM_JOINED
        };
        ov(fe, |o, fe| {
            overlay::set_message(o, fe, s);
            overlay::set_message_timeout(o, MESSAGE_FRAMES);
            overlay::show_message(o, fe, true);
        });
    }

    /// FEHangoutProgression.cpp 0x00290AF0 — "leave <current> for <new>?" (no / yes)
    fn swap_prompt(fe: &mut Feck2, current: u32, new: u32) {
        fe.strings.set_argument(TEAM_SWAP, string_crc("idNewTeamName"), new).ok();
        fe.strings.set_argument(TEAM_SWAP, string_crc("idCurrentTeamName"), current).ok();
        ov(fe, |o, fe| {
            overlay::show_dialog(o, fe, 2, TEAM_SWAP, &[GENERAL_NO, GENERAL_YES]);
            overlay::set_dialog_visible(o, fe, true, 0);
        });
    }

    /// the swap prompt when the player has a team (asserts of 0x00290410 lines 2406 / 2411 logged); false = no team
    fn prompt_swap(&mut self, fe: &mut Feck2, line: u32) -> bool {
        if self.invite_team.is_none() {
            log::error!("assert mpTeamExtendingInvite != NULL (FEHangoutProgression.cpp {line})");
        }
        if fe.career.team == 0 {
            return false;
        }
        let current = fe.career.find_team(fe.career.team).copied();
        if current.is_none() {
            log::error!("assert pTeam != NULL (FEHangoutProgression.cpp {})", line + 5);
        }
        let new = self.invite_team.and_then(|c| fe.career.find_team(c)).map_or(0, |t| t.name);
        Self::swap_prompt(fe, current.map_or(0, |t| t.name), new);
        true
    }

    fn hide_dialog(fe: &mut Feck2) {
        ov(fe, |o, fe| overlay::set_dialog_visible(o, fe, false, 0));
    }

    /// FEHangoutProgression.cpp 0x00290410 — select on the invite dialog (selection 1 = yes)
    pub fn answer_invite(&mut self, fe: &mut Feck2) {
        let yes = fe.overlay.as_ref().is_some_and(|o| o.dialog_selection == 1);
        let team = self.invite_team;
        match self.invite {
            1 if yes => {
                if self.prompt_swap(fe, 2406) {
                    self.invite = 2;
                } else {
                    Self::hide_dialog(fe);
                    let bonus = self.join_team(fe, team);
                    self.joined_message(fe, team, bonus);
                    self.invite = 0;
                    self.page = 0;
                }
            }
            2 => {
                if yes {
                    let bonus = self.join_team(fe, team);
                    self.joined_message(fe, team, bonus);
                }
                self.close_invite(fe);
            }
            3 if yes => {
                if self.prompt_swap(fe, 2438) {
                    self.invite = 4;
                } else {
                    Self::hide_dialog(fe);
                    let bonus = self.join_team(fe, team);
                    self.joined_message(fe, team, bonus);
                    self.invite = 0;
                    if fe.career.count_teams_offering_invites(fe.career.team) == 0 {
                        self.page = 1;
                    } else {
                        self.page = 7;
                        self.fill_invites(fe);
                        self.invite_sel = 0;
                    }
                    self.show_invite(fe, 0);
                    self.show_page(fe);
                }
            }
            3 => {
                Self::hide_dialog(fe);
                self.invite = 0;
                self.back_to_invites(fe);
            }
            4 => {
                if yes {
                    let bonus = self.join_team(fe, team);
                    self.joined_message(fe, team, bonus);
                }
                self.invite = 0;
                Self::hide_dialog(fe);
                self.back_to_invites(fe);
            }
            _ => self.close_invite(fe),
        }
    }

    /// FEHangoutProgression.cpp 0x00290410 LABEL_45: no answer pending, page 0, dialog hidden
    fn close_invite(&mut self, fe: &mut Feck2) {
        self.invite = 0;
        self.page = 0;
        Self::hide_dialog(fe);
    }

    /// FEHangoutProgression.cpp 0x002908E0 — cancel on the invite dialog
    fn decline_invite(&mut self, fe: &mut Feck2) {
        match self.invite {
            1 | 2 => self.close_invite(fe),
            3 | 4 => {
                self.invite = 0;
                Self::hide_dialog(fe);
                self.back_to_invites(fe);
            }
            _ => {}
        }
    }

    /// FEHangoutProgression.cpp 0x0028E740 — the highlighted tier's counts, status and opening criteria
    fn tier_selected(&mut self, fe: &mut Feck2) {
        self.tier = Self::selected(fe);
        let (tier, cat) = ((self.tier as u8) + 1, fe.career.category);
        let c = &fe.career;
        let all = c.count(tier, cat, 0, false);
        let locked = c.count(tier, cat, EVENT_FLAG_LOCKED, false);
        let completed = c.count(tier, cat, EVENT_FLAG_COMPLETED, false);
        let first = c.nth(tier, cat, 0, 0, false).and_then(|e| e.unlock_event);
        Self::set_arg(fe, self.tier_values[0], "dUnlocked", all - locked);
        Self::set_arg(fe, self.tier_values[1], "dLocked", locked);
        Self::set_arg(fe, self.tier_values[2], "dCompleted", completed);
        Self::set_arg(fe, self.tier_heading, "dTier", tier as u32);
        let is_locked = locked == all;
        let status = if is_locked {
            "STATUS_LOCKED"
        } else if completed == all {
            "STATUS_COMPLETE"
        } else {
            "STATUS_OPEN"
        };
        Self::set_arg(fe, self.tier_status, "idStatus", string_crc(status));
        if is_locked {
            fe.el(self.criteria_title).set_visible(true);
            fe.el(self.criteria_info).set_visible(true);
            fe.text(self.criteria_info).set_string(string_crc("TIEROPENING_COMPLETECHALLENGE"));
            if let Some((crc, t)) = first {
                Self::set_arg(fe, self.criteria_info, "dTier", t as u32);
                Self::set_arg(fe, self.criteria_info, "idEvent", crc);
            }
        } else {
            fe.el(self.criteria_title).set_visible(false);
            fe.el(self.criteria_info).set_visible(false);
        }
    }

    /// FEHangoutProgression.cpp 0x0028EBA0 — the highlighted event becomes `m_pEvent`, its status and details
    fn event_selected(&mut self, fe: &mut Feck2) {
        self.event_sel = Self::selected(fe);
        let (tier, cat) = ((self.tier as u8) + 1, fe.career.category);
        let e = fe.career.nth(tier, cat, self.event_sel as u32, 0, false).cloned();
        self.browse = e.as_ref().map(|e| e.crc);
        if let Some(e) = e {
            if e.flags & EVENT_FLAG_LOCKED != 0 {
                for i in 0..2 {
                    fe.el(self.details[i]).set_visible(false);
                }
                fe.text(self.completed_text).set_string(string_crc("STATUS_LOCKED"));
            } else {
                for i in 0..2 {
                    fe.el(self.details[i]).set_visible(true);
                }
                let s = if e.flags & EVENT_FLAG_COMPLETED != 0 { "STATUS_COMPLETE" } else { "STATUS_OPEN" };
                fe.text(self.completed_text).set_string(string_crc(s));
                fe.el(self.locked_text).set_visible(false);
            }
            fe.el(self.completed_text).set_visible(true);
        }
        self.fill_details(fe);
    }

    /// FEHangoutProgression.cpp 0x0028EDA0 — while open: a pending screen change (team invite answers) counts down;
    /// false when it went
    pub fn update_exit(&mut self, fe: &mut Feck2) -> bool {
        if self.exit_countdown == -1 || self.exit_screen == 0 {
            return true;
        }
        if self.exit_countdown != 0 {
            self.exit_countdown -= 1;
            return true;
        }
        self.exit_countdown = -1;
        if self.exit_screen == string_crc("PREVIOUS") {
            fe.goto_previous_screen(1, 0);
        } else {
            fe.goto_screen(self.exit_screen);
        }
        false
    }

    /// FEHangoutProgression.cpp 0x0028EE90 — select: one page on (category / tier / event), page 6 starts the event
    pub fn select(&mut self, fe: &mut Feck2) {
        if self.page == 8 {
            self.answer_invite(fe);
            self.last_dir = 0;
            return;
        }
        self.last_dir = 1;
        let sel = Self::selected(fe);
        let set_category = |p: &Self, fe: &mut Feck2| {
            if p.top == 2 {
                if sel == 0 {
                    fe.career.category = 1;
                } else if sel == 1 {
                    fe.career.category = 3;
                }
            } else if p.top == 3 {
                fe.career.category = sel as u32 + 1;
            }
        };
        if self.page == 0 {
            set_category(self, fe);
            self.category_sel = sel;
        }
        if self.page != 7 {
            self.page += 1;
        }
        if self.page == 6 {
            if let Some(crc) = self.browse {
                self.start_event(fe, crc);
            }
            return;
        }
        if self.page == 1 {
            if self.top == 2 && sel == 0 {
                self.page = 4;
            } else if self.top == 3 && sel <= 1 {
                self.page = 3;
            }
        }
        match self.page {
            5 => {
                self.event_sel = sel;
                let (tier, cat) = ((self.tier as u8) + 1, fe.career.category);
                self.browse = fe.career.nth(tier, cat, sel as u32, 0, false).map(|e| e.crc);
            }
            3 => set_category(self, fe),
            2 => {
                if sel > 1 {
                    if sel == 2 {
                        self.page = 7;
                        self.fill_invites(fe);
                    }
                } else {
                    self.page += 1;
                    fe.career.category = if sel == 0 { 3 } else { 4 };
                }
                self.challenge_sel = sel;
            }
            7 => {
                // the invite question; joined before: no bonus line
                self.invite_sel = sel;
                self.invite_team = self.invite_cache.get(sel as usize).copied();
                let t = self.invite_team.and_then(|c| fe.career.find_team(c)).copied().unwrap_or_default();
                let title = if t.joined_once_already() {
                    fe.strings.set_argument(TEAM_JOIN, string_crc("idJoinTeamName"), t.name).ok();
                    TEAM_JOIN
                } else {
                    fe.strings.set_argument(TEAM_JOIN_BONUS, string_crc("idJoinTeamName"), t.name).ok();
                    fe.strings.set_argument(TEAM_JOIN_BONUS, string_crc("idCashBonus"), t.cash_bonus).ok();
                    TEAM_JOIN_BONUS
                };
                ov(fe, |o, fe| {
                    overlay::show_dialog(o, fe, 2, title, &[GENERAL_NO, GENERAL_YES]);
                    overlay::set_dialog_visible(o, fe, true, 0);
                });
                self.invite = 3;
                self.page = 8;
            }
            _ => {}
        }
        self.show_page(fe);
    }

    /// FEHangoutProgression.cpp 0x0028F270 — back one page, or close (then `m_pEvent` = 0)
    pub fn cancel(&mut self, fe: &mut Feck2) -> bool {
        if self.page == 8 {
            self.decline_invite(fe);
            self.last_dir = 0;
            return true;
        }
        self.last_dir = -1;
        let close = self.page == 0 || (self.page == 4 && self.top == 1) || (self.page == 5 && self.top == 0);
        if close {
            self.browse = None;
            if fe.career.event.is_some() {
                fe.career.requests.push(CareerRequest::ClearEvent);
            }
            self.open(fe, false);
            return false;
        }
        if self.page == 7 {
            self.page = 1;
            self.invite_sel = -1;
            self.show_invite(fe, -1);
        } else if self.page == 3 {
            self.page = (fe.career.category == 3 || fe.career.category == 4) as u32;
        } else {
            self.page -= 1;
        }
        self.show_page(fe);
        true
    }

    /// FEHangoutProgression.cpp 0x0028F3A0 — up / down moved the selection
    pub fn selection_changed(&mut self, fe: &mut Feck2) -> bool {
        match self.page {
            3 => self.tier_selected(fe),
            4 => self.event_selected(fe),
            7 => {
                let s = Self::selected(fe);
                self.show_invite(fe, s);
            }
            _ => {}
        }
        false
    }

    /// FEHangoutProgression.cpp 0x0028F430 — StartEvent: core runs the event set-up (competitor 0 human, reset,
    /// team / opponent car) when the loading screen asks for the event; the browser closes, post race
    /// "RaceCompleteScreen", return "HangoutScreen" (`dword_6F6FC8`).
    pub fn start_event(&mut self, fe: &mut Feck2, crc: u32) {
        fe.career.start_pending = Some(crc);
        fe.results_prompt = false; // CFrontendManager +113 (0x0028F6A9)
        self.open(fe, false);
        fe.post_race_screen = string_crc("RaceCompleteScreen");
        fe.return_screen = string_crc("HangoutScreen");
        self.started = true;
    }

    // ---- after a race ------------------------------------------------------------------------------------------

    /// FEHangoutProgression.cpp 0x0028F790 — one step per front end frame after an event; the steps whose message
    /// is still up wait. Step 8 clears the current event (`dword_6F9BE8 = 0`).
    pub fn update(&mut self, fe: &mut Feck2) -> u32 {
        // byte_6F5D08 (online) is never set
        if fe.career.event.is_none() {
            return 0;
        }
        if self.step != 0 && self.step != 8 && (message_showing(fe) || dialog_showing(fe)) {
            return self.step;
        }
        self.step += 1;
        match self.step {
            1 | 2 => self.feedback_movie(fe),
            3 => self.feedback_tips(fe),
            4 => self.feedback_team(fe),
            5 => self.feedback_result(fe),
            6 => self.feedback_tiers(fe),
            7 => self.feedback_vehicles(fe),
            8 => fe.career.requests.push(CareerRequest::ClearEvent),
            _ => {}
        }
        self.step
    }

    /// FEHangoutProgression.cpp 0x0028FBD0 — step 1 the event's movie (+0x4C, unless failed), step 2 the movie of
    /// the event unlocked last (`dword_6F7248`, then cleared)
    fn feedback_movie(&mut self, fe: &mut Feck2) {
        let Some(e) = fe.career.event.clone() else { return };
        let mut movie = 0;
        if self.step == 1 {
            if e.flags & EVENT_FLAG_FAILED == 0 && e.movie != 0 {
                movie = e.movie;
            }
        } else if self.step == 2 {
            movie = fe.career.unlock_message;
            fe.career.requests.push(CareerRequest::ClearUnlockMessage);
        }
        if movie != 0 {
            play_cut_movie(fe, movie);
        }
    }

    /// FEHangoutProgression.cpp 0x0028FD80 — step 3: the event's tip (+0x50) on TipsScreen
    fn feedback_tips(&mut self, fe: &mut Feck2) {
        let Some(e) = fe.career.event.clone() else { return };
        if e.tips != 0 {
            // FEHangoutScreen 0x0021C890, FEMainMenuScreen 0x0023F1F0
            ov(fe, |o, _| o.first_activate = true);
            crate::feck2::screens::common::fe_tips_screen::set_tip(fe, e.tips as i32);
            fe.goto_screen_name("TipsScreen");
        }
    }

    /// FEHangoutProgression.cpp 0x0028FE70 — step 4: a completed event that unlocks a team offers its invite
    fn feedback_team(&mut self, fe: &mut Feck2) {
        self.invite_team = None;
        let Some(e) = fe.career.event.clone() else { return };
        if e.flags & EVENT_FLAG_COMPLETED == 0 || e.unlock_team == 0 {
            return;
        }
        let Some(t) = fe.career.find_team(e.unlock_team).copied() else {
            log::warn!(
                "WARNING: CFEHangoutProgression::FeedbackTeamUnlocked() - unable to find team with the given id.\n\tCheck that the text ID in the events xls matches the text id in the ai teams/racers xls."
            );
            return;
        };
        if !t.is_newly_unlocked() {
            return;
        }
        fe.career.team_request(TeamRequest::ClearNewlyUnlocked(t.crc));
        self.invite_team = Some(t.crc);
        if t.joined_once_already() {
            log::error!("assert mpTeamExtendingInvite->JoinedOnceAlready() == false (FEHangoutProgression.cpp 2293)");
        }
        fe.strings.set_argument(TEAM_UNLOCKED, string_crc("idTeamName"), t.name).ok();
        fe.strings.set_argument(TEAM_UNLOCKED, string_crc("idCashBonus"), t.cash_bonus).ok();
        ov(fe, |o, fe| {
            overlay::show_dialog(o, fe, 2, TEAM_UNLOCKED, &[GENERAL_NO, GENERAL_YES]);
            overlay::set_dialog_visible(o, fe, true, 0);
        });
        self.invite = 1;
        self.page = 8;
    }

    /// FEHangoutProgression.cpp 0x0028F8C0 — step 5: Tory's comment on the result (one of two lines at random)
    fn feedback_result(&mut self, fe: &mut Feck2) {
        let Some(e) = fe.career.event.clone() else { return };
        let r = ((fe.char_rand.rand_unit() as f32) * 2.0) as i32 + 1;
        let mut positive = true;
        let (mut position, mut diff, mut ahead) = (0i32, 0i32, 0i32);
        let id = if e.category == 3 {
            if e.finish_position == 1 { "TEAM_CHALLENGE_WON".to_string() } else { "TEAM_CHALLENGE_LOST".to_string() }
        } else {
            if e.kind == 3 {
                let mine = e.points.first().copied().unwrap_or(0);
                if e.result_points != 0 {
                    positive = e.result_points <= mine;
                }
                for &p in e.points.iter().skip(1) {
                    if p > mine {
                        ahead += 1;
                    }
                }
                if e.result_position != 0 {
                    if e.result_position as i8 as i32 <= ahead {
                        positive = false;
                    }
                    diff = e.result_position as i8 as i32 - ahead;
                }
                if ahead == 0 {
                    position = 1;
                }
            }
            if e.kind == 1 || e.kind == 2 {
                let mut need = e.result_position as i8;
                if need == 0 {
                    need = 1;
                }
                position = e.finish_position as i32;
                if position > need as i32 {
                    positive = false;
                }
                diff = position - need as i32;
            }
            let level = if positive {
                if position == 1 {
                    "HIGHPOSITIVE"
                } else if diff != 0 {
                    "MEDPOSITIVE"
                } else {
                    "LOWPOSITIVE"
                }
            } else if diff == -1 {
                "HIGHNEGATIVE"
            } else if diff == -2 {
                "MEDNEGATIVE"
            } else {
                "LOWNEGATIVE"
            };
            format!("TORY_RETURNTOHANGOUT_{level}_{r}")
        };
        show_message(fe, string_crc(&id));
    }

    /// FEHangoutProgression.cpp 0x002900A0 — step 6: a category gained an unlocked tier (counts against
    /// `byte_6F9BE4`), then the counts are taken again (GamePlayer 0x000C1DA0)
    fn feedback_tiers(&mut self, fe: &mut Feck2) {
        let mut newly = [false; 3];
        let mut count = 0u8;
        for cat in 0..3u32 {
            count = 0;
            for t in 1..=5u8 {
                if fe.career.count(t, cat + 1, 0, false) > fe.career.count(t, cat + 1, EVENT_FLAG_LOCKED, false) {
                    count += 1;
                }
            }
            if count > fe.career.unlocked_tiers[cat as usize] {
                newly[cat as usize] = true;
            }
        }
        if newly.iter().any(|&n| n) {
            // the counts compared are the last category's (original)
            let id = if newly[1] && count == 1 {
                "TORY_PROFESSIONAL_AVAILABLE"
            } else if newly[2] && count == 1 {
                "TORY_CHALLENGES_AVAILABLE"
            } else {
                "TORY_TIERUNLOCKED"
            };
            show_message(fe, string_crc(id));
        }
        fe.career.requests.push(CareerRequest::CountUnlockedTiers);
    }

    /// FEHangoutProgression.cpp 0x00290340 — step 7: new cars in lock states 2 / 3
    fn feedback_vehicles(&mut self, fe: &mut Feck2) {
        let (progress, all) = (fe.quick_race.progress, fe.quick_race.all_unlocked);
        let Some(db) = fe.vehicles.as_mut() else { return };
        for i in 0..4 {
            db.update_lock_count(i, progress, all);
        }
        if db.get_lock_count_change(3) != 0 || db.get_lock_count_change(2) != 0 {
            show_message(fe, string_crc("TORY_VEHICLESUNLOCKED"));
        }
    }
}

/// `SetMessage(id)` (0x0027B200), `SetMessageTimeout(200)`, `ShowMessage(1)`
fn show_message(fe: &mut Feck2, id: u32) {
    ov(fe, |o, fe| {
        overlay::set_message_string(o, fe, id);
        overlay::set_message_timeout(o, MESSAGE_FRAMES);
        overlay::show_message(o, fe, true);
    });
}

/// FE3DCamera.cpp 0x0027B4F0 — (idb name) the overlay message is up
fn message_showing(fe: &Feck2) -> bool {
    fe.overlay.as_ref().is_some_and(|o| o.message_time >= 0.0)
}

/// FEOnlinePlayerOptionsScreen.cpp 0x0020B120 — (idb name)the overlay dialog is up
fn dialog_showing(fe: &Feck2) -> bool {
    fe.overlay.as_ref().is_some_and(|o| o.dialog_active)
}

/// what core's CompartmentManager 0x0008EDB0 / 0x0008EED0 filter on: category, tier (0 = any), flags set / clear
pub fn matches(e: &CareerEventView, tier: u8, category: u32, flags: u8, without: bool) -> bool {
    category == e.category
        && (tier == 0 || tier == e.tier)
        && (flags == 0 || if without { flags & e.flags == 0 } else { flags & e.flags != 0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// the argument names as `CCRC::CCRCChecked` checks them in 0x0028FE70 / 0x0028EE90 / 0x00290AF0 / 0x002909B0
    #[test]
    fn team_argument_crcs() {
        assert_eq!(string_crc("idTeamName"), 1505755211);
        assert_eq!(string_crc("idCashBonus"), -1449692238i32 as u32);
        assert_eq!(string_crc("idJoinTeamName"), 962033435);
        assert_eq!(string_crc("idNewTeamName"), -256038055i32 as u32);
        assert_eq!(string_crc("idCurrentTeamName"), 1861184354);
        assert_eq!(string_crc("idJoinedTeamName"), 1657834170);
        assert_eq!(string_crc("TEAM_INVITES"), -342428046i32 as u32);
    }
}
