//! The translated physics (VehicleEntity, FastCar, PhysicsManager::ResolveCollisions) driving on real
//! track collision: settle, then full throttle for 10 s; the car must stay on the collision surface,
//! accelerate and change gear.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::fastcar::mesh::intersect_ray_triangle;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PadInput, PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

/// highest ground under (x, z) below `top`
fn ground(t: &TrackWorld, x: f32, z: f32, top: f32) -> Option<f32> {
    let o = [x, top, z];
    let mut best: Option<f32> = None;
    for f in t.qt.faces_near(o) {
        let [a, b, c] = t.mesh.triangle(f);
        if let Some(h) = intersect_ray_triangle(a, b, c, o, [0.0, -1.0, 0.0]) {
            if h.t >= 0.0 && best.is_none_or(|b| h.t < b) {
                best = Some(h.t);
            }
        }
    }
    best.map(|t| top - t)
}

/// a flat, upward-facing spot with room around it
fn spawn_point(t: &TrackWorld) -> [f32; 3] {
    let n = t.mesh.num_faces;
    let mut k = n / 2;
    loop {
        let nm = t.mesh.normal(k);
        let [a, b, c] = t.mesh.triangle(k);
        let cx = (a[0] + b[0] + c[0]) / 3.0;
        let cy = (a[1] + b[1] + c[1]) / 3.0;
        let cz = (a[2] + b[2] + c[2]) / 3.0;
        if nm[1] > 0.99 {
            let ok = [-4.0f32, 0.0, 4.0].iter().all(|dx| {
                [-4.0f32, 0.0, 4.0].iter().all(|dz| ground(t, cx + dx, cz + dz, cy + 5.0).is_some_and(|g| (g - cy).abs() < 0.1))
            });
            if ok {
                return [cx, ground(t, cx, cz, cy + 5.0).unwrap(), cz];
            }
        }
        k = (k + 7919) % n;
    }
}

fn drive(track_name: &str) {
    let gc = load_global_config_with_fallback(0).unwrap();
    let track = TrackWorld::load(track_name).expect("track collision");
    let p = spawn_point(&track);
    let probe = TrackWorld::load(track_name).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(track));
    let mut inputs = LocalConfigInputs::default_spans("240SX");
    inputs.is_player = true;
    let id = w.spawn(&SpawnParams { inputs, position: [p[0], p[1] + 0.6, p[2]], orientation: [0.0, 0.0, 0.0, 1.0], slot: 0 });
    let dt = 1.0 / 24.0;
    let idle = PadInput::default();
    for _ in 0..60 {
        w.step(dt, &[idle]);
    }
    let rest = w.position(id);
    let ride = rest[1] - p[1];
    println!("{track_name}: spawn {p:?} rest {rest:?} ride height {ride:.3}");
    assert!(ride.is_finite() && ride > 0.0 && ride < 1.0, "car did not settle on the surface: {rest:?}");
    let go = PadInput { accelerator: 1.0, ..Default::default() };
    let mut gears = vec![w.car_gear(id)];
    let mut t100 = None;
    let mut top = 0.0f32;
    for f in 1..=240 {
        w.step(dt, &[go]);
        let v = w.velocity(id);
        let s = (v[0] * v[0] + v[2] * v[2]).sqrt();
        top = top.max(s);
        if t100.is_none() && s * 3.6 >= 100.0 {
            t100 = Some(f as f32 * dt);
        }
        let g = w.car_gear(id);
        if *gears.last().unwrap() != g {
            gears.push(g);
        }
        let pos = w.position(id);
        // the chassis stays at ride height over the collision surface under it
        let gy = ground(&probe, pos[0], pos[2], pos[1] + 2.0).unwrap_or(f32::NAN);
        assert!(pos[1].is_finite() && (pos[1] - gy) > -0.2 && (pos[1] - gy) < 2.0,
            "{track_name}: car left the surface at t {:.2}: pos {pos:?} ground {gy}", f as f32 * dt);
        if f % 24 == 0 {
            println!("t {:4.1}s {:6.1} km/h gear {g} engine {:7.1} pos {pos:?} above ground {:.3}",
                f as f32 * dt, s * 3.6, w.engine_speed(id), pos[1] - gy);
        }
    }
    println!("{track_name}: 0-100 km/h {t100:?} s, top {:.1} km/h in 10 s, gears {gears:?}", top * 3.6);
    assert!(top * 3.6 > 55.0, "car did not accelerate: {top}");
    assert!(gears.len() >= 3, "no gear changes: {gears:?}");
}

#[test]
fn drives_on_london() {
    drive("London");
}

#[test]
fn drives_on_lacoast() {
    drive("LACoast");
}

/// Standing start on ExecTest's long flat straight at the original 24 Hz. Expected from the 240SX
/// config alone: engine_power 107.4 kW, mass 1721 kg, ~83% reaching the wheels, traction-limited
/// ~4.6 m/s^2 to ~11 m/s, and physics advancing `1/24 * 24 * LC->timestep` = 0.03 s per 1/24 s frame
/// (0.72x wall time): 100 km/h after ~8.7 s of physics time = ~12 s of game time.
#[test]
fn accelerates_on_exectest() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::load("ExecTest").unwrap()));
    let mut inputs = LocalConfigInputs::default_spans("240SX");
    inputs.is_player = true;
    let id = w.spawn(&SpawnParams { inputs, position: [55.82, -12.57, 133.57], orientation: [0.0, 0.0, 0.0, 1.0], slot: 0 });
    let dt = 1.0 / 24.0;
    for _ in 0..48 {
        w.step(dt, &[PadInput::default()]);
    }
    let go = PadInput { accelerator: 1.0, ..Default::default() };
    let mut t100 = None;
    for f in 1..=24 * 16 {
        w.step(dt, &[go]);
        let v = w.velocity(id);
        if t100.is_none() && (v[0] * v[0] + v[2] * v[2]).sqrt() * 3.6 >= 100.0 {
            t100 = Some(f as f32 * dt);
        }
    }
    println!("ExecTest 0-100 km/h {t100:?} s");
    let t = t100.expect("never reached 100 km/h");
    assert!((10.5..13.5).contains(&t), "0-100 {t} s");
}
