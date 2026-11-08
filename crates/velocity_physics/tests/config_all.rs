//! Loads the global config and every car's local config from the user's install.
use velocity_physics::c_config::{self, load_global_config_with_fallback};
use velocity_physics::vehicle::vehicle_entity::{load_local_config, LocalConfigInputs, CONFIG_CAR_NAMES};

#[test]
fn global_config_loads() {
    let gc = load_global_config_with_fallback(0).expect("global config");
    assert_eq!(gc.gravity, [0.0, -9.8, 0.0]);
    assert_eq!(gc.timestep, 0.03);
    assert_eq!(gc.nos_fade_in_time, 48);
    assert!(gc.RPM_expression.image.items.len() > 3);
    assert_eq!(gc.g_WheelCamOffsetZ, -2.0);
    // every config parameter consumed: the last one is the last line of Handling.cfg
    println!("max_RPM_change {} RPM_multiplier {} ideal_RPM_to_change_gear_up {}", gc.max_RPM_change, gc.RPM_multiplier, gc.ideal_RPM_to_change_gear_up);
}

#[test]
fn every_car_local_config_loads() {
    for name in CONFIG_CAR_NAMES.iter().chain(["UNKNOWN_CAR"].iter()) {
        let mut inp = LocalConfigInputs::default_spans(name);
        inp.is_player = true;
        let (ok, l) = load_local_config(&inp, "Main.cfg", 0);
        if *name == "C2" {
            // shipped data bug: CarBoxes.cfg has no C2 entry, so Handling.cfg stops at CD_box_size
            // (the original asserts "could not load LOCAL config file" and keeps the partial config)
            assert!(!ok);
            assert_eq!(l.lc.mass, 1080.0);
            assert_eq!(l.lc.number_of_boxes, 2);
            assert_eq!(l.lc.upper_suspension_limit, 0.0);
            continue;
        }
        assert!(ok, "{name}");
        assert!(l.lc.mass > 0.0 && l.lc.engine_power > 0.0, "{name}");
        assert_eq!(l.lc.number_of_skid_sensitivities, 3, "{name}");
    }
}

#[test]
fn dump_240sx() {
    let mut inp = LocalConfigInputs::default_spans("240SX");
    inp.is_player = true;
    let (ok, l) = load_local_config(&inp, "Main.cfg", 0);
    assert!(ok);
    let lc = &l.lc;
    println!("{:#?}", lc);
    assert_eq!(lc.mass, 1721.0);
    assert_eq!(l.config_name, Some("240SX"));
    assert_eq!(lc.number_of_forward_gears, 6);
    assert_eq!(lc.gear_ratios, [3.626, 2.2, 1.541, 1.213, 1.0, 0.767]);
    assert_eq!(lc.driving_wheel, [0, 1, 1, 0]);
    assert_eq!(lc.number_of_boxes, 2);
    assert_eq!(lc.CD_box_size[0], [4.4, 0.6, 1.8]);
    // TOP_SPEED 130 mph * MPH, CONVERSION_TO_REAL_MAX_SPEED 1
    assert_eq!(lc.maximum_forward_speed, 58.1152);
    // engine_power = BHP*HP_ADJUSTMENT*HP*ACCELERATION_MOD = 144*1.25*(1/0.001341)*(0.8+0.2*0)
    println!("engine_power {} speed_limit {}", lc.engine_power, lc.speed_limit);
    let _ = c_config::CFG_DIR;
}
