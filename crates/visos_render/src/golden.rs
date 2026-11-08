//! dev harness (no original counterpart): golden-image captures and frame timing, the regression gate of the
//! performance stages (`tools/golden.py` drives it, details "Performance").
//! `VELOCITY_GOLDEN=<dir>`: every frame advances Bevy time by exactly 1/60 s (`TimeUpdateStrategy::ManualDuration`,
//! `VELOCITY_GOLDEN_HZ` changes it) and `clock_ns()` follows that virtual time, so a scripted run is the same frame
//! by frame. Each smoke-test screenshot (any `Screenshot` this module did not spawn) triggers a golden capture:
//! virtual time pauses (game clock, script, particles, front end all hold), the harness waits until no render
//! pipeline is still compiling, saves `<dir>/<VELOCITY_GOLDEN_PREFIX>_<nn>.png` and resumes.
//! `VELOCITY_GOLDEN_PERF=<frames>`: after each capture also time that many frames frozen and that many running
//! (wall clock, `<dir>/perf.csv`) plus the GPU pass timings of Bevy's render diagnostics (`<dir>/gpu.csv`).
//! `VELOCITY_PERF_OFF=envmap,road,mirror,stencil` turns those extra cameras off (A/B timing only).

use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy::render::render_resource::PipelineCache;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::time::{TimeSystems, TimeUpdateStrategy};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

static ON: AtomicBool = AtomicBool::new(false);
static CLOCK_NS: AtomicU64 = AtomicU64::new(0);
static WAITING: AtomicUsize = AtomicUsize::new(0);
/// census (`VELOCITY_GOLDEN_CAMS`): the FSE logs its next frame's pass list
pub(crate) static DUMP_FSE: AtomicBool = AtomicBool::new(false);

/// `VELOCITY_GOLDEN` is set
pub fn enabled() -> bool {
    std::env::var_os("VELOCITY_GOLDEN").is_some()
}

/// golden mode: the game clock's time (virtual, frame-locked, frozen while a capture settles); `None` = real time.
/// `GameClock::now` should return this when it is `Some` (core request).
pub fn clock_ns() -> Option<u64> {
    ON.load(Ordering::Relaxed).then(|| CLOCK_NS.load(Ordering::Relaxed))
}

/// render pipelines still compiling (as of the last rendered frame)
pub fn pipelines_waiting() -> usize {
    WAITING.load(Ordering::Relaxed)
}

/// `VELOCITY_PERF_OFF` names this part (envmap, road, mirror, stencil)
pub fn perf_off(name: &str) -> bool {
    static OFF: OnceLock<Vec<String>> = OnceLock::new();
    OFF.get_or_init(|| {
        std::env::var("VELOCITY_PERF_OFF").map(|v| v.split(',').map(|s| s.trim().to_ascii_lowercase()).collect()).unwrap_or_default()
    })
    .iter()
    .any(|s| s == name)
}

/// marks the harness' own screenshots
#[derive(Component)]
pub struct GoldenShot;

/// an explicit capture (trackview): `image` instead of the primary window, `exit` after it (and its timing)
pub struct GoldenRequest {
    pub image: Option<Handle<Image>>,
    pub exit: bool,
}

/// captures asked for by other systems (golden mode only; see `enabled`)
#[derive(Resource, Default)]
pub struct GoldenRequests(pub Vec<GoldenRequest>);

#[derive(Default)]
enum Phase {
    #[default]
    Idle,
    Settling { frames: u32, idle: u32 },
    AfterShot { frames: u32 },
    Measure { running: bool, samples: Vec<f32> },
}

#[derive(Resource)]
struct Golden {
    dir: PathBuf,
    prefix: String,
    perf: u32,
    queued: std::collections::VecDeque<GoldenRequest>,
    current: Option<GoldenRequest>,
    taken: u32,
    phase: Phase,
    last: Option<Instant>,
    /// wall time of the previous frame (ms)
    dt_ms: f32,
    frozen: (f32, f32, f32),
}

pub struct GoldenPlugin;

impl Plugin for GoldenPlugin {
    fn build(&self, app: &mut App) {
        if let Some(rapp) = app.get_sub_app_mut(RenderApp) {
            rapp.add_systems(Render, count_waiting.in_set(RenderSystems::Cleanup));
        }
        // dev: `VELOCITY_AMBIGUITY=1` logs bevy's system-order ambiguities of Update / PostUpdate (determinism hunts)
        if std::env::var_os("VELOCITY_AMBIGUITY").is_some() {
            use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
            let warn = || ScheduleBuildSettings { ambiguity_detection: LogLevel::Warn, ..Default::default() };
            app.edit_schedule(Update, |s| {
                s.set_build_settings(warn());
            });
            app.edit_schedule(PostUpdate, |s| {
                s.set_build_settings(warn());
            });
        }
        let Some(dir) = std::env::var_os("VELOCITY_GOLDEN").map(PathBuf::from) else { return };
        let _ = std::fs::create_dir_all(&dir);
        let hz: f64 = std::env::var("VELOCITY_GOLDEN_HZ").ok().and_then(|v| v.parse().ok()).unwrap_or(60.0);
        let perf: u32 = std::env::var("VELOCITY_GOLDEN_PERF").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        ON.store(true, Ordering::Relaxed);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / hz)))
            .insert_resource(Golden {
                dir,
                prefix: std::env::var("VELOCITY_GOLDEN_PREFIX").unwrap_or_else(|_| "golden".into()),
                perf,
                queued: Default::default(),
                current: None,
                taken: 0,
                phase: Phase::Idle,
                last: None,
                dt_ms: 0.0,
                frozen: (0.0, 0.0, 0.0),
            })
            .init_resource::<GoldenRequests>()
            .add_systems(First, sync_clock.after(TimeSystems))
            .add_systems(Last, step);
        if std::env::var_os("VELOCITY_GOLDEN_CAMS").is_some() {
            app.add_systems(Last, camera_census.after(step));
        }
        if perf > 0 {
            app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);
        }
    }
}

/// `VELOCITY_GOLDEN_CAMS=1`: at each capture append every active camera (order, target, non-bevy components) to
/// `<dir>/cameras.csv` (which views run a render graph per frame)
fn camera_census(world: &mut World, mut seen: Local<u32>) {
    let (taken, dir, prefix) = {
        let g = world.resource::<Golden>();
        (g.taken, g.dir.clone(), g.prefix.clone())
    };
    if taken == *seen {
        return;
    }
    *seen = taken;
    DUMP_FSE.store(true, Ordering::Relaxed);
    let mut q = world.query::<(Entity, &Camera, Option<&bevy::camera::RenderTarget>)>();
    let cams: Vec<(Entity, isize, String)> = q
        .iter(world)
        .filter(|(_, c, _)| c.is_active)
        .map(|(e, c, t)| {
            let t = match t {
                Some(bevy::camera::RenderTarget::Image(i)) => format!("image{:?}", i.handle.id()),
                Some(bevy::camera::RenderTarget::Window(_)) | None => "window".into(),
                Some(_) => "other".into(),
            };
            (e, c.order, t)
        })
        .collect();
    let mut out = String::new();
    for (e, order, t) in cams {
        use crate::full_screen_effects_manager_pc_xbox as fsem;
        let r = world.entity(e);
        let mut names: Vec<&str> = Vec::new();
        let mut tag = |on: bool, n: &'static str| {
            if on {
                names.push(n)
            }
        };
        tag(r.contains::<fsem::FsePassCamera>(), "fse");
        tag(r.contains::<fsem::RoadCamera>(), "road");
        tag(r.contains::<fsem::CarStencilCamera>(), "stencil");
        tag(r.contains::<crate::sky_render::SkyCamera>(), "sky");
        tag(r.contains::<crate::view_render::ViewSky>(), "viewsky");
        tag(r.contains::<crate::track_render::TrackCamera>(), "track");
        tag(r.contains::<crate::view_render::MirrorView>(), "mirror");
        tag(r.contains::<crate::view_render::PostViewCamera>(), "post");
        tag(r.contains::<crate::view_render::SecondaryTrackCamera>(), "second");
        tag(r.contains::<crate::rain_surface::FrontEndCamera>(), "fe");
        tag(r.contains::<Camera3d>(), "3d");
        tag(r.contains::<Camera2d>(), "2d");
        let layers = r.get::<bevy::camera::visibility::RenderLayers>().map(|l| format!("{:?}", l.iter().collect::<Vec<_>>())).unwrap_or_default();
        out += &format!("{prefix},{},{order},{t},{} {layers}\n", taken - 1, names.join(" "));
    }
    append(dir.join("cameras.csv"), "prefix,shot,order,target,components\n", &out);
}

fn count_waiting(cache: Res<PipelineCache>) {
    WAITING.store(cache.waiting_pipelines().count(), Ordering::Relaxed);
}

fn sync_clock(time: Res<Time<Virtual>>) {
    CLOCK_NS.store(time.elapsed().as_nanos() as u64, Ordering::Relaxed);
}

fn stats(samples: &mut [f32]) -> (f32, f32, f32) {
    if samples.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let mean = samples.iter().sum::<f32>() / samples.len() as f32;
    samples.sort_by(f32::total_cmp);
    let p = |q: f32| samples[((samples.len() - 1) as f32 * q) as usize];
    (mean, p(0.5), p(0.99))
}

fn append(path: PathBuf, header: &str, line: &str) {
    let new = !path.exists();
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        if new {
            let _ = f.write_all(header.as_bytes());
        }
        let _ = f.write_all(line.as_bytes());
    }
}

fn step(
    mut commands: Commands,
    mut g: ResMut<Golden>,
    mut time: ResMut<Time<Virtual>>,
    fresh: Query<(), (Added<Screenshot>, Without<GoldenShot>)>,
    mut requests: ResMut<GoldenRequests>,
    diags: Option<Res<DiagnosticsStore>>,
    mut exit: MessageWriter<AppExit>,
) {
    let now = Instant::now();
    g.dt_ms = g.last.map_or(0.0, |l| (now - l).as_secs_f32() * 1000.0);
    g.last = Some(now);
    if !fresh.is_empty() {
        g.queued.push_back(GoldenRequest { image: None, exit: false });
    }
    g.queued.extend(requests.0.drain(..));
    let dt = g.dt_ms;
    let perf = g.perf;
    let phase = std::mem::take(&mut g.phase);
    if matches!(phase, Phase::Idle) && g.current.as_ref().is_some_and(|c| c.exit) {
        exit.write(AppExit::Success);
        return;
    }
    g.phase = match phase {
        Phase::Idle if !g.queued.is_empty() => {
            g.current = g.queued.pop_front();
            time.pause();
            Phase::Settling { frames: 0, idle: 0 }
        }
        Phase::Idle => Phase::Idle,
        Phase::Settling { frames, idle } => {
            let idle = if pipelines_waiting() == 0 { idle + 1 } else { 0 };
            // `VELOCITY_GOLDEN_SETTLE=<n>`: at least n paused frames (default 6), so runs whose pipeline sets differ
            // pause equally long (anything stepping per frame while paused would otherwise drift)
            let min = std::env::var("VELOCITY_GOLDEN_SETTLE").ok().and_then(|v| v.parse().ok()).unwrap_or(6u32);
            if (frames >= min && idle >= 3) || frames >= 900 {
                if frames >= 900 {
                    log::warn!("golden: pipelines still compiling after 900 frames");
                }
                let path = g.dir.join(format!("{}_{:02}.png", g.prefix, g.taken));
                log::info!("golden: capture {} (settled {frames} frames)", path.display());
                g.taken += 1;
                let shot = match g.current.as_ref().and_then(|c| c.image.clone()) {
                    Some(image) => Screenshot::image(image),
                    None => Screenshot::primary_window(),
                };
                commands.spawn((shot, GoldenShot)).observe(save_to_disk(path));
                Phase::AfterShot { frames: 0 }
            } else {
                Phase::Settling { frames: frames + 1, idle }
            }
        }
        Phase::AfterShot { frames } if frames < 4 => Phase::AfterShot { frames: frames + 1 },
        Phase::AfterShot { .. } if perf > 0 => Phase::Measure { running: false, samples: Vec::new() },
        Phase::AfterShot { .. } => {
            time.unpause();
            Phase::Idle
        }
        Phase::Measure { running, mut samples } => {
            samples.push(dt);
            if (samples.len() as u32) < perf {
                Phase::Measure { running, samples }
            } else if !running {
                g.frozen = stats(&mut samples);
                time.unpause();
                Phase::Measure { running: true, samples: Vec::new() }
            } else {
                let (rm, r50, r99) = stats(&mut samples);
                let (fm, f50, f99) = g.frozen;
                let k = g.taken - 1;
                let line = format!(
                    "{},{k},{:.1},{fm:.3},{f50:.3},{f99:.3},{:.1},{rm:.3},{r50:.3},{r99:.3}\n",
                    g.prefix,
                    1000.0 / fm.max(1e-3),
                    1000.0 / rm.max(1e-3)
                );
                append(
                    g.dir.join("perf.csv"),
                    "prefix,shot,frozen_fps,frozen_mean_ms,frozen_p50_ms,frozen_p99_ms,run_fps,run_mean_ms,run_p50_ms,run_p99_ms\n",
                    &line,
                );
                if let Some(d) = diags {
                    let mut out = String::new();
                    for d in d.iter().filter(|d| d.path().as_str().starts_with("render/")) {
                        if let Some(v) = d.average() {
                            out += &format!("{},{k},{},{v:.4}\n", g.prefix, d.path().as_str());
                        }
                    }
                    append(g.dir.join("gpu.csv"), "prefix,shot,path,avg\n", &out);
                }
                Phase::Idle
            }
        }
    };
}
