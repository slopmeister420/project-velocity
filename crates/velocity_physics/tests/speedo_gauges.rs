//! Speedo::GetRatioForGauge 0x000F4F40's physics reads (VehicleEntity.h inlines): nos 0x00024720 = +0x60 / +0x64
//! (remaining / capacity, asserted remaining <= capacity), zone 0x000F78D0 = +0x68 / +0x6C, turbo 0x000F7970 /
//! 0x000F79A0 = +0x90 byte / +0x94. Burning nitrous must drain the *remaining* getter.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PadInput, PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

const DT: f32 = 1.0 / 24.0;

#[test]
fn nos_remaining_drains_below_capacity() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    let mut inputs = LocalConfigInputs::default_spans("350Z");
    inputs.is_player = true;
    inputs.nos_pm = 1.0;
    let id = w.spawn(&SpawnParams { inputs, position: [0.0, 0.6, 0.0], orientation: [0.0, 0.0, 0.0, 1.0], slot: 0 });
    let throttle = PadInput { axis: 1.0, accelerator: 1.0, ..Default::default() };
    for _ in 0..24 * 8 {
        w.step(DT, &[throttle]);
    }
    let (rem0, cap) = (w.nos_remaining(id), w.nos_capacity(id));
    eprintln!(
        "nos {rem0}/{cap} ratio {} zone {}/{} turbo {} {}",
        w.nos_ratio(id),
        w.zone_remaining(id),
        w.zone_capacity(id),
        w.has_turbo(id),
        w.turbo_ratio(id)
    );
    assert!(cap > 0.0 && rem0 == cap, "full tank before the boost: {rem0}/{cap}");
    assert_eq!(w.nos_ratio(id), 1.0);
    assert!(w.zone_remaining(id) <= w.zone_capacity(id));
    assert_eq!(w.has_turbo(id), 0.0, "no setup, no turbo part");
    let boost = PadInput { buttons: 0x40, ..throttle };
    for _ in 0..24 {
        w.step(DT, &[boost]);
    }
    let rem1 = w.nos_remaining(id);
    eprintln!("after 1 s boost: {rem1}/{} ratio {}", w.nos_capacity(id), w.nos_ratio(id));
    assert!(rem1 < rem0, "boost drains the remaining field");
    assert_eq!(w.nos_capacity(id), cap, "capacity unchanged");
    // default_spans' tank holds a single frame (1/1), so it is empty after the burn
    assert!(w.nos_ratio(id) < 1.0 && w.nos_ratio(id) >= 0.0);
}
