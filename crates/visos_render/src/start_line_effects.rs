//! Render\StartLineEffects.cpp (0x00146D50..0x00147D40) — the start lights over the start line in professional
//! races (two rows of five "LensFlareSecondaryBlob" glows, red, then green and raised once the race is on) and
//! the burning start line laid behind the flag girl's "startlinefireeffect" (stateless "startline_fire_segment"
//! emitters every metre; that type is not shipped, so it only warns, as in the original).

use bevy::math::{Mat4, Vec2, Vec3, Vec4};

use crate::effect_draw::{EffectDraw, EffectShader, EffectTexture, EffectVertex};
use crate::stateless_particles::stateless_particle_mgr::CStatelessParticleMgr;

/// `dword_687260` / `dword_687264` / `dword_7365F0` (bss, 0): light block offset from the line centre (world axes)
pub const LIGHTS_OFFSET: Vec3 = Vec3::new(f32::from_bits(0xBEE6_6666), f32::from_bits(0x4048_F5C3), 0.0);
/// `flt_687268` spacing along the line, `dword_68726C` row spacing, `flt_687270` glow half size
pub const SPACING: f32 = f32::from_bits(0x3ED7_0A3D);
pub const ROW_SPACING: f32 = f32::from_bits(0x3E61_47AE);
pub const HALF_SIZE: f32 = 0.5;
/// `flt_7366C4` (static init 0x004264B0: 1 / 2)
pub const FIRE_HALF_STEP: f32 = 0.5;

#[derive(Clone, Debug)]
pub struct StartLineEffects {
    /// +96 frame on the line: rows along the line, up, across, centre
    pub frame: Mat4,
    /// +176 bounding sphere (centre, radius)
    pub sphere: Vec4,
    /// +196 0 off, 2 red, 3 green
    pub state: u32,
    /// +160 last fire point, +201 no fire point yet
    fire_last: Vec3,
    fire_first: bool,
}

impl Default for StartLineEffects {
    fn default() -> Self {
        Self { frame: Mat4::IDENTITY, sphere: Vec4::ZERO, state: 0, fire_last: Vec3::ZERO, fire_first: true }
    }
}

impl StartLineEffects {
    /// StartLineEffects.cpp 0x00146E20 (reset, also RestartRace via 0x00146D50)
    pub fn reset(&mut self) {
        self.state = 0;
        self.fire_last = Vec3::ZERO;
        self.fire_first = true;
    }

    /// StartLineEffects.cpp 0x00146FE0 — the frame from the start line (`dword_6F5C24` / `dword_6F5C34`), its first point being the
    /// one the player's car (position, forward) faces away from
    pub fn setup(&mut self, line: (Vec3, Vec3), car_pos: Vec3, car_forward: Vec3) {
        let to = (line.0 - car_pos).normalize_or_zero();
        let (a, b) = if car_forward.dot(to) <= 0.0 { (line.0, line.1) } else { (line.1, line.0) };
        let mid = (a + b) * 0.5;
        let dir = (b - a).normalize_or_zero();
        let up = Vec3::Y;
        let side = up.cross(dir).normalize_or_zero();
        self.frame = Mat4::from_cols(dir.extend(0.0), up.extend(0.0), side.extend(0.0), mid.extend(1.0));
        // 0x00147CA0
        let c = mid + Vec3::new(0.0, LIGHTS_OFFSET.y, 0.0);
        self.sphere = c.extend(4.0 * SPACING);
    }

    /// StartLineEffects.cpp 0x00147280 (GameManager::InGameUpdate): lights only in professional races (`byte_6F70E9`), green once
    /// `byte_6F5BDD` (set by the flag girl's go) is set
    pub fn update(&mut self, professional: bool, go: bool) {
        if professional {
            self.state = if go { 3 } else { 2 };
        }
    }

    /// StartLineEffects.cpp 0x001472F0 (TimelineSequencer 0x00174A70): the fire point moved; lay a segment emitter per metre
    pub fn fire_moved(&mut self, p: Vec3, mgr: &mut CStatelessParticleMgr) {
        if self.fire_first {
            self.fire_last = p;
            self.fire_first = false;
            return;
        }
        let d = self.fire_last.distance(p);
        if d <= 1.0 {
            return;
        }
        let dir = (p - self.fire_last).normalize_or_zero();
        let side = dir.cross(Vec3::Y).normalize_or_zero();
        let up = side.cross(dir);
        let n = (d / 1.0) as i32;
        let mut at = self.fire_last + dir * FIRE_HALF_STEP;
        for _ in 0..n {
            let m = Mat4::from_cols(side.extend(0.0), up.extend(0.0), dir.extend(0.0), at.extend(1.0));
            mgr.spawn_named("startline_fire_segment", m);
            at += dir * 1.0;
        }
        self.fire_last += dir * n as f32;
    }

    /// StartLineEffects.cpp 0x001475B0 / 0x001475F0 (after the world) — camera position, right and up (render frame +88 / +104)
    pub fn draw(&self, cam_pos: Vec3, cam_right: Vec3, cam_up: Vec3, draw: &mut EffectDraw) {
        if self.state == 0 {
            return;
        }
        let sh = EffectShader::new(EffectTexture::Misc("lensflaresecondaryblob".into()), 12).z(true, false).bias(256);
        let mut base = self.frame.w_axis.truncate() + LIGHTS_OFFSET;
        let right = cam_right * HALF_SIZE;
        let up = cam_up * HALF_SIZE;
        let (rows, mut colour) = if self.state == 3 {
            base.y += ROW_SPACING * 2.0;
            (2, 0xFF00_C800u32)
        } else {
            (self.state, 0xFFFF_0000u32)
        };
        let view = (base - cam_pos).normalize_or_zero();
        let d = view.dot(self.frame.z_axis.truncate()).max(0.0);
        // VISOS_ColourXBOX 0x002B1510: alpha = d * 255
        colour = (colour & 0x00FF_FFFF) | (((d * 255.0) as i32 as u32 & 0xFF) << 24);
        let along = self.frame.x_axis.truncate() * SPACING;
        let mut p = base;
        for row in 0..rows {
            for _ in 0..5 {
                // 0x001479B0
                // stream +72 = camera up (render frame +104), +88 = camera right (+88)
                let v = [
                    EffectVertex::new(p - right + up, colour, Vec2::new(0.0, 0.0)),
                    EffectVertex::new(p + right + up, colour, Vec2::new(1.0, 0.0)),
                    EffectVertex::new(p - right - up, colour, Vec2::new(0.0, 1.0)),
                    EffectVertex::new(p + right - up, colour, Vec2::new(1.0, 1.0)),
                ];
                draw.strip(&sh, &v);
                p += along;
            }
            p = base + Vec3::new(0.0, ROW_SPACING * (row + 1) as f32, 0.0);
        }
    }
}
