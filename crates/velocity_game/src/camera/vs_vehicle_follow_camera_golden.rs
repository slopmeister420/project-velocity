//! regression: the follow camera at the default tweak values gives bit-identical output to the port before the
//! "Camera.Globals" / "Camera.TurnRates" tweaks were made live (hash recorded from that build, commit 8009e87).
//! The input is a scripted 20 s drive (24 Hz) through every branch of `update` / `update_follow`: throttle, brake,
//! handbrake, steer, drift, bumps and a jump, fury, zone and one-off, view 1, look back, lag, detached camera,
//! point of interest, camera collision, sliding, impact shake, a static bumper view and the rear mirror.

use bevy::math::{Mat3, Vec3};

use super::vs_vehicle_follow_camera::*;

const DT: f32 = 1.0 / 24.0;
const STEPS: u32 = 480;

struct Hash(u64);
impl Hash {
    fn u(&mut self, v: u32) {
        for b in v.to_le_bytes() {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x100_0000_01B3);
        }
    }
    fn f(&mut self, v: f32) {
        self.u(v.to_bits());
    }
    fn v(&mut self, v: Vec3) {
        self.f(v.x);
        self.f(v.y);
        self.f(v.z);
    }
}

/// the scripted car and pad at update `i` (pos, heading, velocity, target fields)
fn script(i: u32, pos: &mut Vec3, yaw: &mut f32) -> FollowTarget {
    let s = i as f32 * DT;
    let speed = if i < 260 { (s * 4.5).min(55.0) } else if i < 330 { (55.0 - (i - 260) as f32 * 0.7).max(0.5) } else { 20.0 + (s * 0.9).sin() * 6.0 };
    let steer = (s * 0.7).sin() * if (120..200).contains(&i) { 1.0 } else { 0.6 };
    *yaw += steer * 0.9 * DT;
    let heading = Mat3::from_rotation_y(*yaw) * Vec3::Z;
    let slip = if (140..220).contains(&i) { steer * 0.35 } else { steer * 0.05 };
    let mut vel = Mat3::from_rotation_y(*yaw + slip) * Vec3::Z * speed;
    vel.y = (s * 6.0).sin() * 0.8 + if (300..312).contains(&i) { 6.0 - (i - 300) as f32 * 1.2 } else { 0.0 };
    let heading = Vec3::new(heading.x, (s * 0.4).sin() * 0.2, heading.z).normalize();
    *pos += vel * DT;
    let mut t = FollowTarget::new(*pos, heading, vel, vel.length());
    t.bridge_forward = heading;
    t.car_up = Vec3::new((s * 1.3).sin() * 0.05, 1.0, 0.0).normalize();
    t.pad.accelerator = if i < 260 || i >= 340 { 1.0 } else { 0.0 };
    t.pad.brake = if (260..330).contains(&i) { 0.8 } else { 0.0 };
    t.pad.axis = (s * 0.33).sin() * 0.3;
    t.pad.steer = steer;
    t.pad.buttons = if (380..395).contains(&i) { PAD_LOOK_BACK } else { 0 } | if (230..240).contains(&i) { 2 } else { 0 };
    t.fury = if (200..260).contains(&i) { ((i - 200) as f32 / 60.0).min(1.0) } else { 0.0 };
    t.extra_yaw = slip * 0.5;
    t.lag = if (100..130).contains(&i) { 0.5 } else if (440..450).contains(&i) { 0.005 } else { 1.0 };
    t.look_around = if (440..450).contains(&i) { 0.7 } else { 0.0 };
    t.trail_offset = if (60..90).contains(&i) { 0.3 } else { 0.0 };
    t.pitch_offset = if (60..90).contains(&i) { 0.05 } else { 0.0 };
    t.in_zone = (150..250).contains(&i);
    t.one_off = if (170..190).contains(&i) { Some((i - 170) as f32 / 20.0) } else { None };
    if (400..440).contains(&i) {
        let k = (i - 400) as f32 / 20.0;
        t.poi = Some(PoiBlend {
            target: *pos + Vec3::new(20.0, 2.0, 5.0),
            t: if k < 1.0 { k } else { 2.0 - k },
            out: k >= 1.0,
            fov_distances: Some([5.0, 40.0, 0.6]),
        });
    }
    t
}

fn run(step: &mut dyn FnMut(&mut VsVehicleFollowCamera, &FollowTarget, &FollowExtras, &mut bool, &mut dyn FnMut(Vec3, Vec3) -> Option<f32>, &mut dyn FnMut(f32, f32) -> f32) -> FollowOut) -> u64 {
    let chase = CameraView { height: 1.7, look_height: 1.8, dist_a: 9.0, dist_b: 9.0, fov: 75.0, chase: 1.0, ..Default::default() };
    let bumper = CameraView { height: 0.7, look_height: 0.7, dist_a: -2.0, dist_b: 1.5, fov: 60.0, chase: 0.0, unknown_532: 1.0, ..Default::default() };
    let mut cams = [
        VsVehicleFollowCamera { view: chase, player: 0, ..Default::default() },
        VsVehicleFollowCamera { view: chase, player: 1, ..Default::default() },
        VsVehicleFollowCamera { view: bumper, player: 1, view_index: 2, ..Default::default() },
        VsVehicleFollowCamera { view: CameraView { chase: 0.0, unknown_528: 0.0, ..chase }, player: 0, mirror: true, rear: true, ..Default::default() },
    ];
    let mut h = Hash(0xCBF2_9CE4_8422_2325);
    let mut seed = 0x1234_5678u32;
    let (mut pos, mut yaw) = (Vec3::new(10.0, 2.0, -5.0), 0.3f32);
    let mut cut = true;
    for i in 0..STEPS {
        let t = script(i, &mut pos, &mut yaw);
        cams[1].view_index = if (40..120).contains(&i) { 1 } else { 0 };
        let x = FollowExtras {
            sliding: (140..220).contains(&i),
            game_time: i as f32 * DT,
            impact: (350..370).contains(&i).then_some(ImpactShake {
                amount: 1.0,
                base: 0.4,
                start: 350.0 * DT,
                end: 366.0 * DT,
                up: Vec3::Y,
                side: Vec3::X,
            }),
            poi_active: t.poi.is_some(),
            tweaks: CameraTweaks { height: 1.7, look_height: 1.8, dist_a: 9.0, dist_b: 9.0, fov: 75.0 },
        };
        for c in cams.iter_mut() {
            let mut hit = |a: Vec3, b: Vec3| ((200..215).contains(&i) && (b - a).length() > 1.0).then_some(0.75);
            let mut rand = |lo: f32, hi: f32| {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
                lo + (hi - lo) * ((seed >> 8) as f32 / 16_777_216.0)
            };
            let out = step(c, &t, &x, &mut cut, &mut hit, &mut rand);
            h.u(out.impact_amount.map_or(1, |a| a.to_bits()));
            h.u(out.poi_done as u32);
            h.u(cut as u32 | (c.render_cut as u32) << 1 | (c.rear as u32) << 2 | (c.dynamic as u32) << 3);
            c.render_cut = false;
            h.v(c.eye);
            h.v(c.look);
            h.v(c.up);
            h.v(c.right);
            for f in [c.fov_rad, c.fov, c.slope, c.bank, c.kite, c.kite_input, c.speed_pitch, c.pedal, c.bounce, c.bounce_vel, c.zone_time, c.zone_fov] {
                h.f(f);
            }
            let b = c.bridge.unwrap();
            h.v(b.position);
            h.v(b.target);
            h.v(b.up);
            h.f(b.fov);
        }
    }
    h.0
}

#[test]
fn default_globals_match_the_pre_tweak_camera() {
    let g = FollowGlobals::default();
    let mut step = |c: &mut VsVehicleFollowCamera, t: &FollowTarget, x: &FollowExtras, cut: &mut bool, hit: &mut dyn FnMut(Vec3, Vec3) -> Option<f32>, rand: &mut dyn FnMut(f32, f32) -> f32| {
        c.update_follow(DT, &g, t, x, cut, hit, rand)
    };
    assert_eq!(run(&mut step), GOLDEN);
}

/// the hash of the pre-tweak build (8009e87, `k::` constants)
const GOLDEN: u64 = 0xC8D9_80B2_5C0A_F156;

#[test]
fn edited_globals_change_the_camera() {
    // a changed tweak must reach the camera (the regression test above would pass on dead tweaks too)
    let g = FollowGlobals { fov_min: 70.0, ..Default::default() };
    let mut step = |c: &mut VsVehicleFollowCamera, t: &FollowTarget, x: &FollowExtras, cut: &mut bool, hit: &mut dyn FnMut(Vec3, Vec3) -> Option<f32>, rand: &mut dyn FnMut(f32, f32) -> f32| {
        c.update_follow(DT, &g, t, x, cut, hit, rand)
    };
    assert_ne!(run(&mut step), GOLDEN);
}

#[test]
fn zero_width_fov_range_stays_finite() {
    // fovMinOveride stepped up to fovMaxOveride: (fov - min) / 0 is NaN, VsClamp makes it 0 (seen in the debug menu)
    let g = FollowGlobals { fov_min: 80.0, ..Default::default() };
    let mut step = |c: &mut VsVehicleFollowCamera, t: &FollowTarget, x: &FollowExtras, cut: &mut bool, hit: &mut dyn FnMut(Vec3, Vec3) -> Option<f32>, rand: &mut dyn FnMut(f32, f32) -> f32| {
        let o = c.update_follow(DT, &g, t, x, cut, hit, rand);
        assert!(c.eye.is_finite() && c.look.is_finite() && c.fov_rad.is_finite());
        o
    };
    run(&mut step);
}

#[test]
fn defaults_are_the_data_values() {
    // the .data bits from the AddTweaks calls (research/core_tweaks.tsv)
    let g = FollowGlobals::default();
    let bits = [
        (g.yaw_slew_rate, 0x42F00000), (g.max_angle, 0x42B40000), (g.z_rotation, 0),
        (g.bank_speed_min, 0x41700000), (g.bank_speed_max, 0x41F00000), (g.bank_max_angle, 0x41000000),
        (g.bank_nos_factor, 0x3F000000), (g.bank_lag, 0x3D75C28F), (g.fov_min, 0x42700000), (g.fov_max, 0x42A00000),
        (g.fov_speed_min, 0x41700000), (g.fov_speed_max, 0x42200000), (g.fov_rate_up, 0x3E19999A),
        (g.fov_rate_down, 0x3D4CCCCD), (g.fov_boost, 0x41A00000), (g.dist_near_fov, 0x40833333),
        (g.dist_far_fov, 0x40666666), (g.eye_y, 0xBEE66666), (g.kite_max_pitch, 0x40800000),
        (g.kite_recede, 0x3F400000), (g.kite_closer, 0x3F19999A), (g.kite_accel_factor, 0x3F800000),
        (g.kite_brake_factor, 0x3F800000), (g.kite_speed_factor, 0x3F000000), (g.kite_accel_rise, 0x3F333333),
        (g.kite_brake_rise, 0x3F8CCCCD), (g.kite_accel_decay, 0x3F733333), (g.kite_brake_decay, 0x3F7851EC),
        (g.pitch_speed_rate, 0x3D75C28F), (g.pitch_speed_min, 0x40A00000), (g.pitch_speed_max, 0x41200000),
        (g.pivot, 0x3F8CCCCD), (g.far_cam_trail, 0x3F99999A), (g.heading_weight, 0x3F19999A),
        (g.yaw_gain, 0x43480000), (g.yaw_full, 0x41700000), (g.yaw_dead, 0x3F000000), (g.vel_speed_min, 0x40000000),
        (g.vel_speed_max, 0x40C00000), (g.bounce_rate, 0x40400000), (g.bounce_damping, 0x3ECCCCCD),
        (g.bounce_amount, 0x3E99999A), (g.shake_v, 0x3C03126F), (g.shake_h, 0x3A83126F),
        (g.shake_speed_min, 0x41C80000), (g.shake_speed_max, 0x42700000), (g.shake_drift, 0),
        (g.shake_3rd_person, 0x3F000000), (g.zone_duration, 0x3F000000), (g.zone_fov_change, 0),
        (g.zone_pitch_change, 0xC1C80000), (g.zone_trail_change, 0),
    ];
    for (i, (v, b)) in bits.iter().enumerate() {
        assert_eq!(v.to_bits(), *b, "global {i}");
    }
    assert!(g.banking && g.pitch_camera && g.zone_use_trail && g.zone_ramp_curve);
}
