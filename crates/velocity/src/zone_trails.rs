//! glue: CZoneTrailRender init (TrailBlazerRender.cpp 0x0014DF70, InGameInit "Trailblazer Effects" step,
//! GameManager 0x000B70B3). 0x0014E480 makes one trail per lane of every trailblazer section of the ai world, so
//! the lines come from `VsAIWorld` (the ai crate's) and the call is made here. The build ray-casts the track
//! collision, which the port loads inside the first GameTick (`race_cars::car_inputs`), so the Init goes out in the
//! first update whose physics world exists, ahead of the trail update.

use bevy::prelude::*;
use velocity_ai::plugin::AiState;
use velocity_game::game_data_loading::game_data_loading::ELoadStep;
use velocity_game::vs_common_main::{GameTick, InGameUpdateSet};
use velocity_game::{EGameMode, LoadStepMessage};
use visos_render::trail_blazer_render::{trail_blazer_render_update, TrailBlazerRender, TrailBlazerRenderCall, ZoneTrailLine};

/// the "Trailblazer Effects" step ran; the Init waits for the track collision
#[derive(Resource, Default)]
struct PendingZoneTrails(bool);

fn zone_trail_step(mut steps: MessageReader<LoadStepMessage>, mut pending: ResMut<PendingZoneTrails>) {
    for m in steps.read() {
        if m.step == ELoadStep::TrailblazerEffects {
            pending.0 = true;
        }
    }
}

/// TrailBlazerRender.cpp 0x0014E480 — every lane line of every trailblazer section, in order
fn zone_trail_lines(ai: &AiState) -> Vec<ZoneTrailLine> {
    let mut lines = Vec::new();
    let Some(w) = ai.world.as_ref() else { return lines };
    let w = &w.world;
    for s in 0..w.get_num_trail_blazer_sections() {
        let Some(sec) = w.get_trail_blazer_section(s) else { continue };
        for (j, lane) in sec.lanes.iter().take(sec.get_num_lanes() as usize).enumerate() {
            let l = &lane.line;
            let points = (0..l.get_num_points()).map(|k| {
                let p = l.get_point(k);
                Vec3::new(p.x, p.y, p.z)
            });
            lines.push(ZoneTrailLine { section: s, lane: j as u32, id: l.get_id(), points: points.collect(), length: l.get_length() });
        }
    }
    lines
}

fn zone_trail_init(
    mut pending: ResMut<PendingZoneTrails>,
    ai: Option<Res<AiState>>,
    physics: Option<Res<velocity_physics::plugin::PhysicsWorldRes>>,
    mut calls: MessageWriter<TrailBlazerRenderCall>,
) {
    if !pending.0 {
        return;
    }
    if !physics.as_deref().is_some_and(|p| p.lock().is_some()) {
        return;
    }
    pending.0 = false;
    let lines = ai.as_deref().map(zone_trail_lines).unwrap_or_default();
    log::info!("zone trails init: {} lines ({:?})", lines.len(), lines.iter().map(|l| (l.section, l.lane, l.id, l.points.len())).collect::<Vec<_>>());
    calls.write(TrailBlazerRenderCall::Init(lines));
}

fn clear_pending(mut pending: ResMut<PendingZoneTrails>) {
    pending.0 = false;
}

/// log only: trail states and the danger particles' gate / running flag when they change
fn log_changes(
    r: Res<TrailBlazerRender>,
    dp: Option<Res<visos_render::danger_zone_render::DangerParticles>>,
    mut last: Local<(Vec<u32>, bool, bool)>,
) {
    let states: Vec<u32> = r.trails.iter().map(|t| t.state).collect();
    if states != last.0 {
        log::info!("zone trails: states {states:?}");
        last.0 = states;
    }
    if let Some(dp) = dp {
        if (dp.enabled, dp.active) != (last.1, last.2) {
            log::info!("danger particles: enabled {} active {} level {:.3}", dp.enabled, dp.active, dp.level);
            (last.1, last.2) = (dp.enabled, dp.active);
        }
    }
}

pub fn build(app: &mut App) {
    if !app.world().contains_resource::<TrailBlazerRender>() {
        return;
    }
    app.init_resource::<PendingZoneTrails>()
        .add_systems(First, zone_trail_step)
        .add_systems(OnExit(velocity_game::InRace), clear_pending)
        .add_systems(Last, log_changes)
        .add_systems(
            GameTick,
            zone_trail_init.in_set(InGameUpdateSet::Effects).before(trail_blazer_render_update).run_if(in_state(EGameMode::InGame)),
        );
}
