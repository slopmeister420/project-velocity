//! glue: the raceline part of CompartmentManager.cpp 0x000897C0's camera list — per player camera p, race vehicle p
//! (`VsAIDriver::15DB0`) on a race section: its +0x2AC (`VsAIRaceVehicle::24A60`) and the points 40, 80, ... m
//! ahead while < 450 m, at most 9 (`VsAIRaceVehicle::214B0`).

use bevy::prelude::*;
use velocity_ai::plugin::AiState;
use velocity_game::camera::game_cameras::stream_views;
use visos_render::track_render::TrackStreamViews;

/// `VsAIRaceVehicle::214B0`'s section walk, bounded: false when it would never end (each pass goes on with the last
/// sub-route point's join distance + the rest, so a join that lands no nearer the end of its section loops forever).
/// Logs the walk once. The endless walk is original (LACoast circuits' duplicated race sections, e.g. Race05 1 <-> 3;
/// unreachable in F3_D, docs/ai.md); the guard is the port's, a hang counts as a crash.
pub(crate) fn point_ahead_ends(w: &velocity_ai::ai::vs_ai_world::VsAIWorld, start: usize, dist: f32, logged: &mut bool) -> bool {
    let secs = w.race_sections();
    let (mut s, mut d) = (start, dist);
    if s == 0 {
        return true;
    }
    let mut walk = Vec::new();
    for _ in 0..64 {
        let len = secs[s].raceline.get_length();
        if d <= len {
            return true;
        }
        let rem = d - len;
        let Some(pi) = secs[s].sub_route_points.last().copied() else { return true };
        let p = &w.sub_route_points[pi];
        walk.push(format!("section {s} len {len:.1} d {d:.1} -> last sub-route point {pi} (from {} to {} join {} dist {:.1})", p.from, p.to, p.join, p.dist));
        s = p.to;
        d = p.dist + rem;
        if s == 0 {
            return true;
        }
    }
    if !*logged {
        *logged = true;
        log::error!("stream raceline: point_ahead({dist:.1}) from section {start} never ends: {}", walk[..walk.len().min(6)].join(" | "));
    }
    false
}

fn stream_raceline(
    mut ai: Option<ResMut<AiState>>,
    mut views: ResMut<TrackStreamViews>,
    players: Res<velocity_game::camera::game_cameras::StreamViewPlayers>,
    mut logged: Local<bool>,
) {
    let Some(w) = ai.as_mut().and_then(|a| a.world.as_mut()) else { return };
    for (v, &p) in views.views.iter_mut().zip(players.0.iter()) {
        let Some(car) = w.race.players.get(p) else { continue };
        let Some(sec) = car.race_section else { continue };
        let mut pts = Vec::new();
        let (step, mut d, mut j) = (40.0f32, 40.0f32, 1);
        while j < 10 && d < 450.0 {
            if !point_ahead_ends(&w.world, sec, car.dist_along + d, &mut logged) {
                break;
            }
            if std::env::var_os("VELOCITY_TICK_TRACE").is_some() {
                log::info!("tick trace: point_ahead sec {sec} along {} d {d}", car.dist_along);
            }
            let q = car.point_ahead(&mut w.world, d);
            pts.push(Vec3::new(q.x, q.y, q.z));
            j += 1;
            d += step;
        }
        v.raceline = Some((car.lateral, pts));
    }
}

pub fn build(app: &mut App) {
    app.add_systems(
        velocity_game::vs_common_main::GameTick,
        stream_raceline.after(stream_views).run_if(in_state(velocity_game::EGameMode::InGame)),
    );
}
