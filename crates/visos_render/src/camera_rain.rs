//! Camera rain (object `unk_721DD0`; functions 0x00133AB0..0x00135F70 sit in LightStreak.cpp's range, the file name
//! is not in the binary; "Graphics.Rain" tweaks): on wet tracks an elParticle "RAIN" system rides with the camera
//! (`byte_721DC8`) and up to 1024 splash sprites flash on the ground around it (`byte_721DC9`).
//!
//! API (core): `CameraRainInit { wet }` at InGameInit (0x00133AB0, `VsAudio::IsTrackWet`), `CameraRainReset` at
//! RestartRace (0x00133CF0), `CameraRainShutdown` at shutdown (destructor list 0x00133CD0 → 0x00134120); fill
//! `CameraRainInput` (view 0 camera mode, car 0 position). The rain-cover trigger (TriggerUpdate 0x00181260 →
//! 0x00134EF0) is `CameraRain::cover`. Update 0x001341A0 runs per rendered frame when not paused (GameManager
//! 0x000B9F43, frame ms · 0.001), before the particle tick; the splash draw 0x00135690 every frame.
//! Dumps: `research/gfx/mist_dec.c` (0x001341A0 / 0x00135690), `rain_dec*.c`.

use bevy::prelude::*;
use visos::handy::crt_rand::CrtRand;

use crate::effect_draw::{EffectDraw,EffectShader, EffectTexture, EffectVertex};
use crate::el_particle::el_particle_control_curve::{ColourCurve, ControlCurve};
use crate::el_particle::el_particle_system::{wxyz, CollisionPlane, ElParticleType, TrailParams};
use crate::el_particle::vs_pool_allocator::EmitterShape;
use crate::el_particle::{ElParticleSystemManager, SystemId, TypeId};
use crate::light_streak::visos_colour;

/// +12: 1024 splash points (xyz, w = life 1 → 0)
pub const SPLASH_POINTS: usize = 1024;
/// the original renders at the TV rate; per-frame steps are scaled by dt·60 (`car_render::XBOX_FRAME_RATE`)
const FRAME_RATE: f32 = 60.0;
/// the Xbox back buffer the sprites are sized in (`dword_7806C8` +8 / +12)
const SCREEN: Vec2 = Vec2::new(640.0, 480.0);

/// "Graphics.Rain" tweaks 0x00135120 (defaults from the data section)
#[derive(Clone, Debug, PartialEq)]
pub struct RainTweaks {
    /// `dword_686F20` "Rain Thickness"
    pub thickness: f32,
    /// `dword_686F24` "Rain Alpha"
    pub alpha: f32,
    /// `dword_686F64` "Rain Length" (trail time)
    pub length: f32,
    /// `flt_686F68` "Rain Speed"
    pub speed: f32,
    /// `dword_686F28` "Rain spawn rate"
    pub spawn_rate: f32,
    /// `dword_686F58` "Rain crisscrossing" (velocity spread x)
    pub crisscross: f32,
    /// `dword_686F2C` / `dword_686F30` "Splash Size (far)" / "(close)"
    pub size_far: f32,
    pub size_close: f32,
    /// `dword_686F34` / `dword_686F38` "Splash width / height factor"
    pub width_factor: f32,
    pub height_factor: f32,
    /// `dword_686F3C` "Splash start forward extra"
    pub forward_extra: f32,
    /// `dword_686F40` "Splash close threshold"
    pub close: f32,
    /// `dword_686F44` "Splash Y bias"
    pub y_bias: f32,
    /// `dword_686F48` "Number of splashes"
    pub count: usize,
    /// `byte_686F4C` "Splash skewed distribution"
    pub skewed: bool,
    /// `dword_686F54` "Splash decay" (life lost per rendered frame)
    pub decay: f32,
    /// `dword_686F5C` "Splash_surface_type": Horiz 0, Plane 1, Raceline 2, Splines 3, Rays 4
    pub surface: u32,
    /// `dword_686F50` "Splash Alpha"
    pub splash_alpha: f32,
}

impl Default for RainTweaks {
    fn default() -> Self {
        Self {
            thickness: 2.0,
            alpha: f32::from_bits(0x3DCC_CCCD),
            length: 0.25,
            speed: 1.0,
            spawn_rate: 250.0,
            crisscross: 16.0,
            size_far: 46.0,
            size_close: 70.0,
            width_factor: 2.0,
            height_factor: 0.5,
            forward_extra: -1.5,
            close: 6.0,
            y_bias: 0.25,
            count: SPLASH_POINTS,
            skewed: true,
            decay: f32::from_bits(0x3DCC_CCCD),
            surface: 4,
            splash_alpha: f32::from_bits(0x3D8F_5C29),
        }
    }
}

/// `dword_686F18`: the level after init / restart
const LEVEL_START: f32 = 0.25;
/// `dword_686F1C`: level rise per second (frames under 5 s)
const LEVEL_RATE: f32 = f32::from_bits(0x3C23_D70A);
/// 0x00133D20: "RAIN" type (256, 1, 2): velocity (0, −20, 0) spread (1, 1, 1), trail 0.2, rate 250 jitter 1, size 0.2,
/// lifetime 20 (0x000790F0)
const TYPE_MAX: i32 = 256;
const TYPE_VEL: Vec3 = Vec3::new(0.0, -20.0, 0.0);
const TYPE_TRAIL: f32 = f32::from_bits(0x3E4C_CCCD);
const TYPE_RATE: f32 = 250.0;
const TYPE_SIZE: f32 = f32::from_bits(0x3E4C_CCCD);
const TYPE_LIFE: f32 = 20.0;
/// `dword_686F6C`: the "splash" collision plane 2 below (report only: kind 2, no callback user)
const PLANE_DEPTH: f32 = 2.0;
/// 0x00133D20: the system's first box (translation (−5, 10, 10), size (5, 0, 5))
const SYS_BOX_POS: Vec3 = Vec3::new(-5.0, 10.0, 10.0);
const SYS_BOX_SIZE: Vec3 = Vec3::new(5.0, 0.0, 5.0);
/// statics of 0x001341A0: box size (13, 5, 15) / under cover (19, 0, 6.5)
const BOX: Vec3 = Vec3::new(13.0, 5.0, 15.0);
const BOX_COVER: Vec3 = Vec3::new(19.0, 0.0, 6.5);
/// `dword_686FA4` box distance ahead, `dword_686F98` extra per unit of camera travel, `flt_686FA0` velocity per
/// unit of travel, `dword_686F9C` fall speed, `dword_686F94` dir y, `dword_686F90` lift, `dword_686F88` / `F8C`
const AHEAD: f32 = 15.0;
const AHEAD_PER_MOVE: f32 = 5.0;
const MOVE_VEL: f32 = 7.0;
const FALL: f32 = -20.0;
const DIR_Y: f32 = f32::from_bits(0x3F19_999A);
const LIFT: f32 = 1.0;
const SPREAD_YZ: f32 = 2.0;
const TRAIL_SIZE_END: f32 = 1.0;
/// splash ranges: `dword_686F84` base, `dword_686F80` / `dword_686F7C` far per travel / far base, `flt_686F70..78`
const NEAR_BASE: f32 = 3.0;
const FAR_PER_MOVE: f32 = 7.5;
const FAR_BASE: f32 = 25.0;
const FREE_FWD: [f32; 2] = [-6.0, 35.0];
const SIDE_MAX: f32 = 12.0;
/// 0x00134EF0's cover reach
const COVER_REACH: f32 = 30.0;

/// what the update needs from the game
#[derive(Resource, Clone, Copy, Debug)]
pub struct CameraRainInput {
    /// VsCameraManager 0x0005FFB0(0) == 1 (view 0 follows a car): splashes spread by the camera speed
    pub follow: bool,
    /// race manager driver 0's position (RainSurface 0x00135FB0 +20: the ground height when no ray hits)
    pub car0: Option<Vec3>,
}

impl Default for CameraRainInput {
    fn default() -> Self {
        Self { follow: true, car0: None }
    }
}

#[derive(Message, Clone, Debug)]
pub struct CameraRainInit {
    pub wet: bool,
}
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct CameraRainReset;
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct CameraRainShutdown;

/// `unk_721DD0`
#[derive(Resource, Debug)]
pub struct CameraRain {
    /// `byte_721DC8` / `byte_721DC9`
    pub rain: bool,
    pub splash: bool,
    pub tweaks: RainTweaks,
    /// +4 type, +8 system
    pub ty: Option<TypeId>,
    pub sys: Option<SystemId>,
    pub points: Vec<Vec4>,
    /// +16404 cover position, +16420 cover direction, +16456 / +16460 cover distances, +16468 under cover
    pub cover_pos: Vec3,
    pub cover_dir: Vec3,
    pub cover_a: f32,
    pub cover_b: f32,
    pub under_cover: bool,
    /// +16464
    pub level: f32,
    /// `dword_7260E4` / `dword_7260E0`: last camera position, last decay
    pub last_cam: Vec3,
    pub last_decay: f32,
    /// `dword_725E28`: the system transform (identity at the camera position)
    pub transform: Mat4,
    pub rng: CrtRand,
}

impl Default for CameraRain {
    fn default() -> Self {
        Self {
            rain: false,
            splash: false,
            tweaks: RainTweaks::default(),
            ty: None,
            sys: None,
            points: vec![Vec4::ZERO; SPLASH_POINTS],
            cover_pos: Vec3::ZERO,
            cover_dir: Vec3::ZERO,
            cover_a: 0.0,
            cover_b: 0.0,
            under_cover: false,
            level: 0.0,
            last_cam: Vec3::ZERO,
            last_decay: 0.0,
            transform: Mat4::IDENTITY,
            rng: CrtRand::default(),
        }
    }
}

/// the camera as the update reads it (VsBaseCamera +72 position, +88 = up × dir, +120 dir)
#[derive(Clone, Copy, Debug)]
pub struct RainCamera {
    pub pos: Vec3,
    pub left: Vec3,
    pub dir: Vec3,
}

impl CameraRain {
    /// "rainq" (0x00133AB0): PARTTEX `RainQuad`, blend 2, SetZState(1, 0), alpha func 7 ref 1, stencil write mask 0
    pub fn shader() -> EffectShader {
        EffectShader::new(EffectTexture::Particle("RainQuad".into()), 2).z(true, false).alpha_test(7, 1)
    }

    /// LightStreak.cpp 0x00133AB0 → 0x00133D20: the "RAIN" type and system (added kept, started by the update), the splash life
    /// randomised (0x00135630), both flags = wet, level 0.25
    pub fn init(&mut self, wet: bool, mgr: &mut ElParticleSystemManager) {
        self.shutdown(mgr);
        let mut ty = ElParticleType::new(TYPE_MAX, 1, 2, "RAIN");
        ty.set_velocity(TYPE_VEL, Vec3::ONE);
        ty.set_gravity(Vec3::ZERO);
        ty.set_shader_code(Self::shader());
        ty.set_trail(TrailParams { time: TYPE_TRAIL, size_end: 1.0, colour: wxyz(0.0, 0.0, 0.0, 0.0) });
        ty.set_emission_rate(ControlCurve::constant(TYPE_RATE), 1.0);
        ty.set_size(ControlCurve::constant(TYPE_SIZE), 0.0);
        ty.set_colour(ColourCurve::constant(Vec4::ONE), Vec4::ZERO);
        ty.set_lifetime(TYPE_LIFE);
        let mut plane = CollisionPlane::new(Vec3::Y, Vec3::new(0.0, -PLANE_DEPTH, 0.0), 0.0, 2);
        plane.name = "splash".into();
        plane.report = true;
        ty.add_collision_plane(plane);
        let (ty, _) = mgr.add_type(ty);
        self.ty = Some(ty);
        self.sys = mgr.add_system("RAIN", ty, false);
        if let Some(s) = self.sys.and_then(|s| mgr.system_mut(s)) {
            s.shape = EmitterShape::new(3, Mat4::from_translation(SYS_BOX_POS), SYS_BOX_SIZE);
        }
        self.randomise_life();
        self.transform = Mat4::IDENTITY;
        self.rain = wet;
        self.splash = wet;
        self.under_cover = false;
        self.reset();
    }

    /// LightStreak.cpp 0x00133CF0 (init, RestartRace)
    pub fn reset(&mut self) {
        self.level = LEVEL_START;
    }

    /// LightStreak.cpp 0x00134120: the type removed, the system removed at once
    pub fn shutdown(&mut self, mgr: &mut ElParticleSystemManager) {
        if let Some(t) = self.ty.take() {
            mgr.remove_type(t);
        }
        if let Some(s) = self.sys.take() {
            mgr.remove_system(s, true);
        }
    }

    /// LightStreak.cpp 0x00135630: every splash point gets a random life
    fn randomise_life(&mut self) {
        for p in self.points.iter_mut() {
            p.w = self.rng.rand_unit() as f32;
        }
    }

    /// LightStreak.cpp 0x00134EF0 (TriggerUpdate 0x00181260, wet tracks only): a rain cover trigger at `pos` of length `len` along
    /// `dir`; `enter` puts the camera under it, else only the distance ahead is shortened
    pub fn cover(&mut self, wet: bool, cam_pos: Vec3, cam_dir: Vec3, pos: Vec3, len: f32, dir: Vec3, enter: bool) {
        if !wet {
            return;
        }
        self.under_cover = false;
        self.cover_a = 0.0;
        self.cover_b = 0.0;
        let along = (pos - cam_pos).dot(cam_dir);
        if enter {
            self.cover_a = len * 0.5 + along;
            self.under_cover = true;
            self.cover_pos = pos;
            self.cover_dir = dir;
            let k = (len + 15.0) * 0.5;
            if cam_dir.dot(dir) <= 0.0 {
                self.cover_pos -= dir * k;
            } else {
                self.cover_pos += dir * k;
            }
        } else if along > 0.0 {
            let d = along - len * 0.5;
            if d < COVER_REACH {
                self.cover_b = COVER_REACH - d;
            }
        }
    }

    /// LightStreak.cpp 0x001341A0 — `dt` = frame ms · 0.001 (not called while paused); `ground(p)` = RainSurface 0x00136090
    pub fn update(&mut self, dt: f32, cam: RainCamera, input: &CameraRainInput, mgr: &mut ElParticleSystemManager, ground: &mut dyn FnMut(Vec3) -> f32) {
        if let Some(s) = self.sys {
            if self.rain {
                mgr.start(s);
            } else {
                mgr.stop(s);
            }
        }
        if !self.rain && !self.splash {
            return;
        }
        if dt < 5.0 {
            self.level += dt * LEVEL_RATE;
            if self.level > 1.0 {
                self.level = 1.0;
            }
        }
        let t = self.tweaks.clone();
        // camera travel per rendered frame, taken per 60 Hz frame here (any frame rate)
        let frames = (dt * FRAME_RATE).max(1e-3);
        let delta = (cam.pos - self.last_cam) / frames;
        self.last_cam = cam.pos;
        let moved = delta.length();
        if self.rain {
            let alpha = t.alpha * self.level;
            let mut ahead = AHEAD;
            let mut size = BOX;
            if self.cover_a == 0.0 {
                if self.cover_b != 0.0 {
                    ahead -= self.cover_b * 0.5;
                    size.z -= self.cover_b * 0.5;
                }
            } else if !self.under_cover {
                ahead += self.cover_a * 0.5;
                size.z -= self.cover_a * 0.5;
            }
            let mut vel = -delta * MOVE_VEL * t.speed;
            vel.y = FALL * t.speed;
            ahead += moved * AHEAD_PER_MOVE;
            let mut dir = cam.dir;
            dir.y = DIR_Y;
            let dir = dir.normalize_or_zero();
            let side = Vec3::Y.cross(dir);
            let up = dir.cross(side);
            let shape = if self.under_cover {
                let mut pos = self.cover_pos - cam.pos;
                pos.y = DIR_Y * ahead + LIFT;
                let d = self.cover_dir;
                let x = Vec3::new(-d.z, 0.0, d.x);
                size = BOX_COVER;
                Mat4::from_cols(x.extend(0.0), Vec4::Y, d.extend(0.0), pos.extend(1.0))
            } else {
                let mut pos = dir * ahead;
                pos.y += LIFT;
                Mat4::from_cols(side.extend(0.0), up.extend(0.0), dir.extend(0.0), pos.extend(1.0))
            };
            let rate = self.level * t.spawn_rate;
            if let Some(s) = self.sys.and_then(|s| mgr.system_mut(s)) {
                s.shape = EmitterShape::new(3, shape, size);
            }
            self.transform = Mat4::from_translation(cam.pos);
            if let Some(s) = self.sys.and_then(|s| mgr.system_mut(s)) {
                s.transform = Some(self.transform);
            }
            if let Some(ty) = self.ty.and_then(|x| mgr.type_mut(x)) {
                ty.set_emission_rate(ControlCurve::constant(rate), 0.0);
                ty.set_trail(TrailParams { time: t.length, size_end: TRAIL_SIZE_END, colour: wxyz(0.0, 0.0, 0.0, 0.0) });
                ty.set_size(ControlCurve::constant(t.thickness), 0.0);
                ty.set_colour(ColourCurve::constant(wxyz(alpha, 1.0, 1.0, 1.0)), Vec4::ZERO);
                ty.set_velocity(vel, Vec3::new(t.crisscross, SPREAD_YZ, SPREAD_YZ));
            }
        }
        if self.splash {
            let mut near = NEAR_BASE + t.forward_extra;
            let mut far = moved * FAR_PER_MOVE + FAR_BASE + NEAR_BASE;
            if self.cover_a == 0.0 {
                if self.cover_b != 0.0 {
                    far = far.min(COVER_REACH - self.cover_b);
                }
            } else if self.under_cover {
                near += self.cover_a;
                far = moved * FAR_PER_MOVE + FAR_BASE + near;
            } else {
                near += self.cover_a * 0.5;
            }
            if self.last_decay != t.decay {
                self.randomise_life();
                self.last_decay = t.decay;
            }
            // life lost per rendered frame, per 60 Hz frame here
            let decay = t.decay * frames;
            for i in 0..t.count.min(SPLASH_POINTS) {
                let w = self.points[i].w - decay;
                if w > 0.0 {
                    self.points[i].w = w;
                    continue;
                }
                let (fwd, side) = if input.follow {
                    // 0x00134EB0 (skewed: unit² towards the near end) / 0x00134E90 VsRandRange
                    let f = if t.skewed {
                        let a = self.rng.rand_unit() as f32;
                        let b = self.rng.rand_unit() as f32;
                        b * a * (far - near) + near
                    } else {
                        rand_range(&mut self.rng, near, far)
                    };
                    let s = (f * 0.5 + 2.0).min(SIDE_MAX);
                    (f, rand_range(&mut self.rng, 0.0 - s, s))
                } else {
                    let f = rand_range(&mut self.rng, FREE_FWD[0], FREE_FWD[1]);
                    (f, rand_range(&mut self.rng, 0.0 - SIDE_MAX, SIDE_MAX))
                };
                let mut p = cam.pos + cam.dir * fwd + cam.left * side;
                p.y = ground(p) + t.y_bias;
                self.points[i] = p.extend(1.0);
            }
        }
    }

    /// LightStreak.cpp 0x00135690 — two sprite passes (BeginSprites 0x002AA770): points from the "close threshold" on with `rain1`
    /// (2 frames by index & 1), nearer ones with `rain2` (4 frames, & 3); GenRenderer 0x002A7AE0 sprites of a fixed
    /// screen size (size · width / height factor in 640×480 pixels) with the top-left corner at the projected
    /// point, colour VISOS(_ftol(128 · level² · alpha · life), 0x80, 0x80, 0x80)
    pub fn draw(&mut self, view: &SpriteView, draw: &mut EffectDraw) {
        if !self.splash {
            return;
        }
        let t = &self.tweaks;
        let a = self.level * self.level * t.splash_alpha;
        let close2 = t.close * t.close;
        // (texture, size (u8 of the texture's 128-texel width / height), frames)
        for (k, (name, tex, size, frames)) in [("rain1", UVec2::new(128, 64), t.size_far, 2u32), ("rain2", UVec2::new(128, 128), t.size_close, 4)].into_iter().enumerate() {
            let sh = sprite_shader(name);
            let mask = frames - 1;
            let w = size * t.width_factor;
            let h = size * t.height_factor;
            for i in 0..t.count.min(SPLASH_POINTS) {
                let p = self.points[i];
                let near = close2 > (view.pos - p.truncate()).length_squared();
                if near != (k == 1) {
                    continue;
                }
                // the CRT rand() call of the loop (result unused)
                self.rng.rand();
                let f = i as u32 & mask;
                let v0 = ((tex.y * f) as f32 * (1.0 / frames as f32)) as i32 as u8;
                let v1 = ((tex.y * (f + 1)) as f32 * (1.0 / frames as f32)) as i32 as u8;
                let u1 = tex.x as u8;
                let alpha = (128.0 * a * p.w) as i32 as u8;
                let argb = visos_colour([alpha, 0x80, 0x80, 0x80]);
                if let Some(q) = view.sprite(p.truncate(), w as i32 as f32, h as i32 as f32, [0, v0, u1, v1], tex, argb) {
                    draw.quad(&sh, q);
                }
            }
        }
    }
}

/// VsRandRange 0x000134F0: unit · (hi − lo) + lo
fn rand_range(rng: &mut CrtRand, lo: f32, hi: f32) -> f32 {
    (rng.rand_unit() * (hi as f64 - lo as f64) + lo as f64) as f32
}

/// Renderer.cpp 0x002AA770 BeginSprites state: blend SRCALPHA / INVSRCALPHA (blend 2), clamp, MODULATE(texture, diffuse), z test,
/// no z write (GenRenderer flag → render state 64 off)
pub fn sprite_shader(name: &str) -> EffectShader {
    EffectShader::new(EffectTexture::Particle(name.into()), 2).z(true, false).wrap(false, false)
}

/// the world camera seen as the original's pretransformed sprite space (640×480, y down)
#[derive(Clone, Copy, Debug)]
pub struct SpriteView {
    pub pos: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub fwd: Vec3,
    /// tan(vfov / 2), width / height
    pub tan_half: f32,
    pub aspect: f32,
}

impl SpriteView {
    /// ShaderManager.cpp 0x002A7840 (projected to pixels, w > 0, x in [−W, 2W], y in [−H, 2H]) then Renderer 0x002A9D50:
    /// corners (x, y), (x + w, y), (x, y + h), (x + w, y + h) at the point's depth; uv = bytes / texture size
    pub fn sprite(&self, p: Vec3, w: f32, h: f32, uv: [u8; 4], tex: UVec2, argb: u32) -> Option<[EffectVertex; 4]> {
        let d = p - self.pos;
        let z = d.dot(self.fwd);
        if z <= 0.0 {
            return None;
        }
        let sx = d.dot(self.right) / (z * self.tan_half * self.aspect);
        let sy = d.dot(self.up) / (z * self.tan_half);
        let x = (sx + 1.0) * 0.5 * SCREEN.x;
        let y = (1.0 - sy) * 0.5 * SCREEN.y;
        if !(x <= 2.0 * SCREEN.x && x >= -SCREEN.x && y <= 2.0 * SCREEN.y && y >= -SCREEN.y) {
            return None;
        }
        let world = |px: f32, py: f32| {
            let nx = px / SCREEN.x * 2.0 - 1.0;
            let ny = 1.0 - py / SCREEN.y * 2.0;
            self.pos + self.fwd * z + self.right * (nx * z * self.tan_half * self.aspect) + self.up * (ny * z * self.tan_half)
        };
        let (u0, v0, u1, v1) = (uv[0] as f32 / tex.x as f32, uv[1] as f32 / tex.y as f32, uv[2] as f32 / tex.x as f32, uv[3] as f32 / tex.y as f32);
        Some([
            EffectVertex::new(world(x, y), argb, Vec2::new(u0, v0)),
            EffectVertex::new(world(x + w, y), argb, Vec2::new(u1, v0)),
            EffectVertex::new(world(x + w, y + h), argb, Vec2::new(u1, v1)),
            EffectVertex::new(world(x, y + h), argb, Vec2::new(u0, v1)),
        ])
    }
}

/// RainSurface.cpp 0x00136090 type 4 "Rays": a track ray from 1 above to 100 below (flags 9 = track, first hit), y =
/// start.y − t·101; no hit → driver 0's y (+20 of 0x00135FB0)
pub fn ground_by_ray(p: Vec3, fallback: f32, ray: &mut dyn FnMut(Vec3, Vec3) -> Option<f32>) -> f32 {
    let a = p + Vec3::Y;
    let b = p - Vec3::Y * 100.0;
    match ray(a, b) {
        Some(t) => a.y - t * 101.0,
        None => fallback,
    }
}

fn camera_rain_messages(
    mut rain: ResMut<CameraRain>,
    mut mgr: ResMut<ElParticleSystemManager>,
    mut init: MessageReader<CameraRainInit>,
    mut reset: MessageReader<CameraRainReset>,
    mut shutdown: MessageReader<CameraRainShutdown>,
) {
    for m in init.read() {
        rain.init(m.wet, &mut mgr);
    }
    for _ in reset.read() {
        rain.reset();
    }
    for _ in shutdown.read() {
        rain.shutdown(&mut mgr);
        rain.rain = false;
        rain.splash = false;
    }
}

/// GameManager 0x000B9F43 / 0x000B9F4D (render flag 0x800000), before the particle tick: run inside the view loop
/// by `el_particle_frame` (per view, with the view's camera; the travel since the last call then spans the two
/// views' cameras in split screen, as the original's single `dword_7260E4`)
#[allow(clippy::too_many_arguments)]
pub(crate) fn camera_rain_view(
    dt: f32,
    paused: bool,
    rain: &mut CameraRain,
    input: &CameraRainInput,
    mgr: &mut ElParticleSystemManager,
    g: &GlobalTransform,
    proj: Option<&Projection>,
    physics: Option<&velocity_physics::plugin::PhysicsWorldRes>,
    draw: &mut EffectDraw,
) {
    if rain.ty.is_none() && !rain.splash {
        return;
    }
    let pos = g.translation();
    let fwd = g.forward().as_vec3();
    let up = g.up().as_vec3();
    let right = g.right().as_vec3();
    if !paused {
        let fallback = input.car0.map_or(pos.y, |c| c.y);
        let mut ray = |a: Vec3, b: Vec3| -> Option<f32> {
            use velocity_physics::vehicle::vehicle_entity::sim::{RAY_FIRST_HIT, RAY_TRACK};
            let h = physics?.lock().as_mut()?.ray_cast(a.to_array(), b.to_array(), RAY_TRACK | RAY_FIRST_HIT, -1)?;
            Some(h.t)
        };
        let mut ground = |p: Vec3| ground_by_ray(p, fallback, &mut ray);
        let cam = RainCamera { pos, left: Vec3::Y.cross(fwd).normalize_or_zero(), dir: fwd };
        rain.update(dt, cam, input, mgr, &mut ground);
    }
    let (tan_half, aspect) = match proj {
        Some(Projection::Perspective(p)) => ((p.fov * 0.5).tan(), p.aspect_ratio),
        _ => (1.0, 4.0 / 3.0),
    };
    let view = SpriteView { pos, right, up, fwd, tan_half, aspect };
    rain.draw(&view, draw);
}

pub struct CameraRainPlugin;

impl Plugin for CameraRainPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::el_particle::ElParticlePlugin>() {
            app.add_plugins(crate::el_particle::ElParticlePlugin);
        }
        app.init_resource::<CameraRain>()
            .init_resource::<CameraRainInput>()
            .add_message::<CameraRainInit>()
            .add_message::<CameraRainReset>()
            .add_message::<CameraRainShutdown>()
            .add_systems(
                PostUpdate,
                camera_rain_messages.in_set(crate::effect_draw::EffectOrder::ElRain).before(crate::el_particle::ElParticleSet),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cam() -> RainCamera {
        RainCamera { pos: Vec3::new(0.0, 2.0, 0.0), left: Vec3::NEG_X, dir: Vec3::NEG_Z }
    }

    #[test]
    fn rain_build_and_level() {
        let mut mgr = ElParticleSystemManager::default();
        let mut r = CameraRain::default();
        r.init(true, &mut mgr);
        assert!(r.rain && r.splash);
        assert_eq!(r.level, LEVEL_START);
        let ty = mgr.type_ref(r.ty.unwrap()).unwrap();
        assert_eq!(ty.data.max_particles, TYPE_MAX);
        assert_eq!(ty.planes.len(), 1);
        let input = CameraRainInput { follow: true, car0: None };
        r.update(1.0 / 60.0, cam(), &input, &mut mgr, &mut |_| 0.0);
        assert!((r.level - (LEVEL_START + LEVEL_RATE / 60.0)).abs() < 1e-6);
        let ty = mgr.type_ref(r.ty.unwrap()).unwrap();
        // rate = level · 250, colour alpha = 0.1 · level
        assert!((ty.data.rate.eval(0.0) - r.level * 250.0).abs() < 1e-3);
        assert!((ty.data.colour.eval(0.0).w - r.level * 0.1).abs() < 1e-6);
        // the transform follows the camera
        assert_eq!(r.transform.w_axis.truncate(), cam().pos);
        // a long frame does not add level (≥ 5 s)
        let l = r.level;
        r.update(6.0, cam(), &input, &mut mgr, &mut |_| 0.0);
        assert_eq!(r.level, l);
        r.reset();
        assert_eq!(r.level, LEVEL_START);
        r.shutdown(&mut mgr);
        assert!(r.ty.is_none() && r.sys.is_none());
    }

    #[test]
    fn splashes_respawn_on_the_ground() {
        let mut mgr = ElParticleSystemManager::default();
        let mut r = CameraRain::default();
        r.init(true, &mut mgr);
        let input = CameraRainInput { follow: true, car0: None };
        // decay 0.1 per 60 Hz frame: every point respawns within 10 frames
        for _ in 0..11 {
            r.update(1.0 / 60.0, cam(), &input, &mut mgr, &mut |_| -3.0);
        }
        for p in &r.points {
            assert!((p.y - (-3.0 + 0.25)).abs() < 1e-5);
            assert!(p.w > 0.0 && p.w <= 1.0);
            // ahead of the camera, between near (1.5) and far (28) when standing still
            let along = -(p.z - 0.0);
            assert!((1.5..=28.0).contains(&along), "{along}");
            assert!(p.x.abs() <= along * 0.5 + 2.0 + 1e-4);
        }
    }

    #[test]
    fn sprites_are_fixed_screen_size() {
        let view = SpriteView { pos: Vec3::ZERO, right: Vec3::X, up: Vec3::Y, fwd: Vec3::NEG_Z, tan_half: 1.0, aspect: 4.0 / 3.0 };
        let q = view.sprite(Vec3::new(0.0, 0.0, -10.0), 92.0, 23.0, [0, 0, 128, 32], UVec2::new(128, 64), 0xFFFF_FFFF).unwrap();
        // top-left at the point; 92 px of 640 at depth 10 (half width 10·4/3) is 92/320·13.33 m
        assert!((q[0].pos - Vec3::new(0.0, 0.0, -10.0)).length() < 1e-4);
        assert!((q[1].pos.x - 92.0 / 320.0 * 10.0 * 4.0 / 3.0).abs() < 1e-4);
        assert!(q[3].pos.y < 0.0);
        assert_eq!(q[2].uv, Vec2::new(1.0, 0.5));
        assert!(view.sprite(Vec3::new(0.0, 0.0, 5.0), 1.0, 1.0, [0; 4], UVec2::ONE, 0).is_none());
        let mut ray = |_: Vec3, _: Vec3| Some(0.5);
        assert_eq!(ground_by_ray(Vec3::new(0.0, 10.0, 0.0), 3.0, &mut ray), 11.0 - 50.5);
        let mut miss = |_: Vec3, _: Vec3| None;
        assert_eq!(ground_by_ray(Vec3::ZERO, 3.0, &mut miss), 3.0);
    }
}
