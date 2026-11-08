//! Translated from the decompilation by tools/phys_c2rs4.py (do not edit by hand; fixes go in the
//! translator or in the hand-written overrides).
#![allow(non_snake_case, unused_parens, unused_labels, unused_braces, unused_variables, unused_mut, unused_assignments, unreachable_code, dead_code, clippy::all)]

use crate::tr_prelude::*;

/// Armco__13EA0 0x00013EA0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_13ea0(cx: &mut Cx, this: Ptr) -> Ptr {
    ov_vector_ctor(cx, this, 4, 3, Ptr::raw(Ptr::func(0x13F00).bits()));
    ov_vector_ctor(cx, this.add(12), 4, 3, Ptr::raw(Ptr::func(0x13F20).bits()));
    let _ = f_13f20(cx, this.add(24));
    return this;
}

/// Armco__13F00 0x00013F00
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_13f00(cx: &mut Cx, this: Ptr) -> Ptr {
    return this;
}

/// Armco__13F20 0x00013F20
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_13f20(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = ov_identity_p(cx, this);
    return this;
}

/// Armco__14000 0x00014000
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_14000(cx: &mut Cx, this: Ptr, a2: Ptr) -> i32 {
    let mut v2 = Ptr::NULL;
    let mut result = 0i32;
    v2 = this.add(48);
    { let p = v2; let t = cx.i(a2); cx.set_i(p, t); }
    { let p = v2.add(4); let t = cx.i(a2.add(4)); cx.set_i(p, t); }
    { let p = v2.add(8); let t = cx.i(a2.add(8)); cx.set_i(p, t); }
    result = cx.i(a2.add(12));
    { let p = v2.add(12); let t = result; cx.set_i(p, t); }
    return result;
}

/// VehicleDatabaseVehicleBinLoading__18CB00 0x0018CB00
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_18cb00(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = f_1b77e0(cx, this);
    return this;
}

/// MaterialProperties::Get 0x0019C1A0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_19c1a0(cx: &mut Cx, a1: i32) -> Ptr {
    let mut v1 = 0i32;
    if (a1 >= 26) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            v1 = crate::ov_zero_i!(cx, "type >= 0 && type < eSURF_TYPE_MAX_TYPES", ".\\src\\Physics\\MaterialProperties.cpp", 108);
            if (v1 == 3) {
            }
        } else {
        }
    }
    return Ptr::glob(0x6879C0).add((12_i32).wrapping_mul(a1));
}

/// MaterialProperties__19C390 0x0019C390
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_19c390(cx: &mut Cx, this: Ptr) -> i32 {
    let mut i = 0i32;
    let mut j = 0i32;
    let mut result = 0i32;
    result = { { let q0_ = cx.ptr_at(this.add(24)); ov_reset_all_objects(cx, q0_) }; 0 };
    { let p = this.add(24328); let t = 1065353216; cx.set_i(p, t); }
    { let p = this.add(16); let t = 1065353216; cx.set_i(p, t); }
    i = 0;
    'lp3: loop {
        if !(i < 100) { break 'lp3; }
        'blp3: {
            if (cx.b(this.add((28_i32).wrapping_mul(i)).add(36)) != 0) {
                let _ = f_1b3850(cx, this, i);
            }
            result = i.wrapping_add(1);
        }
        { i += 1; i };
    }
    j = 0;
    'lp4: loop {
        if !(j < 30) { break 'lp4; }
        'blp4: {
            result = (712_i32).wrapping_mul(j);
            { let p = this.add(4 * ((178_i32).wrapping_mul(j))).add(3384); let t = 0; cx.set_i(p, t); }
        }
        { j += 1; j };
    }
    return result;
}

/// MaterialProperties__19D020 0x0019D020
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_19d020(cx: &mut Cx, mut a1: Ptr) -> Ptr {
    { let p = a1; let t = 0; cx.set_i(p, t); }
    { let p = a1.add(4); let t = 0; cx.set_i(p, t); }
    { let p = a1.add(8); let t = 0; cx.set_i(p, t); }
    return a1;
}

/// MaterialProperties__19D310 0x0019D310
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_19d310(cx: &mut Cx, this: Ptr, a2: Ptr, a3: Ptr, a4: Ptr, a5: i32, a6: i32) -> i64 {
    return f_19d350(cx, this, a2, a3, a4, a5, a6);
}

/// MaterialProperties__19D350 0x0019D350
pub fn f_19d350(cx: &mut Cx, this: Ptr, a2: Ptr, a3: Ptr, a4: Ptr, a5: i32, a6: i32) -> i64 {
    cx.push_frame(24);
    let r = f_19d350_body(cx, this, a2, a3, a4, a5, a6);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_19d350_body(cx: &mut Cx, this: Ptr, a2: Ptr, a3: Ptr, a4: Ptr, a5: i32, a6: i32) -> i64 {
    let mut v7 = 0i64;
    cx.memset(cx.lptr(0), 204, 20);
    { let t = this; cx.set_lpp(5, t); }
    v7 = { let q0_ = this; let q1_ = a2.bits(); let q2_ = a2; let q3_ = cx.lptr(0); let q4_ = a3; let q5_ = a4; let q6_ = a6; let q7_ = (if (a5 != 0) { 23 } else { 17 }); f_19d3e0(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_) };
    return v7;
}

/// VsAIRaceVehicle__1D660 0x0001D660
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d660(cx: &mut Cx, a1: Ptr, a2: Ptr) -> f32 {
    return (((((cx.f(a1) as f64) * (cx.f(a2) as f64)) + ((cx.f(a1.add(4)) as f64) * (cx.f(a2.add(4)) as f64))) + ((cx.f(a1.add(8)) as f64) * (cx.f(a2.add(8)) as f64))) as f32);
}

/// VsAIRaceVehicle__23590 0x00023590
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_23590(cx: &mut Cx, this: Ptr) -> Ptr {
    return this;
}

/// VsAIRaceVehicle__238B0 0x000238B0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_238b0(cx: &mut Cx, mut this: Ptr) -> Ptr {
    let _ = ov_identity_p(cx, this.add(8));
    let _ = ov_identity_p(cx, this.add(24));
    let _ = ov_identity_p(cx, this.add(56));
    { let p = this; let t = -1; cx.set_i(p, t); }
    { let p = this.add(4); let t = -1; cx.set_i(p, t); }
    let _ = ov_vec4_set(cx, this.add(8), 0, 0, 0, 1065353216);
    let _ = ov_vec4_set(cx, this.add(24), 0, 0, 0, 1065353216);
    { let p = this.add(40); let t = 0; cx.set_i(p, t); }
    { let p = this.add(44); let t = 0; cx.set_i(p, t); }
    { let p = this.add(48); let t = 0; cx.set_i(p, t); }
    { let p = this.add(52); let t = 0; cx.set_i(p, t); }
    { let p = this.add(72); let t = 0; cx.set_i(p, t); }
    { let p = this.add(76); let t = 0; cx.set_b(p, t); }
    return this;
}

/// VsAIRaceVehicle__24220 0x00024220
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_24220(cx: &mut Cx, this: Ptr) -> f32 {
    return cx.f(this.add(20));
}

/// VsAIRaceVehicle__24530 0x00024530
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_24530(cx: &mut Cx, this: Ptr) -> i32 {
    return cx.i(this.add(3036));
}

/// VsAIRaceVehicle__24550 0x00024550
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_24550(cx: &mut Cx, this: Ptr, mut a2: Ptr, mut a3: Ptr) -> Ptr {
    { let p = a2; let t = cx.b(this.add(3529)); cx.set_b(p, t); }
    { let p = a3; let t = cx.b(this.add(3530)); cx.set_b(p, t); }
    return a3;
}

/// VsAIRaceVehicle__24870 0x00024870
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_24870(cx: &mut Cx, this: Ptr) -> f32 {
    let mut v1 = Ptr::NULL;
    let mut v4 = Ptr::NULL;
    v4 = ov_car_velocity(cx, this.add(240));
    v1 = ov_car_rot_row(cx, this.add(240), 0);
    return f_1d660(cx, v1, v4);
}

/// VsAIRaceVehicle__248F0 0x000248F0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_248f0(cx: &mut Cx, this: Ptr) -> f32 {
    return (((cx.f(this.add(96)) as f64) / 24.0f64) as f32);
}

/// Renderer__2ABE80 0x002ABE80
pub fn f_2abe80(cx: &mut Cx, this: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    cx.push_frame(16);
    let r = f_2abe80_body(cx, this, a2, a3);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_2abe80_body(cx: &mut Cx, this: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    let mut v4 = 0.0f32;
    let mut v5 = 0.0f32;
    let mut v6 = 0.0f32;
    let mut v7 = 0.0f32;
    let mut v8 = 0.0f32;
    let mut v9 = 0.0f32;
    let mut v10 = 0.0f32;
    let mut v11 = 0.0f32;
    let mut v12 = 0.0f32;
    { let q0_ = cx.lptr(0); let q1_ = a2; ov_vec4_copy(cx, q0_, q1_) };
    v12 = ((({ let q0_ = cx.lptr(0); ov_vec4_x(cx, q0_) } as f64) * (cx.f(this) as f64)) as f32);
    v11 = (((({ let q0_ = cx.lptr(0); ov_vec4_y(cx, q0_) } as f64) * (cx.f(this.add(16)) as f64)) + (v12 as f64)) as f32);
    v4 = (((({ let q0_ = cx.lptr(0); ov_vec4_z(cx, q0_) } as f64) * (cx.f(this.add(32)) as f64)) + (v11 as f64)) as f32);
    let _ = ov_vec4_setx(cx, a3, (v4.to_bits() as i32));
    v10 = ((({ let q0_ = cx.lptr(0); ov_vec4_x(cx, q0_) } as f64) * (cx.f(this.add(4)) as f64)) as f32);
    v9 = (((({ let q0_ = cx.lptr(0); ov_vec4_y(cx, q0_) } as f64) * (cx.f(this.add(20)) as f64)) + (v10 as f64)) as f32);
    v5 = (((({ let q0_ = cx.lptr(0); ov_vec4_z(cx, q0_) } as f64) * (cx.f(this.add(36)) as f64)) + (v9 as f64)) as f32);
    let _ = ov_vec4_sety(cx, a3, (v5.to_bits() as i32));
    v8 = ((({ let q0_ = cx.lptr(0); ov_vec4_x(cx, q0_) } as f64) * (cx.f(this.add(8)) as f64)) as f32);
    v7 = (((({ let q0_ = cx.lptr(0); ov_vec4_y(cx, q0_) } as f64) * (cx.f(this.add(24)) as f64)) + (v8 as f64)) as f32);
    v6 = (((({ let q0_ = cx.lptr(0); ov_vec4_z(cx, q0_) } as f64) * (cx.f(this.add(40)) as f64)) + (v7 as f64)) as f32);
    let _ = ov_vec4_setz(cx, a3, (v6.to_bits() as i32));
    return ov_vec4_setw(cx, a3, 1065353216);
}

/// Renderer__2ABF70 0x002ABF70
pub fn f_2abf70(cx: &mut Cx, this: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    cx.push_frame(16);
    let r = f_2abf70_body(cx, this, a2, a3);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_2abf70_body(cx: &mut Cx, this: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    let mut v4 = 0.0f32;
    let mut v5 = 0.0f32;
    let mut v6 = 0.0f32;
    let mut v7 = 0.0f32;
    let mut v8 = 0.0f32;
    let mut v9 = 0.0f32;
    let mut v10 = 0.0f32;
    let mut v11 = 0.0f32;
    let mut v12 = 0.0f32;
    { let q0_ = cx.lptr(0); ov_identity_p(cx, q0_) };
    v12 = (((ov_vec4_x(cx, a3) as f64) * (cx.f(this) as f64)) as f32);
    v11 = ((((ov_vec4_y(cx, a3) as f64) * (cx.f(this.add(16)) as f64)) + (v12 as f64)) as f32);
    v4 = ((((ov_vec4_z(cx, a3) as f64) * (cx.f(this.add(32)) as f64)) + (v11 as f64)) as f32);
    { let q0_ = cx.lptr(0); let q1_ = (v4.to_bits() as i32); ov_vec4_setx(cx, q0_, q1_) };
    v10 = (((ov_vec4_x(cx, a3) as f64) * (cx.f(this.add(4)) as f64)) as f32);
    v9 = ((((ov_vec4_y(cx, a3) as f64) * (cx.f(this.add(20)) as f64)) + (v10 as f64)) as f32);
    v5 = ((((ov_vec4_z(cx, a3) as f64) * (cx.f(this.add(36)) as f64)) + (v9 as f64)) as f32);
    { let q0_ = cx.lptr(0); let q1_ = (v5.to_bits() as i32); ov_vec4_sety(cx, q0_, q1_) };
    v8 = (((ov_vec4_x(cx, a3) as f64) * (cx.f(this.add(8)) as f64)) as f32);
    v7 = ((((ov_vec4_y(cx, a3) as f64) * (cx.f(this.add(24)) as f64)) + (v8 as f64)) as f32);
    v6 = ((((ov_vec4_z(cx, a3) as f64) * (cx.f(this.add(40)) as f64)) + (v7 as f64)) as f32);
    { let q0_ = cx.lptr(0); let q1_ = (v6.to_bits() as i32); ov_vec4_setz(cx, q0_, q1_) };
    { let q0_ = cx.lptr(0); let q1_ = 1065353216; ov_vec4_setw(cx, q0_, q1_) };
    { let q0_ = a2; let q1_ = cx.lptr(0); ov_vec4_copy(cx, q0_, q1_) };
    return a2;
}

/// Renderer__2AC0C0 0x002AC0C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_2ac0c0(cx: &mut Cx, mut this: Ptr, a2: Ptr) -> Ptr {
    let mut v3 = 0.0f32;
    let mut v4 = 0.0f32;
    let mut v5 = 0.0f32;
    let mut v6 = 0.0f32;
    let mut v7 = 0.0f32;
    let mut v8 = 0.0f32;
    let mut v9 = 0.0f32;
    let mut v10 = 0.0f32;
    let mut v11 = 0.0f32;
    let mut v12 = 0.0f32;
    let mut v13 = 0.0f32;
    let mut v14 = 0.0f32;
    v3 = ((2.0f64 * (cx.f(a2) as f64)) as f32);
    v12 = ((2.0f64 * (cx.f(a2.add(4)) as f64)) as f32);
    v7 = ((2.0f64 * (cx.f(a2.add(8)) as f64)) as f32);
    v13 = (((v3 as f64) * (cx.f(a2.add(12)) as f64)) as f32);
    v4 = (((v12 as f64) * (cx.f(a2.add(12)) as f64)) as f32);
    v6 = (((v7 as f64) * (cx.f(a2.add(12)) as f64)) as f32);
    v8 = (((v3 as f64) * (cx.f(a2) as f64)) as f32);
    v9 = (((v12 as f64) * (cx.f(a2) as f64)) as f32);
    v11 = (((v7 as f64) * (cx.f(a2) as f64)) as f32);
    v10 = (((v12 as f64) * (cx.f(a2.add(4)) as f64)) as f32);
    v5 = (((v7 as f64) * (cx.f(a2.add(4)) as f64)) as f32);
    v14 = (((v7 as f64) * (cx.f(a2.add(8)) as f64)) as f32);
    { let p = this; let t = ((1.0f64 - ((v10 as f64) + (v14 as f64))) as f32); cx.set_f(p, t); }
    { let p = this.add(4); let t = (((v9 as f64) - (v6 as f64)) as f32); cx.set_f(p, t); }
    { let p = this.add(8); let t = (((v11 as f64) + (v4 as f64)) as f32); cx.set_f(p, t); }
    { let p = this.add(12); let t = 0.0f32; cx.set_f(p, t); }
    { let p = this.add(16); let t = (((v9 as f64) + (v6 as f64)) as f32); cx.set_f(p, t); }
    { let p = this.add(20); let t = ((1.0f64 - ((v8 as f64) + (v14 as f64))) as f32); cx.set_f(p, t); }
    { let p = this.add(24); let t = (((v5 as f64) - (v13 as f64)) as f32); cx.set_f(p, t); }
    { let p = this.add(28); let t = 0.0f32; cx.set_f(p, t); }
    { let p = this.add(32); let t = (((v11 as f64) - (v4 as f64)) as f32); cx.set_f(p, t); }
    { let p = this.add(36); let t = (((v5 as f64) + (v13 as f64)) as f32); cx.set_f(p, t); }
    { let p = this.add(40); let t = ((1.0f64 - ((v8 as f64) + (v10 as f64))) as f32); cx.set_f(p, t); }
    { let p = this.add(44); let t = 0.0f32; cx.set_f(p, t); }
    return this;
}

/// Renderer__2AC2C0 0x002AC2C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_2ac2c0(cx: &mut Cx, mut this: Ptr, a2: Ptr) -> Ptr {
    let mut v3 = 0.0f32;
    let mut v4 = 0.0f32;
    let mut v5 = 0.0f32;
    let mut v6 = 0.0f32;
    let mut v7 = 0.0f32;
    let mut v8 = 0.0f32;
    let mut v9 = 0.0f32;
    let mut v10 = 0.0f32;
    let mut v11 = 0.0f32;
    let mut v12 = 0.0f32;
    let mut v13 = 0.0f32;
    let mut v14 = 0.0f32;
    v8 = ((2.0f64 * (cx.f(a2) as f64)) as f32);
    v5 = ((2.0f64 * (cx.f(a2.add(4)) as f64)) as f32);
    v9 = ((2.0f64 * (cx.f(a2.add(8)) as f64)) as f32);
    v6 = (((v8 as f64) * (cx.f(a2.add(12)) as f64)) as f32);
    v10 = (((v5 as f64) * (cx.f(a2.add(12)) as f64)) as f32);
    v11 = (((v9 as f64) * (cx.f(a2.add(12)) as f64)) as f32);
    v14 = (((v8 as f64) * (cx.f(a2) as f64)) as f32);
    v4 = (((v5 as f64) * (cx.f(a2) as f64)) as f32);
    v12 = (((v9 as f64) * (cx.f(a2) as f64)) as f32);
    v3 = (((v5 as f64) * (cx.f(a2.add(4)) as f64)) as f32);
    v13 = (((v9 as f64) * (cx.f(a2.add(4)) as f64)) as f32);
    v7 = (((v9 as f64) * (cx.f(a2.add(8)) as f64)) as f32);
    { let p = this; let t = ((1.0f64 - ((v3 as f64) + (v7 as f64))) as f32); cx.set_f(p, t); }
    { let p = this.add(4); let t = (((v4 as f64) - (v11 as f64)) as f32); cx.set_f(p, t); }
    { let p = this.add(8); let t = (((v12 as f64) + (v10 as f64)) as f32); cx.set_f(p, t); }
    { let p = this.add(12); let t = 0.0f32; cx.set_f(p, t); }
    { let p = this.add(16); let t = (((v4 as f64) + (v11 as f64)) as f32); cx.set_f(p, t); }
    { let p = this.add(20); let t = ((1.0f64 - ((v14 as f64) + (v7 as f64))) as f32); cx.set_f(p, t); }
    { let p = this.add(24); let t = (((v13 as f64) - (v6 as f64)) as f32); cx.set_f(p, t); }
    { let p = this.add(28); let t = 0.0f32; cx.set_f(p, t); }
    { let p = this.add(32); let t = (((v12 as f64) - (v10 as f64)) as f32); cx.set_f(p, t); }
    { let p = this.add(36); let t = (((v13 as f64) + (v6 as f64)) as f32); cx.set_f(p, t); }
    { let p = this.add(40); let t = ((1.0f64 - ((v14 as f64) + (v3 as f64))) as f32); cx.set_f(p, t); }
    { let p = this.add(44); let t = 0.0f32; cx.set_f(p, t); }
    { let p = this.add(56); let t = 0.0f32; cx.set_f(p, t); }
    { let p = this.add(52); let t = 0.0f32; cx.set_f(p, t); }
    { let p = this.add(48); let t = 0.0f32; cx.set_f(p, t); }
    { let p = this.add(60); let t = 1.0f32; cx.set_f(p, t); }
    return this;
}

/// TimelineSequencer__2B7330 0x002B7330
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_2b7330(cx: &mut Cx, a1: f32, a2: Ptr, a3: Ptr, mut a4: Ptr) -> Ptr {
    let mut v5 = 0.0f32;
    v5 = ((1.0f64 - (a1 as f64)) as f32);
    { let p = a4; let t = ((((v5 as f64) * (cx.f(a2) as f64)) + ((a1 as f64) * (cx.f(a3) as f64))) as f32); cx.set_f(p, t); }
    { let p = a4.add(4); let t = ((((v5 as f64) * (cx.f(a2.add(4)) as f64)) + ((a1 as f64) * (cx.f(a3.add(4)) as f64))) as f32); cx.set_f(p, t); }
    { let p = a4.add(8); let t = ((((v5 as f64) * (cx.f(a2.add(8)) as f64)) + ((a1 as f64) * (cx.f(a3.add(8)) as f64))) as f32); cx.set_f(p, t); }
    { let p = a4.add(12); let t = ((((v5 as f64) * (cx.f(a2.add(12)) as f64)) + ((a1 as f64) * (cx.f(a3.add(12)) as f64))) as f32); cx.set_f(p, t); }
    return a2;
}

/// TimelineSequencer__2B73B0 0x002B73B0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_2b73b0(cx: &mut Cx, a1: Ptr, mut a2: Ptr) -> i32 {
    { let p = a2; let t = (-cx.f(a1)); cx.set_f(p, t); }
    { let p = a2.add(4); let t = (-cx.f(a1.add(4))); cx.set_f(p, t); }
    { let p = a2.add(8); let t = (-cx.f(a1.add(8))); cx.set_f(p, t); }
    { let p = a2.add(12); let t = cx.i(a1.add(12)); cx.set_i(p, t); }
    return a2.bits();
}

/// TimelineSequencer__2B7490 0x002B7490
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_2b7490(cx: &mut Cx, mut this: Ptr) -> () {
    let mut v2 = 0.0f32;
    let mut v3 = 0.0f32;
    v2 = ((((((cx.f(this) as f64) * (cx.f(this) as f64)) + ((cx.f(this.add(4)) as f64) * (cx.f(this.add(4)) as f64))) + ((cx.f(this.add(8)) as f64) * (cx.f(this.add(8)) as f64))) + ((cx.f(this.add(12)) as f64) * (cx.f(this.add(12)) as f64))) as f32);
    if (v2 != 0.0f32) {
        v3 = (vsmath::sqrt(v2) as f32);
        { let p = this; let t = (((cx.f(this) as f64) / (v3 as f64)) as f32); cx.set_f(p, t); }
        { let p = this.add(4); let t = (((cx.f(this.add(4)) as f64) / (v3 as f64)) as f32); cx.set_f(p, t); }
        { let p = this.add(8); let t = (((cx.f(this.add(8)) as f64) / (v3 as f64)) as f32); cx.set_f(p, t); }
        { let p = this.add(12); let t = (((cx.f(this.add(12)) as f64) / (v3 as f64)) as f32); cx.set_f(p, t); }
    }
}

/// VsAIVehicle__2C6D0 0x0002C6D0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_2c6d0(cx: &mut Cx, this: Ptr, a2: Ptr) -> Ptr {
    let mut v3 = 0.0f32;
    let mut v4 = 0.0f32;
    let mut v5 = 0.0f32;
    v5 = ov_vec4_z(cx, a2);
    v4 = ov_vec4_y(cx, a2);
    v3 = ov_vec4_x(cx, a2);
    return f_36b130(cx, this.add(240), (v3.to_bits() as i32), (v4.to_bits() as i32), (v5.to_bits() as i32));
}

/// CRim__452C0 0x000452C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_452c0(cx: &mut Cx, a1: f32, a2: f32, a3: f32) -> f64 {
    return ((((a2 as f64) - (a1 as f64)) * (a3 as f64)) + (a1 as f64));
}

/// VsFrameTimer__57F90 0x00057F90
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_57f90(cx: &mut Cx, a1: f32, a2: f32) -> f64 {
    if (a2 <= a1) {
        return (a1 as f64);
    } else {
        return (a2 as f64);
    }
    #[allow(unreachable_code)]
    0.0
}

/// VsCameraManager__613E0 0x000613E0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_613e0(cx: &mut Cx, a1: Ptr, a2: Ptr, a3: f32, a4: Ptr) -> Ptr {
    let mut v5 = 0.0f32;
    let mut v6 = 0.0f32;
    let mut v7 = 0.0f32;
    let mut v8 = 0.0f32;
    let mut v9 = 0.0f32;
    let mut v10 = 0.0f32;
    let mut v11 = 0.0f32;
    let mut v12 = 0.0f32;
    let mut v13 = 0.0f32;
    v13 = ov_vec4_x(cx, a1);
    v12 = ov_vec4_x(cx, a2);
    v5 = (((((v12 as f64) - (ov_vec4_x(cx, a1) as f64)) * (a3 as f64)) + (v13 as f64)) as f32);
    let _ = ov_vec4_setx(cx, a4, (v5.to_bits() as i32));
    v11 = ov_vec4_y(cx, a1);
    v10 = ov_vec4_y(cx, a2);
    v6 = (((((v10 as f64) - (ov_vec4_y(cx, a1) as f64)) * (a3 as f64)) + (v11 as f64)) as f32);
    let _ = ov_vec4_sety(cx, a4, (v6.to_bits() as i32));
    v9 = ov_vec4_z(cx, a1);
    v8 = ov_vec4_z(cx, a2);
    v7 = (((((v8 as f64) - (ov_vec4_z(cx, a1) as f64)) * (a3 as f64)) + (v9 as f64)) as f32);
    return ov_vec4_setz(cx, a4, (v7.to_bits() as i32));
}

/// CarDamage__762F0 0x000762F0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_762f0(cx: &mut Cx, a1: Ptr, a2: Ptr, a3: f32, a4: Ptr) -> Ptr {
    let mut v5 = 0.0f32;
    let mut v6 = 0.0f32;
    let mut v7 = 0.0f32;
    let mut v8 = 0.0f32;
    let mut v9 = 0.0f32;
    let mut v10 = 0.0f32;
    let mut v11 = 0.0f32;
    let mut v12 = 0.0f32;
    let mut v13 = 0.0f32;
    let mut v14 = 0.0f32;
    let mut v15 = 0.0f32;
    let mut v16 = 0.0f32;
    v16 = ov_vec4_x(cx, a1);
    v15 = ov_vec4_x(cx, a2);
    v5 = (((((v15 as f64) - (ov_vec4_x(cx, a1) as f64)) * (a3 as f64)) + (v16 as f64)) as f32);
    let _ = ov_vec4_setx(cx, a4, (v5.to_bits() as i32));
    v14 = ov_vec4_y(cx, a1);
    v13 = ov_vec4_y(cx, a2);
    v6 = (((((v13 as f64) - (ov_vec4_y(cx, a1) as f64)) * (a3 as f64)) + (v14 as f64)) as f32);
    let _ = ov_vec4_sety(cx, a4, (v6.to_bits() as i32));
    v12 = ov_vec4_z(cx, a1);
    v11 = ov_vec4_z(cx, a2);
    v7 = (((((v11 as f64) - (ov_vec4_z(cx, a1) as f64)) * (a3 as f64)) + (v12 as f64)) as f32);
    let _ = ov_vec4_setz(cx, a4, (v7.to_bits() as i32));
    v10 = ov_vec4_w(cx, a1);
    v9 = ov_vec4_w(cx, a2);
    v8 = (((((v9 as f64) - (ov_vec4_w(cx, a1) as f64)) * (a3 as f64)) + (v10 as f64)) as f32);
    return ov_vec4_setw(cx, a4, (v8.to_bits() as i32));
}

/// OSD3dInterface__EB090 0x000EB090
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_eb090(cx: &mut Cx, mut this: Ptr, a2: Ptr) -> Ptr {
    { let p = this; let t = cx.i(a2); cx.set_i(p, t); }
    return this;
}
