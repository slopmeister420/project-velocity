//! Translated from the decompilation by tools/phys_c2rs4.py (do not edit by hand; fixes go in the
//! translator or in the hand-written overrides).
#![allow(non_snake_case, unused_parens, unused_labels, unused_braces, unused_variables, unused_mut, unused_assignments, unreachable_code, dead_code, clippy::all)]

use crate::tr_prelude::*;

/// VehicleEntity__1BB020 0x001BB020
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bb020(cx: &mut Cx, this: Ptr, a2: Ptr) -> Ptr {
    return f_1bb060(cx, a2, this.add(4788));
}

/// VehicleEntity__1BB060 0x001BB060
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bb060(cx: &mut Cx, a1: Ptr, a2: Ptr) -> Ptr {
    let _ = f_1bb0c0(cx, a1, a2);
    let _ = f_1bb0c0(cx, a1.add(16), a2.add(16));
    let _ = f_1bb0c0(cx, a1.add(32), a2.add(32));
    return f_1bb0c0(cx, a1.add(48), a2.add(48));
}

/// VehicleEntity__1BB0C0 0x001BB0C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bb0c0(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = cx.i(a2); cx.set_i(p, t); }
    { let p = a1.add(4); let t = cx.i(a2.add(4)); cx.set_i(p, t); }
    { let p = a1.add(8); let t = cx.i(a2.add(8)); cx.set_i(p, t); }
    { let p = a1.add(12); let t = cx.i(a2.add(12)); cx.set_i(p, t); }
    return a1;
}

/// VehicleEntity__1BB100 0x001BB100
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bb100(cx: &mut Cx, this: Ptr, a2: Ptr) -> Ptr {
    return f_1bb140(cx, a2, this.add(4788));
}

/// VehicleEntity__1BB140 0x001BB140
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bb140(cx: &mut Cx, a1: Ptr, a2: Ptr) -> Ptr {
    let _ = f_1bb190(cx, a1, a2);
    let _ = f_1bb190(cx, a1.add(12), a2.add(16));
    return f_1bb190(cx, a1.add(24), a2.add(32));
}

/// VehicleEntity__1BB190 0x001BB190
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bb190(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = cx.i(a2); cx.set_i(p, t); }
    { let p = a1.add(4); let t = cx.i(a2.add(4)); cx.set_i(p, t); }
    { let p = a1.add(8); let t = cx.i(a2.add(8)); cx.set_i(p, t); }
    return a1;
}

/// VehicleEntity__1BB200 0x001BB200
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bb200(cx: &mut Cx, this: Ptr, a2: i32) -> Ptr {
    return this.add((16_i32).wrapping_mul(a2)).add(4788);
}

/// VehicleEntity__1BB320 0x001BB320
pub fn f_1bb320(cx: &mut Cx, a1: Ptr) -> i32 {
    cx.push_frame(5664);
    let r = f_1bb320_body(cx, a1);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1bb320_body(cx: &mut Cx, a1: Ptr) -> i32 {
    let mut v1 = 0i64;
    let mut v2 = Ptr::NULL;
    let mut v3 = Ptr::NULL;
    let mut v4 = 0i32;
    let mut v5 = Ptr::NULL;
    let mut v6 = Ptr::NULL;
    let mut v7 = Ptr::NULL;
    let mut v8 = 0i32;
    let mut v9 = 0.0f64;
    let mut v10 = 0.0f64;
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
    let mut v34 = 0i32;
    let mut v35 = 0i32;
    let mut v36 = 0i32;
    let mut v37 = 0i32;
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
    let mut v49 = 0.0f64;
    let mut v50 = 0.0f64;
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
    let mut v63 = 0.0f64;
    let mut v64 = 0.0f64;
    let mut v65 = 0.0f64;
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
    let mut v78 = 0i32;
    let mut v79 = 0i32;
    let mut v80 = 0i32;
    let mut v81 = 0i32;
    let mut v82 = Ptr::NULL;
    let mut v83 = Ptr::NULL;
    let mut v84 = Ptr::NULL;
    let mut v85 = Ptr::NULL;
    let mut v86 = Ptr::NULL;
    let mut v87 = Ptr::NULL;
    let mut v88 = Ptr::NULL;
    let mut v89 = Ptr::NULL;
    let mut v90 = 0i32;
    let mut v91 = 0i32;
    let mut v92 = 0i32;
    let mut v93 = 0i32;
    let mut v94 = 0.0f32;
    let mut v95 = Ptr::NULL;
    let mut v96 = 0.0f64;
    let mut v97 = 0.0f64;
    let mut v98 = 0.0f64;
    let mut v99 = 0.0f32;
    let mut v100 = 0i32;
    let mut v101 = 0i32;
    let mut v102 = 0i32;
    let mut v103 = 0i32;
    let mut v104 = 0i32;
    let mut v105 = Ptr::NULL;
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
    let mut v122 = Ptr::NULL;
    let mut v123 = Ptr::NULL;
    let mut v124 = Ptr::NULL;
    let mut v125 = Ptr::NULL;
    let mut v126 = Ptr::NULL;
    let mut v127 = Ptr::NULL;
    let mut v128 = Ptr::NULL;
    let mut v129 = Ptr::NULL;
    let mut v130 = Ptr::NULL;
    let mut v131 = Ptr::NULL;
    let mut v132 = Ptr::NULL;
    let mut v133 = Ptr::NULL;
    let mut v134 = Ptr::NULL;
    let mut v135 = Ptr::NULL;
    let mut v136 = Ptr::NULL;
    let mut v137 = Ptr::NULL;
    let mut v138 = Ptr::NULL;
    let mut v139 = Ptr::NULL;
    let mut v140 = Ptr::NULL;
    let mut v141 = Ptr::NULL;
    let mut v142 = Ptr::NULL;
    let mut v143 = Ptr::NULL;
    let mut v144 = Ptr::NULL;
    let mut v145 = Ptr::NULL;
    let mut v146 = Ptr::NULL;
    let mut v147 = Ptr::NULL;
    let mut v148 = Ptr::NULL;
    let mut v149 = Ptr::NULL;
    let mut v150 = Ptr::NULL;
    let mut v151 = Ptr::NULL;
    let mut v152 = Ptr::NULL;
    let mut v153 = Ptr::NULL;
    let mut v154 = Ptr::NULL;
    let mut v155 = Ptr::NULL;
    let mut v156 = Ptr::NULL;
    let mut v157 = Ptr::NULL;
    let mut v158 = Ptr::NULL;
    let mut v159 = Ptr::NULL;
    let mut v160 = Ptr::NULL;
    let mut v161 = Ptr::NULL;
    let mut v162 = Ptr::NULL;
    let mut v163 = Ptr::NULL;
    let mut v164 = Ptr::NULL;
    let mut v165 = Ptr::NULL;
    let mut v166 = Ptr::NULL;
    let mut v167 = Ptr::NULL;
    let mut v168 = Ptr::NULL;
    let mut v169 = 0.0f32;
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
    let mut v232 = 0.0f32;
    let mut v233 = 0.0f32;
    let mut v234 = 0.0f32;
    let mut v235 = 0.0f32;
    let mut v236 = 0i64;
    { let q0_ = { let q0_ = cx.lf(8); crate::ov_zero_i!(cx, q0_) }; crate::ov_ignore!(cx, q0_) };
    cx.memset(cx.lptr(4), 204, 5660);
    { let t = 0; let o = cx.li(16); cx.set_li(16, (o & !(255 << 24)) | ((t & 255) << 24)); }
    let _ = f_1bfef0(cx, a1.add(1088), 0);
    { let t = cx.f(a1.add(4)); cx.set_lf(5660, t); }
    { let t = { let q0_ = Ptr::raw(cx.li(5660).wrapping_add(3560)); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }; cx.set_lpp(1414, t); }
    v1 = (v1 & !0xFFFF_FFFFi64) | ((cx.lf(5660)).to_bits() as i64);
    if (cx.i(Ptr::raw(cx.li(5660).wrapping_add(4756))) != 1) {
        { let p = Ptr::raw(cx.li(5660).wrapping_add(4388)); let t = 0; cx.set_i(p, t); }
        { let p = cx.lpp(1414).add(160); let t = 0; cx.set_i(p, t); }
        { let t = cx.f(Ptr::raw(cx.li(5660).wrapping_add(3552))); cx.set_lf(5652, t); }
        { let t = { let p = ov_physics_manager(cx).add(32); cx.f(p) }; cx.set_lf(5648, t); }
        v1 = (v1 & !0xFFFF_FFFFi64) | (({ let p = ov_physics_manager(cx).add(28); cx.i(p) }) as i32 as u32 as i64);
        { let t = (v1 as i32); cx.set_li(5644, t); }
        if (cx.i(Ptr::raw(((v1 + 28i64) as i32))) != 0) {
            v1 = (v1 & 0xFFFF_FFFFi64) | ((((cx.lf(5648)).to_bits() as i32) as u32 as i64) << 32);
            if (cx.i(Ptr::raw(cx.li(5648).wrapping_add(72))) != 0) {
                { let t = 0.0f32; cx.set_lf(5608, t); }
                if (cx.i(Ptr::gc(0).add(1896)) != 0) {
                    v2 = ov_car_rot_row(cx, a1, 1);
                    { let q0_ = cx.lptr(5628); let q1_ = v2; f_1bffb0(cx, q0_, q1_) };
                    if (cx.i(cx.lpp(1414).add(64)) != 0) {
                        { let t = (((cx.lf(5632) as f64) - 1.0f64) as f32); cx.set_lf(5632, t); }
                        { let q0_ = cx.lptr(5628); f_1c0380(cx, q0_) };
                    } else if (cx.b(Ptr::raw(cx.li(5660).wrapping_add(4072))) != 0) {
                        { let t = (((cx.lf(5632) as f64) - 0.3f64) as f32); cx.set_lf(5632, t); }
                        { let q0_ = cx.lptr(5628); f_1c0380(cx, q0_) };
                    }
                    if ({ let p = ov_physics_manager(cx).add(2840); cx.i(p) } != 0) {
                        v236 = (v236 & 0xFFFF_FFFFi64) | (((cx.i(cx.ptr_at(cx.ptr_at(Ptr::raw(cx.li(5660).wrapping_add(3556)))).add(12))) as u32 as i64) << 32);
                        v236 = (v236 & !0xFFFF_FFFFi64) | ((cx.f(a1.add(72))).to_bits() as i64);
                        v169 = cx.f(a1.add(64));
                        { let t = 1; let o = cx.li(16); cx.set_li(16, (o & !(255 << 24)) | ((t & 255) << 24)); }
                        v3 = ov_physics_manager(cx);
                        { let t = ({ let q0_ = cx.ptr_at(v3.add(2840)); let q1_ = v4; let q2_ = cx.lptr(5616); let q3_ = v169; let q4_ = f32::from_bits(v236 as u32); let q5_ = ((v236 >> 32) & 4294967295) as i32; f_385920(cx, q0_, q1_, q2_, q3_, q4_, q5_) } as i32); cx.set_li(5608, t); }
                        if (((cx.li(16) >> 24) & 255) == 0) {
                        }
                        if (cx.li(5616) == 0) {
                            v236 = (v236 & 0xFFFF_FFFFi64) | ((({ let p = { let p = ov_physics_manager(cx).add(2840); cx.ptr_at(p) }.add(12); cx.i(p) }) as u32 as i64) << 32);
                            crate::ov_ignore!(cx, 1, "!!!CAxisSortedList__GetCollisionCandidates(): The number of returned candidates could not fit into CAxisSortedList's preallocated array of %d, change the constructor parameters\n");
                        }
                    }
                }
                { let _ = { let q0_ = a1; let q1_ = cx.lptr(5568); vcall(cx, 28, q0_, &[Varg::from(q1_)]) }; };
                v1 = { let q0_ = Ptr::raw(cx.li(5648)); let q1_ = cx.lptr(5588); let q2_ = cx.lptr(5600); let q3_ = cx.lptr(5588); let q4_ = cx.lptr(5568); let q5_ = 0; let q6_ = cx.lptr(5236); ov_qt_find_leaf(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
                { let t = 0; let o = cx.li(4232); cx.set_li(4232, (o & !(255 << 24)) | ((t & 255) << 24)); }
                if (cx.li(5588) < 19) {
                    { let t = 0.0f32; cx.set_lf(4228, t); }
                    { let t = 0.0f32; cx.set_lf(4224, t); }
                    'lp9: loop {
                        'blp9: {
                            v1 = (v1 & 0xFFFF_FFFFi64) | ((((cx.lf(4224)).to_bits() as i32) as u32 as i64) << 32);
                            if (cx.li(4224) >= cx.li(5588)) {
                                break 'lp9;
                            }
                            { let t = { let q0_ = Ptr::raw(cx.li(5648)); let q1_ = cx.li(4224).wrapping_add(cx.li(5600)); ov_qt_face_index(cx, q0_, q1_) }; cx.set_li(4220, t); }
                            { let q0_ = cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4228)).wrapping_add(1069))); let q1_ = cx.ptr_at(Ptr::raw(cx.li(5644).wrapping_add(28))).add((12_i32).wrapping_mul(cx.li(4220))); f_1bb190(cx, q0_, q1_) };
                            if ((-{ let q0_ = cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4228)).wrapping_add(1069))); let q1_ = cx.lptr(5628); f_1c0300(cx, q0_, q1_) }) > (cx.f(Ptr::gc(0).add(1892)) as f64)) {
                                v5 = { let q0_ = Ptr::raw(cx.li(5644)); let q1_ = cx.li(4220); let q2_ = 0; ov_mesh_vertex(cx, q0_, q1_, q2_) };
                                { let q0_ = cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4228)).wrapping_add(1060))); let q1_ = v5; f_1bb190(cx, q0_, q1_) };
                                v6 = { let q0_ = Ptr::raw(cx.li(5644)); let q1_ = cx.li(4220); let q2_ = 1; ov_mesh_vertex(cx, q0_, q1_, q2_) };
                                { let q0_ = cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4228)).wrapping_add(1063))); let q1_ = v6; f_1bb190(cx, q0_, q1_) };
                                v7 = { let q0_ = Ptr::raw(cx.li(5644)); let q1_ = cx.li(4220); let q2_ = 2; ov_mesh_vertex(cx, q0_, q1_, q2_) };
                                { let q0_ = cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4228)).wrapping_add(1066))); let q1_ = v7; f_1bb190(cx, q0_, q1_) };
                                if (cx.i(Ptr::raw(cx.li(5644).wrapping_add(12))) != 0) {
                                    { let p = cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4228)).wrapping_add(1072))); let t = cx.u16(cx.ptr_at(Ptr::raw(cx.li(5644).wrapping_add(24))).add((8_i32).wrapping_mul(cx.li(4220))).add(6)); cx.set_i(p, t); }
                                } else {
                                    { let p = cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4228)).wrapping_add(1072))); let t = cx.f(cx.ptr_at(Ptr::raw(cx.li(5644).wrapping_add(20))).add((16_i32).wrapping_mul(cx.li(4220))).add(12)); cx.set_f(p, t); }
                                }
                                v1 = (v1 & !0xFFFF_FFFFi64) | ((cx.preinc_i(cx.lptr(4228))) as i32 as u32 as i64);
                            }
                        }
                        let _ = cx.preinc_i(cx.lptr(4224));
                    }
                    { let t = 1; let o = cx.li(4232); cx.set_li(4232, (o & !(255 << 24)) | ((t & 255) << 24)); }
                    { let t = cx.lf(4228); cx.set_lf(5588, t); }
                }
                { let t = 0.0f32; cx.set_lf(4216, t); }
                'lp10: loop {
                    if !(cx.li(4216) < 4) { break 'lp10; }
                    'blp10: {
                        { let t = 0; let o = cx.li(16); cx.set_li(16, (o & !(255 << 16)) | ((t & 255) << 16)); }
                        { let q0_ = cx.lptr(4180); f_1da550(cx, q0_) };
                        { let t = 0.0f32; cx.set_lf(4172, t); }
                        { let t = 0.0f32; cx.set_lf(4168, t); }
                        { let t = cx.f(Ptr::gc(0).add(1900)); cx.set_lf(4124, t); }
                        { let t = cx.f(Ptr::gc(0).add(1904)); cx.set_lf(4120, t); }
                        if (cx.i(Ptr::gc(0).add(1896)) != 0) {
                            v9 = ({ let q0_ = a1; let q1_ = cx.li(4216); f_35fb40(cx, q0_, q1_) } as f64);
                            { let t = ((v9 * (cx.lf(4120) as f64)) as f32); cx.set_lf(4172, t); }
                            v10 = ({ let q0_ = a1; let q1_ = cx.li(4216); f_35fb40(cx, q0_, q1_) } as f64);
                            { let t = ((v10 * (cx.lf(4124) as f64)) as f32); cx.set_lf(4168, t); }
                            { let t = 1; let o = cx.li(16); cx.set_li(16, (o & !(255 << 16)) | ((t & 255) << 16)); }
                            { let q0_ = a1; let q1_ = cx.lptr(4152); let q2_ = cx.li(4216); f_361390(cx, q0_, q1_, q2_) };
                            { let t = 1; let o = cx.li(16); cx.set_li(16, (o & !(255 << 16)) | ((t & 255) << 16)); }
                            { let q0_ = cx.lptr(4132); let q1_ = cx.lptr(4152); let q2_ = cx.lptr(5628); let q3_ = ((0.0f64 - (cx.lf(4168) as f64)) as f32); f_1c0210(cx, q0_, q1_, q2_, q3_) };
                        } else {
                            { let q0_ = a1; let q1_ = cx.lptr(4196); let q2_ = cx.li(4216); f_361610(cx, q0_, q1_, q2_) };
                            { let q0_ = a1; let q1_ = cx.lptr(4180); let q2_ = cx.li(4216); f_361390(cx, q0_, q1_, q2_) };
                            { let t = { let q0_ = a1; let q1_ = cx.li(4216); f_35fb40(cx, q0_, q1_) }; cx.set_lf(4208, t); }
                        }
                        { let t = f32::from_bits(0xFFFF_FFFF); cx.set_lf(4116, t); }
                        if ((cx.li(5588) != 0) && (cx.i(Ptr::gc(0).add(2096)) != 0)) {
                            if (cx.i(Ptr::gc(0).add(1896)) != 0) {
                                { let t = 0.0f32; cx.set_lf(4112, t); }
                                'lp11: loop {
                                    if !(cx.li(4112) < cx.li(5588)) { break 'lp11; }
                                    'blp11: {
                                        { let t = 0.0f32; cx.set_lf(4108, t); }
                                        if (((cx.li(4232) >> 24) & 255) != 0) {
                                            { let t = cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4112)).wrapping_add(1060))); cx.set_lpp(1026, t); }
                                            { let t = cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4112)).wrapping_add(1063))); cx.set_lpp(1025, t); }
                                            { let t = cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4112)).wrapping_add(1066))); cx.set_lpp(1024, t); }
                                            { let t = cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4112)).wrapping_add(1069))); cx.set_lpp(1023, t); }
                                        } else {
                                            { let t = { let q0_ = Ptr::raw(cx.li(5648)); let q1_ = cx.li(4112).wrapping_add(cx.li(5600)); ov_qt_face_index(cx, q0_, q1_) }; cx.set_li(4108, t); }
                                            { let t = { let q0_ = Ptr::raw(cx.li(5644)); let q1_ = cx.li(4108); let q2_ = 0; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(1026, t); }
                                            { let t = { let q0_ = Ptr::raw(cx.li(5644)); let q1_ = cx.li(4108); let q2_ = 1; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(1025, t); }
                                            { let t = { let q0_ = Ptr::raw(cx.li(5644)); let q1_ = cx.li(4108); let q2_ = 2; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(1024, t); }
                                            { let t = cx.i(Ptr::raw(cx.li(5644).wrapping_add(28))).wrapping_add((12_i32).wrapping_mul(cx.li(4108))); cx.set_lpp(1023, Ptr::raw(t)); }
                                        }
                                        { let q0_ = cx.lptr(4076); let q1_ = cx.lpp(1025); let q2_ = cx.lpp(1026); f_1c00c0(cx, q0_, q1_, q2_) };
                                        { let q0_ = cx.lptr(4056); let q1_ = cx.lpp(1024); let q2_ = cx.lpp(1026); f_1c00c0(cx, q0_, q1_, q2_) };
                                        { let q0_ = cx.lptr(4036); let q1_ = cx.lptr(4076); let q2_ = cx.lptr(4056); f_1c0270(cx, q0_, q1_, q2_) };
                                        { let t = { let q0_ = cx.lptr(4036); f_1c0330(cx, q0_) }; cx.set_lf(4028, t); }
                                        if (cx.lf(4028) == 0.0f32) {
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(2248); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v170 = cx.i(Ptr::glob(0x77FD48));
                                            v107 = { let q0_ = cx.lptr(2232); let q1_ = cx.i(cx.lpp(1025)); let q2_ = cx.i(cx.lpp(1025).add(4)); let q3_ = cx.i(cx.lpp(1025).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v11 = { let q0_ = cx.lptr(2216); let q1_ = cx.i(cx.lpp(1026)); let q2_ = cx.i(cx.lpp(1026).add(4)); let q3_ = cx.i(cx.lpp(1026).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v11, v107, v170, v236);
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(2212); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v171 = cx.i(Ptr::glob(0x77FD48));
                                            v108 = { let q0_ = cx.lptr(2196); let q1_ = cx.i(cx.lpp(1024)); let q2_ = cx.i(cx.lpp(1024).add(4)); let q3_ = cx.i(cx.lpp(1024).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v12 = { let q0_ = cx.lptr(2180); let q1_ = cx.i(cx.lpp(1026)); let q2_ = cx.i(cx.lpp(1026).add(4)); let q3_ = cx.i(cx.lpp(1026).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v12, v108, v171, v236);
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(2176); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v172 = cx.i(Ptr::glob(0x77FD48));
                                            v109 = { let q0_ = cx.lptr(2160); let q1_ = cx.i(cx.lpp(1024)); let q2_ = cx.i(cx.lpp(1024).add(4)); let q3_ = cx.i(cx.lpp(1024).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v13 = { let q0_ = cx.lptr(2144); let q1_ = cx.i(cx.lpp(1025)); let q2_ = cx.i(cx.lpp(1025).add(4)); let q3_ = cx.i(cx.lpp(1025).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v13, v109, v172, v236);
                                        }
                                        if (((cx.i(Ptr::gc(0).add(1956)) != 0) && (cx.b(Ptr::raw(cx.li(5660).wrapping_add(4072))) != 0)) && (cx.li(4216) == 0)) {
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(2140); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v173 = cx.i(Ptr::glob(0x77FDA8));
                                            v110 = { let q0_ = cx.lptr(2124); let q1_ = cx.i(cx.lpp(1025)); let q2_ = cx.i(cx.lpp(1025).add(4)); let q3_ = cx.i(cx.lpp(1025).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v14 = { let q0_ = cx.lptr(2108); let q1_ = cx.i(cx.lpp(1026)); let q2_ = cx.i(cx.lpp(1026).add(4)); let q3_ = cx.i(cx.lpp(1026).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v14, v110, v173, v236);
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(2104); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v174 = cx.i(Ptr::glob(0x77FDA8));
                                            v111 = { let q0_ = cx.lptr(2088); let q1_ = cx.i(cx.lpp(1024)); let q2_ = cx.i(cx.lpp(1024).add(4)); let q3_ = cx.i(cx.lpp(1024).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v15 = { let q0_ = cx.lptr(2072); let q1_ = cx.i(cx.lpp(1026)); let q2_ = cx.i(cx.lpp(1026).add(4)); let q3_ = cx.i(cx.lpp(1026).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v15, v111, v174, v236);
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(2068); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v175 = cx.i(Ptr::glob(0x77FDA8));
                                            v112 = { let q0_ = cx.lptr(2052); let q1_ = cx.i(cx.lpp(1025)); let q2_ = cx.i(cx.lpp(1025).add(4)); let q3_ = cx.i(cx.lpp(1025).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v16 = { let q0_ = cx.lptr(2036); let q1_ = cx.i(cx.lpp(1024)); let q2_ = cx.i(cx.lpp(1024).add(4)); let q3_ = cx.i(cx.lpp(1024).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v16, v112, v175, v236);
                                            { let q0_ = cx.lptr(4012); let q1_ = cx.lpp(1026); let q2_ = cx.lpp(1025); let q3_ = cx.lpp(1024); f_1c0050(cx, q0_, q1_, q2_, q3_) };
                                            { let q0_ = cx.lptr(4012); let q1_ = 0.33333334f32; f_1c0110(cx, q0_, q1_) };
                                            { let q0_ = cx.lptr(3992); let q1_ = cx.lptr(4012); let q2_ = cx.lpp(1023); let q3_ = 2.0f32; f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(2032); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v176 = cx.i(Ptr::glob(0x77FD8C));
                                            v113 = { let q0_ = cx.lptr(2016); let q1_ = cx.li(3992); let q2_ = cx.li(3996); let q3_ = cx.li(4000); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v17 = { let q0_ = cx.lptr(2000); let q1_ = cx.li(4012); let q2_ = cx.li(4016); let q3_ = cx.li(4020); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v17, v113, v176, v236);
                                        }
                                        if (((cx.i(Ptr::gc(0).add(1960)) != 0) && (cx.b(Ptr::raw(cx.li(5660).wrapping_add(4072))) != 0)) && (cx.li(4216) == 0)) {
                                            { let t = cx.lptr(0).add(4 * ((2_i32).wrapping_mul(cx.li(5556)).wrapping_add(1309))); cx.set_lpp(996, t); }
                                            { let t = cx.lptr(0).add(4 * ((2_i32).wrapping_mul(cx.li(5556)).wrapping_add(1349))); cx.set_lpp(995, t); }
                                            { let t = { let p = f_361400(cx, a1, 0).add(4); cx.f(p) }; cx.set_lf(3976, t); }
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1996); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v177 = cx.i(Ptr::glob(0x77FD58));
                                            v114 = { let q0_ = cx.lptr(1980); let q1_ = cx.i(cx.lpp(995)); let q2_ = cx.li(3976); let q3_ = cx.i(cx.lpp(996).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v18 = { let q0_ = cx.lptr(1964); let q1_ = cx.i(cx.lpp(996)); let q2_ = cx.li(3976); let q3_ = cx.i(cx.lpp(996).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v18, v114, v177, v236);
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1960); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v178 = cx.i(Ptr::glob(0x77FD58));
                                            v115 = { let q0_ = cx.lptr(1944); let q1_ = cx.i(cx.lpp(996)); let q2_ = cx.li(3976); let q3_ = cx.i(cx.lpp(996).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v19 = { let q0_ = cx.lptr(1928); let q1_ = cx.i(cx.lpp(996)); let q2_ = cx.li(3976); let q3_ = cx.i(cx.lpp(995).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v19, v115, v178, v236);
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1924); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v179 = cx.i(Ptr::glob(0x77FD58));
                                            v116 = { let q0_ = cx.lptr(1908); let q1_ = cx.i(cx.lpp(996)); let q2_ = cx.li(3976); let q3_ = cx.i(cx.lpp(995).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v20 = { let q0_ = cx.lptr(1892); let q1_ = cx.i(cx.lpp(995)); let q2_ = cx.li(3976); let q3_ = cx.i(cx.lpp(995).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v20, v116, v179, v236);
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1888); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v180 = cx.i(Ptr::glob(0x77FD58));
                                            v117 = { let q0_ = cx.lptr(1872); let q1_ = cx.i(cx.lpp(995)); let q2_ = cx.li(3976); let q3_ = cx.i(cx.lpp(995).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v21 = { let q0_ = cx.lptr(1856); let q1_ = cx.i(cx.lpp(995)); let q2_ = cx.li(3976); let q3_ = cx.i(cx.lpp(996).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v21, v117, v180, v236);
                                            { let t = cx.f(Ptr::raw(cx.li(5648).wrapping_add(56))); cx.set_lf(3972, t); }
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1852); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v181 = cx.i(Ptr::glob(0x77FD58));
                                            v118 = { let q0_ = cx.lptr(1836); let q1_ = { let q0_ = (((cx.f(cx.lpp(995)) as f64) + (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3976); let q3_ = { let q0_ = (((cx.f(cx.lpp(996).add(4)) as f64) - (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v22 = { let q0_ = cx.lptr(1820); let q1_ = { let q0_ = (((cx.f(cx.lpp(996)) as f64) - (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3976); let q3_ = { let q0_ = (((cx.f(cx.lpp(996).add(4)) as f64) - (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v22, v118, v181, v236);
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1816); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v182 = cx.i(Ptr::glob(0x77FD58));
                                            v119 = { let q0_ = cx.lptr(1800); let q1_ = { let q0_ = (((cx.f(cx.lpp(996)) as f64) - (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3976); let q3_ = { let q0_ = (((cx.f(cx.lpp(996).add(4)) as f64) - (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v23 = { let q0_ = cx.lptr(1784); let q1_ = { let q0_ = (((cx.f(cx.lpp(996)) as f64) - (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3976); let q3_ = { let q0_ = (((cx.f(cx.lpp(995).add(4)) as f64) + (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v23, v119, v182, v236);
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1780); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v183 = cx.i(Ptr::glob(0x77FD58));
                                            v120 = { let q0_ = cx.lptr(1764); let q1_ = { let q0_ = (((cx.f(cx.lpp(996)) as f64) - (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3976); let q3_ = { let q0_ = (((cx.f(cx.lpp(995).add(4)) as f64) + (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v24 = { let q0_ = cx.lptr(1748); let q1_ = { let q0_ = (((cx.f(cx.lpp(995)) as f64) + (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3976); let q3_ = { let q0_ = (((cx.f(cx.lpp(995).add(4)) as f64) + (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v24, v120, v183, v236);
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1744); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v184 = cx.i(Ptr::glob(0x77FD58));
                                            v121 = { let q0_ = cx.lptr(1728); let q1_ = { let q0_ = (((cx.f(cx.lpp(995)) as f64) + (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3976); let q3_ = { let q0_ = (((cx.f(cx.lpp(995).add(4)) as f64) + (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v25 = { let q0_ = cx.lptr(1712); let q1_ = { let q0_ = (((cx.f(cx.lpp(995)) as f64) + (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3976); let q3_ = { let q0_ = (((cx.f(cx.lpp(996).add(4)) as f64) - (cx.lf(3972) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v25, v121, v184, v236);
                                        }
                                        if ((((cx.i(Ptr::gc(0).add(1960)) != 0) || (cx.i(Ptr::gc(0).add(1956)) != 0)) && (cx.b(Ptr::raw(cx.li(5660).wrapping_add(4072))) != 0)) && (cx.li(4216) == 0)) {
                                            { let t = { let p = f_361400(cx, a1, 0).add(4); cx.f(p) }; cx.set_lf(3968, t); }
                                            { let t = ov_barrier_qt(cx, Ptr::glob(0x705F48)); cx.set_lpp(991, t); }
                                            if (cx.lpp(991).bits() != 0) {
                                                { let q0_ = cx.lpp(991); let q1_ = cx.lptr(5568); let q2_ = cx.lptr(3624); let q3_ = cx.lptr(3612); let q4_ = cx.lptr(5568); let q5_ = 0; let q6_ = cx.lptr(3636); ov_qt_find_leaf(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_) };
                                                { let t = cx.lptr(0).add(4 * ((2_i32).wrapping_mul(cx.li(3956)).wrapping_add(909))); cx.set_lpp(901, t); }
                                                { let t = cx.lptr(0).add(4 * ((2_i32).wrapping_mul(cx.li(3956)).wrapping_add(949))); cx.set_lpp(900, t); }
                                                if (cx.i(Ptr::gc(0).add(1960)) != 0) {
                                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1708); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                    v185 = cx.i(Ptr::glob(0x77FDAC));
                                                    v122 = { let q0_ = cx.lptr(1692); let q1_ = cx.i(cx.lpp(900)); let q2_ = cx.li(3968); let q3_ = cx.i(cx.lpp(901).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v26 = { let q0_ = cx.lptr(1676); let q1_ = cx.i(cx.lpp(901)); let q2_ = cx.li(3968); let q3_ = cx.i(cx.lpp(901).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v26, v122, v185, v236);
                                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1672); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                    v186 = cx.i(Ptr::glob(0x77FDAC));
                                                    v123 = { let q0_ = cx.lptr(1656); let q1_ = cx.i(cx.lpp(901)); let q2_ = cx.li(3968); let q3_ = cx.i(cx.lpp(901).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v27 = { let q0_ = cx.lptr(1640); let q1_ = cx.i(cx.lpp(901)); let q2_ = cx.li(3968); let q3_ = cx.i(cx.lpp(900).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v27, v123, v186, v236);
                                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1636); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                    v187 = cx.i(Ptr::glob(0x77FDAC));
                                                    v124 = { let q0_ = cx.lptr(1620); let q1_ = cx.i(cx.lpp(901)); let q2_ = cx.li(3968); let q3_ = cx.i(cx.lpp(900).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v28 = { let q0_ = cx.lptr(1604); let q1_ = cx.i(cx.lpp(900)); let q2_ = cx.li(3968); let q3_ = cx.i(cx.lpp(900).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v28, v124, v187, v236);
                                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1600); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                    v188 = cx.i(Ptr::glob(0x77FDAC));
                                                    v125 = { let q0_ = cx.lptr(1584); let q1_ = cx.i(cx.lpp(900)); let q2_ = cx.li(3968); let q3_ = cx.i(cx.lpp(900).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v29 = { let q0_ = cx.lptr(1568); let q1_ = cx.i(cx.lpp(900)); let q2_ = cx.li(3968); let q3_ = cx.i(cx.lpp(901).add(4)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v29, v125, v188, v236);
                                                    { let t = cx.f(cx.lpp(991).add(56)); cx.set_lf(3596, t); }
                                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1564); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                    v189 = cx.i(Ptr::glob(0x77FDAC));
                                                    v126 = { let q0_ = cx.lptr(1548); let q1_ = { let q0_ = (((cx.f(cx.lpp(900)) as f64) + (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3968); let q3_ = { let q0_ = (((cx.f(cx.lpp(901).add(4)) as f64) - (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v30 = { let q0_ = cx.lptr(1532); let q1_ = { let q0_ = (((cx.f(cx.lpp(901)) as f64) - (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3968); let q3_ = { let q0_ = (((cx.f(cx.lpp(901).add(4)) as f64) - (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v30, v126, v189, v236);
                                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1528); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                    v190 = cx.i(Ptr::glob(0x77FDAC));
                                                    v127 = { let q0_ = cx.lptr(1512); let q1_ = { let q0_ = (((cx.f(cx.lpp(901)) as f64) - (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3968); let q3_ = { let q0_ = (((cx.f(cx.lpp(901).add(4)) as f64) - (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v31 = { let q0_ = cx.lptr(1496); let q1_ = { let q0_ = (((cx.f(cx.lpp(901)) as f64) - (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3968); let q3_ = { let q0_ = (((cx.f(cx.lpp(900).add(4)) as f64) + (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v31, v127, v190, v236);
                                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1492); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                    v191 = cx.i(Ptr::glob(0x77FDAC));
                                                    v128 = { let q0_ = cx.lptr(1476); let q1_ = { let q0_ = (((cx.f(cx.lpp(901)) as f64) - (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3968); let q3_ = { let q0_ = (((cx.f(cx.lpp(900).add(4)) as f64) + (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v32 = { let q0_ = cx.lptr(1460); let q1_ = { let q0_ = (((cx.f(cx.lpp(900)) as f64) + (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3968); let q3_ = { let q0_ = (((cx.f(cx.lpp(900).add(4)) as f64) + (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v32, v128, v191, v236);
                                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1456); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                    v192 = cx.i(Ptr::glob(0x77FDAC));
                                                    v129 = { let q0_ = cx.lptr(1440); let q1_ = { let q0_ = (((cx.f(cx.lpp(900)) as f64) + (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3968); let q3_ = { let q0_ = (((cx.f(cx.lpp(900).add(4)) as f64) + (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    v33 = { let q0_ = cx.lptr(1424); let q1_ = { let q0_ = (((cx.f(cx.lpp(900)) as f64) + (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.li(3968); let q3_ = { let q0_ = (((cx.f(cx.lpp(901).add(4)) as f64) - (cx.lf(3596) as f64)) as f32); ov_coerce_int(cx, q0_) }; ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v33, v129, v192, v236);
                                                }
                                                if (cx.i(Ptr::gc(0).add(1956)) != 0) {
                                                    { let t = ov_barrier_mesh(cx, Ptr::glob(0x705F48)); cx.set_lpp(898, t); }
                                                    { let t = 0.0f32; cx.set_lf(3588, t); }
                                                    'lp12: loop {
                                                        if !(cx.li(3588) < cx.li(5588)) { break 'lp12; }
                                                        'blp12: {
                                                            v34 = { let q0_ = cx.lpp(991); let q1_ = cx.li(3588).wrapping_add(cx.li(3624)); ov_qt_face_index(cx, q0_, q1_) };
                                                            { let t = { let q0_ = cx.lpp(898); let q1_ = v34; let q2_ = 0; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(896, t); }
                                                            v35 = { let q0_ = cx.lpp(991); let q1_ = cx.li(3588).wrapping_add(cx.li(3624)); ov_qt_face_index(cx, q0_, q1_) };
                                                            { let t = { let q0_ = cx.lpp(898); let q1_ = v35; let q2_ = 1; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(895, t); }
                                                            v36 = { let q0_ = cx.lpp(991); let q1_ = cx.li(3588).wrapping_add(cx.li(3624)); ov_qt_face_index(cx, q0_, q1_) };
                                                            { let t = { let q0_ = cx.lpp(898); let q1_ = v36; let q2_ = 2; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(894, t); }
                                                            v37 = { let q0_ = cx.lpp(991); let q1_ = cx.li(3588).wrapping_add(cx.li(3624)); ov_qt_face_index(cx, q0_, q1_) };
                                                            { let t = cx.i(cx.lpp(898).add(28)).wrapping_add((12_i32).wrapping_mul(v37)); cx.set_li(3572, t); }
                                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1420); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                            v193 = cx.i(Ptr::glob(0x77FD6C));
                                                            v130 = { let q0_ = cx.lptr(1404); let q1_ = cx.i(cx.lpp(895)); let q2_ = cx.i(cx.lpp(895).add(4)); let q3_ = cx.i(cx.lpp(895).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            v38 = { let q0_ = cx.lptr(1388); let q1_ = cx.i(cx.lpp(896)); let q2_ = cx.i(cx.lpp(896).add(4)); let q3_ = cx.i(cx.lpp(896).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v38, v130, v193, v236);
                                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1384); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                            v194 = cx.i(Ptr::glob(0x77FD6C));
                                                            v131 = { let q0_ = cx.lptr(1368); let q1_ = cx.i(cx.lpp(894)); let q2_ = cx.i(cx.lpp(894).add(4)); let q3_ = cx.i(cx.lpp(894).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            v39 = { let q0_ = cx.lptr(1352); let q1_ = cx.i(cx.lpp(896)); let q2_ = cx.i(cx.lpp(896).add(4)); let q3_ = cx.i(cx.lpp(896).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v39, v131, v194, v236);
                                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1348); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                            v195 = cx.i(Ptr::glob(0x77FD6C));
                                                            v132 = { let q0_ = cx.lptr(1332); let q1_ = cx.i(cx.lpp(895)); let q2_ = cx.i(cx.lpp(895).add(4)); let q3_ = cx.i(cx.lpp(895).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            v40 = { let q0_ = cx.lptr(1316); let q1_ = cx.i(cx.lpp(894)); let q2_ = cx.i(cx.lpp(894).add(4)); let q3_ = cx.i(cx.lpp(894).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v40, v132, v195, v236);
                                                            { let q0_ = cx.lptr(3556); let q1_ = cx.lpp(896); let q2_ = cx.lpp(895); let q3_ = cx.lpp(894); f_1c0050(cx, q0_, q1_, q2_, q3_) };
                                                            { let q0_ = cx.lptr(3556); let q1_ = 0.33333334f32; f_1c0110(cx, q0_, q1_) };
                                                            { let q0_ = cx.lptr(3536); let q1_ = cx.lptr(3556); let q2_ = Ptr::raw(cx.li(3572)); let q3_ = 2.0f32; f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1312); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                            v196 = cx.i(Ptr::glob(0x77FD40));
                                                            v133 = { let q0_ = cx.lptr(1296); let q1_ = cx.li(3536); let q2_ = cx.li(3540); let q3_ = cx.li(3544); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            v41 = { let q0_ = cx.lptr(1280); let q1_ = cx.li(3556); let q2_ = cx.li(3560); let q3_ = cx.li(3564); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v41, v133, v196, v236);
                                                        }
                                                        let _ = cx.preinc_i(cx.lptr(3588));
                                                    }
                                                }
                                            }
                                        }
                                        if (((((cx.f(Ptr::gc(0).add(1892)) as f64) <= (-{ let q0_ = cx.lpp(1023); let q1_ = cx.lptr(5628); f_1c0300(cx, q0_, q1_) })) && ({ let q0_ = cx.lptr(3524); let q1_ = cx.lptr(3512); let q2_ = cx.lptr(3500); let q3_ = cx.lpp(1026); let q4_ = cx.lpp(1025); let q5_ = cx.lpp(1024); let q6_ = cx.lptr(4132); let q7_ = cx.lptr(5628); ov_ray_triangle(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_) } != 0)) && (cx.lf(3524) > 0.0f32)) && ((cx.lf(4168) + cx.lf(4172)) > cx.lf(3524))) {
                                            if (cx.i(Ptr::gc(0).add(1964)) != 0) {
                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1276); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                v197 = cx.i(Ptr::glob(0x77FDA8));
                                                v134 = { let q0_ = cx.lptr(1260); let q1_ = cx.i(cx.lpp(1025)); let q2_ = cx.i(cx.lpp(1025).add(4)); let q3_ = cx.i(cx.lpp(1025).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                v42 = { let q0_ = cx.lptr(1244); let q1_ = cx.i(cx.lpp(1026)); let q2_ = cx.i(cx.lpp(1026).add(4)); let q3_ = cx.i(cx.lpp(1026).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v42, v134, v197, v236);
                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1240); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                v198 = cx.i(Ptr::glob(0x77FDA8));
                                                v135 = { let q0_ = cx.lptr(1224); let q1_ = cx.i(cx.lpp(1024)); let q2_ = cx.i(cx.lpp(1024).add(4)); let q3_ = cx.i(cx.lpp(1024).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                v43 = { let q0_ = cx.lptr(1208); let q1_ = cx.i(cx.lpp(1026)); let q2_ = cx.i(cx.lpp(1026).add(4)); let q3_ = cx.i(cx.lpp(1026).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v43, v135, v198, v236);
                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1204); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                v199 = cx.i(Ptr::glob(0x77FDA8));
                                                v136 = { let q0_ = cx.lptr(1188); let q1_ = cx.i(cx.lpp(1025)); let q2_ = cx.i(cx.lpp(1025).add(4)); let q3_ = cx.i(cx.lpp(1025).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                v44 = { let q0_ = cx.lptr(1172); let q1_ = cx.i(cx.lpp(1024)); let q2_ = cx.i(cx.lpp(1024).add(4)); let q3_ = cx.i(cx.lpp(1024).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v44, v136, v199, v236);
                                                { let q0_ = cx.lptr(3480); let q1_ = cx.lpp(1026); let q2_ = cx.lpp(1025); let q3_ = cx.lpp(1024); f_1c0050(cx, q0_, q1_, q2_, q3_) };
                                                { let q0_ = cx.lptr(3480); let q1_ = 0.33333334f32; f_1c0110(cx, q0_, q1_) };
                                                { let q0_ = cx.lptr(3460); let q1_ = cx.lptr(3480); let q2_ = cx.lpp(1023); let q3_ = 2.0f32; f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1168); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                v200 = cx.i(Ptr::glob(0x77FD8C));
                                                v137 = { let q0_ = cx.lptr(1152); let q1_ = cx.li(3460); let q2_ = cx.li(3464); let q3_ = cx.li(3468); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                v45 = { let q0_ = cx.lptr(1136); let q1_ = cx.li(3480); let q2_ = cx.li(3484); let q3_ = cx.li(3488); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v45, v137, v200, v236);
                                                { let q0_ = cx.lptr(3440); let q1_ = cx.lptr(4132); let q2_ = cx.lptr(5628); let q3_ = (((cx.lf(4168) as f64) + (cx.lf(4172) as f64)) as f32); f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1132); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                v201 = cx.i(Ptr::glob(0x77FD84));
                                                v138 = { let q0_ = cx.lptr(1116); let q1_ = cx.li(3440); let q2_ = cx.li(3444); let q3_ = cx.li(3448); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                v46 = { let q0_ = cx.lptr(1100); let q1_ = cx.li(4132); let q2_ = cx.li(4136); let q3_ = cx.li(4140); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v46, v138, v201, v236);
                                            }
                                            if ((-{ let q0_ = cx.lpp(1023); let q1_ = cx.lptr(5628); f_1c0300(cx, q0_, q1_) }) >= (cx.f(Ptr::gc(0).add(1892)) as f64)) {
                                                v47 = { let q0_ = cx.lptr(5628); let q1_ = cx.lpp(1023); f_1c0300(cx, q0_, q1_) };
                                                { let t = ((v47 * ((cx.lf(3524) as f64) - (cx.lf(4168) as f64))) as f32); cx.set_lf(12, t); }
                                                { let t = 1; let o = cx.li(16); cx.set_li(16, (o & !(255 << 16)) | ((t & 255) << 16)); }
                                                v48 = { let q0_ = cx.lptr(4152); let q1_ = cx.lpp(1023); f_1c0300(cx, q0_, q1_) };
                                                { let t = ((v48 + (cx.lf(12) as f64)) as f32); cx.set_lf(3432, t); }
                                                { let q0_ = a1; let q1_ = cx.lptr(3416); let q2_ = cx.li(4216); f_361610(cx, q0_, q1_, q2_) };
                                                if (((cx.li(4232) >> 24) & 255) != 0) {
                                                    { let t = ((cx.f(cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4112)).wrapping_add(1072)))).to_bits() as i32) & -65281); cx.set_li(3408, t); }
                                                } else {
                                                    { let t = (if (cx.i(Ptr::raw(cx.li(5644).wrapping_add(12))) != 0) { (cx.u16(cx.ptr_at(Ptr::raw(cx.li(5644).wrapping_add(24))).add((8_i32).wrapping_mul(cx.li(4108))).add(6)) as u8 as i32) } else { (cx.i(cx.ptr_at(Ptr::raw(cx.li(5644).wrapping_add(20))).add((16_i32).wrapping_mul(cx.li(4108))).add(12)) & -65281) }); cx.set_li(3408, t); }
                                                }
                                                { let p = Ptr::raw(cx.li(4216).wrapping_add(cx.li(5660)).wrapping_add(4868)); let t = ((cx.li(3408) >> 0) & 255); cx.set_b(p, t); }
                                                { let t = ({ let q0_ = cx.lpp(1023); let q1_ = cx.lptr(3416); f_1c0300(cx, q0_, q1_) } as f32); cx.set_lf(3404, t); }
                                                if ((cx.lf(3404) <= 0.999999f32) && (cx.lf(3404) >= -0.999999f32)) {
                                                    { let q0_ = cx.lptr(3364); let q1_ = cx.lpp(1023); let q2_ = cx.lptr(3416); let q3_ = ((0.0f64 - (cx.lf(3404) as f64)) as f32); f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                                    { let t = (-{ let q0_ = a1; let q1_ = cx.li(4216); f_35fb40(cx, q0_, q1_) }); cx.set_lf(8, t); }
                                                    v49 = ({ let q0_ = cx.lptr(3364); f_1c0330(cx, q0_) } as f64);
                                                    v236 = (v236 & !0xFFFF_FFFFi64) | (((((cx.lf(8) as f64) / v49) as f32)).to_bits() as i64);
                                                    { let t = 1; let o = cx.li(16); cx.set_li(16, (o & !(255 << 16)) | ((t & 255) << 16)); }
                                                    { let q0_ = cx.lptr(3388); let q1_ = cx.lptr(4152); let q2_ = cx.lptr(3364); let q3_ = f32::from_bits(v236 as u32); f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                                } else {
                                                    { let t = 1; let o = cx.li(16); cx.set_li(16, (o & !(255 << 16)) | ((t & 255) << 16)); }
                                                    { let q0_ = cx.lptr(3388); let q1_ = cx.lptr(4152); f_1bb190(cx, q0_, q1_) };
                                                }
                                                if (({ let _ = { let _ = { v50 = { let q0_ = cx.lptr(3388); let q1_ = cx.lpp(1023); f_1c0300(cx, q0_, q1_) }; v50 }; { { let t = (((cx.lf(3432) as f64) - v50) as f32); cx.set_lf(3380, t); } } }; ((cx.f(a1.add(4 * (cx.li(4216).wrapping_add(272)))).to_bits() as i32) == 0) } && (cx.lf(3380) > 0.0f32)) || (cx.lf(3380) > cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149)))))) {
                                                    { let q0_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(146))); let q1_ = cx.lptr(3388); f_1bb190(cx, q0_, q1_) };
                                                    { let q0_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(143))); let q1_ = cx.lpp(1023); f_1bb190(cx, q0_, q1_) };
                                                    { let p = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149))); let t = cx.lf(3380); cx.set_f(p, t); }
                                                    { let p = a1.add(4 * (cx.li(4216).wrapping_add(272))); let t = 1; cx.set_i(p, t); }
                                                    if (((cx.li(4232) >> 24) & 255) != 0) {
                                                        { let t = cx.lf(4112); cx.set_lf(4116, t); }
                                                    } else {
                                                        { let t = cx.lf(4108); cx.set_lf(4116, t); }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    let _ = cx.preinc_i(cx.lptr(4112));
                                }
                                if ((cx.f(a1.add(4 * (cx.li(4216).wrapping_add(272)))).to_bits() as i32) == 0) {
                                    { let p = Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(3936)); let t = 897988541; cx.set_i(p, t); }
                                }
                                if ((cx.i(Ptr::gc(0).add(2092)) != 0) && ((cx.i(Ptr::raw(cx.li(5660).wrapping_add(5084))) < 2) || (cx.i(Ptr::raw(cx.li(5660).wrapping_add(5084))) == 4))) {
                                    { let t = 0.0f32; cx.set_lf(3356, t); }
                                    'lp13: loop {
                                        if !(cx.li(3356) < cx.li(5608)) { break 'lp13; }
                                        'blp13: {
                                            if (((cx.li(16) >> 24) & 255) == 0) {
                                            }
                                            { let t = cx.f(Ptr::raw(cx.li(5616).wrapping_add((4_i32).wrapping_mul(cx.li(3356))))); cx.set_lf(3352, t); }
                                            if (((((cx.li(3352) == 0) || (cx.i(Ptr::raw(cx.li(3352).wrapping_add(104))) == 0)) || (cx.i(cx.ptr_at(Ptr::raw(cx.li(3352).wrapping_add(104))).add(20)) != 0)) && (cx.li(3352) != 0)) && (cx.i(Ptr::raw(cx.li(3352).wrapping_add(104))) != 0)) {
                                                { let t = cx.f(Ptr::raw(cx.li(3352).wrapping_add(104))); cx.set_lf(3348, t); }
                                                { let t = { let q0_ = Ptr::raw(cx.li(3352)); crate::ov_ident!(cx, q0_) }; cx.set_lpp(836, t); }
                                                v51 = { let q0_ = cx.lpp(836); let q1_ = 3; ov_mat_row(cx, q0_, q1_) };
                                                { let t = ov_vec4_x(cx, v51); cx.set_lf(3328, t); }
                                                v52 = { let q0_ = cx.lpp(836); let q1_ = 3; ov_mat_row(cx, q0_, q1_) };
                                                { let t = ov_vec4_y(cx, v52); cx.set_lf(3332, t); }
                                                v53 = { let q0_ = cx.lpp(836); let q1_ = 3; ov_mat_row(cx, q0_, q1_) };
                                                { let t = ov_vec4_z(cx, v53); cx.set_lf(3336, t); }
                                                if (((cx.li(16) >> 16) & 255) == 0) {
                                                }
                                                { let t = (((cx.lf(4152) as f64) - (cx.lf(3328) as f64)) as f32); cx.set_lf(3308, t); }
                                                { let t = (((cx.lf(4160) as f64) - (cx.lf(3336) as f64)) as f32); cx.set_lf(3316, t); }
                                                { let t = (((cx.lf(4156) as f64) - (cx.lf(3332) as f64)) as f32); cx.set_lf(3312, t); }
                                                { let q0_ = cx.lptr(3288); let q1_ = cx.lptr(3308); let q2_ = cx.lpp(836); f_1c0480(cx, q0_, q1_, q2_) };
                                                { let q0_ = cx.lptr(3248); let q1_ = cx.lptr(4132); let q2_ = cx.lptr(3328); f_1c00c0(cx, q0_, q1_, q2_) };
                                                { let q0_ = cx.lptr(3268); let q1_ = cx.lptr(3248); let q2_ = cx.lpp(836); f_1c0480(cx, q0_, q1_, q2_) };
                                                { let t = cx.f(Ptr::raw(cx.li(3348).wrapping_add(20))); cx.set_lf(3240, t); }
                                                { let t = 0.0f32; cx.set_lf(3236, t); }
                                                'lp14: loop {
                                                    if !(cx.li(3236) < cx.i(Ptr::raw(cx.li(3240).wrapping_add(4)))) { break 'lp14; }
                                                    'blp14: {
                                                        { let t = { let q0_ = Ptr::raw(cx.li(3240)); let q1_ = cx.li(3236); let q2_ = 0; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(808, t); }
                                                        { let t = { let q0_ = Ptr::raw(cx.li(3240)); let q1_ = cx.li(3236); let q2_ = 1; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(807, t); }
                                                        { let t = { let q0_ = Ptr::raw(cx.li(3240)); let q1_ = cx.li(3236); let q2_ = 2; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(806, t); }
                                                        { let t = cx.i(Ptr::raw(cx.li(3240).wrapping_add(28))).wrapping_add((12_i32).wrapping_mul(cx.li(3236))); cx.set_li(3220, t); }
                                                        if ((cx.i(Ptr::gc(0).add(1968)) != 0) && (cx.li(4216) == 3)) {
                                                            { let q0_ = cx.lptr(3180); let q1_ = cx.lpp(808); let q2_ = cx.lpp(836); f_1c04e0(cx, q0_, q1_, q2_) };
                                                            { let q0_ = cx.lptr(3192); let q1_ = cx.lpp(807); let q2_ = cx.lpp(836); f_1c04e0(cx, q0_, q1_, q2_) };
                                                            { let q0_ = cx.lptr(3204); let q1_ = cx.lpp(806); let q2_ = cx.lpp(836); f_1c04e0(cx, q0_, q1_, q2_) };
                                                            { let q0_ = cx.lptr(3180); let q1_ = cx.lptr(3328); f_1c0000(cx, q0_, q1_) };
                                                            { let q0_ = cx.lptr(3192); let q1_ = cx.lptr(3328); f_1c0000(cx, q0_, q1_) };
                                                            { let q0_ = cx.lptr(3204); let q1_ = cx.lptr(3328); f_1c0000(cx, q0_, q1_) };
                                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1096); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                            v202 = cx.i(Ptr::glob(0x77FD58));
                                                            v139 = { let q0_ = cx.lptr(1080); let q1_ = cx.li(3192); let q2_ = cx.li(3196); let q3_ = cx.li(3200); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            v54 = { let q0_ = cx.lptr(1064); let q1_ = cx.li(3180); let q2_ = cx.li(3184); let q3_ = cx.li(3188); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v54, v139, v202, v236);
                                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1060); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                            v203 = cx.i(Ptr::glob(0x77FD58));
                                                            v140 = { let q0_ = cx.lptr(1044); let q1_ = cx.li(3204); let q2_ = cx.li(3208); let q3_ = cx.li(3212); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            v55 = { let q0_ = cx.lptr(1028); let q1_ = cx.li(3180); let q2_ = cx.li(3184); let q3_ = cx.li(3188); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v55, v140, v203, v236);
                                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(1024); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                            v204 = cx.i(Ptr::glob(0x77FD58));
                                                            v141 = { let q0_ = cx.lptr(1008); let q1_ = cx.li(3192); let q2_ = cx.li(3196); let q3_ = cx.li(3200); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            v56 = { let q0_ = cx.lptr(992); let q1_ = cx.li(3204); let q2_ = cx.li(3208); let q3_ = cx.li(3212); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v56, v141, v204, v236);
                                                            { let q0_ = cx.lptr(3160); let q1_ = cx.lptr(3180); let q2_ = cx.lptr(3192); let q3_ = cx.lptr(3204); f_1c0050(cx, q0_, q1_, q2_, q3_) };
                                                            { let q0_ = cx.lptr(3160); let q1_ = 0.33333334f32; f_1c0110(cx, q0_, q1_) };
                                                            { let q0_ = cx.lptr(3120); let q1_ = Ptr::raw(cx.li(3220)); let q2_ = cx.lpp(836); f_1c04e0(cx, q0_, q1_, q2_) };
                                                            { let q0_ = cx.lptr(3140); let q1_ = cx.lptr(3160); let q2_ = cx.lptr(3120); let q3_ = 2.0f32; f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(988); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                            v205 = cx.i(Ptr::glob(0x77FD8C));
                                                            v142 = { let q0_ = cx.lptr(972); let q1_ = cx.li(3140); let q2_ = cx.li(3144); let q3_ = cx.li(3148); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            v57 = { let q0_ = cx.lptr(956); let q1_ = cx.li(3160); let q2_ = cx.li(3164); let q3_ = cx.li(3168); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v57, v142, v205, v236);
                                                        }
                                                        { let q0_ = cx.lptr(3100); let q1_ = cx.lptr(5628); let q2_ = cx.lpp(836); f_1c0480(cx, q0_, q1_, q2_) };
                                                        if ((({ let q0_ = cx.lptr(3088); let q1_ = cx.lptr(3076); let q2_ = cx.lptr(3064); let q3_ = cx.lpp(808); let q4_ = cx.lpp(807); let q5_ = cx.lpp(806); let q6_ = cx.lptr(3268); let q7_ = cx.lptr(3100); ov_ray_triangle(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_) } != 0) && (cx.lf(3088) > 0.0f32)) && ((cx.lf(4168) + cx.lf(4172)) > cx.lf(3088))) {
                                                            if (cx.i(Ptr::gc(0).add(1972)) != 0) {
                                                                { let q0_ = cx.lptr(3020); let q1_ = cx.lpp(808); let q2_ = cx.lpp(836); f_1c04e0(cx, q0_, q1_, q2_) };
                                                                { let q0_ = cx.lptr(3032); let q1_ = cx.lpp(807); let q2_ = cx.lpp(836); f_1c04e0(cx, q0_, q1_, q2_) };
                                                                { let q0_ = cx.lptr(3044); let q1_ = cx.lpp(806); let q2_ = cx.lpp(836); f_1c04e0(cx, q0_, q1_, q2_) };
                                                                { let q0_ = cx.lptr(3020); let q1_ = cx.lptr(3328); f_1c0000(cx, q0_, q1_) };
                                                                { let q0_ = cx.lptr(3032); let q1_ = cx.lptr(3328); f_1c0000(cx, q0_, q1_) };
                                                                { let q0_ = cx.lptr(3044); let q1_ = cx.lptr(3328); f_1c0000(cx, q0_, q1_) };
                                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(952); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                                v206 = cx.i(Ptr::glob(0x77FDA8));
                                                                v143 = { let q0_ = cx.lptr(936); let q1_ = cx.li(3032); let q2_ = cx.li(3036); let q3_ = cx.li(3040); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                v58 = { let q0_ = cx.lptr(920); let q1_ = cx.li(3020); let q2_ = cx.li(3024); let q3_ = cx.li(3028); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v58, v143, v206, v236);
                                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(916); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                                v207 = cx.i(Ptr::glob(0x77FDA8));
                                                                v144 = { let q0_ = cx.lptr(900); let q1_ = cx.li(3044); let q2_ = cx.li(3048); let q3_ = cx.li(3052); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                v59 = { let q0_ = cx.lptr(884); let q1_ = cx.li(3020); let q2_ = cx.li(3024); let q3_ = cx.li(3028); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v59, v144, v207, v236);
                                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(880); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                                v208 = cx.i(Ptr::glob(0x77FDA8));
                                                                v145 = { let q0_ = cx.lptr(864); let q1_ = cx.li(3032); let q2_ = cx.li(3036); let q3_ = cx.li(3040); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                v60 = { let q0_ = cx.lptr(848); let q1_ = cx.li(3044); let q2_ = cx.li(3048); let q3_ = cx.li(3052); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v60, v145, v208, v236);
                                                                { let q0_ = cx.lptr(3000); let q1_ = cx.lptr(3020); let q2_ = cx.lptr(3032); let q3_ = cx.lptr(3044); f_1c0050(cx, q0_, q1_, q2_, q3_) };
                                                                { let q0_ = cx.lptr(3000); let q1_ = 0.33333334f32; f_1c0110(cx, q0_, q1_) };
                                                                { let q0_ = cx.lptr(2960); let q1_ = Ptr::raw(cx.li(3220)); let q2_ = cx.lpp(836); f_1c04e0(cx, q0_, q1_, q2_) };
                                                                { let q0_ = cx.lptr(2980); let q1_ = cx.lptr(3000); let q2_ = cx.lptr(2960); let q3_ = 2.0f32; f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(844); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                                v209 = cx.i(Ptr::glob(0x77FD8C));
                                                                v146 = { let q0_ = cx.lptr(828); let q1_ = cx.li(2980); let q2_ = cx.li(2984); let q3_ = cx.li(2988); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                v61 = { let q0_ = cx.lptr(812); let q1_ = cx.li(3000); let q2_ = cx.li(3004); let q3_ = cx.li(3008); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v61, v146, v209, v236);
                                                                { let q0_ = cx.lptr(2940); let q1_ = cx.lptr(3268); let q2_ = cx.lpp(836); f_1c04e0(cx, q0_, q1_, q2_) };
                                                                { let q0_ = cx.lptr(2940); let q1_ = cx.lptr(3328); f_1c0000(cx, q0_, q1_) };
                                                                { let q0_ = cx.lptr(2900); let q1_ = cx.lptr(3268); let q2_ = cx.lptr(3100); let q3_ = (((cx.lf(4168) as f64) + (cx.lf(4172) as f64)) as f32); f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                                                { let q0_ = cx.lptr(2920); let q1_ = cx.lptr(2900); let q2_ = cx.lpp(836); f_1c04e0(cx, q0_, q1_, q2_) };
                                                                { let q0_ = cx.lptr(2920); let q1_ = cx.lptr(3328); f_1c0000(cx, q0_, q1_) };
                                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(808); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                                v210 = cx.i(Ptr::glob(0x77FD84));
                                                                v147 = { let q0_ = cx.lptr(792); let q1_ = cx.li(2920); let q2_ = cx.li(2924); let q3_ = cx.li(2928); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                v62 = { let q0_ = cx.lptr(776); let q1_ = cx.li(2940); let q2_ = cx.li(2944); let q3_ = cx.li(2948); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v62, v147, v210, v236);
                                                            }
                                                            if ((-{ let q0_ = Ptr::raw(cx.li(3220)); let q1_ = cx.lptr(5628); f_1c0300(cx, q0_, q1_) }) >= (cx.f(Ptr::gc(0).add(1892)) as f64)) {
                                                                v63 = { let q0_ = cx.lptr(3100); let q1_ = Ptr::raw(cx.li(3220)); f_1c0300(cx, q0_, q1_) };
                                                                { let t = ((v63 * ((cx.lf(3088) as f64) - (cx.lf(4168) as f64))) as f32); cx.set_lf(2892, t); }
                                                                { let q0_ = a1; let q1_ = cx.lptr(2856); let q2_ = cx.li(4216); f_361610(cx, q0_, q1_, q2_) };
                                                                { let q0_ = cx.lptr(2876); let q1_ = cx.lptr(2856); let q2_ = cx.lpp(836); f_1c0480(cx, q0_, q1_, q2_) };
                                                                { let t = ({ let q0_ = Ptr::raw(cx.li(3220)); let q1_ = cx.lptr(2876); f_1c0300(cx, q0_, q1_) } as f32); cx.set_lf(2848, t); }
                                                                if ((cx.lf(2848) <= 0.999999f32) && (cx.lf(2848) >= -0.999999f32)) {
                                                                    { let q0_ = cx.lptr(2808); let q1_ = Ptr::raw(cx.li(3220)); let q2_ = cx.lptr(2876); let q3_ = ((0.0f64 - (cx.lf(2848) as f64)) as f32); f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                                                    { let t = (-{ let q0_ = a1; let q1_ = cx.li(4216); f_35fb40(cx, q0_, q1_) }); cx.set_lf(4, t); }
                                                                    v64 = ({ let q0_ = cx.lptr(2808); f_1c0330(cx, q0_) } as f64);
                                                                    v236 = (v236 & !0xFFFF_FFFFi64) | (((((cx.lf(4) as f64) / v64) as f32)).to_bits() as i64);
                                                                    { let q0_ = cx.lptr(2832); let q1_ = cx.lptr(2808); let q2_ = f32::from_bits(v236 as u32); f_1c0160(cx, q0_, q1_, q2_) };
                                                                } else {
                                                                    { let q0_ = cx.lptr(2832); f_1ba1b0(cx, q0_) };
                                                                }
                                                                if (({ let _ = { let _ = { v65 = { let q0_ = cx.lptr(2832); let q1_ = Ptr::raw(cx.li(3220)); f_1c0300(cx, q0_, q1_) }; v65 }; { { let t = (((cx.lf(2892) as f64) - v65) as f32); cx.set_lf(2824, t); } } }; ((cx.f(a1.add(4 * (cx.li(4216).wrapping_add(272)))).to_bits() as i32) == 0) } && (cx.lf(2824) > 0.0f32)) || (cx.lf(2824) > cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149)))))) {
                                                                    { let q0_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(146))); let q1_ = cx.lptr(2832); let q2_ = cx.lpp(836); f_1c04e0(cx, q0_, q1_, q2_) };
                                                                    { let t = 1; let o = cx.li(16); cx.set_li(16, (o & !(255 << 16)) | ((t & 255) << 16)); }
                                                                    { let q0_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(146))); let q1_ = cx.lptr(4152); f_1c0000(cx, q0_, q1_) };
                                                                    { let q0_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(143))); let q1_ = Ptr::raw(cx.li(3220)); let q2_ = cx.lpp(836); f_1c04e0(cx, q0_, q1_, q2_) };
                                                                    { let p = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149))); let t = cx.lf(2824); cx.set_f(p, t); }
                                                                    { let p = a1.add(4 * (cx.li(4216).wrapping_add(272))); let t = 1; cx.set_i(p, t); }
                                                                    let _ = cx.preinc_i(Ptr::raw(cx.li(5660).wrapping_add(4388)));
                                                                }
                                                            }
                                                        }
                                                    }
                                                    let _ = cx.preinc_i(cx.lptr(3236));
                                                }
                                            }
                                        }
                                        let _ = cx.preinc_i(cx.lptr(3356));
                                    }
                                }
                            } else {
                                { let q0_ = cx.lptr(2736); f_1b4660(cx, q0_) };
                                { let q0_ = cx.lptr(2788); let q1_ = { let q0_ = (((cx.lf(4208) as f64) * 1.2f64) as f32); ov_coerce_int(cx, q0_) }; f_1bfec0(cx, q0_, q1_) };
                                { let q0_ = cx.lptr(2752); f_1bff20(cx, q0_) };
                                { let q0_ = cx.lptr(2736); let q1_ = cx.lptr(4180); f_1bb190(cx, q0_, q1_) };
                                if ((cx.i(Ptr::gc(0).add(2004)) != 0) && (cx.li(4216) == 3)) {
                                    { let q0_ = cx.lptr(2736); let q1_ = Ptr::gc(0).bits(); let q2_ = cx.lptr(2632); f_37a7c0(cx, q0_, q1_, q2_) };
                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(772); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                    v211 = cx.i(Ptr::glob(0x77FDA8));
                                    v148 = { let q0_ = cx.lptr(756); let q1_ = cx.li(2644); let q2_ = cx.li(2648); let q3_ = cx.li(2652); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v66 = { let q0_ = cx.lptr(740); let q1_ = cx.li(2632); let q2_ = cx.li(2636); let q3_ = cx.li(2640); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v66, v148, v211, v236);
                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(736); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                    v212 = cx.i(Ptr::glob(0x77FDA8));
                                    v149 = { let q0_ = cx.lptr(720); let q1_ = cx.li(2656); let q2_ = cx.li(2660); let q3_ = cx.li(2664); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v67 = { let q0_ = cx.lptr(704); let q1_ = cx.li(2644); let q2_ = cx.li(2648); let q3_ = cx.li(2652); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v67, v149, v212, v236);
                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(700); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                    v213 = cx.i(Ptr::glob(0x77FDA8));
                                    v150 = { let q0_ = cx.lptr(684); let q1_ = cx.li(2668); let q2_ = cx.li(2672); let q3_ = cx.li(2676); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v68 = { let q0_ = cx.lptr(668); let q1_ = cx.li(2656); let q2_ = cx.li(2660); let q3_ = cx.li(2664); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v68, v150, v213, v236);
                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(664); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                    v214 = cx.i(Ptr::glob(0x77FDA8));
                                    v151 = { let q0_ = cx.lptr(648); let q1_ = cx.li(2632); let q2_ = cx.li(2636); let q3_ = cx.li(2640); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v69 = { let q0_ = cx.lptr(632); let q1_ = cx.li(2668); let q2_ = cx.li(2672); let q3_ = cx.li(2676); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v69, v151, v214, v236);
                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(628); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                    v215 = cx.i(Ptr::glob(0x77FDA8));
                                    v152 = { let q0_ = cx.lptr(612); let q1_ = cx.li(2692); let q2_ = cx.li(2696); let q3_ = cx.li(2700); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v70 = { let q0_ = cx.lptr(596); let q1_ = cx.li(2680); let q2_ = cx.li(2684); let q3_ = cx.li(2688); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v70, v152, v215, v236);
                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(592); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                    v216 = cx.i(Ptr::glob(0x77FDA8));
                                    v153 = { let q0_ = cx.lptr(576); let q1_ = cx.li(2704); let q2_ = cx.li(2708); let q3_ = cx.li(2712); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v71 = { let q0_ = cx.lptr(560); let q1_ = cx.li(2692); let q2_ = cx.li(2696); let q3_ = cx.li(2700); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v71, v153, v216, v236);
                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(556); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                    v217 = cx.i(Ptr::glob(0x77FDA8));
                                    v154 = { let q0_ = cx.lptr(540); let q1_ = cx.li(2716); let q2_ = cx.li(2720); let q3_ = cx.li(2724); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v72 = { let q0_ = cx.lptr(524); let q1_ = cx.li(2704); let q2_ = cx.li(2708); let q3_ = cx.li(2712); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v72, v154, v217, v236);
                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(520); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                    v218 = cx.i(Ptr::glob(0x77FDA8));
                                    v155 = { let q0_ = cx.lptr(504); let q1_ = cx.li(2680); let q2_ = cx.li(2684); let q3_ = cx.li(2688); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v73 = { let q0_ = cx.lptr(488); let q1_ = cx.li(2716); let q2_ = cx.li(2720); let q3_ = cx.li(2724); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v73, v155, v218, v236);
                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(484); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                    v219 = cx.i(Ptr::glob(0x77FDA8));
                                    v156 = { let q0_ = cx.lptr(468); let q1_ = cx.li(2680); let q2_ = cx.li(2684); let q3_ = cx.li(2688); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v74 = { let q0_ = cx.lptr(452); let q1_ = cx.li(2632); let q2_ = cx.li(2636); let q3_ = cx.li(2640); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v74, v156, v219, v236);
                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(448); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                    v220 = cx.i(Ptr::glob(0x77FDA8));
                                    v157 = { let q0_ = cx.lptr(432); let q1_ = cx.li(2692); let q2_ = cx.li(2696); let q3_ = cx.li(2700); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v75 = { let q0_ = cx.lptr(416); let q1_ = cx.li(2644); let q2_ = cx.li(2648); let q3_ = cx.li(2652); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v75, v157, v220, v236);
                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(412); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                    v221 = cx.i(Ptr::glob(0x77FDA8));
                                    v158 = { let q0_ = cx.lptr(396); let q1_ = cx.li(2704); let q2_ = cx.li(2708); let q3_ = cx.li(2712); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v76 = { let q0_ = cx.lptr(380); let q1_ = cx.li(2656); let q2_ = cx.li(2660); let q3_ = cx.li(2664); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v76, v158, v221, v236);
                                    v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(376); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                    v222 = cx.i(Ptr::glob(0x77FDA8));
                                    v159 = { let q0_ = cx.lptr(360); let q1_ = cx.li(2716); let q2_ = cx.li(2720); let q3_ = cx.li(2724); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    v77 = { let q0_ = cx.lptr(344); let q1_ = cx.li(2668); let q2_ = cx.li(2672); let q3_ = cx.li(2676); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v77, v159, v222, v236);
                                }
                                { let t = 0.0f32; cx.set_lf(2624, t); }
                                { let t = 0.0f32; cx.set_lf(2620, t); }
                                'lp15: loop {
                                    if !(cx.li(2620) < cx.li(5588)) { break 'lp15; }
                                    'blp15: {
                                        v78 = { let q0_ = Ptr::raw(cx.li(5648)); let q1_ = cx.li(2620).wrapping_add(cx.li(5600)); ov_qt_face_index(cx, q0_, q1_) };
                                        { let t = { let q0_ = Ptr::raw(cx.li(5644)); let q1_ = v78; let q2_ = 0; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(654, t); }
                                        v79 = { let q0_ = Ptr::raw(cx.li(5648)); let q1_ = cx.li(2620).wrapping_add(cx.li(5600)); ov_qt_face_index(cx, q0_, q1_) };
                                        { let t = { let q0_ = Ptr::raw(cx.li(5644)); let q1_ = v79; let q2_ = 1; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(653, t); }
                                        v80 = { let q0_ = Ptr::raw(cx.li(5648)); let q1_ = cx.li(2620).wrapping_add(cx.li(5600)); ov_qt_face_index(cx, q0_, q1_) };
                                        { let t = { let q0_ = Ptr::raw(cx.li(5644)); let q1_ = v80; let q2_ = 2; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(652, t); }
                                        v81 = { let q0_ = Ptr::raw(cx.li(5648)); let q1_ = cx.li(2620).wrapping_add(cx.li(5600)); ov_qt_face_index(cx, q0_, q1_) };
                                        { let t = cx.i(Ptr::raw(cx.li(5644).wrapping_add(28))).wrapping_add((12_i32).wrapping_mul(v81)); cx.set_li(2604, t); }
                                        if (cx.i(Ptr::gc(0).add(1956)) != 0) {
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(340); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v223 = cx.i(Ptr::glob(0x77FDA8));
                                            v160 = { let q0_ = cx.lptr(324); let q1_ = cx.i(cx.lpp(653)); let q2_ = cx.i(cx.lpp(653).add(4)); let q3_ = cx.i(cx.lpp(653).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v82 = { let q0_ = cx.lptr(308); let q1_ = cx.i(cx.lpp(654)); let q2_ = cx.i(cx.lpp(654).add(4)); let q3_ = cx.i(cx.lpp(654).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v82, v160, v223, v236);
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(304); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v224 = cx.i(Ptr::glob(0x77FDA8));
                                            v161 = { let q0_ = cx.lptr(288); let q1_ = cx.i(cx.lpp(652)); let q2_ = cx.i(cx.lpp(652).add(4)); let q3_ = cx.i(cx.lpp(652).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v83 = { let q0_ = cx.lptr(272); let q1_ = cx.i(cx.lpp(654)); let q2_ = cx.i(cx.lpp(654).add(4)); let q3_ = cx.i(cx.lpp(654).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v83, v161, v224, v236);
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(268); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v225 = cx.i(Ptr::glob(0x77FDA8));
                                            v162 = { let q0_ = cx.lptr(252); let q1_ = cx.i(cx.lpp(653)); let q2_ = cx.i(cx.lpp(653).add(4)); let q3_ = cx.i(cx.lpp(653).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v84 = { let q0_ = cx.lptr(236); let q1_ = cx.i(cx.lpp(652)); let q2_ = cx.i(cx.lpp(652).add(4)); let q3_ = cx.i(cx.lpp(652).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v84, v162, v225, v236);
                                            { let q0_ = cx.lptr(2588); let q1_ = cx.lpp(654); let q2_ = cx.lpp(653); let q3_ = cx.lpp(652); f_1c0050(cx, q0_, q1_, q2_, q3_) };
                                            { let q0_ = cx.lptr(2588); let q1_ = 0.33333334f32; f_1c0110(cx, q0_, q1_) };
                                            { let q0_ = cx.lptr(2568); let q1_ = cx.lptr(2588); let q2_ = Ptr::raw(cx.li(2604)); let q3_ = 2.0f32; f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(232); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                            v226 = cx.i(Ptr::glob(0x77FD8C));
                                            v163 = { let q0_ = cx.lptr(216); let q1_ = cx.li(2568); let q2_ = cx.li(2572); let q3_ = cx.li(2576); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            v85 = { let q0_ = cx.lptr(200); let q1_ = cx.li(2588); let q2_ = cx.li(2592); let q3_ = cx.li(2596); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                            crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v85, v163, v226, v236);
                                        }
                                        if ({ let _ = cx.lpp(654); let _ = cx.lpp(653); let _ = cx.lpp(652); let _ = cx.lptr(2736); let _ = cx.lptr(2788); unmodelled_i("FCTriangleBoxOverlap") } != 0) {
                                            if ((cx.i(Ptr::gc(0).add(1964)) != 0) && (cx.li(4216) == 3)) {
                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(196); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                v227 = cx.i(Ptr::glob(0x77FDA8));
                                                v164 = { let q0_ = cx.lptr(180); let q1_ = cx.i(cx.lpp(653)); let q2_ = cx.i(cx.lpp(653).add(4)); let q3_ = cx.i(cx.lpp(653).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                v86 = { let q0_ = cx.lptr(164); let q1_ = cx.i(cx.lpp(654)); let q2_ = cx.i(cx.lpp(654).add(4)); let q3_ = cx.i(cx.lpp(654).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v86, v164, v227, v236);
                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(160); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                v228 = cx.i(Ptr::glob(0x77FDA8));
                                                v165 = { let q0_ = cx.lptr(144); let q1_ = cx.i(cx.lpp(652)); let q2_ = cx.i(cx.lpp(652).add(4)); let q3_ = cx.i(cx.lpp(652).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                v87 = { let q0_ = cx.lptr(128); let q1_ = cx.i(cx.lpp(654)); let q2_ = cx.i(cx.lpp(654).add(4)); let q3_ = cx.i(cx.lpp(654).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v87, v165, v228, v236);
                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(124); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                v229 = cx.i(Ptr::glob(0x77FDA8));
                                                v166 = { let q0_ = cx.lptr(108); let q1_ = cx.i(cx.lpp(653)); let q2_ = cx.i(cx.lpp(653).add(4)); let q3_ = cx.i(cx.lpp(653).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                v88 = { let q0_ = cx.lptr(92); let q1_ = cx.i(cx.lpp(652)); let q2_ = cx.i(cx.lpp(652).add(4)); let q3_ = cx.i(cx.lpp(652).add(8)); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v88, v166, v229, v236);
                                                { let q0_ = cx.lptr(2548); let q1_ = cx.lpp(654); let q2_ = cx.lpp(653); let q3_ = cx.lpp(652); f_1c0050(cx, q0_, q1_, q2_, q3_) };
                                                { let q0_ = cx.lptr(2548); let q1_ = 0.33333334f32; f_1c0110(cx, q0_, q1_) };
                                                { let q0_ = cx.lptr(2528); let q1_ = cx.lptr(2548); let q2_ = Ptr::raw(cx.li(2604)); let q3_ = 2.0f32; f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                                v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(88); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                                                v230 = cx.i(Ptr::glob(0x77FD8C));
                                                v167 = { let q0_ = cx.lptr(72); let q1_ = cx.li(2528); let q2_ = cx.li(2532); let q3_ = cx.li(2536); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                v89 = { let q0_ = cx.lptr(56); let q1_ = cx.li(2548); let q2_ = cx.li(2552); let q3_ = cx.li(2556); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                                                crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v89, v167, v230, v236);
                                            }
                                            if ((-{ let q0_ = Ptr::raw(cx.li(2604)); let q1_ = cx.lptr(5628); f_1c0300(cx, q0_, q1_) }) < (cx.f(Ptr::gc(0).add(1892)) as f64)) {
                                                { let p = Ptr::glob(0x75AA38).add(4 * (cx.li(2620))); let t = 0; cx.set_i(p, t); }
                                            } else {
                                                let _ = cx.preinc_i(cx.lptr(2624));
                                                { let p = Ptr::glob(0x75AA38).add(4 * (cx.li(2620))); let t = 1; cx.set_i(p, t); }
                                            }
                                        } else {
                                            { let p = Ptr::glob(0x75AA38).add(4 * (cx.li(2620))); let t = 0; cx.set_i(p, t); }
                                        }
                                    }
                                    let _ = cx.preinc_i(cx.lptr(2620));
                                }
                                { let t = 0.0f32; cx.set_lf(2520, t); }
                                { let t = 0.0f32; cx.set_lf(2620, t); }
                                'lp16: loop {
                                    if !(cx.li(2620) < cx.li(5588)) { break 'lp16; }
                                    'blp16: {
                                        if (cx.i(Ptr::glob(0x75AA38).add(4 * (cx.li(2620)))) == 1) {
                                            v90 = { let q0_ = Ptr::raw(cx.li(5648)); let q1_ = cx.li(2620).wrapping_add(cx.li(5600)); ov_qt_face_index(cx, q0_, q1_) };
                                            { let t = { let q0_ = Ptr::raw(cx.li(5644)); let q1_ = v90; let q2_ = 0; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(629, t); }
                                            v91 = { let q0_ = Ptr::raw(cx.li(5648)); let q1_ = cx.li(2620).wrapping_add(cx.li(5600)); ov_qt_face_index(cx, q0_, q1_) };
                                            { let t = { let q0_ = Ptr::raw(cx.li(5644)); let q1_ = v91; let q2_ = 1; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(628, t); }
                                            v92 = { let q0_ = Ptr::raw(cx.li(5648)); let q1_ = cx.li(2620).wrapping_add(cx.li(5600)); ov_qt_face_index(cx, q0_, q1_) };
                                            { let t = { let q0_ = Ptr::raw(cx.li(5644)); let q1_ = v92; let q2_ = 2; ov_mesh_vertex(cx, q0_, q1_, q2_) }; cx.set_lpp(627, t); }
                                            v93 = { let q0_ = Ptr::raw(cx.li(5648)); let q1_ = cx.li(2620).wrapping_add(cx.li(5600)); ov_qt_face_index(cx, q0_, q1_) };
                                            { let t = cx.i(Ptr::raw(cx.li(5644).wrapping_add(28))).wrapping_add((12_i32).wrapping_mul(v93)); cx.set_li(2504, t); }
                                            { let q0_ = Ptr::glob(0x752638).add(4 * ((9_i32).wrapping_mul(cx.li(2520)))); let q1_ = cx.lpp(629); f_1bb190(cx, q0_, q1_) };
                                            { let q0_ = Ptr::glob(0x752638).add(4 * ((9_i32).wrapping_mul(cx.li(2520)).wrapping_add(3))); let q1_ = cx.lpp(628); f_1bb190(cx, q0_, q1_) };
                                            { let q0_ = Ptr::glob(0x752638).add(4 * ((9_i32).wrapping_mul(cx.li(2520)).wrapping_add(6))); let q1_ = cx.lpp(627); f_1bb190(cx, q0_, q1_) };
                                            { let q0_ = Ptr::glob(0x757078).add(4 * ((3_i32).wrapping_mul(cx.li(2520)))); let q1_ = Ptr::raw(cx.li(2504)); f_1bb190(cx, q0_, q1_) };
                                            { let p = Ptr::glob(0x758938).add(4 * ((4_i32).wrapping_mul(cx.li(2520)))); let t = (3_i32).wrapping_mul(cx.li(2520)); cx.set_i(p, t); }
                                            { let p = Ptr::glob(0x75893C).add(4 * ((4_i32).wrapping_mul(cx.li(2520)))); let t = (3_i32).wrapping_mul(cx.li(2520)).wrapping_add(1); cx.set_i(p, t); }
                                            { let p = Ptr::glob(0x758940).add(4 * ((4_i32).wrapping_mul(cx.li(2520)))); let t = (3_i32).wrapping_mul(cx.li(2520)).wrapping_add(2); cx.set_i(p, t); }
                                            let _ = cx.preinc_i(cx.lptr(2520));
                                        }
                                    }
                                    let _ = cx.preinc_i(cx.lptr(2620));
                                }
                                if (cx.li(2624) != 0) {
                                    v94 = f32::from_bits({ let _ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(143))); let _ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(146))); let _ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149))); let _ = cx.lptr(4180); let _ = 0; let _ = cx.i(Ptr::glob(0x752638)); let _ = cx.i(Ptr::glob(0x758938)); let _ = cx.i(Ptr::glob(0x757078)); let _ = cx.li(2624); let _ = 0; unmodelled_i("FCWheelTriangleSoupContact") } as u32);
                                    { let p = a1.add(4 * (cx.li(4216).wrapping_add(272))); let t = v94; cx.set_f(p, t); }
                                    if ((cx.f(a1.add(4 * (cx.li(4216).wrapping_add(272)))).to_bits() as i32) != 0) {
                                        { let t = (vsmath::fabs(cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(144))))) as f32); cx.set_lf(2500, t); }
                                        if (cx.f(Ptr::gc(0).add(568)) > cx.lf(2500)) {
                                            { let t = cx.f(a1.add(4 * (cx.li(4216).wrapping_add(317)))); cx.set_lf(2496, t); }
                                            { let t = (((cx.f(a1.add(4 * (cx.li(4216).wrapping_add(317)))) as f64) * (cx.f(a1.add(4 * (cx.li(4216).wrapping_add(321)))) as f64)) as f32); cx.set_lf(2492, t); }
                                            { let t = ((((cx.f(Ptr::gc(0).add(568)) - cx.lf(2500)) as f64) / (cx.f(Ptr::gc(0).add(568)) as f64)) as f32); cx.set_lf(2488, t); }
                                            { let t = ((((cx.f(Ptr::raw(cx.li(5652).wrapping_add(164))) * cx.lf(2488)) as f64) + (((1.0f32 - cx.lf(2488)) * cx.lf(2496)) as f64)) as f32); cx.set_lf(2484, t); }
                                            { let t = ((((cx.f(Ptr::raw(cx.li(5652).wrapping_add(164))) * cx.lf(2488)) as f64) + (((1.0f32 - cx.lf(2488)) * cx.lf(2492)) as f64)) as f32); cx.set_lf(2480, t); }
                                            { let p = a1.add(4 * (cx.li(4216).wrapping_add(317))); let t = cx.lf(2484); cx.set_f(p, t); }
                                            { let p = a1.add(4 * (cx.li(4216).wrapping_add(321))); let t = (((cx.lf(2480) as f64) / (cx.lf(2484) as f64)) as f32); cx.set_f(p, t); }
                                        }
                                    }
                                } else {
                                    { let p = a1.add(4 * (cx.li(4216).wrapping_add(272))); let t = 0.0f32; cx.set_f(p, t); }
                                }
                            }
                        } else {
                            { let p = a1.add(4 * (cx.li(4216).wrapping_add(272))); let t = 0.0f32; cx.set_f(p, t); }
                        }
                        if ((cx.f(a1.add(4 * (cx.li(4216).wrapping_add(272)))).to_bits() as i32) != 0) {
                            if (((cx.b(Ptr::raw(cx.li(5660).wrapping_add(4072))) != 0) && (cx.f(Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(3936))) != 0.0f32)) && (cx.f(Ptr::gc(0).add(1908)) != 0.0f32)) {
                                if (cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149)))) <= (cx.f(Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(3936))) + cx.f(Ptr::gc(0).add(1908)))) {
                                    if ((cx.f(Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(3936))) - cx.f(Ptr::gc(0).add(1908))) > cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149))))) {
                                        { let p = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149))); let t = (((cx.f(Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(3936))) as f64) - (cx.f(Ptr::gc(0).add(1908)) as f64)) as f32); cx.set_f(p, t); }
                                    }
                                } else {
                                    { let p = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149))); let t = (((cx.f(Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(3936))) as f64) + (cx.f(Ptr::gc(0).add(1908)) as f64)) as f32); cx.set_f(p, t); }
                                }
                                if (cx.f(Ptr::gc(0).add(1912)) != 0.0f32) {
                                    { let q0_ = cx.lptr(2464); let q1_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(143))); let q2_ = Ptr::raw(cx.li(5660).wrapping_add((12_i32).wrapping_mul(cx.li(4216))).wrapping_add(3952)); f_1c00c0(cx, q0_, q1_, q2_) };
                                    { let t = { let q0_ = cx.lptr(2464); f_1c0330(cx, q0_) }; cx.set_lf(2456, t); }
                                    if (cx.lf(2456) > cx.f(Ptr::gc(0).add(1912))) {
                                        { let q0_ = cx.lptr(2464); let q1_ = ((1.0f64 / (cx.lf(2456) as f64)) as f32); f_1c0110(cx, q0_, q1_) };
                                        { let q0_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(143))); let q1_ = Ptr::raw(cx.li(5660).wrapping_add((12_i32).wrapping_mul(cx.li(4216))).wrapping_add(3952)); let q2_ = cx.lptr(2464); let q3_ = cx.f(Ptr::gc(0).add(1912)); f_1c0210(cx, q0_, q1_, q2_, q3_) };
                                        { let q0_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(143))); f_1c0380(cx, q0_) };
                                    }
                                }
                            }
                            { let p = Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(3936)); let t = cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149)))); cx.set_f(p, t); }
                            { let q0_ = Ptr::raw(cx.li(5660).wrapping_add((12_i32).wrapping_mul(cx.li(4216))).wrapping_add(3952)); let q1_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(143))); f_1bb190(cx, q0_, q1_) };
                            if (cx.li(4116) != -1) {
                                { let t = 0.0f32; cx.set_lf(2452, t); }
                                if (((cx.li(4232) >> 24) & 255) != 0) {
                                    { let t = (((cx.f(cx.lptr(0).add(4 * ((13_i32).wrapping_mul(cx.li(4116)).wrapping_add(1072)))).to_bits() as i32) & 65280) >> 8); cx.set_li(2452, t); }
                                } else {
                                    { let t = (if (cx.i(Ptr::raw(cx.li(5644).wrapping_add(12))) != 0) { ((cx.u16(cx.ptr_at(Ptr::raw(cx.li(5644).wrapping_add(24))).add((8_i32).wrapping_mul(cx.li(4116))).add(6)) & 65280) >> 8) } else { ((cx.i(cx.ptr_at(Ptr::raw(cx.li(5644).wrapping_add(20))).add((16_i32).wrapping_mul(cx.li(4116))).add(12)) & 65280) >> 8) }); cx.set_li(2452, t); }
                                }
                                if (cx.li(2452) != 0) {
                                    { let t = cx.li(2452).wrapping_sub(1); cx.set_li(2448, t); }
                                    v95 = ov_physics_manager(cx);
                                    { let t = v95.add((28_i32).wrapping_mul(cx.li(2448))).add(36); cx.set_lpp(606, t); }
                                    { let q0_ = cx.lptr(2432); let q1_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(146))); let q2_ = v95.add((28_i32).wrapping_mul(cx.li(2448))).add(40); f_1c00c0(cx, q0_, q1_, q2_) };
                                    v96 = ({ let q0_ = cx.lptr(2432); f_1c0360(cx, q0_) } as f64);
                                    if (((cx.f(cx.lpp(606).add(16)) as f64) * (cx.f(cx.lpp(606).add(16)) as f64)) > v96) {
                                        { let t = (((cx.f(Ptr::raw(cx.li(5660).wrapping_add(20))) as f64) / (cx.f(Ptr::gc(0).add(600)) as f64)) as f32); cx.set_lf(2416, t); }
                                        { let q0_ = cx.lptr(2416); let q1_ = 0.0f32; let q2_ = 1.0f32; f_1bfe80(cx, q0_, q1_, q2_) };
                                        { let p = a1.add(4 * (cx.li(4216).wrapping_add(317))); let t = (((((cx.f(Ptr::gc(0).add(592)) - cx.f(Ptr::gc(0).add(596))) * cx.lf(2416)) as f64) + (cx.f(Ptr::gc(0).add(596)) as f64)) as f32); cx.set_f(p, t); }
                                        { let q0_ = a1; let q1_ = cx.lptr(2396); let q2_ = cx.li(4216); f_361450(cx, q0_, q1_, q2_) };
                                        { let t = { let q0_ = cx.lptr(2396); f_1c0330(cx, q0_) }; cx.set_lf(2388, t); }
                                        if (cx.lf(2388) > 1.0f32) {
                                            { let t = 0.0f32; cx.set_lf(2400, t); }
                                            { let q0_ = cx.lptr(2396); let q1_ = (((((cx.lf(2416) * cx.f(Ptr::gc(0).add(604))) * cx.f(Ptr::gc(0).add(4))) as f64) * (cx.f(Ptr::raw(cx.li(5652).wrapping_add(12))) as f64)) as f32); f_1c0110(cx, q0_, q1_) };
                                            { let q0_ = a1; let q1_ = cx.lptr(2396); let q2_ = cx.li(4216); f_361ab0(cx, q0_, q1_, q2_) };
                                        }
                                        let _ = cx.preinc_i(cx.lpp(1414).add(160));
                                    }
                                }
                            }
                            { let t = { let q0_ = cx.b(Ptr::raw(cx.li(4216).wrapping_add(cx.li(5660)).wrapping_add(4868))); f_19c1a0(cx, q0_) }; cx.set_lpp(596, t); }
                            { let p = a1.add(4 * (cx.li(4216).wrapping_add(317))); let t = (((cx.f(a1.add(4 * (cx.li(4216).wrapping_add(317)))) as f64) * (cx.f(cx.lpp(596)) as f64)) as f32); cx.set_f(p, t); }
                            if (cx.f(cx.lpp(596).add(4)) != 0.0f32) {
                                { let q0_ = a1; let q1_ = cx.lptr(2368); let q2_ = cx.li(4216); f_361450(cx, q0_, q1_, q2_) };
                                { let t = { let q0_ = cx.lptr(2368); f_1c0330(cx, q0_) }; cx.set_lf(2360, t); }
                                if (cx.lf(2360) > 1.0f32) {
                                    { let t = 0.0f32; cx.set_lf(2372, t); }
                                    { let q0_ = cx.lptr(2368); let q1_ = ((((cx.f(cx.lpp(596).add(4)) * cx.f(Ptr::gc(0).add(4))) as f64) * (cx.f(Ptr::raw(cx.li(5652).wrapping_add(12))) as f64)) as f32); f_1c0110(cx, q0_, q1_) };
                                    { let q0_ = a1; let q1_ = cx.lptr(2368); let q2_ = cx.li(4216); f_361ab0(cx, q0_, q1_, q2_) };
                                }
                            }
                            if (!({ let q0_ = (cx.i(Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(4884))) as f32); let q1_ = 0.0f32; let q2_ = 0.000001f32; ov_feq(cx, q0_, q1_, q2_) } != 0) || (((cx.b(Ptr::raw(cx.li(5660).wrapping_add(4072))) != 0) && (cx.i(Ptr::glob(0x74BA7C)) != 0)) && (cx.i(Ptr::raw(cx.i(Ptr::glob(0x74BA7C)).wrapping_add(12))) != 0))) {
                                v97 = { let q0_ = Ptr::glob(0x74BA7C); let q1_ = cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(146)))); let q2_ = cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(148)))); ov_bump_height(cx, q0_, q1_, q2_) };
                                { let t = ((v97 * (cx.f(cx.lpp(596).add(8)) as f64)) as f32); cx.set_lf(2356, t); }
                                if (cx.f(Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(4884))) != 0.0f32) {
                                    v236 = (v236 & !0xFFFF_FFFFi64) | ((({ let q0_ = a1; let q1_ = cx.li(4216); f_361570(cx, q0_, q1_) } as f32)).to_bits() as i64);
                                    v98 = vsmath::sin(f32::from_bits(v236 as u32));
                                    { let t = (((v98 * (cx.f(Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(4884))) as f64)) + (cx.lf(2356) as f64)) as f32); cx.set_lf(2356, t); }
                                }
                                if (cx.li(4216) == 0) {
                                    { let t = 1; cx.set_li(2352, t); }
                                }
                                { let p = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149))); let t = (((cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149)))) as f64) + (cx.lf(2356) as f64)) as f32); cx.set_f(p, t); }
                                if (cx.f(Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(4032))) != 3.4028235e38f32) {
                                    { let t = (((cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(146)))) as f64) - (cx.f(Ptr::raw(cx.li(5660).wrapping_add((8_i32).wrapping_mul(cx.li(4216))).wrapping_add(4000))) as f64)) as f32); cx.set_lf(2336, t); }
                                    { let t = (((cx.lf(2356) as f64) - (cx.f(Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(4032))) as f64)) as f32); cx.set_lf(2340, t); }
                                    { let t = (((cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(148)))) as f64) - (cx.f(Ptr::raw(cx.li(5660).wrapping_add((8_i32).wrapping_mul(cx.li(4216))).wrapping_add(4004))) as f64)) as f32); cx.set_lf(2344, t); }
                                    { let q0_ = cx.lptr(2316); let q1_ = cx.lptr(2336); let q2_ = Ptr::glob(0x688B64); f_1c0270(cx, q0_, q1_, q2_) };
                                    { let q0_ = cx.lptr(2296); let q1_ = cx.lptr(2316); let q2_ = cx.lptr(2336); f_1c0270(cx, q0_, q1_, q2_) };
                                    if (({ let q0_ = cx.lptr(2296); let q1_ = cx.lptr(2296); f_1c0410(cx, q0_, q1_) } as u8 as i32) != 0) {
                                        { let q0_ = cx.lptr(2296); let q1_ = cx.lptr(2296); let q2_ = Ptr::glob(0x688B64); f_1c05a0(cx, q0_, q1_, q2_) };
                                        { let q0_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(143))); let q1_ = cx.lptr(2296); f_1c0000(cx, q0_, q1_) };
                                        { let q0_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(143))); f_1c0380(cx, q0_) };
                                        v236 = (v236 & !0xFFFF_FFFFi64) | (((-{ let q0_ = a1; let q1_ = cx.li(4216); f_35fb40(cx, q0_, q1_) })).to_bits() as i64);
                                        { let q0_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(146))); let q1_ = cx.lptr(2296); let q2_ = f32::from_bits(v236 as u32); f_1c01b0(cx, q0_, q1_, q2_) };
                                    }
                                }
                                { let p = Ptr::raw(cx.li(5660).wrapping_add((8_i32).wrapping_mul(cx.li(4216))).wrapping_add(4000)); let t = cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(146)))); cx.set_f(p, t); }
                                { let p = Ptr::raw(cx.li(5660).wrapping_add((4_i32).wrapping_mul(cx.li(4216))).wrapping_add(4032)); let t = cx.lf(2356); cx.set_f(p, t); }
                                { let p = Ptr::raw(cx.li(5660).wrapping_add((8_i32).wrapping_mul(cx.li(4216))).wrapping_add(4004)); let t = cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(148)))); cx.set_f(p, t); }
                            }
                        } else {
                            { let p = cx.ptr_at(a1.add(4)).add((4_i32).wrapping_mul(cx.li(4216))).add(4032); let t = 2139095039; cx.set_i(p, t); }
                        }
                        v1 = (v1 & !0xFFFF_FFFFi64) | ((Ptr::gc(0).bits()) as i32 as u32 as i64);
                        v99 = cx.f(Ptr::gc(0).add(2000));
                        v104 = ov_isnan(cx, v99);
                        v100 = ((v99 < 0.0f32) as i32);
                        v101 = ((v99 == 0.0f32) as i32);
                        v102 = 0;
                        v103 = 0;
                        v1 = (v1 & !(0xFF00i64 as i64)) | ((((v8) as i64) & 0xFF) << 8) as i64;
                        if ((v99 != 0.0f32) && ((cx.f(a1.add(4 * (cx.li(4216).wrapping_add(272)))).to_bits() as i32) != 0)) {
                            { let q0_ = cx.lptr(2276); let q1_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(146))); let q2_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(143))); let q3_ = (((cx.f(Ptr::gc(0).add(2000)) as f64) * (cx.f(a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(149)))) as f64)) as f32); f_1c0210(cx, q0_, q1_, q2_, q3_) };
                            { let q0_ = cx.lptr(2256); let q1_ = a1.add(4 * ((34_i32).wrapping_mul(cx.li(4216)).wrapping_add(146))); f_1bb190(cx, q0_, q1_) };
                            v236 = (v236 & !0xFFFF_FFFFi64) | (({ let p = { let q0_ = cx.lptr(52); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) }) as i32 as u32 as i64);
                            v231 = cx.i(Ptr::glob(0x77FD48));
                            v168 = { let q0_ = cx.lptr(36); let q1_ = cx.li(2276); let q2_ = cx.li(2280); let q3_ = cx.li(2284); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                            v105 = { let q0_ = cx.lptr(20); let q1_ = cx.li(2256); let q2_ = cx.li(2260); let q3_ = cx.li(2264); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                            v1 = (v1 & !0xFFFF_FFFFi64) | (({ crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v105, v168, v231, v236); 0 }) as i32 as u32 as i64);
                        }
                    }
                    v1 = (v1 & 0xFFFF_FFFFi64) | (((cx.preinc_i(cx.lptr(4216))) as u32 as i64) << 32);
                }
                if (cx.i(cx.lpp(1414).add(64)) != 0) {
                    v1 = (v1 & 0xFFFF_FFFFi64) | (((a1.bits()) as u32 as i64) << 32);
                    if (((cx.i(a1.add(1088)) == 0) || (cx.i(a1.add(1092)) == 0)) || ((cx.i(a1.add(1096)) != 0) && { let _ = { v1 = (v1 & 0xFFFF_FFFFi64) | (((a1.bits()) as u32 as i64) << 32); }; (cx.i(a1.add(1100)) != 0) })) {
                        v1 = (v1 & !0xFFFF_FFFFi64) | ((a1.bits()) as i32 as u32 as i64);
                        if ((cx.i(a1.add(1096)) != 0) && (cx.i(a1.add(1100)) != 0)) {
                            v1 = (v1 & 0xFFFF_FFFFi64) | (((a1.bits()) as u32 as i64) << 32);
                            if ((cx.i(a1.add(1088)) == 0) || { let _ = { v1 = (v1 & !0xFFFF_FFFFi64) | ((a1.bits()) as i32 as u32 as i64); }; (cx.i(a1.add(1092)) == 0) }) {
                                v234 = (f_361500(cx, a1, 3) as f32);
                                let _ = f_361530(cx, a1, (v234.to_bits() as i32), 0);
                                v235 = (f_361500(cx, a1, 2) as f32);
                                v1 = (v1 & !0xFFFF_FFFFi64) | ((f_361530(cx, a1, (v235.to_bits() as i32), 1)) as i32 as u32 as i64);
                            }
                        }
                    } else {
                        v232 = (f_361500(cx, a1, 0) as f32);
                        let _ = f_361530(cx, a1, (v232.to_bits() as i32), 3);
                        v233 = (f_361500(cx, a1, 1) as f32);
                        v1 = (v1 & !0xFFFF_FFFFi64) | ((f_361530(cx, a1, (v233.to_bits() as i32), 2)) as i32 as u32 as i64);
                    }
                }
            }
        }
    }
    v236 = v1;
    return (v236 as i32);
}

/// VehicleEntity__1BFE80 0x001BFE80
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bfe80(cx: &mut Cx, mut a1: Ptr, a2: f32, a3: f32) -> Ptr {
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

/// VehicleEntity__1BFEC0 0x001BFEC0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bfec0(cx: &mut Cx, mut a1: Ptr, a2: i32) -> Ptr {
    { let p = a1; let t = a2; cx.set_i(p, t); }
    { let p = a1.add(4); let t = a2; cx.set_i(p, t); }
    { let p = a1.add(8); let t = a2; cx.set_i(p, t); }
    return a1;
}

/// VehicleEntity__1BFEF0 0x001BFEF0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bfef0(cx: &mut Cx, mut a1: Ptr, a2: i32) -> Ptr {
    { let p = a1; let t = a2; cx.set_i(p, t); }
    { let p = a1.add(4); let t = a2; cx.set_i(p, t); }
    { let p = a1.add(8); let t = a2; cx.set_i(p, t); }
    { let p = a1.add(12); let t = a2; cx.set_i(p, t); }
    return a1;
}

/// VehicleEntity__1BFF20 0x001BFF20
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bff20(cx: &mut Cx, a1: Ptr) -> Ptr {
    let _ = f_1bff80(cx, a1, 1065353216, 0, 0);
    let _ = f_1bff80(cx, a1.add(12), 0, 1065353216, 0);
    return f_1bff80(cx, a1.add(24), 0, 0, 1065353216);
}

/// VehicleEntity__1BFF80 0x001BFF80
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bff80(cx: &mut Cx, mut a1: Ptr, a2: i32, a3: i32, a4: i32) -> Ptr {
    { let p = a1; let t = a2; cx.set_i(p, t); }
    { let p = a1.add(4); let t = a3; cx.set_i(p, t); }
    { let p = a1.add(8); let t = a4; cx.set_i(p, t); }
    return a1;
}

/// VehicleEntity__1BFFB0 0x001BFFB0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1bffb0(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = ((0.0f64 - (cx.f(a2) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((0.0f64 - (cx.f(a2.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((0.0f64 - (cx.f(a2.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// VehicleEntity__1C0000 0x001C0000
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c0000(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    { let p = a1; let t = (((cx.f(a1) as f64) + (cx.f(a2) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a1.add(4)) as f64) + (cx.f(a2.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a1.add(8)) as f64) + (cx.f(a2.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// VehicleEntity__1C0050 0x001C0050
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c0050(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr, a4: Ptr) -> Ptr {
    { let p = a1; let t = ((((cx.f(a2) + cx.f(a3)) as f64) + (cx.f(a4) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((cx.f(a2.add(4)) + cx.f(a3.add(4))) as f64) + (cx.f(a4.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((cx.f(a2.add(8)) + cx.f(a3.add(8))) as f64) + (cx.f(a4.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a3;
}

/// VehicleEntity__1C00C0 0x001C00C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c00c0(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = (((cx.f(a2) as f64) - (cx.f(a3) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a2.add(4)) as f64) - (cx.f(a3.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a2.add(8)) as f64) - (cx.f(a3.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// VehicleEntity__1C0110 0x001C0110
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c0110(cx: &mut Cx, mut a1: Ptr, a2: f32) -> Ptr {
    { let p = a1; let t = (((cx.f(a1) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a1.add(4)) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a1.add(8)) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// VehicleEntity__1C0160 0x001C0160
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c0160(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: f32) -> Ptr {
    { let p = a1; let t = (((cx.f(a2) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a2.add(4)) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a2.add(8)) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// VehicleEntity__1C01B0 0x001C01B0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c01b0(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: f32) -> Ptr {
    { let p = a1; let t = ((((cx.f(a2) * a3) as f64) + (cx.f(a1) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((cx.f(a2.add(4)) * a3) as f64) + (cx.f(a1.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((cx.f(a2.add(8)) * a3) as f64) + (cx.f(a1.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// VehicleEntity__1C0210 0x001C0210
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c0210(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr, a4: f32) -> Ptr {
    { let p = a1; let t = ((((cx.f(a3) * a4) as f64) + (cx.f(a2) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((cx.f(a3.add(4)) * a4) as f64) + (cx.f(a2.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((cx.f(a3.add(8)) * a4) as f64) + (cx.f(a2.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a3;
}

/// VehicleEntity__1C0270 0x001C0270
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c0270(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = ((((cx.f(a2.add(4)) * cx.f(a3.add(8))) as f64) - ((cx.f(a2.add(8)) * cx.f(a3.add(4))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((cx.f(a2.add(8)) * cx.f(a3)) as f64) - ((cx.f(a2) * cx.f(a3.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((cx.f(a2) * cx.f(a3.add(4))) as f64) - ((cx.f(a2.add(4)) * cx.f(a3)) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// VehicleEntity__1C0300 0x001C0300
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c0300(cx: &mut Cx, a1: Ptr, a2: Ptr) -> f64 {
    return ((((cx.f(a1) as f64) * (cx.f(a2) as f64)) + ((cx.f(a1.add(4)) as f64) * (cx.f(a2.add(4)) as f64))) + ((cx.f(a1.add(8)) as f64) * (cx.f(a2.add(8)) as f64)));
}

/// VehicleEntity__1C0330 0x001C0330
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c0330(cx: &mut Cx, a1: Ptr) -> f32 {
    let mut v3 = 0.0f32;
    v3 = (f_1c0300(cx, a1, a1) as f32);
    return (vsmath::sqrt(v3) as f32);
}

/// VehicleEntity__1C0360 0x001C0360
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c0360(cx: &mut Cx, a1: Ptr) -> f32 {
    return (f_1c0300(cx, a1, a1) as f32);
}

/// VehicleEntity__1C0380 0x001C0380
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c0380(cx: &mut Cx, mut a1: Ptr) -> Ptr {
    let mut v2 = 0.0f32;
    v2 = f_1c0330(cx, a1);
    if (v2 != 0.0f32) {
        return f_1c0110(cx, a1, ((1.0f64 / (v2 as f64)) as f32));
    }
    { let p = a1; let t = 1065353216; cx.set_i(p, t); }
    { let p = a1.add(4); let t = 0; cx.set_i(p, t); }
    { let p = a1.add(8); let t = 0; cx.set_i(p, t); }
    return a1;
}

/// VehicleEntity__1C0410 0x001C0410
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c0410(cx: &mut Cx, a1: Ptr, a2: Ptr) -> i32 {
    let mut v3 = 0.0f32;
    v3 = f_1c0330(cx, a2);
    if (v3 == 0.0f32) {
        return 0;
    }
    let _ = f_1c0160(cx, a1, a2, ((1.0f64 / (v3 as f64)) as f32));
    return 1;
}

/// VehicleEntity__1C0480 0x001C0480
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c0480(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> () {
    { let p = a1; let t = (f_1c0300(cx, a3, a2) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (f_1c0300(cx, a3.add(16), a2) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (f_1c0300(cx, a3.add(32), a2) as f32); cx.set_f(p, t); }
}

/// VehicleEntity__1C04E0 0x001C04E0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c04e0(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = (((((cx.f(a3) * cx.f(a2)) + (cx.f(a3.add(16)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(32)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((((cx.f(a3.add(4)) * cx.f(a2)) + (cx.f(a3.add(20)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(36)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((((cx.f(a3.add(8)) * cx.f(a2)) + (cx.f(a3.add(24)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(40)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    return a3;
}

/// VehicleEntity__1C05A0 0x001C05A0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c05a0(cx: &mut Cx, a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    let mut v4 = 0.0f32;
    v4 = (f_1c0300(cx, a2, a3) as f32);
    return f_1c0210(cx, a1, a2, a3, ((0.0f64 - (v4 as f64)) as f32));
}

/// VehicleEntity__1C2FF0 0x001C2FF0
pub fn f_1c2ff0(cx: &mut Cx, this: Ptr, a2: i32, a3: Ptr, a4: Ptr) -> i64 {
    cx.push_frame(472);
    let r = f_1c2ff0_body(cx, this, a2, a3, a4);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1c2ff0_body(cx: &mut Cx, this: Ptr, a2: i32, a3: Ptr, a4: Ptr) -> i64 {
    let mut v4 = 0.0f64;
    let mut v5 = 0.0f64;
    let mut v6 = 0.0f64;
    let mut v7 = Ptr::NULL;
    let mut v9 = 0.0f64;
    let mut v10 = 0.0f32;
    let mut v11 = 0.0f32;
    let mut v12 = Ptr::NULL;
    let mut v13 = 0i64;
    let mut v15 = 0.0f32;
    let mut v16 = 0i64;
    let mut v17 = 0.0f32;
    let mut v18 = 0.0f32;
    let mut _1B6E80 = Ptr::NULL;
    { let t = this; cx.set_lpp(117, t); }
    { let t = ov_hist_entry(cx, this.add(3560), 0); cx.set_lpp(116, t); }
    { let q0_ = cx.lpp(117); let q1_ = cx.lptr(396); f_1bb020(cx, q0_, q1_) };
    if (((cx.gc.skid_exaggeration_enabled != 0) && (cx.b(cx.lpp(117).add(4072)) != 0)) && (cx.f(cx.lpp(117).add(3772)) != 0.0f32)) {
        { let t = cx.f(cx.lpp(117).add(3772)); cx.set_lf(376, t); }
        { let t = cx.f(cx.lpp(117).add(3704)); cx.set_lf(380, t); }
        { let t = cx.f(cx.lpp(116).add(88)); cx.set_lf(384, t); }
        { let t = { let q0_ = cx.lptr(364); let q1_ = 0; let q2_ = Ptr::gexpr(476); let q3_ = cx.lptr(376); ov_eval_expr(cx, q0_, q1_, q2_, q3_) }; cx.set_li(356, t); }
        if ((cx.li(356) != 0) || (cx.lf(364) == 1.0f32)) {
            { let p = cx.lpp(117).add(4576); let t = 0; cx.set_i(p, t); }
        } else {
            { let p = cx.lpp(117).add(4576); let t = ((((cx.lf(364) - 1.0f32) as f64) * (cx.f(cx.lpp(117).add(3772)) as f64)) as f32); cx.set_f(p, t); }
        }
    }
    if ((cx.f(cx.lpp(117).add(4576)) == 0.0f32) && ((cx.gc.skid_angle_to_ignore as f64) > vsmath::fabs(cx.f(cx.lpp(117).add(4580))))) {
        if (cx.i(cx.lpp(117).add(4756)) != 1) {
            { let t = 0.0f32; cx.set_lf(352, t); }
            'lp3: loop {
                if !(cx.li(352) < 4) { break 'lp3; }
                'blp3: {
                    { let q0_ = cx.lpp(117).add(240); let q1_ = cx.lpp(117).add((cx.li(352) << 6)).add(4132); let q2_ = cx.li(352); f_36e5d0(cx, q0_, q1_, q2_) };
                }
                let _ = cx.preinc_i(cx.lptr(352));
            }
        }
    } else {
        if (cx.gc.target_faked_skid_angle_smooth_tau == 0.0f32) {
            { let p = cx.lpp(117).add(4580); let t = cx.i(cx.lpp(117).add(4576)); cx.set_i(p, t); }
        } else {
            { let t = cx.ptr_at(cx.lpp(117).add(3552)); cx.set_lpp(87, t); }
            { let t = cx.f(cx.lpp(117).add(4576)); cx.set_lf(344, t); }
            { let t = (((cx.f(cx.lpp(117).add(4584)) as f64) - (cx.lf(344) as f64)) as f32); cx.set_lf(340, t); }
            { let t = (((cx.gc.target_faked_skid_angle_smooth_tau as f64) * (cx.f(cx.lpp(87)) as f64)) as f32); cx.set_lf(336, t); }
            { let t = cx.f(cx.lpp(117).add(4588)); cx.set_lf(332, t); }
            { let t = ((1.0f64 / (cx.gc.target_faked_skid_angle_smooth_tau as f64)) as f32); cx.set_lf(328, t); }
            { let t = (vsmath::exp(((0.0f64 - (cx.lf(328) as f64)) as f32)) as f32); cx.set_lf(324, t); }
            { let t = ((((((cx.lf(328) + 1.0f32) * cx.lf(340)) + (cx.lf(332) * cx.f(cx.lpp(87)))) as f64) * (cx.lf(324) as f64)) as f32); cx.set_lf(320, t); }
            { let p = cx.lpp(117).add(4588); let t = ((((((1.0f32 - cx.lf(328)) * cx.lf(332)) - ((cx.lf(340) * cx.lf(328)) / cx.lf(336))) as f64) * (cx.lf(324) as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lpp(117).add(4584); let t = ((((cx.lf(320) - cx.lf(340)) as f64) + (cx.f(cx.lpp(117).add(4584)) as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lpp(117).add(4580); let t = cx.i(cx.lpp(117).add(4584)); cx.set_i(p, t); }
        }
        { let t = cx.lptr(396); cx.set_lpp(79, t); }
        { let t = cx.lptr(428); cx.set_lpp(78, t); }
        { let q0_ = cx.lptr(276); let q1_ = cx.lptr(396); f_1bb190(cx, q0_, q1_) };
        { let q0_ = cx.lptr(256); let q1_ = cx.lpp(78); f_1bb190(cx, q0_, q1_) };
        { let t = (vsmath::cos(cx.f(cx.lpp(117).add(4580))) as f32); cx.set_lf(248, t); }
        { let t = (vsmath::sin(cx.f(cx.lpp(117).add(4580))) as f32); cx.set_lf(244, t); }
        { let q0_ = cx.lptr(296); let q1_ = cx.lpp(78); let q2_ = cx.lf(248); let q3_ = cx.lpp(79); let q4_ = cx.lf(244); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
        { let q0_ = cx.lpp(79); let q1_ = cx.lpp(78); let q2_ = ((0.0f64 - (cx.lf(244) as f64)) as f32); let q3_ = cx.lpp(79); let q4_ = cx.lf(248); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
        { let q0_ = cx.lpp(78); let q1_ = cx.lptr(296); f_1bb190(cx, q0_, q1_) };
        { let q0_ = cx.lpp(117).add(240); let q1_ = cx.li(396); let q2_ = cx.li(400); let q3_ = cx.li(404); let q4_ = cx.li(412); let q5_ = cx.li(416); let q6_ = cx.li(420); let q7_ = cx.li(428); let q8_ = cx.li(432); let q9_ = cx.li(436); f_36a460(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_) };
        { let q0_ = cx.lpp(117).add(240); f_362970(cx, q0_) };
        { let t = 0.0f32; cx.set_lf(240, t); }
        'lp4: loop {
            if !(cx.li(240) < 4) { break 'lp4; }
            'blp4: {
                { let q0_ = cx.lpp(117).add(240); let q1_ = cx.lpp(117).add((cx.li(240) << 6)).add(4132); let q2_ = cx.li(240); f_36e5d0(cx, q0_, q1_, q2_) };
            }
            let _ = cx.preinc_i(cx.lptr(240));
        }
        { let q0_ = cx.lpp(117).add(240); let q1_ = cx.li(276); let q2_ = cx.li(280); let q3_ = cx.li(284); let q4_ = cx.li(412); let q5_ = cx.li(416); let q6_ = cx.li(420); let q7_ = cx.li(256); let q8_ = cx.li(260); let q9_ = cx.li(264); f_36a460(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_) };
        { let _ = { let q0_ = cx.lpp(117).add(240); vcall(cx, 32, q0_, &[]) }; };
        { let p = cx.lpp(117).add(4576); let t = 0; cx.set_i(p, t); }
    }
    if (cx.b(cx.lpp(117).add(4072)) != 0) {
        if (cx.i(cx.lpp(116).add(64)) != 0) {
            { let t = 0.0f32; cx.set_lf(236, t); }
        } else if (cx.i(cx.lpp(117).add(3828)) != 0) {
            { let t = cx.gc.player_lean_reverse_cheat_in_zone; cx.set_lf(236, t); }
        } else {
            { let t = cx.gc.player_lean_reverse_cheat; cx.set_lf(236, t); }
        }
    } else {
        { let t = cx.gc.AI_lean_reverse_cheat; cx.set_lf(236, t); }
    }
    if (cx.lf(236) == 0.0f32) {
        { let p = cx.lpp(117).add(4392); let t = 0; cx.set_i(p, t); }
    } else {
        if ({ let q0_ = cx.lpp(117).add(240); f_364be0(cx, q0_) } < cx.gc.necessary_number_of_touching_wheels) {
            if (cx.i(cx.lpp(117).add(3572)) != 0) {
                _1B6E80 = { let q0_ = cx.lpp(117).add(3560); let q1_ = 1; ov_hist_entry(cx, q0_, q1_) };
                { let p = cx.lpp(116).add(60); let t = (((cx.f(_1B6E80.add(60)) as f64) * (cx.gc.portion_of_lean_change_to_apply_in_one_frame as f64)) as f32); cx.set_f(p, t); }
            } else {
                { let p = cx.lpp(116).add(60); let t = 0; cx.set_i(p, t); }
            }
        } else {
            { let t = 0.0f32; cx.set_lf(232, t); }
            if (({ let q0_ = cx.lpp(117).add(240); let q1_ = 0; f_364a90(cx, q0_, q1_) } != -1) && ({ let q0_ = cx.lpp(117).add(240); let q1_ = 3; f_364a90(cx, q0_, q1_) } != -1)) {
                { let t = ({ let q0_ = cx.lpp(117).add(240); let q1_ = 0; f_3649d0(cx, q0_, q1_) } as f32); cx.set_lf(4, t); }
                v4 = { let q0_ = cx.lpp(117).add(240); let q1_ = 3; f_3649d0(cx, q0_, q1_) };
                { let t = (((((cx.lf(4) as f64) - v4) / (cx.f(cx.ptr_at(cx.lpp(117).add(3552)).add(32)) as f64)) + (cx.lf(232) as f64)) as f32); cx.set_lf(232, t); }
            }
            if (({ let q0_ = cx.lpp(117).add(240); let q1_ = 1; f_364a90(cx, q0_, q1_) } != -1) && ({ let q0_ = cx.lpp(117).add(240); let q1_ = 2; f_364a90(cx, q0_, q1_) } != -1)) {
                { let t = ({ let q0_ = cx.lpp(117).add(240); let q1_ = 1; f_3649d0(cx, q0_, q1_) } as f32); cx.set_lf(0, t); }
                v5 = { let q0_ = cx.lpp(117).add(240); let q1_ = 2; f_3649d0(cx, q0_, q1_) };
                { let t = (((((cx.lf(0) as f64) - v5) / (cx.f(cx.ptr_at(cx.lpp(117).add(3552)).add(36)) as f64)) + (cx.lf(232) as f64)) as f32); cx.set_lf(232, t); }
            }
            v6 = vsmath::asin((((cx.lf(232) as f64) * 0.5f64) as f32));
            { let p = cx.lpp(116).add(60); let t = (v6 as f32); cx.set_f(p, t); }
            if ((cx.f(cx.lpp(116).add(60)) == 0.0f32) && (cx.i(cx.lpp(117).add(3572)) != 0)) {
                v7 = { let q0_ = cx.lpp(117).add(3560); let q1_ = 1; ov_hist_entry(cx, q0_, q1_) };
                { let p = cx.lpp(116).add(60); let t = (((cx.f(v7.add(60)) as f64) * (cx.gc.portion_of_lean_change_to_apply_in_one_frame as f64)) as f32); cx.set_f(p, t); }
            }
        }
        { let t = cx.lptr(428); cx.set_lpp(57, t); }
        { let t = cx.lptr(412); cx.set_lpp(56, t); }
        { let t = (((cx.f(cx.lpp(116).add(60)) as f64) * (cx.lf(236) as f64)) as f32); cx.set_lf(200, t); }
        { let t = (vsmath::cos(cx.lf(200)) as f32); cx.set_lf(196, t); }
        { let t = (vsmath::sin(cx.lf(200)) as f32); cx.set_lf(192, t); }
        { let q0_ = cx.lptr(208); let q1_ = cx.lpp(57); let q2_ = cx.lf(196); let q3_ = cx.lpp(56); let q4_ = cx.lf(192); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
        { let q0_ = cx.lpp(56); let q1_ = cx.lpp(57); let q2_ = ((0.0f64 - (cx.lf(192) as f64)) as f32); let q3_ = cx.lpp(56); let q4_ = cx.lf(196); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
        { let q0_ = cx.lpp(57); let q1_ = cx.lptr(208); f_1bb190(cx, q0_, q1_) };
        v17 = (vsmath::sin(cx.lf(200)) as f32);
        v9 = vsmath::fabs(v17);
        { let p = cx.lpp(117).add(4392); let t = (((-v9) * (cx.gc.sideways_rotation_shift as f64)) as f32); cx.set_f(p, t); }
    }
    if (cx.b(cx.lpp(117).add(4072)) != 0) {
        if (cx.f(cx.ptr_at(cx.lpp(117).add(3552)).add(124)) != 0.0f32) {
            { let t = { let q0_ = cx.lpp(117); f_24870(cx, q0_) }; cx.set_lf(188, t); }
            { let t = (((cx.lf(188) as f64) - (cx.f(cx.lpp(117).add(36)) as f64)) as f32); cx.set_lf(184, t); }
            if ((cx.f(cx.lpp(116).add(48)) == 0.0f32) || (cx.i(cx.lpp(116).add(44)) == 0)) {
                { let t = (((cx.lf(184) as f64) * (cx.f(cx.ptr_at(cx.lpp(117).add(3552)).add(124)) as f64)) as f32); cx.set_lf(184, t); }
            } else {
                { let t = ((((0.0f32 - cx.f(cx.ptr_at(cx.lpp(117).add(3552)).add(124))) as f64) * (cx.lf(184) as f64)) as f32); cx.set_lf(184, t); }
            }
            { let t = vsmath::clamp(cx.lf(184), cx.gc.fMinLeanFB, cx.gc.fMaxLeanFB); cx.set_lf(184, t); }
            { let t = (((cx.lf(184) as f64) * (cx.f(cx.lpp(117).add(4060)) as f64)) as f32); cx.set_lf(184, t); }
            { let t = (((cx.lf(184) as f64) - (cx.f(cx.lpp(117).add(40)) as f64)) as f32); cx.set_lf(180, t); }
            if (vsmath::fabs(cx.lf(180)) > (cx.gc.fStepPreFrameFB as f64)) {
                if (cx.f(cx.lpp(117).add(40)) <= cx.lf(184)) {
                    v10 = (((cx.f(cx.lpp(117).add(40)) as f64) + (cx.gc.fStepPreFrameFB as f64)) as f32);
                } else {
                    v10 = (((cx.f(cx.lpp(117).add(40)) as f64) - (cx.gc.fStepPreFrameFB as f64)) as f32);
                }
                { let t = v10; cx.set_lf(184, t); }
            }
            { let t = cx.lptr(396); cx.set_lpp(44, t); }
            { let t = cx.lptr(412); cx.set_lpp(43, t); }
            { let t = cx.lf(184); cx.set_lf(148, t); }
            { let t = (vsmath::cos(cx.lf(184)) as f32); cx.set_lf(144, t); }
            { let t = (vsmath::sin(cx.lf(148)) as f32); cx.set_lf(140, t); }
            { let q0_ = cx.lptr(156); let q1_ = cx.lpp(44); let q2_ = cx.lf(144); let q3_ = cx.lpp(43); let q4_ = cx.lf(140); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
            { let q0_ = cx.lpp(43); let q1_ = cx.lpp(44); let q2_ = ((0.0f64 - (cx.lf(140) as f64)) as f32); let q3_ = cx.lpp(43); let q4_ = cx.lf(144); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
            { let q0_ = cx.lpp(44); let q1_ = cx.lptr(156); f_1bb190(cx, q0_, q1_) };
            { let p = cx.lpp(117).add(40); let t = cx.lf(184); cx.set_f(p, t); }
            { let p = cx.lpp(117).add(36); let t = cx.lf(188); cx.set_f(p, t); }
        }
        if (cx.f(cx.ptr_at(cx.lpp(117).add(3552)).add(128)) != 0.0f32) {
            { let t = cx.gc.maximum_steering_angle_ever; cx.set_lf(136, t); }
            { let t = ({ let q0_ = cx.lpp(117).add(240); f_362440(cx, q0_) } as f32); cx.set_lf(132, t); }
            { let t = (((cx.lf(132) as f64) / (cx.lf(136) as f64)) as f32); cx.set_lf(128, t); }
            { let t = { let q0_ = cx.lpp(117); f_24220(cx, q0_) }; cx.set_lf(124, t); }
            if (cx.lf(124) < 0.0f32) {
                { let t = 0.0f32; cx.set_lf(124, t); }
            }
            if (cx.gc.fMinLRSpeed <= cx.lf(124)) {
                if (cx.lf(124) <= cx.gc.fMaxLRSpeed) {
                    { let t = ((((cx.lf(124) - cx.gc.fMinLRSpeed) as f64) / ((cx.gc.fMaxLRSpeed - cx.gc.fMinLRSpeed) as f64)) as f32); cx.set_lf(120, t); }
                } else {
                    { let t = 1.0f32; cx.set_lf(120, t); }
                }
            } else {
                { let t = 0.0f32; cx.set_lf(120, t); }
                { let p = cx.lpp(117).add(44); let t = 0; cx.set_i(p, t); }
            }
            { let t = (((cx.lf(120) as f64) * (cx.lf(128) as f64)) as f32); cx.set_lf(116, t); }
            { let t = ((((cx.lf(120) * cx.lf(128)) as f64) - (cx.f(cx.lpp(117).add(44)) as f64)) as f32); cx.set_lf(112, t); }
            if (vsmath::fabs(cx.lf(112)) > (cx.gc.fMaxLeanPerFrameLR as f64)) {
                if (cx.f(cx.lpp(117).add(44)) <= cx.lf(116)) {
                    v11 = (((cx.f(cx.lpp(117).add(44)) as f64) + (cx.gc.fMaxLeanPerFrameLR as f64)) as f32);
                } else {
                    v11 = (((cx.f(cx.lpp(117).add(44)) as f64) - (cx.gc.fMaxLeanPerFrameLR as f64)) as f32);
                }
                { let t = v11; cx.set_lf(116, t); }
            }
            { let t = (((cx.lf(116) as f64) * 0.5f64) as f32); cx.set_lf(116, t); }
            { let t = ((((cx.f(cx.lpp(117).add(44)) * 0.5f32) as f64) + (cx.lf(116) as f64)) as f32); cx.set_lf(116, t); }
            { let t = vsmath::clamp(cx.lf(116), cx.gc.fMinLeanLR, cx.gc.fMaxLeanLR); cx.set_lf(116, t); }
            { let t = cx.lptr(428); cx.set_lpp(27, t); }
            { let t = cx.lptr(412); cx.set_lpp(26, t); }
            { let t = (((cx.lf(116) as f64) * (cx.f(cx.ptr_at(cx.lpp(117).add(3552)).add(128)) as f64)) as f32); cx.set_lf(80, t); }
            { let t = (vsmath::cos(cx.lf(80)) as f32); cx.set_lf(76, t); }
            { let t = (vsmath::sin(cx.lf(80)) as f32); cx.set_lf(72, t); }
            { let q0_ = cx.lptr(88); let q1_ = cx.lpp(27); let q2_ = cx.lf(76); let q3_ = cx.lpp(26); let q4_ = cx.lf(72); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
            { let q0_ = cx.lpp(26); let q1_ = cx.lpp(27); let q2_ = ((0.0f64 - (cx.lf(72) as f64)) as f32); let q3_ = cx.lpp(26); let q4_ = cx.lf(76); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
            { let q0_ = cx.lpp(27); let q1_ = cx.lptr(88); f_1bb190(cx, q0_, q1_) };
            { let p = cx.lpp(117).add(44); let t = cx.lf(116); cx.set_f(p, t); }
        }
    }
    { let q0_ = cx.lptr(56); let q1_ = cx.lptr(428); f_1bb190(cx, q0_, q1_) };
    { let q0_ = cx.lptr(428); let q1_ = cx.lptr(396); f_1bb190(cx, q0_, q1_) };
    { let q0_ = cx.lptr(396); let q1_ = cx.lptr(56); f_1bffb0(cx, q0_, q1_) };
    { let q0_ = cx.lptr(32); let q1_ = cx.lptr(396); f_39d710(cx, q0_, q1_) };
    v18 = (((((0.0f32 - cx.f(cx.ptr_at(cx.lpp(117).add(3552)).add(116))) - cx.gc.COM_pos_y_WRT_graphics_pivot) as f64) + (cx.f(cx.lpp(117).add(4392)) as f64)) as f32);
    v15 = ((((0.0f32 - cx.f(cx.ptr_at(cx.lpp(117).add(3552)).add(112))) as f64) - (cx.gc.COM_pos_x_WRT_graphics_pivot as f64)) as f32);
    v12 = { let q0_ = cx.lpp(117).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
    { let q0_ = cx.lptr(12); let q1_ = v12; let q2_ = v15; let q3_ = cx.lptr(412); let q4_ = v18; f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
    { let q0_ = a3; let q1_ = { let q0_ = (((cx.lf(444) as f64) + (cx.lf(12) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = { let q0_ = (((cx.lf(448) as f64) + (cx.lf(16) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q3_ = { let q0_ = (((cx.lf(452) as f64) + (cx.lf(20) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
    v13 = (v13 & !0xFFFF_FFFFi64) | (({ let q0_ = a4; let q1_ = { let q0_ = ((0.0f64 - (cx.lf(32) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = { let q0_ = ((0.0f64 - (cx.lf(36) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q3_ = { let q0_ = ((0.0f64 - (cx.lf(40) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q4_ = cx.li(44); ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) }.bits()) as i32 as u32 as i64);
    v16 = v13;
    return v16;
}

/// VehicleEntity__1C4070 0x001C4070
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c4070(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: f32, a4: Ptr, a5: f32) -> Ptr {
    { let p = a1; let t = ((((cx.f(a2) * a3) as f64) + ((cx.f(a4) * a5) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((cx.f(a2.add(4)) * a3) as f64) + ((cx.f(a4.add(4)) * a5) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((cx.f(a2.add(8)) * a3) as f64) + ((cx.f(a4.add(8)) * a5) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// VehicleEntity__1C5080 0x001C5080
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c5080(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1.add(12); let t = ((((((cx.f(a2.add(12)) * cx.f(a3.add(12))) + (cx.f(a3) * cx.f(a2))) + (cx.f(a3.add(4)) * cx.f(a2.add(4)))) as f64) + ((cx.f(a3.add(8)) * cx.f(a2.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1; let t = ((((((cx.f(a3.add(4)) * cx.f(a2.add(8))) - (cx.f(a3.add(8)) * cx.f(a2.add(4)))) - (cx.f(a3) * cx.f(a2.add(12)))) as f64) + ((cx.f(a2) * cx.f(a3.add(12))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((((cx.f(a3.add(8)) * cx.f(a2)) - (cx.f(a3) * cx.f(a2.add(8)))) - (cx.f(a3.add(4)) * cx.f(a2.add(12)))) as f64) + ((cx.f(a2.add(4)) * cx.f(a3.add(12))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((((cx.f(a3) * cx.f(a2.add(4))) - (cx.f(a3.add(4)) * cx.f(a2))) - (cx.f(a3.add(8)) * cx.f(a2.add(12)))) as f64) + ((cx.f(a2.add(8)) * cx.f(a3.add(12))) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// VehicleEntity__1C56A0 0x001C56A0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c56a0(cx: &mut Cx, this: Ptr) -> i32 {
    let mut v2 = 0i32;
    let mut result = 0i32;
    { let q0_ = cx.ptr_at(this.add(5108)); f_1c8900(cx, q0_) };
    { let q0_ = cx.ptr_at(this.add(5112)); f_1c8900(cx, q0_) };
    { let q0_ = cx.ptr_at(this.add(5116)); f_1c8900(cx, q0_) };
    result = (this.bits() as u8 as i32);
    if (cx.i(this.add(5096)) != 0) {
        v2 = cx.i(this.add(5096));
        crate::ov_ignore!(cx);
        crate::ov_ignore!(cx, v2);
        return 0;
    }
    return result;
}

/// VehicleEntity__1C5750 0x001C5750
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c5750(cx: &mut Cx, this: Ptr) -> Ptr {
    let mut result = Ptr::NULL;
    result = this;
    if (cx.b(this.add(28)) != 0) {
        { let p = this.add(28); let t = 0; cx.set_b(p, t); }
    }
    return result;
}

/// VehicleEntity__1C5800 0x001C5800
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c5800(cx: &mut Cx, this: Ptr, a2: f32, a3: i32, a4: i32) -> i32 {
    let mut v4 = 0i32;
    let mut v7 = 0i32;
    let mut v9 = 0.0f32;
    let mut v10 = 0i32;
    let mut v11 = 0.0f32;
    let mut v12 = 0.0f32;
    let mut v13 = 0i32;
    let mut v14 = 0.0f32;
    let mut v15 = 0.0f32;
    let mut v16 = 0.0f32;
    let mut v17 = 0.0f32;
    let mut v18 = 0.0f32;
    let mut v19 = Ptr::NULL;
    let mut Instance = Ptr::NULL;
    let mut RaceManager = Ptr::NULL;
    v4 = f_1c6a80(cx, this);
    if (v4 != 2) {
        if (cx.f(this.add(4908)) > 0.0f32) {
            { let p = this.add(4908); let t = (((cx.f(this.add(4908)) as f64) - (a2 as f64)) as f32); cx.set_f(p, t); }
            if (cx.f(this.add(4908)) <= 0.0f32) {
                { let p = this.add(4904); let t = 1065353216; cx.set_i(p, t); }
            }
        }
        if (cx.f(this.add(4916)) > 0.0f32) {
            { let p = this.add(4916); let t = (((cx.f(this.add(4916)) as f64) - (a2 as f64)) as f32); cx.set_f(p, t); }
            if (cx.f(this.add(4916)) <= 0.0f32) {
                { let p = this.add(4912); let t = 1065353216; cx.set_i(p, t); }
            }
        }
        if (cx.f(this.add(48)) > 0.0f32) {
            { let p = this.add(48); let t = (((cx.f(this.add(48)) as f64) - (a2 as f64)) as f32); cx.set_f(p, t); }
            if (cx.f(this.add(48)) < 0.0f32) {
                { let p = this.add(48); let t = 0; cx.set_i(p, t); }
            }
        }
        { let p = this.add(3536); let t = ((cx.f(this.add(48)) > 0.0f32) as i32); cx.set_i(p, t); }
        v10 = cx.i(this.add(5072));
        Instance = ov_ai_world(cx);
        RaceManager = ov_race_manager(cx, Instance);
        v19 = ov_score_record(cx, RaceManager, v10);
        if (v19.bits() != Ptr::NULL.bits()) {
            v17 = f_248f0(cx, this);
            v15 = (f_1da710(cx, this) as f32);
            v18 = (((v17 as f64) + ({ let p = ov_score_manager(cx, v19).add(160); cx.f(p) } as f64)) as f32);
            v16 = (((v15 as f64) + ({ let p = ov_score_manager(cx, v19).add(164); cx.f(p) } as f64)) as f32);
            if (f_1da6c0(cx, this) <= (v18 as f64)) {
                v11 = (f_1da6c0(cx, this) as f32);
                let _ = f_1da6e0(cx, this, v11);
            } else {
                let _ = f_1da6e0(cx, this, v18);
            }
            if (f_1da730(cx, this) <= (v16 as f64)) {
                v12 = (f_1da730(cx, this) as f32);
                crate::ov_ignore!(cx, this, v12);
            } else {
                crate::ov_ignore!(cx, this, v16);
            }
        }
        if (cx.i(this.add(4876)) != 0) {
            let _ = cx.predec_i(this.add(4876));
        }
        if ((cx.i(Ptr::glob(0x749110)) == 2) && (cx.i(this.add(5084)) == 0)) {
            if (cx.f(Ptr::glob(0x749118)) < (cx.f(Ptr::glob(0x74911C)) - cx.f(Ptr::glob(0x688A00)))) {
                { let p = Ptr::glob(0x68580C); let t = (cx.f(Ptr::glob(0x6889F8)).to_bits() as i32); cx.set_i(p, t); }
            } else {
                v14 = vsmath::clamp(((((cx.f(Ptr::glob(0x74911C)) - cx.f(Ptr::glob(0x749118))) as f64) / (cx.f(Ptr::glob(0x688A00)) as f64)) as f32), 0.0f32, 1.0f32);
                { let p = Ptr::glob(0x68580C); let t = (((vsmath::sin((((v14 as f64) * 1.5707964f64) as f32)) * ((cx.f(Ptr::glob(0x6889F8)) as f64) - (cx.f(Ptr::glob(0x688A00).add(4 * (4))) as f64))) + (cx.f(Ptr::glob(0x688A00).add(4 * (4))) as f64)) as f32); cx.set_f(p, t); }
            }
            if (cx.f(Ptr::glob(0x749118)) >= (cx.f(Ptr::glob(0x74911C)) * cx.f(Ptr::glob(0x688A0C)))) {
                { let q0_ = cx.i(Ptr::glob(0x730F30)); let q1_ = cx.i(this.add(5088)); let q2_ = cx.f(Ptr::glob(0x688A08)); ov_slipstream(cx, q0_, q1_, q2_) };
            }
        }
        if (cx.i(this.add(5104)) != 0) {
            if (({ let q0_ = cx.ptr_at(this.add(5104)); ov_input_kind(cx, q0_) } != 0) && ({ let q0_ = cx.ptr_at(this.add(5104)); ov_input_kind(cx, q0_) } != 1)) {
                { let q0_ = this; let q1_ = this.bits(); let q2_ = cx.i(cx.ptr_at(this.add(5104)).add(24)); f_1cbbb0(cx, q0_, q1_, q2_) };
                if ((f_1c6a80(cx, this) == 4) || (f_1c6a80(cx, this) == 1)) {
                    let _ = f_1d1850(cx, this);
                }
            } else {
                let _ = f_1ba8a0(cx, this);
                if (a3 == -1) {
                    { let q0_ = this; let q1_ = a2; let q2_ = cx.f(this.add(164)); f_1ba770(cx, q0_, q1_, q2_) };
                    let _ = f_1cb470(cx, this, this.add(156));
                    let _ = f_1cf0d0(cx, this);
                } else {
                    if (({ let q0_ = cx.ptr_at(this.add(5104)); ov_input_kind(cx, q0_) } == 0) || ({ let q0_ = cx.ptr_at(this.add(5104)); ov_input_kind(cx, q0_) } == 1)) {
                        v9 = f_24870(cx, this);
                        f_1b82a0(cx, this.add(156), a3, a2, a3, v9, 0, a4);
                    }
                    let _ = f_1cb530(cx, this, v7, a3);
                    { let q0_ = this; let q1_ = a2; let q2_ = cx.f(this.add(164)); f_1ba770(cx, q0_, q1_, q2_) };
                    let _ = f_1d1850(cx, this);
                }
            }
        }
        v4 = (v4 & !(0xFFi64 as i32)) | ((((a3) as i64) & 0xFF) << 0) as i32;
        { let p = this.add(92); let t = a3; cx.set_i(p, t); }
        if ((cx.i(this.add(5104)) != 0) && (({ let q0_ = cx.ptr_at(this.add(5104)); ov_input_kind(cx, q0_) } == 1) || ({ v4 = { let q0_ = cx.ptr_at(this.add(5104)); ov_input_kind(cx, q0_) }; v4 } == 4))) {
            v13 = cx.i(cx.ptr_at(this.add(5104)).add(24));
            crate::ov_ignore!(cx);
            v4 = (v4 & !(0xFFi64 as i32)) | (((({ crate::ov_ignore!(cx, v13); 0 }) as i64) & 0xFF) << 0) as i32;
        }
        if ((cx.b(this.add(4072)) != 0) && (a3 != -1)) {
            v4 = (v4 & !(0xFFi64 as i32)) | ((((f_1d1680(cx, this, a2, a3)) as i64) & 0xFF) << 0) as i32;
        }
    }
    return v4;
}

/// VehicleEntity__1C6480 0x001C6480
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c6480(cx: &mut Cx, this: Ptr) -> Ptr {
    let mut v2 = Ptr::NULL;
    if ((cx.i(Ptr::glob(0x75B52C)) & 1) == 0) {
        { let p = Ptr::glob(0x75B52C); let t = cx.i(p).bitor_(1); cx.set_i(p, t); }
        let _ = ov_vec4_xyzw(cx, Ptr::glob(0x75B51C), 0, 0, 0, 1065353216);
    }
    v2 = ov_car_velocity(cx, this.add(240));
    { let q0_ = Ptr::glob(0x75B51C); let q1_ = cx.i(v2); let q2_ = cx.i(v2.add(4)); let q3_ = cx.i(v2.add(8)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
    return Ptr::glob(0x75B51C);
}

/// VehicleEntity__1C66C0 0x001C66C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c66c0(cx: &mut Cx, this: Ptr) -> Ptr {
    let mut v2 = Ptr::NULL;
    if ((cx.i(Ptr::glob(0x75B57C)) & 1) == 0) {
        { let p = Ptr::glob(0x75B57C); let t = cx.i(p).bitor_(1); cx.set_i(p, t); }
        let _ = ov_vec4_xyzw(cx, Ptr::glob(0x75B56C), 0, 0, 1065353216, 1065353216);
    }
    v2 = f_1bb200(cx, this, 0);
    { let q0_ = Ptr::glob(0x75B56C); let q1_ = cx.i(v2); let q2_ = cx.i(v2.add(4)); let q3_ = cx.i(v2.add(8)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
    return Ptr::glob(0x75B56C);
}

/// VehicleEntity__1C6A80 0x001C6A80
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c6a80(cx: &mut Cx, this: Ptr) -> i32 {
    let mut v2 = 0i32;
    if (cx.i(this.add(5096)) == 0) {
        return 0;
    }
    v2 = cx.i(this.add(5096));
    crate::ov_ignore!(cx);
    return crate::ov_zero_i!(cx, v2);
}

/// VehicleEntity__1C8860 0x001C8860
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c8860(cx: &mut Cx, mut this: Ptr) -> Ptr {
    crate::ov_ignore!(cx, this);
    { let p = this; let t = Ptr::glob(0x64A4C0); cx.set_ptr(p, t); }
    let _ = ov_identity_p(cx, this.add(12));
    let _ = f_23590(cx, this.add(28));
    let _ = ov_identity_p(cx, this.add(44));
    let _ = ov_identity_p(cx, this.add(60));
    let _ = f_1c8900(cx, this);
    return this;
}

/// VehicleEntity__1C8900 0x001C8900
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1c8900(cx: &mut Cx, this: Ptr) -> Ptr {
    let mut result = Ptr::NULL;
    crate::ov_set_minus_one!(cx, this.add(4));
    { let p = this.add(8); let t = 0; cx.set_i(p, t); }
    let _ = ov_vec4_set(cx, this.add(12), 0, 0, 0, 1065353216);
    let _ = ov_vec4_set(cx, this.add(28), 0, 0, 0, 1065353216);
    let _ = ov_vec4_set(cx, this.add(44), 0, 0, 0, 1065353216);
    result = ov_vec4_set(cx, this.add(60), 0, 0, 0, 1065353216);
    { let p = this.add(76); let t = 0; cx.set_i(p, t); }
    return result;
}

/// VehicleEntity__1C9070 0x001C9070
pub fn f_1c9070(cx: &mut Cx, this: Ptr, a2: i32, a3: i32, a4: i32) -> i64 {
    cx.push_frame(1788);
    let r = f_1c9070_body(cx, this, a2, a3, a4);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1c9070_body(cx: &mut Cx, this: Ptr, a2: i32, a3: i32, a4: i32) -> i64 {
    let mut v6 = 0.0f64;
    let mut v19 = 0i32;
    let mut v21 = 0i64;
    { let t = this; cx.set_lpp(445, t); }
    { let t = 2; cx.set_li(1776, t); }
    { let t = 1; cx.set_li(1772, t); }
    if (cx.f(cx.lpp(445).add(4096)) != 0.0f32) {
        { let p = cx.ptr_at(cx.lpp(445).add(3552)).add(40); let t = cx.i(cx.lpp(445).add(4096)); cx.set_i(p, t); }
        { let p = cx.ptr_at(cx.lpp(445).add(3552)).add(52); let t = cx.i(cx.lpp(445).add(4096)); cx.set_i(p, t); }
    }
    match cx.i(Ptr::glob(0x688A7C)) {
        1 => {
            { let q0_ = cx.lptr(1724); let q1_ = 0 /*str*/; f_1b59c0(cx, q0_, q1_) };
            { let q0_ = cx.li(1776); let q1_ = " SOFT_TIRES, driving wheels=%s\n\n"; crate::ov_ignore!(cx, q0_, q1_) };
        }
        2 => {
            { let q0_ = cx.lptr(1724); let q1_ = 0 /*str*/; f_1b59c0(cx, q0_, q1_) };
            { let q0_ = cx.li(1776); let q1_ = " MEDIUM_TIRES, driving wheels=%s\n\n"; crate::ov_ignore!(cx, q0_, q1_) };
        }
        3 => {
            { let q0_ = cx.lptr(1724); let q1_ = 0 /*str*/; f_1b59c0(cx, q0_, q1_) };
            { let q0_ = cx.li(1776); let q1_ = " HARD_TIRES, driving wheels=%s\n\n"; crate::ov_ignore!(cx, q0_, q1_) };
        }
        _ => {
        }
    }
    { let q0_ = cx.lptr(1724); f_1b51c0(cx, q0_) };
    { let t = cx.i(cx.lpp(445).add(3556)); cx.set_li(28, t); }
    { let t = cx.li(28); cx.set_lpp(8, Ptr::raw(t)); }
    if (cx.li(28) != 0) {
        { let t = { let q0_ = cx.lpp(8); let q1_ = 1; f_1da110(cx, q0_, q1_) }; cx.set_lpp(2, t); }
    } else {
        { let t = 0; cx.set_lpp(2, Ptr::raw(t)); }
    }
    { let p = cx.lpp(445).add(3556); let t = 0; cx.set_i(p, t); }
    if (cx.i(cx.lpp(445).add(3556)) == 0) {
        crate::ov_ignore!(cx, 0 /*str*/);
        { let t = ov_new(cx, 8); cx.set_lpp(6, t); }
        if (cx.lpp(6).bits() != 0) {
            { let t = { let q0_ = cx.lpp(6); let q1_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(64)); f_1b4730(cx, q0_, q1_) }; cx.set_lpp(1, t); }
        } else {
            { let t = 0; cx.set_lpp(1, Ptr::raw(t)); }
        }
        { let p = cx.lpp(445).add(3556); let t = cx.lpp(1); cx.set_ptr(p, t); }
        crate::ov_ignore!(cx);
    }
    { let t = 0; cx.set_li(252, t); }
    { let t = 0; cx.set_li(256, t); }
    'lp4: loop {
        if !(cx.li(256) < cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(64))) { break 'lp4; }
        'blp4: {
            { let t = cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(445).add(3552)).add(736)).add((12_i32).wrapping_mul(cx.li(256)))); cx.set_li(236, t); }
            { let t = cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(445).add(3552)).add(736)).add((12_i32).wrapping_mul(cx.li(256))).add(4)); cx.set_li(240, t); }
            { let t = cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(445).add(3552)).add(736)).add((12_i32).wrapping_mul(cx.li(256))).add(8)); cx.set_li(244, t); }
            { let t = cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(445).add(3552)).add(740)).add((12_i32).wrapping_mul(cx.li(256)))); cx.set_li(216, t); }
            { let t = cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(445).add(3552)).add(740)).add((12_i32).wrapping_mul(cx.li(256))).add(4)); cx.set_li(220, t); }
            { let t = cx.i(cx.ptr_at(cx.ptr_at(cx.lpp(445).add(3552)).add(740)).add((12_i32).wrapping_mul(cx.li(256))).add(8)); cx.set_li(224, t); }
            { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(445).add(3556))).add((132_i32).wrapping_mul(cx.li(252))).add(52); let q1_ = { let q0_ = (((cx.f(cx.lptr(236)) as f64) / 2.0f64) as f32); ov_coerce_int(cx, q0_) }; let q2_ = { let q0_ = (((cx.f(cx.lptr(240)) as f64) / 2.0f64) as f32); ov_coerce_int(cx, q0_) }; let q3_ = { let q0_ = (((cx.f(cx.lptr(244)) as f64) / 2.0f64) as f32); ov_coerce_int(cx, q0_) }; f_1bff80(cx, q0_, q1_, q2_, q3_) };
            { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(445).add(3556))).add((132_i32).wrapping_mul(cx.li(252))); f_1da870(cx, q0_) };
            { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(445).add(3556))).add((132_i32).wrapping_mul(cx.li(252))).add(116); let q1_ = cx.lptr(216); f_1bb190(cx, q0_, q1_) };
            if (cx.li(256) > 0) {
                { let p = cx.ptr_at(cx.ptr_at(cx.lpp(445).add(3556))).add((132_i32).wrapping_mul(cx.li(252))).add(64); let t = cx.i(cx.ptr_at(cx.lpp(445).add(3556))); cx.set_i(p, t); }
            }
            let _ = cx.preinc_i(cx.lptr(252));
        }
        let _ = cx.preinc_i(cx.lptr(256));
    }
    { let q0_ = cx.lptr(196); let q1_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(12)); let q2_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(16)); let q3_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(20)); let q4_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(24)); f_39d7b0(cx, q0_, q1_, q2_, q3_, q4_) };
    { let q0_ = cx.lptr(176); let q1_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(48)); let q2_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(40)); let q3_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(44)); f_39d8a0(cx, q0_, q1_, q2_, q3_) };
    { let q0_ = cx.lptr(156); let q1_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(60)); let q2_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(52)); let q3_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(56)); f_39d8a0(cx, q0_, q1_, q2_, q3_) };
    { let t = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(12)); cx.set_li(148, t); }
    if (cx.b(cx.lpp(445).add(4074)) != 0) {
        { let p = cx.lptr(148); let t = (((cx.f(cx.lptr(148)) as f64) / (cx.gc.player_to_trafic_mass_ratio as f64)) as f32); cx.set_f(p, t); }
    }
    crate::ov_ignore!(cx, 0 /*str*/);
    { let q0_ = cx.lpp(445).add(240).bits(); let q1_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(28)); let q2_ = Ptr::gc(0); let q3_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(28)); let q4_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(32)); let q5_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(36)); let q6_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(112)); let q7_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(116)); let q8_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(40)); let q9_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(52)); let q10_ = cx.lptr(196); let q11_ = cx.lptr(176); let q12_ = cx.lptr(156); let q13_ = cx.f(cx.lptr(148)); let q14_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(48)); let q15_ = cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(60)); f_35fa20(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_, q10_, q11_, q12_, q13_, q14_, q15_) };
    { let t = 3; cx.set_li(144, t); }
    'lp5: loop {
        if !(cx.li(144) >= 0) { break 'lp5; }
        'blp5: {
            { let q0_ = cx.lpp(445).add((16_i32).wrapping_mul(cx.li(144))).add(4944); let q1_ = { let q0_ = ((0.0f64 - (cx.f(cx.lpp(445).add((16_i32).wrapping_mul(cx.li(144))).add(2068)) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q2_ = cx.i(cx.lpp(445).add((16_i32).wrapping_mul(cx.li(144))).add(2064)); let q3_ = cx.i(cx.lpp(445).add((16_i32).wrapping_mul(cx.li(144))).add(2060)); let q4_ = 1065353216; ov_vec4_set(cx, q0_, q1_, q2_, q3_, q4_) };
        }
        let _ = cx.predec_i(cx.lptr(144));
    }
    crate::ov_ignore!(cx);
    { let q0_ = cx.lpp(445).add(240); let q1_ = cx.lptr(124); let q2_ = Ptr::NULL; f_364f80(cx, q0_, q1_, q2_) };
    { let q0_ = cx.lpp(445).add(1364); let q1_ = cx.lptr(124); let q2_ = cx.gc.suspension_stiffnees_multiplier; f_1caa90(cx, q0_, q1_, q2_) };
    { let q0_ = cx.lpp(445).add(240); let q1_ = cx.lptr(100); f_365080(cx, q0_, q1_) };
    { let q0_ = cx.lpp(445).add(1380); let q1_ = cx.lptr(100); let q2_ = cx.gc.suspension_damping_multiplier; f_1caa90(cx, q0_, q1_, q2_) };
    { let p = cx.lpp(445).add(1344); let t = Ptr::func(0x1BB320); cx.set_ptr(p, t); }
    { let q0_ = cx.lpp(445).add(1428); let q1_ = cx.gc.suspension_geometry; f_1caa70(cx, q0_, q1_) };
    { let p = cx.lpp(445).add(1436); let t = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(120)); cx.set_i(p, t); }
    { let q0_ = cx.lpp(445).add(1508); let q1_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(296)); f_1ba170(cx, q0_, q1_) };
    { let q0_ = cx.lpp(445).add(1524); let q1_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(300)); f_1ba170(cx, q0_, q1_) };
    { let q0_ = cx.lpp(445).add(1540); let q1_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(168)); f_1ba170(cx, q0_, q1_) };
    { let p = cx.lpp(445).add(1440); let t = cx.gc.wheel_dynamics_type; cx.set_i(p, t); }
    { let p = cx.lpp(445).add(264); let t = cx.gc.car_linear_damping; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(268); let t = cx.gc.car_angular_damping; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(1592); let t = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(180)); cx.set_i(p, t); }
    { let p = cx.lpp(445).add(1600); let t = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(448)); cx.set_i(p, t); }
    { let p = cx.lpp(445).add(1604); let t = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(472)); cx.set_i(p, t); }
    { let p = cx.lpp(445).add(1596); let t = cx.gc.physics_vs_play_relaxation_time_in_timesteps; cx.set_f(p, t); }
    { let q0_ = cx.lpp(445).add(1412); let q1_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(68)); f_1ba170(cx, q0_, q1_) };
    { let q0_ = cx.lpp(445).add(1396); let q1_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(72)); f_1ba170(cx, q0_, q1_) };
    { let p = cx.lpp(445).add(2488); let t = cx.gc.suspension_limits_through_constraint_handler; cx.set_i(p, t); }
    { let p = cx.lpp(445).add(1572); let t = cx.gc.relaxation_time_in_timesteps; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(432); let t = cx.gc.maximum_angular_velocity_ever; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(3128); let t = cx.gc.max_balancing_omega_change_start; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(3136); let t = cx.gc.max_balancing_omega_change_to_switch_to_2_wheels; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(3140); let t = cx.gc.min_difference_of_balancing_omega; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(3108); let t = cx.gc.balancing_time_1_wheel; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(3112); let t = cx.gc.balancing_time_2_wheels; cx.set_f(p, t); }
    { let q0_ = cx.lpp(445).add(3092); let q1_ = 0; f_1ba170(cx, q0_, q1_) };
    if (cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(4)) == 0.0f32) {
        { let q0_ = cx.lpp(445).add(240); let q1_ = 0.033333335f32; f_3682b0(cx, q0_, q1_) };
        v6 = (v6 * (cx.gc.dynamic_velocity_multiplier as f64));
        { let p = cx.lptr(92); let t = (v6 as f32); cx.set_f(p, t); }
        { let p = cx.lpp(445).add(1356); let t = cx.li(92); cx.set_i(p, t); }
    } else {
        { let p = cx.lpp(445).add(1356); let t = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(4)); cx.set_i(p, t); }
    }
    crate::ov_ignore!(cx, 0 /*str*/);
    { let q0_ = cx.lpp(445).add(240); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(76)); let q5_ = cx.gc.maximum_reverse_speed; let q6_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(88)); let q7_ = cx.gc.number_of_reverse_gears; let q8_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(52)); let q9_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(92)); let q10_ = (cx.gc.RPM_at_zero_gear_number.to_bits() as i32); let q11_ = (cx.gc.ideal_RPM_to_change_gear_up.to_bits() as i32); let q12_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(96)); let q13_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(280)); let q14_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(284)); let q15_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(288)); let q16_ = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(292)); let q17_ = (cx.gc.level.to_bits() as i32); f_35db90(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_, q10_, q11_, q12_, q13_, q14_, q15_, q16_, q17_) };
    { let p = cx.lptr(88); let t = (v6 as f32); cx.set_f(p, t); }
    crate::ov_ignore!(cx);
    { let p = cx.lpp(445).add(3264); let t = cx.gc.speed_epsilon; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(3272); let t = cx.gc.min_power_advantage_to_change_gear; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(3268); let t = cx.gc.relative_delta_RPM_to_change_gear; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(3120); let t = cx.gc.dont_use_RungeKutta_when_in_air; cx.set_i(p, t); }
    { let p = cx.lpp(445).add(3080); let t = 0; cx.set_i(p, t); }
    { let p = cx.lpp(445).add(3124); let t = 1; cx.set_i(p, t); }
    { let p = { let q0_ = cx.lpp(445).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }.add(64); let t = 0; cx.set_i(p, t); }
    { let p = cx.lpp(445).add(3796); let t = 0; cx.set_i(p, t); }
    { let p = cx.lpp(445).add(1616); let t = cx.gc.portion_of_VSA_to_take; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(1620); let t = cx.gc.max_VSAP; cx.set_f(p, t); }
    { let p = cx.lpp(445).add(1624); let t = cx.gc.steering_exaggeration_factor; cx.set_f(p, t); }
    if (cx.gc.level == 0.0f32) {
        crate::ov_ignore!(cx, 2, " To speed up startup, if you are not going to modify the gearbox you can set in Handling.cfg \"level\" to %g\n");
    }
    if (cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(100)) != 0.0f32) {
        { let t = 0; cx.set_li(84, t); }
        'lp6: loop {
            if !(cx.li(84) < cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(88))) { break 'lp6; }
            'blp6: {
                { let p = cx.lpp(445).add((4_i32).wrapping_mul(cx.li(84))).add(3284); let t = (((cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add(100)) as f64) / (cx.f(cx.ptr_at(cx.lpp(445).add(3552)).add((4_i32).wrapping_mul(cx.li(84))).add(240)) as f64)) as f32); cx.set_f(p, t); }
            }
            let _ = cx.preinc_i(cx.lptr(84));
        }
    }
    { let p = cx.lpp(445).add(3252); let t = cx.i(cx.ptr_at(cx.lpp(445).add(3552)).add(104)); cx.set_i(p, t); }
    { let p = cx.lpp(445).add(1556); let t = ((cx.b(cx.lpp(445).add(4072)) != 0) as i32); cx.set_i(p, t); }
    if (cx.gc.g_ftFarClipDistance != 0.0f32) {
        { let p = Ptr::glob(0x6850A0); let t = (cx.gc.g_ftFarClipDistance.to_bits() as i32); cx.set_i(p, t); }
    }
    if (cx.gc.g_yawRatePerDegree != 0.0f32) {
        { let p = Ptr::glob(0x68519C); let t = (cx.gc.g_yawRatePerDegree.to_bits() as i32); cx.set_i(p, t); }
    }
    if (cx.gc.g_yawRelaxAngle != 0.0f32) {
        { let p = Ptr::glob(0x6851A4); let t = cx.gc.g_yawRelaxAngle; cx.set_f(p, t); }
    }
    if (cx.gc.g_yawDirectionBlend != 0.0f32) {
        { let p = Ptr::glob(0x6851B4); let t = (cx.gc.g_yawDirectionBlend.to_bits() as i32); cx.set_i(p, t); }
    }
    if (cx.gc.g_WheelCamFOV != 0.0f32) {
        { let p = Ptr::glob(0x68768C); let t = (cx.gc.g_WheelCamFOV.to_bits() as i32); cx.set_i(p, t); }
    }
    if (cx.gc.g_WheelCamOffsetX != 0.0f32) {
        { let p = Ptr::glob(0x687690); let t = (cx.gc.g_WheelCamOffsetX.to_bits() as i32); cx.set_i(p, t); }
    }
    if (cx.gc.g_WheelCamOffsetZ != 0.0f32) {
        { let p = Ptr::glob(0x687698); let t = cx.gc.g_WheelCamOffsetZ; cx.set_f(p, t); }
    }
    if (cx.b(cx.lpp(445).add(4072)) != 0) {
        if (cx.i(Ptr::glob(0x74BA7C)) == 0) {
            { let t = ov_new(cx, 16); cx.set_lpp(5, t); }
            if (cx.lpp(5).bits() != 0) {
                { let t = { let q0_ = cx.lpp(5); ov_bump_map_ctor(cx, q0_) }; cx.set_lpp(0, t); }
            } else {
                { let t = 0; cx.set_lpp(0, Ptr::raw(t)); }
            }
            { let p = Ptr::glob(0x74BA7C); let t = cx.lpp(0).bits(); cx.set_i(p, t); }
        }
        { let q0_ = Ptr::glob(0x74BA7C); let q1_ = (cx.gc.bump_map_spacial_size.to_bits() as i32); let q2_ = 30; let q3_ = 1065353216; crate::ov_ignore!(cx, q0_, q1_, q2_, q3_) };
    }
    crate::ov_ignore!(cx);
    { let t = cx.li(1772); cx.set_li(16, t); }
    { let q0_ = cx.lptr(1304); crate::ov_ignore!(cx, q0_) };
    { let q0_ = cx.lptr(1724); f_1b5010(cx, q0_) };
    v21 = ({ let q0_ = v19; let q1_ = cx.li(16); crate::ov_zero_i!(cx, q0_, q1_) } as i64);
    return v21;
}

/// VehicleEntity__1CAA70 0x001CAA70
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1caa70(cx: &mut Cx, mut a1: Ptr, a2: i32) -> i32 {
    { let p = a1; let t = a2; cx.set_i(p, t); }
    { let p = a1.add(4); let t = a2; cx.set_i(p, t); }
    return a2;
}

/// VehicleEntity__1CAA90 0x001CAA90
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1caa90(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: f32) -> Ptr {
    { let p = a1; let t = (((cx.f(a2) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a2.add(4)) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a2.add(8)) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(12); let t = (((cx.f(a2.add(12)) as f64) * (a3 as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// VehicleEntity__1CABF0 0x001CABF0
pub fn f_1cabf0(cx: &mut Cx, this: Ptr, a2: i32, a3: Ptr, a4: i32, a5: Ptr, a6: Ptr, a7: i32) -> i64 {
    cx.push_frame(80);
    let r = f_1cabf0_body(cx, this, a2, a3, a4, a5, a6, a7);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1cabf0_body(cx: &mut Cx, this: Ptr, a2: i32, a3: Ptr, a4: i32, a5: Ptr, a6: Ptr, a7: i32) -> i64 {
    let mut v23 = Ptr::NULL;
    let mut v25 = 0i64;
    let mut flt_688A58 = 0i64;
    { let t = this; cx.set_lpp(19, t); }
    let _ = f_2c6d0(cx, this, a3);
    { let p = cx.lpp(19).add(5008); let t = 0; cx.set_i(p, t); }
    crate::ov_ignore!(cx, 0 /*str*/);
    { let q0_ = cx.lpp(19).add(3560); f_1b6cd0(cx, q0_) };
    crate::ov_ignore!(cx);
    v23 = Ptr::raw(a7);
    { let p = cx.lpp(19).add(4128); let t = a7; cx.set_i(p, t); }
    if ((a6.bits() != Ptr::NULL.bits()) && (cx.ptr_at(a6.add(4)).bits() != Ptr::NULL.bits())) {
        { let p = cx.lpp(19).add(4074); let t = 1; cx.set_b(p, t); }
    } else {
        v23 = cx.lpp(19);
        { let p = cx.lpp(19).add(4074); let t = 0; cx.set_b(p, t); }
    }
    { let q0_ = cx.lpp(19); let q1_ = v23.bits(); let q2_ = 0 /*str*/; let q3_ = cx.i(Ptr::glob(0x75B28C)); f_1c9070(cx, q0_, q1_, q2_, q3_) };
    { let q0_ = cx.lpp(19); let q1_ = a3; f_2c6d0(cx, q0_, q1_) };
    { let q0_ = cx.lpp(19).add(240); let q1_ = 0; let q2_ = 0; let q3_ = 0; f_36b530(cx, q0_, q1_, q2_, q3_) };
    { let q0_ = cx.lpp(19).add(240); let q1_ = 0; let q2_ = 0; let q3_ = 0; f_36b730(cx, q0_, q1_, q2_, q3_) };
    { let q0_ = cx.lpp(19).add(240); let q1_ = cx.i(a5.add(12)); let q2_ = { let q0_ = ((0.0f64 - (cx.f(a5) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q3_ = { let q0_ = ((0.0f64 - (cx.f(a5.add(4)) as f64)) as f32); ov_coerce_int(cx, q0_) }; let q4_ = { let q0_ = ((0.0f64 - (cx.f(a5.add(8)) as f64)) as f32); ov_coerce_int(cx, q0_) }; f_36a370(cx, q0_, q1_, q2_, q3_, q4_) };
    { let _ = { let q0_ = cx.lpp(19).add(240); vcall(cx, 32, q0_, &[]) }; };
    { let q0_ = cx.lpp(19); f_1cb420(cx, q0_) };
    if (a6.bits() != Ptr::NULL.bits()) {
        { let t = (((cx.ptr_at(a6).bits() != Ptr::NULL.bits()) && (cx.i(cx.ptr_at(a6).add(48)) != Ptr::NULL.bits())) as i32); cx.set_li(0, t); }
        { let p = cx.lpp(19).add(144); let t = cx.li(0); cx.set_b(p, t); }
    }
    if ((a6.bits() != Ptr::NULL.bits()) && (cx.ptr_at(a6).bits() != Ptr::NULL.bits())) {
        { let q0_ = cx.lpp(19); let q1_ = ((cx.i(cx.ptr_at(a6).add(396)) == Ptr::NULL.bits()) as i32); let q2_ = 0; f_1da8e0(cx, q0_, q1_, q2_) };
    }
    if (cx.b(cx.lpp(19).add(4072)) != 0) {
        { let q0_ = cx.lpp(19); let q1_ = (((cx.f(Ptr::glob(0x6F71C8)) as f64) + (cx.f(Ptr::glob(0x688A20)) as f64)) as f32); f_1da8b0(cx, q0_, q1_) };
    } else {
        { let q0_ = cx.lpp(19); let q1_ = 0.0f32; f_1da8b0(cx, q0_, q1_) };
    }
    v25 = (v25 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.lpp(19); f_1cb300(cx, q0_) }) as i32 as u32 as i64);
    { let p = Ptr::glob(0x688A58); let t = (v25 as f32); cx.set_f(p, t); }
    return flt_688A58;
}

/// VehicleEntity__1CB300 0x001CB300
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1cb300(cx: &mut Cx, this: Ptr) -> i32 {
    let mut i = 0i32;
    let mut v1 = 0i32;
    if (cx.ptr_at(this.add(3552)).bits() == Ptr::NULL.bits()) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            v1 = crate::ov_zero_i!(cx, "LC", ".\\src\\Physics\\Vehicle\\VehicleEntity.cpp", 5845);
            if (v1 == 3) {
            }
        } else {
        }
    }
    { let p = this.add(3576).add(116); let t = 0; cx.set_i(p, t); }
    { let p = this.add(0).add(96); let t = cx.i(this.add(0).add(100)); cx.set_i(p, t); }
    { let p = this.add(0).add(104); let t = cx.i(this.add(0).add(108)); cx.set_i(p, t); }
    i = 0;
    'lp2: loop {
        if !(i < 4) { break 'lp2; }
        'blp2: {
            { let p = this.add(3576).add(i.wrapping_add(1292)); let t = 0; cx.set_i(p, t); }
            { let p = this.add(3576).add(i.wrapping_add(1296)); let t = 0; cx.set_i(p, t); }
        }
        { i += 1; i };
    }
    { let p = this.add(0).add(48); let t = 0; cx.set_i(p, t); }
    if (cx.b(Ptr::glob(0x6F7158)) != 0) {
        let _ = f_1c56a0(cx, this.add(0));
    }
    { let p = this.add(0).add(132); let t = 0; cx.set_i(p, t); }
    { let p = this.add(0).add(136); let t = cx.i(Ptr::glob(0x688A18)); cx.set_i(p, t); }
    return this.add(0).bits();
}

/// VehicleEntity__1CB420 0x001CB420
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1cb420(cx: &mut Cx, this: Ptr) -> i32 {
    return { let q0_ = this.add(240); let q1_ = this.add(4788); vcall(cx, 24, q0_, &[Varg::from(q1_)]) }.i();
}

/// VehicleEntity__1CB470 0x001CB470
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1cb470(cx: &mut Cx, this: Ptr, a2: Ptr) -> i32 {
    let mut _1B6E80 = Ptr::NULL;
    _1B6E80 = ov_hist_entry(cx, this.add(3560), 0);
    { let p = this.add(3624); let t = 0; cx.set_i(p, t); }
    if (cx.b(a2.add(21)) != 0) {
        { let p = _1B6E80.add(36); let t = 0; cx.set_i(p, t); }
    } else {
        { let p = _1B6E80.add(36); let t = cx.i(a2.add(8)); cx.set_i(p, t); }
    }
    { let p = _1B6E80.add(164); let t = cx.i(a2.add(12)); cx.set_i(p, t); }
    { let p = _1B6E80.add(48); let t = cx.i(a2.add(16)); cx.set_i(p, t); }
    { let p = _1B6E80; let t = (cx.b(a2.add(20)) as f32); cx.set_f(p, t); }
    return _1B6E80.bits();
}

/// VehicleEntity__1CB530 0x001CB530
pub fn f_1cb530(cx: &mut Cx, this: Ptr, a2: i32, a3: i32) -> i64 {
    cx.push_frame(96);
    let r = f_1cb530_body(cx, this, a2, a3);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1cb530_body(cx: &mut Cx, this: Ptr, a2: i32, a3: i32) -> i64 {
    let mut v5 = 0.0f64;
    let mut v6 = 0.0f64;
    let mut v7 = 0.0f64;
    let mut v8 = Ptr::NULL;
    let mut v9 = Ptr::NULL;
    let mut v10 = 0i32;
    let mut v11 = 0.0f64;
    let mut v12 = 0i64;
    let mut v13 = 0i32;
    let mut v14 = 0i32;
    let mut v16 = 0i64;
    let mut v17 = 0i32;
    let mut Instance = Ptr::NULL;
    let mut RaceManager = Ptr::NULL;
    { let t = this; cx.set_lpp(23, t); }
    { let t = ov_hist_entry(cx, this.add(3560), 0); cx.set_lpp(22, t); }
    { let t = f_1cbb00(cx); let o = cx.li(84); cx.set_li(84, (o & !(255 << 24)) | ((t & 255) << 24)); }
    { let t = ov_traffic(cx).add((36_i32).wrapping_mul(a3)).add(24); cx.set_lpp(20, t); }
    if (cx.b(Ptr::glob(0x6F7158)) != 0) {
        Instance = ov_ai_world(cx);
        RaceManager = ov_race_manager(cx, Instance);
        if (cx.b({ let _ = RaceManager; let _ = 0; unmodelled_p("VsAIDriver__15DB0") }.add(253)) != 0) {
            { let t = ((cx.f(cx.lpp(20).add(12)) < 0.0f32) as i32); let o = cx.li(76); cx.set_li(76, (o & !(255 << 24)) | ((t & 255) << 24)); }
            { let q0_ = cx.lpp(20); crate::ov_ignore!(cx, q0_) };
            if (((cx.li(76) >> 24) & 255) != 0) {
                { let t = -0.25f32; cx.set_lf(32, t); }
            } else {
                { let t = 0.25f32; cx.set_lf(32, t); }
            }
            { let p = cx.lpp(20).add(12); let t = cx.lf(32); cx.set_f(p, t); }
            { let p = cx.lpp(20).add(16); let t = cx.i(p).bitor_(2); cx.set_i(p, t); }
        }
    }
    { let p = cx.lpp(23).add(3624); let t = cx.i(cx.lpp(20)); cx.set_i(p, t); }
    v5 = { let q0_ = cx.f(cx.lpp(23).add(3624)); let q1_ = cx.f(cx.lpp(20).add(4)); f_57f90(cx, q0_, q1_) };
    { let p = cx.lpp(22).add(36); let t = (v5 as f32); cx.set_f(p, t); }
    if (((cx.li(84) >> 24) & 255) == 0) {
        { let p = cx.lpp(22).add(36); let t = (((cx.f(cx.lpp(22).add(36)) as f64) * 0.8f64) as f32); cx.set_f(p, t); }
    }
    if (cx.b(cx.lpp(23).add(4900)) != 0) {
        { let p = cx.lpp(22).add(36); let t = (((cx.f(cx.lpp(22).add(36)) as f64) * (cx.f(cx.lpp(23).add(3248)) as f64)) as f32); cx.set_f(p, t); }
    }
    v6 = { let q0_ = ((0.0f64 - (cx.f(cx.lpp(23).add(3624)) as f64)) as f32); let q1_ = cx.f(cx.lpp(20).add(8)); f_57f90(cx, q0_, q1_) };
    { let p = cx.lpp(22).add(164); let t = (v6 as f32); cx.set_f(p, t); }
    { let p = cx.lptr(24); let t = (((cx.i(cx.lpp(20).add(16)) & 2) != 0) as i64); cx.set_q(p, t); }
    { let p = cx.lpp(22).add(48); let t = (cx.q(cx.lptr(24)) as f32); cx.set_f(p, t); }
    { let p = cx.lpp(22).add(68); let t = cx.i(cx.lpp(20).add(12)); cx.set_i(p, t); }
    if ((((cx.li(84) >> 24) & 255) == 0) && (cx.f(cx.lpp(22).add(68)) > 0.0f32)) {
        { let p = cx.lpp(22).add(68); let t = (((cx.f(cx.lpp(22).add(68)) as f64) * 0.8f64) as f32); cx.set_f(p, t); }
    }
    if (cx.f(cx.lpp(22).add(68)) <= 0.0f32) {
        if (cx.f(cx.lpp(22).add(68)) >= 0.0f32) {
            { let t = 0.0f32; cx.set_lf(16, t); }
        } else {
            { let t = -1.0f32; cx.set_lf(16, t); }
        }
        { let t = cx.lf(16); cx.set_lf(20, t); }
    } else {
        { let t = 1.0f32; cx.set_lf(20, t); }
    }
    { let p = cx.lpp(23).add(3780); let t = cx.lf(20); cx.set_f(p, t); }
    v7 = vsmath::fabs(cx.f(cx.lpp(22).add(68)));
    { let p = cx.lpp(22).add(72); let t = (v7 as f32); cx.set_f(p, t); }
    { let p = cx.lptr(8); let t = (((cx.i(cx.lpp(20).add(16)) & 64) != 0) as i64); cx.set_q(p, t); }
    { let p = cx.lpp(22); let t = (cx.q(cx.lptr(8)) as f32); cx.set_f(p, t); }
    { let p = cx.lptr(0); let t = (((cx.i(cx.lpp(20).add(16)) & 128) != 0) as i64); cx.set_q(p, t); }
    { let p = cx.lpp(22).add(4); let t = (cx.q(cx.lptr(0)) as f32); cx.set_f(p, t); }
    v17 = cx.i(cx.lpp(23).add(5072));
    v8 = ov_ai_world(cx);
    v9 = ov_race_manager(cx, v8);
    { let t = crate::ov_null!(cx, v9, v17); cx.set_lpp(18, t); }
    if ((cx.lpp(18).bits() != 0) && ({ let q0_ = cx.lpp(18); ov_ai_vehicle_flag(cx, q0_) } != 0)) {
        { let p = cx.lpp(22).add(4); let t = 1065353216; cx.set_i(p, t); }
    }
    { let t = { let q0_ = cx.lpp(23); f_1b74a0(cx, q0_) }; cx.set_li(68, t); }
    { let t = cx.f(cx.lpp(23).add(3216)); cx.set_lf(64, t); }
    if (cx.li(68) < 90) {
        { let q0_ = cx.lpp(23); let q1_ = cx.lptr(56).add(3); let q2_ = cx.lptr(44).add(3); f_24550(cx, q0_, q1_, q2_) };
        if (((((cx.li(56) >> 24) & 255) != 0) && ((cx.i(cx.lpp(20).add(16)) & 4) != 0)) && (cx.b(cx.lpp(23).add(4928)) == 0)) {
            v10 = { let q0_ = cx.lpp(23).add(240); f_24530(cx, q0_) };
            if (cx.li(64) < v10) {
                if (cx.b(cx.lpp(23).add(4900)) != 0) {
                    if (cx.f(cx.lpp(23).add(3532)) < 0.0f32) {
                        v11 = ({ let q0_ = cx.lpp(23).add(240); f_35d090(cx, q0_) } as f64);
                        { let p = cx.lpp(23).add(3532); let t = (v11 as f32); cx.set_f(p, t); }
                    }
                } else {
                    { let q0_ = cx.lpp(23); f_1d1210(cx, q0_) };
                    { let p = cx.lpp(23).add(48); let t = 1065353216; cx.set_i(p, t); }
                }
            }
        }
        if (((((cx.li(44) >> 24) & 255) != 0) && ((cx.i(cx.lpp(20).add(16)) & 8) != 0)) && (cx.li(64) > 1)) {
            { let q0_ = cx.lpp(23); f_1d1240(cx, q0_) };
        }
        if (((cx.li(56) >> 24) & 255) != 0) {
            { let t = { let q0_ = cx.lpp(23); f_1b74a0(cx, q0_) }; cx.set_li(36, t); }
            if (cx.li(36) < 90) {
                if ((cx.li(36) > cx.i(cx.lpp(23).add(52))) && (cx.li(36) > 1)) {
                    { let q0_ = cx.lpp(23).bits(); f_1d1480(cx, q0_) };
                }
                { let p = cx.lpp(23).add(52); let t = cx.lf(36); cx.set_f(p, t); }
            }
        }
    }
    { let p = cx.lpp(23).add(4928); let t = (((cx.i(cx.lpp(20).add(16)) & 4) != 0) as i32); cx.set_b(p, t); }
    if ((cx.i(Ptr::glob(0x75B9E8)) & 1) == 0) {
        { let p = Ptr::glob(0x75B9E8); let t = cx.i(p).bitor_(1); cx.set_i(p, t); }
        let _ = f_1da910(cx, Ptr::glob(0x75B700));
        crate::ov_ignore!(cx, Ptr::func(0x4354E0));
    }
    if ((cx.i(Ptr::glob(0x688B7C)) != cx.i(Ptr::glob(0x75B28C))) && (cx.i(Ptr::glob(0x688B7C)) != -1)) {
        { let p = Ptr::glob(0x75B288); let t = 1; cx.set_b(p, t); }
    }
    { let p = Ptr::glob(0x688B7C); let t = cx.i(Ptr::glob(0x75B28C)); cx.set_i(p, t); }
    v12 = (v12 & 0xFFFF_FFFFi64) | ((((cx.b(Ptr::glob(0x75B288)) as u8 as i32)) as u32 as i64) << 32);
    if (cx.b(Ptr::glob(0x75B288)) != 0) {
        cx.memcpy(Ptr::raw(cx.i(Ptr::glob(0x75B700))), cx.ptr_at(cx.lpp(23).add(3552)), { let q0_ = cx.i(Ptr::glob(0x75B700)); crate::ov_zero_i!(cx, q0_) });
        v13 = { let _ = Ptr::gc(0); let _ = 0 /*str*/; let _ = cx.i(Ptr::glob(0x75B28C)); unmodelled_i("CConfig__199DC0") };
        v14 = v13;
        if (v13 == 0) {
            v14 = (((({ let _ = Ptr::gc(0); let _ = 0 /*str*/; let _ = 0; unmodelled_i("CConfig__199DC0") }) as i32 as u32 as i64) >> 32i64) as i32);
        }
        v12 = { let q0_ = cx.lpp(23); let q1_ = v14; let q2_ = 0 /*str*/; let q3_ = cx.i(Ptr::glob(0x75B28C)); f_1c9070(cx, q0_, q1_, q2_, q3_) };
        if ((v12 as i32) == 0) {
            v12 = { let q0_ = cx.lpp(23); let q1_ = ((v12 >> 32) & 4294967295) as i32; let q2_ = 0 /*str*/; let q3_ = 0; f_1c9070(cx, q0_, q1_, q2_, q3_) };
            if ((v12 as i32) == 0) {
                v12 = (v12 & 0xFFFF_FFFFi64) | (((cx.lpp(23).bits()) as u32 as i64) << 32);
                cx.memcpy(cx.ptr_at(cx.lpp(23).add(3552)), Ptr::raw(cx.i(Ptr::glob(0x75B700))), 744);
            }
        }
        { let p = Ptr::glob(0x75B288); let t = 0; cx.set_b(p, t); }
    }
    v12 = (v12 & !0xFFFF_FFFFi64) | (((cx.i(Ptr::glob(0x75B9E8)) & 2)) as i32 as u32 as i64);
    if ((cx.i(Ptr::glob(0x75B9E8)) & 2) == 0) {
        { let p = Ptr::glob(0x75B9E8); let t = cx.i(p).bitor_(2); cx.set_i(p, t); }
        let _ = f_1b6c30(cx, Ptr::glob(0x75B5E0));
        v12 = (v12 & !0xFFFF_FFFFi64) | (({ crate::ov_ignore!(cx, Ptr::func(0x4354C0)); 0 }) as i32 as u32 as i64);
    }
    if (cx.i(Ptr::glob(0x75B5DC)) > 0) {
        v12 = (v12 & 0xFFFF_FFFFi64) | (((cx.predec_i(Ptr::glob(0x75B5DC))) as u32 as i64) << 32);
    }
    if (cx.i(Ptr::glob(0x75B5D8)) > 0) {
        v12 = (v12 & !0xFFFF_FFFFi64) | ((cx.predec_i(Ptr::glob(0x75B5D8))) as i32 as u32 as i64);
    }
    if (cx.i(Ptr::glob(0x75B5D4)) > 0) {
        let _ = cx.predec_i(Ptr::glob(0x75B5D4));
    }
    v16 = v12;
    return v16;
}

/// VehicleEntity__1CBB00 0x001CBB00
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1cbb00(cx: &mut Cx) -> i32 {
    let mut i = 0i32;
    let mut v1 = 0i32;
    let mut v3 = 0i32;
    v3 = 0;
    i = 0;
    'lp2: loop {
        if !(cx.b(Ptr::glob(0x685458).add(1 * (i))) != 0) { break 'lp2; }
        'blp2: {
            v3 = v3.wrapping_add(cx.b(Ptr::glob(0x685458).add(1 * (i))));
        }
        { i += 1; i };
    }
    v1 = ((cx.i(Ptr::glob(0x685558)) == v3) as i32);
    return v1;
}

/// VehicleEntity__1CBBB0 0x001CBBB0
pub fn f_1cbbb0(cx: &mut Cx, this: Ptr, a2: i32, a3: i32) -> i32 {
    cx.push_frame(52);
    let r = f_1cbbb0_body(cx, this, a2, a3);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1cbbb0_body(cx: &mut Cx, this: Ptr, a2: i32, a3: i32) -> i32 {
    let mut v3 = 0i32;
    let mut v5 = 0.0f64;
    let mut v6 = 0.0f64;
    let mut v7 = 0.0f64;
    let mut v8 = 0i32;
    let mut result = Ptr::NULL;
    { let t = this; cx.set_lpp(12, t); }
    { let t = ov_hist_entry(cx, this.add(3560), 0); cx.set_lpp(11, t); }
    v3 = { crate::ov_ignore!(cx); 0 };
    v8 = { crate::ov_ignore!(cx, v3); 0 };
    crate::ov_ignore!(cx);
    result = Ptr::raw({ crate::ov_ignore!(cx, a3, v8); 0 });
    { let t = result; cx.set_lpp(10, t); }
    if (result.bits() != 0) {
        { let t = f_1cbb00(cx); let o = cx.li(36); cx.set_li(36, (o & !(255 << 24)) | ((t & 255) << 24)); }
        { let p = cx.lpp(12).add(3624); let t = ((((cx.f(cx.lpp(10).add(12)) + cx.f(cx.lpp(10).add(16))) as f64) * 0.5f64) as f32); cx.set_f(p, t); }
        v5 = { let q0_ = cx.f(cx.lpp(12).add(3624)); let q1_ = cx.f(cx.lpp(10).add(12)); f_57f90(cx, q0_, q1_) };
        { let p = cx.lpp(11).add(36); let t = (v5 as f32); cx.set_f(p, t); }
        if (((cx.li(36) >> 24) & 255) == 0) {
            { let p = cx.lpp(11).add(36); let t = (((cx.f(cx.lpp(11).add(36)) as f64) * 0.8f64) as f32); cx.set_f(p, t); }
        }
        v6 = { let q0_ = ((0.0f64 - (cx.f(cx.lpp(12).add(3624)) as f64)) as f32); let q1_ = cx.f(cx.lpp(10).add(16)); f_57f90(cx, q0_, q1_) };
        { let p = cx.lpp(11).add(164); let t = (v6 as f32); cx.set_f(p, t); }
        { let p = cx.lptr(20); let t = (cx.i(cx.lpp(10).add(24)) as i64); cx.set_q(p, t); }
        { let p = cx.lpp(11).add(48); let t = (cx.q(cx.lptr(20)) as f32); cx.set_f(p, t); }
        { let p = cx.lpp(11).add(68); let t = cx.i(cx.lpp(10).add(20)); cx.set_i(p, t); }
        if ((((cx.li(36) >> 24) & 255) == 0) && (cx.f(cx.lpp(11).add(68)) > 0.0f32)) {
            { let p = cx.lpp(11).add(68); let t = (((cx.f(cx.lpp(11).add(68)) as f64) * 0.8f64) as f32); cx.set_f(p, t); }
        }
        if (cx.f(cx.lpp(11).add(68)) <= 0.0f32) {
            if (cx.f(cx.lpp(11).add(68)) >= 0.0f32) {
                { let t = 0; cx.set_li(12, t); }
            } else {
                { let t = -1082130432; cx.set_li(12, t); }
            }
            { let t = cx.li(12); cx.set_li(16, t); }
        } else {
            { let t = 1065353216; cx.set_li(16, t); }
        }
        { let p = cx.lpp(12).add(3780); let t = cx.li(16); cx.set_i(p, t); }
        v7 = vsmath::fabs(cx.f(cx.lpp(11).add(68)));
        { let p = cx.lpp(11).add(72); let t = (v7 as f32); cx.set_f(p, t); }
        { let p = cx.lptr(4); let t = (cx.i(cx.lpp(10).add(40)) as i64); cx.set_q(p, t); }
        { let p = cx.lpp(11); let t = (cx.q(cx.lptr(4)) as f32); cx.set_f(p, t); }
        { let t = cx.lpp(10); cx.set_lpp(7, t); }
        result = cx.lpp(10);
        { let t = cx.lpp(10); cx.set_lpp(8, t); }
        if (cx.lpp(10).bits() != 0) {
            result = Ptr::raw({ let q0_ = cx.lpp(8); let q1_ = 1; vcall(cx, 0, q0_, &[Varg::from(q1_)]) }.i());
        }
    }
    return result.bits();
}

/// VehicleEntity__1CBDE0 0x001CBDE0
pub fn f_1cbde0(cx: &mut Cx, this: Ptr) -> i64 {
    cx.push_frame(172);
    let r = f_1cbde0_body(cx, this);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1cbde0_body(cx: &mut Cx, this: Ptr) -> i64 {
    let mut v1 = 0i64;
    let mut v2 = Ptr::NULL;
    let mut v3 = Ptr::NULL;
    let mut v4 = 0.0f64;
    let mut v6 = Ptr::NULL;
    let mut v7 = 0.0f64;
    let mut v8 = 0i32;
    let mut v9 = 0.0f64;
    let mut v10 = 0.0f32;
    let mut v11 = 0i32;
    let mut v12 = 0i32;
    let mut v13 = 0i32;
    let mut v14 = 0i32;
    let mut v15 = 0i32;
    let mut v17 = 0i64;
    let mut v18 = 0.0f32;
    let mut SS_min_distance_to_steer = 0.0f64;
    { let t = this; cx.set_lpp(42, t); }
    { let t = ov_hist_entry(cx, this.add(3560), 0); cx.set_lpp(41, t); }
    v1 = (v1 & !0xFFFF_FFFFi64) | ((cx.lpp(42)).bits() as u32 as i64);
    { let p = cx.lpp(42).add(3616); let t = 0; cx.set_i(p, t); }
    if (cx.gc.SS_enabeld != 0) {
        if (cx.i(cx.lpp(42).add(84)) != -1) {
            { let p = cx.lpp(42).add(3632); let t = (((cx.f(cx.lpp(42).add(3632)) as f64) + (cx.f(cx.lpp(42).add(68)) as f64)) as f32); cx.set_f(p, t); }
            v2 = ov_physics_manager(cx);
            { let t = ov_vehicle_array(cx, v2); cx.set_lpp(40, t); }
            { let t = { let p = { let q0_ = cx.lpp(40); let q1_ = cx.i(cx.lpp(42).add(84)); ov_vehicle_slot(cx, q0_, q1_) }; cx.ptr_at(p) }; cx.set_lpp(39, t); }
            v1 = (v1 & !0xFFFF_FFFFi64) | ((cx.lpp(42)).bits() as u32 as i64);
            if (cx.f(cx.lpp(42).add(3632)) > cx.gc.SS_necessary_accumulated_effect_for_steering_bias) {
                v1 = (v1 & !0xFFFF_FFFFi64) | ((Ptr::gc(0).bits()) as i32 as u32 as i64);
                if ((cx.f(cx.lpp(42).add(68)) > cx.gc.SS_necessary_effect_for_steering_bias) && (cx.i(cx.lpp(42).add(3612)) == 0)) {
                    { let _ = { let q0_ = cx.lpp(39).add(240); let q1_ = cx.lptr(140); vcall(cx, 28, q0_, &[Varg::from(q1_)]) }; };
                    { let _ = { let q0_ = cx.lpp(42).add(240); let q1_ = cx.lptr(120); vcall(cx, 28, q0_, &[Varg::from(q1_)]) }; };
                    { let q0_ = cx.lptr(100); let q1_ = cx.lptr(140); let q2_ = cx.lptr(120); f_1c00c0(cx, q0_, q1_, q2_) };
                    v3 = { let q0_ = cx.lpp(42).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                    v4 = { let q0_ = cx.lptr(100); let q1_ = v3; f_1c0300(cx, q0_, q1_) };
                    SS_min_distance_to_steer = (cx.gc.SS_min_distance_to_steer as f64);
                    if (v4 > SS_min_distance_to_steer) {
                        { let q0_ = cx.lptr(80); let q1_ = cx.lptr(120); let q2_ = cx.lptr(100); let q3_ = cx.gc.SS_distance_portion_to_steer_to; f_1c0210(cx, q0_, q1_, q2_, q3_) };
                        v18 = cx.f(cx.ptr_at(cx.lpp(42).add(3552)));
                        v6 = { let q0_ = cx.lpp(39).add(240); ov_car_velocity(cx, q0_) };
                        { let q0_ = cx.lptr(80); let q1_ = v6; let q2_ = v18; f_1c01b0(cx, q0_, q1_, q2_) };
                        if (cx.gc.SS_steer_method == 0.0f32) {
                            v7 = ({ let q0_ = cx.lpp(42).add(240); let q1_ = cx.lptr(80); f_3717d0(cx, q0_, q1_) } as f64);
                            { let p = cx.lpp(42).add(3616); let t = (v7 as f32); cx.set_f(p, t); }
                        } else if (cx.gc.SS_steer_method == 1.0f32) {
                            v1 = { let q0_ = cx.lpp(42).add(240); let q1_ = Ptr::NULL.bits(); let q2_ = cx.lptr(80); f_371c90(cx, q0_, q1_, q2_) };
                            { let p = cx.lpp(42).add(3616); let t = (SS_min_distance_to_steer as f32); cx.set_f(p, t); }
                        } else {
                            v9 = (({ let q0_ = cx.lpp(42).add(240); let q1_ = cx.lptr(80); f_3717d0(cx, q0_, q1_) } as f64) * (1.0f64 - (cx.gc.SS_steer_method as f64)));
                            { let t = (v9 as f32); cx.set_lf(0, t); }
                            { let q0_ = cx.lpp(42).add(240); let q1_ = Ptr::NULL.bits(); let q2_ = cx.lptr(80); f_371c90(cx, q0_, q1_, q2_) };
                            v1 = (v1 & !0xFFFF_FFFFi64) | ((Ptr::gc(0).bits()) as i32 as u32 as i64);
                            { let p = cx.lpp(42).add(3616); let t = (((v9 * (cx.gc.SS_steer_method as f64)) + (cx.lf(0) as f64)) as f32); cx.set_f(p, t); }
                        }
                        v10 = cx.f(cx.lpp(41).add(36));
                        v15 = ov_isnan(cx, v10);
                        v11 = ((v10 < 0.0f32) as i32);
                        v12 = ((v10 == 0.0f32) as i32);
                        v13 = 0;
                        v14 = 0;
                        v1 = (v1 & !(0xFF00i64 as i64)) | ((((v8) as i64) & 0xFF) << 8) as i64;
                        if (v10 != 0.0f32) {
                            { let t = { let q0_ = cx.lptr(100); f_1c0330(cx, q0_) }; cx.set_lf(72, t); }
                            { let q0_ = cx.lpp(39).add(240); let q1_ = cx.lptr(56); f_1b42b0(cx, q0_, q1_) };
                            { let q0_ = cx.lpp(42).add(240); let q1_ = cx.lptr(36); f_1b42b0(cx, q0_, q1_) };
                            { let q0_ = cx.lptr(16); let q1_ = cx.lptr(56); let q2_ = cx.lptr(36); f_1c00c0(cx, q0_, q1_, q2_) };
                            { let t = ((((cx.lf(72) - cx.gc.SS_good_distance) as f64) / (cx.gc.SS_catch_up_time as f64)) as f32); cx.set_lf(8, t); }
                            { let t = cx.lf(8); cx.set_lf(4, t); }
                            if (cx.lf(8) <= cx.gc.SS_max_velocity_increment) {
                                if ((0.0f32 - cx.gc.SS_max_velocity_increment) > cx.lf(8)) {
                                    { let t = ((0.0f64 - (cx.gc.SS_max_velocity_increment as f64)) as f32); cx.set_lf(8, t); }
                                }
                            } else {
                                { let t = cx.gc.SS_max_velocity_increment; cx.set_lf(8, t); }
                            }
                            v1 = (v1 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.lpp(42).add(272); let q1_ = cx.lptr(100); let q2_ = (((((cx.lf(4) / cx.lf(72)) * cx.f(cx.ptr_at(cx.lpp(42).add(3552)))) as f64) / (cx.gc.SS_catch_up_time as f64)) as f32); f_1c01b0(cx, q0_, q1_, q2_) }.bits()) as i32 as u32 as i64);
                        }
                    }
                }
            }
        }
        v1 = (v1 & 0xFFFF_FFFFi64) | (((cx.lpp(42).bits()) as u32 as i64) << 32);
        if (cx.f(cx.lpp(42).add(3632)) > 0.0f32) {
            v1 = (v1 & 0xFFFF_FFFFi64) | (((cx.lpp(42).bits()) as u32 as i64) << 32);
            { let p = cx.lpp(42).add(3632); let t = (((cx.f(cx.lpp(42).add(3632)) as f64) - (cx.gc.SS_passive_accumulated_effect_depletion as f64)) as f32); cx.set_f(p, t); }
            v1 = (v1 & !0xFFFF_FFFFi64) | ((cx.lpp(42)).bits() as u32 as i64);
            if (cx.f(cx.lpp(42).add(3632)) < 0.0f32) {
                { let p = cx.lpp(42).add(3632); let t = 0; cx.set_i(p, t); }
            }
        }
    }
    v17 = v1;
    return v17;
}

/// VehicleEntity__1CC2E0 0x001CC2E0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1cc2e0(cx: &mut Cx, this: Ptr) -> i32 {
    let mut result = 0i32;
    let mut _1B6E80 = Ptr::NULL;
    _1B6E80 = ov_hist_entry(cx, this.add(3560), 0);
    { let p = this.add(3720); let t = 1065353216; cx.set_i(p, t); }
    { let p = cx.ptr_at(this.add(3552)).add(148); let t = cx.i(cx.ptr_at(this.add(3552)).add(272)); cx.set_i(p, t); }
    { let p = Ptr::glob(0x688B70); let t = 1065353216; cx.set_i(p, t); }
    result = cx.i(Ptr::gc(0).add(172));
    { let p = cx.ptr_at(this.add(3552)).add(8); let t = result; cx.set_i(p, t); }
    { let p = _1B6E80.add(144); let t = 0; cx.set_i(p, t); }
    return result;
}

/// VehicleEntity__1CC380 0x001CC380
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1cc380(cx: &mut Cx, this: Ptr) -> i32 {
    let mut result = 0i32;
    { let p = this.add(3828); let t = 0; cx.set_i(p, t); }
    { let p = this.add(89); let t = 0; cx.set_b(p, t); }
    { let p = this.add(3832); let t = 1; cx.set_i(p, t); }
    { let p = this.add(3580); let t = 0; cx.set_i(p, t); }
    { let p = this.add(3588); let t = 0; cx.set_i(p, t); }
    result = cx.b(this.add(4072));
    if (cx.b(this.add(4072)) != 0) {
        { let p = Ptr::glob(0x688B70); let t = 1065353216; cx.set_i(p, t); }
    }
    return result;
}

/// VehicleEntity__1CC3F0 0x001CC3F0
pub fn f_1cc3f0(cx: &mut Cx, a1: Ptr) -> i64 {
    cx.push_frame(104);
    let r = f_1cc3f0_body(cx, a1);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1cc3f0_body(cx: &mut Cx, a1: Ptr) -> i64 {
    let mut v3 = 0.0f32;
    let mut v4 = 0i64;
    let mut v6 = 0.0f64;
    let mut v7 = 0.0f64;
    let mut v9 = Ptr::NULL;
    let mut v10 = Ptr::NULL;
    let mut v13 = 0i64;
    let mut Length = 0.0f64;
    let mut Instance = Ptr::NULL;
    let mut RaceManager = Ptr::NULL;
    let mut PointAtDistance = Ptr::NULL;
    v13 = (v13 & 0xFFFF_FFFFi64) | (((a1.bits()) as u32 as i64) << 32);
    { let t = a1; cx.set_lpp(22, t); }
    { let p = a1.add(3616); let t = 0; cx.set_i(p, t); }
    v13 = (v13 & 0xFFFF_FFFFi64) | (((cx.i(cx.lpp(22).add(5072))) as u32 as i64) << 32);
    Instance = ov_ai_world(cx);
    RaceManager = ov_race_manager(cx, Instance);
    { let t = ov_race_vehicle(cx, RaceManager, ((v13 >> 32) & 4294967295) as i32); cx.set_lpp(21, t); }
    if (cx.lpp(21).bits() == 0) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            v3 = f32::from_bits(crate::ov_zero_i!(cx, "pAIVec", ".\\src\\Physics\\Vehicle\\VehicleEntity.cpp", 6366) as u32);
            { let t = v3; cx.set_lf(80, t); }
            if ((v3.to_bits() as i32) == 3) {
            }
            { let t = 257; let o = cx.li(8); cx.set_li(8, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
        } else {
            { let t = 1; let o = cx.li(8); cx.set_li(8, (o & !(255 << 16)) | ((t & 255) << 16)); }
        }
    }
    v4 = (v4 & !(0xFFi64 as i64)) | (((({ let q0_ = cx.lpp(21); ov_ai_vehicle_flag(cx, q0_) }) as i64) & 0xFF) << 0) as i64;
    v4 = (v4 & 0xFFFF_FFFFi64) | (((((v4 as i32) as u8 as i32)) as u32 as i64) << 32);
    if (((v4 as i32) as u8 as i32) != 0) {
        v4 = (v4 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.lpp(21).bits(); ov_null_i(cx, q0_) }.bits()) as i32 as u32 as i64);
        { let t = (v4 as i32); cx.set_li(76, t); }
        if ((v4 as i32) != 0) {
            v4 = (v4 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.lpp(19); f_1da960(cx, q0_) }) as i32 as u32 as i64);
            { let t = (v4 as i32); cx.set_li(72, t); }
            if ((v4 as i32) != 0) {
                { let t = { let _ = cx.lpp(19); unmodelled_f("VsVehicleFollowCamera__69A20") }; cx.set_lf(4, t); }
                Length = ({ let _ = cx.lpp(18); unmodelled_f("VsAIRoadLine__GetLength") } as f64);
                { let t = (((Length * (cx.lf(4) as f64)) + (cx.f(Ptr::glob(0x688B74)) as f64)) as f32); cx.set_lf(0, t); }
                v6 = { let q0_ = cx.lpp(19); f_1da980(cx, q0_) };
                { let t = (((v6 * (cx.f(Ptr::glob(0x688B78)) as f64)) + (cx.lf(0) as f64)) as f32); cx.set_lf(56, t); }
                v7 = ({ let _ = cx.lpp(18); unmodelled_f("VsAIRoadLine__GetLength") } as f64);
                if ((cx.lf(56) as f64) > v7) {
                    v7 = ({ let _ = cx.lpp(18); unmodelled_f("VsAIRoadLine__GetLength") } as f64);
                    { let t = (v7 as f32); cx.set_lf(56, t); }
                }
                PointAtDistance = { let _ = cx.lpp(18); let _ = cx.lf(56); let _ = 0; let _ = cx.lptr(64); unmodelled_p("VsAIRoadLine__GetPointAtDistance") };
                { let q0_ = cx.lptr(36); let q1_ = PointAtDistance; ov_vec4_copy(cx, q0_, q1_) };
                v13 = (v13 & 0xFFFF_FFFFi64) | (((1065353216) as u32 as i64) << 32);
                v9 = { let q0_ = cx.lpp(22); f_1c66c0(cx, q0_) };
                { let q0_ = cx.lptr(92); let q1_ = v9; ov_vec4_copy(cx, q0_, q1_) };
                v10 = { let q0_ = cx.lpp(22); let q1_ = cx.lptr(12); ov_ve_get_position(cx, q0_, q1_) };
                v4 = ({ let _ = cx.lptr(36); let _ = v10; let _ = cx.li(92); let _ = cx.li(96); let _ = cx.li(100); let _ = v13; let _ = f32::from_bits((v13 >> 32) as u32); unmodelled_f("VehicleEntity__1CC5D0") } as i64);
                { let p = cx.lpp(22).add(3616); let t = (v7 as f32); cx.set_f(p, t); }
            }
        }
    }
    v13 = v4;
    return v13;
}

/// VehicleEntity__1CC790 0x001CC790
pub fn f_1cc790(cx: &mut Cx, a1: i32, a2: i32, a3: Ptr, mut a4: Ptr, mut a5: Ptr, a6: Ptr, a7: Ptr, a8: Ptr, a9: Ptr, a10: f32) -> i64 {
    cx.push_frame(56);
    let r = f_1cc790_body(cx, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1cc790_body(cx: &mut Cx, a1: i32, a2: i32, a3: Ptr, mut a4: Ptr, mut a5: Ptr, a6: Ptr, a7: Ptr, a8: Ptr, a9: Ptr, a10: f32) -> i64 {
    let mut v10 = 0.0f64;
    let mut v11 = 0.0f32;
    let mut v12 = 0.0f64;
    let mut v13 = 0i64;
    let mut v15 = 0i64;
    { let t = a1; cx.set_li(52, t); }
    { let q0_ = cx.lptr(32); let q1_ = a6; let q2_ = a7; f_1c5080(cx, q0_, q1_, q2_) };
    { let q0_ = a3; let q1_ = cx.lptr(32); f_1cca40(cx, q0_, q1_) };
    { let q0_ = cx.lptr(44); let q1_ = -1.0f32; let q2_ = 1.0f32; f_1bfe80(cx, q0_, q1_, q2_) };
    v10 = vsmath::acos(cx.lf(44));
    { let p = a5; let t = ((v10 + v10) as f32); cx.set_f(p, t); }
    if (cx.f(a5) < 0.0f32) {
        if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
            v11 = f32::from_bits(crate::ov_zero_i!(cx, "rot_angle>=0.0f", ".\\src\\Physics\\Vehicle\\VehicleEntity.cpp", 6425) as u32);
            { let t = v11; cx.set_lf(24, t); }
            if ((v11.to_bits() as i32) == 3) {
            }
            { let t = 257; let o = cx.li(0); cx.set_li(0, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
        } else {
            { let t = 1; let o = cx.li(0); cx.set_li(0, (o & !(255 << 16)) | ((t & 255) << 16)); }
        }
    }
    if (cx.f(a5) > 3.1415927f32) {
        { let p = a5; let t = ((6.2831855f64 - (cx.f(a5) as f64)) as f32); cx.set_f(p, t); }
        let _ = f_1cc9f0(cx, a3);
    }
    { let t = 0.0f32; cx.set_lf(20, t); }
    { let t = (f_1c0300(cx, a3, a9) as f32); cx.set_lf(16, t); }
    if ((0.0f32 - cx.f(Ptr::gc(0).add(1020))) > cx.lf(16)) {
        { let t = (((f_1c0330(cx, a8) as f64) * (a10 as f64)) as f32); cx.set_lf(12, t); }
        { let t = ((6.2831855f64 - (cx.f(a5) as f64)) as f32); cx.set_lf(8, t); }
        if (cx.lf(12) > (cx.lf(8) / ((0.0f32 - cx.f(Ptr::gc(0).add(1024))) * cx.lf(16)))) {
            let _ = f_1cc9f0(cx, a3);
            { let t = ((0.0f64 - (cx.lf(16) as f64)) as f32); cx.set_lf(16, t); }
            { let p = a5; let t = cx.lf(8); cx.set_f(p, t); }
            { let t = (((cx.lf(8) as f64) / (cx.lf(12) as f64)) as f32); cx.set_lf(20, t); }
        }
    }
    { let p = a4; let t = (((cx.f(a5) as f64) / (a10 as f64)) as f32); cx.set_f(p, t); }
    { let t = ((((cx.lf(16) * cx.f(Ptr::gc(0).add(1012))) as f64) + (cx.f(Ptr::gc(0).add(1008)) as f64)) as f32); cx.set_lf(4, t); }
    v12 = vsmath::fabs(cx.f(a4));
    v13 = (v13 & !(0xFFi64 as i64)) | (((((((cx.lf(4) as f64) >= v12) as i32)) as i64) & 0xFF) << 0) as i64;
    v15 = v13;
    return v15;
}

/// VehicleEntity__1CC9F0 0x001CC9F0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1cc9f0(cx: &mut Cx, mut a1: Ptr) -> Ptr {
    { let p = a1; let t = ((0.0f64 - (cx.f(a1) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((0.0f64 - (cx.f(a1.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((0.0f64 - (cx.f(a1.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// VehicleEntity__1CCA40 0x001CCA40
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1cca40(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
    let mut v3 = 0.0f32;
    v3 = f_1c0330(cx, a2);
    if (v3 != 0.0f32) {
        return f_1c0160(cx, a1, a2, ((1.0f64 / (v3 as f64)) as f32));
    }
    { let p = a1; let t = 1065353216; cx.set_i(p, t); }
    { let p = a1.add(4); let t = 0; cx.set_i(p, t); }
    { let p = a1.add(8); let t = 0; cx.set_i(p, t); }
    return a1;
}

/// VehicleEntity__1CCAD0 0x001CCAD0
pub fn f_1ccad0(cx: &mut Cx, this: Ptr) -> i64 {
    cx.push_frame(752);
    let r = f_1ccad0_body(cx, this);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1ccad0_body(cx: &mut Cx, this: Ptr) -> i64 {
    let mut x = 0.0f32;
    let mut v2 = Ptr::NULL;
    let mut v3 = Ptr::NULL;
    let mut v4 = Ptr::NULL;
    let mut v5 = Ptr::NULL;
    let mut v6 = 0i32;
    let mut v7 = 0i64;
    let mut v9 = Ptr::NULL;
    let mut xa = 0.0f32;
    let mut xb = 0.0f32;
    let mut v10 = Ptr::NULL;
    let mut v11 = 0i64;
    { let t = this; cx.set_lpp(187, t); }
    { let t = ov_hist_entry(cx, this.add(3560), 0); cx.set_lpp(186, t); }
    if ((((((cx.gc.PL_enabled == 0) || (cx.b(cx.lpp(187).add(3688)) != 0)) || ({ let q0_ = cx.lpp(187).add(240); ov_car_i2876(cx, q0_) } == 0)) || ({ let p = { let q0_ = cx.lpp(187).add(240); ov_car_velocity(cx, q0_) }.add(4); cx.f(p) } <= cx.gc.PL_min_speed_up)) || { let _ = { let _ = { let _ = { let _ = { v2 = { let q0_ = cx.lpp(187).add(240); ov_car_velocity(cx, q0_) }; v2 }; { { let t = (((cx.f(v2) as f64) * ({ let p = { let q0_ = cx.lpp(187).add(240); ov_car_velocity(cx, q0_) }; cx.f(p) } as f64)) as f32); cx.set_lf(0, t); } } }; { v3 = { let q0_ = cx.lpp(187).add(240); ov_car_velocity(cx, q0_) }; v3 } }; { v4 = { let q0_ = cx.lpp(187).add(240); ov_car_velocity(cx, q0_) }; v4 } }; ((cx.lf(0) + (cx.f(v3.add(8)) * cx.f(v4.add(8)))) <= (cx.gc.PL_min_horisontal_speed * cx.gc.PL_min_horisontal_speed)) }) || (cx.i(cx.lpp(187).add(4716)) != 0)) {
        if ((cx.i(cx.lpp(187).add(4716)) != 0) && ((cx.i(cx.lpp(186).add(92)) == 0) || ((cx.i(cx.lpp(187).add(4756)) == 1) && ((cx.i(cx.lpp(187).add(4856)) <= 1) || (cx.f(cx.lpp(187).add(4596)) > (cx.f(cx.lpp(187).add(4592)) * cx.gc.PL_max_total_flight_time_multiplier)))))) {
            if (cx.i(cx.lpp(187).add(4756)) == 1) {
                if (cx.i(cx.lpp(187).add(4784)) != 0) {
                    { let q0_ = cx.lptr(488); let q1_ = cx.lpp(187).add(4688); let q2_ = cx.lpp(187).add(4628); let q3_ = cx.f(cx.lpp(187).add(4596)); f_1ced60(cx, q0_, q1_, q2_, q3_) };
                    { let q0_ = cx.lptr(444); let q1_ = cx.lptr(488); f_1ce8c0(cx, q0_, q1_) };
                } else {
                    { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lptr(444); f_36a090(cx, q0_, q1_) };
                    { let q0_ = cx.lptr(488); let q1_ = cx.lptr(444); f_39d0d0(cx, q0_, q1_) };
                    { let q0_ = Ptr::NULL.bits(); let q1_ = cx.lptr(488); let q2_ = cx.lpp(187).add(4628); let q3_ = cx.f(cx.lpp(187).add(4596)); f_1ceab0(cx, q0_, q1_, q2_, q3_) };
                }
                { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lptr(488); f_36a1e0(cx, q0_, q1_) };
                { let _ = { let q0_ = cx.lpp(187).add(240); vcall(cx, 32, q0_, &[]) }; };
                { let p = cx.lpp(187).add(3576); let t = 0; cx.set_i(p, t); }
                { let q0_ = cx.lpp(187).add(4544); let q1_ = cx.lpp(187).add(4740); let q2_ = cx.lptr(488); f_1c5080(cx, q0_, q1_, q2_) };
            }
            { let p = cx.lpp(187).add(4524); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(187).add(4532); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(187).add(4592); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(187).add(4716); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(187).add(4596); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(187).add(4736); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(187).add(4756); let t = 0; cx.set_i(p, t); }
        }
    } else {
        { let t = 0.0f32; cx.set_lf(680, t); }
        if ((cx.gc.draw_PL_trajectory != 0) && (cx.i(cx.lpp(187).add(4520)) == 0)) {
            crate::ov_ignore!(cx, 0 /*str*/);
            { let t = { let q0_ = (12_i32).wrapping_mul(cx.gc.PL_max_NO_points); ov_new(cx, q0_) }; cx.set_lpp(2, t); }
            { let p = cx.lpp(187).add(4520); let t = cx.lpp(2); cx.set_ptr(p, t); }
            crate::ov_ignore!(cx);
        }
        if (cx.i(cx.lpp(187).add(4528)) == 0) {
            crate::ov_ignore!(cx, 0 /*str*/);
            { let t = { let q0_ = (12_i32).wrapping_mul(cx.gc.PL_max_NO_points); ov_new(cx, q0_) }; cx.set_lpp(1, t); }
            { let p = cx.lpp(187).add(4528); let t = cx.lpp(1); cx.set_ptr(p, t); }
            crate::ov_ignore!(cx);
        }
        { let t = ((({ let q0_ = cx.lpp(187).add(272); f_1c0330(cx, q0_) } as f64) / (cx.gc.full_gravity_in_air_speed as f64)) as f32); cx.set_lf(672, t); }
        if (cx.lf(672) > 1.0f32) {
            { let t = 1.0f32; cx.set_lf(672, t); }
        }
        { let q0_ = cx.lptr(656); let q1_ = Ptr::gc(12); let q2_ = Ptr::gc(0); f_1c00c0(cx, q0_, q1_, q2_) };
        { let q0_ = cx.lpp(187).add(248); let q1_ = Ptr::gc(0); let q2_ = cx.lptr(656); let q3_ = cx.lf(672); f_1c0210(cx, q0_, q1_, q2_, q3_) };
        { let p = cx.lpp(187).add(3576); let t = 1; cx.set_i(p, t); }
        { let _ = { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lptr(708); vcall(cx, 28, q0_, &[Varg::from(q1_)]) }; };
        { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lptr(688); f_1b42b0(cx, q0_, q1_) };
        { let q0_ = cx.lptr(728); let q1_ = cx.lptr(708); f_1bb190(cx, q0_, q1_) };
        { let t = 0.0f32; cx.set_lf(676, t); }
        'lp3: loop {
            if !((cx.li(676) < cx.gc.PL_max_NO_points) && ((cx.lf(732) + cx.gc.PL_max_delta_height) <= cx.lf(712))) { break 'lp3; }
            'blp3: {
                if (cx.gc.draw_PL_trajectory != 0) {
                    { let q0_ = cx.ptr_at(cx.lpp(187).add(4520)).add((12_i32).wrapping_mul(cx.i(cx.lpp(187).add(4524)))); let q1_ = cx.lptr(708); f_1bb190(cx, q0_, q1_) };
                    let _ = cx.preinc_i(cx.lpp(187).add(4524));
                }
                if ((cx.li(676) % cx.gc.PL_ray_node_timestep_multiplier) == 0) {
                    { let q0_ = cx.ptr_at(cx.lpp(187).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(187).add(4532)))); let q1_ = cx.lptr(708); f_1bb190(cx, q0_, q1_) };
                    if (cx.i(cx.lpp(187).add(4532)) != 0) {
                        v10 = cx.ptr_at(cx.lpp(187).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(187).add(4532))));
                        v9 = cx.ptr_at(cx.lpp(187).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(187).add(4532)).wrapping_sub(1)));
                        v5 = ov_physics_manager(cx);
                        if ((({ let q0_ = v5; let q1_ = v6; let q2_ = cx.lptr(620); let q3_ = cx.lptr(632); let q4_ = v9; let q5_ = v10; let q6_ = -1; let q7_ = 17; f_19d3e0(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_) } as i32) as u8 as i32) != 0) {
                            { let q0_ = cx.lpp(187).add(4640); let q1_ = cx.lptr(632); f_1bb190(cx, q0_, q1_) };
                            { let p = cx.lpp(187).add(4592); let t = (((((((cx.lf(620) - 1.0f32) * (cx.gc.PL_ray_node_timestep_multiplier as f32)) * cx.gc.PL_timestep) + cx.lf(680)) as f64) + (cx.gc.PL_flight_time_addition as f64)) as f32); cx.set_f(p, t); }
                            { let q0_ = cx.lptr(600); let q1_ = cx.ptr_at(cx.lpp(187).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(187).add(4532)))); let q2_ = cx.ptr_at(cx.lpp(187).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(187).add(4532)).wrapping_sub(1))); f_1c00c0(cx, q0_, q1_, q2_) };
                            { let q0_ = cx.lpp(187).add(4600); let q1_ = cx.ptr_at(cx.lpp(187).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(187).add(4532)).wrapping_sub(1))); let q2_ = cx.lptr(600); let q3_ = cx.lf(620); f_1c0210(cx, q0_, q1_, q2_, q3_) };
                            { let q0_ = cx.ptr_at(cx.lpp(187).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(187).add(4532)))); let q1_ = cx.lpp(187).add(4600); f_1bb190(cx, q0_, q1_) };
                            { let q0_ = cx.lpp(187).add(4652); let q1_ = cx.lptr(688); f_1bb190(cx, q0_, q1_) };
                            let _ = cx.preinc_i(cx.lpp(187).add(4532));
                            { let p = cx.lpp(187).add(4596); let t = 0; cx.set_i(p, t); }
                            if ((cx.f(cx.lpp(187).add(4592)) <= cx.gc.PL_min_flight_time) || (cx.f(cx.lpp(187).add(4644)) <= cx.gc.PL_min_normal_up_to_accept)) {
                                { let p = cx.lpp(187).add(4716); let t = -1; cx.set_i(p, t); }
                                { let p = cx.lpp(187).add(3576); let t = 0; cx.set_i(p, t); }
                            } else {
                                { let p = cx.lpp(187).add(4716); let t = 1; cx.set_i(p, t); }
                            }
                            break 'lp3;
                        }
                    }
                    let _ = cx.preinc_i(cx.lpp(187).add(4532));
                }
                { let t = (((cx.lf(680) as f64) + (cx.gc.PL_timestep as f64)) as f32); cx.set_lf(680, t); }
                { let q0_ = cx.lptr(688); let q1_ = cx.lpp(187).add(248); let q2_ = cx.gc.PL_timestep; f_1c01b0(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(708); let q1_ = cx.lptr(688); let q2_ = cx.gc.PL_timestep; f_1c01b0(cx, q0_, q1_, q2_) };
            }
            let _ = cx.preinc_i(cx.lptr(676));
        }
        if (((((((cx.f(cx.lpp(187).add(4592)) > cx.f(Ptr::glob(0x6889F4))) && (cx.b(cx.lpp(187).add(4072)) != 0)) && (cx.i(cx.lpp(187).add(4876)) == 0)) && (cx.b(Ptr::glob(0x6F7158)) == 0)) && (({ let q0_ = cx.b(Ptr::glob(0x749128)); let q1_ = 1; ov_jump_cam_can_see(cx, q0_, q1_) } as u8 as i32) != 0)) && (cx.b(Ptr::glob(0x75B278)) != 0)) && ((cx.gc.PL_jump_cam_probability as f64) > (f_1cf070(cx) as f64))) {
            { let p = Ptr::glob(0x749110); let t = 2; cx.set_i(p, t); }
            { let q0_ = cx.b(Ptr::glob(0x749128)); ov_jump_cam_start(cx, q0_) };
            { let p = Ptr::glob(0x749118); let t = (((cx.f(cx.lpp(187).add(4592)) as f64) - (cx.f(Ptr::glob(0x6889FC)) as f64)) as f32); cx.set_f(p, t); }
            { let p = Ptr::glob(0x74911C); let t = cx.i(Ptr::glob(0x749118)); cx.set_i(p, t); }
            { let p = cx.lpp(187).add(4876); let t = 96; cx.set_i(p, t); }
        }
        if (cx.i(cx.lpp(187).add(4716)) != 0) {
            { let p = cx.lpp(187).add(264); let t = cx.gc.car_linear_damping_when_in_air; cx.set_f(p, t); }
            { let p = cx.lpp(187).add(268); let t = cx.gc.car_angular_damping_when_in_air; cx.set_f(p, t); }
            if (cx.gc.PL_eigenalizer_factor != 0.0f32) {
                { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lptr(580); f_1da680(cx, q0_, q1_) };
                { let t = { let q0_ = cx.lptr(580); f_1c0330(cx, q0_) }; cx.set_lf(572, t); }
                { let q0_ = cx.lptr(556); let q1_ = cx.lptr(580); f_1ce870(cx, q0_, q1_) };
                if (cx.lf(556) <= cx.lf(560)) {
                    if (cx.lf(560) <= cx.lf(564)) {
                        { let t = 2; cx.set_li(548, t); }
                    } else {
                        { let t = 1; cx.set_li(548, t); }
                    }
                } else if (cx.lf(556) <= cx.lf(564)) {
                    { let t = 2; cx.set_li(548, t); }
                } else {
                    { let t = 0.0f32; cx.set_lf(548, t); }
                }
                { let p = cx.lptr(0).add(4 * (cx.li(548).wrapping_add(145))); let t = (((cx.f(cx.lptr(0).add(4 * (cx.li(548).wrapping_add(145)))) as f64) * (cx.gc.PL_eigenalizer_factor as f64)) as f32); cx.set_f(p, t); }
                { let q0_ = cx.lptr(580); f_1c0380(cx, q0_) };
                { let q0_ = cx.lptr(580); let q1_ = cx.lf(572); f_1c0110(cx, q0_, q1_) };
                { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lptr(580); f_36b6f0(cx, q0_, q1_) };
            }
            if (cx.gc.PL_omega_scale_mode != 0.0f32) {
                { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lptr(532); f_36b6b0(cx, q0_, q1_) };
                { let q0_ = cx.lptr(512); let q1_ = cx.lptr(532); f_1ce870(cx, q0_, q1_) };
                if ((cx.gc.PL_max_omega_to_scale / cx.gc.PL_omega_scale_x) <= cx.lf(512)) {
                    if ((cx.gc.PL_max_omega_to_scale > cx.lf(512)) && (cx.lf(512) != 0.0f32)) {
                        { let t = ((((cx.gc.PL_max_omega_to_scale / cx.lf(512)) as f64) * (cx.lf(532) as f64)) as f32); cx.set_lf(532, t); }
                    }
                } else {
                    { let t = (((cx.lf(532) as f64) * (cx.gc.PL_omega_scale_x as f64)) as f32); cx.set_lf(532, t); }
                }
                if ((cx.gc.PL_max_omega_to_scale / cx.gc.PL_omega_scale_y) <= cx.lf(516)) {
                    if ((cx.gc.PL_max_omega_to_scale > cx.lf(516)) && (cx.lf(516) != 0.0f32)) {
                        { let t = ((((cx.gc.PL_max_omega_to_scale / cx.lf(516)) as f64) * (cx.lf(536) as f64)) as f32); cx.set_lf(536, t); }
                    }
                } else {
                    { let t = (((cx.lf(536) as f64) * (cx.gc.PL_omega_scale_y as f64)) as f32); cx.set_lf(536, t); }
                }
                if ((cx.gc.PL_max_omega_to_scale / cx.gc.PL_omega_scale_z) <= cx.lf(520)) {
                    if ((cx.gc.PL_max_omega_to_scale > cx.lf(520)) && (cx.lf(520) != 0.0f32)) {
                        { let t = ((((cx.gc.PL_max_omega_to_scale / cx.lf(520)) as f64) * (cx.lf(540) as f64)) as f32); cx.set_lf(540, t); }
                    }
                } else {
                    { let t = (((cx.lf(540) as f64) * (cx.gc.PL_omega_scale_z as f64)) as f32); cx.set_lf(540, t); }
                }
                { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lptr(532); f_36b780(cx, q0_, q1_) };
            }
        } else {
            { let p = cx.lpp(187).add(4716); let t = -1; cx.set_i(p, t); }
            { let p = cx.lpp(187).add(3576); let t = 0; cx.set_i(p, t); }
        }
    }
    v7 = (v7 & !0xFFFF_FFFFi64) | ((cx.lpp(187)).bits() as u32 as i64);
    if ((cx.i(cx.lpp(187).add(4716)) == 1) && (cx.i(cx.lpp(187).add(4736)) == 0)) {
        { let p = cx.lpp(187).add(4592); let t = (((cx.f(cx.lpp(187).add(4592)) as f64) + (cx.f(cx.ptr_at(cx.lpp(187).add(3552))) as f64)) as f32); cx.set_f(p, t); }
        { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lpp(187).add(4720); f_36a140(cx, q0_, q1_) };
        { let q0_ = cx.lpp(187).add(4688); let q1_ = cx.lpp(187).add(4720); f_1bb0c0(cx, q0_, q1_) };
        { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lpp(187).add(4664); f_1da680(cx, q0_, q1_) };
        { let q0_ = cx.lpp(187).add(4676); let q1_ = cx.lpp(187).add(4664); f_1bb190(cx, q0_, q1_) };
        { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lptr(356); f_369d30(cx, q0_, q1_) };
        { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lptr(312); f_369da0(cx, q0_, q1_) };
        { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lptr(268); f_369ec0(cx, q0_, q1_) };
        { let q0_ = cx.lpp(187).add(240); let q1_ = cx.lptr(376); f_36a090(cx, q0_, q1_) };
        if (cx.f(cx.lpp(187).add(268)) <= 0.0f32) {
            { let t = 0.0f32; cx.set_lf(260, t); }
        } else {
            { let t = (((vsmath::exp(((((0.0f32 - cx.gc.PL_timestep) as f64) / (cx.f(cx.lpp(187).add(268)) as f64)) as f32)) - 1.0f64) / (cx.gc.PL_timestep as f64)) as f32); cx.set_lf(260, t); }
        }
        { let t = 0.0f32; cx.set_lf(436, t); }
        'lp4: loop {
            if !(cx.f(cx.lpp(187).add(4592)) > cx.lf(436)) { break 'lp4; }
            'blp4: {
                { let t = (((((cx.lf(316) * cx.f(cx.lpp(187).add(4664))) + (cx.lf(328) * cx.f(cx.lpp(187).add(4668)))) as f64) + ((cx.lf(332) * cx.f(cx.lpp(187).add(4672))) as f64)) as f32); cx.set_lf(256, t); }
                { let t = (((((cx.lf(320) * cx.f(cx.lpp(187).add(4664))) + (cx.lf(332) * cx.f(cx.lpp(187).add(4668)))) as f64) + ((cx.lf(344) * cx.f(cx.lpp(187).add(4672))) as f64)) as f32); cx.set_lf(252, t); }
                { let t = (((((cx.lf(312) * cx.f(cx.lpp(187).add(4664))) + (cx.lf(316) * cx.f(cx.lpp(187).add(4668)))) as f64) + ((cx.lf(320) * cx.f(cx.lpp(187).add(4672))) as f64)) as f32); cx.set_lf(248, t); }
                { let t = ((((cx.f(cx.lpp(187).add(4672)) * cx.lf(256)) as f64) - ((cx.f(cx.lpp(187).add(4668)) * cx.lf(252)) as f64)) as f32); cx.set_lf(244, t); }
                { let t = ((((cx.f(cx.lpp(187).add(4664)) * cx.lf(252)) as f64) - ((cx.f(cx.lpp(187).add(4672)) * cx.lf(248)) as f64)) as f32); cx.set_lf(240, t); }
                { let t = ((((cx.f(cx.lpp(187).add(4668)) * cx.lf(248)) as f64) - ((cx.f(cx.lpp(187).add(4664)) * cx.lf(256)) as f64)) as f32); cx.set_lf(236, t); }
                { let t = (((((cx.lf(268) * cx.lf(244)) + (cx.lf(272) * cx.lf(240))) as f64) + ((cx.lf(276) * cx.lf(236)) as f64)) as f32); cx.set_lf(420, t); }
                { let t = (((((cx.lf(272) * cx.lf(244)) + (cx.lf(284) * cx.lf(240))) as f64) + ((cx.lf(288) * cx.lf(236)) as f64)) as f32); cx.set_lf(424, t); }
                { let t = (((((cx.lf(276) * cx.lf(244)) + (cx.lf(288) * cx.lf(240))) as f64) + ((cx.lf(300) * cx.lf(236)) as f64)) as f32); cx.set_lf(428, t); }
                { let q0_ = cx.lptr(420); let q1_ = cx.lpp(187).add(4664); let q2_ = cx.lf(260); f_1c01b0(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lpp(187).add(4664); let q1_ = cx.lptr(420); let q2_ = cx.gc.PL_timestep; f_1c01b0(cx, q0_, q1_, q2_) };
                { let q0_ = Ptr::NULL.bits(); let q1_ = cx.lpp(187).add(4720); let q2_ = cx.lpp(187).add(4664); let q3_ = cx.gc.PL_timestep; f_1ceab0(cx, q0_, q1_, q2_, q3_) };
                { let q0_ = cx.lptr(376); let q1_ = cx.lpp(187).add(4720); f_1ce8c0(cx, q0_, q1_) };
                { let t = ((((((cx.lf(376) * cx.lf(376)) * cx.lf(356)) + ((cx.lf(388) * cx.lf(388)) * cx.lf(360))) as f64) + (((cx.lf(400) * cx.lf(400)) * cx.lf(364)) as f64)) as f32); cx.set_lf(312, t); }
                { let t = ((((((cx.lf(380) * cx.lf(380)) * cx.lf(356)) + ((cx.lf(392) * cx.lf(392)) * cx.lf(360))) as f64) + (((cx.lf(404) * cx.lf(404)) * cx.lf(364)) as f64)) as f32); cx.set_lf(328, t); }
                { let t = ((((((cx.lf(384) * cx.lf(384)) * cx.lf(356)) + ((cx.lf(396) * cx.lf(396)) * cx.lf(360))) as f64) + (((cx.lf(408) * cx.lf(408)) * cx.lf(364)) as f64)) as f32); cx.set_lf(344, t); }
                { let t = ((0.0f64 - ((((cx.lf(376) * cx.lf(380)) * (cx.lf(364) - cx.lf(356))) + ((cx.lf(388) * cx.lf(392)) * (cx.lf(364) - cx.lf(360)))) as f64)) as f32); cx.set_lf(316, t); }
                { let t = cx.lf(316); cx.set_lf(324, t); }
                { let t = ((0.0f64 - ((((cx.lf(380) * cx.lf(384)) * (cx.lf(364) - cx.lf(356))) + ((cx.lf(392) * cx.lf(396)) * (cx.lf(364) - cx.lf(360)))) as f64)) as f32); cx.set_lf(332, t); }
                { let t = cx.lf(332); cx.set_lf(340, t); }
                { let t = ((0.0f64 - ((((cx.lf(376) * cx.lf(384)) * (cx.lf(364) - cx.lf(356))) + ((cx.lf(388) * cx.lf(396)) * (cx.lf(364) - cx.lf(360)))) as f64)) as f32); cx.set_lf(320, t); }
                { let t = cx.lf(320); cx.set_lf(336, t); }
                { let q0_ = cx.lptr(268); let q1_ = cx.lptr(312); f_1cee80(cx, q0_, q1_) };
            }
            { let t = (((cx.lf(436) as f64) + (cx.gc.PL_timestep as f64)) as f32); cx.set_lf(436, t); }
        }
        { let q0_ = cx.lpp(187).add(4704); let q1_ = cx.lptr(312); let q2_ = cx.lpp(187).add(4676); f_1ce810(cx, q0_, q1_, q2_) };
        { let p = cx.lpp(187).add(4736); let t = 1; cx.set_i(p, t); }
    }
    v7 = (v7 & 0xFFFF_FFFFi64) | (((cx.lpp(187).bits()) as u32 as i64) << 32);
    if (cx.i(cx.lpp(187).add(4736)) != 0) {
        v7 = (v7 & !0xFFFF_FFFFi64) | ((cx.lpp(187)).bits() as u32 as i64);
        if (cx.i(cx.lpp(187).add(4756)) == 0) {
            if ({ let q0_ = cx.lptr(176); let q1_ = cx.lpp(187).add(4676); f_1c0410(cx, q0_, q1_) } == 0) {
                { let q0_ = cx.lptr(176); f_1ba1b0(cx, q0_) };
            }
            { let p = cx.lpp(187).add(4784); let t = 0; cx.set_i(p, t); }
            { let q0_ = cx.lptr(208); let q1_ = cx.lpp(187).add(4640); f_1bb190(cx, q0_, q1_) };
            if (((cx.f(cx.lpp(187).add(4652)) * cx.f(cx.lpp(187).add(4652))) + (cx.f(cx.lpp(187).add(4660)) * cx.f(cx.lpp(187).add(4660)))) <= (cx.gc.PL_max_landing_velocity_to_ignore_car_landing_velocity * cx.gc.PL_max_landing_velocity_to_ignore_car_landing_velocity)) {
                { let t = ((((((cx.f(cx.lpp(187).add(4720)) * cx.f(cx.lpp(187).add(4720))) + (cx.f(cx.lpp(187).add(4732)) * cx.f(cx.lpp(187).add(4732)))) * 2.0f32) as f64) - 1.0f64) as f32); cx.set_lf(196, t); }
                { let t = (((((cx.f(cx.lpp(187).add(4720)) * cx.f(cx.lpp(187).add(4724))) + (cx.f(cx.lpp(187).add(4728)) * cx.f(cx.lpp(187).add(4732)))) as f64) * 2.0f64) as f32); cx.set_lf(200, t); }
                { let t = (((((cx.f(cx.lpp(187).add(4728)) * cx.f(cx.lpp(187).add(4720))) - (cx.f(cx.lpp(187).add(4724)) * cx.f(cx.lpp(187).add(4732)))) as f64) * 2.0f64) as f32); cx.set_lf(204, t); }
                { let q0_ = cx.lptr(220); let q1_ = cx.lptr(196); let q2_ = cx.lptr(208); f_1c0270(cx, q0_, q1_, q2_) };
            } else {
                { let q0_ = cx.lptr(220); let q1_ = cx.lpp(187).add(4652); let q2_ = cx.lptr(208); f_1c0270(cx, q0_, q1_, q2_) };
            }
            { let q0_ = cx.lptr(220); f_1c0380(cx, q0_) };
            { let q0_ = cx.lptr(196); let q1_ = cx.lptr(208); let q2_ = cx.lptr(220); f_1c0270(cx, q0_, q1_, q2_) };
            if (cx.gc.PL_roll_landing_disturbance != 0.0f32) {
                { let t = (((f_1cf090(cx) as f64) * (cx.gc.PL_roll_landing_disturbance as f64)) as f32); cx.set_lf(168, t); }
                { let t = (vsmath::cos(cx.lf(168)) as f32); cx.set_lf(164, t); }
                { let t = (vsmath::sin(cx.lf(168)) as f32); cx.set_lf(160, t); }
                { let q0_ = cx.lptr(144); let q1_ = cx.lptr(208); let q2_ = cx.lf(164); let q3_ = cx.lptr(196); let q4_ = cx.lf(160); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
                { let q0_ = cx.lptr(196); let q1_ = cx.lptr(208); let q2_ = ((0.0f64 - (cx.lf(160) as f64)) as f32); let q3_ = cx.lptr(196); let q4_ = cx.lf(164); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
                { let q0_ = cx.lptr(208); let q1_ = cx.lptr(144); f_1bb190(cx, q0_, q1_) };
            }
            if (cx.gc.PL_pitch_landing_disturbance != 0.0f32) {
                { let t = (((f_1cf090(cx) as f64) * (cx.gc.PL_pitch_landing_disturbance as f64)) as f32); cx.set_lf(136, t); }
                { let t = (vsmath::cos(cx.lf(136)) as f32); cx.set_lf(132, t); }
                { let t = (vsmath::sin(cx.lf(136)) as f32); cx.set_lf(128, t); }
                { let q0_ = cx.lptr(112); let q1_ = cx.lptr(208); let q2_ = cx.lf(132); let q3_ = cx.lptr(220); let q4_ = cx.lf(128); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
                { let q0_ = cx.lptr(220); let q1_ = cx.lptr(208); let q2_ = ((0.0f64 - (cx.lf(128) as f64)) as f32); let q3_ = cx.lptr(220); let q4_ = cx.lf(132); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
                { let q0_ = cx.lptr(208); let q1_ = cx.lptr(112); f_1bb190(cx, q0_, q1_) };
            }
            { let q0_ = cx.lpp(187).add(4740); let q1_ = cx.lptr(196); f_39d0d0(cx, q0_, q1_) };
            if ((({ let q0_ = cx.lpp(187).bits(); let q1_ = cx.lpp(187).add(4628).bits(); let q2_ = cx.lpp(187).add(4628); let q3_ = cx.lptr(100); let q4_ = cx.lptr(88); let q5_ = cx.lpp(187).add(4740); let q6_ = cx.lpp(187).add(4720); let q7_ = cx.lpp(187).add(4676); let q8_ = cx.lptr(176); let q9_ = cx.f(cx.lpp(187).add(4592)); f_1cc790(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_) } as i32) as u8 as i32) == 0) {
                { let t = 0.0f32; cx.set_lf(80, t); }
                { let t = ((((((cx.f(cx.lpp(187).add(4720)) * cx.f(cx.lpp(187).add(4720))) + (cx.f(cx.lpp(187).add(4732)) * cx.f(cx.lpp(187).add(4732)))) * 2.0f32) as f64) - 1.0f64) as f32); cx.set_lf(196, t); }
                { let t = (((((cx.f(cx.lpp(187).add(4720)) * cx.f(cx.lpp(187).add(4724))) + (cx.f(cx.lpp(187).add(4728)) * cx.f(cx.lpp(187).add(4732)))) as f64) * 2.0f64) as f32); cx.set_lf(200, t); }
                { let t = (((((cx.f(cx.lpp(187).add(4728)) * cx.f(cx.lpp(187).add(4720))) - (cx.f(cx.lpp(187).add(4724)) * cx.f(cx.lpp(187).add(4732)))) as f64) * 2.0f64) as f32); cx.set_lf(204, t); }
                { let q0_ = cx.lptr(220); let q1_ = cx.lptr(196); let q2_ = cx.lptr(208); f_1c0270(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(220); f_1c0380(cx, q0_) };
                { let q0_ = cx.lptr(196); let q1_ = cx.lptr(208); let q2_ = cx.lptr(220); f_1c0270(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(60); let q1_ = cx.lptr(196); f_39d0d0(cx, q0_, q1_) };
                if ((({ let q0_ = cx.lpp(187).bits(); let q1_ = cx.lptr(28).bits(); let q2_ = cx.lptr(40); let q3_ = cx.lptr(28); let q4_ = cx.lptr(16); let q5_ = cx.lptr(60); let q6_ = cx.lpp(187).add(4720); let q7_ = cx.lpp(187).add(4676); let q8_ = cx.lptr(176); let q9_ = cx.f(cx.lpp(187).add(4592)); f_1cc790(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_, q8_, q9_) } as i32) as u8 as i32) != 0) {
                    { let q0_ = cx.lpp(187).add(4628); let q1_ = cx.lptr(40); f_1bb190(cx, q0_, q1_) };
                    { let t = cx.lf(28); cx.set_lf(100, t); }
                    { let t = cx.lf(16); cx.set_lf(88, t); }
                    { let t = 1; cx.set_li(80, t); }
                }
                if (cx.li(80) == 0) {
                    { let p = cx.lpp(187).add(4756); let t = -1; cx.set_i(p, t); }
                }
            }
            if (cx.gc.PL_max_omega_extra_should_be_at_least != 0.0f32) {
                x = ({ let q0_ = cx.lpp(187).add(4628); let q1_ = cx.lpp(187).add(352); f_1c0300(cx, q0_, q1_) } as f32);
                if ((cx.gc.PL_max_omega_extra_should_be_at_least as f64) > vsmath::fabs(x)) {
                    xa = ({ let q0_ = cx.lpp(187).add(4628); let q1_ = cx.lpp(187).add(368); f_1c0300(cx, q0_, q1_) } as f32);
                    if ((cx.gc.PL_max_omega_extra_should_be_at_least as f64) > vsmath::fabs(xa)) {
                        xb = ({ let q0_ = cx.lpp(187).add(4628); let q1_ = cx.lpp(187).add(384); f_1c0300(cx, q0_, q1_) } as f32);
                        if ((cx.gc.PL_max_omega_extra_should_be_at_least as f64) > vsmath::fabs(xb)) {
                            { let p = cx.lpp(187).add(4756); let t = -1; cx.set_i(p, t); }
                        }
                    }
                }
            }
            { let q0_ = cx.lpp(187).add(4628); let q1_ = cx.lf(100); f_1c0110(cx, q0_, q1_) };
            v7 = (v7 & !0xFFFF_FFFFi64) | ((Ptr::gc(0).bits()) as i32 as u32 as i64);
            if (cx.gc.PL_increment_omega_once != 0) {
                if (cx.i(cx.lpp(187).add(4784)) != 0) {
                    { let q0_ = cx.lpp(187).add(288); let q1_ = cx.lpp(187).add(4628); f_1bb190(cx, q0_, q1_) };
                } else {
                    { let q0_ = cx.lpp(187).add(288); let q1_ = cx.lpp(187).add(4628); f_1c0000(cx, q0_, q1_) };
                }
                v7 = (v7 & !0xFFFF_FFFFi64) | ((cx.lpp(187)).bits() as u32 as i64);
                { let p = cx.lpp(187).add(4756); let t = -1; cx.set_i(p, t); }
            } else if (cx.i(cx.lpp(187).add(4756)) != 0) {
                { let p = cx.lpp(187).add(4756); let t = -1; cx.set_i(p, t); }
            } else {
                v7 = (v7 & 0xFFFF_FFFFi64) | (((cx.lpp(187).bits()) as u32 as i64) << 32);
                { let p = cx.lpp(187).add(4756); let t = 1; cx.set_i(p, t); }
                v7 = (v7 & !0xFFFF_FFFFi64) | ((cx.lpp(187)).bits() as u32 as i64);
                { let p = cx.lpp(187).add(1592); let t = 0; cx.set_i(p, t); }
            }
        }
    }
    v11 = v7;
    return v11;
}

/// VehicleEntity__1CE810 0x001CE810
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1ce810(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> () {
    { let p = a1; let t = (f_1c0300(cx, a2, a3) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (f_1c0300(cx, a2.add(12), a3) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (f_1c0300(cx, a2.add(24), a3) as f32); cx.set_f(p, t); }
}

/// VehicleEntity__1CE870 0x001CE870
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1ce870(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> () {
    { let p = a1; let t = (vsmath::fabs(cx.f(a2)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (vsmath::fabs(cx.f(a2.add(4))) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (vsmath::fabs(cx.f(a2.add(8))) as f32); cx.set_f(p, t); }
}

/// VehicleEntity__1CE8C0 0x001CE8C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1ce8c0(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> Ptr {
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

/// VehicleEntity__1CEAB0 0x001CEAB0
pub fn f_1ceab0(cx: &mut Cx, a1: i32, a2: Ptr, a3: Ptr, a4: f32) -> i64 {
    cx.push_frame(92);
    let r = f_1ceab0_body(cx, a1, a2, a3, a4);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1ceab0_body(cx: &mut Cx, a1: i32, a2: Ptr, a3: Ptr, a4: f32) -> i64 {
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
    { let t = (f_1c0300(cx, a3, a3) as f32); cx.set_lf(40, t); }
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
        v10 = (v10 & !0xFFFF_FFFFi64) | (({ let q0_ = a2; let q1_ = cx.lptr(12); let q2_ = cx.lptr(48); f_1cec10(cx, q0_, q1_, q2_) }.bits()) as i32 as u32 as i64);
    }
    v13 = v10;
    return v13;
}

/// VehicleEntity__1CEC10 0x001CEC10
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1cec10(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1.add(12); let t = ((((((cx.f(a2.add(12)) * cx.f(a3.add(12))) - (cx.f(a2) * cx.f(a3))) - (cx.f(a2.add(4)) * cx.f(a3.add(4)))) as f64) - ((cx.f(a2.add(8)) * cx.f(a3.add(8))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1; let t = ((((((cx.f(a2.add(4)) * cx.f(a3.add(8))) - (cx.f(a2.add(8)) * cx.f(a3.add(4)))) + (cx.f(a2.add(12)) * cx.f(a3))) as f64) + ((cx.f(a2) * cx.f(a3.add(12))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = ((((((cx.f(a2.add(8)) * cx.f(a3)) - (cx.f(a2) * cx.f(a3.add(8)))) + (cx.f(a2.add(12)) * cx.f(a3.add(4)))) as f64) + ((cx.f(a2.add(4)) * cx.f(a3.add(12))) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = ((((((cx.f(a2) * cx.f(a3.add(4))) - (cx.f(a2.add(4)) * cx.f(a3))) + (cx.f(a2.add(12)) * cx.f(a3.add(8)))) as f64) + ((cx.f(a2.add(8)) * cx.f(a3.add(12))) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// VehicleEntity__1CED60 0x001CED60
pub fn f_1ced60(cx: &mut Cx, a1: Ptr, a2: Ptr, a3: Ptr, a4: f32) -> i64 {
    cx.push_frame(32);
    let r = f_1ced60_body(cx, a1, a2, a3, a4);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1ced60_body(cx: &mut Cx, a1: Ptr, a2: Ptr, a3: Ptr, a4: f32) -> i64 {
    let mut v4 = 0i32;
    let mut v5 = 0i32;
    let mut v6 = 0i32;
    let mut v7 = 0i32;
    let mut v8 = 0i32;
    let mut v9 = 0i32;
    let mut v10 = 0i64;
    let mut v12 = 0i64;
    { let t = (f_1c0300(cx, a3, a3) as f32); cx.set_lf(28, t); }
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
        v10 = (v10 & !0xFFFF_FFFFi64) | (({ let q0_ = a1; let q1_ = cx.lptr(0); let q2_ = a2; f_1cec10(cx, q0_, q1_, q2_) }.bits()) as i32 as u32 as i64);
    }
    v12 = v10;
    return v12;
}

/// VehicleEntity__1CEE80 0x001CEE80
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1cee80(cx: &mut Cx, mut a1: Ptr, a2: Ptr) -> i32 {
    let mut v3 = 0.0f32;
    let mut v4 = 0.0f32;
    let mut v5 = 0.0f32;
    let mut v6 = 0.0f32;
    let mut v7 = 0.0f32;
    let mut result = 0i32;
    v7 = (((cx.f(a2.add(4)) as f64) * (cx.f(a2.add(4)) as f64)) as f32);
    v6 = (((cx.f(a2.add(8)) as f64) * (cx.f(a2.add(8)) as f64)) as f32);
    v5 = (((cx.f(a2.add(8)) as f64) * (cx.f(a2.add(20)) as f64)) as f32);
    v4 = ((((cx.f(a2.add(16)) * cx.f(a2.add(32))) as f64) - ((cx.f(a2.add(20)) * cx.f(a2.add(20))) as f64)) as f32);
    v3 = ((1.0f64 / ((((((0.0f32 - v6) * cx.f(a2.add(16))) - (v7 * cx.f(a2.add(32)))) + ((2.0f32 * cx.f(a2.add(4))) * v5)) + (v4 * cx.f(a2))) as f64)) as f32);
    { let p = a1; let t = (((v3 as f64) * (v4 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(16); let t = (((((cx.f(a2) * cx.f(a2.add(32))) - v6) as f64) * (v3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(32); let t = (((((cx.f(a2) * cx.f(a2.add(16))) - v7) as f64) * (v3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(12); let t = ((((v5 - (cx.f(a2.add(4)) * cx.f(a2.add(32)))) as f64) * (v3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = cx.i(a1.add(12)); cx.set_i(p, t); }
    { let p = a1.add(24); let t = (((((cx.f(a2.add(4)) * cx.f(a2.add(20))) - (cx.f(a2.add(8)) * cx.f(a2.add(16)))) as f64) * (v3 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = cx.i(a1.add(24)); cx.set_i(p, t); }
    { let p = a1.add(28); let t = (((((cx.f(a2.add(4)) * cx.f(a2.add(8))) - (cx.f(a2) * cx.f(a2.add(20)))) as f64) * (v3 as f64)) as f32); cx.set_f(p, t); }
    result = cx.i(a1.add(28));
    { let p = a1.add(20); let t = result; cx.set_i(p, t); }
    return result;
}

/// VehicleEntity__1CF070 0x001CF070
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1cf070(cx: &mut Cx) -> f32 {
    return (vs_rand_unit(cx) as f32);
}

/// VehicleEntity__1CF090 0x001CF090
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1cf090(cx: &mut Cx) -> f32 {
    let mut v3 = 0i32;
    v3 = ov_rand(cx);
    return ((((((v3 as f64) - 16383.5f64) + (v3 as f64)) - 16383.5f64) / 32767.0f64) as f32);
}

/// VehicleEntity__1CF0D0 0x001CF0D0
pub fn f_1cf0d0(cx: &mut Cx, this: Ptr) -> i64 {
    cx.push_frame(696);
    let r = f_1cf0d0_body(cx, this);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1cf0d0_body(cx: &mut Cx, this: Ptr) -> i64 {
    let mut x = Ptr::NULL;
    let mut v1 = Ptr::NULL;
    let mut v2 = 0.0f64;
    let mut v3 = 0i32;
    let mut v4 = 0i32;
    let mut v5 = 0i32;
    let mut v6 = 0i32;
    let mut v8 = Ptr::NULL;
    let mut v9 = Ptr::NULL;
    let mut xa = 0.0f32;
    let mut xb = 0.0f32;
    let mut xc = 0.0f32;
    let mut xd = 0i32;
    let mut v10 = Ptr::NULL;
    let mut v11 = Ptr::NULL;
    let mut v12 = 0i32;
    let mut v13 = 0.0f64;
    let mut v14 = 0.0f64;
    let mut v15 = 0.0f64;
    let mut v16 = Ptr::NULL;
    let mut v17 = 0.0f64;
    let mut v18 = 0i64;
    let mut v20 = Ptr::NULL;
    let mut v21 = Ptr::NULL;
    let mut v22 = 0.0f32;
    let mut v23 = 0i64;
    let mut _1B74A0 = 0i32;
    let mut bb: u32 = 0;
    #[allow(unreachable_code)]
    'cfg: loop {
        match bb {
            0 => {
                { let t = this; cx.set_lpp(173, t); }
                { let t = ov_hist_entry(cx, this.add(3560), 0); cx.set_lpp(172, t); }
                x = { let q0_ = cx.lpp(173).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                v1 = { let q0_ = cx.lpp(173).add(240); ov_car_velocity(cx, q0_) };
                v2 = f_1c0300(cx, v1, x);
                { let p = cx.lpp(173).add(3704); let t = (v2 as f32); cx.set_f(p, t); }
                v3 = { let q0_ = cx.lpp(173).add(240); f_364b20(cx, q0_) };
                { let p = cx.lpp(172).add(56); let t = v3; cx.set_i(p, t); }
                v4 = { let q0_ = cx.lpp(173).add(240); f_364be0(cx, q0_) };
                { let p = cx.lpp(173).add(3604); let t = v4; cx.set_i(p, t); }
                { let t = ((({ let q0_ = cx.lpp(173).add(240); ov_car_i2876(cx, q0_) } != 0) && (cx.i(cx.lpp(173).add(4856)) > 1)) as i32); cx.set_li(16, t); }
                { let p = cx.lpp(172).add(92); let t = cx.li(16); cx.set_i(p, t); }
                v5 = { let q0_ = cx.lpp(173).add(240); f_35dd20(cx, q0_) };
                { let p = cx.lpp(173).add(3688); let t = v5; cx.set_b(p, t); }
                if (cx.f(cx.lpp(172).add(36)) != 0.0f32) { bb = 1; continue 'cfg; } else { bb = 2; continue 'cfg; }
                break 'cfg;
            }
            1 => {
                if (cx.f(cx.lpp(173).add(3704)) > (0.0f32 - cx.gc.speed_threshold)) { bb = 4; continue 'cfg; } else { bb = 5; continue 'cfg; }
                break 'cfg;
            }
            2 => {
                { bb = 3; continue 'cfg; }
                break 'cfg;
            }
            3 => {
                { bb = 20; continue 'cfg; }
                break 'cfg;
            }
            4 => {
                v6 = { let q0_ = cx.lpp(173).add(240); f_35d1e0(cx, q0_) };
                { let p = cx.lpp(173).add(3216); let t = v6; cx.set_i(p, t); }
                if (cx.i(cx.lpp(173).add(3216)) > 0) { bb = 7; continue 'cfg; } else { bb = 8; continue 'cfg; }
                break 'cfg;
            }
            5 => {
                { bb = 6; continue 'cfg; }
                break 'cfg;
            }
            6 => {
                if (cx.i(cx.lpp(172).add(56)) != 0) { bb = 22; continue 'cfg; } else { bb = 23; continue 'cfg; }
                break 'cfg;
            }
            7 => {
                _1B74A0 = { let q0_ = cx.lpp(173); f_1b74a0(cx, q0_) };
                { let p = cx.lpp(173).add(3216); let t = _1B74A0; cx.set_i(p, t); }
                { bb = 9; continue 'cfg; }
                break 'cfg;
            }
            8 => {
                { let p = cx.lpp(173).add(3216); let t = 1; cx.set_i(p, t); }
                { bb = 9; continue 'cfg; }
                break 'cfg;
            }
            9 => {
                if (cx.i(cx.lpp(173).add(3216)) <= 90) { bb = 10; continue 'cfg; } else { bb = 11; continue 'cfg; }
                break 'cfg;
            }
            10 => {
                if (cx.b(cx.lpp(173).add(3688)) == 0) { bb = 13; continue 'cfg; } else { bb = 14; continue 'cfg; }
                break 'cfg;
            }
            11 => {
                { let p = cx.lpp(173).add(3216); let t = cx.i(p).wrapping_sub(100); cx.set_i(p, t); }
                { let q0_ = cx.lpp(173).add(1444); let q1_ = 0; f_1ba170(cx, q0_, q1_) };
                { bb = 12; continue 'cfg; }
                break 'cfg;
            }
            12 => {
                { bb = 19; continue 'cfg; }
                break 'cfg;
            }
            13 => {
                if (cx.i(cx.lpp(172).add(160)) != 0) { bb = 16; continue 'cfg; } else { bb = 17; continue 'cfg; }
                break 'cfg;
            }
            14 => {
                { bb = 15; continue 'cfg; }
                break 'cfg;
            }
            15 => {
                { bb = 12; continue 'cfg; }
                break 'cfg;
            }
            16 => {
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.f(cx.lpp(172).add(36)); let q2_ = 0; f_35d4b0(cx, q0_, q1_, q2_) };
                { bb = 18; continue 'cfg; }
                break 'cfg;
            }
            17 => {
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.f(cx.lpp(172).add(36)); let q2_ = cx.i(cx.ptr_at(cx.lpp(173).add(3552)).add(108)); f_35d4b0(cx, q0_, q1_, q2_) };
                { bb = 18; continue 'cfg; }
                break 'cfg;
            }
            18 => {
                { bb = 15; continue 'cfg; }
                break 'cfg;
            }
            19 => {
                { let q0_ = cx.lpp(173).add(1460); let q1_ = 0; f_1ba170(cx, q0_, q1_) };
                { bb = 20; continue 'cfg; }
                break 'cfg;
            }
            20 => {
                if ((cx.f(cx.lpp(172).add(164)) != 0.0f32) && (cx.i(cx.lpp(173).add(3732)) == 0)) { bb = 39; continue 'cfg; } else { bb = 40; continue 'cfg; }
                break 'cfg;
            }
            21 => {
                { bb = 6; continue 'cfg; }
                break 'cfg;
            }
            22 => {
                if (cx.b(cx.lpp(173).add(3688)) == 0) { bb = 25; continue 'cfg; } else { bb = 26; continue 'cfg; }
                break 'cfg;
            }
            23 => {
                { bb = 24; continue 'cfg; }
                break 'cfg;
            }
            24 => {
                if (cx.gc.backwards_traction_controlled_braking == 0.0f32) { bb = 29; continue 'cfg; } else { bb = 30; continue 'cfg; }
                break 'cfg;
            }
            25 => {
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.f(cx.lpp(172).add(36)); let q2_ = 0; f_35d6f0(cx, q0_, q1_, q2_) };
                { bb = 27; continue 'cfg; }
                break 'cfg;
            }
            26 => {
                { bb = 27; continue 'cfg; }
                break 'cfg;
            }
            27 => {
                { bb = 19; continue 'cfg; }
                break 'cfg;
            }
            28 => {
                { bb = 24; continue 'cfg; }
                break 'cfg;
            }
            29 => {
                { let p = cx.lpp(173).add(1464); let t = cx.gc.backwards_brake_torque; cx.set_f(p, t); }
                { let p = cx.lpp(173).add(1468); let t = cx.gc.backwards_brake_torque; cx.set_f(p, t); }
                { bb = 31; continue 'cfg; }
                break 'cfg;
            }
            30 => {
                { let t = 1; cx.set_li(684, t); }
                { bb = 32; continue 'cfg; }
                break 'cfg;
            }
            31 => {
                { bb = 3; continue 'cfg; }
                break 'cfg;
            }
            32 => {
                if (cx.li(684) < 3) { bb = 33; continue 'cfg; } else { bb = 35; continue 'cfg; }
                break 'cfg;
            }
            33 => {
                v2 = (({ let q0_ = cx.lpp(173).add(240); let q1_ = cx.li(684); f_364d20(cx, q0_, q1_) } as f64) * (cx.gc.backwards_traction_controlled_braking as f64));
                { let p = cx.lptr(680); let t = (v2 as f32); cx.set_f(p, t); }
                if (cx.gc.backwards_brake_torque <= cx.f(cx.lptr(680))) { bb = 36; continue 'cfg; } else { bb = 37; continue 'cfg; }
                break 'cfg;
            }
            34 => {
                let _ = cx.preinc_i(cx.lptr(684));
                { bb = 32; continue 'cfg; }
                break 'cfg;
            }
            35 => {
                { bb = 31; continue 'cfg; }
                break 'cfg;
            }
            36 => {
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.li(684))).add(1460); let t = cx.gc.backwards_brake_torque; cx.set_f(p, t); }
                { bb = 38; continue 'cfg; }
                break 'cfg;
            }
            37 => {
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.li(684))).add(1460); let t = cx.li(680); cx.set_i(p, t); }
                { bb = 38; continue 'cfg; }
                break 'cfg;
            }
            38 => {
                { bb = 34; continue 'cfg; }
                break 'cfg;
            }
            39 => {
                if ((cx.gc.speed_threshold <= cx.f(cx.lpp(173).add(3704))) || (cx.b(cx.lpp(173).add(3688)) != 0)) { bb = 42; continue 'cfg; } else { bb = 43; continue 'cfg; }
                break 'cfg;
            }
            40 => {
                { bb = 41; continue 'cfg; }
                break 'cfg;
            }
            41 => {
                if (((cx.f(cx.lpp(172).add(164)) == 0.0f32) || (cx.i(cx.lpp(173).add(3732)) != 0)) && (cx.f(cx.lpp(172).add(36)) == 0.0f32)) { bb = 68; continue 'cfg; } else { bb = 69; continue 'cfg; }
                break 'cfg;
            }
            42 => {
                { let q0_ = cx.lpp(173).add(1444); f_1ba220(cx, q0_) };
                if (cx.gc.forwards_traction_controlled_braking == 0.0f32) { bb = 45; continue 'cfg; } else { bb = 46; continue 'cfg; }
                break 'cfg;
            }
            43 => {
                { let q0_ = cx.lpp(173).add(1460); let q1_ = 0; f_1ba170(cx, q0_, q1_) };
                if (cx.b(cx.lpp(173).add(3688)) == 0) { bb = 62; continue 'cfg; } else { bb = 63; continue 'cfg; }
                break 'cfg;
            }
            44 => {
                { bb = 41; continue 'cfg; }
                break 'cfg;
            }
            45 => {
                { let t = 0; cx.set_li(668, t); }
                { bb = 48; continue 'cfg; }
                break 'cfg;
            }
            46 => {
                { let t = 0; cx.set_li(676, t); }
                { bb = 55; continue 'cfg; }
                break 'cfg;
            }
            47 => {
                { bb = 44; continue 'cfg; }
                break 'cfg;
            }
            48 => {
                if (cx.li(668) < 4) { bb = 49; continue 'cfg; } else { bb = 51; continue 'cfg; }
                break 'cfg;
            }
            49 => {
                if ({ let q0_ = cx.lpp(173).add(240); let q1_ = cx.li(668); f_364a90(cx, q0_, q1_) } == -1) { bb = 52; continue 'cfg; } else { bb = 53; continue 'cfg; }
                break 'cfg;
            }
            50 => {
                let _ = cx.preinc_i(cx.lptr(668));
                { bb = 48; continue 'cfg; }
                break 'cfg;
            }
            51 => {
                { bb = 47; continue 'cfg; }
                break 'cfg;
            }
            52 => {
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.li(668))).add(1460); let t = cx.i(cx.ptr_at(cx.lpp(173).add(3552)).add(140)); cx.set_i(p, t); }
                { bb = 54; continue 'cfg; }
                break 'cfg;
            }
            53 => {
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.li(668))).add(1460); let t = (((cx.f(cx.ptr_at(cx.lpp(173).add(3552)).add(144)) as f64) * (cx.f(cx.lpp(172).add(164)) as f64)) as f32); cx.set_f(p, t); }
                { bb = 54; continue 'cfg; }
                break 'cfg;
            }
            54 => {
                { bb = 50; continue 'cfg; }
                break 'cfg;
            }
            55 => {
                if (cx.li(676) < 4) { bb = 56; continue 'cfg; } else { bb = 58; continue 'cfg; }
                break 'cfg;
            }
            56 => {
                { let p = cx.lptr(672); let t = ((({ let q0_ = cx.lpp(173).add(240); let q1_ = cx.li(676); f_364d20(cx, q0_, q1_) } as f64) * (cx.gc.forwards_traction_controlled_braking as f64)) as f32); cx.set_f(p, t); }
                if (cx.f(cx.ptr_at(cx.lpp(173).add(3552)).add(144)) <= cx.f(cx.lptr(672))) { bb = 59; continue 'cfg; } else { bb = 60; continue 'cfg; }
                break 'cfg;
            }
            57 => {
                { let t = cx.li(676).wrapping_add(3); cx.set_li(676, t); }
                { bb = 55; continue 'cfg; }
                break 'cfg;
            }
            58 => {
                { bb = 47; continue 'cfg; }
                break 'cfg;
            }
            59 => {
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.li(676))).add(1460); let t = (((cx.f(cx.ptr_at(cx.lpp(173).add(3552)).add(144)) as f64) * (cx.f(cx.lpp(172).add(164)) as f64)) as f32); cx.set_f(p, t); }
                { bb = 61; continue 'cfg; }
                break 'cfg;
            }
            60 => {
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.li(676))).add(1460); let t = cx.li(672); cx.set_i(p, t); }
                { bb = 61; continue 'cfg; }
                break 'cfg;
            }
            61 => {
                { bb = 57; continue 'cfg; }
                break 'cfg;
            }
            62 => {
                if (cx.i(cx.lpp(172).add(160)) != 0) { bb = 65; continue 'cfg; } else { bb = 66; continue 'cfg; }
                break 'cfg;
            }
            63 => {
                { bb = 64; continue 'cfg; }
                break 'cfg;
            }
            64 => {
                { bb = 44; continue 'cfg; }
                break 'cfg;
            }
            65 => {
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.f(cx.lpp(172).add(164)); let q2_ = 0; f_35d940(cx, q0_, q1_, q2_) };
                { bb = 67; continue 'cfg; }
                break 'cfg;
            }
            66 => {
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.f(cx.lpp(172).add(164)); let q2_ = (cx.gc.backwards_drive_traction_control.to_bits() as i32); f_35d940(cx, q0_, q1_, q2_) };
                { bb = 67; continue 'cfg; }
                break 'cfg;
            }
            67 => {
                { bb = 64; continue 'cfg; }
                break 'cfg;
            }
            68 => {
                { let q0_ = cx.lpp(173).add(1444); let q1_ = 0; f_1ba170(cx, q0_, q1_) };
                { let t = 0; cx.set_li(664, t); }
                { bb = 71; continue 'cfg; }
                break 'cfg;
            }
            69 => {
                { bb = 70; continue 'cfg; }
                break 'cfg;
            }
            70 => {
                if (cx.f(cx.ptr_at(cx.lpp(173).add(3552)).add(80)) != 0.0f32) { bb = 78; continue 'cfg; } else { bb = 79; continue 'cfg; }
                break 'cfg;
            }
            71 => {
                if (cx.li(664) < 4) { bb = 72; continue 'cfg; } else { bb = 74; continue 'cfg; }
                break 'cfg;
            }
            72 => {
                if ({ let q0_ = cx.lpp(173).add(240); let q1_ = cx.li(664); f_364a90(cx, q0_, q1_) } == -1) { bb = 75; continue 'cfg; } else { bb = 76; continue 'cfg; }
                break 'cfg;
            }
            73 => {
                let _ = cx.preinc_i(cx.lptr(664));
                { bb = 71; continue 'cfg; }
                break 'cfg;
            }
            74 => {
                { bb = 70; continue 'cfg; }
                break 'cfg;
            }
            75 => {
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.li(664))).add(1460); let t = cx.i(cx.ptr_at(cx.lpp(173).add(3552)).add(140)); cx.set_i(p, t); }
                { bb = 77; continue 'cfg; }
                break 'cfg;
            }
            76 => {
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.li(664))).add(1460); let t = cx.i(cx.ptr_at(cx.lpp(173).add(3552)).add(136)); cx.set_i(p, t); }
                { bb = 77; continue 'cfg; }
                break 'cfg;
            }
            77 => {
                { bb = 73; continue 'cfg; }
                break 'cfg;
            }
            78 => {
                if (cx.f(cx.lpp(173).add(3704)) <= cx.f(cx.ptr_at(cx.lpp(173).add(3552)).add(80))) { bb = 81; continue 'cfg; } else { bb = 82; continue 'cfg; }
                break 'cfg;
            }
            79 => {
                { bb = 80; continue 'cfg; }
                break 'cfg;
            }
            80 => {
                { let p = cx.lpp(173).add(1592); let t = 0; cx.set_i(p, t); }
                if (cx.i(cx.lpp(172).add(20)) != 0) { bb = 95; continue 'cfg; } else { bb = 96; continue 'cfg; }
                break 'cfg;
            }
            81 => {
                if (cx.f(cx.lpp(173).add(3704)) > (cx.f(cx.ptr_at(cx.lpp(173).add(3552)).add(80)) - cx.f(cx.ptr_at(cx.lpp(173).add(3552)).add(84)))) { bb = 84; continue 'cfg; } else { bb = 85; continue 'cfg; }
                break 'cfg;
            }
            82 => {
                { let t = 0; cx.set_li(660, t); }
                { bb = 91; continue 'cfg; }
                break 'cfg;
            }
            83 => {
                { bb = 80; continue 'cfg; }
                break 'cfg;
            }
            84 => {
                { let t = 0; cx.set_li(660, t); }
                { bb = 87; continue 'cfg; }
                break 'cfg;
            }
            85 => {
                { bb = 86; continue 'cfg; }
                break 'cfg;
            }
            86 => {
                { bb = 83; continue 'cfg; }
                break 'cfg;
            }
            87 => {
                if (cx.li(660) < 4) { bb = 88; continue 'cfg; } else { bb = 90; continue 'cfg; }
                break 'cfg;
            }
            88 => {
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.li(660))).add(1444); let t = (((((cx.f(cx.ptr_at(cx.lpp(173).add(3552)).add(80)) - cx.f(cx.lpp(173).add(3704))) / cx.f(cx.ptr_at(cx.lpp(173).add(3552)).add(84))) as f64) * (cx.f(cx.lpp(173).add((4_i32).wrapping_mul(cx.li(660))).add(1444)) as f64)) as f32); cx.set_f(p, t); }
                { bb = 89; continue 'cfg; }
                break 'cfg;
            }
            89 => {
                let _ = cx.preinc_i(cx.lptr(660));
                { bb = 87; continue 'cfg; }
                break 'cfg;
            }
            90 => {
                { bb = 86; continue 'cfg; }
                break 'cfg;
            }
            91 => {
                if (cx.li(660) < 4) { bb = 92; continue 'cfg; } else { bb = 94; continue 'cfg; }
                break 'cfg;
            }
            92 => {
                { bb = 93; continue 'cfg; }
                break 'cfg;
            }
            93 => {
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.postinc_i(cx.lptr(660)))).add(1444); let t = 0; cx.set_i(p, t); }
                { bb = 91; continue 'cfg; }
                break 'cfg;
            }
            94 => {
                { bb = 83; continue 'cfg; }
                break 'cfg;
            }
            95 => {
                let _ = cx.predec_i(cx.lpp(172).add(20));
                { let q0_ = cx.lpp(173).add(1508); let q1_ = (cx.gc.trafic_low_static_friction_after_collision.to_bits() as i32); f_1ba170(cx, q0_, q1_) };
                { let p = cx.lpp(173).add(432); let t = 1142292480; cx.set_i(p, t); }
                { bb = 97; continue 'cfg; }
                break 'cfg;
            }
            96 => {
                { let t = cx.i(cx.ptr_at(cx.lpp(173).add(3552)).add(296)); cx.set_li(656, t); }
                if (cx.i(cx.lpp(173).add(3756)) != 0) { bb = 98; continue 'cfg; } else { bb = 99; continue 'cfg; }
                break 'cfg;
            }
            97 => {
                if (cx.i(cx.lpp(172).add(92)) != 0) { bb = 104; continue 'cfg; } else { bb = 105; continue 'cfg; }
                break 'cfg;
            }
            98 => {
                { let p = cx.lptr(656); let t = (((cx.f(cx.ptr_at(cx.lpp(173).add(3552)).add(296)) as f64) * (cx.gc.friction_boost_with_disabled_skidding as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.lpp(173).add(3252); let t = cx.i(cx.ptr_at(cx.lpp(173).add(3552)).add(104)); cx.set_i(p, t); }
                { bb = 100; continue 'cfg; }
                break 'cfg;
            }
            99 => {
                { bb = 100; continue 'cfg; }
                break 'cfg;
            }
            100 => {
                if (cx.f(cx.lpp(173).add(3704)) <= 0.0f32) { bb = 101; continue 'cfg; } else { bb = 102; continue 'cfg; }
                break 'cfg;
            }
            101 => {
                { let q0_ = cx.lpp(173).add(1508); let q1_ = { let q0_ = (((cx.f(cx.lptr(656)) as f64) * (cx.gc.friction_boost_when_going_backwards as f64)) as f32); ov_coerce_int(cx, q0_) }; f_1ba170(cx, q0_, q1_) };
                { bb = 103; continue 'cfg; }
                break 'cfg;
            }
            102 => {
                { let q0_ = cx.lpp(173).add(1508); let q1_ = cx.li(656); f_1ba170(cx, q0_, q1_) };
                { bb = 103; continue 'cfg; }
                break 'cfg;
            }
            103 => {
                { let p = cx.ptr_at(cx.lpp(173).add(3552)).add(108); let t = cx.i(cx.ptr_at(cx.lpp(173).add(3552)).add(268)); cx.set_i(p, t); }
                { let p = cx.lpp(173).add(152); let t = 0; cx.set_b(p, t); }
                { bb = 97; continue 'cfg; }
                break 'cfg;
            }
            104 => {
                { let q0_ = cx.lpp(173).add(248); let q1_ = Ptr::gc(12); f_1bb190(cx, q0_, q1_) };
                { let p = cx.lpp(173).add(264); let t = cx.gc.car_linear_damping_when_in_air; cx.set_f(p, t); }
                { let p = cx.lpp(173).add(268); let t = cx.gc.car_angular_damping_when_in_air; cx.set_f(p, t); }
                { bb = 106; continue 'cfg; }
                break 'cfg;
            }
            105 => {
                { let q0_ = cx.lpp(173).add(248); let q1_ = Ptr::gc(0); f_1bb190(cx, q0_, q1_) };
                { let p = cx.lpp(173).add(264); let t = cx.gc.car_linear_damping; cx.set_f(p, t); }
                { let p = cx.lpp(173).add(268); let t = cx.gc.car_angular_damping; cx.set_f(p, t); }
                { bb = 106; continue 'cfg; }
                break 'cfg;
            }
            106 => {
                if (((((cx.gc.PL_enabled != 0) && (cx.i(cx.lpp(172).add(92)) != 0)) && ({ let p = { let q0_ = cx.lpp(173).add(240); ov_car_velocity(cx, q0_) }.add(4); cx.f(p) } > cx.gc.PL_min_speed_up)) && { let _ = { let _ = { let _ = { let _ = { v8 = { let q0_ = cx.lpp(173).add(240); ov_car_velocity(cx, q0_) }; v8 }; { { let p = cx.lptr(12); let t = (((cx.f(v8) as f64) * ({ let p = { let q0_ = cx.lpp(173).add(240); ov_car_velocity(cx, q0_) }; cx.f(p) } as f64)) as f32); cx.set_f(p, t); } } }; { v9 = { let q0_ = cx.lpp(173).add(240); ov_car_velocity(cx, q0_) }; v9 } }; { v10 = { let q0_ = cx.lpp(173).add(240); ov_car_velocity(cx, q0_) }; v10 } }; ((cx.f(cx.lptr(12)) + (cx.f(v9.add(8)) * cx.f(v10.add(8)))) > (cx.gc.PL_min_horisontal_speed * cx.gc.PL_min_horisontal_speed)) }) && (cx.i(cx.lpp(173).add(4716)) == 0)) { bb = 107; continue 'cfg; } else { bb = 108; continue 'cfg; }
                break 'cfg;
            }
            107 => {
                { let t = 0; cx.set_li(592, t); }
                if ((cx.gc.draw_PL_trajectory != 0) && (cx.i(cx.lpp(173).add(4520)) == 0)) { bb = 110; continue 'cfg; } else { bb = 111; continue 'cfg; }
                break 'cfg;
            }
            108 => {
                if ((cx.i(cx.lpp(173).add(4716)) != 0) && ((cx.i(cx.lpp(172).add(92)) == 0) || ((cx.i(cx.lpp(173).add(4756)) == 1) && ((cx.i(cx.lpp(173).add(4856)) <= 1) || (cx.f(cx.lpp(173).add(4596)) > (cx.f(cx.lpp(173).add(4592)) * cx.gc.PL_max_total_flight_time_multiplier)))))) { bb = 145; continue 'cfg; } else { bb = 146; continue 'cfg; }
                break 'cfg;
            }
            109 => {
                if ((cx.i(cx.lpp(173).add(4716)) == 1) && (cx.i(cx.lpp(173).add(4736)) == 0)) { bb = 154; continue 'cfg; } else { bb = 155; continue 'cfg; }
                break 'cfg;
            }
            110 => {
                crate::ov_ignore!(cx, 0 /*str*/);
                { let t = { let q0_ = (12_i32).wrapping_mul(cx.gc.PL_max_NO_points); ov_new(cx, q0_) }; cx.set_lpp(6, t); }
                { let p = cx.lpp(173).add(4520); let t = cx.lpp(6); cx.set_ptr(p, t); }
                crate::ov_ignore!(cx);
                { bb = 112; continue 'cfg; }
                break 'cfg;
            }
            111 => {
                { bb = 112; continue 'cfg; }
                break 'cfg;
            }
            112 => {
                if (cx.i(cx.lpp(173).add(4528)) == 0) { bb = 113; continue 'cfg; } else { bb = 114; continue 'cfg; }
                break 'cfg;
            }
            113 => {
                crate::ov_ignore!(cx, 0 /*str*/);
                { let t = { let q0_ = (12_i32).wrapping_mul(cx.gc.PL_max_NO_points); ov_new(cx, q0_) }; cx.set_lpp(5, t); }
                { let p = cx.lpp(173).add(4528); let t = cx.lpp(5); cx.set_ptr(p, t); }
                crate::ov_ignore!(cx);
                { bb = 115; continue 'cfg; }
                break 'cfg;
            }
            114 => {
                { bb = 115; continue 'cfg; }
                break 'cfg;
            }
            115 => {
                { let _ = { let q0_ = cx.lpp(173).add(240); let q1_ = cx.lptr(620); vcall(cx, 28, q0_, &[Varg::from(q1_)]) }; };
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.lptr(600); f_1b42b0(cx, q0_, q1_) };
                { let q0_ = cx.lptr(640); let q1_ = cx.lptr(620); f_1bb190(cx, q0_, q1_) };
                { let t = 0; cx.set_li(588, t); }
                { bb = 116; continue 'cfg; }
                break 'cfg;
            }
            116 => {
                if ((cx.li(588) < cx.gc.PL_max_NO_points) && ((cx.f(cx.lptr(644)) + cx.gc.PL_max_delta_height) <= cx.f(cx.lptr(624)))) { bb = 117; continue 'cfg; } else { bb = 119; continue 'cfg; }
                break 'cfg;
            }
            117 => {
                if (cx.gc.draw_PL_trajectory != 0) { bb = 120; continue 'cfg; } else { bb = 121; continue 'cfg; }
                break 'cfg;
            }
            118 => {
                let _ = cx.preinc_i(cx.lptr(588));
                { bb = 116; continue 'cfg; }
                break 'cfg;
            }
            119 => {
                if (((((cx.f(cx.lpp(173).add(4592)) > cx.f(Ptr::glob(0x6889F4))) && (cx.b(cx.lpp(173).add(4072)) != 0)) && (cx.i(cx.lpp(173).add(4876)) == 0)) && (cx.b(Ptr::glob(0x6F7158)) == 0)) && (({ let q0_ = cx.b(Ptr::glob(0x749128)); let q1_ = 1; ov_jump_cam_can_see(cx, q0_, q1_) } as u8 as i32) != 0)) { bb = 139; continue 'cfg; } else { bb = 140; continue 'cfg; }
                break 'cfg;
            }
            120 => {
                { let q0_ = cx.ptr_at(cx.lpp(173).add(4520)).add((12_i32).wrapping_mul(cx.i(cx.lpp(173).add(4524)))); let q1_ = cx.lptr(620); f_1bb190(cx, q0_, q1_) };
                let _ = cx.preinc_i(cx.lpp(173).add(4524));
                { bb = 122; continue 'cfg; }
                break 'cfg;
            }
            121 => {
                { bb = 122; continue 'cfg; }
                break 'cfg;
            }
            122 => {
                if ((cx.li(588) % cx.gc.PL_ray_node_timestep_multiplier) == 0) { bb = 123; continue 'cfg; } else { bb = 124; continue 'cfg; }
                break 'cfg;
            }
            123 => {
                { let q0_ = cx.ptr_at(cx.lpp(173).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(173).add(4532)))); let q1_ = cx.lptr(620); f_1bb190(cx, q0_, q1_) };
                if (cx.i(cx.lpp(173).add(4532)) != 0) { bb = 126; continue 'cfg; } else { bb = 127; continue 'cfg; }
                break 'cfg;
            }
            124 => {
                { bb = 125; continue 'cfg; }
                break 'cfg;
            }
            125 => {
                { let p = cx.lptr(592); let t = (((cx.f(cx.lptr(592)) as f64) + (cx.gc.PL_timestep as f64)) as f32); cx.set_f(p, t); }
                { let q0_ = cx.lptr(600); let q1_ = Ptr::gc(12); let q2_ = cx.gc.PL_timestep; f_1c01b0(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(620); let q1_ = cx.lptr(600); let q2_ = cx.gc.PL_timestep; f_1c01b0(cx, q0_, q1_, q2_) };
                { bb = 118; continue 'cfg; }
                break 'cfg;
            }
            126 => {
                v21 = cx.ptr_at(cx.lpp(173).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(173).add(4532))));
                v20 = cx.ptr_at(cx.lpp(173).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(173).add(4532)).wrapping_sub(1)));
                v11 = ov_physics_manager(cx);
                if ((({ let q0_ = v11; let q1_ = v12; let q2_ = cx.lptr(580); let q3_ = cx.lptr(556); let q4_ = v20; let q5_ = v21; let q6_ = -1; let q7_ = 17; f_19d3e0(cx, q0_, q1_, q2_, q3_, q4_, q5_, q6_, q7_) } as i32) as u8 as i32) != 0) { bb = 129; continue 'cfg; } else { bb = 130; continue 'cfg; }
                break 'cfg;
            }
            127 => {
                { bb = 128; continue 'cfg; }
                break 'cfg;
            }
            128 => {
                let _ = cx.preinc_i(cx.lpp(173).add(4532));
                { bb = 125; continue 'cfg; }
                break 'cfg;
            }
            129 => {
                { let q0_ = cx.lpp(173).add(4640); let q1_ = cx.lptr(556); f_1bb190(cx, q0_, q1_) };
                { let p = cx.lpp(173).add(4592); let t = (((((((cx.f(cx.lptr(580)) - 1.0f32) * (cx.gc.PL_ray_node_timestep_multiplier as f32)) * cx.gc.PL_timestep) + cx.f(cx.lptr(592))) as f64) + (cx.gc.PL_flight_time_addition as f64)) as f32); cx.set_f(p, t); }
                { let q0_ = cx.lptr(536); let q1_ = cx.ptr_at(cx.lpp(173).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(173).add(4532)))); let q2_ = cx.ptr_at(cx.lpp(173).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(173).add(4532)).wrapping_sub(1))); f_1c00c0(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lpp(173).add(4600); let q1_ = cx.ptr_at(cx.lpp(173).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(173).add(4532)).wrapping_sub(1))); let q2_ = cx.lptr(536); let q3_ = cx.f(cx.lptr(580)); f_1c0210(cx, q0_, q1_, q2_, q3_) };
                { let q0_ = cx.ptr_at(cx.lpp(173).add(4528)).add((12_i32).wrapping_mul(cx.i(cx.lpp(173).add(4532)))); let q1_ = cx.lpp(173).add(4600); f_1bb190(cx, q0_, q1_) };
                { let q0_ = cx.lpp(173).add(4652); let q1_ = cx.lptr(600); f_1bb190(cx, q0_, q1_) };
                let _ = cx.preinc_i(cx.lpp(173).add(4532));
                { let p = cx.lpp(173).add(4596); let t = 0; cx.set_i(p, t); }
                if ((cx.f(cx.lpp(173).add(4592)) <= cx.gc.PL_min_flight_time) || (cx.f(cx.lpp(173).add(4644)) <= cx.gc.PL_min_normal_up_to_accept)) { bb = 132; continue 'cfg; } else { bb = 133; continue 'cfg; }
                break 'cfg;
            }
            130 => {
                { bb = 131; continue 'cfg; }
                break 'cfg;
            }
            131 => {
                { bb = 128; continue 'cfg; }
                break 'cfg;
            }
            132 => {
                if (cx.f(cx.lpp(173).add(4592)) > (cx.gc.higher_restitution_min_flight_time as f32)) { bb = 135; continue 'cfg; } else { bb = 136; continue 'cfg; }
                break 'cfg;
            }
            133 => {
                { let p = cx.lpp(173).add(4716); let t = 1; cx.set_i(p, t); }
                { bb = 134; continue 'cfg; }
                break 'cfg;
            }
            134 => {
                { bb = 119; continue 'cfg; }
                break 'cfg;
            }
            135 => {
                { let p = cx.lpp(173).add(4852); let t = 1; cx.set_i(p, t); }
                { bb = 137; continue 'cfg; }
                break 'cfg;
            }
            136 => {
                { bb = 137; continue 'cfg; }
                break 'cfg;
            }
            137 => {
                { let p = cx.lpp(173).add(4716); let t = -1; cx.set_i(p, t); }
                { bb = 134; continue 'cfg; }
                break 'cfg;
            }
            138 => {
                { bb = 131; continue 'cfg; }
                break 'cfg;
            }
            139 => {
                { let p = Ptr::glob(0x749110); let t = 2; cx.set_i(p, t); }
                { let q0_ = cx.b(Ptr::glob(0x749128)); ov_jump_cam_start(cx, q0_) };
                { let p = Ptr::glob(0x749118); let t = (((cx.f(cx.lpp(173).add(4592)) as f64) - (cx.f(Ptr::glob(0x6889FC)) as f64)) as f32); cx.set_f(p, t); }
                { let p = Ptr::glob(0x74911C); let t = cx.i(Ptr::glob(0x749118)); cx.set_i(p, t); }
                { let p = cx.lpp(173).add(4876); let t = 96; cx.set_i(p, t); }
                { bb = 141; continue 'cfg; }
                break 'cfg;
            }
            140 => {
                { bb = 141; continue 'cfg; }
                break 'cfg;
            }
            141 => {
                if (cx.i(cx.lpp(173).add(4716)) != 0) { bb = 142; continue 'cfg; } else { bb = 143; continue 'cfg; }
                break 'cfg;
            }
            142 => {
                { let q0_ = cx.lpp(173).add(248); let q1_ = Ptr::gc(12); f_1bb190(cx, q0_, q1_) };
                { let p = cx.lpp(173).add(264); let t = cx.gc.car_linear_damping_when_in_air; cx.set_f(p, t); }
                { let p = cx.lpp(173).add(268); let t = cx.gc.car_angular_damping_when_in_air; cx.set_f(p, t); }
                { bb = 144; continue 'cfg; }
                break 'cfg;
            }
            143 => {
                { let p = cx.lpp(173).add(4716); let t = -1; cx.set_i(p, t); }
                { bb = 144; continue 'cfg; }
                break 'cfg;
            }
            144 => {
                { bb = 109; continue 'cfg; }
                break 'cfg;
            }
            145 => {
                if (cx.i(cx.lpp(173).add(4756)) == 1) { bb = 148; continue 'cfg; } else { bb = 149; continue 'cfg; }
                break 'cfg;
            }
            146 => {
                { bb = 147; continue 'cfg; }
                break 'cfg;
            }
            147 => {
                { bb = 109; continue 'cfg; }
                break 'cfg;
            }
            148 => {
                if (cx.i(cx.lpp(173).add(4784)) != 0) { bb = 151; continue 'cfg; } else { bb = 152; continue 'cfg; }
                break 'cfg;
            }
            149 => {
                { bb = 150; continue 'cfg; }
                break 'cfg;
            }
            150 => {
                { let p = cx.lpp(173).add(4524); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(173).add(4532); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(173).add(4592); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(173).add(4716); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(173).add(4596); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(173).add(4736); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(173).add(4756); let t = 0; cx.set_i(p, t); }
                { bb = 147; continue 'cfg; }
                break 'cfg;
            }
            151 => {
                { let q0_ = cx.lptr(424); let q1_ = cx.lpp(173).add(4688); let q2_ = cx.lpp(173).add(4628); let q3_ = cx.f(cx.lpp(173).add(4596)); f_1ced60(cx, q0_, q1_, q2_, q3_) };
                { let q0_ = cx.lptr(380); let q1_ = cx.lptr(424); f_1ce8c0(cx, q0_, q1_) };
                { bb = 153; continue 'cfg; }
                break 'cfg;
            }
            152 => {
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.lptr(380); f_36a090(cx, q0_, q1_) };
                { let q0_ = cx.lptr(424); let q1_ = cx.lptr(380); f_39d0d0(cx, q0_, q1_) };
                { let q0_ = Ptr::NULL.bits(); let q1_ = cx.lptr(424); let q2_ = cx.lpp(173).add(4628); let q3_ = cx.f(cx.lpp(173).add(4596)); f_1ceab0(cx, q0_, q1_, q2_, q3_) };
                { bb = 153; continue 'cfg; }
                break 'cfg;
            }
            153 => {
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.lptr(424); f_36a1e0(cx, q0_, q1_) };
                { let _ = { let q0_ = cx.lpp(173).add(240); vcall(cx, 32, q0_, &[]) }; };
                { bb = 150; continue 'cfg; }
                break 'cfg;
            }
            154 => {
                { let p = cx.lpp(173).add(4592); let t = (((cx.f(cx.lpp(173).add(4592)) as f64) + (cx.f(cx.ptr_at(cx.lpp(173).add(3552))) as f64)) as f32); cx.set_f(p, t); }
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.lpp(173).add(4688); f_36a140(cx, q0_, q1_) };
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.lpp(173).add(4664); f_1da680(cx, q0_, q1_) };
                { let q0_ = cx.lpp(173).add(4676); let q1_ = cx.lpp(173).add(4664); f_1bb190(cx, q0_, q1_) };
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.lptr(316); f_369d30(cx, q0_, q1_) };
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.lptr(272); f_369da0(cx, q0_, q1_) };
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.lptr(228); f_369ec0(cx, q0_, q1_) };
                { let q0_ = cx.lpp(173).add(240); let q1_ = cx.lptr(336); f_36a090(cx, q0_, q1_) };
                if (cx.f(cx.lpp(173).add(268)) <= 0.0f32) { bb = 157; continue 'cfg; } else { bb = 158; continue 'cfg; }
                break 'cfg;
            }
            155 => {
                { bb = 156; continue 'cfg; }
                break 'cfg;
            }
            156 => {
                if ((cx.i(cx.lpp(173).add(4736)) != 0) && (cx.i(cx.lpp(173).add(4756)) == 0)) { bb = 160; continue 'cfg; } else { bb = 161; continue 'cfg; }
                break 'cfg;
            }
            157 => {
                { let t = 0; cx.set_li(220, t); }
                { bb = 159; continue 'cfg; }
                break 'cfg;
            }
            158 => {
                { let p = cx.lptr(220); let t = (((vsmath::exp(((((0.0f32 - cx.gc.PL_timestep) as f64) / (cx.f(cx.lpp(173).add(268)) as f64)) as f32)) - 1.0f64) / (cx.gc.PL_timestep as f64)) as f32); cx.set_f(p, t); }
                { bb = 159; continue 'cfg; }
                break 'cfg;
            }
            159 => {
                { let q0_ = cx.lpp(173).add(4704); let q1_ = cx.lptr(272); let q2_ = cx.lpp(173).add(4676); f_1ce810(cx, q0_, q1_, q2_) };
                { let p = cx.lpp(173).add(4736); let t = 1; cx.set_i(p, t); }
                { bb = 156; continue 'cfg; }
                break 'cfg;
            }
            160 => {
                { let q0_ = cx.lptr(192); let q1_ = cx.lpp(173).add(4640); f_1bb190(cx, q0_, q1_) };
                { let q0_ = cx.lptr(204); let q1_ = cx.lpp(173).add(4652); let q2_ = cx.lpp(173).add(4640); f_1c0270(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(204); f_1c0380(cx, q0_) };
                { let q0_ = cx.lptr(180); let q1_ = cx.lptr(192); let q2_ = cx.lptr(204); f_1c0270(cx, q0_, q1_, q2_) };
                if (cx.gc.PL_roll_landing_disturbance != 0.0f32) { bb = 163; continue 'cfg; } else { bb = 164; continue 'cfg; }
                break 'cfg;
            }
            161 => {
                { bb = 162; continue 'cfg; }
                break 'cfg;
            }
            162 => {
                if (cx.i(cx.lpp(173).add(5080)) != -1) { bb = 202; continue 'cfg; } else { bb = 203; continue 'cfg; }
                break 'cfg;
            }
            163 => {
                { let p = cx.lptr(172); let t = (((f_1cf090(cx) as f64) * (cx.gc.PL_roll_landing_disturbance as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.lptr(168); let t = (vsmath::cos(cx.f(cx.lptr(172))) as f32); cx.set_f(p, t); }
                { let p = cx.lptr(164); let t = (vsmath::sin(cx.f(cx.lptr(172))) as f32); cx.set_f(p, t); }
                { let q0_ = cx.lptr(148); let q1_ = cx.lptr(192); let q2_ = cx.f(cx.lptr(168)); let q3_ = cx.lptr(180); let q4_ = cx.f(cx.lptr(164)); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
                { let q0_ = cx.lptr(180); let q1_ = cx.lptr(192); let q2_ = ((0.0f64 - (cx.f(cx.lptr(164)) as f64)) as f32); let q3_ = cx.lptr(180); let q4_ = cx.f(cx.lptr(168)); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
                { let q0_ = cx.lptr(192); let q1_ = cx.lptr(148); f_1bb190(cx, q0_, q1_) };
                { bb = 165; continue 'cfg; }
                break 'cfg;
            }
            164 => {
                { bb = 165; continue 'cfg; }
                break 'cfg;
            }
            165 => {
                if (cx.gc.PL_pitch_landing_disturbance != 0.0f32) { bb = 166; continue 'cfg; } else { bb = 167; continue 'cfg; }
                break 'cfg;
            }
            166 => {
                { let p = cx.lptr(140); let t = (((f_1cf090(cx) as f64) * (cx.gc.PL_pitch_landing_disturbance as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.lptr(136); let t = (vsmath::cos(cx.f(cx.lptr(140))) as f32); cx.set_f(p, t); }
                { let p = cx.lptr(132); let t = (vsmath::sin(cx.f(cx.lptr(140))) as f32); cx.set_f(p, t); }
                { let q0_ = cx.lptr(116); let q1_ = cx.lptr(192); let q2_ = cx.f(cx.lptr(136)); let q3_ = cx.lptr(204); let q4_ = cx.f(cx.lptr(132)); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
                { let q0_ = cx.lptr(204); let q1_ = cx.lptr(192); let q2_ = ((0.0f64 - (cx.f(cx.lptr(132)) as f64)) as f32); let q3_ = cx.lptr(204); let q4_ = cx.f(cx.lptr(136)); f_1c4070(cx, q0_, q1_, q2_, q3_, q4_) };
                { let q0_ = cx.lptr(192); let q1_ = cx.lptr(116); f_1bb190(cx, q0_, q1_) };
                { bb = 168; continue 'cfg; }
                break 'cfg;
            }
            167 => {
                { bb = 168; continue 'cfg; }
                break 'cfg;
            }
            168 => {
                { let q0_ = cx.lpp(173).add(4740); let q1_ = cx.lptr(180); f_39d0d0(cx, q0_, q1_) };
                { let q0_ = cx.lpp(173).add(4612); let q1_ = cx.lpp(173).add(4740); let q2_ = cx.lpp(173).add(4688); f_1c5080(cx, q0_, q1_, q2_) };
                { let p = cx.lpp(173).add(4784); let t = 1; cx.set_i(p, t); }
                { let q0_ = cx.lpp(173).add(4628); let q1_ = cx.lpp(173).add(4612); f_1cca40(cx, q0_, q1_) };
                if (vsmath::fabs(cx.f(cx.lpp(173).add(4624))) > 1.0f64) { bb = 169; continue 'cfg; } else { bb = 170; continue 'cfg; }
                break 'cfg;
            }
            169 => {
                { let p = cx.lpp(173).add(4756); let t = -1; cx.set_i(p, t); }
                { bb = 171; continue 'cfg; }
                break 'cfg;
            }
            170 => {
                { let q0_ = cx.lpp(173).add(4624); let q1_ = -1.0f32; let q2_ = 1.0f32; f_1bfe80(cx, q0_, q1_, q2_) };
                v13 = vsmath::acos(cx.f(cx.lpp(173).add(4624)));
                { let p = cx.lptr(84); let t = ((v13 + v13) as f32); cx.set_f(p, t); }
                { let q0_ = cx.lptr(84); f_1d1080(cx, q0_) };
                if (cx.f(cx.lptr(84)) < 0.0f32) { bb = 172; continue 'cfg; } else { bb = 173; continue 'cfg; }
                break 'cfg;
            }
            171 => {
                { bb = 162; continue 'cfg; }
                break 'cfg;
            }
            172 => {
                { let q0_ = cx.lpp(173).add(4628); f_1cc9f0(cx, q0_) };
                { let p = cx.lptr(84); let t = ((0.0f64 - (cx.f(cx.lptr(84)) as f64)) as f32); cx.set_f(p, t); }
                { bb = 174; continue 'cfg; }
                break 'cfg;
            }
            173 => {
                { bb = 174; continue 'cfg; }
                break 'cfg;
            }
            174 => {
                if (cx.f(cx.lptr(84)) > 3.1415927f32) { bb = 175; continue 'cfg; } else { bb = 176; continue 'cfg; }
                break 'cfg;
            }
            175 => {
                { let p = cx.lptr(84); let t = ((6.2831855f64 - (cx.f(cx.lptr(84)) as f64)) as f32); cx.set_f(p, t); }
                { let q0_ = cx.lpp(173).add(4628); f_1cc9f0(cx, q0_) };
                { bb = 177; continue 'cfg; }
                break 'cfg;
            }
            176 => {
                { bb = 177; continue 'cfg; }
                break 'cfg;
            }
            177 => {
                { let t = 0; cx.set_li(76, t); }
                { let p = cx.lptr(72); let t = ({ let q0_ = cx.lpp(173).add(4628); let q1_ = cx.lptr(96); f_1c0300(cx, q0_, q1_) } as f32); cx.set_f(p, t); }
                { let p = cx.lptr(68); let t = (((cx.f(cx.lptr(84)) as f64) / (cx.f(cx.lpp(173).add(4592)) as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.lptr(64); let t = ((((cx.f(cx.lptr(72)) * cx.gc.PL_max_omega_change_omega_dependent) as f64) + (cx.gc.PL_max_omega_change_const as f64)) as f32); cx.set_f(p, t); }
                v14 = vsmath::fabs(cx.f(cx.lptr(68)));
                if (v14 > (cx.f(cx.lptr(64)) as f64)) { bb = 178; continue 'cfg; } else { bb = 179; continue 'cfg; }
                break 'cfg;
            }
            178 => {
                { let t = 0; cx.set_li(60, t); }
                { let p = cx.lpp(173).add(4756); let t = -1; cx.set_i(p, t); }
                { bb = 180; continue 'cfg; }
                break 'cfg;
            }
            179 => {
                { bb = 180; continue 'cfg; }
                break 'cfg;
            }
            180 => {
                if (cx.gc.PL_max_omega_extra_should_be_at_least != 0.0f32) { bb = 181; continue 'cfg; } else { bb = 182; continue 'cfg; }
                break 'cfg;
            }
            181 => {
                xa = ({ let q0_ = cx.lpp(173).add(4628); let q1_ = cx.lpp(173).add(352); f_1c0300(cx, q0_, q1_) } as f32);
                if ((cx.gc.PL_max_omega_extra_should_be_at_least as f64) > vsmath::fabs(xa)) { bb = 184; continue 'cfg; } else { bb = 185; continue 'cfg; }
                break 'cfg;
            }
            182 => {
                { bb = 183; continue 'cfg; }
                break 'cfg;
            }
            183 => {
                { let q0_ = cx.lpp(173).add(4628); let q1_ = cx.f(cx.lptr(68)); f_1c0110(cx, q0_, q1_) };
                if (cx.gc.PL_increment_omega_once != 0) { bb = 193; continue 'cfg; } else { bb = 194; continue 'cfg; }
                break 'cfg;
            }
            184 => {
                xb = ({ let q0_ = cx.lpp(173).add(4628); let q1_ = cx.lpp(173).add(368); f_1c0300(cx, q0_, q1_) } as f32);
                if ((cx.gc.PL_max_omega_extra_should_be_at_least as f64) > vsmath::fabs(xb)) { bb = 187; continue 'cfg; } else { bb = 188; continue 'cfg; }
                break 'cfg;
            }
            185 => {
                { bb = 186; continue 'cfg; }
                break 'cfg;
            }
            186 => {
                { bb = 183; continue 'cfg; }
                break 'cfg;
            }
            187 => {
                xc = ({ let q0_ = cx.lpp(173).add(4628); let q1_ = cx.lpp(173).add(384); f_1c0300(cx, q0_, q1_) } as f32);
                if ((cx.gc.PL_max_omega_extra_should_be_at_least as f64) > vsmath::fabs(xc)) { bb = 190; continue 'cfg; } else { bb = 191; continue 'cfg; }
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
                { let p = cx.lpp(173).add(4756); let t = -1; cx.set_i(p, t); }
                { bb = 192; continue 'cfg; }
                break 'cfg;
            }
            191 => {
                { bb = 192; continue 'cfg; }
                break 'cfg;
            }
            192 => {
                { bb = 189; continue 'cfg; }
                break 'cfg;
            }
            193 => {
                if (cx.i(cx.lpp(173).add(4784)) != 0) { bb = 196; continue 'cfg; } else { bb = 197; continue 'cfg; }
                break 'cfg;
            }
            194 => {
                if (cx.i(cx.lpp(173).add(4756)) == 0) { bb = 199; continue 'cfg; } else { bb = 200; continue 'cfg; }
                break 'cfg;
            }
            195 => {
                { bb = 171; continue 'cfg; }
                break 'cfg;
            }
            196 => {
                { let q0_ = cx.lpp(173).add(288); let q1_ = cx.lpp(173).add(4628); f_1bb190(cx, q0_, q1_) };
                { bb = 198; continue 'cfg; }
                break 'cfg;
            }
            197 => {
                { let q0_ = cx.lpp(173).add(288); let q1_ = cx.lpp(173).add(4628); f_1c0000(cx, q0_, q1_) };
                { bb = 198; continue 'cfg; }
                break 'cfg;
            }
            198 => {
                { let p = cx.lpp(173).add(4756); let t = -1; cx.set_i(p, t); }
                { bb = 195; continue 'cfg; }
                break 'cfg;
            }
            199 => {
                { let p = cx.lpp(173).add(4756); let t = 1; cx.set_i(p, t); }
                { let q0_ = cx.lptr(44); let q1_ = cx.lpp(173).add(4628); let q2_ = cx.lpp(173).add(4676); f_1d0fd0(cx, q0_, q1_, q2_) };
                { let t = ((({ let q0_ = cx.lptr(44); f_1c0330(cx, q0_) } as f64) > (cx.gc.PL_min_omega_to_disable_CD as f64)) as i32); cx.set_li(8, t); }
                { let p = cx.lpp(173).add(4760); let t = cx.li(8); cx.set_b(p, t); }
                { bb = 201; continue 'cfg; }
                break 'cfg;
            }
            200 => {
                { bb = 201; continue 'cfg; }
                break 'cfg;
            }
            201 => {
                { bb = 195; continue 'cfg; }
                break 'cfg;
            }
            202 => {
                { let t = 0; cx.set_li(36, t); }
                { bb = 205; continue 'cfg; }
                break 'cfg;
            }
            203 => {
                { bb = 204; continue 'cfg; }
                break 'cfg;
            }
            204 => {
                let _ = cx.preinc_i(cx.lpp(173).add(4048));
                { let t = 0; cx.set_li(32, t); }
                { bb = 209; continue 'cfg; }
                break 'cfg;
            }
            205 => {
                if (cx.li(36) < 4) { bb = 206; continue 'cfg; } else { bb = 208; continue 'cfg; }
                break 'cfg;
            }
            206 => {
                xd = cx.li(36);
                { let p = cx.lptr(4); let t = (-cx.f(cx.lpp(173).add(3704))); cx.set_f(p, t); }
                v15 = ({ let q0_ = cx.lpp(173).add(240); let q1_ = cx.li(36); f_35fb40(cx, q0_, q1_) } as f64);
                v22 = (((cx.f(cx.lptr(4)) as f64) / v15) as f32);
                { let q0_ = cx.lpp(173).add(240); let q1_ = (v22.to_bits() as i32); let q2_ = xd; f_361530(cx, q0_, q1_, q2_) };
                { bb = 207; continue 'cfg; }
                break 'cfg;
            }
            207 => {
                let _ = cx.preinc_i(cx.lptr(36));
                { bb = 205; continue 'cfg; }
                break 'cfg;
            }
            208 => {
                { bb = 204; continue 'cfg; }
                break 'cfg;
            }
            209 => {
                if (cx.li(32) < 4) { bb = 210; continue 'cfg; } else { bb = 212; continue 'cfg; }
                break 'cfg;
            }
            210 => {
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.li(32))).add(1508); let t = (((cx.f(cx.lpp(173).add((4_i32).wrapping_mul(cx.li(32))).add(1508)) as f64) * ({ let p = { let q0_ = cx.b(cx.lpp(173).add(cx.li(32)).add(4868)); f_19c1a0(cx, q0_) }; cx.f(p) } as f64)) as f32); cx.set_f(p, t); }
                { bb = 211; continue 'cfg; }
                break 'cfg;
            }
            211 => {
                let _ = cx.preinc_i(cx.lptr(32));
                { bb = 209; continue 'cfg; }
                break 'cfg;
            }
            212 => {
                { let q0_ = cx.lpp(173).add(1508); let q1_ = cx.f(cx.lpp(173).add(4880)); f_1d1020(cx, q0_, q1_) };
                { let t = 0; cx.set_li(28, t); }
                { bb = 213; continue 'cfg; }
                break 'cfg;
            }
            213 => {
                if (cx.li(28) < 4) { bb = 214; continue 'cfg; } else { bb = 216; continue 'cfg; }
                break 'cfg;
            }
            214 => {
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.li(28))).add(1508); let t = (((cx.f(cx.lpp(173).add((4_i32).wrapping_mul(cx.li(28))).add(1508)) as f64) * (cx.f(cx.lpp(173).add(4912)) as f64)) as f32); cx.set_f(p, t); }
                { let p = cx.lpp(173).add((4_i32).wrapping_mul(cx.li(28))).add(1444); let t = (((cx.f(cx.lpp(173).add((4_i32).wrapping_mul(cx.li(28))).add(1444)) as f64) * (cx.f(cx.lpp(173).add(4904)) as f64)) as f32); cx.set_f(p, t); }
                { bb = 215; continue 'cfg; }
                break 'cfg;
            }
            215 => {
                let _ = cx.preinc_i(cx.lptr(28));
                { bb = 213; continue 'cfg; }
                break 'cfg;
            }
            216 => {
                if ({ let p = { let q0_ = cx.lpp(173).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }.add(24); cx.i(p) } != 0) { bb = 217; continue 'cfg; } else { bb = 218; continue 'cfg; }
                break 'cfg;
            }
            217 => {
                { let t = { let q0_ = cx.lpp(173).add(3560); let q1_ = 0; ov_hist_entry(cx, q0_, q1_) }.add(24); cx.set_lpp(0, t); }
                let _ = cx.predec_i(cx.lpp(0));
                { bb = 219; continue 'cfg; }
                break 'cfg;
            }
            218 => {
                { bb = 219; continue 'cfg; }
                break 'cfg;
            }
            219 => {
                v16 = { let q0_ = cx.lpp(173).add(240); ov_car_velocity(cx, q0_) };
                v17 = (f_1c0330(cx, v16) as f64);
                { let p = cx.lpp(173).add(20); let t = (v17 as f32); cx.set_f(p, t); }
                v18 = (v18 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.lpp(173).add(3560); ov_hist_push(cx, q0_) }) as i32 as u32 as i64);
                v23 = v18;
                return v23;
            }
            220 => {
                break 'cfg;
            }
            221 => {
                break 'cfg;
            }
            _ => unreachable!(),
        }
    }
    #[allow(unreachable_code)]
    0
}

/// VehicleEntity__1D0FD0 0x001D0FD0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d0fd0(cx: &mut Cx, mut a1: Ptr, a2: Ptr, a3: Ptr) -> Ptr {
    { let p = a1; let t = (((cx.f(a2) as f64) + (cx.f(a3) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a2.add(4)) as f64) + (cx.f(a3.add(4)) as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a2.add(8)) as f64) + (cx.f(a3.add(8)) as f64)) as f32); cx.set_f(p, t); }
    return a2;
}

/// VehicleEntity__1D1020 0x001D1020
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d1020(cx: &mut Cx, mut a1: Ptr, a2: f32) -> Ptr {
    { let p = a1; let t = (((cx.f(a1) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = (((cx.f(a1.add(4)) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(8); let t = (((cx.f(a1.add(8)) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(12); let t = (((cx.f(a1.add(12)) as f64) * (a2 as f64)) as f32); cx.set_f(p, t); }
    return a1;
}

/// VehicleEntity__1D1080 0x001D1080
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d1080(cx: &mut Cx, mut a1: Ptr) -> Ptr {
    let mut result = Ptr::NULL;
    result = a1;
    { let p = a1; let t = (((cx.f(a1) as f64) - (((((cx.f(a1) / 6.2831855f32) as i32) as f32) * 6.2831855f32) as f64)) as f32); cx.set_f(p, t); }
    'lp3: loop {
        if !(cx.f(a1) > 3.1415927f32) { break 'lp3; }
        'blp3: {
            result = a1;
        }
        { let p = a1; let t = (((cx.f(a1) as f64) - 6.2831855f64) as f32); cx.set_f(p, t); }
    }
    'lp4: while ((0.0f32 - 3.1415927f32) >= cx.f(a1)) {
        result = a1;
        { let p = a1; let t = (((cx.f(a1) as f64) + 6.2831855f64) as f32); cx.set_f(p, t); }
    }
    return result;
}

/// VehicleEntity__1D1120 0x001D1120
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d1120(cx: &mut Cx, this: Ptr) -> i32 {
    { let p = this.add(3820); let t = 0; cx.set_i(p, t); }
    { let p = this.add(88); let t = 0; cx.set_b(p, t); }
    { let p = this.add(3824); let t = 1; cx.set_i(p, t); }
    { let p = this.add(3580); let t = 0; cx.set_i(p, t); }
    { let p = this.add(3588); let t = 0; cx.set_i(p, t); }
    return this.bits();
}

/// VehicleEntity__1D1170 0x001D1170
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d1170(cx: &mut Cx, this: Ptr) -> Ptr {
    { let p = this.add(4876); let t = 96; cx.set_i(p, t); }
    return this;
}

/// VehicleEntity__1D1190 0x001D1190
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d1190(cx: &mut Cx, this: Ptr, a2: i32, a3: i32) -> Ptr {
    { let p = this.add(4904); let t = a2; cx.set_i(p, t); }
    { let p = this.add(4908); let t = a3; cx.set_i(p, t); }
    return this;
}

/// VehicleEntity__1D11D0 0x001D11D0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d11d0(cx: &mut Cx, this: Ptr, a2: i32, a3: i32) -> Ptr {
    { let p = this.add(4912); let t = a2; cx.set_i(p, t); }
    { let p = this.add(4916); let t = a3; cx.set_i(p, t); }
    return this;
}

/// VehicleEntity__1D1210 0x001D1210
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d1210(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = cx.preinc_i(this.add(3216));
    return this;
}

/// VehicleEntity__1D1240 0x001D1240
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d1240(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = cx.predec_i(this.add(3216));
    return this;
}

/// VehicleEntity__1D1270 0x001D1270
pub fn f_1d1270(cx: &mut Cx, this: Ptr) -> i64 {
    cx.push_frame(32);
    let r = f_1d1270_body(cx, this);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1d1270_body(cx: &mut Cx, this: Ptr) -> i64 {
    let mut v1 = 0i64;
    let mut v2 = Ptr::NULL;
    let mut v4 = 0.0f32;
    let mut v5 = 0.0f32;
    let mut v6 = 0i64;
    let mut v7 = 0.0f32;
    let mut v8 = 0.0f32;
    let mut v10 = Ptr::NULL;
    v10 = this;
    { let p = cx.lptr(28); let t = (f_1da9a0(cx, Ptr::glob(0x6FC098)) as f32); cx.set_f(p, t); }
    { let t = 0; cx.set_li(24, t); }
    if ((cx.f(cx.lptr(28)) <= 0.85f32) || (cx.f(cx.lptr(28)) > 0.92f32)) {
        if ((cx.f(cx.lptr(28)) <= 0.92f32) || (cx.f(cx.lptr(28)) > 1.0f32)) {
            if (cx.f(cx.lptr(28)) <= 1.0f32) {
                { let p = v10.add(4920); let t = 1; cx.set_i(p, t); }
            } else {
                let _ = f_1d1190(cx, v10, 1062836634, 1084227584);
                let _ = f_1d11d0(cx, v10, 1048576000, 1082130432);
                v1 = (v1 & 0xFFFF_FFFFi64) | (((v10.bits()) as u32 as i64) << 32);
                { let p = v10.add(4920); let t = 4; cx.set_i(p, t); }
            }
        } else {
            let _ = f_1d1190(cx, v10, 1067450368, 1084227584);
            { let t = 1090519040; cx.set_li(24, t); }
            { let p = v10.add(4920); let t = 3; cx.set_i(p, t); }
        }
    } else {
        let _ = f_1d1190(cx, v10, 1066192077, 1077936128);
        { let t = 1082130432; cx.set_li(24, t); }
        { let p = v10.add(4920); let t = 2; cx.set_i(p, t); }
    }
    if (cx.f(cx.lptr(24)) > 0.0f32) {
        v7 = cx.f(cx.lptr(24));
        v2 = f_1c66c0(cx, v10);
        { let q0_ = v2; let q1_ = cx.lptr(4); let q2_ = v7; ov_vec4_mul(cx, q0_, q1_, q2_) };
        v8 = { let q0_ = cx.lptr(4); ov_vec4_z(cx, q0_) };
        v5 = { let q0_ = cx.lptr(4); ov_vec4_y(cx, q0_) };
        v4 = { let q0_ = cx.lptr(4); ov_vec4_x(cx, q0_) };
        let _ = f_36b530(cx, v10.add(240), (v4.to_bits() as i32), (v5.to_bits() as i32), (v8.to_bits() as i32));
        { let _ = { let q0_ = v10.add(240); vcall(cx, 32, q0_, &[]) }; };
    }
    v1 = (v1 & !0xFFFF_FFFFi64) | ((v10.bits()) as i32 as u32 as i64);
    if (cx.b(v10.add(4900)) != 0) {
        v1 = (v1 & 0xFFFF_FFFFi64) | (((v10.bits()) as u32 as i64) << 32);
        { let p = v10.add(3532); let t = -1082130432; cx.set_i(p, t); }
        v1 = (v1 & !0xFFFF_FFFFi64) | ((v10.bits()) as i32 as u32 as i64);
        { let p = v10.add(4900); let t = 0; cx.set_b(p, t); }
    }
    v6 = v1;
    return v6;
}

/// VehicleEntity__1D1480 0x001D1480
pub fn f_1d1480(cx: &mut Cx, a1: i32) -> i64 {
    cx.push_frame(52);
    let r = f_1d1480_body(cx, a1);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1d1480_body(cx: &mut Cx, a1: i32) -> i64 {
    let mut v1 = 0i64;
    let mut v2 = Ptr::NULL;
    let mut v3 = Ptr::NULL;
    let mut v5 = 0.0f32;
    let mut v6 = 0.0f32;
    let mut v7 = 0i64;
    let mut v8 = 0.0f32;
    let mut v9 = Ptr::NULL;
    let mut v10 = 0.0f32;
    { let t = a1; cx.set_lpp(12, Ptr::raw(t)); }
    { let p = cx.lptr(44); let t = (f_1da9a0(cx, Ptr::glob(0x6FC098)) as f32); cx.set_f(p, t); }
    { let t = 0; cx.set_li(40, t); }
    if ((cx.f(cx.lptr(44)) <= 0.85f32) || (cx.f(cx.lptr(44)) > 0.92f32)) {
        if (cx.f(cx.lptr(44)) <= 0.92f32) {
            v1 = (v1 & 0xFFFF_FFFFi64) | (((cx.lpp(12).bits()) as u32 as i64) << 32);
            { let p = cx.lpp(12).add(4924); let t = 1; cx.set_i(p, t); }
        } else {
            v1 = (v1 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.lpp(12); let q1_ = 1067450368; let q2_ = 1084227584; f_1d1190(cx, q0_, q1_, q2_) }.bits()) as i32 as u32 as i64);
            { let t = 1090519040; cx.set_li(40, t); }
            { let p = cx.lpp(12).add(4924); let t = 3; cx.set_i(p, t); }
        }
    } else {
        { let q0_ = cx.lpp(12); let q1_ = 1066192077; let q2_ = 1077936128; f_1d1190(cx, q0_, q1_, q2_) };
        { let t = 1082130432; cx.set_li(40, t); }
        v1 = (v1 & !0xFFFF_FFFFi64) | ((cx.lpp(12).bits()) as i32 as u32 as i64);
        { let p = cx.lpp(12).add(4924); let t = 2; cx.set_i(p, t); }
    }
    if (cx.f(cx.lptr(40)) > 0.0f32) {
        v8 = cx.f(cx.lptr(40));
        v2 = { let q0_ = cx.lpp(12); f_1c66c0(cx, q0_) };
        v9 = { let q0_ = v2; let q1_ = cx.lptr(0); let q2_ = v8; ov_vec4_mul(cx, q0_, q1_, q2_) };
        v3 = { let q0_ = cx.lpp(12); f_1c6480(cx, q0_) };
        { let q0_ = v3; let q1_ = cx.lptr(20); let q2_ = v9; ov_vec_add(cx, q0_, q1_, q2_) };
        v10 = { let q0_ = cx.lptr(20); ov_vec4_z(cx, q0_) };
        v6 = { let q0_ = cx.lptr(20); ov_vec4_y(cx, q0_) };
        v5 = { let q0_ = cx.lptr(20); ov_vec4_x(cx, q0_) };
        { let q0_ = cx.lpp(12).add(240); let q1_ = (v5.to_bits() as i32); let q2_ = (v6.to_bits() as i32); let q3_ = (v10.to_bits() as i32); f_36b530(cx, q0_, q1_, q2_, q3_) };
        v1 = (v1 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.lpp(12).add(240); vcall(cx, 32, q0_, &[]) }.i()) as i32 as u32 as i64);
    }
    v7 = v1;
    return v7;
}

/// VehicleEntity__1D1610 0x001D1610
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d1610(cx: &mut Cx, this: Ptr, a2: i32, a3: f32) -> i32 {
    { let p = this.add(5008); let t = a2; cx.set_i(p, t); }
    { let p = this.add(5012); let t = a2; cx.set_i(p, t); }
    { let p = this.add(5016); let t = ov_race_time(cx, Ptr::glob(0x6F5BC8)); cx.set_f(p, t); }
    { let p = this.add(5020); let t = (((cx.f(this.add(5016)) as f64) + (a3 as f64)) as f32); cx.set_f(p, t); }
    return this.bits();
}

/// VehicleEntity__1D1680 0x001D1680
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d1680(cx: &mut Cx, this: Ptr, a2: f32, a3: i32) -> i32 {
    let mut v4 = 0.0f32;
    let mut v5 = Ptr::NULL;
    let mut v6 = 0i32;
    v5 = Ptr::raw({ crate::ov_ignore!(cx, a3); 0 });
    v6 = f_1d1770(cx, this);
    if (cx.b(this.add(56)) != 0) {
        if (v6 != 0) {
            { let p = this.add(60); let t = (((cx.f(this.add(60)) as f64) + (a2 as f64)) as f32); cx.set_f(p, t); }
            v4 = (((cx.f(this.add(60)) as f64) / (cx.f(Ptr::glob(0x688A9C)) as f64)) as f32);
            if (v4 > 1.0f32) {
                v4 = 1.0f32;
            }
            crate::ov_ignore!(cx, v5, 6, -1.0f32, (v4.to_bits() as i32));
        } else {
            { let p = this.add(60); let t = 0.0f32; cx.set_f(p, t); }
        }
    } else if (v6 != 0) {
        { let p = this.add(60); let t = 0.0f32; cx.set_f(p, t); }
        crate::ov_ignore!(cx, v5, 6, 0);
    }
    { let p = this.add(56); let t = v6; cx.set_b(p, t); }
    return v6;
}

/// VehicleEntity__1D1770 0x001D1770
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d1770(cx: &mut Cx, this: Ptr) -> i32 {
    let mut i = 0i32;
    i = 0;
    'lp2: loop {
        if !(i < 4) { break 'lp2; }
        'blp2: {
            if ((((cx.b(this.add(i).add(24)) == 0) && (f_364a90(cx, this.add(240), i) == 1)) && (f_24220(cx, this) > 1.0f32)) && (ov_wheel_on_ground(cx, this, i) != 0)) {
                return 1;
            }
        }
        { i += 1; i };
    }
    return 0;
}

/// VehicleEntity__1D1850 0x001D1850
pub fn f_1d1850(cx: &mut Cx, this: Ptr) -> i64 {
    cx.push_frame(992);
    let r = f_1d1850_body(cx, this);
    cx.pop_frame();
    r
}

#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
fn f_1d1850_body(cx: &mut Cx, this: Ptr) -> i64 {
    let mut x = Ptr::NULL;
    let mut v1 = 0.0f64;
    let mut v2 = Ptr::NULL;
    let mut v3 = 0.0f64;
    let mut v4 = 0.0f64;
    let mut v5 = Ptr::NULL;
    let mut v6 = 0.0f64;
    let mut v7 = 0.0f64;
    let mut v8 = 0.0f64;
    let mut v9 = 0i32;
    let mut xa = Ptr::NULL;
    let mut xb = Ptr::NULL;
    let mut xc = 0.0f32;
    let mut xd = 0.0f32;
    let mut xe = 0.0f32;
    let mut xf = Ptr::NULL;
    let mut xg = 0.0f32;
    let mut xh = 0.0f32;
    let mut xi = 0i32;
    let mut xj = 0.0f32;
    let mut xk = 0.0f32;
    let mut xl = Ptr::NULL;
    let mut xm = 0.0f32;
    let mut xn = 0.0f32;
    let mut xo = 0.0f32;
    let mut xp = 0.0f32;
    let mut xq = 0i32;
    let mut xr = 0.0f32;
    let mut xs = 0.0f32;
    let mut xt = 0.0f32;
    let mut xu = 0i32;
    let mut xv = 0.0f32;
    let mut xw = 0i32;
    let mut v10 = 0i32;
    let mut v11 = Ptr::NULL;
    let mut v12 = 0.0f64;
    let mut v13 = 0.0f64;
    let mut v14 = 0i32;
    let mut v15 = 0.0f64;
    let mut v16 = 0.0f64;
    let mut v17 = Ptr::NULL;
    let mut v18 = Ptr::NULL;
    let mut v19 = 0.0f64;
    let mut v20 = Ptr::NULL;
    let mut v21 = Ptr::NULL;
    let mut v22 = Ptr::NULL;
    let mut v23 = Ptr::NULL;
    let mut v24 = Ptr::NULL;
    let mut v25 = Ptr::NULL;
    let mut v26 = Ptr::NULL;
    let mut v27 = Ptr::NULL;
    let mut v28 = 0.0f64;
    let mut v29 = Ptr::NULL;
    let mut v30 = 0.0f64;
    let mut v31 = Ptr::NULL;
    let mut v32 = Ptr::NULL;
    let mut v33 = 0.0f64;
    let mut v34 = 0.0f64;
    let mut v35 = 0.0f64;
    let mut v36 = 0.0f64;
    let mut v37 = 0.0f64;
    let mut v38 = 0.0f64;
    let mut v39 = 0i32;
    let mut v41 = 0i32;
    let mut v42 = 0i32;
    let mut v43 = 0.0f64;
    let mut v44 = 0.0f64;
    let mut v45 = 0.0f32;
    let mut v46 = 0i32;
    let mut v47 = Ptr::NULL;
    let mut v48 = 0.0f64;
    let mut v49 = Ptr::NULL;
    let mut v50 = 0.0f64;
    let mut v51 = 0.0f64;
    let mut v52 = Ptr::NULL;
    let mut v53 = Ptr::NULL;
    let mut v57 = 0.0f64;
    let mut v58 = 0.0f64;
    let mut v59 = 0.0f64;
    let mut v60 = 0.0f64;
    let mut v61 = 0.0f64;
    let mut v62 = 0.0f64;
    let mut v63 = 0.0f64;
    let mut v64 = 0.0f64;
    let mut v65 = 0.0f64;
    let mut v66 = 0.0f64;
    let mut v67 = 0.0f64;
    let mut v68 = 0.0f64;
    let mut v69 = Ptr::NULL;
    let mut v70 = Ptr::NULL;
    let mut v71 = 0.0f64;
    let mut v72 = 0.0f64;
    let mut v73 = 0.0f64;
    let mut v74 = 0.0f64;
    let mut v75 = 0.0f64;
    let mut v76 = 0.0f64;
    let mut v77 = 0.0f64;
    let mut v78 = 0.0f64;
    let mut v79 = 0.0f64;
    let mut v81 = Ptr::NULL;
    let mut v82 = 0.0f64;
    let mut v83 = 0.0f64;
    let mut v84 = 0.0f64;
    let mut v85 = 0.0f64;
    let mut v86 = 0.0f64;
    let mut v87 = 0.0f64;
    let mut v88 = Ptr::NULL;
    let mut v89 = Ptr::NULL;
    let mut v90 = Ptr::NULL;
    let mut v91 = 0.0f64;
    let mut v92 = 0.0f64;
    let mut v93 = 0.0f64;
    let mut v94 = 0i64;
    let mut v96 = 0.0f32;
    let mut v97 = Ptr::NULL;
    let mut v98 = 0.0f32;
    let mut v99 = 0.0f32;
    let mut v100 = 0.0f32;
    let mut v101 = 0.0f32;
    let mut v102 = 0.0f32;
    let mut v103 = 0i32;
    let mut v104 = 0i64;
    let mut _1B74A0 = 0i32;
    let mut Instance = Ptr::NULL;
    let mut RaceManager = Ptr::NULL;
    let mut VehicleById = Ptr::NULL;
    let mut car_angular_damping_after_landings = 0.0f32;
    { let t = this; cx.set_lpp(247, t); }
    { let t = ov_hist_entry(cx, this.add(3560), 0); cx.set_lpp(246, t); }
    { let t = { let q0_ = cx.lpp(247).add(3560); let q1_ = 1; ov_hist_entry(cx, q0_, q1_) }; cx.set_lpp(245, t); }
    v1 = vsmath::fabs(cx.f(cx.lpp(246).add(68)));
    { let p = cx.lpp(246).add(68); let t = ((((v1 * (cx.f(cx.lpp(246).add(68)) as f64)) * (cx.gc.quadratic_steer_mapping as f64)) + ((1.0f64 - (cx.gc.quadratic_steer_mapping as f64)) * (cx.f(cx.lpp(246).add(68)) as f64))) as f32); cx.set_f(p, t); }
    x = { let q0_ = cx.lpp(247).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
    v2 = { let q0_ = cx.lpp(247).add(240); ov_car_velocity(cx, q0_) };
    v3 = f_1c0300(cx, v2, x);
    { let p = cx.lpp(247).add(3704); let t = (v3 as f32); cx.set_f(p, t); }
    v4 = vsmath::fabs(cx.f(cx.lpp(247).add(3704)));
    { let p = cx.lpp(247).add(3708); let t = (v4 as f32); cx.set_f(p, t); }
    { let p = cx.lpp(247).add(3712); let t = (((cx.f(cx.lpp(247).add(3708)) as f64) * (cx.f(cx.ptr_at(cx.lpp(247).add(3552))) as f64)) as f32); cx.set_f(p, t); }
    xa = { let q0_ = cx.lpp(247).add(240); let q1_ = 2; ov_car_rot_row(cx, q0_, q1_) };
    v5 = { let q0_ = cx.lpp(247).add(240); ov_car_velocity(cx, q0_) };
    v6 = f_1c0300(cx, v5, xa);
    { let p = cx.lpp(246).add(88); let t = (v6 as f32); cx.set_f(p, t); }
    v7 = vsmath::fabs(cx.f(cx.lpp(246).add(88)));
    { let p = cx.lpp(247).add(3748); let t = (v7 as f32); cx.set_f(p, t); }
    { let p = cx.lpp(247).add(3752); let t = (((cx.f(cx.lpp(247).add(3748)) as f64) * (cx.f(cx.ptr_at(cx.lpp(247).add(3552))) as f64)) as f32); cx.set_f(p, t); }
    v8 = vsmath::sqrt(((((cx.f(cx.lpp(247).add(3704)) * cx.f(cx.lpp(247).add(3704))) as f64) + ((cx.f(cx.lpp(246).add(88)) * cx.f(cx.lpp(246).add(88))) as f64)) as f32));
    { let p = cx.lpp(247).add(3784); let t = (v8 as f32); cx.set_f(p, t); }
    v9 = { let q0_ = cx.lpp(247).add(240); f_364b20(cx, q0_) };
    { let p = cx.lpp(246).add(56); let t = v9; cx.set_i(p, t); }
    v10 = { let q0_ = cx.lpp(247).add(240); f_364be0(cx, q0_) };
    { let p = cx.lpp(247).add(3604); let t = v10; cx.set_i(p, t); }
    { let t = ((({ let q0_ = cx.lpp(247).add(240); ov_car_i2876(cx, q0_) } != 0) && (cx.i(cx.lpp(247).add(4856)) > 1)) as i32); cx.set_li(36, t); }
    { let p = cx.lpp(246).add(92); let t = cx.lf(36); cx.set_f(p, t); }
    xb = { let q0_ = cx.lpp(247).add(240); let q1_ = 1; ov_car_rot_row(cx, q0_, q1_) };
    v11 = { let q0_ = cx.lpp(247).add(240); ov_car_omega(cx, q0_) };
    v12 = f_1c0300(cx, v11, xb);
    { let p = cx.lpp(247).add(3760); let t = (v12 as f32); cx.set_f(p, t); }
    v13 = vsmath::fabs(cx.f(cx.lpp(247).add(3760)));
    { let p = cx.lpp(247).add(3764); let t = (v13 as f32); cx.set_f(p, t); }
    v14 = { let q0_ = cx.lpp(247).add(240); f_35dd20(cx, q0_) };
    { let p = cx.lpp(247).add(3688); let t = v14; cx.set_b(p, t); }
    if (cx.i(cx.lpp(247).add(3604)) == 4) {
        { let p = cx.lpp(247).add(4852); let t = 0; cx.set_i(p, t); }
    }
    if (cx.gc.max_surface_velocity_to_guess_from_accelerator_only <= cx.f(cx.lpp(247).add(3784))) {
        if (cx.gc.max_lateral_velocity_for_direction_switch > cx.f(cx.lpp(247).add(3748))) {
            { let p = cx.lpp(246).add(44); let t = ((cx.f(cx.lpp(247).add(3704)) > 0.0f32) as i32); cx.set_i(p, t); }
        }
    } else if (cx.f(cx.lpp(246).add(36)) == 0.0f32) {
        if (cx.f(cx.lpp(246).add(164)) != 0.0f32) {
            { let p = cx.lpp(246).add(44); let t = 0; cx.set_i(p, t); }
        }
    } else {
        { let p = cx.lpp(246).add(44); let t = (((cx.f(cx.lpp(246).add(164)) == 0.0f32) || (cx.f(cx.lpp(247).add(3704)) > 0.0f32)) as i32); cx.set_i(p, t); }
    }
    if (cx.gc.AIS_max_accelerator_release_duration != 0.0f32) {
        if ((((((((cx.f(cx.lpp(246).add(72)) <= cx.gc.AIS_hard_steering_min_value) || (cx.f(cx.lpp(247).add(3784)) <= cx.gc.AIS_min_velocity)) || (cx.i(cx.lpp(246).add(44)) == 0)) || (cx.i(cx.lpp(247).add(3740)) != 0)) || (cx.i(cx.lpp(246).add(76)) != 0)) || (cx.i(cx.lpp(246).add(12)) != 0)) || (cx.i(cx.lpp(246).add(64)) != 0)) || (cx.i(cx.lpp(246).add(92)) != 0)) {
            { let p = cx.lpp(246).add(152); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(144); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(148); let t = 0; cx.set_i(p, t); }
        } else {
            if (cx.f(cx.lpp(246).add(36)) <= cx.gc.AIS_hard_accelerator_min_value) {
                { let p = cx.lpp(246).add(144); let t = 0; cx.set_i(p, t); }
            } else {
                let _ = cx.preinc_i(cx.lpp(246).add(144));
            }
            if (vsmath::fabs(cx.f(cx.lpp(246).add(164))) <= (cx.gc.AIS_hard_accelerator_min_value as f64)) {
                { let p = cx.lpp(246).add(148); let t = 0; cx.set_i(p, t); }
            } else {
                let _ = cx.preinc_i(cx.lpp(246).add(148));
            }
            let _ = cx.preinc_i(cx.lpp(246).add(152));
        }
        if ((cx.i(cx.lpp(246).add(144)) < cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(204))) || (cx.i(cx.lpp(247).add(3828)) != 0)) {
            if (cx.i(cx.lpp(246).add(148)) < cx.gc.AIS_time_to_steer_and_footbrake_for_skid) {
                if (cx.i(cx.lpp(246).add(152)) >= cx.gc.AIS_time_to_steer_only_for_skid) {
                    { let p = cx.lpp(246).add(144); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(148); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(152); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(140); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(132); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(136); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(156); let t = cx.gc.AIS_time_to_hold_handbrake_after_steer_only; cx.set_i(p, t); }
                    { let p = cx.lpp(247).add(3812); let t = 1065353216; cx.set_i(p, t); }
                }
            } else {
                { let p = cx.lpp(246).add(144); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(148); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(152); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(140); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(132); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(136); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(156); let t = cx.gc.AIS_time_to_hold_handbrake_after_steer_and_footbrake; cx.set_i(p, t); }
                { let p = cx.lpp(247).add(3812); let t = 1065353216; cx.set_i(p, t); }
            }
        } else {
            { let p = cx.lpp(246).add(144); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(148); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(152); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(140); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(132); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(136); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(156); let t = cx.gc.AIS_time_to_hold_handbrake_after_steer_and_accelerator; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3812); let t = cx.gc.AIS_kick_multiplier_steer_and_accelerator; cx.set_f(p, t); }
            if (cx.i(cx.lpp(247).add(1360)) == 0) {
                { let p = cx.lpp(247).add(1360); let t = cx.gc.AIS_force_DF_duration; cx.set_i(p, t); }
            }
        }
        if (cx.f(cx.lpp(246).add(36)) > cx.gc.AIS_hard_accelerator_min_value) {
            if (cx.i(cx.lpp(246).add(140)) != 0) {
                if (cx.f(cx.lpp(246).add(72)) <= cx.gc.AIS_hard_steering_min_value) {
                    { let p = cx.lpp(246).add(140); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(132); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(136); let t = 0; cx.set_i(p, t); }
                } else {
                    { let p = cx.lpp(246).add(144); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(148); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(152); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(140); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(132); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(136); let t = 0; cx.set_i(p, t); }
                    { let p = cx.lpp(246).add(156); let t = cx.gc.AIS_time_to_hold_handbrake_after_reapplying_accelerator; cx.set_i(p, t); }
                    crate::ov_ignore!(cx, 1, "Started accelerator initiated skid\n");
                }
            } else {
                { let p = cx.lpp(246).add(132); let t = 1; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(136); let t = cx.i(cx.lpp(246).add(68)); cx.set_i(p, t); }
            }
        }
        if ((cx.i(cx.lpp(246).add(132)) as f32) > cx.gc.AIS_max_accelerator_release_duration) {
            { let p = cx.lpp(246).add(140); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(132); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(136); let t = 0; cx.set_i(p, t); }
        }
        if (((cx.gc.AIS_no_accelerator_max_value > cx.f(cx.lpp(246).add(36))) && (cx.i(cx.lpp(246).add(132)) != 0)) && (cx.gc.AIS_max_accelerator_release_duration > (cx.i(cx.lpp(246).add(132)) as f32))) {
            { let p = cx.lpp(246).add(140); let t = 1; cx.set_i(p, t); }
        }
        if (cx.i(cx.lpp(246).add(132)) != 0) {
            let _ = cx.preinc_i(cx.lpp(246).add(132));
        }
        if ((cx.i(cx.lpp(246).add(156)) != 0) && (cx.b(cx.lpp(247).add(16)) == 0)) {
            { let p = cx.lpp(246).add(48); let t = 1065353216; cx.set_i(p, t); }
            if (cx.predec_i(cx.lpp(246).add(156)) == 0) {
                { let p = cx.lpp(246).add(140); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(132); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(136); let t = 0; cx.set_i(p, t); }
            }
        }
    }
    if (cx.i(cx.lpp(246).add(56)) == 1) {
        if (cx.f(cx.lpp(247).add(3704)) == 0.0f32) {
            if (cx.f(cx.lpp(246).add(88)) <= 0.0f32) {
                { let t = ((0.0f64 - 3.4028235e38f64) as f32); cx.set_lf(32, t); }
            } else {
                { let t = 3.4028235e38f32; cx.set_lf(32, t); }
            }
            { let p = cx.lpp(247).add(3788); let t = cx.lf(32); cx.set_f(p, t); }
        } else {
            { let p = cx.lpp(247).add(3788); let t = (((cx.f(cx.lpp(246).add(88)) as f64) / (cx.f(cx.lpp(247).add(3704)) as f64)) as f32); cx.set_f(p, t); }
        }
        if (cx.f(cx.lpp(247).add(3704)) < 0.0f32) {
            { let p = cx.lpp(247).add(3788); let t = ((0.0f64 - (cx.f(cx.lpp(247).add(3788)) as f64)) as f32); cx.set_f(p, t); }
            if (cx.f(cx.lpp(247).add(3788)) <= 0.0f32) {
                { let p = cx.lpp(247).add(3788); let t = ((0.0f64 - 3.4028235e38f64) as f32); cx.set_f(p, t); }
            } else {
                { let p = cx.lpp(247).add(3788); let t = 2139095039; cx.set_i(p, t); }
            }
        }
        v15 = vsmath::atan2(cx.f(cx.lpp(246).add(88)), cx.f(cx.lpp(247).add(3704)));
        { let p = cx.lpp(247).add(3772); let t = (v15 as f32); cx.set_f(p, t); }
    } else {
        { let p = cx.lpp(247).add(3788); let t = 0; cx.set_i(p, t); }
        { let p = cx.lpp(247).add(3772); let t = 0; cx.set_i(p, t); }
    }
    v16 = vsmath::fabs(cx.f(cx.lpp(247).add(3788)));
    { let p = cx.lpp(247).add(3792); let t = (v16 as f32); cx.set_f(p, t); }
    if ((cx.f(cx.lpp(247).add(3788)) * cx.f(cx.lpp(246).add(68))) <= 0.0f32) {
        if ((cx.f(cx.lpp(247).add(3788)) * cx.f(cx.lpp(246).add(68))) >= 0.0f32) {
            if (cx.f(cx.lpp(246).add(68)) != 0.0f32) {
                { let p = cx.lpp(247).add(3776); let t = -1; cx.set_i(p, t); }
            }
        } else {
            { let p = cx.lpp(247).add(3776); let t = -1; cx.set_i(p, t); }
        }
    } else {
        { let p = cx.lpp(247).add(3776); let t = 1; cx.set_i(p, t); }
    }
    if (cx.i(cx.lpp(247).add(3740)) != 0) {
        let _ = cx.preinc_i(cx.lpp(247).add(3740));
        let _ = cx.preinc_i(cx.lpp(247).add(3656));
        if (((cx.f(cx.lpp(246).add(48)) != 0.0f32) && (cx.f(cx.lpp(245).add(48)) == 0.0f32)) || ((cx.f(cx.lpp(246).add(48)) == 0.0f32) && (cx.f(cx.lpp(245).add(48)) != 0.0f32))) {
            { let p = cx.lpp(246).add(52); let t = (cx.preinc_i(cx.lpp(247).add(3736)) / 2); cx.set_i(p, t); }
            if ((cx.f(cx.lpp(246).add(48)) != 0.0f32) && (cx.i(cx.lpp(246).add(52)) != cx.i(cx.lpp(245).add(52)))) {
                { let p = cx.lpp(246).add(176); let t = cx.i(cx.lpp(247).add(3780)); cx.set_i(p, t); }
            }
        }
        if (((cx.f(cx.lpp(246).add(36)) != 0.0f32) && (cx.f(cx.lpp(245).add(36)) == 0.0f32)) || ((cx.f(cx.lpp(246).add(36)) == 0.0f32) && (cx.f(cx.lpp(245).add(36)) != 0.0f32))) {
            { let p = cx.lpp(247).add(3628); let t = (cx.preinc_i(cx.lpp(247).add(3716)) / 2); cx.set_i(p, t); }
        }
    }
    if (cx.f(cx.lpp(246).add(48)) == 0.0f32) {
        { let p = cx.lpp(247).add(3652); let t = 0; cx.set_i(p, t); }
    } else {
        let _ = cx.preinc_i(cx.lpp(247).add(3652));
    }
    if (cx.gc.max_time_on_2_wheels != 0) {
        if ((((cx.i(cx.lpp(247).add(3796)) > cx.gc.min_time_between_2_wheels) && (cx.i(cx.lpp(246).add(64)) == 0)) && ((((({ let q0_ = cx.lpp(247).add(240); let q1_ = 0; f_364a90(cx, q0_, q1_) } == -1) && ({ let q0_ = cx.lpp(247).add(240); let q1_ = 1; f_364a90(cx, q0_, q1_) } == -1)) && ({ let q0_ = cx.lpp(247).add(240); let q1_ = 3; f_364a90(cx, q0_, q1_) } != -1)) && ({ let q0_ = cx.lpp(247).add(240); let q1_ = 2; f_364a90(cx, q0_, q1_) } != -1)) || (((({ let q0_ = cx.lpp(247).add(240); let q1_ = 3; f_364a90(cx, q0_, q1_) } == -1) && ({ let q0_ = cx.lpp(247).add(240); let q1_ = 2; f_364a90(cx, q0_, q1_) } == -1)) && ({ let q0_ = cx.lpp(247).add(240); let q1_ = 0; f_364a90(cx, q0_, q1_) } != -1)) && ({ let q0_ = cx.lpp(247).add(240); let q1_ = 1; f_364a90(cx, q0_, q1_) } != -1)))) && (cx.gc.max_2_wheel_forward_speed_to_start > cx.f(cx.lpp(247).add(3704)))) {
            { let p = cx.lpp(247).add(3132); let t = cx.gc.max_balancing_angle_to_start; cx.set_f(p, t); }
            { let p = cx.lpp(247).add(3080); let t = 1; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3124); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(64); let t = 1; cx.set_i(p, t); }
        }
        if ((cx.i(cx.lpp(246).add(64)) != 0) && (((((((((({ let q0_ = cx.lpp(247).add(240); let q1_ = 0; f_364a90(cx, q0_, q1_) } != -1) && ({ let q0_ = cx.lpp(247).add(240); let q1_ = 1; f_364a90(cx, q0_, q1_) } != -1)) && ({ let q0_ = cx.lpp(247).add(240); let q1_ = 3; f_364a90(cx, q0_, q1_) } != -1)) && ({ let q0_ = cx.lpp(247).add(240); let q1_ = 2; f_364a90(cx, q0_, q1_) } != -1)) || (cx.i(cx.lpp(247).add(3796)) < cx.gc.max_time_on_2_wheels.wrapping_neg())) || (cx.i(cx.lpp(247).add(3604)) == 0)) || (cx.i(cx.lpp(247).add(3124)) != 0)) || (cx.i(cx.lpp(247).add(4756)) == 1)) || (cx.gc.min_2_wheel_forward_speed > cx.f(cx.lpp(247).add(3704)))) || (((cx.i(cx.lpp(247).add(4856)) == 2) && (cx.i(cx.lpp(245).add(64)) != 0)) && ({ let p = { let q0_ = cx.lpp(247).add(3560); let q1_ = 2; ov_hist_entry(cx, q0_, q1_) }.add(64); cx.i(p) } != 0)))) {
            { let p = cx.lpp(247).add(3080); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3124); let t = 1; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(64); let t = 0; cx.set_i(p, t); }
        }
        if (cx.i(cx.lpp(246).add(64)) != 0) {
            { let p = cx.lpp(247).add(3648); let t = cx.gc.boost_to_steering_on_2_wheels; cx.set_f(p, t); }
            { let p = cx.lpp(247).add(3220); let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(264)) as f64) * (cx.gc.boost_to_engine_power_on_2_wheels as f64)) as f32); cx.set_f(p, t); }
            if (cx.i(cx.lpp(247).add(3796)) <= 0) {
                let _ = cx.predec_i(cx.lpp(247).add(3796));
            } else {
                { let p = cx.lpp(247).add(3796); let t = 0; cx.set_i(p, t); }
            }
            if (cx.i(cx.lpp(247).add(3796)) <= cx.gc.max_balancing_omega_change_interpolation_time.wrapping_neg()) {
                { let p = cx.lpp(247).add(3128); let t = cx.gc.max_balancing_omega_change_end; cx.set_f(p, t); }
                { let p = cx.lpp(247).add(3132); let t = 1078530011; cx.set_i(p, t); }
            } else {
                { let p = cx.lpp(247).add(3128); let t = (((cx.gc.max_balancing_omega_change_start as f64) - ((((cx.gc.max_balancing_omega_change_end - cx.gc.max_balancing_omega_change_start) * (cx.i(cx.lpp(247).add(3796)) as f32)) / (cx.gc.max_balancing_omega_change_interpolation_time as f32)) as f64)) as f32); cx.set_f(p, t); }
            }
        } else {
            { let p = cx.lpp(247).add(3648); let t = 1065353216; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3220); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(264)); cx.set_i(p, t); }
            if (cx.i(cx.lpp(247).add(3796)) >= 0) {
                let _ = cx.preinc_i(cx.lpp(247).add(3796));
            } else {
                { let p = cx.lpp(247).add(3796); let t = 0; cx.set_i(p, t); }
            }
        }
    } else {
        { let p = cx.lpp(247).add(3648); let t = 1065353216; cx.set_i(p, t); }
    }
    { let q0_ = cx.lpp(247); f_1cbde0(cx, q0_) };
    { let q0_ = cx.lpp(247); f_1cc3f0(cx, q0_) };
    if ((((((((cx.i(cx.lpp(247).add(3740)) == 0) && (cx.b(cx.lpp(247).add(16)) == 0)) && (cx.i(cx.lpp(247).add(4756)) != 1)) && (cx.i(cx.lpp(247).add(3652)) < cx.gc.normal_handbrake_time)) && (cx.f(cx.lpp(246).add(48)) != 0.0f32)) && (cx.i(cx.lpp(247).add(3736)) == 0)) && (cx.f(cx.lpp(246).add(72)) > cx.gc.max_steering_input_to_use_normal_handbrake)) && (cx.f(cx.lpp(247).add(3704)) > cx.gc.min_forward_velocity_to_kick)) {
        { let p = cx.lpp(247).add(3768); let t = 0; cx.set_i(p, t); }
        if (cx.f(cx.lpp(246).add(68)) <= cx.gc.min_steer_input_to_kick) {
            if ((0.0f32 - cx.gc.min_steer_input_to_kick) > cx.f(cx.lpp(246).add(68))) {
                { let p = cx.lpp(247).add(3768); let t = 1; cx.set_i(p, t); }
            }
        } else {
            { let p = cx.lpp(247).add(3768); let t = -1; cx.set_i(p, t); }
        }
        if (cx.i(cx.lpp(247).add(3768)) != 0) {
            { let p = cx.lpp(247).add(3676); let t = cx.gc.max_entering_skidding_time__frames; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3740); let t = 1; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3656); let t = 1; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3680); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3732); let t = cx.gc.handbrake_disabled_after_kick__frames; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3756); let t = 0; cx.set_i(p, t); }
            v17 = { let q0_ = cx.lpp(247).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
            { let q0_ = cx.lpp(247).add(3664); let q1_ = v17; f_1bb190(cx, q0_, q1_) };
        }
    }
    if (cx.i(cx.lpp(247).add(3676)) != 0) {
        let _ = cx.preinc_i(cx.lpp(247).add(3680));
        if (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(200)) <= cx.f(cx.lpp(247).add(3792))) || (((cx.i(cx.lpp(247).add(3768)) as f32) * cx.f(cx.lpp(246).add(68))) > 0.0f32)) || ((cx.i(cx.lpp(246).add(52)) != 0) && (cx.i(cx.lpp(246).add(56)) != 0))) {
            { let p = cx.lpp(247).add(3676); let t = 0; cx.set_i(p, t); }
        } else if (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(196)) != 0.0f32) {
            { let q0_ = cx.lpp(247).add(240).bits(); let q1_ = cx.lpp(247).bits(); let q2_ = ((((cx.f(cx.lpp(247).add(3812)) * cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(196))) as f64) * ((cx.i(cx.lpp(247).add(3768)) as f32) as f64)) as f32); let q3_ = 1; f_36ad50(cx, q0_, q1_, q2_, q3_) };
            { let _ = { let q0_ = cx.lpp(247).add(240); vcall(cx, 32, q0_, &[]) }; };
        }
    }
    if (cx.i(cx.lpp(247).add(3676)) == 0) {
        { let p = cx.lpp(247).add(3640); let t = 1065353216; cx.set_i(p, t); }
        { let p = cx.lpp(247).add(3680); let t = 0; cx.set_i(p, t); }
        { let p = cx.lpp(247).add(3812); let t = 1065353216; cx.set_i(p, t); }
        if (cx.i(cx.lpp(246).add(56)) == 0) {
            { let p = cx.lpp(247).add(3756); let t = 1; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3684); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3636); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(80); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(84); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3740); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3656); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3736); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3716); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(52); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3628); let t = 0; cx.set_i(p, t); }
        }
    }
    if ((cx.i(cx.lpp(247).add(3740)) != 0) && (cx.i(cx.lpp(247).add(3652)) <= cx.gc.normal_handbrake_time)) {
        { let p = cx.lpp(247).add(3684); let t = 1; cx.set_i(p, t); }
        if ((cx.i(cx.lpp(247).add(3676)) != 0) && (cx.f(cx.lpp(246).add(48)) == 0.0f32)) {
            { let p = cx.lpp(247).add(3636); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(80); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(246).add(84); let t = 0; cx.set_i(p, t); }
        } else if (cx.f(cx.lpp(246).add(48)) != 0.0f32) {
            if (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(352)) < (cx.i(cx.lpp(247).add(3652)) as f32)) {
                if (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(380)) <= (cx.i(cx.lpp(247).add(3652)) as f32)) {
                    { let p = cx.lpp(247).add(3636); let t = 1065353216; cx.set_i(p, t); }
                } else {
                    { let p = cx.lpp(247).add(3636); let t = (((((cx.i(cx.lpp(247).add(3652)) as f32) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(352))) as f64) / (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(356)) as f64)) as f32); cx.set_f(p, t); }
                }
                { let p = cx.lpp(246).add(80); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(84); let t = 1; cx.set_i(p, t); }
            } else {
                if (cx.i(cx.lpp(246).add(52)) != 0) {
                    { let t = cx.i(cx.lpp(246).add(52)).wrapping_add(1); cx.set_li(28, t); }
                } else {
                    { let t = 0.0f32; cx.set_lf(28, t); }
                }
                { let t = cx.lf(28); cx.set_lf(976, t); }
                if (cx.li(28) >= cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(208))) {
                    { let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(208)).wrapping_sub(1); cx.set_li(976, t); }
                }
                if ((cx.f(cx.lpp(245).add(176)) != cx.f(cx.lpp(247).add(3780))) && (cx.li(976) > 0)) {
                    { let t = 0.0f32; cx.set_lf(976, t); }
                }
                { let p = cx.lpp(247).add(3636); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(80); let t = cx.lf(976); cx.set_f(p, t); }
                { let p = cx.lpp(246).add(84); let t = cx.lf(976); cx.set_f(p, t); }
            }
            if ((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(84)))).add(424)) > (cx.i(cx.lpp(247).add(3652)) as f32)) || ((cx.i(cx.lpp(246).add(52)) == 0) && ((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(380)) + cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(428))) > (cx.i(cx.lpp(247).add(3652)) as f32)))) {
                v18 = { let q0_ = cx.lpp(247).add(240); let q1_ = 1; ov_car_rot_row(cx, q0_, q1_) };
                { let t = ({ let q0_ = cx.lpp(247).add(288); let q1_ = v18; f_1c0300(cx, q0_, q1_) } as f32); cx.set_lf(972, t); }
                { let t = (((((((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(84)))).add(400)) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(80)))).add(400))) * cx.f(cx.lpp(247).add(3636))) + cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(80)))).add(400))) * cx.f(cx.lpp(247).add(3812))) as f64) * ((0.0f32 - cx.f(cx.lpp(246).add(68))) as f64)) as f32); cx.set_lf(968, t); }
                { let t = (((((cx.lf(968) - cx.lf(972)) * cx.gc.kick_omega_change_factor) as f64) + (cx.lf(972) as f64)) as f32); cx.set_lf(964, t); }
                if ((cx.i(cx.lpp(247).add(4756)) != 1) && (cx.i(cx.lpp(246).add(160)) == 0)) {
                    if (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(84)))).add(400)) <= 0.0f32) {
                        xd = (((((0.0f32 - cx.f(cx.lpp(247).add(3812))) * cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(84)))).add(400))) as f64) * ((0.0f32 - cx.f(cx.lpp(246).add(68))) as f64)) as f32);
                        v22 = { let q0_ = cx.lpp(247).add(240); let q1_ = 1; ov_car_rot_row(cx, q0_, q1_) };
                        { let q0_ = cx.lpp(247).add(288); let q1_ = v22; let q2_ = xd; f_1c01b0(cx, q0_, q1_, q2_) };
                    } else {
                        { let t = (vsmath::fabs(cx.lf(968)) as f32); cx.set_lf(24, t); }
                        v19 = vsmath::fabs(cx.lf(972));
                        if ((cx.lf(24) as f64) > v19) {
                            v20 = { let q0_ = cx.lpp(247).add(240); let q1_ = 1; ov_car_rot_row(cx, q0_, q1_) };
                            { let q0_ = cx.lpp(247).add(288); let q1_ = cx.lpp(247).add(288); let q2_ = v20; f_1c05a0(cx, q0_, q1_, q2_) };
                            xc = cx.lf(964);
                            v21 = { let q0_ = cx.lpp(247).add(240); let q1_ = 1; ov_car_rot_row(cx, q0_, q1_) };
                            { let q0_ = cx.lpp(247).add(288); let q1_ = v21; let q2_ = xc; f_1c01b0(cx, q0_, q1_, q2_) };
                        }
                    }
                }
            }
        }
    } else {
        { let p = cx.lpp(247).add(3684); let t = 0; cx.set_i(p, t); }
        { let p = cx.lpp(247).add(3636); let t = 0; cx.set_i(p, t); }
        { let p = cx.lpp(246).add(80); let t = 0; cx.set_i(p, t); }
        { let p = cx.lpp(246).add(84); let t = 0; cx.set_i(p, t); }
    }
    'l_LABEL_267: {
        if (cx.i(cx.lpp(247).add(3684)) == 0) {
            { let p = cx.lpp(247).add(3644); let t = 1065353216; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(1600); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(448)); cx.set_i(p, t); }
            { let p = cx.lpp(247).add(1604); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(472)); cx.set_i(p, t); }
            break 'l_LABEL_267;
        }
        if ((cx.i(cx.lpp(247).add(3684)) == 1) || (cx.i(cx.lpp(247).add(3684)) == 2)) {
            if (cx.i(cx.lpp(247).add(3776)) == 1) {
                { let t = (cx.i(cx.lpp(247).add(3652)) as f32); cx.set_lf(936, t); }
                { let t = { let q0_ = cx.lptr(956); let q1_ = 0; let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(80)))).add(328)); let q3_ = cx.lptr(936); ov_eval_expr(cx, q0_, q1_, q2_, q3_) }; cx.set_li(928, t); }
                if ((cx.li(928) != 0) || (cx.lf(956) == 0.0f32)) {
                    { let t = 0.0f32; cx.set_lf(948, t); }
                } else {
                    { let t = cx.lf(956); cx.set_lf(948, t); }
                }
                { let t = { let q0_ = cx.lptr(956); let q1_ = 0; let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(84)))).add(328)); let q3_ = cx.lptr(936); ov_eval_expr(cx, q0_, q1_, q2_, q3_) }; cx.set_li(928, t); }
                if ((cx.li(928) != 0) || (cx.lf(956) == 0.0f32)) {
                    { let t = 0.0f32; cx.set_lf(944, t); }
                } else {
                    { let t = cx.lf(956); cx.set_lf(944, t); }
                }
                { let p = cx.lpp(247).add(3644); let t = (((((cx.lf(944) - cx.lf(948)) * cx.f(cx.lpp(247).add(3636))) as f64) + (cx.lf(948) as f64)) as f32); cx.set_f(p, t); }
            } else if (cx.i(cx.lpp(247).add(3776)) == -1) {
                { let t = (cx.i(cx.lpp(247).add(3652)) as f32); cx.set_lf(900, t); }
                { let t = { let q0_ = cx.lptr(920); let q1_ = 0; let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(80)))).add(304)); let q3_ = cx.lptr(900); ov_eval_expr(cx, q0_, q1_, q2_, q3_) }; cx.set_li(892, t); }
                if ((cx.li(892) != 0) || (cx.lf(920) == 0.0f32)) {
                    { let t = 0.0f32; cx.set_lf(912, t); }
                } else {
                    { let t = cx.lf(920); cx.set_lf(912, t); }
                }
                { let t = { let q0_ = cx.lptr(920); let q1_ = 0; let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(84)))).add(304)); let q3_ = cx.lptr(900); ov_eval_expr(cx, q0_, q1_, q2_, q3_) }; cx.set_li(892, t); }
                if ((cx.li(892) != 0) || (cx.lf(920) == 0.0f32)) {
                    { let t = 0.0f32; cx.set_lf(908, t); }
                } else {
                    { let t = cx.lf(920); cx.set_lf(908, t); }
                }
                { let p = cx.lpp(247).add(3644); let t = (((((cx.lf(908) - cx.lf(912)) * cx.f(cx.lpp(247).add(3636))) as f64) + (cx.lf(912) as f64)) as f32); cx.set_f(p, t); }
            }
            { let t = (((((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(84)))).add(496)) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(80)))).add(496))) * cx.f(cx.lpp(247).add(3636))) as f64) + (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(80)))).add(496)) as f64)) as f32); cx.set_lf(888, t); }
            if (cx.f(cx.lpp(246).add(36)) == 0.0f32) {
                { let t = ((0.0f64 - (cx.lf(888) as f64)) as f32); cx.set_lf(888, t); }
            }
            if ((cx.b(cx.lpp(247).add(3816)) == 0) || (cx.f(cx.lpp(246).add(72)) > 0.2f32)) {
                if (cx.f(cx.lpp(247).add(3788)) <= cx.gc.tan_of_min_skid_angle_to_apply_bias) {
                    if ((0.0f32 - cx.gc.tan_of_min_skid_angle_to_apply_bias) > cx.f(cx.lpp(247).add(3788))) {
                        { let p = cx.lpp(246).add(68); let t = (((cx.f(cx.lpp(246).add(68)) as f64) - (cx.lf(888) as f64)) as f32); cx.set_f(p, t); }
                        { let q0_ = cx.lpp(246).add(68); let q1_ = -1.0f32; let q2_ = 1.0f32; f_1bfe80(cx, q0_, q1_, q2_) };
                    }
                } else {
                    { let p = cx.lpp(246).add(68); let t = (((cx.f(cx.lpp(246).add(68)) as f64) + (cx.lf(888) as f64)) as f32); cx.set_f(p, t); }
                    { let q0_ = cx.lpp(246).add(68); let q1_ = -1.0f32; let q2_ = 1.0f32; f_1bfe80(cx, q0_, q1_, q2_) };
                }
            }
            { let p = cx.lpp(247).add(1600); let t = (((((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(84)))).add(448)) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(80)))).add(448))) * cx.f(cx.lpp(247).add(3636))) as f64) + (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(80)))).add(448)) as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lpp(247).add(1604); let t = (((((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(84)))).add(472)) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(80)))).add(472))) * cx.f(cx.lpp(247).add(3636))) as f64) + (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add(80)))).add(472)) as f64)) as f32); cx.set_f(p, t); }
            cx.memset(cx.lptr(872), 0, 12);
            { let t = 0.0f32; cx.set_lf(860, t); }
            'lp18: loop {
                'blp18: {
                    if (cx.li(860) >= 2) {
                        { let q0_ = cx.lpp(247).add(240); let q1_ = cx.lptr(872); f_36ba70(cx, q0_, q1_) };
                        break 'lp18;
                    }
                    if (cx.li(860) != 0) {
                        if (cx.f(cx.lpp(247).add(3636)) == 0.0f32) {
                            break 'blp18;
                        }
                        { let t = cx.f(cx.lpp(247).add(3636)); cx.set_lf(856, t); }
                    } else {
                        if (cx.f(cx.lpp(247).add(3636)) == 1.0f32) {
                            break 'blp18;
                        }
                        { let t = ((1.0f64 - (cx.f(cx.lpp(247).add(3636)) as f64)) as f32); cx.set_lf(856, t); }
                    }
                    { let t = 0.0f32; cx.set_lf(864, t); }
                    'lp19: loop {
                        if !(cx.li(864) < cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add((4_i32).wrapping_mul(cx.i(cx.lpp(246).add((4_i32).wrapping_mul(cx.li(860))).add(80)))).add(616))) { break 'lp19; }
                        'blp19: {
                            { let t = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add((16_i32).wrapping_mul(cx.i(cx.lpp(246).add((4_i32).wrapping_mul(cx.li(860))).add(80)))).add(640).add((4_i32).wrapping_mul(cx.li(864)))); cx.set_lf(832, t); }
                            match cx.li(832) {
                                1 => {
                                    { let q0_ = cx.lpp(247).add(240); let q1_ = Ptr::NULL.bits(); let q2_ = cx.lptr(840); f_372330(cx, q0_, q1_, q2_) };
                                }
                                2 => {
                                    { let q0_ = cx.lpp(247).add(240); let q1_ = cx.lptr(840); let q2_ = 0; f_1da9c0(cx, q0_, q1_, q2_) };
                                }
                                3 => {
                                    v23 = { let q0_ = cx.lpp(247).add(240); ov_car_velocity(cx, q0_) };
                                    if ({ let q0_ = cx.lptr(840); let q1_ = v23; f_1c0410(cx, q0_, q1_) } == 0) {
                                        { let q0_ = cx.lptr(840); let q1_ = 1065353216; let q2_ = 0; let q3_ = 0; f_1bff80(cx, q0_, q1_, q2_, q3_) };
                                    }
                                }
                                4 => {
                                    v24 = { let q0_ = cx.lpp(247).add(240); ov_car_velocity(cx, q0_) };
                                    if ({ let q0_ = cx.lptr(816); let q1_ = v24; f_1c0410(cx, q0_, q1_) } != 0) {
                                        { let q0_ = cx.lpp(247).add(240); let q1_ = Ptr::NULL.bits(); let q2_ = cx.lptr(840); f_372330(cx, q0_, q1_, q2_) };
                                        { let q0_ = cx.lptr(840); let q1_ = cx.lptr(816); f_1c0000(cx, q0_, q1_) };
                                        { let q0_ = cx.lptr(840); f_1c0380(cx, q0_) };
                                    } else {
                                        { let q0_ = cx.lptr(840); let q1_ = 1065353216; let q2_ = 0; let q3_ = 0; f_1bff80(cx, q0_, q1_, q2_, q3_) };
                                    }
                                }
                                5 => {
                                    { let q0_ = cx.lpp(247).add(240); let q1_ = cx.lptr(796); f_1b42b0(cx, q0_, q1_) };
                                    { let t = 0.0f32; cx.set_lf(844, t); }
                                    { let t = cx.lf(804); cx.set_lf(840, t); }
                                    { let t = ((0.0f64 - (cx.lf(796) as f64)) as f32); cx.set_lf(848, t); }
                                    if ({ let q0_ = cx.lptr(840); let q1_ = cx.lptr(840); f_1c0410(cx, q0_, q1_) } == 0) {
                                        { let q0_ = cx.lptr(840); let q1_ = 1065353216; let q2_ = 0; let q3_ = 0; f_1bff80(cx, q0_, q1_, q2_, q3_) };
                                    }
                                    if (cx.f(cx.lpp(246).add(88)) < 0.0f32) {
                                        { let q0_ = cx.lptr(840); f_1cc9f0(cx, q0_) };
                                    }
                                }
                                6 => {
                                    { let q0_ = cx.lptr(840); let q1_ = cx.lpp(247).add(3664); f_1bffb0(cx, q0_, q1_) };
                                }
                                _ => {
                                }
                            }
                            if (cx.f(cx.lpp(246).add(36)) > 0.0f32) {
                                { let t = 1; cx.set_li(776, t); }
                            }
                            { let t = cx.f(cx.lpp(247).add(3772)); cx.set_lf(712, t); }
                            { let t = cx.f(cx.lpp(247).add(3704)); cx.set_lf(716, t); }
                            { let t = cx.f(cx.lpp(246).add(88)); cx.set_lf(720, t); }
                            { let t = (cx.i(cx.lpp(247).add(3740)) as f32); cx.set_lf(724, t); }
                            { let t = (cx.i(cx.lpp(247).add(3656)) as f32); cx.set_lf(728, t); }
                            { let t = cx.f(cx.lpp(246).add(36)); cx.set_lf(732, t); }
                            { let t = cx.f(cx.lpp(246).add(164)); cx.set_lf(736, t); }
                            { let t = cx.f(cx.lpp(246).add(48)); cx.set_lf(740, t); }
                            { let t = (cx.i(cx.lpp(246).add(52)) as f32); cx.set_lf(744, t); }
                            { let t = (cx.i(cx.lpp(247).add(3652)) as f32); cx.set_lf(748, t); }
                            { let t = (cx.i(cx.lpp(247).add(3628)) as f32); cx.set_lf(752, t); }
                            { let t = ((((0.0f32 - cx.gc.gravity[1 as usize]) as f64) * (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(12)) as f64)) as f32); cx.set_lf(756, t); }
                            { let t = (cx.i(cx.lpp(246).add(56)) as f32); cx.set_lf(760, t); }
                            v25 = { let q0_ = cx.lpp(247).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                            { let t = ({ let q0_ = cx.lpp(247).add(3664); let q1_ = v25; f_1c0300(cx, q0_, q1_) } as f32); cx.set_lf(764, t); }
                            { let t = (cx.i(cx.lpp(247).add(3604)) as f32); cx.set_lf(768, t); }
                            { let t = { let q0_ = cx.lptr(784); let q1_ = 0; let q2_ = cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3552)).add((16_i32).wrapping_mul(cx.i(cx.lpp(246).add((4_i32).wrapping_mul(cx.li(860))).add(80)))).add(520).add((4_i32).wrapping_mul(cx.li(864)))); let q3_ = cx.lptr(712); ov_eval_expr(cx, q0_, q1_, q2_, q3_) }; cx.set_li(704, t); }
                            if (cx.li(704) != 0) {
                                { let t = 0; let o = cx.li(40); cx.set_li(40, (o & !(255 << 24)) | ((t & 255) << 24)); }
                                { let t = 1; let o = cx.li(40); cx.set_li(40, (o & !(255 << 24)) | ((t & 255) << 24)); }
                                let _ = cx.preinc_i(cx.lptr(700));
                                crate::ov_ignore!(cx, 1, "  Could not compute the expression");
                            } else if (cx.lf(784) != 0.0f32) {
                                { let q0_ = cx.lptr(872); let q1_ = cx.lptr(840); let q2_ = (((cx.lf(784) as f64) * (cx.lf(856) as f64)) as f32); f_1c01b0(cx, q0_, q1_, q2_) };
                            }
                        }
                        let _ = cx.preinc_i(cx.lptr(864));
                    }
                }
                let _ = cx.preinc_i(cx.lptr(860));
            }
        }
    }
    if ((cx.i(cx.lpp(246).add(160)) == 4) && (cx.i(cx.lpp(245).add(160)) < 4)) {
        { let p = cx.lpp(247).add(292); let t = (((cx.f(cx.lpp(247).add(292)) as f64) * (cx.gc.friction_spot_omega_up_multiplier as f64)) as f32); cx.set_f(p, t); }
    }
    if (((cx.gc.NO_past_skidding_checks < cx.i(cx.lpp(247).add(3572))) && (cx.i(cx.lpp(246).add(44)) != 0)) && (cx.i(cx.lpp(246).add(160)) == 0)) {
        if (cx.i(cx.lpp(247).add(3804)) != 0) {
            if ((cx.f(cx.lpp(247).add(3808)) * cx.f(cx.lpp(247).add(3760))) >= 0.0f32) {
                let _ = cx.predec_i(cx.lpp(247).add(3804));
                { let t = (((((0.0f32 - cx.f(cx.lpp(247).add(3808))) * cx.gc.portion_of_omega_up_to_stop) as f64) / ((cx.gc.fish_tail_stop_duration as f32) as f64)) as f32); cx.set_lf(696, t); }
                if (((cx.f(cx.lpp(247).add(3760)) + cx.lf(696)) * cx.lf(696)) < 0.0f32) {
                    { let t = ((0.0f64 - (cx.f(cx.lpp(247).add(3760)) as f64)) as f32); cx.set_lf(696, t); }
                    { let p = cx.lpp(247).add(3804); let t = 0; cx.set_i(p, t); }
                }
                xe = cx.lf(696);
                v26 = { let q0_ = cx.lpp(247).add(240); let q1_ = 1; ov_car_rot_row(cx, q0_, q1_) };
                { let q0_ = cx.lpp(247).add(288); let q1_ = v26; let q2_ = xe; f_1c01b0(cx, q0_, q1_, q2_) };
            } else {
                { let p = cx.lpp(247).add(3804); let t = 0; cx.set_i(p, t); }
            }
        }
        if (cx.i(cx.lpp(247).add(3604)) >= 2) {
            if ((cx.i(cx.lpp(247).add(3744)) == 0) && (cx.f(cx.lpp(247).add(3792)) > cx.gc.tan_of_min_skid_angle_for_fish_tail_removal)) {
                { let p = cx.lpp(247).add(3744); let t = 1; cx.set_i(p, t); }
            }
            v27 = { let q0_ = cx.lpp(247).add(240); let q1_ = 1; ov_car_rot_row(cx, q0_, q1_) };
            v28 = { let q0_ = cx.lpp(247).add(288); let q1_ = v27; f_1c0300(cx, q0_, q1_) };
            { let t = ((((v28 * (cx.f(cx.ptr_at(cx.lpp(247).add(3552))) as f64)) * (cx.f(cx.lpp(247).add(3704)) as f64)) + (cx.f(cx.lpp(246).add(88)) as f64)) as f32); cx.set_lf(20, t); }
            xf = { let q0_ = cx.lpp(247).add(240); let q1_ = 2; ov_car_rot_row(cx, q0_, q1_) };
            v29 = { let q0_ = cx.lpp(247).add(240); f_36ccb0(cx, q0_) };
            v30 = f_1c0300(cx, v29, xf);
            { let t = (((v30 * (cx.f(cx.ptr_at(cx.lpp(247).add(3552))) as f64)) + (cx.lf(20) as f64)) as f32); cx.set_lf(692, t); }
            if ((((((cx.i(cx.lpp(247).add(3744)) != 0) && (cx.i(cx.lpp(247).add(3676)) == 0)) && ((cx.lf(692) * cx.f(cx.lpp(246).add(88))) < 0.0f32)) && (cx.f(cx.lpp(247).add(3704)) > cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(212)))) && (cx.f(cx.lpp(246).add(48)) == 0.0f32)) && (cx.gc.max_omega_up_to_stop > cx.f(cx.lpp(247).add(3764)))) {
                { let p = cx.lpp(247).add(3804); let t = cx.gc.fish_tail_stop_duration.wrapping_sub(1); cx.set_i(p, t); }
                { let p = cx.lpp(247).add(3808); let t = cx.i(cx.lpp(247).add(3760)); cx.set_i(p, t); }
                { let t = (((((0.0f32 - cx.f(cx.lpp(247).add(3808))) * cx.gc.portion_of_omega_up_to_stop) as f64) / ((cx.gc.fish_tail_stop_duration as f32) as f64)) as f32); cx.set_lf(688, t); }
                xg = cx.lf(688);
                v31 = { let q0_ = cx.lpp(247).add(240); let q1_ = 1; ov_car_rot_row(cx, q0_, q1_) };
                { let q0_ = cx.lpp(247).add(288); let q1_ = v31; let q2_ = xg; f_1c01b0(cx, q0_, q1_, q2_) };
                { let p = cx.lpp(247).add(3744); let t = 0; cx.set_i(p, t); }
                { let q0_ = cx.lpp(247).add(240); let q1_ = 0.0f32; f_361ff0(cx, q0_, q1_) };
                { let p = cx.lpp(246).add(68); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(247).add(3676); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(247).add(3684); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(247).add(3636); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(80); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(84); let t = 0; cx.set_i(p, t); }
                if (cx.gc.forward_speed_boost != 0.0f32) {
                    xh = (((cx.gc.forward_speed_boost as f64) * (cx.f(cx.lpp(246).add(36)) as f64)) as f32);
                    v32 = { let q0_ = cx.lpp(247).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                    { let q0_ = cx.lpp(247).add(272); let q1_ = v32; let q2_ = xh; f_1c01b0(cx, q0_, q1_, q2_) };
                }
                if (cx.b(cx.lpp(247).add(4072)) == 0) {
                    { let t = 0.0f32; cx.set_lf(684, t); }
                    'lp20: loop {
                        if !(cx.li(684) < 4) { break 'lp20; }
                        'blp20: {
                            xi = cx.li(684);
                            { let t = (-cx.f(cx.lpp(247).add(3704))); cx.set_lf(16, t); }
                            v33 = ({ let q0_ = cx.lpp(247).add(240); let q1_ = cx.li(684); f_35fb40(cx, q0_, q1_) } as f64);
                            v98 = (((cx.lf(16) as f64) / v33) as f32);
                            { let q0_ = cx.lpp(247).add(240); let q1_ = (v98.to_bits() as i32); let q2_ = xi; f_361530(cx, q0_, q1_, q2_) };
                        }
                        let _ = cx.preinc_i(cx.lptr(684));
                    }
                }
            }
        }
    }
    if (((((cx.i(cx.lpp(246).add(56)) != 1) || (cx.b(cx.lpp(247).add(16)) != 0)) || (cx.i(cx.lpp(247).add(3740)) != 0)) || (cx.i(cx.lpp(247).add(3828)) != 0)) || ((cx.i(cx.lpp(247).add(3596)) != 0) && (cx.f(cx.lpp(247).add(3780)) != (cx.i(cx.lpp(247).add(3592)) as f32)))) {
        { let p = cx.lpp(247).add(3596); let t = 0; cx.set_i(p, t); }
        { let p = cx.lpp(247).add(3600); let t = 0; cx.set_i(p, t); }
        if (cx.i(cx.lpp(247).add(3572)) <= 0) {
            { let p = cx.lpp(246).add(8); let t = 1065353216; cx.set_i(p, t); }
        } else {
            { let p = cx.lpp(246).add(8); let t = (((((cx.f(cx.lpp(245).add(8)) - 1.0f32) * (1.0f32 - (cx.f(cx.ptr_at(cx.lpp(247).add(3552))) / cx.gc.characteristic_decay_time))) as f64) + 1.0f64) as f32); cx.set_f(p, t); }
        }
    } else {
        if (cx.i(cx.lpp(247).add(3596)) == 0) {
            { let p = cx.lpp(247).add(3592); let t = (cx.f(cx.lpp(247).add(3780)) as i32); cx.set_i(p, t); }
        }
        let _ = cx.preinc_i(cx.lpp(247).add(3596));
        xj = ({ let q0_ = cx.lpp(247).add(240); let q1_ = 3; f_364a00(cx, q0_, q1_) } as f32);
        v99 = ({ let q0_ = cx.lpp(247).add(240); let q1_ = 2; f_364a00(cx, q0_, q1_) } as f32);
        xk = (f_1d9a90(cx, v99, xj) as f32);
        v100 = ({ let q0_ = cx.lpp(247).add(240); let q1_ = 1; f_364a00(cx, q0_, q1_) } as f32);
        v96 = ({ let q0_ = cx.lpp(247).add(240); let q1_ = 0; f_364a00(cx, q0_, q1_) } as f32);
        v101 = (f_1d9a90(cx, v96, v100) as f32);
        v34 = f_1d9a90(cx, v101, xk);
        { let p = cx.lpp(247).add(3600); let t = (((v34 / (cx.f(cx.lpp(247).add(1356)) as f64)) + (cx.f(cx.lpp(247).add(3600)) as f64)) as f32); cx.set_f(p, t); }
        { let t = (cx.i(cx.lpp(247).add(3596)) as f32); cx.set_lf(672, t); }
        { let t = cx.f(cx.lpp(247).add(3600)); cx.set_lf(676, t); }
        { let t = { let q0_ = cx.lptr(660); let q1_ = 0; let q2_ = Ptr::gexpr(776); let q3_ = cx.lptr(672); ov_eval_expr(cx, q0_, q1_, q2_, q3_) }; cx.set_li(652, t); }
        if (cx.li(652) != 0) {
            { let p = cx.lpp(246).add(8); let t = 1065353216; cx.set_i(p, t); }
        } else {
            { let p = cx.lpp(246).add(8); let t = cx.lf(660); cx.set_f(p, t); }
        }
    }
    { let p = cx.lpp(247).add(3644); let t = (((cx.f(cx.lpp(247).add(3644)) as f64) * (cx.f(cx.lpp(246).add(8)) as f64)) as f32); cx.set_f(p, t); }
    if (cx.i(cx.lpp(246).add(44)) == 1) {
        { let p = cx.lpp(247).add(3724); let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(148)) as f64) / ((cx.gc.forward_velocity_dependent_steering_constant_B_at_zero_speed + cx.f(cx.lpp(247).add(3784))) as f64)) as f32); cx.set_f(p, t); }
        if ((cx.f(cx.lpp(247).add(3784)) == 0.0f32) || (cx.gc.friction_limited_steereing_method == 0.0f32)) {
            { let p = cx.lpp(247).add(3728); let t = 0; cx.set_i(p, t); }
        } else {
            v35 = vsmath::fabs(((((((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(28)) * cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(296))) * cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(300))) * (0.0f32 - cx.gc.gravity[1 as usize])) as f64) / ((cx.f(cx.lpp(247).add(3784)) * cx.f(cx.lpp(247).add(3784))) as f64)) as f32));
            { let p = cx.lpp(247).add(3728); let t = ((v35 * (cx.gc.forward_velocity_dependent_steering_constant_C as f64)) as f32); cx.set_f(p, t); }
        }
    } else {
        { let p = cx.lpp(247).add(3724); let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(152)) as f64) / ((cx.gc.backward_velocity_dependent_steering_constant_B_at_zero_speed + cx.f(cx.lpp(247).add(3784))) as f64)) as f32); cx.set_f(p, t); }
        if ((cx.f(cx.lpp(247).add(3784)) == 0.0f32) || (cx.gc.friction_limited_steereing_method == 0.0f32)) {
            { let p = cx.lpp(247).add(3728); let t = 0; cx.set_i(p, t); }
        } else {
            v36 = vsmath::fabs(((((((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(28)) * cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(296))) * cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(300))) * (0.0f32 - cx.gc.gravity[1 as usize])) as f64) / ((cx.f(cx.lpp(247).add(3784)) * cx.f(cx.lpp(247).add(3784))) as f64)) as f32));
            { let p = cx.lpp(247).add(3728); let t = ((v36 * (cx.gc.backward_velocity_dependent_steering_constant_C as f64)) as f32); cx.set_f(p, t); }
        }
    }
    { let p = cx.lpp(246).add(40); let t = ((((cx.gc.friction_limited_steereing_method * cx.f(cx.lpp(247).add(3728))) as f64) + (((1.0f32 - cx.gc.friction_limited_steereing_method) * cx.f(cx.lpp(247).add(3724))) as f64)) as f32); cx.set_f(p, t); }
    if (cx.f(cx.lpp(247).add(3644)) != 1.0f32) {
        { let p = cx.lpp(246).add(40); let t = (((cx.f(cx.lpp(246).add(40)) as f64) * (cx.f(cx.lpp(247).add(3644)) as f64)) as f32); cx.set_f(p, t); }
    }
    if (cx.f(cx.lpp(247).add(3648)) != 1.0f32) {
        { let p = cx.lpp(246).add(40); let t = (((cx.f(cx.lpp(246).add(40)) as f64) * (cx.f(cx.lpp(247).add(3648)) as f64)) as f32); cx.set_f(p, t); }
    }
    if (((cx.i(cx.lpp(246).add(44)) != 0) && (cx.f(cx.lpp(245).add(40)) > cx.f(cx.lpp(246).add(40)))) && ((((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(172)) > cx.f(cx.lpp(247).add(3784))) && (cx.f(cx.lpp(246).add(68)) != 0.0f32)) && ({ let q0_ = cx.f(cx.lpp(246).add(72)); let q1_ = cx.f(cx.lpp(245).add(72)); let q2_ = 0.03f32; let q3_ = 0.03f32; f_1d9bd0(cx, q0_, q1_, q2_, q3_) } != 0)) || ((cx.i(cx.lpp(246).add(56)) == 1) && ((cx.f(cx.lpp(246).add(68)) * cx.f(cx.lpp(245).add(68))) >= 0.0f32)))) {
        { let p = cx.lpp(246).add(40); let t = cx.i(cx.lpp(245).add(40)); cx.set_i(p, t); }
    }
    { let q0_ = cx.lpp(246).add(40); let q1_ = 0.0f32; let q2_ = cx.gc.maximum_steering_angle_ever; f_1bfe80(cx, q0_, q1_, q2_) };
    { let p = cx.lpp(247).add(3660); let t = (((cx.f(cx.lpp(246).add(68)) as f64) * (cx.f(cx.lpp(246).add(40)) as f64)) as f32); cx.set_f(p, t); }
    if (cx.f(cx.lpp(247).add(3616)) != 0.0f32) {
        { let q0_ = cx.lpp(247).add(3616); let q1_ = ((((0.0f32 - cx.f(cx.lpp(246).add(40))) as f64) * (cx.gc.SS_friction_limited_SA_multiplier as f64)) as f32); let q2_ = (((cx.f(cx.lpp(246).add(40)) as f64) * (cx.gc.SS_friction_limited_SA_multiplier as f64)) as f32); f_1bfe80(cx, q0_, q1_, q2_) };
        if ((cx.f(cx.lpp(246).add(72)) > 0.1f32) || (cx.f(cx.lpp(246).add(36)) == 0.0f32)) {
            { let p = cx.lpp(247).add(3660); let t = (((((cx.f(cx.lpp(247).add(3616)) - cx.f(cx.lpp(247).add(3660))) * cx.gc.SS_steering_bias_when_steering) as f64) + (cx.f(cx.lpp(247).add(3660)) as f64)) as f32); cx.set_f(p, t); }
        } else {
            { let p = cx.lpp(247).add(3660); let t = (((((cx.f(cx.lpp(247).add(3616)) - cx.f(cx.lpp(247).add(3660))) * cx.gc.SS_steering_bias) as f64) + (cx.f(cx.lpp(247).add(3660)) as f64)) as f32); cx.set_f(p, t); }
        }
    }
    { let t = ({ let q0_ = cx.lpp(247).add(240); f_362440(cx, q0_) } as f32); cx.set_lf(648, t); }
    { let t = 0.0f32; cx.set_lf(644, t); }
    { let t = 0.0f32; cx.set_lf(640, t); }
    if ((cx.f(cx.lpp(246).add(68)) == 0.0f32) || (({ let q0_ = cx.f(cx.lpp(246).add(68)); let q1_ = cx.f(cx.lpp(245).add(68)); let q2_ = 0.03f32; let q3_ = 0.03f32; f_1d9b10(cx, q0_, q1_, q2_, q3_) } != 0) && (cx.f(cx.lpp(246).add(72)) <= 0.99f32))) {
        { let t = (((cx.gc.max_passive_steering_rate as f64) * (cx.f(cx.ptr_at(cx.lpp(247).add(3552))) as f64)) as f32); cx.set_lf(632, t); }
        { let t = (((cx.gc.min_passive_steering_rate as f64) * (cx.f(cx.ptr_at(cx.lpp(247).add(3552))) as f64)) as f32); cx.set_lf(624, t); }
        if (cx.i(cx.lpp(247).add(3740)) != 0) {
            if (cx.gc.passive_steering_change_time_at_initiated_skid == 0.0f32) {
                { let t = 3.4028235e38f32; cx.set_lf(640, t); }
            } else {
                { let t = cx.gc.passive_steering_change_time_at_initiated_skid; cx.set_lf(640, t); }
            }
        } else if (cx.gc.passive_steering_change_time == 0.0f32) {
            { let t = 3.4028235e38f32; cx.set_lf(640, t); }
        } else {
            { let t = cx.gc.passive_steering_change_time; cx.set_lf(640, t); }
        }
    } else {
        { let t = (((cx.gc.max_active_steering_rate as f64) * (cx.f(cx.ptr_at(cx.lpp(247).add(3552))) as f64)) as f32); cx.set_lf(632, t); }
        { let t = (((cx.gc.min_active_steering_rate as f64) * (cx.f(cx.ptr_at(cx.lpp(247).add(3552))) as f64)) as f32); cx.set_lf(624, t); }
        if (cx.i(cx.lpp(247).add(3740)) != 0) {
            if (cx.gc.active_steering_change_time_at_initiated_skid == 0.0f32) {
                { let t = 3.4028235e38f32; cx.set_lf(640, t); }
            } else {
                { let t = cx.gc.active_steering_change_time_at_initiated_skid; cx.set_lf(640, t); }
            }
        } else if (cx.gc.active_steering_change_time == 0.0f32) {
            { let t = 3.4028235e38f32; cx.set_lf(640, t); }
        } else {
            { let t = cx.gc.active_steering_change_time; cx.set_lf(640, t); }
        }
    }
    if (cx.i(cx.lpp(247).add(3828)) != 0) {
        { let t = (((cx.lf(640) as f64) * (cx.gc.zone_steering_time_multiplier as f64)) as f32); cx.set_lf(640, t); }
    }
    if (cx.gc.steering_response_method != 0) {
        if (cx.gc.steering_response_method == 1) {
            { let t = (((cx.f(cx.lpp(247).add(3660)) as f64) - (cx.lf(648) as f64)) as f32); cx.set_lf(620, t); }
            v38 = vsmath::exp(((((0.0f32 - cx.f(cx.ptr_at(cx.lpp(247).add(3552)))) as f64) / (cx.lf(640) as f64)) as f32));
            v37 = ((1.0f64 - v38) * (cx.lf(620) as f64));
            { let t = (v37 as f32); cx.set_lf(620, t); }
            { let t = (((cx.lf(648) as f64) + (cx.lf(620) as f64)) as f32); cx.set_lf(644, t); }
        } else {
            { let t = cx.f(cx.lpp(246).add(40)); cx.set_lf(616, t); }
            { let t = cx.ptr_at(cx.lpp(247).add(3552)); cx.set_lpp(153, t); }
            { let t = (((cx.f(cx.lpp(247).add(3660)) as f64) / (cx.lf(616) as f64)) as f32); cx.set_lf(608, t); }
            { let t = (((cx.f(cx.lpp(247).add(4452)) as f64) - (cx.lf(608) as f64)) as f32); cx.set_lf(604, t); }
            { let t = cx.lf(640); cx.set_lf(600, t); }
            { let t = cx.f(cx.lpp(247).add(4448)); cx.set_lf(596, t); }
            { let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552))) as f64) / (cx.lf(640) as f64)) as f32); cx.set_lf(592, t); }
            v37 = vsmath::exp(((0.0f64 - (cx.lf(592) as f64)) as f32));
            { let t = (v37 as f32); cx.set_lf(588, t); }
            { let t = ((((((cx.lf(592) + 1.0f32) * cx.lf(604)) + (cx.lf(596) * cx.f(cx.lpp(153)))) as f64) * (cx.lf(588) as f64)) as f32); cx.set_lf(584, t); }
            { let p = cx.lpp(247).add(4448); let t = ((((((1.0f32 - cx.lf(592)) * cx.lf(596)) - ((cx.lf(604) * cx.lf(592)) / cx.lf(600))) as f64) * (cx.lf(588) as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lpp(247).add(4452); let t = ((((cx.lf(584) - cx.lf(604)) as f64) + (cx.f(cx.lpp(247).add(4452)) as f64)) as f32); cx.set_f(p, t); }
            { let t = (((cx.f(cx.lpp(247).add(4452)) as f64) * (cx.lf(616) as f64)) as f32); cx.set_lf(644, t); }
        }
    } else {
        { let t = ((((cx.f(cx.ptr_at(cx.lpp(247).add(3552))) / cx.lf(640)) as f64) * (cx.f(cx.lpp(246).add(40)) as f64)) as f32); cx.set_lf(628, t); }
        { let t = ({ let q0_ = cx.lf(632); let q1_ = cx.lf(628); f_1d9ad0(cx, q0_, q1_) } as f32); cx.set_lf(636, t); }
        { let t = ({ let q0_ = cx.lf(624); let q1_ = cx.lf(636); f_1d9a90(cx, q0_, q1_) } as f32); cx.set_lf(636, t); }
        v37 = ({ let q0_ = cx.lf(648); let q1_ = cx.f(cx.lpp(247).add(3660)); let q2_ = cx.lf(636); f_1d9cb0(cx, q0_, q1_, q2_) } as f64);
        { let t = (v37 as f32); cx.set_lf(644, t); }
    }
    if ((cx.i(cx.lpp(246).add(44)) != 0) && (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(188)) != 0.0f32)) {
        if (cx.lf(644) >= 0.0f32) {
            if ((0.0f32 - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(192))) <= cx.f(cx.lpp(247).add(3772))) {
                if ((0.0f32 - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(188))) > cx.f(cx.lpp(247).add(3772))) {
                    { let t = (((((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(192)) + cx.f(cx.lpp(247).add(3772))) / (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(192)) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(188)))) as f64) * (cx.lf(644) as f64)) as f32); cx.set_lf(644, t); }
                }
            } else {
                { let t = 0.0f32; cx.set_lf(644, t); }
            }
        } else if (cx.f(cx.lpp(247).add(3772)) <= cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(192))) {
            if (cx.f(cx.lpp(247).add(3772)) > cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(188))) {
                { let t = (((((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(192)) - cx.f(cx.lpp(247).add(3772))) / (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(192)) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(188)))) as f64) * (cx.lf(644) as f64)) as f32); cx.set_lf(644, t); }
            }
        } else {
            { let t = 0.0f32; cx.set_lf(644, t); }
        }
    }
    { let q0_ = cx.lpp(247).add(240); let q1_ = cx.lf(644); f_361ff0(cx, q0_, q1_) };
    if (((((cx.f(cx.lpp(246).add(48)) == 0.0f32) || (cx.b(cx.lpp(247).add(16)) != 0)) || ((cx.i(cx.lpp(247).add(3740)) != 0) && (cx.i(cx.lpp(247).add(3740)) <= cx.gc.normal_handbrake_time))) || (cx.b(cx.lpp(247).add(3688)) != 0)) || (((cx.gc.LS_speed_max_speed > cx.f(cx.lpp(247).add(3784))) && (cx.f(cx.lpp(246).add(36)) != 0.0f32)) && (cx.f(cx.lpp(246).add(48)) != 0.0f32))) {
        { let q0_ = cx.lpp(247).add(1476); let q1_ = 0; f_1bfef0(cx, q0_, q1_) };
    } else {
        { let q0_ = cx.lpp(247).add(1476); let q1_ = 1; f_1bfef0(cx, q0_, q1_) };
        { let p = cx.lpp(247).add(3756); let t = 0; cx.set_i(p, t); }
    }
    'l_LABEL_410: {
        if (cx.f(cx.lpp(246).add(36)) != 0.0f32) {
            if (cx.f(cx.lpp(247).add(3704)) > (0.0f32 - cx.gc.speed_threshold)) {
                if (cx.i(cx.lpp(247).add(3756)) != 0) {
                    v39 = { let q0_ = cx.lpp(247).add(240); f_35d1e0(cx, q0_) };
                    { let p = cx.lpp(247).add(3216); let t = v39; cx.set_i(p, t); }
                    if (cx.i(cx.lpp(247).add(3216)) > 0) {
                        _1B74A0 = { let q0_ = cx.lpp(247); f_1b74a0(cx, q0_) };
                        { let p = cx.lpp(247).add(3216); let t = _1B74A0; cx.set_i(p, t); }
                    } else {
                        { let p = cx.lpp(247).add(3216); let t = 1; cx.set_i(p, t); }
                    }
                    if (cx.i(cx.lpp(247).add(3216)) <= 90) {
                        if (cx.i(cx.lpp(246).add(160)) == 0) {
                            { let q0_ = cx.lpp(247).add(240); let q1_ = cx.f(cx.lpp(246).add(36)); let q2_ = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(108)); f_35d4b0(cx, q0_, q1_, q2_) };
                        } else {
                            { let q0_ = cx.lpp(247).add(240); let q1_ = cx.f(cx.lpp(246).add(36)); let q2_ = 0; f_35d4b0(cx, q0_, q1_, q2_) };
                        }
                    } else {
                        { let p = cx.lpp(247).add(3216); let t = cx.i(p).wrapping_sub(100); cx.set_i(p, t); }
                        { let q0_ = cx.lpp(247).add(1444); let q1_ = 0; f_1ba170(cx, q0_, q1_) };
                    }
                } else {
                    v41 = { let q0_ = cx.lpp(247).add(240); f_35d1e0(cx, q0_) };
                    { let p = cx.lpp(247).add(3216); let t = v41; cx.set_i(p, t); }
                    if (cx.i(cx.lpp(247).add(3216)) > 0) {
                        v42 = { let q0_ = cx.lpp(247); f_1b74a0(cx, q0_) };
                        { let p = cx.lpp(247).add(3216); let t = v42; cx.set_i(p, t); }
                    } else {
                        { let p = cx.lpp(247).add(3216); let t = 1; cx.set_i(p, t); }
                    }
                    if (cx.i(cx.lpp(247).add(3216)) <= 90) {
                        if (cx.i(cx.lpp(246).add(160)) == 0) {
                            { let q0_ = cx.lpp(247).add(240); let q1_ = cx.f(cx.lpp(246).add(36)); let q2_ = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(108)); f_35d4b0(cx, q0_, q1_, q2_) };
                        } else {
                            { let q0_ = cx.lpp(247).add(240); let q1_ = cx.f(cx.lpp(246).add(36)); let q2_ = 0; f_35d4b0(cx, q0_, q1_, q2_) };
                        }
                    } else {
                        { let p = cx.lpp(247).add(3216); let t = cx.i(p).wrapping_sub(100); cx.set_i(p, t); }
                        { let q0_ = cx.lpp(247).add(1444); let q1_ = 0; f_1ba170(cx, q0_, q1_) };
                    }
                }
                { let q0_ = cx.lpp(247).add(1460); let q1_ = 0; f_1ba170(cx, q0_, q1_) };
                break 'l_LABEL_410;
            }
            if (cx.i(cx.lpp(246).add(56)) != 0) {
                { let q0_ = cx.lpp(247).add(240); let q1_ = cx.f(cx.lpp(246).add(36)); let q2_ = 0; f_35d6f0(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lpp(247).add(1460); let q1_ = 0; f_1ba170(cx, q0_, q1_) };
            } else if (cx.gc.backwards_traction_controlled_braking == 0.0f32) {
                { let p = cx.lpp(247).add(1464); let t = cx.gc.backwards_brake_torque; cx.set_f(p, t); }
                { let p = cx.lpp(247).add(1468); let t = cx.gc.backwards_brake_torque; cx.set_f(p, t); }
            } else {
                { let t = 1; cx.set_li(580, t); }
                'lp21: while (cx.li(580) < 3) {
                    v37 = (({ let q0_ = cx.lpp(247).add(240); let q1_ = cx.li(580); f_364d20(cx, q0_, q1_) } as f64) * (cx.gc.backwards_traction_controlled_braking as f64));
                    { let t = (v37 as f32); cx.set_lf(576, t); }
                    if (cx.gc.backwards_brake_torque <= cx.lf(576)) {
                        { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(580))).add(1460); let t = cx.gc.backwards_brake_torque; cx.set_f(p, t); }
                    } else {
                        { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(580))).add(1460); let t = cx.lf(576); cx.set_f(p, t); }
                    }
                    let _ = cx.preinc_i(cx.lptr(580));
                }
            }
        }
    }
    if ((cx.f(cx.lpp(246).add(164)) != 0.0f32) && (cx.i(cx.lpp(247).add(3732)) == 0)) {
        if ((cx.gc.speed_threshold <= cx.f(cx.lpp(247).add(3704))) || (cx.b(cx.lpp(247).add(3688)) != 0)) {
            if (cx.gc.forwards_traction_controlled_braking == 0.0f32) {
                { let t = 0.0f32; cx.set_lf(564, t); }
                'lp22: loop {
                    if !(cx.li(564) < 4) { break 'lp22; }
                    'blp22: {
                        if ({ let q0_ = cx.lpp(247).add(240); let q1_ = cx.li(564); f_364a90(cx, q0_, q1_) } == -1) {
                            { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(564))).add(1460); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(140)); cx.set_i(p, t); }
                        } else {
                            { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(564))).add(1460); let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(144)) as f64) * (cx.f(cx.lpp(246).add(164)) as f64)) as f32); cx.set_f(p, t); }
                        }
                    }
                    let _ = cx.preinc_i(cx.lptr(564));
                }
            } else {
                { let t = 0.0f32; cx.set_lf(572, t); }
                'lp23: loop {
                    if !(cx.li(572) < 4) { break 'lp23; }
                    'blp23: {
                        { let t = ((({ let q0_ = cx.lpp(247).add(240); let q1_ = cx.li(572); f_364d20(cx, q0_, q1_) } as f64) * (cx.gc.forwards_traction_controlled_braking as f64)) as f32); cx.set_lf(568, t); }
                        if (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(144)) <= cx.lf(568)) {
                            { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(572))).add(1460); let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(144)) as f64) * (cx.f(cx.lpp(246).add(164)) as f64)) as f32); cx.set_f(p, t); }
                        } else {
                            { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(572))).add(1460); let t = cx.lf(568); cx.set_f(p, t); }
                        }
                    }
                    { let t = cx.li(572).wrapping_add(3); cx.set_li(572, t); }
                }
            }
        } else {
            { let q0_ = cx.lpp(247).add(1460); let q1_ = 0; f_1ba170(cx, q0_, q1_) };
            if (cx.i(cx.lpp(246).add(160)) != 0) {
                { let q0_ = cx.lpp(247).add(240); let q1_ = cx.f(cx.lpp(246).add(164)); let q2_ = 0; f_35d940(cx, q0_, q1_, q2_) };
            } else {
                { let q0_ = cx.lpp(247).add(240); let q1_ = cx.f(cx.lpp(246).add(164)); let q2_ = (cx.gc.backwards_drive_traction_control.to_bits() as i32); f_35d940(cx, q0_, q1_, q2_) };
            }
        }
    }
    if (((cx.f(cx.lpp(246).add(164)) == 0.0f32) || (cx.i(cx.lpp(247).add(3732)) != 0)) && (cx.f(cx.lpp(246).add(36)) == 0.0f32)) {
        { let q0_ = cx.lpp(247).add(1444); let q1_ = 0; f_1ba170(cx, q0_, q1_) };
        { let t = 0.0f32; cx.set_lf(560, t); }
        'lp24: loop {
            if !(cx.li(560) < 4) { break 'lp24; }
            'blp24: {
                if ({ let q0_ = cx.lpp(247).add(240); let q1_ = cx.li(560); f_364a90(cx, q0_, q1_) } == -1) {
                    { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(560))).add(1460); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(140)); cx.set_i(p, t); }
                } else {
                    { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(560))).add(1460); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(136)); cx.set_i(p, t); }
                }
            }
            let _ = cx.preinc_i(cx.lptr(560));
        }
    }
    v43 = { let q0_ = cx.lpp(247); ov_speedo_b144(cx, q0_) };
    if (0.0f64 != v43) {
        v44 = { let q0_ = cx.lpp(247); ov_speedo_f148(cx, q0_) };
        v43 = (cx.f(Ptr::glob(0x688A98)) as f64);
        if (v44 > (cx.f(Ptr::glob(0x688A98)) as f64)) {
            v43 = ((({ let q0_ = cx.lpp(247); ov_speedo_f148(cx, q0_) } - (cx.f(Ptr::glob(0x688A98)) as f64)) * (cx.f(Ptr::glob(0x688A94)) as f64)) + 1.0f64);
            { let t = (v43 as f32); cx.set_lf(556, t); }
            if !({ let q0_ = cx.lf(556); let q1_ = 1.0f32; let q2_ = 0.000001f32; vs_fge(cx, q0_, q1_, q2_) } != 0) {
                if (cx.i(Ptr::glob(0x6C18D8)) != Ptr::NULL.bits()) {
                    let _ = crate::ov_zero_i!(cx, "VsFge(turbo_boost, 1.0f)", ".\\src\\Physics\\Vehicle\\VehicleEntity.cpp", 9307);
                    { let t = v45; cx.set_lf(552, t); }
                    if ((v45.to_bits() as i32) == 3) {
                    }
                    { let t = 257; let o = cx.li(12); cx.set_li(12, (o & !(65535 << 16)) | ((t & 65535) << 16)); }
                } else {
                    { let t = 1; let o = cx.li(12); cx.set_li(12, (o & !(255 << 16)) | ((t & 255) << 16)); }
                }
            }
            { let t = 0.0f32; cx.set_lf(548, t); }
            'lp25: loop {
                if !(cx.li(548) < 4) { break 'lp25; }
                'blp25: {
                    { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(548))).add(1444); let t = (((cx.f(cx.lpp(247).add((4_i32).wrapping_mul(cx.li(548))).add(1444)) as f64) * (cx.lf(556) as f64)) as f32); cx.set_f(p, t); }
                }
                let _ = cx.preinc_i(cx.lptr(548));
            }
        }
    }
    if (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(80)) != 0.0f32) {
        if (cx.f(cx.lpp(247).add(3704)) <= cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(80))) {
            if (cx.f(cx.lpp(247).add(3704)) > (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(80)) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(84)))) {
                { let t = 0.0f32; cx.set_lf(544, t); }
                'lp26: loop {
                    if !(cx.li(544) < 4) { break 'lp26; }
                    'blp26: {
                        { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(544))).add(1444); let t = (((((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(80)) - cx.f(cx.lpp(247).add(3704))) / cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(84))) as f64) * (cx.f(cx.lpp(247).add((4_i32).wrapping_mul(cx.li(544))).add(1444)) as f64)) as f32); cx.set_f(p, t); }
                    }
                    let _ = cx.preinc_i(cx.lptr(544));
                }
            }
        } else {
            { let t = 0.0f32; cx.set_lf(544, t); }
            'lp27: loop {
                if !(cx.li(544) < 4) { break 'lp27; }
                'blp27: {
                }
                { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.postinc_i(cx.lptr(544)))).add(1444); let t = 0; cx.set_i(p, t); }
            }
        }
    }
    if (cx.i(Ptr::glob(0x75B9F0)) != cx.i(cx.lpp(247).add(3216))) {
        { let p = Ptr::glob(0x75B9F0); let t = cx.i(cx.lpp(247).add(3216)); cx.set_i(p, t); }
    }
    if (cx.i(Ptr::glob(0x75B9EC)) != { let q0_ = cx.lpp(247).add(240); f_35d1e0(cx, q0_) }) {
        { let p = Ptr::glob(0x75B9EC); let t = { let q0_ = cx.lpp(247).add(240); f_35d1e0(cx, q0_) }; cx.set_i(p, t); }
    }
    v46 = { let q0_ = cx.lpp(247).add(240); f_35d1e0(cx, q0_) };
    { let p = cx.lpp(247).add(3216); let t = v46; cx.set_i(p, t); }
    { let p = cx.lpp(246).add(76); let t = 0; cx.set_i(p, t); }
    { let p = cx.lpp(246).add(12); let t = 0; cx.set_i(p, t); }
    if (cx.i(cx.lpp(247).add(3756)) != 0) {
        { let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(296)) as f64) * (cx.gc.friction_boost_with_disabled_skidding as f64)) as f32); cx.set_lf(540, t); }
        { let p = cx.lpp(247).add(3252); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(104)); cx.set_i(p, t); }
    } else {
        { let t = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(296)); cx.set_lf(540, t); }
        { let p = cx.lpp(247).add(3252); let t = 0; cx.set_i(p, t); }
    }
    if (cx.f(cx.lpp(247).add(3720)) != 1.0f32) {
        { let t = (((cx.lf(540) as f64) * (cx.f(cx.lpp(247).add(3720)) as f64)) as f32); cx.set_lf(540, t); }
    }
    { let t = 1.0f32; cx.set_lf(536, t); }
    if (((((((cx.f(cx.lpp(246).add(72)) > cx.gc.necessary_steering_input) && (cx.f(cx.lpp(246).add(36)) != 0.0f32)) && (cx.i(cx.lpp(247).add(3740)) != 0)) && (({ let _ = { let _ = { let _ = { xl = cx.lpp(247).add(3664); xl }; { v47 = { let q0_ = cx.lpp(247).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) }; v47 } }; { v43 = f_1c0300(cx, v47, xl); v43 } }; ((cx.gc.min_dir_dot_for_no_doughnuts as f64) > v43) } || (cx.i(cx.lpp(245).add(76)) != 0)) || ({ let p = { let q0_ = cx.lpp(247).add(3560); let q1_ = 2; ov_hist_entry(cx, q0_, q1_) }.add(76); cx.i(p) } != 0))) && (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(176)) > cx.f(cx.lpp(247).add(3784)))) && ((cx.f(cx.lpp(246).add(68)) * cx.f(cx.lpp(247).add(3760))) < 0.0f32)) && (cx.f(cx.lpp(246).add(72)) > 0.1f32)) {
        if (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(172)) <= cx.f(cx.lpp(247).add(3784))) {
            { let t = ((((((1.0f32 - cx.gc.driving_wheels_lateral_friction_boost_at_low_speed) * (cx.f(cx.lpp(247).add(3784)) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(172)))) / (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(176)) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(172)))) as f64) + (cx.gc.driving_wheels_lateral_friction_boost_at_low_speed as f64)) as f32); cx.set_lf(536, t); }
        } else {
            { let t = cx.gc.driving_wheels_lateral_friction_boost_at_low_speed; cx.set_lf(536, t); }
        }
        { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(108); let t = 0; cx.set_i(p, t); }
        { let p = cx.lpp(246).add(76); let t = 1; cx.set_i(p, t); }
        { let p = cx.lpp(247).add(3816); let t = 1; cx.set_b(p, t); }
    } else {
        { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(108); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(268)); cx.set_i(p, t); }
        if (cx.i(cx.lpp(245).add(76)) != 0) {
            { let q0_ = cx.lpp(247).add(240); let q1_ = 0.0f32; f_361ff0(cx, q0_, q1_) };
            { let p = cx.lpp(246).add(68); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(4448); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(4452); let t = 0; cx.set_i(p, t); }
        }
    }
    if ((cx.f(cx.lpp(247).add(3764)) > cx.gc.jturn_max_omega_up_to_stop) && ((((((cx.f(cx.lpp(246).add(72)) > cx.gc.backwards_necessary_steering_input) && (cx.f(cx.lpp(247).add(3764)) > cx.gc.jturn_min_omega_up_to_start)) && (cx.f(cx.lpp(246).add(164)) != 0.0f32)) && ((0.0f32 - cx.gc.min_backwards_speed) > cx.f(cx.lpp(247).add(3704)))) && (cx.i(cx.lpp(246).add(44)) == 0)) || (cx.i(cx.lpp(245).add(12)) != 0))) {
        { let p = cx.lpp(246).add(12); let t = 1; cx.set_i(p, t); }
    }
    if (cx.i(cx.lpp(246).add(76)) != 0) {
        { let t = 0.0f32; cx.set_lf(532, t); }
        'lp28: loop {
            if !(cx.li(532) < 4) { break 'lp28; }
            'blp28: {
                if (cx.i(cx.lpp(247).add((4_i32).wrapping_mul(cx.li(532))).add(3224)) != 0) {
                    { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(532))).add(1524); let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(300)) as f64) * (cx.lf(536) as f64)) as f32); cx.set_f(p, t); }
                    { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(532))).add(1508); let t = cx.gc.driving_wheels_tangential_friction_for_doughnuts; cx.set_f(p, t); }
                } else {
                    { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(532))).add(1524); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(300)); cx.set_i(p, t); }
                    { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(532))).add(1508); let t = cx.lf(540); cx.set_f(p, t); }
                }
            }
            let _ = cx.preinc_i(cx.lptr(532));
        }
    } else if (cx.i(cx.lpp(246).add(12)) != 0) {
        v43 = vsmath::fabs(cx.f(cx.lpp(247).add(3772)));
        if (((cx.gc.max_skid_angle_to_reduce_the_boost_effect as f64) <= v43) && { let _ = { let _ = { v48 = vsmath::fabs(cx.f(cx.lpp(247).add(3772))); v48 }; { v43 = (3.1415927f64 - (cx.gc.max_skid_angle_to_reduce_the_boost_effect as f64)); v43 } }; (v48 <= v43) }) {
            { let t = cx.gc.steering_wheels_lateral_friction_boost; cx.set_lf(528, t); }
        } else {
            { let t = (((((cx.gc.steering_wheels_lateral_friction_boost - 1.0f32) * cx.f(cx.lpp(246).add(72))) as f64) + 1.0f64) as f32); cx.set_lf(528, t); }
        }
        { let t = 0.0f32; cx.set_lf(524, t); }
        'lp29: loop {
            if !(cx.li(524) < 4) { break 'lp29; }
            'blp29: {
                if (cx.i(cx.lpp(247).add((4_i32).wrapping_mul(cx.li(524))).add(3224)) != 0) {
                    { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(524))).add(1524); let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(300)) as f64) * (cx.gc.driving_wheels_tangential_friction_boost as f64)) as f32); cx.set_f(p, t); }
                    { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(524))).add(1508); let t = cx.gc.driving_wheels_tangential_friction; cx.set_f(p, t); }
                }
            }
            let _ = cx.preinc_i(cx.lptr(524));
        }
        { let p = cx.lpp(247).add(1524); let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(300)) as f64) * (cx.lf(528) as f64)) as f32); cx.set_f(p, t); }
        { let p = cx.lpp(247).add(1536); let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(300)) as f64) * (cx.lf(528) as f64)) as f32); cx.set_f(p, t); }
        { let p = cx.lpp(247).add(1508); let t = cx.gc.steering_wheels_lateral_friction; cx.set_f(p, t); }
        { let p = cx.lpp(247).add(1520); let t = cx.gc.steering_wheels_lateral_friction; cx.set_f(p, t); }
    } else {
        { let q0_ = cx.lpp(247).add(1524); let q1_ = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(300)); f_1ba170(cx, q0_, q1_) };
        if (cx.i(cx.lpp(246).add(44)) != 0) {
            { let q0_ = cx.lpp(247).add(1508); let q1_ = cx.li(540); f_1ba170(cx, q0_, q1_) };
        } else {
            { let q0_ = cx.lpp(247).add(1508); let q1_ = { let q0_ = (((cx.lf(540) as f64) * (cx.gc.friction_boost_when_going_backwards as f64)) as f32); ov_coerce_int(cx, q0_) }; f_1ba170(cx, q0_, q1_) };
        }
    }
    { let p = cx.lpp(247).add(3691); let t = 0; cx.set_b(p, t); }
    if ((cx.b(cx.lpp(247).add(3688)) == 0) && (((cx.gc.LS_speed_max_speed <= cx.f(cx.lpp(247).add(3784))) || (cx.f(cx.lpp(246).add(36)) == 0.0f32)) || (cx.f(cx.lpp(246).add(48)) == 0.0f32))) {
        if (cx.i(cx.lpp(247).add(3800)) >= cx.gc.time_for_friction_to_grow) {
            if (cx.i(cx.lpp(246).add(76)) == 0) {
                { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(108); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(268)); cx.set_i(p, t); }
                { let p = cx.lpp(247).add(152); let t = 0; cx.set_b(p, t); }
            }
        } else {
            { let p = cx.lpp(247).add(3690); let t = 0; cx.set_b(p, t); }
            if ((cx.f(cx.lpp(246).add(36)) == 0.0f32) || (cx.f(cx.lpp(246).add(48)) == 0.0f32)) {
                if (cx.i(cx.lpp(247).add(3700)) > 1000) {
                    if (cx.i(cx.lpp(247).add(3800)) == 1) {
                        { let p = cx.lpp(247).add(3700); let t = cx.i(cx.lpp(247).add(3692)).wrapping_sub(cx.i(cx.lpp(247).add(3696))); cx.set_i(p, t); }
                    } else {
                        { let p = cx.lpp(247).add(3700); let t = cx.i(cx.lpp(247).add(3800)); cx.set_i(p, t); }
                    }
                }
            } else {
                { let p = cx.lpp(247).add(3696); let t = 0; cx.set_i(p, t); }
            }
            if (cx.f(cx.lpp(246).add(68)) >= 0.1f32) {
                v43 = { let q0_ = cx.lpp(247).add(240); f_362440(cx, q0_) };
                { let t = (v43 as f32); cx.set_lf(420, t); }
                { let q0_ = cx.lpp(247).add(240); let q1_ = (((cx.lf(420) as f64) * 0.4f64) as f32); f_361ff0(cx, q0_, q1_) };
                { let p = cx.lpp(247).add(152); let t = 0; cx.set_b(p, t); }
            } else {
                { let p = cx.lpp(247).add(1516); let t = cx.gc.friction_at_countdown; cx.set_f(p, t); }
                { let p = cx.lpp(247).add(1512); let t = cx.i(cx.lpp(247).add(1516)); cx.set_i(p, t); }
                { let p = cx.lpp(247).add(152); let t = 1; cx.set_b(p, t); }
            }
            { let p = cx.lpp(247).add(3252); let t = 1065353216; cx.set_i(p, t); }
            { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(108); let t = 0; cx.set_i(p, t); }
            if (((cx.f(cx.lpp(246).add(36)) == 0.0f32) || { let _ = { v43 = vsmath::fabs((cx.i(cx.lpp(247).add(3700)) as f32)); v43 }; ((cx.gc.post_countdown_speed_boost_time_bracket as f64) <= v43) }) || (cx.b(cx.lpp(247).add(3689)) != 0)) {
                if (cx.f(cx.lpp(246).add(36)) != 0.0f32) {
                    v43 = vsmath::fabs((cx.i(cx.lpp(247).add(3800)) as f32));
                    if (((cx.gc.LS_speed_boost_duration as f64) > v43) && (cx.b(cx.lpp(247).add(3689)) != 0)) {
                        v43 = (vsmath::fabs((cx.i(cx.lpp(247).add(3800)) as f32)) / (cx.gc.LS_speed_boost_duration as f64));
                        { let t = (v43 as f32); cx.set_lf(412, t); }
                        xp = (((cx.gc.LS_speed_boost as f64) * (cx.lf(412) as f64)) as f32);
                        v53 = { let q0_ = cx.lpp(247).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                        { let q0_ = cx.lpp(247).add(272); let q1_ = v53; let q2_ = xp; f_1c01b0(cx, q0_, q1_, q2_) };
                    }
                }
            } else {
                xn = (((cx.gc.post_countdown_speed_boost_time_bracket as f64) - vsmath::fabs((cx.i(cx.lpp(247).add(3700)) as f32))) as f32);
                v43 = (vsmath::fabs(xn) / (cx.gc.post_countdown_speed_boost_time_bracket as f64));
                { let t = (v43 as f32); cx.set_lf(416, t); }
                xo = ((((((cx.i(cx.lpp(247).add(3800)) as f32) * cx.gc.post_countdown_speed_boost) / (cx.gc.time_for_friction_to_grow as f32)) as f64) * (cx.lf(416) as f64)) as f32);
                v52 = { let q0_ = cx.lpp(247).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                { let q0_ = cx.lpp(247).add(272); let q1_ = v52; let q2_ = xo; f_1c01b0(cx, q0_, q1_, q2_) };
                { let p = cx.lpp(247).add(3700); let t = (cx.gc.post_countdown_speed_boost_time_bracket as i32); cx.set_i(p, t); }
            }
        }
    } else {
        { let p = cx.lpp(247).add(3691); let t = 1; cx.set_b(p, t); }
        { let p = cx.lpp(247).add(1516); let t = cx.gc.friction_at_countdown; cx.set_f(p, t); }
        { let p = cx.lpp(247).add(1512); let t = cx.i(cx.lpp(247).add(1516)); cx.set_i(p, t); }
        { let p = cx.lpp(247).add(152); let t = 1; cx.set_b(p, t); }
        { let p = cx.lpp(247).add(3252); let t = 0; cx.set_i(p, t); }
        v49 = { let q0_ = cx.lpp(247).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
        { let q0_ = cx.lpp(247).add(272); let q1_ = cx.lpp(247).add(272); let q2_ = v49; f_1c05a0(cx, q0_, q1_, q2_) };
        if (cx.b(cx.lpp(247).add(3690)) != 0) {
            { let p = cx.lpp(247).add(304); let t = cx.i(cx.lpp(247).add(4116)); cx.set_i(p, t); }
            { let p = cx.lpp(247).add(312); let t = cx.i(cx.lpp(247).add(4124)); cx.set_i(p, t); }
            { let p = cx.lpp(247).add(292); let t = 0; cx.set_i(p, t); }
        }
        { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(108); let t = 0; cx.set_i(p, t); }
        { let p = cx.lpp(247).add(3800); let t = 0; cx.set_i(p, t); }
        let _ = cx.preinc_i(cx.lpp(247).add(3692));
        if (cx.b(cx.lpp(247).add(3690)) != 0) {
            if (cx.f(cx.lpp(246).add(36)) != 0.0f32) {
                { let q0_ = cx.lptr(484); let q1_ = cx.lpp(247).add(4100); f_1ce8c0(cx, q0_, q1_) };
                { let q0_ = cx.lptr(440); let q1_ = cx.lpp(247).add(320); f_1ce8c0(cx, q0_, q1_) };
                { let t = ({ let q0_ = cx.lptr(440); let q1_ = cx.lptr(508); f_1c0300(cx, q0_, q1_) } as f32); cx.set_lf(432, t); }
                if (vsmath::fabs(cx.lf(432)) > 1.0f64) {
                    { let t = 0.0f32; cx.set_lf(428, t); }
                } else {
                    xm = ({ let q0_ = cx.lptr(440); let q1_ = cx.lptr(508); f_1c0300(cx, q0_, q1_) } as f32);
                    { let t = (vsmath::asin(xm) as f32); cx.set_lf(428, t); }
                }
                if (cx.b(Ptr::glob(0x6F5BDD)) != 0) {
                    if (vsmath::fabs(cx.lf(428)) <= (cx.gc.max_vibration_angle as f64)) {
                        v43 = ((f_1cf090(cx) as f64) * (cx.gc.vibration_angle_increment as f64));
                        { let t = (v43 as f32); cx.set_lf(424, t); }
                    } else {
                        if (cx.lf(428) <= 0.0f32) {
                            v43 = (((-f_1cf070(cx)) as f64) * (cx.gc.vibration_angle_increment as f64));
                        } else {
                            v43 = ((f_1cf070(cx) as f64) * (cx.gc.vibration_angle_increment as f64));
                        }
                        { let t = (v43 as f32); cx.set_lf(424, t); }
                    }
                } else {
                    { let t = ((((cx.f(cx.lpp(246).add(68)) * cx.f(Ptr::glob(0x688A10))) as f64) * (cx.f(cx.lpp(246).add(36)) as f64)) as f32); cx.set_lf(424, t); }
                    v50 = vsmath::fabs(cx.lf(424));
                    v43 = 0.0f64;
                    if (v50 <= 0.0f64) {
                        v51 = vsmath::fabs(cx.lf(428));
                        v43 = (cx.f(Ptr::glob(0x688A1C)) as f64);
                        if ((v51 <= (cx.f(Ptr::glob(0x688A1C)) as f64)) || (cx.b(cx.lpp(247).add(132)) != 0)) {
                            { let p = cx.lpp(247).add(132); let t = 1; cx.set_b(p, t); }
                            { let t = cx.f(cx.lpp(247).add(136)); cx.set_lf(424, t); }
                            if (cx.f(cx.lpp(247).add(136)) <= 0.0f32) {
                                if (cx.lf(428) > cx.f(Ptr::glob(0x688A1C))) {
                                    { let p = cx.lpp(247).add(136); let t = (((cx.f(Ptr::glob(0x688A18)) as f64) * (cx.f(cx.lpp(246).add(36)) as f64)) as f32); cx.set_f(p, t); }
                                }
                            } else if ((0.0f32 - cx.f(Ptr::glob(0x688A1C))) > cx.lf(428)) {
                                { let p = cx.lpp(247).add(136); let t = ((((0.0f32 - cx.f(Ptr::glob(0x688A18))) as f64) * (cx.f(cx.lpp(246).add(36)) as f64)) as f32); cx.set_f(p, t); }
                            }
                        } else if (cx.lf(428) <= 0.0f32) {
                            { let t = ((((0.0f32 - cx.f(Ptr::glob(0x688A10))) as f64) * (cx.f(cx.lpp(246).add(36)) as f64)) as f32); cx.set_lf(424, t); }
                            { let p = cx.lpp(247).add(136); let t = ((0.0f64 - (cx.f(Ptr::glob(0x688A18)) as f64)) as f32); cx.set_f(p, t); }
                        } else {
                            { let t = (((cx.f(Ptr::glob(0x688A10)) as f64) * (cx.f(cx.lpp(246).add(36)) as f64)) as f32); cx.set_lf(424, t); }
                            { let p = cx.lpp(247).add(136); let t = cx.i(Ptr::glob(0x688A18)); cx.set_i(p, t); }
                        }
                    } else {
                        { let p = cx.lpp(247).add(132); let t = 0; cx.set_b(p, t); }
                        if (cx.lf(428) <= cx.f(Ptr::glob(0x688A14))) {
                            if (((0.0f32 - cx.f(Ptr::glob(0x688A14))) > cx.lf(428)) && (cx.lf(424) > 0.0f32)) {
                                { let t = 0.0f32; cx.set_lf(424, t); }
                            }
                        } else if (cx.lf(424) < 0.0f32) {
                            { let t = 0.0f32; cx.set_lf(424, t); }
                        }
                    }
                }
                { let p = cx.lpp(247).add(3691); let t = 0; cx.set_b(p, t); }
            }
        } else {
            { let q0_ = cx.lpp(247).add(240); let q1_ = cx.lpp(247).add(4100); f_36a140(cx, q0_, q1_) };
            { let _ = { let q0_ = cx.lpp(247).add(240); let q1_ = cx.lpp(247).add(4116); vcall(cx, 28, q0_, &[Varg::from(q1_)]) }; };
            { let p = cx.lpp(247).add(3690); let t = 1; cx.set_b(p, t); }
        }
        if ((cx.f(cx.lpp(246).add(36)) != 0.0f32) && (cx.f(cx.lpp(246).add(48)) != 0.0f32)) {
            { let p = cx.lpp(247).add(3696); let t = cx.i(cx.lpp(247).add(3692)); cx.set_i(p, t); }
        }
        { let p = cx.lpp(247).add(3700); let t = 10000; cx.set_i(p, t); }
        { let p = cx.lpp(247).add(3689); let t = ((cx.b(cx.lpp(247).add(3688)) == 0) as i32); cx.set_b(p, t); }
    }
    if ((((((cx.i(cx.lpp(246).add(44)) == 0) || (cx.i(cx.lpp(247).add(3652)) > cx.gc.normal_handbrake_time)) || (cx.i(cx.lpp(246).add(64)) != 0)) || (cx.i(cx.lpp(246).add(92)) != 0)) || (cx.i(cx.lpp(247).add(4756)) == 1)) || (cx.i(cx.lpp(246).add(160)) != 0)) {
        { let p = cx.lpp(247).add(1592); let t = 0; cx.set_i(p, t); }
    } else if ((((cx.i(cx.lpp(246).add(56)) != 1) && (cx.gc.use_physics_vs_play_only_during_skidding != 0)) || (cx.f(cx.lpp(247).add(3708)) <= cx.gc.physics_vs_play_min_speed)) || (cx.i(cx.lpp(246).add(76)) != 0)) {
        if (cx.i(cx.lpp(246).add(76)) != 0) {
            { let p = cx.lpp(247).add(1592); let t = cx.gc.physycs_vs_play_for_doughnuts; cx.set_f(p, t); }
        } else {
            { let p = cx.lpp(247).add(1592); let t = 0; cx.set_i(p, t); }
        }
    } else if (cx.gc.physics_vs_play_reduction_time <= (cx.i(cx.lpp(247).add(3656)) as f32)) {
        { let p = cx.lpp(247).add(1592); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(184)); cx.set_i(p, t); }
    } else {
        { let p = cx.lpp(247).add(1592); let t = (((((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(184)) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(180))) * ((cx.i(cx.lpp(247).add(3656)) as f32) / cx.gc.physics_vs_play_reduction_time)) as f64) + (cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(180)) as f64)) as f32); cx.set_f(p, t); }
    }
    if ((cx.f(cx.lpp(247).add(3764)) > cx.gc.max_omega_up_for_SF) && (cx.gc.max_speed_to_tamper_with_force_DF > cx.f(cx.lpp(247).add(3784)))) {
        { let p = cx.lpp(247).add(1360); let t = 10; cx.set_i(p, t); }
    }
    if (cx.i(cx.lpp(247).add(3572)) > 1) {
        if (cx.f(cx.lpp(246)) == 0.0f32) {
            { let p = cx.lpp(247).add(3584); let t = 0; cx.set_i(p, t); }
            if (cx.f(cx.lpp(245)) != 0.0f32) {
                { let p = cx.lpp(247).add(3588); let t = 0; cx.set_i(p, t); }
            }
        } else {
            let _ = cx.preinc_i(cx.lpp(247).add(3584));
            if (cx.f(cx.lpp(245)) == 0.0f32) {
                let _ = cx.preinc_i(cx.lpp(247).add(3580));
                { let p = cx.lpp(247).add(3588); let t = 0; cx.set_i(p, t); }
            }
        }
        let _ = cx.preinc_i(cx.lpp(247).add(3588));
        if ((((((cx.f(cx.lpp(246).add(4)) == 0.0f32) || (cx.i(cx.lpp(247).add(3828)) != 0)) || (cx.i(cx.lpp(247).add(3820)) != 0)) || (cx.i(cx.lpp(247).add(3612)) != 0)) || (cx.f(cx.lpp(247).add(104)) <= 0.0f32)) || (cx.b(Ptr::glob(0x6F7158)) != 0)) {
            if (((((cx.f(cx.lpp(246)) == 0.0f32) || (cx.i(cx.lpp(247).add(3828)) != 0)) || (cx.i(cx.lpp(247).add(3820)) != 0)) || (cx.i(cx.lpp(247).add(3612)) != 0)) || (cx.f(cx.lpp(247).add(96)) <= 0.0f32)) {
                if ((((cx.i(cx.lpp(247).add(3828)) == 0) || (cx.f(cx.lpp(246).add(4)) == 0.0f32)) || (cx.f(cx.lpp(247).add(104)) <= (cx.i(cx.lpp(247).add(3832)) as f32))) || ({ let _ = { let _ = { let _ = { let _ = { xq = cx.i(cx.lpp(247).add(5072)); xq }; { Instance = ov_ai_world(cx); Instance } }; { RaceManager = ov_race_manager(cx, Instance); RaceManager } }; { VehicleById = crate::ov_null!(cx, RaceManager, xq); VehicleById } }; ov_ai_vehicle_flag(cx, VehicleById) } != 0)) {
                    if ((cx.i(cx.lpp(247).add(3828)) != 0) && ((cx.f(cx.lpp(246).add(4)) == 0.0f32) || ((cx.i(cx.lpp(247).add(3832)) as f32) > cx.f(cx.lpp(247).add(104))))) {
                        { let q0_ = cx.lpp(247); f_1cc380(cx, q0_) };
                    } else if (((cx.i(cx.lpp(247).add(3820)) == 0) || (cx.f(cx.lpp(246)) == 0.0f32)) || (cx.f(cx.lpp(247).add(96)) <= 0.0f32)) {
                        if ((cx.i(cx.lpp(247).add(3820)) != 0) && (((cx.f(cx.lpp(246)) == 0.0f32) || ((cx.gc.stop_nos_when_colliding != 0) && (cx.i(cx.lpp(247).add(4856)) < 2))) || (cx.f(cx.lpp(247).add(96)) <= 0.0f32))) {
                            { let q0_ = cx.lpp(247); f_1d1120(cx, q0_) };
                        }
                    } else {
                        { let p = cx.lpp(247).add(96); let t = (((cx.f(cx.lpp(247).add(96)) as f64) - 1.0f64) as f32); cx.set_f(p, t); }
                        if (cx.f(cx.lpp(247).add(96)) < 0.0f32) {
                            { let p = cx.lpp(247).add(96); let t = 0; cx.set_i(p, t); }
                        }
                    }
                } else {
                    { let p = cx.lpp(247).add(104); let t = (((cx.f(cx.lpp(247).add(104)) as f64) - 1.0f64) as f32); cx.set_f(p, t); }
                    if ((cx.f(cx.lpp(247).add(108)) * 0.25f32) > cx.f(cx.lpp(247).add(104))) {
                        { let p = cx.lpp(247).add(104); let t = (((cx.f(cx.lpp(247).add(108)) as f64) * 0.25f64) as f32); cx.set_f(p, t); }
                        { let p = cx.lpp(247).add(3832); let t = ((cx.f(cx.lpp(247).add(104)) - 5.0f32) as i32); cx.set_i(p, t); }
                    }
                }
            } else {
                { let p = cx.lpp(247).add(3820); let t = 1; cx.set_i(p, t); }
                { let p = cx.lpp(247).add(88); let t = 1; cx.set_b(p, t); }
                { let p = cx.lpp(247).add(3824); let t = 1; cx.set_i(p, t); }
            }
        } else {
            { let p = cx.lpp(247).add(3828); let t = 1; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(89); let t = 1; cx.set_b(p, t); }
            { let p = cx.lpp(247).add(3832); let t = 1; cx.set_i(p, t); }
        }
        if (cx.i(cx.lpp(247).add(3588)) > cx.gc.max_time_between_first_and_second_tap) {
            { let p = cx.lpp(247).add(3580); let t = 0; cx.set_i(p, t); }
        }
        if (cx.i(cx.lpp(247).add(3828)) != 0) {
            if (cx.i(cx.lpp(247).add(3832)) > cx.gc.zone_fade_in_time) {
                { let t = 1.0f32; cx.set_lf(408, t); }
            } else {
                { let t = ((((cx.i(cx.lpp(247).add(3832)) as f32) as f64) / ((cx.gc.zone_fade_in_time as f32) as f64)) as f32); cx.set_lf(408, t); }
            }
            if (cx.f(cx.lpp(247).add(3784)) <= cx.gc.visual_zone_velocity) {
                { let p = cx.lpp(247).add(3836); let t = 1065353216; cx.set_i(p, t); }
            } else {
                { let p = cx.lpp(247).add(3836); let t = (((cx.gc.visual_zone_velocity as f64) / (cx.f(cx.lpp(247).add(3784)) as f64)) as f32); cx.set_f(p, t); }
            }
            { let p = Ptr::glob(0x688B70); let t = ({ let q0_ = 1.0f32; let q1_ = cx.f(cx.lpp(247).add(3836)); let q2_ = cx.lf(408); f_1d9c90(cx, q0_, q1_, q2_) } as f32); cx.set_f(p, t); }
            v57 = { let q0_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(272)); let q1_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(272)) as f64) * (cx.gc.zone_sensitivities_multiplier as f64)) as f32); let q2_ = cx.lf(408); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(148); let t = (v57 as f32); cx.set_f(p, t); }
            v58 = { let q0_ = 1.0f32; let q1_ = cx.gc.zone_friction_multiplier; let q2_ = cx.lf(408); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.lpp(247).add(3720); let t = (v58 as f32); cx.set_f(p, t); }
            v43 = { let q0_ = cx.gc.fMaxSlipStreamDist; let q1_ = (((cx.gc.fMaxSlipStreamDist as f64) * (cx.gc.zone_slipstream_length_multiplier as f64)) as f32); let q2_ = cx.lf(408); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(8); let t = (v43 as f32); cx.set_f(p, t); }
        } else if (cx.i(cx.lpp(247).add(3832)) >= cx.gc.zone_fade_out_time.wrapping_add(2)) {
            if ((cx.i(cx.lpp(247).add(3828)) == 0) && (cx.i(cx.lpp(247).add(3832)) < cx.gc.zone_fade_out_time.wrapping_add(3))) {
                { let q0_ = cx.lpp(247); f_1cc2e0(cx, q0_) };
            }
        } else {
            if (cx.i(cx.lpp(247).add(3832)) > cx.gc.zone_fade_out_time) {
                { let t = 1.0f32; cx.set_lf(404, t); }
            } else {
                { let t = ((((cx.i(cx.lpp(247).add(3832)) as f32) as f64) / ((cx.gc.zone_fade_out_time as f32) as f64)) as f32); cx.set_lf(404, t); }
            }
            { let p = Ptr::glob(0x688B70); let t = ({ let q0_ = cx.f(cx.lpp(247).add(3836)); let q1_ = 1.0f32; let q2_ = cx.lf(404); f_1d9c90(cx, q0_, q1_, q2_) } as f32); cx.set_f(p, t); }
            v59 = { let q0_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(272)) as f64) * (cx.gc.zone_sensitivities_multiplier as f64)) as f32); let q1_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(272)); let q2_ = cx.lf(404); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(148); let t = (v59 as f32); cx.set_f(p, t); }
            v60 = { let q0_ = cx.gc.zone_friction_multiplier; let q1_ = 1.0f32; let q2_ = cx.lf(404); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.lpp(247).add(3720); let t = (v60 as f32); cx.set_f(p, t); }
            v43 = { let q0_ = (((cx.gc.fMaxSlipStreamDist as f64) * (cx.gc.zone_slipstream_length_multiplier as f64)) as f32); let q1_ = cx.gc.fMaxSlipStreamDist; let q2_ = cx.lf(404); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(8); let t = (v43 as f32); cx.set_f(p, t); }
        }
        if ((cx.i(cx.lpp(247).add(3820)) != 0) && (cx.i(cx.lpp(247).add(3824)) < cx.gc.nos_fade_in_time.wrapping_add(2))) {
            if ((cx.i(cx.lpp(247).add(3824)) <= cx.gc.nos_fade_in_time) && (cx.gc.nos_fade_in_time != 0)) {
                { let p = Ptr::glob(0x75B27C); let t = ((((cx.i(cx.lpp(247).add(3824)) as f32) as f64) / ((cx.gc.nos_fade_in_time as f32) as f64)) as f32); cx.set_f(p, t); }
            } else {
                { let p = Ptr::glob(0x75B27C); let t = 1.0f32; cx.set_f(p, t); }
            }
            v61 = { let q0_ = cx.gc.timestep; let q1_ = (((cx.gc.timestep as f64) * (cx.gc.nos_timestep_multiplier as f64)) as f32); let q2_ = cx.f(Ptr::glob(0x75B27C)); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.ptr_at(cx.lpp(247).add(3552)); let t = (v61 as f32); cx.set_f(p, t); }
            v62 = { let q0_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(272)); let q1_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(272)) as f64) * (cx.gc.nos_sensitivities_multiplier as f64)) as f32); let q2_ = cx.f(Ptr::glob(0x75B27C)); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(148); let t = (v62 as f32); cx.set_f(p, t); }
            v63 = { let q0_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(264)); let q1_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(264)) as f64) * (cx.gc.nos_engine_power_multiplier as f64)) as f32); let q2_ = cx.f(Ptr::glob(0x75B27C)); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.lpp(247).add(3220); let t = (v63 as f32); cx.set_f(p, t); }
            v64 = { let q0_ = 1.0f32; let q1_ = cx.gc.nos_max_speed_multiplier; let q2_ = cx.f(Ptr::glob(0x75B27C)); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.lpp(247).add(3260); let t = (v64 as f32); cx.set_f(p, t); }
            v43 = { let q0_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(236)); let q1_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(236)) as f64) * (cx.gc.nos_max_speed_multiplier as f64)) as f32); let q2_ = cx.f(Ptr::glob(0x75B27C)); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(80); let t = (v43 as f32); cx.set_f(p, t); }
        } else if ((cx.i(cx.lpp(247).add(3820)) != 0) || (cx.i(cx.lpp(247).add(3824)) >= cx.gc.nos_fade_out_time.wrapping_add(2))) {
            if ((cx.i(cx.lpp(247).add(3820)) == 0) && (cx.i(cx.lpp(247).add(3824)) < cx.gc.nos_fade_out_time.wrapping_add(3))) {
                { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(80); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(236)); cx.set_i(p, t); }
                { let p = cx.lpp(247).add(3260); let t = 1065353216; cx.set_i(p, t); }
                { let p = cx.lpp(247).add(3220); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(264)); cx.set_i(p, t); }
                { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(148); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(272)); cx.set_i(p, t); }
                { let p = cx.ptr_at(cx.lpp(247).add(3552)); let t = (cx.gc.timestep.to_bits() as i32); cx.set_i(p, t); }
                { let p = Ptr::glob(0x75B27C); let t = 0.0f32; cx.set_f(p, t); }
            }
        } else {
            if ((cx.i(cx.lpp(247).add(3824)) <= cx.gc.nos_fade_out_time) && (cx.gc.nos_fade_out_time != 0)) {
                { let p = Ptr::glob(0x75B27C); let t = ((((cx.i(cx.lpp(247).add(3824)) as f32) as f64) / ((cx.gc.nos_fade_out_time as f32) as f64)) as f32); cx.set_f(p, t); }
            } else {
                { let p = Ptr::glob(0x75B27C); let t = 1.0f32; cx.set_f(p, t); }
            }
            v65 = { let q0_ = (((cx.gc.timestep as f64) * (cx.gc.nos_timestep_multiplier as f64)) as f32); let q1_ = cx.gc.timestep; let q2_ = cx.f(Ptr::glob(0x75B27C)); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.ptr_at(cx.lpp(247).add(3552)); let t = (v65 as f32); cx.set_f(p, t); }
            v66 = { let q0_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(272)) as f64) * (cx.gc.nos_sensitivities_multiplier as f64)) as f32); let q1_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(272)); let q2_ = cx.f(Ptr::glob(0x75B27C)); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(148); let t = (v66 as f32); cx.set_f(p, t); }
            v67 = { let q0_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(264)) as f64) * (cx.gc.nos_engine_power_multiplier as f64)) as f32); let q1_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(264)); let q2_ = cx.f(Ptr::glob(0x75B27C)); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.lpp(247).add(3220); let t = (v67 as f32); cx.set_f(p, t); }
            v68 = { let q0_ = cx.gc.nos_max_speed_multiplier; let q1_ = 1.0f32; let q2_ = cx.f(Ptr::glob(0x75B27C)); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.lpp(247).add(3260); let t = (v68 as f32); cx.set_f(p, t); }
            v43 = { let q0_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(236)) as f64) * (cx.gc.nos_max_speed_multiplier as f64)) as f32); let q1_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(236)); let q2_ = cx.f(Ptr::glob(0x75B27C)); f_1d9c90(cx, q0_, q1_, q2_) };
            { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(80); let t = (v43 as f32); cx.set_f(p, t); }
        }
        if (cx.i(cx.lpp(247).add(3832)) < 10000) {
            let _ = cx.preinc_i(cx.lpp(247).add(3832));
        }
        if (cx.i(cx.lpp(247).add(3824)) < 10000) {
            let _ = cx.preinc_i(cx.lpp(247).add(3824));
        }
    }
    if ((((((cx.i(cx.lpp(247).add(3612)) != 0) || (cx.f(cx.lpp(246).add(36)) == 0.0f32)) || (cx.i(cx.lpp(247).add(3828)) != 0)) || (cx.i(cx.lpp(247).add(3820)) != 0)) || (cx.f(cx.lpp(247).add(3632)) < cx.gc.SS_necessary_accumulated_effect)) || (cx.gc.SS_effect_value_must_drop_to_to_start_acceleration < cx.f(cx.lpp(247).add(68)))) {
        if ((cx.i(cx.lpp(247).add(3612)) != 0) && (((cx.f(cx.lpp(246).add(36)) == 0.0f32) || (cx.gc.SS_passive_accumulated_effect_depletion >= cx.f(cx.lpp(247).add(3632)))) || (cx.i(cx.lpp(247).add(3620)) > cx.gc.SS_max_boost_time))) {
            { let p = cx.lpp(247).add(3612); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3632); let t = 0; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3620); let t = 1; cx.set_i(p, t); }
        }
    } else {
        { let p = cx.lpp(247).add(3612); let t = 1; cx.set_i(p, t); }
        { let p = cx.lpp(247).add(3620); let t = 1; cx.set_i(p, t); }
    }
    if ((cx.i(cx.lpp(247).add(3612)) != 0) && ((cx.gc.SS_fade_in_time + 2.0f32) > (cx.i(cx.lpp(247).add(3620)) as f32))) {
        if ((cx.gc.SS_fade_in_time < (cx.i(cx.lpp(247).add(3620)) as f32)) || (cx.gc.SS_fade_in_time == 0.0f32)) {
            { let t = 1.0f32; cx.set_lf(400, t); }
        } else {
            { let t = ((((cx.i(cx.lpp(247).add(3620)) as f32) as f64) / (cx.gc.SS_fade_in_time as f64)) as f32); cx.set_lf(400, t); }
            if (cx.f(cx.lpp(246).add(36)) != 0.0f32) {
                if (cx.f(cx.lpp(247).add(3632)) <= cx.gc.SS_effect_for_speed_scale) {
                    xs = ((((((cx.gc.SS_speed_scale - 1.0f32) * cx.f(cx.lpp(247).add(3704))) / (cx.gc.SS_fade_in_time + 1.0f32)) as f64) * ((cx.f(cx.lpp(247).add(3632)) / cx.gc.SS_effect_for_speed_scale) as f64)) as f32);
                    v70 = { let q0_ = cx.lpp(247).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                    { let q0_ = cx.lpp(247).add(272); let q1_ = v70; let q2_ = xs; f_1c01b0(cx, q0_, q1_, q2_) };
                } else {
                    xr = (((((cx.gc.SS_speed_scale - 1.0f32) * cx.f(cx.lpp(247).add(3704))) as f64) / ((cx.gc.SS_fade_in_time + 1.0f32) as f64)) as f32);
                    v69 = { let q0_ = cx.lpp(247).add(240); let q1_ = 0; ov_car_rot_row(cx, q0_, q1_) };
                    { let q0_ = cx.lpp(247).add(272); let q1_ = v69; let q2_ = xr; f_1c01b0(cx, q0_, q1_, q2_) };
                }
            }
        }
        v71 = { let q0_ = cx.gc.timestep; let q1_ = (((cx.gc.timestep as f64) * (cx.gc.SS_timestep_multiplier as f64)) as f32); let q2_ = cx.lf(400); f_1d9c90(cx, q0_, q1_, q2_) };
        { let p = cx.ptr_at(cx.lpp(247).add(3552)); let t = (v71 as f32); cx.set_f(p, t); }
        v72 = { let q0_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(272)); let q1_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(272)) as f64) * (cx.gc.SS_sensitivities_multiplier as f64)) as f32); let q2_ = cx.lf(400); f_1d9c90(cx, q0_, q1_, q2_) };
        { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(148); let t = (v72 as f32); cx.set_f(p, t); }
        v73 = { let q0_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(264)); let q1_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(264)) as f64) * (cx.gc.SS_engine_power_multiplier as f64)) as f32); let q2_ = cx.lf(400); f_1d9c90(cx, q0_, q1_, q2_) };
        { let p = cx.lpp(247).add(3220); let t = (v73 as f32); cx.set_f(p, t); }
        v74 = { let q0_ = 1.0f32; let q1_ = cx.gc.SS_max_speed_multiplier; let q2_ = cx.lf(400); f_1d9c90(cx, q0_, q1_, q2_) };
        { let p = cx.lpp(247).add(3260); let t = (v74 as f32); cx.set_f(p, t); }
        v43 = { let q0_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(236)); let q1_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(236)) as f64) * (cx.gc.SS_max_speed_multiplier as f64)) as f32); let q2_ = cx.lf(400); f_1d9c90(cx, q0_, q1_, q2_) };
        { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(80); let t = (v43 as f32); cx.set_f(p, t); }
    } else if ((cx.i(cx.lpp(247).add(3612)) != 0) || ((cx.gc.SS_fade_out_time + 2.0f32) <= (cx.i(cx.lpp(247).add(3620)) as f32))) {
        if ((cx.i(cx.lpp(247).add(3612)) == 0) && ((cx.gc.SS_fade_out_time + 3.0f32) > (cx.i(cx.lpp(247).add(3620)) as f32))) {
            { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(80); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(236)); cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3260); let t = 1065353216; cx.set_i(p, t); }
            { let p = cx.lpp(247).add(3220); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(264)); cx.set_i(p, t); }
            { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(148); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(272)); cx.set_i(p, t); }
            { let p = cx.ptr_at(cx.lpp(247).add(3552)); let t = (cx.gc.timestep.to_bits() as i32); cx.set_i(p, t); }
        }
    } else {
        if ((cx.gc.SS_fade_out_time < (cx.i(cx.lpp(247).add(3620)) as f32)) || (cx.gc.SS_fade_out_time == 0.0f32)) {
            { let t = 1.0f32; cx.set_lf(396, t); }
        } else {
            { let t = ((((cx.i(cx.lpp(247).add(3620)) as f32) as f64) / (cx.gc.SS_fade_out_time as f64)) as f32); cx.set_lf(396, t); }
        }
        v75 = { let q0_ = (((cx.gc.timestep as f64) * (cx.gc.SS_timestep_multiplier as f64)) as f32); let q1_ = cx.gc.timestep; let q2_ = cx.lf(396); f_1d9c90(cx, q0_, q1_, q2_) };
        { let p = cx.ptr_at(cx.lpp(247).add(3552)); let t = (v75 as f32); cx.set_f(p, t); }
        v76 = { let q0_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(272)) as f64) * (cx.gc.SS_sensitivities_multiplier as f64)) as f32); let q1_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(272)); let q2_ = cx.lf(396); f_1d9c90(cx, q0_, q1_, q2_) };
        { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(148); let t = (v76 as f32); cx.set_f(p, t); }
        v77 = { let q0_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(264)) as f64) * (cx.gc.SS_engine_power_multiplier as f64)) as f32); let q1_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(264)); let q2_ = cx.lf(396); f_1d9c90(cx, q0_, q1_, q2_) };
        { let p = cx.lpp(247).add(3220); let t = (v77 as f32); cx.set_f(p, t); }
        v78 = { let q0_ = cx.gc.SS_max_speed_multiplier; let q1_ = 1.0f32; let q2_ = cx.lf(396); f_1d9c90(cx, q0_, q1_, q2_) };
        { let p = cx.lpp(247).add(3260); let t = (v78 as f32); cx.set_f(p, t); }
        v43 = { let q0_ = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(236)) as f64) * (cx.gc.SS_max_speed_multiplier as f64)) as f32); let q1_ = cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(236)); let q2_ = cx.lf(396); f_1d9c90(cx, q0_, q1_, q2_) };
        { let p = cx.ptr_at(cx.lpp(247).add(3552)).add(80); let t = (v43 as f32); cx.set_f(p, t); }
    }
    if (cx.i(cx.lpp(247).add(3620)) < 10000) {
        let _ = cx.preinc_i(cx.lpp(247).add(3620));
    }
    if ((cx.i(cx.lpp(247).add(3612)) != 0) && (cx.f(cx.lpp(247).add(3632)) > 0.0f32)) {
        { let p = cx.lpp(247).add(3632); let t = (((cx.f(cx.lpp(247).add(3632)) as f64) - (cx.gc.SS_active_accumulated_effect_depletion as f64)) as f32); cx.set_f(p, t); }
        if (cx.f(cx.lpp(247).add(3632)) < 0.0f32) {
            { let p = cx.lpp(247).add(3632); let t = 0; cx.set_i(p, t); }
        }
    }
    if (cx.b(cx.lpp(247).add(4072)) != 0) {
        { let p = cx.lpp(246).add(16); let t = 0; cx.set_i(p, t); }
        if (cx.f(cx.lpp(247).add(3712)) > (cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3552)).add(736))) / 2.0f32)) {
            { let p = cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3556))).add(52); let t = cx.i(cx.lpp(247).add(3712)); cx.set_i(p, t); }
            { let p = cx.lpp(246).add(16); let t = 1; cx.set_i(p, t); }
        }
        if (cx.f(cx.lpp(247).add(3752)) > (cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3552)).add(736)).add(8)) / 2.0f32)) {
            { let p = cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3556))).add(60); let t = cx.i(cx.lpp(247).add(3752)); cx.set_i(p, t); }
            { let p = cx.lpp(246).add(16); let t = 1; cx.set_i(p, t); }
        }
        if (cx.i(cx.lpp(246).add(16)) != 0) {
            { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3556))); f_1da870(cx, q0_) };
        } else if ((cx.i(cx.lpp(247).add(3572)) > 0) && (cx.i(cx.lpp(245).add(16)) != 0)) {
            { let p = cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3556))).add(52); let t = (((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3552)).add(736))) as f64) / 2.0f64) as f32); cx.set_f(p, t); }
            { let p = cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3556))).add(60); let t = (((cx.f(cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3552)).add(736)).add(8)) as f64) / 2.0f64) as f32); cx.set_f(p, t); }
            { let q0_ = cx.ptr_at(cx.ptr_at(cx.lpp(247).add(3556))); f_1da870(cx, q0_) };
        }
    }
    { let q0_ = cx.lpp(247); f_1ccad0(cx, q0_) };
    if ((cx.gc.PL_landing_orientation_disturbance_length > 0.0f32) && ((cx.b(cx.lpp(247).add(4536)) != 0) || (cx.i(cx.lpp(246).add(128)) != 0))) {
        if (cx.b(cx.lpp(247).add(4536)) != 0) {
            { let p = cx.lpp(247).add(4536); let t = 0; cx.set_b(p, t); }
            if (cx.gc.PL_landing_camera_tau_multiplier_duration != 0) {
                { let p = cx.lpp(247).add(3840); let t = cx.gc.PL_landing_camera_tau_multiplier_duration; cx.set_i(p, t); }
            }
        }
        if (cx.i(cx.lpp(246).add(128)) == 0) {
            { let q0_ = cx.lpp(247).add(4556); let q1_ = -1.0f32; let q2_ = 1.0f32; f_1bfe80(cx, q0_, q1_, q2_) };
            xt = (vsmath::acos(cx.f(cx.lpp(247).add(4556))) as f32);
            v79 = vsmath::fabs(xt);
            { let t = ((v79 + v79) as f32); cx.set_lf(388, t); }
            { let q0_ = cx.lptr(388); f_1d1080(cx, q0_) };
            { let q0_ = cx.lptr(368); let q1_ = cx.lpp(247).add(4544); f_1bb190(cx, q0_, q1_) };
            { let t = 0.0f32; cx.set_lf(372, t); }
            { let q0_ = cx.lptr(368); f_1c0380(cx, q0_) };
            { let t = (((f_1cf090(cx) as f64) * (cx.gc.PL_landing_orientation_disturbance_magnitude as f64)) as f32); cx.set_lf(360, t); }
            { let t = ((vsmath::fabs(cx.f(cx.lpp(247).add(4540))) / (cx.gc.PL_landing_orientation_disturbance_referecne_speed_up as f64)) as f32); cx.set_lf(352, t); }
            { let q0_ = cx.lptr(352); let q1_ = 0.0f32; let q2_ = 1.0f32; f_1bfe80(cx, q0_, q1_, q2_) };
            { let t = (((cx.lf(360) as f64) * (cx.lf(352) as f64)) as f32); cx.set_lf(360, t); }
            { let q0_ = Ptr::NULL.bits(); let q1_ = cx.lpp(247).add(4560); let q2_ = cx.lptr(368); let q3_ = cx.lf(360); f_1ceab0(cx, q0_, q1_, q2_, q3_) };
            { let q0_ = Ptr::NULL.bits(); let q1_ = cx.lpp(247).add(320); let q2_ = cx.lptr(368); let q3_ = cx.lf(360); f_1ceab0(cx, q0_, q1_, q2_, q3_) };
            { let _ = { let q0_ = cx.lpp(247).add(240); vcall(cx, 32, q0_, &[]) }; };
        }
        if ((cx.preinc_i(cx.lpp(246).add(128)) as f32) > cx.gc.PL_landing_orientation_disturbance_length) {
            { let p = cx.lpp(246).add(128); let t = 0; cx.set_i(p, t); }
            { let q0_ = cx.lpp(247).add(4544); f_1ba1b0(cx, q0_) };
            { let p = cx.lpp(247).add(4556); let t = 1065353216; cx.set_i(p, t); }
            { let q0_ = cx.lpp(247).add(4560); f_1ba1b0(cx, q0_) };
            { let p = cx.lpp(247).add(4572); let t = 1065353216; cx.set_i(p, t); }
        }
    }
    if (cx.i(cx.lpp(247).add(3840)) != 0) {
        let _ = cx.predec_i(cx.lpp(247).add(3840));
    }
    if ((cx.i(cx.lpp(246).add(96)) == 0) && (cx.i(cx.lpp(247).add(4756)) != 1)) {
        if ((cx.i(cx.lpp(246).add(92)) != 0) || (cx.i(cx.lpp(247).add(4716)) == 1)) {
            { let q0_ = cx.lpp(247).add(248); let q1_ = Ptr::gc(12); f_1bb190(cx, q0_, q1_) };
            { let p = cx.lpp(247).add(264); let t = cx.gc.car_linear_damping_when_in_air; cx.set_f(p, t); }
            if (((cx.i(cx.lpp(247).add(3840)) == 0) || (cx.i(cx.lpp(247).add(3604)) >= 4)) || (cx.i(cx.lpp(247).add(4756)) == 1)) {
                { let p = cx.lpp(247).add(268); let t = cx.gc.car_angular_damping_when_in_air; cx.set_f(p, t); }
            } else {
                { let p = cx.lpp(247).add(268); let t = cx.gc.car_angular_damping_after_landings; cx.set_f(p, t); }
            }
        } else {
            { let q0_ = cx.lpp(247).add(248); let q1_ = Ptr::gc(0); f_1bb190(cx, q0_, q1_) };
            { let p = cx.lpp(247).add(264); let t = cx.gc.car_linear_damping; cx.set_f(p, t); }
            if (cx.i(cx.lpp(247).add(3840)) != 0) {
                car_angular_damping_after_landings = cx.gc.car_angular_damping_after_landings;
            } else {
                car_angular_damping_after_landings = cx.gc.car_angular_damping;
            }
            { let p = cx.lpp(247).add(268); let t = car_angular_damping_after_landings; cx.set_f(p, t); }
        }
    }
    v81 = { let q0_ = cx.lpp(247).add(240); ov_car_velocity(cx, q0_) };
    v82 = (f_1c0330(cx, v81) as f64);
    { let p = cx.lpp(247).add(20); let t = (v82 as f32); cx.set_f(p, t); }
    if (cx.i(cx.lpp(247).add(3604)) != 0) {
        if (cx.i(cx.lpp(247).add(3608)) >= cx.gc.min_time_without_wheel_contact_to_match_wheel_rotation_to_car_speed) {
            { let t = 0.0f32; cx.set_lf(344, t); }
            'lp30: loop {
                if !(cx.li(344) < 4) { break 'lp30; }
                'blp30: {
                    xu = cx.li(344);
                    { let t = (-cx.f(cx.lpp(247).add(3704))); cx.set_lf(8, t); }
                    v83 = ({ let q0_ = cx.lpp(247).add(240); let q1_ = cx.li(344); f_35fb40(cx, q0_, q1_) } as f64);
                    v102 = (((cx.lf(8) as f64) / v83) as f32);
                    { let q0_ = cx.lpp(247).add(240); let q1_ = (v102.to_bits() as i32); let q2_ = xu; f_361530(cx, q0_, q1_, q2_) };
                }
                let _ = cx.preinc_i(cx.lptr(344));
            }
            if (((cx.gc.time_to_reduce_thrust != 0) && (cx.i(cx.lpp(247).add(3800)) > cx.gc.time_to_reduce_thrust)) && ((cx.f(cx.lpp(247).add(3704)) < 0.0f32) || ({ v84 = vsmath::fabs(cx.f(cx.lpp(246).add(88))); v84 } > ((cx.f(cx.lpp(247).add(3704)) as f64) * (cx.gc.tan_of_critical_angle as f64))))) {
                { let p = cx.lpp(246).add(108); let t = cx.gc.time_to_reduce_thrust; cx.set_i(p, t); }
            }
        }
        { let p = cx.lpp(247).add(3608); let t = 0; cx.set_i(p, t); }
    } else {
        let _ = cx.preinc_i(cx.lpp(247).add(3608));
    }
    if ((cx.gc.CSM_skid_angle_conversion_factor != 0.0f32) && (cx.gc.CSM_omega_conversion_factor != 0.0f32)) {
        if ((((cx.f(cx.lpp(247).add(3784)) <= cx.gc.CSM_min_speed) || (cx.i(cx.lpp(246).add(44)) == 0)) || (cx.i(cx.lpp(246).add(76)) != 0)) || (cx.i(cx.lpp(246).add(64)) != 0)) {
            { let p = cx.lpp(247).add(4456); let t = 0; cx.set_i(p, t); }
        } else {
            if (cx.f(cx.lpp(247).add(3772)) <= 0.0f32) {
                { let t = -1.0f32; cx.set_lf(4, t); }
            } else {
                { let t = 1.0f32; cx.set_lf(4, t); }
            }
            { let t = cx.lf(4); cx.set_lf(340, t); }
            xv = (vsmath::fabs(cx.f(cx.lpp(247).add(3772))) as f32);
            v85 = vsmath::sqrt(xv);
            { let p = cx.lpp(247).add(4456); let t = (((v85 * (cx.lf(340) as f64)) * (cx.gc.CSM_skid_angle_conversion_factor as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lpp(247).add(4456); let t = ((((cx.gc.CSM_omega_conversion_factor * cx.f(cx.lpp(247).add(3760))) as f64) + (cx.f(cx.lpp(247).add(4456)) as f64)) as f32); cx.set_f(p, t); }
        }
        if (cx.gc.CSM_tau_no_steer > 0.0f32) {
            { let t = cx.ptr_at(cx.lpp(247).add(3552)); cx.set_lpp(84, t); }
            { let t = cx.f(cx.lpp(247).add(4456)); cx.set_lf(332, t); }
            if (cx.f(cx.lpp(246).add(68)) == 0.0f32) {
                { let t = cx.gc.CSM_tau_no_steer; cx.set_lf(328, t); }
            } else {
                { let t = cx.gc.CSM_tau_some_steer_input; cx.set_lf(328, t); }
            }
            { let t = (((cx.f(cx.lpp(247).add(4460)) as f64) - (cx.lf(332) as f64)) as f32); cx.set_lf(324, t); }
            { let t = cx.f(cx.lpp(247).add(4464)); cx.set_lf(320, t); }
            { let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552))) as f64) / (cx.lf(328) as f64)) as f32); cx.set_lf(316, t); }
            { let t = (vsmath::exp(((0.0f64 - (cx.lf(316) as f64)) as f32)) as f32); cx.set_lf(312, t); }
            { let t = ((((((cx.lf(316) + 1.0f32) * cx.lf(324)) + (cx.lf(320) * cx.f(cx.lpp(84)))) as f64) * (cx.lf(312) as f64)) as f32); cx.set_lf(308, t); }
            { let p = cx.lpp(247).add(4464); let t = ((((((1.0f32 - cx.lf(316)) * cx.lf(320)) - ((cx.lf(324) * cx.lf(316)) / cx.lf(328))) as f64) * (cx.lf(312) as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lpp(247).add(4460); let t = ((((cx.lf(308) - cx.lf(324)) as f64) + (cx.f(cx.lpp(247).add(4460)) as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lpp(247).add(4456); let t = cx.i(cx.lpp(247).add(4460)); cx.set_i(p, t); }
        }
        { let t = (vsmath::cos(cx.gc.CSM_camera_motion_angle) as f32); cx.set_lf(0, t); }
        v86 = vsmath::fabs(cx.f(cx.lpp(247).add(4456)));
        { let p = cx.lpp(247).add(4468); let t = ((v86 * (cx.lf(0) as f64)) as f32); cx.set_f(p, t); }
        { let p = cx.lpp(247).add(4472); let t = 0; cx.set_i(p, t); }
        v87 = vsmath::sin(cx.gc.CSM_camera_motion_angle);
        { let p = cx.lpp(247).add(4476); let t = ((v87 * (cx.f(cx.lpp(247).add(4456)) as f64)) as f32); cx.set_f(p, t); }
        { let p = cx.lpp(247).add(4480); let t = (((cx.f(cx.lpp(247).add(4456)) as f64) * (cx.gc.CSM_extra_yaw_conversion as f64)) as f32); cx.set_f(p, t); }
    }
    if (cx.gc.detached_camera_enabled == 0.0f32) {
        { let p = cx.lpp(247).add(4484); let t = 1065353216; cx.set_i(p, t); }
        { let p = cx.lpp(247).add(4504); let t = 0; cx.set_i(p, t); }
        { let p = cx.lpp(247).add(4516); let t = 0; cx.set_i(p, t); }
    } else {
        if (cx.i(cx.lpp(246).add(12)) != 0) {
            { let p = cx.lpp(246).add(104); let t = 1; cx.set_b(p, t); }
        }
        if (((((cx.b(cx.lpp(246).add(104)) != 0) && (cx.i(cx.lpp(246).add(44)) != 0)) && (cx.f(cx.lpp(246).add(36)) != 0.0f32)) && (cx.gc.max_omega_to_attach_camera_after_jturn > cx.f(cx.lpp(247).add(3764)))) && (cx.f(cx.lpp(246).add(100)) == 0.0f32)) {
            { let p = cx.lpp(246).add(100); let t = cx.gc.camera_attachment_factor_after_jturn; cx.set_f(p, t); }
            { let p = cx.lpp(246).add(105); let t = 0; cx.set_b(p, t); }
            { let p = cx.lpp(246).add(104); let t = 0; cx.set_b(p, t); }
        }
        if (cx.i(cx.lpp(246).add(76)) != 0) {
            { let p = cx.lpp(246).add(105); let t = 1; cx.set_b(p, t); }
        }
        if (((((cx.b(cx.lpp(246).add(105)) != 0) && (cx.i(cx.lpp(246).add(44)) != 0)) && (cx.f(cx.lpp(246).add(36)) != 0.0f32)) && (cx.gc.max_omega_to_attach_camera_after_doughnut > cx.f(cx.lpp(247).add(3764)))) && (cx.f(cx.lpp(246).add(100)) == 0.0f32)) {
            { let p = cx.lpp(246).add(100); let t = cx.gc.camera_attachment_factor_after_doughnut; cx.set_f(p, t); }
            { let p = cx.lpp(246).add(105); let t = 0; cx.set_b(p, t); }
            { let p = cx.lpp(246).add(104); let t = 0; cx.set_b(p, t); }
        }
        if ((cx.f(cx.lpp(246).add(48)) != 0.0f32) && (cx.f(cx.lpp(246).add(100)) != 0.0f32)) {
            { let p = cx.lpp(246).add(100); let t = 0; cx.set_i(p, t); }
        }
        if ((((cx.f(cx.lpp(247).add(4488)) == 0.0f32) && (cx.f(cx.lpp(246).add(36)) == 0.0f32)) && (cx.f(cx.lpp(246).add(164)) == 0.0f32)) && (cx.f(cx.lpp(246).add(72)) < 0.1f32)) {
            let _ = cx.preinc_i(cx.lpp(246).add(116));
        } else {
            { let p = cx.lpp(246).add(116); let t = 0; cx.set_i(p, t); }
        }
        if (cx.i(cx.lpp(246).add(116)) > cx.gc.idle_input_realignment_time) {
            { let p = cx.lpp(246).add(120); let t = cx.gc.alignement_duration; cx.set_i(p, t); }
        }
        if (cx.i(cx.lpp(246).add(120)) > 0) {
            if (((cx.f(cx.lpp(246).add(36)) == 0.0f32) && (cx.f(cx.lpp(246).add(164)) == 0.0f32)) && (cx.f(cx.lpp(246).add(72)) < 0.1f32)) {
                let _ = cx.predec_i(cx.lpp(246).add(120));
            } else {
                { let p = cx.lpp(246).add(120); let t = 0; cx.set_i(p, t); }
            }
            if (cx.i(cx.lpp(246).add(120)) == 0) {
                { let p = cx.lpp(246).add(116); let t = -1000; cx.set_i(p, t); }
            }
        }
        if ((cx.i(cx.lpp(246).add(44)) == 0) && (cx.i(cx.lpp(245).add(44)) != 0)) {
            { let p = cx.lpp(246).add(196); let t = cx.gc.time_to_move_out_the_camera_after_detachment.wrapping_add(1); cx.set_i(p, t); }
        }
        if (cx.b(cx.lpp(246).add(112)) != 0) {
            if (cx.i(cx.lpp(247).add(3604)) == 4) {
                { let p = cx.lpp(246).add(112); let t = 0; cx.set_b(p, t); }
            }
        } else if ((cx.i(cx.lpp(247).add(3604)) == 0) && (({ let q0_ = cx.lpp(247).add(288); f_1c0330(cx, q0_) } as f64) > (cx.gc.min_omega_mag_to_detach_on_tumbling as f64))) {
            { let p = cx.lpp(246).add(112); let t = 1; cx.set_b(p, t); }
        }
        if ((((((cx.f(cx.lpp(247).add(20)) <= cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(228))) || (cx.i(cx.lpp(246).add(44)) == 0)) && (cx.f(cx.lpp(246).add(100)) == 0.0f32)) || (cx.b(cx.lpp(246).add(112)) != 0)) || (cx.b(cx.lpp(246).add(105)) != 0)) || (cx.b(cx.lpp(246).add(104)) != 0)) {
            if (((((cx.f(cx.lpp(247).add(20)) <= cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(224))) || (cx.i(cx.lpp(246).add(44)) == 0)) || (cx.b(cx.lpp(246).add(112)) != 0)) || (cx.b(cx.lpp(246).add(105)) != 0)) || (cx.b(cx.lpp(246).add(104)) == 0)) {
                { let p = cx.lpp(247).add(4488); let t = 0; cx.set_i(p, t); }
                { let p = cx.lpp(246).add(100); let t = 0; cx.set_i(p, t); }
            } else {
                { let p = cx.lpp(247).add(4488); let t = ((((cx.f(cx.lpp(247).add(20)) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(224))) as f64) / ((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(228)) - cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(224))) as f64)) as f32); cx.set_f(p, t); }
            }
        } else {
            if (cx.f(cx.lpp(246).add(100)) == 0.0f32) {
                { let p = cx.lpp(247).add(4488); let t = 1065353216; cx.set_i(p, t); }
            } else {
                { let p = cx.lpp(247).add(4488); let t = cx.i(cx.lpp(246).add(100)); cx.set_i(p, t); }
            }
            { let p = cx.lpp(246).add(184); let t = 1065353216; cx.set_i(p, t); }
            if (((cx.f(cx.lpp(246).add(100)) != 0.0f32) && (cx.f(cx.lpp(247).add(20)) > cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(228)))) && (cx.i(cx.lpp(246).add(44)) != 0)) {
                { let p = cx.lpp(246).add(100); let t = 0; cx.set_i(p, t); }
            }
        }
        { let p = cx.lpp(246).add(184); let t = cx.i(cx.lpp(247).add(4488)); cx.set_i(p, t); }
        if ((cx.f(cx.lpp(246).add(184)) != 1.0f32) && (cx.f(cx.lpp(245).add(184)) == 1.0f32)) {
            { let p = cx.lpp(246).add(188); let t = ((cx.i(cx.lpp(247).add(4856)) < 3) as i32); cx.set_b(p, t); }
        }
        if (cx.f(cx.lpp(246).add(184)) == 0.0f32) {
            { let p = cx.lpp(246).add(192); let t = 0; cx.set_i(p, t); }
            let _ = cx.preinc_i(cx.lpp(246).add(196));
        } else {
            let _ = cx.preinc_i(cx.lpp(246).add(192));
            { let p = cx.lpp(246).add(196); let t = 0; cx.set_i(p, t); }
        }
        { let t = ((1.0f64 - (cx.f(cx.lpp(247).add(4488)) as f64)) as f32); cx.set_lf(304, t); }
        if (((((cx.b(cx.lpp(246).add(188)) != 0) && (cx.i(cx.lpp(246).add(44)) != 0)) || (cx.i(cx.lpp(246).add(196)) <= cx.gc.time_to_move_out_the_camera_after_detachment)) && ((cx.b(cx.lpp(246).add(188)) == 0) || (cx.i(cx.lpp(246).add(196)) <= cx.gc.time_to_move_out_the_camera_after_detachment_after_collision))) || (cx.gc.total_trail_increase <= cx.f(cx.lpp(247).add(4500)))) {
            if (cx.gc.time_to_move_in_the_camera_after_attachment < 0) {
                if ((cx.f(cx.lpp(247).add(4488)) != 0.0f32) && (cx.f(cx.lpp(247).add(4500)) > 0.0f32)) {
                    { let p = cx.lpp(247).add(4500); let t = (((cx.f(cx.lpp(247).add(4500)) as f64) - (cx.gc.trail_decrease_in_a_frame as f64)) as f32); cx.set_f(p, t); }
                    if (cx.f(cx.lpp(247).add(4500)) < 0.0f32) {
                        { let p = cx.lpp(247).add(4500); let t = 0; cx.set_i(p, t); }
                    }
                }
            } else if ((cx.i(cx.lpp(246).add(192)) > cx.gc.time_to_move_in_the_camera_after_attachment) && (cx.f(cx.lpp(247).add(4500)) > 0.0f32)) {
                { let p = cx.lpp(247).add(4500); let t = (((cx.f(cx.lpp(247).add(4500)) as f64) - (cx.gc.trail_decrease_in_a_frame as f64)) as f32); cx.set_f(p, t); }
                if (cx.f(cx.lpp(247).add(4500)) < 0.0f32) {
                    { let p = cx.lpp(247).add(4500); let t = 0; cx.set_i(p, t); }
                }
            }
        } else {
            { let p = cx.lpp(247).add(4500); let t = (((cx.f(cx.lpp(247).add(4500)) as f64) + (cx.gc.trail_increase_in_a_frame as f64)) as f32); cx.set_f(p, t); }
            if (cx.f(cx.lpp(247).add(4500)) > (cx.lf(304) * cx.gc.total_trail_increase)) {
                { let p = cx.lpp(247).add(4500); let t = (((cx.lf(304) as f64) * (cx.gc.total_trail_increase as f64)) as f32); cx.set_f(p, t); }
            }
        }
        if (cx.i(cx.lpp(246).add(120)) != 0) {
            { let p = cx.lpp(247).add(4488); let t = 1055286886; cx.set_i(p, t); }
        }
        { let q0_ = cx.lpp(247).add(4500); let q1_ = 0.0f32; let q2_ = 1.0f32; f_1bfe80(cx, q0_, q1_, q2_) };
        if (cx.gc.camera_trail_smoothout_tau <= 0.0f32) {
            { let p = cx.lpp(247).add(4504); let t = cx.i(cx.lpp(247).add(4500)); cx.set_i(p, t); }
        } else {
            { let t = cx.ptr_at(cx.lpp(247).add(3552)); cx.set_lpp(75, t); }
            { let t = cx.f(cx.lpp(247).add(4500)); cx.set_lf(296, t); }
            { let t = (((cx.f(cx.lpp(247).add(4508)) as f64) - (cx.lf(296) as f64)) as f32); cx.set_lf(292, t); }
            { let t = cx.gc.camera_trail_smoothout_tau; cx.set_lf(288, t); }
            { let t = cx.f(cx.lpp(247).add(4512)); cx.set_lf(284, t); }
            { let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552))) as f64) / (cx.lf(288) as f64)) as f32); cx.set_lf(280, t); }
            { let t = (vsmath::exp(((0.0f64 - (cx.lf(280) as f64)) as f32)) as f32); cx.set_lf(276, t); }
            { let t = ((((((cx.lf(280) + 1.0f32) * cx.lf(292)) + (cx.lf(284) * cx.f(cx.lpp(75)))) as f64) * (cx.lf(276) as f64)) as f32); cx.set_lf(272, t); }
            { let p = cx.lpp(247).add(4512); let t = ((((((1.0f32 - cx.lf(280)) * cx.lf(284)) - ((cx.lf(292) * cx.lf(280)) / cx.lf(288))) as f64) * (cx.lf(276) as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lpp(247).add(4508); let t = ((((cx.lf(272) - cx.lf(292)) as f64) + (cx.f(cx.lpp(247).add(4508)) as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lpp(247).add(4504); let t = cx.i(cx.lpp(247).add(4508)); cx.set_i(p, t); }
        }
        if (cx.gc.camera_factor_smoothout_tau <= 0.0f32) {
            { let p = cx.lpp(247).add(4484); let t = cx.i(cx.lpp(247).add(4488)); cx.set_i(p, t); }
        } else {
            { let t = cx.ptr_at(cx.lpp(247).add(3552)); cx.set_lpp(67, t); }
            { let t = cx.f(cx.lpp(247).add(4488)); cx.set_lf(264, t); }
            { let t = (((cx.f(cx.lpp(247).add(4492)) as f64) - (cx.lf(264) as f64)) as f32); cx.set_lf(260, t); }
            { let t = cx.gc.camera_factor_smoothout_tau; cx.set_lf(256, t); }
            { let t = cx.f(cx.lpp(247).add(4496)); cx.set_lf(252, t); }
            { let t = (((cx.f(cx.ptr_at(cx.lpp(247).add(3552))) as f64) / (cx.lf(256) as f64)) as f32); cx.set_lf(248, t); }
            { let t = (vsmath::exp(((0.0f64 - (cx.lf(248) as f64)) as f32)) as f32); cx.set_lf(244, t); }
            { let t = ((((((cx.lf(248) + 1.0f32) * cx.lf(260)) + (cx.lf(252) * cx.f(cx.lpp(67)))) as f64) * (cx.lf(244) as f64)) as f32); cx.set_lf(240, t); }
            { let p = cx.lpp(247).add(4496); let t = ((((((1.0f32 - cx.lf(248)) * cx.lf(252)) - ((cx.lf(260) * cx.lf(248)) / cx.lf(256))) as f64) * (cx.lf(244) as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lpp(247).add(4492); let t = ((((cx.lf(240) - cx.lf(260)) as f64) + (cx.f(cx.lpp(247).add(4492)) as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lpp(247).add(4484); let t = cx.i(cx.lpp(247).add(4492)); cx.set_i(p, t); }
        }
        { let q0_ = cx.lpp(247).add(4484); let q1_ = 0.0f32; let q2_ = 1.0f32; f_1bfe80(cx, q0_, q1_, q2_) };
        if (cx.i(cx.lpp(246).add(120)) == 0) {
            { let p = cx.lpp(247).add(4516); let t = ((((cx.gc.total_pitch_increase * cx.f(cx.lpp(247).add(4504))) as f64) / (cx.gc.total_trail_increase as f64)) as f32); cx.set_f(p, t); }
        }
    }
    if ((cx.gc.cos_of_min_disable_skid_marks_angle != 0.0f32) && (cx.i(cx.lpp(246).add(56)) == 1)) {
        { let t = 0.0f32; cx.set_lf(236, t); }
        'lp31: loop {
            if !(cx.li(236) < 4) { break 'lp31; }
            'blp31: {
                { let t = ((((cx.f(cx.lpp(247).add((4_i32).wrapping_mul(cx.li(236))).add(2128)) * cx.f(cx.lpp(247).add(1624))) as f64) + (cx.f(cx.lpp(247).add(1612)) as f64)) as f32); cx.set_lf(228, t); }
                { let q0_ = cx.lptr(228); let q1_ = ((0.0f64 - (cx.f(cx.lpp(247).add(1620)) as f64)) as f32); let q2_ = cx.f(cx.lpp(247).add(1620)); f_1bfe80(cx, q0_, q1_, q2_) };
                { let q0_ = cx.lptr(204); let q1_ = cx.lf(228); let q2_ = cx.i(cx.lpp(247).add(2060)); let q3_ = cx.i(cx.lpp(247).add(2068)); let q4_ = { let q0_ = ((0.0f64 - (cx.f(cx.lpp(247).add(2076)) as f64)) as f32); ov_coerce_int(cx, q0_) }; f_1d9cf0(cx, q0_, q1_, q2_, q3_, q4_) };
                { let q0_ = cx.lptr(184); let q1_ = cx.lptr(204); let q2_ = cx.lpp(247).add(352); f_1c04e0(cx, q0_, q1_, q2_) };
                v88 = { let q0_ = cx.lpp(247).add(240); ov_car_velocity(cx, q0_) };
                { let t = f_1c0330(cx, v88); cx.set_lf(176, t); }
                if (cx.gc.draw_visual_steering_direction != 0) {
                    { let q0_ = cx.lpp(247).add(240); let q1_ = cx.lptr(140); let q2_ = cx.li(236); f_361390(cx, q0_, q1_, q2_) };
                    { let q0_ = cx.lptr(160); let q1_ = cx.lptr(140); let q2_ = cx.lptr(184); let q3_ = 3.0f32; f_1c0210(cx, q0_, q1_, q2_, q3_) };
                    xw = { let p = { let q0_ = cx.lptr(76); let q1_ = 0; let q2_ = 0; let q3_ = 0; let q4_ = 0; ov_colour_xbox(cx, q0_, q1_, q2_, q3_, q4_) }; cx.i(p) };
                    v103 = cx.i(Ptr::glob(0x77FD9C));
                    v97 = { let q0_ = cx.lptr(60); let q1_ = cx.li(160); let q2_ = cx.li(164); let q3_ = cx.li(168); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    v89 = { let q0_ = cx.lptr(44); let q1_ = cx.li(140); let q2_ = cx.li(144); let q3_ = cx.li(148); ov_vec4_xyz(cx, q0_, q1_, q2_, q3_) };
                    crate::ov_ignore!(cx, Ptr::glob(0x7814E0), v89, v97, v103, xw);
                }
                { let p = cx.lpp(247).add(cx.li(236)).add(24); let t = (((cx.lf(176) != 0.0f32) && { let _ = { v90 = { let q0_ = cx.lpp(247).add(240); ov_car_velocity(cx, q0_) }; v90 }; (({ v91 = { let q0_ = v90; let q1_ = cx.lptr(184); f_1c0300(cx, q0_, q1_) }; v91 } / (cx.lf(176) as f64)) > (cx.gc.cos_of_min_disable_skid_marks_angle as f64)) }) as i32); cx.set_b(p, t); }
            }
            { let t = cx.li(236).wrapping_add(3); cx.set_li(236, t); }
        }
    }
    { let t = 0.0f32; cx.set_lf(132, t); }
    'lp32: loop {
        if !(cx.li(132) < 4) { break 'lp32; }
        'blp32: {
            { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(132))).add(1508); let t = (((cx.f(cx.lpp(247).add((4_i32).wrapping_mul(cx.li(132))).add(1508)) as f64) * (cx.f(cx.lpp(247).add(4912)) as f64)) as f32); cx.set_f(p, t); }
            { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(132))).add(1444); let t = (((cx.f(cx.lpp(247).add((4_i32).wrapping_mul(cx.li(132))).add(1444)) as f64) * (cx.f(cx.lpp(247).add(4904)) as f64)) as f32); cx.set_f(p, t); }
        }
        let _ = cx.preinc_i(cx.lptr(132));
    }
    if (cx.i(cx.lpp(246).add(108)) > 0) {
        { let t = (((((cx.gc.time_to_reduce_thrust as f32) - (cx.i(cx.lpp(246).add(108)) as f32)) as f64) / ((cx.gc.time_to_reduce_thrust as f32) as f64)) as f32); cx.set_lf(128, t); }
        { let t = 0.0f32; cx.set_lf(124, t); }
        'lp33: loop {
            if !(cx.li(124) < 4) { break 'lp33; }
            'blp33: {
                { let p = cx.lpp(247).add((4_i32).wrapping_mul(cx.li(124))).add(1444); let t = (((cx.f(cx.lpp(247).add((4_i32).wrapping_mul(cx.li(124))).add(1444)) as f64) * (cx.lf(128) as f64)) as f32); cx.set_f(p, t); }
            }
            let _ = cx.preinc_i(cx.lptr(124));
        }
        let _ = cx.predec_i(cx.lpp(246).add(108));
    }
    if (cx.i(cx.lpp(247).add(3604)) != 0) {
        { let t = 0.0f32; cx.set_lf(120, t); }
        { let t = 0.0f32; cx.set_lf(116, t); }
        'lp34: loop {
            if !(cx.li(116) < 4) { break 'lp34; }
            'blp34: {
                v92 = { let q0_ = cx.lpp(247).add(240); let q1_ = cx.li(116); f_3649d0(cx, q0_, q1_) };
                { let t = ((v92 + (cx.lf(120) as f64)) as f32); cx.set_lf(120, t); }
            }
            let _ = cx.preinc_i(cx.lptr(116));
        }
        { let t = (((cx.lf(120) as f64) / 4.0f64) as f32); cx.set_lf(120, t); }
        { let t = ((({ let q0_ = cx.lpp(247).add(240); let q1_ = 0; f_35fb40(cx, q0_, q1_) } as f64) / 3.0f64) as f32); cx.set_lf(112, t); }
        if (cx.lf(112) <= cx.lf(120)) {
            { let p = cx.lpp(247).add(1436); let t = cx.i(cx.ptr_at(cx.lpp(247).add(3552)).add(120)); cx.set_i(p, t); }
        } else {
            { let p = cx.lpp(247).add(1436); let t = ((((cx.f(cx.ptr_at(cx.lpp(247).add(3552)).add(120)) + cx.lf(112)) as f64) - (cx.lf(120) as f64)) as f32); cx.set_f(p, t); }
        }
        if (cx.i(cx.lpp(247).add(3828)) != 0) {
            { let p = cx.lpp(247).add(1436); let t = (((cx.f(cx.lpp(247).add(1436)) as f64) + (cx.gc.extra_zone_topple_correction as f64)) as f32); cx.set_f(p, t); }
        }
    }
    if (cx.gc.the_low_speed > cx.f(cx.lpp(247).add(3784))) {
        { let q0_ = cx.lpp(247).add(1508); let q1_ = 0.0f32; let q2_ = cx.gc.maximum_static_friction_ever_at_low_speed; f_1bfe80(cx, q0_, q1_, q2_) };
        { let q0_ = cx.lpp(247).add(1512); let q1_ = 0.0f32; let q2_ = cx.gc.maximum_static_friction_ever_at_low_speed; f_1bfe80(cx, q0_, q1_, q2_) };
        { let q0_ = cx.lpp(247).add(1516); let q1_ = 0.0f32; let q2_ = cx.gc.maximum_static_friction_ever_at_low_speed; f_1bfe80(cx, q0_, q1_, q2_) };
        { let q0_ = cx.lpp(247).add(1520); let q1_ = 0.0f32; let q2_ = cx.gc.maximum_static_friction_ever_at_low_speed; f_1bfe80(cx, q0_, q1_, q2_) };
    }
    if ((cx.i(cx.lpp(247).add(3740)) != 0) && (cx.f(cx.lpp(246).add(164)) != 0.0f32)) {
        { let q0_ = cx.lpp(247).add(1476); let q1_ = 1; f_1bfef0(cx, q0_, q1_) };
    }
    if (cx.i(cx.lpp(247).add(4860)) != 0) {
        let _ = cx.preinc_i(cx.lpp(247).add(4864));
    }
    if (cx.i(cx.lpp(247).add(3732)) > 0) {
        let _ = cx.predec_i(cx.lpp(247).add(3732));
    }
    if (cx.i(cx.lpp(247).add(3676)) > 0) {
        let _ = cx.predec_i(cx.lpp(247).add(3676));
    }
    let _ = cx.preinc_i(cx.lpp(247).add(4048));
    let _ = cx.preinc_i(cx.lpp(247).add(3800));
    if (cx.i(cx.lpp(247).add(4056)) != 0) {
        if (cx.predec_i(cx.lpp(247).add(4056)) != 0) {
            { let p = cx.lpp(247).add(4060); let t = ((((((cx.i(cx.lpp(247).add(4056)) as f32) * cx.gc.FB_roll_exaggeration_12_factor) / (cx.gc.FB_roll_exaggeration_time as f32)) as f64) + 1.0f64) as f32); cx.set_f(p, t); }
        } else {
            { let p = cx.lpp(247).add(4060); let t = 1065353216; cx.set_i(p, t); }
        }
    }
    if (cx.i(cx.lpp(247).add(4408)) != 0) {
        { let t = cx.ptr_at(cx.lpp(247).add(3552)); cx.set_lpp(27, t); }
        { let t = cx.lpp(247).add(4424); cx.set_lpp(26, t); }
        { let t = cx.lpp(247).add(4428); cx.set_lpp(25, t); }
        if ((cx.f(cx.lpp(247).add(4428)) > 0.0f32) && ((((1.0f32 - (1.0f32 / (cx.gc.CS_tau * cx.f(cx.lpp(247).add(4432))))) * cx.f(cx.lpp(25))) - ((cx.f(cx.lpp(26)) / (cx.gc.CS_tau * cx.f(cx.lpp(247).add(4432)))) / ((cx.gc.CS_tau * cx.f(cx.lpp(247).add(4432))) * cx.f(cx.lpp(27))))) > 0.0f32)) {
            { let t = (((cx.gc.CS_tau as f64) * (cx.f(cx.lpp(247).add(4432)) as f64)) as f32); cx.set_lf(96, t); }
        } else {
            { let t = (((cx.gc.CS_tau_back as f64) * (cx.f(cx.lpp(247).add(4432)) as f64)) as f32); cx.set_lf(96, t); }
        }
        { let t = ((1.0f64 / (cx.lf(96) as f64)) as f32); cx.set_lf(92, t); }
        { let t = (vsmath::exp(((0.0f64 - ((1.0f32 / cx.lf(96)) as f64)) as f32)) as f32); cx.set_lf(88, t); }
        { let t = ((((((cx.lf(92) + 1.0f32) * cx.f(cx.lpp(26))) + (cx.f(cx.lpp(25)) * cx.f(cx.lpp(27)))) as f64) * (cx.lf(88) as f64)) as f32); cx.set_lf(84, t); }
        { let t = ((((((1.0f32 - cx.lf(92)) * cx.f(cx.lpp(25))) - ((cx.f(cx.lpp(26)) * cx.lf(92)) / (cx.lf(96) * cx.f(cx.lpp(27))))) as f64) * (cx.lf(88) as f64)) as f32); cx.set_lf(80, t); }
        if ((cx.gc.CS_camera_diff_to_stop as f64) > vsmath::fabs(cx.lf(84))) {
            v93 = vsmath::fabs(cx.lf(80));
            if ((cx.gc.CS_camera_diff_to_stop as f64) > (v93 * (cx.f(cx.lpp(27)) as f64))) {
                { let p = cx.lpp(247).add(4408); let t = 1; cx.set_i(p, t); }
            }
        }
        { let p = cx.lpp(247).add(4424); let t = cx.lf(84); cx.set_f(p, t); }
        { let p = cx.lpp(247).add(4428); let t = cx.lf(80); cx.set_f(p, t); }
        { let q0_ = cx.lpp(247).add(4396); let q1_ = cx.lpp(247).add(4412); let q2_ = cx.lf(84); f_1c0160(cx, q0_, q1_, q2_) };
        if (cx.predec_i(cx.lpp(247).add(4408)) == 0) {
            { let q0_ = cx.lpp(247).add(4396); f_1ba1b0(cx, q0_) };
        }
    }
    v94 = (v94 & !0xFFFF_FFFFi64) | (({ let q0_ = cx.lpp(247).add(3560); ov_hist_push(cx, q0_) }) as i32 as u32 as i64);
    v104 = v94;
    return v104;
}

/// VehicleEntity__1D9A90 0x001D9A90
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d9a90(cx: &mut Cx, a1: f32, a2: f32) -> f64 {
    if (a1 <= a2) {
        return (a2 as f64);
    } else {
        return (a1 as f64);
    }
    #[allow(unreachable_code)]
    0.0
}

/// VehicleEntity__1D9AD0 0x001D9AD0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d9ad0(cx: &mut Cx, a1: f32, a2: f32) -> f64 {
    if (a2 <= a1) {
        return (a2 as f64);
    } else {
        return (a1 as f64);
    }
    #[allow(unreachable_code)]
    0.0
}

/// VehicleEntity__1D9B10 0x001D9B10
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d9b10(cx: &mut Cx, a1: f32, a2: f32, mut a3: f32, a4: f32) -> i32 {
    let mut v5 = 0.0f32;
    let mut v6 = 0.0f32;
    let mut v7 = 0.0f32;
    let mut v8 = 0.0f32;
    let mut v9 = 0.0f32;
    if (a3 == 0.000001f32) {
        a3 = 0.001f32;
    }
    v6 = (vsmath::fabs(a2) as f32);
    v5 = (vsmath::fabs(a1) as f32);
    v8 = (f_1d9a90(cx, v5, v6) as f32);
    v7 = ((vsmath::fabs(a4) * (v8 as f64)) as f32);
    v9 = ((vsmath::fabs(a3) + (v7 as f64)) as f32);
    return (((v9 as f64) >= vsmath::fabs((((a1 as f64) - (a2 as f64)) as f32))) as i32);
}

/// VehicleEntity__1D9BD0 0x001D9BD0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d9bd0(cx: &mut Cx, a1: f32, a2: f32, mut a3: f32, a4: f32) -> i32 {
    let mut v5 = 0.0f32;
    let mut v6 = 0.0f32;
    let mut v7 = 0.0f32;
    let mut v8 = 0.0f32;
    let mut v9 = 0.0f32;
    if (a3 == 0.000001f32) {
        a3 = 0.001f32;
    }
    v6 = (vsmath::fabs(a2) as f32);
    v5 = (vsmath::fabs(a1) as f32);
    v8 = (f_1d9a90(cx, v5, v6) as f32);
    v7 = ((vsmath::fabs(a4) * (v8 as f64)) as f32);
    v9 = ((vsmath::fabs(a3) + (v7 as f64)) as f32);
    return (((a1 - a2) >= (0.0f32 - v9)) as i32);
}

/// VehicleEntity__1D9C90 0x001D9C90
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d9c90(cx: &mut Cx, a1: f32, a2: f32, a3: f32) -> f64 {
    return ((((a2 as f64) - (a1 as f64)) * (a3 as f64)) + (a1 as f64));
}

/// VehicleEntity__1D9CB0 0x001D9CB0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d9cb0(cx: &mut Cx, a1: f32, a2: f32, hi: f32) -> f32 {
    return (((vsmath::clamp((((a2 as f64) - (a1 as f64)) as f32), ((0.0f64 - (hi as f64)) as f32), hi) as f64) + (a1 as f64)) as f32);
}

/// VehicleEntity__1D9CF0 0x001D9CF0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1d9cf0(cx: &mut Cx, mut a1: Ptr, a2: f32, a3: i32, a4: i32, a5: i32) -> i32 {
    let mut v6 = 0.0f32;
    let mut v7 = 0.0f32;
    let mut v8 = 0.0f32;
    let mut v9 = 0.0f32;
    let mut a5 = f32::from_bits(a5 as u32);
    let mut a4 = f32::from_bits(a4 as u32);
    let mut a3 = f32::from_bits(a3 as u32);
    v9 = (vsmath::sin(a2) as f32);
    v8 = (vsmath::cos(a2) as f32);
    v7 = ((((a5 + a3) as f64) * (v8 as f64)) as f32);
    v6 = ((1.0f64 / vsmath::sqrt(((((((4.0f32 * (a5 * a5)) + (a4 * a4)) + ((2.0f32 * v7) * (a4 * v9))) as f64) + ((((((0.0f32 - (a4 * a4)) - (3.0f32 * (a5 * a5))) + (((2.0f32 * a5) + a3) * a3)) * v8) * v8) as f64)) as f32))) as f32);
    { let p = a1; let t = (((((a4 * v9) + v7) as f64) * (v6 as f64)) as f32); cx.set_f(p, t); }
    { let p = a1.add(4); let t = 0; cx.set_i(p, t); }
    { let p = a1.add(8); let t = ((((v9 * v6) as f64) * ((2.0f32 * a5) as f64)) as f32); cx.set_f(p, t); }
    return a1.bits();
}

/// VehicleEntity__1DA110 0x001DA110
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da110(cx: &mut Cx, this: Ptr, a2: i32) -> Ptr {
    let _ = f_1da150(cx, this);
    if ((a2 & 1) != 0) {
        ov_delete(cx, this);
    }
    return this;
}

/// VehicleEntity__1DA150 0x001DA150
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da150(cx: &mut Cx, this: Ptr) -> i32 {
    { let q0_ = cx.ptr_at(this); ov_delete(cx, q0_) };
    return 0;
}

/// VehicleEntity__1DA280 0x001DA280
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da280(cx: &mut Cx, mut this: Ptr, a2: i32, a3: i32, a4: i32, a5: i32) -> Ptr {
    { let p = this; let t = a2; cx.set_i(p, t); }
    { let p = this.add(4); let t = a3; cx.set_i(p, t); }
    { let p = this.add(8); let t = a4; cx.set_i(p, t); }
    { let p = this.add(12); let t = a5; cx.set_i(p, t); }
    return this;
}

/// VehicleEntity__1DA550 0x001DA550
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da550(cx: &mut Cx, this: Ptr) -> Ptr {
    let _ = f_1b4690(cx, this);
    return this;
}

/// VehicleEntity__1DA680 0x001DA680
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da680(cx: &mut Cx, this: Ptr, mut a2: Ptr) -> Ptr {
    { let p = a2; let t = cx.i(this.add(48)); cx.set_i(p, t); }
    { let p = a2.add(4); let t = cx.i(this.add(52)); cx.set_i(p, t); }
    { let p = a2.add(8); let t = cx.i(this.add(56)); cx.set_i(p, t); }
    return a2;
}

/// VehicleEntity__1DA6C0 0x001DA6C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da6c0(cx: &mut Cx, this: Ptr) -> f64 {
    return ((cx.f(this.add(100)) as f64) / 24.0f64);
}

/// VehicleEntity__1DA6E0 0x001DA6E0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da6e0(cx: &mut Cx, this: Ptr, a2: f32) -> Ptr {
    { let p = this.add(96); let t = (((a2 as f64) * 24.0f64) as f32); cx.set_f(p, t); }
    return this;
}

/// VehicleEntity__1DA710 0x001DA710
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da710(cx: &mut Cx, this: Ptr) -> f64 {
    return ((cx.f(this.add(104)) as f64) / 24.0f64);
}

/// VehicleEntity__1DA730 0x001DA730
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da730(cx: &mut Cx, this: Ptr) -> f64 {
    return ((cx.f(this.add(108)) as f64) / 24.0f64);
}

/// VehicleEntity__1DA870 0x001DA870
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da870(cx: &mut Cx, this: Ptr) -> () {
    { let p = this.add(12); let t = f_1c0330(cx, this.add(52)); cx.set_f(p, t); }
}

/// VehicleEntity__1DA8B0 0x001DA8B0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da8b0(cx: &mut Cx, this: Ptr, a2: f32) -> Ptr {
    { let p = this.add(108); let t = (((a2 as f64) * 24.0f64) as f32); cx.set_f(p, t); }
    return this;
}

/// VehicleEntity__1DA8E0 0x001DA8E0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da8e0(cx: &mut Cx, this: Ptr, a2: i32, a3: i32) -> i32 {
    { let p = this.add(3529); let t = a2; cx.set_b(p, t); }
    { let p = this.add(3530); let t = a3; cx.set_b(p, t); }
    return a3;
}

/// VehicleEntity__1DA910 0x001DA910
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da910(cx: &mut Cx, this: Ptr) -> Ptr {
    let mut i = 0i32;
    i = 0;
    'lp2: loop {
        if !(i < 186) { break 'lp2; }
        'blp2: {
            { let p = this.add(4 * (i)); let t = 0; cx.set_i(p, t); }
        }
        { i += 1; i };
    }
    return this;
}

/// VehicleEntity__1DA960 0x001DA960
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da960(cx: &mut Cx, this: Ptr) -> i32 {
    return cx.i(this.add(140));
}

/// VehicleEntity__1DA980 0x001DA980
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da980(cx: &mut Cx, this: Ptr) -> f64 {
    return (cx.f(this.add(164)) as f64);
}

/// VehicleEntity__1DA9A0 0x001DA9A0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da9a0(cx: &mut Cx, this: Ptr) -> f64 {
    return (cx.f(this.add(2936)) as f64);
}

/// VehicleEntity__1DA9C0 0x001DA9C0
#[allow(unused_mut, unused_variables, unused_assignments, unreachable_code, clippy::all)]
pub fn f_1da9c0(cx: &mut Cx, this: Ptr, mut a2: Ptr, a3: i32) -> Ptr {
    { let p = a2; let t = cx.i(this.add(4 * ((4_i32).wrapping_mul(a3))).add(112)); cx.set_i(p, t); }
    { let p = a2.add(4); let t = cx.i(this.add(4 * ((4_i32).wrapping_mul(a3))).add(116)); cx.set_i(p, t); }
    { let p = a2.add(8); let t = cx.i(this.add(4 * ((4_i32).wrapping_mul(a3))).add(120)); cx.set_i(p, t); }
    return a2;
}
