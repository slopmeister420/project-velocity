//! glue: VsAIRaceVehicle 0x00023310 for the core readers (chase camera 0x00067510, zone FX 0x00162D10) — per player
//! race vehicle, its trailblazer (+0x3C8) while it runs (0x00024F10: +0x8C spline set) gives its +0xA8 progress.

use bevy::prelude::*;
use velocity_ai::plugin::AiState;
use velocity_game::game_mode::trail_blazer::PlayerTrailBlazers;
use velocity_game::vs_common_main::{GameTick, InGameUpdateSet};

fn player_trail_blazers(ai: Option<Res<AiState>>, mut out: ResMut<PlayerTrailBlazers>) {
    let players = ai.as_ref().and_then(|a| a.world.as_ref()).map(|w| &w.race.players);
    let v = PlayerTrailBlazers(std::array::from_fn(|p| {
        players
            .and_then(|l| l.get(p))
            .and_then(|v| v.trail_blazer.as_ref())
            .filter(|t| t.active())
            .map(|t| t.progress)
    }));
    if out.0 != v.0 {
        *out = v;
    }
}

pub fn build(app: &mut App) {
    app.init_resource::<PlayerTrailBlazers>().add_systems(
        GameTick,
        player_trail_blazers
            .after(InGameUpdateSet::Ai)
            .before(InGameUpdateSet::Cameras)
            .run_if(in_state(velocity_game::EGameMode::InGame)),
    );
}
