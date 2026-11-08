//! Audio engine and game sound (visos/Audio, src/Audio).

use bevy::prelude::*;

pub mod audio;
pub mod audio_tweaks;
pub mod input;
pub mod plugin;
pub mod trace;
pub mod vs_audio;
pub mod vs_audio_events;
pub mod vs_audio_rev_bands;
pub mod vs_audio_track_list;
pub mod vs_audio_vehics;
pub mod vs_soundtrack;
#[cfg(feature = "wma")]
pub mod wma;

pub use audio::xbox::bink_sound::MovieAudioOut;
pub use input::GameAudioInput;
pub use plugin::{AudioCommand, AudioSystems, AudioUpdateGate, VisosAudio};
pub use vs_audio::{GuiSound, VsAudioInitInfo};

/// the visos audio engine and the game's audio, driven by `GameAudioInput` and `AudioCommand`s.
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        plugin::build(app);
        trace::build(app);
    }
}
