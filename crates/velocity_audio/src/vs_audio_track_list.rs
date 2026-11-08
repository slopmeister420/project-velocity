//! src/Audio/VsAudioTrackList.cpp — the music playlists. `\Audio\MusLists.dfm` (text, `MDV2`) defines three
//! fixed lists of track names and the master table mapping each name to a song file; lists past the
//! third are the soundtrack player's other soundtracks.
//!
//! ```text
//! MDV2
//! 3 x { <list name>  <type>  <shuffle>  <count>  count x <track name> }
//! <master count>
//! master count x { <track name> TAB "<path>" }
//! ```
//! list type: 0 user, 1 front end, 2 in game, 3..5 game-mode specific. names are hashed
//! (`HANDY_CRC32`, case-sensitive) and the track names are also the music bank event names.

use visos::handy::handy_crc::crc32;

/// VsAudioTrackList.cpp 0x0004F9B0 GetFixedListCount: the number of lists the file defines
pub const FIXED_LISTS: usize = 3;
/// total track slots across the editable lists (`+632` starts at 150)
pub const MAX_TRACKS: i32 = 150;

pub fn name_hash(s: &str) -> u32 {
    if s.is_empty() { 0 } else { crc32(s.as_bytes(), 0) }
}

/// a list entry (268 bytes: hash, name[256], master index +260, soundtrack +264)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrackEntry {
    pub hash: u32,
    pub name: String,
    pub master: u32,     // +260 song index in the game soundtrack
    pub soundtrack: u32, // +264
}

/// one fixed list (24 bytes at `VsAudio +556`)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrackListDef {
    pub name: u32,    // +556 hash
    pub kind: u32,    // +560
    pub shuffle: u32, // +564
    pub tracks: Vec<TrackEntry>, // +568 count, +576 entries
}

/// master entry (268 bytes at `+628`)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MasterTrack {
    pub hash: u32,
    pub name: String,
    pub path: String,
}

/// the playlists in the save's options block (`dword_6F9184`, options +0x28..+0xA80, 2648 bytes): written by
/// 0x00105E10, applied by `VsAudioTrackList::SetLists` (load 0x00104F30, and `VsAudio::InitMusic` when
/// `lists[0].name != 0`).
#[derive(Clone, Debug, PartialEq)]
pub struct SavedPlaylists {
    /// {name hash, track count, type, shuffle} (`dword_6F9184..`, 16 B each)
    pub lists: [[u32; 4]; FIXED_LISTS],
    /// {hash, soundtrack, master index} for the tracks of all three lists in order (`dword_6F91B4`, 12 B each)
    pub tracks: [[u32; 3]; MAX_TRACKS as usize],
    /// {type, shuffle} of the soundtrack lists from list 3 on (`dword_6F98BC`, 8 B each; 6/0 when unused)
    pub extra: [[u32; 2]; 100],
}

impl Default for SavedPlaylists {
    fn default() -> Self {
        Self { lists: [[0; 4]; FIXED_LISTS], tracks: [[0; 3]; MAX_TRACKS as usize], extra: [[0; 2]; 100] }
    }
}

impl SavedPlaylists {
    pub const SIZE: usize = 16 * FIXED_LISTS + 12 * MAX_TRACKS as usize + 8 * 100;
    /// the block's offset in the save's 2688-byte options block (`byte_6F915C`)
    pub const OPTIONS_OFFSET: usize = 0x28;

    /// little-endian dwords, `b` = the `SIZE` bytes at options +0x28.
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut it = b.chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]));
        let mut s = Self::default();
        for v in s.lists.iter_mut().flatten().chain(s.tracks.iter_mut().flatten()).chain(s.extra.iter_mut().flatten()) {
            *v = it.next().unwrap_or(0);
        }
        s
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.lists.iter().flatten().chain(self.tracks.iter().flatten()).chain(self.extra.iter().flatten())
            .flat_map(|v| v.to_le_bytes()).collect()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VsAudioTrackList {
    pub lists: [TrackListDef; FIXED_LISTS],
    pub master: Vec<MasterTrack>, // +628
    pub free: i32,                // +632
    pub current_list: u32,        // +548
    pub current_track: i32,       // +552
    /// the playlist editor's working lists (`+572 + 24 * i`, the head pointer of each fixed list)
    pub edit: [Option<EditList>; FIXED_LISTS],
}

/// a node of the editor's doubly linked working list (276 bytes: hash, next, prev, name, master +268,
/// soundtrack +272). arena index 0 is the head sentinel `BuildList` allocates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EditNode {
    pub hash: u32,
    pub next: Option<usize>,
    pub prev: Option<usize>,
    pub name: String,
    pub master: u32,
    pub soundtrack: u32,
}

/// the arena behind one working list; freed nodes stay in `nodes` unlinked (`live` = false).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EditList {
    pub nodes: Vec<EditNode>,
    pub live: Vec<bool>,
}

impl EditList {
    fn alloc(&mut self, n: EditNode) -> usize {
        self.nodes.push(n);
        self.live.push(true);
        self.nodes.len() - 1
    }
    /// the node indices in list order (after the head)
    pub fn order(&self) -> Vec<usize> {
        let mut v = Vec::new();
        let mut p = self.nodes.first().and_then(|h| h.next);
        while let Some(i) = p {
            v.push(i);
            p = self.nodes[i].next;
        }
        v
    }
}

impl VsAudioTrackList {
    /// VsAudioTrackList.cpp 0x0004DBF0 (the file's text read line by line; `None` when it is not `MDV2`).
    pub fn parse_mus_lists(text: &str) -> Option<Self> {
        let mut lines = text.lines().map(|l| l.trim_end_matches(['\r', '\n']));
        if lines.next()? != "MDV2" {
            return None;
        }
        let mut t = VsAudioTrackList { free: MAX_TRACKS, current_track: -1, ..Default::default() };
        let int = |s: Option<&str>| s.and_then(|s| s.trim().parse::<i64>().ok()).unwrap_or(0);
        for i in 0..FIXED_LISTS {
            let name = lines.next()?;
            let l = &mut t.lists[i];
            l.name = name_hash(name);
            l.kind = int(lines.next()) as u32;
            l.shuffle = int(lines.next()) as u32;
            let count = int(lines.next()) as usize;
            for _ in 0..count {
                let n = lines.next()?;
                l.tracks.push(TrackEntry { hash: name_hash(n), name: n.to_string(), master: 0, soundtrack: 0 });
            }
            t.free -= count as i32;
        }
        let count = int(lines.next()) as usize;
        for _ in 0..count {
            let line = lines.next()?;
            // sscanf("%[^ \t] \"%[^\"]\"")
            let name: String = line.chars().take_while(|c| *c != ' ' && *c != '\t').collect();
            let rest = line[name.len()..].trim_start();
            let path = rest.strip_prefix('"').map(|r| r.split('"').next().unwrap_or("")).unwrap_or("").to_string();
            t.master.push(MasterTrack { hash: name_hash(&name), name, path });
        }
        for l in &mut t.lists {
            for e in &mut l.tracks {
                if let Some(m) = t.master.iter().position(|m| m.hash == e.hash) {
                    e.master = m as u32;
                    e.soundtrack = 0;
                }
            }
        }
        Some(t)
    }

    /// load `\Audio\MusLists.dfm` from the install.
    pub fn load() -> Option<Self> {
        let b = visos::file::platform::read(r"\Audio\MusLists.dfm").ok()?;
        Self::parse_mus_lists(&String::from_utf8_lossy(&b))
    }

    /// VsAudioTrackList.cpp 0x0004F980 GetListCount: fixed lists then one per extra soundtrack.
    pub fn list_count(&self, soundtracks: usize) -> usize {
        soundtracks + 2
    }

    /// VsAudioTrackList.cpp 0x0004F9D0 GetListType (`st_kind` gives a soundtrack's +20)
    pub fn list_type(&self, i: usize, st_kind: impl Fn(usize) -> u32) -> u32 {
        if i < FIXED_LISTS { self.lists[i].kind } else { st_kind(i - (FIXED_LISTS - 1)) }
    }

    /// VsAudioTrackList.cpp 0x0004FA60 GetListShuffle
    pub fn list_shuffle(&self, i: usize, st_shuffle: impl Fn(usize) -> u32) -> u32 {
        if i < FIXED_LISTS { self.lists[i].shuffle } else { st_shuffle(i - (FIXED_LISTS - 1)) }
    }

    /// VsAudioTrackList::SetLists 0x0004E2A0 (the list part; types/shuffles of the soundtrack lists and the
    /// restart are `VsAudio::set_lists`). an entry whose hash is not in the master table keeps an empty name.
    pub fn set_lists(&mut self, s: &SavedPlaylists) {
        self.free = MAX_TRACKS;
        let mut v = 0usize;
        for i in 0..FIXED_LISTS {
            let [name, count, kind, shuffle] = s.lists[i];
            let l = &mut self.lists[i];
            l.name = name;
            l.kind = kind;
            l.shuffle = shuffle;
            l.tracks.clear();
            for _ in 0..count {
                // the original reads past the 150 slots on a corrupt count; the port stops there
                let Some(&[hash, soundtrack, master]) = s.tracks.get(v) else { break };
                l.tracks.push(TrackEntry { hash, name: String::new(), master, soundtrack });
                v += 1;
            }
            self.free -= count as i32;
        }
        for l in &mut self.lists {
            for e in &mut l.tracks {
                if let Some(m) = self.master.iter().take_while(|m| m.hash != 0).find(|m| m.hash == e.hash) {
                    e.name = m.name.clone();
                }
            }
        }
    }

    /// SaveFileData 0x00105E10 (the gather; `st` = soundtrack (type, shuffle) by soundtrack index).
    pub fn get_lists(&self, soundtracks: usize, st: impl Fn(usize) -> (u32, u32)) -> SavedPlaylists {
        let mut s = SavedPlaylists::default();
        let mut v = 0usize;
        for (i, l) in self.lists.iter().enumerate() {
            s.lists[i] = [l.name, l.tracks.len() as u32, l.kind, l.shuffle];
            for e in &l.tracks {
                if let Some(t) = s.tracks.get_mut(v) {
                    *t = [e.hash, e.soundtrack, e.master];
                }
                v += 1;
            }
        }
        let n = self.list_count(soundtracks);
        for (k, x) in s.extra.iter_mut().enumerate() {
            let i = FIXED_LISTS + k;
            *x = if i >= n {
                [6, 0]
            } else {
                [self.list_type(i, |j| st(j).0), self.list_shuffle(i, |j| st(j).1)]
            };
        }
        s
    }

    /// VsAudioTrackList::PickList 0x0004F1A0: choose the list to play from. `in_game_mode` is `Some(track
    /// mode)` during a race (`GameInfo +136`), None in the front end. `rand` is crt rand().
    pub fn pick_list(
        &mut self,
        soundtracks: usize,
        st_kind: impl Fn(usize) -> u32,
        in_game_mode: Option<u32>,
        rand: &mut dyn FnMut() -> i32,
    ) {
        let n = self.list_count(soundtracks);
        let mut allowed: Vec<bool> = (0..n).map(|i| self.list_type(i, &st_kind) == 0).collect();
        let want = match in_game_mode {
            Some(mode) => {
                for (i, a) in allowed.iter_mut().enumerate() {
                    if self.list_type(i, &st_kind) == 2 {
                        *a = true;
                    }
                }
                match mode {
                    1 => Some(4),
                    2 => Some(3),
                    3 => Some(5),
                    _ => None,
                }
            }
            None => Some(1),
        };
        if let Some(w) = want {
            for (i, a) in allowed.iter_mut().enumerate() {
                if self.list_type(i, &st_kind) == w {
                    *a = true;
                }
            }
        }
        let cands: Vec<usize> = (0..n)
            .filter(|&i| allowed[i] && (i >= FIXED_LISTS || !self.lists[i].tracks.is_empty()))
            .collect();
        // the original divides by the candidate count unchecked
        if !cands.is_empty() {
            self.current_list = cands[(rand() % cands.len() as i32) as usize] as u32;
        }
        self.current_track = -1;
    }

    /// VsAudioTrackList.cpp 0x0004E660 BuildList: copy fixed list `l` into a fresh editable linked list.
    pub fn build_list(&mut self, l: usize) {
        let mut e = EditList::default();
        e.alloc(EditNode::default());
        let mut tail = 0usize;
        for t in &self.lists[l].tracks {
            let n = e.alloc(EditNode {
                hash: t.hash,
                next: None,
                prev: Some(tail),
                name: t.name.clone(),
                master: t.master,
                soundtrack: t.soundtrack,
            });
            e.nodes[tail].next = Some(n);
            tail = n;
        }
        self.edit[l] = Some(e);
    }

    /// VsAudioTrackList.cpp 0x0004E8C0 AddTrackById: insert a track by name hash after node `after`
    /// (`None` = null, fails). the master-table search is bounded by `free` (the original reuses +632).
    pub fn add_track_by_id(&mut self, l: usize, after: Option<usize>, hash: u32) -> bool {
        let (Some(after), Some(_)) = (after, self.edit[l].as_ref()) else { return false };
        if self.free == 0 {
            return false;
        }
        self.free -= 1;
        let mut n = EditNode { hash, ..Default::default() };
        for i in 0..self.free.max(0) as usize {
            let Some(m) = self.master.get(i) else { break };
            if m.hash == hash {
                n.name = m.name.clone();
                n.master = i as u32;
                n.soundtrack = 0;
                break;
            }
        }
        self.link_after(l, after, n);
        true
    }

    /// VsAudioTrackList.cpp 0x0004E9F0 AddTrackByIndex: insert song `index` counted across the soundtracks
    /// from soundtrack 1 on (`songs[k]` = song count of soundtrack k).
    pub fn add_track_by_index(&mut self, l: usize, after: Option<usize>, mut index: u32, songs: &[usize]) -> bool {
        let (Some(after), Some(_)) = (after, self.edit[l].as_ref()) else { return false };
        if self.free == 0 {
            return false;
        }
        self.free -= 1;
        let mut n = EditNode { hash: u32::MAX, ..Default::default() };
        let mut k = 1usize;
        while k < songs.len() {
            let c = songs[k] as u32;
            if c >= index {
                n.soundtrack = k as u32;
                n.master = index;
                break;
            }
            index -= c;
            k += 1;
        }
        self.link_after(l, after, n);
        true
    }

    fn link_after(&mut self, l: usize, after: usize, mut n: EditNode) {
        let e = self.edit[l].as_mut().unwrap();
        let old = e.nodes[after].next;
        n.next = old;
        n.prev = Some(after);
        let i = e.alloc(n);
        e.nodes[after].next = Some(i);
        if let Some(o) = old {
            e.nodes[o].prev = Some(i);
        }
    }

    /// VsAudioTrackList.cpp 0x0004EB20 RemoveTrack: delete the node after `after`.
    pub fn remove_track(&mut self, l: usize, after: Option<usize>) -> bool {
        let (Some(after), Some(e)) = (after, self.edit[l].as_mut()) else { return false };
        let Some(v4) = e.nodes[after].next else { return false };
        self.free += 1;
        let v3 = e.nodes[v4].next;
        e.nodes[after].next = v3;
        if let Some(v3) = v3 {
            e.nodes[v3].prev = Some(after);
        }
        e.live[v4] = false;
        true
    }

    /// VsAudioTrackList.cpp 0x0004EBE0 Shuffle: close the list into a ring, then repeatedly step
    /// `rand() % 20` nodes round the ring and move that node to the new tail.
    pub fn shuffle(&mut self, l: usize, rand: &mut dyn FnMut() -> i32) {
        let Some(e) = self.edit[l].as_mut() else { return };
        let Some(first) = e.nodes[0].next else { return };
        let mut v8 = Some(0usize);
        let mut count = 0;
        while let Some(p) = v8 {
            if let Some(nx) = e.nodes[p].next {
                count += 1;
                v8 = Some(nx);
            } else {
                e.nodes[p].next = Some(first);
                e.nodes[first].prev = Some(p);
                v8 = None;
            }
        }
        let mut v9 = first;
        let mut v6 = 0usize;
        e.nodes[0].next = None;
        for _ in 0..count {
            let mut j = rand() % 20;
            while j > 0 {
                v9 = e.nodes[v9].next.unwrap();
                j -= 1;
            }
            let (p, n) = (e.nodes[v9].prev.unwrap(), e.nodes[v9].next.unwrap());
            e.nodes[p].next = Some(n);
            e.nodes[n].prev = Some(p);
            e.nodes[v6].next = Some(v9);
            e.nodes[v9].prev = Some(v6);
            v9 = e.nodes[v9].next.unwrap();
            v6 = e.nodes[v6].next.unwrap();
            e.nodes[v6].next = None;
        }
    }

    /// VsAudioTrackList.cpp 0x0004ED80 CommitList: free the working list; with `store` the fixed list is
    /// rebuilt from it (the free-only form walks the old count, here the list itself).
    pub fn commit_list(&mut self, l: usize, store: bool) {
        let Some(e) = self.edit[l].take() else { return };
        if store {
            self.lists[l].tracks = e
                .order()
                .into_iter()
                .map(|i| {
                    let n = &e.nodes[i];
                    TrackEntry { hash: n.hash, name: n.name.clone(), master: n.master, soundtrack: n.soundtrack }
                })
                .collect();
        }
    }

    /// VsAudioTrackList.cpp 0x0004F0A0 GetTrackId: hash of the `n`th track of the working list, 0 past the end.
    pub fn get_track_id(&self, l: usize, n: u32) -> u32 {
        self.get_track(l, n).map_or(0, |i| self.edit[l].as_ref().unwrap().nodes[i].hash)
    }

    /// VsAudioTrackList.cpp 0x0004F120 GetTrack: node index of the `n`th track of the working list.
    pub fn get_track(&self, l: usize, n: u32) -> Option<usize> {
        let e = self.edit[l].as_ref()?;
        let mut p = e.nodes[0].next;
        let mut i = 0;
        while let Some(x) = p {
            if i > n {
                break;
            }
            if i == n {
                return Some(x);
            }
            p = e.nodes[x].next;
            i += 1;
        }
        None
    }

    /// VsAudioTrackList.cpp 0x0004FAD0 SetListType; `set_st(soundtrack, v)` for lists past the fixed ones.
    pub fn set_list_type(&mut self, i: usize, v: u32, set_st: &mut dyn FnMut(usize, u32)) {
        if i < FIXED_LISTS {
            self.lists[i].kind = v;
        } else {
            set_st(i - (FIXED_LISTS - 1), v);
        }
    }

    /// VsAudioTrackList.cpp 0x0004FB50 SetListShuffle
    pub fn set_list_shuffle(&mut self, i: usize, v: u32, set_st: &mut dyn FnMut(usize, u32)) {
        if i < FIXED_LISTS {
            self.lists[i].shuffle = v;
        } else {
            set_st(i - (FIXED_LISTS - 1), v);
        }
    }

    /// VsAudioTrackList.cpp 0x0004FBD0 FreeLists: drop the fixed lists' tracks (and the master table with `master`).
    pub fn free_lists(&mut self, master: bool) {
        for l in &mut self.lists {
            l.tracks.clear();
        }
        if !self.master.is_empty() && master {
            self.master.clear();
        }
    }

    /// VsAudioTrackList.cpp 0x0004F010 GetListName: the name hash of list `i`.
    pub fn get_list_name(&self, i: usize) -> u32 {
        self.lists[i].name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playlist_editor() {
        let mut t = VsAudioTrackList::load().expect("MusLists.dfm");
        let orig: Vec<u32> = t.lists[0].tracks.iter().map(|e| e.hash).collect();
        t.build_list(0);
        assert_eq!((0..orig.len() as u32).map(|i| t.get_track_id(0, i)).collect::<Vec<_>>(), orig);
        assert_eq!(t.get_track_id(0, orig.len() as u32), 0);
        // shuffle keeps the multiset
        let mut s = 7u32;
        let mut rnd = || {
            s = s.wrapping_mul(214013).wrapping_add(2531011);
            ((s >> 16) & 0x7fff) as i32
        };
        t.shuffle(0, &mut rnd);
        let mut got: Vec<u32> = (0..orig.len() as u32).map(|i| t.get_track_id(0, i)).collect();
        assert_eq!(t.get_track_id(0, orig.len() as u32), 0);
        got.sort();
        let mut want = orig.clone();
        want.sort();
        assert_eq!(got, want);
        // add by id after the head, remove it again
        let free = t.free;
        let h = t.master[3].hash;
        assert!(t.add_track_by_id(0, Some(0), h));
        assert_eq!((t.get_track_id(0, 0), t.free), (h, free - 1));
        assert!(t.remove_track(0, Some(0)));
        assert_eq!(t.free, free);
        assert!(!t.remove_track(0, None));
        // commit stores the (shuffled) order back
        t.commit_list(0, true);
        assert!(t.edit[0].is_none());
        let mut after: Vec<u32> = t.lists[0].tracks.iter().map(|e| e.hash).collect();
        after.sort();
        assert_eq!(after, want);
    }

    #[test]
    fn shipped_mus_lists() {
        let t = VsAudioTrackList::load().expect("MusLists.dfm");
        assert_eq!(t.lists.iter().map(|l| l.tracks.len()).collect::<Vec<_>>(), vec![11, 10, 0]);
        assert_eq!((t.lists[0].kind, t.lists[1].kind, t.lists[2].kind), (1, 2, 0));
        assert_eq!(t.master.len(), 21);
        assert_eq!(t.free, MAX_TRACKS - 21);
        for l in &t.lists {
            for e in &l.tracks {
                let m = &t.master[e.master as usize];
                assert_eq!(m.name, e.name);
                assert!(visos::file::resolve(&m.path).exists(), "{}", m.path);
            }
        }
        let mut t2 = t.clone();
        let mut r = 7;
        t2.pick_list(1, |_| 0, None, &mut || { r += 1; r });
        assert_eq!(t2.current_list, 0);
        t2.pick_list(1, |_| 0, Some(0), &mut || 3);
        assert_eq!(t2.current_list, 1);
    }

    /// gather (0x00105E10) -> bytes -> SetLists (0x0004E2A0) -> gather is identical; names come back from the master.
    #[test]
    fn saved_playlists_round_trip() {
        let t = VsAudioTrackList::load().expect("MusLists.dfm");
        let s = t.get_lists(3, |j| if j == 1 { (0, 1) } else { (6, 0) });
        assert_eq!(s.extra[0], [0, 1]);
        assert_eq!(s.extra[1], [6, 0]);
        let b = s.to_bytes();
        assert_eq!(b.len(), SavedPlaylists::SIZE);
        assert_eq!(SavedPlaylists::OPTIONS_OFFSET + SavedPlaylists::SIZE, 2688);
        let s2 = SavedPlaylists::from_bytes(&b);
        assert_eq!(s2, s);
        let mut t2 = t.clone();
        for l in &mut t2.lists {
            l.tracks.clear();
        }
        t2.set_lists(&s2);
        assert_eq!(t2.lists, t.lists);
        assert_eq!(t2.free, t.free);
    }
}
