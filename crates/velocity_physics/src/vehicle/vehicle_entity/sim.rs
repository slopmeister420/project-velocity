//! Running `VehicleEntity`s on the translated code: creation (`VehicleManager` + `Create`), the
//! per-frame `ComputeForcesAndTorques` call and the cars' share of `PhysicsManager::Evolve`.

use super::cx::{Cx, PhysGlobals, Ptr, Sp, TrackWorld, VeMem, VSHIFT};
use super::ov::wheel_collision_callback;
use super::{load_local_config_with_fallback, LocalConfigInputs};
use crate::c_config::CGlobalConfig;
use crate::fastcar::car::{off as caroff, CFastCar};
use crate::fastcar::mem::Mem;
use crate::fastcar::rigid_body::RigidBody;
use crate::tr_prelude::{f_1b32e0, f_1b9630, f_1c2ff0, f_1c5800, f_1cabf0, f_1d1610, f_1da8e0};

/// Handling.cfg camera globals as VehicleEntity 0x001C9070 leaves them (`PhysicsWorld::handling_camera_overrides`)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HandlingCameraOverrides {
    /// `g_ftFarClipDistance` -> 0x6850A0
    pub far_clip_distance: Option<f32>,
    /// `g_yawRatePerDegree` -> 0x68519C
    pub yaw_rate_per_degree: Option<f32>,
    /// `g_yawRelaxAngle` -> 0x6851A4
    pub yaw_relax_angle: Option<f32>,
    /// `g_yawDirectionBlend` -> 0x6851B4
    pub yaw_direction_blend: Option<f32>,
    /// `g_WheelCamFOV` -> 0x68768C
    pub wheel_cam_fov: Option<f32>,
    /// `g_WheelCamOffsetX` -> 0x687690
    pub wheel_cam_offset_x: Option<f32>,
    /// `g_WheelCamOffsetZ` -> 0x687698
    pub wheel_cam_offset_z: Option<f32>,
}

/// One pad record of `VsTrafficManager` (36 bytes from +24 per player), as `VehicleEntity__1CB530`
/// reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PadInput {
    /// +0 analogue throttle/brake axis (positive accelerates, negative brakes)
    pub axis: f32,
    /// +4 accelerator trigger 0..1
    pub accelerator: f32,
    /// +8 brake trigger 0..1
    pub brake: f32,
    /// +12 steering -1..1
    pub steer: f32,
    /// +16 buttons: 2 handbrake, 4 gear up, 8 gear down, 0x40 / 0x80 the two boost buttons
    pub buttons: u32,
}

pub const PAD_HANDBRAKE: u32 = 2;
pub const PAD_GEAR_UP: u32 = 4;
pub const PAD_GEAR_DOWN: u32 = 8;

/// PhysicsManager 0x0019FD80's arguments as TimelineExplosion 0x0017ACD0 passes the `F3EventExplosion` attributes
/// (core's `TimelineObjectMessage::Explosion` 1:1)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Explosion {
    /// the timeline object's position (VsRender 0x0005AD00)
    pub position: [f32; 3],
    /// attrib +0
    pub blast_radius: f32,
    /// +4: object velocity change at the centre (falls off linearly to 0 at the radius, / distance)
    pub force_objects: f32,
    /// +8: object angular velocity, x (0x001A0530 signed unit rand) x falloff per axis
    pub angular_force_objects: f32,
    /// +12 / +16: the same for cars (VE+272 velocity, VE+288 angular velocity)
    pub force_vehicles: f32,
    pub angular_force_vehicles: f32,
    /// +20: only used with a direction (never: `a9` is null)
    pub sector_angle: f32,
    /// +24 byte ? 1 : -1, compared with the vehicle array index (so `Ignore_Player_Car` skips index 1: original)
    pub ignore_vehicle: i32,
}

/// PhysicsObjectManager 0x001E18E0's arguments (the hydrant: GC `friction_spot_friction_multiplier_high_speed`,
/// `friction_spot_radius`, AI race distance, the car, FLT_MAX)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrictionSpot {
    pub position: [f32; 3],
    /// stored at zone +56 (not read by the wheel code)
    pub friction_high: f32,
    /// zone +52: the wheel friction test radius and the face tagging radius
    pub radius: f32,
    /// puddle record +24
    pub race_distance: f32,
    /// the car the ground ray ignores
    pub ignore_car: i32,
    /// zone +60 (the hydrant passes FLT_MAX, the AI its time + `dword_6D3784`)
    pub expiry: f32,
}

pub struct SpawnParams {
    /// vehicle database name (config selection by name hash)
    pub inputs: LocalConfigInputs,
    /// world position of the car
    pub position: [f32; 3],
    /// VS quaternion (x, y, z, w); `Create` stores its conjugate, so the car faces `rotate(q, +X)`
    pub orientation: [f32; 4],
    /// `VsTrafficManager` pad record index; the player is 0
    pub slot: i32,
}

/// `PhysicsManager::RayCast` flags (0x0019D3E0)
pub const RAY_TRACK: i32 = 1;
pub const RAY_OBJECTS: i32 = 2;
pub const RAY_CARS: i32 = 4;
pub const RAY_FIRST_HIT: i32 = 8;
pub const RAY_NORMAL: i32 = 0x10;

/// a ray-cast hit: fraction along the segment, point, normal (with [`RAY_NORMAL`]), what was hit
/// (1 track, 2 object, 3 car)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayCastHit {
    pub t: f32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub kind: i32,
}

/// one wheel after a step
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WheelState {
    /// world matrix (car+1548, row-vector 4x4, last row = position): the wheel's render matrix
    pub matrix: [f32; 16],
    pub contacts: i32,
    pub contact_point: [f32; 3],
    pub contact_normal: [f32; 3],
    /// `eSURF_*` of the face under the wheel (VE+4868)
    pub surface: u8,
    /// suspension compression
    pub suspension: f32,
    /// wheel angular speed
    pub omega: f32,
    pub sliding: bool,
}

/// One vehicle: the entity image and its car.
pub struct VehicleSlot {
    pub ve: VeMem,
    pub car: CFastCar,
    pub slot: i32,
    /// a human drives (`VsAIRaceVehicle__23CC0` in VehicleManager__1DD9F0): the car reads its pad record;
    /// otherwise the ai's controls block (VE+156)
    pub human: bool,
    /// the last `VehicleEntity__1C2FF0` output: graphics position (x y z 1) and model quaternion
    pub graphics_pose: [f32; 8],
}

/// Everything the vehicle code shares: `::GC`, executable globals, the level collision and the
/// vehicles (by id, the `PhysicsManager` vehicle array order).
pub struct PhysicsWorld {
    pub gc: CGlobalConfig,
    pub gcm: Mem,
    pub glob: PhysGlobals,
    pub track: Option<TrackWorld>,
    pub walls: Walls,
    pub vehs: Vec<Option<Box<VehicleSlot>>>,
}

/// The two wall meshes the barrier pass (`PhysicsManager__1A7060`) collides cars with, both built at race
/// load like `Armco::BuildMesh` 0x00012440 / `BarrierRender__108030`: FCMesh + `FCQuadTree::Build(8.0,
/// 2.375, 0, 1)` + `Compact`.
#[derive(Default)]
pub struct Walls {
    /// non-professional races: `BarrierRender` (`dword_705F48` +4 / +8)
    pub barrier: Option<TrackWorld>,
    /// professional races: `Armco` (`dword_6D2458` / `dword_6D245C`)
    pub armco: Option<TrackWorld>,
}

/// The translated code keeps the original's deep call chains (collision passes inside the LCP inside
/// `ResolveCollisions`) with large frames: run it on a thread with room for them.
fn with_big_stack<R: Send>(f: impl FnOnce() -> R + Send) -> R {
    std::thread::scope(|s| {
        std::thread::Builder::new()
            .name("physics".into())
            .stack_size(64 << 20)
            .spawn_scoped(s, f)
            .expect("physics thread")
            .join()
            .unwrap_or_else(|e| std::panic::resume_unwind(e))
    })
}

/// a `Cx` over the whole world; the running vehicle is `cur`
pub fn make_cx<'a>(gc: &'a CGlobalConfig, gcm: &'a Mem, glob: &'a mut PhysGlobals, track: Option<&'a TrackWorld>,
    walls: &'a Walls, vehs: &mut [Option<Box<VehicleSlot>>], cur: usize) -> Cx<'a> {
    let mut c = Cx::new(gc, gcm, glob);
    c.track = track;
    c.barrier = walls.barrier.as_ref();
    c.armco = walls.armco.as_ref();
    for v in vehs.iter_mut().flatten() {
        c.add_vehicle(&mut v.ve as *mut VeMem, &mut v.car as *mut CFastCar);
    }
    c.cur = cur;
    c
}

macro_rules! cx_of {
    ($w:expr, $cur:expr) => {
        make_cx(&$w.gc, &$w.gcm, &mut $w.glob, $w.track.as_ref(), &$w.walls, &mut $w.vehs, $cur)
    };
}

impl PhysicsWorld {
    pub fn new(gc: CGlobalConfig, track: Option<TrackWorld>) -> Self {
        let gcm = gc.to_mem();
        let mut glob = PhysGlobals::default();
        glob.xbe = visos::file::platform::read(r"\default.xbe").ok().map(std::sync::Arc::new);
        // debug-menu tweaked globals keep their values across races (.data in the original)
        crate::physics_tweaks::seed_globals(&mut glob.w);
        // static constructor of the FastCar scratch pool (`CGrowingMemoryPool`, 0x00398760)
        let pool = glob.heap.alloc(0x4000);
        glob.w.insert(0x7B5710, pool.bits() as u32);
        glob.w.insert(0x7B570C, 0x4000);
        glob.w.insert(0x7B5708, 0);
        // +0: the vehicle manager; its array is reached through the `Sp::Vlist` overrides
        glob.world.set_i(0, Ptr { sp: Sp::Vlist, off: 0 }.bits());
        // the scalar members of the PhysicsManager ctor (MaterialProperties.cpp 0x0019C230): +0x10 = 1.0,
        // +0x5F08 = 1.0 (the player's mass scale in car-car contacts, 0x001A0570), +0x5F04 = 29 free trackers
        glob.world.set_f(16, 1.0);
        glob.world.set_f(24328, 1.0);
        glob.world.set_i(24324, 29);
        if track.is_some() {
            glob.world.set_i(28, Ptr { sp: Sp::Mesh, off: 0 }.bits());
            glob.world.set_i(32, Ptr { sp: Sp::Qt, off: 0 }.bits());
        }
        let mut w = Self { gc, gcm, glob, track, walls: Walls::default(), vehs: Vec::new() };
        // the physics-object rigid body pool: 30 bodies at +2844 (ctor 0x001E3200), free list at +24204
        {
            let mut c = cx_of!(w, 0);
            for i in 0..30 {
                let rb = Ptr { sp: Sp::World, off: 2844 + 712 * i };
                crate::tr_prelude::f_1e3200(&mut c, rb);
                c.set_ptr(Ptr { sp: Sp::World, off: 24204 + 4 * i }, rb);
            }
        }
        w
    }

    /// register a compartment's physics instances (`CCompartment::RegisterInstances` 0x002E7B50 ->
    /// AddInstance -> `PhysicsObjectManager__1DFF60`): track `instance.cpt`, race `instance.cpt` /
    /// `uinstance.cpt`. Returns the number of objects made.
    pub fn register_compartment(&mut self, cpt: &[u8]) -> usize {
        with_big_stack(|| {
            let mut c = cx_of!(self, 0);
            crate::physics_objects::physics_object_manager::register_instances(&mut c, cpt)
        })
    }

    /// every placed physics object (position = matrix row 3)
    pub fn objects(&mut self) -> Vec<crate::physics_objects::PhysicsObjectInfo> {
        let c = cx_of!(self, 0);
        crate::physics_objects::physics_object_manager::objects(&c)
    }

    /// MessageManager channels 2 / 4 (object hit / woken) since the last call, in post order
    pub fn take_object_notices(&mut self) -> Vec<crate::physics_objects::PhysicsObjectNotice> {
        std::mem::take(&mut self.glob.obj_notices)
    }

    /// wake the movable object named `name` (0x001E1C00 + 0x001E4710, as a car contact does). False if unknown.
    pub fn wake_object(&mut self, name: u32) -> bool {
        with_big_stack(|| {
            let mut c = cx_of!(self, 0);
            crate::physics_objects::physics_object_manager::wake_object(&mut c, name)
        })
    }

    /// reset the placed object named `name` (instance name hash): 0x001E1C00 + 0x001E37D0. False if unknown.
    pub fn reset_object(&mut self, name: u32) -> bool {
        with_big_stack(|| {
            let mut c = cx_of!(self, 0);
            crate::physics_objects::physics_object_manager::reset_object(&mut c, name)
        })
    }

    /// TimelinePhysics 0x0017D540 (F3EventInstanceMove): move the object named `name` to `location` (z already
    /// negated), wake it if movable and set its velocity / angular velocity (0x001E5270). False if unknown.
    pub fn instance_move(&mut self, name: u32, location: [f32; 3], velocity: [f32; 3], angular: [f32; 3]) -> bool {
        with_big_stack(|| {
            let mut c = cx_of!(self, 0);
            crate::physics_objects::physics_object_manager::instance_move(&mut c, name, location, velocity, angular)
        })
    }

    /// 0x001DE400 (F3EventResetPhysicsObject with `all`, 0x0017DAA0): reset every placed object
    pub fn reset_all_objects(&mut self) {
        with_big_stack(|| {
            let mut c = cx_of!(self, 0);
            crate::physics_objects::physics_object_manager::reset_all_objects(&mut c)
        })
    }

    /// PhysicsManager 0x0019FD80 as TimelineExplosion 0x0017ACD0 calls it (no cone: `a9` null): every movable
    /// object within the blast radius is woken and pushed (+ random spin), every active car but vehicle array index
    /// `ignore_vehicle` too, then every car with VE+5084 == 0 gets VE+4876 = 96 (0x001D1170). Draws 3 `rand` per
    /// object and per car from the world's stream (`rand_state`). Returns the number of objects with a body +
    /// cars pushed.
    pub fn explosion(&mut self, e: &Explosion) -> i32 {
        let e = *e;
        with_big_stack(|| {
            let mut c = cx_of!(self, 0);
            c.push_frame(16);
            let pos = c.lptr(0);
            c.set_v3(pos, e.position);
            c.set_f(pos.add(12), 1.0);
            let world = Ptr { sp: Sp::World, off: 0 };
            let b = |f: f32| f.to_bits() as i32;
            let n = crate::tr_prelude::f_19fd80(&mut c, world, pos, b(e.blast_radius), b(e.force_objects), e.angular_force_objects,
                b(e.force_vehicles), e.angular_force_vehicles, Ptr::NULL, b(e.sector_angle), Ptr::raw(e.ignore_vehicle));
            c.pop_frame();
            n as i32
        })
    }

    /// PhysicsObjectManager 0x001E18E0 (a friction spot, as the AI's VsAIRaceVehicle 0x00019DD0 drops one): a ray
    /// from position + (0, 1, 0) to position - (0, 1, 0) (`unk_7805B0`, MaterialProperties 0x0019D350 flags 17,
    /// `ignore_car`); on a hit 0x001E1A20 registers the spot at the ground point (PhysicsManager 0x001B3500: one of
    /// the 100 zones at PhysicsManager +36, the faces in its quadtree leaf within `radius` tagged (zone + 1) << 8 for
    /// the wheel friction in 0x001BB320) and queues a [`PhysicsPuddle`](crate::physics_objects::PhysicsPuddle)
    /// (2 rand draws). True if the ray hit.
    pub fn friction_spot(&mut self, s: &FrictionSpot) -> bool {
        let s = *s;
        with_big_stack(|| {
            let mut c = cx_of!(self, 0);
            c.push_frame(16);
            let pos = c.lptr(0);
            c.set_v3(pos, s.position);
            c.set_f(pos.add(12), 1.0);
            let pom = c.ptr_at(Ptr { sp: Sp::World, off: 24 });
            let b = |f: f32| f.to_bits() as i32;
            let r = crate::tr_prelude::f_1e18e0(&mut c, pom, pos, b(s.friction_high), b(s.radius), s.race_distance, s.ignore_car, b(s.expiry));
            c.pop_frame();
            r as u8 != 0
        })
    }

    /// MaterialProperties 0x0019C390, the physics part of `GameManager::RestartRace`: every placed object reset
    /// (0x001DE400), +0x10 / +0x5F08 = 1.0, every friction spot removed (0x001B3850 clears its face tags), the 30
    /// tracker bodies' owner (+540) cleared
    pub fn restart_race(&mut self) {
        with_big_stack(|| {
            let mut c = cx_of!(self, 0);
            crate::tr_prelude::f_19c390(&mut c, Ptr { sp: Sp::World, off: 0 });
        })
    }

    /// the active friction spots (PhysicsManager +36 + 28 i: active byte, position, +52 radius, +56, +60)
    pub fn friction_spots(&self) -> Vec<(usize, [f32; 3], f32)> {
        let w = &self.glob.world;
        (0..100)
            .filter(|i| w.i(36 + 28 * i) & 0xFF != 0)
            .map(|i| (i, [w.f(40 + 28 * i), w.f(44 + 28 * i), w.f(48 + 28 * i)], w.f(52 + 28 * i)))
            .collect()
    }

    /// puddle records queued by 0x001E1A20 since the last call (graphics' PuddleRender list)
    pub fn take_puddles(&mut self) -> Vec<crate::physics_objects::PhysicsPuddle> {
        std::mem::take(&mut self.glob.puddles)
    }

    /// the AI race distance asked by the hydrant hit (0x001E1F80 case 0x17, `cx::RaceDistanceHook`); runs on the
    /// physics thread inside the step (must not lock the world). None = distance 0.
    pub fn set_race_distance_hook(&mut self, hook: Option<std::sync::Arc<std::sync::Mutex<super::cx::RaceDistanceFn>>>) {
        self.glob.race_distance = super::cx::RaceDistanceHook(hook);
    }

    /// `SlipstreamRender::SetStrength` (0x0013FB30) calls since the last call: (vehicle index, strength); from
    /// VehicleEntity 0x001C5B38 (VE+0x13E0, the player index) and PhysicsCarInput 0x001BAD9D (the car slipstreamed)
    pub fn take_slipstream(&mut self) -> Vec<(i32, f32)> {
        std::mem::take(&mut self.glob.slipstream)
    }

    /// what the object code raised since the last call (particles, impact sounds, timelines)
    pub fn take_object_events(&mut self) -> Vec<crate::physics_objects::PhysicsObjectEvent> {
        std::mem::take(&mut self.glob.obj_events)
    }

    /// the vehicle array (`PhysicsManager__19E060`): entity pointers in id order
    fn rebuild_vlist(&mut self) {
        let ids: Vec<usize> = self.vehs.iter().enumerate().filter(|(_, v)| v.is_some()).map(|(i, _)| i).collect();
        self.glob.vlist.set_i(0, Ptr { sp: Sp::Vlist, off: 16 }.bits());
        self.glob.vlist.set_i(4, ids.len() as i32);
        for (k, id) in ids.iter().enumerate() {
            self.glob.vlist.set_i(16 + 4 * k, Ptr::ve(*id).bits());
        }
    }

    /// `VehicleManager::Init` (`new VehicleEntity`, PhysicsCarInput__1B9630) then
    /// `VehicleEntity::Create` 0x001CABF0; returns the vehicle id
    pub fn spawn(&mut self, p: &SpawnParams) -> usize {
        with_big_stack(|| self.spawn_inner(p))
    }

    fn spawn_inner(&mut self, p: &SpawnParams) -> usize {
        let id = self.vehs.iter().position(|v| v.is_none()).unwrap_or(self.vehs.len());
        if id == self.vehs.len() {
            self.vehs.push(None);
        }
        assert!(id < 1 << (24 - VSHIFT), "too many vehicles");
        let lc = load_local_config_with_fallback(&p.inputs, 0).lc;
        let mut car = CFastCar { m: Mem::new(caroff::SIZE), cd: None };
        car.cd = Some(wheel_collision_callback(id));
        self.vehs[id] = Some(Box::new(VehicleSlot { ve: VeMem::new(lc, id as u16, &mut self.glob.heap), car, slot: p.slot, human: p.inputs.is_player, graphics_pose: [0.0; 8] }));
        self.rebuild_vlist();
        let mut c = cx_of!(self, id);
        let v = Ptr::ve(id);
        f_1b9630(&mut c, v);
        // VehicleManager.cpp: the vehicle's index (the race manager's id for it)
        c.set_i(v.add(5072), id as i32);
        c.set_b(v.add(4072), p.inputs.is_player as i32);
        // VehicleManager.cpp 0x001DC650: players get VE+0x13E0 = their player index (0 from the ctor otherwise)
        if p.inputs.is_player && p.slot > 0 {
            c.set_i(v.add(5088), p.slot);
        }
        // spans and performance modifiers (model locators / vehicle setup in the original)
        let i = &p.inputs;
        for (o, x) in [(4076, i.span_front_back), (4080, i.span_front), (4084, i.span_back),
            (4088, i.distance_to_front_wheels), (4092, i.distance_to_rear_wheels), (4096, i.wheel_radius_override),
            (112, i.acceleration_pm), (116, i.top_speed_pm), (100, i.nos_pm), (120, i.brake_pm), (124, i.tire_softness_pm),
            (128, i.body_roll_pm)]
        {
            c.set_f(v.add(o), x);
        }
        c.set_ptr(v.add(3552), Ptr { sp: Sp::Lc, off: (id as i32) << VSHIFT });
        c.push_frame(32);
        let pos = c.lptr(0);
        let q = c.lptr(16);
        c.set_v3(pos, p.position);
        for k in 0..4 {
            c.set_f(q.add(4 * k), p.orientation[k as usize]);
        }
        let hash = crate::vehicle_database::string_hash(&p.inputs.name) as i32;
        f_1cabf0(&mut c, v, 0, pos, 0, q, Ptr::NULL, hash);
        c.pop_frame();
        // the setup branch of 0x001CABF0 (`a6[0]`): 0x001DA8E0(VE, setup +0x18C == 0, 0); without a setup the
        // car ctor's zeros (automatic) stay, which the default `automatic: true` reproduces
        f_1da8e0(&mut c, v, (!p.inputs.automatic) as i32, 0);
        // the traffic branch of 0x001CABF0 (`a6[1]`)
        c.set_b(v.add(4074), p.inputs.traffic as i32);
        // single player: every car points at pad record 0 (VehicleManager.cpp, `+24`); players at their own
        c.set_ptr(v.add(5104), Ptr { sp: Sp::Traffic, off: 24 + 36 * p.slot.max(0) });
        c.set_b(v.add(28), 1);
        id
    }

    /// the barrier walls (`BarrierRender__108030`), or none (`nobarriers`, professional races)
    pub fn set_barrier_walls(&mut self, w: Option<TrackWorld>) {
        self.walls.barrier = w;
    }
    /// the Armco walls (`Armco::BuildMesh` 0x00012440: globals `dword_6D2458` mesh, `dword_6D245C` quadtree)
    pub fn set_armco_walls(&mut self, w: Option<TrackWorld>) {
        use super::cx::{COLL_ARMCO, COLL_SHIFT};
        let (m, q) = if w.is_some() {
            (Ptr { sp: Sp::Mesh, off: COLL_ARMCO << COLL_SHIFT }.bits() as u32, Ptr { sp: Sp::Qt, off: COLL_ARMCO << COLL_SHIFT }.bits() as u32)
        } else {
            (0, 0)
        };
        self.glob.w.insert(0x6D2458, m);
        self.glob.w.insert(0x6D245C, q);
        self.walls.armco = w;
    }

    /// this update's score-manager gains for a race car (`VsRaceManager__3D640` record +0x1C8 ->
    /// +0xA0 zone, +0xA4 nitrous), read by `ComputeForcesAndTorques` 0x001C5800 to top up the zone and
    /// NOS meters; `None` = no race record (traffic, police)
    pub fn set_score_gains(&mut self, id: usize, gains: Option<(f32, f32)>) {
        let race_id = self.slot(id).ve.m.i(5072);
        match gains {
            Some((zone, nos)) => {
                let heap = &mut self.glob.heap;
                let r = *self.glob.score_records.entry(race_id).or_insert_with(|| heap.alloc(0x1D0));
                let (blk, o) = self.glob.heap.block_mut(r.off);
                blk.set_f(o + 0xA0, zone);
                blk.set_f(o + 0xA4, nos);
            }
            None => {
                if let Some(r) = self.glob.score_records.remove(&race_id) {
                    self.glob.heap.free(r);
                }
            }
        }
    }

    pub fn slot(&self, id: usize) -> &VehicleSlot {
        self.vehs[id].as_ref().expect("no such vehicle")
    }

    /// VehicleManager__1DD9F0 for one car: `VehicleEntity::ComputeForcesAndTorques` 0x001C5800
    fn compute_forces(&mut self, id: usize, pad: &PadInput, dt: f32) {
        let (slot, human) = (self.slot(id).slot, self.slot(id).human);
        let o = (24 + 36 * slot.max(0)) as usize;
        let t = &mut self.glob.traffic;
        t.set_f(o, pad.axis);
        t.set_f(o + 4, pad.accelerator);
        t.set_f(o + 8, pad.brake);
        t.set_f(o + 12, pad.steer);
        t.set_u(o + 16, pad.buttons);
        t.set_i(o + 24, 0); // no network input id: local pad
        let mut c = cx_of!(self, id);
        let v = Ptr::ve(id);
        let is_player = c.b(v.add(4072)) != 0;
        f_1c5800(&mut c, v, dt, if is_player && human && slot >= 0 { slot } else { -1 }, 0);
    }

    /// one physics frame (`PhysicsManager__19F920` 0x0019F920, race branch): every car's
    /// `ComputeForcesAndTorques` (`VehicleManager__1DD9F0`), then `ResolveCollisions` 0x001B32E0, which
    /// runs the collision passes and `PhysicsManager::Evolve` for all cars. `pads` is indexed by pad slot.
    pub fn step(&mut self, dt: f32, pads: &[PadInput]) {
        with_big_stack(|| self.step_inner(dt, pads))
    }

    fn step_inner(&mut self, dt: f32, pads: &[PadInput]) {
        const MAX_STABLE: f32 = 0.041841004;
        assert!(dt <= MAX_STABLE, "timeStep <= maximumStableTimestep");
        let ids: Vec<usize> = self.vehs.iter().enumerate().filter(|(_, v)| v.is_some()).map(|(i, _)| i).collect();
        for id in ids.iter().copied() {
            let slot = self.slot(id).slot;
            let pad = if slot >= 0 { pads.get(slot as usize).copied().unwrap_or_default() } else { PadInput::default() };
            self.compute_forces(id, &pad, dt);
        }
        let Some(first) = ids.first().copied() else { return };
        let mut c = cx_of!(self, first);
        f_1b32e0(&mut c, Ptr { sp: Sp::World, off: 0 }, dt);
        // render-side object matrices (no render interpolation here: the frame's end state)
        crate::physics_objects::physics_object_manager::sync_matrices(&mut c, 1.0);
        drop(c);
        // SyncModels 0x001DDAE0 (the follow-camera point 0x001C5CF0 it runs next is core's)
        for id in ids.iter().copied() {
            self.sync_model(id);
        }
    }

    /// VehicleEntity.cpp 0x001C1FF0, physics part (non-network branch): `VehicleEntity__1C2FF0` = graphics
    /// position / orientation with the lean cheats, which writes the camera up offset VE+4392, the lean state
    /// VE+36/40/44 and the faked skid angle VE+4576..+4588. The model / wheel / network sync is the renderer's.
    pub(crate) fn sync_model(&mut self, id: usize) {
        let mut c = cx_of!(self, id);
        let v = Ptr::ve(id);
        if c.b(v.add(28)) == 0 {
            return;
        }
        c.push_frame(32);
        let (pos, q) = (c.lptr(0), c.lptr(16));
        f_1c2ff0(&mut c, v, 0, pos, q);
        let pose: [f32; 8] = std::array::from_fn(|k| c.f(pos.add(4 * k as i32)));
        c.pop_frame();
        drop(c);
        self.vehs[id].as_mut().expect("no such vehicle").graphics_pose = pose;
    }

    /// the camera globals VehicleEntity 0x001C9070 overwrites from Handling.cfg (only values != 0); None = never
    /// written by this world, the executable's own value stands
    pub fn handling_camera_overrides(&self) -> HandlingCameraOverrides {
        let g = |a: i32| self.glob.w.get(&a).map(|v| f32::from_bits(*v));
        HandlingCameraOverrides {
            far_clip_distance: g(0x0068_50A0),
            yaw_rate_per_degree: g(0x0068_519C),
            yaw_relax_angle: g(0x0068_51A4),
            yaw_direction_blend: g(0x0068_51B4),
            wheel_cam_fov: g(0x0068_768C),
            wheel_cam_offset_x: g(0x0068_7690),
            wheel_cam_offset_z: g(0x0068_7698),
        }
    }

    /// VE+4392: the camera up offset `VehicleEntity__1C2FF0` leaves (−|sin lean|·GC sideways_rotation_shift; 0
    /// while the lean cheat is 0, i.e. for players with the shipped Handling.cfg), read by 0x001C5CF0
    pub fn camera_up_offset(&self, id: usize) -> f32 {
        self.slot(id).ve.m.f(4392)
    }
    /// VE+5008..+5020 impact shake: amount, base amount, start time, end time (0x00066940 decays +5008)
    pub fn camera_shake(&self, id: usize) -> [f32; 4] {
        let m = &self.slot(id).ve.m;
        [m.f(5008), m.f(5012), m.f(5016), m.f(5020)]
    }
    /// VehicleEntity.cpp 0x001D1610 (TimelineCameraShaker 0x0017A5F0, `F3EventPlayerCameraShake`, on vehicle
    /// array entry 0): +5008 = +5012 = `intensity`, +5016 = race time (`VsAIDriver__15D50`, race manager
    /// 0x6F5BC8 +520, seeded here from `race_time`), +5020 = +5016 + `length`
    pub fn set_camera_shake(&mut self, id: usize, intensity: f32, length: f32, race_time: f32) {
        self.glob.w.insert(0x6F5BC8 + 520, race_time.to_bits());
        let mut c = cx_of!(self, id);
        f_1d1610(&mut c, Ptr::ve(id), intensity.to_bits() as i32, length);
    }
    /// the last `VehicleEntity__1C2FF0` output (SyncModels): graphics position and quaternion (−x −y −z w of the
    /// 0x0039D710 quaternion of the leaned model matrix, as `CompartmentManager__8DD10` stores it)
    pub fn graphics_pose(&self, id: usize) -> ([f32; 3], [f32; 4]) {
        let p = &self.slot(id).graphics_pose;
        ([p[0], p[1], p[2]], [p[4], p[5], p[6], p[7]])
    }

    /// body matrix as `PhysicsManager::Evolve` leaves it at VE+4788 (4x4, rows = axes, last row = position)
    pub fn body_matrix(&self, id: usize) -> [f32; 16] {
        let ve = &self.slot(id).ve;
        std::array::from_fn(|k| ve.m.f(4788 + 4 * k))
    }
    /// world matrix of wheel `i` (car+1548, 4x4)
    pub fn wheel_matrix(&self, id: usize, i: usize) -> [f32; 16] {
        let car = &self.slot(id).car;
        std::array::from_fn(|k| car.m.f(caroff::WHEEL_MATRIX + 64 * i + 4 * k))
    }
    pub fn velocity(&self, id: usize) -> [f32; 3] {
        self.slot(id).car.m.v3(crate::fastcar::rigid_body::off::VELOCITY)
    }
    pub fn position(&self, id: usize) -> [f32; 3] {
        self.slot(id).car.position()
    }
    /// VE+3216 (= body +2976, the CFastCar lives at VE+240): the gearbox's current gear as `Update` keeps it
    /// (> 90 = reverse/neutral encoding)
    pub fn gear(&self, id: usize) -> i32 {
        self.slot(id).car.m.i(2976)
    }
    /// engine speed from the driven wheels and the gear ratio (FastCar 0x0035D090)
    pub fn engine_speed(&mut self, id: usize) -> f32 {
        let mut c = cx_of!(self, id);
        crate::tr_prelude::f_35d090(&mut c, Ptr::car(id))
    }
    /// car+2976: FastCar gearbox gear
    pub fn car_gear(&self, id: usize) -> i32 {
        self.slot(id).car.m.i(2976)
    }

    /// remove a vehicle (its heap blocks stay allocated, as the original's pool does)
    pub fn remove(&mut self, id: usize) {
        if let Some(v) = self.vehs.get_mut(id) {
            *v = None;
        }
        self.rebuild_vlist();
    }

    /// ids of the live vehicles
    pub fn vehicle_ids(&self) -> Vec<usize> {
        self.vehs.iter().enumerate().filter(|(_, v)| v.is_some()).map(|(i, _)| i).collect()
    }

    /// `PhysicsManager::RayCast` 0x0019D3E0 from `from` to `to`. `flags`: [`RAY_TRACK`] | [`RAY_OBJECTS`] |
    /// [`RAY_CARS`], [`RAY_FIRST_HIT`], [`RAY_NORMAL`]; `ignore_car` = vehicle-array index or -1.
    pub fn ray_cast(&mut self, from: [f32; 3], to: [f32; 3], flags: i32, ignore_car: i32) -> Option<RayCastHit> {
        let cur = self.vehicle_ids().first().copied().unwrap_or(0);
        let mut c = cx_of!(self, cur);
        c.push_frame(64);
        let (t, info, a, b) = (c.lptr(0), c.lptr(4), c.lptr(32), c.lptr(48));
        c.set_f(t, 1.0);
        for k in 0..4 {
            c.set_i(info.add(4 * k), 0);
        }
        c.set_v3(a, from);
        c.set_v3(b, to);
        let hit = crate::tr_prelude::f_19d3e0(&mut c, Ptr { sp: Sp::World, off: 0 }, 0, t, info, a, b, ignore_car, flags) as u8 != 0;
        let r = RayCastHit {
            t: c.f(t),
            point: std::array::from_fn(|k| from[k] + (to[k] - from[k]) * c.f(t)),
            normal: c.v3(info),
            kind: c.i(info.add(12)),
        };
        c.pop_frame();
        hit.then_some(r)
    }

    /// per-wheel contact state after the last step (FL, FR? order: FastCar wheel index 0..3)
    pub fn wheel(&self, id: usize, i: usize) -> WheelState {
        let v = self.slot(id);
        let m = &v.car.m;
        let n = m.i(caroff::NCONTACT + 4 * i);
        let r = caroff::REC + caroff::REC_SIZE * i;
        WheelState {
            matrix: std::array::from_fn(|k| m.f(caroff::WHEEL_MATRIX + 64 * i + 4 * k)),
            contacts: n,
            contact_point: if n > 0 { m.v3(r + caroff::R_POINT) } else { [0.0; 3] },
            contact_normal: if n > 0 { m.v3(r + caroff::R_NORMAL) } else { [0.0; 3] },
            surface: v.ve.m.b(4868 + i),
            suspension: m.f(caroff::SUSP_X + 4 * i),
            omega: m.f(caroff::WHEEL_OMEGA + 4 * i),
            sliding: n > 0 && m.i(r + caroff::R_SLIDING) != 0,
        }
    }
    /// angular velocity of the chassis
    pub fn angular_velocity(&self, id: usize) -> [f32; 3] {
        self.slot(id).car.m.v3(crate::fastcar::rigid_body::off::OMEGA)
    }
    /// chassis orientation quaternion (x, y, z, w) as FastCar keeps it
    pub fn orientation(&self, id: usize) -> [f32; 4] {
        let m = &self.slot(id).car.m;
        std::array::from_fn(|k| m.f(crate::fastcar::rigid_body::off::QUAT + 4 * k))
    }

    /// one-line state dump for tests
    pub fn debug_line(&self, id: usize) -> String {
        let m = &self.slot(id).car.m;
        let g4 = |o: usize| [m.f(o), m.f(o + 4), m.f(o + 8), m.f(o + 12)];
        let n4 = |o: usize| [m.i(o), m.i(o + 4), m.i(o + 8), m.i(o + 12)];
        let ve = &self.slot(id).ve;
        format!(
            "contacts {:?} drive {:?} brake {:?} omega {:?} susp {:?} surf {:?} gear ve {} car {}",
            n4(caroff::NCONTACT), g4(caroff::DRIVE_TORQUE), g4(caroff::BRAKE_TORQUE), g4(caroff::WHEEL_OMEGA),
            g4(caroff::SUSP_X), [ve.m.b(4868), ve.m.b(4869), ve.m.b(4870), ve.m.b(4871)], ve.m.i(3216), m.i(2976)
        )
    }
}
