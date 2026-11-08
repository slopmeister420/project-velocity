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

**Unlock launch option (user requirement):** `-u` / `--unlock` boots with the debug build's unlock tweaks on,
default is locked (retail). Implemented in `velocity_game::cli::debug_unlocks` (see ## Core).

**Any frame rate (user requirement):** the game must run correctly at any render rate (30, 60, 144, uncapped).
The original already splits a fixed 24 Hz game update (`GameClock` / VsFrameTimer, `GameTick`) from rendering
with `timer.lerp` interpolation — keep that split: game logic, physics, AI and timing only in `GameTick`;
everything drawn from 24 Hz state (cars, cameras, characters, HUD values that move) interpolates with
`timer.lerp` the way the original's render path does; per-render-frame code (particles, frontend, FE animation,
audio) uses the real frame delta, never an assumed 1/60. No hard-coded per-frame steps outside `GameTick`.

**Headless test runs (user requirement):** the user is often at the PC doing other things. Every run of a game
exe (`velocity.exe --smoke-test`, `fecapture`, `trackview`, ...) goes through `tools/hidden_run.py`, which starts
it on a separate invisible Windows desktop so it can never appear or take focus, and kills it on timeout:
`.venv-ida/Scripts/python tools/hidden_run.py --timeout 300 [--env K=V] -- target/debug/velocity.exe --smoke-test
--game "D:/games/xbox/games/project velocity debug"`. Never launch a window on the user's desktop, never enumerate
or capture its windows. Screenshots and recordings are taken inside the process (Bevy screenshots); wgpu/Vulkan
renders fine on the hidden desktop. Run `tasklist | grep velocity` before a launch and kill leftovers after.
hidden_run sets `VELOCITY_HIDDEN=1`: the exe then inits no SDL, ignores the keyboard and holds no raw input (exits 3
otherwise) — input-inert; never launch a game exe any other way (docs/core.md "Input inertness").

**Bink movies (user decision, 2026-11-04):** port RAD's Bink 1 decoder 1:1 from the debug XBE's `BINK*`
sections into Rust (owner: frontend, alongside VsMovie) — the one exception to "don't port XDK library code".
Don't port FFmpeg's `bink.c` or use NihAV (licences).

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
- The worker exits after ~1 h idle, leaving the server with no session (`idb_list` empty). Reopen it with
  `tools/mcp.py idb_open '{"input_path": "D:/projects/project poland rewrite/research/default.xbe.i64"}'`.
- Keep a dated copy `research/default.xbe.i64.bak` (e.g. `.bak-20261107`) after each session's renames, and never kill the IDA worker without `idb_save` first (a kill corrupted the IDB once; `tools/ida/rebuild_idb.py` rebuilds it).

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
- Never change `PATH` or other system environment variables; pass absolute tool paths or scope a variable to
  one command.
- Build: `cargo build` (MSVC toolchain pinned in `rust-toolchain.toml`; the default gnullvm toolchain is broken
  here). Bevy 0.19. Run: `cargo run -p velocity`.

## Agent token budget (user requirement)

- Work silently: no progress messages to the orchestrator, no helper sub-agents.
- Keep tool output small: write large IDA/decompiler dumps to files under `research/` and grep them; never
  print whole decompilations or listings into the conversation, and do not re-read files you already have.
  Read files by line range (`Read` offset/limit, `grep -n`) rather than whole; cap command output (`| tail`,
  `grep`), especially cargo output — `cargo ... 2>&1 | grep -E "^(error|warning: unused)|test result" | head`.
- Read only your own `docs/<subsystem>.md` in full (it is a short index); grep `docs/<subsystem>/details.md` and other
  subsystems' docs by heading for what you need, never read a details file whole. Keep the index under ~12 KB:
  durable findings go into details.md, the index only lists APIs, open requests, next candidates and headings.
- Look at screenshots sparingly: one confirming image per change, at reduced size if the tool allows.
- Final hand-back: at most ~10 lines (what was ported, test status, what remains). Details belong in your
  `docs/<subsystem>.md`, not in the report.
- **Keep the PC usable (user requirement):** run every cargo build/test through
  `.venv-ida/Scripts/python tools/lowprio.py cargo ...` (below-normal priority; children inherit it). Builds are
  capped at 6 jobs in `.cargo/config.toml`; `hidden_run.py` starts games at below-normal priority. At most one
  hidden game per agent at a time.
- At most **three agents run at once** (one per subsystem; build into your own `CARGO_TARGET_DIR=target/<subsystem>`),
  and each works in **stages**: finish one numbered stage, leave the code
  compiling, update your docs file, hand back, and stop. The next stage is started as a fresh agent that
  reads the docs, not by resuming a long conversation.
- Stages are launched as the custom agents in `.claude/agents/velocity-<subsystem>.md`: they carry the standing
  brief (so the launch prompt is just the stage) and only the six file/shell tools, which keeps the browser,
  docs and app tool schemas out of every agent turn. Mechanical stages (sweeps and reruns, smoke/test scripts,
  doc upkeep, tooling) run on **Sonnet** (`model: sonnet` at launch); porting from the decompile stays on the
  default model.
- Edit every file (source, CLAUDE.md, `docs/*.md`) only with the Edit/Write tools, never with sed, shell appends or
  Python edit scripts: script edits truncated CLAUDE.md to 0 bytes once and `track_view.rs` from 377 to 91 lines
  (unrecoverable, the tree is untracked).

## Crate ownership (parallel agents)

Each agent edits only its own crate(s) and its own `docs/<subsystem>.md`. Shared pieces go through the owner.

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

Status and findings: each owner keeps its docs file current — formats, struct layouts, key functions, what is
ported, what is stubbed. Durable facts only; no progress diaries.

## Subsystem notes (read only what you need)

Per-subsystem formats, APIs, findings and cross-agent requests live in `docs/`:
`docs/core.md`, `docs/graphics.md`, `docs/physics.md`, `docs/audio.md`, `docs/ai.md`, `docs/frontend.md`.
Each owner keeps its own file current (Edit tool). Read your own file in full; read another subsystem's file
only for the API or request you need (grep it rather than reading it whole). Requests to another subsystem go
into that subsystem's file under a "Requests from <you>" line.
