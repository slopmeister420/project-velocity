//! StringManager\StringManager.cpp — `CStringManager`: `Strings\StrIndex.idx` index + per-language `.sbk` sections.
//!
//! index file (little endian):
//! ```text
//! +0x00 u32 index count   +0x04 u32 reserved count  +0x08 u32 section count  +0x0C u32 language count
//! +0x10 u32 language list offset   +0x14 u32 section list offset
//! +0x18 index entries, 16 bytes: u32 crc | u32 handle (runtime) | u16 flags | u16 length | u32 data (runtime)
//! lists: { u32 size, u32 crc, char name[] } chained by size
//! ```
//! section file (`Strings\<language>\<name>.sbk`): `u32 section crc | u32 count | u32 language crc | u32 next (runtime)`,
//! then `count` strings `{ u32 crc, u16 argument count, u16 size, argument[16 * n], utf-16 text }` chained by size.
//! string ids are `crc32(name, 0)` of the upper-case id (`GENERAL_SELECT`), sections `crc32("Audio")`.

use std::collections::HashMap;

use visos::handy::handy_crc;

/// `eStatic`: string is plain text in its section, no arguments to expand.
pub const FLAG_STATIC: u16 = 0x01;
/// `eLoaded`: section holding the string is resident.
pub const FLAG_LOADED: u16 = 0x04;
/// `eRuntimeStatic`: reserved entries at the end of the index, filled at runtime.
pub const FLAG_RUNTIME_STATIC: u16 = 0x40;

/// `0x006C1DFC` .. `0x006C1E44`: placeholder strings returned for failed lookups.
pub const STR_EXPAND_FAILED: &str = "EXPAND FAILED";
pub const STR_NOT_FOUND: &str = "NOT FOUND";
pub const STR_NOT_LOADED: &str = "NOT LOADED";
pub const STR_NO_INDEX: &str = "NO INDEX";

/// port (not original): strings of the port's own screens (PC settings page), id -> text in every language
/// (English only). Looked up by the same crc as the game's ids, only when the game's index lacks the id.
pub const PORT_STRINGS: &[(&str, &str)] = &[
    ("OPTION_PC", "PC Settings"),
    ("OPTION_PC_TITLE", "PC Settings"),
    ("OPTION_PC_DISPLAYMODE", "Display Mode"),
    ("OPTION_PC_DISPLAYMODE_WINDOWED", "Windowed"),
    ("OPTION_PC_DISPLAYMODE_BORDERLESS", "Borderless"),
    ("OPTION_PC_DISPLAYMODE_FULLSCREEN", "Fullscreen"),
    ("OPTION_PC_RESOLUTION", "Resolution"),
    ("OPTION_PC_FPSCAP", "Frame Rate Cap"),
    ("OPTION_PC_FPSCAP_UNCAPPED", "Off"),
    ("OPTION_PC_VSYNC", "V-Sync"),
    ("OPTION_PC_MUSIC", "Music Order"),
    ("OPTION_PC_MUSIC_ORIGINAL", "Original"),
    ("OPTION_PC_MUSIC_RANDOM", "Random"),
];

/// the port string for `crc`, if any.
pub fn port_string(crc: u32) -> Option<&'static str> {
    PORT_STRINGS.iter().find(|(id, _)| string_crc(id) == crc).map(|(_, t)| *t)
}

/// error codes (`0x0B000000 | n`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringError {
    NotInitialised = 0x0B00_0000,
    InvalidFilename = 0x0B00_0003,
    FileInfo = 0x0B00_0004,
    IndexLoaded = 0x0B00_0008,
    InvalidSection = 0x0B00_0009,
    IndexNotLoaded = 0x0B00_000A,
    SectionNotFound = 0x0B00_000B,
    SectionFileInfo = 0x0B00_000C,
    BadFormat = 0x0B00_0007,
    StringNotFound = 0x0B00_0011,
}

/// one 16-byte index entry (`CIndexEntry`).
#[derive(Debug, Clone, Copy)]
pub struct IndexEntry {
    pub crc: u32,       // +0x00
    pub flags: u16,     // +0x08
    pub length: u16,    // +0x0A characters
}

/// `{ size, crc, name }` record of the language and section lists.
#[derive(Debug, Clone)]
pub struct NamedCrc {
    pub crc: u32,
    pub name: String,
}

/// parsed `StrIndex.idx`.
#[derive(Debug, Clone)]
pub struct StringIndex {
    pub entries: Vec<IndexEntry>,
    pub reserved: u32,
    pub languages: Vec<NamedCrc>,
    pub sections: Vec<NamedCrc>,
}

fn u16_at(b: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(o..o + 2)?.try_into().ok()?))
}
fn u32_at(b: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(o..o + 4)?.try_into().ok()?))
}

fn read_named_list(b: &[u8], mut o: usize, count: u32) -> Option<Vec<NamedCrc>> {
    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let size = u32_at(b, o)? as usize;
        let crc = u32_at(b, o + 4)?;
        let raw = b.get(o + 8..o + size)?;
        let name = String::from_utf8_lossy(raw.split(|&c| c == 0).next()?).into_owned();
        out.push(NamedCrc { crc, name });
        if size == 0 {
            return None;
        }
        o += size;
    }
    Some(out)
}

impl StringIndex {
    /// StringManager.cpp 0x002E9F00 — `LoadIndexFileCompleted` (parse half; handles are rust-side maps).
    pub fn parse(b: &[u8]) -> Result<Self, StringError> {
        let f = || -> Option<Self> {
            let count = u32_at(b, 0)?;
            let reserved = u32_at(b, 4)?;
            let sections = u32_at(b, 8)?;
            let languages = u32_at(b, 12)?;
            let lang_off = u32_at(b, 16)? as usize;
            let sect_off = u32_at(b, 20)? as usize;
            let mut entries = Vec::with_capacity(count as usize);
            for i in 0..count as usize {
                let o = 24 + 16 * i;
                entries.push(IndexEntry { crc: u32_at(b, o)?, flags: u16_at(b, o + 8)?, length: u16_at(b, o + 10)? });
            }
            if reserved != 0 {
                // the first reserved entry must be runtime static (assert line 570)
                let first = entries.get((count - reserved) as usize)?;
                if first.flags & FLAG_RUNTIME_STATIC == 0 {
                    return None;
                }
            }
            Some(Self {
                entries,
                reserved,
                languages: read_named_list(b, lang_off, languages)?,
                sections: read_named_list(b, sect_off, sections)?,
            })
        };
        f().ok_or(StringError::BadFormat)
    }

    /// StringManager.cpp 0x002EAB80 — `FindIndexEntry`: linear search on crc.
    pub fn find(&self, crc: u32) -> Option<usize> {
        self.entries.iter().position(|e| e.crc == crc)
    }
}

/// one string of a section file.
#[derive(Debug, Clone)]
pub struct SectionString {
    pub crc: u32,
    /// 16-byte argument records: `u32 formatter crc | u32 argument crc | u32 value (runtime) | u16 position | u16`.
    pub arguments: Vec<[u8; 16]>,
    pub text: Vec<u16>,
}

/// parsed `.sbk`.
#[derive(Debug, Clone)]
pub struct SectionFile {
    pub section_crc: u32,
    pub language_crc: u32,
    pub strings: Vec<SectionString>,
}

impl SectionFile {
    /// StringManager.cpp 0x002EABE0 — `FixupIndexPointers` walk (records chained by their u16 size at +6).
    pub fn parse(b: &[u8]) -> Result<Self, StringError> {
        let f = || -> Option<Self> {
            let section_crc = u32_at(b, 0)?;
            let count = u32_at(b, 4)?;
            let language_crc = u32_at(b, 8)?;
            let mut o = 16usize;
            let mut strings = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let crc = u32_at(b, o)?;
                let argc = u16_at(b, o + 4)? as usize;
                let size = u16_at(b, o + 6)? as usize;
                if size < 8 + 16 * argc {
                    return None;
                }
                let mut arguments = Vec::with_capacity(argc);
                for k in 0..argc {
                    arguments.push(b.get(o + 8 + 16 * k..o + 24 + 16 * k)?.try_into().ok()?);
                }
                let raw = b.get(o + 8 + 16 * argc..o + size)?;
                let text: Vec<u16> =
                    raw.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|&c| c != 0).collect();
                strings.push(SectionString { crc, arguments, text });
                o += size;
            }
            Some(Self { section_crc, language_crc, strings })
        };
        f().ok_or(StringError::BadFormat)
    }
}

/// `CStringManager`.
pub struct CStringManager {
    /// `+0x80` data root (`\Strings`).
    pub root: String,
    /// `+0x44` language name (folder below the root), `+0x40` its crc.
    pub language: String,
    pub language_crc: u32,
    /// `+0x48` index.
    pub index: Option<StringIndex>,
    /// `+0x4C` loaded sections (list head), newest first as in the original.
    pub sections: Vec<SectionFile>,
    /// resolved text per string crc for the strings whose section is loaded.
    resident: HashMap<u32, Vec<u16>>,
    /// argument records of the resident strings (16 bytes each, value at +8 written by `SetArgument`).
    arguments: HashMap<u32, Vec<[u8; 16]>>,
}

impl Default for CStringManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CStringManager {
    /// StringManager.cpp 0x002E9700
    pub fn new() -> Self {
        Self {
            root: "\\Strings".into(),
            language: String::new(),
            language_crc: 0,
            index: None,
            sections: Vec::new(),
            resident: HashMap::new(),
            arguments: HashMap::new(),
        }
    }

    /// StringManager.cpp 0x002ED510 — `IsIndexLoaded`
    pub fn is_index_loaded(&self) -> bool {
        self.index.is_some()
    }

    /// StringManager.cpp 0x002E9A80 — `LoadIndexFile` (synchronous path).
    pub fn load_index_file(&mut self, file: &str) -> Result<(), StringError> {
        if file.is_empty() {
            return Err(StringError::InvalidFilename);
        }
        if self.index.is_some() {
            return Err(StringError::IndexLoaded);
        }
        let path = visos::file::resolve(&format!("{}\\{}", self.root, file));
        let bytes = std::fs::read(path).map_err(|_| StringError::FileInfo)?;
        let idx = StringIndex::parse(&bytes)?;
        log::info!(
            "Loaded String Index file: Indexes {} ({} reserved), Sections {}, Languages {}",
            idx.entries.len(),
            idx.reserved,
            idx.sections.len(),
            idx.languages.len()
        );
        self.index = Some(idx);
        Ok(())
    }

    /// select the language folder; mirrors the `+0x44` name / `+0x40` crc pair set by `SetLanguage`.
    pub fn set_language(&mut self, name: &str) {
        self.language = name.to_string();
        self.language_crc = string_crc(name);
    }

    /// StringManager.cpp 0x002EBD70 — `ChangeLanguage(crc)`: every loaded section of another language is
    /// unloaded and loaded again from the new language's folder (loads are synchronous here).
    pub fn change_language(&mut self, crc: u32) -> Result<(), StringError> {
        let Some(index) = &self.index else {
            log::warn!("CStringManager::ChangeLanguage - Index file is not loaded");
            return Err(StringError::IndexNotLoaded);
        };
        let Some(lang) = index.languages.iter().find(|l| l.crc == crc).map(|l| l.name.clone()) else {
            log::warn!("CStringManager::ChangeLanguage - Language Not Found");
            return Err(StringError::FileInfo);
        };
        self.language = lang;
        self.language_crc = crc;
        let stale: Vec<u32> = self.sections.iter().filter(|s| s.language_crc != crc).map(|s| s.section_crc).collect();
        for s in stale {
            self.unload_section(s);
            if self.load_section(s).is_err() {
                log::warn!("CStringManager::ChangeLanguage - section {s:08X} missing in {}", self.language);
            }
        }
        Ok(())
    }

    /// StringManager.cpp 0x002EAFE0 — `IsSectionLoaded`
    pub fn is_section_loaded(&self, crc: u32) -> bool {
        self.sections.iter().any(|s| s.section_crc == crc)
    }

    /// StringManager.cpp 0x002EA5D0 — `LoadSection(const char*)`
    pub fn load_section_name(&mut self, name: &str) -> Result<(), StringError> {
        if name.is_empty() {
            return Err(StringError::InvalidSection);
        }
        self.load_section(string_crc(name))
    }

    /// StringManager.cpp 0x002EA620 — `LoadSection(crc)`: file name comes from the index section list.
    pub fn load_section(&mut self, crc: u32) -> Result<(), StringError> {
        let Some(index) = &self.index else {
            return Err(StringError::IndexNotLoaded);
        };
        if self.is_section_loaded(crc) {
            log::warn!("CStringManager::LoadSection - Attempting to Load Section thats already loaded");
            return Ok(());
        }
        let Some(file) = index.sections.iter().find(|s| s.crc == crc).map(|s| s.name.clone()) else {
            return Err(StringError::SectionNotFound);
        };
        let path = visos::file::resolve(&format!("{}\\{}\\{}", self.root, self.language, file));
        let bytes = std::fs::read(path).map_err(|_| StringError::SectionFileInfo)?;
        let section = SectionFile::parse(&bytes)?;
        self.section_loaded(section);
        Ok(())
    }

    /// StringManager.cpp 0x002EAF00 — `LoadSectionCompleted`: link at the head, then fix up index pointers.
    fn section_loaded(&mut self, section: SectionFile) {
        self.fixup_index_pointers(&section);
        self.sections.insert(0, section);
    }

    /// StringManager.cpp 0x002EABE0 — `FixupIndexPointers`
    fn fixup_index_pointers(&mut self, section: &SectionFile) {
        let Some(index) = &self.index else { return };
        for s in &section.strings {
            if index.find(s.crc).is_some() {
                self.resident.insert(s.crc, s.text.clone());
                if !s.arguments.is_empty() {
                    self.arguments.insert(s.crc, s.arguments.clone());
                }
            } else {
                log::warn!(
                    "CStringManager::FixupIndexPointers - A string in the new section ({:08X}) doesn't exist in the global index",
                    s.crc
                );
            }
        }
    }

    /// StringManager.cpp 0x002EAD10 — `UnFixupIndexPointers` + unlink.
    pub fn unload_section(&mut self, crc: u32) {
        if let Some(i) = self.sections.iter().position(|s| s.section_crc == crc) {
            let s = self.sections.remove(i);
            for st in &s.strings {
                self.resident.remove(&st.crc);
                self.arguments.remove(&st.crc);
            }
        }
    }

    /// StringManager.cpp 0x002EB020 — `GetStringHandle` + `LockString`: the text a handle resolves to,
    /// with the placeholder strings for missing index / entry / section.
    pub fn get_string(&self, crc: u32) -> Vec<u16> {
        let Some(index) = &self.index else {
            return STR_NO_INDEX.encode_utf16().collect();
        };
        if index.find(crc).is_none() {
            if let Some(t) = port_string(crc) {
                return t.encode_utf16().collect();
            }
            return STR_NOT_FOUND.encode_utf16().collect();
        }
        match self.resident.get(&crc) {
            Some(t) => match self.arguments.get(&crc) {
                Some(args) => self.expand_string(t, args),
                None => t.clone(),
            },
            None => STR_NOT_LOADED.encode_utf16().collect(),
        }
    }

    /// StringManager.cpp 0x002EB360 — `SetArgument(string, argument, value)`: value bits as the formatter reads them.
    pub fn set_argument(&mut self, string: u32, argument: u32, value: u32) -> Result<(), StringError> {
        let Some(args) = self.arguments.get_mut(&string) else { return Err(StringError::StringNotFound) };
        for a in args.iter_mut() {
            if u32::from_le_bytes(a[4..8].try_into().unwrap()) == argument {
                a[8..12].copy_from_slice(&value.to_le_bytes());
                return Ok(());
            }
        }
        Err(StringError::StringNotFound)
    }

    /// StringManager.cpp 0x002EB5B0 — `ExpandString`: text up to each argument's position, then its formatter.
    fn expand_string(&self, text: &[u16], args: &[[u8; 16]]) -> Vec<u16> {
        let mut out = Vec::with_capacity(text.len() + 16);
        let mut copied = 0usize;
        for a in args {
            let formatter = u32::from_le_bytes(a[0..4].try_into().unwrap());
            let value = u32::from_le_bytes(a[8..12].try_into().unwrap());
            let pos = u16::from_le_bytes([a[12], a[13]]) as usize;
            let n = pos.saturating_sub(copied);
            out.extend(text.iter().skip(copied).take(n));
            copied += n;
            let f = if FORMATTERS.iter().any(|(c, _)| *c == formatter) { formatter } else {
                log::warn!("CStringManager::ExpandArgument - Could not locate formatter - using default");
                string_crc("INT")
            };
            if f == 0x4682_6FCB {
                // StringManager.cpp 0x002ED290 — STRINGID: the referenced string, expanded
                out.extend(self.get_string(value));
                continue;
            }
            let s = format_argument(f, value, self.language_crc);
            out.extend(s.encode_utf16());
        }
        out.extend(text.iter().skip(copied));
        out
    }

    /// `GetString` by id name.
    pub fn get_string_id(&self, id: &str) -> Vec<u16> {
        self.get_string(string_crc(id))
    }

    /// convenience: as a rust string.
    pub fn text(&self, crc: u32) -> String {
        String::from_utf16_lossy(&self.get_string(crc))
    }
}

/// formatter crcs: built-in INT / FLOAT (StringManager.cpp) and the game's TIME / TIME_MILLI / CASH
/// (src\GameStringManager\StringManager.cpp, registered by VsCommonMain 0x0016DED8).
const FORMATTERS: [(u32, &str); 6] =
    [(0x4682_6FCB, "STRINGID"), (0x7DB7_4129, "INT"), (0xF8D3_6B38, "FLOAT"), (0x7845_AAED, "TIME"), (0xF78E_A278, "TIME_MILLI"), (0x1168_D711, "CASH")];

/// languages whose FLOAT formatter writes a decimal comma (0x002ED020).
const DECIMAL_COMMA: [u32; 8] = [0x9756_FD40, 0xE074_3459, 0xF5A6_6D3D, 0x66C9_13D6, 0x6661_9320, 0x191B_6ACB, 0x0568_8F2D, 0x145A_E434];

fn format_argument(formatter: u32, value: u32, language: u32) -> String {
    match formatter {
        // StringManager.cpp 0x002ECE50
        0x7DB7_4129 => format!("{}", value as i32),
        // StringManager.cpp 0x002ED020
        0xF8D3_6B38 => {
            let s = format!("{:.5}", f32::from_bits(value));
            if DECIMAL_COMMA.contains(&language) { s.replace('.', ",") } else { s }
        }
        0x7845_AAED => format_time(f32::from_bits(value), false),
        0xF78E_A278 => format_time(f32::from_bits(value), true),
        0x1168_D711 => format_cash(value),
        _ => String::new(),
    }
}

/// GameStringManager\StringManager.cpp 0x000C7030 — `m:ss` / `m:ss.hh` (the sign is dropped as in the original)
fn format_time(t: f32, millis: bool) -> String {
    let t = if t < 0.0 { 0.0 - t } else { t };
    let minutes = (t / 60.0) as i32;
    let seconds = (t as i32) % 60;
    let whole = t as i64 as u32;
    let hundredths = ((t as f64 - whole as f64) * 100.0) as u32;
    let two = |v: u32| if v >= 10 { format!("{}", v) } else { format!("0{}", v) };
    if millis {
        format!("{}:{}.{}", minutes, two(seconds as u32), two(hundredths))
    } else {
        format!("{}:{}", minutes, two(seconds as u32))
    }
}

/// GameStringManager\StringManager.cpp 0x000C7510 — thousands grouping (the millions group is printed from
/// an unset buffer in the original; it is left empty here)
fn format_cash(v: u32) -> String {
    let millions = v / 1_000_000;
    let thousands = v % 1_000_000 / 1000;
    let units = v % 1_000_000 % 1000;
    let group = |g: u32, higher: u32| {
        if g <= 100 && higher != 0 && g < 100 {
            if g < 10 {
                if g != 0 { format!("00{}", g) } else { "000".to_string() }
            } else {
                format!("0{}", g)
            }
        } else {
            format!("{}", g)
        }
    };
    let u = group(units, thousands);
    let th = group(thousands, millions);
    if millions != 0 {
        format!(",{},{}", th, u)
    } else if thousands != 0 {
        format!("{},{}", th, u)
    } else {
        u
    }
}

/// `CCRC(const char*)` / `VsStringAnsi::GetCRC` (0x002B46B0): case-sensitive crc32, seed 0; empty → 0.
pub fn string_crc(s: &str) -> u32 {
    if s.is_empty() { 0 } else { handy_crc::crc32(s.as_bytes(), 0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings_dir() -> std::path::PathBuf {
        visos::file::resolve("\\Strings")
    }

    #[test]
    fn crc_matches_index_names() {
        assert_eq!(string_crc("English"), 0x7E52_E4E2);
        assert_eq!(string_crc("Audio"), 0x1F9E_EE8C);
        assert_eq!(string_crc("GENERAL_SELECT"), 0x2FF4_2DE2);
    }

    #[test]
    fn parse_every_string_file() {
        let dir = strings_dir();
        if !dir.exists() {
            eprintln!("no game data, skipping");
            return;
        }
        let idx = StringIndex::parse(&std::fs::read(dir.join("StrIndex.idx")).unwrap()).unwrap();
        assert_eq!(idx.languages.len(), 7);
        for l in &idx.languages {
            assert_eq!(string_crc(&l.name), l.crc);
        }
        let mut files = 0;
        let mut strings = 0;
        for lang in std::fs::read_dir(&dir).unwrap().flatten() {
            if !lang.path().is_dir() {
                continue;
            }
            for f in std::fs::read_dir(lang.path()).unwrap().flatten() {
                let b = std::fs::read(f.path()).unwrap();
                let s = SectionFile::parse(&b).unwrap_or_else(|e| panic!("{:?}: {:?}", f.path(), e));
                let stem = f.path().file_stem().unwrap().to_string_lossy().into_owned();
                assert_eq!(s.section_crc, string_crc(&stem), "{:?}", f.path());
                assert!(idx.languages.iter().any(|l| l.crc == s.language_crc));
                files += 1;
                strings += s.strings.len();
            }
        }
        assert_eq!(files, 7 * 16);
        assert!(strings > 30000);
    }

    #[test]
    fn lookup_english() {
        if !strings_dir().exists() {
            return;
        }
        let mut m = CStringManager::new();
        m.set_language("English");
        m.load_index_file("StrIndex.idx").unwrap();
        assert_eq!(m.text(string_crc("GENERAL_SELECT")), STR_NOT_LOADED);
        m.load_section_name("Common").unwrap();
        m.load_section_name("Frontend").unwrap();
        m.load_section_name("Online").unwrap();
        assert_eq!(m.text(string_crc("GENERAL_SELECT")), "SELECT");
        assert_eq!(m.text(string_crc("CONNECTION_LOST_TITLE")), "Connection Lost");
        assert_eq!(m.text(0x1234_5678), STR_NOT_FOUND);
    }
}

#[cfg(test)]
mod arg_tests {
    use super::*;

    #[test]
    fn arguments_expand() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let mut m = CStringManager::new();
        m.set_language("English");
        m.load_index_file("StrIndex.idx").unwrap();
        m.load_section_name("Frontend").unwrap();
        m.set_argument(string_crc("PLAYER_CASH_TOTAL"), string_crc("dCashTotal"), 1_234_567).ok();
        let cash = m.text(string_crc("PLAYER_CASH_TOTAL"));
        assert!(cash.starts_with("Cash : "), "{}", cash);
        m.set_argument(string_crc("QUICKRACE_BESTLAPTIME"), string_crc("fLapTime"), 83.25f32.to_bits()).unwrap();
        assert_eq!(m.text(string_crc("QUICKRACE_BESTLAPTIME")), "Best Lap Time : 1:23");
        m.set_argument(string_crc("QUICKRACE_LAPS_VALUE"), string_crc("dLaps"), 3).unwrap();
        assert_eq!(m.text(string_crc("QUICKRACE_LAPS_VALUE")), "3");
        assert_eq!(format_cash(1500), "1,500");
        assert_eq!(format_cash(1005), "1,005");
    }
}
