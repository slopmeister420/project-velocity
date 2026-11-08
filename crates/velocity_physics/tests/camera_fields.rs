//! VE camera fields the chase camera reads: the up offset VE+4392 (SyncModels -> 0x001C2FF0) and the
//! impact shake VE+5008..+5020 (0x001D1610).
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::sim_ai::VehicleControlsRaw;
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

const DT: f32 = 1.0 / 24.0;

/// corner at ~15 m/s; returns the largest |up offset| and the last graphics pose / body position
fn corner(player: bool) -> (f32, ([f32; 3], [f32; 4]), [f32; 3]) {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    let mut inputs = LocalConfigInputs::default_spans("350Z");
    inputs.is_player = player;
    let a = -std::f32::consts::FRAC_PI_4;
    let id = w.spawn(&SpawnParams { inputs, position: [0.0, 0.6, -2500.0], orientation: [0.0, a.sin(), 0.0, a.cos()], slot: -1 });
    w.set_human_control(id, false);
    w.activate(id);
    let mut c = VehicleControlsRaw::default();
    let mut up = 0.0f32;
    for f in 0..24 * 25 {
        let v = w.velocity(id);
        let s = (v[0] * v[0] + v[2] * v[2]).sqrt();
        c.throttle = ((15.0 - s) * 0.5).clamp(0.0, 1.0);
        c.steer = if f > 24 * 12 { 0.3 } else { 0.0 };
        w.set_controls(id, &c);
        w.set_steer_angle(id, c.steer);
        w.step(DT, &[]);
        let u = w.camera_up_offset(id);
        assert!(u.is_finite() && u <= 0.0, "up offset {u}");
        up = up.max(u.abs());
    }
    (up, w.graphics_pose(id), w.position(id))
}

#[test]
fn up_offset_from_the_lean_cheat() {
    // Handling.cfg: AI_lean_reverse_cheat 1, player_lean_reverse_cheat 0, sideways_rotation_shift 0.15
    let (ai, (gp, q), pos) = corner(false);
    assert!(ai > 1e-4 && ai < 0.15, "ai up offset {ai}");
    let d = (0..3).map(|k| (gp[k] - pos[k]).powi(2)).sum::<f32>().sqrt();
    assert!(d < 1.0, "graphics position {gp:?} vs body {pos:?}");
    let n = q.iter().map(|x| x * x).sum::<f32>();
    assert!((n - 1.0).abs() < 1e-3, "quat {q:?}");
    let (pl, _, _) = corner(true);
    assert_eq!(pl, 0.0);
}

#[test]
fn handling_camera_overrides() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let nz = |v: f32| (v != 0.0).then_some(v);
    let want = velocity_physics::vehicle::vehicle_entity::sim::HandlingCameraOverrides {
        far_clip_distance: nz(gc.g_ftFarClipDistance),
        yaw_rate_per_degree: nz(gc.g_yawRatePerDegree),
        yaw_relax_angle: nz(gc.g_yawRelaxAngle),
        yaw_direction_blend: nz(gc.g_yawDirectionBlend),
        wheel_cam_fov: nz(gc.g_WheelCamFOV),
        wheel_cam_offset_x: nz(gc.g_WheelCamOffsetX),
        wheel_cam_offset_z: nz(gc.g_WheelCamOffsetZ),
    };
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    assert_eq!(w.handling_camera_overrides(), Default::default());
    w.spawn(&SpawnParams { inputs: LocalConfigInputs::default_spans("350Z"), position: [0.0, 0.6, 0.0], orientation: [0.0, 0.0, 0.0, 1.0], slot: -1 });
    // shipped Handling.cfg: far clip 0 (kept), yaw 200 / 15 / 0.6
    assert_eq!(want.far_clip_distance, None);
    assert_eq!(want.yaw_rate_per_degree, Some(200.0));
    assert_eq!(w.handling_camera_overrides(), want);
}

#[test]
fn camera_shake_setter() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::flat(3000.0, 14)));
    let id = w.spawn(&SpawnParams { inputs: LocalConfigInputs::default_spans("350Z"), position: [0.0, 0.6, 0.0], orientation: [0.0, 0.0, 0.0, 1.0], slot: -1 });
    assert_eq!(w.camera_shake(id), [0.0; 4]);
    w.set_camera_shake(id, 2.5, 1.5, 10.0);
    assert_eq!(w.camera_shake(id), [2.5, 2.5, 10.0, 11.5]);
}
