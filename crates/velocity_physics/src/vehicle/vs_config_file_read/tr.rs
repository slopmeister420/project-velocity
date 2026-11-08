//! Translated from the decompilation by tools/phys_c2rs4.py (do not edit by hand; fixes go in the
//! translator or in the hand-written overrides).
#![allow(non_snake_case, unused_parens, unused_labels, unused_braces, unused_variables, unused_mut, unused_assignments, unreachable_code, dead_code, clippy::all)]

use crate::tr_prelude::*;

/// VsConfigFileRead__1B59C0 0x001B59C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b59c0(cx: &mut Cx, this: Ptr, a2: i32) -> i32 {
    let mut i = 0i32;
    i = 0;
    'lp2: loop {
        if !(i < cx.i(this.add(40))) { break 'lp2; }
        'blp2: {
            if ({ let _ = cx.i(cx.ptr_at(this.add(8)).add((4_i32).wrapping_mul(i))); let _ = a2; unmodelled_i("sub_3DF020") } == 0) {
                return cx.i(cx.ptr_at(this.add(12)).add((4_i32).wrapping_mul(i)));
            }
        }
        { i += 1; i };
    }
    return 0;
}

/// VsConfigFileRead__1B6C30 0x001B6C30
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b6c30(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = ov_memset(cx, this, 0, 284);
    { let p = this.add(88); let t = 1065353216; cx.set_i(p, t); }
    { let p = this.add(252); let t = 1065353216; cx.set_i(p, t); }
    return this;
}

/// VsConfigFileRead__1B6C90 0x001B6C90
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b6c90(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = ov_memset(cx, this, 0, 208);
    return this;
}

/// VsConfigFileRead__1B6CD0 0x001B6CD0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b6cd0(cx: &mut Cx, mut this: Ptr) -> Ptr {
    let mut v2 = Ptr::NULL;
    let mut v3 = Ptr::NULL;
    crate::ov_ignore!(cx, 0 /*str*/);
    { let q0_ = cx.ptr_at(this); ov_delete(cx, q0_) };
    v3 = ov_new(cx, 1040);
    if (v3.bits() != 0) {
        ov_vector_ctor(cx, v3, 208, 5, Ptr::raw(Ptr::func(0x1B6C90).bits()));
        v2 = v3;
    } else {
        v2 = Ptr::NULL;
    }
    { let p = this; let t = v2; cx.set_ptr(p, t); }
    crate::ov_ignore!(cx);
    { let p = this.add(4); let t = 0; cx.set_i(p, t); }
    { let p = this.add(8); let t = 1; cx.set_i(p, t); }
    { let p = this.add(12); let t = 0; cx.set_i(p, t); }
    { let p = this.add(276); let t = 1065353216; cx.set_i(p, t); }
    { let p = this.add(160); let t = 1065353216; cx.set_i(p, t); }
    return this;
}

/// VsConfigFileRead__1B6F10 0x001B6F10
pub fn f_1b6f10(cx: &mut Cx, a1: Ptr) -> i64 {
    cx.push_frame(124);
    let r = f_1b6f10_body(cx, a1);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1b6f10_body(cx: &mut Cx, a1: Ptr) -> i64 {
    let mut v1 = 0.0f64;
    let mut v2 = 0i64;
    let mut v5 = 0i64;
    let mut _1B74A0 = 0i32;
    { let t = a1; cx.set_lpp(30, t); }
    { let t = ov_hist_entry(cx, a1.add(3560), 0); cx.set_lpp(29, t); }
    if (cx.b(cx.lpp(30).add(4900)) != 0) {
        v1 = ({ let q0_ = cx.lpp(30).add(240); f_35d090(cx, q0_) } as f64);
        v2 = (v2 & 0xFFFF_FFFFi64) | (((Ptr::gc(0).bits()) as u32 as i64) << 32);
        { let p = cx.lptr(112); let t = ((v1 * (cx.f(Ptr::gc(0).add(352)) as f64)) as f32); cx.set_f(p, t); }
        { let p = cx.lpp(30).add(4068); let t = cx.li(112); cx.set_i(p, t); }
        { let t = ((((cx.f(cx.lptr(112)) / 1000.0f32) * ((cx.i(Ptr::glob(0x6D4D98)) as f32) - 1000.0f32)) + 1000.0f32) as i32); cx.set_li(108, t); }
        v2 = (v2 & !0xFFFF_FFFFi64) | ((cx.li(108)) as i32 as u32 as i64);
    } else {
        { let t = 0; cx.set_li(104, t); }
        _1B74A0 = { let q0_ = cx.lpp(30); f_1b74a0(cx, q0_) };
        { let p = cx.lpp(30).add(3216); let t = _1B74A0; cx.set_i(p, t); }
        if (cx.i(cx.lpp(30).add(3216)) > 90) {
            { let p = cx.lpp(30).add(3216); let t = cx.i(p).wrapping_sub(100); cx.set_i(p, t); }
            { let t = 1; cx.set_li(104, t); }
        }
        { let p = cx.lptr(100); let t = ((({ let q0_ = cx.lpp(30).add(240); f_35d090(cx, q0_) } as f64) * (cx.f(Ptr::gc(0).add(352)) as f64)) as f32); cx.set_f(p, t); }
        if ((cx.f(cx.lpp(29).add(36)) != 0.0f32) && (cx.i(cx.lpp(30).add(3740)) > 0)) {
            { let p = cx.lptr(100); let t = (((cx.f(cx.lptr(100)) as f64) * (cx.f(Ptr::gc(0).add(356)) as f64)) as f32); cx.set_f(p, t); }
        }
        if (cx.li(104) != 0) {
            { let t = cx.i(cx.lpp(29).add(36)); cx.set_li(80, t); }
            { let p = cx.lptr(84); let t = (cx.i(cx.lpp(30).add(4048)) as f32); cx.set_f(p, t); }
            { let p = cx.lptr(88); let t = (cx.i(cx.lpp(30).add(3216)) as f32); cx.set_f(p, t); }
            { let t = cx.i(cx.lpp(30).add(4068)); cx.set_li(92, t); }
            { let t = { let q0_ = cx.lptr(68); let q1_ = 0; let q2_ = cx.ptr_at(Ptr::gc(0).add(376)); let q3_ = cx.lptr(80); ov_eval_expr(cx, q0_, q1_, q2_, q3_) }; cx.set_li(60, t); }
            if ((cx.li(60) != 0) || (vsmath::fabs(cx.f(cx.lptr(68))) <= 1.0f64)) {
                { let t = 0; cx.set_li(104, t); }
            } else {
                { let t = cx.li(68); cx.set_li(100, t); }
            }
        }
        if (cx.li(104) == 0) {
            if ((cx.i(cx.lpp(30).add(3216)) > 1) || (cx.f(cx.lpp(29).add(36)) == 0.0f32)) {
                if ((cx.f(cx.lpp(29).add(36)) == 0.0f32) && (cx.f(cx.lptr(100)) > cx.f(cx.lpp(30).add(4068)))) {
                    { let p = cx.lptr(56); let t = (cx.i(Ptr::gc(0).add(340)) as f32); cx.set_f(p, t); }
                } else if ({ let q0_ = cx.lpp(30).add(240); f_364b20(cx, q0_) } == 1) {
                    { let p = cx.lptr(56); let t = (cx.i(Ptr::gc(0).add(336)) as f32); cx.set_f(p, t); }
                } else {
                    { let p = cx.lptr(56); let t = (cx.i(Ptr::gc(0).add(328)) as f32); cx.set_f(p, t); }
                }
            } else {
                { let p = cx.lptr(56); let t = (cx.i(Ptr::gc(0).add(332)) as f32); cx.set_f(p, t); }
            }
            if (cx.f(cx.lptr(100)) <= (cx.f(cx.lpp(30).add(4068)) + cx.f(cx.lptr(56)))) {
                if ((cx.f(cx.lpp(30).add(4068)) - (cx.i(Ptr::gc(0).add(328)) as f32)) > cx.f(cx.lptr(100))) {
                    { let p = cx.lptr(100); let t = (((cx.f(cx.lpp(30).add(4068)) as f64) - ((cx.i(Ptr::gc(0).add(328)) as f32) as f64)) as f32); cx.set_f(p, t); }
                }
            } else {
                { let p = cx.lptr(100); let t = ((((cx.i(Ptr::gc(0).add(328)) as f32) as f64) + (cx.f(cx.lpp(30).add(4068)) as f64)) as f32); cx.set_f(p, t); }
            }
        }
        if ((cx.f(cx.lpp(29).add(36)) == 0.0f32) || (cx.i(cx.lpp(30).add(3740)) <= 0)) {
            { let p = cx.lptr(100); let t = (((((1.0f32 - cx.f(Ptr::glob(0x688A70))) * cx.f(cx.lpp(30).add(4068))) as f64) + ((cx.f(cx.lptr(100)) * cx.f(Ptr::glob(0x688A70))) as f64)) as f32); cx.set_f(p, t); }
        } else {
            { let p = cx.lptr(100); let t = (((((1.0f32 - cx.f(Ptr::gc(0).add(368))) * cx.f(cx.lpp(30).add(4068))) as f64) + ((cx.f(cx.lptr(100)) * cx.f(Ptr::gc(0).add(368))) as f64)) as f32); cx.set_f(p, t); }
        }
        v2 = (v2 & 0xFFFF_FFFFi64) | (((cx.lpp(30).bits()) as u32 as i64) << 32);
        { let p = cx.lpp(30).add(4068); let t = cx.li(100); cx.set_i(p, t); }
        { let t = ((((cx.f(cx.lptr(100)) / 1000.0f32) * ((cx.i(Ptr::glob(0x6D4D98)) as f32) - 1000.0f32)) + 1000.0f32) as i32); cx.set_li(52, t); }
        { let p = cx.lptr(48); let t = ((((cx.li(52) as f32) as f64) / (cx.f(Ptr::glob(0x6889AC)) as f64)) as f32); cx.set_f(p, t); }
        if ((cx.f(cx.lptr(48)) > 0.0f32) && (cx.f(cx.lptr(48)) < 1.0f32)) {
            { let t = cx.li(48); cx.set_li(44, t); }
            { let t = cx.i(Ptr::glob(0x688A6C)).wrapping_sub(1); cx.set_li(40, t); }
            'lp2: loop {
                if !(cx.li(40) > 0) { break 'lp2; }
                'blp2: {
                    { let p = cx.lptr(44); let t = (((cx.f(cx.lptr(44)) as f64) * (cx.f(cx.lptr(48)) as f64)) as f32); cx.set_f(p, t); }
                }
                v2 = (v2 & 0xFFFF_FFFFi64) | (((cx.predec_i(cx.lptr(40))) as u32 as i64) << 32);
            }
            { let p = cx.lptr(44); let t = (((cx.f(cx.lptr(44)) as f64) * (cx.f(Ptr::glob(0x6889AC)) as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lptr(44); let t = (((cx.f(cx.lptr(44)) as f64) * (cx.f(Ptr::glob(0x688A68)) as f64)) as f32); cx.set_f(p, t); }
            { let t = ((((cx.li(52) as f32) * (1.0f32 - cx.f(Ptr::glob(0x688A68)))) + cx.f(cx.lptr(44))) as i32); cx.set_li(52, t); }
        }
        if (cx.li(104) == 0) {
            { let t = cx.i(cx.lpp(29).add(36)); cx.set_li(20, t); }
            { let p = cx.lptr(24); let t = (cx.i(cx.lpp(30).add(4048)) as f32); cx.set_f(p, t); }
            { let p = cx.lptr(28); let t = (cx.i(cx.lpp(30).add(3216)) as f32); cx.set_f(p, t); }
            { let p = cx.lptr(32); let t = (cx.li(52) as f32); cx.set_f(p, t); }
            { let t = { let q0_ = cx.lptr(8); let q1_ = 0; let q2_ = cx.ptr_at(Ptr::gc(0).add(380)); let q3_ = cx.lptr(20); ov_eval_expr(cx, q0_, q1_, q2_, q3_) }; cx.set_li(0, t); }
            if (cx.li(0) == 0) {
                v2 = (v2 & 0xFFFF_FFFFi64) | (((cx.li(52).wrapping_add((cx.f(cx.lptr(8)) as i32))) as u32 as i64) << 32);
                { let t = ((v2 >> 32) as i32); cx.set_li(52, t); }
            }
        }
        if (cx.i(cx.lpp(30).add(3820)) != 0) {
            { let t = (((cx.li(52) as f32) * cx.f(Ptr::glob(0x6889A8))) as i32); cx.set_li(52, t); }
        }
        v2 = (v2 & !0xFFFF_FFFFi64) | ((cx.li(52)) as i32 as u32 as i64);
    }
    v5 = v2;
    return v5;
}

/// VsConfigFileRead__1B74A0 0x001B74A0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b74a0(cx: &mut Cx, this: Ptr) -> i32 {
    let mut v1 = 0i32;
    let mut v3 = 0i32;
    let mut v4 = 0i32;
    v4 = cx.i(this.add(3216));
    if (f_364b20(cx, this.add(240)) == 1) {
        v1 = cx.i(Ptr::gc(0).add(348));
    } else {
        v1 = cx.i(Ptr::gc(0).add(344));
    }
    if ((v4 == cx.i(this.add(4052))) || (((cx.i(this.add(4048)) <= cx.i(Ptr::gc(0).add(372)).wrapping_add(v1)) && ((cx.i(this.add(4052)) > 0) || (v4 <= 0))) && ((cx.i(this.add(4052)) < 0) || (v4 >= 0)))) {
        v3 = cx.i(this.add(4052));
    } else {
        if (v4 <= cx.i(this.add(4052))) {
            v3 = v4;
        } else {
            v3 = cx.i(this.add(4052)).wrapping_add(1);
            if (cx.i(this.add(4056)) == 0) {
                if (cx.i(this.add(4052)) == 1) {
                    { let p = this.add(4060); let t = cx.i(Ptr::gc(0).add(464)); cx.set_i(p, t); }
                    { let p = this.add(4056); let t = cx.i(Ptr::gc(0).add(460)); cx.set_i(p, t); }
                } else if (cx.i(this.add(4052)) == 2) {
                    { let p = this.add(4060); let t = cx.i(Ptr::gc(0).add(468)); cx.set_i(p, t); }
                    { let p = this.add(4056); let t = cx.i(Ptr::gc(0).add(460)); cx.set_i(p, t); }
                }
            }
        }
        { let p = this.add(4048); let t = 0; cx.set_i(p, t); }
    }
    if (v3 != cx.i(this.add(4052))) {
        { let p = this.add(4064); let t = cx.i(this.add(4052)); cx.set_i(p, t); }
    }
    { let p = this.add(4052); let t = v3; cx.set_i(p, t); }
    if ((((cx.i(this.add(4048)) < cx.i(Ptr::gc(0).add(372))) && (v3 != 0)) && (v3 != -1)) && (cx.i(this.add(4064)) != 0)) {
        v3 = v3.wrapping_add(100);
    }
    return v3;
}

/// VsConfigFileRead__1B77E0 0x001B77E0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b77e0(cx: &mut Cx, mut this: Ptr) -> Ptr {
    { let p = this; let t = 1045220557; cx.set_i(p, t); }
    { let p = this.add(4); let t = 1061997773; cx.set_i(p, t); }
    { let p = this.add(8); let t = 1061997773; cx.set_i(p, t); }
    { let p = this.add(12); let t = 1090865070; cx.set_i(p, t); }
    { let p = this.add(16); let t = 1102168719; cx.set_i(p, t); }
    { let p = this.add(20); let t = 1057350484; cx.set_i(p, t); }
    { let p = this.add(24); let t = 1035489772; cx.set_i(p, t); }
    { let p = this.add(28); let t = 1077936128; cx.set_i(p, t); }
    { let p = this.add(32); let t = 1106247680; cx.set_i(p, t); }
    { let p = this.add(36); let t = 1106247680; cx.set_i(p, t); }
    { let p = this.add(40); let t = 1073741824; cx.set_i(p, t); }
    return this;
}

/// VsConfigFileRead__1B7D50 0x001B7D50
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b7d50(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = f_18cb00(cx, this.add(32));
    let _ = f_1b7d90(cx, this);
    return this;
}

/// VsConfigFileRead__1B7D90 0x001B7D90
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b7d90(cx: &mut Cx, mut this: Ptr) -> Ptr {
    { let p = this; let t = 0; cx.set_i(p, t); }
    { let p = this.add(4); let t = 0; cx.set_i(p, t); }
    { let p = this.add(8); let t = 0; cx.set_i(p, t); }
    { let p = this.add(12); let t = 0; cx.set_i(p, t); }
    { let p = this.add(16); let t = 0; cx.set_i(p, t); }
    { let p = this.add(20); let t = 0; cx.set_i(p, t); }
    { let p = this.add(25); let t = 0; cx.set_u16(p, t); }
    { let p = this.add(27); let t = 1; cx.set_b(p, t); }
    { let p = this.add(28); let t = 0; cx.set_b(p, t); }
    return this;
}

/// VsConfigFileRead__1B8260 0x001B8260
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b8260(cx: &mut Cx, a1: f32, a2: f32, hi: f32) -> f32 {
    return (((vsmath::clamp((((a2 as f64) - (a1 as f64)) as f32), ((0.0f64 - (hi as f64)) as f32), hi) as f64) + (a1 as f64)) as f32);
}

/// VsConfigFileRead__1B82A0 0x001B82A0
pub fn f_1b82a0(cx: &mut Cx, a1: Ptr, a2: i32, mut a3: f32, a4: i32, mut a5: f32, a6: i32, a7: i32) -> () {
    cx.push_frame(44);
    let r = f_1b82a0_body(cx, a1, a2, a3, a4, a5, a6, a7);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1b82a0_body(cx: &mut Cx, a1: Ptr, a2: i32, mut a3: f32, a4: i32, mut a5: f32, a6: i32, a7: i32) -> () {
    let mut v8 = 0.0f64;
    let mut v9 = 0.0f64;
    let mut v10 = 0.0f64;
    let mut v11 = 0.0f32;
    let mut _1B8260 = 0.0f64;
    { let t = a1; cx.set_lpp(10, t); }
    if (a7 != 0) {
        a5 = ((((1.0f32 - cx.f(Ptr::glob(0x6889B0))) as f64) * (a5 as f64)) as f32);
    }
    { let t = ov_traffic(cx).add((36_i32).wrapping_mul(a4)).add(24); cx.set_lpp(9, t); }
    { let p = cx.lpp(10).add(28); let t = a7; cx.set_b(p, t); }
    if (a7 != 0) {
        a3 = (((((1.0f32 - cx.f(Ptr::glob(0x6F5B70))) * a3) as f64) + ((((((cx.f(cx.lpp(10).add(72)) - 1.0f32) * cx.f(Ptr::glob(0x74BA78))) + 1.0f32) * a3) * cx.f(Ptr::glob(0x6F5B70))) as f64)) as f32);
    }
    if (a4 <= -1) {
        if (cx.b(cx.lpp(10).add(21)) != 0) {
            { let t = 0.0f32; cx.set_lf(28, t); }
            { let t = cx.f(cx.lpp(10).add(8)); cx.set_lf(24, t); }
        } else {
            { let t = cx.f(cx.lpp(10).add(8)); cx.set_lf(28, t); }
            { let t = 0.0f32; cx.set_lf(24, t); }
        }
        { let t = ((-{ let q0_ = ((0.0f64 - (cx.f(cx.lpp(10)) as f64)) as f32); let q1_ = cx.f(cx.lpp(10).add(32)); let q2_ = cx.f(cx.lpp(10).add(36)); let q3_ = cx.f(cx.lpp(10).add(40)); f_1b86c0(cx, q0_, q1_, q2_, q3_) }) as f32); cx.set_lf(20, t); }
    } else {
        { let t = cx.f(cx.lpp(9)); cx.set_lf(32, t); }
        { let t = ({ let q0_ = cx.lf(32); let q1_ = cx.f(cx.lpp(9).add(4)); f_57f90(cx, q0_, q1_) } as f32); cx.set_lf(28, t); }
        { let t = ({ let q0_ = ((0.0f64 - (cx.lf(32) as f64)) as f32); let q1_ = cx.f(cx.lpp(9).add(8)); f_57f90(cx, q0_, q1_) } as f32); cx.set_lf(24, t); }
        { let t = cx.f(cx.lpp(9).add(12)); cx.set_lf(20, t); }
        { let t = ((-{ let q0_ = cx.lf(20); let q1_ = cx.f(cx.lpp(10).add(32)); let q2_ = cx.f(cx.lpp(10).add(36)); let q3_ = cx.f(cx.lpp(10).add(40)); f_1b86c0(cx, q0_, q1_, q2_, q3_) }) as f32); cx.set_lf(20, t); }
        { let p = cx.lpp(10).add(22); let t = (((cx.i(cx.lpp(9).add(16)) & 4) != 0) as i32); cx.set_b(p, t); }
        { let p = cx.lpp(10).add(23); let t = (((cx.i(cx.lpp(9).add(16)) & 8) != 0) as i32); cx.set_b(p, t); }
        { let p = cx.lpp(10).add(25); let t = (cx.i(cx.lpp(9).add(16)) & 1); cx.set_b(p, t); }
        { let p = cx.lpp(10).add(20); let t = (((cx.i(cx.lpp(9).add(16)) & 64) != 0) as i32); cx.set_b(p, t); }
        if (0.0f64 != { let q0_ = cx.f(Ptr::glob(0x737AB0)); let q1_ = a4; ov_zone_fx_amount(cx, q0_, q1_) }) {
            { let p = cx.lpp(10).add(20); let t = 0; cx.set_b(p, t); }
        }
    }
    if (cx.b(cx.lpp(10).add(21)) != 0) {
        if ((a5 > -0.1f32) && (cx.lf(28) > 0.0f32)) {
            { let p = cx.lpp(10).add(21); let t = 0; cx.set_b(p, t); }
        }
    } else if (((a5 < 0.1f32) && (a5 > -0.1f32)) && (cx.lf(24) > 0.0f32)) {
        { let p = cx.lpp(10).add(21); let t = 1; cx.set_b(p, t); }
    }
    if (cx.b(cx.lpp(10).add(21)) != 0) {
        { let t = cx.lf(24); cx.set_lf(8, t); }
    } else {
        { let t = cx.lf(28); cx.set_lf(8, t); }
    }
    { let t = cx.lf(8); cx.set_lf(16, t); }
    if (cx.b(cx.lpp(10).add(21)) != 0) {
        { let t = cx.lf(28); cx.set_lf(4, t); }
    } else {
        { let t = cx.lf(24); cx.set_lf(4, t); }
    }
    { let t = cx.lf(4); cx.set_lf(12, t); }
    _1B8260 = ({ let q0_ = cx.f(cx.lpp(10)); let q1_ = cx.lf(20); let q2_ = (((cx.f(cx.lpp(10).add(60)) as f64) * (a3 as f64)) as f32); f_1b8260(cx, q0_, q1_, q2_) } as f64);
    { let p = cx.lpp(10); let t = (_1B8260 as f32); cx.set_f(p, t); }
    v11 = (vsmath::fabs(a5) as f32);
    v8 = ({ let q0_ = cx.lpp(10).add(32); let q1_ = cx.f(cx.lpp(10)); let q2_ = (v11.to_bits() as i32); f_1b8980(cx, q0_, q1_, q2_) } as f64);
    { let p = cx.lpp(10).add(4); let t = (v8 as f32); cx.set_f(p, t); }
    v9 = ({ let q0_ = cx.f(cx.lpp(10).add(8)); let q1_ = cx.lf(16); let q2_ = (((cx.f(cx.lpp(10).add(64)) as f64) * (a3 as f64)) as f32); f_1b8260(cx, q0_, q1_, q2_) } as f64);
    { let p = cx.lpp(10).add(8); let t = (v9 as f32); cx.set_f(p, t); }
    v10 = ({ let q0_ = cx.f(cx.lpp(10).add(12)); let q1_ = cx.lf(12); let q2_ = (((cx.f(cx.lpp(10).add(68)) as f64) * (a3 as f64)) as f32); f_1b8260(cx, q0_, q1_, q2_) } as f64);
    { let p = cx.lpp(10).add(12); let t = (v10 as f32); cx.set_f(p, t); }
    if (((vsmath::fabs(a5) >= 1.0f64) || (cx.f(cx.lpp(10).add(8)) >= 0.01f32)) || (cx.f(cx.lpp(10).add(16)) >= 0.2f32)) {
        if (a4 > -1) {
            if ((cx.i(cx.lpp(9).add(16)) & 2) != 0) {
                { let t = 1.0f32; cx.set_lf(0, t); }
            } else {
                { let t = 0.0f32; cx.set_lf(0, t); }
            }
            { let p = cx.lpp(10).add(16); let t = cx.lf(0); cx.set_f(p, t); }
        }
    } else {
        { let p = cx.lpp(10).add(16); let t = 1045220557; cx.set_i(p, t); }
    }
}
