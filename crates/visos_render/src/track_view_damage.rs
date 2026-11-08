//! trackview `--damage` (dev tool, not an original module): runs the physics CarDamage model on the first viewer car (request 1) with a
//! few scripted hits and feeds its events to `car_damage_render`, the way core's glue does in game.

use bevy::prelude::*;
use velocity_physics::car_damage::{CarDamageEvent, CarDamageManager, DamageMesh, DamageMessage, DamageModel};

use crate::car_damage_render::{CarDamageRenderEvent, CarDamageRenderKind, CarDamageSource};
use crate::car_render::CarRender;

/// hits in damage model space (x lateral, y up, z front): point, normal, impulse
const HITS: [([f32; 3], [f32; 3], f32); 4] = [
    ([0.75, 0.45, 2.3], [0.0, 0.0, -1.0], 400.0),
    ([-0.75, 0.45, 2.3], [0.0, 0.0, -1.0], 400.0),
    ([1.1, 0.55, 0.4], [-1.0, 0.0, 0.0], 300.0),
    ([0.0, 0.9, 1.2], [0.0, 0.0, -1.0], 400.0),
];

#[derive(Resource, Default)]
pub struct DamageDemo {
    mgr: Option<CarDamageManager>,
    frame: usize,
}

pub fn damage_demo(
    mut demo: ResMut<DamageDemo>,
    cars: Query<(&CarRender, &CarDamageSource, &GlobalTransform)>,
    mut out: MessageWriter<CarDamageRenderEvent>,
) {
    let Some(car) = cars.iter().find(|(c, _, _)| c.request == 1).map(|(_, _, g)| g.to_matrix()) else { return };
    if demo.mgr.is_none() {
        let Some((_, src, _)) = cars.iter().find(|(c, _, _)| c.request == 1) else { return };
        let model = DamageModel {
            head: src.head,
            brake: src.brake,
            meshes: src.meshes.iter().map(|m| DamageMesh { matrix: m.matrix, positions: m.positions.clone(), attr1: m.attr1.clone() }).collect(),
        };
        let mut mgr = CarDamageManager::default();
        mgr.add(0, model, true);
        demo.mgr = Some(mgr);
        return;
    }
    demo.frame += 1;
    // from frame 104, 8 frames apart (CarDamageFX takes one impact per 5 frames): the particles are still out at
    // the viewer's frame-150 shot
    let Some(f) = demo.frame.checked_sub(96) else { return };
    let k = f / 8;
    if f % 8 != 0 || k == 0 || k > HITS.len() {
        return;
    }
    let mgr = demo.mgr.as_mut().unwrap();
    if let Some(&(point, normal, impulse)) = HITS.get(k - 1) {
        let iv = [normal[0] * impulse, normal[1] * impulse, normal[2] * impulse];
        let msg = DamageMessage { a: 0, b: -1, point, normal, impulse, impulse_vec: iv, scale: 8.0, material: 0 };
        let ident = Mat4::IDENTITY.to_cols_array();
        mgr.on_message(&msg, |_| Some(ident));
    }
    mgr.update(1.0 / 24.0);
    for e in mgr.take_events() {
        let kind = match e {
            CarDamageEvent::LightBroken { light, .. } => CarDamageRenderKind::LightBroken(light),
            CarDamageEvent::LightRepaired { light, .. } => CarDamageRenderKind::LightRepaired(light),
            CarDamageEvent::WindowCrack { window, stage, .. } => CarDamageRenderKind::WindowCrack { window, stage },
            CarDamageEvent::WindowRepaired { window, .. } => CarDamageRenderKind::WindowRepaired(window),
            CarDamageEvent::MeshChanged { mesh, .. } => {
                let m = &mgr.get(0).unwrap().model.meshes[mesh];
                CarDamageRenderKind::MeshChanged { mesh, positions: m.positions.clone(), attr1: m.attr1.clone() }
            }
            // the demo's hits are in model space: the particles want the world point
            CarDamageEvent::CollisionFx { vehicle, msg } => CarDamageRenderKind::CollisionFx {
                vehicle,
                point: car.transform_point3(Vec3::from_array(msg.point)).to_array(),
                impulse: msg.impulse,
            },
        };
        log::info!("damage demo: {kind:?}", kind = DebugKind(&kind));
        out.write(CarDamageRenderEvent { request: 1, kind });
    }
}

struct DebugKind<'a>(&'a CarDamageRenderKind);

impl std::fmt::Debug for DebugKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            CarDamageRenderKind::MeshChanged { mesh, .. } => write!(f, "MeshChanged({mesh})"),
            k => write!(f, "{k:?}"),
        }
    }
}
