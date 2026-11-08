//! Translated from the decompilation by tools/phys_c2rs4.py (do not edit by hand; fixes go in the
//! translator or in the hand-written overrides).
#![allow(non_snake_case, unused_parens, unused_labels, unused_braces, unused_variables, unused_mut, unused_assignments, unreachable_code, dead_code, clippy::all)]

use crate::tr_prelude::*;

/// PhysicsObjectManager__1DE730 0x001DE730
pub fn f_1de730(cx: &mut Cx, this: Ptr, a2: i32, a3: i32) -> i32 {
    cx.push_frame(92);
    let r = f_1de730_body(cx, this, a2, a3);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1de730_body(cx: &mut Cx, this: Ptr, a2: i32, a3: i32) -> i32 {
    let mut v3 = 0i32;
    let mut v4 = 0i32;
    let mut v5 = 0i32;
    let mut v6 = 0i32;
    { let t = this; cx.set_lpp(22, t); }
    let _ = cx.preinc_i(this.add(3216));
    if (cx.i(cx.lpp(22).add(3216)) >= 128) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            let _ = crate::ov_zero_i!(cx, "m_CurrentList<MAX_PHYSICS_OBJECT_LISTS", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjectManager.cpp", 209);
            { let t = v3; cx.set_li(84, t); }
            if (v3 == 3) {
            }
            { let t = 257; let o = cx.li(32); cx.set_li(32, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
        } else {
            { let t = 1; let o = cx.li(32); cx.set_li(32, (o & !(255 << 16)) | ((t & 255) << 16)); }
        }
    }
    if (cx.i(cx.lpp(22).add((4_i32).wrapping_mul(cx.i(cx.lpp(22).add(3216)))).add(1152)) != 0) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            let _ = crate::ov_zero_i!(cx, "m_pPhysicsObjectArrays[m_CurrentList] == NULL", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjectManager.cpp", 210);
            { let t = v4; cx.set_li(80, t); }
            if (v4 == 3) {
            }
            { let t = 257; let o = cx.li(32); cx.set_li(32, (o & !(65535 << 0)) | ((t & 65535) << 0)); }
        } else {
            { let t = 1; let o = cx.li(32); cx.set_li(32, (o & !(255 << 0)) | ((t & 255) << 0)); }
        }
    }
    crate::ov_ignore!(cx, "PreAllocated Physics Object List");
    { let t = a2; cx.set_li(68, t); }
    { let t = ov_new(cx, (160_i32).wrapping_mul(a2).wrapping_add(4)); cx.set_lpp(16, t); }
    if (cx.lpp(16).bits() != 0) {
        { let p = cx.lpp(16); let t = cx.li(68); cx.set_i(p, t); }
        { let q0_ = cx.lpp(16).add(4); let q1_ = 160; let q2_ = cx.li(68); let q3_ = Ptr::raw(Ptr::func(0x1E3630).bits()); ov_vector_ctor(cx, q0_, q1_, q2_, q3_) };
        { let t = cx.lpp(16).add(4); cx.set_lpp(7, t); }
    } else {
        { let t = 0; cx.set_lpp(7, Ptr::raw(t)); }
    }
    { let p = cx.lpp(22).add((4_i32).wrapping_mul(cx.i(cx.lpp(22).add(3216)))).add(1152); let t = cx.lpp(7); cx.set_ptr(p, t); }
    { let p = cx.lpp(22).add((4_i32).wrapping_mul(cx.i(cx.lpp(22).add(3216)))).add(512); let t = a2; cx.set_i(p, t); }
    { let p = cx.lpp(22).add((4_i32).wrapping_mul(cx.i(cx.lpp(22).add(3216)))); let t = 0; cx.set_i(p, t); }
    { let p = cx.lpp(22).add((4_i32).wrapping_mul(cx.i(cx.lpp(22).add(3216)))).add(2704); let t = a3; cx.set_i(p, t); }
    { let p = cx.lpp(22).add(cx.i(cx.lpp(22).add(3216))).add(1024); let t = 1; cx.set_b(p, t); }
    if (a3 == 1) {
        if (cx.i(cx.lpp(22).add((4_i32).wrapping_mul(cx.i(cx.lpp(22).add(3216)))).add(1664)) != 0) {
            if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
                let _ = crate::ov_zero_i!(cx, "m_pCollisionArrays[m_CurrentList] == NULL", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjectManager.cpp", 225);
                { let t = v5; cx.set_li(76, t); }
                if (v5 == 3) {
                }
                { let t = 257; let o = cx.li(24); cx.set_li(24, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
            } else {
                { let t = 1; let o = cx.li(24); cx.set_li(24, (o & !(255 << 16)) | ((t & 255) << 16)); }
            }
        }
        crate::ov_ignore!(cx, "PreAllocated Physics Object List- Collision Array");
        { let t = a2; cx.set_li(60, t); }
        { let t = ov_new(cx, (40_i32).wrapping_mul(a2)); cx.set_lpp(14, t); }
        if (cx.lpp(14).bits() != 0) {
            { let q0_ = cx.lpp(14); let q1_ = 40; let q2_ = cx.li(60); let q3_ = Ptr::raw(Ptr::func(0x1E3280).bits()); ov_vector_ctor(cx, q0_, q1_, q2_, q3_) };
            { let t = cx.lpp(14); cx.set_lpp(5, t); }
        } else {
            { let t = 0; cx.set_lpp(5, Ptr::raw(t)); }
        }
        { let p = cx.lpp(22).add((4_i32).wrapping_mul(cx.i(cx.lpp(22).add(3216)))).add(1664); let t = cx.lpp(5); cx.set_ptr(p, t); }
    }
    if (a3 == 3) {
        if (cx.i(cx.lpp(22).add((4_i32).wrapping_mul(cx.i(cx.lpp(22).add(3216)))).add(2176)) != 0) {
            if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
                let _ = crate::ov_zero_i!(cx, "m_pImmovableArrays[m_CurrentList] == NULL", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjectManager.cpp", 232);
                { let t = v6; cx.set_li(72, t); }
                if (v6 == 3) {
                }
                { let t = 257; let o = cx.li(16); cx.set_li(16, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
            } else {
                { let t = 1; let o = cx.li(16); cx.set_li(16, (o & !(255 << 16)) | ((t & 255) << 16)); }
            }
        }
        crate::ov_ignore!(cx, "PreAllocated Physics Object List- Immovable Array");
        { let t = a2; cx.set_li(52, t); }
        { let t = ov_new(cx, (24_i32).wrapping_mul(a2)); cx.set_lpp(12, t); }
        if (cx.lpp(12).bits() != 0) {
            { let q0_ = cx.lpp(12); let q1_ = 24; let q2_ = cx.li(52); let q3_ = Ptr::raw(Ptr::func(0x1E1DB0).bits()); ov_vector_ctor(cx, q0_, q1_, q2_, q3_) };
            { let t = cx.lpp(12); cx.set_lpp(3, t); }
        } else {
            { let t = 0; cx.set_lpp(3, Ptr::raw(t)); }
        }
        { let p = cx.lpp(22).add((4_i32).wrapping_mul(cx.i(cx.lpp(22).add(3216)))).add(2176); let t = cx.lpp(3); cx.set_ptr(p, t); }
    }
    if (cx.i(cx.lpp(22).add(2692)) == 0) {
        { let t = ov_new(cx, 4804); cx.set_lpp(11, t); }
        if (cx.lpp(11).bits() != 0) {
            { let p = cx.lpp(11); let t = 30; cx.set_i(p, t); }
            { let q0_ = cx.lpp(11).add(4); let q1_ = 160; let q2_ = 30; let q3_ = Ptr::raw(Ptr::func(0x1E3630).bits()); ov_vector_ctor(cx, q0_, q1_, q2_, q3_) };
            { let t = cx.lpp(11).add(4); cx.set_lpp(2, t); }
        } else {
            { let t = 0; cx.set_lpp(2, Ptr::raw(t)); }
        }
        { let p = cx.lpp(22).add(2692); let t = cx.lpp(2); cx.set_ptr(p, t); }
    }
    if (cx.i(cx.lpp(22).add(2696)) == 0) {
        { let t = ov_new(cx, 1200); cx.set_lpp(10, t); }
        if (cx.lpp(10).bits() != 0) {
            { let q0_ = cx.lpp(10); let q1_ = 40; let q2_ = 30; let q3_ = Ptr::raw(Ptr::func(0x1E3280).bits()); ov_vector_ctor(cx, q0_, q1_, q2_, q3_) };
            { let t = cx.lpp(10); cx.set_lpp(1, t); }
        } else {
            { let t = 0; cx.set_lpp(1, Ptr::raw(t)); }
        }
        { let p = cx.lpp(22).add(2696); let t = cx.lpp(1); cx.set_ptr(p, t); }
    }
    if (cx.i(cx.lpp(22).add(2700)) == 0) {
        { let t = ov_new(cx, 2160); cx.set_lpp(9, t); }
        if (cx.lpp(9).bits() != 0) {
            { let q0_ = cx.lpp(9); let q1_ = 72; let q2_ = 30; let q3_ = Ptr::raw(Ptr::func(0x377B50).bits()); ov_vector_ctor(cx, q0_, q1_, q2_, q3_) };
            { let t = cx.lpp(9); cx.set_lpp(0, t); }
        } else {
            { let t = 0; cx.set_lpp(0, Ptr::raw(t)); }
        }
        { let p = cx.lpp(22).add(2700); let t = cx.lpp(0); cx.set_ptr(p, t); }
    }
    return (160_i32).wrapping_mul(a2);
}

/// PhysicsObjectManager__1DEBB0 0x001DEBB0
pub fn f_1debb0(cx: &mut Cx, this: Ptr, a3: Ptr, a4: Ptr) -> i32 {
    cx.push_frame(116);
    let r = f_1debb0_body(cx, this, a3, a4);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1debb0_body(cx: &mut Cx, this: Ptr, a3: Ptr, a4: Ptr) -> i32 {
    let mut v3 = 0i32;
    let mut v5 = Ptr::NULL;
    let mut v6 = 0.0f64;
    let mut v7 = Ptr::NULL;
    let mut v8 = 0i32;
    let mut v9 = 0i32;
    let mut v10 = 0i32;
    let mut v11 = 0i32;
    let mut v12 = 0.0f64;
    let mut v13 = Ptr::NULL;
    let mut v14 = Ptr::NULL;
    let mut v15 = 0.0f64;
    let mut v16 = Ptr::NULL;
    let mut v17 = 0i32;
    let mut v18 = 0i32;
    let mut v19 = 0i32;
    let mut v20 = 0i32;
    let mut v21 = 0.0f64;
    let mut v22 = Ptr::NULL;
    let mut v23 = Ptr::NULL;
    let mut v24 = 0.0f64;
    let mut v25 = Ptr::NULL;
    let mut v26 = 0i32;
    let mut v27 = 0i32;
    let mut v28 = 0i32;
    let mut v29 = 0i32;
    let mut v30 = 0.0f64;
    let mut v31 = Ptr::NULL;
    { let t = this; cx.set_lpp(28, t); }
    { let t = cx.ptr_at(Ptr::raw(cx.i(a3.add(100)).wrapping_add(32))); cx.set_lpp(27, t); }
    { let t = cx.i(cx.lpp(27).add(60)); cx.set_li(104, t); }
    { let t = cx.i(cx.lpp(27).add(68)); cx.set_li(100, t); }
    { let t = cx.i(cx.lpp(27).add(64)); cx.set_li(96, t); }
    if ((cx.i(a3.add(100)) != 0) && (cx.i(Ptr::raw(cx.i(a3.add(100)).wrapping_add(32))) != 0)) {
        { let t = 0; cx.set_li(88, t); }
        { let p = cx.lptr(84); let t = f_1dfb50(cx, a4); cx.set_f(p, t); }
        if (cx.f(cx.lptr(84)) > cx.gc.upper_limit_on_collision_speed) {
            { let q0_ = a4; let q1_ = (((cx.gc.upper_limit_on_collision_speed as f64) / (cx.f(cx.lptr(84)) as f64)) as f32); f_1dfaa0(cx, q0_, q1_) };
            { let t = (cx.gc.upper_limit_on_collision_speed.to_bits() as i32); cx.set_li(84, t); }
        }
        if (cx.gc.peice_radnom_omega_addition != 0.0f32) {
            { let p = cx.lptr(88); let t = ((((cx.f(cx.lptr(84)) / cx.gc.upper_limit_on_collision_speed) as f64) * (cx.gc.peice_radnom_omega_addition as f64)) as f32); cx.set_f(p, t); }
        }
        { let p = cx.lptr(80); let t = { let q0_ = cx.ptr_at(Ptr::raw(cx.i(a3.add(100)).wrapping_add(32))); f_378f60(cx, q0_) }; cx.set_f(p, t); }
        { let t = 0; cx.set_li(76, t); }
        'lp8: loop {
            if !(cx.li(76) < cx.b(cx.lpp(27).add(52))) { break 'lp8; }
            'blp8: {
                { let t = cx.li(104).wrapping_add((116_i32).wrapping_mul(cx.li(76))); cx.set_lpp(18, Ptr::raw(t)); }
                { let t = cx.i(cx.lpp(28).add(2692)).wrapping_add((160_i32).wrapping_mul(cx.i(cx.lpp(28).add(2688)))); cx.set_lpp(17, Ptr::raw(t)); }
                { let p = cx.lpp(17).add(100); let t = cx.i(cx.lpp(28).add(2696)).wrapping_add((40_i32).wrapping_mul(cx.i(cx.lpp(28).add(2688)))); cx.set_i(p, t); }
                { let p = cx.ptr_at(cx.lpp(17).add(100)).add(32); let t = cx.i(cx.lpp(28).add(2700)).wrapping_add((72_i32).wrapping_mul(cx.i(cx.lpp(28).add(2688)))); cx.set_i(p, t); }
                { let p = cx.lpp(17).add(136); let t = cx.i(a3.add(136)); cx.set_i(p, t); }
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(17).add(100)).add(32)).add(60); let t = cx.lpp(18); cx.set_ptr(p, t); }
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(17).add(100)).add(32)).add(52); let t = 1; cx.set_b(p, t); }
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(17).add(100)).add(32)).add(54); let t = 0; cx.set_b(p, t); }
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(17).add(100)).add(32)).add(53); let t = 0; cx.set_b(p, t); }
                { let q0_ = cx.ptr_at(cx.lpp(17).add(100)).add(4); let q1_ = Ptr::raw(cx.i(a3.add(100)).wrapping_add(4)); let q2_ = cx.lpp(18).add(64); f_1dfa50(cx, q0_, q1_, q2_) };
                v5 = Ptr::raw(cx.i(a3.add(100)));
                v6 = { let q0_ = cx.lpp(18); f_1e1e60(cx, q0_) };
                { let p = cx.ptr_at(cx.lpp(17).add(100)); let t = (((v6 * (cx.f(v5) as f64)) / (cx.f(cx.lptr(80)) as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.ptr_at(cx.lpp(17).add(100)).add(29); let t = 0; cx.set_b(p, t); }
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(17).add(100)).add(32)); let q1_ = cx.ptr_at(cx.lpp(17).add(100)).add(16); let q2_ = cx.f(cx.ptr_at(cx.lpp(17).add(100))); f_379660(cx, q0_, q1_, q2_) };
                { let p = cx.ptr_at(cx.lpp(17).add(100)).add(28); let t = 0; cx.set_b(p, t); }
                if (cx.gc.spherical_inertia_threshold != 0.0f32) {
                    { let t = 2139095039; cx.set_li(64, t); }
                    { let t = 0; cx.set_li(60, t); }
                    'lp9: loop {
                        if !(cx.li(60) < 3) { break 'lp9; }
                        'blp9: {
                            if (cx.f(cx.lptr(64)) > cx.f(cx.ptr_at(cx.lpp(17).add(100)).add((4_i32).wrapping_mul(cx.li(60))).add(16))) {
                                { let t = cx.i(cx.ptr_at(cx.lpp(17).add(100)).add((4_i32).wrapping_mul(cx.li(60))).add(16)); cx.set_li(64, t); }
                            }
                        }
                        let _ = cx.preinc_i(cx.lptr(60));
                    }
                    { let p = cx.ptr_at(cx.lpp(17).add(100)).add(28); let t = 1; cx.set_b(p, t); }
                    { let t = 0; cx.set_li(56, t); }
                    'lp10: loop {
                        if !(cx.li(56) < 3) { break 'lp10; }
                        'blp10: {
                            if (cx.f(cx.ptr_at(cx.lpp(17).add(100)).add((4_i32).wrapping_mul(cx.li(56))).add(16)) > (cx.f(cx.lptr(64)) * cx.gc.spherical_inertia_threshold)) {
                                { let p = cx.ptr_at(cx.lpp(17).add(100)).add(28); let t = 0; cx.set_b(p, t); }
                                break 'lp10;
                            }
                        }
                        let _ = cx.preinc_i(cx.lptr(56));
                    }
                }
                v7 = crate::ov_ident!(cx, a3);
                { let q0_ = cx.lpp(17); let q1_ = v7; crate::ov_ignore!(cx, q0_, q1_) };
                { let p = cx.lpp(17).add(152); let t = cx.i(p).bitor_(32); cx.set_i(p, t); }
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(17).add(100)).add(32)); f_378040(cx, q0_) };
                { let q0_ = cx.lpp(17); let q1_ = Ptr::NULL; let q2_ = Ptr::NULL; f_1e5270(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lpp(17); f_1e4710(cx, q0_) };
                if (cx.gc.peice_linear_velocity_drag_scale != 0.0f32) {
                    { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(17).add(100)).add(36)).add(32); let q1_ = a4; let q2_ = cx.gc.peice_linear_velocity_drag_scale; f_1dfaf0(cx, q0_, q1_, q2_) };
                }
                if (cx.gc.peice_radnom_omega_addition != 0.0f32) {
                    v8 = cx.i(cx.ptr_at(cx.lpp(17).add(100)).add(36));
                    { let p = Ptr::raw(v8.wrapping_add(48)); let t = ((((f_1dfbb0(cx) as f64) * (cx.f(cx.lptr(88)) as f64)) + (cx.f(Ptr::raw(v8.wrapping_add(48))) as f64)) as f32); cx.set_f(p, t); }
                    v9 = cx.i(cx.ptr_at(cx.lpp(17).add(100)).add(36));
                    { let p = Ptr::raw(v9.wrapping_add(52)); let t = ((((f_1dfbb0(cx) as f64) * (cx.f(cx.lptr(88)) as f64)) + (cx.f(Ptr::raw(v9.wrapping_add(52))) as f64)) as f32); cx.set_f(p, t); }
                    v10 = cx.i(cx.ptr_at(cx.lpp(17).add(100)).add(36));
                    { let p = Ptr::raw(v10.wrapping_add(56)); let t = ((((f_1dfbb0(cx) as f64) * (cx.f(cx.lptr(88)) as f64)) + (cx.f(Ptr::raw(v10.wrapping_add(56))) as f64)) as f32); cx.set_f(p, t); }
                }
                { let p = cx.lpp(17).add(148); let t = cx.i(a3.add(148)); cx.set_i(p, t); }
                if (cx.i(cx.lpp(18).add(112)) >= 15) {
                    if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
                        let _ = crate::ov_zero_i!(cx, "pBox->ID<PHYS_OBJ_INVALID_INDEX", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjectManager.cpp", 366);
                        { let t = v11; cx.set_li(52, t); }
                        if (v11 == 3) {
                        }
                        { let t = 257; let o = cx.li(4); cx.set_li(4, (o & !(65535 << 0)) | ((t & 65535) << 0)); }
                    } else {
                        { let t = 1; let o = cx.li(4); cx.set_li(4, (o & !(255 << 0)) | ((t & 255) << 0)); }
                    }
                }
                { let p = cx.lpp(17).add(152); let t = (((cx.i(cx.lpp(18).add(112)) & 15) << 8) | (cx.i(cx.lpp(17).add(152)) & -3841)); cx.set_i(p, t); }
                { let p = cx.lpp(17).add(152); let t = cx.i(p).bitor_(4); cx.set_i(p, t); }
                v12 = ({ let q0_ = cx.ptr_at(cx.lpp(17).add(148)).add(16); ov_vec4_w(cx, q0_) } as f64);
                { let p = cx.lpp(17).add(140); let t = (v12 as f32); cx.set_f(p, t); }
                { let p = cx.lpp(17).add(140); let t = (((cx.f(cx.lpp(17).add(140)) as f64) + 1.0f64) as f32); cx.set_f(p, t); }
                { let p = cx.lpp(17).add(140); let t = (((cx.f(cx.lpp(17).add(140)) as f64) * (cx.f(Ptr::glob(0x75BC70)) as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.lpp(17).add(140); let t = ((1.0f64 / (cx.f(cx.lpp(17).add(140)) as f64)) as f32); cx.set_f(p, t); }
                { let q0_ = cx.lpp(17).add(108); let q1_ = 0; let q2_ = 28; ov_memset(cx, q0_, q1_, q2_) };
                v13 = { let q0_ = cx.lpp(17); crate::ov_ident!(cx, q0_) };
                { let p = cx.lpp(17).add(108); let t = v13; cx.set_ptr(p, t); }
                cx.memcpy(cx.lpp(17).add(64), a3.add(64), 32);
                { let p = cx.lpp(17).add(120); let t = cx.lpp(17).add(64); cx.set_ptr(p, t); }
                { let q0_ = cx.ptr_at(cx.lpp(17).add(120)).add(24); let q1_ = 0; crate::ov_ignore!(cx, q0_, q1_) };
                if (cx.preinc_i(cx.lpp(28).add(2688)) == 30) {
                    { let p = cx.lpp(28).add(2688); let t = 0; cx.set_i(p, t); }
                }
            }
            let _ = cx.preinc_i(cx.lptr(76));
        }
        { let t = 0; cx.set_li(48, t); }
        'lp11: loop {
            if !(cx.li(48) < cx.b(cx.lpp(27).add(54))) { break 'lp11; }
            'blp11: {
                { let t = cx.li(100).wrapping_add((cx.li(48) << 6)); cx.set_lpp(11, Ptr::raw(t)); }
                { let t = cx.i(cx.lpp(28).add(2692)).wrapping_add((160_i32).wrapping_mul(cx.i(cx.lpp(28).add(2688)))); cx.set_lpp(10, Ptr::raw(t)); }
                { let p = cx.lpp(10).add(100); let t = cx.i(cx.lpp(28).add(2696)).wrapping_add((40_i32).wrapping_mul(cx.i(cx.lpp(28).add(2688)))); cx.set_i(p, t); }
                { let p = cx.ptr_at(cx.lpp(10).add(100)).add(32); let t = cx.i(cx.lpp(28).add(2700)).wrapping_add((72_i32).wrapping_mul(cx.i(cx.lpp(28).add(2688)))); cx.set_i(p, t); }
                { let p = cx.lpp(10).add(136); let t = cx.i(a3.add(136)); cx.set_i(p, t); }
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(10).add(100)).add(32)).add(68); let t = cx.lpp(11); cx.set_ptr(p, t); }
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(10).add(100)).add(32)).add(52); let t = 0; cx.set_b(p, t); }
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(10).add(100)).add(32)).add(54); let t = 1; cx.set_b(p, t); }
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(10).add(100)).add(32)).add(53); let t = 0; cx.set_b(p, t); }
                { let q0_ = cx.ptr_at(cx.lpp(10).add(100)).add(4); let q1_ = Ptr::raw(cx.i(a3.add(100)).wrapping_add(4)); let q2_ = cx.lpp(11).add(36); f_1dfa50(cx, q0_, q1_, q2_) };
                v14 = Ptr::raw(cx.i(a3.add(100)));
                v15 = { let q0_ = cx.lpp(11); f_1e1e10(cx, q0_) };
                { let p = cx.ptr_at(cx.lpp(10).add(100)); let t = (((v15 * (cx.f(v14) as f64)) / (cx.f(cx.lptr(80)) as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.ptr_at(cx.lpp(10).add(100)).add(29); let t = 0; cx.set_b(p, t); }
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(10).add(100)).add(32)); let q1_ = cx.ptr_at(cx.lpp(10).add(100)).add(16); let q2_ = cx.f(cx.ptr_at(cx.lpp(10).add(100))); f_379660(cx, q0_, q1_, q2_) };
                { let p = cx.ptr_at(cx.lpp(10).add(100)).add(28); let t = 0; cx.set_b(p, t); }
                if (cx.gc.spherical_inertia_threshold != 0.0f32) {
                    { let t = 2139095039; cx.set_li(36, t); }
                    { let t = 0; cx.set_li(32, t); }
                    'lp12: loop {
                        if !(cx.li(32) < 3) { break 'lp12; }
                        'blp12: {
                            if (cx.f(cx.lptr(36)) > cx.f(cx.ptr_at(cx.lpp(10).add(100)).add((4_i32).wrapping_mul(cx.li(32))).add(16))) {
                                { let t = cx.i(cx.ptr_at(cx.lpp(10).add(100)).add((4_i32).wrapping_mul(cx.li(32))).add(16)); cx.set_li(36, t); }
                            }
                        }
                        let _ = cx.preinc_i(cx.lptr(32));
                    }
                    { let p = cx.ptr_at(cx.lpp(10).add(100)).add(28); let t = 1; cx.set_b(p, t); }
                    { let t = 0; cx.set_li(28, t); }
                    'lp13: loop {
                        if !(cx.li(28) < 3) { break 'lp13; }
                        'blp13: {
                            if (cx.f(cx.ptr_at(cx.lpp(10).add(100)).add((4_i32).wrapping_mul(cx.li(28))).add(16)) > (cx.f(cx.lptr(36)) * cx.gc.spherical_inertia_threshold)) {
                                { let p = cx.ptr_at(cx.lpp(10).add(100)).add(28); let t = 0; cx.set_b(p, t); }
                                break 'lp13;
                            }
                        }
                        let _ = cx.preinc_i(cx.lptr(28));
                    }
                }
                v16 = crate::ov_ident!(cx, a3);
                { let q0_ = cx.lpp(10); let q1_ = v16; crate::ov_ignore!(cx, q0_, q1_) };
                { let p = cx.lpp(10).add(152); let t = cx.i(p).bitor_(32); cx.set_i(p, t); }
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(10).add(100)).add(32)); f_378040(cx, q0_) };
                { let q0_ = cx.lpp(10); let q1_ = Ptr::NULL; let q2_ = Ptr::NULL; f_1e5270(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lpp(10); f_1e4710(cx, q0_) };
                if (cx.gc.peice_linear_velocity_drag_scale != 0.0f32) {
                    { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(10).add(100)).add(36)).add(32); let q1_ = a4; let q2_ = cx.gc.peice_linear_velocity_drag_scale; f_1dfaf0(cx, q0_, q1_, q2_) };
                }
                if (cx.gc.peice_radnom_omega_addition != 0.0f32) {
                    v17 = cx.i(cx.ptr_at(cx.lpp(10).add(100)).add(36));
                    { let p = Ptr::raw(v17.wrapping_add(48)); let t = ((((f_1dfbb0(cx) as f64) * (cx.f(cx.lptr(88)) as f64)) + (cx.f(Ptr::raw(v17.wrapping_add(48))) as f64)) as f32); cx.set_f(p, t); }
                    v18 = cx.i(cx.ptr_at(cx.lpp(10).add(100)).add(36));
                    { let p = Ptr::raw(v18.wrapping_add(52)); let t = ((((f_1dfbb0(cx) as f64) * (cx.f(cx.lptr(88)) as f64)) + (cx.f(Ptr::raw(v18.wrapping_add(52))) as f64)) as f32); cx.set_f(p, t); }
                    v19 = cx.i(cx.ptr_at(cx.lpp(10).add(100)).add(36));
                    { let p = Ptr::raw(v19.wrapping_add(56)); let t = ((((f_1dfbb0(cx) as f64) * (cx.f(cx.lptr(88)) as f64)) + (cx.f(Ptr::raw(v19.wrapping_add(56))) as f64)) as f32); cx.set_f(p, t); }
                }
                { let p = cx.lpp(10).add(148); let t = cx.i(a3.add(148)); cx.set_i(p, t); }
                if (cx.i(cx.lpp(11).add(60)) >= 15) {
                    if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
                        let _ = crate::ov_zero_i!(cx, "pCapsule->ID<PHYS_OBJ_INVALID_INDEX", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjectManager.cpp", 477);
                        { let t = v20; cx.set_li(24, t); }
                        if (v20 == 3) {
                        }
                        { let t = 257; let o = cx.li(0); cx.set_li(0, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
                    } else {
                        { let t = 1; let o = cx.li(0); cx.set_li(0, (o & !(255 << 16)) | ((t & 255) << 16)); }
                    }
                }
                { let p = cx.lpp(10).add(152); let t = (((cx.i(cx.lpp(11).add(60)) & 15) << 8) | (cx.i(cx.lpp(10).add(152)) & -3841)); cx.set_i(p, t); }
                { let p = cx.lpp(10).add(152); let t = cx.i(p).bitor_(4); cx.set_i(p, t); }
                v21 = ({ let q0_ = cx.ptr_at(cx.lpp(10).add(148)).add(16); ov_vec4_w(cx, q0_) } as f64);
                { let p = cx.lpp(10).add(140); let t = (v21 as f32); cx.set_f(p, t); }
                { let p = cx.lpp(10).add(140); let t = (((cx.f(cx.lpp(10).add(140)) as f64) + 1.0f64) as f32); cx.set_f(p, t); }
                { let p = cx.lpp(10).add(140); let t = (((cx.f(cx.lpp(10).add(140)) as f64) * (cx.f(Ptr::glob(0x75BC70)) as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.lpp(10).add(140); let t = ((1.0f64 / (cx.f(cx.lpp(10).add(140)) as f64)) as f32); cx.set_f(p, t); }
                { let q0_ = cx.lpp(10).add(108); let q1_ = 0; let q2_ = 28; ov_memset(cx, q0_, q1_, q2_) };
                v22 = { let q0_ = cx.lpp(10); crate::ov_ident!(cx, q0_) };
                { let p = cx.lpp(10).add(108); let t = v22; cx.set_ptr(p, t); }
                cx.memcpy(cx.lpp(10).add(64), a3.add(64), 32);
                { let p = cx.lpp(10).add(120); let t = cx.lpp(10).add(64); cx.set_ptr(p, t); }
                { let q0_ = cx.ptr_at(cx.lpp(10).add(120)).add(24); let q1_ = 0; crate::ov_ignore!(cx, q0_, q1_) };
                if (cx.preinc_i(cx.lpp(28).add(2688)) == 30) {
                    { let p = cx.lpp(28).add(2688); let t = 0; cx.set_i(p, t); }
                }
            }
            let _ = cx.preinc_i(cx.lptr(48));
        }
        { let t = 0; cx.set_li(20, t); }
        'lp14: loop {
            if !(cx.li(20) < cx.b(cx.lpp(27).add(53))) { break 'lp14; }
            'blp14: {
                { let t = cx.li(96).wrapping_add((36_i32).wrapping_mul(cx.li(20))); cx.set_lpp(4, Ptr::raw(t)); }
                { let t = cx.i(cx.lpp(28).add(2692)).wrapping_add((160_i32).wrapping_mul(cx.i(cx.lpp(28).add(2688)))); cx.set_lpp(3, Ptr::raw(t)); }
                { let p = cx.lpp(3).add(100); let t = cx.i(cx.lpp(28).add(2696)).wrapping_add((40_i32).wrapping_mul(cx.i(cx.lpp(28).add(2688)))); cx.set_i(p, t); }
                { let p = cx.ptr_at(cx.lpp(3).add(100)).add(32); let t = cx.i(cx.lpp(28).add(2700)).wrapping_add((72_i32).wrapping_mul(cx.i(cx.lpp(28).add(2688)))); cx.set_i(p, t); }
                { let p = cx.lpp(3).add(136); let t = cx.i(a3.add(136)); cx.set_i(p, t); }
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(3).add(100)).add(32)).add(64); let t = cx.lpp(4); cx.set_ptr(p, t); }
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(3).add(100)).add(32)).add(52); let t = 0; cx.set_b(p, t); }
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(3).add(100)).add(32)).add(54); let t = 0; cx.set_b(p, t); }
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(3).add(100)).add(32)).add(53); let t = 1; cx.set_b(p, t); }
                { let q0_ = cx.ptr_at(cx.lpp(3).add(100)).add(4); let q1_ = Ptr::raw(cx.i(a3.add(100)).wrapping_add(4)); let q2_ = cx.lpp(4).add(20); f_1dfa50(cx, q0_, q1_, q2_) };
                v23 = Ptr::raw(cx.i(a3.add(100)));
                v24 = { let q0_ = cx.lpp(4); f_1e1de0(cx, q0_) };
                { let p = cx.ptr_at(cx.lpp(3).add(100)); let t = (((v24 * (cx.f(v23) as f64)) / (cx.f(cx.lptr(80)) as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.ptr_at(cx.lpp(3).add(100)).add(29); let t = 0; cx.set_b(p, t); }
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(3).add(100)).add(32)); let q1_ = cx.ptr_at(cx.lpp(3).add(100)).add(16); let q2_ = cx.f(cx.ptr_at(cx.lpp(3).add(100))); f_379660(cx, q0_, q1_, q2_) };
                { let p = cx.ptr_at(cx.lpp(3).add(100)).add(28); let t = 1; cx.set_b(p, t); }
                v25 = crate::ov_ident!(cx, a3);
                { let q0_ = cx.lpp(3); let q1_ = v25; crate::ov_ignore!(cx, q0_, q1_) };
                { let p = cx.lpp(3).add(152); let t = cx.i(p).bitor_(32); cx.set_i(p, t); }
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(3).add(100)).add(32)); f_378040(cx, q0_) };
                { let q0_ = cx.lpp(3); let q1_ = Ptr::NULL; let q2_ = Ptr::NULL; f_1e5270(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lpp(3); f_1e4710(cx, q0_) };
                if (cx.gc.peice_linear_velocity_drag_scale != 0.0f32) {
                    { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(3).add(100)).add(36)).add(32); let q1_ = a4; let q2_ = cx.gc.peice_linear_velocity_drag_scale; f_1dfaf0(cx, q0_, q1_, q2_) };
                }
                if (cx.gc.peice_radnom_omega_addition != 0.0f32) {
                    v26 = cx.i(cx.ptr_at(cx.lpp(3).add(100)).add(36));
                    { let p = Ptr::raw(v26.wrapping_add(48)); let t = ((((f_1dfbb0(cx) as f64) * (cx.f(cx.lptr(88)) as f64)) + (cx.f(Ptr::raw(v26.wrapping_add(48))) as f64)) as f32); cx.set_f(p, t); }
                    v27 = cx.i(cx.ptr_at(cx.lpp(3).add(100)).add(36));
                    { let p = Ptr::raw(v27.wrapping_add(52)); let t = ((((f_1dfbb0(cx) as f64) * (cx.f(cx.lptr(88)) as f64)) + (cx.f(Ptr::raw(v27.wrapping_add(52))) as f64)) as f32); cx.set_f(p, t); }
                    v28 = cx.i(cx.ptr_at(cx.lpp(3).add(100)).add(36));
                    { let p = Ptr::raw(v28.wrapping_add(56)); let t = ((((f_1dfbb0(cx) as f64) * (cx.f(cx.lptr(88)) as f64)) + (cx.f(Ptr::raw(v28.wrapping_add(56))) as f64)) as f32); cx.set_f(p, t); }
                }
                { let p = cx.lpp(3).add(148); let t = cx.i(a3.add(148)); cx.set_i(p, t); }
                if (cx.i(cx.lpp(4).add(32)) >= 15) {
                    if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
                        let _ = crate::ov_zero_i!(cx, "pSphere->ID<PHYS_OBJ_INVALID_INDEX", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjectManager.cpp", 573);
                        { let t = v29; cx.set_li(8, t); }
                        if (v29 == 3) {
                        }
                        { let t = 257; let o = cx.li(0); cx.set_li(0, (o & !(65535 << 0)) | ((t & 65535) << 0)); }
                    } else {
                        { let t = 1; let o = cx.li(0); cx.set_li(0, (o & !(255 << 0)) | ((t & 255) << 0)); }
                    }
                }
                { let p = cx.lpp(3).add(152); let t = (((cx.i(cx.lpp(4).add(32)) & 15) << 8) | (cx.i(cx.lpp(3).add(152)) & -3841)); cx.set_i(p, t); }
                { let p = cx.lpp(3).add(152); let t = cx.i(p).bitor_(4); cx.set_i(p, t); }
                v30 = ({ let q0_ = cx.ptr_at(cx.lpp(3).add(148)).add(16); ov_vec4_w(cx, q0_) } as f64);
                { let p = cx.lpp(3).add(140); let t = (v30 as f32); cx.set_f(p, t); }
                { let p = cx.lpp(3).add(140); let t = (((cx.f(cx.lpp(3).add(140)) as f64) + 1.0f64) as f32); cx.set_f(p, t); }
                { let p = cx.lpp(3).add(140); let t = (((cx.f(cx.lpp(3).add(140)) as f64) * (cx.f(Ptr::glob(0x75BC70)) as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.lpp(3).add(140); let t = ((1.0f64 / (cx.f(cx.lpp(3).add(140)) as f64)) as f32); cx.set_f(p, t); }
                { let q0_ = cx.lpp(3).add(108); let q1_ = 0; let q2_ = 28; ov_memset(cx, q0_, q1_, q2_) };
                v31 = { let q0_ = cx.lpp(3); crate::ov_ident!(cx, q0_) };
                { let p = cx.lpp(3).add(108); let t = v31; cx.set_ptr(p, t); }
                cx.memcpy(cx.lpp(3).add(64), a3.add(64), 32);
                { let p = cx.lpp(3).add(120); let t = cx.lpp(3).add(64); cx.set_ptr(p, t); }
                { let q0_ = cx.ptr_at(cx.lpp(3).add(120)).add(24); let q1_ = 0; crate::ov_ignore!(cx, q0_, q1_) };
                if (cx.preinc_i(cx.lpp(28).add(2688)) == 30) {
                    { let p = cx.lpp(28).add(2688); let t = 0; cx.set_i(p, t); }
                }
            }
            let _ = cx.preinc_i(cx.lptr(20));
        }
        let _ = f_1e5670(cx, a3);
        return 1;
    } else {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            let _ = crate::ov_zero_i!(cx, "0", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjectManager.cpp", 271);
            { let t = v3; cx.set_li(92, t); }
            if (v3 == 3) {
            }
            { let t = 257; let o = cx.li(4); cx.set_li(4, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
        } else {
            { let t = 1; let o = cx.li(4); cx.set_li(4, (o & !(255 << 16)) | ((t & 255) << 16)); }
        }
        return 0;
    }
    #[allow(unreachable_code)]
    0
}

/// PhysicsObjectManager__1DFA50 0x001DFA50
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1dfa50(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = (((cx.f(a2) as f64) + (cx.f(a3) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a2.add(4)) as f64) + (cx.f(a3.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a2.add(8)) as f64) + (cx.f(a3.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// PhysicsObjectManager__1DFAA0 0x001DFAA0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1dfaa0(cx: &mut Cx, mut a1: Ptr, a2: f32) -> Ptr {
    { let p = a1; let t = (((cx.f(a1) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a1.add(4)) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a1.add(8)) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// PhysicsObjectManager__1DFAF0 0x001DFAF0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1dfaf0(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: f32) -> Ptr {
    { let p = a1; let t = ((((cx.f(a2) * a3) as f64) + (cx.f(a1) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((cx.f(a2.add(4)) * a3) as f64) + (cx.f(a1.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((cx.f(a2.add(8)) * a3) as f64) + (cx.f(a1.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// PhysicsObjectManager__1DFB50 0x001DFB50
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1dfb50(cx: &mut Cx, a1: Ptr) -> f32 {
    let mut v3 = 0.0f32;
    v3 = (f_1dfb80(cx, a1, a1) as f32);
    return (vsmath::sqrt(v3) as f32);
}

/// PhysicsObjectManager__1DFB80 0x001DFB80
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1dfb80(cx: &mut Cx, a1: Ptr, a2: Ptr) -> f64 {
    return ((((cx.f(a1) as f64) * (cx.f(a2) as f64)) + ((cx.f(a1.add(4)) as f64) * (cx.f(a2.add(4)) as f64))) + ((cx.f(a1.add(8)) as f64) * (cx.f(a2.add(8)) as f64)));
}

/// PhysicsObjectManager__1DFBB0 0x001DFBB0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1dfbb0(cx: &mut Cx) -> f32 {
    let mut v3 = 0i32;
    v3 = ov_rand(cx);
    return ((((((v3 as f64) - 16383.5f64) + (v3 as f64)) - 16383.5f64) / 32767.0f64) as f32);
}

/// PhysicsObjectManager__1DFF60 0x001DFF60
pub fn f_1dff60(cx: &mut Cx, this: Ptr, a3: Ptr) -> i64 {
    cx.push_frame(120);
    let r = f_1dff60_body(cx, this, a3);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1dff60_body(cx: &mut Cx, this: Ptr, a3: Ptr) -> i64 {
    let mut v2 = 0i64;
    let mut v3 = 0i32;
    let mut v4 = 0i32;
    let mut v5 = 0i32;
    let mut v6 = 0i32;
    let mut v7 = 0i32;
    let mut v8 = 0.0f64;
    let mut v9 = Ptr::NULL;
    let mut v11 = 0i64;
    let mut bb: u32 = 0;
    #[allow(unreachable_code)]
    'cfg: loop {
        match bb {
            0 => {
                { let t = this; cx.set_lpp(29, t); }
                crate::ov_ignore!(cx, 0 /*str*/);
                cx.memset(cx.lptr(104), 0, 12);
                { let t = cx.i(a3); cx.set_li(20, t); }
                match cx.li(20) { 0 => { bb = 2; continue 'cfg; } 4 => { bb = 3; continue 'cfg; } 5 => { bb = 4; continue 'cfg; } 1 => { bb = 5; continue 'cfg; } 3 => { bb = 6; continue 'cfg; } 2 => { bb = 7; continue 'cfg; } _ => { bb = 8; continue 'cfg; } }
            }
            1 => {
                break 'cfg;
            }
            2 => {
                { bb = 3; continue 'cfg; }
                break 'cfg;
            }
            3 => {
                { bb = 4; continue 'cfg; }
                break 'cfg;
            }
            4 => {
                { let t = cx.ptr_at(cx.ptr_at(a3.add(4))); cx.set_lpp(28, t); }
                { bb = 9; continue 'cfg; }
                break 'cfg;
            }
            5 => {
                { bb = 6; continue 'cfg; }
                break 'cfg;
            }
            6 => {
                { let t = cx.ptr_at(cx.ptr_at(a3.add(4))); cx.set_lpp(28, t); }
                { let t = cx.ptr_at(a3.add(8)); cx.set_lpp(26, t); }
                if (cx.i(a3.add(12)) != 0) { bb = 11; continue 'cfg; } else { bb = 12; continue 'cfg; }
                break 'cfg;
            }
            7 => {
                { let t = cx.ptr_at(a3.add(8)); cx.set_lpp(26, t); }
                { bb = 9; continue 'cfg; }
                break 'cfg;
            }
            8 => {
                v2 = (v2 & !0xFFFF_FFFFi64) | ((0) as i32 as u32 as i64);
                { bb = 121; continue 'cfg; }
                break 'cfg;
            }
            9 => {
                if (cx.lpp(28).bits() == 0) { bb = 15; continue 'cfg; } else { bb = 16; continue 'cfg; }
                break 'cfg;
            }
            10 => {
                { bb = 5; continue 'cfg; }
                break 'cfg;
            }
            11 => {
                { let t = cx.i(cx.ptr_at(a3.add(12))); cx.set_li(108, t); }
                { bb = 13; continue 'cfg; }
                break 'cfg;
            }
            12 => {
                { bb = 13; continue 'cfg; }
                break 'cfg;
            }
            13 => {
                { bb = 9; continue 'cfg; }
                break 'cfg;
            }
            14 => {
                { bb = 7; continue 'cfg; }
                break 'cfg;
            }
            15 => {
                { bb = 18; continue 'cfg; }
                break 'cfg;
            }
            16 => {
                { bb = 17; continue 'cfg; }
                break 'cfg;
            }
            17 => {
                if ((cx.i(cx.ptr_at(a3.add(4)).add(4)) >> 8) != 0) { bb = 20; continue 'cfg; } else { bb = 21; continue 'cfg; }
                break 'cfg;
            }
            18 => {
                if (cx.lpp(26).bits() != 0) { bb = 26; continue 'cfg; } else { bb = 27; continue 'cfg; }
                break 'cfg;
            }
            19 => {
                { bb = 17; continue 'cfg; }
                break 'cfg;
            }
            20 => {
                if (cx.i(cx.lpp(28)) == 1481459250) { bb = 23; continue 'cfg; } else { bb = 24; continue 'cfg; }
                break 'cfg;
            }
            21 => {
                crate::ov_ignore!(cx);
                crate::ov_ignore!(cx, 1, "Visual model resource missing from compartment\n");
                v2 = (v2 & !0xFFFF_FFFFi64) | ((0) as i32 as u32 as i64);
                { bb = 22; continue 'cfg; }
                break 'cfg;
            }
            22 => {
                { bb = 121; continue 'cfg; }
                break 'cfg;
            }
            23 => {
                { bb = 18; continue 'cfg; }
                break 'cfg;
            }
            24 => {
                crate::ov_ignore!(cx);
                crate::ov_ignore!(cx, 1, "Visual model resource contains bad data\n");
                v2 = (v2 & !0xFFFF_FFFFi64) | ((0) as i32 as u32 as i64);
                { bb = 25; continue 'cfg; }
                break 'cfg;
            }
            25 => {
                { bb = 22; continue 'cfg; }
                break 'cfg;
            }
            26 => {
                if ((cx.i(cx.ptr_at(a3.add(8)).add(4)) >> 8) == 0) { bb = 29; continue 'cfg; } else { bb = 30; continue 'cfg; }
                break 'cfg;
            }
            27 => {
                { bb = 28; continue 'cfg; }
                break 'cfg;
            }
            28 => {
                if (cx.i(cx.lpp(29).add((4_i32).wrapping_mul(cx.i(cx.lpp(29).add(3216))))) >= cx.i(cx.lpp(29).add((4_i32).wrapping_mul(cx.i(cx.lpp(29).add(3216)))).add(512))) { bb = 35; continue 'cfg; } else { bb = 36; continue 'cfg; }
                break 'cfg;
            }
            29 => {
                crate::ov_ignore!(cx, 1, "Physical model resource missing from compartment\n");
                { bb = 31; continue 'cfg; }
                break 'cfg;
            }
            30 => {
                { bb = 31; continue 'cfg; }
                break 'cfg;
            }
            31 => {
                { let t = cx.i(cx.lpp(26)); cx.set_li(100, t); }
                if (cx.li(100) == 0) { bb = 32; continue 'cfg; } else { bb = 33; continue 'cfg; }
                break 'cfg;
            }
            32 => {
                crate::ov_ignore!(cx, 1, "Physical model resource has NULL entry in resource table\n");
                { bb = 34; continue 'cfg; }
                break 'cfg;
            }
            33 => {
                { bb = 34; continue 'cfg; }
                break 'cfg;
            }
            34 => {
                { bb = 28; continue 'cfg; }
                break 'cfg;
            }
            35 => {
                if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) { bb = 38; continue 'cfg; } else { bb = 39; continue 'cfg; }
                break 'cfg;
            }
            36 => {
                { bb = 37; continue 'cfg; }
                break 'cfg;
            }
            37 => {
                { let t = cx.i(cx.lpp(29).add((4_i32).wrapping_mul(cx.i(cx.lpp(29).add(3216)))).add(1152)).wrapping_add((160_i32).wrapping_mul(cx.i(cx.lpp(29).add((4_i32).wrapping_mul(cx.i(cx.lpp(29).add(3216))))))); cx.set_lpp(23, Ptr::raw(t)); }
                { let t = cx.i(a3); cx.set_li(12, t); }
                match cx.li(12) { 1 => { bb = 45; continue 'cfg; } 3 => { bb = 46; continue 'cfg; } 5 => { bb = 47; continue 'cfg; } _ => { bb = 48; continue 'cfg; } }
            }
            38 => {
                let _ = crate::ov_zero_i!(cx, "m_PreAllocatedCounter[m_CurrentList]<m_PreAllocatedSize[m_CurrentList]", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjectManager.cpp", 744);
                { let t = v3; cx.set_li(96, t); }
                if (v3 == 3) { bb = 41; continue 'cfg; } else { bb = 42; continue 'cfg; }
                break 'cfg;
            }
            39 => {
                { let t = 1; let o = cx.li(16); cx.set_li(16, (o & !(255 << 16)) | ((t & 255) << 16)); }
                { bb = 40; continue 'cfg; }
                break 'cfg;
            }
            40 => {
                { bb = 37; continue 'cfg; }
                break 'cfg;
            }
            41 => {
                { bb = 43; continue 'cfg; }
                break 'cfg;
            }
            42 => {
                { bb = 43; continue 'cfg; }
                break 'cfg;
            }
            43 => {
                { let t = 257; let o = cx.li(16); cx.set_li(16, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
                { bb = 40; continue 'cfg; }
                break 'cfg;
            }
            44 => {
                let _ = cx.preinc_i(cx.lpp(29).add((4_i32).wrapping_mul(cx.i(cx.lpp(29).add(3216)))));
                if (cx.i(a3) != cx.i(cx.lpp(29).add((4_i32).wrapping_mul(cx.i(cx.lpp(29).add(3216)))).add(2704))) { bb = 95; continue 'cfg; } else { bb = 96; continue 'cfg; }
                break 'cfg;
            }
            45 => {
                if (cx.i(cx.ptr_at(a3.add(8))) != 0) { bb = 49; continue 'cfg; } else { bb = 50; continue 'cfg; }
                break 'cfg;
            }
            46 => {
                { let t = 0; cx.set_lpp(22, Ptr::raw(t)); }
                { let t = 0; cx.set_lpp(21, Ptr::raw(t)); }
                if (cx.i(cx.ptr_at(a3.add(8))) != 0) { bb = 62; continue 'cfg; } else { bb = 63; continue 'cfg; }
                break 'cfg;
            }
            47 => {
                crate::ov_ignore!(cx, 2, "adding simple animated object\n");
                { bb = 44; continue 'cfg; }
                break 'cfg;
            }
            48 => {
                { bb = 44; continue 'cfg; }
                break 'cfg;
            }
            49 => {
                if (cx.i(cx.lpp(29).add((4_i32).wrapping_mul(cx.i(cx.lpp(29).add(3216)))).add(1664)) == 0) { bb = 52; continue 'cfg; } else { bb = 53; continue 'cfg; }
                break 'cfg;
            }
            50 => {
                { bb = 51; continue 'cfg; }
                break 'cfg;
            }
            51 => {
                { bb = 44; continue 'cfg; }
                break 'cfg;
            }
            52 => {
                if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) { bb = 55; continue 'cfg; } else { bb = 56; continue 'cfg; }
                break 'cfg;
            }
            53 => {
                { bb = 54; continue 'cfg; }
                break 'cfg;
            }
            54 => {
                { let p = cx.lpp(23).add(100); let t = cx.i(cx.lpp(29).add((4_i32).wrapping_mul(cx.i(cx.lpp(29).add(3216)))).add(1664)).wrapping_add((40_i32).wrapping_mul(cx.i(cx.lpp(29).add((4_i32).wrapping_mul(cx.i(cx.lpp(29).add(3216))))))); cx.set_i(p, t); }
                { let q0_ = cx.lpp(23); let q1_ = a3; f_1e3cb0(cx, q0_, q1_) };
                { let p = cx.lpp(23).add(152); let t = ((16_i32).wrapping_mul((((cx.u16(a3.add(56)) & 2) != 0) as i32)) | (cx.i(cx.lpp(23).add(152)) & -17)); cx.set_i(p, t); }
                { let p = cx.lpp(23).add(152); let t = (((((cx.u16(a3.add(56)) & 4) != 0) as i32) << 13) | (cx.i(cx.lpp(23).add(152)) & -8193)); cx.set_i(p, t); }
                { bb = 51; continue 'cfg; }
                break 'cfg;
            }
            55 => {
                let _ = crate::ov_zero_i!(cx, "m_pCollisionArrays[m_CurrentList]!=NULL", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjectManager.cpp", 823);
                { let t = v6; cx.set_li(52, t); }
                if (v6 == 3) { bb = 58; continue 'cfg; } else { bb = 59; continue 'cfg; }
                break 'cfg;
            }
            56 => {
                { let t = 1; let o = cx.li(8); cx.set_li(8, (o & !(255 << 0)) | ((t & 255) << 0)); }
                { bb = 57; continue 'cfg; }
                break 'cfg;
            }
            57 => {
                { bb = 54; continue 'cfg; }
                break 'cfg;
            }
            58 => {
                { bb = 60; continue 'cfg; }
                break 'cfg;
            }
            59 => {
                { bb = 60; continue 'cfg; }
                break 'cfg;
            }
            60 => {
                { let t = 257; let o = cx.li(8); cx.set_li(8, (o & !(65535 << 0)) | ((t & 65535) << 0)); }
                { bb = 57; continue 'cfg; }
                break 'cfg;
            }
            61 => {
                { bb = 46; continue 'cfg; }
                break 'cfg;
            }
            62 => {
                if (cx.i(cx.ptr_at(cx.ptr_at(a3.add(8)))) == 1178816257) { bb = 65; continue 'cfg; } else { bb = 66; continue 'cfg; }
                break 'cfg;
            }
            63 => {
                { bb = 64; continue 'cfg; }
                break 'cfg;
            }
            64 => {
                if ((cx.lpp(21).bits() != 0) || (cx.lpp(22).bits() != 0)) { bb = 89; continue 'cfg; } else { bb = 90; continue 'cfg; }
                break 'cfg;
            }
            65 => {
                { let t = cx.i(cx.ptr_at(a3.add(8))).wrapping_add(16); cx.set_lpp(22, Ptr::raw(t)); }
                if (cx.i(cx.lpp(22).add(60)) < 128) { bb = 68; continue 'cfg; } else { bb = 69; continue 'cfg; }
                break 'cfg;
            }
            66 => {
                if (cx.i(cx.ptr_at(cx.ptr_at(a3.add(8)))) != 1112687430) { bb = 71; continue 'cfg; } else { bb = 72; continue 'cfg; }
                break 'cfg;
            }
            67 => {
                { bb = 64; continue 'cfg; }
                break 'cfg;
            }
            68 => {
                { let p = cx.lpp(22).add(60); let t = cx.i(p).wrapping_add(cx.i(cx.ptr_at(a3.add(8)))); cx.set_i(p, t); }
                { let p = cx.lpp(22).add(64); let t = cx.i(p).wrapping_add(cx.i(cx.ptr_at(a3.add(8)))); cx.set_i(p, t); }
                { let p = cx.lpp(22).add(68); let t = cx.i(p).wrapping_add(cx.i(cx.ptr_at(a3.add(8)))); cx.set_i(p, t); }
                { bb = 70; continue 'cfg; }
                break 'cfg;
            }
            69 => {
                { bb = 70; continue 'cfg; }
                break 'cfg;
            }
            70 => {
                { bb = 67; continue 'cfg; }
                break 'cfg;
            }
            71 => {
                { let t = cx.ptr_at(cx.ptr_at(a3.add(8))); cx.set_lpp(21, t); }
                { let p = cx.lpp(21).add(16); let t = cx.lpp(21).add(32); cx.set_ptr(p, t); }
                if (cx.i(cx.lpp(21).add(12)) != 0) { bb = 74; continue 'cfg; } else { bb = 75; continue 'cfg; }
                break 'cfg;
            }
            72 => {
                { bb = 73; continue 'cfg; }
                break 'cfg;
            }
            73 => {
                { bb = 67; continue 'cfg; }
                break 'cfg;
            }
            74 => {
                { let p = cx.lpp(21).add(24); let t = cx.i(cx.lpp(21).add(16)).wrapping_add((12_i32).wrapping_mul(cx.i(cx.lpp(21)))); cx.set_i(p, t); }
                { bb = 76; continue 'cfg; }
                break 'cfg;
            }
            75 => {
                { let p = cx.lpp(21).add(20); let t = cx.i(cx.lpp(21).add(16)).wrapping_add((12_i32).wrapping_mul(cx.i(cx.lpp(21)))); cx.set_i(p, t); }
                { bb = 76; continue 'cfg; }
                break 'cfg;
            }
            76 => {
                if (cx.i(cx.lpp(21).add(28)) != 0) { bb = 77; continue 'cfg; } else { bb = 78; continue 'cfg; }
                break 'cfg;
            }
            77 => {
                if (cx.i(cx.lpp(21).add(12)) != 0) { bb = 80; continue 'cfg; } else { bb = 81; continue 'cfg; }
                break 'cfg;
            }
            78 => {
                if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) { bb = 83; continue 'cfg; } else { bb = 84; continue 'cfg; }
                break 'cfg;
            }
            79 => {
                { bb = 73; continue 'cfg; }
                break 'cfg;
            }
            80 => {
                v5 = cx.i(cx.lpp(21).add(24)).wrapping_add((8_i32).wrapping_mul(cx.i(cx.lpp(21).add(4))));
                { bb = 82; continue 'cfg; }
                break 'cfg;
            }
            81 => {
                v5 = cx.i(cx.lpp(21).add(20)).wrapping_add((16_i32).wrapping_mul(cx.i(cx.lpp(21).add(4))));
                { bb = 82; continue 'cfg; }
                break 'cfg;
            }
            82 => {
                { let p = cx.lpp(21).add(28); let t = v5; cx.set_i(p, t); }
                { bb = 79; continue 'cfg; }
                break 'cfg;
            }
            83 => {
                let _ = crate::ov_zero_i!(cx, "0", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjectManager.cpp", 792);
                { let t = v4; cx.set_li(80, t); }
                if (v4 == 3) { bb = 86; continue 'cfg; } else { bb = 87; continue 'cfg; }
                break 'cfg;
            }
            84 => {
                { let t = 1; let o = cx.li(8); cx.set_li(8, (o & !(255 << 16)) | ((t & 255) << 16)); }
                { bb = 85; continue 'cfg; }
                break 'cfg;
            }
            85 => {
                { bb = 79; continue 'cfg; }
                break 'cfg;
            }
            86 => {
                { bb = 88; continue 'cfg; }
                break 'cfg;
            }
            87 => {
                { bb = 88; continue 'cfg; }
                break 'cfg;
            }
            88 => {
                { let t = 257; let o = cx.li(8); cx.set_li(8, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
                { bb = 85; continue 'cfg; }
                break 'cfg;
            }
            89 => {
                { let q0_ = cx.lptr(60); let q1_ = a3.add(16); ov_vec4_copy(cx, q0_, q1_) };
                { let p = cx.lpp(23).add(104); let t = cx.i(cx.lpp(29).add((4_i32).wrapping_mul(cx.i(cx.lpp(29).add(3216)))).add(2176)).wrapping_add((24_i32).wrapping_mul(cx.i(cx.lpp(29).add((4_i32).wrapping_mul(cx.i(cx.lpp(29).add(3216))))))); cx.set_i(p, t); }
                { let q0_ = cx.ptr_at(cx.lpp(23).add(104)); let q1_ = cx.lpp(21).bits(); let q2_ = cx.lptr(60); let q3_ = cx.lpp(22).bits(); let q4_ = cx.lpp(23); f_1e32b0(cx, q0_, q1_, q2_, q3_, q4_) };
                { bb = 91; continue 'cfg; }
                break 'cfg;
            }
            90 => {
                { let p = cx.lpp(23).add(104); let t = 0; cx.set_i(p, t); }
                { bb = 91; continue 'cfg; }
                break 'cfg;
            }
            91 => {
                { bb = 44; continue 'cfg; }
                break 'cfg;
            }
            92 => {
                { bb = 47; continue 'cfg; }
                break 'cfg;
            }
            93 => {
                { bb = 48; continue 'cfg; }
                break 'cfg;
            }
            94 => {
                { bb = 44; continue 'cfg; }
                break 'cfg;
            }
            95 => {
                if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) { bb = 98; continue 'cfg; } else { bb = 99; continue 'cfg; }
                break 'cfg;
            }
            96 => {
                { bb = 97; continue 'cfg; }
                break 'cfg;
            }
            97 => {
                { let t = cx.i(a3); cx.set_li(0, t); }
                match cx.li(0) { 0 => { bb = 105; continue 'cfg; } 1 => { bb = 106; continue 'cfg; } 3 => { bb = 107; continue 'cfg; } 4 => { bb = 108; continue 'cfg; } 5 => { bb = 109; continue 'cfg; } 2 => { bb = 110; continue 'cfg; } _ => { bb = 111; continue 'cfg; } }
            }
            98 => {
                let _ = crate::ov_zero_i!(cx, "_params.eType==m_PhysicsListType[m_CurrentList]", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjectManager.cpp", 847);
                { let t = v7; cx.set_li(48, t); }
                if (v7 == 3) { bb = 101; continue 'cfg; } else { bb = 102; continue 'cfg; }
                break 'cfg;
            }
            99 => {
                { let t = 1; let o = cx.li(4); cx.set_li(4, (o & !(255 << 16)) | ((t & 255) << 16)); }
                { bb = 100; continue 'cfg; }
                break 'cfg;
            }
            100 => {
                { bb = 97; continue 'cfg; }
                break 'cfg;
            }
            101 => {
                { bb = 103; continue 'cfg; }
                break 'cfg;
            }
            102 => {
                { bb = 103; continue 'cfg; }
                break 'cfg;
            }
            103 => {
                { let t = 257; let o = cx.li(4); cx.set_li(4, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
                { bb = 100; continue 'cfg; }
                break 'cfg;
            }
            104 => {
                crate::ov_ignore!(cx);
                v2 = (v2 & !0xFFFF_FFFFi64) | ((cx.lpp(23).bits()) as i32 as u32 as i64);
                { bb = 25; continue 'cfg; }
                break 'cfg;
            }
            105 => {
                { bb = 106; continue 'cfg; }
                break 'cfg;
            }
            106 => {
                { bb = 107; continue 'cfg; }
                break 'cfg;
            }
            107 => {
                { bb = 108; continue 'cfg; }
                break 'cfg;
            }
            108 => {
                { bb = 109; continue 'cfg; }
                break 'cfg;
            }
            109 => {
                { let q0_ = cx.lptr(28); let q1_ = a3.add(16); ov_vec4_copy(cx, q0_, q1_) };
                { let q0_ = cx.lptr(28); let q1_ = 1065353216; ov_vec4_setw(cx, q0_, q1_) };
                { let q0_ = cx.lpp(23); let q1_ = cx.lptr(28); let q2_ = a3.add(32); f_1e5740(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lpp(23); crate::ov_ignore!(cx, q0_) };
                { let p = cx.lpp(23).add(144); let t = cx.lpp(28); cx.set_ptr(p, t); }
                { let p = cx.lpp(23).add(148); let t = cx.li(108); cx.set_i(p, t); }
                if (cx.i(cx.lpp(23).add(144)) != 0) { bb = 112; continue 'cfg; } else { bb = 113; continue 'cfg; }
                break 'cfg;
            }
            110 => {
                { let p = cx.lpp(23).add(144); let t = 0; cx.set_i(p, t); }
                { bb = 104; continue 'cfg; }
                break 'cfg;
            }
            111 => {
                { bb = 104; continue 'cfg; }
                break 'cfg;
            }
            112 => {
                v8 = ({ let q0_ = cx.ptr_at(cx.lpp(23).add(144)).add(16); ov_vec4_w(cx, q0_) } as f64);
                { let p = cx.lpp(23).add(140); let t = (v8 as f32); cx.set_f(p, t); }
                { let p = cx.lpp(23).add(140); let t = (((cx.f(cx.lpp(23).add(140)) as f64) + 1.0f64) as f32); cx.set_f(p, t); }
                { let p = cx.lpp(23).add(140); let t = (((cx.f(cx.lpp(23).add(140)) as f64) * (cx.f(Ptr::glob(0x75BC70)) as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.lpp(23).add(140); let t = ((1.0f64 / (cx.f(cx.lpp(23).add(140)) as f64)) as f32); cx.set_f(p, t); }
                { bb = 114; continue 'cfg; }
                break 'cfg;
            }
            113 => {
                { let p = cx.lpp(23).add(140); let t = 0; cx.set_i(p, t); }
                { bb = 114; continue 'cfg; }
                break 'cfg;
            }
            114 => {
                { let q0_ = cx.lpp(23).add(108); let q1_ = 0; let q2_ = 28; ov_memset(cx, q0_, q1_, q2_) };
                v9 = { let q0_ = cx.lpp(23); crate::ov_ident!(cx, q0_) };
                { let p = cx.lpp(23).add(108); let t = v9; cx.set_ptr(p, t); }
                { let p = cx.lpp(23).add(120); let t = cx.lpp(23).add(64); cx.set_ptr(p, t); }
                { let q0_ = cx.ptr_at(cx.lpp(23).add(120)).add(24); let q1_ = 0; crate::ov_ignore!(cx, q0_, q1_) };
                if (cx.i(a3) == 4) { bb = 115; continue 'cfg; } else { bb = 116; continue 'cfg; }
                break 'cfg;
            }
            115 => {
                { let q0_ = cx.i(cx.lpp(23).add(144)); crate::ov_ignore!(cx, q0_) };
                { bb = 117; continue 'cfg; }
                break 'cfg;
            }
            116 => {
                { bb = 117; continue 'cfg; }
                break 'cfg;
            }
            117 => {
                { let p = cx.lpp(23).add(152); let t = ((2_i32).wrapping_mul((((cx.u16(a3.add(56)) & 1) == 0) as i32)) | (cx.i(cx.lpp(23).add(152)) & -3)); cx.set_i(p, t); }
                { bb = 104; continue 'cfg; }
                break 'cfg;
            }
            118 => {
                { bb = 110; continue 'cfg; }
                break 'cfg;
            }
            119 => {
                { bb = 111; continue 'cfg; }
                break 'cfg;
            }
            120 => {
                { bb = 104; continue 'cfg; }
                break 'cfg;
            }
            121 => {
                v11 = v2;
                return v11;
            }
            122 => {
                { bb = 8; continue 'cfg; }
                break 'cfg;
            }
            123 => {
                { bb = 1; continue 'cfg; }
                break 'cfg;
            }
            124 => {
                break 'cfg;
            }
            _ => unreachable!(),
        }
    }
    #[allow(unreachable_code)]
    0
}

/// PhysicsObjectManager__1E18E0 0x001E18E0
pub fn f_1e18e0(cx: &mut Cx, this: Ptr, a2: Ptr, a3: i32, a4: i32, a5: f32, a6: i32, a7: i32) -> i64 {
    cx.push_frame(92);
    let r = f_1e18e0_body(cx, this, a2, a3, a4, a5, a6, a7);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1e18e0_body(cx: &mut Cx, this: Ptr, a2: Ptr, a3: i32, a4: i32, a5: f32, a6: i32, a7: i32) -> i64 {
    let mut v7 = Ptr::NULL;
    let mut v8 = Ptr::NULL;
    let mut v9 = Ptr::NULL;
    let mut v10 = 0i64;
    let mut v12 = 0i64;
    { let t = this; cx.set_lpp(22, t); }
    v7 = ov_up_vector(cx);
    { let q0_ = a2; let q1_ = cx.lptr(68); let q2_ = v7; ov_vec_add(cx, q0_, q1_, q2_) };
    v8 = ov_up_vector(cx);
    { let q0_ = a2; let q1_ = cx.lptr(44); let q2_ = v8; ov_vec_sub(cx, q0_, q1_, q2_) };
    { let t = 0; cx.set_li(32, t); }
    v9 = ov_physics_manager(cx);
    v10 = { let q0_ = v9; let q1_ = cx.lptr(32); let q2_ = cx.lptr(68); let q3_ = cx.lptr(44); let q4_ = 0; let q5_ = a6; f_19d310(cx, q0_, q1_, q2_, q3_, q4_, q5_) };
    { let t = (v10 as i32); let o = cx.li(24); cx.set_li(24, (o & !(255 << 24)) | ((t & 255) << 24)); }
    v10 = (v10 & !0xFFFF_FFFFi64) | ((((v10 as i32) as u8 as i32)) as i32 as u32 as i64);
    if (((v10 as i32) as u8 as i32) != 0) {
        { let q0_ = cx.lptr(4); ov_identity_p(cx, q0_) };
        { let q0_ = cx.lptr(68); let q1_ = cx.lptr(44); let q2_ = cx.f(cx.lptr(32)); let q3_ = cx.lptr(4); f_762f0(cx, q0_, q1_, q2_, q3_) };
        v10 = { let q0_ = cx.lpp(22); let q1_ = cx.lptr(4); let q2_ = a3; let q3_ = a4; let q4_ = a5; let q5_ = a7; f_1e1a20(cx, q0_, q1_, q2_, q3_, q4_, q5_) };
    }
    v10 = (v10 & !(0xFFi64 as i64)) | ((((((cx.li(24) >> 24) & 255)) as i64) & 0xFF) << 0) as i64;
    v12 = v10;
    return v12;
}

/// PhysicsObjectManager__1E1A20 0x001E1A20
pub fn f_1e1a20(cx: &mut Cx, this: Ptr, pos: Ptr, a4: i32, a5: i32, a6: f32, a7: i32) -> i64 {
    cx.push_frame(84);
    let r = f_1e1a20_body(cx, this, pos, a4, a5, a6, a7);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1e1a20_body(cx: &mut Cx, this: Ptr, pos: Ptr, a4: i32, a5: i32, a6: f32, a7: i32) -> i64 {
    let mut v6 = Ptr::NULL;
    let mut v7 = 0i64;
    let mut v9 = 0i64;
    { let t = this; cx.set_lpp(20, t); }
    { let t = ov_vec4_x(cx, pos); cx.set_lf(64, t); }
    { let t = ov_vec4_y(cx, pos); cx.set_lf(68, t); }
    { let t = ov_vec4_z(cx, pos); cx.set_lf(72, t); }
    { let q0_ = Ptr::glob(0x7814E0); let q1_ = pos; let q2_ = cx.i(Ptr::glob(0x77FD48)); let q3_ = 1.0f32; crate::ov_ignore!(cx, q0_, q1_, q2_, q3_) };
    v6 = ov_physics_manager(cx);
    { let q0_ = v6; let q1_ = cx.lptr(64); let q2_ = a4; let q3_ = a5; let q4_ = a7; f_1b3500(cx, q0_, q1_, q2_, q3_, q4_) };
    { let q0_ = cx.lptr(8); ov_identity_p(cx, q0_) };
    { let t = (vs_rand_range(cx, -1.0f32, 1.0f32) as f32); cx.set_lf(24, t); }
    { let t = (vs_rand_range(cx, -1.0f32, 1.0f32) as f32); cx.set_lf(28, t); }
    { let t = (vsmath::sqrt(((((cx.lf(24) * cx.lf(24)) as f64) + ((cx.lf(28) * cx.lf(28)) as f64)) as f32)) as f32); cx.set_lf(0, t); }
    { let t = (((cx.lf(24) as f64) / (cx.lf(0) as f64)) as f32); cx.set_lf(24, t); }
    { let t = (((cx.lf(28) as f64) / (cx.lf(0) as f64)) as f32); cx.set_lf(28, t); }
    { let t = ((15.0f64 * 2.0f64) as f32); cx.set_lf(40, t); }
    { let t = ((15.0f64 * 2.0f64) as f32); cx.set_lf(44, t); }
    { let t = 2; cx.set_li(36, t); }
    { let q0_ = cx.lptr(8); let q1_ = pos; ov_vec4_copy(cx, q0_, q1_) };
    { let q0_ = cx.lptr(8); let q1_ = 1097859072; ov_vec4_setw(cx, q0_, q1_) };
    { let t = a6; cx.set_lf(32, t); }
    { let q0_ = cx.lptr(8); let q1_ = 1; f_1e1f20(cx, q0_, q1_) };
    { let t = 480; cx.set_li(48, t); }
    v7 = (v7 & !0xFFFF_FFFFi64) | (({ { let q0_ = Ptr::glob(0x720840); let q1_ = cx.lptr(8); ov_puddle_add(cx, q0_, q1_) }; 0 }) as i32 as u32 as i64);
    v9 = v7;
    return v9;
}

/// PhysicsObjectManager__1E1DB0 0x001E1DB0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e1db0(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = ov_identity_p(cx, this);
    return this;
}

/// PhysicsObjectManager__1E1DE0 0x001E1DE0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e1de0(cx: &mut Cx, this: Ptr) -> f64 {
    return ((((1.3333334f64 * 3.1415927f64) * (cx.f(this.add(16)) as f64)) * (cx.f(this.add(16)) as f64)) * (cx.f(this.add(16)) as f64));
}

/// PhysicsObjectManager__1E1E10 0x001E1E10
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e1e10(cx: &mut Cx, this: Ptr) -> f64 {
    return (((((1.3333334f64 * 3.1415927f64) * (cx.f(this.add(28)) as f64)) * (cx.f(this.add(28)) as f64)) * (cx.f(this.add(28)) as f64)) + ((((2.0f64 * (cx.f(this.add(32)) as f64)) * 3.1415927f64) * (cx.f(this.add(28)) as f64)) * (cx.f(this.add(28)) as f64)));
}

/// PhysicsObjectManager__1E1E60 0x001E1E60
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e1e60(cx: &mut Cx, this: Ptr) -> f64 {
    return (((8.0f64 * (cx.f(this.add(52)) as f64)) * (cx.f(this.add(56)) as f64)) * (cx.f(this.add(60)) as f64));
}

/// PhysicsObjectManager__1E1F20 0x001E1F20
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e1f20(cx: &mut Cx, this: Ptr, a2: i32) -> Ptr {
    { let p = this.add(44); let t = cx.i(p).bitor_(a2); cx.set_i(p, t); }
    return this;
}
