//! The ai tweak block at 0x006D3698 (`VsAIGlobals`, constructed by a static initialiser in
//! VsAIWorld.cpp and exposed through the tweak menu by `VsAIWorld::AddTweaks`).
//!
//! Defaults are copied bit-exact from the constructor; field comments give the tweak name and offset.

/// nitrous behaviour per aggression level ("AI.Nitrous.<level> Aggression").
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NitrousAggression {
    pub start_straightness_length: f32,    // "Start straightness length"
    pub continue_straightness_length: f32, // "Continue straightness length"
    pub overtaken_time_window: f32,        // "Overtaken time window"
    pub overtaken_length_factor: f32,      // "Overtaken length factor"
    pub minimum_road_straightness: f32,    // "Minimum road straightness"
    pub max_steering: f32,                 // "Max steering"
}

/// slipstream behaviour per skill level ("AI.Slipstream.<level> skill").
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlipstreamSkill {
    pub min_distance: f32,          // "Min distance"
    pub min_speed: f32,             // "Min speed"
    pub min_speed_diff_current: f32, // "Min speed diff (current)"
    pub min_speed_diff_new: f32,    // "Min speed diff (new)"
    pub max_speed_difference: f32,  // "Max speed difference"
    pub semicircle_radius: f32,     // "Semicircle radius"
    pub semicircle_angle: f32,      // "Semicircle angle"
    pub direction_difference: f32,  // "Direction difference"
    pub steering_smoothness: f32,   // "Steering smoothness"
}

/// per-opponent handling override written by the tweak menu ("AI.OpponentHandling.Individual").
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HandlingOverride {
    pub max_speed: f32,               // +0x1A8 "MaxSpeed"
    pub look_ahead_min: f32,          // +0x1AC "LookAheadMin"
    pub look_ahead_max: f32,          // +0x1B0 "LookAheadMax"
    pub steering_factor: f32,         // +0x1B4 "SteeringFactor"
    pub throttle_factor: f32,         // +0x1B8 "ThrottleFactor"
    pub brake_factor: f32,            // +0x1BC "BrakeFactor"
    pub steering_damping_factor: f32, // +0x1C0 "SteeringDampingFactor"
}

#[derive(Debug, Clone, PartialEq)]
pub struct VsAIGlobals {
    pub player_ai_control: bool,                 // +0x000 "PlayerAIControl"
    pub make_mistakes: bool,                     // +0x001 "AI.Mistakes/Make Mistakes"
    pub skid_out_time: f32,                      // +0x004
    pub min_skid_out_steering: f32,              // +0x008
    pub skid_out_handbrake_strength: f32,        // +0x00C
    pub brake_late_time: f32,                    // +0x010
    pub min_brake_late_corner_tightness: f32,    // +0x014
    pub brake_late_brake_multiplier: f32,        // +0x018
    pub brake_late_steering_multiplier: f32,     // +0x01C
    pub misalign_time: f32,                      // +0x020
    pub max_misalign_road_width: f32,            // +0x024
    pub misalign_amount: f32,                    // +0x028
    pub force_follow_raceline: bool,             // +0x02C
    pub raceline_track_cam: bool,                // +0x02D
    pub draw_vehicle_debug: bool,                // +0x02E
    pub draw_driver_debug: bool,                 // +0x02F
    pub draw_avoidance_debug: bool,              // +0x030
    pub draw_mistake_debug: bool,                // +0x031
    pub draw_raceline: bool,                     // +0x032
    pub draw_edges: bool,                        // +0x033
    pub draw_lanes: bool,                        // +0x034
    pub draw_traf_trig_splines: bool,            // +0x035
    pub draw_trail_blazer_splines: bool,         // +0x036
    pub draw_triggers: bool,                     // +0x037
    pub draw_response_curves: bool,              // +0x038
    pub trigger_finish_race: bool,               // +0x039
    pub draw_police_debug: bool,                 // +0x03A
    pub apply_slowdown: bool,                    // +0x03B "AI.OpponentHandling.Global/Apply Slowdown"
    pub use_nitrous: bool,                       // +0x03C
    pub nos_ahead_of_player_threshold: f32,      // +0x040
    pub nos_eor_no_conservation_length: f32,     // +0x044
    pub nos_speed_check_distance: f32,           // +0x048
    pub nos_speed_check_threshold: f32,          // +0x04C
    pub nitrous_aggression: [NitrousAggression; 3], // +0x050 low, med, high
    pub nos_conservation_propensity: [f32; 3],   // +0x098 low, med, high skill
    pub judgment_error: [f32; 3],                // +0x0A4 low, med, high skill
    pub catchup_boost: [f32; 3],                 // +0x0B0 low, med, high skill
    pub failures_enabled: bool,                  // +0x0BC "AI.Failures/Enabled"
    pub min_ai_car_health: f32,                  // +0x0C0
    pub min_player_car_health: f32,              // +0x0C4
    pub max_ai_car_health: f32,                  // +0x0C8
    pub max_player_car_health: f32,              // +0x0CC
    pub max_health_at_prestige: f32,             // +0x0D0
    pub ai_damage_multiplier_from_player: f32,   // +0x0D4
    pub oil_spill_min_delay: f32,                // +0x0D8
    pub oil_spill_max_delay: f32,                // +0x0DC
    pub oil_spill_min_radius: f32,               // +0x0E0
    pub oil_spill_max_radius: f32,               // +0x0E4
    pub oil_spill_friction_mod: f32,             // +0x0E8
    pub oil_spill_lifetime: f32,                 // +0x0EC never set by the constructor (bss zero)
    pub nos_explosion_velocity_increment: f32,   // +0x0F0
    pub overrev_damage_scale: f32,               // +0x0F4
    pub force_failure: u32,                      // +0x0F8 EngineBlown 0, NOSExplosion 1, TyrePuncture 2, 6 = none
    pub use_slipstream: bool,                    // +0x0FC
    pub spd_diff_thresh_quick_overtake: f32,     // +0x100
    pub slipstream: [SlipstreamSkill; 3],        // +0x104 low, med, high
    pub disorient_min_impact_speed: f32,         // +0x170
    pub disorient_max_impact_speed: f32,         // +0x174
    pub disorient_max_delay: f32,                // +0x178
    pub tight_corner_threshold: f32,             // +0x17C
    pub brake_throt_scale: f32,                  // +0x180
    pub raceline_offset_scale: f32,              // +0x184
    pub force_speed_match: bool,                 // +0x188
    pub force_speed: f32,                        // +0x18C
    pub unk_190: [u32; 6],                       // +0x190 VsAIWorld.cpp 0x00037D30 zeroes +0x194..+0x1A4
    pub handling_override: HandlingOverride,     // +0x1A8
    pub unk_1c4: [f32; 7],                       // +0x1C4
    pub skill: u32,                              // +0x1E0 "Skill"
    pub base_aggression: u32,                    // +0x1E4 "Base_Aggression"
    pub difficulty_scale: f32,                   // +0x1E8 "AI/Difficulty Scale"
    pub shortcut_max_distance_to_player: f32,    // +0x1EC
    pub shortcut_max_player_prestige: f32,       // +0x1F0
    pub shortcut_distance_contribution: f32,     // +0x1F4
    pub shortcut_skill_contribution: f32,        // +0x1F8
    pub shortcut_aggression_contribution: f32,   // +0x1FC
    pub shortcut_prestige_contribution: f32,     // +0x200
    pub big_collision_threshold: f32,            // +0x204 "AI/big collision threshold"
    pub revs_min_target: f32,                    // +0x208 "AI.StartlineRevs/Min Target Revs"
    pub revs_max_target: f32,                    // +0x20C
    pub revs_min_ramp_time_start: f32,           // +0x210
    pub revs_max_ramp_time_start: f32,           // +0x214
    pub revs_min_hold_time_start: f32,           // +0x218
    pub revs_max_hold_time_start: f32,           // +0x21C
    pub revs_blend_factor: f32,                  // +0x220
    pub revs_min_time_multiplier: f32,           // +0x224
    pub revs_max_time_multiplier: f32,           // +0x228
    /// not in the block: initialised statics 0x00684E58.. ("AI.OpponentHandling.Global/Min Slowdown at Dist", ..),
    /// kept here so the tweak menu reaches them like the rest
    pub slowdown_min_dist: f32,                  // 0x00684E58
    pub slowdown_max_dist: f32,                  // 0x00684E5C
    pub slowdown_max_accel: f32,                 // 0x00684E60
}

const fn f(bits: u32) -> f32 {
    f32::from_bits(bits)
}

impl Default for VsAIGlobals {
    /// VsAIWorld.cpp 0x00036E00
    fn default() -> Self {
        Self {
            player_ai_control: false,
            make_mistakes: true,
            skid_out_time: f(0x3F80_0000),
            min_skid_out_steering: f(0x3D4C_CCCD),
            skid_out_handbrake_strength: f(0x3DCC_CCCD),
            brake_late_time: f(0x4020_0000),
            min_brake_late_corner_tightness: f(0x3E80_0000),
            brake_late_brake_multiplier: f(0x3E19_999A),
            brake_late_steering_multiplier: f(0x0000_0000),
            misalign_time: f(0x4100_0000),
            max_misalign_road_width: f(0x4180_0000),
            misalign_amount: f(0x4000_0000),
            force_follow_raceline: false,
            raceline_track_cam: false,
            draw_vehicle_debug: false,
            draw_driver_debug: false,
            draw_avoidance_debug: false,
            draw_mistake_debug: false,
            draw_raceline: false,
            draw_edges: false,
            draw_lanes: false,
            draw_traf_trig_splines: false,
            draw_trail_blazer_splines: false,
            draw_triggers: false,
            draw_response_curves: false,
            trigger_finish_race: false,
            draw_police_debug: false,
            apply_slowdown: true,
            use_nitrous: true,
            nos_ahead_of_player_threshold: f(0x4296_0000),
            nos_eor_no_conservation_length: f(0x43AF_0000),
            nos_speed_check_distance: f(0x4220_0000),
            nos_speed_check_threshold: f(0x41A0_0000),
            nitrous_aggression: [
                NitrousAggression {
                    start_straightness_length: f(0x42C8_0000),
                    continue_straightness_length: f(0x4248_0000),
                    overtaken_time_window: f(0x3F80_0000),
                    overtaken_length_factor: f(0x3F40_0000),
                    minimum_road_straightness: f(0x4170_0000),
                    max_steering: f(0x3E19_999A),
                },
                NitrousAggression {
                    start_straightness_length: f(0x428C_0000),
                    continue_straightness_length: f(0x4220_0000),
                    overtaken_time_window: f(0x4000_0000),
                    overtaken_length_factor: f(0x3F19_999A),
                    minimum_road_straightness: f(0x4170_0000),
                    max_steering: f(0x3E19_999A),
                },
                NitrousAggression {
                    start_straightness_length: f(0x4220_0000),
                    continue_straightness_length: f(0x41F0_0000),
                    overtaken_time_window: f(0x4060_0000),
                    overtaken_length_factor: f(0x3F00_0000),
                    minimum_road_straightness: f(0x4170_0000),
                    max_steering: f(0x3E19_999A),
                },
            ],
            nos_conservation_propensity: [f(0x3F00_0000), f(0x3F80_0000), f(0x3FA0_0000)],
            judgment_error: [f(0x3E99_999A), f(0x3E19_999A), f(0x3D4C_CCCD)],
            catchup_boost: [f(0x3FC0_0000), f(0x3FE0_0000), f(0x4000_0000)],
            failures_enabled: true,
            min_ai_car_health: f(0x447A_0000),
            min_player_car_health: f(0x44FA_0000),
            max_ai_car_health: f(0x451C_4000),
            max_player_car_health: f(0x459C_4000),
            max_health_at_prestige: f(0x471C_4000),
            ai_damage_multiplier_from_player: f(0x3F80_0000),
            oil_spill_min_delay: f(0x3F80_0000),
            oil_spill_max_delay: f(0x4040_0000),
            oil_spill_min_radius: f(0x3E99_999A),
            oil_spill_max_radius: f(0x3F80_0000),
            oil_spill_friction_mod: f(0x3F00_0000),
            oil_spill_lifetime: 0.0,
            nos_explosion_velocity_increment: f(0x4170_0000),
            overrev_damage_scale: f(0x4120_0000),
            force_failure: 6,
            use_slipstream: true,
            spd_diff_thresh_quick_overtake: f(0x40A0_0000),
            slipstream: [
                SlipstreamSkill {
                    min_distance: f(0x4080_0000),
                    min_speed: f(0x41C8_0000),
                    min_speed_diff_current: 0.0,
                    min_speed_diff_new: 0.0,
                    max_speed_difference: 0.0,
                    semicircle_radius: f(0x4220_0000),
                    semicircle_angle: f(0x41A0_0000),
                    direction_difference: f(0x4234_0000),
                    steering_smoothness: f(0x3F00_0000),
                },
                SlipstreamSkill {
                    min_distance: f(0x4080_0000),
                    min_speed: f(0x41C8_0000),
                    min_speed_diff_current: f(0xC120_0000),
                    min_speed_diff_new: 0.0,
                    max_speed_difference: f(0x4220_0000),
                    semicircle_radius: f(0x41C8_0000),
                    semicircle_angle: f(0x4254_0000),
                    direction_difference: f(0x41F0_0000),
                    steering_smoothness: f(0x3F19_999A),
                },
                SlipstreamSkill {
                    min_distance: f(0x4080_0000),
                    min_speed: f(0x41C8_0000),
                    min_speed_diff_current: f(0xC1A0_0000),
                    min_speed_diff_new: 0.0,
                    max_speed_difference: f(0x4220_0000),
                    semicircle_radius: f(0x41F0_0000),
                    semicircle_angle: f(0x4270_0000),
                    direction_difference: f(0x41F0_0000),
                    steering_smoothness: f(0x3F33_3333),
                },
            ],
            disorient_min_impact_speed: f(0x42A0_0000),
            disorient_max_impact_speed: f(0x437A_0000),
            disorient_max_delay: f(0x4020_0000),
            tight_corner_threshold: f(0x4000_0000),
            brake_throt_scale: f(0x40A0_0000),
            raceline_offset_scale: f(0x3F7A_E148),
            force_speed_match: false,
            force_speed: f(0x41A0_0000),
            unk_190: [0; 6],
            handling_override: HandlingOverride::default(),
            unk_1c4: [
                f(0x3E80_0000),
                f(0x3F00_0000),
                f(0x3F80_0000),
                f(0x4000_0000),
                f(0x3F00_0000),
                f(0x3E80_0000),
                f(0x3DCC_CCCD),
            ],
            skill: 0,
            base_aggression: 0,
            difficulty_scale: f(0x3F19_999A),
            shortcut_max_distance_to_player: f(0x437A_0000),
            shortcut_max_player_prestige: f(0x461C_4000),
            shortcut_distance_contribution: f(0x3BA3_D70A),
            shortcut_skill_contribution: f(0x3D4C_CCCD),
            shortcut_aggression_contribution: f(0x3D4C_CCCD),
            shortcut_prestige_contribution: f(0x38D1_B717),
            big_collision_threshold: f(0x41F0_0000),
            revs_min_target: f(0x45BB_8000),
            revs_max_target: f(0x45DA_C000),
            revs_min_ramp_time_start: f(0x4020_0000),
            revs_max_ramp_time_start: f(0x4040_0000),
            revs_min_hold_time_start: f(0x3F00_0000),
            revs_max_hold_time_start: f(0x3FC0_0000),
            revs_blend_factor: f(0x3DCC_CCCD),
            revs_min_time_multiplier: f(0x3F4C_CCCD),
            revs_max_time_multiplier: f(0x3F99_999A),
            slowdown_min_dist: f(0x4296_0000),
            slowdown_max_dist: f(0x4348_0000),
            slowdown_max_accel: f(0x3F00_0000),
        }
    }
}
