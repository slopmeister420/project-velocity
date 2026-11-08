//! Start boost VehicleEntity 0x001D1270: gauge bands set the launch result and kick the car forward.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PadInput, PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

const DT: f32 = 1.0 / 24.0;

fn run(gauge: f32) -> (i32, f32) {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    let mut inputs = LocalConfigInputs::default_spans("350Z");
    inputs.is_player = true;
    let a = -std::f32::consts::FRAC_PI_4;
    let id = w.spawn(&SpawnParams { inputs, position: [0.0, 0.6, -2500.0], orientation: [0.0, a.sin(), 0.0, a.cos()], slot: 0 });
    for _ in 0..48 {
        w.step(DT, &[PadInput::default()]);
    }
    w.launch_boost(id, gauge);
    w.step(DT, &[PadInput::default()]);
    let v = w.velocity(id);
    (w.launch_result(id), (v[0] * v[0] + v[2] * v[2]).sqrt())
}

#[test]
fn launch_bands() {
    let (r1, v1) = run(0.5);
    let (r2, v2) = run(0.9);
    let (r3, v3) = run(0.95);
    let (r4, _) = run(1.5);
    eprintln!("launch {r1}/{v1} {r2}/{v2} {r3}/{v3} {r4}");
    assert_eq!((r1, r2, r3, r4), (1, 2, 3, 4));
    assert!(v1 < 0.5 && v2 > 1.0 && v3 > v2);
}
