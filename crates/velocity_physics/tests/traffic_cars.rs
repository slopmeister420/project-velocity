//! Every TRAFFIC.BIN setup builds a car through the traffic branch of 0x001CABF0 (spans and wheel radius
//! from the setup) that settles on flat tarmac and drives off under the AI's controls block.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::sim_ai::VehicleControlsRaw;
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;
use velocity_physics::vehicle_database::vehicle_database::CVehicleDatabase;

#[test]
fn every_traffic_setup_drives() {
    if !visos::file::platform::game_dir().is_dir() {
        return;
    }
    let db = CVehicleDatabase::load().unwrap();
    assert!(!db.traffic.is_empty());
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    let mut ids = Vec::new();
    for (k, t) in db.traffic.iter().enumerate() {
        let name = t.name.clone().unwrap_or_default();
        assert_eq!(velocity_physics::vehicle_database::string_hash(&name), t.hash, "{name}: hash");
        let inputs = LocalConfigInputs::from_traffic(&db, k);
        let id = w.spawn(&SpawnParams { inputs, position: [k as f32 * 12.0 - 1000.0, 1.0, 0.0], orientation: [0.0, 0.0, 0.0, 1.0], slot: -1 });
        w.set_human_control(id, false);
        ids.push((id, name));
    }
    let c = VehicleControlsRaw { throttle: 1.0, ..Default::default() };
    for f in 0..24 * 8 {
        for (id, _) in &ids {
            w.set_controls(*id, &if f < 48 { VehicleControlsRaw::default() } else { c });
        }
        w.step(1.0 / 24.0, &[]);
    }
    for (id, name) in &ids {
        let p = w.position(*id);
        let v = w.velocity(*id);
        let s = (v[0] * v[0] + v[2] * v[2]).sqrt();
        eprintln!("{name}: pos {p:?} speed {s:.1} ride {:.2}", p[1]);
        assert!(p.iter().all(|x| x.is_finite()) && p[1] > 0.0 && p[1] < 3.0, "{name}: not on the ground");
        assert!(s > 3.0, "{name}: does not drive");
    }
    // RayCast's car pass (0x0019D3E0, flags 23 as the jump cam) reads box 1; buses have one box (Handling.cfg
    // `BUS number_of_boxes 1`), which panicked past the end of the box array
    for (id, name) in &ids {
        let p = w.position(*id);
        let (from, to) = ([p[0], p[1] + 0.5, p[2] - 6.0], [p[0], p[1] + 0.5, p[2] + 6.0]);
        let h = w.ray_cast(from, to, 23, -1);
        assert!(h.is_some_and(|h| h.kind == 3), "{name}: ray across the car {h:?}");
    }
    // the database lookups the traffic manager makes (0x0018AEF0 / 0x0018AE10)
    let area = db.traffic[0].unk_10;
    let n = db.count_traffic(area, 0);
    assert!(n > 0 && db.get_nth_traffic(area, 0, n - 1).is_some() && db.get_nth_traffic(area, 0, n).is_none());
}
