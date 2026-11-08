//! Project Velocity (Visual Sciences, F3 MainBranch) — src/main.cpp + src/VsCommonMain.cpp.
//!
//! The binary also hosts the glue between crates that cannot depend on each other (`bridge`), the in-race HUD
//! overlay (`hud`) and the scripted `--smoke-test` run.

// no console window in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai_host;
mod attract;
mod barriers;
mod bridge;
mod car_audio;
mod debug_menu;
mod frame_cap;
mod hud;
mod puddles;
mod race_car_source;
mod recorder;
mod smoke_test;
mod zone_trails;
mod window_keep;
mod stream_raceline;
mod trail_blazer_state;

use bevy::prelude::*;

fn main() {
    if std::env::args().any(|a| a == "--slice-test" || a == "--smoke-test") {
        smoke_test::fresh_save_dir();
    }
    // game dir: --game <dir> -> VELOCITY_GAME_DIR -> exe dir with default.xbe
    if let Err(e) = visos::file::platform::init_game_dir_from_args() {
        eprintln!("{e}");
        #[cfg(not(debug_assertions))]
        message_box(&e.to_string());
        std::process::exit(1);
    }
    let slice = std::env::args().any(|a| a == "--slice-test" || a == "--smoke-test");
    let fps = frame_cap::fps_arg();
    let mut app = App::new();
    // bevy's AudioPlugin is always off: velocity_audio owns the output device (audio/output.rs; the recorder
    // pulls the mix itself while recording)
    let plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Project Velocity".into(),
                // dev: VELOCITY_WINDOW=<w>x<h> (aspect tests of the hud; headless captures)
                resolution: std::env::var("VELOCITY_WINDOW")
                    .ok()
                    .and_then(|v| v.split_once('x').and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?))))
                    .unwrap_or((640, 480))
                    .into(),
                present_mode: frame_cap::present_mode(fps),
                ..default()
            }),
            ..default()
        })
        // all pads come through the visos input layer (SDL3); gilrs would read them a second time
        .set(bevy::log::LogPlugin { custom_layer: file_log_layer, ..default() })
        .disable::<bevy::gilrs::GilrsPlugin>()
        .disable::<bevy::audio::AudioPlugin>();
    app.add_plugins(plugins);
    // a monitor re-enumeration must not despawn the window with its monitor (window_keep.rs)
    window_keep::build(&mut app);
    // winit registered keyboard + mouse raw input when WinitPlugin built its event loop; the game needs no
    // device events (keys come from window messages, pads from SDL), so drop it in every run
    // (= DeviceEvents::Never). A hidden run must end with none left.
    let left = visos::input::raw_input::remove_keyboard_mouse();
    if left != 0 && visos::input::raw_input::hidden_run() {
        eprintln!("hidden run: {left} keyboard/mouse raw input registrations remain, refusing to run");
        std::process::exit(3);
    }
    app.add_plugins((
        visos_render::RenderPlugin,
        velocity_physics::PhysicsPlugin,
        velocity_ai::AiPlugin,
        velocity_audio::AudioPlugin,
        velocity_frontend::FrontendPlugin,
        velocity_game::GamePlugin,
    ));
    bridge::build(&mut app);
    barriers::build(&mut app);
    puddles::build(&mut app);
    zone_trails::build(&mut app);
    race_car_source::build(&mut app);
    hud::build(&mut app);
    debug_menu::build(&mut app);
    stream_raceline::build(&mut app);
    trail_blazer_state::build(&mut app);
    car_audio::build(&mut app);
    attract::build(&mut app);
    ai_host::build(&mut app);
    frame_cap::build(&mut app, fps);
    if slice {
        smoke_test::build(&mut app);
        recorder::build(&mut app);
    }
    app.run();
}

/// opt-in (`--log` or `VELOCITY_LOG=1`): `velocity.log` next to the exe (fallback %LOCALAPPDATA%\Velocity),
/// overwritten per run, unbuffered so a crash or kill still leaves the log; panics are logged into it too.
fn file_log_layer(_app: &mut App) -> Option<bevy::log::BoxedLayer> {
    use bevy::log::tracing_subscriber::{fmt, Layer};
    if !std::env::args().any(|a| a == "--log") && std::env::var_os("VELOCITY_LOG").is_none() {
        return None;
    }
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    let local = std::env::var_os("LOCALAPPDATA").map(|d| std::path::PathBuf::from(d).join("Velocity"));
    let file = [exe_dir, local].into_iter().flatten().find_map(|d| {
        let _ = std::fs::create_dir_all(&d);
        std::fs::File::create(d.join("velocity.log")).ok()
    })?;
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        bevy::log::error!("panic: {info}");
        prev(info);
    }));
    Some(fmt::layer().with_ansi(false).with_writer(std::sync::Mutex::new(file)).boxed())
}

#[cfg(not(debug_assertions))]
fn message_box(text: &str) {
    // no console in release: show the game-dir error in a dialog
    #[cfg(windows)]
    {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn MessageBoxW(hwnd: *mut core::ffi::c_void, text: *const u16, caption: *const u16, kind: u32) -> i32;
        }
        let w = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
        let (t, c) = (w(text), w("Project Velocity"));
        // MB_ICONERROR
        unsafe { MessageBoxW(core::ptr::null_mut(), t.as_ptr(), c.as_ptr(), 0x10) };
    }
    #[cfg(not(windows))]
    let _ = text;
}
