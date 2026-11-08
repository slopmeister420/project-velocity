//! glue: the ai raceline length game flow needs for the race mileage (the opponents', teams' and police cars come
//! from the ai crate's `race_car_source`, installed by its plugin).

use bevy::prelude::*;
use velocity_ai::plugin::AiState;
use velocity_game::game_mode::race_cars::RacelineLength;

/// `VsAIWorld::GetFirstRaceSection` 0x00024300 -> its raceline's length, for EventDatabase 0x00091620
fn raceline_length(ai: Option<Res<AiState>>, mut len: ResMut<RacelineLength>) {
    let l = ai.as_ref().and_then(|a| a.world.as_ref()).and_then(|w| w.world.get_race_section(0)).map(|s| s.get_raceline().get_length());
    if len.0 != l {
        len.0 = l;
    }
}

pub fn build(app: &mut App) {
    app.add_systems(Update, raceline_length);
}
