//! port (not original): the PC settings page's options (`GameOptions.pc`, edited on
//! `feck2::screens::options::fe_pc_options_screen`) reaching the window (mode, fullscreen resolution, present mode),
//! the frame limiter (`FrameRateCap`, read by the velocity bin's `frame_cap.rs`) and audio's per-launch music
//! randomiser. Command-line `--fps` wins over the cap and v-sync, `VELOCITY_WINDOW` over the window mode.

use bevy::prelude::*;
use bevy::window::{Monitor, MonitorSelection, PresentMode, PrimaryMonitor, PrimaryWindow, VideoModeSelection, WindowMode};

use crate::game_options::{display_mode, GameOptions, PcSettings};

/// what the PC settings page shows for the resolution row.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PcDisplayInfo {
    /// fullscreen choices, smallest first: the current monitor's mode sizes plus the window size
    pub resolutions: Vec<(u32, u32)>,
    /// the window's physical size
    pub window: (u32, u32),
    /// the current monitor's desktop size
    pub desktop: (u32, u32),
}

impl PcDisplayInfo {
    /// the size the resolution row shows for `p`: windowed = the window, borderless = the desktop, fullscreen =
    /// the chosen one (the desktop when none)
    pub fn shown(&self, p: &PcSettings) -> (u32, u32) {
        match p.display_mode {
            display_mode::WINDOWED => self.window,
            display_mode::BORDERLESS => self.desktop,
            _ => p.resolution.unwrap_or(self.desktop),
        }
    }

    /// the next fullscreen resolution from `cur` in `dir` (clamped at the ends)
    pub fn step(&self, cur: (u32, u32), dir: i32) -> Option<(u32, u32)> {
        let r = &self.resolutions;
        if r.is_empty() {
            return None;
        }
        let i = match r.iter().position(|&x| x == cur) {
            Some(i) => (i as i32 + dir).clamp(0, r.len() as i32 - 1) as usize,
            // not in the list: the first one past it in that direction
            None if dir > 0 => r.iter().position(|&x| x > cur).unwrap_or(r.len() - 1),
            None => r.iter().rposition(|&x| x < cur).unwrap_or(0),
        };
        Some(r[i])
    }
}

/// the render-rate cap (None = uncapped) the velocity bin's frame limiter applies; `cli` = `--fps` was given and wins.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameRateCap {
    pub fps: Option<u32>,
    pub cli: bool,
}

impl FrameRateCap {
    pub fn from_args() -> Self {
        Self { fps: None, cli: std::env::args().any(|a| a == "--fps") }
    }
}

fn window_override() -> bool {
    std::env::var_os("VELOCITY_WINDOW").is_some()
}

fn current_monitor<'a>(monitors: &'a Query<(&Monitor, Has<PrimaryMonitor>)>) -> Option<&'a Monitor> {
    monitors.iter().find(|(_, p)| *p).or_else(|| monitors.iter().next()).map(|(m, _)| m)
}

/// the monitor's mode sizes + the window size, sorted and deduplicated.
pub fn resolutions(m: Option<&Monitor>, window: (u32, u32)) -> Vec<(u32, u32)> {
    let mut r: Vec<(u32, u32)> = m.map(|m| m.video_modes.iter().map(|v| (v.physical_size.x, v.physical_size.y)).collect()).unwrap_or_default();
    r.push(window);
    r.retain(|&(w, h)| w > 0 && h > 0);
    r.sort_unstable();
    r.dedup();
    r
}

/// fills `Feck2.pc_display` when the window size or the monitor changes.
pub fn pc_display_info(
    mut st: ResMut<crate::FrontendState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    monitors: Query<(&Monitor, Has<PrimaryMonitor>)>,
    mut key: Local<((u32, u32), (u32, u32), usize)>,
) {
    let Some(f) = st.frontend.as_mut() else { return };
    let window = windows.single().map_or((0, 0), |w| (w.physical_width(), w.physical_height()));
    let m = current_monitor(&monitors);
    let desktop = m.map_or(window, |m| (m.physical_width, m.physical_height));
    let k = (window, desktop, m.map_or(0, |m| m.video_modes.len()));
    if *key == k && !f.fe.pc_display.resolutions.is_empty() {
        return;
    }
    *key = k;
    f.fe.pc_display = PcDisplayInfo { resolutions: resolutions(m, window), window, desktop };
}

/// the window mode for `p` on monitor `m` (fullscreen: the monitor's mode of that size with the highest refresh).
pub fn window_mode(p: &PcSettings, m: Option<&Monitor>) -> WindowMode {
    match p.display_mode {
        display_mode::BORDERLESS => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        display_mode::FULLSCREEN => {
            let vm = p.resolution.and_then(|(w, h)| {
                m?.video_modes
                    .iter()
                    .filter(|v| v.physical_size == UVec2::new(w, h))
                    .max_by_key(|v| (v.refresh_rate_millihertz, v.bit_depth))
                    .cloned()
            });
            WindowMode::Fullscreen(MonitorSelection::Current, vm.map_or(VideoModeSelection::Current, VideoModeSelection::Specific))
        }
        _ => WindowMode::Windowed,
    }
}

/// applies `GameOptions.pc` when it changes (and once at startup).
pub fn apply_pc_settings(
    options: Res<GameOptions>,
    mut cap: ResMut<FrameRateCap>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    monitors: Query<(&Monitor, Has<PrimaryMonitor>)>,
    audio: Option<ResMut<velocity_audio::VisosAudio>>,
    mut last: Local<Option<PcSettings>>,
) {
    let p = &options.pc;
    if last.as_ref() == Some(p) {
        return;
    }
    let m = current_monitor(&monitors);
    // a fullscreen resolution needs the monitor's modes, which winit reports after the first update
    if p.display_mode == display_mode::FULLSCREEN && p.resolution.is_some() && m.is_none() && !window_override() {
        return;
    }
    if let Some(mut a) = audio {
        if a.system.music_random != p.music_random {
            if p.music_random {
                // keeps the original picks in hidden / golden runs
                a.system.randomize_music_per_launch();
            } else {
                a.system.music_random = false;
            }
        }
    }
    if !cap.cli {
        let fps = (p.fps_cap > 0).then_some(p.fps_cap);
        if cap.fps != fps {
            cap.fps = fps;
        }
    }
    let mut applied = None;
    if let Ok(mut w) = windows.single_mut() {
        if !cap.cli {
            let pm = if p.vsync { PresentMode::Fifo } else { PresentMode::AutoNoVsync };
            if w.present_mode != pm {
                w.present_mode = pm;
            }
        }
        if !window_override() {
            let mode = window_mode(p, m);
            if w.mode != mode {
                w.mode = mode;
            }
        }
        applied = Some((w.present_mode, w.mode));
    }
    info!(
        "pc settings: present mode {:?}, window mode {:?}, frame cap {:?}{}, music random {}",
        applied.map(|a| a.0),
        applied.map(|a| a.1),
        cap.fps,
        if cap.cli { " (--fps)" } else { "" },
        p.music_random
    );
    *last = Some(p.clone());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_steps() {
        let info = PcDisplayInfo { resolutions: vec![(640, 480), (1280, 720), (1920, 1080)], window: (640, 480), desktop: (1920, 1080) };
        assert_eq!(info.step((640, 480), 1), Some((1280, 720)));
        assert_eq!(info.step((1920, 1080), 1), Some((1920, 1080)));
        assert_eq!(info.step((1000, 700), -1), Some((640, 480)));
        assert_eq!(info.step((1000, 700), 1), Some((1280, 720)));
        let mut p = PcSettings::default();
        assert_eq!(info.shown(&p), (640, 480));
        p.display_mode = display_mode::FULLSCREEN;
        assert_eq!(info.shown(&p), (1920, 1080));
        assert_eq!(resolutions(None, (800, 600)), vec![(800, 600)]);
    }
}
