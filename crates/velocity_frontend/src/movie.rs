//! bevy glue for the one `VsMovieStream` (GameManager +0x5F8 `unk_6F5BC0`):
//! - full-screen movies (GameManager__BE9F0 0x000BE9F0: stop the music, queue the movie, render mode 2, play, push
//!   the movie game mode until it ends), drawn as a full-screen image on their own camera (order 12, over the front
//!   end and the HUD);
//! - the in-race TFT movies (GameManager__BEBB0 0x000BEBB0: render mode 3 into the 64x64 "movie_test" texture,
//!   looping, speech volume, no music stop), driven every render frame by OSD3dInterface's TFT part (0x000EA160).
//!
//! Both play on the real clock at the file's frame rate.
//! - in: `MovieCommand::Play(names)` (queued, played in order), `MovieCommand::PlayTexture`, `MovieCommand::Stop`;
//!   the ai's TFT lines through `velocity_ai::AiTftPlayer` (inserted here, see [`TftLink`]).
//! - out: `MovieFinished` when a full-screen queue is done or skipped; `TftMovieTexture` (the TFT picture, drawn on
//!   the 3D hud's `tft_screen` by `osd::osd_3d_interface`); `MovieAudioOut` holds the decoded PCM of the first
//!   audio track (interleaved s16 at `rate` / `channels`), drained every frame into velocity_audio's Bink sound
//!   buffer (`open` at BinkOpen, `close` at BinkClose; capped at 2 s if nothing drains it).
//! - skip: the movie game mode's Update (0x000C01A0, [`movie_skip`]): MenuSelect or Pause on any device.

use crate::bink::BinkTables;
use crate::vs_movie::vs_movie_stream_xbox::VsMovieStream;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::sync::{Arc, Mutex};

#[derive(Message, Clone, Debug)]
pub enum MovieCommand {
    Play(Vec<String>),
    /// GameManager__BEBB0 (its play branch, LABEL_17): `name` into the TFT texture (mode 3)
    PlayTexture { name: String, looping: bool },
    Stop,
}

#[derive(Message, Clone, Debug)]
pub struct MovieFinished;

/// decoded movie sound waiting for the mixer (owned by velocity_audio: RAD's xbox dsound driver, bink_sound.rs)
pub use velocity_audio::MovieAudioOut;

#[derive(Resource, Default)]
pub struct MoviePlayer {
    pub stream: VsMovieStream,
    tables: Option<Arc<BinkTables>>,
    image: Option<Handle<Image>>,
    size: (u32, u32),
    ents: Vec<Entity>,
    rgba: Vec<u8>,
    pcm: Vec<i16>,
    /// a full-screen movie is up (the front end does not update meanwhile)
    pub playing: bool,
    /// GameManager `unk_6F5BC0` holds a texture movie (set by 0x000BEBB0 even when BinkOpen then fails, cleared by
    /// 0x000BEFD0): what 0x000BF030 tests for the TFT
    pub texture_active: bool,
}

/// the TFT picture: OSD3dInterface +2860, `CTextureManager::Register("movie_test", 64, 64, ...)` (0x000E9C90), shown
/// by 0x000EAD80 on the 3D hud with the "LCD_Movie" shader while `visible`
#[derive(Resource, Default)]
pub struct TftMovieTexture {
    pub image: Option<Handle<Image>>,
    pub size: (u32, u32),
    /// 0x000EAD80's draw test: a line is wanted and has run 2 render frames (its +2900 flip test is the 3D hud's)
    pub visible: bool,
    /// +2896: a line is wanted (the 3D hud flips the TFT open while set, 0x000EA2B0)
    pub wanted: bool,
}

/// OSD3dInterface's TFT fields (`byte_6FC098`; idb names `VsAIPlayerTFTManager__17220..172D0`)
#[derive(Debug, Default)]
pub struct Osd3dTft {
    pub name: String,         // +2864
    pub wanted: bool,         // +2896
    pub frames: u32,          // +2904 render frames since the movie started
    pub vcd: Option<String>,  // +2908
}

/// shared between the ai's `AiTftPlayer` (VsAIPlayerTFTManager 0x00016940 calls) and [`run_tft`]
#[derive(Resource, Clone, Default)]
pub struct TftLink(pub Arc<Mutex<Osd3dTft>>);

struct TftOut(Arc<Mutex<Osd3dTft>>);

impl velocity_ai::ai::vs_ai_player_tft_manager::TftPlayer for TftOut {
    /// 0x000172D0
    fn is_playing(&self) -> bool {
        self.0.lock().unwrap().wanted
    }

    /// 0x00017270 name, 0x00017220(1), 0x000172B0 vcd
    fn play(&mut self, name: &str, vcd: Option<&str>) {
        let mut t = self.0.lock().unwrap();
        t.name = name.to_string();
        if !t.wanted {
            t.frames = 0;
        }
        t.wanted = true;
        t.vcd = vcd.map(str::to_string);
    }
}

#[derive(Component)]
struct MovieView;

pub struct MoviePlugin;

impl Plugin for MoviePlugin {
    fn build(&self, app: &mut App) {
        let link = TftLink::default();
        app.add_message::<MovieCommand>()
            .add_message::<MovieFinished>()
            .init_resource::<MoviePlayer>()
            .init_resource::<MovieAudioOut>()
            .init_resource::<TftMovieTexture>()
            .insert_resource(velocity_ai::plugin::AiTftPlayer(Box::new(TftOut(link.0.clone()))))
            .insert_resource(link)
            .add_systems(Update, (run_tft, run_movie).chain())
            .add_systems(
                velocity_game::vs_common_main::GameTick,
                movie_skip.after(velocity_game::vs_common_main::InGameUpdateSet::Input),
            );
    }
}

/// GameManager.cpp 0x000C01A0 / VsMovieStreamXBOX.cpp 0x002F4B90 — the movie game mode's Update (vtable 0x00631964 +4, made by GameManager__BE9F0
/// through 0x000C0020), once per game update: UserInput::Update, then MenuSelect (6) or Pause (13) pressed on any
/// device (`0x001E6BC0 != 0 && 0x001E6DF0`, mask 0xFFFF) skips the movie (0x002F4B90: state 2, closed by the next
/// stream update; the queue goes on with the next movie). Core updates UserInput at the start of every GameTick.
fn movie_skip(ui: Option<Res<velocity_game::controller::user_input::UserInput>>, mut player: ResMut<MoviePlayer>) {
    use velocity_game::controller::user_input::EAction;
    let Some(ui) = ui else { return };
    if player.playing && (ui.pressed(EAction::MenuSelect, 0xFFFF) || ui.pressed(EAction::Pause, 0xFFFF)) {
        player.stream.close();
    }
}

/// OSD3dInterface.cpp 0x000EA160 (idb `COSDMapRender__EA160`, from GameManager's render 0x000B9D46) — per render
/// frame in race: start the wanted line, stop it once it reached its last frame after 10 frames, stop a movie
/// nobody wants. The movie stream is the one full-screen movies use (GameManager 0x000BF030 = any movie open).
fn run_tft(
    link: Res<TftLink>,
    hud: Option<Res<crate::hud::HudState>>,
    player: Res<MoviePlayer>,
    mut cmds: MessageWriter<MovieCommand>,
    mut tex: ResMut<TftMovieTexture>,
) {
    let mut t = link.0.lock().unwrap();
    // the OSD3dInterface only renders in race
    if !hud.is_some_and(|h| h.active) {
        // GameManager 0x000B8550 / 0x000BC010 stop the TFT movie (0x000BEFD0) when the race goes
        if player.texture_active {
            cmds.write(MovieCommand::Stop);
            t.wanted = false;
            t.frames = 0;
        }
        tex.visible = false;
        tex.wanted = false;
        return;
    }
    let running = player.texture_active || player.stream.bink.is_some() || !player.stream.queue.is_empty();
    if t.wanted {
        if running {
            let (frame, frames) = player.stream.frame_counts();
            if t.frames > 10 && frame >= frames {
                cmds.write(MovieCommand::Stop); // GameManager 0x000BEFD0
                t.wanted = false;
                t.frames = 0;
            }
            t.frames += 1;
        } else {
            // GameManager__BEBB0(+2860, +2864, +2908, loop 1, 0): the per-player vcd's FindFile + async preload
            // (0x002F5E60) is IO; it ends in the same play call
            if t.vcd.is_some() {
                log::debug!("tft: {} from a player vcd (read as a loose file)", t.name);
            }
            cmds.write(MovieCommand::PlayTexture { name: t.name.clone(), looping: true });
            t.frames = 0;
        }
    } else if running {
        cmds.write(MovieCommand::Stop);
        t.frames = 0;
    }
    // 0x000EAD80
    tex.visible = t.wanted && t.frames >= 2;
    tex.wanted = t.wanted;
}

#[allow(clippy::too_many_arguments)]
fn run_movie(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut player: ResMut<MoviePlayer>,
    mut cmds: MessageReader<MovieCommand>,
    mut images: ResMut<Assets<Image>>,
    mut done: MessageWriter<MovieFinished>,
    mut aout: ResMut<MovieAudioOut>,
    mut audio: Option<ResMut<Messages<velocity_audio::AudioCommand>>>,
    options: Option<Res<crate::game_options::GameOptions>>,
    mut tex: ResMut<TftMovieTexture>,
) {
    let p = &mut *player;
    for c in cmds.read() {
        match c {
            MovieCommand::Play(names) => {
                // GameManager__BE9F0: VsAudio::StopMusicEvent, then GameManager__BFFF0(1.0) (movie volume)
                if let Some(a) = audio.as_mut() {
                    a.write(velocity_audio::AudioCommand::StopMusicEvent);
                }
                aout.volume = 1.0 * 32768.0;
                for n in names {
                    p.stream.que_source(n);
                }
                p.stream.set_mode(2);
                p.stream.looping = false; // 0x002F4AD0(2, 0)
            }
            MovieCommand::PlayTexture { name, looping } => {
                // GameManager__BEBB0: 0x000BEFD0 stop, BFFF0(flt_6F9174 = speech volume), queue, mode 3, texture,
                // 0x002F4AD0(2, loop)
                if p.stream.bink.is_some() {
                    p.stream.queue.clear();
                    p.stream.close();
                    aout.close();
                }
                aout.volume = options.as_ref().map_or(1.0, |o| o.volumes[4]) * 32768.0;
                p.stream.que_source(name);
                p.stream.set_mode(3);
                p.stream.looping = *looping;
                p.texture_active = true;
            }
            MovieCommand::Stop => {
                p.stream.queue.clear();
                p.stream.close();
                p.texture_active = false;
            }
        }
    }
    let texture = p.stream.mode == 3;
    if p.stream.bink.is_none() && p.stream.queue.is_empty() {
        if p.playing || (texture && aout.open) {
            finish(&mut commands, p, &mut aout, &mut done, texture);
        }
        return;
    }
    if p.tables.is_none() {
        match BinkTables::load() {
            Some(t) => p.tables = Some(Arc::new(t)),
            None => {
                log::warn!("movie: no Bink tables (default.xbe not readable), movies skipped");
                p.stream.queue.clear();
                return;
            }
        }
    }
    let t = p.tables.clone().unwrap();
    let first = p.stream.bink.is_none();
    if !p.stream.start_next(&t) {
        finish(&mut commands, p, &mut aout, &mut done, texture);
        return;
    }
    let (w, h, rate, ch) = {
        let m = p.stream.bink.as_ref().unwrap();
        let tr = m.file.tracks.first();
        (m.file.width, m.file.height, tr.map(|t| t.rate).unwrap_or(0), tr.map(|t| t.channels()).unwrap_or(0))
    };
    if first {
        log::info!("movie: {} {}x{} mode {}", p.stream.bink.as_ref().unwrap().name, w, h, p.stream.mode);
        aout.open(rate, ch);
    }
    let new_image = |images: &mut Assets<Image>| {
        images.add(Image::new_fill(
            Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        ))
    };
    let target = if texture {
        // the TFT texture: no full-screen view, the front end keeps running
        if tex.image.is_none() || tex.size != (w, h) {
            tex.image = Some(new_image(&mut images));
            tex.size = (w, h);
        }
        tex.image.clone()
    } else {
        if p.image.is_none() || p.size != (w, h) {
            let handle = new_image(&mut images);
            p.size = (w, h);
            p.image = Some(handle.clone());
            for e in p.ents.drain(..) {
                commands.entity(e).despawn();
            }
            let cam = commands
                .spawn((Camera2d, Camera { order: 12, clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() }, MovieView))
                .id();
            let root = commands
                .spawn((
                    Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                    ImageNode::new(handle),
                    UiTargetCamera(cam),
                    MovieView,
                ))
                .id();
            p.ents = vec![cam, root];
        }
        p.playing = true;
        p.image.clone()
    };
    let (mut rgba, mut pcm) = (std::mem::take(&mut p.rgba), std::mem::take(&mut p.pcm));
    pcm.clear();
    let (_, changed) = p.stream.update(&t, time.delta_secs_f64(), &mut rgba, &mut pcm);
    if changed {
        if let Some(mut img) = target.as_ref().and_then(|h| images.get_mut(h)) {
            img.data = Some(rgba.clone());
        }
    }
    aout.pcm.extend(pcm.iter().copied());
    let cap = (aout.rate * aout.channels.max(1) * 2) as usize;
    while aout.pcm.len() > cap.max(1) {
        aout.pcm.pop_front();
    }
    p.rgba = rgba;
    p.pcm = pcm;
}

fn finish(commands: &mut Commands, p: &mut MoviePlayer, aout: &mut MovieAudioOut, done: &mut MessageWriter<MovieFinished>, texture: bool) {
    aout.close();
    if texture {
        return;
    }
    for e in p.ents.drain(..) {
        commands.entity(e).despawn();
    }
    p.image = None;
    p.playing = false;
    done.write(MovieFinished);
}
