//! loading and unloading of sound groups (`.XVG`) and sample banks (`.XVA`), and the sorted event look-up
//! table. visos keeps one function per file; the files here are AUDIO_LoadGroup.cpp, AUDIO_LoadBank.cpp,
//! AUDIO_UnloadGroup.cpp, AUDIO_UnloadBank.cpp and the `_AUDIO_*` helpers (names inferred, no asserts).

use std::sync::{Arc, Weak};

use visos::handy::handy_crc::crc32;

use super::mixer::SampleData;
use super::system::*;
use super::xva::{self, XvaBank, XvaSampleHeader};
use super::xvg::XvgGroup;

/// `VsStringAnsi::Hash` of an upper-cased path (`strupr` 0x0041E950, then `HANDY_CRC32(s, len, 0)`).
pub fn path_hash(path: &str) -> u32 {
    let up = path.to_ascii_uppercase();
    if up.is_empty() { 0 } else { crc32(up.as_bytes(), 0) }
}

/// the directory part of a game path (everything before the last separator).
fn dir_of(path: &str) -> &str {
    match path.rfind(['\\', '/']) {
        Some(i) => &path[..i],
        None => "",
    }
}

impl AudioSystem {
    /// _AUDIO_FindEvent.cpp 0x002BB3E0: binary search of the event table; leaves the last probe in
    /// `last_search` for `_AUDIO_AddEventLookup`.
    pub fn find_event(&mut self, hash: u32) -> i32 {
        let mut lo: i32 = 0;
        let mut hi: i32 = self.event_table.len() as i32 - 1;
        self.last_search = -1;
        while lo <= hi {
            self.last_search = lo + (hi - lo) / 2;
            let h = self.event_table[self.last_search as usize].hash;
            if h <= hash {
                if h >= hash {
                    return self.last_search;
                }
                lo = self.last_search + 1;
            } else {
                hi = self.last_search - 1;
            }
        }
        -1
    }

    /// _AUDIO_AddEventLookup.cpp 0x002BB480: sorted insert after a failed (or successful) search.
    fn add_event_lookup(&mut self, group: GroupId, event: usize) -> i32 {
        let hash = self.groups.get(group).unwrap().events[event].hash;
        let n = self.event_table.len() as i32;
        let mut at = self.find_event(hash);
        if at < 0 {
            let ls = self.last_search;
            if ls < 0 || ls >= n {
                at = if n != 0 {
                    if self.event_table[0].hash <= hash {
                        if self.event_table[n as usize - 1].hash < hash { n } else { at }
                    } else {
                        0
                    }
                } else {
                    0
                };
            } else if self.event_table[ls as usize].hash >= hash {
                if self.event_table[ls as usize].hash > hash {
                    at = ls;
                }
            } else {
                at = ls + 1;
            }
        }
        // the original only ever reaches here with a valid slot; a -1 would write before the table
        let at = at.max(0) as usize;
        if self.event_table.len() >= self.params.events as usize {
            return err::NO_MEMORY;
        }
        self.event_table.insert(at, EventLookup { hash, group: Some(group), event: EventRef { group, event } });
        0
    }

    /// AUDIO_GetEventSounds.cpp 0x002BC2F0: the first sound of an event by hash.
    pub fn event_first_sound(&mut self, hash: u32) -> Option<&super::system::GSound> {
        let i = self.find_event(hash);
        if i < 0 {
            return None;
        }
        let l = self.event_table[i as usize];
        l.group?;
        self.groups.get(l.event.group)?.events.get(l.event.event)?.sounds.first()
    }

    /// AUDIO_GetEventPitchMin 0x002BC360
    pub fn event_pitch_min(&mut self, hash: u32) -> f32 {
        self.event_first_sound(hash).map(|s| s.def.pitch_min).unwrap_or(0.0)
    }

    /// AUDIO_GetEventPitchMax 0x002BC380
    pub fn event_pitch_max(&mut self, hash: u32) -> f32 {
        self.event_first_sound(hash).map(|s| s.def.pitch_max).unwrap_or(0.0)
    }

    /// _AUDIO_FindGroup.cpp 0x00316EC0
    pub fn find_group(&self, path: &str) -> Option<GroupId> {
        let h = path_hash(&format!("{path}.XVG"));
        self.group_list.iter().copied().find(|&g| self.groups.get(g).is_some_and(|g| g.name_hash == h))
    }

    /// _AUDIO_FindBank.cpp 0x003424B0: Some((bank, usable)); a bank still flagged 0x2000 is not usable.
    pub fn find_bank(&self, path: &str) -> Option<(BankId, bool)> {
        let h = path_hash(&format!("{path}.XVA"));
        self.bank_list
            .iter()
            .copied()
            .find(|&b| self.banks.get(b).is_some_and(|b| b.name_hash == h))
            .map(|b| (b, self.banks.get(b).unwrap().flags & 0x2000 == 0))
    }

    /// AUDIO_LoadBank.cpp 0x00316450. `flags & 1` keeps the file's residency flags, otherwise the caller's
    /// flags replace them.
    pub fn load_bank(&mut self, path: &str, flags: u32) -> Result<BankId, i32> {
        if !self.initialised {
            return Err(not_init());
        }
        let found = self.find_bank(path);
        if let Some((b, true)) = found {
            self.banks.get_mut(b).unwrap().refs += 1;
            return Ok(b);
        }
        let file = visos::file::resolve(&format!("{path}.XVA"));
        let bytes = std::fs::read(&file).map_err(|_| err::BAD_FILE)?;
        let (file_flags, headers) = match XvaBank::parse_header(&bytes) {
            Ok(x) => x,
            Err(xva::XvaError::Magic(_)) => return Err(err::BAD_FILE),
            Err(xva::XvaError::Version(_)) => return Err(err::VERSION),
            Err(_) => return Err(err::READ),
        };
        let bank_flags = if flags & 1 != 0 { file_flags } else { flags | (file_flags & 0xFFF8_FF00) };
        let header_at = self.sample_headers.alloc(headers.len() as u32).ok_or(err::NO_MEMORY)?;
        let streamed = bank_flags & xva::BANK_RESIDENCY_MASK == xva::BANK_STREAMED;
        let mut bank = Bank {
            flags: bank_flags,
            refs: 1,
            headers,
            resident: Vec::new(),
            streamed: Vec::new(),
            file: file.clone(),
            name_hash: path_hash(&format!("{path}.XVA")),
            header_at,
        };
        if streamed {
            bank.streamed = vec![Weak::new(); bank.headers.len()];
        } else {
            // _AUDIO_LoadBankDataM 0x003422F0: one block from the first sample's offset, 4-byte aligned samples
            let total: usize = bank.headers.iter().map(|h| (h.size as usize + 3) & !3).sum();
            let start = bank.headers.first().map(|h| h.offset as usize).unwrap_or(0);
            let Some(block) = bytes.get(start..start + total) else {
                self.sample_headers.free(header_at);
                return Err(err::READ);
            };
            let mut at = 0;
            for h in &bank.headers {
                bank.resident.push(Arc::new(SampleData {
                    bytes: block[at..at + h.size as usize].to_vec(),
                    format: h.format,
                }));
                at += (h.size as usize + 3) & !3;
            }
        }
        if bank.flags & 0x2000 != 0 {
            bank.flags &= !0x2000;
        } else {
            let res = bank.flags & xva::BANK_RESIDENCY_MASK;
            for h in &mut bank.headers {
                h.format |= res;
            }
        }
        match found {
            // a 0x2000 bank is re-read in place
            Some((b, false)) => {
                *self.banks.get_mut(b).unwrap() = bank;
                Ok(b)
            }
            _ => {
                let Some(id) = self.banks.alloc(bank) else {
                    self.sample_headers.free(header_at);
                    return Err(err::NO_MEMORY);
                };
                self.bank_list.insert(0, id);
                Ok(id)
            }
        }
    }

    /// AUDIO_UnloadBank.cpp 0x00313C10
    pub fn unload_bank(&mut self, bank: BankId, force: bool) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(b) = self.banks.get_mut(bank) else { return err::BAD_BANK };
        if !force {
            b.refs -= 1;
            if b.refs != 0 {
                return 0;
            }
        }
        let Some(pos) = self.bank_list.iter().position(|&x| x == bank) else { return err::FAILED };
        self.bank_list.remove(pos);
        let b = self.banks.free(bank).unwrap();
        self.sample_headers.free(b.header_at);
        0
    }

    /// sample data of a bank sample for a voice: resident data, or a streamed sample read from the bank file.
    pub(crate) fn sample_data(&mut self, bank: BankId, index: usize) -> Option<Arc<SampleData>> {
        let b = self.banks.get_mut(bank)?;
        if let Some(d) = b.resident.get(index) {
            return Some(d.clone());
        }
        let w = b.streamed.get(index)?;
        if let Some(d) = w.upgrade() {
            return Some(d);
        }
        let h: XvaSampleHeader = *b.headers.get(index)?;
        let bytes = read_range(&b.file, h.offset as u64, h.size as usize)?;
        let d = Arc::new(SampleData { bytes, format: h.format });
        b.streamed[index] = Arc::downgrade(&d);
        Some(d)
    }

    /// AUDIO_LoadGroup.cpp 0x002BD6C0. `path` without extension (`\Audio\GUI\FE_GUI`); `preloaded` reads the
    /// descriptor from the `allSVG.vcd` preload when it holds it.
    pub fn load_group(&mut self, path: &str, preloaded: Option<&visos::file::Vcd>) -> Result<GroupId, i32> {
        if !self.initialised {
            return Err(not_init());
        }
        let existing = self.find_group(path);
        let base = &path[path.rfind(['\\', '/']).map(|i| i + 1).unwrap_or(0)..];
        let bytes = match preloaded.and_then(|v| v.get(&format!("{base}.XVG"))) {
            Some(b) => b.to_vec(),
            None => visos::file::platform::read(&format!("{path}.XVG")).map_err(|_| err::BAD_FILE)?,
        };
        if bytes.len() < 8 || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) != 3 {
            return Err(err::FAILED);
        }
        let (xg, _) = XvgGroup::from_bytes(&bytes).map_err(|_| err::FAILED)?;
        let gid = match existing {
            Some(g) => g,
            None => self
                .groups
                .alloc(Group {
                    flags: 0,
                    refs: 1,
                    category_mask: 0,
                    volume: 0.0,
                    events: Vec::new(),
                    banks: Vec::new(),
                    name_hash: 0,
                    allocs: GroupAllocs::default(),
                })
                .ok_or(err::FAILED)?,
        };
        {
            let g = self.groups.get_mut(gid).unwrap();
            g.category_mask = xg.category_mask;
            g.volume = xg.volume;
            g.name_hash = path_hash(&format!("{path}.XVG"));
        }
        if self.groups.get(gid).unwrap().allocs.banks.is_none() {
            let at = self.group_banks.alloc(xg.banks.len() as u32);
            if at.is_none() {
                if existing.is_none() {
                    self.groups.free(gid);
                }
                return Err(err::NO_MEMORY);
            }
            let g = self.groups.get_mut(gid).unwrap();
            g.allocs.banks = at;
            g.banks = vec![None; xg.banks.len()];
        }
        let xvg_dir = dir_of(path).to_string();
        for (j, br) in xg.banks.iter().enumerate() {
            let rel = if br.dir.is_empty() { br.name.clone() } else { format!("{}/{}", br.dir, br.name) };
            let full = join_path(&rel, &xvg_dir).replace('/', "\\");
            match self.load_bank(&full, 1) {
                Ok(b) => self.groups.get_mut(gid).unwrap().banks[j] = Some(b),
                Err(e) => {
                    log::warn!("audio: bank {full} of {path}: {e:#x}");
                    self.unload_group(Some(gid), false);
                    return Err(e);
                }
            }
        }
        if existing.is_some() {
            return Ok(gid);
        }
        // events, sounds, samples and chains with the original's pool accounting
        let Some(ev_at) = self.group_events.alloc(xg.events.len() as u32) else {
            self.unload_group(Some(gid), false);
            return Err(err::NO_MEMORY);
        };
        self.groups.get_mut(gid).unwrap().allocs.events = Some(ev_at);
        let bank_ids = self.groups.get(gid).unwrap().banks.clone();
        let mut events = Vec::with_capacity(xg.events.len());
        for e in &xg.events {
            let mut sounds = Vec::new();
            if !e.sounds.is_empty() {
                let Some(at) = self.group_sounds.alloc(e.sounds.len() as u32) else {
                    self.unload_group(Some(gid), false);
                    return Err(err::NO_MEMORY);
                };
                self.groups.get_mut(gid).unwrap().allocs.sounds.push(at);
            }
            for s in &e.sounds {
                if !s.samples.is_empty() {
                    let Some(at) = self.group_samples.alloc(s.samples.len() as u32) else {
                        self.unload_group(Some(gid), false);
                        return Err(err::NO_MEMORY);
                    };
                    self.groups.get_mut(gid).unwrap().allocs.samples.push(at);
                }
                if !s.chains.is_empty() {
                    let Some(at) = self.group_chains.alloc(s.chains.len() as u32) else {
                        log::warn!("Failed to get Chain Structs, aborting load of {path}");
                        self.unload_group(Some(gid), false);
                        return Err(err::NO_MEMORY);
                    };
                    self.groups.get_mut(gid).unwrap().allocs.chains.push(at);
                }
                sounds.push(GSound {
                    def: s.clone(),
                    samples: s
                        .samples
                        .iter()
                        .map(|x| GSample {
                            weight: x.weight,
                            volume: x.volume,
                            bank: bank_ids.get(x.bank as usize).copied().flatten(),
                            sample: x.sample,
                        })
                        .collect(),
                    chains: s.chains.clone(),
                    last_sample: -1,
                    last_chain: -1,
                });
            }
            events.push(GEvent { hash: e.hash, unk8: e.unk8, flags: e.flags, unk16: e.unk16, sounds, last: -1 });
        }
        self.groups.get_mut(gid).unwrap().events = events;
        for i in 0..xg.events.len() {
            let r = self.add_event_lookup(gid, i);
            if r != 0 {
                log::warn!("audio: event table full loading {path}");
            }
        }
        self.group_list.insert(0, gid);
        Ok(gid)
    }

    /// AUDIO_UnloadGroup.cpp 0x002BC920. `None` = every group (`-1`).
    pub fn unload_group(&mut self, group: Option<GroupId>, force: bool) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(gid) = group else {
            while let Some(&g) = self.group_list.first() {
                let r = self.unload_group(Some(g), true);
                if r != 0 {
                    return r;
                }
            }
            self.event_table.clear();
            return 0;
        };
        for l in &mut self.event_table {
            if l.group == Some(gid) {
                l.group = None;
            }
        }
        let Some(g) = self.groups.get(gid) else { return err::FAILED };
        let banks: Vec<BankId> = g.banks.iter().flatten().copied().collect();
        let allocs = g.allocs.clone();
        for b in banks {
            self.unload_bank(b, force);
        }
        if let Some(p) = self.group_list.iter().position(|&x| x == gid) {
            self.group_list.remove(p);
        }
        for &a in allocs.chains.iter().rev() {
            self.group_chains.free(a);
        }
        for &a in allocs.samples.iter().rev() {
            self.group_samples.free(a);
        }
        for &a in allocs.sounds.iter().rev() {
            self.group_sounds.free(a);
        }
        if let Some(a) = allocs.events {
            self.group_events.free(a);
        }
        if let Some(a) = allocs.banks {
            self.group_banks.free(a);
        }
        self.groups.free(gid);
        0
    }

    /// event data of a look-up entry, if its group is still loaded.
    pub fn event(&self, r: EventRef) -> Option<&GEvent> {
        self.groups.get(r.group)?.events.get(r.event)
    }
    pub fn event_mut(&mut self, r: EventRef) -> Option<&mut GEvent> {
        self.groups.get_mut(r.group)?.events.get_mut(r.event)
    }
}

/// _AUDIO_JoinPath.cpp 0x00316B90 reduced to what the loader feeds it: a relative path (with `./`, `../`)
/// against a base directory, using `/`.
fn join_path(rel: &str, base: &str) -> String {
    let rel = rel.replace('\\', "/");
    let mut base: Vec<&str> = base.split(['\\', '/']).collect();
    let mut r = rel.as_str();
    loop {
        if let Some(rest) = r.strip_prefix("../") {
            base.pop();
            r = rest;
        } else if let Some(rest) = r.strip_prefix("./") {
            r = rest;
        } else {
            break;
        }
    }
    if rel.starts_with('/') {
        return rel;
    }
    let mut s = base.join("/");
    if !s.is_empty() {
        s.push('/');
    }
    s.push_str(r);
    s
}

fn read_range(path: &std::path::Path, offset: u64, len: usize) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(path).ok()?;
    f.seek(SeekFrom::Start(offset)).ok()?;
    let mut v = vec![0; len];
    f.read_exact(&mut v).ok()?;
    Some(v)
}
