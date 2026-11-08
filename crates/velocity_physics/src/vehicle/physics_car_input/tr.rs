//! Translated from the decompilation by tools/phys_c2rs4.py (do not edit by hand; fixes go in the
//! translator or in the hand-written overrides).
#![allow(non_snake_case, unused_parens, unused_labels, unused_braces, unused_variables, unused_mut, unused_assignments, unreachable_code, dead_code, clippy::all)]

use crate::tr_prelude::*;

/// PhysicsCarInput__1B86C0 0x001B86C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b86c0(cx: &mut Cx, a1: f32, a2: f32, a3: f32, a4: f32) -> f64 {
    let mut v4 = 0i32;
    let mut v5 = 0i32;
    let mut v6 = 0i32;
    let mut v7 = 0i32;
    let mut v9 = 0.0f64;
    let mut v10 = 0.0f32;
    let mut v11 = 0.0f32;
    if ((a1 < -1.0f32) || (a1 > 1.0f32)) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            v4 = crate::ov_zero_i!(cx, "input >= -1.0f && input <= 1.0f", ".\\src\\Physics\\Vehicle\\PhysicsCarInput.cpp", 104);
            if (v4 == 3) {
            }
        } else {
        }
    }
    if ((a2 < 0.0f32) || (a2 >= 1.0f32)) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            v5 = crate::ov_zero_i!(cx, "deadZone >= 0.0f && deadZone < 1.0f", ".\\src\\Physics\\Vehicle\\PhysicsCarInput.cpp", 105);
            if (v5 == 3) {
            }
        } else {
        }
    }
    if ((a3 <= 0.0f32) || (a3 >= 1.0f32)) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            v6 = crate::ov_zero_i!(cx, "kneePoint > 0.0f && kneePoint < 1.0f", ".\\src\\Physics\\Vehicle\\PhysicsCarInput.cpp", 106);
            if (v6 == 3) {
            }
        } else {
        }
    }
    if ((a4 < 0.0f32) || (a4 > 1.0f32)) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            v7 = crate::ov_zero_i!(cx, "initialSlope >= 0.0f && initialSlope <= 1.0f", ".\\src\\Physics\\Vehicle\\PhysicsCarInput.cpp", 107);
            if (v7 == 3) {
            }
        } else {
        }
    }
    v11 = (vsmath::fabs(a1) as f32);
    if (a2 >= v11) {
        return 0.0f64;
    }
    if (a3 >= v11) {
        return ((f_1b8940(cx, a1) * ((v11 as f64) - (a2 as f64))) * (a4 as f64));
    }
    if (v11 >= 0.99f32) {
        return (a1 as f64);
    }
    v9 = f_1b8940(cx, a1);
    v10 = (v9 as f32);
    let _ = f_452c0(cx, ((((a3 - a2) as f64) * (a4 as f64)) as f32), 1.0f32, ((((v11 - a3) as f64) / ((1.0f32 - a3) as f64)) as f32));
    return (v9 * (v10 as f64));
}

/// PhysicsCarInput__1B8940 0x001B8940
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b8940(cx: &mut Cx, a1: f32) -> f64 {
    if (a1 < 0.0f32) {
        return -1.0f64;
    } else {
        return 1.0f64;
    }
    #[allow(unreachable_code)]
    0.0
}

/// PhysicsCarInput__1B8980 0x001B8980
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b8980(cx: &mut Cx, a1: Ptr, a2: f32, a3: i32) -> f32 {
    let mut v3 = 0.0f64;
    let mut v6 = 0.0f32;
    let mut a3 = f32::from_bits(a3 as u32);
    v3 = (vsmath::clamp(((((a3 - cx.f(a1.add(12))) as f64) / ((cx.f(a1.add(16)) - cx.f(a1.add(12))) as f64)) as f32), 0.0f32, 1.0f32) as f64);
    v6 = (v3 as f32);
    { let q0_ = cx.f(a1.add(20)); let q1_ = cx.f(a1.add(24)); let q2_ = v6; f_452c0(cx, q0_, q1_, q2_) };
    return ((v3 * (a2 as f64)) as f32);
}

/// PhysicsCarInput__1B9630 0x001B9630
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b9630(cx: &mut Cx, mut this: Ptr) -> Ptr {
    let mut v2 = Ptr::NULL;
    let mut v3 = Ptr::NULL;
    let mut v4 = 0i32;
    v4 = 0;
    { let p = this; let t = Ptr::glob(0x64A114); cx.set_ptr(p, t); }
    let _ = f_1b7d50(cx, this.add(156));
    let _ = f_35bcd0(cx, this.add(240));
    let _ = f_1b6c30(cx, this.add(3560));
    ov_vector_ctor(cx, this.add(4944), 16, 4, Ptr::raw(Ptr::func(0).bits()));
    crate::ov_ignore!(cx, this.add(5100));
    { let p = this.add(3552); let t = 0.0f32; cx.set_f(p, t); }
    { let p = this.add(3556); let t = 0.0f32; cx.set_f(p, t); }
    { let p = this.add(4520); let t = 0.0f32; cx.set_f(p, t); }
    { let p = this.add(4528); let t = 0.0f32; cx.set_f(p, t); }
    crate::ov_ignore!(cx, 0 /*str*/);
    'lp2: while (v4 < 3) {
        if (cx.b(Ptr::glob(0x6F7158)) != 0) {
            v3 = ov_new(cx, 80);
            if (v3.bits() != Ptr::NULL.bits()) {
                v2 = f_1c8860(cx, v3);
            } else {
                v2 = Ptr::NULL;
            }
            { let p = this.add(4 * (v4)).add(5108); let t = v2; cx.set_ptr(p, t); }
        } else {
            { let p = this.add(4 * (v4)).add(5108); let t = 0.0f32; cx.set_f(p, t); }
        }
        { v4 += 1; v4 };
    }
    crate::ov_ignore!(cx);
    { let p = this.add(5120); let t = 0.0f32; cx.set_f(p, t); }
    let _ = f_1b9940(cx, this);
    return this;
}

/// PhysicsCarInput__1B9900 0x001B9900
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b9900(cx: &mut Cx, this: Ptr) -> i32 {
    return ov_memset(cx, this.add(5040), 0, 32).bits();
}

/// PhysicsCarInput__1B9940 0x001B9940
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b9940(cx: &mut Cx, this: Ptr) -> Ptr {
    let mut i = 0i32;
    let mut j = 0i32;
    let mut v2 = Ptr::NULL;
    { let p = this.add(4852); let t = 0; cx.set_i(p, t); }
    { let p = this.add(88); let t = 0; cx.set_b(p, t); }
    { let p = this.add(89); let t = 0; cx.set_b(p, t); }
    { let p = this.add(36); let t = 0; cx.set_i(p, t); }
    { let p = this.add(44); let t = 0; cx.set_i(p, t); }
    { let p = this.add(40); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4096); let t = 0; cx.set_i(p, t); }
    { let p = this.add(68); let t = 0; cx.set_i(p, t); }
    { let p = this.add(64); let t = 0; cx.set_i(p, t); }
    { let p = this.add(76); let t = 0; cx.set_i(p, t); }
    { let p = this.add(72); let t = 0; cx.set_i(p, t); }
    { let p = this.add(80); let t = 0; cx.set_b(p, t); }
    { let p = this.add(84); let t = -1; cx.set_i(p, t); }
    { let p = this.add(5072); let t = -1; cx.set_i(p, t); }
    { let p = this.add(5076); let t = -1; cx.set_i(p, t); }
    { let p = this.add(5080); let t = -1; cx.set_i(p, t); }
    { let p = this.add(5084); let t = 6; cx.set_i(p, t); }
    { let p = this.add(4932); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4936); let t = 0; cx.set_i(p, t); }
    { let p = this.add(28); let t = 0; cx.set_b(p, t); }
    { let p = this.add(92); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4940); let t = 0; cx.set_b(p, t); }
    let _ = ov_memset(cx, this.add(5024), 0, 16);
    let _ = f_1b9900(cx, this);
    { let p = this.add(5088); let t = 0; cx.set_i(p, t); }
    { let p = this.add(5092); let t = 0; cx.set_i(p, t); }
    { let p = this.add(5096); let t = 0; cx.set_i(p, t); }
    crate::ov_set_minus_one!(cx, this.add(5100));
    { let p = this.add(5104); let t = 0; cx.set_i(p, t); }
    { let p = this.add(5120); let t = 0; cx.set_i(p, t); }
    { let p = this.add(20); let t = 0; cx.set_i(p, t); }
    v2 = cx.ptr_at(this.add(3556));
    if (v2.bits() != Ptr::NULL.bits()) {
        let _ = f_1da110(cx, v2, 1);
    }
    { let p = this.add(3556); let t = 0; cx.set_i(p, t); }
    { let p = this.add(244); let t = this; cx.set_ptr(p, t); }
    let _ = f_1ba220(cx, this.add(3936));
    let _ = f_1ba1b0(cx, this.add(3952));
    let _ = f_1ba1b0(cx, this.add(3964));
    let _ = f_1ba1b0(cx, this.add(3976));
    let _ = f_1ba1b0(cx, this.add(3988));
    let _ = f_1ba1f0(cx, this.add(4000));
    let _ = f_1ba1f0(cx, this.add(4008));
    let _ = f_1ba1f0(cx, this.add(4016));
    let _ = f_1ba1f0(cx, this.add(4024));
    let _ = f_1ba170(cx, this.add(4032), 2139095039);
    { let p = this.add(4048); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4052); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4056); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4060); let t = 1065353216; cx.set_i(p, t); }
    { let p = this.add(4068); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4072); let t = 0; cx.set_b(p, t); }
    { let p = this.add(4856); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4524); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4532); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4592); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4716); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4736); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4756); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4872); let t = 0; cx.set_b(p, t); }
    { let p = this.add(4873); let t = 0; cx.set_b(p, t); }
    { let p = this.add(4874); let t = 0; cx.set_b(p, t); }
    { let p = this.add(4875); let t = 0; cx.set_b(p, t); }
    { let p = this.add(4536); let t = 0; cx.set_b(p, t); }
    { let p = this.add(4540); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4860); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4864); let t = 0; cx.set_i(p, t); }
    let _ = f_1ba1b0(cx, this.add(4396));
    { let p = this.add(4408); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4436); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4440); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4444); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4448); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4452); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4456); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4460); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4464); let t = 0; cx.set_i(p, t); }
    let _ = f_1ba1b0(cx, this.add(4468));
    { let p = this.add(4480); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4484); let t = 1065353216; cx.set_i(p, t); }
    { let p = this.add(4488); let t = 1065353216; cx.set_i(p, t); }
    { let p = this.add(4492); let t = 1065353216; cx.set_i(p, t); }
    { let p = this.add(4496); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4500); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4504); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4508); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4512); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4516); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4876); let t = 0; cx.set_i(p, t); }
    if (cx.b(Ptr::glob(0x6F7158)) != 0) {
        i = 0;
        'lp3: loop {
            if !(i < 3) { break 'lp3; }
            'blp3: {
                if (cx.i(this.add((4_i32).wrapping_mul(i)).add(5108)) != 0) {
                    { let q0_ = cx.ptr_at(this.add((4_i32).wrapping_mul(i)).add(5108)); f_1c8900(cx, q0_) };
                }
            }
            { i += 1; i };
        }
    }
    { let p = this.add(112); let t = 0; cx.set_i(p, t); }
    { let p = this.add(116); let t = 0; cx.set_i(p, t); }
    { let p = this.add(120); let t = 0; cx.set_i(p, t); }
    { let p = this.add(124); let t = 1056964608; cx.set_i(p, t); }
    { let p = this.add(128); let t = 1056964608; cx.set_i(p, t); }
    let _ = f_1ba1b0(cx, this.add(4544));
    { let p = this.add(4556); let t = 1065353216; cx.set_i(p, t); }
    let _ = f_1ba1b0(cx, this.add(4560));
    { let p = this.add(4572); let t = 1065353216; cx.set_i(p, t); }
    { let p = this.add(24); let t = 0; cx.set_b(p, t); }
    { let p = this.add(25); let t = 0; cx.set_b(p, t); }
    { let p = this.add(26); let t = 0; cx.set_b(p, t); }
    { let p = this.add(27); let t = 0; cx.set_b(p, t); }
    { let p = this.add(140); let t = 0; cx.set_i(p, t); }
    { let p = this.add(144); let t = 0; cx.set_b(p, t); }
    { let p = this.add(148); let t = 0; cx.set_i(p, t); }
    { let p = this.add(132); let t = 0; cx.set_b(p, t); }
    { let p = this.add(136); let t = cx.i(Ptr::glob(0x688A18)); cx.set_i(p, t); }
    { let p = this.add(4880); let t = 1065353216; cx.set_i(p, t); }
    j = 0;
    'lp4: loop {
        if !(j < 4) { break 'lp4; }
        'blp4: {
            { let p = this.add((4_i32).wrapping_mul(j)).add(4884); let t = 0; cx.set_i(p, t); }
        }
        { j += 1; j };
    }
    { let p = this.add(4900); let t = 0; cx.set_b(p, t); }
    { let p = this.add(4904); let t = 1065353216; cx.set_i(p, t); }
    { let p = this.add(4908); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4912); let t = 1065353216; cx.set_i(p, t); }
    { let p = this.add(4916); let t = 0; cx.set_i(p, t); }
    { let p = this.add(152); let t = 0; cx.set_b(p, t); }
    { let p = this.add(5008); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4920); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4924); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4928); let t = 0; cx.set_b(p, t); }
    { let p = this.add(5124); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4576); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4580); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4584); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4588); let t = 0; cx.set_i(p, t); }
    { let p = this.add(3844); let t = 0; cx.set_b(p, t); }
    { let p = this.add(3916); let t = 0; cx.set_b(p, t); }
    { let p = this.add(3876); let t = 1050253722; cx.set_i(p, t); }
    { let p = this.add(52); let t = 0; cx.set_i(p, t); }
    { let p = this.add(32); let t = 0; cx.set_i(p, t); }
    { let p = this.add(16); let t = 0; cx.set_b(p, t); }
    return this;
}

/// PhysicsCarInput__1BA170 0x001BA170
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1ba170(cx: &mut Cx, mut a1: Ptr, a2: i32) -> Ptr {
    { let p = a1; let t = a2; cx.set_i(p, t); }
    { let p = a1.add(4); let t = a2; cx.set_i(p, t); }
    { let p = a1.add(8); let t = a2; cx.set_i(p, t); }
    { let p = a1.add(12); let t = a2; cx.set_i(p, t); }
    return a1;
}

/// PhysicsCarInput__1BA1B0 0x001BA1B0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1ba1b0(cx: &mut Cx, mut a1: Ptr) -> Ptr {
    { let p = a1; let t = 0; cx.set_i(p, t); }
    { let p = a1.add(4); let t = 0; cx.set_i(p, t); }
    { let p = a1.add(8); let t = 0; cx.set_i(p, t); }
    return a1;
}

/// PhysicsCarInput__1BA1F0 0x001BA1F0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1ba1f0(cx: &mut Cx, mut a1: Ptr) -> Ptr {
    { let p = a1; let t = 0; cx.set_i(p, t); }
    { let p = a1.add(4); let t = 0; cx.set_i(p, t); }
    return a1;
}

/// PhysicsCarInput__1BA220 0x001BA220
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1ba220(cx: &mut Cx, mut a1: Ptr) -> Ptr {
    { let p = a1; let t = 0; cx.set_i(p, t); }
    { let p = a1.add(4); let t = 0; cx.set_i(p, t); }
    { let p = a1.add(8); let t = 0; cx.set_i(p, t); }
    { let p = a1.add(12); let t = 0; cx.set_i(p, t); }
    return a1;
}

/// PhysicsCarInput__1BA310 0x001BA310
pub fn f_1ba310(cx: &mut Cx, a2: Ptr, a3: Ptr, a4: Ptr, mut a5: Ptr) -> i64 {
    cx.push_frame(84);
    let r = f_1ba310_body(cx, a2, a3, a4, a5);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1ba310_body(cx: &mut Cx, a2: Ptr, a3: Ptr, a4: Ptr, mut a5: Ptr) -> i64 {
    let mut v4 = 0i64;
    let mut v6 = 0i64;
    { let q0_ = cx.lptr(64); let q1_ = a3; ov_vec4_copy(cx, q0_, q1_) };
    { let q0_ = cx.lptr(64); let q1_ = a2; ov_vec4_subeq(cx, q0_, q1_) };
    { let q0_ = cx.lptr(40); let q1_ = a4; ov_vec4_copy(cx, q0_, q1_) };
    { let q0_ = cx.lptr(40); let q1_ = a2; ov_vec4_subeq(cx, q0_, q1_) };
    { let t = ({ let q0_ = cx.lptr(40); let q1_ = cx.lptr(64); ov_vec_dot3(cx, q0_, q1_) } as f32); cx.set_lf(32, t); }
    { let t = ({ let q0_ = cx.lptr(64); let q1_ = cx.lptr(64); ov_vec_dot3(cx, q0_, q1_) } as f32); cx.set_lf(28, t); }
    { let t = (((cx.lf(32) as f64) / (cx.lf(28) as f64)) as f32); cx.set_lf(24, t); }
    if (((cx.lf(32) / cx.lf(28)) <= 1.0f32) && (cx.lf(24) >= 0.0f32)) {
        { let q0_ = cx.lptr(4); let q1_ = cx.lptr(64); ov_vec4_copy(cx, q0_, q1_) };
        { let q0_ = cx.lptr(4); let q1_ = cx.lf(24); ov_vec4_muleq(cx, q0_, q1_) };
        { let q0_ = cx.lptr(4); let q1_ = a2; ov_vec4_addeq(cx, q0_, q1_) };
        { let p = a5; let t = ({ let q0_ = a4; let q1_ = cx.lptr(4); ov_vec_dist_sq(cx, q0_, q1_) } as f32); cx.set_f(p, t); }
        v4 = (v4 & !(0xFFi64 as i64)) | ((((1) as i64) & 0xFF) << 0) as i64;
    } else {
        v4 = (v4 & !(0xFFi64 as i64)) | ((((0) as i64) & 0xFF) << 0) as i64;
    }
    v6 = v4;
    return v6;
}

/// PhysicsCarInput__1BA770 0x001BA770
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1ba770(cx: &mut Cx, this: Ptr, a2: f32, a3: f32) -> i32 {
    let mut v4 = 0.0f64;
    let mut v5 = 0.0f32;
    let mut result = Ptr::NULL;
    let mut _1B6F10 = 0.0f32;
    result = this;
    if (cx.b(this.add(144)) != 0) {
        _1B6F10 = ((f_1b6f10(cx, this) as i32) as f32);
        v5 = ({ let q0_ = (((_1B6F10 as f64) / (cx.f(Ptr::glob(0x688A88).add(4 * (4))) as f64)) as f32); let q1_ = 1.0f32; ov_min(cx, q0_, q1_) } as f32);
        if (((_1B6F10 <= cx.f(Ptr::glob(0x688A90))) || (a3 <= cx.f(Ptr::glob(0x688A88)))) || (v5 < cx.f(this.add(148)))) {
            { let p = this.add(148); let t = (((cx.f(this.add(148)) as f64) - ((cx.f(Ptr::glob(0x688A84)) * a2) as f64)) as f32); cx.set_f(p, t); }
            { let p = this.add(140); let t = 1; cx.set_i(p, t); }
        } else {
            { let p = this.add(148); let t = (((((cx.f(Ptr::glob(0x688A80)) * a2) * v5) as f64) + (cx.f(this.add(148)) as f64)) as f32); cx.set_f(p, t); }
            { let p = this.add(140); let t = 0; cx.set_i(p, t); }
        }
        v4 = (vsmath::clamp(cx.f(this.add(148)), 0.0f32, v5) as f64);
        { let p = this.add(148); let t = (v4 as f32); cx.set_f(p, t); }
        return this.bits();
    }
    return result.bits();
}

/// PhysicsCarInput__1BA8A0 0x001BA8A0
pub fn f_1ba8a0(cx: &mut Cx, a1: Ptr) -> i64 {
    cx.push_frame(236);
    let r = f_1ba8a0_body(cx, a1);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1ba8a0_body(cx: &mut Cx, a1: Ptr) -> i64 {
    let mut v1 = Ptr::NULL;
    let mut v2 = 0.0f64;
    let mut v3 = 0i64;
    let mut v4 = Ptr::NULL;
    let mut v5 = 0i32;
    let mut v6 = Ptr::NULL;
    let mut v7 = Ptr::NULL;
    let mut v8 = Ptr::NULL;
    let mut v9 = Ptr::NULL;
    let mut v10 = Ptr::NULL;
    let mut v11 = Ptr::NULL;
    let mut v12 = Ptr::NULL;
    let mut v13 = Ptr::NULL;
    let mut v14 = Ptr::NULL;
    let mut v15 = Ptr::NULL;
    let mut v16 = Ptr::NULL;
    let mut v17 = Ptr::NULL;
    let mut v18 = 0.0f64;
    let mut v19 = Ptr::NULL;
    let mut v20 = Ptr::NULL;
    let mut v21 = Ptr::NULL;
    let mut v22 = Ptr::NULL;
    let mut v23 = 0.0f64;
    let mut v24 = 0.0f64;
    let mut v25 = Ptr::NULL;
    let mut v26 = Ptr::NULL;
    let mut v28 = Ptr::NULL;
    let mut v29 = 0i64;
    let mut v30 = Ptr::NULL;
    let mut v31 = Ptr::NULL;
    let mut v32 = Ptr::NULL;
    { let t = a1; cx.set_lpp(58, t); }
    { let q0_ = cx.lptr(212); ov_identity_p(cx, q0_) };
    { let q0_ = cx.lptr(188); ov_identity_p(cx, q0_) };
    { let q0_ = cx.lptr(164); ov_identity_p(cx, q0_) };
    { let t = 0.0f32; cx.set_lf(152, t); }
    cx.memset(cx.lptr(136), 0, 12);
    { let p = cx.lpp(58).add(84); let t = -1; cx.set_i(p, t); }
    { let p = cx.lpp(58).add(76); let t = 0; cx.set_i(p, t); }
    if (cx.i(cx.lpp(58).add(72)) != 0) {
        v3 = (v3 & !0xFFFF_FFFFi64) | ((cx.i(cx.lpp(58).add(72)).wrapping_sub(1)) as i32 as u32 as i64);
        { let p = cx.lpp(58).add(72); let t = (v3 as i32); cx.set_i(p, t); }
        v3 = (v3 & 0xFFFF_FFFFi64) | (((cx.lpp(58).bits()) as u32 as i64) << 32);
        { let p = cx.lpp(58).add(68); let t = 0; cx.set_i(p, t); }
    } else {
        v1 = { let q0_ = cx.lpp(58); f_1c6480(cx, q0_) };
        v2 = ov_vec_length_sq(cx, v1);
        v3 = (v3 & !0xFFFF_FFFFi64) | ((Ptr::gc(0).bits()) as i32 as u32 as i64);
        if (v2 > ((cx.f(Ptr::gc(0).add(168)) as f64) * (cx.f(Ptr::gc(0).add(168)) as f64))) {
            v4 = ov_physics_manager(cx);
            { let t = ov_vehicle_array(cx, v4); cx.set_lpp(33, t); }
            { let t = 0.0f32; cx.set_lf(128, t); }
            'lp2: loop {
                'blp2: {
                    v5 = { let q0_ = cx.lpp(33).bits(); ov_vehicle_count(cx, q0_) };
                    if (cx.li(128) >= v5) {
                        break 'lp2;
                    }
                    if ((({ let p = { let p = { let q0_ = cx.lpp(33); let q1_ = cx.li(128); ov_vehicle_slot(cx, q0_, q1_) }; cx.ptr_at(p) }.add(5084); cx.i(p) } != 2) && ({ let p = { let p = { let q0_ = cx.lpp(33); let q1_ = cx.li(128); ov_vehicle_slot(cx, q0_, q1_) }; cx.ptr_at(p) }.add(5084); cx.i(p) } != 3)) && ({ let p = { let p = { let q0_ = cx.lpp(33); let q1_ = cx.li(128); ov_vehicle_slot(cx, q0_, q1_) }; cx.ptr_at(p) }.add(5084); cx.i(p) } != 6)) {
                        v6 = { let q0_ = cx.lpp(33); let q1_ = cx.li(128); ov_vehicle_slot(cx, q0_, q1_) };
                        v7 = { let q0_ = cx.ptr_at(v6); f_1c6480(cx, q0_) };
                        if (ov_vec_length_sq(cx, v7) > ((cx.f(Ptr::gc(0).add(168)) as f64) * (cx.f(Ptr::gc(0).add(168)) as f64))) {
                            v8 = { let q0_ = cx.lpp(33); let q1_ = cx.li(128); ov_vehicle_slot(cx, q0_, q1_) };
                            if (cx.i(v8) != cx.lpp(58).bits()) {
                                v9 = { let q0_ = cx.lpp(33); let q1_ = cx.li(128); ov_vehicle_slot(cx, q0_, q1_) };
                                v10 = { let q0_ = cx.ptr_at(v9); let q1_ = cx.lptr(96); ov_ve_get_position(cx, q0_, q1_) };
                                { let q0_ = cx.lptr(212); let q1_ = v10; ov_vec4_copy(cx, q0_, q1_) };
                                v11 = { let q0_ = cx.lpp(58); let q1_ = cx.lptr(80); ov_ve_get_position(cx, q0_, q1_) };
                                { let q0_ = cx.lptr(212); let q1_ = v11; ov_vec4_subeq(cx, q0_, q1_) };
                                { let t = ({ let q0_ = cx.lptr(212); ov_vec_length(cx, q0_) } as f32); cx.set_lf(136, t); }
                                if (cx.f(cx.ptr_at(cx.lpp(58).add(3552)).add(8)) > cx.lf(136)) {
                                    { let q0_ = cx.lptr(188); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                    v12 = { let q0_ = cx.lpp(33); let q1_ = cx.li(128); ov_vehicle_slot(cx, q0_, q1_) };
                                    v13 = { let q0_ = cx.ptr_at(v12); f_1c66c0(cx, q0_) };
                                    { let q0_ = cx.lptr(188); let q1_ = v13; ov_vec4_subeq(cx, q0_, q1_) };
                                    { let q0_ = cx.lptr(188); let q1_ = cx.f(cx.ptr_at(cx.lpp(58).add(3552)).add(8)); ov_vec4_muleq(cx, q0_, q1_) };
                                    v14 = { let q0_ = cx.lpp(33); let q1_ = cx.li(128); ov_vehicle_slot(cx, q0_, q1_) };
                                    v15 = { let q0_ = cx.ptr_at(v14); let q1_ = cx.lptr(64); ov_ve_get_position(cx, q0_, q1_) };
                                    { let q0_ = cx.lptr(188); let q1_ = v15; ov_vec4_addeq(cx, q0_, q1_) };
                                    v28 = { let q0_ = cx.lpp(58); let q1_ = cx.lptr(48); ov_ve_get_position(cx, q0_, q1_) };
                                    v16 = { let q0_ = cx.lpp(33); let q1_ = cx.li(128); ov_vehicle_slot(cx, q0_, q1_) };
                                    v17 = { let q0_ = cx.ptr_at(v16); let q1_ = cx.lptr(32); ov_ve_get_position(cx, q0_, q1_) };
                                    if ((({ let q0_ = v17; let q1_ = cx.lptr(188); let q2_ = v28; let q3_ = cx.lptr(152); f_1ba310(cx, q0_, q1_, q2_, q3_) } as i32) as u8 as i32) != 0) {
                                        { let t = ((1.0f64 - ((cx.lf(136) / cx.f(cx.ptr_at(cx.lpp(58).add(3552)).add(8))) as f64)) as f32); cx.set_lf(124, t); }
                                        { let t = (((((1.0f32 - cx.lf(124)) * cx.f(Ptr::gc(0).add(176))) as f64) + ((cx.lf(124) * cx.f(Ptr::glob(0x688A64))) as f64)) as f32); cx.set_lf(120, t); }
                                        { let t = ((((cx.lf(120) * 0.5f32) as f64) * ((cx.lf(120) * 0.5f32) as f64)) as f32); cx.set_lf(116, t); }
                                        if (cx.lf(116) > cx.lf(152)) {
                                            v18 = vsmath::fabs(cx.lf(152));
                                            { let t = ((1.0f64 - (v18 / (cx.lf(116) as f64))) as f32); cx.set_lf(112, t); }
                                            { let t = (((cx.lf(112) as f64) * (cx.f(Ptr::glob(0x688A60)) as f64)) as f32); cx.set_lf(112, t); }
                                            { let t = ((((1.0f32 - cx.f(Ptr::glob(0x688A60))) as f64) * (cx.lf(124) as f64)) as f32); cx.set_lf(124, t); }
                                            { let t = (vsmath::pow(cx.lf(112), cx.f(Ptr::glob(0x688A58))) as f32); cx.set_lf(112, t); }
                                            { let t = (vsmath::pow(cx.lf(124), cx.f(Ptr::glob(0x688A5C))) as f32); cx.set_lf(124, t); }
                                            { let t = (((cx.lf(124) as f64) + (cx.lf(112) as f64)) as f32); cx.set_lf(140, t); }
                                            if ((cx.lf(124) + cx.lf(112)) > cx.lf(144)) {
                                                { let p = cx.lpp(58).add(84); let t = cx.lf(128); cx.set_f(p, t); }
                                                { let t = cx.lf(140); cx.set_lf(144, t); }
                                                { let p = cx.lpp(58).add(64); let t = (((cx.lf(152) as f64) / (cx.lf(116) as f64)) as f32); cx.set_f(p, t); }
                                                v30 = { let q0_ = cx.lpp(58); let q1_ = cx.lptr(16); ov_ve_get_position(cx, q0_, q1_) };
                                                v19 = { let q0_ = cx.lpp(33); let q1_ = cx.li(128); ov_vehicle_slot(cx, q0_, q1_) };
                                                v20 = { let q0_ = cx.ptr_at(v19); let q1_ = cx.lptr(0); ov_ve_get_position(cx, q0_, q1_) };
                                                if ({ let q0_ = v20; let q1_ = cx.lptr(188); let q2_ = v30; ov_is_left_of_line(cx, q0_, q1_, q2_) } != 0) {
                                                    { let p = cx.lpp(58).add(64); let t = (((cx.f(cx.lpp(58).add(64)) as f64) * -1.0f64) as f32); cx.set_f(p, t); }
                                                }
                                                v21 = { let q0_ = cx.lpp(33); let q1_ = cx.li(128); ov_vehicle_slot(cx, q0_, q1_) };
                                                v31 = { let q0_ = cx.ptr_at(v21); f_1c66c0(cx, q0_) };
                                                v22 = { let q0_ = cx.lpp(58); f_1c66c0(cx, q0_) };
                                                v23 = ov_vec_dot3(cx, v22, v31);
                                                { let p = cx.lpp(58).add(76); let t = (v23 as f32); cx.set_f(p, t); }
                                                v24 = vsmath::acos(cx.f(cx.lpp(58).add(76)));
                                                { let p = cx.lpp(58).add(76); let t = (v24 as f32); cx.set_f(p, t); }
                                                v25 = { let q0_ = cx.lpp(33); let q1_ = cx.li(128); ov_vehicle_slot(cx, q0_, q1_) };
                                                v32 = { let q0_ = cx.ptr_at(v25); f_1c66c0(cx, q0_) };
                                                v26 = { let q0_ = cx.lpp(58); f_1c66c0(cx, q0_) };
                                                { let q0_ = cx.lptr(164); let q1_ = v26; let q2_ = v32; ov_ai_cross(cx, q0_, q1_, q2_) };
                                                if ({ let q0_ = cx.lptr(164); ov_vec4_y(cx, q0_) } < 0.0f32) {
                                                    { let p = cx.lpp(58).add(76); let t = (((cx.f(cx.lpp(58).add(76)) as f64) * -1.0f64) as f32); cx.set_f(p, t); }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                let _ = cx.preinc_i(cx.lptr(128));
            }
            if (((cx.i(cx.lpp(58).add(84)) != -1) && (cx.b(cx.lpp(58).add(4072)) != 0)) && (cx.i(Ptr::glob(0x749110)) != 2)) {
                { let q0_ = cx.i(Ptr::glob(0x730F30)); let q1_ = cx.i(cx.lpp(58).add(84)); let q2_ = cx.lf(144); ov_slipstream(cx, q0_, q1_, q2_) };
            }
            v3 = ({ let q0_ = cx.lpp(58); let q1_ = Ptr::gc(0); crate::ov_zero_i!(cx, q0_, q1_) } as i64);
            { let p = cx.lpp(58).add(68); let t = (((((1.0f32 - cx.f(Ptr::gc(0).add(180))) * cx.f(cx.lpp(58).add(68))) as f64) + ((cx.lf(144) * cx.f(Ptr::gc(0).add(180))) as f64)) as f32); cx.set_f(p, t); }
        }
    }
    v29 = v3;
    return v29;
}
