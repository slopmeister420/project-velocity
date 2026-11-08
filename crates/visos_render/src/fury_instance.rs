//! Render\FuryInstance.cpp (0x00128E40..0x0012AE10; asserts at 0x00128E40 / 0x00129910 name the file) — one
//! "fury" (nitrous) instance per vehicle index (`NUM_FURYRENDER_INSTANCES` = 6, 1844 bytes at FuryFX +36): the
//! speed streaks round the car, the light beams of every car while the player's nitrous runs, the exhaust
//! "nitrous" particles and the start / stop of the nos sound. The object itself is `fury_fx`.

use bevy::math::{Mat4, Vec2, Vec3, Vec4};
use visos::handy::crt_rand::CrtRand;

use crate::effect_draw::{EffectShader, EffectTexture, EffectVertex};
use crate::fury_fx::{FuryBeam, FuryTweaks, MAX_CAR_BEAMS};
use crate::light_streak::visos_colour;
use crate::stateless_particles::stateless_particle_emitter::flags;
use crate::stateless_particles::stateless_particle_mgr::{CStatelessParticleMgr, NO_EMITTER};

/// the 30 streak records (+0, 52 bytes each)
pub const NUM_STREAKS: usize = 30;
/// `flt_686B78`: streak alpha byte per unit of amount
pub const STREAK_ALPHA: f32 = 8.0;
/// `word_686B74`: u step per vertex pair (1/3 of 4096)
pub const STREAK_U_STEP: u16 = 0x555;
/// `dword_686B70`: streak half width
pub const STREAK_WIDTH: f32 = 0.25;
/// the `0.33f` of 0x00129E10: segment length per unit of +36
pub const STREAK_SEGMENT: f32 = 0.33;
/// `dword_686B7C`: spawn distance ahead of the car (model z)
pub const STREAK_AHEAD: f32 = 3.0;
/// `dword_686B80`: spawn radius round the car's nose axis
pub const STREAK_RADIUS: f32 = f32::from_bits(0x3F8C_CCCD);
/// `flt_686B84` / `flt_686B88`: speed = rand(1, 2) · 1
pub const STREAK_SPEED_SCALE: f32 = 1.0;
pub const STREAK_SPEED_MAX: f32 = 2.0;
/// `dword_686B90` / `dword_686B8C`: +36 grows by 6 while below 6
pub const STREAK_LENGTH_MAX: f32 = 6.0;
pub const STREAK_LENGTH_STEP: f32 = 6.0;
/// `word_71CEA4` (bss, never written): u scroll per frame
pub const STREAK_U_SCROLL: u16 = 0;
/// `dword_686B68`: nitrous particle offset along the model z
pub const NOS_PARTICLE_Z: f32 = f32::from_bits(0xBDCC_CCCD);
/// `dword_686B6C`: the blur amount per unit of fury (0x00129580)
pub const BLUR_SCALE: f32 = f32::from_bits(0x3BF5_C28F);
/// "nitrous" (`\Particle\PartFX.bin`)
pub const NOS_PARTICLE: &str = "nitrous";

/// one speed streak (52 bytes)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FuryStreak {
    /// +0 position relative to the car
    pub pos: Vec3,
    /// +16 direction (−car z at spawn)
    pub vel: Vec3,
    /// +32 distance per 60 Hz frame
    pub speed: f32,
    /// +36 length
    pub length: f32,
    /// +40 alive
    pub alive: bool,
    /// +48 u offset (4096 = 1)
    pub u: u16,
}

/// one exhaust emitter (24 bytes at +1748): +0 handle, +4 started, +8 locator offset (x and z negated)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NosEmitter {
    pub handle: usize,
    pub started: bool,
    pub offset: Vec3,
}

/// what the instance reads of its vehicle (VehicleManager vehicle `+1716`)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FuryVehicle {
    /// VsAIRaceVehicle 0x00024870 (FastCar +32 velocity · +112 row 0, m/s)
    pub speed: f32,
    /// VsAIRaceVehicle 0x00024EF0: VehicleEntity +88 (nitrous on)
    pub nos: bool,
    /// `flt_75B27C`: the physics' nitrous fade (VehicleEntity 0x001D1850, 0..1)
    pub fade: f32,
}

/// what 0x001293E0 asks of the audio (VsAudio::StartNos / StopNos on `dword_6D4558`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NosSound {
    Start,
    Stop,
}

/// the instance (1844 bytes)
#[derive(Clone, Debug)]
pub struct FuryInstance {
    /// +0 streaks
    pub streaks: [FuryStreak; NUM_STREAKS],
    /// +1560 the instance's own beam record (only ever reset)
    pub own_beam: FuryBeam,
    /// +1716 vehicle index
    pub vehicle: usize,
    /// +1720 views (`dword_74A434` +292, at most 1)
    pub views: i32,
    /// +1724 active
    pub active: bool,
    /// +1725 0x001297F0's argument
    pub started_by: bool,
    /// +1728 nitrous fade while VE+88 is on, else 0
    pub nos: f32,
    /// +1732 forward mph / max nitro speed, 0..1
    pub speed: f32,
    /// +1736 the FuryFX object (a pointer; implicit here)
    /// +1740 streaks in use
    pub count: usize,
    /// +1744 count, +1748 records
    pub emitters: Vec<NosEmitter>,
    /// car data +2252 bits 0x00128A40 ors in (1 left, 2 right, 4 centre, 8 both sides); no reader found
    pub nos_sides: u32,
}

impl Default for FuryInstance {
    fn default() -> Self {
        Self {
            streaks: [FuryStreak::default(); NUM_STREAKS],
            own_beam: FuryBeam::default(),
            vehicle: 0,
            views: 0,
            active: false,
            started_by: false,
            nos: 0.0,
            speed: 0.0,
            count: NUM_STREAKS,
            emitters: Vec::new(),
            nos_sides: 0,
        }
    }
}

/// VsRandRange 0x000134F0
fn rand_range(rng: &mut CrtRand, lo: f32, hi: f32) -> f32 {
    (rng.rand_unit() * (hi - lo) as f64 + lo as f64) as f32
}

/// VsVector4::Normalise 0x000135B0 (|v| ≤ 1e-6 → zero)
fn normalise(v: Vec3) -> Vec3 {
    let l = v.length();
    if l.abs() <= 0.000001 {
        return Vec3::ZERO;
    }
    Vec3::new(v.x / l, v.y / l, v.z / l)
}

/// "streak" (+28, FuryFX 0x00128100): spec4, blend 2, wrap, z test no write, alpha func 7 ref 1, stencil write
/// mask 0 (0x002A15F0(1) unmodelled)
pub fn streak_shader() -> EffectShader {
    EffectShader::new(EffectTexture::Misc("spec4".into()), 2).wrap(true, true).z(true, false).alpha_test(7, 1)
}

impl FuryInstance {
    /// FuryInstance.cpp 0x001292C0
    pub fn init(&mut self, vehicle: usize, views: i32) {
        self.nos = 0.0;
        self.active = false;
        self.started_by = false;
        self.vehicle = vehicle;
        self.views = views.min(1);
        for _ in 0..self.views {
            self.own_beam.reset();
        }
        self.count = NUM_STREAKS;
    }

    /// FuryFX.cpp 0x00128680
    pub fn amount(&self) -> f32 {
        if self.active {
            self.nos * self.speed
        } else {
            0.0
        }
    }

    /// FuryFX.cpp 0x00128A40 — the exhaust emitters; `nos` = the car's `l_nos` / `r_nos` (car-locator convention),
    /// None when the car is not set up (+1800 == 0). Old emitters are not released (original: a restart leaks them).
    pub fn init_particles(&mut self, mgr: &mut CStatelessParticleMgr, nos: Option<[Option<Vec3>; 2]>) {
        self.emitters.clear();
        let Some(nos) = nos else { return };
        // the found locators packed in order
        for loc in nos.into_iter().flatten() {
            let h = mgr.add_emitter(visos::handy::handy_crc::crc32(NOS_PARTICLE.as_bytes(), 0));
            if h == NO_EMITTER {
                log::warn!("FuryFX::InitParticleFX - no free emitters");
                continue;
            }
            // (−x, y, −z) of the raw locator; ours already has z negated
            let offset = Vec3::new(-loc.x, loc.y, loc.z);
            if offset.x.abs() > 0.05 {
                if offset.x >= -0.05 {
                    if offset.x > 0.05 {
                        self.nos_sides |= 2;
                    }
                } else {
                    self.nos_sides |= 1;
                }
            } else {
                self.nos_sides |= 4;
            }
            if self.nos_sides & 1 != 0 && self.nos_sides & 2 != 0 {
                self.nos_sides |= 8;
            }
            self.emitters.push(NosEmitter { handle: h, started: false, offset });
        }
    }

    /// FuryFX.cpp 0x00128DC0
    pub fn release_particles(&mut self, mgr: &mut CStatelessParticleMgr) {
        for e in &mut self.emitters {
            mgr.kill_emitter(e.handle);
            e.handle = NO_EMITTER;
        }
    }

    /// FuryInstance.cpp 0x00128E40 (GameManager 0x000B9BF9 per human player, every rendered frame): the speed factor,
    /// the beams of this car reset when nitrous starts, +1728 = the nitrous fade. `zone` = ZoneRender 0x00154FC0.
    pub fn update(&mut self, v: &FuryVehicle, zone: f32, paused: bool, tweaks: &FuryTweaks, beams: &mut [FuryBeam]) {
        let v12 = (v.speed as f64 * 2.25) as f32;
        self.speed = if v12 >= 0.0 {
            if v12 <= tweaks.max_nitro_speed {
                v12 / tweaks.max_nitro_speed
            } else {
                1.0
            }
        } else {
            0.0
        };
        if paused {
            return;
        }
        if v.nos && zone <= 0.0 && self.nos <= 0.0 {
            let b = 2 * self.vehicle;
            if b >= MAX_CAR_BEAMS {
                log::error!("assert: nBeam < MAX_CAR_BEAMS");
            } else {
                beams[b].reset();
                beams[b + 1].reset();
            }
        }
        self.nos = if v.nos { v.fade } else { 0.0 };
    }

    /// FuryInstance.cpp 0x00129090 (GameManager 0x000B9C0A): start the exhaust emitters while the amount is up, stop
    /// them after, and carry them on the car (`car` = its matrix, nose +Z)
    pub fn update_particles(&mut self, mgr: &mut CStatelessParticleMgr, car: &Mat4) {
        let a = self.nos * self.speed;
        for e in &mut self.emitters {
            let Some(em) = mgr.emitters.get_mut(e.handle) else {
                log::error!("assert: EmitterIndex < SLPARTICLEMGR_MAXEMITTERS");
                continue;
            };
            if a <= 0.0 {
                // 0x002D4EF0 stops it and returns 1
                if em.flags & flags::RUNNING != 0 && e.started {
                    em.stop();
                    e.started = false;
                }
            } else if em.flags & flags::RUNNING == 0 {
                em.start();
                e.started = true;
            }
            if em.flags & flags::RUNNING != 0 {
                let mut off = e.offset;
                off.x = -off.x;
                off.z += NOS_PARTICLE_Z;
                let mut m = *car;
                m.w_axis = car.transform_point3(off).extend(1.0);
                em.matrix = m;
            }
        }
    }

    /// FuryInstance.cpp 0x001297F0: on (beams and streaks cleared); `amount_global` = `dword_6F5B70`
    pub fn start(&mut self, by: bool, beams: &mut [FuryBeam], amount_global: &mut f32) {
        self.started_by = by;
        self.active = true;
        self.nos = 0.0;
        *amount_global = 0.0;
        for b in beams.iter_mut().take(MAX_CAR_BEAMS) {
            b.reset();
        }
        self.own_beam.reset();
        for s in &mut self.streaks {
            s.alive = false;
        }
    }

    /// FuryInstance.cpp 0x001298E0
    pub fn stop(&mut self, amount_global: &mut f32) {
        self.active = false;
        *amount_global = 0.0;
    }

    /// the state half of FuryInstance.cpp 0x001293E0 (GameManager 0x000B9CBE per view, render flag 1): the sound,
    /// on / off; returns the amount (+1728 · +1732 as it stood on entry). The caller then draws the car beams
    /// (0x00129910) and, while on with an amount, the streaks (`streak_count` + 0x00129670).
    #[allow(clippy::too_many_arguments)]
    pub fn render_state(
        &mut self,
        nos: bool,
        zone: f32,
        paused: bool,
        nos_playing: &mut bool,
        beams: &mut [FuryBeam],
        amount_global: &mut f32,
        sound: &mut Vec<NosSound>,
    ) -> f32 {
        let v7 = nos && zone <= 0.0;
        let v8 = self.nos * self.speed;
        if !paused && !v7 && self.nos <= 0.0 {
            sound.push(NosSound::Stop);
            *nos_playing = false;
            self.stop(amount_global);
        }
        if v7 && !self.active {
            self.start(true, beams, amount_global);
        }
        // `dword_6D4558 & 0x20`: the sound object's nos-playing flag (mirrored here)
        if v7 && !*nos_playing {
            sound.push(NosSound::Start);
            *nos_playing = true;
        }
        v8
    }

    /// FuryInstance.cpp 0x001293E0: +1740 = (int)(30 · +1732) before the streaks are drawn
    pub fn set_streak_count(&mut self) {
        self.count = ((NUM_STREAKS as f32 * self.speed) as i32).clamp(0, NUM_STREAKS as i32) as usize;
    }

    /// FuryInstance.cpp 0x0012A2B0 (from 0x00129670 unless paused): move the streaks, respawn the ones behind the
    /// camera's plane 4 (`plane` = normal, w; 0x00057200 = n·p + w). One original step per rendered (60 Hz) frame;
    /// `frames` = 60 Hz frames this frame, run as whole steps plus the remainder.
    pub fn update_streaks(&mut self, frames: f32, car: &Mat4, plane: Vec4, rng: &mut CrtRand) {
        let mut left = frames;
        while left > 0.0 {
            let f = left.min(1.0);
            left -= f;
            self.step_streaks(f, car, plane, rng);
        }
    }

    fn step_streaks(&mut self, f: f32, car: &Mat4, plane: Vec4, rng: &mut CrtRand) {
        let car_pos = car.w_axis.truncate();
        // VsRender 0x0005C820(car, 2): model z row
        let z = car.z_axis.truncate();
        for i in 0..self.count {
            let s = &mut self.streaks[i];
            s.pos += s.vel * (s.speed * f);
            s.u = s.u.wrapping_add(STREAK_U_SCROLL) & 0xFFF;
            if STREAK_LENGTH_MAX > s.length {
                s.length += STREAK_LENGTH_STEP * f;
            }
            let p = s.pos + car_pos;
            let d = plane.truncate().dot(p) + plane.w;
            if d < 0.0 || !s.alive {
                s.alive = true;
                s.speed = rand_range(rng, 1.0, STREAK_SPEED_MAX) * STREAK_SPEED_SCALE;
                s.length = 0.0;
                let y = rand_range(rng, 0.0, 1.0);
                let x = rand_range(rng, -1.0, 1.0);
                let mut d = normalise(Vec3::new(x, y, 0.0)) * STREAK_RADIUS;
                d.z = STREAK_AHEAD;
                // Renderer 0x002ABF70: rotation only
                s.pos = car.transform_vector3(d);
                s.vel = Vec3::new(0.0, 0.0, 0.0) - z;
                s.u = 0;
            }
        }
    }

    /// FuryInstance.cpp 0x00129E10 — the streak strips (8 vertices each) at amount `a` round the car at `car_pos`
    pub fn streak_strips(&self, a: f32, car_pos: Vec3) -> Vec<[EffectVertex; 8]> {
        let alpha = ((a * STREAK_ALPHA) as i64 as u32) as u8;
        let c = [
            visos_colour([alpha, 0x80, 0x40, 0x40]),
            visos_colour([alpha, 0x60, 0x40, 0x40]),
            visos_colour([alpha, 0x40, 0x40, 0x40]),
            visos_colour([alpha, 0x40, 0x40, 0x40]),
        ];
        let mut out = Vec::with_capacity(self.count);
        for s in &self.streaks[..self.count.min(NUM_STREAKS)] {
            let side = Vec3::Y.cross(s.vel) * STREAK_WIDTH;
            let step = s.vel * (s.length * STREAK_SEGMENT);
            let mut p = s.pos + car_pos;
            let mut u = s.u;
            let mut v = [EffectVertex::default(); 8];
            for k in (0..8).step_by(2) {
                let uu = u as f32 / 4096.0;
                v[k] = EffectVertex::new(p - side, c[k / 2], Vec2::new(uu, 0.0));
                v[k + 1] = EffectVertex::new(p + side, c[k / 2], Vec2::new(uu, 1.0));
                u = u.wrapping_add(STREAK_U_STEP);
                p += step;
            }
            out.push(v);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn beams() -> Vec<FuryBeam> {
        vec![FuryBeam::default(); MAX_CAR_BEAMS]
    }

    /// FuryInstance.cpp 0x00128E40 / 0x001293E0: nitrous on → active, amount = fade · speed factor; off and faded → stop sound, off
    #[test]
    fn start_amount_stop() {
        let t = FuryTweaks::default();
        let mut b = beams();
        let mut f = FuryInstance::default();
        f.init(0, 1);
        let (mut playing, mut g, mut snd) = (false, 0.5, Vec::new());
        let v = FuryVehicle { speed: 65.0 / 2.25 / 2.0, nos: true, fade: 0.8 };
        f.update(&v, 0.0, false, &t, &mut b);
        assert!((f.speed - 0.5).abs() < 1e-6);
        assert_eq!(f.nos, 0.8);
        let a = f.render_state(true, 0.0, false, &mut playing, &mut b, &mut g, &mut snd);
        assert!((a - 0.4).abs() < 1e-6);
        assert!(f.active && playing && g == 0.0);
        assert_eq!(snd, vec![NosSound::Start]);
        // start zeroes +1728 until the next update
        assert_eq!(f.amount(), 0.0);
        f.update(&v, 0.0, false, &t, &mut b);
        assert!((f.amount() - 0.4).abs() < 1e-6);
        // the zone blocks it
        snd.clear();
        let off = FuryVehicle { nos: false, ..v };
        f.update(&off, 0.0, false, &t, &mut b);
        f.render_state(false, 0.0, false, &mut playing, &mut b, &mut g, &mut snd);
        assert!(!f.active && !playing);
        assert_eq!(snd, vec![NosSound::Stop]);
    }

    /// 0x0012A2B0: a dead streak respawns 3 m ahead within 1.1 of the nose axis and runs backwards
    #[test]
    fn streak_spawn_and_strip() {
        let mut f = FuryInstance::default();
        f.speed = 1.0;
        f.set_streak_count();
        assert_eq!(f.count, 30);
        let car = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let mut rng = CrtRand::default();
        // plane far behind: nothing respawns once alive
        let plane = Vec4::new(0.0, 0.0, 1.0, 1000.0);
        f.update_streaks(1.0, &car, plane, &mut rng);
        for s in &f.streaks {
            assert!(s.alive);
            assert_eq!(s.pos.z, 3.0);
            assert!((Vec2::new(s.pos.x, s.pos.y).length() - STREAK_RADIUS).abs() < 1e-5);
            assert!(s.pos.y >= 0.0);
            assert_eq!(s.vel, Vec3::new(0.0, 0.0, -1.0));
            assert_eq!(s.length, 0.0);
        }
        let before = f.streaks[0];
        f.update_streaks(1.0, &car, plane, &mut rng);
        let s = f.streaks[0];
        assert!((s.pos.z - (before.pos.z - before.speed)).abs() < 1e-5);
        assert_eq!(s.length, 6.0);
        let strips = f.streak_strips(1.0, car.w_axis.truncate());
        assert_eq!(strips.len(), 30);
        let v = strips[0];
        // alpha byte 8 doubled; red 0x80 doubled saturates
        assert_eq!(v[0].argb, 0x10FF_8080);
        assert_eq!(v[6].argb, 0x1080_8080);
        assert_eq!((v[1].pos - v[0].pos).length(), 0.5);
        assert!((v[2].pos.z - v[0].pos.z + 6.0 * 0.33).abs() < 1e-5);
        assert_eq!(v[1].uv, Vec2::new(0.0, 1.0));
    }
}
