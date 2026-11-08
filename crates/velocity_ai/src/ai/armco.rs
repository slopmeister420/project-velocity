//! Armco crash barriers: `armco01.ar` lines turned into a 4m high collision wall mesh
//! (src/Ai/Armco.cpp; the loader 0x00011090 and shutdown 0x00011780 sit below the Armco region).
//!
//! The original hands the mesh to FastCar (`FCMesh` + `FCQuadTree`, 0x003593D0 with cell 8.0 / 2.375);
//! here the mesh is plain data for the physics crate to build its collision from.

use std::path::Path;

use super::vs_math::{vs_distance, VsVector4};

/// one barrier line (8 bytes in the original: point count, type, points).
#[derive(Debug, Clone, Default)]
pub struct ArmcoLine {
    /// 0 plain, 1 skipped (no wall), 2 / 3 face winding +1 / -1
    pub kind: u16,
    /// header byte +4: the line closes on itself (an extra 100m-high post at the start)
    pub closed: bool,
    /// w = 0 for a point within 5m of the previous kept point, else 1
    pub points: Vec<VsVector4>,
}

/// the wall mesh: vertices and faces (`v0, v1, v2, material`).
#[derive(Debug, Clone, Default)]
pub struct ArmcoMesh {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[u16; 4]>,
}

/// `Armco` collision material.
pub const ARMCO_MATERIAL: u16 = 21;

#[derive(Debug, Clone, Default)]
pub struct Armco {
    pub lines: Vec<ArmcoLine>,
    pub mesh: ArmcoMesh,
}

impl Armco {
    /// reads `<level>\<name>` and builds the mesh; None when missing or malformed.
    /// Armco.cpp 0x00011090
    pub fn load(level: &Path, name: &str) -> Option<Self> {
        log::info!("-- Loading armco barriers.");
        let Ok(d) = std::fs::read(crate::resolve(level, name)) else {
            log::error!("    error opening armco file.");
            return None;
        };
        Self::parse(&d)
    }

    /// the file walk of 0x00011090 plus the mesh build.
    pub fn parse(d: &[u8]) -> Option<Self> {
        if d.len() < 8 || &d[..4] != b"SPL0" {
            log::error!("    armco file format appears to be wrong.");
            return None;
        }
        let u32_at = |p: usize| d.get(p..p + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
        let f32_at = |p: usize| u32_at(p).map(f32::from_bits);
        let count = u32_at(4)?;
        let mut a = Armco::default();
        if count == 0 {
            log::info!("    armco file appears to have no armco sections");
            return None;
        }
        let mut p = 16;
        for _ in 0..count {
            let closed = *d.get(p + 4)? != 0;
            let kind = *d.get(p + 5)? as u16;
            let n = u32_at(p + 8)?;
            p += 16;
            let mut line = ArmcoLine { kind, closed, points: Vec::with_capacity(n as usize) };
            let mut last: Option<VsVector4> = None;
            for _ in 0..n {
                let mut v = VsVector4::new(f32_at(p)?, f32_at(p + 4)?, f32_at(p + 8)?, f32_at(p + 12)?);
                if let Some(l) = last
                    && vs_distance(&l, &v) < 5.0
                {
                    v.w = 0.0;
                } else {
                    v.w = 1.0;
                    last = Some(v);
                }
                line.points.push(v);
                p += 16;
            }
            a.lines.push(line);
        }
        if p != d.len() {
            log::error!("    bytes read from armco file does not match the size of the buffer.  File corrupt?");
            return None;
        }
        a.build_mesh();
        Some(a)
    }

    /// every point twice (ground and 4m up), plus a 100m post at the start of closed lines, then a
    /// triangle strip per line.
    /// Armco.cpp 0x00012440
    pub fn build_mesh(&mut self) {
        let mut nverts = 0usize;
        let mut nlines = 0usize;
        for l in &self.lines {
            if l.kind != 1 {
                nverts += 2 * l.points.len();
                if l.closed {
                    nverts += 2;
                }
                nlines += 1;
            }
        }
        let m = &mut self.mesh;
        m.vertices = Vec::with_capacity(nverts);
        for l in &self.lines {
            if l.kind == 1 {
                continue;
            }
            for p in &l.points {
                m.vertices.push([p.x, p.y, p.z]);
                m.vertices.push([p.x, p.y + 4.0, p.z]);
            }
            if l.closed {
                let p = l.points[0];
                m.vertices.push([p.x, p.y, p.z]);
                m.vertices.push([p.x, p.y + f32::from_bits(0x42C8_6666), p.z]);
            }
        }
        debug_assert_eq!(m.vertices.len(), nverts, "mpMesh->number_of_vertices == (int)dNumVerts");
        m.faces = Vec::with_capacity(nverts - 2 * nlines);
        let mut v = 0usize;
        for l in &self.lines {
            if l.kind == 1 {
                continue;
            }
            let winding: i32 = match l.kind {
                0 => 0,
                2 => 1,
                3 => -1,
                _ => {
                    debug_assert!(false);
                    0
                }
            };
            let mut flip = winding >= 0;
            let n = if l.closed { 2 * l.points.len() } else { 2 * l.points.len() - 2 };
            for _ in 0..n {
                let tri = [v, v + 1, v + 2].map(|i| first_vertex(&m.vertices, m.vertices[i]));
                let face = if flip { [tri[0], tri[1], tri[2], ARMCO_MATERIAL] } else { [tri[0], tri[2], tri[1], ARMCO_MATERIAL] };
                m.faces.push(face);
                flip = !flip;
                v += 1;
            }
            v += 2;
        }
    }
}

/// index of the first vertex with these exact coordinates (CircuitEditorTools.cpp 0x0007F7A0), so
/// repeated points share a vertex.
pub(crate) fn first_vertex(verts: &[[f32; 3]], p: [f32; 3]) -> u16 {
    let i = verts.iter().position(|v| v[0] == p[0] && v[1] == p[1] && v[2] == p[2]);
    debug_assert!(i.is_some(), "bFound");
    i.unwrap_or(verts.len()) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_armco() {
        let Some(dir) = crate::game_dir() else { return };
        let mut n = 0;
        let mut stack = vec![dir.join("Tracks")];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("ar")) {
                    let d = std::fs::read(&p).unwrap();
                    let a = Armco::parse(&d).unwrap_or_else(|| panic!("{}", p.display()));
                    let lines = a.lines.iter().filter(|l| l.kind != 1).count();
                    assert_eq!(a.mesh.faces.len(), a.mesh.vertices.len() - 2 * lines, "{}", p.display());
                    assert!(a.mesh.faces.iter().all(|f| (f[0].max(f[1]).max(f[2]) as usize) < a.mesh.vertices.len()));
                    n += 1;
                }
            }
        }
        assert_eq!(n, 21);
    }
}
