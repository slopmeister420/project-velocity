//! Render\SlipstreamRender.cpp — slipstream trails: a car in another's slipstream gets 8 rotating ribbons
//! ("streek01", additive), four from the sides and four rising off the roof, sampled from wobbling points on the
//! car each frame and faded along their length. Init (0x0013F380), reset (0x0013F6E0) and the per-frame update
//! (0x0013F770) sit in the SkyRender.cpp range of the idb. The screen distortion (0x001629B0 -> 0x00140990,
//! "SlipStreamDistort") is drawn by the motion blur (0x00160FF0) over its blurred half buffer.

use bevy::math::{Mat4, Vec2, Vec3};
use visos::handy::crt_rand::CrtRand;

use crate::effect_draw::{engine_colour, EffectDraw, EffectShader, EffectTexture, EffectVertex};
use crate::full_screen_effects_manager_pc_xbox::{prog, FseImage, FseShader, FseVertex, FullScreenEffects, WHITE};

pub const MAX_TRAILS: usize = 16;
pub const MAX_TRAIL_VERTS: usize = 16;
/// `MAX_VEHICLE_COUNT`
pub const MAX_VEHICLES: usize = 22;
/// `byte_6870C0` trail length, `dword_6870DC` strength to alpha, `dword_6870E4` alpha lost per frame
pub const TRAIL_LENGTH: u8 = 16;
pub const STRENGTH_SCALE: f32 = 2.0;
pub const ALPHA_DECAY: f32 = f32::from_bits(0x3DCC_CCCD);
/// per trail type (0 side, 1 top): spin per point (`dword_6870E8` / `dword_6870F0`, by trail parity), wobble
/// (`flt_6870FC` y, `flt_687104` x), brightness (`dword_6870C4`), alpha (`dword_6870CC`), start angle
/// (`dword_68712C`), radius step per point (`dword_68711C`)
pub const SIDE_SPIN: [f32; 2] = [f32::from_bits(0x3E4C_CCCD), f32::from_bits(0xBDCC_CCCD)];
pub const TOP_SPIN: [f32; 2] = [0.5, f32::from_bits(0xBE99_999A)];
pub const WOBBLE_Y: [f32; 2] = [f32::from_bits(0x3DCC_CCCD), f32::from_bits(0x3CCC_CCCD)];
pub const WOBBLE_X: [f32; 2] = [f32::from_bits(0x3E19_999A), f32::from_bits(0x3CCC_CCCD)];
pub const BRIGHTNESS: [u32; 2] = [9, 20];
pub const ALPHA: [f32; 2] = [188.0, 100.0];
pub const START_ANGLE: [f32; 2] = [-0.5, -1.5];
pub const RADIUS_STEP: [f32; 2] = [f32::from_bits(0x3CA3_D70A), f32::from_bits(0xBC23_D70A)];
/// wobble phase steps per frame (`dword_687114`, `dword_687118` x; `dword_68710C`, `dword_687110` y)
pub const PHASE_X_STEP: [f32; 2] = [f32::from_bits(0x3F28_F5C3), f32::from_bits(0x3F19_999A)];
pub const PHASE_Y_STEP: [f32; 2] = [f32::from_bits(0x3E8A_3D71), 0.25];
/// `dword_731E00`: trail origins on the car (static init 0x00426240), 0-3 sides, 4-7 roof
pub const ORIGINS: [[u32; 3]; 8] = [
    [0xBF66_6666, 0x3E99_999A, 0xBF80_0000],
    [0xBF66_6666, 0x3E4C_CCCD, 0xBF80_0000],
    [0x3F66_6666, 0x3E99_999A, 0xBF80_0000],
    [0x3F66_6666, 0x3E4C_CCCD, 0xBF80_0000],
    [0xBF00_0000, 0x3F59_999A, 0x3FC0_0000],
    [0xBE4C_CCCD, 0x3F86_6666, 0x3FC0_0000],
    [0x3F00_0000, 0x3F59_999A, 0x3FC0_0000],
    [0x3E4C_CCCD, 0x3F86_6666, 0x3FC0_0000],
];
/// +3684 rise along the roof trails (0x0013F380)
pub const RISE: [u32; 16] = [
    0, 0x3E80_0000, 0x3EB3_3333, 0x3EDC_28F6, 0x3EE6_6666, 0x3EEB_851F, 0x3EF0_A3D7, 0x3EF5_C28F, 0x3EE6_6666, 0x3ED7_0A3D,
    0x3ECC_CCCD, 0x3EBD_70A4, 0x3EAE_147B, 0x3E99_999A, 0x3E80_0000, 0x3E4C_CCCD,
];

/// `dword_687140`: the distortion band runs from the head point towards the point this many samples older
pub const DISTORT_SPAN: usize = 4;
/// `flt_68713C`: the band's screen extent is stretched by this
pub const DISTORT_STRETCH: f32 = f32::from_bits(0x3F8C_CCCD);
/// `dword_687138` segments per screen unit of band length, at most `dword_687134`
pub const DISTORT_SEGS_PER_UNIT: f32 = 32.0;
pub const DISTORT_MAX_SEGS: i32 = 16;
/// +3764 wobble phase per segment, +3768 wobble size (×0.01 of the screen)
pub const DISTORT_FREQ: f32 = 1.0;
pub const DISTORT_SIZE: f32 = f32::from_bits(0x3E4C_CCCD);
/// the original's per-rendered-frame steps are run as frames of this rate (TV refresh, car_render::XBOX_FRAME_RATE)
const FRAME_RATE: f32 = crate::car_render::XBOX_FRAME_RATE;

/// the car matrix the original reads (`VsAIRaceVehicle__23F40` = car data +0, as CarDamageFX 0x000781C0: model space,
/// nose +Z) from `EffectCar.matrix` = the FastCar body matrix (nose +X; core's `MODEL_TO_BODY` = +90° about y)
pub fn model_matrix(body: Mat4) -> Mat4 {
    body * Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2)
}

/// what the distortion needs of the view: world -> clip, camera position / forward, the original's planes for the
/// D3D z, and the frustum's side + near planes as (normal, d) with `n·p + d >= 0` inside
pub struct DistortView<'a> {
    pub clip_from_world: Mat4,
    pub cam_pos: Vec3,
    pub cam_fwd: Vec3,
    pub near: f32,
    pub far: f32,
    pub planes: &'a [(Vec3, f32)],
}

impl DistortView<'_> {
    /// GameShaderProgramManager_PC_XBOX.cpp 0x0015EC80 — world point to (x, y) in 0..1 of the screen (y down) and
    /// D3D z (from the original near / far)
    pub fn project(&self, p: Vec3) -> (Vec2, f32) {
        let c = self.clip_from_world * p.extend(1.0);
        let xy = Vec2::new((c.x / c.w + 1.0) / 2.0, ((0.0 - c.y) / c.w + 1.0) / 2.0);
        let vz = (p - self.cam_pos).dot(self.cam_fwd).max(1e-6);
        let z = self.far * (1.0 - self.near / vz) / (self.far - self.near);
        (xy, z)
    }

    /// SlipstreamRender.cpp 0x00140820 — clip a segment against the camera planes (last to first); false when both
    /// ends lie outside one plane
    pub fn clip(&self, b: &mut Vec3, a: &mut Vec3) -> bool {
        for &(n, d) in self.planes.iter().rev() {
            let db = n.dot(*b) + d;
            let da = n.dot(*a) + d;
            if db < 0.0 && da < 0.0 {
                return false;
            }
            if db < 0.0 || da < 0.0 {
                if db < 0.0 {
                    let t = da / (da - db);
                    *b = *a + (*b - *a) * t;
                } else {
                    let t = db / (db - da);
                    *a = *b + (*a - *b) * t;
                }
            }
        }
        true
    }
}

/// one ribbon, 224 bytes
#[derive(Clone, Debug, Default)]
pub struct Trail {
    /// +0 origin on the car
    pub origin: Vec3,
    /// +16 ring of world points
    pub points: [Vec3; MAX_TRAIL_VERTS],
    /// +208 alpha, +212 spin per point
    pub alpha: f32,
    pub spin: f32,
    /// +216 texture u (1/4096)
    pub u: u16,
    /// +218 newest point (-1 = none), +219 points, +220 ring length
    pub head: i8,
    pub count: u8,
    pub max: u8,
    /// +221 vehicle, +222 active, +223 0 side / 1 top
    pub vehicle: u8,
    pub active: bool,
    pub top: bool,
}

impl Trail {
    /// SlipstreamRender.cpp 0x00140660
    fn reset(&mut self, len: u8) {
        self.head = -1;
        self.count = 0;
        self.active = false;
        self.max = len;
    }

    /// SlipstreamRender.cpp 0x001406A0
    fn add_point(&mut self, p: Vec3) {
        self.head += 1;
        if self.head >= MAX_TRAIL_VERTS as i8 {
            self.head = 0;
        }
        self.count += 1;
        if self.count >= self.max {
            self.count = self.max;
        }
        self.points[self.head as usize] = p;
    }
}

#[derive(Clone, Debug)]
pub struct SlipstreamRender {
    pub trails: Vec<Trail>,
    /// +3596 first trail of each vehicle's set
    pub sets: [Option<usize>; MAX_VEHICLES],
    /// +3748 rise height, +3752 start height, +3756 / +3760 distortion phase and speed
    pub rise_height: f32,
    pub start_height: f32,
    pub distortion_phase: f32,
    pub distortion_speed: f32,
    /// +3764 "Distortion Wave Step", +3768 "Distortion Amplitude"
    pub distortion_step: f32,
    pub distortion_amplitude: f32,
    /// +3772 texture scroll per frame, +3774 texture step per point
    pub u_scroll: i16,
    pub u_step: u16,
    /// +3780 trail radius per type
    pub width: [f32; 2],
    /// `flt_730F1C`, `dword_730F20` (x) and `flt_730F14`, `dword_730F18` (y) wobble phases
    phase_x: [f32; 2],
    phase_y: [f32; 2],
    pub rand: CrtRand,
    /// real time not yet run as 60 Hz update frames
    acc: f32,
    /// the module's tweak globals (not reset by the in-game init: carry them over when rebuilding the object)
    pub globals: SlipstreamGlobals,
}

/// "Graphics.Slipstream" globals (0x001411C0; .data defaults = the consts above)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlipstreamGlobals {
    /// `byte_6870C0` "trail length" (an S32 tweak on a byte: the menu writes 4 bytes, original bug)
    pub trail_length: u8,
    /// `dword_6870BC` "Test Vehicle" (-1; the 0x0013F770 debug path that forces a set is not ported)
    pub test_vehicle: i32,
    /// `dword_6870C4` / `unk_6870C8` "Trail Brightness", `dword_6870CC` / `unk_6870D0` "Distortion Alpha"
    pub brightness: [u32; 2],
    pub alpha: [f32; 2],
}

impl Default for SlipstreamGlobals {
    fn default() -> Self {
        Self { trail_length: TRAIL_LENGTH, test_vehicle: -1, brightness: BRIGHTNESS, alpha: ALPHA }
    }
}

impl Default for SlipstreamRender {
    /// SlipstreamRender.cpp 0x0013F380 + 0x0013F6E0
    fn default() -> Self {
        let mut trails = vec![Trail::default(); MAX_TRAILS];
        trails.iter_mut().for_each(|t| t.reset(TRAIL_LENGTH));
        Self {
            trails,
            sets: [None; MAX_VEHICLES],
            rise_height: f32::from_bits(0x3FE0_0000),
            start_height: f32::from_bits(0xBECC_CCCD),
            distortion_phase: 0.0,
            distortion_speed: -1.0,
            distortion_step: DISTORT_FREQ,
            distortion_amplitude: DISTORT_SIZE,
            u_scroll: -192,
            u_step: 256,
            width: [0.5, 0.25],
            phase_x: [0.0; 2],
            phase_y: [0.0; 2],
            rand: CrtRand::default(),
            acc: 0.0,
            globals: SlipstreamGlobals::default(),
        }
    }
}

impl SlipstreamRender {
    /// SlipstreamRender.cpp 0x0013FB30 — a vehicle is in a slipstream with this strength (VehicleEntity 0x001C5B38, PhysicsCarInput
    /// 0x001BAD9D); it gets a set of 8 trails if one of the two sets is free
    pub fn set_strength(&mut self, vehicle: usize, strength: f32) {
        if vehicle >= MAX_VEHICLES {
            log::error!("nVehicleIndex < MAX_VEHICLE_COUNT");
            return;
        }
        if self.sets[vehicle].is_none() {
            // 0x0013FD40
            let free = [0usize, 8].into_iter().find(|&k| !self.trails[k].active);
            if let Some(k) = free {
                self.sets[vehicle] = Some(k);
                for i in 0..8 {
                    let t = &mut self.trails[k + i];
                    let o = ORIGINS[i];
                    t.origin = Vec3::new(f32::from_bits(o[0]), f32::from_bits(o[1]), f32::from_bits(o[2]));
                    t.active = true;
                    t.vehicle = vehicle as u8;
                    t.top = i >= 4;
                    t.spin = if i >= 4 { TOP_SPIN[i & 1] } else { SIDE_SPIN[i & 1] };
                }
            }
        }
        if let Some(k) = self.sets[vehicle] {
            for t in &mut self.trails[k..k + 8] {
                t.alpha = (strength * STRENGTH_SCALE).min(1.0);
            }
        }
    }

    /// SlipstreamRender.cpp 0x0013F770 — once per rendered frame while not paused: fade, sample new points from the cars' render
    /// matrices (`car_matrix(vehicle)`), free sets that faded out
    pub fn update(&mut self, car_matrix: impl Fn(usize) -> Option<Mat4>) {
        self.phase_x[0] += PHASE_X_STEP[0];
        self.phase_y[0] += PHASE_Y_STEP[0];
        self.phase_x[1] += PHASE_X_STEP[1];
        self.phase_y[1] += PHASE_Y_STEP[1];
        for i in 0..MAX_TRAILS {
            if !self.trails[i].active {
                continue;
            }
            self.trails[i].alpha -= ALPHA_DECAY;
            let t = &self.trails[i];
            if t.alpha > 0.0 {
                let Some(m) = car_matrix(t.vehicle as usize) else { continue };
                let ty = t.top as usize;
                let mut p = t.origin;
                p.x += self.phase_x[i & 1].sin() * WOBBLE_X[ty];
                p.y += self.phase_y[i & 1].cos() * WOBBLE_Y[ty];
                let w = m.transform_point3(p);
                let step = self.u_scroll as u16;
                let t = &mut self.trails[i];
                t.add_point(w);
                t.u = t.u.wrapping_add(step) & 0xFFF;
            } else {
                let v = t.vehicle as usize;
                self.sets[v] = None;
                let len = self.globals.trail_length;
                self.trails[i].reset(len);
            }
        }
        self.distortion_phase += self.distortion_speed;
    }

    /// `update` once per 60 Hz frame of the real delta `dt` (the original runs it once per rendered TV frame)
    pub fn update_dt(&mut self, dt: f32, car_matrix: impl Fn(usize) -> Option<Mat4>) {
        self.acc = (self.acc + dt).min(0.25);
        while self.acc >= 1.0 / FRAME_RATE {
            self.acc -= 1.0 / FRAME_RATE;
            self.update(&car_matrix);
        }
    }

    /// ProjectedLightBeam_PC_XBOX.cpp 0x001629B0 (slipstream object, `a3` = the motion blur's half buffer): when the
    /// first trail of either set has two or more points, every such trail is drawn as one strip by 0x00140990 with
    /// "SlipStreamDistort" (0x0013F380: defaults after CreateShader's reset = blend 1, z test LESSEQUAL without
    /// write, alpha func ALWAYS, stage MODULATE(texture, diffuse)), sampling `source` at a wobbled screen position
    pub fn render_distortion(&self, fse: &mut FullScreenEffects, source: FseImage, view: &DistortView) {
        if !(0..MAX_TRAILS).step_by(8).any(|i| self.trails[i].count > 1) {
            return;
        }
        let size = fse.image_size(source).as_vec2();
        let shader = FseShader { ffp: crate::full_screen_effects_manager_pc_xbox::FFP_DIFFUSE_ALPHA, ..FseShader::new(1, true) };
        for t in &self.trails {
            if t.count > 1 {
                if let Some((v, z)) = self.distortion_strip(t, size, view) {
                    fse.draw_strip_z(FseImage::BackBuffer, source, shader, prog::COPY, v, z);
                }
            }
        }
    }

    /// SlipstreamRender.cpp 0x00140990 — the band from the head point to `count / 4` times the step to the point
    /// four samples older (stale ring entries are read as the original does), ± the trail radius in y (roof trails
    /// raised by the start height / rise), clipped, projected, stretched by 1.1 and split into up to 16 segments whose
    /// uv (texels of `size`) is pushed in v by sin(i + phase) · 0.2 · 0.01
    pub fn distortion_strip(&self, t: &Trail, size: Vec2, view: &DistortView) -> Option<(Vec<FseVertex>, Vec<f32>)> {
        if t.count < 2 {
            log::error!("Trail.m_nPoints >= 2");
        }
        let h = t.head as i32;
        let p0 = t.points[h.rem_euclid(MAX_TRAIL_VERTS as i32) as usize];
        let mut h2 = h - DISTORT_SPAN as i32;
        if h2 < 0 {
            h2 += MAX_TRAIL_VERTS as i32;
        }
        let mut p1 = t.points[h2 as usize];
        let sb = t.count as f32 / DISTORT_SPAN as f32;
        p1 = p0 + (p1 - p0) * sb;
        let ty = t.top as usize;
        let r = self.width[ty];
        let up = Vec3::new(0.0, r, 0.0);
        let (mut a, mut b, mut c, mut d) = (p0 + up, p1 + up, p0 - up, p1 - up);
        if t.top {
            a.y += self.start_height;
            c.y += self.start_height;
            let lift = f32::from_bits(RISE[DISTORT_SPAN]) * self.rise_height + self.start_height;
            b.y += lift;
            d.y += lift;
        }
        let top = view.clip(&mut a, &mut b);
        let bottom = view.clip(&mut c, &mut d);
        if !(top | bottom) {
            return None;
        }
        let pa = view.project(a);
        let pb = view.project(b);
        let pc = view.project(c);
        let pd = view.project(d);
        let e1 = ((pb.0 - pa.0) * DISTORT_STRETCH, (pb.1 - pa.1) * DISTORT_STRETCH);
        let e2 = ((pd.0 - pc.0) * DISTORT_STRETCH, (pd.1 - pc.1) * DISTORT_STRETCH);
        let len = (e1.0.x * e1.0.x + e1.0.y * e1.0.y).sqrt();
        let n = ((len * DISTORT_SEGS_PER_UNIT) as i32 + 1).min(DISTORT_MAX_SEGS);
        let mut v = Vec::with_capacity(2 * n as usize + 2);
        let mut zs = Vec::with_capacity(2 * n as usize + 2);
        for i in 0..=n {
            let f = i as f32 / n as f32;
            let p = (pa.0 + e1.0 * f, pa.1 + e1.1 * f);
            let q = (pc.0 + e2.0 * f, pc.1 + e2.1 * f);
            let w = (i as f32 * self.distortion_step + self.distortion_phase).sin() * self.distortion_amplitude * 0.01;
            for (pos, z) in [p, q] {
                v.push(FseVertex { pos, uv: Vec2::new(pos.x * size.x, (pos.y + w) * size.y), colour: WHITE });
                zs.push(z);
            }
        }
        Some((v, zs))
    }

    /// SlipstreamRender.cpp 0x0013FDB0 / 0x0013FF20 — every trail with two or more points as a strip turning round its points
    pub fn draw(&mut self, car_matrix: impl Fn(usize) -> Option<Mat4>, draw: &mut EffectDraw) {
        let sh = EffectShader::new(EffectTexture::Misc("streek01".into()), 7).z(true, false).alpha_test(7, 0x20);
        for k in 0..MAX_TRAILS {
            let t = self.trails[k].clone();
            if t.count <= 1 {
                continue;
            }
            let ty = t.top as usize;
            let n = t.count as usize;
            let b = self.globals.brightness[ty];
            let rgb = b | (b << 8) | (b << 16);
            let alpha = self.globals.alpha[ty] * t.alpha;
            let mut idx = t.head as i32;
            let mut u = t.u;
            let mut angle = START_ANGLE[ty];
            let mut radius = self.width[ty];
            let m = car_matrix(t.vehicle as usize).unwrap_or(Mat4::IDENTITY);
            let mut v = Vec::with_capacity(2 * n);
            for i in 0..n {
                let p = t.points[idx as usize];
                let ang = angle;
                angle += t.spin;
                // VsRandUnit is still drawn, scaled by flt_730F24 (0)
                let r = (self.rand.rand_unit() * 0.0 + radius as f64) as f32;
                let off = m.transform_vector3(Vec3::new(ang.sin() * r, ang.cos() * r, 0.0));
                let (mut a, mut c) = (p + off, p - off);
                if t.top {
                    let rise = f32::from_bits(RISE[i]) * self.rise_height + self.start_height;
                    a.y += rise;
                    c.y += rise;
                }
                radius += RADIUS_STEP[ty];
                let fade = if i == 0 { 0.0 } else { 1.0 - i as f32 / (n as f32 - 1.0) };
                let a8 = (fade * alpha) as i32 as u32;
                let col = engine_colour(rgb | (a8 << 24));
                let uu = u as f32 / 4096.0;
                v.push(EffectVertex::new(a, col, Vec2::new(uu, 0.0)));
                v.push(EffectVertex::new(c, col, Vec2::new(uu, 1.0)));
                idx -= 1;
                if idx < 0 {
                    idx += MAX_TRAIL_VERTS as i32;
                }
                u = u.wrapping_add(self.u_step);
            }
            draw.strip(&sh, &v);
        }
    }
}
