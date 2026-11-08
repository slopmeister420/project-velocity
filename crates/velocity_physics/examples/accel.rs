//! standing-start acceleration on the longest flat straight found in a track's collision, at the
//! original 24 Hz frame rate: `cargo run -p velocity_physics --example accel -- <Track> [car]`
use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::fastcar::mesh::intersect_ray_triangle;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PadInput, PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

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

/// (start, heading angle, length): the longest run of ground within 0.15 m of the start height
fn straight(t: &TrackWorld) -> ([f32; 3], f32, f32) {
    let mut best = ([0.0; 3], 0.0, 0.0);
    let n = t.mesh.num_faces;
    for s in 0..400u32 {
        let k = (s.wrapping_mul(2654435761) % n) as u32;
        if t.mesh.normal(k)[1] < 0.99 {
            continue;
        }
        let [a, b, c] = t.mesh.triangle(k);
        let (x, z) = ((a[0] + b[0] + c[0]) / 3.0, (a[2] + b[2] + c[2]) / 3.0);
        let Some(y) = ground(t, x, z, (a[1] + b[1] + c[1]) / 3.0 + 3.0) else { continue };
        for h in 0..16 {
            let ang = h as f32 * std::f32::consts::PI / 8.0;
            let (dx, dz) = (ang.cos(), ang.sin());
            let mut len = 0.0;
            'run: while len < 600.0 {
                for side in [-1.5f32, 0.0, 1.5] {
                    let (px, pz) = (x + dx * (len + 2.0) - dz * side, z + dz * (len + 2.0) + dx * side);
                    match ground(t, px, pz, y + 3.0) {
                        Some(g) if (g - y).abs() < 0.15 => {}
                        _ => break 'run,
                    }
                }
                len += 2.0;
            }
            if len > best.2 {
                best = ([x, y, z], ang, len);
            }
        }
    }
    best
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let tname = args.get(1).map(|s| s.as_str()).unwrap_or("London");
    let car = args.get(2).map(|s| s.as_str()).unwrap_or("240SX");
    let gc = load_global_config_with_fallback(0).unwrap();
    let track = TrackWorld::load(tname).unwrap();
    let (p, ang, len) = straight(&track);
    println!("{tname}: straight from {p:?} heading {:.0} deg, {len} m", ang.to_degrees());
    let mut w = PhysicsWorld::new(gc, Some(track));
    let mut inputs = LocalConfigInputs::default_spans(car);
    inputs.is_player = true;
    // the car faces rotate(q, +X); the engine applies the conjugate, so rotate by -ang about +Y
    let q = [0.0, (-ang / 2.0).sin(), 0.0, (-ang / 2.0).cos()];
    let id = w.spawn(&SpawnParams { inputs, position: [p[0], p[1] + 0.6, p[2]], orientation: q, slot: 0 });
    let dt = 1.0 / 24.0;
    for _ in 0..48 {
        w.step(dt, &[PadInput::default()]);
    }
    let start = w.position(id);
    let go = PadInput { accelerator: 1.0, ..Default::default() };
    let mut t100 = None;
    for f in 1..=24 * 30 {
        w.step(dt, &[go]);
        let v = w.velocity(id);
        let s = (v[0] * v[0] + v[2] * v[2]).sqrt() * 3.6;
        if t100.is_none() && s >= 100.0 {
            t100 = Some(f as f32 * dt);
        }
        let pos = w.position(id);
        let d = ((pos[0] - start[0]).powi(2) + (pos[2] - start[2]).powi(2)).sqrt();
        if f % 24 == 0 {
            println!("t {:4.1} {:6.1} km/h gear {} dist {:6.1}", f as f32 * dt, s, w.car_gear(id), d);
        }
        if d > len - 10.0 {
            break;
        }
    }
    println!("0-100 km/h: {t100:?} s");
}
