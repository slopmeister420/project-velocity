//! glue: Puddle Render (LightStreak.cpp range, object `unk_720840`). InGameInit's "Puddle Render Init" (0x00131690
//! -> 0x00131AC0) generates the puddles along the ai world's race section 0, and the in-race render path
//! (RainSurface 0x00137090 -> 0x00162400 -> 0x00131F50) runs the puddle / wheel test 0x001326A0 before the draw.
//! Both read the ai world (`VsAIWorld`, race manager drivers), which is the ai crate's, so the binary does it.

use bevy::prelude::*;
use velocity_ai::ai::vs_ai_race_vehicle::VehicleRef;
use velocity_ai::ai::vs_ai_road_section::VsAIRaceSection;
use velocity_ai::ai::vs_math::VsVector4;
use velocity_ai::plugin::AiState;
use velocity_game::camera::game_cameras::GameCameras;
use velocity_game::camera::vs_camera_manager::ECameraMode;
use velocity_game::game_data_loading::game_data_loading::ELoadStep;
use velocity_game::{GameManager, LoadStepMessage};
use visos_render::lens_spray_manager::LensSpray;
use visos_render::puddle_render::{PuddleInput, PuddleRender, PuddleRoadLines};
use visos_render::race_effects::EffectCars;

/// race section 0's lines (0x00131AC0: race line +4, edges +8 / +12)
struct SectionLines<'a>(&'a mut VsAIRaceSection);

fn v3(v: VsVector4) -> Vec3 {
    Vec3::new(v.x, v.y, v.z)
}

impl PuddleRoadLines for SectionLines<'_> {
    fn raceline_length(&mut self) -> f32 {
        self.0.raceline.get_length()
    }
    fn raceline_point(&mut self, d: f32) -> Vec3 {
        let mut idx = 0;
        v3(self.0.raceline.get_point_at_distance(d, 0, &mut idx))
    }
    fn edge_point(&mut self, edge: usize, p: Vec3) -> Vec3 {
        let line = if edge == 0 { &mut self.0.left } else { &mut self.0.right };
        // the race line point as GetPointAtDistance returned it (w = 1)
        let p = VsVector4::new(p.x, p.y, p.z, 1.0);
        let seg = line.find_closest_segment(&p);
        let d = line.get_distance_along_line(&p, seg);
        let mut idx = 0;
        v3(line.get_point_at_distance(d, seg, &mut idx))
    }
}

/// "Puddle Render Init" (after "Skid Smoke Init"): 0x00131690 with the current race's track name (+0x78,
/// lower-cased) containing "london" and `VsAudio::IsTrackWet`; runs in `First`, after the ai's "AI World Data" step
/// (previous frame's `Last`) loaded the race sections. Rand = the shared CRT rand (`GameManager.rand`).
fn puddle_render_init(
    mut steps: MessageReader<LoadStepMessage>,
    mut ai: Option<ResMut<AiState>>,
    mut gm: ResMut<GameManager>,
    puddles: Option<ResMut<PuddleRender>>,
) {
    let Some(mut puddles) = puddles else {
        steps.clear();
        return;
    };
    for m in steps.read() {
        if m.step != ELoadStep::PuddleRender {
            continue;
        }
        let london_wet = gm
            .race
            .as_ref()
            .is_some_and(|r| r.wet && r.track_path.to_ascii_lowercase().contains("london"));
        let section = ai.as_mut().and_then(|a| a.world.as_mut()).and_then(|w| w.world.race_sections_mut().first_mut());
        let gm = &mut *gm;
        match section {
            Some(s) => puddles.init(london_wet, Some(&mut SectionLines(s)), &mut gm.rand),
            None => puddles.init(london_wet, None, &mut gm.rand),
        }
        log::info!("puddle render init: london wet {london_wet}, {} puddles", puddles.puddles.len());
    }
}

/// the in-race render path's part (GameManager 0x000B9010, view flag 2 -> RainSurface 0x00137090): RainSurface
/// draws while it is on (+19/+18: wet track or "LONDON" in the track name, CommonInit 0x00136C50; +16, draw list
/// `dword_6D5330` holding more than its sentinel, is not modelled), `byte_6F7157 == 0` and view 0's camera mode
/// (0x0005FFB0) is not 3; then 0x00131F50 runs the test 0x001326A0 (when +13) before drawing. Per rendered frame on
/// the latest update's cars (the original reads them raw), skipped while paused like the whole render path.
#[allow(clippy::too_many_arguments)]
fn puddle_render_update(
    gm: Res<GameManager>,
    cams: Res<GameCameras>,
    ai: Option<Res<AiState>>,
    cars: Option<Res<EffectCars>>,
    puddles: Option<ResMut<PuddleRender>>,
    input: Option<ResMut<PuddleInput>>,
    lens: Option<ResMut<LensSpray>>,
) {
    let (Some(mut puddles), Some(mut input)) = (puddles, input) else { return };
    if gm.paused {
        return;
    }
    let Some(race) = gm.race.as_ref() else { return };
    let rain_surface = race.wet || race.track_path.to_ascii_uppercase().contains("LONDON");
    let two_players = race.num_players > 1;
    let on = rain_surface && !two_players && cams.manager.mode(0) != ECameraMode::Cine;
    input.rain_surface_off = !on;
    let (Some(cars), Some(mut lens)) = (cars, lens) else { return };
    let world = ai.as_ref().and_then(|a| a.world.as_ref());
    input.race_section = world.is_some_and(|w| !w.world.race_sections().is_empty());
    // the render car's driver: players (VsAIDriver 0x00015DB0) then opponents (0x00017310); its +680 race-line
    // distance (0x00015CA0). Police are render cars after the opponents and never get a distance (0x001326A0
    // sets the driver but skips the test, original); `EffectCars` holds the race cars only.
    input.distance = cars
        .cars
        .iter()
        .map(|c| {
            let w = world?;
            let v = match w.race.find_vehicle_by_id(c.vehicle as u32) {
                VehicleRef::Player(i) => w.race.players.get(i),
                VehicleRef::Ai(i) => w.race.ai.get(i),
                _ => None,
            }?;
            v.race_section.map(|_| v.get_dist_along())
        })
        .collect();
    // 0x0012DC20: the camera manager's current player (+0x60) = its render car (players come first)
    let player = cams.manager.unknown_60 as usize;
    input.camera_car = cars.cars.iter().position(|c| {
        world.is_some_and(|w| matches!(w.race.find_vehicle_by_id(c.vehicle as u32), VehicleRef::Player(i) if i == player))
    });
    if on {
        puddles.update(&input, &cars, &mut lens);
    }
}

pub fn build(app: &mut App) {
    app.add_systems(First, puddle_render_init).add_systems(
        Update,
        puddle_render_update.run_if(in_state(velocity_game::EGameMode::InGame)),
    );
}
