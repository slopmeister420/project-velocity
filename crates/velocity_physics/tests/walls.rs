//! The wall pass (`PhysicsManager__1A7060`) against a wall built like `Armco::BuildMesh` 0x00012440: a
//! 4 m strip over a line of points, material 21, `FCQuadTree::Build(8.0, 2.375, 0, 1)`. A car driven
//! into it stops at the wall, both as the barrier set and as the Armco set.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::sim_ai::VehicleControlsRaw;
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

/// strip over `pts` (bottom, top per point); faces alternate winding as `CircuitEditorTools__7F8C0` adds them,
/// starting with the flipped one (an Armco line of kind 3): the faces look towards +z, where the car is
fn wall(pts: &[[f32; 3]]) -> (Vec<[f32; 3]>, Vec<[u16; 4]>) {
    let mut v = Vec::new();
    for p in pts {
        v.push(*p);
        v.push([p[0], p[1] + 4.0, p[2]]);
    }
    let mut f = Vec::new();
    let mut flip = false;
    for i in 0..v.len() - 2 {
        let (a, b, c) = (i as u16, i as u16 + 1, i as u16 + 2);
        f.push(if flip { [a, b, c, 21] } else { [a, c, b, 21] });
        flip = !flip;
    }
    (v, f)
}

fn crash(armco: bool) {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    let pts: Vec<[f32; 3]> = (0..=20).map(|k| [-50.0 + 5.0 * k as f32, 0.0, -2540.0]).collect();
    let (v, f) = wall(&pts);
    let t = TrackWorld::from_walls(&v, &f).expect("wall");
    if armco {
        w.set_armco_walls(Some(t));
    } else {
        w.set_barrier_walls(Some(t));
    }
    let a = -std::f32::consts::FRAC_PI_4;
    let mut inputs = LocalConfigInputs::default_spans("350Z");
    inputs.is_player = true;
    // facing -z, towards the wall 40 m away
    let id = w.spawn(&SpawnParams { inputs, position: [0.0, 0.6, -2500.0], orientation: [0.0, a.sin(), 0.0, a.cos()], slot: -1 });
    w.set_human_control(id, false);
    w.set_control_mode(id, 0);
    let c = VehicleControlsRaw { throttle: 1.0, ..Default::default() };
    let mut hits = 0;
    let mut minz = f32::MAX;
    for _ in 0..24 * 12 {
        w.set_controls(id, &c);
        w.step(1.0 / 24.0, &[]);
        hits += w.take_collisions().iter().filter(|h| h.b == -2).count();
        minz = minz.min(w.position(id)[2]);
    }
    let p = w.position(id);
    eprintln!("armco {armco}: end {p:?}, furthest z {minz:.2}, {hits} wall contacts");
    assert!(p.iter().all(|x| x.is_finite()));
    assert!(minz > -2541.0, "drove through the wall");
    assert!(hits > 0, "no wall contact reported");
}

#[test]
fn car_stops_at_barrier() {
    if visos::file::platform::game_dir().is_dir() {
        crash(false);
    }
}

#[test]
fn car_stops_at_armco() {
    if visos::file::platform::game_dir().is_dir() {
        crash(true);
    }
}
