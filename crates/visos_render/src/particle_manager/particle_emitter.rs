//! src\ParticleManager\ParticleEmitter.cpp — placed particle data (`particle.pld`) and the definition loading of
//! `CParticleManager` (generic `\Particle\PARTDEF.VCD`, then the track's own `PARTDEF.vcd`).

use bevy::math::{Mat4, Quat, Vec3};

/// `'PLDDATA'` + version byte 3 (0x000FF060 refuses other versions)
pub const PLD_MAGIC: &[u8; 7] = b"PLDDATA";
pub const PLD_VERSION: u8 = 3;

/// one placed emitter, 48 bytes
#[derive(Clone, Debug, PartialEq)]
pub struct PldInstance {
    /// +0 position, +16 orientation (applied through its conjugate, Renderer 0x002AC2C0)
    pub position: Vec3,
    pub rotation: Quat,
    /// +32 emitter type name crc, +36 instance id, +40 group crc
    pub type_crc: u32,
    pub id: u32,
    pub group: u32,
    /// +44 manager (0 = stateless; 1 is accepted and does nothing)
    pub manager: i8,
    /// +45 running (from the file), +46 its copy taken at load and restored on reset (0x000FE630)
    pub enabled: bool,
    pub enabled_at_load: bool,
}

impl PldInstance {
    /// AC::Renderer(quat) + translation (0x001007E0)
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation.conjugate(), self.position)
    }
}

/// ParticleEmitter.cpp 0x000FF060: header 16 bytes (magic, version, u32 count), then records
pub fn parse_pld(d: &[u8]) -> Option<Vec<PldInstance>> {
    if d.len() < 16 || &d[0..7] != PLD_MAGIC || d[7] != PLD_VERSION {
        return None;
    }
    let n = u32::from_le_bytes(d[8..12].try_into().ok()?) as usize;
    let f = |o: usize| f32::from_le_bytes(d[o..o + 4].try_into().unwrap());
    let u = |o: usize| u32::from_le_bytes(d[o..o + 4].try_into().unwrap());
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let o = 16 + 48 * i;
        if d.len() < o + 48 {
            return None;
        }
        out.push(PldInstance {
            position: Vec3::new(f(o), f(o + 4), f(o + 8)),
            rotation: Quat::from_xyzw(f(o + 16), f(o + 20), f(o + 24), f(o + 28)),
            type_crc: u(o + 32),
            id: u(o + 36),
            group: u(o + 40),
            manager: d[o + 44] as i8,
            enabled: d[o + 45] != 0,
            enabled_at_load: d[o + 45] != 0,
        });
    }
    Some(out)
}

/// ParticleEmitter.cpp 0x000FEAE0 / 0x000FED00: every `.slp` in a PARTDEF pack, in toc order (`.sfp` files are counted, not loaded)
pub fn partdef_files(path: &str) -> Vec<Vec<u8>> {
    let Ok(v) = visos::file::Vcd::open(path) else {
        log::warn!("ERROR: No particle definitions ({path}) present");
        return Vec::new();
    };
    v.entries()
        .filter(|e| e.name.to_ascii_lowercase().ends_with(".slp"))
        .filter_map(|e| v.data(&e).map(<[u8]>::to_vec))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// every shipped version-3 particle.pld parses exactly; LACoast's version-1 file is refused like the original
    #[test]
    fn parse_all_pld() {
        fn walk(p: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(p).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.file_name().is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("particle.pld")) {
                    out.push(p);
                }
            }
        }
        let mut files = Vec::new();
        walk(visos::file::game_dir(), &mut files);
        for f in &files {
            let d = std::fs::read(f).unwrap();
            if d[7] != PLD_VERSION {
                assert!(parse_pld(&d).is_none());
                continue;
            }
            let v = parse_pld(&d).unwrap_or_else(|| panic!("{f:?}"));
            assert_eq!(d.len(), 16 + 48 * v.len(), "{f:?}");
        }
        assert!(files.len() > 50);
    }
}
