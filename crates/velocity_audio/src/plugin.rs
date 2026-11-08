//! the bevy side: one `VisosAudio` resource holding the engine and `g_AudioSystem`, a mix-output asset fed to
//! bevy_audio, and messages for everything the original calls directly on `g_AudioSystem`.
//!
//! gameplay writes `GameAudioInput` each frame and sends `AudioCommand`s; `AudioSystems::Update` runs
//! `VsAudio::Update` once per frame. without bevy's own `AudioPlugin` (headless) everything still runs, the
//! mix is just never pulled.

use bevy::math::Vec3;
use bevy::prelude::*;

use crate::audio::mixer::{MixerStream, SharedMixer};
use crate::audio::system::AudioSystem;
use crate::input::{AudioSettings, GameAudioInput};
use crate::vs_audio::{GuiSound, VsAudio, VsAudioInitInfo};

/// the engine plus the game's audio manager.
#[derive(Resource)]
pub struct VisosAudio {
    pub system: AudioSystem,
    pub game: VsAudio,
    pub mixer: SharedMixer,
}

impl Default for VisosAudio {
    fn default() -> Self {
        let (mut system, mixer) = AudioSystem::with_default_mixer();
        system.randomize_music_per_launch();
        let game = VsAudio::new(&mut system);
        Self { system, game, mixer }
    }
}

/// the engine's stereo mix as a bevy audio source (never finishes).
#[derive(Asset, TypePath, Clone)]
pub struct VisosMixOutput {
    pub mixer: SharedMixer,
}

impl bevy::audio::Decodable for VisosMixOutput {
    type Decoder = MixerStream;
    fn decoder(&self) -> MixerStream {
        MixerStream::new(self.mixer.clone())
    }
}

/// a call the original makes on `g_AudioSystem` from game code.
#[derive(Message, Clone, Debug)]
pub enum AudioCommand {
    /// `VsAudio::Init` (front end or race)
    Init(VsAudioInitInfo),
    LoadFrontEndGroups,
    Shutdown,
    Pause,
    Resume,
    /// the options screen (`+105` dirty)
    Settings(AudioSettings),
    Gui(GuiSound),
    CarOpen,
    CarClose,
    StartNos,
    StopNos,
    StartSlipstream,
    StopSlipstream,
    StartZone,
    StopZone,
    StartTb,
    StopTb,
    StartMode3,
    StopMode3,
    ResetModes,
    EnterCover { wet: bool },
    ExitCover { wet: bool },
    EnterShop { wet: bool },
    ExitShop { wet: bool },
    RemoveShopAlarm,
    TunnelReverb,
    DefaultReverb,
    /// the pre-race countdown, sent every frame until the start
    Countdown,
    RaceStarted,
    StopGameAudio,
    TrafficMuted(bool),
    /// `VsAudio::PlayPhysicsImpact`
    Impact { event: u32, position: Vec3, strength: f32, car_layer_only: bool },
    /// CarDamageFX 0x00078320: glass particles fired for car `position` (vehicle position, not the hit)
    GlassImpact { position: Vec3 },
    /// GameDataLoading 0x000A1190, load step `LevelSounds` ("Load Level Sounds"): the race's groups
    LoadLevelSounds { track_dir: String, wet: bool },
    /// VsAudioRevBands::Load 0x0004C6F0 from PhysicsManager 0x0019E080 (physics vehicle 0 only): `name` = the
    /// vehicle database model/audio block's first string (`VehicleAudio::model`, e.g. `tier4\wrx\vimprrb0`)
    LoadRevBands { name: String },
    /// VsAudioVehics::LoadAiEngine 0x000531D0 from PhysicsManager 0x0019E080 (0x0019F2B0, once per ai race
    /// vehicle, after its physics setup): `path` = vehicle entry +0x40 +4 (`VehicleAudio::audio`), an `.xvg` group
    LoadAiEngine { path: String },
    /// SaveFileData 0x00104F30: a loaded save's playlists (`SavedPlaylists::from_bytes(&options[0x28..])`)
    SetLists(Box<crate::vs_audio_track_list::SavedPlaylists>),
    /// a dynamic positional loop (`VsAudio::AddEnvSound`)
    AddEnvSound { event: u32, position: Vec3, min: f32, volume: f32, max: f32 },
    /// play (soundtrack, song) right away (`PlayTrack(true)`)
    PlayTrack { soundtrack: usize, song: usize },
    NextTrack,
    PauseMusic,
    ResumeMusic,
    /// `VsAudio::StopMusicEvent` 0x0004D950 (GameManager__BE9F0 0x000BE9F0, before a full-screen movie)
    StopMusicEvent,
    /// `VsAudio::StartLoopEvent210` 0x0004A8D0 (hangout ambience loop)
    StartLoopEvent210,
    /// `VsAudio::StopLoopEvent210` 0x0004A920
    StopLoopEvent210,
    /// `VsAudio::FadeMusic(to, env, time)` 0x0004DA80 (hangout music fade + reverb)
    FadeMusic { to: f32, env: f32, time: f32 },
    /// timeline `F3EventSound` / `F3Event2DSound` (TimelineSound.cpp 0x0017EA00): `object` = the timeline
    /// object's name crc (one 2D emitter each), `event` = the AudioEvent crc
    TimelineSound { object: u32, event: u32 },
    /// timeline `F3EventSoundStop`: the named event (0x0017EBD0) or everything on the object (0x0017EB80)
    TimelineSoundStop { object: u32, event: Option<u32> },
    /// timeline AUDIO_FadeToVolumePreset(preset, item length) (`velocity.vol` preset 0..31)
    FadeToVolumePreset { preset: i32, time: f32 },
    /// timeline `F3EventAudioToggleEnvironment`: VsAudio::StartCityAmbience 0x0004AD60 / StopCityAmbience 0x0004AE10
    CityAmbience(bool),
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AudioSystems {
    Commands,
    Update,
}

/// whether `VsAudio::Update` runs this frame: the original calls it only from CFrontendManager::Update
/// 0x002016DD and GameManager::InGameUpdate 0x000B7EE7 (+ Pause / Resume), never during the race load steps.
/// Default on; the game clears it while it loads a race.
#[derive(Resource)]
pub struct AudioUpdateGate(pub bool);

impl Default for AudioUpdateGate {
    fn default() -> Self {
        Self(true)
    }
}

pub fn build(app: &mut App) {
    app.init_resource::<VisosAudio>()
        .init_resource::<AudioUpdateGate>()
        .init_resource::<GameAudioInput>()
        .init_resource::<crate::MovieAudioOut>()
        .init_resource::<crate::audio_tweaks::AudioTweaks>()
        .add_systems(First, crate::audio_tweaks::sync_audio_tweaks)
        .add_message::<AudioCommand>()
        .configure_sets(Update, (AudioSystems::Commands, AudioSystems::Update).chain())
        .add_systems(
            Update,
            (
                apply_commands.in_set(AudioSystems::Commands),
                (update, crate::audio::xbox::bink_sound::movie_sound).chain().in_set(AudioSystems::Update),
            ),
        );
    // bevy's AudioPlugin is always disabled (main.rs); under SMOKE_RECORD the recorder pulls the mix itself
    // (headless MinimalPlugins apps, i.e. tests, have no WindowPlugin and open no device)
    if std::env::var_os("SMOKE_RECORD").is_none() && app.is_plugin_added::<bevy::window::WindowPlugin>() {
        if null_output() {
            let mixer = app.world().resource::<VisosAudio>().mixer.clone();
            start_null_sink(mixer);
        } else {
            // own the device stream (bevy_audio's stays idle): reopens on stream errors / default device changes
            let mixer = app.world().resource::<VisosAudio>().mixer.clone();
            crate::audio::output::spawn(mixer);
        }
    }
}

/// port tooling: `VELOCITY_AUDIO_OUT=null` (set by `tools/hidden_run.py`) mixes as normal but sends nothing to
/// the sound device
pub fn null_output() -> bool {
    std::env::var("VELOCITY_AUDIO_OUT").is_ok_and(|v| v.eq_ignore_ascii_case("null"))
}

/// pull the mix at real-time rate and drop it, so voices advance exactly as with a device
fn start_null_sink(mixer: crate::audio::mixer::SharedMixer) {
    log::info!("audio: null output (VELOCITY_AUDIO_OUT=null)");
    let _ = std::thread::Builder::new().name("visos null audio".into()).spawn(move || {
        let t0 = std::time::Instant::now();
        let mut rendered = 0u64;
        let mut buf = Vec::new();
        loop {
            let due = (t0.elapsed().as_secs_f64() * crate::audio::mixer::MIX_RATE as f64) as u64;
            if due > rendered {
                buf.resize((due - rendered) as usize * 2, 0.0);
                if let Ok(mut m) = mixer.lock() {
                    m.render(&mut buf);
                }
                rendered = due;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    });
}

fn apply_commands(mut audio: ResMut<VisosAudio>, input: Res<GameAudioInput>, mut msgs: MessageReader<AudioCommand>) {
    let VisosAudio { system: a, game: g, .. } = &mut *audio;
    for m in msgs.read() {
        if let Some(t) = &mut a.trace_cmds {
            let s = format!("{m:?}");
            let mut n = s[..s.find([' ', '(', '{']).unwrap_or(s.len())].to_string();
            // impacts: event, distance to the player car (plays under 5 m), strength, car-layer flag
            if let AudioCommand::Impact { event, position, strength, car_layer_only } = m {
                let pp = input.player.as_ref().map(|p| p.position).unwrap_or(Vec3::ZERO);
                n = format!("Impact[{event:08x} d{:.0} s{:.0}{}]", pp.distance(*position), strength, if *car_layer_only { " c" } else { "" });
            }
            t.push(n);
        }
        match m.clone() {
            AudioCommand::Init(info) => {
                log::info!("audio: init frontend {} track {:?}", info.frontend, info.track_dir);
                g.init(a, &info);
            }
            AudioCommand::LoadFrontEndGroups => {
                g.load_front_end_groups(a);
            }
            AudioCommand::Shutdown => {
                log::info!("audio: shutdown");
                g.shutdown(a);
            }
            AudioCommand::Pause => g.pause(a, &input),
            AudioCommand::Resume => g.resume(a, &input),
            AudioCommand::Settings(s) => g.apply_settings(s),
            AudioCommand::Gui(s) => {
                g.play_gui(a, s);
            }
            AudioCommand::CarOpen => g.play_car_open(a),
            AudioCommand::CarClose => g.play_car_close(a),
            AudioCommand::StartNos => g.start_nos(a),
            AudioCommand::StopNos => g.stop_nos(a),
            AudioCommand::StartSlipstream => g.start_slipstream(a),
            AudioCommand::StopSlipstream => g.stop_slipstream(a),
            AudioCommand::StartZone => g.start_zone(a),
            AudioCommand::StopZone => g.stop_zone(a),
            AudioCommand::StartTb => g.start_tb(a),
            AudioCommand::StopTb => g.stop_tb(a),
            AudioCommand::StartMode3 => g.start_mode3(a),
            AudioCommand::StopMode3 => g.stop_mode3(a),
            AudioCommand::ResetModes => g.reset_modes(a),
            AudioCommand::EnterCover { wet } => g.enter_cover(a, wet),
            AudioCommand::ExitCover { wet } => g.exit_cover(a, wet),
            AudioCommand::EnterShop { wet } => g.enter_shop(a, wet),
            AudioCommand::ExitShop { wet } => g.exit_shop(a, wet),
            AudioCommand::RemoveShopAlarm => g.remove_shop_alarm(a),
            AudioCommand::TunnelReverb => g.set_tunnel_reverb(a),
            AudioCommand::DefaultReverb => g.set_default_reverb(a),
            // MonitorSessions 0x000BD263 pushes a constant 3.0, not the frame time: "ready" plays on the first call
            AudioCommand::Countdown => g.update_countdown(a, 3.0, &input),
            AudioCommand::RaceStarted => g.race_started(a),
            AudioCommand::StopGameAudio => g.stop_game_audio(a),
            AudioCommand::TrafficMuted(m) => g.set_traffic_muted(a, m),
            AudioCommand::Impact { event, position, strength, car_layer_only } => {
                g.play_physics_impact(a, &input, event, position, strength, car_layer_only)
            }
            AudioCommand::GlassImpact { position } => g.play_glass_impact(a, &input, position),
            AudioCommand::SetLists(s) => g.set_lists(a, &s),
            AudioCommand::LoadLevelSounds { track_dir, wet } => g.load_level_sounds(a, &track_dir, wet),
            AudioCommand::LoadRevBands { name } => {
                log::info!("audio: rev bands {name}");
                g.events.rev_bands.load(a, &name);
            }
            AudioCommand::LoadAiEngine { path } => {
                log::info!("audio: ai engine {path}");
                crate::vs_audio_vehics::load_ai_engine(a, &path);
            }
            AudioCommand::AddEnvSound { event, position, min, volume, max } => {
                g.add_env_sound(event, position, min, volume, max, 0.0)
            }
            AudioCommand::PlayTrack { soundtrack, song } => g.play_track(a, true, Some((soundtrack, song))),
            AudioCommand::NextTrack => g.next_track(a),
            AudioCommand::PauseMusic => g.pause_music(a),
            AudioCommand::ResumeMusic => g.resume_music(a),
            AudioCommand::StopMusicEvent => g.stop_music_event(a),
            AudioCommand::StartLoopEvent210 => {
                log::info!("audio: StartLoopEvent210");
                g.start_loop_event_210(a)
            }
            AudioCommand::StopLoopEvent210 => {
                log::info!("audio: StopLoopEvent210");
                g.stop_loop_event_210(a)
            }
            AudioCommand::FadeMusic { to, env, time } => {
                log::info!("audio: FadeMusic to {to} env {env} time {time}");
                g.fade_music(a, to, env, time)
            }
            AudioCommand::TimelineSound { object, event } => {
                g.play_timeline_sound(a, object, event);
            }
            AudioCommand::TimelineSoundStop { object, event } => g.stop_timeline_sound(a, object, event),
            AudioCommand::FadeToVolumePreset { preset, time } => {
                // the original indexes the 32 presets unchecked
                if (0..32).contains(&preset) {
                    a.fade_to_volume_preset(preset as usize, time);
                }
            }
            AudioCommand::CityAmbience(on) => {
                if on {
                    g.start_city_ambience(a)
                } else {
                    g.stop_city_ambience(a)
                }
            }
        }
    }
}

/// `VsAudio::Update` once per frame.
fn update(mut audio: ResMut<VisosAudio>, input: Res<GameAudioInput>, time: Res<Time>, gate: Res<AudioUpdateGate>) {
    if !gate.0 {
        return;
    }
    let VisosAudio { system: a, game: g, .. } = &mut *audio;
    g.update(a, time.delta_secs(), &input);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// headless app: front-end init and a gui sound through messages, the engine ticking in Update.
    #[test]
    fn headless_app_front_end() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, crate::AudioPlugin));
        app.world_mut().write_message(AudioCommand::Init(VsAudioInitInfo { frontend: true, ..Default::default() }));
        app.world_mut().write_message(AudioCommand::LoadFrontEndGroups);
        app.update();
        app.world_mut().write_message(AudioCommand::Gui(GuiSound::Accept));
        app.update();
        let a = app.world().resource::<VisosAudio>();
        assert!(a.game.flags & crate::vs_audio::flags::INITIALISED != 0);
        let mut buf = vec![0.0f32; 4800];
        a.mixer.lock().unwrap().render(&mut buf);
        assert!(buf.iter().any(|v| *v != 0.0), "gui sound mixed");
    }
}
