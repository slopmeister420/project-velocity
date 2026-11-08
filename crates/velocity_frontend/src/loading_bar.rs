//! src\GameDataLoading\LoadingBar.cpp — `CLoadingBar` (GameManager +1392): timings of the load steps between
//! `Begin` and `End`, and the frame GameManager draws after each step (`GameManager::LoadingTimer` 0x000B8C10 ->
//! 0x000B89E0): the front end (loading screen) with a black strip along the bottom and the step's name in grey,
//! right-aligned on x = 460.

use crate::feck2::feck2_manager::{visos_colour, Feck2, COLOUR_BLACK};
use crate::fonts::font_manager::FONT_RIGHT;

/// `MAX_STAGES`
pub const MAX_STAGES: usize = 200;

#[derive(Clone, Debug, Default)]
pub struct CLoadingBar {
    pub stage_data: Vec<f32>, // +0 (800 bytes) expected fraction per stage
    pub stage_count: usize,   // +800
    pub stage: usize,         // +804
    pub start: f64,           // +808 timer at Begin
    pub times: Vec<f64>,      // +816 elapsed at each stage
    pub active: bool,         // +2420
}

impl CLoadingBar {
    /// LoadingBar.cpp 0x000A4280 — `Begin(stageData, stageCount)`
    pub fn begin(&mut self, stage_data: &[f32], stage_count: usize, now: f64) {
        debug_assert!(stage_count < MAX_STAGES, "stageCount < MAX_STAGES");
        if self.active {
            log::warn!("CLoadingBar::Begin: warning: mismatched Begin/End");
        }
        self.stage_data = stage_data.iter().copied().take(MAX_STAGES).collect();
        self.stage_count = stage_count;
        self.stage = 0;
        self.start = now;
        self.times = vec![0.0; MAX_STAGES];
        self.active = true;
    }

    /// LoadingBar.cpp 0x000A4440 — one step done
    pub fn stage(&mut self, now: f64) {
        if !self.active {
            return;
        }
        debug_assert!(self.stage < MAX_STAGES, "m_stage < MAX_STAGES");
        if self.stage < MAX_STAGES {
            self.times[self.stage] = now - self.start;
            self.stage += 1;
        }
    }

    /// LoadingBar.cpp 0x000A4550 — `End`: logs each stage's share of the total for the stage data table
    pub fn end(&mut self) {
        if !self.active {
            log::warn!("CLoadingBar::End: warning: mismatched Begin/End");
            return;
        }
        self.active = false;
        if self.stage != 0 {
            let total = self.times[self.stage - 1] as f32;
            if total != 0.0 {
                log::debug!("CLoadingBar::End: loading complete");
                log::debug!("  stageCount = {};", self.stage);
                let shares: Vec<String> = (0..self.stage).map(|i| format!("{}", self.times[i] as f32 / total)).collect();
                log::debug!("  stageData[] = {{ {{ {} }} }};", shares.join(", "));
                if self.stage != self.stage_count {
                    log::warn!("CLoadingBar::End: warning: stage count mismatch, update stage data");
                }
            }
        }
    }
}

/// GameManager.cpp 0x000B89E0 — after the front end is drawn: the black strip (from 512 - 2.7 line heights to the
/// bottom, `dword_68797C` = 640 wide) and the step name in VISOS_Colour(64,64,64,64) right-aligned on 460, 18 above
/// the bottom of the line. The font is the font manager's current one (FE font 0 here).
pub fn draw_loading_step(fe: &mut Feck2, label: &str) {
    let Some(font) = fe.fonts.font(0) else { return };
    let lh = font.line_height as f64;
    let top = (512.0 - lh * 2.7f32 as f64) as f32;
    fe.draw.rect(0, 0.0, top, 640.0, 512.0, 0.0, 0.0, 1.0, 1.0, COLOUR_BLACK);
    let text: Vec<u16> = label.encode_utf16().collect();
    let y = (512.0 - lh - 18.0) as f32;
    let grey = visos_colour(0x40, 0x40, 0x40, 0x40);
    let Feck2 { fonts, textures, draw, .. } = fe;
    if let Some(font) = fonts.font(0) {
        font.draw_string_at(460.0, y, &text, grey, FONT_RIGHT, textures, draw);
    }
}
