//! CarDamage.cpp (0x0006AFA0-0x00078320): per-car body damage driven by collision message 0.
//!
//! Nine damage bones sit around the body (corners, sides, roof). A collision is assigned to the closest bone
//! whose facing matches the impact normal (0x00070F40) and kept as that bone's pending zone; the next update
//! (0x0006BBA0) pushes the bone inwards up to its shell radius and dents every vertex weighted to it
//! (0x00072580 -> 0x0006FF80). Pushing a bone past 2/3 and 1 of its radius breaks lights (0x00070950); the
//! accumulated push per window cracks the glass in three stages (0x00070B30).
//!
//! Model space is the CarRender model space the original deforms: x lateral (headlight locator x > 0), y up,
//! z along the car (front bones at +z in the default table, impact facings 0x00070DB0).

use bevy::math::{Mat4, Vec3};

pub const MAX_DEFORMABLE_CARS: usize = 8;
pub const NUM_BONES: usize = 9;
pub const MAX_IMPACTS: usize = 56;
pub const MAX_WINDOW_CRACKS: usize = 12;

const IMPACT_SCALE: f32 = 7.0; // dword_685288
const MIN_IMPACT: f32 = 10.0; // dword_685274
const PUSH_SCALE: f32 = 0.001; // dword_685278
const ATTR1_SCALE: f32 = 1.5; // flt_68527C
const DENT_REACH: f32 = 0.75; // dword_685280
const DENT_RADIUS: f32 = 6.0; // dword_685284
const FACING_DEG: f32 = 65.0; // flt_685270
const BIG: f32 = 9999999827968.0;

/// one damage bone (40 bytes at +140 + 40*i)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageBone {
    pub rest: [f32; 3],  // +0
    pub cur: [f32; 3],   // +16
    pub radius: f32,     // +32 shell radius: how far the bone may be pushed
    pub hinge: i16,      // +36 hinged panel (2 bonnet, 4 boot, -1 none)
    pub linked: i8,      // +38 the other bone of the hinge
    pub count: i8,       // +39 dents applied (max 6)
}

const fn bone(x: f32, y: f32, z: f32, radius: f32, hinge: i16, linked: i8) -> DamageBone {
    DamageBone { rest: [x, y, z], cur: [x, y, z], radius, hinge, linked, count: 0 }
}

/// CarDamage static ctor 0x00421930 (unk_6D63C8): the template bone table
pub const DEFAULT_BONES: [DamageBone; NUM_BONES] = [
    bone(0.5, 0.25, 2.0, 0.25, 2, 7),
    bone(0.8, 0.25, 0.55, 0.12, -1, -1),
    bone(0.8, 0.25, -0.55, 0.13, -1, -1),
    bone(0.5, 0.25, -2.0, 0.14, 4, 4),
    bone(-0.5, 0.25, -2.0, 0.15, 4, 3),
    bone(-0.8, 0.25, -0.55, 0.14, -1, -1),
    bone(-0.8, 0.25, 0.55, 0.12, -1, -1),
    bone(-0.5, 0.25, 2.0, 0.25, 2, 0),
    bone(0.0, 0.8, -0.35, 0.025, -1, -1),
];

/// collision message 0 as CarDamage reads it (80 bytes, copied by 0x000764A0 / 0x00076560)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DamageMessage {
    pub a: i32,              // +0
    pub b: i32,              // +4
    pub point: [f32; 3],     // +8 world
    pub normal: [f32; 3],    // +24 world
    pub impulse: f32,        // +40
    pub impulse_vec: [f32; 3], // +56 world
    pub scale: f32,          // +72
    pub material: i8,        // +76
}

impl From<&crate::vehicle::vehicle_entity::sim_ai::CarCollision> for DamageMessage {
    fn from(c: &crate::vehicle::vehicle_entity::sim_ai::CarCollision) -> Self {
        DamageMessage { a: c.a, b: c.b, point: c.point, normal: c.normal, impulse: c.impact, impulse_vec: c.impulse,
            scale: c.scale, material: c.material }
    }
}

/// a bone's pending impact (144 bytes at +808 + 144*i): the message and world->model at impact time
#[derive(Clone, Copy, Debug, Default)]
pub struct DamageZone {
    pub msg: DamageMessage,
    pub world_to_model: [f32; 16], // +80, row-vector
}

/// an applied dent (92 bytes at +2112 + 92*i)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageImpact {
    pub push: [f32; 3],       // +0 bone push applied to the vertices
    pub point: [f32; 3],      // +16 model-space dent centre (nearest weighted vertex)
    pub bone_before: [f32; 3], // +32
    pub bone_after: [f32; 3], // +48
    pub direction: [f32; 3],  // +64 impulse vector, model space
    pub magnitude: f32,       // +80 impulse * 7
    pub bone: u16,            // +84
    pub hinge: i16,           // +86 (-1 for body dents)
    pub time: f32,            // +88
}

/// a window crack (8 bytes at +7284 + 8*i)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowCrack {
    pub window: u16, // 0 front, 1 rear, 2 / 3 sides (0x00070B30 bone map)
    pub stage: u16,  // 0..2
    pub time: f32,
}

/// one vertex group the original walks with VertexIterator (sub-model mesh part), in iteration order:
/// the main model's (car data +1092) sub-models, then each model of the +1208 list
#[derive(Clone, Debug, Default)]
pub struct DamageMesh {
    /// mesh -> model space (sub-model matrix, row-vector 4x4)
    pub matrix: [f32; 16],
    /// vertex element 0 (position, mesh space)
    pub positions: Vec<[f32; 3]>,
    /// vertex element 1 (the original moves it by -1.5 x the dent; normal channel)
    pub attr1: Vec<[f32; 3]>,
}

/// what the damage needs from the car model (CarRender car data)
#[derive(Clone, Debug, Default)]
pub struct DamageModel {
    /// +768 `l_head` locator (z negated, as CarRender stores it)
    pub head: [f32; 3],
    /// +896 `l_brake` locator (z negated)
    pub brake: [f32; 3],
    pub meshes: Vec<DamageMesh>,
}

/// what the renderer must do (shader swaps, particles, counters)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CarDamageEvent {
    /// CarDamageFX 0x00078880 (collision particles), every message for the car; impulse already halved
    CollisionFx { vehicle: i32, msg: DamageMessage },
    /// 0x00073700 (light, repair 0): light 0 right headlight, 1 left headlight, 2 right brake, 3 left brake;
    /// glass particles 0x000781C0, broken shaders, CarRender 0x001157B0(light, 1)
    LightBroken { vehicle: i32, light: u32 },
    /// 0x00073700 (light, repair 1)
    LightRepaired { vehicle: i32, light: u32 },
    /// 0x000730C0 (window, 0, stage): crack texture stage
    WindowCrack { vehicle: i32, window: u32, stage: u32 },
    /// 0x000730C0 (window, 1, 2)
    WindowRepaired { vehicle: i32, window: u32 },
    /// mesh vertices changed (dent or repair)
    MeshChanged { vehicle: i32, mesh: usize },
}

#[derive(Clone, Copy, Debug, Default)]
struct WeightInfo {
    bones: [u16; 2], // +0 the two closest bones
    orig: [f32; 3],  // +4 original mesh-space position
}

/// CCarDamage (0x1EF4 bytes, ctor VsAIRaceVehicle__23620 + 0x0006B8B0)
#[derive(Clone, Debug)]
pub struct CarDamage {
    pub car_id: i32,                  // +0
    pub bones: [DamageBone; NUM_BONES], // +140
    pub pending: u32,                 // +804 bit per bone
    pub zones: [DamageZone; NUM_BONES], // +808
    pub n_impacts: i32,               // +2104 hinge buckles
    pub n_buckles: i32,               // +2108 body dents
    pub impacts: Vec<DamageImpact>,   // +2112, n_impacts + n_buckles entries
    pub window_damage: [f32; 4],      // +7268
    pub cracks: Vec<WindowCrack>,     // +7264 count, +7284
    pub repair_a: i32,                // +7380 animated repair cursor (-1 idle)
    pub repair_b: i32,                // +7384
    pub flags: u32,                   // +7636: 1 deform, 2 window cracks
    pub flags2: u8,                   // +7640: 1 lights, 4 boot hinge, 8 no body deformation
    pub bonnet_side: i32,             // +7644 bonnet hinge side (-1 none)
    pub boot_side: i32,               // +7648
    /// car data +2256 (CarRender): 1 front / 2 rear lights intact, 4 bonnet, 0x10 boot
    pub car_flags: u32,
    pub bounds_min: [f32; 3],         // car data +996
    pub bounds_max: [f32; 3],         // car data +1012
    pub model: DamageModel,
    weights: Vec<WeightInfo>,         // +136 m_DmgWeightInfo, +132 count
    pub events: Vec<CarDamageEvent>,
    /// player headlight breaks not yet counted into [`CarDamageStats`]
    headlight_fx: u32,
}

/// byte_6D6808 counters for the player car (0x00076900 bonnet, 0x00076970 headlights; reset 0x000766F0)
#[derive(Clone, Copy, Debug, Default)]
pub struct CarDamageStats {
    pub bonnets: u32,
    pub headlights: u32,
}

// ---- vector helpers (VsVector4: SSE f32 arithmetic, Length / Dot3 return x87) ----
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn mul(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f64 {
    crate::fastcar::vsmath::dot87(a, b)
}
fn length(a: [f32; 3]) -> f64 {
    dot(a, a).sqrt()
}
fn normalise(a: [f32; 3]) -> [f32; 3] {
    let l = length(a) as f32;
    [a[0] / l, a[1] / l, a[2] / l]
}
/// Renderer 0x002ABDA0: point by row-vector matrix
fn xform_point(m: &[f32; 16], p: [f32; 3]) -> [f32; 3] {
    Mat4::from_cols_array(m).transform_point3(Vec3::from_array(p)).to_array()
}
/// Renderer 0x002ABE80: direction by row-vector matrix
fn xform_dir(m: &[f32; 16], v: [f32; 3]) -> [f32; 3] {
    Mat4::from_cols_array(m).transform_vector3(Vec3::from_array(v)).to_array()
}
fn inverse(m: &[f32; 16]) -> [f32; 16] {
    Mat4::from_cols_array(m).inverse().to_cols_array()
}
/// CarDamage 0x000762F0: (b - a) * t + a
fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|k| ((b[k] as f64 - a[k] as f64) * t as f64 + a[k] as f64) as f32)
}
/// CarDamage 0x00076490: radians to degrees
fn degrees(r: f32) -> f64 {
    r as f64 * 57.295776f32 as f64
}

impl CarDamage {
    /// 0x0006B8B0 (ctor part) + VsAIRaceVehicle__23620
    pub fn new(car_id: i32) -> Self {
        let mut bones = DEFAULT_BONES;
        // 0x0006B8B0: hinge types of the four corner bones
        bones[0].hinge = 2;
        bones[7].hinge = 2;
        bones[3].hinge = 4;
        bones[4].hinge = 4;
        CarDamage {
            car_id, bones, pending: 0, zones: [DamageZone::default(); NUM_BONES], n_impacts: 0, n_buckles: 0,
            impacts: Vec::new(), window_damage: [0.0; 4], cracks: Vec::new(), repair_a: -1, repair_b: -1,
            flags: 1 | 2, flags2: 1, bonnet_side: -1, boot_side: -1, car_flags: 0x17, bounds_min: [0.0; 3],
            bounds_max: [0.0; 3], model: DamageModel::default(), weights: Vec::new(), events: Vec::new(), headlight_fx: 0,
        }
    }

    /// 0x0006D150: build bones and vertex weights from the car model; `deform` = false sets +7640 bit 8 (no body
    /// dents; the original passes VsAIVehicle +0x34C != 0 for race cars, police never deform)
    pub fn setup(&mut self, model: DamageModel, deform: bool) {
        self.flags2 = (if deform { 0 } else { 8 }) | (self.flags2 & 0xF7);
        self.bones = DEFAULT_BONES;
        self.repair_a = -1;
        self.repair_b = -1;
        self.n_impacts = 0;
        self.n_buckles = 0;
        self.pending = 0;
        self.bonnet_side = -1;
        self.boot_side = -1;
        self.impacts.clear();
        self.flags2 &= !4;
        self.window_damage = [0.0; 4];
        self.cracks.clear();
        self.model = model;
        self.bones_from_locators();
        self.weights.clear();
        if self.flags2 & 8 == 0 {
            // 0x0006D920 counts, 0x0006DA40 weights + bounds (both start at the origin), 0x0006EDC0
            let (mut mn, mut mx) = ([0.0f32; 3], [0.0f32; 3]);
            for i in 0..self.model.meshes.len() {
                self.weigh_mesh(i, &mut mn, &mut mx);
            }
            self.bones_from_bounds(mn, mx);
            self.bounds_min = mn;
            self.bounds_max = mx;
        }
        // with bit 8 the original weighs only the bonnet model for its hinge (0x0006E7F0, not ported)
    }

    /// 0x0006CE00: corner bones from the head / brake light locators, side bones between them
    fn bones_from_locators(&mut self) {
        let (head, brake) = (self.model.head, self.model.brake);
        if head[0] <= 0.0 {
            log::warn!("Unexpected Negative X in headlight pos (Fritz removed Assert)");
        }
        let b = &mut self.bones;
        b[7].rest = head;
        b[7].cur = head;
        b[0].rest = head;
        b[0].cur = head;
        b[4].rest = brake;
        b[4].cur = brake;
        b[3].rest = brake;
        b[3].cur = brake;
        for i in [0, 3] {
            b[i].rest[0] *= -1.0;
            b[i].cur[0] *= -1.0;
        }
        self.sides_between(self.bones[7].rest, self.bones[4].rest);
    }

    /// shared tail of 0x0006CE00 / 0x0006EDC0: bones 6/5 at 1/3 and 2/3 from `front` to `rear`, 1/2 mirrored
    fn sides_between(&mut self, front: [f32; 3], rear: [f32; 3]) {
        let b = &mut self.bones;
        b[6].rest = lerp(front, rear, 0.33333);
        b[6].cur = b[6].rest;
        b[1].rest = b[6].rest;
        b[1].cur = b[6].rest;
        b[1].rest[0] *= -1.0;
        b[1].cur[0] *= -1.0;
        b[5].rest = lerp(front, rear, 0.66666);
        b[5].cur = b[5].rest;
        b[2].rest = b[5].rest;
        b[2].cur = b[5].rest;
        b[2].rest[0] *= -1.0;
        b[2].cur[0] *= -1.0;
    }

    /// 0x0006EDC0: corner bones moved to the body bounds (headlight height kept)
    fn bones_from_bounds(&mut self, mn: [f32; 3], mx: [f32; 3]) {
        let b = &mut self.bones;
        b[7].rest[0] = mx[0];
        b[7].rest[2] = mx[2];
        b[0].rest[0] = mn[0];
        b[0].rest[2] = mx[2];
        b[7].cur = b[7].rest;
        b[0].cur = b[0].rest;
        b[4].rest[0] = mx[0];
        b[4].rest[2] = mn[2];
        b[3].rest[0] = mn[0];
        b[3].rest[2] = mn[2];
        b[4].cur = b[4].rest;
        b[3].cur = b[3].rest;
        let y = b[7].rest[1];
        self.sides_between([mx[0], y, mx[2]], [mx[0], y, mn[2]]);
    }

    /// 0x0006DA40 (one mesh): each vertex's two closest bones and the model bounds
    fn weigh_mesh(&mut self, mi: usize, mn: &mut [f32; 3], mx: &mut [f32; 3]) {
        let mesh = &self.model.meshes[mi];
        for p in &mesh.positions {
            let pc = xform_point(&mesh.matrix, *p);
            for k in 0..3 {
                if pc[k] < mn[k] {
                    mn[k] = pc[k];
                }
                if mx[k] < pc[k] {
                    mx[k] = pc[k];
                }
            }
            let mut w = WeightInfo { bones: [0; 2], orig: *p };
            let mut best = [BIG; 2];
            for (bi, b) in self.bones.iter().enumerate() {
                let d = sub(b.rest, pc);
                let d2 = dot(d, d) as f32;
                for i in 0..2 {
                    if best[i] > d2 {
                        let mut j = 1;
                        while j > i {
                            best[j] = best[j - 1];
                            w.bones[j] = w.bones[j - 1];
                            j -= 1;
                        }
                        best[i] = d2;
                        w.bones[i] = bi as u16;
                        break;
                    }
                }
            }
            self.weights.push(w);
        }
    }

    /// 0x00076270: animated repair running
    pub fn repairing(&self) -> bool {
        self.repair_a != -1 || self.repair_b != -1
    }

    /// 0x0006B820 (message 0 listener): `model_to_world` = the car's model matrix now (VehicleEntity 0x001C2FF0)
    pub fn on_message(&mut self, msg: &DamageMessage, model_to_world: &[f32; 16]) {
        if msg.a == self.car_id || msg.b == self.car_id {
            if !self.repairing() {
                self.take_impact(msg, model_to_world, msg.b == self.car_id);
            }
        }
    }

    /// 0x00070F40: assign the hit to a bone and keep the strongest per bone until the next update
    fn take_impact(&mut self, m: &DamageMessage, model_to_world: &[f32; 16], is_b: bool) {
        let mut msg = *m;
        msg.impulse *= 0.5;
        self.events.push(CarDamageEvent::CollisionFx { vehicle: self.car_id, msg });
        if self.flags & 1 == 0 || self.n_impacts + self.n_buckles == MAX_IMPACTS as i32 {
            return;
        }
        let inv = inverse(model_to_world);
        let mut normal = msg.normal;
        if is_b {
            normal = mul(normal, -1.0);
        }
        let ln = xform_dir(&inv, normal);
        let lp = xform_point(&inv, msg.point);
        let mut best = BIG;
        let mut bone = -1i32;
        for (i, b) in self.bones.iter().enumerate() {
            let d = sub(b.rest, lp);
            let d2 = dot(d, d) as f32;
            if best > d2 {
                best = d2;
                bone = i as i32;
            }
        }
        debug_assert!(bone != -1, "nClosestBone != -1");
        let mut accept = false;
        let r = Self::facing(bone, ln);
        if r != 0 {
            if r > 0 {
                let side = match bone {
                    0 => Some(1),
                    3 => Some(2),
                    4 => Some(5),
                    7 => Some(6),
                    _ => None,
                };
                if let Some(s) = side {
                    bone = s;
                    if Self::facing(s, ln) == 0 {
                        accept = true;
                    }
                }
            }
        } else {
            accept = true;
        }
        if !accept {
            return;
        }
        let bi = bone as usize;
        let replace = self.pending & (1 << bi) == 0 || msg.impulse > self.zones[bi].msg.impulse;
        if replace {
            self.zones[bi] = DamageZone { msg, world_to_model: inv };
            self.pending |= 1 << bi;
            if is_b {
                self.zones[bi].msg.normal = mul(self.zones[bi].msg.normal, -1.0);
            }
        }
    }

    /// 0x00070DB0: 1 when the model-space normal is more than 65 degrees off the bone's facing
    fn facing(bone: i32, n: [f32; 3]) -> i32 {
        let axis = match bone {
            0 | 7 => [0.0, 0.0, -1.0],
            1 | 2 => [1.0, 0.0, 0.0],
            3 | 4 => [0.0, 0.0, 1.0],
            5 | 6 => [-1.0, 0.0, 0.0],
            8 => [0.0, -1.0, 0.0],
            _ => [0.0; 3],
        };
        let n = normalise(n);
        let a = crate::fastcar::vsmath::acos(dot(n, axis) as f32) as f32;
        let deg = degrees(a) as f32;
        // 0x0006FE50 sign of a (acos >= 0), truncated
        if (FACING_DEG as f64) < (deg as f64).abs() { if a.to_bits() as i32 >= 0 { 1 } else { -1 } } else { 0 }
    }

    /// 0x0006BBA0 (CarDamage update, from the manager 0x0006ADC0): apply the pending zones
    pub fn update(&mut self, time: f32) {
        if self.repairing() {
            // 0x00075B00 animated repair (debug menu only, 0x00075AB0): not ported
            return;
        }
        for i in 0..NUM_BONES {
            if self.pending & (1 << i) != 0 {
                let z = self.zones[i];
                self.apply_zone(&z, i, time);
            }
        }
        self.pending = 0;
    }

    /// 0x00072580: push bone `bi` by the zone's impact and dent its vertices
    fn apply_zone(&mut self, z: &DamageZone, bi: usize, time: f32) {
        let mag = z.msg.impulse * IMPACT_SCALE;
        if self.flags & 1 == 0 || !(MIN_IMPACT <= mag) {
            return;
        }
        if self.n_impacts + self.n_buckles == MAX_IMPACTS as i32 && self.cracks.len() == MAX_WINDOW_CRACKS {
            return;
        }
        let m = &z.world_to_model;
        let mut dir = z.msg.impulse_vec;
        if z.msg.a != self.car_id {
            dir = mul(dir, -1.0);
        }
        let ldir = xform_dir(m, dir);
        let ln = xform_dir(m, z.msg.normal);
        let mut lp = xform_point(m, z.msg.point);
        let mut ok = true;
        // `Single Bone Damage` tweak (dword_6852BC) is -1: every bone
        self.snap_to_vertex(ln, bi, &mut lp);
        let mut push = mul(ln, mag * PUSH_SCALE);
        let b = self.bones[bi];
        let before = b.cur;
        let after = add(b.cur, mul(push, 1.0));
        let d = length(sub(after, b.rest)) as f32;
        self.lights(bi, d);
        self.windows(bi, d, time);
        let dent;
        if d < self.bones[bi].radius {
            if self.n_buckles < 58 {
                self.bones[bi].cur = after;
            }
            dent = true;
        } else {
            let push0 = push;
            let rem = (self.bones[bi].radius as f64 - length(sub(self.bones[bi].cur, self.bones[bi].rest))) as f32;
            debug_assert!(rem + 0.001 >= 0.0, "(fMinDistShell + 0.001f) >= 0.f");
            if rem <= 0.0 {
                ok = false;
            } else {
                let len = length(sub(after, self.bones[bi].cur)) as f32;
                push = mul(push, rem / len);
                self.bones[bi].cur = add(self.bones[bi].cur, mul(push, 1.0));
            }
            dent = !self.hinge_buckle(push0, bi, z, before);
        }
        if dent && self.n_buckles < 54 && ok && self.bones[bi].count < 6 {
            let mut counter = 0usize;
            for mi in 0..self.model.meshes.len() {
                self.dent_mesh(bi as u16, mi, push, lp, &mut counter, false);
            }
            let idx = (self.n_impacts + self.n_buckles) as usize;
            let rec = DamageImpact { push, point: lp, bone_before: before, bone_after: after, direction: ldir,
                magnitude: mag, bone: bi as u16, hinge: -1, time };
            // the original writes slot n_impacts + n_buckles unchecked (only n_buckles < 54 is tested)
            if idx < MAX_IMPACTS {
                self.impacts.truncate(idx);
                self.impacts.push(rec);
            }
            self.n_buckles += 1;
            self.bones[bi].count += 1;
        }
    }

    /// 0x00071E50: buckle the hinged bonnet / boot instead of denting. The hinge panels (setup 0x0006E7F0 /
    /// 0x0006E9E0 / 0x0006EBD0, buckle 0x0006F5E0) are not ported, so no panel ever buckles and the body dent
    /// runs, as for a car without hinge records.
    fn hinge_buckle(&mut self, _push: [f32; 3], _bone: usize, _z: &DamageZone, _before: [f32; 3]) -> bool {
        false
    }

    /// 0x00071680 + 0x00071B90: dent centre = the vertex weighted to `bone` nearest a probe slid along the
    /// bone's edge by the impact angle
    fn snap_to_vertex(&self, normal: [f32; 3], bone: usize, out: &mut [f32; 3]) {
        let b = &self.bones;
        let (axis, mut slide, span) = match bone {
            0 | 7 => ([0.0, 0.0, -1.0], [if bone == 7 { -1.0 } else { 1.0 }, 0.0, 0.0], b[7].rest[0] as f64 - b[0].rest[0] as f64),
            1 | 2 => ([-1.0, 0.0, 0.0], [0.0, 0.0, if bone == 1 { -1.0 } else { 1.0 }], b[1].rest[2] as f64 - b[2].rest[2] as f64),
            3 | 4 => ([0.0, 0.0, 1.0], [if bone == 4 { 1.0 } else { -1.0 }, 0.0, 0.0], b[3].rest[0] as f64 - b[4].rest[0] as f64),
            5 | 6 => ([1.0, 0.0, 0.0], [0.0, 0.0, if bone == 5 { -1.0 } else { 1.0 }], b[5].rest[2] as f64 - b[6].rest[2] as f64),
            _ => ([0.0, -1.0, 0.0], [0.0; 3], 0.0),
        };
        let span = span as f32;
        let n = normalise(normal);
        let a = crate::fastcar::vsmath::acos(dot(n, axis) as f32) as f32;
        let deg = (degrees(a) as f32).clamp(0.0 - FACING_DEG, FACING_DEG);
        let t = deg / FACING_DEG;
        let half = (span as f64 / 2.0) as f32;
        let off = mul(mul(slide, half), t);
        slide = mul(slide, half);
        let probe = sub(add(b[bone].rest, slide), off);
        if self.flags2 & 8 != 0 {
            return;
        }
        let mut best = BIG;
        let mut counter = 0usize;
        for mesh in &self.model.meshes {
            for p in &mesh.positions {
                let w = self.weights[counter];
                if w.bones.contains(&(bone as u16)) {
                    let pc = xform_point(&mesh.matrix, *p);
                    let d = length(sub(pc, probe)) as f32;
                    if best > d {
                        best = d;
                        *out = pc;
                    }
                }
                counter += 1;
            }
        }
    }

    /// 0x0006FF80: move the vertices weighted to `bone` along the push (`repair` pulls them back)
    fn dent_mesh(&mut self, bone: u16, mi: usize, push: [f32; 3], centre: [f32; 3], counter: &mut usize, repair: bool) -> u32 {
        if self.flags2 & 8 != 0 {
            return 0;
        }
        let l = length(push) as f32;
        let reach = l * DENT_REACH;
        let radius = l * DENT_RADIUS;
        let dir = [push[0] / l, push[1] / l, push[2] / l]; // 0x000763F0
        let mesh = &mut self.model.meshes[mi];
        let inv = inverse(&mesh.matrix);
        let mut hits = 0;
        let mut changed = false;
        for v in 0..mesh.positions.len() {
            let w = self.weights[*counter];
            if w.bones.contains(&bone) {
                let pc = xform_point(&mesh.matrix, mesh.positions[v]);
                let rel = sub(pc, centre);
                let dist = length(rel) as f32;
                let along = dot(dir, rel) as f32;
                let perp2 = dist * dist - along * along;
                let perp = if perp2 >= 0.0 { (perp2 as f64).sqrt() as f32 } else { 0.0 };
                if along <= reach {
                    let mut wt = ((radius - perp) / radius) * 0.65;
                    if wt >= 0.0 {
                        let ang = (perp * 50.26544) / 0.3;
                        wt = ((crate::fastcar::vsmath::cos(ang) + 1.0) * 0.125 + wt as f64) as f32;
                    } else {
                        wt = 0.0;
                    }
                    debug_assert!((0.0..=1.0).contains(&wt), "(fPerpWeight >= 0.f) && (fPerpWeight <= 1.f)");
                    let np = if repair {
                        let s = ((reach - along) * wt) / (1.0 - wt);
                        sub(pc, mul(dir, s))
                    } else {
                        add(pc, mul(dir, (reach - along) * wt))
                    };
                    let delta = sub(np, pc);
                    if let Some(a) = mesh.attr1.get_mut(v) {
                        *a = sub(*a, mul(delta, ATTR1_SCALE));
                    }
                    mesh.positions[v] = xform_point(&inv, np);
                    hits += 1;
                    changed = true;
                }
            }
            *counter += 1;
        }
        if changed {
            self.events.push(CarDamageEvent::MeshChanged { vehicle: self.car_id, mesh: mi });
        }
        hits
    }

    /// 0x00070950: lights go out at 2/3 of a corner's shell, break at the full shell
    fn lights(&mut self, bi: usize, d: f32) {
        let r = self.bones[bi].radius;
        if d >= (r * 2.0) / 3.0 {
            match bi {
                0 => { self.bonnet_side = 0; self.car_flags &= !1; }
                3 => { self.boot_side = 0; self.car_flags &= !2; }
                4 => { self.boot_side = 1; self.car_flags &= !2; }
                7 => { self.bonnet_side = 1; self.car_flags &= !1; }
                _ => {}
            }
        }
        if d >= r {
            let light = match bi {
                0 => Some(1),
                3 => Some(3),
                4 => Some(2),
                7 => Some(0),
                _ => None,
            };
            if let Some(light) = light {
                self.light(light, false);
                if self.car_id == 0 && (bi == 0 || bi == 7) {
                    self.headlight_fx += 1; // 0x00076970
                }
            }
        }
    }

    /// 0x00073700 (shader swaps / particles / CarRender 0x001157B0 belong to the renderer)
    fn light(&mut self, light: u32, repair: bool) {
        if self.flags2 & 1 != 0 {
            let vehicle = self.car_id;
            self.events.push(if repair { CarDamageEvent::LightRepaired { vehicle, light } } else { CarDamageEvent::LightBroken { vehicle, light } });
        }
    }

    /// 0x00070B30: accumulate the push per window, crack at radius x 1 / 6 / 12
    fn windows(&mut self, bi: usize, d: f32, time: f32) {
        if self.cracks.len() >= MAX_WINDOW_CRACKS {
            return;
        }
        let w: i32 = match bi {
            0 | 7 => 0,
            1 | 2 => 3,
            3 | 4 => 1,
            5 | 6 => 2,
            _ => -1,
        };
        if w == -1 {
            return;
        }
        let r = self.bones[bi].radius;
        let thr = [r, (r as f64 * 6.0) as f32, (r as f64 * 12.0) as f32];
        let wi = w as usize;
        self.window_damage[wi] += d;
        for (k, t) in thr.iter().enumerate() {
            if *t > self.window_damage[wi] - d && self.window_damage[wi] >= *t {
                self.cracks.push(WindowCrack { window: w as u16, stage: k as u16, time });
                self.window(w as u32, false, k as u32);
                break;
            }
        }
    }

    /// 0x000730C0 (window, repair, stage)
    fn window(&mut self, window: u32, repair: bool, stage: u32) {
        if self.flags & 2 != 0 {
            let vehicle = self.car_id;
            self.events.push(if repair { CarDamageEvent::WindowRepaired { vehicle, window } } else { CarDamageEvent::WindowCrack { vehicle, window, stage } });
        }
    }

    /// 0x00075780: repair (`full` also resets the hinge types, dent / crack counters and window damage)
    pub fn repair(&mut self, full: bool) {
        for b in &mut self.bones {
            b.count = 0;
            b.cur = b.rest;
        }
        if full {
            self.bones[0].hinge = 2;
            self.bones[7].hinge = 2;
            self.bones[3].hinge = 4;
            self.bones[4].hinge = 4;
            self.n_impacts = 0;
            self.n_buckles = 0;
            self.impacts.clear();
            self.window_damage = [0.0; 4];
            self.cracks.clear();
        }
        self.repair_a = -1;
        self.repair_b = -1;
        self.pending = 0;
        for w in [0, 1, 3, 2] {
            self.window(w, true, 2);
        }
        for l in [0, 1, 3, 2] {
            self.light(l, true);
        }
        self.car_flags |= 0x17;
        // 0x00075470: positions back to the setup copy, element 1 corrected by 1.5 x the (model - original) offset
        if self.flags2 & 8 == 0 {
            let mut counter = 0usize;
            for mi in 0..self.model.meshes.len() {
                let mesh = &mut self.model.meshes[mi];
                for v in 0..mesh.positions.len() {
                    let pc = xform_point(&mesh.matrix, mesh.positions[v]);
                    let orig = self.weights[counter].orig;
                    if let Some(a) = mesh.attr1.get_mut(v) {
                        *a = add(*a, mul(sub(pc, orig), ATTR1_SCALE));
                    }
                    mesh.positions[v] = orig;
                    counter += 1;
                }
                self.events.push(CarDamageEvent::MeshChanged { vehicle: self.car_id, mesh: mi });
            }
        }
    }

    /// how far bone `i` has been pushed in from its rest position
    pub fn bone_push(&self, i: usize) -> f32 {
        length(sub(self.bones[i].cur, self.bones[i].rest)) as f32
    }
}

/// the CarDamage manager (byte_6D63A0: up to 8 deformable cars) + the FX counters (byte_6D6808)
#[derive(Clone, Debug, Default, bevy::prelude::Resource)]
pub struct CarDamageManager {
    pub cars: Vec<CarDamage>,
    pub stats: CarDamageStats,
    /// dword_6D6364: accumulated update time
    pub clock: f32,
}

impl CarDamageManager {
    /// 0x0006AFA0 (Add) + 0x0006B8B0 / 0x0006D150: register a car and build its damage model
    pub fn add(&mut self, car_id: i32, model: DamageModel, deform: bool) -> &mut CarDamage {
        self.cars.retain(|c| c.car_id != car_id);
        debug_assert!(self.cars.len() < MAX_DEFORMABLE_CARS, "m_dCount <= MAX_DEFORMABLE_CARS");
        let mut cd = CarDamage::new(car_id);
        cd.setup(model, deform);
        self.cars.push(cd);
        self.cars.last_mut().unwrap()
    }
    /// 0x0006BB20: unregister
    pub fn remove(&mut self, car_id: i32) {
        self.cars.retain(|c| c.car_id != car_id);
    }
    pub fn get(&self, car_id: i32) -> Option<&CarDamage> {
        self.cars.iter().find(|c| c.car_id == car_id)
    }
    pub fn get_mut(&mut self, car_id: i32) -> Option<&mut CarDamage> {
        self.cars.iter_mut().find(|c| c.car_id == car_id)
    }
    /// message 0 to every car's listener (0x0006B820); `matrix(id)` = that car's model -> world now
    pub fn on_message(&mut self, msg: &DamageMessage, matrix: impl Fn(i32) -> Option<[f32; 16]>) {
        for c in &mut self.cars {
            if msg.a == c.car_id || msg.b == c.car_id {
                if let Some(m) = matrix(c.car_id) {
                    c.on_message(msg, &m);
                }
            }
        }
    }
    /// 0x0006ADC0: update every car
    pub fn update(&mut self, dt: f32) {
        self.clock += dt;
        let time = self.clock;
        for c in &mut self.cars {
            c.update(time);
            self.stats.headlights += std::mem::take(&mut c.headlight_fx);
        }
    }
    /// 0x0006B090 (`GameManager::RestartRace`) + 0x000766F0: full repair of every car
    pub fn repair_all(&mut self) {
        for c in &mut self.cars {
            c.repair(true);
        }
        self.stats = CarDamageStats::default();
    }
    /// all queued renderer events
    pub fn take_events(&mut self) -> Vec<CarDamageEvent> {
        self.cars.iter_mut().flat_map(|c| std::mem::take(&mut c.events)).collect()
    }
}
