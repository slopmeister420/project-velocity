//! LADown Race02 trailblazer end (core's road height question): every track surface layer under a few points
//! of x 519..600, z -50..-115, top to bottom (repeated downward ray casts against the track collision).
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PhysicsWorld, RAY_NORMAL, RAY_TRACK};

#[test]
fn ladown_trailblazer_end_layers() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let track = TrackWorld::load("LADown").expect("LADown collision");
    let mut w = PhysicsWorld::new(gc, Some(track));
    for &(x, z) in &[(503.5, -53.0), (520.0, -52.5), (547.4, -52.5), (560.0, -80.0), (590.0, -110.0), (575.5, -55.2), (580.0, -56.0), (585.0, -57.0), (590.0, -58.0), (597.6, -63.6)] {
        let mut top = 60.0f32;
        let mut layers = Vec::new();
        while top > -80.0 && layers.len() < 8 {
            let Some(h) = w.ray_cast([x, top, z], [x, -80.0, z], RAY_TRACK | RAY_NORMAL, -1) else { break };
            layers.push((h.point[1], h.normal[1]));
            top = h.point[1] - 0.05;
        }
        eprintln!("({x}, {z}): {layers:?}");
        assert!(!layers.is_empty());
    }
}

/// LACoast's arena.qdt is the one non-compact tree: a ray crossing leaves steps through the node
/// array (`*(qt + 84)`, 0x0019DB00), whose file pointer is stale (frontend's Sprint 01 camera panic)
#[test]
fn lacoast_long_rays_cross_leaves() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let track = TrackWorld::load("LACoast").expect("LACoast collision");
    assert_eq!(track.qt.is_compact, 0);
    let (lo, hi) = (track.qt.min, track.qt.max);
    let mut w = PhysicsWorld::new(gc, Some(track));
    let mut hits = 0;
    for k in 0..16 {
        let t = k as f32 / 15.0;
        let z = lo[2] + (hi[2] - lo[2]) * t;
        let x = lo[0] + (hi[0] - lo[0]) * t;
        let y = lo[1] + (hi[1] - lo[1]) * 0.5;
        hits += w.ray_cast([lo[0], y, z], [hi[0], y, z], RAY_TRACK | RAY_NORMAL, -1).is_some() as i32;
        hits += w.ray_cast([x, y, lo[2]], [x, y, hi[2]], RAY_TRACK | RAY_NORMAL, -1).is_some() as i32;
        hits += w.ray_cast([x, hi[1], z], [x + 40.0, lo[1], z - 40.0], RAY_TRACK | RAY_NORMAL, -1).is_some() as i32;
    }
    assert!(hits > 0);
}
