//! Compartment\CompartmentSys.cpp — `CCompartment`: `.cpt` streaming units (track sectors, instances, common
//! textures). A compartment is a memory image: a 64-byte `_CPT` header, a resource table and the resources
//! themselves (models, `.vtf` textures), all addressed by offsets from the header that the game relocates in place.
//!
//! The loading/slot management lives with the compartment owner (core); this module only reads the image so the
//! renderer can register its models and textures (`CompartmentManager.cpp` resource callbacks).

use crate::renderer::renderer_types::{cstr, rd32, rdf, ModelError};

/// `('T'<<24)|('P'<<16)|('C'<<8)|('_')`
pub const CPT_ID: u32 = 0x5450_435F;

/// resource types (`m_type`, low byte of the second dword of a table entry)
pub const RESOURCE_MODEL: u8 = 0;
pub const RESOURCE_TEXTURE: u8 = 1;
/// also an XMF2 model, registered through the same callback as type 0 (used by visual instances too)
pub const RESOURCE_MODEL_ALT: u8 = 8;

/// instance list types (`CCompartment::INSTANCE_*`)
pub const INSTANCE_VISUAL: u32 = 0;
pub const INSTANCE_PHYSICS: u32 = 1;
pub const INSTANCE_COLLISION: u32 = 2;
pub const INSTANCE_IMMOVABLE: u32 = 3;
pub const INSTANCE_ALWAYS_VISIBLE: u32 = 4;
pub const INSTANCE_SIMPLE_ANIMATION: u32 = 5;
pub const INSTANCE_UNKNOWN: u32 = 6;

/// `'INS2'`: legacy instance block written by MC, ignored by the game.
pub const LEGACY_INS_ID: u32 = 0x494E_5332;

#[derive(Clone, Debug)]
pub struct CptHeader {
    pub id: u32,                // +0x00
    pub version: u32,           // +0x04 5 (3 in stale packs: everything below shifts by +4)
    pub unk08: u32,             // +0x08
    pub uncompressed_size: u32, // +0x0C
    pub bounds: [f32; 6],       // +0x10 min xyz, max xyz
    pub resource_count: u32,    // +0x28
    pub resource_table: u32,    // +0x2C
    pub instances: u32,         // +0x30 bit 31 set: instances handled by the owner's callback
}

/// resource table entry (12 bytes)
#[derive(Clone, Debug)]
pub struct CptResource<'a> {
    pub index: usize,
    pub offset: u32, // +0x00
    pub kind: u8,    // +0x04 low byte; the game stores its texture handle in the upper 24 bits
    pub hash: u32,   // +0x08
    /// texture name (texture resources only; names are packed after the header in table order)
    pub name: Option<String>,
    pub data: &'a [u8],
}

/// one placed object (80 bytes; 100 for simple animations)
#[derive(Clone, Debug)]
pub struct CptInstance {
    pub position: [f32; 3], // +0x00
    pub w: f32,             // +0x0C
    pub rotation: [f32; 4], // +0x10 quaternion x, y, z, w
    pub visual: u32,        // +0x3C resource index
    pub physical: u32,      // +0x40
    pub extra: u32,         // +0x44 -1 none
    pub id: u32,            // +0x48
    pub hash: u32,          // +0x4C
}

#[derive(Clone, Debug)]
pub struct CptInstanceList {
    pub kind: u32,
    pub instances: Vec<CptInstance>,
}

pub struct Compartment<'a> {
    pub bytes: &'a [u8],
    pub header: CptHeader,
    pub resources: Vec<CptResource<'a>>,
}

impl<'a> Compartment<'a> {
    /// CompartmentSys.cpp 0x002E7900 — header check and resource registration walk (without relocation).
    pub fn parse(b: &'a [u8]) -> Result<Self, ModelError> {
        let id = rd32(b, 0)?;
        if id != CPT_ID {
            log::error!("m_pCptHeader->m_ID == (U32)(('T'<<24)|('P'<<16)|('C'<<8)|('_'))");
            return Err(ModelError::BadId(id));
        }
        let version = rd32(b, 4)?;
        let k = if version == 3 { 4 } else { 0 };
        let mut bounds = [0f32; 6];
        for (i, v) in bounds.iter_mut().enumerate() {
            *v = rdf(b, 16 + k + 4 * i)?;
        }
        let header = CptHeader {
            id,
            version,
            unk08: rd32(b, 8)?,
            uncompressed_size: rd32(b, 12)?,
            bounds,
            resource_count: rd32(b, 40 + k)?,
            resource_table: rd32(b, 44 + k)?,
            instances: rd32(b, 48 + k)?,
        };
        let mut names = 64usize;
        let mut resources = Vec::new();
        for i in 0..header.resource_count as usize {
            let o = header.resource_table as usize + 12 * i;
            let offset = rd32(b, o)?;
            if offset == 0 {
                continue;
            }
            let kind = rd32(b, o + 4)? as u8;
            let mut name = None;
            if kind == RESOURCE_TEXTURE {
                let n = cstr(b, names)?;
                names += n.len() + 1;
                name = Some(n);
            }
            let data = b.get(offset as usize..).ok_or(ModelError::Range(offset as usize))?;
            resources.push(CptResource { index: i, offset, kind, hash: rd32(b, o + 8)?, name, data });
        }
        Ok(Self { bytes: b, header, resources })
    }

    pub fn models(&self) -> impl Iterator<Item = &CptResource<'a>> {
        self.resources.iter().filter(|r| r.kind == RESOURCE_MODEL || r.kind == RESOURCE_MODEL_ALT)
    }

    pub fn textures(&self) -> impl Iterator<Item = &CptResource<'a>> {
        self.resources.iter().filter(|r| r.kind == RESOURCE_TEXTURE)
    }

    pub fn resource(&self, index: u32) -> Option<&CptResource<'a>> {
        self.resources.iter().find(|r| r.index == index as usize)
    }

    /// CompartmentSys.cpp 0x002E7B50 — the VSL instance lists.
    pub fn instance_lists(&self) -> Result<Vec<CptInstanceList>, ModelError> {
        let mut out = Vec::new();
        let off = self.header.instances;
        if off == 0 {
            return Ok(out);
        }
        let b = self.bytes;
        let base = (off & 0x7FFF_FFFF) as usize;
        if rd32(b, base)? == LEGACY_INS_ID {
            log::debug!("Warning: Compartment file contains legacy INS format generated by MC. Use VSL files with CompC instead.");
            return Ok(out);
        }
        if (off as i32) < 0 {
            return Ok(out);
        }
        let n = rd32(b, base)?;
        if n > INSTANCE_UNKNOWN {
            log::error!("pListHeader->m_numLists <= INSTANCE_Unknown");
        }
        let mut p = base + 16;
        let mut seen = 0u32;
        for _ in 0..n {
            let kind = rd32(b, p)?;
            let count = rd32(b, p + 4)?;
            p += 16;
            if kind >= INSTANCE_UNKNOWN || seen & (1 << kind) != 0 || count > 50000 {
                log::error!("Error parsing Unknown compartment instances - unknown type found");
                break;
            }
            seen |= 1 << kind;
            let stride = if kind == INSTANCE_SIMPLE_ANIMATION { 100 } else { 80 };
            let mut instances = Vec::with_capacity(count as usize);
            for i in 0..count as usize {
                let e = p + stride * i;
                instances.push(CptInstance {
                    position: [rdf(b, e)?, rdf(b, e + 4)?, rdf(b, e + 8)?],
                    w: rdf(b, e + 12)?,
                    rotation: [rdf(b, e + 16)?, rdf(b, e + 20)?, rdf(b, e + 24)?, rdf(b, e + 28)?],
                    visual: rd32(b, e + 60)?,
                    physical: rd32(b, e + 64)?,
                    extra: rd32(b, e + 68)?,
                    id: rd32(b, e + 72)?,
                    hash: rd32(b, e + 76)?,
                });
            }
            p += stride * count as usize;
            out.push(CptInstanceList { kind, instances });
        }
        Ok(out)
    }
}

/// CompartmentSys.cpp 0x002E93E0 — chunk decompressor (RefPack-style LZ, no header). Returns bytes written.
/// The shipped packs are stored uncompressed; kept for completeness.
pub fn decompress(src: &[u8], dst: &mut Vec<u8>) -> usize {
    let start = dst.len();
    let mut s = 0usize;
    let byte = |s: &mut usize| -> u8 {
        let v = src.get(*s).copied().unwrap_or(0);
        *s += 1;
        v
    };
    fn copy_back(dst: &mut Vec<u8>, dist: usize, len: usize) {
        let from = dst.len() - dist;
        for k in 0..len {
            let v = dst[from + k];
            dst.push(v);
        }
    }
    loop {
        let c = byte(&mut s);
        if c & 0x80 == 0 {
            let b1 = byte(&mut s) as usize;
            for _ in 0..(c & 3) {
                let v = byte(&mut s);
                dst.push(v);
            }
            copy_back(dst, b1 + 1 + 8 * (c as usize & 0x60), ((c as usize & 0x1C) >> 2) + 3);
        } else if c & 0x40 == 0 {
            let b1 = byte(&mut s) as usize;
            let b2 = byte(&mut s) as usize;
            for _ in 0..(b1 >> 6) {
                let v = byte(&mut s);
                dst.push(v);
            }
            copy_back(dst, b2 + 1 + 256 * (b1 & 0x3F), (c as usize & 0x3F) + 4);
        } else if c & 0x20 == 0 {
            let b1 = byte(&mut s) as usize;
            let b2 = byte(&mut s) as usize;
            let b3 = byte(&mut s) as usize;
            for _ in 0..(c & 3) {
                let v = byte(&mut s);
                dst.push(v);
            }
            let dist = b2 + 1 + 256 * b1 + 65536 * ((c as usize & 0x10) >> 4);
            copy_back(dst, dist, (((c as usize & 0x0C) >> 2) << 8) + b3 + 5);
        } else {
            let n = 4 * (c as usize & 0x1F) + 4;
            if n > 0x70 {
                for _ in 0..(c & 3) {
                    let v = byte(&mut s);
                    dst.push(v);
                }
                break;
            }
            for _ in 0..n {
                let v = byte(&mut s);
                dst.push(v);
            }
        }
    }
    dst.len() - start
}

#[cfg(test)]
mod tests {
    use super::*;

    /// every compartment parses; every texture resource is a VTF and instance lists resolve to model resources.
    #[test]
    fn parse_every_compartment() {
        let (mut n, mut tex, mut inst) = (0, 0, 0);
        let mut bad = Vec::new();
        for p in crate::test_util::all_vcds() {
            let Ok(vcd) = visos::file::Vcd::open_host(&p, &p.to_string_lossy()) else { continue };
            for e in vcd.entries() {
                if !e.name.to_ascii_lowercase().ends_with(".cpt") {
                    continue;
                }
                let Some(d) = vcd.data(&e) else { continue };
                let c = Compartment::parse(d).unwrap();
                n += 1;
                for t in c.textures() {
                    assert!(t.data.starts_with(b"VTP_"), "{}:{} {:?}", p.display(), e.name, t.name);
                    tex += 1;
                }
                for l in c.instance_lists().unwrap() {
                    for i in &l.instances {
                        inst += 1;
                        if l.kind == INSTANCE_VISUAL {
                            match c.resource(i.visual) {
                                Some(r) if r.kind == RESOURCE_MODEL || r.kind == RESOURCE_MODEL_ALT => {}
                                // null slot: filled by the owner at load (uinstance.cpt)
                                None if i.visual < c.header.resource_count => {}
                                r => bad.push(format!("{}:{} visual {} -> {:?}", p.display(), e.name, i.visual, r.map(|r| r.kind))),
                            }
                        }
                    }
                }
            }
        }
        assert!(bad.is_empty(), "{} bad: {:#?}", bad.len(), &bad[..bad.len().min(10)]);
        assert!(n > 500);
        eprintln!("{n} compartments, {tex} textures, {inst} instances");
    }

    #[test]
    fn decompress_literals_and_matches() {
        // 4 literals "abcd", then a 2-byte match (dist 4, len 3), then end
        let src = [0xE0, b'a', b'b', b'c', b'd', 0x00, 0x03, 0xFC];
        let mut out = Vec::new();
        decompress(&src, &mut out);
        assert_eq!(&out, b"abcdabc");
    }
}
