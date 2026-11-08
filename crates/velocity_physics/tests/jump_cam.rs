//! Jump cam (VehicleEntity 0x001CCAD0): when a player car takes off (horizontal speed > `PL_min_horisontal_speed`
//! 30 mph) the landing predictor sets the flight time +4592; above `dword_6889F4` (0.65) the car asks the replay
//! camera manager 0x001665E0(1), checks the Jump Cam option `byte_75B278` and `PL_jump_cam_probability` (1/3)
//! against VsRandUnit, then sets `dword_749110` = 2, calls 0x001665A0 and starts the countdown at flight time -
//! `dword_6889FC` (0.1).
#![cfg(feature = "tr")]

use std::sync::{Arc, Mutex};
use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::{JumpCamCall, JumpCamWorld, TrackWorld};
use velocity_physics::vehicle::vehicle_entity::sim::{PadInput, PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

const DT: f32 = 1.0 / 24.0;

/// up to `jumps` launches of a player 350Z from `height` at `speed` along +x over a flat plane; stops at the
/// first jump cam start. Returns the world and the manager calls seen
fn jumps(height: f32, speed: f32, can_see: bool, option: bool, jumps: usize) -> (PhysicsWorld, Vec<JumpCamCall>) {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    let mut inputs = LocalConfigInputs::default_spans("350Z");
    inputs.is_player = true;
    let id = w.spawn(&SpawnParams { inputs, position: [-1000.0, 0.6, 0.0], orientation: [0.0, 0.0, 0.0, 1.0], slot: 0 });
    for _ in 0..24 {
        w.step(DT, &[PadInput::default()]);
    }
    let calls = Arc::new(Mutex::new(Vec::new()));
    let seen = calls.clone();
    w.set_jump_cam_hook(Some(Arc::new(Mutex::new(move |c, _: &mut dyn JumpCamWorld| {
        seen.lock().unwrap().push(c);
        can_see
    }))));
    w.set_jump_cam_option(option);
    'outer: for j in 0..jumps {
        let q = w.orientation(id);
        w.teleport(id, [-1000.0 + 60.0 * j as f32, height, 0.0], q, [speed, 0.0, 0.0]);
        for _ in 0..24 * 3 {
            w.step(DT, &[PadInput::default()]);
            if w.review_cameras() == 2 {
                break 'outer;
            }
        }
    }
    let c = calls.lock().unwrap().clone();
    (w, c)
}

#[test]
fn long_flight_starts_jump_cam() {
    let (mut w, calls) = jumps(8.0, 25.0, true, true, 12);
    let (countdown, start) = w.review_timer();
    eprintln!("calls {} review {} timer {countdown} / {start}", calls.len(), w.review_cameras());
    assert_eq!(calls.first(), Some(&JumpCamCall::CanSee { choose: true }));
    assert_eq!(w.review_cameras(), 2);
    assert_eq!(calls.last(), Some(&JumpCamCall::Start));
    assert!(countdown > 0.55 && countdown == start);
    // 0x001C5800 lerps dword_68580C while the countdown runs; InGameUpdate's end puts 24.0 back
    // the lerp runs only for control mode 0 (VE+0x13DC, a bound player car)
    w.set_control_mode(0, 0);
    w.set_review_timer(start);
    w.step(DT, &[PadInput::default()]);
    let slow = w.jump_cam_rate();
    eprintln!("rate during the jump cam {slow}, review {} timer {:?}", w.review_cameras(), w.review_timer());
    assert!(slow.is_finite() && slow > 0.0 && slow != 24.0);
    w.set_jump_cam_rate(24.0);
    assert_eq!(w.jump_cam_rate(), 24.0);
}

/// the 1/3 roll (0x001CF070 VsRandUnit -> CRT rand 0x003DE030) draws from the state handed in with
/// `set_rand_state` (the game's one stream) and hands the advanced state back
#[test]
fn roll_uses_the_handed_in_rand_stream() {
    use visos::handy::crt_rand::CrtRand;
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut started = Vec::new();
    for seed in [1u32, 7, 42, 1234, 99991, 0xDEAD_BEEF, 31337, 2024] {
        let mut w = PhysicsWorld::new(gc.clone(), Some(TrackWorld::flat(3000.0, 14)));
        let mut inputs = LocalConfigInputs::default_spans("350Z");
        inputs.is_player = true;
        let id = w.spawn(&SpawnParams { inputs, position: [-1000.0, 0.6, 0.0], orientation: [0.0, 0.0, 0.0, 1.0], slot: 0 });
        for _ in 0..24 {
            w.step(DT, &[PadInput::default()]);
        }
        let asked = Arc::new(Mutex::new(0usize));
        let seen = asked.clone();
        w.set_jump_cam_hook(Some(Arc::new(Mutex::new(move |c, _: &mut dyn JumpCamWorld| {
            if matches!(c, JumpCamCall::CanSee { .. }) {
                *seen.lock().unwrap() += 1;
            }
            true
        }))));
        w.set_jump_cam_option(true);
        let q = w.orientation(id);
        w.teleport(id, [-1000.0, 8.0, 0.0], q, [25.0, 0.0, 0.0]);
        w.set_rand_state(seed);
        let mut game = CrtRand { state: seed };
        for _ in 0..24 * 3 {
            let before = *asked.lock().unwrap();
            w.set_rand_state(game.state);
            w.step(DT, &[PadInput::default()]);
            let after = w.rand_state();
            if *asked.lock().unwrap() > before {
                // the roll step advanced the handed-in stream (state reachable from the one handed in)
                let mut r = game.clone();
                assert!((0..64).any(|_| { r.rand(); r.state == after }), "seed {seed}: state not advanced from the handed-in one");
            }
            game.state = after;
            if w.review_cameras() == 2 {
                break;
            }
        }
        assert_eq!(*asked.lock().unwrap(), 1, "seed {seed}: one CanSee per jump");
        started.push(w.review_cameras() == 2);
    }
    eprintln!("jump cam started per seed: {started:?}");
    assert!(started.iter().any(|&s| s) && started.iter().any(|&s| !s));
}

/// the hook's world view inside the step: car position (0x001BA270) and the camera ray (0x00167680, flags 23)
#[test]
fn hook_reads_car_and_ray() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    let mut inputs = LocalConfigInputs::default_spans("350Z");
    inputs.is_player = true;
    let id = w.spawn(&SpawnParams { inputs, position: [-1000.0, 0.6, 0.0], orientation: [0.0, 0.0, 0.0, 1.0], slot: 0 });
    for _ in 0..24 {
        w.step(DT, &[PadInput::default()]);
    }
    let got = Arc::new(Mutex::new(Vec::new()));
    let g = got.clone();
    w.set_jump_cam_hook(Some(Arc::new(Mutex::new(move |c, v: &mut dyn JumpCamWorld| {
        if c == (JumpCamCall::CanSee { choose: true }) && g.lock().unwrap().is_empty() {
            let car = v.car_position(0);
            let cam = [car[0] - 20.0, car[1] + 10.0, car[2]];
            let open = v.blocked(cam, car, 0);
            let ground = v.blocked(cam, [car[0] + 20.0, car[1] - 10.0, car[2]], 0);
            let missing = v.car_position(5);
            g.lock().unwrap().push((car, open, ground, missing));
        }
        false
    }))));
    w.set_jump_cam_option(true);
    for j in 0..6 {
        let q = w.orientation(id);
        w.teleport(id, [-1000.0 + 60.0 * j as f32, 8.0, 0.0], q, [25.0, 0.0, 0.0]);
        for _ in 0..24 * 3 {
            w.step(DT, &[PadInput::default()]);
        }
        if !got.lock().unwrap().is_empty() {
            break;
        }
    }
    let (car, open, ground, missing) = got.lock().unwrap()[0];
    eprintln!("car {car:?} open {open} ground {ground}");
    assert!(car[1] > 0.5 && car[0] > -1000.0, "car in flight: {car:?}");
    assert!(!open, "nothing between camera and car");
    assert!(ground, "ray through the plane hits");
    assert_eq!(missing, [0.0; 3]);
}

/// VsConfigFileRead 0x001B82A0: pad button 0x40 sets the boost flag (controls +20) unless the zone FX amount
/// 0x00154FC0 of the pad is non-zero
#[test]
fn zone_fx_blocks_boost() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    let mut inputs = LocalConfigInputs::default_spans("350Z");
    inputs.is_player = true;
    let id = w.spawn(&SpawnParams { inputs, position: [0.0, 0.6, 0.0], orientation: [0.0, 0.0, 0.0, 1.0], slot: 0 });
    let pad = PadInput { axis: 1.0, accelerator: 1.0, buttons: 0x40, ..Default::default() };
    w.step(DT, &[pad]);
    assert!(w.controls(id).nitrous);
    w.set_zone_fx_level(0, 1.5);
    w.step(DT, &[pad]);
    assert!(!w.controls(id).nitrous);
    w.set_zone_fx_level(0, 0.0);
    w.step(DT, &[pad]);
    assert!(w.controls(id).nitrous);
}

#[test]
fn no_camera_or_option_off_keeps_review_off() {
    let (w, calls) = jumps(8.0, 25.0, false, true, 4);
    assert!(!calls.is_empty() && calls.iter().all(|c| *c == JumpCamCall::CanSee { choose: true }));
    assert_eq!(w.review_cameras(), 0);
    let (w, calls) = jumps(8.0, 25.0, true, false, 4);
    assert!(!calls.is_empty() && !calls.contains(&JumpCamCall::Start));
    assert_eq!(w.review_cameras(), 0);
    // too slow for the landing predictor: never asks
    let (_, calls) = jumps(8.0, 5.0, true, true, 2);
    assert!(calls.is_empty());
}
