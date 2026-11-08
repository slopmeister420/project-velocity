//! Two local players (split screen): VehicleManager 0x001DD9F0 gives player i (vehicle array index i <
//! m_nNumPlayers) pad record i (VsTrafficManager +24 + 36·i, read by 0x001CB530); every other car gets -1.
//! Each player car must follow its own pad only.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PadInput, PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

const DT: f32 = 1.0 / 24.0;

fn speed(w: &PhysicsWorld, id: usize) -> f32 {
    let v = w.velocity(id);
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[test]
fn each_player_reads_its_own_pad() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    let ids: Vec<usize> = (0..2)
        .map(|slot| {
            let mut inputs = LocalConfigInputs::default_spans("350Z");
            inputs.is_player = true;
            let p = [0.0, 0.6, -2500.0 + 20.0 * slot as f32];
            w.spawn(&SpawnParams { inputs, position: p, orientation: [0.0, 0.0, 0.0, 1.0], slot })
        })
        .collect();
    for &id in &ids {
        w.set_control_mode(id, 0);
    }
    let idle = PadInput::default();
    for _ in 0..48 {
        w.step(DT, &[idle, idle]);
    }
    let gas = PadInput { axis: 1.0, accelerator: 1.0, ..Default::default() };
    for _ in 0..72 {
        w.step(DT, &[gas, idle]);
    }
    let (a, b) = (speed(&w, ids[0]), speed(&w, ids[1]));
    eprintln!("pad 0 on: {a} / {b}");
    assert!(a > 5.0 && b < 0.5, "player 1 {a}, player 2 {b}");
    for _ in 0..72 {
        w.step(DT, &[idle, gas]);
    }
    let b2 = speed(&w, ids[1]);
    eprintln!("pad 1 on: player 2 {b2}");
    assert!(b2 > 5.0, "player 2 {b2}");
}
