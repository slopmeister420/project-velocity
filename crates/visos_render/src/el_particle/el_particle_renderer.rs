//! elParticleRenderer.cpp (0x00343290..0x00344600): the platform-independent half of the renderer — the frame uv
//! table (0x00343840), the per-system transform (0x00343B40) and the walk over every particle's nodes (0x00343360)
//! that hands quads to the Xbox half (`el_particle_renderer_xbox`, the renderer's vtable slots 0/1/2).

use bevy::math::{Mat4, Vec3, Vec4};

use super::el_particle_renderer_xbox::{quad_billboard, quad_trail, quad_velocity, QuadSink};
use super::el_particle_system::{ElParticleSystem, ElParticleType, MODE_FREELY_ROTATING, MODE_TRAIL, MODE_VELOCITY};
use crate::effect_draw::EffectDraw;

/// `(NumFramesU * NumFramesV) <= MAX_FRAMES`
pub const MAX_FRAMES: i32 = 16;

/// rows 0 / 1 / 2 of the inverse view (`byte_7B5050`, 0x00343B40): camera right, up and look direction
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraBasis {
    pub right: Vec3,
    pub up: Vec3,
    pub fwd: Vec3,
}

/// the trail ribbon's statics (`dword_7B44C8` previous point, `dword_7B44A4` width, `dword_7B44D8` colour,
/// `word_7B44A0` frame): they carry over between particles, systems and frames
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TrailState {
    pub pos: Vec3,
    pub width: f32,
    pub colour: u32,
    pub frame: i16,
}

/// elParticleRenderer.cpp 0x00343840 — uv corners per frame, row by row (8 floats: [0] u0 [1] v1 [2] u1 [3] v1 [4] u1 [5] v0 [6] u0 [7]
/// v0; `flip` swaps v0 / v1)
pub fn frame_uvs(u: i32, v: i32, flip: bool) -> Vec<[f32; 8]> {
    if u * v > MAX_FRAMES {
        log::warn!("(NumFramesU * NumFramesV) <= MAX_FRAMES (elParticleRenderer.cpp 184)");
    }
    let du = (1.0 / u.max(1) as f64) as f32;
    let dv = (1.0 / v.max(1) as f64) as f32;
    let mut out = Vec::new();
    for i in 0..v {
        for j in 0..u {
            let u0 = (j as f64 * du as f64) as f32;
            let v0 = (i as f64 * dv as f64) as f32;
            let u1 = ((j + 1) as f64 * du as f64) as f32;
            let v1 = ((i + 1) as f64 * dv as f64) as f32;
            out.push(if flip { [u0, v0, u1, v0, u1, v1, u0, v1] } else { [u0, v1, u1, v1, u1, v0, u0, v0] });
        }
    }
    if out.is_empty() {
        out.push([0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0]);
    }
    out
}

/// xbOnlineVoiceInput.cpp-range 0x00352010: 0..1 to a byte (`_ftol(x · 255.99f)`)
pub fn colour_byte(x: f32) -> u32 {
    if x <= 0.0 {
        0
    } else if x < 1.0 {
        (x as f64 * 255.99f32 as f64) as i32 as u32 & 0xFF
    } else {
        255
    }
}

/// 0x00351FB0: colour (x r, y g, z b, w a) to D3DCOLOR
pub fn colour_argb(c: Vec4) -> u32 {
    colour_byte(c.w) << 24 | colour_byte(c.x) << 16 | colour_byte(c.y) << 8 | colour_byte(c.z)
}

/// elParticleRenderer.cpp 0x00343360 — every particle, every released node (one more, the next to come, in trail mode) from the head:
/// node k is drawn as the particle was k node-times ago (age, angle, frame stepped back; colour and size stepped
/// along the trail); colour = colour curve + random colour · the particle's colour scale, size = size curve ·
/// size scale. Particles live in the system's transform (0x00343B40: the camera basis is taken into it).
pub fn draw_system(sys: &ElParticleSystem, ty: &ElParticleType, cam: &CameraBasis, trail: &mut TrailState, draw: &mut EffectDraw) {
    if sys.pool.is_empty() {
        return;
    }
    let to_world = sys.transform.unwrap_or(Mat4::IDENTITY);
    let local_cam = match sys.transform {
        Some(t) => {
            let inv = t.inverse();
            CameraBasis { right: inv.transform_vector3(cam.right), up: inv.transform_vector3(cam.up), fwd: inv.transform_vector3(cam.fwd) }
        }
        None => *cam,
    };
    let d = &ty.data;
    let mode = d.rotation_mode;
    let nodes = ty.nodes as f32;
    let seg = ty.node_time;
    let uvs = frame_uvs(d.anim.frames_u, d.anim.frames_v, false);
    let frames = (d.anim.frames_u * d.anim.frames_v).max(1);
    let mut sink = QuadSink { shader: &ty.shader, to_world, uvs: &uvs, draw };
    for p in &sys.pool.items {
        let mut rand_col = d.colour_random * p.colour_rand;
        let mut count = p.released as usize;
        if (count as f32) < nodes && mode == MODE_TRAIL {
            count += 1;
        }
        let mut first = true;
        let mut angle = p.angle;
        let dangle = -p.spin * seg;
        let mut frame = p.frame;
        let dframe = -p.frame_speed * seg;
        let mut age = p.age;
        let mut scale = p.size_scale;
        for k in 0..count.min(p.nodes.len()) {
            let n = &p.nodes[k];
            if !n.dead {
                let fi = ((frame as i32) % frames) as i16;
                let col = colour_argb(d.colour.eval(age) + rand_col);
                let size = d.size.eval(age) * scale;
                match mode {
                    MODE_FREELY_ROTATING => quad_billboard(&mut sink, &local_cam, n.pos, col, size, size, angle, fi),
                    MODE_VELOCITY => quad_velocity(&mut sink, &local_cam, n.pos, n.vel, col, size, size, fi),
                    MODE_TRAIL => quad_trail(&mut sink, &local_cam, trail, n.pos, col, size, fi, first),
                    _ => {}
                }
                first = false;
            }
            scale += p.size_step;
            rand_col += ty.node_colour;
            angle += dangle;
            frame += dframe;
            age -= seg;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uv_table_and_colour() {
        let t = frame_uvs(2, 2, false);
        assert_eq!(t.len(), 4);
        // frame 1: u 0.5..1, v 0..0.5
        assert_eq!(t[1], [0.5, 0.5, 1.0, 0.5, 1.0, 0.0, 0.5, 0.0]);
        assert_eq!(colour_argb(Vec4::new(1.0, 0.5, 0.0, 0.25)), 0x3FFF_7F00);
    }
}
