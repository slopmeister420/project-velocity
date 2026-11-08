//! Chunk\file_chunk.cpp — chunk trees (`.bun`, `.bdl` bundles).
//!
//! ```text
//! +0x00 u32 id        crc of the chunk kind name (see CHUNK_* below)
//! +0x04 u32 name      offset of a nul-terminated name, relative to the file start (0 = none)
//! +0x08 u32 size      bytes of chunk data
//! +0x0C u16 children
//! +0x0E u8  flags     bit 1: data is out of line, +0x10 holds its offset from the chunk
//! +0x0F u8  align     data alignment = 1 << align (inline data starts at +0x10 rounded up)
//! ```
//! inline: data follows the header, then the children (4-byte aligned after the data).
//! out of line: children start at +0x14. padding is filled with 0xAB by the exporter.

use crate::handy::handy_crc;

/// flag bit: data stored out of line.
pub const CHUNK_FLAG_DATA_OFFSET: u8 = 2;

/// crc32 (case-sensitive) of "Chunk" — the file header chunk id the engine checks (0x7A5768).
pub const CHUNK_ID_CHUNK: u32 = 0x92E5_A937;
/// crc32 (case-sensitive) of "Typelib" (0x7A5760).
pub const CHUNK_ID_TYPELIB: u32 = 0x58D3_948D;
/// crc32_no_case of "Timeline" (0x7A5770).
pub const CHUNK_ID_TIMELINE: u32 = 0x23DC_190F;
/// crc32_no_case of "Instances" (0x7A5778).
pub const CHUNK_ID_INSTANCES: u32 = 0x9C2E_14C7;
/// older exporter wrote "Chunk"/"Typelib" ids with the no-case crc; the engine rejects them.
pub const CHUNK_ID_CHUNK_NOCASE: u32 = 0x5324_8633;
pub const CHUNK_ID_TYPELIB_NOCASE: u32 = 0x976E_AD11;

/// a chunk inside a file buffer.
#[derive(Clone, Copy)]
pub struct Chunk<'a> {
    pub file: &'a [u8],
    pub offset: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ChunkError {
    #[error("chunk at {0:#x} runs past the end of the file")]
    OutOfRange(usize),
}

impl<'a> Chunk<'a> {
    /// root chunk of a file (at offset 0).
    pub fn root(file: &'a [u8]) -> Result<Self, ChunkError> {
        let c = Chunk { file, offset: 0 };
        c.check()?;
        Ok(c)
    }

    fn check(&self) -> Result<(), ChunkError> {
        if self.offset + 20 > self.file.len() {
            return Err(ChunkError::OutOfRange(self.offset));
        }
        let d = self.data_offset();
        if d.checked_add(self.size() as usize).is_none_or(|e| e > self.file.len()) {
            return Err(ChunkError::OutOfRange(self.offset));
        }
        Ok(())
    }

    fn u32_at(&self, o: usize) -> u32 {
        rd32(self.file, self.offset + o)
    }

    pub fn id(&self) -> u32 {
        self.u32_at(0)
    }
    pub fn name_offset(&self) -> u32 {
        self.u32_at(4)
    }
    /// name, resolved like FixupChunkTree (0x002ED8B0) does: offset from the file start.
    pub fn name(&self) -> Option<&'a str> {
        cstr(self.file, self.name_offset() as usize)
    }
    pub fn size(&self) -> u32 {
        self.u32_at(8)
    }
    pub fn children(&self) -> u16 {
        u16::from_le_bytes([self.file[self.offset + 12], self.file[self.offset + 13]])
    }
    pub fn flags(&self) -> u8 {
        self.file[self.offset + 14]
    }
    pub fn align(&self) -> u8 {
        self.file[self.offset + 15]
    }

    /// file_chunk.cpp 0x002EDB70 — byte offset of the chunk data.
    pub fn data_offset(&self) -> usize {
        if self.flags() & CHUNK_FLAG_DATA_OFFSET != 0 {
            return self.offset + self.u32_at(16) as usize;
        }
        align_up(self.offset + 16, 1 << self.align())
    }

    /// chunk data bytes.
    pub fn data(&self) -> &'a [u8] {
        let d = self.data_offset();
        &self.file[d..d + self.size() as usize]
    }

    /// data from its start to the end of the file (offset-relative structures may point past `size`).
    pub fn data_tail(&self) -> &'a [u8] {
        &self.file[self.data_offset()..]
    }

    fn first_child_offset(&self) -> usize {
        if self.flags() & CHUNK_FLAG_DATA_OFFSET != 0 {
            self.offset + 20
        } else {
            align_up(self.data_offset() + self.size() as usize, 4)
        }
    }

    /// file_chunk.cpp 0x002ED930 — the chunk after this one and all its descendants.
    pub fn next_sibling(&self) -> Result<Chunk<'a>, ChunkError> {
        let mut c = Chunk { file: self.file, offset: self.first_child_offset() };
        for _ in 0..self.children() {
            c.check()?;
            c = c.next_sibling()?;
        }
        Ok(c)
    }

    /// file_chunk.cpp 0x002EDA10
    pub fn child(&self, index: usize) -> Result<Chunk<'a>, ChunkError> {
        assert!(self.children() > 0, "pCurrentChunk->children > 0");
        assert!(index < self.children() as usize, "nChildIndex < pCurrentChunk->children");
        let mut c = Chunk { file: self.file, offset: self.first_child_offset() };
        c.check()?;
        for _ in 0..index {
            c = c.next_sibling()?;
            c.check()?;
        }
        Ok(c)
    }

    /// all direct children.
    pub fn child_list(&self) -> Result<Vec<Chunk<'a>>, ChunkError> {
        let mut out = Vec::with_capacity(self.children() as usize);
        let mut c = Chunk { file: self.file, offset: self.first_child_offset() };
        for i in 0..self.children() {
            if i > 0 {
                c = out.last().map(|p: &Chunk<'a>| p.next_sibling()).unwrap()?;
            }
            c.check()?;
            out.push(c);
        }
        Ok(out)
    }

    /// first direct child with this id (the loaders' linear scan).
    pub fn find_child(&self, id: u32) -> Option<Chunk<'a>> {
        self.child_list().ok()?.into_iter().find(|c| c.id() == id)
    }

    /// human name for known ids.
    pub fn id_name(&self) -> Option<&'static str> {
        match self.id() {
            CHUNK_ID_CHUNK | CHUNK_ID_CHUNK_NOCASE => Some("Chunk"),
            CHUNK_ID_TYPELIB | CHUNK_ID_TYPELIB_NOCASE => Some("Typelib"),
            CHUNK_ID_TIMELINE => Some("Timeline"),
            CHUNK_ID_INSTANCES => Some("Instances"),
            _ => None,
        }
    }
}

impl std::fmt::Debug for Chunk<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("offset", &self.offset)
            .field("id", &format_args!("{:#010x}", self.id()))
            .field("kind", &self.id_name())
            .field("name", &self.name())
            .field("size", &self.size())
            .field("children", &self.children())
            .finish()
    }
}

/// file_chunk.cpp 0x002EDC50
pub fn align_up(v: usize, a: usize) -> usize {
    let r = v & (a - 1);
    if r != 0 { v + a - r } else { v }
}

pub fn rd32(b: &[u8], o: usize) -> u32 {
    b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])).unwrap_or(0)
}

/// nul-terminated string at `o` (0 = null pointer).
pub fn cstr(b: &[u8], o: usize) -> Option<&str> {
    if o == 0 || o >= b.len() {
        return None;
    }
    let end = b[o..].iter().position(|&c| c == 0)? + o;
    std::str::from_utf8(&b[o..end]).ok()
}

/// id for an arbitrary chunk kind name as the newer exporter writes it.
pub fn chunk_id_no_case(name: &str) -> u32 {
    handy_crc::name_hash(name)
}
