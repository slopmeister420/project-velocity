//! src/Physics/PhysicsObjects/PhysicsObjectManager.cpp
#[cfg(feature = "tr")]
pub mod tr;

#[cfg(feature = "tr")]
pub use hand::*;

#[cfg(feature = "tr")]
mod hand {
    use super::super::{PhysicsObjectEvent, PhysicsObjectInfo};
    use crate::tr_prelude::*;

    /// `PhysicsManager` +24: the object manager
    const PM_OBJECT_MANAGER: i32 = 24;
    /// object manager image size (`m_CurrentList` at +3216 is the last member used)
    const POM_SIZE: i32 = 3328;

    /// the object manager, created on first use (`m_CurrentList` starts before list 0)
    fn object_manager(cx: &mut Cx) -> Ptr {
        let pm = ov_physics_manager(cx);
        let p = cx.ptr_at(pm.add(PM_OBJECT_MANAGER));
        if !p.is_null() {
            return p;
        }
        let p = cx.glob.heap.alloc(POM_SIZE);
        cx.set_i(p.add(3216), -1);
        cx.set_ptr(pm.add(PM_OBJECT_MANAGER), p);
        p
    }

    /// CCompartment::RegisterInstances 0x002E7B50, physics lists only: each Physics (1) / Immovable (3)
    /// instance list gets a manager list (CompartmentManager__86F70 -> 0x001DE730) and every record goes
    /// through F3CompartmentManager::AddInstance 0x0008AEA0. The file image stays resident (resource
    /// pointers and instance records point into it). Returns the number of objects made.
    pub fn register_instances(cx: &mut Cx, cpt: &[u8]) -> usize {
        let u = |o: usize| cpt.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).unwrap_or(0);
        // only the v5 layout is read (stale v3 packs are misparsed by the engine too)
        if u(0) != u32::from_le_bytes(*b"_CPT") || u(4) != 5 {
            return 0;
        }
        let (rc, rt, ib) = (u(0x28) as usize, u(0x2C) as usize, (u(0x30) & 0x7FFF_FFFF) as usize);
        if (u(0x30) as i32) < 0 || ib == 0 || ib + 16 > cpt.len() || rt + 12 * rc > cpt.len() {
            return 0;
        }
        let img = cx.glob.heap.alloc_bytes(cpt);
        // the relocated resource table: {pointer (0 = empty slot), kind, hash}
        let table = cx.glob.heap.alloc((12 * rc).max(4) as i32);
        for k in 0..rc {
            let off = u(rt + 12 * k) as i32;
            let e = table.add(12 * k as i32);
            if off != 0 {
                cx.set_ptr(e, img.add(off));
            }
            cx.set_i(e.add(4), u(rt + 12 * k + 4) as i32);
            cx.set_i(e.add(8), u(rt + 12 * k + 8) as i32);
        }
        let pom = object_manager(cx);
        let mut made = 0;
        let mut seen = 0u32;
        let mut p = ib + 16;
        for _ in 0..u(ib) {
            let (ty, n) = (u(p), u(p + 4) as usize);
            p += 16;
            // each type once; the exporter pads SimpleAnimation records, so nothing after one parses
            if ty > 4 || seen & (1 << ty) != 0 {
                break;
            }
            seen |= 1 << ty;
            if (ty == 1 || ty == 3) && n > 0 {
                f_1de730(cx, pom, n as i32, ty as i32);
                for k in 0..n {
                    if add_instance(cx, pom, img.add((p + 80 * k) as i32), ty as i32, table, rc as i32) {
                        made += 1;
                    }
                }
            }
            p += 80 * n;
        }
        made
    }

    /// F3CompartmentManager::AddInstance 0x0008AEA0 cases 1 and 3: the AddCompartmentObject params
    /// (CompartmentManager__8DDE0 layout) from an instance record, then `PhysicsObjectManager__1DFF60`
    fn add_instance(cx: &mut Cx, pom: Ptr, rec: Ptr, ty: i32, table: Ptr, count: i32) -> bool {
        let (vis, phys, extra) = (cx.i(rec.add(60)), cx.i(rec.add(64)), cx.i(rec.add(68)));
        if vis as u32 >= count as u32 || phys as u32 >= count as u32 {
            log::warn!("physics objects: Physical model instance using bad index");
            return false;
        }
        let a = cx.glob.heap.alloc(64);
        cx.set_i(a, ty);
        cx.set_ptr(a.add(4), table.add(12 * vis));
        cx.set_ptr(a.add(8), table.add(12 * phys));
        if extra != -1 {
            cx.set_ptr(a.add(12), table.add(12 * extra));
        }
        for k in 0..4 {
            let w = cx.i(rec.add(4 * k));
            cx.set_i(a.add(16 + 4 * k), w);
        }
        for k in 0..3 {
            let q = cx.f(rec.add(16 + 4 * k));
            cx.set_f(a.add(32 + 4 * k), 0.0 - q);
        }
        let w = cx.i(rec.add(28));
        cx.set_i(a.add(44), w);
        if ty == 1 {
            let m = cx.i(rec.add(32));
            cx.set_i(a.add(48), m);
        }
        let name = cx.i(rec.add(76));
        cx.set_i(a.add(52), name);
        let fl = cx.u16(rec.add(74));
        cx.set_u16(a.add(56), fl);
        let r = f_1dff60(cx, pom, a) as i32;
        cx.glob.heap.free(a);
        if r == 0 {
            return false;
        }
        let obj = Ptr::raw(r);
        cx.set_ptr(obj.add(136), rec);
        // physics objects with a special role, by visual resource hash (CCRC of the model name)
        let hash = cx.i(table.add(12 * vis + 8));
        match (ty, hash) {
            // propLT_streetlamp_common: StartLineEffects' light index (render side, not kept: 0)
            (1, -2080849669) => {
                f_1e5e80(cx, obj, 0, 0);
            }
            // propld_dummycopcar
            (1, -142311904) => {
                f_1e5e80(cx, obj, 1, 0);
            }
            // prop_fire_hydrant_top: numbered in load order
            (1, 648758418) => {
                let n = cx.i(Ptr::glob(0x6F0D68));
                f_1e5e80(cx, obj, 2, n);
                cx.set_i(Ptr::glob(0x6F0D68), n + 1);
            }
            _ => {
                cx.set_u16(rec.add(56), 0xFFFF);
                cx.set_u16(rec.add(58), 0xFFFF);
            }
        }
        true
    }

    /// PhysicsObjectManager.cpp 0x001E1F80: listener for message 2 (a car hit a physics object), run from
    /// `MessageManager__CA2C0` inside the collision pass
    pub fn f_1e1f80(cx: &mut Cx, msg: Ptr) {
        if msg.is_null() {
            return;
        }
        let obj = cx.ptr_at(msg.add(4));
        let inst = cx.ptr_at(obj.add(136));
        let point = cx.v3(msg.add(8));
        let impulse = cx.f(msg.add(40));
        // 0x001E2240: the instance's impact sound
        let (sound, mapped) = match cx.i(inst.add(48)) as u32 {
            0 => (207699420, true),
            2031 => (1146417784, true),
            s => (s, false),
        };
        let flags = cx.i(obj.add(152));
        cx.glob.obj_events.push(PhysicsObjectEvent::Impact { sound, mapped, point, impulse, again: flags & 0x1000 != 0 });
        let flags = flags | 0x1000;
        cx.set_i(obj.add(152), flags);
        let debris = |cx: &mut Cx| {
            let kind = cx.i(cx.ptr_at(obj.add(136)).add(52)) as u32;
            let object: [f32; 16] = std::array::from_fn(|k| cx.f(obj.add(4 * k as i32)));
            cx.glob.obj_events.push(PhysicsObjectEvent::CollisionParticles { kind, object, point, impulse });
        };
        // 0x00101300: the instance type
        match cx.i(inst.add(52)) {
            0 | 14 => {}
            15 => {
                if flags & 4 == 0 {
                    debris(cx);
                }
            }
            23 => {
                if flags & 0x40 == 0 {
                    // position copy (0x00101320 / Characters 0x0007A840: matrix row 3), the AI race distance there
                    // (first race section's raceline: FindClosestSegment + GetDistanceAlongLine), then the friction
                    // spot 0x001E18E0(pos, GC+592, GC+608, distance, msg +0, FLT_MAX); 0x40 only if its ray hit
                    cx.push_frame(16);
                    let pos = cx.lptr(0);
                    for k in 0..4 {
                        let v = cx.i(obj.add(48 + 4 * k));
                        cx.set_i(pos.add(4 * k), v);
                    }
                    let at = cx.v3(pos);
                    let distance = cx.glob.race_distance.call(at);
                    let pom = cx.ptr_at(Ptr { sp: Sp::World, off: PM_OBJECT_MANAGER });
                    let radius_low = cx.i(Ptr { sp: Sp::Gc, off: 592 });
                    let radius = cx.i(Ptr { sp: Sp::Gc, off: 608 });
                    let car = cx.i(msg);
                    let hit = f_1e18e0(cx, pom, pos, radius_low, radius, distance, car, 0x7F7F_FFFF);
                    cx.pop_frame();
                    if hit as u8 != 0 {
                        let flags = cx.i(obj.add(152));
                        cx.set_i(obj.add(152), flags | 0x40);
                    }
                    debris(cx);
                    // VsAudio::AddEnvSound(dword_6D4558, -33188540, position, 10, 100, 50, 0)
                    cx.glob.obj_events.push(PhysicsObjectEvent::Explosion { point: at });
                }
            }
            // original: passes the message as the object; ParticleManager 0x000FFC30 ignores that argument
            _ => debris(cx),
        }
        f_1e5e00(cx, obj, msg.bits());
    }

    /// the per-frame part of `PhysicsObjectManager__1E0EE0` 0x001E0EE0 (render pass) that moves objects: every
    /// visible, not hidden, awake movable gets its matrix from its rigid body (`PhysicsObjects__1E58C0`)
    pub fn sync_matrices(cx: &mut Cx, interp: f32) {
        let pom = cx.ptr_at(Ptr { sp: Sp::World, off: PM_OBJECT_MANAGER });
        if pom.is_null() {
            return;
        }
        let last = cx.i(pom.add(3216));
        for l in (0..=last.min(127)).rev() {
            let n = cx.i(pom.add(4 * l));
            let arr = cx.ptr_at(pom.add(1152 + 4 * l));
            if arr.is_null() {
                continue;
            }
            for k in (0..n).rev() {
                let obj = arr.add(160 * k);
                let fl = cx.i(obj.add(152));
                if fl & 8 == 0 && fl & 2 != 0 {
                    let mov = cx.ptr_at(obj.add(100));
                    if !mov.is_null() && f_1b41f0(cx, mov) as u8 != 0 {
                        f_1e58c0(cx, obj, interp);
                    }
                }
            }
        }
    }

    /// PhysicsObjectManager.cpp 0x001E1C00: the object whose instance name hash (`RaceTimelineSequencer__172C70`,
    /// record +0x4C) is `name`, lists 127..0 and objects last to first; null if none
    pub fn f_1e1c00(cx: &mut Cx, name: u32) -> Ptr {
        let pom = cx.ptr_at(Ptr { sp: Sp::World, off: PM_OBJECT_MANAGER });
        if pom.is_null() {
            return Ptr::NULL;
        }
        for i in (0..128).rev() {
            let arr = cx.ptr_at(pom.add(1152 + 4 * i));
            for j in (0..cx.i(pom.add(4 * i))).rev() {
                let obj = arr.add(160 * j);
                let inst = cx.ptr_at(obj.add(136));
                if !inst.is_null() && cx.i(inst.add(76)) as u32 == name {
                    return obj;
                }
            }
        }
        Ptr::NULL
    }

    /// `TimelineObjectMessage::ResetPhysicsObject`: lookup 0x001E1C00 then the object's reset 0x001E37D0 (back to
    /// its instance placement, asleep, visible unless the instance hides it, hit/broken flags cleared). False if
    /// no object has that name.
    pub fn reset_object(cx: &mut Cx, name: u32) -> bool {
        let obj = f_1e1c00(cx, name);
        if obj.is_null() {
            return false;
        }
        f_1e37d0(cx, obj);
        true
    }

    /// lookup 0x001E1C00 then the wake 0x001E4710 (what 0x001AAED0 does on contact). False if no object has that
    /// name or it is not movable.
    pub fn wake_object(cx: &mut Cx, name: u32) -> bool {
        let obj = f_1e1c00(cx, name);
        if obj.is_null() || cx.ptr_at(obj.add(100)).is_null() {
            return false;
        }
        f_1e4710(cx, obj);
        true
    }

    /// TimelinePhysics.cpp 0x0017D540 (F3EventInstanceMove, `location` z already negated)
    pub fn instance_move(cx: &mut Cx, name: u32, location: [f32; 3], velocity: [f32; 3], angular: [f32; 3]) -> bool {
        let obj = f_1e1c00(cx, name);
        if obj.is_null() {
            // "Timeline F3EventInstanceMove - target object %s not found" (+ assert)
            return false;
        }
        // matrix copy (0x00101320 / 0x000611B0), row 3 = location, written back by 0x0017DCF0
        for k in 0..3 {
            cx.set_f(obj.add(48 + 4 * k as i32), location[k]);
        }
        cx.push_frame(48);
        let (vel, ang, saved) = (cx.lptr(0), cx.lptr(16), cx.lptr(32));
        for k in 0..3 {
            cx.set_f(vel.add(4 * k as i32), velocity[k]);
            cx.set_f(ang.add(4 * k as i32), angular[k]);
        }
        let imm = cx.ptr_at(obj.add(104));
        if !imm.is_null() {
            for k in 0..4 {
                let v = cx.i(imm.add(4 * k));
                cx.set_i(saved.add(4 * k), v);
            }
            for k in 0..3 {
                cx.set_f(imm.add(4 * k as i32), location[k]);
            }
        }
        if !cx.ptr_at(obj.add(100)).is_null() {
            f_1e4710(cx, obj);
        }
        f_1e5270(cx, obj, vel, ang);
        let imm = cx.ptr_at(obj.add(104));
        if !imm.is_null() {
            for k in 0..4 {
                let v = cx.i(saved.add(4 * k));
                cx.set_i(imm.add(4 * k), v);
            }
        }
        cx.pop_frame();
        true
    }

    /// PhysicsObjectManager 0x001DE400 (attributed to VehicleManager.cpp; F3EventResetPhysicsObject with `all`,
    /// 0x0017DAA0): reset every object of the active lists of type 1/3 (0x001E37D0), then hand back every
    /// awake-listed object (+2692 array, flag 1) via 0x001E5670
    pub fn reset_all_objects(cx: &mut Cx) {
        let pom = cx.ptr_at(Ptr { sp: Sp::World, off: PM_OBJECT_MANAGER });
        if pom.is_null() {
            return;
        }
        for i in (0..128).rev() {
            let ty = cx.i(pom.add(4 * (i + 676)));
            if cx.b(pom.add(i + 1024)) == 1 && (ty == 1 || ty == 3) {
                for j in (0..cx.i(pom.add(4 * i))).rev() {
                    let obj = cx.ptr_at(pom.add(4 * (i + 288))).add(160 * j);
                    f_1e37d0(cx, obj);
                }
            }
        }
        let arr = cx.ptr_at(pom.add(4 * 673));
        if !arr.is_null() {
            for k in 0..30 {
                let o = arr.add(160 * k);
                if cx.i(o.add(152)) & 1 != 0 {
                    f_1e5670(cx, o);
                }
            }
        }
    }

    /// every object of every manager list (types 1 and 3)
    pub fn objects(cx: &Cx) -> Vec<PhysicsObjectInfo> {
        let pm = Ptr { sp: Sp::World, off: 0 };
        let pom = cx.ptr_at(pm.add(PM_OBJECT_MANAGER));
        let mut out = Vec::new();
        if pom.is_null() {
            return out;
        }
        let last = cx.i(pom.add(3216));
        for l in 0..=last.min(127) {
            let n = cx.i(pom.add(4 * l));
            let arr = cx.ptr_at(pom.add(1152 + 4 * l));
            let ty = cx.i(pom.add(2704 + 4 * l)) as u32;
            if arr.is_null() {
                continue;
            }
            for k in 0..n {
                let obj = arr.add(160 * k);
                let inst = cx.ptr_at(obj.add(136));
                if inst.is_null() {
                    continue;
                }
                let mov = cx.ptr_at(obj.add(100));
                out.push(PhysicsObjectInfo {
                    name: cx.i(inst.add(76)) as u32,
                    kind: cx.i(inst.add(52)) as u32,
                    list_type: ty,
                    matrix: std::array::from_fn(|j| cx.f(obj.add(4 * j as i32))),
                    awake: !mov.is_null() && !cx.ptr_at(mov.add(36)).is_null(),
                    flags: cx.i(obj.add(152)) as u32,
                    collides: (!mov.is_null() && !cx.ptr_at(mov.add(32)).is_null()) || !cx.ptr_at(obj.add(104)).is_null(),
                    object_type: cx.u16(inst.add(56)) as u16,
                    object_index: cx.u16(inst.add(58)) as u16,
                });
            }
        }
        out
    }
}
