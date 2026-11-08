# Project Velocity — Rust/Bevy rewrite

Decomp-style rewrite of **Project Velocity** (Visual Sciences, Dundee; Sierra; Xbox, cancelled 2005), in the
spirit of iw4L (`D:\projects\iw4L`). Internal codename **F3**, source root
`d:\source\F3\MainBranch\Velocity\Code\Common`, engine **ViSOS** (`TNT\Version1\visos`), XDK **Dec 2003**.
Goal: behaviour 1:1 with the original, function by function, reading the user's own game data at runtime.

## Deployment target (user requirement)

The shipped artifact is **one executable that runs straight from the original raw game files**: drop it into
the game folder (next to `default.xbe`), or point it at one with `--game <dir>`. Resolution order:
`--game` arg → `VELOCITY_GAME_DIR` → the exe's own directory if it contains `default.xbe` → error naming all three.
- No preprocessing, extraction or conversion step, no build-time asset baking, no required cache. Every format is
  parsed at runtime from the install exactly as the original does. A cache is allowed only as a transparent speedup
  that is rebuilt automatically.
- Never write into the game dir (it may be read-only); logs/saves go under the exe dir or the user profile.
- Assets are not loaded through Bevy's `AssetServer` file paths into `assets/` — readers go through `visos::file`
  (game-dir resolution, VCD packs, loose-folder override) and build Bevy assets in memory.

**Input = SDL3 (user requirement).** The engine input layer (`visos` InputManager/InputDevice/InputBinding/
InputMapping, `Input\Xboxpad\Config1.ini`) is ported on top of SDL3 (`sdl3` crate, statically linked — the exe
stays standalone), not Bevy's gilrs input. Gamepads via SDL_Gamepad (Xbox pad layout = the original's), keyboard
via the original KEYMAP.INI mapping. Steam hides pads from an in-game SDL: override
`SDL_GAMECONTROLLER_IGNORE_DEVICES`/`_EXCEPT` to "" and set `SDL_GAMECONTROLLER_ALLOW_STEAM_VIRTUAL_GAMEPAD`=0.
Every consumer (frontend, game input, debug menu) reads input through this layer only.

**Unlock launch option (user requirement, TODO):** the original debug exe boots into a mode with everything
unlocked (cars, races incl. the `Available: NO` cruises, etc.). Reproduce that exact mode behind `-u` / `--unlock`
(trace what the original sets — e.g. the jump/debug path that the slice test currently fakes — and default to the
locked retail behaviour without the flag). Not started yet.

**Bink movies (user decision):** skipped for now — movie screens run their flow but show nothing. Proper
implementation later (planned route: a Rust port of a Bink 1 decoder; mind that FFmpeg's `bink.c` is LGPL).

## Game data and binaries

Game dir: `D:\games\xbox\games\project velocity debug` (read-only; override with `VELOCITY_GAME_DIR`).
- `default.xbe` = **F3_D.exe, Xbox Debug** (7.3 MB) — the analysis target: asserts carry `__FILE__`/`__LINE__`
  and the expression text, `/RTC` is on (`memset(…, 0xCC, …)` prologues), no inlining.
- `F3_R.xbe` = Release, `F3_M.xbe` / `unknown.xbe` = Master. Use them only to confirm what survives optimisation.
- `autoexec.cli` is the debug console script the game runs at boot (`debugMenu.Add …`, `tweak.save …`).
- Data is mostly plaintext (`*.txt`, `*.ini`, `*.cfg`) plus `*.VCD` containers, `*.BIN` databases, `.xva/.xvg` audio,
  `.bik` movies, `.wma` music.

## IDA (headless, via MCP)

- IDB: `research/default.xbe.i64` (IDA 9.5, built-in `xbe.dll` loader). Never commit it.
- Server: `.venv-ida/Scripts/idalib-mcp.exe --unsafe --host 127.0.0.1 --port 8745 research/default.xbe.i64`
  (ida-pro-mcp, idalib backend, Python 3.14 venv). Check it is up with `netstat -ano | grep 8745` before
  starting another — **one worker owns the IDB; never start a second server or open it in another IDA**.
- Client: `.venv-ida/Scripts/python tools/mcp.py <tool> '<json>'` (database id is filled in automatically),
  `tools/mcp.py list`, `tools/mcp.py schema <tool>`.
- Scripts: write an IDAPython file with the **Write tool** under `tools/ida/` and run
  `tools/mcp.py py tools/ida/<file>.py` (uses `py_exec_file`). Do not use `py_eval` or bash heredocs for code
  containing backslashes — both mangle `\\`.
- All agents share the one worker: keep scripts targeted (no whole-database sweeps unless needed), and run
  `tools/mcp.py idb_save '{}'` after a batch of renames. Never call `idb_close`.

### What the debug XBE gives you

- **Object-file layout is intact.** Game code is `0x11000–0x3DDD20`; each `.cpp` occupies one contiguous range.
  `research/objregions.tsv` = `start  end  source-file` (299 regions); `research/funcmap.tsv` = per-function
  attribution; `research/srcfiles.txt` = all 591 source paths. Unnamed functions in a region are pre-named
  `<FileStem>__<ADDR>` (e.g. `VehicleEntity__1C51D0`). Rename to the real `Class::Method` as you identify them.
- Asserts: `if (off_6C18D8) off_6C18D8("expr", ".\\src\\…\\File.cpp", line)`. The expression text names members
  and parameters — the best naming source in the binary.
- Methods are `__thiscall`; Hex-Rays often mis-types `this` (ecx) as a `float` under `__userpurge`. Fix the
  prototype (`set_type`) before reading member offsets.
- XDK libraries sit in their own sections (`D3D`, `D3DX`, `XGRPH`, `DSOUND`, `WMADEC`, `BINK*`, `XONLINE`,
  `XNET`) and CRT/XAPI code after `0x3DDD20` — not game code, do not port it; map it to Rust/Bevy equivalents.
- `research/strings.tsv` = every string (`addr<TAB>text`).

## Rewrite conventions (all agents)

- **Mirror the original tree.** One Rust module per original `.cpp`, same name in snake_case
  (`src/Physics/Vehicle/VehicleEntity.cpp` → `velocity_physics::vehicle::vehicle_entity`). Types keep the original
  class names (`VehicleEntity`, `VsRaceManager`); methods keep the original names in snake_case.
- **Every ported function carries one doc line citing its origin**: `/// VehicleEntity.cpp 0x001C51D0`.
  This is the decomp trail; keep it to that one line.
- **Accuracy over idiom.** Same arithmetic order, `f32` where the original uses float, constants copied
  bit-exact (read them from the binary, print with `{:e}`/hex, never retype a rounded value), same branch
  structure and early-outs. Structs keep field order; note the original offset in a trailing `// +0x13FC` comment
  when it helps the next reader. Original bugs are kept (and noted in one line) unless they crash the port.
- Platform layers (D3D, DirectSound, XInput, file I/O, threads) are **replaced** by Bevy/Rust equivalents at the
  lowest layer that the game code calls; everything above that stays a faithful port.
- Game data is read from the user's install at runtime. No game assets in the repo.
- Every file-format reader gets a test that parses **every** matching file in the game dir (positive control),
  and round-trips where a writer exists. A reader is not done until it passes on all files.
- Comments terse, lowercase, only where the code is not self-evident.
- Build: `cargo build` (MSVC toolchain pinned in `rust-toolchain.toml`; the default gnullvm toolchain is broken
  here). Bevy 0.19. Run: `cargo run -p velocity`.

## Agent token budget (user requirement)

- Work silently: no progress messages to the orchestrator, no helper sub-agents.
- Keep tool output small: write large IDA/decompiler dumps to files under `research/` and grep them; never
  print whole decompilations or listings into the conversation, and do not re-read files you already have.
- Final hand-back: at most ~10 lines (what was ported, test status, what remains). Details belong in your
  AGENTS.md section, not in the report.
- Edit AGENTS.md only with the Edit tool (an in-place script edit once truncated it to 0 bytes).

## Crate ownership (parallel agents)

Each agent edits only its own crate(s) and its own section below. Shared pieces go through the owner.

| crate | owner | original source |
|---|---|---|
| `visos` | core | `visos/*` engine core: File, VCD, Chunk, Type, Handy, Memory, Maths, Timer, Instances, Compartment, Timeline file |
| `velocity_game` | core | GameMode, GameDataLoading, EventDatabase, Timeline(Manager), Triggers, Camera, Replays, ScoreManager, GameBridge, Cli, Message, Controller, GameInput |
| `velocity` (bin) | core | `main.cpp`, `VsCommonMain.cpp` — plugin wiring already in place |
| `visos_render` | graphics | visos Renderer, `src/Render/*`, `src/Core/VsRender`, particles, Bone/VertexAnimation, Fury FX, sky, CarPalette |
| `velocity_physics` | physics | `src/Physics/*`, `src/CarDamage`, `src/VehicleDatabase*`, `src/Physics/Vehicle/*` |
| `velocity_audio` | audio | visos `Audio/*`, `src/Audio/*`, music/WMA, `.xva/.xvg/.sbk` |
| `velocity_ai` | ai | `src/Ai/*`, `OpponentDatabase`, `TeamDatabase`, `AI/` data |
| `velocity_frontend` | frontend | `src/Feck2/*`, `src/Osd*`, `src/OSDMap`, `src/GameMenuManager`, Fonts, StringManager, FontHelper, VsMovie |

Status and findings: each owner keeps its section current — formats, struct layouts, key functions, what is
ported, what is stubbed. Durable facts only; no progress diaries.

## Core (engine, formats, game flow)

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
  (`cli::debug_menu::CDebugMenu::load_cli` parses it for the debug menu tree).

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
- **Race session** (`game_mode::race_session`): `RaceSession.state` Init/Reset/Intro/Countdown/Go/Racing/Finished/
  PostRace, `race_time`; `RaceSessionMessage::{ResetToGrid, ClearVehicleInputs, StartFlagGirl, ReleaseCars,
  RaceFinished, StartPostRace}`. Set `RaceSession.race_finished` when the race manager says so. Flag girl not ported
  (start is immediate).
- **Triggers** (`triggers::trigger_update`): fill `TriggerVehicles` each update (AI/physics); effects come out as
  `TriggerMessage` (AI hints, respawn, speed clamp, timeline...), audio ones go straight to `AudioCommand`.
- **Race timelines**: `visos::timeline::timeline_sequencer::TimelineSequencer` (+ `TimelinePeer` trait);
  `timeline_manager::race_timeline_sequencer::RaceTimelines` plays them on TIMELINE triggers and publishes
  `RaceTimelineEvent` (Begin/Update/End/Matrix/BoneAnim) for whoever owns the bound objects.
- **Audio wiring** (done here, at the original call sites): Init + LoadFrontEndGroups on FE init, Init(track) on
  race init, Shutdown on mode shutdown, Pause/Resume, StopGameAudio on restart, RemoveShopAlarm at Reset, Countdown,
  RaceStarted, cover/shop triggers. `GameAudioInput.player/camera` are filled from the player car each update.
- **Frontend ↔ game**: the binary bridges `FrontendRequest` to `velocity_game::GameFlowRequest`
  (`SetControllerMask`, `StartQuickRace{race, professional, wet, car: PlayerCarChoice}`) and stops the FE.
- **Vertical slice stand-ins** (`velocity_game::slice`, to be replaced by real APIs): kinematic player car on
  `gfx\arena.msh` (until physics exposes spawn / `PhysicsCarInput` / transform), car body model through
  `visos_render::model_builder` (until car rendering lands). Real: `LoadTrackRender` + `TrackCamera`, follow camera
  (`camera::vs_vehicle_follow_camera`, VsVehicleFollowCamera.cpp 0x67510), HUD = frontend OSD fed from
  `slice::RaceStatus` (binary `hud.rs`). `velocity --slice-test` runs menu → quick race (Cruise) → car select →
  race → drive, screenshots in `research/screens/slice_*.png`.
- **Requests to other agents**: physics — a vehicle API (spawn at a grid transform, per-update `PhysicsCarInput`
  {steer, throttle, brake, handbrake, gear}, read transform/velocity/rpm/gear/wheels), and the track collision
  query; graphics — car rendering (vehicle folder + setup → entity with body parts, wheels, paint).

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

## Graphics

_owner: graphics agent_

**Coordinate convention (all agents): game coordinates are used in Bevy as-is — no axis is negated, no winding
flip.** Positions, normals, instance positions/quaternions and object matrices go straight into Bevy's right-handed
Y-up space; stored triangles are front-facing counter-clockwise (cull `Back`). Verified by sign text:
`research/screens/exectest_signs.png` reads "MALTMAN'S" and "Port of Long Beach / Piers A-J" correctly (a z-mirror
made them read backwards). D3D matrices from the game (row-vector, `v' = v*M`) map to glam as
`Mat4::from_cols_array(&m)` (same 16 floats in memory order); `D3DXMatrixRotationY(a)` == `Mat4::from_rotation_y(a)`.
Viewer yaw 0 looks down -Z.

### Render API for game flow (`visos_render::RenderPlugin` adds both plugins)
- **Track** (`track_render`): write `LoadTrackRender::new("LACoast")` (dir under `\Tracks`; `fog` flag) to build the
  track under one `TrackRoot` entity (geometry, instances, textures, lights, fog, sky passes); `UnloadTrackRender`
  (or a new load) despawns it. Tag the game's 3D camera with `TrackCamera`: it is set to order 1 / no colour clear /
  no tonemapping, the four sky cameras (orders -3..0, render layers 1-4) follow its rotation and take its
  `RenderTarget`. Use a vertical fov of `sky_render::BACKGROUND_SPAN` (0.8185 rad = 60° horizontal at 4:3) to
  match the background gradient. `TrackRoot` carries bounds; `TrackLights` (resource) the track's lights.
- **Car** (`car_render`): write `SpawnCarRender { request: u64, spec: CarSpec::new("LA/350z", colour), transform }`
  (`CarSpec.parts` = part index per `PART_SLOTS` bonnet/f_bumper/r_bumper/exhaust/spoiler/trim/scoop; `pearl`;
  `wheel` model stem). The root gets `CarRender { request, spec }` + `CarRenderParts { body, wheels: [FL, FR, RL,
  RR], bounds, wheel_radius }`. Each frame set the root `Transform` (chassis pose, game space) and each wheel
  entity's `Transform` (relative to the root, e.g. from the physics wheel render matrices; initial placement is
  only estimated from the body bounds). `DespawnCarRender { request }` removes it. FE car-select can use the same.
- Viewer: `trackview --track ExecTest --car LA/350z --shot …` (`research/screens/car_350z.png`).

**Screenshots** (`research/screens/`): `exectest_01.png` (ExecTest overview), `exectest_signs.png` (street level,
text check), `lacoast_01.png` (LACoast overview + sky), `lacoast_street.png` (street level: sky gradient, lit
instanced palms/lamps), `car_350z.png`. Re-render: `cargo build -p visos_render --bin trackview`, then
`target/debug/trackview.exe [--game <dir>] --track <dir under Tracks> --shot <png> [--cam x,y,z,yawdeg,pitchdeg]
[--size WxH] [--car <dir>] [--nofog]` (headless image target, waits 150 frames, exits; no window). Running the exe
directly needs `--game` or `VELOCITY_GAME_DIR` (cargo sets it). Without `--shot`: fly-cam window (RMB look,
WASD/QE, shift). `cargo run -p velocity -- --track ExecTest` switches the game app into the viewer.
`target/debug/vtfdump.exe [--game=<dir>] '\Tracks\ExecTest\ARENAT.VCD' <outdir> [filter]` dumps textures as .tga.
Everything is read at runtime through `visos::file` (`Vcd::open`, `resolve`); the WGSL is embedded in the binary.

### Formats
- **VTF** (`VTP_`; `renderer::texture_manager`, `CTextureManager::CreateFromVtf` 0x29E3F0). v1 header 48 B:
  +4 u16 version, +6 u8 mips, +7 u8 clut entries (255 = 256), +8/+A u16 w/h, +14 u32 format, +18 clut format
  (1 = rgba8), +1C data, +20 clut, +24 D3D header (zeroed, filled by XGSetTextureHeader), +2C `0xD0FEC0FF`.
  Formats: 9 DXT1, 10/11 DXT2/3 (`D3DFMT` 0x0E), 12/13 DXT4/5 (0x0F) — linear (not swizzled) blocks, full mip
  chain; 2 = P8 (one level, rows stored bottom-up, ps2 clut: rgb doubled with u8 wrap, alpha as is); 7 = ARGB8888
  (unused in data); 1 = P4 asserts. 3649 of 3670 are DXT5. 20 files carry a pre-release **v0** header
  (+6 w, +8 h, +A mips, +B clut, +C fmt, +14 data, +18 clut) the shipping loader cannot read (decoded anyway).
  `.tga` (D3DX path) via `renderer::d3dx_tga`.
- **XMF2** model (`renderer::renderer_types`; relocated in place by `GDL_RelocateModel` 0x9EF30, drawn by
  `CRenderer::DrawPrimitiveData` 0x2A9160). All pointers are offsets from the model start. Header: +0 'XMF2',
  +C flags, +10 sphere, +20 vertex table {ndesc, descs (260 B), nvb, vbs (24 B: +10 data, +14 size)},
  +24/+28 objects (96 B: matrix, sphere, +54 parts, +58 name, +5E u8 nparts) -> parts (16 B: +8 u16 nprims,
  +C prims) -> prims (16 B: +0 shader, +8 ndraws, +C draws) -> draw (60 B: +0 desc idx, +4 vb idx, +8 stream byte
  offset, +C D3DPT (5 list, 6 strip), +10 index kind (1 = 16-bit IB, the only one in data), +2C index count,
  +30 index data, +34/+36 min/max index, +3A start index). Vertex desc = u32 stride + 16 `D3DVERTEXSHADERINPUT`
  {stream, offset, format, tess}; registers 0 pos, 1 weights, 2 normal, 3 diffuse, 9 uv0, 15 indices; formats
  FLOAT1-4 (0x12-0x42), D3DCOLOR 0x40, NORMPACKED3 0x16, PBYTE4 0x44, SHORT4 0x45. Track sectors = pos + D3DCOLOR
  (prelit) + uv, stride 32/64 with garbage padding; props = pos + normal + uv (lit). Shader (20 B): +0 program,
  +4 flags, +8 name (Maya material, not a lookup key), +C nstages (always 1), +10 stage. Stage: +0 texture name,
  +4 handle, +8 flags, +14 **command stream** (`eMS_*` records `code, size, payload`, code 0 ends).
  8102 models / 84367 draws parse; the 12 legacy `XMF1` files are rejected by the game as well.
- **CPT** compartment (`compartment_sys`; `CCompartment::OnLoaded` 0x2E7900, `RegisterInstances` 0x2E7B50):
  64 B header '_CPT', version 5 (+28 resource count, +2C table, +30 instance block; stale **v3** packs (LADown)
  shift these by +4), +10 six-float bounds. Resource table 12 B {offset (0 = empty slot), u8 type (0/8 model,
  1 texture), hash}; texture names are packed C strings from +0x40 in table order. Instance block: u32 nlists,
  12 pad, then lists {type, count, 2 pad} of 80 B records (100 for type 5 SimpleAnimation): pos, w, quat xyzw,
  +3C visual resource, +40 physical, +44 extra, +48 id, +4C name hash. `uinstance.cpt` points at empty slots that
  are filled at load. Shipped packs are stored uncompressed; the chunk codec (RefPack-style LZ,
  `CCompartment::Decompress` 0x2E93E0) is ported anyway. Track layout: `CPTS.VCD` = common.cpt (all track
  textures), instance.cpt (placed props), `Sector_N_high/low.cpt` (one world-space XMF each). Single-arena tracks
  (ExecTest) use `ARENAM.VCD` (one xmf) + `ARENAT.VCD` (textures).
- **Lights.txt** (`vs_render`): 8 rows read column-wise — rows 0-3 = x/y/z/w of 3 light directions (+1 unused),
  rows 4-7 = r/g/b/a of 3 colours + ambient (`FECK2Element3D::LoadFoyerLights` 0x298CD0 shows the layout).
  Lighting = ambient + sum(max(0, n . -dir) * colour), saturated (`VsRender::BuildLightMatrix` 0x58AF0 negates dirs).
- **SkySetup.cfg** (`sky_render`, `CSkyRender::LoadConfig` 0x13ED90): sequential `( ... )` groups, version 1,
  distances x 0.02.

### Ported
- `renderer::shader_manager` — ShaderManager.cpp: stage command interpreter `CShaderManager::SetShaderStage`
  0x2A4960 (only codes 1 blend mode, 2 alpha test + z func, 3 clamp, 4 no z-write, 5 depth bias, 6 stencil and
  22 uv anim occur in data), `GetBlendModeParams` 0x2A67C0, alpha/z func tables 0x65CE30/0x65CE50, stage-0 reset
  MODULATE(diffuse, texture) `SetStageOps` 0x2A7120. `XboxStageMaterial` + `xbox_stage.wgsl`: fixed-function stage
  0 in gamma space (textures uploaded `Rgba8Unorm`, result decoded to linear only for bevy's srgb target), alpha
  test on the 8-bit alpha, D3D blend factors/ops/colour write masks in `specialize`, z funcs mapped to reverse-Z.
  Tonemapping off on render cameras.
- `sky_render` — SkyRender.cpp: config load, `CSkyRender::Load` 0x13B5D0 (code-built shaders: "bg" SkyGradient
  blend 1, "cloud" Cloud_layer_0N blend 3 colour = texture*TFACTOR, "skycyl" Sky_cylinder blend 3, buildings
  sky_cylinder_buildings blend 3; all z test/write off, clamp on bg/cyl), `CSkyRender::Render` 0x13BF30 matrices
  (cylinder scale(0.44, cfg*0.02, 0.44)*rotY(sun rot) + height offset; buildings scale(0.44, cfg*0.02, 0.44)),
  `RenderClouds` 0x13CB90 (scale(1,-1,1)*rotY(sun+pi) + layer height, texture matrix scale = tiling, offset =
  scroll), `UpdateClouds` 0x13BE20 (v += dt_ms * 1e-5 * speed, wrapped to 0..1; frame time assumed ms),
  `RenderBackground` 0x13CF40 (screen quad, v from -asin(fwd.y)/vfov over 0.8185/vfov, colour = background top).
  Draw order kept with one camera per pass. Background written rgb-only (original rgba with alpha 0, which made
  bevy's camera compositing drop the world). Steep downward views show the cylinder's lower rim — the sky is camera
  centred in the original too. `RenderSunMoon` 0x13C390 (billboards at camera + dir*1000, "sun"/"sunb" sun_poly,
  "moon") not drawn yet.
- `vs_render` — `VsRender::SetDirectionalLight` 0x58A10, `SetAmbient` 0x58AC0; models without a diffuse stream are
  lit per vertex at mesh build (lights are static per track). Point lights: `PtLights.txt` (GameDataLoading.cpp
  0x9DE90; record 32 B = pos + radius in w, colour) and `object_lights` = VsRender.cpp 0x59140: per object up to two
  point lights in range (sticky slots, radius-0 lights skipped) as directional slots 0-1 with colour scaled by
  clamp((r-d)/r*2, 0, 1), empty slots black, slot 2 = scene light 0. Cars use it every frame (`TrackPointLights`);
  world geometry stays prelit.
- `fog_render` — the fog object (functions in the CarShaderManager_PC_XBOX.cpp range): init from the Setup.txt fog
  block 0x15A470/0x15A4E0, render states 0x15A990 = D3DFOG_EXP table fog, density = Intensity/1000 (0.3 → 0.0003),
  colour = ColourTop; done in the stage shaders on eye depth. The gradient-fog extras (bottom colour, elevations)
  are loaded (GameDataLoading.cpp 0x9E300) but unused by this renderer.
- `car_render` + `renderer::car_shader_manager` — chassis `<car>_base` + parts, high LOD from VehicleS.vcd, wheels
  from wheels_m.vcd; paint shaders (name test of GameDataLoading.cpp 0x9EAF0) use `CarPaintMaterial` =
  `Shaders/ShinyCarPaint.vxu` + `Default.pxu` ported to WGSL (paint lerp by fresnel pow(n.-camdir, c18.w), base
  alpha masks the painted area, 3 directional lights + ambient, env map spec3 and glitter noise from
  `\VFX\misc.vcd`). Placeholders until GlobalMatInputResolver is traced: c12 env 0.25, c13 noise 0, c18.w 2,
  spec powers 8, neon scale 0.
- `model_builder` — shared XMF→mesh/material building (`Builder`, `SpawnOpts`, `TexSet`).

**Request from core (vertical slice):** car rendering for game flow — vehicle folder + `VehicleSetup` (fitted
parts, wheels, paint) → an entity game flow can move each frame. Until then `velocity_game::slice::player_car`
spawns `<car>_base.xmf` + the `*0` parts through `model_builder::Builder` as a stand-in. The world camera is a
`TrackCamera` driven by the follow camera.

### Stubbed / not yet
Sun/moon billboards, generic vertex/pixel shader programs (`Shaders/*.vxu/*.pxu` are
plain-text xvs/xps assembly; only the car paint pair is ported), sector streaming/LOD (every `_high` sector is
loaded), CarPalette colour tables (caller passes colours), car lights/decals/damage/LOD, effects, particles.
Depth bias (code 5) uses bevy `depth_bias`, not the original projection-z shift (-value/1024).

## Physics

_owner: physics agent_

**Request from core (vertical slice):** a vehicle API game flow can drive — spawn the player car (vehicle folder
`\VEHICLES\LA\240SX` + setup) at a grid transform on the loaded track, per-update input (steer, throttle, brake,
handbrake, gear), read back transform / velocity / rpm / gear / wheel contacts; plus a ground/collision query.
Until then `velocity_game::slice::player_car` is a kinematic stand-in on `gfx\arena.msh`.

### Architecture (what the binary contains)

- The car model is Eugene Laptev's **FastCar** library, linked without asserts/strings into the unattributed range
  **0x355B60-0x3C63D0** (auto-named `_AUDIO_Resample__*`; region attribution there is meaningless). Decompile dump:
  `research/phys_dump/FastCarLib.c`. Objects found so far: `CRigidBody` 0x369640-0x36E7B0 (vtable 0x675894),
  `CFastCar` 0x35DD40-0x369640 + 0x36E7B0-0x372B70 (vtable 0x675854), tyre solver 0x372B70, Maple-generated
  closed-form torque solvers 0x3B3B70/0x3B9D90, expression parser 0x3A9000-0x3AC2A0, `FCMesh`/`FCQuadTree`,
  constraint handler 0x389630 + LCP 0x3BE9A0 (not ported yet).
- The game layer: `VehicleEntity` (src/Physics/Vehicle) computes inputs/forces and sets `CFastCar` members;
  `PhysicsManager::Evolve` (0x1ADF70) integrates every car: `h = frame_dt*24 * LC->timestep * dword_688B70`,
  substepped into 2/3 when h > 0.041841, `GC->number_of_evolves` times, via `CFastCar::Evolve` (vtable[0]).
  Game time is measured in frames of 1/24 s (`SEC` in the cfgs); frame dt is asserted <= 0.041841 s.
- `CFastCar::DetectWheelContacts` calls the callback at car+1104 (`VehicleEntity::WheelCollision` 0x1BB320) which
  fills the contact records; 0x368060 is the library's flat-plane version (y = car+2836).

### Numeric model

- Inline float math is **SSE scalar** (exact f32). Helpers that return a value (dot products, `VsSqrt`, `VsSin`,
  ...) return it in **x87 st0 unrounded** (`fst` then `ret`): in Rust they return f64 (`fastcar::vsmath`), rounded
  where the original stores to a float. x87 precision assumed 53-bit.
- Hex-Rays convention used by the translators: `(float)(a op b)` = f32 op; an uncast float op = x87 (f64).
  `tools/phys_c2rs.py` (straight-line functions) and `tools/phys_c2rs2.py` (snippets with control flow,
  `BASE=name` maps `*(T*)(BASE+off)` to `name.f(off)`) produce reviewed Rust from the dumps.
- FastCar objects are kept as raw 32-bit words by original byte offset (`fastcar::mem::Mem`): the library
  aliases members (e.g. linear and angular impulses both accumulate at RB+512) and compares floats as ints.

### Config (`Vehicles\cfgs`) — ported, `c_config.rs`, `vehicle/vs_config_file_read.rs`

- Text preprocessor: `define NAME value`, identifiers (incl. digit-leading `240SX`, `4WD`) replaced by their
  value repeatedly; a value of `//` comments out the rest; `#` and `//` comments; the trailing-whitespace trim
  in 0x1B6400 is a no-op (original bug). Then every whitespace token that compiles as a constant expression is
  replaced by its `%.7g` value (XDK CRT: round half up, 3-digit exponent).
- Readers expect the next significant line to start with the parameter name (`sscanf(" name %f %f")`), so
  `CConfig.inl` order is the file order. `stop_loading N` first: non-zero ends the file successfully (define-only
  files end with `stop_loading 1`).
- Global (`::GC`, CGlobalConfig 0x870 bytes, load 0x18F870) is read with `LC`=`//`, `GC`=``; local (per car,
  CLocalConfig 744 bytes, load 0x19A220) with `LC`=``, `GC`=`//`, all 49 car names (off_688AA0) `//` except the
  car's own, `BUS`/`NOT_BUS`/`DEFAULT`, `bPLAYER`, spans and PM values as `%g` defines (0x1C9070).
- Expression language (FastCar 0x3AAD60): f32 stack machine, functions abs fabs acos asin atan atan2 cos exp
  max min if not eq lt lte gt gte tan sqrt sin step deg rad log10, constants pi/e (case-insensitive), division by
  |b|<=0.001 gives 0, unary minus binds tighter than `^`. Runtime expressions (force_expression, RPM_expression...)
  are compiled with a variable list and optimised (0x3AB110 reassociates constant pairs).
- Shipped data bug: C2 has no CarBoxes.cfg entry -> its local config stops at `CD_box_size` (original asserts).

### Ported so far

- `fastcar::rigid_body` (Euler 0x36D9E0, RK4 0x36DA90, derivatives, quaternions), `fastcar::car` (ctor, parameters
  0x35E4D0, Ackermann steering 0x361FF0, wheel frames), `car_evolve` (Evolve 0x3652D0 incl. two-wheel balancing and
  physics-vs-play), `car_contacts` (0x362A30, 0x363810), `car_tyre` (0x372B70), `car_wheel` (render matrices
  0x36E7B0), `solver` (0x3B3B70/0x3B9D90). `tests/fastcar_flat.rs`: 240SX settles and drives on a flat plane.
- Helper agents ported `vehicle_database/*` (all bins, `string_hash` = case-sensitive CRC of the upper-cased name)
  and `fastcar::{mesh,quadtree}`, `material_properties`, `c_bump_map` (arena.msh/qdt formats documented in
  those modules).

### Requests from AI (ai agent; nothing in velocity_physics was edited)

- Insert `velocity_ai::plugin::AiPhysics(Box<dyn AiPhysicsWorld + Send + Sync>)` once the race cars exist; the
  traits are in `velocity_ai::ai::ai_context` / `vs_ai_vehicle` (position, orientation, forward, velocity, speed,
  the `VehicleControls` block at VehicleEntity+0x9C, teleport, steer angle, control mode, gear/rpm/NOS, line checks).
  Until then the AI loads its data but does not drive.
- Bind cars like PhysicsManager 0x0019E080: `race.ai[i].base.set_physics_vehicle(h)`, players `set_physics_vehicle`
  + `set_player(true)` + `set_human_control(true)`, traffic `traffic.vehicles[i].base.set_physics_vehicle`, police
  `police.vehicles[i].set_physics_vehicle`; then teleport race cars to their grid slots before the grid reset.
- Teleport orientations are quaternions applied through their conjugate (body forward = `vs_quat_rotate(q, +X)`,
  see the AI section); `steer_angle_to` (0x003717D0) and `set_steer_angle` must use the same sign as the AI's
  `get_steer_towards` (negative steers towards +x when facing +z).
- Write `velocity_ai::plugin::AiCollisionMessage { a, b, impact }` (car ids, -1 for scenery) for car collisions
  (the world message handled at 0x000160B0).
- Armco: build the collision mesh from `AiWorld.armco.mesh` (`FCMesh` + `FCQuadTree(8.0, 2.375, 0, 1)` at 0x003593D0)
  and resolve traffic blocker physics objects through `AiWorld::link_traffic_blocker_objects`.

## Audio

_owner: audio agent_ — crate `velocity_audio` (`cargo test -p velocity_audio`, 21 tests; `--features wma` adds the WMA path).

**Use from other crates.** `AudioPlugin` adds resource `VisosAudio { system: AudioSystem, game: VsAudio, mixer }`,
resource `GameAudioInput` (overwrite each frame: camera, player car rpm/gear/speed/wheels/surfaces, ai, traffic,
police, wet, tft) and message `AudioCommand` (Init{frontend,track_dir,race_mode}, LoadFrontEndGroups, Gui(GuiSound),
Nos/Slipstream/Zone/Tb/Mode3 start/stop, Countdown, RaceStarted, Impact{event,pos,strength}, Settings, PlayTrack,
Pause/Resume, Shutdown...). Order: `AudioSystems::Commands` then `::Update` in `Update`. Output is an engine-side
48 kHz stereo mixer exposed as bevy audio source `VisosMixOutput` (only registered if bevy's AudioPlugin exists;
headless works). Everything decodes in memory at runtime from raw game files via `visos::file`.

**Module map.** `audio/` = visos audio layer (AUDIO_* in the binary): `xva` banks, `adpcm`, `xvg` groups, `load`
(LoadGroup 0x2BD6C0, LoadBank 0x316450), `event` (PlayEvent 0x2BE440, selection, chains, update 0x314DF0), `sample`,
`emitter` (3D 0x315180), `system` (Update 0x2BCFE0, category volumes/presets), `xbox/voice` (dsound voices; DS3D
emulated: inverse distance, pan, doppler), `mixer`. Game side: `vs_audio` (VsAudio.cpp, `g_AudioSystem` 0x6D4558,
Init 0x47A00, Update 0x48BC0), `vs_audio_vehics` (VsAudioVehics.cpp from 0x4FD50), `vs_audio_events` (player engine,
gear/turbo, old-id table 0x684F10), `vs_audio_rev_bands` (VsAudioRevBands, 0x4C6F0..0x4D470), `vs_audio_track_list`
(MusLists.dfm), `vs_soundtrack` (soundtrack player 0x54FD0..0x56840), `wma` (feature), `input`, `plugin`.

**Formats.**
- `.XVA` 'ASVX' v3: 16-B header (magic, ver, sampleCount, bankFlags) + 12-B {size, format, fileOffset}. format bits
  0-3 codec (0 PCM, 1 Xbox ADPCM), 4-6 channels, 7-11 bits, 12-15 rate idx (1 8000,2 11025,3 12000,4 16000,
  5 22050,6 24000,7 32000,8 44100,9 48000,A 64000,B 28000,else 30000). bankFlags&0x70000 residency (0x20000 =
  streamed, 2 KB-aligned). Xbox ADPCM = IMA, 36-B blocks/channel (s16 pred, u8 index, u8 0), stereo interleaves
  4-B words, 64 frames/block (header + 63 nibbles; the 64th nibble is filler). 8-bit PCM is signed.
- `.xvg` 'GSVS' v3: events 20 B {hash, ?, flags, ?, soundCount}, sounds 64 B (weight, vol/pitch(cents)/delay ranges,
  selection flags), samples 16 B {weight, vol%, bank, sample}, chains 12 B, then debug names. event hash =
  HANDY_CRC32(uppercase name, 0); EnvFX groups use small numeric ids. Selection = flags&0xF00 (0 random, 0x100 seq,
  0x200 control param, 0x300/400/500 min/mid/max), 0x1000 weighted, 0x2000 no-repeat; event 0x20000 loop.
- `.rbd` rev bands: group[128], count, cents/rpm ranges, smoothing, volume; 40-B bands {event, lo, hi, fadeIn,
  fadeOut, vol, centsRef, cents, flags 1 on/2 on-load/4 off-load}.
- `envaudio.aee` 'AED1': count, 76-B records {name[32], pos (z negated on load), dir, min, vol, max, ?, event};
  names "GLOBAL_" / "ONESHOT_a:b". `velocity.vol` = 32x32 category volume presets. `MusLists.dfm` (MDV2 text):
  3 lists + master table name→`.wma` path; track names are also music-bank event names.
- Unused by the game: `ProjectVelocity.mix`, `vsfx.bin`, `asp/*` (no xbe refs); `voice.bin` = DSP image.
  Music XVA banks (`Audio\Music\fe\music_fe`, `ingame\music`) are not loaded by the original but hold the same songs.

**Decisions / quirks.**
- Music default = XVA music banks (exact); WMA (WMAv2 44.1k, 68 files) via `oxideav-wma` 0.0.4 behind feature
  `wma` + env `VELOCITY_MUSIC=wma`: decodes all files with 0 frame failures but only ~20 dB SNR vs ffmpeg.
- Data bugs kept: vvipesu0.xvg references samples 3..5 of a 3-sample bank; aee "volume" values (e.g. 40) fail
  SetEmitterVolume's range check exactly like the original; LACoast AudioEmitter_5 has max distance 0.
- Dead in this build: software mixdown path, road-noise/wheelspin events, numeric events 210/979/1314/1801.
- Original sets only the DS listener position (no orientation/velocity); doppler factor 0.3, speed 343.3.

## AI

_owner: ai agent_ — crate `velocity_ai`; raw dumps in `research/ai_dump/` (`clean/*.c`, `_callers.txt`, `_globals.txt`).

### Ported modules (`src/ai/`, one per .cpp)
- `vs_math` VsVector4 helpers: `a-b`/`a+b`/`a*s` return a NEW vector with w=1 (VsSub 0x16450), `-=` etc keep w;
  Normalise -> zero if len<=1e-6; statics VS_ZERO 0x7805C0 / UNIT_X 0x7805D0 / UNIT_Y 0x7805B0. Y is up,
  steering math is in xz. **Orientation quaternions act through their CONJUGATE**: the engine turns a quat
  into a column-convention matrix (0x2AC2C0) and transforms row vectors (D3DXVec3Transform), so
  `vs_quat_rotate` (0x2ABDE0) rotates by conj(q). Car body forward = `vs_quat_rotate(orientation, +X)`; spawn
  code builds `axis_angle(pi/2 - angle(+Z, dir), +Y)` for a car facing `dir` (confirmed: traffic only stays
  on its lanes with this convention). Physics must interpret teleport orientations the same way.
- `vs_ai_response_curve`, `vs_ai_road_line`, `vs_ai_tools`, `vs_ai_road_section`, `vs_ai_globals` (tweaks,
  defaults from 0x36E00), `vs_ai_world` (spline/barrier/handling loading, route graph), `vs_ai_vehicle`,
  `vs_ai_driver`, `vs_ai_race_vehicle`, `vs_ai_startline_revs`, `vs_ai_player_tft_manager`, `vs_race_manager`,
  `vs_traffic_manager` (14-car pool, spawn/despawn by player proximity 140/220m, triggers),
  `vs_ai_traffic_vehicle` (lane following, queueing, red blockers, lane end), `vs_ai_traffic_light`,
  `vs_police_manager` (Patrol/Alerted/Chase/Pursuit; alerted by a player >30 m/s within 20 m of a traffic cop or
  hitting one; wanted level; gives up at 200/300 m), `vs_ai_police_vehicle` (race part + traffic part on one
  physics car; police traffic parts are spawned by the traffic manager via `TrafficSlot::Police`), `armco`
  (`armco01.ar` = SPL0 lines -> 4 m wall mesh, material 21; data for physics to build an FCMesh),
  `ai_world::AiWorld` = VsAIWorld Init/LoadWorldData/Reset/Update/Shutdown + collision handler 0x160B0.
  Police/Armco/glue regions: police ctor in VsAIPlayerTFTManager region (0x17420..0x17530), police manager
  ctor/state table in VsAIWorld region (0x38790..0x389C0), armco loader 0x11090 below the Armco region.
- `VsAIWorld::Init(traffic, single_player, police, barriers, armco)` from GameManager 0x000B5F80:
  traffic = option && !network, police = (traffic || pursuit) && !drag, barriers = !professional,
  armco = professional (`byte_6F70E9` is professional). Ids: race cars from num_players, traffic from
  num_opponents, police +14. Update order: race manager, traffic (if enabled and sections), police.
  Grid heading = axis_angle((90 - start_direction) deg, +y); eventdef start line = `start_line_original`
  (0x6F5C24), finish line = lap line `start_line` (0x6F5C44), unknown point = finish point (0x6F5C64).
- `plugin.rs` (AiPlugin): load steps AiWorldInit/AiWorldData/TftManager in `Last`; `GameTick`
  `InGameUpdateSet::Ai`: RaceSessionMessage::ResetToGrid, `AiCollisionMessage`, then `AiWorld::update` (1/24 s);
  writes `RaceSession.race_finished`. Optional resources `AiPhysics` (cars; without it the AI loads but does not
  drive), `AiHostResource`, `AiTftPlayer`, `AiRespawns`. Opponents are assigned in database order until the
  event's competitor list is exposed (PhysicsManager 0x19E080 takes them from the event).
- Still wanted from core: the event competitors' opponent entries (EventDatabaseEvents 0x92070), the countdown
  start/end times (GameManager +0xCD8/+0xCDC, start-line revs), the speech flag (`byte_6F915E`), network race.
- Traffic code region misattribution: traffic vehicle 0x29830..0x29EA0 is labelled VsAITrafficLight, light
  reset/update 0x29570/0x295A0 is labelled VsAITools. Traffic car layout 332 B: +0x18 cruise 14, +0x1C max
  speed, +0xF0 traffic.bin entry (type bits at +0x0C: 1 ambient, 2 fire engine), +0xF4 lane, +0x100 lookahead,
  +0x130 dist, +0x134 next blocker, +0x13C behaviour, +0x144 end mode (1 stop, 2 despawn), +0x148 lane offset.
  Lane (100 B): +0x28 speed, +0x2C car count, +0x54/+0x58 race section/dist (never set by ai code), +0x5C blockers.
- Default tweaks never take the race car's corner-braking branch (|corner tightness| <= (2*pi/8)^3 = 0.4845 <
  0.5*skill + 0.6 "Difficulty Scale"), so race ai drives flat out and relies on the physics car to corner.
  Crate root: `crc`, `text_file` (VsStringAnsi reader), `opponent_database`, `team_database`.
- Region misattribution in the idb: VsAIRaceVehicle ctor/init/reset are at 0x17E40/18000/18190 (Police region),
  driver ctor/reset/speak in Armco region (0x14350..0x146E0), respawn in TrafficVehicle region (0x2B800..2BB30),
  `VsAIDriver::SetOpponent` is 0x1B4120 (PhysicsManager region): opponent=+4, temperament(+0x3C)=opponent+0x48.
- Race setup lives in game code: PhysicsManager 0x19E080 binds physics cars (`set_physics_vehicle`,
  `set_player(true)`, `set_human_control`) and opponents; GameManager teleports (activates) cars before
  `VsRaceManager::reset_to_grid` (0x3C5D0 only resets active ai cars).
- Inputs the ai needs are traits in `ai_context.rs` (`AiPhysicsWorld`, `AiPhysicsVehicle`, `AiHost`,
  `AiGameState`, `AiEventInfo`); `VehicleControls` mirrors VehicleEntity+0x9C (steer +0, throttle +8,
  brake +0xC, handbrake +0x10, nitrous +0x14; defaults per 0x1B7D90).

### Formats (all files in the game parse; tests cover every one)
- `AISpline.spl`: "AISPLINES", version byte @9 (=2); counts @0x10: race sections, traffic sections, set B,
  set C, lights, blockers, trailblazer; 48-byte header. Race section = 3 road lines (raceline, left, right).
  Road line = 16-byte header {kind u32 (2/10), circular u8, flags[3] (flags[1]&7 shortcut level, &0x80 locked),
  num_points, lane id} + vec4 points (w = cumulative distance). Road section = 16-byte header + lanes.
  Blocker 32 B {pos, id, flag, numLanes, physObjId} + lane ids padded to 4; light 32 B {id, f32 cycle, 4 blocker
  ids, 2 zero}. LACoast circuits duplicate sections, so "untracked sections" warnings there are genuine.
- `*.br` barriers: "SPL0", count @4, 16-byte header, road lines.
- OPPONENT.BIN (loaded from `\VEHICLES\XMLBIN.VCD`): f32 1.5, i16 count, 136-byte entries (+0 name[16],
  +16 short[16], +32 crc, +36 group, +40 flags, +52 setups, +56 u8[12] tft line counts, +72 base aggression,
  +76 skill, +80 level), then 52-byte setups, then 12 string lists (u32 len + bytes).
- TEAMS.BIN: u32 count, u32 pool size, teams, colour-name pool.
- Handling `AI\Handling\<car>.txt`: name, 4 curve names, max speed, lookahead min/max, steer/throttle/brake
  factors, steer damping. Every race car uses "350Z", traffic "TRAFFIC". Original bug kept: slot +0x14 looks up
  "RC_sp3" but loads the brake curve. Startline rev curves Revs1..3 (table 0x684E84) cycle per ai car.
- CRC: reflected CRC32, seed 0, no final xor (0x2FD200); names hashed via `string_crc`.

### Headless lap test (`src/ai/lap_test.rs`)

Kinematic bicycle stand-in (no slip, 200 m/s^2 lateral cap) implementing `AiPhysicsWorld`; one player with
`player_ai_control` plus 0/3 opponents. London Circuit Race01 laps in ~85 s, LADown Circuit Race01 in ~52 s
with 14 traffic cars (90 spawns, lights cycling, no red-blocker runs, traffic within 2.6 m of its lane, a police
car joins the traffic); race cars stay within 6 m of the edge lines (corner apex cutting). Pursuit test: both
police cars leave the grid chasing a scripted player and stay within 11 m of it over 60 s.
`plugin::tests`: a `jumptogame` boot builds the AI world from the core's load steps.

## Frontend / UI

_owner: frontend agent_

### Talking to the frontend (game flow / core)

`velocity_frontend::FrontendPlugin` (wired in `velocity`) boots the frontend at `MovieScreen` unless the app was
started with `--track` or `VELOCITY_NO_FRONTEND` is set. Bevy messages:
- in: `FrontendCommand::Start("<ScreenName>")` (= `CFrontendManager::Start(crc)`, shows the FE),
  `FrontendCommand::Stop` (hide, stop updating).
- out: `FrontendRequestMsg(FrontendRequest)`: `PlayIntroMovies` (movie screen activate, `GameManager::B4D30`),
  `SetControllerMask{player, mask}` (`GameInput::SetDeviceMask` from main menu start / quick race),
  `StartQuickRace`, `Exit`.
- on `StartQuickRace` game flow builds the event from `Frontend.fe.quick_race` (`QuickRaceSettings`: the
  `GameVars` quick race globals 0x6F70E8..: track/race indices into the frontend's `CEventDatabase`, race type,
  laps, opponents, weather, difficulty, pro, `player_setups[2]` = `VehicleSetup` of the chosen cars,
  `vehicle_tier`, `automatic`); then the original goes to `LoadingScreen` (not ported) and after the race to
  `post_race_screen` ("RaceCompleteScreen"), returning to `return_screen` ("QuickRaceStartScreen").
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
| `feck2::feck2_elements::*` | FECK2Element2D.cpp, FECK2ElementStaticText.cpp |
| `feck2::screens::common::fe_2d_overlay` | FE2DOverlay.cpp (creation, attach, reset, nav prompts, borders, load) |
| `feck2::screens::startup::fe_main_menu_screen` | main menu, legal, movie screens |
| `feck2::screens::startup::fe_profile_select_screen` | STUB: Start returns to the previous screen (no profiles) |
| `feck2::screens::quick_race::fe_quick_race_text_screen` | FEQuickRaceTextScreen.cpp (create 0x226390, track/race/options rows, random) |
| `feck2::screens::quick_race::fe_quick_race_car_select_screen` | car select (create 0x2223D0: tier -> car -> colour -> commit) |
| `feck2::screens::quick_race::fe_quick_race_start_screen` | race summary (create 0x2203A0, in FECarShowroomScreen.cpp region) |
| `osd::osd_manager` | Osd\OSDManager.cpp (item pool, anchor/justify layout 0xEE5F0, text render 0xEEA70, FormatTime 0xEF840) |
| `osd::osd_screens::osd_in_game_screen` | OSDInGameScreen.cpp (Create 0xF82F0, Update 0xF99E0 race-status part) |
| `osd::speedo` | Osd\Speedo.cpp + OSD3dInterfaceConfiguration.cpp hud config (0xEB9F0) |
| `osd::osd_map` | CircuitEditor\CHUDMapConstructor.cpp, OSDMap\COSDMapRender.cpp |
| `game_menu_manager::game_menu_manager` | GameMenuManager.cpp, Menus\GamePauseMenu.cpp, CQuitMenu (GameTweaksMenu.cpp region) |
| `hud` | bevy glue: `HudPlugin`, `RaceHudInput`, `HudCommand`, `PauseMenuMsg` |

Quick race flow: MainMenu (Start -> ProfileSelect stub -> back, A on QUICK RACE) -> QuickRaceTextScreen
(Select) -> QuickRaceCarSelectScreen (tier, car, colour stage, commit) -> QuickRaceStartScreen (A =
`StartQuickRace`, X back to car, Y back to track, B to main menu). Car select reads tier counts / nth car /
default setup from `velocity_physics::vehicle_database`; the 3D foyer (car models, logos, polaroids), colour
swatches, stats panel and automod are not ported. Car names are the vehicle entry hash used as a string id.

FE draw order is submission order (`Renderer` 0x2A9B00 draws immediately, fonts flush per string), so the car
select's car name (created after the title bar) is drawn under the pulsing bar as in the original.

#### In-race HUD (`hud::HudPlugin`, added by `FrontendPlugin`)

- in: `HudCommand::Start { hud, drag, inside_edge, outside_edge, closed }` at race start (`hud` = digital hud
  pack name under `OSD\HUDS`, falls back to `digione`; edges = the track's inside / outside x/z polylines for
  the minimap, empty = no map), `HudCommand::Stop`, `HudCommand::OpenPauseMenu`. Every frame game flow fills
  the `RaceHudInput` resource: `status: OsdRaceStatus` (racing, race length, position, lap/laps, drag rounds,
  lap time, points, combo, multiplier, wrong way), `speedo: SpeedoInput` (speed m/s — shown * 2.25, rpm, gear
  (0 N, <0 R), nos / zone / turbo ratios), `map: MapInput` (player pos, forward vector, rivals, police).
- out: `PauseMenuMsg(PauseMenuRequest)`: `Resume`, `Restart`, `QuitToFrontend`, `OnlineOptions`,
  `RumbleChanged(bool)`, `OptionsChanged` (volumes / audio mode / display toggles in `CGameMenuManager.options`).
  The pause button (action 13) opens the pause menu.
- drawn into a transparent 640x480 overlay (`DrawList::rasterise_overlay`) on camera order 11, no clear.

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
units (`px / tex size * hud size`). PROVISIONAL: the HUD plane is shown with an ortho map x -16..16,
y 12..-12 (the OSD3dInterface camera that places hud models relative to the car camera is not traced), and the
`+44` gauge pass (blend 4) is not drawn. 3D analogue huds (`OSD\OSDM.vcd` models, `OSDD.VCD` `.osd`) not ported.

**Minimap (`osd::osd_map`):** `CHUDMapConstructor` 0x7DEE0 ported exactly (square bounds over both edges,
border pad, Bresenham edges = 255 into `[x*size+z]`, 0x80 outline pass, palette = hud map colour with
alpha `max(index, min)`); `COSDMapRender::Update` 0xE8E70 window = 900 world units, rotated by heading + pi
(plus a quarter turn so forward is up — the map screen model's uv orientation is assumed), blips
`plyrblip/rvalblip/blueblip/redblip` and `nav_grid` from `OSD\OSDT.vcd`. Drawn as a 2D panel at the hud's map
position through the provisional plane mapping.

**Pause menu (`game_menu_manager`):** `CGameMenuManager` stack (8), `CGameMenu` layout (≤13 rows, 42 apart,
centred 800 wide), up/down with 0.2 s repeat, select (action 6), cancel (7) pops (last one = resume),
left/right (0/1/5). Pause items (0xAF070): Resume, Audio, Video, Rumble (idRumble), Restart, Quit to FE,
Online Options; Audio (0xAD940: six volumes ±0.05, audio mode 0 stereo/1 surround/2 mono, Done), Display
(0xAE880: ratio, TFT, names, jump/crash/police cam toggles, Done), Quit confirm (0xB0DB0: YES/NO; the confirm
text is shown one row above, which the original does elsewhere). All in-race menus set no-frame, so only the
`>` cursor and Tahoma text are drawn. Not ported: opponent name tags, OSD graphic items of the in-game screen.

Movies: `Movies\PAL\**\*.bik` are **Bink 1** (`BIKi`). No Rust Bink decoder exists; the only open
implementation is FFmpeg's `bink.c`/`binkaudio.c` (LGPL), so the route is a Rust port of that decoder or
linking FFmpeg. The movie screen therefore only emits `PlayIntroMovies` and moves on.

Captures: `cargo test -p velocity_frontend capture` writes `target/fe_captures/*.png` from the headless driver
(`frontend::Frontend`): fonts, legal pages, main menu, quick_race_text, car_select_tier/car, quick_race_start,
osd_in_game, race_hud(_paused), pause_menu / pause_audio / pause_quit.
