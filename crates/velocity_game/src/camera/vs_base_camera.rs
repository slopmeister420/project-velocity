//! src\Camera\VsBaseCamera.cpp — the camera every game camera derives from: a logic frame (position, right, up,
//! dir), a render copy taken once per update, field of view, aspect and the near/far planes. Frustum planes and
//! the debug drawing are the renderer's (visos_render builds its own from the transform).

use bevy::math::{Mat3, Mat4, Quat, Vec3};
use bevy::prelude::Transform;

/// `fNewFOV<3.0f` (radians)
pub const MAX_FOV: f32 = 3.0;

#[derive(Clone, Debug, PartialEq)]
pub struct VsBaseCamera {
    /// +0x08 world matrix built from the render frame (0x0005D2E0)
    pub matrix: Mat4,
    pub pos_render: Vec3,   // +0x48
    pub right_render: Vec3, // +0x58
    pub up_render: Vec3,    // +0x68
    pub dir_render: Vec3,   // +0x78
    pub fov_render: f32,    // +0x88
    /// +0x8C vertical field of view (radians)
    pub fov: f32,
    pub aspect: f32,        // +0x90
    pub pos: Vec3,          // +0xD4
    /// +0xE4 / +0xE8 near-plane half extents (0x0005D1D0)
    pub half_width: f32,
    pub half_height: f32,
    pub near: f32,          // +0xEC
    pub far: f32,           // +0xF0
    pub right: Vec3,        // +0xF8
    pub up: Vec3,           // +0x108
    pub dir: Vec3,          // +0x118
}

impl Default for VsBaseCamera {
    /// VsRender.cpp 0x0005CD10 (constructor): identity frame looking down +z
    fn default() -> Self {
        let mut c = VsBaseCamera {
            matrix: Mat4::IDENTITY,
            pos_render: Vec3::ZERO,
            right_render: Vec3::X,
            up_render: Vec3::Y,
            dir_render: Vec3::Z,
            fov_render: 1.0,
            fov: 1.0,
            aspect: 4.0 / 3.0,
            pos: Vec3::ZERO,
            half_width: 0.0,
            half_height: 0.0,
            near: 0.5,
            far: 600.0,
            right: Vec3::X,
            up: Vec3::Y,
            dir: Vec3::Z,
        };
        c.update_extents();
        c
    }
}

impl VsBaseCamera {
    /// VsBaseCamera.cpp 0x0005D0B0
    pub fn set_fov(&mut self, fov: f32) {
        if fov >= MAX_FOV {
            log::error!("assert: fNewFOV<3.0f");
        }
        self.fov = fov;
        self.update_extents();
    }

    /// VsBaseCamera.cpp 0x0005D160 — only on a change
    pub fn set_aspect(&mut self, aspect: f32) {
        if aspect != self.aspect {
            self.aspect = aspect;
            self.update_extents();
        }
    }

    /// VsRender.cpp 0x0005C850
    pub fn set_near(&mut self, near: f32) {
        self.near = near;
    }

    /// VsCameraManager.cpp 0x000602F0
    pub fn set_far(&mut self, far: f32) {
        self.far = far;
    }

    /// VsBaseCamera.cpp 0x0005D1D0 — `tan(fov / 2) * near` and that times the aspect
    pub fn update_extents(&mut self) {
        let h = (self.fov * 0.5).tan() * self.near;
        self.half_height = h;
        self.half_width = h * self.aspect;
    }

    /// VsBaseCamera.cpp 0x0005D250 — take the logic frame for rendering, then build the matrix
    pub fn copy_to_render(&mut self) {
        self.pos_render = self.pos;
        self.right_render = self.right;
        self.up_render = self.up;
        self.dir_render = self.dir;
        self.fov_render = self.fov;
        self.build_matrix();
    }

    /// VsBaseCamera.cpp 0x0005D2E0 — rows -right, up, dir, position (the original then sets the projection,
    /// the two frustum plane sets and the renderer's camera from it)
    pub fn build_matrix(&mut self) {
        self.matrix = Mat4::from_cols_array(&[
            -self.right_render.x, -self.right_render.y, -self.right_render.z, 0.0,
            self.up_render.x, self.up_render.y, self.up_render.z, 0.0,
            self.dir_render.x, self.dir_render.y, self.dir_render.z, 0.0,
            self.pos_render.x, self.pos_render.y, self.pos_render.z, 0.0,
        ]);
    }

    /// VsBaseCamera.cpp 0x0005D540 — the render frame looking at `target` (unnormalised cross products)
    pub fn look_at_render(&mut self, pos: Vec3, target: Vec3, up: Vec3) {
        self.pos_render = pos;
        if target != pos {
            self.dir_render = target - pos;
        }
        self.dir_render = normalise(self.dir_render);
        self.right_render = up.cross(self.dir_render);
        self.up_render = self.dir_render.cross(self.right_render);
    }

    /// VsBaseCamera.cpp 0x0005D770
    pub fn set_frame(&mut self, pos: Vec3, right: Vec3, up: Vec3, dir: Vec3) {
        self.pos = pos;
        self.right = right;
        self.up = up;
        self.dir = dir;
        self.check_frame();
    }

    /// VsBaseCamera.cpp 0x0005D930 — rows 0..2 of `m` (row-vector convention) are right, up, dir
    pub fn set_frame_matrix(&mut self, pos: Vec3, m: &Mat4) {
        self.pos = pos;
        self.right = m.x_axis.truncate();
        self.up = m.y_axis.truncate();
        self.dir = m.z_axis.truncate();
        self.check_frame();
    }

    /// VsBaseCamera.cpp 0x0005DB10 — look at `target` from `pos` (dir kept when they coincide)
    pub fn look_at(&mut self, pos: Vec3, target: Vec3, up: Vec3) {
        self.pos = pos;
        if target != pos {
            self.dir = target - pos;
        }
        self.dir = normalise(self.dir);
        self.right = up.cross(self.dir);
        self.up = self.dir.cross(self.right);
        self.check_frame();
    }

    /// VsBaseCamera.cpp 0x0005DD30 — rotate the frame by `m`, then re-orthogonalise from dir and up
    pub fn rotate(&mut self, m: &Mat3) {
        self.right = *m * self.right;
        self.up = *m * self.up;
        self.dir = *m * self.dir;
        self.dir = normalise(self.dir);
        self.up = normalise(self.up);
        self.right = self.up.cross(self.dir);
        self.up = self.dir.cross(self.right);
        self.check_frame();
    }

    fn check_frame(&self) {
        if self.right.length() <= 0.0 || self.up.length() <= 0.0 || self.dir.length() <= 0.0 {
            log::error!("assert: VsBaseCamera frame has a zero axis");
        }
    }

    /// the render frame as a Bevy transform (game coordinates are Bevy's; a Bevy camera looks down -z, so the
    /// local z axis is `-dir`)
    pub fn render_transform(&self) -> Transform {
        let dir = normalise(self.dir_render);
        let up = normalise(self.up_render);
        let right = normalise(up.cross(-dir));
        let up = (-dir).cross(right);
        Transform { translation: self.pos_render, rotation: Quat::from_mat3(&Mat3::from_cols(right, up, -dir)), ..Default::default() }
    }
}

/// AC::Renderer 0x002AC2C0 — the engine's quaternion matrix, applied to row vectors: a rotation by the
/// conjugate (see ## AI in CLAUDE.md)
pub fn quat_matrix(q: Quat) -> Mat3 {
    Mat3::from_quat(q.conjugate())
}

/// Renderer 0x002AB7E0 — RotX(a) * RotY(b) * RotZ(c) in D3D order, as a glam matrix applied to column vectors
pub fn yaw_pitch_roll(a: f32, b: f32, c: f32) -> Mat3 {
    Mat3::from_rotation_z(c) * Mat3::from_rotation_y(b) * Mat3::from_rotation_x(a)
}

/// VsVector4::Normalise — zero when the length is at most 1e-6
pub fn normalise(v: Vec3) -> Vec3 {
    let l = v.length();
    if l <= 1e-6 { Vec3::ZERO } else { v / l }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_at_and_transform() {
        let mut c = VsBaseCamera::default();
        c.look_at(Vec3::new(0.0, 0.0, 30.0), Vec3::ZERO, Vec3::Y);
        assert!((c.dir - Vec3::NEG_Z).length() < 1e-6);
        c.copy_to_render();
        let t = c.render_transform();
        assert!((t.forward().as_vec3() - Vec3::NEG_Z).length() < 1e-5);
        c.set_fov(std::f32::consts::FRAC_PI_4);
        assert!((c.half_height - (std::f32::consts::FRAC_PI_8).tan() * 0.5).abs() < 1e-6);
    }
}
