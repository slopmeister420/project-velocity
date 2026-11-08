//! The inline float math wrappers (`VsSqrt` 0x130F0, `VsFabs` 0x64B90, ...). They call the CRT double
//! routines and leave the result on the x87 stack (`fst` to a local, then return), so the caller
//! sees the unrounded value. They return `f64` here; callers round where the original stores to a
//! float. x87 precision control is assumed to be 53-bit (the CRT default).

#[inline]
pub fn sqrt(x: f32) -> f64 {
    (x as f64).sqrt()
}
#[inline]
pub fn fabs(x: f32) -> f64 {
    (x as f64).abs()
}
#[inline]
pub fn sin(x: f32) -> f64 {
    (x as f64).sin()
}
#[inline]
pub fn cos(x: f32) -> f64 {
    (x as f64).cos()
}
#[inline]
pub fn tan(x: f32) -> f64 {
    (x as f64).tan()
}
#[inline]
pub fn exp(x: f32) -> f64 {
    (x as f64).exp()
}
#[inline]
pub fn acos(x: f32) -> f64 {
    (x as f64).acos()
}
#[inline]
pub fn asin(x: f32) -> f64 {
    (x as f64).asin()
}
#[inline]
pub fn atan(x: f32) -> f64 {
    (x as f64).atan()
}
#[inline]
pub fn atan2(y: f32, x: f32) -> f64 {
    (y as f64).atan2(x as f64)
}
#[inline]
pub fn pow(x: f32, y: f32) -> f64 {
    (x as f64).powf(y as f64)
}
#[inline]
pub fn log10(x: f32) -> f64 {
    (x as f64).log10()
}

/// `VsClamp` 0x000141E0 (SSE compares; returns via a float local, so rounded)
#[inline]
pub fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    if v > lo {
        if v > hi {
            hi
        } else {
            v
        }
    } else {
        lo
    }
}

/// x87 helpers returning a dot product in st0 (e.g. 0x0035F9D0, 0x0036AA50, 0x00361310, 0x0036D8F0)
#[inline]
pub fn dot87(a: [f32; 3], b: [f32; 3]) -> f64 {
    (a[0] as f64 * b[0] as f64) + (a[1] as f64 * b[1] as f64) + (a[2] as f64 * b[2] as f64)
}
