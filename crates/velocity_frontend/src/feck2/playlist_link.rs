//! the request / mirror channel between the plain `Feck2` screens and audio's bevy-side playlist state
//! (`VisosAudio.game.tracks` = `VsAudioTrackList` at `dword_6D4558`, `.soundtrack` = `VsSoundtrackPlayer` at
//! `dword_6D45C4`). the Quick Race music playlist screen (FEQuickRaceTextScreen.cpp 0x0022B7E0..0x0022D7B0) and
//! the hangout's DJ menu (0x0021C2A0) call those objects directly in the original.
//!
//! mirror: `tracks` / `soundtracks` are copies the screens read and edit like the originals. `run_frontend`
//! copies audio -> mirror every frame (`sync_in`, skipped while `dirty`) and mirror -> audio after the frame
//! when a screen set `dirty` (`sync_out`: lists, kinds, shuffles, working lists; the playing list / track stay
//! audio's). requests: `PlaylistRequest`s the screens raise (play a song, save the lists) are executed by the
//! bevy side after `sync_out`. `rand` is the frontend's own CRT rand for `Shuffle` (the original's is global).

use velocity_audio::vs_audio_track_list::{VsAudioTrackList, FIXED_LISTS};
use velocity_audio::VisosAudio;
use visos::handy::crt_rand::CrtRand;

/// a `VsSoundtrack` as the screens read it (28 bytes: name, songs, type +20, shuffle +24)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlaylistSoundtrack {
    pub name: String,
    pub kind: u32,
    pub shuffle: u32,
    /// song names (the game soundtrack's are the master table's, never listed by the screens)
    pub songs: Vec<String>,
}

/// what a list is called: a string id (fixed lists) or raw text (a soundtrack's name)
#[derive(Clone, Debug, PartialEq)]
pub enum ListName {
    Hash(u32),
    Text(String),
}

/// calls the screens make on `VsAudio`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaylistRequest {
    /// VsAudio.cpp 0x0004F540 `PlayTrack(1, 0, soundtrack, song)` / `NextTrack(1, node)` (0x0004F4B0, which plays the node)
    PlayTrack { soundtrack: usize, song: usize },
    /// SaveFileData 0x00105E10 (`VsAudio::get_lists`: the playlists into the save's options block)
    SaveLists,
}

#[derive(Clone, Debug)]
pub struct PlaylistLink {
    pub tracks: VsAudioTrackList,
    pub soundtracks: Vec<PlaylistSoundtrack>,
    pub dirty: bool,
    pub requests: Vec<PlaylistRequest>,
    /// AudioPlaylistScreen +0: the list being edited (FEHangoutScreen 0x0021C370 sets it, `SetList` 0x0022BB20)
    pub list: usize,
    pub rand: CrtRand,
}

impl Default for PlaylistLink {
    fn default() -> Self {
        Self {
            tracks: VsAudioTrackList::default(),
            // VsSoundtrackPlayer::Init 0x000550D0: the game's own soundtrack only
            soundtracks: vec![PlaylistSoundtrack { name: "Project Velocity".into(), kind: 6, shuffle: 0, songs: Vec::new() }],
            dirty: false,
            requests: Vec::new(),
            list: 0,
            rand: CrtRand::default(),
        }
    }
}

impl PlaylistLink {
    /// VsAudioTrackList.cpp 0x0004F9B0 `GetFixedListCount`
    pub fn fixed_list_count(&self) -> usize {
        FIXED_LISTS
    }

    /// VsAudioTrackList.cpp 0x0004F980 `GetListCount`
    pub fn list_count(&self) -> usize {
        self.tracks.list_count(self.soundtracks.len().max(1))
    }

    /// VsSoundtrackPlayer.cpp `GetSoundtrack`
    pub fn soundtrack(&self, i: usize) -> Option<&PlaylistSoundtrack> {
        self.soundtracks.get(i)
    }

    /// VsAudioTrackList.cpp 0x0004F010 `GetListName` for fixed lists, the soundtrack's name for the others
    pub fn list_name(&self, i: usize) -> ListName {
        if i < FIXED_LISTS {
            ListName::Hash(self.tracks.get_list_name(i))
        } else {
            ListName::Text(self.soundtracks.get(i - (FIXED_LISTS - 1)).map(|s| s.name.clone()).unwrap_or_default())
        }
    }

    /// VsAudioTrackList.cpp 0x0004F9D0 `GetListType`
    pub fn list_type(&self, i: usize) -> u32 {
        self.tracks.list_type(i, |j| self.soundtracks.get(j).map_or(6, |s| s.kind))
    }

    /// VsAudioTrackList.cpp 0x0004FA60 `GetListShuffle`
    pub fn list_shuffle(&self, i: usize) -> u32 {
        self.tracks.list_shuffle(i, |j| self.soundtracks.get(j).map_or(0, |s| s.shuffle))
    }

    /// VsAudioTrackList.cpp 0x0004FAD0 `SetListType`
    pub fn set_list_type(&mut self, i: usize, v: u32) {
        let st = &mut self.soundtracks;
        self.tracks.set_list_type(i, v, &mut |j, v| {
            if let Some(s) = st.get_mut(j) {
                s.kind = v;
            }
        });
        self.dirty = true;
    }

    /// VsAudioTrackList.cpp 0x0004FB50 `SetListShuffle`
    pub fn set_list_shuffle(&mut self, i: usize, v: u32) {
        let st = &mut self.soundtracks;
        self.tracks.set_list_shuffle(i, v, &mut |j, v| {
            if let Some(s) = st.get_mut(j) {
                s.shuffle = v;
            }
        });
        self.dirty = true;
    }

    /// song counts per soundtrack (`AddTrackByIndex`'s walk over soundtracks 1..)
    pub fn song_counts(&self) -> Vec<usize> {
        self.soundtracks.iter().map(|s| s.songs.len()).collect()
    }

    /// audio -> mirror (every frame, unless a screen edit is waiting to go back)
    pub fn sync_in(&mut self, a: &VisosAudio) {
        if self.dirty {
            return;
        }
        if self.tracks != a.game.tracks {
            self.tracks = a.game.tracks.clone();
        }
        let master = &a.game.tracks.master;
        let sts: Vec<PlaylistSoundtrack> = a
            .game
            .soundtrack
            .soundtracks
            .iter()
            .enumerate()
            .map(|(i, s)| PlaylistSoundtrack {
                name: s.name.clone(),
                kind: s.kind,
                shuffle: s.shuffle,
                songs: if i == 0 {
                    Vec::new()
                } else {
                    s.songs.iter().map(|&id| master.get(id as usize).map_or_else(|| id.to_string(), |m| m.name.clone())).collect()
                },
            })
            .collect();
        if !sts.is_empty() && self.soundtracks != sts {
            self.soundtracks = sts;
        }
    }

    /// mirror -> audio, then the requests; `play` runs a `PlayTrack` (the bevy side writes `AudioCommand::PlayTrack`)
    pub fn sync_out(&mut self, a: &mut VisosAudio, mut play: impl FnMut(usize, usize)) {
        if std::mem::take(&mut self.dirty) {
            let t = &mut a.game.tracks;
            let (cl, ct) = (t.current_list, t.current_track);
            *t = self.tracks.clone();
            t.current_list = cl;
            t.current_track = ct;
            for (i, s) in self.soundtracks.iter().enumerate() {
                if let Some(d) = a.game.soundtrack.soundtracks.get_mut(i) {
                    d.kind = s.kind;
                    d.shuffle = s.shuffle;
                }
            }
        }
        for r in std::mem::take(&mut self.requests) {
            match r {
                PlaylistRequest::PlayTrack { soundtrack, song } => play(soundtrack, song),
                PlaylistRequest::SaveLists => {
                    a.game.get_lists();
                }
            }
        }
    }
}
