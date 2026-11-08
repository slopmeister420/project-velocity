//! Frontend menus, OSD/HUD, fonts and strings (src/Feck2, src/Osd, src/OSDMap, Fonts, StringManager).
//!
//! bevy side: [`FrontendPlugin`] runs `CFrontendManager::Update`/`Render` once a frame while the frontend is
//! active, rasterises the 2D draw list into a 640x480 image shown full-window, and exchanges messages with game
//! flow: [`FrontendCommand`] in, [`FrontendRequestMsg`] out.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub mod bink;
pub mod fe_tweaks;
pub mod feck2;
pub mod font_helper;
pub mod fonts;
pub mod foyer;
pub mod frontend;
pub mod game_menu_manager;
pub mod game_options;
pub mod hud;
pub mod loading_bar;
pub mod movie;
pub mod osd;
pub mod pc_settings;
pub mod plate_font;
pub mod render2d;
pub mod string_manager;
pub mod user_input;
pub mod vs_movie;
pub mod vtf;

#[cfg(test)]
mod capture_tests;

pub use feck2::feck2_manager::FrontendRequest;

/// commands from game flow (GameManager) to the frontend.
#[derive(Message, Debug, Clone)]
pub enum FrontendCommand {
    /// `CFrontendManager::Start(crc("<screen>"))` and show the frontend.
    Start(String),
    /// leave the frontend (race starting): keeps the loading screen up until game flow is in game, then stops
    /// updating and hides it.
    Stop,
    /// game flow is back in the front end after a race (`FrontEndReInit`): start on the post race screen.
    ReturnFromRace,
}

/// a request the frontend raised (see `FrontendRequest`).
#[derive(Message, Debug, Clone)]
pub struct FrontendRequestMsg(pub FrontendRequest);

/// display size of the frontend image.
pub const DISPLAY_W: u32 = 640;
pub const DISPLAY_H: u32 = 480;

#[derive(Resource)]
pub struct FrontendState {
    pub frontend: Option<frontend::Frontend>,
    pub active: bool,
    pub image: Handle<Image>,
    pub mapping: user_input::InputMapping,
    pub pads: [[feck2::feck2_manager::ActionState; feck2::feck2_manager::action::COUNT]; 2],
    pub root: Option<Entity>,
    /// `Stop` received; hidden once the game mode is in game
    pub stop_pending: bool,
}

/// the frontend's 2D camera and image node
#[derive(Component)]
pub struct FrontendView;

pub struct FrontendPlugin;

impl Plugin for FrontendPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FrontendCommand>()
            .add_message::<FrontendRequestMsg>()
            .insert_resource(game_options::GameOptions::load())
            .add_systems(Startup, setup_frontend)
            .init_resource::<foyer::FoyerView>()
            .add_systems(
                Update,
                (watch_game_mode, handle_commands, career_in, run_frontend, career_out, plate_sync, foyer::run_foyer, foyer::foyer_car_poses, foyer::foyer_car_parts,foyer::foyer_material_textures, foyer::foyer_mtv, apply_options).chain(),
            )
            .init_resource::<fe_tweaks::FrontendTweaks>()
            .add_systems(Update, fe_tweaks::fe_tweaks.before(run_frontend))
            .add_systems(PostUpdate, foyer::foyer_debug)
            // port: the PC settings page (window mode / resolution / present mode / frame cap / music order)
            .insert_resource(pc_settings::FrameRateCap::from_args())
            .add_systems(Update, pc_settings::pc_display_info.before(run_frontend))
            .add_systems(Update, pc_settings::apply_pc_settings.after(apply_options))
            .add_plugins(hud::HudPlugin)
            .add_plugins(movie::MoviePlugin)
            .add_systems(Startup, boot_movie)
            .add_systems(Update, boot_movie_exit);
    }
}

/// dev: `VELOCITY_MOVIE=In003[,In001..]` plays those movies at boot; `VELOCITY_MOVIE_EXIT=<s>` quits after them or
/// after `s` real seconds
fn boot_movie(mut cmds: MessageWriter<movie::MovieCommand>) {
    if let Ok(v) = std::env::var("VELOCITY_MOVIE") {
        cmds.write(movie::MovieCommand::Play(v.split(',').map(|s| s.trim().to_string()).collect()));
    }
}

fn boot_movie_exit(time: Res<Time<Real>>, mut done: MessageReader<movie::MovieFinished>, mut exit: MessageWriter<AppExit>) {
    let Some(secs) = std::env::var("VELOCITY_MOVIE_EXIT").ok().and_then(|v| v.parse::<f32>().ok()) else { return };
    if done.read().count() > 0 || (secs > 0.0 && time.elapsed_secs() > secs) {
        exit.write(AppExit::Success);
    }
}

/// whether the app was asked to boot into the frontend (not the graphics track viewer).
fn boot_into_frontend() -> bool {
    std::env::var_os("VELOCITY_NO_FRONTEND").is_none() && !std::env::args().any(|a| a == "--track")
}

fn setup_frontend(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut cmds: MessageWriter<FrontendCommand>) {
    let img = Image::new_fill(
        Extent3d { width: DISPLAY_W, height: DISPLAY_H, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let image = images.add(img);
    // licence plate text (graphics' LicencePlateRender 0x0012DF40 draws with the FE / game font sets)
    if visos::file::game_dir().exists() {
        commands.insert_resource(plate_font::PlateFonts::hook());
    }
    commands.insert_resource(FrontendState {
        frontend: None,
        active: false,
        image,
        mapping: user_input::InputMapping::load_default(),
        pads: Default::default(),
        root: None,
        stop_pending: false,
    });
    if boot_into_frontend() {
        // GameManager boots the frontend at the intro movie screen; `skipInitialScreens` / `jumptoscreen`
        // (byte_6F905B) make `Start(0)` (FECK2Manager 0x001FEAF0) open the main menu instead
        let screen = if command_line().skip_initial_screens { "MainMenuScreen" } else { "MovieScreen" };
        cmds.write(FrontendCommand::Start(screen.into()));
    }
}

/// main.cpp 0x0016D120 — the debug command line (`jumptoscreen <Screen>`, `skipInitialScreens`, ...)
fn command_line() -> velocity_game::main_cpp::CommandLine {
    let args: Vec<String> = std::env::args().skip(1).collect();
    velocity_game::main_cpp::CommandLine::parse(&args)
}

fn handle_commands(
    mut commands: Commands,
    mut state: ResMut<FrontendState>,
    options: Res<game_options::GameOptions>,
    mut msgs: MessageReader<FrontendCommand>,
) {
    for m in msgs.read() {
        match m {
            FrontendCommand::Start(screen) => {
                if state.frontend.is_none() {
                    let mut f = frontend::Frontend::new(options.clone());
                    // `jumptoscreen`: dword_6F6FC4, the main menu goes there on its first update
                    let cl = command_line();
                    f.fe.next_screen_crc = cl.jump_to_screen.unwrap_or(0);
                    f.fe.jump_to_car_customisation = cl.jump_to_car_customisation;
                    state.frontend = Some(f);
                    state.mapping = user_input::InputMapping::load_config(options.controller_config[0]);
                }
                if let Some(f) = state.frontend.as_mut() {
                    f.fe.start(string_manager::string_manager::string_crc(screen));
                }
                state.active = true;
                if state.root.is_none() {
                    // drawn over the foyer camera (order 0, clears black): no clear here
                    let cam = commands
                        .spawn((Camera2d, Camera { order: 10, clear_color: ClearColorConfig::None, ..default() }, FrontendView))
                        .id();
                    let image = state.image.clone();
                    let root = commands
                        .spawn((
                            Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                            ImageNode::new(image),
                            UiTargetCamera(cam),
                            FrontendView,
                        ))
                        .id();
                    state.root = Some(root);
                }
            }
            FrontendCommand::Stop => {
                state.stop_pending = true;
            }
            FrontendCommand::ReturnFromRace => {
                state.stop_pending = false;
                if let Some(f) = state.frontend.as_mut() {
                    // FrontEndReInit 0x000B5A80: CreateAllScreens again, then Start(m_dReturnFrontEndScreen)
                    f.reinit();
                    f.fe.return_from_race();
                    state.active = true;
                }
            }
        }
    }
}

/// the front end around a race: hidden once the race is in game (the loading screen stays up while it loads), and
/// back on the post race screen when game flow returns to the front end mode (GameManager 0x000BE730 ->
/// `FrontEndReInit` 0x000B5A80). Without game flow (captures) `Stop` hides at once.
fn watch_game_mode(
    mode: Option<Res<State<velocity_game::EGameMode>>>,
    mut state: ResMut<FrontendState>,
    mut cmds: MessageWriter<FrontendCommand>,
    mut was_racing: Local<bool>,
) {
    use velocity_game::EGameMode;
    let Some(mode) = mode else {
        if state.stop_pending {
            state.stop_pending = false;
            state.active = false;
        }
        return;
    };
    match *mode.get() {
        EGameMode::InGame | EGameMode::PostRace | EGameMode::EndOfGame | EGameMode::ReturnToFrontEnd => {
            *was_racing = true;
            if state.stop_pending {
                state.stop_pending = false;
                state.active = false;
            }
        }
        EGameMode::FrontEnd if *was_racing => {
            *was_racing = false;
            cmds.write(FrontendCommand::ReturnFromRace);
        }
        _ => {}
    }
}

/// raw xbox pad value from the first bevy gamepad and the keyboard stand-in.
pub(crate) fn raw_value(p: user_input::PadInput, keys: &ButtonInput<KeyCode>, pad: Option<&Gamepad>) -> f32 {
    use user_input::PadInput as P;
    let key = |k: KeyCode| if keys.pressed(k) { 1.0 } else { 0.0 };
    let btn = |b: GamepadButton| pad.map(|g| if g.pressed(b) { 1.0 } else { 0.0 }).unwrap_or(0.0);
    let axis = |a: GamepadAxis| pad.and_then(|g| g.get(a)).unwrap_or(0.0);
    let maxabs = |a: f32, b: f32| if b.abs() > a.abs() { b } else { a };
    match p {
        P::A => maxabs(btn(GamepadButton::South), maxabs(key(KeyCode::Enter), key(KeyCode::NumpadEnter))),
        P::B => maxabs(btn(GamepadButton::East), maxabs(key(KeyCode::Escape), key(KeyCode::Backspace))),
        P::X => maxabs(btn(GamepadButton::West), key(KeyCode::KeyX)),
        P::Y => maxabs(btn(GamepadButton::North), key(KeyCode::KeyY)),
        P::White => maxabs(btn(GamepadButton::LeftTrigger), key(KeyCode::KeyQ)),
        P::Black => maxabs(btn(GamepadButton::RightTrigger), key(KeyCode::KeyE)),
        P::Start => maxabs(btn(GamepadButton::Start), key(KeyCode::Space)),
        P::Back => maxabs(btn(GamepadButton::Select), key(KeyCode::Tab)),
        P::Up => maxabs(btn(GamepadButton::DPadUp), key(KeyCode::ArrowUp)),
        P::Down => maxabs(btn(GamepadButton::DPadDown), key(KeyCode::ArrowDown)),
        P::Left => maxabs(btn(GamepadButton::DPadLeft), key(KeyCode::ArrowLeft)),
        P::Right => maxabs(btn(GamepadButton::DPadRight), key(KeyCode::ArrowRight)),
        P::LTrigger => maxabs(axis(GamepadAxis::LeftZ), maxabs(btn(GamepadButton::LeftTrigger2), key(KeyCode::PageUp))),
        P::RTrigger => maxabs(axis(GamepadAxis::RightZ), maxabs(btn(GamepadButton::RightTrigger2), key(KeyCode::PageDown))),
        P::StickLButton => btn(GamepadButton::LeftThumb),
        P::StickRButton => btn(GamepadButton::RightThumb),
        P::StickLX => axis(GamepadAxis::LeftStickX),
        P::StickLY => axis(GamepadAxis::LeftStickY),
        P::StickRX => axis(GamepadAxis::RightStickX),
        P::StickRY => axis(GamepadAxis::RightStickY),
    }
}

fn run_frontend(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    pads: Query<&Gamepad>,
    mut state: ResMut<FrontendState>,
    mut images: ResMut<Assets<Image>>,
    mut out: MessageWriter<FrontendRequestMsg>,
    mut views: Query<(&mut Visibility, Option<&mut Camera>), With<FrontendView>>,
    mut options: ResMut<game_options::GameOptions>,
    audio: Option<ResMut<Messages<velocity_audio::AudioCommand>>>,
    mut attract: Option<ResMut<velocity_game::font_helper::CAttractMode>>,
    mut gm: Option<ResMut<velocity_game::GameManager>>,
    mut clock: Local<f64>,
    mut flow: Option<ResMut<velocity_game::save_game::save_vars::SaveFlow>>,
    (player, mut movies, mut visos_audio): (Res<movie::MoviePlayer>, MessageWriter<movie::MovieCommand>, Option<ResMut<velocity_audio::VisosAudio>>),
) {
    // a full-screen movie is a game mode of its own (GameManager__BE9F0): the front end waits
    if player.playing {
        return;
    }
    // GameManager +0x190's timer (the attract start's srand seed): time spent in the front end
    *clock += time.delta_secs_f64();
    let active = state.active;
    for (mut v, cam) in views.iter_mut() {
        *v = if active { Visibility::Inherited } else { Visibility::Hidden };
        // the FE overlay camera shares order 10 with the in-race 3D hud (`osd_3d_interface::HUD_3D_ORDER`)
        if let Some(mut c) = cam {
            if c.is_active != active {
                c.is_active = active;
            }
        }
    }
    if !active {
        return;
    }
    let pad = pads.iter().next();
    let raw = |p: user_input::PadInput| raw_value(p, &keys, pad);
    let st = &mut *state;
    let next = st.mapping.step(&st.pads[0], &raw);
    st.pads[0] = next;
    let mut input = feck2::feck2_manager::FeInput::default();
    input.pads[0] = next;
    input.pads[1] = st.pads[1];
    let Some(f) = st.frontend.as_mut() else { return };
    // options edited elsewhere (the in-race menus) reach the front end's copy
    if options.is_changed() && !f.fe.options_changed {
        f.fe.options = (*options).clone();
    }
    // g_AttractMode and the game globals the main menu's attract calls use (FEMainMenuScreen 0x0023C400)
    if let (Some(a), Some(g)) = (attract.as_ref(), gm.as_ref()) {
        let link = &mut f.fe.attract;
        // written by someone else since the last frame = core's main menu stand-in (velocity/src/attract.rs, asked
        // to go) still makes these calls: leave them to it rather than run the idle timer twice
        let stand_in = a.is_changed();
        link.mode = (!stand_in).then(|| (**a).clone());
        link.rand = g.rand.clone();
        link.seed = (*clock * 1000.0) as u64 as u32;
        link.jump_active = g.jump.active && !link.loading_jump;
        link.started = false;
    }
    // XMem's status block is polled by the screens' updates (RaceCompleteScreen 0x00205900)
    if let Some(fl) = flow.as_deref_mut() {
        f.fe.save_flow = Some(std::mem::take(fl));
    }
    // the garage globals (`g_FEGameVariables`: cash, prestige, garage, setups) are core's `gm.vars`: the screens
    // work on them for the update and write their changes straight back
    if let Some(g) = gm.as_deref() {
        garage_in(&mut f.fe, &g.vars);
    }
    // the DJ booth's playlist mirror (feck2::playlist_link)
    if let Some(v) = visos_audio.as_deref() {
        f.fe.playlist.sync_in(v);
    }
    f.frame(&input, time.delta_secs());
    let mut plays = Vec::new();
    if let Some(v) = visos_audio.as_deref_mut() {
        f.fe.playlist.sync_out(v, |soundtrack, song| plays.push((soundtrack, song)));
    }
    if let Some(g) = gm.as_mut() {
        garage_out(&f.fe, &mut g.bypass_change_detection().vars);
    }
    if let (Some(fl), Some(v)) = (flow.as_deref_mut(), f.fe.save_flow.take()) {
        *fl = v;
    }
    let mut jumped = false;
    if let Some(m) = f.fe.attract.mode.take() {
        if let Some(a) = attract.as_mut() {
            if **a != m {
                **a = m;
            }
        }
        if let Some(g) = gm.as_mut() {
            if std::mem::take(&mut f.fe.attract.started) {
                g.rand = f.fe.attract.rand.clone();
            }
            // 0x000B3550 `CGameJump::Set`; core's front end update races it, the front end stops once in game
            if let Some(r) = f.fe.attract.jump.take() {
                g.jump.set(&r.track, &r.race, r.professional, r.wet, r.drag);
                jumped = true;
            }
        }
    }
    for r in f.fe.requests.drain(..) {
        out.write(FrontendRequestMsg(r));
    }
    if !f.fe.movies.is_empty() {
        movies.write(movie::MovieCommand::Play(std::mem::take(&mut f.fe.movies)));
    }
    let music: Vec<_> = f.fe.music.drain(..).collect();
    let sounds: Vec<_> = f.fe.sounds.drain(..).collect();
    let door_sounds: Vec<bool> = f.fe.foyer_vehicles.door_sounds.drain(..).collect();
    if let Some(mut a) = audio {
        for (soundtrack, song) in plays {
            a.write(velocity_audio::AudioCommand::PlayTrack { soundtrack, song });
        }
        for open in door_sounds {
            a.write(if open { velocity_audio::AudioCommand::CarOpen } else { velocity_audio::AudioCommand::CarClose });
        }
        use crate::feck2::feck2_manager::FeMusicCall as M;
        for m in music {
            a.write(match m {
                M::FadeMusic { to, env, time } => velocity_audio::AudioCommand::FadeMusic { to, env, time },
                M::StartLoopEvent210 => velocity_audio::AudioCommand::StartLoopEvent210,
                M::StopLoopEvent210 => velocity_audio::AudioCommand::StopLoopEvent210,
            });
        }
        for s in sounds {
            a.write(velocity_audio::AudioCommand::Gui(s));
        }
    }
    if std::mem::take(&mut f.fe.options_changed) {
        if f.fe.options.controller_config[0] != options.controller_config[0] {
            st.mapping = user_input::InputMapping::load_config(f.fe.options.controller_config[0]);
        }
        *options = f.fe.options.clone();
    }
    if jumped {
        st.stop_pending = true;
    }
    let Some(f) = st.frontend.as_mut() else { return };
    // transparent where the frontend draws nothing: the 3D foyer shows through
    // widescreen (user-requested deviation from the 4:3 original): more virtual width, content centred, bars tiled
    let size = views.iter().find_map(|(_, c)| c.and_then(|c| c.physical_target_size()));
    let (w, extra) = size.map_or((DISPLAY_W, 0.0), |s| render2d::wide_size(s.x as f32 / s.y.max(1) as f32));
    let frame = if extra > 0.0 {
        let mut dl = render2d::DrawList { quads: f.fe.draw.quads.clone() };
        dl.widen(0, extra, |_| 0.5);
        dl.rasterise_overlay_virt(&f.fe.textures, w, DISPLAY_H, render2d::SCREEN_W as f32 + extra)
    } else {
        f.fe.draw.rasterise_overlay(&f.fe.textures, DISPLAY_W, DISPLAY_H)
    };
    if let Some(mut img) = images.get_mut(&st.image) {
        if img.texture_descriptor.size.width != w {
            img.texture_descriptor.size.width = w;
        }
        img.data = Some(frame.data);
    }
}

/// `gm.vars` garage globals into the screens' `GarageVars` / quick race garage list
fn garage_in(fe: &mut feck2::feck2_manager::Feck2, v: &velocity_game::game_mode::game_player::CFEGameVariables) {
    fe.garage.cash = v.cash;
    fe.garage.prestige = v.prestige;
    fe.garage.current_vehicle = v.garage_vehicle;
    fe.garage.setups.clone_from(&v.garage_setups);
    if fe.quick_race.garage != v.garage {
        fe.quick_race.garage.clone_from(&v.garage);
    }
    fe.quick_race.garage_selected = v.garage_slot;
}

/// the screens' garage edits back into `gm.vars` (only what changed)
fn garage_out(fe: &feck2::feck2_manager::Feck2, v: &mut velocity_game::game_mode::game_player::CFEGameVariables) {
    v.cash = fe.garage.cash;
    v.prestige = fe.garage.prestige;
    v.garage_vehicle = fe.garage.current_vehicle;
    v.garage_setups.clone_from(&fe.garage.setups);
    if v.garage != fe.quick_race.garage {
        v.garage.clone_from(&fe.quick_race.garage);
    }
    v.garage_slot = fe.quick_race.garage_selected;
}

/// core's career state into the screens' `CareerView` (events, current event, flags) and core's answers
fn career_in(
    gm: Option<Res<velocity_game::GameManager>>,
    msgs: Option<Res<Messages<velocity_game::game_mode::career::CareerMessage>>>,
    mut cursor: Local<bevy::ecs::message::MessageCursor<velocity_game::game_mode::career::CareerMessage>>,
    teams: Option<Res<velocity_game::career_teams::CareerTeams>>,
    flow: Option<Res<velocity_game::save_game::save_vars::SaveFlow>>,
    mut state: ResMut<FrontendState>,
) {
    let Some(f) = state.frontend.as_mut() else { return };
    let c = &mut f.fe.career;
    // this frame's answers only (the screens read them in their update)
    c.messages = msgs.map(|m| cursor.read(&m).cloned().collect()).unwrap_or_default();
    if let Some(t) = teams
        && (t.is_changed() || c.teams.len() != t.teams.len())
    {
        c.teams.clone_from(&t.teams);
    }
    let Some(gm) = gm else { return };
    c.present = true;
    for m in &c.messages {
        if let velocity_game::game_mode::career::CareerMessage::ReturnedFromRace { .. } = m {
            // a started save is polled through `SaveFlow` by the screen; none started (core's view of quit / profile
            // differed from Start's, or the profile could not be gathered): -1 if core tried, else 0
            if !flow.as_ref().is_some_and(|f| f.in_progress()) {
                let tried = !gm.quit_race && gm.vars.save_enabled;
                c.race_save_status = Some(if tried { -1 } else { 0 });
            }
        }
    }
    c.save_enabled = gm.vars.save_enabled;
    c.quit_race = gm.quit_race;
    let db = &gm.event_db;
    let view = |e: &velocity_game::event_database::event_database::CEvent| career_event_view(db, e);
    if gm.is_changed() || c.events.is_empty() {
        c.events = gm.event_db.events.iter().map(view).collect();
    }
    c.event = match &gm.vars.event {
        Some(velocity_game::game_mode::game_player::EventRef::Database(i)) => gm.event_db.events.get(*i).map(view),
        _ => None,
    };
    c.completed_any = gm.vars.completed_any;
    c.profile_active = gm.vars.profile_active;
    c.unlocked_tiers = gm.vars.unlocked_tiers;
    c.unlock_message = gm.vars.unlock_message;
    c.team = gm.vars.team;
    // dword_6F71D8 (GameVars +0x268; core has it as unknown_264[1])
    c.tips_watched = gm.vars.unknown_264[1];
}

/// one database event as the hangout reads it (the `CEvent` fields and sub-records the browser and the messages use)
pub fn career_event_view(
    db: &velocity_game::event_database::event_database_track_loading::CEventDatabase,
    e: &velocity_game::event_database::event_database::CEvent,
) -> feck2::screens::hangout::CareerEventView {
    use feck2::screens::hangout::CareerEventView;
    {
        let round0 = e.races.as_ref().and_then(|r| r.rounds.first());
        let race0 = round0.and_then(|r| r.races.first());
        let record = race0.and_then(|r| r.race).and_then(|i| db.races.get(i));
        let last = e.races.as_ref().and_then(|r| r.rounds.last()).and_then(|r| r.races.first());
        CareerEventView {
            crc: e.crc,
            category: e.category,
            kind: e.kind,
            tier: e.tier,
            flags: e.flags,
            area: e.area,
            unlock_team: e.unlock_team,
            unlock_message: e.unlock_message,
            movie: e.unknown_4c[0],
            tips: e.unknown_4c[1],
            loading_tip: e.unknown_4c[2],
            race_count: round0.map_or(0, |r| r.races.len() as u32),
            laps: race0.map_or(0, |r| r.laps as u32),
            opponents: race0.map_or(0, |r| r.num_competitors as u32),
            race_type: record.map_or(0, |r| r.race_type as u32),
            track_icon: record.map(|r| r.icon.clone()),
            unlock_event: e.unlock.as_ref().and_then(|u| {
                let crc = *u.event_crcs.first()?;
                let ev = (*u.events.first()?)?;
                Some((crc, db.events.get(ev)?.tier))
            }),
            result_position: e.result.as_ref().map_or(0, |r| r.position),
            result_points: e.result.as_ref().map_or(0, |r| r.points),
            points: e.competitors.as_ref().map(|c| c.iter().map(|c| c.points).collect()).unwrap_or_default(),
            finish_position: last.and_then(|r| r.finish_positions.first().copied()).unwrap_or(0),
            current_race: e.current_race().map(|r| {
                let comps = e.competitors.as_deref().unwrap_or(&[]);
                feck2::screens::hangout::CurrentRaceView {
                    competitors: r.num_competitors,
                    humans: r.human_count(comps),
                    details: r.race.and_then(|i| db.races.get(i)).map(|d| (d.display_crc, d.category == 2)),
                    entrants: r.competitors.iter().map(|c| c.and_then(|i| comps.get(i)).map(|c| c.opponent_crc)).collect(),
                }
            }),
        }
    }
}

/// the screens' career requests to game flow (`GameFlowRequest`); `ClearEvent` writes `m_pEvent` directly as the
/// original front end does (CFEHangoutProgression 0x0028F790 step 8)
/// the licence plate globals (`dword_6F71DC` style, `unk_6F71E0` text): PlateCustomizationScreen's writes go to
/// graphics' `LicencePlate` (what LicencePlateRender 0x0012DC60 reads) and the style to GamePlayer +0x26C (core's
/// `vars.unknown_264[2]`, saved with the profile); otherwise the screens see the current values
fn plate_sync(
    mut state: ResMut<FrontendState>,
    plate: Option<ResMut<visos_render::licence_plate_render::LicencePlate>>,
    gm: Option<ResMut<velocity_game::GameManager>>,
) {
    let Some(f) = state.frontend.as_mut() else { return };
    let Some(mut plate) = plate else {
        f.fe.plate_changed = false;
        return;
    };
    if std::mem::take(&mut f.fe.plate_changed) {
        log::info!("licence plate: style {} \"{}\"", f.fe.plate.style, f.fe.plate.text);
        plate.style = f.fe.plate.style;
        plate.text = f.fe.plate.text.clone();
        if let Some(mut gm) = gm {
            gm.vars.unknown_264[2] = f.fe.plate.style;
        }
    } else if (plate.style, plate.text.as_str()) != (f.fe.plate.style, f.fe.plate.text.as_str()) {
        f.fe.plate.style = plate.style;
        f.fe.plate.text = plate.text.clone();
    }
}

type CoreCareer<'a> = (
    Option<ResMut<'a, velocity_game::game_mode::career::Career>>,
    Option<Res<'a, velocity_game::controller::user_input::UserInput>>,
    Option<ResMut<'a, velocity_audio::VisosAudio>>,
    Option<ResMut<'a, velocity_game::save_game::save_vars::SaveFlow>>,
    Option<ResMut<'a, Messages<velocity_game::game_mode::career::CareerMessage>>>,
    Option<ResMut<'a, Messages<velocity_game::game_mode::race_session::RaceSessionMessage>>>,
    Option<ResMut<'a, velocity_game::game_mode::race_cars::PlayerCarChoice>>,
    Option<Res<'a, velocity_game::game_mode::race_cars::RaceCarSourceRes>>,
);

/// FEDummyGameScreen.cpp 0x00209340, result branch: `dword_6F7244 += prestige`; the current race's finish positions
/// (the player `position`, the others random free places; nobody human: in slot order); race over (0x00091620, its
/// mileage), round over (0x00091810), event over (0x00091890); `byte_6F5D08 = 0`. Then RaceCompleteScreen's game
/// side (`career::race_complete`: temp event, the save, the prestige), as entering it after a race does.
fn dummy_result(gm: &mut velocity_game::GameManager, core: &mut CoreCareer, position: u32, prestige: u32) {
    use velocity_game::game_mode::race_session::{apply_event_outcome, RaceSessionMessage};
    let g = gm;
    g.vars.event_state = g.vars.event_state.wrapping_add(prestige);
    let mut session = Vec::new();
    if let Some(ev) = g.vars.current_event_mut(&mut g.event_db) {
        let comps = ev.competitors.clone().unwrap_or_default();
        if let Some(race) = ev.current_race_mut() {
            let n = race.num_competitors as usize;
            let mut used = [false; 16];
            if race.human_count(&comps) != 0 {
                let p = (position as usize).wrapping_sub(1) & 15;
                if let Some(f) = race.finish_positions.first_mut() {
                    *f = p as u8 + 1;
                }
                used[p] = true;
                for k in 1..n {
                    let mut j = 0usize;
                    while used[j] {
                        j = (g.rand.rand() as u32 % n as u32) as usize;
                    }
                    used[j] = true;
                    if let Some(f) = race.finish_positions.get_mut(k) {
                        *f = j as u8 + 1;
                    }
                }
            } else {
                for k in 0..n {
                    if let Some(f) = race.finish_positions.get_mut(k) {
                        *f = k as u8 + 1;
                    }
                }
            }
        }
        let (round_over, laps) = ev.race_finished(&mut g.rand);
        session.push(RaceSessionMessage::Mileage { laps });
        if round_over && ev.round_finished(&mut g.rand) {
            let o = ev.update_at_end_of_event();
            apply_event_outcome(g, &o);
            session.push(RaceSessionMessage::EventOver(o));
        }
    }
    g.quit_race = false;
    if let Some(m) = core.5.as_mut() {
        for s in session {
            m.write(s);
        }
    }
    let (Some(career), Some(ui), Some(flow)) = (core.0.as_mut(), core.1.as_ref(), core.3.as_mut()) else { return };
    let saved = velocity_game::game_mode::career::race_complete(g, career, ui, core.2.as_deref_mut(), flow);
    if let Some(m) = core.4.as_mut() {
        m.write(velocity_game::game_mode::career::CareerMessage::ReturnedFromRace { saved });
    }
}

#[allow(clippy::too_many_arguments)]
fn career_out(
    mut gm: Option<ResMut<velocity_game::GameManager>>,
    flow: Option<ResMut<Messages<velocity_game::GameFlowRequest>>>,
    team_requests: Option<ResMut<Messages<velocity_game::career_teams::TeamRequest>>>,
    mut state: ResMut<FrontendState>,
    mut core: CoreCareer,
) {
    let mut team_requests = team_requests;
    use feck2::screens::hangout::CareerRequest as R;
    use velocity_game::GameFlowRequest as G;
    let Some(f) = state.frontend.as_mut() else { return };
    let reqs: Vec<R> = f.fe.career.requests.drain(..).collect();
    let mut flow = flow;
    for r in reqs {
        log::info!("frontend career: {r:?}");
        let g = match r {
            R::NewProfile { name } => G::NewProfile { name },
            R::LoadProfile { name } => G::LoadProfile { name },
            R::SaveProfile => G::SaveProfile,
            R::DeleteProfile { name } => G::DeleteProfile { name },
            R::StartCareerEvent { event } => G::StartCareerEvent { event },
            R::ClearEvent => {
                if let Some(gm) = gm.as_mut() {
                    gm.vars.event = None;
                }
                continue;
            }
            R::ClearUnlockMessage => {
                if let Some(gm) = gm.as_mut() {
                    gm.vars.unlock_message = 0;
                }
                continue;
            }
            R::CountUnlockedTiers => {
                if let Some(gm) = gm.as_mut() {
                    let gm = &mut **gm;
                    gm.vars.count_unlocked_tiers(&gm.event_db);
                }
                continue;
            }
            R::SetTeam { team } => {
                if let Some(gm) = gm.as_mut() {
                    gm.vars.team = team;
                }
                continue;
            }
            R::CountTeamJoined => {
                // dword_6F71D4 = g_TeamsJoinedCount
                if let Some(gm) = gm.as_mut() {
                    let n = gm.vars.teams_joined_mut();
                    *n = n.wrapping_add(1);
                }
                continue;
            }
            R::SetTipsWatched { flags } => {
                if let Some(gm) = gm.as_mut() {
                    gm.vars.unknown_264[1] = flags;
                }
                continue;
            }
            R::SelectEvent { event } => {
                if let (Some(gm), Some(career), Some(choice)) = (gm.as_mut(), core.0.as_deref(), core.6.as_deref_mut()) {
                    let gm = &mut **gm;
                    match gm.event_db.find_event(event) {
                        Some(i) => {
                            gm.vars.event = Some(velocity_game::game_mode::game_player::EventRef::Database(i));
                            velocity_game::game_mode::career::start_event(gm, career, core.7.as_deref(), choice);
                        }
                        None => log::error!("assert: g_FEGameVariables.m_pEvent != NULL"),
                    }
                }
                continue;
            }
            R::DummyResult { position, prestige } => {
                if let Some(gm) = gm.as_mut() {
                    dummy_result(gm, &mut core, position, prestige);
                }
                continue;
            }
            // GameManager 0x000BBF60 on the current m_pEvent without StartEvent's reset
            R::ContinueEvent => G::StartCurrentEvent,
            R::SetCompletedAny => {
                if let Some(gm) = gm.as_mut() {
                    gm.vars.completed_any = true;
                }
                continue;
            }
            R::EvaluateUnlocks => {
                // the garage already went back into gm.vars (`garage_out`), so a bought car counts
                if let Some(gm) = gm.as_mut() {
                    let g = &mut **gm;
                    g.event_db.evaluate_unlocks(&g.vars, false);
                }
                continue;
            }
            R::CopyShownCar => {
                // dword_6F724C = FE3DVehicles +1824 (core's `PlayerCarChoice` is the current car)
                let shown = f.fe.foyer_vehicles.slots.first().and_then(|v| v.setup.as_ref());
                if let (Some(s), Some(db), Some(choice)) =
                    (shown, core.0.as_deref().and_then(|c| c.vehicles.as_ref()), core.6.as_deref_mut())
                {
                    *choice = velocity_game::game_mode::career::car_choice(db, s);
                }
                continue;
            }
            R::Team(t) => {
                match team_requests.as_mut() {
                    Some(m) => {
                        m.write(t);
                    }
                    None => log::warn!("frontend career: no team database for {t:?}"),
                }
                continue;
            }
        };
        match flow.as_mut() {
            Some(m) => {
                m.write(g);
            }
            None => log::warn!("frontend career: no game flow for {g:?}"),
        }
    }
}

/// apply changed options: audio volumes / output (`VsAudio` settings dirty), the display globals, then save them
/// to the profile. The first run only applies.
fn apply_options(options: Res<game_options::GameOptions>, audio: Option<ResMut<Messages<velocity_audio::AudioCommand>>>, mut saved: Local<bool>) {
    if !options.is_changed() {
        return;
    }
    options.apply_globals();
    if let Some(mut a) = audio {
        a.write(velocity_audio::AudioCommand::Settings(options.audio_settings()));
    }
    if *saved {
        options.save();
    }
    *saved = true;
}
