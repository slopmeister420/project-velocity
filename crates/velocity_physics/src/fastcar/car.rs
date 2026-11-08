//! FastCar `CFastCar` (unattributed object 0x35DD40-0x369640, 0x36E7B0-0x372B70): a chassis rigid
//! body with four suspended wheels. Members are kept by their original byte offsets (see [`off`]).
//! Wheel order: 0 and 3 are front (left/right), 1 and 2 rear; body axes x forward, y up, z right.

use super::mem::{cross, madd, mul_cols, Mem, V3};
use super::rigid_body::{self as rb, off as rbo, RigidBody};
use super::vsmath;

/// byte offsets of the `CFastCar` members (beyond the `CRigidBody` base)
pub mod off {
    /// contact records, `REC + 136 * (wheel + k)` (the library indexes them this way)
    pub const REC: usize = 544;
    pub const REC_SIZE: usize = 136;
    // record fields
    pub const R_FORCE: usize = 0; // V3
    pub const R_FN: usize = 16;
    pub const R_FT1: usize = 20;
    pub const R_FT2: usize = 24;
    pub const R_NORMAL: usize = 28; // V3
    pub const R_POINT: usize = 40; // V3
    pub const R_DEPTH: usize = 52;
    pub const R_REL_POS: usize = 56; // V3
    pub const R_REL_VEL: usize = 68; // V3
    pub const R_T1: usize = 84; // V3
    pub const R_T2: usize = 100; // V3
    pub const R_VN: usize = 116;
    pub const R_VT1: usize = 120;
    pub const R_VT2: usize = 124;
    pub const R_MU: usize = 128;
    pub const R_SLIDING: usize = 132;

    pub const NCONTACT: usize = 1088; // i32[4]
    /// wheel collision callback `void cb(CFastCar*)` (VehicleEntity::WheelCollision)
    pub const CD_CALLBACK: usize = 1104;
    pub const DYNAMIC_VELOCITY: usize = 1116;
    pub const SLIP_COUNTER: usize = 1120;
    pub const SPRING: usize = 1124; // [4]
    pub const DAMPER: usize = 1140; // [4]
    pub const LOWER_LIMIT: usize = 1156; // [4]
    pub const UPPER_LIMIT: usize = 1172; // [4]
    pub const SUSP_GEOMETRY_FRONT: usize = 1188;
    pub const SUSP_GEOMETRY_BACK: usize = 1192;
    pub const TOPPLE_CORRECTION: usize = 1196;
    pub const WHEEL_DYNAMICS_TYPE: usize = 1200;
    pub const DRIVE_TORQUE: usize = 1204; // [4]
    pub const BRAKE_TORQUE: usize = 1220; // [4]
    pub const WHEEL_LOCKED: usize = 1236; // i32[4]
    pub const WHEEL_FLAG2: usize = 1252; // i32[4]
    pub const MU: usize = 1268; // [4]
    pub const LATERAL_RATIO: usize = 1284; // [4]
    pub const STATIC_DYNAMIC_RATIO: usize = 1300; // [4]
    pub const IS_PLAYER: usize = 1316;
    pub const FLAG_1320: usize = 1320;
    pub const FLAG_1324: usize = 1324;
    pub const MERGE_CONTACTS: usize = 1328;
    pub const RELAXATION_STEPS: usize = 1332;
    pub const FLAG_1336: usize = 1336;
    pub const FLAG_1340: usize = 1340;
    pub const FLAG_1344: usize = 1344;
    pub const FLAG_1348: usize = 1348;
    pub const PVP: usize = 1352;
    pub const PVP_RELAXATION: usize = 1356;
    pub const PVP_INTO_TURN: usize = 1360;
    pub const PVP_STRAIGHTEN: usize = 1364;
    pub const SKID_ANGLE: usize = 1368;
    pub const VISUAL_STEER: usize = 1372;
    pub const VISUAL_STEER_RATE: usize = 1376;
    pub const VISUAL_STEER_MAX: usize = 1380;
    pub const VISUAL_STEER_GAIN: usize = 1384;
    pub const WHEEL_ANGLE: usize = 1416; // [4]
    pub const WHEEL_OMEGA: usize = 1432; // [4]
    pub const CHASSIS_TOI: usize = 1448; // V3
    pub const CHASSIS_MASS: usize = 1464;
    pub const WHEEL_INV_TOI: usize = 1468; // [4] x V4
    pub const WHEEL_MASS: usize = 1532; // [4]
    pub const WHEEL_MATRIX: usize = 1548; // [4] x 16
    pub const RADIUS: usize = 1804; // [4]
    pub const WHEEL_POS: usize = 1820; // [4] x V4, body frame
    pub const STEERING: usize = 1884;
    pub const WHEEL_STEER: usize = 1888; // [4]
    pub const AXIS_W: usize = 1904; // [4] x V4
    pub const WHEEL_POS_W: usize = 1968; // [4] x V4
    pub const WHEEL_VEL_W: usize = 2032; // [4] x V4
    pub const WHEEL_OMEGA_W: usize = 2096; // [4] x V4
    pub const WHEEL_TORQUE: usize = 2160; // [4]
    pub const WHEEL_TORQUE_KEEP: usize = 2176; // [4]
    pub const WHEEL_2192: usize = 2192; // [4]
    pub const IN_CD: usize = 2240;
    pub const FLAG_2244: usize = 2244;
    pub const SUSP_LIMITS_CH: usize = 2248;
    pub const SUSP_OFFSET: usize = 2252; // V3
    pub const SUSP_X: usize = 2268; // [4] compression state
    pub const SUSP_2284: usize = 2284; // [4]
    pub const SUSP_REST: usize = 2300; // [4]
    pub const SUSP_PREV: usize = 2316; // [4]
    pub const ROLL_FRONT: usize = 2332;
    pub const ROLL_BACK: usize = 2336;
    pub const WHEEL_DIR: usize = 2348; // [4] x V4
    pub const WHEEL_DIR_SET: usize = 2412; // i32[4]
    pub const NET_TORQUE: usize = 2428; // [4]
    pub const STATIC_LOAD: usize = 2444; // [4]
    pub const EFF_MASS: usize = 2460; // [4]
    pub const EFF_2476: usize = 2476; // [4]
    pub const EFF_2492: usize = 2492; // [4]
    pub const EFF_2508: usize = 2508; // [4]
    pub const ALL_ON_GROUND_FAIL: usize = 2524;
    pub const SLIP: usize = 2528; // [4]
    pub const MAX_SLIP_SPEED: usize = 2544; // [4]
    pub const H: usize = 2560;
    pub const WHEEL_FREE: usize = 2564; // i32[4]
    pub const TORQUE_DIR: usize = 2580; // [4] x V4
    pub const SOLVER: usize = 2644; // 36 floats
    pub const TWO_WHEELS: usize = 2840;
    pub const BALANCE_PAIR_A: usize = 2844;
    pub const BALANCE_PAIR_B: usize = 2848;
    pub const BALANCE_ANGLE: usize = 2852; // [4]
    pub const BALANCE_TIME_1: usize = 2868;
    pub const BALANCE_TIME_2: usize = 2872;
    pub const AIRBORNE: usize = 2876;
    pub const BALANCE_FLAG_2880: usize = 2880;
    pub const BALANCE_STATE: usize = 2884;
    pub const BALANCE_MAX_OMEGA: usize = 2888;
    pub const BALANCE_MAX_ANGLE: usize = 2892;
    pub const BALANCE_MIN_2896: usize = 2896;
    pub const BALANCE_MIN_2900: usize = 2900;
    pub const BALANCE_RELAX: usize = 2904;
    pub const BALANCE_PROJECT: usize = 2908;
    pub const SPIN_ACCUM: usize = 2912;
    pub const FLAG_2964: usize = 2964;
    pub const COUNTDOWN_2968: usize = 2968;
    /// flat-ground height used by `flat_ground_contacts`
    pub const GROUND_HEIGHT: usize = 2836;
    /// through the engine/gearbox members (car+3296); the entity resumes at car+3300
    pub const SIZE: usize = 3300;
}
use off::*;

#[inline]
fn rec(i: usize, k: usize) -> usize {
    REC + REC_SIZE * (i + k)
}

/// Wheel collision callback (`this+1104`), called with the car during `DetectWheelContacts`.
pub type CdCallback = Box<dyn FnMut(&mut CFastCar) + Send + Sync>;

/// `CFastCar` (vtable 0x675854).
pub struct CFastCar {
    pub m: Mem,
    pub cd: Option<CdCallback>,
}

impl std::fmt::Debug for CFastCar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CFastCar").field("position", &self.m.v3(rbo::POSITION)).finish()
    }
}

impl RigidBody for CFastCar {
    fn mem(&self) -> &Mem {
        &self.m
    }
    fn mem_mut(&mut self) -> &mut Mem {
        &mut self.m
    }
    /// (FastCar) 0x00362930
    fn update_derived(&mut self) {
        self.update_wheel_frames();
        self.m.set_i(rbo::DIRTY, 0);
    }
    /// (FastCar) 0x003652D0
    fn evolve(&mut self, h: f32, keep_impulses: bool) {
        self.evolve_car(h, keep_impulses);
    }
    /// (FastCar) 0x0035FC60 — chassis matrix, translated by the suspension offset when enabled
    fn matrix(&self) -> [f32; 16] {
        let m = &self.m;
        let mut o = [0.0f32; 16];
        for r in 0..3 {
            o[4 * r..4 * r + 3].copy_from_slice(&m.v3(rbo::ROTATION + 16 * r));
        }
        let p = if m.i(FLAG_2244) != 0 {
            let a = m.v3(rbo::POSITION);
            let b = m.v3(SUSP_OFFSET);
            [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
        } else {
            m.v3(rbo::POSITION)
        };
        o[12..15].copy_from_slice(&p);
        o[15] = 1.0;
        o
    }
    /// (FastCar) 0x003616D0
    fn position(&self) -> V3 {
        let a = self.m.v3(rbo::POSITION);
        let b = self.m.v3(SUSP_OFFSET);
        [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
    }
}

impl Default for CFastCar {
    fn default() -> Self {
        Self::new()
    }
}

impl CFastCar {
    /// (FastCar) 0x0035DD40 — constructor
    pub fn new() -> Self {
        let mut c = CFastCar { m: Mem::new(SIZE), cd: None };
        rb::rb_construct(&mut c.m);
        let m = &mut c.m;
        m.set_v4(WHEEL_ANGLE, [0.0; 4]);
        for i in 0..4 {
            m.set_f(WHEEL_TORQUE_KEEP + 4 * i, 0.0);
            m.set_u(WHEEL_TORQUE + 4 * i, m.u(WHEEL_TORQUE_KEEP + 4 * i));
            m.set_f(WHEEL_2192 + 4 * i, 0.0);
        }
        for o in [2208, 2224, DRIVE_TORQUE, BRAKE_TORQUE, LOWER_LIMIT, UPPER_LIMIT] {
            m.set_v4(o, [0.0; 4]);
        }
        for i in 0..8 {
            m.set_i(WHEEL_LOCKED + 4 * i, 0);
        }
        m.set_i(IS_PLAYER, 1);
        m.set_i(FLAG_1320, 1);
        m.set_i(FLAG_2244, 1);
        m.set_i(SUSP_LIMITS_CH, 0);
        m.set_i(FLAG_1336, 0);
        m.set_i(FLAG_1340, 0);
        m.set_i(FLAG_1324, 0);
        m.set_i(FLAG_1344, 1);
        m.set_i(IN_CD, 0);
        m.set_v4(MU, [1.0; 4]);
        m.set_v4(LATERAL_RATIO, [1.0; 4]);
        m.set_v4(STATIC_DYNAMIC_RATIO, [1.5; 4]);
        m.set_f(DYNAMIC_VELOCITY, 1.0);
        m.set_i(SLIP_COUNTER, 0);
        m.set_f(RELAXATION_STEPS, 1.0);
        m.set_u(CD_CALLBACK, 0);
        m.set_i(1108, 0);
        m.set_i(1112, 0);
        m.set_f(STEERING, 0.0);
        m.set_v4(WHEEL_STEER, [0.0; 4]);
        m.set_v3(rbo::POSITION, [0.0; 3]);
        m.set_v4(WHEEL_ANGLE, [0.0; 4]);
        m.set_v3(rbo::OMEGA, [0.0; 3]);
        m.set_v3(rbo::VELOCITY, [0.0; 3]);
        m.set_v4(WHEEL_OMEGA, [0.0; 4]);
        m.set_i(FLAG_1348, 0);
        m.set_i(SUSP_GEOMETRY_FRONT, 0);
        m.set_i(SUSP_GEOMETRY_BACK, 0);
        m.set_i(TOPPLE_CORRECTION, 0);
        m.set_v3(SUSP_OFFSET, [0.0; 3]);
        m.set_v4(SUSP_X, [0.0; 4]);
        m.set_v4(SUSP_2284, [0.0; 4]);
        m.set_f(ROLL_FRONT, 0.0);
        m.set_f(ROLL_BACK, 0.0);
        m.set_i(MERGE_CONTACTS, 1);
        m.set_i(WHEEL_DYNAMICS_TYPE, 1);
        m.set_i(TWO_WHEELS, 0);
        m.set_v4(BALANCE_ANGLE, [0.0; 4]);
        m.set_f(BALANCE_TIME_1, 0.0);
        m.set_f(BALANCE_TIME_2, 0.0);
        m.set_f(BALANCE_RELAX, 1.0);
        m.set_i(BALANCE_STATE, 0);
        m.set_f(BALANCE_MAX_OMEGA, 0.0);
        m.set_u(BALANCE_MAX_ANGLE, 0x40490FDB);
        m.set_f(BALANCE_MIN_2896, 0.0);
        m.set_i(FLAG_2964, 0);
        m.set_i(BALANCE_PAIR_A, -1);
        m.set_f(PVP, 0.0);
        m.set_f(PVP_RELAXATION, 5.0);
        m.set_i(1388, 0);
        m.set_i(1392, 0);
        m.set_f(BALANCE_PROJECT, 0.0);
        m.set_i(SPIN_ACCUM, 0);
        m.set_i(BALANCE_FLAG_2880, 0);
        m.set_f(SKID_ANGLE, 0.0);
        for j in 0..4 {
            let o = WHEEL_MATRIX + 64 * j;
            m.set_v4(o, [1.0, 0.0, 0.0, 0.0]);
            m.set_v4(o + 16, [0.0, 1.0, 0.0, 0.0]);
            m.set_v4(o + 32, [0.0, 0.0, 1.0, 0.0]);
            m.set_v4(o + 48, [0.0, 0.0, 0.0, 1.0]);
        }
        m.set_f(VISUAL_STEER, 0.0);
        m.set_u(VISUAL_STEER_RATE, 1045220557);
        m.set_u(VISUAL_STEER_MAX, 1045220557);
        m.set_u(VISUAL_STEER_GAIN, 1067030938);
        m.set_i(COUNTDOWN_2968, 0);
        c
    }

    #[inline]
    pub fn n_contacts(&self, w: usize) -> i32 {
        self.m.i(NCONTACT + 4 * w)
    }

    /// (FastCar) 0x0035FA20 — `SetParameters` from spans: r0 = front-left wheel, r1 = rear-left wheel
    #[allow(clippy::too_many_arguments)]
    pub fn set_parameters_from_spans(
        &mut self,
        gravity: V3,
        span_front_back: f32,
        span_front: f32,
        span_back: f32,
        com_x: f32,
        com_y: f32,
        front_radius: f32,
        back_radius: f32,
        chassis_toi: V3,
        front_toi: V3,
        back_toi: V3,
        mass: f32,
        front_mass: f32,
        back_mass: f32,
    ) -> bool {
        let r0 = [span_front_back / 2.0 - com_x, 0.0 - com_y, span_front / 2.0];
        let r1 = [0.0 - (span_front_back / 2.0 + com_x), 0.0 - com_y, span_back / 2.0];
        self.set_parameters(gravity, r0, r1, front_radius, back_radius, chassis_toi, front_toi, back_toi, mass, front_mass, back_mass)
    }

    /// (FastCar) 0x0035E4D0
    #[allow(clippy::too_many_arguments)]
    pub fn set_parameters(
        &mut self,
        gravity: V3,
        r0: V3,
        r1: V3,
        front_radius: f32,
        back_radius: f32,
        a8: V3,
        a9: V3,
        a10: V3,
        a11: f32,
        a12: f32,
        a13: f32,
    ) -> bool {
        let glen = vsmath::sqrt(vsmath::dot87(gravity, gravity) as f32);
        if 0.0 == glen {
            self.report_error("gravity must have non-zero length.");
        }
        if a11 <= 0.0 || a12 <= 0.0 || a13 <= 0.0 {
            self.report_error("mass should be positive");
        }
        if a8[0] < 0.0 || a8[1] < 0.0 || a8[2] < 0.0 {
            self.report_error("chassis_TOI cannot be negative.");
        }
        if a9[0] < 0.0 || a9[1] < 0.0 || a9[2] < 0.0 {
            self.report_error("front_wheel_TOI cannot be negative.");
        }
        if a10[0] < 0.0 || a10[1] < 0.0 || a10[2] < 0.0 {
            self.report_error("back_wheel_TOI cannot be negative.");
        }
        let tri = |t: V3| t[2] > t[0] + t[1] || t[0] > t[1] + t[2] || t[1] > t[0] + t[2];
        if tri(a8) {
            self.report_error("chassis_TOI does not follow triangle rule: I1+I2>I3");
        }
        if tri(a9) {
            self.report_error("front_wheel_TOI does not follow triangle rule.");
        }
        if tri(a10) {
            self.report_error("back_wheel_TOI does not follow triangle rule.");
        }
        if r0[0] == 0.0 {
            self.report_error("r0[0] must not be zero.");
        }
        if r0[2] == 0.0 {
            self.report_error("r0[2] must not be zero.");
        }
        if r1[0] == 0.0 {
            self.report_error("r1[0] must not be zero.");
        }
        if r1[2] == 0.0 {
            self.report_error("r1[2] must not be zero.");
        }
        let k = 0.33333334f32 * a11;
        if k * (r1[1] * r1[1] + r1[2] * r1[2]) > a8[0] * 20.0 {
            self.report_warning("chassis_TOI[0] seems to be too small");
        }
        if a8[0] * 0.05f32 > k * (r1[1] * r1[1] + r1[2] * r1[2]) && self.m.i(TWO_WHEELS) != 0 {
            self.report_warning("chassis_TOI[0] seems to be too large");
        }
        if k * (r1[2] * r1[2] + r1[0] * r1[0]) > a8[1] * 20.0 {
            self.report_warning("chassis_TOI[1] seems to be too small");
        }
        if a8[1] * 0.05f32 > k * (r1[2] * r1[2] + r1[0] * r1[0]) {
            self.report_warning("chassis_TOI[1] seems to be too large");
        }
        if k * (r1[1] * r1[1] + r1[0] * r1[0]) > a8[2] * 20.0 {
            self.report_warning("chassis_TOI[2] seems to be too small");
        }
        if a8[2] * 0.05f32 > k * (r1[1] * r1[1] + r1[0] * r1[0]) {
            self.report_warning("chassis_TOI[2] seems to be too large");
        }
        if r0[0] * r1[0] >= 0.0 {
            self.report_warning("r0[0] and r1[0] should be of different signs.");
        }
        if r0[1] * r1[1] < 0.0 {
            self.report_warning("r0[1] and r1[1] should be of the same signs.");
        }
        if r0[2] * r1[2] <= 0.0 {
            self.report_warning("r0[2] and r1[2] should be of the same signs.");
        }
        let a0 = vsmath::fabs(r0[0]) as f32;
        let b0 = vsmath::fabs(r1[0]) * 0.5;
        if b0 > a0 as f64 || {
            let c = vsmath::fabs(r0[0]) as f32;
            let d = vsmath::fabs(r1[0]);
            c as f64 > d + d
        } {
            self.report_warning("Center of wheel_mass seems to lie too far from the geometric center of wheels.");
        }
        if a9[0] != a9[1] {
            self.report_warning("front_wheel_TOI[0] should be equal to front_wheel_TOI[1], front_wheel_TOI[0] used");
        }
        if a10[0] != a10[1] {
            self.report_warning("back_wheel_TOI[0] should be equal to back_wheel_TOI[1], back_wheel_TOI[0] used");
        }
        if a12 * 5.0 > a11 {
            self.report_warning("front_wheel_mass of wheel does not seem to be much smaller than mass of the chassis.");
        }
        if a13 * 5.0 > a11 {
            self.report_warning("back_wheel_mass of wheel does not seem to be much smaller than mass of the chassis.");
        }
        if a9[0] * 5.0 > a8[0] || a9[1] * 5.0 > a8[1] || a9[2] * 5.0 > a8[2] {
            self.report_warning("front_wheel_TOI of wheel does not seem to be much smaller than chassis_TOI of the chassis.");
        }
        if a10[0] * 5.0 > a8[0] || a10[1] * 5.0 > a8[1] || a10[2] * 5.0 > a8[2] {
            self.report_warning("back_wheel_TOI of wheel does not seem to be much smaller than chassis_TOI of the chassis.");
        }
        let m = &mut self.m;
        m.set_i(rbo::SPHERICAL, 0);
        m.set_v3(rbo::GRAVITY, gravity);
        m.set_v3(WHEEL_POS, r0);
        m.set_v3(WHEEL_POS + 48, [r0[0], r0[1], 0.0 - r0[2]]);
        m.set_v3(WHEEL_POS + 16, r1);
        m.set_v3(WHEEL_POS + 32, [r1[0], r1[1], 0.0 - r1[2]]);
        m.set_f(RADIUS, front_radius);
        m.set_f(RADIUS + 4, back_radius);
        m.set_f(RADIUS + 8, back_radius);
        m.set_f(RADIUS + 12, front_radius);
        m.set_v3(CHASSIS_TOI, a8);
        for (o, v) in [
            (1468, 1.0 / a9[0]),
            (1472, 1.0 / a9[0]),
            (1476, 1.0 / a9[2]),
            (1516, 1.0 / a9[0]),
            (1520, 1.0 / a9[0]),
            (1524, 1.0 / a9[2]),
            (1484, 1.0 / a10[0]),
            (1488, 1.0 / a10[0]),
            (1492, 1.0 / a10[2]),
            (1500, 1.0 / a10[0]),
            (1504, 1.0 / a10[0]),
            (1508, 1.0 / a10[2]),
        ] {
            m.set_f(o, v);
        }
        m.set_f(CHASSIS_MASS, a11);
        m.set_f(WHEEL_MASS, a12);
        m.set_f(WHEEL_MASS + 4, a13);
        m.set_f(WHEEL_MASS + 8, a13);
        m.set_f(WHEEL_MASS + 12, a12);
        let f = |o: usize| m.f(o);
        let i0 = (((((f(1824) * f(1824) + f(1828) * f(1828)) * 2.0) * f(1532) + f(1448))
            + ((f(1840) * f(1840) + f(1844) * f(1844)) * 2.0) * f(1536))
            + 2.0 / f(1468))
            + 2.0 / f(1484);
        let i1 = (((((f(1820) * f(1820) + f(1828) * f(1828)) * 2.0) * f(1532) + f(1452))
            + ((f(1836) * f(1836) + f(1844) * f(1844)) * 2.0) * f(1536))
            + 2.0 / f(1472))
            + 2.0 / f(1488);
        let i2 = (((f(1820) * f(1820) + f(1824) * f(1824)) * 2.0) * f(1532) + f(1456))
            + ((f(1836) * f(1836) + f(1840) * f(1840)) * 2.0) * f(1536);
        let total = ((2.0 * f(1532)) + f(1464)) + (2.0 * f(1536));
        m.set_f(96, i0);
        m.set_f(100, i1);
        m.set_f(104, i2);
        m.set_f(rbo::MASS, total);
        m.set_i(rbo::DIRTY, 1);
        for i in 0..4 {
            let v = if i != 0 && i != 3 {
                let half = m.f(rbo::MASS) * 0.5;
                let a = (vsmath::fabs(m.f(1820)) as f32) * half;
                let b = vsmath::fabs(m.f(1820)) as f32;
                (a as f64 / (vsmath::fabs(m.f(1836)) + b as f64)) as f32
            } else {
                let half = m.f(rbo::MASS) * 0.5;
                let a = (vsmath::fabs(m.f(1836)) as f32) * half;
                let b = vsmath::fabs(m.f(1820)) as f32;
                (a as f64 / (vsmath::fabs(m.f(1836)) + b as f64)) as f32
            };
            m.set_f(STATIC_LOAD + 4 * i, v);
        }
        let hm = |a: f32, b: f32| -> f32 { ((a as f64 * b as f64) / (a as f64 + b as f64)) as f32 };
        if m.i(FLAG_1344) != 0 {
            let mut minload = m.f(rbo::MASS);
            for i in 0..4 {
                if minload > m.f(STATIC_LOAD + 4 * i) {
                    minload = m.f(STATIC_LOAD + 4 * i);
                }
            }
            let _ = minload;
            for i in 0..4 {
                let px = m.f(WHEEL_POS + 16 * i);
                let pz = m.f(WHEEL_POS + 16 * i + 8);
                let mut a = m.f(104) / (px * px);
                let mut b = m.f(96) / (pz * pz);
                let sl = m.f(STATIC_LOAD + 4 * i);
                if sl * 2.0 > a {
                    m.set_f(104, ((sl * 2.0) / a) * m.f(104));
                }
                if sl * 2.0 > b {
                    m.set_f(96, ((sl * 2.0) / b) * m.f(96));
                }
                a = m.f(104) / (px * px);
                b = m.f(96) / (pz * pz);
                let c = hm(a, b);
                let v = hm(c, m.f(rbo::MASS));
                m.set_f(EFF_MASS + 4 * i, v);
            }
        } else {
            for i in 0..4 {
                let px = m.f(WHEEL_POS + 16 * i);
                let pz = m.f(WHEEL_POS + 16 * i + 8);
                let a = m.f(104) / (px * px);
                let b = m.f(96) / (pz * pz);
                let c = hm(a, b);
                let v = hm(c, m.f(rbo::MASS));
                m.set_f(EFF_MASS + 4 * i, v);
            }
        }
        for i in 0..4 {
            let px = m.f(WHEEL_POS + 16 * i);
            let py = m.f(WHEEL_POS + 16 * i + 4);
            let pz = m.f(WHEEL_POS + 16 * i + 8);
            let r = m.f(RADIUS + 4 * i);
            m.set_f(EFF_2476 + 4 * i, m.f(100) / (px * px));
            let d = 0.66f32 * r - py;
            if d != 0.0 {
                let v = hm(m.f(96) / (d * d), m.f(EFF_2476 + 4 * i));
                m.set_f(EFF_2476 + 4 * i, v);
            }
            let v = hm(m.f(rbo::MASS), m.f(EFF_2476 + 4 * i));
            m.set_f(EFF_2476 + 4 * i, v);
            m.set_f(EFF_2508 + 4 * i, m.f(100) / (pz * pz));
            let d = r - py;
            if d != 0.0 {
                let v = hm(m.f(104) / (d * d), m.f(EFF_2508 + 4 * i));
                m.set_f(EFF_2508 + 4 * i, v);
            }
            let v = hm(m.f(rbo::MASS), m.f(EFF_2508 + 4 * i));
            m.set_f(EFF_2508 + 4 * i, v);
            m.set_f(EFF_2492 + 4 * i, (1.0 / m.f(WHEEL_INV_TOI + 16 * i + 8)) / (r * r));
            let v = hm(m.f(EFF_2508 + 4 * i), m.f(EFF_2492 + 4 * i));
            m.set_f(EFF_2492 + 4 * i, v);
        }
        m.set_i(2788, 0);
        let s = self.default_springs(None);
        self.m.set_v4(SPRING, s);
        let d = self.critical_dampers();
        self.m.set_v4(DAMPER, d);
        self.m.set_v4(SUSP_PREV, [0.0; 4]);
        self.m.set_v4(SUSP_2284, [0.0; 4]);
        self.update_derived();
        true
    }

    /// (FastCar) 0x00364F80 — springs that carry the static load at a third of the radius (or `defl`)
    pub fn default_springs(&self, defl: Option<[f32; 4]>) -> [f32; 4] {
        let m = &self.m;
        let n: f64 = if m.i(TWO_WHEELS) != 0 { 2.0 } else { 4.0 };
        let g = m.v3(rbo::GRAVITY);
        let mut o = [0.0f32; 4];
        for (i, x) in o.iter_mut().enumerate() {
            let gl = vsmath::sqrt(vsmath::dot87(g, g) as f32);
            *x = match defl {
                Some(d) => (gl * m.f(STATIC_LOAD + 4 * i) as f64 * 4.0 / (n * d[i] as f64)) as f32,
                None => (gl * m.f(STATIC_LOAD + 4 * i) as f64 * 4.0 / (m.f(RADIUS + 4 * i) as f64 * 1.0 / 3.0 * n)) as f32,
            };
        }
        o
    }

    /// (FastCar) 0x00365080 — `2 sqrt(k m)`
    pub fn critical_dampers(&self) -> [f32; 4] {
        let mut o = [0.0f32; 4];
        for (i, x) in o.iter_mut().enumerate() {
            let v = vsmath::sqrt(self.m.f(SPRING + 4 * i) * self.m.f(EFF_MASS + 4 * i));
            *x = (v + v) as f32;
        }
        o
    }

    /// (FastCar) 0x00362490 — rigid-body derived data plus wheel axes, positions and velocities
    pub fn update_wheel_frames(&mut self) {
        rb::rb_update_world_inertia(&mut self.m);
        let m = &mut self.m;
        let r = m.m34(rbo::ROTATION);
        for i in 0..4 {
            let st = m.f(WHEEL_STEER + 4 * i);
            let dir = [(-vsmath::sin(st)) as f32, 0.0, vsmath::cos(st) as f32];
            m.set_v3(AXIS_W + 16 * i, mul_cols(&r, dir));
            m.set_v3(WHEEL_POS_W + 16 * i, mul_cols(&r, m.v3(WHEEL_POS + 16 * i)));
            let v = cross(m.v3(rbo::OMEGA), m.v3(WHEEL_POS_W + 16 * i));
            m.set_v3(WHEEL_VEL_W + 16 * i, v);
            let p = add(m.v3(WHEEL_POS_W + 16 * i), m.v3(rbo::POSITION));
            m.set_v3(WHEEL_POS_W + 16 * i, p);
            let v = add(m.v3(WHEEL_VEL_W + 16 * i), m.v3(rbo::VELOCITY));
            m.set_v3(WHEEL_VEL_W + 16 * i, v);
            let w = madd(m.v3(rbo::OMEGA), m.v3(AXIS_W + 16 * i), m.f(WHEEL_OMEGA + 4 * i));
            m.set_v3(WHEEL_OMEGA_W + 16 * i, w);
        }
    }

    /// (FastCar) 0x003623A0 — wrap to (-pi, pi]
    pub fn normalize_angle(a: f32) -> f32 {
        let tp = 6.2831855f32;
        let mut x = a - ((((a / tp) as i32) as f32) * tp);
        while x > 3.1415927f32 {
            x -= tp;
        }
        while 0.0 - 3.1415927f32 >= x {
            x += tp;
        }
        x
    }

    /// (FastCar) 0x00361FF0 — Ackermann steering of the front wheels
    pub fn set_steering(&mut self, angle: f32) {
        let m = &mut self.m;
        let mut a = angle;
        if a == 0.0 {
            m.set_f(STEERING, 0.0);
            m.set_f(WHEEL_STEER, 0.0);
            m.set_f(WHEEL_STEER + 12, 0.0);
        } else {
            a = Self::normalize_angle(a);
            if a <= 3.1415927f32 / 2.0 {
                if (0.0 - 3.1415927f32) / 2.0 > a {
                    a += 3.1415927f32;
                }
            } else {
                a -= 3.1415927f32;
            }
            m.set_f(STEERING, a);
            let l = m.f(1820) - m.f(1836);
            let t = vsmath::tan(a) as f32;
            if a <= 0.0 {
                m.set_f(WHEEL_STEER, a);
                let v = vsmath::atan((t * l) / ((2.0 * m.f(1828)) * t + l)) as f32;
                m.set_f(WHEEL_STEER + 12, v);
            } else {
                let v = vsmath::atan((t * l) / (l - (2.0 * m.f(1828)) * t)) as f32;
                m.set_f(WHEEL_STEER, v);
                m.set_f(WHEEL_STEER + 12, a);
            }
        }
        let r = m.m34(rbo::ROTATION);
        for i in [0usize, 3] {
            let st = m.f(WHEEL_STEER + 4 * i);
            let dir = [(-vsmath::sin(st)) as f32, 0.0, vsmath::cos(st) as f32];
            m.set_v3(AXIS_W + 16 * i, mul_cols(&r, dir));
            let w = madd(m.v3(rbo::OMEGA), m.v3(AXIS_W + 16 * i), m.f(WHEEL_OMEGA + 4 * i));
            m.set_v3(WHEEL_OMEGA_W + 16 * i, w);
        }
        m.set_i(rbo::DIRTY, 1);
    }

    /// (FastCar) 0x00362440
    pub fn steering(&self) -> f32 {
        self.m.f(STEERING)
    }

    /// (FastCar) 0x00361C00 — axial part of a torque goes to the wheel torque accumulator
    pub(crate) fn add_wheel_torque(&mut self, t: V3, w: usize) {
        let a = vsmath::dot87(self.m.v3(AXIS_W + 16 * w), t) as f32;
        self.m.set_f(WHEEL_TORQUE + 4 * w, self.m.f(WHEEL_TORQUE + 4 * w) + a);
    }

    /// (FastCar) 0x00361710 — force at a point of a wheel
    pub fn add_wheel_force_at(&mut self, f: V3, p: V3, w: usize) {
        if self.m.i(WHEEL_FREE + 4 * w) != 0 {
            rb::rb_add_force_at(&mut self.m, f, p);
            return;
        }
        let r = sub(p, self.m.v3(WHEEL_POS_W + 16 * w));
        let t = cross(r, f);
        let c = self.m.v3(WHEEL_POS_W + 16 * w);
        rb::rb_add_force_at(&mut self.m, f, c);
        self.add_wheel_torque(t, w);
    }

    /// (FastCar) 0x00361D40 — impulse version: axial part to `WHEEL_TORQUE_KEEP`, rest to the body
    pub fn add_wheel_impulse_torque(&mut self, t: V3, w: usize) {
        let ax = self.m.v3(AXIS_W + 16 * w);
        let a = vsmath::dot87(ax, t) as f32;
        self.m.set_f(WHEEL_TORQUE_KEEP + 4 * w, self.m.f(WHEEL_TORQUE_KEEP + 4 * w) + a);
        let rest = madd(t, ax, 0.0 - a);
        rb::rb_add_impulse(&mut self.m, rest);
    }

    /// (FastCar) 0x00364A90 — 1 any sliding contact, 0 touching, -1 none
    pub fn wheel_contact_state(&self, w: usize) -> i32 {
        let n = self.n_contacts(w);
        for k in 0..n.max(0) as usize {
            if self.m.i(rec(w, k) + R_SLIDING) != 0 {
                return 1;
            }
        }
        if n != 0 {
            0
        } else {
            -1
        }
    }

    /// (FastCar) 0x00364B20
    pub fn contact_state(&self) -> i32 {
        for i in 0..4 {
            if self.wheel_contact_state(i) == 1 {
                return 1;
            }
        }
        for i in 0..4 {
            if self.wheel_contact_state(i) == 0 {
                return 0;
            }
        }
        -1
    }

    /// (FastCar) 0x00364BB0
    pub fn wheel_touches(&self, w: usize) -> bool {
        self.n_contacts(w) > 0
    }

    /// (FastCar) 0x00364BE0
    pub fn touching_wheels(&self) -> i32 {
        (0..4).filter(|&i| self.n_contacts(i) > 0).count() as i32
    }

    /// (FastCar) 0x00364C70 — worst slip ratio below 2, or 2 when a wheel slides
    pub fn max_slip(&self) -> f32 {
        if self.contact_state() == 1 {
            return 2.0;
        }
        let mut v = -1.0f32;
        for i in 0..4 {
            let s = self.m.f(SLIP + 4 * i);
            if s > v && s < 2.0 {
                v = s;
            }
        }
        v
    }

    /// (FastCar) 0x00364D20 — remaining longitudinal grip torque of a wheel
    pub fn wheel_grip_torque(&self, w: usize) -> f64 {
        let m = &self.m;
        let r = rec(w, 0);
        if self.n_contacts(w) <= 0 || m.f(r + R_FN) <= 0.0 || self.wheel_contact_state(w) == -1 {
            return 0.0;
        }
        let mu = m.f(r + R_MU);
        let fnn = m.f(r + R_FN);
        let ft = m.f(r + R_FT1);
        let lr = m.f(LATERAL_RATIO + 4 * w);
        let d = (((mu * mu) * fnn) * fnn) - (((ft * ft) / lr) / lr);
        if m.i(WHEEL_DIR_SET + 4 * w) != 0 || d < 0.0 {
            vsmath::fabs(mu * fnn) * m.f(RADIUS + 4 * w) as f64
        } else {
            vsmath::sqrt(d) * m.f(RADIUS + 4 * w) as f64
        }
    }

    /// (FastCar) 0x003724B0 — turning radius of the current steering (signed)
    pub fn turning_radius(&self) -> f32 {
        let m = &self.m;
        if m.f(STEERING) == 0.0 {
            return 0.0;
        }
        let r = turning_radius_for(m.f(STEERING), m.f(1820), m.f(1828), 0.0 - m.f(1836)) as f32;
        if m.f(STEERING) > 0.0 {
            r
        } else {
            0.0 - r
        }
    }

    /// (FastCar) 0x003697C0
    pub(crate) fn wheel_omega_from_keep(&mut self) {
        for i in 0..4 {
            let v = self.m.f(2160 + 4 * i) * self.m.f(1476 + 16 * i);
            self.m.set_f(1400 + 4 * i, v);
        }
    }

    /// (FastCar) 0x00369640
    pub(crate) fn integrate_wheel_omega(&mut self, h: f32) {
        for i in 0..4 {
            if self.m.i(WHEEL_LOCKED + 4 * i) == 1 {
                self.m.set_f(WHEEL_OMEGA + 4 * i, 0.0);
            } else if self.m.i(WHEEL_FLAG2 + 4 * i) != 1 {
                let v = self.m.f(1400 + 4 * i) * h + self.m.f(WHEEL_OMEGA + 4 * i);
                self.m.set_f(WHEEL_OMEGA + 4 * i, v);
            }
        }
    }

    /// (FastCar) 0x003696F0 — wheel impulses into wheel angular velocity
    pub(crate) fn apply_wheel_impulses(&mut self) {
        for i in 0..4 {
            let v = self.m.f(2192 + 4 * i) * self.m.f(1476 + 16 * i) + self.m.f(WHEEL_OMEGA + 4 * i);
            self.m.set_f(WHEEL_OMEGA + 4 * i, v);
        }
    }

    /// (FastCar) 0x00364750 — clamp wheel speed and advance the wheel angles
    pub(crate) fn integrate_wheel_angles(&mut self, h: f32) {
        let lim = ((2.0 * 3.1415927f32) * 10.123457f32) / h;
        for i in 0..4 {
            let o = WHEEL_OMEGA + 4 * i;
            if self.m.f(o) > lim {
                self.m.set_f(o, lim);
            }
        }
        for i in 0..4 {
            let o = WHEEL_OMEGA + 4 * i;
            if 0.0 - lim > self.m.f(o) {
                self.m.set_f(o, 0.0 - lim);
            }
        }
        for i in 0..4 {
            let v = self.m.f(WHEEL_OMEGA + 4 * i) * h + self.m.f(WHEEL_ANGLE + 4 * i);
            self.m.set_f(WHEEL_ANGLE + 4 * i, v);
        }
        for i in 0..4 {
            let v = Self::normalize_angle(self.m.f(WHEEL_ANGLE + 4 * i));
            self.m.set_f(WHEEL_ANGLE + 4 * i, v);
        }
    }

    /// (FastCar) 0x00365250 — clear contacts and run the wheel collision callback
    pub fn detect_wheel_contacts(&mut self) {
        for i in 0..4 {
            self.m.set_i(NCONTACT + 4 * i, 0);
        }
        self.m.set_i(IN_CD, 1);
        // 0x003651F0: callback, else vtable[9] (empty in this build)
        if let Some(mut cb) = self.cd.take() {
            cb(self);
            if self.cd.is_none() {
                self.cd = Some(cb);
            }
        }
        self.m.set_i(IN_CD, 0);
    }
}

#[inline]
pub(crate) fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
#[inline]
pub(crate) fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// (FastCar) 0x003AEDD0 — turning radius for a steering angle and wheel geometry (x87 result)
pub fn turning_radius_for(a1: f32, a2: f32, a3: f32, a4: f32) -> f64 {
    let v9 = vsmath::sin(a1) as f32;
    let v7 = a3 * v9;
    let v6 = a4 + a2;
    let v8 = (vsmath::cos(a1) * v6 as f64 + v7 as f64) as f32;
    vsmath::sqrt(((4.0 * a4) * a4) + ((v8 * v8) / (v9 * v9)))
}

/// (FastCar) 0x0039D7B0 — box inertia
pub fn box_inertia(mass: f32, x: f32, y: f32, z: f32) -> V3 {
    let k = 0.083333336f32 * mass;
    [k * (y * y + z * z), k * (x * x + z * z), k * (x * x + y * y)]
}

/// (FastCar) 0x0039D8A0 — wheel (disc) inertia
pub fn wheel_inertia(mass: f32, radius: f32, width: f32) -> V3 {
    let a = (((0.25f32 * radius) * radius) + ((0.083333336f32 * width) * width)) * mass;
    [a, a, (((mass * 1.0) / 2.0) * radius) * radius]
}

impl CFastCar {
    /// (FastCar) 0x00368060 — wheel contacts against the horizontal plane `y = this+2836`
    pub fn flat_ground_contacts(&mut self) {
        let m = &mut self.m;
        let n = [0.0f32, 1.0, 0.0];
        for i in 0..4 {
            m.set_i(NCONTACT + 4 * i, 0);
            let r = REC + REC_SIZE * i;
            m.set_v3(r + R_NORMAL, n);
            let ax = m.v3(AXIS_W + 16 * i);
            let t1 = super::car_evolve::normalize_keep(cross(m.v3(r + R_NORMAL), ax));
            m.set_v3(r + R_T1, t1);
            let d = cross(m.v3(r + R_T1), ax);
            let p = madd(m.v3(WHEEL_POS_W + 16 * i), d, m.f(RADIUS + 4 * i));
            m.set_v3(r + R_POINT, p);
            let depth = -(vsmath::dot87(m.v3(r + R_POINT), m.v3(r + R_NORMAL)) - m.f(GROUND_HEIGHT) as f64);
            m.set_f(r + R_DEPTH, depth as f32);
            if m.f(r + R_DEPTH) > 0.0 {
                m.set_i(NCONTACT + 4 * i, m.i(NCONTACT + 4 * i) + 1);
            }
        }
    }
}
