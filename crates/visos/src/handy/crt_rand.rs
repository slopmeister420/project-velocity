//! the XDK CRT `rand()` (0x003DE030, per-thread state at +20 of the thread data, seed 1) and `VsRandUnit`
//! (0x00013510) on top of it. one generator per owner; the original shares a single state.

#[derive(Clone, Debug)]
pub struct CrtRand {
    pub state: u32,
}

impl Default for CrtRand {
    fn default() -> Self {
        Self { state: 1 }
    }
}

impl CrtRand {
    /// `srand`
    pub fn seed(&mut self, s: u32) {
        self.state = s;
    }
    /// 0x003DE030 — MS LCG, 0..=0x7FFF
    pub fn rand(&mut self) -> i32 {
        self.state = self.state.wrapping_mul(214013).wrapping_add(2531011);
        ((self.state >> 16) & 0x7FFF) as i32
    }
    /// VsRandUnit 0x00013510 — 0..=1, returned on the x87 stack (float return, rounded by the caller's store)
    pub fn rand_unit(&mut self) -> f64 {
        self.rand() as f64 / 32767.0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn msvc_sequence() {
        let mut r = super::CrtRand::default();
        // msvc rand() from seed 1
        assert_eq!([r.rand(), r.rand(), r.rand()], [41, 18467, 6334]);
    }
}
