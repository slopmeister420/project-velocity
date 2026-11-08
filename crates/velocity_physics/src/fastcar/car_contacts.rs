//! `CFastCar` wheel contact preparation (0x362A30) and wheel forces (0x363810).

use super::car::{add, off::*, sub, CFastCar};
use super::car_evolve::normalize_keep;
use super::mem::{cross, madd, V3};
use super::rigid_body::{self as rb, off as rbo};
use super::vsmath;

#[inline]
pub(crate) fn rec(i: usize, k: usize) -> usize {
    REC + REC_SIZE * (i + k)
}

fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

impl CFastCar {
    /// (FastCar) 0x00362A30 — contact frames, merged or depth-sorted contacts, slip state
    pub fn prepare_contacts(&mut self) {
        let m = &mut self.m;
        for i in 0..4 {
            for k in 0..m.i(NCONTACT + 4 * i).max(0) as usize {
                let r = rec(i, k);
                m.set_v3(r + R_REL_POS, sub(m.v3(r + R_POINT), m.v3(WHEEL_POS_W + 16 * i)));
            }
        }
        if m.i(MERGE_CONTACTS) != 0 {
            for i in 0..4 {
                let n = m.i(NCONTACT + 4 * i);
                if n < 2 {
                    continue;
                }
                let mut p = [0.0f32; 3];
                let mut nr = [0.0f32; 3];
                let mut wsum = 0.0f32;
                for k in 0..n as usize {
                    let r = rec(i, k);
                    let d = m.f(r + R_DEPTH);
                    p = madd(p, m.v3(r + R_REL_POS), d);
                    nr = madd(nr, m.v3(r + R_NORMAL), d);
                    wsum += m.f(r + R_DEPTH);
                }
                p = scale(p, 1.0 / wsum);
                nr = normalize_keep(nr);
                let ax = m.v3(AXIS_W + 16 * i);
                let a = vsmath::dot87(ax, p) as f32;
                let radial = madd(p, ax, 0.0 - a);
                let len = vsmath::sqrt(vsmath::dot87(radial, radial) as f32);
                let k = (1.0 - m.f(RADIUS + 4 * i) as f64 / len) as f32;
                p = madd(p, radial, 0.0 - k);
                let side = normalize_keep(cross(ax, p));
                let b = vsmath::dot87(nr, side) as f32;
                nr = normalize_keep(madd(nr, side, 0.0 - b));
                let mut deepest = 0.0f32;
                for k in 0..n as usize {
                    let r = rec(i, k);
                    let c = vsmath::dot87(m.v3(r + R_NORMAL), nr) as f32;
                    if c * m.f(r + R_DEPTH) > deepest {
                        deepest = c * m.f(r + R_DEPTH);
                    }
                }
                let r0 = rec(i, 0);
                m.set_v3(r0 + R_POINT, add(p, m.v3(WHEEL_POS_W + 16 * i)));
                m.set_v3(r0 + R_NORMAL, nr);
                m.set_f(r0 + R_DEPTH, deepest);
                m.set_i(NCONTACT + 4 * i, 1);
            }
        } else {
            for i in 0..4 {
                loop {
                    let mut swapped = false;
                    let n = m.i(NCONTACT + 4 * i);
                    let mut k = 1;
                    while k < n as usize {
                        if m.f(rec(i, k) + R_DEPTH) > m.f(rec(i, k - 1) + R_DEPTH) {
                            let a: Vec<u32> = m.words(rec(i, k), 34).to_vec();
                            m.copy_words(rec(i, k), rec(i, k - 1), 34);
                            for (j, w) in a.iter().enumerate() {
                                m.set_u(rec(i, k - 1) + 4 * j, *w);
                            }
                            swapped = true;
                        }
                        k += 1;
                    }
                    if !swapped {
                        break;
                    }
                }
            }
        }
        for i in 0..4 {
            for k in 0..m.i(NCONTACT + 4 * i).max(0) as usize {
                let r = rec(i, k);
                let ax = m.v3(AXIS_W + 16 * i);
                let t1 = normalize_keep(cross(m.v3(r + R_NORMAL), ax));
                m.set_v3(r + R_T1, t1);
                m.set_v3(r + R_T2, cross(t1, m.v3(r + R_NORMAL)));
                let spin = if m.i(WHEEL_LOCKED + 4 * i) != 0 {
                    [0.0; 3]
                } else {
                    cross(m.v3(WHEEL_OMEGA_W + 16 * i), m.v3(r + R_REL_POS))
                };
                let rv = add(m.v3(WHEEL_VEL_W + 16 * i), spin);
                m.set_v3(r + R_REL_VEL, rv);
                m.set_f(r + R_VN, vsmath::dot87(rv, m.v3(r + R_NORMAL)) as f32);
                m.set_f(r + R_VT1, vsmath::dot87(rv, m.v3(r + R_T1)) as f32);
                m.set_f(r + R_VT2, vsmath::dot87(rv, m.v3(r + R_T2)) as f32);
                let vt1 = m.f(r + R_VT1);
                let vt2 = m.f(r + R_VT2);
                let dv = m.f(DYNAMIC_VELOCITY);
                if m.i(SLIP_COUNTER) != 0 || vt1 * vt1 + vt2 * vt2 > dv * dv || m.f(MU + 4 * i) == 0.0 {
                    m.set_i(r + R_SLIDING, 1);
                    m.set_f(r + R_MU, m.f(MU + 4 * i) / m.f(STATIC_DYNAMIC_RATIO + 4 * i));
                } else {
                    m.set_i(r + R_SLIDING, 0);
                    m.set_u(r + R_MU, m.u(MU + 4 * i));
                }
            }
        }
    }

    /// (FastCar) 0x00363810 — brake/drive torques, the tyre solver, suspension state, applied forces
    pub fn wheel_forces(&mut self) {
        let mut gripping = 0i32;
        for i in 0..4 {
            if self.m.i(rec(i, 0) + R_SLIDING) == 0 {
                gripping += 1;
            }
        }
        self.m.set_v4(NET_TORQUE, [0.0; 4]);
        for i in 0..4 {
            let m = &mut self.m;
            if m.i(WHEEL_LOCKED + 4 * i) != 0 {
                continue;
            }
            m.set_f(NET_TORQUE + 4 * i, (0.0 - m.f(DRIVE_TORQUE + 4 * i)) + m.f(WHEEL_TORQUE + 4 * i));
            let sliding = m.i(rec(i, 0) + R_SLIDING) != 0;
            let brake = m.f(BRAKE_TORQUE + 4 * i);
            let om = m.f(WHEEL_OMEGA + 4 * i);
            let r = m.f(RADIUS + 4 * i);
            if !sliding && brake > 0.0 && om != 0.0 {
                let b = if om <= 0.0 { brake } else { 0.0 - brake };
                let g = vsmath::dot87(m.v3(rec(i, 0) + R_T1), m.v3(rbo::GRAVITY)) as f32;
                let stop = (((-1.0 / gripping as f32) * m.f(CHASSIS_MASS)) * r) * (((om * r) / m.f(H)) - g);
                let v = if vsmath::fabs(b) <= vsmath::fabs(stop) { m.f(NET_TORQUE + 4 * i) + b } else { m.f(NET_TORQUE + 4 * i) + stop };
                m.set_f(NET_TORQUE + 4 * i, v);
            } else if sliding && brake > 0.0 && om != 0.0 {
                let w = m.f(MU + 4 * i) * m.f(rbo::MASS);
                let g = m.v3(rbo::GRAVITY);
                let gl = vsmath::sqrt(vsmath::dot87(g, g) as f32);
                let limit = (gl * w as f64 * r as f64) as f32;
                let b = if om <= 0.0 { brake } else { 0.0 - brake };
                if vsmath::fabs(b + m.f(DRIVE_TORQUE + 4 * i)) <= limit as f64 || (b + m.f(DRIVE_TORQUE + 4 * i)) * om >= 0.0 {
                    let v7 = if om <= 0.0 {
                        m.f(DYNAMIC_VELOCITY) / m.f(WHEEL_INV_TOI + 16 * i + 8)
                    } else {
                        (0.0 - m.f(DYNAMIC_VELOCITY)) / m.f(WHEEL_INV_TOI + 16 * i + 8)
                    };
                    let mut t = v7 / (r * m.f(H));
                    if vsmath::fabs(t) > brake as f64 {
                        t = if t <= 0.0 { 0.0 - brake } else { brake };
                    }
                    m.set_f(NET_TORQUE + 4 * i, m.f(NET_TORQUE + 4 * i) + t);
                } else {
                    m.set_i(WHEEL_FREE + 4 * i, 1);
                    m.set_i(WHEEL_OMEGA + 4 * i, 0);
                }
            }
        }
        self.tyre_solver();
        let ty = self.m.i(WHEEL_DYNAMICS_TYPE);
        for i in 0..4 {
            let m = &mut self.m;
            let relax = |m: &super::mem::Mem| -> f32 {
                let d = m.f(SUSP_PREV + 4 * i) - m.f(SUSP_REST + 4 * i);
                let k = (vsmath::sqrt(m.f(SPRING + 4 * i) / m.f(WHEEL_MASS + 4 * i)) * (-m.f(H)) as f64) as f32;
                (vsmath::exp(k) * d as f64 + m.f(SUSP_REST + 4 * i) as f64) as f32
            };
            if ty == 2 {
                let v = relax(m);
                m.set_f(SUSP_X + 4 * i, v);
            } else if ty == 1 {
                if m.i(NCONTACT + 4 * i) <= 0 {
                    let v = relax(m);
                    m.set_f(SUSP_X + 4 * i, v);
                } else if vsmath::dot87(m.v3(rec(i, 0) + R_NORMAL), m.v3(rbo::ROTATION + 16)) <= 0.0 {
                    if m.f(SUSP_REST + 4 * i) <= m.f(SUSP_PREV + 4 * i) {
                        m.set_u(SUSP_X + 4 * i, m.u(SUSP_REST + 4 * i));
                    } else {
                        let v = relax(m);
                        m.set_f(SUSP_X + 4 * i, v);
                    }
                } else if m.f(SUSP_PREV + 4 * i) <= m.f(SUSP_REST + 4 * i) {
                    m.set_u(SUSP_X + 4 * i, m.u(SUSP_REST + 4 * i));
                } else {
                    let v = relax(m);
                    m.set_f(SUSP_X + 4 * i, v);
                }
            } else {
                m.set_u(SUSP_X + 4 * i, m.u(SUSP_REST + 4 * i));
            }
            m.set_u(SUSP_PREV + 4 * i, m.u(SUSP_X + 4 * i));
        }
        for i in 0..4 {
            if self.m.i(WHEEL_DIR_SET + 4 * i) != 0 {
                // 0x00361AB0
                let f = self.m.v3(WHEEL_DIR + 16 * i);
                let p = self.m.v3(WHEEL_POS_W + 16 * i);
                rb::rb_add_force_at(&mut self.m, f, p);
            } else {
                let t = self.m.f(NET_TORQUE + 4 * i);
                self.m.set_f(WHEEL_TORQUE + 4 * i, self.m.f(WHEEL_TORQUE + 4 * i) + t);
            }
            for k in 0..self.m.i(NCONTACT + 4 * i).max(0) as usize {
                let r = rec(i, k);
                if !(self.m.f(r + R_FN) > 0.0) {
                    continue;
                }
                let force = self.m.v3(r + R_FORCE);
                if self.m.i(FLAG_2244) != 0 {
                    let up = self.m.v3(rbo::ROTATION + 16);
                    let at = madd(self.m.v3(r + R_POINT), up, self.m.f(SUSP_X + 4 * i));
                    if self.m.i(WHEEL_FREE + 4 * i) != 0 {
                        rb::rb_add_force_at(&mut self.m, force, at);
                    } else {
                        let t = cross(self.m.v3(r + R_REL_POS), force);
                        rb::rb_add_force_at(&mut self.m, force, at);
                        self.add_wheel_torque(t, i);
                    }
                    let m = &mut self.m;
                    let front = i == 0 || i == 3;
                    if front && m.i(SUSP_GEOMETRY_FRONT) == 1 || !front && m.i(SUSP_GEOMETRY_BACK) == 1 {
                        let arm = scale(m.v3(rbo::ROTATION + 16), m.f(RADIUS + 4 * i) + m.f(TOPPLE_CORRECTION));
                        let c = cross(arm, force);
                        let d = vsmath::dot87(c, m.v3(rbo::ROTATION)) as f32;
                        let t = scale(m.v3(rbo::ROTATION), d);
                        m.set_v3(rbo::TORQUE, add(m.v3(rbo::TORQUE), t));
                    } else if m.f(TOPPLE_CORRECTION) != 0.0 {
                        let arm = scale(m.v3(rbo::ROTATION + 16), m.f(TOPPLE_CORRECTION));
                        let c = cross(arm, force);
                        let d = vsmath::dot87(c, m.v3(rbo::ROTATION)) as f32;
                        let t = scale(m.v3(rbo::ROTATION), d);
                        m.set_v3(rbo::TORQUE, add(m.v3(rbo::TORQUE), t));
                    }
                } else {
                    let p = madd(self.m.v3(r + R_POINT), self.m.v3(r + R_NORMAL), self.m.f(r + R_DEPTH));
                    self.add_wheel_force_at(force, p, i);
                }
            }
        }
    }
}
