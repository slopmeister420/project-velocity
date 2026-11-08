//! glue: the attract mode in the race (`g_AttractMode`, core `font_helper::CAttractMode`; the front end's main
//! menu makes the FEMainMenuScreen calls 0x000B30E0 / 0x000B34A0 / 0x000B3470 itself).
//! GameManager 0x000BD8B0 (countdown, attract or replay) puts every player race vehicle under ai control
//! (`RaceSessionMessage::AiTakesRacers`), InGameUpdate's review switch 0x000B7D20 player 0 (`PlayerHumanControl`):
//! VsAIPoliceVehicle 0x00018920 on the ai's player vehicles, read in the writing tick.

use bevy::prelude::*;
use velocity_ai::plugin::{AiPhysics, AiState};
use velocity_game::game_mode::race_session::RaceSessionMessage;
use velocity_game::replays::cameras::replay_camera_manager::PlayerHumanControl;
use velocity_game::vs_common_main::{GameTick, InGameUpdateSet};
use velocity_game::EGameMode;

/// SetHumanControl on the ai's player race vehicles (0x000BD8B0: all of them off; the review switch: player 0)
fn player_ai_control(
    mut session: MessageReader<RaceSessionMessage>,
    mut review: MessageReader<PlayerHumanControl>,
    ai: Option<ResMut<AiState>>,
    phys: Option<ResMut<AiPhysics>>,
) {
    let mut calls: Vec<(Option<usize>, bool)> = Vec::new();
    for m in session.read() {
        if matches!(m, RaceSessionMessage::AiTakesRacers) {
            calls.push((None, false));
        }
    }
    for m in review.read() {
        calls.push((Some(m.player), m.human));
    }
    if calls.is_empty() {
        return;
    }
    let (Some(mut ai), Some(mut phys)) = (ai, phys) else { return };
    let Some(w) = ai.world.as_mut() else { return };
    let time = w.world.time;
    for (only, human) in calls {
        for (i, v) in w.race.players.iter_mut().enumerate() {
            if only.is_some_and(|p| p != i) {
                continue;
            }
            let Some(h) = v.base.physics else { continue };
            if v.human_control != human {
                log::info!("attract / review: player {i} human control {human}");
            }
            v.set_human_control_on(human, &w.world, phys.0.vehicle(h), time);
        }
    }
}

pub fn build(app: &mut App) {
    app.add_systems(
        GameTick,
        player_ai_control
            .after(InGameUpdateSet::UpdateSessions)
            .after(InGameUpdateSet::Cameras)
            .run_if(in_state(EGameMode::InGame)),
    );
}
