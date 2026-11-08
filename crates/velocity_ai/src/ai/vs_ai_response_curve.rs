//! Piecewise-linear response curves: `AI/ResCurve/<name>.txt` (src/Ai/VsAIResponseCurve.cpp).
//!
//! File: `(n)` then n `(input, output)` pairs, inputs ascending. n must be > 1.

use std::path::Path;

use super::vs_math::vs_fmod;
use crate::crc::string_crc;
use crate::text_file::VsTextFile;

/// 60 bytes in the original.
#[derive(Debug, Clone, Default)]
pub struct VsAIResponseCurve {
    pub items: Vec<(f32, f32)>, // +0x00 m_dNumItems, +0x04 items
    pub wrap: bool,             // +0x08 input wraps modulo the last x
    pub name: String,           // +0x09 char[35]
    pub crc: u32,               // +0x2C
    pub draw: bool,             // +0x30 debug-draw while evaluating
    pub draw_pos: (i32, i32),   // +0x34 debug-draw screen position
}

impl VsAIResponseCurve {
    /// VsAIResponseCurve.cpp 0x00025130
    pub fn new() -> Self {
        Self::default()
    }

    /// loads `<ai_dir>/ResCurve/<name>.txt`.
    /// VsAIResponseCurve.cpp 0x00025350
    pub fn load(&mut self, ai_dir: &Path, name: &str) -> bool {
        self.name = name.to_owned();
        self.crc = string_crc(&self.name);
        let path = crate::resolve(ai_dir, &format!("ResCurve\\{name}.txt"));
        let Ok(mut f) = VsTextFile::load(&path) else {
            log::error!("Error loading text file {}", path.display());
            return false;
        };
        self.load_from(&mut f)
    }

    /// body of `load` once the file is open.
    /// VsAIResponseCurve.cpp 0x00025350
    pub fn load_from(&mut self, f: &mut VsTextFile) -> bool {
        let n = f.read_single_number().unwrap_or(0.0) as i32 as u32; // ftol
        if n <= 1 {
            log::error!("\n ** AI_ASSERT - Attempted to load invalid Response Curve file. **");
        }
        self.items = (0..n).map(|_| f.read_double_number().unwrap_or((0.0, 0.0))).collect();
        true
    }

    /// clamps below the first and above the last point, lerps in between.
    /// VsAIResponseCurve.cpp 0x00025610
    pub fn get_value(&self, mut x: f32) -> f32 {
        debug_assert!(!self.items.is_empty(), "m_dNumItems > 0");
        let n = self.items.len();
        if self.items[0].0 <= x {
            if self.wrap {
                x = vs_fmod(x, self.items[n - 1].0);
            }
            for i in 1..n {
                let a = self.items[i - 1];
                let b = self.items[i];
                if b.0 > x {
                    let t = (x - a.0) / (b.0 - a.0);
                    return (b.1 - a.1) * t + a.1;
                }
            }
            self.items[n - 1].1
        } else {
            self.items[0].1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_response_curves() {
        let Some(dir) = crate::game_dir() else { return };
        let ai = dir.join("AI");
        let mut n = 0;
        for e in std::fs::read_dir(ai.join("ResCurve")).unwrap() {
            let p = e.unwrap().path();
            let name = p.file_stem().unwrap().to_string_lossy().into_owned();
            let mut rc = VsAIResponseCurve::new();
            assert!(rc.load(&ai, &name));
            assert!(rc.items.len() > 1, "{name}");
            assert!(rc.items.windows(2).all(|w| w[0].0 < w[1].0), "{name} not ascending");
            n += 1;
        }
        assert_eq!(n, 11);
        let mut rc = VsAIResponseCurve::new();
        rc.load(&ai, "RC_Ip5");
        assert_eq!(rc.items.len(), 9);
        assert_eq!(rc.get_value(0.5), 0.03125);
        assert_eq!(rc.get_value(-1.0), 0.0);
        assert_eq!(rc.get_value(2.0), 1.0);
        assert_eq!(rc.get_value(0.5625), (0.095367431640625 - 0.03125) * 0.5 + 0.03125);
    }
}
