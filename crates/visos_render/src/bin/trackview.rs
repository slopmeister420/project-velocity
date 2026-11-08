//! headless track viewer: `cargo run -p visos_render --bin trackview -- --track ExecTest --shot out.png`
//! (without `--shot` it opens a window with a fly camera: right mouse look, WASD/QE, shift fast).

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use visos_render::track_view::{TrackViewConfig, TrackViewPlugin};

fn main() {
    if let Err(e) = visos::file::platform::init_game_dir_from_args() {
        eprintln!("{e}");
        std::process::exit(1);
    }
    let cfg = TrackViewConfig::from_args().expect("--track <name> required");
    let mut app = App::new();
    if cfg.shot.is_some() {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, ..default() })
                .disable::<WinitPlugin>(),
        )
        // golden mode times frames: no runner wait
        .add_plugins(ScheduleRunnerPlugin::run_loop(if visos_render::golden::enabled() {
            std::time::Duration::ZERO
        } else {
            std::time::Duration::from_secs_f64(1.0 / 60.0)
        }));
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: format!("trackview {}", cfg.track), ..default() }),
            ..default()
        }));
    }
    app.add_plugins(TrackViewPlugin(cfg)).run();
}
