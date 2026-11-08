//! glue: the ai side of `GameAudioInput` (the ai world is the ai crate's). VsAudio::UpdateAiCars 0x00053290,
//! UpdateTraffic 0x00053660 and UpdatePolice 0x00054060 read the race / traffic / police managers each update;
//! here the same reads fill `ai`, `traffic` and `police` after the physics frame (player and camera are core's).

use bevy::prelude::*;
use std::collections::HashMap;
use velocity_ai::ai::vs_ai_police_vehicle::PoliceVehicleMode;
use velocity_ai::plugin::AiState;
use velocity_audio::input::{AiCarAudioInput, GameAudioInput, PoliceAudioInput, TrafficAudioInput};
use velocity_audio::vs_audio_vehics::ai_engine_event;
use velocity_game::game_mode::race_cars::{car_audio_input, RaceCar, RaceCars};
use velocity_game::vs_common_main::{GameTick, InGameUpdateSet};
use velocity_physics::plugin::{PhysicsCar, PhysicsCarState, PhysicsWorldRes};

#[allow(clippy::type_complexity)]
fn ai_audio_input(
    ai: Option<Res<AiState>>,
    cars: Res<RaceCars>,
    q: Query<(&PhysicsCar, &PhysicsCarState, &RaceCar)>,
    physics: Option<Res<PhysicsWorldRes>>,
    mut audio: ResMut<GameAudioInput>,
) {
    audio.ai.clear();
    audio.traffic.clear();
    audio.police.clear();
    let Some(w) = ai.as_ref().and_then(|a| a.world.as_ref()) else { return };
    let states: HashMap<usize, (&PhysicsCarState, bool)> = q.iter().map(|(pc, st, rc)| (pc.id, (st, rc.active))).collect();
    let car = |id: Option<usize>| id.and_then(|i| states.get(&i)).map(|(st, active)| car_audio_input(st, *active));
    // VsVehicleAudio::UpdateEmitters reads the VE; VE+4073 (the lights / bus flag) for the passby kind
    let mut guard = physics.as_deref().map(|p| p.lock());
    // 0x001B6F10 (`engine_rpm`, smooths VE+4068 per call) where the original calls it: UpdateAiCars 0x000534EC
    // only without the startline revs (VsAudioVehics 0x00054F50). UpdateTraffic 0x00053B36 is inside `slot +88 != 0`,
    // which nothing ever sets (audio's scan), so traffic never calls it
    let mut rpm: HashMap<usize, i32> = HashMap::new();
    if let Some(pw) = guard.as_mut().and_then(|g| g.as_mut()) {
        let ids = pw.vehicle_ids();
        let mut call = |id: Option<usize>| {
            if let Some(i) = id.filter(|i| ids.contains(i)) {
                rpm.insert(i, pw.engine_rpm(i));
            }
        };
        for v in &w.race.ai {
            if !v.startline_revs.active {
                call(v.base.physics);
            }
        }
    }
    let car = |id: Option<usize>| {
        car(id).map(|mut c| {
            c.engine_rpm = id.and_then(|i| rpm.get(&i).copied()).unwrap_or(0);
            c
        })
    };
    let world = guard.as_ref().and_then(|g| g.as_ref());
    let ve_4073 = |id: Option<usize>| {
        id.zip(world).is_some_and(|(i, w)| w.vehicle_ids().contains(&i) && w.slot(i).ve.m.b(4073) != 0)
    };
    // 0x00053290: i in players..players + ai -> m_pAIRaceVehicles[i - players] (+356 engine event, revs +260 / 0x189E0)
    for v in &w.race.ai {
        // keep the slot (players + ai index) for an ai without a physics car
        let Some(c) = car(v.base.physics) else {
            audio.ai.push(AiCarAudioInput { present: false, ..Default::default() });
            continue;
        };
        let engine_event = v
            .base
            .physics
            .and_then(|id| cars.ai_engines.iter().find(|(i, _)| *i == id))
            .map(|(_, p)| ai_engine_event(p))
            .unwrap_or(0);
        let revs = &v.startline_revs;
        audio.ai.push(AiCarAudioInput { car: c, engine_event, ai_rpm: revs.active.then(|| revs.get_revs()), present: true });
    }
    // 0x00053660: every traffic manager slot (0x00024AE0), its VE (+0x24)
    if let Some(tm) = w.traffic.as_ref() {
        for v in &tm.vehicles {
            let mut c = car(v.base.physics).unwrap_or_default();
            c.active = c.active && v.base.physics.is_some();
            audio.traffic.push(TrafficAudioInput {
                car: c,
                siren: v.is_triggered_type(),
                emergency: ve_4073(v.base.physics),
                ai_speed: v.base.speed,
                horn: v.hazard,
            });
        }
    }
    // 0x00054060: police units (0x00024C00, 20-byte entries); mode != 0 -> the traffic part's VE, else the race part's
    if let Some(pm) = w.police.as_ref() {
        for v in &pm.vehicles {
            let id = if v.mode != PoliceVehicleMode::Race { v.traffic.base.physics } else { v.race.base.physics };
            audio.police.push(PoliceAudioInput { car: car(id).unwrap_or_default(), siren: v.siren });
        }
    }
}

pub fn build(app: &mut App) {
    app.add_systems(
        GameTick,
        ai_audio_input
            .after(InGameUpdateSet::Physics)
            .before(InGameUpdateSet::Cameras)
            .run_if(in_state(velocity_game::EGameMode::InGame)),
    );
}
