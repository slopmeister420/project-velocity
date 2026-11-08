//! the soundtrack player (`VsSoundtrackPlayer`, 0x00054FD0..0x00056840; its .cpp has no asserts so the name is
//! not in the binary). on the xbox it enumerates the dashboard soundtracks plus the game's own "soundtrack"
//! (the MusLists master table's `.wma` files) and streams the songs through the wma decoder into two
//! dsound streams.
//!
//! the port plays a song from one of two sources:
//! - `MusicSource::Wma`: the song's `.wma` decoded in memory (cargo feature `wma`, see `wma`),
//! - `MusicSource::Bank` (default): the same song from the music banks (`music_fe.XVA`/`music.XVA`), whose
//!   events carry the MusLists track names. the banks ship with this build and decode exactly, while the
//!   available pure-rust wma decoder measures ~20 dB SNR against a reference decode.

use crate::audio::system::{AudioSystem, EmitterId, EventSlot};
use crate::vs_audio_track_list::{name_hash, VsAudioTrackList};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MusicSource {
    #[default]
    Bank,
    Wma,
}

/// one soundtrack (28 bytes: name, song ids, count +8, game disc +12, id +16, type +20, shuffle +24)
#[derive(Clone, Debug)]
pub struct Soundtrack {
    pub name: String,
    pub songs: Vec<u32>,
    pub game_disc: bool,
    pub id: u32,
    pub kind: u32,
    pub shuffle: u32,
}

/// player state (+12)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlayerState {
    #[default]
    Stopped = 0,
    Paused = 1,
    Playing = 2,
}

pub struct VsSoundtrackPlayer {
    pub source: MusicSource,
    pub state: PlayerState, // +12
    pub volume_mb: i32,     // +8
    pub soundtracks: Vec<Soundtrack>, // +20/+24
    pub soundtrack: usize,  // +28
    pub song: usize,        // +100
    pub song_id: u32,       // +104
    pub event: Option<EventSlot>,
    pub emitter: Option<EmitterId>,
    #[cfg(feature = "wma")]
    pub stream: Option<crate::wma::WmaStream>,
}

impl Default for VsSoundtrackPlayer {
    /// VsSoundtrackPlayer.cpp 0x00054FD0
    fn default() -> Self {
        Self {
            source: if std::env::var("VELOCITY_MUSIC").is_ok_and(|v| v.eq_ignore_ascii_case("wma")) {
                MusicSource::Wma
            } else {
                MusicSource::Bank
            },
            state: PlayerState::Stopped,
            volume_mb: -10000,
            soundtracks: Vec::new(),
            soundtrack: 0,
            song: 0,
            song_id: 0,
            event: None,
            emitter: None,
            #[cfg(feature = "wma")]
            stream: None,
        }
    }
}

impl VsSoundtrackPlayer {
    /// VsSoundtrackPlayer.cpp 0x00055850 / 0x000550D0 (`EnumSoundtracks` + `Init`): the game's own soundtrack (18 songs: master table
    /// indices) followed by dashboard soundtracks (none on this platform).
    pub fn init(&mut self, audio: &mut AudioSystem, emitter: EmitterId) {
        self.soundtracks = vec![Soundtrack {
            name: "Project Velocity".into(),
            songs: (0..18).collect(),
            game_disc: true,
            id: 0,
            kind: 6,
            shuffle: 0,
        }];
        self.set_song(0);
        self.emitter = Some(emitter);
        self.event = Some(audio.new_slot());
    }

    /// VsSoundtrackPlayer.cpp 0x00055CA0 SetSoundtrack
    pub fn set_soundtrack(&mut self, i: usize) {
        self.soundtrack = i;
        self.set_song(0);
    }

    /// VsSoundtrackPlayer.cpp 0x00055CE0 SetSong (`VsSoundtrack::GetSongInfo`: on the game disc the song id is the index)
    pub fn set_song(&mut self, i: usize) {
        self.song = i;
        self.song_id = i as u32;
    }

    fn song_count(&self) -> usize {
        self.soundtracks.get(self.soundtrack).map_or(0, |s| s.songs.len())
    }

    /// VsSoundtrackPlayer.cpp 0x00055700 NextSong: 1 while the player is not stopped.
    pub fn next_song(&mut self) -> i32 {
        if self.state != PlayerState::Stopped {
            return 1;
        }
        let n = self.song_count();
        // unsigned modulo by the song count (divide by zero on an empty soundtrack, as the original)
        self.set_song(if n == 0 { 0 } else { (self.song + 1) % n });
        0
    }

    /// VsSoundtrackPlayer.cpp 0x00055760 PrevSong
    pub fn prev_song(&mut self) -> i32 {
        if self.state != PlayerState::Stopped {
            return 1;
        }
        let s = if self.song == 0 { self.song_count().wrapping_sub(1) } else { self.song - 1 };
        self.song = s;
        self.set_song(s);
        0
    }

    /// VsSoundtrackPlayer.cpp 0x00055640 NextSoundtrack
    pub fn next_soundtrack(&mut self) -> i32 {
        if self.state != PlayerState::Stopped {
            return 1;
        }
        let n = self.soundtracks.len();
        self.set_soundtrack(if n == 0 { 0 } else { (self.soundtrack + 1) % n });
        0
    }

    /// VsSoundtrackPlayer.cpp 0x00055690 PrevSoundtrack
    pub fn prev_soundtrack(&mut self) -> i32 {
        if self.state != PlayerState::Stopped {
            return 1;
        }
        let s = if self.soundtrack == 0 { self.soundtracks.len().wrapping_sub(1) } else { self.soundtrack - 1 };
        self.soundtrack = s;
        self.set_soundtrack(s);
        0
    }

    /// VsSoundtrackPlayer.cpp 0x00055F90 Shutdown: close the song, drop the streams and the soundtrack table.
    pub fn shutdown(&mut self, audio: &mut AudioSystem) -> i32 {
        self.close_song(audio);
        #[cfg(feature = "wma")]
        {
            self.stream = None;
        }
        self.soundtracks.clear();
        self.state = PlayerState::Stopped;
        0
    }

    /// VsSoundtrackPlayer.cpp 0x00055310 Play
    pub fn play(&mut self, audio: &mut AudioSystem, tracks: &VsAudioTrackList) {
        match self.state {
            PlayerState::Stopped => self.open_song(audio, tracks),
            PlayerState::Paused => self.set_paused(audio, false),
            PlayerState::Playing => {}
        }
        self.state = PlayerState::Playing;
    }

    /// VsSoundtrackPlayer.cpp 0x00055390 Stop
    pub fn stop(&mut self, audio: &mut AudioSystem) {
        if self.state != PlayerState::Stopped {
            self.close_song(audio);
        }
        self.state = PlayerState::Stopped;
    }

    /// VsSoundtrackPlayer.cpp 0x00055400 Pause: 1 when not playing.
    pub fn pause(&mut self, audio: &mut AudioSystem) -> i32 {
        if self.state != PlayerState::Playing {
            return 1;
        }
        self.set_paused(audio, true);
        self.state = PlayerState::Paused;
        0
    }

    fn set_paused(&mut self, audio: &mut AudioSystem, paused: bool) {
        if let Some(ps) = self.event.and_then(|s| audio.slot_sample(s)) {
            if paused {
                audio.pause_sample(ps);
            } else {
                audio.resume_sample(ps);
            }
        }
        #[cfg(feature = "wma")]
        if let Some(s) = &self.stream {
            s.set_paused(paused);
        }
    }

    /// VsSoundtrackPlayer.cpp 0x000554B0 SetVolume: `dword_782A88[vol * master * 255]` on both streams.
    pub fn set_volume(&mut self, audio: &mut AudioSystem, vol: f32, master: f32) {
        self.volume_mb = audio.voices.volume_mb(vol * master);
        let lin = if self.volume_mb <= -10000 { 0.0 } else { 10f32.powf(self.volume_mb as f32 / 2000.0) };
        if let Some(ps) = self.event.and_then(|s| audio.slot_sample(s)) {
            // the engine applies master volume itself
            let v = if master > 0.0 { (lin / master).clamp(0.0, 1.0) } else { 0.0 };
            audio.set_sample_volume(ps, v);
        }
        #[cfg(feature = "wma")]
        if let Some(s) = &self.stream {
            s.set_gain(lin);
        }
    }

    fn master_index(&self) -> usize {
        self.soundtracks.get(self.soundtrack).and_then(|s| s.songs.get(self.song)).copied().unwrap_or(self.song_id) as usize
    }

    /// VsSoundtrackPlayer.cpp 0x00055E30 OpenSong
    fn open_song(&mut self, audio: &mut AudioSystem, tracks: &VsAudioTrackList) {
        // the game disc song index addresses the master table directly (also past the 18 enumerated songs)
        let idx = if self.soundtrack == 0 { self.song_id as usize } else { self.master_index() };
        let Some(m) = tracks.master.get(idx) else { return };
        match self.source {
            MusicSource::Bank => {
                if let (Some(e), Some(s)) = (self.emitter, self.event) {
                    audio.stop_slot(s);
                    let h = name_hash(&m.name.to_ascii_uppercase());
                    if audio.play_event(e, h, None, 0, Some(s)) != 0 {
                        log::warn!("audio: no music event {}", m.name);
                    }
                }
            }
            MusicSource::Wma => {
                #[cfg(feature = "wma")]
                {
                    self.stream = crate::wma::WmaStream::open(&m.path, audio.voices.mixer.clone());
                }
                #[cfg(not(feature = "wma"))]
                log::warn!("audio: built without the `wma` feature; {} not played", m.path);
            }
        }
    }

    /// VsSoundtrackPlayer.cpp 0x00055F00 CloseSong
    fn close_song(&mut self, audio: &mut AudioSystem) {
        if let Some(s) = self.event {
            audio.stop_slot(s);
        }
        #[cfg(feature = "wma")]
        {
            self.stream = None;
        }
    }

    /// VsSoundtrackPlayer.cpp 0x000561C0 Update: feed the streams; a finished song closes and the player stops, so
    /// `VsAudio::UpdateMusic` starts the next one.
    pub fn update(&mut self, audio: &mut AudioSystem) {
        if self.state != PlayerState::Playing {
            return;
        }
        let finished = match self.source {
            MusicSource::Bank => self.event.is_none_or(|s| audio.slot(s).is_none()),
            MusicSource::Wma => {
                #[cfg(feature = "wma")]
                {
                    self.stream.as_ref().is_none_or(|s| s.finished())
                }
                #[cfg(not(feature = "wma"))]
                true
            }
        };
        if finished {
            log::info!("audio: soundtrack {} song {} finished", self.soundtrack, self.song);
            self.close_song(audio);
            self.state = PlayerState::Stopped;
        }
    }
}
