//! Instances\file_instances.cpp — Instances chunks (placed, typed objects in bundles).
//!
//! all offsets relative to the Instances chunk data:
//! ```text
//! +0x00 u32 version (1)          +0x04 u32 ?
//! +0x08 u32 typeRefCount         +0x0C u32 -> FileTypeRef[] (12 bytes)
//! +0x10 u32 linkCount            +0x14 u32 -> FileInstanceLink[] (16 bytes)
//! +0x18 u32 instanceCount        +0x1C u32 -> FileInstance[] (96 bytes)
//! FileInstance: +0 nameCrc, +4 name, +8 parent name, +0x0C u32 (1), +0x10 VsMatrix world (row-major,
//!   row3 = position), +0x50 -> FileTypeRef, +0x54 -> instance data, +0x58 groupCrc, +0x5C group name
//! ```

use std::sync::Arc;

use crate::chunk::file_chunk::{cstr, rd32};
use crate::type_::file_type::{read_type_ref, FileTypeRef};
use crate::type_::vs_type::{decode, CEnvironment, CType, Value};

/// `FILE_INSTANCES_VERSION` (file_tlf.cpp 0x002EE230 checks == 1)
pub const FILE_INSTANCES_VERSION: u32 = 1;

#[derive(Clone, Debug)]
pub struct FileInstanceLink {
    pub crc: u32,
    pub a: Option<String>, // +4
    pub b: Option<String>, // +8
    pub d: u32,            // +12
}

#[derive(Clone, Debug)]
pub struct FileInstance {
    pub name_crc: u32,
    pub name: Option<String>,
    pub parent: Option<String>,
    pub flags: u32,
    /// row-major 4x4, translation in m[3][0..3]
    pub world: [[f32; 4]; 4],
    pub type_ref: Option<FileTypeRef>,
    pub data_offset: u32,
    pub group_crc: u32,
    pub group: Option<String>,
    /// decoded instance data (`type_ref.ty` applied at `data_offset`)
    pub value: Option<Value>,
}

impl FileInstance {
    pub fn type_name(&self) -> Option<&str> {
        self.type_ref.as_ref().map(|t| t.name.as_str())
    }
    pub fn ty(&self) -> Option<&Arc<CType>> {
        self.type_ref.as_ref().and_then(|t| t.ty.as_ref())
    }
    pub fn position(&self) -> [f32; 3] {
        [self.world[3][0], self.world[3][1], self.world[3][2]]
    }
}

#[derive(Clone, Debug)]
pub struct FileInstances {
    pub version: u32,
    pub type_refs: Vec<FileTypeRef>,
    pub links: Vec<FileInstanceLink>,
    pub instances: Vec<FileInstance>,
}

#[derive(Debug, thiserror::Error)]
pub enum InstancesError {
    #[error("Invalid instances chunk version {0}")]
    Version(u32),
}

pub(crate) fn read_matrix(b: &[u8], at: usize) -> [[f32; 4]; 4] {
    let mut m = [[0.0; 4]; 4];
    for (r, row) in m.iter_mut().enumerate() {
        for (c, v) in row.iter_mut().enumerate() {
            *v = f32::from_bits(rd32(b, at + 16 * r + 4 * c));
        }
    }
    m
}

/// file_tlf.cpp 0x002EE250 (FixupInstancesFile) + file_instances.cpp 0x002EE3C0 / 0x002EE450.
/// `data` must extend from the chunk data start (offsets may point past the chunk size into the string pool).
pub fn read_instances(data: &[u8], env: &CEnvironment) -> Result<FileInstances, InstancesError> {
    let version = rd32(data, 0);
    if version != FILE_INSTANCES_VERSION {
        return Err(InstancesError::Version(version));
    }
    let s = |o: u32| cstr(data, o as usize).map(str::to_owned);
    let nrefs = rd32(data, 8) as usize;
    let refs_off = rd32(data, 12) as usize;
    let type_refs = (0..nrefs).map(|i| read_type_ref(data, refs_off + 12 * i, env)).collect();
    let nlinks = rd32(data, 16) as usize;
    let links_off = rd32(data, 20) as usize;
    let links = (0..nlinks)
        .map(|i| {
            let o = links_off + 16 * i;
            FileInstanceLink { crc: rd32(data, o), a: s(rd32(data, o + 4)), b: s(rd32(data, o + 8)), d: rd32(data, o + 12) }
        })
        .collect();
    let n = rd32(data, 24) as usize;
    let off = rd32(data, 28) as usize;
    let mut instances = Vec::with_capacity(n);
    for i in 0..n {
        let o = off + 96 * i;
        let tr = rd32(data, o + 80);
        let type_ref = (tr != 0).then(|| read_type_ref(data, tr as usize, env));
        let data_offset = rd32(data, o + 84);
        let value = match (&type_ref, data_offset) {
            (Some(FileTypeRef { ty: Some(t), .. }), d) if d != 0 => Some(decode(t, data, d as usize)),
            _ => None,
        };
        instances.push(FileInstance {
            name_crc: rd32(data, o),
            name: s(rd32(data, o + 4)),
            parent: s(rd32(data, o + 8)),
            flags: rd32(data, o + 12),
            world: read_matrix(data, o + 16),
            type_ref,
            data_offset,
            group_crc: rd32(data, o + 88),
            group: s(rd32(data, o + 92)),
            value,
        });
    }
    Ok(FileInstances { version, type_refs, links, instances })
}
