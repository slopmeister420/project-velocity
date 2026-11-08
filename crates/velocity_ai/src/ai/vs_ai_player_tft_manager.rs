//! TFT ("trash talk") lines: the opponents' video/voice taunts, throttled so they do not overlap
//! (src/Ai/VsAIPlayerTFTManager.cpp; the instance is the static at 0x006D26E0).
//!
//! The movie player itself (0x006FC098) belongs to the frontend; the manager asks [`TftPlayer`].

/// the movie/voice player the lines are sent to.
pub trait TftPlayer {
    /// a line is currently playing (`VsMovie +0xB50`)
    fn is_playing(&self) -> bool;
    /// start `name`, optionally with the per-player vcd
    fn play(&mut self, name: &str, vcd: Option<&str>);
}

/// a player that never plays anything.
#[derive(Debug, Default)]
pub struct NullTft;
impl TftPlayer for NullTft {
    fn is_playing(&self) -> bool {
        false
    }
    fn play(&mut self, _name: &str, _vcd: Option<&str>) {}
}

/// tft event types ("MovieType" tweak).
pub mod tft {
    pub const CHIT_CHAT: u32 = 0;
    pub const GENERIC_ANGER: u32 = 1;
    pub const BEING_HIT_BIG: u32 = 2;
    pub const GENERIC_JOY: u32 = 3;
    pub const WINNING: u32 = 4;
    pub const STUNT_SUCCESS: u32 = 5;
    pub const LOST: u32 = 6;
    pub const BEING_OVERTAKEN: u32 = 7;
    pub const MAKE_MISTAKE: u32 = 8;
    pub const SPECIAL: u32 = 9;
}

#[derive(Debug, Clone)]
pub struct VsAIPlayerTFTManager {
    pub last_time: f32,          // +0x00
    pub last_volume: f32,        // +0x04
    pub vcds: Vec<String>,       // +0x08 / +0x0C
    pub debug_character: u32,    // +0x10 "TFTCharacter"
    pub debug_movie_type: u32,   // +0x18 "MovieType"
    pub debug_type_choice: u32,  // +0x20 "Type Choice"
    pub debug_play: bool,        // +0x24 "Play Movie"
    pub time_between: f32,       // +0x28 "Time Between TFTs"
    pub time_between_same: f32,  // +0x2C "Time Between Same Char TFTs"
    pub priority_player: i32,    // +0x30 index that bypasses the throttle
    /// `byte_684E54` "Movie On TFT"
    pub movie_on_tft: bool,
}

impl Default for VsAIPlayerTFTManager {
    /// VsAIDriver.cpp 0x00016770 (static constructor of 0x006D26E0)
    fn default() -> Self {
        Self {
            last_time: 0.0,
            last_volume: 0.0,
            vcds: Vec::new(),
            debug_character: 0,
            debug_movie_type: 0,
            debug_type_choice: 0,
            debug_play: false,
            time_between: f32::from_bits(0x4120_0000),
            time_between_same: f32::from_bits(0x41A0_0000),
            priority_player: -1,
            movie_on_tft: true,
        }
    }
}

impl VsAIPlayerTFTManager {
    /// line names are plain when per-player vcds are loaded.
    /// VsAIDriver.cpp 0x00015C40
    pub fn has_vcds(&self) -> bool {
        !self.vcds.is_empty()
    }

    /// VsAIDriver.cpp 0x00015B30
    pub fn get_time_between_same(&self) -> f32 {
        self.time_between_same
    }

    /// plays `name` for player slot `player` unless throttled; `speech_enabled` is `byte_6F915E`.
    /// VsAIPlayerTFTManager.cpp 0x00016940
    pub fn play_line(&mut self, name: &str, player: i32, volume: f32, time: f32, speech_enabled: bool, out: &mut dyn TftPlayer) -> bool {
        if !self.movie_on_tft {
            return false;
        }
        if player != self.priority_player && !speech_enabled {
            return false;
        }
        if out.is_playing() && self.last_volume >= volume {
            return false;
        }
        if player != self.priority_player && self.last_time != -1.0 && self.last_time + self.time_between > time {
            return false;
        }
        self.last_time = time;
        self.last_volume = volume;
        let vcd = if self.vcds.is_empty() {
            None
        } else {
            debug_assert!((player as usize) < self.vcds.len(), "vcdIndex < m_VCDCount");
            self.vcds.get(player as usize).map(|s| s.as_str())
        };
        out.play(name, vcd);
        true
    }

    /// VsAIPlayerTFTManager.cpp 0x00016B20
    pub fn play_story(&mut self, movie: &str, time: f32, speech_enabled: bool, out: &mut dyn TftPlayer) -> bool {
        let name = format!("tft/story/{movie}");
        let p = self.priority_player;
        self.play_line(&name, p, 1.0, time, speech_enabled, out)
    }
}
