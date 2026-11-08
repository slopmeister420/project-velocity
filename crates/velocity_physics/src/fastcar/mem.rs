//! Byte-offset view of a FastCar object. The library reads its members as floats and ints through
//! the same storage (and compares float members as ints), so objects are kept as raw 32-bit words
//! addressed by their original byte offset; every access below mirrors one `this + N` of the binary.

pub type V3 = [f32; 3];
pub type V4 = [f32; 4];

/// Raw object storage, addressed by byte offset (4-byte aligned).
#[derive(Clone, Debug, PartialEq)]
pub struct Mem {
    w: Vec<u32>,
}

impl Mem {
    pub fn new(bytes: usize) -> Self {
        Self { w: vec![0; bytes.div_ceil(4)] }
    }
    /// little-endian file image (zero-padded to a word)
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut m = Self::new(b.len());
        for (k, c) in b.chunks(4).enumerate() {
            let mut w = [0u8; 4];
            w[..c.len()].copy_from_slice(c);
            m.w[k] = u32::from_le_bytes(w);
        }
        m
    }
    pub fn len_bytes(&self) -> usize {
        self.w.len() * 4
    }
    #[inline]
    pub fn f(&self, off: usize) -> f32 {
        f32::from_bits(self.w[off >> 2])
    }
    #[inline]
    pub fn set_f(&mut self, off: usize, v: f32) {
        // PHYSNANTRAP=1: panic at the first NaN store (the -1 sentinel excepted)
        if v.is_nan() && v.to_bits() != 0xFFFF_FFFF && nan_trap() {
            panic!("NaN stored at mem +{off}");
        }
        self.w[off >> 2] = v.to_bits();
    }
    #[inline]
    pub fn i(&self, off: usize) -> i32 {
        self.w[off >> 2] as i32
    }
    #[inline]
    pub fn set_i(&mut self, off: usize, v: i32) {
        self.w[off >> 2] = v as u32;
    }
    #[inline]
    pub fn u(&self, off: usize) -> u32 {
        self.w[off >> 2]
    }
    #[inline]
    pub fn set_u(&mut self, off: usize, v: u32) {
        self.w[off >> 2] = v;
    }
    /// byte member (stored in the low byte of a little-endian word)
    #[inline]
    pub fn b(&self, off: usize) -> u8 {
        (self.w[off >> 2] >> ((off & 3) * 8)) as u8
    }
    #[inline]
    pub fn set_b(&mut self, off: usize, v: u8) {
        let s = (off & 3) * 8;
        let w = &mut self.w[off >> 2];
        *w = (*w & !(0xFF << s)) | ((v as u32) << s);
    }
    #[inline]
    pub fn v3(&self, off: usize) -> V3 {
        [self.f(off), self.f(off + 4), self.f(off + 8)]
    }
    #[inline]
    pub fn set_v3(&mut self, off: usize, v: V3) {
        self.set_f(off, v[0]);
        self.set_f(off + 4, v[1]);
        self.set_f(off + 8, v[2]);
    }
    #[inline]
    pub fn v4(&self, off: usize) -> V4 {
        [self.f(off), self.f(off + 4), self.f(off + 8), self.f(off + 12)]
    }
    #[inline]
    pub fn set_v4(&mut self, off: usize, v: V4) {
        for (k, x) in v.iter().enumerate() {
            self.set_f(off + 4 * k, *x);
        }
    }
    /// 3x3 matrix stored as three rows of 4 floats (stride 16)
    #[inline]
    pub fn m34(&self, off: usize) -> M34 {
        let mut m = [0.0; 12];
        for (k, x) in m.iter_mut().enumerate() {
            *x = self.f(off + 4 * k);
        }
        m
    }
    #[inline]
    pub fn set_m34(&mut self, off: usize, m: &M34) {
        for (k, x) in m.iter().enumerate() {
            self.set_f(off + 4 * k, *x);
        }
    }
    /// 3x3 matrix stored densely (9 floats)
    #[inline]
    pub fn m33(&self, off: usize) -> M33 {
        let mut m = [0.0; 9];
        for (k, x) in m.iter_mut().enumerate() {
            *x = self.f(off + 4 * k);
        }
        m
    }
    #[inline]
    pub fn set_m33(&mut self, off: usize, m: &M33) {
        for (k, x) in m.iter().enumerate() {
            self.set_f(off + 4 * k, *x);
        }
    }
    pub fn words(&self, off: usize, n: usize) -> &[u32] {
        &self.w[off >> 2..(off >> 2) + n]
    }
    pub fn copy_words(&mut self, dst: usize, src: usize, n: usize) {
        self.w.copy_within(src >> 2..(src >> 2) + n, dst >> 2);
    }
}

/// rows of 4 (the 4th column is padding or a translation)
pub type M34 = [f32; 12];
pub type M33 = [f32; 9];

// ---- the small vector helpers the library inlines as separate functions; float order is kept

#[inline]
pub fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
#[inline]
pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
#[inline]
pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
#[inline]
pub fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
/// `a += b * s` written as `b*s + a` (the library's operand order)
#[inline]
pub fn madd(a: V3, b: V3, s: f32) -> V3 {
    [b[0] * s + a[0], b[1] * s + a[1], b[2] * s + a[2]]
}
#[inline]
pub fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
#[inline]
pub fn neg(a: V3) -> V3 {
    [0.0 - a[0], 0.0 - a[1], 0.0 - a[2]]
}
/// rows `m[0..3], m[4..7], m[8..11]`: row-vector dot products (`R * v`)
#[inline]
pub fn mul_rows(m: &M34, v: V3) -> V3 {
    [
        dot([m[0], m[1], m[2]], v),
        dot([m[4], m[5], m[6]], v),
        dot([m[8], m[9], m[10]], v),
    ]
}
/// columns (`R^T * v`), operand order `m[0]*v0 + m[4]*v1 + m[8]*v2`
#[inline]
pub fn mul_cols(m: &M34, v: V3) -> V3 {
    [
        m[0] * v[0] + m[4] * v[1] + m[8] * v[2],
        m[1] * v[0] + m[5] * v[1] + m[9] * v[2],
        m[2] * v[0] + m[6] * v[1] + m[10] * v[2],
    ]
}
#[inline]
pub fn row(m: &M34, r: usize) -> V3 {
    [m[4 * r], m[4 * r + 1], m[4 * r + 2]]
}

/// `PHYSNANTRAP` set: trap NaN stores (diagnostic)
pub fn nan_trap() -> bool {
    static T: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *T.get_or_init(|| std::env::var("PHYSNANTRAP").is_ok())
}
