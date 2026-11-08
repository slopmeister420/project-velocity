//! Handling controls on a flat tarmac plane: a constant-steer skidpad (yaw rate, radius, lateral
//! acceleration, steer sign) and a straight-line brake test. `HANDDBG=1` prints the traces.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::fastcar::car::off;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::sim_ai::VehicleControlsRaw;
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

const DT: f32 = 1.0 / 24.0;
const TARMAC: u32 = 14;

fn world(car: &str, player: bool) -> (PhysicsWorld, usize) {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, TARMAC)));
    let mut inputs = LocalConfigInputs::default_spans(car);
    inputs.is_player = player;
    // the car faces -z (body forward row 0); far from the plane edge
    let a = -std::f32::consts::FRAC_PI_4;
    let id = w.spawn(&SpawnParams { inputs, position: [0.0, 0.6, -2500.0], orientation: [0.0, a.sin(), 0.0, a.cos()], slot: -1 });
    w.set_human_control(id, false);
    for _ in 0..48 {
        w.step(DT, &[]);
    }
    (w, id)
}

fn speed(w: &PhysicsWorld, id: usize) -> f32 {
    let v = w.velocity(id);
    (v[0] * v[0] + v[2] * v[2]).sqrt()
}

fn dbg() -> bool {
    std::env::var("HANDDBG").is_ok()
}

/// hold `target` m/s with a throttle/brake controller while steering `steer`; returns
/// (speed, yaw rate about +y per physics second, heading change, front wheel angle)
fn skidpad(car: &str, target: f32, steer: f32) -> (f32, f32, f32, f32) {
    let (mut w, id) = world(car, false);
    let mut c = VehicleControlsRaw::default();
    let mut yaw = Vec::new();
    let mut fwd0 = None;
    let mut fwd_last = [0.0f32; 3];
    let mut wheel_steer = 0.0;
    for f in 0..24 * 40 {
        let s = speed(&w, id);
        c.throttle = ((target - s) * 0.5).clamp(0.0, 1.0);
        c.brake = ((s - target - 1.0) * 0.5).clamp(0.0, 1.0);
        c.steer = if f > 24 * 15 { steer } else { 0.0 };
        w.set_controls(id, &c);
        w.set_steer_angle(id, c.steer);
        w.step(DT, &[]);
        let r = w.body_row(id, 0);
        let fwd = [r[0], r[1], r[2]];
        if f == 24 * 15 {
            fwd0 = Some(fwd);
        }
        if f > 24 * 30 {
            let wv = w.angular_velocity(id);
            yaw.push(wv[1]);
            wheel_steer = w.slot(id).car.m.f(off::STEERING);
        }
        fwd_last = fwd;
        if dbg() && f % 24 == 0 {
            eprintln!("{car} v{target} s{steer} t{:.0}: speed {s:.2} angvel {:?} steer {:.3} pos {:?}", f as f32 / 24.0,
                w.angular_velocity(id), w.slot(id).car.m.f(off::STEERING), w.position(id));
        }
    }
    let f0 = fwd0.unwrap();
    // heading change: did the nose swing towards +x?
    let turn = f0[2] * fwd_last[0] - f0[0] * fwd_last[2];
    let om = yaw.iter().sum::<f32>() / yaw.len() as f32;
    (speed(&w, id), om, turn, wheel_steer)
}

#[test]
fn skidpad_350z() {
    if !visos::file::platform::game_dir().is_dir() {
        return;
    }
    for &(v, s) in &[(10.0f32, 0.3f32), (10.0, -0.3), (20.0, 0.3), (20.0, 1.0), (30.0, 0.5), (40.0, 0.3)] {
        let (sp, om, turn, ws) = skidpad("350Z", v, s);
        // angular velocity is per physics second; the world advances 0.72 physics s per game s
        let a = sp * om.abs();
        eprintln!("350Z target {v} steer {s:+}: speed {sp:.2} yaw {om:+.3} rad/s radius {:.1} m lat {a:.2} m/s^2 ({:.2} g) turn {turn:+.2} wheel {ws:+.3}",
            sp / om.abs(), a / 9.81);
        assert!(sp.is_finite() && om.is_finite());
    }
}

/// full brake from 30 m/s: stopping distance and mean deceleration
#[test]
fn brake_350z() {
    if !visos::file::platform::game_dir().is_dir() {
        return;
    }
    let (mut w, id) = world("350Z", false);
    let mut c = VehicleControlsRaw::default();
    let mut f = 0;
    while speed(&w, id) < 30.0 && f < 24 * 60 {
        c.throttle = 1.0;
        w.set_controls(id, &c);
        w.step(DT, &[]);
        f += 1;
    }
    let p0 = w.position(id);
    let v0 = speed(&w, id);
    c = VehicleControlsRaw { brake: 1.0, ..Default::default() };
    let mut t = 0;
    while speed(&w, id) > 0.5 && t < 24 * 30 {
        w.set_controls(id, &c);
        w.step(DT, &[]);
        t += 1;
        if dbg() && t % 6 == 0 {
            eprintln!("brake t{t}: speed {:.2} gear {}", speed(&w, id), w.car_gear(id));
        }
    }
    let p1 = w.position(id);
    let d = ((p1[0] - p0[0]).powi(2) + (p1[2] - p0[2]).powi(2)).sqrt();
    let secs = t as f32 * DT;
    eprintln!("350Z brake from {v0:.2} m/s: {secs:.2} game s, {d:.1} m, mean {:.2} m/s^2 per game s", v0 / secs);
    assert!(secs < 20.0, "car did not stop");
}

/// `steer_angle_to` (0x003717D0) against targets at known bearings: the angle it returns, fed to
/// `set_steer_angle`, must turn the car towards the target
#[test]
fn steer_angle_to_bearings() {
    if !visos::file::platform::game_dir().is_dir() {
        return;
    }
    let (mut w, id) = world("350Z", false);
    let p = w.position(id);
    let f = w.body_row(id, 0);
    // right-handed y-up: rotating the forward by +b about +y
    let (fx, fz) = (f[0], f[2]);
    for b in [-0.3f32, -0.1, 0.0, 0.1, 0.3] {
        let (dx, dz) = (fx * b.cos() + fz * b.sin(), -fx * b.sin() + fz * b.cos());
        let t = [p[0] + 30.0 * dx, p[1], p[2] + 30.0 * dz];
        let a = w.steer_angle_to(id, t);
        eprintln!("bearing {b:+.2}: steer_angle_to {a:+.4}");
        assert!(a.is_finite());
        if b != 0.0 {
            // positive steer turns the car negatively about +y (skidpad: yaw rate < 0), so a target rotated
            // by +b about +y needs a negative angle
            assert!(a * b < 0.0, "steer_angle_to points away from the target");
            assert!((a.abs() - b.abs()).abs() < 0.15, "angle scale");
        }
    }
}

/// two cars nose to tail at speed: the slipstream search (0x001BA8A0) runs on the car behind
#[test]
fn slipstream_pair() {
    if !visos::file::platform::game_dir().is_dir() {
        return;
    }
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, TARMAC)));
    let a = -std::f32::consts::FRAC_PI_4;
    let q = [0.0, a.sin(), 0.0, a.cos()];
    let mut ids = Vec::new();
    // the rotation leaves the cars facing -z: the follower starts 12 m behind at larger z
    for (k, z) in [(0, -2500.0f32), (1, -2488.0)] {
        let mut inputs = LocalConfigInputs::default_spans("350Z");
        inputs.is_player = k == 1;
        let id = w.spawn(&SpawnParams { inputs, position: [0.0, 0.6, z], orientation: q, slot: -1 });
        w.set_human_control(id, false);
        // PhysicsManager 0x0019E080: player 0, race ai 1 (the ctor's 6 = none is skipped by the search)
        w.set_control_mode(id, if k == 1 { 0 } else { 1 });
        ids.push(id);
    }
    let c = VehicleControlsRaw { throttle: 1.0, ..Default::default() };
    let mut src = None;
    for _ in 0..24 * 25 {
        for &id in &ids {
            w.set_controls(id, &c);
        }
        w.step(DT, &[]);
        src = src.or(w.slipstream_source(ids[1]));
    }
    eprintln!("slipstream source of the follower: {src:?}, speeds {:.1} {:.1}", speed(&w, ids[0]), speed(&w, ids[1]));
    assert!(w.position(ids[1]).iter().all(|x| x.is_finite()));
    assert_eq!(src, Some(ids[0]), "no slipstream behind the leader");
}
