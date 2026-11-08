//! The physics side of the trailblazer (TrailBlazer.cpp 0x000C4870 / 0x000C4E50 / 0x000C4FF0), the audio gear
//! step 0x001B74A0 and the police handling scaling of PhysicsManager 0x0019E080, on a flat tarmac plane.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::{Cx, Ptr, TrackWorld};
use velocity_physics::vehicle::vehicle_entity::sim::{PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::sim_ai::{TrailBlazerGlobals, VehicleControlsRaw};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

const DT: f32 = 1.0 / 24.0;

fn world() -> (PhysicsWorld, usize) {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    let inputs = LocalConfigInputs::default_spans("350Z");
    let a = -std::f32::consts::FRAC_PI_4;
    let id = w.spawn(&SpawnParams { inputs, position: [0.0, 0.6, -2500.0], orientation: [0.0, a.sin(), 0.0, a.cos()], slot: -1 });
    w.set_human_control(id, false);
    w.set_control_mode(id, 1);
    for _ in 0..24 {
        w.step(DT, &[]);
    }
    (w, id)
}

fn time_scale(w: &mut PhysicsWorld) -> f32 {
    Cx::new(&w.gc, &w.gcm, &mut w.glob).f(Ptr::glob(0x0068_8B70))
}

#[test]
fn trail_blazer_globals_flag_and_reset() {
    let (mut w, id) = world();
    let saved = w.trail_blazer_globals();
    assert!(saved.iter().all(|v| v.is_finite()));
    // 0x000C4870
    w.set_trail_blazer_globals(&TrailBlazerGlobals {
        zone_sensitivities: Some(1.5),
        friction_sa: Some(1.0),
        steer_bias_steering: Some(1.0),
        steer_bias: Some(1.0),
        ..Default::default()
    });
    w.set_trail_blazer_flag(id, true);
    assert!(w.trail_blazer_flag(id));
    let g = w.trail_blazer_globals();
    assert_eq!([g[1], g[2], g[3], g[4]], [1.5, 1.0, 1.0, 1.0]);
    assert_eq!((g[0], g[5]), (saved[0], saved[5]));
    assert_eq!(w.gc.zone_sensitivities_multiplier, 1.5);
    // 0x000C4FF0 at full progress, then drive with the flag on
    w.set_trail_blazer_globals(&TrailBlazerGlobals { zone_velocity: Some(67.056), g_686b58: Some(110.0), ..Default::default() });
    let c = VehicleControlsRaw { throttle: 1.0, ..Default::default() };
    for _ in 0..24 * 5 {
        w.set_controls(id, &c);
        w.step(DT, &[]);
    }
    assert!(w.position(id).iter().all(|v| v.is_finite()));
    // 0x000C4E50: flag off, restore, reset
    w.set_trail_blazer_flag(id, false);
    w.set_trail_blazer_globals(&TrailBlazerGlobals {
        zone_velocity: Some(saved[0]),
        zone_sensitivities: Some(saved[1]),
        friction_sa: Some(saved[2]),
        steer_bias_steering: Some(saved[3]),
        steer_bias: Some(saved[4]),
        g_686b58: Some(saved[5]),
    });
    assert_eq!(w.trail_blazer_globals(), saved);
    w.trail_blazer_reset(id);
    assert_eq!(time_scale(&mut w), 1.0);
    for _ in 0..24 {
        w.step(DT, &[]);
    }
    assert!(w.position(id).iter().all(|v| v.is_finite()));
}

#[test]
fn shift_gear_follows_the_request() {
    let (mut w, id) = world();
    let c = VehicleControlsRaw { throttle: 1.0, ..Default::default() };
    let mut held = Vec::new();
    for _ in 0..24 * 20 {
        w.set_controls(id, &c);
        w.step(DT, &[]);
        let g = w.shift_gear(id);
        held.push(if g >= 100 { g - 100 } else { g });
    }
    let req = w.gear(id);
    let last = *held.last().unwrap();
    assert!(req >= 2, "requested gear {req}");
    assert_eq!(last, req, "held gear {last} vs request {req}");
    // it steps one gear at a time
    assert!(held.windows(2).all(|p| (p[1] - p[0]).abs() <= 1), "{held:?}");
}

#[test]
fn police_handling_scales_the_config() {
    let (mut w, id) = world();
    // VE+3220 = car +2980
    let before = w.slot(id).car.m.f(2980);
    w.police_handling(id);
    assert_eq!(w.slot(id).car.m.f(2980), before * 2.0);
    for _ in 0..24 {
        w.step(DT, &[]);
    }
    assert!(w.position(id).iter().all(|v| v.is_finite()));
}
