//! XBox\_AUDIO_InitM.cpp and the other `_AUDIO_*M` platform functions: the voice (dsound buffer) pools and
//! what the engine asks of them. dsound itself is replaced by `mixer::Mixer`; the 3d buffer/listener model
//! the mcpx applied in hardware (distance rolloff, panning, doppler) is reproduced in `commit`.

use std::sync::Arc;

use bevy::math::Vec3;

use crate::audio::mixer::{SampleData, SharedMixer, MIX_RATE};
use crate::audio::pool::{Id, Pool};
use crate::audio::xva;

/// 64 mono 3d buffers (`dword_782E90`), 64 mono 2d buffers (`dword_782E94`), 8 stereo (`dword_782E98`).
pub const VOICES_3D: usize = 64;
pub const VOICES_2D: usize = 64;
pub const VOICES_STEREO: usize = 8;
pub const VOICE_COUNT: usize = VOICES_3D + VOICES_2D + VOICES_STEREO;
/// the soundtrack player's stream, outside the pooled voices
pub const MUSIC_VOICE: usize = VOICE_COUNT;
/// mixer voices to create
pub const MIXER_VOICES: usize = VOICE_COUNT + 1;

/// `DSBVOLUME_MIN`
pub const VOLUME_MIN: i32 = -10000;

/// one dsound buffer of a pool (16 bytes: link, flags, format, `IDirectSoundBuffer*`).
pub struct Voice {
    pub pool: u8, // flags & 7: 1 = 3d, 2 = 2d, 4 = stereo
    pub format: u32,
    pub slot: usize, // mixer voice index
}

/// per-buffer state dsound keeps.
#[derive(Clone, Debug)]
struct BufferState {
    volume: i32,                // mB
    mixbins: Option<[i32; 2]>,  // mB per front L/R mixbin (SetMixBinVolumes)
    freq: u32,                  // Hz
    is_3d: bool,
    position: Vec3,
    velocity: Vec3,
    doppler: f32,
    min_distance: f32,
    max_distance: f32,
}

impl Default for BufferState {
    fn default() -> Self {
        Self {
            volume: 0,
            mixbins: None,
            freq: MIX_RATE,
            is_3d: false,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            doppler: 1.0,
            min_distance: 1.0,
            max_distance: 1.0e9,
        }
    }
}

pub struct VoiceSystem {
    pub mixer: SharedMixer,
    /// every live voice of the three pools in one id space (per-pool ids collided: index and generation both
    /// start at 0/1 in each pool, so a stereo voice's id also named a 3d voice and frees/stops hit the wrong one)
    ids: Pool<Voice>,
    /// per-pool free mixer slots (the pool limits)
    free_slots: [Vec<usize>; 3],
    buffers: Vec<BufferState>,
    /// `dword_782A88`: mB for a linear 0..255 volume
    pub volume_table: [i32; 256],
    // ds listener (only the position is ever set by the engine)
    pub listener_position: Vec3,
    pub listener_front: Vec3,
    pub listener_top: Vec3,
    pub listener_velocity: Vec3,
    pub doppler_factor: f32,
    pub rolloff_factor: f32,
    // _AUDIO_UpdateListenerM accumulation (dword_782A48/782A80/782A20, byte_782EEC..)
    listener_count: u32,
    listener_mask: u32,
    listener_moved: u32,
    acc_pos: Vec3,
    acc_front: Vec3,
    acc_top: Vec3,
    acc_vel: Vec3,
    pub avg_position: Vec3,
    pub avg_front: Vec3,
    pub avg_top: Vec3,
    pub avg_velocity: Vec3,
}

impl VoiceSystem {
    /// _AUDIO_InitM.cpp 0x003134B0 / 0x00314050 (`_AUDIO_InitVoicesM`)
    pub fn new(mixer: SharedMixer) -> Self {
        let mut volume_table = [0i32; 256];
        volume_table[0] = VOLUME_MIN;
        for (i, v) in volume_table.iter_mut().enumerate().skip(1) {
            // ftol(log10(i * 0.003921569f) * 2000.0)
            let x = i as f32 * f32::from_bits(0x3B80_8081);
            *v = ((x as f64).log10() * 2000.0) as i32;
        }
        let sizes = [VOICES_3D, VOICES_2D, VOICES_STEREO];
        let mut free_slots: [Vec<usize>; 3] = Default::default();
        let mut base = 0;
        for (p, &n) in sizes.iter().enumerate() {
            free_slots[p] = (base..base + n).rev().collect();
            base += n;
        }
        Self {
            mixer,
            ids: Pool::new(VOICE_COUNT),
            free_slots,
            buffers: vec![BufferState::default(); VOICE_COUNT],
            volume_table,
            listener_position: Vec3::ZERO,
            listener_front: Vec3::Z,
            listener_top: Vec3::Y,
            listener_velocity: Vec3::ZERO,
            doppler_factor: f32::from_bits(0x3FB3_3333), // 1.4
            rolloff_factor: 1.0,
            listener_count: 0,
            listener_mask: 0,
            listener_moved: 0,
            acc_pos: Vec3::ZERO,
            acc_front: Vec3::ZERO,
            acc_top: Vec3::ZERO,
            acc_vel: Vec3::ZERO,
            avg_position: Vec3::ZERO,
            avg_front: Vec3::Z,
            avg_top: Vec3::Y,
            avg_velocity: Vec3::ZERO,
        }
    }

    pub fn reset(&mut self) {
        let mut m = self.mixer.lock().unwrap();
        for id in self.ids.ids() {
            let v = self.ids.get(id).unwrap();
            m.stop(v.slot);
            m.set_buffer(v.slot, None);
            self.free_slots[v.pool.trailing_zeros() as usize].push(v.slot);
        }
        self.ids.reset();
    }

    /// `dword_782A88[ftol(v * 255)]`
    pub fn volume_mb(&self, linear: f32) -> i32 {
        self.volume_table[((linear * 255.0) as i32).clamp(0, 255) as usize]
    }

    /// _AUDIO_AllocVoiceM.cpp 0x003144E0: a mono voice from the 3d pool when `want_3d`, else the 2d pool;
    /// stereo formats always take a stereo voice. resets the buffer when the format changes.
    pub fn alloc(&mut self, format: u32, want_3d: bool) -> Option<Id<Voice>> {
        let mono = xva::channels_from_format(format) == 1;
        let pool = if want_3d && mono {
            0
        } else if mono {
            1
        } else {
            2
        };
        let slot = *self.free_slots[pool].last()?;
        let id = self.ids.alloc(Voice { pool: 1 << pool, format, slot })?;
        self.free_slots[pool].pop();
        let b = &mut self.buffers[slot];
        *b = BufferState { is_3d: pool == 0, ..BufferState::default() };
        let mut m = self.mixer.lock().unwrap();
        m.stop(slot);
        m.set_buffer(slot, None);
        Some(id)
    }

    fn find(&self, id: Id<Voice>) -> Option<(usize, usize)> {
        self.ids.get(id).map(|v| (v.pool.trailing_zeros() as usize, v.slot))
    }

    /// _AUDIO_FreeVoiceM.cpp 0x003146C0: stop and return to its pool.
    pub fn free(&mut self, id: Id<Voice>) {
        if let Some((p, slot)) = self.find(id) {
            {
                let mut m = self.mixer.lock().unwrap();
                m.stop(slot);
                m.set_buffer(slot, None);
            }
            self.ids.free(id);
            self.free_slots[p].push(slot);
        }
    }

    pub fn slot(&self, id: Id<Voice>) -> Option<usize> {
        self.find(id).map(|x| x.1)
    }

    /// `IDirectSoundBuffer_SetBufferData`
    pub fn set_buffer_data(&mut self, id: Id<Voice>, data: Option<Arc<SampleData>>) {
        if let Some(slot) = self.slot(id) {
            self.mixer.lock().unwrap().set_buffer(slot, data);
        }
    }
    pub fn queue_buffer_data(&mut self, id: Id<Voice>, data: Arc<SampleData>) {
        if let Some(slot) = self.slot(id) {
            self.mixer.lock().unwrap().queue_buffer(slot, data);
        }
    }

    /// `IDirectSoundBuffer_Play(buffer, 0, 0, flags)`: 1 = looping, 2 = from start.
    pub fn play(&mut self, id: Id<Voice>, flags: u32) {
        if let Some(slot) = self.slot(id) {
            self.push(slot);
            let mut m = self.mixer.lock().unwrap();
            if flags & 2 != 0 {
                m.voices[slot].pos = 0.0;
            }
            m.play(slot, flags & 1 != 0);
        }
    }
    /// `IDirectSoundBuffer_Stop` (keeps the play cursor)
    pub fn stop(&mut self, id: Id<Voice>) {
        if let Some(slot) = self.slot(id) {
            self.mixer.lock().unwrap().stop(slot);
        }
    }
    /// `IDirectSoundBuffer_GetStatus` & DSBSTATUS_PLAYING
    pub fn is_playing(&self, id: Id<Voice>) -> bool {
        self.slot(id).is_some_and(|s| self.mixer.lock().unwrap().is_playing(s))
    }
    pub fn set_volume(&mut self, id: Id<Voice>, mb: i32) {
        if let Some(slot) = self.slot(id) {
            self.buffers[slot].volume = mb;
        }
    }
    /// SetMixBinVolumes on the front left/right bins; `None` restores the default routing.
    pub fn set_mixbins(&mut self, id: Id<Voice>, bins: Option<[i32; 2]>) {
        if let Some(slot) = self.slot(id) {
            self.buffers[slot].mixbins = bins;
        }
    }
    pub fn set_frequency(&mut self, id: Id<Voice>, hz: u32) {
        if let Some(slot) = self.slot(id) {
            self.buffers[slot].freq = hz;
        }
    }
    pub fn set_position(&mut self, id: Id<Voice>, p: Vec3) {
        if let Some(slot) = self.slot(id) {
            self.buffers[slot].position = p;
        }
    }
    pub fn set_velocity(&mut self, id: Id<Voice>, v: Vec3) {
        if let Some(slot) = self.slot(id) {
            self.buffers[slot].velocity = v;
        }
    }
    pub fn set_doppler(&mut self, id: Id<Voice>, f: f32) {
        if let Some(slot) = self.slot(id) {
            self.buffers[slot].doppler = f;
        }
    }
    pub fn set_distances(&mut self, id: Id<Voice>, min: Option<f32>, max: Option<f32>) {
        if let Some(slot) = self.slot(id) {
            if let Some(v) = min {
                self.buffers[slot].min_distance = v;
            }
            if let Some(v) = max {
                self.buffers[slot].max_distance = v;
            }
        }
    }

    /// _AUDIO_UpdateListenerM.cpp 0x00341BD0 (-1)
    pub fn listener_begin(&mut self) {
        self.listener_count = 0;
        self.listener_mask = 0;
        self.listener_moved = 0;
        self.acc_pos = Vec3::ZERO;
        self.acc_front = Vec3::ZERO;
        self.acc_top = Vec3::ZERO;
        self.acc_vel = Vec3::ZERO;
    }
    /// `_AUDIO_UpdateListenerM(listener)`
    pub fn listener_add(&mut self, flags: u32, pos: Vec3, front: Vec3, top: Vec3, vel: Vec3) {
        self.listener_count += 1;
        self.listener_mask |= flags;
        if flags & 0x20_0000 != 0 {
            self.listener_moved += 1;
            self.acc_pos += pos;
            self.acc_front += front;
            self.acc_top += top;
            self.acc_vel += vel;
        }
    }
    /// `_AUDIO_UpdateListenerM(-2)`: averages; only the position reaches the ds listener.
    pub fn listener_end(&mut self) {
        if self.listener_mask & 0x20_0000 == 0 {
            return;
        }
        let n = self.listener_moved as f32;
        self.avg_position = self.acc_pos / n;
        self.avg_front = self.acc_front / n;
        self.avg_top = self.acc_top / n;
        self.avg_velocity = self.acc_vel / n;
        self.listener_position = self.avg_position;
    }

    /// recompute every playing buffer's mix gains and frequency from the dsound state (the mcpx 3d
    /// pipeline: inverse distance rolloff clamped at min/max distance, pan from the listener-space
    /// direction, doppler from the listener/source velocities).
    pub fn commit(&mut self) {
        let mut m = self.mixer.lock().unwrap();
        for id in self.ids.ids() {
            let slot = self.ids.get(id).unwrap().slot;
            let b = &self.buffers[slot];
            let (gains, freq) = self.mix_params(b);
            m.set_gains(slot, gains[0], gains[1]);
            m.set_freq(slot, freq);
        }
    }

    fn push(&self, slot: usize) {
        let b = &self.buffers[slot];
        let (gains, freq) = self.mix_params(b);
        let mut m = self.mixer.lock().unwrap();
        m.set_gains(slot, gains[0], gains[1]);
        m.set_freq(slot, freq);
    }

    fn mix_params(&self, b: &BufferState) -> ([f32; 2], f32) {
        let lin = |mb: i32| if mb <= VOLUME_MIN { 0.0 } else { 10f32.powf(mb as f32 / 2000.0) };
        let mut g = [lin(b.volume); 2];
        if let Some(bins) = b.mixbins {
            g = [g[0] * lin(bins[0]), g[1] * lin(bins[1])];
        }
        let mut freq = b.freq as f32;
        if b.is_3d {
            let rel = b.position - self.listener_position;
            let d = rel.length();
            let dc = d.clamp(b.min_distance.max(1e-6), b.max_distance.max(b.min_distance));
            let atten = (b.min_distance.max(1e-6) / dc).powf(self.rolloff_factor);
            // listener space: x right = top x front (ds3d is left handed), z = front
            let right = self.listener_top.cross(self.listener_front).normalize_or_zero();
            let x = if d > 1e-6 { rel.dot(right) / d } else { 0.0 };
            let ang = (x.clamp(-1.0, 1.0) + 1.0) * std::f32::consts::FRAC_PI_4;
            let l = (ang.cos() * std::f32::consts::SQRT_2).min(1.0);
            let r = (ang.sin() * std::f32::consts::SQRT_2).min(1.0);
            g = [g[0] * atten * l, g[1] * atten * r];
            if d > 1e-6 && b.doppler > 0.0 {
                let u = rel / d;
                let c = 343.3f32;
                let f = self.doppler_factor * b.doppler;
                let vl = self.listener_velocity.dot(u) * f;
                let vs = b.velocity.dot(u) * f;
                let ratio = ((c + vl) / (c + vs).max(1.0)).clamp(0.25, 4.0);
                freq *= ratio;
            }
        }
        (g, freq.clamp(1.0, 192000.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_ids_are_unique_across_pools() {
        let mixer = Arc::new(std::sync::Mutex::new(crate::audio::mixer::Mixer::new(MIXER_VOICES)));
        let mut vs = VoiceSystem::new(mixer);
        let v3d = vs.alloc(0x10, true).unwrap();
        let st = vs.alloc(0x20, false).unwrap();
        assert_eq!(vs.slot(st), Some(VOICES_3D + VOICES_2D));
        assert_ne!(vs.slot(v3d), vs.slot(st));
        vs.free(v3d);
        // a stale free of the 3d id must not take the stereo voice
        vs.free(v3d);
        assert_eq!(vs.slot(st), Some(VOICES_3D + VOICES_2D));
    }
}
