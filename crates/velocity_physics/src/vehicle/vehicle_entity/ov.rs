//! Hand-written stand-ins for calls the translated code makes outside the physics code
//! (tools/phys_c2rs4_ov.txt routes them here): trivial accessors, the CRT random generator, debug
//! output, and the race/AI/camera systems that are not modelled in the physics crate.

use super::cx::{Cx, Ptr, Sp, HIST_COUNT, HIST_ENTRY};

/// debug output / bookkeeping that has no effect on the simulation
#[macro_export]
macro_rules! ov_ignore {
    ($($t:tt)*) => {
        ()
    };
}
/// a getter returning its object (CompartmentManager__8DE50, ParticleManager__101320)
#[macro_export]
macro_rules! ov_ident {
    ($cx:expr, $p:expr) => {
        $p
    };
}
#[macro_export]
macro_rules! ov_zero_i {
    ($($t:tt)*) => {
        0i32
    };
}
#[macro_export]
macro_rules! ov_null {
    ($($t:tt)*) => {
        $crate::vehicle::vehicle_entity::cx::Ptr::NULL
    };
}
/// a system outside the physics crate that this code path needs
#[macro_export]
macro_rules! ov_unmodelled {
    ($($t:tt)*) => {
        panic!("call into a system the physics crate does not model: {}", stringify!($($t)*))
    };
}

/// VsAIRaceVehicle__248C0 0x000248C0 — row `k` of the car's rotation matrix
pub fn ov_car_rot_row(_cx: &mut Cx, car: Ptr, k: i32) -> Ptr {
    car.add(16 * k + 112)
}
/// VsBaseCamera__5EC70 0x0005EC70 — row `k` of a 4x4 matrix (`this + 16 * k`)
pub fn ov_mat_row(_cx: &mut Cx, m: Ptr, k: i32) -> Ptr {
    m.add(16 * k)
}
/// VsAIRaceVehicle__24200 0x00024200 — the car's linear velocity
pub fn ov_car_velocity(_cx: &mut Cx, car: Ptr) -> Ptr {
    car.add(32)
}
/// VehicleEntity::GetPosition (PhysicsCarInput__1BA270) 0x001BA270: the car's position (car vtbl +0x1C)
/// into `out` as a VsVector4 with w = 1; returns `out`
pub fn ov_ve_get_position(cx: &mut Cx, ve: Ptr, out: Ptr) -> Ptr {
    use crate::fastcar::rigid_body::RigidBody;
    let car = ve.add(240);
    let p = cx.car_at(car).position();
    cx.set_v3(out, p);
    cx.set_f(out.add(12), 1.0);
    out
}
/// VsAITools::IsLeftOfLine 0x00028090 (xz plane; x87 differences, rounded where stored)
pub fn ov_is_left_of_line(cx: &mut Cx, a: Ptr, b: Ptr, c: Ptr) -> i32 {
    let (a, b, c) = (cx.v3(a), cx.v3(b), cx.v3(c));
    let dz = (a[2] as f64 - c[2] as f64) as f32;
    let p = ((b[0] as f64 - a[0] as f64) * dz as f64) as f32;
    let dx = (a[0] as f64 - c[0] as f64) as f32;
    let v = (p as f64 - (b[2] as f64 - a[2] as f64) * dx as f64) as f32;
    (v <= 0.0) as i32
}
/// VsAITools__293D0 0x000293D0: `out.xyz = a x b` (w untouched)
pub fn ov_ai_cross(cx: &mut Cx, out: Ptr, a: Ptr, b: Ptr) -> Ptr {
    let (a, b) = (cx.v3(a), cx.v3(b));
    let c = |p: f32, q: f32, r: f32, t: f32| ((p * q) as f64 - r as f64 * t as f64) as f32;
    cx.set_f(out, c(b[2], a[1], b[1], a[2]));
    cx.set_f(out.add(4), c(b[0], a[2], b[2], a[0]));
    cx.set_f(out.add(8), c(b[1], a[0], b[0], a[1]));
    out
}
/// VsRaceManager__3D640 as `VehicleEntity::ComputeForcesAndTorques` uses it (0x001C5800): the race
/// record whose score manager feeds this update's zone / nitrous gains; only cars given gains have one
pub fn ov_score_record(cx: &mut Cx, _rm: Ptr, id: i32) -> Ptr {
    cx.glob.score_records.get(&id).copied().unwrap_or(Ptr::NULL)
}
/// VsRaceManager__3DF30 0x0003DF30: the record's `m_pScoreManager` (+0x1C8); the stand-in record is its
/// own score manager (+0xA0 zone gain, +0xA4 nitrous gain)
pub fn ov_score_manager(_cx: &mut Cx, rec: Ptr) -> Ptr {
    rec
}
/// BarrierRender 0x001B3F40 (`+8`): the barrier wall quadtree, null without barriers
pub fn ov_barrier_qt(cx: &mut Cx, _br: Ptr) -> Ptr {
    if cx.barrier.is_some() { Ptr { sp: Sp::Qt, off: super::cx::COLL_BARRIER << super::cx::COLL_SHIFT } } else { Ptr::NULL }
}
/// BarrierRender 0x001B3F20 (`+4`): the barrier wall mesh
pub fn ov_barrier_mesh(cx: &mut Cx, _br: Ptr) -> Ptr {
    if cx.barrier.is_some() { Ptr { sp: Sp::Mesh, off: super::cx::COLL_BARRIER << super::cx::COLL_SHIFT } } else { Ptr::NULL }
}
/// PhysicsManager__1B4710 0x001B4710 — the car's angular velocity
pub fn ov_car_omega(_cx: &mut Cx, car: Ptr) -> Ptr {
    car.add(48)
}
/// VsAIRaceVehicle__24850 0x00024850
pub fn ov_car_i2876(cx: &mut Cx, car: Ptr) -> i32 {
    cx.i(car.add(2876))
}
/// Speedo__F79A0 0x000F79A0
pub fn ov_speedo_f148(cx: &mut Cx, p: Ptr) -> f64 {
    cx.f(p.add(148)) as f64
}
/// VsAIDriver__15D50 0x00015D50 (race manager +520: race time; `PhysicsWorld::set_camera_shake` seeds it)
pub fn ov_race_time(cx: &mut Cx, rm: Ptr) -> f32 {
    cx.f(rm.add(520))
}
/// Speedo__F7970 0x000F7970
pub fn ov_speedo_b144(cx: &mut Cx, p: Ptr) -> f64 {
    cx.b(p.add(144)) as f64
}

/// VsAIRaceVehicle__23310 0x00023310 — the AI driver steers this player car (not modelled: never)
pub fn ov_ai_vehicle_flag(_cx: &mut Cx, _p: Ptr) -> i32 {
    0
}
/// VsAIWorld::GetInstance
pub fn ov_ai_world(_cx: &mut Cx) -> Ptr {
    Ptr::NULL
}
/// VsAIWorld::GetRaceManager
pub fn ov_race_manager(_cx: &mut Cx, _p: Ptr) -> Ptr {
    Ptr::NULL
}
/// VsRaceManager__3D640 0x0003D640 — the race's record for a driver (no race manager: none)
pub fn ov_race_vehicle(_cx: &mut Cx, _p: Ptr, _id: i32) -> Ptr {
    Ptr::NULL
}
/// PhysicsManager__19E060 — the vehicle array
pub fn ov_vehicle_array(_cx: &mut Cx, _p: Ptr) -> Ptr {
    Ptr { sp: Sp::Vlist, off: 0 }
}
/// VsArray::operator[] on the vehicle array: the slot holding vehicle `i`'s entity pointer
pub fn ov_vehicle_slot(_cx: &mut Cx, _a: Ptr, i: i32) -> Ptr {
    Ptr { sp: Sp::Vlist, off: 16 + 4 * i }
}
/// GameManager__C0470 — number of entries in the vehicle array
pub fn ov_vehicle_count(cx: &mut Cx, _a: i32) -> i32 {
    cx.glob.vlist.i(4)
}
/// OSDInGameScreen__FAF10(list, i) — the slot holding vehicle `i`'s entity pointer
pub fn ov_vehicle_slot2(_cx: &mut Cx, _a: Ptr, _st0: f64, i: i32) -> Ptr {
    Ptr { sp: Sp::Vlist, off: 16 + 4 * i }
}
pub fn ov_null_i(_cx: &mut Cx, _i: i32) -> Ptr {
    Ptr::NULL
}

/// VsConfigFileRead__1B6E80 0x001B6E80 — history entry `k` steps back (the `k > count` branch
/// substitutes the head index, as the original does)
pub fn ov_hist_entry(cx: &mut Cx, h: Ptr, k: i32) -> Ptr {
    let head = cx.i(h.add(4));
    let count = cx.i(h.add(12));
    let mut a2 = k;
    if a2 > count {
        a2 = head;
    }
    let mut v3 = head;
    if v3 < a2 {
        while a2 > 0 {
            v3 -= 1;
            if v3 < 0 {
                v3 = HIST_COUNT - 1;
            }
            a2 -= 1;
        }
    } else {
        v3 -= a2;
    }
    cx.ptr_at(h).add(HIST_ENTRY * v3)
}
/// VsConfigFileRead__1B6DB0 0x001B6DB0 — advance the head, copying the current entry forward
pub fn ov_hist_push(cx: &mut Cx, h: Ptr) -> i32 {
    let v2 = cx.i(h.add(4));
    let mut n = v2 + 1;
    if n >= HIST_COUNT {
        n = 0;
    }
    cx.set_i(h.add(4), n);
    let base = cx.ptr_at(h);
    let src = base.add(HIST_ENTRY * v2);
    let dst = base.add(HIST_ENTRY * n);
    cx.copy_words(dst, src, HIST_ENTRY / 4);
    let c = cx.i(h.add(12));
    if c < 0xFFFF {
        cx.set_i(h.add(12), c + 1);
    }
    0
}

/// (FastCar expr) 0x003AAFF0
pub fn ov_eval_expr(cx: &mut Cx, out: Ptr, _z: i32, e: Ptr, vars: Ptr) -> i32 {
    cx.eval_expr(out, e, vars)
}

/// (FastCar) 0x003651F0 — the wheel collision callback at car+1104 (`VehicleEntity::WheelCollision`);
/// without one the car's vtable[9], empty in this build
pub fn ov_detect_wheel_contacts(cx: &mut Cx, car: Ptr) {
    let id = car.vid();
    if cx.car_at(car).cd.is_none() {
        return;
    }
    let prev = std::mem::replace(&mut cx.cur, id);
    crate::tr_prelude::f_1bb320(cx, Ptr::car(id));
    cx.cur = prev;
}

pub fn ov_coerce_int(_cx: &mut Cx, f: f32) -> i32 {
    f.to_bits() as i32
}
pub fn ov_isnan(_cx: &mut Cx, f: f32) -> i32 {
    f.is_nan() as i32
}

/// CRT `rand` (0x003DE030): the game's one stream; `glob.rand_state` is handed in/out around each step
/// (`PhysicsWorld::{rand_state, set_rand_state}`)
pub fn ov_rand(cx: &mut Cx) -> i32 {
    let mut r = visos::handy::crt_rand::CrtRand { state: cx.glob.rand_state };
    let v = r.rand();
    cx.glob.rand_state = r.state;
    v
}
/// VsRandUnit 0x00013510
pub fn vs_rand_unit(cx: &mut Cx) -> f64 {
    ov_rand(cx) as f64 / 32767.0
}

/// `operator new` (0x001FDB40)
pub fn ov_new(cx: &mut Cx, bytes: i32) -> Ptr {
    cx.glob.heap.alloc(bytes)
}
/// `operator delete`
pub fn ov_delete(cx: &mut Cx, p: Ptr) {
    // the entity's initial history / config images are part of the vehicle, not heap blocks
    if matches!(p.sp, Sp::Heap(_)) {
        cx.glob.heap.free(p)
    }
}

/// VsFge(a, b, eps)
pub fn vs_fge(_cx: &mut Cx, a: f32, b: f32, eps: f32) -> i32 {
    (a >= b - eps) as i32
}

/// PhysicsManager__19D3E0 0x0019D3E0 — world ray cast (track, objects, cars): not ported yet
#[allow(clippy::too_many_arguments)]
pub fn ov_world_raycast(_cx: &mut Cx, _a1: Ptr, _a2: i32, _a3: Ptr, _a4: Ptr, _a5: Ptr, _a6: Ptr, _a7: i32, _a8: i32) -> i32 {
    unmodelled_i("PhysicsManager::RayCast (0x19D3E0)")
}

/// `VsAIRaceVehicle__24F90` — the PhysicsManager (level collision)
pub fn ov_physics_manager(_cx: &mut Cx) -> Ptr {
    Ptr { sp: Sp::World, off: 0 }
}
/// FCQuadTree::FindLeaf 0x0035A8D0: `*first`, `*count` of the leaf containing `pos`; returns the node
#[allow(clippy::too_many_arguments)]
pub fn ov_qt_find_leaf(cx: &mut Cx, qt: Ptr, _a2: Ptr, first: Ptr, count: Ptr, pos: Ptr, hint: i32, trace: Ptr) -> i64 {
    assert_eq!(qt.sp, Sp::Qt);
    let t = cx.coll(qt.off);
    let p = cx.v3(pos);
    let mut tr = crate::fastcar::quadtree::FcQtTrace::default();
    let (f, n, node) = t.qt.find_leaf(p, hint as u32, if trace.is_null() { None } else { Some(&mut tr) });
    cx.set_i(first, f as i32);
    cx.set_i(count, n as i32);
    if !trace.is_null() {
        for k in 0..20 {
            for j in 0..2 {
                cx.set_f(trace.add(8 * k + 4 * j as i32), tr.min[k as usize][j]);
                cx.set_f(trace.add(160 + 8 * k + 4 * j as i32), tr.max[k as usize][j]);
            }
        }
        cx.set_i(trace.add(320), tr.depth);
    }
    node as i64
}
/// FCQuadTree::GetFaceIndex 0x0011F2E0
pub fn ov_qt_face_index(cx: &mut Cx, qt: Ptr, i: i32) -> i32 {
    cx.coll(qt.off).qt.face_index(i as u32) as i32
}
/// FCMesh::GetVertex 0x0011F280 — pointer to vertex `k` of `face`
pub fn ov_mesh_vertex(cx: &mut Cx, mesh: Ptr, face: i32, k: i32) -> Ptr {
    let t = cx.coll(mesh.off);
    let (v, _, _) = t.mesh_arrays();
    let base = mesh.off & !((1 << super::cx::COLL_SHIFT) - 1);
    Ptr { sp: Sp::Mesh, off: base + v + 12 * t.mesh.index(face as u32, k as usize) as i32 }
}
/// FCIntersectRayTriangle (FastCar) 0x00383540: `*t, *u, *v` of the hit; non-zero on a hit
#[allow(clippy::too_many_arguments)]
pub fn ov_ray_triangle(cx: &mut Cx, t: Ptr, u: Ptr, v: Ptr, v0: Ptr, v1: Ptr, v2: Ptr, orig: Ptr, dir: Ptr) -> i32 {
    // out values the original does not reach keep what the caller had
    let mut hit = crate::fastcar::mesh::RayHit { t: cx.f(t), u: cx.f(u), v: cx.f(v) };
    let r = crate::fastcar::mesh::intersect_ray_triangle_raw(cx.v3(v0), cx.v3(v1), cx.v3(v2), cx.v3(orig), cx.v3(dir), &mut hit);
    cx.set_f(t, hit.t);
    cx.set_f(u, hit.u);
    cx.set_f(v, hit.v);
    r as i32
}
/// CBumpMap::GetHeight 0x001DA2D0
pub fn ov_bump_height(cx: &mut Cx, _map: Ptr, x: f32, z: f32) -> f64 {
    cx.track.and_then(|t| t.bump.as_ref()).map(|b| b.get_height(x, z) as f64).unwrap_or(0.0)
}
/// CBumpMap::CBumpMap 0x001B3BD0 (+12 enabled = 1, +4 = 0, +0 = 30; `Init` stays ignored: the
/// height table is `TrackWorld::bump`, which `ov_bump_height` reads)
pub fn ov_bump_map_ctor(cx: &mut Cx, this: Ptr) -> Ptr {
    cx.set_i(this.add(12), 1);
    cx.set_i(this.add(4), 0);
    cx.set_i(this, 30);
    this
}
/// VsFeq(a, b, eps)
pub fn ov_feq(_cx: &mut Cx, a: f32, b: f32, eps: f32) -> i32 {
    ((a - b).abs() <= eps) as i32
}

/// an argument of a virtual call
#[derive(Clone, Copy, Debug)]
pub enum Varg {
    P(Ptr),
    F(f32),
    I(i32),
    S(&'static str),
}
impl From<&'static str> for Varg {
    fn from(s: &'static str) -> Self {
        Varg::S(s)
    }
}
impl Varg {
    /// the argument as the callee's parameter kind (the decompiler's casts are not trusted)
    pub fn f(self) -> f32 {
        match self {
            Varg::F(f) => f,
            Varg::I(i) => f32::from_bits(i as u32),
            Varg::P(p) => f32::from_bits(p.bits() as u32),
            Varg::S(_) => 0.0,
        }
    }
    pub fn d(self) -> f64 {
        self.f() as f64
    }
    pub fn i(self) -> i32 {
        match self {
            Varg::F(f) => f.to_bits() as i32,
            Varg::I(i) => i,
            Varg::P(p) => p.bits(),
            Varg::S(_) => 0,
        }
    }
    pub fn q(self) -> i64 {
        self.i() as i64
    }
    pub fn p(self) -> Ptr {
        match self {
            Varg::P(p) => p,
            Varg::I(i) => Ptr::raw(i),
            Varg::F(f) => Ptr::raw(f.to_bits() as i32),
            Varg::S(_) => Ptr::NULL,
        }
    }
}

/// a virtual call: cars go to the hand-routed FastCar methods, every other object through its
/// vtable (`*this`) to the translated function at that slot
pub fn vcall(cx: &mut Cx, slot: i32, this: Ptr, args: &[Varg]) -> Varg {
    if this.sp == Sp::Ve && super::cx::vsplit(this.off).1 == super::cx::CAR {
        car_vcall(cx, slot, this, args);
        return Varg::I(0);
    }
    let vt = cx.i(this);
    let f = cx.i(Ptr::glob(vt.wrapping_add(slot))) as u32;
    crate::tr_prelude::vdispatch(cx, f, this, args)
}

impl From<f64> for Varg {
    fn from(f: f64) -> Self {
        Varg::F(f as f32)
    }
}
impl From<Ptr> for Varg {
    fn from(p: Ptr) -> Self {
        Varg::P(p)
    }
}
impl From<f32> for Varg {
    fn from(f: f32) -> Self {
        Varg::F(f)
    }
}
impl From<i32> for Varg {
    fn from(i: i32) -> Self {
        Varg::I(i)
    }
}

/// virtual calls through the car's vtable 0x675854
pub fn car_vcall(cx: &mut Cx, slot: i32, this: Ptr, args: &[Varg]) {
    use crate::fastcar::rigid_body::RigidBody;
    let id = this.vid();
    // a pointer argument the decompiler typed as an integer
    let pa: Vec<Varg> = args.iter().enumerate().map(|(k, a)| match (slot, k, a) {
        (16 | 20 | 24 | 28, _, Varg::I(x)) => Varg::P(Ptr::raw(*x)),
        // Evolve's step passed through an integer-typed slot: the float's bits
        (0, 0, Varg::I(x)) => Varg::F(f32::from_bits(*x as u32)),
        _ => *a,
    }).collect();
    let args = &pa[..];
    match (slot, args) {
        // +0x00 (0x3652D0): Evolve(h, keepImpulses)
        (0, [Varg::F(h), Varg::I(k)]) => evolve_car(cx, id, *h, *k != 0),
        // +0x10 (0x3627E0) / +0x14 (0x362890): save / restore the car state (constraint handler)
        (16, [Varg::P(st)]) => {
            crate::tr_prelude::f_3627e0(cx, this, *st);
        }
        (20, [Varg::P(st)]) => {
            let st = *st;
            crate::tr_prelude::f_362890(cx, this, st);
        }
        // +0x20 (0x362930): wheel frames, clear dirty
        (32, []) | (32, [Varg::F(_)]) => cx.car_at(this).update_derived(),
        // +0x1C (0x3616D0): position of the body = centre + suspension offset
        (28, [Varg::P(out)]) => {
            let p = cx.car_at(this).position();
            cx.set_v3(*out, p);
        }
        // +0x18 (0x35FC60): 4x4 body matrix
        (24, [Varg::P(out)]) => {
            let m = cx.car_at(this).matrix();
            for (k, v) in m.iter().enumerate() {
                cx.set_f(out.add(4 * k as i32), *v);
            }
        }
        // +0x08 / +0x0C (VehicleEntity__1D9F20): parameter error / warning text
        (8, [Varg::S(msg)]) => log::error!("FastCar: {msg}"),
        (12, [Varg::S(msg)]) => log::warn!("FastCar: {msg}"),
        _ => panic!("car vtable slot +{slot} with {args:?} not routed"),
    }
}

thread_local! {
    /// the `Cx` a running `CFastCar::Evolve` belongs to, for its wheel-collision callback
    static CUR_CX: std::cell::Cell<*mut ()> = const { std::cell::Cell::new(std::ptr::null_mut()) };
}

/// `CFastCar::Evolve` (0x003652D0, translated) of vehicle `id`. With `VELOCITY_EVOLVE_DIFF` set the
/// hand port runs on a copy of the same state and the first differing car words are logged.
pub fn evolve_car(cx: &mut Cx, id: usize, h: f32, keep: bool) {
    if !evolve_diff() {
        crate::tr_prelude::f_3652d0(cx, Ptr::car(id), h, keep as i32);
        return;
    }
    let before = cx.car_of_mut(id).m.clone();
    crate::tr_prelude::f_3652d0(cx, Ptr::car(id), h, keep as i32);
    let tr = cx.car_of_mut(id).m.clone();
    cx.car_of_mut(id).m = before;
    evolve_car_hand(cx, id, h, keep);
    let hand = std::mem::replace(&mut cx.car_of_mut(id).m, tr.clone());
    let diffs: Vec<String> = (0..tr.len_bytes() / 4)
        .filter(|&k| tr.u(4 * k) != hand.u(4 * k))
        .take(12)
        .map(|k| format!("+{} tr {:e} hand {:e}", 4 * k, tr.f(4 * k), hand.f(4 * k)))
        .collect();
    if !diffs.is_empty() {
        EVOLVE_DIFFS.with(|c| c.set(c.get() + 1));
        eprintln!("evolve diff car {id}: {}", diffs.join(", "));
    }
}

fn evolve_diff() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VELOCITY_EVOLVE_DIFF").is_some())
}

thread_local! {
    /// evolves whose hand port disagreed with the translation (diff mode)
    pub static EVOLVE_DIFFS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// the hand-ported `CFastCar::Evolve`; its collision callback re-enters this `Cx`
pub fn evolve_car_hand(cx: &mut Cx, id: usize, h: f32, keep: bool) {
    use crate::fastcar::rigid_body::RigidBody;
    let car = cx.car_of_mut(id) as *mut crate::fastcar::car::CFastCar;
    let prev = CUR_CX.with(|c| c.replace(cx as *mut Cx as *mut ()));
    // SAFETY: the car is only reached again through the callback below, which is handed the same car
    unsafe { (*car).evolve(h, keep) };
    CUR_CX.with(|c| c.set(prev));
}

/// the callback installed at car+1104: `VehicleEntity::WheelCollision` 0x001BB320 on the `Cx` that
/// is running this car's `Evolve`
pub fn wheel_collision_callback(id: usize) -> crate::fastcar::car::CdCallback {
    Box::new(move |car: &mut crate::fastcar::car::CFastCar| {
        let p = CUR_CX.with(|c| c.get());
        assert!(!p.is_null(), "car {id} evolved outside a physics context");
        // SAFETY: set by `evolve_car` for the duration of the call; single-threaded
        let cx = unsafe { &mut *(p as *mut Cx) };
        cx.vehs[id].1 = car as *mut _;
        let prev = std::mem::replace(&mut cx.cur, id);
        crate::tr_prelude::f_1bb320(cx, Ptr::car(id));
        cx.cur = prev;
    })
}

#[allow(dead_code)]
const _CAR: usize = crate::fastcar::car::off::SIZE;

/// VsVector4::VsVector4 (copy)
pub fn ov_vec4_copy(cx: &mut Cx, d: Ptr, s: Ptr) -> Ptr {
    cx.copy_words(d, s, 4);
    d
}
/// VsVector4::VsVector4(x, y, z, w) — components passed as raw 32-bit words
pub fn ov_vec4_xyzw(cx: &mut Cx, d: Ptr, x: i32, y: i32, z: i32, w: i32) -> Ptr {
    for (k, v) in [x, y, z, w].into_iter().enumerate() {
        cx.set_i(d.add(4 * k as i32), v);
    }
    d
}
/// VsVector4::VsVector4_XYZ (w = 1.0)
pub fn ov_vec4_xyz(cx: &mut Cx, d: Ptr, x: i32, y: i32, z: i32) -> Ptr {
    ov_vec4_xyzw(cx, d, x, y, z, 0x3F80_0000)
}
/// ColourXBOX.cpp 0x002B11C0
pub fn ov_colour_xbox(cx: &mut Cx, this: Ptr, r: i32, g: i32, b: i32, a: i32) -> Ptr {
    // FontManager__2B1190 = min(2x, 255); bytes a r g b... stored +3 r, +2 g, +1 b, +0 a
    let c = |v: i32| (2 * (v & 0xFF)).min(255);
    cx.set_b(this.add(3), c(r));
    cx.set_b(this.add(2), c(g));
    cx.set_b(this.add(1), c(b));
    cx.set_b(this, c(a));
    this
}
/// VsVector4::Set
pub fn ov_vec4_set(cx: &mut Cx, d: Ptr, x: i32, y: i32, z: i32, w: i32) -> Ptr {
    ov_vec4_xyzw(cx, d, x, y, z, w)
}

#[macro_export]
macro_rules! ov_zero_f {
    ($($t:tt)*) => {
        0.0f32
    };
}

/// `VsTrafficManager__42F60` — the traffic manager (only its pad records exist here)
pub fn ov_traffic(_cx: &mut Cx) -> Ptr {
    Ptr { sp: Sp::Traffic, off: 0 }
}
/// GameInput__A5A40 0x000A5A40 — input source kind of a pad record (0 = local pad, no net id)
pub fn ov_input_kind(cx: &mut Cx, pad: Ptr) -> i32 {
    if cx.i(pad.add(24)) == 0 {
        0
    } else {
        panic!("networked input source")
    }
}
/// `eh vector constructor iterator` (0x00011050)
pub fn ov_vector_ctor(cx: &mut Cx, p: Ptr, size: i32, n: i32, ctor: Ptr) {
    for k in 0..n {
        let e = p.add(k * size);
        match ctor.off {
            0x1B6C90 => cx.memset(e, 0, size),
            // `VsVector4::VsVector4` (empty)
            0 => {}
            // element constructors are translated methods
            a => {
                crate::tr_prelude::vdispatch(cx, a as u32, e, &[]);
            }
        }
    }
}
/// `memset` (0x003DE070)
pub fn ov_memset(cx: &mut Cx, p: Ptr, v: i32, n: i32) -> Ptr {
    cx.memset(p, v, n);
    p
}
/// `memcpy` (0x003DE740)
pub fn ov_memcpy(cx: &mut Cx, d: Ptr, s: Ptr, n: i32) -> Ptr {
    cx.memcpy(d, s, n);
    d
}
pub fn ov_identity_p(_cx: &mut Cx, p: Ptr) -> Ptr {
    p
}
pub fn ov_vec4_x(cx: &mut Cx, p: Ptr) -> f32 {
    cx.f(p)
}
pub fn ov_vec4_y(cx: &mut Cx, p: Ptr) -> f32 {
    cx.f(p.add(4))
}
pub fn ov_vec4_z(cx: &mut Cx, p: Ptr) -> f32 {
    cx.f(p.add(8))
}
/// VsVector4::Dot3 / VsDot3
pub fn ov_vec_dot3(cx: &mut Cx, a: Ptr, b: Ptr) -> f64 {
    let (a, b) = (cx.v3(a), cx.v3(b));
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) as f64
}
pub fn ov_vec_length_sq(cx: &mut Cx, a: Ptr) -> f64 {
    ov_vec_dot3(cx, a, a)
}
pub fn ov_vec_length(cx: &mut Cx, a: Ptr) -> f64 {
    (ov_vec_length_sq(cx, a) as f32).sqrt() as f64
}
pub fn ov_vec_dist_sq(cx: &mut Cx, a: Ptr, b: Ptr) -> f64 {
    let (a, b) = (cx.v3(a), cx.v3(b));
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]) as f64
}
/// VsAdd 0x00024960 (thiscall `a`, out, b): out = a + b, w = 1 (VsVector4_XYZ + copy)
pub fn ov_vec_add(cx: &mut Cx, a: Ptr, o: Ptr, b: Ptr) -> Ptr {
    let (a, b) = (cx.v3(a), cx.v3(b));
    cx.set_v3(o, [a[0] + b[0], a[1] + b[1], a[2] + b[2]]);
    cx.set_f(o.add(12), 1.0);
    o
}
/// VsSub 0x00016450 (thiscall `a`, out, b): out = a - b, w = 1
pub fn ov_vec_sub(cx: &mut Cx, a: Ptr, o: Ptr, b: Ptr) -> Ptr {
    let (a, b) = (cx.v3(a), cx.v3(b));
    cx.set_v3(o, [a[0] - b[0], a[1] - b[1], a[2] - b[2]]);
    cx.set_f(o.add(12), 1.0);
    o
}
/// VsRandRange 0x000134F0: VsRandUnit() * (hi - lo) + lo, in st0
pub fn vs_rand_range(cx: &mut Cx, lo: f32, hi: f32) -> f64 {
    vs_rand_unit(cx) * (hi as f64 - lo as f64) + lo as f64
}
/// VsAIRaceVehicle__24CC0: `&unk_7805B0` = (0, 1, 0, 0) (static ctor 0x00431EF0)
pub fn ov_up_vector(cx: &mut Cx) -> Ptr {
    let p = Ptr::glob(0x7805B0);
    for (k, v) in [0.0f32, 1.0, 0.0, 0.0].into_iter().enumerate() {
        cx.set_f(p.add(4 * k as i32), v);
    }
    p
}
/// LightStreak__133750(`unk_720840` = PuddleRender, record): append the 48-byte puddle record, re-sort by race
/// distance (qsort 0x001337C0 / 0x00133800) and mark the list dirty (0x00133A20); graphics' list, so it leaves
/// as a `PhysicsPuddle`
pub fn ov_puddle_add(cx: &mut Cx, _mgr: Ptr, rec: Ptr) {
    let w: [u32; 12] = std::array::from_fn(|k| cx.i(rec.add(4 * k as i32)) as u32);
    let f = |k: usize| f32::from_bits(w[k]);
    cx.glob.puddles.push(crate::physics_objects::PhysicsPuddle {
        position: [f(0), f(1), f(2)],
        radius: f(3),
        direction: [f(4), f(5)],
        race_distance: f(6),
        kind: w[7] as i32,
        size: [f(8), f(9)],
        life: w[10] as i32,
        flags: w[11],
    });
}
/// VehicleManager__1DE400 (on the object manager): the hand port
pub fn ov_reset_all_objects(cx: &mut Cx, _pom: Ptr) {
    crate::physics_objects::physics_object_manager::reset_all_objects(cx)
}
pub fn ov_min(_cx: &mut Cx, a: f32, b: f32) -> f64 {
    if a <= b { a as f64 } else { b as f64 }
}

/// stand-ins for calls into systems this crate does not model; reaching one is a bug report
pub fn unmodelled_p(what: &str) -> Ptr {
    panic!("call into a system the physics crate does not model: {what}")
}
pub fn unmodelled_i(what: &str) -> i32 {
    panic!("call into a system the physics crate does not model: {what}")
}
pub fn unmodelled_f(what: &str) -> f32 {
    panic!("call into a system the physics crate does not model: {what}")
}
pub fn unmodelled_d(what: &str) -> f64 {
    panic!("call into a system the physics crate does not model: {what}")
}
pub fn unmodelled_v(what: &str) {
    panic!("call into a system the physics crate does not model: {what}")
}

/// GameInput__A7890 0x000A7890 — `*this = -1.0f` (the decompiler shows extra arguments)
#[macro_export]
macro_rules! ov_set_minus_one {
    ($cx:expr, $p:expr $(, $r:expr)*) => {{
        $cx.set_i($p, -1082130432)
    }};
}
/// VsVector4::operator-= (xyz)
pub fn ov_vec4_subeq(cx: &mut Cx, a: Ptr, b: Ptr) -> Ptr {
    let (x, y) = (cx.v3(a), cx.v3(b));
    cx.set_v3(a, [x[0] - y[0], x[1] - y[1], x[2] - y[2]]);
    a
}
/// VsVector4::operator+= (xyz)
pub fn ov_vec4_addeq(cx: &mut Cx, a: Ptr, b: Ptr) -> Ptr {
    let (x, y) = (cx.v3(a), cx.v3(b));
    cx.set_v3(a, [x[0] + y[0], x[1] + y[1], x[2] + y[2]]);
    a
}
/// VsVector4::operator*(this, &result, s)
pub fn ov_vec4_mul(cx: &mut Cx, v: Ptr, out: Ptr, s: f32) -> Ptr {
    let x = cx.v3(v);
    cx.set_v3(out, [x[0] * s, x[1] * s, x[2] * s]);
    out
}
/// VsVector4::operator*= (xyz)
pub fn ov_vec4_muleq(cx: &mut Cx, a: Ptr, s: f32) -> Ptr {
    let x = cx.v3(a);
    cx.set_v3(a, [x[0] * s, x[1] * s, x[2] * s]);
    a
}

/// D3DXMatrixIdentity (via `AB::Renderer` 0x002AB2B0)
pub fn ov_mat_identity(cx: &mut Cx, m: Ptr) -> i32 {
    for k in 0..16 {
        cx.set_f(m.add(4 * k), if k % 5 == 0 { 1.0 } else { 0.0 });
    }
    m.bits()
}
/// VsVector4::SetX / SetY / SetZ / SetW
pub fn ov_vec4_setx(cx: &mut Cx, p: Ptr, v: i32) -> Ptr {
    cx.set_i(p, v);
    p
}
pub fn ov_vec4_sety(cx: &mut Cx, p: Ptr, v: i32) -> Ptr {
    cx.set_i(p.add(4), v);
    p.add(4)
}
pub fn ov_vec4_setz(cx: &mut Cx, p: Ptr, v: i32) -> Ptr {
    cx.set_i(p.add(8), v);
    p.add(8)
}
pub fn ov_vec4_setw(cx: &mut Cx, p: Ptr, v: i32) -> Ptr {
    cx.set_i(p.add(12), v);
    p.add(12)
}
/// VsVector4::W
pub fn ov_vec4_w(cx: &mut Cx, p: Ptr) -> f32 {
    cx.f(p.add(12))
}
/// CRT memmove (0x003E22A0)
pub fn ov_memmove(cx: &mut Cx, d: Ptr, s: Ptr, n: i32) -> Ptr {
    let b: Vec<i32> = (0..n).map(|k| cx.b(s.add(k))).collect();
    for (k, v) in b.into_iter().enumerate() {
        cx.set_b(d.add(k as i32), v);
    }
    d
}
/// CRT qsort (0x003E2B80); the only comparator passed is FastCar 0x00384190. Insertion sort: the
/// CRT's quicksort orders equal keys differently, the comparator never reports ties for distinct
/// records in practice.
pub fn ov_qsort(cx: &mut Cx, base: Ptr, n: i32, size: i32, cmp: Ptr) {
    assert_eq!(cmp, Ptr::func(0x384190), "qsort comparator");
    let mut tmp = vec![0i32; size as usize];
    for i in 1..n {
        let mut j = i;
        while j > 0 && crate::tr_prelude::f_384190(cx, base.add((j - 1) * size), base.add(j * size)) > 0 {
            for k in 0..size {
                tmp[k as usize] = cx.b(base.add((j - 1) * size + k));
            }
            for k in 0..size {
                let v = cx.b(base.add(j * size + k));
                cx.set_b(base.add((j - 1) * size + k), v);
            }
            for k in 0..size {
                cx.set_b(base.add(j * size + k), tmp[k as usize]);
            }
            j -= 1;
        }
    }
}
/// MessageManager__CA2C0(manager, type, message): game messages raised by the physics (collisions)
pub fn ov_message(cx: &mut Cx, _mgr: Ptr, ty: i32, msg: Ptr) {
    let words: Vec<i32> = (0..20).map(|k| cx.i(msg.add(4 * k))).collect();
    cx.glob.messages.push((ty, words));
    // channels 2 / 4 (object hit / woken) are also heard by RaceTimelineSequencer's restore point: the object's
    // name hash (RaceTimelineSequencer__172C70 = *(obj+136)+76)
    if !msg.is_null() && (ty == 2 || ty == 4) {
        let obj = cx.ptr_at(msg.add(if ty == 2 { 4 } else { 0 }));
        let inst = if obj.is_null() { Ptr::NULL } else { cx.ptr_at(obj.add(136)) };
        if !inst.is_null() {
            let object = cx.i(inst.add(76)) as u32;
            use crate::physics_objects::PhysicsObjectNotice as E;
            cx.glob.obj_notices.push(if ty == 2 { E::Hit { object } } else { E::Woken { object } });
        }
    }
    // type 2 (car hit a physics object) goes straight to its listener, the object manager
    if ty == 2 {
        crate::physics_objects::physics_object_manager::f_1e1f80(cx, msg);
    }
}
/// SlipstreamRender__13FB30 0x0013FB30 (SetStrength): queued for the renderer
pub fn ov_slipstream(cx: &mut Cx, _mgr: i32, vehicle: i32, strength: f32) {
    cx.glob.slipstream.push((vehicle, strength));
}
/// tyrespray__154FC0 0x00154FC0 (ZoneRender `flt_737AB0`, the caller passes its first float, unused): pad `a2`'s
/// zone FX level (+24 + 4·a2, graphics' `ZoneFx`, mirrored by `PhysicsWorld::set_zone_fx_level`) × 0.5
pub fn ov_zone_fx_amount(cx: &mut Cx, _this: f32, a2: i32) -> f64 {
    if (a2 as u32) < 2 { cx.f(Ptr::glob(0x737AB0 + 24 + 4 * a2)) as f64 * 0.5 } else { 0.0 }
}
/// LightStreak__133970 0x00133970 (a VehicleEntity inline linked into LightStreak.cpp): FastCar 0x00364BB0 on the
/// car (VE+240): wheel `a2`'s contact count car+1088+4·a2 > 0
pub fn ov_wheel_on_ground(cx: &mut Cx, this: Ptr, a2: i32) -> i32 {
    (cx.i(this.add(240).add(1088 + 4 * a2)) > 0) as i32
}
/// ReplayCamera_Manager__1665E0 0x001665E0 (`this` = byte_749128 read as a byte by the caller, unused)
pub fn ov_jump_cam_can_see(cx: &mut Cx, _mgr: i32, choose: i32) -> i32 {
    let hook = cx.glob.jump_cam.clone();
    hook.call(super::cx::JumpCamCall::CanSee { choose: choose as u8 != 0 }, &mut CxJumpCamWorld(cx)) as i32
}
/// ReplayCamera_Manager__1665A0 0x001665A0
pub fn ov_jump_cam_start(cx: &mut Cx, _mgr: i32) {
    let hook = cx.glob.jump_cam.clone();
    hook.call(super::cx::JumpCamCall::Start, &mut CxJumpCamWorld(cx));
}

/// the running step's physics as the replay camera manager reads it (0x001649A0 / 0x00164EC0's callees)
pub struct CxJumpCamWorld<'a, 'b>(pub &'a mut Cx<'b>);

impl super::cx::JumpCamWorld for CxJumpCamWorld<'_, '_> {
    /// PhysicsManager__19E060 + VsArray::operator[] + PhysicsCarInput 0x001BA270
    fn car_position(&mut self, vehicle: u32) -> [f32; 3] {
        let cx = &mut *self.0;
        // VsArray asserts the index; out of range reads as no car
        if vehicle as i32 >= ov_vehicle_count(cx, 0) {
            return [0.0; 3];
        }
        let arr = ov_vehicle_array(cx, Ptr::NULL);
        let slot = ov_vehicle_slot(cx, arr, vehicle as i32);
        let ve = Ptr::raw(cx.i(slot));
        if ve.bits() == Ptr::NULL.bits() {
            return [0.0; 3];
        }
        cx.push_frame(16);
        let out = cx.lptr(0);
        ov_ve_get_position(cx, ve, out);
        let p = cx.v3(out);
        cx.pop_frame();
        p
    }
    /// ReplayCamera_Orbit 0x00167680 -> MaterialProperties 0x0019D350(a5 = 1 -> flags 23) -> 0x0019D3E0
    fn blocked(&mut self, from: [f32; 3], to: [f32; 3], vehicle: u32) -> bool {
        let cx = &mut *self.0;
        cx.push_frame(64);
        let (t, info, a, b) = (cx.lptr(0), cx.lptr(4), cx.lptr(32), cx.lptr(48));
        // 0x00167680: t = 0.0 (RayCast only writes it)
        cx.set_f(t, 0.0);
        cx.set_v3(a, from);
        cx.set_v3(b, to);
        let r = crate::tr_prelude::f_19d3e0(cx, Ptr { sp: Sp::World, off: 0 }, 0, t, info, a, b, vehicle as i32, 23);
        cx.pop_frame();
        // 0x00167680: `(_BYTE)v4 != 0`
        r as u8 != 0
    }
}
/// ParticleManager__1000E0 0x001000E0 (FireOffParticleTrail): event is the instance type passed in a float slot
pub fn ov_particle_trail(cx: &mut Cx, _mgr: i32, _a2: Ptr, from: Ptr, to: Ptr, ev: f32) {
    let event = ev.to_bits();
    if event == 24 || event == 26 {
        let (from, to) = (cx.v3(from), cx.v3(to));
        cx.glob.obj_events.push(crate::physics_objects::PhysicsObjectEvent::Trail { from, to, event });
    }
}
/// TimelineBridge__98D10 0x00098D10: the race timeline sequencer (always present here; core plays the timeline)
pub fn ov_timeline_bridge(_cx: &mut Cx, _p: Ptr) -> i32 {
    1
}
/// RaceTimelineSequencer::PlayTimeline (seq, edx, timeline id from instance +36)
pub fn ov_play_timeline(cx: &mut Cx, _seq: i32, _edx: i32, timeline: i32) {
    cx.glob.obj_events.push(crate::physics_objects::PhysicsObjectEvent::PlayTimeline { timeline: timeline as u32 });
}
