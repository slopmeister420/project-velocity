//! Game flow: modes, events, timelines, triggers, cameras, replays, scoring (src/GameMode and friends).

pub mod camera;
pub mod career_teams;
pub mod cli;
pub mod controller;
pub mod core_;
pub mod event_database;
pub mod font_helper;
pub mod game_bridge;
pub mod game_data_loading;
pub mod game_input;
pub mod game_menu_manager;
pub mod game_mode;
pub mod main_cpp;
pub mod message;
pub mod register_file_types;
pub mod replays;
pub mod save_game;
pub mod score_manager;
pub mod timeline_manager;
pub mod triggers;
pub mod vs_common_main;

use bevy::prelude::*;

pub use game_mode::game_manager::{
    EGameMode, GameFlowRequest, GameManager, GameMenuMessage, GameModeMessage, InRace, LoadStepMessage, PauseMenuChoice,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        game_mode::game_manager::build_with(app, main_cpp::CommandLine::from_env());
    }
}
