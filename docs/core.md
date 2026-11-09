# Core (engine, formats, game flow) - INDEX

_owner: core agent. Details live in `docs/core/details.md` (the full former core.md, ~220 KB): NEVER read it whole; grep it
by heading (`grep -n '^###' docs/core/details.md`, then `Read` with offset/limit). Other subsystems: same pattern (`docs/<x>.md` index, `docs/<x>/details.md`)._

## APIs other crates use (details: "Game flow", "Input, update loop, race runtime", "Stage 4")
- `EGameMode` state (Boot/FrontEnd/InGame/PostRace/EndOfGame/ReturnToFrontEnd); `GameManager::set_game_mode`; `GameManager.race: Option<RaceSelection>`, `.event_def`, `.rand` (shared rand stream).
- `LoadStepMessage { step: ELoadStep, paths: Option<RacePaths> }`: init is message driven, `step.owner()`/`label()`; read with `MessageReader`, act on own steps. `GameModeMessage::{FrontEndShutdown, InGameShutdown, RaceRestart, TweaksReset}`.
- `RacePaths` (track_dir, race_dir with trailing `\`, arena, cpts, triggers, cameras, markers, timeline, ...); `RaceData` (triggers, timeline bundle).
- `visos::input` (`input_manager::snapshot/key_value`), `velocity_game::controller::user_input::UserInput` (`value/pressed(EAction, mask)`), `GameInput::device_mask(p)`.
- `vs_common_main::GameTick` (24 Hz) with `InGameUpdateSet::{Input, Osd, CrashCam, MonitorSessions, UpdateSessions, PostRaceSequence, Ai, Physics, Cameras, World, Audio, Render, Effects, Timeline}`; `GameClock.timer.scaled_dt()/.lerp`.
- `RaceSession` (state, race_time, drag rounds) + `RaceSessionMessage::*`; AI fills `RaceProgress` (incl. `player_positions`, `player_state`, `results_busy`).
- `TriggerVehicles` in, `TriggerMessage` out; `TimelineObjectMessage`, `TimelineWorld`, `TimelineCameraView`, `FlagGirlEvent`, `TrailBlazerMessage`, `CrashCameraRespawn`.
- `GameBridge` (swap per update; writers `car(n)/camera(n)/timeline_model(n)/camera_previous(0)`, renderers `car_pose(n,lerp)/camera_view/osd_view`).
- `PhysicsWorld::handling_camera_overrides()` consumed in `race_cars.rs`; `core_tweaks` / debug menu tweak registrars (`TweakRegistrars`).
- Graphics messages written by core: `LoadCatsEyes { race_dir }` / `UnloadCatsEyes`, `UnloadTrackRender`; ai: `AiRespawnOnRaceline` via bin `ai_host::crash_respawn`.
- Smoke env: `SMOKE_DIRECT=1`, `SMOKE_RACE=n`, `SMOKE_DEBUG_MENU_PATH/_SHOT` (see "Fast test runs").

## Input inertness (audit 2026-11-08, after the user's keyboard stopped working during a sweep; cause unconfirmed)
- Every system-wide input touchpoint of velocity.exe: (1) winit 0.30.13 registers keyboard+mouse raw input at event-loop creation (default `DeviceEvents::WhenFocused` = RIDEV_DEVNOTIFY, no INPUTSINK/NOLEGACY/NOHOTKEYS/CAPTUREMOUSE; bevy_winit 0.19.1 never calls `listen_device_events`). (2) SDL3 3.4.18 `SDL_Init(GAMEPAD)` (= joystick+events): RAWINPUT joystick driver registers gamepad/joystick/multi-axis usages with RIDEV_INPUTSINK (background), HIDAPI opens pads, GameInput/WGI/XInput drivers, `SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1`, rumble writes. SDL's keyboard hook (WH_KEYBOARD_LL) and raw keyboard live in the video subsystem, which we never init. (3) No SetWindowsHookEx / SendInput / keybd_event / BlockInput / SetForegroundWindow / SwitchDesktop anywhere in crates or tools (grep 2026-11-08); hidden_run.py only CreateDesktop + CreateProcess on it; sweep.py / golden.py launch only via hidden_run. Most plausible culprits: SDL's background joystick raw input / HIDAPI pad access from several parallel hidden games, or winit's keyboard raw input registration.
- Guarantee: `tools/hidden_run.py` sets `VELOCITY_HIDDEN=1` -> `visos::input::raw_input::hidden_run()`; `InputManager::new` then never calls any SDL function (scripted ports only, no rumble), `user_input::sample_devices` ignores the keyboard. velocity main, in every run, right after `DefaultPlugins` (winit event loop built) calls `raw_input::remove_keyboard_mouse()` (RIDEV_REMOVE for generic-desktop mouse/keyboard = DeviceEvents::Never; nothing in the bin uses device events) and logs the remaining registrations ("raw input: registrations left"); a hidden run with any keyboard/mouse registration left exits 3. So a hidden run holds no raw input, hooks or HID handles at all. fecapture / trackview disable WinitPlugin and never create an InputManager. Frontend's own `ButtonInput<KeyCode>` read only sees the hidden desktop's (never focused) window.

## Pad hotplug (2026-11-09)
- `InputManager::update` each frame: SDL_PumpEvents + UpdateJoysticks + UpdateGamepads, drains the event queue, closes pads that report disconnected (port stays reserved, others do not shift), opens newly listed gamepads into the lowest free of 4 `PadSlots` ports; logs "pad N connected/disconnected". Bookkeeping unit-tested (`pad_tests`); real pads untested (hidden runs init no SDL).

## Race start fixes (2026-11-09)
- Tokyo "spawn in the middle of nowhere": `tracks.ini` lists 17 Tokyo races but the install ships only CIRCUIT/RACE01, SPRINT/RACE01-02 (also LADOWN SPRINT/RACE07 and Tokyo/ExecTest/TestArea cruise lack data; Tokyo2 is not in tracks.ini at all). No eventdef.txt -> grid read as zeros -> car at (0,0,0) = sea plane; the old golden "tokyo" (area 3) was TOKYO\CIRCUIT\RACE05 = this. Fix (deviation): `game_data_loading::resolve_race_path` (used by `RacePaths::new` and the replay camera init) substitutes the nearest shipped race of the track (same type lowest RACEnn, else CIRCUIT/SPRINT/DRAG/RELAY/CRUISE) with a warn log. Tokyo2's real grid is x~1020 (in bounds); not reachable from the menus.
- Cruise (London, London2, LADown): 0xA2EF0 pulls the grid 3.5 m short of the start line unconditionally (our port is exact), but cruise eventdefs carry the template line (-30,-12.5,2)..(30,-12.5,2) ("NOT USED"), so the original too would project the car onto it (London: (-1150,-12.4,1.5), off the map). Deviation: `CEventDefinition::load` skips the pull for `\CRUISE\` dirs (London now (-1150,0,334) on the bridge road; London2 / LADown / LACoast checked by first frame).
- Golden refs `tokyo_00..07` re-captured (car on the road, flag girl beside it); shot `research/screens/tokyo_start_fixed.png`.

## Race-end camera, crash notes (2026-11-09)
- Void race-end camera: PostRaceSequence state 2 (0x000C35C0) cut to the sequence camera (marker + orientation * offset) even when `setup` (0xC3F70) found no POST_RACE_STUNT_AREA marker; no shipped Markers.mfk has one (the original derefs the null `CMarkers::Find`), so the view sat at (0,0,0) with a zero matrix = sea/void plane. Deviation: no marker -> no camera cut, the chase/replay camera stays (`post_race_sequence.rs`, unit test). Verified London circuit (`SMOKE_LAPS=1`, shot `7f_post_race_cam` 2 s after the finish, `research/core_postrace/research/screens/c_new_7f_post_race_cam.png`). The "before" run did not reach the finish in time (no screenshot). Relay (only LONDON/RELAY/RACE01 ships) has no relay-specific code anywhere; with `SMOKE_LAPS=1` it finishes (race time 17.2, no panic, log `research/core_relay.log`); the user's relay crash is not reproduced.
- New smoke vars: `SMOKE_LAPS=<n>` (direct race lap override), shot `7f_post_race_cam`.
- `crates/velocity/src/crash_log.rs`: always-on panic hook + SetUnhandledExceptionFilter append to `%LOCALAPPDATA%\Velocity\crash.log` (fallback exe dir) with message, location, backtrace, version + git hash (`VELOCITY_GIT_HASH` from build.rs); release non-hidden runs show a MessageBox. Unit-tested (forced panic). `--log` unchanged.

## Career: "Event 05 done, Event 06 locked" (2026-11-09) - not an unlock bug
- UG tier 1 list (cat 1, by `order`): 1,4,7,10,13 (LA), 16,18 (London). "Event 05" = EV13 `0x51D1A2E1` = LADOWN DRAG RACE01 (single, 1 round, kind 1), not a relay: no event in EVENTS.VCD uses a relay race (London Relay is `Available: NO`, quick race only). "Event 06" = EV16 `0xC8D8F35B` requires the story event (cat 4) `0x25A61F72` completed, which itself needs EV7 + 7 completed cat 1/2 events of tier 1 (EvaluateUnlocks 0x8F020 ported exactly; user's save has 4 of those). So London opens only after 7 LA events (5 UG + 2 pro) and the story event; the completion of EV13 itself is recorded correctly (save shows it).
- The crash/double RaceOver: drag runs 3 rounds and the original calls RaceOver (0xBDB50) after each; the event is over after round 1 (`current_round` past the end). Original keeps its stored race pointer (+12) valid; port now falls back to `races.last_race` in `current_race(_mut)` (finish positions of later rounds land in the last race, event over / prize re-run each round like the original). Test `completion_unlocks_dependants`.

## OPEN requests
- DONE 2026-11-08 (audio request): main.rs always disables bevy's `AudioPlugin`; nothing else uses bevy_audio. velocity_audio's `plugin.rs` gate (was `is_plugin_added::<bevy AudioPlugin>`) is now `SMOKE_RECORD` unset && `WindowPlugin` present (MinimalPlugins tests open no device). Smoke exit 0, log "audio: null output" under hidden_run (`research/core_audio_smoke.log`).
- Requests from frontend (2026-11-07, gauges list): the garage's Styling > Gauges pick is stored in the setup (`VehicleSetup.unk_188` = `VsStringAnsi::Hash` of the `\OSD\HUDS` stem, 0 = digione). Race start (`crates/velocity/src/hud.rs` `HudCommand::Start { hud: "digione" }`) should send `velocity_frontend::osd::speedo::HudConfig::name_for_hash(player_setup.unk_188)` (OSD3dInterface::OnGameInit 0x000E9FD0 uses `GamePlayer +392`, then `OSD3dInterfaceConfiguration::Load` 0x000EB001), per player in split screen (`dword_6FD1F8` = one-shot override hash). Does `GamePlayer` keep the setup hash? If not, copy it at race setup.
- Regression check 2026-11-08 (HEAD, target/core, `--direct`, serial): the 8 sweep timeouts (LONDON2 x4, LADOWN x4) all pass in 92-110 s (they were load-induced, PC busy; no regression, no bisect needed); one run closed its window at 4 s of driving (exit 0, "No windows are open", `no_race`), passed on rerun. `GameClock::now` returns `visos_render::golden::clock_ns()` first (golden mode only) — done. Reruns: `research/sweep/core_rerun*.tsv`.
- Full sweep 2026-11-08 (HEAD b776fbe, target/core via `VELOCITY_EXE=target/core/debug/velocity.exe` — sweep.py defaults to target/debug!, `--parallel 2 --direct`, 8042 s): 163 pass, LADOWN_CIRCUIT_RACE09_dry "No windows are open" at 16 s (known window-closed flake) passed on rerun = **164/164**; all 164 logs show "raw input: registrations left [], keyboard/mouse 0". Log `research/core_sweep_run.log`, `research/sweep/core_results.tsv`.
- FIXED 2026-11-08 ("No windows are open" / `no_race` flake): every failing log shows `bevy_winit: Monitor removed X` x2 right before. bevy_window 0.19 links windows to monitors (`OnMonitor` -> `HasWindows`, `linked_spawn`) and bevy_winit `create_monitors` (each `about_to_wait`) despawns a Monitor whose HMONITOR is missing from one enumeration (display reconfig, desktop changes) -> the primary window is despawned with it -> exit 0. Would hit the real game too (monitor off / resolution change). Fix: `crates/velocity/src/window_keep.rs` observer strips `OnMonitor` from windows (nothing reads it); unit test with negative control (without it the window dies). Not the swapchain guard / audio. Sweep check `research/sweep/core_winkeep.tsv`.
- (done, see above) Requests from graphics (2026-11-07, perf golden gate, user priority): make `GameClock::now` (vs_common_main.rs) frame-locked in golden runs: `if let Some(t) = visos_render::golden::clock_ns() { return t; }` first (`None` unless `VELOCITY_GOLDEN` is set, so the normal game is unchanged). Without it the race golden images (and their perf workload) follow the wall clock and differ run to run; menus / trackview scenes are already deterministic. See graphics details "Performance".
- Requests from graphics (2026-11-08, perf gate determinism): race golden images depend on things that are not rendering. (1) Same build, `VELOCITY_GOLDEN_SETTLE=60` vs default 6 (only the number of paused frames per capture differs) -> london_wet_cams diverges from shot 01 on: something steps per render frame while `Time<Virtual>` is paused and feeds game state — suspect `EngineShake`'s 3 `VsRandRange` per Update from the shared `gm.rand` (render-rate draws from the game RNG also make the race depend on the frame rate). Please draw nothing from `gm.rand` while virtual time is paused (or per GameTick / own RNG). (2) With equal pauses, any change to the system graph (graphics added 2 systems, no game-visible change; shots 02-06 bit-exact) moves shots 07+ by one GameTick (lap time 8.54 vs 8.58): the smoke script's shot trigger vs GameTick is order-ambiguous. Please order the smoke-test step systems explicitly `.after(<game tick system>)`. Repro: `research/golden/p3base_off` vs `p3new_off` (graphics details "Performance" stage 3).
- DONE 2026-11-08 (golden determinism, request above): EngineShake was already per GameTick (`update` is a GameTick system), game state is identical between SETTLE 6 / 60 runs (same log; the image difference is render-side rain ripples only). The real cause of the one-tick shift: smoke race shots / key windows used `s.t` (render-frame time); now `dt` = (`total_updates` - `Script.race_u`) * `timer.dt` (smoke_test.rs, direct.rs split_pad, flag-girl shot via `intro_t` in game time). Verified: binaries differing by 3 inert systems (PreUpdate/Update/PostUpdate) give bit-exact london_wet_cams except shot 06 (spark particles: unordered `ResMut<Particles>` systems, request put in graphics.md); tokyo + split pass compare. Same binary twice: bit-exact to shot 06, noise-level after. Scripts: `VELOCITY_PERF_DIR=<dir with velocity.exe> tools/golden.py capture ...`.
- Requests from graphics (2026-11-08, perf stage 7): (1) london_wet_cams golden run (`SMOKE_DIRECT=1 SMOKE_AREA=2 SMOKE_WET=1 SMOKE_CAMS=1 SMOKE_DRIVE=20`, `VELOCITY_GOLDEN` set): `update_cameras` took **27.4 s in a single GameTick call** (~14 s after shot 00), then 0.2-2.7 s per call until shot 01, while `vs_fly_camera: Error finding ground under camera` logs every ~0.6 s (normal calls 0.1 ms). Looks like the fly camera's ground search via `ground_y` (vs_camera_manager.rs:250) looping / scanning when no ground is found. Repro + numbers: `research/perf/baseline.md` "Stage 7 trace". (2) `effect_clock` (race_effects_bridge.rs, Update) is unordered vs its readers `race_effects_frame` / `update_particles` (visos_render, both run after the set `visos_render::race_effects::RaceEffectsMessages`): please add `.before(visos_render::race_effects::RaceEffectsMessages)`, and the same for `mode_messages` (it writes RaceEffectsReset/Shutdown read there); bevy's ambiguity report (`VELOCITY_AMBIGUITY=1`, needs the trace_chrome build for names) lists both pairs.
- DONE 2026-11-08 (graphics perf stage 7 request above): (1) the fly camera's ground cast (`game_cameras::update_cameras` `ground`) was a 1000 m cast down from the timeline camera's *relative* y (never hit, "Error finding ground" every update, the 27 s calls); now 1:1 VsFlyCamera 0x00064090: LineCheck (RAY_TRACK|RAY_NORMAL) at the camera x/z from grid slot 0 y (`dword_6F5C0C` = `event_def.start_positions[0]`) +5 to -5. No more errors; the intro shot (london_wet_cams_00) now sits at street level. (2) Shot tick: `in_game_init` now does InGameInit's SetRate(24) / vblank 0 (0x000BF4B0) / `VsFrameTimer::reset` 0x00057310 (0x000B6074..0x000B608C; was missing: the update phase vs render frames followed the boot/load frame count; `total_updates` restarts at 0 per race). smoke_test: in-race shots wait in `flush_shots` for update `tick+1` (`Script.shot_tick`; tick = count before the frame's updates) and log "shot at update N lerp L"; `watch_cams` counts its 1.5 s in updates. london_wet_cams `--fps 0` vs `--fps 20` (`research/golden/c8_fps0` / `c8_fps20`): same updates/lerps at all 13 shots, 13/13 pass (10 bit-exact, 07/09/12 max <= 30 noise). All race refs change (graphics: recapture `ref`). (3) `effect_clock` / `mode_messages` `.before(RaceEffectsMessages)`.
- To core: none else open (all "Requests from ..." sections below are done). Wheel-camera fields (fov, offset x/z) from `handling_camera_overrides` not applied: no live global yet.
- Requests from physics (2026-11-07) — done: `race_cars::race_release` calls `launch_boost(id, 0.0)` per human car on ReleaseCars. The gauge (`unk_6FC098`+2936 = COSDMapRender) is written only by its ctor 0x000E9A80 (0.0; no other store or xref in the XBE), so the result is always 1; ai reads `launch_result` already. Log: `research/core_s9_race.log`.
- Engine shake 0x11ABB0 (graphics leftover) done 2026-11-07: `game_manager::EngineShake` (3 `VsRandRange(-10,10)` per Update from the shared `gm.rand`, 0xBB1C0; per-player rpm filled in `race_cars::car_outputs` from the existing per-tick `engine_rpm` call, no extra smoothing call) applied to the player cars' render root in `place_cars` (+-5 mm / +-1 cm * min(rpm/8000,1)). Camera and golden test untouched (camera reads the physics pose); 0x11AAB0 / 0x11A8F0 are shader-flag masks, not ported. Log `research/core_s11_race.log`.
- From core (not done by others): graphics still parses GDL point lights itself instead of `gdl_lights::load_lights` (request in graphics.md); `GDL_RelocateModel/Anim` fixups: graphics reads unrelocated; score OSD item 0x169BF0 is frontend OSD.

- Requests from graphics (2026-11-08, perf stage 8, low priority): race state depends on entity allocation order. Spawning 4 extra (inactive, render-only) camera entities mid-race changed london_wet_cams shot 06's barrier debris positions (same shot update / lerp, frame timing identical); graphics now spawns them unconditionally, so the gate is stable. Somewhere a `Query` iteration order (entity index) feeds the RNG or collision event order (collision listener -> `race_effect_messages` debris). Repro: graphics details "Performance stage 8".

## Stage list / next candidates
- Done: coverage audit, stage 2 (race timeline loads, seek fix), stage 3 (replay camera bridge writes), stage 4 (rumble `vibrate_effects` + visos `set_rumble`, camera view write-back 0x5F800). The remaining gameplay bucket is teardown / ctor / net / debug / template code (triaged in "Coverage stage 2/3/4"); real leftovers: 0x603C0/0x60CE0 cinematic camera set (unreferenced), LoadModelGeometryAsynch 0xA0520. Debug-menu rumble tweaks are live on `VibratePresets` (`core_tweaks::add_vibrate_tweaks`, one `row!` per preset). Next stage: re-audit by owner crate (physics/ai/audio coverage) rather than core.

## Coverage
- `tools/coverage/cov.py core`: 601/3939 functions, 251421/579262 bytes (43.4%) after stage 5 (592 / 248053 / 42.8% before; details: "Coverage audit", "Coverage stage 2/3/4"). Stage 5 (2026-11-07) found no real core gameplay gap left: the rest is MEM/FILE/Pool/Input* XDK-side, net MessageManager, debug draws, teardown, and already-ported code whose comment is not an origin line (`/// File.cpp 0x...` at the line start is what cov.py counts; 0x1812C0, 0xBD9B0, 0x1E69F0 credited). Uncredited-but-done in other crates: 0x6ADC0/0x6AE70 (physics), 0x98A80/0xBBB00 (flag_girl_models, game_bridge).

## Section index of details.md (heading, line)
- L5: VCD packs — `visos::file` (ready to use)
- L33: Bundles (`.bun`, `.bdl`) — `visos::chunk`, `visos::type_`, `visos::instances`, `visos::timeline`
- L56: Compartments (`.cpt`, `index.idx` in `Tracks\*\CPTS.VCD`, `RACE.VCD`) — `visos::compartment`
- L67: Text configs — `visos::handy::vs_text_file::VsTextFile` (use it for every `.txt`/`.ini`/`.cfg`)
- L75: Game flow — `velocity_game` (states, load steps)
- L107: Input, update loop, race runtime — use from other crates
- L201: Frame-rate independence (render side of the 24 Hz split)
- L253: Debug menu (in-game tweak menu) — `game_menu_manager::{game_menu_manager, game_tweaks_menu}`, binary `debug_menu.r
- L377: Unlock mode (`-u` / `--unlock`) — `cli::debug_unlocks::DebugUnlocks` resource
- L392: Career events — `event_database::{event_database, event_database_events, event_database_bin_loading}`
- L427: Career ("The Scene") — `game_mode::career`, `register_file_types::save_file_data`
- L506: Cameras, replay cameras, markers — `camera::*`, `replays::cameras::*`, `controller::vibrate_effects`
- L605: Attract mode, review cameras — `font_helper::CAttractMode`, `replays::cameras`, binary `attract.rs`
- L652: Stage 4: timeline objects, flag girl, score, bridges, post race — use from other crates
- L691: On-disk file types → loader → owner
- L714: Cross-crate glue: race effects, particles, physics objects (done; were graphics' and physics' requests)
- L769: Pause menu (done; was the frontend's request)
- L800: Car lights (done; was graphics' request)
- L808: Original render rate (fact)
- L819: Car damage glue (done; was physics' and graphics' requests) — `game_mode::car_damage_bridge`
- L853: Wheel spin, camera rain (done; were graphics' requests)
- L873: Puddles, lens splash, tyre spray (done; were graphics' requests) — binary `puddles.rs`, `race_effects_bridge`
- L897: DOF, zone FX, danger zone inputs (done; were graphics' requests) — `game_mode::race_effects_bridge`
- L936: Race audio wiring (done; were audio's requests)
- L966: Driver, fade, lens spray, held gear (done; were graphics' stage 3 and audio requests) — `race_effects_bridge`
- L999: TrailBlazer calls (done; were physics' and ai's requests) — `trail_blazer::{apply_trail_blazer_calls, apply_world_
- L1033: Smoke options, career keyboard, small requests (done; were frontend's, ai's and graphics' requests)
- L1059: Restore point object messages, object resets (done; were physics' requests) — `race_timeline_sequencer`
- L1091: Instance move, reset all objects (done; were physics' requests) — `race_timeline_sequencer::PhysicsObjectResets`
- L1100: Zone trails, danger particles, timeline particle groups (done; were graphics' requests)
- L1145: Teams joined (done; was the frontend's request)
- L1153: Requests from graphics (physics object lighting, CompartmentManager 0x00087D00) — done
- L1161: Requests from graphics (flag girl spec index, Characters.cpp 0x0007A460) — done
- L1168: Requests from graphics (car headlights, TrackFX 0x0014B030) — done (see "Chase camera", HeadlightView)
- L1181: Requests from graphics (car light flares, FlareRender 0x00123930 / 0x00123970) — done
- L1204: Requests from graphics (dynamic envmap, DangerZoneRender 0x001214A0) — done
- L1217: Requests from physics (chase camera VE fields: up offset, player camera shake) — done
- L1238: Post race mode, quit the event, attract stand-in gone (done; were frontend's requests)
- L1255: Timeline audio, ai / traffic / police audio input (done; were audio's requests)
- L1345: Requests from physics (HUD / audio rpm, transmission) — rpm done (call sites checked in code: player order audio
- L1373: Requests from audio (rpm call order, traffic rpm call, throttle read) — done
- L1398: Requests from ai (respawn cameras, timeline AI, name colour) — done
- L1416: Requests from frontend (cinematic bars) — done
- L1431: Fast test runs (SMOKE_DIRECT, parallel sweep)
- L1471: Requests from physics (jump cam, attract RandomiseSetup, zone FX boost block) — all done
- L1579: Requests from ai (drag rounds, 2026-11-05) — done: rounds restart in place
- L1602: Requests from audio (no VsAudio::Update during the race load) — done
- L1615: Requests from frontend (next event race, split screen, garage copy) — done (split-screen race itself open)
- L1655: Requests from graphics (dummy police car, flashes) — done
- L1676: Split-screen race (core side) — `game_mode::game_views`, `race_cars`
- L1723: Requests from graphics (split screen views, 2026-11-05) — done (see "Split-screen race")
- L1734: Requests from graphics (rear-view mirror, per-view zone FX, 2026-11-05) — done
- L1756: Requests from graphics (per-view HeadlightView / in-car hide, 2026-11-06) — done
- L1777: Requests from graphics (FuryFX / nitrous, 2026-11-06) — done
- L1815: Requests from ai (split screen, 2026-11-05) — done: `RaceProgress.player_positions` / `player_states` (Vec by pla
- L1824: Requests from physics (jump cam rate, dummy cop tag, split screen) — done (rate: see "London ramps" / "System Upd
- L1832: Requests from frontend (race points, split HUD, ContinueEvent) — 2026-11-06 — points done
- L1858: Requests from frontend (results OSD drives `results_busy`, 2026-11-06) — smoke presses done
- L1875: Requests from physics (shared rand stream for the jump cam roll, 2026-11-06) — done
- L1892: Requests from graphics (player 1 zone blur focus, 2026-11-06) — done
- L1903: Requests from physics (explosions, friction spots, race restart, 2026-11-06) — done
- L1939: Requests from physics (speedo gauges: nos names fixed, zone / turbo getters, 2026-11-06) — done
- L1955: Requests from ai (crash camera respawn, 2026-11-06)
- L1968: Requests from frontend (debug-menu tweaks, 2026-11-06) — done (title note dropped, split time read from the tree
- L1983: Requests from graphics (graphics AddTweaks into the debug menu, 2026-11-06) — done (`core_tweaks::GFX_SITES`, see
- L2004: Requests from ai (debug-menu tweaks, 2026-11-06) — done (`SMOKE_DEBUG_MENU_PATH` / `_SHOT`, see "Debug menu")
- L2015: Requests from audio (audio AddTweaks into the debug menu, 2026-11-06) — done (see "Debug menu"; DopplerFactor →
- L2030: Requests from physics (physics AddTweaks into the debug menu, 2026-11-06) — done (see "Debug menu"; Turbo.Rise ra
- L2047: Requests from physics (Handling.cfg camera overrides, 2026-11-06)
- L2062: IDB rebuild (2026-11-07, tooling)
- L2071: Coverage audit (2026-11-07)
- L2119: Coverage stage 2 (2026-11-07): race timeline loads, seek fix, triage
- L2151: Requests from graphics (cat's eyes, 2026-11-07)
- L2162: Coverage stage 3 (2026-11-07): replay camera bridge writes, triage
- L2181: Coverage stage 4 (2026-11-07): rumble (VibrateEffects), camera view write-back, triage
