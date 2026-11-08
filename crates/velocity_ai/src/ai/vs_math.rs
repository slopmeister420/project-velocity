//! VsVector4 and scalar helpers the ai code calls (visos Maths inlines, COMDAT-folded into Armco.obj).
//!
//! Stand-in until core publishes visos maths; arithmetic order follows the debug xbe so results
//! match it as closely as f32 allows (the original runs on x87).

use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct VsVector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Armco.cpp 0x00431EB0 (static init of 0x007805C0)
pub const VS_ZERO: VsVector4 = VsVector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
/// Armco.cpp 0x00431ED0 (static init of 0x007805D0)
pub const VS_UNIT_X: VsVector4 = VsVector4 { x: 1.0, y: 0.0, z: 0.0, w: 0.0 };
/// Armco.cpp 0x00431EF0 (static init of 0x007805B0)
pub const VS_UNIT_Y: VsVector4 = VsVector4 { x: 0.0, y: 1.0, z: 0.0, w: 0.0 };

impl VsVector4 {
    /// Armco.cpp 0x00013880
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
    /// w = 1.
    /// VsAIDriver.cpp 0x00016510
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z, w: 1.0 }
    }
    /// Armco.cpp 0x00013540
    pub fn set(&mut self, x: f32, y: f32, z: f32, w: f32) {
        *self = Self { x, y, z, w };
    }
    /// Armco.cpp 0x00024140
    pub fn length_squared(&self) -> f32 {
        let a = self.x * self.x;
        let b = self.y * self.y + a;
        self.z * self.z + b
    }
    /// Armco.cpp 0x00013040
    pub fn length(&self) -> f32 {
        vs_sqrt(self.length_squared())
    }
    /// Armco.cpp 0x000135B0
    pub fn normalise(&mut self) {
        let l = self.length();
        if vs_abs(l) <= 0.000001 {
            *self = VS_ZERO;
            return;
        }
        self.x /= l;
        self.y /= l;
        self.z /= l;
    }
    /// VsAIRaceVehicle.cpp 0x000240C0
    pub fn dot3(&self, o: &Self) -> f32 {
        vs_dot3(self, o)
    }
    /// VsAITools.cpp 0x00029340
    pub fn cross(&self, o: &Self) -> Self {
        // VsAITools.cpp 0x000293D0 fills a default-constructed temp
        Self {
            x: self.y * o.z - self.z * o.y,
            y: self.z * o.x - self.x * o.z,
            z: self.x * o.y - self.y * o.x,
            w: 0.0,
        }
    }
}

/// xyz only, w untouched.
/// Armco.cpp 0x00012E60
impl SubAssign for VsVector4 {
    fn sub_assign(&mut self, o: Self) {
        self.x -= o.x;
        self.y -= o.y;
        self.z -= o.z;
    }
}
/// VsAIRaceVehicle.cpp 0x00024DE0
impl AddAssign for VsVector4 {
    fn add_assign(&mut self, o: Self) {
        self.x += o.x;
        self.y += o.y;
        self.z += o.z;
    }
}
/// VsAIDriver.cpp 0x00016580
impl MulAssign<f32> for VsVector4 {
    fn mul_assign(&mut self, s: f32) {
        self.x *= s;
        self.y *= s;
        self.z *= s;
    }
}
/// new vector, w = 1.
/// VsAIDriver.cpp 0x00016450
impl Sub for VsVector4 {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self::xyz(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
/// new vector, w = 1.
/// VsAIRaceVehicle.cpp 0x00024960
impl Add for VsVector4 {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self::xyz(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
/// new vector, w = 1.
/// VsAIRaceVehicle.cpp 0x00024020
impl Mul<f32> for VsVector4 {
    type Output = Self;
    fn mul(self, s: f32) -> Self {
        Self::xyz(self.x * s, self.y * s, self.z * s)
    }
}

/// VsAIDriver.cpp 0x00016610
pub fn vs_dot3(a: &VsVector4, b: &VsVector4) -> f32 {
    let p = a.x * b.x;
    let q = a.y * b.y + p;
    a.z * b.z + q
}
/// |b - a|
/// Armco.cpp 0x00012D10
pub fn vs_distance(a: &VsVector4, b: &VsVector4) -> f32 {
    let mut d = *b;
    d -= *a;
    d.length()
}
/// |b - a|^2
/// VsAIRaceVehicle.cpp 0x00023FA0
pub fn vs_distance_squared(a: &VsVector4, b: &VsVector4) -> f32 {
    let mut d = *b;
    d -= *a;
    d.length_squared()
}
/// exact compare of xyz.
/// VsAITools.cpp 0x000292B0
pub fn vs_vec_ne(a: &VsVector4, b: &VsVector4) -> bool {
    !(b.x == a.x && b.y == a.y && b.z == a.z)
}

/// Armco.cpp 0x000141A0
pub fn vs_abs(a: f32) -> f32 {
    if a <= 0.0 { 0.0 - a } else { a }
}
/// Armco.cpp 0x000141E0
pub fn vs_clamp(a: f32, lo: f32, hi: f32) -> f32 {
    if a <= lo {
        lo
    } else if a <= hi {
        a
    } else {
        hi
    }
}
/// VsAIDriver.cpp 0x00015FD0
pub fn vs_min(a: f32, b: f32) -> f32 {
    if a <= b { a } else { b }
}
/// sqrt is done in double precision by the crt.
/// Armco.cpp 0x000130F0
pub fn vs_sqrt(a: f32) -> f32 {
    (a as f64).sqrt() as f32
}
/// VsAIDriver.cpp 0x00016010
pub fn vs_fge(a: f32, b: f32, eps: f32) -> bool {
    a >= b - eps
}
/// VsAIDriver.cpp 0x00016050
pub fn vs_fle(a: f32, b: f32, eps: f32) -> bool {
    b + eps >= a
}
/// VsAIRaceVehicle.cpp 0x00025050
pub fn vs_feq(a: f32, b: f32, eps: f32) -> bool {
    eps > vs_abs(a - b)
}
/// VsAIRaceVehicle.cpp 0x00024580
pub fn vs_sin(a: f32) -> f32 {
    (a as f64).sin() as f32
}
/// VsAIRaceVehicle.cpp 0x000246B0
pub fn vs_cos(a: f32) -> f32 {
    (a as f64).cos() as f32
}
/// VsAIRaceVehicle.cpp 0x00024930
pub fn vs_acos(a: f32) -> f32 {
    (a as f64).acos() as f32
}
/// VsAIResponseCurve.cpp 0x00025870
pub fn vs_fmod(a: f32, b: f32) -> f32 {
    ((a as f64) % (b as f64)) as f32
}

/// the crt `rand()` the game seeds and draws from (msvc lcg, 15-bit output).
#[derive(Debug, Clone)]
pub struct VsRand {
    pub seed: u32,
}

impl Default for VsRand {
    fn default() -> Self {
        Self { seed: 1 }
    }
}

impl VsRand {
    pub fn rand(&mut self) -> i32 {
        self.seed = self.seed.wrapping_mul(214013).wrapping_add(2531011);
        ((self.seed >> 16) & 0x7FFF) as i32
    }
    /// Armco.cpp 0x00013510
    pub fn unit(&mut self) -> f32 {
        (self.rand() as f64 / 32767.0) as f32
    }
    /// Armco.cpp 0x000134F0
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        self.unit() * (hi - lo) + lo
    }
}

/// quaternion (x, y, z, w) rotating `angle` radians about `axis`.
/// TimelineSequencer.cpp 0x002B7750
pub fn vs_quat_axis_angle(angle: f32, axis: &VsVector4) -> VsVector4 {
    let h = 0.5 * angle;
    let s = vs_sin(h);
    VsVector4::new(axis.x * s, axis.y * s, axis.z * s, vs_cos(h))
}

/// `v` through the matrix of quaternion `q` (AC::Renderer 0x002AC2C0 builds the column-convention
/// matrix, D3DXVec3Transform 0x00487BF0 multiplies a row vector), i.e. rotated by the CONJUGATE of `q`.
/// Car bodies follow this convention: forward = `vs_quat_rotate(orientation, UNIT_X)`.
/// Renderer.cpp 0x002ABDE0
pub fn vs_quat_rotate(q: &VsVector4, v: &VsVector4) -> VsVector4 {
    let u = VsVector4::new(-q.x, -q.y, -q.z, 0.0);
    let t = u.cross(v) * 2.0;
    let r = *v + t * q.w + u.cross(&t);
    VsVector4::xyz(r.x, r.y, r.z)
}

/// `v` rotated by `a` about +y (D3DXMatrixRotationY 0x00488700 then D3DXVec3Transform 0x00487BF0).
pub fn vs_rotate_y(v: &VsVector4, a: f32) -> VsVector4 {
    let (s, c) = (vs_sin(a), vs_cos(a));
    VsVector4::xyz(v.x * c + v.z * s, v.y, v.z * c - v.x * s)
}
