//! `CCatMullRom` / `CSimpleSpline` (code 0x0007BA20..0x0007D380 and 0x0007FBD0..0x00080F50, the idb's Characters /
//! CircuitEditorTools ranges): the Catmull-Rom spline the hangout camera flies along (FEHangoutScreen 0x0021B0D0 /
//! 0x0021AE70). Only the variant those use is ported: flags 2 (a fixed parameter step, `+36`), open (flag 1
//! clear), no banking (flag 0x80 clear, so the roll / `+44` passes 0x0007C970 / 0x0007CDA0 do nothing).

/// `CCatMullRom` (flags 2)
#[derive(Clone, Debug, Default)]
pub struct CCatMullRom {
    /// +36 parameter step per output point
    pub step: f32,
    /// +20 control points
    pub ctrl: Vec<[f32; 4]>,
    /// +0 output points
    pub out: Vec<[f32; 4]>,
}

impl CCatMullRom {
    /// 0x0007BA20 — ctor(step, flags 2); `+40` 5.0 and `+44` 0.001 only matter for banking
    pub fn new(step: f32) -> Self {
        Self { step, ctrl: Vec::new(), out: Vec::new() }
    }

    /// 0x0007BB40 — `AddCtrlPt`
    pub fn add(&mut self, p: [f32; 4]) {
        self.ctrl.push(p);
    }

    /// 0x0007C1A0 — `Build(useCtrlW = 0)`: the segments (p0, p1, p2, p3) with the ends doubled, then the last
    /// control point
    pub fn build(&mut self) {
        let n = self.ctrl.len();
        if n < 2 {
            return;
        }
        self.out.clear();
        let ctrl = self.ctrl.clone();
        let c = |i: usize| ctrl[i.min(n - 1)];
        let mut segs = vec![[c(0), c(0), c(1), c(2)]];
        if n > 2 {
            for i in 1..n - 2 {
                segs.push([c(i - 1), c(i), c(i + 1), c(i + 2)]);
            }
            segs.push([c(n - 3), c(n - 2), c(n - 1), c(n - 1)]);
        }
        for s in segs {
            self.segment(s);
        }
        self.out.push(c(n - 1));
    }

    /// 0x0007C4C0 — one segment: p1, then t = step, 2 step, ... while t < 1 - step / 2 (step from 0x0007C440:
    /// `+36` as is with flag 2)
    fn segment(&mut self, p: [[f32; 4]; 4]) {
        let dt = self.step;
        let end = 1.0 - dt * 0.5;
        let dw = (p[2][3] - p[1][3]) * dt;
        let mut w = p[1][3];
        let mut t = dt;
        self.out.push(p[1]);
        w += dw;
        while end > t {
            let t2 = t * t;
            let t3 = t2 * t;
            let b0 = ((-0.5 * t3) + (1.0 * t2)) - (0.5 * t) + 0.0;
            let b1 = ((1.5 * t3) - (2.5 * t2)) + (0.0 * t) + 1.0;
            let b2 = ((-1.5 * t3) + (2.0 * t2)) + (0.5 * t) + 0.0;
            let b3 = ((0.5 * t3) - (0.5 * t2)) + (0.0 * t) + 0.0;
            let mut o = [0.0; 4];
            for k in 0..3 {
                o[k] = p[3][k] * b3 + (p[2][k] * b2 + (p[1][k] * b1 + p[0][k] * b0));
            }
            o[3] = w;
            self.out.push(o);
            t += dt;
            w += dw;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spline_passes_through_the_control_points() {
        let mut s = CCatMullRom::new(f32::from_bits(0x3E19999A)); // 0.15 (dword_6C128C)
        s.add([0.0, 0.0, 0.0, 1.0]);
        s.add([10.0, 0.0, 0.0, 1.0]);
        s.build();
        // 1 + 6 samples (t = 0.15 .. 0.9) + the end point
        assert_eq!(s.out.len(), 8);
        assert_eq!(s.out[0], [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(s.out[7], [10.0, 0.0, 0.0, 1.0]);
        assert!(s.out.windows(2).all(|w| w[1][0] > w[0][0]));
    }
}
