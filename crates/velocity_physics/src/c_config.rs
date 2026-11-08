//! src/Physics/CConfig.cpp - the FastCar configuration structures. `CGlobalConfig` (`::GC`, one
//! instance, `dword_74B618`) and `CLocalConfig` (`VehicleEntity::LC`, one per car, 744 bytes) are
//! read from `Vehicles\cfgs\*.cfg` in a fixed parameter order (the original `CConfig.inl`); every
//! field keeps the name of its config parameter.
#![allow(non_snake_case)]

use crate::vehicle::vs_config_file_read::{ConfigReader, Defines, Expression, ReadError, TextFile};

/// `CGlobalConfig` (0x870 bytes). Field order and offsets as in the original.
#[derive(Clone, Debug, Default)]
pub struct CGlobalConfig {
    pub gravity: [f32; 3], // +0x0
    pub gravity_when_in_air: [f32; 3], // +0xC
    pub full_gravity_in_air_speed: f32, // +0x18
    pub dont_use_RungeKutta_when_in_air: i32, // +0x1C
    pub timestep: f32, // +0x20
    pub number_of_evolves: i32, // +0x24
    pub car_linear_damping: f32, // +0x28
    pub car_linear_damping_when_in_air: f32, // +0x2C
    pub car_angular_damping: f32, // +0x30
    pub car_angular_damping_when_in_air: f32, // +0x34
    pub car_angular_damping_after_landings: f32, // +0x38
    pub dynamic_velocity_multiplier: f32, // +0x3C
    pub relaxation_time_in_timesteps: f32, // +0x40
    pub max_speed_to_tamper_with_force_DF: f32, // +0x44
    pub max_omega_up_for_SF: f32, // +0x48
    pub maximum_angular_velocity_ever: f32, // +0x4C
    pub portion_of_VSA_to_take: f32, // +0x50
    pub max_VSAP: f32, // +0x54
    pub steering_exaggeration_factor: f32, // +0x58
    pub first_tap_max_length: i32, // +0x5C
    pub max_time_between_first_and_second_tap: i32, // +0x60
    pub nos_fade_in_time: i32, // +0x64
    pub nos_fade_out_time: i32, // +0x68
    pub nos_timestep_multiplier: f32, // +0x6C
    pub nos_sensitivities_multiplier: f32, // +0x70
    pub nos_engine_power_multiplier: f32, // +0x74
    pub nos_max_speed_multiplier: f32, // +0x78
    pub nos_capacity: f32, // +0x7C
    pub stop_nos_when_colliding: i32, // +0x80
    pub visual_zone_velocity: f32, // +0x84
    pub zone_fade_in_time: i32, // +0x88
    pub zone_fade_out_time: i32, // +0x8C
    pub zone_sensitivities_multiplier: f32, // +0x90
    pub zone_friction_multiplier: f32, // +0x94
    pub zone_slipstream_length_multiplier: f32, // +0x98
    pub zone_steering_time_multiplier: f32, // +0x9C
    pub extra_zone_topple_correction: f32, // +0xA0
    pub SS_enabeld: i32, // +0xA4
    pub fMinSlipStreamSpeed: f32, // +0xA8
    pub fMaxSlipStreamDist: f32, // +0xAC
    pub fSlipStreamWidth: f32, // +0xB0
    pub fBlendPercent: f32, // +0xB4
    pub SS_necessary_accumulated_effect_for_steering_bias: f32, // +0xB8
    pub SS_necessary_effect_for_steering_bias: f32, // +0xBC
    pub SS_min_distance_to_steer: f32, // +0xC0
    pub SS_distance_portion_to_steer_to: f32, // +0xC4
    pub SS_steer_method: f32, // +0xC8
    pub SS_steering_bias: f32, // +0xCC
    pub SS_steering_bias_when_steering: f32, // +0xD0
    pub SS_friction_limited_SA_multiplier: f32, // +0xD4
    pub SS_active_accumulated_effect_depletion: f32, // +0xD8
    pub SS_passive_accumulated_effect_depletion: f32, // +0xDC
    pub SS_good_distance: f32, // +0xE0
    pub SS_catch_up_time: f32, // +0xE4
    pub SS_max_velocity_increment: f32, // +0xE8
    pub SS_necessary_accumulated_effect: f32, // +0xEC
    pub SS_effect_value_must_drop_to_to_start_acceleration: f32, // +0xF0
    pub SS_fade_in_time: f32, // +0xF4
    pub SS_fade_out_time: f32, // +0xF8
    pub SS_timestep_multiplier: f32, // +0xFC
    pub SS_sensitivities_multiplier: f32, // +0x100
    pub SS_engine_power_multiplier: f32, // +0x104
    pub SS_max_speed_multiplier: f32, // +0x108
    pub SS_max_boost_time: i32, // +0x10C
    pub SS_effect_for_speed_scale: f32, // +0x110
    pub SS_speed_scale: f32, // +0x114
    pub player_to_trafic_mass_ratio: f32, // +0x118
    pub suspension_stiffnees_multiplier: f32, // +0x11C
    pub suspension_damping_multiplier: f32, // +0x120
    pub suspension_limits_through_constraint_handler: i32, // +0x124
    pub wheel_dynamics_type: i32, // +0x128
    pub maximum_reverse_speed: f32, // +0x12C
    pub number_of_reverse_gears: i32, // +0x130
    pub RPM_at_zero_gear_number: f32, // +0x134
    pub ideal_RPM_to_change_gear_up: f32, // +0x138
    pub level: f32, // +0x13C
    pub speed_epsilon: f32, // +0x140
    pub backwards_drive_traction_control: f32, // +0x144
    pub max_RPM_change: i32, // +0x148
    pub max_RPM_change_at_gear_1: i32, // +0x14C
    pub max_RPM_change_at_skid: i32, // +0x150
    pub max_RPM_change_wo_accelerator: i32, // +0x154
    pub min_same_gear_duration: i32, // +0x158
    pub min_same_gear_duration_at_skid: i32, // +0x15C
    pub RPM_multiplier: f32, // +0x160
    pub RPM_skid_multiplier: f32, // +0x164
    pub min_power_advantage_to_change_gear: f32, // +0x168
    pub relative_delta_RPM_to_change_gear: f32, // +0x16C
    pub RPM_decay_factor: f32, // +0x170
    pub pause_between_gears: i32, // +0x174
    pub RPM_expression: Expression, // +0x178
    pub DP_expression: Expression, // +0x17C
    pub min_time_between_backfires: i32, // +0x180
    pub backfire_probability_density: Expression, // +0x184
    pub COM_pos_x_WRT_graphics_pivot: f32, // +0x188
    pub COM_pos_y_WRT_graphics_pivot: f32, // +0x18C
    pub suspension_geometry: i32, // +0x190
    pub player_lean_reverse_cheat: f32, // +0x194
    pub player_lean_reverse_cheat_in_zone: f32, // +0x198
    pub AI_lean_reverse_cheat: f32, // +0x19C
    pub sideways_rotation_shift: f32, // +0x1A0
    pub portion_of_lean_change_to_apply_in_one_frame: f32, // +0x1A4
    pub necessary_number_of_touching_wheels: i32, // +0x1A8
    pub fMinLeanFB: f32, // +0x1AC
    pub fMaxLeanFB: f32, // +0x1B0
    pub fStepPreFrameFB: f32, // +0x1B4
    pub fMaxLeanPerFrameLR: f32, // +0x1B8
    pub fMinLeanLR: f32, // +0x1BC
    pub fMaxLeanLR: f32, // +0x1C0
    pub fMinLRSpeed: f32, // +0x1C4
    pub fMaxLRSpeed: f32, // +0x1C8
    pub FB_roll_exaggeration_time: i32, // +0x1CC
    pub FB_roll_exaggeration_12_factor: f32, // +0x1D0
    pub FB_roll_exaggeration_23_factor: f32, // +0x1D4
    pub skid_exaggeration_enabled: i32, // +0x1D8
    pub skid_exaggeration_factor: Expression, // +0x1DC
    pub target_faked_skid_angle_smooth_tau: f32, // +0x1E0
    pub skid_angle_to_ignore: f32, // +0x1E4
    pub backwards_brake_torque: f32, // +0x1E8
    pub forwards_traction_controlled_braking: f32, // +0x1EC
    pub backwards_traction_controlled_braking: f32, // +0x1F0
    pub speed_threshold: f32, // +0x1F4
    pub quadratic_steer_mapping: f32, // +0x1F8
    pub active_steering_change_time: f32, // +0x1FC
    pub passive_steering_change_time: f32, // +0x200
    pub active_steering_change_time_at_initiated_skid: f32, // +0x204
    pub passive_steering_change_time_at_initiated_skid: f32, // +0x208
    pub min_active_steering_rate: f32, // +0x20C
    pub min_passive_steering_rate: f32, // +0x210
    pub max_active_steering_rate: f32, // +0x214
    pub max_passive_steering_rate: f32, // +0x218
    pub maximum_steering_angle_ever: f32, // +0x21C
    pub steering_response_method: i32, // +0x220
    pub friction_limited_steereing_method: f32, // +0x224
    pub forward_velocity_dependent_steering_constant_B_at_zero_speed: f32, // +0x228
    pub forward_velocity_dependent_steering_constant_C: f32, // +0x22C
    pub backward_velocity_dependent_steering_constant_B_at_zero_speed: f32, // +0x230
    pub backward_velocity_dependent_steering_constant_C: f32, // +0x234
    pub max_face_normal_up_to_reduce_friction: f32, // +0x238
    pub friction_boost_when_going_backwards: f32, // +0x23C
    pub friction_boost_with_disabled_skidding: f32, // +0x240
    pub lateral_friction_boost_when_skidding_without_accelerator: f32, // +0x244
    pub maximum_static_friction_ever_at_low_speed: f32, // +0x248
    pub the_low_speed: f32, // +0x24C
    pub friction_spot_friction_multiplier_high_speed: f32, // +0x250
    pub friction_spot_friction_multiplier_low_speed: f32, // +0x254
    pub friction_spot_high_speed: f32, // +0x258
    pub friction_spot_drag_WN: f32, // +0x25C
    pub friction_spot_radius: f32, // +0x260
    pub friction_spot_omega_up_multiplier: f32, // +0x264
    pub friction_at_countdown: f32, // +0x268
    pub time_for_friction_to_grow: i32, // +0x26C
    pub post_countdown_speed_boost_time_bracket: f32, // +0x270
    pub post_countdown_speed_boost: f32, // +0x274
    pub max_vibration_angle: f32, // +0x278
    pub vibration_angle_increment: f32, // +0x27C
    pub LS_speed_max_speed: f32, // +0x280
    pub LS_speed_boost: f32, // +0x284
    pub LS_speed_boost_duration: f32, // +0x288
    pub necessary_steering_input: f32, // +0x28C
    pub min_dir_dot_for_no_doughnuts: f32, // +0x290
    pub driving_wheels_lateral_friction_boost_at_low_speed: f32, // +0x294
    pub driving_wheels_tangential_friction_for_doughnuts: f32, // +0x298
    pub physycs_vs_play_for_doughnuts: f32, // +0x29C
    pub min_backwards_speed: f32, // +0x2A0
    pub jturn_min_omega_up_to_start: f32, // +0x2A4
    pub jturn_max_omega_up_to_stop: f32, // +0x2A8
    pub backwards_necessary_steering_input: f32, // +0x2AC
    pub steering_wheels_lateral_friction_boost: f32, // +0x2B0
    pub steering_wheels_lateral_friction: f32, // +0x2B4
    pub driving_wheels_tangential_friction_boost: f32, // +0x2B8
    pub driving_wheels_tangential_friction: f32, // +0x2BC
    pub max_skid_angle_to_reduce_the_boost_effect: f32, // +0x2C0
    pub max_surface_velocity_to_guess_from_accelerator_only: f32, // +0x2C4
    pub max_lateral_velocity_for_direction_switch: f32, // +0x2C8
    pub physics_vs_play_reduction_time: f32, // +0x2CC
    pub physics_vs_play_relaxation_time_in_timesteps: f32, // +0x2D0
    pub use_physics_vs_play_only_during_skidding: i32, // +0x2D4
    pub physics_vs_play_min_speed: f32, // +0x2D8
    pub entering_skidding_steering_boost: f32, // +0x2DC
    pub min_steer_input_to_apply_steering_boost: f32, // +0x2E0
    pub max_entering_skidding_time__frames: i32, // +0x2E4
    pub slip_velocity: f32, // +0x2E8
    pub min_forward_velocity_to_kick: f32, // +0x2EC
    pub min_steer_input_to_kick: f32, // +0x2F0
    pub handbrake_disabled_after_kick__frames: i32, // +0x2F4
    pub kick_disabled_after_kick__frames: i32, // +0x2F8
    pub tan_of_min_skid_angle_to_apply_bias: f32, // +0x2FC
    pub normal_handbrake_time: i32, // +0x300
    pub max_steering_input_to_use_normal_handbrake: f32, // +0x304
    pub accumulated_NI_skid_sensitivity_boost: Expression, // +0x308
    pub characteristic_decay_time: f32, // +0x30C
    pub AIS_max_accelerator_release_duration: f32, // +0x310
    pub AIS_min_velocity: f32, // +0x314
    pub AIS_hard_steering_min_value: f32, // +0x318
    pub AIS_hard_accelerator_min_value: f32, // +0x31C
    pub AIS_no_accelerator_max_value: f32, // +0x320
    pub AIS_time_to_hold_handbrake_after_reapplying_accelerator: i32, // +0x324
    pub AIS_time_to_hold_handbrake_after_steer_and_accelerator: i32, // +0x328
    pub AIS_time_to_steer_and_footbrake_for_skid: i32, // +0x32C
    pub AIS_time_to_hold_handbrake_after_steer_and_footbrake: i32, // +0x330
    pub AIS_time_to_steer_only_for_skid: i32, // +0x334
    pub AIS_time_to_hold_handbrake_after_steer_only: i32, // +0x338
    pub AIS_kick_multiplier_steer_and_accelerator: f32, // +0x33C
    pub AIS_force_DF_duration: i32, // +0x340
    pub kick_omega_change_factor: f32, // +0x344
    pub max_omega_up_to_stop: f32, // +0x348
    pub tan_of_min_skid_angle_for_fish_tail_removal: f32, // +0x34C
    pub NO_past_skidding_checks: i32, // +0x350
    pub NO_past_skidding_successes: i32, // +0x354
    pub forward_speed_boost: f32, // +0x358
    pub fish_tail_stop_duration: i32, // +0x35C
    pub portion_of_omega_up_to_stop: f32, // +0x360
    pub dpos_to_stop_aligning: f32, // +0x364
    pub dangle_to_stop_aligning: f32, // +0x368
    pub smooth_align_tau: f32, // +0x36C
    pub min_dpos_to_cancel_aligning: f32, // +0x370
    pub target_align_velocity: f32, // +0x374
    pub max_aligning_time: i32, // +0x378
    pub min_time_between_2_wheels: i32, // +0x37C
    pub min_time_on_2_wheels: i32, // +0x380
    pub max_time_on_2_wheels: i32, // +0x384
    pub max_2_wheel_forward_speed_to_start: f32, // +0x388
    pub min_2_wheel_forward_speed: f32, // +0x38C
    pub max_balancing_angle_to_start: f32, // +0x390
    pub max_balancing_omega_change_start: f32, // +0x394
    pub max_balancing_omega_change_end: f32, // +0x398
    pub max_balancing_omega_change_interpolation_time: i32, // +0x39C
    pub boost_to_steering_on_2_wheels: f32, // +0x3A0
    pub boost_to_engine_power_on_2_wheels: f32, // +0x3A4
    pub max_balancing_omega_change_to_switch_to_2_wheels: f32, // +0x3A8
    pub min_difference_of_balancing_omega: f32, // +0x3AC
    pub balancing_time_1_wheel: f32, // +0x3B0
    pub balancing_time_2_wheels: f32, // +0x3B4
    pub min_up_up_to_disable_chassis_CD: f32, // +0x3B8
    pub min_face_normal_up_to_disable_chassis_CD: f32, // +0x3BC
    pub PL_enabled: i32, // +0x3C0
    pub PL_eigenalizer_factor: f32, // +0x3C4
    pub PL_max_omega_to_scale: f32, // +0x3C8
    pub PL_omega_scale_x: f32, // +0x3CC
    pub PL_omega_scale_y: f32, // +0x3D0
    pub PL_omega_scale_z: f32, // +0x3D4
    pub PL_min_speed_up: f32, // +0x3D8
    pub PL_min_horisontal_speed: f32, // +0x3DC
    pub PL_min_flight_distance: f32, // +0x3E0
    pub PL_min_flight_time: f32, // +0x3E4
    pub PL_min_normal_up_to_accept: f32, // +0x3E8
    pub PL_max_omega_extra_should_be_at_least: f32, // +0x3EC
    pub PL_max_omega_change_const: f32, // +0x3F0
    pub PL_max_omega_change_omega_dependent: f32, // +0x3F4
    pub PL_flight_time_addition: f32, // +0x3F8
    pub PL_min_dot_product_to_reverse_extra_rotation: f32, // +0x3FC
    pub PL_max_rot_angle_multiplier_to_reverse_extra_rotation: f32, // +0x400
    pub PL_max_NO_points: i32, // +0x404
    pub PL_max_delta_height: f32, // +0x408
    pub PL_timestep: f32, // +0x40C
    pub PL_ray_node_timestep_multiplier: i32, // +0x410
    pub PL_increment_omega_once: i32, // +0x414
    pub PL_use_Q_OMEGA_only: i32, // +0x418
    pub PL_max_total_flight_time_multiplier: f32, // +0x41C
    pub PL_roll_landing_disturbance: f32, // +0x420
    pub PL_pitch_landing_disturbance: f32, // +0x424
    pub PL_landing_omega_scale: f32, // +0x428
    pub PL_landing_v_up_scale: f32, // +0x42C
    pub PL_max_landing_omega: f32, // +0x430
    pub PL_min_landing_omega: f32, // +0x434
    pub PL_vertical_rebounce: f32, // +0x438
    pub PL_max_vertical_rebounce_velocity: f32, // +0x43C
    pub PL_landing_inertia_scale: f32, // +0x440
    pub PL_landing_time_precision_to_use_inertia_scale: f32, // +0x444
    pub PL_landing_orientation_disturbance_length: f32, // +0x448
    pub PL_landing_orientation_disturbance_magnitude: f32, // +0x44C
    pub PL_landing_orientation_disturbance_referecne_speed_up: f32, // +0x450
    pub PL_landing_camera_tau_multiplier: f32, // +0x454
    pub PL_landing_camera_limits_multiplier: f32, // +0x458
    pub PL_landing_camera_tau_multiplier_duration: i32, // +0x45C
    pub PL_max_landing_velocity_to_ignore_car_landing_velocity: f32, // +0x460
    pub PL_acceleration_time_portion: f32, // +0x464
    pub PL_extend_up_extent_by: f32, // +0x468
    pub PL_min_omega_to_disable_CD: f32, // +0x46C
    pub PL_min_face_normal_up_to_disable_chassis_CD: f32, // +0x470
    pub PL_portion_of_flight_time_to_disable_CD: f32, // +0x474
    pub PL_min_deviation_angle: f32, // +0x478
    pub PL_jump_cam_probability: f32, // +0x47C
    pub max_NO_iterations_coeffecient: f32, // +0x480
    pub contacts_relaxation_timestep_multiplier: f32, // +0x484
    pub projection_timestep_multiplier: f32, // +0x488
    pub treat_stacking: i32, // +0x48C
    pub handle_contact_pairs: i32, // +0x490
    pub do_collision_resolution_for_contacts: i32, // +0x494
    pub do_error_resolution_for_contacts: i32, // +0x498
    pub diag_scale: f32, // +0x49C
    pub car_car_inertia_scale: f32, // +0x4A0
    pub car_car_inertia_scale_min: f32, // +0x4A4
    pub contact_friction_between_cars: f32, // +0x4A8
    pub contact_restitution_between_cars: f32, // +0x4AC
    pub min_normal_up_for_timestep_subdivision: f32, // +0x4B0
    pub min_approaching_velocity_for_timestep_subdivision: f32, // +0x4B4
    pub number_of_binary_divisions: i32, // +0x4B8
    pub size_up_extension_for_car_car_CD: f32, // +0x4BC
    pub max_normal_not_forward_to_move_point: f32, // +0x4C0
    pub max_relp_side: f32, // +0x4C4
    pub relp_to_leave: f32, // +0x4C8
    pub min_sideways_relp_for_traffic: f32, // +0x4CC
    pub max_relp_up_for_player: f32, // +0x4D0
    pub sideways_collision_exaggeration: f32, // +0x4D4
    pub front_front_sideways_kick_factor: f32, // +0x4D8
    pub front_back_sideways_kick_factor: f32, // +0x4DC
    pub front_front_sideways_omega_kick_factor: f32, // +0x4E0
    pub front_back_sideways_omega_kick_factor: f32, // +0x4E4
    pub max_sideways_kick: f32, // +0x4E8
    pub max_omega_kick: f32, // +0x4EC
    pub min_approaching_velocity_for_kick: f32, // +0x4F0
    pub explicit_steer_angle: f32, // +0x4F4
    pub explicit_steer_angle_front_back_low_speed: f32, // +0x4F8
    pub explicit_steer_time: i32, // +0x4FC
    pub front_front_explicit_steer_delay_time: i32, // +0x500
    pub front_back_explicit_steer_delay_time: i32, // +0x504
    pub front_back_low_speed_explicit_steer_delay_time: i32, // +0x508
    pub trafic_low_static_friction_after_collision: f32, // +0x50C
    pub trafic_low_static_friction_after_collision_duration: i32, // +0x510
    pub car_ground_inertia_scale: f32, // +0x514
    pub car_ground_inertia_scale_at_initiated_skid: f32, // +0x518
    pub car_ground_inertia_scale_min: f32, // +0x51C
    pub max_speed_to_handle_contact_pairs: f32, // +0x520
    pub ground_contact_friction_with_horizontal_features: f32, // +0x524
    pub ground_contact_friction_with_vertical_features: f32, // +0x528
    pub ground_contact_restitution_with_horizontal_features: f32, // +0x52C
    pub ground_contact_restitution_with_vertical_features: f32, // +0x530
    pub ground_filer_out_vertical_contacts_when_driving_on_all_wheels: i32, // +0x534
    pub lift_box0_center_when_driving_on_all_wheels_by: f32, // +0x538
    pub moved_contact_torque_comparison_factor: f32, // +0x53C
    pub minimum_contact_velocity_to_filter: f32, // +0x540
    pub compare_coeffecient: f32, // +0x544
    pub additional_penetration_allowance: f32, // +0x548
    pub low_speed_max_backface_angle: f32, // +0x54C
    pub try_to_improve_car_bottom_CD: i32, // +0x550
    pub max_normal_sideways_projection: f32, // +0x554
    pub max_normal_forward_projection: f32, // +0x558
    pub higher_restitution_min_flight_time: i32, // +0x55C
    pub higher_restitution_min_normal_up: f32, // +0x560
    pub higher_restitution: f32, // +0x564
    pub wl_max_cos_of_normal_up: f32, // +0x568
    pub wl_min_of_cos_of_normal_car_forward: f32, // +0x56C
    pub wl_necessary_speed_change: f32, // +0x570
    pub wl_max_speed_change: f32, // +0x574
    pub wl_delta_omega_car_z: f32, // +0x578
    pub wl_car_car_delta_omega_car_z_for_player: f32, // +0x57C
    pub CS_min_car_speed: f32, // +0x580
    pub CS_min_delta_speed: f32, // +0x584
    pub CS_acceptable_shake_distance: f32, // +0x588
    pub CS_tau: f32, // +0x58C
    pub CS_tau_back: f32, // +0x590
    pub CS_duration: i32, // +0x594
    pub CS_criticality: f32, // +0x598
    pub CS_motion_scale: f32, // +0x59C
    pub CS_sideways_motion_scale: f32, // +0x5A0
    pub CS_sideways_tau_scale: f32, // +0x5A4
    pub CS_car_car_motion_scale: f32, // +0x5A8
    pub CS_car_car_sideways_motion_scale: f32, // +0x5AC
    pub CS_camera_diff_to_stop: f32, // +0x5B0
    pub CVI_inertia_reference_frame: f32, // +0x5B4
    pub CVI_tau_car_up: f32, // +0x5B8
    pub CVI_tau_car_down: f32, // +0x5BC
    pub CVI_motion_scale_car_up: f32, // +0x5C0
    pub CVI_motion_scale_car_down: f32, // +0x5C4
    pub CVI_max_motion_car_up: f32, // +0x5C8
    pub CVI_max_motion_car_down: f32, // +0x5CC
    pub CVI_motion_stop_decay_const: f32, // +0x5D0
    pub CSM_min_speed: f32, // +0x5D4
    pub CSM_assumed_camera_distance: f32, // +0x5D8
    pub CSM_camera_motion_angle: f32, // +0x5DC
    pub CSM_skid_angle_conversion_factor: f32, // +0x5E0
    pub CSM_omega_conversion_factor: f32, // +0x5E4
    pub CSM_tau_no_steer: f32, // +0x5E8
    pub CSM_tau_some_steer_input: f32, // +0x5EC
    pub CSM_extra_yaw_conversion: f32, // +0x5F0
    pub detached_camera_enabled: f32, // +0x5F4
    pub time_to_move_out_the_camera_after_detachment: i32, // +0x5F8
    pub time_to_move_out_the_camera_after_detachment_after_collision: i32, // +0x5FC
    pub total_trail_increase: f32, // +0x600
    pub total_pitch_increase: f32, // +0x604
    pub trail_increase_in_a_frame: f32, // +0x608
    pub time_to_move_in_the_camera_after_attachment: i32, // +0x60C
    pub trail_decrease_in_a_frame: f32, // +0x610
    pub min_omega_mag_to_detach_on_tumbling: f32, // +0x614
    pub max_omega_to_attach_camera_after_jturn: f32, // +0x618
    pub camera_attachment_factor_after_jturn: f32, // +0x61C
    pub max_omega_to_attach_camera_after_doughnut: f32, // +0x620
    pub camera_attachment_factor_after_doughnut: f32, // +0x624
    pub idle_input_realignment_time: i32, // +0x628
    pub alignement_duration: i32, // +0x62C
    pub camera_trail_smoothout_tau: f32, // +0x630
    pub camera_factor_smoothout_tau: f32, // +0x634
    pub car_barrier_inertia_scale: f32, // +0x638
    pub car_barrier_inertia_scale_at_initiated_skid: f32, // +0x63C
    pub car_barrier_inertia_scale_min: f32, // +0x640
    pub barrier_contact_friction_with_horizontal_features: f32, // +0x644
    pub barrier_contact_friction_with_vertical_features: f32, // +0x648
    pub barrier_contact_restitution_with_horizontal_features: f32, // +0x64C
    pub barrier_contact_restitution_with_vertical_features: f32, // +0x650
    pub barrier_contacts_relaxation_timestep_multiplier: f32, // +0x654
    pub barrier_send_collision_message: i32, // +0x658
    pub barrier_filer_out_vertical_contacts_when_driving_on_all_wheels: i32, // +0x65C
    pub max_stastic_object_radius: f32, // +0x660
    pub car_static_object_inertia_scale: f32, // +0x664
    pub car_static_object_inertia_scale_at_initiated_skid: f32, // +0x668
    pub car_static_object_inertia_scale_min: f32, // +0x66C
    pub static_object_contact_friction_with_horizontal_features: f32, // +0x670
    pub static_object_contact_friction_with_vertical_features: f32, // +0x674
    pub static_object_contact_restitution_with_horizontal_features: f32, // +0x678
    pub static_object_contact_restitution_with_vertical_features: f32, // +0x67C
    pub static_object_filter_out_vertical_contacts_when_driving_on_all_wheels: i32, // +0x680
    pub TT_min_car_velocity_for_through_tree_transition: f32, // +0x684
    pub TT_max_car_velocity_for_through_tree_transition: f32, // +0x688
    pub TT_max_capsule_radius: f32, // +0x68C
    pub TT_min_capsule_halflength: f32, // +0x690
    pub TT_forward_extent_extension_min: f32, // +0x694
    pub TT_forward_extent_extension_max: f32, // +0x698
    pub TT_car_static_object_inertia_scale: f32, // +0x69C
    pub TT_car_static_object_inertia_scale_min: f32, // +0x6A0
    pub car_static_objects_max_normal_car_x_for_timestep_subdivision: f32, // +0x6A4
    pub car_static_objects_min_approaching_velocity_for_timestep_subdivision: f32, // +0x6A8
    pub car_static_objects_number_of_binary_divisions: i32, // +0x6AC
    pub min_normal_up_to_filter_out: f32, // +0x6B0
    pub car_object_inertia_scale: f32, // +0x6B4
    pub car_object_inertia_scale_at_initiated_skid: f32, // +0x6B8
    pub car_object_inertia_scale_min: f32, // +0x6BC
    pub car_object_friction: f32, // +0x6C0
    pub car_object_restitution: f32, // +0x6C4
    pub drag_scale: f32, // +0x6C8
    pub drag_comparison_factor: f32, // +0x6CC
    pub neighbor_objects_drag_scale: f32, // +0x6D0
    pub neighbor_objects_radnom_omega_addition: f32, // +0x6D4
    pub neighbor_objects_car_reference_velocity: f32, // +0x6D8
    pub peice_linear_velocity_drag_scale: f32, // +0x6DC
    pub peice_radnom_omega_addition: f32, // +0x6E0
    pub upper_limit_on_collision_speed: f32, // +0x6E4
    pub size_up_extension_for_car_object_CD: f32, // +0x6E8
    pub lift_box0_by_for_car_object_CD: f32, // +0x6EC
    pub NO_car_boxes_to_collide_with_objects: i32, // +0x6F0
    pub car_objects_max_normal_car_x_for_timestep_subdivision: f32, // +0x6F4
    pub car_objects_min_approaching_velocity_for_timestep_subdivision: f32, // +0x6F8
    pub car_objects_number_of_binary_divisions: i32, // +0x6FC
    pub object_ground_max_NO_iterations_coeffecient: f32, // +0x700
    pub object_ground_friction: f32, // +0x704
    pub object_ground_restitution: f32, // +0x708
    pub object_ground_inertia_scale: f32, // +0x70C
    pub length_of_CD_rays: f32, // +0x710
    pub min_time_alive: f32, // +0x714
    pub max_time_alive: f32, // +0x718
    pub max_position_change_to_put_to_sleep: f32, // +0x71C
    pub max_omega_to_put_to_sleep: f32, // +0x720
    pub min_contact_normal_up_to_put_to_sleep: f32, // +0x724
    pub max_angle_change_to_put_to_sleep: f32, // +0x728
    pub objects_mass_multiplier: f32, // +0x72C
    pub spherical_inertia_threshold: f32, // +0x730
    pub objects_min_extent: f32, // +0x734
    pub objects_gravity: [f32; 3], // +0x738
    pub objects_gravity_in_air: [f32; 3], // +0x744
    pub objects_linear_damping: f32, // +0x750
    pub objects_linear_damping_increase_time: f32, // +0x754
    pub objects_angular_damping: f32, // +0x758
    pub objects_angular_damping_increase_time: f32, // +0x75C
    pub objects_handle_contact_pairs: i32, // +0x760
    pub min_normal_up_to_collide_triangle_with_wheels: f32, // +0x764
    pub wheel_CD_with_raycasts: i32, // +0x768
    pub up_radius_multiplier: f32, // +0x76C
    pub down_radius_multiplier: f32, // +0x770
    pub max_penetration_change_per_step: f32, // +0x774
    pub max_normal_change_per_step: f32, // +0x778
    pub min_time_without_wheel_contact_to_match_wheel_rotation_to_car_speed: i32, // +0x77C
    pub time_to_reduce_thrust: i32, // +0x780
    pub tan_of_critical_angle: f32, // +0x784
    pub bump_map_spacial_size: f32, // +0x788
    pub blast_radius: f32, // +0x78C
    pub blast_velocity_increment: f32, // +0x790
    pub blast_angular_velocity_increment: f32, // +0x794
    pub blast_centre_up_shift: f32, // +0x798
    pub cos_of_min_disable_skid_marks_angle: f32, // +0x79C
    pub camera_shift_forward: f32, // +0x7A0
    pub draw_quad_tree_output: i32, // +0x7A4
    pub draw_quad_tree_cell: i32, // +0x7A8
    pub draw_triangles_intersecting_the_wheel: i32, // +0x7AC
    pub draw_static_object_triangles: i32, // +0x7B0
    pub draw_static_object_triangles_intersecting_the_wheel: i32, // +0x7B4
    pub draw_chassis_CD_boxes: i32, // +0x7B8
    pub draw_debug_camera_chassis_CD_boxes: f32, // +0x7BC
    pub draw_chassis_ground_CD_boxes: i32, // +0x7C0
    pub draw_chassis_car_car_CD_boxes: i32, // +0x7C4
    pub draw_chassis_CD_contacts: f32, // +0x7C8
    pub draw_objects_CD_contacts: f32, // +0x7CC
    pub draw_wheel_CD_contacts: f32, // +0x7D0
    pub draw_wheel_CD_boxes: i32, // +0x7D4
    pub draw_colliding_boxes: i32, // +0x7D8
    pub draw_colliding_boxes_contacts: f32, // +0x7DC
    pub draw_alive_objects: i32, // +0x7E0
    pub draw_proximity_objects: f32, // +0x7E4
    pub draw_proximity_static_objects: f32, // +0x7E8
    pub draw_wheel_forces: f32, // +0x7EC
    pub draw_PL_trajectory: i32, // +0x7F0
    pub draw_PL_rays: i32, // +0x7F4
    pub draw_PL_landing_spot: f32, // +0x7F8
    pub draw_PL_Q_hit: i32, // +0x7FC
    pub draw_PL_Q_sought: i32, // +0x800
    pub draw_flight_K: f32, // +0x804
    pub draw_departure_omega: f32, // +0x808
    pub draw_extra_omega: f32, // +0x80C
    pub draw_current_omega: f32, // +0x810
    pub draw_visual_steering_direction: i32, // +0x814
    pub chassis_CD_enabled: i32, // +0x818
    pub resolve_collisions: i32, // +0x81C
    pub do_CD: i32, // +0x820
    pub barrier_CD_enabled: i32, // +0x824
    pub car_object_CD_enabled: i32, // +0x828
    pub static_object_CD_enabled: i32, // +0x82C
    pub wheel_CD_enabled: i32, // +0x830
    pub tty_output_steering_angle: i32, // +0x834
    pub tty_output_skidding_forces: i32, // +0x838
    pub tty_output_skid_start_stop: i32, // +0x83C
    pub tty_output_going_forwards_backwards: i32, // +0x840
    pub tty_output_anti_fish_tailing_work: i32, // +0x844
    pub tty_output_pl_data: i32, // +0x848
    pub tty_output_ss_data: i32, // +0x84C
    pub tty_output_2wheels: i32, // +0x850
    pub g_ftFarClipDistance: f32, // +0x854
    pub g_yawRatePerDegree: f32, // +0x858
    pub g_yawRelaxAngle: f32, // +0x85C
    pub g_yawDirectionBlend: f32, // +0x860
    pub g_WheelCamFOV: f32, // +0x864
    pub g_WheelCamOffsetX: f32, // +0x868
    pub g_WheelCamOffsetZ: f32, // +0x86C
    /// set after `PL_omega_scale_z`: 0 when all three scales are 1, else 2
    pub PL_omega_scale_mode: f32, // +0x870

}

/// result of a `Load`: `Stopped` when the file only carried defines and ended with `stop_loading 1`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadResult {
    Stopped,
    Loaded,
}

impl CGlobalConfig {
    /// CConfig.cpp 0x0018F870 (CGlobalConfig::Load)
    pub fn load(&mut self, path: &str, defines: &mut Defines) -> Result<LoadResult, ReadError> {
        let file = TextFile::open(path).ok_or_else(|| ReadError { name: path.into(), line: String::new() })?;
        let mut r = ConfigReader::new(file, defines);
        self.load_from(&mut r)
    }

    pub fn load_from(&mut self, r: &mut ConfigReader) -> Result<LoadResult, ReadError> {
        let mut stop = 0i32;
        r.read_int(&mut stop, "stop_loading")?;
        if stop != 0 {
            return Ok(LoadResult::Stopped);
        }
        r.read_reals(&mut self.gravity, "gravity")?;
        r.read_reals(&mut self.gravity_when_in_air, "gravity_when_in_air")?;
        r.read_real(&mut self.full_gravity_in_air_speed, "full_gravity_in_air_speed")?;
        r.read_int(&mut self.dont_use_RungeKutta_when_in_air, "dont_use_RungeKutta_when_in_air")?;
        r.read_real(&mut self.timestep, "timestep")?;
        r.read_int(&mut self.number_of_evolves, "number_of_evolves")?;
        r.read_real(&mut self.car_linear_damping, "car_linear_damping")?;
        r.read_real(&mut self.car_linear_damping_when_in_air, "car_linear_damping_when_in_air")?;
        r.read_real(&mut self.car_angular_damping, "car_angular_damping")?;
        r.read_real(&mut self.car_angular_damping_when_in_air, "car_angular_damping_when_in_air")?;
        r.read_real(&mut self.car_angular_damping_after_landings, "car_angular_damping_after_landings")?;
        r.read_real(&mut self.dynamic_velocity_multiplier, "dynamic_velocity_multiplier")?;
        r.read_real(&mut self.relaxation_time_in_timesteps, "relaxation_time_in_timesteps")?;
        r.read_real(&mut self.max_speed_to_tamper_with_force_DF, "max_speed_to_tamper_with_force_DF")?;
        r.read_real(&mut self.max_omega_up_for_SF, "max_omega_up_for_SF")?;
        r.read_real(&mut self.maximum_angular_velocity_ever, "maximum_angular_velocity_ever")?;
        r.read_real(&mut self.portion_of_VSA_to_take, "portion_of_VSA_to_take")?;
        r.read_real(&mut self.max_VSAP, "max_VSAP")?;
        r.read_real(&mut self.steering_exaggeration_factor, "steering_exaggeration_factor")?;
        r.read_int(&mut self.first_tap_max_length, "first_tap_max_length")?;
        r.read_int(&mut self.max_time_between_first_and_second_tap, "max_time_between_first_and_second_tap")?;
        r.read_int(&mut self.nos_fade_in_time, "nos_fade_in_time")?;
        r.read_int(&mut self.nos_fade_out_time, "nos_fade_out_time")?;
        r.read_real(&mut self.nos_timestep_multiplier, "nos_timestep_multiplier")?;
        r.read_real(&mut self.nos_sensitivities_multiplier, "nos_sensitivities_multiplier")?;
        r.read_real(&mut self.nos_engine_power_multiplier, "nos_engine_power_multiplier")?;
        r.read_real(&mut self.nos_max_speed_multiplier, "nos_max_speed_multiplier")?;
        r.read_real(&mut self.nos_capacity, "nos_capacity")?;
        r.read_int(&mut self.stop_nos_when_colliding, "stop_nos_when_colliding")?;
        r.read_real(&mut self.visual_zone_velocity, "visual_zone_velocity")?;
        r.read_int(&mut self.zone_fade_in_time, "zone_fade_in_time")?;
        r.read_int(&mut self.zone_fade_out_time, "zone_fade_out_time")?;
        r.read_real(&mut self.zone_sensitivities_multiplier, "zone_sensitivities_multiplier")?;
        r.read_real(&mut self.zone_friction_multiplier, "zone_friction_multiplier")?;
        r.read_real(&mut self.zone_slipstream_length_multiplier, "zone_slipstream_length_multiplier")?;
        r.read_real(&mut self.zone_steering_time_multiplier, "zone_steering_time_multiplier")?;
        r.read_real(&mut self.extra_zone_topple_correction, "extra_zone_topple_correction")?;
        r.read_int(&mut self.SS_enabeld, "SS_enabeld")?;
        r.read_real(&mut self.fMinSlipStreamSpeed, "fMinSlipStreamSpeed")?;
        r.read_real(&mut self.fMaxSlipStreamDist, "fMaxSlipStreamDist")?;
        r.read_real(&mut self.fSlipStreamWidth, "fSlipStreamWidth")?;
        r.read_real(&mut self.fBlendPercent, "fBlendPercent")?;
        r.read_real(&mut self.SS_necessary_accumulated_effect_for_steering_bias, "SS_necessary_accumulated_effect_for_steering_bias")?;
        r.read_real(&mut self.SS_necessary_effect_for_steering_bias, "SS_necessary_effect_for_steering_bias")?;
        r.read_real(&mut self.SS_min_distance_to_steer, "SS_min_distance_to_steer")?;
        r.read_real(&mut self.SS_distance_portion_to_steer_to, "SS_distance_portion_to_steer_to")?;
        r.read_real(&mut self.SS_steer_method, "SS_steer_method")?;
        r.read_real(&mut self.SS_steering_bias, "SS_steering_bias")?;
        r.read_real(&mut self.SS_steering_bias_when_steering, "SS_steering_bias_when_steering")?;
        r.read_real(&mut self.SS_friction_limited_SA_multiplier, "SS_friction_limited_SA_multiplier")?;
        r.read_real(&mut self.SS_active_accumulated_effect_depletion, "SS_active_accumulated_effect_depletion")?;
        r.read_real(&mut self.SS_passive_accumulated_effect_depletion, "SS_passive_accumulated_effect_depletion")?;
        r.read_real(&mut self.SS_good_distance, "SS_good_distance")?;
        r.read_real(&mut self.SS_catch_up_time, "SS_catch_up_time")?;
        r.read_real(&mut self.SS_max_velocity_increment, "SS_max_velocity_increment")?;
        r.read_real(&mut self.SS_necessary_accumulated_effect, "SS_necessary_accumulated_effect")?;
        r.read_real(&mut self.SS_effect_value_must_drop_to_to_start_acceleration, "SS_effect_value_must_drop_to_to_start_acceleration")?;
        r.read_real(&mut self.SS_fade_in_time, "SS_fade_in_time")?;
        r.read_real(&mut self.SS_fade_out_time, "SS_fade_out_time")?;
        r.read_real(&mut self.SS_timestep_multiplier, "SS_timestep_multiplier")?;
        r.read_real(&mut self.SS_sensitivities_multiplier, "SS_sensitivities_multiplier")?;
        r.read_real(&mut self.SS_engine_power_multiplier, "SS_engine_power_multiplier")?;
        r.read_real(&mut self.SS_max_speed_multiplier, "SS_max_speed_multiplier")?;
        r.read_int(&mut self.SS_max_boost_time, "SS_max_boost_time")?;
        r.read_real(&mut self.SS_effect_for_speed_scale, "SS_effect_for_speed_scale")?;
        r.read_real(&mut self.SS_speed_scale, "SS_speed_scale")?;
        r.read_real(&mut self.player_to_trafic_mass_ratio, "player_to_trafic_mass_ratio")?;
        r.read_real(&mut self.suspension_stiffnees_multiplier, "suspension_stiffnees_multiplier")?;
        r.read_real(&mut self.suspension_damping_multiplier, "suspension_damping_multiplier")?;
        r.read_int(&mut self.suspension_limits_through_constraint_handler, "suspension_limits_through_constraint_handler")?;
        r.read_int(&mut self.wheel_dynamics_type, "wheel_dynamics_type")?;
        r.read_real(&mut self.maximum_reverse_speed, "maximum_reverse_speed")?;
        r.read_int(&mut self.number_of_reverse_gears, "number_of_reverse_gears")?;
        r.read_real(&mut self.RPM_at_zero_gear_number, "RPM_at_zero_gear_number")?;
        r.read_real(&mut self.ideal_RPM_to_change_gear_up, "ideal_RPM_to_change_gear_up")?;
        r.read_real(&mut self.level, "level")?;
        r.read_real(&mut self.speed_epsilon, "speed_epsilon")?;
        r.read_real(&mut self.backwards_drive_traction_control, "backwards_drive_traction_control")?;
        r.read_int(&mut self.max_RPM_change, "max_RPM_change")?;
        r.read_int(&mut self.max_RPM_change_at_gear_1, "max_RPM_change_at_gear_1")?;
        r.read_int(&mut self.max_RPM_change_at_skid, "max_RPM_change_at_skid")?;
        r.read_int(&mut self.max_RPM_change_wo_accelerator, "max_RPM_change_wo_accelerator")?;
        r.read_int(&mut self.min_same_gear_duration, "min_same_gear_duration")?;
        r.read_int(&mut self.min_same_gear_duration_at_skid, "min_same_gear_duration_at_skid")?;
        r.read_real(&mut self.RPM_multiplier, "RPM_multiplier")?;
        r.read_real(&mut self.RPM_skid_multiplier, "RPM_skid_multiplier")?;
        r.read_real(&mut self.min_power_advantage_to_change_gear, "min_power_advantage_to_change_gear")?;
        r.read_real(&mut self.relative_delta_RPM_to_change_gear, "relative_delta_RPM_to_change_gear")?;
        r.read_real(&mut self.RPM_decay_factor, "RPM_decay_factor")?;
        r.read_int(&mut self.pause_between_gears, "pause_between_gears")?;
        self.RPM_expression = r.read_expression("RPM_expression", "forwards time_since_last_gear_change current_gear revs_previous")?;
        self.DP_expression = r.read_expression("DP_expression", "forwards time_since_last_gear_change current_gear revs_previous")?;
        r.read_int(&mut self.min_time_between_backfires, "min_time_between_backfires")?;
        self.backfire_probability_density = r.read_expression("backfire_probability_density", "time_since_last_gear_change")?;
        r.read_real(&mut self.COM_pos_x_WRT_graphics_pivot, "COM_pos_x_WRT_graphics_pivot")?;
        r.read_real(&mut self.COM_pos_y_WRT_graphics_pivot, "COM_pos_y_WRT_graphics_pivot")?;
        r.read_int(&mut self.suspension_geometry, "suspension_geometry")?;
        r.read_real(&mut self.player_lean_reverse_cheat, "player_lean_reverse_cheat")?;
        r.read_real(&mut self.player_lean_reverse_cheat_in_zone, "player_lean_reverse_cheat_in_zone")?;
        r.read_real(&mut self.AI_lean_reverse_cheat, "AI_lean_reverse_cheat")?;
        r.read_real(&mut self.sideways_rotation_shift, "sideways_rotation_shift")?;
        r.read_real(&mut self.portion_of_lean_change_to_apply_in_one_frame, "portion_of_lean_change_to_apply_in_one_frame")?;
        r.read_int(&mut self.necessary_number_of_touching_wheels, "necessary_number_of_touching_wheels")?;
        r.read_real(&mut self.fMinLeanFB, "fMinLeanFB")?;
        r.read_real(&mut self.fMaxLeanFB, "fMaxLeanFB")?;
        r.read_real(&mut self.fStepPreFrameFB, "fStepPreFrameFB")?;
        r.read_real(&mut self.fMaxLeanPerFrameLR, "fMaxLeanPerFrameLR")?;
        r.read_real(&mut self.fMinLeanLR, "fMinLeanLR")?;
        r.read_real(&mut self.fMaxLeanLR, "fMaxLeanLR")?;
        r.read_real(&mut self.fMinLRSpeed, "fMinLRSpeed")?;
        r.read_real(&mut self.fMaxLRSpeed, "fMaxLRSpeed")?;
        r.read_int(&mut self.FB_roll_exaggeration_time, "FB_roll_exaggeration_time")?;
        r.read_real(&mut self.FB_roll_exaggeration_12_factor, "FB_roll_exaggeration_12_factor")?;
        r.read_real(&mut self.FB_roll_exaggeration_23_factor, "FB_roll_exaggeration_23_factor")?;
        r.read_int(&mut self.skid_exaggeration_enabled, "skid_exaggeration_enabled")?;
        self.skid_exaggeration_factor = r.read_expression("skid_exaggeration_factor", "skid_angle forward_speed lateral_speed")?;
        r.read_real(&mut self.target_faked_skid_angle_smooth_tau, "target_faked_skid_angle_smooth_tau")?;
        r.read_real(&mut self.skid_angle_to_ignore, "skid_angle_to_ignore")?;
        r.read_real(&mut self.backwards_brake_torque, "backwards_brake_torque")?;
        r.read_real(&mut self.forwards_traction_controlled_braking, "forwards_traction_controlled_braking")?;
        r.read_real(&mut self.backwards_traction_controlled_braking, "backwards_traction_controlled_braking")?;
        r.read_real(&mut self.speed_threshold, "speed_threshold")?;
        r.read_real(&mut self.quadratic_steer_mapping, "quadratic_steer_mapping")?;
        r.read_real(&mut self.active_steering_change_time, "active_steering_change_time")?;
        r.read_real(&mut self.passive_steering_change_time, "passive_steering_change_time")?;
        r.read_real(&mut self.active_steering_change_time_at_initiated_skid, "active_steering_change_time_at_initiated_skid")?;
        r.read_real(&mut self.passive_steering_change_time_at_initiated_skid, "passive_steering_change_time_at_initiated_skid")?;
        r.read_real(&mut self.min_active_steering_rate, "min_active_steering_rate")?;
        r.read_real(&mut self.min_passive_steering_rate, "min_passive_steering_rate")?;
        r.read_real(&mut self.max_active_steering_rate, "max_active_steering_rate")?;
        r.read_real(&mut self.max_passive_steering_rate, "max_passive_steering_rate")?;
        r.read_real(&mut self.maximum_steering_angle_ever, "maximum_steering_angle_ever")?;
        r.read_int(&mut self.steering_response_method, "steering_response_method")?;
        r.read_real(&mut self.friction_limited_steereing_method, "friction_limited_steereing_method")?;
        r.read_real(&mut self.forward_velocity_dependent_steering_constant_B_at_zero_speed, "forward_velocity_dependent_steering_constant_B_at_zero_speed")?;
        r.read_real(&mut self.forward_velocity_dependent_steering_constant_C, "forward_velocity_dependent_steering_constant_C")?;
        r.read_real(&mut self.backward_velocity_dependent_steering_constant_B_at_zero_speed, "backward_velocity_dependent_steering_constant_B_at_zero_speed")?;
        r.read_real(&mut self.backward_velocity_dependent_steering_constant_C, "backward_velocity_dependent_steering_constant_C")?;
        r.read_real(&mut self.max_face_normal_up_to_reduce_friction, "max_face_normal_up_to_reduce_friction")?;
        r.read_real(&mut self.friction_boost_when_going_backwards, "friction_boost_when_going_backwards")?;
        r.read_real(&mut self.friction_boost_with_disabled_skidding, "friction_boost_with_disabled_skidding")?;
        r.read_real(&mut self.lateral_friction_boost_when_skidding_without_accelerator, "lateral_friction_boost_when_skidding_without_accelerator")?;
        r.read_real(&mut self.maximum_static_friction_ever_at_low_speed, "maximum_static_friction_ever_at_low_speed")?;
        r.read_real(&mut self.the_low_speed, "the_low_speed")?;
        r.read_real(&mut self.friction_spot_friction_multiplier_high_speed, "friction_spot_friction_multiplier_high_speed")?;
        r.read_real(&mut self.friction_spot_friction_multiplier_low_speed, "friction_spot_friction_multiplier_low_speed")?;
        r.read_real(&mut self.friction_spot_high_speed, "friction_spot_high_speed")?;
        r.read_real(&mut self.friction_spot_drag_WN, "friction_spot_drag_WN")?;
        r.read_real(&mut self.friction_spot_radius, "friction_spot_radius")?;
        r.read_real(&mut self.friction_spot_omega_up_multiplier, "friction_spot_omega_up_multiplier")?;
        r.read_real(&mut self.friction_at_countdown, "friction_at_countdown")?;
        r.read_int(&mut self.time_for_friction_to_grow, "time_for_friction_to_grow")?;
        r.read_real(&mut self.post_countdown_speed_boost_time_bracket, "post_countdown_speed_boost_time_bracket")?;
        r.read_real(&mut self.post_countdown_speed_boost, "post_countdown_speed_boost")?;
        r.read_real(&mut self.max_vibration_angle, "max_vibration_angle")?;
        r.read_real(&mut self.vibration_angle_increment, "vibration_angle_increment")?;
        r.read_real(&mut self.LS_speed_max_speed, "LS_speed_max_speed")?;
        r.read_real(&mut self.LS_speed_boost, "LS_speed_boost")?;
        r.read_real(&mut self.LS_speed_boost_duration, "LS_speed_boost_duration")?;
        r.read_real(&mut self.necessary_steering_input, "necessary_steering_input")?;
        r.read_real(&mut self.min_dir_dot_for_no_doughnuts, "min_dir_dot_for_no_doughnuts")?;
        r.read_real(&mut self.driving_wheels_lateral_friction_boost_at_low_speed, "driving_wheels_lateral_friction_boost_at_low_speed")?;
        r.read_real(&mut self.driving_wheels_tangential_friction_for_doughnuts, "driving_wheels_tangential_friction_for_doughnuts")?;
        r.read_real(&mut self.physycs_vs_play_for_doughnuts, "physycs_vs_play_for_doughnuts")?;
        r.read_real(&mut self.min_backwards_speed, "min_backwards_speed")?;
        r.read_real(&mut self.jturn_min_omega_up_to_start, "jturn_min_omega_up_to_start")?;
        r.read_real(&mut self.jturn_max_omega_up_to_stop, "jturn_max_omega_up_to_stop")?;
        r.read_real(&mut self.backwards_necessary_steering_input, "backwards_necessary_steering_input")?;
        r.read_real(&mut self.steering_wheels_lateral_friction_boost, "steering_wheels_lateral_friction_boost")?;
        r.read_real(&mut self.steering_wheels_lateral_friction, "steering_wheels_lateral_friction")?;
        r.read_real(&mut self.driving_wheels_tangential_friction_boost, "driving_wheels_tangential_friction_boost")?;
        r.read_real(&mut self.driving_wheels_tangential_friction, "driving_wheels_tangential_friction")?;
        r.read_real(&mut self.max_skid_angle_to_reduce_the_boost_effect, "max_skid_angle_to_reduce_the_boost_effect")?;
        r.read_real(&mut self.max_surface_velocity_to_guess_from_accelerator_only, "max_surface_velocity_to_guess_from_accelerator_only")?;
        r.read_real(&mut self.max_lateral_velocity_for_direction_switch, "max_lateral_velocity_for_direction_switch")?;
        r.read_real(&mut self.physics_vs_play_reduction_time, "physics_vs_play_reduction_time")?;
        r.read_real(&mut self.physics_vs_play_relaxation_time_in_timesteps, "physics_vs_play_relaxation_time_in_timesteps")?;
        r.read_int(&mut self.use_physics_vs_play_only_during_skidding, "use_physics_vs_play_only_during_skidding")?;
        r.read_real(&mut self.physics_vs_play_min_speed, "physics_vs_play_min_speed")?;
        r.read_real(&mut self.entering_skidding_steering_boost, "entering_skidding_steering_boost")?;
        r.read_real(&mut self.min_steer_input_to_apply_steering_boost, "min_steer_input_to_apply_steering_boost")?;
        r.read_int(&mut self.max_entering_skidding_time__frames, "max_entering_skidding_time__frames")?;
        r.read_real(&mut self.slip_velocity, "slip_velocity")?;
        r.read_real(&mut self.min_forward_velocity_to_kick, "min_forward_velocity_to_kick")?;
        r.read_real(&mut self.min_steer_input_to_kick, "min_steer_input_to_kick")?;
        r.read_int(&mut self.handbrake_disabled_after_kick__frames, "handbrake_disabled_after_kick__frames")?;
        r.read_int(&mut self.kick_disabled_after_kick__frames, "kick_disabled_after_kick__frames")?;
        r.read_real(&mut self.tan_of_min_skid_angle_to_apply_bias, "tan_of_min_skid_angle_to_apply_bias")?;
        r.read_int(&mut self.normal_handbrake_time, "normal_handbrake_time")?;
        r.read_real(&mut self.max_steering_input_to_use_normal_handbrake, "max_steering_input_to_use_normal_handbrake")?;
        self.accumulated_NI_skid_sensitivity_boost = r.read_expression("accumulated_NI_skid_sensitivity_boost", "NI_skid_time NI_weighed_skid_time")?;
        r.read_real(&mut self.characteristic_decay_time, "characteristic_decay_time")?;
        r.read_real(&mut self.AIS_max_accelerator_release_duration, "AIS_max_accelerator_release_duration")?;
        r.read_real(&mut self.AIS_min_velocity, "AIS_min_velocity")?;
        r.read_real(&mut self.AIS_hard_steering_min_value, "AIS_hard_steering_min_value")?;
        r.read_real(&mut self.AIS_hard_accelerator_min_value, "AIS_hard_accelerator_min_value")?;
        r.read_real(&mut self.AIS_no_accelerator_max_value, "AIS_no_accelerator_max_value")?;
        r.read_int(&mut self.AIS_time_to_hold_handbrake_after_reapplying_accelerator, "AIS_time_to_hold_handbrake_after_reapplying_accelerator")?;
        r.read_int(&mut self.AIS_time_to_hold_handbrake_after_steer_and_accelerator, "AIS_time_to_hold_handbrake_after_steer_and_accelerator")?;
        r.read_int(&mut self.AIS_time_to_steer_and_footbrake_for_skid, "AIS_time_to_steer_and_footbrake_for_skid")?;
        r.read_int(&mut self.AIS_time_to_hold_handbrake_after_steer_and_footbrake, "AIS_time_to_hold_handbrake_after_steer_and_footbrake")?;
        r.read_int(&mut self.AIS_time_to_steer_only_for_skid, "AIS_time_to_steer_only_for_skid")?;
        r.read_int(&mut self.AIS_time_to_hold_handbrake_after_steer_only, "AIS_time_to_hold_handbrake_after_steer_only")?;
        r.read_real(&mut self.AIS_kick_multiplier_steer_and_accelerator, "AIS_kick_multiplier_steer_and_accelerator")?;
        r.read_int(&mut self.AIS_force_DF_duration, "AIS_force_DF_duration")?;
        r.read_real(&mut self.kick_omega_change_factor, "kick_omega_change_factor")?;
        r.read_real(&mut self.max_omega_up_to_stop, "max_omega_up_to_stop")?;
        r.read_real(&mut self.tan_of_min_skid_angle_for_fish_tail_removal, "tan_of_min_skid_angle_for_fish_tail_removal")?;
        r.read_int(&mut self.NO_past_skidding_checks, "NO_past_skidding_checks")?;
        r.read_int(&mut self.NO_past_skidding_successes, "NO_past_skidding_successes")?;
        r.read_real(&mut self.forward_speed_boost, "forward_speed_boost")?;
        r.read_int(&mut self.fish_tail_stop_duration, "fish_tail_stop_duration")?;
        r.read_real(&mut self.portion_of_omega_up_to_stop, "portion_of_omega_up_to_stop")?;
        r.read_real(&mut self.dpos_to_stop_aligning, "dpos_to_stop_aligning")?;
        r.read_real(&mut self.dangle_to_stop_aligning, "dangle_to_stop_aligning")?;
        r.read_real(&mut self.smooth_align_tau, "smooth_align_tau")?;
        r.read_real(&mut self.min_dpos_to_cancel_aligning, "min_dpos_to_cancel_aligning")?;
        r.read_real(&mut self.target_align_velocity, "target_align_velocity")?;
        r.read_int(&mut self.max_aligning_time, "max_aligning_time")?;
        r.read_int(&mut self.min_time_between_2_wheels, "min_time_between_2_wheels")?;
        r.read_int(&mut self.min_time_on_2_wheels, "min_time_on_2_wheels")?;
        r.read_int(&mut self.max_time_on_2_wheels, "max_time_on_2_wheels")?;
        r.read_real(&mut self.max_2_wheel_forward_speed_to_start, "max_2_wheel_forward_speed_to_start")?;
        r.read_real(&mut self.min_2_wheel_forward_speed, "min_2_wheel_forward_speed")?;
        r.read_real(&mut self.max_balancing_angle_to_start, "max_balancing_angle_to_start")?;
        r.read_real(&mut self.max_balancing_omega_change_start, "max_balancing_omega_change_start")?;
        r.read_real(&mut self.max_balancing_omega_change_end, "max_balancing_omega_change_end")?;
        r.read_int(&mut self.max_balancing_omega_change_interpolation_time, "max_balancing_omega_change_interpolation_time")?;
        r.read_real(&mut self.boost_to_steering_on_2_wheels, "boost_to_steering_on_2_wheels")?;
        r.read_real(&mut self.boost_to_engine_power_on_2_wheels, "boost_to_engine_power_on_2_wheels")?;
        r.read_real(&mut self.max_balancing_omega_change_to_switch_to_2_wheels, "max_balancing_omega_change_to_switch_to_2_wheels")?;
        r.read_real(&mut self.min_difference_of_balancing_omega, "min_difference_of_balancing_omega")?;
        r.read_real(&mut self.balancing_time_1_wheel, "balancing_time_1_wheel")?;
        r.read_real(&mut self.balancing_time_2_wheels, "balancing_time_2_wheels")?;
        r.read_real(&mut self.min_up_up_to_disable_chassis_CD, "min_up_up_to_disable_chassis_CD")?;
        r.read_real(&mut self.min_face_normal_up_to_disable_chassis_CD, "min_face_normal_up_to_disable_chassis_CD")?;
        r.read_int(&mut self.PL_enabled, "PL_enabled")?;
        r.read_real(&mut self.PL_eigenalizer_factor, "PL_eigenalizer_factor")?;
        r.read_real(&mut self.PL_max_omega_to_scale, "PL_max_omega_to_scale")?;
        r.read_real(&mut self.PL_omega_scale_x, "PL_omega_scale_x")?;
        r.read_real(&mut self.PL_omega_scale_y, "PL_omega_scale_y")?;
        r.read_real(&mut self.PL_omega_scale_z, "PL_omega_scale_z")?;
        self.PL_omega_scale_mode = if self.PL_omega_scale_x == 1.0 && self.PL_omega_scale_y == 1.0 && self.PL_omega_scale_z == 1.0 { 0.0 } else { 2.0 };
        r.read_real(&mut self.PL_min_speed_up, "PL_min_speed_up")?;
        r.read_real(&mut self.PL_min_horisontal_speed, "PL_min_horisontal_speed")?;
        r.read_real(&mut self.PL_min_flight_distance, "PL_min_flight_distance")?;
        r.read_real(&mut self.PL_min_flight_time, "PL_min_flight_time")?;
        r.read_real(&mut self.PL_min_normal_up_to_accept, "PL_min_normal_up_to_accept")?;
        r.read_real(&mut self.PL_max_omega_extra_should_be_at_least, "PL_max_omega_extra_should_be_at_least")?;
        r.read_real(&mut self.PL_max_omega_change_const, "PL_max_omega_change_const")?;
        r.read_real(&mut self.PL_max_omega_change_omega_dependent, "PL_max_omega_change_omega_dependent")?;
        r.read_real(&mut self.PL_flight_time_addition, "PL_flight_time_addition")?;
        r.read_real(&mut self.PL_min_dot_product_to_reverse_extra_rotation, "PL_min_dot_product_to_reverse_extra_rotation")?;
        r.read_real(&mut self.PL_max_rot_angle_multiplier_to_reverse_extra_rotation, "PL_max_rot_angle_multiplier_to_reverse_extra_rotation")?;
        r.read_int(&mut self.PL_max_NO_points, "PL_max_NO_points")?;
        r.read_real(&mut self.PL_max_delta_height, "PL_max_delta_height")?;
        r.read_real(&mut self.PL_timestep, "PL_timestep")?;
        r.read_int(&mut self.PL_ray_node_timestep_multiplier, "PL_ray_node_timestep_multiplier")?;
        r.read_int(&mut self.PL_increment_omega_once, "PL_increment_omega_once")?;
        r.read_int(&mut self.PL_use_Q_OMEGA_only, "PL_use_Q_OMEGA_only")?;
        r.read_real(&mut self.PL_max_total_flight_time_multiplier, "PL_max_total_flight_time_multiplier")?;
        r.read_real(&mut self.PL_roll_landing_disturbance, "PL_roll_landing_disturbance")?;
        r.read_real(&mut self.PL_pitch_landing_disturbance, "PL_pitch_landing_disturbance")?;
        r.read_real(&mut self.PL_landing_omega_scale, "PL_landing_omega_scale")?;
        r.read_real(&mut self.PL_landing_v_up_scale, "PL_landing_v_up_scale")?;
        r.read_real(&mut self.PL_max_landing_omega, "PL_max_landing_omega")?;
        r.read_real(&mut self.PL_min_landing_omega, "PL_min_landing_omega")?;
        r.read_real(&mut self.PL_vertical_rebounce, "PL_vertical_rebounce")?;
        r.read_real(&mut self.PL_max_vertical_rebounce_velocity, "PL_max_vertical_rebounce_velocity")?;
        r.read_real(&mut self.PL_landing_inertia_scale, "PL_landing_inertia_scale")?;
        r.read_real(&mut self.PL_landing_time_precision_to_use_inertia_scale, "PL_landing_time_precision_to_use_inertia_scale")?;
        r.read_real(&mut self.PL_landing_orientation_disturbance_length, "PL_landing_orientation_disturbance_length")?;
        r.read_real(&mut self.PL_landing_orientation_disturbance_magnitude, "PL_landing_orientation_disturbance_magnitude")?;
        r.read_real(&mut self.PL_landing_orientation_disturbance_referecne_speed_up, "PL_landing_orientation_disturbance_referecne_speed_up")?;
        r.read_real(&mut self.PL_landing_camera_tau_multiplier, "PL_landing_camera_tau_multiplier")?;
        r.read_real(&mut self.PL_landing_camera_limits_multiplier, "PL_landing_camera_limits_multiplier")?;
        r.read_int(&mut self.PL_landing_camera_tau_multiplier_duration, "PL_landing_camera_tau_multiplier_duration")?;
        r.read_real(&mut self.PL_max_landing_velocity_to_ignore_car_landing_velocity, "PL_max_landing_velocity_to_ignore_car_landing_velocity")?;
        r.read_real(&mut self.PL_acceleration_time_portion, "PL_acceleration_time_portion")?;
        r.read_real(&mut self.PL_extend_up_extent_by, "PL_extend_up_extent_by")?;
        r.read_real(&mut self.PL_min_omega_to_disable_CD, "PL_min_omega_to_disable_CD")?;
        r.read_real(&mut self.PL_min_face_normal_up_to_disable_chassis_CD, "PL_min_face_normal_up_to_disable_chassis_CD")?;
        r.read_real(&mut self.PL_portion_of_flight_time_to_disable_CD, "PL_portion_of_flight_time_to_disable_CD")?;
        r.read_real(&mut self.PL_min_deviation_angle, "PL_min_deviation_angle")?;
        r.read_real(&mut self.PL_jump_cam_probability, "PL_jump_cam_probability")?;
        r.read_real(&mut self.max_NO_iterations_coeffecient, "max_NO_iterations_coeffecient")?;
        r.read_real(&mut self.contacts_relaxation_timestep_multiplier, "contacts_relaxation_timestep_multiplier")?;
        r.read_real(&mut self.projection_timestep_multiplier, "projection_timestep_multiplier")?;
        r.read_int(&mut self.treat_stacking, "treat_stacking")?;
        r.read_int(&mut self.handle_contact_pairs, "handle_contact_pairs")?;
        r.read_int(&mut self.do_collision_resolution_for_contacts, "do_collision_resolution_for_contacts")?;
        r.read_int(&mut self.do_error_resolution_for_contacts, "do_error_resolution_for_contacts")?;
        r.read_real(&mut self.diag_scale, "diag_scale")?;
        r.read_real(&mut self.car_car_inertia_scale, "car_car_inertia_scale")?;
        r.read_real(&mut self.car_car_inertia_scale_min, "car_car_inertia_scale_min")?;
        r.read_real(&mut self.contact_friction_between_cars, "contact_friction_between_cars")?;
        r.read_real(&mut self.contact_restitution_between_cars, "contact_restitution_between_cars")?;
        r.read_real(&mut self.min_normal_up_for_timestep_subdivision, "min_normal_up_for_timestep_subdivision")?;
        r.read_real(&mut self.min_approaching_velocity_for_timestep_subdivision, "min_approaching_velocity_for_timestep_subdivision")?;
        r.read_int(&mut self.number_of_binary_divisions, "number_of_binary_divisions")?;
        r.read_real(&mut self.size_up_extension_for_car_car_CD, "size_up_extension_for_car_car_CD")?;
        r.read_real(&mut self.max_normal_not_forward_to_move_point, "max_normal_not_forward_to_move_point")?;
        r.read_real(&mut self.max_relp_side, "max_relp_side")?;
        r.read_real(&mut self.relp_to_leave, "relp_to_leave")?;
        r.read_real(&mut self.min_sideways_relp_for_traffic, "min_sideways_relp_for_traffic")?;
        r.read_real(&mut self.max_relp_up_for_player, "max_relp_up_for_player")?;
        r.read_real(&mut self.sideways_collision_exaggeration, "sideways_collision_exaggeration")?;
        r.read_real(&mut self.front_front_sideways_kick_factor, "front_front_sideways_kick_factor")?;
        r.read_real(&mut self.front_back_sideways_kick_factor, "front_back_sideways_kick_factor")?;
        r.read_real(&mut self.front_front_sideways_omega_kick_factor, "front_front_sideways_omega_kick_factor")?;
        r.read_real(&mut self.front_back_sideways_omega_kick_factor, "front_back_sideways_omega_kick_factor")?;
        r.read_real(&mut self.max_sideways_kick, "max_sideways_kick")?;
        r.read_real(&mut self.max_omega_kick, "max_omega_kick")?;
        r.read_real(&mut self.min_approaching_velocity_for_kick, "min_approaching_velocity_for_kick")?;
        r.read_real(&mut self.explicit_steer_angle, "explicit_steer_angle")?;
        r.read_real(&mut self.explicit_steer_angle_front_back_low_speed, "explicit_steer_angle_front_back_low_speed")?;
        r.read_int(&mut self.explicit_steer_time, "explicit_steer_time")?;
        r.read_int(&mut self.front_front_explicit_steer_delay_time, "front_front_explicit_steer_delay_time")?;
        r.read_int(&mut self.front_back_explicit_steer_delay_time, "front_back_explicit_steer_delay_time")?;
        r.read_int(&mut self.front_back_low_speed_explicit_steer_delay_time, "front_back_low_speed_explicit_steer_delay_time")?;
        r.read_real(&mut self.trafic_low_static_friction_after_collision, "trafic_low_static_friction_after_collision")?;
        r.read_int(&mut self.trafic_low_static_friction_after_collision_duration, "trafic_low_static_friction_after_collision_duration")?;
        r.read_real(&mut self.car_ground_inertia_scale, "car_ground_inertia_scale")?;
        r.read_real(&mut self.car_ground_inertia_scale_at_initiated_skid, "car_ground_inertia_scale_at_initiated_skid")?;
        r.read_real(&mut self.car_ground_inertia_scale_min, "car_ground_inertia_scale_min")?;
        r.read_real(&mut self.max_speed_to_handle_contact_pairs, "max_speed_to_handle_contact_pairs")?;
        r.read_real(&mut self.ground_contact_friction_with_horizontal_features, "ground_contact_friction_with_horizontal_features")?;
        r.read_real(&mut self.ground_contact_friction_with_vertical_features, "ground_contact_friction_with_vertical_features")?;
        r.read_real(&mut self.ground_contact_restitution_with_horizontal_features, "ground_contact_restitution_with_horizontal_features")?;
        r.read_real(&mut self.ground_contact_restitution_with_vertical_features, "ground_contact_restitution_with_vertical_features")?;
        r.read_int(&mut self.ground_filer_out_vertical_contacts_when_driving_on_all_wheels, "ground_filer_out_vertical_contacts_when_driving_on_all_wheels")?;
        r.read_real(&mut self.lift_box0_center_when_driving_on_all_wheels_by, "lift_box0_center_when_driving_on_all_wheels_by")?;
        r.read_real(&mut self.moved_contact_torque_comparison_factor, "moved_contact_torque_comparison_factor")?;
        r.read_real(&mut self.minimum_contact_velocity_to_filter, "minimum_contact_velocity_to_filter")?;
        r.read_real(&mut self.compare_coeffecient, "compare_coeffecient")?;
        r.read_real(&mut self.additional_penetration_allowance, "additional_penetration_allowance")?;
        r.read_real(&mut self.low_speed_max_backface_angle, "low_speed_max_backface_angle")?;
        r.read_int(&mut self.try_to_improve_car_bottom_CD, "try_to_improve_car_bottom_CD")?;
        r.read_real(&mut self.max_normal_sideways_projection, "max_normal_sideways_projection")?;
        r.read_real(&mut self.max_normal_forward_projection, "max_normal_forward_projection")?;
        r.read_int(&mut self.higher_restitution_min_flight_time, "higher_restitution_min_flight_time")?;
        r.read_real(&mut self.higher_restitution_min_normal_up, "higher_restitution_min_normal_up")?;
        r.read_real(&mut self.higher_restitution, "higher_restitution")?;
        r.read_real(&mut self.wl_max_cos_of_normal_up, "wl_max_cos_of_normal_up")?;
        r.read_real(&mut self.wl_min_of_cos_of_normal_car_forward, "wl_min_of_cos_of_normal_car_forward")?;
        r.read_real(&mut self.wl_necessary_speed_change, "wl_necessary_speed_change")?;
        r.read_real(&mut self.wl_max_speed_change, "wl_max_speed_change")?;
        r.read_real(&mut self.wl_delta_omega_car_z, "wl_delta_omega_car_z")?;
        r.read_real(&mut self.wl_car_car_delta_omega_car_z_for_player, "wl_car_car_delta_omega_car_z_for_player")?;
        r.read_real(&mut self.CS_min_car_speed, "CS_min_car_speed")?;
        r.read_real(&mut self.CS_min_delta_speed, "CS_min_delta_speed")?;
        r.read_real(&mut self.CS_acceptable_shake_distance, "CS_acceptable_shake_distance")?;
        r.read_real(&mut self.CS_tau, "CS_tau")?;
        r.read_real(&mut self.CS_tau_back, "CS_tau_back")?;
        r.read_int(&mut self.CS_duration, "CS_duration")?;
        r.read_real(&mut self.CS_criticality, "CS_criticality")?;
        r.read_real(&mut self.CS_motion_scale, "CS_motion_scale")?;
        r.read_real(&mut self.CS_sideways_motion_scale, "CS_sideways_motion_scale")?;
        r.read_real(&mut self.CS_sideways_tau_scale, "CS_sideways_tau_scale")?;
        r.read_real(&mut self.CS_car_car_motion_scale, "CS_car_car_motion_scale")?;
        r.read_real(&mut self.CS_car_car_sideways_motion_scale, "CS_car_car_sideways_motion_scale")?;
        r.read_real(&mut self.CS_camera_diff_to_stop, "CS_camera_diff_to_stop")?;
        r.read_real(&mut self.CVI_inertia_reference_frame, "CVI_inertia_reference_frame")?;
        r.read_real(&mut self.CVI_tau_car_up, "CVI_tau_car_up")?;
        r.read_real(&mut self.CVI_tau_car_down, "CVI_tau_car_down")?;
        r.read_real(&mut self.CVI_motion_scale_car_up, "CVI_motion_scale_car_up")?;
        r.read_real(&mut self.CVI_motion_scale_car_down, "CVI_motion_scale_car_down")?;
        r.read_real(&mut self.CVI_max_motion_car_up, "CVI_max_motion_car_up")?;
        r.read_real(&mut self.CVI_max_motion_car_down, "CVI_max_motion_car_down")?;
        r.read_real(&mut self.CVI_motion_stop_decay_const, "CVI_motion_stop_decay_const")?;
        r.read_real(&mut self.CSM_min_speed, "CSM_min_speed")?;
        r.read_real(&mut self.CSM_assumed_camera_distance, "CSM_assumed_camera_distance")?;
        r.read_real(&mut self.CSM_camera_motion_angle, "CSM_camera_motion_angle")?;
        r.read_real(&mut self.CSM_skid_angle_conversion_factor, "CSM_skid_angle_conversion_factor")?;
        r.read_real(&mut self.CSM_omega_conversion_factor, "CSM_omega_conversion_factor")?;
        r.read_real(&mut self.CSM_tau_no_steer, "CSM_tau_no_steer")?;
        r.read_real(&mut self.CSM_tau_some_steer_input, "CSM_tau_some_steer_input")?;
        r.read_real(&mut self.CSM_extra_yaw_conversion, "CSM_extra_yaw_conversion")?;
        r.read_real(&mut self.detached_camera_enabled, "detached_camera_enabled")?;
        r.read_int(&mut self.time_to_move_out_the_camera_after_detachment, "time_to_move_out_the_camera_after_detachment")?;
        r.read_int(&mut self.time_to_move_out_the_camera_after_detachment_after_collision, "time_to_move_out_the_camera_after_detachment_after_collision")?;
        r.read_real(&mut self.total_trail_increase, "total_trail_increase")?;
        r.read_real(&mut self.total_pitch_increase, "total_pitch_increase")?;
        r.read_real(&mut self.trail_increase_in_a_frame, "trail_increase_in_a_frame")?;
        r.read_int(&mut self.time_to_move_in_the_camera_after_attachment, "time_to_move_in_the_camera_after_attachment")?;
        r.read_real(&mut self.trail_decrease_in_a_frame, "trail_decrease_in_a_frame")?;
        r.read_real(&mut self.min_omega_mag_to_detach_on_tumbling, "min_omega_mag_to_detach_on_tumbling")?;
        r.read_real(&mut self.max_omega_to_attach_camera_after_jturn, "max_omega_to_attach_camera_after_jturn")?;
        r.read_real(&mut self.camera_attachment_factor_after_jturn, "camera_attachment_factor_after_jturn")?;
        r.read_real(&mut self.max_omega_to_attach_camera_after_doughnut, "max_omega_to_attach_camera_after_doughnut")?;
        r.read_real(&mut self.camera_attachment_factor_after_doughnut, "camera_attachment_factor_after_doughnut")?;
        r.read_int(&mut self.idle_input_realignment_time, "idle_input_realignment_time")?;
        r.read_int(&mut self.alignement_duration, "alignement_duration")?;
        r.read_real(&mut self.camera_trail_smoothout_tau, "camera_trail_smoothout_tau")?;
        r.read_real(&mut self.camera_factor_smoothout_tau, "camera_factor_smoothout_tau")?;
        r.read_real(&mut self.car_barrier_inertia_scale, "car_barrier_inertia_scale")?;
        r.read_real(&mut self.car_barrier_inertia_scale_at_initiated_skid, "car_barrier_inertia_scale_at_initiated_skid")?;
        r.read_real(&mut self.car_barrier_inertia_scale_min, "car_barrier_inertia_scale_min")?;
        r.read_real(&mut self.barrier_contact_friction_with_horizontal_features, "barrier_contact_friction_with_horizontal_features")?;
        r.read_real(&mut self.barrier_contact_friction_with_vertical_features, "barrier_contact_friction_with_vertical_features")?;
        r.read_real(&mut self.barrier_contact_restitution_with_horizontal_features, "barrier_contact_restitution_with_horizontal_features")?;
        r.read_real(&mut self.barrier_contact_restitution_with_vertical_features, "barrier_contact_restitution_with_vertical_features")?;
        r.read_real(&mut self.barrier_contacts_relaxation_timestep_multiplier, "barrier_contacts_relaxation_timestep_multiplier")?;
        r.read_int(&mut self.barrier_send_collision_message, "barrier_send_collision_message")?;
        r.read_int(&mut self.barrier_filer_out_vertical_contacts_when_driving_on_all_wheels, "barrier_filer_out_vertical_contacts_when_driving_on_all_wheels")?;
        r.read_real(&mut self.max_stastic_object_radius, "max_stastic_object_radius")?;
        r.read_real(&mut self.car_static_object_inertia_scale, "car_static_object_inertia_scale")?;
        r.read_real(&mut self.car_static_object_inertia_scale_at_initiated_skid, "car_static_object_inertia_scale_at_initiated_skid")?;
        r.read_real(&mut self.car_static_object_inertia_scale_min, "car_static_object_inertia_scale_min")?;
        r.read_real(&mut self.static_object_contact_friction_with_horizontal_features, "static_object_contact_friction_with_horizontal_features")?;
        r.read_real(&mut self.static_object_contact_friction_with_vertical_features, "static_object_contact_friction_with_vertical_features")?;
        r.read_real(&mut self.static_object_contact_restitution_with_horizontal_features, "static_object_contact_restitution_with_horizontal_features")?;
        r.read_real(&mut self.static_object_contact_restitution_with_vertical_features, "static_object_contact_restitution_with_vertical_features")?;
        r.read_int(&mut self.static_object_filter_out_vertical_contacts_when_driving_on_all_wheels, "static_object_filter_out_vertical_contacts_when_driving_on_all_wheels")?;
        r.read_real(&mut self.TT_min_car_velocity_for_through_tree_transition, "TT_min_car_velocity_for_through_tree_transition")?;
        r.read_real(&mut self.TT_max_car_velocity_for_through_tree_transition, "TT_max_car_velocity_for_through_tree_transition")?;
        r.read_real(&mut self.TT_max_capsule_radius, "TT_max_capsule_radius")?;
        r.read_real(&mut self.TT_min_capsule_halflength, "TT_min_capsule_halflength")?;
        r.read_real(&mut self.TT_forward_extent_extension_min, "TT_forward_extent_extension_min")?;
        r.read_real(&mut self.TT_forward_extent_extension_max, "TT_forward_extent_extension_max")?;
        r.read_real(&mut self.TT_car_static_object_inertia_scale, "TT_car_static_object_inertia_scale")?;
        r.read_real(&mut self.TT_car_static_object_inertia_scale_min, "TT_car_static_object_inertia_scale_min")?;
        r.read_real(&mut self.car_static_objects_max_normal_car_x_for_timestep_subdivision, "car_static_objects_max_normal_car_x_for_timestep_subdivision")?;
        r.read_real(&mut self.car_static_objects_min_approaching_velocity_for_timestep_subdivision, "car_static_objects_min_approaching_velocity_for_timestep_subdivision")?;
        r.read_int(&mut self.car_static_objects_number_of_binary_divisions, "car_static_objects_number_of_binary_divisions")?;
        r.read_real(&mut self.min_normal_up_to_filter_out, "min_normal_up_to_filter_out")?;
        r.read_real(&mut self.car_object_inertia_scale, "car_object_inertia_scale")?;
        r.read_real(&mut self.car_object_inertia_scale_at_initiated_skid, "car_object_inertia_scale_at_initiated_skid")?;
        r.read_real(&mut self.car_object_inertia_scale_min, "car_object_inertia_scale_min")?;
        r.read_real(&mut self.car_object_friction, "car_object_friction")?;
        r.read_real(&mut self.car_object_restitution, "car_object_restitution")?;
        r.read_real(&mut self.drag_scale, "drag_scale")?;
        r.read_real(&mut self.drag_comparison_factor, "drag_comparison_factor")?;
        r.read_real(&mut self.neighbor_objects_drag_scale, "neighbor_objects_drag_scale")?;
        r.read_real(&mut self.neighbor_objects_radnom_omega_addition, "neighbor_objects_radnom_omega_addition")?;
        r.read_real(&mut self.neighbor_objects_car_reference_velocity, "neighbor_objects_car_reference_velocity")?;
        r.read_real(&mut self.peice_linear_velocity_drag_scale, "peice_linear_velocity_drag_scale")?;
        r.read_real(&mut self.peice_radnom_omega_addition, "peice_radnom_omega_addition")?;
        r.read_real(&mut self.upper_limit_on_collision_speed, "upper_limit_on_collision_speed")?;
        r.read_real(&mut self.size_up_extension_for_car_object_CD, "size_up_extension_for_car_object_CD")?;
        r.read_real(&mut self.lift_box0_by_for_car_object_CD, "lift_box0_by_for_car_object_CD")?;
        r.read_int(&mut self.NO_car_boxes_to_collide_with_objects, "NO_car_boxes_to_collide_with_objects")?;
        r.read_real(&mut self.car_objects_max_normal_car_x_for_timestep_subdivision, "car_objects_max_normal_car_x_for_timestep_subdivision")?;
        r.read_real(&mut self.car_objects_min_approaching_velocity_for_timestep_subdivision, "car_objects_min_approaching_velocity_for_timestep_subdivision")?;
        r.read_int(&mut self.car_objects_number_of_binary_divisions, "car_objects_number_of_binary_divisions")?;
        r.read_real(&mut self.object_ground_max_NO_iterations_coeffecient, "object_ground_max_NO_iterations_coeffecient")?;
        r.read_real(&mut self.object_ground_friction, "object_ground_friction")?;
        r.read_real(&mut self.object_ground_restitution, "object_ground_restitution")?;
        r.read_real(&mut self.object_ground_inertia_scale, "object_ground_inertia_scale")?;
        r.read_real(&mut self.length_of_CD_rays, "length_of_CD_rays")?;
        r.read_real(&mut self.min_time_alive, "min_time_alive")?;
        r.read_real(&mut self.max_time_alive, "max_time_alive")?;
        r.read_real(&mut self.max_position_change_to_put_to_sleep, "max_position_change_to_put_to_sleep")?;
        r.read_real(&mut self.max_omega_to_put_to_sleep, "max_omega_to_put_to_sleep")?;
        r.read_real(&mut self.min_contact_normal_up_to_put_to_sleep, "min_contact_normal_up_to_put_to_sleep")?;
        r.read_real(&mut self.max_angle_change_to_put_to_sleep, "max_angle_change_to_put_to_sleep")?;
        r.read_real(&mut self.objects_mass_multiplier, "objects_mass_multiplier")?;
        r.read_real(&mut self.spherical_inertia_threshold, "spherical_inertia_threshold")?;
        r.read_real(&mut self.objects_min_extent, "objects_min_extent")?;
        r.read_reals(&mut self.objects_gravity, "objects_gravity")?;
        r.read_reals(&mut self.objects_gravity_in_air, "objects_gravity_in_air")?;
        r.read_real(&mut self.objects_linear_damping, "objects_linear_damping")?;
        r.read_real(&mut self.objects_linear_damping_increase_time, "objects_linear_damping_increase_time")?;
        r.read_real(&mut self.objects_angular_damping, "objects_angular_damping")?;
        r.read_real(&mut self.objects_angular_damping_increase_time, "objects_angular_damping_increase_time")?;
        r.read_int(&mut self.objects_handle_contact_pairs, "objects_handle_contact_pairs")?;
        r.read_real(&mut self.min_normal_up_to_collide_triangle_with_wheels, "min_normal_up_to_collide_triangle_with_wheels")?;
        r.read_int(&mut self.wheel_CD_with_raycasts, "wheel_CD_with_raycasts")?;
        r.read_real(&mut self.up_radius_multiplier, "up_radius_multiplier")?;
        r.read_real(&mut self.down_radius_multiplier, "down_radius_multiplier")?;
        r.read_real(&mut self.max_penetration_change_per_step, "max_penetration_change_per_step")?;
        r.read_real(&mut self.max_normal_change_per_step, "max_normal_change_per_step")?;
        r.read_int(&mut self.min_time_without_wheel_contact_to_match_wheel_rotation_to_car_speed, "min_time_without_wheel_contact_to_match_wheel_rotation_to_car_speed")?;
        r.read_int(&mut self.time_to_reduce_thrust, "time_to_reduce_thrust")?;
        r.read_real(&mut self.tan_of_critical_angle, "tan_of_critical_angle")?;
        r.read_real(&mut self.bump_map_spacial_size, "bump_map_spacial_size")?;
        r.read_real(&mut self.blast_radius, "blast_radius")?;
        r.read_real(&mut self.blast_velocity_increment, "blast_velocity_increment")?;
        r.read_real(&mut self.blast_angular_velocity_increment, "blast_angular_velocity_increment")?;
        r.read_real(&mut self.blast_centre_up_shift, "blast_centre_up_shift")?;
        r.read_real(&mut self.cos_of_min_disable_skid_marks_angle, "cos_of_min_disable_skid_marks_angle")?;
        r.read_real(&mut self.camera_shift_forward, "camera_shift_forward")?;
        r.read_int(&mut self.draw_quad_tree_output, "draw_quad_tree_output")?;
        r.read_int(&mut self.draw_quad_tree_cell, "draw_quad_tree_cell")?;
        r.read_int(&mut self.draw_triangles_intersecting_the_wheel, "draw_triangles_intersecting_the_wheel")?;
        r.read_int(&mut self.draw_static_object_triangles, "draw_static_object_triangles")?;
        r.read_int(&mut self.draw_static_object_triangles_intersecting_the_wheel, "draw_static_object_triangles_intersecting_the_wheel")?;
        r.read_int(&mut self.draw_chassis_CD_boxes, "draw_chassis_CD_boxes")?;
        r.read_real(&mut self.draw_debug_camera_chassis_CD_boxes, "draw_debug_camera_chassis_CD_boxes")?;
        r.read_int(&mut self.draw_chassis_ground_CD_boxes, "draw_chassis_ground_CD_boxes")?;
        r.read_int(&mut self.draw_chassis_car_car_CD_boxes, "draw_chassis_car_car_CD_boxes")?;
        r.read_real(&mut self.draw_chassis_CD_contacts, "draw_chassis_CD_contacts")?;
        r.read_real(&mut self.draw_objects_CD_contacts, "draw_objects_CD_contacts")?;
        r.read_real(&mut self.draw_wheel_CD_contacts, "draw_wheel_CD_contacts")?;
        r.read_int(&mut self.draw_wheel_CD_boxes, "draw_wheel_CD_boxes")?;
        r.read_int(&mut self.draw_colliding_boxes, "draw_colliding_boxes")?;
        r.read_real(&mut self.draw_colliding_boxes_contacts, "draw_colliding_boxes_contacts")?;
        r.read_int(&mut self.draw_alive_objects, "draw_alive_objects")?;
        r.read_real(&mut self.draw_proximity_objects, "draw_proximity_objects")?;
        r.read_real(&mut self.draw_proximity_static_objects, "draw_proximity_static_objects")?;
        r.read_real(&mut self.draw_wheel_forces, "draw_wheel_forces")?;
        r.read_int(&mut self.draw_PL_trajectory, "draw_PL_trajectory")?;
        r.read_int(&mut self.draw_PL_rays, "draw_PL_rays")?;
        r.read_real(&mut self.draw_PL_landing_spot, "draw_PL_landing_spot")?;
        r.read_int(&mut self.draw_PL_Q_hit, "draw_PL_Q_hit")?;
        r.read_int(&mut self.draw_PL_Q_sought, "draw_PL_Q_sought")?;
        r.read_real(&mut self.draw_flight_K, "draw_flight_K")?;
        r.read_real(&mut self.draw_departure_omega, "draw_departure_omega")?;
        r.read_real(&mut self.draw_extra_omega, "draw_extra_omega")?;
        r.read_real(&mut self.draw_current_omega, "draw_current_omega")?;
        r.read_int(&mut self.draw_visual_steering_direction, "draw_visual_steering_direction")?;
        r.read_int(&mut self.chassis_CD_enabled, "chassis_CD_enabled")?;
        r.read_int(&mut self.resolve_collisions, "resolve_collisions")?;
        r.read_int(&mut self.do_CD, "do_CD")?;
        r.read_int(&mut self.barrier_CD_enabled, "barrier_CD_enabled")?;
        r.read_int(&mut self.car_object_CD_enabled, "car_object_CD_enabled")?;
        r.read_int(&mut self.static_object_CD_enabled, "static_object_CD_enabled")?;
        r.read_int(&mut self.wheel_CD_enabled, "wheel_CD_enabled")?;
        r.read_int(&mut self.tty_output_steering_angle, "tty_output_steering_angle")?;
        r.read_int(&mut self.tty_output_skidding_forces, "tty_output_skidding_forces")?;
        r.read_int(&mut self.tty_output_skid_start_stop, "tty_output_skid_start_stop")?;
        r.read_int(&mut self.tty_output_going_forwards_backwards, "tty_output_going_forwards_backwards")?;
        r.read_int(&mut self.tty_output_anti_fish_tailing_work, "tty_output_anti_fish_tailing_work")?;
        r.read_int(&mut self.tty_output_pl_data, "tty_output_pl_data")?;
        r.read_int(&mut self.tty_output_ss_data, "tty_output_ss_data")?;
        r.read_int(&mut self.tty_output_2wheels, "tty_output_2wheels")?;
        r.read_real(&mut self.g_ftFarClipDistance, "g_ftFarClipDistance")?;
        r.read_real(&mut self.g_yawRatePerDegree, "g_yawRatePerDegree")?;
        r.read_real(&mut self.g_yawRelaxAngle, "g_yawRelaxAngle")?;
        r.read_real(&mut self.g_yawDirectionBlend, "g_yawDirectionBlend")?;
        r.read_real(&mut self.g_WheelCamFOV, "g_WheelCamFOV")?;
        r.read_real(&mut self.g_WheelCamOffsetX, "g_WheelCamOffsetX")?;
        r.read_real(&mut self.g_WheelCamOffsetZ, "g_WheelCamOffsetZ")?;
        Ok(LoadResult::Loaded)
    }
}

/// `CLocalConfig` (744 bytes, `VehicleEntity+0xDE0`).
#[derive(Clone, Debug, Default)]
pub struct CLocalConfig {
    pub timestep: f32, // +0x0
    pub dynamic_velocity: f32, // +0x4
    pub fMaxSlipStreamDist: f32, // +0x8
    pub mass: f32, // +0xC
    pub MOI_box_size: [f32; 3], // +0x10
    pub span_front_back: f32, // +0x1C
    pub span_front: f32, // +0x20
    pub span_back: f32, // +0x24
    pub front_wheel_radius: f32, // +0x28
    pub front_wheel_width: f32, // +0x2C
    pub front_wheel_mass: f32, // +0x30
    pub back_wheel_radius: f32, // +0x34
    pub back_wheel_width: f32, // +0x38
    pub back_wheel_mass: f32, // +0x3C
    pub number_of_boxes: i32, // +0x40
    pub upper_suspension_limit: f32, // +0x44
    pub lower_suspension_limit: f32, // +0x48
    pub maximum_forward_speed: f32, // +0x4C
    pub speed_limit: f32, // +0x50
    pub speed_limit_delta_speed: f32, // +0x54
    pub number_of_forward_gears: i32, // +0x58
    pub engine_power: f32, // +0x5C
    pub startline_acc_multiplier: f32, // +0x60
    pub gear_ratios_multiplier: f32, // +0x64
    pub portion_of_power_to_apply_as_propulsion_force: f32, // +0x68
    pub forwards_drive_traction_control: f32, // +0x6C
    pub COM_pos_x: f32, // +0x70
    pub COM_pos_y: f32, // +0x74
    pub topple_correction: f32, // +0x78
    pub fLeanMulFB: f32, // +0x7C
    pub fLeanMulLR: f32, // +0x80
    pub verticalWheelOffset: f32, // +0x84
    pub permanent_brake_torque: f32, // +0x88
    pub permanent_brake_torque_without_ground_contact: f32, // +0x8C
    pub forwards_brake_torque: f32, // +0x90
    pub forward_velocity_dependent_steering_constant_A_at_high_speed: f32, // +0x94
    pub backward_velocity_dependent_steering_constant_A_at_high_speed: f32, // +0x98
    pub static_friction_longitudinal: f32, // +0x9C
    pub static_friction_lateral: f32, // +0xA0
    pub static_friction_up_the_walls: f32, // +0xA4
    pub static_to_dynamic_friction_ratio: f32, // +0xA8
    pub the_low_speed_min: f32, // +0xAC
    pub the_low_speed_max: f32, // +0xB0
    pub physics_vs_play_max: f32, // +0xB4
    pub physics_vs_play_min: f32, // +0xB8
    pub skid_angle_to_start_reducing_steering_angle: f32, // +0xBC
    pub skid_angle_to_zero_steering_angle: f32, // +0xC0
    pub angular_orientation_up_kick_when_entering_skidding: f32, // +0xC4
    pub tan_of_skid_angle: f32, // +0xC8
    pub AIS_time_to_steer_and_accelerator_for_skid: i32, // +0xCC
    pub number_of_skid_sensitivities: i32, // +0xD0
    pub min_local_velocity_x_for_fish_tail_removal: f32, // +0xD4
    pub PL_use_spherical_rotation: i32, // +0xD8
    pub car_ground_inertia_at_initiated_skid_min_velocity: f32, // +0xDC
    pub camera_max_no_yaw_speed_low: f32, // +0xE0
    pub camera_max_no_yaw_speed_high: f32, // +0xE4
    pub barrier_contact_allowed_penetration_with_vertical_features: f32, // +0xE8
    /// copy of `speed_limit` taken while loading
    pub speed_limit_loaded: f32, // +0xEC
    pub gear_ratios: [f32; 6], // +0xF0
    /// copy of `engine_power`
    pub engine_power_loaded: f32, // +0x108
    /// copy of `forwards_drive_traction_control`
    pub forwards_drive_traction_control_loaded: f32, // +0x10C
    /// copies of the two steering constants
    pub forward_steering_constant_A_loaded: f32, // +0x110
    pub backward_steering_constant_A_loaded: f32, // +0x114
    pub driving_wheel: [i32; 4], // +0x118
    /// copy of `static_friction_longitudinal`
    pub static_friction_longitudinal_loaded: f32, // +0x128
    /// `static_friction_lateral / static_friction_longitudinal`
    pub lateral_to_longitudinal_friction_ratio: f32, // +0x12C
    pub sensitivity_into_turn: [Expression; 6], // +0x130
    pub sensitivity_to_straighten_up: [Expression; 6], // +0x148
    pub full_strength_time: [f32; 6], // +0x160
    /// running sum of `full_strength_time` (start time of each block)
    pub cumulative_full_strength_time: [f32; 6], // +0x178
    pub omega_kick_magnitude: [f32; 6], // +0x190
    pub max_kick_duration: [f32; 6], // +0x1A8
    pub physics_vs_play_method_into_turn: [f32; 6], // +0x1C0
    pub physics_vs_play_method_to_straighten_up: [f32; 6], // +0x1D8
    pub bias_on_input: [f32; 6], // +0x1F0
    pub force_expression: [[Expression; 4]; 6], // +0x208
    pub NO_skidding_forces: [i32; 6], // +0x268
    pub force_direction: [[i32; 4]; 6], // +0x280
    /// `CD_box_size=new real[number_of_boxes][3]`
    pub CD_box_size: Vec<[f32; 3]>, // +0x2E0
    pub CD_box_position: Vec<[f32; 3]>, // +0x2E4
}

/// variables of `sensitivity_into_turn` / `sensitivity_to_straighten_up`
pub const SENSITIVITY_VARS: &str = "handbrake_press_time";
/// variables of `force_expression`
pub const FORCE_VARS: &str = "skid_angle forward_speed lateral_speed skid_time continous_skid_time forwards backwards handbrake handbrake_press continous_handbrake_press_duration accelerator_press weight is_slipping dot_of_velocities number_of_touching_wheels";

impl CLocalConfig {
    /// CConfig.cpp 0x0019A220 (CLocalConfig::Load)
    pub fn load(&mut self, path: &str, defines: &mut Defines) -> Result<LoadResult, ReadError> {
        let file = TextFile::open(path).ok_or_else(|| ReadError { name: path.into(), line: String::new() })?;
        let mut r = ConfigReader::new(file, defines);
        self.load_from(&mut r)
    }

    pub fn load_from(&mut self, r: &mut ConfigReader) -> Result<LoadResult, ReadError> {
        let mut stop = 0i32;
        r.read_int(&mut stop, "stop_loading")?;
        if stop != 0 {
            return Ok(LoadResult::Stopped);
        }
        r.read_real(&mut self.timestep, "timestep")?;
        r.read_real(&mut self.dynamic_velocity, "dynamic_velocity")?;
        r.read_real(&mut self.fMaxSlipStreamDist, "fMaxSlipStreamDist")?;
        r.read_real(&mut self.mass, "mass")?;
        r.read_reals(&mut self.MOI_box_size, "MOI_box_size")?;
        r.read_real(&mut self.span_front_back, "span_front_back")?;
        r.read_real(&mut self.span_front, "span_front")?;
        r.read_real(&mut self.span_back, "span_back")?;
        r.read_real(&mut self.front_wheel_radius, "front_wheel_radius")?;
        r.read_real(&mut self.front_wheel_width, "front_wheel_width")?;
        r.read_real(&mut self.front_wheel_mass, "front_wheel_mass")?;
        r.read_real(&mut self.back_wheel_radius, "back_wheel_radius")?;
        r.read_real(&mut self.back_wheel_width, "back_wheel_width")?;
        r.read_real(&mut self.back_wheel_mass, "back_wheel_mass")?;
        r.read_int(&mut self.number_of_boxes, "number_of_boxes")?;
        let n = self.number_of_boxes.max(0) as usize;
        self.CD_box_size = vec![[0.0; 3]; n];
        self.CD_box_position = vec![[0.0; 3]; n];
        for i in 0..n {
            r.read_reals(&mut self.CD_box_size[i], "CD_box_size")?;
            r.read_reals(&mut self.CD_box_position[i], "CD_box_position")?;
        }
        r.read_real(&mut self.upper_suspension_limit, "upper_suspension_limit")?;
        r.read_real(&mut self.lower_suspension_limit, "lower_suspension_limit")?;
        r.read_real(&mut self.maximum_forward_speed, "maximum_forward_speed")?;
        r.read_real(&mut self.speed_limit, "speed_limit")?;
        self.speed_limit_loaded = self.speed_limit;
        r.read_real(&mut self.speed_limit_delta_speed, "speed_limit_delta_speed")?;
        r.read_int(&mut self.number_of_forward_gears, "number_of_forward_gears")?;
        r.read_real(&mut self.engine_power, "engine_power")?;
        self.engine_power_loaded = self.engine_power;
        r.read_real(&mut self.startline_acc_multiplier, "startline_acc_multiplier")?;
        r.read_ints(&mut self.driving_wheel, "driving_wheel")?;
        // one read sized by the gear count (1..6); other counts read nothing
        let g = self.number_of_forward_gears;
        if (1..=6).contains(&g) {
            r.read_reals(&mut self.gear_ratios[..g as usize], "gear_ratios")?;
        }
        r.read_real(&mut self.gear_ratios_multiplier, "gear_ratios_multiplier")?;
        r.read_real(&mut self.portion_of_power_to_apply_as_propulsion_force, "portion_of_power_to_apply_as_propulsion_force")?;
        r.read_real(&mut self.forwards_drive_traction_control, "forwards_drive_traction_control")?;
        self.forwards_drive_traction_control_loaded = self.forwards_drive_traction_control;
        r.read_real(&mut self.COM_pos_x, "COM_pos_x")?;
        r.read_real(&mut self.COM_pos_y, "COM_pos_y")?;
        r.read_real(&mut self.topple_correction, "topple_correction")?;
        r.read_real(&mut self.fLeanMulFB, "fLeanMulFB")?;
        r.read_real(&mut self.fLeanMulLR, "fLeanMulLR")?;
        r.read_real(&mut self.verticalWheelOffset, "verticalWheelOffset")?;
        r.read_real(&mut self.permanent_brake_torque, "permanent_brake_torque")?;
        r.read_real(&mut self.permanent_brake_torque_without_ground_contact, "permanent_brake_torque_without_ground_contact")?;
        r.read_real(&mut self.forwards_brake_torque, "forwards_brake_torque")?;
        r.read_real(
            &mut self.forward_velocity_dependent_steering_constant_A_at_high_speed,
            "forward_velocity_dependent_steering_constant_A_at_high_speed",
        )?;
        self.forward_steering_constant_A_loaded = self.forward_velocity_dependent_steering_constant_A_at_high_speed;
        r.read_real(
            &mut self.backward_velocity_dependent_steering_constant_A_at_high_speed,
            "backward_velocity_dependent_steering_constant_A_at_high_speed",
        )?;
        self.backward_steering_constant_A_loaded = self.backward_velocity_dependent_steering_constant_A_at_high_speed;
        r.read_real(&mut self.static_friction_longitudinal, "static_friction_longitudinal")?;
        r.read_real(&mut self.static_friction_lateral, "static_friction_lateral")?;
        self.static_friction_longitudinal_loaded = self.static_friction_longitudinal;
        self.lateral_to_longitudinal_friction_ratio = self.static_friction_lateral / self.static_friction_longitudinal;
        r.read_real(&mut self.static_friction_up_the_walls, "static_friction_up_the_walls")?;
        r.read_real(&mut self.static_to_dynamic_friction_ratio, "static_to_dynamic_friction_ratio")?;
        r.read_real(&mut self.the_low_speed_min, "the_low_speed_min")?;
        r.read_real(&mut self.the_low_speed_max, "the_low_speed_max")?;
        r.read_real(&mut self.physics_vs_play_max, "physics_vs_play_max")?;
        r.read_real(&mut self.physics_vs_play_min, "physics_vs_play_min")?;
        r.read_real(&mut self.skid_angle_to_start_reducing_steering_angle, "skid_angle_to_start_reducing_steering_angle")?;
        r.read_real(&mut self.skid_angle_to_zero_steering_angle, "skid_angle_to_zero_steering_angle")?;
        r.read_real(
            &mut self.angular_orientation_up_kick_when_entering_skidding,
            "angular_orientation_up_kick_when_entering_skidding",
        )?;
        r.read_real(&mut self.tan_of_skid_angle, "tan_of_skid_angle")?;
        r.read_int(&mut self.AIS_time_to_steer_and_accelerator_for_skid, "AIS_time_to_steer_and_accelerator_for_skid")?;
        r.read_int(&mut self.number_of_skid_sensitivities, "number_of_skid_sensitivities")?;
        for b in self.force_expression.iter_mut() {
            for e in b.iter_mut() {
                *e = Expression::default();
            }
        }
        for k in 0..self.number_of_skid_sensitivities.clamp(0, 6) as usize {
            self.sensitivity_into_turn[k] = r.read_expression("sensitivity_into_turn", SENSITIVITY_VARS)?;
            self.sensitivity_to_straighten_up[k] = r.read_expression("sensitivity_to_straighten_up", SENSITIVITY_VARS)?;
            r.read_real(&mut self.full_strength_time[k], "full_strength_time")?;
            r.read_real(&mut self.omega_kick_magnitude[k], "omega_kick_magnitude")?;
            r.read_real(&mut self.max_kick_duration[k], "max_kick_duration")?;
            r.read_real(&mut self.physics_vs_play_method_into_turn[k], "physics_vs_play_method_into_turn")?;
            r.read_real(&mut self.physics_vs_play_method_to_straighten_up[k], "physics_vs_play_method_to_straighten_up")?;
            r.read_real(&mut self.bias_on_input[k], "bias_on_input")?;
            r.read_int(&mut self.NO_skidding_forces[k], "NO_skidding_forces")?;
            for f in 0..self.NO_skidding_forces[k].clamp(0, 4) as usize {
                r.read_int(&mut self.force_direction[k][f], "force_direction")?;
                self.force_expression[k][f] = r.read_expression("force_expression", FORCE_VARS)?;
            }
        }
        self.cumulative_full_strength_time[0] = self.full_strength_time[0];
        for k in 1..self.number_of_skid_sensitivities.clamp(0, 6) as usize {
            self.cumulative_full_strength_time[k] = self.cumulative_full_strength_time[k - 1] + self.full_strength_time[k];
        }
        r.read_real(&mut self.min_local_velocity_x_for_fish_tail_removal, "min_local_velocity_x_for_fish_tail_removal")?;
        r.read_int(&mut self.PL_use_spherical_rotation, "PL_use_spherical_rotation")?;
        r.read_real(
            &mut self.car_ground_inertia_at_initiated_skid_min_velocity,
            "car_ground_inertia_at_initiated_skid_min_velocity",
        )?;
        r.read_real(&mut self.camera_max_no_yaw_speed_low, "camera_max_no_yaw_speed_low")?;
        r.read_real(&mut self.camera_max_no_yaw_speed_high, "camera_max_no_yaw_speed_high")?;
        r.read_real(
            &mut self.barrier_contact_allowed_penetration_with_vertical_features,
            "barrier_contact_allowed_penetration_with_vertical_features",
        )?;
        Ok(LoadResult::Loaded)
    }
}

/// Directory of the config files.
pub const CFG_DIR: &str = "Vehicles\\cfgs\\";

/// `Main.cfg` -> `Main<n>.cfg` when a numbered variant is requested (`.cfg`/`.CFG` kept)
pub fn numbered(name: &str, n: i32) -> String {
    if n <= 0 {
        return name.into();
    }
    for ext in [".cfg", ".CFG"] {
        if let Some(i) = name.find(ext) {
            return format!("{}{}{}", &name[..i], n, ext);
        }
    }
    name.into()
}

/// Reads `Main.cfg` through the given defines and calls `load` for every file it lists.
/// Returns false when any file failed (the original asserts and carries on).
pub fn load_main<F: FnMut(&str, &mut Defines) -> bool>(main: &str, n: i32, defines: &mut Defines, mut load: F) -> bool {
    let path = format!("{}{}", CFG_DIR, numbered(main, n));
    let Some(file) = TextFile::open(&path) else {
        log::error!(" !!!Could not load {}!!!", path);
        return false;
    };
    let mut ok = true;
    let mut r = ConfigReader::new(file, defines);
    while r.next_line() {
        let line = String::from_utf8_lossy(&r.line).into_owned();
        let sub = format!("{}{}", CFG_DIR, line.trim_start_matches([' ', '\t']));
        if load(&sub, r.defines) {
            log::debug!(" {} loaded ok.", sub);
        } else {
            log::error!(" !!!Could not load {}!!!", sub);
            ok = false;
        }
    }
    ok
}

/// the debug/platform switches defined before every load
fn common_defines(d: &mut Defines) {
    d.define("bLOOP", "0");
    d.define("bEUGENE_LAPTEV", "0");
    d.define("bEL", "0");
    d.define("EUGENE_LAPTEV", "//");
    d.define("EL", "//");
    d.define("bMASTER", "0");
    d.define("MASTER", "//");
    d.define("bPC", "0");
    d.define("PC", "//");
}

/// CConfig.cpp 0x00199DC0 — loads `::GC` from `Main.cfg` (LC lines commented out).
pub fn load_global_config(gc: &mut CGlobalConfig, main: &str, n: i32) -> bool {
    let mut d = Defines::new();
    d.define("LC", "//");
    d.define("GC", "");
    common_defines(&mut d);
    load_main(main, n, &mut d, |path, defs| gc.load(path, defs).is_ok())
}

/// `CConfig::LoadGlobal` with the `Mainsafe.cfg` fallback used by `VehicleEntity::Update`.
pub fn load_global_config_with_fallback(n: i32) -> Option<CGlobalConfig> {
    let mut gc = CGlobalConfig::default();
    if load_global_config(&mut gc, "Main.cfg", n) {
        return Some(gc);
    }
    let mut gc = CGlobalConfig::default();
    load_global_config(&mut gc, "Mainsafe.cfg", 0).then_some(gc)
}

pub(crate) fn local_common_defines(d: &mut Defines, professional_race: bool) {
    d.define("LC", "");
    d.define("GC", "//");
    d.define("bLOOP", "0");
    d.define("bPROFESSIONAL_RACE", if professional_race { "1" } else { "0" });
    d.define("bEUGENE_LAPTEV", "0");
    d.define("bEL", "0");
    d.define("EUGENE_LAPTEV", "//");
    d.define("EL", "//");
    d.define("bMASTER", "0");
    d.define("MASTER", "//");
    d.define("bPC", "0");
    d.define("PC", "//");
}
