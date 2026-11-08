//! port diagnostics, not in the original: `VELOCITY_AUDIO_TRACE=<file>` appends a once-a-second summary of the
//! audio state (frame deltas, player engine input, active events with their pitch ratio, PlayEvent calls and
//! failures, music) so a headless run can be checked for missing or wrong-rate sounds.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write as _;

use bevy::prelude::*;

use crate::audio::xvg::XvgGroup;
use crate::input::GameAudioInput;
use crate::plugin::VisosAudio;

#[derive(Resource)]
pub struct AudioTrace {
    file: std::fs::File,
    names: HashMap<u32, String>,
    t: f32,
    frames: u32,
    dt_min: f32,
    dt_max: f32,
    engine: bool,
}

fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("xvg")) {
            out.push(p);
        }
    }
}

pub fn build(app: &mut App) {
    let Some(path) = std::env::var_os("VELOCITY_AUDIO_TRACE") else { return };
    let Ok(file) = std::fs::File::create(&path) else { return };
    let mut files = Vec::new();
    walk(visos::file::game_dir(), &mut files);
    let mut names = HashMap::new();
    for p in files {
        if let Ok((g, _)) = std::fs::read(&p).map_err(|_| ()).and_then(|b| XvgGroup::from_bytes(&b).map_err(|_| ())) {
            for e in g.events {
                names.entry(e.hash).or_insert(e.name);
            }
        }
    }
    app.insert_resource(AudioTrace { file, names, t: 0.0, frames: 0, dt_min: f32::MAX, dt_max: 0.0,
        engine: std::env::var_os("VELOCITY_AUDIO_TRACE_ENGINE").is_some() })
        .add_systems(Startup, |mut a: ResMut<VisosAudio>| {
            a.system.trace_plays = Some(Vec::new());
            a.system.trace_cmds = Some(Vec::new());
        })
        .add_systems(Update, trace.after(crate::plugin::AudioSystems::Update));
}

/// `VELOCITY_AUDIO_TRACE_ENGINE=1`: one line per frame with the engine state: player rpm in (0x001B6F10), gear,
/// throttle, UpdateRpm's x1.25 (+0x8C), rev band smoothed norm, each playing band `index:cents/volume`, then per
/// ai racer `rpm:cents/volume` of its engine loop.
fn engine_line(s: &mut String, audio: &VisosAudio, input: &GameAudioInput) {
    let Some(p) = &input.player else { return };
    let (a, g) = (&audio.system, &audio.game);
    let rb = &g.events.rev_bands;
    let va_rpm = g.player_cars.first().map(|v| v.rpm).unwrap_or(0.0);
    let _ = write!(s, "E rpm {} gear {} thr {:.2} va {:.0} rb {} n {:.3} sm {:.3} |", p.engine_rpm, p.gear, p.throttle,
        va_rpm, rb.rpm, rb.norm_rpm, rb.smooth);
    for (i, &slot) in rb.band_events.iter().enumerate() {
        if let Some(smp) = a.slot_sample(slot).and_then(|id| a.samples.get(id)) {
            let _ = write!(s, " {i}:{:.0}/{:.2}", smp.pitch, smp.volume);
        }
    }
    let _ = write!(s, " | ai");
    for va in g.ai_cars.iter().skip(1) {
        match a.slot_sample(va.engine).and_then(|id| a.samples.get(id)) {
            Some(smp) => {
                let _ = write!(s, " {:.0}:{:.0}/{:.2}", va.last_rpm, smp.pitch, smp.volume);
            }
            None => s.push_str(" -"),
        }
    }
    s.push('\n');
}

fn trace(mut tr: ResMut<AudioTrace>, mut audio: ResMut<VisosAudio>, input: Res<GameAudioInput>, time: Res<Time>) {
    if tr.engine {
        let mut s = String::new();
        engine_line(&mut s, &audio, &input);
        let _ = tr.file.write_all(s.as_bytes());
    }
    let dt = time.delta_secs();
    tr.t += dt;
    tr.frames += 1;
    tr.dt_min = tr.dt_min.min(dt);
    tr.dt_max = tr.dt_max.max(dt);
    if tr.t < 1.0 {
        return;
    }
    let name = |h: u32| tr.names.get(&h).cloned().unwrap_or_else(|| format!("{h:#010x}"));
    let mut s = String::new();
    let _ = write!(s, "t+{:.2}s frames {} dt {:.4}..{:.4}", tr.t, tr.frames, tr.dt_min, tr.dt_max);
    if let Some(p) = &input.player {
        let _ = write!(s, " | player rpm {} gear {} speed {:.1}", p.engine_rpm, p.gear, p.speed);
    }
    let a = &audio.system;
    let mut active: Vec<String> = Vec::new();
    let mut e = a.event_head;
    while let Some(id) = e {
        let Some(ev) = a.events.get(id) else { break };
        let h = a.groups.get(ev.event.group).and_then(|g| g.events.get(ev.event.event)).map(|g| g.hash).unwrap_or(0);
        let ratio = ev.playing.and_then(|p| a.samples.get(p)).map(|p| p.freq_ratio).unwrap_or(0.0);
        active.push(format!("{}@{:.2}", name(h), ratio));
        e = ev.next;
    }
    active.sort();
    let _ = write!(s, "\n  active[{}]: {}", active.len(), active.join(" "));
    let g = &audio.game;
    let _ = write!(s, "\n  music: state {:?} st {} song {} list {}", g.soundtrack.state, g.soundtrack.soundtrack, g.soundtrack.song,
        g.tracks.current_list);
    let plays = audio.system.trace_plays.replace(Vec::new()).unwrap_or_default();
    let mut counts: HashMap<(u32, bool), u32> = HashMap::new();
    for (h, r) in plays {
        *counts.entry((h, r == 0)).or_default() += 1;
    }
    let mut pl: Vec<String> = counts.iter().map(|((h, ok), n)| format!("{}{}x{n}", name(*h), if *ok { "" } else { "!FAIL" })).collect();
    pl.sort();
    let _ = write!(s, "\n  plays: {}", pl.join(" "));
    let cmds = audio.system.trace_cmds.replace(Vec::new()).unwrap_or_default();
    let mut cc: Vec<(String, u32)> = Vec::new();
    for c in cmds {
        match cc.iter_mut().find(|(n, _)| *n == c) {
            Some(e) => e.1 += 1,
            None => cc.push((c, 1)),
        }
    }
    let cc: Vec<String> = cc.iter().map(|(n, k)| format!("{n}x{k}")).collect();
    let _ = writeln!(s, "\n  cmds: {}", cc.join(" "));
    let _ = tr.file.write_all(s.as_bytes());
    tr.t = 0.0;
    tr.frames = 0;
    tr.dt_min = f32::MAX;
    tr.dt_max = 0.0;
}
