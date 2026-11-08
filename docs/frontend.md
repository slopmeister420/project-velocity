# Frontend / UI - INDEX

_owner: frontend agent. Details live in `docs/frontend/details.md` (the full former frontend.md, ~140 KB): NEVER read it whole; grep it
by heading (`grep -n '^###' docs/frontend/details.md`, then `Read` with offset/limit)._

## APIs other crates use (details: "Talking to the frontend")
- `velocity_frontend::FrontendPlugin`; boots at `MovieScreen` unless `--track` or `VELOCITY_NO_FRONTEND`. Drawn as a 640x480 cpu image on its own `Camera2d` (order 10).
- In (Bevy messages): `FrontendCommand::{Start("<Screen>"), Stop, ReturnFromRace}` (ReturnFromRace is sent by `watch_game_mode` itself); HUD: `HudCommand::{Stop, OpenPauseMenu}`.
- Out: `FrontendRequestMsg(FrontendRequest::{PlayIntroMovies, SetControllerMask{player,mask}, StartQuickRace, Exit})`; on StartQuickRace game flow reads `Frontend.fe.quick_race` (`QuickRaceSettings`).
- HUD inputs: `RaceHudInput` (`.split: Option<SplitHudInput>`), core's `GameViews`, `PlayerRaceStatus`, `RaceProgress`; career via `career.rs` / `gm.vars`; movies via `Feck2.movies`, `MovieAudioOut` lives in velocity_audio.
- Tweaks: `fe_tweaks.rs` registers on core's `CTweakManager` (`gm.tweaks`). Captures go to `target/fe_captures`; `fecapture` bin, `trackview --fecam`.

## OPEN requests
- From graphics (door / wheel / disc parts): DONE 2026-11-07, `foyer::foyer_car_parts` drives `CarRenderParts` (doors, fe_wheels, discs) every frame; details "Car showroom" (Part matrices). Shots `research/screens/fe_showroom_door_open.png`, `fe_garage_steer.png`.
- To core: "Still open in core.md" naming item in Career (details L827 area); otherwise all Requests sections in details.md are done.
- From audio (2026-11-07): DONE 2026-11-07, AudioPlaylistScreen (DJ booth) ported (`screens/quick_race/fe_audio_playlist_screen.rs`) on a mirror/request channel `feck2/playlist_link.rs` (`Feck2.playlist`; details "AudioPlaylistScreen (DJ booth)"). Audio: nothing needed; `next_song/prev_song/next_soundtrack/prev_soundtrack` are not called by this screen. Shots `research/screens/fe_playlist_tracks.png`, `fe_playlist_edit.png`.
- 2026-11-08 session (frontend): (1) perf `hudopt` (`VELOCITY_PERF_OFF=hudopt` = old): `run_hud` / map screen skip rasterise + texture upload while the draw list equals last frame's (`HudResources.last_hud/last_map`, `Quad: PartialEq`), Visibility written only on change; golden london/menus/split bit-identical A/B, london frames 4-6 (stopped car) +8 % fps, driving frames unchanged (the list changes every frame). Left: rasteriser itself (per-pixel f32, naive) and the cpu-image approach. (2) MINIMAP FIX: `RaceHudInput.map` was never filled (marker stuck at world origin); `name_tags` in `crates/velocity/src/hud.rs` now fills player position + body row 0 (`rotation * X`, = VehicleEntity 0x001C66C0) + rivals (police blips still not fed). Shots `research/screens/fe_minimap_4x3.png`, `fe_minimap_1280x540.png`. (3) WIDESCREEN (deliberate, user-requested deviation from the 4:3 original; identical at 4:3): window aspect > 4:3 gives the cpu image `wide_size` (640*k px, virtual width 512*k; `rasterise_overlay_virt`); `DrawList::widen` (render2d.rs) = full-width bars (< 1/4 height) tile sideways, taller full-width quads stretch, everything else keeps its size and moves by an anchor of its group (consecutive touching quads: a string / gauge stays together). FE (`lib.rs run_frontend`): all centred. In-race (`hud.rs frame_slide` end + `wide_anchor`): left / centre / right by position (speedo right, position / lap top right), name tags scale across the whole width; the 3D hud (`run_osd_3d_view`) widens the interface camera aspect (`InterfaceProjection`) and shifts map / TFT models by the same anchor rule. Split screen: unchanged (stretched halves). Dev: `VELOCITY_WINDOW=WxH` (main.rs) for aspect captures; shots `research/screens/fe_wide_menu_1280x540.png`, `fe_wide_race_1280x540.png`.
- 2026-11-09 PC SETTINGS (user request, port-only): OPTIONS gets `OPTION_PC` at row 3 (rows after it shift by one; `fe_options_screen.rs`) -> `PcOptionsScreen` (`screens/options/fe_pc_options_screen.rs`, video-page look, values right-aligned at x 500): Display Mode (Windowed / Borderless / Fullscreen), Resolution (only enabled in Fullscreen: disabled-entry colours, up/down skip it; list = primary monitor's mode sizes + window size, `Feck2.pc_display` filled by `pc_settings::pc_display_info`), Frame Rate Cap (30/60/120/144/240/Off), V-Sync, Music Order (Original / Random = per launch, default Random); labels kept short so values fit right-aligned (fecapture has no window, so its Resolution value reads "-"). Verified in velocity.exe: options.ini FpsCap=60 VSync=0 -> `present mode Some(AutoNoVsync) .. frame cap Some(60)` + limiter `frame cap: Some(60) fps`; with `--fps 30` the log shows `(--fps)`. `GameOptions.pc` (`PcSettings`), options.ini keys `DisplayMode`, `Resolution=WxH`, `FpsCap`, `VSync`, `MusicOrder=Random|Original` (test `pc_settings_round_trip`). Applied live by `pc_settings::apply_pc_settings` (Window.mode / present mode Fifo|AutoNoVsync, `FrameRateCap` resource read by core's `frame_cap.rs` limiter, `VisosAudio.system.music_random` / `randomize_music_per_launch`); logs `pc settings: present mode .., window mode .., frame cap ..` and the limiter `frame cap: Some(n) fps`. `--fps` wins over cap + vsync, `VELOCITY_WINDOW` over window mode. Strings: `string_manager::PORT_STRINGS` (English, used when the game index lacks the id). Core files touched (minimal): `frame_cap.rs` (limiter always on, reads `FrameRateCap`), `bridge.rs` (camera widescreen forced off). Ratio row REMOVED from VideoOptionsScreen (TFT row 1, names row 2); `Widescreen=` key read but ignored, `widescreen()` always false. Open for audio: "Original Order" only clears `music_random`; the music rng keeps its per-launch seed (needs an audio API to reseed 1 for the exact original sequence). Shots `research/screens/fe_options_pc_row.png`, `fe_pc_settings.png` (resolution greyed), `fe_pc_settings_skip.png`, `fe_video_no_ratio.png`. tools/hidden_run.py now gives every run a throwaway APPDATA + VELOCITY_SAVE_DIR (seeded from research/golden/profile_seed, deleted after; `--keep-profile`) unless `--env APPDATA=` is passed (golden.py does).
- From graphics (2026-11-08, perf, partly DONE above): race HUD costs 6.7 ms/frame on the main thread (stage 7 trace: `hud::run_hud` 2.96, `run_osd_3d` 3.72; stage 8 trace 2026-11-08: 6.2 + 4.5 = 10.7 ms, now the largest main-thread item since render and main thread are balanced, `research/perf/baseline.md` "Stage 8 trace"). Both re-rasterise on the CPU and re-upload every frame: `run_hud` `dl.rasterise_overlay(.., 640, 480)` + `img.data = ..` (a full 1.2 MB texture upload per frame), `run_osd_3d_view` the 256² map screen likewise; `run_hud` also writes every `HudView` Visibility each frame (change detection always fires). Please skip the rasterise + `images.get_mut` when the draw list (and its textures) equals last frame's (keep the previous list in `HudState`), and write Visibility only when it differs. Graphics measures it with `tools/golden.py perf` once landed.
- From graphics: none open (MTV call sites wired except the unported text-screen MoveCamera 0x22B3C3 / 0x227680; "MTV overlay call sites" in details.md).

## Next candidates
- Tyre texture on the foyer wheels waits for graphics (`CarSpec.tyre`, docs/graphics.md). Done 2026-11-07: brake 0x25D020 (no bounce, respawn key), tyre 0x25D110 state, wheel model (`CarSpec.wheel` = WHEELS.BIN +0x1C stem, shot `research/screens/fe_wheel_model_change.png`), door sounds (`door_sounds` -> `AudioCommand::CarOpen/CarClose`).
- Decals DONE 2026-11-07 (details "Coverage triage"); graphics ANSWERED: `CarRenderParts.decal_slots: [u8; 6]` (kinds 0..5, CarRender 0x116BC0) is ready, OPEN for frontend: copy it into `fe.foyer_vehicles.decal_slots` (`Some(map u32)`) in `foyer_car_parts`; mix-your-own paint sliders (0x002505F0 etc., needs physics colour +0x9C/+0xA0 + 0x00187BA0); 0x00274250 FE3DCamera free-roam (debug); 0x00241E20 NetworkServerDetails (online, cut); 0x00270B10 FE3DCamera layer setup; 0x0021D010 FEHangoutScreen layer events; 0x00264450 FE3DCharacters debug screen; StatsScreen 0x00205F80 (mock-up, low value).

## Coverage
- Triage 2026-11-07 (details "Coverage triage"): biggest ~60 unported non-online funcs classified: only 2 real gaps (gauges list = DONE, `fecapture gauges`; car customisation decals = DONE; mix-your-own sliders = next); the rest are debug (FE3DCamera free-roam, AlloyEditor, submission shots), cut online screens' layers, the StatsScreen mock-up, or ported elsewhere / other crates / XDK.
- Physics API for mix-your-own (done 2026-11-07, `vehicle_database_colours.rs`): `setup.colours[0].{hue(), shade(), set_hue_shade(h, s)}` = setup +0x9C / +0xA0 (`unk_14`); `db.update_mix_colour(&mut setup)` = 0x00187BA0 (reads hue/shade, clamps shade to -255..255 and writes it back, writes packed `colours[0].unk_34` / `unk_38` = setup +0xBC / +0xC0, `b | g<<8 | r<<16 | a<<24`); `db.hue_wheel_colour(hue)` = 0x00187880 (hue 0..1529, packed).
- Core request open: in-race HUD should pick the hud by the setup's gauge hash (`HudConfig::name_for_hash`), core.md "OPEN requests".
- `python tools/coverage_fe/cov.py`: 1094 funcs / 467,411 B (60.6%) of 3250 / 770,780 B after decals (was 1084 / 463,044) (details: "Coverage audit"). Origin line format `/// File.cpp 0xA / 0xB — text`.

## Heading index of details.md (line)
- L5: Talking to the frontend (game flow / core)
- L30: Formats
- L53: FECK2 (src\Feck2)
- L76: Ported
- L154: In-race HUD (`hud::HudPlugin`, added by `FrontendPlugin`)
- L285: Debug-menu tweaks (`fe_tweaks.rs`, core's `CTweakManager` on `gm.tweaks`)
- L331: Main menu background (3D foyer) — `foyer.rs`, `fe_3d_camera.rs`
- L417: Garage (`feck2::screens::garage`)
- L469: Results OSD (`osd::osd_screens::osd_race_over_stat_screen`, `COSDRaceOverStatScreen` `byte
- L532: Overlay menu (`fe_2d_overlay_menu.rs`, FE2DOverlay 0x27D220..0x280EC0)
- L543: Options screens (`feck2::screens::options`, code 0x230AC0..0x23A4D0, no `__FILE__`)
- L606: Bink movies (`bink`, `vs_movie`, `movie.rs`) — RAD Bink 1 ported from the XBE's `BINK` sec
- L741: Career ("The Scene") — `feck2::screens::hangout`, profile select
- L995: Requests from physics (transmission choice into the setup) — done
- L1000: Requests from core (GamePlayer names, plate text) — answers to "Requests from frontend" in
- L1014: Requests from graphics (foyer character spec index, FE3DCharacters → Characters.cpp 0x0007
- L1022: Requests from graphics (3d hud lighting, front-end car shadow) — done
- L1044: Multiplayer / split screen (`fe_multiplayer_screen`)
- L1059: Requests from core (profile options, after-race save popups, garage copy, DummyGameScreen)
- L1086: Requests from graphics (FErefl floor reflection, front-end flares, character cross-fade) —
- L1115: Requests from core (split screen OSD, 2026-11-05)
- L1132: Split screen HUD (two players, `byte_6F7157`)
- L1172: Requests from graphics (split screen HUD areas, 2026-11-05)
- L1182: Requests from graphics (name tags per view, 2026-11-06) — done (see "Name tags per view" u
- L1194: Requests/notes from core (race points for the results table, 2026-11-06)
- L1204: Requests from core (3d interface clip planes, 2026-11-06)
- L1216: Coverage audit (2026-11-07)
- L1271: Car showroom (`fe_car_showroom_screen`, `gCarShowroomScreen` `dword_75F448`, 4276 B; dumps
- L1309: Background material textures (logo wall, 2026-11-07; dumps `research/fe_logo_dc.c`, `fe_ga
- L1331: Front wheel steer (FE3DVehicles +1256)
- L1341: Requests from graphics (foyer default lights, 2026-11-07) — done (`foyer.rs` uses `vs_rend
- L1347: Requests from graphics (MTV glitch overlay start/stop, 2026-11-07)
- DONE 2026-11-07: mix-your-own paint (`colour_mode`/`mix_hold`/`mix_apply` in `fe_car_customisation_screen.rs`, `fecapture mix`, shots `research/screens/fe_mix_{before,hue,shade}.png`; foyer uses packed `unk_34/unk_38` when colour hash == 0, key includes them), `decal_slots` copied in `foyer_car_parts`, `spec.decals` + key (shot `fe_decals_hks_visible.png`). Not ported: FE2DOverlay 0x0027F280 nav toggle (also skipped in swatches).
- **Requests from graphics (2026-11-07, decals invisible on the foyer car): DONE (below).** `foyer.rs` (~l.420-432) never sets `spec.decals`, so every decal shader shows `bonnet_off`. Add `spec.decals = visos_render::car_render::CarDecals::from_setup(setup, db);` next to `spec.tyre = ...`, and add `spec.decals.clone()` (`CarDecals: Clone + PartialEq`) to the `under` tuple / `CarKey` so a picked decal respawns the car. Only HKS01/HKS02/Blitz01/Injen01 (ML_4x1__*) and VT_window_HKS01 have textures in DECALST.VCD, so "HKS 2" shows, most others legitimately stay off.
