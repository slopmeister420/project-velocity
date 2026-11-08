//! Compartment\CompartmentSys.cpp — track streaming compartments (`Sector_N_high.cpt`, `common.cpt`,
//! `instance.cpt` inside `CPTS.VCD`) and their `index.idx`.
//!
//! `.cpt`: 64-byte `CptHeader`, then the body:
//! ```text
//! +0x00 u32 '_CPT'  +0x04 u32 version (3 or 5)  +0x0C u32 size (sectors: body only; common/instance: whole file)
//! +0x10 f32[6] bounds (min xyz, max xyz; 0x7F7FFFFF-style sentinels when unused)
//! +0x28 u32 entryCount  +0x2C u32 -> CptEntry[] (12 bytes)  +0x30 u32 instance lists (bit31: skip)
//! ```
//! all shipped files are raw. the streaming loader can also take a run of chunks `{u32 compSize, RefPack data}`,
//! each chunk start 32-byte aligned relative to +0x40 (CompartmentManager.cpp 0x0008C2F0).
//! after the header come nul-separated names, one per texture entry in entry order.

use crate::chunk::file_chunk::rd32;

/// ('T'<<24)|('P'<<16)|('C'<<8)|'_'
pub const CPT_ID: u32 = 0x5450_435F;
/// "_IDX"
pub const IDX_ID: u32 = 0x5844_495F;
/// legacy MC instance format marker ('INS2')
pub const INS_LEGACY_ID: u32 = 0x494E_5332;
pub const CPT_HEADER_SIZE: usize = 64;

/// entry kinds handled by CompartmentSys.cpp 0x002E7900.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CptEntryKind {
    /// 0 / 8: mesh-like data, registered through the application's specialisation
    Mesh(u8),
    /// 1: texture, named from the name list
    Texture,
    /// 2,3,5,6,7,9..12: ignored by the engine
    Other(u8),
}

#[derive(Clone, Debug)]
pub struct CptEntry {
    /// offset of the data from the header start (0 = none)
    pub offset: u32,
    /// low byte = kind, upper 24 bits are kind-specific
    pub kind_word: u32,
    pub hash: u32,
    /// texture name for kind 1
    pub name: Option<String>,
}

impl CptEntry {
    pub fn kind(&self) -> CptEntryKind {
        match self.kind_word as u8 {
            0 | 8 => CptEntryKind::Mesh(self.kind_word as u8),
            1 => CptEntryKind::Texture,
            k => CptEntryKind::Other(k),
        }
    }
}

/// instance list ids (`INSTANCE_*`), CompartmentSys.cpp 0x002E7B50.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstanceListKind {
    Visual = 0,
    Physics = 1,
    Collision = 2,
    Immovable = 3,
    AlwaysVisible = 4,
    SimpleAnimation = 5,
}

/// record size of each list: 80 bytes, SimpleAnimation 100. (the exporter pads SimpleAnimation records to
/// 112, so the engine misreads any list after one — kept.)
pub fn instance_record_size(kind: InstanceListKind) -> usize {
    if kind == InstanceListKind::SimpleAnimation { 100 } else { 80 }
}

#[derive(Clone, Debug)]
pub struct InstanceList {
    pub kind: InstanceListKind,
    /// raw records (`instance_record_size` each), decoded by the application
    pub records: Vec<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct Compartment {
    pub version: u32,
    pub uncompressed_size: u32,
    pub bounds: [f32; 6],
    pub entries: Vec<CptEntry>,
    pub instance_lists: Vec<InstanceList>,
    pub legacy_instances: bool,
    /// the engine's "Error parsing ... compartment instances" (parse stops, earlier lists are kept)
    pub instance_error: Option<String>,
    /// header + decompressed body; entry offsets index into this
    pub data: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum CptError {
    #[error("not a compartment (id {0:#x})")]
    BadId(u32),
    #[error("compartment truncated")]
    Truncated,
    #[error("decompressed {got} bytes, header says {want}")]
    SizeMismatch { got: usize, want: usize },
    #[error("Error parsing compartment instances - unknown type found")]
    BadInstances,
}

impl Compartment {
    /// CompartmentSys.cpp 0x002E7900 (+ decompression from CompartmentManager.cpp 0x0008C2F0).
    pub fn parse(file: &[u8]) -> Result<Self, CptError> {
        if file.len() < CPT_HEADER_SIZE {
            return Err(CptError::Truncated);
        }
        let id = rd32(file, 0);
        if id != CPT_ID {
            return Err(CptError::BadId(id));
        }
        let uncompressed_size = rd32(file, 12);
        // streamed sectors count the size from +0x40, common/instance from +0; shipped files are all raw
        let data = if file.len() == uncompressed_size as usize + CPT_HEADER_SIZE || file.len() == uncompressed_size as usize {
            file.to_vec()
        } else {
            decompress_body(file, uncompressed_size as usize)?
        };
        // version 3 headers have one more dword before the bounds; the engine reads the v5 layout only
        // (version-3 sectors exist only past LADown sector 50 and would be misread)
        let shift = if rd32(&data, 4) <= 3 { 4 } else { 0 };
        let mut bounds = [0.0; 6];
        for (i, b) in bounds.iter_mut().enumerate() {
            *b = f32::from_bits(rd32(&data, 16 + shift + 4 * i));
        }
        let n = rd32(&data, 40 + shift) as usize;
        let eo = rd32(&data, 44 + shift) as usize;
        if n.checked_mul(12).and_then(|x| x.checked_add(eo)).is_none_or(|e| e > data.len()) {
            return Err(CptError::Truncated);
        }
        let mut names = data[CPT_HEADER_SIZE..].split(|&c| c == 0);
        let mut entries = Vec::with_capacity(n);
        for i in 0..n {
            let o = eo + 12 * i;
            if o + 12 > data.len() {
                return Err(CptError::Truncated);
            }
            let mut e = CptEntry { offset: rd32(&data, o), kind_word: rd32(&data, o + 4), hash: rd32(&data, o + 8), name: None };
            if e.offset != 0 && e.kind() == CptEntryKind::Texture {
                e.name = names.next().map(|s| String::from_utf8_lossy(s).into_owned());
            }
            entries.push(e);
        }
        let mut c = Compartment {
            version: rd32(&data, 4),
            uncompressed_size,
            bounds,
            entries,
            instance_lists: Vec::new(),
            legacy_instances: false,
            instance_error: None,
            data,
        };
        let il = rd32(&c.data, 48 + shift);
        if il != 0 {
            c.read_instance_lists(il)?;
        }
        Ok(c)
    }

    /// CompartmentSys.cpp 0x002E7B50
    fn read_instance_lists(&mut self, field: u32) -> Result<(), CptError> {
        let base = (field & 0x7FFF_FFFF) as usize;
        if rd32(&self.data, base) == INS_LEGACY_ID {
            self.legacy_instances = true;
            return Ok(());
        }
        if field & 0x8000_0000 != 0 {
            return Ok(());
        }
        let num = rd32(&self.data, base) as usize;
        debug_assert!(num <= 6, "pListHeader->m_numLists <= INSTANCE_Unknown");
        let mut p = base + 16;
        let mut seen = 0u32;
        for _ in 0..num {
            let k = rd32(&self.data, p);
            p += 4;
            let kind = match k {
                0 => InstanceListKind::Visual,
                1 => InstanceListKind::Physics,
                2 => InstanceListKind::Collision,
                3 => InstanceListKind::Immovable,
                4 => InstanceListKind::AlwaysVisible,
                5 => InstanceListKind::SimpleAnimation,
                _ => {
                    self.instance_error = Some(format!("Error parsing Unknown compartment instances ({k})"));
                    break;
                }
            };
            let count = rd32(&self.data, p) as usize;
            // a repeated list (or > 50000 visuals) aborts the parse; lists read so far stay
            if seen & (1 << k) != 0 || (kind == InstanceListKind::Visual && count > 50000) {
                self.instance_error = Some(format!("Error parsing {kind:?} compartment instances"));
                break;
            }
            seen |= 1 << k;
            p += 12;
            let sz = instance_record_size(kind);
            if count.checked_mul(sz).and_then(|n| n.checked_add(p)).is_none_or(|e| e > self.data.len()) {
                self.instance_error = Some(format!("{kind:?} list runs past the end"));
                break;
            }
            let records = (0..count).map(|i| self.data[p + i * sz..p + (i + 1) * sz].to_vec()).collect();
            p += count * sz;
            self.instance_lists.push(InstanceList { kind, records });
        }
        Ok(())
    }

    /// bytes of an entry from its offset to the end of the body.
    pub fn entry_data(&self, e: &CptEntry) -> Option<&[u8]> {
        (e.offset != 0).then(|| self.data.get(e.offset as usize..)).flatten()
    }
}

/// CompartmentManager.cpp 0x0008C2F0 — decompress the chunk run after the header.
fn decompress_body(file: &[u8], uncompressed: usize) -> Result<Vec<u8>, CptError> {
    let mut out = file[..CPT_HEADER_SIZE].to_vec();
    let src = &file[CPT_HEADER_SIZE..];
    let mut s = 0usize;
    while out.len() - CPT_HEADER_SIZE < uncompressed {
        if s + 4 > src.len() {
            return Err(CptError::Truncated);
        }
        let n = rd32(src, s) as usize;
        s += 4;
        let end = (s + n).min(src.len());
        refpack_decompress(&src[s..end], &mut out);
        s += n;
        while s & 0x1F != 0 {
            s += 1;
        }
    }
    if out.len() - CPT_HEADER_SIZE != uncompressed {
        return Err(CptError::SizeMismatch { got: out.len() - CPT_HEADER_SIZE, want: uncompressed });
    }
    Ok(out)
}

/// CompartmentSys.cpp 0x002E93E0 — RefPack (EA QFS) decompression without the stream header.
pub fn refpack_decompress(src: &[u8], out: &mut Vec<u8>) {
    let mut p = 0usize;
    let get = |p: &mut usize| -> usize {
        let v = src.get(*p).copied().unwrap_or(0) as usize;
        *p += 1;
        v
    };
    loop {
        if p >= src.len() {
            return;
        }
        let c = get(&mut p);
        let (lit, off, len);
        if c & 0x80 == 0 {
            let b1 = get(&mut p);
            lit = c & 3;
            off = b1 + ((c & 0x60) << 3) + 1;
            len = ((c & 0x1C) >> 2) + 3;
        } else if c & 0x40 == 0 {
            let b1 = get(&mut p);
            let b2 = get(&mut p);
            lit = b1 >> 6;
            off = ((b1 & 0x3F) << 8) + b2 + 1;
            len = (c & 0x3F) + 4;
        } else if c & 0x20 == 0 {
            let b1 = get(&mut p);
            let b2 = get(&mut p);
            let b3 = get(&mut p);
            lit = c & 3;
            off = (((c & 0x10) >> 4) << 16) + (b1 << 8) + b2 + 1;
            len = (((c & 0x0C) >> 2) << 8) + b3 + 5;
        } else {
            let n = 4 * (c & 0x1F) + 4;
            if n > 0x70 {
                // end code: up to 3 trailing literals
                for _ in 0..(c & 3) {
                    out.push(get(&mut p) as u8);
                }
                return;
            }
            for _ in 0..n {
                out.push(get(&mut p) as u8);
            }
            continue;
        }
        for _ in 0..lit {
            out.push(get(&mut p) as u8);
        }
        let start = out.len().saturating_sub(off);
        for i in 0..len {
            let b = out[start + i];
            out.push(b);
        }
    }
}

/// `index.idx`: `{'_IDX', u32 version (2), u32 count, ...}` — read by CompartmentSys.cpp 0x002E6CC0
/// (requires version >= the manager's minimum). records are kept raw for the application.
#[derive(Clone, Debug)]
pub struct CompartmentIndex {
    pub version: u32,
    pub count: u32,
    pub data: Vec<u8>,
}

impl CompartmentIndex {
    pub fn parse(file: &[u8]) -> Option<Self> {
        (rd32(file, 0) == IDX_ID).then(|| CompartmentIndex { version: rd32(file, 4), count: rd32(file, 8), data: file.to_vec() })
    }
}
