//! Render\SkidMarkRender.cpp — tyre marks as triangle strips in a ring of 30 chunks of 20 vertices. A wheel that
//! slides lays grey marks ("skidmark"); front wheels that cross the start line in contact lay "skid_flame" marks
//! for 1.5 s (the burning start line), which fade over two seconds.

use bevy::math::{Vec2, Vec3};

use crate::effect_draw::{EffectDraw, EffectShader, EffectTexture, EffectVertex};
use crate::race_effects::{EffectCar, EffectCars};

pub const MAX_CHUNKS: usize = 30;
pub const MAX_CHUNK_VERTS: usize = 20;
/// `byte_686FF8` (skid marks on), `dword_686FF0` half width, `dword_686FF4` height above the road,
/// `dword_687000` mark colour, `dword_687004` squared step between segments, `dword_686FFC` depth bias
pub const ENABLED: bool = true;
pub const HALF_WIDTH: f32 = f32::from_bits(0x3E99_999A);
pub const FLAME_HALF_WIDTH: f32 = 0.15;
pub const HEIGHT: f32 = f32::from_bits(0x3DA3_D70A);
pub const MARK_COLOUR: u32 = 0x4080_8080;
pub const STEP_SQ: f32 = f32::from_bits(0x4080_8080);
pub const DEPTH_BIAS: i16 = 0x40;
/// fade per second of the flame marks, per channel (static init 0x00425DB0..: (end - start) / 2 s): red, green,
/// blue, alpha (`flt_7266EC`, `flt_7266E4`, `flt_7266E8`, `flt_7266F0`)
pub const FADE: [f32; 4] = [0.5, 0.5, 0.0, 0.2];
/// front wheels (FastCar order 0 and 3) test the start line
pub const START_LINE_WHEELS: u32 = 9;
/// time the flame marks run after crossing (0x3FC00000)
pub const FLAME_TIME: f32 = 1.5;

/// one strip, 1372 bytes in the original
#[derive(Clone, Debug, Default)]
pub struct SkidChunk {
    /// +0 point the next step is measured from, +16 / +32 last left / right edge
    pub last: Vec3,
    pub last_left: Vec3,
    pub last_right: Vec3,
    /// +48 vertices (position, D3DCOLOR)
    pub verts: Vec<(Vec3, u32)>,
    /// +1328 bounds
    pub bounds: (Vec3, Vec3),
    /// +1344 car, +1348 wheel
    pub car: i32,
    pub wheel: i32,
    /// +1356 segments laid (0 = only the start point known)
    pub state: u32,
    /// +1360 time the mark started
    pub time: f32,
    /// +1368 the wheel stopped sliding
    pub finished: bool,
    /// +1369 0 skid mark, 1 flame mark
    pub flame: bool,
}

impl SkidChunk {
    /// SkidMarkRender.cpp 0x001395F0
    fn reset(&mut self) {
        *self = Self { wheel: -1, bounds: (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), ..Default::default() };
    }

    /// SkidMarkRender.cpp 0x001396B0: `last` takes the point, which is then raised and stored as a vertex
    fn add_vertex(&mut self, p: &mut Vec3, colour: u32) {
        self.last = *p;
        p.y += HEIGHT;
        self.bounds.0 = self.bounds.0.min(*p);
        self.bounds.1 = self.bounds.1.max(*p);
        self.verts.push((*p, colour));
    }
}

/// the start line record of a wheel (VehicleEntity +5040 + 8 * wheel): flame timer, armed, side of the line
#[derive(Clone, Copy, Debug, Default)]
pub struct StartLineWheel {
    pub timer: f32,
    pub armed: bool,
    pub left: bool,
}

#[derive(Clone, Debug)]
pub struct SkidMarkRender {
    pub chunks: Vec<SkidChunk>,
    /// +41160 chunks in use, +41164 next (ring)
    pub count: usize,
    pub head: usize,
    /// per car and wheel start line records
    pub start_line: Vec<[StartLineWheel; 4]>,
}

impl Default for SkidMarkRender {
    /// RainSurface.cpp 0x001382F0 (SkidMarkRender init): all chunks reset
    fn default() -> Self {
        let mut chunks = vec![SkidChunk::default(); MAX_CHUNKS];
        chunks.iter_mut().for_each(SkidChunk::reset);
        Self { chunks, count: 0, head: 0, start_line: Vec::new() }
    }
}

/// VsAITools::IsLeftOfLine (xz)
fn is_left_of_line(a: Vec3, b: Vec3, c: Vec3) -> bool {
    (b.x - a.x) * (a.z - c.z) - (b.z - a.z) * (a.x - c.x) <= 0.0
}

impl SkidMarkRender {
    /// SkidMarkRender.cpp 0x00139170: the newest of the last 8 chunks for a car's wheel
    fn find(&self, car: i32, wheel: i32) -> Option<usize> {
        let n = self.count.min(8);
        (0..n).map(|i| (self.head + MAX_CHUNKS - i) % MAX_CHUNKS).find(|&k| {
            let c = &self.chunks[k];
            c.state != 0 && c.car == car && c.wheel == wheel
        })
    }

    /// SkidMarkRender.cpp 0x00139240
    fn new_chunk(&mut self, car: i32, wheel: i32, flame: bool) -> usize {
        let k = if self.count == MAX_CHUNKS {
            self.head = (self.head + 1) % MAX_CHUNKS;
            self.head
        } else {
            self.count += 1;
            let k = self.head;
            self.head += 1;
            k
        };
        let c = &mut self.chunks[k];
        c.reset();
        c.car = car;
        c.wheel = wheel;
        c.flame = flame;
        k
    }

    /// SkidMarkRender.cpp 0x00139100
    fn find_or_new(&mut self, car: i32, wheel: i32, flame: bool) -> usize {
        match self.find(car, wheel) {
            Some(k) if !self.chunks[k].finished => k,
            _ => self.new_chunk(car, wheel, flame),
        }
    }

    /// SkidMarkRender.cpp 0x00138500 (GameManager::InGameUpdate) — fade the flame marks, then lay marks for every car
    pub fn update(&mut self, cars: &EffectCars) {
        if !ENABLED {
            return;
        }
        let x = cars.dt * 255.0 * 1.0;
        let sub = [(x * FADE[0]) as i32 as u32, (x * FADE[1]) as i32 as u32, (x * FADE[2]) as i32 as u32, (x * FADE[3]) as i32 as u32];
        for c in self.chunks.iter_mut().filter(|c| c.flame) {
            for (_, col) in &mut c.verts {
                let ch = |v: u32, s: u32| v.saturating_sub(s);
                let a = ch(*col >> 24, sub[3]);
                let r = ch((*col >> 16) & 0xFF, sub[0]);
                let g = ch((*col >> 8) & 0xFF, sub[1]);
                let b = ch(*col & 0xFF, sub[2]);
                *col = b | (g << 8) | (r << 16) | (a << 24);
            }
        }
        if self.start_line.len() < cars.cars.len() {
            self.start_line.resize(cars.cars.len(), Default::default());
        }
        for (i, car) in cars.cars.iter().enumerate() {
            self.update_car(i, car, cars, START_LINE_WHEELS);
        }
    }

    /// SkidMarkRender.cpp 0x001398A0 (RestartRace): start line records cleared
    pub fn reset_start_line(&mut self) {
        self.start_line.iter_mut().for_each(|r| *r = Default::default());
    }

    /// SkidMarkRender.cpp 0x00138840
    fn update_car(&mut self, idx: usize, car: &EffectCar, cars: &EffectCars, mask: u32) {
        let (a, b) = cars.start_line.unwrap_or((Vec3::ZERO, Vec3::ZERO));
        let line_sq = (a - b).length_squared();
        let mid = (a + b) * 0.5;
        for w in 0..4usize {
            let wh = &car.wheels[w];
            let p = Vec3::new(wh.centre.x, wh.centre.y - wh.radius, wh.centre.z);
            let rec = &mut self.start_line[idx][w];
            if cars.start_line.is_some() && mask & (1 << w) != 0 {
                if line_sq <= (p - mid).length_squared() {
                    rec.armed = false;
                } else {
                    let left = is_left_of_line(a, b, p);
                    if rec.armed {
                        if left != rec.left && wh.contacts > 0 {
                            rec.timer = FLAME_TIME;
                        }
                    } else {
                        rec.armed = true;
                    }
                    rec.left = left;
                }
            }
            let timer = rec.timer;
            let flame = timer > 0.0;
            if (wh.sliding && !wh.no_marks) || flame {
                let mut k = self.find_or_new(idx as i32, w as i32, flame);
                if self.chunks[k].verts.len() == MAX_CHUNK_VERTS || self.chunks[k].flame != flame {
                    let old = self.chunks[k].clone();
                    let n = self.new_chunk(idx as i32, w as i32, flame);
                    let c = &mut self.chunks[n];
                    c.state = 1;
                    if old.verts.len() >= 2 {
                        c.last_left = old.verts[old.verts.len() - 2].0;
                        c.last_right = old.verts[old.verts.len() - 1].0;
                    }
                    c.last = old.last;
                    k = n;
                }
                let c = &mut self.chunks[k];
                if p.distance_squared(c.last) > STEP_SQ {
                    let dir = (p - c.last).normalize_or_zero();
                    let half = if flame { FLAME_HALF_WIDTH } else { HALF_WIDTH };
                    let perp = Vec3::new(-dir.z * half, 0.0, dir.x * half);
                    let (mut left, mut right) = (p + perp, p - perp);
                    if c.state != 0 {
                        let colour = if flame {
                            let a = if timer >= 0.5 { (255.0 * 1.0 * 0.4f32) as i32 } else { (255.0 * 1.0 * 0.4f32 * (timer / 0.5)) as i32 };
                            0x00FF_FFFF | ((a as u32) << 24)
                        } else {
                            MARK_COLOUR
                        };
                        if c.state == 1 {
                            let (mut l, mut r) = (c.last_left, c.last_right);
                            c.add_vertex(&mut l, colour);
                            c.last_left = l;
                            c.add_vertex(&mut r, colour);
                            c.last_right = r;
                        }
                        c.add_vertex(&mut left, colour);
                        c.add_vertex(&mut right, colour);
                    } else {
                        c.last_left = left;
                        c.last_right = right;
                        c.last = p;
                        c.time = cars.time;
                    }
                    c.state += 1;
                }
            } else if let Some(k) = self.find(idx as i32, w as i32) {
                self.chunks[k].finished = true;
            }
            if rec_timer_running(&self.start_line[idx][w]) {
                self.start_line[idx][w].timer -= cars.dt;
            }
        }
    }

    /// SkidMarkRender.cpp 0x00139410 — draw the marks of one kind (0 after the reflections, 1 the flame marks) as strips; uv u
    /// alternates across, v every second pair (0x00139770)
    pub fn draw(&self, flame: bool, draw: &mut EffectDraw) {
        let sh = if flame {
            EffectShader::new(EffectTexture::Misc("skid_flame".into()), 7).bias(DEPTH_BIAS).wrap(false, true).z(true, false)
        } else {
            EffectShader::new(EffectTexture::Misc("skidmark".into()), 3).bias(DEPTH_BIAS).z(true, false)
        };
        for c in self.chunks.iter().filter(|c| c.flame == flame && c.verts.len() >= 3) {
            let v: Vec<EffectVertex> = c
                .verts
                .iter()
                .enumerate()
                .map(|(i, &(p, col))| {
                    let u = if i & 1 != 0 { 1.0 } else { 0.0 };
                    let v = if i & 2 != 0 { 1.0 } else { 0.0 };
                    EffectVertex::new(p, col, Vec2::new(u, v))
                })
                .collect();
            draw.strip(&sh, &v);
        }
    }
}

fn rec_timer_running(r: &StartLineWheel) -> bool {
    r.timer > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::race_effects::EffectWheel;

    fn car_at(x: f32, sliding: bool) -> EffectCar {
        let w = |z: f32| EffectWheel { centre: Vec3::new(x, 0.33, z), radius: 0.33, contacts: 1, sliding, surface: 0, no_marks: false };
        EffectCar { wheels: [w(-0.8), w(-0.8), w(0.8), w(0.8)], ..Default::default() }
    }

    /// a sliding car driving along x lays 2 vertices per 2 m step after the first, chunks of 20 linked up; a front
    /// wheel crossing the start line switches to flame marks that fade
    #[test]
    fn marks_and_flames() {
        let mut s = SkidMarkRender::default();
        let mut cars = EffectCars { dt: 1.0 / 24.0, ..Default::default() };
        for i in 0..40 {
            cars.cars = vec![car_at(i as f32 * 2.5, true)];
            s.update(&cars);
        }
        let marks: Vec<&SkidChunk> = s.chunks.iter().filter(|c| c.wheel == 0 && !c.verts.is_empty()).collect();
        assert!(marks.len() >= 3, "chunks {}", marks.len());
        assert!(marks.iter().all(|c| c.verts.len() <= MAX_CHUNK_VERTS && !c.flame));
        assert!(marks[0].verts.iter().all(|v| (v.0.y - HEIGHT).abs() < 1e-5 || v.0.y > 0.0));
        // start line across x = 50 (z from -5 to 5)
        let mut s = SkidMarkRender::default();
        cars.start_line = Some((Vec3::new(50.0, 0.0, -5.0), Vec3::new(50.0, 0.0, 5.0)));
        for i in 0..12 {
            cars.cars = vec![car_at(46.0 + i as f32 * 0.8, false)];
            s.update(&cars);
        }
        assert!(s.chunks.iter().any(|c| c.flame && !c.verts.is_empty()), "no flame marks");
        let a0 = s.chunks.iter().find(|c| c.flame && !c.verts.is_empty()).unwrap().verts[0].1 >> 24;
        cars.cars = vec![car_at(200.0, false)];
        for _ in 0..24 {
            s.update(&cars);
        }
        let a1 = s.chunks.iter().find(|c| c.flame && !c.verts.is_empty()).unwrap().verts[0].1 >> 24;
        assert!(a1 < a0, "flame marks did not fade: {a0} -> {a1}");
    }
}
