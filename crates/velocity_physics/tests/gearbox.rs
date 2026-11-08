//! Player gearbox on a flat tarmac plane: the setup's automatic flag (VehicleSetup +0x18C ->
//! 0x001DA8E0) decides whether FastCar's gear choice 0x0035D1E0 shifts up by itself, the pad's gear up
//! (0x001CB530 -> 0x001D1210) shifts a manual car, and the HUD/audio rpm 0x001B6F10 stays in the audio
//! rev range. `GEARDBG=1` prints the traces.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PadInput, PhysicsWorld, SpawnParams, PAD_GEAR_UP};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

const DT: f32 = 1.0 / 24.0;
const TARMAC: u32 = 14;

fn world(automatic: bool) -> (PhysicsWorld, usize) {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, TARMAC)));
    let mut inputs = LocalConfigInputs::default_spans("350Z");
    inputs.is_player = true;
    inputs.automatic = automatic;
    let a = -std::f32::consts::FRAC_PI_4;
    let id = w.spawn(&SpawnParams { inputs, position: [0.0, 0.6, -2500.0], orientation: [0.0, a.sin(), 0.0, a.cos()], slot: 0 });
    w.set_audio_max_rpm(6000);
    for _ in 0..48 {
        w.step(DT, &[PadInput::default()]);
    }
    (w, id)
}

/// full throttle for `frames`; `up_every` > 0 taps gear up (one frame held) that often. Returns the gears
/// seen and the rpm range of 0x001B6F10 (one call per frame, as the HUD makes)
fn drive(w: &mut PhysicsWorld, id: usize, frames: usize, up_every: usize) -> (Vec<i32>, i32, i32) {
    let mut gears = vec![];
    let (mut lo, mut hi) = (i32::MAX, i32::MIN);
    for f in 0..frames {
        let tap = up_every > 0 && f % up_every == up_every - 1;
        let pad = PadInput { axis: 1.0, accelerator: 1.0, buttons: if tap { PAD_GEAR_UP } else { 0 }, ..Default::default() };
        w.step(DT, &[pad]);
        let g = w.car_gear(id);
        let rpm = w.engine_rpm(id);
        if f > 24 {
            lo = lo.min(rpm);
            hi = hi.max(rpm);
        }
        if gears.last() != Some(&g) {
            gears.push(g);
        }
        if std::env::var("GEARDBG").is_ok() && f % 12 == 0 {
            let v = w.velocity(id);
            eprintln!("f {f} gear {g} rpm {rpm} speed {:.1}", (v[0] * v[0] + v[2] * v[2]).sqrt());
        }
    }
    (gears, lo, hi)
}

#[test]
fn automatic_shifts_up() {
    let (mut w, id) = world(true);
    let (gears, lo, hi) = drive(&mut w, id, 24 * 40, 0);
    eprintln!("automatic: gears {gears:?} rpm {lo}..{hi}");
    assert!(gears.iter().copied().max().unwrap() >= 4, "{gears:?}");
    assert!(gears.windows(2).any(|p| p[1] == p[0] + 1));
    // 0x001B6F10 maps FastCar's engine speed into the audio's 1000..6000 band (plus the shift dip)
    assert!(lo >= 900 && hi <= 6500, "{lo}..{hi}");
}

#[test]
fn manual_holds_gear_until_gear_up() {
    let (mut w, id) = world(false);
    let (gears, _, _) = drive(&mut w, id, 24 * 15, 0);
    eprintln!("manual, no shifts: {gears:?}");
    assert!(gears.iter().copied().max().unwrap() <= 1, "{gears:?}");
    let (gears, _, _) = drive(&mut w, id, 24 * 15, 48);
    eprintln!("manual, gear up every 2 s: {gears:?}");
    assert!(gears.iter().copied().max().unwrap() >= 3, "{gears:?}");
}
