//! Translated from the decompilation by tools/phys_c2rs4.py (do not edit by hand; fixes go in the
//! translator or in the hand-written overrides).
#![allow(non_snake_case, unused_parens, unused_labels, unused_braces, unused_variables, unused_mut, unused_assignments, unreachable_code, dead_code, clippy::all)]

use crate::tr_prelude::*;

/// PhysicsManager__19D3E0 0x0019D3E0
pub fn f_19d3e0(cx: &mut Cx, a1: Ptr, a2: i32, mut a3: Ptr, a4: Ptr, a5: Ptr, a6: Ptr, a7: i32, a8: i32) -> i64 {
    cx.push_frame(156);
    let r = f_19d3e0_body(cx, a1, a2, a3, a4, a5, a6, a7, a8);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_19d3e0_body(cx: &mut Cx, a1: Ptr, a2: i32, mut a3: Ptr, a4: Ptr, a5: Ptr, a6: Ptr, a7: i32, a8: i32) -> i64 {
    let mut v8 = 0.0f32;
    let mut v9 = 0i64;
    let mut v10 = Ptr::NULL;
    let mut v11 = 0.0f64;
    let mut v12 = 0i32;
    let mut v13 = 0i32;
    let mut v14 = 0i32;
    let mut v15 = 0i32;
    let mut v16 = 0i32;
    let mut v17 = 0i32;
    let mut v18 = 0.0f32;
    let mut v20 = 0i64;
    { let t = a1; cx.set_lpp(38, t); }
    if (a4.bits() == Ptr::NULL.bits()) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            v8 = f32::from_bits(crate::ov_zero_i!(cx, "info", ".\\src\\Physics\\PhysicsManager.cpp", 667) as u32);
            { let t = v8; cx.set_lf(148, t); }
            if ((v8.to_bits() as i32) == 3) {
            }
            { let t = 257; let o = cx.li(0); cx.set_li(0, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
        } else {
            { let t = 1; let o = cx.li(0); cx.set_li(0, (o & !(255 << 16)) | ((t & 255) << 16)); }
        }
    }
    v9 = (v9 & 0xFFFF_FFFFi64) | (((cx.lpp(38).bits()) as u32 as i64) << 32);
    v9 = (v9 & !0xFFFF_FFFFi64) | ((cx.i(cx.lpp(38).add(32))) as i32 as u32 as i64);
    { let t = (v9 as i32); cx.set_li(144, t); }
    'l_LABEL_44: {
        if (cx.i(Ptr::raw(((v9 + 72i64) as i32))) != 0) {
            { let t = 1.0f32; cx.set_lf(136, t); }
            { let q0_ = cx.lptr(116); let q1_ = a6; let q2_ = a5; f_19d920(cx, q0_, q1_, q2_) };
            if (((a8 & 1) != 0) && ((({ let q0_ = cx.lptr(136); let q1_ = a4; let q2_ = a5; let q3_ = a6; let q4_ = cx.lptr(116); let q5_ = cx.lpp(36); let q6_ = cx.ptr_at(cx.lpp(38).add(28)); let q7_ = a8; f_19db00(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_) } as i32) as u8 as i32) != 0)) {
                { let p = a4.add(12); let t = 1; cx.set_i(p, t); }
            }
            if ((a8 & 2) != 0) {
                { let t = ov_barrier_qt(cx, Ptr::glob(0x705F48)); cx.set_lpp(36, t); }
                if (cx.lpp(36).bits() != 0) {
                    v10 = ov_barrier_mesh(cx, Ptr::glob(0x705F48));
                    if ((({ let q0_ = cx.lptr(136); let q1_ = a4; let q2_ = a5; let q3_ = a6; let q4_ = cx.lptr(116); let q5_ = cx.lpp(36); let q6_ = v10; let q7_ = a8; f_19db00(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_) } as i32) as u8 as i32) != 0) {
                        { let p = a4.add(12); let t = 2; cx.set_i(p, t); }
                    }
                }
            }
            v9 = (v9 & !0xFFFF_FFFFi64) | (((a8 & 4)) as i32 as u32 as i64);
            if ((a8 & 4) == 0) {
                { let p = a3; let t = cx.lf(136); cx.set_f(p, t); }
                v9 = (v9 & !(0xFFi64 as i64)) | ((((((cx.lf(136) < 1.0f32) as i32)) as i64) & 0xFF) << 0) as i64;
                break 'l_LABEL_44;
            }
            v11 = ({ let q0_ = cx.lptr(116); f_19d9f0(cx, q0_) } as f64);
            { let t = (v11 as f32); cx.set_lf(88, t); }
            v16 = { let q0_ = cx.lf(88); ov_isnan(cx, q0_) };
            v12 = ((cx.lf(88) < 0.0f32) as i32);
            v13 = ((cx.lf(88) == 0.0f32) as i32);
            v14 = 0;
            v15 = 0;
            v9 = (v9 & !(0xFF00i64 as i64)) | ((((v17) as i64) & 0xFF) << 8) as i64;
            if (cx.lf(88) == 0.0f32) {
                v9 = (v9 & !(0xFFi64 as i64)) | ((((0) as i64) & 0xFF) << 0) as i64;
                break 'l_LABEL_44;
            }
            { let q0_ = cx.lptr(96); let q1_ = cx.lptr(116); let q2_ = ((1.0f64 / (cx.lf(88) as f64)) as f32); f_19d970(cx, q0_, q1_, q2_) };
            { let t = 0.0f32; cx.set_lf(84, t); }
            'lp2: loop {
                'blp2: {
                    v9 = (v9 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.i(cx.lpp(38)); ov_vehicle_count(cx, q0_) }) as i32 as u32 as i64);
                    if (cx.li(84) >= (v9 as i32)) {
                        break 'lp2;
                    }
                    if (a7 < -1) {
                        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
                            v18 = f32::from_bits(crate::ov_zero_i!(cx, "ignore_car_idx >= -1", ".\\src\\Physics\\PhysicsManager.cpp", 713) as u32);
                            { let t = v18; cx.set_lf(80, t); }
                            if ((v18.to_bits() as i32) == 3) {
                            }
                            { let t = 257; let o = cx.li(0); cx.set_li(0, (o & !(65535 << 0)) | ((t & 65535) << 0)); }
                        } else {
                            { let t = 1; let o = cx.li(0); cx.set_li(0, (o & !(255 << 0)) | ((t & 255) << 0)); }
                        }
                    }
                    if (a7 != -1) {
                        if ((a8 & 32) != 0) {
                            if (cx.li(84) != a7) {
                                break 'blp2;
                            }
                        } else if (cx.li(84) == a7) {
                            break 'blp2;
                        }
                    }
                    { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(38)); let q1_ = v11; let q2_ = cx.li(84); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }; cx.set_lpp(19, t); }
                    if (cx.b(cx.lpp(19).add(28)) != 0) {
                        { let _ = { let q0_ = cx.lpp(19).add(240); let q1_ = cx.lptr(60); vcall(cx, 28, q0_, &[Varg::from(q1_)]) }; };
                        { let q0_ = cx.lptr(40); let q1_ = a5; let q2_ = cx.lptr(60); f_19d920(cx, q0_, q1_, q2_) };
                        v11 = { let q0_ = cx.lptr(40); let q1_ = cx.lptr(96); f_19d9c0(cx, q0_, q1_) };
                        { let t = (v11 as f32); cx.set_lf(32, t); }
                        { let t = cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(19).add(3556))).add(12)); cx.set_lf(28, t); }
                        if (cx.i(cx.ptr_at(cx.lpp(19).add(3556)).add(4)) >= 2) {
                            { let t = (((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(19).add(3556))).add(12)) as f64) + (cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(19).add(3556))).add(188)) as f64)) as f32); cx.set_lf(28, t); }
                        }
                        if (((0.0f32 - cx.lf(28)) <= (0.0f32 - cx.lf(32))) && (((0.0f32 - cx.lf(32)) - cx.lf(88)) <= cx.lf(28))) {
                            { let t = cx.ptr_at(cx.ptr_at(cx.lpp(19).add(3556))); cx.set_lpp(6, t); }
                            { let t = 0.0f32; cx.set_lf(20, t); }
                            if (cx.i(cx.ptr_at(cx.lpp(19).add(3556)).add(4)) >= 2) {
                                { let t = ((((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(19).add(3556))).add(252)) - cx.f(cx.lpp(6).add(56))) as f64) + (cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(19).add(3556))).add(188)) as f64)) as f32); cx.set_lf(20, t); }
                            }
                            { let t = cx.f(cx.lpp(6).add(56)); cx.set_lf(16, t); }
                            { let p = cx.lpp(6).add(56); let t = (((cx.f(cx.lpp(6).add(56)) as f64) + (cx.lf(20) as f64)) as f32); cx.set_f(p, t); }
                            if (((({ let q0_ = cx.lptr(8); let q1_ = a5; let q2_ = cx.lptr(116); let q3_ = cx.lpp(6); f_383ad0(cx, q0_, q1_, q2_, q3_) } as i32) as u8 as i32) != 0) && (cx.lf(136) > cx.lf(8))) {
                                { let t = cx.lf(8); cx.set_lf(136, t); }
                                { let p = a4.add(12); let t = 3; cx.set_i(p, t); }
                                if ((a8 & 16) != 0) {
                                    { let q0_ = a4; let q1_ = cx.lptr(116); f_19d8d0(cx, q0_, q1_) };
                                    let _ = f_19da20(cx, a4);
                                }
                            }
                            { let p = cx.lpp(6).add(56); let t = cx.lf(16); cx.set_f(p, t); }
                        }
                    }
                }
                let _ = cx.preinc_i(cx.lptr(84));
            }
            { let p = a3; let t = cx.lf(136); cx.set_f(p, t); }
            v9 = (v9 & !(0xFFi64 as i64)) | ((((((cx.lf(136) < 1.0f32) as i32)) as i64) & 0xFF) << 0) as i64;
            break 'l_LABEL_44;
        }
        v9 = (v9 & !(0xFFi64 as i64)) | ((((0) as i64) & 0xFF) << 0) as i64;
    }
    v20 = v9;
    return v20;
}

/// PhysicsManager__19D8D0 0x0019D8D0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_19d8d0(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = ((0.0f64 - (cx.f(a2) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((0.0f64 - (cx.f(a2.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((0.0f64 - (cx.f(a2.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// PhysicsManager__19D920 0x0019D920
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_19d920(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = (((cx.f(a2) as f64) - (cx.f(a3) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a2.add(4)) as f64) - (cx.f(a3.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a2.add(8)) as f64) - (cx.f(a3.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// PhysicsManager__19D970 0x0019D970
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_19d970(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: f32) -> Ptr {
    { let p = a1; let t = (((cx.f(a2) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a2.add(4)) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a2.add(8)) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// PhysicsManager__19D9C0 0x0019D9C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_19d9c0(cx: &mut Cx, a1: Ptr, a2: Ptr) -> f64 {
    return ((((cx.f(a1) as f64) * (cx.f(a2) as f64)) + ((cx.f(a1.add(4)) as f64) * (cx.f(a2.add(4)) as f64))) + ((cx.f(a1.add(8)) as f64) * (cx.f(a2.add(8)) as f64)));
}

/// PhysicsManager__19D9F0 0x0019D9F0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_19d9f0(cx: &mut Cx, a1: Ptr) -> f32 {
    let mut v3 = 0.0f32;
    v3 = (f_19d9c0(cx, a1, a1) as f32);
    return (vsmath::sqrt(v3) as f32);
}

/// PhysicsManager__19DA20 0x0019DA20
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_19da20(cx: &mut Cx, mut a1: Ptr) -> Ptr {
    let mut v2 = 0.0f32;
    v2 = f_19d9f0(cx, a1);
    if (v2 != 0.0f32) {
        return f_19dab0(cx, a1, ((1.0f64 / (v2 as f64)) as f32));
    }
    { let p = a1; let t = 1065353216; cx.set_i(p, t); }
    { let p = a1.add(4); let t = 0; cx.set_i(p, t); }
    { let p = a1.add(8); let t = 0; cx.set_i(p, t); }
    return a1;
}

/// PhysicsManager__19DAB0 0x0019DAB0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_19dab0(cx: &mut Cx, mut a1: Ptr, a2: f32) -> Ptr {
    { let p = a1; let t = (((cx.f(a1) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a1.add(4)) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a1.add(8)) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// PhysicsManager__19DB00 0x0019DB00
pub fn f_19db00(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr, a4: Ptr, a5: Ptr, a6: Ptr, a7: Ptr, a8: i32) -> i64 {
    cx.push_frame(484);
    let r = f_19db00_body(cx, a1, a2, a3, a4, a5, a6, a7, a8);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_19db00_body(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr, a4: Ptr, a5: Ptr, a6: Ptr, a7: Ptr, a8: i32) -> i64 {
    let mut v8 = 0i64;
    let mut v10 = 0i64;
    { let q0_ = cx.lptr(468); let q1_ = a3; f_19e030(cx, q0_, q1_) };
    { let t = 0; let o = cx.li(460); cx.set_li(460, (o & !(255 << 24)) | ((t & 255) << 24)); }
    'lp4: while (1 != 0) {
        v8 = { let q0_ = a6; let q1_ = cx.lptr(440); let q2_ = cx.lptr(452); let q3_ = cx.lptr(440); let q4_ = cx.lptr(468); let q5_ = 0; let q6_ = cx.lptr(108); ov_qt_find_leaf(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
        { let t = (v8 as i32); cx.set_li(100, t); }
        { let t = 0.0f32; cx.set_lf(96, t); }
        if (cx.li(440) != 0) {
            { let t = 0.0f32; cx.set_lf(92, t); }
            'lp5: loop {
                'blp5: {
                    v8 = (v8 & 0xFFFF_FFFFi64) | ((((cx.lf(92)).to_bits() as i32) as u32 as i64) << 32);
                    if (cx.li(92) >= cx.li(440)) {
                        break 'lp5;
                    }
                    { let t = { let q0_ = a6; let q1_ = cx.li(92).wrapping_add(cx.li(452)); ov_qt_face_index(cx, q0_, q1_) }; cx.set_li(88, t); }
                    { let t = { let q0_ = a7; let q1_ = cx.li(88); let q2_ = 0; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(21, t); }
                    { let t = { let q0_ = a7; let q1_ = cx.li(88); let q2_ = 1; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(20, t); }
                    { let t = { let q0_ = a7; let q1_ = cx.li(88); let q2_ = 2; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(19, t); }
                    v8 = (v8 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.lptr(68); let q1_ = cx.lptr(56); let q2_ = cx.lptr(44); let q3_ = cx.lpp(21); let q4_ = cx.lpp(20); let q5_ = cx.lpp(19); let q6_ = a3; let q7_ = a5; ov_ray_triangle(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_) }) as i32 as u32 as i64);
                    if ((v8 as i32) != 0) {
                        v8 = (v8 & !0xFFFF_FFFFi64) | ((a1.bits()) as i32 as u32 as i64);
                        if ((cx.f(a1) > cx.lf(68)) && (cx.lf(68) > 0.0f32)) {
                            { let p = a1; let t = cx.lf(68); cx.set_f(p, t); }
                            { let t = 1; cx.set_li(96, t); }
                            { let t = 1; let o = cx.li(460); cx.set_li(460, (o & !(255 << 24)) | ((t & 255) << 24)); }
                            v8 = (v8 & 0xFFFF_FFFFi64) | ((((a8 & 16)) as u32 as i64) << 32);
                            if ((a8 & 16) != 0) {
                                { let q0_ = a2; let q1_ = Ptr::raw(cx.i(a7.add(28)).wrapping_add((12_i32).wrapping_mul(cx.li(88)))); f_19e030(cx, q0_, q1_) };
                            }
                            v8 = (v8 & !0xFFFF_FFFFi64) | (((a8 & 8)) as i32 as u32 as i64);
                            if ((a8 & 8) != 0) {
                                break 'lp5;
                            }
                        }
                    }
                }
                let _ = cx.preinc_i(cx.lptr(92));
            }
        } else if (cx.li(452) == 0) {
            if ((cx.i(a6.add(92)) != 0) && (cx.li(428) == 0)) {
                break 'lp4;
            }
            v8 = (v8 & 0xFFFF_FFFFi64) | (((a6.bits()) as u32 as i64) << 32);
            if (cx.i(a6.add(92)) == 0) {
                break 'lp4;
            }
        }
        if (cx.li(96) != 0) {
            break 'lp4;
        }
        if (cx.i(a6.add(92)) != 0) {
            { let t = cx.lptr(0).add(4 * ((2_i32).wrapping_mul(cx.li(428)).wrapping_add(27))); cx.set_lpp(9, t); }
            { let t = cx.lptr(0).add(4 * ((2_i32).wrapping_mul(cx.li(428)).wrapping_add(67))); cx.set_lpp(8, t); }
        } else {
            { let t = cx.i(a6.add(84)).wrapping_add((40_i32).wrapping_mul(cx.li(100))).wrapping_add(4); cx.set_lpp(9, Ptr::raw(t)); }
            { let t = cx.i(a6.add(84)).wrapping_add((40_i32).wrapping_mul(cx.li(100))).wrapping_add(12); cx.set_lpp(8, Ptr::raw(t)); }
        }
        { let t = (((cx.f(a4.add((4_i32).wrapping_mul(cx.i(Ptr::glob(0x688980).add(4 * (0)))))) as f64) - (cx.f(cx.lptr(0).add(4 * (cx.i(Ptr::glob(0x688980).add(4 * (0))).wrapping_add(117)))) as f64)) as f32); cx.set_lf(20, t); }
        v8 = ({ let q0_ = a4; let q1_ = cx.i(Ptr::glob(0x688984)); crate::ov_zero_i!(cx, q0_, q1_) } as i64);
        { let t = (((cx.f(a4.add((4_i32).wrapping_mul(cx.i(Ptr::glob(0x688984))))) as f64) - (cx.f(cx.lptr(0).add(4 * (cx.i(Ptr::glob(0x688984)).wrapping_add(117)))) as f64)) as f32); cx.set_lf(24, t); }
        { let t = 1.0f32; cx.set_lf(12, t); }
        { let t = 0.0f32; cx.set_lf(8, t); }
        'lp6: loop {
            if !(cx.li(8) < 2) { break 'lp6; }
            'blp6: {
                v8 = (v8 & 0xFFFF_FFFFi64) | ((((cx.lf(8)).to_bits() as i32) as u32 as i64) << 32);
                if (cx.f(cx.lptr(0).add(4 * (cx.li(8).wrapping_add(5)))) <= 0.0f32) {
                    v8 = (v8 & !0xFFFF_FFFFi64) | ((cx.lf(8)).to_bits() as i64);
                    if (cx.f(cx.lptr(0).add(4 * (cx.li(8).wrapping_add(5)))) < 0.0f32) {
                        v8 = (v8 & 0xFFFF_FFFFi64) | (((cx.i(Ptr::glob(0x688980).add(4 * (cx.li(8))))) as u32 as i64) << 32);
                        v8 = (v8 & !0xFFFF_FFFFi64) | ((cx.lf(8)).to_bits() as i64);
                        { let t = (((((cx.f(cx.lpp(9).add((4_i32).wrapping_mul(cx.li(8)))) - cx.f(a6.add(56))) - cx.f(cx.lptr(0).add(4 * (((v8 >> 32) as i32).wrapping_add(117))))) as f64) / (cx.f(cx.lptr(0).add(4 * (cx.li(8).wrapping_add(5)))) as f64)) as f32); cx.set_lf(0, t); }
                        if (cx.lf(12) > cx.lf(0)) {
                            { let t = cx.lf(0); cx.set_lf(12, t); }
                        }
                    }
                } else {
                    v8 = (v8 & !0xFFFF_FFFFi64) | ((cx.lf(8)).to_bits() as i64);
                    v8 = (v8 & 0xFFFF_FFFFi64) | ((((cx.lf(8)).to_bits() as i32) as u32 as i64) << 32);
                    { let t = (((((cx.f(cx.lpp(8).add((4_i32).wrapping_mul(cx.li(8)))) + cx.f(a6.add(56))) - cx.f(cx.lptr(0).add(4 * (cx.i(Ptr::glob(0x688980).add(4 * (cx.li(8)))).wrapping_add(117))))) as f64) / (cx.f(cx.lptr(0).add(4 * (cx.li(8).wrapping_add(5)))) as f64)) as f32); cx.set_lf(4, t); }
                    if (cx.lf(12) > cx.lf(4)) {
                        { let t = cx.lf(4); cx.set_lf(12, t); }
                    }
                }
            }
            let _ = cx.preinc_i(cx.lptr(8));
        }
        if ((cx.lf(12) >= 1.0f32) || (cx.lf(12) <= 0.0f32)) {
            break 'lp4;
        }
        { let p = cx.lptr(0).add(4 * (cx.i(Ptr::glob(0x688980).add(4 * (0))).wrapping_add(117))); let t = ((((cx.lf(12) * cx.lf(20)) as f64) + (cx.f(cx.lptr(0).add(4 * (cx.i(Ptr::glob(0x688980).add(4 * (0))).wrapping_add(117)))) as f64)) as f32); cx.set_f(p, t); }
        { let p = cx.lptr(0).add(4 * (cx.i(Ptr::glob(0x688984)).wrapping_add(117))); let t = ((((cx.lf(12) * cx.lf(24)) as f64) + (cx.f(cx.lptr(0).add(4 * (cx.i(Ptr::glob(0x688984)).wrapping_add(117)))) as f64)) as f32); cx.set_f(p, t); }
        { let p = cx.lptr(0).add(4 * (cx.i(Ptr::glob(0x688988)).wrapping_add(117))); let t = (((((cx.f(a4.add((4_i32).wrapping_mul(cx.i(Ptr::glob(0x688988))))) - cx.f(cx.lptr(0).add(4 * (cx.i(Ptr::glob(0x688988)).wrapping_add(117))))) * cx.lf(12)) as f64) + (cx.f(cx.lptr(0).add(4 * (cx.i(Ptr::glob(0x688988)).wrapping_add(117)))) as f64)) as f32); cx.set_f(p, t); }
    }
    v8 = (v8 & !(0xFFi64 as i64)) | ((((((cx.li(460) >> 24) & 255)) as i64) & 0xFF) << 0) as i64;
    v10 = v8;
    return v10;
}

/// PhysicsManager__19E030 0x0019E030
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_19e030(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = cx.i(a2); cx.set_i(p, t); }
    { let p = a1.add(4); let t = cx.i(a2.add(4)); cx.set_i(p, t); }
    { let p = a1.add(8); let t = cx.i(a2.add(8)); cx.set_i(p, t); }
    return a1;
}

/// PhysicsManager__19FD80 0x0019FD80
pub fn f_19fd80(cx: &mut Cx, a1: Ptr, a3: Ptr, a4: i32, a5: i32, a6: f32, a7: i32, a8: f32, a9: Ptr, mut a10: i32, a11: Ptr) -> i64 {
    cx.push_frame(184);
    let r = f_19fd80_body(cx, a1, a3, a4, a5, a6, a7, a8, a9, a10, a11);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_19fd80_body(cx: &mut Cx, a1: Ptr, a3: Ptr, a4: i32, a5: i32, a6: f32, a7: i32, a8: f32, a9: Ptr, mut a10: i32, a11: Ptr) -> i64 {
    let mut a2 = 0.0f64;
    let mut v11 = Ptr::NULL;
    let mut v12 = 0i32;
    let mut v13 = 0i32;
    let mut v14 = 0.0f64;
    let mut v15 = 0i32;
    let mut v16 = 0.0f64;
    let mut v17 = 0i32;
    let mut v18 = 0i32;
    let mut v19 = 0.0f64;
    let mut v20 = 0.0f64;
    let mut v21 = 0i32;
    let mut v24 = 0.0f32;
    let mut v25 = 0.0f32;
    let mut v26 = 0i64;
    let mut v27 = 0.0f32;
    let mut a10 = f32::from_bits(a10 as u32);
    let mut a7 = f32::from_bits(a7 as u32);
    let mut a5 = f32::from_bits(a5 as u32);
    let mut a4 = f32::from_bits(a4 as u32);
    { let t = a1; cx.set_lpp(45, t); }
    { let t = 0; cx.set_li(176, t); }
    if (a9.bits() != Ptr::NULL.bits()) {
        a2 = vsmath::cos((((a10 as f64) * 0.017453292f64) as f32));
        a10 = (a2 as f32);
    }
    if ({ let p = ov_physics_manager(cx).add(2836); cx.i(p) } != 0) {
        v25 = cx.f(a3.add(8));
        v24 = cx.f(a3);
        v11 = ov_physics_manager(cx);
        { let t = ({ let q0_ = cx.ptr_at(v11.add(2836)); let q1_ = v12; let q2_ = cx.lptr(168); let q3_ = v24; let q4_ = v25; let q5_ = (a4.to_bits() as i32); f_385920(cx, q0_, q1_, q2_, q3_, q4_, q5_) } as i32); cx.set_li(160, t); }
        if (cx.li(168) == 0) {
            let _ = ov_physics_manager(cx);
            crate::ov_ignore!(cx, 1, "!CAxisSortedList__GetCollisionCandidates(): The number of returned candidates could not fit into CAxisSortedList's preallocated array of %d, change the constructor parameters\n");
        }
        { let t = 0; cx.set_li(156, t); }
        'lp4: loop {
            if !(cx.li(156) < cx.li(160)) { break 'lp4; }
            'blp4: {
                { let t = cx.ptr_at(Ptr::raw(cx.li(168).wrapping_add((4_i32).wrapping_mul(cx.li(156))))); cx.set_lpp(38, t); }
                if ((cx.lpp(38).bits() != 0) && (cx.i(cx.lpp(38).add(100)) != 0)) {
                    { let q0_ = cx.lptr(136); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(38).add(100)).add(32)); let q2_ = a3; f_19d920(cx, q0_, q1_, q2_) };
                    a2 = ({ let q0_ = cx.lptr(136); f_1a0480(cx, q0_) } as f64);
                    { let p = cx.lptr(128); let t = (a2 as f32); cx.set_f(p, t); }
                    if ((a4 * a4) > cx.f(cx.lptr(128))) {
                        a2 = vsmath::sqrt(cx.f(cx.lptr(128)));
                        { let p = cx.lptr(104); let t = (a2 as f32); cx.set_f(p, t); }
                        if (a9.bits() != Ptr::NULL.bits()) {
                            a2 = { let q0_ = cx.lptr(136); let q1_ = a9; f_19d9c0(cx, q0_, q1_) };
                            { let p = cx.lptr(100); let t = (a2 as f32); cx.set_f(p, t); }
                            if ((cx.f(cx.lptr(104)) == 0.0f32) || (a10 > (cx.f(cx.lptr(100)) / cx.f(cx.lptr(104))))) {
                                break 'blp4;
                            }
                            { let q0_ = cx.lptr(112); let q1_ = a9; let q2_ = cx.lptr(136); f_1a03f0(cx, q0_, q1_, q2_) };
                            if (({ let q0_ = cx.lptr(112); f_1a04a0(cx, q0_) } as u8 as i32) == 0) {
                                { let q0_ = cx.lptr(112); let q1_ = 1065353216; let q2_ = 0; let q3_ = 0; f_1a0360(cx, q0_, q1_, q2_, q3_) };
                            }
                        }
                        { let q0_ = cx.lpp(38); f_1e4710(cx, q0_) };
                        { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(38).add(100)).add(36)).add(32); let q1_ = cx.lptr(136); let q2_ = ((((((a4 - cx.f(cx.lptr(104))) / a4) * a5) as f64) / (cx.f(cx.lptr(104)) as f64)) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                        if (a9.bits() != Ptr::NULL.bits()) {
                            a2 = ((((f_1a0510(cx) as f64) * ((a4 as f64) - (cx.f(cx.lptr(104)) as f64))) / (a4 as f64)) * (a6 as f64));
                            v27 = (a2 as f32);
                            { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(38).add(100)).add(36)).add(48); let q1_ = cx.lptr(112); let q2_ = v27; f_1a0390(cx, q0_, q1_, q2_) };
                        } else {
                            v13 = cx.i(cx.ptr_at(cx.lpp(38).add(100)).add(36));
                            v14 = (((((f_1a0530(cx) as f64) * ((a4 as f64) - (cx.f(cx.lptr(104)) as f64))) / (a4 as f64)) * (a6 as f64)) + (cx.f(Ptr::raw(v13.wrapping_add(48))) as f64));
                            { let p = cx.ptr_at(cx.ptr_at(cx.lpp(38).add(100)).add(36)).add(48); let t = (v14 as f32); cx.set_f(p, t); }
                            v15 = cx.i(cx.ptr_at(cx.lpp(38).add(100)).add(36));
                            v16 = (((((f_1a0530(cx) as f64) * ((a4 as f64) - (cx.f(cx.lptr(104)) as f64))) / (a4 as f64)) * (a6 as f64)) + (cx.f(Ptr::raw(v15.wrapping_add(52))) as f64));
                            { let p = cx.ptr_at(cx.ptr_at(cx.lpp(38).add(100)).add(36)).add(52); let t = (v16 as f32); cx.set_f(p, t); }
                            v17 = cx.i(cx.ptr_at(cx.lpp(38).add(100)).add(36));
                            a2 = (((((f_1a0530(cx) as f64) * ((a4 as f64) - (cx.f(cx.lptr(104)) as f64))) / (a4 as f64)) * (a6 as f64)) + (cx.f(Ptr::raw(v17.wrapping_add(56))) as f64));
                            { let p = cx.ptr_at(cx.ptr_at(cx.lpp(38).add(100)).add(36)).add(56); let t = (a2 as f32); cx.set_f(p, t); }
                        }
                        { let q0_ = cx.lptr(44); f_1b4210(cx, q0_) };
                        { let t = cx.lpp(38); cx.set_lpp(12, t); }
                        { let q0_ = Ptr::glob(0x6FAA34); let q1_ = 2; let q2_ = cx.lptr(44); ov_message(cx, q0_, q1_, q2_) };
                        if ({ let q0_ = cx.ptr_at(cx.lpp(38).add(100)); f_1b41f0(cx, q0_) } != 0) {
                            { let p = cx.ptr_at(cx.ptr_at(cx.lpp(38).add(100)).add(36)).add(536); let t = 1; cx.set_i(p, t); }
                            { let t = cx.li(176).wrapping_add(1); cx.set_li(176, t); }
                        }
                    }
                }
            }
            let _ = cx.preinc_i(cx.lptr(156));
        }
    }
    { let t = 0; cx.set_li(36, t); }
    'lp5: loop {
        'blp5: {
            v18 = { let q0_ = cx.i(cx.lpp(45)); ov_vehicle_count(cx, q0_) };
            if (cx.li(36) >= v18) {
                break 'lp5;
            }
            if (({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(45)); let q1_ = a2; let q2_ = cx.li(36); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(28); cx.b(p) } != 0) && (cx.li(36) != a11.bits())) {
                { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(45)); let q1_ = a2; let q2_ = cx.li(36); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }; cx.set_lpp(8, t); }
                { let q0_ = cx.lptr(16); let q1_ = cx.lpp(8).add(304); let q2_ = a3; f_19d920(cx, q0_, q1_, q2_) };
                a2 = ({ let q0_ = cx.lptr(16); f_19d9f0(cx, q0_) } as f64);
                { let p = cx.lptr(8); let t = (a2 as f32); cx.set_f(p, t); }
                if (cx.f(cx.lptr(8)) < 0.01f32) {
                    { let t = Ptr::raw(1008981770); cx.set_lpp(2, t); }
                    { let q0_ = cx.lptr(16); let q1_ = 0; let q2_ = 1008981770; let q3_ = 0; f_1a0360(cx, q0_, q1_, q2_, q3_) };
                }
                if (a4 > cx.f(cx.lptr(8))) {
                    { let q0_ = cx.lpp(8).add(272); let q1_ = cx.lptr(16); let q2_ = ((((((a4 - cx.f(cx.lptr(8))) / a4) * a7) as f64) / (cx.f(cx.lptr(8)) as f64)) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                    v19 = (((((f_1a0530(cx) as f64) * ((a4 as f64) - (cx.f(cx.lptr(8)) as f64))) / (a4 as f64)) * (a8 as f64)) + (cx.f(cx.lpp(8).add(288)) as f64));
                    { let p = cx.lpp(8).add(288); let t = (v19 as f32); cx.set_f(p, t); }
                    v20 = (((((f_1a0530(cx) as f64) * ((a4 as f64) - (cx.f(cx.lptr(8)) as f64))) / (a4 as f64)) * (a8 as f64)) + (cx.f(cx.lpp(8).add(292)) as f64));
                    { let p = cx.lpp(8).add(292); let t = (v20 as f32); cx.set_f(p, t); }
                    a2 = (((((f_1a0530(cx) as f64) * ((a4 as f64) - (cx.f(cx.lptr(8)) as f64))) / (a4 as f64)) * (a8 as f64)) + (cx.f(cx.lpp(8).add(296)) as f64));
                    { let p = cx.lpp(8).add(296); let t = (a2 as f32); cx.set_f(p, t); }
                    { let t = cx.li(176).wrapping_add(1); cx.set_li(176, t); }
                }
            }
        }
        let _ = cx.preinc_i(cx.lptr(36));
    }
    { let t = 0; cx.set_li(4, t); }
    'lp6: loop {
        'blp6: {
            v21 = { let q0_ = cx.i(cx.lpp(45)); ov_vehicle_count(cx, q0_) };
            if (cx.li(4) >= v21) {
                break 'lp6;
            }
            { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(45)); let q1_ = a2; let q2_ = cx.li(4); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }; cx.set_lpp(0, t); }
            if ((cx.lpp(0).bits() != 0) && (cx.i(cx.lpp(0).add(5084)) == 0)) {
                { let q0_ = cx.lpp(0); f_1d1170(cx, q0_) };
            }
        }
        let _ = cx.preinc_i(cx.lptr(4));
    }
    v26 = (cx.li(176) as i64);
    return v26;
}

/// PhysicsManager__1A0360 0x001A0360
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a0360(cx: &mut Cx, mut a1: Ptr, a2: i32, a3: i32, a4: i32) -> Ptr {
    { let p = a1; let t = a2; cx.set_i(p, t); }
    { let p = a1.add(4); let t = a3; cx.set_i(p, t); }
    { let p = a1.add(8); let t = a4; cx.set_i(p, t); }
    return a1;
}

/// PhysicsManager__1A0390 0x001A0390
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a0390(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: f32) -> Ptr {
    { let p = a1; let t = ((((cx.f(a2) * a3) as f64) + (cx.f(a1) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((cx.f(a2.add(4)) * a3) as f64) + (cx.f(a1.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((cx.f(a2.add(8)) * a3) as f64) + (cx.f(a1.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// PhysicsManager__1A03F0 0x001A03F0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a03f0(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = ((((cx.f(a2.add(4)) * cx.f(a3.add(8))) as f64) - ((cx.f(a2.add(8)) * cx.f(a3.add(4))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((cx.f(a2.add(8)) * cx.f(a3)) as f64) - ((cx.f(a2) * cx.f(a3.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((cx.f(a2) * cx.f(a3.add(4))) as f64) - ((cx.f(a2.add(4)) * cx.f(a3)) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// PhysicsManager__1A0480 0x001A0480
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a0480(cx: &mut Cx, a1: Ptr) -> f32 {
    return (f_19d9c0(cx, a1, a1) as f32);
}

/// PhysicsManager__1A04A0 0x001A04A0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a04a0(cx: &mut Cx, a1: Ptr) -> i32 {
    let mut v2 = 0.0f32;
    v2 = f_19d9f0(cx, a1);
    if (v2 == 0.0f32) {
        return 0;
    }
    let _ = f_19dab0(cx, a1, ((1.0f64 / (v2 as f64)) as f32));
    return 1;
}

/// PhysicsManager__1A0510 0x001A0510
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a0510(cx: &mut Cx) -> f32 {
    return (vs_rand_unit(cx) as f32);
}

/// PhysicsManager__1A0530 0x001A0530
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a0530(cx: &mut Cx) -> f32 {
    let mut v3 = 0i32;
    v3 = ov_rand(cx);
    return ((((((v3 as f64) - 16383.5f64) + (v3 as f64)) - 16383.5f64) / 32767.0f64) as f32);
}

/// PhysicsManager__1A0570 0x001A0570
pub fn f_1a0570(cx: &mut Cx, a1: i32, a3: Ptr, a4: Ptr, a5: i32, a6: Ptr) -> i64 {
    cx.push_frame(2496);
    let r = f_1a0570_body(cx, a1, a3, a4, a5, a6);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1a0570_body(cx: &mut Cx, a1: i32, a3: Ptr, a4: Ptr, a5: i32, a6: Ptr) -> i64 {
    let mut a2 = 0.0f64;
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
    let mut v19 = 0.0f64;
    let mut v20 = Ptr::NULL;
    let mut v21 = Ptr::NULL;
    let mut v22 = Ptr::NULL;
    let mut v23 = Ptr::NULL;
    let mut v24 = Ptr::NULL;
    let mut v25 = Ptr::NULL;
    let mut v26 = Ptr::NULL;
    let mut v27 = Ptr::NULL;
    let mut v28 = Ptr::NULL;
    let mut v29 = Ptr::NULL;
    let mut v30 = Ptr::NULL;
    let mut v31 = Ptr::NULL;
    let mut v32 = 0i32;
    let mut v33 = Ptr::NULL;
    let mut v34 = Ptr::NULL;
    let mut v35 = Ptr::NULL;
    let mut v36 = Ptr::NULL;
    let mut v37 = Ptr::NULL;
    let mut v38 = Ptr::NULL;
    let mut v39 = Ptr::NULL;
    let mut v40 = Ptr::NULL;
    let mut v41 = Ptr::NULL;
    let mut v42 = Ptr::NULL;
    let mut v43 = Ptr::NULL;
    let mut v44 = Ptr::NULL;
    let mut v45 = Ptr::NULL;
    let mut v46 = Ptr::NULL;
    let mut v47 = 0.0f64;
    let mut v48 = 0.0f64;
    let mut v49 = Ptr::NULL;
    let mut v50 = Ptr::NULL;
    let mut v51 = 0i32;
    let mut v52 = 0.0f64;
    let mut v53 = Ptr::NULL;
    let mut v54 = 0.0f64;
    let mut v55 = Ptr::NULL;
    let mut v56 = Ptr::NULL;
    let mut v57 = 0.0f64;
    let mut v58 = Ptr::NULL;
    let mut v59 = 0.0f64;
    let mut v60 = Ptr::NULL;
    let mut v61 = Ptr::NULL;
    let mut v62 = Ptr::NULL;
    let mut v63 = 0.0f64;
    let mut v64 = Ptr::NULL;
    let mut v65 = Ptr::NULL;
    let mut v66 = Ptr::NULL;
    let mut v67 = 0.0f64;
    let mut v68 = Ptr::NULL;
    let mut v69 = Ptr::NULL;
    let mut v70 = 0.0f64;
    let mut v71 = 0.0f64;
    let mut v73 = Ptr::NULL;
    let mut v74 = Ptr::NULL;
    let mut v75 = Ptr::NULL;
    let mut v76 = Ptr::NULL;
    let mut v77 = Ptr::NULL;
    let mut v78 = Ptr::NULL;
    let mut v79 = Ptr::NULL;
    let mut v80 = Ptr::NULL;
    let mut v81 = Ptr::NULL;
    let mut v82 = Ptr::NULL;
    let mut v83 = Ptr::NULL;
    let mut v84 = Ptr::NULL;
    let mut v85 = Ptr::NULL;
    let mut v86 = Ptr::NULL;
    let mut v87 = Ptr::NULL;
    let mut v88 = Ptr::NULL;
    let mut v89 = Ptr::NULL;
    let mut v90 = Ptr::NULL;
    let mut v91 = Ptr::NULL;
    let mut v92 = Ptr::NULL;
    let mut v93 = Ptr::NULL;
    let mut v94 = Ptr::NULL;
    let mut v95 = Ptr::NULL;
    let mut v96 = Ptr::NULL;
    let mut v97 = Ptr::NULL;
    let mut v98 = Ptr::NULL;
    let mut v99 = Ptr::NULL;
    let mut v100 = Ptr::NULL;
    let mut v101 = Ptr::NULL;
    let mut v102 = Ptr::NULL;
    let mut v103 = Ptr::NULL;
    let mut v104 = Ptr::NULL;
    let mut v105 = Ptr::NULL;
    let mut v106 = Ptr::NULL;
    let mut v107 = Ptr::NULL;
    let mut v108 = Ptr::NULL;
    let mut v109 = Ptr::NULL;
    let mut v110 = 0i32;
    let mut v111 = 0i32;
    let mut v112 = 0i32;
    let mut v113 = 0i32;
    let mut v114 = 0i32;
    let mut v115 = 0i32;
    let mut v116 = 0i32;
    let mut v117 = 0i32;
    let mut v118 = 0i32;
    let mut v119 = 0i32;
    let mut v120 = 0i32;
    let mut v121 = 0i32;
    let mut v122 = 0i32;
    let mut v123 = 0i32;
    let mut v124 = 0i32;
    let mut v125 = 0i32;
    let mut v126 = 0i32;
    let mut v127 = 0i32;
    let mut v128 = 0i32;
    let mut v129 = 0i32;
    let mut v130 = 0i32;
    let mut v131 = 0i32;
    let mut v132 = 0i32;
    let mut v133 = 0i32;
    let mut v134 = 0i32;
    let mut v135 = 0i32;
    let mut v136 = 0i32;
    let mut v137 = 0i32;
    let mut v138 = 0i32;
    let mut v139 = 0i32;
    let mut v140 = 0i32;
    let mut v141 = 0i32;
    let mut v142 = 0i32;
    let mut v143 = 0i32;
    let mut v144 = 0i32;
    let mut v145 = 0i32;
    let mut v146 = 0i32;
    let mut v147 = 0i64;
    let mut v148 = 0i32;
    let mut v149 = 0i32;
    let mut v150 = 0i32;
    let mut v151 = 0i32;
    let mut v152 = 0i32;
    let mut v153 = 0i32;
    let mut v154 = 0i32;
    let mut v155 = 0i32;
    let mut v156 = 0i32;
    let mut v157 = 0i32;
    let mut v158 = 0i32;
    let mut v159 = 0i32;
    let mut v160 = 0i32;
    let mut v161 = 0i32;
    let mut v162 = 0i32;
    let mut v163 = 0i32;
    let mut v164 = 0i32;
    let mut v165 = 0i32;
    let mut v166 = 0i32;
    let mut v167 = 0i32;
    let mut v168 = 0i32;
    let mut v169 = 0i32;
    let mut v170 = 0i32;
    let mut v171 = 0i32;
    let mut v172 = 0i32;
    let mut v173 = 0i32;
    let mut v174 = 0i32;
    let mut v175 = 0i32;
    let mut v176 = 0i32;
    let mut v177 = 0i32;
    let mut v178 = 0i32;
    let mut v179 = 0i32;
    let mut v180 = 0i32;
    let mut v181 = 0i32;
    let mut v182 = 0i32;
    let mut v183 = 0i32;
    let mut v184 = 0i32;
    let mut v185 = Ptr::NULL;
    let mut v186 = 0.0f32;
    let mut v187 = 0.0f32;
    let mut v188 = 0.0f32;
    let mut v189 = 0.0f32;
    let mut v190 = 0.0f32;
    let mut v191 = 0.0f32;
    let mut v192 = 0.0f32;
    let mut v193 = Ptr::NULL;
    let mut v194 = 0.0f32;
    let mut v195 = 0.0f32;
    let mut v196 = 0.0f32;
    let mut v197 = 0.0f32;
    let mut v198 = 0.0f32;
    let mut v199 = 0.0f32;
    let mut v200 = 0.0f32;
    { let t = a1; cx.set_lpp(623, Ptr::raw(t)); }
    { let t = 1; cx.set_li(2488, t); }
    'lp12: loop {
        if !(cx.li(2488) < cx.i(a3)) { break 'lp12; }
        'blp12: {
            { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(623)); let q1_ = a2; let q2_ = cx.li(2488); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }; cx.set_lpp(621, t); }
            if (cx.b(cx.lpp(621).add(28)) != 0) {
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(621).add(3556))).add(56); let t = (((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(621).add(3556))).add(56)) as f64) + (cx.f(Ptr::gc(0).add(1212)) as f64)) as f32); cx.set_f(p, t); }
            }
        }
        let _ = cx.preinc_i(cx.lptr(2488));
    }
    { let t = 0; cx.set_li(2480, t); }
    'lp13: loop {
        if !(cx.li(2480) < cx.i(a3)) { break 'lp13; }
        'blp13: {
            { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(623)); let q1_ = a2; let q2_ = cx.li(2480); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }; cx.set_lpp(619, t); }
            if (cx.b(cx.lpp(619).add(28)) != 0) {
                if (cx.i(Ptr::gc(0).add(1988)) != 0) {
                    { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(619).add(3556))); let q1_ = cx.i(cx.lpp(619).add(3556)); let q2_ = cx.lptr(2376); f_37a7c0(cx, q0_, q1_, q2_) };
                    v148 = { let p = { let q0_ = cx.lptr(1328); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v110 = cx.i(Ptr::glob(0x77FD64));
                    v73 = { let q0_ = cx.lptr(1312); let q1_ = cx.li(2388); let q2_ = cx.li(2392); let q3_ = cx.li(2396); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v6 = { let q0_ = cx.lptr(1296); let q1_ = cx.li(2376); let q2_ = cx.li(2380); let q3_ = cx.li(2384); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v6, v73, v110, v148);
                    v149 = { let p = { let q0_ = cx.lptr(1292); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v111 = cx.i(Ptr::glob(0x77FD64));
                    v74 = { let q0_ = cx.lptr(1276); let q1_ = cx.li(2400); let q2_ = cx.li(2404); let q3_ = cx.li(2408); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v7 = { let q0_ = cx.lptr(1260); let q1_ = cx.li(2388); let q2_ = cx.li(2392); let q3_ = cx.li(2396); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v7, v74, v111, v149);
                    v150 = { let p = { let q0_ = cx.lptr(1256); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v112 = cx.i(Ptr::glob(0x77FD64));
                    v75 = { let q0_ = cx.lptr(1240); let q1_ = cx.li(2412); let q2_ = cx.li(2416); let q3_ = cx.li(2420); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v8 = { let q0_ = cx.lptr(1224); let q1_ = cx.li(2400); let q2_ = cx.li(2404); let q3_ = cx.li(2408); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v8, v75, v112, v150);
                    v151 = { let p = { let q0_ = cx.lptr(1220); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v113 = cx.i(Ptr::glob(0x77FD64));
                    v76 = { let q0_ = cx.lptr(1204); let q1_ = cx.li(2376); let q2_ = cx.li(2380); let q3_ = cx.li(2384); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v9 = { let q0_ = cx.lptr(1188); let q1_ = cx.li(2412); let q2_ = cx.li(2416); let q3_ = cx.li(2420); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v9, v76, v113, v151);
                    v152 = { let p = { let q0_ = cx.lptr(1184); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v114 = cx.i(Ptr::glob(0x77FD64));
                    v77 = { let q0_ = cx.lptr(1168); let q1_ = cx.li(2436); let q2_ = cx.li(2440); let q3_ = cx.li(2444); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v10 = { let q0_ = cx.lptr(1152); let q1_ = cx.li(2424); let q2_ = cx.li(2428); let q3_ = cx.li(2432); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v10, v77, v114, v152);
                    v153 = { let p = { let q0_ = cx.lptr(1148); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v115 = cx.i(Ptr::glob(0x77FD64));
                    v78 = { let q0_ = cx.lptr(1132); let q1_ = cx.li(2448); let q2_ = cx.li(2452); let q3_ = cx.li(2456); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v11 = { let q0_ = cx.lptr(1116); let q1_ = cx.li(2436); let q2_ = cx.li(2440); let q3_ = cx.li(2444); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v11, v78, v115, v153);
                    v154 = { let p = { let q0_ = cx.lptr(1112); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v116 = cx.i(Ptr::glob(0x77FD64));
                    v79 = { let q0_ = cx.lptr(1096); let q1_ = cx.li(2460); let q2_ = cx.li(2464); let q3_ = cx.li(2468); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v12 = { let q0_ = cx.lptr(1080); let q1_ = cx.li(2448); let q2_ = cx.li(2452); let q3_ = cx.li(2456); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v12, v79, v116, v154);
                    v155 = { let p = { let q0_ = cx.lptr(1076); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v117 = cx.i(Ptr::glob(0x77FD64));
                    v80 = { let q0_ = cx.lptr(1060); let q1_ = cx.li(2424); let q2_ = cx.li(2428); let q3_ = cx.li(2432); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v13 = { let q0_ = cx.lptr(1044); let q1_ = cx.li(2460); let q2_ = cx.li(2464); let q3_ = cx.li(2468); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v13, v80, v117, v155);
                    v156 = { let p = { let q0_ = cx.lptr(1040); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v118 = cx.i(Ptr::glob(0x77FD64));
                    v81 = { let q0_ = cx.lptr(1024); let q1_ = cx.li(2424); let q2_ = cx.li(2428); let q3_ = cx.li(2432); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v14 = { let q0_ = cx.lptr(1008); let q1_ = cx.li(2376); let q2_ = cx.li(2380); let q3_ = cx.li(2384); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v14, v81, v118, v156);
                    v157 = { let p = { let q0_ = cx.lptr(1004); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v119 = cx.i(Ptr::glob(0x77FD64));
                    v82 = { let q0_ = cx.lptr(988); let q1_ = cx.li(2436); let q2_ = cx.li(2440); let q3_ = cx.li(2444); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v15 = { let q0_ = cx.lptr(972); let q1_ = cx.li(2388); let q2_ = cx.li(2392); let q3_ = cx.li(2396); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v15, v82, v119, v157);
                    v158 = { let p = { let q0_ = cx.lptr(968); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v120 = cx.i(Ptr::glob(0x77FD64));
                    v83 = { let q0_ = cx.lptr(952); let q1_ = cx.li(2448); let q2_ = cx.li(2452); let q3_ = cx.li(2456); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v16 = { let q0_ = cx.lptr(936); let q1_ = cx.li(2400); let q2_ = cx.li(2404); let q3_ = cx.li(2408); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v16, v83, v120, v158);
                    v159 = { let p = { let q0_ = cx.lptr(932); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v121 = cx.i(Ptr::glob(0x77FD64));
                    v84 = { let q0_ = cx.lptr(916); let q1_ = cx.li(2460); let q2_ = cx.li(2464); let q3_ = cx.li(2468); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v17 = { let q0_ = cx.lptr(900); let q1_ = cx.li(2412); let q2_ = cx.li(2416); let q3_ = cx.li(2420); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v17, v84, v121, v159);
                }
                { let t = 0; cx.set_li(2368, t); }
                'lp14: loop {
                    if !(cx.li(2368) < cx.li(2480)) { break 'lp14; }
                    'blp14: {
                        { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(623)); let q1_ = a2; let q2_ = cx.li(2368); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }; cx.set_lpp(591, t); }
                        if (cx.b(cx.lpp(591).add(28)) != 0) {
                            { let t = 0; cx.set_li(2360, t); }
                            { let t = 0; cx.set_li(2356, t); }
                            { let t = cx.ptr_at(cx.ptr_at(cx.lpp(619).add(3556))); cx.set_lpp(588, t); }
                            { let t = cx.ptr_at(cx.ptr_at(cx.lpp(591).add(3556))); cx.set_lpp(587, t); }
                            { let t = a5; cx.set_li(2340, t); }
                            { let p = cx.lptr(2332); let t = ((((cx.f(cx.lpp(588).add(12)) + cx.f(cx.lpp(587).add(12))) as f64) + ((2.0f32 * cx.f(Ptr::gc(0).add(1212))) as f64)) as f32); cx.set_f(p, t); }
                            { let t = 0; cx.set_lpp(582, Ptr::raw(t)); }
                            v18 = vsmath::fabs((((cx.f(cx.lpp(587)) as f64) - (cx.f(cx.lpp(588)) as f64)) as f32));
                            a2 = (cx.f(cx.lptr(2332)) as f64);
                            if (v18 <= (cx.f(cx.lptr(2332)) as f64)) {
                                v19 = vsmath::fabs((((cx.f(cx.lpp(587).add(8)) as f64) - (cx.f(cx.lpp(588).add(8)) as f64)) as f32));
                                a2 = (cx.f(cx.lptr(2332)) as f64);
                                if ((v19 <= (cx.f(cx.lptr(2332)) as f64)) && (({ let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(2360))); let q1_ = cx.lptr(2340); let q2_ = cx.lpp(588); let q3_ = cx.lpp(587); let q4_ = 0.0f32; f_38cf90(cx, q0_, q1_, q2_, q3_, q4_) } as i32) != 0)) {
                                    if (cx.i(Ptr::gc(0).add(2008)) != 0) {
                                        { let q0_ = cx.lpp(588); let q1_ = cx.lptr(2188).bits(); let q2_ = cx.lptr(2188); f_37a7c0(cx, q0_, q1_, q2_) };
                                        v160 = { let p = { let q0_ = cx.lptr(896); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v122 = cx.i(Ptr::glob(0x77FDA8));
                                        v85 = { let q0_ = cx.lptr(880); let q1_ = cx.li(2200); let q2_ = cx.li(2204); let q3_ = cx.li(2208); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v20 = { let q0_ = cx.lptr(864); let q1_ = cx.li(2188); let q2_ = cx.li(2192); let q3_ = cx.li(2196); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v20, v85, v122, v160);
                                        v161 = { let p = { let q0_ = cx.lptr(860); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v123 = cx.i(Ptr::glob(0x77FDA8));
                                        v86 = { let q0_ = cx.lptr(844); let q1_ = cx.li(2212); let q2_ = cx.li(2216); let q3_ = cx.li(2220); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v21 = { let q0_ = cx.lptr(828); let q1_ = cx.li(2200); let q2_ = cx.li(2204); let q3_ = cx.li(2208); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v21, v86, v123, v161);
                                        v162 = { let p = { let q0_ = cx.lptr(824); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v124 = cx.i(Ptr::glob(0x77FDA8));
                                        v87 = { let q0_ = cx.lptr(808); let q1_ = cx.li(2224); let q2_ = cx.li(2228); let q3_ = cx.li(2232); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v22 = { let q0_ = cx.lptr(792); let q1_ = cx.li(2212); let q2_ = cx.li(2216); let q3_ = cx.li(2220); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v22, v87, v124, v162);
                                        v163 = { let p = { let q0_ = cx.lptr(788); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v125 = cx.i(Ptr::glob(0x77FDA8));
                                        v88 = { let q0_ = cx.lptr(772); let q1_ = cx.li(2188); let q2_ = cx.li(2192); let q3_ = cx.li(2196); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v23 = { let q0_ = cx.lptr(756); let q1_ = cx.li(2224); let q2_ = cx.li(2228); let q3_ = cx.li(2232); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v23, v88, v125, v163);
                                        v164 = { let p = { let q0_ = cx.lptr(752); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v126 = cx.i(Ptr::glob(0x77FDA8));
                                        v89 = { let q0_ = cx.lptr(736); let q1_ = cx.li(2248); let q2_ = cx.li(2252); let q3_ = cx.li(2256); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v24 = { let q0_ = cx.lptr(720); let q1_ = cx.li(2236); let q2_ = cx.li(2240); let q3_ = cx.li(2244); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v24, v89, v126, v164);
                                        v165 = { let p = { let q0_ = cx.lptr(716); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v127 = cx.i(Ptr::glob(0x77FDA8));
                                        v90 = { let q0_ = cx.lptr(700); let q1_ = cx.li(2260); let q2_ = cx.li(2264); let q3_ = cx.li(2268); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v25 = { let q0_ = cx.lptr(684); let q1_ = cx.li(2248); let q2_ = cx.li(2252); let q3_ = cx.li(2256); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v25, v90, v127, v165);
                                        v166 = { let p = { let q0_ = cx.lptr(680); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v128 = cx.i(Ptr::glob(0x77FDA8));
                                        v91 = { let q0_ = cx.lptr(664); let q1_ = cx.li(2272); let q2_ = cx.li(2276); let q3_ = cx.li(2280); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v26 = { let q0_ = cx.lptr(648); let q1_ = cx.li(2260); let q2_ = cx.li(2264); let q3_ = cx.li(2268); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v26, v91, v128, v166);
                                        v167 = { let p = { let q0_ = cx.lptr(644); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v129 = cx.i(Ptr::glob(0x77FDA8));
                                        v92 = { let q0_ = cx.lptr(628); let q1_ = cx.li(2236); let q2_ = cx.li(2240); let q3_ = cx.li(2244); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v27 = { let q0_ = cx.lptr(612); let q1_ = cx.li(2272); let q2_ = cx.li(2276); let q3_ = cx.li(2280); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v27, v92, v129, v167);
                                        v168 = { let p = { let q0_ = cx.lptr(608); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v130 = cx.i(Ptr::glob(0x77FDA8));
                                        v93 = { let q0_ = cx.lptr(592); let q1_ = cx.li(2236); let q2_ = cx.li(2240); let q3_ = cx.li(2244); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v28 = { let q0_ = cx.lptr(576); let q1_ = cx.li(2188); let q2_ = cx.li(2192); let q3_ = cx.li(2196); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v28, v93, v130, v168);
                                        v169 = { let p = { let q0_ = cx.lptr(572); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v131 = cx.i(Ptr::glob(0x77FDA8));
                                        v94 = { let q0_ = cx.lptr(556); let q1_ = cx.li(2248); let q2_ = cx.li(2252); let q3_ = cx.li(2256); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v29 = { let q0_ = cx.lptr(540); let q1_ = cx.li(2200); let q2_ = cx.li(2204); let q3_ = cx.li(2208); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v29, v94, v131, v169);
                                        v170 = { let p = { let q0_ = cx.lptr(536); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v132 = cx.i(Ptr::glob(0x77FDA8));
                                        v95 = { let q0_ = cx.lptr(520); let q1_ = cx.li(2260); let q2_ = cx.li(2264); let q3_ = cx.li(2268); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v30 = { let q0_ = cx.lptr(504); let q1_ = cx.li(2212); let q2_ = cx.li(2216); let q3_ = cx.li(2220); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v30, v95, v132, v170);
                                        v171 = { let p = { let q0_ = cx.lptr(500); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v133 = cx.i(Ptr::glob(0x77FDA8));
                                        v96 = { let q0_ = cx.lptr(484); let q1_ = cx.li(2272); let q2_ = cx.li(2276); let q3_ = cx.li(2280); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v31 = { let q0_ = cx.lptr(468); let q1_ = cx.li(2224); let q2_ = cx.li(2228); let q3_ = cx.li(2232); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v31, v96, v133, v171);
                                        { let q0_ = cx.lpp(587); let q1_ = v32; let q2_ = cx.lptr(2188); f_37a7c0(cx, q0_, q1_, q2_) };
                                        v172 = { let p = { let q0_ = cx.lptr(464); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v134 = cx.i(Ptr::glob(0x77FD48));
                                        v97 = { let q0_ = cx.lptr(448); let q1_ = cx.li(2200); let q2_ = cx.li(2204); let q3_ = cx.li(2208); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v33 = { let q0_ = cx.lptr(432); let q1_ = cx.li(2188); let q2_ = cx.li(2192); let q3_ = cx.li(2196); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v33, v97, v134, v172);
                                        v173 = { let p = { let q0_ = cx.lptr(428); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v135 = cx.i(Ptr::glob(0x77FD48));
                                        v98 = { let q0_ = cx.lptr(412); let q1_ = cx.li(2212); let q2_ = cx.li(2216); let q3_ = cx.li(2220); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v34 = { let q0_ = cx.lptr(396); let q1_ = cx.li(2200); let q2_ = cx.li(2204); let q3_ = cx.li(2208); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v34, v98, v135, v173);
                                        v174 = { let p = { let q0_ = cx.lptr(392); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v136 = cx.i(Ptr::glob(0x77FD48));
                                        v99 = { let q0_ = cx.lptr(376); let q1_ = cx.li(2224); let q2_ = cx.li(2228); let q3_ = cx.li(2232); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v35 = { let q0_ = cx.lptr(360); let q1_ = cx.li(2212); let q2_ = cx.li(2216); let q3_ = cx.li(2220); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v35, v99, v136, v174);
                                        v175 = { let p = { let q0_ = cx.lptr(356); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v137 = cx.i(Ptr::glob(0x77FD48));
                                        v100 = { let q0_ = cx.lptr(340); let q1_ = cx.li(2188); let q2_ = cx.li(2192); let q3_ = cx.li(2196); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v36 = { let q0_ = cx.lptr(324); let q1_ = cx.li(2224); let q2_ = cx.li(2228); let q3_ = cx.li(2232); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v36, v100, v137, v175);
                                        v176 = { let p = { let q0_ = cx.lptr(320); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v138 = cx.i(Ptr::glob(0x77FD48));
                                        v101 = { let q0_ = cx.lptr(304); let q1_ = cx.li(2248); let q2_ = cx.li(2252); let q3_ = cx.li(2256); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v37 = { let q0_ = cx.lptr(288); let q1_ = cx.li(2236); let q2_ = cx.li(2240); let q3_ = cx.li(2244); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v37, v101, v138, v176);
                                        v177 = { let p = { let q0_ = cx.lptr(284); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v139 = cx.i(Ptr::glob(0x77FD48));
                                        v102 = { let q0_ = cx.lptr(268); let q1_ = cx.li(2260); let q2_ = cx.li(2264); let q3_ = cx.li(2268); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v38 = { let q0_ = cx.lptr(252); let q1_ = cx.li(2248); let q2_ = cx.li(2252); let q3_ = cx.li(2256); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v38, v102, v139, v177);
                                        v178 = { let p = { let q0_ = cx.lptr(248); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v140 = cx.i(Ptr::glob(0x77FD48));
                                        v103 = { let q0_ = cx.lptr(232); let q1_ = cx.li(2272); let q2_ = cx.li(2276); let q3_ = cx.li(2280); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v39 = { let q0_ = cx.lptr(216); let q1_ = cx.li(2260); let q2_ = cx.li(2264); let q3_ = cx.li(2268); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v39, v103, v140, v178);
                                        v179 = { let p = { let q0_ = cx.lptr(212); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v141 = cx.i(Ptr::glob(0x77FD48));
                                        v104 = { let q0_ = cx.lptr(196); let q1_ = cx.li(2236); let q2_ = cx.li(2240); let q3_ = cx.li(2244); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v40 = { let q0_ = cx.lptr(180); let q1_ = cx.li(2272); let q2_ = cx.li(2276); let q3_ = cx.li(2280); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v40, v104, v141, v179);
                                        v180 = { let p = { let q0_ = cx.lptr(176); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v142 = cx.i(Ptr::glob(0x77FD48));
                                        v105 = { let q0_ = cx.lptr(160); let q1_ = cx.li(2236); let q2_ = cx.li(2240); let q3_ = cx.li(2244); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v41 = { let q0_ = cx.lptr(144); let q1_ = cx.li(2188); let q2_ = cx.li(2192); let q3_ = cx.li(2196); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v41, v105, v142, v180);
                                        v181 = { let p = { let q0_ = cx.lptr(140); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v143 = cx.i(Ptr::glob(0x77FD48));
                                        v106 = { let q0_ = cx.lptr(124); let q1_ = cx.li(2248); let q2_ = cx.li(2252); let q3_ = cx.li(2256); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v42 = { let q0_ = cx.lptr(108); let q1_ = cx.li(2200); let q2_ = cx.li(2204); let q3_ = cx.li(2208); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v42, v106, v143, v181);
                                        v182 = { let p = { let q0_ = cx.lptr(104); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v144 = cx.i(Ptr::glob(0x77FD48));
                                        v107 = { let q0_ = cx.lptr(88); let q1_ = cx.li(2260); let q2_ = cx.li(2264); let q3_ = cx.li(2268); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v43 = { let q0_ = cx.lptr(72); let q1_ = cx.li(2212); let q2_ = cx.li(2216); let q3_ = cx.li(2220); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v43, v107, v144, v182);
                                        v183 = { let p = { let q0_ = cx.lptr(68); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                        v145 = cx.i(Ptr::glob(0x77FD48));
                                        v108 = { let q0_ = cx.lptr(52); let q1_ = cx.li(2272); let q2_ = cx.li(2276); let q3_ = cx.li(2280); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        v44 = { let q0_ = cx.lptr(36); let q1_ = cx.li(2224); let q2_ = cx.li(2228); let q3_ = cx.li(2232); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v44, v108, v145, v183);
                                    }
                                    if (cx.f(Ptr::gc(0).add(2012)) != 0.0f32) {
                                        { let t = 0; cx.set_li(2180, t); }
                                        'lp15: loop {
                                            if !(cx.li(2180) < cx.li(2340)) { break 'lp15; }
                                            'blp15: {
                                                { let t = a4.add((1560_i32).wrapping_mul(cx.li(2180).wrapping_add(cx.li(2360)))); cx.set_lpp(544, t); }
                                                { let q0_ = cx.lptr(2160); let q1_ = cx.lpp(544).add(1276); let q2_ = cx.lpp(544).add(1332); let q3_ = (((cx.f(Ptr::gc(0).add(2012)) as f64) * (cx.f(cx.lpp(544).add(1288)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                                v184 = { let p = { let q0_ = cx.lptr(32); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                                v146 = cx.i(Ptr::glob(0x77FD8C));
                                                v109 = { let q0_ = cx.lptr(16); let q1_ = cx.li(2160); let q2_ = cx.li(2164); let q3_ = cx.li(2168); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                v45 = { let q0_ = cx.lptr(0); let q1_ = cx.i(cx.lpp(544).add(1276)); let q2_ = cx.i(cx.lpp(544).add(1280)); let q3_ = cx.i(cx.lpp(544).add(1284)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v45, v109, v146, v184);
                                            }
                                            let _ = cx.preinc_i(cx.lptr(2180));
                                        }
                                    }
                                    { let t = 0; cx.set_li(2152, t); }
                                    if ((cx.i(Ptr::gc(0).add(1208)) != 0) && (cx.b(cx.lpp(591).add(4072)) != 0)) {
                                        v185 = { let q0_ = cx.lpp(591).add(240); ov_car_velocity(cx, q0_) };
                                        v46 = { let q0_ = cx.lpp(619).add(240); ov_car_velocity(cx, q0_) };
                                        { let q0_ = cx.lptr(2136); let q1_ = v46; let q2_ = v185; f_19d920(cx, q0_, q1_, q2_) };
                                        v47 = ({ let q0_ = cx.lptr(2136); f_1a0480(cx, q0_) } as f64);
                                        a2 = ((cx.f(Ptr::gc(0).add(1204)) as f64) * (cx.f(Ptr::gc(0).add(1204)) as f64));
                                        if (v47 > a2) {
                                            { let t = 0; cx.set_li(2128, t); }
                                            'lp16: loop {
                                                if !(cx.li(2128) < cx.li(2340)) { break 'lp16; }
                                                'blp16: {
                                                    v48 = vsmath::fabs(cx.f(a4.add((1560_i32).wrapping_mul(cx.li(2128).wrapping_add(cx.li(2360)))).add(1336)));
                                                    a2 = (cx.f(Ptr::gc(0).add(1200)) as f64);
                                                    if (v48 > a2) {
                                                        { let t = 1; cx.set_li(2152, t); }
                                                        break 'lp16;
                                                    }
                                                }
                                                let _ = cx.preinc_i(cx.lptr(2128));
                                            }
                                            if (cx.li(2152) != 0) {
                                                { let q0_ = cx.lptr(2068); let q1_ = cx.lpp(588); f_19e030(cx, q0_, q1_) };
                                                { let q0_ = cx.lptr(2080); let q1_ = cx.lpp(587); f_19e030(cx, q0_, q1_) };
                                                { let q0_ = cx.lptr(2004); let q1_ = cx.lpp(588); f_19e030(cx, q0_, q1_) };
                                                { let q0_ = cx.lptr(2016); let q1_ = cx.lpp(587); f_19e030(cx, q0_, q1_) };
                                                v186 = cx.f(cx.ptr_at(cx.lpp(619).add(3552)));
                                                v49 = { let q0_ = cx.lpp(619).add(240); ov_car_velocity(cx, q0_) };
                                                { let q0_ = cx.lptr(1972); let q1_ = v49; let q2_ = v186; f_19d970(cx, q0_, q1_, q2_) };
                                                v187 = cx.f(cx.ptr_at(cx.lpp(591).add(3552)));
                                                v50 = { let q0_ = cx.lpp(591).add(240); ov_car_velocity(cx, q0_) };
                                                { let q0_ = cx.lptr(1984); let q1_ = v50; let q2_ = v187; f_19d970(cx, q0_, q1_, q2_) };
                                                { let q0_ = cx.lptr(2100); let q1_ = cx.lptr(2068); let q2_ = cx.lptr(1972); f_19d920(cx, q0_, q1_, q2_) };
                                                { let q0_ = cx.lptr(2112); let q1_ = cx.lptr(2080); let q2_ = cx.lptr(1984); f_19d920(cx, q0_, q1_, q2_) };
                                                { let t = 0; cx.set_li(1960, t); }
                                                { let t = 0; cx.set_li(1964, t); }
                                                'lp17: loop {
                                                    if !(cx.li(1964) < cx.i(Ptr::gc(0).add(1208))) { break 'lp17; }
                                                    'blp17: {
                                                        { let q0_ = cx.lptr(2036); let q1_ = cx.lptr(2100); let q2_ = cx.lptr(2068); f_1a38d0(cx, q0_, q1_, q2_) };
                                                        { let q0_ = cx.lptr(2048); let q1_ = cx.lptr(2112); let q2_ = cx.lptr(2080); f_1a38d0(cx, q0_, q1_, q2_) };
                                                        { let q0_ = cx.lptr(2036); let q1_ = 0.5f32; f_19dab0(cx, q0_, q1_) };
                                                        { let q0_ = cx.lptr(2048); let q1_ = 0.5f32; f_19dab0(cx, q0_, q1_) };
                                                        { let q0_ = cx.lpp(588); let q1_ = cx.lptr(2036); f_19e030(cx, q0_, q1_) };
                                                        { let q0_ = cx.lpp(587); let q1_ = cx.lptr(2048); f_19e030(cx, q0_, q1_) };
                                                        if (({ let q0_ = cx.lpp(588); let q1_ = cx.lpp(587); f_38c3d0(cx, q0_, q1_) } as i32) != 0) {
                                                            { let q0_ = cx.lptr(2068); let q1_ = cx.lptr(2036); f_19e030(cx, q0_, q1_) };
                                                            { let q0_ = cx.lptr(2080); let q1_ = cx.lptr(2048); f_19e030(cx, q0_, q1_) };
                                                            { let t = 1; cx.set_li(1960, t); }
                                                        } else {
                                                            { let q0_ = cx.lptr(2100); let q1_ = cx.lptr(2036); f_19e030(cx, q0_, q1_) };
                                                            { let q0_ = cx.lptr(2112); let q1_ = cx.lptr(2048); f_19e030(cx, q0_, q1_) };
                                                            { let t = 0; cx.set_li(1960, t); }
                                                        }
                                                    }
                                                    let _ = cx.preinc_i(cx.lptr(1964));
                                                }
                                                if (cx.li(1960) == 0) {
                                                    { let p = cx.lptr(1956); let t = ((1.0f64 / (((cx.i(Ptr::gc(0).add(1208)) as f32) + 1.0f32) as f64)) as f32); cx.set_f(p, t); }
                                                    { let q0_ = cx.lpp(588); let q1_ = cx.lptr(1972); let q2_ = cx.f(cx.lptr(1956)); f_1a0390(cx, q0_, q1_, q2_) };
                                                    { let q0_ = cx.lpp(587); let q1_ = cx.lptr(1984); let q2_ = cx.f(cx.lptr(1956)); f_1a0390(cx, q0_, q1_, q2_) };
                                                }
                                                { let t = a5.wrapping_sub(cx.li(2360)); cx.set_li(2340, t); }
                                                { let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(2360))); let q1_ = cx.lptr(2340); let q2_ = cx.lpp(588); let q3_ = cx.lpp(587); let q4_ = 0.0f32; f_38cf90(cx, q0_, q1_, q2_, q3_, q4_) };
                                                { let t = 0; cx.set_li(1932, t); }
                                                'lp18: loop {
                                                    if !(cx.li(1932) < cx.li(2340)) { break 'lp18; }
                                                    'blp18: {
                                                        { let q0_ = cx.lptr(1940); let q1_ = cx.lpp(588); let q2_ = cx.lptr(2004); f_19d920(cx, q0_, q1_, q2_) };
                                                        { let q0_ = cx.lptr(1940); let q1_ = cx.lpp(587); f_1a3920(cx, q0_, q1_) };
                                                        { let q0_ = cx.lptr(1940); let q1_ = cx.lptr(2016); f_1a3880(cx, q0_, q1_) };
                                                        v51 = (1560_i32).wrapping_mul(cx.li(1932).wrapping_add(cx.li(2360)));
                                                        a2 = ({ let q0_ = a4.add(v51).add(1332); let q1_ = cx.lptr(1940); f_19d9c0(cx, q0_, q1_) } + (cx.f(a4.add(v51).add(1288)) as f64));
                                                        { let p = a4.add((1560_i32).wrapping_mul(cx.li(1932).wrapping_add(cx.li(2360)))).add(1288); let t = (a2 as f32); cx.set_f(p, t); }
                                                    }
                                                    let _ = cx.preinc_i(cx.lptr(1932));
                                                }
                                            }
                                        }
                                    }
                                    { let t = 0; cx.set_li(1928, t); }
                                    'lp19: loop {
                                        if !(cx.li(1928) < cx.li(2340)) { break 'lp19; }
                                        'blp19: {
                                            { let t = a4.add((1560_i32).wrapping_mul(cx.li(1928).wrapping_add(cx.li(2360)))); cx.set_lpp(481, t); }
                                            { let p = cx.lpp(481).add(216); let t = cx.li(2480); cx.set_i(p, t); }
                                            { let p = cx.lpp(481).add(220); let t = cx.li(2368); cx.set_i(p, t); }
                                            { let p = cx.lpp(481).add(4); let t = cx.lpp(619).add(240); cx.set_ptr(p, t); }
                                            { let p = cx.lpp(481).add(8); let t = cx.lpp(591).add(240); cx.set_ptr(p, t); }
                                            if (cx.b(cx.lpp(591).add(4072)) != 0) {
                                                { let p = cx.lpp(481).add(1320); let t = cx.i(cx.lpp(623).add(24328)); cx.set_i(p, t); }
                                                { let p = cx.lpp(481).add(1316); let t = 1065353216; cx.set_i(p, t); }
                                            }
                                            if ((cx.li(2356) == 0) || (cx.b(cx.lpp(619).add(4073)) != 0)) {
                                                { let p = cx.lpp(481).add(1540); let t = cx.i(Ptr::gc(0).add(1184)); cx.set_i(p, t); }
                                            } else {
                                                { let p = cx.lpp(481).add(1540); let t = cx.i(Ptr::gc(0).add(1188)); cx.set_i(p, t); }
                                            }
                                            { let p = cx.lpp(481).add(1544); let t = cx.i(Ptr::gc(0).add(1188)); cx.set_i(p, t); }
                                            { let p = cx.lpp(481).add(1272); let t = cx.i(Ptr::gc(0).add(1196)); cx.set_i(p, t); }
                                            if ((cx.li(2152) != 0) && (cx.f(cx.lpp(481).add(1272)) < 0.0f32)) {
                                                { let p = cx.lpp(481).add(1272); let t = 0; cx.set_i(p, t); }
                                            }
                                            { let p = cx.lpp(481).add(1268); let t = cx.i(Ptr::gc(0).add(1192)); cx.set_i(p, t); }
                                            if (cx.f(Ptr::gc(0).add(1220)) != 0.0f32) {
                                                { let p = cx.lpp(481).add(212); let t = 1; cx.set_i(p, t); }
                                                { let q0_ = cx.lpp(481).add(188); let q1_ = cx.lpp(481).add(1276); let q2_ = cx.lpp(588); f_19d920(cx, q0_, q1_, q2_) };
                                                { let q0_ = cx.lpp(481).add(200); let q1_ = cx.lpp(481).add(1276); let q2_ = cx.lpp(587); f_19d920(cx, q0_, q1_, q2_) };
                                                { let t = 0; let o = cx.li(1900); cx.set_li(1900, (o & !(255 << 24)) | ((t & 255) << 24)); }
                                                v188 = ({ let q0_ = cx.lpp(481).add(1332); let q1_ = cx.lpp(588).add(28); f_19d9c0(cx, q0_, q1_) } as f32);
                                                if ((cx.f(Ptr::gc(0).add(1216)) as f64) > vsmath::fabs(v188)) {
                                                    v189 = ({ let q0_ = cx.lpp(481).add(1332); let q1_ = cx.lpp(588).add(40); f_19d9c0(cx, q0_, q1_) } as f32);
                                                    if ((cx.f(Ptr::gc(0).add(1216)) as f64) > vsmath::fabs(v189)) {
                                                        { let t = 1; let o = cx.li(1900); cx.set_li(1900, (o & !(255 << 24)) | ((t & 255) << 24)); }
                                                        { let q0_ = cx.lptr(1908); let q1_ = cx.lpp(481).add(188); let q2_ = cx.lpp(588).add(16); f_1a39d0(cx, q0_, q1_, q2_) };
                                                        { let p = cx.lptr(1896); let t = { let q0_ = cx.lptr(1908); f_19d9f0(cx, q0_) }; cx.set_f(p, t); }
                                                        if (cx.f(cx.lptr(1896)) <= cx.f(Ptr::gc(0).add(1220))) {
                                                            if (((((cx.f(Ptr::gc(0).add(1228)) > cx.f(cx.lptr(1896))) && (cx.b(cx.lpp(619).add(4073)) == 0)) && (cx.i(cx.lpp(619).add(5084)) == 2)) && (cx.b(cx.lpp(591).add(4072)) != 0)) && (cx.f(cx.lptr(1896)) > 0.0f32)) {
                                                                { let q0_ = cx.lpp(481).add(188); let q1_ = cx.lptr(1908); let q2_ = ((((cx.f(Ptr::gc(0).add(1228)) / cx.f(cx.lptr(1896))) as f64) - 1.0f64) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                                                            }
                                                        } else {
                                                            { let q0_ = cx.lpp(481).add(188); let q1_ = cx.lptr(1908); let q2_ = ((((cx.f(Ptr::gc(0).add(1220)) / cx.f(cx.lptr(1896))) as f64) - 1.0f64) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                                                        }
                                                        if (cx.f(cx.lpp(481).add(192)) <= cx.f(Ptr::gc(0).add(1232))) {
                                                            if ((0.0f32 - cx.f(Ptr::gc(0).add(1232))) > cx.f(cx.lpp(481).add(192))) {
                                                                { let p = cx.lpp(481).add(192); let t = ((0.0f64 - (cx.f(Ptr::gc(0).add(1232)) as f64)) as f32); cx.set_f(p, t); }
                                                            }
                                                        } else {
                                                            { let p = cx.lpp(481).add(192); let t = cx.i(Ptr::gc(0).add(1232)); cx.set_i(p, t); }
                                                        }
                                                    }
                                                }
                                                v190 = ({ let q0_ = cx.lpp(481).add(1332); let q1_ = cx.lpp(587).add(28); f_19d9c0(cx, q0_, q1_) } as f32);
                                                a2 = vsmath::fabs(v190);
                                                if ((cx.f(Ptr::gc(0).add(1216)) as f64) > a2) {
                                                    v191 = ({ let q0_ = cx.lpp(481).add(1332); let q1_ = cx.lpp(587).add(40); f_19d9c0(cx, q0_, q1_) } as f32);
                                                    a2 = vsmath::fabs(v191);
                                                    if ((cx.f(Ptr::gc(0).add(1216)) as f64) > a2) {
                                                        if (cx.b(cx.lpp(591).add(4072)) != 0) {
                                                            { let p = cx.lpp(481).add(204); let t = 0; cx.set_i(p, t); }
                                                            { let q0_ = cx.lptr(1908); let q1_ = cx.lpp(481).add(200); let q2_ = cx.lpp(587).add(16); f_1a39d0(cx, q0_, q1_, q2_) };
                                                            a2 = ({ let q0_ = cx.lptr(1908); f_19d9f0(cx, q0_) } as f64);
                                                            { let p = cx.lptr(1892); let t = (a2 as f32); cx.set_f(p, t); }
                                                            if ((cx.f(cx.lptr(1892)) > cx.f(Ptr::gc(0).add(1224))) || (cx.f(cx.lptr(1892)) > 0.0f32)) {
                                                                { let q0_ = cx.lpp(481).add(200); let q1_ = cx.lptr(1908); let q2_ = ((((cx.f(Ptr::gc(0).add(1224)) / cx.f(cx.lptr(1892))) as f64) - 1.0f64) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                                                            }
                                                            if (((cx.li(1900) >> 24) & 255) != 0) {
                                                                a2 = { let q0_ = cx.lpp(619).add(240); f_369d10(cx, q0_) };
                                                                if ((a2 < 2000.0f64) && ({ let p = { let q0_ = cx.lpp(619).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }.add(28); cx.i(p) } == 0)) {
                                                                    { let q0_ = cx.lptr(1876); let q1_ = cx.lpp(619).add(272); let q2_ = cx.lpp(591).add(272); f_19d920(cx, q0_, q1_, q2_) };
                                                                    { let t = cx.lpp(481); cx.set_lpp(582, t); }
                                                                    { let q0_ = cx.lptr(2312); let q1_ = cx.lpp(619).add(272); f_19e030(cx, q0_, q1_) };
                                                                    { let q0_ = cx.lptr(2292); let q1_ = cx.lpp(591).add(272); f_19e030(cx, q0_, q1_) };
                                                                    { let p = { let q0_ = cx.lpp(619).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }.add(20); let t = cx.i(Ptr::gc(0).add(1296)); cx.set_i(p, t); }
                                                                    v192 = ({ let q0_ = cx.lptr(1876); let q1_ = cx.lpp(481).add(1332); f_19d9c0(cx, q0_, q1_) } as f32);
                                                                    v52 = vsmath::fabs(v192);
                                                                    a2 = (cx.f(Ptr::gc(0).add(1264)) as f64);
                                                                    if (v52 <= a2) {
                                                                        if ({ let p = { let q0_ = cx.lpp(619).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }.add(24); cx.i(p) } == 0) {
                                                                            { let p = { let q0_ = cx.lpp(619).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }.add(24); let t = cx.i(Ptr::gc(0).add(1288)); cx.set_i(p, t); }
                                                                        }
                                                                    } else {
                                                                        { let p = { let q0_ = cx.lpp(619).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }.add(24); let t = 0; cx.set_i(p, t); }
                                                                    }
                                                                }
                                                            }
                                                        } else {
                                                            { let p = cx.lpp(481).add(204); let t = 0; cx.set_i(p, t); }
                                                            { let q0_ = cx.lptr(1908); let q1_ = cx.lpp(481).add(200); let q2_ = cx.lpp(587).add(16); f_1a39d0(cx, q0_, q1_, q2_) };
                                                            a2 = ({ let q0_ = cx.lptr(1908); f_19d9f0(cx, q0_) } as f64);
                                                            { let p = cx.lptr(1868); let t = (a2 as f32); cx.set_f(p, t); }
                                                            if (cx.f(cx.lptr(1868)) <= cx.f(Ptr::gc(0).add(1220))) {
                                                                if (((cx.i(cx.lpp(591).add(5084)) == 2) && (cx.f(Ptr::gc(0).add(1228)) > cx.f(cx.lptr(1868)))) && (cx.f(cx.lptr(1868)) > 0.0f32)) {
                                                                    { let q0_ = cx.lpp(481).add(200); let q1_ = cx.lptr(1908); let q2_ = ((((cx.f(Ptr::gc(0).add(1228)) / cx.f(cx.lptr(1868))) as f64) - 1.0f64) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                                                                }
                                                            } else {
                                                                { let q0_ = cx.lpp(481).add(200); let q1_ = cx.lptr(1908); let q2_ = ((((cx.f(Ptr::gc(0).add(1220)) / cx.f(cx.lptr(1868))) as f64) - 1.0f64) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                                                            }
                                                            if (cx.f(cx.lpp(481).add(204)) <= cx.f(Ptr::gc(0).add(1232))) {
                                                                if ((0.0f32 - cx.f(Ptr::gc(0).add(1232))) > cx.f(cx.lpp(481).add(204))) {
                                                                    { let p = cx.lpp(481).add(204); let t = ((0.0f64 - (cx.f(Ptr::gc(0).add(1232)) as f64)) as f32); cx.set_f(p, t); }
                                                                }
                                                            } else {
                                                                { let p = cx.lpp(481).add(204); let t = cx.i(Ptr::gc(0).add(1232)); cx.set_i(p, t); }
                                                            }
                                                        }
                                                    }
                                                }
                                                if (cx.b(cx.lpp(591).add(4072)) != 0) {
                                                    if (cx.f(cx.lpp(481).add(204)) <= cx.f(Ptr::gc(0).add(1232))) {
                                                        if ((0.0f32 - cx.f(Ptr::gc(0).add(1232))) > cx.f(cx.lpp(481).add(204))) {
                                                            { let p = cx.lpp(481).add(204); let t = ((0.0f64 - (cx.f(Ptr::gc(0).add(1232)) as f64)) as f32); cx.set_f(p, t); }
                                                        }
                                                    } else {
                                                        { let p = cx.lpp(481).add(204); let t = cx.i(Ptr::gc(0).add(1232)); cx.set_i(p, t); }
                                                    }
                                                }
                                                if (cx.b(cx.lpp(619).add(4073)) != 0) {
                                                    { let p = cx.lpp(481).add(192); let t = 0; cx.set_i(p, t); }
                                                }
                                            }
                                        }
                                        let _ = cx.preinc_i(cx.lptr(1928));
                                    }
                                    if (cx.li(2340) != 0) {
                                        { let p = cx.lpp(619).add(4856); let t = 0; cx.set_i(p, t); }
                                        { let p = cx.lpp(591).add(4856); let t = 0; cx.set_i(p, t); }
                                    }
                                    { let t = cx.li(2360).wrapping_add(cx.li(2340)); cx.set_li(2360, t); }
                                }
                            }
                            if ((cx.li(2360) != 0) && (cx.i(Ptr::gc(0).add(2076)) != 0)) {
                                { let q0_ = cx.lptr(1688); f_1b42f0(cx, q0_) };
                                { let t = cx.i(cx.ptr_at(cx.lpp(619).add(3552))); cx.set_li(1688, t); }
                                { let t = cx.i(Ptr::gc(0).add(1152)); cx.set_li(1764, t); }
                                { let t = cx.i(Ptr::gc(0).add(1156)); cx.set_li(1696, t); }
                                { let t = cx.i(Ptr::gc(0).add(1160)); cx.set_li(1692, t); }
                                { let t = cx.i(Ptr::gc(0).add(1164)); cx.set_li(1768, t); }
                                { let t = cx.i(Ptr::gc(0).add(1168)); cx.set_li(1812, t); }
                                { let t = cx.i(Ptr::gc(0).add(1172)); cx.set_li(1776, t); }
                                { let t = cx.i(Ptr::gc(0).add(1176)); cx.set_li(1780, t); }
                                { let t = cx.i(Ptr::gc(0).add(1180)); cx.set_li(1772, t); }
                                { let q0_ = cx.lptr(1724); let q1_ = a6; f_19e030(cx, q0_, q1_) };
                                { let q0_ = cx.lptr(1700); let q1_ = Ptr::gc(0); f_19e030(cx, q0_, q1_) };
                                { let t = -1; cx.set_li(1680, t); }
                                { let t = 0; cx.set_li(1676, t); }
                                { let t = 0; cx.set_li(1672, t); }
                                { let t = 0; let o = cx.li(1648); cx.set_li(1648, (o & !(255 << 24)) | ((t & 255) << 24)); }
                                if (cx.b(cx.lpp(591).add(4072)) != 0) {
                                    v193 = { let q0_ = cx.lpp(619).add(240); ov_car_velocity(cx, q0_) };
                                    v53 = { let q0_ = cx.lpp(591).add(240); ov_car_velocity(cx, q0_) };
                                    { let q0_ = cx.lptr(1632); let q1_ = v53; let q2_ = v193; f_19d920(cx, q0_, q1_, q2_) };
                                    if ((cx.f(Ptr::gc(0).add(1420)) > 0.0f32) && (cx.i(cx.lpp(591).add(4408)) == 0)) {
                                        v54 = ({ let q0_ = cx.lptr(1632); f_1a0480(cx, q0_) } as f64);
                                        a2 = ((cx.f(Ptr::gc(0).add(1408)) as f64) * (cx.f(Ptr::gc(0).add(1408)) as f64));
                                        if (v54 > a2) {
                                            { let q0_ = cx.lpp(591).add(240); let q1_ = cx.lptr(1656); f_1b42b0(cx, q0_, q1_) };
                                            { let t = 1; let o = cx.li(1648); cx.set_li(1648, (o & !(255 << 24)) | ((t & 255) << 24)); }
                                        }
                                    }
                                    { let t = 0; cx.set_li(1624, t); }
                                    'lp20: loop {
                                        if !(cx.li(1624) < cx.li(2360)) { break 'lp20; }
                                        'blp20: {
                                            v55 = { let q0_ = cx.lpp(591); let q1_ = 0; f_1bb200(cx, q0_, q1_) };
                                            { let p = cx.lptr(1672); let t = ({ let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(1624))).add(1332); let q1_ = v55; f_19d9c0(cx, q0_, q1_) } as f32); cx.set_f(p, t); }
                                            a2 = vsmath::fabs(cx.f(a4.add((1560_i32).wrapping_mul(cx.li(1624))).add(1336)));
                                            if (((cx.f(Ptr::gc(0).add(1384)) as f64) > a2) && (cx.f(cx.lptr(1672)) > cx.f(Ptr::gc(0).add(1388)))) {
                                                v56 = { let q0_ = cx.lpp(591); let q1_ = 1; f_1bb200(cx, q0_, q1_) };
                                                v194 = ({ let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(1624))).add(1332); let q1_ = v56; f_19d9c0(cx, q0_, q1_) } as f32);
                                                a2 = vsmath::fabs(v194);
                                                if ((cx.f(Ptr::gc(0).add(1388)) as f64) > a2) {
                                                    { let p = cx.lptr(1676); let t = ({ let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(1624))).add(1332); let q1_ = cx.lptr(1632); f_19d9c0(cx, q0_, q1_) } as f32); cx.set_f(p, t); }
                                                    v57 = vsmath::fabs(cx.f(cx.lptr(1676)));
                                                    a2 = (cx.f(Ptr::gc(0).add(1392)) as f64);
                                                    if (v57 >= a2) {
                                                        { let t = cx.li(1624); cx.set_li(1680, t); }
                                                        break 'lp20;
                                                    }
                                                }
                                            }
                                        }
                                        let _ = cx.preinc_i(cx.lptr(1624));
                                    }
                                }
                                { let q0_ = a4; let q1_ = cx.li(2360); let q2_ = Ptr::NULL; let q3_ = 0; let q4_ = Ptr::NULL; let q5_ = 0; let q6_ = cx.lptr(1688); f_389630(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
                                if (cx.lpp(582).bits() != 0) {
                                    { let t = cx.lpp(582); cx.set_lpp(405, t); }
                                    { let q0_ = cx.lptr(1604); let q1_ = cx.ptr_at(cx.lpp(582).add(8)).add(64); let q2_ = cx.ptr_at(cx.lpp(582).add(4)).add(64); f_19d920(cx, q0_, q1_, q2_) };
                                    { let q0_ = cx.lptr(1584); let q1_ = cx.lptr(2292); let q2_ = cx.lptr(2312); f_19d920(cx, q0_, q1_, q2_) };
                                    { let q0_ = cx.lptr(1564); let q1_ = cx.ptr_at(cx.lpp(405).add(4)).add(32); let q2_ = cx.lptr(2312); f_19d920(cx, q0_, q1_, q2_) };
                                    { let t = 0; cx.set_li(1568, t); }
                                    { let p = cx.lptr(1556); let t = { let q0_ = cx.lptr(1564); f_19d9f0(cx, q0_) }; cx.set_f(p, t); }
                                    { let q0_ = cx.lptr(1540); let q1_ = cx.lptr(1604); let q2_ = cx.lptr(1584); f_1a03f0(cx, q0_, q1_, q2_) };
                                    { let t = { let q0_ = cx.lpp(619).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }; cx.set_lpp(383, t); }
                                    if (cx.f(cx.lptr(1544)) <= 0.0f32) {
                                        v62 = { let q0_ = cx.ptr_at(cx.lpp(405).add(4)); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                                        v63 = { let q0_ = cx.lptr(1584); let q1_ = v62; f_19d9c0(cx, q0_, q1_) };
                                        a2 = 0.0f64;
                                        if (v63 <= 0.0f64) {
                                            { let p = cx.lptr(1504); let t = (((cx.f(cx.lptr(1556)) as f64) * (cx.f(Ptr::gc(0).add(1248)) as f64)) as f32); cx.set_f(p, t); }
                                            { let p = cx.lptr(1500); let t = (((cx.f(cx.lptr(1556)) as f64) * (cx.f(Ptr::gc(0).add(1240)) as f64)) as f32); cx.set_f(p, t); }
                                            if (cx.f(cx.lptr(1504)) > cx.f(Ptr::gc(0).add(1260))) {
                                                { let t = cx.i(Ptr::gc(0).add(1260)); cx.set_li(1504, t); }
                                            }
                                            if (cx.f(cx.lptr(1500)) > cx.f(Ptr::gc(0).add(1256))) {
                                                { let t = cx.i(Ptr::gc(0).add(1256)); cx.set_li(1500, t); }
                                            }
                                            { let p = cx.lpp(383).add(28); let t = cx.i(Ptr::gc(0).add(1276)).wrapping_neg().wrapping_sub(cx.i(Ptr::gc(0).add(1280))); cx.set_i(p, t); }
                                            if (cx.i(cx.lpp(383).add(24)) != 0) {
                                                { let p = cx.lpp(383).add(28); let t = cx.i(p).wrapping_sub(cx.i(cx.lpp(383).add(24))); cx.set_i(p, t); }
                                                { let p = cx.lpp(383).add(32); let t = ((0.0f64 - (cx.f(Ptr::gc(0).add(1272)) as f64)) as f32); cx.set_f(p, t); }
                                            } else {
                                                { let p = cx.ptr_at(cx.lpp(405).add(4)).add(52); let t = ((0.0f64 - (cx.f(cx.lptr(1504)) as f64)) as f32); cx.set_f(p, t); }
                                                v198 = ((0.0f64 - (cx.f(cx.lptr(1500)) as f64)) as f32);
                                                v65 = { let q0_ = cx.ptr_at(cx.lpp(405).add(4)); let q1_ = 2; ov_car_rot_row(cx, q0_, q1_) };
                                                { let q0_ = cx.ptr_at(cx.lpp(405).add(4)).add(32); let q1_ = v65; let q2_ = v198; f_1a0390(cx, q0_, q1_, q2_) };
                                                { let p = cx.lpp(383).add(32); let t = ((0.0f64 - (cx.f(Ptr::gc(0).add(1268)) as f64)) as f32); cx.set_f(p, t); }
                                            }
                                        } else {
                                            { let p = cx.lptr(1512); let t = (((cx.f(cx.lptr(1556)) as f64) * (cx.f(Ptr::gc(0).add(1252)) as f64)) as f32); cx.set_f(p, t); }
                                            { let p = cx.lptr(1508); let t = (((cx.f(cx.lptr(1556)) as f64) * (cx.f(Ptr::gc(0).add(1244)) as f64)) as f32); cx.set_f(p, t); }
                                            if (cx.f(cx.lptr(1512)) > cx.f(Ptr::gc(0).add(1260))) {
                                                { let t = cx.i(Ptr::gc(0).add(1260)); cx.set_li(1512, t); }
                                            }
                                            if (cx.f(cx.lptr(1508)) > cx.f(Ptr::gc(0).add(1256))) {
                                                { let t = cx.i(Ptr::gc(0).add(1256)); cx.set_li(1508, t); }
                                            }
                                            { let p = cx.lpp(383).add(28); let t = cx.i(Ptr::gc(0).add(1284)).wrapping_add(cx.i(Ptr::gc(0).add(1276))); cx.set_i(p, t); }
                                            if (cx.i(cx.lpp(383).add(24)) != 0) {
                                                { let p = cx.lpp(383).add(28); let t = cx.i(p).wrapping_add(cx.i(cx.lpp(383).add(24))); cx.set_i(p, t); }
                                                { let p = cx.lpp(383).add(32); let t = cx.i(Ptr::gc(0).add(1272)); cx.set_i(p, t); }
                                            } else {
                                                { let p = cx.ptr_at(cx.lpp(405).add(4)).add(52); let t = ((0.0f64 - (cx.f(cx.lptr(1512)) as f64)) as f32); cx.set_f(p, t); }
                                                v197 = cx.f(cx.lptr(1508));
                                                v64 = { let q0_ = cx.ptr_at(cx.lpp(405).add(4)); let q1_ = 2; ov_car_rot_row(cx, q0_, q1_) };
                                                { let q0_ = cx.ptr_at(cx.lpp(405).add(4)).add(32); let q1_ = v64; let q2_ = v197; f_1a0390(cx, q0_, q1_, q2_) };
                                                { let p = cx.lpp(383).add(32); let t = cx.i(Ptr::gc(0).add(1268)); cx.set_i(p, t); }
                                            }
                                        }
                                    } else {
                                        v58 = { let q0_ = cx.ptr_at(cx.lpp(405).add(4)); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                                        v59 = { let q0_ = cx.lptr(1584); let q1_ = v58; f_19d9c0(cx, q0_, q1_) };
                                        a2 = 0.0f64;
                                        if (v59 <= 0.0f64) {
                                            { let p = cx.lptr(1520); let t = (((cx.f(cx.lptr(1556)) as f64) * (cx.f(Ptr::gc(0).add(1248)) as f64)) as f32); cx.set_f(p, t); }
                                            { let p = cx.lptr(1516); let t = (((cx.f(cx.lptr(1556)) as f64) * (cx.f(Ptr::gc(0).add(1240)) as f64)) as f32); cx.set_f(p, t); }
                                            if (cx.f(cx.lptr(1520)) > cx.f(Ptr::gc(0).add(1260))) {
                                                { let t = cx.i(Ptr::gc(0).add(1260)); cx.set_li(1520, t); }
                                            }
                                            if (cx.f(cx.lptr(1516)) > cx.f(Ptr::gc(0).add(1256))) {
                                                { let t = cx.i(Ptr::gc(0).add(1256)); cx.set_li(1516, t); }
                                            }
                                            { let p = cx.lpp(383).add(28); let t = cx.i(Ptr::gc(0).add(1280)).wrapping_add(cx.i(Ptr::gc(0).add(1276))); cx.set_i(p, t); }
                                            if (cx.i(cx.lpp(383).add(24)) != 0) {
                                                { let p = cx.lpp(383).add(28); let t = cx.i(p).wrapping_add(cx.i(cx.lpp(383).add(24))); cx.set_i(p, t); }
                                                { let p = cx.lpp(383).add(32); let t = cx.i(Ptr::gc(0).add(1272)); cx.set_i(p, t); }
                                            } else {
                                                { let p = cx.ptr_at(cx.lpp(405).add(4)).add(52); let t = cx.li(1520); cx.set_i(p, t); }
                                                v196 = cx.f(cx.lptr(1516));
                                                v61 = { let q0_ = cx.ptr_at(cx.lpp(405).add(4)); let q1_ = 2; ov_car_rot_row(cx, q0_, q1_) };
                                                { let q0_ = cx.ptr_at(cx.lpp(405).add(4)).add(32); let q1_ = v61; let q2_ = v196; f_1a0390(cx, q0_, q1_, q2_) };
                                                { let p = cx.lpp(383).add(32); let t = cx.i(Ptr::gc(0).add(1268)); cx.set_i(p, t); }
                                            }
                                        } else {
                                            { let p = cx.lptr(1528); let t = (((cx.f(cx.lptr(1556)) as f64) * (cx.f(Ptr::gc(0).add(1252)) as f64)) as f32); cx.set_f(p, t); }
                                            { let p = cx.lptr(1524); let t = (((cx.f(cx.lptr(1556)) as f64) * (cx.f(Ptr::gc(0).add(1244)) as f64)) as f32); cx.set_f(p, t); }
                                            if (cx.f(cx.lptr(1528)) > cx.f(Ptr::gc(0).add(1260))) {
                                                { let t = cx.i(Ptr::gc(0).add(1260)); cx.set_li(1528, t); }
                                            }
                                            if (cx.f(cx.lptr(1524)) > cx.f(Ptr::gc(0).add(1256))) {
                                                { let t = cx.i(Ptr::gc(0).add(1256)); cx.set_li(1524, t); }
                                            }
                                            { let p = cx.lpp(383).add(28); let t = cx.i(Ptr::gc(0).add(1276)).wrapping_neg().wrapping_sub(cx.i(Ptr::gc(0).add(1284))); cx.set_i(p, t); }
                                            if (cx.i(cx.lpp(383).add(24)) != 0) {
                                                { let p = cx.lpp(383).add(28); let t = cx.i(p).wrapping_sub(cx.i(cx.lpp(383).add(24))); cx.set_i(p, t); }
                                                { let p = cx.lpp(383).add(32); let t = ((0.0f64 - (cx.f(Ptr::gc(0).add(1272)) as f64)) as f32); cx.set_f(p, t); }
                                            } else {
                                                { let p = cx.ptr_at(cx.lpp(405).add(4)).add(52); let t = cx.li(1528); cx.set_i(p, t); }
                                                v195 = ((0.0f64 - (cx.f(cx.lptr(1524)) as f64)) as f32);
                                                v60 = { let q0_ = cx.ptr_at(cx.lpp(405).add(4)); let q1_ = 2; ov_car_rot_row(cx, q0_, q1_) };
                                                { let q0_ = cx.ptr_at(cx.lpp(405).add(4)).add(32); let q1_ = v60; let q2_ = v195; f_1a0390(cx, q0_, q1_, q2_) };
                                                { let p = cx.lpp(383).add(32); let t = ((0.0f64 - (cx.f(Ptr::gc(0).add(1268)) as f64)) as f32); cx.set_f(p, t); }
                                            }
                                        }
                                    }
                                }
                                if ((cx.b(cx.lpp(591).add(4072)) != 0) && (cx.li(1680) != -1)) {
                                    { let p = cx.lptr(1496); let t = ({ let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(1680))).add(1332); let q1_ = cx.lpp(591).add(272); f_19d9c0(cx, q0_, q1_) } as f32); cx.set_f(p, t); }
                                    a2 = vsmath::fabs((((cx.f(cx.lptr(1496)) as f64) - (cx.f(cx.lptr(1676)) as f64)) as f32));
                                    { let p = cx.lptr(1492); let t = (a2 as f32); cx.set_f(p, t); }
                                    if (cx.f(cx.lptr(1492)) > cx.f(Ptr::gc(0).add(1392))) {
                                        { let p = cx.lptr(1488); let t = (((((cx.f(cx.lptr(1492)) - cx.f(Ptr::gc(0).add(1392))) * cx.f(cx.lptr(1672))) as f64) / ((cx.f(Ptr::gc(0).add(1396)) - cx.f(Ptr::gc(0).add(1392))) as f64)) as f32); cx.set_f(p, t); }
                                        if (cx.f(cx.lptr(1488)) > 1.0f32) {
                                            { let t = 1065353216; cx.set_li(1488, t); }
                                        }
                                        { let p = cx.lptr(1488); let t = (((cx.f(cx.lptr(1488)) as f64) * (cx.f(Ptr::gc(0).add(1404)) as f64)) as f32); cx.set_f(p, t); }
                                        v199 = ((0.0f64 - (cx.f(cx.lptr(1488)) as f64)) as f32);
                                        v66 = { let q0_ = cx.lpp(591).add(240); let q1_ = 2; ov_car_rot_row(cx, q0_, q1_) };
                                        { let q0_ = cx.lpp(591).add(288); let q1_ = v66; let q2_ = v199; f_1a0390(cx, q0_, q1_, q2_) };
                                        { let q0_ = cx.lptr(1472); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(1680))).add(1276); let q2_ = cx.lpp(591).add(304); f_19d920(cx, q0_, q1_, q2_) };
                                        v67 = ({ let q0_ = cx.lptr(1472); f_19d9f0(cx, q0_) } as f64);
                                        a2 = ((v67 * (cx.f(cx.lptr(1488)) as f64)) + (cx.f(cx.lpp(591).add(276)) as f64));
                                        { let p = cx.lpp(591).add(276); let t = (a2 as f32); cx.set_f(p, t); }
                                    }
                                }
                                if ((((cx.li(1648) >> 24) & 255) != 0) && (cx.i(cx.lpp(591).add(4756)) != 1)) {
                                    v68 = { let q0_ = cx.lpp(591).add(240); ov_car_velocity(cx, q0_) };
                                    { let q0_ = cx.lptr(1452); let q1_ = cx.lptr(1656); let q2_ = v68; f_19d920(cx, q0_, q1_, q2_) };
                                    a2 = ({ let q0_ = cx.lptr(1452); f_19d9f0(cx, q0_) } as f64);
                                    { let p = cx.lpp(591).add(4428); let t = (a2 as f32); cx.set_f(p, t); }
                                    if (cx.f(cx.lpp(591).add(4428)) > cx.f(Ptr::gc(0).add(1412))) {
                                        { let p = cx.lpp(591).add(4424); let t = 0; cx.set_i(p, t); }
                                        { let q0_ = cx.lpp(591).add(4412); let q1_ = cx.lptr(1452); let q2_ = ((1.0f64 / (cx.f(cx.lpp(591).add(4428)) as f64)) as f32); f_19d970(cx, q0_, q1_, q2_) };
                                        v69 = { let q0_ = cx.lpp(591).add(240); let q1_ = 2; ov_car_rot_row(cx, q0_, q1_) };
                                        v200 = (f_19d9c0(cx, v69, a4.add(1332)) as f32);
                                        a2 = (1.0f64 - vsmath::fabs(v200));
                                        { let p = cx.lptr(1444); let t = (a2 as f32); cx.set_f(p, t); }
                                        { let p = cx.lptr(1440); let t = (((((1.0f32 - cx.f(Ptr::gc(0).add(1452))) * cx.f(cx.lptr(1444))) as f64) + (cx.f(Ptr::gc(0).add(1452)) as f64)) as f32); cx.set_f(p, t); }
                                        { let q0_ = cx.lpp(591).add(4412); let q1_ = (((cx.f(cx.lptr(1440)) as f64) * (cx.f(Ptr::gc(0).add(1448)) as f64)) as f32); f_19dab0(cx, q0_, q1_) };
                                        { let p = cx.lpp(591).add(4432); let t = (((((1.0f32 - cx.f(Ptr::gc(0).add(1444))) * cx.f(cx.lptr(1444))) as f64) + (cx.f(Ptr::gc(0).add(1444)) as f64)) as f32); cx.set_f(p, t); }
                                        { let p = cx.lpp(591).add(4408); let t = cx.i(Ptr::gc(0).add(1428)); cx.set_i(p, t); }
                                    }
                                }
                                { let t = 0; cx.set_li(1436, t); }
                                'lp21: loop {
                                    if !(cx.li(1436) < cx.li(2360)) { break 'lp21; }
                                    'blp21: {
                                        { let t = a4.add((1560_i32).wrapping_mul(cx.li(1436))); cx.set_lpp(358, t); }
                                        if (cx.f(cx.lpp(358).add(236)) != 0.0f32) {
                                            v70 = vsmath::fabs(cx.f(cx.lpp(358).add(236)));
                                            { let p = cx.lptr(1428); let t = (v70 as f32); cx.set_f(p, t); }
                                            { let q0_ = cx.lptr(1344); f_238b0(cx, q0_) };
                                            { let t = cx.li(2480); cx.set_li(1344, t); }
                                            { let t = cx.li(2368); cx.set_li(1348, t); }
                                            { let t = cx.li(1428); cx.set_li(1384, t); }
                                            { let q0_ = cx.lptr(1352); let q1_ = cx.i(cx.lpp(358).add(1276)); let q2_ = cx.i(cx.lpp(358).add(1280)); let q3_ = cx.i(cx.lpp(358).add(1284)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                            { let q0_ = cx.lptr(1368); let q1_ = cx.i(cx.lpp(358).add(1332)); let q2_ = cx.i(cx.lpp(358).add(1336)); let q3_ = cx.i(cx.lpp(358).add(1340)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                            { let t = 0; cx.set_li(1388, t); }
                                            { let t = 1084227584; cx.set_li(1416, t); }
                                            { let t = -1; let o = cx.li(1420); cx.set_li(1420, (o & !(255 << 0)) | ((t & 255) << 0)); }
                                            { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(623)); let q1_ = v70; let q2_ = cx.li(2480); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.i(p) }.wrapping_add(240); cx.set_li(1392, t); }
                                            { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(623)); let q1_ = v71; let q2_ = cx.li(2368); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.i(p) }.wrapping_add(240); cx.set_li(1396, t); }
                                            { let q0_ = cx.lpp(623); let q1_ = cx.lpp(358); let q2_ = cx.lptr(1400); f_1b39b0(cx, q0_, q1_, q2_) };
                                            { let q0_ = Ptr::glob(0x6FAA34); let q1_ = 0; let q2_ = cx.lptr(1344); ov_message(cx, q0_, q1_, q2_) };
                                        }
                                    }
                                    let _ = cx.preinc_i(cx.lptr(1436));
                                }
                            }
                            { let t = 0; cx.set_li(2360, t); }
                        }
                    }
                    let _ = cx.preinc_i(cx.lptr(2368));
                }
            }
        }
        let _ = cx.preinc_i(cx.lptr(2480));
    }
    { let t = 1; cx.set_li(1336, t); }
    'lp22: loop {
        if !(cx.li(1336) < cx.i(a3)) { break 'lp22; }
        'blp22: {
            { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(623)); let q1_ = a2; let q2_ = cx.li(1336); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }; cx.set_lpp(333, t); }
            if (cx.b(cx.lpp(333).add(28)) != 0) {
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(333).add(3556))).add(56); let t = (((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(333).add(3552)).add(736)).add(4)) as f64) * 0.5f64) as f32); cx.set_f(p, t); }
            }
        }
        let _ = cx.preinc_i(cx.lptr(1336));
    }
    v147 = ({ let q0_ = a3; let q1_ = cx.li(1336); crate::ov_zero_i!(cx, q0_, q1_) } as i64);
    return v147;
}

/// PhysicsManager__1A3880 0x001A3880
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a3880(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = (((cx.f(a1) as f64) + (cx.f(a2) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a1.add(4)) as f64) + (cx.f(a2.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a1.add(8)) as f64) + (cx.f(a2.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// PhysicsManager__1A38D0 0x001A38D0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a38d0(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = (((cx.f(a2) as f64) + (cx.f(a3) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a2.add(4)) as f64) + (cx.f(a3.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a2.add(8)) as f64) + (cx.f(a3.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// PhysicsManager__1A3920 0x001A3920
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a3920(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = (((cx.f(a1) as f64) - (cx.f(a2) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a1.add(4)) as f64) - (cx.f(a2.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a1.add(8)) as f64) - (cx.f(a2.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// PhysicsManager__1A3970 0x001A3970
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a3970(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr, a4: f32) -> Ptr {
    { let p = a1; let t = ((((cx.f(a3) * a4) as f64) + (cx.f(a2) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((cx.f(a3.add(4)) * a4) as f64) + (cx.f(a2.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((cx.f(a3.add(8)) * a4) as f64) + (cx.f(a2.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a3;
}

/// PhysicsManager__1A39D0 0x001A39D0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a39d0(cx: &mut Cx, a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    let mut v4 = 0.0f32;
    v4 = (f_19d9c0(cx, a2, a3) as f32);
    return f_1a3970(cx, a1, a2, a3, ((0.0f64 - (v4 as f64)) as f32));
}

/// PhysicsManager__1A3A30 0x001A3A30
pub fn f_1a3a30(cx: &mut Cx, a1: Ptr, a3: Ptr, a4: Ptr, a5: i32, a6: Ptr, mut a7: Ptr) -> i64 {
    cx.push_frame(3352);
    let r = f_1a3a30_body(cx, a1, a3, a4, a5, a6, a7);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1a3a30_body(cx: &mut Cx, a1: Ptr, a3: Ptr, a4: Ptr, a5: i32, a6: Ptr, mut a7: Ptr) -> i64 {
    let mut a2 = 0.0f64;
    let mut v7 = 0i32;
    let mut v8 = 0.0f64;
    let mut v9 = 0i32;
    let mut v10 = 0i32;
    let mut v11 = Ptr::NULL;
    let mut v12 = 0i32;
    let mut v13 = 0i32;
    let mut v14 = 0i32;
    let mut v15 = Ptr::NULL;
    let mut v16 = Ptr::NULL;
    let mut v17 = Ptr::NULL;
    let mut v18 = Ptr::NULL;
    let mut v19 = Ptr::NULL;
    let mut v20 = Ptr::NULL;
    let mut v21 = Ptr::NULL;
    let mut v22 = Ptr::NULL;
    let mut v23 = Ptr::NULL;
    let mut v24 = Ptr::NULL;
    let mut v25 = Ptr::NULL;
    let mut v26 = Ptr::NULL;
    let mut v27 = Ptr::NULL;
    let mut v28 = Ptr::NULL;
    let mut v29 = Ptr::NULL;
    let mut v30 = Ptr::NULL;
    let mut v31 = Ptr::NULL;
    let mut v32 = Ptr::NULL;
    let mut v33 = Ptr::NULL;
    let mut v34 = Ptr::NULL;
    let mut v35 = Ptr::NULL;
    let mut v36 = Ptr::NULL;
    let mut v37 = Ptr::NULL;
    let mut v38 = Ptr::NULL;
    let mut v39 = Ptr::NULL;
    let mut v40 = Ptr::NULL;
    let mut v41 = Ptr::NULL;
    let mut v42 = Ptr::NULL;
    let mut v43 = 0i32;
    let mut v44 = 0i32;
    let mut v45 = Ptr::NULL;
    let mut v46 = Ptr::NULL;
    let mut v47 = Ptr::NULL;
    let mut v48 = Ptr::NULL;
    let mut v49 = Ptr::NULL;
    let mut v50 = 0.0f64;
    let mut v51 = Ptr::NULL;
    let mut v52 = Ptr::NULL;
    let mut v53 = 0.0f64;
    let mut v54 = Ptr::NULL;
    let mut v55 = Ptr::NULL;
    let mut v56 = Ptr::NULL;
    let mut v57 = Ptr::NULL;
    let mut v58 = Ptr::NULL;
    let mut v59 = 0.0f64;
    let mut v60 = Ptr::NULL;
    let mut v61 = 0.0f64;
    let mut v62 = Ptr::NULL;
    let mut v63 = 0.0f64;
    let mut v64 = Ptr::NULL;
    let mut v65 = Ptr::NULL;
    let mut v66 = 0i32;
    let mut v68 = Ptr::NULL;
    let mut v69 = Ptr::NULL;
    let mut v70 = Ptr::NULL;
    let mut v71 = Ptr::NULL;
    let mut v72 = Ptr::NULL;
    let mut v73 = Ptr::NULL;
    let mut v74 = Ptr::NULL;
    let mut v75 = Ptr::NULL;
    let mut v76 = Ptr::NULL;
    let mut v77 = Ptr::NULL;
    let mut v78 = Ptr::NULL;
    let mut v79 = Ptr::NULL;
    let mut v80 = Ptr::NULL;
    let mut v81 = Ptr::NULL;
    let mut v82 = Ptr::NULL;
    let mut v83 = Ptr::NULL;
    let mut v84 = Ptr::NULL;
    let mut v85 = Ptr::NULL;
    let mut v86 = Ptr::NULL;
    let mut v87 = Ptr::NULL;
    let mut v88 = Ptr::NULL;
    let mut v89 = Ptr::NULL;
    let mut v90 = Ptr::NULL;
    let mut v91 = Ptr::NULL;
    let mut v92 = Ptr::NULL;
    let mut v93 = Ptr::NULL;
    let mut v94 = Ptr::NULL;
    let mut v95 = Ptr::NULL;
    let mut v96 = Ptr::NULL;
    let mut v97 = 0i32;
    let mut v98 = 0i32;
    let mut v99 = 0i32;
    let mut v100 = 0i32;
    let mut v101 = 0i32;
    let mut v102 = 0i32;
    let mut v103 = 0i32;
    let mut v104 = 0i32;
    let mut v105 = 0i32;
    let mut v106 = 0i32;
    let mut v107 = 0i32;
    let mut v108 = 0i32;
    let mut v109 = 0i32;
    let mut v110 = 0i32;
    let mut v111 = 0i32;
    let mut v112 = 0i32;
    let mut v113 = 0i32;
    let mut v114 = 0i32;
    let mut v115 = 0i32;
    let mut v116 = 0i32;
    let mut v117 = 0i32;
    let mut v118 = 0i32;
    let mut v119 = 0i32;
    let mut v120 = 0i32;
    let mut v121 = 0i32;
    let mut v122 = 0i32;
    let mut v123 = 0i32;
    let mut v124 = 0i32;
    let mut v125 = 0i32;
    let mut v126 = 0i64;
    let mut v127 = 0.0f32;
    let mut v128 = 0i32;
    let mut v129 = 0i32;
    let mut v130 = 0i32;
    let mut v131 = 0i32;
    let mut v132 = 0i32;
    let mut v133 = 0i32;
    let mut v134 = 0i32;
    let mut v135 = 0i32;
    let mut v136 = 0i32;
    let mut v137 = 0i32;
    let mut v138 = 0i32;
    let mut v139 = 0i32;
    let mut v140 = 0i32;
    let mut v141 = 0i32;
    let mut v142 = 0i32;
    let mut v143 = 0i32;
    let mut v144 = 0i32;
    let mut v145 = 0i32;
    let mut v146 = 0i32;
    let mut v147 = 0i32;
    let mut v148 = 0i32;
    let mut v149 = 0i32;
    let mut v150 = 0i32;
    let mut v151 = 0i32;
    let mut v152 = 0i32;
    let mut v153 = 0i32;
    let mut v154 = 0i32;
    let mut v155 = 0.0f32;
    let mut v156 = 0.0f32;
    let mut v157 = Ptr::NULL;
    let mut v158 = Ptr::NULL;
    let mut v159 = 0.0f32;
    let mut v160 = 0i32;
    let mut v161 = 0i32;
    let mut v162 = 0.0f32;
    let mut v163 = 0.0f32;
    let mut v164 = 0.0f32;
    let mut v165 = 0.0f32;
    let mut v166 = 0.0f32;
    let mut bb: u32 = 0;
    #[allow(unreachable_code)]
    'cfg: loop {
        match bb {
            0 => {
                v7 = -858993460;
                { let t = a1; cx.set_lpp(837, t); }
                { let p = Ptr::glob(0x74D474); let t = 0; cx.set_b(p, t); }
                { let t = 0.0f32; cx.set_lf(3344, t); }
                { bb = 1; continue 'cfg; }
                break 'cfg;
            }
            1 => {
                if (cx.li(3344) < cx.i(a3)) { bb = 2; continue 'cfg; } else { bb = 4; continue 'cfg; }
                break 'cfg;
            }
            2 => {
                { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(837)); let q1_ = a2; let q2_ = cx.li(3344); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }; cx.set_lpp(835, t); }
                if (cx.b(cx.lpp(835).add(28)) != 0) { bb = 5; continue 'cfg; } else { bb = 6; continue 'cfg; }
                break 'cfg;
            }
            3 => {
                v7 = cx.preinc_i(cx.lptr(3344));
                { bb = 1; continue 'cfg; }
                break 'cfg;
            }
            4 => {
                v126 = ({ let q0_ = cx.li(3344); let q1_ = v7; crate::ov_zero_i!(cx, q0_, q1_) } as i64);
                return v126;
            }
            5 => {
                { let t = 0.0f32; cx.set_lf(3336, t); }
                if (cx.b(cx.lpp(835).add(4072)) == 0) { bb = 8; continue 'cfg; } else { bb = 9; continue 'cfg; }
                break 'cfg;
            }
            6 => {
                { bb = 7; continue 'cfg; }
                break 'cfg;
            }
            7 => {
                { bb = 3; continue 'cfg; }
                break 'cfg;
            }
            8 => {
                { let p = Ptr::glob(0x74D474); let t = 0; cx.set_b(p, t); }
                { bb = 10; continue 'cfg; }
                break 'cfg;
            }
            9 => {
                { bb = 10; continue 'cfg; }
                break 'cfg;
            }
            10 => {
                { let t = 0; let o = cx.li(3332); cx.set_li(3332, (o & !(255 << 24)) | ((t & 255) << 24)); }
                { let t = cx.lpp(835).add(3604); cx.set_lpp(832, t); }
                if ((cx.b(Ptr::glob(0x74D474)) == 0) && (((cx.ptr_at(cx.lpp(832)).bits() >= 3) && ((cx.i(cx.lpp(835).add(4388)) == 0) || (cx.i(cx.lpp(835).add(4388)) == cx.i(cx.lpp(832))))) || ((cx.i(cx.lpp(835).add(4756)) == 1) && (cx.f(cx.lpp(835).add(4596)) < 0.3f32)))) { bb = 11; continue 'cfg; } else { bb = 12; continue 'cfg; }
                break 'cfg;
            }
            11 => {
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(28); let q2_ = cx.f(Ptr::gc(0).add(1336)); f_1a0390(cx, q0_, q1_, q2_) };
                { let t = 1; let o = cx.li(3332); cx.set_li(3332, (o & !(255 << 24)) | ((t & 255) << 24)); }
                { bb = 13; continue 'cfg; }
                break 'cfg;
            }
            12 => {
                { bb = 13; continue 'cfg; }
                break 'cfg;
            }
            13 => {
                if (cx.i(cx.lpp(835).add(4756)) == 1) { bb = 14; continue 'cfg; } else { bb = 15; continue 'cfg; }
                break 'cfg;
            }
            14 => {
                v8 = vsmath::fabs(cx.f(cx.lpp(835).add(276)));
                { let t = (((v8 * (cx.f(cx.ptr_at(cx.lpp(835).add(3552))) as f64)) * 0.75f64) as f32); cx.set_lf(3324, t); }
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(28); let q2_ = cx.lf(3324); f_1a0390(cx, q0_, q1_, q2_) };
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(56); let t = (((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(56)) as f64) + (cx.lf(3324) as f64)) as f32); cx.set_f(p, t); }
                { bb = 16; continue 'cfg; }
                break 'cfg;
            }
            15 => {
                { let t = 0.0f32; cx.set_lf(3324, t); }
                { bb = 16; continue 'cfg; }
                break 'cfg;
            }
            16 => {
                { let t = 0.0f32; cx.set_lf(2288, t); }
                if (cx.b(cx.lpp(835).add(4073)) != 0) { bb = 17; continue 'cfg; } else { bb = 18; continue 'cfg; }
                break 'cfg;
            }
            17 => {
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))); let q1_ = cx.lptr(2236).bits(); let q2_ = cx.lptr(2236); f_37abb0(cx, q0_, q1_, q2_) };
                { let t = 0.0f32; cx.set_lf(2228, t); }
                { bb = 20; continue 'cfg; }
                break 'cfg;
            }
            18 => {
                if ({ let p = { let q0_ = cx.lpp(835).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }.add(16); cx.i(p) } != 0) { bb = 42; continue 'cfg; } else { bb = 43; continue 'cfg; }
                break 'cfg;
            }
            19 => {
                if (cx.li(2288) != 0) { bb = 80; continue 'cfg; } else { bb = 81; continue 'cfg; }
                break 'cfg;
            }
            20 => {
                if (cx.li(2228) < 4) { bb = 21; continue 'cfg; } else { bb = 23; continue 'cfg; }
                break 'cfg;
            }
            21 => {
                { let q0_ = cx.ptr_at(cx.lpp(837).add(32)); let q1_ = cx.lptr(2208); let q2_ = cx.lptr(2220); let q3_ = cx.lptr(2208); let q4_ = cx.lptr(0).add(4 * ((3_i32).wrapping_mul(cx.li(2228)).wrapping_add(559))); let q5_ = 0; let q6_ = Ptr::NULL; ov_qt_find_leaf(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
                { let t = 0.0f32; cx.set_lf(2200, t); }
                { bb = 24; continue 'cfg; }
                break 'cfg;
            }
            22 => {
                let _ = cx.preinc_i(cx.lptr(2228));
                { bb = 20; continue 'cfg; }
                break 'cfg;
            }
            23 => {
                { bb = 19; continue 'cfg; }
                break 'cfg;
            }
            24 => {
                if (cx.li(2200) < cx.li(2208)) { bb = 25; continue 'cfg; } else { bb = 27; continue 'cfg; }
                break 'cfg;
            }
            25 => {
                { let t = 0.0f32; cx.set_lf(2192, t); }
                { let t = 0.0f32; cx.set_lf(2196, t); }
                { bb = 28; continue 'cfg; }
                break 'cfg;
            }
            26 => {
                let _ = cx.preinc_i(cx.lptr(2200));
                { bb = 24; continue 'cfg; }
                break 'cfg;
            }
            27 => {
                { bb = 22; continue 'cfg; }
                break 'cfg;
            }
            28 => {
                if (cx.li(2196) < cx.li(2288)) { bb = 29; continue 'cfg; } else { bb = 31; continue 'cfg; }
                break 'cfg;
            }
            29 => {
                v9 = { let q0_ = cx.ptr_at(cx.lpp(837).add(32)); let q1_ = cx.li(2200).wrapping_add(cx.li(2220)); ov_qt_face_index(cx, q0_, q1_) };
                if ((cx.f(cx.lptr(0).add(4 * (cx.li(2196).wrapping_add(574)))).to_bits() as i32) == v9) { bb = 32; continue 'cfg; } else { bb = 33; continue 'cfg; }
                break 'cfg;
            }
            30 => {
                let _ = cx.preinc_i(cx.lptr(2196));
                { bb = 28; continue 'cfg; }
                break 'cfg;
            }
            31 => {
                if (cx.li(2192) == 0) { bb = 36; continue 'cfg; } else { bb = 37; continue 'cfg; }
                break 'cfg;
            }
            32 => {
                { let t = 1; cx.set_li(2192, t); }
                { bb = 31; continue 'cfg; }
                break 'cfg;
            }
            33 => {
                { bb = 34; continue 'cfg; }
                break 'cfg;
            }
            34 => {
                { bb = 30; continue 'cfg; }
                break 'cfg;
            }
            35 => {
                { bb = 34; continue 'cfg; }
                break 'cfg;
            }
            36 => {
                v10 = { let q0_ = cx.ptr_at(cx.lpp(837).add(32)); let q1_ = cx.li(2200).wrapping_add(cx.li(2220)); ov_qt_face_index(cx, q0_, q1_) };
                { let p = cx.lptr(0).add(4 * (cx.li(2288).wrapping_add(574))); let t = v10; cx.set_i(p, t); }
                if (cx.li(2288) < 256) { bb = 39; continue 'cfg; } else { bb = 40; continue 'cfg; }
                break 'cfg;
            }
            37 => {
                { bb = 38; continue 'cfg; }
                break 'cfg;
            }
            38 => {
                { bb = 26; continue 'cfg; }
                break 'cfg;
            }
            39 => {
                let _ = cx.preinc_i(cx.lptr(2288));
                { bb = 41; continue 'cfg; }
                break 'cfg;
            }
            40 => {
                { bb = 41; continue 'cfg; }
                break 'cfg;
            }
            41 => {
                { bb = 38; continue 'cfg; }
                break 'cfg;
            }
            42 => {
                v127 = cx.f(cx.ptr_at(cx.lpp(835).add(3552)));
                v11 = { let q0_ = cx.lpp(835).add(240); ov_car_velocity(cx, q0_) };
                { let q0_ = cx.lptr(2176); let q1_ = v11; let q2_ = v127; f_19d970(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(2148); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))); let q2_ = cx.lptr(2176); f_1a38d0(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(2176); let q1_ = 2.0f32; f_19dab0(cx, q0_, q1_) };
                { let t = { let q0_ = cx.lptr(2176); f_19d9f0(cx, q0_) }; cx.set_lf(2168, t); }
                { let t = cx.lf(2168); cx.set_lf(2164, t); }
                { let t = 0.0f32; cx.set_lf(2140, t); }
                { let t = 0.0f32; cx.set_lf(2136, t); }
                { bb = 45; continue 'cfg; }
                break 'cfg;
            }
            43 => {
                { let q0_ = cx.ptr_at(cx.lpp(837).add(32)); let q1_ = cx.lptr(2088); let q2_ = cx.lptr(2088); let q3_ = cx.lptr(2076); let q4_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))); let q5_ = 0; let q6_ = Ptr::NULL; ov_qt_find_leaf(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
                { let t = 0.0f32; cx.set_lf(2068, t); }
                { bb = 73; continue 'cfg; }
                break 'cfg;
            }
            44 => {
                { bb = 19; continue 'cfg; }
                break 'cfg;
            }
            45 => {
                if (1 != 0) { bb = 46; continue 'cfg; } else { bb = 47; continue 'cfg; }
                break 'cfg;
            }
            46 => {
                { let t = ({ let q0_ = cx.ptr_at(cx.lpp(837).add(32)); let q1_ = cx.lptr(2128); let q2_ = cx.lptr(2128); let q3_ = cx.lptr(2116); let q4_ = cx.lptr(2148); let q5_ = 0; let q6_ = Ptr::NULL; ov_qt_find_leaf(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) } as i32); cx.set_li(2108, t); }
                if (cx.li(2108) != cx.li(2136)) { bb = 48; continue 'cfg; } else { bb = 49; continue 'cfg; }
                break 'cfg;
            }
            47 => {
                { bb = 44; continue 'cfg; }
                break 'cfg;
            }
            48 => {
                { let t = 0.0f32; cx.set_lf(2104, t); }
                { bb = 51; continue 'cfg; }
                break 'cfg;
            }
            49 => {
                { bb = 50; continue 'cfg; }
                break 'cfg;
            }
            50 => {
                { let t = cx.lf(2108); cx.set_lf(2136, t); }
                { let t = (((cx.lf(2168) as f64) - (cx.f(cx.ptr_at(cx.lpp(837).add(32)).add(56)) as f64)) as f32); cx.set_lf(2168, t); }
                let _ = cx.preinc_i(cx.lptr(2140));
                if (cx.lf(2168) <= 0.0f32) { bb = 69; continue 'cfg; } else { bb = 70; continue 'cfg; }
                break 'cfg;
            }
            51 => {
                if (cx.li(2104) < cx.li(2116)) { bb = 52; continue 'cfg; } else { bb = 54; continue 'cfg; }
                break 'cfg;
            }
            52 => {
                { let t = 0.0f32; cx.set_lf(2096, t); }
                { let t = 0.0f32; cx.set_lf(2100, t); }
                { bb = 55; continue 'cfg; }
                break 'cfg;
            }
            53 => {
                let _ = cx.preinc_i(cx.lptr(2104));
                { bb = 51; continue 'cfg; }
                break 'cfg;
            }
            54 => {
                { bb = 50; continue 'cfg; }
                break 'cfg;
            }
            55 => {
                if (cx.li(2100) < cx.li(2288)) { bb = 56; continue 'cfg; } else { bb = 58; continue 'cfg; }
                break 'cfg;
            }
            56 => {
                v12 = { let q0_ = cx.ptr_at(cx.lpp(837).add(32)); let q1_ = cx.li(2104).wrapping_add(cx.li(2128)); ov_qt_face_index(cx, q0_, q1_) };
                if ((cx.f(cx.lptr(0).add(4 * (cx.li(2100).wrapping_add(574)))).to_bits() as i32) == v12) { bb = 59; continue 'cfg; } else { bb = 60; continue 'cfg; }
                break 'cfg;
            }
            57 => {
                let _ = cx.preinc_i(cx.lptr(2100));
                { bb = 55; continue 'cfg; }
                break 'cfg;
            }
            58 => {
                if (cx.li(2096) == 0) { bb = 63; continue 'cfg; } else { bb = 64; continue 'cfg; }
                break 'cfg;
            }
            59 => {
                { let t = 1; cx.set_li(2096, t); }
                { bb = 58; continue 'cfg; }
                break 'cfg;
            }
            60 => {
                { bb = 61; continue 'cfg; }
                break 'cfg;
            }
            61 => {
                { bb = 57; continue 'cfg; }
                break 'cfg;
            }
            62 => {
                { bb = 61; continue 'cfg; }
                break 'cfg;
            }
            63 => {
                v13 = { let q0_ = cx.ptr_at(cx.lpp(837).add(32)); let q1_ = cx.li(2104).wrapping_add(cx.li(2128)); ov_qt_face_index(cx, q0_, q1_) };
                { let p = cx.lptr(0).add(4 * (cx.li(2288).wrapping_add(574))); let t = v13; cx.set_i(p, t); }
                if (cx.li(2288) < 256) { bb = 66; continue 'cfg; } else { bb = 67; continue 'cfg; }
                break 'cfg;
            }
            64 => {
                { bb = 65; continue 'cfg; }
                break 'cfg;
            }
            65 => {
                { bb = 53; continue 'cfg; }
                break 'cfg;
            }
            66 => {
                let _ = cx.preinc_i(cx.lptr(2288));
                { bb = 68; continue 'cfg; }
                break 'cfg;
            }
            67 => {
                { bb = 68; continue 'cfg; }
                break 'cfg;
            }
            68 => {
                { bb = 65; continue 'cfg; }
                break 'cfg;
            }
            69 => {
                { bb = 47; continue 'cfg; }
                break 'cfg;
            }
            70 => {
                { bb = 71; continue 'cfg; }
                break 'cfg;
            }
            71 => {
                { let q0_ = cx.lptr(2148); let q1_ = cx.lptr(2176); let q2_ = ((((0.0f32 - cx.f(cx.ptr_at(cx.lpp(837).add(32)).add(56))) as f64) / (cx.lf(2164) as f64)) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                { bb = 45; continue 'cfg; }
                break 'cfg;
            }
            72 => {
                { bb = 71; continue 'cfg; }
                break 'cfg;
            }
            73 => {
                if (cx.li(2068) < cx.li(2076)) { bb = 74; continue 'cfg; } else { bb = 76; continue 'cfg; }
                break 'cfg;
            }
            74 => {
                v14 = { let q0_ = cx.ptr_at(cx.lpp(837).add(32)); let q1_ = cx.li(2068).wrapping_add(cx.li(2088)); ov_qt_face_index(cx, q0_, q1_) };
                { let p = cx.lptr(0).add(4 * (cx.li(2288).wrapping_add(574))); let t = v14; cx.set_i(p, t); }
                if (cx.li(2288) < 256) { bb = 77; continue 'cfg; } else { bb = 78; continue 'cfg; }
                break 'cfg;
            }
            75 => {
                let _ = cx.preinc_i(cx.lptr(2068));
                { bb = 73; continue 'cfg; }
                break 'cfg;
            }
            76 => {
                { bb = 44; continue 'cfg; }
                break 'cfg;
            }
            77 => {
                let _ = cx.preinc_i(cx.lptr(2288));
                { bb = 79; continue 'cfg; }
                break 'cfg;
            }
            78 => {
                { bb = 79; continue 'cfg; }
                break 'cfg;
            }
            79 => {
                { bb = 75; continue 'cfg; }
                break 'cfg;
            }
            80 => {
                { let t = 0.0f32; cx.set_lf(2060, t); }
                { let t = 0.0f32; cx.set_lf(2052, t); }
                { let t = 0.0f32; cx.set_lf(2044, t); }
                if (cx.i(Ptr::gc(0).add(2072)) != 0) { bb = 83; continue 'cfg; } else { bb = 84; continue 'cfg; }
                break 'cfg;
            }
            81 => {
                { bb = 82; continue 'cfg; }
                break 'cfg;
            }
            82 => {
                if (((cx.li(3332) >> 24) & 255) != 0) { bb = 307; continue 'cfg; } else { bb = 308; continue 'cfg; }
                break 'cfg;
            }
            83 => {
                { let t = 0.0f32; cx.set_lf(2040, t); }
                { bb = 86; continue 'cfg; }
                break 'cfg;
            }
            84 => {
                { bb = 85; continue 'cfg; }
                break 'cfg;
            }
            85 => {
                { bb = 93; continue 'cfg; }
                break 'cfg;
            }
            86 => {
                { bb = 87; continue 'cfg; }
                break 'cfg;
            }
            87 => {
                if (cx.li(2040) >= cx.i(cx.ptr_at(cx.lpp(835).add(3556)).add(4))) { bb = 90; continue 'cfg; } else { bb = 91; continue 'cfg; }
                break 'cfg;
            }
            88 => {
                let _ = cx.preinc_i(cx.lptr(2040));
                { bb = 86; continue 'cfg; }
                break 'cfg;
            }
            89 => {
                { let t = 0.0f32; cx.set_lf(1696, t); }
                { let t = 0.0f32; cx.set_lf(1700, t); }
                { bb = 162; continue 'cfg; }
                break 'cfg;
            }
            90 => {
                { bb = 93; continue 'cfg; }
                break 'cfg;
            }
            91 => {
                { bb = 92; continue 'cfg; }
                break 'cfg;
            }
            92 => {
                if (((cx.b(cx.lpp(835).add(4072)) != 0) && (cx.i(Ptr::gc(0).add(1976)) != 0)) || ((cx.li(3344) == cx.i(Ptr::glob(0x6D5344))) && (cx.f(Ptr::gc(0).add(1980)) != 0.0f32))) { bb = 95; continue 'cfg; } else { bb = 96; continue 'cfg; }
                break 'cfg;
            }
            93 => {
                if ((cx.li(3336) != 0) && (cx.i(Ptr::gc(0).add(2076)) != 0)) { bb = 223; continue 'cfg; } else { bb = 224; continue 'cfg; }
                break 'cfg;
            }
            94 => {
                { bb = 92; continue 'cfg; }
                break 'cfg;
            }
            95 => {
                if (((cx.i(Ptr::glob(0x6D5344)) != 0) && (cx.f(Ptr::gc(0).add(1980)) != 0.0f32)) && (cx.i(cx.lpp(835).add(32)) != 0)) { bb = 98; continue 'cfg; } else { bb = 99; continue 'cfg; }
                break 'cfg;
            }
            96 => {
                { bb = 97; continue 'cfg; }
                break 'cfg;
            }
            97 => {
                if (((cx.b(cx.lpp(835).add(4072)) != 0) && (cx.li(2040) == 0)) && (cx.i(Ptr::gc(0).add(1984)) != 0)) { bb = 104; continue 'cfg; } else { bb = 105; continue 'cfg; }
                break 'cfg;
            }
            98 => {
                { let q0_ = cx.lpp(835).add(1476); let q1_ = 1; f_1a6ed0(cx, q0_, q1_) };
                if (cx.i(Ptr::glob(0x74D470)) != cx.i(Ptr::glob(0x6D5344))) { bb = 101; continue 'cfg; } else { bb = 102; continue 'cfg; }
                break 'cfg;
            }
            99 => {
                { bb = 100; continue 'cfg; }
                break 'cfg;
            }
            100 => {
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add((132_i32).wrapping_mul(cx.li(2040))); let q1_ = cx.i(cx.lpp(835).add(3556)); let q2_ = cx.lptr(1940); f_37a7c0(cx, q0_, q1_, q2_) };
                v128 = { let p = { let q0_ = cx.lptr(1048); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v97 = cx.i(Ptr::glob(0x77FDA8));
                v68 = { let q0_ = cx.lptr(1032); let q1_ = cx.li(1952); let q2_ = cx.li(1956); let q3_ = cx.li(1960); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v15 = { let q0_ = cx.lptr(1016); let q1_ = cx.li(1940); let q2_ = cx.li(1944); let q3_ = cx.li(1948); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v15, v68, v97, v128);
                v129 = { let p = { let q0_ = cx.lptr(1012); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v98 = cx.i(Ptr::glob(0x77FDA8));
                v69 = { let q0_ = cx.lptr(996); let q1_ = cx.li(1964); let q2_ = cx.li(1968); let q3_ = cx.li(1972); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v16 = { let q0_ = cx.lptr(980); let q1_ = cx.li(1952); let q2_ = cx.li(1956); let q3_ = cx.li(1960); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v16, v69, v98, v129);
                v130 = { let p = { let q0_ = cx.lptr(976); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v99 = cx.i(Ptr::glob(0x77FDA8));
                v70 = { let q0_ = cx.lptr(960); let q1_ = cx.li(1976); let q2_ = cx.li(1980); let q3_ = cx.li(1984); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v17 = { let q0_ = cx.lptr(944); let q1_ = cx.li(1964); let q2_ = cx.li(1968); let q3_ = cx.li(1972); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v17, v70, v99, v130);
                v131 = { let p = { let q0_ = cx.lptr(940); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v100 = cx.i(Ptr::glob(0x77FDA8));
                v71 = { let q0_ = cx.lptr(924); let q1_ = cx.li(1940); let q2_ = cx.li(1944); let q3_ = cx.li(1948); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v18 = { let q0_ = cx.lptr(908); let q1_ = cx.li(1976); let q2_ = cx.li(1980); let q3_ = cx.li(1984); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v18, v71, v100, v131);
                v132 = { let p = { let q0_ = cx.lptr(904); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v101 = cx.i(Ptr::glob(0x77FDA8));
                v72 = { let q0_ = cx.lptr(888); let q1_ = cx.li(2000); let q2_ = cx.li(2004); let q3_ = cx.li(2008); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v19 = { let q0_ = cx.lptr(872); let q1_ = cx.li(1988); let q2_ = cx.li(1992); let q3_ = cx.li(1996); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v19, v72, v101, v132);
                v133 = { let p = { let q0_ = cx.lptr(868); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v102 = cx.i(Ptr::glob(0x77FDA8));
                v73 = { let q0_ = cx.lptr(852); let q1_ = cx.li(2012); let q2_ = cx.li(2016); let q3_ = cx.li(2020); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v20 = { let q0_ = cx.lptr(836); let q1_ = cx.li(2000); let q2_ = cx.li(2004); let q3_ = cx.li(2008); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v20, v73, v102, v133);
                v134 = { let p = { let q0_ = cx.lptr(832); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v103 = cx.i(Ptr::glob(0x77FDA8));
                v74 = { let q0_ = cx.lptr(816); let q1_ = cx.li(2024); let q2_ = cx.li(2028); let q3_ = cx.li(2032); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v21 = { let q0_ = cx.lptr(800); let q1_ = cx.li(2012); let q2_ = cx.li(2016); let q3_ = cx.li(2020); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v21, v74, v103, v134);
                v135 = { let p = { let q0_ = cx.lptr(796); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v104 = cx.i(Ptr::glob(0x77FDA8));
                v75 = { let q0_ = cx.lptr(780); let q1_ = cx.li(1988); let q2_ = cx.li(1992); let q3_ = cx.li(1996); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v22 = { let q0_ = cx.lptr(764); let q1_ = cx.li(2024); let q2_ = cx.li(2028); let q3_ = cx.li(2032); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v22, v75, v104, v135);
                v136 = { let p = { let q0_ = cx.lptr(760); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v105 = cx.i(Ptr::glob(0x77FDA8));
                v76 = { let q0_ = cx.lptr(744); let q1_ = cx.li(1988); let q2_ = cx.li(1992); let q3_ = cx.li(1996); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v23 = { let q0_ = cx.lptr(728); let q1_ = cx.li(1940); let q2_ = cx.li(1944); let q3_ = cx.li(1948); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v23, v76, v105, v136);
                v137 = { let p = { let q0_ = cx.lptr(724); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v106 = cx.i(Ptr::glob(0x77FDA8));
                v77 = { let q0_ = cx.lptr(708); let q1_ = cx.li(2000); let q2_ = cx.li(2004); let q3_ = cx.li(2008); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v24 = { let q0_ = cx.lptr(692); let q1_ = cx.li(1952); let q2_ = cx.li(1956); let q3_ = cx.li(1960); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v24, v77, v106, v137);
                v138 = { let p = { let q0_ = cx.lptr(688); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v107 = cx.i(Ptr::glob(0x77FDA8));
                v78 = { let q0_ = cx.lptr(672); let q1_ = cx.li(2012); let q2_ = cx.li(2016); let q3_ = cx.li(2020); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v25 = { let q0_ = cx.lptr(656); let q1_ = cx.li(1964); let q2_ = cx.li(1968); let q3_ = cx.li(1972); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v25, v78, v107, v138);
                v139 = { let p = { let q0_ = cx.lptr(652); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v108 = cx.i(Ptr::glob(0x77FDA8));
                v79 = { let q0_ = cx.lptr(636); let q1_ = cx.li(2024); let q2_ = cx.li(2028); let q3_ = cx.li(2032); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v26 = { let q0_ = cx.lptr(620); let q1_ = cx.li(1976); let q2_ = cx.li(1980); let q3_ = cx.li(1984); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v26, v79, v108, v139);
                { bb = 97; continue 'cfg; }
                break 'cfg;
            }
            101 => {
                crate::ov_ignore!(cx, 1, "%s\n");
                { let p = Ptr::glob(0x74D470); let t = cx.i(Ptr::glob(0x6D5344)); cx.set_i(p, t); }
                { bb = 103; continue 'cfg; }
                break 'cfg;
            }
            102 => {
                { bb = 103; continue 'cfg; }
                break 'cfg;
            }
            103 => {
                { bb = 100; continue 'cfg; }
                break 'cfg;
            }
            104 => {
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add((132_i32).wrapping_mul(cx.li(2040))); let q1_ = cx.i(cx.lpp(835).add(3556)); let q2_ = cx.lptr(1836); f_37a7c0(cx, q0_, q1_, q2_) };
                v140 = { let p = { let q0_ = cx.lptr(616); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v109 = cx.i(Ptr::glob(0x77FD6C));
                v80 = { let q0_ = cx.lptr(600); let q1_ = cx.li(1848); let q2_ = cx.li(1852); let q3_ = cx.li(1856); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v27 = { let q0_ = cx.lptr(584); let q1_ = cx.li(1836); let q2_ = cx.li(1840); let q3_ = cx.li(1844); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v27, v80, v109, v140);
                v141 = { let p = { let q0_ = cx.lptr(580); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v110 = cx.i(Ptr::glob(0x77FD6C));
                v81 = { let q0_ = cx.lptr(564); let q1_ = cx.li(1860); let q2_ = cx.li(1864); let q3_ = cx.li(1868); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v28 = { let q0_ = cx.lptr(548); let q1_ = cx.li(1848); let q2_ = cx.li(1852); let q3_ = cx.li(1856); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v28, v81, v110, v141);
                v142 = { let p = { let q0_ = cx.lptr(544); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v111 = cx.i(Ptr::glob(0x77FD6C));
                v82 = { let q0_ = cx.lptr(528); let q1_ = cx.li(1872); let q2_ = cx.li(1876); let q3_ = cx.li(1880); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v29 = { let q0_ = cx.lptr(512); let q1_ = cx.li(1860); let q2_ = cx.li(1864); let q3_ = cx.li(1868); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v29, v82, v111, v142);
                v143 = { let p = { let q0_ = cx.lptr(508); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v112 = cx.i(Ptr::glob(0x77FD6C));
                v83 = { let q0_ = cx.lptr(492); let q1_ = cx.li(1836); let q2_ = cx.li(1840); let q3_ = cx.li(1844); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v30 = { let q0_ = cx.lptr(476); let q1_ = cx.li(1872); let q2_ = cx.li(1876); let q3_ = cx.li(1880); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v30, v83, v112, v143);
                v144 = { let p = { let q0_ = cx.lptr(472); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v113 = cx.i(Ptr::glob(0x77FD6C));
                v84 = { let q0_ = cx.lptr(456); let q1_ = cx.li(1896); let q2_ = cx.li(1900); let q3_ = cx.li(1904); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v31 = { let q0_ = cx.lptr(440); let q1_ = cx.li(1884); let q2_ = cx.li(1888); let q3_ = cx.li(1892); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v31, v84, v113, v144);
                v145 = { let p = { let q0_ = cx.lptr(436); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v114 = cx.i(Ptr::glob(0x77FD6C));
                v85 = { let q0_ = cx.lptr(420); let q1_ = cx.li(1908); let q2_ = cx.li(1912); let q3_ = cx.li(1916); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v32 = { let q0_ = cx.lptr(404); let q1_ = cx.li(1896); let q2_ = cx.li(1900); let q3_ = cx.li(1904); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v32, v85, v114, v145);
                v146 = { let p = { let q0_ = cx.lptr(400); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v115 = cx.i(Ptr::glob(0x77FD6C));
                v86 = { let q0_ = cx.lptr(384); let q1_ = cx.li(1920); let q2_ = cx.li(1924); let q3_ = cx.li(1928); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v33 = { let q0_ = cx.lptr(368); let q1_ = cx.li(1908); let q2_ = cx.li(1912); let q3_ = cx.li(1916); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v33, v86, v115, v146);
                v147 = { let p = { let q0_ = cx.lptr(364); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v116 = cx.i(Ptr::glob(0x77FD6C));
                v87 = { let q0_ = cx.lptr(348); let q1_ = cx.li(1884); let q2_ = cx.li(1888); let q3_ = cx.li(1892); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v34 = { let q0_ = cx.lptr(332); let q1_ = cx.li(1920); let q2_ = cx.li(1924); let q3_ = cx.li(1928); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v34, v87, v116, v147);
                v148 = { let p = { let q0_ = cx.lptr(328); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v117 = cx.i(Ptr::glob(0x77FD6C));
                v88 = { let q0_ = cx.lptr(312); let q1_ = cx.li(1884); let q2_ = cx.li(1888); let q3_ = cx.li(1892); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v35 = { let q0_ = cx.lptr(296); let q1_ = cx.li(1836); let q2_ = cx.li(1840); let q3_ = cx.li(1844); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v35, v88, v117, v148);
                v149 = { let p = { let q0_ = cx.lptr(292); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v118 = cx.i(Ptr::glob(0x77FD6C));
                v89 = { let q0_ = cx.lptr(276); let q1_ = cx.li(1896); let q2_ = cx.li(1900); let q3_ = cx.li(1904); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v36 = { let q0_ = cx.lptr(260); let q1_ = cx.li(1848); let q2_ = cx.li(1852); let q3_ = cx.li(1856); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v36, v89, v118, v149);
                v150 = { let p = { let q0_ = cx.lptr(256); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v119 = cx.i(Ptr::glob(0x77FD6C));
                v90 = { let q0_ = cx.lptr(240); let q1_ = cx.li(1908); let q2_ = cx.li(1912); let q3_ = cx.li(1916); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v37 = { let q0_ = cx.lptr(224); let q1_ = cx.li(1860); let q2_ = cx.li(1864); let q3_ = cx.li(1868); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v37, v90, v119, v150);
                v151 = { let p = { let q0_ = cx.lptr(220); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v120 = cx.i(Ptr::glob(0x77FD6C));
                v91 = { let q0_ = cx.lptr(204); let q1_ = cx.li(1920); let q2_ = cx.li(1924); let q3_ = cx.li(1928); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v38 = { let q0_ = cx.lptr(188); let q1_ = cx.li(1872); let q2_ = cx.li(1876); let q3_ = cx.li(1880); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v38, v91, v120, v151);
                { bb = 106; continue 'cfg; }
                break 'cfg;
            }
            105 => {
                { bb = 106; continue 'cfg; }
                break 'cfg;
            }
            106 => {
                { let t = 0.0f32; cx.set_lf(2052, t); }
                { let t = 0.0f32; cx.set_lf(2048, t); }
                { bb = 107; continue 'cfg; }
                break 'cfg;
            }
            107 => {
                if (cx.li(2048) < cx.li(2288)) { bb = 108; continue 'cfg; } else { bb = 110; continue 'cfg; }
                break 'cfg;
            }
            108 => {
                { let t = { let q0_ = cx.ptr_at(cx.lpp(837).add(28)); let q1_ = (cx.f(cx.lptr(0).add(4 * (cx.li(2048).wrapping_add(574)))).to_bits() as i32); let q2_ = 0; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(457, t); }
                { let t = { let q0_ = cx.ptr_at(cx.lpp(837).add(28)); let q1_ = (cx.f(cx.lptr(0).add(4 * (cx.li(2048).wrapping_add(574)))).to_bits() as i32); let q2_ = 1; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(456, t); }
                { let t = { let q0_ = cx.ptr_at(cx.lpp(837).add(28)); let q1_ = (cx.f(cx.lptr(0).add(4 * (cx.li(2048).wrapping_add(574)))).to_bits() as i32); let q2_ = 2; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(455, t); }
                { let q0_ = cx.lptr(1804); let q1_ = cx.lpp(456); let q2_ = cx.lpp(457); f_19d920(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(1784); let q1_ = cx.lpp(455); let q2_ = cx.lpp(457); f_19d920(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(1764); let q1_ = cx.lptr(1804); let q2_ = cx.lptr(1784); f_1a03f0(cx, q0_, q1_, q2_) };
                { let t = { let q0_ = cx.lptr(1764); f_19d9f0(cx, q0_) }; cx.set_lf(1756, t); }
                if (cx.lf(1756) == 0.0f32) { bb = 111; continue 'cfg; } else { bb = 112; continue 'cfg; }
                break 'cfg;
            }
            109 => {
                let _ = cx.preinc_i(cx.lptr(2048));
                { bb = 107; continue 'cfg; }
                break 'cfg;
            }
            110 => {
                if (cx.li(2052) != 0) { bb = 123; continue 'cfg; } else { bb = 124; continue 'cfg; }
                break 'cfg;
            }
            111 => {
                v152 = { let p = { let q0_ = cx.lptr(184); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v121 = cx.i(Ptr::glob(0x77FD48));
                v92 = { let q0_ = cx.lptr(168); let q1_ = cx.i(cx.lpp(456)); let q2_ = cx.i(cx.lpp(456).add(4)); let q3_ = cx.i(cx.lpp(456).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v39 = { let q0_ = cx.lptr(152); let q1_ = cx.i(cx.lpp(457)); let q2_ = cx.i(cx.lpp(457).add(4)); let q3_ = cx.i(cx.lpp(457).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v39, v92, v121, v152);
                v153 = { let p = { let q0_ = cx.lptr(148); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v122 = cx.i(Ptr::glob(0x77FD48));
                v93 = { let q0_ = cx.lptr(132); let q1_ = cx.i(cx.lpp(455)); let q2_ = cx.i(cx.lpp(455).add(4)); let q3_ = cx.i(cx.lpp(455).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v40 = { let q0_ = cx.lptr(116); let q1_ = cx.i(cx.lpp(457)); let q2_ = cx.i(cx.lpp(457).add(4)); let q3_ = cx.i(cx.lpp(457).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v40, v93, v122, v153);
                v154 = { let p = { let q0_ = cx.lptr(112); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v123 = cx.i(Ptr::glob(0x77FD48));
                v94 = { let q0_ = cx.lptr(96); let q1_ = cx.i(cx.lpp(455)); let q2_ = cx.i(cx.lpp(455).add(4)); let q3_ = cx.i(cx.lpp(455).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v41 = { let q0_ = cx.lptr(80); let q1_ = cx.i(cx.lpp(456)); let q2_ = cx.i(cx.lpp(456).add(4)); let q3_ = cx.i(cx.lpp(456).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v41, v94, v123, v154);
                { bb = 113; continue 'cfg; }
                break 'cfg;
            }
            112 => {
                { bb = 113; continue 'cfg; }
                break 'cfg;
            }
            113 => {
                { let t = cx.i(cx.ptr_at(cx.lpp(837).add(28)).add(28)).wrapping_add((12_i32).wrapping_mul((cx.f(cx.lptr(0).add(4 * (cx.li(2048).wrapping_add(574)))).to_bits() as i32))); cx.set_lpp(438, Ptr::raw(t)); }
                if ((((((cx.i(Ptr::gc(0).add(1332)) != 0) && (cx.b(Ptr::glob(0x74D474)) == 0)) && (cx.i(cx.lpp(832)) == 4)) && (cx.f(cx.lpp(438).add(4)) > cx.f(Ptr::gc(0).add(1712)))) || ((({ let p = { let q0_ = cx.lpp(835).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }.add(64); cx.i(p) } != 0) && (cx.f(cx.lpp(438).add(4)) > cx.f(Ptr::gc(0).add(956)))) && ({ let p = { let q0_ = cx.lpp(835).add(240); let q1_ = 1; ov_car_rot_row(cx, q0_, q1_) }.add(4); cx.f(p) } > cx.f(Ptr::gc(0).add(952))))) || ((((((cx.i(cx.lpp(835).add(4756)) == 1) && (cx.b(cx.lpp(835).add(4760)) != 0)) && (cx.f(cx.lpp(835).add(4780)) > cx.f(Ptr::gc(0).add(1144)))) && (cx.f(cx.lpp(438).add(4)) > cx.f(Ptr::gc(0).add(1136)))) && (cx.f(cx.lpp(835).add(4596)) > (cx.f(cx.lpp(835).add(4592)) * cx.f(Ptr::gc(0).add(1140))))) && (cx.f(cx.lpp(835).add(4592)) > cx.f(cx.lpp(835).add(4596))))) { bb = 114; continue 'cfg; } else { bb = 115; continue 'cfg; }
                break 'cfg;
            }
            114 => {
                { let t = 0.0f32; cx.set_lf(1748, t); }
                if ((((((((cx.i(Ptr::gc(0).add(2120)) != 0) && (cx.b(a7) == 0)) && (cx.i(cx.lpp(835).add(4756)) == 1)) && (cx.b(cx.lpp(835).add(4760)) != 0)) && (cx.f(cx.lpp(835).add(4780)) > cx.f(Ptr::gc(0).add(1144)))) && (cx.f(cx.lpp(438).add(4)) > cx.f(Ptr::gc(0).add(1136)))) && (cx.f(cx.lpp(835).add(4596)) > (cx.f(cx.lpp(835).add(4592)) * cx.f(Ptr::gc(0).add(1140))))) && (cx.f(cx.lpp(835).add(4592)) > cx.f(cx.lpp(835).add(4596)))) { bb = 117; continue 'cfg; } else { bb = 118; continue 'cfg; }
                break 'cfg;
            }
            115 => {
                { let t = ({ let q0_ = cx.lpp(457); let q1_ = cx.lpp(456); let q2_ = cx.lpp(455); let q3_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add((132_i32).wrapping_mul(cx.li(2040))); f_3813b0(cx, q0_, q1_, q2_, q3_) } as i32); cx.set_li(1748, t); }
                { bb = 116; continue 'cfg; }
                break 'cfg;
            }
            116 => {
                if (cx.li(1748) != 0) { bb = 120; continue 'cfg; } else { bb = 121; continue 'cfg; }
                break 'cfg;
            }
            117 => {
                { let p = a7; let t = 1; cx.set_b(p, t); }
                { bb = 119; continue 'cfg; }
                break 'cfg;
            }
            118 => {
                { bb = 119; continue 'cfg; }
                break 'cfg;
            }
            119 => {
                { bb = 116; continue 'cfg; }
                break 'cfg;
            }
            120 => {
                { let q0_ = cx.lptr(1732); let q1_ = cx.lpp(457); let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add((132_i32).wrapping_mul(cx.li(2040))); f_19d920(cx, q0_, q1_, q2_) };
                let _ = cx.preinc_i(cx.lptr(2052));
                { let p = Ptr::glob(0x75AA38).add(4 * (cx.li(2048))); let t = 1; cx.set_i(p, t); }
                { bb = 122; continue 'cfg; }
                break 'cfg;
            }
            121 => {
                { let p = Ptr::glob(0x75AA38).add(4 * (cx.li(2048))); let t = 0; cx.set_i(p, t); }
                { bb = 122; continue 'cfg; }
                break 'cfg;
            }
            122 => {
                { bb = 109; continue 'cfg; }
                break 'cfg;
            }
            123 => {
                { let t = 0.0f32; cx.set_lf(2044, t); }
                { let t = 0.0f32; cx.set_lf(1724, t); }
                { let t = 0.0f32; cx.set_lf(2048, t); }
                { bb = 126; continue 'cfg; }
                break 'cfg;
            }
            124 => {
                { bb = 125; continue 'cfg; }
                break 'cfg;
            }
            125 => {
                if (cx.li(2052) != 0) { bb = 151; continue 'cfg; } else { bb = 152; continue 'cfg; }
                break 'cfg;
            }
            126 => {
                if (cx.li(2048) < cx.li(2288)) { bb = 127; continue 'cfg; } else { bb = 129; continue 'cfg; }
                break 'cfg;
            }
            127 => {
                if (cx.i(Ptr::glob(0x75AA38).add(4 * (cx.li(2048)))) == 1) { bb = 130; continue 'cfg; } else { bb = 131; continue 'cfg; }
                break 'cfg;
            }
            128 => {
                let _ = cx.preinc_i(cx.lptr(2048));
                { bb = 126; continue 'cfg; }
                break 'cfg;
            }
            129 => {
                { bb = 125; continue 'cfg; }
                break 'cfg;
            }
            130 => {
                { let t = cx.f(cx.lptr(0).add(4 * (cx.li(2048).wrapping_add(574)))); cx.set_lf(1720, t); }
                { let q0_ = Ptr::glob(0x757078).add(4 * ((3_i32).wrapping_mul(cx.li(1724)))); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(837).add(28)).add(28)).add((12_i32).wrapping_mul((cx.f(cx.lptr(0).add(4 * (cx.li(2048).wrapping_add(574)))).to_bits() as i32))); f_19e030(cx, q0_, q1_) };
                { let t = 0.0f32; cx.set_lf(1712, t); }
                { bb = 133; continue 'cfg; }
                break 'cfg;
            }
            131 => {
                { bb = 132; continue 'cfg; }
                break 'cfg;
            }
            132 => {
                { bb = 128; continue 'cfg; }
                break 'cfg;
            }
            133 => {
                if (cx.li(1712) < 3) { bb = 134; continue 'cfg; } else { bb = 136; continue 'cfg; }
                break 'cfg;
            }
            134 => {
                { let t = 0.0f32; cx.set_lf(1708, t); }
                { let t = { let q0_ = cx.ptr_at(cx.lpp(837).add(28)); let q1_ = (cx.f(cx.lptr(0).add(4 * (cx.li(2048).wrapping_add(574)))).to_bits() as i32); let q2_ = cx.li(1712); f_1b45f0(cx, q0_, q1_, q2_) }; cx.set_li(1704, t); }
                { let t = 0.0f32; cx.set_lf(1716, t); }
                { bb = 137; continue 'cfg; }
                break 'cfg;
            }
            135 => {
                let _ = cx.preinc_i(cx.lptr(1712));
                { bb = 133; continue 'cfg; }
                break 'cfg;
            }
            136 => {
                if (cx.i(cx.ptr_at(cx.lpp(837).add(28)).add(12)) != 0) { bb = 148; continue 'cfg; } else { bb = 149; continue 'cfg; }
                break 'cfg;
            }
            137 => {
                if (cx.li(1716) < cx.li(2044)) { bb = 138; continue 'cfg; } else { bb = 140; continue 'cfg; }
                break 'cfg;
            }
            138 => {
                if (cx.i(Ptr::glob(0x74BD00).add(4 * (cx.li(1716)))) == cx.li(1704)) { bb = 141; continue 'cfg; } else { bb = 142; continue 'cfg; }
                break 'cfg;
            }
            139 => {
                let _ = cx.preinc_i(cx.lptr(1716));
                { bb = 137; continue 'cfg; }
                break 'cfg;
            }
            140 => {
                if (cx.li(1708) == 0) { bb = 145; continue 'cfg; } else { bb = 146; continue 'cfg; }
                break 'cfg;
            }
            141 => {
                { let p = Ptr::glob(0x758938).add(4 * ((4_i32).wrapping_mul(cx.li(1724)).wrapping_add(cx.li(1712)))); let t = cx.li(1716); cx.set_i(p, t); }
                { let t = 1; cx.set_li(1708, t); }
                { bb = 140; continue 'cfg; }
                break 'cfg;
            }
            142 => {
                { bb = 143; continue 'cfg; }
                break 'cfg;
            }
            143 => {
                { bb = 139; continue 'cfg; }
                break 'cfg;
            }
            144 => {
                { bb = 143; continue 'cfg; }
                break 'cfg;
            }
            145 => {
                v42 = { let q0_ = cx.ptr_at(cx.lpp(837).add(28)); let q1_ = (cx.f(cx.lptr(0).add(4 * (cx.li(2048).wrapping_add(574)))).to_bits() as i32); let q2_ = cx.li(1712); ov_mesh_vertex(cx, q0_, q1_, q2_) };
                { let q0_ = Ptr::glob(0x752638).add(4 * ((3_i32).wrapping_mul(cx.li(2044)))); let q1_ = v42; f_19e030(cx, q0_, q1_) };
                { let p = Ptr::glob(0x758938).add(4 * ((4_i32).wrapping_mul(cx.li(1724)).wrapping_add(cx.li(1712)))); let t = cx.li(2044); cx.set_i(p, t); }
                v43 = { let q0_ = cx.ptr_at(cx.lpp(837).add(28)); let q1_ = (cx.f(cx.lptr(0).add(4 * (cx.li(2048).wrapping_add(574)))).to_bits() as i32); let q2_ = cx.li(1712); f_1b45f0(cx, q0_, q1_, q2_) };
                { let p = Ptr::glob(0x74BD00).add(4 * (cx.postinc_i(cx.lptr(2044)))); let t = v43; cx.set_i(p, t); }
                { bb = 147; continue 'cfg; }
                break 'cfg;
            }
            146 => {
                { bb = 147; continue 'cfg; }
                break 'cfg;
            }
            147 => {
                { bb = 135; continue 'cfg; }
                break 'cfg;
            }
            148 => {
                v44 = (cx.u16(cx.ptr_at(cx.ptr_at(cx.lpp(837).add(28)).add(24)).add((8_i32).wrapping_mul(cx.li(1720))).add(6)) as u8 as i32);
                { bb = 150; continue 'cfg; }
                break 'cfg;
            }
            149 => {
                v44 = (cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(837).add(28)).add(20)).add((16_i32).wrapping_mul(cx.li(1720))).add(12)) & -65281);
                { bb = 150; continue 'cfg; }
                break 'cfg;
            }
            150 => {
                { let p = Ptr::glob(0x758944).add(4 * ((4_i32).wrapping_mul(cx.postinc_i(cx.lptr(1724))))); let t = v44; cx.set_i(p, t); }
                { bb = 132; continue 'cfg; }
                break 'cfg;
            }
            151 => {
                if ((cx.i(Ptr::gc(0).add(2120)) != 0) && (cx.b(a7) != 0)) { bb = 154; continue 'cfg; } else { bb = 155; continue 'cfg; }
                break 'cfg;
            }
            152 => {
                { bb = 153; continue 'cfg; }
                break 'cfg;
            }
            153 => {
                { bb = 161; continue 'cfg; }
                break 'cfg;
            }
            154 => {
                crate::ov_ignore!(cx, 1, ".");
                { bb = 156; continue 'cfg; }
                break 'cfg;
            }
            155 => {
                { bb = 156; continue 'cfg; }
                break 'cfg;
            }
            156 => {
                { let t = a5.wrapping_sub(cx.li(3336)); cx.set_li(2060, t); }
                if (({ let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(3336))); let q1_ = cx.lptr(2060); let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add((132_i32).wrapping_mul(cx.li(2040))); let q3_ = Ptr::glob(0x752638); let q4_ = cx.li(2044); let q5_ = Ptr::glob(0x758938); let q6_ = Ptr::glob(0x757078); let q7_ = cx.li(2052); let q8_ = ((cx.b(Ptr::glob(0x74D474)) as u8 as i32) as f32); let q9_ = 0.08f32; let q10_ = 0; f_394ea0(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_, q10_) } as i32) != 0) { bb = 157; continue 'cfg; } else { bb = 158; continue 'cfg; }
                break 'cfg;
            }
            157 => {
                { bb = 89; continue 'cfg; }
                break 'cfg;
            }
            158 => {
                { bb = 159; continue 'cfg; }
                break 'cfg;
            }
            159 => {
                { bb = 153; continue 'cfg; }
                break 'cfg;
            }
            160 => {
                { bb = 159; continue 'cfg; }
                break 'cfg;
            }
            161 => {
                { bb = 88; continue 'cfg; }
                break 'cfg;
            }
            162 => {
                if (2 != 0) { bb = 163; continue 'cfg; } else { bb = 164; continue 'cfg; }
                break 'cfg;
            }
            163 => {
                if (cx.li(1700) >= cx.li(2060)) { bb = 165; continue 'cfg; } else { bb = 166; continue 'cfg; }
                break 'cfg;
            }
            164 => {
                if (cx.b(cx.lpp(835).add(4072)) != 0) { bb = 181; continue 'cfg; } else { bb = 182; continue 'cfg; }
                break 'cfg;
            }
            165 => {
                if (cx.li(1696) != 0) { bb = 168; continue 'cfg; } else { bb = 169; continue 'cfg; }
                break 'cfg;
            }
            166 => {
                { bb = 167; continue 'cfg; }
                break 'cfg;
            }
            167 => {
                { let t = a4.add((1560_i32).wrapping_mul(cx.li(1696).wrapping_add(cx.li(3336)))); cx.set_lpp(423, t); }
                { let t = a4.add((1560_i32).wrapping_mul(cx.li(1700).wrapping_add(cx.li(3336)))); cx.set_lpp(422, t); }
                if (cx.b(Ptr::glob(0x74D474)) != 0) { bb = 172; continue 'cfg; } else { bb = 173; continue 'cfg; }
                break 'cfg;
            }
            168 => {
                { let p = cx.lpp(835).add(4856); let t = 0; cx.set_i(p, t); }
                { bb = 170; continue 'cfg; }
                break 'cfg;
            }
            169 => {
                { bb = 170; continue 'cfg; }
                break 'cfg;
            }
            170 => {
                { let t = cx.li(3336).wrapping_add(cx.li(1696)); cx.set_li(3336, t); }
                { bb = 161; continue 'cfg; }
                break 'cfg;
            }
            171 => {
                { bb = 167; continue 'cfg; }
                break 'cfg;
            }
            172 => {
                v45 = { let q0_ = cx.lpp(835); let q1_ = 2; f_1bb200(cx, q0_, q1_) };
                v155 = ({ let q0_ = cx.lpp(422).add(1332); let q1_ = v45; f_19d9c0(cx, q0_, q1_) } as f32);
                if ((vsmath::fabs(v155) > (cx.f(Ptr::gc(0).add(1364)) as f64)) || { let _ = { let _ = { v46 = { let q0_ = cx.lpp(835); let q1_ = 0; f_1bb200(cx, q0_, q1_) }; v46 }; { v156 = ({ let q0_ = cx.lpp(422).add(1332); let q1_ = v46; f_19d9c0(cx, q0_, q1_) } as f32); v156 } }; (vsmath::fabs(v156) > (cx.f(Ptr::gc(0).add(1368)) as f64)) }) { bb = 175; continue 'cfg; } else { bb = 176; continue 'cfg; }
                break 'cfg;
            }
            173 => {
                { bb = 174; continue 'cfg; }
                break 'cfg;
            }
            174 => {
                { bb = 164; continue 'cfg; }
                break 'cfg;
            }
            175 => {
                { bb = 178; continue 'cfg; }
                break 'cfg;
            }
            176 => {
                { bb = 177; continue 'cfg; }
                break 'cfg;
            }
            177 => {
                { bb = 174; continue 'cfg; }
                break 'cfg;
            }
            178 => {
                let _ = cx.preinc_i(cx.lptr(1700));
                { bb = 162; continue 'cfg; }
                break 'cfg;
            }
            179 => {
                { bb = 177; continue 'cfg; }
                break 'cfg;
            }
            180 => {
                { bb = 162; continue 'cfg; }
                break 'cfg;
            }
            181 => {
                if (cx.f(cx.lpp(835).add(3784)) <= cx.f(Ptr::gc(0).add(1344))) { bb = 184; continue 'cfg; } else { bb = 185; continue 'cfg; }
                break 'cfg;
            }
            182 => {
                { bb = 183; continue 'cfg; }
                break 'cfg;
            }
            183 => {
                { let q0_ = cx.lpp(423).add(1332); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(1700).wrapping_add(cx.li(3336)))).add(1332); f_19e030(cx, q0_, q1_) };
                { let q0_ = cx.lpp(423).add(1276); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(1700).wrapping_add(cx.li(3336)))).add(1276); f_19e030(cx, q0_, q1_) };
                { let p = cx.lpp(423).add(1288); let t = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(1700).wrapping_add(cx.li(3336)))).add(1288)); cx.set_i(p, t); }
                { let p = cx.lpp(423).add(4); let t = cx.lpp(835).add(240); cx.set_ptr(p, t); }
                { let p = cx.lpp(423).add(8); let t = 0; cx.set_i(p, t); }
                if ((cx.i(cx.lpp(835).add(4756)) == 1) && ((cx.f(Ptr::gc(0).add(1092)) as f64) > vsmath::fabs((((cx.f(cx.lpp(835).add(4592)) as f64) - (cx.f(cx.lpp(835).add(4596)) as f64)) as f32)))) { bb = 198; continue 'cfg; } else { bb = 199; continue 'cfg; }
                break 'cfg;
            }
            184 => {
                { let q0_ = cx.lptr(1624); let q1_ = cx.lpp(422).add(1276); let q2_ = cx.lpp(422).add(1332); let q3_ = cx.f(cx.lpp(422).add(1288)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                { let q0_ = cx.lptr(1604); let q1_ = cx.lptr(1624); let q2_ = cx.lpp(835).add(304); f_19d920(cx, q0_, q1_, q2_) };
                { let t = ({ let q0_ = cx.lptr(1604); let q1_ = cx.lpp(422).add(1332); f_19d9c0(cx, q0_, q1_) } as f32); cx.set_lf(1596, t); }
                if (cx.lf(1596) > cx.f(Ptr::gc(0).add(1356))) { bb = 187; continue 'cfg; } else { bb = 188; continue 'cfg; }
                break 'cfg;
            }
            185 => {
                { let q0_ = cx.lptr(1672); let q1_ = cx.lpp(835).add(272); f_19e030(cx, q0_, q1_) };
                { let q0_ = cx.lptr(1652); let q1_ = cx.lpp(422).add(1276); let q2_ = cx.lpp(835).add(304); f_19d920(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(1672); let q1_ = cx.lpp(835).add(288); let q2_ = cx.lptr(1652); f_1a6f00(cx, q0_, q1_, q2_) };
                { let t = ((-{ let q0_ = cx.lptr(1672); let q1_ = cx.lpp(422).add(1332); f_19d9c0(cx, q0_, q1_) }) as f32); cx.set_lf(1644, t); }
                { let t = (((((cx.f(Ptr::gc(0).add(1348)) * cx.lf(1644)) * cx.f(cx.ptr_at(cx.lpp(835).add(3552)))) as f64) + (cx.f(Ptr::gc(0).add(1352)) as f64)) as f32); cx.set_lf(1640, t); }
                if (cx.f(cx.lpp(422).add(1288)) > cx.lf(1640)) { bb = 191; continue 'cfg; } else { bb = 192; continue 'cfg; }
                break 'cfg;
            }
            186 => {
                if (cx.i(cx.lpp(422).add(1556)) == 25) { bb = 195; continue 'cfg; } else { bb = 196; continue 'cfg; }
                break 'cfg;
            }
            187 => {
                { bb = 178; continue 'cfg; }
                break 'cfg;
            }
            188 => {
                { bb = 189; continue 'cfg; }
                break 'cfg;
            }
            189 => {
                { bb = 186; continue 'cfg; }
                break 'cfg;
            }
            190 => {
                { bb = 189; continue 'cfg; }
                break 'cfg;
            }
            191 => {
                { bb = 178; continue 'cfg; }
                break 'cfg;
            }
            192 => {
                { bb = 193; continue 'cfg; }
                break 'cfg;
            }
            193 => {
                { bb = 186; continue 'cfg; }
                break 'cfg;
            }
            194 => {
                { bb = 193; continue 'cfg; }
                break 'cfg;
            }
            195 => {
                { let q0_ = cx.lptr(1580); let q1_ = cx.lpp(422).add(1276); let q2_ = cx.lpp(835).add(304); f_19d920(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(1580); let q1_ = cx.lptr(1580); let q2_ = cx.lpp(422).add(1332); f_1a39d0(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lpp(422).add(1276); let q1_ = cx.lpp(835).add(304); let q2_ = cx.lptr(1580); f_1a38d0(cx, q0_, q1_, q2_) };
                { bb = 197; continue 'cfg; }
                break 'cfg;
            }
            196 => {
                { bb = 197; continue 'cfg; }
                break 'cfg;
            }
            197 => {
                { bb = 183; continue 'cfg; }
                break 'cfg;
            }
            198 => {
                { let p = cx.lpp(423).add(1540); let t = cx.i(Ptr::gc(0).add(1088)); cx.set_i(p, t); }
                { let p = cx.lpp(423).add(1544); let t = cx.i(Ptr::gc(0).add(1088)); cx.set_i(p, t); }
                { bb = 200; continue 'cfg; }
                break 'cfg;
            }
            199 => {
                if (((cx.i(cx.lpp(835).add(3740)) != 0) && (cx.f(cx.lpp(835).add(3784)) > cx.f(cx.ptr_at(cx.lpp(835).add(3552)).add(220)))) && (cx.i(cx.lpp(835).add(3604)) >= 3)) { bb = 201; continue 'cfg; } else { bb = 202; continue 'cfg; }
                break 'cfg;
            }
            200 => {
                { let t = (vsmath::fabs(cx.f(cx.lpp(423).add(1336))) as f32); cx.set_lf(1572, t); }
                if ((cx.i(cx.lpp(835).add(4852)) != 0) && (cx.f(cx.lpp(423).add(1336)) > cx.f(Ptr::gc(0).add(1376)))) { bb = 204; continue 'cfg; } else { bb = 205; continue 'cfg; }
                break 'cfg;
            }
            201 => {
                { let p = cx.lpp(423).add(1540); let t = cx.i(Ptr::gc(0).add(1304)); cx.set_i(p, t); }
                { bb = 203; continue 'cfg; }
                break 'cfg;
            }
            202 => {
                { let p = cx.lpp(423).add(1540); let t = cx.i(Ptr::gc(0).add(1300)); cx.set_i(p, t); }
                { bb = 203; continue 'cfg; }
                break 'cfg;
            }
            203 => {
                { let p = cx.lpp(423).add(1544); let t = cx.i(Ptr::gc(0).add(1308)); cx.set_i(p, t); }
                { bb = 200; continue 'cfg; }
                break 'cfg;
            }
            204 => {
                { let p = cx.lpp(423).add(1272); let t = cx.i(Ptr::gc(0).add(1380)); cx.set_i(p, t); }
                { bb = 206; continue 'cfg; }
                break 'cfg;
            }
            205 => {
                { let p = cx.lpp(423).add(1272); let t = ((((cx.f(Ptr::gc(0).add(1324)) * cx.lf(1572)) as f64) + (((1.0f32 - cx.lf(1572)) * cx.f(Ptr::gc(0).add(1328))) as f64)) as f32); cx.set_f(p, t); }
                { bb = 206; continue 'cfg; }
                break 'cfg;
            }
            206 => {
                if ((cx.i(cx.lpp(835).add(4756)) == 1) || (cx.b(Ptr::glob(0x74D474)) != 0)) { bb = 207; continue 'cfg; } else { bb = 208; continue 'cfg; }
                break 'cfg;
            }
            207 => {
                { let p = cx.lpp(423).add(1268); let t = 0; cx.set_i(p, t); }
                { bb = 209; continue 'cfg; }
                break 'cfg;
            }
            208 => {
                { let p = cx.lpp(423).add(1268); let t = ((((cx.f(Ptr::gc(0).add(1316)) * cx.lf(1572)) as f64) + (((1.0f32 - cx.lf(1572)) * cx.f(Ptr::gc(0).add(1320))) as f64)) as f32); cx.set_f(p, t); }
                { bb = 209; continue 'cfg; }
                break 'cfg;
            }
            209 => {
                if (cx.lf(1572) < 0.1f32) { bb = 210; continue 'cfg; } else { bb = 211; continue 'cfg; }
                break 'cfg;
            }
            210 => {
                { let q0_ = cx.lptr(1556); let q1_ = cx.lpp(423).add(1276); let q2_ = cx.lpp(835).add(304); f_19d920(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(1536); let q1_ = cx.lptr(1556); let q2_ = cx.lpp(423).add(1332); f_1a03f0(cx, q0_, q1_, q2_) };
                v47 = { let q0_ = cx.lpp(835).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                { let t = ({ let q0_ = cx.lptr(1536); let q1_ = v47; f_19d9c0(cx, q0_, q1_) } as f32); cx.set_lf(1528, t); }
                v157 = cx.lpp(835).add(304);
                v48 = { let q0_ = cx.lpp(835).add(240); let q1_ = 1; ov_car_rot_row(cx, q0_, q1_) };
                { let t = (f_19d9c0(cx, v48, v157) as f32); cx.set_lf(4, t); }
                v158 = cx.lpp(423).add(1276);
                v49 = { let q0_ = cx.lpp(835).add(240); let q1_ = 1; ov_car_rot_row(cx, q0_, q1_) };
                v50 = f_19d9c0(cx, v49, v158);
                v159 = (((cx.lf(4) as f64) - v50) as f32);
                v51 = { let q0_ = cx.lpp(835).add(240); let q1_ = 1; ov_car_rot_row(cx, q0_, q1_) };
                { let q0_ = cx.lptr(1512); let q1_ = v51; let q2_ = v159; f_19d970(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(1536); let q1_ = cx.lptr(1512); let q2_ = cx.lpp(423).add(1332); f_1a03f0(cx, q0_, q1_, q2_) };
                v52 = { let q0_ = cx.lpp(835).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                { let t = ({ let q0_ = cx.lptr(1536); let q1_ = v52; f_19d9c0(cx, q0_, q1_) } as f32); cx.set_lf(1504, t); }
                { let t = (((cx.lf(1528) as f64) + (cx.lf(1504) as f64)) as f32); cx.set_lf(1500, t); }
                if ((cx.lf(1504) * cx.lf(1528)) < 0.0f32) { bb = 213; continue 'cfg; } else { bb = 214; continue 'cfg; }
                break 'cfg;
            }
            211 => {
                { bb = 212; continue 'cfg; }
                break 'cfg;
            }
            212 => {
                if (cx.f(Ptr::gc(0).add(1992)) != 0.0f32) { bb = 219; continue 'cfg; } else { bb = 220; continue 'cfg; }
                break 'cfg;
            }
            213 => {
                if (((cx.lf(1500) * cx.lf(1528)) > 0.0f32) || { let _ = { { let t = (vsmath::fabs(cx.lf(1500)) as f32); cx.set_lf(0, t); } }; ({ v53 = (vsmath::fabs(cx.lf(1528)) * (cx.f(Ptr::gc(0).add(1340)) as f64)); v53 } > (cx.lf(0) as f64)) }) { bb = 216; continue 'cfg; } else { bb = 217; continue 'cfg; }
                break 'cfg;
            }
            214 => {
                { bb = 215; continue 'cfg; }
                break 'cfg;
            }
            215 => {
                { bb = 212; continue 'cfg; }
                break 'cfg;
            }
            216 => {
                { let q0_ = cx.lpp(423).add(1276); let q1_ = cx.lptr(1512); f_1a3880(cx, q0_, q1_) };
                { bb = 218; continue 'cfg; }
                break 'cfg;
            }
            217 => {
                { bb = 218; continue 'cfg; }
                break 'cfg;
            }
            218 => {
                { bb = 215; continue 'cfg; }
                break 'cfg;
            }
            219 => {
                { let q0_ = cx.lptr(1484); let q1_ = cx.lpp(423).add(1276); let q2_ = cx.lpp(423).add(1332); let q3_ = 1.0f32; f_1a3970(cx, q0_, q1_, q2_, q3_) };
                v160 = { let p = { let q0_ = cx.lptr(76); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v124 = cx.i(Ptr::glob(0x77FD8C));
                v95 = { let q0_ = cx.lptr(60); let q1_ = cx.li(1484); let q2_ = cx.li(1488); let q3_ = cx.li(1492); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v54 = { let q0_ = cx.lptr(44); let q1_ = cx.i(cx.lpp(423).add(1276)); let q2_ = cx.i(cx.lpp(423).add(1280)); let q3_ = cx.i(cx.lpp(423).add(1284)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v54, v95, v124, v160);
                { let q0_ = cx.lptr(1484); let q1_ = cx.lpp(423).add(1276); let q2_ = cx.lpp(423).add(1332); let q3_ = (((cx.f(Ptr::gc(0).add(1992)) as f64) * (cx.f(cx.lpp(423).add(1288)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                { let q0_ = cx.lptr(1464); let q1_ = cx.lpp(423).add(1276); f_19e030(cx, q0_, q1_) };
                { let t = (((cx.lf(1468) as f64) + 0.05f64) as f32); cx.set_lf(1468, t); }
                v161 = { let p = { let q0_ = cx.lptr(40); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                v125 = cx.i(Ptr::glob(0x77FD48));
                v96 = { let q0_ = cx.lptr(24); let q1_ = cx.li(1484); let q2_ = cx.li(1488); let q3_ = cx.li(1492); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                v55 = { let q0_ = cx.lptr(8); let q1_ = cx.li(1464); let q2_ = cx.li(1468); let q3_ = cx.li(1472); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v55, v96, v125, v161);
                { bb = 221; continue 'cfg; }
                break 'cfg;
            }
            220 => {
                { bb = 221; continue 'cfg; }
                break 'cfg;
            }
            221 => {
                let _ = cx.preinc_i(cx.lptr(1696));
                { bb = 178; continue 'cfg; }
                break 'cfg;
            }
            222 => {
                { bb = 85; continue 'cfg; }
                break 'cfg;
            }
            223 => {
                { let q0_ = cx.lptr(1280); f_1b42f0(cx, q0_) };
                { let t = cx.f(cx.ptr_at(cx.lpp(835).add(3552))); cx.set_lf(1280, t); }
                { let t = cx.f(Ptr::gc(0).add(1152)); cx.set_lf(1356, t); }
                { let t = cx.f(Ptr::gc(0).add(1156)); cx.set_lf(1288, t); }
                { let t = cx.f(Ptr::gc(0).add(1160)); cx.set_lf(1284, t); }
                { let t = cx.f(Ptr::gc(0).add(1164)); cx.set_lf(1360, t); }
                if ((cx.f(Ptr::gc(0).add(1312)) == 0.0f32) || { let _ = { a2 = ({ let q0_ = cx.lpp(835).add(272); f_1a0480(cx, q0_) } as f64); a2 }; (((cx.f(Ptr::gc(0).add(1312)) as f64) * (cx.f(Ptr::gc(0).add(1312)) as f64)) <= a2) }) { bb = 226; continue 'cfg; } else { bb = 227; continue 'cfg; }
                break 'cfg;
            }
            224 => {
                { bb = 225; continue 'cfg; }
                break 'cfg;
            }
            225 => {
                { bb = 82; continue 'cfg; }
                break 'cfg;
            }
            226 => {
                { let t = cx.f(Ptr::gc(0).add(1168)); cx.set_lf(1404, t); }
                { bb = 228; continue 'cfg; }
                break 'cfg;
            }
            227 => {
                { let t = 1; cx.set_li(1404, t); }
                { bb = 228; continue 'cfg; }
                break 'cfg;
            }
            228 => {
                { let t = cx.f(Ptr::gc(0).add(1172)); cx.set_lf(1368, t); }
                { let t = cx.f(Ptr::gc(0).add(1176)); cx.set_lf(1372, t); }
                { let t = cx.f(Ptr::gc(0).add(1180)); cx.set_lf(1364, t); }
                { let q0_ = cx.lptr(1316); let q1_ = a6; f_19e030(cx, q0_, q1_) };
                { let q0_ = cx.lptr(1292); let q1_ = Ptr::gc(0); f_19e030(cx, q0_, q1_) };
                { let t = 0; let o = cx.li(1252); cx.set_li(1252, (o & !(255 << 24)) | ((t & 255) << 24)); }
                if ((((cx.f(Ptr::gc(0).add(1420)) > 0.0f32) && (cx.i(cx.lpp(835).add(4756)) != 1)) && (cx.i(cx.lpp(835).add(4408)) == 0)) && (cx.f(cx.lpp(835).add(3784)) > cx.f(Ptr::gc(0).add(1408)))) { bb = 229; continue 'cfg; } else { bb = 230; continue 'cfg; }
                break 'cfg;
            }
            229 => {
                { let q0_ = cx.lpp(835).add(240); let q1_ = cx.lptr(1260); f_1b42b0(cx, q0_, q1_) };
                { let t = 1; let o = cx.li(1252); cx.set_li(1252, (o & !(255 << 24)) | ((t & 255) << 24)); }
                { bb = 231; continue 'cfg; }
                break 'cfg;
            }
            230 => {
                { bb = 231; continue 'cfg; }
                break 'cfg;
            }
            231 => {
                { let p = cx.lpp(835).add(4536); let t = 0; cx.set_b(p, t); }
                if (cx.i(cx.lpp(835).add(4756)) == 1) { bb = 232; continue 'cfg; } else { bb = 233; continue 'cfg; }
                break 'cfg;
            }
            232 => {
                { let q0_ = cx.lpp(835).add(288); let q1_ = cx.lpp(835).add(4628); f_1a3880(cx, q0_, q1_) };
                { let p = cx.lpp(835).add(292); let t = 0; cx.set_i(p, t); }
                if (cx.f(Ptr::gc(0).add(1064)) != 1.0f32) { bb = 235; continue 'cfg; } else { bb = 236; continue 'cfg; }
                break 'cfg;
            }
            233 => {
                { bb = 234; continue 'cfg; }
                break 'cfg;
            }
            234 => {
                { let t = f32::from_bits(0xFFFF_FFFF); cx.set_lf(1232, t); }
                { let t = 0.0f32; cx.set_lf(1228, t); }
                { let t = 0.0f32; cx.set_lf(1224, t); }
                if (cx.b(cx.lpp(835).add(4072)) != 0) { bb = 262; continue 'cfg; } else { bb = 263; continue 'cfg; }
                break 'cfg;
            }
            235 => {
                { let q0_ = cx.lpp(835).add(288); let q1_ = cx.f(Ptr::gc(0).add(1064)); f_19dab0(cx, q0_, q1_) };
                { bb = 237; continue 'cfg; }
                break 'cfg;
            }
            236 => {
                { bb = 237; continue 'cfg; }
                break 'cfg;
            }
            237 => {
                if (cx.f(Ptr::gc(0).add(1072)) != 0.0f32) { bb = 238; continue 'cfg; } else { bb = 239; continue 'cfg; }
                break 'cfg;
            }
            238 => {
                a2 = ({ let q0_ = cx.lpp(835).add(288); f_19d9f0(cx, q0_) } as f64);
                { let t = (a2 as f32); cx.set_lf(1248, t); }
                if (cx.lf(1248) > cx.f(Ptr::gc(0).add(1072))) { bb = 241; continue 'cfg; } else { bb = 242; continue 'cfg; }
                break 'cfg;
            }
            239 => {
                { bb = 240; continue 'cfg; }
                break 'cfg;
            }
            240 => {
                if (cx.f(Ptr::gc(0).add(1076)) != 0.0f32) { bb = 244; continue 'cfg; } else { bb = 245; continue 'cfg; }
                break 'cfg;
            }
            241 => {
                { let q0_ = cx.lpp(835).add(288); let q1_ = (((cx.f(Ptr::gc(0).add(1072)) as f64) / (cx.lf(1248) as f64)) as f32); f_19dab0(cx, q0_, q1_) };
                { bb = 243; continue 'cfg; }
                break 'cfg;
            }
            242 => {
                { bb = 243; continue 'cfg; }
                break 'cfg;
            }
            243 => {
                { bb = 240; continue 'cfg; }
                break 'cfg;
            }
            244 => {
                a2 = ({ let q0_ = cx.lpp(835).add(288); f_19d9f0(cx, q0_) } as f64);
                { let t = (a2 as f32); cx.set_lf(1244, t); }
                if ((cx.lf(1244) != 0.0f32) && (cx.f(Ptr::gc(0).add(1076)) > cx.lf(1244))) { bb = 247; continue 'cfg; } else { bb = 248; continue 'cfg; }
                break 'cfg;
            }
            245 => {
                { bb = 246; continue 'cfg; }
                break 'cfg;
            }
            246 => {
                if (cx.f(Ptr::gc(0).add(1068)) != 1.0f32) { bb = 250; continue 'cfg; } else { bb = 251; continue 'cfg; }
                break 'cfg;
            }
            247 => {
                { let q0_ = cx.lpp(835).add(288); let q1_ = (((cx.f(Ptr::gc(0).add(1076)) as f64) / (cx.lf(1244) as f64)) as f32); f_19dab0(cx, q0_, q1_) };
                { bb = 249; continue 'cfg; }
                break 'cfg;
            }
            248 => {
                { bb = 249; continue 'cfg; }
                break 'cfg;
            }
            249 => {
                { bb = 246; continue 'cfg; }
                break 'cfg;
            }
            250 => {
                a2 = { let q0_ = cx.lpp(835).add(272); let q1_ = cx.lpp(835).add(4640); f_19d9c0(cx, q0_, q1_) };
                { let t = (a2 as f32); cx.set_lf(1240, t); }
                { let q0_ = cx.lpp(835).add(272); let q1_ = cx.lpp(835).add(4640); let q2_ = ((((cx.f(Ptr::gc(0).add(1068)) - 1.0f32) as f64) * (cx.lf(1240) as f64)) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                { bb = 252; continue 'cfg; }
                break 'cfg;
            }
            251 => {
                { bb = 252; continue 'cfg; }
                break 'cfg;
            }
            252 => {
                if ((cx.f(Ptr::gc(0).add(1080)) != 0.0f32) && (cx.b(cx.lpp(835).add(4072)) != 0)) { bb = 253; continue 'cfg; } else { bb = 254; continue 'cfg; }
                break 'cfg;
            }
            253 => {
                v56 = { let q0_ = cx.lpp(835); let q1_ = 1; f_1bb200(cx, q0_, q1_) };
                a2 = { let q0_ = cx.lpp(835).add(272); let q1_ = v56; f_19d9c0(cx, q0_, q1_) };
                { let t = (a2 as f32); cx.set_lf(1236, t); }
                if (cx.lf(1236) >= 0.0f32) { bb = 256; continue 'cfg; } else { bb = 257; continue 'cfg; }
                break 'cfg;
            }
            254 => {
                { bb = 255; continue 'cfg; }
                break 'cfg;
            }
            255 => {
                { bb = 234; continue 'cfg; }
                break 'cfg;
            }
            256 => {
                { let p = cx.lpp(835).add(4540); let t = 0; cx.set_i(p, t); }
                { bb = 258; continue 'cfg; }
                break 'cfg;
            }
            257 => {
                if (((0.0f32 - cx.lf(1236)) * cx.f(Ptr::gc(0).add(1080))) > cx.f(Ptr::gc(0).add(1084))) { bb = 259; continue 'cfg; } else { bb = 260; continue 'cfg; }
                break 'cfg;
            }
            258 => {
                { bb = 255; continue 'cfg; }
                break 'cfg;
            }
            259 => {
                { let t = (((cx.lf(1236) as f64) / ((((0.0f32 - cx.lf(1236)) * cx.f(Ptr::gc(0).add(1080))) / cx.f(Ptr::gc(0).add(1084))) as f64)) as f32); cx.set_lf(1236, t); }
                { bb = 261; continue 'cfg; }
                break 'cfg;
            }
            260 => {
                { bb = 261; continue 'cfg; }
                break 'cfg;
            }
            261 => {
                { let p = cx.lpp(835).add(4540); let t = cx.lf(1236); cx.set_f(p, t); }
                { let p = cx.lpp(835).add(4536); let t = 1; cx.set_b(p, t); }
                { bb = 258; continue 'cfg; }
                break 'cfg;
            }
            262 => {
                { let t = 0.0f32; cx.set_lf(1220, t); }
                { bb = 265; continue 'cfg; }
                break 'cfg;
            }
            263 => {
                { bb = 264; continue 'cfg; }
                break 'cfg;
            }
            264 => {
                a2 = { let q0_ = a4; let q1_ = cx.li(3336); let q2_ = Ptr::NULL; let q3_ = 0; let q4_ = Ptr::NULL; let q5_ = 0; let q6_ = cx.lptr(1280); f_389630(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
                if ((cx.b(cx.lpp(835).add(4072)) != 0) && (cx.li(1232) != -1)) { bb = 279; continue 'cfg; } else { bb = 280; continue 'cfg; }
                break 'cfg;
            }
            265 => {
                if (cx.li(1220) < cx.li(3336)) { bb = 266; continue 'cfg; } else { bb = 268; continue 'cfg; }
                break 'cfg;
            }
            266 => {
                v57 = { let q0_ = cx.lpp(835); let q1_ = 0; f_1bb200(cx, q0_, q1_) };
                v162 = ({ let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(1220))).add(1332); let q1_ = v57; f_19d9c0(cx, q0_, q1_) } as f32);
                { let t = (vsmath::fabs(v162) as f32); cx.set_lf(1224, t); }
                a2 = vsmath::fabs(cx.f(a4.add((1560_i32).wrapping_mul(cx.li(1220))).add(1336)));
                if (((cx.f(Ptr::gc(0).add(1384)) as f64) > a2) && (cx.lf(1224) > cx.f(Ptr::gc(0).add(1388)))) { bb = 269; continue 'cfg; } else { bb = 270; continue 'cfg; }
                break 'cfg;
            }
            267 => {
                let _ = cx.preinc_i(cx.lptr(1220));
                { bb = 265; continue 'cfg; }
                break 'cfg;
            }
            268 => {
                { bb = 264; continue 'cfg; }
                break 'cfg;
            }
            269 => {
                v58 = { let q0_ = cx.lpp(835); let q1_ = 1; f_1bb200(cx, q0_, q1_) };
                v163 = ({ let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(1220))).add(1332); let q1_ = v58; f_19d9c0(cx, q0_, q1_) } as f32);
                a2 = vsmath::fabs(v163);
                if ((cx.f(Ptr::gc(0).add(1388)) as f64) > a2) { bb = 272; continue 'cfg; } else { bb = 273; continue 'cfg; }
                break 'cfg;
            }
            270 => {
                { bb = 271; continue 'cfg; }
                break 'cfg;
            }
            271 => {
                { bb = 267; continue 'cfg; }
                break 'cfg;
            }
            272 => {
                { let t = ({ let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(1220))).add(1332); let q1_ = cx.lpp(835).add(272); f_19d9c0(cx, q0_, q1_) } as f32); cx.set_lf(1228, t); }
                v59 = vsmath::fabs(cx.lf(1228));
                a2 = (cx.f(Ptr::gc(0).add(1392)) as f64);
                if (v59 >= a2) { bb = 275; continue 'cfg; } else { bb = 276; continue 'cfg; }
                break 'cfg;
            }
            273 => {
                { bb = 274; continue 'cfg; }
                break 'cfg;
            }
            274 => {
                { bb = 271; continue 'cfg; }
                break 'cfg;
            }
            275 => {
                { let t = cx.lf(1220); cx.set_lf(1232, t); }
                { bb = 268; continue 'cfg; }
                break 'cfg;
            }
            276 => {
                { bb = 277; continue 'cfg; }
                break 'cfg;
            }
            277 => {
                { bb = 274; continue 'cfg; }
                break 'cfg;
            }
            278 => {
                { bb = 277; continue 'cfg; }
                break 'cfg;
            }
            279 => {
                { let t = ({ let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(1232))).add(1332); let q1_ = cx.lpp(835).add(272); f_19d9c0(cx, q0_, q1_) } as f32); cx.set_lf(1216, t); }
                { let t = (vsmath::fabs((((cx.lf(1216) as f64) - (cx.lf(1228) as f64)) as f32)) as f32); cx.set_lf(1212, t); }
                if (cx.lf(1212) > cx.f(Ptr::gc(0).add(1392))) { bb = 282; continue 'cfg; } else { bb = 283; continue 'cfg; }
                break 'cfg;
            }
            280 => {
                { bb = 281; continue 'cfg; }
                break 'cfg;
            }
            281 => {
                if ((((cx.li(1252) >> 24) & 255) != 0) && (cx.i(cx.lpp(835).add(4756)) != 1)) { bb = 288; continue 'cfg; } else { bb = 289; continue 'cfg; }
                break 'cfg;
            }
            282 => {
                { let t = (((((cx.lf(1212) - cx.f(Ptr::gc(0).add(1392))) * cx.lf(1224)) as f64) / ((cx.f(Ptr::gc(0).add(1396)) - cx.f(Ptr::gc(0).add(1392))) as f64)) as f32); cx.set_lf(1208, t); }
                if (cx.lf(1208) > 1.0f32) { bb = 285; continue 'cfg; } else { bb = 286; continue 'cfg; }
                break 'cfg;
            }
            283 => {
                { bb = 284; continue 'cfg; }
                break 'cfg;
            }
            284 => {
                { bb = 281; continue 'cfg; }
                break 'cfg;
            }
            285 => {
                { let t = 1.0f32; cx.set_lf(1208, t); }
                { bb = 287; continue 'cfg; }
                break 'cfg;
            }
            286 => {
                { bb = 287; continue 'cfg; }
                break 'cfg;
            }
            287 => {
                { let t = (((cx.lf(1208) as f64) * (cx.f(Ptr::gc(0).add(1400)) as f64)) as f32); cx.set_lf(1208, t); }
                v164 = ((0.0f64 - (cx.lf(1208) as f64)) as f32);
                v60 = { let q0_ = cx.lpp(835).add(240); let q1_ = 2; ov_car_rot_row(cx, q0_, q1_) };
                { let q0_ = cx.lpp(835).add(288); let q1_ = v60; let q2_ = v164; f_1a0390(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(1192); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(1232))).add(1276); let q2_ = cx.lpp(835).add(304); f_19d920(cx, q0_, q1_, q2_) };
                v61 = ({ let q0_ = cx.lptr(1192); f_19d9f0(cx, q0_) } as f64);
                { let p = cx.lpp(835).add(276); let t = (((v61 * (cx.lf(1208) as f64)) + (cx.f(cx.lpp(835).add(276)) as f64)) as f32); cx.set_f(p, t); }
                { bb = 284; continue 'cfg; }
                break 'cfg;
            }
            288 => {
                v62 = { let q0_ = cx.lpp(835).add(240); ov_car_velocity(cx, q0_) };
                { let q0_ = cx.lptr(1172); let q1_ = cx.lptr(1260); let q2_ = v62; f_19d920(cx, q0_, q1_, q2_) };
                v63 = ({ let q0_ = cx.lptr(1172); f_19d9f0(cx, q0_) } as f64);
                { let p = cx.lpp(835).add(4428); let t = (v63 as f32); cx.set_f(p, t); }
                if (cx.f(cx.lpp(835).add(4428)) > cx.f(Ptr::gc(0).add(1412))) { bb = 291; continue 'cfg; } else { bb = 292; continue 'cfg; }
                break 'cfg;
            }
            289 => {
                { bb = 290; continue 'cfg; }
                break 'cfg;
            }
            290 => {
                if (cx.b(cx.lpp(835).add(4536)) != 0) { bb = 294; continue 'cfg; } else { bb = 295; continue 'cfg; }
                break 'cfg;
            }
            291 => {
                { let p = cx.lpp(835).add(4424); let t = 0; cx.set_i(p, t); }
                { let q0_ = cx.lpp(835).add(4412); let q1_ = cx.lptr(1172); let q2_ = ((1.0f64 / (cx.f(cx.lpp(835).add(4428)) as f64)) as f32); f_19d970(cx, q0_, q1_, q2_) };
                v64 = { let q0_ = cx.lpp(835).add(240); let q1_ = 2; ov_car_rot_row(cx, q0_, q1_) };
                v165 = (f_19d9c0(cx, v64, a4.add(1332)) as f32);
                { let t = ((1.0f64 - vsmath::fabs(v165)) as f32); cx.set_lf(1164, t); }
                { let t = (((((1.0f32 - cx.f(Ptr::gc(0).add(1440))) * cx.lf(1164)) as f64) + (cx.f(Ptr::gc(0).add(1440)) as f64)) as f32); cx.set_lf(1160, t); }
                { let q0_ = cx.lpp(835).add(4412); let q1_ = (((cx.lf(1160) as f64) * (cx.f(Ptr::gc(0).add(1436)) as f64)) as f32); f_19dab0(cx, q0_, q1_) };
                { let p = cx.lpp(835).add(4432); let t = (((((1.0f32 - cx.f(Ptr::gc(0).add(1444))) * cx.lf(1164)) as f64) + (cx.f(Ptr::gc(0).add(1444)) as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.lpp(835).add(4408); let t = cx.i(Ptr::gc(0).add(1428)); cx.set_i(p, t); }
                { bb = 293; continue 'cfg; }
                break 'cfg;
            }
            292 => {
                { bb = 293; continue 'cfg; }
                break 'cfg;
            }
            293 => {
                { bb = 290; continue 'cfg; }
                break 'cfg;
            }
            294 => {
                v166 = ((((0.0f32 - cx.f(cx.lpp(835).add(4540))) as f64) * (cx.f(Ptr::gc(0).add(1080)) as f64)) as f32);
                v65 = { let q0_ = cx.lpp(835); let q1_ = 1; f_1bb200(cx, q0_, q1_) };
                { let q0_ = cx.lpp(835).add(272); let q1_ = v65; let q2_ = v166; f_1a0390(cx, q0_, q1_, q2_) };
                { bb = 296; continue 'cfg; }
                break 'cfg;
            }
            295 => {
                { bb = 296; continue 'cfg; }
                break 'cfg;
            }
            296 => {
                { let t = 0.0f32; cx.set_lf(1152, t); }
                { let t = ((0.0f64 - 3.4028235e38f64) as f32); cx.set_lf(1148, t); }
                { let t = 0.0f32; cx.set_lf(1156, t); }
                { bb = 297; continue 'cfg; }
                break 'cfg;
            }
            297 => {
                if (cx.li(1156) < cx.li(3336)) { bb = 298; continue 'cfg; } else { bb = 300; continue 'cfg; }
                break 'cfg;
            }
            298 => {
                { let t = (vsmath::fabs(cx.f(a4.add((1560_i32).wrapping_mul(cx.li(1156))).add(236))) as f32); cx.set_lf(1144, t); }
                if (cx.lf(1144) > cx.lf(1148)) { bb = 301; continue 'cfg; } else { bb = 302; continue 'cfg; }
                break 'cfg;
            }
            299 => {
                let _ = cx.preinc_i(cx.lptr(1156));
                { bb = 297; continue 'cfg; }
                break 'cfg;
            }
            300 => {
                if (cx.lf(1148) > 0.0f32) { bb = 304; continue 'cfg; } else { bb = 305; continue 'cfg; }
                break 'cfg;
            }
            301 => {
                { let t = cx.lf(1144); cx.set_lf(1148, t); }
                { let t = cx.lf(1156); cx.set_lf(1152, t); }
                { bb = 303; continue 'cfg; }
                break 'cfg;
            }
            302 => {
                { bb = 303; continue 'cfg; }
                break 'cfg;
            }
            303 => {
                { bb = 299; continue 'cfg; }
                break 'cfg;
            }
            304 => {
                { let q0_ = cx.lptr(1060); f_238b0(cx, q0_) };
                { let t = cx.lf(3344); cx.set_lf(1060, t); }
                { let t = f32::from_bits(0xFFFF_FFFF); cx.set_lf(1064, t); }
                { let t = cx.lf(1148); cx.set_lf(1100, t); }
                { let q0_ = cx.lptr(1068); let q1_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(1152))).add(1276)); let q2_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(1152))).add(1280)); let q3_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(1152))).add(1284)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                { let q0_ = cx.lptr(1084); let q1_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(1152))).add(1332)); let q2_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(1152))).add(1336)); let q3_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(1152))).add(1340)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                { let t = 0.0f32; cx.set_lf(1104, t); }
                { let t = 8.0f32; cx.set_lf(1132, t); }
                { let t = cx.lpp(835).add(240); cx.set_lpp(277, t); }
                { let t = 0.0f32; cx.set_lf(1112, t); }
                { let q0_ = cx.lpp(837); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(1152))); let q2_ = cx.lptr(1116); f_1b39b0(cx, q0_, q1_, q2_) };
                { let t = cx.f(a4.add((1560_i32).wrapping_mul(cx.li(1152))).add(1556)); cx.set_lf(1052, t); }
                v66 = cx.li(1052);
                v66 = (v66 & !(0xFFi64 as i32)) | ((((((cx.li(1052) >> 0) & 255)) as i64) & 0xFF) << 0) as i32;
                { let q0_ = cx.b(Ptr::glob(0x6FDC88)); let q1_ = v66; let q2_ = ((cx.li(1052) >> 0) & 255); let q3_ = cx.lptr(1060); let q4_ = cx.li(3344); crate::ov_ignore!(cx, q0_, q1_, q2_, q3_, q4_) };
                { let t = ((cx.li(1052) >> 0) & 255); let o = cx.li(1136); cx.set_li(1136, (o & !(255 << 0)) | ((t & 255) << 0)); }
                { let q0_ = Ptr::glob(0x6FAA34); let q1_ = 0; let q2_ = cx.lptr(1060); ov_message(cx, q0_, q1_, q2_) };
                { bb = 306; continue 'cfg; }
                break 'cfg;
            }
            305 => {
                { bb = 306; continue 'cfg; }
                break 'cfg;
            }
            306 => {
                { bb = 225; continue 'cfg; }
                break 'cfg;
            }
            307 => {
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(28); let q2_ = ((0.0f64 - (cx.f(Ptr::gc(0).add(1336)) as f64)) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                { bb = 309; continue 'cfg; }
                break 'cfg;
            }
            308 => {
                { bb = 309; continue 'cfg; }
                break 'cfg;
            }
            309 => {
                if (cx.lf(3324) != 0.0f32) { bb = 310; continue 'cfg; } else { bb = 311; continue 'cfg; }
                break 'cfg;
            }
            310 => {
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(28); let q2_ = ((0.0f64 - (cx.lf(3324) as f64)) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(56); let t = (((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(56)) as f64) - (cx.lf(3324) as f64)) as f32); cx.set_f(p, t); }
                { bb = 312; continue 'cfg; }
                break 'cfg;
            }
            311 => {
                { bb = 312; continue 'cfg; }
                break 'cfg;
            }
            312 => {
                if ((((cx.i(Ptr::gc(0).add(1360)) != 0) && (cx.b(cx.lpp(835).add(4072)) != 0)) && (cx.b(Ptr::glob(0x74D474)) == 0)) && (((cx.li(3332) >> 24) & 255) != 0)) { bb = 313; continue 'cfg; } else { bb = 314; continue 'cfg; }
                break 'cfg;
            }
            313 => {
                if (cx.li(3336) != 0) { bb = 316; continue 'cfg; } else { bb = 317; continue 'cfg; }
                break 'cfg;
            }
            314 => {
                { bb = 315; continue 'cfg; }
                break 'cfg;
            }
            315 => {
                { let t = 0.0f32; cx.set_lf(3336, t); }
                { bb = 7; continue 'cfg; }
                break 'cfg;
            }
            316 => {
                { let _ = { let q0_ = cx.lpp(835).add(240); let q1_ = a2; vcall(cx, 32, q0_, &[Varg::from(q1_)]) }; };
                { let q0_ = cx.lpp(835); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(16); f_1bb100(cx, q0_, q1_) };
                { let _ = { let q0_ = cx.lpp(835).add(240); let q1_ = cx.i(cx.ptr_at(cx.lpp(835).add(3556))); vcall(cx, 28, q0_, &[Varg::from(q1_)]) }; };
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(16); let q2_ = cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(116)); f_1a0390(cx, q0_, q1_, q2_) };
                { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(28); let q2_ = cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(835).add(3556))).add(120)); f_1a0390(cx, q0_, q1_, q2_) };
                { let q0_ = cx.ptr_at(cx.lpp(835).add(3556)); f_1b4460(cx, q0_) };
                { bb = 318; continue 'cfg; }
                break 'cfg;
            }
            317 => {
                { bb = 318; continue 'cfg; }
                break 'cfg;
            }
            318 => {
                { let p = Ptr::glob(0x74D474); let t = 1; cx.set_b(p, t); }
                let _ = cx.predec_i(cx.lptr(3344));
                { bb = 315; continue 'cfg; }
                break 'cfg;
            }
            319 => {
                break 'cfg;
            }
            320 => {
                break 'cfg;
            }
            _ => unreachable!(),
        }
    }
    #[allow(unreachable_code)]
    0
}

/// PhysicsManager__1A6ED0 0x001A6ED0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a6ed0(cx: &mut Cx, mut a1: Ptr, a2: i32) -> Ptr {
    { let p = a1; let t = a2; cx.set_i(p, t); }
    { let p = a1.add(4); let t = a2; cx.set_i(p, t); }
    { let p = a1.add(8); let t = a2; cx.set_i(p, t); }
    { let p = a1.add(12); let t = a2; cx.set_i(p, t); }
    return a1;
}

/// PhysicsManager__1A6F00 0x001A6F00
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a6f00(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = (((((cx.f(a2.add(4)) * cx.f(a3.add(8))) - (cx.f(a2.add(8)) * cx.f(a3.add(4)))) as f64) + (cx.f(a1) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((((cx.f(a2.add(8)) * cx.f(a3)) - (cx.f(a2) * cx.f(a3.add(8)))) as f64) + (cx.f(a1.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((((cx.f(a2) * cx.f(a3.add(4))) - (cx.f(a2.add(4)) * cx.f(a3))) as f64) + (cx.f(a1.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a3;
}

/// PhysicsManager__1A6FA0 0x001A6FA0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1a6fa0(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = (((((cx.f(a3) * cx.f(a2)) + (cx.f(a3.add(12)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(24)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((((cx.f(a3.add(4)) * cx.f(a2)) + (cx.f(a3.add(16)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(28)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((((cx.f(a3.add(8)) * cx.f(a2)) + (cx.f(a3.add(20)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(32)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    return a3;
}

/// PhysicsManager__1A7060 0x001A7060
pub fn f_1a7060(cx: &mut Cx, a1: Ptr, a3: Ptr, a4: Ptr, a5: i32, a6: Ptr) -> i64 {
    cx.push_frame(2036);
    let r = f_1a7060_body(cx, a1, a3, a4, a5, a6);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1a7060_body(cx: &mut Cx, a1: Ptr, a3: Ptr, a4: Ptr, a5: i32, a6: Ptr) -> i64 {
    let mut a2 = 0.0f64;
    let mut v6 = 0.0f64;
    let mut v7 = 0i64;
    let mut v8 = 0i32;
    let mut v9 = 0i32;
    let mut v10 = Ptr::NULL;
    let mut v11 = 0i32;
    let mut v12 = 0i32;
    let mut v13 = 0i32;
    let mut v14 = Ptr::NULL;
    let mut v15 = Ptr::NULL;
    let mut v16 = Ptr::NULL;
    let mut v17 = Ptr::NULL;
    let mut v18 = 0i32;
    let mut v19 = Ptr::NULL;
    let mut v20 = 0.0f64;
    let mut v21 = Ptr::NULL;
    let mut v22 = Ptr::NULL;
    let mut v23 = Ptr::NULL;
    let mut v24 = 0.0f64;
    let mut v25 = Ptr::NULL;
    let mut v26 = Ptr::NULL;
    let mut v28 = Ptr::NULL;
    let mut v29 = Ptr::NULL;
    let mut v30 = Ptr::NULL;
    let mut v31 = Ptr::NULL;
    let mut v32 = Ptr::NULL;
    let mut v33 = 0i32;
    let mut v34 = 0i32;
    let mut v35 = 0i32;
    let mut v36 = 0i32;
    let mut v37 = 0i32;
    let mut v38 = 0i64;
    let mut v39 = 0.0f32;
    let mut v40 = 0i32;
    let mut v41 = 0i32;
    let mut v42 = 0i32;
    let mut v43 = 0.0f32;
    let mut v44 = 0i32;
    let mut v45 = 0i32;
    let mut v46 = Ptr::NULL;
    let mut v47 = 0.0f32;
    let mut v48 = 0.0f32;
    { let t = a1; cx.set_lpp(508, t); }
    v7 = (v7 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.ptr_at(a1); let q1_ = a2; let q2_ = 0; ov_vehicle_slot2(cx, q0_, q1_, q2_) }.bits()) as i32 as u32 as i64);
    v7 = (v7 & 0xFFFF_FFFFi64) | (((cx.i(cx.ptr_at(Ptr::raw((v7 as i32))).add(3552))) as u32 as i64) << 32);
    { let t = ((v7 >> 32) as i32); cx.set_lpp(507, Ptr::raw(t)); }
    cx.memset(cx.lptr(2016), 0, 12);
    'lp17: while (cx.li(2016) < 2) {
        if (cx.li(2016) != 0) {
            if (cx.li(2016) == 1) {
                { let t = f_1b4640(cx); cx.set_lpp(505, Ptr::raw(t)); }
            }
        } else {
            { let t = ov_barrier_qt(cx, Ptr::glob(0x705F48)); cx.set_lpp(505, t); }
        }
        if (cx.lpp(505).bits() != 0) {
            { let t = 0; cx.set_lpp(503, Ptr::raw(t)); }
            if (cx.li(2016) != 0) {
                if (cx.li(2016) == 1) {
                    { let t = f_1b4650(cx); cx.set_lpp(503, Ptr::raw(t)); }
                }
            } else {
                { let t = ov_barrier_mesh(cx, Ptr::glob(0x705F48)); cx.set_lpp(503, t); }
            }
            { let t = 0; cx.set_li(2008, t); }
            'lp18: loop {
                'blp18: {
                    v7 = (v7 & 0xFFFF_FFFFi64) | (((a3.bits()) as u32 as i64) << 32);
                    if (cx.li(2008) >= cx.i(a3)) {
                        break 'lp18;
                    }
                    if (({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(508)); let q1_ = v6; let q2_ = cx.li(2008); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(28); cx.b(p) } != 0) && ((({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(508)); let q1_ = v6; let q2_ = cx.li(2008); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(5084); cx.i(p) } == 0) || ({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(508)); let q1_ = v6; let q2_ = cx.li(2008); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(5084); cx.i(p) } == 1)) || ({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(508)); let q1_ = v6; let q2_ = cx.li(2008); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(5084); cx.i(p) } == 4))) {
                        { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(508)); let q1_ = v6; let q2_ = cx.li(2008); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }; cx.set_lpp(501, t); }
                        { let t = cx.i(cx.lpp(501).add(3604)); cx.set_li(2000, t); }
                        { let t = 0; cx.set_li(964, t); }
                        if (cx.b(cx.lpp(501).add(4073)) != 0) {
                            { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(501).add(3556))); let q1_ = cx.i(cx.lpp(501).add(3556)); let q2_ = cx.lptr(912); f_37abb0(cx, q0_, q1_, q2_) };
                            { let t = 0; cx.set_li(904, t); }
                            'lp19: loop {
                                if !(cx.li(904) < 4) { break 'lp19; }
                                'blp19: {
                                    { let q0_ = cx.lpp(505); let q1_ = cx.lptr(0).add(4 * ((3_i32).wrapping_mul(cx.li(904)).wrapping_add(228))); let q2_ = cx.lptr(896); let q3_ = cx.lptr(884); let q4_ = cx.lptr(0).add(4 * ((3_i32).wrapping_mul(cx.li(904)).wrapping_add(228))); let q5_ = 0; let q6_ = Ptr::NULL; ov_qt_find_leaf(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
                                    { let t = 0; cx.set_li(876, t); }
                                    'lp20: loop {
                                        if !(cx.li(876) < cx.li(884)) { break 'lp20; }
                                        'blp20: {
                                            { let t = 0; cx.set_li(868, t); }
                                            { let t = 0; cx.set_li(872, t); }
                                            'lp21: loop {
                                                if !(cx.li(872) < cx.li(964)) { break 'lp21; }
                                                'blp21: {
                                                    v8 = { let q0_ = cx.lpp(505); let q1_ = cx.li(876).wrapping_add(cx.li(896)); ov_qt_face_index(cx, q0_, q1_) };
                                                    if (cx.i(cx.lptr(0).add(4 * (cx.li(872).wrapping_add(243)))) == v8) {
                                                        { let t = 1; cx.set_li(868, t); }
                                                        break 'lp21;
                                                    }
                                                }
                                                let _ = cx.preinc_i(cx.lptr(872));
                                            }
                                            if (cx.li(868) == 0) {
                                                v9 = { let q0_ = cx.lpp(505); let q1_ = cx.li(876).wrapping_add(cx.li(896)); ov_qt_face_index(cx, q0_, q1_) };
                                                { let p = cx.lptr(0).add(4 * (cx.li(964).wrapping_add(243))); let t = v9; cx.set_i(p, t); }
                                                if (cx.li(964) < 256) {
                                                    let _ = cx.preinc_i(cx.lptr(964));
                                                }
                                            }
                                        }
                                        let _ = cx.preinc_i(cx.lptr(876));
                                    }
                                }
                                let _ = cx.preinc_i(cx.lptr(904));
                            }
                        } else if ({ let p = { let q0_ = cx.lpp(501).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }.add(16); cx.i(p) } != 0) {
                            v39 = cx.f(cx.lpp(507));
                            v10 = { let q0_ = cx.lpp(501).add(240); ov_car_velocity(cx, q0_) };
                            { let q0_ = cx.lptr(852); let q1_ = v10; let q2_ = v39; f_19d970(cx, q0_, q1_, q2_) };
                            { let q0_ = cx.lptr(824); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(501).add(3556))); let q2_ = cx.lptr(852); f_1a38d0(cx, q0_, q1_, q2_) };
                            { let q0_ = cx.lptr(852); let q1_ = 2.0f32; f_19dab0(cx, q0_, q1_) };
                            v6 = ({ let q0_ = cx.lptr(852); f_19d9f0(cx, q0_) } as f64);
                            { let p = cx.lptr(844); let t = (v6 as f32); cx.set_f(p, t); }
                            { let t = cx.li(844); cx.set_li(840, t); }
                            { let t = 0; cx.set_li(816, t); }
                            { let t = 0; cx.set_li(812, t); }
                            'lp22: while (1 != 0) {
                                { let t = ({ let q0_ = cx.lpp(505); let q1_ = cx.lptr(792); let q2_ = cx.lptr(804); let q3_ = cx.lptr(792); let q4_ = cx.lptr(824); let q5_ = 0; let q6_ = Ptr::NULL; ov_qt_find_leaf(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) } as i32); cx.set_li(784, t); }
                                if (cx.li(784) != cx.li(812)) {
                                    { let t = 0; cx.set_li(780, t); }
                                    'lp23: loop {
                                        if !(cx.li(780) < cx.li(792)) { break 'lp23; }
                                        'blp23: {
                                            { let t = 0; cx.set_li(772, t); }
                                            { let t = 0; cx.set_li(776, t); }
                                            'lp24: loop {
                                                if !(cx.li(776) < cx.li(964)) { break 'lp24; }
                                                'blp24: {
                                                    v11 = { let q0_ = cx.lpp(505); let q1_ = cx.li(780).wrapping_add(cx.li(804)); ov_qt_face_index(cx, q0_, q1_) };
                                                    if (cx.i(cx.lptr(0).add(4 * (cx.li(776).wrapping_add(243)))) == v11) {
                                                        { let t = 1; cx.set_li(772, t); }
                                                        break 'lp24;
                                                    }
                                                }
                                                let _ = cx.preinc_i(cx.lptr(776));
                                            }
                                            if (cx.li(772) == 0) {
                                                v12 = { let q0_ = cx.lpp(505); let q1_ = cx.li(780).wrapping_add(cx.li(804)); ov_qt_face_index(cx, q0_, q1_) };
                                                { let p = cx.lptr(0).add(4 * (cx.li(964).wrapping_add(243))); let t = v12; cx.set_i(p, t); }
                                                if (cx.li(964) < 256) {
                                                    let _ = cx.preinc_i(cx.lptr(964));
                                                }
                                            }
                                        }
                                        let _ = cx.preinc_i(cx.lptr(780));
                                    }
                                }
                                { let t = cx.li(784); cx.set_li(812, t); }
                                { let p = cx.lptr(844); let t = (((cx.f(cx.lptr(844)) as f64) - (cx.f(cx.lpp(505).add(56)) as f64)) as f32); cx.set_f(p, t); }
                                let _ = cx.preinc_i(cx.lptr(816));
                                if (cx.f(cx.lptr(844)) <= 0.0f32) {
                                    break 'lp22;
                                }
                                { let q0_ = cx.lptr(824); let q1_ = cx.lptr(852); let q2_ = ((((0.0f32 - cx.f(cx.lpp(505).add(56))) as f64) / (cx.f(cx.lptr(840)) as f64)) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                            }
                        } else {
                            { let q0_ = cx.lpp(505); let q1_ = cx.lptr(764); let q2_ = cx.lptr(764); let q3_ = cx.lptr(752); let q4_ = cx.ptr_at(cx.ptr_at(cx.lpp(501).add(3556))); let q5_ = 0; let q6_ = Ptr::NULL; ov_qt_find_leaf(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
                            { let t = 0; cx.set_li(744, t); }
                            'lp25: loop {
                                if !(cx.li(744) < cx.li(752)) { break 'lp25; }
                                'blp25: {
                                    v13 = { let q0_ = cx.lpp(505); let q1_ = cx.li(744).wrapping_add(cx.li(764)); ov_qt_face_index(cx, q0_, q1_) };
                                    { let p = cx.lptr(0).add(4 * (cx.li(964).wrapping_add(243))); let t = v13; cx.set_i(p, t); }
                                    if (cx.li(964) < 256) {
                                        let _ = cx.preinc_i(cx.lptr(964));
                                    }
                                }
                                let _ = cx.preinc_i(cx.lptr(744));
                            }
                        }
                        if (cx.li(964) != 0) {
                            { let t = 0; cx.set_li(736, t); }
                            { let t = 0; cx.set_li(728, t); }
                            { let t = 0; cx.set_li(720, t); }
                            if (cx.i(Ptr::gc(0).add(2072)) != 0) {
                                { let t = 0; cx.set_li(716, t); }
                                'lp26: loop {
                                    if !(cx.li(716) < cx.i(cx.ptr_at(cx.lpp(501).add(3556)).add(4))) { break 'lp26; }
                                    'blp26: {
                                        { let t = 0; cx.set_li(728, t); }
                                        { let t = 0; cx.set_li(724, t); }
                                        'lp27: loop {
                                            if !(cx.li(724) < cx.li(964)) { break 'lp27; }
                                            'blp27: {
                                                { let t = { let q0_ = cx.lpp(503); let q1_ = cx.i(cx.lptr(0).add(4 * (cx.li(724).wrapping_add(243)))); let q2_ = 0; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(178, t); }
                                                { let t = { let q0_ = cx.lpp(503); let q1_ = cx.i(cx.lptr(0).add(4 * (cx.li(724).wrapping_add(243)))); let q2_ = 1; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(177, t); }
                                                { let t = { let q0_ = cx.lpp(503); let q1_ = cx.i(cx.lptr(0).add(4 * (cx.li(724).wrapping_add(243)))); let q2_ = 2; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(176, t); }
                                                { let q0_ = cx.lptr(688); let q1_ = cx.lpp(177); let q2_ = cx.lpp(178); f_19d920(cx, q0_, q1_, q2_) };
                                                { let q0_ = cx.lptr(668); let q1_ = cx.lpp(176); let q2_ = cx.lpp(178); f_19d920(cx, q0_, q1_, q2_) };
                                                { let q0_ = cx.lptr(648); let q1_ = cx.lptr(688); let q2_ = cx.lptr(668); f_1a03f0(cx, q0_, q1_, q2_) };
                                                v6 = ({ let q0_ = cx.lptr(648); f_19d9f0(cx, q0_) } as f64);
                                                { let p = cx.lptr(640); let t = (v6 as f32); cx.set_f(p, t); }
                                                if (cx.f(cx.lptr(640)) == 0.0f32) {
                                                    v40 = { let p = { let q0_ = cx.lptr(176); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                                    v33 = cx.i(Ptr::glob(0x77FD48));
                                                    v28 = { let q0_ = cx.lptr(160); let q1_ = cx.i(cx.lpp(177)); let q2_ = cx.i(cx.lpp(177).add(4)); let q3_ = cx.i(cx.lpp(177).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v14 = { let q0_ = cx.lptr(144); let q1_ = cx.i(cx.lpp(178)); let q2_ = cx.i(cx.lpp(178).add(4)); let q3_ = cx.i(cx.lpp(178).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v14, v28, v33, v40);
                                                    v41 = { let p = { let q0_ = cx.lptr(140); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                                    v34 = cx.i(Ptr::glob(0x77FD48));
                                                    v29 = { let q0_ = cx.lptr(124); let q1_ = cx.i(cx.lpp(176)); let q2_ = cx.i(cx.lpp(176).add(4)); let q3_ = cx.i(cx.lpp(176).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v15 = { let q0_ = cx.lptr(108); let q1_ = cx.i(cx.lpp(178)); let q2_ = cx.i(cx.lpp(178).add(4)); let q3_ = cx.i(cx.lpp(178).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v15, v29, v34, v41);
                                                    v42 = { let p = { let q0_ = cx.lptr(104); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                                    v35 = cx.i(Ptr::glob(0x77FD48));
                                                    v30 = { let q0_ = cx.lptr(88); let q1_ = cx.i(cx.lpp(176)); let q2_ = cx.i(cx.lpp(176).add(4)); let q3_ = cx.i(cx.lpp(176).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v16 = { let q0_ = cx.lptr(72); let q1_ = cx.i(cx.lpp(177)); let q2_ = cx.i(cx.lpp(177).add(4)); let q3_ = cx.i(cx.lpp(177).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v16, v30, v35, v42);
                                                }
                                                { let t = cx.i(cx.lpp(503).add(28)).wrapping_add((12_i32).wrapping_mul(cx.i(cx.lptr(0).add(4 * (cx.li(724).wrapping_add(243)))))); cx.set_lpp(159, Ptr::raw(t)); }
                                                if (((cx.i(Ptr::gc(0).add(1628)) != 0) && (cx.li(2000) == 4)) && (cx.f(cx.lpp(159).add(4)) > cx.f(Ptr::gc(0).add(1712)))) {
                                                    { let t = 0; cx.set_li(632, t); }
                                                } else {
                                                    { let t = ({ let q0_ = cx.lpp(178); let q1_ = cx.lpp(177); let q2_ = cx.lpp(176); let q3_ = cx.ptr_at(cx.ptr_at(cx.lpp(501).add(3556))).add((132_i32).wrapping_mul(cx.li(716))); f_3813b0(cx, q0_, q1_, q2_, q3_) } as i32); cx.set_li(632, t); }
                                                }
                                                if (cx.li(632) != 0) {
                                                    { let q0_ = cx.lptr(616); let q1_ = cx.lpp(178); let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(501).add(3556))).add((132_i32).wrapping_mul(cx.li(716))); f_19d920(cx, q0_, q1_, q2_) };
                                                    v6 = { let q0_ = cx.lpp(159); let q1_ = cx.lptr(616); f_19d9c0(cx, q0_, q1_) };
                                                    { let p = cx.lptr(608); let t = (v6 as f32); cx.set_f(p, t); }
                                                    let _ = cx.preinc_i(cx.lptr(728));
                                                    { let p = Ptr::glob(0x75AA38).add(4 * (cx.li(724))); let t = 1; cx.set_i(p, t); }
                                                } else {
                                                    { let p = Ptr::glob(0x75AA38).add(4 * (cx.li(724))); let t = 0; cx.set_i(p, t); }
                                                }
                                            }
                                            let _ = cx.preinc_i(cx.lptr(724));
                                        }
                                        if (cx.li(728) != 0) {
                                            { let t = 0; cx.set_li(720, t); }
                                            { let t = 0; cx.set_li(604, t); }
                                            { let t = 0; cx.set_li(724, t); }
                                            'lp28: loop {
                                                if !(cx.li(724) < cx.li(964)) { break 'lp28; }
                                                'blp28: {
                                                    if (cx.i(Ptr::glob(0x75AA38).add(4 * (cx.li(724)))) == 1) {
                                                        { let q0_ = Ptr::glob(0x757078).add(4 * ((3_i32).wrapping_mul(cx.li(604)))); let q1_ = cx.ptr_at(cx.lpp(503).add(28)).add((12_i32).wrapping_mul(cx.i(cx.lptr(0).add(4 * (cx.li(724).wrapping_add(243)))))); f_19e030(cx, q0_, q1_) };
                                                        { let t = cx.i(cx.lptr(0).add(4 * (cx.li(724).wrapping_add(243)))); cx.set_li(600, t); }
                                                        { let t = 0; cx.set_li(592, t); }
                                                        'lp29: loop {
                                                            if !(cx.li(592) < 3) { break 'lp29; }
                                                            'blp29: {
                                                                { let t = 0; cx.set_li(588, t); }
                                                                { let t = { let q0_ = cx.lpp(503); let q1_ = cx.i(cx.lptr(0).add(4 * (cx.li(724).wrapping_add(243)))); let q2_ = cx.li(592); f_1b45f0(cx, q0_, q1_, q2_) }; cx.set_li(584, t); }
                                                                { let t = 0; cx.set_li(596, t); }
                                                                'lp30: loop {
                                                                    if !(cx.li(596) < cx.li(720)) { break 'lp30; }
                                                                    'blp30: {
                                                                        if (cx.i(Ptr::glob(0x74D478).add(4 * (cx.li(596)))) == cx.li(584)) {
                                                                            { let p = Ptr::glob(0x758938).add(4 * ((4_i32).wrapping_mul(cx.li(604)).wrapping_add(cx.li(592)))); let t = cx.li(596); cx.set_i(p, t); }
                                                                            { let t = 1; cx.set_li(588, t); }
                                                                            break 'lp30;
                                                                        }
                                                                    }
                                                                    let _ = cx.preinc_i(cx.lptr(596));
                                                                }
                                                                if (cx.li(588) == 0) {
                                                                    v17 = { let q0_ = cx.lpp(503); let q1_ = cx.i(cx.lptr(0).add(4 * (cx.li(724).wrapping_add(243)))); let q2_ = cx.li(592); ov_mesh_vertex(cx, q0_, q1_, q2_) };
                                                                    { let q0_ = Ptr::glob(0x752638).add(4 * ((3_i32).wrapping_mul(cx.li(720)))); let q1_ = v17; f_19e030(cx, q0_, q1_) };
                                                                    { let p = Ptr::glob(0x758938).add(4 * ((4_i32).wrapping_mul(cx.li(604)).wrapping_add(cx.li(592)))); let t = cx.li(720); cx.set_i(p, t); }
                                                                    v18 = { let q0_ = cx.lpp(503); let q1_ = cx.i(cx.lptr(0).add(4 * (cx.li(724).wrapping_add(243)))); let q2_ = cx.li(592); f_1b45f0(cx, q0_, q1_, q2_) };
                                                                    { let p = Ptr::glob(0x74D478).add(4 * (cx.postinc_i(cx.lptr(720)))); let t = v18; cx.set_i(p, t); }
                                                                }
                                                            }
                                                            let _ = cx.preinc_i(cx.lptr(592));
                                                        }
                                                        if (cx.i(cx.lpp(503).add(12)) != 0) {
                                                            { let p = Ptr::glob(0x758944).add(4 * ((4_i32).wrapping_mul(cx.li(604)))); let t = (cx.u16(cx.ptr_at(cx.lpp(503).add(24)).add((8_i32).wrapping_mul(cx.li(600))).add(6)) as u8 as i32); cx.set_i(p, t); }
                                                        } else {
                                                            { let p = Ptr::glob(0x758944).add(4 * ((4_i32).wrapping_mul(cx.li(604)))); let t = (cx.i(cx.ptr_at(cx.lpp(503).add(20)).add((16_i32).wrapping_mul(cx.li(600))).add(12)) & -65281); cx.set_i(p, t); }
                                                        }
                                                        let _ = cx.preinc_i(cx.lptr(604));
                                                    }
                                                }
                                                let _ = cx.preinc_i(cx.lptr(724));
                                            }
                                        }
                                        if (cx.li(728) != 0) {
                                            { let t = a5.wrapping_sub(cx.li(2024)); cx.set_li(736, t); }
                                            if (({ let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(2024))); let q1_ = cx.lptr(736); let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(501).add(3556))).add((132_i32).wrapping_mul(cx.li(716))); let q3_ = Ptr::glob(0x752638); let q4_ = cx.li(720); let q5_ = Ptr::glob(0x758938); let q6_ = Ptr::glob(0x757078); let q7_ = cx.li(728); let q8_ = 1036831949.0f32; let q9_ = 1034147594.0f32; let q10_ = 0; f_394ea0(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_, q10_) } as i32) != 0) {
                                                { let t = 0; cx.set_li(576, t); }
                                                { let t = 0; cx.set_li(580, t); }
                                                'lp31: loop {
                                                    if !(cx.li(580) < cx.li(736)) { break 'lp31; }
                                                    'blp31: {
                                                        { let t = a4.add((1560_i32).wrapping_mul(cx.li(576).wrapping_add(cx.li(2024)))); cx.set_lpp(143, t); }
                                                        if (((cx.i(Ptr::gc(0).add(1628)) == 0) || (cx.li(2000) != 4)) || (cx.f(a4.add((1560_i32).wrapping_mul(cx.li(580).wrapping_add(cx.li(2024)))).add(1336)) <= cx.f(Ptr::gc(0).add(1712)))) {
                                                            { let q0_ = cx.lpp(143).add(1332); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(580).wrapping_add(cx.li(2024)))).add(1332); f_19e030(cx, q0_, q1_) };
                                                            { let q0_ = cx.lpp(143).add(1276); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(580).wrapping_add(cx.li(2024)))).add(1276); f_19e030(cx, q0_, q1_) };
                                                            { let p = cx.lpp(143).add(1288); let t = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(580).wrapping_add(cx.li(2024)))).add(1288)); cx.set_i(p, t); }
                                                            { let p = cx.lpp(143).add(4); let t = cx.lpp(501).add(240); cx.set_ptr(p, t); }
                                                            { let p = cx.lpp(501).add(4856); let t = 0; cx.set_i(p, t); }
                                                            { let p = cx.lpp(143).add(8); let t = 0; cx.set_i(p, t); }
                                                            if ((cx.i(cx.lpp(501).add(3740)) != 0) && (cx.i(cx.lpp(501).add(3604)) >= 3)) {
                                                                { let p = cx.lpp(143).add(1540); let t = cx.i(Ptr::gc(0).add(1596)); cx.set_i(p, t); }
                                                            } else {
                                                                { let p = cx.lpp(143).add(1540); let t = cx.i(Ptr::gc(0).add(1592)); cx.set_i(p, t); }
                                                            }
                                                            { let p = cx.lpp(143).add(1544); let t = cx.i(Ptr::gc(0).add(1600)); cx.set_i(p, t); }
                                                            v6 = vsmath::fabs(cx.f(cx.lpp(143).add(1336)));
                                                            { let p = cx.lptr(568); let t = (v6 as f32); cx.set_f(p, t); }
                                                            if (cx.f(cx.lpp(507).add(232)) == 0.0f32) {
                                                                { let p = cx.lpp(143).add(1272); let t = ((((cx.f(Ptr::gc(0).add(1612)) * cx.f(cx.lptr(568))) as f64) + (((1.0f32 - cx.f(cx.lptr(568))) * cx.f(Ptr::gc(0).add(1616))) as f64)) as f32); cx.set_f(p, t); }
                                                            } else {
                                                                v19 = { let q0_ = cx.lpp(501).add(240); ov_car_velocity(cx, q0_) };
                                                                v43 = ({ let q0_ = cx.lpp(143).add(1332); let q1_ = v19; f_19d9c0(cx, q0_, q1_) } as f32);
                                                                v20 = vsmath::fabs(v43);
                                                                v6 = ((1.0f64 / ((((cx.f(cx.lpp(507).add(232)) as f64) / v20) / (cx.f(cx.lpp(507)) as f64)) + 0.5f64)) + -1.0f64);
                                                                { let p = cx.lpp(143).add(1272); let t = (v6 as f32); cx.set_f(p, t); }
                                                                if (cx.f(cx.lpp(143).add(1272)) < -0.99f32) {
                                                                    { let p = cx.lpp(143).add(1272); let t = -1082298204; cx.set_i(p, t); }
                                                                }
                                                            }
                                                            { let p = cx.lpp(143).add(1268); let t = ((((cx.f(Ptr::gc(0).add(1604)) * cx.f(cx.lptr(568))) as f64) + (((1.0f32 - cx.f(cx.lptr(568))) * cx.f(Ptr::gc(0).add(1608))) as f64)) as f32); cx.set_f(p, t); }
                                                            if (cx.f(cx.lptr(568)) < 0.1f32) {
                                                                { let p = cx.lpp(143).add(1280); let t = cx.i(cx.lpp(501).add(308)); cx.set_i(p, t); }
                                                            }
                                                            if (cx.f(Ptr::gc(0).add(1992)) != 0.0f32) {
                                                                { let q0_ = cx.lptr(552); let q1_ = cx.lpp(143).add(1276); let q2_ = cx.lpp(143).add(1332); let q3_ = 1.0f32; f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                                                v44 = { let p = { let q0_ = cx.lptr(68); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                                                v36 = cx.i(Ptr::glob(0x77FD8C));
                                                                v31 = { let q0_ = cx.lptr(52); let q1_ = cx.li(552); let q2_ = cx.li(556); let q3_ = cx.li(560); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                v21 = { let q0_ = cx.lptr(36); let q1_ = cx.i(cx.lpp(143).add(1276)); let q2_ = cx.i(cx.lpp(143).add(1280)); let q3_ = cx.i(cx.lpp(143).add(1284)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v21, v31, v36, v44);
                                                                { let q0_ = cx.lptr(552); let q1_ = cx.lpp(143).add(1276); let q2_ = cx.lpp(143).add(1332); let q3_ = (((cx.f(Ptr::gc(0).add(1992)) as f64) * (cx.f(cx.lpp(143).add(1288)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                                                { let q0_ = cx.lptr(532); let q1_ = cx.lpp(143).add(1276); f_19e030(cx, q0_, q1_) };
                                                                { let p = cx.lptr(536); let t = (((cx.f(cx.lptr(536)) as f64) + 0.05f64) as f32); cx.set_f(p, t); }
                                                                v45 = { let p = { let q0_ = cx.lptr(32); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                                                v37 = cx.i(Ptr::glob(0x77FD48));
                                                                v32 = { let q0_ = cx.lptr(16); let q1_ = cx.li(552); let q2_ = cx.li(556); let q3_ = cx.li(560); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                v22 = { let q0_ = cx.lptr(0); let q1_ = cx.li(532); let q2_ = cx.li(536); let q3_ = cx.li(540); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v22, v32, v37, v45);
                                                                v46 = cx.lpp(143).add(1332);
                                                                v23 = { let q0_ = cx.lpp(501).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                                                                v47 = (f_19d9c0(cx, v23, v46) as f32);
                                                                v24 = vsmath::fabs(v47);
                                                                v6 = (cx.f(Ptr::glob(0x68898C)) as f64);
                                                                if (v24 > (cx.f(Ptr::glob(0x68898C)) as f64)) {
                                                                    { let t = 1; cx.set_li(524, t); }
                                                                }
                                                            }
                                                            let _ = cx.preinc_i(cx.lptr(576));
                                                        }
                                                    }
                                                    let _ = cx.preinc_i(cx.lptr(580));
                                                }
                                                { let t = cx.li(2024).wrapping_add(cx.li(576)); cx.set_li(2024, t); }
                                            }
                                        }
                                    }
                                    let _ = cx.preinc_i(cx.lptr(716));
                                }
                            }
                            if ((cx.li(2024) != 0) && (cx.i(Ptr::gc(0).add(2076)) != 0)) {
                                { let q0_ = cx.lptr(344); f_1b42f0(cx, q0_) };
                                { let t = cx.i(cx.ptr_at(cx.lpp(501).add(3552))); cx.set_li(344, t); }
                                { let t = cx.i(Ptr::gc(0).add(1152)); cx.set_li(420, t); }
                                if (cx.f(Ptr::gc(0).add(1620)) == 0.0f32) {
                                    { let t = cx.i(Ptr::gc(0).add(1156)); cx.set_li(352, t); }
                                } else {
                                    { let t = cx.i(Ptr::gc(0).add(1620)); cx.set_li(352, t); }
                                }
                                { let t = cx.i(Ptr::gc(0).add(1160)); cx.set_li(348, t); }
                                { let t = cx.i(Ptr::gc(0).add(1164)); cx.set_li(424, t); }
                                { let t = cx.i(Ptr::gc(0).add(1168)); cx.set_li(468, t); }
                                { let t = cx.i(Ptr::gc(0).add(1172)); cx.set_li(432, t); }
                                { let t = cx.i(Ptr::gc(0).add(1176)); cx.set_li(436, t); }
                                { let t = cx.i(Ptr::gc(0).add(1180)); cx.set_li(428, t); }
                                { let q0_ = cx.lptr(380); let q1_ = a6; f_19e030(cx, q0_, q1_) };
                                { let q0_ = cx.lptr(356); let q1_ = Ptr::gc(0); f_19e030(cx, q0_, q1_) };
                                { let t = 0; let o = cx.li(316); cx.set_li(316, (o & !(255 << 24)) | ((t & 255) << 24)); }
                                if (((cx.f(Ptr::gc(0).add(1420)) > 0.0f32) && (cx.i(cx.lpp(501).add(4408)) == 0)) && (cx.f(cx.lpp(501).add(3784)) > cx.f(Ptr::gc(0).add(1408)))) {
                                    { let q0_ = cx.lpp(501).add(240); let q1_ = cx.lptr(324); f_1b42b0(cx, q0_, q1_) };
                                    { let t = 1; let o = cx.li(316); cx.set_li(316, (o & !(255 << 24)) | ((t & 255) << 24)); }
                                }
                                { let q0_ = a4; let q1_ = cx.li(2024); let q2_ = Ptr::NULL; let q3_ = 0; let q4_ = Ptr::NULL; let q5_ = 0; let q6_ = cx.lptr(344); f_389630(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
                                if (((cx.li(316) >> 24) & 255) != 0) {
                                    v25 = { let q0_ = cx.lpp(501).add(240); ov_car_velocity(cx, q0_) };
                                    { let q0_ = cx.lptr(300); let q1_ = cx.lptr(324); let q2_ = v25; f_19d920(cx, q0_, q1_, q2_) };
                                    v6 = ({ let q0_ = cx.lptr(300); f_19d9f0(cx, q0_) } as f64);
                                    { let p = cx.lpp(501).add(4428); let t = (v6 as f32); cx.set_f(p, t); }
                                    if (cx.f(cx.lpp(501).add(4428)) > cx.f(Ptr::gc(0).add(1412))) {
                                        { let p = cx.lpp(501).add(4424); let t = 0; cx.set_i(p, t); }
                                        { let q0_ = cx.lpp(501).add(4412); let q1_ = cx.lptr(300); let q2_ = ((1.0f64 / (cx.f(cx.lpp(501).add(4428)) as f64)) as f32); f_19d970(cx, q0_, q1_, q2_) };
                                        v26 = { let q0_ = cx.lpp(501).add(240); let q1_ = 2; ov_car_rot_row(cx, q0_, q1_) };
                                        v48 = (f_19d9c0(cx, v26, a4.add(1332)) as f32);
                                        v6 = (1.0f64 - vsmath::fabs(v48));
                                        { let p = cx.lptr(292); let t = (v6 as f32); cx.set_f(p, t); }
                                        { let p = cx.lptr(288); let t = (((((1.0f32 - cx.f(Ptr::gc(0).add(1440))) * cx.f(cx.lptr(292))) as f64) + (cx.f(Ptr::gc(0).add(1440)) as f64)) as f32); cx.set_f(p, t); }
                                        { let q0_ = cx.lpp(501).add(4412); let q1_ = (((cx.f(cx.lptr(288)) as f64) * (cx.f(Ptr::gc(0).add(1436)) as f64)) as f32); f_19dab0(cx, q0_, q1_) };
                                        { let p = cx.lpp(501).add(4432); let t = (((((1.0f32 - cx.f(Ptr::gc(0).add(1444))) * cx.f(cx.lptr(292))) as f64) + (cx.f(Ptr::gc(0).add(1444)) as f64)) as f32); cx.set_f(p, t); }
                                        { let p = cx.lpp(501).add(4408); let t = cx.i(Ptr::gc(0).add(1428)); cx.set_i(p, t); }
                                    }
                                }
                                if (cx.i(Ptr::gc(0).add(1624)) != 0) {
                                    { let t = 0; cx.set_li(280, t); }
                                    { let p = cx.lptr(276); let t = ((0.0f64 - 3.4028235e38f64) as f32); cx.set_f(p, t); }
                                    { let t = 0; cx.set_li(284, t); }
                                    'lp32: loop {
                                        if !(cx.li(284) < cx.li(2024)) { break 'lp32; }
                                        'blp32: {
                                            v6 = vsmath::fabs(cx.f(a4.add((1560_i32).wrapping_mul(cx.li(284))).add(236)));
                                            { let p = cx.lptr(272); let t = (v6 as f32); cx.set_f(p, t); }
                                            if (cx.f(cx.lptr(272)) > cx.f(cx.lptr(276))) {
                                                { let t = cx.li(272); cx.set_li(276, t); }
                                                { let t = cx.li(284); cx.set_li(280, t); }
                                            }
                                        }
                                        let _ = cx.preinc_i(cx.lptr(284));
                                    }
                                    if (cx.f(cx.lptr(276)) > 0.0f32) {
                                        { let q0_ = cx.lptr(188); f_238b0(cx, q0_) };
                                        { let t = cx.li(2008); cx.set_li(188, t); }
                                        { let t = -2; cx.set_li(192, t); }
                                        { let t = cx.li(276); cx.set_li(228, t); }
                                        { let q0_ = cx.lptr(196); let q1_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(280))).add(1276)); let q2_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(280))).add(1280)); let q3_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(280))).add(1284)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                        { let q0_ = cx.lptr(212); let q1_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(280))).add(1332)); let q2_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(280))).add(1336)); let q3_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(280))).add(1340)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                        { let t = 0; cx.set_li(232, t); }
                                        { let t = 1090519040; cx.set_li(260, t); }
                                        { let t = cx.lpp(501).add(240); cx.set_lpp(59, t); }
                                        { let t = 0; cx.set_li(240, t); }
                                        { let q0_ = cx.lpp(508); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(280))); let q2_ = cx.lptr(244); f_1b39b0(cx, q0_, q1_, q2_) };
                                        { let t = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(280))).add(1556)); cx.set_li(180, t); }
                                        { let q0_ = cx.b(Ptr::glob(0x6FDC88)); let q1_ = cx.lptr(188); let q2_ = cx.li(180); let q3_ = cx.lptr(188); let q4_ = cx.li(2008); crate::ov_ignore!(cx, q0_, q1_, q2_, q3_, q4_) };
                                        { let t = cx.li(180); let o = cx.li(264); cx.set_li(264, (o & !(255 << 0)) | ((t & 255) << 0)); }
                                        { let q0_ = Ptr::glob(0x6FAA34); let q1_ = 0; let q2_ = cx.lptr(188); ov_message(cx, q0_, q1_, q2_) };
                                    }
                                }
                            }
                            { let t = 0; cx.set_li(2024, t); }
                        }
                    }
                }
                let _ = cx.preinc_i(cx.lptr(2008));
            }
        }
        v7 = (v7 & !0xFFFF_FFFFi64) | ((cx.preinc_i(cx.lptr(2016))) as i32 as u32 as i64);
    }
    v38 = v7;
    return v38;
}

/// PhysicsManager__1A89F0 0x001A89F0
pub fn f_1a89f0(cx: &mut Cx, this: Ptr, a2: Ptr, a3: Ptr, a4: i32, a5: Ptr) -> i32 {
    cx.push_frame(16924);
    let r = f_1a89f0_body(cx, this, a2, a3, a4, a5);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1a89f0_body(cx: &mut Cx, this: Ptr, a2: Ptr, a3: Ptr, a4: i32, a5: Ptr) -> i32 {
    let mut v5 = 0.0f64;
    let mut v6 = 0i32;
    let mut v7 = Ptr::NULL;
    let mut v8 = Ptr::NULL;
    let mut v9 = 0i32;
    let mut v10 = Ptr::NULL;
    let mut v11 = Ptr::NULL;
    let mut v12 = Ptr::NULL;
    let mut v14 = 0i32;
    let mut v16 = Ptr::NULL;
    let mut v17 = Ptr::NULL;
    let mut v18 = Ptr::NULL;
    let mut v19 = Ptr::NULL;
    let mut v20 = 0i32;
    let mut v21 = 0.0f64;
    let mut v22 = 0.0f32;
    let mut v23 = Ptr::NULL;
    let mut v24 = Ptr::NULL;
    let mut v25 = Ptr::NULL;
    let mut v26 = Ptr::NULL;
    let mut v27 = Ptr::NULL;
    let mut v28 = 0.0f64;
    let mut v29 = 0.0f64;
    let mut v30 = 0i32;
    let mut v32 = 0.0f32;
    let mut v33 = Ptr::NULL;
    let mut v34 = Ptr::NULL;
    let mut v35 = Ptr::NULL;
    let mut v36 = Ptr::NULL;
    let mut v37 = 0i64;
    let mut v38 = 0i32;
    let mut v39 = 0i32;
    let mut v40 = 0i32;
    let mut v41 = 0i32;
    let mut v42 = 0i64;
    let mut v43 = 0i32;
    let mut v44 = 0i32;
    let mut v45 = 0.0f32;
    let mut v46 = Ptr::NULL;
    let mut v47 = 0i32;
    let mut v48 = 0i32;
    let mut v49 = 0.0f32;
    let mut v50 = 0.0f32;
    let mut v51 = 0.0f32;
    let mut Vertex = Ptr::NULL;
    let mut car_static_object_inertia_scale = 0.0f32;
    crate::ov_ignore!(cx, 16924);
    v6 = -858993460;
    { let t = this; cx.set_lpp(4230, t); }
    { let t = 0; cx.set_li(16916, t); }
    { let t = 0; cx.set_li(16912, t); }
    'lp13: loop {
        if !(cx.li(16912) < cx.i(a2)) { break 'lp13; }
        'blp13: {
            if ((({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(4230)); let q1_ = v5; let q2_ = cx.li(16912); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(28); cx.b(p) } != 0) && ((({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(4230)); let q1_ = v5; let q2_ = cx.li(16912); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(5084); cx.i(p) } == 0) || ({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(4230)); let q1_ = v5; let q2_ = cx.li(16912); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(5084); cx.i(p) } == 1)) || ({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(4230)); let q1_ = v5; let q2_ = cx.li(16912); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(5084); cx.i(p) } == 4))) && ({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(4230)); let q1_ = v5; let q2_ = cx.li(16912); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(4388); cx.i(p) } == 0)) {
                { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(4230)); let q1_ = v5; let q2_ = cx.li(16912); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }; cx.set_lpp(4227, t); }
                { let t = cx.i(cx.lpp(4227).add(3604)); cx.set_li(16904, t); }
                if ({ let p = ov_physics_manager(cx).add(2840); cx.i(p) } != 0) {
                    if ({ let p = { let p = ov_physics_manager(cx).add(2840); cx.ptr_at(p) }.add(40); cx.b(p) } == 0) {
                        v7 = ov_physics_manager(cx);
                        { let q0_ = cx.ptr_at(v7.add(2840)); let q1_ = 0.0f32; f_385b80(cx, q0_, q1_) };
                    }
                    v37 = cx.q(cx.ptr_at(cx.ptr_at(cx.lpp(4227).add(3556))).add(8));
                    v32 = cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(4227).add(3556))));
                    v8 = ov_physics_manager(cx);
                    { let t = ({ let q0_ = cx.ptr_at(v8.add(2840)); let q1_ = v9; let q2_ = cx.lptr(16896); let q3_ = v32; let q4_ = f32::from_bits(v37 as u32); let q5_ = ((v37 >> 32) & 4294967295) as i32; f_385920(cx, q0_, q1_, q2_, q3_, q4_, q5_) } as i32); cx.set_li(16888, t); }
                    if (cx.li(16896) == 0) {
                        let _ = ov_physics_manager(cx);
                        crate::ov_ignore!(cx, 1, "!!!CAxisSortedList__GetCollisionCandidates(): The number of returned candidates could not fit into CAxisSortedList's preallocated array of %d, change the constructor parameters\n");
                    }
                    { let t = 0; cx.set_li(16884, t); }
                    // LABEL_15:
                    'b_LABEL_15: loop {
                        if (cx.li(16884) >= cx.li(16888)) {
                            break 'blp13;
                        }
                        { let t = cx.ptr_at(Ptr::raw(cx.li(16896).wrapping_add((4_i32).wrapping_mul(cx.li(16884))))); cx.set_lpp(4220, t); }
                        if ((((cx.lpp(4220).bits() != 0) && (cx.i(cx.lpp(4220).add(104)) != 0)) && ((cx.i(cx.ptr_at(cx.lpp(4220).add(104)).add(20)) != 0) || (cx.i(cx.ptr_at(cx.lpp(4220).add(104)).add(16)) != 0))) && (cx.i(cx.lpp(4220).add(104)) != 0)) {
                            { let t = cx.ptr_at(cx.lpp(4220).add(104)); cx.set_lpp(4219, t); }
                            { let t = { let q0_ = cx.lpp(4220); crate::ov_ident!(cx, q0_) }; cx.set_lpp(4218, t); }
                            v10 = { let q0_ = cx.lpp(4218); let q1_ = 3; ov_mat_row(cx, q0_, q1_) };
                            { let p = cx.lptr(16856); let t = ov_vec4_x(cx, v10); cx.set_f(p, t); }
                            v11 = { let q0_ = cx.lpp(4218); let q1_ = 3; ov_mat_row(cx, q0_, q1_) };
                            { let p = cx.lptr(16860); let t = ov_vec4_y(cx, v11); cx.set_f(p, t); }
                            v12 = { let q0_ = cx.lpp(4218); let q1_ = 3; ov_mat_row(cx, q0_, q1_) };
                            v5 = (ov_vec4_z(cx, v12) as f64);
                            { let p = cx.lptr(16864); let t = (v5 as f32); cx.set_f(p, t); }
                            { let p = cx.lptr(16836); let t = (((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(4227).add(3556)))) as f64) - (cx.f(cx.lptr(16856)) as f64)) as f32); cx.set_f(p, t); }
                            { let p = cx.lptr(16844); let t = (((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(4227).add(3556))).add(8)) as f64) - (cx.f(cx.lptr(16864)) as f64)) as f32); cx.set_f(p, t); }
                            { let p = cx.lptr(16840); let t = (((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(4227).add(3556))).add(4)) as f64) - (cx.f(cx.lptr(16860)) as f64)) as f32); cx.set_f(p, t); }
                            { let q0_ = cx.lptr(16764); f_1b4660(cx, q0_) };
                            { let q0_ = cx.lptr(16764); let q1_ = cx.lptr(16836); let q2_ = cx.lpp(4218); f_1aad50(cx, q0_, q1_, q2_) };
                            { let q0_ = cx.lptr(16780); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(4227).add(3556))).add(16); let q2_ = cx.lpp(4218); f_1aae70(cx, q0_, q1_, q2_) };
                            { let q0_ = cx.lptr(16816); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(4227).add(3556))).add(52); f_19e030(cx, q0_, q1_) };
                            { let t = cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(4227).add(3556))).add(12)); cx.set_li(16776, t); }
                            if (cx.i(cx.lpp(4219).add(20)) != 0) {
                                { let t = cx.ptr_at(cx.lpp(4219).add(20)); cx.set_lpp(4189, t); }
                                { let t = 0; cx.set_li(16752, t); }
                                { let t = 0; cx.set_li(16748, t); }
                                'lp14: loop {
                                    if !(cx.li(16748) < cx.i(cx.lpp(4189).add(4))) { break 'lp14; }
                                    'blp14: {
                                        { let t = { let q0_ = cx.lpp(4189); let q1_ = cx.li(16748); let q2_ = 0; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(4186, t); }
                                        { let t = { let q0_ = cx.lpp(4189); let q1_ = cx.li(16748); let q2_ = 1; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(4185, t); }
                                        { let t = { let q0_ = cx.lpp(4189); let q1_ = cx.li(16748); let q2_ = 2; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(4184, t); }
                                        { let t = cx.i(cx.lpp(4189).add(28)).wrapping_add((12_i32).wrapping_mul(cx.li(16748))); cx.set_lpp(4183, Ptr::raw(t)); }
                                        if (((cx.gc.static_object_filter_out_vertical_contacts_when_driving_on_all_wheels != 0) && (cx.li(16904) == 4)) && (cx.f(cx.lpp(4183).add(4)) > cx.gc.min_normal_up_to_filter_out)) {
                                            { let t = 0; cx.set_li(16728, t); }
                                        } else {
                                            { let t = ({ let q0_ = cx.lpp(4186); let q1_ = cx.lpp(4185); let q2_ = cx.lpp(4184); let q3_ = cx.lptr(16764); f_3813b0(cx, q0_, q1_, q2_, q3_) } as i32); cx.set_li(16728, t); }
                                        }
                                        if (cx.li(16728) != 0) {
                                            { let q0_ = cx.lptr(16712); let q1_ = cx.lpp(4186); let q2_ = cx.lptr(16764); f_19d920(cx, q0_, q1_, q2_) };
                                            { let p = cx.lptr(16704); let t = ({ let q0_ = cx.lpp(4183); let q1_ = cx.lptr(16712); f_19d9c0(cx, q0_, q1_) } as f32); cx.set_f(p, t); }
                                            if (cx.f(cx.lptr(16704)) >= 0.0f32) {
                                                { let p = Ptr::glob(0x75AA38).add(4 * (cx.li(16748))); let t = 0; cx.set_i(p, t); }
                                            } else {
                                                let _ = cx.preinc_i(cx.lptr(16752));
                                                { let p = Ptr::glob(0x75AA38).add(4 * (cx.li(16748))); let t = 1; cx.set_i(p, t); }
                                            }
                                        } else {
                                            { let p = Ptr::glob(0x75AA38).add(4 * (cx.li(16748))); let t = 0; cx.set_i(p, t); }
                                        }
                                    }
                                    let _ = cx.preinc_i(cx.lptr(16748));
                                }
                                { let t = 0; cx.set_li(16700, t); }
                                if (cx.li(16752) != 0) {
                                    { let t = 0; cx.set_li(16696, t); }
                                    { let t = 0; cx.set_li(16692, t); }
                                    'lp15: loop {
                                        if !(cx.li(16692) < cx.i(cx.lpp(4189).add(4))) { break 'lp15; }
                                        'blp15: {
                                            if (cx.i(Ptr::glob(0x75AA38).add(4 * (cx.li(16692)))) == 1) {
                                                { let q0_ = Ptr::glob(0x757078).add(4 * ((3_i32).wrapping_mul(cx.li(16696)))); let q1_ = cx.ptr_at(cx.lpp(4189).add(28)).add((12_i32).wrapping_mul(cx.li(16692))); f_19e030(cx, q0_, q1_) };
                                                { let t = 0; cx.set_li(16684, t); }
                                                'lp16: loop {
                                                    if !(cx.li(16684) < 3) { break 'lp16; }
                                                    'blp16: {
                                                        { let t = 0; cx.set_li(16680, t); }
                                                        { let t = { let q0_ = cx.lpp(4189); let q1_ = cx.li(16692); let q2_ = cx.li(16684); f_1b45f0(cx, q0_, q1_, q2_) }; cx.set_li(16676, t); }
                                                        { let t = 0; cx.set_li(16688, t); }
                                                        'lp17: loop {
                                                            if !(cx.li(16688) < cx.li(16700)) { break 'lp17; }
                                                            'blp17: {
                                                                if (cx.i(Ptr::glob(0x74EBE8).add(4 * (cx.li(16688)))) == cx.li(16676)) {
                                                                    { let p = Ptr::glob(0x758938).add(4 * ((4_i32).wrapping_mul(cx.li(16696)).wrapping_add(cx.li(16684)))); let t = cx.li(16688); cx.set_i(p, t); }
                                                                    { let t = 1; cx.set_li(16680, t); }
                                                                    break 'lp17;
                                                                }
                                                            }
                                                            let _ = cx.preinc_i(cx.lptr(16688));
                                                        }
                                                        if (cx.li(16680) == 0) {
                                                            Vertex = { let q0_ = cx.lpp(4189); let q1_ = cx.li(16692); let q2_ = cx.li(16684); ov_mesh_vertex(cx, q0_, q1_, q2_) };
                                                            { let q0_ = Ptr::glob(0x752638).add(4 * ((3_i32).wrapping_mul(cx.li(16700)))); let q1_ = Vertex; f_19e030(cx, q0_, q1_) };
                                                            { let p = Ptr::glob(0x758938).add(4 * ((4_i32).wrapping_mul(cx.li(16696)).wrapping_add(cx.li(16684)))); let t = cx.li(16700); cx.set_i(p, t); }
                                                            v14 = { let q0_ = cx.lpp(4189); let q1_ = cx.li(16692); let q2_ = cx.li(16684); f_1b45f0(cx, q0_, q1_, q2_) };
                                                            { let p = Ptr::glob(0x74EBE8).add(4 * (cx.postinc_i(cx.lptr(16700)))); let t = v14; cx.set_i(p, t); }
                                                        }
                                                    }
                                                    let _ = cx.preinc_i(cx.lptr(16684));
                                                }
                                                let _ = cx.preinc_i(cx.lptr(16696));
                                            }
                                        }
                                        let _ = cx.preinc_i(cx.lptr(16692));
                                    }
                                }
                                if (cx.li(16752) != 0) {
                                    { let t = a4.wrapping_sub(cx.li(16916)); cx.set_li(16668, t); }
                                    if (({ let q0_ = a3.add((1560_i32).wrapping_mul(cx.li(16916))); let q1_ = cx.lptr(16668); let q2_ = cx.lptr(16764); let q3_ = Ptr::glob(0x752638); let q4_ = cx.li(16700); let q5_ = Ptr::glob(0x758938); let q6_ = Ptr::glob(0x757078); let q7_ = cx.li(16752); let q8_ = 0.1f32; let q9_ = 0.08f32; let q10_ = 0; f_394ea0(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_, q10_) } as i32) != 0) {
                                        { let t = 0; cx.set_li(16656, t); }
                                        { let t = 0; cx.set_li(16660, t); }
                                        'lp18: loop {
                                            'blp18: {
                                                if (cx.li(16660) >= cx.li(16668)) {
                                                    { let t = cx.li(16916).wrapping_add(cx.li(16656)); cx.set_li(16916, t); }
                                                    break 'lp18;
                                                }
                                                { let t = a3.add((1560_i32).wrapping_mul(cx.li(16656).wrapping_add(cx.li(16916)))); cx.set_lpp(4163, t); }
                                                { let q0_ = cx.lptr(16636); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(16660).wrapping_add(cx.li(16916)))).add(1332); let q2_ = cx.lpp(4218); f_1aadb0(cx, q0_, q1_, q2_) };
                                                { let q0_ = cx.lpp(4163).add(1332); let q1_ = cx.lptr(16636); f_19e030(cx, q0_, q1_) };
                                                if (((cx.gc.static_object_filter_out_vertical_contacts_when_driving_on_all_wheels == 0) || (cx.li(16904) != 4)) || (cx.f(cx.lpp(4163).add(1336)) <= cx.gc.min_normal_up_to_filter_out)) {
                                                    { let q0_ = cx.lptr(16636); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(16660).wrapping_add(cx.li(16916)))).add(1276); let q2_ = cx.lpp(4218); f_1aadb0(cx, q0_, q1_, q2_) };
                                                    { let q0_ = cx.lpp(4163).add(1276); let q1_ = cx.lptr(16636); let q2_ = cx.lptr(16856); f_1a38d0(cx, q0_, q1_, q2_) };
                                                    { let p = cx.lpp(4163).add(1288); let t = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(16660).wrapping_add(cx.li(16916)))).add(1288)); cx.set_i(p, t); }
                                                    if (cx.b(cx.lpp(4227).add(4072)) != 0) {
                                                        if (cx.f(cx.lpp(4227).add(3784)) <= cx.gc.minimum_contact_velocity_to_filter) {
                                                            { let q0_ = cx.lptr(16568); let q1_ = cx.lpp(4163).add(1276); let q2_ = cx.lpp(4163).add(1332); let q3_ = cx.f(cx.lpp(4163).add(1288)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                                            { let q0_ = cx.lptr(16548); let q1_ = cx.lptr(16568); let q2_ = cx.lpp(4227).add(304); f_19d920(cx, q0_, q1_, q2_) };
                                                            { let p = cx.lptr(16540); let t = ({ let q0_ = cx.lptr(16548); let q1_ = cx.lpp(4163).add(1332); f_19d9c0(cx, q0_, q1_) } as f32); cx.set_f(p, t); }
                                                            if (cx.f(cx.lptr(16540)) > cx.gc.low_speed_max_backface_angle) {
                                                                break 'blp18;
                                                            }
                                                        } else {
                                                            { let q0_ = cx.lptr(16616); let q1_ = cx.lpp(4227).add(272); f_19e030(cx, q0_, q1_) };
                                                            { let q0_ = cx.lptr(16596); let q1_ = cx.lpp(4163).add(1276); let q2_ = cx.lpp(4227).add(304); f_19d920(cx, q0_, q1_, q2_) };
                                                            { let q0_ = cx.lptr(16616); let q1_ = cx.lpp(4227).add(288); let q2_ = cx.lptr(16596); f_1a6f00(cx, q0_, q1_, q2_) };
                                                            { let p = cx.lptr(16588); let t = ((-{ let q0_ = cx.lptr(16616); let q1_ = cx.lpp(4163).add(1332); f_19d9c0(cx, q0_, q1_) }) as f32); cx.set_f(p, t); }
                                                            { let p = cx.lptr(16584); let t = (((((cx.gc.compare_coeffecient * cx.f(cx.lptr(16588))) * cx.f(cx.ptr_at(cx.lpp(4227).add(3552)))) as f64) + (cx.gc.additional_penetration_allowance as f64)) as f32); cx.set_f(p, t); }
                                                            if (cx.f(cx.lpp(4163).add(1288)) > cx.f(cx.lptr(16584))) {
                                                                break 'blp18;
                                                            }
                                                        }
                                                    }
                                                    { let p = cx.lpp(4163).add(4); let t = cx.lpp(4227).add(240); cx.set_ptr(p, t); }
                                                    { let p = cx.lpp(4227).add(4856); let t = 0; cx.set_i(p, t); }
                                                    { let p = cx.lpp(4163).add(8); let t = 0; cx.set_i(p, t); }
                                                    { let p = cx.lpp(4227).add(4856); let t = 0; cx.set_i(p, t); }
                                                    { let p = cx.lpp(4163).add(216); let t = cx.lpp(4220); cx.set_ptr(p, t); }
                                                    { let p = cx.lpp(4163).add(220); let t = cx.li(16912); cx.set_i(p, t); }
                                                    if (cx.i(cx.lpp(4227).add(3740)) != 0) {
                                                        car_static_object_inertia_scale = cx.gc.car_static_object_inertia_scale_at_initiated_skid;
                                                    } else {
                                                        car_static_object_inertia_scale = cx.gc.car_static_object_inertia_scale;
                                                    }
                                                    { let p = cx.lpp(4163).add(1540); let t = car_static_object_inertia_scale; cx.set_f(p, t); }
                                                    { let p = cx.lpp(4163).add(1544); let t = cx.gc.car_static_object_inertia_scale_min; cx.set_f(p, t); }
                                                    { let p = cx.lptr(16536); let t = (vsmath::fabs(cx.f(cx.lpp(4163).add(1336))) as f32); cx.set_f(p, t); }
                                                    { let p = cx.lpp(4163).add(1272); let t = ((((cx.gc.static_object_contact_restitution_with_horizontal_features * cx.f(cx.lptr(16536))) as f64) + (((1.0f32 - cx.f(cx.lptr(16536))) * cx.gc.static_object_contact_restitution_with_vertical_features) as f64)) as f32); cx.set_f(p, t); }
                                                    { let p = cx.lpp(4163).add(1268); let t = ((((cx.gc.static_object_contact_friction_with_horizontal_features * cx.f(cx.lptr(16536))) as f64) + (((1.0f32 - cx.f(cx.lptr(16536))) * cx.gc.static_object_contact_friction_with_vertical_features) as f64)) as f32); cx.set_f(p, t); }
                                                    if (cx.f(cx.lptr(16536)) < 0.1f32) {
                                                        { let p = cx.lpp(4163).add(1280); let t = cx.i(cx.lpp(4227).add(308)); cx.set_i(p, t); }
                                                    }
                                                    if (cx.gc.draw_chassis_CD_contacts != 0.0f32) {
                                                        { let q0_ = cx.lptr(16520); let q1_ = cx.lpp(4163).add(1276); let q2_ = cx.lpp(4163).add(1332); let q3_ = 1.0f32; f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                                        v43 = { let p = { let q0_ = cx.lptr(140); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                                        v38 = cx.i(Ptr::glob(0x77FD8C));
                                                        v33 = { let q0_ = cx.lptr(124); let q1_ = cx.li(16520); let q2_ = cx.li(16524); let q3_ = cx.li(16528); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                        v16 = { let q0_ = cx.lptr(108); let q1_ = cx.i(cx.lpp(4163).add(1276)); let q2_ = cx.i(cx.lpp(4163).add(1280)); let q3_ = cx.i(cx.lpp(4163).add(1284)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v16, v33, v38, v43);
                                                        { let q0_ = cx.lptr(16520); let q1_ = cx.lpp(4163).add(1276); let q2_ = cx.lpp(4163).add(1332); let q3_ = (((cx.gc.draw_chassis_CD_contacts as f64) * (cx.f(cx.lpp(4163).add(1288)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                                        { let q0_ = cx.lptr(16500); let q1_ = cx.lpp(4163).add(1276); f_19e030(cx, q0_, q1_) };
                                                        { let p = cx.lptr(16504); let t = (((cx.f(cx.lptr(16504)) as f64) + 0.05f64) as f32); cx.set_f(p, t); }
                                                        v44 = { let p = { let q0_ = cx.lptr(104); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                                        v39 = cx.i(Ptr::glob(0x77FD48));
                                                        v34 = { let q0_ = cx.lptr(88); let q1_ = cx.li(16520); let q2_ = cx.li(16524); let q3_ = cx.li(16528); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                        v17 = { let q0_ = cx.lptr(72); let q1_ = cx.li(16500); let q2_ = cx.li(16504); let q3_ = cx.li(16508); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v17, v34, v39, v44);
                                                    }
                                                    let _ = cx.preinc_i(cx.lptr(16656));
                                                }
                                            }
                                            let _ = cx.preinc_i(cx.lptr(16660));
                                        }
                                    }
                                }
                            } else if (cx.i(cx.ptr_at(cx.lpp(4220).add(104)).add(16)) != 0) {
                                { let t = cx.ptr_at(cx.ptr_at(cx.lpp(4220).add(104)).add(16)); cx.set_lpp(4123, t); }
                                { let t = a4.wrapping_sub(cx.li(16916)); cx.set_li(16484, t); }
                                if ({ let q0_ = cx.lpp(4123); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(16916))); let q2_ = cx.lptr(16484); let q3_ = cx.lptr(16764); let q4_ = 0.0f32; f_378750(cx, q0_, q1_, q2_, q3_, q4_) } != 0) {
                                    { let t = 0; cx.set_li(16476, t); }
                                    if ((((((((cx.gc.TT_forward_extent_extension_max != 0.0f32) && (cx.b(cx.lpp(4123).add(54)) == 1)) && (cx.b(cx.lpp(4123).add(53)) == 0)) && (cx.b(cx.lpp(4123).add(52)) == 0)) && (cx.b(cx.lpp(4227).add(4072)) != 0)) && (cx.f(cx.lpp(4227).add(3784)) > cx.gc.TT_min_car_velocity_for_through_tree_transition)) && (cx.gc.TT_max_capsule_radius > cx.f(cx.ptr_at(cx.lpp(4123).add(68)).add(28)))) && (cx.f(cx.ptr_at(cx.lpp(4123).add(68)).add(32)) > cx.gc.TT_min_capsule_halflength)) {
                                        if (cx.f(cx.lpp(4227).add(3784)) <= cx.gc.TT_max_car_velocity_for_through_tree_transition) {
                                            { let p = cx.lptr(16472); let t = ((((((cx.gc.TT_forward_extent_extension_max - cx.gc.TT_forward_extent_extension_min) * (cx.f(cx.lpp(4227).add(3784)) - cx.gc.TT_min_car_velocity_for_through_tree_transition)) / (cx.gc.TT_max_car_velocity_for_through_tree_transition - cx.gc.TT_min_car_velocity_for_through_tree_transition)) as f64) + (cx.gc.TT_forward_extent_extension_min as f64)) as f32); cx.set_f(p, t); }
                                        } else {
                                            { let t = (cx.gc.TT_forward_extent_extension_max.to_bits() as i32); cx.set_li(16472, t); }
                                        }
                                        { let p = cx.lptr(16816); let t = (((cx.f(cx.lptr(16816)) as f64) + (cx.f(cx.lptr(16472)) as f64)) as f32); cx.set_f(p, t); }
                                        { let t = a4.wrapping_sub(cx.li(16916)); cx.set_li(16484, t); }
                                        { let q0_ = cx.lpp(4123); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(16916))); let q2_ = cx.lptr(16484); let q3_ = cx.lptr(16764); let q4_ = 0.0f32; f_378750(cx, q0_, q1_, q2_, q3_, q4_) };
                                        { let p = cx.lptr(16816); let t = (((cx.f(cx.lptr(16816)) as f64) - (cx.f(cx.lptr(16472)) as f64)) as f32); cx.set_f(p, t); }
                                        { let t = 1; cx.set_li(16476, t); }
                                    }
                                    { let t = 0; cx.set_li(16468, t); }
                                    if (((cx.gc.car_static_objects_number_of_binary_divisions != 0) && (cx.b(cx.lpp(4227).add(4072)) != 0)) && (cx.li(16476) == 0)) {
                                        v18 = { let q0_ = cx.lpp(4227).add(240); ov_car_velocity(cx, q0_) };
                                        { let q0_ = cx.lptr(16452); let q1_ = v18; f_19e030(cx, q0_, q1_) };
                                        if (({ let q0_ = cx.lptr(16452); f_1a0480(cx, q0_) } as f64) > ((cx.gc.car_static_objects_min_approaching_velocity_for_timestep_subdivision as f64) * (cx.gc.car_static_objects_min_approaching_velocity_for_timestep_subdivision as f64))) {
                                            { let q0_ = cx.lptr(16432); let q1_ = cx.lptr(16780); f_19e030(cx, q0_, q1_) };
                                            { let t = 0; cx.set_li(16424, t); }
                                            'lp19: loop {
                                                if !(cx.li(16424) < cx.li(16484)) { break 'lp19; }
                                                'blp19: {
                                                    v45 = ({ let q0_ = a3.add((1560_i32).wrapping_mul(cx.li(16424).wrapping_add(cx.li(16916)))).add(1332); let q1_ = cx.lptr(16432); f_19d9c0(cx, q0_, q1_) } as f32);
                                                    if ((cx.gc.car_static_objects_max_normal_car_x_for_timestep_subdivision as f64) > vsmath::fabs(v45)) {
                                                        { let t = 1; cx.set_li(16468, t); }
                                                        break 'lp19;
                                                    }
                                                }
                                                let _ = cx.preinc_i(cx.lptr(16424));
                                            }
                                            if (cx.li(16468) != 0) {
                                                { let q0_ = cx.lptr(16388); let q1_ = cx.lptr(16764); f_19e030(cx, q0_, q1_) };
                                                { let q0_ = cx.lptr(16348); let q1_ = cx.lptr(16764); f_19e030(cx, q0_, q1_) };
                                                v46 = cx.lpp(4218);
                                                v19 = { let q0_ = cx.lpp(4227).add(240); ov_car_velocity(cx, q0_) };
                                                { let q0_ = cx.lptr(16308); let q1_ = v19; let q2_ = v46; f_1aad50(cx, q0_, q1_, q2_) };
                                                { let q0_ = cx.lptr(16328); let q1_ = cx.lptr(16308); let q2_ = cx.f(cx.ptr_at(cx.lpp(4227).add(3552))); f_19d970(cx, q0_, q1_, q2_) };
                                                { let q0_ = cx.lptr(16408); let q1_ = cx.lptr(16388); let q2_ = cx.lptr(16328); f_19d920(cx, q0_, q1_, q2_) };
                                                { let t = 0; cx.set_li(16296, t); }
                                                { let t = 0; cx.set_li(16300, t); }
                                                'lp20: loop {
                                                    if !(cx.li(16300) < cx.gc.car_static_objects_number_of_binary_divisions) { break 'lp20; }
                                                    'blp20: {
                                                        { let q0_ = cx.lptr(16368); let q1_ = cx.lptr(16408); let q2_ = cx.lptr(16388); f_1a38d0(cx, q0_, q1_, q2_) };
                                                        { let q0_ = cx.lptr(16368); let q1_ = 0.5f32; f_19dab0(cx, q0_, q1_) };
                                                        { let q0_ = cx.lptr(16764); let q1_ = cx.lptr(16368); f_19e030(cx, q0_, q1_) };
                                                        if ({ let q0_ = cx.lpp(4123); let q1_ = cx.f(cx.lptr(16764)); let q2_ = cx.li(16768); let q3_ = cx.li(16772); let q4_ = cx.li(16776); let q5_ = cx.li(16780); let q6_ = cx.li(16784); let q7_ = cx.li(16788); let q8_ = cx.li(16792); let q9_ = cx.li(16796); let q10_ = cx.li(16800); let q11_ = cx.li(16804); let q12_ = cx.li(16808); let q13_ = cx.li(16812); let q14_ = cx.li(16816); let q15_ = cx.li(16820); let q16_ = cx.li(16824); f_3789c0(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_, q10_, q11_, q12_, q13_, q14_, q15_, q16_) } != 0) {
                                                            { let q0_ = cx.lptr(16388); let q1_ = cx.lptr(16368); f_19e030(cx, q0_, q1_) };
                                                            { let t = 1; cx.set_li(16296, t); }
                                                            { let t = 10; cx.set_li(16288, t); }
                                                            { let q0_ = cx.lptr(680); let q1_ = 1560; let q2_ = 10; let q3_ = Ptr::raw(Ptr::func(0x398D80).bits()); ov_vector_ctor(cx, q0_, q1_, q2_, q3_) };
                                                            if (({ let q0_ = cx.lpp(4123); let q1_ = cx.lptr(680); let q2_ = cx.lptr(16288); let q3_ = cx.lptr(16764); let q4_ = 0.08f32; f_378750(cx, q0_, q1_, q2_, q3_, q4_) } == 0) || (cx.li(16288) == 0)) {
                                                                { let t = 1; cx.set_li(672, t); }
                                                            }
                                                        } else {
                                                            { let q0_ = cx.lptr(16408); let q1_ = cx.lptr(16368); f_19e030(cx, q0_, q1_) };
                                                            { let t = 0; cx.set_li(16296, t); }
                                                        }
                                                    }
                                                    let _ = cx.preinc_i(cx.lptr(16300));
                                                }
                                                if (cx.li(16296) == 0) {
                                                    { let p = cx.lptr(668); let t = ((1.0f64 / (((cx.gc.car_static_objects_number_of_binary_divisions as f32) + 1.0f32) as f64)) as f32); cx.set_f(p, t); }
                                                    { let q0_ = cx.lptr(16764); let q1_ = cx.lptr(16328); let q2_ = cx.f(cx.lptr(668)); f_1a0390(cx, q0_, q1_, q2_) };
                                                }
                                                { let t = a4.wrapping_sub(cx.li(16916)); cx.set_li(16484, t); }
                                                { let q0_ = cx.lpp(4123); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(16916))); let q2_ = cx.lptr(16484); let q3_ = cx.lptr(16764); let q4_ = 0.0f32; f_378750(cx, q0_, q1_, q2_, q3_, q4_) };
                                                { let q0_ = cx.lptr(652); let q1_ = cx.lptr(16764); let q2_ = cx.lptr(16348); f_19d920(cx, q0_, q1_, q2_) };
                                                { let t = 0; cx.set_li(624, t); }
                                                'lp21: loop {
                                                    if !(cx.li(624) < cx.li(16484)) { break 'lp21; }
                                                    'blp21: {
                                                        v20 = (1560_i32).wrapping_mul(cx.li(624).wrapping_add(cx.li(16916)));
                                                        v21 = { let q0_ = a3.add(v20).add(1332); let q1_ = cx.lptr(652); f_19d9c0(cx, q0_, q1_) };
                                                        { let p = a3.add((1560_i32).wrapping_mul(cx.li(624).wrapping_add(cx.li(16916)))).add(1288); let t = (((cx.f(a3.add(v20).add(1288)) as f64) - v21) as f32); cx.set_f(p, t); }
                                                    }
                                                    let _ = cx.preinc_i(cx.lptr(624));
                                                }
                                            }
                                        }
                                    }
                                    { let t = 0; cx.set_li(616, t); }
                                    { let t = 0; cx.set_li(620, t); }
                                    'lp22: loop {
                                        if !(cx.li(620) < cx.li(16484)) { break 'lp22; }
                                        'blp22: {
                                            { let t = a3.add((1560_i32).wrapping_mul(cx.li(616).wrapping_add(cx.li(16916)))); cx.set_lpp(153, t); }
                                            { let q0_ = cx.lptr(596); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(620).wrapping_add(cx.li(16916)))).add(1332); let q2_ = cx.lpp(4218); f_1aadb0(cx, q0_, q1_, q2_) };
                                            { let q0_ = cx.lpp(153).add(1332); let q1_ = cx.lptr(596); f_19e030(cx, q0_, q1_) };
                                            if (((cx.gc.static_object_filter_out_vertical_contacts_when_driving_on_all_wheels == 0) || (cx.li(16904) != 4)) || (cx.f(cx.lpp(153).add(1336)) <= cx.gc.min_normal_up_to_filter_out)) {
                                                { let q0_ = cx.lptr(596); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(620).wrapping_add(cx.li(16916)))).add(1276); let q2_ = cx.lpp(4218); f_1aadb0(cx, q0_, q1_, q2_) };
                                                { let q0_ = cx.lpp(153).add(1276); let q1_ = cx.lptr(596); let q2_ = cx.lptr(16856); f_1a38d0(cx, q0_, q1_, q2_) };
                                                { let p = cx.lpp(153).add(1288); let t = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(620).wrapping_add(cx.li(16916)))).add(1288)); cx.set_i(p, t); }
                                                { let p = cx.lpp(153).add(8); let t = cx.lpp(4227).add(240); cx.set_ptr(p, t); }
                                                { let p = cx.lpp(4227).add(4856); let t = 0; cx.set_i(p, t); }
                                                { let p = cx.lpp(153).add(4); let t = 0; cx.set_i(p, t); }
                                                { let p = cx.lpp(4227).add(4856); let t = 0; cx.set_i(p, t); }
                                                { let p = cx.lpp(153).add(216); let t = cx.lpp(4220); cx.set_ptr(p, t); }
                                                { let p = cx.lpp(153).add(220); let t = cx.li(16912); cx.set_i(p, t); }
                                                if (cx.li(16476) != 0) {
                                                    { let p = cx.lpp(153).add(1540); let t = cx.gc.TT_car_static_object_inertia_scale; cx.set_f(p, t); }
                                                    { let p = cx.lpp(153).add(1544); let t = cx.gc.TT_car_static_object_inertia_scale_min; cx.set_f(p, t); }
                                                } else {
                                                    if (cx.i(cx.lpp(4227).add(3740)) != 0) {
                                                        v22 = cx.gc.car_static_object_inertia_scale_at_initiated_skid;
                                                    } else {
                                                        v22 = cx.gc.car_static_object_inertia_scale;
                                                    }
                                                    { let p = cx.lpp(153).add(1540); let t = v22; cx.set_f(p, t); }
                                                    { let p = cx.lpp(153).add(1544); let t = cx.gc.car_static_object_inertia_scale_min; cx.set_f(p, t); }
                                                }
                                                { let p = cx.lptr(588); let t = (vsmath::fabs(cx.f(cx.lpp(153).add(1336))) as f32); cx.set_f(p, t); }
                                                { let p = cx.lpp(153).add(1272); let t = ((((cx.gc.static_object_contact_restitution_with_horizontal_features * cx.f(cx.lptr(588))) as f64) + (((1.0f32 - cx.f(cx.lptr(588))) * cx.gc.static_object_contact_restitution_with_vertical_features) as f64)) as f32); cx.set_f(p, t); }
                                                { let p = cx.lpp(153).add(1268); let t = ((((cx.gc.static_object_contact_friction_with_horizontal_features * cx.f(cx.lptr(588))) as f64) + (((1.0f32 - cx.f(cx.lptr(588))) * cx.gc.static_object_contact_friction_with_vertical_features) as f64)) as f32); cx.set_f(p, t); }
                                                if (cx.f(cx.lptr(588)) < 0.1f32) {
                                                    { let p = cx.lpp(153).add(1280); let t = cx.i(cx.lpp(4227).add(308)); cx.set_i(p, t); }
                                                }
                                                if (cx.gc.draw_chassis_CD_contacts != 0.0f32) {
                                                    { let q0_ = cx.lptr(572); let q1_ = cx.lpp(153).add(1276); let q2_ = cx.lpp(153).add(1332); let q3_ = 1.0f32; f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                                    v47 = { let p = { let q0_ = cx.lptr(68); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                                    v40 = cx.i(Ptr::glob(0x77FD8C));
                                                    v35 = { let q0_ = cx.lptr(52); let q1_ = cx.li(572); let q2_ = cx.li(576); let q3_ = cx.li(580); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v23 = { let q0_ = cx.lptr(36); let q1_ = cx.i(cx.lpp(153).add(1276)); let q2_ = cx.i(cx.lpp(153).add(1280)); let q3_ = cx.i(cx.lpp(153).add(1284)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v23, v35, v40, v47);
                                                    { let q0_ = cx.lptr(572); let q1_ = cx.lpp(153).add(1276); let q2_ = cx.lpp(153).add(1332); let q3_ = (((cx.gc.draw_chassis_CD_contacts as f64) * (cx.f(cx.lpp(153).add(1288)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                                    { let q0_ = cx.lptr(552); let q1_ = cx.lpp(153).add(1276); f_19e030(cx, q0_, q1_) };
                                                    { let p = cx.lptr(556); let t = (((cx.f(cx.lptr(556)) as f64) + 0.05f64) as f32); cx.set_f(p, t); }
                                                    v48 = { let p = { let q0_ = cx.lptr(32); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                                    v41 = cx.i(Ptr::glob(0x77FD48));
                                                    v36 = { let q0_ = cx.lptr(16); let q1_ = cx.li(572); let q2_ = cx.li(576); let q3_ = cx.li(580); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v24 = { let q0_ = cx.lptr(0); let q1_ = cx.li(552); let q2_ = cx.li(556); let q3_ = cx.li(560); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v24, v36, v41, v48);
                                                }
                                                let _ = cx.preinc_i(cx.lptr(616));
                                            }
                                        }
                                        let _ = cx.preinc_i(cx.lptr(620));
                                    }
                                    { let t = cx.li(16916).wrapping_add(cx.li(616)); cx.set_li(16916, t); }
                                }
                            }
                            if ((cx.li(16916) != 0) && (cx.gc.resolve_collisions != 0)) {
                                { let q0_ = cx.lptr(368); f_1b42f0(cx, q0_) };
                                { let t = cx.i(cx.ptr_at(cx.lpp(4227).add(3552))); cx.set_li(368, t); }
                                { let t = (cx.gc.max_NO_iterations_coeffecient.to_bits() as i32); cx.set_li(444, t); }
                                { let t = (cx.gc.contacts_relaxation_timestep_multiplier.to_bits() as i32); cx.set_li(376, t); }
                                { let t = (cx.gc.projection_timestep_multiplier.to_bits() as i32); cx.set_li(372, t); }
                                { let t = cx.gc.treat_stacking; cx.set_li(448, t); }
                                { let t = cx.gc.handle_contact_pairs; cx.set_li(492, t); }
                                { let t = cx.gc.do_collision_resolution_for_contacts; cx.set_li(456, t); }
                                { let t = cx.gc.do_error_resolution_for_contacts; cx.set_li(460, t); }
                                { let t = (cx.gc.diag_scale.to_bits() as i32); cx.set_li(452, t); }
                                { let q0_ = cx.lptr(404); let q1_ = a5; f_19e030(cx, q0_, q1_) };
                                { let q0_ = cx.lptr(380); let q1_ = Ptr::gc(0); f_19e030(cx, q0_, q1_) };
                                { let t = -1; cx.set_li(360, t); }
                                { let t = 0; cx.set_li(356, t); }
                                { let t = 0; cx.set_li(352, t); }
                                if (cx.b(cx.lpp(4227).add(4072)) != 0) {
                                    { let t = 0; cx.set_li(348, t); }
                                    'lp23: loop {
                                        if !(cx.li(348) < cx.li(16916)) { break 'lp23; }
                                        'blp23: {
                                            v25 = { let q0_ = cx.lpp(4227); let q1_ = 0; f_1bb200(cx, q0_, q1_) };
                                            v49 = ({ let q0_ = a3.add((1560_i32).wrapping_mul(cx.li(348))).add(1332); let q1_ = v25; f_19d9c0(cx, q0_, q1_) } as f32);
                                            { let p = cx.lptr(352); let t = (vsmath::fabs(v49) as f32); cx.set_f(p, t); }
                                            if (((cx.gc.wl_max_cos_of_normal_up as f64) > vsmath::fabs(cx.f(a3.add((1560_i32).wrapping_mul(cx.li(348))).add(1336)))) && (cx.f(cx.lptr(352)) > cx.gc.wl_min_of_cos_of_normal_car_forward)) {
                                                v26 = { let q0_ = cx.lpp(4227); let q1_ = 1; f_1bb200(cx, q0_, q1_) };
                                                v50 = ({ let q0_ = a3.add((1560_i32).wrapping_mul(cx.li(348))).add(1332); let q1_ = v26; f_19d9c0(cx, q0_, q1_) } as f32);
                                                if ((cx.gc.wl_min_of_cos_of_normal_car_forward as f64) > vsmath::fabs(v50)) {
                                                    { let p = cx.lptr(356); let t = ({ let q0_ = a3.add((1560_i32).wrapping_mul(cx.li(348))).add(1332); let q1_ = cx.lpp(4227).add(272); f_19d9c0(cx, q0_, q1_) } as f32); cx.set_f(p, t); }
                                                    if (vsmath::fabs(cx.f(cx.lptr(356))) >= (cx.gc.wl_necessary_speed_change as f64)) {
                                                        { let t = cx.li(348); cx.set_li(360, t); }
                                                        break 'lp23;
                                                    }
                                                }
                                            }
                                        }
                                        let _ = cx.preinc_i(cx.lptr(348));
                                    }
                                }
                                v5 = { let q0_ = a3; let q1_ = cx.li(16916); let q2_ = Ptr::NULL; let q3_ = 0; let q4_ = Ptr::NULL; let q5_ = 0; let q6_ = cx.lptr(368); f_389630(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
                                if ((cx.b(cx.lpp(4227).add(4072)) != 0) && (cx.li(360) != -1)) {
                                    { let p = cx.lptr(344); let t = ({ let q0_ = a3.add((1560_i32).wrapping_mul(cx.li(360))).add(1332); let q1_ = cx.lpp(4227).add(272); f_19d9c0(cx, q0_, q1_) } as f32); cx.set_f(p, t); }
                                    { let p = cx.lptr(340); let t = (vsmath::fabs((((cx.f(cx.lptr(344)) as f64) - (cx.f(cx.lptr(356)) as f64)) as f32)) as f32); cx.set_f(p, t); }
                                    if (cx.f(cx.lptr(340)) > cx.gc.wl_necessary_speed_change) {
                                        { let p = cx.lptr(336); let t = (((((cx.f(cx.lptr(340)) - cx.gc.wl_necessary_speed_change) * cx.f(cx.lptr(352))) as f64) / ((cx.gc.wl_max_speed_change - cx.gc.wl_necessary_speed_change) as f64)) as f32); cx.set_f(p, t); }
                                        if (cx.f(cx.lptr(336)) > 1.0f32) {
                                            { let t = 1065353216; cx.set_li(336, t); }
                                        }
                                        { let p = cx.lptr(336); let t = (((cx.f(cx.lptr(336)) as f64) * (cx.gc.wl_delta_omega_car_z as f64)) as f32); cx.set_f(p, t); }
                                        v51 = ((0.0f64 - (cx.f(cx.lptr(336)) as f64)) as f32);
                                        v27 = { let q0_ = cx.lpp(4227).add(240); let q1_ = 2; ov_car_rot_row(cx, q0_, q1_) };
                                        { let q0_ = cx.lpp(4227).add(288); let q1_ = v27; let q2_ = v51; f_1a0390(cx, q0_, q1_, q2_) };
                                        { let q0_ = cx.lptr(320); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(360))).add(1276); let q2_ = cx.lpp(4227).add(304); f_19d920(cx, q0_, q1_, q2_) };
                                        v28 = ({ let q0_ = cx.lptr(320); f_19d9f0(cx, q0_) } as f64);
                                        { let p = cx.lpp(4227).add(276); let t = (((v28 * (cx.f(cx.lptr(336)) as f64)) + (cx.f(cx.lpp(4227).add(276)) as f64)) as f32); cx.set_f(p, t); }
                                    }
                                }
                                { let t = 0; cx.set_li(308, t); }
                                { let p = cx.lptr(304); let t = ((0.0f64 - 3.4028235e38f64) as f32); cx.set_f(p, t); }
                                { let t = 0; cx.set_li(312, t); }
                                'lp24: loop {
                                    if !(cx.li(312) < cx.li(16916)) { break 'lp24; }
                                    'blp24: {
                                        { let p = cx.lptr(300); let t = (vsmath::fabs(cx.f(a3.add((1560_i32).wrapping_mul(cx.li(312))).add(236))) as f32); cx.set_f(p, t); }
                                        if (cx.f(cx.lptr(300)) > cx.f(cx.lptr(304))) {
                                            { let t = cx.li(300); cx.set_li(304, t); }
                                            { let t = cx.li(312); cx.set_li(308, t); }
                                        }
                                    }
                                    let _ = cx.preinc_i(cx.lptr(312));
                                }
                                if (cx.f(cx.lptr(304)) > 0.0f32) {
                                    if ({ let q0_ = cx.lpp(4220); f_1e5ca0(cx, q0_) } != 0) {
                                        { let q0_ = cx.lptr(216); f_238b0(cx, q0_) };
                                        { let t = cx.li(16912); cx.set_li(216, t); }
                                        { let t = -1; cx.set_li(220, t); }
                                        { let t = cx.li(304); cx.set_li(256, t); }
                                        { let q0_ = cx.lptr(224); let q1_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(308))).add(1276)); let q2_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(308))).add(1280)); let q3_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(308))).add(1284)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                        { let q0_ = cx.lptr(240); let q1_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(308))).add(1332)); let q2_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(308))).add(1336)); let q3_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(308))).add(1340)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                        { let t = 0; cx.set_li(260, t); }
                                        { let t = 1106247680; cx.set_li(288, t); }
                                        { let t = -1; let o = cx.li(292); cx.set_li(292, (o & !(255 << 0)) | ((t & 255) << 0)); }
                                        { let t = cx.lpp(4227).add(240); cx.set_lpp(66, t); }
                                        { let t = 0; cx.set_li(268, t); }
                                        { let q0_ = cx.lpp(4230); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(308))); let q2_ = cx.lptr(272); f_1b39b0(cx, q0_, q1_, q2_) };
                                        { let q0_ = Ptr::glob(0x6FAA34); let q1_ = 0; let q2_ = cx.lptr(216); ov_message(cx, q0_, q1_, q2_) };
                                    } else {
                                        { let q0_ = cx.lptr(156); f_1b4210(cx, q0_) };
                                        { let t = cx.lpp(4220); cx.set_lpp(40, t); }
                                        { let t = cx.li(16912); cx.set_li(156, t); }
                                        { let t = cx.li(304); cx.set_li(196, t); }
                                        { let t = 0; cx.set_li(200, t); }
                                        { let t = 0; cx.set_li(204, t); }
                                        { let q0_ = cx.lptr(164); let q1_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(308))).add(1276)); let q2_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(308))).add(1280)); let q3_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(308))).add(1284)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                        { let q0_ = cx.lptr(180); let q1_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(308))).add(1332)); let q2_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(308))).add(1336)); let q3_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(308))).add(1340)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                        { let q0_ = Ptr::glob(0x6FAA34); let q1_ = 2; let q2_ = cx.lptr(156); ov_message(cx, q0_, q1_, q2_) };
                                    }
                                    if (cx.b(cx.lpp(4227).add(4072)) != 0) {
                                        { let t = { let q0_ = cx.lpp(4220); f_1b46f0(cx, q0_) }; cx.set_li(148, t); }
                                        if (cx.li(148) != 0) {
                                            v29 = ({ let q0_ = cx.lpp(4220); f_1b46c0(cx, q0_) } as f64);
                                            if ((cx.f(cx.lptr(304)) as f64) >= v29) {
                                                { let t = ov_timeline_bridge(cx, Ptr::glob(0x6F5BC8)); cx.set_li(144, t); }
                                                if (cx.li(144) != 0) {
                                                    { let q0_ = cx.li(144); let q1_ = v30; let q2_ = cx.li(148); ov_play_timeline(cx, q0_, q1_, q2_) };
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            { let t = 0; cx.set_li(16916, t); }
                        }
                        let _ = cx.preinc_i(cx.lptr(16884));
                        continue 'b_LABEL_15;
                        #[allow(unreachable_code)]
                        break;
                    }
                }
            }
        }
        v6 = cx.preinc_i(cx.lptr(16912));
    }
    v42 = ({ let q0_ = cx.li(16912); let q1_ = v6; crate::ov_zero_i!(cx, q0_, q1_) } as i64);
    return (v42 as i32);
}

/// PhysicsManager__1AAD50 0x001AAD50
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1aad50(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> () {
    { let p = a1; let t = (f_19d9c0(cx, a3, a2) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (f_19d9c0(cx, a3.add(16), a2) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (f_19d9c0(cx, a3.add(32), a2) as f32); cx.set_f(p, t); }
}

/// PhysicsManager__1AADB0 0x001AADB0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1aadb0(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = (((((cx.f(a3) * cx.f(a2)) + (cx.f(a3.add(16)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(32)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((((cx.f(a3.add(4)) * cx.f(a2)) + (cx.f(a3.add(20)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(36)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((((cx.f(a3.add(8)) * cx.f(a2)) + (cx.f(a3.add(24)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(40)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    return a3;
}

/// PhysicsManager__1AAE70 0x001AAE70
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1aae70(cx: &mut Cx, a1: Ptr, a2: Ptr, a3: Ptr) -> () {
    f_1aad50(cx, a1, a2, a3);
    f_1aad50(cx, a1.add(12), a2.add(12), a3);
    f_1aad50(cx, a1.add(24), a2.add(24), a3);
}

/// PhysicsManager__1AAED0 0x001AAED0
pub fn f_1aaed0(cx: &mut Cx, a1: i32, a3: Ptr, a4: Ptr, a5: i32, a6: Ptr) -> i64 {
    cx.push_frame(904);
    let r = f_1aaed0_body(cx, a1, a3, a4, a5, a6);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1aaed0_body(cx: &mut Cx, a1: i32, a3: Ptr, a4: Ptr, a5: i32, a6: Ptr) -> i64 {
    let mut a2 = 0.0f64;
    let mut v6 = Ptr::NULL;
    let mut v7 = Ptr::NULL;
    let mut v8 = 0i32;
    let mut v9 = 0i32;
    let mut v10 = Ptr::NULL;
    let mut v11 = Ptr::NULL;
    let mut v12 = 0i32;
    let mut v13 = 0i32;
    let mut v14 = 0.0f64;
    let mut v15 = 0i32;
    let mut v16 = 0.0f64;
    let mut v17 = 0i32;
    let mut v18 = Ptr::NULL;
    let mut v19 = 0.0f64;
    let mut v20 = Ptr::NULL;
    let mut v21 = Ptr::NULL;
    let mut v22 = 0i32;
    let mut v23 = 0i32;
    let mut v24 = Ptr::NULL;
    let mut v27 = 0.0f32;
    let mut v28 = Ptr::NULL;
    let mut v29 = 0.0f32;
    let mut v30 = 0i64;
    let mut v31 = 0i32;
    let mut v32 = 0i64;
    let mut v33 = 0i64;
    let mut v34 = 0i32;
    let mut v35 = 0.0f32;
    let mut v36 = 0.0f32;
    let mut v37 = 0.0f32;
    { let t = a1; cx.set_lpp(209, Ptr::raw(t)); }
    let _ = cx.preinc_i(Ptr::glob(0x750358));
    { let t = 0; cx.set_li(832, t); }
    { let t = 0; cx.set_li(828, t); }
    'lp12: loop {
        if !(cx.li(828) < cx.i(a3)) { break 'lp12; }
        'blp12: {
            if (({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(209)); let q1_ = a2; let q2_ = cx.li(828); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(28); cx.b(p) } != 0) && ((({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(209)); let q1_ = a2; let q2_ = cx.li(828); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(5084); cx.i(p) } == 0) || ({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(209)); let q1_ = a2; let q2_ = cx.li(828); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(5084); cx.i(p) } == 1)) || ({ let p = { let p = { let q0_ = cx.ptr_at(cx.lpp(209)); let q1_ = a2; let q2_ = cx.li(828); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(5084); cx.i(p) } == 4))) {
                { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(209)); let q1_ = a2; let q2_ = cx.li(828); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }; cx.set_lpp(206, t); }
                if ({ let p = ov_physics_manager(cx).add(2836); cx.i(p) } != 0) {
                    if ({ let p = { let p = ov_physics_manager(cx).add(2836); cx.ptr_at(p) }.add(40); cx.b(p) } == 0) {
                        v6 = ov_physics_manager(cx);
                        { let q0_ = cx.ptr_at(v6.add(2836)); let q1_ = 0.0f32; f_385b80(cx, q0_, q1_) };
                    }
                    v30 = cx.q(cx.ptr_at(cx.ptr_at(cx.lpp(206).add(3556))).add(8));
                    v27 = cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(206).add(3556))));
                    v7 = ov_physics_manager(cx);
                    { let t = ({ let q0_ = cx.ptr_at(v7.add(2836)); let q1_ = v8; let q2_ = cx.lptr(816); let q3_ = v27; let q4_ = f32::from_bits(v30 as u32); let q5_ = ((v30 >> 32) & 4294967295) as i32; f_385920(cx, q0_, q1_, q2_, q3_, q4_, q5_) } as i32); cx.set_li(808, t); }
                    if (cx.li(816) == 0) {
                        let _ = ov_physics_manager(cx);
                        crate::ov_ignore!(cx, 1, "!CAxisSortedList__GetCollisionCandidates(): The number of returned candidates could not fit into CAxisSortedList's preallocated array of %d, change the constructor parameters\n");
                    }
                    { let t = cx.lpp(206).add(3604); cx.set_lpp(201, t); }
                    if ((cx.li(808) != 0) && (cx.i(cx.lpp(201)) >= 3)) {
                        { let p = cx.ptr_at(cx.ptr_at(cx.lpp(206).add(3556))).add(56); let t = (((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(206).add(3556))).add(56)) as f64) + (cx.f(Ptr::gc(0).add(1768)) as f64)) as f32); cx.set_f(p, t); }
                        { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(206).add(3556))); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(206).add(3556))).add(28); let q2_ = cx.f(Ptr::gc(0).add(1772)); f_1a0390(cx, q0_, q1_, q2_) };
                    }
                    // LABEL_15:
                    'b_LABEL_15: loop {
                        { let t = 0; cx.set_li(800, t); }
                        'lp13: loop {
                            if !(cx.li(800) < cx.li(808)) { break 'lp13; }
                            'blp13: {
                                { let t = cx.ptr_at(Ptr::raw(cx.li(816).wrapping_add((4_i32).wrapping_mul(cx.li(800))))); cx.set_lpp(199, t); }
                                if (((cx.lpp(199).bits() != 0) && (cx.i(cx.lpp(199).add(100)) != 0)) && ((cx.i(cx.lpp(199).add(152)) & 1) != 0)) {
                                    { let t = cx.ptr_at(cx.ptr_at(cx.lpp(199).add(100)).add(32)); cx.set_lpp(198, t); }
                                    { let q0_ = cx.lptr(724); ov_identity_p(cx, q0_) };
                                    if ({ let q0_ = cx.ptr_at(cx.lpp(199).add(100)); f_1b41f0(cx, q0_) } != 0) {
                                        if (cx.i(cx.ptr_at(cx.lpp(199).add(100)).add(36)) == 0) {
                                            if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
                                                v9 = crate::ov_zero_i!(cx, "pObj->m_pMovableObject->m_pRigidBody", ".\\src\\Physics\\PhysicsManager.cpp", 4412);
                                                { let t = v9; cx.set_li(696, t); }
                                                if (v9 == 3) {
                                                }
                                                { let t = 257; let o = cx.li(0); cx.set_li(0, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
                                            } else {
                                                { let t = 1; let o = cx.li(0); cx.set_li(0, (o & !(255 << 16)) | ((t & 255) << 16)); }
                                            }
                                        }
                                        { let _ = { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(199).add(100)).add(36)); let q1_ = cx.lptr(724); vcall(cx, 24, q0_, &[Varg::from(q1_)]) }; };
                                        cx.memset(cx.lptr(704), 0, 12);
                                    } else {
                                        cx.memcpy(cx.lptr(724), { let q0_ = cx.lpp(199); crate::ov_ident!(cx, q0_) }, 64);
                                        { let q0_ = cx.lptr(704); let q1_ = cx.ptr_at(cx.lpp(199).add(100)).add(4); let q2_ = cx.lptr(724); f_1aadb0(cx, q0_, q1_, q2_) };
                                    }
                                    { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(199).add(100)).add(32)); let q1_ = { let q0_ = (((cx.f(cx.lptr(772)) as f64) + (cx.f(cx.lptr(704)) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = { let q0_ = (((cx.f(cx.lptr(776)) as f64) + (cx.f(cx.lptr(708)) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q3_ = { let q0_ = (((cx.f(cx.lptr(780)) as f64) + (cx.f(cx.lptr(712)) as f64)) as f32); ov_coerce_int(cx, q0_) }; f_1a0360(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(199).add(100)).add(32)).add(16); let q1_ = cx.lptr(724); f_19e030(cx, q0_, q1_) };
                                    { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(199).add(100)).add(32)).add(28); let q1_ = cx.lptr(740); f_19e030(cx, q0_, q1_) };
                                    { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(199).add(100)).add(32)).add(40); let q1_ = cx.lptr(756); f_19e030(cx, q0_, q1_) };
                                    { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(199).add(100)).add(32)); let q1_ = cx.lpp(206).add(272); let q2_ = ((0.0f64 - (cx.f(cx.ptr_at(cx.lpp(206).add(3552))) as f64)) as f32); f_1a0390(cx, q0_, q1_, q2_) };
                                    if ((cx.i(cx.lpp(199).add(152)) & 32) != 0) {
                                        { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(199).add(100)).add(32)); f_3785b0(cx, q0_) };
                                    } else {
                                        { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(199).add(100)).add(32)); f_3781b0(cx, q0_) };
                                    }
                                    { let p = cx.lptr(688); let t = ({ let q0_ = cx.i(Ptr::gc(0).add(1776)); let q1_ = cx.i(cx.ptr_at(cx.lpp(206).add(3556)).add(4)); f_1ac600(cx, q0_, q1_) } as i64); cx.set_q(p, t); }
                                    'lp14: while (cx.li(692) < cx.li(688)) {
                                        { let t = cx.i(cx.ptr_at(cx.lpp(206).add(3556))).wrapping_add((132_i32).wrapping_mul(cx.li(692))); cx.set_lpp(171, Ptr::raw(t)); }
                                        { let t = a5.wrapping_sub(cx.li(832)); cx.set_li(676, t); }
                                        if ({ let q0_ = cx.lpp(198); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(832))); let q2_ = cx.lptr(676); let q3_ = cx.lpp(171); let q4_ = 0.0f32; f_378750(cx, q0_, q1_, q2_, q3_, q4_) } != 0) {
                                            { let t = ((cx.b(cx.ptr_at(cx.lpp(199).add(100)).add(29)) == 0) as i32); let o = cx.li(668); cx.set_li(668, (o & !(255 << 24)) | ((t & 255) << 24)); }
                                            { let q0_ = cx.lpp(199); f_1e4710(cx, q0_) };
                                            { let t = cx.i(cx.ptr_at(cx.lpp(199).add(100)).add(36)); cx.set_li(664, t); }
                                            if (cx.f(Ptr::gc(0).add(2012)) != 0.0f32) {
                                                { let t = 0; cx.set_li(660, t); }
                                                'lp15: loop {
                                                    if !(cx.li(660) < cx.li(676)) { break 'lp15; }
                                                    'blp15: {
                                                        { let q0_ = cx.lptr(644); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(660).wrapping_add(cx.li(832)))).add(1276); let q2_ = a4.add((1560_i32).wrapping_mul(cx.li(660).wrapping_add(cx.li(832)))).add(1332); let q3_ = (((cx.f(Ptr::gc(0).add(2012)) as f64) * (cx.f(a4.add((1560_i32).wrapping_mul(cx.li(660).wrapping_add(cx.li(832)))).add(1288)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                                        v34 = { let p = { let q0_ = cx.lptr(36); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                                        v31 = cx.i(Ptr::glob(0x77FD8C));
                                                        v28 = { let q0_ = cx.lptr(20); let q1_ = cx.li(644); let q2_ = cx.li(648); let q3_ = cx.li(652); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                        v10 = { let q0_ = cx.lptr(4); let q1_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(660).wrapping_add(cx.li(832)))).add(1276)); let q2_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(660).wrapping_add(cx.li(832)))).add(1280)); let q3_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(660).wrapping_add(cx.li(832)))).add(1284)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v10, v28, v31, v34);
                                                    }
                                                    let _ = cx.preinc_i(cx.lptr(660));
                                                }
                                            }
                                            if (((cx.li(668) >> 24) & 255) != 0) {
                                                v32 = cx.q(cx.lpp(198).add(8));
                                                v29 = cx.f(cx.lpp(198));
                                                v11 = ov_physics_manager(cx);
                                                { let t = ({ let q0_ = cx.ptr_at(v11.add(2836)); let q1_ = v12; let q2_ = cx.lptr(632); let q3_ = v29; let q4_ = f32::from_bits(v32 as u32); let q5_ = ((v32 >> 32) & 4294967295) as i32; f_385920(cx, q0_, q1_, q2_, q3_, q4_, q5_) } as i32); cx.set_li(624, t); }
                                                if (cx.li(632) == 0) {
                                                    let _ = ov_physics_manager(cx);
                                                    crate::ov_ignore!(cx, 1, "!CAxisSortedList__GetCollisionCandidates(): (2) The number of returned candidates could not fit into CAxisSortedList's preallocated array of %d, change the constructor parameters\n");
                                                }
                                                { let t = 0; cx.set_li(620, t); }
                                                'lp16: loop {
                                                    if !(cx.li(620) < cx.li(624)) { break 'lp16; }
                                                    'blp16: {
                                                        { let t = cx.ptr_at(Ptr::raw(cx.li(632).wrapping_add((4_i32).wrapping_mul(cx.li(620))))); cx.set_lpp(154, t); }
                                                        if ((((cx.lpp(154).bits() != 0) && (cx.i(cx.lpp(154).add(100)) != 0)) && (cx.b(cx.ptr_at(cx.lpp(154).add(100)).add(29)) == 0)) && (cx.i(cx.ptr_at(cx.lpp(154).add(100)).add(32)) != 0)) {
                                                            { let q0_ = cx.lptr(600); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(154).add(100)).add(32)); let q2_ = cx.lpp(198); f_19d920(cx, q0_, q1_, q2_) };
                                                            a2 = ({ let q0_ = cx.lptr(600); f_19d9f0(cx, q0_) } as f64);
                                                            if (((cx.f(cx.lpp(198).add(12)) as f64) + (cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(154).add(100)).add(32)).add(12)) as f64)) > a2) {
                                                                { let q0_ = cx.lpp(154); f_1e4710(cx, q0_) };
                                                                if (cx.f(Ptr::gc(0).add(1744)) != 0.0f32) {
                                                                    { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(154).add(100)).add(36)).add(32); let q1_ = cx.lpp(206).add(272); let q2_ = cx.f(Ptr::gc(0).add(1744)); f_1a0390(cx, q0_, q1_, q2_) };
                                                                    if (cx.f(Ptr::gc(0).add(1748)) != 0.0f32) {
                                                                        { let t = 1065353216; cx.set_li(592, t); }
                                                                        if (cx.f(Ptr::gc(0).add(1752)) > cx.f(cx.lpp(206).add(3784))) {
                                                                            { let p = cx.lptr(592); let t = (((cx.f(cx.lpp(206).add(3784)) as f64) / (cx.f(Ptr::gc(0).add(1752)) as f64)) as f32); cx.set_f(p, t); }
                                                                        }
                                                                        { let p = cx.lptr(592); let t = (((cx.f(cx.lptr(592)) as f64) * (cx.f(Ptr::gc(0).add(1748)) as f64)) as f32); cx.set_f(p, t); }
                                                                        v13 = cx.i(cx.ptr_at(cx.lpp(154).add(100)).add(36));
                                                                        let _ = f_1a0530(cx);
                                                                        v14 = ((a2 * (cx.f(cx.lptr(592)) as f64)) + (cx.f(Ptr::raw(v13.wrapping_add(48))) as f64));
                                                                        { let p = cx.ptr_at(cx.ptr_at(cx.lpp(154).add(100)).add(36)).add(48); let t = (v14 as f32); cx.set_f(p, t); }
                                                                        v15 = cx.i(cx.ptr_at(cx.lpp(154).add(100)).add(36));
                                                                        let _ = f_1a0530(cx);
                                                                        v16 = ((v14 * (cx.f(cx.lptr(592)) as f64)) + (cx.f(Ptr::raw(v15.wrapping_add(52))) as f64));
                                                                        { let p = cx.ptr_at(cx.ptr_at(cx.lpp(154).add(100)).add(36)).add(52); let t = (v16 as f32); cx.set_f(p, t); }
                                                                        v17 = cx.i(cx.ptr_at(cx.lpp(154).add(100)).add(36));
                                                                        let _ = f_1a0530(cx);
                                                                        a2 = ((v16 * (cx.f(cx.lptr(592)) as f64)) + (cx.f(Ptr::raw(v17.wrapping_add(56))) as f64));
                                                                        { let p = cx.ptr_at(cx.ptr_at(cx.lpp(154).add(100)).add(36)).add(56); let t = (a2 as f32); cx.set_f(p, t); }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    let _ = cx.preinc_i(cx.lptr(620));
                                                }
                                            }
                                            { let t = 0; cx.set_li(588, t); }
                                            if ((cx.i(Ptr::gc(0).add(1788)) != 0) && (cx.b(cx.lpp(206).add(4072)) != 0)) {
                                                v18 = { let q0_ = cx.lpp(206).add(240); ov_car_velocity(cx, q0_) };
                                                { let q0_ = cx.lptr(572); let q1_ = v18; f_19e030(cx, q0_, q1_) };
                                                v19 = ({ let q0_ = cx.lptr(572); f_1a0480(cx, q0_) } as f64);
                                                a2 = ((cx.f(Ptr::gc(0).add(1784)) as f64) * (cx.f(Ptr::gc(0).add(1784)) as f64));
                                                if (v19 > a2) {
                                                    { let t = 0; cx.set_li(564, t); }
                                                    'lp17: loop {
                                                        if !(cx.li(564) < cx.li(676)) { break 'lp17; }
                                                        'blp17: {
                                                            v20 = { let q0_ = cx.lpp(206).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                                                            v35 = ({ let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(564).wrapping_add(cx.li(832)))).add(1332); let q1_ = v20; f_19d9c0(cx, q0_, q1_) } as f32);
                                                            a2 = vsmath::fabs(v35);
                                                            if ((cx.f(Ptr::gc(0).add(1780)) as f64) > a2) {
                                                                { let t = 1; cx.set_li(588, t); }
                                                                break 'lp17;
                                                            }
                                                        }
                                                        let _ = cx.preinc_i(cx.lptr(564));
                                                    }
                                                    if (cx.li(588) != 0) {
                                                        { let q0_ = cx.lptr(528); let q1_ = cx.lpp(171); f_19e030(cx, q0_, q1_) };
                                                        { let q0_ = cx.lptr(488); let q1_ = cx.lpp(171); f_19e030(cx, q0_, q1_) };
                                                        v36 = cx.f(cx.ptr_at(cx.lpp(206).add(3552)));
                                                        v21 = { let q0_ = cx.lpp(206).add(240); ov_car_velocity(cx, q0_) };
                                                        { let q0_ = cx.lptr(468); let q1_ = v21; let q2_ = v36; f_19d970(cx, q0_, q1_, q2_) };
                                                        { let q0_ = cx.lptr(548); let q1_ = cx.lptr(528); let q2_ = cx.lptr(468); f_19d920(cx, q0_, q1_, q2_) };
                                                        { let p = cx.lptr(456); let t = 0i64; cx.set_q(p, t); }
                                                        'lp18: while (cx.li(460) < cx.i(Ptr::gc(0).add(1788))) {
                                                            { let q0_ = cx.lptr(508); let q1_ = cx.lptr(548); let q2_ = cx.lptr(528); f_1a38d0(cx, q0_, q1_, q2_) };
                                                            { let q0_ = cx.lptr(508); let q1_ = 0.5f32; f_19dab0(cx, q0_, q1_) };
                                                            { let q0_ = cx.lpp(171); let q1_ = cx.lptr(508); f_19e030(cx, q0_, q1_) };
                                                            cx.memcpy(cx.lptr(840), cx.lpp(171), { let q0_ = cx.lptr(840); crate::ov_zero_i!(cx, q0_) });
                                                            if ({ let q0_ = cx.lpp(198); let q1_ = cx.f(cx.lptr(840)); let q2_ = cx.li(844); let q3_ = cx.li(848); let q4_ = cx.li(852); let q5_ = cx.li(856); let q6_ = cx.li(860); let q7_ = cx.li(864); let q8_ = cx.li(868); let q9_ = cx.li(872); let q10_ = cx.li(876); let q11_ = cx.li(880); let q12_ = cx.li(884); let q13_ = cx.li(888); let q14_ = cx.li(892); let q15_ = cx.li(896); let q16_ = cx.li(900); f_3789c0(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_, q10_, q11_, q12_, q13_, q14_, q15_, q16_) } != 0) {
                                                                { let q0_ = cx.lptr(528); let q1_ = cx.lptr(508); f_19e030(cx, q0_, q1_) };
                                                                { let t = 1; cx.set_li(456, t); }
                                                            } else {
                                                                { let q0_ = cx.lptr(548); let q1_ = cx.lptr(508); f_19e030(cx, q0_, q1_) };
                                                                { let t = 0; cx.set_li(456, t); }
                                                            }
                                                            let _ = cx.preinc_i(cx.lptr(460));
                                                        }
                                                        if (cx.li(456) == 0) {
                                                            { let p = cx.lptr(452); let t = ((1.0f64 / (((cx.i(Ptr::gc(0).add(1788)) as f32) + 1.0f32) as f64)) as f32); cx.set_f(p, t); }
                                                            { let q0_ = cx.lpp(171); let q1_ = cx.lptr(468); let q2_ = cx.f(cx.lptr(452)); f_1a0390(cx, q0_, q1_, q2_) };
                                                        }
                                                        { let t = a5.wrapping_sub(cx.li(832)); cx.set_li(676, t); }
                                                        { let q0_ = cx.lpp(198); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(832))); let q2_ = cx.lptr(676); let q3_ = cx.lpp(171); let q4_ = 0.0f32; f_378750(cx, q0_, q1_, q2_, q3_, q4_) };
                                                        { let q0_ = cx.lptr(436); let q1_ = cx.lpp(171); let q2_ = cx.lptr(488); f_19d920(cx, q0_, q1_, q2_) };
                                                        { let t = 0; cx.set_li(428, t); }
                                                        'lp19: loop {
                                                            if !(cx.li(428) < cx.li(676)) { break 'lp19; }
                                                            'blp19: {
                                                                v22 = (1560_i32).wrapping_mul(cx.li(428).wrapping_add(cx.li(832)));
                                                                a2 = ((cx.f(a4.add(v22).add(1288)) as f64) - { let q0_ = a4.add(v22).add(1332); let q1_ = cx.lptr(436); f_19d9c0(cx, q0_, q1_) });
                                                                { let p = a4.add((1560_i32).wrapping_mul(cx.li(428).wrapping_add(cx.li(832)))).add(1288); let t = (a2 as f32); cx.set_f(p, t); }
                                                            }
                                                            let _ = cx.preinc_i(cx.lptr(428));
                                                        }
                                                    }
                                                }
                                            }
                                            { let t = 0; cx.set_li(424, t); }
                                            'lp20: loop {
                                                if !(cx.li(424) < cx.li(676)) { break 'lp20; }
                                                'blp20: {
                                                    { let p = a4.add((1560_i32).wrapping_mul(cx.li(424).wrapping_add(cx.li(832)))).add(4); let t = cx.li(664); cx.set_i(p, t); }
                                                    if (cx.i(a4.add((1560_i32).wrapping_mul(cx.li(424).wrapping_add(cx.li(832)))).add(4)) == 0) {
                                                        { let t = 1; cx.set_li(420, t); }
                                                        { let p = cx.ptr_at(cx.lpp(206).add(3552)); let t = 0; cx.set_i(p, t); }
                                                    }
                                                    { let p = a4.add((1560_i32).wrapping_mul(cx.li(424).wrapping_add(cx.li(832)))).add(8); let t = cx.lpp(206).add(240); cx.set_ptr(p, t); }
                                                    if (cx.i(cx.lpp(206).add(3740)) != 0) {
                                                        v23 = cx.i(Ptr::gc(0).add(1720));
                                                    } else {
                                                        v23 = cx.i(Ptr::gc(0).add(1716));
                                                    }
                                                    { let p = a4.add((1560_i32).wrapping_mul(cx.li(424).wrapping_add(cx.li(832)))).add(1540); let t = v23; cx.set_i(p, t); }
                                                    { let p = a4.add((1560_i32).wrapping_mul(cx.li(424).wrapping_add(cx.li(832)))).add(1544); let t = cx.i(Ptr::gc(0).add(1724)); cx.set_i(p, t); }
                                                    { let p = a4.add((1560_i32).wrapping_mul(cx.li(424).wrapping_add(cx.li(832)))).add(216); let t = -1; cx.set_i(p, t); }
                                                    { let p = a4.add((1560_i32).wrapping_mul(cx.li(424).wrapping_add(cx.li(832)))).add(220); let t = cx.li(828); cx.set_i(p, t); }
                                                    { let p = a4.add((1560_i32).wrapping_mul(cx.li(424).wrapping_add(cx.li(832)))).add(1272); let t = cx.i(Ptr::gc(0).add(1732)); cx.set_i(p, t); }
                                                    { let p = a4.add((1560_i32).wrapping_mul(cx.li(424).wrapping_add(cx.li(832)))).add(1268); let t = cx.i(Ptr::gc(0).add(1728)); cx.set_i(p, t); }
                                                }
                                                let _ = cx.preinc_i(cx.lptr(424));
                                            }
                                            if (cx.li(676) != 0) {
                                                { let p = cx.lpp(206).add(4856); let t = 0; cx.set_i(p, t); }
                                                if ({ let q0_ = cx.ptr_at(cx.lpp(199).add(100)); f_1b41f0(cx, q0_) } != 0) {
                                                    { let p = cx.ptr_at(cx.ptr_at(cx.lpp(199).add(100)).add(36)).add(536); let t = 0; cx.set_i(p, t); }
                                                }
                                                if ((cx.i(cx.lpp(199).add(152)) & 16) != 0) {
                                                    { let t = cx.ptr_at(cx.ptr_at(cx.lpp(199).add(100)).add(32)); cx.set_lpp(104, t); }
                                                    if (cx.b(cx.lpp(104).add(53)).wrapping_add(cx.b(cx.lpp(104).add(54))).wrapping_add(cx.b(cx.lpp(104).add(52))) > 1) {
                                                        v24 = { let q0_ = cx.lpp(206).add(240); ov_car_velocity(cx, q0_) };
                                                        a2 = { let q0_ = a4.add((1560_i32).wrapping_mul(cx.li(832))).add(1332); let q1_ = v24; f_19d9c0(cx, q0_, q1_) };
                                                        v37 = (a2 as f32);
                                                        { let q0_ = cx.lptr(400); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(832))).add(1332); let q2_ = v37; f_19d970(cx, q0_, q1_, q2_) };
                                                        { let q0_ = { let p = ov_physics_manager(cx).add(24); cx.ptr_at(p) }; let q1_ = cx.lpp(199); let q2_ = cx.lptr(400); f_1debb0(cx, q0_, q1_, q2_) };
                                                        { let t = 0; cx.set_li(676, t); }
                                                        { let t = 0; cx.set_li(832, t); }
                                                        continue 'b_LABEL_15;
                                                    }
                                                }
                                            }
                                            { let t = cx.li(832).wrapping_add(cx.li(676)); cx.set_li(832, t); }
                                        }
                                        let _ = cx.preinc_i(cx.lptr(692));
                                    }
                                    if (cx.li(832) != 0) {
                                        { let q0_ = cx.lptr(216); f_1b42f0(cx, q0_) };
                                        { let t = 1023969417; cx.set_li(216, t); }
                                        { let t = cx.i(Ptr::gc(0).add(1152)); cx.set_li(292, t); }
                                        { let t = cx.i(Ptr::gc(0).add(1156)); cx.set_li(224, t); }
                                        { let t = cx.i(Ptr::gc(0).add(1160)); cx.set_li(220, t); }
                                        { let t = cx.i(Ptr::gc(0).add(1164)); cx.set_li(296, t); }
                                        { let t = cx.i(Ptr::gc(0).add(1888)); cx.set_li(340, t); }
                                        { let p = cx.lptr(304); let t = cx.q(Ptr::gc(0).add(1172)); cx.set_q(p, t); }
                                        { let t = cx.i(Ptr::gc(0).add(1180)); cx.set_li(300, t); }
                                        { let q0_ = cx.lptr(252); let q1_ = a6; f_19e030(cx, q0_, q1_) };
                                        { let q0_ = cx.lptr(228); let q1_ = Ptr::gc(0); f_19e030(cx, q0_, q1_) };
                                        { let q0_ = a4; let q1_ = cx.li(832); let q2_ = Ptr::NULL; let q3_ = 0; let q4_ = Ptr::NULL; let q5_ = 0; let q6_ = cx.lptr(216); f_389630(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
                                        if (cx.f(Ptr::gc(0).add(1736)) != 0.0f32) {
                                            a2 = ({ let q0_ = cx.lpp(206).add(272); f_19d9f0(cx, q0_) } as f64);
                                            { let p = cx.lptr(204); let t = (a2 as f32); cx.set_f(p, t); }
                                            { let t = 0; cx.set_li(208, t); }
                                            'lp21: loop {
                                                if !(cx.li(208) < cx.li(832)) { break 'lp21; }
                                                'blp21: {
                                                    a2 = ({ let q0_ = cx.ptr_at(a4.add((1560_i32).wrapping_mul(cx.li(208))).add(4)).add(32); f_19d9f0(cx, q0_) } as f64);
                                                    { let p = cx.lptr(200); let t = (a2 as f32); cx.set_f(p, t); }
                                                    if (cx.f(cx.lptr(200)) > (cx.f(cx.lptr(204)) * cx.f(Ptr::gc(0).add(1740)))) {
                                                        { let q0_ = cx.ptr_at(a4.add((1560_i32).wrapping_mul(cx.li(208))).add(4)).add(32); let q1_ = cx.lpp(206).add(272); let q2_ = cx.f(Ptr::gc(0).add(1736)); f_1a0390(cx, q0_, q1_, q2_) };
                                                    }
                                                }
                                                let _ = cx.preinc_i(cx.lptr(208));
                                            }
                                        }
                                        { let t = 0; cx.set_li(196, t); }
                                        'lp22: loop {
                                            if !(cx.li(196) < cx.li(832)) { break 'lp22; }
                                            'blp22: {
                                                if (cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(220)) != -1) {
                                                    a2 = vsmath::fabs(cx.f(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(236)));
                                                    { let p = cx.lptr(192); let t = (a2 as f32); cx.set_f(p, t); }
                                                    if (cx.f(cx.lptr(192)) > 0.0f32) {
                                                        if (({ let q0_ = cx.lpp(199); f_1e5ca0(cx, q0_) } != 0) && (cx.i(Ptr::glob(0x750358)) > 3)) {
                                                            { let q0_ = cx.lptr(108); f_238b0(cx, q0_) };
                                                            { let t = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(220)); cx.set_li(108, t); }
                                                            { let t = -1; cx.set_li(112, t); }
                                                            { let t = cx.li(192); cx.set_li(148, t); }
                                                            { let q0_ = cx.lptr(116); let q1_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(1276)); let q2_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(1280)); let q3_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(1284)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                                            { let q0_ = cx.lptr(132); let q1_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(1332)); let q2_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(1336)); let q3_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(1340)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                                            { let t = 0; cx.set_li(152, t); }
                                                            { let t = 1106247680; cx.set_li(180, t); }
                                                            { let t = -1; let o = cx.li(184); cx.set_li(184, (o & !(255 << 0)) | ((t & 255) << 0)); }
                                                            { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(209)); let q1_ = a2; let q2_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(220)); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.i(p) }.wrapping_add(240); cx.set_li(156, t); }
                                                            { let t = 0; cx.set_li(160, t); }
                                                            { let q0_ = cx.lpp(209); let q1_ = a4.add((1560_i32).wrapping_mul(cx.li(196))); let q2_ = cx.lptr(164); f_1b39b0(cx, q0_, q1_, q2_) };
                                                            { let p = Ptr::glob(0x750358); let t = 0; cx.set_i(p, t); }
                                                            { let q0_ = Ptr::glob(0x6FAA34); let q1_ = 0; let q2_ = cx.lptr(108); ov_message(cx, q0_, q1_, q2_) };
                                                        } else {
                                                            { let q0_ = cx.lptr(48); f_1b4210(cx, q0_) };
                                                            { let t = cx.i(cx.ptr_at(cx.lpp(199).add(100)).add(36)); cx.set_li(40, t); }
                                                            { let t = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(220)); cx.set_li(48, t); }
                                                            { let t = cx.lpp(199); cx.set_lpp(13, t); }
                                                            { let t = cx.li(40); cx.set_li(92, t); }
                                                            { let t = { let p = { let q0_ = cx.ptr_at(cx.lpp(209)); let q1_ = a2; let q2_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(220)); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.i(p) }.wrapping_add(240); cx.set_li(96, t); }
                                                            { let t = cx.li(192); cx.set_li(88, t); }
                                                            { let q0_ = cx.lptr(56); let q1_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(1276)); let q2_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(1280)); let q3_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(1284)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                                            { let q0_ = cx.lptr(72); let q1_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(1332)); let q2_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(1336)); let q3_ = cx.i(a4.add((1560_i32).wrapping_mul(cx.li(196))).add(1340)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
                                                            { let q0_ = Ptr::glob(0x6FAA34); let q1_ = 2; let q2_ = cx.lptr(48); ov_message(cx, q0_, q1_, q2_) };
                                                        }
                                                    }
                                                }
                                            }
                                            let _ = cx.preinc_i(cx.lptr(196));
                                        }
                                    }
                                    { let t = 0; cx.set_li(832, t); }
                                }
                            }
                            let _ = cx.preinc_i(cx.lptr(800));
                        }
                        #[allow(unreachable_code)]
                        break;
                    }
                    if ((cx.li(808) != 0) && (cx.i(cx.lpp(201)) >= 3)) {
                        { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(206).add(3556))); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(206).add(3556))).add(28); let q2_ = cx.f(Ptr::gc(0).add(1772)); f_1a0390(cx, q0_, q1_, q2_) };
                        { let p = cx.ptr_at(cx.ptr_at(cx.lpp(206).add(3556))).add(56); let t = (((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(206).add(3556))).add(56)) as f64) - (cx.f(Ptr::gc(0).add(1768)) as f64)) as f32); cx.set_f(p, t); }
                    }
                }
            }
        }
        let _ = cx.preinc_i(cx.lptr(828));
    }
    v33 = ({ let q0_ = a3; let q1_ = cx.li(828); crate::ov_zero_i!(cx, q0_, q1_) } as i64);
    return v33;
}

/// PhysicsManager__1AC600 0x001AC600
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1ac600(cx: &mut Cx, a1: i32, a2: i32) -> i32 {
    if (a1 >= a2) {
        return a2;
    } else {
        return a1;
    }
    #[allow(unreachable_code)]
    0
}

/// PhysicsManager__1AC680 0x001AC680
pub fn f_1ac680(cx: &mut Cx, a1: i32, a2: i32, a3: Ptr, a4: i32, a5: Ptr) -> i64 {
    cx.push_frame(1516);
    let r = f_1ac680_body(cx, a1, a2, a3, a4, a5);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1ac680_body(cx: &mut Cx, a1: i32, a2: i32, a3: Ptr, a4: i32, a5: Ptr) -> i64 {
    let mut v5 = 0i64;
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
    let mut v18 = Ptr::NULL;
    let mut v19 = Ptr::NULL;
    let mut v20 = Ptr::NULL;
    let mut v21 = Ptr::NULL;
    let mut v22 = Ptr::NULL;
    let mut v23 = Ptr::NULL;
    let mut v24 = Ptr::NULL;
    let mut v25 = Ptr::NULL;
    let mut v26 = 0i32;
    let mut v27 = 0i32;
    let mut v28 = 0i32;
    let mut v29 = 0i32;
    let mut v30 = 0i32;
    let mut v31 = 0i32;
    let mut v32 = 0i32;
    let mut v33 = 0i32;
    let mut v34 = Ptr::NULL;
    let mut v35 = 0i32;
    let mut v36 = Ptr::NULL;
    let mut v37 = Ptr::NULL;
    let mut v39 = Ptr::NULL;
    let mut v40 = Ptr::NULL;
    let mut v41 = Ptr::NULL;
    let mut v42 = Ptr::NULL;
    let mut v43 = Ptr::NULL;
    let mut v44 = Ptr::NULL;
    let mut v45 = Ptr::NULL;
    let mut v46 = Ptr::NULL;
    let mut v47 = Ptr::NULL;
    let mut v48 = Ptr::NULL;
    let mut v49 = Ptr::NULL;
    let mut v50 = Ptr::NULL;
    let mut v51 = Ptr::NULL;
    let mut v52 = Ptr::NULL;
    let mut v53 = Ptr::NULL;
    let mut v54 = Ptr::NULL;
    let mut v55 = Ptr::NULL;
    let mut v56 = Ptr::NULL;
    let mut v57 = Ptr::NULL;
    let mut v58 = Ptr::NULL;
    let mut v59 = Ptr::NULL;
    let mut v60 = Ptr::NULL;
    let mut v61 = 0i32;
    let mut v62 = 0i32;
    let mut v63 = 0i32;
    let mut v64 = 0i32;
    let mut v65 = 0i32;
    let mut v66 = 0i32;
    let mut v67 = 0i32;
    let mut v68 = 0i32;
    let mut v69 = 0i32;
    let mut v70 = 0i32;
    let mut v71 = 0i32;
    let mut v72 = 0i32;
    let mut v73 = 0i32;
    let mut v74 = 0i32;
    let mut v75 = 0i32;
    let mut v76 = 0i32;
    let mut v77 = 0i32;
    let mut v78 = 0i32;
    let mut v79 = 0i32;
    let mut v80 = 0i32;
    let mut v81 = 0i32;
    let mut v82 = 0i32;
    let mut v83 = 0i64;
    let mut v84 = 0i32;
    let mut v85 = 0i32;
    let mut v86 = 0i32;
    let mut v87 = 0i32;
    let mut v88 = 0i32;
    let mut v89 = 0i32;
    let mut v90 = 0i32;
    let mut v91 = 0i32;
    let mut v92 = 0i32;
    let mut v93 = 0i32;
    let mut v94 = 0i32;
    let mut v95 = 0i32;
    let mut v96 = 0i32;
    let mut v97 = 0i32;
    let mut v98 = 0i32;
    let mut v99 = 0i32;
    let mut v100 = 0i32;
    let mut v101 = 0i32;
    let mut v102 = 0i32;
    let mut v103 = 0i32;
    let mut v104 = 0i32;
    let mut v105 = 0i32;
    let mut v106 = 0i32;
    let mut v107 = 0i32;
    let mut v108 = 0i32;
    { let t = a1; cx.set_lpp(378, Ptr::raw(t)); }
    { let t = 0; cx.set_li(1508, t); }
    v5 = (v5 & !0xFFFF_FFFFi64) | ((ov_physics_manager(cx).bits()) as i32 as u32 as i64);
    if (cx.i(Ptr::raw(((v5 + 2836i64) as i32))) != 0) {
        { let t = 0; cx.set_li(1504, t); }
        'lp11: loop {
            if !(cx.li(1504) < 30) { break 'lp11; }
            'blp11: {
                v5 = (v5 & 0xFFFF_FFFFi64) | (((cx.lpp(378).bits()) as u32 as i64) << 32);
                if (cx.i(cx.lpp(378).add((712_i32).wrapping_mul(cx.li(1504))).add(3384)) != 0) {
                    { let t = cx.ptr_at(cx.lpp(378).add((712_i32).wrapping_mul(cx.li(1504))).add(3384)); cx.set_lpp(375, t); }
                    { let t = cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)); cx.set_lpp(374, t); }
                    { let t = cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(36)); cx.set_lpp(373, t); }
                    v5 = (v5 & 0xFFFF_FFFFi64) | ((((({ let q0_ = cx.ptr_at(cx.lpp(378).add(32)); let q1_ = cx.lptr(1484); let q2_ = cx.lptr(1484); let q3_ = cx.lptr(1472); let q4_ = cx.lpp(374); let q5_ = 0; let q6_ = Ptr::NULL; ov_qt_find_leaf(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) } >> 32i64) as i32)) as u32 as i64) << 32);
                    if (cx.li(1472) != 0) {
                        { let t = 0; cx.set_li(1460, t); }
                        { let t = 0; cx.set_li(1452, t); }
                        if (cx.i(Ptr::gc(0).add(2016)) != 0) {
                            { let t = 0; cx.set_li(1444, t); }
                            'lp12: loop {
                                if !(cx.li(1444) < cx.b(cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(52))) { break 'lp12; }
                                'blp12: {
                                    { let t = cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(60)).wrapping_add((116_i32).wrapping_mul(cx.li(1444))); cx.set_lpp(360, Ptr::raw(t)); }
                                    { let q0_ = cx.lpp(360); let q1_ = cx.lptr(1340).bits(); let q2_ = cx.lptr(1340); f_37a7c0(cx, q0_, q1_, q2_) };
                                    v84 = { let p = { let q0_ = cx.lptr(792); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v61 = cx.i(Ptr::glob(0x77FDA8));
                                    v39 = { let q0_ = cx.lptr(776); let q1_ = cx.li(1352); let q2_ = cx.li(1356); let q3_ = cx.li(1360); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v6 = { let q0_ = cx.lptr(760); let q1_ = cx.li(1340); let q2_ = cx.li(1344); let q3_ = cx.li(1348); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v6, v39, v61, v84);
                                    v85 = { let p = { let q0_ = cx.lptr(756); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v62 = cx.i(Ptr::glob(0x77FDA8));
                                    v40 = { let q0_ = cx.lptr(740); let q1_ = cx.li(1364); let q2_ = cx.li(1368); let q3_ = cx.li(1372); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v7 = { let q0_ = cx.lptr(724); let q1_ = cx.li(1352); let q2_ = cx.li(1356); let q3_ = cx.li(1360); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v7, v40, v62, v85);
                                    v86 = { let p = { let q0_ = cx.lptr(720); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v63 = cx.i(Ptr::glob(0x77FDA8));
                                    v41 = { let q0_ = cx.lptr(704); let q1_ = cx.li(1376); let q2_ = cx.li(1380); let q3_ = cx.li(1384); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v8 = { let q0_ = cx.lptr(688); let q1_ = cx.li(1364); let q2_ = cx.li(1368); let q3_ = cx.li(1372); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v8, v41, v63, v86);
                                    v87 = { let p = { let q0_ = cx.lptr(684); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v64 = cx.i(Ptr::glob(0x77FDA8));
                                    v42 = { let q0_ = cx.lptr(668); let q1_ = cx.li(1340); let q2_ = cx.li(1344); let q3_ = cx.li(1348); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v9 = { let q0_ = cx.lptr(652); let q1_ = cx.li(1376); let q2_ = cx.li(1380); let q3_ = cx.li(1384); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v9, v42, v64, v87);
                                    v88 = { let p = { let q0_ = cx.lptr(648); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v65 = cx.i(Ptr::glob(0x77FDA8));
                                    v43 = { let q0_ = cx.lptr(632); let q1_ = cx.li(1400); let q2_ = cx.li(1404); let q3_ = cx.li(1408); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v10 = { let q0_ = cx.lptr(616); let q1_ = cx.li(1388); let q2_ = cx.li(1392); let q3_ = cx.li(1396); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v10, v43, v65, v88);
                                    v89 = { let p = { let q0_ = cx.lptr(612); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v66 = cx.i(Ptr::glob(0x77FDA8));
                                    v44 = { let q0_ = cx.lptr(596); let q1_ = cx.li(1412); let q2_ = cx.li(1416); let q3_ = cx.li(1420); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v11 = { let q0_ = cx.lptr(580); let q1_ = cx.li(1400); let q2_ = cx.li(1404); let q3_ = cx.li(1408); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v11, v44, v66, v89);
                                    v90 = { let p = { let q0_ = cx.lptr(576); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v67 = cx.i(Ptr::glob(0x77FDA8));
                                    v45 = { let q0_ = cx.lptr(560); let q1_ = cx.li(1424); let q2_ = cx.li(1428); let q3_ = cx.li(1432); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v12 = { let q0_ = cx.lptr(544); let q1_ = cx.li(1412); let q2_ = cx.li(1416); let q3_ = cx.li(1420); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v12, v45, v67, v90);
                                    v91 = { let p = { let q0_ = cx.lptr(540); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v68 = cx.i(Ptr::glob(0x77FDA8));
                                    v46 = { let q0_ = cx.lptr(524); let q1_ = cx.li(1388); let q2_ = cx.li(1392); let q3_ = cx.li(1396); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v13 = { let q0_ = cx.lptr(508); let q1_ = cx.li(1424); let q2_ = cx.li(1428); let q3_ = cx.li(1432); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v13, v46, v68, v91);
                                    v92 = { let p = { let q0_ = cx.lptr(504); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v69 = cx.i(Ptr::glob(0x77FDA8));
                                    v47 = { let q0_ = cx.lptr(488); let q1_ = cx.li(1388); let q2_ = cx.li(1392); let q3_ = cx.li(1396); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v14 = { let q0_ = cx.lptr(472); let q1_ = cx.li(1340); let q2_ = cx.li(1344); let q3_ = cx.li(1348); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v14, v47, v69, v92);
                                    v93 = { let p = { let q0_ = cx.lptr(468); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v70 = cx.i(Ptr::glob(0x77FDA8));
                                    v48 = { let q0_ = cx.lptr(452); let q1_ = cx.li(1400); let q2_ = cx.li(1404); let q3_ = cx.li(1408); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v15 = { let q0_ = cx.lptr(436); let q1_ = cx.li(1352); let q2_ = cx.li(1356); let q3_ = cx.li(1360); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v15, v48, v70, v93);
                                    v94 = { let p = { let q0_ = cx.lptr(432); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v71 = cx.i(Ptr::glob(0x77FDA8));
                                    v49 = { let q0_ = cx.lptr(416); let q1_ = cx.li(1412); let q2_ = cx.li(1416); let q3_ = cx.li(1420); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v16 = { let q0_ = cx.lptr(400); let q1_ = cx.li(1364); let q2_ = cx.li(1368); let q3_ = cx.li(1372); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v16, v49, v71, v94);
                                    v95 = { let p = { let q0_ = cx.lptr(396); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v72 = cx.i(Ptr::glob(0x77FDA8));
                                    v50 = { let q0_ = cx.lptr(380); let q1_ = cx.li(1424); let q2_ = cx.li(1428); let q3_ = cx.li(1432); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v17 = { let q0_ = cx.lptr(364); let q1_ = cx.li(1376); let q2_ = cx.li(1380); let q3_ = cx.li(1384); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v17, v50, v72, v95);
                                }
                                let _ = cx.preinc_i(cx.lptr(1444));
                            }
                            { let t = 0; cx.set_li(1332, t); }
                            'lp13: loop {
                                if !(cx.li(1332) < cx.b(cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(53))) { break 'lp13; }
                                'blp13: {
                                    { let t = cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(64)).wrapping_add((36_i32).wrapping_mul(cx.li(1332))); cx.set_lpp(332, Ptr::raw(t)); }
                                    { let q0_ = cx.lptr(1252); let q1_ = cx.lpp(332); let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(16); let q3_ = cx.f(cx.lpp(332).add(16)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1264); let q1_ = cx.lpp(332); let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(16); let q3_ = ((0.0f64 - (cx.f(cx.lpp(332).add(16)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1276); let q1_ = cx.lpp(332); let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(28); let q3_ = cx.f(cx.lpp(332).add(16)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1288); let q1_ = cx.lpp(332); let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(28); let q3_ = ((0.0f64 - (cx.f(cx.lpp(332).add(16)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1300); let q1_ = cx.lpp(332); let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(40); let q3_ = cx.f(cx.lpp(332).add(16)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1312); let q1_ = cx.lpp(332); let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(40); let q3_ = ((0.0f64 - (cx.f(cx.lpp(332).add(16)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    v96 = { let p = { let q0_ = cx.lptr(360); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v73 = cx.i(Ptr::glob(0x77FDA8));
                                    v51 = { let q0_ = cx.lptr(344); let q1_ = cx.li(1264); let q2_ = cx.li(1268); let q3_ = cx.li(1272); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v18 = { let q0_ = cx.lptr(328); let q1_ = cx.li(1252); let q2_ = cx.li(1256); let q3_ = cx.li(1260); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v18, v51, v73, v96);
                                    v97 = { let p = { let q0_ = cx.lptr(324); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v74 = cx.i(Ptr::glob(0x77FDA8));
                                    v52 = { let q0_ = cx.lptr(308); let q1_ = cx.li(1288); let q2_ = cx.li(1292); let q3_ = cx.li(1296); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v19 = { let q0_ = cx.lptr(292); let q1_ = cx.li(1276); let q2_ = cx.li(1280); let q3_ = cx.li(1284); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v19, v52, v74, v97);
                                    v98 = { let p = { let q0_ = cx.lptr(288); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v75 = cx.i(Ptr::glob(0x77FDA8));
                                    v53 = { let q0_ = cx.lptr(272); let q1_ = cx.li(1312); let q2_ = cx.li(1316); let q3_ = cx.li(1320); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v20 = { let q0_ = cx.lptr(256); let q1_ = cx.li(1300); let q2_ = cx.li(1304); let q3_ = cx.li(1308); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v20, v53, v75, v98);
                                }
                                let _ = cx.preinc_i(cx.lptr(1332));
                            }
                            { let t = 0; cx.set_li(1244, t); }
                            'lp14: loop {
                                if !(cx.li(1244) < cx.b(cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(54))) { break 'lp14; }
                                'blp14: {
                                    { let t = cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(68)).wrapping_add((cx.li(1244) << 6)); cx.set_lpp(310, Ptr::raw(t)); }
                                    { let q0_ = cx.lptr(1164); let q1_ = cx.lpp(310); let q2_ = cx.lpp(310).add(16); let q3_ = cx.f(cx.lpp(310).add(32)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1176); let q1_ = cx.lpp(310); let q2_ = cx.lpp(310).add(16); let q3_ = ((0.0f64 - (cx.f(cx.lpp(310).add(32)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    if (vsmath::fabs(cx.f(cx.lpp(310).add(56))) <= 0.5f64) {
                                        if (vsmath::fabs(cx.f(cx.lpp(310).add(52))) <= 0.5f64) {
                                            { let q0_ = cx.lptr(1132); let q1_ = cx.lpp(374).add(28); f_19e030(cx, q0_, q1_) };
                                        } else {
                                            { let q0_ = cx.lptr(1132); let q1_ = cx.lpp(374).add(16); f_19e030(cx, q0_, q1_) };
                                        }
                                        { let q0_ = cx.lptr(1144); let q1_ = cx.lpp(374).add(40); f_19e030(cx, q0_, q1_) };
                                    } else {
                                        { let q0_ = cx.lptr(1132); let q1_ = cx.lpp(374).add(16); f_19e030(cx, q0_, q1_) };
                                        { let q0_ = cx.lptr(1144); let q1_ = cx.lpp(374).add(28); f_19e030(cx, q0_, q1_) };
                                    }
                                    { let q0_ = cx.lptr(1188); let q1_ = cx.lptr(1164); let q2_ = cx.lptr(1132); let q3_ = cx.f(cx.lpp(310).add(28)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1200); let q1_ = cx.lptr(1164); let q2_ = cx.lptr(1132); let q3_ = ((0.0f64 - (cx.f(cx.lpp(310).add(28)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1212); let q1_ = cx.lptr(1164); let q2_ = cx.lptr(1144); let q3_ = cx.f(cx.lpp(310).add(28)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1224); let q1_ = cx.lptr(1164); let q2_ = cx.lptr(1144); let q3_ = ((0.0f64 - (cx.f(cx.lpp(310).add(28)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    v99 = { let p = { let q0_ = cx.lptr(252); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v76 = cx.i(Ptr::glob(0x77FDA8));
                                    v54 = { let q0_ = cx.lptr(236); let q1_ = cx.li(1200); let q2_ = cx.li(1204); let q3_ = cx.li(1208); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v21 = { let q0_ = cx.lptr(220); let q1_ = cx.li(1188); let q2_ = cx.li(1192); let q3_ = cx.li(1196); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v21, v54, v76, v99);
                                    v100 = { let p = { let q0_ = cx.lptr(216); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v77 = cx.i(Ptr::glob(0x77FDA8));
                                    v55 = { let q0_ = cx.lptr(200); let q1_ = cx.li(1224); let q2_ = cx.li(1228); let q3_ = cx.li(1232); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v22 = { let q0_ = cx.lptr(184); let q1_ = cx.li(1212); let q2_ = cx.li(1216); let q3_ = cx.li(1220); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v22, v55, v77, v100);
                                    { let q0_ = cx.lptr(1188); let q1_ = cx.lptr(1176); let q2_ = cx.lptr(1132); let q3_ = cx.f(cx.lpp(310).add(28)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1200); let q1_ = cx.lptr(1176); let q2_ = cx.lptr(1132); let q3_ = ((0.0f64 - (cx.f(cx.lpp(310).add(28)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1212); let q1_ = cx.lptr(1176); let q2_ = cx.lptr(1144); let q3_ = cx.f(cx.lpp(310).add(28)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1224); let q1_ = cx.lptr(1176); let q2_ = cx.lptr(1144); let q3_ = ((0.0f64 - (cx.f(cx.lpp(310).add(28)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    v101 = { let p = { let q0_ = cx.lptr(180); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v78 = cx.i(Ptr::glob(0x77FDA8));
                                    v56 = { let q0_ = cx.lptr(164); let q1_ = cx.li(1200); let q2_ = cx.li(1204); let q3_ = cx.li(1208); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v23 = { let q0_ = cx.lptr(148); let q1_ = cx.li(1188); let q2_ = cx.li(1192); let q3_ = cx.li(1196); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v23, v56, v78, v101);
                                    v102 = { let p = { let q0_ = cx.lptr(144); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v79 = cx.i(Ptr::glob(0x77FDA8));
                                    v57 = { let q0_ = cx.lptr(128); let q1_ = cx.li(1224); let q2_ = cx.li(1228); let q3_ = cx.li(1232); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v24 = { let q0_ = cx.lptr(112); let q1_ = cx.li(1212); let q2_ = cx.li(1216); let q3_ = cx.li(1220); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v24, v57, v79, v102);
                                    { let q0_ = cx.lptr(1164); let q1_ = cx.lptr(1164); let q2_ = cx.lpp(310).add(16); let q3_ = cx.f(cx.lpp(310).add(28)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    { let q0_ = cx.lptr(1176); let q1_ = cx.lptr(1176); let q2_ = cx.lpp(310).add(16); let q3_ = ((0.0f64 - (cx.f(cx.lpp(310).add(28)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                    v103 = { let p = { let q0_ = cx.lptr(108); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                    v80 = cx.i(Ptr::glob(0x77FDA8));
                                    v58 = { let q0_ = cx.lptr(92); let q1_ = cx.li(1176); let q2_ = cx.li(1180); let q3_ = cx.li(1184); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v25 = { let q0_ = cx.lptr(76); let q1_ = cx.li(1164); let q2_ = cx.li(1168); let q3_ = cx.li(1172); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v25, v58, v80, v103);
                                }
                                let _ = cx.preinc_i(cx.lptr(1244));
                            }
                        }
                        { let t = 0; cx.set_li(1124, t); }
                        { let q0_ = cx.lpp(373); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)).add(16); f_36a090(cx, q0_, q1_) };
                        { let _ = { let q0_ = cx.lpp(373); let q1_ = cx.i(cx.ptr_at(cx.lpp(375).add(100)).add(32)); vcall(cx, 28, q0_, &[Varg::from(q1_)]) }; };
                        if ((cx.i(cx.lpp(375).add(152)) & 32) != 0) {
                            { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)); f_3785b0(cx, q0_) };
                        } else {
                            { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(32)); f_3781b0(cx, q0_) };
                        }
                        { let t = 0; cx.set_li(1452, t); }
                        { let t = 0; cx.set_li(1448, t); }
                        'lp15: loop {
                            'blp15: {
                                v5 = (v5 & 0xFFFF_FFFFi64) | (((cx.li(1448)) as u32 as i64) << 32);
                                if (cx.li(1448) >= cx.li(1472)) {
                                    break 'lp15;
                                }
                                v26 = { let q0_ = cx.ptr_at(cx.lpp(378).add(32)); let q1_ = cx.li(1448).wrapping_add(cx.li(1484)); ov_qt_face_index(cx, q0_, q1_) };
                                { let t = { let q0_ = cx.ptr_at(cx.lpp(378).add(28)); let q1_ = v26; let q2_ = 0; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(280, t); }
                                v27 = { let q0_ = cx.ptr_at(cx.lpp(378).add(32)); let q1_ = cx.li(1448).wrapping_add(cx.li(1484)); ov_qt_face_index(cx, q0_, q1_) };
                                { let t = { let q0_ = cx.ptr_at(cx.lpp(378).add(28)); let q1_ = v27; let q2_ = 1; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(279, t); }
                                v28 = { let q0_ = cx.ptr_at(cx.lpp(378).add(32)); let q1_ = cx.li(1448).wrapping_add(cx.li(1484)); ov_qt_face_index(cx, q0_, q1_) };
                                { let t = { let q0_ = cx.ptr_at(cx.lpp(378).add(28)); let q1_ = v28; let q2_ = 2; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(278, t); }
                                v29 = { let q0_ = cx.lpp(374); let q1_ = cx.lpp(280); let q2_ = cx.lpp(279); let q3_ = cx.lpp(278); f_378ac0(cx, q0_, q1_, q2_, q3_) };
                                { let t = v29; cx.set_li(1108, t); }
                                if ((v29 == 0) && (cx.f(Ptr::gc(0).add(1808)) != 0.0f32)) {
                                    { let t = (((({ let q0_ = cx.lptr(1100); let q1_ = cx.lptr(1088); let q2_ = cx.lptr(1076); let q3_ = cx.lpp(280); let q4_ = cx.lpp(279); let q5_ = cx.lpp(278); let q6_ = cx.lpp(374); let q7_ = Ptr::glob(0x688990); ov_ray_triangle(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_) } != 0) && (cx.f(cx.lptr(1100)) > 0.0f32)) && (cx.f(Ptr::gc(0).add(1808)) > cx.f(cx.lptr(1100)))) as i32); cx.set_li(0, t); }
                                    { let t = cx.li(0); cx.set_li(1108, t); }
                                    if (cx.li(0) != 0) {
                                        { let t = 1; cx.set_li(1068, t); }
                                    }
                                }
                                if (cx.li(1108) != 0) {
                                    { let q0_ = cx.lptr(1052); let q1_ = cx.lpp(280); let q2_ = cx.lpp(374); f_19d920(cx, q0_, q1_, q2_) };
                                    let _ = cx.preinc_i(cx.lptr(1452));
                                    { let p = Ptr::glob(0x75AA38).add(4 * (cx.li(1448))); let t = 1; cx.set_i(p, t); }
                                } else {
                                    { let p = Ptr::glob(0x75AA38).add(4 * (cx.li(1448))); let t = 0; cx.set_i(p, t); }
                                }
                            }
                            let _ = cx.preinc_i(cx.lptr(1448));
                        }
                        if (cx.li(1452) != 0) {
                            { let t = 0; cx.set_li(1124, t); }
                            { let t = 0; cx.set_li(1044, t); }
                            { let t = 0; cx.set_li(1448, t); }
                            'lp16: loop {
                                if !(cx.li(1448) < cx.li(1472)) { break 'lp16; }
                                'blp16: {
                                    v5 = (v5 & 0xFFFF_FFFFi64) | (((cx.li(1448)) as u32 as i64) << 32);
                                    if (cx.i(Ptr::glob(0x75AA38).add(4 * (cx.li(1448)))) == 1) {
                                        v30 = cx.i(cx.lpp(378).add(28));
                                        v31 = { let q0_ = cx.ptr_at(cx.lpp(378).add(32)); let q1_ = cx.li(1448).wrapping_add(cx.li(1484)); ov_qt_face_index(cx, q0_, q1_) };
                                        { let q0_ = Ptr::glob(0x757078).add(4 * ((3_i32).wrapping_mul(cx.li(1044)))); let q1_ = cx.ptr_at(Ptr::raw(v30.wrapping_add(28))).add((12_i32).wrapping_mul(v31)); f_19e030(cx, q0_, q1_) };
                                        { let t = 0; cx.set_li(1036, t); }
                                        'lp17: loop {
                                            if !(cx.li(1036) < 3) { break 'lp17; }
                                            'blp17: {
                                                { let t = 0; cx.set_li(1032, t); }
                                                v104 = cx.li(1036);
                                                v32 = { let q0_ = cx.ptr_at(cx.lpp(378).add(32)); let q1_ = cx.li(1448).wrapping_add(cx.li(1484)); ov_qt_face_index(cx, q0_, q1_) };
                                                { let t = { let q0_ = cx.ptr_at(cx.lpp(378).add(28)); let q1_ = v32; let q2_ = v104; f_1b45f0(cx, q0_, q1_, q2_) }; cx.set_li(1028, t); }
                                                { let t = 0; cx.set_li(1040, t); }
                                                'lp18: loop {
                                                    if !(cx.li(1040) < cx.li(1124)) { break 'lp18; }
                                                    'blp18: {
                                                        v5 = (v5 & 0xFFFF_FFFFi64) | (((cx.li(1040)) as u32 as i64) << 32);
                                                        if (cx.i(Ptr::glob(0x750360).add(4 * (cx.li(1040)))) == cx.li(1028)) {
                                                            v5 = (v5 & 0xFFFF_FFFFi64) | (((cx.li(1036)) as u32 as i64) << 32);
                                                            { let p = Ptr::glob(0x758938).add(4 * ((4_i32).wrapping_mul(cx.li(1044)).wrapping_add(cx.li(1036)))); let t = cx.li(1040); cx.set_i(p, t); }
                                                            { let t = 1; cx.set_li(1032, t); }
                                                            break 'lp18;
                                                        }
                                                    }
                                                    let _ = cx.preinc_i(cx.lptr(1040));
                                                }
                                                if (cx.li(1032) == 0) {
                                                    v105 = cx.li(1036);
                                                    v33 = { let q0_ = cx.ptr_at(cx.lpp(378).add(32)); let q1_ = cx.li(1448).wrapping_add(cx.li(1484)); ov_qt_face_index(cx, q0_, q1_) };
                                                    v34 = { let q0_ = cx.ptr_at(cx.lpp(378).add(28)); let q1_ = v33; let q2_ = v105; ov_mesh_vertex(cx, q0_, q1_, q2_) };
                                                    { let q0_ = Ptr::glob(0x752638).add(4 * ((3_i32).wrapping_mul(cx.li(1124)))); let q1_ = v34; f_19e030(cx, q0_, q1_) };
                                                    { let p = Ptr::glob(0x758938).add(4 * ((4_i32).wrapping_mul(cx.li(1044)).wrapping_add(cx.li(1036)))); let t = cx.li(1124); cx.set_i(p, t); }
                                                    v106 = cx.li(1036);
                                                    v35 = { let q0_ = cx.ptr_at(cx.lpp(378).add(32)); let q1_ = cx.li(1448).wrapping_add(cx.li(1484)); ov_qt_face_index(cx, q0_, q1_) };
                                                    v5 = (v5 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.ptr_at(cx.lpp(378).add(28)); let q1_ = v35; let q2_ = v106; f_1b45f0(cx, q0_, q1_, q2_) }) as i32 as u32 as i64);
                                                    { let p = Ptr::glob(0x750360).add(4 * (cx.li(1124))); let t = (v5 as i32); cx.set_i(p, t); }
                                                    v5 = (v5 & 0xFFFF_FFFFi64) | (((cx.preinc_i(cx.lptr(1124))) as u32 as i64) << 32);
                                                }
                                            }
                                            let _ = cx.preinc_i(cx.lptr(1036));
                                        }
                                        let _ = cx.preinc_i(cx.lptr(1044));
                                    }
                                }
                                let _ = cx.preinc_i(cx.lptr(1448));
                            }
                        }
                        if (cx.li(1452) != 0) {
                            { let t = a4.wrapping_sub(cx.li(1508)); cx.set_li(1460, t); }
                            if ({ let q0_ = cx.lpp(374); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(1508))).bits(); let q2_ = cx.lptr(1460); let q3_ = Ptr::glob(0x752638); let q4_ = cx.li(1124); let q5_ = Ptr::glob(0x758938); let q6_ = Ptr::glob(0x757078); let q7_ = cx.li(1452); let q8_ = 0.1f32; let q9_ = 0.08f32; f_378bd0(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_) } != 0) {
                                if (cx.li(1460) != 0) {
                                    { let p = cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(36)).add(536); let t = 0; cx.set_i(p, t); }
                                    if ((cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(375).add(100)).add(36)).add(532)) as f32) > cx.f(Ptr::gc(0).add(1812))) {
                                        { let t = 0; cx.set_li(1024, t); }
                                        'lp19: loop {
                                            if !(cx.li(1024) < cx.li(1460)) { break 'lp19; }
                                            'blp19: {
                                                if (vsmath::fabs(cx.f(a3.add((1560_i32).wrapping_mul(cx.li(1024).wrapping_add(cx.li(1508)))).add(1336))) > (cx.f(Ptr::gc(0).add(1828)) as f64)) {
                                                    { let q0_ = cx.lpp(375); f_1e4ae0(cx, q0_) };
                                                    break 'lp19;
                                                }
                                            }
                                            let _ = cx.preinc_i(cx.lptr(1024));
                                        }
                                    }
                                }
                                { let t = 0; cx.set_li(1020, t); }
                                'lp20: loop {
                                    'blp20: {
                                        v5 = (v5 & 0xFFFF_FFFFi64) | (((cx.li(1020)) as u32 as i64) << 32);
                                        if (cx.li(1020) >= cx.li(1460)) {
                                            break 'lp20;
                                        }
                                        { let p = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(4); let t = cx.lpp(373); cx.set_ptr(p, t); }
                                        { let p = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(8); let t = 0; cx.set_i(p, t); }
                                        { let p = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1540); let t = cx.i(Ptr::gc(0).add(1804)); cx.set_i(p, t); }
                                        { let p = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1544); let t = cx.i(Ptr::gc(0).add(1804)); cx.set_i(p, t); }
                                        { let p = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(216); let t = -1; cx.set_i(p, t); }
                                        { let p = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(220); let t = -1; cx.set_i(p, t); }
                                        { let p = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1272); let t = cx.i(Ptr::gc(0).add(1800)); cx.set_i(p, t); }
                                        { let p = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1268); let t = cx.i(Ptr::gc(0).add(1796)); cx.set_i(p, t); }
                                        if (cx.f(Ptr::gc(0).add(1996)) != 0.0f32) {
                                            { let q0_ = cx.lptr(1004); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1276); let q2_ = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1332); let q3_ = 1.0f32; f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                            v107 = { let p = { let q0_ = cx.lptr(72); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                            v81 = cx.i(Ptr::glob(0x77FD8C));
                                            v59 = { let q0_ = cx.lptr(56); let q1_ = cx.li(1004); let q2_ = cx.li(1008); let q3_ = cx.li(1012); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v36 = { let q0_ = cx.lptr(40); let q1_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1276)); let q2_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1280)); let q3_ = cx.i(a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1284)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v36, v59, v81, v107);
                                            { let q0_ = cx.lptr(1004); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1276); let q2_ = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1332); let q3_ = (((cx.f(Ptr::gc(0).add(1996)) as f64) * (cx.f(a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1288)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                            { let q0_ = cx.lptr(984); let q1_ = a3.add((1560_i32).wrapping_mul(cx.li(1508))).add(1276); f_19e030(cx, q0_, q1_) };
                                            { let p = cx.lptr(988); let t = (((cx.f(cx.lptr(988)) as f64) + 0.05f64) as f32); cx.set_f(p, t); }
                                            v108 = { let p = { let q0_ = cx.lptr(36); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                            v82 = cx.i(Ptr::glob(0x77FD48));
                                            v60 = { let q0_ = cx.lptr(20); let q1_ = cx.li(1004); let q2_ = cx.li(1008); let q3_ = cx.li(1012); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v37 = { let q0_ = cx.lptr(4); let q1_ = cx.li(984); let q2_ = cx.li(988); let q3_ = cx.li(992); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v37, v60, v82, v108);
                                        }
                                        let _ = cx.preinc_i(cx.lptr(1508));
                                    }
                                    let _ = cx.preinc_i(cx.lptr(1020));
                                }
                            }
                        }
                    }
                    if (cx.li(1508) != 0) {
                        { let q0_ = cx.lptr(800); f_1b42f0(cx, q0_) };
                        { let t = 1023969417; cx.set_li(800, t); }
                        { let t = cx.i(Ptr::gc(0).add(1792)); cx.set_li(876, t); }
                        { let t = cx.i(Ptr::gc(0).add(1156)); cx.set_li(808, t); }
                        { let t = cx.i(Ptr::gc(0).add(1160)); cx.set_li(804, t); }
                        { let t = cx.i(Ptr::gc(0).add(1164)); cx.set_li(880, t); }
                        { let t = cx.i(Ptr::gc(0).add(1888)); cx.set_li(924, t); }
                        { let t = cx.i(Ptr::gc(0).add(1172)); cx.set_li(888, t); }
                        { let t = cx.i(Ptr::gc(0).add(1176)); cx.set_li(892, t); }
                        { let t = cx.i(Ptr::gc(0).add(1180)); cx.set_li(884, t); }
                        { let q0_ = cx.lptr(836); let q1_ = a5; f_19e030(cx, q0_, q1_) };
                        { let q0_ = cx.lptr(812); let q1_ = Ptr::gc(0).add(1848); f_19e030(cx, q0_, q1_) };
                        { let q0_ = a3; let q1_ = cx.li(1508); let q2_ = Ptr::NULL; let q3_ = 0; let q4_ = Ptr::NULL; let q5_ = 0; let q6_ = cx.lptr(800); f_389630(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
                        { let t = 0; cx.set_li(1508, t); }
                    }
                }
            }
            v5 = (v5 & !0xFFFF_FFFFi64) | ((cx.preinc_i(cx.lptr(1504))) as i32 as u32 as i64);
        }
    }
    v83 = v5;
    return v83;
}

/// PhysicsManager__1ADF00 0x001ADF00
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1adf00(cx: &mut Cx, this: Ptr, a2: f32) -> i32 {
    let mut i = 0i32;
    let mut v3 = Ptr::NULL;
    let mut result = 0i32;
    i = 0;
    'lp2: loop {
        if !(i < 30) { break 'lp2; }
        'blp2: {
            v3 = cx.ptr_at(this.add(4 * ((178_i32).wrapping_mul(i))).add(3384));
            if (v3.bits() != Ptr::NULL.bits()) {
                let _ = f_1e4c30(cx, v3, a2);
            }
            result = i.wrapping_add(1);
        }
        { i += 1; i };
    }
    return result;
}

/// PhysicsManager__1ADF70 0x001ADF70
pub fn f_1adf70(cx: &mut Cx, a1: i32, a3: f32, a4: Ptr) -> Ptr {
    cx.push_frame(2580);
    let r = f_1adf70_body(cx, a1, a3, a4);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1adf70_body(cx: &mut Cx, a1: i32, a3: f32, a4: Ptr) -> Ptr {
    let mut a2 = 0.0f64;
    let mut v4 = 0i32;
    let mut v5 = Ptr::NULL;
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
    let mut v18 = Ptr::NULL;
    let mut v19 = Ptr::NULL;
    let mut v20 = Ptr::NULL;
    let mut v21 = Ptr::NULL;
    let mut v22 = Ptr::NULL;
    let mut v23 = Ptr::NULL;
    let mut v24 = Ptr::NULL;
    let mut v25 = Ptr::NULL;
    let mut v26 = Ptr::NULL;
    let mut v27 = Ptr::NULL;
    let mut v28 = Ptr::NULL;
    let mut v29 = Ptr::NULL;
    let mut v30 = Ptr::NULL;
    let mut v31 = Ptr::NULL;
    let mut v32 = Ptr::NULL;
    let mut v33 = Ptr::NULL;
    let mut v34 = Ptr::NULL;
    let mut v35 = Ptr::NULL;
    let mut v36 = Ptr::NULL;
    let mut v37 = Ptr::NULL;
    let mut v38 = Ptr::NULL;
    let mut v39 = Ptr::NULL;
    let mut v40 = Ptr::NULL;
    let mut v41 = Ptr::NULL;
    let mut v42 = Ptr::NULL;
    let mut v43 = Ptr::NULL;
    let mut v44 = Ptr::NULL;
    let mut v45 = 0i32;
    let mut v46 = Ptr::NULL;
    let mut v47 = Ptr::NULL;
    let mut v48 = Ptr::NULL;
    let mut v49 = Ptr::NULL;
    let mut v50 = Ptr::NULL;
    let mut v51 = Ptr::NULL;
    let mut v52 = Ptr::NULL;
    let mut v53 = Ptr::NULL;
    let mut v54 = Ptr::NULL;
    let mut v55 = Ptr::NULL;
    let mut v56 = Ptr::NULL;
    let mut v57 = Ptr::NULL;
    let mut v58 = Ptr::NULL;
    let mut v59 = Ptr::NULL;
    let mut v60 = Ptr::NULL;
    let mut v61 = Ptr::NULL;
    let mut v62 = Ptr::NULL;
    let mut v63 = Ptr::NULL;
    let mut v65 = Ptr::NULL;
    let mut v66 = Ptr::NULL;
    let mut v67 = Ptr::NULL;
    let mut v68 = Ptr::NULL;
    let mut v69 = Ptr::NULL;
    let mut v70 = Ptr::NULL;
    let mut v71 = Ptr::NULL;
    let mut v72 = Ptr::NULL;
    let mut v73 = Ptr::NULL;
    let mut v74 = Ptr::NULL;
    let mut v75 = Ptr::NULL;
    let mut v76 = Ptr::NULL;
    let mut v77 = Ptr::NULL;
    let mut v78 = Ptr::NULL;
    let mut v79 = Ptr::NULL;
    let mut v80 = Ptr::NULL;
    let mut v81 = Ptr::NULL;
    let mut v82 = Ptr::NULL;
    let mut v83 = Ptr::NULL;
    let mut v84 = Ptr::NULL;
    let mut v85 = Ptr::NULL;
    let mut v86 = Ptr::NULL;
    let mut v87 = Ptr::NULL;
    let mut v88 = Ptr::NULL;
    let mut v89 = Ptr::NULL;
    let mut v90 = Ptr::NULL;
    let mut v91 = Ptr::NULL;
    let mut v92 = Ptr::NULL;
    let mut v93 = Ptr::NULL;
    let mut v94 = Ptr::NULL;
    let mut v95 = Ptr::NULL;
    let mut v96 = Ptr::NULL;
    let mut v97 = Ptr::NULL;
    let mut v98 = Ptr::NULL;
    let mut v99 = Ptr::NULL;
    let mut v100 = Ptr::NULL;
    let mut v101 = Ptr::NULL;
    let mut v102 = Ptr::NULL;
    let mut v103 = Ptr::NULL;
    let mut v104 = Ptr::NULL;
    let mut v105 = Ptr::NULL;
    let mut v106 = Ptr::NULL;
    let mut v107 = Ptr::NULL;
    let mut v108 = Ptr::NULL;
    let mut v109 = Ptr::NULL;
    let mut v110 = Ptr::NULL;
    let mut v111 = Ptr::NULL;
    let mut v112 = Ptr::NULL;
    let mut v113 = Ptr::NULL;
    let mut v114 = Ptr::NULL;
    let mut v115 = Ptr::NULL;
    let mut v116 = Ptr::NULL;
    let mut v117 = Ptr::NULL;
    let mut v118 = Ptr::NULL;
    let mut v119 = Ptr::NULL;
    let mut v120 = Ptr::NULL;
    let mut v121 = Ptr::NULL;
    let mut v122 = 0i32;
    let mut v123 = 0i32;
    let mut v124 = 0i32;
    let mut v125 = 0i32;
    let mut v126 = 0i32;
    let mut v127 = 0i32;
    let mut v128 = 0i32;
    let mut v129 = 0i32;
    let mut v130 = 0i32;
    let mut v131 = 0i32;
    let mut v132 = 0i32;
    let mut v133 = 0i32;
    let mut v134 = 0i32;
    let mut v135 = 0i32;
    let mut v136 = 0i32;
    let mut v137 = 0i32;
    let mut v138 = 0i32;
    let mut v139 = 0i32;
    let mut v140 = 0i32;
    let mut v141 = 0i32;
    let mut v142 = 0i32;
    let mut v143 = 0i32;
    let mut v144 = 0i32;
    let mut v145 = 0i32;
    let mut v146 = 0i32;
    let mut v147 = 0i32;
    let mut v148 = 0i32;
    let mut v149 = 0i32;
    let mut v150 = 0i32;
    let mut v151 = 0i32;
    let mut v152 = 0i32;
    let mut v153 = 0i32;
    let mut v154 = 0i32;
    let mut v155 = 0i32;
    let mut v156 = 0i32;
    let mut v157 = 0i32;
    let mut v158 = 0i32;
    let mut v159 = 0i32;
    let mut v160 = 0i32;
    let mut v161 = 0i32;
    let mut v162 = 0i32;
    let mut v163 = 0i32;
    let mut v164 = 0i32;
    let mut v165 = 0i32;
    let mut v166 = 0i32;
    let mut v167 = 0i32;
    let mut v168 = 0i32;
    let mut v169 = 0i32;
    let mut v170 = 0i32;
    let mut v171 = 0i32;
    let mut v172 = 0i32;
    let mut v173 = 0i32;
    let mut v174 = 0i32;
    let mut v175 = 0i32;
    let mut v176 = 0i32;
    let mut v177 = 0i32;
    let mut v178 = 0i32;
    let mut v179 = 0i32;
    let mut v180 = 0i32;
    let mut v181 = 0i32;
    let mut v182 = 0i32;
    let mut v183 = 0i32;
    let mut v184 = 0i32;
    let mut v185 = 0i32;
    let mut v186 = 0i32;
    let mut v187 = 0i32;
    let mut v188 = 0i32;
    let mut v189 = 0i32;
    let mut v190 = 0i32;
    let mut v191 = 0i32;
    let mut v192 = 0i32;
    let mut v193 = 0i32;
    let mut v194 = 0i32;
    let mut v195 = 0i32;
    let mut v196 = 0i32;
    let mut v197 = 0i32;
    let mut v198 = 0i32;
    let mut v199 = 0i32;
    let mut v200 = 0i32;
    let mut v201 = 0i32;
    let mut v202 = 0i32;
    let mut v203 = 0i32;
    let mut v204 = 0i32;
    let mut v205 = 0i32;
    let mut v206 = 0i32;
    let mut v207 = 0i32;
    let mut v208 = 0i32;
    let mut v209 = 0i32;
    let mut v210 = 0i32;
    let mut v211 = 0i32;
    let mut v212 = 0i32;
    let mut v213 = 0i32;
    let mut v214 = 0i32;
    let mut v215 = 0i32;
    let mut v216 = 0i32;
    let mut v217 = 0i32;
    let mut v218 = 0i32;
    let mut v219 = 0i32;
    let mut v220 = 0i32;
    let mut v221 = 0i32;
    let mut v222 = 0i32;
    let mut v223 = 0i32;
    let mut v224 = 0i32;
    let mut v225 = 0i32;
    let mut v226 = 0i32;
    let mut v227 = 0i32;
    let mut v228 = 0i32;
    let mut v229 = 0i32;
    let mut v230 = 0i32;
    let mut v231 = 0i32;
    let mut v232 = 0i32;
    let mut v233 = 0i32;
    let mut v234 = 0.0f32;
    let mut v235 = 0i32;
    let mut v236 = 0.0f32;
    let mut v237 = 0i32;
    { let t = a1; cx.set_li(2576, t); }
    if (a3 > 0.041841004f32) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            v4 = crate::ov_zero_i!(cx, "timeStep <= maximumStableTimestep", ".\\src\\Physics\\PhysicsManager.cpp", 5223);
            { let t = v4; cx.set_li(2572, t); }
            if (v4 == 3) {
            }
            { let t = 257; let o = cx.li(8); cx.set_li(8, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
        } else {
            { let t = 1; let o = cx.li(8); cx.set_li(8, (o & !(255 << 16)) | ((t & 255) << 16)); }
        }
    }
    { let p = cx.lptr(2568); let t = (((a3 as f64) * 24.0f64) as f32); cx.set_f(p, t); }
    { let t = 0; cx.set_li(2564, t); }
    'lp7: loop {
        if !(cx.li(2564) < cx.i(a4)) { break 'lp7; }
        'blp7: {
            if ({ let p = { let p = { let q0_ = cx.ptr_at(Ptr::raw(cx.li(2576))); let q1_ = a2; let q2_ = cx.li(2564); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }.add(28); cx.b(p) } != 0) {
                { let t = { let p = { let q0_ = cx.ptr_at(Ptr::raw(cx.li(2576))); let q1_ = a2; let q2_ = cx.li(2564); ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.ptr_at(p) }; cx.set_lpp(640, t); }
                { let t = { let q0_ = cx.lpp(640).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }; cx.set_lpp(639, t); }
                if (cx.i(cx.lpp(639).add(28)) <= 0) {
                    if ((cx.i(cx.lpp(639).add(28)) < 0) && (cx.preinc_i(cx.lpp(639).add(28)) >= cx.i(Ptr::gc(0).add(1276)).wrapping_neg())) {
                        { let q0_ = cx.lpp(640).add(240); let q1_ = cx.f(cx.lpp(639).add(32)); f_361ff0(cx, q0_, q1_) };
                    }
                } else if (cx.predec_i(cx.lpp(639).add(28)) <= cx.i(Ptr::gc(0).add(1276))) {
                    { let q0_ = cx.lpp(640).add(240); let q1_ = cx.f(cx.lpp(639).add(32)); f_361ff0(cx, q0_, q1_) };
                }
                { let t = 0; cx.set_li(2552, t); }
                'lp8: loop {
                    if !(cx.li(2552) < cx.i(Ptr::gc(0).add(36))) { break 'lp8; }
                    'blp8: {
                        { let p = cx.lptr(2548); let t = ((((cx.f(cx.lptr(2568)) * cx.f(cx.ptr_at(cx.lpp(640).add(3552)))) as f64) * (cx.f(Ptr::glob(0x688B70)) as f64)) as f32); cx.set_f(p, t); }
                        if (cx.f(cx.lptr(2548)) > 0.041841004f32) {
                            if ((cx.f(cx.lptr(2548)) / 2.0f32) >= 0.041841004f32) {
                                { let _ = { let q0_ = cx.lpp(640).add(240); let q1_ = ((cx.f(cx.lptr(2548)) as f64) / 3.0f64); let q2_ = 0; vcall(cx, 0, q0_, &[Varg::from(q1_), Varg::from(q2_)]) }; };
                                { let _ = { let q0_ = cx.lpp(640).add(240); let q1_ = ((cx.f(cx.lptr(2548)) as f64) / 3.0f64); let q2_ = 0; vcall(cx, 0, q0_, &[Varg::from(q1_), Varg::from(q2_)]) }; };
                                { let _ = { let q0_ = cx.lpp(640).add(240); let q1_ = ((cx.f(cx.lptr(2548)) as f64) / 3.0f64); let q2_ = 0; vcall(cx, 0, q0_, &[Varg::from(q1_), Varg::from(q2_)]) }; };
                            } else {
                                { let _ = { let q0_ = cx.lpp(640).add(240); let q1_ = ((cx.f(cx.lptr(2548)) as f64) / 2.0f64); let q2_ = 0; vcall(cx, 0, q0_, &[Varg::from(q1_), Varg::from(q2_)]) }; };
                                { let _ = { let q0_ = cx.lpp(640).add(240); let q1_ = ((cx.f(cx.lptr(2548)) as f64) / 2.0f64); let q2_ = 0; vcall(cx, 0, q0_, &[Varg::from(q1_), Varg::from(q2_)]) }; };
                            }
                        } else {
                            { let _ = { let q0_ = cx.lpp(640).add(240); let q1_ = cx.li(2548); let q2_ = 0; vcall(cx, 0, q0_, &[Varg::from(q1_), Varg::from(q2_)]) }; };
                        }
                        if ((cx.b(cx.lpp(640).add(4072)) != 0) && (cx.i(cx.lpp(640).add(4756)) == 1)) {
                            { let p = cx.lpp(640).add(4596); let t = (((cx.f(cx.lpp(640).add(4596)) as f64) + (cx.f(cx.ptr_at(cx.lpp(640).add(3552))) as f64)) as f32); cx.set_f(p, t); }
                            if (cx.b(cx.lpp(640).add(4760)) != 0) {
                                { let q0_ = cx.lptr(2528); let q1_ = cx.lpp(640).add(4740); let q2_ = cx.lpp(640).add(4764); f_1b07b0(cx, q0_, q1_, q2_) };
                                { let q0_ = cx.lptr(2540); let q1_ = -1.0f32; let q2_ = 1.0f32; f_1b0720(cx, q0_, q1_, q2_) };
                                a2 = vsmath::acos(cx.f(cx.lptr(2540)));
                                { let p = cx.lpp(640).add(4780); let t = (a2 as f32); cx.set_f(p, t); }
                            }
                        }
                        { let _ = { let q0_ = cx.lpp(640).add(240); let q1_ = cx.lpp(640).add(4788); vcall(cx, 24, q0_, &[Varg::from(q1_)]) }; };
                        if (cx.i(cx.lpp(640).add(4756)) == 1) {
                            { let q0_ = cx.lptr(2484); let q1_ = cx.lpp(640).add(4788); f_1b0760(cx, q0_, q1_) };
                            if (cx.i(cx.lpp(640).add(4784)) != 0) {
                                { let q0_ = cx.lpp(640).add(4764); let q1_ = cx.lpp(640).add(4688); let q2_ = cx.lpp(640).add(4628); let q3_ = cx.f(cx.lpp(640).add(4596)); f_1b0f90(cx, q0_, q1_, q2_, q3_) };
                                { let q0_ = cx.lpp(640).add(4788); let q1_ = cx.lpp(640).add(4764); f_1b0af0(cx, q0_, q1_) };
                            } else {
                                { let q0_ = cx.lpp(640).add(4764); let q1_ = cx.lpp(640).add(4788); f_39d710(cx, q0_, q1_) };
                                if (cx.f(Ptr::gc(0).add(1124)) == 0.0f32) {
                                    { let q0_ = Ptr::NULL.bits(); let q1_ = cx.lpp(640).add(4764); let q2_ = cx.lpp(640).add(4628); let q3_ = cx.f(cx.lpp(640).add(4596)); f_1b0ce0(cx, q0_, q1_, q2_, q3_) };
                                } else {
                                    { let p = cx.lptr(2456); let t = ((((cx.f(cx.lpp(640).add(4592)) * cx.f(Ptr::gc(0).add(1124))) as f64) + (cx.f(cx.ptr_at(cx.lpp(640).add(3552))) as f64)) as f32); cx.set_f(p, t); }
                                    if (cx.f(cx.lptr(2456)) <= cx.f(cx.lpp(640).add(4596))) {
                                        if ((cx.f(cx.lptr(2456)) + cx.f(cx.ptr_at(cx.lpp(640).add(3552)))) <= cx.f(cx.lpp(640).add(4596))) {
                                            { let q0_ = cx.lptr(2464); let q1_ = cx.lpp(640).add(4628); let q2_ = ((((cx.f(cx.lptr(2456)) / cx.f(cx.lpp(640).add(4592))) as f64) + 1.0f64) as f32); f_19d970(cx, q0_, q1_, q2_) };
                                        } else {
                                            { let q0_ = cx.lptr(2464); let q1_ = cx.lpp(640).add(4628); f_19e030(cx, q0_, q1_) };
                                        }
                                    } else {
                                        { let q0_ = cx.lptr(2464); let q1_ = cx.lpp(640).add(4628); let q2_ = (((cx.f(cx.lpp(640).add(4596)) as f64) / (cx.f(cx.lptr(2456)) as f64)) as f32); f_19d970(cx, q0_, q1_, q2_) };
                                    }
                                    { let q0_ = Ptr::NULL.bits(); let q1_ = cx.lpp(640).add(4764); let q2_ = cx.lptr(2464); let q3_ = cx.f(cx.lpp(640).add(4596)); f_1b0ce0(cx, q0_, q1_, q2_, q3_) };
                                }
                                { let q0_ = cx.lpp(640).add(4788); let q1_ = cx.lpp(640).add(4764); f_1b0af0(cx, q0_, q1_) };
                            }
                            { let q0_ = cx.lpp(640).add(240); let q1_ = cx.i(cx.lpp(640).add(4788)); let q2_ = cx.i(cx.lpp(640).add(4792)); let q3_ = cx.i(cx.lpp(640).add(4796)); let q4_ = cx.i(cx.lpp(640).add(4804)); let q5_ = cx.i(cx.lpp(640).add(4808)); let q6_ = cx.i(cx.lpp(640).add(4812)); let q7_ = cx.i(cx.lpp(640).add(4820)); let q8_ = cx.i(cx.lpp(640).add(4824)); let q9_ = cx.i(cx.lpp(640).add(4828)); f_36a460(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_) };
                            { let q0_ = cx.lpp(640).add(240); f_362970(cx, q0_) };
                            { let t = 0; cx.set_li(2452, t); }
                            'lp9: loop {
                                if !(cx.li(2452) < 4) { break 'lp9; }
                                'blp9: {
                                    { let q0_ = cx.lpp(640).add(240); let q1_ = cx.lpp(640).add((cx.li(2452) << 6)).add(4132); let q2_ = cx.li(2452); f_36e5d0(cx, q0_, q1_, q2_) };
                                }
                                let _ = cx.preinc_i(cx.lptr(2452));
                            }
                            { let q0_ = cx.lpp(640).add(240); let q1_ = cx.li(2484); let q2_ = cx.li(2488); let q3_ = cx.li(2492); let q4_ = cx.li(2496); let q5_ = cx.li(2500); let q6_ = cx.li(2504); let q7_ = cx.li(2508); let q8_ = cx.li(2512); let q9_ = cx.li(2516); f_36a460(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_) };
                            { let _ = { let q0_ = cx.lpp(640).add(240); vcall(cx, 32, q0_, &[]) }; };
                        }
                    }
                    let _ = cx.preinc_i(cx.lptr(2552));
                }
                let _ = cx.preinc_i(cx.lpp(640).add(4856));
                if (cx.b(cx.lpp(640).add(4072)) != 0) {
                    if ((cx.i(Ptr::gc(0).add(2032)) != 0) && (cx.i(cx.lpp(640).add(4524)) != 0)) {
                        { let t = 0; cx.set_li(2448, t); }
                        'lp10: loop {
                            if !(cx.li(2448) < cx.i(cx.lpp(640).add(4524)).wrapping_sub(1)) { break 'lp10; }
                            'blp10: {
                                v179 = { let p = { let q0_ = cx.lptr(2068); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                v122 = cx.i(Ptr::glob(0x77FD9C));
                                v65 = { let q0_ = cx.lptr(2052); let q1_ = cx.i(cx.ptr_at(cx.lpp(640).add(4520)).add((12_i32).wrapping_mul(cx.li(2448).wrapping_add(1)))); let q2_ = cx.i(cx.ptr_at(cx.lpp(640).add(4520)).add((12_i32).wrapping_mul(cx.li(2448).wrapping_add(1))).add(4)); let q3_ = cx.i(cx.ptr_at(cx.lpp(640).add(4520)).add((12_i32).wrapping_mul(cx.li(2448).wrapping_add(1))).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                v5 = { let q0_ = cx.lptr(2036); let q1_ = cx.i(cx.ptr_at(cx.lpp(640).add(4520)).add((12_i32).wrapping_mul(cx.li(2448)))); let q2_ = cx.i(cx.ptr_at(cx.lpp(640).add(4520)).add((12_i32).wrapping_mul(cx.li(2448))).add(4)); let q3_ = cx.i(cx.ptr_at(cx.lpp(640).add(4520)).add((12_i32).wrapping_mul(cx.li(2448))).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v5, v65, v122, v179);
                            }
                            let _ = cx.preinc_i(cx.lptr(2448));
                        }
                    }
                    if ((cx.i(Ptr::gc(0).add(2036)) != 0) && (cx.i(cx.lpp(640).add(4532)) != 0)) {
                        { let t = 0; cx.set_li(2444, t); }
                        'lp11: loop {
                            if !(cx.li(2444) < cx.i(cx.lpp(640).add(4532)).wrapping_sub(1)) { break 'lp11; }
                            'blp11: {
                                v180 = { let p = { let q0_ = cx.lptr(2032); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                v123 = cx.i(Ptr::glob(0x77FD8C));
                                v66 = { let q0_ = cx.lptr(2016); let q1_ = cx.i(cx.ptr_at(cx.lpp(640).add(4528)).add((12_i32).wrapping_mul(cx.li(2444).wrapping_add(1)))); let q2_ = cx.i(cx.ptr_at(cx.lpp(640).add(4528)).add((12_i32).wrapping_mul(cx.li(2444).wrapping_add(1))).add(4)); let q3_ = cx.i(cx.ptr_at(cx.lpp(640).add(4528)).add((12_i32).wrapping_mul(cx.li(2444).wrapping_add(1))).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                v6 = { let q0_ = cx.lptr(2000); let q1_ = cx.i(cx.ptr_at(cx.lpp(640).add(4528)).add((12_i32).wrapping_mul(cx.li(2444)))); let q2_ = cx.i(cx.ptr_at(cx.lpp(640).add(4528)).add((12_i32).wrapping_mul(cx.li(2444))).add(4)); let q3_ = cx.i(cx.ptr_at(cx.lpp(640).add(4528)).add((12_i32).wrapping_mul(cx.li(2444))).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v6, v66, v123, v180);
                            }
                            let _ = cx.preinc_i(cx.lptr(2444));
                        }
                    }
                    if ((cx.f(Ptr::gc(0).add(2040)) != 0.0f32) && (cx.i(cx.lpp(640).add(4716)) != 0)) {
                        { let q0_ = cx.lptr(2428); let q1_ = cx.lptr(2408); let q2_ = cx.lpp(640).add(4640); f_39ceb0(cx, q0_, q1_, q2_) };
                        { let q0_ = cx.lptr(2388); let q1_ = cx.lpp(640).add(4600); let q2_ = cx.lptr(2428); let q3_ = ((0.0f64 - (cx.f(Ptr::gc(0).add(2040)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                        { let q0_ = cx.lptr(2368); let q1_ = cx.lpp(640).add(4600); let q2_ = cx.lptr(2428); let q3_ = cx.f(Ptr::gc(0).add(2040)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                        v181 = { let p = { let q0_ = cx.lptr(1996); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v124 = cx.i(Ptr::glob(0x77FD8C));
                        v67 = { let q0_ = cx.lptr(1980); let q1_ = cx.li(2368); let q2_ = cx.li(2372); let q3_ = cx.li(2376); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v7 = { let q0_ = cx.lptr(1964); let q1_ = cx.li(2388); let q2_ = cx.li(2392); let q3_ = cx.li(2396); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v7, v67, v124, v181);
                        { let q0_ = cx.lptr(2388); let q1_ = cx.lpp(640).add(4600); let q2_ = cx.lptr(2408); let q3_ = ((0.0f64 - (cx.f(Ptr::gc(0).add(2040)) as f64)) as f32); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                        { let q0_ = cx.lptr(2368); let q1_ = cx.lpp(640).add(4600); let q2_ = cx.lptr(2408); let q3_ = cx.f(Ptr::gc(0).add(2040)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                        v182 = { let p = { let q0_ = cx.lptr(1960); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v125 = cx.i(Ptr::glob(0x77FD8C));
                        v68 = { let q0_ = cx.lptr(1944); let q1_ = cx.li(2368); let q2_ = cx.li(2372); let q3_ = cx.li(2376); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v8 = { let q0_ = cx.lptr(1928); let q1_ = cx.li(2388); let q2_ = cx.li(2392); let q3_ = cx.li(2396); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v8, v68, v125, v182);
                    }
                    if ((cx.i(Ptr::gc(0).add(2044)) != 0) && (cx.i(cx.lpp(640).add(4736)) != 0)) {
                        if (cx.i(Ptr::glob(0x751AD4)) == 0) {
                            { let t = ov_new(cx, 8); cx.set_lpp(481, t); }
                            if (cx.lpp(481).bits() != 0) {
                                { let t = { let q0_ = cx.lpp(481); let q1_ = 2; f_1b4730(cx, q0_, q1_) }; cx.set_lpp(1, t); }
                            } else {
                                { let t = 0; cx.set_lpp(1, Ptr::raw(t)); }
                            }
                            { let p = Ptr::glob(0x751AD4); let t = cx.lpp(1).bits(); cx.set_i(p, t); }
                        }
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD4)))).add(52); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(640).add(3556))).add(52); f_19e030(cx, q0_, q1_) };
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD4)))).add(184); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(640).add(3556))).add(184); f_19e030(cx, q0_, q1_) };
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD4)))).add(248); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(640).add(3556))).add(248); f_19e030(cx, q0_, q1_) };
                        { let p = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD4)))).add(196); let t = cx.i(Ptr::raw(cx.i(Ptr::glob(0x751AD4)))); cx.set_i(p, t); }
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD4)))); let q1_ = cx.lpp(640).add(4600); f_19e030(cx, q0_, q1_) };
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD4)))).add(16); let q1_ = cx.lpp(640).add(4720); f_1b0900(cx, q0_, q1_) };
                        let _ = f_1b4460(cx, Ptr::glob(0x751AD4));
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD4)))); let q1_ = cx.lptr(2264).bits(); let q2_ = cx.lptr(2264); f_37a7c0(cx, q0_, q1_, q2_) };
                        v183 = { let p = { let q0_ = cx.lptr(1920); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v126 = cx.i(Ptr::glob(0x77FD64));
                        v69 = { let q0_ = cx.lptr(1904); let q1_ = cx.li(2276); let q2_ = cx.li(2280); let q3_ = cx.li(2284); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v9 = { let q0_ = cx.lptr(1888); let q1_ = cx.li(2264); let q2_ = cx.li(2268); let q3_ = cx.li(2272); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v9, v69, v126, v183);
                        v184 = { let p = { let q0_ = cx.lptr(1884); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v127 = cx.i(Ptr::glob(0x77FD64));
                        v70 = { let q0_ = cx.lptr(1868); let q1_ = cx.li(2288); let q2_ = cx.li(2292); let q3_ = cx.li(2296); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v10 = { let q0_ = cx.lptr(1852); let q1_ = cx.li(2276); let q2_ = cx.li(2280); let q3_ = cx.li(2284); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v10, v70, v127, v184);
                        v185 = { let p = { let q0_ = cx.lptr(1848); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v128 = cx.i(Ptr::glob(0x77FD64));
                        v71 = { let q0_ = cx.lptr(1832); let q1_ = cx.li(2300); let q2_ = cx.li(2304); let q3_ = cx.li(2308); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v11 = { let q0_ = cx.lptr(1816); let q1_ = cx.li(2288); let q2_ = cx.li(2292); let q3_ = cx.li(2296); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v11, v71, v128, v185);
                        v186 = { let p = { let q0_ = cx.lptr(1812); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v129 = cx.i(Ptr::glob(0x77FD64));
                        v72 = { let q0_ = cx.lptr(1796); let q1_ = cx.li(2264); let q2_ = cx.li(2268); let q3_ = cx.li(2272); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v12 = { let q0_ = cx.lptr(1780); let q1_ = cx.li(2300); let q2_ = cx.li(2304); let q3_ = cx.li(2308); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v12, v72, v129, v186);
                        v187 = { let p = { let q0_ = cx.lptr(1776); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v130 = cx.i(Ptr::glob(0x77FD64));
                        v73 = { let q0_ = cx.lptr(1760); let q1_ = cx.li(2324); let q2_ = cx.li(2328); let q3_ = cx.li(2332); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v13 = { let q0_ = cx.lptr(1744); let q1_ = cx.li(2312); let q2_ = cx.li(2316); let q3_ = cx.li(2320); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v13, v73, v130, v187);
                        v188 = { let p = { let q0_ = cx.lptr(1740); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v131 = cx.i(Ptr::glob(0x77FD64));
                        v74 = { let q0_ = cx.lptr(1724); let q1_ = cx.li(2336); let q2_ = cx.li(2340); let q3_ = cx.li(2344); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v14 = { let q0_ = cx.lptr(1708); let q1_ = cx.li(2324); let q2_ = cx.li(2328); let q3_ = cx.li(2332); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v14, v74, v131, v188);
                        v189 = { let p = { let q0_ = cx.lptr(1704); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v132 = cx.i(Ptr::glob(0x77FD64));
                        v75 = { let q0_ = cx.lptr(1688); let q1_ = cx.li(2348); let q2_ = cx.li(2352); let q3_ = cx.li(2356); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v15 = { let q0_ = cx.lptr(1672); let q1_ = cx.li(2336); let q2_ = cx.li(2340); let q3_ = cx.li(2344); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v15, v75, v132, v189);
                        v190 = { let p = { let q0_ = cx.lptr(1668); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v133 = cx.i(Ptr::glob(0x77FD64));
                        v76 = { let q0_ = cx.lptr(1652); let q1_ = cx.li(2312); let q2_ = cx.li(2316); let q3_ = cx.li(2320); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v16 = { let q0_ = cx.lptr(1636); let q1_ = cx.li(2348); let q2_ = cx.li(2352); let q3_ = cx.li(2356); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v16, v76, v133, v190);
                        v191 = { let p = { let q0_ = cx.lptr(1632); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v134 = cx.i(Ptr::glob(0x77FD64));
                        v77 = { let q0_ = cx.lptr(1616); let q1_ = cx.li(2312); let q2_ = cx.li(2316); let q3_ = cx.li(2320); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v17 = { let q0_ = cx.lptr(1600); let q1_ = cx.li(2264); let q2_ = cx.li(2268); let q3_ = cx.li(2272); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v17, v77, v134, v191);
                        v192 = { let p = { let q0_ = cx.lptr(1596); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v135 = cx.i(Ptr::glob(0x77FD64));
                        v78 = { let q0_ = cx.lptr(1580); let q1_ = cx.li(2324); let q2_ = cx.li(2328); let q3_ = cx.li(2332); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v18 = { let q0_ = cx.lptr(1564); let q1_ = cx.li(2276); let q2_ = cx.li(2280); let q3_ = cx.li(2284); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v18, v78, v135, v192);
                        v193 = { let p = { let q0_ = cx.lptr(1560); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v136 = cx.i(Ptr::glob(0x77FD64));
                        v79 = { let q0_ = cx.lptr(1544); let q1_ = cx.li(2336); let q2_ = cx.li(2340); let q3_ = cx.li(2344); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v19 = { let q0_ = cx.lptr(1528); let q1_ = cx.li(2288); let q2_ = cx.li(2292); let q3_ = cx.li(2296); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v19, v79, v136, v193);
                        v194 = { let p = { let q0_ = cx.lptr(1524); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v137 = cx.i(Ptr::glob(0x77FD64));
                        v80 = { let q0_ = cx.lptr(1508); let q1_ = cx.li(2348); let q2_ = cx.li(2352); let q3_ = cx.li(2356); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v20 = { let q0_ = cx.lptr(1492); let q1_ = cx.li(2300); let q2_ = cx.li(2304); let q3_ = cx.li(2308); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v20, v80, v137, v194);
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD4)))).add(132); let q1_ = cx.i(Ptr::glob(0x751AD4)); let q2_ = cx.lptr(2264); f_37a7c0(cx, q0_, q1_, q2_) };
                        v195 = { let p = { let q0_ = cx.lptr(1488); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v138 = cx.i(Ptr::glob(0x77FD64));
                        v81 = { let q0_ = cx.lptr(1472); let q1_ = cx.li(2276); let q2_ = cx.li(2280); let q3_ = cx.li(2284); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v21 = { let q0_ = cx.lptr(1456); let q1_ = cx.li(2264); let q2_ = cx.li(2268); let q3_ = cx.li(2272); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v21, v81, v138, v195);
                        v196 = { let p = { let q0_ = cx.lptr(1452); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v139 = cx.i(Ptr::glob(0x77FD64));
                        v82 = { let q0_ = cx.lptr(1436); let q1_ = cx.li(2288); let q2_ = cx.li(2292); let q3_ = cx.li(2296); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v22 = { let q0_ = cx.lptr(1420); let q1_ = cx.li(2276); let q2_ = cx.li(2280); let q3_ = cx.li(2284); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v22, v82, v139, v196);
                        v197 = { let p = { let q0_ = cx.lptr(1416); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v140 = cx.i(Ptr::glob(0x77FD64));
                        v83 = { let q0_ = cx.lptr(1400); let q1_ = cx.li(2300); let q2_ = cx.li(2304); let q3_ = cx.li(2308); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v23 = { let q0_ = cx.lptr(1384); let q1_ = cx.li(2288); let q2_ = cx.li(2292); let q3_ = cx.li(2296); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v23, v83, v140, v197);
                        v198 = { let p = { let q0_ = cx.lptr(1380); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v141 = cx.i(Ptr::glob(0x77FD64));
                        v84 = { let q0_ = cx.lptr(1364); let q1_ = cx.li(2264); let q2_ = cx.li(2268); let q3_ = cx.li(2272); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v24 = { let q0_ = cx.lptr(1348); let q1_ = cx.li(2300); let q2_ = cx.li(2304); let q3_ = cx.li(2308); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v24, v84, v141, v198);
                        v199 = { let p = { let q0_ = cx.lptr(1344); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v142 = cx.i(Ptr::glob(0x77FD64));
                        v85 = { let q0_ = cx.lptr(1328); let q1_ = cx.li(2324); let q2_ = cx.li(2328); let q3_ = cx.li(2332); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v25 = { let q0_ = cx.lptr(1312); let q1_ = cx.li(2312); let q2_ = cx.li(2316); let q3_ = cx.li(2320); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v25, v85, v142, v199);
                        v200 = { let p = { let q0_ = cx.lptr(1308); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v143 = cx.i(Ptr::glob(0x77FD64));
                        v86 = { let q0_ = cx.lptr(1292); let q1_ = cx.li(2336); let q2_ = cx.li(2340); let q3_ = cx.li(2344); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v26 = { let q0_ = cx.lptr(1276); let q1_ = cx.li(2324); let q2_ = cx.li(2328); let q3_ = cx.li(2332); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v26, v86, v143, v200);
                        v201 = { let p = { let q0_ = cx.lptr(1272); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v144 = cx.i(Ptr::glob(0x77FD64));
                        v87 = { let q0_ = cx.lptr(1256); let q1_ = cx.li(2348); let q2_ = cx.li(2352); let q3_ = cx.li(2356); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v27 = { let q0_ = cx.lptr(1240); let q1_ = cx.li(2336); let q2_ = cx.li(2340); let q3_ = cx.li(2344); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v27, v87, v144, v201);
                        v202 = { let p = { let q0_ = cx.lptr(1236); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v145 = cx.i(Ptr::glob(0x77FD64));
                        v88 = { let q0_ = cx.lptr(1220); let q1_ = cx.li(2312); let q2_ = cx.li(2316); let q3_ = cx.li(2320); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v28 = { let q0_ = cx.lptr(1204); let q1_ = cx.li(2348); let q2_ = cx.li(2352); let q3_ = cx.li(2356); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v28, v88, v145, v202);
                        v203 = { let p = { let q0_ = cx.lptr(1200); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v146 = cx.i(Ptr::glob(0x77FD64));
                        v89 = { let q0_ = cx.lptr(1184); let q1_ = cx.li(2312); let q2_ = cx.li(2316); let q3_ = cx.li(2320); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v29 = { let q0_ = cx.lptr(1168); let q1_ = cx.li(2264); let q2_ = cx.li(2268); let q3_ = cx.li(2272); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v29, v89, v146, v203);
                        v204 = { let p = { let q0_ = cx.lptr(1164); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v147 = cx.i(Ptr::glob(0x77FD64));
                        v90 = { let q0_ = cx.lptr(1148); let q1_ = cx.li(2324); let q2_ = cx.li(2328); let q3_ = cx.li(2332); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v30 = { let q0_ = cx.lptr(1132); let q1_ = cx.li(2276); let q2_ = cx.li(2280); let q3_ = cx.li(2284); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v30, v90, v147, v204);
                        v205 = { let p = { let q0_ = cx.lptr(1128); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v148 = cx.i(Ptr::glob(0x77FD64));
                        v91 = { let q0_ = cx.lptr(1112); let q1_ = cx.li(2336); let q2_ = cx.li(2340); let q3_ = cx.li(2344); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v31 = { let q0_ = cx.lptr(1096); let q1_ = cx.li(2288); let q2_ = cx.li(2292); let q3_ = cx.li(2296); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v31, v91, v148, v205);
                        v206 = { let p = { let q0_ = cx.lptr(1092); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v149 = cx.i(Ptr::glob(0x77FD64));
                        v92 = { let q0_ = cx.lptr(1076); let q1_ = cx.li(2348); let q2_ = cx.li(2352); let q3_ = cx.li(2356); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v32 = { let q0_ = cx.lptr(1060); let q1_ = cx.li(2300); let q2_ = cx.li(2304); let q3_ = cx.li(2308); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v32, v92, v149, v206);
                    }
                    if ((cx.i(Ptr::gc(0).add(2048)) != 0) && (cx.i(cx.lpp(640).add(4756)) == 1)) {
                        if (cx.i(Ptr::glob(0x751AD0)) == 0) {
                            { let t = ov_new(cx, 8); cx.set_lpp(264, t); }
                            if (cx.lpp(264).bits() != 0) {
                                { let t = { let q0_ = cx.lpp(264); let q1_ = 2; f_1b4730(cx, q0_, q1_) }; cx.set_lpp(0, t); }
                            } else {
                                { let t = 0; cx.set_lpp(0, Ptr::raw(t)); }
                            }
                            { let p = Ptr::glob(0x751AD0); let t = cx.lpp(0).bits(); cx.set_i(p, t); }
                        }
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD0)))).add(52); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(640).add(3556))).add(52); f_19e030(cx, q0_, q1_) };
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD0)))).add(184); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(640).add(3556))).add(184); f_19e030(cx, q0_, q1_) };
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD0)))).add(248); let q1_ = cx.ptr_at(cx.ptr_at(cx.lpp(640).add(3556))).add(248); f_19e030(cx, q0_, q1_) };
                        { let p = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD0)))).add(196); let t = cx.i(Ptr::raw(cx.i(Ptr::glob(0x751AD0)))); cx.set_i(p, t); }
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD0)))); let q1_ = cx.lpp(640).add(4600); f_19e030(cx, q0_, q1_) };
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD0)))).add(16); let q1_ = cx.lpp(640).add(4740); f_1b0900(cx, q0_, q1_) };
                        let _ = f_1b4460(cx, Ptr::glob(0x751AD0));
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD0)))); let q1_ = cx.i(Ptr::glob(0x751AD0)); let q2_ = cx.lptr(2160); f_37a7c0(cx, q0_, q1_, q2_) };
                        v207 = { let p = { let q0_ = cx.lptr(1052); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v150 = cx.i(Ptr::glob(0x77FD84));
                        v93 = { let q0_ = cx.lptr(1036); let q1_ = cx.li(2172); let q2_ = cx.li(2176); let q3_ = cx.li(2180); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v33 = { let q0_ = cx.lptr(1020); let q1_ = cx.li(2160); let q2_ = cx.li(2164); let q3_ = cx.li(2168); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v33, v93, v150, v207);
                        v208 = { let p = { let q0_ = cx.lptr(1016); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v151 = cx.i(Ptr::glob(0x77FD84));
                        v94 = { let q0_ = cx.lptr(1000); let q1_ = cx.li(2184); let q2_ = cx.li(2188); let q3_ = cx.li(2192); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v34 = { let q0_ = cx.lptr(984); let q1_ = cx.li(2172); let q2_ = cx.li(2176); let q3_ = cx.li(2180); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v34, v94, v151, v208);
                        v209 = { let p = { let q0_ = cx.lptr(980); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v152 = cx.i(Ptr::glob(0x77FD84));
                        v95 = { let q0_ = cx.lptr(964); let q1_ = cx.li(2196); let q2_ = cx.li(2200); let q3_ = cx.li(2204); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v35 = { let q0_ = cx.lptr(948); let q1_ = cx.li(2184); let q2_ = cx.li(2188); let q3_ = cx.li(2192); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v35, v95, v152, v209);
                        v210 = { let p = { let q0_ = cx.lptr(944); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v153 = cx.i(Ptr::glob(0x77FD84));
                        v96 = { let q0_ = cx.lptr(928); let q1_ = cx.li(2160); let q2_ = cx.li(2164); let q3_ = cx.li(2168); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v36 = { let q0_ = cx.lptr(912); let q1_ = cx.li(2196); let q2_ = cx.li(2200); let q3_ = cx.li(2204); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v36, v96, v153, v210);
                        v211 = { let p = { let q0_ = cx.lptr(908); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v154 = cx.i(Ptr::glob(0x77FD84));
                        v97 = { let q0_ = cx.lptr(892); let q1_ = cx.li(2220); let q2_ = cx.li(2224); let q3_ = cx.li(2228); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v37 = { let q0_ = cx.lptr(876); let q1_ = cx.li(2208); let q2_ = cx.li(2212); let q3_ = cx.li(2216); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v37, v97, v154, v211);
                        v212 = { let p = { let q0_ = cx.lptr(872); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v155 = cx.i(Ptr::glob(0x77FD84));
                        v98 = { let q0_ = cx.lptr(856); let q1_ = cx.li(2232); let q2_ = cx.li(2236); let q3_ = cx.li(2240); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v38 = { let q0_ = cx.lptr(840); let q1_ = cx.li(2220); let q2_ = cx.li(2224); let q3_ = cx.li(2228); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v38, v98, v155, v212);
                        v213 = { let p = { let q0_ = cx.lptr(836); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v156 = cx.i(Ptr::glob(0x77FD84));
                        v99 = { let q0_ = cx.lptr(820); let q1_ = cx.li(2244); let q2_ = cx.li(2248); let q3_ = cx.li(2252); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v39 = { let q0_ = cx.lptr(804); let q1_ = cx.li(2232); let q2_ = cx.li(2236); let q3_ = cx.li(2240); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v39, v99, v156, v213);
                        v214 = { let p = { let q0_ = cx.lptr(800); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v157 = cx.i(Ptr::glob(0x77FD84));
                        v100 = { let q0_ = cx.lptr(784); let q1_ = cx.li(2208); let q2_ = cx.li(2212); let q3_ = cx.li(2216); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v40 = { let q0_ = cx.lptr(768); let q1_ = cx.li(2244); let q2_ = cx.li(2248); let q3_ = cx.li(2252); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v40, v100, v157, v214);
                        v215 = { let p = { let q0_ = cx.lptr(764); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v158 = cx.i(Ptr::glob(0x77FD84));
                        v101 = { let q0_ = cx.lptr(748); let q1_ = cx.li(2208); let q2_ = cx.li(2212); let q3_ = cx.li(2216); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v41 = { let q0_ = cx.lptr(732); let q1_ = cx.li(2160); let q2_ = cx.li(2164); let q3_ = cx.li(2168); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v41, v101, v158, v215);
                        v216 = { let p = { let q0_ = cx.lptr(728); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v159 = cx.i(Ptr::glob(0x77FD84));
                        v102 = { let q0_ = cx.lptr(712); let q1_ = cx.li(2220); let q2_ = cx.li(2224); let q3_ = cx.li(2228); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v42 = { let q0_ = cx.lptr(696); let q1_ = cx.li(2172); let q2_ = cx.li(2176); let q3_ = cx.li(2180); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v42, v102, v159, v216);
                        v217 = { let p = { let q0_ = cx.lptr(692); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v160 = cx.i(Ptr::glob(0x77FD84));
                        v103 = { let q0_ = cx.lptr(676); let q1_ = cx.li(2232); let q2_ = cx.li(2236); let q3_ = cx.li(2240); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v43 = { let q0_ = cx.lptr(660); let q1_ = cx.li(2184); let q2_ = cx.li(2188); let q3_ = cx.li(2192); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v43, v103, v160, v217);
                        v218 = { let p = { let q0_ = cx.lptr(656); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v161 = cx.i(Ptr::glob(0x77FD84));
                        v104 = { let q0_ = cx.lptr(640); let q1_ = cx.li(2244); let q2_ = cx.li(2248); let q3_ = cx.li(2252); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v44 = { let q0_ = cx.lptr(624); let q1_ = cx.li(2196); let q2_ = cx.li(2200); let q3_ = cx.li(2204); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v44, v104, v161, v218);
                        { let q0_ = cx.ptr_at(Ptr::raw(cx.i(Ptr::glob(0x751AD0)))).add(132); let q1_ = v45; let q2_ = cx.lptr(2160); f_37a7c0(cx, q0_, q1_, q2_) };
                        v219 = { let p = { let q0_ = cx.lptr(620); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v162 = cx.i(Ptr::glob(0x77FD84));
                        v105 = { let q0_ = cx.lptr(604); let q1_ = cx.li(2172); let q2_ = cx.li(2176); let q3_ = cx.li(2180); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v46 = { let q0_ = cx.lptr(588); let q1_ = cx.li(2160); let q2_ = cx.li(2164); let q3_ = cx.li(2168); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v46, v105, v162, v219);
                        v220 = { let p = { let q0_ = cx.lptr(584); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v163 = cx.i(Ptr::glob(0x77FD84));
                        v106 = { let q0_ = cx.lptr(568); let q1_ = cx.li(2184); let q2_ = cx.li(2188); let q3_ = cx.li(2192); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v47 = { let q0_ = cx.lptr(552); let q1_ = cx.li(2172); let q2_ = cx.li(2176); let q3_ = cx.li(2180); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v47, v106, v163, v220);
                        v221 = { let p = { let q0_ = cx.lptr(548); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v164 = cx.i(Ptr::glob(0x77FD84));
                        v107 = { let q0_ = cx.lptr(532); let q1_ = cx.li(2196); let q2_ = cx.li(2200); let q3_ = cx.li(2204); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v48 = { let q0_ = cx.lptr(516); let q1_ = cx.li(2184); let q2_ = cx.li(2188); let q3_ = cx.li(2192); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v48, v107, v164, v221);
                        v222 = { let p = { let q0_ = cx.lptr(512); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v165 = cx.i(Ptr::glob(0x77FD84));
                        v108 = { let q0_ = cx.lptr(496); let q1_ = cx.li(2160); let q2_ = cx.li(2164); let q3_ = cx.li(2168); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v49 = { let q0_ = cx.lptr(480); let q1_ = cx.li(2196); let q2_ = cx.li(2200); let q3_ = cx.li(2204); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v49, v108, v165, v222);
                        v223 = { let p = { let q0_ = cx.lptr(476); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v166 = cx.i(Ptr::glob(0x77FD84));
                        v109 = { let q0_ = cx.lptr(460); let q1_ = cx.li(2220); let q2_ = cx.li(2224); let q3_ = cx.li(2228); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v50 = { let q0_ = cx.lptr(444); let q1_ = cx.li(2208); let q2_ = cx.li(2212); let q3_ = cx.li(2216); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v50, v109, v166, v223);
                        v224 = { let p = { let q0_ = cx.lptr(440); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v167 = cx.i(Ptr::glob(0x77FD84));
                        v110 = { let q0_ = cx.lptr(424); let q1_ = cx.li(2232); let q2_ = cx.li(2236); let q3_ = cx.li(2240); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v51 = { let q0_ = cx.lptr(408); let q1_ = cx.li(2220); let q2_ = cx.li(2224); let q3_ = cx.li(2228); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v51, v110, v167, v224);
                        v225 = { let p = { let q0_ = cx.lptr(404); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v168 = cx.i(Ptr::glob(0x77FD84));
                        v111 = { let q0_ = cx.lptr(388); let q1_ = cx.li(2244); let q2_ = cx.li(2248); let q3_ = cx.li(2252); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v52 = { let q0_ = cx.lptr(372); let q1_ = cx.li(2232); let q2_ = cx.li(2236); let q3_ = cx.li(2240); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v52, v111, v168, v225);
                        v226 = { let p = { let q0_ = cx.lptr(368); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v169 = cx.i(Ptr::glob(0x77FD84));
                        v112 = { let q0_ = cx.lptr(352); let q1_ = cx.li(2208); let q2_ = cx.li(2212); let q3_ = cx.li(2216); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v53 = { let q0_ = cx.lptr(336); let q1_ = cx.li(2244); let q2_ = cx.li(2248); let q3_ = cx.li(2252); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v53, v112, v169, v226);
                        v227 = { let p = { let q0_ = cx.lptr(332); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v170 = cx.i(Ptr::glob(0x77FD84));
                        v113 = { let q0_ = cx.lptr(316); let q1_ = cx.li(2208); let q2_ = cx.li(2212); let q3_ = cx.li(2216); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v54 = { let q0_ = cx.lptr(300); let q1_ = cx.li(2160); let q2_ = cx.li(2164); let q3_ = cx.li(2168); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v54, v113, v170, v227);
                        v228 = { let p = { let q0_ = cx.lptr(296); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v171 = cx.i(Ptr::glob(0x77FD84));
                        v114 = { let q0_ = cx.lptr(280); let q1_ = cx.li(2220); let q2_ = cx.li(2224); let q3_ = cx.li(2228); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v55 = { let q0_ = cx.lptr(264); let q1_ = cx.li(2172); let q2_ = cx.li(2176); let q3_ = cx.li(2180); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v55, v114, v171, v228);
                        v229 = { let p = { let q0_ = cx.lptr(260); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v172 = cx.i(Ptr::glob(0x77FD84));
                        v115 = { let q0_ = cx.lptr(244); let q1_ = cx.li(2232); let q2_ = cx.li(2236); let q3_ = cx.li(2240); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v56 = { let q0_ = cx.lptr(228); let q1_ = cx.li(2184); let q2_ = cx.li(2188); let q3_ = cx.li(2192); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v56, v115, v172, v229);
                        v230 = { let p = { let q0_ = cx.lptr(224); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                        v173 = cx.i(Ptr::glob(0x77FD84));
                        v116 = { let q0_ = cx.lptr(208); let q1_ = cx.li(2244); let q2_ = cx.li(2248); let q3_ = cx.li(2252); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        v57 = { let q0_ = cx.lptr(192); let q1_ = cx.li(2196); let q2_ = cx.li(2200); let q3_ = cx.li(2204); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                        crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v57, v116, v173, v230);
                    }
                    if (cx.i(cx.lpp(640).add(4756)) == 1) {
                        { let _ = { let q0_ = cx.lpp(640).add(240); let q1_ = cx.lptr(2140); vcall(cx, 28, q0_, &[Varg::from(q1_)]) }; };
                        if (cx.f(Ptr::gc(0).add(2052)) != 0.0f32) {
                            { let q0_ = cx.lptr(2120); let q1_ = cx.lptr(2140); let q2_ = cx.lpp(640).add(4704); let q3_ = cx.f(Ptr::gc(0).add(2052)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                            v231 = { let p = { let q0_ = cx.lptr(188); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                            v174 = cx.i(Ptr::glob(0x77FD48));
                            v117 = { let q0_ = cx.lptr(172); let q1_ = cx.li(2120); let q2_ = cx.li(2124); let q3_ = cx.li(2128); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                            v58 = { let q0_ = cx.lptr(156); let q1_ = cx.li(2140); let q2_ = cx.li(2144); let q3_ = cx.li(2148); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v58, v117, v174, v231);
                        }
                        if (cx.f(Ptr::gc(0).add(2056)) != 0.0f32) {
                            { let q0_ = cx.lptr(2120); let q1_ = cx.lptr(2140); let q2_ = cx.lpp(640).add(4676); let q3_ = cx.f(Ptr::gc(0).add(2056)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                            v232 = { let p = { let q0_ = cx.lptr(152); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                            v175 = cx.i(Ptr::glob(0x77FD9C));
                            v118 = { let q0_ = cx.lptr(136); let q1_ = cx.li(2120); let q2_ = cx.li(2124); let q3_ = cx.li(2128); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                            v59 = { let q0_ = cx.lptr(120); let q1_ = cx.li(2140); let q2_ = cx.li(2144); let q3_ = cx.li(2148); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v59, v118, v175, v232);
                        }
                        if (cx.f(Ptr::gc(0).add(2060)) != 0.0f32) {
                            { let q0_ = cx.lptr(2120); let q1_ = cx.lptr(2140); let q2_ = cx.lpp(640).add(4628); let q3_ = cx.f(Ptr::gc(0).add(2060)); f_1a3970(cx, q0_, q1_, q2_, q3_) };
                            v233 = { let p = { let q0_ = cx.lptr(116); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                            v176 = cx.i(Ptr::glob(0x77FDAC));
                            v119 = { let q0_ = cx.lptr(100); let q1_ = cx.li(2120); let q2_ = cx.li(2124); let q3_ = cx.li(2128); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                            v60 = { let q0_ = cx.lptr(84); let q1_ = cx.li(2140); let q2_ = cx.li(2144); let q3_ = cx.li(2148); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v60, v119, v176, v233);
                        }
                        if (cx.f(Ptr::gc(0).add(2064)) != 0.0f32) {
                            v234 = cx.f(Ptr::gc(0).add(2064));
                            v61 = { let q0_ = cx.lpp(640).add(240); ov_car_omega(cx, q0_) };
                            { let q0_ = cx.lptr(2120); let q1_ = cx.lptr(2140); let q2_ = v61; let q3_ = v234; f_1a3970(cx, q0_, q1_, q2_, q3_) };
                            v235 = { let p = { let q0_ = cx.lptr(80); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                            v177 = cx.i(Ptr::glob(0x77FD9C));
                            v120 = { let q0_ = cx.lptr(64); let q1_ = cx.li(2120); let q2_ = cx.li(2124); let q3_ = cx.li(2128); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                            v62 = { let q0_ = cx.lptr(48); let q1_ = cx.li(2140); let q2_ = cx.li(2144); let q3_ = cx.li(2148); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v62, v120, v177, v235);
                        }
                    }
                }
                if ((cx.f(Ptr::gc(0).add(2028)) != 0.0f32) && (cx.b(cx.lpp(640).add(4072)) != 0)) {
                    { let t = 0; cx.set_li(2112, t); }
                    'lp12: loop {
                        if !(cx.li(2112) < 4) { break 'lp12; }
                        'blp12: {
                            if (cx.i(cx.lpp(640).add((4_i32).wrapping_mul(cx.li(2112))).add(1328)) != 0) {
                                { let q0_ = cx.lptr(2076); let q1_ = cx.lpp(640).add((136_i32).wrapping_mul(cx.li(2112))).add(784); f_19e030(cx, q0_, q1_) };
                                if (cx.f(Ptr::gc(0).add(2028)) > 0.0f32) {
                                    { let t = 0; cx.set_li(2080, t); }
                                }
                                a2 = vsmath::fabs(cx.f(Ptr::gc(0).add(2028)));
                                v236 = (a2 as f32);
                                { let q0_ = cx.lptr(2096); let q1_ = cx.lpp(640).add((136_i32).wrapping_mul(cx.li(2112))).add(824); let q2_ = cx.lptr(2076); let q3_ = v236; f_1a3970(cx, q0_, q1_, q2_, q3_) };
                                v237 = { let p = { let q0_ = cx.lptr(44); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                                v178 = cx.i(Ptr::glob(0x77FD9C));
                                v121 = { let q0_ = cx.lptr(28); let q1_ = cx.li(2096); let q2_ = cx.li(2100); let q3_ = cx.li(2104); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                v63 = { let q0_ = cx.lptr(12); let q1_ = cx.i(cx.lpp(640).add((136_i32).wrapping_mul(cx.li(2112))).add(824)); let q2_ = cx.i(cx.lpp(640).add((136_i32).wrapping_mul(cx.li(2112))).add(828)); let q3_ = cx.i(cx.lpp(640).add((136_i32).wrapping_mul(cx.li(2112))).add(832)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v63, v121, v178, v237);
                            }
                        }
                        let _ = cx.preinc_i(cx.lptr(2112));
                    }
                }
            }
        }
        let _ = cx.preinc_i(cx.lptr(2564));
    }
    return a4;
}

/// PhysicsManager__1B0720 0x001B0720
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b0720(cx: &mut Cx, mut a1: Ptr, a2: f32, a3: f32) -> Ptr {
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

/// PhysicsManager__1B0760 0x001B0760
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b0760(cx: &mut Cx, a1: Ptr, a2: Ptr) -> Ptr {
    let _ = f_19e030(cx, a1, a2);
    let _ = f_19e030(cx, a1.add(12), a2.add(16));
    return f_19e030(cx, a1.add(24), a2.add(32));
}

/// PhysicsManager__1B07B0 0x001B07B0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b07b0(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1.add(12); let t = ((((((cx.f(a2.add(12)) * cx.f(a3.add(12))) + (cx.f(a3) * cx.f(a2))) + (cx.f(a3.add(4)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(8)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1; let t = ((((((cx.f(a3.add(4)) * cx.f(a2.add(8))) - (cx.f(a3.add(8)) * cx.f(a2.add(4)))) - (cx.f(a3) * cx.f(a2.add(12)))) as f64) + ((cx.f(a2) * cx.f(a3.add(12))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((((cx.f(a3.add(8)) * cx.f(a2)) - (cx.f(a3) * cx.f(a2.add(8)))) - (cx.f(a3.add(4)) * cx.f(a2.add(12)))) as f64) + ((cx.f(a2.add(4)) * cx.f(a3.add(12))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((((cx.f(a3) * cx.f(a2.add(4))) - (cx.f(a3.add(4)) * cx.f(a2))) - (cx.f(a3.add(8)) * cx.f(a2.add(12)))) as f64) + ((cx.f(a2.add(8)) * cx.f(a3.add(12))) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// PhysicsManager__1B0900 0x001B0900
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b0900(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
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

/// PhysicsManager__1B0AF0 0x001B0AF0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b0af0(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = ((((((cx.f(a2) * cx.f(a2)) + (cx.f(a2.add(12)) * cx.f(a2.add(12)))) * 2.0f32) as f64) - 1.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((((cx.f(a2) * cx.f(a2.add(4))) + (cx.f(a2.add(8)) * cx.f(a2.add(12)))) as f64) * 2.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((((cx.f(a2.add(8)) * cx.f(a2)) - (cx.f(a2.add(4)) * cx.f(a2.add(12)))) as f64) * 2.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(16); let t = (((((cx.f(a2) * cx.f(a2.add(4))) - (cx.f(a2.add(8)) * cx.f(a2.add(12)))) as f64) * 2.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(20); let t = ((((((cx.f(a2.add(4)) * cx.f(a2.add(4))) + (cx.f(a2.add(12)) * cx.f(a2.add(12)))) * 2.0f32) as f64) - 1.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(24); let t = (((((cx.f(a2.add(4)) * cx.f(a2.add(8))) + (cx.f(a2) * cx.f(a2.add(12)))) as f64) * 2.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(32); let t = (((((cx.f(a2.add(8)) * cx.f(a2)) + (cx.f(a2.add(4)) * cx.f(a2.add(12)))) as f64) * 2.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(36); let t = (((((cx.f(a2.add(4)) * cx.f(a2.add(8))) - (cx.f(a2) * cx.f(a2.add(12)))) as f64) * 2.0f64) as f32); cx.set_f(p, t); }
    { let p = a1.add(40); let t = ((((((cx.f(a2.add(8)) * cx.f(a2.add(8))) + (cx.f(a2.add(12)) * cx.f(a2.add(12)))) * 2.0f32) as f64) - 1.0f64) as f32); cx.set_f(p, t); }
    return a2;
}

/// PhysicsManager__1B0CE0 0x001B0CE0
pub fn f_1b0ce0(cx: &mut Cx, a1: i32, a2: Ptr, a3: Ptr, a4: f32) -> i64 {
    cx.push_frame(92);
    let r = f_1b0ce0_body(cx, a1, a2, a3, a4);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1b0ce0_body(cx: &mut Cx, a1: i32, a2: Ptr, a3: Ptr, a4: f32) -> i64 {
    let mut v4 = 0i32;
    let mut v5 = 0i32;
    let mut v6 = 0i32;
    let mut v7 = 0i32;
    let mut v8 = 0i32;
    let mut v9 = 0i32;
    let mut v10 = 0i64;
    let mut v11 = 0.0f64;
    let mut v13 = 0i64;
    { let t = a1; cx.set_li(80, t); }
    /* return-value packing */
    { let t = cx.f(a2); cx.set_lf(48, t); }
    { let t = cx.f(a2.add(4)); cx.set_lf(52, t); }
    { let t = cx.f(a2.add(8)); cx.set_lf(56, t); }
    { let t = cx.f(a2.add(12)); cx.set_lf(60, t); }
    { let t = (f_19d9c0(cx, a3, a3) as f32); cx.set_lf(40, t); }
    v8 = { let q0_ = cx.lf(40); ov_isnan(cx, q0_) };
    v4 = ((cx.lf(40) < 0.0f32) as i32);
    v5 = ((cx.lf(40) == 0.0f32) as i32);
    v6 = 0;
    v7 = 0;
    v10 = (v10 & !(0xFF00i64 as i64)) | ((((v9) as i64) & 0xFF) << 8) as i64;
    if (cx.lf(40) != 0.0f32) {
        { let t = (vsmath::sqrt(cx.lf(40)) as f32); cx.set_lf(40, t); }
        { let t = ((((0.5f32 * cx.lf(40)) as f64) * (a4 as f64)) as f32); cx.set_lf(36, t); }
        v11 = vsmath::sin(cx.lf(36));
        { let t = ((v11 / (cx.lf(40) as f64)) as f32); cx.set_lf(32, t); }
        { let t = (vsmath::cos(cx.lf(36)) as f32); cx.set_lf(24, t); }
        { let t = (((cx.lf(32) as f64) * (cx.f(a3) as f64)) as f32); cx.set_lf(12, t); }
        { let t = (((cx.lf(32) as f64) * (cx.f(a3.add(4)) as f64)) as f32); cx.set_lf(16, t); }
        { let t = (((cx.lf(32) as f64) * (cx.f(a3.add(8)) as f64)) as f32); cx.set_lf(20, t); }
        v10 = (v10 & !0xFFFF_FFFFi64) | (({ let q0_ = a2; let q1_ = cx.lptr(12); let q2_ = cx.lptr(48); f_1b0e40(cx, q0_, q1_, q2_) }.bits()) as i32 as u32 as i64);
    }
    v13 = v10;
    return v13;
}

/// PhysicsManager__1B0E40 0x001B0E40
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b0e40(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1.add(12); let t = ((((((cx.f(a2.add(12)) * cx.f(a3.add(12))) - (cx.f(a2) * cx.f(a3))) - (cx.f(a2.add(4)) * cx.f(a3.add(4)))) as f64) - ((cx.f(a2.add(8)) * cx.f(a3.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1; let t = ((((((cx.f(a2.add(4)) * cx.f(a3.add(8))) - (cx.f(a2.add(8)) * cx.f(a3.add(4)))) + (cx.f(a2.add(12)) * cx.f(a3))) as f64) + ((cx.f(a2) * cx.f(a3.add(12))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((((cx.f(a2.add(8)) * cx.f(a3)) - (cx.f(a2) * cx.f(a3.add(8)))) + (cx.f(a2.add(12)) * cx.f(a3.add(4)))) as f64) + ((cx.f(a2.add(4)) * cx.f(a3.add(12))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((((cx.f(a2) * cx.f(a3.add(4))) - (cx.f(a2.add(4)) * cx.f(a3))) + (cx.f(a2.add(12)) * cx.f(a3.add(8)))) as f64) + ((cx.f(a2.add(8)) * cx.f(a3.add(12))) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// PhysicsManager__1B0F90 0x001B0F90
pub fn f_1b0f90(cx: &mut Cx, a1: Ptr, a2: Ptr, a3: Ptr, a4: f32) -> i64 {
    cx.push_frame(32);
    let r = f_1b0f90_body(cx, a1, a2, a3, a4);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1b0f90_body(cx: &mut Cx, a1: Ptr, a2: Ptr, a3: Ptr, a4: f32) -> i64 {
    let mut v4 = 0i32;
    let mut v5 = 0i32;
    let mut v6 = 0i32;
    let mut v7 = 0i32;
    let mut v8 = 0i32;
    let mut v9 = 0i32;
    let mut v10 = 0i64;
    let mut v12 = 0i64;
    { let t = (f_19d9c0(cx, a3, a3) as f32); cx.set_lf(28, t); }
    v8 = { let q0_ = cx.lf(28); ov_isnan(cx, q0_) };
    v4 = ((cx.lf(28) < 0.0f32) as i32);
    v5 = ((cx.lf(28) == 0.0f32) as i32);
    v6 = 0;
    v7 = 0;
    v10 = (v10 & !(0xFF00i64 as i64)) | ((((v9) as i64) & 0xFF) << 8) as i64;
    if (cx.lf(28) != 0.0f32) {
        { let t = (vsmath::sqrt(cx.lf(28)) as f32); cx.set_lf(28, t); }
        { let t = ((((0.5f32 * cx.lf(28)) as f64) * (a4 as f64)) as f32); cx.set_lf(24, t); }
        { let t = ((vsmath::sin(cx.lf(24)) / (cx.lf(28) as f64)) as f32); cx.set_lf(20, t); }
        { let t = (vsmath::cos(cx.lf(24)) as f32); cx.set_lf(12, t); }
        { let t = (((cx.lf(20) as f64) * (cx.f(a3) as f64)) as f32); cx.set_lf(0, t); }
        { let t = (((cx.lf(20) as f64) * (cx.f(a3.add(4)) as f64)) as f32); cx.set_lf(4, t); }
        { let t = (((cx.lf(20) as f64) * (cx.f(a3.add(8)) as f64)) as f32); cx.set_lf(8, t); }
        v10 = (v10 & !0xFFFF_FFFFi64) | (({ let q0_ = a1; let q1_ = cx.lptr(0); let q2_ = a2; f_1b0e40(cx, q0_, q1_, q2_) }.bits()) as i32 as u32 as i64);
    }
    v12 = v10;
    return v12;
}

/// PhysicsManager__1B10B0 0x001B10B0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b10b0(cx: &mut Cx, a1: Ptr) -> Ptr {
    let _ = f_1b1130(cx, a1, 1065353216, 0, 0, 0);
    let _ = f_1b1130(cx, a1.add(16), 0, 1065353216, 0, 0);
    let _ = f_1b1130(cx, a1.add(32), 0, 0, 1065353216, 0);
    return f_1b1130(cx, a1.add(48), 0, 0, 0, 1065353216);
}

/// PhysicsManager__1B1130 0x001B1130
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b1130(cx: &mut Cx, mut a1: Ptr, a2: i32, a3: i32, a4: i32, a5: i32) -> Ptr {
    { let p = a1; let t = a2; cx.set_i(p, t); }
    { let p = a1.add(4); let t = a3; cx.set_i(p, t); }
    { let p = a1.add(8); let t = a4; cx.set_i(p, t); }
    { let p = a1.add(12); let t = a5; cx.set_i(p, t); }
    return a1;
}

/// PhysicsManager__1B1170 0x001B1170
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b1170(cx: &mut Cx, a1: Ptr) -> Ptr {
    let _ = f_19d020(cx, a1);
    let _ = f_19d020(cx, a1.add(12));
    return f_19d020(cx, a1.add(24));
}

/// PhysicsManager__1B11B0 0x001B11B0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b11b0(cx: &mut Cx, a1: Ptr, a3: Ptr) -> () {
    let mut i = 0i32;
    let mut a2 = 0.0f64;
    let mut v3 = 0i32;
    i = 0;
    'lp2: loop {
        if !(i < cx.i(a3)) { break 'lp2; }
        'blp2: {
            v3 = { let p = { let q0_ = cx.ptr_at(a1); let q1_ = a2; let q2_ = i; ov_vehicle_slot2(cx, q0_, q1_, q2_) }; cx.i(p) };
            if (cx.b(Ptr::raw(v3.wrapping_add(28))) != 0) {
                { let q0_ = Ptr::raw(v3); let q1_ = cx.ptr_at(cx.ptr_at(Ptr::raw(v3.wrapping_add(3556)))).add(16); f_1bb100(cx, q0_, q1_) };
                { let _ = { let q0_ = Ptr::raw(v3.wrapping_add(240)); let q1_ = cx.i(cx.ptr_at(Ptr::raw(v3.wrapping_add(3556)))); vcall(cx, 28, q0_, &[Varg::from(q1_)]) }; };
                { let q0_ = cx.ptr_at(cx.ptr_at(Ptr::raw(v3.wrapping_add(3556)))); let q1_ = cx.ptr_at(cx.ptr_at(Ptr::raw(v3.wrapping_add(3556)))).add(16); let q2_ = cx.f(cx.ptr_at(cx.ptr_at(Ptr::raw(v3.wrapping_add(3556)))).add(116)); f_1a0390(cx, q0_, q1_, q2_) };
                { let q0_ = cx.ptr_at(cx.ptr_at(Ptr::raw(v3.wrapping_add(3556)))); let q1_ = cx.ptr_at(cx.ptr_at(Ptr::raw(v3.wrapping_add(3556)))).add(28); let q2_ = cx.f(cx.ptr_at(cx.ptr_at(Ptr::raw(v3.wrapping_add(3556)))).add(120)); f_1a0390(cx, q0_, q1_, q2_) };
                { let q0_ = cx.ptr_at(Ptr::raw(v3.wrapping_add(3556))); f_1b4460(cx, q0_) };
                { let p = Ptr::raw(v3.wrapping_add(3604)); let t = f_364be0(cx, Ptr::raw(v3.wrapping_add(240))); cx.set_i(p, t); }
            }
        }
        { i += 1; i };
    }
}

/// PhysicsManager__1B32E0 0x001B32E0
pub fn f_1b32e0(cx: &mut Cx, this: Ptr, a2: f32) -> i32 {
    cx.push_frame(40604);
    let r = f_1b32e0_body(cx, this, a2);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1b32e0_body(cx: &mut Cx, this: Ptr, a2: f32) -> i32 {
    let mut v2 = 0.0f64;
    let mut v5 = 0i64;
    let mut v8 = 0.0f64;
    let mut v9 = 0i32;
    let mut v10 = 0i32;
    let mut v12 = 0i64;
    let mut bb: u32 = 0;
    #[allow(unreachable_code)]
    'cfg: loop {
        match bb {
            0 => {
                { let t = this; cx.set_lpp(10150, t); }
                { let t = { let q0_ = cx.i(this); ov_vehicle_count(cx, q0_) }; cx.set_li(40592, t); }
                { let t = 0; let o = cx.li(40580); cx.set_li(40580, (o & !(255 << 24)) | ((t & 255) << 24)); }
                if (cx.gc.do_CD == 0) { bb = 1; continue 'cfg; } else { bb = 2; continue 'cfg; }
                break 'cfg;
            }
            1 => {
                { bb = 4; continue 'cfg; }
                break 'cfg;
            }
            2 => {
                { bb = 3; continue 'cfg; }
                break 'cfg;
            }
            3 => {
                { let t = 26; cx.set_li(40572, t); }
                { let q0_ = cx.lptr(8); let q1_ = 1560; let q2_ = 26; let q3_ = Ptr::raw(Ptr::func(0x398D80).bits()); ov_vector_ctor(cx, q0_, q1_, q2_, q3_) };
                { let t = { let p = ov_physics_manager(cx).add(32); cx.ptr_at(p) }; cx.set_lpp(0, t); }
                v5 = (v5 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.ptr_at(cx.lpp(10150)); let q1_ = v2; let q2_ = 0; ov_vehicle_slot2(cx, q0_, q1_, q2_) }.bits()) as i32 as u32 as i64);
                v5 = (v5 & 0xFFFF_FFFFi64) | (((cx.i(Ptr::raw((v5 as i32)))) as u32 as i64) << 32);
                if (cx.i(cx.ptr_at(Ptr::raw((v5 as i32))).add(3556)) != 0) { bb = 7; continue 'cfg; } else { bb = 8; continue 'cfg; }
                break 'cfg;
            }
            4 => {
                v5 = (v5 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.lpp(10150).bits(); let q1_ = a2; let q2_ = cx.lptr(40592); f_1adf70(cx, q0_, q1_, q2_) }.bits()) as i32 as u32 as i64);
                { bb = 5; continue 'cfg; }
                break 'cfg;
            }
            5 => {
                v12 = v5;
                return (v12 as i32);
            }
            6 => {
                { bb = 3; continue 'cfg; }
                break 'cfg;
            }
            7 => {
                v5 = (v5 & !0xFFFF_FFFFi64) | ((cx.lpp(0).bits()) as i32 as u32 as i64);
                if (cx.i(cx.lpp(0).add(72)) != 0) { bb = 10; continue 'cfg; } else { bb = 11; continue 'cfg; }
                break 'cfg;
            }
            8 => {
                { bb = 9; continue 'cfg; }
                break 'cfg;
            }
            9 => {
                { bb = 5; continue 'cfg; }
                break 'cfg;
            }
            10 => {
                { let q0_ = cx.lpp(10150); let q1_ = a2; f_1adf00(cx, q0_, q1_) };
                { let q0_ = cx.lpp(10150); let q1_ = cx.lptr(40592); f_1b11b0(cx, q0_, q1_) };
                { let q0_ = cx.lpp(10150).bits(); let q1_ = cx.lptr(40592); let q2_ = cx.lptr(8); let q3_ = 26; let q4_ = Ptr::glob(0x68899C); f_1a0570(cx, q0_, q1_, q2_, q3_, q4_) };
                { let q0_ = cx.lpp(10150); let q1_ = cx.lptr(40592); let q2_ = cx.lptr(8); let q3_ = 26; let q4_ = Ptr::glob(0x68899C); let q5_ = cx.lptr(40580).add(3); f_1a3a30(cx, q0_, q1_, q2_, q3_, q4_, q5_) };
                if (cx.gc.barrier_CD_enabled != 0) { bb = 13; continue 'cfg; } else { bb = 14; continue 'cfg; }
                break 'cfg;
            }
            11 => {
                { bb = 12; continue 'cfg; }
                break 'cfg;
            }
            12 => {
                { bb = 9; continue 'cfg; }
                break 'cfg;
            }
            13 => {
                { let q0_ = cx.lpp(10150); let q1_ = cx.lptr(40592); let q2_ = cx.lptr(8); let q3_ = 26; let q4_ = Ptr::glob(0x68899C); f_1a7060(cx, q0_, q1_, q2_, q3_, q4_) };
                { bb = 15; continue 'cfg; }
                break 'cfg;
            }
            14 => {
                { bb = 15; continue 'cfg; }
                break 'cfg;
            }
            15 => {
                if (cx.gc.static_object_CD_enabled != 0) { bb = 16; continue 'cfg; } else { bb = 17; continue 'cfg; }
                break 'cfg;
            }
            16 => {
                { let q0_ = cx.lpp(10150); let q1_ = cx.lptr(40592); let q2_ = cx.lptr(8); let q3_ = 26; let q4_ = Ptr::glob(0x68899C); f_1a89f0(cx, q0_, q1_, q2_, q3_, q4_) };
                { bb = 18; continue 'cfg; }
                break 'cfg;
            }
            17 => {
                { bb = 18; continue 'cfg; }
                break 'cfg;
            }
            18 => {
                if (cx.gc.car_object_CD_enabled != 0) { bb = 19; continue 'cfg; } else { bb = 20; continue 'cfg; }
                break 'cfg;
            }
            19 => {
                { let q0_ = cx.lpp(10150).bits(); let q1_ = cx.lptr(40592); let q2_ = cx.lptr(8); let q3_ = 26; let q4_ = Ptr::glob(0x68899C); f_1aaed0(cx, q0_, q1_, q2_, q3_, q4_) };
                { bb = 21; continue 'cfg; }
                break 'cfg;
            }
            20 => {
                { bb = 21; continue 'cfg; }
                break 'cfg;
            }
            21 => {
                { let q0_ = cx.lpp(10150); let q1_ = v8; let q2_ = v9; let q3_ = cx.lptr(8); let q4_ = 26; crate::ov_ignore!(cx, q0_, q1_, q2_, q3_, q4_) };
                { let q0_ = cx.lpp(10150).bits(); let q1_ = v10; let q2_ = cx.lptr(8); let q3_ = 26; let q4_ = Ptr::glob(0x68899C); f_1ac680(cx, q0_, q1_, q2_, q3_, q4_) };
                { bb = 4; continue 'cfg; }
                break 'cfg;
            }
            22 => {
                { bb = 12; continue 'cfg; }
                break 'cfg;
            }
            23 => {
                break 'cfg;
            }
            24 => {
                break 'cfg;
            }
            _ => unreachable!(),
        }
    }
    #[allow(unreachable_code)]
    0
}

/// PhysicsManager__1B3500 0x001B3500
pub fn f_1b3500(cx: &mut Cx, this: Ptr, pos: Ptr, a4: i32, a5: i32, a6: i32) -> i64 {
    cx.push_frame(116);
    let r = f_1b3500_body(cx, this, pos, a4, a5, a6);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1b3500_body(cx: &mut Cx, this: Ptr, pos: Ptr, a4: i32, a5: i32, a6: i32) -> i64 {
    let mut v5 = 0i64;
    let mut v7 = 0i64;
    { let t = this; cx.set_lpp(28, t); }
    { let t = -1; cx.set_li(108, t); }
    { let t = 0; cx.set_li(104, t); }
    'lp3: loop {
        if !(cx.li(104) < 100) { break 'lp3; }
        'blp3: {
            if (cx.b(cx.lpp(28).add((28_i32).wrapping_mul(cx.li(104))).add(36)) == 0) {
                { let t = cx.li(104); cx.set_li(108, t); }
                break 'lp3;
            }
        }
        let _ = cx.preinc_i(cx.lptr(104));
    }
    if (cx.li(108) == -1) {
        v5 = (v5 & 0xFFFF_FFFFi64) | (((0) as u32 as i64) << 32);
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            let _ = crate::ov_zero_i!(cx, "0", ".\\src\\Physics\\PhysicsManager.cpp", 5865);
            { let t = (v5 as i32); cx.set_li(100, t); }
            if ((v5 as i32) == 3) {
            }
            { let t = 257; let o = cx.li(0); cx.set_li(0, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
        } else {
            { let t = 1; let o = cx.li(0); cx.set_li(0, (o & !(255 << 16)) | ((t & 255) << 16)); }
        }
        v5 = (v5 & !0xFFFF_FFFFi64) | ((-1) as i32 as u32 as i64);
    } else {
        { let p = cx.lpp(28).add((28_i32).wrapping_mul(cx.li(108))).add(36); let t = 1; cx.set_b(p, t); }
        { let p = cx.lpp(28).add((28_i32).wrapping_mul(cx.li(108))).add(56); let t = a4; cx.set_i(p, t); }
        { let p = cx.lpp(28).add((28_i32).wrapping_mul(cx.li(108))).add(52); let t = a5; cx.set_i(p, t); }
        { let p = cx.lpp(28).add((28_i32).wrapping_mul(cx.li(108))).add(40); let t = cx.i(pos); cx.set_i(p, t); }
        { let p = cx.lpp(28).add((28_i32).wrapping_mul(cx.li(108))).add(44); let t = cx.i(pos.add(4)); cx.set_i(p, t); }
        { let p = cx.lpp(28).add((28_i32).wrapping_mul(cx.li(108))).add(48); let t = cx.i(pos.add(8)); cx.set_i(p, t); }
        { let p = cx.lpp(28).add((28_i32).wrapping_mul(cx.li(108))).add(60); let t = a6; cx.set_i(p, t); }
        { let q0_ = cx.ptr_at(cx.lpp(28).add(32)); let q1_ = cx.lptr(80); let q2_ = cx.lptr(92); let q3_ = cx.lptr(80); let q4_ = pos; let q5_ = 0; let q6_ = Ptr::NULL; ov_qt_find_leaf(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
        { let q0_ = cx.lptr(36); f_1b4820(cx, q0_) };
        { let t = cx.i(cx.lpp(28).add((28_i32).wrapping_mul(cx.li(108))).add(52)); cx.set_li(64, t); }
        { let t = cx.i(cx.lpp(28).add((28_i32).wrapping_mul(cx.li(108))).add(40)); cx.set_li(36, t); }
        { let t = cx.i(cx.lpp(28).add((28_i32).wrapping_mul(cx.li(108))).add(44)); cx.set_li(40, t); }
        { let t = cx.i(cx.lpp(28).add((28_i32).wrapping_mul(cx.li(108))).add(48)); cx.set_li(44, t); }
        { let t = 1065353216; cx.set_li(68, t); }
        { let t = 0; cx.set_li(52, t); }
        { let t = 1065353216; cx.set_li(56, t); }
        { let t = 0; cx.set_li(60, t); }
        { let t = 0; cx.set_li(28, t); }
        'lp4: loop {
            'blp4: {
                v5 = (v5 & 0xFFFF_FFFFi64) | (((cx.li(28)) as u32 as i64) << 32);
                if (cx.li(28) >= cx.li(80)) {
                    break 'lp4;
                }
                { let t = { let q0_ = cx.ptr_at(cx.lpp(28).add(32)); let q1_ = cx.li(28).wrapping_add(cx.li(92)); ov_qt_face_index(cx, q0_, q1_) }; cx.set_li(24, t); }
                { let t = { let q0_ = cx.ptr_at(cx.lpp(28).add(28)); let q1_ = cx.li(24); let q2_ = 0; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(5, t); }
                { let t = { let q0_ = cx.ptr_at(cx.lpp(28).add(28)); let q1_ = cx.li(24); let q2_ = 1; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(4, t); }
                { let t = { let q0_ = cx.ptr_at(cx.lpp(28).add(28)); let q1_ = cx.li(24); let q2_ = 2; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(3, t); }
                { let t = ({ let q0_ = cx.lptr(36); let q1_ = cx.lpp(5); let q2_ = cx.lpp(4); let q3_ = cx.lpp(3); f_3a8f30(cx, q0_, q1_, q2_, q3_) } as i32); cx.set_li(8, t); }
                if (cx.li(8) != 0) {
                    { let t = (cx.li(108).wrapping_add(1) << 8); cx.set_li(4, t); }
                    if (cx.i(cx.ptr_at(cx.lpp(28).add(28)).add(12)) != 0) {
                        { let p = cx.ptr_at(cx.ptr_at(cx.lpp(28).add(28)).add(24)).add((8_i32).wrapping_mul(cx.li(24))).add(6); let t = cx.u16(p).bitor_(((cx.li(4) >> 0) & 65535)); cx.set_u16(p, t); }
                    } else {
                        { let p = cx.ptr_at(cx.ptr_at(cx.lpp(28).add(28)).add(20)).add((16_i32).wrapping_mul(cx.li(24))).add(12); let t = cx.i(p).bitor_(cx.li(4)); cx.set_i(p, t); }
                    }
                }
            }
            let _ = cx.preinc_i(cx.lptr(28));
        }
        v5 = (v5 & !0xFFFF_FFFFi64) | ((cx.li(108)) as i32 as u32 as i64);
    }
    v7 = v5;
    return v7;
}

/// PhysicsManager__1B3850 0x001B3850
pub fn f_1b3850(cx: &mut Cx, this: Ptr, a2: i32) -> i64 {
    cx.push_frame(20);
    let r = f_1b3850_body(cx, this, a2);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1b3850_body(cx: &mut Cx, this: Ptr, a2: i32) -> i64 {
    let mut i = 0i32;
    let mut v2 = 0i64;
    let mut v4 = 0i64;
    let mut v8 = Ptr::NULL;
    let mut FaceIndex = 0i32;
    v8 = this;
    v2 = (v2 & !0xFFFF_FFFFi64) | (((28_i32).wrapping_mul(a2)) as i32 as u32 as i64);
    v2 = (v2 & 0xFFFF_FFFFi64) | ((((cx.b(this.add((28_i32).wrapping_mul(a2)).add(36)) as u8 as i32)) as u32 as i64) << 32);
    if (cx.b(this.add((28_i32).wrapping_mul(a2)).add(36)) != 0) {
        { let p = v8.add((28_i32).wrapping_mul(a2).wrapping_add(36)); let t = 0; cx.set_b(p, t); }
        v2 = (v2 & 0xFFFF_FFFFi64) | ((((({ let q0_ = cx.ptr_at(v8.add(32)); let q1_ = cx.lptr(0); let q2_ = cx.lptr(12); let q3_ = cx.lptr(0); let q4_ = v8.add((28_i32).wrapping_mul(a2).wrapping_add(40)); let q5_ = 0; let q6_ = Ptr::NULL; ov_qt_find_leaf(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) } >> 32i64) as i32)) as u32 as i64) << 32);
        i = 0;
        'lp2: loop {
            'blp2: {
                v2 = (v2 & !0xFFFF_FFFFi64) | ((i) as i32 as u32 as i64);
                if (i >= cx.li(0)) {
                    break 'lp2;
                }
                FaceIndex = { let q0_ = cx.ptr_at(v8.add(32)); let q1_ = i.wrapping_add(cx.li(12)); ov_qt_face_index(cx, q0_, q1_) };
                if (cx.i(cx.ptr_at(v8.add(28)).add(12)) != 0) {
                    { let p = cx.ptr_at(cx.ptr_at(v8.add(28)).add(24)).add((8_i32).wrapping_mul(FaceIndex)).add(6); let t = (cx.u16(cx.ptr_at(cx.ptr_at(v8.add(28)).add(24)).add((8_i32).wrapping_mul(FaceIndex)).add(6)) as u8 as i32); cx.set_u16(p, t); }
                } else {
                    { let p = cx.ptr_at(cx.ptr_at(v8.add(28)).add(20)).add((16_i32).wrapping_mul(FaceIndex)).add(12); let t = cx.i(p).bitand_(-65281); cx.set_i(p, t); }
                }
                v2 = (v2 & 0xFFFF_FFFFi64) | (((i.wrapping_add(1)) as u32 as i64) << 32);
            }
            { i += 1; i };
        }
    }
    v4 = v2;
    return v4;
}

/// PhysicsManager__1B39B0 0x001B39B0
pub fn f_1b39b0(cx: &mut Cx, this: Ptr, a2: Ptr, a3: Ptr) -> i64 {
    cx.push_frame(124);
    let r = f_1b39b0_body(cx, this, a2, a3);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1b39b0_body(cx: &mut Cx, this: Ptr, a2: Ptr, a3: Ptr) -> i64 {
    let mut v3 = 0i64;
    let mut v5 = 0i64;
    { let t = this; cx.set_lpp(30, t); }
    { let q0_ = cx.lptr(52); ov_identity_p(cx, q0_) };
    { let q0_ = cx.lptr(52); ov_mat_identity(cx, q0_) };
    { let t = cx.i(a2.add(1332)); cx.set_li(52, t); }
    { let t = cx.i(a2.add(1336)); cx.set_li(56, t); }
    { let t = cx.i(a2.add(1340)); cx.set_li(60, t); }
    { let t = cx.i(a2.add(1344)); cx.set_li(68, t); }
    { let t = cx.i(a2.add(1348)); cx.set_li(72, t); }
    { let t = cx.i(a2.add(1352)); cx.set_li(76, t); }
    { let t = cx.i(a2.add(1356)); cx.set_li(84, t); }
    { let t = cx.i(a2.add(1360)); cx.set_li(88, t); }
    { let t = cx.i(a2.add(1364)); cx.set_li(92, t); }
    { let q0_ = cx.lptr(28); ov_identity_p(cx, q0_) };
    { let q0_ = cx.lptr(28); let q1_ = cx.i(a2.add(236)); let q2_ = cx.i(a2.add(240)); let q3_ = cx.i(a2.add(244)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
    { let q0_ = cx.lptr(4); ov_identity_p(cx, q0_) };
    { let q0_ = cx.lptr(52); let q1_ = cx.lptr(28); let q2_ = cx.lptr(4); f_2abe80(cx, q0_, q1_, q2_) };
    v3 = (v3 & !0xFFFF_FFFFi64) | (({ let q0_ = a3; let q1_ = cx.lptr(4); ov_vec4_copy(cx, q0_, q1_) }.bits()) as i32 as u32 as i64);
    v5 = v3;
    return v5;
}

/// PhysicsManager__1B41F0 0x001B41F0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b41f0(cx: &mut Cx, this: Ptr) -> i32 {
    return ((cx.i(this.add(36)) != 0) as i32);
}

/// PhysicsManager__1B4210 0x001B4210
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b4210(cx: &mut Cx, mut this: Ptr) -> Ptr {
    let _ = ov_identity_p(cx, this.add(8));
    let _ = ov_identity_p(cx, this.add(24));
    { let p = this; let t = -1; cx.set_i(p, t); }
    { let p = this.add(4); let t = 0; cx.set_i(p, t); }
    let _ = ov_vec4_set(cx, this.add(8), 0, 0, 0, 1065353216);
    let _ = ov_vec4_set(cx, this.add(24), 0, 0, 0, 1065353216);
    { let p = this.add(40); let t = 0; cx.set_i(p, t); }
    { let p = this.add(44); let t = 0; cx.set_i(p, t); }
    { let p = this.add(48); let t = 0; cx.set_i(p, t); }
    return this;
}

/// PhysicsManager__1B42B0 0x001B42B0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b42b0(cx: &mut Cx, this: Ptr, mut a2: Ptr) -> Ptr {
    { let p = a2; let t = cx.i(this.add(32)); cx.set_i(p, t); }
    { let p = a2.add(4); let t = cx.i(this.add(36)); cx.set_i(p, t); }
    { let p = a2.add(8); let t = cx.i(this.add(40)); cx.set_i(p, t); }
    return a2;
}

/// PhysicsManager__1B42F0 0x001B42F0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b42f0(cx: &mut Cx, mut this: Ptr) -> Ptr {
    { let p = this; let t = 1036831949; cx.set_i(p, t); }
    { let p = this.add(48); let t = 1008981770; cx.set_i(p, t); }
    { let p = this.add(52); let t = 1036831949; cx.set_i(p, t); }
    { let p = this.add(56); let t = 1008981770; cx.set_i(p, t); }
    let _ = f_19d020(cx, this.add(12));
    { let p = this.add(60); let t = 2139095039; cx.set_i(p, t); }
    { let p = this.add(64); let t = 1050253722; cx.set_i(p, t); }
    { let p = this.add(68); let t = 0; cx.set_i(p, t); }
    { let p = this.add(72); let t = 0; cx.set_i(p, t); }
    { let p = this.add(76); let t = 1086324736; cx.set_i(p, t); }
    { let p = this.add(80); let t = 1; cx.set_i(p, t); }
    { let p = this.add(84); let t = 1066192077; cx.set_i(p, t); }
    { let p = this.add(4); let t = 1069547520; cx.set_i(p, t); }
    { let p = this.add(8); let t = 0; cx.set_i(p, t); }
    { let p = this.add(88); let t = 1; cx.set_i(p, t); }
    { let p = this.add(92); let t = 1; cx.set_i(p, t); }
    { let p = this.add(96); let t = 1; cx.set_i(p, t); }
    { let p = this.add(100); let t = 1; cx.set_i(p, t); }
    { let p = this.add(104); let t = 1; cx.set_i(p, t); }
    { let p = this.add(108); let t = 1; cx.set_i(p, t); }
    { let p = this.add(112); let t = 1; cx.set_i(p, t); }
    { let p = this.add(116); let t = 1; cx.set_i(p, t); }
    { let p = this.add(120); let t = 0; cx.set_i(p, t); }
    { let p = this.add(124); let t = 1; cx.set_i(p, t); }
    let _ = f_1a0360(cx, this.add(36), 0, 1065353216, 0);
    return this;
}

/// PhysicsManager__1B4460 0x001B4460
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b4460(cx: &mut Cx, this: Ptr) -> i32 {
    let mut i = 0i32;
    let mut result = 0i32;
    i = 1;
    'lp2: loop {
        if !(i < cx.i(this.add(4))) { break 'lp2; }
        'blp2: {
            { let q0_ = cx.ptr_at(this).add((132_i32).wrapping_mul(i)); f_1b44c0(cx, q0_) };
            result = i.wrapping_add(1);
        }
        { i += 1; i };
    }
    return result;
}

/// PhysicsManager__1B44C0 0x001B44C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b44c0(cx: &mut Cx, this: Ptr) -> Ptr {
    let mut i = 0i32;
    let mut j = 0i32;
    let mut k = 0i32;
    let mut result = Ptr::NULL;
    { let q0_ = this; let q1_ = this.add(116); let q2_ = cx.ptr_at(this.add(64)).add(16); f_1a6fa0(cx, q0_, q1_, q2_) };
    result = { let q0_ = this; let q1_ = cx.ptr_at(this.add(64)); f_1a3880(cx, q0_, q1_) };
    i = 0;
    'lp4: loop {
        if !(i < 3) { break 'lp4; }
        'blp4: {
            j = 0;
            'lp5: loop {
                if !(j < 3) { break 'lp5; }
                'blp5: {
                    { let p = this.add((12_i32).wrapping_mul(i)).add(16).add((4_i32).wrapping_mul(j)); let t = 0; cx.set_i(p, t); }
                    k = 0;
                    'lp6: loop {
                        if !(k < 3) { break 'lp6; }
                        'blp6: {
                            { let p = this.add((12_i32).wrapping_mul(i)).add(16).add((4_i32).wrapping_mul(j)); let t = ((((cx.f(this.add((16_i32).wrapping_mul(i)).add(68).add((4_i32).wrapping_mul(k))) * cx.f(cx.ptr_at(this.add(64)).add((12_i32).wrapping_mul(k)).add(16).add((4_i32).wrapping_mul(j)))) as f64) + (cx.f(this.add((12_i32).wrapping_mul(i)).add(16).add((4_i32).wrapping_mul(j))) as f64)) as f32); cx.set_f(p, t); }
                        }
                        { k += 1; k };
                    }
                }
                { j += 1; j };
            }
        }
        { i += 1; i };
    }
    return result;
}

/// PhysicsManager__1B45F0 0x001B45F0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b45f0(cx: &mut Cx, this: Ptr, a2: i32, a3: i32) -> i32 {
    if (cx.i(this.add(12)) != 0) {
        return cx.u16(cx.ptr_at(this.add(24)).add((8_i32).wrapping_mul(a2)).add((2_i32).wrapping_mul(a3)));
    } else {
        return cx.i(cx.ptr_at(this.add(20)).add((16_i32).wrapping_mul(a2)).add((4_i32).wrapping_mul(a3)));
    }
    #[allow(unreachable_code)]
    0
}

/// PhysicsManager__1B4640 0x001B4640
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b4640(cx: &mut Cx) -> i32 {
    return cx.i(Ptr::glob(0x6D245C));
}

/// PhysicsManager__1B4650 0x001B4650
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b4650(cx: &mut Cx) -> i32 {
    return cx.i(Ptr::glob(0x6D2458));
}

/// PhysicsManager__1B4660 0x001B4660
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b4660(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = f_1b4690(cx, this);
    return this;
}

/// PhysicsManager__1B4690 0x001B4690
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b4690(cx: &mut Cx, this: Ptr) -> Ptr {
    { let p = this.add(12); let t = 2139095039; cx.set_i(p, t); }
    return this;
}

/// PhysicsManager__1B46C0 0x001B46C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b46c0(cx: &mut Cx, this: Ptr) -> f32 {
    return { let q0_ = cx.ptr_at(this.add(136)); ov_vec4_w(cx, q0_) };
}

/// PhysicsManager__1B46F0 0x001B46F0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b46f0(cx: &mut Cx, this: Ptr) -> i32 {
    return cx.i(cx.ptr_at(this.add(136)).add(36));
}

/// PhysicsManager__1B4730 0x001B4730
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b4730(cx: &mut Cx, mut this: Ptr, a2: i32) -> Ptr {
    let mut v3 = Ptr::NULL;
    let mut v4 = Ptr::NULL;
    v4 = ov_new(cx, (132_i32).wrapping_mul(a2));
    if (v4.bits() != 0) {
        ov_vector_ctor(cx, v4, 132, a2, Ptr::raw(Ptr::func(0x1B47D0).bits()));
        v3 = v4;
    } else {
        v3 = Ptr::NULL;
    }
    { let p = this; let t = v3; cx.set_ptr(p, t); }
    { let p = this.add(4); let t = a2; cx.set_i(p, t); }
    { let p = cx.ptr_at(this).add(64); let t = 0; cx.set_i(p, t); }
    return this;
}

/// PhysicsManager__1B47D0 0x001B47D0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b47d0(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = f_1b4660(cx, this);
    let _ = f_1b1170(cx, this.add(16));
    let _ = f_1b10b0(cx, this.add(68));
    { let p = this.add(64); let t = 0; cx.set_i(p, t); }
    return this;
}

/// PhysicsManager__1B4820 0x001B4820
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b4820(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = f_1b4690(cx, this);
    return this;
}

/// PhysicsManager__1B5010 0x001B5010
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b5010(cx: &mut Cx, this: Ptr) -> Ptr {
    return f_1b51c0(cx, this);
}

/// PhysicsManager__1B51C0 0x001B51C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1b51c0(cx: &mut Cx, mut this: Ptr) -> Ptr {
    { let q0_ = cx.ptr_at(this.add(8)); ov_delete(cx, q0_) };
    { let p = this.add(8); let t = 0; cx.set_i(p, t); }
    { let q0_ = cx.ptr_at(this.add(12)); ov_delete(cx, q0_) };
    { let p = this.add(12); let t = 0; cx.set_i(p, t); }
    { let q0_ = cx.ptr_at(this.add(16)); ov_delete(cx, q0_) };
    { let p = this.add(16); let t = 0; cx.set_i(p, t); }
    { let q0_ = cx.ptr_at(this.add(20)); ov_delete(cx, q0_) };
    { let p = this.add(20); let t = 0; cx.set_i(p, t); }
    { let p = this.add(4); let t = 0; cx.set_i(p, t); }
    { let p = this.add(40); let t = 0; cx.set_i(p, t); }
    { let p = this.add(28); let t = 0; cx.set_i(p, t); }
    { let p = this.add(36); let t = 0; cx.set_i(p, t); }
    { let p = this.add(24); let t = 0; cx.set_i(p, t); }
    { let p = this.add(32); let t = 0; cx.set_i(p, t); }
    { let p = this; let t = 0; cx.set_i(p, t); }
    return this;
}
