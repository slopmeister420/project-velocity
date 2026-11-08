//! Cars placed at the eventdef grid height (above the collision) fall and settle: no launch on landing.
//! (Was a NaN blow-up: 0x003987C0 / 0x003717D0 return their result in st0 and had been typed void.)
#![cfg(feature = "tr")]

use visos::handy::vs_text_file::VsTextFile;
use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

struct Grid {
    start_positions: Vec<[f32; 3]>,
    start_direction: f32,
}

/// the grid part of `eventdef.txt` (the slot shift onto the start line is left out)
/// (name, type, laps, opponents, colours, start positions, player
/// order, start direction)
fn grid(path: &str) -> Grid {
    let mut t = VsTextFile::default();
    assert!(t.load(None, path), "{path}");
    t.read_string().unwrap();
    t.read_s32().unwrap();
    t.read_s32().unwrap();
    let n = t.read_s32().unwrap().1;
    for _ in 0..n {
        t.read_string().unwrap();
    }
    for _ in 0..2 * n {
        t.read_f32n::<3>().unwrap();
    }
    let ns = t.read_s32().unwrap().1;
    let raw: Vec<[f32; 3]> = (0..ns).map(|_| t.read_f32n::<3>().unwrap().1).collect();
    // player order, z negated into game coordinates (GameDataLoading.cpp)
    let start_positions = (0..ns).map(|_| raw[t.read_s32().unwrap().1 as usize]).map(|p| [p[0], p[1], -p[2]]).collect();
    Grid { start_positions, start_direction: t.read_f32().unwrap().1 }
}

fn drop_on_grid(track: &str, race: &str, lift: f32, cars: usize) {
    if !visos::file::platform::game_dir().is_dir() {
        return;
    }
    let def = grid(&format!(r"\Tracks\{track}\{race}\eventdef.txt"));
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::load(track).unwrap()));
    let ang = ((0.0 - def.start_direction) + 90.0).to_radians() * 0.5;
    let q = [0.0, ang.sin(), 0.0, ang.cos()];
    for i in 0..cars {
        let p = def.start_positions[i];
        let mut inputs = LocalConfigInputs::default_spans("350Z");
        inputs.is_player = i == 0;
        let g = w.ray_cast([p[0], p[1] + 5.0, p[2]], [p[0], p[1] - 50.0, p[2]], 1 | 16, -1).map(|h| h.point[1]);
        eprintln!("{track} slot {i}: grid y {:.2}, ground {g:?}", p[1]);
        w.spawn(&SpawnParams { inputs, position: [p[0], p[1] + lift, p[2]], orientation: q, slot: if i == 0 { 0 } else { -1 } });
    }
    let mut worst = 0.0f32;
    let mut settled = 0.0f32;
    for f in 0..96 {
        w.step(1.0 / 24.0, &[]);
        for id in 0..cars {
            let v = w.velocity(id);
            let s = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            if std::env::var("DROPDBG").is_ok() && id == 0 && f < 40 {
                eprintln!("f {f} pos {:?} v {v:?}", w.position(id));
            }
            let s = if s.is_finite() { s } else { f32::INFINITY };
            worst = worst.max(s);
            if f >= 72 {
                settled = settled.max(s);
            }
        }
    }
    eprintln!("{track} lift {lift}: worst speed {worst:.2} m/s, settled {settled:.3} m/s");
    // free fall from the grid height is ~17 m/s at most; anything above is a launch
    assert!(worst < 20.0, "{track}: car launched at {worst} m/s on landing");
    assert!(settled < 1.0, "{track}: car still moving at {settled} m/s");
}

#[test]
fn grid_drop_ladown() {
    drop_on_grid("LADown", r"Circuit\Race01", 0.0, 1);
}

#[test]
fn grid_drop_ladown_lifted() {
    drop_on_grid("LADown", r"Circuit\Race01", 2.0, 1);
}

#[test]
fn grid_drop_london_pack() {
    drop_on_grid("London", r"Circuit\Race01", 0.5, 4);
}
