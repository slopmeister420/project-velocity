//! the front end's debug-menu tweaks: the nine AddTweaks blocks of the frontend's objects (`CTweakSystem::addTweak`
//! 0x00085A90 float / 0x00085B40 S32 / 0x00085BF0 enum / 0x00085CA0 bool on `FindSystem || CreateSystem` 0x00086250)
//! and the tweak-changed callback (FECK2Manager.cpp 0x00202EA0). Bevy glue: the original points each tweak at its
//! global; here a `TweakCell` stands in and `sync` mirrors it with the port's variable both ways (a menu edit wins,
//! otherwise the cell follows the variable). Cells without a port variable are the variable themselves (their
//! readers are not ported).
//!
//! When: SetGameMode 0x000BBD30 rebuilds the tree on every mode change (core's `TweaksReset`). The front end
//! (boot VsCommonMain 0x0016E221 / FrontEndReInit 0x000B5A80) runs `CFrontendManager::Initialise` (Player Info,
//! FELights) and `CreateAllScreens` (FE3DCamera, alloy editor, customisation, main menu, showroom, hangout); InGameInit
//! (0x000B5EE0) runs Initialise + `CreateCommonScreens` (FE3DCamera only) and the race OSD adds Game.OSD
//! (COSDInGameScreen::Create 0x000F82F0). Mode 3 (end of game) registers nothing. `register` runs whenever the tree
//! lacks "Player Info" in a mode that has the set (STAND-IN for those call sites; core adds "System" at once on the
//! reset, the original after the mode's init, so after a mode change System sorts after these systems).

use bevy::prelude::*;
use velocity_game::cli::tweak::{CTweakManager, TweakCell, TweakData};

/// a tweak mirrored with a port variable (`last` = the value both agreed on last frame)
pub struct Mirror {
    pub cell: TweakCell,
    last: TweakData,
}

impl Mirror {
    fn new(v: TweakData) -> Self {
        Mirror { cell: TweakCell::new(v.clone()), last: v }
    }

    /// Some(new value) when the menu changed the cell since the last call (store it into the variable); otherwise
    /// the cell takes the variable's value
    pub fn sync(&mut self, live: TweakData) -> Option<TweakData> {
        let c = self.cell.get();
        if c != self.last {
            self.last = c.clone();
            return Some(c);
        }
        if c != live {
            self.cell.set(live.clone());
        }
        self.last = live;
        None
    }
}

fn cell(v: TweakData) -> TweakCell {
    TweakCell::new(v)
}

fn as_bool(v: &TweakData) -> bool {
    match v {
        TweakData::Bool(b) => *b,
        TweakData::Int(i) => *i != 0,
        TweakData::Float(f) => *f != 0.0,
        TweakData::String(_) => false,
    }
}

fn as_f32(v: &TweakData) -> f32 {
    match v {
        TweakData::Float(f) => *f,
        TweakData::Int(i) => *i as f32,
        TweakData::Bool(b) => *b as i32 as f32,
        TweakData::String(_) => 0.0,
    }
}

fn as_i32(v: &TweakData) -> i32 {
    match v {
        TweakData::Int(i) => *i,
        TweakData::Float(f) => *f as i32,
        TweakData::Bool(b) => *b as i32,
        TweakData::String(_) => 0,
    }
}

/// the tweaked variables (globals in the original; initial values from .data, bss = 0)
#[derive(Resource)]
pub struct FrontendTweaks {
    /// "Game.OSD.Split Time Display Duration" `dword_6860EC` (2.0), read by core's ScoreManager 0x0016BE50
    pub split_time_duration: TweakCell,
    /// "Player Info.Cash" `dword_6F719C` = core's `gm.vars.cash`
    pub cash: Mirror,
    /// "Player Info.Prestige Points" `dword_6F71A0` = `gm.vars.prestige`
    pub prestige: Mirror,
    /// "Frontend.FELights.AmbientR/G/B" `byte_75CA88` xyz (the character light set copy, see `fe_lights_add_tweaks`)
    pub ambient: [TweakCell; 3],
    /// "Frontend.FELights.Point Lit Car" `byte_6865E4` (1; CarRender 0x00118D70, graphics)
    pub point_lit_car: TweakCell,
    /// "Frontend.FELights.WriteFile" `unk_75CA50`
    pub write_file: TweakCell,
    /// "Frontend.FELights.Light1..3.R/G/B" `byte_75CA58` / `75CA68` / `75CA78` xyz
    pub light_colour: [[TweakCell; 3]; 3],
    /// "...Light1..3.Rotation" `flt_75CA38` / `unk_75CA3C` / `unk_75CA40`
    pub light_rotation: [TweakCell; 3],
    /// "...Light1..3.Elevation" `flt_75CA44` / `dword_75CA48` / `unk_75CA4C`
    pub light_elevation: [TweakCell; 3],
    /// "Frontend.Display cheat screen" `byte_75EDB4` = `fe_hangout_screen::DISPLAY_CHEAT_SCREEN`
    pub display_cheat_screen: Mirror,
    /// "Frontend.Unlock ALL Events" `byte_6F0D70` = core's `DebugUnlocks.events`
    pub unlock_all_events: Mirror,
    /// "Frontend.Vehicles.Enable High Showroom Cars" `unk_75F44C` (no reader in F3_D)
    pub high_showroom_cars: TweakCell,
    /// "Frontend.Vehicles.Enable Showroom screenshots" `byte_75F44D` (FEHangoutScreen 0x0021EBF0 / 0x0021EF50)
    pub showroom_screenshots: TweakCell,
    /// "Frontend.Vehicles.Enable Hidden Showroom screenshots" `byte_75F44F`
    pub hidden_showroom_screenshots: TweakCell,
    /// "Frontend.Vehicles.Output car colour" `unk_7680B4` (no reader in F3_D)
    pub output_car_colour: TweakCell,
    /// "Frontend.Enable 2D" `byte_6C118C` (1) = `feck2_manager::ENABLE_2D`
    pub enable_2d: Mirror,
    /// "Frontend.Enable Memory Output" `byte_75C7A1` (FE3DCamera 0x00277BE0, FE2DOverlay 0x00281C40 memory logs)
    pub memory_output: TweakCell,
    /// `byte_736554`: both "Frontend.Characters.Character Camera" and "Frontend.Camera.Free Roam"
    pub free_roam: TweakCell,
    /// "Frontend.Vehicles.Submission Sheet screenshots - ALL" `word_7680B5`
    pub submission_all: TweakCell,
    /// "Frontend.Vehicles.Submission Sheet screenshots - THIS CAR" `byte_7680B6`
    pub submission_this_car: TweakCell,
    /// "Frontend.Garage.Sell back multiplier" `flt_6C1404` (1.0; kit refunds 0x0018AB70, not ported)
    pub sell_back_multiplier: TweakCell,
    /// "Frontend.Vehicles.Unlock ALL Vehicles" `byte_6877F0` = core's `DebugUnlocks.vehicles` and the FE copy
    /// `quick_race.all_unlocked`
    pub unlock_all_vehicles: Mirror,
    /// "Frontend.Vehicles.Enable Showroom level cars in customisation" `byte_76897C` (FEPlayerGarageScreen 0x0024A8E0)
    pub showroom_cars_in_customisation: TweakCell,
    /// "Frontend.Vehicles.Wheel change bounce time / height / oscillations" `dword_6C1510` 0.5 / `flt_6C1514` 0.005 /
    /// `dword_6C1518` 2 (FE3DVehicles 0x0025E7D0) = `fe_3d_vehicles::{BOUNCE_TIME, BOUNCE_HEIGHT, BOUNCE_OSCILLATIONS}`
    pub bounce_time: Mirror,
    pub bounce_height: Mirror,
    pub bounce_oscillations: Mirror,
    /// "Frontend.Enable empty environment" `byte_7691BC` (FE3DCamera 0x00273000)
    pub empty_environment: TweakCell,
    /// "Frontend.Camera.Enable Duration Change" `byte_7691BE` = `fe_3d_camera::ENABLE_DURATION_CHANGE`
    pub enable_duration_change: Mirror,
    /// "Frontend.Camera.Move Duration" `dword_6C1530` (0.65) = `fe_3d_camera::MOVE_DURATION_TWEAK`
    pub move_duration: Mirror,
    /// "Frontend.Camera.Output camera position" `byte_7691BD` (0x00273ED0)
    pub output_camera_position: TweakCell,
    /// "Frontend.Camera.Change Up Axis" `byte_7691BF` (0x00273BB0..)
    pub change_up_axis: TweakCell,
    /// "Frontend.Camera.Debug Artist Cam.GarageCameraIndex" `dword_7065A4` -> graphics' `FrontEndRender.garage`
    pub garage_camera_index: Mirror,
    /// "...Debug Artist Cam.GarageCamera FOV " `dword_6C1534` (30)
    pub garage_camera_fov: TweakCell,
    /// "...Camera SIDE / FRONT / TOP / 3D Pos X/Y/Z" = camera table positions 1..4 (`dword_769290`..`dword_7692C0`,
    /// `FoyerState.table[1..=4][0]`)
    pub camera_pos: [[Mirror; 3]; 4],
}

impl Default for FrontendTweaks {
    fn default() -> Self {
        use TweakData::{Bool, Float, Int};
        let f0 = || cell(Float(0.0));
        let b0 = || cell(Bool(false));
        FrontendTweaks {
            split_time_duration: cell(Float(f32::from_bits(0x4000_0000))),
            cash: Mirror::new(Int(0)),
            prestige: Mirror::new(Int(0)),
            ambient: [f0(), f0(), f0()],
            point_lit_car: cell(Bool(true)),
            write_file: b0(),
            light_colour: std::array::from_fn(|_| [f0(), f0(), f0()]),
            light_rotation: [f0(), f0(), f0()],
            light_elevation: [f0(), f0(), f0()],
            display_cheat_screen: Mirror::new(Bool(false)),
            unlock_all_events: Mirror::new(Bool(false)),
            high_showroom_cars: b0(),
            showroom_screenshots: b0(),
            hidden_showroom_screenshots: b0(),
            output_car_colour: b0(),
            enable_2d: Mirror::new(Bool(true)),
            memory_output: b0(),
            free_roam: b0(),
            submission_all: b0(),
            submission_this_car: b0(),
            sell_back_multiplier: cell(Float(f32::from_bits(0x3F80_0000))),
            unlock_all_vehicles: Mirror::new(Bool(true)),
            showroom_cars_in_customisation: b0(),
            bounce_time: Mirror::new(Float(f32::from_bits(0x3F00_0000))),
            bounce_height: Mirror::new(Float(f32::from_bits(0x3BA3_D70A))),
            bounce_oscillations: Mirror::new(Int(2)),
            empty_environment: b0(),
            enable_duration_change: Mirror::new(Bool(false)),
            move_duration: Mirror::new(Float(f32::from_bits(0x3F26_6666))),
            output_camera_position: b0(),
            change_up_axis: b0(),
            garage_camera_index: Mirror::new(Int(0)),
            garage_camera_fov: cell(Int(30)),
            camera_pos: std::array::from_fn(|_| std::array::from_fn(|_| Mirror::new(Float(0.0)))),
        }
    }
}

/// 0.01f / 0.1f / 0.001f / 0.05f as pushed by the callers
const STEP_0_01: f32 = f32::from_bits(0x3C23_D70A);
const STEP_0_1: f32 = f32::from_bits(0x3DCC_CCCD);
const STEP_0_001: f32 = f32::from_bits(0x3A83_126F);
const STEP_0_05: f32 = f32::from_bits(0x3D4C_CCCD);
const ONE: f32 = f32::from_bits(0x3F80_0000);

/// Speedo.cpp region (COSDInGameScreen) 0x000F82F0 — the tail of `COSDInGameScreen::Create`: "Game.OSD"
pub fn osd_in_game_add_tweaks(tm: &mut CTweakManager, t: &FrontendTweaks) {
    let s = tm.create_system("Game.OSD");
    tm.add_tweak_f32(s, "Split Time Display Duration", t.split_time_duration.clone(), 0.0, f32::from_bits(0x4120_0000), 0.5);
}

/// FECK2Manager.cpp 0x001FDC20 — `CFrontendManager::Initialise`: "Player Info" (then 0x00203450)
pub fn frontend_manager_add_tweaks(tm: &mut CTweakManager, t: &FrontendTweaks) {
    let s = tm.create_system("Player Info");
    tm.add_tweak_i32(s, "Cash", t.cash.cell.clone(), 0, 1_000_000, 1000);
    tm.add_tweak_i32(s, "Prestige Points", t.prestige.cell.clone(), 0, 100_000, 100);
    fe_lights_add_tweaks(tm, t);
}

/// FECK2Object.cpp 0x00203450 — "Frontend.FELights" (+ MessageManager channel 1 handler `unk_75CA98`)
///
/// the values are the front-end character light set (`dword_769E58`: three lights + ambient), copied in by
/// 0x00203AE0 whenever a character element computes its lights (CElementCharacter 0x00285190, Light2's elevation
/// forced to 90) and written back with rotation / elevation as directions by the tweak-changed handler 0x00203B70.
/// Neither is ported (Hex-Rays fails on 0x00203B70; the character lights are graphics'), so the cells hold bss 0.
pub fn fe_lights_add_tweaks(tm: &mut CTweakManager, t: &FrontendTweaks) {
    let s = tm.create_system("Frontend.FELights");
    for (i, n) in ["AmbientR", "AmbientG", "AmbientB"].iter().enumerate() {
        tm.add_tweak_f32(s, n, t.ambient[i].clone(), 0.0, ONE, STEP_0_01);
    }
    tm.add_tweak_bool(s, "Point Lit Car", t.point_lit_car.clone());
    tm.add_tweak_bool(s, "WriteFile", t.write_file.clone());
    for l in 0..3 {
        let s = tm.create_system(&format!("Frontend.FELights.Light{}", l + 1));
        for (i, n) in ["R", "G", "B"].iter().enumerate() {
            tm.add_tweak_f32(s, n, t.light_colour[l][i].clone(), 0.0, ONE, STEP_0_01);
        }
        tm.add_tweak_f32(s, "Rotation", t.light_rotation[l].clone(), 0.0, f32::from_bits(0x43B4_0000), 4.0);
        tm.add_tweak_f32(s, "Elevation", t.light_elevation[l].clone(), f32::from_bits(0xC2B4_0000), f32::from_bits(0x42B4_0000), 1.0);
    }
}

/// FEHangoutScreen.cpp 0x00218390 (idb FEOnlineMatchInfoScreen) — HangoutScreen create: "Frontend"
pub fn hangout_add_tweaks(tm: &mut CTweakManager, t: &FrontendTweaks) {
    let s = tm.create_system("Frontend");
    tm.add_tweak_bool(s, "Display cheat screen", t.display_cheat_screen.cell.clone());
    tm.add_tweak_bool(s, "Unlock ALL Events", t.unlock_all_events.cell.clone());
}

/// FEHangoutScreen.cpp 0x0021CB70 — CarShowroomScreen create: "Frontend.Vehicles" (two blocks)
pub fn showroom_add_tweaks(tm: &mut CTweakManager, t: &FrontendTweaks) {
    let s = tm.create_system("Frontend.Vehicles");
    tm.add_tweak_bool(s, "Enable High Showroom Cars", t.high_showroom_cars.clone());
    let s = tm.create_system("Frontend.Vehicles");
    tm.add_tweak_bool(s, "Enable Showroom screenshots", t.showroom_screenshots.clone());
    tm.add_tweak_bool(s, "Enable Hidden Showroom screenshots", t.hidden_showroom_screenshots.clone());
}

/// FEMainMenuScreen.cpp 0x0023B4A0 (idb FEOnlineSignInScreen) — MainMenuScreen create
pub fn main_menu_add_tweaks(tm: &mut CTweakManager, t: &FrontendTweaks) {
    let s = tm.create_system("Frontend.Vehicles");
    tm.add_tweak_bool(s, "Output car colour", t.output_car_colour.clone());
    let s = tm.create_system("Frontend");
    tm.add_tweak_bool(s, "Enable 2D", t.enable_2d.cell.clone());
    tm.add_tweak_bool(s, "Enable Memory Output", t.memory_output.clone());
    let s = tm.create_system("Frontend.Characters");
    tm.add_tweak_bool(s, "Character Camera", t.free_roam.clone());
}

/// FECarCustomisationScreen.cpp 0x00247BE0 (idb FEPlayerGarageScreen) — CarCustomisationScreen create
pub fn customisation_add_tweaks(tm: &mut CTweakManager, t: &FrontendTweaks) {
    let s = tm.create_system("Frontend.Vehicles");
    tm.add_tweak_bool(s, "Submission Sheet screenshots - ALL", t.submission_all.clone());
    tm.add_tweak_bool(s, "Submission Sheet screenshots - THIS CAR", t.submission_this_car.clone());
    let s = tm.create_system("Frontend.Garage");
    tm.add_tweak_f32(s, "Sell back multiplier", t.sell_back_multiplier.clone(), STEP_0_01, ONE, STEP_0_01);
}

/// FEAlloyEditor_Basic.cpp 0x00258770 — "Frontend.Vehicles" (create 0x00258690, CreateAllScreens 0x001FEBE6)
pub fn alloy_editor_add_tweaks(tm: &mut CTweakManager, t: &FrontendTweaks) {
    let s = tm.create_system("Frontend.Vehicles");
    tm.add_tweak_bool(s, "Unlock ALL Vehicles", t.unlock_all_vehicles.cell.clone());
    tm.add_tweak_bool(s, "Enable Showroom level cars in customisation", t.showroom_cars_in_customisation.clone());
    tm.add_tweak_f32(s, "Wheel change bounce time", t.bounce_time.cell.clone(), STEP_0_1, f32::from_bits(0x4040_0000), STEP_0_1);
    tm.add_tweak_f32(s, "Wheel change bounce height", t.bounce_height.cell.clone(), STEP_0_001, STEP_0_05, STEP_0_001);
    tm.add_tweak_i32(s, "Wheel change bounce oscillations", t.bounce_oscillations.cell.clone(), 1, 5, 1);
}

/// FE3DCamera.cpp 0x0026FC40 (from the ctor 0x0026F3D0, CreateCommonScreens)
pub fn fe_3d_camera_add_tweaks(tm: &mut CTweakManager, t: &FrontendTweaks) {
    let s = tm.create_system("Frontend");
    tm.add_tweak_bool(s, "Enable empty environment", t.empty_environment.clone());
    let s = tm.create_system("Frontend.Camera");
    tm.add_tweak_bool(s, "Enable Duration Change", t.enable_duration_change.cell.clone());
    tm.add_tweak_f32(s, "Move Duration", t.move_duration.cell.clone(), f32::from_bits(0x3EB3_3333), f32::from_bits(0x3F26_6666), STEP_0_05);
    tm.add_tweak_bool(s, "Free Roam", t.free_roam.clone());
    tm.add_tweak_bool(s, "Output camera position", t.output_camera_position.clone());
    tm.add_tweak_bool(s, "Change Up Axis", t.change_up_axis.clone());
    let s = tm.create_system("Frontend.Camera.Debug Artist Cam");
    tm.add_tweak_enum(s, "GarageCameraIndex", t.garage_camera_index.cell.clone(), "CAM_OFF = 0, CAM_SIDE = 1, CAM_FRONT = 2, CAM_TOP = 3, CAM_FULL = 4");
    tm.add_tweak_i32(s, "GarageCamera FOV ", t.garage_camera_fov.clone(), 0, 90, 1);
    let (lo, hi) = (f32::from_bits(0xC1F0_0000), f32::from_bits(0x41F0_0000));
    for (k, view) in ["SIDE", "FRONT", "TOP", "3D"].iter().enumerate() {
        for (a, axis) in ["X", "Y", "Z"].iter().enumerate() {
            tm.add_tweak_f32(s, &format!("Camera {view} Pos {axis}"), t.camera_pos[k][a].cell.clone(), lo, hi, STEP_0_1);
        }
    }
}

/// which sets a game mode has (see the module header)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TweakSet {
    /// boot / FrontEndReInit: Initialise + CreateAllScreens
    FrontEnd,
    /// InGameInit: Initialise + CreateCommonScreens + the race OSD
    InGame,
}

/// the registrations of one set in the original's call order
pub fn register(tm: &mut CTweakManager, t: &FrontendTweaks, set: TweakSet) {
    frontend_manager_add_tweaks(tm, t);
    match set {
        TweakSet::FrontEnd => {
            // CreateAllScreens 0x001FEB90: CreateCommonScreens (FE3DCamera) first, then the screens in order
            fe_3d_camera_add_tweaks(tm, t);
            alloy_editor_add_tweaks(tm, t);
            customisation_add_tweaks(tm, t);
            main_menu_add_tweaks(tm, t);
            showroom_add_tweaks(tm, t);
            hangout_add_tweaks(tm, t);
        }
        TweakSet::InGame => {
            fe_3d_camera_add_tweaks(tm, t);
            osd_in_game_add_tweaks(tm, t);
        }
    }
}

/// `gm.mode` -> the set its init registers (mode 3 = end of game: none)
pub fn set_for_mode(mode: Option<u32>) -> Option<TweakSet> {
    match mode {
        None | Some(0) | Some(4) => Some(TweakSet::FrontEnd),
        Some(1) | Some(2) => Some(TweakSet::InGame),
        _ => None,
    }
}

/// per frame: register after a reset, mirror the cells with their variables, then the tweak-changed callback
#[allow(clippy::too_many_arguments)]
pub fn fe_tweaks(
    mut t: ResMut<FrontendTweaks>,
    gm: Option<ResMut<velocity_game::GameManager>>,
    unlocks: Option<ResMut<velocity_game::cli::debug_unlocks::DebugUnlocks>>,
    mut state: ResMut<crate::FrontendState>,
    career: Option<ResMut<velocity_game::game_mode::career::Career>>,
    fe_render: Option<ResMut<visos_render::rain_surface::FrontEndRender>>,
    messages: Option<Res<Messages<velocity_game::message::message_manager::GameMessage>>>,
    mut cursor: Local<bevy::ecs::message::MessageCursor<velocity_game::message::message_manager::GameMessage>>,
) {
    use std::sync::atomic::Ordering::Relaxed;
    use TweakData::{Bool, Float, Int};
    let Some(mut gm) = gm else { return };
    let t = &mut *t;
    let mut fe = state.frontend.as_mut().map(|f| &mut f.fe);

    // variables -> cells first, so a fresh registration takes the live value as its default
    if let Some(v) = t.cash.sync(Int(gm.vars.cash)) {
        gm.vars.cash = as_i32(&v);
    }
    if let Some(v) = t.prestige.sync(Int(gm.vars.prestige)) {
        gm.vars.prestige = as_i32(&v);
    }
    use crate::feck2::screens::hangout::fe_hangout_screen::DISPLAY_CHEAT_SCREEN;
    if let Some(v) = t.display_cheat_screen.sync(Bool(DISPLAY_CHEAT_SCREEN.load(Relaxed))) {
        DISPLAY_CHEAT_SCREEN.store(as_bool(&v), Relaxed);
    }
    use crate::feck2::feck2_manager::ENABLE_2D;
    if let Some(v) = t.enable_2d.sync(Bool(ENABLE_2D.load(Relaxed))) {
        ENABLE_2D.store(as_bool(&v), Relaxed);
    }
    use crate::feck2::screens::common::fe_3d_vehicles::{BOUNCE_HEIGHT, BOUNCE_OSCILLATIONS, BOUNCE_TIME};
    if let Some(v) = t.bounce_time.sync(Float(f32::from_bits(BOUNCE_TIME.load(Relaxed)))) {
        BOUNCE_TIME.store(as_f32(&v).to_bits(), Relaxed);
    }
    if let Some(v) = t.bounce_height.sync(Float(f32::from_bits(BOUNCE_HEIGHT.load(Relaxed)))) {
        BOUNCE_HEIGHT.store(as_f32(&v).to_bits(), Relaxed);
    }
    if let Some(v) = t.bounce_oscillations.sync(Int(BOUNCE_OSCILLATIONS.load(Relaxed))) {
        BOUNCE_OSCILLATIONS.store(as_i32(&v), Relaxed);
    }
    use crate::feck2::screens::common::fe_3d_camera::{ENABLE_DURATION_CHANGE, MOVE_DURATION_TWEAK};
    if let Some(v) = t.enable_duration_change.sync(Bool(ENABLE_DURATION_CHANGE.load(Relaxed))) {
        ENABLE_DURATION_CHANGE.store(as_bool(&v), Relaxed);
    }
    if let Some(v) = t.move_duration.sync(Float(f32::from_bits(MOVE_DURATION_TWEAK.load(Relaxed)))) {
        MOVE_DURATION_TWEAK.store(as_f32(&v).to_bits(), Relaxed);
    }
    if let Some(mut u) = unlocks {
        if let Some(v) = t.unlock_all_events.sync(Bool(u.events)) {
            u.events = as_bool(&v);
        }
        if let Some(v) = t.unlock_all_vehicles.sync(Bool(u.vehicles)) {
            u.vehicles = as_bool(&v);
            // GetEntryLockState 0x0018D2A0 reads the global live; the FE's copy of it
            if let Some(fe) = fe.as_deref_mut() {
                fe.quick_race.all_unlocked = u.vehicles;
            }
        }
    }
    // dword_7065A4 lives in the tweak; graphics reads `!= 0` (FECK2ElementCamera 0x002999B0, CarRender 0x00111B20)
    let idx = t.garage_camera_index.cell.i32();
    t.garage_camera_index.sync(Int(idx));
    if let Some(mut r) = fe_render {
        if r.garage != (idx != 0) {
            r.garage = idx != 0;
        }
    }
    if let Some(fe) = fe.as_deref_mut() {
        for k in 0..4 {
            for a in 0..3 {
                if let Some(v) = t.camera_pos[k][a].sync(Float(fe.foyer.table[k + 1][0][a])) {
                    fe.foyer.table[k + 1][0][a] = as_f32(&v);
                }
            }
        }
    }

    // registration after core's TweaksReset (see the module header)
    if let Some(set) = set_for_mode(gm.mode) {
        if gm.tweaks.find_system("Player Info").is_none() {
            log::debug!("Tweak Add Frontend ({set:?})");
            register(&mut gm.tweaks, t, set);
        }
    }

    // MessageManager channel 1 -> FECK2Manager.cpp 0x00202EA0
    let Some(messages) = messages else { return };
    let names: Vec<String> = cursor
        .read(&messages)
        .filter_map(|m| match m {
            velocity_game::message::message_manager::GameMessage::TweakChanged(n) => Some(n.clone()),
            #[allow(unreachable_patterns)]
            _ => None,
        })
        .collect();
    let mut career = career;
    for name in names {
        tweak_changed(&name, &mut gm, fe.as_deref_mut(), career.as_deref_mut());
    }
}

/// FECK2Manager.cpp 0x00202EA0 — the tweak-changed callback (the original compares the tweak's data pointer)
pub fn tweak_changed(
    name: &str,
    gm: &mut velocity_game::GameManager,
    fe: Option<&mut crate::feck2::feck2_manager::Feck2>,
    career: Option<&mut velocity_game::game_mode::career::Career>,
) {
    let value = |gm: &velocity_game::GameManager, sys: &str, n: &str| {
        let s = gm.tweaks.find_system(sys)?;
        let id = gm.tweaks.find_tweak(s, n)?;
        Some(gm.tweaks.tweak(id).cell.bool())
    };
    match name {
        // &dword_6F71A0: EventDatabase 0x0008F020(db, 0) + CVehicleDatabase::UnhideVehicles (gpVehicleDatabase)
        "Player Info.Prestige Points" => {
            let g = &mut *gm;
            g.event_db.evaluate_unlocks(&g.vars, false);
            let p = g.vars.prestige as u32;
            if let Some(db) = career.and_then(|c| c.vehicles.as_mut()) {
                db.unhide_vehicles(p);
            }
            if let Some(db) = fe.and_then(|f| f.vehicles.as_mut()) {
                db.unhide_vehicles(p);
            }
        }
        // &byte_6F5B75: EventDatabaseTrackLoading 0x00094990(db, on); the port has the game's and the FE's copy
        "System.Scan Track Folder" => {
            let on = value(gm, "System", "Scan Track Folder").unwrap_or(false);
            gm.event_db.set_all_unlocked(on);
            if let Some(fe) = fe {
                fe.events.set_all_unlocked(on);
            }
        }
        // &byte_6F0D70 set: 0x00092CD0(event, 1) for every event
        "Frontend.Unlock ALL Events" => {
            if value(gm, "Frontend", "Unlock ALL Events").unwrap_or(false) {
                gm.event_db.unlock_all_events();
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(tm: &CTweakManager, path: &str) -> Vec<String> {
        let s = tm.find_system(path).unwrap_or_else(|| panic!("{path}"));
        tm.systems[s].tweaks.iter().map(|v| v.name.clone()).collect()
    }

    #[test]
    fn frontend_tree_matches_the_original() {
        use TweakData::{Bool, Float, Int};
        let t = FrontendTweaks::default();
        let mut tm = CTweakManager::default();
        register(&mut tm, &t, TweakSet::FrontEnd);
        // children are added at the head (0x000865B0): newest system first
        let top: Vec<&str> = tm.systems[0].children.iter().map(|&c| tm.systems[c].name.as_str()).collect();
        assert_eq!(top, ["Frontend", "Player Info"]);
        let fe = tm.find_system("Frontend").unwrap();
        let kids: Vec<&str> = tm.systems[fe].children.iter().map(|&c| tm.systems[c].name.as_str()).collect();
        assert_eq!(kids, ["Characters", "Garage", "Vehicles", "Camera", "FELights"]);
        assert_eq!(rows(&tm, "Player Info"), ["Cash", "Prestige Points"]);
        assert_eq!(rows(&tm, "Frontend"), ["Enable empty environment", "Enable 2D", "Enable Memory Output", "Display cheat screen", "Unlock ALL Events"]);
        assert_eq!(
            rows(&tm, "Frontend.Vehicles"),
            [
                "Unlock ALL Vehicles",
                "Enable Showroom level cars in customisation",
                "Wheel change bounce time",
                "Wheel change bounce height",
                "Wheel change bounce oscillations",
                "Submission Sheet screenshots - ALL",
                "Submission Sheet screenshots - THIS CAR",
                "Output car colour",
                "Enable High Showroom Cars",
                "Enable Showroom screenshots",
                "Enable Hidden Showroom screenshots",
            ]
        );
        assert_eq!(rows(&tm, "Frontend.Camera"), ["Enable Duration Change", "Move Duration", "Free Roam", "Output camera position", "Change Up Axis"]);
        assert_eq!(rows(&tm, "Frontend.Camera.Debug Artist Cam").len(), 14);
        assert_eq!(rows(&tm, "Frontend.FELights"), ["AmbientR", "AmbientG", "AmbientB", "Point Lit Car", "WriteFile"]);
        assert_eq!(rows(&tm, "Frontend.FELights.Light2"), ["R", "G", "B", "Rotation", "Elevation"]);
        assert_eq!(rows(&tm, "Frontend.Characters"), ["Character Camera"]);
        assert_eq!(rows(&tm, "Frontend.Garage"), ["Sell back multiplier"]);
        assert!(tm.find_system("Game.OSD").is_none());
        // ranges / steps / defaults
        let get = |p: &str, n: &str| tm.tweak(tm.find_tweak(tm.find_system(p).unwrap(), n).unwrap());
        let cash = get("Player Info", "Cash");
        assert_eq!((cash.min.clone(), cash.max.clone(), cash.step.clone()), (Int(0), Int(1_000_000), Int(1000)));
        let md = get("Frontend.Camera", "Move Duration");
        assert_eq!((md.min.clone(), md.max.clone(), md.default.clone()), (Float(0.35), Float(0.65), Float(0.65)));
        assert_eq!(get("Frontend.Vehicles", "Unlock ALL Vehicles").default, Bool(true));
        assert_eq!(get("Frontend", "Enable 2D").default, Bool(true));
        let gc = get("Frontend.Camera.Debug Artist Cam", "GarageCameraIndex");
        assert_eq!(gc.enumerators.len(), 5);
        assert_eq!(gc.enumerators[4], ("CAM_FULL".to_string(), 4));
        assert!(get("Frontend.Camera.Debug Artist Cam", "GarageCamera FOV ").default == Int(30));
        // the shared byte_736554: one cell behind two tweaks
        get("Frontend.Characters", "Character Camera").cell.set(Bool(true));
        assert!(get("Frontend.Camera", "Free Roam").cell.bool());
    }

    #[test]
    fn in_game_set_and_mirror() {
        let mut t = FrontendTweaks::default();
        let mut tm = CTweakManager::default();
        register(&mut tm, &t, set_for_mode(Some(1)).unwrap());
        let top: Vec<&str> = tm.systems[0].children.iter().map(|&c| tm.systems[c].name.as_str()).collect();
        assert_eq!(top, ["Game", "Frontend", "Player Info"]);
        assert!(tm.find_system("Frontend.Vehicles").is_none());
        assert!(set_for_mode(Some(3)).is_none());
        let s = tm.find_system("Game.OSD").unwrap();
        let st = tm.tweak(tm.find_tweak(s, "Split Time Display Duration").unwrap());
        assert_eq!((st.max.clone(), st.step.clone(), st.default.clone()), (TweakData::Float(10.0), TweakData::Float(0.5), TweakData::Float(2.0)));
        // mirror: the variable flows in, a menu edit flows out once
        assert_eq!(t.cash.sync(TweakData::Int(250_000)), None);
        assert_eq!(t.cash.cell.i32(), 250_000);
        let id = tm.find_tweak(tm.find_system("Player Info").unwrap(), "Cash").unwrap();
        tm.tweak_mut(id).increment(1.0);
        assert_eq!(t.cash.sync(TweakData::Int(250_000)), Some(TweakData::Int(251_000)));
        assert_eq!(t.cash.sync(TweakData::Int(251_000)), None);
    }
}
