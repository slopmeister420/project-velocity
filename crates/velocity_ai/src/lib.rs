//! Race, traffic and police AI (src/Ai, OpponentDatabase, TeamDatabase).

use std::path::{Path, PathBuf};

use bevy::prelude::*;

pub mod ai;
pub mod ai_tweaks;
pub mod barrier_render;
pub mod crc;
pub mod opponent_database;
pub mod physics_ai;
pub mod plugin;
pub mod race_car_source;
pub mod team_database;
pub mod text_file;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        plugin::build(app);
    }
}

/// the game install as resolved by `visos::file` (`--game` / `VELOCITY_GAME_DIR` / exe dir), if present.
pub fn game_dir() -> Option<PathBuf> {
    let p = visos::file::platform::game_dir().to_path_buf();
    p.is_dir().then_some(p)
}

/// case-insensitive lookup of `name` (xbox style, `\` separated) under `root`.
pub fn resolve(root: &Path, name: &str) -> PathBuf {
    visos::file::platform::resolve_in(root, name)
}
