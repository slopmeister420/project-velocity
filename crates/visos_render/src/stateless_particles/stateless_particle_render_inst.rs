//! StatelessParticles\StatelessParticleRenderInst.cpp + XBox\StatelessParticleRenderInst_XBOX.cpp — the per-draw
//! parameter block (`unk_7A0570`) and the CPU evaluation that turns an emitter into camera-facing quads. Each
//! particle is regenerated from the emitter seed every frame, so the same seed always gives the same particles.

use bevy::math::{Mat4, Vec2, Vec3, Vec4};

use super::stateless_particle_emitter::CEmitter;
use super::stateless_particle_type::{round_up, CType};

/// StatelessParticleRenderInst.cpp 0x00322850: one step of the block's generator (0x00322880), as 0..1
fn rand(seed: &mut u32) -> f64 {
    *seed = ((20077u32.wrapping_mul(*seed & 0xFFFF)).wrapping_add(12345)) & 0x7FFF;
    *seed as f64 / 32767.0
}

/// StatelessParticleRenderInst.cpp 0x003228B0: `a + rand * range` (stored as float)
fn rand_range(seed: &mut u32, a: f32, range: f32) -> f32 {
    (rand(seed) * range as f64 + a as f64) as f32
}

/// StatelessParticleRenderInst.cpp 0x003226D0: seconds to 1/600 s ticks
fn ticks(t: f32) -> i16 {
    (t * 600.0) as i32 as i16
}

/// 0x000452C0
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (b - a) * t + a
}

/// `unk_7A0570`, filled per emitter copy by StatelessParticleMgr.cpp 0x002D3C80
#[derive(Clone, Debug, Default)]
pub struct CRenderInst {
    /// +4 generator state (the emitter seed)
    pub seed: u32,
    /// +8
    pub num_particles: u32,
    /// +16 particle life, +20 it in ticks
    pub life: f32,
    pub life_ticks: i16,
    /// +22 emitter time folded into the current life (ticks), +24 emission window, +26 / +28 the birth offsets
    /// still alive at the start / end of the run
    pub phase: i16,
    pub window: i16,
    pub alive_from: i16,
    pub alive_until: i16,
    /// +32 / +36 birth size and range, +40 size at death, +44 size scale, +48 alpha scale
    pub size_min: f32,
    pub size_range: f32,
    pub size_end: f32,
    pub size_scale: f32,
    pub alpha_scale: f32,
    /// +52 spawn box origin (zero), +68 box size, +84 velocity, +100 velocity range, +116 acceleration
    pub box_base: Vec4,
    pub box_size: Vec4,
    pub vel_min: Vec4,
    pub vel_range: Vec4,
    pub accel: Vec4,
    /// +132 birth colour, +136 its range (bytewise, wrapping), +140 colour at death; bytes b, g, r, a
    pub colour: [u8; 4],
    pub colour_range: [u8; 4],
    pub colour_end: [u8; 4],
    /// +144 / +148 spin speed and range
    pub rot_min: f32,
    pub rot_range: f32,
    /// +152 frames, +156 anim rate, +160 frames over life, +161 random start angle, +162 axis billboard
    pub num_frames: u32,
    pub anim_rate: u32,
    pub anim_over_life: bool,
    pub random_angle: bool,
    pub axis_billboard: bool,
    /// +164 emitter matrix
    pub matrix: Mat4,
}

impl CRenderInst {
    /// StatelessParticleMgr.cpp 0x002D3C80 — set up one copy of an emitter at time `t` with size and alpha scales
    pub fn setup(ty: &CType, em: &CEmitter, t: f32, size: f32, alpha: f32) -> Self {
        let life = ty.life;
        let window = ty.emit_time;
        let end = round_up(ty.duration(), life);
        let mut last = end - life + window;
        let stop = em.stop_time;
        if stop > 0.0 && stop < last {
            last = stop;
        }
        let mut phase = t - window;
        let mut last_phase = last - window;
        if phase > 0.0 {
            phase %= life;
            last_phase %= life;
        }
        let (mut from, mut until) = (0.0, 0.0);
        if last + life > t {
            if t < last || phase >= last_phase {
                if t >= last {
                    from = 0.0;
                    until = life - last_phase;
                }
            } else {
                from = life - last_phase;
                until = life;
            }
        } else {
            from = life;
            until = life;
        }
        let c = CType::colour_bytes(ty.colour_min);
        let cmax = CType::colour_bytes(ty.colour_max);
        // 0x002D3FF0: bytewise difference
        let range = [cmax[0].wrapping_sub(c[0]), cmax[1].wrapping_sub(c[1]), cmax[2].wrapping_sub(c[2]), cmax[3].wrapping_sub(c[3])];
        Self {
            seed: em.seed,
            num_particles: ty.num_particles,
            life,
            life_ticks: ticks(life),
            phase: ticks(phase),
            window: ticks(window),
            alive_from: ticks(from),
            alive_until: ticks(until),
            size_min: ty.size_min,
            size_range: ty.size_max - ty.size_min,
            size_end: ty.size_end,
            size_scale: em.scale * size,
            alpha_scale: alpha,
            box_base: Vec4::ZERO,
            box_size: ty.box_size,
            vel_min: ty.vel_min,
            vel_range: ty.vel_max - ty.vel_min,
            accel: ty.accel,
            colour: c,
            colour_range: range,
            colour_end: CType::colour_bytes(ty.colour_end),
            rot_min: ty.rot_min,
            rot_range: ty.rot_max - ty.rot_min,
            num_frames: ty.num_frames,
            anim_rate: ty.anim_rate,
            anim_over_life: ty.anim_mode == 1,
            random_angle: ty.random_start_angle(),
            axis_billboard: ty.axis_billboard(),
            matrix: em.matrix,
        }
    }

    /// StatelessParticleRenderInst.cpp 0x00321FC0 — every live particle as (position, colour ARGB, size, angle, frame)
    pub fn particles(&self) -> Vec<(Vec3, u32, f32, f32, u16)> {
        let mut out = Vec::new();
        let mut seed = self.seed;
        let rand_vec = |seed: &mut u32, a: Vec4, r: Vec4| -> Vec3 {
            // 0x003228D0
            let x = (rand(seed) * r.x as f64 + a.x as f64) as f32;
            let y = (rand(seed) * r.y as f64 + a.y as f64) as f32;
            let z = (rand(seed) * r.z as f64 + a.z as f64) as f32;
            Vec3::new(x, y, z)
        };
        for _ in 0..self.num_particles {
            // 0x00322810
            let birth = (rand(&mut seed) * self.window as f64) as i32 as i16;
            let mut frame = (rand(&mut seed) * self.num_frames as u16 as i16 as f64) as i32 as i16;
            let start_angle = if self.random_angle { rand_range(&mut seed, 0.0, std::f32::consts::TAU) } else { 0.0 };
            let spin = rand_range(&mut seed, self.rot_min, self.rot_range);
            let mut size = rand_range(&mut seed, self.size_min, self.size_range);
            // 0x00322980: r, g, b, a order
            let mut col = [0u8; 4];
            for k in [2usize, 1, 0, 3] {
                col[k] = ((rand(&mut seed) * self.colour_range[k] as f64) as i32 as u8).wrapping_add(self.colour[k]);
            }
            let pos0 = rand_vec(&mut seed, self.box_base, self.box_size);
            let vel = rand_vec(&mut seed, self.vel_min, self.vel_range);
            let age = birth.wrapping_add(self.phase);
            if !(age >= 0
                && self.alive_from <= birth
                && (self.alive_until <= birth || age <= self.life_ticks)
                && self.alive_until as i32 <= birth as i32 + self.life_ticks as i32)
            {
                continue;
            }
            let cycles = age as f64 / self.life_ticks as f64;
            let t = (cycles - cycles as i32 as f64) as f32;
            if self.anim_over_life {
                frame = ((self.anim_rate.wrapping_mul(self.num_frames)) as f64 * t as f64) as i32 as i16;
            }
            frame &= (self.num_frames as i16).wrapping_sub(1);
            size = lerp(size, self.size_end, t);
            let r = lerp(col[2] as f32, self.colour_end[2] as f32, t) as i32 as u8;
            let g = lerp(col[1] as f32, self.colour_end[1] as f32, t) as i32 as u8;
            let b = lerp(col[0] as f32, self.colour_end[0] as f32, t) as i32 as u8;
            let a = lerp(col[3] as f32, self.colour_end[3] as f32, t) as i32 as u8;
            size *= self.size_scale;
            let a = ((a as f32 * self.alpha_scale) as i32) as u8;
            // VISOS_ColourXBOX 0x002B11C0: each component doubled and clamped again
            let d = |v: u8| (2 * v as u32).min(255);
            let argb = (d(a) << 24) | (d(r) << 16) | (d(g) << 8) | d(b);
            let s = t * self.life;
            let p = pos0 + vel * s + self.accel.truncate() * (s * s * 0.5) - self.box_size.truncate() * 0.5;
            let p = self.matrix.transform_point3(p);
            let angle = (spin * s + start_angle) % std::f32::consts::TAU;
            out.push((p, argb, size, angle, frame as u16));
        }
        out
    }
}

/// StatelessParticleRenderInst.cpp 0x003224C0: uv rectangles of the animation frames (`dword_7B44E0`, 8 floats each), not flipped
pub fn frame_table(frames_u: u32, frames_v: u32) -> Vec<[f32; 8]> {
    let su = 1.0 / frames_u as f64;
    let sv = 1.0 / frames_v as f64;
    let mut out = Vec::new();
    for i in 0..frames_v {
        for j in 0..frames_u {
            let u0 = (j as f64 * su) as f32;
            let v0 = (i as f64 * sv) as f32;
            let u1 = ((j + 1) as f64 * su) as f32;
            let v1 = ((i + 1) as f64 * sv) as f32;
            out.push([u1, v1, u0, v1, u0, v0, u1, v0]);
        }
    }
    out
}

/// StatelessParticleRenderInst_XBOX.cpp 0x00321930 — one quad as two triangles (6 vertices: position, ARGB, uv).
/// `right` / `up` are the billboard axes (rows 0 and 1 of the camera or axis matrix).
pub fn push_quad(out: &mut Vec<(Vec3, u32, Vec2)>, p: Vec3, argb: u32, size: f32, angle: f32, uv: &[f32; 8], right: Vec3, up: Vec3) {
    let ang = 45.0f32 * 0.017453292 + angle;
    let ax = ang.sin() * size * 0.7071;
    let ay = ang.cos() * size * 0.7071;
    let a = right * ax + up * ay;
    let b = right * ay + up * (-ax);
    let (v0, v1, v2, v3) = (p + b, p + a, p - b, p - a);
    let uvp = |k: usize| Vec2::new(uv[2 * k], uv[2 * k + 1]);
    out.extend([(v0, argb, uvp(1)), (v1, argb, uvp(2)), (v2, argb, uvp(3)), (v0, argb, uvp(1)), (v2, argb, uvp(3)), (v3, argb, uvp(0))]);
}

/// StatelessParticleRenderInst.cpp 0x00322710: billboard axes turning only around `axis` (the emitter's y) to face the camera
pub fn axis_billboard(axis: Vec3, cam_forward: Vec3) -> (Vec3, Vec3) {
    let s = axis.dot(cam_forward);
    let v = (cam_forward - axis * s).normalize_or_zero();
    (axis.cross(v), axis)
}
