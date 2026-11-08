//! Render\SparkRender.cpp — collision sparks: up to 100 short ribbons ("spark1", additive) that fly, fall and
//! bounce off the ground under the player's car. Positions are kept relative to the player's car (`+16536`), so
//! sparks travel with it. Init (`Spark Render Init`, 0x00141730) and the collision handler (0x00141980) sit in
//! the SlipstreamRender.cpp range of the idb.

use bevy::math::{Vec2, Vec3, Vec4};
use visos::handy::crt_rand::CrtRand;

use crate::effect_draw::{engine_colour, EffectDraw, EffectShader, EffectTexture, EffectVertex};

/// `MAX_SPARKS`, `MAX_SPARK_SECTIONS`
pub const MAX_SPARKS: usize = 100;
pub const MAX_SECTIONS: usize = 8;
/// `dword_687144`: weakest impact that throws sparks
pub const MIN_IMPULSE: f32 = f32::from_bits(0x3F36_DB6E);
/// `dword_687148`: car against car
pub const CAR_CAR_SCALE: f32 = 0.25;
/// `dword_68714C` bounce, `flt_687150` position jitter (xz), `flt_687154` / `flt_687158` height jitter,
/// `flt_68715C` sideways speed jitter, `flt_687160` share of the car's velocity, `dword_687164` top upward speed,
/// `dword_687168` sparks per hit, `dword_68716C` sparks per unit of strength, `flt_687178` life per upward speed
pub const BOUNCE: f32 = 0.5;
pub const POS_JITTER: f32 = 0.5;
pub const HEIGHT_MIN: f32 = 0.0;
pub const HEIGHT_MAX: f32 = 0.5;
pub const SIDE_SPEED: f32 = 3.0;
pub const CAR_VELOCITY_SHARE: f32 = -1.0;
pub const MAX_UP_SPEED: f32 = 7.0;
pub const MAX_PER_HIT: i32 = 25;
pub const PER_STRENGTH: f32 = 0.2;
pub const LIFE_PER_SPEED: f32 = f32::from_bits(0x3E33_3333);
/// `+16516` gravity (0x41A00000)
pub const GRAVITY: f32 = 20.0;
/// the four spark styles (0x00141730): width, sections min / max, upward speed per strength
pub const STYLE_WIDTH: [f32; 4] = [f32::from_bits(0x3CCC_CCCD); 4];
pub const STYLE_MIN_SECTIONS: [u8; 4] = [3, 1, 2, 1];
pub const STYLE_MAX_SECTIONS: [u8; 4] = [4, 3, 4, 3];
pub const STYLE_UP_SPEED: [f32; 4] = [f32::from_bits(0x3D8F_5C29); 4];

/// a collision as the physics message carries it (`MessageManager` collision record): car index (+0), the other
/// car (+4, -1 = scenery), contact point (+8), normal (+24), impulse (+40), scale (+72), surface material (+76)
#[derive(Clone, Copy, Debug)]
pub struct SparkCollision {
    pub car: usize,
    pub other: i32,
    pub point: Vec3,
    pub normal: Vec3,
    pub impulse: f32,
    pub scale: f32,
    pub material: i8,
}

/// one spark, 164 bytes
#[derive(Clone, Debug, Default)]
pub struct Spark {
    /// +0 ring of positions relative to the origin
    pub pos: [Vec3; MAX_SECTIONS],
    /// +128 velocity, w = remaining life
    pub vel: Vec4,
    /// +144
    pub width: f32,
    /// +148 newest position, +149 positions in use, +150 ring length
    pub head: u8,
    pub count: u8,
    pub max: u8,
    /// +160 style, +161 material of the hit
    pub style: u8,
    pub material: i8,
}

#[derive(Clone, Debug)]
pub struct SparkRender {
    /// live sparks, newest first (the +16400 list)
    pub sparks: Vec<Spark>,
    /// `+16520` ground plane (normal, d)
    pub plane: Vec4,
    /// `+16536` origin (the player's car)
    pub origin: Vec3,
    /// `byte_73646D` next style
    next_style: u8,
    pub rand: CrtRand,
}

impl Default for SparkRender {
    /// SparkRender.cpp 0x001418D0: empty, gravity 20, plane y = 0
    fn default() -> Self {
        Self { sparks: Vec::new(), plane: Vec4::new(0.0, 1.0, 0.0, 0.0), origin: Vec3::ZERO, next_style: 0, rand: CrtRand::default() }
    }
}

impl SparkRender {
    fn rand_range(&mut self, lo: f32, hi: f32) -> f32 {
        (self.rand.rand_unit() * (hi - lo) as f64 + lo as f64) as f32
    }

    /// SparkRender.cpp 0x00141980 — sparks for a collision on most materials; `car_velocity` = the hit car's velocity,
    /// `ground` = its wheel plane when three wheels touch (VehicleEntity 0x001BAF60)
    pub fn on_collision(&mut self, c: &SparkCollision, car_velocity: Vec3, ground: Option<Vec4>, player_pos: Vec3) {
        if c.impulse < MIN_IMPULSE {
            return;
        }
        if !matches!(c.material, -1 | 19 | 5 | 7 | 14 | 16 | 21) {
            return;
        }
        let dir = car_velocity.normalize_or_zero();
        let d = c.normal.dot(dir);
        let vel = car_velocity * (d + 1.0);
        if let Some(p) = ground {
            self.plane = p;
        }
        let mut strength = c.impulse * c.scale;
        if c.other >= 0 {
            strength *= CAR_CAR_SCALE;
        }
        self.emit(c.point, c.normal, vel, strength, c.material, player_pos);
    }

    /// SparkRender.cpp 0x00141E40
    pub fn emit(&mut self, point: Vec3, normal: Vec3, velocity: Vec3, mut strength: f32, material: i8, player_pos: Vec3) {
        if material == 19 {
            strength *= 1.4;
        }
        let v = velocity / 2.8;
        let n = ((strength * PER_STRENGTH) as i32).min(MAX_PER_HIT);
        for _ in 0..n {
            if self.sparks.len() >= MAX_SPARKS {
                continue;
            }
            let s = self.init_spark(point, normal, v, strength, material, player_pos);
            self.sparks.insert(0, s);
        }
    }

    /// SparkRender.cpp 0x00142C50
    fn init_spark(&mut self, point: Vec3, normal: Vec3, v: Vec3, strength: f32, material: i8, player_pos: Vec3) -> Spark {
        if self.next_style > 3 {
            self.next_style = 0;
        }
        let style = self.next_style as usize;
        let r = self.rand.rand();
        // rand() >> 16 is always 0, so every spark gets the style's shortest ring
        let max = STYLE_MIN_SECTIONS[style] as i32 + (r >> 16) % (STYLE_MAX_SECTIONS[style] as i32 + 1 - STYLE_MIN_SECTIONS[style] as i32);
        let mut p = point - player_pos;
        p.x += self.rand_range(-POS_JITTER, POS_JITTER);
        p.y += self.rand_range(HEIGHT_MIN, HEIGHT_MAX);
        p.z += self.rand_range(-POS_JITTER, POS_JITTER);
        let mut up_max = strength * STYLE_UP_SPEED[style] + 4.0;
        if up_max > MAX_UP_SPEED {
            up_max = MAX_UP_SPEED;
        }
        let mut vel = normal;
        vel.x += self.rand_range(-SIDE_SPEED, SIDE_SPEED);
        vel.y += self.rand_range(1.0, up_max);
        vel.z += self.rand_range(-SIDE_SPEED, SIDE_SPEED);
        vel += v * CAR_VELOCITY_SHARE;
        self.next_style += 1;
        let mut pos = [Vec3::ZERO; MAX_SECTIONS];
        pos[0] = p;
        Spark {
            pos,
            vel: vel.extend(vel.y * LIFE_PER_SPEED),
            width: STYLE_WIDTH[style],
            head: 0,
            count: 1,
            max: max as u8,
            style: style as u8,
            material,
        }
    }

    /// SparkRender.cpp 0x00141F80 + 0x00142A80 — move every spark (`ground` = the player's wheel plane, 0x001429C0)
    pub fn update(&mut self, dt: f32, ground: Option<Vec4>) {
        if !self.sparks.is_empty() {
            if let Some(p) = ground {
                self.plane = p;
            }
        }
        let (plane, origin) = (self.plane, self.origin);
        for s in &mut self.sparks {
            let cur = s.pos[s.head as usize];
            if s.count < s.max {
                s.count += 1;
            }
            s.head += 1;
            if s.head >= s.max {
                s.head = 0;
            }
            s.pos[s.head as usize] = cur + s.vel.truncate() * dt;
            s.vel.y -= GRAVITY * dt;
            s.vel.w -= dt;
            let w = s.pos[s.head as usize] + origin;
            // below the ground plane: bounce
            if plane.truncate().dot(w) + plane.w < 0.0 {
                s.vel.y *= 0.0 - BOUNCE;
            }
        }
    }

    /// SparkRender.cpp 0x00142120 + 0x001422D0 — ribbons facing the camera; dead ones (alpha below zero) are freed
    pub fn draw(&mut self, camera: Vec3, draw: &mut EffectDraw) {
        let sh = EffectShader::new(EffectTexture::Misc("spark1".into()), 7).z(true, false);
        let origin = self.origin;
        let mut keep = Vec::with_capacity(self.sparks.len());
        for s in self.sparks.drain(..) {
            let (base, k) = if s.material == 5 || s.material == 7 { (0x0020_50FFu32, 912.0f32) } else { (0x0040_80FF, 712.0) };
            let n = s.count as usize;
            let alpha = if s.vel.w < 0.0 { Some((s.vel.w * k + 64.0) as i32) } else { None };
            if alpha.is_some_and(|a| a < 0) {
                continue;
            }
            if n >= 2 {
                let colour = engine_colour(base | ((alpha.unwrap_or(0x40) as u32) << 24));
                let step = (4096 / (n as i32 - 1)) as u16;
                let head = s.pos[s.head as usize];
                let view = head + origin - camera;
                let side = s.vel.truncate().cross(view).normalize_or_zero() * s.width;
                let mut idx = s.head as i32;
                let mut u: u16 = 0;
                let mut v = Vec::with_capacity(2 * n);
                for _ in 0..n {
                    let p = s.pos[idx as usize];
                    let uu = u as f32 / 4096.0;
                    v.push(EffectVertex::new(p + side + origin, colour, Vec2::new(uu, 0.0)));
                    v.push(EffectVertex::new(p - side + origin, colour, Vec2::new(uu, 1.0)));
                    u = u.wrapping_add(step);
                    idx -= 1;
                    if idx < 0 {
                        idx += s.count as i32;
                    }
                }
                draw.strip(&sh, &v);
            }
            keep.push(s);
        }
        self.sparks = keep;
    }
}
