//! Translated from the decompilation by tools/phys_c2rs4.py (do not edit by hand; fixes go in the
//! translator or in the hand-written overrides).
#![allow(non_snake_case, unused_parens, unused_labels, unused_braces, unused_variables, unused_mut, unused_assignments, unreachable_code, dead_code, clippy::all)]

use crate::tr_prelude::*;

/// PhysicsObjectParticleTracker__1E3200 0x001E3200
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e3200(cx: &mut Cx, mut this: Ptr) -> Ptr {
    let _ = f_369890(cx, this);
    { let p = this; let t = Ptr::glob(0x64B774); cx.set_ptr(p, t); }
    { let p = this.add(532); let t = 0; cx.set_i(p, t); }
    { let p = this.add(536); let t = 0; cx.set_i(p, t); }
    { let p = this.add(540); let t = 0; cx.set_i(p, t); }
    { let p = this.add(580); let t = 0; cx.set_i(p, t); }
    let _ = f_23590(cx, this.add(680));
    let _ = ov_identity_p(cx, this.add(696));
    return this;
}

/// PhysicsObjectParticleTracker__1E3280 0x001E3280
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e3280(cx: &mut Cx, this: Ptr) -> i32 {
    { let p = this.add(29); let t = 0; cx.set_b(p, t); }
    { let p = this.add(32); let t = 0; cx.set_i(p, t); }
    { let p = this.add(36); let t = 0; cx.set_i(p, t); }
    return this.bits();
}

/// PhysicsObjectParticleTracker__1E32B0 0x001E32B0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e32b0(cx: &mut Cx, this: Ptr, a2: i32, a3: Ptr, a4: i32, a5: Ptr) -> i32 {
    let mut i = 0i32;
    let mut v5 = Ptr::NULL;
    let mut v7 = 0.0f32;
    let mut v8 = 0.0f32;
    let mut v9 = 0.0f32;
    let mut v10 = Ptr::NULL;
    let mut v11 = Ptr::NULL;
    let mut v12 = 0.0f32;
    { let p = this.add(20); let t = a2; cx.set_i(p, t); }
    let _ = ov_vec4_copy(cx, this, a3);
    let _ = ov_vec4_setw(cx, this, -915135488);
    { let p = this.add(16); let t = a4; cx.set_i(p, t); }
    if (cx.i(this.add(20)) != 0) {
        let _ = ov_vec4_setw(cx, this, 0);
        i = 0;
        'lp2: loop {
            if !(i < cx.i(cx.ptr_at(this.add(20)))) { break 'lp2; }
            'blp2: {
                v12 = { let q0_ = cx.ptr_at(cx.ptr_at(this.add(20)).add(16)).add((12_i32).wrapping_mul(i)); f_1e35d0(cx, q0_) };
                if ((v12 as f64) > (ov_vec4_w(cx, this) as f64)) {
                    let _ = ov_vec4_setw(cx, this, (v12.to_bits() as i32));
                }
            }
            { i += 1; i };
        }
        { let p = this.add(16); let t = 0; cx.set_i(p, t); }
    } else {
        { let p = this.add(16); let t = a4; cx.set_i(p, t); }
        { let q0_ = cx.ptr_at(this.add(16)); f_377bb0(cx, q0_) };
        { let q0_ = this; let q1_ = cx.i(cx.ptr_at(this.add(16)).add(12)); ov_vec4_setw(cx, q0_, q1_) };
        { let q0_ = cx.ptr_at(this.add(16)); f_1e3590(cx, q0_) };
        { let q0_ = cx.ptr_at(this.add(16)).add(16); let q1_ = 1065353216; let q2_ = 0; let q3_ = 0; f_1e3560(cx, q0_, q1_, q2_, q3_) };
        { let q0_ = cx.ptr_at(this.add(16)).add(28); let q1_ = 0; let q2_ = 1065353216; let q3_ = 0; f_1e3560(cx, q0_, q1_, q2_, q3_) };
        { let q0_ = cx.ptr_at(this.add(16)).add(40); let q1_ = 0; let q2_ = 0; let q3_ = 1065353216; f_1e3560(cx, q0_, q1_, q2_, q3_) };
        { let q0_ = cx.ptr_at(this.add(16)); f_3781b0(cx, q0_) };
    }
    if ({ let p = ov_physics_manager(cx).add(2840); cx.i(p) } == 0) {
        crate::ov_ignore!(cx, 0 /*str*/);
        crate::ov_ignore!(cx, "new C2AxisSortedList2");
        v11 = ov_new(cx, 64);
        if (v11.bits() != Ptr::NULL.bits()) {
            v10 = f_383fc0(cx, v11, 96, 10, 20, 3000);
        } else {
            v10 = Ptr::NULL;
        }
        { let p = ov_physics_manager(cx).add(2840); let t = v10; cx.set_ptr(p, t); }
        crate::ov_ignore!(cx);
    }
    if ((cx.gc.max_stastic_object_radius as f64) <= (ov_vec4_w(cx, this) as f64)) {
        if ((ov_vec4_w(cx, this) as f64) >= (cx.gc.max_stastic_object_radius as f64)) {
            let _ = ov_vec4_w(cx, this);
            crate::ov_ignore!(cx, 1, "!!immovable object ignored as having too too big bounding sphere:%g>%g\n");
        }
        { let p = a5.add(152); let t = cx.i(p).bitand_(!1); cx.set_i(p, t); }
        return a5.bits();
    } else {
        v9 = ov_vec4_w(cx, this);
        v8 = ov_vec4_z(cx, this);
        v7 = ov_vec4_x(cx, this);
        v5 = ov_physics_manager(cx);
        { let q0_ = cx.ptr_at(v5.add(2840)); let q1_ = a5; let q2_ = v7; let q3_ = v8; let q4_ = v9; let q5_ = 0; f_3847c0(cx, q0_, q1_, q2_, q3_, q4_, q5_) };
        { let p = a5.add(152); let t = cx.i(p).bitor_(1); cx.set_i(p, t); }
        return a5.bits();
    }
    #[allow(unreachable_code)]
    0
}

/// PhysicsObjectParticleTracker__1E3560 0x001E3560
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e3560(cx: &mut Cx, mut a1: Ptr, a2: i32, a3: i32, a4: i32) -> Ptr {
    { let p = a1; let t = a2; cx.set_i(p, t); }
    { let p = a1.add(4); let t = a3; cx.set_i(p, t); }
    { let p = a1.add(8); let t = a4; cx.set_i(p, t); }
    return a1;
}

/// PhysicsObjectParticleTracker__1E3590 0x001E3590
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e3590(cx: &mut Cx, mut a1: Ptr) -> Ptr {
    { let p = a1; let t = 0; cx.set_i(p, t); }
    { let p = a1.add(4); let t = 0; cx.set_i(p, t); }
    { let p = a1.add(8); let t = 0; cx.set_i(p, t); }
    return a1;
}

/// PhysicsObjectParticleTracker__1E35D0 0x001E35D0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e35d0(cx: &mut Cx, a1: Ptr) -> f32 {
    let mut v3 = 0.0f32;
    v3 = (f_1e3600(cx, a1, a1) as f32);
    return (vsmath::sqrt(v3) as f32);
}

/// PhysicsObjectParticleTracker__1E3600 0x001E3600
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e3600(cx: &mut Cx, a1: Ptr, a2: Ptr) -> f64 {
    return ((((cx.f(a1) as f64) * (cx.f(a2) as f64)) + ((cx.f(a1.add(4)) as f64) * (cx.f(a2.add(4)) as f64))) + ((cx.f(a1.add(8)) as f64) * (cx.f(a2.add(8)) as f64)));
}

/// PhysicsObjectParticleTracker__1E3630 0x001E3630
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e3630(cx: &mut Cx, this: Ptr) -> i32 {
    let _ = ov_identity_p(cx, this);
    let _ = f_13ea0(cx, this.add(64));
    { let p = this.add(100); let t = 0; cx.set_i(p, t); }
    { let p = this.add(104); let t = 0; cx.set_i(p, t); }
    { let p = this.add(136); let t = 0; cx.set_i(p, t); }
    { let p = this.add(144); let t = 0; cx.set_i(p, t); }
    { let p = this.add(148); let t = 0; cx.set_i(p, t); }
    { let p = this.add(152); let t = cx.i(p).bitand_(!1); cx.set_i(p, t); }
    { let p = this.add(152); let t = cx.i(p).bitor_(2); cx.set_i(p, t); }
    { let p = this.add(152); let t = cx.i(p).bitand_(!4); cx.set_i(p, t); }
    { let p = this.add(152); let t = cx.i(p).bitand_(!8); cx.set_i(p, t); }
    { let p = this.add(152); let t = cx.i(p).bitand_(!16); cx.set_i(p, t); }
    { let p = this.add(152); let t = cx.i(p).bitand_(!32); cx.set_i(p, t); }
    { let p = this.add(152); let t = cx.i(p).bitand_(!64); cx.set_i(p, t); }
    { let p = this.add(152); let t = cx.i(p).bitand_(!128); cx.set_i(p, t); }
    { let p = this.add(152); let t = cx.i(p).bitor_(3840); cx.set_i(p, t); }
    { let p = this.add(152); let t = cx.i(p).bitand_(!4096); cx.set_i(p, t); }
    { let p = this.add(156); let t = 0; cx.set_i(p, t); }
    let _ = f_1e5fc0(cx, this.add(64));
    return this.bits();
}

/// PhysicsObjectParticleTracker__1E37D0 0x001E37D0
pub fn f_1e37d0(cx: &mut Cx, this: Ptr) -> i64 {
    cx.push_frame(52);
    let r = f_1e37d0_body(cx, this);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1e37d0_body(cx: &mut Cx, this: Ptr) -> i64 {
    let mut v1 = Ptr::NULL;
    let mut v3 = Ptr::NULL;
    let mut v4 = 0.0f32;
    let mut v5 = 0i64;
    let mut v6 = 0.0f32;
    { let t = this; cx.set_lpp(12, t); }
    { let p = this.add(152); let t = cx.i(p).bitand_(!4); cx.set_i(p, t); }
    { let p = cx.lpp(12).add(152); let t = cx.i(p).bitand_(!64); cx.set_i(p, t); }
    { let p = cx.lpp(12).add(152); let t = cx.i(p).bitand_(!128); cx.set_i(p, t); }
    { let p = cx.lpp(12).add(152); let t = ((2_i32).wrapping_mul((((cx.u16(cx.ptr_at(cx.lpp(12).add(136)).add(74)) & 1) == 0) as i32)) | (cx.i(cx.lpp(12).add(152)) & -3)); cx.set_i(p, t); }
    if (cx.i(cx.lpp(12).add(100)) != 0) {
        { let q0_ = cx.lptr(28); let q1_ = { let q0_ = ((0.0f64 - (cx.f(cx.ptr_at(cx.lpp(12).add(136)).add(16)) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = { let q0_ = ((0.0f64 - (cx.f(cx.ptr_at(cx.lpp(12).add(136)).add(20)) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q3_ = { let q0_ = ((0.0f64 - (cx.f(cx.ptr_at(cx.lpp(12).add(136)).add(24)) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q4_ = cx.i(cx.ptr_at(cx.lpp(12).add(136)).add(28)); f_1da280(cx, q0_, q1_, q2_, q3_, q4_) };
        { let q0_ = cx.lpp(12); let q1_ = cx.lptr(28); f_2ac0c0(cx, q0_, q1_) };
        { let q0_ = cx.lpp(12); let q1_ = cx.ptr_at(cx.lpp(12).add(136)); f_14000(cx, q0_, q1_) };
        { let q0_ = cx.lpp(12); let q1_ = Ptr::NULL; let q2_ = Ptr::NULL; f_1e5270(cx, q0_, q1_, q2_) };
        { let p = cx.ptr_at(cx.lpp(12).add(100)).add(29); let t = 0; cx.set_b(p, t); }
        { let q0_ = cx.lpp(12); f_1e4ae0(cx, q0_) };
    }
    if (cx.i(cx.lpp(12).add(104)) != 0) {
        { let q0_ = cx.lptr(4); let q1_ = { let q0_ = ((0.0f64 - (cx.f(cx.ptr_at(cx.lpp(12).add(136)).add(16)) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = { let q0_ = ((0.0f64 - (cx.f(cx.ptr_at(cx.lpp(12).add(136)).add(20)) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q3_ = { let q0_ = ((0.0f64 - (cx.f(cx.ptr_at(cx.lpp(12).add(136)).add(24)) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q4_ = cx.i(cx.ptr_at(cx.lpp(12).add(136)).add(28)); f_1da280(cx, q0_, q1_, q2_, q3_, q4_) };
        { let q0_ = cx.lpp(12); let q1_ = cx.lptr(4); f_2ac0c0(cx, q0_, q1_) };
        { let q0_ = cx.lpp(12); let q1_ = cx.ptr_at(cx.lpp(12).add(136)); f_14000(cx, q0_, q1_) };
        if ((cx.i(cx.lpp(12).add(152)) & 1) != 0) {
            v6 = { let q0_ = cx.ptr_at(cx.lpp(12).add(136)); ov_vec4_z(cx, q0_) };
            v4 = { let q0_ = cx.ptr_at(cx.lpp(12).add(136)); ov_vec4_x(cx, q0_) };
            v3 = cx.lpp(12);
            v1 = ov_physics_manager(cx);
            { let q0_ = cx.ptr_at(v1.add(2840)); let q1_ = v3; let q2_ = v4; let q3_ = v6; f_384c70(cx, q0_, q1_, q2_, q3_) };
        }
    }
    { let p = cx.lpp(12).add(152); let t = cx.i(p).bitand_(!4096); cx.set_i(p, t); }
    v5 = { { let q0_ = cx.lpp(12); crate::ov_ignore!(cx, q0_) }; 0i64 };
    return v5;
}

/// PhysicsObjectParticleTracker__1E3CB0 0x001E3CB0
pub fn f_1e3cb0(cx: &mut Cx, this: Ptr, a3: Ptr) -> i32 {
    cx.push_frame(120);
    let r = f_1e3cb0_body(cx, this, a3);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1e3cb0_body(cx: &mut Cx, this: Ptr, a3: Ptr) -> i32 {
    let mut v2 = Ptr::NULL;
    let mut v3 = 0.0f64;
    let mut v4 = 0.0f64;
    let mut v5 = 0.0f64;
    let mut v7 = 0.0f32;
    let mut v8 = 0.0f32;
    let mut v9 = Ptr::NULL;
    let mut v10 = 0.0f32;
    { let t = this; cx.set_lpp(29, t); }
    v2 = this;
    if (cx.i(this.add(100)) != 0) {
        { let t = cx.ptr_at(a3.add(8)); cx.set_lpp(28, t); }
        { let t = cx.f(cx.ptr_at(cx.lpp(28))); cx.set_lf(108, t); }
        if (cx.li(108) == 1178816257) {
            { let t = cx.i(cx.lpp(28)).wrapping_add(16); cx.set_lpp(26, Ptr::raw(t)); }
            { let p = cx.ptr_at(cx.lpp(29).add(100)).add(32); let t = cx.lpp(26); cx.set_ptr(p, t); }
            { let p = cx.lpp(26).add(60); let t = cx.i(p).wrapping_add(cx.i(cx.lpp(28))); cx.set_i(p, t); }
            { let p = cx.lpp(26).add(64); let t = cx.i(p).wrapping_add(cx.i(cx.lpp(28))); cx.set_i(p, t); }
            { let p = cx.lpp(26).add(68); let t = cx.i(p).wrapping_add(cx.i(cx.lpp(28))); cx.set_i(p, t); }
            { let t = ((0.0f64 - (cx.f(a3.add(32)) as f64)) as f32); cx.set_lf(84, t); }
            { let t = ((0.0f64 - (cx.f(a3.add(36)) as f64)) as f32); cx.set_lf(88, t); }
            { let t = ((0.0f64 - (cx.f(a3.add(40)) as f64)) as f32); cx.set_lf(92, t); }
            { let t = cx.f(a3.add(44)); cx.set_lf(96, t); }
            { let q0_ = cx.lpp(26).add(16); let q1_ = cx.lptr(84); f_1e4460(cx, q0_, q1_) };
            { let t = 0.0f32; cx.set_lf(76, t); }
            'lp10: loop {
                if !(cx.li(76) < cx.b(cx.lpp(26).add(52))) { break 'lp10; }
                'blp10: {
                    { let t = 0.0f32; cx.set_lf(72, t); }
                    'lp11: loop {
                        if !(cx.li(72) < 3) { break 'lp11; }
                        'blp11: {
                            if (cx.gc.objects_min_extent > cx.f(cx.ptr_at(cx.lpp(26).add(60)).add((116_i32).wrapping_mul(cx.li(76))).add((4_i32).wrapping_mul(cx.li(72))).add(52))) {
                                { let p = cx.ptr_at(cx.lpp(26).add(60)).add((116_i32).wrapping_mul(cx.li(76))).add((4_i32).wrapping_mul(cx.li(72))).add(52); let t = cx.gc.objects_min_extent; cx.set_f(p, t); }
                            }
                        }
                        let _ = cx.preinc_i(cx.lptr(72));
                    }
                }
                let _ = cx.preinc_i(cx.lptr(76));
            }
            { let t = 0.0f32; cx.set_lf(68, t); }
            'lp12: loop {
                if !(cx.li(68) < cx.b(cx.lpp(26).add(53))) { break 'lp12; }
                'blp12: {
                    if (cx.gc.objects_min_extent > cx.f(cx.ptr_at(cx.lpp(26).add(64)).add((36_i32).wrapping_mul(cx.li(68))).add(16))) {
                        { let p = cx.ptr_at(cx.lpp(26).add(64)).add((36_i32).wrapping_mul(cx.li(68))).add(16); let t = cx.gc.objects_min_extent; cx.set_f(p, t); }
                    }
                }
                let _ = cx.preinc_i(cx.lptr(68));
            }
            { let t = 0.0f32; cx.set_lf(64, t); }
            'lp13: loop {
                if !(cx.li(64) < cx.b(cx.lpp(26).add(54))) { break 'lp13; }
                'blp13: {
                    if (({ let q0_ = cx.ptr_at(cx.lpp(26).add(68)).add((cx.li(64) << 6)).add(48); f_1e4440(cx, q0_) } < 0.80999994f32) || ({ let q0_ = cx.ptr_at(cx.lpp(26).add(68)).add((cx.li(64) << 6)).add(48); f_1e4440(cx, q0_) } > 1.21f32)) {
                        { let p = cx.ptr_at(cx.lpp(26).add(68)).add((cx.li(64) << 6)).add(48); let t = 0; cx.set_i(p, t); }
                        { let p = cx.ptr_at(cx.lpp(26).add(68)).add((cx.li(64) << 6)).add(52); let t = 1065353216; cx.set_i(p, t); }
                        { let p = cx.ptr_at(cx.lpp(26).add(68)).add((cx.li(64) << 6)).add(56); let t = 0; cx.set_i(p, t); }
                    }
                    if (cx.gc.objects_min_extent > cx.f(cx.ptr_at(cx.lpp(26).add(68)).add((cx.li(64) << 6)).add(28))) {
                        { let p = cx.ptr_at(cx.lpp(26).add(68)).add((cx.li(64) << 6)).add(28); let t = cx.gc.objects_min_extent; cx.set_f(p, t); }
                    }
                    if (cx.gc.objects_min_extent > cx.f(cx.ptr_at(cx.lpp(26).add(68)).add((cx.li(64) << 6)).add(32))) {
                        { let p = cx.ptr_at(cx.lpp(26).add(68)).add((cx.li(64) << 6)).add(32); let t = cx.gc.objects_min_extent; cx.set_f(p, t); }
                        { let p = cx.ptr_at(cx.lpp(26).add(68)).add((cx.li(64) << 6)).add(32); let t = cx.gc.objects_min_extent; cx.set_f(p, t); }
                    }
                }
                let _ = cx.preinc_i(cx.lptr(64));
            }
            { let p = cx.ptr_at(cx.lpp(29).add(100)).add(36); let t = 0; cx.set_i(p, t); }
            { let p = cx.ptr_at(cx.lpp(29).add(100)).add(29); let t = 0; cx.set_b(p, t); }
            { let q0_ = cx.lpp(26); let q1_ = cx.ptr_at(cx.lpp(29).add(100)).add(4); f_379720(cx, q0_, q1_) };
            { let t = 0.0f32; cx.set_lf(60, t); }
            'lp14: loop {
                if !(cx.li(60) < cx.b(cx.lpp(26).add(52))) { break 'lp14; }
                'blp14: {
                    { let q0_ = cx.ptr_at(cx.lpp(26).add(60)).add((116_i32).wrapping_mul(cx.li(60))).add(64); let q1_ = cx.ptr_at(cx.lpp(29).add(100)).add(4); f_1e43a0(cx, q0_, q1_) };
                }
                let _ = cx.preinc_i(cx.lptr(60));
            }
            { let t = 0.0f32; cx.set_lf(56, t); }
            'lp15: loop {
                if !(cx.li(56) < cx.b(cx.lpp(26).add(53))) { break 'lp15; }
                'blp15: {
                    { let q0_ = cx.ptr_at(cx.lpp(26).add(64)).add((36_i32).wrapping_mul(cx.li(56))).add(20); let q1_ = cx.ptr_at(cx.lpp(29).add(100)).add(4); f_1e43a0(cx, q0_, q1_) };
                }
                let _ = cx.preinc_i(cx.lptr(56));
            }
            { let t = 0.0f32; cx.set_lf(52, t); }
            'lp16: loop {
                if !(cx.li(52) < cx.b(cx.lpp(26).add(54))) { break 'lp16; }
                'blp16: {
                    { let q0_ = cx.ptr_at(cx.lpp(26).add(68)).add((cx.li(52) << 6)).add(36); let q1_ = cx.ptr_at(cx.lpp(29).add(100)).add(4); f_1e43a0(cx, q0_, q1_) };
                }
                let _ = cx.preinc_i(cx.lptr(52));
            }
            { let q0_ = cx.lpp(26); f_377bb0(cx, q0_) };
            { let q0_ = cx.lpp(26); f_3781b0(cx, q0_) };
            { let p = cx.ptr_at(cx.lpp(29).add(100)); let t = cx.i(a3.add(48)); cx.set_i(p, t); }
            { let q0_ = cx.lpp(26); let q1_ = cx.ptr_at(cx.lpp(29).add(100)).add(16); let q2_ = (((cx.f(a3.add(48)) as f64) * (cx.gc.objects_mass_multiplier as f64)) as f32); f_379660(cx, q0_, q1_, q2_) };
            { let p = cx.ptr_at(cx.lpp(29).add(100)).add(28); let t = 0; cx.set_b(p, t); }
            if (cx.gc.spherical_inertia_threshold != 0.0f32) {
                { let t = 3.4028235e38f32; cx.set_lf(48, t); }
                { let t = 0.0f32; cx.set_lf(44, t); }
                'lp17: loop {
                    if !(cx.li(44) < 3) { break 'lp17; }
                    'blp17: {
                        if (cx.lf(48) > cx.f(cx.ptr_at(cx.lpp(29).add(100)).add((4_i32).wrapping_mul(cx.li(44))).add(16))) {
                            { let t = cx.f(cx.ptr_at(cx.lpp(29).add(100)).add((4_i32).wrapping_mul(cx.li(44))).add(16)); cx.set_lf(48, t); }
                        }
                    }
                    let _ = cx.preinc_i(cx.lptr(44));
                }
                { let p = cx.ptr_at(cx.lpp(29).add(100)).add(28); let t = 1; cx.set_b(p, t); }
                { let t = 0.0f32; cx.set_lf(40, t); }
                'lp18: loop {
                    if !(cx.li(40) < 3) { break 'lp18; }
                    'blp18: {
                        if (cx.f(cx.ptr_at(cx.lpp(29).add(100)).add((4_i32).wrapping_mul(cx.li(40))).add(16)) > (cx.lf(48) * cx.gc.spherical_inertia_threshold)) {
                            { let p = cx.ptr_at(cx.lpp(29).add(100)).add(28); let t = 0; cx.set_b(p, t); }
                            break 'lp18;
                        }
                    }
                    let _ = cx.preinc_i(cx.lptr(40));
                }
            }
            { let q0_ = cx.lptr(24); let q1_ = cx.ptr_at(cx.lpp(29).add(100)).add(4); let q2_ = cx.lpp(26).add(16); f_1e4650(cx, q0_, q1_, q2_) };
            v3 = (ov_vec4_z(cx, a3.add(16)) as f64);
            v10 = ((v3 + (cx.lf(32) as f64)) as f32);
            v4 = (ov_vec4_y(cx, a3.add(16)) as f64);
            v8 = ((v4 + (cx.lf(28) as f64)) as f32);
            v5 = (ov_vec4_x(cx, a3.add(16)) as f64);
            v7 = ((v5 + (cx.lf(24) as f64)) as f32);
            { let q0_ = cx.lpp(26); let q1_ = (v7.to_bits() as i32); let q2_ = (v8.to_bits() as i32); let q3_ = (v10.to_bits() as i32); f_1e3560(cx, q0_, q1_, q2_, q3_) };
            { let q0_ = cx.lpp(26); f_3781b0(cx, q0_) };
            { let q0_ = cx.lptr(4); let q1_ = cx.lpp(26); let q2_ = cx.lptr(24); f_1e43f0(cx, q0_, q1_, q2_) };
            { let q0_ = cx.lpp(29); let q1_ = cx.lpp(26).add(16); f_1e4370(cx, q0_, q1_) };
            { let q0_ = cx.lpp(29).add(16); let q1_ = cx.lpp(26).add(28); f_1e4370(cx, q0_, q1_) };
            { let q0_ = cx.lpp(29).add(32); let q1_ = cx.lpp(26).add(40); f_1e4370(cx, q0_, q1_) };
            { let q0_ = cx.lpp(29).add(48); let q1_ = cx.lptr(4); f_1e4370(cx, q0_, q1_) };
            { let p = cx.lpp(29).add(44); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(29).add(28); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(29).add(12); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(29).add(60); let t = 1065353216; cx.set_i(p, t); }
            v2 = Ptr::raw(({ let q0_ = cx.lpp(29); let q1_ = Ptr::NULL; let q2_ = Ptr::NULL; f_1e5270(cx, q0_, q1_, q2_) } as i32));
        } else {
            v2 = Ptr::raw({ crate::ov_ignore!(cx, 3, "[CPhysicsObjectManager__BuildPhysicsObjectCollision] - Header types do not match!\n"); 0 });
        }
    }
    v9 = v2;
    return v9.bits();
}

/// PhysicsObjectParticleTracker__1E4370 0x001E4370
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e4370(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = cx.i(a2); cx.set_i(p, t); }
    { let p = a1.add(4); let t = cx.i(a2.add(4)); cx.set_i(p, t); }
    { let p = a1.add(8); let t = cx.i(a2.add(8)); cx.set_i(p, t); }
    return a1;
}

/// PhysicsObjectParticleTracker__1E43A0 0x001E43A0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e43a0(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = (((cx.f(a1) as f64) - (cx.f(a2) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a1.add(4)) as f64) - (cx.f(a2.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a1.add(8)) as f64) - (cx.f(a2.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// PhysicsObjectParticleTracker__1E43F0 0x001E43F0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e43f0(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = (((cx.f(a2) as f64) - (cx.f(a3) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a2.add(4)) as f64) - (cx.f(a3.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a2.add(8)) as f64) - (cx.f(a3.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// PhysicsObjectParticleTracker__1E4440 0x001E4440
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e4440(cx: &mut Cx, a1: Ptr) -> f32 {
    return (f_1e3600(cx, a1, a1) as f32);
}

/// PhysicsObjectParticleTracker__1E4460 0x001E4460
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e4460(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = ((((((cx.f(a2) * cx.f(a2)) + (cx.f(a2.add(12)) * cx.f(a2.add(12)))) * 2.0f32) as f64) - 1.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((((cx.f(a2) * cx.f(a2.add(4))) + (cx.f(a2.add(8)) * cx.f(a2.add(12)))) as f64) * 2.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((((cx.f(a2.add(8)) * cx.f(a2)) - (cx.f(a2.add(4)) * cx.f(a2.add(12)))) as f64) * 2.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(12); let t = (((((cx.f(a2) * cx.f(a2.add(4))) - (cx.f(a2.add(8)) * cx.f(a2.add(12)))) as f64) * 2.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(16); let t = ((((((cx.f(a2.add(4)) * cx.f(a2.add(4))) + (cx.f(a2.add(12)) * cx.f(a2.add(12)))) * 2.0f32) as f64) - 1.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(20); let t = (((((cx.f(a2.add(4)) * cx.f(a2.add(8))) + (cx.f(a2) * cx.f(a2.add(12)))) as f64) * 2.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(24); let t = (((((cx.f(a2.add(8)) * cx.f(a2)) + (cx.f(a2.add(4)) * cx.f(a2.add(12)))) as f64) * 2.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(28); let t = (((((cx.f(a2.add(4)) * cx.f(a2.add(8))) - (cx.f(a2) * cx.f(a2.add(12)))) as f64) * 2.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(32); let t = ((((((cx.f(a2.add(8)) * cx.f(a2.add(8))) + (cx.f(a2.add(12)) * cx.f(a2.add(12)))) * 2.0f32) as f64) - 1.0f64) as f32); cx.set_f(p, t); }
    return a2;
}

/// PhysicsObjectParticleTracker__1E4650 0x001E4650
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e4650(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = (((((cx.f(a3) * cx.f(a2)) + (cx.f(a3.add(12)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(24)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((((cx.f(a3.add(4)) * cx.f(a2)) + (cx.f(a3.add(16)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(28)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((((cx.f(a3.add(8)) * cx.f(a2)) + (cx.f(a3.add(20)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(32)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    return a3;
}

/// PhysicsObjectParticleTracker__1E4710 0x001E4710
pub fn f_1e4710(cx: &mut Cx, this: Ptr) -> i32 {
    cx.push_frame(112);
    let r = f_1e4710_body(cx, this);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1e4710_body(cx: &mut Cx, this: Ptr) -> i32 {
    let mut v1 = Ptr::NULL;
    let mut v2 = Ptr::NULL;
    let mut v4 = Ptr::NULL;
    let mut bb: u32 = 0;
    #[allow(unreachable_code)]
    'cfg: loop {
        match bb {
            0 => {
                { let t = this; cx.set_lpp(27, t); }
                v1 = this;
                if (cx.i(this.add(100)) != 0) { bb = 1; continue 'cfg; } else { bb = 2; continue 'cfg; }
                break 'cfg;
            }
            1 => {
                if ({ let q0_ = cx.ptr_at(cx.lpp(27).add(100)); f_1b41f0(cx, q0_) } != 0) { bb = 4; continue 'cfg; } else { bb = 5; continue 'cfg; }
                break 'cfg;
            }
            2 => {
                { bb = 3; continue 'cfg; }
                break 'cfg;
            }
            3 => {
                { bb = 8; continue 'cfg; }
                break 'cfg;
            }
            4 => {
                { bb = 7; continue 'cfg; }
                break 'cfg;
            }
            5 => {
                { bb = 6; continue 'cfg; }
                break 'cfg;
            }
            6 => {
                if ({ let p = ov_physics_manager(cx).add(24324); cx.i(p) } >= 0) { bb = 10; continue 'cfg; } else { bb = 11; continue 'cfg; }
                break 'cfg;
            }
            7 => {
                { let p = cx.ptr_at(cx.lpp(27).add(100)).add(29); let t = 1; cx.set_b(p, t); }
                v1 = cx.ptr_at(cx.lpp(27).add(100));
                { let p = Ptr::raw(cx.i(v1.add(36)).wrapping_add(532)); let t = 0; cx.set_i(p, t); }
                { bb = 8; continue 'cfg; }
                break 'cfg;
            }
            8 => {
                v4 = v1;
                return v4.bits();
            }
            9 => {
                { bb = 6; continue 'cfg; }
                break 'cfg;
            }
            10 => {
                { bb = 13; continue 'cfg; }
                break 'cfg;
            }
            11 => {
                { bb = 12; continue 'cfg; }
                break 'cfg;
            }
            12 => {
                crate::ov_ignore!(cx, 1, " Too many awake objects >=%d. Deactivating the oldest.\n");
                { let t = -1; cx.set_li(104, t); }
                { let t = 0; cx.set_lpp(25, Ptr::raw(t)); }
                { let t = 0; cx.set_li(96, t); }
                { bb = 19; continue 'cfg; }
                break 'cfg;
            }
            13 => {
                v2 = ov_physics_manager(cx);
                { let p = cx.ptr_at(cx.lpp(27).add(100)).add(36); let t = { let p = v2.add((4_i32).wrapping_mul({ let p = ov_physics_manager(cx).add(24324); cx.i(p) })).add(24204); cx.i(p) }; cx.set_i(p, t); }
                { let t = ov_physics_manager(cx).add(24324); cx.set_lpp(0, t); }
                let _ = cx.predec_i(cx.lpp(0));
                { let t = cx.ptr_at(cx.ptr_at(cx.lpp(27).add(100)).add(36)); cx.set_lpp(22, t); }
                { let p = cx.lpp(22).add(540); let t = cx.lpp(27); cx.set_ptr(p, t); }
                { let q0_ = cx.lpp(22); let q1_ = Ptr::gc(1848); let q2_ = cx.ptr_at(cx.lpp(27).add(100)).add(16); let q3_ = { let q0_ = (((cx.f(cx.ptr_at(cx.lpp(27).add(100))) as f64) * (cx.gc.objects_mass_multiplier as f64)) as f32); ov_coerce_int(cx, q0_) }; let q4_ = cx.b(cx.ptr_at(cx.lpp(27).add(100)).add(28)); f_369a80(cx, q0_, q1_, q2_, q3_, q4_) };
                { let p = cx.lpp(22).add(24); let t = cx.gc.objects_linear_damping; cx.set_f(p, t); }
                { let p = cx.lpp(22).add(28); let t = cx.gc.objects_angular_damping; cx.set_f(p, t); }
                { let q0_ = cx.lptr(72); let q1_ = cx.ptr_at(cx.lpp(27).add(100)).add(4); let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(27).add(100)).add(32)).add(16); f_1e4650(cx, q0_, q1_, q2_) };
                { let t = { let q0_ = cx.lpp(27); crate::ov_ident!(cx, q0_) }; cx.set_lpp(16, t); }
                { let q0_ = cx.lpp(22); let q1_ = { let q0_ = (((cx.f(cx.lpp(16).add(48)) as f64) + (cx.f(cx.lptr(72)) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = { let q0_ = (((cx.f(cx.lpp(16).add(52)) as f64) + (cx.f(cx.lptr(76)) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q3_ = { let q0_ = (((cx.f(cx.lpp(16).add(56)) as f64) + (cx.f(cx.lptr(80)) as f64)) as f32); ov_coerce_int(cx, q0_) }; f_36b130(cx, q0_, q1_, q2_, q3_) };
                { let q0_ = cx.lptr(24); let q1_ = cx.lpp(16); f_1e4370(cx, q0_, q1_) };
                { let q0_ = cx.lptr(36); let q1_ = cx.lpp(16).add(16); f_1e4370(cx, q0_, q1_) };
                { let q0_ = cx.lptr(48); let q1_ = cx.lpp(16).add(32); f_1e4370(cx, q0_, q1_) };
                { let q0_ = cx.lpp(22); let q1_ = cx.lptr(24); f_36a3e0(cx, q0_, q1_) };
                { let q0_ = cx.lpp(22); f_1e6000(cx, q0_) };
                { let _ = { let q0_ = cx.lpp(22); vcall(cx, 32, q0_, &[]) }; };
                { let t = 0; cx.set_li(16, t); }
                { bb = 14; continue 'cfg; }
                break 'cfg;
            }
            14 => {
                if (cx.li(16) < 6) { bb = 15; continue 'cfg; } else { bb = 17; continue 'cfg; }
                break 'cfg;
            }
            15 => {
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(27).add(100)).add(36)).add((4_i32).wrapping_mul(cx.li(16))).add(556); let t = 2139095039; cx.set_i(p, t); }
                { bb = 16; continue 'cfg; }
                break 'cfg;
            }
            16 => {
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(27).add(100)).add(36)).add((16_i32).wrapping_mul(cx.postinc_i(cx.lptr(16)))).add(596); let t = 2139095039; cx.set_i(p, t); }
                { bb = 14; continue 'cfg; }
                break 'cfg;
            }
            17 => {
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(27).add(100)).add(36)).add(536); let t = 0; cx.set_i(p, t); }
                { let q0_ = cx.lptr(8); f_1e6070(cx, q0_) };
                { let t = cx.lpp(27); cx.set_lpp(2, t); }
                { let q0_ = Ptr::glob(0x6FAA34); let q1_ = 4; let q2_ = cx.lptr(8); ov_message(cx, q0_, q1_, q2_) };
                { bb = 7; continue 'cfg; }
                break 'cfg;
            }
            18 => {
                { bb = 12; continue 'cfg; }
                break 'cfg;
            }
            19 => {
                if (cx.li(96) < { let p = { let p = ov_physics_manager(cx).add(2836); cx.ptr_at(p) }; cx.u16(p) }) { bb = 20; continue 'cfg; } else { bb = 22; continue 'cfg; }
                break 'cfg;
            }
            20 => {
                { let t = { let p = { let p = { let p = ov_physics_manager(cx).add(2836); cx.ptr_at(p) }.add(20); cx.ptr_at(p) }.add((16_i32).wrapping_mul(cx.li(96))); cx.ptr_at(p) }; cx.set_lpp(23, t); }
                if (((cx.i(cx.lpp(23).add(100)) != 0) && ({ let q0_ = cx.ptr_at(cx.lpp(23).add(100)); f_1b41f0(cx, q0_) } != 0)) && (cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(23).add(100)).add(36)).add(532)) > cx.li(104))) { bb = 23; continue 'cfg; } else { bb = 24; continue 'cfg; }
                break 'cfg;
            }
            21 => {
                let _ = cx.preinc_i(cx.lptr(96));
                { bb = 19; continue 'cfg; }
                break 'cfg;
            }
            22 => {
                if (cx.lpp(25).bits() != 0) { bb = 26; continue 'cfg; } else { bb = 27; continue 'cfg; }
                break 'cfg;
            }
            23 => {
                { let t = cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(23).add(100)).add(36)).add(532)); cx.set_li(104, t); }
                { let t = cx.lpp(23); cx.set_lpp(25, t); }
                { bb = 25; continue 'cfg; }
                break 'cfg;
            }
            24 => {
                { bb = 25; continue 'cfg; }
                break 'cfg;
            }
            25 => {
                { bb = 21; continue 'cfg; }
                break 'cfg;
            }
            26 => {
                { let q0_ = cx.lpp(25); f_1e4ae0(cx, q0_) };
                { let q0_ = cx.lpp(25); f_1e5670(cx, q0_) };
                { let p = cx.lpp(25).add(152); let t = cx.i(p).bitand_(!2); cx.set_i(p, t); }
                { bb = 13; continue 'cfg; }
                break 'cfg;
            }
            27 => {
                { bb = 28; continue 'cfg; }
                break 'cfg;
            }
            28 => {
                v1 = Ptr::raw({ crate::ov_ignore!(cx, 1, "!!weird problem: no oldest object\n"); 0 });
                { bb = 3; continue 'cfg; }
                break 'cfg;
            }
            29 => {
                { bb = 28; continue 'cfg; }
                break 'cfg;
            }
            30 => {
                break 'cfg;
            }
            31 => {
                break 'cfg;
            }
            _ => unreachable!(),
        }
    }
    #[allow(unreachable_code)]
    0
}

/// PhysicsObjectParticleTracker__1E4AE0 0x001E4AE0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e4ae0(cx: &mut Cx, this: Ptr) -> i32 {
    let mut i = 0i32;
    let mut v1 = Ptr::NULL;
    let mut v2 = Ptr::NULL;
    let mut v4 = Ptr::NULL;
    let mut v6 = Ptr::NULL;
    v1 = Ptr::raw((v1.bits() & !(0xFFu32 as i32)) | ((((this.bits() as u8 as i32)) & 0xFF) << 0));
    if (cx.ptr_at(this.add(100)).bits() != Ptr::NULL.bits()) {
        v1 = Ptr::raw((v1.bits() & !(0xFFu32 as i32)) | ((({ let q0_ = cx.ptr_at(this.add(100)); f_1b41f0(cx, q0_) }) & 0xFF) << 0));
        if ((v1.bits() as u8 as i32) != 0) {
            v6 = Ptr::raw(cx.i(cx.ptr_at(this.add(100)).add(36)));
            { let p = v6.add(532); let t = 0; cx.set_i(p, t); }
            { let p = v6.add(580); let t = 0; cx.set_i(p, t); }
            { let p = v6.add(536); let t = 0; cx.set_i(p, t); }
            let _ = f_1e58c0(cx, this, 1.0f32);
            let _ = f_36b530(cx, v6, 0, 0, 0);
            let _ = f_36b730(cx, v6, 0, 0, 0);
            i = 0;
            'lp2: loop {
                if !(i < 6) { break 'lp2; }
                'blp2: {
                    { let p = v6.add(4 * (i.wrapping_add(139))); let t = 2139095039; cx.set_i(p, t); }
                    { let p = v6.add(4 * ((4_i32).wrapping_mul(i).wrapping_add(149))); let t = 2139095039; cx.set_i(p, t); }
                }
                { i += 1; i };
            }
            v4 = ov_physics_manager(cx).add(24324);
            let _ = cx.preinc_i(v4);
            v2 = ov_physics_manager(cx);
            { let p = v2.add((4_i32).wrapping_mul({ let p = ov_physics_manager(cx).add(24324); cx.i(p) })).add(24204); let t = v6; cx.set_ptr(p, t); }
            { let p = v6.add(540); let t = 0; cx.set_i(p, t); }
            v1 = cx.ptr_at(this.add(100));
            { let p = v1.add(36); let t = 0; cx.set_i(p, t); }
        }
    }
    return v1.bits();
}

/// PhysicsObjectParticleTracker__1E4C30 0x001E4C30
pub fn f_1e4c30(cx: &mut Cx, this: Ptr, dt: f32) -> Ptr {
    cx.push_frame(96);
    let r = f_1e4c30_body(cx, this, dt);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1e4c30_body(cx: &mut Cx, this: Ptr, dt: f32) -> Ptr {
    let mut x = 0.0f32;
    let mut v2 = Ptr::NULL;
    let mut v3 = 0.0f64;
    let mut v4 = 0.0f64;
    let mut v5 = Ptr::NULL;
    let mut v6 = Ptr::NULL;
    let mut v8 = 0.0f64;
    let mut v9 = 0.0f64;
    let mut xa = 0.0f32;
    let mut v11 = Ptr::NULL;
    let mut v12 = 0.0f32;
    let mut v13 = Ptr::NULL;
    let mut max_angle_change_to_put_to_sleep = 0.0f64;
    { let t = this; cx.set_lpp(23, t); }
    v2 = this;
    if (cx.i(this.add(100)) != 0) {
        v2 = Ptr::raw((v2.bits() & !(0xFFu32 as i32)) | ((({ let q0_ = cx.ptr_at(cx.lpp(23).add(100)); f_1b41f0(cx, q0_) }) & 0xFF) << 0));
        if ((v2.bits() as u8 as i32) != 0) {
            { let t = cx.ptr_at(cx.ptr_at(cx.lpp(23).add(100)).add(36)); cx.set_lpp(22, t); }
            if ((cx.preinc_i(cx.lpp(22).add(532)) as f32) <= cx.gc.max_time_alive) {
                { let q0_ = cx.lpp(22); f_1e6000(cx, q0_) };
                { let q0_ = cx.lptr(68); let q1_ = cx.i(cx.lpp(22).add(64)); let q2_ = cx.i(cx.lpp(22).add(68)); let q3_ = cx.i(cx.lpp(22).add(72)); let q4_ = 0; ov_vec4_xyzw(cx, q0_, q1_, q2_, q3_, q4_) };
                { let _ = { let q0_ = cx.lpp(22); let q1_ = (dt.to_bits() as i32); let q2_ = 0; vcall(cx, 0, q0_, &[Varg::from(q1_), Varg::from(q2_)]) }; };
                { let q0_ = cx.lptr(44); let q1_ = cx.i(cx.lpp(22).add(64)); let q2_ = cx.i(cx.lpp(22).add(68)); let q3_ = cx.i(cx.lpp(22).add(72)); let q4_ = 0; ov_vec4_xyzw(cx, q0_, q1_, q2_, q3_, q4_) };
                if ({ let q0_ = cx.lpp(23); f_1e5d40(cx, q0_) } != 0) {
                    { let q0_ = cx.b(Ptr::glob(0x6FDC88)); let q1_ = cx.lptr(68); let q2_ = cx.lptr(68); let q3_ = cx.lptr(44); let q4_ = cx.f(cx.ptr_at(cx.lpp(23).add(136)).add(52)); ov_particle_trail(cx, q0_, q1_, q2_, q3_, q4_) };
                }
                if (cx.gc.objects_angular_damping_increase_time != 0.0f32) {
                    v3 = vsmath::exp((((((-(cx.i(cx.lpp(22).add(532)) as f32)) / 25.0f32) as f64) / (cx.gc.objects_angular_damping_increase_time as f64)) as f32));
                    { let p = cx.lpp(22).add(28); let t = ((v3 * (cx.gc.objects_angular_damping as f64)) as f32); cx.set_f(p, t); }
                }
                if (cx.gc.objects_linear_damping_increase_time != 0.0f32) {
                    v4 = vsmath::exp((((((-(cx.i(cx.lpp(22).add(532)) as f32)) / 25.0f32) as f64) / (cx.gc.objects_linear_damping_increase_time as f64)) as f32));
                    { let p = cx.lpp(22).add(24); let t = ((v4 * (cx.gc.objects_linear_damping as f64)) as f32); cx.set_f(p, t); }
                }
                { let t = cx.ptr_at(cx.ptr_at(cx.lpp(23).add(100)).add(32)); cx.set_lpp(9, t); }
                { let q0_ = cx.lpp(22); let q1_ = cx.lpp(9).add(16); f_36a090(cx, q0_, q1_) };
                { let _ = { let q0_ = cx.lpp(22); let q1_ = cx.lpp(9); vcall(cx, 28, q0_, &[Varg::from(q1_)]) }; };
                if ((cx.i(cx.lpp(23).add(152)) & 32) != 0) {
                    { let q0_ = cx.lpp(9); f_3785b0(cx, q0_) };
                } else {
                    { let q0_ = cx.lpp(9); f_3781b0(cx, q0_) };
                }
                if ({ let p = ov_physics_manager(cx).add(2836); cx.i(p) } != 0) {
                    x = cx.f(cx.lpp(22).add(72));
                    v12 = cx.f(cx.lpp(22).add(64));
                    v11 = cx.lpp(23);
                    v5 = ov_physics_manager(cx);
                    { let q0_ = cx.ptr_at(v5.add(2836)); let q1_ = v11; let q2_ = v12; let q3_ = x; f_384c70(cx, q0_, q1_, q2_, q3_) };
                }
                if (cx.i(cx.lpp(22).add(536)) <= 1) {
                    { let q0_ = cx.lpp(22).add(8); let q1_ = Ptr::gc(1848); f_1e4370(cx, q0_, q1_) };
                } else {
                    { let q0_ = cx.lpp(22).add(8); let q1_ = Ptr::gc(1860); f_1e4370(cx, q0_, q1_) };
                }
                let _ = cx.preinc_i(cx.lpp(22).add(536));
                { let t = 0.0f32; cx.set_lf(32, t); }
                { let t = 0.0f32; cx.set_lf(28, t); }
                if (cx.i(cx.lpp(22).add(532)) > 7) {
                    { let t = 0.0f32; cx.set_lf(24, t); }
                    'lp3: loop {
                        if !(cx.li(24) < 6) { break 'lp3; }
                        'blp3: {
                            if (cx.f(cx.lpp(22).add((4_i32).wrapping_mul(cx.li(24))).add(556)) > cx.gc.max_position_change_to_put_to_sleep) {
                                { let t = 1; cx.set_li(32, t); }
                                break 'lp3;
                            }
                        }
                        let _ = cx.preinc_i(cx.lptr(24));
                    }
                    if (cx.li(32) == 0) {
                        v6 = { let q0_ = cx.lpp(22); ov_car_omega(cx, q0_) };
                        max_angle_change_to_put_to_sleep = (f_1e4440(cx, v6) as f64);
                        if ((((cx.gc.max_omega_to_put_to_sleep as f64) * (cx.gc.max_omega_to_put_to_sleep as f64)) > max_angle_change_to_put_to_sleep) && (cx.i(cx.lpp(22).add(536)) <= 3)) {
                            { let t = 0.0f32; cx.set_lf(20, t); }
                            'lp4: loop {
                                if !(cx.li(20) < 6) { break 'lp4; }
                                'blp4: {
                                    if (cx.f(cx.lpp(22).add((16_i32).wrapping_mul(cx.li(20))).add(596)) != 3.4028235e38f32) {
                                        { let q0_ = cx.lpp(22).add((16_i32).wrapping_mul(cx.li(20))).add(584); let q1_ = cx.lpp(22).add(80); f_1e5190(cx, q0_, q1_) };
                                        xa = (max_angle_change_to_put_to_sleep as f32);
                                        v8 = vsmath::fabs(xa);
                                        max_angle_change_to_put_to_sleep = (cx.gc.max_angle_change_to_put_to_sleep as f64);
                                        if (v8 <= max_angle_change_to_put_to_sleep) {
                                            break 'blp4;
                                        }
                                    }
                                    { let t = 1; cx.set_li(32, t); }
                                    break 'lp4;
                                }
                                let _ = cx.preinc_i(cx.lptr(20));
                            }
                            if (cx.li(32) == 0) {
                                { let t = 1; cx.set_li(28, t); }
                            }
                        }
                    }
                }
                { let q0_ = cx.lptr(4); let q1_ = cx.lpp(22).add(64); let q2_ = cx.lpp(22).add(544); f_1e43f0(cx, q0_, q1_, q2_) };
                v9 = ({ let q0_ = cx.lptr(4); f_1e35d0(cx, q0_) } as f64);
                { let p = cx.lpp(22).add((4_i32).wrapping_mul(cx.i(cx.lpp(22).add(580)))).add(556); let t = (v9 as f32); cx.set_f(p, t); }
                { let q0_ = cx.lpp(22).add((16_i32).wrapping_mul(cx.i(cx.lpp(22).add(580)))).add(584); let q1_ = cx.lpp(22).add(80); f_1e5150(cx, q0_, q1_) };
                if (cx.preinc_i(cx.lpp(22).add(580)) == 6) {
                    { let p = cx.lpp(22).add(580); let t = 0; cx.set_i(p, t); }
                }
                v2 = { let q0_ = cx.lpp(22).add(544); let q1_ = cx.lpp(22).add(64); f_1e4370(cx, q0_, q1_) };
                if (cx.li(28) != 0) {
                    v2 = Ptr::raw((v2.bits() & !(0xFFu32 as i32)) | ((({ let q0_ = cx.lpp(23); f_1e4ae0(cx, q0_) }) & 0xFF) << 0));
                }
            } else {
                { let q0_ = cx.lpp(23); f_1e5670(cx, q0_) };
                v2 = Ptr::raw((cx.i(cx.lpp(23).add(152)) & -3));
                { let p = cx.lpp(23).add(152); let t = v2; cx.set_ptr(p, t); }
            }
        }
    }
    v13 = v2;
    return v13;
}

/// PhysicsObjectParticleTracker__1E5150 0x001E5150
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5150(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = cx.i(a2); cx.set_i(p, t); }
    { let p = a1.add(4); let t = cx.i(a2.add(4)); cx.set_i(p, t); }
    { let p = a1.add(8); let t = cx.i(a2.add(8)); cx.set_i(p, t); }
    { let p = a1.add(12); let t = cx.i(a2.add(12)); cx.set_i(p, t); }
    return a1;
}

/// PhysicsObjectParticleTracker__1E5190 0x001E5190
pub fn f_1e5190(cx: &mut Cx, a1: Ptr, a2: Ptr) -> i64 {
    cx.push_frame(8);
    let r = f_1e5190_body(cx, a1, a2);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1e5190_body(cx: &mut Cx, a1: Ptr, a2: Ptr) -> i64 {
    let mut v2 = 0i64;
    let mut v4 = 0i64;
    { let t = -107374176.0f32; cx.set_lf(0, t); }
    { let t = -107374176.0f32; cx.set_lf(4, t); }
    { let t = ((f_1e3600(cx, a1, a2) + ((cx.f(a1.add(12)) as f64) * (cx.f(a2.add(12)) as f64))) as f32); cx.set_lf(0, t); }
    { let q0_ = cx.lptr(0); let q1_ = -1.0f32; let q2_ = 1.0f32; f_1e5230(cx, q0_, q1_, q2_) };
    let _ = vsmath::acos(cx.lf(0));
    v4 = v2;
    return v4;
}

/// PhysicsObjectParticleTracker__1E5230 0x001E5230
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5230(cx: &mut Cx, mut a1: Ptr, a2: f32, a3: f32) -> Ptr {
    let mut result = Ptr::NULL;
    result = a1;
    if (a2 <= cx.f(a1)) {
        if (cx.f(a1) > a3) {
            { let p = a1; let t = a3; cx.set_f(p, t); }
            return a1;
        }
    } else {
        { let p = a1; let t = a2; cx.set_f(p, t); }
    }
    return result;
}

/// PhysicsObjectParticleTracker__1E5270 0x001E5270
pub fn f_1e5270(cx: &mut Cx, this: Ptr, a3: Ptr, a4: Ptr) -> i64 {
    cx.push_frame(72);
    let r = f_1e5270_body(cx, this, a3, a4);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1e5270_body(cx: &mut Cx, this: Ptr, a3: Ptr, a4: Ptr) -> i64 {
    let mut v3 = Ptr::NULL;
    let mut v4 = Ptr::NULL;
    let mut v5 = 0i64;
    let mut v7 = Ptr::NULL;
    let mut v8 = Ptr::NULL;
    let mut v9 = 0.0f32;
    let mut v10 = 0.0f32;
    let mut v11 = 0.0f32;
    let mut v12 = 0.0f32;
    let mut v13 = 0.0f32;
    let mut v14 = 0.0f32;
    let mut v15 = 0i64;
    { let t = this; cx.set_lpp(17, t); }
    if ((cx.i(this.add(100)) != 0) && (cx.i(cx.ptr_at(cx.lpp(17).add(100)).add(32)) != 0)) {
        { let t = cx.ptr_at(cx.ptr_at(cx.lpp(17).add(100)).add(32)); cx.set_lpp(16, t); }
        { let t = { let q0_ = cx.lpp(17); crate::ov_ident!(cx, q0_) }; cx.set_lpp(15, t); }
        { let q0_ = cx.lpp(16).add(16); let q1_ = cx.lpp(15); f_1e4370(cx, q0_, q1_) };
        { let q0_ = cx.lpp(16).add(28); let q1_ = cx.lpp(15).add(16); f_1e4370(cx, q0_, q1_) };
        { let q0_ = cx.lpp(16).add(40); let q1_ = cx.lpp(15).add(32); f_1e4370(cx, q0_, q1_) };
        { let q0_ = cx.lpp(16); let q1_ = cx.lpp(15).add(48); f_1e4370(cx, q0_, q1_) };
        { let q0_ = cx.lptr(44); let q1_ = cx.ptr_at(cx.lpp(17).add(100)).add(4); let q2_ = cx.lpp(16).add(16); f_1e4650(cx, q0_, q1_, q2_) };
        { let q0_ = cx.lpp(16); let q1_ = cx.lptr(44); f_1e55d0(cx, q0_, q1_) };
        if ((cx.i(cx.lpp(17).add(152)) & 32) != 0) {
            { let q0_ = cx.lpp(16); f_3785b0(cx, q0_) };
        } else {
            { let q0_ = cx.lpp(16); f_3781b0(cx, q0_) };
        }
        if ({ let q0_ = cx.ptr_at(cx.lpp(17).add(100)); f_1b41f0(cx, q0_) } != 0) {
            { let t = cx.ptr_at(cx.ptr_at(cx.lpp(17).add(100)).add(36)); cx.set_lpp(9, t); }
            { let q0_ = cx.lpp(9); let q1_ = cx.lpp(16).add(16); f_36a3e0(cx, q0_, q1_) };
            { let q0_ = cx.lpp(9); let q1_ = cx.lpp(16); f_36b0f0(cx, q0_, q1_) };
            { let q0_ = cx.lpp(9); f_1e6000(cx, q0_) };
            if (a3.bits() != Ptr::NULL.bits()) {
                { let q0_ = cx.lpp(9).add(32); let q1_ = a3; f_1e4370(cx, q0_, q1_) };
            }
            if (a4.bits() != Ptr::NULL.bits()) {
                { let q0_ = cx.lpp(9).add(48); let q1_ = a4; let q2_ = 0.017453292f32; f_1e5620(cx, q0_, q1_, q2_) };
            }
            { let _ = { let q0_ = cx.lpp(9); vcall(cx, 32, q0_, &[]) }; };
        }
        if ({ let p = ov_physics_manager(cx).add(2836); cx.i(p) } == 0) {
            crate::ov_ignore!(cx, 0 /*str*/);
            crate::ov_ignore!(cx, "new CAxisSortedList");
            { let t = ov_new(cx, 64); cx.set_lpp(3, t); }
            if (cx.lpp(3).bits() != 0) {
                { let t = { let q0_ = cx.lpp(3); let q1_ = 96; let q2_ = 10; let q3_ = 20; let q4_ = 2000; f_383fc0(cx, q0_, q1_, q2_, q3_, q4_) }; cx.set_lpp(1, t); }
            } else {
                { let t = 0; cx.set_lpp(1, Ptr::raw(t)); }
            }
            { let p = ov_physics_manager(cx).add(2836); let t = cx.lpp(1); cx.set_ptr(p, t); }
            crate::ov_ignore!(cx);
        }
        v13 = cx.f(cx.lpp(16).add(12));
        v11 = cx.f(cx.lpp(16).add(8));
        v9 = cx.f(cx.lpp(16));
        v7 = cx.lpp(17);
        v3 = ov_physics_manager(cx);
        { let q0_ = cx.ptr_at(v3.add(2836)); let q1_ = v7; let q2_ = v9; let q3_ = v11; let q4_ = v13; let q5_ = 0; f_3847c0(cx, q0_, q1_, q2_, q3_, q4_, q5_) };
    }
    if (cx.i(cx.lpp(17).add(104)) != 0) {
        { let p = cx.lptr(20); let t = { let q0_ = cx.ptr_at(cx.lpp(17).add(104)); ov_vec4_x(cx, q0_) }; cx.set_f(p, t); }
        { let p = cx.lptr(24); let t = { let q0_ = cx.ptr_at(cx.lpp(17).add(104)); ov_vec4_y(cx, q0_) }; cx.set_f(p, t); }
        { let p = cx.lptr(28); let t = { let q0_ = cx.ptr_at(cx.lpp(17).add(104)); ov_vec4_z(cx, q0_) }; cx.set_f(p, t); }
        if ({ let p = ov_physics_manager(cx).add(2840); cx.i(p) } == 0) {
            crate::ov_ignore!(cx, 0 /*str*/);
            crate::ov_ignore!(cx, "new C2AxisSortedList2");
            { let t = ov_new(cx, 64); cx.set_lpp(2, t); }
            if (cx.lpp(2).bits() != 0) {
                { let t = { let q0_ = cx.lpp(2); let q1_ = 96; let q2_ = 10; let q3_ = 20; let q4_ = 3000; f_383fc0(cx, q0_, q1_, q2_, q3_, q4_) }; cx.set_lpp(0, t); }
            } else {
                { let t = 0; cx.set_lpp(0, Ptr::raw(t)); }
            }
            { let p = ov_physics_manager(cx).add(2840); let t = cx.lpp(0); cx.set_ptr(p, t); }
            crate::ov_ignore!(cx);
        }
        v14 = { let q0_ = cx.ptr_at(cx.lpp(17).add(104)); ov_vec4_w(cx, q0_) };
        v12 = cx.f(cx.lptr(28));
        v10 = cx.f(cx.lptr(20));
        v8 = cx.lpp(17);
        v4 = ov_physics_manager(cx);
        { let q0_ = cx.ptr_at(v4.add(2840)); let q1_ = v8; let q2_ = v10; let q3_ = v12; let q4_ = v14; let q5_ = 0; f_3847c0(cx, q0_, q1_, q2_, q3_, q4_, q5_) };
    }
    v5 = (v5 & 0xFFFF_FFFFi64) | ((((cx.i(cx.lpp(17).add(152)) | 1)) as u32 as i64) << 32);
    v5 = (v5 & !0xFFFF_FFFFi64) | ((cx.lpp(17).bits()) as i32 as u32 as i64);
    { let p = cx.lpp(17).add(152); let t = ((v5 >> 32) as i32); cx.set_i(p, t); }
    v15 = v5;
    return v15;
}

/// PhysicsObjectParticleTracker__1E55D0 0x001E55D0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e55d0(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = (((cx.f(a1) as f64) + (cx.f(a2) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a1.add(4)) as f64) + (cx.f(a2.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a1.add(8)) as f64) + (cx.f(a2.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// PhysicsObjectParticleTracker__1E5620 0x001E5620
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5620(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: f32) -> Ptr {
    { let p = a1; let t = (((cx.f(a2) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a2.add(4)) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a2.add(8)) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// PhysicsObjectParticleTracker__1E5670 0x001E5670
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5670(cx: &mut Cx, this: Ptr) -> i32 {
    let mut v2 = Ptr::NULL;
    let mut v3 = Ptr::NULL;
    if (cx.i(this.add(100)) != 0) {
        let _ = f_1e4ae0(cx, this);
    }
    if (cx.i(this.add(100)) != 0) {
        v3 = { let p = ov_physics_manager(cx).add(2836); cx.ptr_at(p) };
        if (v3.bits() != Ptr::NULL.bits()) {
            let _ = f_3844f0(cx, v3, this);
        }
    }
    if (cx.i(this.add(104)) != 0) {
        v2 = { let p = ov_physics_manager(cx).add(2840); cx.ptr_at(p) };
        if (v2.bits() != Ptr::NULL.bits()) {
            let _ = f_3844f0(cx, v2, this);
        }
    }
    { let p = this.add(152); let t = cx.i(p).bitand_(!1); cx.set_i(p, t); }
    return this.bits();
}

/// PhysicsObjectParticleTracker__1E5740 0x001E5740
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5740(cx: &mut Cx, this: Ptr, a2: Ptr, a3: Ptr) -> i32 {
    let _ = f_2ac0c0(cx, this, a3);
    return f_14000(cx, this, a2);
}
