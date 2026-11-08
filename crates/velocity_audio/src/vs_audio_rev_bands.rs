//! src/Audio/VsAudioRevBands.cpp — the player engine: a set of looping recordings ("rev bands"), each
//! audible over an rpm window with cross-fades at its edges and re-pitched to track the smoothed rpm.
//!
//! `.RBD` (binary, written by the tools from the `.erb` text next to it), 164 + 40 * bands bytes:
//! ```text
//! +0   char group[128]        sound group holding the band events
//! +128 u32 bandCount   +132 i32 centsMin  +136 i32 centsMax  +140 i32 rpmMin  +144 i32 rpmMax
//! +148 f32 smoothing   +152 f32 accMax    +156 f32 accRateUp +160 f32 volume
//! +164 bands[40]: u32 event, f32 lo, f32 hi, f32 fadeIn, f32 fadeOut, f32 volume, i32 centsRef, i32 cents,
//!                 u32 flags (1 = enabled, 2 = on-load, 4 = off-load), (runtime event)
//! ```

use crate::audio::system::{AudioSystem, EmitterId, EventSlot};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RevBand {
    pub event: u32,     // +0
    pub lo: f32,        // +4 normalised rpm
    pub hi: f32,        // +8
    pub fade_in: f32,   // +12
    pub fade_out: f32,  // +16
    pub volume: f32,    // +20
    pub cents_ref: i32, // +24
    pub cents: i32,     // +28
    pub flags: u32,     // +32
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RevBandFile {
    pub group: String,     // +0
    pub cents_min: i32,    // +132
    pub cents_max: i32,    // +136
    pub rpm_min: i32,      // +140
    pub rpm_max: i32,      // +144
    pub smoothing: f32,    // +148
    pub acc_max: f32,      // +152
    pub acc_rate_up: f32,  // +156
    pub volume: f32,       // +160
    pub bands: Vec<RevBand>,
}

impl RevBandFile {
    pub fn parse(b: &[u8]) -> Option<Self> {
        let u = |o: usize| b.get(o..o + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap()));
        let f = |o: usize| u(o).map(f32::from_bits);
        let name = b.get(..128)?;
        let end = name.iter().position(|&c| c == 0).unwrap_or(128);
        let count = u(128)? as usize;
        let mut bands = Vec::with_capacity(count);
        for i in 0..count {
            let o = 164 + 40 * i;
            bands.push(RevBand {
                event: u(o)?,
                lo: f(o + 4)?,
                hi: f(o + 8)?,
                fade_in: f(o + 12)?,
                fade_out: f(o + 16)?,
                volume: f(o + 20)?,
                cents_ref: u(o + 24)? as i32,
                cents: u(o + 28)? as i32,
                flags: u(o + 32)?,
            });
        }
        Some(Self {
            group: String::from_utf8_lossy(&name[..end]).into_owned(),
            cents_min: u(132)? as i32,
            cents_max: u(136)? as i32,
            rpm_min: u(140)? as i32,
            rpm_max: u(144)? as i32,
            smoothing: f(148)?,
            acc_max: f(152)?,
            acc_rate_up: f(156)?,
            volume: f(160)?,
            bands,
        })
    }
}

/// `VsAudioRevBands` (76 bytes, the global at `dword_6D4DB4`).
pub struct VsAudioRevBands {
    pub rbd: Option<RevBandFile>,  // +0
    pub emitter: Option<EmitterId>, // +4
    pub active: bool,              // +8
    pub rpm: i32,                  // +20
    pub norm_rpm: f32,             // +24
    pub rpm_range: f32,            // +28
    pub cents_range: f32,          // +32
    pub volume: f32,               // +36
    pub smooth: f32,               // +40
    pub acc: f32,                  // +44
    pub acc_rate_down: f32,        // +48
    pub acc_rate_up: f32,          // +52
    pub smoothing: f32,            // +56
    pub acc_scale: f32,            // +60
    pub acc_max: f32,              // +64
    pub on_load: f32,              // +68
    pub off_load: f32,             // +72
    /// per band runtime event (`band +36`)
    pub band_events: Vec<EventSlot>,
    /// port: `VsAudio::frame_scale` (the smoothing steps are per rendered frame)
    pub frame_scale: f32,
}

impl Default for VsAudioRevBands {
    /// VsAudioRevBands.cpp 0x0004D3E0
    fn default() -> Self {
        Self {
            rbd: None,
            emitter: None,
            active: false,
            rpm: 0,
            norm_rpm: 0.0,
            rpm_range: 0.0,
            cents_range: 0.0,
            volume: 1.0,
            smooth: 0.0,
            acc: 0.0,
            acc_rate_down: f32::from_bits(0x3DCC_CCCD), // 0.1
            acc_rate_up: 0.0,
            smoothing: 0.0,
            acc_scale: 0.0,
            acc_max: 0.0,
            frame_scale: 1.0,
            on_load: 0.0,
            off_load: 0.0,
            band_events: Vec::new(),
        }
    }
}

/// the wrx's ancillary groups every car uses (`\audio\vehicle\tier4\wrx\vimprRB0` with "RB" replaced).
pub const ANCILLARY_BASE: &str = r"\audio\vehicle\tier4\wrx\vimprRB0";

impl VsAudioRevBands {
    /// VsAudioRevBands.cpp 0x0004C6F0: load the car's rbd and the shared ancillary groups (`dv`, `ex`, `gc`,
    /// `su` variants of the wrx's). `name` = the vehicle database's model/audio block +0x00 (e.g.
    /// `tier4\wrx\vimprrb0`), loaded as `\Audio\vehicle\<name>` (PhysicsManager 0x0019F149, physics vehicle 0).
    pub fn load(&mut self, audio: &mut AudioSystem, name: &str) -> bool {
        let rbd_path = format!(r"\Audio\vehicle\{name}");
        if !self.load_rbd(audio, &rbd_path) {
            log::warn!("Unable to load revband - Annoying assert number 357 removed by Fritz");
        }
        let n = ANCILLARY_BASE.len();
        for sfx in ["dv", "ex", "gc", "su"] {
            let p = format!("{}{}{}", &ANCILLARY_BASE[..n - 3], sfx, &ANCILLARY_BASE[n - 1..]);
            if let Err(e) = audio.load_group(&p, None) {
                log::warn!("audio: failed to load {p}: {e:#x}");
            }
        }
        true
    }

    /// VsAudioRevBands.cpp 0x0004D470
    pub fn load_rbd(&mut self, audio: &mut AudioSystem, path: &str) -> bool {
        self.stop(audio);
        self.rbd = None;
        let Ok(bytes) = visos::file::platform::read(&format!("{path}.RBD")) else { return false };
        let Some(rbd) = RevBandFile::parse(&bytes) else { return false };
        self.set_volume(rbd.volume);
        let _ = audio.load_group(&rbd.group, None);
        let i = audio.find_event(rbd.bands.first().map(|b| b.event).unwrap_or(0));
        if i != -1 {
            let r = audio.event_table[i as usize].event;
            if let Some(e) = audio.event_mut(r) {
                e.unk16 = 0;
            }
        }
        self.rpm_range = (rbd.rpm_max - rbd.rpm_min) as f32;
        self.cents_range = (rbd.cents_max - rbd.cents_min) as f32;
        self.acc_rate_up = rbd.acc_rate_up;
        self.smoothing = rbd.smoothing;
        if self.smoothing < 0.4 {
            self.smoothing = f32::from_bits(0x3ECC_CCCD); // 0.4
        }
        self.acc_scale = rbd.volume;
        self.acc_max = rbd.acc_max;
        while self.band_events.len() < rbd.bands.len() {
            self.band_events.push(audio.new_slot());
        }
        self.rbd = Some(rbd);
        true
    }

    /// VsAudioRevBands.cpp 0x0004D770
    pub fn set_volume(&mut self, v: f32) {
        self.volume = v.clamp(0.0, 1.0);
    }

    /// VsAudioRevBands.cpp 0x0004CBF0: clamp to the rbd's rpm range and normalise.
    pub fn set_rpm(&mut self, mut rpm: i32) {
        let Some(r) = &self.rbd else { return };
        if rpm > r.rpm_max {
            rpm = r.rpm_max;
        }
        if rpm < r.rpm_min {
            rpm = r.rpm_min;
        }
        self.rpm = rpm;
        let d = (rpm - r.rpm_min).max(0);
        self.norm_rpm = d as f32 / self.rpm_range;
    }

    /// VsAudioRevBands.cpp 0x0004D2C0: on/off-load band volumes from the throttle (0..1) on a half-cosine.
    pub fn set_load(&mut self, load: f32) {
        let mut v = 100.0 * load;
        if 100.0 * load > 100.0 {
            v = 100.0;
        }
        let k = (1.0f32 / 100.0) * std::f32::consts::PI;
        self.off_load = ((k * v + 0.5 * std::f32::consts::PI).sin() * 0.5 + 0.5) * 0.5;
        self.on_load = (k * (0.0 - v) - 0.5 * std::f32::consts::PI).sin() * 0.5 + 0.5;
    }

    /// VsAudioRevBands.cpp 0x0004CC90
    pub fn start(&mut self) {
        if self.rbd.is_some() {
            self.active = true;
        }
    }

    /// VsAudioRevBands.cpp 0x0004CCC0
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// VsAudioRevBands.cpp 0x0004CCE0
    pub fn stop(&mut self, audio: &mut AudioSystem) {
        if self.rbd.is_some() {
            if self.active {
                for &s in &self.band_events {
                    audio.stop_slot(s);
                }
            }
            self.active = false;
        }
    }

    /// VsAudioRevBands.cpp 0x0004CD90. `gear` is the player's current gear (`dword_6D4DA0`).
    pub fn update(&mut self, audio: &mut AudioSystem, gear: i32) {
        let Some(rbd) = &self.rbd else { return };
        if !self.active {
            return;
        }
        let Some(emitter) = self.emitter else { return };
        let mut v13 = self.norm_rpm;
        let v12 = v13 - self.smooth - self.acc;
        let rate = if v12 >= 0.0 { self.acc_rate_up } else { self.acc_rate_down };
        // both factors are per reference frame
        self.acc += v12 * crate::vs_audio::per_frame(rate, self.frame_scale);
        self.smooth += (v13 - self.smooth) * crate::vs_audio::per_frame(self.smoothing, self.frame_scale);
        v13 = self.smooth;
        // the original computes the clamped acceleration term here and never uses it
        let _v11 = (self.acc * self.acc_scale).clamp(0.0, self.acc_max.max(0.0));
        let cmin = rbd.cents_min as f32;
        let cmax = rbd.cents_max as f32;
        let v6 = cmin + (cmax - cmin) * v13;
        let bands = rbd.bands.clone();
        for (i, b) in bands.iter().enumerate() {
            let slot = self.band_events[i];
            let mut v3 = 0.0;
            if b.flags & 2 != 0 {
                v3 = self.on_load;
            }
            if b.flags & 4 != 0 {
                v3 = self.off_load;
            }
            if gear > 100 {
                v3 = self.off_load;
            }
            let pitch = (v6 - (b.cents - b.cents_ref) as f32) as i32;
            if audio.slot(slot).is_some() {
                if b.lo >= v13 || v13 >= b.hi || b.flags == 0 {
                    audio.stop_slot(slot);
                }
            } else if v13 >= b.lo && b.hi >= v13 && b.flags & 1 != 0 {
                audio.play_event(emitter, b.event, None, 256, Some(slot));
                band_set_pitch(audio, slot, pitch);
            }
            if audio.slot(slot).is_some() {
                if b.lo + b.fade_in <= v13 {
                    if v13 <= b.hi - b.fade_out {
                        band_set_volume(audio, slot, b, (v3 * self.volume * 100.0) as i32);
                    } else {
                        let mut t = v13 - (b.hi - b.fade_out);
                        t /= b.fade_out;
                        t *= 100.0;
                        t = 100.0 - t;
                        t *= v3;
                        t *= self.volume;
                        band_set_volume(audio, slot, b, t as i32);
                    }
                } else {
                    let mut t = v13 - b.lo;
                    t /= b.fade_in;
                    t *= 100.0;
                    t *= v3;
                    t *= self.volume;
                    band_set_volume(audio, slot, b, t as i32);
                }
                band_set_pitch(audio, slot, pitch);
            }
        }
    }
}

/// VsAudioRevBands.cpp 0x0004D1E0 VsRevBand::SetPitch
fn band_set_pitch(audio: &mut AudioSystem, slot: EventSlot, cents: i32) {
    if let Some(ps) = audio.slot_sample(slot) {
        audio.set_sample_pitch(ps, cents as f32);
    }
}

/// VsAudioRevBands.cpp 0x0004D230 VsRevBand::SetVolume: percent scaled by the band volume, 0..100.
fn band_set_volume(audio: &mut AudioSystem, slot: EventSlot, b: &RevBand, percent: i32) {
    if let Some(ps) = audio.slot_sample(slot) {
        let v = ((percent as f32 * b.volume) as i32).clamp(0, 100);
        audio.set_sample_volume(ps, v as f32 / 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rbds() -> Vec<std::path::PathBuf> {
        let mut v = Vec::new();
        fn walk(d: &std::path::Path, v: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(d).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, v);
                } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("rbd")) {
                    v.push(p);
                }
            }
        }
        walk(visos::file::game_dir(), &mut v);
        v
    }

    #[test]
    fn every_rbd_parses() {
        let files = rbds();
        assert!(!files.is_empty());
        for p in &files {
            let b = std::fs::read(p).unwrap();
            let r = RevBandFile::parse(&b).unwrap();
            assert_eq!(b.len(), 164 + 40 * r.bands.len(), "{}", p.display());
            assert!(r.rpm_min < r.rpm_max && r.cents_min < r.cents_max);
            for band in &r.bands {
                assert!(band.lo < band.hi && band.flags & !7 == 0, "{}: {band:?}", p.display());
            }
            eprintln!("{}: {} bands, rpm {}..{}, group {}", p.display(), r.bands.len(), r.rpm_min, r.rpm_max, r.group);
        }
    }

    /// every vehicle database car's rev bands (`VehicleAudio::model`, as LoadRevBands gets it): which load, and
    /// that a loaded set always has a sounding band over the whole rpm sweep
    #[test]
    fn every_car_engine_plays_across_rpm() {
        let db = velocity_physics::vehicle_database::vehicle_database::CVehicleDatabase::load().unwrap();
        let mut names: Vec<String> = db.vehicles.iter().filter_map(|v| v.audio.model.clone()).collect();
        names.sort();
        names.dedup();
        let (mut ok, mut failed) = (Vec::new(), Vec::new());
        for name in names {
            let (mut audio, _mixer) = AudioSystem::with_default_mixer();
            let mut rb = VsAudioRevBands::default();
            rb.load(&mut audio, &name);
            if rb.rbd.is_none() {
                failed.push(name);
                continue;
            }
            rb.emitter = Some(audio.create_emitter(0x4000_0001).unwrap());
            rb.set_load(1.0);
            rb.start();
            let rbd = rb.rbd.clone().unwrap();
            let (mut lo_ratio, mut hi_ratio) = (0.0f32, 0.0f32);
            for step in 0..60 {
                let rpm = rbd.rpm_min + (rbd.rpm_max - rbd.rpm_min) * step / 59;
                rb.set_rpm(rpm);
                for _ in 0..10 {
                    rb.update(&mut audio, 3);
                    audio.update(1.0 / 60.0);
                }
                let playing: Vec<usize> = (0..rbd.bands.len()).filter(|&i| audio.slot(rb.band_events[i]).is_some()).collect();
                assert!(!playing.is_empty(), "{name} rpm {rpm}: no band");
                let r = playing.iter().filter_map(|&i| audio.slot_sample(rb.band_events[i])).map(|s| audio.samples.get(s).unwrap().freq_ratio).fold(0.0, f32::max);
                if step == 0 {
                    lo_ratio = r;
                }
                hi_ratio = r;
            }
            assert!(hi_ratio > 0.0 && lo_ratio > 0.0, "{name}: silent band");
            eprintln!("{name}: ratio {lo_ratio:.3} -> {hi_ratio:.3}");
            ok.push(name);
        }
        eprintln!("loaded {ok:?}, not shipped {failed:?}");
        assert!(!ok.is_empty());
    }

    #[test]
    fn engine_crossfades_and_tracks_rpm() {
        let (mut audio, mixer) = AudioSystem::with_default_mixer();
        let mut rb = VsAudioRevBands::default();
        assert!(rb.load(&mut audio, r"tier4\wrx\vimprrb0"));
        assert!(rb.rbd.is_some());
        rb.emitter = Some(audio.create_emitter(0x4000_0001).unwrap());
        rb.set_load(1.0);
        rb.start();
        let mut buf = vec![0.0f32; 1600];
        let mut prev_pitch = None;
        let rbd = rb.rbd.clone().unwrap();
        for step in 0..120 {
            let rpm = rbd.rpm_min + (rbd.rpm_max - rbd.rpm_min) * step / 119;
            rb.set_rpm(rpm);
            rb.update(&mut audio, 3);
            audio.update(1.0 / 60.0);
            mixer.lock().unwrap().render(&mut buf);
            let playing: Vec<usize> = (0..rbd.bands.len()).filter(|&i| audio.slot(rb.band_events[i]).is_some()).collect();
            assert!(!playing.is_empty() && playing.len() <= 4, "rpm {rpm}: {playing:?}");
            // on- and off-load bands overlap, so up to two pairs cross-fade at once
            // the lowest playing band's pitch rises with rpm
            let p = audio.slot_sample(rb.band_events[playing[0]]).map(|s| audio.samples.get(s).unwrap().pitch);
            if let (Some(a), Some(b)) = (prev_pitch, p) {
                assert!(b >= a - 1.0 || playing[0] != 0);
            }
            prev_pitch = p;
        }
        assert!(buf.iter().map(|v| v * v).sum::<f32>() > 0.0);
        rb.stop(&mut audio);
        assert!(rb.band_events.iter().all(|&s| audio.slot(s).is_none()));
    }
}
