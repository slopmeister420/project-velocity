//! elParticle\Xbox\elParticleRenderer_Xbox.cpp (0x00317890..0x003184A0): the quads. Each particle node becomes
//! two triangles of 24-byte vertices (pos, D3DCOLOR, uv) in a 128-quad list (`dword_6C242C`) drawn as
//! D3DPT_TRIANGLELIST with the type's shader (+68) whenever the shader or the transform changes (0x003181F0);
//! here the triangles go to `EffectDraw` per shader.

use bevy::math::{Mat4, Vec2, Vec3};

use super::el_particle_renderer::{CameraBasis, TrailState};
use crate::effect_draw::{EffectDraw, EffectShader, EffectVertex};

/// where quads go: the system's shader, its transform to world, the frame uv table
pub struct QuadSink<'a> {
    pub shader: &'a EffectShader,
    pub to_world: Mat4,
    pub uvs: &'a [[f32; 8]],
    pub draw: &'a mut EffectDraw,
}

/// elParticleRenderer_Xbox.cpp 0x00318060 — corners a b c d (a → (u0, v0), b → (u1, v0), c → (u1, v1), d → (u0, v1) of the frame), colour
/// `ca` on a / b, `cb` on c / d; triangles (a, b, c) and (a, c, d)
pub fn quad(sink: &mut QuadSink, a: Vec3, b: Vec3, c: Vec3, d: Vec3, ca: u32, cb: u32, frame: i16) {
    let t = sink.uvs.get(frame.max(0) as usize).unwrap_or(&sink.uvs[0]);
    let m = sink.to_world;
    let v = |p: Vec3, col: u32, u: f32, w: f32| EffectVertex::new(m.transform_point3(p), col, Vec2::new(u, w));
    let va = v(a, ca, t[6], t[7]);
    let vb = v(b, ca, t[4], t[5]);
    let vc = v(c, cb, t[2], t[3]);
    let vd = v(d, cb, t[0], t[1]);
    sink.draw.triangles(sink.shader, [va, vb, vc, va, vc, vd]);
}

/// elParticleRenderer_Xbox.cpp 0x00317C80 (vtable slot 0, MODE_FREELY_ROTATING) — camera-facing quad turned by `angle`: r = right·sin a +
/// up·cos a, b = −(fwd × r)·w, h = r·h; corners centre − b/2 + h/2, + b, − h
#[allow(clippy::too_many_arguments)]
pub fn quad_billboard(sink: &mut QuadSink, cam: &CameraBasis, pos: Vec3, colour: u32, w: f32, h: f32, angle: f32, frame: i16) {
    let r = cam.right * angle.sin() + cam.up * angle.cos();
    let b = -cam.fwd.cross(r) * w;
    let r = r * h;
    let a = pos - b * 0.5 + r * 0.5;
    let p1 = a + b;
    let p2 = p1 - r;
    let p3 = a - r;
    quad(sink, a, p1, p2, p3, colour, colour, frame);
}

/// elParticleRenderer_Xbox.cpp 0x00317E90 (slot 1, MODE_VELOCITY) — quad along the node's velocity: b = normalise(−(v̂ × fwd)), h =
/// normalise(b × fwd); b·(−w), h·h
#[allow(clippy::too_many_arguments)]
pub fn quad_velocity(sink: &mut QuadSink, cam: &CameraBasis, pos: Vec3, vel: Vec3, colour: u32, w: f32, h: f32, frame: i16) {
    let d = vel.normalize_or_zero();
    let b = (-d.cross(cam.fwd)).normalize_or_zero();
    let up = b.cross(cam.fwd).normalize_or_zero();
    let b = b * -w;
    let up = up * h;
    let a = pos - b * 0.5 + up * 0.5;
    let p1 = a + b;
    let p2 = p1 - up;
    let p3 = a - up;
    quad(sink, a, p1, p2, p3, colour, colour, frame);
}

/// elParticleRenderer_Xbox.cpp 0x00317AA0 (slot 2, MODE_TRAIL) — a ribbon piece from the previous node to this one, both ends as wide as their
/// sizes across the view (side = normalise((prev − pos with the look-direction part removed) × fwd)); the first
/// node of a particle only records itself
#[allow(clippy::too_many_arguments)]
pub fn quad_trail(sink: &mut QuadSink, cam: &CameraBasis, st: &mut TrailState, pos: Vec3, colour: u32, size: f32, frame: i16, first: bool) {
    if !first {
        let d = st.pos - pos;
        let along = d - cam.fwd * d.dot(cam.fwd);
        let side = along.cross(cam.fwd).normalize_or_zero();
        let here = side * size;
        let there = side * st.width;
        let a = st.pos - there * 0.5;
        let b = a + there;
        let c = pos + here * 0.5;
        let e = c - here;
        quad(sink, a, b, c, e, st.colour, colour, st.frame);
    }
    st.colour = colour;
    st.frame = frame;
    st.pos = pos;
    st.width = size;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect_draw::EffectTexture;

    #[test]
    fn billboard_faces_camera() {
        let sh = EffectShader::new(EffectTexture::White, 2);
        let mut draw = EffectDraw::default();
        let uvs = [[0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0]];
        let mut sink = QuadSink { shader: &sh, to_world: Mat4::from_translation(Vec3::new(0.0, 0.0, 5.0)), uvs: &uvs, draw: &mut draw };
        let cam = CameraBasis { right: Vec3::X, up: Vec3::Y, fwd: Vec3::NEG_Z };
        quad_billboard(&mut sink, &cam, Vec3::ZERO, 0xFFFF_FFFF, 2.0, 2.0, 0.0, 0);
        let v = &draw.batches[0].1;
        assert_eq!(v.len(), 6);
        // 2 x 2 around the transformed centre, in the camera plane
        for x in v {
            assert_eq!(x.pos.z, 5.0);
            assert!((x.pos.x.abs() - 1.0).abs() < 1e-6 && (x.pos.y.abs() - 1.0).abs() < 1e-6);
        }
        assert_eq!((v[0].uv, v[5].uv), (Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0)));
    }
}
