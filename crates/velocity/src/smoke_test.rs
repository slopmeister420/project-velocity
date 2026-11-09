//! `--smoke-test` (alias `--slice-test`): drives the shipped flow with a scripted pad (visos input layer, port 0) — main menu, Quick
//! Race, an LADown circuit, car select, start — then drives forward, opens the pause menu, quits to the front end
//! and waits for the quick race start screen, saving screenshots to `research/screens/smoke_*.png` (relative to
//! the working directory), then exits.

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use velocity_frontend::FrontendState;
use velocity_game::game_mode::race_session::{ESessionState, RaceSession};
use velocity_game::{EGameMode, GameManager};
use visos::input::input_device::{key, DeviceKeys, NUM_KEYS};
use visos::input::input_manager::set_script;

mod career3;
mod direct;

fn crc(s: &str) -> u32 {
    velocity_frontend::string_manager::string_manager::string_crc(s)
}

#[derive(Resource, Default)]
struct Script {
    t: f32,
    /// time of the last key press
    last: f32,
    /// key held until this time
    hold_until: f32,
    held: Option<usize>,
    /// frames the held key was sent on
    held_frames: u32,
    /// `GameClock.timer.total_updates` when the held key was first sent
    press_updates: i32,
    /// SMOKE_DEBUG_MENU: 0 closed, 1 walking the path, 2 shot taken (BACK next), 3 done
    debug_step: u32,
    /// SMOKE_DEBUG_MENU: index of the next path name
    debug_seg: usize,
    /// SMOKE_DEBUG_MENU: RIGHT presses made on the tweak, and when the menu was closed (SMOKE_DEBUG_MENU_SHOT_CLOSED)
    debug_presses: u32,
    debug_closed_at: Option<f32>,
    shots: Vec<&'static str>,
    race_t: Option<f32>,
    /// `GameClock.timer.total_updates` when `race_t` was set: race shots / key windows count game ticks, not render
    /// frames (a system added to the render graph must not move a golden shot by a tick)
    race_u: i32,
    intro_t: Option<f32>,
    /// current front end screen and when it became current
    screen: (u32, f32),
    done_at: Option<f32>,
    /// pause menu walk after driving: step, time of the last step
    quit: Option<(u32, f32)>,
    /// back in the front end after the quit
    quit_done: bool,
    /// first physics object hit / car spark collision: time seen
    object_hit: Option<f32>,
    spark_hit: Option<f32>,
    /// SMOKE_HIT: the movable object the car steers at
    target: Option<Vec3>,
    /// SMOKE_CAREER: 0 idle, 1 new profile asked (A on an empty profile slot), 2 career item picked on the main
    /// menu, 3 career entered (the hangout starts UT0EVENT1 itself), 5 reload asked after the race, 6 reloaded,
    /// 7 in the second race (the hangout's restart of UT0EVENT1; driven 20 s, then done)
    career: u32,
    /// SMOKE_ATTRACT: attract races back on the main menu so far (two are played)
    attract_runs: u32,
    /// SMOKE_VIA: (time the via point was reached, heading then)
    via_passed: Option<(f32, Vec3)>,
    via_i: usize,
    /// SMOKE_VIA phase ("a|b|..."): advanced at each trailblazer end (CarFlag(false))
    via_phase: usize,
    /// follow driver stuck check: (time of the last check, position then), reversing until
    stuck: Option<(f32, Vec3)>,
    reverse_until: f32,
    /// the profile the front end made (`CareerMessage::Created`)
    profile: Option<String>,
    /// shots asked for this frame, captured by `flush_shots`
    pending: Vec<std::path::PathBuf>,
    /// `total_updates` before this frame's game updates (set by `run`) and whether the race runs
    tick: i32,
    in_race: bool,
    /// in the race the pending shots wait for this `total_updates` (the first update after the one the condition
    /// was seen on): the image is that exact tick, whatever the frame / update phase
    shot_tick: Option<i32>,
    /// SMOKE_CAREER=3 (`career3`)
    c3: career3::C3,
}

/// logs the race's physics object events and spark collisions and screenshots the first of each shortly after
/// (`6a_object_hit`, `6b_sparks`)
fn watch_hits(
    mut commands: Commands,
    mut s: ResMut<Script>,
    mut objects: MessageReader<velocity_physics::plugin::PhysicsObjectMessage>,
    mut sparks: MessageReader<visos_render::race_effects::SparkCollisionMessage>,
) {
    for m in objects.read() {
        log::info!("smoke-test: physics object {:?} at {:.1}", m.0, s.t);
        if s.object_hit.is_none() {
            s.object_hit = Some(s.t);
        }
    }
    for m in sparks.read() {
        let c = &m.0;
        if c.impulse * c.scale > 1.0 {
            log::info!("smoke-test: collision car {} other {} impulse {:.2} scale {} material {} at {:.1}", c.car, c.other, c.impulse, c.scale, c.material, s.t);
            if s.spark_hit.is_none() {
                s.spark_hit = Some(s.t);
            }
        }
    }
    let now = s.t;
    if s.object_hit.is_some_and(|t| now - t > 0.3) {
        shot(&mut commands, &mut s, "6a_object_hit");
    }
    if s.spark_hit.is_some_and(|t| now - t > 0.1) {
        shot(&mut commands, &mut s, "6b_sparks");
    }
}

/// the trailblazer's events (not the per-update ones), its OSD message, the time stretch; a screenshot 1.5 s into
/// the challenge (`2d_trailblazer`) and at its end (`2e_trailblazer_end`)
fn watch_trail(
    mut commands: Commands,
    mut s: ResMut<Script>,
    mut tb: MessageReader<velocity_game::game_mode::trail_blazer::TrailBlazerMessage>,
    gm: Res<GameManager>,
    clock: Res<velocity_game::vs_common_main::GameClock>,
    mut last: Local<(u32, Option<f32>, Option<f32>)>,
    mut stretch: Local<f32>,
    mut triggers: MessageReader<velocity_game::triggers::trigger_update::TriggerMessage>,
    mut objects: MessageReader<velocity_game::timeline_manager::objects::timeline_object::TimelineObjectMessage>,
) {
    use velocity_game::game_mode::trail_blazer::TrailEvent;
    use velocity_game::timeline_manager::objects::timeline_object::TimelineObjectMessage;
    use velocity_game::triggers::trigger_update::TriggerMessage;
    for m in triggers.read() {
        if matches!(m, TriggerMessage::Timeline { .. } | TriggerMessage::BigParticles | TriggerMessage::Trailblazer { .. }) {
            log::info!("smoke-test: trigger {m:?} at {:.1}", s.t);
        }
    }
    for m in objects.read() {
        if matches!(m, TimelineObjectMessage::TrailBlazerFirewall(_) | TimelineObjectMessage::InstanceMove { .. }) {
            log::info!("smoke-test: timeline {m:?} at {:.1}", s.t);
        }
    }
    for m in tb.read() {
        match m.event {
            TrailEvent::RenderParams { .. } | TrailEvent::TimeStretch(_) | TrailEvent::Intensity(_) | TrailEvent::Vibrate(_) => {}
            TrailEvent::DangerZone(Some(_)) | TrailEvent::AudioFadeIn(_) => {}
            TrailEvent::PhysicsGlobals { zone_sensitivities: None, .. } => {}
            e => {
                log::info!("smoke-test: trailblazer {} {e:?} at {:.1} (stretch {})", m.vehicle, s.t, clock.timer.time_stretch);
                if matches!(e, TrailEvent::CarFlag(true)) {
                    last.1 = Some(s.t);
                }
                if matches!(e, TrailEvent::CarFlag(false)) {
                    last.2 = Some(s.t);
                    // the next SMOKE_VIA phase
                    s.via_phase += 1;
                    s.via_i = 0;
                    s.via_passed = None;
                }
            }
        }
    }
    if clock.timer.time_stretch != *stretch {
        *stretch = clock.timer.time_stretch;
        log::info!("smoke-test: time stretch {} at {:.2}", *stretch, s.t);
    }
    if gm.trail_blazer_message != last.0 {
        last.0 = gm.trail_blazer_message;
        log::info!("smoke-test: osd trailblazer message {:#x} at {:.1}", last.0, s.t);
    }
    if last.1.is_some_and(|t| s.t - t > 1.5) {
        shot(&mut commands, &mut s, "2d_trailblazer");
    }
    if last.2.is_some_and(|t| s.t - t > 0.5) {
        shot(&mut commands, &mut s, "2e_trailblazer_end");
    }
}

/// SMOKE_CAMS: log view 0's camera mode / car view / look back, the bridge cut and `HeadlightView` on change, and a
/// screenshot 1.5 s into each car view (`3a_view0` .. `3d_view3`, `3e_look_back`)
fn watch_cams(
    mut commands: Commands,
    mut s: ResMut<Script>,
    cams: Res<velocity_game::camera::game_cameras::GameCameras>,
    head: Res<visos_render::track_fx::HeadlightView>,
    bridge: Res<velocity_game::game_bridge::game_bridge::GameBridge>,
    clock: Res<velocity_game::vs_common_main::GameClock>,
    mut last: Local<Option<(i32, u32, bool, Option<u64>)>>,
    mut since: Local<i32>,
) {
    if std::env::var_os("SMOKE_CAMS").is_none() {
        return;
    }
    let Some(pc) = cams.manager.players.first() else { return };
    let rear = pc.follow.as_ref().is_some_and(|f| f.rear || f.unknown_585);
    let now = (head.camera_mode, pc.view, rear, head.inside);
    if *last != Some(now) {
        let f = pc.follow.as_ref();
        log::info!(
            "smoke-test: camera mode {} view {} chase {:?} look back {} headlight inside {:?} bridge cut {} at {:.1}",
            now.0, now.1, f.map(|f| f.view.chase), now.2, now.3, bridge.cut, s.t
        );
        *last = Some(now);
        *since = clock.timer.total_updates;
    }
    // 1.5 s of game updates (not render frames) after the view change
    if head.camera_mode == 1 && (clock.timer.total_updates - *since) as f32 * clock.timer.dt > 1.5 {
        const NAMES: [&str; 4] = ["3a_view0", "3b_view1", "3c_view2", "3d_view3"];
        let name = if rear { "3e_look_back" } else { NAMES[(pc.view as usize).min(3)] };
        shot(&mut commands, &mut s, name);
    }
}

/// SMOKE_REVIEW=<1|2> ("Enable Cameras For Review", `dword_749110`, set 2 s after Go: 1 = the ai drives the player
/// and the replay cameras show it) and SMOKE_ATTRACT=1 (the attract mode; `SMOKE_ATTRACT_IDLE=<s>` shortens the
/// 300 s main menu idle time, default 8): log the replay camera shots, the attract state, the race end; shots
/// `7a_replay_wheel` / `7b_replay_orbit` / `7c_replay_fixed` 1 s into the first shot of each type, `7d_finished`,
/// `7e_post_race` (8 s after the finish, then done) and `8a_attract_race`, `8b_attract_fade`
#[allow(clippy::too_many_arguments)]
fn watch_replay(
    mut commands: Commands,
    mut s: ResMut<Script>,
    mut cams: ResMut<velocity_game::camera::game_cameras::GameCameras>,
    mut attract: ResMut<velocity_game::font_helper::CAttractMode>,
    session: Res<RaceSession>,
    mode: Res<State<EGameMode>>,
    dof: Option<Res<visos_render::danger_zone_render::DepthOfFieldInput>>,
    mut last: Local<(Option<(u32, u32, bool, Option<usize>)>, f32, Option<(bool, bool)>, Option<f32>, Option<ESessionState>, Option<f32>)>,
    (mut review_at, mut cycled): (Local<Option<f32>>, Local<Option<velocity_game::replays::cameras::replay_camera_manager::EReplayCamera>>),
) {
    use velocity_game::replays::cameras::replay_camera_manager::EReplayCamera;
    let review: u32 = std::env::var("SMOKE_REVIEW").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let attract_on = std::env::var_os("SMOKE_ATTRACT").is_some();
    if review == 0 && !attract_on {
        return;
    }
    // the main menu idle time (`dword_685698`) cut for the run
    if attract_on && !s.shots.contains(&"_attract_idle") {
        s.shots.push("_attract_idle");
        attract.idle_time = std::env::var("SMOKE_ATTRACT_IDLE").ok().and_then(|v| v.parse().ok()).unwrap_or(8.0);
        log::info!("smoke-test: attract idle time {} s", attract.idle_time);
    }
    let a = (attract.playing, attract.quitting);
    if last.2 != Some(a) {
        log::info!("smoke-test: attract playing {} quitting {} timer {:.1} at {:.1}", a.0, a.1, attract.timer, s.t);
        last.2 = Some(a);
        if a.1 {
            last.3 = Some(s.t);
        }
    }
    if last.3.is_some_and(|t| s.t - t > 0.25) {
        shot(&mut commands, &mut s, "8b_attract_fade");
    }
    // 8 s after the finish (the race is in mode 2 by then: the post race menus are not ported)
    if let Some(fin) = last.5 {
        // the end camera takes over ~1 s after the finish (PostRaceSequence state 2); results restart at ~2.5 s
        if s.t - fin > 2.0 {
            shot(&mut commands, &mut s, "7f_post_race_cam");
        }
        if s.t - fin > 8.0 {
            shot(&mut commands, &mut s, "7e_post_race");
            if s.done_at.is_none() {
                let t = s.t;
                s.done_at = Some(t);
            }
        }
    }
    if *mode.get() != EGameMode::InGame {
        return;
    }
    if last.4 != Some(session.state) {
        log::info!("smoke-test: session {:?} race time {:.1} at {:.1}", session.state, session.race_time, s.t);
        last.4 = Some(session.state);
    }
    let racing_t = s.race_t.map(|t| s.t - t);
    if review != 0 && racing_t.is_some_and(|d| d > 2.0) && cams.replay.review_cameras != review && !s.shots.contains(&"_review_on") {
        s.shots.push("_review_on");
        cams.replay.review_cameras = review;
        *review_at = Some(s.t);
        log::info!("smoke-test: review cameras {review} at {:.1}", s.t);
    }
    // SMOKE_REPLAY_CYCLE=1 (debug, not an original path: every shipped CAMERA_SWITCH trigger has radius² -1, so
    // 0x00166700 never runs and the wheel camera stays): 6 s into the review the fixed cameras (0x00164920 picks
    // the nearest seeing one), 12 s the orbit camera (0x00166690's else branch), 18 s the wheel camera again
    if let Some(at) = *review_at {
        if std::env::var_os("SMOKE_REPLAY_CYCLE").is_some() {
            let d = s.t - at;
            let want = if d < 6.0 {
                None
            } else if d < 12.0 {
                Some(EReplayCamera::Fixed)
            } else if d < 18.0 {
                Some(EReplayCamera::Orbit)
            } else {
                Some(EReplayCamera::Wheel)
            };
            let r = &mut cams.replay;
            if let Some(c) = want.filter(|&c| c != r.camera && *cycled != Some(c)) {
                *cycled = Some(c);
                if c == EReplayCamera::Orbit {
                    let v = r.vehicle;
                    r.orbit.reset_for(v);
                }
                if c == EReplayCamera::Fixed {
                    r.fixed.rechoose = true;
                }
                r.camera = c;
            }
        }
    }
    let r = &cams.replay;
    let now = (r.camera as u32, r.vehicle, r.updating, r.fixed.record);
    if last.0 != Some(now) {
        log::info!(
            "smoke-test: replay camera {:?} car {} updating {} fixed {:?} dof replay {:?} at {:.1}",
            r.camera, r.vehicle, r.updating, r.fixed.record, dof.as_ref().map(|d| (d.replay, d.replay_camera_type)), s.t
        );
        last.0 = Some(now);
        last.1 = s.t;
    }
    if r.updating && s.t - last.1 > 1.0 {
        let name = match r.camera {
            EReplayCamera::Wheel => Some("7a_replay_wheel"),
            EReplayCamera::Orbit => Some("7b_replay_orbit"),
            EReplayCamera::Fixed => Some("7c_replay_fixed"),
            EReplayCamera::None => None,
        };
        if let Some(n) = name {
            shot(&mut commands, &mut s, n);
        }
    }
    if attract.playing && racing_t.is_some_and(|d| d > 4.0) {
        let name = if s.attract_runs == 0 { "8a_attract_race" } else { "8d_attract_race2" };
        if !s.shots.contains(&name) {
            log::info!("smoke-test: attract race {} playing at {:.1}", s.attract_runs + 1, s.t);
        }
        shot(&mut commands, &mut s, name);
    }
    if matches!(session.state, ESessionState::Finished | ESessionState::PostRace) {
        shot(&mut commands, &mut s, "7d_finished");
        let now = s.t;
        last.5.get_or_insert(now);
    }
}

fn shot(_commands: &mut Commands, s: &mut Script, name: &'static str) {
    if s.shots.contains(&name) {
        return;
    }
    s.shots.push(name);
    let dir = std::path::Path::new("research/screens");
    let _ = std::fs::create_dir_all(dir);
    // SMOKE_PREFIX renames the files (default "smoke") so parallel runs do not overwrite each other
    let prefix = std::env::var("SMOKE_PREFIX").unwrap_or_else(|_| "smoke".into());
    let path = dir.join(format!("{prefix}_{name}.png"));
    log::info!("smoke-test: screenshot {}", path.display());
    s.pending.push(path);
    if s.in_race {
        let t = s.tick + 1;
        s.shot_tick.get_or_insert(t);
    }
}

/// one primary-window capture per frame: Bevy's `extract_screenshots` despawns a second `Screenshot` of the same
/// target in a frame ("Duplicate render target"), so with `SMOKE_RECORD` the recorder's per-frame capture used to
/// swallow the smoke shots taken by Update systems (all the `3*_view*` shots of the recorded cams run). The pending
/// shots ride on a capture already spawned this frame, or get their own.
fn flush_shots(
    mut commands: Commands,
    mut s: ResMut<Script>,
    fresh: Query<Entity, (With<Screenshot>, Without<bevy::render::view::screenshot::Capturing>)>,
    clock: Res<velocity_game::vs_common_main::GameClock>,
) {
    if s.pending.is_empty() {
        return;
    }
    if let Some(t) = s.shot_tick {
        // (a counter reset by InGameInit also releases it)
        if clock.timer.total_updates < t && clock.timer.total_updates >= s.tick {
            return;
        }
        log::info!("smoke-test: shot at update {} lerp {:.3}", clock.timer.total_updates, clock.timer.lerp);
        s.shot_tick = None;
    }
    let mut e = match fresh.iter().next() {
        Some(e) => commands.entity(e),
        None => commands.spawn(Screenshot::primary_window()),
    };
    for path in s.pending.drain(..) {
        e.observe(save_to_disk(path));
    }
}

/// one SMOKE_DEBUG_MENU step (see the call in `run`)
fn debug_menu_walk(s: &mut Script, dm: Option<&velocity_game::game_menu_manager::game_menu_manager::CDebugMenuManager>) {
    let path = std::env::var("SMOKE_DEBUG_MENU_PATH").unwrap_or_else(|_| "System.System Update Rate".into());
    let names: Vec<&str> = std::iter::once("Tweaks").chain(path.split('.')).collect();
    let shot = |s: &mut Script| {
        let name = std::env::var("SMOKE_DEBUG_MENU_SHOT").unwrap_or_else(|_| "debug_menu.png".into());
        let p = std::path::Path::new("research/screens").join(name);
        log::info!("smoke-test: screenshot {}", p.display());
        s.last = s.t;
        s.pending.push(p);
        s.debug_step = 2;
    };
    match s.debug_step {
        0 => {
            log::info!("smoke-test: debug menu open at {:.1}", s.t);
            press(s, key::BACK);
            s.debug_step = 1;
            s.debug_seg = 0;
        }
        1 => {
            // SMOKE_DEBUG_MENU_SHOT_CLOSED: the shot 1.5 s after BACK closed the menu (the game runs again)
            if let Some(t0) = s.debug_closed_at {
                if s.t - t0 > 1.5 {
                    shot(s);
                    s.debug_step = 3;
                }
                return;
            }
            let Some(menu) = dm.and_then(|d| d.stack.last()) else { return };
            if s.debug_seg >= names.len() {
                // the last system is open, or the tweak was stepped
                if std::env::var_os("SMOKE_DEBUG_MENU_SHOT_CLOSED").is_some() {
                    log::info!("smoke-test: debug menu closed at {:.1}", s.t);
                    press(s, key::BACK);
                    s.debug_closed_at = Some(s.t);
                } else {
                    shot(s);
                }
                return;
            }
            let want = names[s.debug_seg];
            let Some(idx) = menu.items.iter().position(|i| i.text == want) else {
                let rows: Vec<&str> = menu.items.iter().map(|i| i.text.as_str()).collect();
                log::error!("smoke-test: debug menu: no row {want:?} in {:?}: {rows:?}", menu.name);
                shot(s);
                return;
            };
            if menu.selected < idx {
                press(s, key::DOWN);
            } else if menu.selected > idx {
                press(s, key::UP);
            } else if menu.items[idx].tweak.is_some() {
                // SMOKE_DEBUG_MENU_STEPS=<n> RIGHT presses (default 1)
                let steps: u32 = std::env::var("SMOKE_DEBUG_MENU_STEPS").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
                log::info!("smoke-test: debug menu RIGHT on {want:?}");
                press(s, key::RIGHT);
                s.debug_presses += 1;
                if s.debug_presses >= steps {
                    s.debug_seg = names.len();
                }
            } else {
                log::info!("smoke-test: debug menu A on {want:?}");
                press(s, key::A);
                s.debug_seg += 1;
            }
        }
        _ => {
            press(s, key::BACK);
            s.debug_step = 3;
        }
    }
}

fn press(s: &mut Script, k: usize) {
    log::debug!("smoke-test: press {} at {:.1}", visos::input::input_device::XBOXPAD_KEYS[k], s.t);
    s.held = Some(k);
    s.held_frames = 0;
    s.hold_until = s.t + 0.12;
    s.last = s.t;
}

#[allow(clippy::too_many_arguments)]
fn run(
    mut commands: Commands,
    time: Res<Time>,
    mut s: ResMut<Script>,
    fe: Res<FrontendState>,
    mode: Res<State<EGameMode>>,
    session: Res<RaceSession>,
    gm: Res<GameManager>,
    objects: Option<Res<velocity_physics::plugin::PhysicsObjects>>,
    cars: Query<(&velocity_game::game_mode::race_cars::RaceCar, &velocity_physics::plugin::PhysicsCarState)>,
    mut exit: MessageWriter<AppExit>,
    mut flow: MessageWriter<velocity_game::GameFlowRequest>,
    mut career_msgs: MessageReader<velocity_game::game_mode::career::CareerMessage>,
    hud: Res<velocity_game::game_mode::race_cars::RaceStatus>,
    mut ai: Option<ResMut<velocity_ai::plugin::AiState>>,
    clock: Res<velocity_game::vs_common_main::GameClock>,
    (choice, dm): (
        Option<Res<velocity_game::game_mode::race_cars::PlayerCarChoice>>,
        Option<Res<velocity_game::game_menu_manager::game_menu_manager::CDebugMenuManager>>,
    ),
) {
    use velocity_game::game_mode::career::CareerMessage;
    s.tick = clock.timer.total_updates;
    s.in_race = *mode.get() == EGameMode::InGame;
    let c3 = career3::enabled();
    if std::env::var_os("SMOKE_AI_DRIVE").is_some() || c3 {
        if let Some(w) = ai.as_mut().and_then(|a| a.world.as_mut()) {
            w.world.globals.player_ai_control = true;
        }
    }
    // SMOKE_FOLLOW_LINE: steer the player at its race vehicle's raceline point 25 m ahead (VsAIRaceVehicle
    // point_ahead), e.g. to reach LADown Race02's trailblazer trigger
    let follow = std::env::var_os("SMOKE_FOLLOW_LINE").is_some().then(|| {
        let w = ai.as_mut().and_then(|a| a.world.as_mut())?;
        let car = w.race.players.first()?;
        car.race_section?;
        // a running trailblazer: its spline's first point 20 m beyond the nearest one
        if let Some(sp) = car.trail_blazer.as_ref().and_then(|t| t.spline) {
            let p = Vec3::new(car.base.position.x, car.base.position.y, car.base.position.z);
            if let Some(line) = w.world.find_trail_blazer_line_mut(sp) {
                let pts: Vec<Vec3> = line.points.iter().map(|q| Vec3::new(q.x, q.y, q.z)).collect();                let near = (0..pts.len()).min_by(|&a, &b| pts[a].distance(p).total_cmp(&pts[b].distance(p)))?;
                let t = pts[near..].iter().find(|q| q.distance(p) > 20.0).or(pts.last())?;
                return Some(*t);
            }
        }
        // LACoast's duplicated race sections (1 <-> 3) make the walk endless (original); no target then
        let sec = car.race_section?;
        let mut logged = true;
        if !crate::stream_raceline::point_ahead_ends(&w.world, sec, car.dist_along + 25.0, &mut logged) {
            return None;
        }
        let q = car.point_ahead(&mut w.world, 25.0);
        Some(Vec3::new(q.x, q.y, q.z))
    }).flatten();
    use velocity_game::register_file_types::save_file_data;
    let career_mode = std::env::var_os("SMOKE_CAREER").is_some();
    let attract_mode = std::env::var_os("SMOKE_ATTRACT").is_some();
    for m in career_msgs.read() {
        log::info!("smoke-test: career {m:?}");
        if c3 {
            career3::on_message(&mut s, m, &gm, &mut flow);
        }
        if let CareerMessage::Created { name } = m {
            s.profile = Some(name.clone());
        }
        // back from the race: the hangout restarts UT0EVENT1 on its first update while `completed_any` is false
        // (0x0021C430), so it cannot be left through its quit dialog; check the autosave and reload it directly
        if let CareerMessage::ReturnedFromRace { .. } = m {
            if s.career == 3 {
                let name = s.profile.clone().unwrap_or_else(|| gm.vars.profile_name.clone());
                let p = save_file_data::primary_path(&name);
                log::info!("smoke-test: career save {} exists={}", p.display(), p.is_file());
                flow.write(velocity_game::GameFlowRequest::LoadProfile { name });
                s.career = 5;
            }
        }
        if let CareerMessage::Loaded { ok, .. } = m {
            if s.career == 5 {
                log::info!(
                    "smoke-test: profile reloaded ok={ok}: cash {} prestige {} garage {:08x?} completed events {}",
                    gm.vars.cash,
                    gm.vars.prestige,
                    gm.vars.garage,
                    gm.event_db.events.iter().filter(|e| e.flags & 2 != 0).count()
                );
                // the hangout restarts UT0EVENT1 now: the second race load (car / track teardown of the first race)
                s.career = 6;
            }
        }
    }
    s.t += time.delta_secs();
    let mut keys: DeviceKeys = [0.0; NUM_KEYS];
    if let Some(k) = s.held {
        // held 0.12 s and at least 3 frames: after a long frame (race load, wet effects) a time-only hold could
        // end before any frame saw the key (seen: London Circuit Race06 wet, the quit confirm A was lost)
        // in the race the game menu runs per game update (0x000A9DF0 from InGameUpdate, 0.2 s auto repeat): hold
        // the key until a frame in which at least one update sampled it (at most 2), never by time
        let updates = clock.timer.total_updates;
        if s.held_frames == 0 {
            s.press_updates = updates;
        }
        let in_race = *mode.get() == EGameMode::InGame;
        let hold = if in_race { s.held_frames == 0 || updates == s.press_updates } else { s.t < s.hold_until || s.held_frames < 3 };
        if hold {
            keys[k] = 1.0;
            s.held_frames += 1;
        } else {
            s.held = None;
        }
    }
    let ready = s.held.is_none() && s.t - s.last > 0.6;
    // coverage sweep: a race that ends before the drive does (drag, short sprint) is a pass too
    let sweep = std::env::var_os("SMOKE_RACE_PATH").is_some();
    let finished = *mode.get() == EGameMode::PostRace
        || matches!(session.state, ESessionState::Finished | ESessionState::PostRace);
    if sweep && finished && s.race_t.is_some() && s.done_at.is_none() {
        log::info!("smoke-test: race finished (session {:?}) at {:.1}", session.state, s.t);
        let now = s.t;
        s.done_at = Some(now + 2.0);
    }
    let post_race = c3 && *mode.get() == EGameMode::PostRace;
    if post_race {
        career3::post_race(&mut commands, &mut s, ready);
    }
    // the results OSD (0x000F35C0) waits for select in session PostRace: step 1, table page 2 -> 3 -> done
    if *mode.get() == EGameMode::InGame && session.state == ESessionState::PostRace && ready && s.t - s.last > 1.0 {
        log::info!("smoke-test: results A at {:.1}", s.t);
        press(&mut s, key::A);
    }
    if c3 && *mode.get() == EGameMode::InGame {
        career3::in_race(&mut commands, &mut s, choice.as_deref());
    }
    if *mode.get() == EGameMode::InGame {
        let now = s.t;
        if s.career == 6 {
            log::info!("smoke-test: career second race at {now:.1}");
            s.career = 7;
            s.race_t = None;
            s.intro_t = None;
            s.quit = None;
        }
        // intro_t holds game time (ticks * dt), not render time
        let gnow = clock.timer.total_updates as f32 * clock.timer.dt;
        if session.state == ESessionState::Countdown && *s.intro_t.get_or_insert(gnow) + 3.0 < gnow {
            shot(&mut commands, &mut s, "0d_flag_girl");
        }
        if session.state == ESessionState::Racing || session.state == ESessionState::Go {
            let now = s.t;
            if s.race_t.is_none() {
                s.race_u = clock.timer.total_updates;
                log::info!("smoke-test: race start at update {} frame {}", s.race_u, clock.timer.frames);
            }
            s.race_t.get_or_insert(now);
            let dt = (clock.timer.total_updates - s.race_u) as f32 * clock.timer.dt;
            if dt > 0.5 {
                shot(&mut commands, &mut s, "1_start");
            }
            // the opponents pull away from the grid
            if dt > 2.0 {
                if !s.shots.contains(&"1b_opponents") {
                    log::info!(
                        "smoke-test: hud position {}/{} lap {}/{} racing {}",
                        hud.position, hud.race_length, hud.lap, hud.laps, hud.racing
                    );
                }
                shot(&mut commands, &mut s, "1b_opponents");
            }
            // SMOKE_DRIVE=<secs> drives longer than the default 12 s; SMOKE_AI_DRIVE lets the ai drive the player car
            // ("PlayerAIControl", VsAIGlobals +0x000, read at the race vehicle's reset)
            let mut drive: f32 = std::env::var("SMOKE_DRIVE").ok().and_then(|v| v.parse().ok()).unwrap_or(12.0);
            // a debug menu walk (many DOWN presses) runs to its end before the quit walk
            if std::env::var_os("SMOKE_DEBUG_MENU").is_some() && s.debug_step < 3 {
                drive = drive.max(dt + 1.0);
            }
            // career: the second race is driven 20 s (throttle to 22 s) and the run ends in it
            if s.career == 7 {
                drive = drive.max(22.0);
                if dt > 20.0 && s.done_at.is_none() {
                    shot(&mut commands, &mut s, "6_career_second_race");
                    log::info!("smoke-test: career second race driven {dt:.1} s");
                    let now = s.t;
                    s.done_at = Some(now);
                }
            }
            // full throttle with a little steer later, then a brake (brake lights) at 9.5..10.7 s
            if dt > 9.5 && dt < 10.7 && follow.is_none() {
                keys[key::LTRIGGER] = 1.0;
            } else if dt > 1.0 && dt < drive {
                keys[key::RTRIGGER] = 1.0;
            }
            if dt > 10.4 {
                shot(&mut commands, &mut s, "2b_braking");
            }
            // SMOKE_ZONE: hold X (CarZoneMode, Config1.ini) 3..9 s; shot once the zone FX has built up
            if std::env::var_os("SMOKE_ZONE").is_some() && dt > 3.0 && dt < 9.0 {
                keys[key::X] = 1.0;
                if dt > 6.5 {
                    shot(&mut commands, &mut s, "2c_zone");
                }
            }
            // SMOKE_NOS: hold B (CarNitrous, Config1.ini) 3..6 s; shot at 5 s (fury fov / blur / ribbons)
            if std::env::var_os("SMOKE_NOS").is_some() && dt > 3.0 && dt < 6.0 {
                keys[key::B] = 1.0;
                if dt > 5.0 {
                    shot(&mut commands, &mut s, "2f_nos");
                }
            }
            // SMOKE_DEBUG_MENU: BACK (Online) opens the debug menu, then the path "Tweaks." + SMOKE_DEBUG_MENU_PATH
            // (default "System.System Update Rate"): each name is found among the current menu's rows, DOWN / UP
            // until the cursor is on it, then A (a system) or RIGHT (a tweak, the last name); shot
            // research/screens/<SMOKE_DEBUG_MENU_SHOT> (default debug_menu.png), BACK closes it (pause toggle)
            if std::env::var_os("SMOKE_DEBUG_MENU").is_some() && dt > 3.0 && ready && s.debug_step < 3 {
                debug_menu_walk(&mut s, dm.as_deref());
            }
            // SMOKE_CAMS: BLACK (CameraChange) every 4 s from 3 s (views 1, 2, 3, 0, ...), WHITE (CarRearView) held
            // 13..14.5 s and 17..18.5 s (looking back on the in-car view, then on the chase view)
            if std::env::var_os("SMOKE_CAMS").is_some() {
                if dt > 3.0 && (dt - 3.0).rem_euclid(4.0) < 0.15 {
                    keys[key::BLACK] = 1.0;
                }
                if (13.0..14.5).contains(&dt) || (17.0..18.5).contains(&dt) {
                    keys[key::WHITE] = 1.0;
                }
            }
            let player = cars.iter().find(|(c, _)| c.slot == 0).map(|(_, st)| (st.position, st.rotation));
            // SMOKE_MAX_SPEED=<m/s>: brake instead of throttle above that speed (so the follow driver makes corners)
            let max_speed: Option<f32> = std::env::var("SMOKE_MAX_SPEED").ok().and_then(|v| v.parse().ok());
            if let (Some(max), Some(st)) = (max_speed, cars.iter().find(|(c, _)| c.slot == 0).map(|(_, st)| st)) {
                if keys[key::RTRIGGER] > 0.0 && st.velocity.length() > max {
                    keys[key::RTRIGGER] = 0.0;
                    keys[key::LTRIGGER] = 1.0;
                }
            }
            if (dt * 0.5) as i32 != ((dt - time.delta_secs()) * 0.5) as i32 {
                log::info!("smoke-test: {dt:.0} s player at {:?} line {follow:?}", player.map(|p| p.0));            }
            if std::env::var_os("SMOKE_HIT").is_some() {
                // debug: steer at the nearest visible movable object ahead (car forward = +X)
                if let (Some((pos, rot)), Some(objs)) = (player, objects.as_deref()) {
                    if s.target.is_none() && dt > 1.0 {
                        let fwd = rot * Vec3::X;
                        s.target = objs
                            .0
                            .iter()
                            .filter(|o| o.list_type == 1 && o.collides && o.flags & 2 != 0 && o.flags & 8 == 0)
                            .map(|o| Vec3::new(o.matrix[12], o.matrix[13], o.matrix[14]))
                            .filter(|p| {
                                let d = *p - pos;
                                let l = d.length();
                                (15.0..200.0).contains(&l) && d.normalize().dot(fwd) > 0.97
                            })
                            .min_by(|a, b| a.distance(pos).total_cmp(&b.distance(pos)));
                        log::info!("smoke-test: car at {pos:?}, target object {:?}", s.target);
                    }
                    if let Some(t) = s.target {
                        let d = (rot.inverse() * (t - pos)).normalize_or_zero();
                        let ang = d.z.atan2(d.x);
                        keys[key::STICK_LX] = (ang * 10.0).clamp(-1.0, 1.0);
                        if (dt * 4.0) as i32 != ((dt - time.delta_secs()) * 4.0) as i32 {
                            let awake: Vec<_> = objs.0.iter().filter(|o| o.awake).map(|o| (o.name, o.flags)).collect();
                            log::info!("smoke-test: target {:.1} m (local {d:?}), awake objects {awake:x?}", t.distance(pos));
                        }
                    }
                }
            } else if let (Some(t), Some((pos, rot))) = (follow, player) {
                // SMOKE_VIA=x,z: within 120 m, aim at that point (e.g. LADown Race02's trailblazer trigger 349.6,-71.1,
                // which sits off the raceline), then hold the heading 1 s before taking the line (or the running
                // trailblazer's spline) again
                // (several points: "x,z;x,z", taken in turn; "...|..." = phases, the next one armed at each
                // trailblazer end)
                let vias: Vec<Vec3> = std::env::var("SMOKE_VIA")
                    .unwrap_or_default()
                    .split('|')
                    .nth(s.via_phase)
                    .unwrap_or_default()
                    .split(';')
                    .filter_map(|v| {
                        let (a, b) = v.split_once(',')?;
                        Some(Vec3::new(a.trim().parse().ok()?, pos.y, b.trim().parse().ok()?))
                    })
                    .collect();
                let via = vias.get(s.via_i).copied();
                let t = match (via, s.via_passed) {
                    (Some(v), None) if v.distance(pos) < 120.0 => {
                        let behind = (v - pos).with_y(0.0).dot(rot * Vec3::X) < 0.0;
                        if v.distance(pos) < 1.0 || (behind && v.distance(pos) < 15.0) {
                            log::info!("smoke-test: via point {} reached at {:.1} ({:.1} m off)", s.via_i, s.t, v.distance(pos));
                            s.via_i += 1;
                            if s.via_i == vias.len() {
                                s.via_passed = Some((s.t, (rot * Vec3::X).with_y(0.0).normalize_or_zero()));
                            }
                        }
                        v
                    }
                    (Some(_), Some((at, h))) if s.t - at < 1.0 => pos + h * 40.0,
                    _ => t,
                };
                // SMOKE_LINE_OFFSET=<m>: aim beside the line (sideways in the xz plane, + = one side, - = the other)
                let off: f32 = std::env::var("SMOKE_LINE_OFFSET").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let fl = (t - pos).with_y(0.0).normalize_or_zero();
                let t = t + Vec3::new(-fl.z, 0.0, fl.x) * off;
                let d = (rot.inverse() * (t - pos)).normalize_or_zero();
                let ang = d.z.atan2(d.x);
                keys[key::STICK_LX] = (ang * 3.0).clamp(-1.0, 1.0);
                // stuck against a wall (moved < 1.5 m in 2 s with the throttle on): reverse 2 s, steering the other way
                let now = s.t;
                match s.stuck {
                    Some((at, p)) if now - at > 2.0 => {
                        if dt > 5.0 && dt < drive && now > s.reverse_until && p.distance(pos) < 1.5 {
                            let near: Vec<_> = cars
                                .iter()
                                .filter(|(c, st)| c.slot != 0 && st.position.distance(pos) < 12.0)
                                .map(|(c, st)| (c.slot, c.kind, st.position))
                                .collect();
                            log::info!("smoke-test: stuck at {pos:?}, reversing (cars near {near:?})");
                            s.reverse_until = now + 2.0;
                        }
                        s.stuck = Some((now, pos));
                    }
                    None => s.stuck = Some((now, pos)),
                    _ => {}
                }
                if now < s.reverse_until {
                    keys[key::RTRIGGER] = 0.0;
                    keys[key::LTRIGGER] = 1.0;
                    keys[key::STICK_LX] = -keys[key::STICK_LX].signum();
                }
            } else if dt > 8.0 && dt < 9.0 {
                keys[key::STICK_LX] = 0.4;
            }
            // SMOKE_LOOK_POLICE: look back (WHITE, CarRearView) while an active police car is within 80 m behind
            if std::env::var_os("SMOKE_LOOK_POLICE").is_some() {
                use velocity_game::game_mode::race_cars::CarKind;
                if let Some((pos, rot)) = player {
                    let behind = cars.iter().any(|(c, st)| {
                        c.kind == CarKind::Police && c.active && st.position.distance(pos) < 80.0 && (st.position - pos).dot(rot * Vec3::X) < 0.0
                    });
                    if behind {
                        keys[key::WHITE] = 1.0;
                        shot(&mut commands, &mut s, "9b_police_behind");
                    }
                }
            }
            // SMOKE_POLICE: steer at the nearest active police car (a hit or a pass over 30 m/s within 20 m alerts
            // it, VsPoliceManager 0x00039D80 / 0x00039E70, and the chase turns its top lights on)
            if std::env::var_os("SMOKE_POLICE").is_some() && dt > 1.0 {
                use velocity_game::game_mode::race_cars::CarKind;
                if let Some((pos, rot)) = player {
                    let cop = cars
                        .iter()
                        .filter(|(c, _)| c.kind == CarKind::Police && c.active)
                        .map(|(_, st)| st.position)
                        .min_by(|a, b| a.distance(pos).total_cmp(&b.distance(pos)));
                    // police cars join the traffic ~200 m out every 30 s: follow the line (SMOKE_FOLLOW_LINE) until
                    // one is within 120 m
                    if let Some(t) = cop.filter(|t| t.distance(pos) < 120.0 || follow.is_none()) {
                        let d = (rot.inverse() * (t - pos)).normalize_or_zero();
                        keys[key::STICK_LX] = (d.z.atan2(d.x) * 3.0).clamp(-1.0, 1.0);
                        if (dt * 0.5) as i32 != ((dt - time.delta_secs()) * 0.5) as i32 {
                            log::info!("smoke-test: police car {:.1} m away", t.distance(pos));
                        }
                    }
                }
            }
            if dt > 6.0 {
                shot(&mut commands, &mut s, "2_driving");
            }
            if dt > drive && s.career != 7 && !c3 {
                shot(&mut commands, &mut s, "3_driving");
                if s.quit.is_none() {
                    let now = s.t;
                    log::info!("smoke-test: drove {dt:.1} s, quitting at {now:.1}");
                    s.quit = Some((0, now));
                }
            }
        }
        // pause (the game's pause button), then Quit to front end (5 down) and confirm Yes
        if let Some((step, at)) = s.quit {
            let now = s.t;
            if ready && now - at > 0.8 {
                match step {
                    0 => {
                        press(&mut s, key::START);
                        s.quit = Some((1, now));
                    }
                    // START not taken within 3 s (seen in the sweep): press it again
                    1 if !gm.paused && now - at > 3.0 => {
                        log::warn!("smoke-test: not paused 3 s after START (session {:?}), pressing it again at {now:.1}", session.state);
                        s.quit = Some((0, now));
                    }
                    1 if gm.paused => {
                        shot(&mut commands, &mut s, "4a_pause_menu");
                        s.quit = Some((2, now));
                    }
                    2..=6 => {
                        press(&mut s, key::DOWN);
                        s.quit = Some((step + 1, now));
                    }
                    7 => {
                        press(&mut s, key::A);
                        s.quit = Some((8, now));
                    }
                    8 => {
                        shot(&mut commands, &mut s, "4b_quit_confirm");
                        press(&mut s, key::A);
                        s.quit = Some((9, now));
                    }
                    // still paused 5 s after the confirm: the A was not taken, press it again
                    9 if gm.paused && now - at > 5.0 => {
                        log::warn!("smoke-test: still paused after the quit confirm, pressing A again at {now:.1}");
                        s.quit = Some((8, now));
                    }
                    _ => {}
                }
            }
        }
    } else if let Some(f) = fe.frontend.as_ref().filter(|_| !direct::enabled() || s.quit.is_some()) {
        // cruise races ship "Available: NO": the test runs in unlock mode (`--slice-test` implies `-u`)
        let f = &*f;
        let cur = f.fe.current.map(|c| f.fe.screens[c].crc).unwrap_or(0);
        if s.screen.0 != cur {
            s.screen = (cur, s.t);
            log::info!("smoke-test: screen {cur:08x} race type {} at {:.1}", f.fe.quick_race.race_type, s.t);
            if cur == crc("HangoutScreen") && career_mode {
                let name = if s.quit.is_some() { "5c_career_back" } else { "0f_hangout" };
                shot(&mut commands, &mut s, name);
            }
        }
        let ours = c3 && (post_race || career3::fe(&mut commands, &mut s, f, cur, ready, &gm));
        // let each screen settle (enter animations ignore input)
        if !ours && ready && s.t - s.screen.1 > 1.5 {
            if cur == crc("LegalScreen") {
                // wait the pages out
            } else if cur == crc("ProfileSelectScreen") {
                // career: A on the first (empty) slot opens the on-screen keyboard (cursor on DONE): UP UP (backspace,
                // 'Q'), A types "Q", DOWN DOWN back to DONE, A -> NewProfile { "Q" }; quick race: B back
                shot(&mut commands, &mut s, "0e_profile_select");
                if career_mode && s.career == 0 {
                    press(&mut s, key::A);
                    s.career = 10;
                } else if career_mode && (10..16).contains(&s.career) {
                    const KB: [usize; 6] = [key::UP, key::UP, key::A, key::DOWN, key::DOWN, key::A];
                    let k = KB[(s.career - 10) as usize];
                    press(&mut s, k);
                    s.career = if s.career == 15 { 1 } else { s.career + 1 };
                } else if !career_mode {
                    press(&mut s, key::B);
                }
            } else if cur == crc("PlateCustomizationScreen") {
                // a new profile goes through the plate screen: A picks the style, the keyboard opens on DONE, A
                // finishes (then the front end saves the profile)
                shot(&mut commands, &mut s, "0g_plate");
                press(&mut s, key::A);
            } else if cur == crc("MainMenuScreen") && attract_mode {
                // SMOKE_ATTRACT: idle on the main menu until the attract mode starts; back from it = done
                // two attract races back to back: after the first, idle again for the next one
                shot(&mut commands, &mut s, "0a_main_menu");
                if s.race_t.is_some() && s.done_at.is_none() {
                    s.attract_runs += 1;
                    log::info!("smoke-test: attract race {} over, main menu at {:.1}", s.attract_runs, s.t);
                    if s.attract_runs < 2 {
                        shot(&mut commands, &mut s, "8c_attract_back");
                        s.race_t = None;
                        s.intro_t = None;
                    } else {
                        shot(&mut commands, &mut s, "8e_attract_back2");
                        let now = s.t;
                        s.done_at = Some(now + 2.0);
                    }
                }
            } else if cur == crc("MainMenuScreen") && career_mode {
                // career: Start -> profile select (new profile) -> back here, down to the career item, select
                // (the hangout then starts UT0EVENT1 by itself)
                if s.quit.is_some() {
                } else if !s.shots.contains(&"_started") {
                    shot(&mut commands, &mut s, "0a_main_menu");
                    s.shots.push("_started");
                    press(&mut s, key::START);
                } else if s.career == 1 && s.profile.is_some() {
                    press(&mut s, key::DOWN);
                    s.career = 2;
                } else if s.career == 2 {
                    press(&mut s, key::A);
                    s.career = 3;
                }
            } else if cur == crc("MainMenuScreen") {
                shot(&mut commands, &mut s, "0a_main_menu");
                // Start (profile select, B back), then Quick Race
                if !s.shots.contains(&"_started") {
                    s.shots.push("_started");
                    press(&mut s, key::START);
                } else {
                    press(&mut s, key::A);
                }
            } else if cur == crc("QuickRaceTextScreen") {
                let q = &f.fe.quick_race;
                let row = f.quick_race_text.lock().map(|t| t.row).unwrap_or(0);
                let track = q.track.map(|t| &f.fe.events.tracks[t]);
                // an LADown circuit (grid, start line and opponents)
                // SMOKE_RACE_TYPE picks the race type (1 cruise, default 3 circuit)
                let want_type =
                    std::env::var("SMOKE_RACE_TYPE").ok().and_then(|v| v.parse().ok()).unwrap_or(3);
                // SMOKE_AREA (1 LA, 2 London, 3 Tokyo) and SMOKE_TRACK (circuits; default LADOWN) pick the track
                let want_area: u32 = std::env::var("SMOKE_AREA").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
                // without SMOKE_TRACK: LADOWN in LA, else the area's first track with a race of that type
                let want_track = std::env::var("SMOKE_TRACK").unwrap_or_else(|_| {
                    let ev = &f.fe.events;
                    let first = ev.tracks.iter().find(|t| {
                        t.area == want_area && t.races.get(want_type as usize).is_some_and(|r| !r.is_empty())
                    });
                    match first {
                        _ if want_area == 1 => "LADOWN".into(),
                        Some(t) => t.name.clone(),
                        None => "LADOWN".into(),
                    }
                });
                // SMOKE_RACE=N: the track's race N of that type (race dir RACE%02d, e.g. 2 = LADown Circuit RACE02,
                // which has trailblazer triggers); row 4 steps the first race slot
                let want_race = std::env::var("SMOKE_RACE").ok().and_then(|v| v.parse::<u32>().ok());
                let race_ok = |r: Option<usize>| match (want_race, r) {
                    (None, _) => true,
                    (Some(n), Some(r)) => f.fe.events.races[r].path.to_ascii_uppercase().contains(&format!("RACE{n:02}")),
                    (Some(_), None) => false,
                };
                // SMOKE_RACE_PATH=TRACK\TYPE\RACEnn (coverage sweep, tools/sweep.py): that exact race; category
                // (row 0), area (1), type (3), race slot (4) are stepped until races[0] is it
                let race_key = |r: usize| {
                    let race = &f.fe.events.races[r];
                    format!("{}\\{}", f.fe.events.tracks[race.track].name, race.path).to_ascii_uppercase().trim_end_matches('\\').to_string()
                };
                let sweep_target = std::env::var("SMOKE_RACE_PATH").ok().map(|p| {
                    let p = p.replace('/', "\\").to_ascii_uppercase();
                    let p = p.trim_matches('\\').to_string();
                    (p.clone(), (0..f.fe.events.races.len()).find(|&r| race_key(r) == p))
                });
                if let Some((p, None)) = &sweep_target {
                    log::error!("smoke-test: SMOKE_RACE_PATH {p} not in the event database");
                    exit.write(AppExit::error());
                    return;
                }
                let sweep_row = sweep_target.as_ref().and_then(|(_, r)| *r).map(|r| {
                    let want = &f.fe.events.races[r];
                    let want_track = &f.fe.events.tracks[want.track];
                    let cur = q.races[0].map(|c| &f.fe.events.races[c]);
                    match (track, cur) {
                        (_, Some(c)) if c.category != want.category => 0,
                        (Some(t), _) if t.area != want_track.area => 1,
                        _ if q.race_type != want.race_type as u32 => 3,
                        _ if q.races[0] != Some(r) => 4,
                        _ => -1,
                    }
                });
                if s.screen.1 + 60.0 < s.t && sweep_row.is_some_and(|r| r >= 0) {
                    log::error!("smoke-test: SMOKE_RACE_PATH not reachable from the quick race screen");
                    exit.write(AppExit::error());
                    return;
                }
                let want_row = match track {
                    _ if sweep_row.is_some() => sweep_row.unwrap_or(-1),
                    Some(t) if t.area as u32 != want_area => 1,
                    _ if q.race_type != want_type => 3,
                    Some(t) if want_type == 3 && !t.name.eq_ignore_ascii_case(&want_track) => 4,
                    _ if !race_ok(q.races[0]) => 4,
                    _ => -1,
                };
                // SMOKE_WET=1: options section (1) row 3 = weather -> rain (FEQuickRaceTextScreen 0x0022A380)
                let (section, weather) = f.quick_race_text.lock().map(|t| (t.section, t.weather)).unwrap_or((0, 0));
                let want_wet = std::env::var_os("SMOKE_WET").is_some() && weather != 1;
                if want_row < 0 && want_wet {
                    if section == 1 && row == 3 {
                        press(&mut s, key::RIGHT);
                    } else if section == 1 && row > 3 {
                        press(&mut s, key::UP);
                    } else {
                        press(&mut s, key::DOWN);
                    }
                } else if want_row < 0 {
                    if !s.shots.contains(&"0b_quick_race") {
                        let picked = q.races[0].map(race_key).unwrap_or_default();
                        log::info!("smoke-test: quick race {picked} weather {weather} at {:.1}", s.t);
                    }
                    shot(&mut commands, &mut s, "0b_quick_race");
                    press(&mut s, key::A);
                } else if row < want_row {
                    press(&mut s, key::DOWN);
                } else if row > want_row {
                    press(&mut s, key::UP);
                } else {
                    press(&mut s, key::RIGHT);
                }
            } else if cur == crc("QuickRaceCarSelectScreen") {
                press(&mut s, key::A);
            } else if cur == crc("RaceCompleteScreen") {
                shot(&mut commands, &mut s, "5a_race_complete");
            } else if cur == crc("QuickRaceStartScreen") {
                if s.quit.is_some() {
                    // back from the quit: the start screen again, then done
                    shot(&mut commands, &mut s, "5b_quick_race_start_after_quit");
                    if !s.quit_done {
                        s.quit_done = true;
                        let now = s.t;
                        s.done_at = Some(now);
                    }
                } else {
                    shot(&mut commands, &mut s, "0c_quick_race_start");
                    press(&mut s, key::A);
                }
            }
        }
    }
    // SMOKE_ATTRACT: no pad input in the race (Select / Start would leave the attract mode); SMOKE_REVIEW: none once
    // the review cameras are on (the ai drives the player); the results OSD's A presses (session PostRace) still go out
    let review_on = std::env::var_os("SMOKE_REVIEW").is_some() && s.shots.contains(&"_review_on");
    if *mode.get() == EGameMode::InGame && (attract_mode || review_on || c3) && session.state != ESessionState::PostRace {
        keys = [0.0; NUM_KEYS];
    }
    set_script(Some(keys));
    // overall limit: 300 s, or 120 s beyond SMOKE_DRIVE
    let limit = std::env::var("SMOKE_DRIVE").ok().and_then(|v| v.parse::<f32>().ok()).map_or(300.0, |d| (d + 120.0).max(300.0));
    if s.done_at.is_some_and(|d| s.t - d > 1.0) || s.t > limit {
        if s.t > limit {
            log::error!("smoke-test: timed out");
        }
        set_script(None);
        exit.write(AppExit::Success);
    }
}

/// SMOKE_CAREER needs an empty profile list (a leftover profile in slot 0 turns the new-profile walk into a load):
/// a new folder under `VELOCITY_SAVE_DIR` (or `<exe dir>\smoke_saves`) becomes this run's `VELOCITY_SAVE_DIR`.
/// Called first thing in `main`, before any thread reads the environment.
pub fn fresh_save_dir() {
    if std::env::var_os("SMOKE_CAREER").is_none() {
        return;
    }
    let base = std::env::var_os("VELOCITY_SAVE_DIR").map(std::path::PathBuf::from).unwrap_or_else(|| {
        std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("smoke_saves"))).unwrap_or_else(|| "smoke_saves".into())
    });
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    let dir = base.join(format!("career_{stamp}_{}", std::process::id()));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("smoke-test: cannot create save dir {}: {e}", dir.display());
        return;
    }
    // SAFETY: called at the top of main while the process is single threaded
    unsafe { std::env::set_var("VELOCITY_SAVE_DIR", &dir) };
    eprintln!("smoke-test: career save dir {}", dir.display());
}

/// SMOKE_JUMP_CAM=1: the "Jump Cam" video option (`byte_75B278`) on, and a shot / log line while the jump cam
/// (`dword_749110` == 2) is up
fn watch_jump_cam(
    mut commands: Commands,
    mut s: ResMut<Script>,
    opts: Option<ResMut<velocity_frontend::game_options::GameOptions>>,
    cams: Res<velocity_game::camera::game_cameras::GameCameras>,
    cars: Query<(&velocity_game::game_mode::race_cars::RaceCar, &velocity_physics::plugin::PhysicsCarState)>,
    mut last: Local<u32>,
    mut rising: Local<bool>,
) {
    if std::env::var_os("SMOKE_JUMP_CAM").is_none() {
        return;
    }
    // take-offs of the player car (upward speed over 3 m/s)
    if let Some((_, st)) = cars.iter().find(|(c, _)| c.slot == 0) {
        let up = st.velocity.y > 3.0;
        if up && !*rising {
            log::info!("smoke-test: player rising {:.1} m/s at {:?} (speed {:.1}) t {:.1}", st.velocity.y, st.position, st.velocity.length(), s.t);
        }
        *rising = up;
    }
    use velocity_frontend::game_options::display;
    if let Some(mut o) = opts {
        if !o.display[display::JUMP_CAM] {
            o.display[display::JUMP_CAM] = true;
        }
    }
    let r = &cams.replay;
    if r.review_cameras != *last {
        log::info!("smoke-test: review cameras {} -> {} (timer {:.3}) at {:.1}", *last, r.review_cameras, r.review_timer, s.t);
        *last = r.review_cameras;
    }
    if r.review_cameras == 2 {
        shot(&mut commands, &mut s, "9_jump_cam");
    }
}

/// SMOKE_DEBUG_MENU: player 0's camera when the menu opens and when the closed shot is taken (the edit's effect)
fn debug_menu_cam_log(s: Res<Script>, cams: Option<Res<velocity_game::camera::game_cameras::GameCameras>>, mut logged: Local<u32>) {
    let Some(c) = cams else { return };
    let phase = match (s.debug_step, s.debug_closed_at) {
        (1, None) => 1,
        (3, Some(_)) => 2,
        _ => 0,
    };
    if phase > *logged {
        if let Some(v) = c.manager.view(0) {
            let dir = (v.target - v.position).normalize_or_zero();
            log::info!("smoke-test: camera {phase}: up {:.4} dir {:.4} fov {:.4} (ZRotation {})", v.up, dir, v.fov, c.manager.globals.z_rotation);
        }
        *logged = phase;
    }
}

pub fn build(app: &mut App) {
    app.add_systems(Update, watch_jump_cam);
    if std::env::var_os("SMOKE_DEBUG_MENU").is_some() {
        app.add_systems(Update, debug_menu_cam_log);
    }
    app.init_resource::<Script>()
        .add_systems(PreUpdate, run.before(velocity_game::controller::user_input::sample_devices))
        .add_systems(Update, (watch_hits, watch_trail, watch_cams.run_if(in_state(velocity_game::EGameMode::InGame)), watch_replay))
        .add_systems(PostUpdate, flush_shots);
    if direct::enabled() {
        app.add_systems(PreUpdate, direct::drive.before(run));
    }
    if direct::split() {
        app.add_systems(PreUpdate, direct::split_pad.after(run).before(velocity_game::controller::user_input::sample_devices));
    }
    // the smoke implies `-u`; the career walk checks the retail unlocks
    if career3::enabled() {
        app.insert_resource(velocity_game::cli::debug_unlocks::DebugUnlocks::default());
        app.add_systems(Update, career3::ai_drive);
    }
}
