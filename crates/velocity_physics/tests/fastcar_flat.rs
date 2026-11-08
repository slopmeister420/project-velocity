//! FastCar core on a flat plane: settle under gravity, then drive the rear wheels.
use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::fastcar::car::{box_inertia, off::*, wheel_inertia, CFastCar};
use velocity_physics::fastcar::rigid_body::{off as rbo, RigidBody};
use velocity_physics::vehicle::vehicle_entity::{load_local_config, LocalConfigInputs};

fn build() -> CFastCar {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut inp = LocalConfigInputs::default_spans("240SX");
    inp.is_player = true;
    let lc = load_local_config(&inp, "Main.cfg", 0).1.lc;
    let mut c = CFastCar::new();
    let ct = box_inertia(lc.mass, lc.MOI_box_size[0], lc.MOI_box_size[1], lc.MOI_box_size[2]);
    let ft = wheel_inertia(lc.front_wheel_mass, lc.front_wheel_radius, lc.front_wheel_width);
    let bt = wheel_inertia(lc.back_wheel_mass, lc.back_wheel_radius, lc.back_wheel_width);
    c.set_parameters_from_spans(
        gc.gravity, lc.span_front_back, lc.span_front, lc.span_back, lc.COM_pos_x, lc.COM_pos_y,
        lc.front_wheel_radius, lc.back_wheel_radius, ct, ft, bt, lc.mass, lc.front_wheel_mass, lc.back_wheel_mass,
    );
    let s = c.default_springs(None);
    for i in 0..4 {
        c.m.set_f(SPRING + 4 * i, s[i] * gc.suspension_stiffnees_multiplier);
    }
    let d = c.critical_dampers();
    for i in 0..4 {
        c.m.set_f(DAMPER + 4 * i, d[i] * gc.suspension_damping_multiplier);
        c.m.set_f(MU + 4 * i, lc.static_friction_longitudinal);
        c.m.set_f(LATERAL_RATIO + 4 * i, lc.lateral_to_longitudinal_friction_ratio);
        c.m.set_f(STATIC_DYNAMIC_RATIO + 4 * i, lc.static_to_dynamic_friction_ratio);
        c.m.set_f(UPPER_LIMIT + 4 * i, lc.upper_suspension_limit);
        c.m.set_f(LOWER_LIMIT + 4 * i, lc.lower_suspension_limit);
    }
    c.m.set_i(WHEEL_DYNAMICS_TYPE, gc.wheel_dynamics_type);
    c.m.set_f(rbo::LINEAR_DAMPING_TAU, gc.car_linear_damping);
    c.m.set_f(rbo::ANGULAR_DAMPING_TAU, gc.car_angular_damping);
    c.m.set_f(rbo::MAX_OMEGA, gc.maximum_angular_velocity_ever);
    c.m.set_f(RELAXATION_STEPS, gc.relaxation_time_in_timesteps);
    c.m.set_f(DYNAMIC_VELOCITY, lc.dynamic_velocity);
    c.m.set_f(TOPPLE_CORRECTION, lc.topple_correction);
    c.m.set_i(SUSP_GEOMETRY_FRONT, gc.suspension_geometry as i32);
    c.m.set_i(SUSP_GEOMETRY_BACK, gc.suspension_geometry as i32);
    c.m.set_f(GROUND_HEIGHT, 0.0);
    c.cd = Some(Box::new(|car: &mut CFastCar| car.flat_ground_contacts()));
    c.m.set_v3(rbo::POSITION, [0.0, 0.6, 0.0]);
    c.update_derived();
    c
}

#[test]
fn settles_and_drives() {
    let mut c = build();
    let h = 0.03f32;
    for _ in 0..100 {
        c.evolve(h, false);
    }
    let p = c.m.v3(rbo::POSITION);
    println!("rest y {:.4} contacts {:?} susp {:?}", p[1], (0..4).map(|i| c.n_contacts(i)).collect::<Vec<_>>(), c.m.v4(SUSP_X));
    assert!(p[1] > 0.0 && p[1] < 1.0 && p[1].is_finite());
    for step in 0..400 {
        for i in [1usize, 2] {
            c.m.set_f(DRIVE_TORQUE + 4 * i, -800.0);
        }
        c.evolve(h, false);
        if step % 50 == 0 {
            let v = c.m.v3(rbo::VELOCITY);
            println!("t {:5.2} pos {:?} vel {:?} wheel omega {:?}", step as f32 * h, c.m.v3(rbo::POSITION), v, c.m.v4(WHEEL_OMEGA));
        }
    }
    let v = c.m.v3(rbo::VELOCITY);
    assert!(v.iter().all(|x| x.is_finite()));
}
