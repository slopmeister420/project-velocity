//! src\TimelineManager\Objects\TimelineCamera.cpp (+ the camera envelope of TimelineCameraShaker.cpp) — a
//! `VSCamera` instance becomes a dynamic camera (0x220 bytes, vtable 0x00640FD0) that a cut-scene event puts on
//! player 0's cine slot for the event's length, looking at a physics object (or the player car) with the attached
//! `F3CameraShaker`'s shake and wobble. Envelope events ramp shake, handy-cam, field of view and depth of field.

use bevy::math::{Mat4, Vec3};
use visos::handy::crt_rand::CrtRand;
use visos::handy::handy_crc::crc32;
use visos::type_::vs_type::Value;

use super::timeline_object::{
    field_bool, field_f32, field_i32, field_str, field_vec3, EventPhase, HandlerCtx, ObjectState, TimelineObjectMessage,
    TimelineObjects,
};
use crate::camera::vs_base_camera::VsBaseCamera;
use crate::game_bridge::camera_bridge::CameraBridgeData;

/// VsRandRange 0x000134F0
pub fn vs_rand_range(r: &mut CrtRand, lo: f32, hi: f32) -> f32 {
    (r.rand_unit() * (hi as f64 - lo as f64) + lo as f64) as f32
}

/// CRim 0x000452C0 — `(b - a) * t + a` on the x87 stack
pub fn lerp_x87(a: f32, b: f32, t: f64) -> f64 {
    (b as f64 - a as f64) * t + a as f64
}

/// VsCameraManager 0x000613D0 — degrees to radians (x87)
pub fn deg_to_rad(d: f32) -> f64 {
    d as f64 * f32::from_bits(0x3C8E_FA35) as f64
}

/// TimelineCamera.cpp 0x00179140 — the envelope's interpolation of `t` by name crc
pub fn interpolate(kind: i32, t: f32) -> f64 {
    let k = kind as u32;
    if k == crc32(b"linear", 0) {
        t as f64
    } else if k == crc32(b"cosine", 0) {
        (1.0 - (t * std::f32::consts::PI).cos() as f64) * 0.5
    } else if k == 0xF9FF_8B09 {
        // the third name (`off_6410B0`): squared
        (t * t).clamp(0.0, 1.0) as f64
    } else if k == crc32(b"invExp", 0) {
        (1.0 - (1.0 - t) * (1.0 - t)).clamp(0.0, 1.0) as f64
    } else {
        log::error!("assert: unknown camera envelope interpolation");
        t as f64
    }
}

/// the dynamic camera and the object's saved depth of field
#[derive(Clone, Debug)]
pub struct TimelineCamera {
    pub camera: VsBaseCamera,
    /// +0x1F4 the VSCamera attributes
    pub attributes: Option<Value>,
    /// +0x1F8 the attached shaker object (name crc)
    pub shaker: Option<u32>,
    /// +0x1FC look-at physics object (name crc); none = the player car
    pub look_at: Option<u32>,
    /// +0x200 the cut-scene owns the view
    pub active: bool,
    pub shake: f32,       // +0x204
    pub handy_cam: f32,   // +0x208 (never read by the update)
    pub phases: [f32; 3], // +0x20C..+0x214
    pub dof: f32,         // +0x218
    /// object +0x14 / +0x18 the depth of field before the cut scene (-1 = none saved)
    pub previous_dof: f32,
    pub previous_focus: f32,
}

impl TimelineCamera {
    /// TimelineAI.cpp 0x00177E00 (object) / TimelineCamera.cpp 0x001785B0 (camera) and onBind 0x00177F20: the
    /// attributes, then the frame from the instance's world matrix
    pub fn new(attributes: Option<&Value>, world: &Mat4) -> Self {
        let mut camera = VsBaseCamera::default();
        camera.set_frame(world.w_axis.truncate(), world.x_axis.truncate(), world.y_axis.truncate(), world.z_axis.truncate());
        TimelineCamera {
            camera,
            attributes: attributes.cloned(),
            shaker: None,
            look_at: None,
            active: false,
            shake: 0.0,
            handy_cam: 0.0,
            phases: [0.0; 3],
            dof: 0.0,
            previous_dof: -1.0,
            previous_focus: -1.0,
        }
    }

    /// TimelineCamera.cpp 0x00178120 (VsCrashCamera 0x00062FE0)
    pub fn set_position(&mut self, p: Vec3) {
        self.camera.pos = p;
    }

    /// TimelineCamera.cpp 0x001781C0 — take player 0's view for a cut scene
    pub fn begin(&mut self, look_at: Option<&str>, shaker: Option<&Value>, ctx: &mut HandlerCtx) {
        if !ctx.wmm.request(3, "TimelineCamera", ctx.world.game_time) {
            log::warn!("Oops, ASP ought not be denied...");
            return;
        }
        self.active = true;
        let c = super::timeline_object::ccrc(look_at.unwrap_or(""));
        self.look_at = ctx.world.objects.contains_key(&c).then_some(c);
        ctx.out.push(TimelineObjectMessage::CutSceneCamera { active: true });
        if ctx.wmm.ref_count == 1 {
            // the depth of field in force before the first cut scene (DangerZoneRender +0x08 / +0x30)
            self.previous_dof = ctx.world.dof.0;
            self.previous_focus = ctx.world.dof.1;
        }
        self.reset(shaker, ctx);
    }

    /// TimelineCamera.cpp 0x00178740 — envelopes back to the attributes
    fn reset(&mut self, shaker: Option<&Value>, ctx: &mut HandlerCtx) {
        self.phases = [0.0; 3];
        if let Some(a) = self.attributes.clone() {
            let a = Some(&a);
            let fov = deg_to_rad((self.camera.aspect as f64 * field_f32(a, "HorizontalFOV") as f64) as f32) as f32;
            self.camera.set_fov(fov);
            self.camera.set_near(field_f32(a, "NearClip"));
            self.camera.set_far(field_f32(a, "FarClip"));
            self.dof = field_f32(a, "FocusDistance");
            ctx.out.push(TimelineObjectMessage::DepthOfFieldSetup {
                enabled: field_bool(a, "DOF"),
                near: field_f32(a, "FocusDistanceNear"),
                far: field_f32(a, "FocusDistanceFar"),
            });
            ctx.out.push(TimelineObjectMessage::DepthOfFieldFocus { focus: self.dof });
        }
        match shaker {
            Some(s) => {
                self.shake = field_f32(Some(s), "ShakeAmount");
                self.handy_cam = field_f32(Some(s), "WobbleAmount");
            }
            None => {
                self.shake = 0.0;
                self.handy_cam = 0.0;
            }
        }
    }

    /// TimelineCamera.cpp 0x001783C0 — give the view back (only once the last cut scene leaves WMM mode 3)
    pub fn end(&mut self, ctx: &mut HandlerCtx) {
        ctx.wmm.leave(3, 0.0, ctx.world.game_time);
        if ctx.wmm.ref_count > 0 {
            return;
        }
        if self.previous_dof >= 0.0 {
            if self.previous_focus < 0.0 {
                log::error!("assert: m_previousDofFactor >= 0.0f && m_previousFocusDist >= 0.0f");
            }
            ctx.out.push(TimelineObjectMessage::DepthOfField { focus: self.previous_focus, factor: self.previous_dof });
            self.previous_dof = -1.0;
            self.previous_focus = -1.0;
        }
        ctx.out.push(TimelineObjectMessage::CutSceneCamera { active: false });
        ctx.out.push(TimelineObjectMessage::PlayerCarBlend(0.01));
        self.active = false;
    }

    /// TimelineCamera.cpp 0x00179340 / 0x001793A0 / 0x00179400 / 0x00179480 — one envelope step
    pub fn envelope(&mut self, kind: &str, interp: i32, begin: f32, end: f32, u: f32, ctx: &mut HandlerCtx) {
        let v = lerp_x87(begin, end, interpolate(interp, u));
        match kind {
            "shake" => self.shake = v as f32,
            "handyCam" => self.handy_cam = v as f32,
            "fieldOfView" => self.camera.set_fov((deg_to_rad(0.75) * v as f32 as f64) as f32),
            "depthOfField" => {
                self.dof = v as f32;
                ctx.out.push(TimelineObjectMessage::DepthOfFieldFocus { focus: self.dof });
            }
            _ => {}
        }
    }

    /// TimelineCamera.cpp 0x00178950 — look at the target with the shaker's shake and wobble
    pub fn update(&mut self, dt: f32, shaker: Option<&Value>, ctx: &mut HandlerCtx) -> CameraBridgeData {
        let mut target = match self.look_at.and_then(|o| ctx.world.objects.get(&o)) {
            Some(p) => *p,
            None => ctx.world.player,
        };
        if shaker.is_some() {
            target += field_vec3(shaker, "LookAtOffset");
        }
        let d = target - self.camera.pos;
        let len = d.length();
        let dir = d / len;
        let up = Vec3::Y;
        if let Some(s) = shaker {
            let s = Some(s);
            let r = &mut *ctx.rand;
            let (h, v) = (field_f32(s, "ShakeHorizontal"), field_f32(s, "ShakeVertical"));
            let z = vs_rand_range(r, 0.0 - h, h);
            let y = vs_rand_range(r, 0.0 - v, v);
            let x = vs_rand_range(r, 0.0 - h, h);
            let shake = Vec3::new(x, y, z) * ((len * self.shake) * 0.05);
            let (fmin, fmax) = (field_f32(s, "WobbleFreqMin"), field_f32(s, "WobbleFreqMax"));
            if !(0.0..=1.0).contains(&fmin) {
                log::error!("assert: shakerAttribs->fWobbleFreqMin >= 0.0f && shakerAttribs->fWobbleFreqMin <= 1.0f");
            }
            if !(0.0..=1.0).contains(&fmax) {
                log::error!("assert: shakerAttribs->fWobbleFreqMax >= 0.0f && shakerAttribs->fWobbleFreqMax <= 1.0f");
            }
            let mut wobble = [0.0f32; 3];
            for (k, (mul, amp)) in [(1.0f32, "WobbleRoll"), (1.7, "WobblePitch"), (0.8, "WobbleYaw")].into_iter().enumerate() {
                let f = vs_rand_range(r, fmin, fmax);
                let f = lerp_x87(0.1, 5.0, f as f64) as f32 * mul;
                self.phases[k] += (dt * f) * std::f32::consts::TAU;
                wobble[k] = (self.phases[k] as f64).sin() as f32 * (0.05 * field_f32(s, amp));
            }
            let side = up.cross(dir);
            let mut offset = side * wobble[2];
            offset.y = wobble[1];
            offset *= len;
            // the rolled up vector (side * roll, y = 1, normalised) is built and never used
            target += shake;
            target += offset;
        }
        let view = CameraBridgeData { position: self.camera.pos, target, up, fov: self.camera.fov };
        let pos = self.camera.pos;
        self.camera.look_at(pos, target, up);
        view
    }
}

/// the camera / shaker event handlers (vtables 0x0064110C, 0x00641134, 0x00641218)
pub fn event(objs: &mut TimelineObjects, i: usize, name: &str, value: Option<&Value>, phase: EventPhase, ctx: &mut HandlerCtx) {
    // a shaker forwards only "shake" envelopes to its camera (0x0017A0F0)
    let (cam_index, shaker_only) = match &objs.objects[i].state {
        ObjectState::Shaker { camera } => match camera.and_then(|c| objs.find(c)) {
            Some(c) => (c, true),
            None => {
                log::error!("assert: timelineCameraShaker");
                return;
            }
        },
        _ => (i, false),
    };
    let shaker_data = match &objs.objects[cam_index].state {
        ObjectState::Camera(c) => c.shaker.and_then(|s| objs.find(s)).and_then(|s| objs.objects[s].data.clone()),
        _ => return,
    };
    let ObjectState::Camera(cam) = &mut objs.objects[cam_index].state else { return };
    match (name, phase) {
        // TimelineCamera.cpp 0x00179500 / 0x001795E0
        ("F3EventCutSceneCamera", EventPhase::Begin { .. }) => cam.begin(field_str(value, "LookAt"), shaker_data.as_ref(), ctx),
        ("F3EventCutSceneCamera", EventPhase::End) => cam.end(ctx),
        // TimelineCamera.cpp 0x001796E0 / TimelineCameraShaker.cpp 0x0017A0F0; begin (0x00179690 / 0x0017A0A0) runs
        // the update with the local start time
        ("F3EventCameraEnvelope", EventPhase::Update { u } | EventPhase::Begin { t: u, .. }) => {
            let ty = field_i32(value, "EnvelopeType") as u32;
            let kind = ["shake", "handyCam", "fieldOfView", "depthOfField"].into_iter().find(|k| crc32(k.as_bytes(), 0) == ty);
            if let Some(k) = kind {
                if shaker_only && k != "shake" {
                    return;
                }
                let (b, e) = (field_f32(value, "ValueBegin"), field_f32(value, "ValueEnd"));
                cam.envelope(k, field_i32(value, "InterpolationType"), b, e, u, ctx);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolations() {
        assert_eq!(interpolate(crc32(b"linear", 0) as i32, 0.25), 0.25);
        assert!((interpolate(crc32(b"cosine", 0) as i32, 0.5) - 0.5).abs() < 1e-6);
        assert!((interpolate(crc32(b"invExp", 0) as i32, 0.5) - 0.75).abs() < 1e-6);
    }
}
