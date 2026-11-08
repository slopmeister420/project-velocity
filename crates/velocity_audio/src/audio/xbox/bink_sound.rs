//! RAD Bink's xbox DirectSound driver (XBE `BINK` section, 0x00543EF0..0x00544320): the sound system Bink uses
//! by default (BinkOpen installs `0x00544280` when no other is set). one streaming dsound buffer per Bink sound
//! track, outside the visos voice pool and its category volumes; Bink's sound service (0x0053E270) copies the
//! decoded track ring into it. here the buffer is the mixer's `StreamVoice` and the ring is `MovieAudioOut::pcm`.
//!
//! VsMovie only ever touches track 0: BinkSetVolume(bink, 0, ftol(this+124)) right after BinkOpen
//! (VsMovieStreamXBOX 0x002F3CC0), with +124 = `GameManager__BFFF0(v)` = v * 32768.0 (1.0 from GameManager__BE9F0,
//! `flt_6F9174` for the in-race TFT movies). the only BinkPause is BinkClose's (0x00541C60 -> 0x005401E0).

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::audio::mixer::SharedMixer;

/// dsound's silence floor (DSBVOLUME_MIN, millibels)
const VOLUME_MIN: i32 = -10000;

/// the decoded sound of the playing Bink's first track, written by the movie player (velocity_frontend
/// `movie.rs`, re-exported there as `movie::MovieAudioOut`) and drained into the mixer every frame.
#[derive(Resource, Debug)]
pub struct MovieAudioOut {
    pub rate: u32,
    pub channels: u32,
    /// interleaved s16 decoded but not yet handed to the sound buffer
    pub pcm: VecDeque<i16>,
    /// VsMovie +124 (Bink volume units, 32768 = full): passed to BinkSetVolume as `ftol` at open
    pub volume: f32,
    /// BinkPause state (0x005401E0)
    pub paused: bool,
    /// a Bink with sound is open (BinkOpen .. BinkClose)
    pub open: bool,
    /// bumped on every `open`, so close + open within one frame still reopens the buffer
    pub generation: u32,
}

impl Default for MovieAudioOut {
    fn default() -> Self {
        // GameManager__BE9F0 0x000BE9F0: GameManager__BFFF0(1.0) before every full-screen movie
        Self { rate: 0, channels: 0, pcm: VecDeque::new(), volume: 1.0 * 32768.0, paused: false, open: false, generation: 0 }
    }
}

impl MovieAudioOut {
    /// BinkOpen 0x0053E9A0 opened sound track 0 at `rate` / `channels` (VsMovieStreamXBOX 0x002F3CC0)
    pub fn open(&mut self, rate: u32, channels: u32) {
        self.rate = rate;
        self.channels = channels;
        self.pcm.clear();
        self.paused = false;
        self.open = rate != 0 && channels != 0;
        self.generation = self.generation.wrapping_add(1);
    }

    /// BinkClose 0x00541C60: BinkPause(1), then the driver closes its buffer (queued sound is dropped)
    pub fn close(&mut self) {
        self.paused = true;
        self.open = false;
        self.pcm.clear();
    }
}

/// driver state of one track (the 380-byte Bink sound struct; driver fields from +0x7C).
#[derive(Debug, Clone, PartialEq)]
pub struct BinkSound {
    pub freq: u32,     // +0x8C
    pub bits: u32,     // +0x90
    pub channels: u32, // +0x94
    pub volume: i32,   // +0x98
    pub pan: i32,      // +0x9C
    pub paused: bool,  // +0xA4
    /// dsound buffer bytes: 600 ms rounded up to 1 KB
    pub buffer_bytes: u32, // +0x84
}

/// 0x005435E0: the dsound volume in millibels for a Bink volume (fistp rounds to nearest; a negative volume
/// gives log10 NaN -> the integer indefinite, whose low dword 0 is used: full volume).
pub fn volume_mb(volume: i32) -> i32 {
    let v = volume.min(0x7FFF);
    if v == 0 {
        return VOLUME_MIN;
    }
    let x = (v as f64 * f32::from_bits(0x3800_0100) as f64).log10() * f32::from_bits(0x4550_5555) as f64;
    if x.is_nan() {
        return 0;
    }
    (x.round_ties_even() as i32).max(VOLUME_MIN)
}

/// IDirectSoundBuffer_SetVolume on the mcpx: millibels -> linear
fn linear(mb: i32) -> f32 {
    if mb <= VOLUME_MIN {
        0.0
    } else {
        10f32.powf(mb as f32 / 2000.0)
    }
}

impl BinkSound {
    /// 0x00543FF0 (the open function `0x00544280` returns): freq, bits, channels of the track; volume and pan
    /// 0x7FFF, six mixbins at 0 mB (front pair = the track's channels), the buffer filled with silence.
    pub fn open(mixer: &SharedMixer, freq: u32, bits: u32, channels: u32) -> Self {
        let bps = (bits >> 3) * channels * freq;
        let s = Self {
            freq,
            bits,
            channels,
            volume: 0x7FFF,
            pan: 0x7FFF,
            paused: false,
            buffer_bytes: (bps * 600 / 1000 + 0x3FF) & !0x3FF,
        };
        if let Ok(mut m) = mixer.lock() {
            m.stream.reset(freq, channels as usize);
            m.stream.playing = true;
        }
        s.apply(mixer);
        s
    }

    /// 0x005435E0: SetFrequency, SetVolume, the mixbins (pan is stored but never applied)
    fn apply(&self, mixer: &SharedMixer) {
        let g = linear(volume_mb(self.volume));
        if let Ok(mut m) = mixer.lock() {
            m.stream.target = g;
        }
    }

    /// 0x00543C70 (track +12, BinkSetVolume 0x00540D30)
    pub fn set_volume(&mut self, mixer: &SharedMixer, volume: i32) {
        self.volume = volume.min(0x7FFF);
        self.apply(mixer);
    }

    /// 0x00543CA0 (track +16, BinkSetPan)
    pub fn set_pan(&mut self, mixer: &SharedMixer, pan: i32) {
        self.pan = pan.min(0x10000);
        self.apply(mixer);
    }

    /// 0x00543E60 (track +20, BinkPause 0x005401E0): stop / restart the buffer where it was
    pub fn pause(&mut self, mixer: &SharedMixer, on: bool) {
        self.paused = on;
        if let Ok(mut m) = mixer.lock() {
            m.stream.playing = !on;
        }
    }

    /// 0x00543F70 (track +28, BinkClose): stop and release the buffer
    pub fn close(self, mixer: &SharedMixer) {
        if let Ok(mut m) = mixer.lock() {
            m.stream.playing = false;
            m.stream.pcm.clear();
        }
    }
}

/// the driver side of the movie: open / volume / pause / close follow `MovieAudioOut`, its pcm is moved into
/// the stream buffer (Bink's sound service 0x0053E270, every BinkWait / DoFrame).
pub fn movie_sound(mut out: ResMut<MovieAudioOut>, audio: Res<crate::VisosAudio>, mut track: Local<Option<(u32, BinkSound)>>) {
    let mixer = &audio.mixer;
    let generation = out.generation;
    let reopen = out.open && track.as_ref().is_none_or(|(g, _)| *g != generation);
    if (!out.open || reopen) && track.is_some() {
        track.take().unwrap().1.close(mixer);
    }
    if reopen {
        let mut s = BinkSound::open(mixer, out.rate, 16, out.channels);
        // VsMovieStreamXBOX 0x002F3CC0: BinkSetVolume(bink, 0, ftol(+124))
        s.set_volume(mixer, out.volume as i32);
        if let Ok(mut m) = mixer.lock() {
            m.stream.snap_gain();
        }
        *track = Some((generation, s));
    }
    let Some((_, s)) = track.as_mut() else {
        out.pcm.clear();
        return;
    };
    if s.paused != out.paused {
        s.pause(mixer, out.paused);
    }
    if s.volume != (out.volume as i32).min(0x7FFF) {
        s.set_volume(mixer, out.volume as i32);
    }
    if let Ok(mut m) = mixer.lock() {
        m.stream.pcm.extend(out.pcm.drain(..));
        // nothing pulls the mix (headless tests): keep at most 4 s like a full ring
        let cap = (s.freq * s.channels * 4) as usize;
        let n = m.stream.pcm.len();
        if n > cap {
            m.stream.pcm.drain(..n - cap);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_curve() {
        assert_eq!(volume_mb(0), -10000);
        assert_eq!(volume_mb(32768), 0);
        assert_eq!(volume_mb(0x7FFF), 0);
        // 1.0 / 2 in Bink units -> -1003 mB (5/3 of dsound's 20 log10)
        assert_eq!(volume_mb(16384), -1003);
        assert_eq!(volume_mb(1), -10000);
        assert_eq!(volume_mb(-5), 0);
    }

    #[test]
    fn movie_stream_plays_resampled_and_stops_on_close() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, crate::AudioPlugin));
        let mixer = app.world().resource::<crate::VisosAudio>().mixer.clone();
        {
            let mut o = app.world_mut().resource_mut::<MovieAudioOut>();
            o.open(24000, 2);
            o.pcm.extend((0..24000 * 2).map(|i| if i % 2 == 0 { 8000 } else { -8000 }));
        }
        app.update();
        let mut buf = vec![0.0f32; 2 * 4800];
        mixer.lock().unwrap().render(&mut buf);
        assert!((buf[100] - 8000.0 / 32768.0).abs() < 1e-3 && (buf[101] + 8000.0 / 32768.0).abs() < 1e-3);
        // 4800 frames at 48 kHz used 2400 source frames
        assert_eq!(mixer.lock().unwrap().stream.pcm.len(), (24000 - 2400) * 2);
        app.world_mut().resource_mut::<MovieAudioOut>().close();
        app.update();
        mixer.lock().unwrap().render(&mut buf);
        assert!(buf.iter().all(|v| *v == 0.0));
    }
}
