//! Memory model for the translated `VehicleEntity` code: every pointer of the original is a [`Ptr`]
//! (address space + byte offset) and every load/store goes through [`Cx`], so the generated code can
//! keep the binary's `*(T *)(base + N)` accesses one to one.

use std::collections::HashMap;

use crate::c_config::{CGlobalConfig, CLocalConfig};
use crate::fastcar::car::CFastCar;
use crate::fastcar::mem::Mem;

/// Byte offset of the `CFastCar` inside `VehicleEntity`.
pub const CAR: i32 = 240;
/// `VehicleEntity` size used for the image (largest member offset seen + slack).
pub const VE_SIZE: usize = 5200;
/// `CCarStateHistory` entries (`VsConfigFileRead__1B6DB0`): 5 of 208 bytes.
pub const HIST_ENTRY: i32 = 208;
pub const HIST_COUNT: i32 = 5;
/// per-vehicle spaces (entity, history, local config, heap) carry the vehicle id in the offset's high
/// bits: `off = id << VSHIFT | local`
pub const VSHIFT: u32 = 16;
pub const VMASK: i32 = (1 << VSHIFT) - 1;
#[inline]
pub fn vsplit(off: i32) -> (usize, i32) {
    ((off >> VSHIFT) as usize, off & VMASK)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sp {
    Null,
    /// stack frame of a translated function (depth)
    L(u16),
    /// the `VehicleEntity` (the car lives at +240)
    Ve,
    /// history buffer (`*(VE+3560)`)
    Hist,
    /// `CLocalConfig` image (`*(VE+3552)`)
    Lc,
    /// `::GC` image
    Gc,
    /// expression member of the local config at that offset
    LExpr,
    /// expression member of the global config at that offset
    GExpr,
    /// a global variable of the executable (offset = address)
    Glob,
    /// `operator new` heap (the offset is the block's address in [`Heap`]; the field is unused)
    Heap(u16),
    /// `VsTrafficManager` (only its pad records are modelled)
    Traffic,
    /// a function address used as a value (offset = address)
    Func,
    /// an integer used as an address (no object behind it)
    Raw,
    /// `PhysicsManager` (level collision owner)
    World,
    /// the physics manager's vehicle array (`PhysicsManager__19E060`): count at +4, entries from +16
    Vlist,
    /// `arena.msh` file image
    Mesh,
    /// `arena.qdt` file image
    Qt,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ptr {
    pub sp: Sp,
    pub off: i32,
}

impl Ptr {
    pub const NULL: Ptr = Ptr { sp: Sp::Null, off: 0 };
    pub const VE0: Ptr = Ptr { sp: Sp::Ve, off: 0 };
    pub const CAR: Ptr = Ptr { sp: Sp::Ve, off: CAR };
    /// the entity of vehicle `id`
    pub fn ve(id: usize) -> Ptr {
        Ptr { sp: Sp::Ve, off: (id as i32) << VSHIFT }
    }
    /// the `CFastCar` of vehicle `id`
    pub fn car(id: usize) -> Ptr {
        Ptr::ve(id).add(CAR)
    }
    /// vehicle id of a per-vehicle pointer
    pub fn vid(self) -> usize {
        vsplit(self.off).0
    }
    #[inline]
    pub fn add(self, n: i32) -> Ptr {
        Ptr { sp: self.sp, off: self.off.wrapping_add(n) }
    }
    pub fn gc(off: i32) -> Ptr {
        Ptr { sp: Sp::Gc, off }
    }
    pub fn gexpr(off: i32) -> Ptr {
        Ptr { sp: Sp::GExpr, off }
    }
    pub fn func(addr: i32) -> Ptr {
        Ptr { sp: Sp::Func, off: addr }
    }
    pub fn glob(addr: i32) -> Ptr {
        Ptr { sp: Sp::Glob, off: addr }
    }
    pub fn is_null(self) -> bool {
        self.sp == Sp::Null && self.off == 0
    }

    /// the pointer as a 32-bit integer, as the original moves pointers through `int`s. Reversible with
    /// [`Ptr::raw`]: globals keep their real address, every other space gets a 16 MB window above
    /// 0x40000000, so integer arithmetic on the value stays inside the object.
    pub fn bits(self) -> i32 {
        let k: u32 = match self.sp {
            Sp::Null | Sp::Raw | Sp::Glob | Sp::Func => return self.off,
            Sp::Ve => 0,
            Sp::Hist => 1,
            Sp::Lc => 2,
            Sp::Gc => 3,
            Sp::LExpr => 4,
            Sp::GExpr => 5,
            Sp::Traffic => 6,
            Sp::World => 7,
            Sp::Mesh => 8,
            Sp::Qt => 9,
            Sp::Vlist => 62,
            Sp::L(d) => 10 + d as u32,
            Sp::Heap(_) => 42 + (self.off >> 24) as u32,
        };
        let off = if let Sp::Heap(_) = self.sp { self.off & 0xFF_FFFF } else { self.off };
        assert!(k < 63 && (0..1 << 24).contains(&off), "pointer {:?} does not fit an integer handle", self);
        (0x4000_0000u32 + (k << 24) + off as u32) as i32
    }

    /// an integer the original used as an address; integers made by [`Ptr::bits`] map back to their
    /// object, executable addresses to globals, anything else is a raw value that panics if used
    pub fn raw(v: i32) -> Ptr {
        let u = v as u32;
        if u == 0 {
            return Ptr::NULL;
        }
        if (0x4000_0000..0x8000_0000).contains(&u) {
            let k = (u - 0x4000_0000) >> 24;
            let off = (u & 0xFF_FFFF) as i32;
            let sp = match k {
                0 => Sp::Ve,
                1 => Sp::Hist,
                2 => Sp::Lc,
                3 => Sp::Gc,
                4 => Sp::LExpr,
                5 => Sp::GExpr,
                6 => Sp::Traffic,
                7 => Sp::World,
                8 => Sp::Mesh,
                9 => Sp::Qt,
                10..=41 => Sp::L((k - 10) as u16),
                62 => Sp::Vlist,
                _ => return Ptr { sp: Sp::Heap(0), off: (((k - 42) << 24) | off as u32) as i32 },
            };
            return Ptr { sp, off };
        }
        if (0x1_0000..0x100_0000).contains(&u) {
            return Ptr { sp: Sp::Glob, off: v };
        }
        Ptr { sp: Sp::Raw, off: v }
    }
}

/// `operator new` / `delete`: blocks at addresses of their own (16-aligned, freed addresses reused)
#[derive(Clone, Debug, Default)]
pub struct Heap {
    blocks: std::collections::BTreeMap<i32, Mem>,
    free: Vec<(i32, i32)>,
    next: i32,
}

impl Heap {
    /// 20 handle windows of 16 MB (`Ptr::bits` codes 42..61)
    const LIMIT: i32 = 20 << 24;
    pub fn alloc(&mut self, bytes: i32) -> Ptr {
        let n = ((bytes.max(1) + 15) & !15).max(16);
        let a = if let Some(k) = self.free.iter().position(|f| f.1 >= n) {
            self.free.swap_remove(k).0
        } else {
            let a = self.next;
            self.next += n;
            assert!(self.next <= Self::LIMIT, "physics heap exhausted");
            a
        };
        self.blocks.insert(a, Mem::new(bytes.max(1) as usize));
        Ptr { sp: Sp::Heap(0), off: a }
    }
    /// a block holding a copy of `b` (a loaded file image)
    pub fn alloc_bytes(&mut self, b: &[u8]) -> Ptr {
        let p = self.alloc(b.len() as i32);
        self.blocks.insert(p.off, Mem::from_bytes(b));
        p
    }
    pub fn free(&mut self, p: Ptr) {
        if p.is_null() {
            return;
        }
        assert!(matches!(p.sp, Sp::Heap(_)), "delete of {p:?}");
        let m = self.blocks.remove(&p.off).unwrap_or_else(|| panic!("delete of a non-block {p:?}"));
        self.free.push((p.off, ((m.len_bytes() as i32 + 15) & !15).max(16)));
    }
    pub fn block(&self, a: i32) -> (&Mem, usize) {
        let (s, m) = self.blocks.range(..=a).next_back().unwrap_or_else(|| panic!("heap address {a:#x} unallocated"));
        (m, (a - s) as usize)
    }
    pub fn block_mut(&mut self, a: i32) -> (&mut Mem, usize) {
        let (s, m) = self.blocks.range_mut(..=a).next_back().unwrap_or_else(|| panic!("heap address {a:#x} unallocated"));
        let l = (a - *s) as usize;
        (m, l)
    }
}

/// Executable globals touched by the vehicle code (initial values from `default.xbe` .data).
#[derive(Clone, Debug)]
pub struct PhysGlobals {
    /// globals written so far (or seeded); everything else reads its initial value from the XBE
    pub w: HashMap<i32, u32>,
    /// the executable image (`\default.xbe`), for initial values of globals
    pub xbe: Option<std::sync::Arc<Vec<u8>>>,
    /// `VsTrafficManager` pad records: 36 bytes per player from +24
    pub traffic: Mem,
    /// CRT `rand` state (per thread in the original; the game runs physics on one thread)
    pub rand_state: u32,
    /// `PhysicsManager` image (zones at +36, 28 bytes each; mesh +28 and quadtree +32 are handles)
    pub world: Mem,
    /// the vehicle array (`Sp::Vlist`)
    pub vlist: Mem,
    /// pointers stored outside the vehicles (world, traffic, ...)
    pub xptrs: HashMap<(Sp, i32), Ptr>,
    /// `MessageManager__CA2C0` posts (type, first 16 words of the message), drained by the owner
    pub messages: Vec<(i32, Vec<i32>)>,
    /// physics-object side effects for other subsystems (particles, impact sounds, timelines), drained by the owner
    pub obj_events: Vec<crate::physics_objects::PhysicsObjectEvent>,
    /// MessageManager channels 2 / 4 (object hit / woken) for the restore point, drained by the owner
    pub obj_notices: Vec<crate::physics_objects::PhysicsObjectNotice>,
    /// `SlipstreamRender::SetStrength` calls (vehicle index, strength), drained by the owner
    pub slipstream: Vec<(i32, f32)>,
    /// the replay camera manager calls of the jump cam (0x001665E0 / 0x001665A0), answered by core
    pub jump_cam: JumpCamHook,
    pub heap: Heap,
    /// race id (VE+5072) -> stand-in race record carrying the score gains (`ov_score_record`)
    pub score_records: HashMap<i32, Ptr>,
    /// the AI race distance asked by 0x001E1F80's hydrant case
    pub race_distance: RaceDistanceHook,
    /// puddle records for graphics' PuddleRender (0x001E1A20 -> LightStreak__133750), drained by the owner
    pub puddles: Vec<crate::physics_objects::PhysicsPuddle>,
    /// the track mesh image once written (friction spots mark faces: PhysicsManager 0x001B3500 / 0x001B3850);
    /// `Sp::Mesh` set 0 reads this copy when present
    pub track_mesh: Option<Mem>,
}

impl Default for PhysGlobals {
    fn default() -> Self {
        let mut w = HashMap::new();
        for (a, v) in GLOBAL_INIT {
            w.insert(*a, *v);
        }
        Self { w, rand_state: 1, xbe: None, traffic: Mem::new(24 + 36 * 16), world: Mem::new(0x10000), vlist: Mem::new(16 + 4 * 64), xptrs: HashMap::new(), messages: Vec::new(), obj_events: Vec::new(), obj_notices: Vec::new(), slipstream: Vec::new(), jump_cam: JumpCamHook::default(), heap: Heap::default(), score_records: HashMap::new(), race_distance: RaceDistanceHook::default(), puddles: Vec::new(), track_mesh: None }
    }
}

impl PhysGlobals {
    /// the word at `addr` (written value, else the xbe image, else 0)
    pub fn word(&self, addr: i32) -> u32 {
        if let Some(v) = self.w.get(&addr) {
            return *v;
        }
        self.xbe.as_ref().and_then(|x| crate::material_properties::xbe_read(x, addr as u32, 4)).map_or(0, |b| u32::from_le_bytes(b.try_into().unwrap()))
    }
}

/// a `ReplayCamera_Manager` call made by the jump cam (VehicleEntity 0x001CCAD0 / 0x001CF0D0)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JumpCamCall {
    /// 0x001665E0(choose): fixed camera choose 0x00166B70 when `choose`, then "a camera sees the car" 0x001649A0
    CanSee { choose: bool },
    /// 0x001665A0: manager +17 = 1, then the fixed camera choose 0x00166B70
    Start,
}

/// what the replay camera manager reads from the physics while answering a [`JumpCamCall`] (the original's fixed
/// camera reads the PhysicsManager directly); served by the running step's `Cx`
pub trait JumpCamWorld {
    /// vehicle array [vehicle] (PhysicsManager 0x0019E060 + VsArray::operator[]) -> PhysicsCarInput 0x001BA270
    /// (`VehicleEntity::GetPosition`, the car body position); a vehicle not in the array gives 0
    fn car_position(&mut self, vehicle: u32) -> [f32; 3];
    /// ReplayCamera_Orbit 0x00167680(from, to, vehicle): MaterialProperties 0x0019D350(world, t, from, to, 1,
    /// vehicle) -> RayCast 0x0019D3E0 flags 23 (track | barrier walls | cars | normal), `vehicle` ignored; any hit
    fn blocked(&mut self, from: [f32; 3], to: [f32; 3], vehicle: u32) -> bool;
}

/// the hook's signature: the manager call and the step's view of the world
pub type JumpCamFn = dyn FnMut(JumpCamCall, &mut dyn JumpCamWorld) -> bool + Send;

/// core's answer to [`JumpCamCall`]s; runs on the physics thread inside the step (must not lock the world: read it
/// through the [`JumpCamWorld`] argument)
#[derive(Clone, Default)]
pub struct JumpCamHook(pub Option<std::sync::Arc<std::sync::Mutex<JumpCamFn>>>);

impl JumpCamHook {
    /// without a hook the manager has no camera: `CanSee` is false
    pub fn call(&self, c: JumpCamCall, w: &mut dyn JumpCamWorld) -> bool {
        self.0.as_ref().is_some_and(|h| (h.lock().unwrap())(c, w))
    }
}

impl std::fmt::Debug for JumpCamHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JumpCamHook({})", if self.0.is_some() { "set" } else { "none" })
    }
}

/// the AI's race distance of a point: PhysicsObjectManager 0x001E1F80 case 0x17 asks
/// `VsAIRaceSection::GetRaceline(VsAIWorld::GetFirstRaceSection())` for `FindClosestSegment` then
/// `GetDistanceAlongLine` at the hydrant (this crate cannot see the AI world); runs inside the step
pub type RaceDistanceFn = dyn FnMut([f32; 3]) -> f32 + Send;

/// core's (ai's) answer to the race distance question; without a hook the distance is 0
#[derive(Clone, Default)]
pub struct RaceDistanceHook(pub Option<std::sync::Arc<std::sync::Mutex<RaceDistanceFn>>>);

impl RaceDistanceHook {
    pub fn call(&self, at: [f32; 3]) -> f32 {
        self.0.as_ref().map_or(0.0, |h| (h.lock().unwrap())(at))
    }
}

impl std::fmt::Debug for RaceDistanceHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RaceDistanceHook({})", if self.0.is_some() { "set" } else { "none" })
    }
}

/// address, initial 32-bit value
pub const GLOBAL_INIT: &[(i32, u32)] = &[
    (0x688B70, 0x3F80_0000), // physics time scale (1.0)
    (0x6C18D8, 0),           // assert handler (none: asserts fall through)
];

/// One translated function's stack frame: the `v132[]`-style float array plus pointer slots.
#[derive(Clone, Debug)]
pub struct Frame {
    pub m: Mem,
    pub p: HashMap<i32, Ptr>,
}

/// The runtime image of a `VehicleEntity`.
pub struct VeMem {
    /// vehicle id (high bits of every per-vehicle pointer)
    pub id: u16,
    /// entity bytes except the car region (the car is a separate `CFastCar`)
    pub m: Mem,
    pub hist: Mem,
    pub lc: Mem,
    pub lcfg: CLocalConfig,
    /// pointer-valued members of the entity, by local offset
    pub ptrs: HashMap<i32, Ptr>,
    /// pointers stored anywhere else in this vehicle's spaces, by (space, full offset)
    pub xptrs: HashMap<(Sp, i32), Ptr>,
}

impl VeMem {
    pub fn new(lcfg: CLocalConfig, id: u16, heap: &mut Heap) -> Self {
        let base = (id as i32) << VSHIFT;
        let mut ptrs = HashMap::new();
        ptrs.insert(3552, Ptr { sp: Sp::Lc, off: base });
        ptrs.insert(3560, Ptr { sp: Sp::Hist, off: base });
        let mut v = Self { id, m: Mem::new(VE_SIZE), hist: Mem::new((HIST_ENTRY * HIST_COUNT) as usize), lc: lcfg.to_mem(), lcfg, ptrs, xptrs: HashMap::new() };
        // CD_box_size / CD_box_position: `new real[number_of_boxes][3]` blocks the image points at
        for (field, boxes) in [(736, v.lcfg.CD_box_size.clone()), (740, v.lcfg.CD_box_position.clone())] {
            let h = heap.alloc(12 * boxes.len().max(1) as i32);
            for (k, b) in boxes.iter().enumerate() {
                heap.block_mut(h.off).0.set_v3(12 * k, *b);
            }
            v.lc.set_i(field, h.bits());
            v.xptrs.insert((Sp::Lc, base + field as i32), h);
        }
        v
    }
}

/// The level collision the vehicle code reads: `PhysicsManager` (`VsAIRaceVehicle__24F90()`) with the
/// track mesh at +28 and its quadtree at +32, both kept as raw file images so the translated code can
/// address them as the original does.
pub struct TrackWorld {
    pub mesh: crate::fastcar::mesh::FcMesh,
    pub qt: crate::fastcar::quadtree::FcQuadTree,
    pub mesh_img: Mem,
    pub qt_img: Mem,
    pub bump: Option<crate::c_bump_map::CBumpMap>,
}

impl TrackWorld {
    /// `\Tracks\<track>\gfxrena.msh` + `arena.qdt`, read through the game-dir resolver
    pub fn load(track: &str) -> std::io::Result<Self> {
        use crate::fastcar::{mesh::FcMesh, quadtree::FcQuadTree};
        let mp = FcMesh::track_path(track);
        let qp = FcQuadTree::track_path(track);
        let mb = visos::file::platform::read(&mp)?;
        let qb = visos::file::platform::read(&qp)?;
        Self::from_bytes(mb, &qb)
    }

    /// an `arena.msh` / `arena.qdt` pair already in memory
    pub fn from_bytes(mb: Vec<u8>, qb: &[u8]) -> std::io::Result<Self> {
        use crate::fastcar::{mesh::FcMesh, quadtree::FcQuadTree};
        let bad = |e: crate::fastcar::mesh::MeshError| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string());
        let mesh = FcMesh::load(&mb).map_err(bad)?;
        let qt = FcQuadTree::load(qb).map_err(bad)?;
        // meshes without stored normals get them computed at load (FCMesh); place them after the faces
        let mut mb = mb;
        if u32::from_le_bytes(mb[28..32].try_into().unwrap()) == 0 {
            let end = 32 + 12 * mesh.num_verts as usize + mesh.num_faces as usize * if mesh.short_faces != 0 { 8 } else { 16 };
            mb.truncate(end);
            for n in &mesh.normals {
                for c in n {
                    mb.extend_from_slice(&c.to_le_bytes());
                }
            }
        }
        Ok(Self { mesh, qt, mesh_img: Mem::from_bytes(&mb), qt_img: Mem::from_bytes(qb), bump: crate::c_bump_map::CBumpMap::load().ok() })
    }

    /// a wall mesh built at race load (`Armco::BuildMesh` 0x00012440, `BarrierRender__108030`): short
    /// faces (three vertex indices + material), `FCMesh::ComputeNormals`, `FCQuadTree::Build(8.0, 2.375,
    /// 0, 1)` and `Compact`; `None` for an empty mesh (Build refuses it)
    pub fn from_walls(verts: &[[f32; 3]], faces: &[[u16; 4]]) -> Option<Self> {
        use crate::fastcar::{mesh::FcMesh, quadtree::FcQuadTree};
        let mut mesh = FcMesh {
            num_verts: verts.len() as u32,
            num_faces: faces.len() as u32,
            short_faces: 1,
            verts: verts.to_vec(),
            faces16: faces.to_vec(),
            ..Default::default()
        };
        mesh.compute_normals();
        let mut qt = FcQuadTree::new();
        if !qt.build(&mesh, 8.0, 2.375, 0, 1) {
            return None;
        }
        qt.compact();
        let mut m = Vec::new();
        for x in [mesh.num_verts, mesh.num_faces, 0, 1, 0, 0, 0, 1] {
            m.extend_from_slice(&x.to_le_bytes());
        }
        for v in mesh.verts.iter() {
            for c in v {
                m.extend_from_slice(&c.to_le_bytes());
            }
        }
        for f in &mesh.faces16 {
            for c in f {
                m.extend_from_slice(&c.to_le_bytes());
            }
        }
        for n in &mesh.normals {
            for c in n {
                m.extend_from_slice(&c.to_le_bytes());
            }
        }
        let qb = qt.to_image();
        Some(Self { mesh, qt, mesh_img: Mem::from_bytes(&m), qt_img: Mem::from_bytes(&qb), bump: None })
    }

    /// test ground: a flat square of half-size `half` at y = 0 (two faces of surface `surf`, one compact
    /// quadtree leaf)
    pub fn flat(half: f32, surf: u32) -> Self {
        let mut m = Vec::new();
        let w = |b: &mut Vec<u8>, x: u32| b.extend_from_slice(&x.to_le_bytes());
        let f = |b: &mut Vec<u8>, x: f32| b.extend_from_slice(&x.to_le_bytes());
        for x in [4, 2, 0, 0, 0, 0, 0, 1] {
            w(&mut m, x);
        }
        for (x, z) in [(-half, -half), (half, -half), (half, half), (-half, half)] {
            f(&mut m, x);
            f(&mut m, 0.0);
            f(&mut m, z);
        }
        // counter-clockwise seen from above (+y normal)
        for t in [[0u32, 3, 2], [0, 2, 1]] {
            for i in t {
                w(&mut m, i);
            }
            w(&mut m, surf);
        }
        for _ in 0..2 {
            f(&mut m, 0.0);
            f(&mut m, 1.0);
            f(&mut m, 0.0);
        }
        let mut q = vec![0u8; 96];
        let put = |q: &mut Vec<u8>, o: usize, x: u32| q[o..o + 4].copy_from_slice(&x.to_le_bytes());
        put(&mut q, 0x00, 2);
        put(&mut q, 0x04, 1);
        for (k, v) in [-half, -1.0, -half, half, 2.0 * half - 1.0, half].into_iter().enumerate() {
            put(&mut q, 0x14 + 4 * k, v.to_bits());
        }
        put(&mut q, 0x34, 8f32.to_bits());
        put(&mut q, 0x38, 2.5f32.to_bits());
        put(&mut q, 0x3C, 0);
        put(&mut q, 0x40, 2);
        put(&mut q, 0x44, 1);
        put(&mut q, 0x48, 1);
        put(&mut q, 0x4C, 8);
        put(&mut q, 0x50, 2);
        put(&mut q, 0x5C, 1);
        w(&mut q, 2 << 24);
        w(&mut q, 0);
        w(&mut q, 1);
        Self::from_bytes(m, &q).expect("flat track")
    }

    /// byte offsets of the mesh arrays inside the file image (the stored pointers are stale)
    pub fn mesh_arrays(&self) -> (i32, i32, i32) {
        let nv = self.mesh_img.i(0);
        let nf = self.mesh_img.i(4);
        let short = self.mesh_img.i(12) != 0;
        let verts = 32;
        let faces = verts + 12 * nv;
        let normals = faces + nf * if short { 8 } else { 16 };
        (verts, faces, normals)
    }
}

pub struct Cx<'a> {
    /// every live vehicle by id (null = free id); raw because a car's wheel-collision callback runs
    /// while that car is in the middle of its own `Evolve`
    pub vehs: Vec<(*mut VeMem, *mut CFastCar)>,
    /// the vehicle whose code is running (target of the hand-written overrides)
    pub cur: usize,
    pub gc: &'a CGlobalConfig,
    pub gcm: &'a Mem,
    pub glob: &'a mut PhysGlobals,
    pub track: Option<&'a TrackWorld>,
    /// the barrier (`BarrierRender`) and Armco walls: collision sets 1 and 2 of `Sp::Mesh` / `Sp::Qt`
    pub barrier: Option<&'a TrackWorld>,
    pub armco: Option<&'a TrackWorld>,
    pub frames: Vec<Frame>,
}

/// `Sp::Mesh` / `Sp::Qt` offsets: bits 22-23 pick the collision set (0 track, 1 barrier, 2 armco)
pub const COLL_SHIFT: i32 = 22;
pub const COLL_TRACK: i32 = 0;
pub const COLL_BARRIER: i32 = 1;
pub const COLL_ARMCO: i32 = 2;

fn car_off(off: i32) -> Option<usize> {
    let o = off - CAR;
    (o >= 0 && (o as usize) < crate::fastcar::car::off::SIZE).then_some(o as usize)
}

macro_rules! space {
    ($self:ident, $p:ident, $m:ident, $o:ident, $body:expr) => {{
        let $o = $p.off as usize;
        match $p.sp {
            Sp::L(d) => {
                let $m = &$self.frames[d as usize].m;
                $body
            }
            Sp::Ve => {
                let (v, l) = vsplit($p.off);
                match car_off(l) {
                    Some(co) => {
                        let $m = &$self.car_of(v).m;
                        let $o = co;
                        $body
                    }
                    None => {
                        let $m = &$self.ve_of(v).m;
                        let $o = l as usize;
                        $body
                    }
                }
            }
            Sp::Hist | Sp::Lc => {
                let (v, l) = vsplit($p.off);
                let ve = $self.ve_of(v);
                let $m = if $p.sp == Sp::Hist { &ve.hist } else { &ve.lc };
                let $o = l as usize;
                $body
            }
            Sp::Heap(_) => {
                let ($m, l) = $self.glob.heap.block($p.off);
                let $o = l;
                $body
            }
            Sp::Gc => {
                let $m = $self.gcm;
                $body
            }
            Sp::Traffic => {
                let $m = &$self.glob.traffic;
                $body
            }
            Sp::World => {
                let $m = &$self.glob.world;
                $body
            }
            Sp::Vlist => {
                let $m = &$self.glob.vlist;
                $body
            }
            Sp::Mesh => {
                let $m = match &$self.glob.track_mesh {
                    Some(t) if $p.off >> COLL_SHIFT == 0 => t,
                    _ => &$self.coll($p.off).mesh_img,
                };
                let $o = ($p.off & ((1 << COLL_SHIFT) - 1)) as usize;
                $body
            }
            Sp::Qt => {
                let $m = &$self.coll($p.off).qt_img;
                let $o = ($p.off & ((1 << COLL_SHIFT) - 1)) as usize;
                $body
            }
            _ => panic!("load from {:?}", $p),
        }
    }};
}

macro_rules! space_mut {
    ($self:ident, $p:ident, $m:ident, $o:ident, $body:expr) => {{
        let $o = $p.off as usize;
        match $p.sp {
            Sp::L(d) => {
                let $m = &mut $self.frames[d as usize].m;
                $body
            }
            Sp::Ve => {
                let (v, l) = vsplit($p.off);
                match car_off(l) {
                    Some(co) => {
                        let $m = &mut $self.car_of_mut(v).m;
                        let $o = co;
                        $body
                    }
                    None => {
                        let $m = &mut $self.ve_of_mut(v).m;
                        let $o = l as usize;
                        $body
                    }
                }
            }
            Sp::Hist | Sp::Lc => {
                let (v, l) = vsplit($p.off);
                let sp = $p.sp;
                let ve = $self.ve_of_mut(v);
                let $m = if sp == Sp::Hist { &mut ve.hist } else { &mut ve.lc };
                let $o = l as usize;
                $body
            }
            Sp::Heap(_) => {
                let ($m, l) = $self.glob.heap.block_mut($p.off);
                let $o = l;
                $body
            }
            Sp::Traffic => {
                let $m = &mut $self.glob.traffic;
                $body
            }
            Sp::World => {
                let $m = &mut $self.glob.world;
                $body
            }
            Sp::Vlist => {
                let $m = &mut $self.glob.vlist;
                $body
            }
            // only the track's face material words are written (0x001B3500 / 0x001B3850): copy on first write
            Sp::Mesh if $p.off >> COLL_SHIFT == 0 => {
                if $self.glob.track_mesh.is_none() {
                    $self.glob.track_mesh = Some($self.coll(0).mesh_img.clone());
                }
                let $m = $self.glob.track_mesh.as_mut().unwrap();
                let $o = $p.off as usize;
                $body
            }
            _ => panic!("store to {:?}", $p),
        }
    }};
}

impl<'a> Cx<'a> {
    pub fn new(gc: &'a CGlobalConfig, gcm: &'a Mem, glob: &'a mut PhysGlobals) -> Self {
        Self { vehs: Vec::new(), cur: 0, gc, gcm, glob, track: None, barrier: None, armco: None, frames: Vec::new() }
    }
    /// the collision set a `Sp::Mesh` / `Sp::Qt` offset belongs to
    pub fn coll(&self, off: i32) -> &'a TrackWorld {
        match off >> COLL_SHIFT {
            COLL_BARRIER => self.barrier.expect("no barrier collision"),
            COLL_ARMCO => self.armco.expect("no armco collision"),
            _ => self.track.expect("no track"),
        }
    }
    /// make vehicle `id` reachable
    pub fn add_vehicle(&mut self, ve: *mut VeMem, car: *mut CFastCar) {
        let id = unsafe { (*ve).id } as usize;
        if self.vehs.len() <= id {
            self.vehs.resize(id + 1, (std::ptr::null_mut(), std::ptr::null_mut()));
        }
        self.vehs[id] = (ve, car);
    }
    pub fn ve_of(&self, id: usize) -> &VeMem {
        let p = self.vehs.get(id).map(|v| v.0).unwrap_or(std::ptr::null_mut());
        assert!(!p.is_null(), "vehicle {id} is not reachable");
        unsafe { &*p }
    }
    pub fn ve_of_mut(&mut self, id: usize) -> &mut VeMem {
        let p = self.vehs.get(id).map(|v| v.0).unwrap_or(std::ptr::null_mut());
        assert!(!p.is_null(), "vehicle {id} is not reachable");
        unsafe { &mut *p }
    }
    pub fn car_of(&self, id: usize) -> &CFastCar {
        let p = self.vehs.get(id).map(|v| v.1).unwrap_or(std::ptr::null_mut());
        assert!(!p.is_null(), "car {id} is not reachable");
        unsafe { &*p }
    }
    pub fn car_of_mut(&mut self, id: usize) -> &mut CFastCar {
        let p = self.vehs.get(id).map(|v| v.1).unwrap_or(std::ptr::null_mut());
        assert!(!p.is_null(), "car {id} is not reachable");
        unsafe { &mut *p }
    }
    /// the running vehicle's entity
    pub fn ve(&self) -> &VeMem {
        self.ve_of(self.cur)
    }
    pub fn ve_mut(&mut self) -> &mut VeMem {
        self.ve_of_mut(self.cur)
    }
    /// the `CFastCar` a car pointer refers to
    pub fn car_at(&mut self, car: Ptr) -> &mut CFastCar {
        assert!(car.sp == Sp::Ve && vsplit(car.off).1 == CAR, "not a car pointer: {car:?}");
        self.car_of_mut(car.vid())
    }

    // ---- frames
    pub fn push_frame(&mut self, bytes: usize) {
        // the debug build fills its frames with 0xCC
        let mut m = Mem::new(bytes);
        for o in (0..bytes).step_by(4) {
            m.set_u(o, 0xCCCC_CCCC);
        }
        self.frames.push(Frame { m, p: HashMap::new() });
    }
    pub fn pop_frame(&mut self) {
        self.frames.pop();
    }
    fn top(&self) -> &Frame {
        self.frames.last().expect("frame")
    }
    fn top_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("frame")
    }
    pub fn lptr(&self, off: i32) -> Ptr {
        Ptr { sp: Sp::L(self.frames.len() as u16 - 1), off }
    }
    pub fn lf(&self, off: usize) -> f32 {
        self.top().m.f(off)
    }
    pub fn li(&self, off: usize) -> i32 {
        self.top().m.i(off)
    }
    pub fn set_lf(&mut self, off: usize, v: f32) {
        self.top_mut().m.set_f(off, v)
    }
    pub fn set_li(&mut self, off: usize, v: i32) {
        let t = self.top_mut();
        t.m.set_i(off, v);
        if !t.p.is_empty() {
            t.p.remove(&(off as i32));
        }
    }
    /// `LODWORD(v[n])` holding a pointer
    pub fn lpp(&self, n: i32) -> Ptr {
        // a slot written as an integer holds a pointer handle (`Ptr::bits`)
        self.top().p.get(&(4 * n)).copied().unwrap_or_else(|| Ptr::raw(self.top().m.i(4 * n as usize)))
    }
    pub fn set_lpp(&mut self, n: i32, p: Ptr) {
        let t = self.top_mut();
        t.p.insert(4 * n, p);
        t.m.set_i(4 * n as usize, p.bits());
    }

    // ---- loads / stores
    pub fn f(&self, p: Ptr) -> f32 {
        if p.sp == Sp::Glob {
            return f32::from_bits(self.glob_u(p));
        }
        space!(self, p, m, o, m.f(o))
    }
    pub fn i(&self, p: Ptr) -> i32 {
        if p.sp == Sp::Glob {
            return self.glob_u(p) as i32;
        }
        if p.sp == Sp::Mesh && matches!(p.off & ((1 << COLL_SHIFT) - 1), 16 | 20 | 24 | 28) {
            // the file image keeps stale pointers: hand out handles to the arrays instead
            return self.ptr_at(p).bits();
        }
        if p.sp == Sp::Qt && matches!(p.off & ((1 << COLL_SHIFT) - 1), 12 | 16 | 84 | 88) {
            // same for the quadtree's index / node arrays (LACoast's arena.qdt is the one non-compact tree)
            return self.ptr_at(p).bits();
        }
        space!(self, p, m, o, m.i(o))
    }
    pub fn b(&self, p: Ptr) -> i32 {
        if p.sp == Sp::Glob {
            return (self.glob_u(p) & 0xFF) as i32;
        }
        space!(self, p, m, o, m.b(o) as i32)
    }
    pub fn i16(&self, p: Ptr) -> i32 {
        let lo = self.b(p) as u32;
        let hi = self.b(p.add(1)) as u32;
        (lo | hi << 8) as u16 as i16 as i32
    }
    pub fn set_f(&mut self, p: Ptr, v: f32) {
        // PHYSNANTRAP=1: panic at the first NaN store
        if v.is_nan() && v.to_bits() != 0xFFFF_FFFF && crate::fastcar::mem::nan_trap() {
            panic!("NaN stored at {p:?}");
        }
        self.forget_ptr(p);
        if p.sp == Sp::Glob {
            self.glob.w.insert(p.off, v.to_bits());
            return;
        }
        space_mut!(self, p, m, o, m.set_f(o, v))
    }
    pub fn set_i(&mut self, p: Ptr, v: i32) {
        self.forget_ptr(p);
        if p.sp == Sp::Glob {
            self.glob.w.insert(p.off, v as u32);
            return;
        }
        space_mut!(self, p, m, o, m.set_i(o, v))
    }
    pub fn set_b(&mut self, p: Ptr, v: i32) {
        if p.sp == Sp::Glob {
            let w = self.glob_u(p);
            self.glob.w.insert(p.off, (w & !0xFF) | (v as u32 & 0xFF));
            return;
        }
        space_mut!(self, p, m, o, m.set_b(o, v as u8))
    }
    pub fn u16(&self, p: Ptr) -> i32 {
        self.i16(p) & 0xFFFF
    }
    pub fn set_u16(&mut self, p: Ptr, v: i32) {
        self.set_i16(p, v)
    }
    pub fn set_i16(&mut self, p: Ptr, v: i32) {
        self.set_b(p, v & 0xFF);
        self.set_b(p.add(1), (v >> 8) & 0xFF);
    }
    fn forget_ptr(&mut self, p: Ptr) {
        match p.sp {
            Sp::Ve => {
                let (v, l) = vsplit(p.off);
                if car_off(l).is_none() {
                    let ve = self.ve_of_mut(v);
                    if !ve.ptrs.is_empty() {
                        ve.ptrs.remove(&l);
                    }
                }
            }
            Sp::L(d) => {
                self.frames[d as usize].p.remove(&p.off);
            }
            Sp::Hist | Sp::Lc => {
                let ve = self.ve_of_mut(p.vid());
                if !ve.xptrs.is_empty() {
                    ve.xptrs.remove(&(p.sp, p.off));
                }
            }
            _ => {
                if !self.glob.xptrs.is_empty() {
                    self.glob.xptrs.remove(&(p.sp, p.off));
                }
            }
        }
    }
    fn glob_u(&self, p: Ptr) -> u32 {
        if let Some(v) = self.glob.w.get(&p.off) {
            return *v;
        }
        let xbe = self.glob.xbe.as_ref().unwrap_or_else(|| panic!("global {:#X} read without the XBE image", p.off));
        // .bss (beyond the raw data) starts zeroed
        crate::material_properties::xbe_read(xbe, p.off as u32, 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).unwrap_or(0)
    }

    /// a pointer stored at `p`
    pub fn ptr_at(&self, p: Ptr) -> Ptr {
        let stored = match p.sp {
            Sp::Hist | Sp::Lc => self.ve_of(p.vid()).xptrs.get(&(p.sp, p.off)),
            Sp::Ve | Sp::L(_) => None,
            _ => self.glob.xptrs.get(&(p.sp, p.off)),
        };
        if let Some(v) = stored {
            return *v;
        }
        match p.sp {
            Sp::Ve => {
                let (v, l) = vsplit(p.off);
                if car_off(l).is_none() {
                    if let Some(x) = self.ve_of(v).ptrs.get(&l) {
                        return *x;
                    }
                }
                // integers read through a pointer-typed expression come back as `Sp::Raw` (bits intact)
                match self.i(p) {
                    0 => Ptr::NULL,
                    x => Ptr::raw(x),
                }
            }
            Sp::L(d) => self.frames[d as usize].p.get(&p.off).copied().unwrap_or_else(|| Ptr::raw(self.frames[d as usize].m.i(p.off as usize))),
            Sp::Lc => match self.i(p) {
                0 => Ptr::NULL,
                v => Ptr { sp: Sp::LExpr, off: (p.off & !VMASK) | v },
            },
            Sp::World => match p.off {
                28 if self.track.is_some() => Ptr { sp: Sp::Mesh, off: 0 },
                32 if self.track.is_some() => Ptr { sp: Sp::Qt, off: 0 },
                _ => match self.i(p) {
                    0 => Ptr::NULL,
                    x => Ptr::raw(x),
                },
            },
            Sp::Mesh => {
                let t = self.coll(p.off);
                let base = p.off & !((1 << COLL_SHIFT) - 1);
                let (v, f, n) = t.mesh_arrays();
                let short = t.mesh_img.i(12) != 0;
                match p.off - base {
                    16 => Ptr { sp: Sp::Mesh, off: base + v },
                    20 if !short => Ptr { sp: Sp::Mesh, off: base + f },
                    24 if short => Ptr { sp: Sp::Mesh, off: base + f },
                    28 => Ptr { sp: Sp::Mesh, off: base + n },
                    _ => Ptr::NULL,
                }
            }
            Sp::Qt => {
                // image = 96-byte FCQuadTree, nodes (+84: 40 B each, or +88: compact words), indices (+12 / +16)
                let img = &self.coll(p.off).qt_img;
                let base = p.off & !((1 << COLL_SHIFT) - 1);
                let compact = img.i(92) != 0;
                let short = img.i(8) != 0;
                let idx = 96 + img.i(4) * if compact { 4 } else { 40 };
                match p.off - base {
                    84 if !compact => Ptr { sp: Sp::Qt, off: base + 96 },
                    88 if compact => Ptr { sp: Sp::Qt, off: base + 96 },
                    12 if !short => Ptr { sp: Sp::Qt, off: base + idx },
                    16 if short => Ptr { sp: Sp::Qt, off: base + idx },
                    _ => Ptr::NULL,
                }
            }
            Sp::Gc => match self.gcm.i(p.off as usize) {
                0 => Ptr::NULL,
                v => Ptr { sp: Sp::GExpr, off: v },
            },
            _ => match self.i(p) {
                0 => Ptr::NULL,
                x => Ptr::raw(x),
            },
        }
    }
    pub fn set_ptr(&mut self, p: Ptr, v: Ptr) {
        match p.sp {
            Sp::Ve => {
                self.set_i(p, v.bits());
                let (id, l) = vsplit(p.off);
                if car_off(l).is_none() {
                    self.ve_of_mut(id).ptrs.insert(l, v);
                }
            }
            Sp::L(d) => {
                let f = &mut self.frames[d as usize];
                f.p.insert(p.off, v);
                f.m.set_i(p.off as usize, v.bits());
            }
            Sp::Hist | Sp::Lc => {
                self.set_i(p, v.bits());
                self.ve_of_mut(p.vid()).xptrs.insert((p.sp, p.off), v);
            }
            _ => {
                self.set_i(p, v.bits());
                self.glob.xptrs.insert((p.sp, p.off), v);
            }
        }
    }

    // ---- ++/--
    pub fn preinc_i(&mut self, p: Ptr) -> i32 {
        let v = self.i(p).wrapping_add(1);
        self.set_i(p, v);
        v
    }
    pub fn predec_i(&mut self, p: Ptr) -> i32 {
        let v = self.i(p).wrapping_sub(1);
        self.set_i(p, v);
        v
    }
    pub fn postinc_i(&mut self, p: Ptr) -> i32 {
        let v = self.i(p);
        self.set_i(p, v.wrapping_add(1));
        v
    }
    pub fn postdec_i(&mut self, p: Ptr) -> i32 {
        let v = self.i(p);
        self.set_i(p, v.wrapping_sub(1));
        v
    }
    pub fn preinc_b(&mut self, p: Ptr) -> i32 {
        let v = (self.b(p) + 1) & 0xFF;
        self.set_b(p, v);
        v
    }

    pub fn ld(&self, off: usize) -> f64 {
        let m = &self.top().m;
        f64::from_bits(m.u(off) as u64 | (m.u(off + 4) as u64) << 32)
    }
    pub fn set_ld(&mut self, off: usize, v: f64) {
        let b = v.to_bits();
        let m = &mut self.top_mut().m;
        m.set_u(off, b as u32);
        m.set_u(off + 4, (b >> 32) as u32);
    }
    pub fn d(&self, p: Ptr) -> f64 {
        f64::from_bits(self.i(p) as u32 as u64 | (self.i(p.add(4)) as u32 as u64) << 32)
    }
    pub fn set_d(&mut self, p: Ptr, v: f64) {
        let b = v.to_bits();
        self.set_i(p, b as u32 as i32);
        self.set_i(p.add(4), (b >> 32) as u32 as i32);
    }
    pub fn q(&self, p: Ptr) -> i64 {
        (self.i(p) as u32 as u64 | (self.i(p.add(4)) as u32 as u64) << 32) as i64
    }
    pub fn set_q(&mut self, p: Ptr, v: i64) {
        self.set_i(p, v as u32 as i32);
        self.set_i(p.add(4), (v >> 32) as u32 as i32);
    }
    pub fn preinc_f(&mut self, p: Ptr) -> f32 {
        let v = (self.f(p) as f64 + 1.0) as f32;
        self.set_f(p, v);
        v
    }
    pub fn postinc_b(&mut self, p: Ptr) -> i32 {
        let v = self.b(p);
        self.set_b(p, (v + 1) & 0xFF);
        v
    }
    pub fn memset(&mut self, p: Ptr, v: i32, n: i32) {
        for k in 0..n {
            self.set_b(p.add(k), v & 0xFF);
        }
    }
    pub fn memcpy(&mut self, d: Ptr, s: Ptr, n: i32) {
        for k in 0..n {
            let v = self.b(s.add(k));
            self.set_b(d.add(k), v);
        }
    }

    // ---- vectors
    pub fn v3(&self, p: Ptr) -> [f32; 3] {
        [self.f(p), self.f(p.add(4)), self.f(p.add(8))]
    }
    pub fn set_v3(&mut self, p: Ptr, v: [f32; 3]) {
        self.set_f(p, v[0]);
        self.set_f(p.add(4), v[1]);
        self.set_f(p.add(8), v[2]);
    }
    pub fn copy_words(&mut self, dst: Ptr, src: Ptr, n: i32) {
        for k in 0..n {
            let v = self.i(src.add(4 * k));
            self.set_i(dst.add(4 * k), v);
        }
    }

    /// `AUDIO_Resample__3AAFF0(&out, 0, expr, vars)`: 0 on success
    pub fn eval_expr(&mut self, out: Ptr, e: Ptr, vars: Ptr) -> i32 {
        let r = {
            let ex = match e.sp {
                Sp::LExpr => self.ve_of(e.vid()).lcfg.expr_at((e.off & VMASK) as usize),
                Sp::GExpr => self.gc.expr_at(e.off as usize),
                _ => None,
            };
            let Some(ex) = ex else { return 1 };
            let n = ex.image.var_count();
            let v: Vec<f32> = (0..n as i32).map(|k| self.f(vars.add(4 * k))).collect();
            ex.eval(&v)
        };
        match r {
            Some(r) => {
                self.set_f(out, r);
                0
            }
            None => 1,
        }
    }
}

/// compound bit operators of the translated code (`x |= y` etc.)
pub trait IntOps {
    fn bitor_(self, b: i32) -> i32;
    fn bitand_(self, b: i32) -> i32;
    fn bitxor_(self, b: i32) -> i32;
    fn shl_(self, b: i32) -> i32;
    fn shr_(self, b: i32) -> i32;
}
impl IntOps for i32 {
    fn bitor_(self, b: i32) -> i32 {
        self | b
    }
    fn bitand_(self, b: i32) -> i32 {
        self & b
    }
    fn bitxor_(self, b: i32) -> i32 {
        self ^ b
    }
    fn shl_(self, b: i32) -> i32 {
        self.wrapping_shl(b as u32)
    }
    fn shr_(self, b: i32) -> i32 {
        self.wrapping_shr(b as u32)
    }
}
