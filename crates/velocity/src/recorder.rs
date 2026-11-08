//! dev tool: `SMOKE_RECORD=<dir>` records the primary window at 30 fps of real time into `<dir>/%05d.png`
//! (downscaled to fit 640x480), and runs ffmpeg into `<dir>.mp4` on exit when it is on PATH.
//! `SMOKE_RECORD_RACE=1` records only while in a race. Sound: Bevy's audio output is disabled while recording
//! (main.rs) and a thread here pulls the visos mixer at real-time rate instead (so the mix advances exactly as
//! the device would), writing it to `<dir>/audio.wav` from the first recorded frame on; ffmpeg muxes it in.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use velocity_audio::audio::mixer::{SharedMixer, MIX_RATE};
use velocity_game::EGameMode;

/// `SMOKE_RECORD` is set
pub fn enabled() -> bool {
    std::env::var_os("SMOKE_RECORD").is_some()
}

/// the mix pump: renders `MIX_RATE` stereo frames per real second; while `on` they go to the wav
struct AudioTap {
    on: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<u64>>,
    wav: PathBuf,
}

fn wav_header(f: &mut std::fs::File, frames: u64) -> std::io::Result<()> {
    let data = (frames * 4) as u32;
    f.seek(SeekFrom::Start(0))?;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    // pcm, 2 channels, rate, byte rate, block align 4, 16 bits
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&2u16.to_le_bytes())?;
    f.write_all(&MIX_RATE.to_le_bytes())?;
    f.write_all(&(MIX_RATE * 4).to_le_bytes())?;
    f.write_all(&4u16.to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&data.to_le_bytes())
}

fn start_tap(mixer: SharedMixer, wav: &Path) -> Option<AudioTap> {
    let mut f = std::fs::File::create(wav).map_err(|e| log::error!("recorder: {}: {e}", wav.display())).ok()?;
    wav_header(&mut f, 0).ok()?;
    let on = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(AtomicBool::new(false));
    let (on2, stop2) = (on.clone(), stop.clone());
    let worker = std::thread::spawn(move || {
        let t0 = std::time::Instant::now();
        let (mut rendered, mut written) = (0u64, 0u64);
        let mut buf = Vec::new();
        let mut bytes = Vec::new();
        while !stop2.load(Ordering::Relaxed) {
            let due = (t0.elapsed().as_secs_f64() * MIX_RATE as f64) as u64;
            if due > rendered {
                let n = (due - rendered) as usize;
                buf.resize(n * 2, 0.0);
                match mixer.lock() {
                    Ok(mut m) => m.render(&mut buf),
                    Err(_) => buf.fill(0.0),
                }
                rendered = due;
                if on2.load(Ordering::Relaxed) {
                    bytes.clear();
                    for s in &buf {
                        bytes.extend_from_slice(&((s * 32767.0) as i16).to_le_bytes());
                    }
                    if f.write_all(&bytes).is_ok() {
                        written += n as u64;
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let _ = wav_header(&mut f, written);
        written
    });
    Some(AudioTap { on, stop, worker: Some(worker), wav: wav.to_path_buf() })
}

const FPS: f64 = 30.0;
const MAX_W: u32 = 640;
const MAX_H: u32 = 480;

/// one captured image and the output frame numbers it fills (more than one when rendering is below 30 fps)
type Job = (Image, std::ops::Range<u32>);

/// `None` stops the encoder
type Msg = Option<Job>;

#[derive(Resource)]
struct Recorder {
    dir: PathBuf,
    race_only: bool,
    /// real time recorded so far
    t: f64,
    /// frames requested so far
    frames: u32,
    tx: Option<Sender<Msg>>,
    worker: Option<JoinHandle<u32>>,
    tap: Option<AudioTap>,
    /// `SMOKE_RECORD_FROM` and the recordable time seen so far
    from: f64,
    skipped: f64,
}

pub fn build(app: &mut App) {
    let Some(dir) = std::env::var_os("SMOKE_RECORD").map(PathBuf::from) else { return };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        log::error!("recorder: cannot create {}: {e}", dir.display());
        return;
    }
    // stale frames from an earlier run would end up in the video
    for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
        if e.path().extension().is_some_and(|x| x == "png") {
            let _ = std::fs::remove_file(e.path());
        }
    }
    let (tx, rx) = channel::<Msg>();
    let out = dir.clone();
    // png encoding off the main thread
    let worker = std::thread::spawn(move || {
        let mut written = 0;
        for (img, frames) in rx.into_iter().map_while(|m| m) {
            let Ok(dynimg) = img.try_into_dynamic() else { continue };
            let mut rgb = dynimg.to_rgb8();
            let (w, h) = rgb.dimensions();
            if w > MAX_W || h > MAX_H {
                let s = (MAX_W as f32 / w as f32).min(MAX_H as f32 / h as f32);
                // even sizes for yuv420p
                let (nw, nh) = (((w as f32 * s) as u32) & !1, ((h as f32 * s) as u32) & !1);
                rgb = image::imageops::resize(&rgb, nw.max(2), nh.max(2), image::imageops::FilterType::Triangle);
            } else if w % 2 != 0 || h % 2 != 0 {
                rgb = image::imageops::crop_imm(&rgb, 0, 0, w & !1, h & !1).to_image();
            }
            let first = out.join(format!("{:05}.png", frames.start));
            if let Err(e) = rgb.save(&first) {
                log::error!("recorder: {}: {e}", first.display());
                continue;
            }
            // a capture that never arrived leaves a hole that would end ffmpeg's sequence: fill with the last frame
            if frames.start > written && written > 0 {
                let prev = out.join(format!("{:05}.png", written - 1));
                for n in written..frames.start {
                    let _ = std::fs::copy(&prev, out.join(format!("{n:05}.png")));
                }
            }
            for n in frames.start + 1..frames.end {
                let _ = std::fs::copy(&first, out.join(format!("{n:05}.png")));
            }
            written = written.max(frames.end);
        }
        written
    });
    let race_only = std::env::var("SMOKE_RECORD_RACE").is_ok_and(|v| v == "1");
    log::info!("recorder: {} at {FPS} fps{}", dir.display(), if race_only { " (race only)" } else { "" });
    let tap = app.world().get_resource::<velocity_audio::VisosAudio>().and_then(|a| start_tap(a.mixer.clone(), &dir.join("audio.wav")));
    let from = std::env::var("SMOKE_RECORD_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    app.insert_resource(Recorder { dir, race_only, t: 0.0, frames: 0, tx: Some(tx), worker: Some(worker), tap, from, skipped: 0.0 })
        .add_systems(Update, capture)
        .add_systems(Last, finish);
}

/// samples by elapsed real time: each due 1/30 s step gets the current frame
fn capture(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mode: Option<Res<State<EGameMode>>>,
    mut r: ResMut<Recorder>,
) {
    if r.race_only && mode.is_none_or(|m| *m.get() != EGameMode::InGame) {
        return;
    }
    // SMOKE_RECORD_FROM=<secs>: start only after that much (real) recordable time, e.g. late in a long race
    r.skipped += time.delta_secs_f64();
    if r.skipped < r.from {
        return;
    }
    if let Some(tap) = &r.tap {
        tap.on.store(true, Ordering::Relaxed);
    }
    r.t += time.delta_secs_f64();
    let due = (r.t * FPS) as u32 + 1;
    if due <= r.frames {
        return;
    }
    let frames = r.frames..due;
    r.frames = due;
    let Some(tx) = r.tx.clone() else { return };
    commands.spawn(Screenshot::primary_window()).observe(move |c: On<ScreenshotCaptured>| {
        let _ = tx.send(Some((c.image.clone(), frames.clone())));
    });
}

/// on exit: wait for the encoder, then ffmpeg the frames into `<dir>.mp4`
fn finish(mut exit: MessageReader<AppExit>, mut r: ResMut<Recorder>) {
    if exit.read().next().is_none() {
        return;
    }
    // in-flight screenshots never arrive once the app stops: stop after what is queued
    if let Some(tx) = r.tx.take() {
        let _ = tx.send(None);
    }
    let Some(worker) = r.worker.take() else { return };
    let n = worker.join().unwrap_or(0);
    let mut wav = None;
    if let Some(mut tap) = r.tap.take() {
        tap.stop.store(true, Ordering::Relaxed);
        let frames = tap.worker.take().and_then(|w| w.join().ok()).unwrap_or(0);
        log::info!("recorder: {:.1} s of audio in {}", frames as f64 / MIX_RATE as f64, tap.wav.display());
        if frames > 0 {
            wav = Some(tap.wav);
        }
    }
    let mp4 = r.dir.with_extension("mp4");
    log::info!("recorder: {n} frames in {}", r.dir.display());
    let pattern = r.dir.join("%05d.png");
    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.args(["-y", "-loglevel", "error", "-framerate", "30", "-i"]).arg(&pattern);
    if let Some(w) = &wav {
        cmd.arg("-i").arg(w).args(["-c:a", "aac", "-b:a", "160k", "-shortest"]);
    }
    let status = cmd.args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "23"]).arg(&mp4).status();
    match status {
        Ok(s) if s.success() => log::info!("recorder: wrote {}", mp4.display()),
        Ok(s) => log::error!("recorder: ffmpeg failed ({s}), frames left in {}", r.dir.display()),
        Err(_) => log::info!("recorder: no ffmpeg on PATH, frames left in {}", r.dir.display()),
    }
}
