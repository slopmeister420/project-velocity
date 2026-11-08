//! StatelessParticles\StatelessParticleType.cpp — one emitter type, a 256-byte record (260 from file version
//! 0x6B) of `.slp` files (`PARTDEF.VCD`) and `\Particle\PartFX.bin`. Particles have no state: every frame each
//! one is re-derived from the emitter's seed and age (see `stateless_particle_render_inst`).

use bevy::math::Vec4;

/// file version from which records carry the sort key (+256) and flags are a bit set
pub const VERSION_SORT_KEY: u32 = 0x6B;
/// below this version colour components above 128 are clamped (0x002D4F90)
pub const VERSION_COLOUR_CLAMP: u32 = 0x6A;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CType {
    /// +0 `m_TypeName` (32 bytes)
    pub name: String,
    /// +32 `m_TypeNameCRC`: case-sensitive crc32 of the name, recomputed at load
    pub name_crc: u32,
    /// +36 particles per emitter
    pub num_particles: u32,
    /// +40 alpha scale of the first copy, +44 size scale of the first copy
    pub copy_alpha: f32,
    pub copy_size: f32,
    /// +48 copies drawn (trails), each +52 seconds older, +56 smaller and +60 more transparent
    pub num_copies: u32,
    pub copy_time_step: f32,
    pub copy_size_step: f32,
    pub copy_alpha_step: f32,
    /// +64 / +68 birth size range, +72 size at death
    pub size_min: f32,
    pub size_max: f32,
    pub size_end: f32,
    /// +76 / +92 birth colour range, +108 colour at death (rgba, 0..128 = 0..1)
    pub colour_min: Vec4,
    pub colour_max: Vec4,
    pub colour_end: Vec4,
    /// +124 / +128 spin speed range (radians per second)
    pub rot_min: f32,
    pub rot_max: f32,
    /// +132 spawn box (centred), +148 / +164 velocity range, +180 acceleration
    pub box_size: Vec4,
    pub vel_min: Vec4,
    pub vel_max: Vec4,
    pub accel: Vec4,
    /// +196 emission window: particles are born over this many seconds
    pub emit_time: f32,
    /// +200 particle life, +204 emitter duration (< 0 = endless)
    pub life: f32,
    pub duration: f32,
    /// +208 animation frames (1, 2, 4, 8 or 16), +212 1 = frames run over the life, +216 cycles per life
    pub num_frames: u32,
    pub anim_mode: u32,
    pub anim_rate: u32,
    /// +220 texture name (31 bytes) in PARTTEX.VCD, empty = white
    pub texture: String,
    /// +251 flags: 1 ?, 2 no random start angle, 4 billboard around the emitter's y axis
    pub flags: u8,
    /// +252 blend mode (ShaderManager blend table)
    pub blend_mode: u32,
    /// +256 sort key (high byte of the draw sort key)
    pub sort_key: u32,
}

fn f32_at(r: &[u8], o: usize) -> f32 {
    f32::from_le_bytes(r[o..o + 4].try_into().unwrap())
}
fn u32_at(r: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(r[o..o + 4].try_into().unwrap())
}
fn v4_at(r: &[u8], o: usize) -> Vec4 {
    Vec4::new(f32_at(r, o), f32_at(r, o + 4), f32_at(r, o + 8), f32_at(r, o + 12))
}
fn cstr(b: &[u8]) -> String {
    let n = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..n]).into_owned()
}

/// StatelessParticleMgr.cpp 0x002D4F00 (original bug kept: y over 128 clamps z, z over 128 clamps y)
fn clamp_colour(c: &mut Vec4) {
    if c.x > 128.0 {
        c.x = 128.0;
    }
    if c.y > 128.0 {
        c.z = 128.0;
    }
    if c.z > 128.0 {
        c.y = 128.0;
    }
    if c.w > 128.0 {
        c.w = 128.0;
    }
}

impl CType {
    /// record size for a file version (0x002D4470: `256 + 4 * (version >= 107)`)
    pub fn record_size(version: u32) -> usize {
        256 + 4 * (version >= VERSION_SORT_KEY) as usize
    }

    /// StatelessParticleType.cpp 0x002D62D0 (load) + StatelessParticleMgr.cpp 0x002D4F90 (version fix-ups)
    pub fn parse(r: &[u8], version: u32) -> Option<Self> {
        if r.len() < 256 {
            return None;
        }
        let name = cstr(&r[0..32]);
        let mut t = Self {
            name_crc: visos::handy::handy_crc::crc32(name.as_bytes(), 0),
            name,
            num_particles: u32_at(r, 36),
            copy_alpha: f32_at(r, 40),
            copy_size: f32_at(r, 44),
            num_copies: u32_at(r, 48),
            copy_time_step: f32_at(r, 52),
            copy_size_step: f32_at(r, 56),
            copy_alpha_step: f32_at(r, 60),
            size_min: f32_at(r, 64),
            size_max: f32_at(r, 68),
            size_end: f32_at(r, 72),
            colour_min: v4_at(r, 76),
            colour_max: v4_at(r, 92),
            colour_end: v4_at(r, 108),
            rot_min: f32_at(r, 124),
            rot_max: f32_at(r, 128),
            box_size: v4_at(r, 132),
            vel_min: v4_at(r, 148),
            vel_max: v4_at(r, 164),
            accel: v4_at(r, 180),
            emit_time: f32_at(r, 196),
            life: f32_at(r, 200),
            duration: f32_at(r, 204),
            num_frames: u32_at(r, 208),
            anim_mode: u32_at(r, 212),
            anim_rate: u32_at(r, 216),
            texture: cstr(&r[220..251]),
            flags: r[251],
            blend_mode: u32_at(r, 252),
            sort_key: if version < VERSION_SORT_KEY { 0 } else { r.get(256..260).map(|b| u32::from_le_bytes(b.try_into().unwrap())).unwrap_or(0) },
        };
        if version < VERSION_COLOUR_CLAMP {
            clamp_colour(&mut t.colour_min);
            clamp_colour(&mut t.colour_max);
            clamp_colour(&mut t.colour_end);
        }
        if version < VERSION_SORT_KEY {
            t.flags = (t.flags != 0) as u8;
        }
        Some(t)
    }

    /// StatelessParticleType.cpp 0x002D7130: endless emitters run for 100000 seconds
    pub fn duration(&self) -> f32 {
        if self.duration >= 0.0 {
            self.duration
        } else {
            100000.0
        }
    }

    /// StatelessParticleType.cpp 0x002D7D60
    pub fn is_endless(&self) -> bool {
        self.duration < 0.0
    }

    /// StatelessParticleType.cpp 0x002D7300: the duration rounded up to whole lives, plus the emission window
    pub fn total_time(&self) -> f32 {
        round_up(self.duration(), self.life) + self.emit_time
    }

    /// StatelessParticleType.cpp 0x002D5EC0: frames across the texture
    pub fn frames_u(&self) -> u32 {
        match self.num_frames {
            2 | 4 => 2,
            8 | 16 => 4,
            _ => 1,
        }
    }

    /// StatelessParticleType.cpp 0x002D5FD0: frames down the texture
    pub fn frames_v(&self) -> u32 {
        match self.num_frames {
            4 | 8 => 2,
            16 => 4,
            _ => 1,
        }
    }

    /// StatelessParticleType.cpp 0x002D7280
    pub fn random_start_angle(&self) -> bool {
        self.flags & 2 == 0
    }

    /// StatelessParticleType.cpp 0x002D72E0
    pub fn axis_billboard(&self) -> bool {
        self.flags & 4 != 0
    }

    /// StatelessParticleMgr.cpp 0x002D3680: bounding sphere of one emitter of this type in emitter space (centre xyz,
    /// radius w) — per axis the extremes of `vel * t + accel * t * t / 2` over the life (0x002D3A80) for both velocity
    /// bounds, widened to include the origin and by half the spawn box, plus the largest size over time
    pub fn bounding_sphere(&self, emitter_scale: f32) -> Vec4 {
        let (mut lo, mut hi) = ([1.0e8_f32; 3], [-1.0e8_f32; 3]);
        let (acc, vmin, vmax, life) = (self.accel.to_array(), self.vel_min.to_array(), self.vel_max.to_array(), self.life);
        for k in 0..3 {
            axis_extent(acc[k], vmin[k], life, &mut lo[k], &mut hi[k]);
            axis_extent(acc[k], vmax[k], life, &mut lo[k], &mut hi[k]);
        }
        let half = self.box_size.to_array().map(|v| v.abs() * 0.5);
        let (mut centre, mut d2) = ([0.0_f32; 3], 0.0_f32);
        for k in 0..3 {
            lo[k] = lo[k].min(0.0);
            hi[k] = hi[k].max(0.0);
            lo[k] -= half[k];
            hi[k] += half[k];
            centre[k] = (hi[k] + lo[k]) * 0.5;
            d2 += (hi[k] - centre[k]) * (hi[k] - centre[k]);
        }
        let mut r = if d2 >= 1.0 { d2.sqrt() } else { 1.0 };
        let size = if self.size_end >= self.size_max { self.size_end } else { self.size_max };
        r += self.copy_size * size * emitter_scale;
        Vec4::new(centre[0], centre[1], centre[2], r)
    }

    /// StatelessParticleType.cpp 0x002D6F10 / 0x002D6F70 / 0x002D6FD0: colour as `VISOS_ColourXBOX` bytes (b, g, r, a), each `min(2 * c, 255)`
    pub fn colour_bytes(c: Vec4) -> [u8; 4] {
        let b = |v: f32| (2 * (v as i32 as u8 as i32)).min(255) as u8;
        [b(c.z), b(c.y), b(c.x), b(c.w)]
    }
}

/// StatelessParticleMgr.cpp 0x002D3A80: widen `[lo, hi]` by the position at the end of the life (`v * t + a * t * t / 2`)
/// and, when the axis turns round inside the life (0x002D3BB0), by the position at the turning point
pub fn axis_extent(a: f32, v: f32, life: f32, lo: &mut f32, hi: &mut f32) {
    let half = life * 0.5;
    let end = v * life + a * half * life;
    if end < *lo {
        *lo = end;
    }
    if end > *hi {
        *hi = end;
    }
    if a >= 0.000001 {
        // uninitialised out value when 0x002D3BB0 returns true without writing is taken as 0
        let t = solve_turn(0.5 * a, v, life * 100.0).map(|t| (t * 0.5).abs());
        if let Some(t) = t.filter(|t| *t <= life) {
            let p = v * t + a * (t * 0.5) * t;
            if p < *lo {
                *lo = p;
            }
            if p > *hi {
                *hi = p;
            }
        }
    }
}

/// StatelessParticleMgr.cpp 0x002D3BB0 (`Some(t)` = returned true; 0 stands for the unwritten out value)
fn solve_turn(a1: f32, a2: f32, a3: f32) -> Option<f32> {
    let v5 = if a2 > 0.0 { -a2 } else { 0.0 };
    let ok = if a1 < 0.0 { a3 * v5 <= a1 } else { a3 * v5 >= a1 };
    if ok {
        Some(v5 / a1)
    } else if v5 < 0.0 {
        (a3 <= 0.0).then_some(0.0)
    } else if a3 >= 0.0 {
        (v5 >= 1.0).then_some(0.0)
    } else {
        None
    }
}

/// StatelessParticleRenderInst.cpp 0x00322680: smallest multiple of `step` not below `v` — `_ftol(ceil(v / step))`
/// (0x000863B0 -> CRT 0x003E2A80 = ceil, control word 0x1B3F; it was ported as a truncation, which ended every
/// one-shot shorter than one life at once)
pub fn round_up(v: f32, step: f32) -> f32 {
    (v / step).ceil() as i32 as f32 * step
}

/// StatelessParticleMgr.cpp 0x002D4470: a `.slp` / PartFX.bin file = u32 version, u32 count, records
pub fn parse_file(bytes: &[u8]) -> Option<(u32, Vec<CType>)> {
    let version = u32::from_le_bytes(bytes.get(0..4)?.try_into().ok()?);
    let count = u32::from_le_bytes(bytes.get(4..8)?.try_into().ok()?) as usize;
    let size = CType::record_size(version);
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let o = 8 + i * size;
        out.push(CType::parse(bytes.get(o..o + size)?, version)?);
    }
    Some((version, out))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// every shipped .slp and PartFX.bin parses to whole records with sane fields
    #[test]
    fn parse_all_particle_types() {
        let mut files = 0;
        let mut types = 0;
        let mut check = |name: &str, d: &[u8]| {
            let (v, t) = parse_file(d).unwrap_or_else(|| panic!("{name}: parse"));
            assert_eq!(d.len(), 8 + t.len() * CType::record_size(v), "{name}: size");
            for x in &t {
                assert!(matches!(x.num_frames, 1 | 2 | 4 | 8 | 16), "{name} {}: frames {}", x.name, x.num_frames);
                assert!(x.life > 0.0, "{name} {}: life", x.name);
            }
            files += 1;
            types += t.len();
        };
        if let Ok(d) = visos::file::platform::read("\\Particle\\PartFX.bin") {
            check("PartFX.bin", &d);
        }
        for p in crate::test_util::all_vcds() {
            if !p.file_name().is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("partdef.vcd")) {
                continue;
            }
            let v = visos::file::Vcd::open_host(&p, &p.to_string_lossy()).unwrap();
            for e in v.entries() {
                if e.name.to_ascii_lowercase().ends_with(".slp") {
                    check(e.name, v.data(&e).unwrap());
                }
            }
        }
        assert!(files > 200 && types > files, "{files} files {types} types");
    }
}
