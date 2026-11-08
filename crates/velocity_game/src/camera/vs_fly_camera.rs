//! src\Camera\VsFlyCamera.cpp — two cameras:
//! - `VsFlyCamera` (524 bytes, one per player, `GameManager+0x22C`): a look-at camera either driven by hand
//!   (mode 0: the crash camera writes its position and target) or by a timeline camera (mode 1: the flag girl
//!   intro). Each update it hands its view to the player's active camera.
//! - `VsRoamingCamera` (508 bytes, `GameManager+0x230`): the debug free camera (mode 4), flown with the
//!   DebugFreeCam actions while the world is frozen.

use bevy::math::{Mat3, Mat4, Vec3};

use super::vs_base_camera::{normalise, yaw_pitch_roll, VsBaseCamera};
use crate::controller::user_input::EAction;
use crate::game_bridge::camera_bridge::CameraBridgeData;

/// `dword_685118` near plane of timeline cameras
pub const TIMELINE_NEAR: f32 = 0.25;

/// what a timeline camera object puts out (TimelineCamera +0x754: +0x40 position, +0x50 target, +0x60 up,
/// +0x70 fov in degrees, +0x758 cut). Heights are relative to the ground under the point.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TimelineCameraOutput {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_deg: f32,
    pub cut: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VsFlyCamera {
    pub base: VsBaseCamera,
    pub target: Vec3, // +0x1F0
    pub player: u32,  // +0x200
    /// +0x208 0 = by hand, 1 = timeline
    pub mode: u32,
    /// set when the render side must not interpolate (a camera cut)
    pub cut: bool,
}

impl Default for VsFlyCamera {
    /// VsCrashCameraManager.cpp 0x00063FA0 (the constructor sits in that object's range)
    fn default() -> Self {
        VsFlyCamera { base: VsBaseCamera::default(), target: Vec3::ZERO, player: 0, mode: 0, cut: false }
    }
}

impl VsFlyCamera {
    /// VsFlyCamera.cpp 0x00064090 — take the timeline camera's view, lifting every point onto the ground found
    /// under it (`ground_y` casts from 5 m above to 5 m below the grid height)
    pub fn timeline(&mut self, tc: &mut TimelineCameraOutput, ground_y: &dyn Fn(Vec3) -> Option<f32>) {
        let g = ground_y(tc.position).unwrap_or_else(|| {
            log::error!("Error finding ground under camera");
            0.0
        });
        let mut pos = tc.position;
        pos.y += g;
        let mut target = tc.target;
        target.y += g;
        let mut up = tc.up;
        up.y += g;
        let up = normalise(up - pos);
        self.target = target;
        self.base.look_at(pos, target, up);
        self.base.up = normalise(self.base.up);
        self.base.right = normalise(self.base.right);
        self.base.set_near(TIMELINE_NEAR);
        self.base.set_fov(tc.fov_deg * f32::from_bits(0x3C8E_FA35));
        if tc.cut {
            self.cut = true;
            tc.cut = false;
        }
        let sum = |v: Vec3| v.x.abs() + v.y.abs() + v.z.abs();
        if sum(self.base.up) <= 0.1 {
            log::warn!("VsFlyCamera::Timeline - Up vector looks incorrect.  Check cuts in timeline");
        }
        if sum(self.base.dir) <= 0.1 {
            log::warn!("VsFlyCamera::Timeline - At vector looks incorrect.  Check cuts in timeline");
        }
        if sum(self.base.right) <= 0.1 {
            log::warn!("VsFlyCamera::Timeline - Right vector looks incorrect.  Check cuts in timeline");
        }
        if sum(self.base.pos) <= 0.1 {
            log::warn!("VsFlyCamera::Timeline - Position vector looks incorrect.  Check cuts in timeline");
        }
    }

    /// VsFlyCamera.cpp 0x00064790 — per update; returns the view the player's active camera takes
    /// (`CameraBridge` record and `look_at` on the active camera)
    pub fn update(&mut self, tc: Option<&mut TimelineCameraOutput>, ground_y: &dyn Fn(Vec3) -> Option<f32>) -> CameraBridgeData {
        let hand = match self.mode {
            0 => true,
            1 => {
                if let Some(tc) = tc {
                    self.timeline(tc, ground_y);
                }
                false
            }
            _ => {
                log::warn!("Unknown fly camera mode");
                true
            }
        };
        if hand {
            let dir = normalise(self.target - self.base.pos);
            self.base.dir = dir;
            let right = normalise(Vec3::Y.cross(dir));
            let up = normalise(dir.cross(right));
            CameraBridgeData { position: self.base.pos, target: self.target, up, fov: self.base.fov }
        } else {
            CameraBridgeData { position: self.base.pos, target: self.target, up: Vec3::Y, fov: self.base.fov }
        }
    }
}

/// `VsRoamingCamera` (vtable 0x00629B70)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VsRoamingCamera {
    pub base: VsBaseCamera,
    /// +0x1F0 / +0x1F4 / +0x1F8 rotation angles about x, y, z
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    /// +0x94 orientation matrix
    pub orientation: Mat4,
}

const TWO_PI: f32 = 6.283185;

fn wrap(a: f32) -> f32 {
    let mut a = a;
    if a > TWO_PI {
        a -= TWO_PI;
    }
    if a < 0.0 {
        a += TWO_PI;
    }
    a
}

impl VsRoamingCamera {
    /// VsFlyCamera.cpp 0x00064E50 — fly with the debug actions; returns the view and whether the world stays
    /// frozen (`GameManager+0x22A`: frozen unless DebugFreeCamPlay is held)
    pub fn update(&mut self, value: &dyn Fn(EAction) -> f32) -> (CameraBridgeData, bool) {
        self.rot_x = wrap(self.rot_x - value(EAction::DebugFreeCamVert) * 0.04);
        self.rot_y = wrap(self.rot_y - value(EAction::DebugFreeCamHoriz) * 0.04);
        let m = yaw_pitch_roll(self.rot_x, self.rot_y, self.rot_z);
        self.orientation = Mat4::from_mat3(m);
        let pan = value(EAction::DebugFreeCamPanLeft) - value(EAction::DebugFreeCamPanRight);
        let mv = Vec3::new(0.25 * pan, 0.0, value(EAction::DebugFreeCamForwardBack) * 1.0);
        let mv = m * mv;
        let mut pos = self.base.pos + mv;
        pos.y += 0.25 * (value(EAction::DebugFreeCamMoveUp) - value(EAction::DebugFreeCamMoveDown));
        self.base.set_frame_matrix(pos, &self.orientation);
        let bridge = CameraBridgeData {
            position: self.base.pos,
            target: self.base.pos + self.base.dir,
            up: self.base.up,
            fov: self.base.fov,
        };
        let frozen = value(EAction::DebugFreeCamPlay) == 0.0;
        (bridge, frozen)
    }

    /// VsFlyCamera.cpp 0x00064BC0 (switching on) — start where the player's camera is, facing its heading
    pub fn take_over(&mut self, pos: Vec3, dir: Vec3) {
        let flat = normalise(Vec3::new(dir.x, 0.0, dir.z));
        let mut a = Vec3::Z.dot(flat).clamp(-1.0, 1.0).acos();
        if flat.x < 0.0 {
            a = TWO_PI - a;
        }
        self.rot_x = 0.0;
        self.rot_y = a;
        self.rot_z = 0.0;
        self.base.pos = pos;
    }
}

/// the frame `yaw_pitch_roll` rotates +z to (for tests and tools)
pub fn heading_of(m: &Mat3) -> Vec3 {
    *m * Vec3::Z
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fly_camera_by_hand_and_timeline() {
        let mut f = VsFlyCamera::default();
        f.base.pos = Vec3::new(0.0, 2.0, -10.0);
        f.target = Vec3::ZERO;
        let b = f.update(None, &|_| Some(0.0));
        assert!((b.up.y - 1.0).abs() < 0.05);
        f.mode = 1;
        let mut tc = TimelineCameraOutput { position: Vec3::new(5.0, 1.0, 0.0), target: Vec3::ZERO, up: Vec3::new(5.0, 2.0, 0.0), fov_deg: 50.0, cut: true };
        let b = f.update(Some(&mut tc), &|_| Some(10.0));
        assert_eq!(b.position, Vec3::new(5.0, 11.0, 0.0));
        assert!(f.cut && !tc.cut);
        assert!((f.base.fov - 50f32.to_radians()).abs() < 1e-5);
    }

    #[test]
    fn roaming_camera_moves_forward_along_its_heading() {
        let mut r = VsRoamingCamera::default();
        r.take_over(Vec3::ZERO, Vec3::Z);
        let (b, frozen) = r.update(&|a| if a == EAction::DebugFreeCamForwardBack { 1.0 } else { 0.0 });
        assert!(frozen);
        assert!((b.position - Vec3::Z).length() < 1e-5, "{}", b.position);
    }
}
