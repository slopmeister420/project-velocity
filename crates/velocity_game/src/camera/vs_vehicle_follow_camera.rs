//! src\Camera\VsVehicleFollowCamera.cpp — the chase camera. `update` is `VsVehicleFollowCamera::Update`
//! (0x00067510), `update_follow` its per-update wrapper (0x00066940: tweak defaults, chase / point of interest
//! dispatch, camera collision, banking, shake, bridge record); `camera_point` is the camera half of
//! `VehicleEntity` 0x001C5CF0 (the point the camera follows: shift forward, skid side shift, vertical inertia).
//! The original runs both inside the physics frame (`PhysicsManager` 0x0019F920 → "SyncModels" 0x001DDAE0, right
//! after ResolveCollisions, same dt), i.e. on this update's car state.
//!
//! Views whose record [14] is 0 (and the mirror) take the static camera 0x000690B0 instead of `update`. Renderer
//! cut requests (VsCameraManager 0x00060380 = GameBridge +0x14) are left in `render_cut` for the glue.
//! Not ported: the `VsRender` debug override 0x0005C210 (bss 0: the bridge record is always written).

use bevy::math::{Mat3, Vec3};

use crate::game_bridge::camera_bridge::CameraBridgeData;
use velocity_physics::vehicle::vehicle_entity::sim::PadInput;

/// tweakables (VsVehicleFollowCamera.cpp globals 0x00685118..0x00685260, "Camera.*" menus of 0x00065920; bss
/// ones are 0: the tweak registration does not set a value and F3_D never loads a tweak file)
/// the follow camera globals' .data values: registered ones are only the defaults of [`FollowGlobals`] (what the
/// camera reads), the rest are never written in F3_D and are read here directly
pub mod k {
    /// `flt_6851F4` "Car->Cam max angle" (degrees)
    pub const MAX_ANGLE: f32 = 90.0;
    /// `dword_6851F0` "Yaw slew rate" (no reader besides VsFlyCamera 0x00065610's write)
    pub const YAW_SLEW_RATE: f32 = 120.0;
    /// `flt_685134` / `flt_685138` fovMin/MaxOveride: fov range (degrees) over the speed range below
    pub const FOV_MIN: f32 = 60.0;
    pub const FOV_MAX: f32 = 80.0;
    /// `dword_68513C` fovBoost: fov added at full fury
    pub const FOV_BOOST: f32 = 20.0;
    /// `dword_685140` / `dword_685144` fovSpeedLow/High (m/s)
    pub const FOV_SPEED_MIN: f32 = 15.0;
    pub const FOV_SPEED_MAX: f32 = 40.0;
    /// `dword_68514C` / `dword_685150` fovChangeRate / Release, per update
    pub const FOV_RATE_UP: f32 = 0.15;
    pub const FOV_RATE_DOWN: f32 = 0.05;
    /// `dword_685154` / `dword_685158` trailSpeedLow/High, `dword_685148` speedDamping
    pub const PITCH_SPEED_MIN: f32 = 5.0;
    pub const PITCH_SPEED_MAX: f32 = 10.0;
    pub const PITCH_SPEED_RATE: f32 = 0.06;
    /// `dword_68517C` kite speed factor, `dword_685170` kiteMaxPitchAngle (degrees)
    pub const KITE_SPEED_FACTOR: f32 = 0.5;
    pub const KITE_MAX_PITCH: f32 = 4.0;
    /// `dword_685174` kiteMaxReceedDist, `dword_685178` kiteMaxGetCloserDist
    pub const KITE_RECEDE: f32 = 0.75;
    pub const KITE_CLOSER: f32 = 0.6;
    /// `flt_685180` / `flt_685184` kite accel / brake factor
    pub const KITE_ACCEL_FACTOR: f32 = 1.0;
    pub const KITE_BRAKE_FACTOR: f32 = 1.0;
    /// `dword_685188` / `dword_68518C` accel / brake rise time (s)
    pub const KITE_ACCEL_RISE: f32 = 0.7;
    pub const KITE_BRAKE_RISE: f32 = 1.1;
    /// `dword_685190` / `dword_685194` kite accel / brake decay factor (per update)
    pub const KITE_ACCEL_DECAY: f32 = 0.95;
    pub const KITE_BRAKE_DECAY: f32 = 0.97;
    /// `flt_685160` / `flt_685164` cameraTrailMin/Max: distance (m) at the fov range ends
    pub const DIST_NEAR_FOV: f32 = 4.1;
    pub const DIST_FAR_FOV: f32 = 3.6;
    /// `dword_685168` farCamTrailMultiplier (view 1)
    pub const FAR_CAM_TRAIL: f32 = 1.2;
    /// `flt_68516C` pivotPoint
    pub const PIVOT: f32 = 1.1;
    /// `dword_68515C` globalHeightDiff
    pub const EYE_Y: f32 = -0.45;
    /// `dword_6851AC` / `dword_6851B0` use dir below / use vel above speed
    pub const VEL_SPEED_MIN: f32 = 2.0;
    pub const VEL_SPEED_MAX: f32 = 6.0;
    /// `dword_6851B4` direction blend
    pub const HEADING_WEIGHT: f32 = 0.6;
    /// `dword_68522C`: look-around turn per unit of CameraRotateHoriz (detached camera only)
    pub const LOOK_AROUND: f32 = 0.4;
    /// `flt_6851A8` yawDeadZoneAngle, `dword_68519C` yawRatePerDegree, `flt_6851A0`, `flt_6851A4` yawRelaxAngle
    pub const YAW_DEAD: f32 = 0.5;
    pub const YAW_GAIN: f32 = 200.0;
    pub const YAW_MAX_RATE: f32 = 180.0;
    pub const YAW_FULL: f32 = 15.0;
    /// `flt_685200`: slopes flatter than this (degrees) are ignored; `flt_6851F8` / `flt_6D5E3C` (bss 0) limits
    pub const SLOPE_DEAD: f32 = 7.0;
    pub const SLOPE_MIN: f32 = -30.0;
    pub const SLOPE_MAX: f32 = 0.0;
    /// `flt_6851FC` slope pitch rate (degrees per second)
    pub const SLOPE_RATE: f32 = 10.0;
    /// "Camera.Globals" bounce rate / damping / amount (`dword_6851B8` / `6851BC` / `6851C0`), `byte_6D5E30`
    /// (bss 0: rises are not ignored), `byte_6851D4` "pitch camera" (1: the bounce pitches, else lifts by
    /// `dword_6851D8`), soft limit 0x00068FA0 args `dword_6851CC` / `6851D0` / `6851C4` / `6851C8`
    pub const BOUNCE_RATE: f32 = 3.0;
    pub const BOUNCE_DAMPING: f32 = 0.4;
    pub const BOUNCE_AMOUNT: f32 = 0.3;
    pub const BOUNCE_FALL_ONLY: bool = false;
    pub const PITCH_CAMERA: bool = true;
    pub const BOUNCE_LIFT: f32 = 0.02;
    pub const BOUNCE_KNEE: f32 = 0.5;
    pub const BOUNCE_SOFT_RATE: f32 = 2.0;
    pub const BOUNCE_LIMIT_NEG: f32 = 20.0;
    pub const BOUNCE_LIMIT_POS: f32 = 20.0;
    /// "Camera.Globals.Zone": use trail `byte_68521C` 1, use ramp curve `byte_68521D` 1, duration `flt_685218`,
    /// fov change `dword_6D5E28` (bss 0), pitch change `dword_685198` -25, trail change `dword_6D5E2C` (bss 0)
    pub const ZONE_USE_TRAIL: bool = true;
    pub const ZONE_RAMP_CURVE: bool = true;
    pub const ZONE_DURATION: f32 = 0.5;
    pub const ZONE_FOV_CHANGE: f32 = 0.0;
    pub const ZONE_PITCH_CHANGE: f32 = -25.0;
    pub const ZONE_TRAIL_CHANGE: f32 = 0.0;
    /// "Banking": enabled `byte_68511C` 1, speed low/high `dword_685120` / `685124`, max angle `flt_685128`
    /// (degrees), nos factor `dword_68512C`, lag `dword_685130`
    pub const BANKING: bool = true;
    pub const BANK_SPEED_MIN: f32 = 15.0;
    pub const BANK_SPEED_MAX: f32 = 30.0;
    pub const BANK_MAX_ANGLE: f32 = 8.0;
    pub const BANK_NOS_FACTOR: f32 = 0.5;
    pub const BANK_LAG: f32 = 0.06;
    /// "Camera.Globals.Shake": vertical `flt_6851DC`, horizontal `flt_6851E0`, min/max speed `dword_6851E4` /
    /// `6851E8`, drift factor `dword_6D5E34` (bss 0), 3rd person scale `dword_6851EC`; impact shake
    /// `flt_685228` / `flt_685224` / `dword_685220`; ZRotation `dword_6D5E38` (bss 0)
    pub const SHAKE_V: f32 = 0.008;
    pub const SHAKE_H: f32 = 0.001;
    pub const SHAKE_SPEED_MIN: f32 = 25.0;
    pub const SHAKE_SPEED_MAX: f32 = 60.0;
    pub const SHAKE_DRIFT: f32 = 0.0;
    pub const SHAKE_3RD_PERSON: f32 = 0.5;
    pub const IMPACT_UP: f32 = 1.0;
    pub const IMPACT_SIDE: f32 = 1.0;
    pub const IMPACT_BLEND: f32 = 0.5;
    pub const Z_ROTATION: f32 = 0.0;
    /// "Camera" Height / InterestHeight / RearOffset / BumperOffset / FOVMin (`dword_685204..685214`), copied over
    /// player 0's view every update (0x00066830)
    pub const HEIGHT: f32 = 1.8;
    pub const INTEREST_HEIGHT: f32 = 1.8;
    pub const REAR_OFFSET: f32 = 9.0;
    pub const BUMPER_OFFSET: f32 = 9.0;
    pub const FOV_VIEW: f32 = 40.0;
    /// `flt_686B5C` (0x41A00000): the static camera's fov added at full fury, times the view's [16]
    pub const FOV_FURY: f32 = 20.0;
}

/// the tweakable follow camera globals ("Camera.Globals.*" VsFlyCamera 0x00065920, "Camera.TurnRates" and
/// "ZRotation" 0x00065640): the original's .data / bss globals the camera reads every update; `Default` = the
/// .data values ([`k`]). Kept in `VsCameraManager.globals`, bound to the debug menu cells by core_tweaks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FollowGlobals {
    pub yaw_slew_rate: f32,     // dword_6851F0
    pub max_angle: f32,         // flt_6851F4
    pub z_rotation: f32,        // dword_6D5E38
    pub banking: bool,          // byte_68511C
    pub bank_speed_min: f32,    // dword_685120
    pub bank_speed_max: f32,    // dword_685124
    pub bank_max_angle: f32,    // flt_685128
    pub bank_nos_factor: f32,   // dword_68512C
    pub bank_lag: f32,          // dword_685130
    pub fov_min: f32,           // flt_685134
    pub fov_max: f32,           // flt_685138
    pub fov_speed_min: f32,     // dword_685140
    pub fov_speed_max: f32,     // dword_685144
    pub fov_rate_up: f32,       // dword_68514C
    pub fov_rate_down: f32,     // dword_685150
    pub fov_boost: f32,         // dword_68513C
    pub dist_near_fov: f32,     // flt_685160
    pub dist_far_fov: f32,      // flt_685164
    pub eye_y: f32,             // dword_68515C
    pub kite_max_pitch: f32,    // dword_685170
    pub kite_recede: f32,       // dword_685174
    pub kite_closer: f32,       // dword_685178
    pub kite_accel_factor: f32, // flt_685180
    pub kite_brake_factor: f32, // flt_685184
    pub kite_speed_factor: f32, // dword_68517C
    pub kite_accel_rise: f32,   // dword_685188
    pub kite_brake_rise: f32,   // dword_68518C
    pub kite_accel_decay: f32,  // dword_685190
    pub kite_brake_decay: f32,  // dword_685194
    pub pitch_speed_rate: f32,  // dword_685148
    pub pitch_speed_min: f32,   // dword_685154
    pub pitch_speed_max: f32,   // dword_685158
    pub pivot: f32,             // flt_68516C
    pub far_cam_trail: f32,     // dword_685168
    pub heading_weight: f32,    // dword_6851B4
    pub yaw_gain: f32,          // dword_68519C
    pub yaw_full: f32,          // flt_6851A4
    pub yaw_dead: f32,          // flt_6851A8
    pub vel_speed_min: f32,     // dword_6851AC
    pub vel_speed_max: f32,     // dword_6851B0
    pub bounce_rate: f32,       // dword_6851B8
    pub bounce_damping: f32,    // dword_6851BC
    pub bounce_amount: f32,     // dword_6851C0
    pub pitch_camera: bool,     // byte_6851D4
    pub shake_v: f32,           // flt_6851DC
    pub shake_h: f32,           // flt_6851E0
    pub shake_speed_min: f32,   // dword_6851E4
    pub shake_speed_max: f32,   // dword_6851E8
    pub shake_drift: f32,       // dword_6D5E34
    pub shake_3rd_person: f32,  // dword_6851EC
    pub zone_use_trail: bool,   // byte_68521C
    pub zone_ramp_curve: bool,  // byte_68521D
    pub zone_duration: f32,     // flt_685218
    pub zone_fov_change: f32,   // dword_6D5E28
    pub zone_pitch_change: f32, // dword_685198
    pub zone_trail_change: f32, // dword_6D5E2C
}

impl Default for FollowGlobals {
    fn default() -> Self {
        FollowGlobals {
            yaw_slew_rate: k::YAW_SLEW_RATE,
            max_angle: k::MAX_ANGLE,
            z_rotation: k::Z_ROTATION,
            banking: k::BANKING,
            bank_speed_min: k::BANK_SPEED_MIN,
            bank_speed_max: k::BANK_SPEED_MAX,
            bank_max_angle: k::BANK_MAX_ANGLE,
            bank_nos_factor: k::BANK_NOS_FACTOR,
            bank_lag: k::BANK_LAG,
            fov_min: k::FOV_MIN,
            fov_max: k::FOV_MAX,
            fov_speed_min: k::FOV_SPEED_MIN,
            fov_speed_max: k::FOV_SPEED_MAX,
            fov_rate_up: k::FOV_RATE_UP,
            fov_rate_down: k::FOV_RATE_DOWN,
            fov_boost: k::FOV_BOOST,
            dist_near_fov: k::DIST_NEAR_FOV,
            dist_far_fov: k::DIST_FAR_FOV,
            eye_y: k::EYE_Y,
            kite_max_pitch: k::KITE_MAX_PITCH,
            kite_recede: k::KITE_RECEDE,
            kite_closer: k::KITE_CLOSER,
            kite_accel_factor: k::KITE_ACCEL_FACTOR,
            kite_brake_factor: k::KITE_BRAKE_FACTOR,
            kite_speed_factor: k::KITE_SPEED_FACTOR,
            kite_accel_rise: k::KITE_ACCEL_RISE,
            kite_brake_rise: k::KITE_BRAKE_RISE,
            kite_accel_decay: k::KITE_ACCEL_DECAY,
            kite_brake_decay: k::KITE_BRAKE_DECAY,
            pitch_speed_rate: k::PITCH_SPEED_RATE,
            pitch_speed_min: k::PITCH_SPEED_MIN,
            pitch_speed_max: k::PITCH_SPEED_MAX,
            pivot: k::PIVOT,
            far_cam_trail: k::FAR_CAM_TRAIL,
            heading_weight: k::HEADING_WEIGHT,
            yaw_gain: k::YAW_GAIN,
            yaw_full: k::YAW_FULL,
            yaw_dead: k::YAW_DEAD,
            vel_speed_min: k::VEL_SPEED_MIN,
            vel_speed_max: k::VEL_SPEED_MAX,
            bounce_rate: k::BOUNCE_RATE,
            bounce_damping: k::BOUNCE_DAMPING,
            bounce_amount: k::BOUNCE_AMOUNT,
            pitch_camera: k::PITCH_CAMERA,
            shake_v: k::SHAKE_V,
            shake_h: k::SHAKE_H,
            shake_speed_min: k::SHAKE_SPEED_MIN,
            shake_speed_max: k::SHAKE_SPEED_MAX,
            shake_drift: k::SHAKE_DRIFT,
            shake_3rd_person: k::SHAKE_3RD_PERSON,
            zone_use_trail: k::ZONE_USE_TRAIL,
            zone_ramp_curve: k::ZONE_RAMP_CURVE,
            zone_duration: k::ZONE_DURATION,
            zone_fov_change: k::ZONE_FOV_CHANGE,
            zone_pitch_change: k::ZONE_PITCH_CHANGE,
            zone_trail_change: k::ZONE_TRAIL_CHANGE,
        }
    }
}

impl FollowGlobals {
    /// VsFlyCamera.cpp 0x00065610 — the vehicle database's GLOBALS.BIN record (+36 / +40) sets the turn rates
    pub fn set_turn_rates(&mut self, globals: &[f32; 11]) {
        self.yaw_slew_rate = globals[9];
        self.max_angle = globals[10];
    }
}

/// Armco.cpp 0x000141E0 `VsClamp`: `comiss v, lo; jbe` → lo also for NaN (a tweak range of zero width gives
/// 0 / 0 here), then hi when v > hi; no panic on crossed bounds, unlike `f32::clamp`
fn vs_clamp(v: f32, lo: f32, hi: f32) -> f32 {
    if v > lo {
        if v > hi { hi } else { v }
    } else {
        lo
    }
}

/// pad record bit 0 (`a4+16 & 1`): look back
pub const PAD_LOOK_BACK: u32 = 1;

/// VsCameraManager 0x000613D0 (`0.017453292f`)
pub fn rad(d: f32) -> f32 {
    d * f32::from_bits(0x3C8E_FA35)
}

/// one view of the car's camera table (`VehicleData` list a, 80-byte records; VsCameraManager 0x0005F5A0)
#[derive(Clone, Copy, Debug)]
pub struct CameraView {
    pub height: f32,      // [2] -> +504
    pub look_height: f32, // [0] -> +508
    pub dist_a: f32,      // [4] -> +512
    pub dist_b: f32,      // [5] -> +516
    pub fov: f32,         // [11] -> +520
    /// [14] -> +524 (0x00069650): non-zero = the chase camera (`update`), else the static camera 0x000690B0
    pub chase: f32,
    pub unknown_528: f32, // [15] -> +528
    pub unknown_532: f32, // [16] -> +532
    /// [17] near plane of chase views (others use 0.5)
    pub near: f32,
}

impl Default for CameraView {
    /// the shared defaults `VsFlyCamera` 0x00066830 copies (`dword_685204..685214`)
    fn default() -> Self {
        CameraView {
            height: k::HEIGHT,
            look_height: k::INTEREST_HEIGHT,
            dist_a: k::REAR_OFFSET,
            dist_b: k::BUMPER_OFFSET,
            fov: k::FOV_VIEW,
            chase: 0.0,
            unknown_528: 0.0,
            unknown_532: 0.0,
            near: 0.5,
        }
    }
}

/// the "Camera" tweak globals `dword_685204..685214` (height, interest height, rear / bumper offset, fov); .data
/// holds the defaults, VsFlyCamera 0x000668C0 copies player 0's view into them, 0x00066830 copies them back
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraTweaks {
    pub height: f32,
    pub look_height: f32,
    pub dist_a: f32,
    pub dist_b: f32,
    pub fov: f32,
}

impl Default for CameraTweaks {
    fn default() -> Self {
        CameraTweaks { height: k::HEIGHT, look_height: k::INTEREST_HEIGHT, dist_a: k::REAR_OFFSET, dist_b: k::BUMPER_OFFSET, fov: k::FOV_VIEW }
    }
}

impl CameraTweaks {
    /// VsFlyCamera.cpp 0x000668C0
    pub fn from_view(v: &CameraView) -> Self {
        CameraTweaks { height: v.height, look_height: v.look_height, dist_a: v.dist_a, dist_b: v.dist_b, fov: v.fov }
    }
}

impl CameraView {
    pub fn from_record(r: &[f32; 20]) -> Self {
        CameraView {
            height: r[2],
            look_height: r[0],
            dist_a: r[4],
            dist_b: r[5],
            fov: r[11],
            chase: r[14],
            unknown_528: r[15],
            unknown_532: r[16],
            near: r[17],
        }
    }
}

/// the point of interest as 0x0006A410 hands it over (object position, blend +0x10, out +0x18) and the fov
/// distances of 0x0006A570 (`None` when "extra fov" is off)
#[derive(Clone, Copy, Debug)]
pub struct PoiBlend {
    pub target: Vec3,
    pub t: f32,
    pub out: bool,
    /// min distance, max distance, fov factor at max (`dword_685258` / `68525C` / `685260`)
    pub fov_distances: Option<[f32; 3]>,
}

/// what `Update` reads, field by field from where the original takes it
#[derive(Clone, Copy, Debug)]
pub struct FollowTarget {
    /// a5: the camera point of `VehicleEntity` 0x001C5CF0 ([`camera_point`])
    pub position: Vec3,
    /// a6: VE+4788 row 0 (0x001BB1C0(0))
    pub heading: Vec3,
    /// a8: model +Z through this update's car bridge rotation (CarBridge 0x000971A0 +16, AC::Renderer +
    /// Renderer 0x002ABDE0): the rendered nose, the static camera's look when not looking back
    pub bridge_forward: Vec3,
    /// VehicleEntity 0x001C6630: VE+4788 row 1 (the static camera's up)
    pub car_up: Vec3,
    /// VsCameraManager 0x00060260 for the camera's player (view 3, the in-car view)
    pub in_car_view: bool,
    /// a7: FastCar +32 velocity (0x001B42B0)
    pub velocity: Vec3,
    /// VE+20 (`VsAIRaceVehicle__24220` on the VehicleEntity): |velocity|
    pub speed: f32,
    /// a4: the pad record (`VsTrafficManager` +24 + 36·VE+92), bit 0 = look back
    pub pad: PadInput,
    /// FuryFX 0x00128680 for the camera's player (+580)
    pub fury: f32,
    /// FuryFX "Fov Change" tweak `dword_686B5C` (read by 0x000690B0; default `FOV_FURY`)
    pub fury_fov: f32,
    /// VE+4480 (camera skid motion extra yaw, radians)
    pub extra_yaw: f32,
    /// VE+4484 per-view lag factor (1 = none, < 0.01 = detached camera)
    pub lag: f32,
    /// VE+4504 / VE+4516 detached camera distance / pitch (radians)
    pub trail_offset: f32,
    pub pitch_offset: f32,
    /// manager player record +41 (zone FX, 0x00069A70)
    pub in_zone: bool,
    /// one-off object of the ai vehicle (0x00023310 → +968 → +168)
    pub one_off: Option<f32>,
    /// CameraRotateHoriz (action 24, mask 0xFFFF), read only by a detached camera
    pub look_around: f32,
    pub poi: Option<PoiBlend>,
}

impl FollowTarget {
    pub fn new(position: Vec3, heading: Vec3, velocity: Vec3, speed: f32) -> Self {
        FollowTarget {
            position,
            heading,
            bridge_forward: heading,
            car_up: Vec3::Y,
            in_car_view: false,
            velocity,
            speed,
            pad: PadInput::default(),
            fury: 0.0,
            fury_fov: k::FOV_FURY,
            extra_yaw: 0.0,
            lag: 1.0,
            trail_offset: 0.0,
            pitch_offset: 0.0,
            in_zone: false,
            one_off: None,
            look_around: 0.0,
            poi: None,
        }
    }
}

/// what the 0x00066940 wrapper reads besides the target
#[derive(Clone, Copy, Debug, Default)]
pub struct FollowExtras {
    /// FastCar 0x00364B20 == 1 (a wheel sliding)
    pub sliding: bool,
    /// `VsAIDriver__15D50` game time
    pub game_time: f32,
    /// VE+5008 impact shake: amount, base +5012, start +5016, end +5020, VE rows 1 and 2
    pub impact: Option<ImpactShake>,
    /// the point of interest is running (0x0006A910, +0x24)
    pub poi_active: bool,
    /// the "Camera" tweak globals 0x00066830 copies over player 0's view
    pub tweaks: CameraTweaks,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ImpactShake {
    pub amount: f32,
    pub base: f32,
    pub start: f32,
    pub end: f32,
    pub up: Vec3,
    pub side: Vec3,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FollowOut {
    /// new VE+5008 when the impact shake ran
    pub impact_amount: Option<f32>,
    /// the point of interest ran out (0x00069BE0(0, 0, 0.5))
    pub poi_done: bool,
}

#[derive(Clone, Debug)]
pub struct VsVehicleFollowCamera {
    pub view: CameraView,
    pub right: Vec3, // +248
    pub up: Vec3,    // +264
    pub look: Vec3,  // +280
    pub eye: Vec3,   // +212 (as the bridge got it)
    /// +140 base camera fov (vertical, radians; 0x0005D0B0 = rad(+548))
    pub fov_rad: f32,
    /// +496 looking back, +497 the mirror camera
    pub rear: bool,
    pub mirror: bool,
    pub slope: f32,       // +500 (radians)
    pub bank: f32,        // +536 (radians)
    pub kite: f32,        // +540
    pub speed_pitch: f32, // +544
    pub fov: f32,         // +548 (degrees)
    pub pedal: f32,       // +552
    pub kite_input: f32,  // +556
    /// +560 index of the car view (1 far chase, > 1 on the car)
    pub view_index: u32,
    pub bounce: f32,      // +564
    pub bounce_vel: f32,  // +568
    pub last_vel_y: f32,  // +572
    /// +580
    pub player: u32,
    /// +584 the chase flag changed zero-ness (0x00069650): cut next update
    pub cut_pending: bool,
    pub unknown_585: bool,
    pub zone_time: f32,   // +588
    pub zone_fov: f32,    // +592
    /// +596 chase or point of interest last update
    pub dynamic: bool,
    /// the bridge record of the last update (0x00066940 writes it)
    pub bridge: Option<CameraBridgeData>,
    /// port: VsCameraManager 0x00060380(1) (GameBridge +0x14, render the latest update uninterpolated) was
    /// called; the glue moves it to the bridge
    pub render_cut: bool,
}

impl Default for VsVehicleFollowCamera {
    fn default() -> Self {
        VsVehicleFollowCamera {
            view: CameraView::default(),
            right: Vec3::X,
            up: Vec3::Y,
            look: Vec3::Z,
            eye: Vec3::ZERO,
            fov_rad: rad(k::FOV_MIN),
            rear: false,
            mirror: false,
            slope: 0.0,
            bank: 0.0,
            kite: 0.0,
            speed_pitch: 0.0,
            fov: k::FOV_MIN,
            pedal: 0.0,
            kite_input: 0.0,
            view_index: 0,
            bounce: 0.0,
            bounce_vel: 0.0,
            last_vel_y: 0.0,
            player: 0,
            cut_pending: false,
            unknown_585: false,
            zone_time: 0.0,
            zone_fov: 0.0,
            dynamic: false,
            bridge: None,
            render_cut: false,
        }
    }
}

/// `VsClamp(x, 0, 1)` (NaN → 0, as the original: a zero-width tweak range must not poison the camera)
fn clamp01(x: f32) -> f32 {
    vs_clamp(x, 0.0, 1.0)
}

/// CRim 0x000452C0
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (b - a) * t + a
}

/// VsCameraManager 0x000613E0 (xyz)
fn lerp3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    Vec3::new((b.x - a.x) * t + a.x, (b.y - a.y) * t + a.y, (b.z - a.z) * t + a.z)
}

fn flat_normalised(v: Vec3) -> Vec3 {
    Vec3::new(v.x, 0.0, v.z).normalize_or_zero()
}

/// VsVehicleFollowCamera.cpp 0x00068FA0 — soft limit: below the limit as is, above it eased towards it
pub fn soft_limit(x: f32, knee: f32, rate: f32, neg: f32, pos: f32) -> f32 {
    let (lim, sign) = if x >= 0.0 { (pos, 1.0) } else { (neg, -1.0) };
    let a = x.abs();
    if lim > a {
        return x;
    }
    let t = (a - lim * knee) * rate;
    let f = 1.0 - 1.0 / (t + 1.0);
    ((1.0 - knee) * f + knee) * lim * sign
}

/// VsVehicleFollowCameraPointOfInterest.cpp 0x0006A570 — fov factor of the point of interest by its distance
fn poi_fov_scale(target: Vec3, eye: Vec3, d: Option<[f32; 3]>) -> f32 {
    let Some([lo, hi, at_max]) = d else { return 1.0 };
    let dist = (target - eye).length();
    if lo <= dist {
        if dist <= hi { 1.0 - (1.0 - at_max) * (dist - lo) / (hi - lo) } else { at_max }
    } else {
        1.0
    }
}

impl VsVehicleFollowCamera {
    /// VsVehicleFollowCamera.cpp 0x000696C0 — the zone ramp, 0..1 over `ZONE_DURATION`
    fn zone_ramp(&mut self, g: &FollowGlobals, dt: f32, in_zone: bool) -> f32 {
        let v = if in_zone {
            self.zone_time = (self.zone_time + dt).min(g.zone_duration);
            let v = self.zone_time / g.zone_duration;
            if g.zone_ramp_curve { v * v } else { v }
        } else {
            self.zone_time = (self.zone_time - dt).max(0.0);
            let v = self.zone_time / g.zone_duration;
            if g.zone_ramp_curve { 1.0 - (1.0 - v) * (1.0 - v) } else { v }
        };
        clamp01(v)
    }

    /// VsVehicleFollowCamera.cpp 0x000693A0 — the heading turned towards the velocity (and the lag / detached camera)
    fn trail_heading(&self, g: &FollowGlobals, t: &FollowTarget, signed: f32, vel: Vec3, flat_fwd: Vec3, look: Vec3) -> Vec3 {
        let mut w = clamp01((t.speed - g.vel_speed_min) / (g.vel_speed_max - g.vel_speed_min));
        let dir = if signed >= 0.0 {
            vel
        } else {
            w = 0.0;
            -vel
        };
        let w = (1.0 - g.heading_weight) * w;
        let lag = t.lag;
        if lag >= 0.01 {
            let want = lerp3(flat_fwd, dir, w);
            if lag == 1.0 { want } else { lerp3(look, want, lag) }
        } else {
            let a = (0.0 - t.look_around) * k::LOOK_AROUND;
            let (s, c) = (a.sin(), a.cos());
            Vec3::new(look.x * c + look.z * s, look.y, -look.x * s + look.z * c)
        }
    }

    /// VsVehicleFollowCamera.cpp 0x00067510 Update — `cut` = the manager's player record +40 (0x00069620; set by 0x0002C620, cleared 0x000699F0)
    pub fn update(&mut self, dt: f32, g: &FollowGlobals, t: &FollowTarget, cut: &mut bool) {
        let max_angle = rad(g.max_angle);
        let look_back = t.pad.buttons & PAD_LOOK_BACK != 0;
        if look_back != self.rear || self.cut_pending {
            *cut = true;
            self.cut_pending = false;
        }
        if !self.mirror && look_back != self.rear {
            self.rear = look_back;
            self.render_cut = true;
            // 0x0005DD30 with RotY(pi)
            let m = Mat3::from_rotation_y(std::f32::consts::PI);
            let look = (m * self.look).normalize_or_zero();
            let up = (m * self.up).normalize_or_zero();
            self.right = up.cross(look);
            self.up = look.cross(self.right);
            self.look = look;
        }
        let fwd = if self.rear { -t.heading } else { t.heading };
        let vel = if self.rear { -t.velocity } else { t.velocity };
        let flat_fwd = flat_normalised(fwd);
        let up0 = Vec3::Y;
        let (mut right, mut look);
        if *cut {
            self.render_cut = true;
            look = flat_fwd;
            right = up0.cross(look);
            let acc = t.pad.axis.max(t.pad.accelerator);
            let brk = (0.0 - t.pad.axis).max(t.pad.brake);
            self.pedal = if t.speed < 1.0 { 0.0 } else { acc - brk };
            *cut = false;
        } else {
            right = self.right;
            look = self.look;
        }

        // velocity on the ground, turned by the skid motion's extra yaw
        let mut flat_vel = Vec3::new(vel.x, 0.0, vel.z);
        let speed_xz = flat_vel.length();
        if self.view_index != 3 {
            let (s, c) = (t.extra_yaw.sin(), t.extra_yaw.cos());
            flat_vel = Vec3::new(flat_vel.x * c + flat_vel.z * s, 0.0, -flat_vel.x * s + flat_vel.z * c);
        }
        flat_vel = if speed_xz <= 0.1 { Vec3::ZERO } else { flat_vel / speed_xz };
        let along = flat_fwd.dot(flat_vel);

        // road slope pitch
        let mut slope = (fwd.y / (fwd.x * fwd.x + fwd.z * fwd.z).sqrt()).atan();
        if rad(k::SLOPE_DEAD) > slope.abs() {
            slope = 0.0;
        }
        slope = slope.clamp(rad(k::SLOPE_MIN), rad(k::SLOPE_MAX));
        let max = rad(k::SLOPE_RATE) * dt;
        self.slope += (slope - self.slope).clamp(0.0 - max, max);

        right = flat_normalised(right);
        look = flat_normalised(look);
        let speed = t.speed;

        // yaw spring towards the trailed heading
        let signed = if along < 0.0 { 0.0 - speed_xz } else { speed_xz };
        let want = self.trail_heading(g, t, signed, flat_vel, flat_fwd, look);
        let ang = want.dot(right).atan2(want.dot(look));
        let a = ang.abs();
        let mut yaw = 0.0f32;
        if rad(g.yaw_dead) <= a {
            if yaw <= max_angle && 0.0 - max_angle <= yaw {
                let rate = rad((g.yaw_gain * a).min(k::YAW_MAX_RATE));
                let full = rad(g.yaw_full);
                let r = if ang > full {
                    rate
                } else if 0.0 - full > ang {
                    0.0 - rate
                } else {
                    rate * (ang / full)
                };
                yaw = r * dt;
            }
        }

        // kite: the camera recedes on the throttle, closes in on the brakes
        let mut acc = t.pad.axis.max(t.pad.accelerator);
        let mut fury = t.fury;
        let mut brk = (0.0 - t.pad.axis).max(t.pad.brake);
        if t.pad.buttons & 2 != 0 {
            brk = 1.0;
        }
        if speed < 1.0 {
            brk = 0.0;
            fury = 0.0;
            acc = 0.0;
        }
        let pedal = acc - brk + fury;
        let d = pedal - self.pedal;
        self.pedal = pedal;
        if d < 0.0 {
            self.kite_input += d;
            self.kite_input = self.kite_input.max(0.0 - brk);
        } else if d > 0.0 {
            self.kite_input += d;
            self.kite_input = self.kite_input.min(acc);
        }
        self.kite_input = self.kite_input.clamp(-1.0, 1.0);
        if t.pad.steer.abs() > 0.8 {
            self.kite_input = 0.0;
        }
        if self.kite_input < 0.0 {
            if self.kite <= self.kite_input {
                self.kite *= g.kite_brake_decay;
                self.kite_input = self.kite;
            } else {
                self.kite -= dt / g.kite_brake_rise;
            }
        } else if self.kite_input <= self.kite {
            self.kite *= g.kite_accel_decay;
            self.kite_input = self.kite;
        } else {
            self.kite += dt / g.kite_accel_rise;
        }
        self.kite = self.kite.clamp(-1.0, 1.0);

        // low-speed pitch and the kite's pitch / distance
        let sp = clamp01((speed - g.pitch_speed_min) / (g.pitch_speed_max - g.pitch_speed_min));
        self.speed_pitch = clamp01((sp - self.speed_pitch) * g.pitch_speed_rate + self.speed_pitch);
        let half_pi = std::f32::consts::PI * 0.5;
        let kite_acc = (self.kite.max(0.0) * g.kite_accel_factor * half_pi).sin();
        let kite_brk = ((0.0 - self.kite).max(0.0) * g.kite_brake_factor * half_pi).sin();
        let p = clamp01(self.speed_pitch * g.kite_speed_factor + kite_acc);
        let mut pitch_deg = (1.0 - p) * g.kite_max_pitch;
        let kite_dist = g.kite_recede * kite_acc - g.kite_closer * kite_brk;

        // vertical bounce spring on the velocity's vertical change
        let vy = vel.y;
        let mut acc_y = (vy - self.last_vel_y) / dt;
        if k::BOUNCE_FALL_ONLY {
            acc_y = acc_y.min(0.0);
        }
        let f = ((0.0 - acc_y) - self.bounce) * g.bounce_rate;
        self.last_vel_y = vy;
        self.bounce_vel += f;
        self.bounce_vel *= g.bounce_damping;
        self.bounce += self.bounce_vel * dt;
        let bounce = soft_limit(self.bounce * g.bounce_amount, k::BOUNCE_KNEE, k::BOUNCE_SOFT_RATE, k::BOUNCE_LIMIT_NEG, k::BOUNCE_LIMIT_POS);
        if g.pitch_camera {
            pitch_deg += bounce;
        }

        // zone: pitch down (and trail / fov change, both 0 as shipped)
        let mut zone = self.zone_ramp(g, dt, t.in_zone);
        if let Some(x) = t.one_off {
            zone = zone * 1.0 - x * x;
        }
        let zone_pitch = (0.0 - g.zone_pitch_change) * zone;
        let (mut zone_trail, mut zone_fov) = (0.0, 0.0);
        if g.zone_use_trail {
            zone_trail = (0.0 - g.zone_trail_change) * zone;
        } else {
            zone_fov = (0.0 - g.zone_fov_change) * zone;
        }

        // basis: D3D RotX(pitch) * (rows(right, up, look) * RotY(yaw)), look = (0,0,1) through it
        let basis = Mat3::from_cols(right, up0, look);
        let pitch = rad(pitch_deg) + (rad(zone_pitch) + t.pitch_offset) - self.slope;
        let m = Mat3::from_rotation_y(yaw) * basis * Mat3::from_rotation_x(pitch);
        look = m * Vec3::Z;
        right = up0.cross(look);
        let up = look.cross(right);

        // fov from speed and fury
        let fs = clamp01((speed - g.fov_speed_min) / (g.fov_speed_max - g.fov_speed_min));
        let target = t.fury * g.fov_boost + lerp(g.fov_min, g.fov_max, fs);
        self.fov -= self.zone_fov;
        let d = target - self.fov;
        self.fov = if d <= 0.0 { d * g.fov_rate_down + self.fov } else { d * g.fov_rate_up + self.fov };
        self.fov += zone_fov;
        self.zone_fov = zone_fov;
        self.fov = vs_clamp(self.fov, g.fov_min - g.zone_fov_change, g.fov_max + g.fov_boost);
        self.fov_rad = rad(self.fov);

        // distance and eye
        let ft = clamp01((self.fov - g.fov_min) / (g.fov_max - g.fov_min));
        let mut dist = lerp(g.dist_near_fov, g.dist_far_fov, ft) + kite_dist + zone_trail;
        dist += t.trail_offset;
        if self.view_index == 1 {
            dist *= g.far_cam_trail;
        }
        let mut eye = t.position;
        if self.view_index <= 1 {
            eye += fwd * g.pivot;
            eye -= look * dist;
            eye -= flat_normalised(look) * g.pivot;
            eye.y += g.eye_y;
        }

        // point of interest
        if let Some(poi) = t.poi {
            let dir = (poi.target - eye).normalize_or_zero();
            look = if poi.out { dir * poi.t + flat_fwd * (1.0 - poi.t) } else { dir * poi.t + look * (1.0 - poi.t) };
            right = up0.cross(look);
            self.fov = self.view.fov * poi_fov_scale(poi.target, eye, poi.fov_distances);
            self.fov_rad = rad(self.fov);
        }
        eye.y += self.view.height;
        if !g.pitch_camera {
            eye.y += bounce * k::BOUNCE_LIFT;
        }
        self.right = right;
        self.up = up;
        self.look = look;
        self.eye = eye;
    }

    /// VsFlyCamera.cpp 0x00066940 — per update: player 0's view takes the tweak defaults, the chase camera
    /// (`update`) runs when the view is a chase view or a point of interest is up, the eye is pulled in front of
    /// track geometry (0x000692C0: `hit` = line check fraction from the car point + 1 m up), then banking, shake
    /// and the bridge record (target = eye + look + shake, up banked). `rand` = VsRandRange.
    pub fn update_follow(
        &mut self,
        dt: f32,
        g: &FollowGlobals,
        t: &FollowTarget,
        x: &FollowExtras,
        cut: &mut bool,
        hit: &mut dyn FnMut(Vec3, Vec3) -> Option<f32>,
        rand: &mut dyn FnMut(f32, f32) -> f32,
    ) -> FollowOut {
        let mut out = FollowOut::default();
        if self.player == 0 && !self.mirror {
            // 0x00066830: the tweak globals, which 0x000668C0 filled from player 0's view at init / view change
            let w = x.tweaks;
            self.view.height = w.height;
            self.view.look_height = w.look_height;
            self.view.dist_a = w.dist_a;
            self.view.dist_b = w.dist_b;
            self.view.fov = w.fov;
        }
        let dynamic = self.view.chase != 0.0 || x.poi_active;
        if self.dynamic != dynamic {
            self.render_cut = true;
            *cut = true;
        }
        self.dynamic = dynamic;
        let mut up;
        if dynamic {
            self.unknown_585 = self.view.chase == 0.0;
            self.update(dt, g, t, cut);
            if let Some(p) = t.poi {
                out.poi_done = p.out && p.t.abs() <= 0.000001;
            }
            let from = t.position + Vec3::Y;
            if let Some(f) = hit(from, self.eye) {
                self.eye = from + (self.eye - from) * f;
            }
            up = self.up;
        } else {
            // looking back (+585 = pad bit 0) or the mirror: along the heading's reverse, else the rendered nose
            self.unknown_585 = t.pad.buttons & PAD_LOOK_BACK != 0;
            let flag = self.rear || self.unknown_585;
            let dir = if flag { -t.heading } else { t.bridge_forward };
            let point = Vec3::new(dir.x * 20.0 + t.position.x, dir.y * 20.0 + t.position.y, dir.z * 20.0 + t.position.z);
            up = t.car_up;
            let (target, eye) = self.update_static(t.position, point, up, flag, t.fury, t.fury_fov, t.in_car_view);
            self.look_at(eye, target, up);
        }
        let speed = t.speed;
        if dynamic && g.banking {
            let sf = clamp01((speed - g.bank_speed_min) / (g.bank_speed_max - g.bank_speed_min));
            let nf = clamp01((t.fury + 1.0) / (g.bank_nos_factor + 1.0));
            let mut a = t.pad.steer.abs() * (rad(g.bank_max_angle) * (sf * nf));
            if self.look.cross(t.heading).y < 0.0 {
                a *= -1.0;
            }
            self.bank += (a - self.bank) * g.bank_lag;
            let axis = self.look.normalize_or_zero();
            if axis != Vec3::ZERO {
                up = Mat3::from_axis_angle(axis, self.bank) * up;
            }
        }
        let mut s = clamp01((speed - g.shake_speed_min) / (g.shake_speed_max - g.shake_speed_min));
        if x.sliding {
            s *= g.shake_drift;
        }
        if self.view_index <= 1 {
            s *= g.shake_3rd_person;
        }
        let z = rand(0.0 - g.shake_h, g.shake_h);
        let y = rand(0.0 - g.shake_v, g.shake_v);
        let xx = rand(0.0 - g.shake_h, g.shake_h);
        let mut shake = Vec3::new(xx, y, z) * s;
        if let Some(i) = x.impact.filter(|i| i.amount != 0.0) {
            let r1 = rand(0.0 - k::IMPACT_UP, k::IMPACT_UP);
            let r2 = rand(0.0 - k::IMPACT_SIDE, k::IMPACT_SIDE);
            let f = 1.0 - (x.game_time - i.start) / (i.end - i.start);
            let mut amount = i.base * f;
            let v = (i.up * r1 * k::IMPACT_BLEND + i.side * r2 * (1.0 - k::IMPACT_BLEND)) * amount;
            if amount < 0.0 || x.game_time > i.end {
                amount = 0.0;
            }
            out.impact_amount = Some(amount);
            shake += v;
        }
        if g.z_rotation != 0.0 {
            up = Mat3::from_rotation_z(rad(g.z_rotation)) * up;
        }
        self.bridge = Some(CameraBridgeData { position: self.eye, target: self.eye + self.look + shake, up, fov: self.fov_rad });
        out
    }

    /// VsVehicleFollowCamera.cpp 0x000690B0 — the static camera: from the car point `car` towards `point` (20 m along the look), eye pulled
    /// back by the rear (`flag`) or bumper offset and up by the view height, target lifted by the interest height;
    /// fov = the view's, widened by fury (`flt_686B5C` · [16]) unless in the in-car view. Returns (target, eye).
    #[allow(clippy::too_many_arguments)]
    pub fn update_static(&mut self, car: Vec3, point: Vec3, up: Vec3, flag: bool, fury: f32, fury_fov: f32, in_car: bool) -> (Vec3, Vec3) {
        let d = super::vs_base_camera::normalise(car - point);
        let dist = if flag { self.view.dist_a } else { self.view.dist_b };
        let eye = (car - d * dist) + up * self.view.height;
        let target = point + up * self.view.look_height;
        let fov = if in_car {
            self.view.fov
        } else {
            // CRim 0x000452C0 lerp
            let hi = fury_fov * self.view.unknown_532 + self.view.fov;
            (hi - self.view.fov) * fury + self.view.fov
        };
        let r = rad(fov);
        if r >= 3.0 {
            log::error!("assert: fNewFOV<3.0f");
        }
        self.fov_rad = r;
        (target, eye)
    }

    /// VsBaseCamera.cpp 0x0005DB10 — look from `eye` at `target` (dir kept when they coincide), right = up × dir,
    /// up = dir × right (not normalised)
    pub fn look_at(&mut self, eye: Vec3, target: Vec3, up: Vec3) {
        self.eye = eye;
        if target != eye {
            self.look = target - eye;
        }
        self.look = super::vs_base_camera::normalise(self.look);
        self.right = up.cross(self.look);
        self.up = self.look.cross(self.right);
    }
}

/// `VehicleEntity` 0x001C5CF0, camera half — the point the chase camera follows: the car position pushed
/// forward (`camera_shift_forward`), sideways along the camera while skidding (VE+4456 on: VE+4468 along the
/// look, VE+4476 along the right; not on view 3), up by VE+4392, the VE+4396 offset while VE+4408 counts, then
/// the vertical inertia spring (CVI_*) on its height, whose state lives in VE+4436 / +4440 / +4444.
pub mod camera_point {
    use bevy::math::Vec3;

    /// the VE / config values the function reads
    #[derive(Clone, Copy, Debug, Default)]
    pub struct CameraPointIn {
        pub position: Vec3,
        pub heading: Vec3,
        pub up: Vec3,
        pub shift_forward: f32,
        /// VE+4456 / +4468 / +4476
        pub skid: f32,
        pub skid_look: f32,
        pub skid_right: f32,
        /// VE+4392
        pub up_offset: f32,
        /// VE+4408 != 0 → VE+4396..4404
        pub offset: Option<Vec3>,
        /// VE+3840 (landing frames left)
        pub landing: bool,
        /// *(VE+3552) +0
        pub lc0: f32,
        /// VE+276
        pub ve_276: f32,
    }

    /// CVI_* of `::GC` (+0x5B4..+0x5D0) and the landing multipliers (+0x454 / +0x458)
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Cvi {
        pub inertia_reference_frame: f32,
        pub tau_up: f32,
        pub tau_down: f32,
        pub scale_up: f32,
        pub scale_down: f32,
        pub max_up: f32,
        pub max_down: f32,
        pub stop_decay: f32,
        pub landing_tau: f32,
        pub landing_limits: f32,
    }

    /// VE+4436 (rate), +4440 (height), +4444
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct CviState {
        pub rate: f32,
        pub height: f32,
        pub frame: f32,
    }

    /// `look` / `right` = the camera's +280 / +248 from its last update
    pub fn camera_point(i: &CameraPointIn, g: &Cvi, st: &mut CviState, look: Vec3, right: Vec3, view: u32) -> Vec3 {
        let mut p = i.position;
        p += i.heading * i.shift_forward;
        if i.skid != 0.0 && view != 3 {
            p += look * i.skid_look;
            p += right * i.skid_right;
        }
        p += i.up * i.up_offset;
        if let Some(o) = i.offset {
            p.x += o.x;
            p.y += o.y;
            p.z += o.z;
        }
        if g.tau_up > 0.0 {
            let y = p.y;
            let d = st.height - y;
            let rate = st.rate;
            let (mut tau, scale, rising) = if d >= 0.0 { (g.tau_down, g.scale_down, false) } else { (g.tau_up, g.scale_up, true) };
            let lim = if i.landing {
                tau *= g.landing_tau;
                g.landing_limits
            } else {
                1.0
            };
            let a = 1.0 / tau;
            let e = (0.0 - a).exp();
            let n = ((a + 1.0) * d + rate * i.lc0) * e;
            st.rate = ((1.0 - a) * rate - (d * a) / (tau * i.lc0)) * e;
            st.height += n - d;
            if rising {
                let over = y - st.height;
                if over > g.max_up * lim {
                    st.height += (over - g.max_up * lim) * g.stop_decay;
                    st.rate = 0.0;
                }
            } else {
                let over = st.height - y;
                if over > g.max_down * lim {
                    st.height -= (over - g.max_down * lim) * g.stop_decay;
                    st.rate = 0.0;
                }
            }
            let moved = st.height - y;
            if g.inertia_reference_frame != 0.0 {
                st.height += st.frame * i.lc0 * g.inertia_reference_frame;
                st.frame = i.ve_276;
            }
            p.y = moved * scale + y;
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 24.0;

    fn drive(c: &mut VsVehicleFollowCamera, cut: &mut bool, pos: Vec3, heading: Vec3, vel: Vec3, throttle: f32) {
        let mut t = FollowTarget::new(pos, heading, vel, vel.length());
        t.pad.accelerator = throttle;
        c.update(DT, &FollowGlobals::default(), &t, cut);
    }

    #[test]
    fn trails_a_moving_car() {
        let mut c = VsVehicleFollowCamera::default();
        let mut cut = true;
        let mut pos = Vec3::ZERO;
        for _ in 0..240 {
            pos += Vec3::Z * 20.0 * DT;
            drive(&mut c, &mut cut, pos, Vec3::Z, Vec3::Z * 20.0, 1.0);
        }
        assert!(!cut);
        assert!(c.eye.z < pos.z - 2.0 && c.eye.z > pos.z - 8.0, "{} {}", c.eye, pos);
        assert!(c.eye.y > pos.y);
        assert!(c.look.z > 0.9);
        assert!(c.fov > 60.0 && c.fov < 80.0);
    }

    /// a car sliding: nose along +Z, sliding along a velocity 30 degrees to its right (+X side). The trailed
    /// heading = lerp(heading, velocity, 0.4) once over 6 m/s, so the look settles between the two, on the
    /// velocity's side, and the yaw spring turns at min(200·angle, 180) deg/s.
    #[test]
    fn slide_leads_the_look_towards_the_velocity() {
        let mut c = VsVehicleFollowCamera::default();
        let mut cut = true;
        let heading = Vec3::Z;
        let a = rad(30.0);
        let vel = Vec3::new(a.sin(), 0.0, a.cos()) * 20.0;
        drive(&mut c, &mut cut, Vec3::ZERO, heading, vel, 0.0);
        // first update after the cut: look starts on the heading and turns one step towards the velocity
        let yaw0 = c.look.x.atan2(c.look.z);
        assert!(yaw0 > 0.0, "{yaw0}");
        let mut pos = Vec3::ZERO;
        let mut last = yaw0;
        for _ in 0..480 {
            pos += vel * DT;
            let y = c.look.x.atan2(c.look.z);
            assert!(y >= last - 1e-6, "{y} {last}");
            last = y;
            drive(&mut c, &mut cut, pos, heading, vel, 0.0);
        }
        let yaw = c.look.x.atan2(c.look.z);
        // want = normalise-free lerp: (0,0,1) + (v - (0,0,1)) * 0.4
        let want = Vec3::Z + (Vec3::new(a.sin(), 0.0, a.cos()) - Vec3::Z) * 0.4;
        let want_yaw = want.x.atan2(want.z);
        assert!(yaw > 0.0 && yaw < a, "{yaw} {a}");
        assert!((yaw - want_yaw).abs() < rad(k::YAW_DEAD) + 0.002, "{yaw} {want_yaw}");
        // the eye sits behind the look, on the opposite side of the slide
        assert!(c.eye.x < pos.x, "{} {}", c.eye, pos);
        // below 2 m/s the velocity carries no weight: the look stays on the heading
        let mut c2 = VsVehicleFollowCamera::default();
        let mut cut2 = true;
        for _ in 0..48 {
            drive(&mut c2, &mut cut2, Vec3::ZERO, heading, Vec3::new(a.sin(), 0.0, a.cos()) * 1.5, 0.0);
        }
        assert!(c2.look.x.abs() < 1e-4, "{}", c2.look);
    }

    /// fov target = lerp(60, 80, clamp((v - 15) / 25)) approached at 0.15 per update (0.05 down), distance
    /// = lerp(4.1, 3.6, (fov - 60) / 20) + kite; pitch (1 - clamp(speed_pitch·0.5 + kite)) · 4 degrees
    #[test]
    fn fov_and_distance_follow_the_speed() {
        let settle = |v: f32| {
            let mut c = VsVehicleFollowCamera::default();
            let mut cut = true;
            for _ in 0..480 {
                drive(&mut c, &mut cut, Vec3::ZERO, Vec3::Z, Vec3::Z * v, 0.0);
            }
            c
        };
        let slow = settle(10.0);
        let mid = settle(27.5);
        let fast = settle(50.0);
        assert!((slow.fov - 60.0).abs() < 1e-3, "{}", slow.fov);
        assert!((mid.fov - 70.0).abs() < 1e-2, "{}", mid.fov);
        assert!((fast.fov - 80.0).abs() < 1e-2, "{}", fast.fov);
        assert!((fast.fov_rad - rad(80.0)).abs() < 1e-4);
        // horizontal distance from the car point: dist·cos(pitch) + pivot·(1 - 1) along the heading
        let horiz = |c: &VsVehicleFollowCamera| -c.eye.z;
        let pitch = |c: &VsVehicleFollowCamera| (-c.look.y).asin();
        let d_slow = 4.1f32;
        let d_fast = 3.6f32;
        assert!((horiz(&slow) - d_slow * pitch(&slow).cos()).abs() < 1e-3, "{} {}", horiz(&slow), pitch(&slow));
        assert!((horiz(&fast) - d_fast * pitch(&fast).cos()).abs() < 1e-3, "{} {}", horiz(&fast), pitch(&fast));
        assert!(horiz(&fast) < horiz(&mid) && horiz(&mid) < horiz(&slow));
        // pitch: 4 degrees at 10 m/s with speed_pitch -> 1: (1 - 0.5) * 4 = 2 degrees
        assert!((pitch(&slow) - rad(2.0)).abs() < 1e-3, "{}", pitch(&slow).to_degrees());
        // the eye height = car + height 1.8 - 0.45 - dist·sin(-pitch)
        assert!((slow.eye.y - (1.8 - 0.45 + d_slow * pitch(&slow).sin())).abs() < 1e-3, "{}", slow.eye.y);
    }

    /// throttle pressed: the kite rises at dt / 0.7 and the camera recedes by 0.75·sin(kite·pi/2); once it
    /// reaches the pedal it decays by 0.95 per update
    #[test]
    fn throttle_kite_recedes() {
        let mut c = VsVehicleFollowCamera::default();
        let mut cut = true;
        drive(&mut c, &mut cut, Vec3::ZERO, Vec3::Z, Vec3::Z * 12.0, 0.0);
        for _ in 0..12 {
            drive(&mut c, &mut cut, Vec3::ZERO, Vec3::Z, Vec3::Z * 12.0, 1.0);
        }
        assert!((c.kite - 12.0 * DT / k::KITE_ACCEL_RISE).abs() < 1e-4, "{}", c.kite);
        let pitch = (-c.look.y).asin();
        let horiz = -c.eye.z;
        let expect = (4.1 + k::KITE_RECEDE * (c.kite * std::f32::consts::FRAC_PI_2).sin()) * pitch.cos();
        assert!((horiz - expect).abs() < 1e-3, "{horiz} {expect}");
    }

    #[test]
    fn soft_limit_matches_68fa0() {
        assert_eq!(soft_limit(3.0, 0.5, 2.0, 20.0, 20.0), 3.0);
        // at 30: t = (30 - 10) * 2 = 40, f = 1 - 1/41, (0.5 f + 0.5) * 20
        let v = soft_limit(30.0, 0.5, 2.0, 20.0, 20.0);
        assert!((v - (0.5 * (1.0 - 1.0 / 41.0) + 0.5) * 20.0).abs() < 1e-5);
        assert!(soft_limit(-30.0, 0.5, 2.0, 20.0, 20.0) < -19.0);
    }

    #[test]
    fn static_view_sits_on_the_car() {
        // a bumper view ([14] = 0): eye along the nose by the bumper offset, up by the height, looking ahead
        let view = CameraView { height: 0.7, look_height: 0.7, dist_a: -2.0, dist_b: 1.5, fov: 60.0, chase: 0.0, unknown_532: 1.0, ..Default::default() };
        let mut c = VsVehicleFollowCamera { view, player: 1, view_index: 2, dynamic: true, ..Default::default() };
        let mut cut = false;
        let car = Vec3::new(10.0, 2.0, -5.0);
        let mut t = FollowTarget::new(car, Vec3::Z, Vec3::Z * 20.0, 20.0);
        let x = FollowExtras::default();
        let mut hit = |_: Vec3, _: Vec3| None;
        let mut zero = |_: f32, _: f32| 0.0;
        let g = FollowGlobals::default();
        c.update_follow(DT, &g, &t, &x, &mut cut, &mut hit, &mut zero);
        assert!(c.render_cut && cut, "chase -> static cuts");
        assert!((c.eye - (car + Vec3::Z * 1.5 + Vec3::Y * 0.7)).length() < 1e-5);
        assert!((c.look - Vec3::Z).length() < 1e-5);
        assert!((c.fov_rad - rad(60.0)).abs() < 1e-6);
        let b = c.bridge.unwrap();
        assert!((b.target - (c.eye + c.look)).length() < 1e-5);
        // looking back: rear offset, facing backwards; fury widens the fov by 20 · [16]
        t.pad.buttons = PAD_LOOK_BACK;
        t.fury = 0.5;
        c.update_follow(DT, &g, &t, &x, &mut cut, &mut hit, &mut zero);
        assert!((c.eye - (car - Vec3::Z * -2.0 + Vec3::Y * 0.7)).length() < 1e-5);
        assert!((c.look + Vec3::Z).length() < 1e-5);
        assert!((c.fov_rad - rad(70.0)).abs() < 1e-6);
        // the in-car view keeps the view's fov
        t.in_car_view = true;
        c.update_follow(DT, &g, &t, &x, &mut cut, &mut hit, &mut zero);
        assert!((c.fov_rad - rad(60.0)).abs() < 1e-6);
    }
}
