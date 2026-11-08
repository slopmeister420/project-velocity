# AI - INDEX

_owner: ai agent. Crate `velocity_ai`. Details live in `docs/ai/details.md` (the full former ai.md, ~55 KB): NEVER read it whole; grep it by heading (`grep -n '^###' docs/ai/details.md`, then `Read` with offset/limit)._

## APIs other crates use (details: "Ported modules", then the named section)
- `AiPlugin`: load steps AiWorldInit/AiWorldData/TftManager in `Last`; `GameTick` `InGameUpdateSet::Ai` (`AiWorld::update` with `GameClock.timer.scaled_dt()`); collisions after Physics (`PhysicsCollision` -> `AiCollisionMessage` -> `on_collision`). `AiPhysics` installs itself over `PhysicsWorldRes`.
- Optional resources: `AiHostResource`, `AiTftPlayer`, `AiRespawns`, `RaceCarSourceRes` (`AiRaceCarSource`: opponent / team / police cars; `team_car_by_team`).
- `RaceProgress` filled by ai each update: `order`, `order_ids`, `player_position(s)`, `player_state`, `racers`, `active_racers`, `leader_human`, `racer_records` (`results_busy` / `car_flags` not ai's).
- Messages in: `RaceSessionMessage::{ResetToGrid, EventOver, DragResult}`, `GameModeMessage::RaceRestart`, `TimelineObjectMessage` (EnableRaceSection, RespawnRacersOnRaceline, PlayerHumanControl, Respawn), `TriggerMessage::{AiHint, Trailblazer}`, `plugin::AiRespawnOnRaceline { vehicle, speed }` (from the crash camera), `CareerMessage::{Created, Loaded}`.
- Messages / resources out: `TrailBlazerMessage { vehicle }`, `SetPhysicsWalls`, `FadeStart(t)`, `plugin::AiCinematicBars(bool)`, `RaceSessionMessage::OsdFlags`, `plugin::PoliceFlash([f32;4])`, `visos_render::flare_render::FlareFlash`, `AiTeams` / `CareerTeams` (`TeamRequest`).
- `VsAIDriver::name_colour()`; `AiWorld::{enable_race_section, drag_eliminated_car, crash_camera_respawn}`; tweaks `ai_tweaks::{add_tweaks, add_tft_tweaks}` on `GameManager.tweaks` (151 + 7 tweaks).
- Host / physics traits in `ai_context.rs`: `AiPhysicsWorld`, `AiPhysicsVehicle`, `AiHost`, `AiGameState`, `AiEventInfo`; adapter `physics_ai::PhysicsAi`. The ai shares the game rand stream (`GameManager.rand.state`) and installs the physics race-distance hook itself.
- Env: `VELOCITY_AI_TRACE=<s>`; tests: `tests/ai_lap.rs` (needs `VELOCITY_GAME_DIR`), `lap_test` (`SWEEP=`, `LAPDBG=1`, `LAPDIFF=x`, `LAPWALLS=0`).

## OPEN requests
- To ai: none.
- From ai (filed in core.md): configurable debug-menu path so the smoke reaches an ai tweak set; per-player `player_positions` / `player_states` in `RaceProgress` for player 1's HUD; crash-camera respawn event is ignored by core's `apply_crash_events` (ai side ready); core's round-2 drag reload re-creates the ai world.
- Known original behaviour (do not "fix"): no corner braking, recover loop, LACoast placeholder eventdefs, `point_ahead` endless walk on duplicate sections.

## Next candidates / coverage
- Coverage 2026-11-06: 398/866 functions, 144147/190095 bytes (75.8%) of the ai regions; the rest is debug draw/text, misattributed CAlloyEditor code, physics helpers, ctors/dtors. Regenerate with `tools/ai_coverage/cov.py` then `tag.py`; list in `research/ai_coverage.txt`.
- Unported: OSD gap fade 0x169940, chased-by-police and slipstream strength score inputs.

## Section index of details.md (heading, line)
- L5: Coverage; debug-menu tweaks
- L33: Ported modules (`src/ai/`, one per .cpp) — world init, plugin, binding, controls block, walls, score, race car source, corner braking, recover loop, lap results, start line, cruise, police skill
- L184: AI review (headless smokes)
- L215: Split screen (two human players)
- L238: Restarts and drag rounds
- L267: Respawn flow
- L291: Explosions and the ai
- L307: Timeline AI events and race progress
- L323: Formats (AISpline, barriers, OPPONENT.BIN, TEAMS.BIN, handling)
- L340: Requests from physics — status
- L354: Real-physics lap test
- L366: Headless lap test
- L375: TrailBlazer runner
- L396: Respawn fade
- L403: Career teams
- L423: Requests from core — done
- L436: Requests from core (name tag colour)
- L447: Requests from graphics (police flash for the 3d hud)
- L458: Requests from frontend (cinematic bars off after a respawn)
- L466: Requests from core (point_ahead endless walk)
- L487: Requests from graphics (flash records, dummy police car)
- L514: Requests from core (race record block shared with the score manager)
- L533: Requests from physics (friction spots)
