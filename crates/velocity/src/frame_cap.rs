//! dev options (not original): `--fps <n>` caps the render rate at n frames per second with vsync off
//! (`--fps 0` = vsync off, uncapped; without the option the PC settings page's frame cap / v-sync apply, see
//! velocity_frontend `pc_settings.rs`), and
//! `--frame-log <file>` writes one csv line per rendered in-race frame (real time, lerp, camera, player car, and
//! the update's own camera view) to check that the camera and the cars move smoothly at any rate.

use std::io::Write;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::window::PresentMode;
use velocity_game::camera::game_cameras::GameCameras;
use velocity_game::game_mode::race_cars::RaceCars;
use velocity_game::vs_common_main::GameClock;
use velocity_game::EGameMode;
use velocity_frontend::pc_settings::FrameRateCap;
use visos_render::car_render::CarRender;

fn arg_value(name: &str) -> Option<String> {
    let mut it = std::env::args();
    while let Some(a) = it.next() {
        if a == name {
            return it.next();
        }
    }
    None
}

/// `--fps` value: None = not given, Some(0) = uncapped
pub fn fps_arg() -> Option<u32> {
    arg_value("--fps").map(|v| v.parse().unwrap_or(0))
}

pub fn present_mode(fps: Option<u32>) -> PresentMode {
    if fps.is_some() { PresentMode::AutoNoVsync } else { PresentMode::default() }
}

#[derive(Resource)]
struct Limiter {
    period: Duration,
    next: Instant,
    fps: Option<u32>,
}

/// sleep out the rest of the frame (coarse sleep, then spin the last 1.5 ms). The cap comes from
/// `velocity_frontend::pc_settings::FrameRateCap` (`--fps`, else the PC settings page), so it can change live.
fn limit(mut l: ResMut<Limiter>, cap: Res<FrameRateCap>) {
    if l.fps != cap.fps {
        l.fps = cap.fps;
        info!("frame cap: {:?} fps", cap.fps);
        if let Some(n) = cap.fps {
            l.period = Duration::from_secs_f64(1.0 / n as f64);
            l.next = Instant::now() + l.period;
        }
    }
    if l.fps.is_none() {
        return;
    }
    let now = Instant::now();
    if l.next > now {
        let wait = l.next - now;
        if wait > Duration::from_micros(1500) {
            std::thread::sleep(wait - Duration::from_micros(1500));
        }
        while Instant::now() < l.next {
            std::hint::spin_loop();
        }
        let p = l.period;
        l.next += p;
    } else {
        // late: restart the schedule from now
        l.next = now + l.period;
    }
}

#[derive(Resource)]
struct FrameLog {
    file: std::fs::File,
    start: Instant,
}

fn frame_log(
    mut log: ResMut<FrameLog>,
    clock: Res<GameClock>,
    cars: Res<RaceCars>,
    cams: Res<GameCameras>,
    tfs: Query<&GlobalTransform>,
    roots: Query<(Entity, &CarRender)>,
) {
    let Some(cam) = cars.camera.and_then(|c| tfs.get(c).ok()) else { return };
    let Some(car) = roots.iter().find(|(_, r)| r.request == 0).and_then(|(e, _)| tfs.get(e).ok()) else { return };
    let Some(raw) = cams.manager.view(0) else { return };
    let (c, f, k) = (cam.translation(), cam.forward(), car.translation());
    let t = log.start.elapsed().as_secs_f64();
    let line = format!(
        "{t:.6},{},{:.5},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
        clock.timer.total_updates, clock.timer.lerp,
        c.x, c.y, c.z, f.x, f.y, f.z, k.x, k.y, k.z,
        raw.position.x, raw.position.y, raw.position.z, raw.target.x, raw.target.y, raw.target.z
    );
    let _ = log.file.write_all(line.as_bytes());
}

pub fn build(app: &mut App, fps: Option<u32>) {
    // `--fps` wins over the PC settings page (pc_settings skips the cap and v-sync while `cli` is set)
    app.insert_resource(FrameRateCap { fps: fps.filter(|&n| n > 0), cli: fps.is_some() })
        .insert_resource(Limiter { period: Duration::from_millis(16), next: Instant::now(), fps: None })
        .add_systems(Last, limit);
    if let Some(path) = arg_value("--frame-log") {
        match std::fs::File::create(&path) {
            Ok(mut file) => {
                let _ = file.write_all(b"time,updates,lerp,cx,cy,cz,fx,fy,fz,kx,ky,kz,rx,ry,rz,tx,ty,tz\n");
                app.insert_resource(FrameLog { file, start: Instant::now() })
                    .add_systems(Last, frame_log.run_if(in_state(EGameMode::InGame)));
            }
            Err(e) => log::error!("--frame-log {path}: {e}"),
        }
    }
}
