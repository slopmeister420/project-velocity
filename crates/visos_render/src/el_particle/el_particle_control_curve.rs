//! `elParticleControlCurve_T` (template header, no .cpp of its own; constructors inlined into the users' ranges):
//! a value over a particle's age through four keys at 0, t1, t2, t3, optionally looping.

use bevy::math::Vec4;

/// `elParticleControlCurve_T<float>`, 32 bytes: +0..+12 values, +16 t1, +20 t2, +24 t3 (end), +28 loop
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlCurve {
    pub v: [f32; 4],
    pub t: [f32; 3],
    pub looped: bool,
}

/// `elParticleControlCurve_T<VsVector4>`, 80 bytes: +0..+48 colours (x r, y g, z b, w a), +64 t1, +68 t2, +72 t3,
/// +76 loop
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColourCurve {
    pub c: [Vec4; 4],
    pub t: [f32; 3],
    pub looped: bool,
}

/// elParticleSystem.cpp 0x002D0870
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (b - a) * t + a
}

impl ControlCurve {
    /// CarDamageFX.cpp-range 0x00079750 (`CarDamageFX__791C0`): constant value, times 1, no loop
    pub fn constant(a: f32) -> Self {
        Self { v: [a; 4], t: [1.0; 3], looped: false }
    }

    /// CarDamageFX.cpp-range 0x000791F0: keys (0, v0), (t1, v1), (t2, v2) — v3 = v2, t3 = t2. "t2 < 0" clamps t1 to
    /// 0, "t3 < t2" raises t2 (only) to t1.
    pub fn keys(v0: f32, t1: f32, v1: f32, t2: f32, v2: f32, looped: bool) -> Self {
        let mut c = Self { v: [v0, v1, v2, v2], t: [t1, t2, t2], looped };
        if c.t[0] < 0.0 {
            log::warn!("Error: elParticleControlCurve_T::elParticleControlCurve_T: t2 < 0.");
            c.t[0] = 0.0;
        }
        if c.t[0] > c.t[1] {
            log::warn!("Error: elParticleControlCurve_T::elParticleControlCurve_T: t3 < t2.");
            c.t[1] = c.t[0];
        }
        c
    }

    /// elParticleSystem.cpp 0x002D0520
    pub fn eval(&self, mut t: f32) -> f32 {
        if t <= 0.0 {
            return self.v[0];
        }
        if self.looped {
            t %= self.t[2];
        } else if t >= self.t[2] {
            return self.v[3];
        }
        if t > self.t[0] {
            if t >= self.t[1] {
                lerp(self.v[2], self.v[3], (t - self.t[1]) / (self.t[2] - self.t[1]))
            } else {
                lerp(self.v[1], self.v[2], (t - self.t[0]) / (self.t[1] - self.t[0]))
            }
        } else {
            lerp(self.v[0], self.v[1], t / self.t[0])
        }
    }
}

/// elParticleRenderer.cpp 0x003440E0: a + (b − a)·t per component
fn lerp4(a: Vec4, b: Vec4, t: f32) -> Vec4 {
    (b - a) * t + a
}

impl ColourCurve {
    /// LightStreak.cpp-range 0x00135D50 / 0x00135E20: one colour, times 1, no loop
    pub fn constant(c: Vec4) -> Self {
        Self { c: [c; 4], t: [1.0; 3], looped: false }
    }

    /// CarDamageFX.cpp-range 0x000792E0: c0 at age 0, c1 from `t` on (c2 = c3 = c1, t1 = t2 = t3 = t); "t2 < 0"
    /// clamps t1 to 0
    pub fn two(c0: Vec4, t: f32, c1: Vec4, looped: bool) -> Self {
        let mut c = Self { c: [c0, c1, c1, c1], t: [t; 3], looped };
        if c.t[0] < 0.0 {
            log::warn!("Error: elParticleControlCurve_T::elParticleControlCurve_T: t2 < 0.");
            c.t[0] = 0.0;
        }
        c
    }

    /// elParticleRenderer.cpp 0x00343E80
    pub fn eval(&self, mut t: f32) -> Vec4 {
        if t <= 0.0 {
            return self.c[0];
        }
        if self.looped {
            t %= self.t[2];
        } else if t >= self.t[2] {
            return self.c[3];
        }
        if t > self.t[0] {
            if t >= self.t[1] {
                lerp4(self.c[2], self.c[3], (t - self.t[1]) / (self.t[2] - self.t[1]))
            } else {
                lerp4(self.c[1], self.c[2], (t - self.t[0]) / (self.t[1] - self.t[0]))
            }
        } else {
            lerp4(self.c[0], self.c[1], t / self.t[0])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curves() {
        let c = ControlCurve::keys(1.0, 0.2, 2.0, 0.5, 4.0, false);
        assert_eq!((c.eval(-1.0), c.eval(0.1), c.eval(0.2), c.eval(0.35), c.eval(0.5), c.eval(9.0)), (1.0, 1.5, 2.0, 3.0, 4.0, 4.0));
        assert_eq!(ControlCurve::constant(20.0).eval(3.0), 20.0);
        // t1 > t2 raises t2 only
        let c = ControlCurve::keys(0.0, 2.0, 1.0, 1.0, 3.0, false);
        assert_eq!(c.t, [2.0, 2.0, 1.0]);
        let l = ControlCurve { v: [0.0, 1.0, 1.0, 0.0], t: [1.0, 2.0, 3.0], looped: true };
        assert_eq!(l.eval(3.5), 0.5);
        let k = ColourCurve::two(Vec4::new(1.0, 1.0, 1.0, 0.25), 0.5, Vec4::new(1.0, 1.0, 1.0, 0.0), false);
        assert_eq!(k.eval(0.25).w, 0.125);
        assert_eq!(k.eval(0.5).w, 0.0);
    }
}
