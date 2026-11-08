//! the sound device side, owned here instead of bevy_audio (which opens the default device once at startup and
//! never reopens): a supervisor thread opens the default output, plays the shared mix on it, and reopens when the
//! stream reports an error or the default device changes. the mixer (all voices, music) is never touched by a
//! reopen, only the device stream is rebuilt. every open / reopen / failure is logged (log::info / warn).

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::mixer::{MixerStream, SharedMixer};

/// poll period of the supervisor
const POLL: Duration = Duration::from_millis(500);
/// wait before retrying after a failed open
const RETRY: Duration = Duration::from_secs(2);

/// what the supervisor needs from the platform; the real one is cpal/rodio, tests use a fake.
pub trait Backend {
    /// the open stream (dropping it closes the device); may be `!Send`, so it lives on the supervisor thread
    type Sink;
    /// id of the current default output device (None = no device)
    fn default_id(&mut self) -> Option<String>;
    /// open the default device and start playing `mixer` on it; `errors` is set by the stream's error callback.
    /// returns the sink and a description (device name + format) for the log.
    fn open(&mut self, mixer: &SharedMixer, errors: Arc<AtomicBool>) -> Result<(Self::Sink, String), String>;
}

pub struct Supervisor<B: Backend> {
    backend: B,
    mixer: SharedMixer,
    sink: Option<B::Sink>,
    open_id: Option<String>,
    errors: Arc<AtomicBool>,
    retry_at: Option<Instant>,
    fails: u32,
    pub opens: u32,
}

impl<B: Backend> Supervisor<B> {
    pub fn new(backend: B, mixer: SharedMixer) -> Self {
        Self { backend, mixer, sink: None, open_id: None, errors: Arc::new(AtomicBool::new(false)), retry_at: None, fails: 0, opens: 0 }
    }

    pub fn is_open(&self) -> bool {
        self.sink.is_some()
    }

    /// one supervision step: (re)open when closed, after a stream error, or when the default device changed.
    pub fn poll(&mut self, now: Instant) {
        let cur = self.backend.default_id();
        let mut why = None;
        if self.sink.is_none() {
            if self.retry_at.is_none_or(|t| now >= t) {
                why = Some(if self.opens == 0 { "start" } else { "retry" });
            }
        } else if self.errors.swap(false, Relaxed) {
            why = Some("stream error");
        } else if cur.is_some() && cur != self.open_id {
            why = Some("default device changed");
        }
        let Some(why) = why else { return };
        // close the old stream first (the mix state lives in the shared mixer and survives)
        let had = self.sink.take().is_some();
        self.errors.store(false, Relaxed);
        match self.backend.open(&self.mixer, self.errors.clone()) {
            Ok((sink, desc)) => {
                self.opens += 1;
                self.fails = 0;
                self.retry_at = None;
                self.sink = Some(sink);
                self.open_id = cur;
                log::info!("audio: output {} ({why}{}): {desc}", if self.opens == 1 { "opened" } else { "reopened" }, if had { ", old stream closed" } else { "" });
            }
            Err(e) => {
                self.fails += 1;
                self.retry_at = Some(now + RETRY);
                self.open_id = cur;
                // first failure, then about every 30 s
                if self.fails == 1 || self.fails % 15 == 0 {
                    log::warn!("audio: output open failed ({why}, attempt {}): {e}", self.fails);
                }
            }
        }
    }

    pub fn run(mut self) -> ! {
        loop {
            self.poll(Instant::now());
            std::thread::sleep(POLL);
        }
    }
}

/// cpal / rodio device output
pub struct DeviceBackend;

fn id_of(d: &cpal::Device) -> Option<String> {
    use cpal::traits::DeviceTrait;
    d.id().ok().map(|i| i.to_string())
}

fn name_of(d: &cpal::Device) -> String {
    use cpal::traits::DeviceTrait;
    d.description().map(|x| x.name().to_string()).unwrap_or_else(|_| "?".into())
}

fn open_device(d: cpal::Device, mixer: &SharedMixer, errors: &Arc<AtomicBool>) -> Result<(rodio::MixerDeviceSink, String), String> {
    let name = name_of(&d);
    let id = id_of(&d).unwrap_or_default();
    let e = errors.clone();
    let b = rodio::DeviceSinkBuilder::from_device(d)
        .map_err(|e| format!("'{name}': {e}"))?
        .with_error_callback(move |err| {
            log::warn!("audio: output stream error: {err}");
            e.store(true, Relaxed);
        });
    let mut sink = b.open_sink_or_fallback().map_err(|e| format!("'{name}': {e}"))?;
    sink.log_on_drop(false);
    let c = sink.config();
    let desc = format!("'{name}' id {id} {} ch {} Hz {:?} buffer {:?} (mix 48000 Hz stereo f32)", c.channel_count(), c.sample_rate(), c.sample_format(), c.buffer_size());
    sink.mixer().add(MixerStream::new(mixer.clone()));
    Ok((sink, desc))
}

impl Backend for DeviceBackend {
    type Sink = rodio::MixerDeviceSink;

    fn default_id(&mut self) -> Option<String> {
        use cpal::traits::HostTrait;
        cpal::default_host().default_output_device().and_then(|d| id_of(&d))
    }

    fn open(&mut self, mixer: &SharedMixer, errors: Arc<AtomicBool>) -> Result<(Self::Sink, String), String> {
        use cpal::traits::HostTrait;
        let host = cpal::default_host();
        let def = host.default_output_device().ok_or("no default output device")?;
        match open_device(def, mixer, &errors) {
            Ok(r) => Ok(r),
            Err(first) => {
                // the default will not open: take any other real output so the game is not silent
                if let Ok(devs) = host.output_devices() {
                    for d in devs {
                        let n = name_of(&d);
                        if let Ok((s, desc)) = open_device(d, mixer, &errors) {
                            return Ok((s, format!("{desc} [fallback, default failed: {first}; used '{n}']")));
                        }
                    }
                }
                Err(first)
            }
        }
    }
}

/// start the supervisor thread on the real device.
pub fn spawn(mixer: SharedMixer) {
    let _ = std::thread::Builder::new().name("visos audio output".into()).spawn(move || {
        Supervisor::new(DeviceBackend, mixer).run();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct Fake {
        default: Arc<Mutex<Option<String>>>,
        fail_next: Arc<AtomicBool>,
        last_errors: Arc<Mutex<Option<Arc<AtomicBool>>>>,
        log: Arc<Mutex<Vec<String>>>,
    }

    impl Backend for Fake {
        type Sink = ();
        fn default_id(&mut self) -> Option<String> {
            self.default.lock().unwrap().clone()
        }
        fn open(&mut self, _m: &SharedMixer, errors: Arc<AtomicBool>) -> Result<((), String), String> {
            if self.fail_next.swap(false, Relaxed) {
                return Err("busy".into());
            }
            let id = self.default.lock().unwrap().clone().unwrap_or_default();
            self.log.lock().unwrap().push(id.clone());
            *self.last_errors.lock().unwrap() = Some(errors);
            Ok(((), format!("fake {id}")))
        }
    }

    /// real default device, muted (`cargo test -p velocity_audio -- --ignored real_device`): opens and stays open.
    #[test]
    #[ignore]
    fn real_device_muted() {
        super::super::mixer::diag::MUTE.store(true, Relaxed);
        let mixer: SharedMixer = Arc::new(Mutex::new(super::super::mixer::Mixer::new(4)));
        let mut s = Supervisor::new(DeviceBackend, mixer.clone());
        let t = Instant::now();
        s.poll(t);
        assert!(s.is_open(), "default device opens");
        std::thread::sleep(Duration::from_millis(800));
        assert!(mixer.lock().unwrap().clock > 0, "device pulls the mix");
        s.poll(t);
        assert_eq!(s.opens, 1);
    }

    /// stream error and default-device change reopen the output; the mixer (and its clock) is untouched.
    #[test]
    fn reopens_on_error_and_default_change() {
        let mixer: SharedMixer = Arc::new(Mutex::new(super::super::mixer::Mixer::new(4)));
        let default = Arc::new(Mutex::new(Some("A".to_string())));
        let fail_next = Arc::new(AtomicBool::new(false));
        let last_errors = Arc::new(Mutex::new(None));
        let log = Arc::new(Mutex::new(Vec::new()));
        let fake = Fake { default: default.clone(), fail_next: fail_next.clone(), last_errors: last_errors.clone(), log: log.clone() };
        let mut s = Supervisor::new(fake, mixer.clone());
        let t0 = Instant::now();
        s.poll(t0);
        assert!(s.is_open() && s.opens == 1);
        mixer.lock().unwrap().render(&mut vec![0.0; 960]);
        let clock = mixer.lock().unwrap().clock;
        s.poll(t0);
        assert_eq!(s.opens, 1, "stable: no reopen");
        // simulated stream error (what cpal's error callback does)
        last_errors.lock().unwrap().as_ref().unwrap().store(true, Relaxed);
        s.poll(t0);
        assert_eq!(s.opens, 2, "error -> reopen");
        // default device switched
        *default.lock().unwrap() = Some("B".into());
        s.poll(t0);
        assert_eq!(s.opens, 3);
        assert_eq!(*log.lock().unwrap(), ["A", "A", "B"]);
        // device list momentarily empty: keep the stream
        *default.lock().unwrap() = None;
        s.poll(t0);
        assert_eq!(s.opens, 3);
        // failed open: closed, retried after the delay only
        *default.lock().unwrap() = Some("C".into());
        fail_next.store(true, Relaxed);
        s.poll(t0);
        assert!(!s.is_open());
        s.poll(t0 + Duration::from_millis(500));
        assert!(!s.is_open(), "waits for the retry delay");
        s.poll(t0 + RETRY);
        assert!(s.is_open() && s.opens == 4);
        assert_eq!(mixer.lock().unwrap().clock, clock, "mix state survives reopens");
    }
}
