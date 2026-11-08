# Core (engine, formats, game flow)

_owner: core agent_

### VCD packs — `visos::file` (ready to use)

```rust
use visos::file::{Vcd, VsFileDirectoryDetails, VsVCDLoader, game_dir, resolve};
let v = Vcd::open(r"\FE\FE.vcd")?;              // xbox-style game path, resolved case-insensitively
for e in v.entries() { e.name; e.crc; e.size; e.offset; }   // toc order
let bytes: Option<&[u8]> = v.get("ASP.GAS");    // case-insensitive (crc of lowercased name)
let e = v.find_hash(0xE13AA739);                // by precomputed hash; v.data(&e) -> bytes
visos::handy::handy_crc::name_hash("asp.gas")   // == 0xE13AA739
visos::file::platform::read(r"\Events\x.txt")   // loose files
```
- Format (`File\VsFileDirectory.cpp`, `File\FILE_VCDLoader.cpp`), all LE: `u32 count:24|version:8` (version 1
  everywhere), `u32 dataSize`, `FileCrcEntry[count] {u32 crc, u32 size, u32 offset}`, data area (`dataSize` bytes,
  offsets relative to its start, files padded to 2048-byte *absolute* boundaries), then name TOC
  `FileEntry[count] {char name[128], u32 size, u32 offset}`. File size == `8 + 12n + dataSize + 136n` exactly.
  No compression, no flags. Names are bare file names (no dirs).
- Hash = `HANDY_CRC32NoCase(name, strlen, 0)`: reflected CRC-32 (poly `0xEDB88320`), **seed 0, no final xor**,
  each byte `tolower`ed. `crc32` / `crc32_wide` / `*_no_case` variants are all in `visos::handy::handy_crc`.
- Engine lookup is a linear first-crc-match (`VsFileDirectoryDetails::FindFile` 0x29D770); the resident loader
  (`VsVCDLoader`, max 40 packs) builds a crc map where later duplicates win. No shipped pack has duplicates.
- `VsFileDirectoryDetails::open(dir)`: a real folder named like the pack (minus `.vcd`) **takes precedence** over
  the pack; otherwise the pack is used. `load_file(name)` returns the bytes.
- Test `crates/visos/tests/vcd_all.rs` parses all 302 packs (12k+ entries), checks every crc against its name and
  every range. Only `PhysObjs\PHYSOBJM.VCD` is corrupt as shipped (CRC TOC overwritten with file data); the game
  never references it.
- File types inside packs: xmf 6853, vtf 3670, cpt 541, slp 221, bin 204, tga 141, xvg 130, gaf 128, fml/fob 29,
  manifest 19, xml 17, bun 10, txt 9, osd 5, idx 5, gas 3, gvf 2, tmp 2, gvs/vcd/msh/qdt 1.

### Bundles (`.bun`, `.bdl`) — `visos::chunk`, `visos::type_`, `visos::instances`, `visos::timeline`

```rust
let (b, env) = visos::chunk::Bundle::load_standalone(&bytes)?;   // or Bundle::load(&bytes, &mut env)
for i in &b.instances.unwrap().instances { i.name; i.world /*row-major, row3 = pos*/; i.type_name(); i.value }
i.value.field("Radius")?.as_f32(); .field("FocalPoint")?.as_vec4(); .as_str(); .as_bool()
for t in &b.timelines { t.length; t.channels[..].tracks[..].items[..] /*kind,start,length,payload*/ }
```
- Chunk tree (`Chunk\file_chunk.cpp`): header `u32 id, u32 nameOff(from file start), u32 size, u16 children,
  u8 flags (bit1 = data out of line, +0x10 = offset), u8 alignShift`; inline data at +0x10 aligned to `1<<align`,
  children after the data (4-aligned). Pad bytes are `0xAB`. Ids: `Chunk`/`Typelib` = **case-sensitive** crc32
  (`0x92E5A937`/`0x58D3948D`), `Timeline`/`Instances` = no-case (`0x23DC190F`/`0x9C2E14C7`). 134 of 155 shipped
  bundles were written with no-case `Chunk`/`Typelib` ids, which the engine's checks reject (`engine_accepts`).
- Types are **self-describing**: the Typelib holds VSTYPE text (`struct F3HeatHaze {VsVector FocalPoint; float
  Radius; bool Enabled; }`), parsed by `type_::vs_type::CEnvironment` (port of the VSTYPE parser, 0x2E4340..).
  Layout rules: int/float/string(ptr)/enum = 4 bytes, bool = 1 byte, **all alignments 4** except
  VsVector/VsQuaternion/VsMatrix = 16; no tail padding; arrays `T name[N]` stride = size padded to T's alignment;
  `VsVarArray<T>::SelfPtr` = `{u32 count, u32 offset}`. Strings in data are offsets relative to the chunk data.
- Instances chunk: version 1, `FileInstance` 96 bytes (name, parent, flags, VsMatrix world, typeRef, data, group).
  Timeline chunk: version 5 only (version-4 chunks exist and are skipped by the engine: `rejected_timelines`);
  targets (96 bytes, matrix + point list), channels (40) → tracks (24) → items (16: kind, start, length, payload;
  kind 4 = typed event decoded to `Value`). Tests: `tests/bundles_all.rs` (155 bundles, all types/values resolve).

### Compartments (`.cpt`, `index.idx` in `Tracks\*\CPTS.VCD`, `RACE.VCD`) — `visos::compartment`

- `Compartment::parse(bytes)`: 64-byte header `'_CPT'`, version 5 (v3 shifts fields +4; the engine only reads the
  v5 layout), size (sectors: body only; common/instance: whole file), bounds f32[6], entries
  `{u32 offset-from-header, u32 kind(low byte)|extra, u32 hash}` (kind 0/8 mesh, 1 texture named from the
  nul-separated list at +0x40, others ignored), instance lists (Visual/Physics/Collision/Immovable/AlwaysVisible:
  80-byte raw records, SimpleAnimation 100). The exporter pads SimpleAnimation records to 112 so the engine
  misparses anything after one (kept: `instance_error`). All shipped cpts are raw; `refpack_decompress` ports the
  streaming loader's RefPack (EA QFS) decoder. Payload decoding (mesh/texture/instance records) is graphics'.
  Test: `tests/cpt_all.rs` (541 cpts, 5 indexes).

### Text configs — `visos::handy::vs_text_file::VsTextFile` (use it for every `.txt`/`.ini`/`.cfg`)

The engine's line reader (VsStringAnsi.cpp object range 0x2B2D80..): skips blank lines and lines starting with `#`
or `//`; `read_s32/read_f32/read_f32n::<N>()` read numbers after the first `(` on the next line (`( 1, 2.5, 3 )`),
`read_string` the first quoted string, `read_key_value` a `KEY : "VALUE"` / `KEY "VALUE"` line,
`read_line_value_s32/f32` the number after the first space. `load(Some(&dir), name)` reads from a
`VsFileDirectoryDetails` (pack or folder), `load(None, path)` a loose game path. Each read returns `None` at EOF.

### Game flow — `velocity_game` (states, load steps)

- `EGameMode` Bevy state = `GameManager+0x210`: `Boot` → `FrontEnd`(0) → `InGame`(1) → `PostRace`(2) →
  `EndOfGame`(3) → `ReturnToFrontEnd`(4). `GameManager::set_game_mode` ports SetGameMode (0x000BBD30).
- **Subsystem init is message driven**: on entering FrontEnd / InGame the core writes one `LoadStepMessage
  { step: ELoadStep, paths: Option<RacePaths> }` per original init step, in the original order
  (`game_data_loading::{front_end_init_steps, in_game_init_steps}`, from GameManager.cpp 0x000B5510 / 0x000B4F80 /
  0x000B5F80 and GameDataLoading.cpp 0x0009CEC0). `step.owner()` says which crate handles it and `step.label()` is
  the original string. Read them with `MessageReader<LoadStepMessage>` and act on your own steps.
  `GameModeMessage::{FrontEndShutdown, InGameShutdown, RaceRestart, TweaksReset}` carry mode-change side effects.
- `RacePaths` gives the per-race files: `track_dir` `\Tracks\LACOAST\`, `race_dir` `\Tracks\LACOAST\CIRCUIT\RACE01\`,
  `arena_t/arena_m/cpts/race_vcd/triggers/cameras/markers/timeline/heat_haze/skin_inst/setup(wet)/event_def`.
- The current race is `GameManager.race: Option<RaceSelection>` (track, type, race, paths, players, traffic, wet,
  professional); its parsed `eventdef.txt` is `GameManager.event_def` (`CEventDefinition`: opponents, grid start
  positions in game coordinates, start/finish lines, laps). Core-loaded race data is in `RaceData`
  (`triggers`, `timeline` bundle).
- Command line (`main_cpp::CommandLine`, main.cpp 0x0016D120): `jumptogame TRACKS\LACOAST\ CIRCUIT\RACE01\`
  boots straight into that race (via `CGameJump`, GameVars.cpp 0x000C2860), plus `professional/underground`,
  `wet/dry`, `profiler`, `skipInitialScreens`, `jumptoscreen`, `setCar`, `noaudio`, `nomusic`, `nobarriers`,
  `novibrate`, `setflaggirl`, `hud`. `--x value` options (e.g. `--game`) are skipped. Original quirk kept:
  `splitscreen` never takes effect (GameVars 0x000C2EC0 clears it).
- Event DB (`event_database::event_database_track_loading::CEventDatabase`): `\FE\TRACKS.INI` (99 races, 82 present
  in this build) + `\Tracks\<T>\setup.ini`; race types UNKNOWN/CRUISE/SPRINT/CIRCUIT/RELAY/DRAG/PURSUIT; race dir =
  `\` + track path + `TYPE\RACE\`.
- Triggers (`timeline_manager::objects::timeline_trigger`): `triggers.trg` v1.5 only (63 load, 5 older rejected as
  the original does): 16-byte header, 80-byte records (position, quaternion with xyz negated on load, dimensions,
  event crc, type crc).
- Tweaks (`cli::tweak`, tweak.cpp): `CTweakManager` systems under `$`, typed values held in `TweakCell`s,
  `Tweaks.txt` export and `tweak.set` save formats. F3_D has **no console**: `autoexec.cli` is never read by the game
  (`cli::debug_menu::CDebugMenu::load_cli` parses it; nothing in F3_D shows that tree: the in-game debug menu is
  GameMenuManager's tweak menu, see "Debug menu").

### Input, update loop, race runtime — use from other crates

- **Input = `visos::input`** (SDL3, `sdl3-sys` built from source and linked statically; Bevy's gilrs is disabled
  in the binary). `input_manager::snapshot()` / `key_value(device, key)` / `any_key_value(key)` give raw xbox-pad
  keys (`input_device::key::*`, order of `XBOXPAD_KEYS`) for devices 0..3 (SDL pads) and 4 (keyboard, PC table
  `user_input::KEYBOARD_KEYS`). Steam hint overrides are applied in `InputManager::new`. `set_script(Some(keys))`
  replaces port 0 (tests, `--slice-test`). Game actions: `velocity_game::controller::user_input::UserInput`
  (`value/pressed(EAction, mask)`, `Config1.ini` bindings); player masks `game_input::GameInput::device_mask(p)`.
  `FE\KEYMAP.INI` is not read by F3_D (it is a PS2-style action remap); the keyboard uses the PC table.
- **Fixed 24 Hz update**: schedule `vs_common_main::GameTick`, run from `PreUpdate` while `VsFrameTimer::tick`
  asks (max 2 per frame). Put per-update game systems in `InGameUpdateSet::{Input, Osd, CrashCam, MonitorSessions,
  UpdateSessions, PostRaceSequence, Ai, Physics, Cameras, World, Audio, Render, Effects, Timeline}` (chained in
  GameManager::InGameUpdate order). `GameClock.timer.scaled_dt()` = update dt, `.lerp` = render interpolation.
- **Race session** (`game_mode::race_session`, MonitorSessions 0xBC5F0 / UpdateSessions 0xBD7A0 ported state by
  state): `RaceSession.state` Init/Reset/Intro/Countdown/Go/Racing/Finished/PostRace, `race_time`, drag rounds
  (3 per InGameInit, `drag_continue` = +0x142 next round via the PostRace mode). Messages: `RaceSessionMessage::
  {ResetToGrid, ClearVehicleInputs, ReleaseCars, RaceFinished, StartPostRace, AiTakesRacers, OsdFlags, Hud,
  CameraMode{1 follow, 3 cine}, FlagGirl(FlagGirlEvent), DragResult, Mileage{laps}, EventOver(EventOutcome),
  PostRace(PostRaceEvent)}`. The AI fills `RaceProgress` (finish order as event competitor slots, order ids,
  player place/state, racers, results_busy, car_flags, racer_records) and sets `RaceSession.race_finished`.
  Race end (0xBDB50) writes finish positions and runs race/round/event progression; EndOfGame (0xBE540) goes back
  to the FE when quit or the event is finished (after the race records 0xBE430), else the next race.
- **Grid hold / Go for the players** (`race_cars::race_release`, Physics set before `car_inputs`):
  ClearVehicleInputs = 0x000BE840 (each player car: steer 0, handbrake 1, brake 0, wheel lock on via body
  0x0035DCC0 = car +1236/+1248), ReleaseCars = 0x000BD9B0 (wheel lock off; controls +0x16 = 1 and the start boost
  `VehicleEntity__1D1270` not ported). The AI's grid state (0x0001B4F0) also calls 0x0035DCC0(1) and a human car
  never leaves it (human branch returns before the state machine), so without ReleaseCars the player stays locked.
- **Triggers** (`triggers::trigger_update`): fill `TriggerVehicles` each update (AI/physics); effects come out as
  `TriggerMessage` (AI hints, respawn, speed clamp, timeline...), audio ones go straight to `AudioCommand`.
- **Race timelines**: `visos::timeline::timeline_sequencer::TimelineSequencer` (+ `TimelinePeer` trait);
  `timeline_manager::race_timeline_sequencer::RaceTimelines` plays them on TIMELINE triggers and publishes
  `RaceTimelineEvent` (Begin/Update/End/Matrix/BoneAnim; Matrix in glam convention, translation in `w_axis` —
  visos `path_update` was fixed, it used to drop the translation). Timeline objects: see "Stage 4" below.
- **Audio wiring** (done here, at the original call sites; race details in "Race audio wiring"): Init +
  LoadFrontEndGroups on FE init, Init(track) on race init, Shutdown on mode shutdown (before the next Init), Pause/Resume, StopGameAudio on restart, RemoveShopAlarm at Reset, Countdown,
  RaceStarted, cover/shop triggers. `GameAudioInput.player/camera` are filled from the player car each update.
- **Frontend ↔ game**: the binary bridges `FrontendRequest` to `velocity_game::GameFlowRequest`
  (`SetControllerMask`, `StartQuickRace{race, professional, wet, car: PlayerCarChoice}`) and stops the FE.
- **Race cars** (`game_mode::race_cars`, the stand-in `slice` module is gone): "Load Arena" step →
  `LoadTrackRender` + the `TrackCamera`; "Physics Manager" step → `LoadPhysicsTrack` (written inside the next
  GameTick so it cannot expire) and the race cars (PhysicsManager 0x19E080: players first on the eventdef grid,
  heading `rotation_y(dir - 90°)`, car forward = +X). `plan_race_cars` = the "Race Competitors" passes: humans
  over the current race's competitor slots (vehicle id n, the player's `PlayerCarChoice.setup` = the FE car
  select's `VehicleSetup`), then opponents (ids after the players, as the AI numbers them; grid = competitor
  slot): team car for categories 2/3 without the temp flag, else the opponent's setup `tier - 1`. The opponent
  and team databases are the AI crate's, so its plugin installs `RaceCarSourceRes`
  (`velocity_ai::race_car_source`). Physics inputs = `LocalConfigInputs::from_setup`; players pad slot = id, opponents -1
  (AI controls). Cars wait as `PendingCar` until the collision exists, then `car_inputs` creates them itself in
  id order at the eventdef height (VehicleManager 0x1DC650; ids must equal the AI's) and inserts `PhysicsCar` +
  `PhysicsCarState`. Each car renders through `SpawnCarRender { request: id }`: `car_spec` = vehicle folder,
  `CarSpec::with_colour(CarColour::from_record(colours[0].unk_0c, colour_a, colour_b))`, parts from the fitted
  bodyparts' model names (`Bodypart.string_10`, e.g. "f_bumper3" → slot + index; VehicleEntity 0x001C08B0 loads
  exactly those, `+0x10` + `_med`/`_low`). Not done: `RandomiseSetup` (`byte_6F5A28`), setup `+0x18C` clear.
- **Traffic / police cars** (same pass, after the race cars): flags as VsAIWorld::Init (traffic = option && !network,
  police = (traffic || pursuit) && !drag); area = race entry's track `traffic_area` (+0x84). Traffic: 14 cars, ids
  from the race's competitor count (the ai's numbering), car i < 3 kind 2 when the area has one else kind 1, i ≥ 3
  kind 1, `get_nth_traffic(area, kind, i % count)`, `LocalConfigInputs::from_traffic`. Police: 2 cars at +14,
  `RaceCarSource::police_car(group crc of "LA"/"LONDON"/"TOKYO", n)` (opponents with flag 2, setup 0). Both are
  created at the origin and **deactivated** right after the physics spawn (the ai teleports/activates them);
  `RaceCar { kind: CarKind, active }` (read from `is_active` each tick); inactive ones are invalid in the car
  bridge and hidden (`place_cars` hides any id at or after the race-car count without a valid pose). Police and
  traffic both render via `SpawnCarRender` (traffic: `CarSpec::traffic(name, Vec3::ONE)`, VehicleEntity 0x001C0D60
  no-setup branch; the loader sets no paint colour, white here); the `traffic_render` stand-in is gone. Traffic roots
  now get `CarLightInput` too. Known gap: police without traffic (pursuit with the option off) get physics ids
  `first..` while the ai numbers them `first + 14`.
- **Mileage**: `RaceSessionMessage::Mileage` → `PlayerCarChoice.setup.add_mileage(race_mileage(len, laps))`
  (`GamePlayer__C1E70(0)` = `dword_6F724C`); `RacelineLength` = first race section's raceline length, filled by the
  binary (`velocity/src/race_car_source.rs`) from `AiState`. The opponent/team/police `RaceCarSource` is the ai
  crate's (`velocity_ai::race_car_source`, installed by its plugin).
- **Flag girl models** (`game_mode::flag_girl_models`): one `SpawnSkinnedModel { pack: FlagGirl, model:
  flag_girl_model(name) }` per intro model (request `0x0F1A6000 + k`), `ModelMatrix` → root `Transform`,
  `ModelVisible` → `Visibility` (hidden until placed). Bone anim items: the peer's `bone_anim` (TimelineSequencer
  0x002B5810) emits `FlagGirlEvent::BoneAnim { model, animation, frame, weight }` (frame = 0x002B5BA0 before the
  wrap, weight = 0x002B5D00); the glue wraps/clamps by the anim's loop flag (`bone_anim_frame`) and sends
  `SkinnedModelAnim { time }`. `FilePayload::BoneAnim` now carries blend in/out, weight, start/end (+0x08..+0x18).
  GameTick Physics set: `PhysicsTimestep` = scaled dt, players' `PhysicsCarInput`
  from UserInput (held with the handbrake before Go), `run_physics_tick`, then car bridge / `TriggerVehicles` /
  `GameAudioInput` / `TimelineWorld` / `RaceStatus` from `PhysicsCarState`. Render (`Update`): car bridge pose
  interpolated onto `CarRender` roots (models are nose +Z, FastCar body nose +X: `MODEL_TO_BODY` = +90° about
  y), FastCar wheels matched to render wheels by position once.
- **Player options** (binary `bridge::apply_options`, from the frontend's `GameOptions`): `controller_config[pad]`
  → `UserInput::set_config(pad, n)` (`Config<n+1>.ini`), `display[0]` → `GameCameras.manager.widescreen`,
  `display[1]` → `gm.vars.speech` (`byte_6F915E`), `display[2]` → `gm.vars.player_names` (`byte_6F915F`).
  Rumble has no consumer (no vibration output).
- For the AI: countdown times = `RaceData.flag_girl` (`in_game_time` +0xCD8, `allow_go_time` +0xCDC), speech =
  `gm.vars.speech`, network = `gm.vars.network_race`; `TriggerMessage::AiHint` is SetCheckpoint (0x21980), the
  AI is its consumer.
- **Per-update cameras** (`camera::game_cameras`): crash cameras + point of interest (CrashCam set), replay
  cameras only in attract mode / review (as InGameUpdate gates them), post race camera, `CameraChange`, the
  camera manager with the flag girl intro camera as the fly cameras' timeline output (mode 1), follow cameras on
  the cars. `RaceSessionMessage::CameraMode` is applied after the Timeline set: **messages written in one
  GameTick do not survive to the next when frames run faster than 24 Hz** — read them in the same tick.
  World camera priority: cut-scene `TimelineCameraView`, `OverrideView` (replay / post race), manager view.

### Frame-rate independence (render side of the 24 Hz split)

- Original render path (VsCommonMain 0x0016DD20 → GameManager::Render → `GameBridge__97C40(lerp)`, lerp =
  `GameManager__BFA50` = timer +0x20): skipped entirely while paused (`GameManager+0x221`); car bridge 0x96E20,
  **camera bridge 0x96A20** (per player: pos/target/up (vec4 lerp `CarDamage__762F0`) + fov lerped between the two
  bridge buffers → `look_at_render` 0x5D540 + fov 0x5D740 + matrix 0x5D2E0 on the manager's current camera; [1] =
  mirror camera `VsCameraManager__5F5D0`; no teleport snap), OSD bridge 0x98470, timeline bridge 0x98A80 (sequencer
  render 0x2B4C50: time = lerp(+0x10, +0x0C) and bone anims 0x2B5810 evaluated *at render*; flag girl model
  matrices: translation lerped, rotation latest, no lerp when the update did not write it).
- Every camera writes the bridge during its update (follow 0x66940 into [0] or the mirror's [1], fly 0x64790,
  VsRender 0x5C220 debug, post race 0xC44A0, replay cameras, TimelineCamera 0x178950 last) — last writer wins.
  Port: `game_cameras::write_camera_bridge` (GameTick, after Timeline, before `camera_modes`) writes the same
  priority; `race_cars::place_camera` draws `GameBridge::camera_view(0, lerp)` (falls back to the update view until
  both buffers hold a record). `place_cars` / `place_traffic` / `place_camera` do nothing while paused.
- Flag girls (`flag_girl_models::apply_events`): bone anim frame = lerp of the last two updates' unwrapped frames
  (frame is linear in sequencer time within an item), wrapped after; `SkinnedModelAnim` goes out every frame.
- The original speedo does **not** interpolate: `Speedo::GetRatioForGauge` reads the OSD bridge's latest record
  (`OSDBridge__98770`, rpm) and the race vehicle's speed directly, so HUD needles step at 24 Hz as in F3_D.
- Dev options (binary `frame_cap.rs`): `--fps <n>` = vsync off + sleep limiter (`--fps 0` uncapped; default keeps
  vsync), `--frame-log <csv>` = per in-race frame: time, updates, lerp, camera pos/forward, player car root, the
  update's raw view. Measured on the smoke test (screen-space 2nd difference of the player car, median / p95):
  30 fps 0.00009/0.0020, 60 0.00008/0.0011, uncapped (~98 fps, debug build limit) 0.00003/0.0004, against
  0.004–0.035 / 0.08 with the old 24 Hz camera at every rate.
- **Smoke test** (more options under "Smoke options"): `velocity --smoke-test` (alias `--slice-test`, implies `-u`) runs menu → quick race (LADown
  circuit) → car select → flag girl intro → drive 12 s, screenshots `research/screens/smoke_*.png`
  (`smoke_1b_opponents`: 2 s after Go, opponents leaving the grid; `smoke_2b_braking`: LTRIGGER held 9.5..10.7 s,
  brake lights). Run the exe with `VELOCITY_GAME_DIR` set.
  Runs to the end (exit 0) with 4 race cars, 14 traffic and 2 police cars bound by the ai; cars are coloured.
- **Recording** (binary `recorder.rs`, smoke test only): `SMOKE_RECORD=<dir>` captures the primary window with
  in-process Bevy screenshots at 30 fps of *real* time (each due 1/30 s step gets the current frame; slow frames are
  duplicated, a capture that never arrives is filled with the previous frame), PNG-encoded on a worker thread into
  `<dir>/%05d.png` (downscaled to fit 640x480, even sizes), then `ffmpeg … -c:v libx264 -pix_fmt yuv420p -crf 23
  <dir>.mp4` on `AppExit` if ffmpeg is on PATH. `SMOKE_RECORD_RACE=1` records only in `EGameMode::InGame`.
  Sound: with `SMOKE_RECORD` set, main.rs disables Bevy's `AudioPlugin` and the recorder's thread is the audio
  device (pulls the visos mixer at 48 kHz real time), writing `<dir>/audio.wav` from the first recorded frame;
  ffmpeg muxes it as AAC (`-shortest`).
  `tools/record_smoke.sh [name] [--env K=V …]` runs it through `hidden_run.py` (race only, log
  `research/video/<name>.log`) and prints `research/video/<name>.mp4`; a circuit run is ~30-35 s (intro, 12 s
  drive, pause/quit), ~1.5 min wall time in the debug build. Delete the frame dir afterwards (~220 MB).
- **Flag girl ground drop** (`flag_girl_models::drop_to_ground`, GameManager 0x000B9010 draw ALPHA): per model i,
  ray (x, grid[i].y + 5, z) → (x, grid[i].y − 5, z), grid = `GameManager+0x44` = eventdef `start_positions`
  indexed by the **model** index (not the player); `VsAITools::28380` → 0x0019D350 flags 0 → 0x0019D3E0 flags 17
  (track | normal), ignore −1; hit → translation y = hit point y. LADown: grid y 8.91 → road 5.79.
- **Stream views** (`game_cameras::stream_views`, GameTick World set; cleared on leaving InGame): per player the
  manager's current camera (`Armco::13F70` = manager +0x24 + 44p) → position, dir and the six planes of VsRender
  0x0005C9C0 (rows −right/up/dir, local normals (∓1,0,a·tan), (0,∓1,tan), near n = dir, far n = −dir,
  w = −n·point; `flt_685094` = −1; follow/mirror cams use aspect 4:3 and the manager's follow/mirror near/far),
  review = `review_cameras != 0`. The binary (`stream_raceline.rs`) adds the raceline: player vehicle p
  (`VsAIDriver::15DB0` = m_pPlayerVehicles) with a race section → (+0x2AC, points `point_ahead` 40·j, j = 1..9).
  **+0x2AC (`VsAIRaceVehicle::24A60`) is the ai's `lateral` (offset from the raceline), not a speed** as the
  StreamView doc calls it; the original streamer's weight (|v| − 16, /44) works on that value.

### Debug menu (in-game tweak menu) — `game_menu_manager::{game_menu_manager, game_tweaks_menu}`, binary `debug_menu.rs`
- **The in-game debug menu is GameMenuManager's, not `CDebugMenu`**: `CDebugMenu::AddChild/AddItem` (0x00082820 /
  0x00082B70) have no callers in F3_D, so the `autoexec.cli` tree (`cli::debug_menu`) is never shown. What F3_D shows:
  - GameInput 0x000A4F10 → GameManager 0x000BC3D0(debug = 1) → 0x000BC0C0 (pause + `VsAudio::Pause`, +0x222 = 1):
    **in a race BACK (`Online` action)** → "Debug" menu 0x000AE160 (Resume 3 → LeaveAll + unpause, Tweaks 2);
    **in the front end WHITE (`MenuL2`) held + BLACK (`MenuR2`) pressed on any pad** (players 0 → 1 first) → "Front
    End" menu 0x000ACDD0 (Tweaks 1, Resume 2 = no case in OnSelect 0x000ACFA0: does nothing, kept; Profiler Mode 3 =
    jump to LACOAST RACE1, Music Manager 4 = 0x000B1950 soundtrack menu: both logged "not ported"). START still
    opens the pause menu (front end's). The button again (either) = LeaveAll + unpause (0x000BC3D0's toggle).
  - Tweaks = GamePauseMenu.cpp 0x000AF970 `EnterTweaksMenu`: root `CGameTweakSystemMenu` "Tweaks" (0x000AFAC0:
    child systems (id i | 0x10000), the system's tweaks (`CTweakableMenuItem` 0x000B01C0, id 0), "Export Tweaks to
    TTY" (1, 0x000AFF60 → `export_tty` = tweak.cpp 0x000851A0, to the log), "Export Tweaks to File" (2, 0x000AFFB0 →
    `Tweaks.txt` next to the exe, the game dir is read-only); 1100 x 600), then re-enters the remembered path
    `gs_tweakStack` (`dword_6F54E8[16]`, depth `dword_6F552C`): OnLeave 0x000AFD90 pops it, OnLeaveAll 0x000AFDE0
    (`byte_6F5528`) keeps it, so reopening after Resume / the button returns to the same system; a vanished entry
    sets depth `i - 1` (original bug kept, -1 loses the next push like the original's write to `dword_6F54E4`).
  - **Editing** (0x000B07E0, current item each update, mask 0xFFFF): MenuRight / MenuLeft press (or held once the
    menu's +1120 timer ran out, 0.125 s after a change) → `increment(1)` / `decrement(1)`; MenuLeftRight beyond ±0.1
    (timer out) → by `ceil(v * 5)` (0x00086390 = CRT ceil 0x003E2A80). Peeks are read-only. A change sends
    `GameMessage::TweakChanged(full name)` (MessageManager channel 1) and refreshes "value (default)" (0x000B0A20;
    peeks show the value only, refreshed only on a rebuild, as the original).
  - `CGameMenu::Update` 0x000AA3E0 as the front end's port (0.2 s up / down repeat, select, cancel = LeaveMenu,
    the last one unpauses) plus **MenuR2 held toggles the menus' visibility every update** (OSD mask 4096;
    select / cancel show them again). Menus are popped one update later (0x000A9C40), as the original.
  - Drawing: `CGameMenu::Build` 0x000AAC30 → `DebugMenuText` OSD items (title 0x000AADF0 text at x + 16 px, flags
    0x1000 = the front end's ANCHOR_CENTRE_Y (y = 500); cursor ">" 0x000AC570; rows 0x000AC860 / 0x000B0270 at
    42-unit pitch, debug font 12, mask 4096); the binary's `debug_menu.rs` makes them items of its own
    `COSDManager` (front end crate, public API) with `CFontHelper::load_game_fonts`, rasterises to a 640x480 overlay
    on camera order 40. **Not drawn**: frame / title bar / scrollbar / tweak slider graphics (`menuComponents`,
    `sliders` textures; the front end's framed menus skip them too).
- Runs in `GameTick` (`InGameUpdateSet::Input`, after `update`, every mode, also paused):
  `font_helper::system_tweaks_in` → `debug_menu_update` (reads `GameMenuMessage`) → `system_tweaks_out`.
- **Gate: F3_D always allows it** (no flag, no `-u` check; only attract mode blocks the button; POI / crash camera
  early-outs of 0x000BC3D0 are not ported). F3_R has the same code ("New DebugMenu"); F3_M has no "New DebugMenu"
  alloc tag (tags are compiled out there, so unknown). The port follows F3_D: available without `-u`.
- Registered so far: **"System"** = FontHelper 0x000B3800 (`font_helper::add_system_tweaks`, at boot =
  GameManager::Init 0x000BADE0 and after every `TweaksReset` = SetGameMode 0x000BBEFE): Enum_Memory_Dump (enum,
  `dword_6F5B68`), System Update Rate (1..60 step 1, `dword_68580C` ↔ `jump_cam::SystemUpdateRate`), Scan Track
  Folder (`byte_6F5B75` ↔ `DebugUnlocks.races`; applied by the frontend's FECK2 callback 0x00202EA0 in
  `velocity_frontend::fe_tweaks`), Texture Viewer (`byte_6F5B65`, no viewer). Cells sync with their resources each
  update (`SystemTweaks`). Test `game_tweaks_menu::tests::system_menu_rows_and_path_survive_leave_all`.
- **Core's registrations** (`game_mode::core_tweaks`, all 13 functions 1:1 from `research/core_tweaks.tsv`, made by
  `tools/ida/core_tweak_regs.py`; details `research/core_tweak_info.txt`): `Reg` builder (`sys/f/i/b/e`, float
  ranges as the passed bits, .data defaults; a bound cell starts at the live value = the original's default),
  `CoreTweaks` keeps `(TweakCell, Option<Live>)` (`Live` = get/set fn pointers on the World, `live!(Res, r =>
  r.field)`); `core_tweaks_in/out` (exclusive) bracket `debug_menu_update`. Registered by `core_tweak_steps` (Last,
  LoadStepMessage cursor; TweaksReset clears the cells) at: **Tweaks** step = InGameInit 0x000B6247.. (GameManager
  0x000B4170 [Debug Cameras, Physics, Graphics.Display/Camera/Shadow, Game, Graphics.Contrast, then graphics'
  SpecularChars, VibrateEffects 0x001EDA50 ×8 "controller.vibrate.<EFFECT>" from `off_688C10`], POI 0x0006A930,
  Crash trigger 0x00063160, Crash camera 0x00061D30); **CompartmentPreInit** = CompartmentManager 0x00087010
  (vtable 0x0062BB7C slot 0; step choice a guess); **CameraManager** = VsCameraManager 0x0005ECC0 → 0x00065920
  (Camera.Globals.*) then 0x00065640 (Camera.TurnRates, Camera); **AiWorldData** = ScoreManager 0x00167F30 (called
  from 0x00017F60 = per vehicle, local only; the function is ReplayCamera_Wheel-labelled); **ReplayCameraManager** =
  0x00165640; **HeatHazeInit** = 0x000C9450 (from the heat haze load StringManager 0x000C90A0). Boot: GameManager::Init
  0x000BAE37 → "Network" 0x000CDD70 (dropped by the first TweaksReset). net 0x000CF200 (Network.Timings) only from
  the network session code: ported (`add_network_timing_tweaks`) but never called.
  - **Live**: Camera.Crash (both) → `GameCameras.crash.{tweaks,trigger}`, Camera → `manager.camera_tweaks`,
    Camera.PointOfInterest → `poi_tweaks`, Widescreen → `manager.widescreen`, Enable Cameras For Review →
    `replay.review_cameras`, Texture Viewer shares the System cell (same global), Game.ScoreManager.* →
    `ScoreTweaksRes` (the binary's `ai_host::score_tweaks` copies it into every ai vehicle's `ScoreManager.tw`, plus
    the frontend's "Game.OSD" Split Time Display Duration → `ScoreTweaks.split_time_duration`, read by
    `lap_positions` 0x0016BE50 for both gaps).
  - **Live camera globals (2026-11-06)**: Camera.Globals.* / Camera.TurnRates / ZRotation →
    `GameCameras.manager.globals` (`FollowGlobals`, 56 fields, `Default` = `k::` .data values; `glob!(field)` in
    core_tweaks); `update` / `update_follow` take `&FollowGlobals` (read every update, as the original's globals).
    `k::` keeps only the never-written globals (slope, YAW_MAX_RATE, LOOK_AROUND, soft limit, bounce lift, impact,
    FOV_FURY, Camera view defaults). Yaw slew rate (`dword_6851F0`) has no reader. 0x00065610 (end of 0x0005ECC0)
    sets slew / max angle from the vehicle database's GLOBALS.BIN +36 / +40 (`FollowGlobals::set_turn_rates`;
    shipped 120 / 90 = .data). **Graphics.Camera** → `manager.clip` (`ClipTweaks` far 600 / near 1 / mirror on /
    mirror far 200): read at camera setup (init 0x0005ECC0, intro 0x000B73D0, free / 3d interface 0x000B7730 /
    0x000B75D0), `change_view` writes `clip.near` = the view's [17] (0x0005F140 `dword_6850A4`; persists into the next
    race's setup, original), place_mirror reads `clip.mirror` (0x000B88B0). **0x000C0A90** = a MessageManager
    channel-1 listener (`dword_6F5B98`, vtable 0x00631A08, added 0x000B4170 / removed 0x000B8550): on *every*
    TweakChanged, player 0's current camera (+0x24 by mode) gets far · (0.5 split) and near, its mirror the mirror
    far → `VsCameraManager::clip_tweak_changed`, system `game_cameras::clip_tweaks_changed` (GameTick Input, after
    `core_tweaks_out`, in game only). So any tweak edit sets the chase view's near to `clip.near` (original). Its
    tail (`byte_7B21E1` "Display Debug Graph" → `unk_7AB208`+0x6FD8) is not ported. Clamps whose bounds/inputs are
    tweaks use `VsClamp` 0x000141E0 (`comiss; jbe` → lo on NaN): fovMin = fovMax gave 0/0 → NaN eye → Bevy `Dir3`
    panic before. Requests: physics (Handling.cfg writes into 0x6850A0/68519C/6851A4/6851B4), frontend (3d interface
    planes). Dumps `research/core_camtweak_{xrefs.txt,dc*.c}`.
  - Test `camera::vs_vehicle_follow_camera_golden`: a scripted 480-update drive (all branches: kite, bounce, jump,
    drift, zone, one-off, fury, view 1, look back, lag / detached, poi, collision, sliding, impact, static bumper view,
    mirror) hashed over every output bit = `0xC8D980B25C0AF156`, recorded from 8009e87 (the `k::` build) → bit-identical
    at default tweaks; plus edited globals change it, fovMin = fovMax stays finite, defaults = the .data bits.
  - Seen (`SMOKE_DIRECT=1 SMOKE_DEBUG_MENU=1 SMOKE_DEBUG_MENU_PATH=Camera.Globals.FOV.fovMinOveride
    SMOKE_DEBUG_MENU_STEPS=20 SMOKE_DEBUG_MENU_SHOT_CLOSED=1`, log `research/core_debug_menu_cam.log`): fovMinOveride →
    80, camera fov 1.0472 → 1.3963 rad 1.5 s after closing (`research/screens/debug_menu_cam.png`, wider view); exit 0.
    ZRotation rotates the up about **world** Z (Renderer 0x002AB580, 0x00066940): along world X it barely shows.
    Plain `SMOKE_DIRECT=1` race exit 0 (`research/core_cam_tweaks_race.log`).
  - **Unbound** (cell only): Debug Cameras, Physics, the debug displays, Render* flags,
    Graphics.Contrast (`byte_6F5C78`, graphics' StartLineEffects object), TrailBlazer Start/End Speed (ai's
    TrailBlazer 0x000C4FF0), HeatHaze, Compartments, vibrate (no output), Network, Reload Replay Camera File.
  - Test `core_tweaks::tests::registrations_match_the_original_calls` (every call vs the TSV: order, names, float
    range bits, step, int ranges, enum elements, .data defaults) and `live_binding_round_trips`.
- **Graphics' registrations replayed** (`Reg::gfx(origin)` over `GraphicsTweaks.calls`, one cell per slot in
  `CoreTweaks.gfx`, synced with `GraphicsTweaks.values` in `core_tweaks_in/out`): `core_tweaks::GFX_SITES` maps each
  PHASES call site to the load step running there (CommonInit sites → FullScreenEffects / AdditionalShaderData /
  GlowInit / FogInit / TrackFxInit / FadeRenderInit / ReflectionInit, in both modes' common init; FE → FeCarRender /
  FeDofRender / MtvEffects; InGame → Tweaks (fog) / ProjectedLightBeams / SlipStreamRender / RainRender / DofRenderInit /
  CMeshRender / SkyRender / Barriers ×2 / CarRender / FuryFx; SpecularChars inside 0x000B4170; police CarDamage
  0x0006B8B0 at AiWorldData when the race has police). Site → step is by address order, not verified per step.
  **Audio and physics** the same way (`Reg::ext(ExtSrc::{Audio, Phys}, origin)`, cells in `CoreTweaks.ext`, synced
  with `AudioTweaks.values` / `PhysicsTweaks.values`): audio `ADD_TWEAKS_ORDER` at the **in-game** AudioInit step
  (CommonInit 0x000B5169 → VsAudio::Init → 0x000484D0; nothing in the FE); physics 0x0019BEC0 (Materials) inside
  the Tweaks step at 0x000B6251 (between POI and the crash trigger), 0x001B8A00 + 0x001DC650 at PhysicsManagerInit
  (0x000B679E). "Debug Cameras.Follow vehicle" (`dword_688B80`) has no core consumer (SyncModels not ported).
- Seen (2026-11-06, `SMOKE_DIRECT=1`, logs `research/core_debug_menu_{ai,gfx,audio,phys}.log`): `AI.Difficulty Scale`
  → "tweak AI.Difficulty Scale = 0.65" (`research/screens/debug_menu_ai.png`), `Graphics.3DLightBeams.Extent` → 44
  (`debug_menu_gfx.png`), `Audio.ENVFX.DopplerFactor` → 0.4, `Turbo.Rise rate` → 0.2; all exit 0.
- **Smoke**: `SMOKE_DIRECT=1 SMOKE_DEBUG_MENU=1 [SMOKE_DEBUG_MENU_PATH=AI.Difficulty Scale] [SMOKE_DEBUG_MENU_SHOT=x.png]
  [SMOKE_DEBUG_MENU_STEPS=n (RIGHT presses)] [SMOKE_DEBUG_MENU_SHOT_CLOSED=1 (BACK first, shot 1.5 s later; logs player
  0's camera at open and at the shot: "smoke-test: camera 1/2")]`. Nested systems are path parts
  (`Camera.Globals.FOV.fovMinOveride`; ZRotation sits in `Camera`, not `Camera.TurnRates`).
  (race, 3 s after Go): BACK, then "Tweaks" + the dotted path (default `System.System Update Rate`): each name is
  looked up in the top menu's rows, DOWN/UP to it, A on a system, RIGHT on a tweak (last name), shot
  `research/screens/<shot>`, BACK closes; a missing name logs the rows and shoots. The rows sit in the upper half,
  values at x + w − 90 run off the right edge for long enum names (same geometry as the original); the title
  (flags 0x1000) at y = 500 over a row is original behaviour (frontend checked 0x000EE5F0). IDA dumps
  `research/core_dbgmenu_dc*.c`, `core_dbgmenu_vt.txt` (vtables), `core_tweak_regs.txt` (every AddTweaks caller).
- **Not done**: "System" registers at the reset, the original adds it after the mode's init (so it lists first in
  the original's root, last here behind the frontend's late registration).
- **AddTweaks registrations**: every caller in `research/core_tweak_regs.txt` is registered (ai, frontend directly;
  graphics, audio, physics replayed by core; core's own in `core_tweaks`).

### Unlock mode (`-u` / `--unlock`) — `cli::debug_unlocks::DebugUnlocks` resource

- F3_D takes its command line only from `XGetLaunchInfo` (debugger cmdline, main.cpp 0x0016CE50), so a
  dashboard boot has no args; "everything unlocked" comes from three debug-menu bool tweaks, applied by the
  tweak-changed callback `FECK2Manager.cpp 0x00202EA0`:
  "Unlock ALL Vehicles" `byte_6877F0` (**the only bool tweak defaulting to 1** in .data; read by
  `CVehicleDatabase::GetEntryLockState` 0x18D2A0), "Scan Track Folder" `byte_6F5B75` (-> 0x00094990 every race
  unlocked, incl. `Available: NO`), "Unlock ALL Events" `byte_6F0D70` (-> 0x00092CD0(event, 1) for every career
  event). `jumptogame`/`profiler`/`scantracks` call `GameManager 0x000BE7E0` (+0x35 = 1, races unlocked);
  main calls 0x000BE810 (lock) first. All 147 bool tweaks: `research/core_booltweaks.tsv`.
- Port: `-u` sets all three; without it all off. Races: `boot` (game copy) and `velocity/src/bridge.rs`
  `frontend_unlocks` (FE's own `fe.events` copy + `quick_race.all_unlocked`); career events: `boot` calls
  `CEventDatabase::unlock_all_events` after the build. `--slice-test` implies `-u` (it picks a
  shipped-unavailable cruise); the fake unlock in the harness is gone.

### Career events — `event_database::{event_database, event_database_events, event_database_bin_loading}`

- `CEventDatabase::build` (0x00090090, run in `boot`): TRACKS.INI, then `\EVENTS\EVENTS.VCD` (97 `*.bin`, f32 1.5,
  u16 count, events), link passes 0x8F450 (prerequisite events / vehicles), 0x8F620 (race slot counts; cups
  `n * 0.5^(rounds-1)`), 0x8F800 (track by event area + crc, race by category + type + crc), unlock evaluation
  0x8F020 (repeat ≤50 passes: prerequisites completed, vehicles owned, prestige, completed-in-tier count).
  Record layout is in the `event_database.rs` header. Events are kept in the original's list order (0x92E80
  insert-before rule). Shipped data: 22 race entries do not resolve (absent from TRACKS.INI or other category),
  1 dangling prerequisite — the original warns identically.
- `CEvent` = 96-byte record (+0x00 category 1 UG / 2 pro, +0x04 kind 1 single / 2 cup / 3 series, +0x15 flags
  1 locked 2 completed 4 failed 8 temp 0x10 finished). Rounds/races/competitors are Vecs, pointers are indices.
  Progression: `reset` 0x920E0, `change_and_set_up_round` 0x922B0 (cup winners), `race_finished` 0x91620,
  `round_finished` 0x91810, `update_at_end_of_event` 0x91890 → `EventOutcome` (prize 0x8EA60 tables, wager, team
  unlock, reward). Completion → `CEvent::complete` (0x92D40). Wired to race end by `race_session::race_over`
  (0x000BDB50, finish positions from `RaceProgress.order`).
- Temp events (quick race / jump): `build_temp_single_race_event` 0x93630, `build_temp_series_event` 0x93920;
  opponents are named in turn AL, BUD, JULIO, ROBYN, JAVIER (crc32 of the upper name). Quirk kept: the temp result
  needs `points <= 0xFFFFFFFF`, so a temp series always ends "failed".
- Current event: `GameManager.vars.event` (`dword_6F9BE8`, `EventRef::{Database, Temp}`), read with
  `gm.current_event()`; set when the pending jump resolves (`front_end_update`): `GameFlowRequest::StartQuickRace
  { event: QuickRaceEvent }` (OnSelect 0x221F00 globals, filled by the binary's bridge) or the jump's own event.
- `game_mode::game_player::CFEGameVariables` (`g_FEGameVariables` 0x006F6F70, SetInitialVariables 0x000C1930):
  cash 250000, garage 300ZX/RX8/NSX, prestige, reward, speech (`byte_6F915E`, default on), network race
  (`byte_6F7158`, off; set only by the online session), flag girl index (`byte_6F70F0`), unlocked tiers.
- For the AI (PhysicsManager 0x0019E080): race cars = `current_event().current_race().competitors` (slots →
  `CEvent.competitors[i]`: `human != 0` = player, else `opponent_crc` → `OpponentDatabase::get_opponent_by_crc`);
  opponent vehicle = tier `event.tier - 1` (0 if tier 0) setup, `\vehicles\opponent\<short>\tier%d.vcd`; for
  career categories 2/3 (non-temp) the team car instead. Countdown: `RaceData.flag_girl` (`FlagGirlTimeline`,
  loaded on the FlagGirlTimeline step from `\Timeline\FlagGirl.vcd` entry `flag_girl % 10`): `in_game_time` =
  TimelineSequencer +0xCD8 (F3EventCamChange "InGame" − 1e-5, the intro-skip / countdown start), `allow_go_time`
  = +0xCDC (F3EventFlagGirl "ALLOWGO"). Speech `gm.vars.speech`, network `gm.vars.network_race`.
- `visos::handy::crt_rand::CrtRand` = CRT `rand` 0x3DE030 + `VsRandUnit` 0x13510 (`GameManager.rand`).
- Fixed in visos: timeline typed-event data pointers (strings, var arrays) are relative to the data block itself
  (FixupTimelineItems 0x2EE0D0 → FixupDataOfType base = data), not the timeline chunk.

### Career ("The Scene") — `game_mode::career`, `register_file_types::save_file_data`

- **Save = "Primary" block** (`dword_701428`, 18656 bytes; SaveFileData.cpp is really 0x001044D0..0x00106B30, idb
  "ParticleManager"): `PrimarySave` mirrors it field for field (offsets in the struct; test `layout_round_trip`):
  version crc "Version_25" 0xA248A95E, garage per slot as struct-of-arrays (vehicle / wheel / tyre / brake / colour
  brand+hash / bodypart set+part / 20 decal hashes, mileage ftol), owned wheels (`+0x2A == 1`, 30), bought bodyparts
  (flags & 8, 500), the 84-byte vars block `dword_6F719C..` (cash, prestige, count, garage crcs, current car, slot,
  reward, +0x25C, team `dword_6F71D0`, 7 undecoded words), the 2688-byte options block `byte_6F915C` (+1 16:9, +2
  speech, +3 names, +0x25 pad-0 Config index, +0x28.. audio playlists 0x00105E10 kept opaque), race records of tracks
  with an unlocked race (id crc, best record, best time; 300), events in list order (crc, completed, locked; 150) +
  completed count, 10x324 custom alloys (`unk_6F83AC`, opaque). Gather = 0x00106040/0x001061D0 (0x00105370, 0x00105890,
  0x00105960, 0x00105B10, 0x00105CB0); apply = 0x001050C0 (version check, 0x001044D0 garage via SetupDefaultVehicle +
  finds, current car = saved slot -> `PlayerCarChoice`, 0x00104C40 wheels, 0x00104CC0 parts flag 8 / +0x1C = 0,
  0x00104D80 race records by `+0x08` id, 0x00104E30 event flags, `byte_6F9058`, UnhideVehicles / UpdateLockCount).
  Not applied here: audio playlists (0x00104F30), team lock refresh (0x0016E790, ai), alloys (frontend).
- Storage replaces SAVE_Flow_*_M: `<root>\<profile>\Primary`, root = `VELOCITY_SAVE_DIR` or `%APPDATA%\Velocity\Saves`
  or `<exe dir>\saves` (never the game dir); write via tmp + rename; `list_profiles()`. Test
  `career_save_load_round_trip` (real data: gather -> disk -> fresh databases -> apply -> gather is identical).
- `CFEGameVariables` gained `team`, `unknown_264`, `profile_name` (`unk_6F7174`), `garage_setups` (+0x734, 556 B
  each; `init_garage_setups` = FECK2Manager 0x002022F0 at boot, `add_garage_vehicle` 0x000C20F0, `garage_setup`
  0x000C1F90), `save_enabled` (`byte_6F9054`), `profile_active` (`byte_6F9055`), `completed_any` (`byte_6F9058`).
  `Career` resource: save name (`byte_701388`), last `primary` block, the game's `CVehicleDatabase` (kept from boot).
  `UserInput.config` = per-pad Config index (+0x14).
- Requests (`GameFlowRequest`, handled by `career::career_requests` after `game_flow_requests`): `NewProfile{name}`
  (0x00234550: SetInitialVariables + event reset + garage setups, flags on), `LoadProfile{name}` (0x002345E0 +
  0x001050C0), `SaveProfile` (0x00234700), `DeleteProfile{name}` (0x002347F0), `StartCareerEvent{event crc}`
  (`m_pEvent` = event, StartEvent 0x0028F430: competitor 0 human, reset 0x000920E0 (+`dword_6F7244` = 0), category 2
  with a team / 3 -> team car (`RaceCarSource::team_car_by_team`, ai), 4 -> opponent `+0x44` tier car, players = 1;
  then the jump to the event's current race with `career_start`, so `front_end_update` keeps the database event).
  Answers: `CareerMessage::{Created, Loaded{ok}, Saved{ok}, Deleted{ok}, EventStarted, ReturnedFromRace{saved}}`.
- Race end: `race_session::apply_event_outcome` = 0x00091890 tail (was missing: outcome was computed, never applied):
  success -> reward += `+0x18`, `CEvent::complete` (unlock pass, message -> `unlock_message`); then cash += prize ±
  wager. Back in the FE (`OnEnter(FrontEnd)` after any race) `race_complete` = RaceCompleteScreen Enter 0x002055F0 +
  Start 0x00205830: free a temp event, autosave (through `SaveFlow`, below) when `save_enabled` and not `quit_race` (`byte_6F5D08`), then prestige +=
  `dword_6F7244` for a database event (after the save, kept). The career event stays in `vars.event` for the hangout
  (0x0028F790 clears it at step 8 — frontend).
- **Options block** (`save_file_data::OptionsBlock`, the 0x28-byte head of `byte_6F915C`; 0x001050C0 / 0x00106040 /
  0x001061D0 copy all 2688 bytes): +0 `unk_00` (only init / save touch it), +1 16:9, +2 speech, +3 names, +4
  soundtrack i32 (`dword_6F9160`), +8..+0x1C volumes in address order master, music, engine, **tyres, speech, sfx**,
  +0x20 audio mode, +0x24 `time_format` (`byte_6F9180`: COSDManager::FormatTime flag / "--:--" vs "--:--.--",
  VsCameraManager 0x0005F0F0), +0x25 pad 0 config (written from UserInput just before a save), +0x26 rumble, +0x27
  pad; then playlists +0x28 (3 x 16 list headers), +0x58 (150 x 12 tracks), +0x760 (100 x 8 type / shuffle).
  `Career.options` = the live globals; `gather(.., &OptionsBlock)` writes it (speech / names from `vars`),
  `Applied.options` = the loaded one (load also sets `byte_6D45C1` = 1, the audio menu's "settings changed" flag:
  the front end's GameOptions change detection does that job). `new_profile` resets it to the 0x000C1930 defaults
  (keeps +0x25). Binary (`bridge.rs`): `profile_options` pushes `Career.options` into `GameOptions` on
  `Created` / `Loaded{ok}` (volume order mapped), `apply_options` copies `GameOptions` back every frame. Not in the
  block: jump / crash / police cam, language (`options.ini` only). Test `options_block_layout`.
- **Save flow** (`save_game::{save_vars, save_flow_save_m}`, `SaveFlow` resource; SaveGame\_SAVE_vars.cpp 0x00304D80,
  _SAVE_Flow_Save_M.cpp 0x002FFD30, node table `unk_6C1FD0`, XMem.cpp poll 0x0030EF60 / answer 0x0030EF50). The
  debug XBE **does show after-race popups**: 0x001061D0 saves via 0x00300680 (no overwrite question), so the flow
  is Saving (status 2 popup `SaveGame_Save_BeginSave_Title` / `SaveGame_Save_BeginSave`, flags 128, four write steps,
  min 3 s from the node's start) -> SaveComplete (status 2 `SaveGame_Save_SaveComplete_Title` / `..._SaveComplete`
  "Save Successful.", returns at once, then 3 s) -> Exit (status 3). Between nodes the poll returns status 0. A write
  error -> ErrorSaving (status 1 question, flags 401, option `SaveGame_OK` / response `OK`) -> UserExit (-1).
  ConfirmOverwrite (status 1, flags 931, Overwrite / No) only for 0x002FFD00 (profile select save, a6 = 0) over an
  existing save; NoSpace / Reboot never (no block count). Port: `start_save(path, bytes, no_confirm)` writes as soon
  as Saving is entered (the file is on disk even if nobody polls), `poll(dt) -> SaveStatus` (status, flags, title /
  text / option / response crcs, English fallbacks), `respond(crc)`, `in_progress()`, `written()`.
  `race_complete` uses it (`ReturnedFromRace{saved}` = the write succeeded); `GameFlowRequest::SaveProfile` still
  saves directly (no flow). Tests `save_flow_save_m::tests`. IDA dumps `research/core_saveflow.c`,
  `core_savetable.c`.
- Binary glue (`bridge.rs::career_sync`): frontend garage -> core every FE frame, core -> frontend after
  Created / Loaded / EventStarted / ReturnedFromRace; EventStarted sets the FE post race / return screens
  (HangoutScreen, MainMenuScreen fallback) and stops the FE. Frontend / ai requests are in their docs.
- Smoke: `SMOKE_CAREER=1` always runs on a **fresh save dir** (`smoke_test::fresh_save_dir`, top of `main`: a new
  `career_<ms>_<pid>` folder under `VELOCITY_SAVE_DIR` or `<exe dir>\smoke_saves` becomes this run's
  `VELOCITY_SAVE_DIR`). The old stall was the smoke's: a leftover "Q" in slot 0 made A a LoadProfile, so `Created`
  never came and the main menu step waited for it; the game behaved. After the reload the hangout's restart of
  UT0EVENT1 is now driven as a **second race, 20 s** (`6_career_second_race`, then exit); seen: second race loads,
  player at 2nd/4, exit 0. Flow: main menu Start ->
  ProfileSelectScreen A on the empty slot (front end makes + saves "PLAYER1") -> main menu DOWN + A (career) ->
  HangoutScreen starts UT0EVENT1 (0xF0CA4F4C) itself -> race, quit -> RaceComplete (no autosave after a quit since `quit_race` is honoured) -> HangoutScreen
  (`5c_career_back`). The hangout restarts UT0EVENT1 on its first update while `completed_any` is false (0x0021C430,
  original), so a quit tutorial can never be left through its quit dialog: the smoke reloads the profile with
  `LoadProfile` on `ReturnedFromRace`, logs cash / garage, then drives the second race (above). Clip
  `research/video/core_career.mp4` (older, single race). `SMOKE_EVENT` is gone.
- Plain smoke: main menu Start -> ProfileSelectScreen B -> main menu A (quick race) -> ... as before; exit 0.

### Cameras, replay cameras, markers — `camera::*`, `replays::cameras::*`, `controller::vibrate_effects`

- **Region labels are shifted here**: the VsFlyCamera ctor is in VsCrashCameraManager's range (0x63FA0), the
  follow-camera ctor/params in VsFlyCamera's (0x653B0, 0x65320), the fixed replay camera's core in
  "ZoneRender_PC_XBOX" (0x163930..0x164B80), the wheel camera in "ReplayCamera_Orbit" (0x167860..), and
  "ReplayCamera_Wheel" from 0x167F30 on is ScoreManager. Name by class, not by idb region.
- Engine math (row vectors, D3D memory): `v*M` = glam `M * v` with the same 16 floats; D3D `A*B` = glam `B*A`;
  RotX/Y/Z and D3DXMatrixRotationAxis match glam's; **AC::Renderer 0x2AC2C0 (quat→matrix) rotates by the
  conjugate** → `vs_base_camera::quat_matrix`; Renderer 0x2AB7E0 = `yaw_pitch_roll(a,b,c)` = Rz(c)·Ry(b)·Rx(a).
- `VsBaseCamera`: logic frame +0xD4.. (pos/right/up/dir), render copy +0x48.. taken by `copy_to_render`
  (0x5D250), fov is vertical radians, `render_transform()` gives the Bevy transform. `VsCameraManager`
  (`g_CameraMan`): per player follow + mirror (`VsVehicleFollowCamera`), fly cam (mode 3 "cine"), roaming debug
  cam (4), 3d interface cam (5); `change_view` 0x5F140 cycles the car's view table (VehicleData list a,
  `PlayerCarChoice.camera_views`) under the per-player mask. `CCinematicCameraSet` (0x60790) and the replay
  manager's `DecideCameraShot` (0x165FF0) are referenced by nothing — not ported (both added as functions in
  the idb).
- `VsFlyCamera` (per player, GM+0x22C): mode 0 by hand (crash cam), mode 1 from a timeline camera output
  (flag girl intro; heights relative to ground ±5 m around the grid). InGameInit ends with every player on its
  fly cam (mode 3) following the intro unless network. `VsRoamingCamera` = debug free cam (actions
  DebugFreeCam*, world frozen = GM+0x22A unless DebugFreeCamPlay held; toggle 0x64BC0).
- Crash camera (`g_CrashCameraManager` 0x6D58B0): collision message → head-on test (0x63620) → spline round
  the car (Catmull-Rom 0x61710, original bug kept: 4th control point = 3rd), WMM mode 2, time stretch 0.2,
  respawn on stop. **Off by default** (`byte_6D58A8` bss). Point of interest (WMM mode 1) also off by default
  (`byte_6D60B9`); both expose events instead of touching OSD/AI.
- Replay cameras (no replay recording/playback exists in F3_D; see "Attract mode, review cameras" for when they
  run): `CReplayCameraManager` updates from session state ≥ 5 (Racing) while the attract mode plays, "Enable
  Cameras For Review" (`dword_749110`) is on or +0x1BE is set; CAMERA_SWITCH triggers (`target_crc` =
  camera id) pick a fixed camera, else wheel cam, else orbit. `cameras.cam` ("FXDCAMPOS", v4; layout in
  `replay_camera_fixed.rs`): 65 files, 63 parse, London Circuit Race04 and London Drag Race01 truncated as
  shipped. Fixed cams slide on dolly tracks, turn ≤ max angle, zoom 10..250 m.
- **Chase camera** (`vs_vehicle_follow_camera`; dumps `research/core_chase_*.c`): `update` = 0x00067510 in full,
  `update_follow` = wrapper 0x00066940, `camera_point` = camera half of VehicleEntity 0x001C5CF0 (glue
  `game_cameras::follow_inputs` reads VE memory). The original runs it inside the physics frame
  (0x0019F920 → SyncModels 0x001DDAE0, after ResolveCollisions, same dt); here it runs in the Cameras set right
  after Physics, on the same state. Fixes against the old port: fov is the base camera's **vertical** fov
  (+140 = rad(+548); the old 4:3 horizontal→vertical conversion made the view ~20% narrower); upward slope limit
  `flt_6D5E3C` is bss 0, not +30; speed = VE+20, heading = VE+4788 row 0, velocity = FastCar +32; the eye follows
  the 0x001C5CF0 point (camera_shift_forward, **skid side shift VE+4468/+4476 along the camera look/right while
  VE+4456 is on** — the drift side look — and the CVI vertical inertia spring, state in VE+4436/+4440/+4444,
  written back); velocity turned by VE+4480; lag VE+4484 (detached camera < 0.01 = CameraRotateHoriz look-around);
  VE+4504/+4516 distance/pitch; "kite" +540/+552/+556 (recede 0.75 m on throttle, close 0.6 m on brake, also
  pitch; this is what the header called the gear-change sway); bounce spring → pitch (`byte_6851D4` = 1);
  zone ramp → +25° pitch down; fury (graphics' `FuryFx::amount(player)`, 0x00128680) → kite
  and +20° fov; view 1 ×1.2, view > 1 eye on the car; point-of-interest blend + fov 0x0006A570; camera collision
  0x000692C0; banking (up rolled about the look up to 8°·|steer| above 15..30 m/s); speed shake; player 0's view
  overwritten by the "Camera" tweaks every update (0x00066830; = the view's own values, LADown view 0 height 1.7).
  Clock (`FollowExtras.game_time`) = `VsAIDriver__15D50(&unk_6F5BC8)` = GameManager +0x208 = `RaceSession.race_time`
  (was `timer.game_time()`); only the +5008 shake decay reads it. The manager's +40 cut flag is the
  snap (`PlayerCams.cut`). Record [14] (+524) is the chase flag (was misnamed `mounted`, now `chase`).
  Tests: `slide_leads_the_look_towards_the_velocity`, `fov_and_distance_follow_the_speed`,
  `throttle_kite_recedes`, `soft_limit_matches_68fa0`, `static_view_sits_on_the_car`.
- **Static camera** (0x00066940 else-branch + 0x000690B0 `update_static` + 0x0005DB10 `look_at`): runs when the view
  is not a chase view ([14] = 0) and no point of interest is up (+596 `dynamic`; a change of it = render cut + +40
  cut). +585 = pad look back; flag = +496 rear | +585; look dir = flag ? −heading : a8 = this update's car bridge
  rotation applied to model +Z (= `PhysicsCarState.rotation · X`); point = car point + dir·20; eye = car point −
  normalise(car − point)·(flag ? [4] rear : [5] bumper) + car up · [2]; target = point + up · [0]; up = VE+4788 row 1
  (0x001C6630), no banking / collision; fov = [11], or lerp([11], 20 (`flt_686B5C`)·[16] + [11], fury) unless the
  in-car view (0x00060260: view 3); writes only +140. LADown car: views 0/1 chase, 2 bumper, 3 in-car.
- **Mirror**: 0x001C5CF0 runs VE+4936 (the mirror) right after VE+4932 with the same inputs (shares the +40 cut and
  the VsRandRange stream; sees the follow camera's new VE+5008). Mirror = record 0 with [14] = 0, [15] = 0, +496 =
  +497 = 1, so it is always static, looking back; its record goes to bridge [1] (+52 when +496 && +497).
- **"Camera" tweak globals** `dword_685204..685214` = `VsCameraManager.camera_tweaks` (`CameraTweaks`): .data
  defaults 1.8/1.8/9/9/40, **0x000668C0 copies player 0's view into them** at init and on a view change in mode 1,
  0x00066830 copies them back over player 0's non-mirror view every update. Ported 1:1 for every view (user
  decision): the .data defaults are never seen in a race (LADown view 0: height 1.7, poi fov 75). A view changed
  outside mode 1 keeps the old tweaks (original). Also kept (user chose no blend): `change_view` sets the +40 cut (0x0005F140 itself does not call 0x0002C620; only a [14] zero-ness
  change does, via +584), so chase → chase view changes snap.
- **Renderer cut** = GameBridge +0x14 (`GameBridge.cut`; 0x00060380 sets it, swap 0x00097A00 clears it, read by
  0x000BFAB0): GameManager::Render 0x000BBB00 then draws the whole bridge (cars, cameras, OSD, timeline) and
  0x000B9010 the physics objects with lerp 1 (`GameBridge::render_lerp`, used by `place_cars`, `place_camera`,
  `place_objects`, flag girl models, DOF car position). Setters ported: view change, follow `update` (look-back
  flip, +40 cut consumed), +596 change, fly cam timeline cut (0x00064090), crash cam start/end (+ 0x0002C620(0)),
  post race camera 0x000C4460 (+ 0x0002C620(0)), TimelineCamera begin / end (`CutSceneCamera` message; end also
  0x0002C620(0)); cameras leave requests in `render_cut` (camera / manager), `write_camera_bridge` moves them to the
  bridge. InGameUpdate's review-camera timer (0x000B837F) = `jump_cam::review_timer_countdown`. Not wired: pause 0x000BC0C0 (=1) /
  unpause 0x000BC2C0 (=0) (render is skipped while paused, so only the order corner case differs), and message 3
  of 0x0002C620 (MessageManager CameraCut, listeners not looked at).
- **One-off object** (0x00023310) = the **ai race vehicle's trailblazer** (+0x3C8, written by 0x00023250), running
  = 0x00024F10 (+0x8C spline != 0), value = TB +0xA8 progress: `trail_blazer::PlayerTrailBlazers` per player, filled
  by the binary (`velocity/src/trail_blazer_state.rs`, GameTick after Ai, before Cameras); read by the chase
  camera (zone · 1 − p²) and `ZoneFxInput.one_off` (player 0).
- **HeadlightView** (graphics' request, `game_cameras::headlight_view`, Update, InGame; default on exit):
  `camera_mode` = mode(0); `inside` = player 0's race car physics id (= its `CarRender.request`) when view 0's mode
  1 follow / mode 2 mirror camera is not an outside view (!0x00069880: [15] == 0 || (in-car view && poi running)).
  LADown: inside on view 2 (bumper, [15] = 0), not on 0/1/3.
- Remaining: VE+4392 / VE+5008..+5020 have no physics writer (0); the VsRender debug override 0x0005C210 (bss 0).
- Seen (`SMOKE_CAMS=1 SMOKE_DRIVE=24`, LADown): BLACK every 4 s cycles views 1, 2, 3, 0 (bridge cut on each),
  WHITE look back on view 3 and view 0, exit 0. Clip `research/video/core_cams.mp4`; shots `rec_3a..3d_view*`
  (`3e_look_back` is not reached: the look back lasts exactly the 1.5 s shot delay).
- **Smoke shots lost while recording (fixed)**: Bevy's `extract_screenshots` despawns a second primary-window
  `Screenshot` spawned in the same frame ("Duplicate render target" warning), so the recorder's per-frame capture
  swallowed smoke shots (13 lost in the old cams clip, all `3*_view*`). `smoke_test::shot` now queues the path and
  `flush_shots` (PostUpdate) adds `save_to_disk` observers to the capture already spawned this frame, or spawns one.
- `Markers.mfk` (f32 1.0, count, 48-byte markers, name crc +0x24): all 3 shipped files are empty; used by
  PostRaceSequence ("POST_RACE_STUNT_AREA").
- Wiring (`camera::game_cameras::GameCameras` resource + `CWorldModeManager` resource): InGameInit camera load
  steps, markers and replay init/CheckForLoad run; the per-update calls run on the physics cars (see "Per-update
  cameras" above; `CamWorld` implements `CrashCameraWorld` / `ReplayWorld`).

### Attract mode, review cameras — `font_helper::CAttractMode`, `replays::cameras`, binary `attract.rs`
- **`g_AttractMode` = `byte_6F5A28`** (0x4C bytes; its code is in FontHelper.cpp's range 0x000B3000..0x000B3730,
  hence `velocity_game::font_helper`): +0 playing (0x000B36B0), +1 fixed race, +2/+0x22 track/race char[32], +0x42
  timer on, +0x44 timer, +0x48 quitting (0x000B36F0). .data: idle `dword_685698` 300 s, race `dword_68569C` 45 s,
  movie chance `dword_6856A0` 0.1; `attractmode <track> <race>` (main 0x0016D228 → 0x000B3060) = that race, 1 s,
  300 s, 0. Front end (FEMainMenuScreen Update 0x0023C400 → 0x000B30E0 each update; Up/Down/Select → 0x000B34A0
  reset; Leave → 0x000B3470): idle runs out → 0x000B34D0 `srand(GM+0x190 timer)`, VsRandUnit ≤ 0.1 → intro movies
  In001..3 (0x000B3620, not played: Bink), else playing = 1 and 0x000B3550 → GameVars 0x000C2810 = CGameJump::Set +
  Execute("MainMenuScreen") (= post race screen, GameManager 0x000BE7C0): a random row of `aTracksLondon`
  0x006856A8 (London circuit 01, 02 wet, 05, 06 wet, sprint 01 wet). The front end's main menu makes these calls (the
  binary's stand-in is gone).
- In the race: Countdown 0x000BD8B0 puts every player race vehicle under ai (`AiTakesRacers` → binary
  `player_ai_control` → `set_human_control_on(false)`); GameInput 0x000A4F10 zeroes the pad record
  (`race_cars::car_inputs`); Go 0x000BD9B0 sets OSD flag 0x2000; state 3→4 starts the 45 s timer (0x000BD25C);
  InGameUpdate (Osd set, unpaused) "g_AttractMode.GameUpdate" 0x000B3170: timer, Select (action 6) / Start (13) on
  any pad (`value != 0 || changed`) or timeout → fade start(0.5) → alpha > 0.999 → set(1), `quit_race` = 1
  (`byte_6F5D08`), SetGameMode(3) → EndOfGame → FE main menu. Also gated on it: pause button 0x000BC3D0 (no pause),
  motion blur 0x000B96B9, DOF `replay` (0x00120D20 / 0x00120E80: car focus unless the wheel camera, type 3 focus
  `dword_686908`), point of interest 0x00069BE0 (off anyway), vibration 0x001ED880 (no rumble), RandomiseSetup in
  PhysicsManager 0x0019E080 (not ported, physics request), COSDInGameScreen (frontend request).
- **Review** `dword_749110` ("Enable Cameras For Review" tweak 0..2, ReplayCamera_Fixed 0x00165640; InGameInit
  0x000B6055 zeroes it and `dword_749114`): InGameUpdate 0x000B7D20 on a change: from 1 → player 0 human again, 0 →
  1 → player 0 ai (`PlayerHumanControl` message, binary applies it), then the camera cut. 2 = the jump cam
  (VehicleEntity 0x001CCAD0 → fixed cameras only, countdown `dword_749118`, InGameUpdate 0x000B8329): lives in
  physics' Cx memory, never fires in the port (physics request).
- Manager fixes (were untested): gate = attract || review || +0x1BE (was the command line flag only), else
  `updating` = 0 (0x00166630); update time = `timer.update_time()` 0x000BF690 (was race time); a shot change writes
  the record into both camera bridge buffers (no lerp across it; see "Coverage stage 3"); follow cameras skipped while `updating` ||
  review || post race camera (0x001C5CF0's gate; was "a view was returned"). The view goes through the camera
  bridge like every camera, so playback is interpolated by `timer.lerp`.
- **Shipped data never switches shots**: every CAMERA_SWITCH trigger (only LADown drag 01, London2 circuit 04 / 06
  have any) is stored with radius² −1 like the other disabled triggers and 0x0017F860 only stores the
  quaternion, so 0x00166910 never enters one, 0x00166700 never runs and the camera stays the wheel camera of car 0
  that reset (0x001658C0) leaves (+0x1BD "decide" is read only by the unreferenced DecideCameraShot).
- Smoke: `SMOKE_ATTRACT=1` (`SMOKE_ATTRACT_IDLE=<s>`, default 8) idles on the main menu and plays **two attract
  races back to back** (idle again after the first; shots `8a/8b/8c`, `8d_attract_race2`, `8e_attract_back2`). Seen
  with the front end's own attract calls: London circuit 06 (wet), back on the main menu, London circuit 05, back,
  exit 0 (log `research/core_attract_smoke.log`). `SMOKE_REVIEW=1` (review 2 s
  after Go, no pad input after) on `SMOKE_RACE_TYPE=5` (LADown drag 01): the ai drives the player to the finish
  (race time 42.5 s), `7d_finished`, `7e_post_race` (8 s after, then done); `SMOKE_REPLAY_CYCLE=1` (debug, not an
  original path) forces fixed (6 s; picked camera 1, zoomed TV shot of the player), orbit (12 s), wheel (18 s);
  DOF type follows (2 / 1 / 3). Clip `research/video/core_replay.mp4` (61.6 s, race only; shots `rec_7a..7d`).
  After the finish the race goes to mode 2 (PostRace, race still loaded); the drag's next round now leaves it
  (2 -> 1, seen: round 2 Init / Intro / Countdown right after, `research/core_review_smoke.log`). The post race
  menus are the front end's.
- Career quirk now matching the original: a quit race does not autosave (RaceCompleteScreen 0x00205830).

### Stage 4: timeline objects, flag girl, score, bridges, post race — use from other crates

- **Timeline objects** (`timeline_manager::objects::timeline_object`): each race timeline channel binds (0x170B80)
  to one object per bundle-instance name, typed by the channel type name (table 0x74A6C0: VSCamera, F3CameraShaker,
  F3ScreenEffectController, F3AIController, F3*Object/"Model" → physics, VSAudioEmitter, F3AudioController,
  F3TimelineController, F3ParticleGroupController, F3HeatHaze, F3*Trigger, F3ExplosionObject); typed events go to
  the handler registered for the event type (lists 0x687770..). All 237 shipped channels bind. Output:
  `TimelineObjectMessage` (race sections, respawns with resolved trigger points, player AI control, cut-scene
  camera / DOF, screen flash, cinematic bars, player camera shake, trailblazer firewall, explosions, heat haze,
  particle systems, physics object visibility/move/matrix/reset/particles, fader preset, city ambience, 2D sounds)
  + `AudioCommand` (music, reverb, traffic mute) + `TriggerMessage` (spawners fired by F3EventTrigger). Timelines
  playing timelines and trigger enable/move are handled in core. Physics/AI fill `TimelineWorld` (player pos,
  object positions by name crc, racers, game time, DOF). `TimelineCameraView` = the cut-scene camera (WMM mode 3,
  shaker wobble/shake from F3CameraShaker) for player 0. Quirks kept: StopTimeline 0x1723B0 is empty; sound
  emitters always play 2D (their +0x10 instance is never set); "Model" maps to physics so the skinned-model
  object (index 12) is never made; cinematic bars ignore `On`; screen flash passes 0.5, not the intensity.
- **Flag girl intro** (`timeline_sequencer::{FlagGirlIntro, FlagGirlSequencer}`, peer 0x174480..0x176A30): models
  = instance links containing FLAGGIRL/FG_ (≤5), cameras = VSCamera instances (vertical fov from HorizontalFOV at
  4:3), emitters = F3ParticleEmitter; everything placed at the player's grid slot rotated by −start direction
  (`dword_6F5C10` = radians). `FlagGirlEvent`s (model matrices/visibility, camera choice/cut, InGame cameras,
  ShowHud, AllowGo, particles, start line fire) come out as `RaceSessionMessage::FlagGirl`. State 2 rewinds and
  plays it with the players on cine cameras; ALLOWGO sets `started`; Skip seeks to the "InGame" time.
- **ScoreManager** (`score_manager`): per-car `ScoreManager::update(&ScoreInputs, in_zone, out)` + `on_collision`,
  driven by the AI race vehicle (0x18A40) in session states 4/5. **MessageManager**: `message::GameMessage`
  (Collision, TweakChanged, VehicleReset, CameraCut, PhysicsObjectReset) registered. **GameBridge** resource:
  `swap` each update ("Update Game Bridge"), writers fill `car(n)` / `camera(n)` / `timeline_model(n)`, renderers
  read `car_pose(n, lerp)` / `camera_view` / `osd_view`.
- **PostRaceSequence** (`PostRaceSequenceRes`, started by StartPostRace, events as `RaceSessionMessage::PostRace`);
  shipped Markers are empty so it never finds POST_RACE_STUNT_AREA. **TrailBlazer** (`game_mode::trail_blazer`,
  behind `TrailSpline`/`TrailCar`) is started by `TriggerMessage::Trailblazer` and updated by the ai race vehicle,
  writing `TrailBlazerMessage`s (consumers: "TrailBlazer calls", "DOF, zone FX, danger zone inputs").
- **UpdateThread**: `CUpdateThread::tick(now)` = one pass of the keep-alive loop (present every 0.1 s).
- **Requests**: AI — fill `RaceProgress`, apply `TimelineObjectMessage` race-section/respawn/AI-control and run
  ScoreManager/TrailBlazer per race car; physics — `TimelineWorld` positions, physics-object and explosion
  messages, `Mileage`; audio — `FadeToVolumePreset`, `CityAmbience`, timeline `Sound`/`SoundStop`; graphics —
  flag girl models (`FlagGirlEvent::ModelMatrix/ModelVisible`), particles, heat haze, screen flash, cinematic bars.
- Frontend options request and the physics answers (second car, pad slot -1, landing NaN, `from_setup`,
  shared world) are integrated (see Race cars / Player options above).

### On-disk file types → loader → owner

| files | read by (original) | owner / reader |
|---|---|---|
| `*.vcd` | `VsFileDirectory`, `FILE_VCDLoader` | core: `visos::file` |
| `timeline.bun` | `RaceTimelineSequencer.cpp`, `GameDataLoading.cpp` | core: `visos::chunk::Bundle` |
| `heathaze.bdl`, `SkinInst.bdl` | heat haze / skinned objects (render) | graphics, via `Bundle` |
| `*.cpt`, `index.idx` (CPTS.VCD, RACE.VCD) | `CompartmentSys.cpp`, `src\Compartment\CompartmentManager.cpp` | container core (`visos::compartment`), payload graphics |
| `.xmf` models, `.vtf`/`.tga` textures, `.vxu/.pxu` shaders, `.ced` cat's eyes, `SkySetup.cfg`, `FGlit*.bin` | render | graphics |
| `PARTDEF.VCD` `.slp/.sfp`, `particle.pld`, `PartFX.bin`, `PARTTEX.VCD` | `ParticleEmitter.cpp` | graphics |
| `asp.vcd` / `FlagGirl` `.gaf/.gas` (bone anims) | `RaceTimelineSequencer`, `TimelineSequencer` | graphics (anim), core (timeline) |
| `arena.qdt`, `gfx\arena.msh`, `arena0.mop`, `arena.bin`, `Vehicles\CFGS\*.cfg`, `Main.cfg`, `XMLBIN.VCD` `vehicle/setup/upgrades/wheels/DECALS/GLOBALS/COLOURS/Traffic.bin` | `PhysicsManager`, `MaterialProperties`, `CConfig`, `VehicleDatabase` | physics |
| `AISpline.spl`, `armco01.ar`, `barier01.br`/`noentr01.br` (`SPL0`), `traflane.tl`, `OPPONENT.BIN`, `TEAMS.BIN` | `VsAIWorld.cpp`, `OpponentDatabase`, `TeamDatabase` | ai |
| `.xvg/.xva/.sbk/.erb/.rbd`, `velocity.vol`, `envaudio.aee`, `MusLists.dfm`/`MusicDB.dfm`, `.wma`, `allSVG.vcd`, `d:\audio\voice.bin` | `VsAudio*`, `CSpoke` | audio |
| `FE*.vcd`, `.fob/.fml` fonts, `Game.OSD`, `OSDT/OSDM.vcd`, `Strings\*`, `Chars/Crowd/Dance/Cameras/lights.txt`, `costing.txt`, `.bik` | Feck2 / OSD / FontManager / StringManager / VsMovie | frontend |
| `EVENTS.VCD`, `eventdef.txt`, `TRACKS.INI`, `setup.ini`, `Setup.txt`/`SetupWET.txt` | `EventDatabase*`, `GameDataLoading` | core |
| `triggers.trg` | `TimelineTrigger.cpp` | core |
| `cameras.cam` (`FXDCAMPOS`) | `ReplayCamera_Fixed.cpp` | core |
| `Markers.mfk` | (attributed to `VibrateEffects.cpp`) | core |
| `autoexec.cli`, `Tweaks.txt`, `Input\Xboxpad\Config1.ini`, `FE\KEYMAP.INI` | Cli / tweak / input | core |
| `Timeline.tml` (`VSTL`), `RceSec*.rs`, `*.vsl` (XML) | not referenced by name in F3_D (tool sources) | — |


### Cross-crate glue: race effects, particles, physics objects (done; were graphics' and physics' requests)
- `game_mode::race_effects_bridge` (velocity_game, not the binary: it needs GameTick sets and TriggerMessage):
  "Particle Init" step → `LoadParticles { area: track_dir, sub: race_dir minus track_dir }`, InGameShutdown →
  `UnloadParticles` + `RaceEffectsShutdown`, RaceRestart → `RaceEffectsReset`, `EffectClock { paused: gm.paused,
  scale: timer.time_stretch }`, flag girl `StartLineFire(pos)` → graphics' `StartLineFire`.
- `EffectCars` (GameTick Effects set, then `race_effects_update`): race cars (kind Race) in slot order, read
  straight off the physics images: matrix = `body_matrix`, velocity, VE+20 speed, VE+4788 row 0 heading
  (0x001BB200(0)), VE+3691 smoke off, wheels = centre 0x00361400 (car+2240 ? car+1968+16i : car+1596+64i),
  radius car+1804+4i, contacts car+1088+4i, sliding 0x00364A90 (records car+676+136·(i+k), k < contacts — the
  original's index, kept), surface VE+4868+i, no marks VE+24+i; ground plane 0x001BAF60 (≥3 wheels touching →
  0x001DA190 plane through wheel centres 0,1,2: n = (p2−p1)×(p0−p1) normalised, w = −n·p0). Globals: dt = scaled
  dt, time = `game_time()`, start line = eventdef start line, professional = race selection, go =
  `RaceSession.started` (`byte_6F5BDD` is only ever set, by ALLOWGO, TimelineSequencer 0x00175270).
  `RaceEffectsInit` goes out on the first update with cars (the original runs the effect inits at the end of
  InGameInit, once the cars exist).
- `forward_collisions` (GameTick World set): `PhysicsCollision` → `SparkCollisionMessage` as the collision listener
  CarPalette 0x0010D5F0 does (for `b != -1`, car-car or wall, point.y = car a's FastCar position y + 0.3
  `dword_68676C`, then SparkRender 0x00141980; its audio 0x00052D10 / BarrierRender 0x0010B590 calls are not
  here); `PhysicsSlipstream` → `SlipstreamStrength` 1:1 (`vehicle as usize`, so -1 hits the graphics assert as the
  unsigned original would).
- `fill_motion_blur` (GameTick Effects set, after the effects): `MotionBlurInput` for view 0 = the slot-0 race car:
  speed = FastCar +32 velocity · +112 row 0 (VsAIRaceVehicle 0x00024870), `ve_44` = VE+0x44 (0x00130070),
  `wheels_down` = car +1088+4i > 0 for all four (0x00133970 → 0x00364BB0), position 0x001BA270, velocity; gate =
  not two players (`byte_6F7157`, = `race.num_players > 1`), not attract mode and `review_cameras == 0`
  (`dword_749110`). The view's render flag 0x10 is assumed set (no immediate write of view +24 found in GameManager).
  `fury` / `zone` stay 0 here (graphics feeds FuryFX 0x00128680 / ZoneRender 0x00154FC0 itself). Values are the latest update's, as the
  original reads them raw at render; the focus is position + velocity·1000, so the 24 Hz step is invisible.
  `FseCameraPlanes` = `game_cameras::current_planes(manager, 0)`. Reset to inactive on leaving InGame.
- `PhysicsObjectMessage` (GameTick World set, read in the tick that wrote it): CollisionParticles →
  `ObjectCollisionParticles`, Trail → `ParticleTrail`, Impact → `AudioCommand::Impact { MapOldId(sound) unless
  mapped, strength = impulse, car_layer_only = again }` (0x001E1F80's PlayPhysicsImpact args), PlayTimeline →
  `TriggerMessage::Timeline` (the Timeline set plays it this update), Explosion (hydrant, 0x001E1F80 case 0x17) →
  `AudioCommand::AddEnvSound { -33188540, pos, 10, 100, 50 }` (the spray object 0x001E18E0 is not simulated).
- `LoadPhysicsRaceObjects { race_dir }` is written by `race_cars::car_inputs` right after `LoadPhysicsTrack` (same
  tick; physics' Setup chain loads the track first). `TimelineWorld.objects = PhysicsObjects::by_name()` each update.
- **Physics object drawing** (`game_mode::physics_object_render`; PhysicsObjectManager 0x001E0EE0 + PhysicsObjects
  0x001E58C0): graphics draws only Visual instance lists, so the Physics (1) / Immovable (3) instances of the
  registered compartments (track `CPTS.VCD:instance.cpt`, race `RACE.VCD:instance/uinstance.cpt`) are drawn here.
  `plan_objects` matches each instance to its `PhysicsObjects` entry (name hash + list type, nearest position) and
  resolves the visual (own slot, or an empty `uinstance` slot by hash); one hidden template per model (textures
  decoded on demand from every compartment), copies share its meshes, under the `TrackRoot`. Pose = last two
  updates' matrices, position lerp / rotation slerp by `timer.lerp` (0x001E58C0 does the same for awake bodies),
  shown when `flags & 8 == 0 && flags & 2`; frozen while paused; only moved objects are rewritten. London Race01:
  1852 objects, all matched, position error 0, rotation 0.001 rad (physics matrix = the instance quaternion as-is,
  same as graphics' instances; test `london_objects_match_their_instances`). Not ported: draw-distance fade (obj
  +140, 0x0008DF40, byte_6F7157 doubles it), per-object lighting 0x001E3A60 (vertex lighting is baked unrotated).
- **Seen in game** (London Circuit Race01): `SMOKE_HIT=1 SMOKE_AREA=2` steers the player at the nearest visible
  movable object ahead; it is knocked (awake, flags 0x1F07, Impact sound 2017) and drawn tilted, sparks from the
  kerb / wall hits (`smoke_6a_object_hit`, `6b_sparks`). LADown / Tokyo have no objects (stale v3 cpts / none).
- Smoke test: `SMOKE_AREA` (1 LA, 2 London, 3 Tokyo) and `SMOKE_TRACK` pick the race; without SMOKE_TRACK it is
  LADOWN in LA, else the area's first track with a race of that type (London → LONDON). `SMOKE_PREFIX` renames the
  screenshots (default `smoke`; parallel agents' runs otherwise overwrite each other). The hit watcher logs every
  `PhysicsObjectMessage` and every spark collision with impulse·scale > 1 and screenshots the first of each.
- Timeline particle systems → `ParticleGroup`: done, see "Zone trails, danger particles, timeline particle groups".

### Pause menu (done; was the frontend's request)
- In-race HUD = the frontend's `HudPlugin`, driven by the binary's `hud.rs`: `HudCommand::Start` on entering
  InGame (`digione`, no minimap edges yet), `Stop` on leaving, `RaceHudInput` from `RaceStatus` (now with
  `rpm`/`gear`). The old black-keyed stand-in overlay is gone.
- **HUD race values = the ai race manager**, as COSDInGameScreen::Update 0x000F99E0 reads them for its player (0):
  `hud.rs::race_manager_status` (GameTick, after the Ai set) fills `RaceStatus.racing` = player vehicle +0xF4
  (`started`, set on the first start line crossing; never in a cruise, so the old cruise guard is gone),
  `position` = `*GetPlayerVehicle(p)` 0x000346C0 = RaceInfo.dPlayerPos[p] (`player_positions[0]`), `lap` =
  player vehicle +0x2FC (0x00017E00), `race_length` = GetRaceLength 0x000348C0 -> 0x0003DCA0 (players + active ai);
  `car_outputs` fills only laps / lap time / speed / rpm / gear. Before the start line the OSD shows OSD_NO_POS and
  "--/laps" (original: the grid is behind the line). LADOWN 4-car races log `position 4/4 lap 0`, the HUD shows "/4".
- Core (`game_manager`): pause button = `pause_button` (0x000BC3D0: mode 2/3 never; mode 1 only in session states
  4..6 unless network; paused -> LeaveAll + `unpause_game` 0x000BC2C0, else `pause_game` 0x000BC0C0 which writes
  `GameMenuMessage::EnterPauseMenu`). Menu choices come back as `GameFlowRequest::PauseMenu(PauseMenuChoice)`
  (bin maps `PauseMenuMsg`): Resume -> unpause; QuitToFrontend -> `quit_race` (0x000BBF90(0): LeaveAll, unpause,
  SetGameMode(3), `quit_race = true`; EndOfGame then returns to the FE); Restart -> `restart_from_menu`
  (GameTweaksMenu 0x000B12A0: in game round 0 -> `restart_pending` or drag restart 0x000BF270; round > 0 -> event
  reset + a no-op SetGameMode(1); post race -> reset + SetGameMode(1) or (3)). Both `AudioCommand::Resume`s of the
  original are sent. `m_bQuitRace` (+0x140) **is `byte_6F5D08`** (GM 0x006F5BC8 + 0x140; the old "never cleared"
  note missed the absolute-address xrefs): also set by the attract end (0x000B33C6), cleared when a race finishes
  (MonitorSessions 0x000BD58E, ported) and by FEDummyGameScreen 0x00209340; RaceCompleteScreen 0x00205830 skips the
  autosave while it is set (`career::race_complete`), so a quit race no longer autosaves.
- Not wired: `GameMenuMessage::LeaveAll` has no frontend command (see Requests to frontend, docs/frontend.md).
- The HUD menu reads the visos layer (`InputMapping::step` ignores its `raw`), so the scripted port 0 drives it.
  Keyboard (core `KEYBOARD_KEYS`, visos device 4): **Enter = START (pause), Space = A (select)** — Enter in the
  pause menu toggles pause, it does not select.
- Smoke test continues after driving: START (pause) -> 5x DOWN + A (Quit Race) -> A (Yes), then `RaceCompleteScreen` ->
  `QuickRaceStartScreen` (`smoke_4a_pause_menu`, `4b_quit_confirm`, `5b_quick_race_start_after_quit`; the
  RaceCompleteScreen passes in ~0.2 s, too short for `5a`). Passes for circuit and cruise. Back on the start
  screen its race panel is empty and the foyer dark (frontend re-init after a race, not checked).

### Car lights (done; was graphics' request)
- `race_cars::car_light_inputs` (Update, InGame): every `CarRender` root whose `request` is a live physics id
  (race + police cars; traffic is not a CarRender yet) gets `CarLightInput { braking: VE+168 > 0, reversing:
  VE+3216 < 0, damage: 0 }`, re-inserted only on change. CarRender.cpp 0x00112A50 calls 0x001B74A0 per rendered
  frame, which mutates the shift-hold state (+4056/+4060); the port reads its input VE+3216 (= FastCar gear
  car+2976, `PhysicsCarState.gear`) instead — same sign, differs only mid-shift. Light damage +2250 stays 0 until
  car damage feeds it.

### Original render rate (fact)
- F3_D renders at the **TV refresh rate**, not a fixed rate: CreateDevice params (Renderer 0x002A81C0) are 640x480,
  `FullScreen_RefreshRateInHz = 0` (= the dashboard's current mode) and `FullScreen_PresentationInterval =
  0x80000001` (ONE_OR_IMMEDIATE: one vblank, tear when late). Nothing in game code selects PAL/NTSC (no video-flag
  query; `\Movies\Pal\%s.bik` in GameManager 0x000BEBB0 is used unconditionally), so it is 60 Hz on NTSC / PAL60
  and 50 Hz on PAL50 consoles. `VsFrameTimer +0x0C` (vblank mode) is never written (the 0x00057A40 vblank-locked
  path is dead; the free-running path 0x00057690 runs). So per-rendered-frame steps (e.g. brake light 0.25/frame,
  `dword_6867A0`) were refresh-dependent in the original; 60 Hz (NTSC, the dev target) is the reference rate.
- Workspace scan for assumed render rates outside GameTick: only `visos_render::car_render::XBOX_FRAME_RATE` = 60
  (correct for NTSC); the other `1/60` hits are tests, `fecapture`/`trackview`/`track_view_fx` tool loops.

### Car damage glue (done; was physics' and graphics' requests) — `game_mode::car_damage_bridge`
- GameTick Physics chain: `car_inputs` -> `setup_car_damage` -> `repair_on_restart` -> `physics_tick` -> `car_outputs`
  -> `forward_car_damage`. Setup ("Car Damage", CarRender 0x00110220 -> 0x0006AE70 -> 0x0006D150): once per `RaceCar`
  with a `PhysicsCar` whose `CarRender` root has graphics' `CarDamageSource` -> `SetupCarDamage { vehicle: slot,
  model from the source, model_to_root: MODEL_TO_BODY, deform: race && human }`; police get it without dents
  (0x0006D150(car, this, 1)), traffic none (`CarDamageSetupDone` marks either). `RaceRestart` -> `RepairCarDamage(None)`
  (RestartRace 0x000BE0F0 -> 0x0006B090). Every `PhysicsCarDamage` -> `CarDamageRenderEvent { request: vehicle }`
  (MeshChanged copies the vertices from `CarDamageManager`); `CollisionFx { vehicle, msg }` -> `CollisionFx { vehicle,
  point: msg.point, impulse: msg.impulse }` (CarDamageFX 0x00078880; world point, impulse halved by physics).
  CarDamage makes no audio calls.
- **Barriers** (graphics' `barrier_render`): binary `barriers.rs` answers the "Barriers" step (InGameInit 0x000B6D1C,
  "Barrier Render Init") in `First` (the ai's world data step ran in the previous `Last`) with `LoadBarrierRender
  { arrows: VsAIWorld barriers 0 lines' points, no_entry: barriers 1, render: network_race ? 1 : !professional }`
  (0x00107560; always called, `nobarriers` only stops the ai's load). `race_effects_bridge` sends
  `UnloadBarrierRender` (0x00107CE0) on InGameShutdown and a `BarrierHit { car_a, material, point }` per collision
  in `forward_collisions` unless professional (`byte_6F70E9`; 0x0010B590 returns first thing), point = the lifted
  spark point (CarPalette 0x0010D5F0 writes the y into the message before both calls).
- **Physics world across races**: `despawn_race` clears `PhysicsWorldRes` directly. The `UnloadPhysicsTrack`
  message alone was never read (PhysicsTick does not run in the FE, the message expired), so the old world and its
  cars lived into the next race; the ai bound its new race cars to them and panicked (`slot` index out of bounds in
  `VsRaceManager::update_lap_crossing`) once the new track replaced the world — seen on the hangout's back-to-back
  career race. Fix not yet seen through a full second race (the career smoke exits while it loads).
- Light bits: CarDamage 0x00073700 calls CarRender 0x001157B0(**0**, light, broken) — always car 0's +2250, whoever
  broke a light (quirk kept, implemented by graphics' `CarLightDamage`); `CarLightInput.damage` stays 0.
- VertexIterator facts (for anyone walking xmf vertex groups): 0x002C4DA0 count = max−min+1 (indexed) / index
  count (plain), push-buffer draws fail; element 0/1/2/3 = vertex-desc register 0/2/9/3 offsets (0x002C4C70); the
  damage walk uses only each object's first part (`*(object+84)`), mesh matrix = that part's +0.
- Seen (London, `SMOKE_HIT=1 SMOKE_AREA=2`): 6 cars set up (4 race + 2 police, 30..69 vertex groups, locators front /
  rear), the player's kerb/wall hits dent ~20 groups of car 0 (debug log `car_damage_bridge=debug`); no light broke
  at those impulses (~2-3). Clip `research/video/core_damage.mp4`.
- Collision listener CarPalette 0x0010D5F0: its audio call `VsAudioTrackList__52D10` is an **empty function** in
  F3_D (nothing to wire); `BarrierRender__10B590` (player car a == 0, material 21, `byte_6F70E9 == 0`: stores the
  hit's 0..1 position along the current barrier's roadline at barrier +36) is graphics' `BarrierHit` (wired, above).

### Wheel spin, camera rain (done; were graphics' requests)
- **Wheel spin** (`race_cars::{record_wheel_spins, car_wheel_spins}`): GameTick keeps the last two updates' FastCar
  `WHEEL_OMEGA` (+1432, wheel 0) per physics id (`WheelSpins`); every rendered frame (Update, InGame, frozen while
  paused) each `CarRender` root with a live id gets `CarWheelSpin { omega: lerp(prev, latest, timer.lerp) }`
  (CarRender 0x00116E30 reads VE+240 raw; lerped here), re-inserted only on change.
- **Camera rain** (`race_effects_bridge::{camera_rain_steps, camera_rain_input}`): "Rain Render Init" step
  (0x000B65A5 → 0x00133AB0) → `CameraRainInit { wet: race.wet }`, `RaceRestart` (0x000BE2E2 → 0x00133CF0) →
  `CameraRainReset`, `InGameShutdown` → `CameraRainShutdown` (0x00133CD0 sits on the VsAIWorld destructor list); per
  rendered frame `CameraRainInput { follow: mode(0) == Follow (0x0005FFB0), car0: slot-0 race car position }`.
- **Rain cover** (`trigger_update::update_vehicle` → `RainCover`): TriggerUpdate 0x00181120 (called per trigger by
  0x00181920): for a non-traffic (0x00024A40 == 0) player (0x00015D30) car and a TUNNEL trigger (type 0x94473365),
  every update, inside or not → `CameraRain::cover(wet, cam pos, cam dir, trigger +0, +40 depth, row 2 of
  Renderer 0x002AC0C0(+16 quat), 0x00180F80 result)`, camera = manager's current camera (VsRender 0x0005ACD0,
  `game_cameras::current_camera_{position,dir}`). Row 2 = (2zx − 2yw, 2zy + 2xw, 1 − (2xx + 2yy)) (test
  `cover_dir_is_renderer_row2`). 0x00134EF0 rewrites the cover state per call, so the last tunnel trigger wins.
- Seen (wet London, `SMOKE_AREA=2 SMOKE_WET=1 SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=25`): "camera rain init: wet true",
  rain streaks on the grid and while driving (101 km/h), exit 0. Clip `research/video/core_rain.mp4` (56 s); the
  chase camera hides the wheels, so the blur discs are not checkable in it.
- Licence plates / decals: see "Smoke options, career keyboard, small requests".

### Puddles, lens splash, tyre spray (done; were graphics' requests) — binary `puddles.rs`, `race_effects_bridge`
- The ai world is the ai crate's (velocity_game cannot depend on it), so the puddle glue is in the binary.
  "Puddle Render Init" step → `PuddleRender::init(london_wet, SectionLines(race section 0), &mut gm.rand)` in
  `First` (after the ai's world data step), `london_wet` = `race.wet` && `track_path` lower-cased contains "london"
  (0x00131690 copies the race's track +0x78 and lower-cases it). `SectionLines` = 0x00131AC0's calls: race line
  `get_length` / `get_point_at_distance(d, 0)`, edges +8 (`left`) / +12 (`right`): `find_closest_segment(p)`,
  `get_distance_along_line(p, seg)`, `get_point_at_distance(d, seg)`.
- **The puddle test is a render-path call, not an update one**: 0x001326A0 is called only by the puddle draw
  0x00131F50 (when +13), which only RainSurface 0x00137090 calls (via 0x00162400, view flag 2), and RainSurface
  runs only while on (+19/+18 = wet or "LONDON" in the track +0x78, CommonInit 0x00136C50; +16 = draw list
  `dword_6D5330` +8 > 1 — a per-view draw list reset to 1 by GameManager 0x000B9010, not modelled), `byte_6F7157 ==
  0` and `mode(0) != 3`. So `puddle_render_update` (Update, InGame, skipped while paused like the render path) sets
  `PuddleInput.rain_surface_off = !on` and, when on, fills `race_section` (any race section), `distance` = the
  EffectCars car's ai vehicle (`find_vehicle_by_id`: players 0x00015DB0 / opponents 0x00017310) `get_dist_along`
  (+0x2A8 = +680, 0x00015CA0; None without a race section), `camera_car` = the EffectCars car of player
  `manager.unknown_60` (0x0012DC20), then `PuddleRender::update` on the latest update's `EffectCars` (sets
  `LensSpray.splash`, read by the next GameTick's `lens.update`). Quirk (0x001326A0): police render cars get a
  driver but the race-section branch never tests them; EffectCars holds race cars only anyway.
- Tyre spray: `TyreSprayInit { wet: race.wet, track: race.track_path }` is written with `RaceEffectsInit` (first
  update with EffectCars, as the original runs the effect inits once the cars exist); shutdown = the existing
  `RaceEffectsShutdown` (InGameShutdown).
- `place_cars` runs in `Update`, so the `CarRender` root transforms the slipstream trails read are written before
  PostUpdate's effect build.

### DOF, zone FX, danger zone inputs (done; were graphics' requests) — `game_mode::race_effects_bridge`
- GameTick Effects set: `danger_zone_record` → `fill_render_feed` → `dof_world`; after the Timeline set
  `dof_timeline`; per rendered frame (Update, InGame) `fill_render_inputs`, `zone_audio`. Cleared on leaving InGame.
- **ZoneFxInput** (view 0, GameManager 0x000B9010 → 0x00162D10): `active` = player car present (render flag 0x4000000
  assumed set on the race view), `paused` = `gm.paused`, `speed` = VE+20 lerped between the last two updates by
  `timer.lerp` (**`VsAIRaceVehicle__24220` is called on the VehicleEntity**: it is VE+0x14, not an ai field), `zone_on` =
  VE+89, `fury` 0 (FuryFX 0x128680 not ported), `one_off` = player 0's trailblazer running (0x23310, see "Chase
  camera"). VE+89 is set by the zone button (`CarZoneMode` = X, `boost[1]`) while the VE+104
  meter > 0 (VehicleEntity tr.rs ~5565; LADown start meter 2400). `MotionBlurInput.zone` = `ZoneFx::amount(0)` per frame
  (`fill_motion_blur` keeps it), `ZoneFx.in_zone[p]` → `PlayerCams.in_zone` (+0x29, was `unknown_29`; 0x163740),
  `ZoneFxAudio` → `AudioCommand::StartZone/StopZone` 1:1 (StopZone every frame out of the zone, as the original).
- **DepthOfFieldInput** (0x120D20 / 0x120E80): `replay` = the attract mode playing (`byte_6F5A28` +0),
  `replay_camera_type` = `CReplayCameraManager.camera` (`byte_749128` +0), `other` = player 0's
  camera mode == Cine (3) || `review_cameras == 2`; `byte_718E70` is read only (bss 0). `car_position` = player car
  `GameBridge::car_pose(id, lerp)` (the original reads the ai vehicle position raw; lerped to match the lerped camera).
- **Timeline DOF** (`TimelineObjectMessage`, applied to `ResMut<DepthOfField>` in the writing tick): reset 0x178740 =
  `DepthOfFieldSetup` (0x179AE0 force +5 = 1, **never cleared anywhere**, so after a cut scene the DOF stays drawn
  with the restored focus; 0x0BFF70 enabled = attr "DOF"; 0x179B00 blend from/to = FocusDistanceNear/Far; 0x179A30
  car focus 0) then `DepthOfFieldFocus { focus }` = 0x120BF0(dof, blend 0, off 0); envelope 0x179480 = same Focus;
  end 0x1783C0 = `DepthOfField { focus: saved +0x30, factor: saved +0x08 }` = 0x120CF0. (Before, every call was sent
  as `{focus: dof, factor: dof}`.) `TimelineWorld.dof` = (factor, focus) each update.
- **Danger zone**: `TrailEvent::RenderParams { on, prev, cur, success }` (0x0C5F90 stores a *pointer* to TB+0x60, so
  `prev`/`cur` are patched to the end-of-update values; `on`/`success` are by value at the call) and
  `TrailEvent::DangerEnabled` (0x0C6080 start → 1, 0x0C6120 outro end → 0; 0x0C6080 also zeroes `byte_719168` +29,
  the danger particles' gate of 0x1234A0). Runners write `TrailBlazerMessage { vehicle, event }`; all trailblazers
  feed the one global object (last writer wins). `DangerZoneInput` = that record + `lerp` = `timer.lerp`, `paused`.
  The ai runs the TrailBlazer now (see "TrailBlazer calls").
- World camera: `TrackCamera` is spawned at `track_render::WORLD_CAMERA_ORDER` (-40) (graphics' system also enforces
  it); no core code depends on the order; HUD cameras are 10+.
- Smoke: `SMOKE_ZONE=1` holds X 3..9 s after Go (`*_2c_zone` shot at 6.5 s); log `zone on/off (VE+104 meter …)`.
  Seen (LADown): zone on as X goes down, off on release 6 s later (meter 2400 → 2257), grey wobble rim drawn. Clip `research/video/core_zone.mp4`.

Graphics' damage FX / barrier requests and frontend's HUD / smoke requests are done (see "Car damage glue", "Pause
menu" and "Career"). Frontend facts kept:
- The front end now sends the career requests itself (`velocity_frontend::career_out`): New/Load/Save/
  DeleteProfile from the profile select, `StartCareerEvent` from the loading screen's 10th frame after the hangout
  picked an event (so `EventStarted` arrives while the loading screen is up), and writes `gm.vars.event = None`
  at CFEHangoutProgression step 8 (0x0028F790). It reads `CareerMessage` with its own cursor.

### Race audio wiring (done; were audio's requests)
- **Mode shutdown order**: `GameModeMessage::{FrontEndShutdown, InGameShutdown}` -> `AudioCommand::Shutdown` now in
  `game_manager::mode_audio_shutdown`, run in `StateTransition` before `DependentTransitions`, so it lands ahead of
  the next mode's `OnEnter` Init (SetGameMode 0x000BBD30 runs FrontEndShutdown 0x000B5CB0 before InGameInit). All
  mode messages are written in PreUpdate (GameTick / flow requests). Before, the late Shutdown (PostUpdate) killed
  the race's Init: the race was silent.
- **"Load Level Sounds"** step -> `AudioCommand::LoadLevelSounds { track_dir: race.track_path, wet: race.wet }`
  (GameDataLoading 0x000A1190), forwarded in `race_cars::race_load_steps` so it stays in step order before the
  Physics Manager step's rev bands. Order seen: Shutdown, Init, LoadLevelSounds, LoadRevBands, countdown.
- **Rev bands**: PhysicsManager 0x0019F120: competitor slot 0 when human -> `VsAudioRevBands::Load(VE+0xE8 entry
  +0x40 +0)` = `AudioCommand::LoadRevBands { name: db.vehicles[setup.vehicle].audio.model }` (`spawn_race_cars`).
- **Player car audio input** (`race_cars::player_audio_wheels`, VsAudio::UpdateVehicles 0x000524B0; VE+0xF0 = car):
  contact 0x00364BB0 (`wheel_touches`), compression 0x003649D0 (car +2268+4w), slip 0x0036CCB0 = car +160 (one
  vector for all wheels, UpdateBrakes 0x00051B00), reversing VE+0xC90 < 0 (int), throttle / front pair = newest state
  history entry (VE+0xDE8 via 0x001B6E80(0) = `ve.hist` at 208·head) +0x24 / +0x2C. Surfaces: VsVehicleAudio
  +40/+44 are written only by VsAudio::Init (-1) / FreeVehicleAudio (0) (scan of the audio range; no outside ref to
  g_AudioSystem+0x20), so the player's `surface` is `[u32::MAX; 2]` (tarmac names). Gear = 0x001B74A0's shift-held
  value (`race_cars::held_gear`, see "Driver, fade, lens spray, held gear").
- Car-car / wall hits make no sound in F3_D: PlayPhysicsImpact 0x00052E90 is called only by CarDamageFX 0x00078320
  (glass) and PhysicsObjectManager 0x001E1F80 (objects); the collision listener's 0x00052D10 is empty.
- **Glass**: graphics' `car_damage_fx::GlassFxFired { request }` -> `AudioCommand::GlassImpact { position: that
  vehicle's PhysicsCarState.position }` (`car_damage_bridge::forward_glass`, PostUpdate, InGame).
- **Career playlists**: `load_profile` sends `AudioCommand::SetLists(SavedPlaylists::from_bytes(options[0x28..]))`
  after a successful apply (0x00104F30); `save_profile` writes `VisosAudio.game.get_lists().to_bytes()` to options
  +0x28 after gather (0x00105E10; skipped when no `VisosAudio` exists, e.g. tests).
- Seen (London, `SMOKE_HIT=1 SMOKE_AREA=2`, `VELOCITY_AUDIO_TRACE`): rev idle/stressed/unstressed bands, gear
  engage/disengage, flag girl ready/go + airhorn, skid_tarmac_dry_brake_left/right, drift_light, imp_suspension_sml/
  med, imp_metal_sign + impact_car_physics (object hit), river/boat env, MusicTrack01; only `xa_env_london_pub_01`
  fails (data, see audio.md). Clip with sound `research/video/core_audio.mp4` (37.6 s, mean -23 dB).

### Driver, fade, lens spray, held gear (done; were graphics' stage 3 and audio requests) — `race_effects_bridge`
- Glass sound: see "Race audio wiring".
- **Driver** (`fill_render_inputs`, Update, InGame, skipped while paused as the render path is): `CarDriverInput { cine,
  velocity }` = ZoneRender 0x00163800 -> CarPalette 0x0010D580 (`byte_7065A9` = renderer +49 front end (0 on the race
  renderer) || (single player `byte_6F7157 == 0` && camera mode(0) == 3) || `dword_749110 == 2`) and car 0's velocity
  (VehicleEntity 0x001C6480) lerped by `timer.lerp` between the last two updates (`RenderFeed.velocity`, slot-0 race
  car, `PhysicsWorld::velocity`). Inserted on the first frame, removed on leaving InGame (absent = plain racing). The FE
  never refreshes `byte_7065A9` (FECK2ElementCamera calls CarRender 0x00119900 with front end = 1 directly).
- **Fade** (`FadeRender`): "CommonInit::Fade Render Init" step (front end and in game) -> `init()`
  (`render_load_steps`, PostUpdate, any state); `RaceRestart` -> `reset()` (RestartRace 0x000BE234); GameTick Effects
  set (after `fill_motion_blur`, not paused: InGameUpdate's paused branch skips it) -> pending `FadeStart`s then
  `update(timer.update_time() (0x000BF690), gm.paused)`. PostRaceSequence 0x000C37C0 / 0x000C38E0 = its
  `PostRaceEvent::Fade(t)` -> `start(t)` in `update_post_race_sequence` / `update_sessions`, and 0x000C37F0 reads
  `alpha()` (the never-written `RaceProgress.fade` is gone). So a finished race now fades to black in 1 s and the
  sequence reaches state 2 (no shipped POST_RACE_STUNT_AREA marker: setup warns and returns, then fades back in).
  `FadeStart(t)` (message, registered in every app) is for the ai's respawn calls (VsAIRaceVehicle 0x00022C90
  start(delay), 0x00018A40 start(-delay); global fade, called for any car — request in docs/ai.md).
  Attract mode (FontHelper 0x000B3170 start(0.5) / set(1) / alpha) is wired: see "Attract mode, review cameras".
  Not wired: 0x000BDD20 (start(1.25) when not `fading()`, alpha > `flt_62F658` loop): **no code or data
  reference** in F3_D (byte scan for its address: none), dead. FE camera (FECK2ElementCamera 0x002999B0) only *draws*
  the fade (0x00123250 when `dword_7065A4 == 0`); in the FE nothing starts or updates it and CommonInit zeroes it, so
  graphics' always-on `fade_render_system` matches.
- **Lens spray** (`LensSpray`): "Lens Spray Render" InGameInit step -> `init()` (0x0012D400; only clears when not
  yet enabled, so drops carry over between races as in the original; no shutdown exists, 0x0012DAE0 is
  unreferenced); GameTick Effects (before the fade, not paused) -> `update(LensSprayInput { camera:
  game_cameras::current_camera_position (VsRender 0x0005ACD0 = manager +0x60 player's current camera), forward_speed:
  that player's race car FastCar +32 velocity . +112 row 0 (0x0012DC20 -> VsAIDriver 0x00015DB0 -> 0x00024870; 0
  without a car), wet: race.wet })`; the input is also mirrored into `LensSprayInput`.
- **Audio gear**: now `PhysicsWorld::shift_gear` (see "TrailBlazer calls"); the old read-only `held_gear` disagreed
  with the original on ~280 updates of a race (VE+3216 request != held +4052, which the real call steps).
- Ordering note: lens / fade run after `race_effects_update` (skid smoke, start line effects inside it); the original
  order is skid smoke, lens, particles, heat haze, fade, start line — the states are independent.

### TrailBlazer calls (done; were physics' and ai's requests) — `trail_blazer::{apply_trail_blazer_calls, apply_world_mode_stretch}`
- The ai runs the TrailBlazer per player race vehicle and writes `TrailBlazerMessage`s (Ai set, and from its trigger
  pass after Physics). `apply_trail_blazer_calls` applies the trailblazer's immediate calls in message order; two
  instances share one cursor (`TrailBlazerCalls` resource): in race_cars' Physics chain right before `physics_tick`
  and in the Cameras set (after the ai's trigger pass), so each message is applied once, before the next physics frame.
  `CarFlag(b)` → `set_trail_blazer_flag` (VE+0x10), `CarReset` → `trail_blazer_reset` (0x001CC2E0), `PhysicsGlobals`
  → `set_trail_blazer_globals`, `TimeStretch(v)` → `CWorldModeManager.time_stretch` (0x000C61D0, WMM +0x10),
  `Message(crc)` → `GameManager.trail_blazer_message` (+0x108C, 0x000C6200; zeroed by InGameInit 0x000B72C4 and
  RestartRace 0x000BE413; `unk_6F5BC8` is the GameManager), `AudioStart/Stop` → `AudioCommand::StartTb/StopTb`,
  `AudioFadeIn` → `VisosAudio.game.set_tb_fade_in(0)` directly (0x000C5A80 discards its lerp and pushes 0; no
  AudioCommand exists for it). `Vibrate` has no output (no rumble consumer). `RenderParams`/`DangerEnabled` feed the
  danger zone (below); `Respawn` is the ai's.
- **WMM stretch was never applied**: InGameUpdate 0x000B7839 starts with UpdateThread 0x000C6B70 → VsFrameTimer
  0x00057E40(WMM +0x10). Now `apply_world_mode_stretch` (GameTick Input set, after `update`, InGame, also while
  paused). Every player's TrailBlazer writes the stretch every update (1.0 when idle — its ctor stretch is 1,1,1),
  so it overrides any other WMM stretch writer, as in the original. Crash cam / PhysicsManager 0x0019F920 stretch
  writes are still not ported.
- Timeline calls (`update_race_timelines`, before the TIMELINE trigger plays): `TimelineSetRestorePoint` →
  `RaceTimelines::set_restore_point` (0x00171490), `TimelineRestore` → `restore` (0x001713D0: stops the emitters of
  TimelineSkinnedModel objects, none exist → no-op), `TimelineRestoreAndClear` → `restore_and_clear` (0x00171620:
  every timeline played since the restore point (`restore_timelines`, +0x54, each PlayTimeline call recorded) is
  stopped + rewound (0x002B4E30) and its channels' objects reset (0x00171710 = `TimelineObjects::reset_object`,
  vtable +0x18); recorded physics objects (`restore_objects`, +0x68) → `ResetPhysicsObject`), `TimelineKeep` →
  `keep_restore_point` (0x00171580), `PlayTimeline(n)` → `play`. RestartRace's 0x00171100 ends with Keep too.
  The +0x68 list is filled by listeners on MessageManager channels 2 (PhysicsManager 0x0019FD80, car hits a physics
  object, msg +4 = object) and 4 (0x001E4710 object woken, msg +0 = object) — `EMessage::VehicleReset` (2) is
  misnamed; nothing in the port sends either yet (request in docs/physics.md); `record_physics_object(name)` is the
  entry. Test `restore_point_stops_timelines_played_since` (LADown Race02 timeline).
- **Audio gear**: `car_outputs` calls `PhysicsWorld::shift_gear(id)` for car 0 once per update (UpdatePlayerEngine
  0x0004C44E → 0x001B74A0, mutating) and passes the result as `CarAudioInput.gear`; `held_gear` is gone.
- Seen in game: see "Smoke options" below. Requests filed: graphics (TrailBlazerRender `Render`/`Intensity`, danger
  particles `DangerZone`), frontend (OSD item 13 from `trail_blazer_message`), physics (messages 2/4, object reset,
  wake panic), ai (stretched update dt).

### Smoke options, career keyboard, small requests (done; were frontend's, ai's and graphics' requests)
- Career smoke: A on the empty profile slot opens the on-screen keyboard; the smoke then presses UP, UP, A, DOWN,
  DOWN, A (`career` steps 10..15) → `NewProfile { "Q" }`. Seen: Created/Saved/EventStarted/ReturnedFromRace/Loaded ok.
  Front end's `career_out` also writes `gm.vars.unlock_message = 0` and calls `count_unlocked_tiers` itself.
- New smoke options (binary `smoke_test.rs`): `SMOKE_RACE=N` (race dir RACE%02d of the chosen track/type, row 4 of
  QuickRaceTextScreen), `SMOKE_DRIVE=<s>` (drive length, default 12), `SMOKE_FOLLOW_LINE=1` (steer at the player race
  vehicle's raceline point 25 m ahead, or 20 m along a running trailblazer's spline), `SMOKE_LINE_OFFSET=<m>`,
  `SMOKE_VIA="x,z;x,z"` (aim at those points in turn once within 120 m), `SMOKE_AI_DRIVE=1` (sets PlayerAIControl;
  has no effect yet, ai.md), `SMOKE_WET=1` (quick race options row 3 → rain), `SMOKE_MAX_SPEED=<m/s>` (brake instead
  of throttle above it; stuck = < 1.5 m in 2 s → reverse 2 s, logs the cars within 12 m). `SMOKE_BIG_PARTICLES` is
  retired (see "Race02 street route"). Recorder: `SMOKE_RECORD_FROM=<s>`
  skips the first seconds of recordable time. (`VELOCITY_NO_RACE_OBJECTS` is gone: the physics wake panic is
  fixed.) The smoke logs trailblazer events, time stretch changes and the OSD message and shoots
  `2d_trailblazer` (1.5 s in) / `2e_trailblazer_end`.
- **Seen (LADown Circuit Race02)**: `SMOKE_RACE=2 VELOCITY_NO_RACE_OBJECTS=1 "SMOKE_VIA=340.5,-53.3;349.6,-71.1"
  SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=128`: the trigger at (349.6, -71.1) (box 3x4.8x12.9, rotated 27° about y; the
  second one at the same spot is disabled, radius² −1) starts the trailblazer (spline 0x3E7ED980, 8 m, success
  timeline 0x3B48C487, respawn 0xE96F3018); stretch 0.1 → 1 within 0.3 s real (ai dt is unstretched, ai.md), danger
  tint + warp drawn; the scripted car leaves the 8 m corridor → fail: globals restored, CarReset, respawn, OSD
  0x643ce31, stretch 0.01 one update, outro DangerEnabled(false) + StopTb, 5 s later RestoreAndClear + message 0.
  Clip `research/video/core_trailblazer.mp4` (SMOKE_RECORD_FROM=95). Plain smoke and career smoke still exit 0.
- `career_teams` (`TeamView`, `CareerTeams` find/count/get invites, `TeamRequest`), registered in the game plugin;
  test `invite_filters_keep_list_order`. ScoreManager.cpp 888 assert moved to the race switch's default arm.
- Graphics: `car_spec` sets `spec.decals = CarDecals::from_setup`; `LicencePlate.front_end` set on entering FrontEnd
  (true) / InGame (false) (CommonInit → 0x0012DC60); style/text keep the GamePlayer defaults (online sign-in not ported).

### Restore point object messages, object resets (done; were physics' requests) — `race_timeline_sequencer`
- **Trailblazer timeline calls moved before the physics frame**: the trailblazer calls `TimelineBridge__98D10`
  (SetRestorePoint / Restore / RestoreAndClear / Keep / PlayTimeline) inside the ai update, i.e. before that
  update's physics. `trail_blazer_timeline_calls` (shared `MessageCursor`, resource `TrailBlazerTimelineCalls`)
  runs in race_cars' Physics chain after `apply_trail_blazer_calls` (Ai-set messages) and again in the Timeline
  set before `update_race_timelines` (the ai's trigger pass after Physics); its PlayTimeline events wait in the
  resource and are dispatched first by `update_race_timelines` (which no longer reads `TrailBlazerMessage`).
- Channels 2 / 4 (car hit object PhysicsManager 0x0019FD80, object woken 0x001E4710 → listener 0x00172570):
  `record_object_notices` right after `physics_tick` feeds every `PhysicsObjectNoticeMessage` into
  `record_physics_object`, so a restore point set by this update's ai sees this physics frame's notices, as the
  original.
- Resets: `PhysicsObjectResets` collects `TimelineObjectMessage::ResetPhysicsObject { object: Some(name) }` (shared
  cursor; `collect_object_resets` after the Timeline set so they cannot expire at high frame rates) and
  `send_object_resets` writes `velocity_physics::plugin::ResetPhysicsObject(name)` right before `physics_tick`:
  RestoreAndClear's resets reach the same physics frame (original: inside the ai update); timeline-handler resets
  (TimelinePhysics 0x0017DAA0, the physics timeline object's reset) the next one. `None` (VehicleManager 0x001DE400,
  reset every vehicle) only warns: no physics port (request in physics.md).
- Wake: 0x001E4710's only non-physics caller is F3EventInstanceMove (TimelinePhysics 0x0017D540: lookup, matrix
  0x0017DCF0, wake when +100, velocities 0x001E5270 with the +104 swap); it needs a physics message carrying the
  move (request in physics.md); `PhysicsWorld::wake_object` alone would wake before the matrix is set.
- `VELOCITY_NO_RACE_OBJECTS` is removed; race objects always load.
- Seen: LADown Race02 `SMOKE_RACE=2 SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=170` with 210 race objects and 14 traffic
  CarRenders, exit 0 (log `research/core_race02_clean.log`). **"Player still from 174 to 176 s" is the smoke
  driver**: the logged `dt` is real time since Go and keeps counting while paused; throttle stops at `dt > drive`
  (170), the car coasts to ~172, then START opens the pause menu (`4a_pause_menu`) and the paused game freezes the
  car through the quit dialog. Same pattern in the wet London clip (drive 25 → frozen from 26 s).
- Seen (wet London, `SMOKE_AREA=2 SMOKE_WET=1 SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=25`): "puddle render init: london wet
  true, 65 puddles", tyre spray mist behind the cars, puddles faint (black, low texture alpha, as graphics notes).
  Clip `research/video/core_wet.mp4` (51 s, recorded before graphics fixed `puddle_test`'s puddle index, so no
  splash in it). Graphics now logs `lens splash: car 0 wheel w in puddle k ...` (puddles 0..4 in 25 s of the same
  smoke; graphics.md "Puddles / lens splash trigger").

### Instance move, reset all objects (done; were physics' requests) — `race_timeline_sequencer::PhysicsObjectResets`
- The shared cursor now also collects `TimelineObjectMessage::InstanceMove` → `velocity_physics::plugin::
  PhysicsInstanceMove` (fields 1:1, z already negated) and `ResetPhysicsObject { object: None }` (0x001DE400 on the
  physics object manager, PhysicsManager+24: every placed object, **not** a vehicle reset) → `ResetAllPhysicsObjects`;
  `send_object_resets` writes resets, then reset-alls, then moves right before `physics_tick` (physics orders its own
  application). Messages registered by race_cars when the physics plugin is absent (tests). Race02 smoke (60 s
  follow-line) exits 0. Seen later: Race02's trailblazer pass sends two `InstanceMove`s (objects 0xF8E7A95E →
  (489.6, −10.2, −21.7), 0x00ACA5B4 → (489.5, −10.1, −53.2), zero velocities) from the race timeline.

### Zone trails, danger particles, timeline particle groups (done; were graphics' requests)
- **Zone trail init** (binary `zone_trails.rs`; the lines are the ai world's): InGameInit "Trailblazer Effects" step
  (GameManager 0x000B70B3 → 0x0014DF70) sets a pending flag; the first GameTick whose `PhysicsWorldRes` holds a world
  (Effects set, before `trail_blazer_render_update`) writes `TrailBlazerRenderCall::Init(lines)`, lines = 0x0014E480:
  every lane (`get_num_lanes`) of every trailblazer section (`get_num_trail_blazer_sections`) in order, `ZoneTrailLine
  { section, lane, id, points xyz, length }`. LADown Race02: 2 lines (0xA777883A 314 points, 0x3E7ED980 108).
- `race_effects_bridge`: `trail_render_calls` → `trail_blazer_render_update` chained in the Effects set after
  `race_effects_update` (not gated by pause; 0x0014E530 checks `EffectClock.paused` itself): `TrailEvent::Render
  {spline}` → `Render { vehicle, spline }`, `Intensity` → `Intensity`, `DangerZone(Some(l))` → `DangerParticleCall::
  Level(l)`, `None` → `Stop`, `DangerEnabled(true)` (0x000C6080) → `Enable(false)` (+29 = 0). Reset (0x000BE2BA) /
  shutdown are direct calls on `ResMut<TrailBlazerRender>` in `mode_messages` (a message written in Update would
  expire before the next GameTick at high frame rates). Shutdown at InGameShutdown is a port choice: the original
  frees the trails only in the next Init (0x0014E280) but never draws them outside the race.
- +29 = 1 (`danger_particle_enables`, GameTick after the Timeline set): `TriggerMessage::BigParticles` (new; the
  TIMELINE trigger arm of 0x00180310 compares its target with `VsStringAnsi::Hash("TriggerBigParticles")` =
  case-sensitive crc32 0xCACBD0F4 after playing the timeline; test `big_particles_timeline_trigger`) and
  `TimelineObjectMessage::TrailBlazerFirewall(on)` (0x0017A750) → `Enable(on)`.
- `TimelineObjectMessage::ParticleSystem { system, active }` (TimelineParticleGroup 0x0017C390 / unbind 0x0017C260)
  → `ParticleGroup { group: system, start: active }` (StartGroup 0x000FF900 / StopGroup 0x000FFA70), after Timeline.
- Seen (LADown Circuit Race02, `SMOKE_RACE=2 "SMOKE_VIA=340.5,-53.3;349.6,-71.1" SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=215
  SMOKE_MAX_SPEED=22 SMOKE_BIG_PARTICLES=1 (retired) SMOKE_RECORD_FROM=200`): trail states 0 → 1 → 2 → 3 (success fade) → 0,
  the capped car completes the trailblazer (success timeline 0x3B48C487; uncapped it fails in the x≈550 corner),
  danger particles enabled → active at level 0.24, stopped at the end; clip `research/video/core_trails.mp4` (39 s;
  ribbons faint pink on the road at the start, chase debris/fire bright round the camera). Log lines `zone trails:` /
  `danger particles:` from `zone_trails::log_changes`.
- **Race02 street route** (geometry from the ignored probe `velocity_game/tests/race02_probe.rs`): the E-W deck
  (y −9.6, x 340..575) is off the circuit (the circuit runs x 320 southward, z −310 eastward, x 548 northward on the
  deck). The street (−17.9) is reached only by a **down-ramp at x 465..510, z −50..−55** (−9.6 → −15.7, under the deck
  from x 515); a raised divider (−7.7) at z −60, x 470..510 separates it from the trailblazer lane (z −65..−72, whose
  spline ends on a kicker at (515.4, −7.3, −67.1) and lands back on the deck); an overpass (−4.6) covers z −45..−60 at
  x 395..420; the deck ends in a wall at x 577.5 (the old "stopped 3 m short of the hole" = that wall). Route that
  works: `SMOKE_RACE=2 SMOKE_FOLLOW_LINE=1 "SMOKE_VIA=340,-54;372,-65;420,-66;455,-53;475,-52.5;500,-52.5;547,-52.5;
  580,-52.5" SMOKE_DRIVE=212 SMOKE_MAX_SPEED=22` (z −54 at x 340 misses the trailblazer start trigger, whose
  corridor fail would respawn the car): the street's ASP_Particles timelines (triggers 18..40) and the firewall
  (`TrailBlazerFirewall(true)` → danger particles enabled, not active: no trailblazer gives a level) fire. Clip
  `research/video/core_danger.mp4` (`SMOKE_RECORD_FROM=200`, 38 s).
- **TriggerBigParticles never fires, in the original too**: trigger 17 (name 0xC63160BB, target 0xCACBD0F4, at
  (547.4, −17.8, −52.5)) is stored with radius² −1 (disabled) like 16; only the timeline trigger objects
  (0x0017F430 / 0x0017F4E0 via 0x0017FF50, vtable 0x641B48) enable triggers, and no Race02 timeline channel or item
  names 0xC63160BB. The car drove through its centre with no `TriggerBigParticles:` log. `SMOKE_BIG_PARTICLES` is gone.
- **Physics panic on the E-W deck** (3 of ~6 runs, x 340..370, z −52..−60, ~1 s after reaching (340, −54)):
  `cx.rs:709 load from Ptr { sp: Null }` in `Cx::i` ← `physics_manager::tr::f_1a3a30_body` ← `f_1b32e0` ←
  `PhysicsWorld::step_inner` (logs `research/core_danger_run2.log`, `research/core_danger_panic.log`). Reported in
  physics.md.

### Teams joined (done; was the frontend's request)
- `dword_6F71D4` (+0x264, idb `g_TeamsJoinedCount`) = `CFEGameVariables::teams_joined()` / `teams_joined_mut()`
  (storage still `unknown_264[0]`, so the frontend's `unknown_264[0]` write keeps compiling; once it uses
  `*gm.vars.teams_joined_mut() += 1` (wrapping) core can split the array into a real field). Only xrefs:
  SetInitialVariables 0x000C1930 (= 0) and join_team 0x00290BC0's first join; read only by the save.
- `set_initial_variables` now also zeroes, in the original order after `dword_6F71CC`: `team` (`dword_6F71D0`),
  teams joined, `unknown_264[1]` (`dword_6F71D8`) and `[2]` (`dword_6F71DC`); before, a new profile kept the old team.

### Requests from graphics (physics object lighting, CompartmentManager 0x00087D00) — done
- Wired in `physics_object_render::plan_objects`: each template's `SpawnOpts.light_set` = `instance_light_set(version
  of the model's compartment, TrackInstanceLights.sky_ambient)` (the model's compartment stands for the instance's).
- `physics_object_render`: the physics/immovable instances (types 1/3) get the same light set as visual instances
  (record +120). Set `o.light_set = Some(track_lights.lights.instance_light_set(cpt.header.version,
  instance_lights.sky_ambient))` on the `SpawnOpts` (`Res<visos_render::track_render::TrackInstanceLights>`);
  flag-1 stages then draw ambient-only (LA sky: 0.25 grey) like the visual props (graphics.md "Light sets").

### Requests from graphics (flag girl spec index, Characters.cpp 0x0007A460) — done
- `flag_girl_models::spec_char_index` (Update, after `apply_events`) inserts `SpecCharIndex(k)` on each flag girl's
  `SkinnedModelRender` root, k = request − `0x0F1A6000` = model index.
- Insert `visos_render::specular_chars::SpecCharIndex(k)` on each flag girl's skinned root (k = the flag girl's index
  in GameManager 0x000B9010's loop, = 0x0007A460 a11): picks its SpecularChars record / diffuse light set. All five
  default records are equal today (no FGlit file ships), so this only matters with a settings file.

### Requests from graphics (car headlights, TrackFX 0x0014B030) — done (see "Chase camera", HeadlightView)
- Each rendered frame set `visos_render::track_fx::HeadlightView { camera_mode, inside }`: `camera_mode` = view 0's
  camera mode (VsCameraManager 0x0005FFB0; 3 shows the beams of culled cars), `inside` = the `CarRender.request` of
  the car the camera sits in — CarRender 0x001127F0: race car (type 0) with a vehicle, view's mode 1 → VehicleEntity
  0x001C5780, mode 2 → 0x001C57C0 gives a camera, `!VsVehicleFollowCamera 0x00069880(cam)` and cam +580 == view.
  Default (mode 0, none) = every chase view. Car light damage and brake level are already read from
  `CarLightInput` / `CarLights`; the zone mask from `ZoneFx`. (`HeadlightView` is written now.)
- Done: `race_cars::car_spec` ends with `spec.with_setup_underbody(setup, db)` (race / police cars; traffic uses
  `CarSpec::traffic`, no setup). Request text: `spec = spec.with_setup_underbody(setup, &db)` (CarRender 0x110220 / 0x115920: shadow
  width/length from the vehicle data, neon from the setup's colour slot 1). Without it every car gets the 2.9 × 4.8
  default shadow and no neon. The shadow / neon quads read wheel contacts and ground points from `PhysicsWorldRes`
  by `CarRender.request` = vehicle id (already so).

### Requests from graphics (car light flares, FlareRender 0x00123930 / 0x00123970) — done
- Binary `ai_host.rs`: `CoreHost::traffic_hazard` (VsAITrafficVehicle 0x0002B4B0) / `police_light` (SetSiren
  0x000179B0) queue `CarFlareMessage::HeadlightFlash` / `TopLight` (request = the ai's vehicle id = physics id =
  `CarRender.request`; intensity unused, as 0x00123970 stores only on + colour) in `FlareCalls`; `flare_calls`
  (Update) writes them in call order (log "flare call: ..."). `traffic_flare_lights` (Update, InGame) = 0x00125190's
  per-frame test: each bound traffic vehicle i -> `TrafficLights { index: is_triggered_type() then i }`, written only
  when it differs from the car's `CarFlareState.traffic`. Police are not in the traffic loop (VE+5080 stays -1).
  Siren on = `start_chase` (Alerted / Chase / Pursuit), off = `to_traffic`. Smoke `SMOKE_POLICE=1` steers at the
  nearest active police car (within 120 m when SMOKE_FOLLOW_LINE), `SMOKE_LOOK_POLICE=1` holds WHITE (look back) while
  an active police car is within 80 m behind (shot `9b_police_behind`).
- Seen: traffic hazards flash on LADown (HeadlightFlash on/off for traffic ids 13/14). **London2 Circuit Race03**
  (`SMOKE_DIRECT=1 SMOKE_AREA=2 SMOKE_TRACK=LONDON2 SMOKE_RACE_TYPE=3 SMOKE_RACE=3 SMOKE_FOLLOW_LINE=1`, ~39 s after
  Go) alerts police car 18 (proximity), TopLight on (blue 0x800070FF / red 0x80FF0000) until it gives up ~50 s later;
  looking back shows the London police car with its blue top light flare. Clip `research/video/core_police.mp4`
  (+ `SMOKE_LOOK_POLICE=1 SMOKE_RECORD_FROM=60`, log `research/video/core_police.log`). LADown runs (with and without
  SMOKE_POLICE) never got a chase: police join the traffic ~200 m away every 30 s and leave again.
- (original request) The ai already calls `AiHost::traffic_hazard(vehicle_id, on)` and `AiHost::police_light(vehicle_id, light, on, _,
  a, r, g, b)`; in `CoreHost` map the vehicle id to its `CarRender.request` and write
  `visos_render::flare_render::CarFlareMessage::HeadlightFlash { request, on }` /
  `CarFlareMessage::TopLight { request, light, on, a, r, g, b }`. For each traffic car write
  `CarFlareMessage::TrafficLights { request, index: Some(traffic manager index) }` while
  `VsAITrafficVehicle::is_triggered_type()` (0x0002B5D0) holds (vehicle +5080 ≥ 0), `None` otherwise.

### Requests from graphics (dynamic envmap, DangerZoneRender 0x001214A0) — done
- Wired: `race_effects_bridge::dynamic_envmap_steps` (PostUpdate): step `FeCarRender` ("CarRender", FrontEndInit
  0x000B5829 push 1) / `CarRender` ("Car Render", InGameInit 0x000B6E52 push 0) → ZoneRender 0x00163770 → 0x0010D7E0
  = `DynamicEnvmapInit { front_end, two_players: race.num_players > 1 }`; 0x00163770 puts 0x001637B0 on the VsAIWorld
  destructor list → `DynamicEnvmapShutdown` on Front End / InGame shutdown. `DynamicEnvmapInput` left default (race
  car 0). Seen: "dynamic envmap: init (enabled true)" in the race.
- Original request: at GameManager::CommonInit (where CarPalette 0x0010D7E0 inits the car renderer) write
  `visos_render::car_shader_manager_pc_xbox::DynamicEnvmapInit { front_end, two_players }` (`two_players` =
  `byte_6F7157`, `race.num_players > 1`), and `DynamicEnvmapShutdown` at shutdown. Optionally set
  `DynamicEnvmapInput { player: Some(request) }` (default = lowest `CarRender.request`). Without the init the cars
  keep the static `ENVMAP1.TGA` reflection. While it runs, in-game car meshes move to render layer 20
  (`CAR_LAYER`) and graphics adds that layer to the `TrackCamera`; any other camera that must see in-game cars needs it.

### Requests from physics (chase camera VE fields: up offset, player camera shake) — done
- Wired: `follow_inputs` reads `PhysicsWorld::camera_up_offset(id)`; `game_cameras::player_camera_shake` (GameTick
  after the Timeline set, before `write_camera_bridge`) → `set_camera_shake(0, intensity, length,
  RaceSession.race_time)`, the same clock the follow camera now decays against (see "Chase camera").
- **VE+4392 is live now**: every physics frame ends with SyncModels' physics part (VehicleEntity 0x001C1FF0 ->
  translated 0x001C2FF0, per active car, right after ResolveCollisions; also on `activate`), which writes
  +4392 = −|sin(lean)|·GC `sideways_rotation_shift`. `follow_inputs` reads VE memory, so nothing to change; accessor
  `PhysicsWorld::camera_up_offset(id)`. With the shipped Handling.cfg (`player_lean_reverse_cheat 0`,
  `AI_lean_reverse_cheat 1`) it is 0 for players and up to ~0.15 for AI cars, as in the original.
- **+5008..+5020 is the timeline player camera shake, not a collision shake**: its only writer is VehicleEntity
  0x001D1610, called only by TimelineCameraShaker 0x0017A5F0 (`F3EventPlayerCameraShake` begin) on vehicle-array
  entry 0 with (Intensity, event length). Please consume `TimelineObjectMessage::PlayerCameraShake { intensity,
  length }` (nothing reads it yet) with `PhysicsWorld::set_camera_shake(id_of_vehicle_array_0, intensity, length,
  race_time)`: +5008 = +5012 = intensity, +5016 = race time (`VsAIDriver__15D50` = race manager +520), +5020 =
  +5016 + length. Pass the same clock as `FollowExtras.game_time` so 0x00066940's decay lines up. Read back with
  `camera_shake(id)` = [+5008, +5012, +5016, +5020]. +5008 is also zeroed by 0x001CABF0 (setup) and 0x001B9940.
- Owners confirmed: +4436/+4440/+4444 (CVI rate, height, frame) are written only by 0x001C5CF0 (yours) and zeroed by
  PhysicsCarInput 0x001B9940 (physics' vehicle reset) — no other physics writer, keep writing them back.
- Also available: `PhysicsWorld::graphics_pose(id)` = 0x001C2FF0's graphics position and quaternion (lean cheats,
  COM-to-pivot offset) as SyncModels hands it to the car model.

### Post race mode, quit the event, attract stand-in gone (done; were frontend's requests)
- **`InRace` computed state** (`velocity_game::InRace` = `EGameMode::InGame | PostRace`): the original keeps the race
  loaded through mode 2 and frees it only in EndOfGame 0x000BE540, so every race init / teardown hook is on
  `OnEnter(InRace)` / `OnExit(InRace)` (in_game_init, licence plate, career `from_race`, review camera reset, hud
  start/stop, despawn_race, flag girls, physics object render, motion blur, camera rain, headlight view, stream
  views, zone trails). Before, entering PostRace ran the InGame teardown (cars, track, physics gone under the post
  race menu). **2 -> 1** (post race Restart, next drag round via `drag_continue`) now returns `Some(InGame)`: the state
  leaves PostRace, InRace stays (no InGameInit), `restart_pending` → RaceRestart on the next InGame update, as
  RestartRace on the loaded race. Test `set_mode_rules`.
- **`gm.quit_event` = `byte_6F5D09`** (GM +0x141): written only by GameTweaksMenu 0x000B17A0 (case 2: event flags &
  4 failed → 1, & 2 completed → 0, then LeaveAll + SetGameMode(3); case 3 → 0 + SetGameMode(3); dump
  `research/core_post_race_menu_b17a0.c`). EndOfGame: `+0x140 || byte_6F5D09` → back to the FE without the race
  records (0x000BE430); else records, back when event +0x15 & 0x10. The binary's `hud::menus_to_game` sets it from
  `PauseMenuRequest::EndOfGame { quit_event }` (the front end's `hud::end_of_game` makes the SetGameMode(3) in the
  same frame; EndOfGame reads it on the next update).
- `velocity/src/attract.rs` keeps only `player_ai_control`; the main menu's attract calls are the front end's.

### Timeline audio, ai / traffic / police audio input (done; were audio's requests)
- `race_effects_bridge::timeline_audio` (GameTick after the Timeline set, same tick): `TimelineObjectMessage::Sound`
  → `AudioCommand::TimelineSound { object, event }` (always 2D), `SoundStop` → `TimelineSoundStop`,
  `FadeToVolumePreset` → `FadeToVolumePreset`, `CityAmbience(on)` → `CityAmbience(on)`.
- Binary `car_audio.rs` (GameTick after Physics, before Cameras) fills `GameAudioInput.{ai, traffic, police}` from
  `AiState` + each car's `PhysicsCarState` (`race_cars::car_audio_input`, the player's base too). Dumps
  `research/core_audio_update_dec.txt`. **UpdateAiCars 0x00053290 recounts every update**: audio +12 = race manager
  +60 = player vehicles (0x00015C20), +16 = +56 = ai race vehicles (0x000173B0, idb `VsAIPlayerTFTManager__*` is
  misnamed: 0x00017310 = `m_pAIRaceVehicles[i]`, 984 B), +24 = traffic manager count; i in players..total →
  `ai[i - players]` = **the opponents** (so the race ai *does* get an engine loop: engine event +356 = hash of the
  upper-cased base name of vehicle entry +0x40 +4, set by PhysicsManager 0x0019E080 after `LoadAiEngine` of that
  path; port: `RaceCars.ai_engines` → `vs_audio_vehics::ai_engine_event`), `present` = true, `ai_rpm` = startline
  revs `active` (+0x104) ? `get_revs()` (0x00023B90). Traffic: every manager slot (0x00024AE0), VE = vehicle +0x24,
  `active` = VE+28, siren = 0x0002B5D0 `is_triggered_type`, emergency = VE+4073 (physics' `is_bus` byte), ai_speed =
  +0x90, horn = +0x140 (ai's `hazard`). Police: 20-byte units (0x00024C00): mode +8 != 0 → traffic part's VE, else the
  race part's; siren +12. Requests filed in docs/audio.md (LoadAiEngine command, per-update counts).
- Seen (LADown circuit, `SMOKE_DRIVE=30 SMOKE_FOLLOW_LINE=1`, `VELOCITY_AUDIO_TRACE` → `research/audio/
  core_traffic_trace.txt`): `passby_dry_light` on traffic all along the drive, `horn_car_1`; the three opponents'
  engine events (`v300zai0`, `vceliai0`, `vsilvai0`) failed then (no AIEngine load yet; fixed, see "Ai engines").
  Clip `research/video/core_traffic_audio.mp4` (55 s, mean −19.4 dB).

- **Ai engines (done; was audio's request)**: `spawn_race_cars` writes `AudioCommand::LoadAiEngine { path }` (vehicle
  entry +0x40 +4) for every opponent right before `cars.ai_engines.push` (PhysicsManager 0x0019E080 at 0x0019F2B0,
  Physics Manager step, plan order, after the player's LoadRevBands). `car_audio.rs` pushes `present: false` for an ai
  without a physics car, so ai i keeps audio slot players + i. Seen (LADown circuit, `SMOKE_DRIVE=20
  SMOKE_FOLLOW_LINE=1`, trace `research/audio/core_ai_engines_trace.txt`): "ai engine" for vceliai0 / v300zai0 /
  vsilvai0, all three loops active the whole race, no failed events. Clip `research/video/audio_ai_engines.mp4`.

**Requests from frontend — done** (answer in docs/frontend.md "Requests from core (GamePlayer names, plate text)"):
- `quit_event` copy removed from `hud.rs::menus_to_game`. GamePlayer accessors: `tips_watched()` (+0x268),
  `plate_style()` (+0x26C), `plate_text()` / `set_plate_text()` (+0x270 `unk_6F71E0` = inline **wide** string,
  MEM_Alloc 0x002C1850 widens each byte; the last 16 bytes of the saved 84-byte vars block, so the save carries it;
  8 chars put the nul on `dword_6F71F0`, outside the block). `set_initial_variables` writes "OCTANE" (0x000C1930).
  `career_requests` pulls graphics' `LicencePlate` into GamePlayer every frame and pushes GamePlayer into it after
  NewProfile / a good LoadProfile. Storage is still `unknown_264` until the front end uses the accessors.
- `hud.rs`: `HudCommand::Start` moved from `OnEnter(InRace)` to `First` on the "Generating OSD Map" step ("Load 3d
  Interface" in a cruise), with the map edges of InGameInit 0x000B6874..0x000B6A1E (no map in a cruise / without a
  race section; inside = the shorter of race section +0x08 / +0x0C (+0x0C when `len(+0x0C) <= len(+0x08)`), closed
  = +0x08 IsCircular). `hud_slide` (GameTick after Timeline) maps `RaceSessionMessage::Hud(false/true)` to
  `HudSlideReset` / `HudSlideIn`. `name_tags` (Update, InGame) fills `RaceHudInput.name_tags`: player = player
  vehicle 0's bridge pose, one entry per `race.ai` with a valid pose (slot = `id` +0x78, name = driver `get_name`),
  colour = the ai's `VsAIDriver::name_colour` (0x00015F80). Seen: LADown career race, map drawn, speedo in place.
- Smoke: PlateCustomizationScreen -> A (shot `0g_plate`); keys held in the race until a frame in which
  `total_updates` advanced since the press (FE keeps 0.12 s / 3 frames). Career smoke: Created -> plate -> race ->
  Loaded -> second race, exit 0.

Original requests (kept for reference):
- `gm.quit_event` is now set by the front end's `hud::end_of_game` right before its `set_game_mode(3)` (as
  0x000B17A0 does). Please drop the copy in the binary's `hud.rs::menus_to_game` (the `EndOfGame` arm can just
  `continue`). `career_out` now uses `gm.vars.teams_joined_mut()`, so `unknown_264[0]` can become `teams_joined`.
- Name tags (COSDInGameScreen 0x000F9340, shown with the Player names option): please fill
  `RaceHudInput.name_tags` every frame while racing: `player` = driver 0's position and `vehicles` = one
  `NameTagVehicle { slot, pos, name, colour }` per AI race vehicle (`VsRaceManager` +36 array, i.e. the opponents:
  `slot` = vehicle +120, the race slot; `pos` = the car's render position (the same space as the race camera's
  `GlobalTransform`, interpolated like the car body); `name` = the driver profile's name (vehicle +564 profile, 0x000153B0
  = profile +4, empty when 0x00015D30 says so); `colour` = the profile colour (0x00015F80: lerp of profile +68 by
  +64 * 0.01) as a D3DCOLOR). `view_proj` is filled by the front end from the `TrackCamera`. Leave `vehicles` empty
  outside a race.
- **Hud slide (OSD3dInterface +2929 / +2932)**: please send `HudCommand::HudSlideReset` where GameManager 0x000BC5F0
  calls 0x000EAF00 (two calls: 0x000BC68A, 0x000BC79A) and `HudCommand::HudSlideIn` where TimelineSequencer 0x00175270
  calls 0x000EAF60 (0x00175325, 0x00175561). Until the first one arrives the HUD keeps the speedo / map in place (a
  stand-in: the ctor state "out" would slide them off for good). GameManager 0x000BD8B0 reads 0x000EAFA0 (slide == 0).
- **PlateCustomizationScreen (career smoke)**: the screen is ported, so a new profile now goes ProfileSelectScreen ->
  PlateCustomizationScreen (0x8559D7AA) -> back (then saves), as the original. `SMOKE_CAREER=1` stalls there (timed out
  at 28 s on that screen): please add `else if cur == crc("PlateCustomizationScreen") { press(&mut s, key::A); }` to
  the smoke's screen chain (A picks the style, the keyboard opens with the cursor on DONE, A finishes).
- **Licence plate globals**: `vars.unknown_264[2]` = `dword_6F71DC` = plate style (front end writes it in
  `lib.rs::plate_sync`); the text `unk_6F71E0` (GamePlayer +0x270, a VsString) is not kept by core — only in graphics'
  `LicencePlate`. Please name the style field, keep the text in GamePlayer / the save, and set `LicencePlate` from them
  on profile load (the front end mirrors `LicencePlate` back into the screens). `game_manager.rs:676`'s comment
  ("only FEOnlineSignInScreen ... online, not ported") is outdated: that range is PlateCustomizationScreen.
- **Minimap edges**: `velocity/src/hud.rs` sends `HudCommand::Start` with empty edges, so the 3D hud's map screen
  (`hud_map_screen`) is never drawn; please pass the track's inside / outside x/z polylines.
- **Tips seen**: `GameVars.unknown_264[1]` is `dword_6F71D8`, the TipsScreen's "movie seen" bits (bit `tip - 1`; set by
  FETipsScreen 0x002400D0, read by 0x00240310). The front end reads / writes it (`CareerRequest::SetTipsWatched`);
  please give it a name (`tips_watched`) when convenient.
- **Sweep "still paused after the quit confirm" (smoke script, not the menu)**: the smoke's 3-frame minimum hold
  (`smoke_test.rs:440`, `s.t < s.hold_until || s.held_frames < 3`) outlasts CGameMenu::Update's 0.2 s auto repeat
  (0x000AA3E0, `+588 = 0.2` = 0x3E4CCCCD, held DOWN repeats) whenever frames are >= ~0.07 s, so a DOWN moves twice:
  the 5 DOWNs land on ONLINE_OPTIONS (A does nothing, stuck paused: RACE04_wet_1, RACE02_dry_2, RACE07_wet_2,
  RACE08_dry_1 logged no `pause menu:` line) or wrap to RESUME (RACE04_wet_2, SPRINT05_wet_1 logged `pause menu:
  Resume`, then step 9 waits for a pause that never comes). The menu is now 1:1 per game update (InGameUpdate
  0x000B77F0 calls `g_GameMenuManager.Update` 0x000A9DF0 with the update step 0x000BF650, input sampled once per
  update), which makes a time-based hold matter even more (max 2 updates per frame). Please hold a smoke key for
  the first frame in which `GameClock.timer.total_updates` has advanced since the press (>= 1 game update seen,
  at most 2), not for a time / frame count — or check `HudState`'s menu selection before pressing A. Debug trace:
  `RUST_LOG=info,velocity_frontend::game_menu_manager=debug` logs every move / select.
  **Done**: in `EGameMode::InGame` the smoke holds a key until the first frame in which `total_updates` has advanced
  since the press (`Script.press_updates`); outside the race the 0.12 s / 3-frame hold stays (the FE runs per frame).

### Requests from physics (HUD / audio rpm, transmission) — rpm done (call sites checked in code: player order audio `engine_rpm`, `shift_gear`, HUD `engine_rpm`; no traffic call)
- **Done**: `car_outputs` (after `physics_tick`, same state) calls `PhysicsWorld::engine_rpm(player)` twice per
  update, audio first (UpdateRpm 0x00052401 → `CarAudioInput.engine_rpm`), then the HUD (0x000F9D72 → `hud.rpm =
  (float)(int)`; in the original the OSD call is in the next update's Osd set, before its physics frame, so it sees
  the same state). Binary `car_audio.rs`: one call per ai race vehicle without startline revs (UpdateAiCars
  0x000534EC else branch); none for traffic (UpdateTraffic 0x00053B36 sits under slot +88, which nothing ever sets;
  audio's scan); police make no call (UpdatePolice 0x00054060 has
  none). `car_audio_input` leaves `engine_rpm` 0 for the caller. `set_audio_max_rpm(6000)` is called in
  `car_inputs` right after the human slot-0 car is spawned (StartPlayerEngine 0x0004C3A0 runs inside the port's
  audio, so the physics global is set as the car exists, before the first `engine_rpm`). CarRender 0x0011ABD5's
  call (engine shake) is graphics'. Audio's traffic port has no engine pitch from rpm (request in audio.md).
- (original request) **HUD and engine audio rpm are the wrong field.** `race_cars` feeds `hud.rpm` and `engine_rpm` from
  `PhysicsCarState.engine_speed * 9.549` (FastCar rad/s as rpm: 5500..7700 at full throttle, the 6000 gauge pegged).
  The original reads `VsConfigFileRead 0x001B6F10` (= `PhysicsWorld::engine_rpm(id)`): engine speed x GC
  `RPM_multiplier`, rate limited, smoothed in VE+4068 and mapped to `1000 + rpm/1000 * (dword_6D4D98 - 1000)`
  (1000..~6000). Call sites: COSDInGameScreen::Update 0x000F9D72 (OSDBridge +0 = `(float)engine_rpm(player VE)`, once
  per OSD update), VsVehicleAudio::UpdateRpm 0x00052401 (per car, `(float)` x dword_627610 into +0x8C), VsAudio::
  UpdateAiCars 0x000534EC / UpdateTraffic 0x00053B36, CarRender 0x0011ABD5 (engine shake, /8000). Each call smooths
  once more, so call it where the original does (not once into a cached field).
- **`PhysicsWorld::set_audio_max_rpm(rpm)`** = `dword_6D4D98`, which VsAudioEvents::StartPlayerEngine 0x0004C3A0 sets
  to 6000 (audio's `ai_max_rpm`). It is 0 in the xbe image, which turns 0x001B6F10's scale upside down: call it when
  the player engine starts (before the first `engine_rpm`).
- Transmission: nothing to do in core — `LocalConfigInputs::from_setup` now carries `automatic` from the setup's
  `unk_18c` (+0x18C) and the spawn applies it (0x001DA8E0). With `automatic == false` the car never shifts up by
  itself; gear up/down come from `PhysicsCarInput.gear_up/gear_down` (pad +16 bits 4/8, edge-triggered in 0x001CB530),
  so a manual car also needs `EAction::CarGearUp/Down` bound on the keyboard (KEYMAP.INI) or it sits in 1st.
- `VELOCITY_PHYS_GEAR_TRACE=1` logs each human car's gear / rpm per physics tick ("phys gear:").

### Requests from audio (rpm call order, traffic rpm call, throttle read) — done
- `player_audio_wheels(w, id, ..)` reads the history entry from the VE+3560 heap block (`ve.ptrs[&3560]`,
  `w.glob.heap.block`); `car_outputs` calls `engine_rpm` (audio), `shift_gear`, `engine_rpm` (HUD) in that order;
  `car_audio.rs` makes no traffic `engine_rpm` call. Seen (LADown circuit, `SMOKE_DIRECT=1 SMOKE_FOLLOW_LINE=1
  SMOKE_DRIVE=22`, `VELOCITY_AUDIO_TRACE_ENGINE=1` -> `research/audio/core_engine_trace.txt`): `thr 1.00` on every
  full-throttle frame (421 of 1154; 0 on the grid and in the pause menu), rpm 814 -> 5413 in 1st/2nd, drops on each
  shift (gear 102 = shift held), 3rd at ~4500. Clip with sound `research/video/core_engine.mp4` (49 s, mean -16.3 dB,
  rev counter and gear on the speedo). Original request:
- **Throttle / front suspension always 0** (`race_cars.rs::player_audio_wheels`): the VE input history is not in
  `ve.hist` (that Mem stays all zeros) — VE+3560 is a heap block, `ve.ptrs[&3560]` = `Ptr { sp: Heap(0), .. }`.
  Read it as the physics code does (checked in a probe against the 350Z on the flat plane: throttle 1, +0x2C 1):
  `let hp = s.ve.ptrs[&3560]; let (m, o) = w.glob.heap.block(hp.off + e as i32);` then `m.f(o + 0x24)` /
  `m.i(o + 0x2C)` (e = 208 * head as now). With 0 the rev bands only ever play the off-load set (on-load bands at
  volume 0), the dump valve never fires and UpdateSuspension's front pair is wrong (LA trace
  `research/audio/rpm_trace.txt`: `thr 0.00` on every frame of a full-throttle drive).
- **Player order**: 0x001B6F10 itself calls 0x001B74A0 (steps the held gear, VE+4048/4052) at 0x001B6F10's else
  branch. The original order per update is UpdateRpm 0x00052401 (`engine_rpm`) first, then UpdatePlayerEngine
  0x0004C44E (`shift_gear`). `race_cars.rs` calls `shift_gear` before `engine_rpm`, so the gear audio sees is one
  step older and the rpm one step newer than the original's: please call `w.engine_rpm(pc.id)` first, then
  `w.shift_gear(pc.id)`, then the HUD's `engine_rpm`.
- **Traffic**: drop the `engine_rpm` call per active traffic car in `car_audio.rs`. UpdateTraffic 0x00053B36 is
  inside `if (slot +88 != 0)` and nothing ever plays into a traffic slot's +88 (scan of every VsAudio*.cpp function,
  `research/audio/traffic88.txt`: only UpdateTraffic's own StopEvent touches it), so the original never calls
  0x001B6F10 for traffic; the extra calls change VE+4068 smoothing that CarRender's engine shake reads.

### Requests from ai (respawn cameras, timeline AI, name colour) — done
- Binary `ai_host.rs` installs `AiHostResource(CoreHost)`: `camera_cut` / `crash_camera_reset` /
  `camera_point_of_interest_clear` are recorded into `game_cameras::AiCameraCalls` (shared `Arc<Mutex<Vec>>`, the host
  runs inside the ai's systems) and applied in call order by `apply_ai_camera_calls` (GameTick, after Ai, before
  Physics): `Cut{id}` → player record `id`'s cut (only players' records exist; the original writes any id's),
  `CrashCameraReset{physics}` → the car's slot < 4 → `crash.skip` (0x00063BA0), `PointOfInterestClear(p)` →
  `poi.set(None, 0, p)` (0x00069BE0). Cleared on leaving InRace.
- A respawn (VsAITrafficVehicle 0x0002B800) writes `velocity_ai::plugin::AiCameraCut { id }` (race car id, +0x78)
  and `GameMessage::CameraCut`: please apply the cut to the camera manager's player record (0x0002C620 on
  `dword_6D55C8`: `+44*id+40` = 1). For race car 0 it also calls `AiHost::crash_camera_reset(physics)` (VsCrashCamera
  Manager 0x00063BA0, the crash camera of the car's pad slot when < 4) and `AiHost::camera_point_of_interest_clear(0.5)`
  (0x00069BE0 with no vehicle): nothing implements `AiHost` yet; install an `AiHostResource` or ask for messages.
- Done on the ai side (docs/ai.md "Timeline AI events and race progress"): `TimelineObjectMessage`
  EnableRaceSection / RespawnRacersOnRaceline / PlayerHumanControl / Respawn are applied after
  `InGameUpdateSet::Timeline`; `RaceProgress` is filled after every Ai-set update (all fields but `results_busy`,
  `car_flags`). Note RespawnRacersOnRaceline (0x00176D90) respawns the **player** cars only.
- Name tag colour: `VsAIDriver::name_colour()` (docs/ai.md).

### Requests from frontend (cinematic bars) — done
- One stream `race_session::OsdCinematicBars(bool)` (OSDManager 0x000ED6E0 on the game OSD), written in GameTick call
  order by: crash camera / point of interest events (`game_cameras::apply_crash_events` / `apply_poi_events`),
  `TimelineObjectMessage::CinematicBars` (`race_effects_bridge::timeline_audio`), RestartRace (false, `game_manager`),
  the ai hand-back (`ai_host::ai_bars` forwards `AiCinematicBars`). Binary `hud.rs::cinematic_bars` →
  `HudCommand::CinematicBars(b)`.
- OSDManager 0x000ED6E0(show) shows / hides the in-game OSD's menu_top / menu_bottom bars (0x000F9980, items 23 /
  24). Please send `velocity_frontend::hud::HudCommand::CinematicBars(bool)` for the events nobody consumes yet:
  `CrashCameraEvent::CinematicBars` (0x00062C10 true / 0x00062D90 false), `PoiEvent::CinematicBars` (0x0006A720 /
  0x0006A7B0), `TimelineObjectMessage::CinematicBars` (0x0017A540 / 0x0017A580), GameManager::RestartRace 0x000BE26D
  (false, when the OSD manager exists), and the ai's hand-back after a respawn (VsAIRaceVehicle 0x00018BD7: false right
  after SetHumanControl(1); asked of the ai in docs/ai.md).
- ai: done as the message `velocity_ai::plugin::AiCinematicBars(bool)` (written by the Ai-set update in `GameTick`;
  velocity_ai cannot depend on velocity_frontend). Please forward it 1:1 as `HudCommand::CinematicBars(b)`. The same
  hand-back also clears OSD flag 1 (0x00023D00), which the ai writes itself as `RaceSessionMessage::OsdFlags`.
## Fast test runs (SMOKE_DIRECT, parallel sweep)
- `SMOKE_DIRECT=1` (`crates/velocity/src/smoke_test/direct.rs`): boots straight into the race chosen by SMOKE_AREA /
  SMOKE_TRACK / SMOKE_RACE_TYPE / SMOKE_RACE / SMOKE_RACE_PATH / SMOKE_WET / SMOKE_CAR. It enters the front end's own
  QuickRaceTextScreen once (option defaults come from the screen), stores the selections as qr_select / car select do and
  pushes `FrontendRequest::StartQuickRace` (same GameFlowRequest path as the loading screen). The normal pause-menu
  walk afterwards is unchanged; without the variable nothing changes.
- `tools/sweep.py --parallel [N]` (default 3) runs N games at once: own log, `SMOKE_PREFIX` and save dir
  (`research/sweep/saves/<key>`) per run, rows merged into results.tsv (same columns) as each finishes. `--direct`
  adds SMOKE_DIRECT=1, `--tsv FILE` writes another results file (use it to test without touching results.tsv).
- Timing, plain smoke (LADown, `SMOKE_DRIVE=10`, hidden, target/fast): before (menu presses) 88 s wall, after
  (`SMOKE_DIRECT=1`) 42 s wall, both exit 0 and reach "drove 10.0 s".
- **LACOAST Circuit Race05 hang (the only non-pass keys, dry and wet, "killed")**: `VELOCITY_TICK_TRACE=1` markers
  (vs_common_main, one thread) bounded it to `stream_raceline`: the ai's `point_ahead` (0x000214B0) walks section 1
  -> 3 -> 1 ... forever (both racelines 255.1, each one's last sub-route point joins the other at dist 255.1, so d
  never drops below the length). **Original behaviour** (ai.md: LACoast circuits duplicate race sections, +16 is the
  distance on the target section; unreachable in F3_D, whose only caller is CompartmentManager 0x000897C0 and whose
  LACoast eventdefs int3 at load), so the guard stays. `stream_raceline.rs::point_ahead_ends` simulates the walk (64
  steps) before each call and stops the raceline points there, logging the walk once ("never ends").
- Sweep state (`--failed --direct --parallel 3`, target/core, 2026-11-05): Race05 dry / wet pass (98 / 99 s), so
  `research/sweep/results.tsv` is **164 / 164 pass**. The in-race smoke key hold is per game update (see "Requests
  from frontend").
- Sweep after the shared rand stream / explosions / restart wiring (2026-11-06, `--parallel 2 --direct`, target/core,
  `research/sweep/core_results.tsv`, log `research/core_sweep_run.log`): 159 pass first run, then all 5 pass on
  rerun = **164 / 164**. The failures were environmental (CIRCUIT_RACE03 dry+wet: "Monitor removed" closed the
  hidden window; SPRINT_RACE04_wet: wgpu "SwapchainAcquireSemaphore still in use" panic) except an **intermittent
  hang on LACOAST Sprint Race03** (killed 3 times under parallel load, passed 3 times): the player reaches section 1
  (raceline 296.0) whose last sub-route point joins section 3 at 296.0 and back (same duplicated-section walk as
  Race05; the guard logs "never ends" and stops). **Root cause (2026-11-06, symbolised stack of the hung process):
  not game code** — the sweep's `SMOKE_FOLLOW_LINE` steering in `smoke_test::run` (Update) called the unguarded
  `point_ahead(25)` and walked 1 <-> 3 forever. The walk itself is the original's (0x000214B0 has no other exit), so
  the fix is in the harness: it now runs `stream_raceline::point_ahead_ends` first and has no target when the walk
  never ends. Those two are the only `point_ahead` callers; any new caller needs the same guard.
- Full regression sweep after back-face culling removal / all-subsystem debug tweaks / fury FX / camera tweaks (2026-11-06,
  `--parallel 2 --direct`, target/core, `research/sweep/core_results.tsv`, log `research/core_sweep_run.log`, 7987 s):
  163 pass first run; LONDON2_CIRCUIT_RACE09_wet died after 12 s on the known wgpu-hal vulkan swapchain-semaphore panic
  (environmental, parallel-run flake, not game code) and passed on rerun (99 s) = **164 / 164**. No regression from the
  recent commits.
- **Hung-process stacks**: `.venv-ida/Scripts/python tools/stacks.py <pid> 60 "<repo>/target/core/debug"` (dbghelp, suspends all
  threads, symbolised with the pdb dir as 3rd arg) names the hanging function directly; quicker than tick traces.

### Requests from physics (jump cam, attract RandomiseSetup, zone FX boost block) — all done
- **Jump cam wired** (`game_mode::jump_cam`): `JumpCam` resource = `Arc<Mutex<JumpCamState>>` (fixed camera, manager
  +0x1BE review, +17) + the hook closure. Physics chain: `jump_cam_pre` (before `physics_tick`: hook set every tick
  (Arc clone, survives world reloads), option `byte_75B278` = `replay.jump_cam_option` (binary copies
  `GameOptions.display[JUMP_CAM]`), `set_review_cameras`, `set_review_timer`, `replay.fixed` moved into the state),
  `jump_cam_post` (fixed back, +17, `review_cameras()` / `review_timer()` -> `replay.review_cameras` /
  `review_timer` / `review_timer_start`). Hook: CanSee = 0x001665E0 (choose -> `fixed.rechoose`, then
  `fixed.choose(review, HookWorld)`), Start = 0x001665A0 (+17, rechoose). `HookWorld` adapts `JumpCamWorld` to
  `ReplayWorld` (wheel / axes zero: unused by 0x001649A0). Countdown 0x000B8329 = `review_timer_countdown` (Effects
  set): -0.04166 per update, at <= 0 review 0 + render cut (0x00060380(1)). Not done: `dword_68580C` = 24 (physics
  Cx global, no setter; request in physics.md; written by VehicleEntity 0x001C5800, the jump's slow motion).
  VsRender 0x0005C8A0 = the camera manager getter (`dword_6F1038`, created on first use), i.e. the 0x00060380(1)
  call's `this` (dump `research/core_jcend.c`), so the countdown end is complete apart from 68580C. `fixed.vehicle` is left as the
  replay manager sets it (= physics id while no car was removed). `CamWorld::blocked` now = 0x00167680 (flags 23,
  followed car ignored).
- **Jump cam in runs** (`SMOKE_JUMP_CAM=1`: option on, logs take-offs "player rising", the hook's "jump cam: can see
  ... (fixed cameras n, vehicle, car at, camera)" / "start", review changes, shot `9_jump_cam`). Trigger chain of
  0x001CCAD0 (checked against tr.rs): landing predictor at take-off (PL_enabled, VE+3688 == 0, car +2876 != 0, up
  speed > PL_min_speed_up, horizontal > PL_min_horisontal_speed, VE+4716 == 0) -> flight +4592 > 0.65 && player
  VE+4072 && cooldown VE+4876 == 0 && !network -> CanSee(1) **before** the option / probability tests (so CanSee is
  called with the option off too) -> option -> 1/3 chance -> review 2, Start, countdown = flight − 0.1, cooldown 96.
  LADown is full of qualifying jumps (circuit Race01 ~4 s after Go at (365, 270); Race02 route at 44 / 52 / 73 s) and
  CanSee is reached, but it answers false: **only London / London2 races ship fixed cameras** (`Cameras.cam` count >
  0; every LA / LACoast / Tokyo / ExecTest file is empty except LADown drag 01 with 2), and 0x001649A0 returns 0 with
  no camera, as the original. So the jump cam can only ever fire in London — and the London runs (London Circuit 01,
  London2 Circuit 03, raceline driven 120 s) had no take-off over 3 m/s at all (flat city), so it was not seen
  firing then (it fires now, see "Why Race09 did not fire" below). Logs
  `research/core_jump_{ladown2,lacoast,london1,london2}.log`, `research/core_police.log` (LADown Race01 CanSee).
  Next to try: a London race with a known ramp / drop, or `SMOKE_VIA` over one.
- **London ramps** (ignored probe `crates/velocity/tests/london_jump_probe.rs`: raceline crests with a slope change >
  0.12 near fixed cameras -> `research/core_london_crests.txt`): London2 Circuit 08 (-583, 7.2, -437) (camera 11 m
  away, but ~3 km in: not reached in 200 s of follow-line driving), Circuit 09 (-334, 7.0, -198) (camera 52 m).
  **London2 Circuit Race09** follow-line takes off at (-493, -9.8, -489), 53 m/s, 94..142 s after Go, and reaches
  CanSee -> `CReplayCameraFixed::choose` -> `nearest` -> physics' `blocked`, which **panics in physics**
  (`f_19d3e0` car pass reads VE+3556 object +188 from a 132-byte block; request in physics.md, log
  `research/core_jump_london2_r09bt.log`) — fixed by physics since. Race08 log `research/core_jump_london2_r08.log`.
- **Why Race09 did not fire, and where it does** (`jump_cam_post` now logs player 0's VE+4592 predicted flight and
  cooldown VE+4876 on change; the CanSee line logs the option): the 3.1 m/s hop at (-492, -9.8, -489) predicts
  **0.634 s** (wet 0.642 / 0.649), under `flt_6889F4` 0.65, so 0x001CCAD0 never calls CanSee — same in the original.
  ~10 s later the crest at (-578, 7.8, -439) (raceline pt 62, camera 18) predicts 0.95 s, CanSee → true, option on,
  then the **1/3 roll** (`PL_jump_cam_probability` "1/3" evaluated to 0.3333 > VsRandUnit) decides; the dry
  follow-line run fails it every time (5 runs), the wet `SMOKE_LINE_OFFSET=-1.5` run passed it once and failed on a
  rerun. The port's conditions match the original; the difference is the random stream: physics' VsRandUnit uses its
  own CRT `rand` state (`cx.glob.rand_state`, seeded 1 per world), while the original's is the one CRT stream every
  system draws from, so in F3_D the roll there is effectively random. Next crest (-210, 8, -197): CanSee false.
- **What the jump cam shows (original too)**: review 2 skips the follow cameras (0x001C5CF0 gate `dword_749110 ==
  0`) and runs the replay manager, whose camera type is never switched to fixed (only 0x00166700 does, from
  CAMERA_SWITCH triggers, which never fire), so CanSee's fixed pick is unused and the view is the manager's current
  camera: the **wheel camera of vehicle 0** (reset 0x001658C0 sets type 3) for flight − 0.1 s, then the countdown
  cut back. Clip `research/video/core_jumpcam.mp4` (`VELOCITY_EXE=target/core/debug/velocity.exe
  tools/record_smoke.sh core_jumpcam --env SMOKE_DIRECT=1 SMOKE_AREA=2 SMOKE_TRACK=LONDON2 SMOKE_RACE_TYPE=3
  SMOKE_RACE=9 SMOKE_FOLLOW_LINE=1 SMOKE_LINE_OFFSET=-1.5 SMOKE_WET=1 SMOKE_JUMP_CAM=1 SMOKE_DRIVE=105
  SMOKE_RECORD_FROM=75`, each as `--env`): the wheel view at ~25.5..27 s, then back on the chase camera. A clip of a
  failed roll: `research/video/core_jumpcam_nofire.mp4`. The wheel camera's reset 0x001678E0 (this+364, called by
  0x001658C0 after the fixed camera's) is an **empty function** in F3_D; ported as `CReplayCameraWheel::reset` (no-op)
  and called from `CReplayCameraManager::reset` in that place.
- **`dword_68580C` "System Update Rate" is dead in F3_D**: its xrefs are writes only (VsCommonMain 0x0016E17C 24.0 at
  boot, FrontEndInit 0x000B5627, InGameInit 0x000B6099, InGameUpdate 0x000B836B, VehicleEntity 0x001C5800) plus the
  debug tweak registration (FontHelper 0x000B38AC, range 1..60). The frame timer's rate is set by VsBounds 0x00057450
  with immediates (FrontEndInit **50.0**, InGameInit 24.0, boot 24.0), never from it, so the jump cam "slow motion"
  changes nothing. Port: `jump_cam::SystemUpdateRate` (24 on entering InRace and at the countdown end), written to
  physics before each step (`set_jump_cam_rate`) and read back after it (`jump_cam_rate`). Not ported: the FE timer
  at 50 Hz (FrontEndInit 0x000B561A).
- **+0x18C clear done**: `plan_race_cars`: current race entry flag 2 (PhysicsManager 0x001B4010 =
  `event.current_race().flags & 2`, the drag flag) -> `setup.unk_18c = 0` before RandomiseSetup, both passes.
- **RandomiseSetup done**: `race_load_steps` passes `Some(&mut gm.rand)` (a copy written back) to `plan_race_cars`
  while `CAttractMode.playing`; every planned car's setup goes through `db.randomise_setup(.., true, ..)` in pass
  order (players 0x0019E30E, then opponents 0x0019E53A). Difference kept small: the original randomises the garage /
  opponent database setups in place (a lasting change of the player's car after an attract race); the port only the
  race's copies. The `+0x18C` clear: see above.
- **Zone FX done**: `race_cars::zone_fx_levels` (Physics chain, right before `physics_tick`) mirrors graphics'
  `ZoneFx.level[0..2]` with `set_zone_fx_level`.
- (history) Jump cam was blocked on a ray test inside `step`; physics' answer and the original requests:
- **Physics answer (jump cam world, 2026-11-05)**: the hook now gets the step's world view. Signature
  `PhysicsWorld::set_jump_cam_hook(Some(Arc<Mutex<cx::JumpCamFn>>))`, `JumpCamFn = dyn FnMut(JumpCamCall, &mut dyn
  JumpCamWorld) -> bool + Send` (`velocity_physics::vehicle::vehicle_entity::cx`). `JumpCamWorld::car_position(
  vehicle) -> [f32;3]` = vehicle array [vehicle] -> 0x001BA270; `blocked(from, to, vehicle) -> bool` = 0x00167680
  (RayCast flags 23 = track | walls | cars, `vehicle` ignored). Wiring, 1:1 with ReplayCamera_Manager:
  1. On race load (after the world exists) install the hook once, capturing the replay manager state the calls touch
     as `Arc<Mutex<..>>`: the fixed camera (manager +260) and the manager's +17 flag; share that Arc with
     `GameCameras.replay` (or copy in before / back after each `physics_tick`). Remove it (`None`) on unload.
  2. Adapt the view to `ReplayWorld` (its methods take `&self`): a wrapper holding `RefCell<&mut dyn JumpCamWorld>`,
     `car_position` -> `Vec3::from(w.car_position(v))`, `blocked` -> `w.blocked(from.into(), to.into(), v)`;
     `wheel_position` / `car_axes` are not reached by 0x001649A0 / 0x00164EC0 (return zero).
  3. `CanSee { choose }` (0x001665E0): if `choose`, 0x00166B70 = set the fixed camera's re-pick byte +13 = 1; then
     return 0x001649A0 (`CReplayCameraFixed::choose(review, &view)`, review = 0x00166490(byte_749128)).
     `Start` (0x001665A0): manager +17 = 1, then 0x00166B70 (+13 = 1); return value unused.
  4. The fixed camera's `vehicle` (+0) is the vehicle array index: physics' array holds the live car ids in
     ascending order (= the physics car id while no car was removed), so set it from `PhysicsCar.id`.
  5. Unchanged from below: option / review / timer setters before the step, `review_cameras()` == 2 after it starts
     InGameUpdate 0x000B8329's countdown.
  Also for 1:1: `CamWorld::blocked` (game_cameras.rs) casts `RAY_TRACK | RAY_FIRST_HIT` with no ignored car; the
  original 0x00167680 is flags 23 ignoring the followed car = `ray_cast(from, to, RAY_TRACK | RAY_OBJECTS | RAY_CARS
  | RAY_NORMAL, vehicle as i32)` (`line_check(from, to, true, vehicle)`).
- **Jump cam** (physics.md "Requests from core (jump cam, attract RandomiseSetup)"): install
  `PhysicsWorld::set_jump_cam_hook(Some(Arc<Mutex<FnMut(JumpCamCall) -> bool + Send>>))` answering
  `CanSee { choose }` = ReplayCamera_Manager 0x001665E0 (choose 0x00166B70 when `choose`, return 0x001649A0) and
  `Start` = 0x001665A0 (+17 = 1, choose). It is called inside `step` on the physics thread: it must not lock
  `PhysicsWorldRes`; give it shared state (e.g. an `Arc<Mutex<>>` of the fixed cameras + car positions). Each tick
  before the step: `set_jump_cam_option(video Jump Cam)` (`byte_75B278`), `set_review_cameras(replay.review)`,
  `set_review_timer(countdown)`; after it read `review_cameras()` / `review_timer()` back (2 = jump cam started)
  and run InGameUpdate 0x000B8329's countdown.
- **RandomiseSetup**: `db.randomise_setup(&mut setup, true, &mut rand)` for every race car in
  `race_cars::plan_race_cars` while `CAttractMode.playing` (PhysicsManager 0x0019E30E / 0x0019E53A), with the game's
  VsRandUnit state.
- **Zone FX boost block**: each tick before the step `set_zone_fx_level(pad, level)` for pads 0/1 from graphics'
  `ZoneFx` level (`flt_737AB0` +24 + 4·pad; 0x00154FC0 halves it); while non-zero the pad's boost is cleared.

### Requests from ai (drag rounds, 2026-11-05) — done: rounds restart in place
- **Cause of the round 2 reload**: `InRace` is a ComputedState and bevy_state 0.19 defaults
  `ALLOW_SAME_STATE_TRANSITIONS = true` for computed states, so InGame -> PostRace and PostRace -> InGame ran
  OnExit(InRace) + OnEnter(InRace) (= despawn + InGameInit: cars planned twice, physics id mismatches, ai world
  rebuilt). `InRace` now sets it false; test `game_manager::tests::in_race_survives_post_race`. Mode 2 -> 1 is
  RestartRace on the loaded race (`restart_pending` -> `RaceRestart`), as 0x000BF270 / 0x000BB1C0. This also stops
  the post race mode of every race from reloading the race. Seen (`SMOKE_DIRECT=1 SMOKE_RACE_TYPE=5 SMOKE_REVIEW=1`,
  `research/core_drag_inplace.log`, exit 0): round 1 finishes at 42.7 s, PostRace -> Intro -> Countdown in 0.2 s
  with no reload (one car plan, one AI world load, no physics id errors), "AI: drag car 3 eliminated".
- GameManager **+0x14C is the eliminated car** (VsRaceManager 0x0003DB70, set in MonitorSessions' drag branch when the
  player qualified), not a round winner; RestartRace 0x000BE0F0 deactivates it. The ai now does both itself: it reads
  `RaceSessionMessage::DragResult(WINNER|QUALIFIED)` and `GameModeMessage::RaceRestart` (+ `RaceSession.drag_round`:
  > 0 = continued round, 0 = 0x000BF270's drag restart, which reactivates every ai car). Please keep `drag_round`
  intact through `restart()` on a continued round (as now) and don't deactivate cars yourself; `drag_winner` can stay
  None (or go). The ai also runs VsAIWorld::Reset on every RaceRestart now (RestartRace's first call).
- **Please check (drag round 2 reloads the race)**: `SMOKE_DIRECT=1 SMOKE_RACE_TYPE=5 SMOKE_REVIEW=1` (log
  `research/ai_review/drag_rounds.log`): round 1 finishes (42.2 s, player last = ELIMINATED, yet a round 2 follows),
  then the PostRace -> InGame step re-runs the race load: `race_cars` plans the cars twice (log lines 538 / 553), then
  ~17 `ERROR race_cars: race car N: physics vehicle id M` mismatches, and the ai world is rebuilt with 4 ai cars
  (round 1 had 3). A rebuilt ai world also drops the +0x14C car (a pointer into the old world in the original, so the
  original's next round must keep the loaded race and only RestartRace it). Also: should an eliminated player get a
  next round at all (`drag_out` is set)?

### Requests from audio (no VsAudio::Update during the race load) — done
- `game_manager::in_game_init` (OnEnter InRace, with the race `Init`) sets `AudioUpdateGate(false)` and resets
  `GameAudioInput` to default (the last race's cars / camera are gone); `audio_update_on` (GameTick, Audio set, InRace,
  = InGameUpdate 0x000B7EE7) sets it true on the first in-game update; `front_end_init` sets it true
  (CFrontendManager::Update 0x002016DD). Frame order: OnEnter (Init) -> Update applies Init, update gated -> PostUpdate
  LoadLevelSounds step -> next frame's GameTick opens the gate, Update applies LoadLevelSounds before the update.
- (original request) VsAudio::Update 0x00048BC0 is called only by CFrontendManager::Update 0x002016DD, GameManager::InGameUpdate
  0x000B7EE7 and Pause/Resume, so it never runs while a race loads. The port ticks it every frame: between the race
  `Init` and `LoadLevelSounds` it plays with the previous race's `GameAudioInput` (second career race: city_ambience
  and 7 passbys fail before the groups load, `research/audio/review2/career.txt`). Audio added resource
  `velocity_audio::AudioUpdateGate(bool)` (default true): please set it false when the race Init is sent and true again
  at the first InGameUpdate (and true in the front end), and/or reset `GameAudioInput` on leaving InGame.

### Requests from frontend (next event race, split screen, garage copy) — done (split-screen race itself open)
- **`GameFlowRequest::StartCurrentEvent`** (handled in `career::career_requests`): `career::start_current_event` =
  GameManager 0x000BBF60 (+0x214 = 1, +0x218 = 0; dump `research/core_bbf60.c`) = the race part of StartEvent: jump to
  `m_pEvent`'s current race (professional = category 2, dry), `career_start` (the event is raced as it is: no reset,
  no `event_state` clear, car / `num_players` untouched); answers `CareerMessage::EventStarted { event crc }` and sets
  `from_race`, like StartCareerEvent. `start_event` now ends by calling it. Please map `CareerRequest::ContinueEvent`.
- **Players**: `front_end_update` sets `sel.num_players` (-> `gm.num_players`, `race.num_players`) = 2 when the quick
  race event has `two_players`, else 1 (MultiplayerScreen 0x00217D70 / main menu 0x0023CE80 `dword_6F5C00`). Player
  2's car: `PlayerCarChoice.setup2` (`dword_6F7478`), filled by the binary bridge from `quick_race.player_setups[1]`;
  `plan_race_cars(.., player2, ..)` gives human 1 that setup (falls back to player 1's).
- **career_sync**: the copy branches are gone; it only handles `EventStarted` (post race / return screen, Stop).
- **Split-screen race (not started; what it needs)**: core: two `TrackCamera`s with viewports (top / bottom halves,
  `place_camera` per player from `GameBridge::camera_view(p, lerp)`), camera manager follow / mirror per player (exists
  per player), pad 1 mask -> player 1 input (`GameInput::device_mask(1)`), `stream_views` for player 1, HUD per player.
  Graphics: everything that assumes one world camera / view 0 (`TrackCamera` singletons: sky, FSE, DOF, motion blur
  (already off when `byte_6F7157`), zone FX, lens spray, camera rain, flares `HeadlightView.inside`, dynamic envmap
  `two_players`), the per-view render flags. AI: two `m_pPlayerVehicles`, player 1 human (control mode 0), its
  race vehicle / positions in `RaceProgress.player_positions[1]`, TFT per player. Frontend: OSD per player viewport.
  Physics: pad slot 1 input for car id 1 (`LocalConfigInputs` pad = id already).
- **Continue an event (needed for multi-race events)**: DummyGameScreen (FEDummyGameScreen.cpp 0x00209340, now ported)
  is where RaceCompleteScreen goes while the career event is not over (flag 0x10). Its "Play Race" goes to the
  LoadingScreen, whose 10th frame = GameManager 0x000BBF60 starting `m_pEvent` **as it is** (next race, no
  `StartEvent` reset / `event_state` clear). The front end sends `CareerRequest::ContinueEvent`, which has no
  `GameFlowRequest` yet (logged and dropped in `velocity_frontend::lib.rs::career_out`). Please add one (e.g.
  `GameFlowRequest::StartCurrentEvent`: the race part of `StartCareerEvent` without `ev.reset` / `event_state = 0`)
  and I will map it. The result branch (a chosen position) is done in the front end with your public pieces:
  `race_finished` / `round_finished` / `update_at_end_of_event` / `race_session::apply_event_outcome`, then
  `career::race_complete` + `CareerMessage::ReturnedFromRace` (the save), plus `RaceSessionMessage::{Mileage,
  EventOver}` (`lib.rs::dummy_result`). Note the Mileage message is written while in the front end.
- **Split screen** (MultiplayerScreen 0x00217D70 entry 2, now ported): sets `Frontend.fe.quick_race.two_players`
  (`byte_6F7157`; the main menu's quick race clears it), both players pick cars (`quick_race.player_setups[0/1]`,
  names `car_name_crc` / `car_name_crc2`), then `StartQuickRace` as usual. The original also sets
  `dword_6F5C00` (= `gm.num_players`) to 2 there and to 1 in the main menu's quick race (0x0023CE80); please take
  `num_players` from `two_players` when building the quick race (and player 2's car from `player_setups[1]`).
  The split-screen race itself (cameras, OSD per player) is yours / graphics'.
- **Shared garage copy (done on my side)**: `run_frontend` now reads `gm.vars` {cash, prestige, garage_vehicle,
  garage_setups, garage, garage_slot} into the screens right before the FE update and writes them back right after
  (bypassing change detection). `bridge.rs::career_sync`'s copy branches (`push` and the FrontEnd write-back) can go;
  keep its `EventStarted` part (post race / return screen, `FrontendCommand::Stop`).

### Requests from graphics (dummy police car, flashes) — done
- Wired: `physics_object_render::dummy_police_cars` (Update, InGame, kept while paused, cleared on leaving InRace)
  refills `DummyPoliceCars` every frame from the snapshot: objects with `is_dummy_cop_car()` that are drawn
  (`flags & 8 == 0 && flags & 2`), key = `PhysicsObjects` index, matrix = rotation + translation of the lerped pose
  (same lerp as `place_objects`); `place_objects` keeps those objects' own models hidden. `spawn_traffic_and_police`
  sets `spec.police = true`. `race_effects_bridge::timeline_screen_flash` (GameTick after the Timeline set, InGame)
  turns `TimelineObjectMessage::ScreenFlash { time_in, time_out, 0.5 }` into `FlareFlash(FlashRecord { rise: TimeIn
  (item +4), fall: TimeOut (+8), intensity 0.5, screen: true })` (F3EventScreenFlash = {bool TestAttribute; float
  TimeIn; float TimeOut; float Intensity; VsVector RGB}); log "screen flash: rise .. fall ..".
- (original request) **Dummy police car** (PhysicsObjectParticleTracker 0x001E3B70, from the object draw 0x001E0710 / 0x001E0EE0): every
  frame fill `visos_render::car_render::DummyPoliceCars` (clear it first) with one `DummyPoliceCar { key, matrix }`
  per drawn physics object (`flags & 8 == 0 && flags & 2`) whose type tag (record +56, `PhysicsObjects__1E5F30(obj, 1)`)
  is 1 = `propld_dummycopcar` (LADown Circuit Race02 / Race09), matrix = the lerped object pose (as you draw it), key =
  a stable per-object number (object index). `physics_object_render` must **not** draw those objects' own model — the
  original draws the police car copy instead, or nothing when the race has no police car. Physics exposes the tag
  (physics.md request).
- **`CarSpec.police = true`** on the police cars' specs (CarRender 0x00110220: +2328 bit 0x4000 for car index past the
  race + traffic cars); the dummies copy the first one.
- **Screen flash** (TimelineCameraShaker 0x0017A490): write `visos_render::flare_render::FlareFlash(FlashRecord { rise:
  item +4, fall: item +8, intensity: 0.5, screen: true, ..default })` where the shaker item (a4 +12 record) fires.

### Split-screen race (core side) — `game_mode::game_views`, `race_cars`
- **Views** = GameManager 0x000B3EC0 ("Init Game Views" step, `race_cars::init_game_views`, PostUpdate) with the
  rect setter 0x000B3960: `GameViews { views (+588), second (+988) }`, 10 x 40 bytes (x, y, w, h px of 640x480
  `dword_68797C/687980`, +28 player, +32 kind, +36 used). Kinds: 0 full, 1 mirror (W/3, 0, W/3, H/6), 2/3 top /
  bottom, 4/5 left / right, 6..9 quarters. 1 player: views[0] = 0, views[1] = 1 (mirror), second[0] = 0. 2 players:
  views[0] unused, views[1]/[2] = 2/3 (4/5 when **`byte_6F9180`** = options +0x24 is set — 0x000B3EC0 reads it as the
  split direction; the `time_format` name is doubtful), second[0]/[1] the same. 3/4 players: quarters, views[0]
  unused. Test `game_views::tests::two_players_split_top_bottom`.
- **Cameras**: every world camera has `PlayerView(p)`; player 0's is spawned at LevelArena as before, players 1..
  at Init Game Views; `track_views` (Update, InRace) keeps graphics' `TrackView { player, rect: GameViews::fraction(
  player_view(p)) }` (whole screen until the views exist) on each, and graphics sets viewports / orders / targets and
  moves player ≥ 1 to `SecondaryTrackCamera` (`player_viewports` is gone). Far = 20000 × `SPLIT_FAR_SCALE` with two
  players (0x0005ECC0). `place_camera` draws each player from `GameBridge::camera_view(p, lerp)` (cut-scene / override
  views only for player 0, as `write_camera_bridge`). Despawned with the race; `GameViews` reset then.
- **Camera manager** (`camera_load_steps`): split = `byte_6F7157` = players > 1 (was the dead `splitscreen` flag, so
  the far planes were never halved); each player's view table = 0x0005F0F0 on its own setup's vehicle
  (`dword_6F724C[139 * p]`, player 1 = `setup2`): list a (+8) for one player, list b (+12) side by side
  (`byte_6F9180`), list c (+16) top / bottom; vehicles from `Career.vehicles`, falls back to `PlayerCarChoice`'s table.
- **Players' cars / pads** (physics' split steps): `plan_race_cars` puts the humans first (ids 0..n-1), pad slot = id;
  the ai's `bind_race_cars` sets control mode 0 + human control per player; `car_inputs` reads player p's actions with
  `GameInput::device_mask(p)`. The quick race masks (0x0023CE80): player 0 = the selecting pad, player 1 = the rest.
- **`PlayerRaceStatus(Vec<RaceStatus>)`** (race_cars): `RaceStatus` per player for the HUD (COSDInGameScreen per player,
  0x000F99E0 reads everything by its player index): [0] = copy of `RaceStatus` (end of `car_outputs`), [p ≥ 1] speed /
  gear / laps / lap time and one `engine_rpm(p)` call per update (its screen's OSDBridge call) in `car_outputs`;
  racing / position / lap / race length per player in the binary's `hud.rs::race_manager_status`. Cleared on despawn.
  The frontend's `RaceHudInput` is still single (request in frontend.md).
- **Smoke `SMOKE_SPLIT=1`** (implies SMOKE_DIRECT; `smoke_test::direct::split_pad`): `two_players`, both players on the
  same car, masks 1 / !1, pad 1 scripted on port 1 (`visos::input::input_manager::set_port_script(port, keys)`; new):
  throttle 1 s..SMOKE_DRIVE, left steer 3..5 s; logs both players' status every 2 s, shot `1c_split`.
- Seen (LADown circuit, `SMOKE_SPLIT=1 SMOKE_DRIVE=14`, log `research/video/core_split.log`, exit 0): two 300ZX
  players on grid 0 / 1 bound as Player(0) / Player(1), both drive on their own pads (10 s: 24 / 25 m/s, positions
  2/5 and 1/5), top / bottom views with each player's chase camera, pause / quit walk as usual. Clip
  `research/video/core_split.mp4`. The HUD draws once over the whole screen (player 0's values, frontend request).
- **Stream views** are per player already: the caller 0x0008A270 passes 1 player, 2 with `byte_6F7157` (= the
  manager's players); a player without a current camera adds no view, so `game_cameras::StreamViewPlayers` keeps
  each view's player and the binary's `stream_raceline` uses that player's race vehicle (`v24`).
- **Audio camera = player 0 only, in split screen too** (`game_cameras::audio_camera`, GameTick Audio set, after the
  cameras; was filled in `car_outputs` before the Cameras set, one update stale): VsAudio::UpdateListener 0x0004B180
  = `Armco::13F70(g_CameraMan, 0)` position / direction 0x00023EB0, velocity = vehicle array [0] 0x001C6480 only
  when mode(0) == 1 (else `unk_7805C0`, zero; was always the car's); UpdateEmitters 0x00050770's view index
  `dword_685030` = 0x0005F430(0) = player 0's view (+4) (was hard-coded 0). VsAudio::UpdateVehicles 0x000524B0 reads
  no camera. Audio side difference (theirs): 0x0004B180 tests |dir|² and |position|² ≥ 0.01, the port tests |up|².
- Already per player before: camera manager follow / mirror per player, bridge camera records, `car_inputs` pad
  mask `GameInput::device_mask(slot)` for every human car (pad slot = id), the ai's binding (control mode 0 + human
  control per player, ai.md). `RaceProgress.player_positions / player_states` added for the ai.
- Open: frontend (OSD per view, request in frontend.md; `RaceStatus` is player 0 only).

### Requests from graphics (split screen views, 2026-11-05) — done (see "Split-screen race")
- Put `visos_render::view_render::TrackView { player: p, rect: GameViews::fraction(view) }` on **every** player's
  world camera (player 0 too); graphics cannot see `PlayerView`. With it graphics keeps player 0's camera as the
  `TrackCamera`, removes `TrackCamera` from player ≥ 1 (they get `SecondaryTrackCamera`), and sets every view
  camera's viewport (target-size based, so it also works on the FSE back buffer), order, clear, target, msaa, sky
  copies and the full-screen aspect itself — `player_viewports` can go (or must agree). Details in graphics.md
  "Split screen — two players".
- VsCameraManager 0x0005ECC0: with `byte_6F7157` both cameras' far planes are × 0.5 (600 → 300, 200 → 100): set
  `PerspectiveProjection.far` × `visos_render::view_render::SPLIT_FAR_SCALE` for two players (bevy uses `far` only
  for frustum culling, as the original's camera planes).

### Requests from graphics (rear-view mirror, per-view zone FX, 2026-11-05) — done
- Done: `race_cars::init_game_views` spawns `RaceCars.mirror_camera` (one player, view 1 kind 1; `MirrorView { player
  0, rect, on }`, no TrackCamera / TrackView); `place_mirror` (Update, InGame) sets `on` = GameManager 0x000B88B0
  (checked in IDA, dump `research/core_mirror_b88b0.c`: `byte_685808` (const, never written) && `dword_749110` != 2
  && mode(view player) == 1 && current camera +524 == 0 && !+585) and places it from `camera_view(0, lerp)[1]`
  (skipped while paused / before a record exists) with the manager's mirror near / far. Seen (`SMOKE_DIRECT=1
  SMOKE_CAMS=1`, shot `coremir_3c_view2`): mirror at the top centre on the bumper view only. `ZoneFxSecondInput`:
  `fill_render_feed` keeps player 1's car (human race slot 1: VE+20 pair, VE+89, trailblazer [1]),
  `fill_render_inputs` writes `Some(ZoneFxInput { player: 1, .. })` in two-player races, else `None` (also on leaving
  InRace).
- **Mirror view** (GameManager view 1, kind 1, one player): spawn one world camera (`Camera3d`, `Projection` fov from
  the bridge record, near / far = the manager's mirror near / far) with `visos_render::view_render::MirrorView {
  player: 0, rect: GameViews::fraction(views[1]), on }` — **no** `TrackCamera` / `TrackView` — and place it every
  rendered frame from `GameBridge::camera_view(0, lerp)[1]` (pos / target / up / fov, as `place_camera` does with
  [0]). `on` = GameManager 0x000B88B0's +36 for the view: `byte_685808` (1) && review cameras (`dword_749110`) != 2
  && the player's camera mode == 1 (follow) && the current follow camera's +524 ([14]) == 0 && !+585 (looking back)
  — i.e. only the static (bumper / in-car) views. Graphics sets its viewport, order (inside the FSE, after the lens
  spray), target, layers, clears and `is_active` (graphics.md "Rear-view mirror").
- **Zone FX for player 1**: fill `visos_render::zone_render_pc_xbox::ZoneFxSecondInput(Some(ZoneFxInput { player: 1,
  .. }))` like `ZoneFxInput` for player 0 (player 1's car speed / zone flag / one-off) in a two-player race; `None`
  otherwise. 0x00162D10 runs per view with the view's player.

### Requests from graphics (per-view HeadlightView / in-car hide, 2026-11-06) — done
- `game_cameras::headlight_view` (Update, InGame, after `race_cars::place_mirror`; both reset on leaving InRace) fills
  `HeadlightViews` with `view_headlight(p, mode)` = CarRender 0x001127F0 for view player p: `player1` = (1, mode(1))
  while the manager has > 1 player (`byte_6F7157`), `mirror` = (0, mode 2) while player 0's `MirrorView.on` —
  GameManager 0x000B9010 switches the mirror view's player to mode 2 (0x0005FFE0(2, p)) before 0x001127F0, so the
  car's mirror camera VehicleEntity 0x001C57C0 (= manager +0x10) is tested: camera +580 == p and !0x00069880
  (0x00069810 now uses the camera's own player for the in-car view test). Dumps `research/core_b9010.c`.
- **TrackStreamViews** were already per player (see "Split-screen race": 0x0008A270 passes 2 players with
  `byte_6F7157`, 0x000897C0 adds one view per player with a current camera, `stream_raceline` uses that player's
  race vehicle; dump `research/core_stream_8a270.c`), and graphics' `update_track_streaming` uses every view: the
  "still single" note in graphics.md is stale.
- (original request)
- CarRender 0x001127F0 runs per view with the view's player and camera mode, and culls (+2251) the car that view's
  camera sits in — graphics now hides that car (body, wheels, lights, shadow, flares) per view. Please fill
  `visos_render::track_fx::HeadlightViews { player1, mirror }` every rendered frame next to `HeadlightView`:
  `player1` = `HeadlightView { camera_mode: mode(1), inside }` for player 1 in a two-player race (same rule as
  `headlight_view`, with player 1's follow / mirror camera and race car), else None; `mirror` = the mirror view's
  (camera mode 2 = player 0's mirror camera, VehicleEntity 0x001C57C0: `inside` = player 0's car when that camera
  is not an outside view, !0x00069880) while `MirrorView.on`, else None. Without them graphics assumes player 1
  outside every car and the mirror inside view 0's car.

### Requests from graphics (FuryFX / nitrous, 2026-11-06) — done
- **Nos sound**: `race_effects_bridge::nos_audio` (Update, like `zone_audio`) forwards every `FuryNosAudio` 1:1 as
  `AudioCommand::StartNos` / `StopNos` (logs "nos audio: ..." on a change only; StopNos repeats per frame).
- **Camera fury**: the original's four camera reads (0x00066940 at 0x00066C5E, Update 0x00067E54 / 0x00068801,
  static 0x000691D9) all call `FuryFX__128680(&byte_71A0D0, camera +0x244)` = the camera's **player** index (= the
  vehicle index, players come first). `update_cameras` now sets `t.fury = FuryFx::amount(p)` (0 without the
  resource) and `t.fury_fov = FuryFx.tweaks.fov_change` (`dword_686B5C`, the only reader is 0x000690B0; FuryFX
  0x001287B0 registers it as "Fov Change"), passed to `update_static`. The camera-side re-derivation
  (`fury_amount`, `GameCameras.fury_active`) is gone. The camera runs in GameTick and sees the state of the last
  rendered frame, as the original's physics-frame camera saw the last render's.
- **`dword_6F5B70`** (xrefs `research/core_fury_xr.txt`, disasm `research/core_fury_calls.txt`): writers are only
  ZoneRender 0x00162D10 (`fstp` at 0x00162D64 = 0x00154FC0(view player), per rendered view, before the level
  step 0x00154BD0) and FuryInstance 0x001297F0 / 0x001298E0 (zero) = graphics' `ZoneFx.amount_global`; readers
  PhysicsManager 0x0019F920 and VsConfigFileRead 0x001B82A0 (+ an unfunctioned copy at 0x001B802A). `race_cars::
  zone_fx_levels` (right before `physics_tick`) now writes `amount_global` into the physics glob 0x6F5B70 (direct
  `glob.w` insert; physics has no setter) next to the zone levels.
- Smoke `SMOKE_NOS=1`: holds B (CarNitrous) 3..6 s after Go, shot `2f_nos` at 5 s. Seen (`SMOKE_DIRECT=1 SMOKE_NOS=1
  SMOKE_DRIVE=10`, LADown, `research/core_nos_smoke.log`): StopNos from the start, StartNos at the press, StopNos
  after the release, nitrous blur + ribbons in `corenos_2f_nos`, exit 0.

(original request)
FuryFX 0x00128680 is ported in `visos_render::fury_fx` (graphics.md "Fury FX"); its state runs per rendered frame
from `RaceEffectsInit` / `RaceEffectsReset` / `RaceEffectsShutdown` and reads physics (VE+88, `flt_75B27C`, forward
speed) itself. Graphics already feeds it into the motion blur (0x0012FB60) and the zone gate (0x00154BD0), so
`MotionBlurInput.fury` / `ZoneFxInput.fury` can stay 0.
- **Nos sound**: forward `visos_render::fury_fx::FuryNosAudio { start }` (0x001293E0's VsAudio::StartNos / StopNos)
  as `AudioCommand::StartNos` / `StopNos`, like `ZoneFxAudio` (StopNos repeats every unpaused frame while the
  player's nitrous is off, as the original). Nothing sends them today.
- **Camera fury**: `game_cameras::fury_amount` re-derives 0x00128680 at the camera update; the original reads the
  instance state the render keeps. Please use `Res<visos_render::fury_fx::FuryFx>::amount(vehicle id)` (0 until
  InGameInit) instead, and the "Fov Change" tweak `FuryFx.tweaks.fov_change` (`dword_686B5C`) for `FOV_FURY`.
- **`dword_6F5B70`**: physics reads glob 0x6F5B70 (VsConfigFileRead 0x001B82A0 / PhysicsManager 0x0019F920, the
  zone time stretch) but nothing writes it. The value is `ZoneFx.amount_global` (ZoneRender 0x00162D64 writes
  0x00154FC0 of the view player before its level step; FuryFX 0x001297F0 / 0x001298E0 zero it) — please copy it into
  the physics glob each frame if that path is meant to run.
- Not surfaced (graphics assumes defaults): traffic vehicle +32 (`VsAIVehicle.active`, read by 0x00129910 through
  0x00017CF0; taken as true) and +33 (`flag_021`: 0x00129910 resets that car's beams and clears it; taken as false).

### Requests from ai (split screen, 2026-11-05) — done: `RaceProgress.player_positions` / `player_states` (Vec by player index) added

- `RaceProgress` holds only player 0's place / state. For player 1's HUD please add `player_positions: Vec<u32>`
  (RaceInfo.dPlayerPos by player index, 1-based) and `player_states: Vec<u32>` (0x00023CE0 per player); the ai will
  fill them in `fill_progress` next to `player_position` / `player_state` (it already keeps `VsRaceManager.
  player_positions` per player). Everything else the ai needs for two humans is in place (docs/ai.md "Split screen"):
  physics ids 0..n-1 = the players with pad slot = id, ai cars after them. Note: a front-end two-player race seats
  competitors − 2 ai cars (`byte_6F70D8` stays 1); only the split-screen jump (+0x204) has none.

### Requests from physics (jump cam rate, dummy cop tag, split screen) — done (rate: see "London ramps" / "System Update Rate"; dummies: "Requests from graphics (dummy police car, flashes)"; split screen: control modes / human control are set by the ai's binding, pads by `car_inputs`)
- `dword_68580C`: `PhysicsWorld::jump_cam_rate()` after each step (0x001C5800 eases it 0.005 -> 18.0 during the
  jump cam, player cars in control mode 0 only) feeds your clock (VsCommonMain 0x0016E17C); `set_jump_cam_rate(24.0)`
  at the countdown end (InGameUpdate 0x000B836B) and at FrontEndInit / InGameInit. physics.md "jump cam end".
- Dummy police cars: `PhysicsObjects` resource entries with `is_dummy_cop_car()` (record +56 == 1; LADown has 16).
- Split screen: physics is ready (physics.md "Split screen"): players = first physics ids, pad slot = player index,
  `set_control_mode(id, 0)` + `set_human_control(id, true)`, `PhysicsCarInput` from the second pad on slot 1.

### Requests from frontend (race points, split HUD, ContinueEvent) — 2026-11-06 — points done
- `RaceStatus` (and `PlayerRaceStatus[p]`) now carry `points` (race vehicle 0x000244E0 +84 = the score manager's
  `stats.score`; in the original the score manager writes the vehicle's block, ScoreManager +188 -> vehicle),
  `combo` (+152 `combo_score`), `multiplier` (+156 `combo_count`), `wrong_way_time` (+168), filled by the binary's
  `hud.rs::race_manager_status` from each player's race vehicle `score` (VsRaceManager 0x0003DF30 = +0x1C8); `feed`
  copies them into `RaceHudInput.status` (`wrong_way` = > 1.0). Per racer: `race_cars::RacerPoints(Vec<u32>)` = +84
  by place (index = place − 1, `race_order`), cleared on leaving InRace; getter `RacerPoints::points_in_place(place)`
  (1-based, 0 when empty). Note filed in frontend.md. Dump `research/core_osd_f99e0.c`.
- Speedo gauges per player (`RaceStatus.{nos, zone, turbo}`, `car_outputs` for every human race car, player 0 also
  fed into `RaceHudInput.speedo`): Speedo::GetRatioForGauge 0x000F4F40 on physics vehicle [p] — nos 0x00024720 = VE
  +96 / +100 (raw division as the original; 0 / 0 gives NaN, `f32::min` then shows a full gauge), zone / turbo:
  see "Requests from physics (speedo gauges ...)".
  `PlayerRaceStatus[1]` carries points / combo / multiplier / wrong-way / nos / zone / turbo for the split HUD.
- Gap (ai, request in ai.md): the ai's own record block `VsAIRaceVehicle.stats` ([u32; 25], +0x1CC) is never synced
  with the score manager's `RaceStats` (same memory in the original), so ai reads of `stats[21]` (the two prestige
  reads in `vs_ai_race_vehicle.rs`) and `stats[11]` (fastest lap, `racer_records`) miss the score manager's writes.
- Done on my side: `CareerRequest::ContinueEvent` -> `GameFlowRequest::StartCurrentEvent` (`lib.rs::career_out`);
  the split HUD reads `GameViews` + `PlayerRaceStatus[1]` itself (`hud::split_in`), nothing needed from `hud.rs`.
- **Race points (race vehicle +84)**: the OSD's points item (COSDInGameScreen 0x000F99E0, `OsdRaceStatus.points`)
  and the results OSD's race stats table (OSDManager 0x000F09C0: every racer's +84 as "prestige won", the player's
  into `dword_6F7244` = your `vars.event_state`, which RaceCompleteScreen 0x00205830 adds to the prestige) need the
  +84 points per race vehicle. Please add them to `RaceStatus` / `PlayerRaceStatus` (`points`, plus combo +152 /
  multiplier +156 / wrong way +168 of the driver if you have them) and to whatever per-racer list you keep for the
  results; I will then port the results OSD (`byte_6FD208`, steps 0x000F35C0) and drive your `results_busy` from it.
  Until then `event_state` stays 0 after a real race and the prestige never rises (seen in SMOKE_CAREER=3).

### Requests from frontend (results OSD drives `results_busy`, 2026-11-06) — smoke presses done
- Done: `smoke_test::run` presses A (one game update held, >= 1 s apart, `ready` = released) while the mode is InGame
  and `session.state == PostRace`, logging "smoke-test: results A at <t>", for every smoke mode (career3's
  `post_race` then handles the EGameMode::PostRace menu as before). `results_busy` / `post_race_step` untouched.
  Fix: the in-race key mute for attract / review / career3 (`smoke_test::run` end) swallowed those presses, so
  `SMOKE_CAREER=3` sat on the results OSD forever; it no longer applies in session PostRace. Seen (2026-11-06, log
  `research/core_career3.log`): 3 presses per race, End of Game -> post race menu -> ReturnedFromRace{saved}, all
  three events, prestige 0 -> 1091 -> 1298 (the reload shows 1091: prestige is added after the autosave), exit 0.
- The results OSD is ported (frontend.md "Results OSD"): `velocity_frontend::hud::results_update` runs 0x000F35C0 on
  `RaceSession.post_race_step` each GameTick before `InGameUpdateSet::UpdateSessions` and writes
  `RaceProgress.results_busy` (and `gm.vars.event_state` = the player's +84). Please keep `results_busy` untouched by
  everyone else (the ai already leaves it alone), and keep `post_race_step` reset on restart.
- **Smoke scripts**: mode 2 (race-over menu) now needs three select presses in session PostRace (step 1, then
  table page 2 -> 3 -> done), as in the original. `career3::post_race` and anything waiting for `EGameMode::PostRace`
  will stall without them: please press A (a release in between: the step needs an edge) while `session.state ==
  PostRace`. Stand-in for runs until then: `VELOCITY_RESULTS_AUTO=<s>` (frontend dev env) presses after `s` s.

### Requests from physics (shared rand stream for the jump cam roll, 2026-11-06) — done
- `race_cars::physics_tick` (the one live `run_physics_tick` call; the other two are in tests) hands `gm.rand.state`
  in with `set_rand_state` before the step and writes `rand_state()` back after it — only when the world existed
  before the step (a world loaded inside the step keeps its own seed and does not overwrite `gm.rand`).
- **Draws that happen later in the port** (inside the original's physics frame, so they interleave with physics'
  draws there; the port runs them from the events after the step, so the stream order differs): the audio impact
  pick (`VsAudio::PlayPhysicsImpact` 0x00052E90 -> `_AUDIO_RandomFloat` 0x002BB910, from `PhysicsObjectEvent::
  Impact`), the AI collision listener 0x000160B0, CarDamageFX 0x00078320 (not ported). Audio's pick uses its own
  `CrtRand` (velocity_audio `audio/system.rs`), not `gm.rand`, so it is also on a separate stream for now.
- (original request)
- Answer to "jump cam roll uses a private rand stream": `PhysicsWorld::rand_state() -> u32` / `set_rand_state(u32)`
  (velocity_physics sim_ai.rs). Every physics rand draw (jump cam roll 0x001CF070, 0x001CF090, 0x001A0530,
  0x001DFBB0) happens inside the step in the original's order, so in `race_cars.rs` around `run_physics_tick` (both
  call sites): lock `PhysicsWorldRes`, `set_rand_state(gm.rand.state)`, run the tick, then `gm.rand.state =
  rand_state()`. The jump cam hook draws nothing (0x001665E0/0x001665A0 reach no rand), so its code needs no stream.
  Details and the not-yet-ported in-frame draws (audio impact sound select, explosions): physics.md.

### Requests from graphics (player 1 zone blur focus, 2026-11-06) — done
- `race_effects_bridge::fill_motion_blur` builds `MotionBlurInput` per slot with one helper; in a two-player race
  `MotionBlurSecondInput(Some(..))` = the slot-1 race car's (same fields, `zone` 0), else None; cleared with
  `MotionBlurInput` on leaving InRace.
- (original request)
- In a two-player race fill `visos_render::zone_render_pc_xbox::MotionBlurSecondInput(Some(MotionBlurInput { position,
  velocity, wheels_down, .. }))` from the **slot-1** race car exactly as `fill_motion_blur` fills `MotionBlurInput` from
  slot 0 (only those three fields are read; `active`/speed are not needed); `None` (default) otherwise. Original: the zone
  FX 0x00162D10 → 0x00160BA0 → 0x00160920(view +28) focuses the blur on VehicleManager 0x001DDCC0(player) = the view
  player's car (graphics.md "Zone blur focus per view"). Without it player 1's view keeps player 0's last focus.

### Requests from physics (explosions, friction spots, race restart, 2026-11-06) — done
- `race_timeline_sequencer::PhysicsObjectResets` (the cursor collector also used for resets / moves) collects
  `TimelineObjectMessage::Explosion` 1:1 into `PhysicsExplosion` and counts `GameModeMessage::RaceRestart` (read in
  both `collect_object_resets` and `send_object_resets`, so a restart written in any frame survives to the next
  physics frame); `send_object_resets` (Physics chain, before `physics_tick`) writes `RestartPhysicsWorld` first,
  then the explosions, resets, moves. Both run inside the step, so the rand hand-in/out covers their draws. The
  explosion's tyrespray 0x00154B80(pos, 5, 1) is an empty method in F3_D (`mov [ebp-4], ecx; retn 0Ch`): nothing to
  wire. Puddles: see the speedo section below.
- (original request)
- **Explosions**: on `TimelineObjectMessage::Explosion { position, blast_radius, force_objects, angular_force_objects,
  force_vehicles, angular_force_vehicles, sector_angle, ignore_player }` write
  `velocity_physics::plugin::PhysicsExplosion(sim::Explosion { position, blast_radius, force_objects,
  angular_force_objects, force_vehicles, angular_force_vehicles, sector_angle, ignore_vehicle: ignore_player })` (1:1,
  = PhysicsManager 0x0019FD80 from TimelineExplosion 0x0017ACD0; physics runs it in the next tick's Setup inside
  `run_physics_tick`, so the rand hand-in/out around the tick covers its draws). The tyrespray 0x00154B80(pos, 5, 1)
  that follows it is empty in F3_D (see above). Or call `PhysicsWorld::explosion(&Explosion)` directly (then hand the rand
  state in/out around it).
- **Race restart**: `GameManager::RestartRace` (0x000BE126) calls MaterialProperties 0x0019C390: write
  `velocity_physics::plugin::RestartPhysicsWorld` there (= every object reset 0x001DE400 + friction spots removed +
  PhysicsManager +0x10/+0x5F08 = 1.0 + tracker owners cleared). It replaces a `ResetAllPhysicsObjects` you may send
  there today (RestartRace also clears `unk_720840`, graphics' puddle list, at 0x000BE20C).
- **Puddles**: forward `velocity_physics::plugin::PhysicsPuddleMessage(PhysicsPuddle { position, radius, direction,
  race_distance, kind, size, life, flags })` to graphics' `PuddleRender` (request in graphics.md).
  **Graphics side ready** (2026-11-06): write `visos_render::puddle_render::PuddleAdd((&msg.0).into())` for each
  message (LightStreak__133750: append, sort, RainSurface +18), and `visos_render::puddle_render::PuddleRestart` at
  `GameManager::RestartRace` (0x000BE20C → 0x00131EC0, drops the trailing runtime puddles), e.g. next to
  `CameraRainReset` in `race_effects_bridge`.
- **Race distance hook** (hydrant hit, PhysicsObjectManager 0x001E1F80 case 0x17): install
  `PhysicsWorld::set_race_distance_hook(Some(Arc<Mutex<dyn FnMut([f32; 3]) -> f32 + Send>>))` at race load, answering
  `VsAIRaceSection::GetRaceline(VsAIWorld::GetFirstRaceSection())` -> `VsAIRoadLine::FindClosestSegment(p)` ->
  `GetDistanceAlongLine(p, seg)` from the AI world (runs on the physics thread inside the step; must not lock the
  physics world). Without it the puddle's race distance is 0. `PhysicsObjectEvent::Explosion` keeps meaning the
  hydrant's `AddEnvSound` (unchanged).
  **ai (2026-11-06): installed by velocity_ai's `send_walls` once the physics world exists (docs/ai.md "Requests from
  physics (friction spots)"); core need not do it.** The ai also now draws from `GameManager.rand` (shared stream).

### Requests from physics (speedo gauges: nos names fixed, zone / turbo getters, 2026-11-06) — done
- `car_outputs`: nos = `nos_ratio`, zone = `zone_ratio`, turbo = 1.0 when `has_turbo == 0.0` else `turbo_ratio`
  (all f64 -> f32), per human race car (player 0 into `RaceStatus`, p ≥ 1 into `PlayerRaceStatus[p]`).
- Puddles done: `race_effects_bridge::forward_puddles` (GameTick physics chain, right after `physics_tick` /
  `record_object_notices`) writes `PuddleAdd((&msg.0).into())` per `PhysicsPuddleMessage`; `puddle_restart`
  (PostUpdate) writes `PuddleRestart` on `GameModeMessage::RaceRestart` (RestartRace 0x000BE20C).
- (original request)
- **The nos getters were swapped and are now fixed**: `PhysicsWorld::nos_remaining(id)` = VE +96 (m_NosRemaining,
  0x000248F0), `nos_capacity(id)` = VE +100. `race_cars.rs:866`'s `nos_capacity / nos_remaining` now computes
  +100 / +96 (inverted): replace it with `w.nos_ratio(id)` (= 0x00024720, +96 / +100 in x87, f64; store as f32).
- New (sim_ai.rs, checked against the XBE disasm of each inline): `zone_ratio(id) -> f64` = Speedo 0x000F78D0 (VE
  +104 / +108 fdiv), `zone_remaining` / `zone_capacity` (+104 / +108, frames), `has_turbo(id) -> f64` = 0x000F7970
  (+144 byte via fild, 0.0 / 1.0), `turbo_ratio(id) -> f64` = 0x000F79A0 (+148, written by PhysicsCarInput 0x001BA770).
  GetRatioForGauge 0x000F4F40: case 1 nos_ratio, 2 zone_ratio, 3 `has_turbo == 0.0` -> its no-turbo branch, else
  turbo_ratio. The two "Remaining<=Capacity" asserts are not kept (0/0 = NaN as the original).

### Requests from ai (crash camera respawn, 2026-11-06)
- VsCrashCamera 0x00062D90 calls VsAIRaceVehicle 0x000208D0(crashed car, `flt_6850F0`) when `+20` (failure) or
  `byte_6850E8 && flt_6850EC >= 0x00015C60(car)` (the race vehicle's speed); `vs_crash_camera::stop` emits
  `CrashCameraEvent::Respawn` but `game_cameras::apply_crash_events` drops it (`_ => {}`), so crashed cars are never
  put back. Please forward it as `velocity_ai::plugin::AiRespawnOnRaceline { vehicle, speed: kick_speed }` (e.g. via
  the bin crate, like `AiCinematicBars`); the ai applies it at the start of its Ai set (crash cam runs before).
- FYI: the ai now installs physics' race distance hook itself and draws from `GameManager.rand` (above).
- **Done (crash camera respawn)**: `apply_crash_events` turns `CrashCameraEvent::Respawn { vehicle, kick_speed }` into
  core's `game_cameras::CrashCameraRespawn { vehicle, speed }` (both call sites: CrashCam set update and the ai's
  crash camera reset call); the binary's `ai_host::crash_respawn` (GameTick, after CrashCam, before Ai) forwards it
  as `AiRespawnOnRaceline`. A respawn from the ai's reset call (written after the Ai set) reaches the ai next update.
  Race distance hook: nothing to do in core (the ai installs it).

### Requests from frontend (debug-menu tweaks, 2026-11-06) — done (title note dropped, split time read from the tree via `ai_host::score_tweaks`; System order not done, see "Debug menu")
- The frontend's nine AddTweaks blocks and the callback 0x00202EA0 are in (`velocity_frontend::fe_tweaks`, docs/frontend.md
  "Debug-menu tweaks"); remove them from "AddTweaks registrations still unported". The callback now applies "Scan Track
  Folder" (0x00094990 on `gm.event_db` and the FE copy), "Unlock ALL Events" and the prestige re-evaluation.
- **Title y**: not a mapping bug. 0x000EE5F0 sets y = 500 for flags 0x1000, and `COSDManager::Update` runs on the menu
  manager `dword_6FD1FC` every frame (0x000BB637 / 0x000BB7DC / 0x000B7FA8), so the original's debug menu title sits at
  mid screen over the rows too. Please drop the "open question" note.
- **Split Time Display Duration** (`dword_6860EC`, "Game.OSD", 2.0): `ScoreManager::lap_positions` (0x0016BE50) hard codes
  `ahead_shown = 2.0`; read the live value from the tree instead, e.g. `tm.find_system("Game.OSD")` +
  `find_tweak(s, "Split Time Display Duration")` -> `tweak(id).cell.f32()` (2.0 while unregistered).
- **System order (optional)**: SetGameMode 0x000BBD30 adds "System" (FontHelper 0x000B3800) *after* the mode's init, so
  in the original the root lists System first after a mode change; core adds it at the reset, the frontend registers a
  frame later, so System sorts last. A registration hook run by the TweaksReset handler before `add_system_tweaks`
  (e.g. a `TweakRegistrars` resource of `fn(&mut CTweakManager, mode)` in velocity_game) would make it 1:1.

### Requests from graphics (graphics AddTweaks into the debug menu, 2026-11-06) — done (`core_tweaks::GFX_SITES`, see "Debug menu")
visos_render cannot see `CTweakManager`, so its 21 registrations are recorded call lists:
`visos_render::gfx_tweaks::GraphicsTweaks` (resource, from `RenderPlugin`) — `calls(origin) -> &[GfxTweakCall]`
(`System(path)` = createSystem, then `Float{name, slot, min, max, step}` / `Int{..}` / `Enum{name, slot, elements}` /
`Bool{name, slot}`), `values: Vec<GfxValue>` (one slot per tweak, `Float/Int/Bool`), `PHASES` / `phase_origins(phase)`
= where the original calls them (CommonInit, FrontEndInit, InGameInit with call-site addresses; nested ones listed in
call order; Spec at InGameInit is called from GameManager 0x000B4170 at 0x000B4BD1 among core's own registrations;
CarDamage 0x0006B8B0 per police car from VsAIPoliceVehicle 0x00018000). `sync_graphics_tweaks` (First) already pushes
changed slots into the live render resources and pulls live values back every frame. Please:
- replay at those call sites (after a TweaksReset, the mode's init re-adds them as the original does):
  `let mut s = 0; for c in gt.calls(origin).unwrap() { match c { System(p) => s = tm.create_system(p),
  Float{name, slot, min, max, step} => { tm.add_tweak_f32(s, name, cell(slot), *min, *max, *step); } Int{..} =>
  add_tweak_i32, Enum{..} => add_tweak_enum, Bool{..} => add_tweak_bool } }` with `cell(slot)` = one `TweakCell`
  per slot kept in a core resource, created from `gt.values[slot]` at registration (that value is the default the
  menu shows, as the original's). The BarrierRender pair (0x000B6D2E / 0x000B6D48) registers twice: `insert`
  dedups, fine.
- around the debug menu (as `system_tweaks_in/out`): before, `cell.set(gt.values[slot])`; after,
  `gt.values[slot] = cell` (Float/Int/Bool → `GfxValue`) — only for registered slots.
- then a smoke variant (`SMOKE_DEBUG_MENU`) that opens a graphics system (e.g. Tweaks → Graphics → 3DLightBeams,
  RIGHT on Extent) so graphics can take the confirming screenshot.

### Requests from ai (debug-menu tweaks, 2026-11-06) — done (`SMOKE_DEBUG_MENU_PATH` / `_SHOT`, see "Debug menu")
- The ai's AddTweaks are in (`velocity_ai::ai_tweaks`, docs/ai.md "Debug-menu tweaks"): VsAIWorld::AddTweaks 0x0002D360
  ("AI.*", 25 systems / 151 tweaks, at the AiWorldData step) and VsAIPlayerTFTManager 0x00016C30 ("TFT", TftManager
  step), registered directly on `GameManager.tweaks`; remove both from "AddTweaks registrations still unported".
- **Smoke path**: `SMOKE_DEBUG_MENU` always enters the root's first child; after a race load that is the frontend's
  newest in-game system ("Game"), so no other subsystem can reach its own set. Please make the walk configurable, e.g.
  `SMOKE_DEBUG_MENU_PATH=AI.Nitrous` (find each name among the rows and press DOWN until the cursor is on it, then A;
  default = today's walk), shot name from `SMOKE_DEBUG_MENU_SHOT` (default `debug_menu.png`). The ai's confirming
  screenshot (`AI` → RIGHT on Difficulty Scale, expect "tweak AI.Difficulty Scale = 0.65") waits on this; graphics'
  request above needs the same.

### Requests from audio (audio AddTweaks into the debug menu, 2026-11-06) — done (see "Debug menu"; DopplerFactor → 0.4 seen)
velocity_audio cannot see `CTweakManager` (velocity_game depends on it), so its 7 registrations (12 systems, 36 tweaks)
are recorded call lists in graphics' format: `velocity_audio::audio_tweaks::AudioTweaks` (resource, from `AudioPlugin`),
`calls(origin) -> &[AudioTweakCall]` (`System(path)` / `Float{name, slot, min, max, step}` / `Int{..}` / `Bool{name,
slot}`; no enums), `values: Vec<AudioTweakValue>` (`Float/Int/Bool`), `sync_audio_tweaks` (First) already pushes edited
slots into `VisosAudio.game` and pulls live values back. Please:
- replay `for o in audio_tweaks::ADD_TWEAKS_ORDER { calls(o) }` (Impact, Debug, Music, Vehics, Engine, Slowdown, EnvFx)
  where core sends the **in-game** `AudioCommand::Init` (game_manager.rs, the race Init): the original is
  GameManager::CommonInit 0x000B5169 → VsAudio::Init → AddTweaks 0x000484D0 at 0x00048362 (after LoadEnvAudio), which
  returns at once for the front-end Init (VsAudio +104), so nothing is registered in the FE. One `TweakCell` per slot
  created from `values[slot]` at registration (the default the menu shows).
- around the debug menu as for graphics: before, `cell.set(values[slot])`; after, `values[slot] = cell`.
- confirming smoke (with `SMOKE_DEBUG_MENU_PATH`): Tweaks → Audio → ENVFX, RIGHT on DopplerFactor, expect
  "tweak Audio.ENVFX.DopplerFactor = 0.4"; shot for audio.

### Requests from physics (physics AddTweaks into the debug menu, 2026-11-06) — done (see "Debug menu"; Turbo.Rise rate → 0.2 seen, log `research/core_debug_menu_phys.log`)
velocity_physics cannot see `CTweakManager`, so its 3 registrations (37 systems, 113 tweaks) are recorded call lists in
graphics' format: `velocity_physics::physics_tweaks::PhysicsTweaks` (resource, from `PhysicsPlugin`), `calls(origin) ->
&[PhysTweakCall]` (`System(String)` / `Float{name, slot, min, max, step}` / `Int{..}` / `Bool{name, slot}`; no enums),
`values: Vec<PhysTweakValue>` (`Float/Int/Bool`), `PHASES` / `phase_origins(PhysTweakPhase::InGameInit)`.
`sync_physics_tweaks` (First) already writes edited slots into the globals (current world + process-wide, new worlds
are seeded) and pulls game writes back. Please:
- replay in InGameInit at the original call sites: 0x000B6251 → `calls(0x0019BEC0)` (CConfig::AddMaterialTweaks,
  "Materials.*"), 0x000B679E (PhysicsManager::Init) → `calls(0x001B8A00)` then `calls(0x001DC650)` (JumpCam, Network,
  Handling*, EngineSequencer, PhysicsObjects, Turbo, StartLineBehaviour, SkidVibrate, then "Debug Cameras", which core's
  GameManager 0x000B4170 also creates: same system). One `TweakCell` per slot from `values[slot]` at registration.
- around the debug menu as for graphics: before, `cell.set(values[slot])`; after, `values[slot] = cell`.
- "Debug Cameras.Follow vehicle" (`dword_688B80`, -1..6) is read by VehicleManager SyncModels 0x001DDAE0 / 0x001DDBC0
  (which car the debug camera follows): if core ports that, read `values[pt.slot_of(0x688B80)]`.
- confirming smoke: Tweaks → Turbo (or Materials → GRASS), RIGHT on "Rise rate", expect "tweak Turbo.Rise rate = 0.2";
  shot for physics. Remove the physics line from "AddTweaks registrations still unported".

### Requests from physics (Handling.cfg camera overrides, 2026-11-06)
- Answer to your 0x6850A0/68519C/6851A4/6851B4 request: `PhysicsWorld::handling_camera_overrides()` (lock
  `PhysicsWorldRes`) returns `velocity_physics::vehicle::vehicle_entity::sim::HandlingCameraOverrides`, one
  `Option<f32>` per global VehicleEntity 0x001C9070 writes from Handling.cfg (only when the cfg value != 0):
  `far_clip_distance` → `manager.clip.far`, `yaw_rate_per_degree` → `globals.yaw_gain`, `yaw_relax_angle` →
  `globals.yaw_full`, `yaw_direction_blend` → `globals.heading_weight`, plus the wheel camera `wheel_cam_fov` /
  `wheel_cam_offset_x` / `wheel_cam_offset_z` (0x68768C / 0x687690 / 0x687698). None = not written, keep yours.
- The write happens when a car is spawned (`spawn` runs 0x001C9070), so apply after the race's cars spawn (once, or
  when the value changes), not every tick, or debug-menu edits of those globals get overwritten. Shipped cfg:
  far clip None, yaw 200 / 15 / 0.6 (= the .data defaults).
- Done: `race_cars.rs` applies far clip / yaw_gain / yaw_full / heading_weight once after a batch of spawns (Some only).
  Wheel-camera fields (fov, offset x/z) not applied: no live global for them yet. Golden camera test, check and headless
  smoke (exit 0, `research/core_hcam.log`) green. "System" tweak set still re-added right after the new root
  (`TweaksReset`), not via a `TweakRegistrars` hook; ordering vs mode registrars left as is.

### IDB rebuild (2026-11-07, tooling)
- `research/default.xbe.i64` was rebuilt from `default.xbe` by `tools/ida/rebuild_idb.py` (re-runnable; `--harvest-only`
  writes the plan `research/ida_rebuild_names.tsv` without IDA). The damaged idb's section headers were zeroed but its
  id0 b-tree was intact: `tools/ida/salvage_names.py` recovered 38698 user names (`research/ida_salvaged_names.tsv`).
- Applied: 36720 salvaged, 1089 from `/// File.cpp 0x…` citations (Rust `Type::fn` -> `Type::PascalFn`), 268 from
  rename tsvs, 2 stems. Disagreements (224) in `research/ida_rebuild_conflicts.txt`; salvaged spelling was kept over
  citations. Lost: types/prototypes, comments, structs set in the old idb (only names were salvaged).
- Backup `research/default.xbe.i64.bak`.

## Coverage audit (2026-11-07)

Tools: `tools/coverage/cov.py core [names.tsv]` (regions x funcmap x `/// File.cpp 0x...` origin lines in `visos`,
`velocity_game`, `velocity`), `tools/ida/cov_names.py` (names/flags), `tools/coverage/tag.py core`; output
`research/core_coverage.txt`. Scope = core source files (GameMode, GameDataLoading, EventDatabase, TimelineManager,
Triggers, Camera, Replays, ScoreManager, GameBridge, Cli, Message, Controller, GameInput, main, VsCommonMain, visos
Memory/File/Handy/Timer/Instances/Compartment/Type/Chunk/Timeline, Input*).
- First audit: 446/3939 functions, 197886/579262 bytes (34.2%), gameplay bucket 747/130976.
- **Reconciled (2026-11-07)**: 77 bare `/// 0x…` / inline `// 0x…` lines on genuinely ported functions got their
  file name (follow camera Update 0x67510 / static 0x690B0 / soft limit / zone ramp / trail heading, the POI's
  begin/start/end/finish/speed/fov, VsCatmullRom 0x614F0/0x61620/0x61710, TimelineSequencer Play/Stop/SetTime/
  Update/UpdateTrack/UpdatePath/ease/splines (the idb labels 0x2B4xxx "VsStringAnsi"), file_tlf fixups, VsTextFile
  line values, bridge swaps, inline-ported callees in trail_blazer / race_session / replay / post race / update
  thread). Left `mentioned` on purpose: functions only referenced (CCinematicCameraSet 0x603C0/0x60CE0, GameManager
  0xB8550 / LoadingTimer / Render, MessageManager add/remove, vibration, glue ranges, partial ports such as ReleaseCars
  0xBD9B0 without the start boost, camera collision 0x692C0 without its line check). Not in scope, still bare: the
  save flow nodes (`save_flow_save_m.rs`, SaveGame is not a core-scope dir).
- **Tool changes**: `cov.py core` also credits origin lines in the other owners' crates (leading address list only;
  own crates keep "every address on the line") — 69 core-scope functions are ported there (frontend
  EventDatabaseTrackLoading / LoadingBar / UserInput pieces, ai EventDatabaseEvents / ScoreManagerChecks, graphics
  GameDataLoading incl. **GDL_RelocateModel 0x9EF30**: graphics reads xmf structures without relocation and ports its
  shader overrides in `model_builder.rs`). `tag.py` splits the old gameplay bucket with `research/core_func_sig.tsv`
  (`tools/ida/core_cov_sig.py`): **network** = ≥ half of the non-trivial calls go into `src\Network` (minus
  NetworkLogger, which local timeline code calls) / Online code, the name or strings name a net class, or every caller
  is net (fixed point); **template-dup** = same mnemonic sequence with operands masked to their types (≥ 48 bytes) as
  a ported function or a lower-addressed unported one. Both are heuristics.
- **Now**: ported 568/3939 functions, 240506/579262 bytes (**41.5%**). Unported: platform 1641/162977; gameplay
  396/66223; ctor-dtor 302/40835; getter 706/20232; debug 51/17696; template-dup 207/16320; network (cut online
  mode) 66/13824; mis-attributed 2/649.
- Findings: **TriggerUpdate 0x1813C0 is a debug draw** (8 box corners through the trigger matrix, 12 lines via
  FontManager 0x2B0AC0; gated by `byte_74B458` in TimelineTrigger 0x17F620 and VsAIWorld's debug render 0x35610) —
  renamed `TriggerUpdate_DebugRenderTrigger`, not ported. **GamePlayer 0xC1480 = the CFEGameVariables constructor**
  (zero fill, default setups) = `CFEGameVariables::new` (Default = new + SetInitialVariables).
- Ported this stage: `timeline_sequencer::FlagGirlSequencer::{bind_anims 0x176640, load_models 0x175870, find_anim
  0x174930}` (+ `TimelineModelLoads`, `model_file_name` = LoadModelGeometry 0x9FE10's name rule, called in
  `FlagGirlIntro::new` as 0x9CB30 / 0xBA920 do; the geometry itself stays graphics' `SpawnSkinnedModel`);
  `game_data_loading::gdl_lights::{load_lights 0xA0B40, load_point_lights_bin 0x9DA10, load_point_lights_txt 0x9DE90,
  light_name_ids 0x9D860}` (returns `GdlLights`; graphics still parses lights itself, request in graphics.md).
- **Flag girl model names**: 0x176490 stores the upper-cased instance link name and LoadModelGeometry loads
  `<name>.xmf` through VsFileDirectoryDetails::LoadFile (no-case crc, no stripping). Shipped link names carry no digit
  suffix (`LON_FLAGGIRL_A`, `SUICIDE_FG_01_A`, …): test `flag_girl_model_loads` finds 12/12 models and 26/26 anims
  over the 10 intros, so graphics' `flag_girl_model` digit strip is a no-op on shipped data.
- Verified: `cargo test -p velocity_game` green (79 lib); `SMOKE_DIRECT=1` LADown race exit 0, drove 10 s, no
  "Missing timeline model" / "Failed to load" (log `research/core_cov_race.log`).
- Top unported gameplay after the first audit: 0x5E220 VsBaseCamera, 0x9AED0 BatchLoader, 0xBEBB0 movie path, the
  RaceTimelineSequencer loads, 0x603C0 / 0x60CE0 cinematic camera set (unreferenced), MessageManager 0xCA4C0 / 0xCAE80,
  LoadingTimer 0xB8C10, LoadModelGeometryAsynch 0xA0520 (triage below).

### Coverage stage 2 (2026-11-07): race timeline loads, seek fix, triage
- **Before / after**: 240506 → 244577 bytes (41.5% → 42.2%), 568 → 577 functions; gameplay bucket 66223 → 58647
  (incl. 4 retags to debug). Dumps `research/core_s6/*.c`; IDA renames in `tools/ida/core_s6_ren.py` (saved).
- **Ported** (`race_timeline_sequencer.rs`): `load_animations` = RaceTimelineData::LoadAnimations 0x171B90 (channel
  +0x0C bone anim / +0x10 vertex anim names; "undefined" cleared (stricmp), vertex anims logged + cleared;
  GDL_LoadAnim 0xA0080 `<name>.gaf` from `\ASP\ASP.vcd`; records `{VsStringAnsi::Hash 0x2B46B0 (case-sensitive crc,
  0 for ""), file}` qsorted by hash 0x171B30), `load_models` = 0x171F20 (instance links of type crc32("Model"),
  LoadModelGeometry from ASP.vcd, ASP pushed as the texture dir), `find_animation` 0x172260 (sorted search, **a miss
  returns the first animation**: peer +0x18 0x16FEB0 uses it), `find_model` 0x172320 (only TimelineSkinnedModel
  0x17E0C0 reads it, never made), `unload` = 0x170E50 (EndOfGame 0xBE5CC → 0xBAC30; on `InGameShutdown` in
  `mode_messages`, now chained before `core_load_steps`). Called from the LoadTimelineData step in 0x170570 order
  (anims, models, then the binds). Test `race_timeline_anim_and_model_loads` (every shipped timeline.bun).
- `TimelineObjects::bind` = peer BindToTarget 0x16F860 (type checked first: an unknown type never reuses an object);
  **camera shaker attach moved to `post_bind`** = 0x170570's vtable +0x10 pass after all binds and the sort (the shaker
  vtable 0x64115C has 0x179E70 at +0x10; attaching at bind time missed cameras bound later).
- visos `seek` 0x2B5E70 now calls `seek_typed` = 0x2B6040: playing → begin; the end call for the previous item tests its
  kind == 1 (bone anim), not 4 (original bug kept: never fires on single-kind tracks), also stopped-after-playing.
- **Triage (not ported, why)**: 0x5E220 = frustum debug draw (12 FontManager lines, `byte_6D55C2`) → renamed
  `VsBaseCamera::DebugRenderFrustum`; 0x9AED0 = an STL merge/sort helper (iterator calls only, template); 0xBEBB0 movie
  path is ported by the frontend (`velocity_frontend::movie`, its citation lacks the `GameManager.cpp` form so cov.py
  does not credit it); MessageManager 0xCA4C0 / 0xCAE80 = net vehicle state pack / jitter (net__ calls, cut online);
  0xBAF40 font test, 0xBF770 / 0xBF910 load-timing CSV → renamed Debug*; 0x2B5320 bone/vertex blend tree setup
  (graphics); 0x2B6E90 / 0x2B7660 = matrix→quat / quat mul helpers used by InGameUpdate, Renderer, VehicleEntity (port
  where those callers are); 0xA0220 async anim load, 0x9ED30 model texture fixup (render), 0x8FA80 EventDatabase free,
  0x1E6470 UserInput teardown, 0x182070 / 0x93D60 / 0xC2550 ctors, 0x1650C0 fixed-cam debug draw, 0xB5380 platform init. Next real candidates: 0x164D70 / 0x1674E0 fixed / orbit replay
  camera bridge writes (cut → both bridge buffers; check against `write_camera_bridge`'s replay path). The rest of the
  gameplay bucket is mostly such teardown / container / ctor code under heuristic tags.
- Shipped data: race timelines reference 8 bone anims (7 load; LADown circuit Race04 `BB_Fall` is in no pack, the
  original logs "Failed to load anim" too) and no "Model" links. Verified: `cargo test -p velocity_game` 80 lib green
  (golden chase camera unchanged), visos lib green; `SMOKE_DIRECT=1` LADown race exit 0, same log errors as before
  (`research/core_s6/race.log`).

### Requests from graphics (cat's eyes, 2026-11-07)
- GameDataLoading::LoadGameData "Load catseyes" (0x0009D055, after the compartment index) calls CatsEyeRender
  0x0011CDB0 with the race directory: please write `visos_render::cats_eye_render::LoadCatsEyes { race_dir }` there
  (race dir **with** its trailing `\`, e.g. `\Tracks\LADown\Circuit\Race02\`; graphics appends `CatsEyes.ced`; only
  LADown Circuit Race02 / Race09 ship one, a missing file is fine), and `UnloadCatsEyes` at race shutdown. The draw
  (0x0011CF70, render flag 0x100 views) is graphics' own and uses the race's player car (lowest race `CarRender.request`).
- **Done** (`race_cars::race_load_steps`): the `ELoadStep::CatsEye` step ("Load catseyes", in the original step order)
  writes `LoadCatsEyes { race_dir }` (from the step's `RacePaths`) and logs `Load catseyes: <dir>CatsEyes.ced`;
  `despawn_race` writes `UnloadCatsEyes` next to `UnloadTrackRender` (not original: F3_D only replaces the pointer on
  the next load). Messages registered by core too, so tests without the graphics plugin run.

### Coverage stage 3 (2026-11-07): replay camera bridge writes, triage
- `cov.py core`: 577 → 580 functions, 244577 → 245496 bytes (42.2% → 42.4%); gameplay bucket 58647 → 58023.
- Verified: `cargo test -p velocity_game` 80 lib green (golden chase camera unchanged); `SMOKE_DIRECT=1 SMOKE_RACE=2`
  (LADown Circuit Race02) exit 0, log `Load catseyes: \Tracks\LADOWN\CIRCUIT\RACE02\CatsEyes.ced` and no graphics
  "failed to load" for it (`research/core_s7/race.log`; only the known particle-type errors).
- **Replay camera bridge writes** ported 1:1 (`replay_camera_manager::{fixed_bridge, orbit_bridge, wheel_bridge}`):
  ReplayCameraFixed::WriteBridge 0x164D70 (loaded +0x64, camera +0x58, aim 0x163E80 → eye *(+0x58), target +0x34,
  up +0x44, fov +0x54), ReplayCameraOrbit::WriteBridge 0x1674E0 (+0x5C; eye +0x24, centre +0x04, up +0x34, fov
  `dword_687688`), ReplayCameraWheel::WriteBridge 0x167D60 (+0x40; +0x0C/+0x1C/+0x2C, `dword_68768C`). Their cut flag
  (+0x0C / +0x5E / +0x09) does **not** set the GameBridge renderer cut (+0x14): it calls CameraBridge::SwapViews
  0x969B0 (flips views 0 **and** 1 write index), writes the same record, swaps back, i.e. the record lands in both
  buffers of view 0 so the lerp has nothing to cross. Port: `CReplayCameraManager.bridge_both` → `write_camera_bridge`
  copies the replay record into `GameBridge::camera_previous(0)`; `ReplayEvent::Cut` is gone (it used to set
  `render_cut`, which also cut the car/timeline lerp — not original).
- Triage (not ported): 0xA3DD0 = empty RTC frame (no body); 0x169BF0 = ScoreManager::CreateScoreOsdItem (COSDManager
  string items for score messages: frontend OSD); 0x9FC90 = GDL_RelocateAnim (`GAF` pointer fixups: graphics' anim
  loader reads unrelocated); 0x1E7A20 = UserInput binding-clash count (STL iteration); 0x2E9040 / 0x9C3D0 / 0x99BE0 =
  STL tree / rotate templates. IDA renames `tools/ida/core_s7_ren.py` (saved, backup `.bak-20261107`).

### Coverage stage 4 (2026-11-07): rumble (VibrateEffects), camera view write-back, triage
- **Rumble ported** (`controller::vibrate_effects`, `add_vibration` from `race_cars::plugin`): `VibrateEffects` per pad
  (`unk_75C1A0 + 100 * pad`: 8 x {preset, strength, time}, count) = Add 0x1ED6D0 (ignored at 8), AddOrUpdate 0x1ED760,
  UpdateExisting 0x1ED7E0 (time < 0 = preset duration), Clear 0x1ED460, Tick 0x1ED480 (motor = max over
  `intensity * strength * time / duration`, ages by dt, swap-removes finished); presets = `off_688C10` rows (name, motor
  0 light / 1 heavy, intensity, duration — `core_tweaks::VIBRATE_EFFECTS`; the debug-menu tweak copies do not feed
  `VibratePresets` yet, a `live!` closure cannot capture the row index). Triggers: collision handler 0x1ED2D0
  (`PhysicsCollision.impact` >= 1 -> small <= 6, medium <= 15, large, `Add` on both players' pads), per update
  0x1ED8E0 + 0x1EDD70 in `InGameUpdateSet::Effects` after the zone trail (VE+68 > 0.1 slipstream, VE+88 nitrous, VE+89
  zone and not a trailblazer; `dt` = scaled dt), skid 0x1D1680 / 0x1D1770 (no flagged wheel with FastCar 0x364A90 == 1,
  speed > 1, contacts; ramps over 1.5 s = `dword_688A9C`; its VE+56/+60 state is kept in `Vibration`), trailblazer
  effect 7 (`trail_blazer::apply_trail_blazer_calls`: AddOrUpdate(7, 1) then UpdateExisting(7, -1, sqrt level)),
  `Vibration::stop` = 0x1EDD20 (race init, `InGameShutdown`, `RaceRestart`, post race fade start). The pad = the
  first device of `GameInput::device_mask(player)`; rumble runs while not attract (0x1ED880), `Career.options.rumble`
  (`byte_6F9182`) and not paused (port: motors zeroed on pause). Output = `visos::input::input_manager::set_rumble(port,
  motor, v)` -> `SDL_RumbleGamepad` (light = high-frequency motor, heavy = low-frequency, 100 ms lease) sent at each
  `InputManager::update`. Tests `effects_tick_and_expire`, `skid_ramps_and_collisions_pick_presets`.
- **Camera view write-back** `VsCameraManager::write_back_view` = 0x5F800 (Update's follow-mode branch, player 0): the live
  tweaked view (height, interest height, dist a / b, fov, near) goes back into the car's table row of the current view
  ([2], [0], [4], [5], [11], [17]), so a later view change restores the tweaked values.
- **Already done, only uncredited**: camera collision 0x692C0 (`update_follow`'s `hit` callback = `world.line_check`
  from the car point + 1 m; eye = from + (eye - from) * fraction, `game_cameras.rs`).
- **Not ported (why)**: ReleaseCars start boost `VehicleEntity__1D1270` = physics (request in physics.md: needs the OSD launch
  gauge float `OSD3dInterface+2936` and `PhysicsWorld::launch_boost`); 0x5E0A0 / 0x5DF70 = frustum-box tests (graphics'
  `compartment_manager` has 0x5E0A0); 0x5F6D0 / 0x5F460 = camera teardown; GameInput 0xA6320 / 0xA5CA0 / 0xA5BA0 = net pad
  records; UserInput 0x1E7560 / 0x1E68E0 = directory scan / STL walk; VibrateEffects 0x1EE2xx-0x1EEFxx = XDK
  (XGetSongInfo, FileTimeToSystemTime) mis-attributed; GameManager 0xB8C10 LoadingTimer (debug timing CSV + loading
  bar draw), 0xBA7D0 / 0xB4EB0 / 0xC07B0 / 0xB8F50 teardown / ctor / timing dump. Dumps `research/core_s8/*`.
