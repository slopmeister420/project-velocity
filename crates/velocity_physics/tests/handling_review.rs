//! Handling review on the flat tarmac plane with stock database setups (setup_default_vehicle, spans from the
//! model's wheel locators as 0x001CABF0 does): 0-100 km/h, speed reached in each gear, top speed, braking from
//! 100 km/h and the steady-state lateral grip. Times are game seconds (the physics runs 0.72 s per game s);
//! speeds are FastCar m/s. `cargo test --release --test handling_review -- --nocapture` prints the table.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PadInput, PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::sim_ai::VehicleControlsRaw;
use velocity_physics::vehicle::vehicle_entity::{load_local_config_with_fallback, LocalConfigInputs};
use velocity_physics::vehicle_database::{CVehicleDatabase, PartRef, VehicleSetup};

const DT: f32 = 1.0 / 24.0;
const TARMAC: u32 = 14;
const KMH100: f32 = 100.0 / 3.6;
/// `+` = fully upgraded (C2 is left out: its config stops at CD_box_size, engine power 0, see physics.md)
const CARS: [&str; 7] = ["300ZX", "300ZX+", "240SX", "350Z", "SKYLINE", "VIPER", "MINI"];

/// the highest-level part of every performance type (6 air filter .. 12 suspension, 8 NOS) of the car's own
/// set, the best disc/pad of each brake kind and the softest tyre
fn upgrade_fully(db: &CVehicleDatabase, s: &mut VehicleSetup) {
    let Some(set) = s.fitted_bodyparts.iter().flatten().next().map(|p| p.set) else { return };
    for t in 6..=12 {
        let parts = &db.bodypart_sets[set].parts[t];
        if let Some((index, _)) = parts.iter().enumerate().max_by_key(|(_, p)| p.level) {
            s.fitted_bodyparts[t] = Some(PartRef { set, part_type: t, index });
        }
    }
    for slot in 0..2 {
        let Some(cur) = s.brake_parts[slot] else { continue };
        let kind = db.brake_parts[cur].kind;
        s.brake_parts[slot] = (0..db.brake_parts.len()).filter(|&i| db.brake_parts[i].kind == kind).max_by_key(|&i| db.brake_parts[i].level);
    }
    s.tyre = (0..db.tyres.len()).min_by_key(|&i| db.tyres[i].grip);
}

fn inputs(db: &CVehicleDatabase, name: &str) -> Option<(LocalConfigInputs, bool)> {
    let (name, full) = match name.strip_suffix('+') {
        Some(n) => (n, true),
        None => (name, false),
    };
    let vi = db.find_vehicle_by_name(name)?;
    let mut s = VehicleSetup::new();
    db.setup_default_vehicle(&mut s, db.vehicles[vi].hash);
    if full {
        upgrade_fully(db, &mut s);
    }
    let mut i = LocalConfigInputs::from_setup(db, &s);
    let loc = i.spans_from_vehicle_model(db, &s);
    if std::env::var("HRDEFSPANS").is_ok() {
        let d = LocalConfigInputs::default_spans(&i.name);
        (i.span_front_back, i.span_front, i.span_back) = (d.span_front_back, d.span_front, d.span_back);
        (i.distance_to_front_wheels, i.distance_to_rear_wheels) = (d.distance_to_front_wheels, d.distance_to_rear_wheels);
    }
    i.is_player = true;
    Some((i, loc))
}

fn world(inputs: &LocalConfigInputs, human: bool) -> (PhysicsWorld, usize) {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, TARMAC)));
    let a = -std::f32::consts::FRAC_PI_4;
    let id = w.spawn(&SpawnParams {
        inputs: inputs.clone(),
        position: [0.0, 0.6, -2500.0],
        orientation: [0.0, a.sin(), 0.0, a.cos()],
        slot: if human { 0 } else { -1 },
    });
    w.set_human_control(id, human);
    w.set_audio_max_rpm(6000);
    for _ in 0..48 {
        w.step(DT, &[PadInput::default()]);
    }
    (w, id)
}

fn speed(w: &PhysicsWorld, id: usize) -> f32 {
    let v = w.velocity(id);
    (v[0] * v[0] + v[2] * v[2]).sqrt()
}

struct Straight {
    t100: f32,
    gear_max: Vec<(i32, f32)>,
    top: f32,
    t_top: f32,
}

/// full throttle on the pad (automatic gearbox) for 90 game s
fn straight(inp: &LocalConfigInputs) -> Straight {
    let (mut w, id) = world(inp, true);
    let pad = PadInput { axis: 1.0, accelerator: 1.0, ..Default::default() };
    let (mut t100, mut top, mut t_top) = (f32::NAN, 0.0f32, 0.0);
    let mut gear_max: Vec<(i32, f32)> = vec![];
    for f in 0..24 * 90 {
        w.step(DT, &[pad]);
        let s = speed(&w, id);
        let g = w.car_gear(id);
        if t100.is_nan() && s >= KMH100 {
            t100 = (f + 1) as f32 * DT;
        }
        match gear_max.iter_mut().find(|e| e.0 == g) {
            Some(e) => e.1 = e.1.max(s),
            None => gear_max.push((g, s)),
        }
        if s > top + 0.05 {
            t_top = (f + 1) as f32 * DT;
        }
        top = top.max(s);
        if std::env::var("HRDBG").is_ok() && f % 24 == 0 {
            eprintln!("{} t{} gear {g} speed {s:.2} rpm {}", inp.name, f / 24, w.engine_rpm(id));
        }
    }
    gear_max.sort_by_key(|e| e.0);
    Straight { t100, gear_max, top, t_top }
}

/// accelerate past 100 km/h, then full brake on the pad: (start speed, distance, game seconds)
fn brake(inp: &LocalConfigInputs) -> (f32, f32, f32) {
    let (mut w, id) = world(inp, true);
    let pad = PadInput { axis: 1.0, accelerator: 1.0, ..Default::default() };
    let mut f = 0;
    while speed(&w, id) < KMH100 && f < 24 * 60 {
        w.step(DT, &[pad]);
        f += 1;
    }
    let p0 = w.position(id);
    let v0 = speed(&w, id);
    let pad = PadInput { axis: -1.0, brake: 1.0, ..Default::default() };
    let mut t = 0;
    let mut last = v0;
    while t < 24 * 30 {
        w.step(DT, &[pad]);
        t += 1;
        let s = speed(&w, id);
        // stopped, or the brake turned into reverse
        if s < 0.2 || s > last + 0.01 {
            break;
        }
        last = s;
    }
    let p1 = w.position(id);
    let d = ((p1[0] - p0[0]).powi(2) + (p1[2] - p0[2]).powi(2)).sqrt();
    (v0, d, t as f32 * DT)
}

/// constant speed (throttle/brake controller on the AI controls) and a fixed front wheel angle: mean
/// (speed, |yaw rate| per physics s) over the last 8 game s
fn skidpad(inp: &LocalConfigInputs, target: f32, steer: f32) -> (f32, f32) {
    let (mut w, id) = world(inp, false);
    let mut c = VehicleControlsRaw::default();
    let (mut sv, mut yaw, mut n) = (0.0f32, 0.0f32, 0);
    for f in 0..24 * 34 {
        let s = speed(&w, id);
        c.throttle = ((target - s) * 0.5).clamp(0.0, 1.0);
        c.brake = ((s - target - 1.0) * 0.5).clamp(0.0, 1.0);
        c.steer = if f > 24 * 14 { steer } else { 0.0 };
        w.set_controls(id, &c);
        w.set_steer_angle(id, c.steer);
        w.step(DT, &[]);
        if f >= 24 * 26 {
            sv += speed(&w, id);
            yaw += w.angular_velocity(id)[1].abs();
            n += 1;
        }
    }
    (sv / n as f32, yaw / n as f32)
}

/// one skidpad run, `HRCAR` / `HRSTEER` / `HRSPEED` (for PHYSNANTRAP=1 hunts)
#[test]
fn single_skidpad() {
    let (Ok(car), Ok(st)) = (std::env::var("HRCAR"), std::env::var("HRSTEER")) else { return };
    let v: f32 = std::env::var("HRSPEED").ok().and_then(|s| s.parse().ok()).unwrap_or(25.0);
    let db = CVehicleDatabase::load().expect("vehicle database");
    let (inp, _) = inputs(&db, &car).expect("car");
    let (s, om) = skidpad(&inp, v, st.parse().unwrap());
    eprintln!("{car} {st} at {v}: speed {s} yaw {om}");
    assert!(s.is_finite() && om.is_finite());
}

/// the wheel locators 0x001CABF0 reads: every database car's chassis model has them except COPTO and HIN01
/// (those take the original's "ill-chosen defaults")
#[test]
fn wheel_locators_every_car() {
    if !visos::file::platform::game_dir().is_dir() {
        return;
    }
    let db = CVehicleDatabase::load().expect("vehicle database");
    let mut without = vec![];
    for v in &db.vehicles {
        let mut s = VehicleSetup::new();
        db.setup_default_vehicle(&mut s, v.hash);
        let mut i = LocalConfigInputs::from_setup(&db, &s);
        let ok = i.spans_from_vehicle_model(&db, &s);
        if !ok {
            without.push(v.name.clone().unwrap_or_default());
            assert_eq!((i.span_front_back, i.span_front, i.distance_to_front_wheels), (1.0, 2.0, 0.5));
        } else {
            assert!((2.2..3.0).contains(&i.span_front_back) && (1.3..1.7).contains(&i.span_front), "{:?}", v.name);
        }
        eprintln!("{:?}: locators {ok} fb {:.3} f {:.3} b {:.3} df {:.3} dr {:.3}", v.name,
            i.span_front_back, i.span_front, i.span_back, i.distance_to_front_wheels, i.distance_to_rear_wheels);
    }
    without.sort();
    assert_eq!(without, ["COPTO", "HIN01"]);
}

#[test]
fn handling_review_table() {
    if !visos::file::platform::game_dir().is_dir() {
        return;
    }
    let db = CVehicleDatabase::load().expect("vehicle database");
    let only = std::env::var("HRCAR").ok();
    for name in CARS {
        if only.as_deref().is_some_and(|o| !o.eq_ignore_ascii_case(name)) {
            continue;
        }
        let Some((inp, loc)) = inputs(&db, name) else {
            eprintln!("{name}: not in the vehicle database");
            continue;
        };
        let lc = load_local_config_with_fallback(&inp, 0).lc;
        eprintln!(
            "{name}: locators {loc} spans fb {:.3} f {:.3} b {:.3} | pm acc {} top {} brake {} tire {} roll {} nos {} | \
             mass {} power {} speed_limit {} gears {} {:?} x{} wheel r {} brake_torque {} sf_long {} sf_lat {}",
            inp.span_front_back, inp.span_front, inp.span_back, inp.acceleration_pm, inp.top_speed_pm, inp.brake_pm,
            inp.tire_softness_pm, inp.body_roll_pm, inp.nos_pm, lc.mass, lc.engine_power, lc.speed_limit,
            lc.number_of_forward_gears, lc.gear_ratios, lc.gear_ratios_multiplier, lc.back_wheel_radius,
            lc.forwards_brake_torque, lc.static_friction_longitudinal, lc.static_friction_lateral
        );
        let s = straight(&inp);
        let (v0, d, tb) = brake(&inp);
        let mut lat = vec![];
        for st in [0.1f32, 0.2, 0.3, 0.5] {
            let (v, om) = skidpad(&inp, 25.0, st);
            lat.push((st, v, v * om));
        }
        let best = lat.iter().fold(0.0f32, |m, e| m.max(e.2));
        let gears: Vec<String> = s.gear_max.iter().map(|(g, v)| format!("{g}:{:.0}", v * 3.6)).collect();
        eprintln!(
            "ROW | {name} | {:.2} | {} | {:.1} ({:.0} s) | {:.1} m ({:.2} s from {:.1}) | {:.2} g | {}",
            s.t100, gears.join(" "), s.top * 3.6, s.t_top, d, tb, v0 * 3.6, best / 9.81,
            lat.iter().map(|e| format!("{}:{:.1}/{:.2}", e.0, e.1, e.2 / 9.81)).collect::<Vec<_>>().join(" ")
        );
        assert!(s.t100.is_finite() && s.top > KMH100, "{name} never reached 100 km/h");
        assert!(d.is_finite() && d > 0.0 && tb < 29.0, "{name} did not stop");
        assert!(best.is_finite() && best > 0.0);
    }
}
