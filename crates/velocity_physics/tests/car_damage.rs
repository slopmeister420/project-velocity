//! CarDamage (src/CarDamage/CarDamage.cpp): a box-shaped car body is hit head-on twice (exact expected
//! bone pushes, light break and window crack), and a physics car driven into a barrier dents its body.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::car_damage::*;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::sim_ai::VehicleControlsRaw;
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

const IDENT: [f32; 16] = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];

/// surface lattice of a car-sized box (x +-0.85, y 0..1.3, z +-2.2), 0.1 m spacing
fn box_model() -> DamageModel {
    let mut pos = Vec::new();
    let (nx, ny, nz) = (17, 13, 44);
    for i in 0..=nx {
        for j in 0..=ny {
            for k in 0..=nz {
                if i == 0 || i == nx || j == 0 || j == ny || k == 0 || k == nz {
                    pos.push([-0.85 + 0.1 * i as f32, 0.1 * j as f32, -2.2 + 0.1 * k as f32]);
                }
            }
        }
    }
    let attr1 = vec![[0.0; 3]; pos.len()];
    DamageModel { head: [0.7, 0.6, 2.0], brake: [0.7, 0.7, -2.1], meshes: vec![DamageMesh { matrix: IDENT, positions: pos, attr1 }] }
}

fn front_z(cd: &CarDamage) -> f32 {
    cd.model.meshes[0].positions.iter().filter(|p| (p[0] - 0.05).abs() < 0.02 && (p[1] - 0.6).abs() < 0.02).map(|p| p[2]).fold(f32::MIN, f32::max)
}

#[test]
fn head_on_hits() {
    let mut mgr = CarDamageManager::default();
    mgr.add(0, box_model(), true);
    let cd = mgr.get(0).unwrap();
    // corner bones on the body bounds, sides between them (0x0006EDC0)
    assert_eq!(cd.bones[7].rest[0], 0.85);
    assert_eq!(cd.bones[7].rest[2], 2.2);
    assert_eq!(cd.bones[0].rest[0], -0.85);
    assert!((cd.bones[6].rest[2] - 0.7333).abs() < 1e-3 && cd.bones[1].rest[0] == -0.85);
    let z0 = front_z(cd);
    // impulse 40 -> halved 20 -> magnitude 140 -> push 0.14 (< shell 0.25)
    let msg = DamageMessage { a: 0, b: -1, point: [0.3, 0.5, 2.3], normal: [0.0, 0.0, -1.0], impulse: 40.0,
        impulse_vec: [0.0, 0.0, -40.0], scale: 8.0, material: 0 };
    mgr.on_message(&msg, |_| Some(IDENT));
    assert_eq!(mgr.get(0).unwrap().pending, 1 << 7);
    mgr.update(1.0 / 24.0);
    let cd = mgr.get(0).unwrap();
    assert!((cd.bone_push(7) - 0.14).abs() < 1e-5, "push {}", cd.bone_push(7));
    assert_eq!((cd.n_buckles, cd.impacts.len(), cd.pending), (1, 1, 0));
    assert_eq!(cd.impacts[0].bone, 7);
    assert!((cd.impacts[0].magnitude - 140.0).abs() < 1e-4);
    let z1 = front_z(cd);
    assert!(z1 < z0 - 0.05, "front not dented: {z0} -> {z1}");
    let ev = mgr.take_events();
    assert!(ev.iter().any(|e| matches!(e, CarDamageEvent::CollisionFx { vehicle: 0, .. })));
    assert!(ev.iter().any(|e| matches!(e, CarDamageEvent::MeshChanged { vehicle: 0, mesh: 0 })));
    assert!(!ev.iter().any(|e| matches!(e, CarDamageEvent::LightBroken { .. })));
    // second hit: 0.28 > shell, the bone stops at 0.25, the right headlight breaks, the windscreen cracks
    mgr.on_message(&msg, |_| Some(IDENT));
    mgr.update(1.0 / 24.0);
    let cd = mgr.get(0).unwrap();
    assert!((cd.bone_push(7) - 0.25).abs() < 1e-5, "push {}", cd.bone_push(7));
    assert_eq!(cd.car_flags & 1, 0);
    assert_eq!(cd.bonnet_side, 1);
    assert_eq!(cd.cracks, vec![WindowCrack { window: 0, stage: 0, time: cd.cracks[0].time }]);
    assert!((cd.window_damage[0] - 0.42).abs() < 1e-5);
    let ev = mgr.take_events();
    assert!(ev.contains(&CarDamageEvent::LightBroken { vehicle: 0, light: 0 }));
    assert!(ev.contains(&CarDamageEvent::WindowCrack { vehicle: 0, window: 0, stage: 0 }));
    assert_eq!(mgr.stats.headlights, 1);
    // a side-on normal at the front corner is passed to the side bone 6 (0x00070F40)
    let side = DamageMessage { point: [0.9, 0.5, 2.0], normal: [-1.0, 0.0, 0.0], ..msg };
    mgr.on_message(&side, |_| Some(IDENT));
    assert_eq!(mgr.get(0).unwrap().pending, 1 << 6);
    // RestartRace repair: bones and vertices back
    mgr.repair_all();
    let cd = mgr.get(0).unwrap();
    assert_eq!(cd.bone_push(7), 0.0);
    assert_eq!(front_z(cd), z0);
    assert!(cd.impacts.is_empty() && cd.cracks.is_empty() && cd.car_flags & 0x17 == 0x17);
}

#[test]
fn barrier_crash_dents_the_front() {
    if !visos::file::platform::game_dir().is_dir() {
        return;
    }
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    let mut v = Vec::new();
    for k in 0..=20 {
        let x = -50.0 + 5.0 * k as f32;
        v.push([x, 0.0, -2540.0]);
        v.push([x, 4.0, -2540.0]);
    }
    let mut f = Vec::new();
    for i in 0..v.len() as u16 - 2 {
        f.push(if i % 2 == 1 { [i, i + 1, i + 2, 21] } else { [i, i + 2, i + 1, 21] });
    }
    w.set_barrier_walls(Some(TrackWorld::from_walls(&v, &f).expect("wall")));
    let a = -std::f32::consts::FRAC_PI_4;
    let mut inputs = LocalConfigInputs::default_spans("350Z");
    inputs.is_player = true;
    let id = w.spawn(&SpawnParams { inputs, position: [0.0, 0.6, -2500.0], orientation: [0.0, a.sin(), 0.0, a.cos()], slot: -1 });
    w.set_human_control(id, false);
    w.set_control_mode(id, 0);
    let mut mgr = CarDamageManager::default();
    mgr.add(id as i32, box_model(), true);
    // damage model (z front, x lateral) -> car root (+X forward, +Z right): row-vector rows = images of x, y, z
    let local = bevy::math::Mat4::from_cols_array(&[0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    let z0 = front_z(mgr.get(id as i32).unwrap());
    let c = VehicleControlsRaw { throttle: 1.0, ..Default::default() };
    let mut hits = 0;
    for _ in 0..24 * 10 {
        w.set_controls(id, &c);
        w.step(1.0 / 24.0, &[]);
        let m = (bevy::math::Mat4::from_cols_array(&w.body_matrix(id)) * local).to_cols_array();
        for col in w.take_collisions() {
            hits += 1;
            mgr.on_message(&DamageMessage::from(&col), |_| Some(m));
        }
        mgr.update(1.0 / 24.0);
    }
    let cd = mgr.get(id as i32).unwrap();
    let pushes: Vec<f32> = (0..NUM_BONES).map(|i| cd.bone_push(i)).collect();
    eprintln!("{hits} collisions, {} dents, bone pushes {pushes:?}, front z {z0} -> {}", cd.n_buckles, front_z(cd));
    assert!(hits > 0);
    assert!(cd.n_buckles > 0, "no dent from a 40 m run into the barrier");
    let front = pushes[0].max(pushes[7]).max(pushes[1]).max(pushes[6]);
    assert!(front > 0.0 && pushes[3] == 0.0 && pushes[4] == 0.0, "front bones not pushed / rear pushed");
    assert!(cd.impacts.iter().all(|i| i.magnitude >= 10.0 && i.point.iter().all(|x| x.is_finite())));
}
