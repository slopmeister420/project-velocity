//! "The Scene" career flow, game side: profiles (the profile select's new / load / save / delete, FE code
//! 0x00234550..0x002347F0 in the idb's "FEOnlineSignInScreen" range), starting a career event
//! (CFEHangoutProgression StartEvent 0x0028F430) and what RaceCompleteScreen does to the career on the way back
//! (Enter 0x002055F0, Start 0x00205830: free a temporary event, autosave, prestige). The screens themselves are
//! the front end's; it (or the binary) asks for these through `GameFlowRequest` and hears `CareerMessage`.

use bevy::prelude::*;
use velocity_audio::vs_audio_track_list::SavedPlaylists;
use velocity_audio::{AudioCommand, VisosAudio};
use velocity_physics::vehicle_database::vehicle_database::{CVehicleDatabase, VehicleSetup};
use visos_render::licence_plate_render::LicencePlate;

use crate::controller::user_input::UserInput;
use crate::event_database::event_database::EVENT_FLAG_TEMP;
use crate::event_database::event_database_track_loading::ERaceType;
use crate::game_mode::game_manager::{GameFlowRequest, GameManager};
use crate::game_mode::game_player::EventRef;
use crate::game_mode::race_cars::{PlayerCarChoice, RaceCarSourceRes};
use crate::register_file_types::save_file_data::{self, OptionsBlock, PrimarySave};
use crate::save_game::save_vars::SaveFlow;

/// SaveFileData's globals and the core's vehicle database (`dword_74B4A8`, the game's copy)
#[derive(Resource, Default)]
pub struct Career {
    /// `byte_701388` the save container (profile) name
    pub save_name: String,
    /// `dword_701428` the last block saved or loaded (keeps the words other crates own)
    pub primary: PrimarySave,
    /// the game's vehicle database (loaded at boot)
    pub vehicles: Option<CVehicleDatabase>,
    /// set when a race starts, cleared by the RaceCompleteScreen pass on the way back
    pub from_race: bool,
    /// the option globals `byte_6F915C..byte_6F9182` (the binary keeps them in step with the front end's
    /// `GameOptions`: front end -> here every frame, here -> front end after `Created` / `Loaded`)
    pub options: OptionsBlock,
}

/// what happened to the career (for the front end / binary to refresh its copies)
#[derive(Message, Clone, Debug, PartialEq)]
pub enum CareerMessage {
    /// a new profile: `SetInitialVariables` ran (0x00234550)
    Created { name: String },
    /// a load finished (0x002337C0 polling, then 0x001050C0); `ok` false = missing or wrong version
    Loaded { name: String, ok: bool },
    Saved { name: String, ok: bool },
    Deleted { name: String, ok: bool },
    /// StartEvent ran: post race screen "RaceCompleteScreen", return screen "HangoutScreen"
    EventStarted { event: u32 },
    /// back in the front end after a race: the career variables changed (outcome, prestige)
    ReturnedFromRace { saved: bool },
}

/// the player's car choice for a setup (vehicle folder, camera views), as the quick race bridge builds it
pub fn car_choice(vdb: &CVehicleDatabase, setup: &VehicleSetup) -> PlayerCarChoice {
    let mut car = PlayerCarChoice::default();
    if let Some(v) = setup.vehicle.and_then(|i| vdb.vehicles.get(i)) {
        car.vehicle_path = Some(v.path.clone());
        car.camera_view = v.data.list_a.first().copied();
        car.camera_views = v.data.list_a.clone();
    }
    car.setup = Some(setup.clone());
    car
}

/// FE 0x00234550 — new profile: SetInitialVariables (GamePlayer 0x000C1930, then the event reset), the garage
/// setups (0x002022F0), the default name (`unk_6F90DC`) into the save and profile names, profile flags on
pub fn new_profile(gm: &mut GameManager, career: &mut Career, choice: &mut PlayerCarChoice, name: &str) {
    let g = &mut *gm;
    g.vars.set_initial_variables();
    g.event_db.reset_events(&mut g.rand);
    if let Some(db) = career.vehicles.as_ref() {
        g.vars.init_garage_setups(db);
        if let Some(s) = g.vars.garage_setup(g.vars.garage_slot as usize) {
            *choice = car_choice(db, s);
        }
    }
    // byte_701397 / byte_6F7183 = 0: 15 characters
    let name: String = name.chars().take(15).collect();
    career.save_name = name.clone();
    g.vars.profile_name = name;
    g.vars.save_enabled = true;
    g.vars.profile_active = true;
    career.primary = PrimarySave::default();
    // SetInitialVariables resets the option globals too (not byte_6F9181)
    career.options = OptionsBlock { controller_config: career.options.controller_config, ..Default::default() };
    g.vars.speech = career.options.speech;
    g.vars.player_names = career.options.player_names;
}

/// SaveFileData 0x001061D0 (0x00106040 the same from the profile select): gather, then save "Primary"
pub fn save_profile(gm: &GameManager, career: &mut Career, ui: &UserInput, audio: Option<&mut VisosAudio>) -> bool {
    if !gather_profile(gm, career, ui, audio) {
        return false;
    }
    let r = save_file_data::save_primary(&career.save_name, &career.primary);
    match r {
        Ok(()) => {
            log::info!("career: saved {}", save_file_data::primary_path(&career.save_name).display());
            true
        }
        Err(e) => {
            log::error!("career: save failed: {e}");
            false
        }
    }
}

/// SaveFileData 0x001061D0 up to the save call: the version, vars, `byte_6F9181` = pad 0's config, the
/// playlists (0x00105E10), the options block copy, then the garage / wheels / parts / races / events gathers
fn gather_profile(gm: &GameManager, career: &mut Career, ui: &UserInput, audio: Option<&mut VisosAudio>) -> bool {
    let Some(db) = career.vehicles.as_ref() else {
        log::error!("save: no vehicle database");
        return false;
    };
    career.options.controller_config = ui.config.first().copied().unwrap_or(0);
    let mut p = std::mem::take(&mut career.primary);
    p.gather(&gm.vars, &gm.event_db, db, &career.options);
    // 0x00105E10: the audio playlists (VsAudioTrackList::GetLists) into options +0x28
    if let Some(a) = audio {
        let b = a.game.get_lists().to_bytes();
        let o = SavedPlaylists::OPTIONS_OFFSET;
        p.options[o..o + b.len()].copy_from_slice(&b);
    }
    career.primary = p;
    true
}

/// FE 0x002345E0 (load "Primary" of the chosen profile) + 0x002337C0 on success: SaveFileData 0x001050C0
pub fn load_profile(
    gm: &mut GameManager,
    career: &mut Career,
    ui: &mut UserInput,
    choice: &mut PlayerCarChoice,
    audio: &mut MessageWriter<AudioCommand>,
    name: &str,
) -> bool {
    let name: String = name.chars().take(127).collect();
    career.save_name = name.clone();
    gm.vars.save_enabled = true;
    gm.vars.profile_active = true;
    gm.vars.profile_name = name.chars().take(15).collect();
    let Some(p) = save_file_data::load_primary(&name) else {
        log::warn!("career: no save for {name}");
        return false;
    };
    let Some(db) = career.vehicles.as_mut() else { return false };
    let g = &mut *gm;
    let Some(a) = p.apply(&mut g.vars, &mut g.event_db, db) else {
        log::warn!("career: {name}: save version mismatch");
        return false;
    };
    // 0x00104F30: the saved playlists -> VsAudioTrackList::SetLists
    let o = SavedPlaylists::OPTIONS_OFFSET;
    audio.write(AudioCommand::SetLists(Box::new(SavedPlaylists::from_bytes(&p.options[o..]))));
    ui.set_config(0, a.controller_config);
    career.options = a.options;
    if let Some(s) = a.current_setup.as_ref() {
        *choice = car_choice(db, s);
    }
    g.vars.count_unlocked_tiers(&g.event_db);
    career.primary = p;
    true
}

/// CFEHangoutProgression 0x0028F430 — start the event in `g_FEGameVariables.m_pEvent`: the first competitor
/// is the player, the event is reset, team / pink slip events race the team's or opponent's car, one player,
/// post race "RaceCompleteScreen" and return "HangoutScreen" (front end). Then the loading screen starts the
/// game (GameManager 0x000BBF60), here the race selection the jump path resolves.
pub fn start_event(gm: &mut GameManager, career: &Career, source: Option<&RaceCarSourceRes>, choice: &mut PlayerCarChoice) -> bool {
    let g = &mut *gm;
    let Some(ev) = g.vars.current_event_mut(&mut g.event_db) else {
        log::error!("assert: pEvent != NULL");
        return false;
    };
    if let Some(c) = ev.competitors.as_mut().and_then(|c| c.first_mut()) {
        c.human = 1;
    }
    ev.reset(&mut g.rand);
    let (category, unk_44, tier, unlock_team) = (ev.category, ev.unknown_44, ev.tier, ev.unlock_team);
    // 0x000920E0 clears dword_6F7244
    g.vars.event_state = 0;
    let db = career.vehicles.as_ref();
    let team_car = |team: u32| -> Option<VehicleSetup> { source.and_then(|s| db.and_then(|d| s.0.team_car_by_team(d, team))) };
    if category == 2 && g.vars.team != 0 {
        match team_car(g.vars.team) {
            Some(s) => {
                g.vars.garage_vehicle = s.vehicle.and_then(|v| db?.vehicles.get(v)).map_or(0, |v| v.hash);
                *choice = car_choice(db.unwrap(), &s);
            }
            None => log::error!("assert: Problem getting team car information"),
        }
    } else if category == 4 {
        if let (Some(src), Some(d)) = (source, db) {
            if let Some((s, _)) = src.0.opponent_car(d, unk_44, tier as usize) {
                *choice = car_choice(d, &s);
            }
        }
    } else if category == 3 {
        if unlock_team != 0 {
            if let Some(s) = team_car(unlock_team) {
                g.vars.garage_vehicle = s.vehicle.and_then(|v| db?.vehicles.get(v)).map_or(0, |v| v.hash);
                *choice = car_choice(db.unwrap(), &s);
            }
        } else {
            log::warn!("WARNING: Team challenge event does not have a valid team ID.");
        }
    }
    // dword_6F5C00 = 1
    g.num_players = 1;
    start_current_event(g)
}

/// GameManager 0x000BBF60 (+0x214 = 1, +0x218 = 0; from the LoadingScreen's 10th frame): race `m_pEvent`'s current
/// race as it is — the end of StartEvent, and DummyGameScreen's "Play Race" (next race of an unfinished event, no
/// reset). Professional = category 2, dry.
pub fn start_current_event(g: &mut GameManager) -> bool {
    let Some(ev) = g.vars.current_event(&g.event_db) else {
        log::error!("assert: pEvent != NULL");
        return false;
    };
    let category = ev.category;
    let Some(r) = ev.current_race().and_then(|r| r.race) else {
        log::error!("career: the event's current race is not in TRACKS.INI");
        return false;
    };
    let race = &g.event_db.races[r];
    let track = g.event_db.tracks[race.track].path.clone();
    let path = race.path.clone();
    let drag = race.race_type == ERaceType::Drag;
    g.jump.set(&track, &path, category == 2, false, drag);
    g.career_start = true;
    true
}

/// RaceCompleteScreen Enter (0x002055F0) and Start (0x00205830), game side: a temporary event is freed; unless the
/// race was quit (`byte_6F5D08` = GameManager +0x140 `m_bQuitRace`: pause menu quit, attract mode end) the career
/// is saved when a profile is in use; then the event's prestige (`dword_6F7244`) is added — after the save, as in
/// the original.
/// The save goes through the save flow (0x001061D0 -> `SAVE_Flow_Save_M` 0x00300680): the file is written at
/// once, the front end polls `SaveFlow` for the Saving / SaveComplete popups (or the error question).
pub fn race_complete(
    gm: &mut GameManager,
    career: &mut Career,
    ui: &UserInput,
    audio: Option<&mut VisosAudio>,
    flow: &mut SaveFlow,
) -> bool {
    if matches!(&gm.vars.event, Some(EventRef::Temp(e)) if e.flags & EVENT_FLAG_TEMP != 0) {
        gm.vars.event = None;
    }
    let saved = !gm.quit_race && gm.vars.save_enabled && gather_profile(gm, career, ui, audio) && {
        flow.start_save(save_file_data::primary_path(&career.save_name), career.primary.to_bytes(), true);
        flow.written() == Some(true)
    };
    let non_temp = gm.vars.current_event(&gm.event_db).is_some_and(|e| e.flags & EVENT_FLAG_TEMP == 0);
    if non_temp {
        gm.vars.prestige = gm.vars.prestige.wrapping_add(gm.vars.event_state as i32);
    }
    saved
}

/// career requests (read every frame next to `game_flow_requests`)
#[allow(clippy::too_many_arguments)]
pub fn career_requests(
    mut gm: ResMut<GameManager>,
    mut career: ResMut<Career>,
    mut ui: ResMut<UserInput>,
    mut choice: ResMut<PlayerCarChoice>,
    source: Option<Res<RaceCarSourceRes>>,
    mut r: MessageReader<GameFlowRequest>,
    mut out: MessageWriter<CareerMessage>,
    mut audio_out: MessageWriter<AudioCommand>,
    mut audio: Option<ResMut<VisosAudio>>,
    mut plate: Option<ResMut<LicencePlate>>,
) {
    // the plate globals are GamePlayer's (`dword_6F71DC`, `unk_6F71E0`); graphics' `LicencePlate` is the copy the
    // render and PlateCustomizationScreen use: take the screen's edits, give back what a new / loaded profile has
    if let Some(p) = plate.as_deref() {
        if (gm.vars.plate_style(), gm.vars.plate_text().as_str()) != (p.style, p.text.as_str()) {
            *gm.vars.plate_style_mut() = p.style;
            gm.vars.set_plate_text(&p.text);
        }
    }
    let push_plate = |gm: &GameManager, plate: &mut Option<ResMut<LicencePlate>>| {
        if let Some(p) = plate.as_deref_mut() {
            p.style = gm.vars.plate_style();
            p.text = gm.vars.plate_text();
        }
    };
    for m in r.read() {
        match m {
            GameFlowRequest::NewProfile { name } => {
                new_profile(&mut gm, &mut career, &mut choice, name);
                push_plate(&gm, &mut plate);
                out.write(CareerMessage::Created { name: career.save_name.clone() });
            }
            GameFlowRequest::LoadProfile { name } => {
                let ok = load_profile(&mut gm, &mut career, &mut ui, &mut choice, &mut audio_out, name);
                if ok {
                    push_plate(&gm, &mut plate);
                }
                out.write(CareerMessage::Loaded { name: name.clone(), ok });
            }
            GameFlowRequest::SaveProfile => {
                let ok = save_profile(&gm, &mut career, &ui, audio.as_deref_mut());
                out.write(CareerMessage::Saved { name: career.save_name.clone(), ok });
            }
            GameFlowRequest::DeleteProfile { name } => {
                let ok = save_file_data::delete_primary(name).is_ok();
                out.write(CareerMessage::Deleted { name: name.clone(), ok });
            }
            GameFlowRequest::StartCareerEvent { event } => {
                let Some(i) = gm.event_db.find_event(*event) else {
                    log::error!("assert: g_FEGameVariables.m_pEvent != NULL && \"Unable to find event to start.\"");
                    continue;
                };
                gm.vars.event = Some(EventRef::Database(i));
                if start_event(&mut gm, &career, source.as_deref(), &mut choice) {
                    career.from_race = true;
                    out.write(CareerMessage::EventStarted { event: *event });
                }
            }
            GameFlowRequest::StartCurrentEvent => {
                let crc = gm.vars.current_event(&gm.event_db).map(|e| e.crc);
                if let Some(crc) = crc
                    && start_current_event(&mut gm)
                {
                    career.from_race = true;
                    out.write(CareerMessage::EventStarted { event: crc });
                }
            }
            _ => {}
        }
    }
}

/// entering the front end after a race: RaceCompleteScreen's career part (see `race_complete`)
pub fn on_front_end(
    mut gm: ResMut<GameManager>,
    mut career: ResMut<Career>,
    ui: Res<UserInput>,
    mut out: MessageWriter<CareerMessage>,
    mut audio: Option<ResMut<VisosAudio>>,
    mut flow: ResMut<SaveFlow>,
) {
    if !career.from_race {
        return;
    }
    career.from_race = false;
    let saved = race_complete(&mut gm, &mut career, &ui, audio.as_deref_mut(), &mut flow);
    out.write(CareerMessage::ReturnedFromRace { saved });
}

/// any race start marks the way back (quick races go through RaceCompleteScreen too)
pub fn on_in_game(mut career: ResMut<Career>) {
    career.from_race = true;
}
