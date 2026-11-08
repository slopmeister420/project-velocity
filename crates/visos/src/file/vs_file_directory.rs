//! File\VsFileDirectory.cpp — VCD pack files and the "file directory" abstraction over them.
//!
//! VCD layout (version 1 = `VCDVERSION_LATEST`, all little-endian):
//! ```text
//! +0   u32  count:24 | version:8                       VsVCDHeader
//! +4   u32  data size (bytes in the data area)
//! +8   FileCrcEntry[count]   { u32 crc, u32 size, u32 offset }      "CRC TOC"
//! +8+12n                     data area; offsets are relative to its start
//! +8+12n+dataSize  FileEntry[count]  { char name[128], u32 size, u32 offset }   "Name TOC"
//! ```
//! `crc = crc32_no_case(name, 0)` (HANDY_CRC.cpp). The packer pads so files start on 2048-byte
//! absolute boundaries. Lookup is a linear scan for the first crc match.

use std::fmt;
use std::path::{Path, PathBuf};

use crate::file::platform;
use crate::handy::handy_crc;

/// `VCDVERSION_LATEST`
pub const VCDVERSION_LATEST: u32 = 1;
/// size of a `FileEntry` (name toc record)
pub const FILE_ENTRY_SIZE: usize = 136;
/// size of a `FileCrcEntry`
pub const FILE_CRC_ENTRY_SIZE: usize = 12;
/// `char name[128]` in `FileEntry`
pub const FILE_ENTRY_NAME_LEN: usize = 128;

#[derive(Debug, thiserror::Error)]
pub enum VcdError {
    #[error("io error on {0}: {1}")]
    Io(PathBuf, std::io::Error),
    #[error("{0}: file too short for header ({1} bytes)")]
    Truncated(String, usize),
    #[error("{name}: size {actual} does not match header (expected {expected})")]
    SizeMismatch { name: String, actual: usize, expected: usize },
}

/// `VsVCDHeader` — first 8 bytes of a pack.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VsVCDHeader {
    pub count_and_version: u32, // +0x00
    pub data_size: u32,         // +0x04
}

impl VsVCDHeader {
    pub fn parse(b: &[u8]) -> Option<Self> {
        Some(Self { count_and_version: rd32(b, 0)?, data_size: rd32(b, 4)? })
    }
    /// VsFileDirectory.cpp 0x00090260 (shared inline)
    pub fn get_file_total(&self) -> u32 {
        self.count_and_version & 0xFF_FFFF
    }
    /// VsFileDirectory.cpp 0x0029DD50
    pub fn get_version(&self) -> u32 {
        self.count_and_version >> 24
    }
    /// VsFileDirectory.cpp 0x0029DD70
    pub fn get_data_size(&self) -> u32 {
        self.data_size
    }
    /// byte offset of the data area (offsets in the tocs are relative to it).
    pub fn data_start(&self) -> usize {
        if self.get_version() < 1 {
            8 + FILE_ENTRY_SIZE * self.get_file_total() as usize
        } else {
            8 + FILE_CRC_ENTRY_SIZE * self.get_file_total() as usize
        }
    }
    /// byte offset of the name toc (version >= 1).
    pub fn name_toc_start(&self) -> usize {
        8 + FILE_CRC_ENTRY_SIZE * self.get_file_total() as usize + self.data_size as usize
    }
    /// total pack size implied by the header (version >= 1).
    pub fn total_size(&self) -> usize {
        self.name_toc_start() + FILE_ENTRY_SIZE * self.get_file_total() as usize
    }
}

/// `FileCrcEntry`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileCrcEntry {
    pub crc: u32,    // +0x00
    pub size: u32,   // +0x04
    pub offset: u32, // +0x08 relative to the data area
}

/// `FileEntry` — name toc record.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String, // +0x00 char[128]
    pub size: u32,    // +0x80
    pub offset: u32,  // +0x84
}

/// one file inside a pack, as returned by iteration/lookup.
#[derive(Clone, Copy, Debug)]
pub struct VcdEntry<'a> {
    pub index: usize,
    pub name: &'a str,
    pub crc: u32,
    pub size: u32,
    /// relative to the data area
    pub offset: u32,
}

/// a whole VCD read into memory (the original's `VsVCDLoader::LoadVCD` also reads the whole pack).
pub struct Vcd {
    pub path: String,
    pub header: VsVCDHeader,
    pub crc_entries: Vec<FileCrcEntry>,
    pub file_entries: Vec<FileEntry>,
    bytes: Vec<u8>,
}

impl fmt::Debug for Vcd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vcd").field("path", &self.path).field("files", &self.crc_entries.len()).finish()
    }
}

impl Vcd {
    /// open a pack by game path (`\FE\FE.vcd`).
    pub fn open(game_path: &str) -> Result<Self, VcdError> {
        Self::open_host(&platform::resolve(game_path), game_path)
    }

    /// open a pack by host path.
    pub fn open_host(path: &Path, display: &str) -> Result<Self, VcdError> {
        let bytes = std::fs::read(path).map_err(|e| VcdError::Io(path.to_path_buf(), e))?;
        Self::from_bytes(display.to_string(), bytes)
    }

    /// parse a pack already in memory.
    pub fn from_bytes(path: String, bytes: Vec<u8>) -> Result<Self, VcdError> {
        let header = VsVCDHeader::parse(&bytes).ok_or_else(|| VcdError::Truncated(path.clone(), bytes.len()))?;
        let n = header.get_file_total() as usize;
        let mut crc_entries = Vec::with_capacity(n);
        let mut file_entries = Vec::with_capacity(n);
        if header.get_version() >= 1 {
            let expected = header.total_size();
            if bytes.len() != expected {
                return Err(VcdError::SizeMismatch { name: path, actual: bytes.len(), expected });
            }
            for i in 0..n {
                let o = 8 + i * FILE_CRC_ENTRY_SIZE;
                crc_entries.push(FileCrcEntry {
                    crc: rd32(&bytes, o).unwrap(),
                    size: rd32(&bytes, o + 4).unwrap(),
                    offset: rd32(&bytes, o + 8).unwrap(),
                });
            }
            let base = header.name_toc_start();
            for i in 0..n {
                file_entries.push(parse_file_entry(&bytes[base + i * FILE_ENTRY_SIZE..]));
            }
        } else {
            // pre-crc format: name toc directly after the header, crcs computed here.
            let need = 8 + n * FILE_ENTRY_SIZE;
            if bytes.len() < need {
                return Err(VcdError::Truncated(path, bytes.len()));
            }
            for i in 0..n {
                let e = parse_file_entry(&bytes[8 + i * FILE_ENTRY_SIZE..]);
                crc_entries.push(FileCrcEntry { crc: handy_crc::name_hash(&e.name), size: e.size, offset: e.offset });
                file_entries.push(e);
            }
        }
        Ok(Self { path, header, crc_entries, file_entries, bytes })
    }

    pub fn len(&self) -> usize {
        self.crc_entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.crc_entries.is_empty()
    }

    fn entry(&self, i: usize) -> VcdEntry<'_> {
        let c = &self.crc_entries[i];
        VcdEntry { index: i, name: &self.file_entries[i].name, crc: c.crc, size: c.size, offset: c.offset }
    }

    /// all entries in toc order.
    pub fn entries(&self) -> impl Iterator<Item = VcdEntry<'_>> + '_ {
        (0..self.len()).map(|i| self.entry(i))
    }

    /// VsFileDirectory.cpp 0x0029D770 — first entry whose crc matches the name's.
    pub fn find(&self, name: &str) -> Option<VcdEntry<'_>> {
        self.find_hash(handy_crc::name_hash(name))
    }

    /// lookup by precomputed `crc32_no_case(name, 0)`.
    pub fn find_hash(&self, crc: u32) -> Option<VcdEntry<'_>> {
        self.crc_entries.iter().position(|e| e.crc == crc).map(|i| self.entry(i))
    }

    /// bytes of an entry, `None` if its range lies outside the data area.
    pub fn data(&self, e: &VcdEntry<'_>) -> Option<&[u8]> {
        let start = self.header.data_start().checked_add(e.offset as usize)?;
        let end = start.checked_add(e.size as usize)?;
        let data_end = self.header.data_start() + self.header.data_size as usize;
        if e.offset as usize + e.size as usize > self.header.data_size as usize || end > data_end {
            return None;
        }
        self.bytes.get(start..end)
    }

    /// find + data.
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.find(name).and_then(|e| self.data(&e))
    }

    /// whole pack bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// consistency problems: crc vs name, name toc vs crc toc, ranges. empty for a sound pack.
    pub fn validate(&self) -> Vec<String> {
        let mut out = Vec::new();
        for e in self.entries() {
            if handy_crc::name_hash(e.name) != e.crc {
                out.push(format!("{}: entry {} crc {:08x} != hash of {:?}", self.path, e.index, e.crc, e.name));
            }
            let fe = &self.file_entries[e.index];
            if (fe.size, fe.offset) != (e.size, e.offset) {
                out.push(format!("{}: entry {} name toc disagrees with crc toc", self.path, e.index));
            }
            if self.data(&e).is_none() {
                out.push(format!("{}: entry {} ({:?}) out of range", self.path, e.index, e.name));
            }
        }
        out
    }
}

fn parse_file_entry(b: &[u8]) -> FileEntry {
    let raw = &b[..FILE_ENTRY_NAME_LEN];
    let end = raw.iter().position(|&c| c == 0).unwrap_or(FILE_ENTRY_NAME_LEN);
    FileEntry {
        name: raw[..end].iter().map(|&c| c as char).collect(),
        size: rd32(b, 128).unwrap(),
        offset: rd32(b, 132).unwrap(),
    }
}

fn rd32(b: &[u8], o: usize) -> Option<u32> {
    b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

/// error codes returned by `VsFileDirectoryDetails::LoadFile`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LoadFileError {
    /// -1: pack has no toc
    #[error("no table of contents")]
    NoToc,
    /// -2: name not found
    #[error("file not found")]
    NotFound,
    /// -3: pack is empty
    #[error("pack empty")]
    Empty,
    /// 0x03000010 and other FILE_* errors from the platform layer
    #[error("io error")]
    Io,
}

/// `VsFileDirectoryDetails` — a directory that is either a VCD pack or a real folder.
/// the folder wins when both exist (Open, 0x0029C490).
pub enum VsFileDirectoryDetails {
    Pack(Vcd),
    Folder(String),
}

impl VsFileDirectoryDetails {
    /// VsFileDirectory.cpp 0x0029C490
    pub fn open(directory_name: &str) -> Result<Self, VcdError> {
        let mut name = directory_name.trim_end_matches('\\').to_string();
        let lower = name.to_ascii_lowercase();
        let temp = if lower.ends_with(".vcd") { &name[..name.len() - 4] } else { &name[..] };
        if platform::resolve(temp).is_dir() {
            name = temp.to_string();
            return Ok(Self::Folder(name));
        }
        Ok(Self::Pack(Vcd::open(&name)?))
    }

    /// VsFileDirectory.cpp 0x0029CEC0 — returns an owned copy (the original returns a pointer into
    /// a preloaded pack or a fresh allocation).
    pub fn load_file(&self, file_name: &str) -> Result<Vec<u8>, LoadFileError> {
        match self {
            Self::Pack(v) => {
                if v.is_empty() {
                    return Err(LoadFileError::Empty);
                }
                let e = v.find(file_name).ok_or(LoadFileError::NotFound)?;
                v.data(&e).map(|d| d.to_vec()).ok_or(LoadFileError::Io)
            }
            Self::Folder(dir) => {
                debug_assert!(file_name.len() + dir.len() + 1 < platform::FILE_MAX_PATHNAME);
                match platform::read(&format!("{}\\{}", dir, file_name)) {
                    Ok(b) => Ok(b),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(LoadFileError::NotFound),
                    Err(_) => Err(LoadFileError::Io),
                }
            }
        }
    }

    /// the file names in directory order (EventDatabase.cpp 0x00090210 / 0x000901C0 walk the header's name
    /// toc: `GetFileTotal`, entry `136 * i`). a folder is listed in case-insensitive name order.
    pub fn file_names(&self) -> Vec<String> {
        match self {
            Self::Pack(v) => v.entries().map(|e| e.name.to_string()).collect(),
            Self::Folder(dir) => {
                let mut v: Vec<String> = std::fs::read_dir(platform::resolve(dir))
                    .map(|rd| {
                        rd.filter_map(|e| e.ok())
                            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                            .map(|e| e.file_name().to_string_lossy().into_owned())
                            .collect()
                    })
                    .unwrap_or_default();
                v.sort_by_key(|n| n.to_ascii_lowercase());
                v
            }
        }
    }

    /// VsFileDirectory.cpp 0x0029D770 — (size) if present.
    pub fn find_file(&self, name: &str) -> Option<u32> {
        match self {
            Self::Pack(v) => v.find(name).map(|e| e.size),
            Self::Folder(dir) => {
                std::fs::metadata(platform::resolve(&format!("{}\\{}", dir, name))).ok().map(|m| m.len() as u32)
            }
        }
    }
}
