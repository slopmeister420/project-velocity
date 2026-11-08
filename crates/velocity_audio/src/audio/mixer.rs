//! software voice renderer replacing the mcpx voice processor that dsound drives on the xbox.
//! the port's lowest audio layer: fixed voice slots, each playing one sample's raw bank data (xbox adpcm
//! decoded block by block, or 16-bit pcm) at an arbitrary frequency into a 48 kHz stereo mix.
//! all policy (which voice, what volume/pitch/3d) stays in the ported `xbox::voice` layer above.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::adpcm;
use super::xva;

/// the mcpx mixes at 48 kHz.
pub const MIX_RATE: u32 = 48000;

/// raw bytes of one bank sample plus its format word.
#[derive(Debug)]
pub struct SampleData {
    pub bytes: Vec<u8>,
    pub format: u32,
}

impl SampleData {
    pub fn channels(&self) -> usize {
        xva::channels_from_format(self.format).clamp(1, 2) as usize
    }
    pub fn is_adpcm(&self) -> bool {
        xva::format_tag(self.format) == 105
    }
    pub fn frames(&self) -> usize {
        let ch = self.channels();
        if self.is_adpcm() {
            adpcm::frames_in(self.bytes.len(), ch)
        } else {
            self.bytes.len() / (2 * ch)
        }
    }
    pub fn rate(&self) -> u32 {
        xva::rate_from_format(self.format)
    }
}

/// one decoded adpcm block kept per voice so playback decodes each block once.
#[derive(Default)]
struct BlockCache {
    block: usize,
    valid: bool,
    pcm: Vec<i16>,
}

#[derive(Default)]
pub struct MixVoice {
    pub data: Option<Arc<SampleData>>,
    queue: Vec<Arc<SampleData>>,
    pub playing: bool,
    pub looping: bool,
    /// read position in source frames
    pub pos: f64,
    /// playback frequency in Hz
    pub freq: f32,
    /// linear gains actually applied (ramped toward `target`)
    gain: [f32; 2],
    pub target: [f32; 2],
    cache: BlockCache,
}

impl MixVoice {
    fn frame(&mut self, idx: usize) -> [f32; 2] {
        let Some(d) = self.data.clone() else { return [0.0; 2] };
        let ch = d.channels();
        let s = if d.is_adpcm() {
            let block = idx / adpcm::FRAMES_PER_BLOCK;
            if !self.cache.valid || self.cache.block != block {
                let bb = adpcm::BLOCK_BYTES * ch;
                self.cache.pcm.clear();
                if let Some(b) = d.bytes.get(block * bb..block * bb + bb) {
                    adpcm::decode_block(b, ch, &mut self.cache.pcm, adpcm::FRAMES_PER_BLOCK);
                }
                self.cache.block = block;
                self.cache.valid = true;
            }
            let o = (idx % adpcm::FRAMES_PER_BLOCK) * ch;
            let l = *self.cache.pcm.get(o).unwrap_or(&0);
            let r = if ch == 2 { *self.cache.pcm.get(o + 1).unwrap_or(&0) } else { l };
            [l, r]
        } else {
            let o = idx * 2 * ch;
            let rd = |o: usize| d.bytes.get(o..o + 2).map(|b| i16::from_le_bytes([b[0], b[1]])).unwrap_or(0);
            let l = rd(o);
            [l, if ch == 2 { rd(o + 2) } else { l }]
        };
        [s[0] as f32 / 32768.0, s[1] as f32 / 32768.0]
    }

    /// advance to the next queued buffer (dsound stream packet chaining); false when nothing is queued.
    fn next_buffer(&mut self) -> bool {
        if self.queue.is_empty() {
            return false;
        }
        self.data = Some(self.queue.remove(0));
        self.cache.valid = false;
        true
    }

    fn render(&mut self, out: &mut [f32]) {
        if !self.playing || self.data.is_none() {
            return;
        }
        let n = out.len() / 2;
        let step = self.freq as f64 / MIX_RATE as f64;
        let start = self.gain;
        for i in 0..n {
            // linear gain ramp over the render chunk avoids zipper noise on per-frame volume changes
            let t = (i + 1) as f32 / n as f32;
            let g = [start[0] + (self.target[0] - start[0]) * t, start[1] + (self.target[1] - start[1]) * t];
            let frames = self.data.as_ref().map(|d| d.frames()).unwrap_or(0);
            if frames == 0 {
                self.playing = false;
                break;
            }
            let i0 = self.pos as usize;
            if i0 >= frames {
                if self.looping {
                    self.pos -= frames as f64;
                    continue;
                }
                if self.next_buffer() {
                    self.pos -= frames as f64;
                    continue;
                }
                self.playing = false;
                break;
            }
            let frac = (self.pos - i0 as f64) as f32;
            let a = self.frame(i0);
            let b = if i0 + 1 < frames { self.frame(i0 + 1) } else if self.looping { self.frame(0) } else { a };
            out[i * 2] += (a[0] + (b[0] - a[0]) * frac) * g[0];
            out[i * 2 + 1] += (a[1] + (b[1] - a[1]) * frac) * g[1];
            self.pos += step;
        }
        self.gain = self.target;
    }
}

/// a streaming buffer written from the game thread: the dsound buffer RAD Bink's xbox sound driver
/// (`audio::xbox::bink_sound`) creates outside the visos voice pool. interleaved s16 at `rate`, played once.
#[derive(Default)]
pub struct StreamVoice {
    pub rate: u32,
    pub channels: usize,
    /// written, not yet played
    pub pcm: VecDeque<i16>,
    pub playing: bool,
    /// fraction of the next source frame already played
    pos: f64,
    gain: f32,
    pub target: f32,
}

impl StreamVoice {
    /// flush and set the format (`IDirectSoundBuffer` recreated per BinkOpen)
    pub fn reset(&mut self, rate: u32, channels: usize) {
        *self = Self { rate, channels: channels.clamp(1, 2), target: self.target, gain: self.target, ..Self::default() };
    }

    /// start at the target gain (volume set before the first sample plays, no ramp)
    pub fn snap_gain(&mut self) {
        self.gain = self.target;
    }

    fn render(&mut self, out: &mut [f32]) {
        let ch = self.channels;
        if !self.playing || ch == 0 || self.rate == 0 {
            return;
        }
        let n = out.len() / 2;
        let step = self.rate as f64 / MIX_RATE as f64;
        let start = self.gain;
        for i in 0..n {
            // underrun: silence until more is written (the driver pads the dsound buffer with silence)
            if self.pcm.len() < 2 * ch {
                break;
            }
            let g = start + (self.target - start) * ((i + 1) as f32 / n as f32);
            let s = |k: usize| self.pcm[k] as f32 / 32768.0;
            let (a, b) = if ch == 2 { ([s(0), s(1)], [s(2), s(3)]) } else { ([s(0); 2], [s(1); 2]) };
            let f = self.pos as f32;
            out[i * 2] += (a[0] + (b[0] - a[0]) * f) * g;
            out[i * 2 + 1] += (a[1] + (b[1] - a[1]) * f) * g;
            self.pos += step;
            while self.pos >= 1.0 && self.pcm.len() >= ch {
                self.pcm.drain(..ch);
                self.pos -= 1.0;
            }
        }
        self.gain = self.target;
    }
}

/// all voices of the output device.
pub struct Mixer {
    pub voices: Vec<MixVoice>,
    /// the movie sound stream (Bink)
    pub stream: StreamVoice,
    /// frames rendered since creation (the device clock)
    pub clock: u64,
}

impl Mixer {
    pub fn new(voices: usize) -> Self {
        Self { voices: (0..voices).map(|_| MixVoice::default()).collect(), stream: StreamVoice::default(), clock: 0 }
    }

    /// set a voice's buffer (`IDirectSoundBuffer_SetBufferData`); stops it and rewinds.
    pub fn set_buffer(&mut self, v: usize, data: Option<Arc<SampleData>>) {
        let mv = &mut self.voices[v];
        mv.data = data;
        mv.queue.clear();
        mv.playing = false;
        mv.pos = 0.0;
        mv.cache.valid = false;
    }
    pub fn queue_buffer(&mut self, v: usize, data: Arc<SampleData>) {
        self.voices[v].queue.push(data);
    }
    /// `IDirectSoundBuffer_Play`
    pub fn play(&mut self, v: usize, looping: bool) {
        let mv = &mut self.voices[v];
        mv.looping = looping;
        mv.playing = mv.data.is_some();
        // start at the target gain rather than ramping up from silence
        mv.gain = mv.target;
    }
    /// `IDirectSoundBuffer_Stop`
    pub fn stop(&mut self, v: usize) {
        self.voices[v].playing = false;
    }
    pub fn is_playing(&self, v: usize) -> bool {
        self.voices[v].playing
    }
    pub fn set_freq(&mut self, v: usize, hz: f32) {
        self.voices[v].freq = hz;
    }
    pub fn set_gains(&mut self, v: usize, l: f32, r: f32) {
        self.voices[v].target = [l, r];
    }
    /// play cursor in source frames
    pub fn position(&self, v: usize) -> f64 {
        self.voices[v].pos
    }

    /// render `out.len() / 2` stereo frames, overwriting `out`.
    pub fn render(&mut self, out: &mut [f32]) {
        out.fill(0.0);
        for v in &mut self.voices {
            v.render(out);
        }
        self.stream.render(out);
        for s in out.iter_mut() {
            *s = s.clamp(-1.0, 1.0);
        }
        self.clock += (out.len() / 2) as u64;
    }
}

pub type SharedMixer = Arc<Mutex<Mixer>>;

/// rodio source pulling from the shared mixer (the dsound output stream).
pub struct MixerStream {
    mixer: SharedMixer,
    buf: Vec<f32>,
    at: usize,
}

impl MixerStream {
    pub fn new(mixer: SharedMixer) -> Self {
        diag::start();
        Self { mixer, buf: vec![0.0; 1024], at: 1024 }
    }
}

/// port diagnostics: `VELOCITY_AUDIO_DIAG=<file>` logs once a second what the sound device pulled from the mix
/// (frames, longest gap between pulls, longest mixer lock wait, mix rms); `VELOCITY_AUDIO_DIAG_MUTE=1` sends
/// silence to the device after measuring (inaudible device-path test).
pub mod diag {
    use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering::Relaxed};
    use std::sync::OnceLock;
    use std::time::Instant;

    pub static ON: AtomicBool = AtomicBool::new(false);
    pub static MUTE: AtomicBool = AtomicBool::new(false);
    pub static GAIN: AtomicU32 = AtomicU32::new(0);
    static FRAMES: AtomicU64 = AtomicU64::new(0);
    static SUMSQ_MILLI: AtomicU64 = AtomicU64::new(0);
    static MAX_GAP_US: AtomicU64 = AtomicU64::new(0);
    static MAX_LOCK_US: AtomicU64 = AtomicU64::new(0);
    static LAST_US: AtomicU64 = AtomicU64::new(0);
    static T0: OnceLock<Instant> = OnceLock::new();

    pub fn now_us() -> u64 {
        T0.get_or_init(Instant::now).elapsed().as_micros() as u64
    }

    pub fn start() {
        // VELOCITY_AUDIO_MUTE=1: silence to the device (inaudible runs), independent of the diag file
        if std::env::var("VELOCITY_AUDIO_MUTE").is_ok_and(|v| v == "1") {
            MUTE.store(true, Relaxed);
        }
        let path = std::env::var_os("VELOCITY_AUDIO_DIAG");
        if ON.swap(true, Relaxed) {
            return;
        }
        let Some(path) = path else {
            // no diag file: one log line per second for the first 30 s, then one a minute (user logs)
            now_us();
            let _ = std::thread::Builder::new().name("visos audio stats".into()).spawn(|| {
                let mut sec = 0u32;
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    sec += 1;
                    let frames = FRAMES.swap(0, Relaxed);
                    let ss = SUMSQ_MILLI.swap(0, Relaxed) as f64 / 1000.0;
                    let (gap, lock) = (MAX_GAP_US.swap(0, Relaxed), MAX_LOCK_US.swap(0, Relaxed));
                    if sec <= 30 || sec % 60 == 0 {
                        let rms = if frames > 0 { (ss / (frames as f64 * 2.0)).sqrt() } else { 0.0 };
                        let db = if rms > 0.0 { 20.0 * rms.log10() } else { -999.0 };
                        bevy::log::info!(
                            "audio device: t {sec}s frames {frames} max_gap_ms {:.1} max_lock_ms {:.2} rms_db {db:.1}",
                            gap as f64 / 1000.0,
                            lock as f64 / 1000.0
                        );
                    }
                }
            });
            return;
        };
        MUTE.store(std::env::var("VELOCITY_AUDIO_DIAG_MUTE").is_ok_and(|v| v == "1"), Relaxed);
        // VELOCITY_AUDIO_DIAG_GAIN=<g>: scale the device output instead of muting (low-level loopback checks)
        if let Some(g) = std::env::var("VELOCITY_AUDIO_DIAG_GAIN").ok().and_then(|v| v.parse::<f32>().ok()) {
            MUTE.store(true, Relaxed);
            GAIN.store(g.to_bits(), Relaxed);
        }
        now_us();
        let _ = std::thread::Builder::new().name("visos audio diag".into()).spawn(move || {
            use std::io::Write;
            let Ok(mut f) = std::fs::File::create(path) else { return };
            let mut sec = 0u32;
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                sec += 1;
                let frames = FRAMES.swap(0, Relaxed);
                let ss = SUMSQ_MILLI.swap(0, Relaxed) as f64 / 1000.0;
                let rms = if frames > 0 { (ss / (frames as f64 * 2.0)).sqrt() } else { 0.0 };
                let db = if rms > 0.0 { 20.0 * rms.log10() } else { -999.0 };
                let _ = writeln!(
                    f,
                    "{sec} frames {frames} max_gap_ms {:.1} max_lock_ms {:.2} rms_db {db:.1}",
                    MAX_GAP_US.swap(0, Relaxed) as f64 / 1000.0,
                    MAX_LOCK_US.swap(0, Relaxed) as f64 / 1000.0
                );
                let _ = f.flush();
            }
        });
    }

    pub fn record(buf: &[f32], lock_us: u64, t_us: u64) {
        let last = LAST_US.swap(t_us, Relaxed);
        if last != 0 {
            MAX_GAP_US.fetch_max(t_us - last, Relaxed);
        }
        MAX_LOCK_US.fetch_max(lock_us, Relaxed);
        FRAMES.fetch_add((buf.len() / 2) as u64, Relaxed);
        let ss: f64 = buf.iter().map(|&s| (s as f64) * (s as f64)).sum();
        SUMSQ_MILLI.fetch_add((ss * 1000.0) as u64, Relaxed);
    }
}

impl Iterator for MixerStream {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.at >= self.buf.len() {
            let on = diag::ON.load(std::sync::atomic::Ordering::Relaxed);
            let t = if on { diag::now_us() } else { 0 };
            if let Ok(mut m) = self.mixer.lock() {
                let l = if on { diag::now_us() - t } else { 0 };
                m.render(&mut self.buf);
                if on {
                    diag::record(&self.buf, l, t);
                }
            } else {
                self.buf.fill(0.0);
            }
            if diag::MUTE.load(std::sync::atomic::Ordering::Relaxed) {
                let g = f32::from_bits(diag::GAIN.load(std::sync::atomic::Ordering::Relaxed));
                self.buf.iter_mut().for_each(|s| *s *= g);
            }
            self.at = 0;
        }
        let s = self.buf[self.at];
        self.at += 1;
        Some(s)
    }
}

impl bevy::audio::Source for MixerStream {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> bevy::audio::ChannelCount {
        std::num::NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> bevy::audio::SampleRate {
        std::num::NonZero::new(MIX_RATE).unwrap()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcm_tone(frames: usize, rate: u32) -> Arc<SampleData> {
        let mut bytes = Vec::new();
        for i in 0..frames {
            let v = ((i as f32 * 440.0 * std::f32::consts::TAU / rate as f32).sin() * 16000.0) as i16;
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        // pcm, 1 channel, 16 bits, 48 kHz
        Arc::new(SampleData { bytes, format: (9 << 12) | (16 << 7) | (1 << 4) })
    }

    /// bevy_audio's chain (rodio Player on the device mixer at 44.1 / 48 kHz) carries a looping tone at its level
    #[test]
    fn device_chain_keeps_level() {
        for dev_rate in [44100u32, 48000] {
            let mixer: SharedMixer = Arc::new(Mutex::new(Mixer::new(4)));
            {
                let mut m = mixer.lock().unwrap();
                m.set_buffer(0, Some(pcm_tone(48000, 48000)));
                m.set_freq(0, 48000.0);
                m.set_gains(0, 0.5, 0.5);
                m.play(0, true);
            }
            let (ctl, mut src) = rodio::mixer::mixer(std::num::NonZero::new(2).unwrap(), std::num::NonZero::new(dev_rate).unwrap());
            let player = rodio::Player::connect_new(&ctl);
            player.append(MixerStream::new(mixer.clone()));
            let n = dev_rate as usize * 2 * 2;
            let out: Vec<f32> = (0..n).map(|_| Iterator::next(&mut src).unwrap_or(0.0)).collect();
            let tail = &out[n / 2..];
            let rms = (tail.iter().map(|v| (v * v) as f64).sum::<f64>() / tail.len() as f64).sqrt();
            let pulled = mixer.lock().unwrap().clock;
            eprintln!("device {dev_rate}: rms {rms:.4} pulled {pulled}");
            // tone peak 16000/32768 * 0.5 -> rms ~0.173
            assert!((rms - 0.1726).abs() < 0.02, "{dev_rate}: {rms}");
            // the device pulls the 48 kHz mix in real time (2 s of device output)
            assert!((pulled as i64 - 96000).abs() < 4096, "{dev_rate}: {pulled}");
        }
    }

    #[test]
    fn plays_once_at_unity_step() {
        let mut m = Mixer::new(2);
        m.set_buffer(0, Some(pcm_tone(4800, 48000)));
        m.set_freq(0, 48000.0);
        m.set_gains(0, 1.0, 1.0);
        m.play(0, false);
        let mut out = vec![0.0; 2 * 6000];
        m.render(&mut out);
        assert!(!m.is_playing(0));
        let rms = (out[..9600].iter().map(|v| v * v).sum::<f32>() / 9600.0).sqrt();
        assert!((rms - 16000.0 / 32768.0 / 2f32.sqrt()).abs() < 0.01, "{rms}");
        assert!(out[9600..].iter().all(|&v| v == 0.0));
    }

    #[test]
    fn half_frequency_doubles_duration_and_loops() {
        let mut m = Mixer::new(1);
        m.set_buffer(0, Some(pcm_tone(4800, 48000)));
        m.set_freq(0, 24000.0);
        m.set_gains(0, 1.0, 1.0);
        m.play(0, false);
        let mut out = vec![0.0; 2 * 9000];
        m.render(&mut out);
        assert!(m.is_playing(0));
        m.render(&mut out);
        assert!(!m.is_playing(0));
        m.set_buffer(0, Some(pcm_tone(100, 48000)));
        m.play(0, true);
        m.render(&mut out);
        assert!(m.is_playing(0));
    }
}
