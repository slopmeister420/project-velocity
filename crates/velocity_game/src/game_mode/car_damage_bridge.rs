//! glue (not an original file): CarDamage on the physics / render crates' public APIs at the original call sites —
//! the damage model built with the car model (CarRender.cpp 0x00110220, then "Car Damage" 0x0006AE70 -> CarDamage
//! 0x0006D150), the repair in `GameManager::RestartRace` (0x000BE0F0 -> 0x0006B090) and the renderer's half of
//! every damage event (CarDamage 0x00073700 / 0x000730C0 / vertex writes through VertexIterator).

use bevy::prelude::*;
use velocity_physics::car_damage::{CarDamageEvent, CarDamageManager, DamageMesh, DamageModel};
use velocity_physics::plugin::{PhysicsCar, PhysicsCarDamage, RepairCarDamage, SetupCarDamage};
use visos_render::car_damage_render::{CarDamageRenderEvent, CarDamageRenderKind, CarDamageSource};
use visos_render::car_render::CarRender;

use crate::game_mode::game_manager::GameModeMessage;
use crate::game_mode::race_cars::{CarKind, RaceCar, MODEL_TO_BODY};

/// the car's damage model has been sent
#[derive(Component)]
pub struct CarDamageSetupDone;

/// the renderer's damage source as physics' damage model (same order: chassis groups, then the parts')
pub fn damage_model(src: &CarDamageSource) -> DamageModel {
    DamageModel {
        head: src.head,
        brake: src.brake,
        meshes: src
            .meshes
            .iter()
            .map(|m| DamageMesh { matrix: m.matrix, positions: m.positions.clone(), attr1: m.attr1.clone() })
            .collect(),
    }
}

/// "Car Damage" (CarRender 0x00110220 -> 0x0006AE70): every race car the race manager knows (body dents only on
/// the human cars: 0x0006D150 arg = !VsAIDriver 0x00015D30) and the police cars (0x0006D150(car, this, 1): no
/// dents); traffic has none. Sent once the physics car and its render model both exist (physics' `car_damage`
/// drops ids not yet in the world). Model space = the render root's (nose +Z) -> body frame (nose +X).
pub(crate) fn setup_car_damage(
    mut commands: Commands,
    cars: Query<(Entity, &RaceCar), (With<PhysicsCar>, Without<CarDamageSetupDone>)>,
    roots: Query<(&CarRender, &CarDamageSource)>,
    mut setup: MessageWriter<SetupCarDamage>,
) {
    for (e, car) in &cars {
        if car.kind == CarKind::Traffic {
            commands.entity(e).insert(CarDamageSetupDone);
            continue;
        }
        let Some((_, src)) = roots.iter().find(|(r, _)| r.request == car.slot as u64) else { continue };
        commands.entity(e).insert(CarDamageSetupDone);
        log::info!("car damage {}: {} vertex groups, head {:?} brake {:?}", car.slot, src.meshes.len(), src.head, src.brake);
        setup.write(SetupCarDamage {
            vehicle: car.slot,
            model: damage_model(src),
            model_to_root: Mat4::from_quat(MODEL_TO_BODY).to_cols_array(),
            deform: car.kind == CarKind::Race && car.human,
        });
    }
}

/// GameManager::RestartRace 0x000BE0F0 -> CarDamage 0x0006B090 (every car fully repaired; 0x000766F0 counters)
pub(crate) fn repair_on_restart(mut r: MessageReader<GameModeMessage>, mut repair: MessageWriter<RepairCarDamage>) {
    for m in r.read() {
        if *m == GameModeMessage::RaceRestart {
            repair.write(RepairCarDamage(None));
        }
    }
}

/// CarDamageFX.cpp 0x00078320 tail: each glass emitter started plays `VsAudio::PlayPhysicsImpact` at the car's
/// position (CarDamage 0x000761B0 -> vehicle position, not the hit point); request = vehicle id.
pub(crate) fn forward_glass(
    mut r: MessageReader<visos_render::car_damage_fx::GlassFxFired>,
    cars: Query<(&PhysicsCar, &velocity_physics::plugin::PhysicsCarState)>,
    mut audio: MessageWriter<velocity_audio::AudioCommand>,
) {
    for g in r.read() {
        if let Some((_, s)) = cars.iter().find(|(c, _)| c.id as u64 == g.request) {
            audio.write(velocity_audio::AudioCommand::GlassImpact { position: s.position });
        }
    }
}

/// the renderer's side of each damage event, for the car's render request (= vehicle id): 0x00073700 light shader
/// swaps + CarRender 0x001157B0 (car 0's +2250 bits, done by the renderer), 0x000730C0 glass stages, dented /
/// repaired vertices, `CollisionFx` (CarDamageFX 0x00078880 impact particles). CarDamage makes no
/// audio calls. Read in the tick that wrote them.
pub(crate) fn forward_car_damage(
    mut r: MessageReader<PhysicsCarDamage>,
    mgr: Res<CarDamageManager>,
    mut out: MessageWriter<CarDamageRenderEvent>,
) {
    for PhysicsCarDamage(e) in r.read() {
        let (vehicle, kind) = match *e {
            CarDamageEvent::LightBroken { vehicle, light } => (vehicle, CarDamageRenderKind::LightBroken(light)),
            CarDamageEvent::LightRepaired { vehicle, light } => (vehicle, CarDamageRenderKind::LightRepaired(light)),
            CarDamageEvent::WindowCrack { vehicle, window, stage } => (vehicle, CarDamageRenderKind::WindowCrack { window, stage }),
            CarDamageEvent::WindowRepaired { vehicle, window } => (vehicle, CarDamageRenderKind::WindowRepaired(window)),
            CarDamageEvent::MeshChanged { vehicle, mesh } => {
                let Some(m) = mgr.get(vehicle).and_then(|c| c.model.meshes.get(mesh)) else { continue };
                (vehicle, CarDamageRenderKind::MeshChanged { mesh, positions: m.positions.clone(), attr1: m.attr1.clone() })
            }
            // CarDamage 0x00070F40 -> CarDamageFX 0x00078880 (world point, impulse as physics left it = halved)
            CarDamageEvent::CollisionFx { vehicle, msg } => {
                (vehicle, CarDamageRenderKind::CollisionFx { vehicle, point: msg.point, impulse: msg.impulse })
            }
        };
        match &kind {
            CarDamageRenderKind::MeshChanged { mesh, .. } => log::debug!("car damage {vehicle}: mesh {mesh} changed"),
            CarDamageRenderKind::CollisionFx { impulse, .. } => log::debug!("car damage {vehicle}: collision fx {impulse}"),
            k => log::info!("car damage {vehicle}: {k:?}"),
        }
        if vehicle >= 0 {
            out.write(CarDamageRenderEvent { request: vehicle as u64, kind });
        }
    }
}
