//! `CFastCar` tyre/contact force solver (0x372B70): spring-damper normal forces, the closed-form
//! drive-torque distribution, friction ellipse clamping, slip ratios and suspension travel limits.

use super::car::{off::*, CFastCar};
use super::car_contacts::rec;
use super::car_evolve::normalize_keep;
use super::mem::{madd, V3};
use super::rigid_body::off as rbo;
use super::solver;
use super::vsmath;

/// `dword_6CD69C` spin accumulator threshold (10000.0)
const SPIN_LIMIT: u32 = 0x461C4000;
/// `dword_6CD6AC` upward kick (10.0)
const SPIN_KICK: u32 = 0x41200000;
/// `dword_6CD6A8`
const PREFER_FIRST_CONTACT: i32 = 1;
/// `dword_7B5314` (bss, never written)
const ISOTROPIC_CLAMP: i32 = 0;
/// `dword_6CD6A0` / `dword_6CD6A4`
const SIDE_NORMAL_LO: u32 = 0x3E99999A;
const SIDE_NORMAL_HI: u32 = 0x3F333333;

fn dot32(a: V3, b: V3) -> f32 {
    vsmath::dot87(a, b) as f32
}

impl CFastCar {
    /// (FastCar) 0x00372B70
    pub fn tyre_solver(&mut self) {
        let mut solved = false;
        let up = self.m.v3(rbo::ROTATION + 16);
        let h = self.m.f(H);
        let relax = self.m.f(RELAXATION_STEPS);
        let free_count: i32 = (0..4).map(|i| self.m.i(WHEEL_FREE + 4 * i)).sum();
        let mut k34 = [0.0f32; 4];
        let mut k46 = [f32::from_bits(0xCCCCCCCC); 4];
        let mut k40 = [0.0f32; 4];
        for i in 0..4 {
            let m = &self.m;
            k34[i] = m.f(EFF_2492 + 4 * i) / (relax * h);
            if m.i(WHEEL_FREE + 4 * i) != 0 {
                k46[i] = (0.5 * m.f(EFF_2508 + 4 * i)) / ((relax * h) * free_count as f32);
            }
            k40[i] = (0.5 * m.f(EFF_2476 + 4 * i)) / (relax * h);
        }
        let mut upside_down = up[1] < 0.0;
        for i in 0..4 {
            if self.m.i(NCONTACT + 4 * i) > 0 {
                upside_down = dot32(self.m.v3(rec(i, 0) + R_NORMAL), up) <= 0.0;
                break;
            }
        }
        let mut _fn_sum = 0.0f32;
        for i in 0..4 {
            for k in 0..self.m.i(NCONTACT + 4 * i).max(0) as usize {
                let m = &mut self.m;
                let r = rec(i, k);
                let mut f = m.f(r + R_DEPTH) * m.f(SPRING + 4 * i);
                f = (0.0 - m.f(DAMPER + 4 * i)) * m.f(r + R_VN) + f;
                f = ((m.f(r + R_VN) * h) * m.f(SPRING + 4 * i)) + f;
                m.set_f(r + R_FN, f);
                _fn_sum += f;
                m.set_f(r + R_FT1, 0.0);
                m.set_f(r + R_FT2, 0.0);
            }
        }
        let m = &self.m;
        let one_each = (0..4).all(|i| m.i(NCONTACT + 4 * i) == 1);
        let none_sliding = (0..4).all(|i| m.i(rec(i, 0) + R_SLIDING) == 0);
        if !one_each || !none_sliding {
            self.m.set_i(ALL_ON_GROUND_FAIL, 1);
        } else {
            self.m.set_f(ALL_ON_GROUND_FAIL, 0.0);
            if self.m.i(IS_PLAYER) == 1 {
                solved = self.solve_drive(free_count, upside_down);
            }
        }
        // LABEL_93: viscous friction towards zero tangential velocity
        for i in 0..4 {
            for k in 0..self.m.i(NCONTACT + 4 * i).max(0) as usize {
                let m = &mut self.m;
                let r = rec(i, k);
                if m.i(r + R_SLIDING) == 0 && m.f(r + R_FN) > 0.0 {
                    let kk = if m.i(WHEEL_FREE + 4 * i) != 0 { k46[i] } else { k34[i] };
                    m.set_f(r + R_FT1, (0.0 - kk) * m.f(r + R_VT1) + m.f(r + R_FT1));
                    m.set_f(r + R_FT2, ((0.0 - m.f(BALANCE_RELAX)) * k40[i]) * m.f(r + R_VT2) + m.f(r + R_FT2));
                }
            }
        }
        if (self.m.i(SPIN_ACCUM) as f32) > f32::from_bits(SPIN_LIMIT) {
            let k = f32::from_bits(SPIN_KICK);
            let kick = [up[0] * k, up[1] * k, up[2] * k];
            let v = self.m.v3(rbo::VELOCITY);
            self.m.set_v3(rbo::VELOCITY, [v[0] + kick[0], v[1] + kick[1], v[2] + kick[2]]);
            self.m.set_f(SPIN_ACCUM, 0.0);
        }
        if (0..4).all(|i| self.m.i(WHEEL_FREE + 4 * i) == 0) {
            self.m.set_f(SPIN_ACCUM, 0.0);
        }
        if !solved {
            for i in 0..4 {
                let m = &mut self.m;
                if m.f(NET_TORQUE + 4 * i) == 0.0 {
                    continue;
                }
                let mut best: i32 = -1;
                let mut bestv = -1.0f32;
                for k in 0..m.i(NCONTACT + 4 * i).max(0) as usize {
                    if m.i(rec(i, k) + R_SLIDING) == 0 {
                        let v = vsmath::fabs(vsmath::dot87(up, m.v3(rec(i, k) + R_NORMAL)) as f32) as f32;
                        if v > bestv {
                            bestv = v;
                            best = k as i32;
                        }
                    }
                }
                if best == -1 {
                    continue;
                }
                if PREFER_FIRST_CONTACT != 0 && m.i(rec(i, 0) + R_SLIDING) == 0 {
                    best = 0;
                }
                let r = rec(i, best as usize);
                if m.f(r + R_FN) > 0.0 {
                    let net = vsmath::fabs(m.f(NET_TORQUE + 4 * i));
                    if (m.f(r + R_MU) * m.f(r + R_FN)) as f64 > net / m.f(RADIUS + 4 * i) as f64 {
                        let s = (0.0 - m.f(NET_TORQUE + 4 * i)) / m.f(RADIUS + 4 * i);
                        let t = m.v3(r + R_T1);
                        m.set_v3(WHEEL_DIR + 16 * i, [t[0] * s, t[1] * s, t[2] * s]);
                        m.set_i(WHEEL_DIR_SET + 4 * i, 1);
                    }
                }
            }
        }
        // sliding contacts: full dynamic friction against the slip direction
        for i in 0..4 {
            for k in 0..self.m.i(NCONTACT + 4 * i).max(0) as usize {
                let m = &mut self.m;
                let r = rec(i, k);
                if m.i(r + R_SLIDING) == 1 && m.f(r + R_FN) > 0.0 {
                    let lr = m.f(LATERAL_RATIO + 4 * i);
                    let lim = if lr <= 1.0 { m.f(r + R_FN) * m.f(r + R_MU) } else { (m.f(r + R_FN) * m.f(r + R_MU)) * lr };
                    let (vt1, vt2) = (m.f(r + R_VT1), m.f(r + R_VT2));
                    let s = vsmath::sqrt(vt1 * vt1 + vt2 * vt2) as f32;
                    if s == 0.0 {
                        m.set_f(r + R_FT1, 0.0);
                        m.set_f(r + R_FT2, 0.0);
                    } else {
                        m.set_f(r + R_FT1, ((0.0 - lim) * vt1) / s);
                        m.set_f(r + R_FT2, ((0.0 - lim) * vt2) / s);
                    }
                }
            }
        }
        // friction ellipse
        for i in 0..4 {
            for k in 0..self.m.i(NCONTACT + 4 * i).max(0) as usize {
                self.clamp_friction(i, rec(i, k));
            }
        }
        // static friction limited by what stops the slip within one step
        for i in 0..4 {
            let n = self.m.i(NCONTACT + 4 * i);
            if n <= 0 {
                continue;
            }
            let m = &mut self.m;
            let (mut s1, mut s2) = (0.0f32, 0.0f32);
            let (mut b1, mut b2) = (-1i32, -1i32);
            let (mut m1, mut m2) = (-1.0f32, -1.0f32);
            let active = |m: &super::mem::Mem, k: usize| m.i(rec(i, k) + R_SLIDING) != 0 && m.f(rec(i, k) + R_FN) > 0.0;
            for k in 0..n as usize {
                if active(m, k) {
                    s1 += m.f(rec(i, k) + R_FT1);
                    s2 += m.f(rec(i, k) + R_FT2);
                }
            }
            for k in 0..n as usize {
                if active(m, k) {
                    if m.f(rec(i, k) + R_FT1) * s1 > m1 {
                        m1 = m.f(rec(i, k) + R_FT1) * s1;
                        b1 = k as i32;
                    }
                    if m.f(rec(i, k) + R_FT2) * s2 > m2 {
                        m2 = m.f(rec(i, k) + R_FT2) * s2;
                        b2 = k as i32;
                    }
                }
            }
            if b1 == -1 {
                continue;
            }
            if s1 != 0.0 {
                let vt = m.f(rec(i, b1 as usize) + R_VT1) - 0.0;
                if m.i(WHEEL_FREE + 4 * i) != 0 {
                    let a = (0.0 - vt) * k46[i];
                    if a / s1 < 1.0 {
                        for k in 0..n as usize {
                            if active(m, k) {
                                m.set_f(rec(i, k) + R_FT1, m.f(rec(i, k) + R_FT1) * (a / s1));
                            }
                        }
                    }
                } else {
                    let tr = m.f(NET_TORQUE + 4 * i) / m.f(RADIUS + 4 * i);
                    if (tr + s1) * vt < 0.0 {
                        let a = (((0.0 - vt) * m.f(EFF_2492 + 4 * i)) / h) - tr;
                        if vt * a < 0.0 && a / s1 < 1.0 {
                            for k in 0..n as usize {
                                if active(m, k) {
                                    m.set_f(rec(i, k) + R_FT1, m.f(rec(i, k) + R_FT1) * (a / s1));
                                }
                            }
                        }
                    }
                }
            }
            if s2 != 0.0 {
                let vt = m.f(rec(i, b2.max(0) as usize) + R_VT2) - 0.0;
                let a = (0.0 - vt) * k40[i];
                if a / s2 < 1.0 {
                    for k in 0..n as usize {
                        if active(m, k) {
                            m.set_f(rec(i, k) + R_FT2, (a / s2) * m.f(rec(i, k) + R_FT2));
                        }
                    }
                }
            }
        }
        // 0x00377A30: contact force vector
        for i in 0..4 {
            for k in 0..self.m.i(NCONTACT + 4 * i).max(0) as usize {
                let m = &mut self.m;
                let r = rec(i, k);
                let (n, t1, t2) = (m.v3(r + R_NORMAL), m.v3(r + R_T1), m.v3(r + R_T2));
                let (a, b, c) = (m.f(r + R_FN), m.f(r + R_FT1), m.f(r + R_FT2));
                let f = [
                    (n[0] * a + t1[0] * b) + t2[0] * c,
                    (n[1] * a + t1[1] * b) + t2[1] * c,
                    (n[2] * a + t1[2] * b) + t2[2] * c,
                ];
                m.set_v3(r + R_FORCE, f);
            }
        }
        self.slip_ratios();
        self.suspension_limits(up);
    }

    /// friction ellipse clamp of one contact (part of 0x00372B70)
    fn clamp_friction(&mut self, i: usize, r: usize) {
        let m = &mut self.m;
        let (f1, f2) = (m.f(r + R_FT1), m.f(r + R_FT2));
        let mut a124 = f1 * f1;
        let mut a125 = f2 * f2;
        let len = vsmath::sqrt(a124 + a125) as f32;
        if !(m.f(r + R_FN) > 0.0) {
            return;
        }
        let lr = m.f(LATERAL_RATIO + 4 * i);
        let a130 = lr * lr;
        let a131 = m.f(r + R_MU) * m.f(r + R_FN);
        let ell = |a124: f32, a125: f32| (vsmath::sqrt((a124 + a125) / ((a125 / a130) + a124)) * a131 as f64) as f32;
        let a132 = ell(a124, a125);
        if !(len > a132 && (a124 + a125) > 0.0) {
            return;
        }
        let scale_both = |m: &mut super::mem::Mem, s: f32| {
            m.set_f(r + R_FT1, s * m.f(r + R_FT1));
            m.set_f(r + R_FT2, s * m.f(r + R_FT2));
        };
        if ISOTROPIC_CLAMP != 0 || lr == 1.0 {
            scale_both(m, a132 / len);
        } else if lr >= 1.0 {
            if lr > 1.0 {
                let v11 = vsmath::fabs(m.f(r + R_FT2));
                let reduce = |m: &mut super::mem::Mem| {
                    m.set_f(r + R_FT1, m.f(r + R_FT1) / lr);
                    let a124 = m.f(r + R_FT1) * m.f(r + R_FT1);
                    let e = ell(a124, a125);
                    let l = vsmath::sqrt(a124 + a125) as f32;
                    scale_both(m, e / l);
                };
                if (a131 * lr) as f64 <= v11 {
                    reduce(m);
                } else {
                    let a1760 = vsmath::sqrt(a131 * a131 - (a125 / a130)) as f32;
                    let a139 = (a1760 as f64 / vsmath::fabs(m.f(r + R_FT1))) as f32;
                    // comiss + jbe: a NaN ratio (rounded sqrt of a negative) takes this branch
                    if !(a139 > 1.0 / lr) {
                        reduce(m);
                    } else {
                        m.set_f(r + R_FT1, m.f(r + R_FT1) * a139);
                    }
                }
            }
        } else if a131 as f64 <= vsmath::fabs(m.f(r + R_FT1)) {
            m.set_f(r + R_FT2, m.f(r + R_FT2) * lr);
            a125 = m.f(r + R_FT2) * m.f(r + R_FT2);
            let e = ell(a124, a125);
            let l = vsmath::sqrt(a124 + a125) as f32;
            scale_both(m, e / l);
        } else {
            let v10 = vsmath::fabs(m.f(r + R_FT2));
            let a1759 = (lr as f64 / v10) as f32;
            let a133 = (vsmath::sqrt(a131 * a131 - a124) * a1759 as f64) as f32;
            if !(a133 > lr) {
                m.set_f(r + R_FT2, m.f(r + R_FT2) * lr);
                a125 = m.f(r + R_FT2) * m.f(r + R_FT2);
                a124 = m.f(r + R_FT1) * m.f(r + R_FT1);
                let e = ell(a124, a125);
                let l = vsmath::sqrt(a124 + a125) as f32;
                scale_both(m, e / l);
            } else {
                m.set_f(r + R_FT2, m.f(r + R_FT2) * a133);
            }
        }
    }

    /// slip ratio and slip speed per wheel (part of 0x00372B70)
    fn slip_ratios(&mut self) {
        for i in 0..4 {
            let m = &mut self.m;
            if m.i(NCONTACT + 4 * i) == 0 {
                m.set_f(SLIP + 4 * i, -1.0);
                m.set_f(MAX_SLIP_SPEED + 4 * i, -1.0);
                continue;
            }
            let mut maxv = -1.0f32;
            for k in 0..m.i(NCONTACT + 4 * i).max(0) as usize {
                let r = rec(i, k);
                let s = vsmath::sqrt(m.f(r + R_VT1) * m.f(r + R_VT1) + m.f(r + R_VT2) * m.f(r + R_VT2)) as f32;
                if s > maxv {
                    maxv = s;
                }
            }
            m.set_f(MAX_SLIP_SPEED + 4 * i, maxv);
            let r = rec(i, 0);
            let (f1, f2, fnn, mu) = (m.f(r + R_FT1), m.f(r + R_FT2), m.f(r + R_FN), m.f(r + R_MU));
            let v = if m.i(r + R_SLIDING) != 0 {
                2.0
            } else if m.i(WHEEL_DIR_SET + 4 * i) != 0 {
                let a = dot32(m.v3(WHEEL_DIR + 16 * i), m.v3(r + R_T1));
                (vsmath::sqrt((f1 + a) * (f1 + a) + f2 * f2) / (fnn * mu) as f64) as f32
            } else if fnn <= 0.0 {
                -2.0
            } else {
                let a160 = vsmath::sqrt(f1 * f1 + f2 * f2) as f32;
                let lr = m.f(LATERAL_RATIO + 4 * i);
                let a161 = lr * lr;
                let (a162, a163) = (f1 * f1, f2 * f2);
                let a1765 = mu * fnn;
                let a164 = (vsmath::sqrt((a162 + a163) / ((a163 / a161) + a162)) * a1765 as f64) as f32;
                if vsmath::fabs((a163 / a161) + a162) <= 0.0 {
                    0.0
                } else {
                    a160 / a164
                }
            };
            m.set_f(SLIP + 4 * i, v);
        }
    }

    /// suspension travel limits: rest positions and chassis offset (part of 0x00372B70)
    fn suspension_limits(&mut self, up: V3) {
        let mut pen = [0.0f32; 8];
        let mut pidx = [-1i32; 8];
        let mut pcount = [0usize; 4];
        if self.m.i(FLAG_2244) != 0 {
            for i in 0..4 {
                for k in 0..self.m.i(NCONTACT + 4 * i).max(0) as usize {
                    let m = &mut self.m;
                    let r = rec(i, k);
                    let a184 = dot32(m.v3(r + R_NORMAL), up);
                    let a185 = vsmath::fabs(a184) as f32;
                    let (lo, hi) = (f32::from_bits(SIDE_NORMAL_LO), f32::from_bits(SIDE_NORMAL_HI));
                    let (a186, a187) = if a185 <= hi {
                        if lo <= a185 {
                            let x = (a185 - lo) / (hi - lo);
                            (x, 1.0 - x)
                        } else {
                            (0.0, 1.0)
                        }
                    } else {
                        (1.0, 0.0)
                    };
                    if a186 != 0.0 {
                        let a = vsmath::fabs((m.f(r + R_DEPTH) * a186) / a184) as f32;
                        if a as f64 > vsmath::fabs(m.f(SUSP_REST + 4 * i)) {
                            m.set_f(SUSP_REST + 4 * i, (m.f(r + R_DEPTH) * a186) / a184);
                        }
                    }
                    if a187 != 0.0 {
                        let slot = (i + pcount[i]).min(7);
                        pen[slot] = m.f(r + R_DEPTH) * a187;
                        pidx[slot] = k as i32;
                        pcount[i] += 1;
                    }
                }
            }
        }
        if self.m.i(SUSP_LIMITS_CH) != 0 {
            log::warn!("FastCar: suspension_limits_through_constraint_handler is not ported");
        } else {
            for i in 0..4 {
                let m = &mut self.m;
                if m.i(NCONTACT + 4 * i) <= 0 {
                    continue;
                }
                let (lo, hi, rest) = (m.f(LOWER_LIMIT + 4 * i), m.f(UPPER_LIMIT + 4 * i), m.f(SUSP_REST + 4 * i));
                if hi == 0.0 || rest <= hi {
                    if lo != 0.0 && lo > rest {
                        if pcount[i] == 0 {
                            pcount[i] = 1;
                            pidx[i] = 0;
                        }
                        let d = dot32(m.v3(rec(i, pidx[i] as usize) + R_NORMAL), up);
                        pen[i] = pen[i] - d * (lo - rest);
                        m.set_u(SUSP_REST + 4 * i, m.u(LOWER_LIMIT + 4 * i));
                    }
                } else {
                    if pcount[i] == 0 {
                        pcount[i] = 1;
                        pidx[i] = 0;
                    }
                    let d = dot32(m.v3(rec(i, pidx[i] as usize) + R_NORMAL), up);
                    pen[i] = d * (rest - hi) + pen[i];
                    m.set_u(SUSP_REST + 4 * i, m.u(UPPER_LIMIT + 4 * i));
                }
            }
        }
        let m = &mut self.m;
        if m.i(COUNTDOWN_2968) == -3 {
            m.set_i(COUNTDOWN_2968, 3);
        }
        if m.i(COUNTDOWN_2968) > 0 {
            m.set_i(COUNTDOWN_2968, m.i(COUNTDOWN_2968) - 1);
        }
        if m.i(FLAG_2244) == 0 {
            return;
        }
        let mut iter = 0;
        loop {
            let mut thr = m.f(RADIUS) / 1000.0;
            let (mut bi, mut bk) = (-1i32, -1i32);
            for i in 0..4 {
                if m.i(NCONTACT + 4 * i) > 0 {
                    for k in 0..pcount[i] {
                        if pen[(i + k).min(7)] > thr {
                            thr = pen[(i + k).min(7)];
                            bi = i as i32;
                            bk = k as i32;
                        }
                    }
                }
            }
            if bi != -1 {
                let (bi, bk) = (bi as usize, bk as usize);
                let bslot = (bi + bk).min(7);
                let bn = m.v3(rec(bi, pidx[bslot] as usize) + R_NORMAL);
                m.set_v3(SUSP_OFFSET, madd(m.v3(SUSP_OFFSET), bn, pen[bslot]));
                for i in 0..4 {
                    if m.i(NCONTACT + 4 * i) > 0 {
                        for k in 0..pcount[i] {
                            if i != bi || k != bk {
                                let s = (i + k).min(7);
                                let d = vsmath::dot87(m.v3(rec(i, pidx[s] as usize) + R_NORMAL), bn);
                                pen[s] = (pen[s] as f64 - d * pen[bslot] as f64) as f32;
                            }
                        }
                    }
                }
                pen[bslot] = 0.0;
            }
            iter += 1;
            if bi == -1 || iter >= 10 {
                break;
            }
        }
    }

    /// the player all-wheels-on-ground drive solver (part of 0x00372B70); returns true when solved
    fn solve_drive(&mut self, free_count: i32, upside_down: bool) -> bool {
        let m = &mut self.m;
        let mut nsum = [0.0f32; 3];
        let mut c = [0.0f32; 3];
        for i in 0..4 {
            let n = m.v3(rec(i, 0) + R_NORMAL);
            nsum = [nsum[0] + n[0], nsum[1] + n[1], nsum[2] + n[2]];
            let p = m.v3(WHEEL_POS + 16 * i);
            c = [c[0] + p[0], c[1] + p[1], c[2] + p[2]];
        }
        nsum = normalize_keep(nsum);
        c = [c[0] * 0.25, c[1] * 0.25, c[2] * 0.25];
        for i in 0..4 {
            let p = m.v3(WHEEL_POS + 16 * i);
            let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
            let l = vsmath::sqrt(vsmath::dot87(d, d) as f32) as f32;
            let dn = normalize_keep(d);
            nsum = madd(nsum, dn, m.f(rec(i, 0) + R_DEPTH) / (2.0 * l));
        }
        nsum = normalize_keep(nsum);
        let g = m.v3(rbo::GRAVITY);
        let gn = dot32(g, nsum);
        let gt = madd(g, nsum, 0.0 - gn);
        let gx = dot32(m.v3(rbo::ROTATION), gt);
        let gz = dot32(m.v3(rbo::ROTATION + 32), gt);
        for i in 0..4 {
            if m.i(WHEEL_FREE + 4 * i) == 0 {
                continue;
            }
            if vsmath::fabs(m.f(STEERING)) > 0.1f32 as f64 {
                let add = ((m.i(NCONTACT + 4 * i) as f32 * m.f(STEERING)) * 16.0) as i32;
                m.set_i(SPIN_ACCUM, m.i(SPIN_ACCUM).wrapping_add(add));
            }
            let r = rec(i, 0);
            let grip = (m.f(r + R_FN) * m.f(r + R_MU)) * m.f(RADIUS + 4 * i);
            let a = dot32(m.v3(r + R_T1), gt);
            let t = (((1.0 / free_count as f32) * m.f(rbo::MASS)) * m.f(RADIUS + 4 * i)) * a;
            m.set_f(NET_TORQUE + 4 * i, t);
            if vsmath::fabs(t) > vsmath::fabs(grip) {
                let s = vsmath::fabs(grip / t);
                m.set_f(NET_TORQUE + 4 * i, (s * t as f64) as f32);
            }
        }
        let (r0, r1) = if upside_down {
            (m.f(RADIUS), m.f(RADIUS + 4))
        } else {
            (0.0 - m.f(RADIUS), 0.0 - m.f(RADIUS + 4))
        };
        let net = m.v4(NET_TORQUE);
        let mut out = [0.0f32; 36];
        if vsmath::fabs(m.f(STEERING)) > 0.001f32 as f64 {
            debug_assert!(m.i(FLAG_1320) != 0);
            solver::solve_steered(
                &mut out,
                &net,
                0.0,
                m.f(1820),
                m.f(1828),
                0.0 - m.f(1836),
                m.f(1844),
                gx,
                gz,
                r0,
                r1,
                m.f(rbo::MASS),
                m.f(100),
                1.0 / m.f(1476),
                1.0 / m.f(1492),
                m.f(WHEEL_STEER),
            );
            for (k, v) in out.iter().enumerate() {
                m.set_f(SOLVER + 4 * k, *v);
            }
            // 0x00372560: two-parameter least squares onto the per-wheel torque vectors
            let a2 = |row: usize, col: usize| m.f(SOLVER + 16 * row + 4 * col);
            let (mut s7, mut s6, mut s5, mut s4, mut s3) = (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
            for r in 0..3 {
                for c in 0..4 {
                    s7 = a2(r, c) * a2(r, c) + s7;
                    s6 = a2(r, c) * a2(r + 3, c) + s6;
                    s5 = a2(r, c) * a2(r + 6, c) + s5;
                    s4 = a2(r + 3, c) * a2(r + 3, c) + s4;
                    s3 = a2(r + 3, c) * a2(r + 6, c) + s3;
                }
            }
            let d = -1.0 / (s7 * s4 - s6 * s6);
            let p = (s4 * s5 - s6 * s3) * d;
            let q = ((0.0 - s6) * s5 + s7 * s3) * d;
            let mut td = [0.0f32; 12];
            for c in 0..4 {
                td[3 * c] = (a2(0, c) * p + a2(3, c) * q) + a2(6, c);
                td[3 * c + 1] = (a2(1, c) * p + a2(4, c) * q) + a2(7, c);
                td[3 * c + 2] = (a2(2, c) * p + a2(5, c) * q) + a2(8, c);
            }
            for c in 0..4 {
                m.set_v3(TORQUE_DIR + 16 * c, [td[3 * c], td[3 * c + 1], td[3 * c + 2]]);
            }
        } else {
            solver::solve_straight(
                &mut out,
                &net,
                m.f(1820),
                m.f(1828),
                0.0 - m.f(1836),
                m.f(1844),
                gx,
                gz,
                r0,
                r1,
                m.f(rbo::MASS),
                1.0 / m.f(1476),
                1.0 / m.f(1492),
            );
            for (k, v) in out.iter().take(16).enumerate() {
                if k < 15 {
                    m.set_f(TORQUE_DIR + 4 * k, *v);
                }
            }
        }
        // LABEL_87: project the body-frame wheel forces onto the contact frames
        let rot = m.m34(rbo::ROTATION);
        for i in 0..4 {
            let v = super::mem::mul_cols(&rot, m.v3(TORQUE_DIR + 16 * i));
            let r = rec(i, 0);
            let a = vsmath::dot87(v, m.v3(r + R_NORMAL));
            m.set_f(r + R_FN, (a + m.f(r + R_FN) as f64) as f32);
            m.set_f(r + R_FT1, dot32(v, m.v3(r + R_T1)));
            m.set_f(r + R_FT2, dot32(v, m.v3(r + R_T2)));
        }
        true
    }
}
