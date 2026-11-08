//! Translated from the decompilation by tools/phys_c2rs4.py (do not edit by hand; fixes go in the
//! translator or in the hand-written overrides).
#![allow(non_snake_case, unused_parens, unused_labels, unused_braces, unused_variables, unused_mut, unused_assignments, unreachable_code, dead_code, clippy::all)]

use crate::tr_prelude::*;

/// PhysicsObjects__1E58C0 0x001E58C0
pub fn f_1e58c0(cx: &mut Cx, this: Ptr, a2: f32) -> i64 {
    cx.push_frame(192);
    let r = f_1e58c0_body(cx, this, a2);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1e58c0_body(cx: &mut Cx, this: Ptr, a2: f32) -> i64 {
    let mut v2 = 0.0f32;
    let mut v3 = 0.0f32;
    let mut v4 = 0.0f32;
    let mut v5 = 0.0f32;
    let mut v6 = Ptr::NULL;
    let mut v7 = 0i64;
    let mut v9 = 0i64;
    { let t = this; cx.set_lpp(47, t); }
    if ((a2 < 0.0f32) || (a2 > 1.0f32)) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            let _ = crate::ov_zero_i!(cx, "interp >= 0.0f && interp <= 1.0f", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjects.cpp", 827);
            { let t = v2; cx.set_lf(184, t); }
            if ((v2.to_bits() as i32) == 3) {
            }
            { let t = 257; let o = cx.li(4); cx.set_li(4, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
        } else {
            { let t = 1; let o = cx.li(4); cx.set_li(4, (o & !(255 << 16)) | ((t & 255) << 16)); }
        }
    }
    if (cx.i(cx.lpp(47).add(100)) == 0) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            let _ = crate::ov_zero_i!(cx, "m_pMovableObject", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjects.cpp", 829);
            { let t = v3; cx.set_lf(180, t); }
            if ((v3.to_bits() as i32) == 3) {
            }
            { let t = 257; let o = cx.li(4); cx.set_li(4, (o & !(65535 << 0)) | ((t & 65535) << 0)); }
        } else {
            { let t = 1; let o = cx.li(4); cx.set_li(4, (o & !(255 << 0)) | ((t & 255) << 0)); }
        }
    }
    if !({ let q0_ = cx.ptr_at(cx.lpp(47).add(100)); f_1b41f0(cx, q0_) } != 0) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            let _ = crate::ov_zero_i!(cx, "m_pMovableObject->IsAwake()", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjects.cpp", 830);
            { let t = v4; cx.set_lf(176, t); }
            if ((v4.to_bits() as i32) == 3) {
            }
            { let t = 257; let o = cx.li(0); cx.set_li(0, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
        } else {
            { let t = 1; let o = cx.li(0); cx.set_li(0, (o & !(255 << 16)) | ((t & 255) << 16)); }
        }
    }
    if (cx.i(cx.ptr_at(cx.lpp(47).add(100)).add(36)) == 0) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            let _ = crate::ov_zero_i!(cx, "m_pMovableObject->m_pRigidBody", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjects.cpp", 832);
            { let t = v5; cx.set_lf(172, t); }
            if ((v5.to_bits() as i32) == 3) {
            }
            { let t = 257; let o = cx.li(0); cx.set_li(0, (o & !(65535 << 0)) | ((t & 65535) << 0)); }
        } else {
            { let t = 1; let o = cx.li(0); cx.set_li(0, (o & !(255 << 0)) | ((t & 255) << 0)); }
        }
    }
    { let t = cx.ptr_at(cx.ptr_at(cx.lpp(47).add(100)).add(36)); cx.set_lpp(42, t); }
    { let q0_ = cx.lptr(148); let q1_ = cx.i(cx.lpp(42).add(80)); let q2_ = cx.i(cx.lpp(42).add(84)); let q3_ = cx.i(cx.lpp(42).add(88)); let q4_ = cx.i(cx.lpp(42).add(92)); f_1da280(cx, q0_, q1_, q2_, q3_, q4_) };
    { let q0_ = cx.lptr(124); let q1_ = cx.i(cx.lpp(42).add(64)); let q2_ = cx.i(cx.lpp(42).add(68)); let q3_ = cx.i(cx.lpp(42).add(72)); let q4_ = 0; ov_vec4_xyzw(cx, q0_, q1_, q2_, q3_, q4_) };
    { let q0_ = cx.lptr(100); f_23590(cx, q0_) };
    { let q0_ = cx.lptr(76); ov_identity_p(cx, q0_) };
    { let q0_ = cx.lpp(42).add(696); let q1_ = cx.lptr(124); let q2_ = a2; let q3_ = cx.lptr(76); f_613e0(cx, q0_, q1_, q2_, q3_) };
    { let q0_ = cx.lptr(76); let q1_ = 1065353216; ov_vec4_setw(cx, q0_, q1_) };
    { let q0_ = a2; let q1_ = cx.lpp(42).add(680); let q2_ = cx.lptr(148); let q3_ = cx.lptr(100); f_2b7330(cx, q0_, q1_, q2_, q3_) };
    { let q0_ = cx.lptr(100); f_2b7490(cx, q0_) };
    { let q0_ = cx.lptr(52); f_23590(cx, q0_) };
    { let q0_ = cx.lptr(100); let q1_ = cx.lptr(52); f_2b73b0(cx, q0_, q1_) };
    { let q0_ = cx.lpp(47); let q1_ = cx.lptr(52); f_2ac2c0(cx, q0_, q1_) };
    { let q0_ = cx.lptr(28); let q1_ = cx.i(cx.ptr_at(cx.lpp(47).add(100)).add(4)); let q2_ = cx.i(cx.ptr_at(cx.lpp(47).add(100)).add(8)); let q3_ = cx.i(cx.ptr_at(cx.lpp(47).add(100)).add(12)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
    v6 = { let q0_ = cx.lpp(47); let q1_ = cx.lptr(8); let q2_ = cx.lptr(28); f_2abf70(cx, q0_, q1_, q2_) };
    { let q0_ = cx.lptr(76); let q1_ = v6; ov_vec4_subeq(cx, q0_, q1_) };
    v7 = (v7 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.lpp(47); let q1_ = cx.lptr(76); f_14000(cx, q0_, q1_) }) as i32 as u32 as i64);
    v9 = v7;
    return v9;
}

/// PhysicsObjects__1E5CA0 0x001E5CA0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5ca0(cx: &mut Cx, this: Ptr) -> i32 {
    let mut v1 = 0i32;
    if (cx.i(this.add(136)) == 0) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            v1 = crate::ov_zero_i!(cx, "m_pOriginalInstance", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjects.cpp", 865);
            if (v1 == 3) {
            }
        } else {
        }
    }
    return ((cx.i(cx.ptr_at(this.add(136)).add(52)) == 12) as i32);
}

/// PhysicsObjects__1E5D40 0x001E5D40
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5d40(cx: &mut Cx, this: Ptr) -> i32 {
    let mut v1 = 0i32;
    if (cx.i(this.add(136)) == 0) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            v1 = crate::ov_zero_i!(cx, "m_pOriginalInstance", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjects.cpp", 871);
            if (v1 == 3) {
            }
        } else {
        }
    }
    return (((cx.i(cx.ptr_at(this.add(136)).add(52)) == 24) || (cx.i(cx.ptr_at(this.add(136)).add(52)) == 26)) as i32);
}

/// PhysicsObjects__1E5E00 0x001E5E00
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5e00(cx: &mut Cx, this: Ptr, a2: i32) -> i32 {
    let mut v2 = 0i32;
    let mut result = 0i32;
    { let p = this.add(152); let t = cx.i(p).bitor_(4); cx.set_i(p, t); }
    if (f_1e5f30(cx, this, 0) != 0) {
        v2 = f_1e5f60(cx, this);
        { let q0_ = cx.i(Ptr::glob(0x736958)); let q1_ = v2; crate::ov_ignore!(cx, q0_, q1_) };
    }
    result = (((cx.i(this.add(152)) & 8192) != 0) as i32);
    if ((cx.i(this.add(152)) & 8192) != 0) {
        return f_1e5670(cx, this);
    }
    return result;
}

/// PhysicsObjects__1E5E80 0x001E5E80
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5e80(cx: &mut Cx, this: Ptr, a2: i32, a3: i32) -> Ptr {
    let mut result = Ptr::NULL;
    if (a2 < 3) {
        { let p = cx.ptr_at(this.add(136)).add(56); let t = a2; cx.set_u16(p, t); }
        { let p = cx.ptr_at(this.add(136)).add(58); let t = a3; cx.set_u16(p, t); }
        return this;
    } else {
        result = Ptr::raw(0 /*str*/);
        if (0 /*str*/ != Ptr::NULL.bits()) {
            if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
                result = Ptr::raw(crate::ov_zero_i!(cx, "!\"ERROR: CPhysicsObject__SetTypeAndIndex - trying to use ePhysicsObjectType that doesn't exist!!!\"", ".\\src\\Physics\\PhysicsObjects\\PhysicsObjects.cpp", 910));
                if (result.bits() == Ptr::raw(3).bits()) {
                }
            } else {
            }
        }
    }
    return result;
}

/// PhysicsObjects__1E5F30 0x001E5F30
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5f30(cx: &mut Cx, this: Ptr, a2: i32) -> i32 {
    return ((a2 == cx.u16(cx.ptr_at(this.add(136)).add(56))) as i32);
}

/// PhysicsObjects__1E5F60 0x001E5F60
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5f60(cx: &mut Cx, this: Ptr) -> i32 {
    return cx.u16(cx.ptr_at(this.add(136)).add(58));
}

/// PhysicsObjects__1E5F80 0x001E5F80
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5f80(cx: &mut Cx, a1: i32) -> () {

}

/// PhysicsObjects__1E5FA0 0x001E5FA0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5fa0(cx: &mut Cx, a1: i32) -> () {

}

/// PhysicsObjects__1E5FC0 0x001E5FC0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e5fc0(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = f_eb090(cx, this.add(24), Ptr::glob(0x77FD9C));
    { let p = this.add(28); let t = 0; cx.set_i(p, t); }
    return this;
}

/// PhysicsObjects__1E6000 0x001E6000
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e6000(cx: &mut Cx, this: Ptr) -> Ptr {
    { let q0_ = this.add(696); let q1_ = cx.i(this.add(64)); let q2_ = cx.i(this.add(68)); let q3_ = cx.i(this.add(72)); let q4_ = 0; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
    return { let q0_ = this.add(680); let q1_ = cx.i(this.add(80)); let q2_ = cx.i(this.add(84)); let q3_ = cx.i(this.add(88)); let q4_ = cx.i(this.add(92)); ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
}

/// PhysicsObjects__1E6070 0x001E6070
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1e6070(cx: &mut Cx, mut this: Ptr) -> Ptr {
    { let p = this; let t = 0; cx.set_i(p, t); }
    return this;
}
