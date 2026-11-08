//! glue: InGameInit's "Barrier Render Init" (GameManager 0x000B6D1C) hands the ai world's barrier lines to the
//! renderer: `BarrierRender__107560(network_race ? 1 : !professional)` reads `VsAIWorld` barriers 0 (arrows) and
//! 1 (no entry). The ai world is the ai crate's, so the binary does it; the unload (0x00107CE0) and the per-hit
//! call (0x0010B590) are sent by `velocity_game::game_mode::race_effects_bridge`.

use bevy::prelude::*;
use velocity_ai::plugin::AiState;
use velocity_game::game_data_loading::game_data_loading::ELoadStep;
use velocity_game::{GameManager, LoadStepMessage};
use visos_render::barrier_render::LoadBarrierRender;

/// runs in `First`, so the ai's "AI World Data" step (handled in the previous frame's `Last`) has loaded the lines
fn barrier_render_init(
    mut steps: MessageReader<LoadStepMessage>,
    ai: Option<Res<AiState>>,
    gm: Res<GameManager>,
    mut out: MessageWriter<LoadBarrierRender>,
) {
    for m in steps.read() {
        if m.step != ELoadStep::Barriers {
            continue;
        }
        let world = ai.as_ref().and_then(|a| a.world.as_ref()).map(|w| &w.world);
        let lines = |kind: u32| -> Vec<Vec<Vec3>> {
            let Some(w) = world else { return Vec::new() };
            (0..w.get_num_barriers(kind) as i32)
                .filter_map(|i| w.get_barrier(kind, i))
                .map(|l| l.points.iter().map(|p| Vec3::new(p.x, p.y, p.z)).collect())
                .collect()
        };
        let render = if gm.vars.network_race { true } else { !gm.race.as_ref().is_some_and(|r| r.professional) };
        let (arrows, no_entry) = (lines(0), lines(1));
        log::info!("barrier render init: {} arrow / {} no entry lines, render {render}", arrows.len(), no_entry.len());
        out.write(LoadBarrierRender { arrows, no_entry, render });
    }
}

pub fn build(app: &mut App) {
    app.add_systems(First, barrier_render_init);
}
