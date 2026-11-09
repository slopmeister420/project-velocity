//! engine-level checks against the real data: load groups, play events, tick the engine and measure the
//! rendered mix.

use bevy::math::Vec3;

use super::system::*;
use super::xvg::event_hash;

fn rms(b: &[f32]) -> f32 {
    (b.iter().map(|v| v * v).sum::<f32>() / b.len().max(1) as f32).sqrt()
}

/// tick the engine at 60 Hz and render the matching mix; returns the rendered stereo frames.
fn run(audio: &mut AudioSystem, mixer: &super::mixer::SharedMixer, seconds: f32) -> Vec<f32> {
    let mut out = Vec::new();
    let ticks = (seconds * 60.0) as usize;
    for _ in 0..ticks {
        audio.update(1.0 / 60.0);
        let mut buf = vec![0.0f32; 800 * 2];
        mixer.lock().unwrap().render(&mut buf);
        out.extend_from_slice(&buf);
    }
    out
}

#[test]
fn gui_event_plays_and_ends() {
    let (mut a, mixer) = AudioSystem::with_default_mixer();
    let g = a.load_group(r"\Audio\GUI\FE_GUI", None).expect("load FE_GUI");
    assert_eq!(a.groups.get(g).unwrap().events.len(), 5);
    let e = a.create_emitter(emitter_flags::ALIVE | emitter_flags::FLAT).unwrap();
    let slot = a.new_slot();
    assert_eq!(a.play_event(e, event_hash("GUI_ACCEPT"), None, 0, Some(slot)), 0);
    let ev = a.slot(slot).expect("event handle filled");
    assert!(a.event_sample(ev).is_some());
    assert_eq!(a.play_event(e, event_hash("GUI_ACCEPT"), None, 0, Some(slot)), err::ALREADY_PLAYING);
    let out = run(&mut a, &mixer, 2.0);
    let first = rms(&out[..48000]);
    let tail = rms(&out[out.len() - 9600..]);
    eprintln!("gui_accept rms {first:.4}, tail {tail:.6}");
    assert!(first > 0.01 && tail == 0.0);
    // the event died with its sample and cleared the caller's handle
    assert!(a.slot(slot).is_none());
    assert_eq!(a.events.alloc_count(), 0);
    assert_eq!(a.samples.alloc_count(), 0);
    assert_eq!(a.unknown_play(e), err::UNKNOWN_EVENT);
}

impl AudioSystem {
    fn unknown_play(&mut self, e: EmitterId) -> i32 {
        self.play_event(e, 0xDEAD_BEEF, None, 0, None)
    }
}

#[test]
fn looping_engine_band_and_pitch() {
    let (mut a, mixer) = AudioSystem::with_default_mixer();
    a.load_group(r"\audio\vehicle\tier4\wrx\vimprrb0", None).expect("load revband group");
    let e = a.create_emitter(emitter_flags::ALIVE | emitter_flags::FLAT).unwrap();
    // first band of the wrx's rbd
    let slot = a.new_slot();
    assert_eq!(a.play_event(e, 0xAA51_9A28, None, 256, Some(slot)), 0);
    let ps = a.slot_sample(slot).unwrap();
    let v = a.samples.get(ps).unwrap().voice.unwrap();
    let mix_slot = a.voices.slot(v).unwrap();
    run(&mut a, &mixer, 0.5);
    let f0 = mixer.lock().unwrap().voices[mix_slot].freq;
    a.set_sample_pitch(ps, 1200.0);
    run(&mut a, &mixer, 0.1);
    let f1 = mixer.lock().unwrap().voices[mix_slot].freq;
    eprintln!("band freq {f0} -> {f1}");
    assert!((f1 / f0 - 2.0).abs() < 0.01);
    // still looping after several seconds
    let out = run(&mut a, &mixer, 4.0);
    assert!(rms(&out[out.len() - 9600..]) > 0.001);
    a.stop_event(a.slot(slot));
    assert!(a.slot(slot).is_none());
    let out = run(&mut a, &mixer, 0.2);
    assert_eq!(rms(&out), 0.0);
}

#[test]
fn positional_emitter_attenuates_and_pans() {
    let (mut a, mixer) = AudioSystem::with_default_mixer();
    a.load_group(r"\audio\vehicle\tier4\wrx\vimprrb0", None).unwrap();
    let l = a.create_listener(emitter_flags::ALIVE).unwrap();
    a.set_listener_position(l, Vec3::ZERO);
    // camera looking along +Z, as VsAudio::update_listener hands it over (x/z flipped)
    a.set_listener_direction(l, Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
    let e = a.create_emitter(emitter_flags::ALIVE).unwrap();
    a.set_emitter_distances(e, 1.0, 100.0);
    let measure = |a: &mut AudioSystem, p: Vec3| {
        a.set_emitter_position(e, p);
        let out = run(a, &mixer, 0.3);
        let tail = &out[out.len() - 4800..];
        let l: Vec<f32> = tail.iter().step_by(2).copied().collect();
        let r: Vec<f32> = tail.iter().skip(1).step_by(2).copied().collect();
        (rms(&l), rms(&r))
    };
    let slot = a.new_slot();
    a.play_event(e, 0xAA51_9A28, None, 0, Some(slot));
    let near = measure(&mut a, Vec3::new(0.0, 0.0, 1.0));
    let far = measure(&mut a, Vec3::new(0.0, 0.0, 10.0));
    let right = measure(&mut a, Vec3::new(5.0, 0.0, 0.0));
    eprintln!("near {near:?} far {far:?} right {right:?}");
    // inverse distance: 10x further = -20 dB
    let ratio = (far.0 + far.1) / (near.0 + near.1);
    assert!((ratio - 0.1).abs() < 0.03, "{ratio}");
    assert!(right.1 > right.0 * 5.0);
}

#[test]
fn volume_table_matches_formula() {
    let (a, _) = AudioSystem::with_default_mixer();
    let t = &a.voices.volume_table;
    assert_eq!(t[0], -10000);
    assert_eq!(t[255], 0);
    assert_eq!(t[128], -598); // ftol(2000 * log10(128 * 0.003921569f))
}

#[test]
fn music_tracks_keep_playing() {
    for (g, n) in [(r"\Audio\Music\ingame\music", "MUSICTRACK"), (r"\Audio\Music\fe\music_fe", "FEMUSICTRACK")] {
        let (mut a, mixer) = AudioSystem::with_default_mixer();
        let gid = a.load_group(g, None).unwrap();
        let names: Vec<String> = a.groups.get(gid).unwrap().events.iter().map(|_| String::new()).collect();
        let e = a.create_emitter(0x4000_0001).unwrap();
        for i in 1..=names.len().min(12) {
            let name = format!("{n}{i:02}");
            let slot = a.new_slot();
            if a.play_event(e, event_hash(&name), None, 0, Some(slot)) != 0 {
                continue;
            }
            run(&mut a, &mixer, 3.0);
            eprintln!("{name}: alive after 3 s {}", a.slot(slot).is_some());
            assert!(a.slot(slot).is_some(), "{name} ended within 3 s");
            a.stop_slot(slot);
        }
    }
}
