//! In-race HUD on the bevy side: the OSD in-game screen, the speedo, the minimap and the pause menu, drawn
//! into a transparent 640x480 overlay image above the 3D view (camera order 11, no clear).
//!
//! Game flow: send `HudCommand::Start` when the race begins (hud name from the car / profile, `digione` if
//! none, plus the track's inside / outside edge polylines for the minimap), fill `RaceHudInput` every frame,
//! `HudCommand::Stop` after the race. Game flow's pause gate (GameManager 0x000BC3D0) opens / closes the pause
//! menu with `OpenPauseMenu` / `LeaveAllMenus`; its choices come back as `PauseMenuMsg`.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::feck2::feck2_manager::{action, ActionState, FeInput};
use crate::font_helper::font_helper::CFontHelper;
use crate::game_menu_manager::game_menu_manager::{CGameMenu, CGameMenuManager, PauseMenuRequest, RaceOverEvent};
use crate::osd::osd_3d_interface::{self as o3d, COSD3DInterface};
use crate::osd::osd_manager::COSDManager;
use crate::osd::osd_map::{COSDMapRender, HudMap, HudMapDesc, MapInput};
use crate::osd::osd_screens::osd_in_game_screen::{self as ig, COSDInGameScreen, OsdRaceStatus};
use crate::osd::osd_screens::osd_race_over_stat_screen::{COSDRaceOverStatScreen, RaceResults, RacerResult, ResultsCtx, ResultsEvent};
use crate::osd::speedo::{HudConfig, Speedo, SpeedoInput};
use crate::render2d::{DrawList, TextureStore};
use crate::string_manager::string_manager::CStringManager;
use crate::{user_input, DISPLAY_H, DISPLAY_W};

/// what the HUD shows; game flow writes it every frame while racing.
#[derive(Resource, Default, Clone)]
pub struct RaceHudInput {
    pub status: OsdRaceStatus,
    pub speedo: SpeedoInput,
    pub map: MapInput,
    /// the AI vehicles for the name tags (game flow fills `player` / `vehicles`; `view_proj` comes from the race's
    /// 3D camera in `run_hud`)
    pub name_tags: ig::NameTagInput,
    /// the TFT line state for the 3D hud (`movie::TftMovieTexture`, filled by `tft_in`): +2896 wanted, +2904 >= 2
    pub tft_wanted: bool,
    pub tft_frames_ok: bool,
    /// two players (`byte_6F7157`): player 1's screen / speedo values and the view layout; filled by `split_in` from
    /// core's `GameViews` and `PlayerRaceStatus`
    pub split: Option<SplitHudInput>,
}

/// player 1 of a split screen race (COSDInGameScreen +20 = 1, OSD3dInterface view 1)
#[derive(Clone, Debug, Default)]
pub struct SplitHudInput {
    /// `byte_6F9180`: left / right halves (views kind 4 / 5) instead of top / bottom (2 / 3)
    pub side_by_side: bool,
    /// the players' HUD views (GameManager +988 [0] / [1]) as x, y, w, h fractions of the screen
    pub rects: [[f32; 4]; 2],
    pub status: OsdRaceStatus,
    pub speedo: SpeedoInput,
}

/// a speedo drawn through the 3d interface camera of a view: the full-screen 512 target squashed into the view
/// (the view's viewport, 4:3 projection kept: VsCameraManager 0x0005F8F0)
fn into_view(dl: &mut DrawList, from: usize, rect: [f32; 4]) {
    let (w, h) = (crate::render2d::SCREEN_W as f32, crate::render2d::SCREEN_H as f32);
    for q in &mut dl.quads[from..] {
        for p in &mut q.pos {
            p[0] = rect[0] * w + p[0] * rect[2];
            p[1] = rect[1] * h + p[1] * rect[3];
        }
    }
}

/// the view's viewport clip (0x000B3C30 sets the viewport for the view's whole pass): quads from `from` on are cut
/// to the rect, axis-aligned ones with their uvs interpolated, quads wholly outside dropped
fn clip_to_view(dl: &mut DrawList, from: usize, rect: [f32; 4]) {
    let (w, h) = (crate::render2d::SCREEN_W as f32, crate::render2d::SCREEN_H as f32);
    let lo = [rect[0] * w, rect[1] * h];
    let hi = [(rect[0] + rect[2]) * w, (rect[1] + rect[3]) * h];
    let tail: Vec<_> = dl.quads.drain(from..).collect();
    for mut q in tail {
        let outside = (0..2).any(|a| q.pos.iter().all(|p| p[a] < lo[a]) || q.pos.iter().all(|p| p[a] > hi[a]));
        if outside {
            continue;
        }
        // per axis: clamp each corner, moving its uv along the edge to the corner that differs only on that axis
        for a in 0..2 {
            let (pos, uv) = (q.pos, q.uv);
            for i in 0..4 {
                let c = pos[i][a].clamp(lo[a], hi[a]);
                if c == pos[i][a] {
                    continue;
                }
                let b = 1 - a;
                let Some(j) = (0..4).find(|&j| pos[j][b] == pos[i][b] && pos[j][a] != pos[i][a]) else { continue };
                let t = (c - pos[i][a]) / (pos[j][a] - pos[i][a]);
                q.pos[i][a] = c;
                q.uv[i] = [uv[i][0] + (uv[j][0] - uv[i][0]) * t, uv[i][1] + (uv[j][1] - uv[i][1]) * t];
            }
        }
        dl.quads.push(q);
    }
}

#[derive(Message, Debug, Clone)]
pub enum HudCommand {
    Start {
        hud: String,
        drag: bool,
        inside_edge: Vec<[f32; 3]>,
        outside_edge: Vec<[f32; 3]>,
        closed: bool,
    },
    Stop,
    /// `CGamePauseMenu` entered by game flow (GameManager 0x000BC3D0 gate: no pause before Go)
    OpenPauseMenu,
    /// GameMenuManager.cpp 0x000A9BD0 — `CGameMenuManager::LeaveAll` (pause button resume, restart, quit)
    LeaveAllMenus,
    /// OSD3dInterface.cpp 0x000EAF00 (from GameManager 0x000BC5F0): the hud parked off screen (slide -12, sliding out)
    HudSlideReset,
    /// OSD3dInterface.cpp 0x000EAF60 (from TimelineSequencer 0x00175270): the hud slides in
    HudSlideIn,
    /// OSDManager.cpp 0x000ED6E0 (player 0's in-game OSD -> 0x000F9980): the menu_top / menu_bottom cinematic bars.
    /// Shown by VsCrashCamera 0x00062C10, POI camera 0x0006A720, TimelineCameraShaker 0x0017A540; hidden by
    /// 0x00062D90, 0x0006A7B0, 0x0017A580, GameManager::RestartRace 0x000BE0F0, VsAIRaceVehicle 0x00018A40.
    CinematicBars(bool),
}

/// OSD3dInterface +2929 / +2932 (`byte_6FC098`, a static: survives races; ctor 0x000E9A80: out, 0)
#[derive(Debug, Clone, Copy)]
pub struct HudSlide {
    pub out: bool,   // +2929
    pub value: f32,  // +2932, -12 (off) .. 0 (in place)
    /// STAND-IN until core sends the slide calls: no offset before the first `HudSlideReset` / `HudSlideIn`
    /// (the ctor's "out" would otherwise slide the hud away for good)
    pub driven: bool,
    /// `dword_6FD1E0` (a function static of 0x000EA2B0, first value `dword_685FFC` 3.0): the TFT's drop, 0 open
    pub tft_drop: f32,
}

impl Default for HudSlide {
    fn default() -> Self {
        Self { out: true, value: 0.0, driven: false, tft_drop: crate::osd::osd_3d_interface::DROP_MAX }
    }
}

impl HudSlide {
    /// OSD3dInterface.cpp 0x000EA2B0 — one step per render call (a per-frame count like the FE animations)
    pub fn step(&mut self) {
        self.value = if self.out { self.value - f32::from_bits(0x3EA8_F5C3) } else { self.value + f32::from_bits(0x3EA8_F5C3) };
        self.value = self.value.clamp(-12.0, 0.0);
    }

    /// the offset used this frame (0 while not driven)
    pub fn offset(&self) -> f32 {
        if self.driven { self.value } else { 0.0 }
    }
}

/// a pause menu choice for game flow (resume / restart / quit to frontend / options).
#[derive(Message, Debug, Clone)]
pub struct PauseMenuMsg(pub PauseMenuRequest);

#[derive(Resource)]
pub struct HudState {
    pub active: bool,
    pub image: Handle<Image>,
    root: Option<Entity>,
    pads: [ActionState; action::COUNT],
    mapping: user_input::InputMapping,
    res: Option<HudResources>,
    /// `Stop` came while the game is in mode 2 (PostRace): the OSD stays up for the post race menu until the mode
    /// leaves PostRace (the original shuts the OSD down at EndOfGame)
    stop_deferred: bool,
    /// game updates (GameTick) since the last frame: COSDInGameScreen::Update's per-update attract fade steps
    ticks: u32,
    /// OSD3dInterface's hud slide
    pub slide: HudSlide,
    /// the last `Start` (hud, drag, edges, closed): the resources are made again when the player count changes
    start: Option<(String, bool, Vec<[f32; 3]>, Vec<[f32; 3]>, bool)>,
}

/// GameManager 0x000B3EC0 views of a two player race: Some(side by side, the players' HUD view rects (+988 [0] / [1]))
pub fn split_views(v: &velocity_game::game_mode::game_views::GameViews) -> Option<(bool, [[f32; 4]; 2])> {
    use velocity_game::game_mode::game_views::GameViews;
    if v.players != 2 || !v.second[0].used || !v.second[1].used {
        return None;
    }
    let side = v.second[0].kind == 4;
    Some((side, [GameViews::fraction(&v.second[0]).to_array(), GameViews::fraction(&v.second[1]).to_array()]))
}

/// player 1's HUD values: `PlayerRaceStatus[1]` (COSDInGameScreen::Update reads everything by its player index) over
/// player 0's race-wide fields (event, drag rounds, session, attract), and the views from `GameViews`
fn split_in(
    views: Option<Res<velocity_game::game_mode::game_views::GameViews>>,
    players: Option<Res<velocity_game::game_mode::race_cars::PlayerRaceStatus>>,
    mut input: ResMut<RaceHudInput>,
) {
    let split = views.as_deref().and_then(split_views);
    let Some((side_by_side, rects)) = split else {
        if input.split.is_some() {
            input.split = None;
        }
        return;
    };
    let mut status = input.status.clone();
    // STAND-IN: nos / zone / turbo ratios of player 1's car are not fed (0)
    let mut speedo = SpeedoInput::default();
    if let Some(p) = players.as_ref().and_then(|p| p.0.get(1)) {
        status.racing = p.racing;
        status.race_length = p.race_length;
        status.position = p.position;
        status.lap = p.lap;
        status.laps = p.laps;
        status.lap_time = p.lap_time;
        speedo.speed = p.speed;
        speedo.rpm = p.rpm;
        speedo.gear = p.gear;
    }
    // STAND-IN: per-driver values core does not split yet: no points / combo / wrong way of player 1's car
    status.points = 0;
    status.combo = 0;
    status.multiplier = 0;
    status.wrong_way = false;
    input.split = Some(SplitHudInput { side_by_side, rects, status, speedo });
}

/// the original's OSD-side globals.
pub struct HudResources {
    pub textures: TextureStore,
    pub fonts: CFontHelper,
    pub strings: CStringManager,
    pub osd: COSDManager,
    pub screen: COSDInGameScreen,
    pub speedo: Option<Speedo>,
    pub map: Option<COSDMapRender>,
    pub menus: CGameMenuManager,
    pub time: f32,
    /// OSD3dInterface's TFT / map models (from the hud config)
    pub osd3d: Option<COSD3DInterface>,
    /// this frame's map screen texture (the `hud_map_screen` model's), empty when the map is not drawn
    pub map_screen: DrawList,
    /// perf (`VELOCITY_PERF_OFF=hudopt`): the quads last rasterised into the hud image / each view's map image; the
    /// rasterise + texture upload is skipped while the draw list is unchanged (textures are fixed per resources)
    last_hud: Vec<crate::render2d::Quad>,
    /// widescreen: extra virtual units across (0 at 4:3 and in a split screen), set by `run_hud` from the window
    pub wide_extra: f32,
    last_map: std::sync::Mutex<[Option<Vec<crate::render2d::Quad>>; 2]>,
    /// two players (`byte_6F7157`): Some(side by side `byte_6F9180`)
    pub split: Option<bool>,
    /// player 1's in-game screen and speedo (OSD3dInterface +3360) in a split screen race
    pub screen2: Option<COSDInGameScreen>,
    pub speedo2: Option<Speedo>,
    /// `byte_6FD208` the post race results (stepped by `results_update`)
    pub results: COSDRaceOverStatScreen,
}

impl HudResources {
    /// `COSD3DInterface::OnGameInit` (0x000E9C90) + `OSDInGameScreen::Create` for one race.
    pub fn new(hud: &str, drag: bool, inside: &[[f32; 3]], outside: &[[f32; 3]], closed: bool) -> Self {
        Self::new_split(hud, drag, inside, outside, closed, None)
    }

    /// OSD3dInterface.cpp 0x000E9FD0 / OSDInGameScreen.cpp 0x000FAB40 — `new` for one or two players: OnGameInit loads each player's hud config (`Load(name, p)`), the
    /// OSD screens are created per player (COSDInGameScreen +20), both on the one game OSD manager `dword_6FD200`.
    /// Both players use `hud` (core passes one name; GamePlayer +392 of player 1 is not wired).
    pub fn new_split(hud: &str, drag: bool, inside: &[[f32; 3]], outside: &[[f32; 3]], closed: bool, split: Option<bool>) -> Self {
        let mut textures = TextureStore::default();
        let mut fonts = CFontHelper::default();
        fonts.load_game_fonts(&mut textures);
        let mut strings = CStringManager::new();
        strings.set_language("English");
        if strings.load_index_file("StrIndex.idx").is_ok() {
            let sections: Vec<String> = strings.index.as_ref().map(|i| i.sections.iter().map(|s| s.name.trim_end_matches(".sbk").to_string()).collect()).unwrap_or_default();
            for s in sections {
                let _ = strings.load_section_name(&s);
            }
        }
        let mut osd = COSDManager::default();
        // OSDInGameScreen 0x000FAB40: \osd\osdT.vcd (OSD_overlay_game_logo, menu_top, menu_bottom)
        textures.load_vcd("\\OSD\\OSDT.vcd");
        let screen = ig::create(&mut osd, &fonts, &strings, &textures, drag, 0, split);
        let screen2 = split.map(|_| ig::create(&mut osd, &fonts, &strings, &textures, drag, 1, split));
        let config = HudConfig::load(hud).or_else(|e| {
            log::error!("ERROR: COSD3DInterface::OnGameInit - failed to load hud config ({:?}), using digione", e);
            HudConfig::load("digione")
        });
        let config = config.ok();
        let speedo2 = config.as_ref().filter(|_| split.is_some()).map(|c| Speedo::new(c.split_player(1), &mut textures));
        let speedo = config.map(|c| if split.is_some() { c.split_player(0) } else { c }).map(|c| Speedo::new(c, &mut textures));
        let map = if inside.len() > 1 && outside.len() > 1 {
            let colour = speedo.as_ref().map(|s| s.config.map_colour).unwrap_or([255, 208, 48]);
            let centre = speedo.as_ref().map(|s| s.config.map[0]).unwrap_or([-10.75, -7.2, 1.0]);
            let m = HudMap::construct(&HudMapDesc { inside, outside, size: 128, border: 4, closed });
            Some(COSDMapRender::new(m, colour, 0, centre, &mut textures))
        } else {
            None
        };
        // GameManager::InGameInit "Pause Screens Init" (0x000FC9E0)
        let menus = CGameMenuManager {
            screens: crate::osd::osd_screens::osd_screens::COSDPauseScreens::new(&textures),
            game_mode: 1,
            ..Default::default()
        };
        let mut osd3d = speedo.as_ref().map(|s| COSD3DInterface::new([s.config.tft[0], s.config.tft[1]], [s.config.map[0], s.config.map[1]]));
        if let (Some(o), Some(s2)) = (osd3d.as_mut(), speedo2.as_ref()) {
            o.split = true;
            o.map2 = [s2.config.map[0], s2.config.map[1]];
        }
        Self {
            textures,
            fonts,
            strings,
            osd,
            screen,
            speedo,
            map,
            menus,
            time: 0.0,
            osd3d,
            map_screen: DrawList::default(),
            last_hud: Vec::new(),
            wide_extra: 0.0,
            last_map: std::sync::Mutex::new([None, None]),
            split,
            screen2,
            speedo2,
            results: COSDRaceOverStatScreen::new(),
        }
    }

    /// one frame: in-game screen update, OSD update, menus, then draw.
    pub fn frame(&mut self, input: &RaceHudInput, dt: f32) -> DrawList {
        self.frame_slide(input, dt, &mut HudSlide::default())
    }

    /// `frame` with OSD3dInterface's hud slide (kept by `HudState` across races)
    pub fn frame_slide(&mut self, input: &RaceHudInput, dt: f32, slide: &mut HudSlide) -> DrawList {
        self.time += dt;
        ig::update(&mut self.screen, &mut self.osd, &input.status);
        // player 1's screen with its own race values (the session / attract parts act on the shared manager again)
        if let (Some(s2), Some(sp)) = (self.screen2.as_mut(), input.split.as_ref()) {
            ig::update(s2, &mut self.osd, &sp.status);
        }
        if !input.status.attract {
            let names = self.menus.options.display[crate::game_options::display::NAMES];
            ig::update_name_tags(&mut self.screen, &mut self.osd, &self.fonts, &self.textures, &input.name_tags, names);
            // player 1's screen (Update 0x000FA915 per screen) has its own tag items (Create's tail is not gated by
            // +20), filled the same way: distance to driver 0, OSD bridge always applied (`byte_68565C` 1, never written)
            if let (Some(s2), Some(_)) = (self.screen2.as_mut(), input.split.as_ref()) {
                ig::update_name_tags(s2, &mut self.osd, &self.fonts, &self.textures, &input.name_tags, names);
            }
        }
        self.osd.update(self.time, &self.fonts, &self.strings, &self.textures);
        let mut dl = DrawList::default();
        let mut tags_end = 0;
        // GameManager 0x000B9010: the 3D OSD pass 0x000EF5F0 runs inside each player view (render flag 0x200000; the
        // mirror view has none) with that view's camera, before the 3D huds and the 2D overlay. It draws every in-3D
        // item of the manager, so with two players both screens' tags are drawn in each half (original, kept).
        let rects = input.split.as_ref().map(|s| s.rects);
        let views = [
            (input.name_tags.view_proj.as_ref(), rects.map(|r| r[0])),
            (input.name_tags.view_proj1.as_ref().filter(|_| rects.is_some()), rects.map(|r| r[1])),
        ];
        for (vp, rect) in views {
            let Some(vp) = vp else { continue };
            let from = dl.quads.len();
            self.osd.render_3d(&self.fonts, &self.textures, vp, &mut dl);
            if let Some(r) = rect {
                into_view(&mut dl, from, r);
                clip_to_view(&mut dl, from, r);
            }
        }
        tags_end = tags_end.max(dl.quads.len());
        // OSD3dInterface.cpp 0x000EA2B0: speedo, TFT and map only while the game OSD manager is enabled (+29 bit 0)
        // and its mask has no bit 1 (set before the start, in the attract race and after the finish)
        self.map_screen.clear();
        if let Some(o) = &mut self.osd3d {
            o.out.drawn = false;
            o.out.map_screen = false;
        }
        if let Some(o) = &mut self.osd3d {
            o.out2.drawn = false;
        }
        if !self.osd.disabled && self.osd.hidden_mask & 1 == 0 {
            slide.step();
            let off = slide.offset();
            if let Some(s) = &mut self.speedo {
                // Speedo 0x000F46F0: position = base + (-slide, slide, 0)
                s.set_offset([0.0 - off, off, 0.0]);
                let from = dl.quads.len();
                s.render(&input.speedo, &mut dl);
                if let Some(r) = rects {
                    into_view(&mut dl, from, r[0]);
                }
            }
            // the TFT (single player) and the map frame / screen models
            if let Some(o) = &mut self.osd3d {
                o.render(&mut slide.tft_drop, off, input.tft_wanted, input.tft_frames_ok);
            }
            if let Some(m) = &mut self.map {
                // the hud config's map position with x += slide, y += slide
                let base = self.speedo.as_ref().map(|s| s.config.map[0]).unwrap_or([-10.75, -7.2, 1.0]);
                m.centre = [base[0] + off, base[1] + off, base[2]];
                m.render(&input.map, dt, &mut self.map_screen);
                if let Some(o) = &mut self.osd3d {
                    o.out.map_screen = true;
                }
            }
            // GameManager 0x000B9010: 0x000EA2B0 again for view 1 (the slide steps once per call), player 1's speedo
            // (+3360) and map slot; the map screen texture is the one map (`unk_6F5BC8`)
            if let (Some(s2), Some(sp), Some(r)) = (self.speedo2.as_mut(), input.split.as_ref(), rects) {
                slide.step();
                let off = slide.offset();
                s2.set_offset([0.0 - off, off, 0.0]);
                let from = dl.quads.len();
                s2.render(&sp.speedo, &mut dl);
                into_view(&mut dl, from, r[1]);
                if let Some(o) = &mut self.osd3d {
                    o.render_view1(off);
                    o.out2.map_screen = o.out.map_screen;
                }
            }
        }
        self.osd.render(&self.fonts, &self.strings, &self.textures, &mut dl);
        // widescreen (user-requested deviation from the 4:3 original): name tags follow the world view across the
        // whole width; every other element keeps its size and sticks to the left / right / centre by where it is
        if self.wide_extra > 0.0 && input.split.is_none() {
            let scale = (512.0 + self.wide_extra) / 512.0;
            for q in &mut dl.quads[..tags_end] {
                for p in &mut q.pos {
                    p[0] *= scale;
                }
            }
            dl.widen(tags_end, self.wide_extra, wide_anchor);
        }
        dl
    }

    /// one game update's parts, in GameManager::InGameUpdate (0x000B77F0) order: "g_OSDPauseScreens.Update"
    /// (0x000FCAF0), "g_GameMenuManager.Update" (0x000A9DF0, `dt` = the update step 0x000BF650, `pad` = this update's
    /// UserInput state) and COSDInGameScreen::Update's attract fade
    pub fn game_update(&mut self, input: &RaceHudInput, pad: &FeInput, dt: f32) {
        self.menus.update_screens(&mut self.osd, &self.fonts, &self.strings, &self.textures);
        if self.menus.is_open() {
            self.menus.update(pad, dt, &mut self.osd, &self.fonts, &mut self.strings, &self.textures);
        }
        ig::attract_step(&mut self.screen, &mut self.osd, &input.status);
    }

    /// `n` game updates without input at the 24 Hz step
    pub fn game_updates(&mut self, input: &RaceHudInput, n: u32) {
        for _ in 0..n {
            self.game_update(input, &FeInput::default(), 1.0 / 24.0);
        }
    }

    pub fn open_pause(&mut self) {
        if !self.menus.is_open() {
            let m = CGameMenu::pause(&self.menus.options, &mut self.strings);
            self.menus.enter(m, &mut self.osd);
        }
    }
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HudCommand>()
            .add_message::<PauseMenuMsg>()
            .init_resource::<RaceHudInput>()
            .add_systems(Startup, setup_hud)
            .add_systems(
                Update,
                (handle_hud_commands, trail_blazer_in, tft_in, camera_in, game_in, split_in, osd_flags_in, post_race_menu, run_hud, run_osd_3d, end_of_game).chain(),
            )
            .add_systems(velocity_game::vs_common_main::GameTick, count_game_updates)
            .add_systems(
                velocity_game::vs_common_main::GameTick,
                results_update
                    .before(velocity_game::vs_common_main::InGameUpdateSet::UpdateSessions)
                    .run_if(in_state(velocity_game::EGameMode::InGame))
                    .run_if(|gm: Option<Res<velocity_game::GameManager>>| gm.is_some_and(|g| !g.paused)),
            );
    }
}

/// what 0x000F09C0 reads of the race: `VsAIWorld::GetVehicleInPosition(place)` per place (driver name 0x000153B0,
/// record block 0x000244E0 +4 finish time) from the ai, the +84 points by place from core's `RacerPoints`, the
/// player's place and the event (`dword_6F9BE8`)
fn race_results(
    ai: Option<&velocity_ai::plugin::AiState>,
    points: Option<&velocity_game::game_mode::race_cars::RacerPoints>,
    gm: &velocity_game::GameManager,
    opponents: &mut Option<Option<velocity_ai::opponent_database::OpponentDatabase>>,
) -> RaceResults {
    use velocity_ai::ai::vs_ai_driver::DriverRef;
    let mut out = RaceResults::default();
    if let Some(rm) = ai.and_then(|a| a.world.as_ref()).map(|w| &w.race) {
        out.player_place = rm.player_positions.first().copied().unwrap_or(0);
        for (i, &r) in rm.race_order.iter().enumerate() {
            let v = rm.vehicle(r);
            let name = v
                .and_then(|v| match v.driver {
                    Some(DriverRef::Ai(d)) => rm.drivers.get(d).and_then(|d| d.get_name(v)),
                    _ => None,
                })
                .unwrap_or("");
            use velocity_ai::ai::vs_ai_race_vehicle::stat_f;
            out.racers.push(RacerResult {
                name: name.to_string(),
                time: v.map_or(0.0, |v| stat_f(&v.stats, 1)),
                points: points.map_or(0, |p| p.points_in_place(i + 1)),
                top_speed: v.map_or(0.0, |v| stat_f(&v.stats, 0)),
                fastest_lap: v.map_or(0.0, |v| stat_f(&v.stats, 11)),
                lead_secs: v.map_or(0.0, |v| stat_f(&v.stats, 17)),
                overtaken: v.map_or(0, |v| v.stats[18]),
                grid: v.map_or(0, |v| v.stats[22] as u8),
            });
        }
    }
    // page 3 (0x000F2EC0): the whole event; cups / series name their competitors through the opponent record
    if let Some(e) = gm.current_event() {
        if e.kind == 2 || e.kind == 3 {
            if opponents.is_none() {
                *opponents = Some(velocity_ai::opponent_database::OpponentDatabase::load());
            }
            let db = opponents.as_ref().and_then(|d| d.as_ref());
            out.competitor_names = e
                .competitors
                .iter()
                .flatten()
                .map(|c| if c.human != 0 { String::new() } else { db.and_then(|d| d.get_opponent_by_crc(c.opponent_crc)).map_or(String::new(), |o| o.name.clone()) })
                .collect();
        }
        out.event_full = Some(e.clone());
    }
    out.event = gm.current_event().map(|e| ResultsEvent {
        category: e.category,
        kind: e.kind,
        per_cup_race: e.competitors_per_cup_race,
        competitors: e.competitors.as_ref().map_or(0, |c| c.len() as u8),
        prize: e.prize,
    });
    out
}

/// GameManager 0x000BDFE0's `OSDManager 0x000F35C0(byte_6FD208, step)`: core runs that step machine on
/// `RaceSession.post_race_step` and reads `RaceProgress.results_busy` (= !the step's result), so the step runs here
/// once per game update in session PostRace, before UpdateSessions. Outside PostRace a leftover table is released
/// (RestartRace 0x000BE0F0 / EndOfGame 0x000BE540 call 0x000F3130). The table's player +84 goes into
/// `dword_6F7244` = `gm.vars.event_state`. Dev: `VELOCITY_RESULTS_AUTO=<s>` presses select after `s` s of
/// waiting, `VELOCITY_RESULTS_SHOT=<png>` saves one screenshot 1 s into the race table.
#[allow(clippy::too_many_arguments)]
fn results_update(
    mut commands: Commands,
    state: Option<ResMut<HudState>>,
    session: Option<Res<velocity_game::game_mode::race_session::RaceSession>>,
    progress: Option<ResMut<velocity_game::game_mode::race_session::RaceProgress>>,
    points: Option<Res<velocity_game::game_mode::race_cars::RacerPoints>>,
    ai: Option<Res<velocity_ai::plugin::AiState>>,
    gm: Option<ResMut<velocity_game::GameManager>>,
    (ui, gi): (Option<Res<velocity_game::controller::user_input::UserInput>>, Option<Res<velocity_game::game_input::game_input::GameInput>>),
    mut dev: Local<(u32, bool)>,
    mut opponents: Local<Option<Option<velocity_ai::opponent_database::OpponentDatabase>>>,
) {
    use velocity_game::game_mode::race_session::ESessionState;
    let (Some(mut state), Some(session), Some(mut progress), Some(mut gm)) = (state, session, progress, gm) else { return };
    let Some(r) = state.res.as_mut() else {
        if progress.results_busy {
            progress.results_busy = false;
        }
        return;
    };
    if session.state != ESessionState::PostRace {
        if r.results.table.is_some() || r.results.text_item.is_some() {
            r.results.reset(&mut r.osd);
        }
        *dev = (0, false);
        return;
    }
    let step = session.post_race_step;
    let mask = gi.as_ref().map_or(0xFFFF, |g| g.device_mask(0));
    let mut select = ui.as_ref().map_or(0.0, |u| u.value(velocity_game::controller::user_input::EAction::MenuSelect, mask));
    if let Some(s) = std::env::var("VELOCITY_RESULTS_AUTO").ok().and_then(|v| v.parse::<f32>().ok()) {
        if step == 1 || step == 3 {
            dev.0 += 1;
            if dev.0 as f32 >= s * 24.0 {
                select = 1.0;
                dev.0 = 0;
            }
        }
    }
    let results = race_results(ai.as_deref(), points.as_deref(), &gm, &mut opponents);
    let mut cx = ResultsCtx { osd: &mut r.osd, fonts: &r.fonts, strings: &r.strings, tex: &r.textures, select, results: &results };
    let done = r.results.step(step, &mut cx);
    if let Some(p) = r.results.player_points.take() {
        gm.vars.event_state = p;
    }
    if progress.results_busy == done {
        progress.results_busy = !done;
    }
    // `VELOCITY_RESULTS_SHOT_PAGE=<n>` picks the page shot (default 2)
    let shot_page = std::env::var("VELOCITY_RESULTS_SHOT_PAGE").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(2);
    if r.results.page == shot_page && r.results.timer >= 1.0 && !dev.1 {
        if let Ok(path) = std::env::var("VELOCITY_RESULTS_SHOT") {
            dev.1 = true;
            log::info!("results: screenshot {path}");
            commands
                .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                .observe(bevy::render::view::screenshot::save_to_disk(path));
        }
    }
}

/// one game update (GameTick) for the per-update OSD parts
fn count_game_updates(state: Option<ResMut<HudState>>) {
    if let Some(mut s) = state {
        s.ticks += 1;
    }
}

/// what COSDInGameScreen::Update reads of the game besides the race manager: `g_AttractMode` (0x000B36B0 /
/// 0x000B36F0) and the session phase `dword_6F5BEC` (core's `RaceSession.state`)
fn game_in(
    attract: Option<Res<velocity_game::font_helper::CAttractMode>>,
    session: Option<Res<velocity_game::game_mode::race_session::RaceSession>>,
    gm: Option<Res<velocity_game::GameManager>>,
    mut state: ResMut<HudState>,
    mut input: ResMut<RaceHudInput>,
) {
    // the game mode (CQuitMenu's dtor) and byte_6F7158 for the menus
    if let (Some(gm), Some(r)) = (gm, state.res.as_mut()) {
        r.menus.game_mode = gm.mode.unwrap_or(1) as u8;
        r.menus.online = gm.vars.network_race;
    }
    let (playing, quitting) = attract.map_or((false, false), |a| (a.playing, a.quitting));
    let state = session.map(|s| s.state as u8);
    let st = &input.status;
    if (st.attract, st.attract_quitting, st.session_state) != (playing, quitting, state) {
        let st = &mut input.status;
        st.attract = playing;
        st.attract_quitting = quitting;
        st.session_state = state;
    }
}

/// core's `RaceSessionMessage::OsdFlags` = the session machine's VsCrashCamera 0x00063030 (set) / VsAIRaceVehicle
/// 0x00023D00 (clear) on the game OSD manager `dword_6FD200` (+36 mask)
fn osd_flags_in(
    msgs: Option<Res<Messages<velocity_game::game_mode::race_session::RaceSessionMessage>>>,
    mut cursor: Local<bevy::ecs::message::MessageCursor<velocity_game::game_mode::race_session::RaceSessionMessage>>,
    mut state: ResMut<HudState>,
) {
    // without game flow (fecapture) the message is not registered
    let Some(msgs) = msgs else { return };
    for m in cursor.read(&msgs) {
        if let velocity_game::game_mode::race_session::RaceSessionMessage::OsdFlags { set, clear } = m {
            if let Some(res) = state.res.as_mut() {
                res.osd.hidden_mask |= *set;
                res.osd.hidden_mask &= !*clear;
            }
        }
    }
}

/// GameManager.cpp 0x000BB1C0 mode 2: unless another drag round follows (race type 5, round < rounds, not out), a
/// `CRaceOverMenu` is entered when no menu is open (then VsAudio::Pause 0x00049090); the menus update with the OSD.
fn post_race_menu(
    gm: Option<Res<velocity_game::GameManager>>,
    session: Option<Res<velocity_game::game_mode::race_session::RaceSession>>,
    mode: Option<Res<State<velocity_game::EGameMode>>>,
    mut state: ResMut<HudState>,
    audio: Option<ResMut<Messages<velocity_audio::AudioCommand>>>,
) {
    let (Some(gm), Some(session), Some(mode)) = (gm, session, mode) else { return };
    if *mode.get() != velocity_game::EGameMode::PostRace || gm.mode != Some(2) {
        return;
    }
    let ev = gm.current_event();
    // event race +4 == 5; the temporary (jump) events carry no race type, so the race selection's, as core's mode 2
    let drag = gm.race.as_ref().is_some_and(|r| r.race_type == velocity_game::event_database::event_database_track_loading::ERaceType::Drag);
    if drag && session.drag_round < session.drag_rounds && !session.drag_out {
        return;
    }
    let Some(r) = state.res.as_mut() else { return };
    if r.menus.is_open() {
        return;
    }
    let flags = ev.map_or(0, |e| e.flags);
    let ev = RaceOverEvent {
        online: gm.vars.network_race,
        completed: flags & velocity_game::event_database::event_database::EVENT_FLAG_COMPLETED != 0,
        failed: flags & velocity_game::event_database::event_database::EVENT_FLAG_FAILED != 0,
        drag,
    };
    log::info!("post race: RaceOver menu {ev:?}");
    r.menus.enter_race_over(ev, &mut r.osd);
    if let Some(mut a) = audio {
        a.write(velocity_audio::AudioCommand::Pause);
    }
}

/// GameTweaksMenu.cpp 0x000B17A0's own game calls: `byte_6F5D09` (quit the event) and SetGameMode(3)
fn end_of_game(
    mut r: MessageReader<PauseMenuMsg>,
    gm: Option<ResMut<velocity_game::GameManager>>,
    next: Option<ResMut<NextState<velocity_game::EGameMode>>>,
    modes: Option<ResMut<Messages<velocity_game::GameModeMessage>>>,
) {
    let (Some(mut gm), Some(mut next), Some(mut modes)) = (gm, next, modes) else { return };
    for PauseMenuMsg(q) in r.read() {
        let PauseMenuRequest::EndOfGame { quit_event } = q else { continue };
        log::info!("race over: end of game (quit event {quit_event})");
        gm.quit_event = *quit_event;
        let mut msgs = Vec::new();
        if let Some(s) = gm.set_game_mode(3, &mut msgs) {
            next.set(s);
        }
        for m in msgs {
            modes.write(m);
        }
    }
}

fn setup_hud(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let img = Image::new_fill(
        Extent3d { width: DISPLAY_W, height: DISPLAY_H, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let image = images.add(img);
    commands.insert_resource(HudState {
        active: false,
        image,
        root: None,
        pads: Default::default(),
        mapping: user_input::InputMapping::load_default(),
        res: None,
        stop_deferred: false,
        ticks: 0,
        slide: HudSlide::default(),
        start: None,
    });
}

#[derive(Component)]
struct HudView;

fn handle_hud_commands(
    mut commands: Commands,
    mut state: ResMut<HudState>,
    mut msgs: MessageReader<HudCommand>,
    mode: Option<Res<State<velocity_game::EGameMode>>>,
    views: Option<Res<velocity_game::game_mode::game_views::GameViews>>,
) {
    let post_race = mode.as_ref().is_some_and(|m| *m.get() == velocity_game::EGameMode::PostRace);
    if state.stop_deferred && !post_race {
        state.stop_deferred = false;
        state.active = false;
        state.res = None;
    }
    for m in msgs.read() {
        match m {
            HudCommand::Start { hud, drag, inside_edge, outside_edge, closed } => {
                let split = views.as_deref().and_then(split_views).map(|(side, _)| side);
                state.res = Some(HudResources::new_split(hud, *drag, inside_edge, outside_edge, *closed, split));
                state.start = Some((hud.clone(), *drag, inside_edge.clone(), outside_edge.clone(), *closed));
                state.active = true;
                state.stop_deferred = false;
                if state.root.is_none() {
                    let cam = commands
                        .spawn((Camera2d, Camera { order: 11, clear_color: ClearColorConfig::None, ..default() }, HudView))
                        .id();
                    let image = state.image.clone();
                    let root = commands
                        .spawn((
                            Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                            ImageNode::new(image),
                            UiTargetCamera(cam),
                            HudView,
                        ))
                        .id();
                    state.root = Some(root);
                }
            }
            HudCommand::Stop if post_race => state.stop_deferred = true,
            HudCommand::Stop => {
                state.active = false;
                state.res = None;
            }
            HudCommand::OpenPauseMenu => {
                if let Some(r) = state.res.as_mut() {
                    r.open_pause();
                }
            }
            HudCommand::LeaveAllMenus => {
                if let Some(r) = state.res.as_mut() {
                    r.menus.leave_all(&mut r.osd);
                }
            }
            HudCommand::HudSlideReset => {
                state.slide.value = -12.0;
                state.slide.out = true;
                state.slide.driven = true;
            }
            HudCommand::HudSlideIn => {
                state.slide.out = false;
                state.slide.driven = true;
            }
            HudCommand::CinematicBars(show) => {
                if let Some(r) = state.res.as_mut() {
                    ig::set_cinematic_bars(&r.screen, &mut r.osd, *show);
                }
            }
        }
    }
}

/// the race camera's world -> clip matrix for the 3D OSD pass (the original projects with the device's current
/// view / projection in ShaderManager 0x002A7840); player 1's view camera for the split screen pass. The rear-view
/// mirror (render flags 0x08000620, no 0x200000) has no 3D OSD pass, so it is never read.
#[allow(clippy::type_complexity)]
fn camera_in(
    cams: Query<(&Camera, &GlobalTransform), (With<visos_render::track_render::TrackCamera>, Without<visos_render::view_render::MirrorView>)>,
    second: Query<(&Camera, &GlobalTransform, &visos_render::view_render::TrackView), With<visos_render::view_render::SecondaryTrackCamera>>,
    mut input: ResMut<RaceHudInput>,
) {
    let vp = cams
        .iter()
        .find(|(c, _)| c.is_active)
        .map(|(c, g)| (c.clip_from_view() * g.to_matrix().inverse()).to_cols_array());
    if input.name_tags.view_proj != vp {
        input.name_tags.view_proj = vp;
    }
    let vp1 = second
        .iter()
        .find(|(c, _, v)| c.is_active && v.player == 1)
        .map(|(c, g, _)| (c.clip_from_view() * g.to_matrix().inverse()).to_cols_array());
    if input.name_tags.view_proj1 != vp1 {
        input.name_tags.view_proj1 = vp1;
    }
}

/// `GameManager +0x108C` (0x000FAEA0) for OSD item 13: read from core's resource directly
fn trail_blazer_in(gm: Option<Res<velocity_game::GameManager>>, mut input: ResMut<RaceHudInput>) {
    let m = gm.map_or(0, |g| g.trail_blazer_message);
    if input.status.trail_blazer_message != m {
        input.status.trail_blazer_message = m;
    }
}

/// the TFT line state of `movie::run_tft` for the 3D hud. Dev: `VELOCITY_TFT=<movie>` (e.g. `TFT\Al\daveC_1`) wants
/// that line whenever none is wanted in race.
fn tft_in(
    tft: Option<Res<crate::movie::TftMovieTexture>>,
    link: Option<Res<crate::movie::TftLink>>,
    state: Res<HudState>,
    mut input: ResMut<RaceHudInput>,
    mut dev: Local<Option<Option<String>>>,
) {
    let dev = dev.get_or_insert_with(|| std::env::var("VELOCITY_TFT").ok().filter(|s| !s.is_empty()));
    if let (Some(name), Some(link), true) = (dev.as_ref(), link.as_ref(), state.active) {
        let mut t = link.0.lock().unwrap();
        if !t.wanted {
            t.name = name.clone();
            t.frames = 0;
            t.wanted = true;
        }
    }
    let (w, ok) = tft.map_or((false, false), |t| (t.wanted, t.visible));
    if (input.tft_wanted, input.tft_frames_ok) != (w, ok) {
        input.tft_wanted = w;
        input.tft_frames_ok = ok;
    }
}

/// the 3D hud's bevy side: spawn the view at the first race, then per frame place the models from
/// `COSD3DInterface.out` (`out2` for player 1's view), the movie / map textures onto their screens, and hide
/// everything outside a race. The light set is the view player's race car (`CarRender.request` = race car id =
/// player index for the humans, core's race_cars).
#[allow(clippy::too_many_arguments)]
fn run_osd_3d(
    mut commands: Commands,
    state: Res<HudState>,
    input: Res<RaceHudInput>,
    mut views: Local<[Option<o3d::Osd3dView>; 2]>,
    tft: Option<Res<crate::movie::TftMovieTexture>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<visos_render::renderer::shader_manager::XboxStageMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut q: Query<(&mut Transform, &mut Visibility)>,
    mut cams: Query<&mut Camera>,
    mut projs: Query<&mut Projection>,
    game_cams: Option<Res<velocity_game::camera::game_cameras::GameCameras>>,
    (car_sets, flash): (Option<Res<visos_render::car_render::CarLightSets>>, Option<Res<velocity_ai::plugin::PoliceFlash>>),
) {
    let res = state.res.as_ref().filter(|_| state.active);
    let split = res.is_some_and(|r| r.split.is_some());
    // view 1 (player 1's half) only in a split screen race; its models live on their own layer and camera
    for (p, layer) in [(0, o3d::HUD_3D_LAYER), (1, o3d::HUD_3D_LAYER_P2)] {
        if views[p].is_none() && res.is_some() && (p == 0 || split) {
            views[p] = Some(o3d::spawn_view(&mut commands, &mut meshes, &mut materials, &mut images, layer));
            // the entities exist from the next frame
            return;
        }
    }
    let rects = input.split.as_ref().map(|s| s.rects);
    for p in 0..2 {
        let Some(v) = views[p].as_ref() else { continue };
        let o = res.and_then(|r| r.osd3d.as_ref());
        let out = o.map(|o| if p == 0 { o.out.clone() } else { o.out2.clone() }).unwrap_or_default();
        let out = if p == 1 && !split { o3d::Osd3dFrame::default() } else { out };
        // the player's interface3d camera planes; the tweak values before the cameras exist
        let clip = game_cams.as_ref().map(|g| {
            g.manager.players.get(p).and_then(|pc| pc.interface3d.as_ref()).map_or((g.manager.clip.near, g.manager.clip.far), |c| (c.near, c.far))
        });
        run_osd_3d_view(v, &out, rects.map(|r| r[p]), p as u64, res, &tft, &mut materials, &mut images, &mut q, &mut cams, &mut projs, clip, &car_sets, &flash);
    }
}

/// one view of `run_osd_3d`: camera (viewport = the player's half while split), light set, model placement
#[allow(clippy::too_many_arguments)]
fn run_osd_3d_view(
    v: &o3d::Osd3dView,
    out: &o3d::Osd3dFrame,
    rect: Option<[f32; 4]>,
    request: u64,
    res: Option<&HudResources>,
    tft: &Option<Res<crate::movie::TftMovieTexture>>,
    materials: &mut Assets<visos_render::renderer::shader_manager::XboxStageMaterial>,
    images: &mut Assets<Image>,
    q: &mut Query<(&mut Transform, &mut Visibility)>,
    cams: &mut Query<&mut Camera>,
    projs: &mut Query<&mut Projection>,
    clip: Option<(f32, f32)>,
    car_sets: &Option<Res<visos_render::car_render::CarLightSets>>,
    flash: &Option<Res<velocity_ai::plugin::PoliceFlash>>,
) {
    // the live "Graphics.Camera" Near/FarClipDistance (VsBaseCamera +0xEC / +0xF0, core's 0x000B75D0 + 0x000C0A90)
    if let Some((near, far)) = clip {
        if let Ok(mut pr) = projs.get_mut(v.camera) {
            if let Projection::Custom(c) = &mut *pr {
                if let Some(ip) = c.get_mut::<o3d::InterfaceProjection>() {
                    if (ip.0.near, ip.0.far) != (near, far) {
                        ip.0.near = near;
                        ip.0.far = far;
                    }
                }
            }
        }
    }
    // widescreen (user-requested deviation from the 4:3 original): the interface camera's aspect follows the window, so
    // the models keep their shape, and each one is moved sideways by its anchor (same rule as the 2D layer)
    let k = 1.0 + res.map_or(0.0, |r| r.wide_extra) / crate::render2d::SCREEN_W as f32;
    if let Ok(mut pr) = projs.get_mut(v.camera) {
        if let Projection::Custom(c) = &mut *pr {
            if let Some(ip) = c.get_mut::<o3d::InterfaceProjection>() {
                if ip.0.aspect_ratio != o3d::INTERFACE_ASPECT * k {
                    ip.0.aspect_ratio = o3d::INTERFACE_ASPECT * k;
                }
            }
        }
    }
    let widen = |group: Mat4, m: Mat4| -> Mat4 {
        if k <= 1.0 {
            return m;
        }
        let t = (velocity_game::camera::vs_camera_manager::INTERFACE_FOV * 0.5).tan();
        let depth = |z: f32| o3d::INTERFACE_EYE[2] - z;
        let n = group.w_axis.x / (depth(group.w_axis.z) * t * o3d::INTERFACE_ASPECT);
        let a = wide_anchor((n + 1.0) * 256.0);
        Mat4::from_translation(Vec3::X * ((2.0 * a - 1.0) * (k - 1.0) * depth(m.w_axis.z) * t * o3d::INTERFACE_ASPECT)) * m
    };
    if let Ok(mut c) = cams.get_mut(v.camera) {
        if c.is_active != out.drawn {
            c.is_active = out.drawn;
        }
        let want = rect.and_then(|r| c.physical_target_size().map(|s| visos_render::view_render::viewport_of(Vec4::from_array(r), s)));
        let same = match (&c.viewport, &want) {
            (None, None) => true,
            (Some(a), Some(b)) => a.physical_position == b.physical_position && a.physical_size == b.physical_size,
            _ => false,
        };
        if !same {
            c.viewport = want;
        }
    }
    let mut place = |e: Entity, m: Option<Mat4>| {
        if let Ok((mut t, mut vis)) = q.get_mut(e) {
            let want = if m.is_some() { Visibility::Inherited } else { Visibility::Hidden };
            if *vis != want {
                *vis = want;
            }
            if let Some(m) = m {
                let n = Transform::from_matrix(m);
                if *t != n {
                    *t = n;
                }
            }
        }
    };
    let d = out.drawn;
    if d {
        // +2824: the view player's car set (VsAIRaceVehicle 0x00023F40(a3): race car p = player p's request) x 1.5,
        // ambient to the flash +2912
        let set = o3d::hud_lights(car_sets.as_ref().and_then(|s| s.0.get(&request)), flash.as_ref().map_or([0.0; 4], |f| f.0));
        let lit = set.light_fixed_function(None, Vec4::ONE);
        for h in &v.lit_materials {
            if materials.get(h).is_some_and(|m| m.params.lit != lit || m.params.arg.z != 1.0) {
                if let Some(mut m) = materials.get_mut(h) {
                    m.set_lit(Some(&set));
                }
            }
        }
    }
    let t = d && out.tft;
    let tg = out.tft_frame;
    place(v.tft_hinge, t.then_some(widen(tg, out.tft_hinge)));
    place(v.tft_frame, t.then_some(widen(tg, out.tft_frame)));
    place(v.tft_screen, (t && !out.tft_movie).then_some(widen(tg, out.tft_frame)));
    place(v.tft_movie, (t && out.tft_movie).then_some(widen(tg, out.tft_frame)));
    place(v.map_frame, d.then_some(widen(out.map, out.map)));
    place(v.map_screen, (d && out.map_screen).then_some(widen(out.map, out.map)));
    if t && out.tft_movie {
        let img = tft.as_ref().and_then(|t| t.image.clone());
        for h in &v.movie_materials {
            if materials.get(h).is_some_and(|m| m.texture != img) {
                if let Some(mut m) = materials.get_mut(h) {
                    m.texture = img.clone();
                }
            }
        }
    }
    if let (Some(r), true) = (res, d && out.map_screen) {
        if hud_opt() {
            let mut last = r.last_map.lock().unwrap();
            if last[request as usize].as_deref() == Some(&r.map_screen.quads[..]) {
                return;
            }
            last[request as usize] = Some(r.map_screen.quads.clone());
        }
        let frame = r.map_screen.rasterise_overlay(&r.textures, o3d::MAP_SCREEN_SIZE, o3d::MAP_SCREEN_SIZE);
        if let Some(mut img) = images.get_mut(&v.map_image) {
            img.data = Some(frame.data);
        }
    }
}

fn run_hud(
    time: Res<Time>,
    input: Res<RaceHudInput>,
    mut state: ResMut<HudState>,
    mut images: ResMut<Assets<Image>>,
    mut out: MessageWriter<PauseMenuMsg>,
    mut views: Query<&mut Visibility, With<HudView>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut options: ResMut<crate::game_options::GameOptions>,
    clock: Option<Res<velocity_game::vs_common_main::GameClock>>,
) {
    let active = state.active;
    let want_vis = if active { Visibility::Inherited } else { Visibility::Hidden };
    for mut v in views.iter_mut() {
        if !hud_opt() || *v != want_vis {
            *v = want_vis;
        }
    }
    let ticks = std::mem::take(&mut state.ticks);
    if !active {
        return;
    }
    let st = &mut *state;
    let Some(r) = st.res.as_mut() else { return };
    // the views came after Start (or went): OnGameInit / Create again with the player count
    let want = input.split.as_ref().map(|s| s.side_by_side);
    if r.split != want {
        if let Some((hud, drag, inside, outside, closed)) = &st.start {
            log::info!("hud: {} player layout", if want.is_some() { 2 } else { 1 });
            *r = HudResources::new_split(hud, *drag, inside, outside, *closed, want);
        }
    }
    if !r.menus.is_open() && r.menus.options != *options {
        r.menus.options = (*options).clone();
    }
    // the game updates owed this frame: UserInput is sampled once per update (GameManager::Update), so only the
    // first update of a frame sees a press; the mapping reads the visos input layer itself
    let dt = clock.as_ref().map(|c| c.timer.update_dt()).unwrap_or(1.0 / 24.0);
    for _ in 0..ticks {
        let next = st.mapping.step(&st.pads, &|_| 0.0);
        st.pads = next;
        let mut fe_in = FeInput::default();
        fe_in.pads[0] = next;
        r.game_update(&input, &fe_in, dt);
    }
    // widescreen layout of the window (none in a split screen: the halves are the world's own)
    let (img_w, extra) = windows
        .single()
        .ok()
        .filter(|_| r.split.is_none())
        .map_or((DISPLAY_W, 0.0), |w| crate::render2d::wide_size(w.physical_width() as f32 / w.physical_height().max(1) as f32));
    if r.wide_extra != extra {
        r.wide_extra = extra;
        r.last_hud.clear();
    }
    let dl = r.frame_slide(&input, time.delta_secs(), &mut st.slide);
    for q in r.menus.requests.drain(..) {
        out.write(PauseMenuMsg(q));
    }
    // the in-race audio / display menus edit the same options: applied and saved by `apply_options`
    if r.menus.options != *options {
        *options = r.menus.options.clone();
    }
    if hud_opt() {
        if r.last_hud == dl.quads {
            return;
        }
        r.last_hud.clone_from(&dl.quads);
    }
    let frame = if extra > 0.0 {
        dl.rasterise_overlay_virt(&r.textures, img_w, DISPLAY_H, crate::render2d::SCREEN_W as f32 + extra)
    } else {
        dl.rasterise_overlay(&r.textures, DISPLAY_W, DISPLAY_H)
    };
    if let Some(mut img) = images.get_mut(&st.image) {
        if img.texture_descriptor.size.width != img_w {
            img.texture_descriptor.size.width = img_w;
        }
        img.data = Some(frame.data);
    }
}

/// widescreen anchor of a hud element by its centre in the 512-unit layout: 0 left, 1 right, 0.5 centre (the 3D hud
/// models use the same rule on their projected position, `run_osd_3d_view`)
fn wide_anchor(cx: f32) -> f32 {
    let n = cx / 256.0 - 1.0;
    // wide thresholds: a label and its value are separate items drawn apart, they must land on the same side
    if n < -0.1 {
        0.0
    } else if n > 0.1 {
        1.0
    } else {
        0.5
    }
}

/// perf switch: `VELOCITY_PERF_OFF=hudopt` restores the per-frame rasterise + upload
fn hud_opt() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| !std::env::var("VELOCITY_PERF_OFF").is_ok_and(|v| v.split(',').any(|s| s.trim().eq_ignore_ascii_case("hudopt"))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_to_view_cuts_quads_and_uvs() {
        let q = |x0: f32, y0: f32, x1: f32, y1: f32| crate::render2d::Quad {
            texture: 1,
            pos: [[x0, y0], [x1, y0], [x0, y1], [x1, y1]],
            uv: [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
            colour: [0xFFFF_FFFF; 4],
        };
        let mut dl = DrawList::default();
        dl.quads = vec![q(0.0, 0.0, 10.0, 10.0), q(100.0, 246.0, 120.0, 266.0), q(10.0, 300.0, 20.0, 310.0)];
        // the top half of the 512 target; the first quad is before `from` and stays
        clip_to_view(&mut dl, 1, [0.0, 0.0, 1.0, 0.5]);
        assert_eq!(dl.quads.len(), 2);
        let c = &dl.quads[1];
        assert_eq!(c.pos, [[100.0, 246.0], [120.0, 246.0], [100.0, 256.0], [120.0, 256.0]]);
        assert_eq!(c.uv, [[0.0, 0.0], [1.0, 0.0], [0.0, 0.5], [1.0, 0.5]]);
    }
}
