//! Timeline\file_tlf.cpp — Timeline chunks (`.bun` bundles), version 5.
//!
//! all offsets relative to the Timeline chunk data:
//! ```text
//! +0x00 u32 version (5)  +0x04 u32 nameCrc  +0x08 u32 name
//! +0x0C u32 typeRefCount +0x10 -> FileTypeRef[] (12)
//! +0x14 u32 targetCount  +0x18 -> FileTarget[] (96)
//! +0x1C u32 channelCount +0x20 -> FileChannel[] (40)
//! +0x24 f32 length (seconds)
//! FileTarget  (96): +0 crc, +4 name, +0x10 VsMatrix, +0x50 circular, +0x54 pointCount, +0x58 -> f32[3][] points
//! FileChannel (40): +0 crc, +4 name, +8 typeCrc, +0x0C str, +0x10 str, +0x14 trackCount, +0x18 -> FileTrack[],
//!                   +0x1C/+0x20 runtime (bone anim info, render matrices)
//! FileTrack   (24): +0 crc, +4 name, +8 itemCount, +0x0C -> FileItem[]
//! FileItem    (16): +0 kind, +4 f32 start, +8 f32 length, +0x0C -> payload
//!   kind 1 (bone animation): payload +0 ?, +4 str animation, +8 f32 blend in, +0x0C f32 blend out, +0x10 f32 weight,
//!     +0x14 f32 start frame, +0x18 f32 end frame (normalised), +0x1C runtime anim cache
//!   kind 3: payload +4 str
//!   kind 2 (path): payload +0 target crc, +8 f32 ease in, +0x0C f32 ease out, +0x10 f32 start, +0x14 f32 end,
//!     +0x18 u8 flags, +0x19 u8 axis flags, +0x1C runtime target
//!   kind 4 (typed event): payload +0 crc, +4 u32 mode (1 = gets an end call, 2 = no updates), +8 -> FileTypeRef,
//!     +0x0C -> data of that type
//! ```

use crate::chunk::file_chunk::{cstr, rd32};
use crate::instances::file_instances::read_matrix;
use crate::type_::file_type::{read_type_ref, FileTypeRef};
use crate::type_::vs_type::{decode, CEnvironment, Value};

/// file_chunk.cpp 0x002EDC80 — the only version the fix-up accepts.
pub const FILE_TLF_VERSION: u32 = 5;

pub const ITEM_KIND_BONE_ANIM: u32 = 1;
pub const ITEM_KIND_PATH: u32 = 2;
pub const ITEM_KIND_STRING: u32 = 3;
pub const ITEM_KIND_TYPED: u32 = 4;

#[derive(Clone, Debug)]
pub struct FileTarget {
    pub crc: u32,
    pub name: Option<String>,
    pub world: [[f32; 4]; 4],
    /// +0x50 the point list loops
    pub circular: bool,
    pub points: Vec<[f32; 3]>,
}

#[derive(Clone, Debug)]
pub enum FilePayload {
    None,
    BoneAnim { unk_00: u32, animation: Option<String>, blend_in: f32, blend_out: f32, weight: f32, start: f32, end: f32 },
    String(Option<String>),
    Typed { crc: u32, mode: u32, type_ref: Option<FileTypeRef>, value: Option<Value> },
    Path { target_crc: u32, ease_in: f32, ease_out: f32, start: f32, end: f32, flags: u8, axis_flags: u8 },
    Unknown(u32),
}

#[derive(Clone, Debug)]
pub struct FileItem {
    pub kind: u32,
    pub start: f32,
    pub length: f32,
    pub payload: FilePayload,
}

#[derive(Clone, Debug)]
pub struct FileTrack {
    pub crc: u32,
    pub name: Option<String>,
    pub items: Vec<FileItem>,
}

#[derive(Clone, Debug)]
pub struct FileChannel {
    pub crc: u32,
    pub name: Option<String>,
    pub type_crc: u32,
    pub str_a: Option<String>,
    pub str_b: Option<String>,
    pub tracks: Vec<FileTrack>,
}

#[derive(Clone, Debug)]
pub struct FileTimeline {
    pub version: u32,
    pub name_crc: u32,
    pub name: Option<String>,
    pub type_refs: Vec<FileTypeRef>,
    pub targets: Vec<FileTarget>,
    pub channels: Vec<FileChannel>,
    pub length: f32,
}

#[derive(Debug, thiserror::Error)]
pub enum TlfError {
    #[error("Error: incorrect version number for timeline file ({0})")]
    Version(u32),
}

/// file_chunk.cpp 0x002EDCA0 (FixupTimeline) with file_tlf.cpp 0x002EDDD0 / 0x002EDE30 / 0x002EDFA0 /
/// file_tlf.cpp 0x002EE060 / 0x002EE0D0. (the original also converts version <= 2 targets; unreachable after the
/// version == 5 check.)
pub fn read_timeline(data: &[u8], env: &CEnvironment) -> Result<FileTimeline, TlfError> {
    let version = rd32(data, 0);
    if version != FILE_TLF_VERSION {
        return Err(TlfError::Version(version));
    }
    let s = |o: u32| cstr(data, o as usize).map(str::to_owned);
    let arr = |count_at: usize| (rd32(data, count_at) as usize, rd32(data, count_at + 4) as usize);

    let (n, off) = arr(12);
    let type_refs = (0..n).map(|i| read_type_ref(data, off + 12 * i, env)).collect();

    let (n, off) = arr(20);
    let targets = (0..n)
        .map(|i| {
            let o = off + 96 * i;
            let np = rd32(data, o + 84) as usize;
            let po = rd32(data, o + 88) as usize;
            FileTarget {
                crc: rd32(data, o),
                name: s(rd32(data, o + 4)),
                world: read_matrix(data, o + 16),
                circular: rd32(data, o + 80) != 0,
                points: (0..np)
                    .map(|k| {
                        let p = po + 12 * k;
                        [0, 4, 8].map(|d| f32::from_bits(rd32(data, p + d)))
                    })
                    .collect(),
            }
        })
        .collect();

    let (n, off) = arr(28);
    let channels = (0..n)
        .map(|i| {
            let o = off + 40 * i;
            let (nt, to) = arr(o + 20);
            FileChannel {
                crc: rd32(data, o),
                name: s(rd32(data, o + 4)),
                type_crc: rd32(data, o + 8),
                str_a: s(rd32(data, o + 12)),
                str_b: s(rd32(data, o + 16)),
                tracks: (0..nt).map(|k| read_track(data, to + 24 * k, env)).collect(),
            }
        })
        .collect();

    Ok(FileTimeline {
        version,
        name_crc: rd32(data, 4),
        name: s(rd32(data, 8)),
        type_refs,
        targets,
        channels,
        length: f32::from_bits(rd32(data, 36)),
    })
}

fn read_track(data: &[u8], o: usize, env: &CEnvironment) -> FileTrack {
    let ni = rd32(data, o + 8) as usize;
    let io = rd32(data, o + 12) as usize;
    let items = (0..ni)
        .map(|k| {
            let it = io + 16 * k;
            let kind = rd32(data, it);
            let p = rd32(data, it + 12) as usize;
            let payload = if p == 0 {
                FilePayload::None
            } else {
                match kind {
                    ITEM_KIND_BONE_ANIM => {
                        let fl = |o: usize| f32::from_bits(rd32(data, p + o));
                        FilePayload::BoneAnim {
                            unk_00: rd32(data, p),
                            animation: cstr(data, rd32(data, p + 4) as usize).map(str::to_owned),
                            blend_in: fl(8),
                            blend_out: fl(12),
                            weight: fl(16),
                            start: fl(20),
                            end: fl(24),
                        }
                    }
                    ITEM_KIND_PATH => {
                        let fl = |o: usize| f32::from_bits(rd32(data, p + o));
                        FilePayload::Path {
                            target_crc: rd32(data, p),
                            ease_in: fl(8),
                            ease_out: fl(12),
                            start: fl(16),
                            end: fl(20),
                            flags: data.get(p + 24).copied().unwrap_or(0),
                            axis_flags: data.get(p + 25).copied().unwrap_or(0),
                        }
                    }
                    ITEM_KIND_STRING => FilePayload::String(cstr(data, rd32(data, p + 4) as usize).map(str::to_owned)),
                    ITEM_KIND_TYPED => {
                        let tr = rd32(data, p + 8) as usize;
                        let d = rd32(data, p + 12) as usize;
                        let type_ref = (tr != 0).then(|| read_type_ref(data, tr, env));
                        let value = match &type_ref {
                            // the data's own pointers are relative to the data itself (FixupTimelineItems 0x002EE0D0
                            // passes the data address as the fix-up base to FixupDataOfType 0x002ED740)
                            Some(FileTypeRef { ty: Some(t), .. }) if d != 0 && d < data.len() => {
                                Some(decode(t, &data[d..], 0))
                            }
                            _ => None,
                        };
                        FilePayload::Typed { crc: rd32(data, p), mode: rd32(data, p + 4), type_ref, value }
                    }
                    _ => FilePayload::Unknown(p as u32),
                }
            };
            FileItem {
                kind,
                start: f32::from_bits(rd32(data, it + 4)),
                length: f32::from_bits(rd32(data, it + 8)),
                payload,
            }
        })
        .collect();
    FileTrack { crc: rd32(data, o), name: cstr(data, rd32(data, o + 4) as usize).map(str::to_owned), items }
}
