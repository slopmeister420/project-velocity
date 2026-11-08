//! playing samples: AUDIO_PlaySample.cpp, AUDIO_QueueSample.cpp, AUDIO_StopSample and the per-sample
//! setters, plus the `_AUDIO_*M` voice side they drive (setup/start/update/volume/pitch).

use super::system::*;
use super::xva;
use super::xbox::voice::VOLUME_MIN;

use sample_flags as sf;

impl AudioSystem {
    fn residency(&self, ps: SampleId) -> u32 {
        self.samples.get(ps).map(|p| p.flags & xva::BANK_RESIDENCY_MASK).unwrap_or(0)
    }

    /// _AUDIO_FreePlayingSample.cpp 0x00312440 / 0x00313EA0 (`_AUDIO_ReleaseVoiceM`)
    pub(crate) fn free_playing_sample(&mut self, ps: SampleId) {
        if let Some(p) = self.samples.get_mut(ps) {
            if let Some(v) = p.voice.take() {
                self.voices.stop(v);
                self.voices.free(v);
            }
        }
        self.samples.free(ps);
    }

    fn free_stream(&mut self, ps: SampleId) {
        if let Some(p) = self.samples.get_mut(ps) {
            p.stream = None;
        }
        // the port keeps no stream buffer objects; release one pool slot
        if let Some(&id) = self.streams.ids().first() {
            self.streams.free(id);
        }
    }

    /// AUDIO_PlaySample.cpp 0x003425A0
    #[allow(clippy::too_many_arguments)]
    pub fn play_sample(
        &mut self,
        bank: BankId,
        index: usize,
        emitter: EmitterId,
        flags: u16,
        category_mask: u32,
        volume: f32,
        pitch: f32,
        callback: Option<SampleCallback>,
    ) -> Result<SampleId, i32> {
        if !self.initialised {
            return Err(not_init());
        }
        let Some(b) = self.banks.get(bank) else { return Err(err::BAD_BANK) };
        if b.flags & 0x2000 != 0 {
            return Err(err::BAD_BANK);
        }
        if index >= b.headers.len() {
            return Err(err::BAD_SAMPLE_INDEX);
        }
        let header = b.headers[index];
        if !self.emitters.get(emitter).is_some_and(|e| e.flags & emitter_flags::ALIVE != 0) {
            return Err(err::BAD_EMITTER);
        }
        let ps = PlayingSample {
            next: None,
            flags: flags as u32 | (header.format & 0xFFFF_0000) | sf::ALIVE,
            queue_next: None,
            queue_index: 0,
            emitter,
            bank,
            index,
            stream: None,
            resample_pos: 0.0,
            freq_ratio: 0.0,
            category_mask,
            volume,
            fade_rate: 0.0,
            fade_time: 0.0,
            pitch,
            callback,
            voice: None,
            pan: 0.0,
        };
        let Some(id) = self.samples.alloc(ps) else { return Err(err::NO_SAMPLE) };
        if header.size == 0 {
            self.samples.free(id);
            return Err(err::EMPTY_SAMPLE);
        }
        if self.residency(id) == xva::BANK_STREAMED {
            if self.streams.alloc(StreamBuf).is_none() {
                self.samples.free(id);
                return Err(err::NO_STREAM);
            }
            self.samples.get_mut(id).unwrap().stream = Some(StreamBuf);
        }
        let r = self.start_sample_m(id);
        if r != 0 {
            if self.residency(id) == xva::BANK_STREAMED {
                self.free_stream(id);
            }
            self.free_playing_sample(id);
            return Err(r);
        }
        let e = self.emitters.get_mut(emitter).unwrap();
        e.flags |= emitter_flags::HAS_SAMPLES;
        e.flags &= !0x100;
        let head = e.samples;
        e.samples = Some(id);
        let p = self.samples.get_mut(id).unwrap();
        p.flags |= sf::STARTED;
        p.next = head;
        self.list_changed = true;
        Ok(id)
    }

    /// AUDIO_QueueSample.cpp 0x00342A50: append a sample to play after `after` (and its queue) ends.
    pub fn queue_sample(
        &mut self,
        bank: BankId,
        index: usize,
        after: SampleId,
        flags: u16,
        callback: Option<SampleCallback>,
    ) -> Result<SampleId, i32> {
        if !self.initialised {
            return Err(not_init());
        }
        let Some(b) = self.banks.get(bank) else { return Err(err::BAD_SAMPLE_INDEX) };
        if index >= b.headers.len() {
            return Err(err::BAD_SAMPLE_INDEX);
        }
        let header = b.headers[index];
        let Some(a) = self.samples.get(after).cloned() else { return Err(err::BAD_PLAYING_SAMPLE) };
        if a.flags & sf::ALIVE == 0 {
            return Err(0x0900_001A);
        }
        let ps = PlayingSample {
            next: None,
            flags: flags as u32 | (header.format & 0xFFFF_0000) | sf::ALIVE | sf::QUEUED,
            queue_next: None,
            queue_index: 0,
            emitter: a.emitter,
            bank,
            index,
            stream: None,
            resample_pos: 0.0,
            freq_ratio: 0.0,
            category_mask: a.category_mask,
            volume: a.volume,
            fade_rate: 0.0,
            fade_time: 0.0,
            pitch: 0.0,
            callback,
            voice: None,
            pan: 0.0,
        };
        let Some(id) = self.samples.alloc(ps) else { return Err(err::NO_SAMPLE) };
        // append to the end of the queue
        let mut j = after;
        while let Some(n) = self.samples.get(j).and_then(|p| p.queue_next) {
            j = n;
        }
        let qi = self.samples.get(j).unwrap().queue_index + 1;
        self.samples.get_mut(j).unwrap().queue_next = Some(id);
        self.samples.get_mut(id).unwrap().queue_index = qi;
        let res = self.residency(id);
        if (res == 0x40000 || res == xva::BANK_STREAMED) && qi < self.params.stream_queue {
            if self.streams.alloc(StreamBuf).is_none() {
                self.samples.free(id);
                return Err(err::NO_STREAM);
            }
            self.samples.get_mut(id).unwrap().stream = Some(StreamBuf);
        }
        self.list_changed = true;
        Ok(id)
    }

    /// AUDIO_StopSample.cpp 0x003149B0
    pub fn stop_sample(&mut self, ps: SampleId) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(p) = self.samples.get(ps).cloned() else { return err::BAD_PLAYING_SAMPLE };
        let r = if p.flags & sf::STARTED == 0 {
            if p.flags & sf::STOPPABLE != 0 { self.stop_voice_m(ps) } else { err::NOT_PLAYING }
        } else {
            let r = self.stop_voice_m(ps);
            if r != 0 {
                return r;
            }
            r
        };
        if let Some(x) = self.samples.get_mut(ps) {
            x.flags &= !sf::STARTED;
        }
        if p.flags & sf::ALIVE == 0 {
            return r;
        }
        let Some(em) = self.emitters.get(p.emitter) else { return r };
        // find it in its emitter's list (or in a list member's queue)
        let mut prev: Option<SampleId> = None;
        let mut cur = em.samples;
        loop {
            let Some(c) = cur else { return err::FAILED };
            if c == ps {
                break;
            }
            if p.flags & sf::STOPPABLE != 0 {
                let mut qp = c;
                let mut q = self.samples.get(c).and_then(|x| x.queue_next);
                while let Some(qq) = q {
                    if qq == ps {
                        if self.samples.get(ps).and_then(|x| x.stream).is_some() {
                            self.free_stream(ps);
                        }
                        let nxt = self.samples.get(ps).and_then(|x| x.queue_next);
                        self.samples.get_mut(qp).unwrap().queue_next = nxt;
                        self.free_playing_sample(ps);
                        self.list_changed = true;
                        if let Some(cb) = p.callback {
                            self.sample_callback(cb, 2);
                        }
                        return r;
                    }
                    qp = qq;
                    q = self.samples.get(qq).and_then(|x| x.queue_next);
                }
            }
            prev = Some(c);
            cur = self.samples.get(c).and_then(|x| x.next);
        }
        // free its queue
        let mut q = self.samples.get(ps).and_then(|x| x.queue_next);
        while let Some(qq) = q {
            let n = self.samples.get(qq).and_then(|x| x.queue_next);
            if self.samples.get(qq).and_then(|x| x.stream).is_some() {
                self.free_stream(qq);
            }
            self.free_playing_sample(qq);
            q = n;
        }
        let next = self.samples.get(ps).and_then(|x| x.next);
        match prev {
            Some(pv) => self.samples.get_mut(pv).unwrap().next = next,
            None => self.emitters.get_mut(p.emitter).unwrap().samples = next,
        }
        if self.samples.get(ps).and_then(|x| x.stream).is_some() {
            self.free_stream(ps);
        }
        self.free_playing_sample(ps);
        if let Some(e) = self.emitters.get_mut(p.emitter) {
            if e.samples.is_none() {
                e.flags &= !emitter_flags::HAS_SAMPLES;
            }
        }
        self.list_changed = true;
        if let Some(cb) = p.callback {
            self.sample_callback(cb, 2);
        }
        r
    }

    /// AUDIO_SetSampleVolume.cpp 0x002BE350
    pub fn set_sample_volume(&mut self, ps: SampleId, v: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(p) = self.samples.get_mut(ps) else { return err::BAD_PLAYING_SAMPLE };
        if p.flags & 4 != 0 {
            return err::FADING;
        }
        if !(0.0..=1.0).contains(&v) {
            return err::BAD_PARAM;
        }
        p.flags &= !sf::FADING;
        if p.volume != v {
            p.volume = v;
            p.flags |= sf::VOLUME_DIRTY;
        }
        0
    }

    /// AUDIO_SetSamplePitch.cpp 0x002BF1D0: cents, -9600..2400.
    pub fn set_sample_pitch(&mut self, ps: SampleId, cents: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(p) = self.samples.get_mut(ps) else { return err::BAD_PLAYING_SAMPLE };
        if p.flags & 4 != 0 {
            return err::FADING;
        }
        if !(-9600.0..=2400.0).contains(&cents) {
            return err::BAD_PARAM;
        }
        if p.pitch != cents {
            p.pitch = cents;
            p.flags |= sf::PITCH_DIRTY;
        }
        0
    }

    /// AUDIO_SetSamplePan.cpp 0x002BEB80: -1 (left) .. 1 (right), used with the `PAN` sample flag.
    pub fn set_sample_pan(&mut self, ps: SampleId, pan: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(p) = self.samples.get_mut(ps) else { return err::BAD_PLAYING_SAMPLE };
        p.flags &= !sf::FADING;
        p.pan = pan;
        0
    }

    /// AUDIO_FadeSampleVolume.cpp 0x002BFD50: fade from `from` (-1 = current) to `to` over `time` seconds.
    pub fn fade_sample_volume(&mut self, ps: SampleId, from: f32, to: f32, time: f32) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(p) = self.samples.get_mut(ps) else { return err::BAD_PLAYING_SAMPLE };
        if p.flags & sf::STARTED == 0 {
            return err::NOT_PLAYING;
        }
        if !((0.0..=1.0).contains(&from) || from == -1.0) || !(0.0..=1.0).contains(&to) || time == 0.0 {
            return err::BAD_PARAM;
        }
        if from != -1.0 {
            p.volume = from;
        }
        p.fade_rate = (to - p.volume) / time;
        p.fade_time = time;
        p.flags |= sf::FADING;
        0
    }

    /// AUDIO_PauseSample.cpp 0x002BEAA0
    pub fn pause_sample(&mut self, ps: SampleId) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(p) = self.samples.get_mut(ps) else { return err::BAD_PLAYING_SAMPLE };
        if p.flags & sf::STARTED == 0 {
            return err::NOT_PAUSED;
        }
        p.flags |= sf::PAUSED;
        let mut q = p.queue_next;
        while let Some(qq) = q {
            let x = self.samples.get_mut(qq).unwrap();
            x.flags |= sf::PAUSED;
            q = x.queue_next;
        }
        // _AUDIO_PauseVoiceM 0x003175A0
        if let Some(v) = self.samples.get(ps).and_then(|p| p.voice) {
            self.voices.stop(v);
        }
        0
    }

    /// AUDIO_ResumeSample.cpp 0x002BE9E0
    pub fn resume_sample(&mut self, ps: SampleId) -> i32 {
        if !self.initialised {
            return not_init();
        }
        let Some(p) = self.samples.get_mut(ps) else { return err::BAD_PLAYING_SAMPLE };
        if p.flags & sf::PAUSED == 0 {
            return err::BAD_PLAYING_SAMPLE;
        }
        p.flags &= !sf::PAUSED;
        let mut q = p.queue_next;
        while let Some(qq) = q {
            let x = self.samples.get_mut(qq).unwrap();
            x.flags &= !sf::PAUSED;
            q = x.queue_next;
        }
        // _AUDIO_ResumeVoiceM 0x00317460: resident buffers continue, streams restart unless looping
        let p = self.samples.get(ps).unwrap();
        let (voice, flags) = (p.voice, p.flags);
        if let Some(v) = voice {
            let play = if flags & xva::BANK_RESIDENCY_MASK == 0 {
                flags & sf::LOOP
            } else if flags & sf::LOOP != 0 {
                1
            } else {
                2
            };
            self.voices.play(v, play);
        }
        self.apply_sample_volume_m(ps);
        self.apply_sample_pitch_m(ps);
        0
    }

    /// pause every sample of an emitter (`_AUDIO_PauseEmitterM` 0x00316320)
    pub(crate) fn pause_emitter_samples(&mut self, e: EmitterId) -> i32 {
        let mut k = self.emitters.get(e).and_then(|e| e.samples);
        while let Some(s) = k {
            let r = self.pause_sample(s);
            if r != 0 {
                return r;
            }
            k = self.samples.get(s).and_then(|x| x.next);
        }
        0
    }

    /// resume an emitter's paused samples (`_AUDIO_ResumeEmitterM` 0x003163B0)
    pub(crate) fn resume_emitter_samples(&mut self, e: EmitterId) -> i32 {
        let mut k = self.emitters.get(e).and_then(|e| e.samples);
        while let Some(s) = k {
            if self.samples.get(s).is_some_and(|x| x.flags & sf::PAUSED != 0) {
                let r = self.resume_sample(s);
                if r != 0 {
                    return r;
                }
            }
            k = self.samples.get(s).and_then(|x| x.next);
        }
        0
    }

    /// _AUDIO_SampleVolume.cpp 0x00351F40: sample x emitter x master x every category in the mask.
    pub fn sample_volume(&self, ps: &PlayingSample) -> f32 {
        let ev = self.emitters.get(ps.emitter).map(|e| e.volume).unwrap_or(0.0);
        let mut v = ps.volume * ev * self.master_volume;
        for i in 0..32 {
            if ps.category_mask & (1 << i) != 0 {
                v *= self.category_volume[i];
            }
        }
        v
    }

    /// _AUDIO_ApplySampleVolumeM.cpp 0x00343080
    pub(crate) fn apply_sample_volume_m(&mut self, ps: SampleId) -> i32 {
        let Some(p) = self.samples.get(ps).cloned() else { return 0 };
        let Some(v) = p.voice else { return 0 };
        let vol = self.sample_volume(&p).clamp(0.0, 1.0);
        let ef = self.emitters.get(p.emitter).map(|e| e.flags).unwrap_or(0);
        if ef & emitter_flags::FLAT != 0 && (ef & emitter_flags::PAN != 0 || p.flags & sf::PAN != 0) {
            self.voices.set_volume(v, 0);
            let l = (0.5 - p.pan / 2.0) * vol;
            let r = (p.pan / 2.0 + 0.5) * vol;
            let bins = [self.voices.volume_mb(l), self.voices.volume_mb(r)];
            self.voices.set_mixbins(v, Some(bins));
        } else {
            self.voices.set_mixbins(v, None);
            let mb = self.voices.volume_mb(vol);
            self.voices.set_volume(v, mb);
        }
        0
    }

    /// _AUDIO_ApplySamplePitchM.cpp 0x00342F80: frequency = rate * 2^(cents/1200), 188..191983 Hz.
    pub(crate) fn apply_sample_pitch_m(&mut self, ps: SampleId) -> i32 {
        let Some(p) = self.samples.get(ps).cloned() else { return 0 };
        let rate = self.banks.get(p.bank).map(|b| xva::rate_from_format(b.headers[p.index].format)).unwrap_or(22050);
        let mut cents = p.pitch;
        if p.flags & sf::GLOBAL_PITCH != 0 {
            cents += self.global_pitch;
        }
        let f = 2f32.powf(cents / 1200.0);
        let freq = (f * rate as f32) as i32;
        self.samples.get_mut(ps).unwrap().freq_ratio = (freq as f64 / 48000.0) as f32;
        if !(188..=191983).contains(&freq) {
            return err::BAD_PARAM;
        }
        if let Some(v) = p.voice {
            self.voices.set_frequency(v, freq as u32);
        }
        0
    }

    /// _AUDIO_SetupVoiceM.cpp 0x003515A0: take a voice for the sample's format and give it the data.
    fn setup_voice_m(&mut self, ps: SampleId) -> i32 {
        let p = self.samples.get(ps).unwrap().clone();
        let format = self.banks.get(p.bank).map(|b| b.headers[p.index].format & 0xFFFF).unwrap_or(0);
        let ef = self.emitters.get(p.emitter).map(|e| e.flags).unwrap_or(0);
        let positional = ef & emitter_flags::FLAT != emitter_flags::FLAT;
        if let Some(v) = p.voice {
            self.voices.free(v);
        }
        let Some(v) = self.voices.alloc(format, positional) else {
            self.samples.get_mut(ps).unwrap().voice = None;
            return err::NO_MEMORY;
        };
        self.samples.get_mut(ps).unwrap().voice = Some(v);
        let e = self.emitters.get_mut(p.emitter).unwrap();
        if positional && xva::channels_from_format(format) == 1 {
            e.flags |= emitter_flags::VOICE_3D;
            let (min, max) = (e.min_distance, e.max_distance);
            self.voices.set_distances(v, Some(0.1), Some(max));
            self.voices.set_distances(v, Some(min), None);
        } else {
            e.flags &= !emitter_flags::VOICE_3D;
        }
        let data = self.sample_data(p.bank, p.index);
        if data.is_none() {
            return err::READ;
        }
        self.voices.set_buffer_data(v, data);
        0
    }

    /// _AUDIO_StartSampleM.cpp 0x00351CB0
    pub(crate) fn start_sample_m(&mut self, ps: SampleId) -> i32 {
        let r = self.setup_voice_m(ps);
        if r != 0 {
            return r;
        }
        let p = self.samples.get(ps).unwrap();
        let flags = if p.flags & sf::LOOP != 0 { 3 } else { 2 };
        let v = p.voice.unwrap();
        // volume and pitch go in before the voice starts so the first mixed frame is right
        let r = self.apply_sample_volume_m(ps);
        if r != 0 {
            return r;
        }
        let r = self.apply_sample_pitch_m(ps);
        if r != 0 {
            return r;
        }
        self.update_voice_3d(ps);
        self.voices.play(v, flags);
        0
    }

    /// _AUDIO_StopVoiceM.cpp 0x00314D20
    pub(crate) fn stop_voice_m(&mut self, ps: SampleId) -> i32 {
        if let Some(v) = self.samples.get(ps).and_then(|p| p.voice) {
            self.voices.stop(v);
        }
        0
    }

    fn update_voice_3d(&mut self, ps: SampleId) {
        let Some(p) = self.samples.get(ps) else { return };
        let Some(v) = p.voice else { return };
        let Some(e) = self.emitters.get(p.emitter) else { return };
        if e.flags & emitter_flags::VOICE_3D != 0 {
            let (pos, vel, moving) = (e.position, e.velocity, e.flags & emitter_flags::MOVING != 0);
            self.voices.set_position(v, pos);
            if moving {
                self.voices.set_doppler(v, 1.0);
                self.voices.set_velocity(v, vel);
            } else {
                self.voices.set_doppler(v, 0.0);
            }
        }
    }

    /// _AUDIO_UpdateVoiceM.cpp 0x003511A0: end-of-buffer detection and 3d/volume refresh.
    fn update_voice_m(&mut self, ps: SampleId) -> i32 {
        let p = self.samples.get_mut(ps).unwrap();
        p.flags &= !sf::VOICE_DONE;
        let (voice, flags) = (p.voice, p.flags);
        if let Some(v) = voice {
            if !self.voices.is_playing(v) && flags & sf::LOOP == 0 {
                self.samples.get_mut(ps).unwrap().flags |= sf::VOICE_DONE;
            }
            let ef = self.emitters.get(self.samples.get(ps).unwrap().emitter).map(|e| e.flags).unwrap_or(0);
            if ef & emitter_flags::VOICE_3D != 0 {
                self.update_voice_3d(ps);
            } else {
                self.apply_sample_volume_m(ps);
            }
        }
        let p = self.samples.get_mut(ps).unwrap();
        if p.flags & sf::VOICE_DONE != 0 {
            p.flags |= sf::DONE | sf::FINISHED;
            p.flags &= !sf::STARTED;
        }
        0
    }

    /// _AUDIO_UpdateSampleM.cpp 0x003418C0
    pub(crate) fn update_sample_m(&mut self, ps: SampleId) -> i32 {
        let dt = self.frame_time;
        let Some(p) = self.samples.get_mut(ps) else { return 0 };
        if p.flags & sf::PAUSED != 0 {
            return 0;
        }
        if p.flags & sf::FADING != 0 {
            if p.fade_time <= 0.0 {
                p.volume -= p.fade_rate;
                p.fade_time += 1.0;
                p.flags |= sf::VOLUME_DIRTY;
                if p.fade_time >= 0.0 {
                    p.flags &= !sf::FADING;
                }
            } else {
                p.volume += dt * p.fade_rate;
                p.fade_time -= dt;
                p.flags |= sf::VOLUME_DIRTY;
                if p.fade_time <= 0.0 {
                    p.flags &= !sf::FADING;
                }
            }
            p.volume = p.volume.clamp(0.0, 1.0);
        }
        let p = self.samples.get(ps).unwrap().clone();
        let ef = self.emitters.get(p.emitter).map(|e| e.flags).unwrap_or(0);
        if (p.flags & sf::VOLUME_DIRTY != 0
            || self.category_dirty & p.category_mask != 0
            || self.all_dirty
            || ef & emitter_flags::PITCH_DIRTY != 0)
            && self.apply_sample_volume_m(ps) != 0
            || (p.flags & sf::PITCH_DIRTY != 0 || self.pitch_all_dirty) && self.apply_sample_pitch_m(ps) != 0
        {
            return err::FAILED;
        }
        let r = self.update_voice_m(ps);
        let p = self.samples.get_mut(ps).unwrap();
        if p.flags & sf::FINISHED != 0 {
            p.flags &= !sf::FINISHED;
            let code = if p.flags & sf::STOPPED_CB != 0 { 3 } else { 1 };
            let cb = p.callback;
            let streamed = p.flags & xva::BANK_RESIDENCY_MASK == xva::BANK_STREAMED && p.stream.is_some();
            if let Some(cb) = cb {
                self.sample_callback(cb, code);
            }
            if streamed && self.samples.contains(ps) {
                self.free_stream(ps);
            }
        }
        if let Some(p) = self.samples.get_mut(ps) {
            p.flags &= 0xFF4F_FFFF;
        }
        r
    }

    /// dispatch a sample callback.
    pub(crate) fn sample_callback(&mut self, cb: SampleCallback, code: i32) {
        match cb {
            SampleCallback::Event(e) => self.event_sample_callback(code, e),
        }
    }

    /// _AUDIO_UpdateEmitterSamples.cpp 0x00315540
    pub(crate) fn update_emitter_samples(&mut self, e: EmitterId) -> i32 {
        let Some(em) = self.emitters.get(e) else { return 0 };
        if em.flags & emitter_flags::ALIVE == 0 {
            return 0;
        }
        let mut result = 0;
        self.list_changed = false;
        let mut cur = em.samples;
        let mut prev: Option<SampleId> = None;
        while let Some(c) = cur {
            if !self.samples.contains(c) {
                break;
            }
            let mut next = self.samples.get(c).unwrap().next;
            let r = self.update_sample_m(c);
            if r != 0 {
                result = r;
            }
            if self.samples.get(c).is_some_and(|p| p.flags & sample_flags::VOICE_DONE != 0) {
                if let Some(q) = self.samples.get_mut(c).unwrap().queue_next.take() {
                    // promote the queued sample into the list in its place and start it
                    let cn = self.samples.get(c).unwrap().next;
                    let qs = self.samples.get_mut(q).unwrap();
                    qs.next = cn;
                    qs.flags &= !sample_flags::QUEUED;
                    qs.queue_index = 0;
                    self.samples.get_mut(c).unwrap().next = Some(q);
                    next = Some(q);
                    self.list_changed = true;
                    let mut n = 0;
                    let mut i = self.samples.get(q).unwrap().queue_next;
                    while let Some(ii) = i {
                        n += 1;
                        self.samples.get_mut(ii).unwrap().queue_index = n;
                        i = self.samples.get(ii).unwrap().queue_next;
                    }
                    if self.start_sample_m(q) == 0 {
                        self.samples.get_mut(q).unwrap().flags |= sample_flags::STARTED;
                    }
                }
                match prev {
                    Some(pv) => self.samples.get_mut(pv).unwrap().next = next,
                    None => self.emitters.get_mut(e).unwrap().samples = next,
                }
                self.free_playing_sample(c);
            } else {
                prev = Some(c);
            }
            if self.list_changed {
                cur = self.emitters.get(e).and_then(|x| x.samples);
                prev = None;
                self.list_changed = false;
            } else {
                cur = next;
            }
        }
        if let Some(em) = self.emitters.get_mut(e) {
            em.flags &= 0xFF0F_FFFF;
            if em.samples.is_none() {
                em.flags &= !emitter_flags::HAS_SAMPLES;
            }
            em.flags &= !emitter_flags::SOFTWARE;
        }
        result
    }

    /// _AUDIO_CommitM.cpp 0x003158D0 (the commit half)
    pub(crate) fn commit_voices_m(&mut self) {
        self.voices.commit();
    }

    /// the mB a sample's voice ended up with (diagnostics/tests)
    pub fn sample_volume_mb(&self, ps: SampleId) -> i32 {
        self.samples.get(ps).map(|p| self.voices.volume_mb(self.sample_volume(p).clamp(0.0, 1.0))).unwrap_or(VOLUME_MIN)
    }
}
