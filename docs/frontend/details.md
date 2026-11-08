# Frontend / UI

_owner: frontend agent_

### Talking to the frontend (game flow / core)

`velocity_frontend::FrontendPlugin` (wired in `velocity`) boots the frontend at `MovieScreen` unless the app was
started with `--track` or `VELOCITY_NO_FRONTEND` is set. Bevy messages:
- in: `FrontendCommand::Start("<ScreenName>")` (= `CFrontendManager::Start(crc)`, shows the FE),
  `FrontendCommand::Stop` (stop updating and hide — deferred until `State<EGameMode>` is InGame, so the loading
  screen stays up while the race loads; without game flow it hides at once), `FrontendCommand::ReturnFromRace`
  (`FrontEndReInit` 0x000B5A80 tail: `Feck2::return_from_race` starts `post_race_screen` and cuts the history
  back to it). `watch_game_mode` sends ReturnFromRace itself when the mode goes InGame/PostRace/EndOfGame/
  ReturnToFrontEnd -> FrontEnd, so game flow needs no call.
- out: `FrontendRequestMsg(FrontendRequest)`: `PlayIntroMovies` (movie screen activate, `GameManager::B4D30`),
  `SetControllerMask{player, mask}` (`GameInput::SetDeviceMask` from main menu start / quick race),
  `StartQuickRace`, `Exit`.
- on `StartQuickRace` game flow builds the event from `Frontend.fe.quick_race` (`QuickRaceSettings`: the
  `GameVars` quick race globals 0x6F70E8..: track/race indices into the frontend's `CEventDatabase`, race type,
  laps, opponents, weather, difficulty, pro, `player_setups[2]` = `VehicleSetup` of the chosen cars,
  `vehicle_tier`, `automatic`). Flow: QuickRaceStartScreen Select (0x00221EE0) sets `post_race_screen`
  "RaceCompleteScreen" / `return_screen` "QuickRaceStartScreen" and fades to `LoadingScreen`, whose Update
  (0x00241520) raises `StartQuickRace` on its 10th frame (= `GameManager__BBF60`, GM +532); after the race
  RaceCompleteScreen goes back to `return_screen` on its first update.
- the frontend is drawn as a 640x480 cpu-rasterised image on its own `Camera2d` (order 10) UI node.
- dependencies: `velocity_game` (event database), `velocity_physics` (vehicle database for car select; the
  frontend loads its own copy at start). Everything is read at runtime through `visos::file` from the raw game
  files; captures go to `target/fe_captures`, nothing is written into the game dir.

### Formats

- **Strings** (`Strings\StrIndex.idx` + `Strings\<Language>\<Section>.sbk`; note: these `.sbk` are *string*
  banks, not audio): ids are `crc32(id, 0)` **case sensitive** (`GENERAL_SELECT` = `0x2FF42DE2`); languages and
  sections in the index are `{u32 size, u32 crc, char name[]}` records; index entries 16 bytes
  `{crc, handle, u16 flags (1 static, 4 loaded, 0x40 runtime reserved), u16 length, data}`; section file
  `{u32 section crc, u32 count, u32 language crc, u32 next}` + strings `{u32 crc, u16 argc, u16 size,
  16-byte args, utf-16}`. 7 languages x 16 sections, 34k strings. Args are 16-byte records
  `{formatter crc, arg crc, value, u16 position}`; formatters INT, FLOAT (decimal comma per language), TIME
  (m:ss), TIME_MILLI, CASH, STRINGID (inserts another string); `SetArgument(string, arg, value)` 0x2EB360.
  Lookup misses return `NOT FOUND` / `NOT LOADED` / `NO INDEX` like the original.
- **Fonts** (`Fonts\FONTS.VCD`: `<name>.fml` FontCreator xml, `<name>.fob` metrics, `<name>.vtf` page): see the
  header of `fonts/font_manager.rs`. Space is drawn with the metrics of `'s'` (tweak at 0x6C18E0 = -1001).
  `CFontHelper` maps font ids: FE set 0 Velocity10, 1 FEFont11, 2 FEFont12, 3 FEFont14, 4 FEFontTitle,
  12 debug; ids 0..4 have **x scale 0.75**.
- **Textures** `.vtf` (`VTP_`): `u16 version, u16 mips, u16 w, u16 h, u16 w, u16 h, u32 fmt, u32 fmt, u32, u32
  data offset`; format 13 = DXT5 (every FE/OSD/font page), 2 = 8-bit swizzled. Private reader `vtf.rs` until
  visos_render publishes one.
- `Input\Xboxpad\Config1.ini`: `Input.SetMapping <Action> <Expr>` (`DIGITAL()`, `DEADZONE(x,d)`); action order
  (UserInput.cpp 0x1E7C20): MenuLeft, MenuRight, MenuDown, MenuUp, MenuUpDown, MenuLeftRight, MenuSelect,
  MenuCancel, AlloyEditHoriz/Vert, MouseLeftRight/UpDown/Select, Pause, Online, Skip, MenuOption1/2, MenuL1..3,
  MenuR1..3, CameraRotateHoriz/Vert, CameraChange, Car*... Private reader `user_input.rs`.

### FECK2 (src\Feck2)

- **Screens are built in code**, not data: each screen's `Init` fills an `SElementDesc` (84 bytes: +8 position,
  +40 colour, +44 name crc, +52 vcd, +56 texture, +64 kind, +68 font id, +72 string crc (0xFFFF none)) and calls
  `CLayer::CreateElement`; the desc is reused between calls, so unset fields carry over (the port does the same).
  Element kinds: 1 2D sprite, 2 2D line, 3 3D, 4 camera, 5 static text, 6/7 character.
- **Virtual screen is 512x512** stretched to the display (FE textures are authored 512 wide, text boxes centre on
  x=256). 2D sprites: quad `(-pivot-0.5)*size`, x pre-scaled by 1.33331 (flag 4) then divided by aspect 1.3333334.
- Lists: a screen's layers ascend by priority (0 first for events, drawn last); a layer's elements are prepended,
  so the last created element is drawn first.
- Events (`CEvent` id, controller, param a, param b; 91 ids): 1 assign functors, 2 post-enter, 3 global
  preload, 4 preload, 8 exit, 9 enter, 10 activate, 11 start, 12 leave, 14 exit, 15 start screen, 18 goto,
  19 goto previous, 20 goto next, 21 update (param a = dt float seconds), input press/held/release triplets:
  up 22, right 25, down 28, left 31, camera L/R/D/U 34/37/40/43, select 46, pause 49, online 52, cancel 55,
  option1 58, option2 61, L1 64, L2 67, L3 70, R1 73, R2 76, R3 79. Screen change (`OnStartScreen` 0x201940):
  new.Enter(9), new.PostEnter(2), old.Leave(12), new.Activate(10), new.Start(11).
- Element animation durations are int "seconds" decremented by `(int)(left - dt)`: effectively frame counts.
- Common overlay (`gpCommon2DOverlay`, 0x7696B0): borders, nav prompts (A select ...), title, messages,
  dialog, 2D background on three layers around the screen's own.
- Object files are shifted against `objregions.tsv`: e.g. `FEMainMenuScreen.cpp` really starts at 0x23B370
  (main menu, legal 0x23D1D0, movie 0x23E760); the screen's `FECK2Create*Screen` tags in `CreateAllScreens`
  (0x1FEB90) name the create function that follows each tag.

### Ported

| module | original |
|---|---|
| `string_manager::string_manager` | StringManager\StringManager.cpp (index, sections, lookup) |
| `fonts::font_manager` | Fonts\FontManager.cpp (`CBaseFont` metrics, line layout, wrap, align, draw, kerning) |
| `font_helper::font_helper` | src\FontHelper\FontHelper.cpp (font id table, FE / game sets) |
| `feck2::feck2_manager` | FECK2Manager.cpp (screens, history, global events, input -> events, update, render) |
| `feck2::feck2_object` | FECK2Object.cpp (events, descs, CScreen/CLayer, element base, move animation) |
| `feck2::feck2_elements::*` | FECK2Element2D.cpp, FECK2Element2DLine.cpp (+ GeomTool 0x003C6CF0), FECK2ElementStaticText.cpp |
| `feck2::screens::common::fe_2d_overlay` | FE2DOverlay.cpp (creation, attach, reset, nav prompts, borders, load) |
| `feck2::screens::startup::fe_main_menu_screen` | main menu, legal, movie screens |
| `feck2::screens::startup::fe_profile_select_screen` | FEProfileSelectScreen (create 0x00232C60, code 0x00232D60..0x00234EB0, idb "FEOnlineSignInScreen"): list / new / load / save / delete |
| `feck2::screens::hangout::fe_hangout_screen` | FEHangoutScreen.cpp (create 0x00218390, code 0x002181A0..0x0021FA60): characters + props (Start 0x00219300 / Leave 0x00219920), menus, quit dialog, tutorial start |
| `feck2::screens::hangout::fe_hangout_progression` | FEHangoutProgression.cpp (0x0028B8A0..0x00290340): Tory's event browser, StartEvent, the after-race messages |
| `feck2::screens::common::fe_keyboard_overlay` | FEKeyboardOverlay.cpp (0x00268850..0x0026F7A0, `gpCommonKeyboard` made by FE3DCharacters 0x00267FC0) |
| `feck2::screens::common::fe_spline` | `CCatMullRom` / `CSimpleSpline` (0x0007BA20..0x0007D380, 0x0007FBD0..; idb Characters / CircuitEditorTools) |
| `plate_font` | `PlateFontHook` for graphics' LicencePlateRender 0x0012DF40 (FE set FEFont12 id 2, game set FEFont11 id 6) |
| `feck2::screens::quick_race::fe_quick_race_text_screen` | FEQuickRaceTextScreen.cpp (create 0x226390, track/race/options rows, random) |
| `feck2::screens::quick_race::fe_quick_race_car_select_screen` | car select (create 0x2223D0: tier -> car -> colour -> commit) |
| `feck2::screens::quick_race::fe_quick_race_start_screen` | race summary (create 0x2203A0, in FECarShowroomScreen.cpp region) |
| `osd::osd_manager` | Osd\OSDManager.cpp (item pool, anchor/justify layout 0xEE5F0, text render 0xEEA70, FormatTime 0xEF840) |
| `osd::osd_screens::osd_in_game_screen` | OSDInGameScreen.cpp (Create 0xF82F0, Update 0xF99E0 race-status part, name tags 0xF9340) |
| `osd::osd_screens::osd_screens` | OSDScreens.cpp `g_OSDPauseScreens` (0xFC9E0, 0xFCAF0) + the pause screens' callbacks (0xF7E90.., 0xFB3A0.., 0xFD310..) |
| `osd::osd_screens::osd_race_over_stat_screen` | `COSDRaceOverStatScreen` (0x000F0100..0x000F3800): post race results steps, race table 0x000F09C0, page 3 summaries 0x000F2EC0 |
| `feck2::screens::options::fe_profile_options_screen` | ProfileOptionsScreen (create 0x00234FE0, code ..0x00236290): plate / info / save / load / new / delete |
| `osd::speedo` | Osd\Speedo.cpp + OSD3dInterfaceConfiguration.cpp hud config (0xEB9F0) |
| `osd::osd_map` | CircuitEditor\CHUDMapConstructor.cpp, OSDMap\COSDMapRender.cpp |
| `game_menu_manager::game_menu_manager` | GameMenuManager.cpp, Menus\GamePauseMenu.cpp, CQuitMenu + CRaceOverMenu (GameTweaksMenu.cpp region) |
| `hud` | bevy glue: `HudPlugin`, `RaceHudInput`, `HudCommand`, `PauseMenuMsg` |
| `feck2::screens::garage::fe_player_garage_screen` | FEPlayerGarageScreen.cpp (code 0x00246170..0x00247990): use / sell car + yes/no dialog |
| `feck2::screens::garage::fe_car_customisation_screen` | FECarCustomisationScreen.cpp (code 0x00247990..0x00254130): parts / paint carousel, buy dialog |
| `feck2::screens::common::fe_loading_screen` | FETipsScreen.cpp `LoadingScreen` (create 0x002405C0): background, tip, starts the game |
| `feck2::screens::results::fe_race_complete_screen` | RaceCompleteScreen (create 0x00204E40): CONTINUE overlay, after-race save wait, back to `return_screen` |
| `feck2::screens::common::fe_tips_screen` | FETipsScreen.cpp `TipsScreen` (`gpTipsScreen`, create 0x0023F110, code ..0x00240590): tutorials list, tip text, tip movies |
| `bink::*` | RAD Bink 1 (XBE `BINK` section 0x0053E0C0..0x005500C4): container, video, audio, tables |
| `vs_movie::vs_movie_stream_xbox`, `movie` | VsMovie\Xbox\VsMovieStreamXBOX.cpp + full-screen glue (GameManager__BE9F0) |
| `osd::osd_3d_interface` | Osd\OSD3dInterface.cpp 3D part (0x000E9C90 Register, 0x000EA2B0 TFT / map, 0x000EAD80 LCD) + `hud::run_osd_3d` |
| `feck2::screens::options::fe_plate_customization_screen` | PlateCustomizationScreen (create 0x0023A540, code ..0x0023B370) |
| `feck2::screens::results::fe_dummy_game_screen` | FEDummyGameScreen.cpp (create 0x00207E20, Init 0x00207F50, events 0x00208560, code ..0x002096D0): the between-races screen |
| `feck2::screens::multiplayer::fe_multiplayer_screen` | `MultiplayerScreen` (0x00217230..0x00218130, idb FEOnlineMatchInfoScreen range): Online / System Link / Split Screen |
| `fe_tweaks` | the frontend's nine AddTweaks blocks + the tweak-changed callback FECK2Manager.cpp 0x00202EA0 (see "Debug-menu tweaks") |
| `feck2::screens::common::fe_car_stats_list` | R2 car statistics list (0x002894A0..0x0028B800, idb FECK2ElementStaticText): car select +5448, garage +40 |
| `feck2::screens::garage::fe_pink_slip_screen` | `PinkSlipScreen` (create 0x00244EA0, code ..0x00246060, idb FENetworkServerDetailsScreen range, tagged online by mistake): the garage's car list |
| `feck2::screens::showroom::fe_car_showroom_screen` | FECarShowroomScreen.cpp (create 0x0021C9C0, code 0x0021CAB0..0x002201D0, idb FEHangoutScreen range): car list per tier, the car on the turntable, purchase |
| `loading_bar` | GameDataLoading\LoadingBar.cpp (`CLoadingBar` Begin/Stage/End) + GameManager 0x000B89E0 step strip (unused: loads have no per-step frames yet) |

Quick race flow: MainMenu (Start -> ProfileSelect, B back, A on QUICK RACE) -> QuickRaceTextScreen
(Select) -> QuickRaceCarSelectScreen (tier, car, colour stage, commit) -> QuickRaceStartScreen (A =
`StartQuickRace`, X back to car, Y back to track, B to main menu). Car select reads tier counts / nth car /
default setup from `velocity_physics::vehicle_database`. Car names are the vehicle entry hash used as a string id.
Car select 3D / panels (see "3D foyer" below): camera entries 7 / 8 / 9 per stage (0x224E80 / 0x224EC0), the
car goes up on the "quickR" Vehicles.txt PLAYER1CAR spot after the stage-2 countdown (countdown 1 hides both
slots, -1 shows, 0x225890 → 0x259D20); option 1 in stage 2 opens the colour swatches
(`fe_colour_swatches`, FEHangoutProgression 0x290DF0..0x291CA0: 5x6 "white" 16x16 squares 20 apart at (80,365),
tinted top/bottom with the record's D3DCOLORs, 16x4 highlighter, name text at (140,345), d-pad moves = 0x224E20
recolour of slot 0); select/cancel close them; the committed setup is slot 0's (with its colour). Car info panel
(`fe_car_info_panel`, panel 0 of 0x287A30..0x2893F0, origin (320,335)): cost/mileage/transmission lines, three
9-segment bars (accel `(6-3*(ts+acc)/2 - 5)/5` (0x189500), top speed `(t-120)/100`, handling modifier; orange
(255,110,0) lit / grey (64,64,64)), FE_statsBox + FE_info_highlighter sliding 250 px over 30 frames, R2 prompt =
overlay nav 13 at (320,268) (`position_nav` 0x27D3D0). Logo wall: see "Background material textures". Not ported:
automod, the camera's mode-2 timing
(`2720D0(2, 0.25)`, 5 s move). Camera tables: the FE3DCamera ctor (0x0026F3D0 -> 0x0026F8B0) overwrites the static
init tables from `\FE\Cameras.txt` (11 x CamPos / CamLookAt / CamAxis; entries 0..4 equal the statics, 5..10 differ,
e.g. 9 = CARMODIFICATION (3.197,1.2,24.404) looking at (5.289,0.7,17.787), i.e. the PLAYER1CAR bay) —
`FoyerState::load_cameras`, `FoyerState.table` (this resolved the "camera 9 misses the spot" question).
Checked against the data (not the look): stage -> entry 7/8/9 = 0x00224E80 (`this+1360` 0 -> 7, 1 -> 8, else 9);
Cameras.txt 7 TIERSELECT (7.9,2.61,7.476)->(9.774,2.541,8.21) and 8 CARSELECT (8.1,1.3,8.1)->(9.595,1.0,9.002)
look at the logo wall, not the car (the car is hidden until stage 2); 9 aims 7° off the quickR PLAYER1CAR spot
(4.22,0.2,19.5, rot 326, 5.1 m away) and 10 QUICKRACESUMMARY (0.7,1.1,19.0)->(6.289,1.2,20.1) 11° off (3.6 m), both
well inside the 50° fov. Shot: `research/screens/fe_carselect.png` (= `scene_car_select_colour_stage`, entry 9
frames the whole 300ZX left of the info panel, re-run after the Cameras.txt change).
Main menu state `dword_6F6FD0` = `Feck2.main_menu_state` (was a process static, which leaked between the capture
tests: `capture_options_flow` left it at 3, so `capture_quick_race_flow`'s Start did nothing).

FE draw order is submission order (`Renderer` 0x2A9B00 draws immediately, fonts flush per string), so the car
select's car name (created after the title bar) is drawn under the pulsing bar as in the original.

#### In-race HUD (`hud::HudPlugin`, added by `FrontendPlugin`)

- in: `HudCommand::Start { hud, drag, inside_edge, outside_edge, closed }` at race start (`hud` = digital hud
  pack name under `OSD\HUDS`, falls back to `digione`; edges = the track's inside / outside x/z polylines for
  the minimap, empty = no map), `HudCommand::Stop`, `HudCommand::OpenPauseMenu` (sent by core's 0x000BC3D0 pause
  gate, no pause before Go — the HUD no longer reads the pause button itself), `HudCommand::LeaveAllMenus`
  (`CGameMenuManager::LeaveAll` 0x000A9BD0: pause button resume, restart, quit). Every frame game flow fills
  the `RaceHudInput` resource: `status: OsdRaceStatus` (racing, race length, position, lap/laps, drag rounds,
  lap time, points, combo, multiplier, wrong way), `speedo: SpeedoInput` (speed m/s — shown * 2.25, rpm, gear
  (0 N, <0 R), nos / zone / turbo ratios), `map: MapInput` (player pos, forward vector, rivals, police).
- out: `PauseMenuMsg(PauseMenuRequest)`: `Resume`, `Restart`, `QuitToFrontend`, `OnlineOptions`,
  `RumbleChanged(bool)`, `OptionsChanged` (volumes / audio mode / display toggles in `CGameMenuManager.options`).
- drawn into a transparent 640x480 overlay (`DrawList::rasterise_overlay`) on camera order 11, no clear.
- also in: `HudCommand::CinematicBars(bool)` (OSDManager 0x000ED6E0, see "Cinematic bars"); name tag colours =
  `VsAIDriver::name_colour()` D3DCOLOR (core's `hud.rs` passes it; 0x000F9340 replaces only the alpha).
- `HudPlugin` is the shipped overlay; `crates/velocity/src/hud.rs` only bridges game flow to it (Start/Stop,
  `RaceHudInput` from `RaceStatus`, `GameMenuMessage` -> `HudCommand`, `PauseMenuMsg` -> `GameFlowRequest`).
- **Cruise:** one OSD screen for every race type (`OSDInGameScreen__FAB40`, no race-type gate in Create/Update
  except drag). Update keys everything on driver +244 ("crossed the start line"), which only
  `VsRaceManager::UpdateLapCrossing` sets — and `VsRaceManager::Update` skips that for race type 1 (0x0003B349). So
  a cruise always shows lap time `--:--.--` (`--:--` without hundredths), lap `--/<event laps>`, position
  `OSD_NO_POS`, plus race length / points as usual. `hud.rs` passes `racing = status.racing && !cruise`.

**OSD space (resolved):** items live in a **1333x1000** space with **y up**, mapped onto the same 512x512 2D
target as the FE: `sx = x/1333*512`, `sy = 512 - y/1000*512` (OSDManager 0xEF6D0/0xEF710, constants
`flt_636BA4`/`flt_636BA8`, target `flt_628E34` = 512); pixel sizes go back with 0xEF7E0/0xEF810. The renderer
2D size (`dword_7806C8+8/+12`) is just the back buffer (`CRenderer::Init` 0x2A81C0). Anchors 0x80/0x100/0x200
x = 0/1333/666, 0x400/0x800/0x1000 y = 0/1000/500; justify 2/4/8 x, 0x10/0x20/0x40 y; 0x2000 drop shadow,
0x8000 hidden, 0x4000 disabled; mask +96 vs manager mask. Fonts from the game set (ids 7 Arial09, 8 Arial11,
9 Velocity15, 10 Velocity26, 12 Tahoma12Bold for menus). Graphic items: bottom-left at (x, y), extending up.

**Speedo (`osd::speedo`):** hud packs `OSD\HUDS\<n>.vcd` = `<n>-config.txt` + `-base/-revs/-nos/-zone/-boost/
-font/-thumb.vtf`. Config = `#` comments and one `(…)`/`"…"` value per line in a fixed order (parser follows
OSD3dInterfaceConfiguration 0xEB9F0; all 9 packs parse). Gauges dispatch `[shape + 5*format]` (0xF42E0):
needle, h/v bar, circle segment (analogue: 15 deg wedges; digital: one wedge per lit segment, rpm/6000),
offset dial; digits from the font strip (`0123456789RN`, u = glyph*spacing/fontwidth). Geometry is in HUD
units (`px / tex size * hud size`), projected through the 3d interface camera (below); the `+44` gauge pass
(blend 4) is not drawn. The analogue gauge models in OSDM.vcd (`hud_rev_frame_*`, `gauge_rev_*`, ...) are loaded by
no code (no string refs) and `OSDD.VCD` `.osd` is unused too.

**3D hud (`osd::osd_3d_interface`, done)**: GameManager's render 0x000BA112 selects VsCameraManager camera 5 (the 3d
interface camera, 0x000B75D0: eye (0,0,30) at the origin, fov 45° vertical, 4:3, near 1 far 600) before 0x000EA2B0 —
so the hud space is a perspective view, not the car camera. `project()` maps hud points to the 512 target (speedo
quads, exact as they face the camera); the models render on a Camera3d at order 10 (`HUD_3D_ORDER`, the FE overlay
camera is deactivated while hidden), layer 12, no clear. Slots (0x000E9C30, 96 B: model, render struct +4 with world
matrix ptr and the light set +2824 at +16, matrix +32): +2344 `tft_frame`, +2440 `tft_hinge`, +2536 `tft_screen`,
+2632 `hud_map_frame`, +2728 `hud_map_screen` (strcpy from `off_686004..686014` into the config at +832/+896/+960/
+1280/+1344; Hex-Rays drops the source, read the disassembly 0x000EC874 / 0x000ECB4F); textures in OSDT.VCD. Per
render (inside the speedo's OSD gate, single player): TFT pos = config +800 with y += `dword_6FD1E0` (static drop,
first 3.0 `dword_685FFC`) taken *before* the step; wanted (+2896): flip +2900 += 0.1 (`685FF8`) to 0, drop -= 0.16
(`685FF4`) to 0; else flip -= 0.1 to -pi, drop += 0.16 to 3; hinge = Renderer 0x002AB7E0(rot) + pos, frame = rot.x +
flip, screen on the frame's matrix: LCD_Movie (movie texture, blend 1, z off) when wanted && +2904 >= 2 && flip >=
-1.3, else its own shader (FuryInstance 0x0012A7D0). Map: config +1248 pos with x/y += slide (+2932), rot +1264;
frame model, then COSDMapRender::Render 0x000E9670 draws `hud_map_screen` once per layer (map, grid, police, blips,
each with its texture matrix) — here composited on the cpu into a 256² texture (`HudResources.map_screen`, panel =
the whole texture) put on the screen model. The map
frame is nearly invisible as authored (its HUD_rev_ver5_backPlate_001 ring area is black with low alpha; opaque it
shows as a black ring). Core sends no track edges (`velocity/src/hud.rs`), so no map screen in game yet; the map
texture's orientation on the model is unchecked. Dev: `VELOCITY_TFT=TFT\Javier\UK_male_1_anim3_A` wants that line
whenever none is wanted. Shot `research/screens/fe_osd3d.png` (plain smoke with that env, LADown: TFT top left with
the movie, speedo). Light set wired (see "Requests from graphics (3d hud lighting …)": the OSDM prims show texture x
the set's ambient, white without a police flash). Tests `load_every_osdm_model`, `hud_lights_follow_the_police_flash` (all OSDM models parse, the five have their OSDT textures),
`tft_flips_open_and_closes`; `capture_osd_in_game` also writes `osd_map_screen.png`.

**Minimap (`osd::osd_map`):** `CHUDMapConstructor` 0x7DEE0 ported exactly (square bounds over both edges,
border pad, Bresenham edges = 255 into `[x*size+z]`, 0x80 outline pass, palette = hud map colour with
alpha `max(index, min)`); `COSDMapRender::Update` 0xE8E70 window = 900 world units, rotated by heading + pi
(plus a quarter turn so forward is up — the map screen model's uv orientation is assumed), blips
`plyrblip/rvalblip/blueblip/redblip` and `nav_grid` from `OSD\OSDT.vcd`. Drawn into the `hud_map_screen` texture
(see "3D hud").

**Pause menu (`game_menu_manager`):** `CGameMenuManager` stack (8), `CGameMenu` layout (≤13 rows, 42 apart,
centred 800 wide), up/down with 0.2 s repeat, select (action 6), cancel (7) pops (last one = resume),
left/right (0/1/5). Pause items (0xAF070): Resume, Audio, Video, Rumble (idRumble), Restart, Quit to FE,
Online Options; Audio (0xAD940: six volumes ±0.05, audio mode 0 stereo/1 surround/2 mono, Done), Display
(0xAE880: ratio, TFT, names, jump/crash/police cam toggles, Done), Quit confirm (0xB0DB0: YES/NO; the confirm
text is drawn by the quit screen). All in-race menus set no-frame, and `CGameMenu::Build` (0x000AAC30) does
everything inside its no-frame check, so the menus draw nothing themselves.
**Per game update (1:1)**: InGameUpdate 0x000B77F0 runs `g_OSDPauseScreens.Update` (0x000FCAF0), the OSD manager,
then `g_GameMenuManager.Update` (0x000A9DF0: top menu vtable +4 with dt = the update step 0x000BF650, 1/24), later
the attract fade. `HudResources::game_update(input, pad, dt)` does those three; `run_hud` steps the input mapping once
per owed GameTick (only the first update of a frame sees a press). `frame` / `frame_slide` no longer take input. The
repeat (0x000AA3E0, `+588` 0.2 = 0x3E4CCCCD) therefore counts game updates: a held DOWN moves again on the 6th update.
**Sweep quit timeouts (2026-11-05, smoke script)**: the smoke's 3-frame minimum hold outlasted the 0.2 s repeat on
slow frames, so DOWNs moved twice: the quit landed on ONLINE_OPTIONS (no `pause menu:` line, stuck paused) or wrapped
to RESUME (`pause menu: Resume`). Fix filed in core.md ("Requests from frontend"). Test in
`capture_race_over_menu_and_attract_hud` (held down: 1 move for 5 updates, 2nd on the 6th). Debug trace:
`RUST_LOG=info,velocity_frontend::game_menu_manager=debug` (every move with dt, every select). Checked with the per-update
menu: plain smoke exit 0, 5 single moves, Quit select -> `QuitToFrontend` (`research/fe_smoke_menu.log`). The menu /
OSD pause screens / CRaceOverMenu quit flow itself was not at fault; the remaining fix is core's smoke hold.

**OSD pause screens (`osd::osd_screens::osd_screens`, `g_OSDPauseScreens` `dword_6FDC70`)**: five 52-byte
`COSDScreen`s (InGameInit "Pause Screens Init" 0x000FC9E0) draw the menus as OSD items: 0 pause, 1 display, 2 audio,
3 race over, 4 quit. State machine 0x000FC7D0 (+11 bit 1 active / bit 2 items made, +12 state, callbacks +28 init,
+32 state 1, +36 state 3, +40 state 4, +44 release (state 5), +48 default = state 2), run once per game update
(`CGameMenuManager::update_screens` from `HudResources::game_updates`; InGameUpdate "g_OSDPauseScreens.Update",
mode 2 after the menu manager). Menu vtable (+0 dtor, +4 Update, +12 OnEnter, +16 OnLeave, +24 OnSelect, +28 OnSide):
OnEnter = 0x000AE100 (screen active, state 0, menu), OnLeave = 0x000AE140 (state 4); EnterMenu 0x000A9A40 calls +12 on
the new top only (a covered menu keeps its screen, its items are hidden by mask 4), the pop 0x000A9C40 calls +16 then
the dtor. Items: font 6 (FEFont11), x 640 (`dword_637274`), layer 110, flags 0x28, first row orange
VISOS(0x80,0x7C,0x4B,3), others white; state 2 recolours by `selected`. Rows (y0, step, mask): pause 650/55/4
(0x000FB3A0, plus the "pauseScreen" osdT.vcd panel 1600x600 at (640,500) layer 1000 flags 0x48 mask 0, and online
MENU_RESTART disabled), display 650/55/64 (0x000FD310), audio 650/45/128 (0x000F7E90), race over 575/75/256 (0x000FC210,
panel mask 256), quit 525/75/2048 (0x000FBD40, panel + the confirm text at y 685 flags 0x48). Only the pause screen
fades: 0x000FB7A0 alpha = fade*255 (disabled rows 0x40*fade), fade += 0.25 per update until > 1; 0x000FB940 the
reverse; its state 2 (0x000FBA30) uses alpha 0x40 (VISOS, = 128) for disabled rows. OSD mask bits of the menus
(set 0x00063030 / clear 0x00023D00): pause ctor `|= 8`, leave `&= ~8`; audio / display ctor `|= 4`, enter `&= ~128` /
`~64` `|= 4`, leave `|= 128` / `64` `&= ~4`, dtor `&= ~4`; quit ctor `|= 4 | 8 | 256`, enter `&= ~2048 |= 4`, leave
`|= 2048 &= ~4`, dtor (0x000B10B0) mode 1 `&= ~4`, mode 2 `&= ~256` (`CGameMenuManager.game_mode` from `gm.mode`).
**OSD draw order (fixed)**: 0x000EDB00 links a new item before the first one with a layer <= its own and Render
0x000EF4A0 walks from the head, so high layers are drawn first and low layers on top (newest first among equals,
`SOSDItem.seq`); the port drew ascending before.

**Post race menu (`CRaceOverMenu`, GameTweaksMenu.cpp, vtable 0x0062F3E4)**: GameManager::Update 0x000BB1C0 mode 2:
unless a drag round follows (race type 5, round +0x148 < rounds +0x144, not out +0x150 — core then goes to mode 1),
enter `new CRaceOverMenu` when no menu is open, then VsAudio::Pause (0x00049090); the HUD's `post_race_menu`
(State PostRace and `gm.mode == 2`) does this. Ctor 0x000B1450 "RaceOver": MENU_RESTART_EVENT (1, not online
`byte_6F7158`), MENU_RETURN_TO_FE (2), MENU_CONTINUE_EVENT (3, unless event +0x15 & 2 completed / & 4 failed or
drag), 800x600, no frame, OSD mask `|= 4 | 8, &= ~256`. EnterMenu calls vtable +12 once (0x000B16E0, selection via
0x000AA110 which ignores 0 / out of range): online 1, failed 0, not completed 2, else 1. Select 0x000B17A0: 1
failed -> restart at once (0x000B12A0), else CQuitMenu(restart); 2 failed / completed -> `byte_6F5D09` = failed,
LeaveAll, SetGameMode(3), else CQuitMenu(quit) (YES = 0x000BBF90(0) quit race); 3 -> `byte_6F5D09` = 0, LeaveAll,
SetGameMode(3). `PauseMenuRequest::EndOfGame { quit_event }` is handled by `hud::end_of_game` (`gm.quit_event` =
`byte_6F5D09`, then the menu's own `gm.set_game_mode(3)` + NextState + GameModeMessages). The HUD's Stop is deferred
while the state is PostRace (`HudState.stop_deferred`), so the OSD and menu stay up until EndOfGame. The race over
and quit screens are OSD pause screens 3 / 4 (above). Test `capture_race_over_menu_and_attract_hud` (`race_over.png`, `attract_hud.png`). Game shot
`research/screens/fe_postrace.png` = `SMOKE_REVIEW=1 SMOKE_RACE_TYPE=2 SMOKE_AREA=2` (London sprint 02, ai-driven,
completed quick race: RESTART_EVENT / RETURN_TO_FE selected, race items masked by state 7, the speedo is not an OSD
item). The drag check uses `gm.race.race_type` (core's): the temporary jump events' race entries carry no type, so
event race +4 read 0 for LADown drag 01 and the menu opened after round 1.

#### Debug-menu tweaks (`fe_tweaks.rs`, core's `CTweakManager` on `gm.tweaks`)
- Registrations (IDA dumps `research/fe_dc.c` = last run only; `research/fe_tweak_readers.c`, `fe_tweak_info.txt` (callers,
  .data values, xrefs of every global), `fe_tweak_order.txt`, `fe_tweak_modes.c`, `fe_felights.c`): Initialise 0x001FDC20
  "Player Info" Cash (0..1e6 step 1000) / Prestige Points (0..1e5, 100), then 0x00203450 "Frontend.FELights" (Ambient
  R/G/B + Light1..3 R/G/B 0..1 step 0.01f, Rotation 0..360 step 4, Elevation -90..90 step 1, Point Lit Car, WriteFile);
  CreateAllScreens 0x001FEB90 in order: FE3DCamera 0x0026FC40 ("Frontend" Enable empty environment; "Frontend.Camera"
  Enable Duration Change, Move Duration 0.35..0.65 step 0.05f, Free Roam, Output camera position, Change Up Axis;
  "...Debug Artist Cam" GarageCameraIndex enum CAM_OFF..CAM_FULL, "GarageCamera FOV " (trailing space) 0..90, Camera
  SIDE/FRONT/TOP/3D Pos X/Y/Z -30..30 step 0.1f), FEAlloyEditor_Basic 0x00258770 (Unlock ALL Vehicles, Enable Showroom
  level cars in customisation, Wheel change bounce time 0.1..3 / height 0.001..0.05 / oscillations 1..5),
  CarCustomisation 0x00247BE0 (Submission Sheet screenshots ALL / THIS CAR, "Frontend.Garage" Sell back multiplier
  0.01..1), MainMenu 0x0023B4A0 (Output car colour, Enable 2D, Enable Memory Output, "Frontend.Characters" Character
  Camera), CarShowroom 0x0021CB70 (Enable High Showroom Cars, Enable (Hidden) Showroom screenshots), Hangout 0x00218390
  (Display cheat screen, Unlock ALL Events). InGameInit 0x000B5EE0: Initialise + CreateCommonScreens (FE3DCamera only),
  COSDInGameScreen::Create 0x000F82F0 "Game.OSD" Split Time Display Duration (0..10 step 0.5, 2.0). SetGameMode
  0x000BBD30 frees the tree on every mode change except 1<->2 and adds "System" *after* the mode's init;
  ReturnToFrontEnd = SetGameMode(4) + FrontEndReInit + mode 0 directly (no second reset).
- Port: `fe_tweaks` (Update, before `run_frontend`) registers the mode's set whenever the tree lacks "Player Info"
  (`set_for_mode`: None/0/4 front end, 1/2 in game, 3 none). Wired variables (`Mirror::sync`: a menu edit wins, else the
  cell follows): Cash / Prestige = `gm.vars`; Unlock ALL Events / Vehicles = core's `DebugUnlocks` (+ FE
  `quick_race.all_unlocked`); Display cheat screen = `fe_hangout_screen::DISPLAY_CHEAT_SCREEN` (go_to_race takes the
  DummyGameScreen route for every event, 0x00219F8C); Enable 2D = `feck2_manager::ENABLE_2D` (was a const); Enable
  Duration Change / Move Duration = `fe_3d_camera::{ENABLE_DURATION_CHANGE, MOVE_DURATION_TWEAK}` via `move_duration`
  (0x00272020, at a vector move's start and on table moves); camera positions = `FoyerState.table[1..=4][0]`;
  GarageCameraIndex -> graphics' `FrontEndRender.garage` (`!= 0`). Cells that are their own variable (readers not
  ported): FELights (handler 0x00203B70 — Hex-Rays fails — writes them back into the character light set `dword_769E58`,
  graphics'), Point Lit Car (CarRender 0x00118D70), screenshot / submission flags, Output car colour and Enable High
  Showroom Cars (no reader in F3_D), Memory Output, Free Roam = Character Camera (one cell, both point at
  `byte_736554`), empty environment, output camera position, change up axis, FOV, sell back (kits not ported), wheel
  bounce (0x0025E7D0 not ported), Split Time (core's ScoreManager reads `dword_6860EC`; request in core.md).
- Callback 0x00202EA0 on `GameMessage::TweakChanged(full name)`: "Player Info.Prestige Points" ->
  `event_db.evaluate_unlocks(vars, false)` (0x0008F020) + `unhide_vehicles(prestige)` on `Career.vehicles` and the FE's
  `vehicles`; "System.Scan Track Folder" -> `set_all_unlocked(on)` (0x00094990) on `gm.event_db` and `fe.events`;
  "Frontend.Unlock ALL Events" on -> `unlock_all_events` (0x00092CD0 loop).
- Tests `fe_tweaks::tests::{frontend_tree_matches_the_original, in_game_set_and_mirror}`. Game run: core's
  `SMOKE_DIRECT=1 SMOKE_DEBUG_MENU=1` (exit 0, log `research/fe_tweaks_smoke.log`): the root now starts with "Game"
  (System sorts last, see core.md request), so the smoke's "A System" opens Game (rows OSD / exports); shot
  `research/screens/fe_debug_menu_tweaks.png` (core's `debug_menu.png` restored).
- **Debug menu title y (checked, not a mapping bug)**: `CGameMenu::BuildTitleBar` 0x000AADF0 makes the title with
  `CreateTextItem(.., layer 900, life -1, flags 0x1000, mask 0)` (CreateItem 0x000EDD00: x, y, layer, life, flags +4,
  mask) and `COSDManager::UpdateItem` 0x000EE5F0 *sets* y (`+40`, EF990) to 500.0 (0x43FA0000) for 0x1000 (0x400 -> 0,
  0x800 -> 1000; x: 0x80 0 / 0x100 1333 / 0x200 666). `COSDManager::Update` 0x000EE910 runs on the menu manager
  `dword_6FD1FC` every frame (GameManager::Update 0x000BB637 / 0x000BB7DC, InGameUpdate 0x000B7FA8), so the original
  title also sits at OSD y 500 (mid screen) over the rows: the port's `ANCHOR_CENTRE_Y` mapping is right, kept.
  Dumps `research/fe_osd_anchor.c`, `fe_osd_create_items.c`, `fe_osd_create_item.c`, `fe_osd_update.c`, `fe_osd_upd_xr.txt`.

#### Main menu background (3D foyer) — `foyer.rs`, `fe_3d_camera.rs`

- Behind the main menu the original draws the **`hangout` foyer** (`FE3DCamera::SetFoyer("hangout")` in Activate
  0x23C0D0), no movie: `Foyers\hangout\Model\hangout.xmf` + area model (`LA/London/Tokyo.xmf`) + `HANGOUTT.VCD`,
  the player car on `Vehicles.txt`'s spot, characters (`Chars/Crowd/Dance.txt`) and particles. Behind state 0
  ("press start") a black "white" 2D background fades out over 75. Every other FE screen keeps the foyer.
- FE camera = `CElementCamera::Load` 0x299770 → 0x299860: **vertical fov 66°**, aspect 4:3, near 1, far 300;
  `FE3DCamera::SetFOV` 0x272140 sets 50° in all quick race screens (never reset). `FoyerState.fov`.
- `FECK2Element3D::LoadFoyerLights` 0x298CD0 = foyer `Lights.txt` + `PtLights.txt` (GDL 0x9DE90) → inserted as
  `TrackLights` / `TrackPointLights` so the car is lit like the original (frontend car spec `frontend = true`).
- **Geometry vs lights:** FE3DCamera registers fixed backgrounds at construction (0x26F3D0: hangout, LA, London,
  Tokyo, modArea, carSel, hangCE, empty); `SetFoyer(name)` (0x272070) only reloads lights (LoadFoyerLights). So
  "quickR" (car select) keeps the hangout geometry under quickR's lights; quickR.xmf is never drawn. `foyer.rs`
  always builds hangout + area model and takes lights from `FoyerState.name`. `FoyerState.custom` = a direct
  position/look-at (`snap_to`, as character cameras do).
- **Vehicles** (`fe_3d_vehicles`, FE3DVehicles `dword_768978`): `Vehicles.txt` loader 0x2595E0 (spot kinds
  PLAYERCAR 1, PLAYER1CAR 2, PLAYER2CAR 3, CUSTOMCAR 4, CARSLOT 5, SHOWROOM_SINGLE 6, RANDOMCAR 7, NETWORK_CAR 8),
  `show` 0x259D20 (y += wheel `+0x0C`/2; PLAYERCAR = garage car, empty garage → first LA tier-1 car stand-in),
  `set_visible` 0x25E570, `set_colour` 0x25E210. `foyer.rs` spawns one `SpawnCarRender` per visible slot with
  `CarSpec::with_colour(CarColour::from_record(..))`, respawning when car / colour / spot changes. The main menu
  enter loads hangout's file once and reloads it after the car select repointed it.
- **Characters** (`fe_3d_characters`, FE3DCharacters 0x25F440..0x2624E0 + CElementCharacter 0x284A30..0x285910):
  Chars.txt (model = the `Texture` line, anims `<Texture>_STAND/EXIT/SELECT/SELECT2/IDLE1/2/3`, cameras,
  NameID, prop), Crowd.txt (slots 0,4,5,6), Dance.txt (0,4,5); counts honoured — shipped hangout has 5 chars
  and `CHARACTERS : (0)` crowd/dancers. Elements are created **hidden** (0x260710.., by the main menu's Enter
  0x23BED0 too) and shown only by FEHangoutScreen Start 0x219300 (hidden by its Leave 0x219920) and 0x2456B0 (a
  screen in the FENetworkServerDetails range, not ported; 0x22CB90 / 0x245C40 only hide). **So the main menu has no
  people in the original either** (checked: xrefs of 0x261510/0x2615F0/0x2616A0, `research/fe_scene_xr.txt`).
- **THE SCENE Start / Leave (1:1, 2026-11-06; dumps `research/fe_scene_dc2..6.c`)**: Start keeps the character
  (+276 is zeroed only at create; the old port reset it — Start's `v12[68] = 0` is +272, unknown), SetBackgroundVisible
  (1,1,0), camera on Leave's saved +340/+356 then ChangeCharacter(0), nav 3 GENERAL_RESULTS while CFrontendManager
  +113 (`Feck2.results_prompt`), overlay +36 (R_MENU_CONTENT border) hidden, FadeArrows(-128,1,0), loop event 210,
  FadeMusic(char volume, reverb, 1), menu closed -> move timer = +436. Leave saves the character's +464-kind camera
  (0x262380 -> +292/+308/+324, copies +340/+356/+372), overlay +534 off, exit screen set -> FadeMusic(1, 0.1, 1) and
  the characters hidden + props dropped (0x261300), else they stay shown; StopLoopEvent210. ChangeCharacter's
  previous == character copies +292.. into +340..; else FadeMusic + +340/+356 = previous LOOKAT. Music calls go to
  `Feck2.music` (`FeMusicCall`), dropped in `lib.rs` until audio adds the commands (request in audio.md).
- **Character props**: Enter (chars shown or just created) 0x2609D0 makes a "CharacterProp" 3D element per char
  whose `Prop` is not "NONE" (model `<Prop>.xmf` in `\Foyers\Props\Model\PROPSM.VCD`, textures PROPST.VCD), 0x25F2A0
  (labelled FE3DVehicles) sets its world = RotationY(record +272) + record position, Start's 0x273990 puts it on
  the camera's list (reflected layers), 0x261510 shows / hides it with the chars. Shipped: only Showroom's
  "Powerbook". `FE3DCharacters.props`, drawn by `foyer.rs::build_prop`. Test `hangout_props`; `fecapture scene` (career
  script into the hangout, right x2) -> `scene_hangout_showroom.png` = `research/screens/fe_scene_showroom_prop.png`
  (Showroom holding the Powerbook; log `research/fe_scene_capture.log`, exit 0). Note `fecapture career`'s shots from
  `career_tory_page2` on are the LoadingScreen (Tory's A starts the event), not the browser pages / quit dialog.
- Character anim state: current/previous/queued/default, time wraps → queued anim, else back to default (0x285350); idle
  pickers per FE frame: chars 0x261750 (timer 600, then 300..320; HANGOUT_EDITOR drags crowd #2 along), crowd
  0x2619B0 (0/3 with partners 1/4), dancers 0x261C70 (frames + 60). `foyer.rs` spawns `SpawnSkinnedModel`
  (CharPack::Chars), drives `SkinnedModelAnim { anim, time: Some(t) }` and the root Visibility. Anims load
  synchronously (state 3 at once); the previous-anim crossfade (1 s) is graphics' `SkinnedModelBlend`.
- **Black hangout background (FIXED 2026-11-06: graphics' `read_attr` now decodes 0x31; stand-in `=3` removed; shot
  `research/screens/fe_bg_menu_fixed.png`)**: hangout.xmf positions are
  D3DVSDT_NORMSHORT3 (0x31, stride 16: pos i16x3 +0, D3DCOLOR +6, uv NORMSHORT2 +10; the object matrices hold the
  de-quantisation scale + centre); visos_render's `read_attr` has no 0x31 arm, so every vertex sat at the origin and
  the 37 objects collapsed to points (67 meshes spawned and view-visible, zero-size boxes, camera inside the room at
  (0.25, 2.94, 6.61)). Request in graphics.md "Requests from frontend (black FE hangout background …)". Not a lighting
  issue: the original draws the background elements with no light set (FECK2ElementCamera 0x002999B0 →
  0x00299D00 → FuryInstance 0x0012A7A0 with the element's render struct +172, never given a set), so the prelit
  vertex colours (avg 0.19/0.20/0.16, alpha 0x80) pass as in our builder; CElementCamera::Load 0x00299770 sets the
  scene ambient to 0.4 but LoadFoyerLights overwrites it, and it only affects the car / characters. LA/London/Tokyo
  .xmf are FLOAT3 (not affected). Dev: `VELOCITY_FOYER_DEBUG=1` logs background mesh count / visibility / world
  bounds vs the camera once (`foyer::foyer_debug`), `=2` draws them full bright (texture alone), `=3` was a temporary
  control, now removed. Shots: before
  `research/screens/fe_bg_scene_before.png`; control `fe_bg_menu_normshort3.png`, `fe_bg_scene_normshort3.png`
  (logs `research/fe_bg_scene1.log`, `fe_bg_scene3.log`, `fe_bg_menu3.log`; dumps `research/fe_bg_dc*.c`).
- `fecapture` shots: scene_hangout_characters (characters forced visible + Tory's LOOKAT camera), scene_car_select_
  tier/car/colour_stage/swatches/swatch_colour, scene_car_select_spot (debug view of the PLAYER1CAR spot).
- Foyer particles: graphics' StatelessParticles (see below).
- **Loading screen tutorial labels (ported)**: Enter's tutorial branch (tip = `*(dword_6F9BE8 + 0x54)` =
  `CareerEventView.loading_tip` of the pending career event, else `CFELoadingScreen.tutorial_tip`, 0 for quick races)
  calls 0x00241670: tips 1/5/6/7 label actions 27 (steering) + 30 (accel/brake), tip 2 adds 39 (zone), 3 adds 38
  (nitrous), 4 adds 33 (handbrake). 0x00241840: the first `off_6C1850` input (A B X Y BLACK STICK_RY STICK_LX LEFT
  LTRIGGER RTRIGGER) equal to the input pad 0's mapping binds the action to (UserInput 0x001E6F80 = first type-2 node of
  the binding, InputManager 0x003D5720; here `fe_config_options_screen::action_button`, device handle check 0x001EA020
  taken as valid) gets the action's CONFIGCONTROL_* text (TipText i) and line i (0x00241940: ClearPoints, three points
  `dword_76A518[i] + (284, 196)`, shown). The point table (static init 0x00431080) is placeholder data in this build:
  input 0 (10,10)(100,10)(100,100), all others (10,10)(40,10)(40,40); the text offsets `dword_6C1800` are placeholders
  too (most (100,100)), so labels overlap (STEERING / ZONE MODE in the shot) as authored. Config1: tip 2 labels 3 inputs (test
  `capture_loading_tutorial_labels`, shot `research/screens/fe_loading_tutorial.png`).
- **2D line element (ported, `feck2_element_2d_line`)**: `CElement2DLine` 0xA8 B, vtable 0x0065A234 (Init slot 11
  0x00293410: +108 five points, +156 count, +160/+164 desc colour; AddPoint 0x00293590 asserts < 5, ClearPoints
  0x00293670, Load 0x00293690 = true, Render slot 12 0x002936B0 = GeomTool 0x003C6CF0(points, count, 1.0, colour)).
  GeomTool: per segment a quad 1.0 either side of the points (side = cross(dir, z)), mitred against the next segment's
  side line (plane 0x00143060, ray hit 0x002C3E10, |dot| < 1e-6 -> 0), texture `dword_7B5714` (drawn untextured), raw
  512-space positions (no aspect pre-scale). Test `mitred_corner`.

#### Garage (`feck2::screens::garage`)

- `GarageVars` = the garage part of `g_FEGameVariables`: cash 250000 (0x3D090), 300ZX / RX8 / NSX (SetInitialVariables
  0x000C1930), one `VehicleSetup` per slot (6, free slots a 350Z, 0x002022F0). `setup_parts_cost` = 0x00185140.
- PlayerGarageScreen: car on modArea's CARSLOT spot, USE / SELL with a yes/no dialog, both leave for HangoutScreen.
  Reached from PinkSlipScreen's VIEW CAR or `jumptoscreen PlayerGarageScreen` (starts on the current garage car).
- **PinkSlipScreen (ported 2026-11-07, `fe_pink_slip_screen`, dump `research/fe_pinkslip*.c`)**: hangout garage character ->
  GARAGE_PINKSLIPS. Enter: camera ups reset / no move, hangout Vehicles.txt, all cars shown; Start: hangout background,
  the garage character's SELECT camera at once (FE3DCharacters 0x00262380 `character_camera(1, 2, 0)`), fov 50,
  characters + props shown; Activate: overlay (select / back, cash, prestige, HANGOUT_GARAGE), list = GameVars 0x000C2400
  (owned setups) one entry per car (name = vehicle hash) with a 16x16 "white" swatch (`menu::add_colour_entry` =
  FE2DOverlay 0x0027E9D0: "EntryIcon" at the info column, gradient top/bottom = the body colour's D3DCOLORs, setup
  +188/+192), window 5, title GARAGE_PINKSLIPS; "usecar" (300,300) / "viewcar" right-aligned on 500, lit (0x80,0x60,
  0x60,0x20) / dark (0x80,0x20,0x20,0x20) by left/right. A: view -> FE3DVehicles garage slot (0x00246040), `SetCar`
  (0x00246010) on PlayerGarageScreen, transition, 20 frames; use -> `dword_6F71C4` / `dword_6F71C0`, back. B: back next
  update (overlay first activate, 0x0021C890). Leaving to another screen hides the characters (0x00245C40).
  `fecapture pinkslip` -> `pinkslip_*.png`. Steer 0x0025E040(0, 28.0) ported (state only, see "Front wheel steer").
- CarCustomisationScreen (`jumptoscreen CarCustomisationScreen` = `byte_6F905A`, backing out of level 1 stays):
  CUSTOMCAR spot, icon carousel levels 1 category -> 2 group -> 3 item -> (4 paint brand) -> 5 part list / swatches ->
  6 buy / sell-back dialog; browsing fits the part at once, cancel restores the setup copy, Leave writes the bought
  setup back to the slot; camera per part type (bodyparts by type, wheels 22, brakes/tyres 23, else 13; far orbit
  moves are snapped). Not ported: decals, gauges list, mix-your-own colours, the debug LOD/colour sliders,
  screenshots, team discounts, performance-kit price, wheel/tyre/brake models on the car.
- **Hangout -> garage (fixed 2026-11-05)**: GARAGE_CUSTOMISE (0x0021C010) sets `dword_6F90C4` = `dword_6F71C4` (slot) and
  `dword_6F90C8` = `dword_6F71C0` (vehicle) before CarCustomisationScreen (`menu_select`); the port left `custom_slot`
  at 0, so the screen always edited garage slot 0.
- **Current car (`dword_6F724C`, fixed)**: a 556-byte setup *copy* the race is built from (= core's
  `PlayerCarChoice`). Writers (xrefs `research/fe_xr724c.txt`): load 0x001044D0, StartEvent 0x0028F430 (team /
  opponent car), car select, and the hangout Select 0x00219CC0 — on a non-Tory character (0x00219DAB) and when Tory's
  browser opens (0x00219EC6) it copies FE3DVehicles +1824 (`foyer_vehicles.slots[0].setup`, the garage car on show).
  Ported as `CareerRequest::CopyShownCar` (`career_out`: `career::car_choice`). Without it a bought part only reached
  the race after a profile reload.
- **Bought part -> next race (fixed 2026-11-06)**: CarCustomisationScreen edits the shown car (FE3DVehicles +1824),
  buying keeps it in the copy +9951 (0x002520B0), and Leave 0x0024B340 writes the copy into the garage slot
  (GameVars 0x000C2040) — *after* the hangout's Enter (screen order Enter, PostEnter, old Leave, Activate). The
  hangout Activate 0x00219690 shows the cars again (0x00259D20) after that, so Tory's Select copies the bought setup;
  the port showed them only in Enter, so race 2 ran without the part until a profile reload. Now `ho_activate` calls
  `show_hangout_cars`. Not ported there: 0x0025E040(0, 28.0) (slot 0's turn target +1256).
- **End to end (SMOKE_CAREER=3, `research/fe_career3.log`, fresh `VELOCITY_SAVE_DIR`, `SMOKE_DRIVE=1100` so the
  smoke's 300 s limit does not cut it)**: event 1 (UT0EVENT1, tutorial) cash 250000 -> 250400; garage front bumper
  Style 2 buy 250400 -> 250200, slot 0 parts `0:20/2`, gm.vars updated on the way back (shot
  `research/screens/fe_garage_buy.png`); event 2 (56bc66f8) races with `0:20/2` (after the Activate fix), 251400;
  reload from the autosave keeps `0:20/2`; event 3 (cfb53742, locked until 56bc66f8 completed: unlock seen) 252600;
  completed 3, tiers [1,0,0], exit 0. Prizes 1000 / 3000 / 3000 came in as 400 / 1200 / 1200 (the AI-driven
  player's placing). Tory's after-race steps 1..8 ran each time (step 5 = her result line); no tier / vehicle /
  team unlocks in these events, so no invite dialog. `SetCompletedAny` stops the tutorial restarting (the run before
  it raced UT0EVENT1 twice). DummyGameScreen's Play Race (`ContinueEvent` -> `StartCurrentEvent`) is mapped but
  not exercised: the smoke takes the 1ST debug result.
- **Prestige from the race (results OSD ported 2026-11-06)**: `dword_6F71A0 += dword_6F7244` at RaceCompleteScreen
  Start (0x00205830, unless event +21 & 8; core does it in `race_complete`); `dword_6F7244` (= core's
  `gm.vars.event_state`) is set by the debug DummyGameScreen and by the race table below. See "Results OSD".

#### Results OSD (`osd::osd_screens::osd_race_over_stat_screen`, `COSDRaceOverStatScreen` `byte_6FD208`)
- Steps 0x000F35C0, run by GameManager 0x000BDFE0 (core's session PostRace, `RaceSession.post_race_step`): 0 =
  0x000F3130 (release table / items +16 +84 +88, page 0, flags 7, OSD mask `&= ~8`) + 0x000F3220 (the +20 text,
  font 10 at (666, 650): bss never written, so empty), 1 = 0x000F32B0 (select edge: UserInput 0x001E6BC0 action 6,
  player 0's device mask, > 0.5, flag 4 = held; flags 7 means a held A must be let go first), 2 = 0x000F3360 (text
  off, 0x000F2F40(2)), 3 = 0x000F3400 (select -> page + 1, 4 = done; a press while faded only restores alpha; after 10
  s (+12 += 0.033333335 per update, never reset by 0x000F3130: carries over a restart, original) alpha
  `clamp((15 - t) / 5, 0.1, 1)` via 0x000F0270 = `VISOS_Colour(ftol(128 a), r, g, b)` of each item). 0x000F2F40(page):
  1 TV summary 0x000F0450 (6 empty slots, flags 0x40|0x80), 2 race table 0x000F09C0 (flags |= 8), 3 0x000F2EC0 per
  kind (0x000F14B0 single / 0x000F26B0 cup / 0x000F1DA0 series; no event = null read in the original, no table here).
- **Page 3 (ported 2026-11-06)**, items as the race table (font 11, rows y 660 − 45 r, row 0 / title yellow, mask 8):
  single `OSD_STATSTITLE_STATISTICS`, 7 columns POSTIONS / NAME / OVERTAKEN (+72 "%d") / SECSINLEAD (+68 m:ss, no NA) /
  FASTESTLAP (+44, "NA" < 0.1) / TOPSPEED (`(int)(+0 * 3.6f * 0.625)` "%dmph") / GRIDPOSITION (+88 u8); x tables
  `dword_686078` / `dword_68605C` (0 95|50 340 475 650 860 1000, width [6]); justify {8,2,8,8,8,8,8}. Series
  `OSD_STATSTITLE_LEAGUE`, 4 columns POSTIONS / NAME / PRESTIGETOTAL (competitor +0x0C) / TOTALPOINTS (+8) over the
  first *racing-count* event competitors (+0x24), rank = 1 + others ahead (points, then +0x0C, then other human, then
  lower index; a human at a higher index than a level AI shares its rank, the row is overwritten — original), x tables
  `dword_6860A4` / `dword_686094` (0 100|75 375 550, width 550). Cup: title `OSD_STATSTITLE_RACE`, bracket rows in
  table +20/+28 (0x000F0790 rows, released by 0x000F0100, **not faded** by 0x000F0270): per round i <= current
  (+0x20 +2), `(total / per) / (i+1)` races (total = *(+0x24), per = +0x17; the row count uses `(total >> i) / per`,
  differs from round 2 on: original overflow, grown here), x = 300 (i+1), slot k y = j >= n/2 ? 600 − 45k − 200((j+1)>>1)
  : 45(per−k) + 600 − 200((j+1)>>1); shown = finish order (0x00091CC0(k+1)) when the round seats a human (0x00092B50)
  and j == 0, overridden at the current round by event +21 & 0x10; else slot competitors; null = string "test"; yellow
  when k < per/2 and i < current, or i == rounds − 1 && k == 0 && (shown || no human). Event +21 & 6: extra row at x
  300 (cur+2): `OSD_STATS_WINNERHEADING` (y 645, yellow) and the previous round's (+0x20 +0x10) first race winner (y 555).
  Names: competitor +4 = opponent record (name at +0), here `OpponentDatabase::get_opponent_by_crc` (loaded once by
  `hud::results_update`), "PLAYER" for +0x14. Inputs: `RaceResults.event_full` (core's `CEvent` clone),
  `competitor_names`, `RacerResult` +0/+44/+68/+72/+88 = ai `stats[0]/[11]/[17]/[18]/[22]&0xFF`. Test
  `page3_summaries_by_event_kind`; dumps `research/fe_osd_p3_*.c`, `fe_osd_p3_tabs.json`. Checked in game (drag smoke as
  below + `VELOCITY_RESULTS_SHOT_PAGE=3`, log `research/fe_results_page3.log`, exit 0): `research/screens/
  fe_results_page3.png` (Statistics: BUD / PLAYER / AL / JULIO, 129..121 mph, grid 1..4). Cup / series not seen in game.
- Race table 0x000F09C0: title `OSD_STATSTITLE_RACE` font 9 at (666, 800); rows = racing cars + 2, row 0 headers
  (`OSD_STATS_POSTIONS` / `_NAME` / `_TIME` / `_PRESTIGEWON`, [4] `_MONEYWON` when kind 1 and prize +0x30 != 0, else
  `_POINTSWON` kind 3, `_QUALIFIED_STATUS` kind 2; [5] `_WORLDRANK` online), row 1 empty, place p in row p + 1: "%d",
  "PLAYER" when p == `*GetPlayerVehicle(0)` else the driver name (0x000153B0), time = record +4 "%d:%d" / "%d:0%d"
  (`(int)(float)(t/60)`, `(int)t % 60`) or "NA" below 0.1, "%d" of +84, money = CompartmentManager 0x0008EA60(prize,
  **place 1-based**, competitors +0x24, category) (off by one: 1st gets 2nd's share, last 0 + warning — original),
  points 0x0008E9C0(place), qualified `p <= +0x17 / 2` -> `_QUALIFIED_YES/NO` as a string item in column 4. Items:
  font 11, y = 660 − 45 row, x = (1333 − `dword_686044`[columns]) / 2 + `dword_686044`[c] (strings) or `dword_68602C`[c]
  (texts: 0 50 300 480 690 860; header x 0 95 300 480 690 860), flags strings 0x2008, texts `{8,2,8,8,8,8}[c] |
  0x2000`, mask 8, layer 0, permanent; colour row 0 / title `dword_77FD84` = VISOS(0x80,0x80,0x80,0) yellow (static
  init 0x00431BD0), others VISOS(0xFF,0x80,0x80,0x80) white; offline the column-5 items get 0x4000 (0x000F37B0(0)).
  The player's +84 -> `dword_6F7244`.
- Glue `hud::results_update` (GameTick, before `InGameUpdateSet::UpdateSessions`, InGame, not paused): step =
  `post_race_step`, writes `RaceProgress.results_busy = !done` (core's 0x000BDFE0 switch reads it in the same update)
  and `gm.vars.event_state`. Inputs (`hud::race_results`): ai `race_order` / drivers / `stats[1]`, +84 by place = core's
  `RacerPoints::points_in_place`, event = `gm.current_event()`. Outside session PostRace a leftover table is released
  (stands in for the 0x000F3130 calls of RestartRace 0x000BE0F0 / EndOfGame 0x000BE540). Dev: `VELOCITY_RESULTS_AUTO=<s>`
  presses select after `s` s of waiting, `VELOCITY_RESULTS_SHOT=<png>` saves one shot 1 s into page
  `VELOCITY_RESULTS_SHOT_PAGE` (default 2).
- Checked in game: `SMOKE_REVIEW=1 SMOKE_RACE_TYPE=5 SMOKE_RACE=1` + `VELOCITY_RESULTS_AUTO=2` (LADown drag 01, exit 0,
  log `research/fe_results_smoke.log`): Finished -> PostRace -> table -> "End of Game UPDATE" -> drag round 2. Shot
  `research/screens/fe_results_table.png` (Race Stats: BUD / PLAYER / AL / JULIO, 0:41..0:48, prestige 4123 / 2877 /
  1525 / 745; a quick race has no 5th column). The London sprint (`SMOKE_AREA=2 SMOKE_RACE_TYPE=2`) did not finish
  this session: the ai-driven player stood still from ~240 s race time (not frontend).
- Consequence: the race-over menu (mode 2) now comes only after three selects (results, table, page 3); smoke scripts
  must press A in session PostRace (request in core.md). Test `race_table_columns_and_steps`. Dumps
  `research/fe_osd_f09c0.txt`, `fe_osd_F35C0.c`, `fe_osd_f2f40.c`, `fe_osd_F3360.c`, `fe_osd_F3400.c`, `fe_gm_BDFE0.c`,
  `fe_res_funcs.txt` (object functions + callers), `fe_results_tabs.txt` (column tables).
- `fecapture garage` / `fecapture custom` capture both (`garage_*.png`, `garage_custom_*.png`); reference shot
  `research/screens/fe_garage.png`. Checked: garage title / car name / use + sell texts at (300,300) / (500,300),
  cash 250,000; custom: front bumper Style 2 buy dialog (cost 200), after YES cash 249,800 and the part on the car.

#### Overlay menu (`fe_2d_overlay_menu.rs`, FE2DOverlay 0x27D220..0x280EC0)

Screens list their items on the overlay: `sMenuEntry` (44 B) list +392, count +408, selection +412, border slide
+416, shown +420, horizontal +421, window +424 (-1 = entries stay where created), style +428 (0 bottom, 1→2 right,
3 centred, 5 icon carousel), font +432. Metrics 0x27D860, layout 0x27F840 (colours: selected orange / dark,
unselected grey / (32,32,32)), carousel 0x27FD80 (icons 90 apart, slide/fade 25 frames, 100/40/30/0 %). Layer
functors: `COMMONMENUOVERLAY` 0x276C70 moves the selection (sound Up for both directions), select plays Accept or
Error and swallows a disabled entry; `COMMON2DOVERLAY` 0x276F10 plays the cursor/back/accept sounds (+534/+535/+536
mute them), Start → `OptionsScreen` unless options are hidden. Sounds go to `Feck2.sounds` → `AudioCommand::Gui`.
CFrontendManager fix: `GetPreviousScreenCRC` = `history[level-1]` (`history` is `dword_6F6F70`).

#### Options screens (`feck2::screens::options`, code 0x230AC0..0x23A4D0, no `__FILE__`)

- `OptionsScreen`: Profile, Language, Video, Sound, Controller config, Tutorials, Tweak (style 1, window 5).
  Tutorials → `TipsScreen` (0x0023A2CB: overlay `first_activate`, tip -1), Tweak → in-game tweak menu: **not ported**.
- `TipsScreen` (`CFETipsScreen` 0x1C B: +0 screen, +4 layer, +8 two "LoadingTipInfo" texts font 0 at (30,75) / (30,368)
  `dword_6C135C/64`, +16 tip (-1 list), +20 loaded, +22 go back, +24 countdown). Activate = options look (title
  GENERAL_OPTIONS, menu title OPTION_TUTORIALS, style 1, window 5, 2D background "loadingBackground" LOADSCR.vcd).
  Start: tip -1 -> list of tips 1..8 (`dword_75C670` = loading screen TUTORIAL_TITLES), entry enabled = movie seen
  (0x00240310, `dword_6F71D8` bit tip-1 = `CareerView.tips_watched`, saved as core's `unknown_264[1]`); a set tip (the
  hangout's after-race step 3, 0x0028FD80 -> 0x0023F1F0) seen -> straight back, else its text, 5 frames, movie, back
  (`GotoPreviousScreen(1,0)` + `first_activate`). Select (list) = text + movie after 5 frames (always), back to the list
  (0x00240020). Text (0x0023FE80): title = tip title, lines `GAMETIPS_<dword_688C90>_INFO_VIDEO_<1|2>` box 460x100, menu
  hidden, menu header `Fade(-128,0,0)` (element vtable +36 = Fade, vtable 0x0065A110). Movie (0x002400D0):
  `BE9F0("TIPS\" + dword_688CB4[tip])` (null, null, zone, nos, drift, slip, drag, mgears, tblazer — no tblazer.bik
  shipped), then the seen bit. Data bug kept: `dword_688C90` lacks two commas ("SLIPSTREAM" "DRAG" "MANUALGEARS" =
  one string, TRAILBLAZER at 6, 7/8 null = strcat(NULL) crash in the original, "" here), so SLIPSTREAM reads NOT FOUND
  and DRAG shows TRAILBLAZER's lines. Decomp strings dropped by Hex-Rays (strcpy source): check the disassembly
  (`GAMETIPS_`, `TIPS\`, `CUTS\`). Test `capture_tips_flow` (`tips_list/text/back.png`, `research/screens/fe_tips.png`).
- `ProfileOptionsScreen` (`dword_765C30`, 0x24 B): menu `dword_765EC8` (OPTION_PROFILE_EDIT_PLATE / _INFO / SAVE /
  LOAD / NEWPROFILE / DELETEPROFILE, style 1, title element +344 = OPTION_PROFILE), info elements "profileProgress"
  (font 1, x 120, y 150 + 25 i, `dword_765EB8` PROFILE_INFO_CASH / PRESTIGE / PINKSLIPS / PROGRESSION; dCash
  `dword_6F719C`, dPrestige `dword_6F71A0`, dPinkSlips 0, dPercent as the profile select's). Select (0x00235BA0):
  info shown -> back to the menu; dialog up -> row 4 and SaveGame_Yes (selection 0) -> ProfileSelect mode 5, then
  hide; else 0 PlateCustomizationScreen, 1 info, 2 save (mode 2), 3 load (mode 1), 4 dialog
  SaveGame_New_Confirm [SaveGame_Yes, SaveGame_No], 5 delete (mode 4) — the modes via 0x00232D40 on the profile select
  object (`CFEProfileOptionsScreen.profile_select`). Cancel closes the info and goes back either way.
- `PlateCustomizationScreen` (`CFEPlateCustomizationScreen` `dword_766960`, 0x20 B: +0 screen, +4 layer, +20 loaded,
  +24 stage, +28 style; IDA dump `research/fe_plate.c`): layer prio 4; events PRELOAD / LEAVE / EXIT nothing, Enter
  0x0023ABA0 (attach + camera), Activate 0x0023AC10 (options look, SELECT / BACK, title CUSTOMIZE_PLATE_HEADING,
  borders, style 1, enable; it maps the Update functor again), Start 0x0023ADD0 stage 0, Update 0x0023B0D0 (keyboard
  done && visible: text -> `unk_6F71E0`, hide, GotoPrevious(1,0)), Select 0x0023B170 (falls through to the keyboard
  while it is up and not done, else stage + 1), Cancel 0x0023B1D0 stage - 1. Stage step 0x0023AE10: -1 GotoPrevious;
  0 keyboard off + style menu (0x0023AEF0: `dword_766BE8` = OPTION_PLATE_STYLE_AMERICAN / BRITISH / JAPANESE, static
  init 0x0042DE90, selection `dword_6F71DC`); 1 style = menu selection -> `dword_6F71DC`, menu off, keyboard
  (0x0023AFA0: show, +74 case locked, max 8, SetString(plate text)); 2 both off, GotoPrevious. The keyboard's
  clear-on-first applies: the first key on the pre-filled text clears it (a cursor move first keeps it).
  `Feck2.plate` (`PlateVars`, default 0 / "OCTANE") is mirrored by `lib.rs::plate_sync` with graphics'
  `LicencePlate` (style / text the plate render draws) and writes the style to core's `vars.unknown_264[2]`
  (GamePlayer +0x26C). A new profile now goes ProfileSelect -> PlateCustomizationScreen -> back (Start +44 saves).
  Test `capture_plate_flow` (`plate_style/keyboard/back.png`: BRITISH, "OCTAN").
- fecapture runs without game flow: `hud::osd_flags_in` takes `Messages<RaceSessionMessage>` as optional (a plain
  `MessageReader` panicked "Message not initialized" there and broke every fecapture mode).
- `fecapture options` = every options screen read-only (`options*.png`; `research/screens/fe_options.png`); checked:
  the list (5-row window), video / audio / controller / language pages, profile menu, info page (250000 / 0 / 0 /
  0 %) and the new-profile dialog.
- `VideoOptionsScreen`: Screen adjust (no handler), Ratio (`byte_6F915D`, 4:3 / 16:9: the 2D renderer reads it,
  `game_options::widescreen()`), TFT display (`byte_6F915E`, trash talk), Player names (`byte_6F915F`); left/right.
  Port 2026-11-09: the Ratio row is removed (UI widens automatically); `widescreen()` always false. PC settings
  page: see frontend.md "PC SETTINGS".
- `AudioOptionsScreen`: Type and SFX are **disabled in the original**; Music opens the soundtrack picker
  (`dword_6F9160`, -1 RANDOM) whose playlist accessors are stubs in F3_D → only RANDOM. Volumes are edited in the
  in-race audio menu (pause menu), which shares the same options.
- `ConfigOptionsScreen`: controller picture + 10 labels (action → `CONFIGCONTROL_*`, table 0x688B98), left/right
  through `Config<n>.ini` (only Config1 ships → "Default" / A).
- `LanguageOptionsScreen`: flag carousel (FEFlags.vcd flags are placeholder white boxes; `Language.sbk` is not
  shipped so names/loading texts read "NOT LOADED" — both as in the original), select → `ChangeLanguage` 0x2EBD70.
- **Options = `game_options::GameOptions`** (bevy resource, defaults from SetInitialVariables 0x0C1930; also
  `Feck2.options` + `options_changed`, and the pause menu's `CGameMenuManager.options`). On change the plugin sends
  `AudioCommand::Settings`, applies the globals and saves `%APPDATA%\Velocity\options.ini` (language, controller
  mapping per pad, volumes, audio mode, rumble, display toggles, soundtrack); loaded at startup.
- Hangout music calls: `Feck2.music` (`FeMusicCall`) now drains in lib.rs into `AudioCommand::{FadeMusic, StartLoopEvent210,
  StopLoopEvent210}` (audio, 2026-11-06); `fecapture` adds `AudioPlugin` + FE `Init` so `fecapture scene` logs them.
- Headless 3D captures: `cargo run -p velocity_frontend --bin fecapture` → `target/fe_captures/scene_*.png`.
- IDB names for these functions: `tools/ida/fe_opt_ren.py` (not applied yet; the worker was down).

#### Bink movies (`bink`, `vs_movie`, `movie.rs`) — RAD Bink 1 ported from the XBE's `BINK` section

All 125 `Movies\PAL\**\*.bik` are `BIKi`: In001..3 512x512 25 fps + 44100 stereo DCT audio, Tips 512x512, TFT / Cuts
64x64 12 fps + 22050 / 11025 mono. Test `bink::tests::decode_every_bik`: every frame of every file decodes, each plane
ends within one word of its declared size (luma) / the packet end (chroma), audio length = duration within a block
(11750 frames, 0 inconsistent); dumps `target/fe_captures/bink_in003_250.png` / `bink_in003.wav`.
- **VsMovie -> Bink entry points** (VsMovieStreamXBOX.cpp, IDA dumps `research/fe_vsmovie.c`, `research/fe_bink.c`):
  BinkOpen 0x0053E9A0 (open 0x002F3CC0: `D:\Movies\PAL\%s.bik`, flags 0 / 0x84000 sound / 0xA200000), BinkWait 0x0053FFB0,
  BinkDoFrame 0x00541340, BinkNextFrame 0x00541900, BinkCopyToBuffer 0x00541300 (-> 0x0053F970), BinkGetRealtime
  0x005404A0, BinkGetSummary 0x005402D0, BinkSetVolume 0x00540D30, BinkClose 0x00541C60, BinkSoundUseDirectSound
  0x0053E690, BinkSetMemory 0x0053E670, BinkGetError 0x0053E150. Bink struct: +0 w, +4 h, +8 frames, +12 frame num
  (1-based), +16 last frame, +20/+24 fps, +28 read error, +32 decode flags, +36 header flags, +184 planes (120 B,
  0x0053E8D0), +208 tracks, +228 frame data, +236 index, +768 bundle buffers (0x00545D80 sizes).
- **Update 0x002F3720** (mode +84: 1 BINKSURFACE32 (3), 2 full screen YUY2 (13), 3 texture +96): `!BinkWait && (state != 4
  || !shown)` -> NextFrame (after the first), DoFrame, copy; ends when `LastFrameNum == Frames` unless loop (+108).
- **Container** (`bink_file`): header `{sig, size-8, frames, largest, index count, w, h, fps num/den, flags, tracks}`,
  per track max size / `rate | flags<<16` (0x10000000 DCT, 0x20000000 stereo; DCT = flags >= 0 && bit 28) / id, index
  (bit 0 key). Decode flags: open & 0x8FFFFFFF | header 0x20000 (gray), alpha 0x100000 only if opened with it (VsMovie
  never does: alpha planes are skipped), BIKf/g |= 0x18000, BIKh |= 0x8000 (bit 15 = no plane size prefix + signed
  colours 0x00544E00; 0x10000 = swapped chroma). Packet: per track `u32 size` + data, then video. One frame buffer when
  every frame is a key frame (0x0053E6E0), else two (swapped per frame, 0x00547B60).
- **Video** (`bink_video`, plane decoder 0x00545E20): planes w/h rounded to 8, chroma half; bit reader = LSB-first
  u32 words. Per plane: 9 bundles (block types, sub types, colours, patterns, x / y offset, intra DC, inter DC, runs;
  count width `floor(log2(n+511))+1`, cap 15), trees read in the order types, sub, 16 colour context trees + colour,
  pattern, x, y, runs (0x00544450: 4-bit table index, then merge-sort permutation 0x005443E0 or an explicit list); per
  8-row strip each bundle refills only when used up (count 0 = finished). Blocks 0 skip, 1 scaled 16x16 (sub 3 run, 5
  intra, 6 fill, 8 pattern, 9 raw; odd strips skip it), 2 motion, 3 run (scan `unk_7D5188`), 4 residue (7-bit mask
  budget, 0x0054F780), 5 intra (0x0054EF00 coefficients + 4-bit quant, `dword_7D3100`), 6 fill, 7 inter (`unk_7D4120`), 8
  pattern (`dword_7D2F50/2F10` masks), 9 raw. IDCT (0x0054E3C0/E6F0/EB20) = `coef*q>>11`, columns, rows, `(u16)(x+127)>>8`
  (wraps, no clamp). Coefficient / residue internal order -> natural: `BINK_ORDER`. The dirty-macroblock map (a7/a8,
  for the blitter) is not kept. Out-of-plane motion reads give 0 (original: adjacent memory).
- **Tables** (`bink::tables`) are read from `default.xbe` at runtime (BINKDATA / BINKBSS are initialised in the image).
- **Audio** (`bink_audio`): open 0x00543280 (frame 512 / 1024 / 2048 by rate < 22050 / < 44100, bands from
  `dword_7D2698`, scale 2/sqrt(n)), block 0x00542E80 (DCT: skip 2 bits; per channel two 29-bit floats, 8-bit band
  quants `dword_7D2700`, coefficients 0x00542BA0 with zero runs `byte_7D2688`), transform = Ooura fftsg `ddct(n, 1)`
  (0x0054E0B0) — ported as an f64 FFT DCT-III (same values up to rounding, not Ooura's butterflies); s16 clamp
  (0x005429F0 / 0x00542AC0, truncation assumed); 0x005434B0 crossfade of n/16 bytes with an **unsigned** `div` (as the
  binary; acts as floor division). RDFT tracks (0x0054DF70) not ported (none shipped).
- **YUV->RGB (ported, `bink_video::yuv_to_rgb_mmx`)**: BinkCopyToBuffer 0x00541300 -> 0x0053F970 -> rows 0x00548DB0
  (setup 0x00548980) with the row blitter `dword_7D2A0C`, picked by 0x00549570 from the six selectors at `off_7D2A60`
  (0x00352360.. — Bink code linked outside the BINK section). Format 3/5 (BINKSURFACE32, VsMovie mode 1) = 0x00352260:
  shifts 0x005484B0(24, 16, 8, 0) = X8R8G8B8, row fn BINK32 0x00550FC0, biases R 0x61D0 / G 0x78A7 / B 0x5E8D equal on
  both rows (no dither). Per pixel: `y' = (Y<<6)*0x4A89>>16`, `u' = (Cb<<4)*0x6810>>16`, `v' = (Cr<<5)*0x6810>>16`;
  R = 2v'+y', G = y'-(u'+v'), B = 5u'+y', each `paddsw` bias, `psubusw 0x7000`, `>>4` (constants BINKDATA
  0x007D2A30/38/40). One chroma sample per pixel pair, chroma row y>>1; gray (0x20000) = chroma 0x80; the X byte is 0
  (`qword_7D2A48`, BINKDATA, only ever read by punpck*), and the port now writes 0 too (`to_rgba(out, x)`). Dumps
  `research/fe_yuv_blit.txt`, `fe_yuv_sel.txt`, `fe_yuv_consts.txt`.
  Full-screen mode 2 copies YUY2 (format 13, selector 0x00352050) and the GPU converts; the port uses the BINK32 maths
  with X = 255 there (STAND-IN, opaque like the YUY2 result). Texture mode 3 (checked): 0x002F3E80 takes the texture's
  D3DFORMAT through 0x002F40B0 (2/16 -> 8, 3/28 -> 9, 4/29 -> 7, 5/17 -> 10, 6/18 -> 5, 7/30 -> 3, 36 -> 13, 37 -> 14,
  else assert + 3; `bink_surface_type`); the TFT texture "movie_test" is Registered (0x0029F2E0, format straight into
  CreateTexture) as 18 = `D3DFMT_LIN_A8R8G8B8` -> surface 5 (32A), which selector 0x00352260 handles like 3: X byte 0.
  The LCD_Movie draw is blend 1 (no blend), so the 0 alpha is never seen. Test `mmx_yuv_blitter`.
- **STAND-INS / not ported**: BinkWait (audio-synced in RAD) -> real clock at
  `frame / fps`; Bink's IO / memory / sound threads; the scaling flags 0x10000000..0x50000000 (none shipped).
- **Wiring**: `vs_movie::vs_movie_stream_xbox::VsMovieStream` (queue 0x002F4C90, mode 0x002F4BD0, open, update);
  `movie::MoviePlugin`: `MovieCommand::Play(names)` / `Stop`, `MovieFinished`, full-screen ImageNode on camera order 12;
  `MoviePlayer.playing` pauses `run_frontend`; `Feck2.movies` (GameManager__BE9F0 calls) is drained into Play — the
  attract start (0x000B3620 In001..3) fills it. Skip = the movie game mode (GameManager__BE9F0 -> 0x000C0020, vtable
  0x00631964: +0 dtor 0x000C0370, +4 Update 0x000C01A0): per game update UserInput::Update, then MenuSelect (6) or Pause
  (13) `0x001E6BC0 != 0 && 0x001E6DF0` with mask 0xFFFF -> VsMovieStream 0x002F4B90 (state +80: 1 -> 0, else 2 =
  close at the next update, the queue goes on) — `movie::movie_skip` on GameTick with core's `UserInput` (pad A / Start,
  keyboard Space / Enter through core's KEYBOARD_KEYS; B no longer skips; no skip without core's UserInput). Base class
  0x0063196C's Update 0x000C0110 is empty; 0x000C02E0 (vtable 0x00631974 slot) is a variant testing action 6 only.
  Sound: `MovieAudioOut` for the mixer (request in docs/audio.md). Dev: `VELOCITY_MOVIE=In003` plays at boot,
  `VELOCITY_MOVIE_EXIT=<s>` quits after them or after `s` s. Checked: In001 / In002 play 2.64 / 2.21 s (67 / 56 frames at
  25 fps); shot `research/screens/fe_movie.png` (In003), clip `research/video/fe_movie.mp4` (`--smoke-test` +
  `SMOKE_RECORD` + `VELOCITY_MOVIE=In001,In002,In003 VELOCITY_MOVIE_EXIT=35`; no movie sound until audio drains
  `MovieAudioOut`).
- **Requests from audio (movie sound, done on audio's side)**: `MovieAudioOut` now lives in velocity_audio
  (`bink_sound.rs`, re-exported here); audio made the minimal movie.rs edits: `aout.open(rate, ch)` per opened movie,
  `aout.close()` in finish, and on `MovieCommand::Play` (= GameManager__BE9F0) `AudioCommand::StopMusicEvent` +
  `aout.volume = 32768` (BFFF0(1.0)). Please keep those calls when you rework movie.rs (TFT mode 3: volume
  `flt_6F9174 * 32768`, no StopMusicEvent; mode 1 opens with BinkSetSoundTrack(0) = no sound, don't `open`). BinkWait
  is not audio-synced in this build: it runs on the ms clock 0x005428A0 (docs/audio.md "Bink movie sound"), so the
  real-clock `frame / fps` stand-in is the original behaviour, not a stand-in. `VELOCITY_MOVIE` at boot sends the
  music stop before the FE starts its music, so FE music plays under dev-boot movies.
- **BE9F0 callers (all wired)**: attract 0x000B3620 (In001..3), FETipsScreen 0x002400D0 (`TIPS\<name>`), FEHangoutScreen
  0x0021C430 and FEHangoutProgression 0x0028FBD0 (`CUTS\<off_685400>`); all via `Feck2.movies`. VsMovie opens
  `\Movies\PAL\<name>.bik` (0x002F3CC0, dir +48 "\Movies"), the shipped files sit in `Tips\` / `Cuts\` / `TFT\<who>\`.
  Every BE9F0 movie sets `looping = false` (0x002F4AD0(2, 0)). Looping (0x002F4AD0 +108) now wraps when the frame
  after the last is due, so FrameNum == Frames is visible for one frame time (`frame_counts`, VsMovie +112 / +116).
- **Coverage audit (`research/movie_coverage.txt`, from IDA function lists `movie_names.tsv`; origin-line match =
  a `0xADDR` citation inside the function)**: 73/269 functions carry their own citation (VsMovieStreamXBOX.cpp region
  0x2F3160..0x2F6710 = 129 funcs, 12 cited; BINK sections 140, 61 cited). The 196 uncited (36.7 KB) split: 117 are
  reached only through cited functions (small VsMovie ctor/getters/STL list helpers, BinkOpen/IO/thread subcalls,
  folded into the Rust equivalents); 42 have only uncited callers; 37 have no code caller (vtable / dispatch-table
  entries): BINKY16 (5), BINK16 (4), BINK32 alt blitters 0x551370 / 0x551420, the 16 bpp and 8-bit surface row
  blitters, and 0x5438F0..0x543DC0 (audio variants picked by table). Nothing missing is reachable with the shipped
  data: VsMovie only creates BINKSURFACE32 / YUY2 / 32A, and every shipped .bik is `BIKi` DCT audio. Possible
  exceptions (uncited, not proven dead): 0x548630 / 0x5488E0 (callers uncited) and the 0x543860 audio helper.
  Playing all 125 `.bik` in one headless run (`VELOCITY_MOVIE=<all names>`, ~14 min real time) opened and played
  every file, 0 BinkOpen failures, 0 asserts.
- **In-race TFT movies (wired, not drawn)**: the player is OSD3dInterface (`byte_6FC098`; idb names
  `VsAIPlayerTFTManager__17220..172D0`, `COSDMapRender__E9C90/EA160`): +2860 texture "movie_test" 64x64 (Register
  0x000E9C90, shader "LCD_Movie" +2856), +2864 name, +2896 wanted, +2904 render frames, +2908 vcd. The ai's
  `play_line` (0x00016940) = `velocity_ai::plugin::AiTftPlayer`, inserted by `MoviePlugin` (`TftOut` on the shared
  `TftLink`). `movie::run_tft` = 0x000EA160 per render frame while the HUD is active: wanted + a movie open -> after
  10 frames and FrameNum >= Frames stop (0x000BEFD0) and unwant; wanted + none -> `MovieCommand::PlayTexture` (=
  0x000BEBB0 LABEL_17: stop, volume `flt_6F9174` (speech, `volumes[4]`) * 32768, queue, mode 3, texture, loop 1; no
  music stop; the vcd FindFile / async preload 0x002F5E60 + 0x000BEB90 / 0x000B9D09 replay is IO and skipped);
  unwanted + open -> stop. "Open" = 0x000BF030 `unk_6F5BC0 != 0` = `MoviePlayer.texture_active` (set by the play even when
  BinkOpen fails, so a missing line, e.g. the ai's `TFT/BUD/USA_Male_2_anim1_B`, is dropped after 11 frames with one
  BinkOpen warning). With the HUD inactive a running TFT movie is stopped (GameManager 0x000B8550 / 0x000BC010 call
  0x000BEFD0 at race shutdown; done here instead of by core). Output `movie::TftMovieTexture { image, visible }` (`visible` = 0x000EAD80's test: wanted
  and >= 2 frames; its other test `+2900 < -1.3` and the draw on the 3D hud's LCD model (`v5[634]` Fury instance,
  OSDM.vcd) are not ported — the 3D analogue huds are not drawn). 0x000EAD80 also rewrites the name to "zach" and
  wants again on every draw (debug leftover; harmless until the next line sets the name, not ported).
  The texture is now drawn on `tft_screen` (see "3D hud"); `TftMovieTexture.wanted` = +2896 for the flip.
- The movie screen's Activate (0x0023EDF0) calls 0x000B4D30 = `GameManager::InitDatabases`, not a movie player
  (`FrontendRequest::PlayIntroMovies` is misnamed; kept for core's bridge).

Captures: `cargo test -p velocity_frontend capture` writes `target/fe_captures/*.png` from the headless driver
(`frontend::Frontend`): fonts, legal pages, main menu, quick_race_text, car_select_tier/car, quick_race_start,
osd_in_game, race_hud(_paused), loading_screen, pause_menu / pause_audio / pause_quit, options(_video/_audio/_controller/_language).

**Foyer particles (done; was graphics' request)**: `foyer.rs` writes `LoadParticles { area: "\Foyers\hangout\",
sub: None }` when it builds background 0 (the hangout), i.e. once per front end session; core unloads at race start.

**Return from race = FrontEndReInit (0x000B5A80):** the original runs FrontEndInit, `CFrontendManager::Initialise`
and `CreateAllScreens` again before `Start(m_dReturnFrontEndScreen)`, so every screen and the common objects
(overlay, FE3DCamera, FE3DVehicles, FE3DCharacters — all created by CreateAllScreens / CreateCommonScreens) are
new. `Frontend::reinit` (`Feck2::reset_screens` + `create_all_screens`) does the same on `ReturnFromRace`; history,
post race / return screens and the game variables survive. Without it QuickRaceStartScreen came back with its
panel still faded out by its own Select (`layer_fade_all(-5)`; Enter only loads once, +1096). The start screen's
Enter now ports its FE3DCamera part (snap, "quickR", fov 50, re-load the Vehicles spots when none are loaded, move to
camera entry 10 = `dword_769280` / `dword_769370`); not ported: mode-2 timing and the 5 s move duration
(`0x00272110`, also 5.0 in car select / text screen, 1.0 main menu — `MOVE_DURATION` 0.65 is used everywhere).

**Keyboard (checked):** `FE\KEYMAP.INI` and `Input\Xboxpad\Config1.ini` name Xbox pad buttons only (PAUSE = START,
MenuSelect = DIGITAL(A)); the original has no keyboard mapping. The PC stand-in is core's
`velocity_game::controller::user_input::KEYBOARD_KEYS` (Enter = START, Space = A, Escape = BACK, Backspace = B), so
in race Enter pauses and Space selects by that table, not by the original. The frontend's old `lib.rs::raw_value`
(Enter = A, Space = Start) is dead since `step` reads visos.

#### Career ("The Scene") — `feck2::screens::hangout`, profile select

- **Glue (`lib.rs`)**: `career_in` fills `Feck2.career: CareerView` every frame from core's `GameManager` (database
  events as `CareerEventView {crc, category, kind, tier, flags}`, `vars.event`, `completed_any`, `profile_active`)
  and this frame's `CareerMessage`s (own `MessageCursor`); `career_out` turns the screens' `CareerRequest`s into
  `GameFlowRequest::{NewProfile, LoadProfile, SaveProfile, DeleteProfile, StartCareerEvent}` and does
  `ClearEvent` (`gm.vars.event = None`, progression step 8) itself. Without game flow (fecapture) requests are
  logged and dropped.
- **ProfileSelectScreen** (`CFEProfileSelectScreen` 0x3C B): `+48` mode 0 load / 1 load (options) / 2 save / 3
  delete (picked from load) / 4 delete / 5 new; `+52` state 0 listing, 1 list up, 2 keyboard, 4/6 saving, 5 loading,
  7 deleting, 8/9 leave (8 GotoPrevious(1,0), 9 GotoPrevious(1, MainMenuScreen); a new profile goes to
  PlateCustomizationScreen). The XMem status poll (0x002337C0: 3 done, -1 failed) is replaced by core's
  `Loaded/Saved/Deleted{ok}`. List (0x00234240): valid profiles (version "Version_25"; invalid ones counted, warning
  message), `PROFILE_EMPTY` up to 5, `OPTION_DELETEPROFILE` in load mode. Details (0x002348E0) from the save:
  cash +0x1EA0, prestige +0x1EA4, "pink slips" = cars +0x0004, progression `100 * (completed / events)` with
  integer division (0 until everything is done; then 1 for any progress — original bug kept). Cancel goes back only
  when `history_level` (dword_6F6FC0) != 0. A new profile is named on the keyboard (Select on an empty slot: show +
  `SetString(unk_6F90DC)` (empty, core keeps no name), state 2; Update takes the name when DONE was pressed, Cancel
  hides it and returns to state 1; Start hands it the name check 0x00234BF0 = `valid_profile_name`, Leave clears it).
  A new profile goes to PlateCustomizationScreen; its return through Start +44 saves it. Not ported: XMem popups
  (+40), `byte_6F9059`.
- **On-screen keyboard** (`fe_keyboard_overlay`, `CFEKeyboardOverlay` 0x74 B, ctor 0x00268560(screen 0, max 16,
  clear-on-first 1, accents 0, x 112, y 158)): three layers outside any screen — backing (prio 5), keys, top (prio 1,
  all event functors 0x00269990) — run by `CScreen::ProcessEvent` after the overlay's menu layer (0x00283D5F: top,
  keys, backing while +5) and drawn after the screen's layers (0x00283F98 while +4); `Feck2.keyboard` holds the
  object and those flags. Rows: [accents 84, 1..0], [caps 84, A..J], [shift 84, K..T], [symbols 84, U..Z,
  backspace 84], [left 42, right 42, space 126, DONE 84]; pages 0x0026D9E0 / 0x0026DDE0 / 0x0026E1F0 recode the
  keys (labels stay — original), key sprite = `VKeyboardCharacters` (pressed caps/shift: `...Depressed`), colours
  static init 0x0042F470.. (`visos_colour`). Cursor starts on DONE (disabled until the name is non-blank and the
  screen's check passes; disabled keys are skipped by the moves 0x0026BB50/0x0026BD00/0x0026BE80/0x0026BF00).
  Shift is on for the first letter; X backspace, Y space, Start DONE, L1/R1 cursor, R2 next page; held d-pad repeats
  after 0.333 s then every 0.085 s (frame delta). The `_` cursor blinks on the game update count (`(FM+8)+24`, here
  `floor(stopwatch * 24)`): shown while `fmod(n, 19) < 14.25`. Codes > 255 (€, Ÿ) type a space (original).
- **HangoutScreen** (`CFEHangoutScreen`): characters 0 Tory (events), 1 garage (`GARAGE_CUSTOMISE` /
  `GARAGE_PINKSLIPS` / `EDITOR_ALLOYEDITOR` -> CarCustomisation / PinkSlip / AlloyEditorBasic), 2 showroom
  (`CARSHOWROOM_BUY` -> CarShowroomScreen), 3 DJ (playlists -> AudioPlaylistScreen); left/right change character,
  select opens the menu, cancel closes it or opens the quit dialog (title 0x34FE3B70, YES/NO; YES -> MainMenuScreen
  after `dword_6C1288` = 20 frames). Screens that are not ported are refused (logged): AlloyEditorBasic (editor),
  AudioPlaylistScreen (DJ; 0x0022BA30.., next candidate). Static crc tables:
  `dword_75F43C` RACEINFO_TYPE/CATEGORY/EVENT, `dword_75F430..438` the garage entries, `dword_75F408` CARSHOWROOM_BUY.
  First update with `!completed_any && profile_active` starts "UT0EVENT1" (0x0021C430) -> LoadingScreen, after
  queueing its cut scene `CUTS\<off_685400[event +0x48]>` (`CareerEventView.unlock_message`; index 0 "NONE" is
  non-null, so it is queued and fails to open, as in the original). It sets `byte_6F9058 = 1` (0x0021C618; the only
  other writer is the load 0x00104E30) — now through `CareerRequest::SetCompletedAny` into core's
  `vars.completed_any` (fixed 2026-11-05: only the `CareerView` copy was set, `career_in` read core's false back, and
  the hangout restarted the tutorial after every race, completed or not).
- **Starting an event**: StartEvent (0x0028F430) sets `CareerView.start_pending`; the LoadingScreen's 10th frame
  (GameManager 0x000BBF60 starts `m_pEvent`) sends `StartCareerEvent` instead of `StartQuickRace`; core runs its
  StartEvent part + the jump, the binary sets post race RaceCompleteScreen / return HangoutScreen and stops the FE.
- **Hangout camera flight**: changes build two `CCatMullRom` splines (flags 2: fixed step `+36`, open, no banking;
  segments (p0,p1,p2,p3) with doubled ends, samples p1 then t = step.. while t < 1 - step/2, last point appended)
  for positions and look-ats. Left/right (0x0021B0D0): prev LOOKAT, the MOVERIGHT cameras (kind 1) of the new
  character going left (reversed) / of the old one going right, new LOOKAT, step = first MOVERIGHT camera's +52 float
  (0.2 if none). Kind to kind (0x0021AE70, step `dword_6C128C` 0.15): menu open 0->2 (camera 2), close 2->0 (3), Tory
  0->2, browser closed 2->0 (3), leaving 2->3 (4). Points -> `HangoutScreen.flight` (0x0021AD30), move timer
  `(n-1) * +436` (0.06); 0x0021A310 each update while the timer runs: when the FE3DCamera move ended (+224 <= 0) the
  next point starts a `FoyerState::start_vector_move` (Start sets mode 1 linear, +228 = 0.06, +347 end once less than
  a frame is left; mode 0 = sin ease), the menu / arrows swap at half the points (dbl_649780 = 0.5), camera 4 arms the
  loading screen countdown. The camera move uses the real frame delta (FE update event dt). Up vectors: the flight
  keeps the start / end cameras' CamAxis (+396 / +412); per point `CarDamage 0x000762F0` gives `up = to + (from - to)
  * move_timer` (+432 = flight time left in seconds, not a 0..1 fraction — original) into the camera's target and
  start up (`0x00271DA0` / `0x00271DD0`, `FoyerState::set_target_up` / `set_up`); the camera element mixes them like
  position (`+48 = normalise(+64 + (+80 - +64) * s)`, at rest `+80`). Table moves take the table's up. Shipped axes
  are all (0,1,0) except one (0,0.1,0), so nothing tilts with the shipped data.
- **Tory's event browser** (`CFEHangoutProgression` pages, `+8`): 0 categories (0x0028DE90: UNDERGROUND /
  PROFESSIONAL / CHALENGES, icon `FE_Icon_RaceAvailable` while open events remain), 1 challenges (TEAM / STORY /
  TEAM_INVITES), 3 tiers (`EVENT_TIER_n`, counts / status / opening criteria), 4 events (locked disabled; summary of
  the highlighted one), 5 details (area, kind, laps or A-to-B, opponents, race count, race type `dword_6F0DA8`
  RACECATEGORY_*, track icon), 6 = StartEvent. Top level from the tier 1 unlocked counts (0x0028C9C0). Counts / n-th
  event = `CareerView::count / nth` (CompartmentManager 0x0008EDB0 / 0x0008EED0 filters). `dword_6F9BDC` =
  `CareerView::category`, browsed `dword_6F9BE8` = `browse`. Up/down (0x00219B70) refresh tier / event info.
  `fecapture career` A from Tory goes straight to the details (fresh career: top 0), the next A starts the event.
- **Teams** (`CareerView.teams` = core's `CareerTeams` (the ai mirrors `CTeamDatabase`), `CareerView.team` =
  `dword_6F71D0`): page 1 TEAM_INVITES enabled when `count_teams_offering_invites(team) > 0`; page 7 `fill_invites`
  (0x0028E4B0, crc cache of 10 at +108) lists the team names; `show_invite` (0x0028E5F0) TEAM name, BONUS / STATUS
  "%d" via SetText, CAR string (BENEFIT +204 never set: shows NOT FOUND, original). Select on page 7 (0x0028EE90
  case 7): `mpTeamExtendingInvite` (+220, a crc here) = cache[sel], question 0xEDD256C1 (idJoinTeamName,
  idCashBonus) or 0xFD7A9488 when joined before, dialog `[GENERAL_NO, GENERAL_YES]` (selection 1 = yes), +224 = 3,
  page 8. Answers `answer_invite` (0x00290410) / `decline_invite` (0x002908E0): +224 1 after-race unlock, 2 its swap
  prompt, 3 browser invite, 4 its swap prompt; with a current team the swap prompt 0x93203028 (idNewTeamName,
  idCurrentTeamName) first; `join_team` (0x00290BC0): team, unless joined once cash += bonus, `update_cash`,
  `CountTeamJoined`, `SetJoinedOnce`; `joined_message` (0x002909B0, `set_message` 0x0027B1A0, 200 frames) 0x5F144362 /
  0x2CDEB106. The hangout's select with the browser closed answers a waiting invite (0x00219CC0). Requests:
  `CareerRequest::{SetTeam, CountTeamJoined, Team(TeamRequest)}`; `career_out` writes `gm.vars.team`,
  `gm.vars.unknown_264[0]` (= `dword_6F71D4`, naming asked of core) and `Messages<TeamRequest>`; team flag edits
  are also applied to the local copy at once (`CareerView::team_request`). StartEvent's team car is core's part of
  StartEvent (`career.rs`, `RaceCarSource::team_car_by_team`, done). Still open in core.md: naming
  `unknown_264[0]` (our `++dword_6F71D4`, idb `g_TeamsJoinedCount`: written only by SetInitialVariables and
  join_team's first-join branch, read by nobody but the save); `career_out` keeps writing `unknown_264[0]` until core
  renames it.
- `fecapture teams` (all events unlocked, `TEAMS.BIN` via `velocity_ai::team_database` with the first two event unlock
  teams unlocked — fecapture is why `velocity_ai` is a frontend dependency): `teams_tory_offer`, `teams_challenges`,
  `teams_invites` (= `research/screens/fe_teams.png`), `teams_invite_question`, `teams_joined`.
- fecapture's hangout 3D scene renders black (camera values checked: correct); the same at HEAD before this stage,
  so not a frontend camera issue — unexplained, the earlier `career_hangout.png` showed the scene.
  **Fixed by graphics** (graphics.md "Fixed: black FE hangout": the car-stencil view marker was bevy's default).
- **Black 3D check (done; was graphics' request)**: every fecapture shot measures its 3D area = pixels where the FE
  2D image (`FrontendState.image`) has alpha 0; fewer than 0.5% non-zero there -> error log and exit code 2 after
  the script (hidden_run passes it through). Shots fully covered by 2D (career_tory_page2..4, quit dialog) are
  skipped. All five modes pass (dark hangout main menu ~2-6% non-zero, hangout ~30-40%, car select / garage ~99%).
  Negative control `FECAPTURE_NO_3D=1` (all Camera3d off, 2D camera clears black): `garage` exits 2.
- **After-race messages** (0x0028F790, one step per frame, waits while the overlay message (0x0027B4F0) or dialog is
  up): 1/2 movies (event +0x4C unless failed; `dword_6F7248` then cleared via `CareerRequest::ClearUnlockMessage`) ->
  `CUTS\<off_685400[n]>` (`CUT_MOVIES`, 13 names, [0] "NONE"); 3 tip (+0x50) -> TipsScreen; 4 team unlocked (0x0028FE70: completed event
  with `unlock_team`, team newly unlocked -> `ClearNewlyUnlocked`, dialog 0x2AD3A09C (idTeamName, idCashBonus),
  +224 = 1, page 8; unknown team -> the FeedbackTeamUnlocked warning); 5 Tory's
  result line `TORY_RETURNTOHANGOUT_{HIGH,MED,LOW}{POSITIVE,NEGATIVE}_{1,2}` / TEAM_CHALLENGE_WON/LOST (0x0028F8C0,
  series by points, else last round first race finish vs result position); 6 TORY_PROFESSIONAL_AVAILABLE /
  CHALLENGES_AVAILABLE / TIERUNLOCKED against `byte_6F9BE4`, then `CountUnlockedTiers`; 7 TORY_VEHICLESUNLOCKED from
  `fe.vehicles` lock count changes (states 2/3); 8 clears the event. Messages: `overlay::set_message_string`
  (0x0027B200) + timeout 200 + show. Event fields come from `lib.rs::career_event_view` (`CareerEventView`).
- **RaceCompleteScreen save popups (ported, 2026-11-05)**: Update polls core's `SaveFlow` (= XMem 0x0030EF60; lent to
  the FE update as `Feck2.save_flow` by `run_frontend`, last block in `Feck2.save_status` = `dword_701404`) with the
  update's dt. Status +2: 0/1 back one screen + nav icon 1; 3 question: Select's answer (+12, overlay dialog selection
  0x00205B00, only while the last status was 1) -> `respond(responses[0|1])` once, then 0x00205BE0; 4 popup = 0x00205BE0
  (options `(flags & 0x70) >> 4`: none -> hide dialog, message = text +12 (`set_message_string`), show; else unless a
  dialog is up: message off, `set_dialog_count` (0x0027B980, new), dialog strings (title = text, options), dialog
  visible at `(flags & 0xE00) >> 9`); 5 done -> ready; 6 respond(+32). Core's save = Saving popup (4 polls, then 3 s
  idle with the popup kept), SaveComplete popup once, 3 s idle, Exit (3). A failed write: ErrorSaving question after
  Saving's 3 s, OK -> UserExit (-1) -> back. STAND-IN kept: `ReturnedFromRace` with no flow running (core did not save)
  -> `race_save_status` 0 ready / -1 back. Test `capture_race_complete_save_popups`, shot `research/screens/
  fe_race_save_popup.png` ("Saving data. Do not switch off your Xbox console.").
- **DummyGameScreen (ported)** (`CFEDummyGameScreen` 0x48 B: +8 indicator FE_Arrow_16 (75,240), +12 DUMMYGAME_SELECT_RESULT
  font 2 (80,210), +16 seven rows `dword_75D798` (DUMMYGAME_PLAYRACE, _1ST.._6TH) font 2 (100, 240+30i), +44 CURRENT_EVENT
  font 3 (60,100) arg idCurrentEvent = event crc, +48 CURRENT_RACE font 2 (60,140) arg idCurrentRace = race record +4, +52
  PRESTIGE_GAINED font 1 (60,170) orange, arg dPrestige; +57 valid, +60 selection, +64 competitors, +68 prestige). Reached
  from RaceCompleteScreen (return HangoutScreen + event lacks 0x10 = not over: every multi-race event between races)
  and from the hangout's StartEvent when the event's race has no record (0x00219F70 / 0x002092A0: **18 shipped events**,
  test `shipped_event_races_have_records`; they read "RACE CURRENTLY NOT AVAILABLE IN BUILD" with row 0 grey). Enter shows
  rows 0..competitors; up/down wrap (indicator y 236+30 sel); left / right *held* -10 (not below 0) / +10 per update;
  cancel back one. Select: sel != 0 or no human seated -> `CareerRequest::DummyResult{position, prestige}`
  (`lib.rs::dummy_result`: `event_state += prestige`, positions: player = sel, others `rand() % n` into free places,
  nobody human = slot order; race / round / event over with core's pieces; `quit_race = 0`; then core's `race_complete`
  save + `ReturnedFromRace`), GotoScreen RaceCompleteScreen; sel 0 on a valid race -> race class into
  `quick_race.pro` / `amateur` (`byte_6F70E9/E8`), LoadingScreen -> `CareerRequest::ContinueEvent` -> core's
  `GameFlowRequest::StartCurrentEvent` (0x000BBF60, mapped in `career_out`). Hangout route: `SelectEvent` = core's
  `career::start_event` (StartEvent's game part) and the pending event as `career.event` for this frame. Not ported:
  the "Display cheat screen" tweak `byte_75EDB4`. Test `capture_dummy_game_screen`, shot `research/screens/fe_dummy_game.png`.
- **RaceCompleteScreen save (ported)**: Start 0x00205830: `byte_6F5D08` (`CareerView.quit_race` = gm.quit_race) ->
  ready; else `byte_6F9054` (`CareerView.save_enabled` = core's `vars.save_enabled`, set by profile new / load /
  save 0x00234550 / 0x002345E0 / 0x00234700) -> not ready + save 0x001061D0 ("Primary", = core's `race_complete`
  save); else ready. Update 0x00205900: ready -> GotoPrevious(0, return screen; HangoutScreen -> DummyGameScreen when
  the event lacks flag 0x10 = EVENT_FLAG_FINISHED, see "DummyGameScreen"), then the XMem status (0x0030EF60, `dword_701404` +0) +2:
  0/1 (failed) `byte_6F7165` = 0, nav icon 1 (0x0027AD30), GotoPrevious(1, 0); 3 (question: answer +12 from Select
  0x00205B00 = overlay +244 -> callback +32 / +36 via 0x0030EF50) and 4 (popup 0x00205BE0: dialog with `(+4 & 0x70)
  >> 4` options +16.., default `(+4 & 0xE00) >> 9`, else message +12) come from core's `SaveFlow` (see "save
  popups" above); 5 (done) ready; 6 callback +32. Without a running flow: core's `ReturnedFromRace { saved }` caught
  in `career_in` (tried `!quit_race && save_enabled` -1, else 0; STAND-IN: 0 also readies), `CareerView.race_save_status`.
  A save costs one extra FE update before leaving, as in the original.
- Open: Xbox Live / system link screens (no network layer).
- `fecapture career` = profile select, keyboard (empty / typed "Qq"), hangout, mid-flight, garage menu, Tory's
  browser pages, quit dialog (`career_*.png`; events from a fresh event database, no game flow). Reference shot
  `research/screens/fe_keyboard.png`. IDA dumps: `research/fe_keyboard.c`, `fe_browser.c`, `fe_spline*.c`, `fe_kb*.c`.

**Licence plate text (done)**: `plate_font::PlateFonts` is inserted as `PlateFontHook` at frontend startup (when the
game dir resolves). It keeps its own two `CFontHelper`s: FE set (`LoadFrontendFonts`, id 2 = FEFont12) and game set
(`LoadGameFonts`, id 6 = FEFont11; shares the fefont11 crc with id 1, so its scale is 0.75 like the FE ids). Glyph
quads come from `CBaseFont::draw_string_at` (flags 0) on a scaled copy; page = the font's single texture as
`DecodedTexture`. Test `plate_fonts_draw_octane`.

**HUD "1st/1" (career smoke)**: not the HUD. The OSD prints `"/%d"` of `VsAIWorld::GetRaceLength` (0x000348C0 ->
VsRaceManager 0x0003DCA0 = players + active ai cars) after the position string; core's `velocity/src/hud.rs` feeds
`race_length = 1` and `race_cars.rs` feeds `position: 1`, `lap: 1` (placeholders). Filed in docs/core.md.

**Team invites / team unlocked (done; were ai's and core's requests)**: see "Teams" under Career. Decomps:
`research/ai_dump/fe_teams.c`, `fe_team_accessors.c`, `research/fe_browser.c` (0x0028EE90 / 0x0028F270).

**Trailblazer OSD message (done; was core's request)**: COSDInGameScreen::Update (0x000F99E0): item 13 (+52,
`OSD_INGAME_TRAILBLAZER_INFO`) gets the string of `GameManager+0x108C` (0x000FAEA0) and is shown while it is non-zero,
hidden otherwise. `OsdRaceStatus.trail_blazer_message`, filled by `hud.rs::trail_blazer_in` from
`GameManager.trail_blazer_message` (core's hud.rs untouched). Check run (core's): LADown Race02 with `SMOKE_RACE=2
VELOCITY_NO_RACE_OBJECTS=1 "SMOKE_VIA=340.5,-53.3;349.6,-71.1" SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=128` (0x643ce31, 5 s).

**Teams joined / quit event (done; were core's requests)**: `career_out` bumps `gm.vars.teams_joined_mut()`;
`hud::end_of_game` sets `gm.quit_event` before `set_game_mode(3)`. Asked core (core.md "Requests from frontend") to
drop its `quit_event` copy in the binary's `hud.rs::menus_to_game`. The race stays loaded in PostRace (`InRace` =
InGame | PostRace); a post race Restart / next drag round switches the state back to InGame.

**Name tags (`osd_in_game_screen::update_name_tags`, 0x000F9340; Create's end makes one text item per racer: font 6,
flags 0x180000, mask 3)**: called from Update (0x000FA915) unless the attract mode plays; with Player names
(`byte_6F915F`, off by default) or `byte_6FD754` (debug, 0): per AI race vehicle (race manager +36 / +56) item 27 +
slot (vehicle +120): visible, text = profile name, colour = profile colour with alpha `ftol(a * 128)` (raw, so at
most half opaque), a = 1 up to 20 m from driver 0, lerp to 0 at 40 m (`flt_6860C0`, `6860C4`, `6860C8`, far
`flt_6FD750` bss 0), hidden at alpha 0; position = car + (0, 1.6 + dist / 40 * 4, 0) (`dword_6FD770`, static init
0x00423860) into item +36..+44 (via the OSD bridge 0x00098470 / 0x000F98E0, applied in single player because
`byte_68565C` = 1); zoom (0.1, 2.0). Drawn by the 3D pass `COSDManager::render_3d` (0x000EF5F0 from GameManager
0x000B9010 -> 0x000EEE00 -> FontManager 0x002AD680): projected with the race camera (`RaceHudInput.name_tags.
view_proj`, filled by `hud::camera_in` from `TrackCamera`), skipped behind the eye, alpha faded from `zoom_y` (2) to
1e6 by clip w, on top (zoom_x != 0), centred (flag 0x100000); the manager mask is not checked. Items are created on
first use (the original makes event +21 of them in Create). Vehicles come from game flow (request in core.md); test
`capture_race_hud` (`race_hud_names.png`). Two players: one pass per player view, see "Name tags per view".

Smoke status (3D hud / plate / movie skip stage, built to `target/fe`): plain exit 0 (`research/fe_smoke_osd3d.log`,
with `VELOCITY_TFT`) and `SMOKE_ATTRACT=1` (two attract races, `research/fe_smoke_attract.log`) exit 0;
`SMOKE_CAREER=1` now times out on PlateCustomizationScreen (the smoke script presses nothing
there; request in core.md) — `research/fe_smoke_career.log`.

**Speedo / map gate (attract race)**: OSD3dInterface 0x000EA2B0 draws the speedo and the map only while the game OSD
manager is enabled (+29 bit 0) and its mask has no bit 1, so they are hidden before the start, in the attract race and
after the finish (state 0 / attract 3 / 7 set bit 1). Not ported: the in-3D map model.
**Hud slide (ported, `hud::HudSlide` in `HudState`, kept across races like the static)**: inside that gate, per render
call, +2932 -= 0.33 while +2929 ("out") else += 0.33, clamped -12..0; speedo position = config + (-slide, slide, 0)
(`Speedo::set_offset` = 0x000F46F0), map position x / y += slide. Ctor 0x000E9A80: out, 0; 0x000EAF00 = -12 + out
(`HudCommand::HudSlideReset`, GameManager 0x000BC5F0), 0x000EAF60 = in (`HudSlideIn`, TimelineSequencer 0x00175270).
Both sent by core (request in core.md); STAND-IN: no offset until the first one arrives. The step is per frame like
the FE's frame-count animations (36 frames to slide in).

**Attract mode, main menu side (done; was core's request)**: `g_AttractMode` (`byte_6F5A28`) = core's
`velocity_game::font_helper::CAttractMode`. `run_frontend` copies it, `gm.rand`, a seed (ms in the front end, for
GM +0x190) and `gm.jump.active` (GameVars 0x000C2740) into `Feck2.attract: AttractLink` before the FE update and
writes the mode (and the rand, if 0x000B34D0 ran) back after. FEMainMenuScreen: Update 0x0023C663 (after the
start-text fade) `front_end_update(dt, ...)` with "MainMenuScreen", then a pending jump runs `CGameJump::Execute`
(0x000C2860 FE part: GotoScreen("LoadingScreen"), GameManager 0x000BE7C0 post race screen = MainMenuScreen) and the
update returns; Up / Down / Select 0x000B34A0 `reset_timer`, Leave 0x000B3470 `stop_timer` (Start 0x0023C970 does
not reset). A start (0x000B3550 -> 0x000C2810) also does `CGameJump::Set` (the plugin's `gm.jump.set`) and
`stop_pending`; core's front end update races the jump, so that LoadingScreen's 10th frame raises nothing
(`AttractLink.loading_jump`, cleared on reinit). Movies (In001..3) go to `Feck2.movies` (played). Core's stand-in
`velocity/src/attract.rs::main_menu_attract` is asked to go (docs/core.md "Requests from frontend"). Until it goes,
`run_frontend` leaves the timer calls to it whenever `CAttractMode` was written by another system since the last
frame (`is_changed()`), otherwise both counted the idle down (2x) and SMOKE_ATTRACT hit a second attract race before
exiting; the jump mirror still sends a stand-in start through Execute / LoadingScreen. Remove that guard with it.
Smoke status this stage: plain exit 0; `SMOKE_REVIEW=1 SMOKE_RACE_TYPE=5` exit 0; London sprint review (post race
shot) exit 0; `SMOKE_CAREER=1` (needs a fresh `VELOCITY_SAVE_DIR`, an old "Q" save stalls it on the main menu) and
`SMOKE_ATTRACT=1` before the guard exit 127 in the *second* race load (graphics, bevy_pbr cluster overflow — request
in graphics.md); SMOKE_ATTRACT after the guard not re-verified (run stopped: headless audio is audible).
`byte_6F5D08` = GameManager +0x140 `gm.quit_race` (m_bQuitRace): readers RaceCompleteScreen 0x00205830 / 0x00205900,
FEHangoutProgression 0x0028F7A7. Tests `capture_main_menu_attract_start`.

**Attract HUD (done)**: COSDInGameScreen::Update 0x000F99E0, `OsdRaceStatus.{attract, attract_quitting,
session_state}` filled by `hud::game_in` from `CAttractMode` (+0 / +0x48) and `RaceSession.state` (`dword_6F5BEC`;
None without game flow = untouched mask). Session phase -> OSD manager (`dword_6FD200`) mask +36 every update: state
0 `|= 1 | 32` (items 25 OSD_WAITING_FOR_PLAYERS shown / 26 hidden: single player has no network state), 3: attract
`|= 1, &= ~32`, show 22 (logo) and 20, else `&= ~1, |= 32`; both hide 25 / 26; 7 `|= 1`. Mask bit 1 hides the mask-3
race items, bit 2 too (core's `RaceSessionMessage::OsdFlags` COUNTDOWN), 32 the attract items, 0x2000 RESULTS.
`hud::osd_flags_in` now applies core's `OsdFlags {set, clear}` to the same mask. Items (Create 0x000F82F0): 20
Velocity26 / 21 Velocity15 (black), both text buffer 8 = literal "Press Start" (0x000F8E78), flags 8 mask 32; 22 =
graphic `OSD_overlay_game_logo` (OSDT.vcd, loaded by the in-game screen init 0x000FAB40 with menu_top /
menu_bottom), 480x120 at layout 22 (666, 62.5), flags 2 (left), mask 32 — so it sits right of centre as authored.
The per-update fade (`ig::attract_step`, 0x000F9A70..0x000F9C08, run once per GameTick via `HudState.ticks`):
statics dir `dword_6FD758` 0, phase `6FD75C` 0, step `6860E4` 2, alpha `6860E8` 0x80, lap secs `6FD760` 0;
dir ±step; phase 0 + dir -1 + alpha 0 -> fade in, phase 1, hide 20 / show 21; phase 1 + alpha 0x80 -> hold; quitting
-20 a step (u32: `max(a,0)` 0x000FAFB0 never clamps, wraps — original); colour VISOS_Colour(alpha byte, 128,128,128)
on 20 / 21; lap secs > 1 and phase 0 -> fade out; lap secs = `_ftol(lap time)` while racing. **Graphic items now
render** (`COSDManager::render_graphic` = 0x000EEEF0: strip from (x + just_x, y + just_y) up by height, uv origin
+56/+60 and size +64/+68 in 4096 fixed point, item colour). Name tags and the speedo / map gate: see "Name tags"
above.
**Cinematic bars (ported)**: items 23 / 24 (+92 / +96) = graphics `menu_top` / `menu_bottom` (FAB40 texture slots +4 /
+8), 1344x200 at (664, 900) / (664, 100), layer 100, flags 0x48, mask 1024, created hidden (Create 0x000F82F0 tail).
Only OSDManager 0x000ED6E0(show) -> 0x000F9980 toggles them: `HudCommand::CinematicBars(bool)` ->
`ig::set_cinematic_bars`. Callers (all outside the frontend; requests in core.md / ai.md): VsCrashCamera 0x00062C10 on /
0x00062D90 off, POI camera 0x0006A720 / 0x0006A7B0, TimelineCameraShaker 0x0017A540 / 0x0017A580 (F3EventCinematicBars),
RestartRace 0x000BE26D off, VsAIRaceVehicle 0x00018BD7 off (respawn hand-back). Test in `capture_race_hud`
(`race_hud_bars.png`, `research/screens/fe_cinematic_bars.png`).

### Requests from physics (transmission choice into the setup) — done
- Car select commit (0x0022446D) writes `setup.unk_18c = (transmission != 1)` (FE working setup +0x720 +0x18C) into the
  setup stored in `QuickRaceSettings.player_setups[player]`; `quick_race.automatic` is kept too. Not read back: the
  toggle 0x002258F0 only flips `+5432` (drag -> manual) and never reads the setup; no other ported FE screen toggles it.

### Requests from core (GamePlayer names, plate text) — answers to "Requests from frontend" in core.md
- Named accessors now exist on `CFEGameVariables`: `tips_watched()` / `tips_watched_mut()` (+0x268), `plate_style()` /
  `plate_style_mut()` (+0x26C), `plate_text()` / `set_plate_text()` (+0x270 `unk_6F71E0`, inline wide string in the
  saved block), `teams_joined()` / `_mut()`. Please switch `lib.rs` (`unknown_264[1]`, `unknown_264[2]`) and the
  hangout comment to them; core then turns the array into fields.
- Plate: core's `career_requests` copies graphics' `LicencePlate` (style, text) into GamePlayer every frame and sets
  `LicencePlate` from GamePlayer after NewProfile / a successful LoadProfile, so the text is saved and restored; the
  `plate_sync` style write is now redundant (harmless).
- Done as asked: `quit_event` copy dropped from the binary; `RaceHudInput.name_tags` filled every InGame frame
  (colour = the ai's `name_colour`, 0x00015F80); `HudSlideReset` / `HudSlideIn` sent from `RaceSessionMessage::Hud`
  (MonitorSessions Init / Reset, flag girl "InGame" / SHOWHUD); `HudCommand::Start` now carries the first race
  section's edges (sent on the "Generating OSD Map" step, or "Load 3d Interface" in a cruise: no map);
  PlateCustomizationScreen A in the career smoke.

### Requests from graphics (foyer character spec index, FE3DCharacters → Characters.cpp 0x00079EC0) — done
- Element +764 = record +284: the element ctor copies the whole record to element +480 (0x00286180, `this[71] =
  a2[71]`). Writers of record +284: Chars.txt loader 0x0025F440 = the loop index, Crowd.txt 0x0025FD60 = `+61316`
  (char count) + i, Dance.txt 0x00260250 = `+61316 + +61320` + i (one running index). `FECharacter.index`;
  `foyer.rs` inserts `SpecCharIndex(index as u8)` on the skinned root. Test `hangout_chars` (0..4).
  Dumps `research/fe_spec764*.{txt,c}`. Shot `research/screens/fe_spec_chars.png` (fecapture
  `scene_hangout_characters`).

### Requests from graphics (3d hud lighting, front-end car shadow) — done
- Done: `spawn_view` builds tft_frame / hinge / screen, map frame and map screen with `light_set =
  VsLights::default()` (lit copies in `Osd3dView.lit_materials`; the LCD_Movie copy of tft_screen is unlit);
  `run_osd_3d` (while drawn) sets `o3d::hud_lights(CarLightSets[0], PoliceFlash)` on them, request 0 = race car 0
  (player 0, core's race_cars id). `foyer.rs` specs call `.with_setup_underbody(setup, db)`; the car key includes
  (wheel height, neon) so a fitted wheel / neon respawns the car. Test `hud_lights_follow_the_police_flash`.
  Shot `research/screens/fe_osd3d_lit.png` (plain smoke + `VELOCITY_TFT`, exit 0, `research/fe_smoke_lit.log`): TFT
  frame and map frame now light grey (texture x white ambient); the TFT screen shows white before the movie is up.
- Original request:
- **3d hud light set** (replaces the "STAND-IN: the light set … is not applied" note; graphics.md "Light sets"): every
  OSDM prim has stage flags 1 and no normals, so in the original its rgb is the hud set's ambient, not the vertex
  colour. In `osd_3d_interface::spawn_view` build the five slot models with `opts.light_set =
  Some(VsLights::default())` (the builder then returns lit material copies), and each render (OSD3dInterface
  0x000EA2B0) call `XboxStageMaterial::set_lit(Some(&set))` on the tft frame / hinge / screen (own shader), map frame
  and map screen materials, with `set = car_render::CarLightSets.0[&player car request].hud_light_set(flash)` (fall
  back to `VsLights::default()`); keep `set_lit(None)` on the LCD_Movie materials (GFXHelper 0x0012AE10 arg 0).
  `flash` = the last `VsPoliceManager::flash` colour (0x0003A1C0): ai answer — read the resource
  `velocity_ai::plugin::PoliceFlash([f32; 4])` (rgb + weight, zero by default and at each race's AI init).
- **Front-end car shadow**: `foyer.rs` should build its `CarSpec` with `.with_setup_underbody(setup, db)` (shadow
  size, neon, and `wheel_height` = the setup wheel's WHEELS.BIN +0x0C = CarRender +1816, the shadow height; FE3DVehicles
  0x0025C890). The shadow itself is drawn by graphics for every `spec.frontend` car.

#### Multiplayer / split screen (`fe_multiplayer_screen`)
- `MultiplayerScreen` (0x1C B: +8 loaded, +12 -1 on Start, +16 selection, +20 "DebugTransmissionText" font 0 (100,300)
  MULTIPLAYER_TOGGLE_TRANSMISSION): Activate = SELECT / BACK, nav 2 = MULTIPLAYER_PEER_TO_PEER / _CLIENT_SERVER by
  `byte_6F7159` (`quick_race.peer_to_peer`, option 1 toggles), borders, style 1, entries `dword_75EDA0`
  MULTIPLAYER_OPTION_ONLINE / _SYSTEMLINK / _SPLITSCREEN. Select: 0 `byte_6F90BF` = 1, OnlineMatchOptionsScreen when the
  network game is signed in (state 11) else `byte_6F90C2` = 1 + OnlineSignInScreen; 1 NetworkMenuScreen; 2 split screen:
  `byte_6F7157` (`two_players`) = 1, `byte_6F70D8` = 1, `dword_6F5C00` = 2, car select next = QuickRaceCarSelectScreen,
  QuickRaceTextScreen. The Xbox Live / system link screens (FEOnline*, FENetwork*) are **not ported** (no network
  layer): those selects log "Screen not found" and stay. Main menu quick race (0x0023CE80) clears `two_players`.
- Split flow checked: track -> car select player 1 -> player 2 (existing `set_next_screen(1, QuickRaceStartScreen)`) ->
  start screen with both rows (0x002211C0: GENERAL_PLAYER2 + `*dword_6F7478` = `quick_race.car_name_crc2`). Game side
  (player count 2, player 2's car, split render) asked of core. Test `capture_split_screen_flow`, shots
  `research/screens/fe_multiplayer.png`, `fe_split_screen.png`. The white box in the transmission text is the string's
  inline button glyph (font icon, not drawn by the port's font path — pre-existing, all such strings).

### Requests from core (profile options, after-race save popups, garage copy, DummyGameScreen) — 2026-11-05 — all done
- Done: save popups (RaceCompleteScreen polls `SaveFlow`), DummyGameScreen, and the shared garage copy: `run_frontend`
  copies `gm.vars` {cash, prestige, garage_vehicle, garage_setups, garage, garage_slot} into `Feck2.garage` /
  `quick_race` right before the FE update and back right after (`garage_in` / `garage_out`, change detection bypassed);
  core was told `career_sync`'s copy can go (core.md). Without game flow (fecapture) the FE's own copy is used.
- Original request:
- **Profile options: done in core + binary, nothing needed.** Layout = core.md "Options block"
  (`velocity_game::register_file_types::save_file_data::OptionsBlock`). The binary pushes the saved / new-profile
  values into `GameOptions` on `CareerMessage::{Created, Loaded{ok}}` (your change detection re-applies audio /
  display) and copies `GameOptions` into `Career.options` every frame for the save. A new profile resets volumes /
  display to the SetInitialVariables defaults, as the original. Not saved: jump / crash / police cam, language.
- **After-race popups exist in F3_D** (states 3 / 4 of RaceCompleteScreen Update 0x00205900): poll
  `ResMut<velocity_game::save_game::save_vars::SaveFlow>` once per FE update instead of the stand-in:
  `let s = flow.poll(dt)` = `XMem 0x0030EF60` (`s.status` = `*dword_701404`: 1 question, 2 popup, 3 done, -1 user
  exit, 0 nothing / idle), `s.flags` (+4), `s.title` / `s.text` (+8 / +0xC string crcs; text args `idGameTitle`,
  `nRequiredBlocks`), `s.options` (+0x10..), `s.responses` (+0x20 / +0x24 = what Select passes to 0x0030EF50:
  `flow.respond(s.responses[i])`), `s.english` fallbacks. Poll only while `flow.in_progress()` or keep polling: an
  idle flow returns status 0 (case 2 = default). A normal save shows "Saving" (popup) for >= 3 s, then
  "SaveComplete" (popup, once; 3 s idle at status 0 in which the popup stays up), then 3 = ready. Core already
  wrote the file when `ReturnedFromRace{saved}` arrives. ProfileSelectScreen's 0x002337C0 polls the same block
  (ConfirmOverwrite there needs core to route `SaveProfile` through the flow with `no_confirm = false`; ask if wanted).
- **Shared garage copy**: core owns the authoritative values (`GameManager.vars`: cash, prestige, garage,
  garage_setups, garage_vehicle, garage_slot). The front end already depends on velocity_game: once its screens read /
  write `gm.vars` directly, tell core and `bridge.rs::career_sync` goes. Core has nothing to add.
- **DummyGameScreen** (FEDummyGameScreen.cpp 0x00208D60..0x0020A210) is a Feck2 screen: front end's. Core's side is
  already there (`gm.quit_race` clear = 0x00209340 is `GameManager.quit_race`).

### Requests from graphics (FErefl floor reflection, front-end flares, character cross-fade) — done 2026-11-06
- Done: the foyer camera carries `FrontEndCamera`; `run_foyer` inserts `FeReflection::from_lights_txt` with every
  Lights.txt load (off when the file / block is missing); all background meshes are on layers [0, FE_REFL_ENV_LAYER]:
  the camera's list +65 *is* the background list and +292 is its shown flag — SetBackgroundVisible 0x00273620 sets it
  through vtable +120 (0x0029A310; +112 0x0029A120 adds, +116 0x0029A220 reads, vtable 0x0065AAC0), hidden roots drop
  out through Visibility. `FrontEndRender` stays default: `dword_7065A4` is the "GarageCameraIndex" debug tweak (CAM_OFF
  0..CAM_FULL 4) written only by the submission screenshot code 0x002528D0 / FEPlayerGarageScreen 0x0024D260 (not
  ported), `byte_7065C0` only by CarPalette 0x0010CE80(1) from FEHangoutScreen 0x0021EF50 when the "Enable (Hidden)
  Showroom screenshots" tweaks `byte_75F44D/E/F` are on (dumps `research/fe_7065a4.c`, `fe_flares_off.c`). Cross-fade:
  `FECharacter::blend` (0x00285350: weight `(now - +132) / 1.0` while < 1 and +116 loaded; +132 is set only when a
  queued anim starts, not on the fall back to the default) -> `SkinnedModelBlend` each frame of the fade, then one
  `prev: None, weight 1` (`FoyerView.blending`). Test `crossfade_after_queued_anim`. Dump `research/fe_xfade.c`.
- Original request:
- Insert `visos_render::rain_surface::FrontEndCamera` on the foyer camera (`FoyerCamera`). Graphics then draws the
  front-end flares there (FECK2ElementCamera 0x002999B0 → 0x00299C5A) and, in foyers with a reflection, adds its own
  reflection / paste / post cameras (orders −5, 1, 2) around it and sets its `Tonemapping` (view-key marker; leave it).
- Keep `visos_render::rain_surface::FrontEndRender { garage, flares_off }` current: `garage` = `dword_7065A4 != 0` (the
  customisation view cycle CarCustomisation 0x002527A0 / 0x00252830), `flares_off` = `byte_7065C0` (CarPalette
  0x0010CE80, caller not traced).
- At `FECK2Element3D::LoadFoyerLights` (0x00298CD0; `foyer.rs` where the Lights.txt is read) insert
  `visos_render::rain_surface::FeReflection::from_lights_txt(&text)` (REFLECTION block; carSel and modArea have one).
- Elements with FECK2ElementCamera's reflect flag (+292 + i, the first element list +65 drawn by 0x00299D00): give their
  meshes `RenderLayers::from_layers(&[0, FE_REFL_ENV_LAYER])` so "Reflect Environment" foyers mirror them.
- Character cross-fade (the FE character element draw 0x00285350 → Characters 0x00079EC0): when the anim changes keep
  the previous one (+116) and the change time (+132); while (now − +132) < `dword_6C1780` (1.0 s) write
  `visos_render::skinned_object_render::SkinnedModelBlend { request, prev: Some((prev anim, prev time +2564)), weight:
  (now − +132) / 1.0 }` each frame (prev time advances by its frame increment, wrapping when +2569 is set, else held at
  1), then `prev: None, weight: 1.0`.

### Requests from core (split screen OSD, 2026-11-05)
- Core builds the race's views (GameManager 0x000B3EC0 = `velocity_game::game_mode::game_views::GameViews`, resource):
  `views` (+588) world views and `second` (+988) — for two players `second[0]` / `second[1]` are the players' halves
  (kinds 2 / 3 top / bottom, 4 / 5 left / right when `byte_6F9180` = options +0x24 is set; note +0x24 is what core's
  `OptionsBlock.time_format` calls it — 0x000B3EC0 reads it as the split direction, please check your FormatTime
  reading). Rects are in 640x480 pixels. Please draw COSDInGameScreen per player into its view (the "splitscreen
  player 1/2 speedo / TFT position" tweaks, strings 0x00636314..), reading player p's values. Player 1's race values:
  `RaceProgress.player_positions[1]` / `player_states[1]` (added, the ai fills them); `RaceStatus` is still player 0
  only — tell me which per-player fields you need and core will add a `RaceStatus` per player.
- **Per-player status is in (2026-11-05)**: `velocity_game::game_mode::race_cars::PlayerRaceStatus(Vec<RaceStatus>)`,
  one entry per player (index = the screen's player, COSDInGameScreen +0x14): racing, lap / laps, lap time, position,
  race length, speed, rpm (one 0x001B6F10 call per player per update), gear; [0] equals `RaceStatus`.
- **Done (2026-11-05, frontend side only, core needs nothing)**: `hud::split_in` reads `GameViews` (players 2, +988 [0] /
  [1] rects, kind 4 = side by side) and `PlayerRaceStatus[1]` itself into `RaceHudInput.split: Option<SplitHudInput>`
  (player 0's race-wide fields + player 1's racing / lap / laps / lap time / position / race length / speed / rpm /
  gear). See "Split screen HUD" below.

#### Split screen HUD (two players, `byte_6F7157`)
- **OSD screens**: one `COSDInGameScreen` per player (Create 0x000F82F0, +20 = player) on the one game OSD manager
  `dword_6FD200`, drawn once over the whole screen. Create picks the item table: single `unk_6860F0`, top / bottom
  `unk_6861C8` / `unk_6862A0`, side by side (`byte_6F9180`) `unk_686378` / `unk_686450` (27 x (x, y), dump
  `research/fe_osd_layouts.txt`, `ig::layout`); player 1's screen skips items 20..26 (attract texts, logo, bars,
  network messages: `if (+20 == 0)`). Update 0x000F99E0 reads every value by +20 (`split.status` for player 1).
- **Hud configs**: OnGameInit 0x000E9FD0 loads `Load(name, p)` per player (errors "...for player 1/2. speedos will
  not be initialised."); Load 0x000EB9F0's split branch reads the "splitscreen player p+1" speedo position into block
  +400 p, TFT +800 + 224 p = tft lines [2+2p, 3+2p], map +1248 + 548 p = map lines [2+2p, 3+2p]
  (`HudConfig::split_player`; the "player 2 TFT position" label for player 0's rotation is a mislabelled read).
  Both players use the `HudCommand::Start` name (core sends "digione"; GamePlayer +392 of player 1 not wired).
- **Per view** (GameManager 0x000B9010 calls 0x000EA2B0(.., a3) per HUD view; dump `research/fe_split_osd.c`,
  `fe_split_cfg.c`): the hud slide steps once per call (twice a frame when split, kept), speedo +2940 + 420 a3
  (`speedo2` = +3360), the TFT block only `if (!byte_6F7157)` (no TFT in split), the map frame / screen at the
  view's map slot (0x000ECFD0 / 0x000ED080), light set = race vehicle a3's (0x00023F40). The map screen texture is
  the one `unk_6F5BC8` map (player 0's window). Speedos are 2D quads through the 4:3 interface camera, squashed
  into the view rect (`hud::into_view`); the 3D models of player 1 are a second `Osd3dView` on layer
  `HUD_3D_LAYER_P2` (23) with the bottom / right viewport; both HUD cameras use `InterfaceProjection` (fixed 4:3,
  bevy no longer refits the aspect to the viewport / window).
- STAND-IN: player 1's points / combo / multiplier / wrong way and nos / zone / turbo ratios are 0 (not in
  `PlayerRaceStatus`). If the views appear after `Start`, `run_hud` rebuilds the resources with the player count.
- **Name tags per view (ported 2026-11-06)**: GameManager 0x000B9010 runs the 3D OSD pass 0x000EF5F0 inside each player
  view (render flag 0x200000; the mirror's flags 0x08000620 lack it) with that view's camera, before the 3D huds and the
  2D OSD. `frame_slide` calls `render_3d` once with `name_tags.view_proj` (player 0's `TrackCamera`) and, when split,
  once with `name_tags.view_proj1` (`hud::camera_in`: the active `SecondaryTrackCamera` with `TrackView { player: 1 }`),
  each through `into_view(rect p)` (full 4:3 projection squashed, as the world) and `clip_to_view` (viewport 0x000B3C30:
  quads cut to the rect, uvs interpolated). Both screens own tag items (Create 0x000F82F0's tag loop is outside the
  `+20 == 0` gates, dump `research/fe_osd_create_f82f0.c`) and Update 0x000F99E0 calls 0x000F9340 per screen (distance
  to driver 0 for both), and 0x000EF5F0 draws every in-3D item of the manager, so each half draws both screens' tags
  at the same spot (doubled alpha: original, kept). `byte_68565C` (OSD bridge 0x00098470 gate) is only read (one xref),
  static 1, so the bridge applies for two players too. Test `capture_split_hud` (`split_hud_names.png`,
  `split_hud_side_names.png`: equal tag quads in each half, none crossing), `hud::tests::clip_to_view_cuts_quads_and_uvs`;
  copy `research/screens/fe_split_tags_capture.png` (one rival, each half's own camera). Game run (`SMOKE_SPLIT=1`,
  `APPDATA` scoped to a scratch dir with `PlayerNames=1`, log `research/fe_split_tags.log`, exit 0): shot
  `research/screens/fe_split_tags.png` shows no tag — no AI car is in either view at the 1c_split / start / opponents
  shots (the three AIs are off frame), so the in-game look is unconfirmed.
- Test `capture_split_hud` (`split_hud.png`, `split_hud_side.png`). Game shot `research/screens/fe_split_hud.png`
  (`SMOKE_SPLIT=1`, log `research/fe_split_smoke.log`, exit 0): 1st / 2nd, two speedos, each half's map frame; the
  views exist before `Start`, so no rebuild happens.

### Requests from graphics (split screen HUD areas, 2026-11-05)
- GameManager 0x000B9010 after the world views: for each HUD view (+988; two players: [0] top half = player 0, [1]
  bottom half = player 1, same rects as the world views) the camera manager's camera i is switched to mode 5 (the OSD
  camera) and OSD3dInterface 0x000EA2B0(…, i) draws that player's 3D HUD inside the view's viewport (when +36 is 6/7
  only outside the pause), then the draw lists are flushed and the camera restored; the 2D OSD (`COSDManager::Render`
  `dword_6FD200`, then `dword_6FD1FC`) runs once afterwards over the whole screen (its per-player layout is
  OSD3dInterfaceConfiguration::Load's `byte_6F7157` branch). The world views' rects per player are in
  `visos_render::view_render::SplitScreen` (`rect(p)`, x/y/w/h fraction); give the 3D HUD camera of player p that
  viewport. Note the original keeps the 4:3 projection in the half (squashed 2:1); graphics does the same for the world.

### Requests from graphics (name tags per view, 2026-11-06) — done (see "Name tags per view" under Split screen HUD)
- Graphics draws no part of the name tags (they are `COSDManager::render_3d` text items, yours). In GameManager
  0x000B9010 the 3D OSD pass `OSDManager 0x000EF5F0(dword_6FD200)` sits **inside the per-view loop** (after
  SpecularChars 0x00144DF0, `research/gfx/ss_b9010.c` line ~402), gated by render flag 0x200000: every player view has
  flags −1 (0x000B88B0), the rear-view mirror has 0x08000620 (no 0x200000). So with two players the name tags are drawn
  once in each half — projected with that view's camera, clipped to that view's viewport (`SplitScreen::rect(p)`,
  squashed 4:3 projection) — and never in the mirror. The item data (alpha by distance to driver 0, position) are
  updated once per frame by 0x000F9340 as now. Please give player 1's half its own name-tag pass with player 1's
  camera (`TrackView { player: 1 }` / `SecondaryTrackCamera` camera; `RaceHudInput.name_tags.view_proj` is player
  0's only), and check whether 0x000F9340 / the OSD bridge 0x00098470 apply the items for two players
  (`byte_68565C`).

### Requests/notes from core (race points for the results table, 2026-11-06)
- Per-racer race vehicle +84 points for OSDManager 0x000F09C0 ("prestige won" column, `dword_6F7244`):
  `velocity_game::game_mode::race_cars::RacerPoints` (resource, `Vec<u32>` by place, index = place − 1 from the ai's
  `race_order` = `GetVehicleInPosition`), getter `points_in_place(place)` (1-based, 0 when no racer). Filled every
  frame by the binary's `hud.rs::race_manager_status`, cleared on leaving InRace (so read it while the results OSD
  runs in PostRace). The player's own +84 is `RaceStatus.points` / `PlayerRaceStatus[p].points`.
- `PlayerRaceStatus[1]` now also carries points / combo / multiplier / wrong_way_time / nos / zone / turbo:
  `hud::split_in`'s two STAND-IN blocks (no points / combo / wrong way, no nos / zone / turbo for player 1) can read
  them.

### Requests from core (3d interface clip planes, 2026-11-06)
- `osd_3d_interface::INTERFACE_NEAR / INTERFACE_FAR` (1 / 600) are the "Graphics.Camera" Near/FarClipDistance tweaks
  (`dword_6850A4` / `6850A0`), now live: read the 3d interface camera's planes from
  `GameCameras.manager.players[p].interface3d` (`near` / `far`, set by 0x000B75D0 and the 0x000C0A90 listener)
  instead of the constants. Same values at default.
- Done: `hud::run_osd_3d` writes the view's `InterfaceProjection` near/far each frame from the player's
  `interface3d` camera (`manager.clip` near/far while the cameras don't exist); the constants are only the spawn values.
- "System" tweak order: not a frontend change. The frontend registers its System tree one frame after the
  TweaksReset handler; making it first (original: SetGameMode adds it after the mode init) needs core's registration
  hook (`TweakRegistrars`, see docs/core.md "System order"). Left to core.


## Coverage audit (2026-11-07)

`tools/coverage_fe/{cov,tag}.py` + `tools/ida/fe_cov_names.py` (no shared tools/coverage existed yet) -> `research/frontend_coverage.txt`
(unported list, largest first, with tag, name, strings, `noted` = address appears in a comment, checked unported).
Scope: Feck2/*, Osd*, OSDMap, GameMenuManager, FontHelper, StringManager (both), Fonts, VsMovie = 3250 functions, 770,780 B.
Rerun: `python tools/coverage_fe/cov.py` (summary), `ctx.py` (each commented-but-uncredited address with its enclosing
fn / origin line -> `research/fe_cov_mentions.tsv`), `tag.py` (report; header from `research/fe_cov_hdr.txt`, edit by hand).
- **Origin line format (what cov.py credits)**: `/// File.cpp 0xA / 0xB / Other.cpp 0xC — text` (also `//` inside a body):
  functions merged into one Rust fn (AssignEvents, Init, ctor, inline functors, per-part creates) are listed after the
  first address. `/// Name 0x...` without `.cpp`, `(+ 0x...)`, `0xA -> 0xB` and bare `/// 0x... —` lines are NOT credited.
- **Tightened 2026-11-07**: all 449 `mentioned` functions checked one by one: 340 were genuinely ported (malformed origin
  lines: `/// FEHangoutProgression 0x..`, bare `/// 0x..` in the options / tips / multiplayer / car info / VsMovie files;
  merged AssignEvents / Init / ctors / functors / overlay part creates / OSD pause screen callbacks / speedo gauges) and
  now carry origin lines; 109 (35 KB) stay unported (`noted`): CGameMenu frame / title / scrollbar (no-frame menus never
  draw them), submission screenshots, gauges list, decals / mix-your-own / debug sliders, LoadFoyerLights (graphics'
  parser, no FOG), FE3DVehicles paint / turntable / part loading, track previews, CHARS model streaming, online bits,
  attract-mode functions (core's `CAttractMode`), physics getters, object range bounds.
- Coverage: 631 funcs / 309,700 B (40.2%) -> 971 / 420,756 B (54.6%) after the tightening -> 992 / 429,570 B (55.7%)
  with the R2 statistics list below -> 1050 / 450,742 B (58.5%) with the showroom / pink slip screens -> 1054 / 452,929 B
  (58.8%) with the logo wall material swaps and the front wheel steer.
- Remaining big non-online / non-debug / non-editor gaps checked 2026-11-07: 0x0025B4D0 / 0x0025BA40 wheel matrices and
  0x0025BF80 door parts (wait on graphics' part entities), 0x0025CB20 / 0x0025CF40 part model loading (wheel / brake /
  tyre models on the FE car; starts the wheel-change bounce 0x0025EAB0), 0x00205F80 / 0x00206DB0 "StatsScreen" (created
  by CreateAllScreens, fills hard-coded placeholder rows "42secs" / "152mph" / "100,000": a mock-up, no navigation
  to it found), the QuickRaceText / DummyGame / Tips region AssignEvents functions (0x0022EE10, 0x0022BDF0, 0x0022DBC0,
  0x00209F10, 0x002426C0: layers of the online screens sharing those object files).
- Unported by tag after the tightening (count/bytes): logic 369/141k; online(XBL, cut) 553/93k; getter-trivial 912/41k;
  editor-screen (AlloyEditor) 137/31k; gameplay-UI 128/22k; platform 129/11k; debug 47/10k.
- Caveat: funcmap region attribution is coarse (e.g. FETipsScreen region holds NetworkServerDetails/Loading screen code,
  FECK2ElementStaticText holds screen update bodies); tags are heuristic (no decompile), "mis-attributed" detects via assert path only.
- Top unported (addr size purpose): 0x00274250 2195 FE3DCamera debug free-roam / character orbit camera (UserInput
  actions 21..46, `byte_6C1550`, DepthOfField focus; only with the "Free Roam" tweak); 0x00241E20 2085 NetworkServerDetails;
  0x0025BF80 2070 FE3DVehicles door-part animation (t in 0..1; needs graphics' car part transforms); 0x00270B10 1704
  FE3DCamera layer setup; 0x0021D010 1490 FEHangoutScreen layer events; 0x00264450 1294 = AssignEvents of the
  FE3DCharacters debug character screen (14 functors 0x264960..0x266670); 0x00262730 1212 = character model streaming
  (`\Chars\CHARS_T.vcd`, GDL_RelocateModel: graphics' SpawnSkinnedModel does this); 0x0025B4D0 / 0x0025BA40 FE3DVehicles
  wheel parts; 0x0022EE10 / 0x0022BDF0 QuickRaceTextScreen layers; 0x00206370 FECK2Object layer functor setup.
- **R2 car statistics list (ported 2026-11-07, `fe_car_stats_list`; dumps `research/fe_stats_list.c`,
  `fe_stats_helpers.c`, `fe_stats_data.txt`, callers `fe_stats_xr.txt`)**: 32-byte object (+0 row list, +8 layer = the
  overlay's +364 via 0x002105B0, +12 setup, +16 "HighlighterGraphic" FE_track_stats_box at (-30, 145), +20 top, +24
  bottom, +28 y). Fill 0x002897A0: [prices: CARSELECTION_CAR_BASEMODEL, CARSELECTION_VALUE + vehicle +0x10], category 1
  with sub categories 1..4, category 2 (sub 0), sub 5 (colours), decal category when any decal, [prices: total = vehicle +
  0x00185140]; rows 0x0028B110 = label (50, y) / text (250, y) / optional "%d" value, then texts moved right of their
  labels (+10) and values right-aligned at label x + 200. Installed parts 0x0028A480 by `GetPartCetegories` tables
  (`dword_687850` / `dword_6878E0`) and `GetPartTypeLabel` (`InitPartLabels` 0x00185420 tables copied as consts: physics'
  `CVehicleDatabase` has no label tables). Visible band y 150..350, scroll 0x0028AEC0 by 5 per press / held frame.
  Car select (stages 1/2, R2 = MenuR2 = BLACK, 0x00224BE0): no prices, overlay up/down arrows at (200, 155..350); up/down
  scroll it instead of the swatches; select / cancel / option 1 hide it. Garage (0x00247460): with prices, no arrows.
  Option 2's refresh after a randomise is not wired (automod not ported). Test
  `fe_car_stats_list::tests::label_tables_match_their_strings`; `fecapture garage` -> `garage_stats` / `_scrolled` / `_off`
  (log `research/fe_stats_capture.log`, exit 0); shot `research/screens/fe_stats_list.png` (garage 300ZX: Base Model /
  Value 20000, Performance: NOS Level 2 200, Exhaust Stock 0, Suspension Level 3 300, Tyres VS Medium 100, rows below
  y 350 hidden). Car select's list (no prices, arrows): `fecapture` default script `scene_car_select_stats` =
  `research/screens/fe_carselect_stats.png` (garage 300ZX: Performance rows, up / down arrows, CAR SETUP panel).

#### Car showroom (`fe_car_showroom_screen`, `gCarShowroomScreen` `dword_75F448`, 4276 B; dumps `research/fe_showroom*.c`,
`fe_showroom_data.txt`, `fe_doors.c`, `fe_2728e0.c`)
- Reached from the hangout's showroom character (0x0021C210, already wired). Elements: "CarGraphic" (dynamic 2D at (60,340),
  the car's `DisplayLogo_<name>` 256x64 from `<vehicle path>\VEHICLEF.VCD`), "UnlockText" (font 2 at (70,300), black,
  CARSHOWROOM_UNLOCKCRITERIA_PRESTIGE with dPrestige = unlock value, half of it for hidden cars). Swatches at (140,345).
- **List mode** (camera (-5.28,1.61,-0.5) -> (-7.28,0.4,-3.71), Activate 0x0021DD54): left/right = tier (0x0021F650: next tier
  with cars, all cars counted when `byte_6877F0` "Unlock ALL Vehicles" — static 1 in F3_D — else non-hidden; clamp 0..7 keeps
  the old tier), up/down = car (0x0021F780, clamped), overlay menu style 2 = the tier's cars (hidden = GENERAL_HIDDEN
  disabled), title VEHICLE_TIER<n>, 2D background `Showroom_<name>[_hidden]` (shown on a tier change, hidden on a car
  change with dir 0 — original). A (not hidden): `dword_6F90CC` = `GarageVars.showroom_vehicle` -> SHOWROOM_SINGLE spot
  (carSel Vehicles.txt (-9.284,0.38,-3.709) rot 67), the camera flies (vector move, linear, 1.2 s `0x00272110`) to the
  spot's camera; the menu is built and slid in once less than half the move is left (0x0021EBF0), the move ends within
  0.1² (car mode: doors open, orbit pitch-only `orbit_lock` 2 with limits 0..20° `set_pitch_limits` 0x002729D0, orbit from
  the spot camera 0x002727D0 `orbit_from`).
- **Car mode**: menu PURCHASE (disabled while locked) / TESTDRIVE (always disabled; Select would copy the shown car to
  `dword_6F724C`) / CHANGECOLOUR (disabled for tier 7 team cars), static init 0x0042BA40; title = car name. X toggles the
  doors (nav 2 OPEN/CLOSEDOORS). The right stick's horizontal turns the car (turntable 0x0025B090: stick³·2 °/frame, let
  go: hold 150 frames then 0.2 °/frame eased in over 50; per FE frame like the original), the vertical pitches the
  camera. B: colour mode -> default colour back; dialog -> closed; else back to the list (the car eases back from its
  angle to the spot's: `base + wrap(base - saved) · sin(1 − left·90°)`, `left` = the move's seconds left — kept), and
  the car is hidden at the move's end. B in the list -> PREVIOUS after the 20-frame transition.
- **Purchase** (0x0021E2D0, dialog options [GENERAL_YES, GENERAL_NO], selection 0 = yes): ask 0x632C0669 (idCarName,
  dPrice = vehicle +0x10; owned: 0xB18EB24C) -> cash < price: CARSHOWROOM_INSUFFICIENTCASH, garage count >= 6:
  CARSHOWROOM_SLOTSFULL, else cash -= price, `AddGarageVehicle` (GameVars 0x000C20F0, `GarageVars::add_garage_vehicle`),
  `CareerRequest::EvaluateUnlocks` (EventDatabase 0x0008F020, run by `career_out` after `garage_out`), ask 0xBADFAC82 (BOUGHT,
  "current car?") -> yes: `dword_6F71C4` slot, `dword_6F71C0` hash, `CopyShownCar`; race vehicles (tier not 6/7/8) then
  ask 0x12C50AF2 -> yes: CarCustomisationScreen with `custom_slot` = the bought slot (0x0021EAE0).
- FE3DVehicles additions: `base_rotation` (+1252), `door_t` (+2384) / `doors_open` (+2576), flags +15694 / +15695 / +15698 /
  +15699, `spin_hold` +15704, `door_window` +15708/+15712; `set_rotation` 0x0025DB50 (wrap once, steps the doors),
  `open_doors` 0x0025D9A0, `set_doors` 0x0025D940, `update_door` = the state part of 0x0025BF80. `foyer::foyer_car_poses`
  writes each foyer car root's transform every frame (rotation dropped from the respawn key). Door sounds
  (`VsAudio::PlayCarOpen/Close`, wired 2026-11-07: `update_door` pushes `door_sounds` — open when the part moved this call and t in 0..=0.2 (0x0025C1CE), close when a closing part reaches 0 (0x0025C3A6), once per part step as the original — and `lib.rs` sends `AudioCommand::CarOpen/CarClose`).
- Part matrices (2026-11-07, `foyer::foyer_car_parts`, every frame from `CarRenderParts`): door k = `T(h)*slerp(I, open^-1,
  door_t[k])*T(-h)`; wheels `T(pivot - (0,bounce,0)) * RotY(steer) * base_rot` (f_l `-steer`, f_r `+steer`, rear 0,
  0x0025B4D0), discs likewise with scale -1 x on the right (0x0025BA40). `FE3DVehicles::step_doors` (0x0025BF80 state, each
  frame from `Frontend::frame`) replaces the stepping only in `set_rotation`. Wheel change bounce: `start_wheel_bounce`
  0x0025EAB0 (flag +15716, start +15720; from `apply_part` for wheels = 0x0025CF40 and body part 12 = 0x0025CB20) and
  `update_heights` 0x0025E7D0: `t = (now-start)/time`, `bounce = sin(2 pi osc t) * height / (trunc(t*osc)+1)`, slot y =
  ground + wheel +0x0C/2 + bounce, flag off and bounce 0 when t > 1; tweaks `BOUNCE_TIME/HEIGHT/OSCILLATIONS` 0.5 / 0.005 / 2
  (mirrored by `fe_tweaks`). The car respawn key now holds the spot without bounce and the brake part names. Brake (0x25D020)
  and tyre (0x25D110) changes (dumped 2026-11-07, `research/fe_dc_25D020.txt`/`25D110`): brake = reload the 2 disc/pad models (LoadBrakeModel 0x25A7F0) + DrawWheelsAndBrakes, no bounce; tyre = TyresT.vcd texture -> CarRender 0x118910 sets "_tyre" on 4 wheels, no bounce (needs `CarSpec.tyre`, requested from graphics). Wheel change 0x25CF40 = LoadWheelModel 0x25A5A0 x2 (front = WHEELS.BIN +0x1C stem, rear +0x20, "_High" in the FE) + DrawWheelsAndBrakes + bounce; `foyer.rs` sets `CarSpec.wheel` from the setup wheel's +0x1C and keys the respawn on it. `fecapture wheels`: the 300ZX has only a "Stock" wheel style, so frame 450 fits `wheel_001_size3_20` directly. Background fallback with no track = area 1 verified
  (`dword_6F7144` null check at 0x2252D0-region callers, fe_logo_dc.c / fe_carsel.c).
- Not ported: the screenshot tweak mode (0x0021FFB0, `byte_75F44D/E/F`), the CarGraphic element's vtable +4 call before
  texture changes, the FE manager loading flag (+112; loads are synchronous), camera L/R held flag +29 (kept, unread).
- Tests `fe_3d_vehicles::tests::{turntable_holds_then_eases_back_in, doors_ease_over_two_seconds}`. `fecapture showroom`
  (`jumptoscreen CarShowroomScreen`) -> `showroom_*.png`.

#### Background material textures (logo wall, 2026-11-07; dumps `research/fe_logo_dc.c`, `fe_gap_dc.c`)
- FE3DCamera 0x0026F7A0 = background element `index` (asserts <= 7); its vtable +76 (FECK2Element3D vtable 0x0065AC90)
  = 0x00298690: TextureManager 0x0029F650 by name, else load the given vcd (GDL 0x0009D760) and take the texture, then
  `FindShader(model, materialName)` and set that shader's stage 0 texture handle. Lasts as long as the element.
  Port: `FoyerState::set_material_texture` (list `material_textures`, one per (background, material)) +
  `foyer::foyer_material_textures` (after `run_foyer`): backgrounds are built with `Builder.records`. FindShader
  0x0012AC80 walks objects (model +36 count, +40 96-byte records, +84 prim list of 16-byte {shader id, kind}) and takes
  the first shader whose name **contains** the material (`strstr` 0x003DEE50, case sensitive: the shaders are
  `carlogo_01_0`, `polaroid_06_0`, ... so an exact match finds nothing); every prim of that shader gets a cloned
  `XboxStageMaterial` with the new texture; dropped swaps restore the built material; a texture missing from the vcd
  falls back to the background's own pack, else untextured + warning.
- Car select (0x00225A80, from ChangeTier 0x002252D0 and ShowTier 0x00225A30 when shown; only in stage 0): the area
  model (`dword_6F7144` +136 = track area, 1 without a track; LA/London/Tokyo.xmf all carry the materials) gets
  `carlogo_0<n>` = `DisplayLogo_<name>` and `polaroid_0<n>` = `Polaroid_<name>` from `<path>\VEHICLEF.VCD` (vehicle
  +0 / +4; garage tier = `dword_6F71A8` hashes), `polaroid_bg_0<n>` = FE.vcd `polaroid_bg`; slots count+1..6 =
  FE.vcd `general_OFF` (polaroid, polaroid_bg, carlogo). > 6 cars asserts (logged). ChangeCar 0x002254D0 sets
  `carlogo_current` / `polaroid_current` of the chosen car (after the info panel and ToggleTransmission).
- carlogo_07..09 exist in the models but are never set (stay as authored).
- Shot `research/screens/fe_carselect_logos.png` (`fecapture` default script `scene_car_select_tier`, garage tier:
  300ZX / RX-8 / NSX logos and polaroids, slots 4..6 general_OFF; before: six authored "ALPINE" logos and white
  polaroids). Log `research/fe_logo_capture.log` (exit 0, no missing textures).

#### Front wheel steer (FE3DVehicles +1256)
- 0x0025E040 `set_steer(slot, deg)`: ignored past the slot count, wrapped once (> 360 / < 0), then SetRotation
  0x0025DB50 with the current rotation when loaded. Callers: HoActivate 0x002198F8 and HoUpdate 0x0021AB22 (every
  update, after the menu slide) 28.0, PinkSlip activate 0x00245990 28.0 (doc said 20 before: wrong), 0x0024EF80 (tyre
  list of PartList, ported in `part_list`) 35.0. Each is followed by 0x0025E7D0(slot) = y = ground +1260 + wheel +0x0C / 2 + bounce +1264
  (bounce: FE3DVehicles +15716 flag / +15720 start, set only by 0x0025EAB0 from the unported part loading 0x0025CB20 /
  0x0025CF40; `sin(2π·osc·t)·height/n` with the AlloyEditor tweaks), so only the steer is kept. Not drawn: needs
  per-wheel entities (request in graphics.md "front-end door parts", wheels bullet; matrices 0x0025B4D0 / 0x0025BA40,
  dump `research/fe_wheels_dc.c`).

### Requests from graphics (foyer default lights, 2026-11-07) — done (`foyer.rs` uses `vs_render_defaults`)
`FECK2Element3D::LoadFoyerLights` 0x298CD0 keeps VsRender's start-up set 0x583F0 when the foyer has no lights.txt
(garage, plynth, Network, empty, hangCE, CutScene, props, Test). `foyer.rs:223` should use
`.unwrap_or_else(visos_render::vs_render::VsLights::vs_render_defaults)` instead of `.unwrap_or_default()` (ambient 1,
no directionals = flat full-bright). See graphics.md "Light sets".

### MTV overlay call sites (wired 2026-11-07)
`FoyerState.mtv` queues `MtvCall::{Start{seconds,no_glitch}, Stop{immediate}}`; `foyer::foyer_mtv` drains it onto
`MtvEffect` each frame. 0x00275E50 is really a camera SHAKE (`FoyerState::shake(time, ax, ay)`: +388/+392 time,
+396/+400 amplitudes, +404 follow flag when time < 0 = move time): sets the fields and `mtv.start(time, false)`.
0x00275F10 (`update_shake`, per frame): left <= 0 -> nothing; follow flag and move finished (+356) -> left = 0 +
`mtv.stop(false)`; else offset = (-ax + rand(0..2ax), -ay + rand(0..2ay)) * left/total in camera space
(right = up0 x fwd, up = fwd x right), eye and look-at both moved, left -= dt. Callers: car select OnEnter 0x223695
(0.1) and UpdateCamera 0x224FE8 (0.02, `0x3CA3D70A`), time -1. MoveCamera 0x22B3C3 (text screen) and 0x227680 (track
logo, `_p`/`_u` FETRACKS.vcd) are not ported (3D track previews). ChangeCar 0x225820 (inside the vehicle-found
block) and ShowTierLogos 0x225E84: `Start(0.1, true)`. `fecapture quickcar` = QuickRaceCarSelectScreen script
(shots `quickcar_*`, `research/screens/fe_quickcar_*.png`). The shots were almost black (mean max-rgb 7) because
`show_hangout_and_track_area` fell back to area 0 (hangout alone, camera 7 faces a plain wall) when no track is picked: the
original's `dword_6F7144` +136 is 1 (LA, the logo wall model) then; fixed (mean 48 / 25 after the car goes up). Lights
were fine (quickR lights.txt: ambient 0.4, no PtLights so ClosestLights is a no-op) and the MTV overlay is not involved.
0x0024EF80 is the tyre list of PartList 0x24E1B0 (icon entries + `set_steer(0, 35.0)`): ported in `part_list`.
Wheel bounce 0x25EAB0 / part loading 0x25CB20 still wait for per-wheel entities (graphics).

### Requests from graphics (MTV glitch overlay start/stop, 2026-11-07) — done
The front end's "MTV" overlay (object `byte_720208`, LightStreak.cpp 0x1300D0..0x1310E3) is ported as
`visos_render::mtv_effect::MtvEffect` (resource, in `RenderPlugin`); it updates and draws itself every frame while a
`FrontEndCamera` exists. It needs the three call sites (all `ResMut<MtvEffect>`):
1. `FE3DCamera` 0x275E50 (the camera move: OnEnter 0x223695 / UpdateCamera 0x224FE8 of CFEQuickRaceCarSelectScreen,
   CFEQuickRaceTextScreen::MoveCamera 0x22B3C3): after it stores the move time (+388/+392, a negative a2 → +228),
   `mtv.start(time, false)`.
2. `FE3DCamera` 0x275F10 (from FE3DCharacters::Update 0x271B46): where +392 > 0 and +356 && +404 it zeroes +392,
   `mtv.stop(false)`.
3. CFEQuickRaceCarSelectScreen::ChangeCar 0x225820, 0x225E84 (FEQuickRaceCarSelectScreen 0x225A80) and 0x227744
   (0x227680): `mtv.start(0.1, true)` (0x3DCCCCCD, flag 1 = tint only).
Nothing else (init = the resource default, as FrontEndInit 0xB587D). Test view: `trackview --track ExecTest --fecam --mtv`.

#### AudioPlaylistScreen (DJ booth) — PORTED 2026-11-07 (`quick_race/fe_audio_playlist_screen.rs`, `feck2/playlist_link.rs`)
- Channel: `Feck2.playlist: PlaylistLink` = mirror of `VisosAudio.game.tracks` (`VsAudioTrackList`, edit lists included) and the
  soundtracks (name/kind/shuffle/song names) + `requests: Vec<PlaylistRequest>` (`PlayTrack{soundtrack,song}`, `SaveLists`) + `list`
  (screen +0, set by the hangout's DJ select 0x0021C370) + own `CrtRand` for Shuffle. `run_frontend` does `sync_in` before the frame
  (skipped while `dirty`), `sync_out` after (mirror -> audio when a screen set `dirty`; current_list/track stay audio's), then
  `PlayTrack` -> `AudioCommand::PlayTrack`, `SaveLists` -> `VsAudio::get_lists()` (0x00105E10, Cancel on the first menu).
  fecapture/tests fill the mirror with `VsAudioTrackList::load()`. The hangout DJ menu (0x0021C2A0) lists `list_count()` lists.
- NextTrack(1, node) 0x0004F4B0 -> PlayTrack(1, node, 0, 0): mapped to `PlayTrack` of the node's (soundtrack, master song).
- Not ported/deviations: FE3DCamera layer calls 0x00271CF0/0x00272940 folded into set_position_look_at(char 3 SELECT camera);
  BuildList/CommitList/Shuffle/Add/Remove only for fixed lists (original indexes past its array for soundtrack lists); `Select` in
  state 1 for a soundtrack list opens the edit menu like the original. Test `capture_audio_playlist`; `fecapture playlist`.
- Original notes (scope as found):
- Code 0x0022B7E0..0x0022D7B0 inside FEQuickRaceTextScreen.cpp's object range (the real QuickRaceTextScreen ends 0x22B470).
  Screen object 44 B (`gAudioPlaylistScreen` `dword_75FF98`, create 0x22BA30 / free 0x22BAB0 / set list 0x22BB20): +0 list
  index, +4 screen, +8 layer, +12 entered, +16 leave countdown (-1), +20 next screen crc, +24 state (0 list kind/shuffle row,
  1 track list, 2 add-by-id list, 3 edit menu, 4 add-by-index list), +28 selected track node, +32/+36 two "mainmenutext"
  texts at (x 0x43880000?, y 290/310), +40 edit selector. Create 0x22BB30 (screen "AudioPlaylistScreen", layer
  "AudioPlaylistScreen_2DLayer"); layer events 0x22BDF0: 9->0x22C250 Enter (camera, texts off), 10->0x22C430 Start (nav
  SELECT/BACK, heading HANGOUT_DJBOOTH, menu window 5), 11->0x22C310 Activate (foyer background, camera), 25->0x22C7C0 right,
  31->0x22C770 left (both -> 0x22C560 step: state 0 row 0 = list kind +-1 wrap 0..6, row 1 = shuffle, states 2/4 swap lists),
  46->0x22C880 Select, 55->0x22CA90 Cancel (4->2/ 2->1, 1->0 + CommitList(store), 0 -> PREVIOUS after 20 frames), 21->0x22CB90
  Update (countdown, GotoPreviousScreen / GotoScreen), 58->0x22CC90 option (1 -> 2 = add by id list 0x22D310), 61->0x22CCD0
  Shuffle(list) + refresh, 22/28->0x22CD20/0x22CD50 move the edit selector (0x22CD80). Menus: 0x22CED0 (state 0 rows), 0x22D080
  (track list: fixed lists from the working list nodes, soundtrack lists from the soundtrack's songs), 0x22D310 (named tracks of
  the master list, 268 B entries), 0x22D3F0 (all songs of soundtracks 1..). Kind names 0x22B7E0 (FRONTEND INGAME LONDON LA TOKYO
  NEVER) / shuffle 0x22B950 (RANDOM / SEQUENTIAL). Select calls VsAudio::NextTrack / PlayTrack (0x22C880 state 3).
- The audio state lives in a bevy resource (`VisosAudio.game.tracks` / `.soundtrack`), the screens in plain `Feck2`: needs a
  request / mirror channel (like `Feck2.music`) before the screen can be ported; also needs the overlay menu's text entries
  (`AddTextEntry`) and FE3DCamera CharacterCamera / SetFOV calls.

## Coverage triage (stage 4, 2026-11-07; dumps `research/fe_tri_dec.c`, `fe_custom_region.c`, `fe_ocfgload.c`)
Biggest ~60 unported functions outside online, checked in IDA (decompile + xrefs). Classes: R = reachable real gap,
D = debug / editor only, C = ctor / teardown / template / layer-setup of a cut screen, P = ported elsewhere, X = other crate / XDK.
- **R, ported now**: 0x002515D0 / 0x002514E0 / 0x00251550 customisation screen **gauges list** (Styling -> Gauges, `gauges_list` in
  `fe_car_customisation_screen.rs`): lists `\OSD\HUDS\*.vcd` that have a `-config.txt` (entry text = the config's `hud name`, icon
  `<STEM>-THUMB` from the hud's own vcd), selection = the setup's hash; pick (select at level 5 -> Purchase 0x0024F860 branch ->
  0x002514E0) stores `VsStringAnsi::Hash(stem)` in the setup copy's +0x188 (`VehicleSetup.unk_188`, saved in the profile) and goes to
  level 2; cancel 0x00251550 puts the bought setup back and drops the list. Original quirk kept: only the copy gets the hash (a later
  part purchase copies the shown setup over it). The in-race side: `OSD3dInterfaceConfiguration::Load(hash)` 0x000EB001 = hash 0 ->
  "digione", else the `\OSD\HUDS` stem whose hash matches -> `HudConfig::name_for_hash` (+ `HudConfig::list`), the hash being
  `GamePlayer +392` (= setup +0x188). Core still sends `HudCommand::Start{hud:"digione"}` (request in docs/core.md). Test
  `speedo::tests::hud_hash_round_trip`; `fecapture gauges` -> `gauges_list.png` / `gauges_list2.png` / `gauges_picked.png` (log
  `research/fe_gauges_capture.log`; shot: Gauges / "Defi Analogue" with the four hud thumbnails).
- **R, ported (2026-11-07)**: car customisation **decals** (Styling group 2) in `fe_car_customisation_screen.rs`: `decal_menu` 0x00250C20
  (overlay list "off" + every decal of the kind, kind = item 0 -> 0, 1 -> 1, 6/7 -> 2, else 3), `decal_step` 0x00250EE0 (left/right,
  clamp, `set_setup_decal` on the shown setup with the slot's existing flags; Option2 / Y = flip flags 1<->2, logo items only),
  `decal_slot_step` 0x002512E0 (up/down, per-item slot `decal_slot[item]`, copies shown -> as-bought), `decal_status_update` 0x002513F0
  ("SLOT n" text only for items 2 and 4, original quirk), `shown_decal` 0x002509A0, `decal_index` 0x00250BB0; purchase checks for part
  types 27..34 (installed = chosen index equals the as-bought slot's, never "free", value = decal +0x0C word, name = hash). Setup flag
  tables are the setup's own (+2276 = FE3DVehicles +1824 + 0x1C4). Slot counts: graphics request (`decal_slots`, docs/graphics.md),
  default = table sizes. `fecapture decals` -> `target/fe_captures/decals_*.png`, shots `research/screens/fe_decals_list.png`,
  `fe_decals_slot2.png`. Foyer car gets `CarSpec.tyre` from `setup.tyre` (respawn key), shot `fe_tyre_texture.png`.
- **R, not ported (next)**: mix-your-own paint (item 4 of the paint group, 0x002505F0 mode `colour_slider_mode`, hue 0..1529 step 4 via
  0x0024B3E0, shade -255..255 step 2 via 0x0024B480 -> 0x0024C750 moves the two arrows and writes the setup colour +0x9C/+0xA0 then
  VehicleDatabaseColours 0x00187BA0 / 0x00187880 = hue wheel to rgb; left/right *held* events 0x0024B790/0x0024B820 -> 0x0024B9C0, up/down
  0x0024BA20 swaps the row and moves the highlighter to y = 100 + 20 * row). The six RGB/HSV tweak sliders 0x002539A0/0x00253B00/0x00253CD0
  are debug (`byte_6F905A`). Physics' `Colour` lacks +0x9C/+0xA0 and 0x00187BA0 (noted "not ported" in vehicle_database.rs).
- **D**: 0x00274250 / 0x00272AF0 / 0x00274EE0 / 0x00274B40 / 0x00275F10 / 0x00276810 / 0x00270B10 FE3DCamera free-roam + orbit spline +
  camera location arrays (`Free Roam` tweak, asserts `mpCameraLocations == NULL`); 0x002528D0 submission / screengrab TGA writer
  (`Screengrab_%s`, `Submission_%s`); 0x00255BB0 / 0x002545C0 / 0x00256350 / 0x00282D70 / 0x00254C30 / 0x002541F0 AlloyEditor
  screens; 0x000B0270 / 0x000B1950 tweak-menu item builders (`GameTweaksMenu`, core's debug menu); 0x00264450 / 0x002634D0 /
  0x00265B60 / 0x002654C0 / 0x00266070 FE3DCharacters debug character screen.
- **C (cut screens sharing object files)**: 0x0022EE10 / 0x0022BDF0 / 0x0022DBC0 / 0x0022D7D0 / 0x0022E4E0 FEQuickRaceTextScreen
  region = online screens' layers (OnlineNewProfile, SignIn, LoadAndSave); 0x00209A30 / 0x00209F10 DummyGame region = OnlinePlayerOptions;
  0x002426C0 / 0x00242DB0 / 0x00241E20 Tips region = NetworkServerDetails; 0x00206370 FECK2Object layer functors; 0x00205F80 /
  0x00206DB0 / 0x00207440 / 0x00207730 "StatsScreen" mock-up (hard-coded "42secs" / "152mph" rows, nothing navigates to it);
  0x00293810 / 0x00293E40 FECK2Element2DLine region = online voice recorder (VoiceRecordSprite / VoiceDoneButton);
  0x00281470 FE2DOverlay invitation_Received (online); 0x000ABB60 / 0x000AB230 / 0x000AADF0 CGameMenu scrollbar / frame / title
  (no-frame menus never build them).
- **P (already ported elsewhere)**: 0x000F3A70 OSDManager speedo load (`-base`, `-revs`, ..., shaders `speedo_base` / `speedo_font`) =
  `osd/speedo.rs` `HudConfig` + `SpeedoTextures`; 0x000F3F60 SetupShadersForGauge = the gauge quads' blend modes there;
  0x00298CD0 LoadFoyerLights = graphics' parser used by `foyer.rs`; 0x000C9C10 HeatHaze_LoadBundle = core's `timeline_heat_haze`
  (bundle read in `visos` `bundles_all` test); 0x0025B4D0 / 0x0025BA40 / 0x0025BF80 / 0x0025CB20 / 0x0025E250 FE3DVehicles part
  matrices / doors / models = `foyer::foyer_car_parts` (see "Car showroom").
- **X (other crate / XDK / platform)**: 0x000C8190 + 0x002EC810 StringManager.cpp range = XDK / CRT switch tables and string
  registration of the XBL string pool (mis-attributed); 0x002F6040 / 0x002F3160 VsMovieStreamXBOX D3D / stream (bink is software);
  0x000B3170 CAttractMode::GameUpdate = core's `CAttractMode`; 0x002999B0 FECK2ElementCamera (`pCamera` layer camera hook,
  FE3DCamera does it).

### Mix-your-own paint (2026-11-07)
0x002505F0 `colour_mode` (on: hue/shade from setup copy colour 0 if its hash == 0 else 0/0; off: hash != 0 -> copy colour back, else saved hue/shade re-applied), held left/right 0x0024B790/0x0024B820 -> 0x0024B9C0 (`mix_hold`: row 0 hue +-4 wrapping 0..1529 = 0x0024B3E0; row 1 shade +-2 clamp -255..255 = 0x0024B480), up/down swaps `slider_row` (0x0024BA20), 0x0024C750 arrows x = 0.083 * hue + 184 / shade / 4 + 248. FE3DVehicles 0x0025E170 = hash 0, kind 0, hue, shade on colour 0 then 0x00187BA0 (`update_mix_colour`); the car paints from the packed +0x34/+0x38 of such a colour. Quirk kept: arrows keep stale positions until the first move. Capture `fecapture mix` (log `research/fe_mix_capture.log`).
