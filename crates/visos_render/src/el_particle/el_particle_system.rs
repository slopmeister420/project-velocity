//! elParticleSystem.cpp (0x002CECC0..0x002D2E60): `elParticleType` (the parameter set, 0x78 bytes + a 432-byte
//! "effect" block at +8 — the layout the SFP files store) and `elParticleSystem` (one emitter instance, 0x9C
//! bytes) with its particles. The vtables / smart pointers / STL vector helpers of the file are replaced by Rust
//! ownership; everything that computes is ported here.

use bevy::math::{Mat4, Vec3, Vec4};

use super::el_particle_control_curve::{ColourCurve, ControlCurve};
use super::vs_pool_allocator::{BlockBuffer, EmitterShape, ParticlePool, VsRand};
use crate::effect_draw::{EffectShader, EffectTexture};

/// +148 rotation mode: `MODE_FREELY_ROTATING` billboards (the only mode `setRotationParams` accepts)
pub const MODE_FREELY_ROTATING: u32 = 0;
/// stretched along the node velocity (renderer slot 1, 0x00317E90)
pub const MODE_VELOCITY: u32 = 1;
/// ribbon through the trail nodes (renderer slot 2, 0x00317AA0; one node more than segments)
pub const MODE_TRAIL: u32 = 2;

/// particle header bytes and node bytes (0x002D1A00: +80 = 48, +84 = 32)
pub const HEADER_BYTES: usize = 48;
pub const NODE_BYTES: usize = 32;

/// +172, 24 bytes (`elParticleSystem__2D1CB0` defaults 0.1, 1, 0): trail length in seconds, size factor of the last
/// node, colour change over the trail
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrailParams {
    pub time: f32,
    pub size_end: f32,
    pub colour: Vec4,
}

/// +392, 20 bytes (0x002D1CF0 defaults 1, 1, 1, 1, 0, 0): texture frames u × v, frame rate range (frames per
/// second = frames · rand(lo, hi)), +408 random start frame, +410 random jump on every new frame
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimParams {
    pub frames_u: i32,
    pub frames_v: i32,
    pub rate_lo: f32,
    pub rate_hi: f32,
    pub random_start: u16,
    pub random_jump: u16,
}

/// one collision plane (56 bytes in the type's vector, 0x002D1DE0 defaults): +0 bounce (v += n·(−1 − √k)·(n·v)),
/// +4 plane (normal, w = −n·p), +20 name, +48 kind (0 bounce, 1 kill, other = report only), +52 report flag
#[derive(Clone, Debug, PartialEq)]
pub struct CollisionPlane {
    pub bounce: f32,
    pub plane: Vec4,
    pub name: String,
    pub kind: u32,
    pub report: bool,
}

impl CollisionPlane {
    /// LightStreak.cpp-range 0x00135C20: plane through `point` with `normal`
    pub fn new(normal: Vec3, point: Vec3, bounce: f32, kind: u32) -> Self {
        Self { bounce, plane: normal.extend(-normal.dot(point)), name: String::new(), kind, report: false }
    }

    /// VsBounds 0x00057200: n·p + w
    pub fn distance(&self, p: Vec3) -> f32 {
        self.plane.truncate().dot(p) + self.plane.w
    }
}

/// the 432-byte effect block (`new 0x1B0`, defaults 0x002D1400)
#[derive(Clone, Debug, PartialEq)]
pub struct EffectData {
    /// +0 name (debug copy), +4 name crc
    pub name: String,
    pub crc: u32,
    /// +8 particles alive at most
    pub max_particles: i32,
    /// +12 seconds (≤ 0 = forever)
    pub lifetime: f32,
    /// +16 acceleration
    pub gravity: Vec3,
    /// +32 start velocity, +48 its spread
    pub velocity: Vec3,
    pub velocity_spread: Vec3,
    /// +64 the systems' default shape
    pub shape: EmitterShape,
    /// +144 1: velocity + ball · spread per axis, 0: velocity ± spread.x along its direction
    pub velocity_mode: u32,
    /// +148 `MODE_*`
    pub rotation_mode: u32,
    /// +152 start angle lo/hi, spin lo/hi (degrees)
    pub rotation: [f32; 4],
    /// +168 trail segments
    pub segments: i32,
    pub trail: TrailParams,
    /// +196 0 continuous (rate), 1 one burst of +236 at start
    pub emission_mode: u32,
    /// +200 particles per second over the system's age, +232 jitter of the spacing (0..1)
    pub rate: ControlCurve,
    pub rate_jitter: f32,
    pub burst: i32,
    /// +240 size over a particle's age, +272 lowest random size scale (scale = rand(1, this))
    pub size: ControlCurve,
    pub size_random: f32,
    /// +276 colour over age, +356 random colour added scaled by rand(0, 1)
    pub colour: ColourCurve,
    pub colour_random: Vec4,
    /// +372 velocity damping per second
    pub drag: f32,
    /// +376 random kicks per second, +380 their size
    pub walk_rate: f32,
    pub walk_size: f32,
    /// +384 blend mode, +388 texture name (SFP types)
    pub blend: u32,
    pub texture: String,
    pub anim: AnimParams,
    /// +424 bit 0: velocity in the shape matrix's frame
    pub flags: u32,
}

/// `elParticleType` (0x78 bytes): the effect block plus what is derived from it
#[derive(Clone, Debug)]
pub struct ElParticleType {
    /// +8 (owned when +12)
    pub data: EffectData,
    /// +16 collision planes
    pub planes: Vec<CollisionPlane>,
    /// +36 texture name, +64 blend, +68 shader (+72 owned)
    pub texture: String,
    pub blend: u32,
    pub shader: EffectShader,
    /// +76 nodes per particle (segments, +1 in trail mode; 1..255)
    pub nodes: i32,
    /// +88 particle bytes = 48 + nodes · 32
    pub particle_bytes: usize,
    /// +92 seconds per trail node, +96 size ratio per node, +100 colour step per node
    pub node_time: f32,
    pub node_size_ratio: f32,
    pub node_colour: Vec4,
    /// +116 frames u · v
    pub frames: i32,
}

/// CarDamageFX.cpp-range 0x00078FE0: the (w, x, y, z) argument order of the inline vector constructor
pub fn wxyz(w: f32, x: f32, y: f32, z: f32) -> Vec4 {
    Vec4::new(x, y, z, w)
}

impl ElParticleType {
    /// elParticleSystem.cpp 0x002D0890 — `new elParticleType(maxParticles, numSegments, mode, name)`: defaults 0x002D13A0 / 0x002D1400,
    /// the default shader 0x002D0D70("", 2), derived values 0x002D1A00
    pub fn new(max_particles: i32, segments: i32, rotation_mode: u32, name: &str) -> Self {
        let data = EffectData {
            name: name.to_string(),
            crc: visos::handy::handy_crc::crc32(name.as_bytes(), 0),
            max_particles,
            lifetime: 0.5,
            gravity: Vec3::ZERO,
            velocity: Vec3::ZERO,
            velocity_spread: Vec3::ZERO,
            shape: EmitterShape::default(),
            velocity_mode: 1,
            rotation_mode,
            rotation: [0.0; 4],
            segments,
            trail: TrailParams { time: f32::from_bits(0x3DCC_CCCD), size_end: 1.0, colour: Vec4::ZERO },
            emission_mode: 0,
            rate: ControlCurve::constant(1.0),
            rate_jitter: 0.0,
            burst: 0,
            size: ControlCurve::constant(1.0),
            size_random: 0.0,
            colour: ColourCurve::constant(Vec4::ONE),
            colour_random: Vec4::ZERO,
            drag: 0.0,
            walk_rate: 0.0,
            walk_size: 0.0,
            blend: 2,
            texture: String::new(),
            anim: AnimParams { frames_u: 1, frames_v: 1, rate_lo: 1.0, rate_hi: 1.0, random_start: 0, random_jump: 0 },
            flags: 0,
        };
        let mut t = Self {
            data,
            planes: Vec::new(),
            texture: String::new(),
            blend: 2,
            shader: EffectShader::new(EffectTexture::White, 2),
            nodes: 1,
            particle_bytes: HEADER_BYTES + NODE_BYTES,
            node_time: 0.0,
            node_size_ratio: 0.0,
            node_colour: Vec4::ZERO,
            frames: 1,
        };
        t.set_shader("", 2);
        t.derive();
        t
    }

    /// elParticleSystem.cpp 0x002D1A00
    fn derive(&mut self) {
        let extra = (self.data.rotation_mode == MODE_TRAIL) as i32;
        self.nodes = self.data.segments + extra;
        if self.nodes > 255 {
            log::warn!("Warning: elParticleType::elParticleType: too many nodes: clamping...");
            self.nodes = 255;
            self.data.segments = self.nodes - extra;
        }
        if self.data.segments < 1 {
            log::warn!("Warning: elParticleType::elParticleType: m_numSegments less than 1: setting to 1...");
            self.data.segments = 1;
            self.nodes = self.data.segments + extra;
        }
        self.particle_bytes = HEADER_BYTES + self.nodes as usize * NODE_BYTES;
        self.set_trail(self.data.trail);
        self.set_anim(self.data.anim);
    }

    /// elParticleSystem.cpp 0x002D0D70 `elParticleType::setShader(texture, blend)`: "particleshader" = stage flags 0, the texture
    /// `Particle\<name>` (none: the default white texture), the blend, z test without z write
    pub fn set_shader(&mut self, texture: &str, blend: u32) {
        let tex = if texture.is_empty() { EffectTexture::White } else { EffectTexture::Particle(texture.to_ascii_lowercase()) };
        self.shader = EffectShader::new(tex, blend).z(true, false);
        self.texture = texture.to_string();
        self.blend = blend;
    }

    /// elParticleSystem.cpp 0x002D0F30: use a shader built by the caller
    pub fn set_shader_code(&mut self, shader: EffectShader) {
        self.shader = shader;
    }

    /// elParticleSystem.cpp 0x002D0B70: +16
    pub fn set_gravity(&mut self, g: Vec3) {
        self.data.gravity = g;
    }

    /// elParticleSystem.cpp 0x002D0C10: +32 / +48, velocity mode 1
    pub fn set_velocity(&mut self, v: Vec3, spread: Vec3) {
        self.data.velocity = v;
        self.data.velocity_spread = spread;
        self.data.velocity_mode = 1;
    }

    /// elParticleSystem.cpp 0x002D0CA0: continuous emission at `rate` per second, spacing jitter clamped to 0..1
    pub fn set_emission_rate(&mut self, rate: ControlCurve, jitter: f32) {
        self.data.rate = rate;
        self.data.emission_mode = 0;
        self.data.rate_jitter = jitter;
        if self.data.rate_jitter < 0.0 {
            self.data.rate_jitter = 0.0;
        }
        if self.data.rate_jitter > 1.0 {
            self.data.rate_jitter = 1.0;
        }
    }

    /// elParticleSystem.cpp 0x002D0F70 `setRotationParams` (start angle lo/hi, spin lo/hi in degrees); only for MODE_FREELY_ROTATING
    pub fn set_rotation_params(&mut self, r: [f32; 4]) {
        if self.data.rotation_mode != MODE_FREELY_ROTATING {
            log::warn!("Warning: elParticleType::setRotationParams: This function should only be called if the 'mode' value passed into the constructor was MODE_FREELY_ROTATING.");
            return;
        }
        self.data.rotation_mode = MODE_FREELY_ROTATING;
        self.data.rotation = r;
    }

    /// elParticleSystem.cpp 0x002D0FE0: size curve, +272 = 1 − randomness
    pub fn set_size(&mut self, size: ControlCurve, randomness: f32) {
        self.data.size = size;
        self.data.size_random = 1.0 - randomness;
    }

    /// elParticleSystem.cpp 0x002D1020: trail params and the per-node steps (+92 time / (n − 1), +96 size_end^(1/(n − 1)), +100 colour /
    /// (n − 1); all 0 for one node)
    pub fn set_trail(&mut self, t: TrailParams) {
        self.data.trail = t;
        if self.nodes <= 1 {
            self.node_size_ratio = 0.0;
            self.node_time = 0.0;
            self.node_colour = Vec4::ZERO;
        } else {
            let n = (self.nodes - 1) as f32;
            self.node_time = t.time / n;
            self.node_size_ratio = t.size_end.powf(1.0 / n);
            self.node_colour = t.colour / n;
        }
    }

    /// elParticleSystem.cpp 0x002D1130: +392 anim, +116 frames = u · v
    pub fn set_anim(&mut self, a: AnimParams) {
        self.data.anim = a;
        self.frames = a.frames_v * a.frames_u;
    }

    /// elParticleSystem.cpp 0x002D1180: colour curve and the random colour (+356)
    pub fn set_colour(&mut self, c: ColourCurve, random: Vec4) {
        self.data.colour = c;
        self.data.colour_random = random;
    }

    /// CarDamageFX.cpp-range 0x000790F0: +12
    pub fn set_lifetime(&mut self, l: f32) {
        self.data.lifetime = l;
    }

    /// elParticleSystem.cpp 0x002D11D0 `addCollisionPlane`
    pub fn add_collision_plane(&mut self, p: CollisionPlane) {
        self.planes.push(p);
    }
}

/// one node (32 bytes from particle +48): +0 position, +12 dead, +16 velocity
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Node {
    pub pos: Vec3,
    pub dead: bool,
    pub vel: Vec3,
}

/// one particle (48-byte header + nodes)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Particle {
    /// +0 age
    pub age: f32,
    /// +8 nodes released, +10 nodes dead
    pub released: u16,
    pub dead: u16,
    /// +12 angle, +16 spin (radians)
    pub angle: f32,
    pub spin: f32,
    /// +20 frame, +24 frames per second
    pub frame: f32,
    pub frame_speed: f32,
    /// +28 random walk accumulator
    pub walk: f32,
    /// +32 random colour scale, +36 size scale, +40 size step per node
    pub colour_rand: f32,
    pub size_scale: f32,
    pub size_step: f32,
    pub nodes: Vec<Node>,
}

/// `elParticleSystem` (0x9C bytes)
#[derive(Clone, Debug)]
pub struct ElParticleSystem {
    /// +8 name (VsHashedString)
    pub name: String,
    /// +16 the type
    pub ty: usize,
    /// +20 transform the particles live in (a pointer the owner keeps current; `None` = world)
    pub transform: Option<Mat4>,
    /// +24 emitter shape
    pub shape: EmitterShape,
    /// +104 emitter velocity: spawn points move back by velocity · (time since the emission)
    pub velocity: Vec3,
    /// +120 age, +124 emission accumulator, +128 next emission
    pub age: f32,
    pub accum: f32,
    pub next: f32,
    /// +132 emitting, +133 burst fired
    pub emitting: bool,
    pub burst_done: bool,
    /// +140
    pub pool: ParticlePool<Particle>,
}

/// elParticleSystemManager 0x002C6840 `SacrificeParticle`, seen from the emitting system: free the first block of
/// the system holding the most blocks (`own` = the emitting system's pool, at its place in the manager's order)
pub trait Sacrifice {
    fn sacrifice(&mut self, own: &mut ParticlePool<Particle>, buf: &mut BlockBuffer);
}

/// no other systems (tests)
pub struct NoOthers;

impl Sacrifice for NoOthers {
    fn sacrifice(&mut self, own: &mut ParticlePool<Particle>, buf: &mut BlockBuffer) {
        if own.block_count() > 0 {
            own.free_block(buf);
        }
    }
}

/// what an emission / tick needs besides the system
pub struct SystemCtx<'a> {
    pub ty: &'a ElParticleType,
    pub rng: &'a mut VsRand,
    pub crt: &'a mut visos::handy::crt_rand::CrtRand,
    pub others: &'a mut dyn Sacrifice,
}

impl ElParticleSystem {
    /// elParticleSystem.cpp 0x002CECC0 `elParticleSystem(name, type)`: shape = the type's default, pool element = type +88
    pub fn new(name: &str, ty_index: usize, ty: &ElParticleType) -> Self {
        Self {
            name: name.to_string(),
            ty: ty_index,
            transform: None,
            shape: ty.data.shape,
            velocity: Vec3::ZERO,
            age: 0.0,
            accum: 0.5,
            next: 1.0,
            emitting: false,
            burst_done: false,
            pool: ParticlePool::new(ty.particle_bytes),
        }
    }

    /// elParticleSystem.cpp 0x002CF100 `start`: continuous types start emitting, burst types fire +236 particles at once
    pub fn start(&mut self, ctx: &mut SystemCtx, buf: &mut BlockBuffer) {
        self.age = 0.0;
        match ctx.ty.data.emission_mode {
            0 => self.emitting = true,
            1 => {
                for _ in 0..ctx.ty.data.burst {
                    self.emit(ctx, buf, 0.0, None, None);
                }
                self.burst_done = true;
            }
            _ => {}
        }
    }

    /// elParticleSystem.cpp 0x002CF170 `stop`
    pub fn stop(&mut self) {
        self.emitting = false;
    }

    /// elParticleSystem.cpp 0x002CF190: no particles left and nothing more to come (burst fired, or a non-looping rate curve past its end
    /// at 0)
    pub fn finished(&self, ty: &ElParticleType) -> bool {
        if !self.pool.is_empty() {
            return false;
        }
        match ty.data.emission_mode {
            0 => !ty.data.rate.looped && self.age > ty.data.rate.t[2] && ty.data.rate.v[3] == 0.0,
            1 => self.burst_done,
            _ => false,
        }
    }

    /// elParticleSystem.cpp 0x002CF310 — one particle `s` seconds old: makes room below the type's maximum by freeing the oldest, then
    /// a pool slot; when the shared buffer is out of blocks the manager sacrifices one (0x002C6840) and it tries
    /// once more
    pub fn emit(&mut self, ctx: &mut SystemCtx, buf: &mut BlockBuffer, s: f32, pos: Option<Vec3>, vel: Option<Vec3>) {
        let ty = ctx.ty;
        let d = &ty.data;
        while self.pool.len() as i32 >= d.max_particles {
            if self.pool.is_empty() {
                break;
            }
            self.pool.free(buf, 0);
        }
        let i = match self.pool.alloc(buf, Particle::default()) {
            Some(i) => i,
            None => {
                ctx.others.sacrifice(&mut self.pool, buf);
                match self.pool.alloc(buf, Particle::default()) {
                    Some(i) => i,
                    None => {
                        log::warn!("! \"SacrificeParticle() had no effect!\" (elParticleSystem.cpp 323)");
                        return;
                    }
                }
            }
        };
        let rng = &mut *ctx.rng;
        let mut p = Particle { nodes: vec![Node::default(); ty.nodes.max(1) as usize], ..Default::default() };
        p.frame = if d.anim.random_start != 0 { rng.range(0.0, (ty.frames as f32 - f32::from_bits(0x3A83_126F)) as f32) } else { 0.0 };
        p.frame_speed = ty.frames as f32 * rng.range(d.anim.rate_lo, d.anim.rate_hi);
        p.walk = 0.5;
        p.colour_rand = rng.range(0.0, 1.0);
        p.size_scale = rng.range(1.0, d.size_random);
        p.size_step = if ty.nodes <= 1 { 0.0 } else { (p.size_scale * d.trail.size_end - p.size_scale) / (ty.nodes - 1) as f32 };
        let mut at = match pos {
            Some(p) => p,
            None => self.shape.sample(rng) - self.velocity * s,
        };
        let mut v = match vel {
            Some(v) => v,
            None => {
                let mut v;
                if d.velocity_mode != 0 {
                    v = d.velocity;
                    let r = rng.in_sphere(1.0);
                    v += r * d.velocity_spread;
                } else {
                    let b = d.velocity.normalize_or_zero() * rng.range(-d.velocity_spread.x, d.velocity_spread.x);
                    v = d.velocity + b;
                }
                if d.flags & 1 != 0 {
                    v = self.shape.matrix.transform_vector3(v);
                }
                v
            }
        };
        p.released = 1;
        p.dead = 0;
        // VsCameraManager 0x000613D0: degrees · 0.017453292f
        let rad = f32::from_bits(0x3C8E_FA35);
        p.angle = rng.range(d.rotation[0], d.rotation[1]) * rad;
        p.spin = rng.range(d.rotation[2], d.rotation[3]) * rad;
        at += v * s;
        v += d.gravity * s;
        let n0 = Node { pos: at, dead: false, vel: v };
        for n in &mut p.nodes {
            *n = n0;
        }
        self.pool.items[i] = p;
    }

    /// elParticleSystem.cpp 0x002CEEB0 `tick(dt)`: every particle (0x002CFA50, the dead are freed), then while emitting the rate curve at
    /// the system's age decides how many particles are born this step, each aged by the part of the step since
    /// its birth; the spacing to the next one is 1 ± jitter/2
    pub fn tick(&mut self, ctx: &mut SystemCtx, buf: &mut BlockBuffer, dt: f32) {
        if dt < 0.0 {
            log::warn!("Warning: elParticleSystem::tick: timeStep < 0.");
        }
        if dt == 0.0 {
            return;
        }
        let mut i = 0;
        while i < self.pool.len() {
            if self.tick_particle(ctx, i, dt) {
                self.pool.free(buf, i);
            } else {
                i += 1;
            }
        }
        if self.emitting {
            self.age += dt;
            let rate = ctx.ty.data.rate.eval(self.age);
            let v = dt * rate;
            self.accum += v;
            let mut n = 0.0f32;
            while self.accum >= self.next {
                let s = -(self.next - self.accum) / v * dt;
                self.emit(ctx, buf, s, None, None);
                n += 1.0;
                let c = n + 1.0;
                let j = 0.5 * ctx.ty.data.rate_jitter;
                self.next = ctx.rng.range(c - j, c + j);
            }
            self.accum -= n;
            self.next -= n;
        }
    }

    /// elParticleSystem.cpp 0x002CFA50 — one particle; true = it died (the caller frees it)
    fn tick_particle(&mut self, ctx: &mut SystemCtx, i: usize, dt: f32) -> bool {
        let ty = ctx.ty;
        let d = &ty.data;
        let p = &mut self.pool.items[i];
        let nodes = ty.nodes as usize;
        // release the trail nodes whose time has come, each aged by its share of the particle's age
        if (p.released as usize) < nodes {
            let mut t = p.released as f32 * ty.node_time;
            while p.age > t {
                let k = p.released as usize;
                let dead = node_update(ty, &mut p.nodes[k], p.age - t, None, false);
                p.released += 1;
                if dead {
                    p.dead += 1;
                }
                if p.released as usize >= nodes {
                    break;
                }
                t += ty.node_time;
            }
        }
        p.angle += dt * p.spin;
        let frames = ty.frames;
        if d.anim.random_jump != 0 {
            let old = p.frame as i32;
            p.frame += dt * p.frame_speed;
            if p.frame as i32 != old {
                p.frame += (ctx.crt.rand() % frames) as f32;
            }
            while (frames as f32) < p.frame {
                p.frame -= frames as f32;
            }
        } else {
            p.frame += dt * p.frame_speed;
            while (frames as f32) < p.frame {
                p.frame -= frames as f32;
            }
            while p.frame < 0.0 {
                p.frame += frames as f32;
            }
        }
        p.age += dt;
        // the age of the newest node, by the full trail time per released node (original arithmetic)
        let newest = p.age - (p.released as i32 - 1) as f32 * d.trail.time;
        if d.lifetime > 0.0 && newest >= d.lifetime {
            return true;
        }
        p.walk += dt * d.walk_rate;
        let mut kick = Vec3::ZERO;
        while p.walk > 1.0 {
            kick += ctx.rng.in_sphere(d.walk_size);
            p.walk -= 1.0;
        }
        let released = p.released as usize;
        for j in 0..released {
            if node_update(ty, &mut p.nodes[j], dt, Some(kick), j == released - 1) {
                p.dead += 1;
            }
        }
        // 0x002D0180: nodes not yet released keep the kicks
        for n in p.nodes.iter_mut().take(nodes).skip(released) {
            if !n.dead {
                n.vel += kick;
            }
        }
        p.dead != 0
    }
}

/// elParticleSystem.cpp 0x002CFEC0 — move a node by its velocity, collide with the type's planes (only kind 0 unless `last`), then
/// gravity, drag and the random kick; true = killed by a kind-1 plane
fn node_update(ty: &ElParticleType, n: &mut Node, dt: f32, kick: Option<Vec3>, last: bool) -> bool {
    if n.dead {
        return false;
    }
    let prev = n.pos;
    n.pos += n.vel * dt;
    for pl in &ty.planes {
        if !last && pl.kind != 0 {
            continue;
        }
        let now = pl.distance(n.pos);
        if now >= 0.0 {
            continue;
        }
        if pl.distance(prev) < 0.0 {
            continue;
        }
        if pl.kind != 0 {
            // the report callback (type +32) has no user here
            if pl.kind == 1 {
                n.dead = true;
                return true;
            }
        } else {
            let normal = pl.plane.truncate();
            let dot = normal.dot(n.vel);
            n.vel += normal * ((-1.0 - pl.bounce.sqrt()) * dot);
        }
    }
    n.vel += ty.data.gravity * dt;
    n.vel *= 1.0 - dt * ty.data.drag;
    if let Some(k) = kick {
        n.vel += k;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use visos::handy::crt_rand::CrtRand;

    #[test]
    fn emit_and_age() {
        let mut ty = ElParticleType::new(150, 1, 0, "test");
        ty.set_emission_rate(ControlCurve::constant(20.0), 0.0);
        ty.set_velocity(Vec3::new(0.0, 0.0, -10.0), Vec3::ZERO);
        ty.set_lifetime(0.5);
        assert_eq!((ty.nodes, ty.particle_bytes, ty.frames), (1, 80, 1));
        let mut sys = ElParticleSystem::new("test", 0, &ty);
        let mut buf = BlockBuffer::new(0x32000);
        let (mut rng, mut crt) = (VsRand::default(), CrtRand::default());
        let mut ctx = SystemCtx { ty: &ty, rng: &mut rng, crt: &mut crt, others: &mut NoOthers };
        sys.start(&mut ctx, &mut buf);
        // one second at 60 fps: 20 born, those older than 0.5 s gone
        for _ in 0..60 {
            sys.tick(&mut ctx, &mut buf, 1.0 / 60.0);
        }
        assert!((9..=11).contains(&sys.pool.len()), "{}", sys.pool.len());
        for p in &sys.pool.items {
            assert!(p.age < 0.5 + 1e-4);
            // moved by the velocity for its age
            assert!((p.nodes[0].pos.z + 10.0 * p.age).abs() < 0.2);
        }
        sys.stop();
        for _ in 0..40 {
            sys.tick(&mut ctx, &mut buf, 1.0 / 60.0);
        }
        assert!(sys.pool.is_empty());
        assert_eq!(buf.used, 0);
    }

    #[test]
    fn bounce_and_kill_planes() {
        let mut ty = ElParticleType::new(10, 1, 0, "p");
        ty.add_collision_plane(CollisionPlane::new(Vec3::Y, Vec3::ZERO, 0.25, 0));
        let mut n = Node { pos: Vec3::new(0.0, 0.1, 0.0), dead: false, vel: Vec3::new(0.0, -1.0, 0.0) };
        assert!(!node_update(&ty, &mut n, 0.2, None, true));
        // v += n·(−1 − 0.5)·(−1)
        assert_eq!(n.vel.y, 0.5);
        ty.planes[0].kind = 1;
        let mut n = Node { pos: Vec3::new(0.0, 0.1, 0.0), dead: false, vel: Vec3::new(0.0, -1.0, 0.0) };
        assert!(!node_update(&ty, &mut n.clone(), 0.2, None, false));
        assert!(node_update(&ty, &mut n, 0.2, None, true));
    }
}
