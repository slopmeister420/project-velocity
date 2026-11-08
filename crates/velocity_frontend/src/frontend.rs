//! frontend bring-up: the `GameMenuManager::GetFrontendManager` / `CreateAllScreens` sequence for the ported
//! screens, plus a headless driver used by the capture tests.

use crate::feck2::feck2_manager::{Feck2, FeInput};
use crate::feck2::screens;

/// keeps the screen objects alive (the original's `gp*Screen` globals).
pub struct Frontend {
    pub fe: Feck2,
    pub main_menu: std::sync::Arc<std::sync::Mutex<screens::startup::fe_main_menu_screen::CFEMainMenuScreen>>,
    pub legal: std::sync::Arc<std::sync::Mutex<screens::startup::fe_main_menu_screen::CFELegalScreen>>,
    pub movie: std::sync::Arc<std::sync::Mutex<screens::startup::fe_main_menu_screen::CFEMovieScreen>>,
    pub quick_race_text: std::sync::Arc<std::sync::Mutex<screens::quick_race::fe_quick_race_text_screen::CFEQuickRaceTextScreen>>,
    pub quick_race_car_select: std::sync::Arc<std::sync::Mutex<screens::quick_race::fe_quick_race_car_select_screen::CFEQuickRaceCarSelectScreen>>,
    pub quick_race_start: std::sync::Arc<std::sync::Mutex<screens::quick_race::fe_quick_race_start_screen::CFEQuickRaceStartScreen>>,
    pub profile_select: std::sync::Arc<std::sync::Mutex<screens::startup::fe_profile_select_screen::CFEProfileSelectScreen>>,
    pub profile_options: std::sync::Arc<std::sync::Mutex<screens::options::fe_profile_options_screen::CFEProfileOptionsScreen>>,
    pub plate_customization: std::sync::Arc<std::sync::Mutex<screens::options::fe_plate_customization_screen::CFEPlateCustomizationScreen>>,
    pub options: std::sync::Arc<std::sync::Mutex<screens::options::fe_options_screen::CFEOptionsScreen>>,
    pub video_options: std::sync::Arc<std::sync::Mutex<screens::options::fe_video_options_screen::CFEVideoOptionsScreen>>,
    pub pc_options: std::sync::Arc<std::sync::Mutex<screens::options::fe_pc_options_screen::CFEPcOptionsScreen>>,
    pub audio_options: std::sync::Arc<std::sync::Mutex<screens::options::fe_audio_options_screen::CFEAudioOptionsScreen>>,
    pub config_options: std::sync::Arc<std::sync::Mutex<screens::options::fe_config_options_screen::CFEConfigOptionsScreen>>,
    pub language_options: std::sync::Arc<std::sync::Mutex<screens::options::fe_language_options_screen::CFELanguageOptionsScreen>>,
    pub player_garage: std::sync::Arc<std::sync::Mutex<screens::garage::fe_player_garage_screen::CFEPlayerGarageScreen>>,
    pub car_customisation: std::sync::Arc<std::sync::Mutex<screens::garage::fe_car_customisation_screen::CFECarCustomisationScreen>>,
    pub loading: std::sync::Arc<std::sync::Mutex<screens::common::fe_loading_screen::CFELoadingScreen>>,
    pub tips: std::sync::Arc<std::sync::Mutex<screens::common::fe_tips_screen::CFETipsScreen>>,
    pub race_complete: std::sync::Arc<std::sync::Mutex<screens::results::fe_race_complete_screen::CFERaceCompleteScreen>>,
    pub dummy_game: std::sync::Arc<std::sync::Mutex<screens::results::fe_dummy_game_screen::CFEDummyGameScreen>>,
    pub multiplayer: std::sync::Arc<std::sync::Mutex<screens::multiplayer::fe_multiplayer_screen::CFEMultiplayerScreen>>,
    pub hangout: std::sync::Arc<std::sync::Mutex<screens::hangout::fe_hangout_screen::CFEHangoutScreen>>,
    pub audio_playlist: std::sync::Arc<std::sync::Mutex<screens::quick_race::fe_audio_playlist_screen::CAudioPlaylistScreen>>,
    pub car_showroom: std::sync::Arc<std::sync::Mutex<screens::showroom::fe_car_showroom_screen::CFECarShowroomScreen>>,
    pub pink_slip: std::sync::Arc<std::sync::Mutex<screens::garage::fe_pink_slip_screen::CFEPinkSlipScreen>>,
    pub time: f32,
}

impl Frontend {
    /// FECK2Manager.cpp 0x001FDC20 + 0x001FEB90 — `Initialise` then `CreateAllScreens` (ported screens only).
    pub fn new(options: crate::game_options::GameOptions) -> Self {
        let mut fe = Feck2::new();
        fe.load_resources(&options.language);
        fe.options = options;
        Self::create_all_screens(fe)
    }

    /// GameManager.cpp 0x000B5A80 — `FrontEndReInit` after a race: the screens are made again from scratch
    /// (`CFrontendManager::Initialise` + `CreateAllScreens`); the caller then starts the post race screen.
    pub fn reinit(&mut self) {
        let mut fe = std::mem::take(&mut self.fe);
        fe.reset_screens();
        // a jump's LoadingScreen hidden before its 10th frame
        fe.attract.loading_jump = false;
        let time = self.time;
        *self = Self::create_all_screens(fe);
        self.time = time;
    }

    /// FECK2Manager.cpp 0x001FEB90 / 0x001FEFC0 — `CreateAllScreens` (ported screens only) + `CreateCommonScreens`
    fn create_all_screens(mut fe: Feck2) -> Self {
        // CreateCommonScreens (0x001FEFC0)
        screens::common::fe_2d_overlay::create_common_overlay(&mut fe);
        // gpCommonKeyboard (FE3DCharacters.cpp 0x00267FC0)
        screens::common::fe_keyboard_overlay::create_common_keyboard(&mut fe);
        // FE3DCamera ctor 0x0026F3D0: the camera tables from \FE\Cameras.txt (0x0026F8B0)
        fe.foyer.load_cameras();
        // the cash readout's argument as the player variables set it (0x0027CDD0)
        screens::garage::ov(&mut fe, screens::common::fe_2d_overlay::update_cash);
        // CreateAllScreens order
        let movie = screens::startup::fe_main_menu_screen::create_movie_screen(&mut fe);
        let legal = screens::startup::fe_main_menu_screen::create_legal_screen(&mut fe);
        let main_menu = screens::startup::fe_main_menu_screen::create_main_menu_screen(&mut fe);
        let quick_race_start = screens::quick_race::fe_quick_race_start_screen::create_quick_race_start_screen(&mut fe);
        let quick_race_car_select = screens::quick_race::fe_quick_race_car_select_screen::create_quick_race_car_select_screen(&mut fe);
        let quick_race_text = screens::quick_race::fe_quick_race_text_screen::create_quick_race_text_screen(&mut fe);
        let language_options = screens::options::fe_language_options_screen::create_language_options_screen(&mut fe);
        let profile_select = screens::startup::fe_profile_select_screen::create_profile_select_screen(&mut fe);
        let profile_options = screens::options::fe_profile_options_screen::create_profile_options_screen(&mut fe, profile_select.clone());
        let plate_customization = screens::options::fe_plate_customization_screen::create_plate_customization_screen(&mut fe);
        let video_options = screens::options::fe_video_options_screen::create_video_options_screen(&mut fe);
        // port: the PC settings page (OPTION_PC)
        let pc_options = screens::options::fe_pc_options_screen::create_pc_options_screen(&mut fe);
        let audio_options = screens::options::fe_audio_options_screen::create_audio_options_screen(&mut fe);
        let config_options = screens::options::fe_config_options_screen::create_config_options_screen(&mut fe);
        let options = screens::options::fe_options_screen::create_options_screen(&mut fe);
        // CreateAllScreens order: customisation (0x00247990) then the garage (0x00246170)
        let car_customisation = screens::garage::fe_car_customisation_screen::create_car_customisation_screen(&mut fe);
        let player_garage = screens::garage::fe_player_garage_screen::create_player_garage_screen(&mut fe);
        // the garage car list (0x00244EA0) sets the garage screen's car
        let pink_slip = screens::garage::fe_pink_slip_screen::create_pink_slip_screen(&mut fe, player_garage.clone());
        let loading = screens::common::fe_loading_screen::create_loading_screen(&mut fe);
        let tips = screens::common::fe_tips_screen::create_tips_screen(&mut fe);
        let race_complete = screens::results::fe_race_complete_screen::create_race_complete_screen(&mut fe);
        let dummy_game = screens::results::fe_dummy_game_screen::create_dummy_game_screen(&mut fe);
        let multiplayer = screens::multiplayer::fe_multiplayer_screen::create_multiplayer_screen(&mut fe);
        // FECK2CreateCarShowroomScreen (0x0021C9C0) comes before the hangout's
        let car_showroom = screens::showroom::fe_car_showroom_screen::create_car_showroom_screen(&mut fe);
        let hangout = screens::hangout::fe_hangout_screen::create_hangout_screen(&mut fe);
        let audio_playlist = screens::quick_race::fe_audio_playlist_screen::create_audio_playlist_screen(&mut fe);
        // the garage car is set by the garage list screen (not ported); start on the current garage car
        player_garage.lock().unwrap().set_car(fe.quick_race.garage_selected, fe.garage.current_vehicle);
        Self {
            fe,
            main_menu,
            legal,
            movie,
            quick_race_text,
            quick_race_car_select,
            quick_race_start,
            profile_select,
            profile_options,
            plate_customization,
            options,
            video_options,
            pc_options,
            audio_options,
            config_options,
            language_options,
            player_garage,
            car_customisation,
            loading,
            tips,
            race_complete,
            dummy_game,
            multiplayer,
            hangout,
            audio_playlist,
            car_showroom,
            pink_slip,
            time: 0.0,
        }
    }

    /// one frame: `CFrontendManager::Update` then `Render`.
    pub fn frame(&mut self, input: &FeInput, dt: f32) {
        self.time += dt;
        self.fe.foyer.update(dt);
        self.fe.update(input, self.time);
        // FE3DCamera camera layer (0x00273AD0..0x00273C80): the camera stick held orbits the camera
        let stick = screens::garage::fe_player_garage_screen::camera_stick(&self.fe);
        if stick.0 != 0.0 || stick.1 != 0.0 {
            self.fe.foyer.camera_held(stick);
        }
        // FE3DVehicles' per-frame update (FEAlloyEditor_Basic.cpp 0x0025B090): the showroom turntable
        let now = self.fe.stopwatch;
        self.fe.foyer_vehicles.update_turntable(stick.0, now);
        self.fe.foyer_vehicles.step_doors(now);
        if self.fe.foyer_vehicles.bounce_on {
            self.fe.foyer_vehicles.update_heights(now);
        }
        self.fe.render();
        let now = self.fe.time;
        let Feck2 { characters, char_rand, .. } = &mut self.fe;
        characters.update(now, char_rand);
    }
}
