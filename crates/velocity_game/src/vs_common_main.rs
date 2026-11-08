//! src\VsCommonMain.cpp — the main loop (0x0016DD20). Each Bevy frame stands for one pass of the original inner loop:
//! `VsFrameTimer::Tick` is called until it asks for a render; every `true` runs one game update, here the
//! `GameTick` schedule (GameManager::Update, 0x000BB1C0, at 24 Hz). Render-side systems run in Bevy's `Update`
//! and read the interpolation state from `GameClock`.
//!
//! In-game update order (GameManager::InGameUpdate 0x000B77F0) is exposed as `InGameUpdateSet`, chained in that
//! order inside `GameTick`; put per-tick game systems in the matching set.

use std::time::Instant;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

use crate::core_::vs_frame_timer::{VsFrameTimer, GAME_RATE, MAX_UPDATES_PER_FRAME};

/// one fixed game update (1/24 s).
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameTick;

/// GameManager::InGameUpdate (0x000B77F0) phases, in order.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InGameUpdateSet {
    /// "Update Game Bridge", "Update Input" (GameManager::Update before the mode dispatch)
    Input,
    /// movie, OSD pause / subtitle screens, game OSD, menu manager, texture viewer
    Osd,
    /// osd map, attract mode, "Crash Cam Update"
    CrashCam,
    /// "MonitorSessions" (race session state machine, 0x000BC5F0)
    MonitorSessions,
    /// "UpdateSessions" (0x000BD7A0)
    UpdateSessions,
    /// "g_PostRaceSequence.Update"
    PostRaceSequence,
    /// "VsAIWorld->Update", follow-camera point of interest
    Ai,
    /// "CPhysicsManager->Update", physics objects
    Physics,
    /// debug camera, replay cameras, post race camera, "g_CameraMan.Update"
    Cameras,
    /// sky, barriers, skid marks
    World,
    /// "g_AudioSystem.AudioUpdate"
    Audio,
    /// shader manager, OSD managers, car render, car damage
    Render,
    /// "CCompartmentManager.Update", skid smoke, lens spray, particles, heat haze, fade, start line, zone trails,
    /// vibration
    Effects,
    /// TFT manager, time-of-day, race timeline sequencer
    Timeline,
}

/// the frame timer plus its clock.
#[derive(Resource)]
pub struct GameClock {
    pub timer: VsFrameTimer,
    pub start: Instant,
    /// ticks of `start` (nanoseconds)
    pub manual: Option<u64>,
}

impl Default for GameClock {
    /// VsCommonMain.cpp 0x0016DD20 "Frame Timer": 24 Hz, at most 2 updates per rendered frame.
    fn default() -> Self {
        let mut timer = VsFrameTimer::new(GAME_RATE, 1_000_000_000, 0);
        timer.max_updates = MAX_UPDATES_PER_FRAME;
        GameClock { timer, start: Instant::now(), manual: None }
    }
}

impl GameClock {
    pub fn now(&self) -> u64 {
        // golden mode (VELOCITY_GOLDEN): virtual frame-locked clock; None otherwise
        if let Some(t) = visos_render::golden::clock_ns() {
            return t;
        }
        self.manual.unwrap_or_else(|| self.start.elapsed().as_nanos() as u64)
    }
}

/// run the game updates owed this frame.
pub(crate) fn main_loop(world: &mut World) {
    loop {
        let due = {
            let mut c = world.resource_mut::<GameClock>();
            let now = c.now();
            c.timer.tick(now)
        };
        if !due {
            break;
        }
        world.run_schedule(GameTick);
    }
}

pub(crate) fn build(app: &mut App) {
    app.init_resource::<GameClock>()
        .init_schedule(GameTick)
        .configure_sets(
            GameTick,
            (
                InGameUpdateSet::Input,
                InGameUpdateSet::Osd,
                InGameUpdateSet::CrashCam,
                InGameUpdateSet::MonitorSessions,
                InGameUpdateSet::UpdateSessions,
                InGameUpdateSet::PostRaceSequence,
                InGameUpdateSet::Ai,
                InGameUpdateSet::Physics,
                InGameUpdateSet::Cameras,
                InGameUpdateSet::World,
                InGameUpdateSet::Audio,
                InGameUpdateSet::Render,
                InGameUpdateSet::Effects,
                InGameUpdateSet::Timeline,
            )
                .chain(),
        )
        .add_systems(PreUpdate, main_loop);
    if std::env::var_os("VELOCITY_TICK_TRACE").is_some() {
        add_tick_trace(app);
    }
}

/// dev: `VELOCITY_TICK_TRACE=1` logs a line between every GameTick set and at each main schedule, so a hang shows
/// the last stage that started (the log stops right after it)
fn add_tick_trace(app: &mut App) {
    use InGameUpdateSet::*;
    let sets = [Input, Osd, CrashCam, MonitorSessions, UpdateSessions, PostRaceSequence, Ai, Physics, Cameras, World, Audio, Render, Effects, Timeline];
    for (i, s) in sets.iter().enumerate() {
        let name = format!("{s:?}");
        let sys = move || log::info!("tick trace: {name}");
        if i == 0 {
            app.add_systems(GameTick, sys.before(*s));
        } else {
            app.add_systems(GameTick, sys.after(sets[i - 1]).before(*s));
        }
    }
    app.add_systems(GameTick, (|| log::info!("tick trace: end")).after(Timeline));
    // one thread: the markers bound the hanging set exactly
    app.edit_schedule(GameTick, |s| {
        s.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
    });
    app.add_systems(First, || log::info!("tick trace: First"));
    app.add_systems(PreUpdate, || log::info!("tick trace: PreUpdate"));
    app.add_systems(Update, || log::info!("tick trace: Update"));
    app.add_systems(PostUpdate, || log::info!("tick trace: PostUpdate"));
    app.add_systems(Last, || log::info!("tick trace: Last"));
}
