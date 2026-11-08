//! FastCar `CRigidBody` (unattributed object 0x369640-0x36E7B0): a single rigid body with Euler and
//! RK4 integration, force/impulse accumulators and quaternion orientation. `CFastCar` and the
//! physics objects derive from it; their virtual overrides go through [`RigidBody`].

use super::mem::{cross, madd, mul_cols, Mem, V3, V4};
use super::vsmath;

/// byte offsets of the base-class members
pub mod off {
    pub const VTABLE: usize = 0;
    pub const GRAVITY: usize = 8; // V3
    pub const LINEAR_DAMPING_TAU: usize = 24;
    pub const ANGULAR_DAMPING_TAU: usize = 28;
    pub const VELOCITY: usize = 32; // V3
    pub const OMEGA: usize = 48; // V3
    pub const POSITION: usize = 64; // V3
    pub const QUAT: usize = 80; // x y z w
    pub const INERTIA_DIAG: usize = 96; // V3, body frame
    pub const ROTATION: usize = 112; // 3 rows of 4
    pub const ACCELERATION: usize = 160; // V3, last derivative
    pub const ANGULAR_ACCELERATION: usize = 176; // V3
    pub const MAX_OMEGA: usize = 192;
    pub const SPHERICAL: usize = 376;
    pub const MASS: usize = 380;
    pub const INERTIA_WORLD: usize = 384; // 3x3, rows of 5 floats (row stride 16, diag stride 20)
    pub const INERTIA_WORLD_INV: usize = 432;
    pub const FORCE: usize = 480; // V3
    pub const TORQUE: usize = 496; // V3
    /// linear and angular impulses accumulate into the same vector in this build
    pub const IMPULSE: usize = 512; // V3
    pub const DIRTY: usize = 528;
    /// size of the base object
    pub const SIZE: usize = 532;
}
use off::*;

/// symmetric 3x3 at `o` in the library's layout (rows 16 bytes apart)
fn sym(m: &Mem, o: usize) -> [f32; 9] {
    [
        m.f(o), m.f(o + 4), m.f(o + 8),
        m.f(o + 16), m.f(o + 20), m.f(o + 24),
        m.f(o + 32), m.f(o + 36), m.f(o + 40),
    ]
}

/// (FastCar) 0x0036AA80 — quaternion product, `[x y z w]`
pub fn quat_mul(a: V4, b: V4) -> V4 {
    let w = ((a[3] * b[3] - a[0] * b[0]) - a[1] * b[1]) - a[2] * b[2];
    let x = ((a[1] * b[2] - a[2] * b[1]) + a[3] * b[0]) + a[0] * b[3];
    let y = ((a[2] * b[0] - a[0] * b[2]) + a[3] * b[1]) + a[1] * b[3];
    let z = ((a[0] * b[1] - a[1] * b[0]) + a[3] * b[2]) + a[2] * b[3];
    [x, y, z, w]
}

/// (FastCar) 0x0036A230 — normalise `[x y z w]`; the zero quaternion becomes identity
pub fn quat_normalize(q: &mut V4) {
    if q[3] == 0.0 && q[0] == 0.0 && q[1] == 0.0 && q[2] == 0.0 {
        q[3] = 1.0;
        return;
    }
    let s = (((q[3] * q[3] + q[0] * q[0]) + q[1] * q[1]) + q[2] * q[2]) as f32;
    let inv = (1.0 / vsmath::sqrt(s)) as f32;
    q[3] *= inv;
    q[0] *= inv;
    q[1] *= inv;
    q[2] *= inv;
}

/// (FastCar) 0x0036A8F0 — rotate `q` by angular velocity `w` over `h`
pub fn quat_integrate(q: V4, w: V3, h: f32) -> V4 {
    let mut a = vsmath::dot87(w, w) as f32;
    if a == 0.0 {
        return q;
    }
    a = vsmath::sqrt(a) as f32;
    let half = (0.5 * a) * h;
    let s = (vsmath::sin(half) / a as f64) as f32;
    let c = vsmath::cos(half) as f32;
    quat_mul([s * w[0], s * w[1], s * w[2], c], q)
}

/// (FastCar) 0x0036C770 — rotation rows from `[x y z w]`
pub fn quat_to_rows(q: V4) -> [f32; 12] {
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    let mut m = [0.0f32; 12];
    m[0] = ((x * x + w * w) * 2.0) - 1.0;
    m[1] = (x * y + z * w) * 2.0;
    m[2] = (z * x - y * w) * 2.0;
    m[4] = (x * y - z * w) * 2.0;
    m[5] = ((y * y + w * w) * 2.0) - 1.0;
    m[6] = (y * z + x * w) * 2.0;
    m[8] = (z * x + y * w) * 2.0;
    m[9] = (y * z - x * w) * 2.0;
    m[10] = ((z * z + w * w) * 2.0) - 1.0;
    m
}

/// (FastCar) 0x0036C960 — inverse of a symmetric 3x3 (input `a` in rows of 4, output rows of 4)
pub fn sym_inverse(a: &[f32; 12]) -> [f32; 12] {
    let v7 = a[1] * a[1];
    let v6 = a[2] * a[2];
    let v5 = a[2] * a[6];
    let v4 = a[5] * a[10] - a[6] * a[6];
    let v3 = 1.0 / ((((0.0 - v6) * a[5] - v7 * a[10]) + (2.0 * a[1]) * v5) + v4 * a[0]);
    let mut o = [0.0f32; 12];
    o[0] = v4 * v3;
    o[5] = (a[0] * a[10] - v6) * v3;
    o[10] = (a[0] * a[5] - v7) * v3;
    o[4] = (v5 - a[1] * a[10]) * v3;
    o[1] = o[4];
    o[8] = (a[1] * a[6] - a[2] * a[5]) * v3;
    o[2] = o[8];
    o[9] = (a[1] * a[2] - a[0] * a[6]) * v3;
    o[6] = o[9];
    o
}

/// The virtual interface (vtable at 0x675894 for the base, 0x675854 for `CFastCar`).
pub trait RigidBody {
    fn mem(&self) -> &Mem;
    fn mem_mut(&mut self) -> &mut Mem;

    /// vtable[8] — recompute orientation-derived data (base: 0x0036CE00)
    fn update_derived(&mut self) {
        rb_update_world_inertia(self.mem_mut());
        self.mem_mut().set_i(DIRTY, 0);
    }

    /// vtable[0] — Euler step (base: 0x0036D9E0)
    fn evolve(&mut self, h: f32, keep_impulses: bool) {
        rb_evolve_euler(self, h, keep_impulses);
    }

    /// vtable[1] — RK4 step (0x0036DA90, shared)
    fn evolve_rk4(&mut self, h: f32, keep_impulses: bool) {
        rb_evolve_rk4(self, h, keep_impulses);
    }

    /// vtable[6] — world matrix rows + translation (base: 0x00369FD0)
    fn matrix(&self) -> [f32; 16] {
        let m = self.mem();
        let mut o = [0.0f32; 16];
        for r in 0..3 {
            let v = m.v3(ROTATION + 16 * r);
            o[4 * r..4 * r + 3].copy_from_slice(&v);
        }
        o[12..15].copy_from_slice(&m.v3(POSITION));
        o[15] = 1.0;
        o
    }

    /// vtable[7] (base: 0x0036B0B0)
    fn position(&self) -> V3 {
        self.mem().v3(POSITION)
    }

    /// vtable[2] / vtable[3]: parameter error / warning reporters
    fn report_error(&self, msg: &str) {
        log::error!("FastCar: {msg}");
    }
    fn report_warning(&self, msg: &str) {
        log::warn!("FastCar: {msg}");
    }
}

/// (FastCar) 0x0036C460 — world rotation from the quaternion, world inertia and its inverse
pub fn rb_update_world_inertia(m: &mut Mem) {
    let r = quat_to_rows(m.v4(QUAT));
    for k in [0usize, 1, 2, 4, 5, 6, 8, 9, 10] {
        m.set_f(ROTATION + 4 * k, r[k]);
    }
    if m.i(SPHERICAL) != 0 {
        return;
    }
    let (r0, r1, r2, r4, r5, r6, r8, r9, r10) = (
        m.f(112), m.f(116), m.f(120), m.f(128), m.f(132), m.f(136), m.f(144), m.f(148), m.f(152),
    );
    let (i0, i1, i2) = (m.f(96), m.f(100), m.f(104));
    m.set_f(384, ((r0 * r0) * i0 + (r4 * r4) * i1) + (r8 * r8) * i2);
    m.set_f(404, ((r1 * r1) * i0 + (r5 * r5) * i1) + (r9 * r9) * i2);
    m.set_f(424, ((r2 * r2) * i0 + (r6 * r6) * i1) + (r10 * r10) * i2);
    let a = 0.0 - ((r0 * r1) * (i2 - i0) + (r4 * r5) * (i2 - i1));
    m.set_f(388, a);
    m.set_f(400, a);
    let b = 0.0 - ((r1 * r2) * (i2 - i0) + (r5 * r6) * (i2 - i1));
    m.set_f(408, b);
    m.set_f(420, b);
    let c = 0.0 - ((r0 * r2) * (i2 - i0) + (r4 * r6) * (i2 - i1));
    m.set_f(392, c);
    m.set_f(416, c);
    let src = [
        m.f(384), m.f(388), m.f(392), 0.0, m.f(400), m.f(404), m.f(408), 0.0, m.f(416), m.f(420), m.f(424), 0.0,
    ];
    let inv = sym_inverse(&src);
    for k in [0usize, 1, 2, 4, 5, 6, 8, 9, 10] {
        // rows of the inverse are 16 bytes apart, same layout as the input
        m.set_f(INERTIA_WORLD_INV + 4 * k, inv[k]);
    }
}

/// (FastCar) 0x00369A80 — mass properties; equal diagonal entries switch to spherical mode
pub fn rb_set_mass<B: RigidBody + ?Sized>(b: &mut B, gravity: V3, inertia: V3, mass: f32, spherical: bool) {
    let m = b.mem_mut();
    m.set_v3(GRAVITY, gravity);
    let mut sph = spherical as i32;
    if inertia[0] == inertia[1] && inertia[0] == inertia[2] {
        sph = 1;
    }
    m.set_i(SPHERICAL, sph);
    if sph != 0 {
        let avg = ((inertia[0] + inertia[1]) + inertia[2]) / 3.0;
        m.set_f(96, avg);
        m.set_f(100, avg);
        m.set_f(104, avg);
        for (o, v) in [(384, avg), (404, avg), (424, avg), (388, 0.0), (392, 0.0), (400, 0.0), (408, 0.0), (416, 0.0), (420, 0.0)] {
            m.set_f(o, v);
        }
        let inv = 1.0 / avg;
        for (o, v) in [(432, inv), (452, inv), (472, inv), (436, 0.0), (440, 0.0), (448, 0.0), (456, 0.0), (464, 0.0), (468, 0.0)] {
            m.set_f(o, v);
        }
    } else {
        m.set_v3(INERTIA_DIAG, inertia);
    }
    m.set_i(DIRTY, 1);
    m.set_f(MASS, mass);
    rb_normalize(b);
}

/// (FastCar) 0x00369890 — base constructor state
pub fn rb_construct(m: &mut Mem) {
    m.set_f(92, 1.0);
    m.set_f(80, 0.0);
    m.set_f(84, 0.0);
    m.set_f(88, 0.0);
    m.set_f(LINEAR_DAMPING_TAU, 0.0);
    m.set_f(ANGULAR_DAMPING_TAU, 0.0);
    m.set_f(IMPULSE, 0.0);
    rb_clear_accumulators(m);
    m.set_v3(POSITION, [0.0; 3]);
    m.set_v3(VELOCITY, [0.0; 3]);
    m.set_v3(OMEGA, [0.0; 3]);
    rb_set_quat(m, [0.0, 0.0, 0.0, 1.0]);
    m.set_f(MAX_OMEGA, 0.0);
}

/// the six `0x0036B9D0`/`36B880`/`36BA40`/`36C230`/`36C1C0`/`36C2A0` constructor calls
fn rb_clear_accumulators(m: &mut Mem) {
    m.set_v3(IMPULSE, [0.0; 3]);
    rb_force_from_impulse(m);
    m.set_v3(IMPULSE, [0.0; 3]);
    m.set_v3(IMPULSE, [0.0; 3]);
    rb_torque_from_impulse(m);
    m.set_v3(IMPULSE, [0.0; 3]);
}

/// (FastCar) 0x0036A370 — set orientation `(w, x, y, z)` arguments, stored `[x y z w]`
pub fn rb_set_quat(m: &mut Mem, q: V4) {
    let mut q = q;
    quat_normalize(&mut q);
    m.set_v4(QUAT, q);
    m.set_i(DIRTY, 1);
}

/// (FastCar) 0x0036B880 — force := accumulator (+ its body-to-world image when x is non-zero)
pub fn rb_force_from_impulse(m: &mut Mem) {
    m.set_v3(FORCE, m.v3(IMPULSE));
    if m.i(IMPULSE) != 0 {
        let r = m.m34(ROTATION);
        let v = madd_cols(m.v3(FORCE), &r, m.v3(IMPULSE));
        m.set_v3(FORCE, v);
    }
}

/// (FastCar) 0x0036C1C0
pub fn rb_torque_from_impulse(m: &mut Mem) {
    m.set_v3(TORQUE, m.v3(IMPULSE));
    if m.i(IMPULSE) != 0 {
        let r = m.m34(ROTATION);
        let v = madd_cols(m.v3(TORQUE), &r, m.v3(IMPULSE));
        m.set_v3(TORQUE, v);
    }
}

/// (FastCar) 0x0036B8F0 — `a += R^T * v`, each row `(c0*v0 + c1*v1) + c2*v2 + a`
fn madd_cols(a: V3, r: &[f32; 12], v: V3) -> V3 {
    let t = mul_cols(r, v);
    [t[0] + a[0], t[1] + a[1], t[2] + a[2]]
}

/// (FastCar) 0x0036BBC0 — force at a world point
pub fn rb_add_force_at(m: &mut Mem, f: V3, p: V3) {
    let r = [p[0] - m.f(64), p[1] - m.f(68), p[2] - m.f(72)];
    let t = cross(r, f);
    m.set_v3(FORCE, add_f(m.v3(FORCE), f));
    m.set_v3(TORQUE, add_f(m.v3(TORQUE), t));
}

/// (FastCar) 0x0036BE70 / 0x0036C040 — impulse at a world point (both terms into `IMPULSE`)
pub fn rb_add_impulse_at(m: &mut Mem, j: V3, p: V3) {
    let r = [p[0] - m.f(64), p[1] - m.f(68), p[2] - m.f(72)];
    let t = cross(r, j);
    m.set_v3(IMPULSE, add_f(m.v3(IMPULSE), j));
    m.set_v3(IMPULSE, add_f(m.v3(IMPULSE), t));
}

/// (FastCar) 0x0036BA70 force += f
pub fn rb_add_force(m: &mut Mem, f: V3) {
    m.set_v3(FORCE, add_f(m.v3(FORCE), f));
}

/// (FastCar) 0x0036C350 / 0x0036C3E0 — accumulate into `IMPULSE`
pub fn rb_add_impulse(m: &mut Mem, j: V3) {
    m.set_v3(IMPULSE, add_f(m.v3(IMPULSE), j));
}

#[inline]
fn add_f(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// (FastCar) 0x0036D740 — apply and clear the impulse accumulator
pub fn rb_apply_impulses(m: &mut Mem) {
    let r = m.m34(ROTATION);
    let j = m.v3(IMPULSE);
    // rows dotted (x87 helper 0x0036D8F0), stored to floats
    let mut b = [
        vsmath::dot87([r[0], r[1], r[2]], j) as f32,
        vsmath::dot87([r[4], r[5], r[6]], j) as f32,
        vsmath::dot87([r[8], r[9], r[10]], j) as f32,
    ];
    b[0] /= m.f(96);
    b[1] /= m.f(100);
    b[2] /= m.f(104);
    let w = mul_cols(&r, b);
    m.set_v3(OMEGA, add_f(m.v3(OMEGA), w));
    let v = madd(m.v3(VELOCITY), j, 1.0 / m.f(MASS));
    m.set_v3(VELOCITY, v);
    m.set_v3(IMPULSE, [0.0; 3]);
    m.set_v3(IMPULSE, [0.0; 3]);
}

/// (FastCar) 0x0036CE40 — angular acceleration from torque, with the gyroscopic term unless the
/// rotation per step reaches pi/4
pub fn rb_angular_acceleration(m: &Mem, h: f32) -> V3 {
    if m.i(SPHERICAL) != 0 {
        let s = m.f(432);
        let t = m.v3(TORQUE);
        return [t[0] * s, t[1] * s, t[2] * s];
    }
    let ii = sym(m, INERTIA_WORLD_INV);
    let w = m.v3(OMEGA);
    let wl = vsmath::sqrt(vsmath::dot87(w, w) as f32);
    let t = m.v3(TORQUE);
    let apply = |x: V3| -> V3 {
        [
            (ii[0] * x[0] + ii[1] * x[1]) + ii[2] * x[2],
            (ii[1] * x[0] + ii[4] * x[1]) + ii[5] * x[2],
            (ii[2] * x[0] + ii[5] * x[1]) + ii[8] * x[2],
        ]
    };
    if (3.1415927f32 / 4.0) as f64 <= wl * h as f64 {
        return apply(t);
    }
    let i = sym(m, INERTIA_WORLD);
    let ly = (i[1] * w[0] + i[4] * w[1]) + i[5] * w[2];
    let lz = (i[2] * w[0] + i[5] * w[1]) + i[8] * w[2];
    let lx = (i[0] * w[0] + i[1] * w[1]) + i[2] * w[2];
    let v7 = ((0.0 - w[1]) * lz + w[2] * ly) + t[0];
    let v6 = ((0.0 - w[2]) * lx + w[0] * lz) + t[1];
    let v5 = ((0.0 - w[0]) * ly + w[1] * lx) + t[2];
    apply([v7, v6, v5])
}

/// (FastCar) 0x0036D280 — derivatives: returns (linear acceleration, angular acceleration)
pub fn rb_derivatives(m: &Mem, h: f32) -> (V3, V3) {
    let mut aa = rb_angular_acceleration(m, h);
    let atau = m.f(ANGULAR_DAMPING_TAU);
    if atau > 0.0 {
        let k = ((vsmath::exp(0.0 - h / atau) - 1.0) / h as f64) as f32;
        aa = madd(aa, m.v3(OMEGA), k);
    }
    let g = m.v3(GRAVITY);
    let f = m.v3(FORCE);
    let inv_m = 1.0 / m.f(MASS);
    let ltau = m.f(LINEAR_DAMPING_TAU);
    if ltau <= 0.0 {
        return ([f[0] * inv_m + g[0], f[1] * inv_m + g[1], f[2] * inv_m + g[2]], aa);
    }
    let r = h / ltau;
    if r <= 0.0001f32 {
        let a = [f[0] * inv_m + g[0], f[1] * inv_m + g[1], f[2] * inv_m + g[2]];
        return (madd(a, m.v3(VELOCITY), -1.0 / ltau), aa);
    }
    let k = ((vsmath::exp(0.0 - r) - 1.0) as f32) / h;
    let v = m.v3(VELOCITY);
    let a = [
        (v[0] - (f[0] * inv_m + g[0]) * ltau) * k,
        (v[1] - (f[1] * inv_m + g[1]) * ltau) * k,
        (v[2] - (f[2] * inv_m + g[2]) * ltau) * k,
    ];
    (a, aa)
}

/// (FastCar) 0x0036D620 — velocities from the stored derivatives, omega clamped to MAX_OMEGA
pub fn rb_integrate_velocities(m: &mut Mem, h: f32) {
    let v = madd(m.v3(VELOCITY), m.v3(ACCELERATION), h);
    m.set_v3(VELOCITY, v);
    let w = madd(m.v3(OMEGA), m.v3(ANGULAR_ACCELERATION), h);
    m.set_v3(OMEGA, w);
    let max = m.f(MAX_OMEGA);
    if max != 0.0 {
        let l = vsmath::sqrt(vsmath::dot87(w, w) as f32) as f32;
        if l > max {
            let s = max / l;
            m.set_v3(OMEGA, [w[0] * s, w[1] * s, w[2] * s]);
        }
    }
}

/// (FastCar) 0x0036D5C0 — position and orientation
pub fn rb_integrate_positions(m: &mut Mem, h: f32) {
    let p = madd(m.v3(POSITION), m.v3(VELOCITY), h);
    m.set_v3(POSITION, p);
    let q = quat_integrate(m.v4(QUAT), m.v3(OMEGA), h);
    m.set_v4(QUAT, q);
}

/// (FastCar) 0x0036CDB0 — renormalise and refresh derived data
pub fn rb_normalize<B: RigidBody + ?Sized>(b: &mut B) {
    let mut q = b.mem().v4(QUAT);
    quat_normalize(&mut q);
    b.mem_mut().set_v4(QUAT, q);
    b.update_derived();
}

/// (FastCar) 0x0036D9E0
pub fn rb_evolve_euler<B: RigidBody + ?Sized>(b: &mut B, h: f32, keep_impulses: bool) {
    if b.mem().i(DIRTY) != 0 {
        b.update_derived();
    }
    if !keep_impulses {
        rb_apply_impulses(b.mem_mut());
    }
    if h != 0.0 {
        let (a, aa) = rb_derivatives(b.mem(), h);
        let m = b.mem_mut();
        m.set_v3(ACCELERATION, a);
        m.set_v3(ANGULAR_ACCELERATION, aa);
        rb_integrate_velocities(m, h);
        rb_integrate_positions(m, h);
    }
    rb_force_from_impulse(b.mem_mut());
    rb_torque_from_impulse(b.mem_mut());
    rb_normalize(b);
}

/// (FastCar) 0x0036DA90 — classic RK4 over position, quaternion, velocity and omega
pub fn rb_evolve_rk4<B: RigidBody + ?Sized>(b: &mut B, h: f32, keep_impulses: bool) {
    rb_normalize(b);
    if !keep_impulses {
        rb_apply_impulses(b.mem_mut());
    }
    if h == 0.0 {
        rb_normalize(b);
        return;
    }
    let m0 = b.mem();
    let q0 = m0.v4(QUAT);
    let p0 = m0.v3(POSITION);
    let w0 = m0.v3(OMEGA);
    let v0 = m0.v3(VELOCITY);
    let half = h / 2.0;
    // k1
    let (mut a1, mut aa1) = rb_derivatives(b.mem(), h);
    a1 = scale_f(a1, h);
    aa1 = scale_f(aa1, h);
    {
        let m = b.mem_mut();
        m.set_v4(QUAT, q0);
        let q = quat_integrate(q0, w0, half);
        m.set_v4(QUAT, q);
        m.set_v3(POSITION, madd(p0, v0, half));
        m.set_v3(OMEGA, madd(w0, aa1, 0.5));
        m.set_v3(VELOCITY, madd(v0, a1, 0.5));
    }
    rb_normalize(b);
    // k2
    let (mut a2, mut aa2) = rb_derivatives(b.mem(), h);
    a2 = scale_f(a2, h);
    aa2 = scale_f(aa2, h);
    {
        let m = b.mem_mut();
        let wm = madd(w0, aa1, 0.5);
        m.set_v4(QUAT, q0);
        let q = quat_integrate(q0, wm, half);
        m.set_v4(QUAT, q);
        for k in 0..3 {
            m.set_f(POSITION + 4 * k, (((a1[k] / 2.0) + v0[k]) * h) / 2.0 + p0[k]);
        }
        m.set_v3(OMEGA, madd(w0, aa2, 0.5));
        m.set_v3(VELOCITY, madd(v0, a2, 0.5));
    }
    rb_normalize(b);
    // k3
    let (mut a3, mut aa3) = rb_derivatives(b.mem(), h);
    a3 = scale_f(a3, h);
    aa3 = scale_f(aa3, h);
    {
        let m = b.mem_mut();
        m.set_v4(QUAT, q0);
        let wm = madd(w0, aa2, 0.5);
        let q = quat_integrate(q0, wm, h);
        m.set_v4(QUAT, q);
        for k in 0..3 {
            m.set_f(POSITION + 4 * k, ((a2[k] / 2.0) + v0[k]) * h + p0[k]);
        }
        m.set_v3(OMEGA, [w0[0] + aa3[0], w0[1] + aa3[1], w0[2] + aa3[2]]);
        m.set_v3(VELOCITY, [v0[0] + a3[0], v0[1] + a3[1], v0[2] + a3[2]]);
    }
    rb_normalize(b);
    // k4
    let (mut a4, mut aa4) = rb_derivatives(b.mem(), h);
    a4 = scale_f(a4, h);
    aa4 = scale_f(aa4, h);
    {
        let m = b.mem_mut();
        m.set_v4(QUAT, q0);
        let wm = [
            ((aa1[0] + aa2[0]) + aa3[0]) / 6.0 + w0[0],
            ((aa1[1] + aa2[1]) + aa3[1]) / 6.0 + w0[1],
            ((aa1[2] + aa2[2]) + aa3[2]) / 6.0 + w0[2],
        ];
        let q = quat_integrate(q0, wm, h);
        m.set_v4(QUAT, q);
        for k in 0..3 {
            m.set_f(POSITION + 4 * k, ((((a1[k] + a2[k]) + a3[k]) / 6.0) + v0[k]) * h + p0[k]);
            m.set_f(OMEGA + 4 * k, ((((2.0 * aa2[k]) + aa1[k]) + (2.0 * aa3[k])) + aa4[k]) / 6.0 + w0[k]);
            m.set_f(VELOCITY + 4 * k, ((((2.0 * a2[k]) + a1[k]) + (2.0 * a3[k])) + a4[k]) / 6.0 + v0[k]);
            m.set_f(ACCELERATION + 4 * k, ((((2.0 * a2[k]) + a1[k]) + (2.0 * a3[k])) + a4[k]) / 6.0 / h);
            m.set_f(ANGULAR_ACCELERATION + 4 * k, ((((2.0 * aa2[k]) + aa1[k]) + (2.0 * aa3[k])) + aa4[k]) / 6.0);
        }
    }
    rb_normalize(b);
    rb_force_from_impulse(b.mem_mut());
    rb_torque_from_impulse(b.mem_mut());
}

#[inline]
fn scale_f(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// (FastCar) 0x0036B490 — `R * v` with the x87 dot helper, rounded per component
pub fn rows_mul87(r: &[f32; 12], v: V3) -> V3 {
    [
        vsmath::dot87([r[0], r[1], r[2]], v) as f32,
        vsmath::dot87([r[4], r[5], r[6]], v) as f32,
        vsmath::dot87([r[8], r[9], r[10]], v) as f32,
    ]
}

/// (FastCar) 0x0036B780 — set omega from a body-frame vector (`R^T` image)
pub fn rb_set_omega_body(m: &mut Mem, wb: V3) {
    let r = m.m34(ROTATION);
    m.set_v3(OMEGA, mul_cols(&r, wb));
    m.set_i(DIRTY, 1);
}

/// (FastCar) 0x0036B6B0 — omega in the body frame
pub fn rb_omega_body(m: &Mem) -> V3 {
    rows_mul87(&m.m34(ROTATION), m.v3(OMEGA))
}

