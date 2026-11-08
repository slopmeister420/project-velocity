//! `CFastCar::Evolve` (0x3652D0) and the two-wheel balancing it contains.

use super::car::{add, off::*, sub, CFastCar};
use super::mem::{cross, madd, V3, V4};
use super::rigid_body::{self as rb, off as rbo, quat_integrate, quat_to_rows};
use super::vsmath;

#[inline]
fn rec(i: usize) -> usize {
    REC + REC_SIZE * i
}

/// (FastCar) 0x00367A70 — normalise, zero vector becomes (1,0,0)
fn normalize_or_x(a: V3) -> V3 {
    let l = vsmath::sqrt(vsmath::dot87(a, a) as f32) as f32;
    if l != 0.0 {
        let s = 1.0 / l;
        [a[0] * s, a[1] * s, a[2] * s]
    } else {
        [1.0, 0.0, 0.0]
    }
}

/// (FastCar) 0x00367B60 — remove the `n` component: `a - n (a.n)`
fn reject(a: V3, n: V3) -> V3 {
    let d = vsmath::dot87(a, n) as f32;
    madd(a, n, 0.0 - d)
}

/// (FastCar) 0x00363740 / 0x003719B0 — normalise in place, zero vector unchanged
pub(crate) fn normalize_keep(a: V3) -> V3 {
    let l = vsmath::sqrt(vsmath::dot87(a, a) as f32) as f32;
    if l != 0.0 {
        [a[0] * (1.0 / l), a[1] * (1.0 / l), a[2] * (1.0 / l)]
    } else {
        a
    }
}

/// signed angle between `a` and `b` about `axis` (the 0.9 branch uses the cross product length)
fn signed_angle(a: V3, b: V3, axis: V3) -> f32 {
    let d = vsmath::dot87(b, a) as f32;
    if vsmath::fabs(d) >= 0.9f32 as f64 {
        let c = sub(a, b);
        let l = vsmath::sqrt(vsmath::dot87(c, c) as f32) as f32;
        if l <= 0.00000001f32 {
            0.0
        } else {
            let x = cross(b, c);
            if vsmath::dot87(x, axis) <= 0.0 {
                0.0 - l
            } else {
                l
            }
        }
    } else {
        let x = cross(b, a);
        if vsmath::dot87(x, axis) <= 0.0 {
            (-vsmath::acos(d)) as f32
        } else {
            vsmath::acos(d) as f32
        }
    }
}

impl CFastCar {
    /// (FastCar) 0x003652D0
    pub fn evolve_car(&mut self, h: f32, _keep: bool) {
        self.m.set_f(H, h);
        for i in 0..4 {
            let v = (self.m.i(WHEEL_LOCKED + 4 * i) != 0 || self.m.i(WHEEL_FLAG2 + 4 * i) != 0) as i32;
            self.m.set_i(WHEEL_FREE + 4 * i, v);
        }
        if self.m.f(H) != 0.0 && self.m.i(FLAG_2244) != 0 {
            self.m.set_v3(SUSP_OFFSET, [0.0; 3]);
            self.m.set_v4(SUSP_X, [0.0; 4]);
            self.m.set_v4(SUSP_2284, [0.0; 4]);
            self.m.set_f(ROLL_FRONT, 0.0);
            self.m.set_f(ROLL_BACK, 0.0);
        }
        for i in 0..4 {
            self.m.set_i(WHEEL_DIR_SET + 4 * i, 0);
        }
        if self.m.i(rbo::DIRTY) != 0 {
            rb::RigidBody::update_derived(self);
        }
        self.apply_wheel_impulses();
        rb::rb_apply_impulses(&mut self.m);
        if self.m.f(H) != 0.0 {
            self.m.set_v4(SUSP_REST, [0.0; 4]);
            self.detect_wheel_contacts();
            self.prepare_contacts();
            self.choose_balance_pair();
            let any = (0..4).any(|i| self.m.i(NCONTACT + 4 * i) != 0);
            if any {
                self.m.set_i(AIRBORNE, 0);
                self.wheel_forces();
                if self.m.f(PVP) == 0.0 {
                    rb::rb_evolve_euler(self, h, false);
                    self.m.set_f(SKID_ANGLE, 0.0);
                } else {
                    self.evolve_physics_vs_play(h);
                }
                let mut a = self.m.f(SKID_ANGLE);
                let max = self.m.f(VISUAL_STEER_MAX);
                // 0x003679E0 clamp
                if 0.0 - max <= a {
                    if a > max {
                        a = max;
                    }
                } else {
                    a = 0.0 - max;
                }
                let v = (a - self.m.f(VISUAL_STEER)) * self.m.f(VISUAL_STEER_RATE) + self.m.f(VISUAL_STEER);
                self.m.set_f(VISUAL_STEER, v);
            } else {
                self.m.set_i(AIRBORNE, 1);
                if self.m.i(BALANCE_FLAG_2880) != 0 {
                    rb::rb_evolve_euler(self, h, true);
                } else {
                    rb::rb_evolve_rk4(self, h, true);
                }
                self.airborne_wheels();
            }
            self.two_wheel_balance();
            self.after_balance(h);
        }
        for i in 0..4 {
            self.m.set_f(WHEEL_2192 + 4 * i, 0.0);
        }
        if self.m.f(H) != 0.0 {
            for i in 0..4 {
                self.m.set_u(WHEEL_TORQUE + 4 * i, self.m.u(WHEEL_TORQUE_KEEP + 4 * i));
            }
        }
        for i in 0..4 {
            self.m.set_u(WHEEL_FREE + 4 * i, self.m.u(WHEEL_LOCKED + 4 * i));
        }
        rb::RigidBody::update_derived(self);
        for i in 0..4 {
            let mtx = self.wheel_matrix(i);
            for (k, v) in mtx.iter().enumerate() {
                self.m.set_f(WHEEL_MATRIX + 64 * i + 4 * k, *v);
            }
        }
        if self.m.i(SLIP_COUNTER) > 0 {
            self.m.set_i(SLIP_COUNTER, self.m.i(SLIP_COUNTER) - 1);
        }
    }

    /// the 2840..2848 pair selection after contact preparation
    fn choose_balance_pair(&mut self) {
        let m = &mut self.m;
        let c = |m: &super::mem::Mem, i: usize| m.i(NCONTACT + 4 * i) != 0;
        if m.i(TWO_WHEELS) == 0 || m.i(BALANCE_STATE) != 0 {
            m.set_i(BALANCE_PAIR_A, -1);
            return;
        }
        let (c0, c1, c2, c3) = (c(m, 0), c(m, 1), c(m, 2), c(m, 3));
        if (!c0 || !c1 || c2 && c3) && (!c0 && !c1 || c2 || c3) {
            if (!c2 || !c3 || c0 && c1) && (!c2 && !c3 || c0 || c1) {
                if c0 && c1 && c2 && c3 && m.i(BALANCE_STATE) == 0 {
                    m.set_i(BALANCE_PAIR_A, -2);
                } else {
                    m.set_i(BALANCE_PAIR_A, -1);
                }
            } else {
                m.set_i(BALANCE_PAIR_A, 3);
                m.set_i(BALANCE_PAIR_B, 2);
            }
        } else {
            m.set_i(BALANCE_PAIR_A, 0);
            m.set_i(BALANCE_PAIR_B, 1);
        }
    }

    /// physics-vs-play integration (the `PVP != 0` branch of Evolve)
    fn evolve_physics_vs_play(&mut self, h: f32) {
        if self.m.i(rbo::DIRTY) != 0 {
            rb::rb_normalize(self);
        }
        rb::rb_apply_impulses(&mut self.m);
        let (a, aa) = rb::rb_derivatives(&self.m, h);
        self.m.set_v3(rbo::ACCELERATION, a);
        self.m.set_v3(rbo::ANGULAR_ACCELERATION, aa);
        let m = &self.m;
        let tr = self.turning_radius();
        let vel = m.v3(rbo::VELOCITY);
        let fwd = vsmath::dot87(vel, m.v3(rbo::ROTATION)) as f32;
        let side = vsmath::dot87(vel, m.v3(rbo::ROTATION + 32)) as f32;
        let target_omega: f32 = if tr == 0.0 {
            0.0
        } else {
            let mut sp = vsmath::sqrt(fwd * fwd + side * side) as f32;
            if fwd < 0.0 {
                let a = m.f(DYNAMIC_VELOCITY) - fwd;
                let b = vsmath::fabs(side);
                if (a as f64) > b * f32::from_bits(FLT_6CD698) as f64
                    && (m.f(1204) < 0.0 || m.f(1208) < 0.0 || m.f(1212) < 0.0 || m.f(1216) < 0.0)
                    && m.f(1204) <= 0.0
                    && m.f(1208) <= 0.0
                    && m.f(1212) <= 0.0
                    && m.f(1216) <= 0.0
                {
                    sp = 0.0 - sp;
                }
            }
            (0.0 - sp) / tr
        };
        let wy = vsmath::dot87(m.v3(rbo::OMEGA), m.v3(rbo::ROTATION + 16)) as f32;
        let mut into_turn: i32 = -1;
        let cur: f32;
        if self.contact_state() == 0 || self.m.f(SKID_ANGLE) == 0.0 {
            if self.contact_state() != 0 {
                let a = vsmath::atan2(side, fwd) as f32;
                self.m.set_f(SKID_ANGLE, a);
            } else {
                self.m.set_f(SKID_ANGLE, 0.0);
            }
            cur = vsmath::dot87(self.m.v3(rbo::OMEGA), self.m.v3(rbo::ROTATION + 16)) as f32;
        } else {
            let a = vsmath::atan2(side, fwd) as f32;
            into_turn = (self.m.f(PVP_INTO_TURN) > 0.0 && a * self.m.f(1884) < 0.0) as i32;
            let mut c = target_omega - (a - self.m.f(SKID_ANGLE)) / self.m.f(H);
            c += wy;
            cur = c;
            self.m.set_f(SKID_ANGLE, a);
        }
        let blend = if into_turn == 1 {
            cur * self.m.f(PVP_INTO_TURN) + (1.0 - self.m.f(PVP_INTO_TURN)) * target_omega
        } else if into_turn != 0 {
            target_omega
        } else {
            cur * self.m.f(PVP_STRAIGHTEN) + (1.0 - self.m.f(PVP_STRAIGHTEN)) * target_omega
        };
        let rate = (blend - wy) / (self.m.f(PVP_RELAXATION) * self.m.f(H));
        let cur_aa = vsmath::dot87(self.m.v3(rbo::ANGULAR_ACCELERATION), self.m.v3(rbo::ROTATION + 16)) as f32;
        let aa = madd(self.m.v3(rbo::ANGULAR_ACCELERATION), self.m.v3(rbo::ROTATION + 16), (rate - cur_aa) * self.m.f(PVP));
        self.m.set_v3(rbo::ANGULAR_ACCELERATION, aa);
        rb::rb_integrate_velocities(&mut self.m, h);
        rb::rb_integrate_positions(&mut self.m, h);
        rb::rb_force_from_impulse(&mut self.m);
        rb::rb_torque_from_impulse(&mut self.m);
        rb::rb_normalize(self);
    }

    /// wheel spin and suspension relaxation while no wheel touches
    fn airborne_wheels(&mut self) {
        for i in 0..4 {
            let m = &mut self.m;
            let base = (0.0 - m.f(DRIVE_TORQUE + 4 * i)) + m.f(WHEEL_TORQUE + 4 * i);
            m.set_f(NET_TORQUE + 4 * i, base);
            let brake = if m.f(WHEEL_OMEGA + 4 * i) <= 0.0 {
                m.f(BRAKE_TORQUE + 4 * i)
            } else {
                0.0 - m.f(BRAKE_TORQUE + 4 * i)
            };
            let stop = ((-1.0 / m.f(1476 + 16 * i)) * m.f(WHEEL_OMEGA + 4 * i)) / m.f(H);
            let a = vsmath::fabs(stop);
            let b = vsmath::fabs(brake);
            let v = if b <= a { m.f(NET_TORQUE + 4 * i) + brake } else { m.f(NET_TORQUE + 4 * i) + stop };
            m.set_f(NET_TORQUE + 4 * i, v);
            let t = m.f(NET_TORQUE + 4 * i);
            m.set_f(WHEEL_TORQUE + 4 * i, m.f(WHEEL_TORQUE + 4 * i) + t);
        }
        let ty = self.m.i(WHEEL_DYNAMICS_TYPE);
        for i in 0..4 {
            let m = &mut self.m;
            let relax = || {
                let k = (vsmath::sqrt(m.f(SPRING + 4 * i) / m.f(WHEEL_MASS + 4 * i)) * (-m.f(H)) as f64) as f32;
                let e = vsmath::exp(k);
                (e * m.f(SUSP_PREV + 4 * i) as f64) as f32
            };
            if ty == 2 {
                let v = relax();
                m.set_f(SUSP_X + 4 * i, v);
                m.set_u(SUSP_PREV + 4 * i, m.u(SUSP_X + 4 * i));
            } else if ty == 1 {
                if m.i(NCONTACT + 4 * i) <= 0 {
                    let v = relax();
                    m.set_f(SUSP_X + 4 * i, v);
                    m.set_u(SUSP_PREV + 4 * i, m.u(SUSP_X + 4 * i));
                } else if vsmath::dot87(m.v3(rec(i) + R_NORMAL), m.v3(rbo::ROTATION + 16)) <= 0.0 {
                    if m.f(SUSP_PREV + 4 * i) >= 0.0 {
                        m.set_f(SUSP_X + 4 * i, 0.0);
                        m.set_f(SUSP_PREV + 4 * i, 0.0);
                    } else {
                        let v = relax();
                        m.set_f(SUSP_X + 4 * i, v);
                        m.set_u(SUSP_PREV + 4 * i, m.u(SUSP_X + 4 * i));
                    }
                } else if m.f(SUSP_PREV + 4 * i) <= 0.0 {
                    m.set_f(SUSP_X + 4 * i, 0.0);
                    m.set_f(SUSP_PREV + 4 * i, 0.0);
                } else {
                    let v = relax();
                    m.set_f(SUSP_X + 4 * i, v);
                    m.set_u(SUSP_PREV + 4 * i, m.u(SUSP_X + 4 * i));
                }
            } else {
                m.set_f(SUSP_X + 4 * i, 0.0);
            }
        }
    }

    /// the two-wheel pair test and correction of Evolve (active only with `TWO_WHEELS`)
    fn two_wheel_balance(&mut self) {
        let m = &self.m;
        if m.i(TWO_WHEELS) != 0 && m.i(BALANCE_PAIR_A) == -2 && m.f(BALANCE_TIME_2) != 0.0 {
            let mut dw = [0.0f32; 2];
            for (pass, out) in dw.iter_mut().enumerate() {
                if pass != 0 {
                    self.m.set_i(BALANCE_PAIR_A, 3);
                    self.m.set_i(BALANCE_PAIR_B, 2);
                } else {
                    self.m.set_i(BALANCE_PAIR_A, 0);
                    self.m.set_i(BALANCE_PAIR_B, 1);
                }
                let m = &self.m;
                let a = m.i(BALANCE_PAIR_A) as usize;
                let b = m.i(BALANCE_PAIR_B) as usize;
                let axis = normalize_or_x(sub(m.v3(rec(b) + R_POINT), m.v3(rec(a) + R_POINT)));
                let f = add(m.v3(rec(a)), m.v3(rec(b)));
                let s = add(m.v3(rec(b) + R_POINT), m.v3(rec(a) + R_POINT));
                let mid = [s[0] * 0.5, s[1] * 0.5, s[2] * 0.5];
                let out_dir = normalize_or_x(reject(sub(m.v3(rbo::POSITION), mid), axis));
                let fdir = normalize_or_x(reject(normalize_or_x(f), axis));
                let ang = signed_angle(out_dir, fdir, axis);
                let w = vsmath::dot87(m.v3(rbo::OMEGA), axis) as f32;
                *out = ang / m.f(BALANCE_TIME_2) + w;
            }
            let mag = [vsmath::fabs(dw[0]) as f32, vsmath::fabs(dw[1]) as f32];
            let (lo, hi) = if mag[1] <= mag[0] { (1usize, 0usize) } else { (0usize, 1usize) };
            let m = &mut self.m;
            if mag[lo] <= m.f(BALANCE_MIN_2896) && m.f(BALANCE_MIN_2900) as f64 <= vsmath::fabs(dw[lo] + dw[hi]) {
                if lo == 0 {
                    m.set_i(BALANCE_PAIR_A, 0);
                    m.set_i(BALANCE_PAIR_B, 1);
                } else {
                    m.set_i(BALANCE_PAIR_A, 3);
                    m.set_i(BALANCE_PAIR_B, 2);
                }
            } else {
                m.set_i(BALANCE_PAIR_A, -1);
            }
        } else if self.m.i(BALANCE_PAIR_A) == -2 {
            self.m.set_i(BALANCE_PAIR_A, -1);
        }
        self.balance_correction();
    }

    fn balance_correction(&mut self) {
        let m = &self.m;
        if m.i(TWO_WHEELS) == 0
            || m.i(BALANCE_PAIR_A) == -1
            || m.i(BALANCE_STATE) != 0
            || m.f(BALANCE_TIME_1) == 0.0 && m.f(BALANCE_TIME_2) == 0.0
        {
            return;
        }
        let a = m.i(BALANCE_PAIR_A) as usize;
        let b = m.i(BALANCE_PAIR_B) as usize;
        let ca = m.i(NCONTACT + 4 * a) != 0;
        let cb = m.i(NCONTACT + 4 * b) != 0;
        let both;
        let mut one = 0usize;
        let (axis, rel, f, out_dir);
        if ca && cb {
            both = true;
            axis = normalize_or_x(sub(m.v3(rec(b) + R_POINT), m.v3(rec(a) + R_POINT)));
            rel = sub(m.v3(rec(b) + R_POINT), m.v3(rbo::POSITION));
            f = add(m.v3(rec(a)), m.v3(rec(b)));
            let s = add(m.v3(rec(b) + R_POINT), m.v3(rec(a) + R_POINT));
            let mid = [s[0] * 0.5, s[1] * 0.5, s[2] * 0.5];
            out_dir = normalize_or_x(reject(sub(m.v3(rbo::POSITION), mid), axis));
        } else if ca || cb {
            both = false;
            one = if m.i(NCONTACT + 4 * a) <= 0 { b } else { a };
            axis = normalize_or_x(m.v3(rec(one) + R_T1));
            rel = sub(m.v3(rec(one) + R_POINT), m.v3(rbo::POSITION));
            f = m.v3(rec(one));
            out_dir = normalize_or_x(reject(sub(m.v3(rbo::POSITION), m.v3(rec(one) + R_POINT)), axis));
        } else {
            return;
        }
        let rel = reject(rel, axis);
        let fdir = normalize_or_x(reject(normalize_or_x(f), axis));
        let ang = signed_angle(out_dir, fdir, axis);
        let m = &mut self.m;
        if both || m.f(BALANCE_TIME_1) == 0.0 || ang == 0.0 {
            if !both || m.f(BALANCE_TIME_2) == 0.0 || ang == 0.0 {
                return;
            }
            let w = vsmath::dot87(m.v3(rbo::OMEGA), axis) as f32;
            let need = ang / m.f(BALANCE_TIME_2) + w;
            if vsmath::fabs(need) > m.f(BALANCE_MAX_OMEGA) as f64 {
                m.set_i(BALANCE_STATE, 1);
                return;
            }
            if vsmath::fabs(ang) > m.f(BALANCE_MAX_ANGLE) as f64 {
                m.set_i(BALANCE_STATE, 2);
                return;
            }
            let rate = (0.0 - ang) / m.f(BALANCE_TIME_2);
            self.balance_rotate(axis, w, rate, rel, a);
        } else {
            let w = vsmath::dot87(m.v3(rbo::OMEGA), axis) as f32;
            let tip = cross(m.v3(rbo::VELOCITY), rel);
            let k = m.f(rbo::MASS) / m.f(96);
            let need = (w as f64 - vsmath::dot87(tip, axis) * k as f64) as f32;
            if vsmath::fabs(need) > m.f(BALANCE_MAX_OMEGA) as f64 {
                m.set_i(BALANCE_STATE, 1);
                return;
            }
            if vsmath::fabs(ang) > m.f(BALANCE_MAX_ANGLE) as f64 {
                m.set_i(BALANCE_STATE, 2);
                return;
            }
            let rate = (0.0 - ang) / m.f(BALANCE_TIME_1);
            self.balance_rotate(axis, w, rate, rel, one);
        }
        rb::RigidBody::update_derived(self);
    }

    /// rotate the chassis about the balancing axis through the contact line
    fn balance_rotate(&mut self, axis: V3, w: f32, rate: f32, rel: V3, wheel: usize) {
        let m = &mut self.m;
        let om = madd(m.v3(rbo::OMEGA), axis, 0.0 - w);
        m.set_v3(rbo::OMEGA, om);
        let dw = [axis[0] * rate, axis[1] * rate, axis[2] * rate];
        let q = quat_integrate(m.v4(rbo::QUAT), dw, m.f(H));
        m.set_v4(rbo::QUAT, q);
        let dq: V4 = quat_integrate([0.0, 0.0, 0.0, 1.0], dw, m.f(H));
        let r = quat_to_rows(dq);
        let rr = [
            vsmath::dot87([r[0], r[1], r[2]], rel) as f32,
            vsmath::dot87([r[4], r[5], r[6]], rel) as f32,
            vsmath::dot87([r[8], r[9], r[10]], rel) as f32,
        ];
        let d = sub(rr, rel);
        if m.f(BALANCE_PROJECT) == 0.0 {
            let p = add(m.v3(rbo::POSITION), d);
            m.set_v3(rbo::POSITION, p);
        } else {
            let n = m.v3(rec(wheel) + R_NORMAL);
            let k = vsmath::dot87(d, n) as f32;
            let p = madd(m.v3(rbo::POSITION), n, k);
            m.set_v3(rbo::POSITION, p);
            let t = reject(d, n);
            let s = 1.0 - m.f(BALANCE_PROJECT);
            let p = add(m.v3(rbo::POSITION), [t[0] * s, t[1] * s, t[2] * s]);
            m.set_v3(rbo::POSITION, p);
        }
    }

    /// LABEL_206 of Evolve: suspension roll angles and the wheel spin/angle integration
    fn after_balance(&mut self, h: f32) {
        let m = &mut self.m;
        m.set_f(ROLL_FRONT, 0.0);
        m.set_f(ROLL_BACK, 0.0);
        if m.i(SUSP_GEOMETRY_FRONT) == 1 {
            let a = vsmath::atan2((m.f(2268) - m.f(2280)) * 0.5, m.f(1828)) as f32;
            m.set_f(ROLL_FRONT, a);
            let c = (m.f(2268) + m.f(2280)) * 0.5;
            m.set_f(2296, c);
            m.set_f(2284, c);
        }
        if m.i(SUSP_GEOMETRY_BACK) == 1 {
            let a = vsmath::atan2((m.f(2272) - m.f(2276)) * 0.5, m.f(1844)) as f32;
            m.set_f(ROLL_BACK, a);
            let c = (m.f(2272) + m.f(2276)) * 0.5;
            m.set_f(2292, c);
            m.set_f(2288, c);
        }
        self.wheel_omega_from_keep();
        self.integrate_wheel_omega(h);
        self.integrate_wheel_angles(h);
    }
}

/// `flt_6CD698` (2.0)
const FLT_6CD698: u32 = 0x40000000;
