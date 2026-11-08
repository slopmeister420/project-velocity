//! VertexAnimation\VertexAnim.cpp / VertexModel.cpp / VertexAnimBlendTree(Layer).cpp — morph-target vertex
//! animation. `.gvf` anims ('GVF0') hold per frame one byte weight per target; `.gvs` ('GVS0') lists anim names; the
//! targets themselves (a "vertex model": base positions + s16 deltas per target) hang off the XMF at +0x34.
//!
//! Shipped data: `\Chars\CHARS_A.VCD` has DJ_Select.gvf / DJ_Select2.gvf / CharacterAnims.gvs, but **no shipped
//! XMF carries a vertex model** (+0x34 is 0 in every CHARS_M / FG_M model, test `no_shipped_vertex_model`), so the
//! blend tree has nothing to deform: Characters.cpp 0x00079EC0 only runs it for a front-end character element with a
//! vertex anim (+448 slot) and the checksum assert (VertexAnimBlendTree.cpp 360) would fire. Ported as data + maths
//! (sampling, layer blend, active-target limit, delta apply) with no render hook.

/// `'GVF0'`
pub const GVF_MAGIC: &[u8; 4] = b"0FVG";
/// `'GVS0'`
pub const GVS_MAGIC: &[u8; 4] = b"0SVG";
/// 0x002D8370 / 0x002D8C10: weights at or below this are off
pub const WEIGHT_EPSILON: f32 = 0.0001;
/// 0x002D8C10: delta units (`0.00024414062` = 1/4096)
pub const DELTA_SCALE: f32 = 0.000_244_140_62;

fn u32_at(d: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(d.get(o..o + 4)?.try_into().ok()?))
}

fn f32_at(d: &[u8], o: usize) -> Option<f32> {
    Some(f32::from_bits(u32_at(d, o)?))
}

fn cstr(d: &[u8], o: usize) -> Option<String> {
    let s = d.get(o..)?;
    let n = s.iter().position(|&b| b == 0).unwrap_or(s.len());
    Some(String::from_utf8_lossy(&s[..n]).into_owned())
}

/// one target of an anim (12 B: +0 name, +4 active, +8 unused)
#[derive(Clone, Debug, PartialEq)]
pub struct AnimTarget {
    pub name: String,
    pub active: bool,
}

/// one vertex anim (info: +0 name[128], +128 checksum, +132 fps, +136 duration, +140 frames, +144 loop, +152 target
/// count, +156 targets, +160 frame count, +164 frame pointers -> one weight byte per target)
#[derive(Clone, Debug, PartialEq)]
pub struct VertexAnim {
    pub name: String,
    pub checksum: u32,
    pub fps: f32,
    pub duration: f32,
    pub frames: u32,
    pub looping: u32,
    pub targets: Vec<AnimTarget>,
    pub weights: Vec<Vec<u8>>,
}

impl VertexAnim {
    /// a `.gvf` file: 'GVF0', +8 size, +0x10 count, +0x14 absolute offsets of the infos (relocated in place)
    pub fn parse_file(d: &[u8]) -> Option<Vec<VertexAnim>> {
        if d.get(0..4)? != GVF_MAGIC {
            return None;
        }
        let n = u32_at(d, 0x10)? as usize;
        (0..n).map(|i| Self::parse_info(d, u32_at(d, 0x14 + 4 * i)? as usize)).collect()
    }

    fn parse_info(d: &[u8], o: usize) -> Option<VertexAnim> {
        let nt = u32_at(d, o + 152)? as usize;
        let tt = u32_at(d, o + 156)? as usize;
        let targets = (0..nt)
            .map(|i| Some(AnimTarget { name: cstr(d, u32_at(d, tt + 12 * i)? as usize)?, active: u32_at(d, tt + 12 * i + 4)? != 0 }))
            .collect::<Option<Vec<_>>>()?;
        let nf = u32_at(d, o + 160)? as usize;
        let ft = u32_at(d, o + 164)? as usize;
        let weights = (0..nf)
            .map(|k| {
                let p = u32_at(d, ft + 4 * k)? as usize;
                Some(d.get(p..p + nt)?.to_vec())
            })
            .collect::<Option<Vec<_>>>()?;
        Some(VertexAnim {
            name: cstr(d, o)?,
            checksum: u32_at(d, o + 128)?,
            fps: f32_at(d, o + 132)?,
            duration: f32_at(d, o + 136)?,
            frames: u32_at(d, o + 140)?,
            looping: u32_at(d, o + 144)?,
            targets,
            weights,
        })
    }

    /// VertexAnim.cpp 0x002E2230 — normalised time step: dt / duration, fractional part (`_ftol` truncation)
    pub fn frame_inc(&self, dt: f32) -> f32 {
        let v = dt / self.duration;
        v - (v as i32) as f32
    }

    /// VertexAnim.cpp 0x002E2390 — weight of `target` at normalised time `t`: frame = n·t looping else (n − 1)·t
    /// (t ≤ 0.99999), lerp of the two neighbouring frames' bytes / 255; the last frame wraps to 0 when looping, else
    /// holds
    pub fn sample(&self, t: f32, target: usize) -> f32 {
        let n = self.frames as i32;
        let f = if self.looping == 1 { n as f32 * t.min(0.99999) } else { (n - 1) as f32 * t.min(0.99999) };
        let a = f as i32;
        let mut b = a + 1;
        let mut frac = f - a as f32;
        if b >= n {
            if self.looping == 1 {
                b = 0;
            } else {
                b = a;
                frac = 0.0;
            }
        }
        let w = |k: i32| self.weights.get(k as usize).and_then(|f| f.get(target)).copied().unwrap_or(0) as f32;
        let v0 = w(a) / 255.0;
        (w(b) / 255.0 - v0) * frac + v0
    }
}

/// a `.gvs` anim set: 'GVS0', +0x10 name[128], +0x90 count, +0x94 offset of 64-byte anim names
pub fn parse_anim_set(d: &[u8]) -> Option<(String, Vec<String>)> {
    if d.get(0..4)? != GVS_MAGIC {
        return None;
    }
    let n = u32_at(d, 0x90)? as usize;
    let o = u32_at(d, 0x94)? as usize;
    Some((cstr(d, 0x10)?, (0..n).map(|i| cstr(d, o + 64 * i)).collect::<Option<Vec<_>>>()?))
}

/// one target of a vertex model (20 B: +0 name, +4 delta count, +8 index count, +12 deltas {s16 x, y, z, vertex
/// count}, +16 u16 vertex indices)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VertexTarget {
    pub name: String,
    pub deltas: Vec<[i16; 4]>,
    pub indices: Vec<u16>,
}

/// a blend layer (VertexAnimBlendTreeLayer): mode 0 = an anim at a normalised time, mode 1 = one target by name
pub enum VertexLayer<'a> {
    Anim { anim: &'a VertexAnim, time: f32, weight: f32 },
    Target { name: &'a str, weight: f32 },
}

/// VertexAnimBlendTree.cpp 0x002D8540 (UpdateLeaf) — the targets' weights: zeroed, then each layer in order lerps
/// them by its weight: anim layers towards the anim's sample for its active targets, target layers towards 1
/// (case-insensitive name match; an unknown name stops the update, as the original returns)
pub fn update_leaf(targets: &[VertexTarget], layers: &[VertexLayer]) -> Vec<f32> {
    let mut w = vec![0.0f32; targets.len()];
    for l in layers {
        match l {
            VertexLayer::Anim { anim, time, weight } => {
                for i in 0..anim.targets.len() {
                    if anim.targets[i].active && i < w.len() {
                        let s = anim.sample(*time, i);
                        w[i] = (s - w[i]) * weight + w[i];
                    }
                }
            }
            VertexLayer::Target { name, weight } => {
                let Some(j) = targets.iter().position(|t| t.name.eq_ignore_ascii_case(name)) else { return w };
                w[j] = (1.0 - w[j]) * weight + w[j];
            }
        }
    }
    w
}

/// VertexAnimBlendTree.cpp 0x002D8370 tail — at most `max` targets stay on: the smallest weight above the epsilon
/// is zeroed (the first of equals) until the count fits; `max` ≤ 0 = no limit
pub fn limit_active(w: &mut [f32], max: i32) {
    if max <= 0 {
        return;
    }
    loop {
        let mut n = 0;
        let mut min = 999_999.0f32;
        let mut at = 0;
        for (i, &v) in w.iter().enumerate() {
            if v > WEIGHT_EPSILON {
                if v < min {
                    min = v;
                    at = i;
                }
                n += 1;
            }
        }
        if n > max {
            w[at] = 0.0;
            n -= 1;
        }
        if n <= max {
            break;
        }
    }
}

/// VertexAnimBlendTree.cpp 0x002D8C10 — positions = base, then per target above the epsilon each delta record
/// (x, y, z)·w/4096 is added to its `count` vertices from the index stream
pub fn apply(base: &[[f32; 4]], targets: &[VertexTarget], w: &[f32]) -> Vec<[f32; 4]> {
    let mut out = base.to_vec();
    for (t, &wt) in targets.iter().zip(w) {
        if wt <= WEIGHT_EPSILON {
            continue;
        }
        let s = wt * DELTA_SCALE;
        let mut idx = t.indices.iter();
        for d in &t.deltas {
            let v = [d[0] as f32 * s, d[1] as f32 * s, d[2] as f32 * s];
            for _ in 0..(d[3] as u16) {
                let Some(&k) = idx.next() else { break };
                if let Some(p) = out.get_mut(k as usize) {
                    p[0] += v[0];
                    p[1] += v[1];
                    p[2] += v[2];
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(name: &str) -> Vec<(String, Vec<u8>)> {
        let v = visos::file::Vcd::open(name).unwrap();
        v.entries().map(|e| (e.name.to_string(), v.data(&e).unwrap().to_vec())).collect()
    }

    /// every shipped .gvf parses and samples in range; the .gvs names resolve to them
    #[test]
    fn shipped_vertex_anims() {
        let files = pack(r"\Chars\CHARS_A.VCD");
        let mut names = Vec::new();
        for (n, d) in &files {
            if !n.to_ascii_lowercase().ends_with(".gvf") {
                continue;
            }
            let anims = VertexAnim::parse_file(d).unwrap_or_else(|| panic!("{n}"));
            for a in anims {
                assert_eq!(a.weights.len() as u32, a.frames, "{n}");
                assert!(a.duration > 0.0 && (a.fps * a.duration - a.frames as f32).abs() < 0.01, "{n}");
                for i in 0..a.targets.len() {
                    for k in 0..=20 {
                        let w = a.sample(k as f32 / 20.0, i);
                        assert!((0.0..=1.0).contains(&w), "{n}");
                    }
                }
                names.push(a.name.to_ascii_lowercase());
            }
        }
        assert_eq!(names.len(), 2);
        let (_, d) = files.iter().find(|(n, _)| n.to_ascii_lowercase().ends_with(".gvs")).unwrap();
        let (_, set) = parse_anim_set(d).unwrap();
        assert_eq!(set.len(), 2);
        assert!(set.iter().all(|s| names.contains(&s.to_ascii_lowercase())));
    }

    /// no shipped character model carries a vertex model (+0x34), so the blend tree never has targets
    #[test]
    fn no_shipped_vertex_model() {
        for p in [r"\Chars\CHARS_M.VCD", r"\FlagGirl\FG_M.vcd"] {
            for (n, d) in pack(p) {
                if d.len() >= 0x38 && &d[0..4] == b"XMF2" {
                    assert_eq!(u32_at(&d, 0x34), Some(0), "{p} {n}");
                }
            }
        }
    }

    #[test]
    fn blend_limit_apply() {
        let anim = VertexAnim {
            name: "a".into(),
            checksum: 0,
            fps: 1.0,
            duration: 2.0,
            frames: 2,
            looping: 0,
            targets: vec![AnimTarget { name: "t0".into(), active: true }, AnimTarget { name: "t1".into(), active: false }],
            weights: vec![vec![0, 255], vec![255, 255]],
        };
        assert!((anim.sample(0.5, 0) - 0.5).abs() < 1e-6);
        assert!((anim.frame_inc(3.0) - 0.5).abs() < 1e-6);
        let targets = vec![
            VertexTarget { name: "t0".into(), deltas: vec![[4096, 0, 0, 2]], indices: vec![0, 2] },
            VertexTarget { name: "T1".into(), deltas: vec![[0, 8192, 0, 1]], indices: vec![1] },
        ];
        let w = update_leaf(&targets, &[VertexLayer::Anim { anim: &anim, time: 0.5, weight: 1.0 }, VertexLayer::Target { name: "t1", weight: 0.25 }]);
        assert_eq!(w, vec![0.5, 0.25]);
        let mut lim = w.clone();
        limit_active(&mut lim, 1);
        assert_eq!(lim, vec![0.5, 0.0]);
        let p = apply(&[[0.0; 4]; 3], &targets, &w);
        assert!((p[0][0] - 0.5).abs() < 1e-6 && (p[2][0] - 0.5).abs() < 1e-6 && (p[1][1] - 0.5).abs() < 1e-6);
    }
}
