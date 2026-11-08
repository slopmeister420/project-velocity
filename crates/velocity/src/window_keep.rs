//! Keeps the game window alive when winit re-enumerates the monitors.
//!
//! bevy_window 0.19 links windows to their monitor (`OnMonitor` -> `HasWindows`, `linked_spawn`), and
//! bevy_winit's `create_monitors` despawns a `Monitor` entity as soon as its handle is missing from one
//! `available_monitors()` pass. On Windows the HMONITORs change on any display reconfiguration (monitor off,
//! resolution change, a desktop switch elsewhere), so the primary window was despawned with its monitor and
//! the app quit with "No windows are open" (the sweep's intermittent `no_race`: "Monitor removed" x2 right
//! before it). Nothing in the game reads `OnMonitor`, so windows never keep the link.

use bevy::prelude::*;
use bevy::window::OnMonitor;

pub fn build(app: &mut App) {
    app.add_observer(unlink_window_monitor);
}

fn unlink_window_monitor(ev: On<Insert, OnMonitor>, mut commands: Commands) {
    if let Ok(mut e) = commands.get_entity(ev.entity) {
        e.try_remove::<OnMonitor>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::window::Monitor;

    fn run(keep: bool) -> bool {
        let mut app = App::new();
        if keep {
            build(&mut app);
        }
        let monitor = app
            .world_mut()
            .spawn(Monitor {
                name: None,
                physical_height: 480,
                physical_width: 640,
                physical_position: IVec2::ZERO,
                refresh_rate_millihertz: None,
                scale_factor: 1.0,
                video_modes: Vec::new(),
            })
            .id();
        let window = app.world_mut().spawn((Window::default(), OnMonitor(monitor))).id();
        app.update();
        assert_eq!(app.world().get::<OnMonitor>(window).is_none(), keep);
        app.world_mut().entity_mut(monitor).despawn();
        app.update();
        app.world().get_entity(window).is_ok()
    }

    #[test]
    fn monitor_despawn_keeps_window() {
        // negative control: bevy's linked_spawn takes the window down with the monitor
        assert!(!run(false));
        assert!(run(true));
    }
}
