//! `COSDRaceOverStatScreen` (`byte_6FD208`; code 0x000F0100..0x000F3800 in the OSDManager.cpp region of
//! objregions, class name from the memory tag "COSDRaceOverStatScreen::CreateTVStyleSummary") — the post race
//! results on the game OSD manager (`dword_6FD200`), stepped by GameManager 0x000BDFE0 (core's session PostRace
//! step) through 0x000F35C0. Page 2 = the race table 0x000F09C0 (position / name / time / prestige won, plus money
//! won, points won or qualified by event kind); page 1 = the TV-style summary 0x000F0450 (no items in this build);
//! page 3 = the per-kind summaries 0x000F2EC0: 0x000F14B0 single (statistics), 0x000F26B0 cup (bracket),
//! 0x000F1DA0 series (league).

use crate::feck2::feck2_manager::visos_colour;
use crate::font_helper::font_helper::CFontHelper;
use crate::osd::osd_manager::{flag, COSDManager, ItemId};
use crate::render2d::{Colour, TextureStore};
use crate::string_manager::string_manager::{string_crc, CStringManager};
use velocity_game::event_database::event_database::{CEvent, CEventCompetitor};

/// `dword_77FD84` = VISOS_Colour(0x80, 0x80, 0x80, 0) (static init 0x00431BD0): the title and header colour
pub const COLOUR_TITLE: Colour = 0xFFFF_FF00;
/// `dword_686044` — header column x (string items), also the table width by column count
const HEAD_X: [u32; 6] = [0x0000_0000, 0x42BE_0000, 0x4396_0000, 0x43F0_0000, 0x442C_8000, 0x4457_0000];
/// `dword_68602C` — text column x
const TEXT_X: [u32; 6] = [0x0000_0000, 0x4248_0000, 0x4396_0000, 0x43F0_0000, 0x442C_8000, 0x4457_0000];
/// the stack table at `ebp-0x1C..` (0x000F09C0): text justify per column (name left, the rest centred)
const JUSTIFY: [u32; 6] = [8, 2, 8, 8, 8, 8];
/// `dword_686078` — statistics header x (0x000F14B0; [6] = `unk_686090` the table width)
const STATS_HEAD_X: [u32; 7] = [0x0000_0000, 0x42BE_0000, 0x43AA_0000, 0x43ED_8000, 0x4422_8000, 0x4457_0000, 0x447A_0000];
/// `dword_68605C` — statistics text x
const STATS_TEXT_X: [u32; 7] = [0x0000_0000, 0x4248_0000, 0x43AA_0000, 0x43ED_8000, 0x4422_8000, 0x4457_0000, 0x447A_0000];
/// 0x000F14B0 stack `ebp-0x34..`: text justify per column
const STATS_JUSTIFY: [u32; 7] = [8, 2, 8, 8, 8, 8, 8];
/// `dword_6860A4` — league header x (0x000F1DA0; [3] = `unk_6860B0` the table width)
const LEAGUE_HEAD_X: [u32; 4] = [0x0000_0000, 0x42C8_0000, 0x43BB_8000, 0x4409_8000];
/// `dword_686094` — league text x
const LEAGUE_TEXT_X: [u32; 4] = [0x0000_0000, 0x4296_0000, 0x43BB_8000, 0x4409_8000];
/// 0x000F1DA0 stack `ebp-0x28..`: text justify per column
const LEAGUE_JUSTIFY: [u32; 4] = [8, 2, 8, 8];
/// every item of the screen: layer 0, permanent, centred + drop shadow, OSD mask 8
const LIFE: f32 = -1.0;
const MASK: u32 = 8;

/// one table row (16 bytes, 0x000F0560 init / 0x000F06A0 release): 6 columns of char[32] text (+4), string crc
/// (+8) and item (+12)
#[derive(Clone, Debug, Default)]
pub struct StatRow {
    pub texts: Vec<String>,
    pub crcs: Vec<u32>,
    pub items: Vec<Option<ItemId>>,
}

impl StatRow {
    /// OSDManager.cpp 0x000F0560
    fn new(n: usize) -> Self {
        Self { texts: vec![String::new(); n], crcs: vec![0; n], items: vec![None; n] }
    }
}

/// OSDManager.cpp 0x000F3750 — the stat table (0x20 bytes, ctor = `Default`)
#[derive(Clone, Debug, Default)]
pub struct StatTable {
    pub title: Option<ItemId>,          // +0
    pub tv_items: Vec<Option<ItemId>>,  // +4 count, +8 items (+12 their buffers)
    pub rows: Vec<StatRow>,             // +16 count, +24
    /// +20 count, +28: the cup bracket's rows (0x000F26B0, rows of 0x000F0790); 0x000F0270 does not fade them
    pub bracket: Vec<StatRow>,
}

impl StatTable {
    /// OSDManager.cpp 0x000F0100 / 0x000F3650 / 0x000F06A0 — release every item
    fn release(self, osd: &mut COSDManager) {
        if let Some(t) = self.title {
            osd.release(t);
        }
        for it in self.tv_items.into_iter().flatten() {
            osd.release(it);
        }
        for r in self.rows.into_iter().chain(self.bracket) {
            for it in r.items.into_iter().flatten() {
                osd.release(it);
            }
        }
    }

    fn each_item(&self) -> impl Iterator<Item = ItemId> + '_ {
        self.title
            .into_iter()
            .chain(self.tv_items.iter().flatten().copied())
            .chain(self.rows.iter().flat_map(|r| r.items.iter().flatten().copied()))
    }
}

/// what 0x000F09C0 reads of a finished racer (`VsAIWorld::GetVehicleInPosition(place)`): the driver name
/// (VsAIDriver 0x000153B0) and the record block 0x000244E0 (`stats` of the ai race vehicle)
#[derive(Clone, Debug, Default)]
pub struct RacerResult {
    pub name: String,
    pub time: f32,        // +4 finish time
    pub points: u32,      // +84 prestige / score
    pub top_speed: f32,   // +0 m/s
    pub fastest_lap: f32, // +44
    pub lead_secs: f32,   // +68 seconds in the lead
    pub overtaken: u32,   // +72
    pub grid: u8,         // +88 grid position
}

/// the event fields 0x000F09C0 reads of `dword_6F9BE8` (`CEvent`)
#[derive(Clone, Copy, Debug, Default)]
pub struct ResultsEvent {
    pub category: u32,           // +0x00
    pub kind: u32,               // +0x04 1 single, 2 cup, 3 series
    pub per_cup_race: u8,        // +0x17
    pub competitors: u8,         // +0x24 count byte
    pub prize: f32,              // +0x30
}

/// the race results the table is built from, by place (index 0 = 1st)
#[derive(Clone, Debug, Default)]
pub struct RaceResults {
    pub racers: Vec<RacerResult>,
    /// `*VsAIWorld::GetPlayerVehicle(0)` (RaceInfo.dPlayerPos[0], 1-based)
    pub player_place: u32,
    pub event: Option<ResultsEvent>,
    /// `byte_6F7158` (online: world rank column, police names)
    pub online: bool,
    /// the whole `dword_6F9BE8` for page 3 (cup rounds / races +0x20, competitors +0x24)
    pub event_full: Option<CEvent>,
    /// per event competitor: the opponent's name (competitor +4 = the opponent record, name at +0); "" for humans
    pub competitor_names: Vec<String>,
}

/// `byte_6FD208`
#[derive(Debug, Default)]
pub struct COSDRaceOverStatScreen {
    pub table: Option<StatTable>,   // +0
    pub page: u32,                  // +4
    /// +8: 4 select held, 8 race table, 0x20 faded, 0x40 / 0x80 TV summary
    pub flags: u8,
    pub timer: f32,                 // +12
    pub text_item: Option<ItemId>,  // +16
    /// +20 char buffer: static bss, never written (F3220's GetPlayerVehicle result is unused in this build)
    pub text: String,
    pub item_84: Option<ItemId>,    // +84
    pub item_88: Option<ItemId>,    // +88
    /// `dword_6F7244` written by the race table (the player's +84), taken by the caller
    pub player_points: Option<u32>,
}

/// game-side context of one results update
pub struct ResultsCtx<'a> {
    pub osd: &'a mut COSDManager,
    pub fonts: &'a CFontHelper,
    pub strings: &'a CStringManager,
    pub tex: &'a TextureStore,
    /// UserInput 0x001E6BC0(action 6 MenuSelect, GameInput 0x000A59D0 mask of player 0)
    pub select: f32,
    pub results: &'a RaceResults,
}

fn truncate31(mut s: String) -> String {
    // sprintf into char[32] (`sub_3DDD20(buf, 32, ...)`)
    while s.len() > 31 {
        s.pop();
    }
    s
}

impl COSDRaceOverStatScreen {
    /// OSDManager.cpp 0x000F3070 — ctor
    pub fn new() -> Self {
        Self::default()
    }

    /// OSDManager.cpp 0x000F35C0 — one step of GameManager 0x000BDFE0; true = the step is done
    pub fn step(&mut self, step: u32, cx: &mut ResultsCtx) -> bool {
        match step {
            0 => {
                self.reset(cx.osd);
                self.show_text(cx);
                false
            }
            1 => self.select_pressed(cx.select),
            2 => {
                self.show_race_table(cx);
                false
            }
            3 => self.page_update(cx),
            _ => false,
        }
    }

    /// OSDManager.cpp 0x000F3130 — release everything, page 0, flags 7, OSD mask `&= ~8` (0x00023D00); also called
    /// by GameManager::RestartRace 0x000BE0F0 and EndOfGame 0x000BE540
    pub fn reset(&mut self, osd: &mut COSDManager) {
        if let Some(t) = self.table.take() {
            t.release(osd);
        }
        for it in [self.text_item.take(), self.item_84.take(), self.item_88.take()].into_iter().flatten() {
            osd.release(it);
        }
        self.page = 0;
        self.flags = 7;
        osd.hidden_mask &= !MASK;
    }

    /// OSDManager.cpp 0x000F3220 — the +20 text, font 10 at (666, 650)
    fn show_text(&mut self, cx: &mut ResultsCtx) {
        let id = cx.osd.create_text_item(cx.fonts, cx.tex, &self.text, 10, 666.0, 650.0, 0, LIFE, flag::SHADOW | flag::JUSTIFY_CENTRE_X, MASK);
        self.text_item = Some(id);
    }

    /// OSDManager.cpp 0x000F32B0 — select went down this update (flag 4 = held)
    fn select_pressed(&mut self, select: f32) -> bool {
        let mut pressed = false;
        if select <= 0.5 {
            self.flags &= !4;
        } else {
            if self.flags & 4 == 0 {
                pressed = true;
            }
            self.flags |= 4;
        }
        pressed
    }

    /// OSDManager.cpp 0x000F3360 — text off, page 2 (the race table)
    fn show_race_table(&mut self, cx: &mut ResultsCtx) {
        if let Some(t) = self.table.take() {
            t.release(cx.osd);
        }
        self.page = 0;
        if let Some(it) = self.text_item.take() {
            cx.osd.release(it);
        }
        self.set_page(2, cx);
    }

    /// OSDManager.cpp 0x000F2F40 — build page `page` (4 = none, keeps the page number)
    fn set_page(&mut self, page: u32, cx: &mut ResultsCtx) {
        if self.page == page {
            return;
        }
        if let Some(t) = self.table.take() {
            t.release(cx.osd);
        }
        if page == 4 {
            return;
        }
        self.flags &= !0x40;
        self.flags &= !0x80;
        match page {
            1 => {
                self.table = Some(tv_style_summary());
                self.flags |= 0x40;
                self.flags |= 0x80;
            }
            2 => {
                let (t, pts) = race_table(cx);
                self.table = Some(t);
                if pts.is_some() {
                    self.player_points = pts;
                }
                self.flags |= 8;
            }
            3 => self.table = event_summary(cx),
            _ => {}
        }
        self.page = page;
    }

    /// OSDManager.cpp 0x000F3400 — select pages on (true after the last), the table fades after 10 s
    fn page_update(&mut self, cx: &mut ResultsCtx) -> bool {
        let mut pressed = false;
        let mut next = self.page;
        if cx.select <= 0.5 {
            self.flags &= !4;
        } else {
            if self.flags & 4 == 0 {
                pressed = true;
            }
            self.flags |= 4;
        }
        if pressed {
            next = self.page + 1;
            if next == 4 {
                return true;
            }
        }
        if pressed {
            self.timer = 0.0;
            if self.flags & 0x20 != 0 {
                self.fade(cx.osd, 1.0);
                self.flags &= !0x20;
            } else {
                self.set_page(next, cx);
            }
        } else if self.flags & 0x80 == 0 && self.timer > 10.0 {
            let a = ((15.0 - self.timer) / 5.0).clamp(0.1, 1.0);
            self.fade(cx.osd, a);
            self.flags |= 0x20;
        }
        self.timer += f32::from_bits(0x3D08_8889); // 0.033333335
        false
    }

    /// OSDManager.cpp 0x000F0270 — every table item's alpha = `ftol(128 * a)` (VISOS_Colour doubles every channel
    /// again, so the 0..255 colour bytes stay clamped at 255 or 0 — original)
    fn fade(&mut self, osd: &mut COSDManager, a: f32) {
        let Some(t) = self.table.as_ref() else { return };
        let alpha = (128.0f64 * a as f64) as i32 as u32;
        for id in t.each_item() {
            let it = osd.item_mut(id);
            let c = it.colour;
            it.colour = visos_colour(alpha & 0xFF, (c >> 16) & 0xFF, (c >> 8) & 0xFF, c & 0xFF);
        }
    }
}

/// OSDManager.cpp 0x000F0450 — `CreateTVStyleSummary`: six empty slots, no items in this build
fn tv_style_summary() -> StatTable {
    StatTable { title: None, tv_items: vec![None; 6], ..Default::default() }
}

/// OSDManager.cpp 0x000F09C0 / 0x000F37B0 — the race table (`OSD_STATSTITLE_RACE`); returns the player's +84 (`dword_6F7244`)
pub fn race_table(cx: &mut ResultsCtx) -> (StatTable, Option<u32>) {
    let res = cx.results;
    let mut t = StatTable::default();
    t.title = cx.osd.create_string_item(cx.fonts, cx.strings, cx.tex, string_crc("OSD_STATSTITLE_RACE"), 9, 666.0, 800.0, 0, LIFE, flag::SHADOW | flag::JUSTIFY_CENTRE_X, MASK);
    if let Some(id) = t.title {
        cx.osd.item_mut(id).colour = COLOUR_TITLE;
    }
    let racing = res.racers.len();
    t.rows = vec![StatRow::default(); racing + 2];
    t.rows[0] = StatRow::new(6);
    t.rows[1] = StatRow::new(6);
    let head = &mut t.rows[0].crcs;
    head[0] = string_crc("OSD_STATS_POSTIONS");
    head[1] = string_crc("OSD_STATS_NAME");
    head[2] = string_crc("OSD_STATS_TIME");
    head[3] = string_crc("OSD_STATS_PRESTIGEWON");
    let mut count = 3usize;
    if res.online {
        head[5] = string_crc("OSD_STATS_WORLDRANK");
        count += 1;
    }
    let ev = res.event;
    let kind = ev.map_or(0, |e| e.kind);
    let money = kind == 1 && ev.is_some_and(|e| e.prize != 0.0);
    if money {
        head[4] = string_crc("OSD_STATS_MONEYWON");
        count += 1;
    } else if kind == 3 {
        head[4] = string_crc("OSD_STATS_POINTSWON");
        count += 1;
    } else if kind == 2 {
        head[4] = string_crc("OSD_STATS_QUALIFIED_STATUS");
        count += 1;
    }
    let start = (1333.0 - f32::from_bits(HEAD_X[count])) / 2.0;
    let mut player_points = None;
    for place in 1..=racing as u32 {
        let r = &res.racers[place as usize - 1];
        let mut row = StatRow::new(6);
        row.texts[0] = format!("{}", place);
        row.texts[1] = if place == res.player_place {
            "PLAYER".to_string()
        } else {
            // online: the police vehicle's name (+5092) — no network game here
            truncate31(r.name.clone())
        };
        row.texts[2] = if r.time >= 0.1 {
            let min = ((r.time as f64 / 60.0) as f32) as i32;
            let sec = (r.time as i32) % 60;
            if sec as u32 >= 10 { format!("{}:{}", min, sec) } else { format!("{}:0{}", min, sec) }
        } else {
            "NA".to_string()
        };
        row.texts[3] = format!("{}", r.points as i32);
        if money {
            let e = ev.unwrap_or_default();
            // CompartmentManager 0x0008EA60 gets the 1-based place where it expects a 0-based one (original)
            let cash = velocity_game::event_database::event_database_bin_loading::prize_money(e.prize, place as u8, e.competitors, e.category);
            row.texts[4] = format!("{}", cash as i32);
        } else if kind == 3 {
            row.texts[4] = format!("{}", velocity_game::event_database::event_database_bin_loading::points_for_position(place as u8));
        } else if kind == 2 {
            let per = ev.map_or(0, |e| e.per_cup_race) as u32;
            row.crcs[4] = string_crc(if place <= per / 2 { "OSD_STATS_QUALIFIED_YES" } else { "OSD_STATS_QUALIFIED_NO" });
        }
        if res.online {
            row.texts[5] = format!("{}", place);
        }
        if place == res.player_place {
            player_points = Some(r.points);
        }
        t.rows[place as usize + 1] = row;
    }
    let white = visos_colour(0xFF, 0x80, 0x80, 0x80);
    for (ri, row) in t.rows.iter_mut().enumerate() {
        let y = (660.0f64 - ri as f64 * 45.0) as f32;
        for c in 0..row.crcs.len() {
            if row.crcs[c] != 0 {
                let x = start + f32::from_bits(HEAD_X[c]);
                row.items[c] = cx.osd.create_string_item(cx.fonts, cx.strings, cx.tex, row.crcs[c], 11, x, y, 0, LIFE, flag::SHADOW | flag::JUSTIFY_CENTRE_X, MASK);
            } else if !row.texts[c].is_empty() {
                let x = start + f32::from_bits(TEXT_X[c]);
                row.items[c] = Some(cx.osd.create_text_item(cx.fonts, cx.tex, &row.texts[c], 11, x, y, 0, LIFE, JUSTIFY[c] | flag::SHADOW, MASK));
            }
            if let Some(id) = row.items[c] {
                cx.osd.item_mut(id).colour = if ri != 0 { white } else { COLOUR_TITLE };
            }
        }
    }
    // the world rank column is disabled offline (0x000F37B0(0))
    for row in &t.rows {
        if !res.online {
            if let Some(id) = row.items.get(5).copied().flatten() {
                cx.osd.item_mut(id).flags |= flag::DISABLED;
            }
        }
    }
    (t, player_points)
}

/// "%d:%d" / "%d:0%d" of a time in seconds (`(int)(float)(t / 60.0)`, `(int)t % 60`, unsigned `>= 10`)
fn min_sec(t: f32) -> String {
    let min = ((t as f64 / 60.0) as f32) as i32;
    let sec = (t as i32) % 60;
    if sec as u32 >= 10 { format!("{}:{}", min, sec) } else { format!("{}:0{}", min, sec) }
}

/// the item loop shared by 0x000F14B0 / 0x000F1DA0: font 11, row r at y 660 - 45 r, strings at start + head x
/// (0x2008), texts at start + text x (justify | 0x2000), row 0 `dword_77FD84`, the rest VISOS(0xFF,0x80,0x80,0x80)
fn create_table_items(cx: &mut ResultsCtx, rows: &mut [StatRow], start: f32, head_x: &[u32], text_x: &[u32], justify: &[u32]) {
    let white = visos_colour(0xFF, 0x80, 0x80, 0x80);
    for (ri, row) in rows.iter_mut().enumerate() {
        let y = (660.0f64 - ri as f64 * 45.0) as f32;
        for c in 0..row.crcs.len() {
            if row.crcs[c] != 0 {
                let x = start + f32::from_bits(head_x[c]);
                row.items[c] = cx.osd.create_string_item(cx.fonts, cx.strings, cx.tex, row.crcs[c], 11, x, y, 0, LIFE, flag::SHADOW | flag::JUSTIFY_CENTRE_X, MASK);
            } else if !row.texts[c].is_empty() {
                let x = start + f32::from_bits(text_x[c]);
                row.items[c] = Some(cx.osd.create_text_item(cx.fonts, cx.tex, &row.texts[c], 11, x, y, 0, LIFE, justify[c] | flag::SHADOW, MASK));
            }
            if let Some(id) = row.items[c] {
                cx.osd.item_mut(id).colour = if ri != 0 { white } else { COLOUR_TITLE };
            }
        }
    }
}

/// the page title: font 9 at (666, 800), 0x2008, `dword_77FD84`
fn create_title(cx: &mut ResultsCtx, name: &str) -> Option<ItemId> {
    let id = cx.osd.create_string_item(cx.fonts, cx.strings, cx.tex, string_crc(name), 9, 666.0, 800.0, 0, LIFE, flag::SHADOW | flag::JUSTIFY_CENTRE_X, MASK);
    if let Some(id) = id {
        cx.osd.item_mut(id).colour = COLOUR_TITLE;
    }
    id
}

/// OSDManager.cpp 0x000F2EC0 — page 3 by event kind (`dword_6F9BE8` +4); no event = no table (the original reads
/// through the null event)
pub fn event_summary(cx: &mut ResultsCtx) -> Option<StatTable> {
    match cx.results.event_full.as_ref()?.kind {
        1 => Some(statistics_table(cx)),
        2 => Some(cup_table(cx)),
        3 => Some(league_table(cx)),
        _ => None,
    }
}

/// OSDManager.cpp 0x000F14B0 — single event: `OSD_STATSTITLE_STATISTICS`, per place overtaken / seconds in the lead
/// / fastest lap / top speed / grid position
pub fn statistics_table(cx: &mut ResultsCtx) -> StatTable {
    let res = cx.results;
    let mut t = StatTable { title: create_title(cx, "OSD_STATSTITLE_STATISTICS"), ..Default::default() };
    let racing = res.racers.len();
    t.rows = vec![StatRow::default(); racing + 2];
    t.rows[0] = StatRow::new(7);
    t.rows[1] = StatRow::new(7);
    let head = &mut t.rows[0].crcs;
    for (c, s) in ["OSD_STATS_POSTIONS", "OSD_STATS_NAME", "OSD_STATS_OVERTAKEN", "OSD_STATS_SECSINLEAD", "OSD_STATS_FASTESTLAP", "OSD_STATS_TOPSPEED", "OSD_STATS_GRIDPOSITION"].iter().enumerate() {
        head[c] = string_crc(s);
    }
    for place in 1..=racing as u32 {
        let r = &res.racers[place as usize - 1];
        let mut row = StatRow::new(7);
        row.texts[0] = format!("{}", place);
        // online: the police vehicle's name (+5092) — no network game here
        row.texts[1] = if place == res.player_place { "PLAYER".to_string() } else { truncate31(r.name.clone()) };
        row.texts[2] = format!("{}", r.overtaken as i32);
        row.texts[3] = min_sec(r.lead_secs);
        row.texts[4] = if r.fastest_lap >= 0.1 { min_sec(r.fastest_lap) } else { "NA".to_string() };
        let kmh = r.top_speed * f32::from_bits(0x4066_6666); // 3.6f
        row.texts[5] = format!("{}mph", ((kmh as f64 * 0.625) as f32) as i32);
        row.texts[6] = format!("{}", r.grid);
        t.rows[place as usize + 1] = row;
    }
    let start = (1333.0 - f32::from_bits(STATS_HEAD_X[6])) / 2.0;
    create_table_items(cx, &mut t.rows, start, &STATS_HEAD_X, &STATS_TEXT_X, &STATS_JUSTIFY);
    t
}

/// OSDManager.cpp 0x000F1DA0 — series: `OSD_STATSTITLE_LEAGUE`, the first `racing` event competitors ranked by
/// points (+8), then prestige total (+0x0C), then a human first, then competitor order
pub fn league_table(cx: &mut ResultsCtx) -> StatTable {
    let res = cx.results;
    let mut t = StatTable { title: create_title(cx, "OSD_STATSTITLE_LEAGUE"), ..Default::default() };
    let racing = res.racers.len();
    t.rows = vec![StatRow::default(); racing + 2];
    t.rows[0] = StatRow::new(4);
    t.rows[1] = StatRow::new(4);
    let head = &mut t.rows[0].crcs;
    for (c, s) in ["OSD_STATS_POSTIONS", "OSD_STATS_NAME", "OSD_STATS_PRESTIGETOTAL", "OSD_STATS_TOTALPOINTS"].iter().enumerate() {
        head[c] = string_crc(s);
    }
    // the competitor array is read up to the racing count (beyond it the original reads past the array)
    let none = CEventCompetitor::default();
    let comps: &[CEventCompetitor] = res.event_full.as_ref().and_then(|e| e.competitors.as_deref()).unwrap_or(&[]);
    let comp = |i: usize| comps.get(i).unwrap_or(&none);
    let mut rank = vec![0u32; racing];
    for (i, rk) in rank.iter_mut().enumerate() {
        let me = comp(i);
        let mut ahead = 0u32;
        for j in 0..racing {
            if i == j {
                continue;
            }
            let o = comp(j);
            if o.points > me.points {
                ahead += 1;
            } else if o.points == me.points {
                if o.unknown_0c > me.unknown_0c {
                    ahead += 1;
                } else if o.unknown_0c == me.unknown_0c && (o.human != 0 || i > j) {
                    ahead += 1;
                }
            }
        }
        *rk = ahead + 1;
    }
    for i in 0..racing {
        let c = comp(i);
        // two humans level on points and prestige share a rank: the later one overwrites the row (original)
        let r = rank[i] as usize + 1;
        let mut row = StatRow::new(4);
        row.texts[0] = format!("{}", rank[i]);
        row.texts[1] = if c.human != 0 {
            "PLAYER".to_string()
        } else {
            truncate31(res.competitor_names.get(i).cloned().unwrap_or_default())
        };
        row.texts[2] = format!("{}", c.unknown_0c as i32);
        row.texts[3] = format!("{}", c.points as i32);
        t.rows[r] = row;
    }
    let start = (1333.0 - f32::from_bits(LEAGUE_HEAD_X[3])) / 2.0;
    create_table_items(cx, &mut t.rows, start, &LEAGUE_HEAD_X, &LEAGUE_TEXT_X, &LEAGUE_JUSTIFY);
    t
}

/// OSDManager.cpp 0x000F26B0 / 0x000F0790 — cup (+ the bracket rows): `OSD_STATSTITLE_RACE` over a bracket, one column per round up to the current
/// one (x = 300 (round + 1)), one block of `per race` names per race; a finished / failed event adds
/// `OSD_STATS_WINNERHEADING` and the previous round's first-race winner. Rows go to +28 (not faded).
pub fn cup_table(cx: &mut ResultsCtx) -> StatTable {
    let res = cx.results;
    let mut t = StatTable { title: create_title(cx, "OSD_STATSTITLE_RACE"), ..Default::default() };
    let Some(ev) = res.event_full.as_ref() else { return t };
    let none = Default::default();
    let races = ev.races.as_ref().unwrap_or(&none);
    let rounds = races.rounds.len() as u32; // +0x20 +1
    let cur = races.current_round as u32; // +0x20 +2
    let comps: &[CEventCompetitor] = ev.competitors.as_deref().unwrap_or(&[]);
    let total = comps.len() as u32; // *(+0x24)
    let per = ev.competitors_per_cup_race as u32; // +0x17
    // per == 0 divides by zero in the original
    let div = |a: u32| if per == 0 { 0 } else { a / per };
    let mut count = 0u32;
    let mut left = total;
    for _ in 0..=cur {
        count += div(left);
        left >>= 1;
    }
    let finished = ev.flags & (2 | 4) != 0;
    if finished {
        count += 1;
    }
    t.bracket = vec![StatRow::default(); count as usize];
    // the rows are made with (total / per) / (round + 1) races per round but counted with (total >> round) / per:
    // they differ from round 2 on, where the original writes past the array (grown here)
    let row_at = |b: &mut Vec<StatRow>, r: usize| {
        if r >= b.len() {
            b.resize(r + 1, StatRow::default());
        }
    };
    let mut r = 0usize;
    for i in 0..=cur {
        for _ in 0..div(total) / (i + 1) {
            row_at(&mut t.bracket, r);
            t.bracket[r] = StatRow::new(per as usize);
            r += 1;
        }
    }
    let humans: Vec<bool> = comps.iter().map(|c| c.human != 0).collect();
    let name = |c: usize| -> String {
        if comps.get(c).is_some_and(|c| c.human != 0) {
            "PLAYER".to_string()
        } else {
            res.competitor_names.get(c).cloned().unwrap_or_default()
        }
    };
    r = 0;
    for i in 0..=cur {
        let round = races.rounds.get(i as usize);
        let n = div(total) / (i + 1);
        for j in 0..n {
            let race = round.and_then(|rd| rd.races.get(j as usize));
            let x = (300 * (i + 1)) as f32;
            for k in 0..per {
                let up = 200 * ((j + 1) >> 1);
                let y = if j >= n >> 1 {
                    600.0f64 - k as f64 * 45.0 - up as f64
                } else {
                    (per - k) as f64 * 45.0 + 600.0 - up as f64
                } as f32;
                // EventDatabaseEvents 0x00092B50
                let has_human = round.is_some_and(|rd| rd.has_human(&humans));
                let mut show = false;
                if has_human {
                    if j == 0 {
                        show = true;
                    }
                    if i == cur {
                        show = ev.flags & 0x10 != 0;
                    }
                }
                let comp = race.and_then(|rc| if show { rc.competitor_in_position(k as u8 + 1) } else { rc.competitors.get(k as usize).copied().flatten() });
                let row = &mut t.bracket[r];
                let item = if let Some(c) = comp {
                    row.texts[k as usize] = name(c);
                    Some(cx.osd.create_text_item(cx.fonts, cx.tex, &row.texts[k as usize], 11, x, y, 0, LIFE, flag::SHADOW | flag::JUSTIFY_CENTRE_X, MASK))
                } else {
                    row.crcs[k as usize] = string_crc("test");
                    cx.osd.create_string_item(cx.fonts, cx.strings, cx.tex, row.crcs[k as usize], 11, x, y, 0, LIFE, flag::SHADOW | flag::JUSTIFY_CENTRE_X, MASK)
                };
                row.items[k as usize] = item;
                let yellow = if k >= per >> 1 || i >= cur {
                    i as i64 == rounds as i64 - 1 && k == 0 && (show || !has_human)
                } else {
                    true
                };
                if let (true, Some(id)) = (yellow, item) {
                    cx.osd.item_mut(id).colour = COLOUR_TITLE;
                }
            }
            r += 1;
        }
    }
    if finished {
        let x = (300 * (cur + 2)) as f32;
        row_at(&mut t.bracket, r);
        let mut row = StatRow::new(2);
        row.crcs[0] = string_crc("OSD_STATS_WINNERHEADING");
        row.items[0] = cx.osd.create_string_item(cx.fonts, cx.strings, cx.tex, row.crcs[0], 11, x, 600.0 + 45.0, 0, LIFE, flag::SHADOW | flag::JUSTIFY_CENTRE_X, MASK);
        if let Some(id) = row.items[0] {
            cx.osd.item_mut(id).colour = COLOUR_TITLE;
        }
        // the previous round's (+0x20 +0x10) first race, place 1 (a null competitor crashes the original)
        let winner = races
            .previous_round
            .and_then(|p| races.rounds.get(p as usize))
            .and_then(|rd| rd.races.first())
            .and_then(|rc| rc.competitor_in_position(1));
        row.texts[1] = winner.map(name).unwrap_or_default();
        let y = ((0.0f64 - 45.0) as f32 as f64 + 600.0) as f32;
        row.items[1] = Some(cx.osd.create_text_item(cx.fonts, cx.tex, &row.texts[1], 11, x, y, 0, LIFE, flag::SHADOW | flag::JUSTIFY_CENTRE_X, MASK));
        t.bracket[r] = row;
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(osd: &'a mut COSDManager, fonts: &'a CFontHelper, strings: &'a CStringManager, tex: &'a TextureStore, res: &'a RaceResults, select: f32) -> ResultsCtx<'a> {
        ResultsCtx { osd, fonts, strings, tex, select, results: res }
    }

    #[test]
    fn race_table_columns_and_steps() {
        let fonts = CFontHelper::default();
        let strings = CStringManager::new();
        let tex = TextureStore::default();
        let mut osd = COSDManager::default();
        let res = RaceResults {
            racers: vec![
                RacerResult { name: "Ai One".into(), time: 125.4, points: 340, ..Default::default() },
                RacerResult { name: "Ai Two".into(), time: 0.0, points: 7, ..Default::default() },
                RacerResult { name: "x".into(), time: 61.0, points: 1200, ..Default::default() },
            ],
            player_place: 3,
            event: Some(ResultsEvent { kind: 3, ..Default::default() }),
            online: false,
            ..Default::default()
        };
        let mut s = COSDRaceOverStatScreen::new();
        assert!(!s.step(0, &mut ctx(&mut osd, &fonts, &strings, &tex, &res, 1.0)));
        // A still held from the race: no press until it is let go
        assert!(!s.step(1, &mut ctx(&mut osd, &fonts, &strings, &tex, &res, 1.0)));
        assert!(!s.step(1, &mut ctx(&mut osd, &fonts, &strings, &tex, &res, 0.0)));
        assert!(s.step(1, &mut ctx(&mut osd, &fonts, &strings, &tex, &res, 1.0)));
        s.step(2, &mut ctx(&mut osd, &fonts, &strings, &tex, &res, 1.0));
        let t = s.table.as_ref().unwrap();
        assert_eq!(t.rows.len(), 5);
        assert!(t.rows[1].items.iter().all(|i| i.is_none()));
        assert_eq!(t.rows[2].texts[..4], ["1", "Ai One", "2:05", "340"]);
        assert_eq!(t.rows[3].texts[2], "NA");
        assert_eq!(t.rows[4].texts[..5], ["3", "PLAYER", "1:01", "1200", "5"]);
        assert_eq!(s.player_points, Some(1200));
        let it = osd.items[t.rows[4].items[3].unwrap()].clone();
        // 4 columns: start (1333 - 690) / 2, prestige text at +480, row 4 at 660 - 180
        assert_eq!((it.x, it.y, it.colour), (321.5 + 480.0, 480.0, 0xFFFF_FFFF));
        assert_eq!(osd.items[t.rows[4].items[1].unwrap()].flags & 0x0E, flag::JUSTIFY_LEFT);
        // page 2 -> 3 on the next press (held select first released), 3 -> done
        assert!(!s.step(3, &mut ctx(&mut osd, &fonts, &strings, &tex, &res, 0.0)));
        assert!(!s.step(3, &mut ctx(&mut osd, &fonts, &strings, &tex, &res, 1.0)));
        assert_eq!(s.page, 3);
        assert!(s.table.is_none());
        assert!(!s.step(3, &mut ctx(&mut osd, &fonts, &strings, &tex, &res, 0.0)));
        assert!(s.step(3, &mut ctx(&mut osd, &fonts, &strings, &tex, &res, 1.0)));
    }

    #[test]
    fn page3_summaries_by_event_kind() {
        use velocity_game::event_database::event_database::{CEventRaceEntry, CEventRaces, CEventRound};
        let fonts = CFontHelper::default();
        let strings = CStringManager::new();
        let tex = TextureStore::default();
        let mut osd = COSDManager::default();
        let racer = |name: &str, lead: f32, lap: f32, speed: f32, grid: u8| RacerResult { name: name.into(), lead_secs: lead, fastest_lap: lap, top_speed: speed, overtaken: 2, grid, ..Default::default() };
        let mut res = RaceResults {
            racers: vec![racer("Bud", 65.5, 0.0, 50.0, 3), racer("me", 5.0, 71.9, 44.4, 1)],
            player_place: 2,
            event_full: Some(CEvent { kind: 1, ..Default::default() }),
            ..Default::default()
        };
        // single: statistics, 7 columns, width 1000
        let t = event_summary(&mut ctx(&mut osd, &fonts, &strings, &tex, &res, 0.0)).unwrap();
        assert_eq!(t.rows.len(), 4);
        assert_eq!(t.rows[2].texts, ["1", "Bud", "2", "1:05", "NA", "112mph", "3"]);
        assert_eq!(t.rows[3].texts[1..6], ["PLAYER", "2", "0:05", "1:11", "99mph"]);
        let it = &osd.items[t.rows[3].items[6].unwrap()];
        assert_eq!((it.x, it.y), (166.5 + 1000.0, 525.0));
        t.release(&mut osd);
        // series: ranked by points, then prestige, then the human
        let comp = |points, pres, human| CEventCompetitor { points, unknown_0c: pres, human, ..Default::default() };
        res.racers.push(racer("x", 0.0, 0.0, 0.0, 0));
        res.event_full = Some(CEvent { kind: 3, competitors: Some(vec![comp(10, 5, 1), comp(20, 0, 0), comp(10, 5, 0)]), ..Default::default() });
        res.competitor_names = vec![String::new(), "Julio".into(), "Al".into()];
        let t = event_summary(&mut ctx(&mut osd, &fonts, &strings, &tex, &res, 0.0)).unwrap();
        assert_eq!(t.rows[2].texts, ["1", "Julio", "0", "20"]);
        assert_eq!(t.rows[3].texts, ["2", "PLAYER", "5", "10"]);
        assert_eq!(t.rows[4].texts, ["3", "Al", "5", "10"]);
        assert_eq!(osd.items[t.rows[3].items[1].unwrap()].x, 391.5 + 75.0);
        t.release(&mut osd);
        // cup: 4 competitors, 2 per race, round 1 of 2 finished, human in round 0
        let race = |comps: Vec<usize>, fin: Vec<u8>| CEventRaceEntry { num_competitors: 2, competitors: comps.into_iter().map(Some).collect(), finish_positions: fin, ..Default::default() };
        let r0 = CEventRound { races: vec![race(vec![0, 1], vec![2, 1]), race(vec![2, 3], vec![1, 2])], ..Default::default() };
        let r1 = CEventRound { races: vec![race(vec![1, 2], vec![2, 1])], ..Default::default() };
        res.event_full = Some(CEvent {
            kind: 2,
            flags: 2 | 0x10,
            competitors_per_cup_race: 2,
            competitors: Some(vec![comp(0, 0, 1), comp(0, 0, 0), comp(0, 0, 0), comp(0, 0, 0)]),
            races: Some(CEventRaces { current_round: 1, rounds: vec![r0, r1], previous_round: Some(1), ..Default::default() }),
            ..Default::default()
        });
        res.competitor_names = vec![String::new(), "B".into(), "C".into(), "D".into()];
        let t = event_summary(&mut ctx(&mut osd, &fonts, &strings, &tex, &res, 0.0)).unwrap();
        // 2 + 1 races, + the winner row
        assert_eq!(t.bracket.len(), 4);
        // round 0 seats the human: race 0 shows its finish order, race 1 its slots
        assert_eq!(t.bracket[0].texts, ["B", "PLAYER"]);
        assert_eq!(t.bracket[1].texts, ["C", "D"]);
        // round 1 (current, no human): slots
        assert_eq!(t.bracket[2].texts, ["B", "C"]);
        assert_eq!(t.bracket[3].texts[1], "C");
        let a = &osd.items[t.bracket[0].items[0].unwrap()];
        assert_eq!((a.x, a.y, a.colour), (300.0, 690.0, COLOUR_TITLE));
        let b = &osd.items[t.bracket[1].items[1].unwrap()];
        assert_eq!((b.x, b.y), (300.0, 355.0));
        let w = &osd.items[t.bracket[3].items[1].unwrap()];
        assert_eq!((w.x, w.y), (900.0, 555.0));
        // no event: no page 3
        res.event_full = None;
        assert!(event_summary(&mut ctx(&mut osd, &fonts, &strings, &tex, &res, 0.0)).is_none());
    }
}
