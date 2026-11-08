# AI

_owner: ai agent_ — crate `velocity_ai`; raw dumps in `research/ai_dump/` (`clean/*.c`, `_callers.txt`, `_globals.txt`).

### Coverage (2026-11-06, `research/ai_coverage.txt`)
- Ported 398/866 functions, 144147/190095 bytes (75.8%) of the ai regions (src/Ai/*, OpponentDatabase, TeamDatabase)
  by origin-line address (any `//`/`///` line with a `File.cpp 0x...` citation). The rest is not gameplay: debug
  draw/text (debug draw 0x35610, ApplyDebugHandling, TFT "Play Movie" 0x16FF0; 8 debug fns / 10 KB), **CAlloyEditor code misattributed to VsTrafficManager
  0x40AC0..0x43610** (10 KB), VehicleEntity transform helpers 0x2C580..0x2C720 (physics), ctors/dtors/VsArray,
  and assert-wrapped accessors folded into field access. Regenerate: `tools/ai_coverage/cov.py` then `tag.py`
  (objregions × funcmap × origin lines; tags from `research/ai_dump/*.c`, names `research/ai_dump/cov_names.tsv`
  from `tools/ida/ai_cov_names.py`).
- **Debug-menu tweaks** (`ai_tweaks.rs`, on core's `GameManager.tweaks`): `add_tweaks` = VsAIWorld::AddTweaks
  0x2D360 (25 systems "AI", "AI.Mistakes.Skid out", ... "AI.StartlineRevs", 151 tweaks, run at the end of the
  AiWorldData step = LoadWorldData 0x30348) and `add_tft_tweaks` = 0x16C30 (top-level "TFT", 7 tweaks, TftManager
  step = VsAIDriver 0x16820; TFTCharacter enum = "<opponent name> = i, ..." over the ai cars). Names, kinds,
  ranges (f32 bits), steps, order as the binary: test `ai_tweaks::tests::registered_tree_matches_the_original`
  compares against `src/ai_tweaks_original.txt`, generated from the decompiles by `tools/ai_tweak_table.py`.
  Original quirks kept: "Min brake late corner tightness" max 0, "AI.Nitrous.high Skill", empty "AI.Data".
  Wiring: each tweak's `TweakCell` is bound by an accessor to the variable in `AiWorld` (`AiTweaks` resource);
  `tweaks_out` (cells → ai, first in the Ai-set chain) / `tweaks_in` (ai → cells, after `update`, so the ai's own
  writes such as TightCornerThreshold show). The statics (tweak block 0x6D3698, TFT manager 0x6D26E0, slowdown
  0x684E58..60, now `VsAIGlobals.slowdown_*` read by 0x19370) persist across races: `AiWorld::init_with` pushes
  them into each new world before Init reads them. PlayerAISpeedRC "0.0".."1.0" = y of points 0..8 of race car 0's
  speed-ratio curve (0x15DB0(rm,0) → handling +4 → curve +4 items, +4+8k; shared by every 350Z car), reread from
  the per-race curve at each registration. Root order: head insertion, so the frontend's later in-game "Game" /
  "Player Info" come before TFT and AI. Headless check (`SMOKE_DIRECT=1 SMOKE_DEBUG_MENU=1`, log
  `research/ai_review/ai_tweaks_smoke.log`) runs clean but the smoke's fixed walk enters "Game", not an ai set: no ai
  screenshot yet; request for a configurable path filed in core.md.
- Added before: `VsRaceManager::reset_scores` (0x3DA60, RestartRace after VsAIWorld::Reset, both plugin reset
  paths); 0x34DB0 is `find_section_by_lane_id` (set B).

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
  `InGameUpdateSet::Ai`: RaceSessionMessage::ResetToGrid, then `AiWorld::update` (1/24 s), writes
  `RaceSession.race_finished`; after `InGameUpdateSet::Physics` in the same tick: `PhysicsCollision` →
  `AiCollisionMessage` → `on_collision` (messages do not survive to the next tick). `AiPhysics` is installed automatically over
  `PhysicsWorldRes` (`physics_ai::PhysicsAi`, moved from velocity_physics tests; handle = physics vehicle id) when
  the physics plugin is present; optional `AiHostResource`, `AiTftPlayer`, `AiRespawns`. Opponents = the current
  race's non-human competitor slots by opponent crc (database order without an event).
- Binding (`bind_race_cars`, PhysicsManager 0x19E080): race car with ai id N ↔ physics vehicle N once it exists.
  Players: `SetPlayer`, `SetHumanControl(1)` (`set_human_control_on`), control mode 0; opponents: base +0x20
  (active), control mode 1 (4 network). The AI does not update until the players are bound (the original creates
  every car first; an update with no active car would mark the race finished), and a `ResetToGrid` that arrives
  earlier is replayed then. After each update `AiPhysicsWorld::end_update` writes the human-control flags
  (VehicleManager__1DD9F0) and flushes the controls blocks. Glue: traffic/police are switched off (warning)
  while no physics cars exist for them.
- **Controls block is shared state** (VsAIVehicle 0x2C7B0 returns VE+0x9C itself; VsAIRaceVehicle +40 points at
  it). For a human car ComputeForcesAndTorques 0x1C5800 runs 0x1B82A0 on that block every physics frame: steer/
  throttle/brake step from their **previous** values towards the pad (0x1B8260 clamp step), +0x15 is a reverse
  latch, and handbrake becomes 0.2 when |speed| < 1, throttle < 0.01 and handbrake < 0.2. The ai writes a human
  car's block only in 0x18A40 (failure 0 → throttle 0; any failure → throttle/brake 0). `PhysicsAi` therefore
  reloads every proxy from physics in `begin_access` (start of `with_ctx`) and writes back only the blocks taken
  through `controls()` (`dirty`) in `end_access`/`end_update`. The old flush wrote a never-reloaded copy into every
  car each update, which reset the human car's pad state (player stood at the grid with ai cars present). Smoke
  quick race and `SMOKE_CAREER=1`: player 0 → 27/39 m/s; clip `research/video/ai_drive.mp4`.
- Race counts (`race_state`, VsRaceManager 0x3AD10 / VsAIWorld::Init 0x2C960): `AiEventInfo` is the current
  event race entry (`dword_6F9BE8`: type +0x04, laps +0x14, **competitors +0x15 incl. humans**; eventdef only as
  fallback); ai cars = competitors − `human_count` (0x91B70) when `+0x4767` (`byte_6F70D8`, see "Split screen")
  else 0; traffic ids start at
  the competitor count. Per update: countdown = `RaceData.flag_girl` `in_game_time`/`allow_go_time` (timeline
  times compared with the ai's own tick clock since AI init — assumed aligned as the frame timer restarts at race
  load), speech `gm.vars.speech`, network `gm.vars.network_race`. `TriggerMessage::AiHint` → `set_checkpoint`
  (0x21980) on the race clock, read after the Physics set (`trigger_hints`).
- Walls (physics' request): `barrier_render::build_collision_mesh` = BarrierRender 0x108030 (arrow then no-entry
  lines, kind byte = header flags[0], 1 skipped; vertices y and y+4 (`dword_68656C`); side 2 = +1, 3 = −1, 0 = vote
  per point: nearest raceline segment over all sections by **distance to the segment start** (original), its
  midpoint, `IsLeftOfLine(p[i], p[(i+1)%n], mid)` left −1 else +1; faces 2n−2 per line via AddToFaceList 0x7F8C0
  alternating from `side >= 0`, shared first-vertex lookup 0x7F7A0). Armco kind 0 has no vote (always +1). Built
  on the AiWorldData step (barrier unless `CommandLine.no_barriers`, armco when loaded), held in
  `AiState.pending_walls` and written as `SetPhysicsWalls` from the Ai set once `PhysicsWorldRes` exists (same tick
  as physics' `set_walls`). 39 shipped races build a mesh; London C/R01 faces 1070/1140 look towards the raceline.
- Score (`VsAIRaceVehicle.score` = +0x1C8, `ScoreManager::new(local)` in `init` 0x17F60, players local; reset
  0x168F70 in `reset`): `update_score` in `update` (0x18A40, after debug-select) runs core's `ScoreManager::update`
  in race phases 4/5 with `ScoreInputs` from the car/scene (ahead car's split time = its stats[3+split] via the new
  `AiVehicleView.stats`), events to `AiHost::score_event`, then `set_score_gains(Some((zone, nos)))` each update
  for race-manager cars (police race part: none, 0x3D640 does not find it). `start_switch` / `race_switch` =
  `AiPhysicsVehicle::launch_result` / `shift_result` (ScoreManagerChecks 0x16CAB0/0x16CBD0 → 0x16CD50/0x16CD70 on
  the car's physics vehicle); 0x169E30 clears the shift result (0x16AFE0) after scoring it, inside its racing
  (+244) branch → `clear_shift_result` when `racing && race_switch != 0`. Not fed: slipstream strength (LightStreak
  0x130070, render), chased-by-police.
  `on_collision`: the score manager of car `a` (message's first id, 0x169240). 0x169940 (OSD gap fade) not ported.
- Binding also covers traffic (control mode 2) and police (mode 1, both parts) by their ai ids once physics has
  those vehicles; a manager whose cars are missing after the players bind is switched off (core spawns none yet).
  As in 0x19E080: traffic = mode 2, bind (0x2BD00), `set_traffic_index(i)` (VE+0x13D8, i = manager slot 0..13);
  police = mode 1, bind (0x177E0), `police_handling()` once (VE+3220 ×2, LC +264/+92 ×2, +80/+76 ×1.5).
- `race_car_source::AiRaceCarSource` (installed as `RaceCarSourceRes` by AiPlugin when absent): opponent car =
  `OpponentDatabase::build_vehicle_setup` (OpponentDatabase.cpp 0xE72C0, moved from the binary), team and police
  car glue (same as core's `velocity/src/race_car_source.rs`, which can go). `team_car` verified against
  0x19E45C: categories 2/3 without event flag 8, `find_team(opponent +0x24)`, slot = team `+0x00` = the setup
  TeamDatabase 0x16EC30 builds for the vehicle named by the **team crc** (+0x04) when FindVehicle succeeds (null
  otherwise → opponent setup `laps-1`); the +0x1C team car is only StartEvent's (`team_car_by_team`).
- Traffic code region misattribution: traffic vehicle 0x29830..0x29EA0 is labelled VsAITrafficLight, light
  reset/update 0x29570/0x295A0 is labelled VsAITools. Traffic car layout 332 B: +0x18 cruise 14, +0x1C max
  speed, +0xF0 traffic.bin entry (type bits at +0x0C: 1 ambient, 2 fire engine), +0xF4 lane, +0x100 lookahead,
  +0x130 dist, +0x134 next blocker, +0x13C behaviour, +0x144 end mode (1 stop, 2 despawn), +0x148 lane offset.
  Lane (100 B): +0x28 speed, +0x2C car count, +0x54/+0x58 race section/dist (never set by ai code), +0x5C blockers.
- **No corner braking is ORIGINAL behaviour** (re-verified against the binary): 0x1C010 = `threshold(0x6D3814) =
  (float)GetSkillLevel * 0.5 + DifficultyScale(0x6D3880)`, tightness = cube of the sum of `GetSteerTowards`
  (0x2C1B0, cross length clamped to pi/8 = 0x3EC90FDB) to lookahead points 1 and 2 → |ct| <= 0.4845. 0x1C1C0
  takes the curve branch only when |ct| >= threshold. DifficultyScale = 0.6 (0x36E00 stores 0x625310 =
  0x3F19999A; AddTweaks 0x2D3D8 registers it 0..1 step 0.05, never set by autoexec.cli or any data file);
  GetSkillLevel (0x15330) = 1 for players, else OPPONENT.BIN +0x4C, which is 0..2 for all 31 opponents. So the
  minimum threshold is 0.6 and the branch is dead; "TightCornerThreshold" (0x2F3D3) is a tweak on the same
  global but is overwritten every update. Only lowering Difficulty Scale below 0.4845 in the debug menu makes
  skill-0 drivers brake (`LAPDIFF=0` in the lap test: no lap-time gain). Race speed control otherwise comes only
  from speed match (+0x0C/+0x10: following, police, finish), the physics car and catch-up throttle (up to 2.0).
- **Recover loop is ORIGINAL logic** (0x20490 stuck: |speed| < 1.5 for 2 s after the grace time; 0x1E510 brake
  = reverse, steer `-GetSteerTowards(lookahead 0)`; 0x205A0 leave when |speed| > 5 and forward·dir(lookahead 0)
  > 0.9, clearing the stuck flag, +4 s grace; 0x206A0 respawn after 9 s stuck). A car pinned nose-first on the
  outside of a hairpin already faces the lookahead, so it backs out to 5 m/s and drives straight back in; leaving
  clears the stuck time, so the 9 s respawn never fires. Speed +0x90 is signed (negative reversing), as ported.
  On FastCar the scraping also posts a collision message most frames (~14/s per car); the damage reaches the
  max (~1000) after ~2 min and the car fails (failure 5): 0x22FA0 then zeroes its controls and skips the state
  machine, so it stands for good (also original).
  Crate root: `crc`, `text_file` (VsStringAnsi reader), `opponent_database`, `team_database`.
- **AI on physics cars (London Circuit Race01, `tests/ai_lap.rs`)**: the AI never brakes for corners in the
  original either (0x1C1C0 compare verified in asm; threshold only written by the 0x36E00 ctor and AddTweaks;
  handling steering curve/factor/damping are loaded but read nowhere). What decides the corners is the car's
  setup: build test cars like PhysicsManager 0x19E080 (`LocalConfigInputs::from_setup` of the opponent's
  `build_vehicle_setup` / a stock setup) — every shipped setup has tyre softness >= 0.5 (opponents 0.6), while
  `default_spans` gives 0 (hard tyres: ~half the lateral grip), which is what put cars off at the first S-bend.
  55 m/s is the 350Z limit at top-speed PM 0 (155 mph * 0.8). Also set control modes (0 player, 1 opponents).
  Result: laps 166-193 s (was 219-232), 99-187 m off-road per car; remaining losses are oversteer spins at
  full lock (ct pinned 0.4845) and a crest before a 75-degree right at ~d1395.
  **Model wheel spans** (physics' `from_setup` now reads the 350Z's `f_l_wheel`/`r_l_wheel` locators, 2026-11-05):
  all 4 `ai_lap` tests pass unchanged. Pack: player 189.4 s, opponents 180.7 / 175.3 / 182.3 s (102-195 m off-road,
  122 of 386 collision messages with walls); player alone 183.2 s (151 m off-road, 224 recover frames); respawns
  stuck 9.17 s / flipped 3.12 s after the disturbance, 0.0 m off the raceline. No original reference: London Circuit
  Race01's EventDef "Lap time" is (0) (only Race11-15 / LADown cruise carry 60/120 s limits).
- Region misattribution in the idb: VsAIRaceVehicle ctor/init/reset are at 0x17E40/18000/18190 (Police region),
  driver ctor/reset/speak in Armco region (0x14350..0x146E0), respawn in TrafficVehicle region (0x2B800..2BB30),
  `VsAIDriver::SetOpponent` is 0x1B4120 (PhysicsManager region): opponent=+4, temperament(+0x3C)=opponent+0x48.
- Race setup lives in game code: PhysicsManager 0x19E080 binds physics cars (`set_physics_vehicle`,
  `set_player(true)`, `set_human_control`) and opponents; GameManager teleports (activates) cars before
  `VsRaceManager::reset_to_grid` (0x3C5D0 only resets active ai cars).
- Inputs the ai needs are traits in `ai_context.rs` (`AiPhysicsWorld`, `AiPhysicsVehicle`, `AiHost`,
  `AiGameState`, `AiEventInfo`); `VehicleControls` mirrors VehicleEntity+0x9C (steer +0, throttle +8,
  brake +0xC, handbrake +0x10, nitrous +0x14; defaults per 0x1B7D90).

- **Start line**: ProcessSplineData 0x31040 snaps a circular raceline to the finish line (`dword_6F5C44/54`) and
  then copies it over `dword_6F5C24/34`, so on circuits `start_line_original` = finish line (plugin does this after
  `load_world_data`). Every LACoast EventDef is a placeholder (0 grid slots, both lines at the origin): the original
  breaks into the debugger there (ProjectOntoLine "VecA != VecB" from LoadEventDefinition 0xA3300 and from
  SetStartPoint; assert handler 0x2A76E0 returns 3 -> int3). The plugin passes a degenerate line as `None` (warns):
  no snap, no lap counting.
- **Cruise**: no AISpline, so no race sections. The original still creates race cars (VsRaceManager::Init 0x3AD10:
  ai = competitors − humans when single player, no race-type gate), skips the race manager update without sections
  (VsAIWorld::Update 0x34280, ported), but GameManager 0xBC5F0 calls ResetToGrid every race → 0x20DF0 → respawn
  0x2BB30 → SetRaceSection(GetFirstRaceSection() = NULL) → GetSubRoutePoint reads through NULL (+0x18): the
  original does not avoid the path. Port: `set_race_section` with a missing section stores None section/sub point.
- **Police skill**: police drivers never get an opponent (0x38C10/0x38D30; SetOpponent only from PhysicsManager),
  so GetSkillLevel 0x15330 reads an unset pointer in the original; the port's AIA_NOT_SET (4) indexes past the
  3-entry catchup/slipstream tables, so those two use the no-driver default 1.
- Sweep test `lap_test::ai_sweep_all_races` (`--ignored`, `SWEEP=<substr>` filter): every race with an AISpline
  (59), 3 opponents + traffic + police, 90 s, synthetic start line; all pass. Smoke runs pass for LACoast/London
  cruise, LADown/London/Tokyo circuit, LACoast circuit and sprint.

### AI review (headless smokes, 2026-11-05)
- `VELOCITY_AI_TRACE=<s>` (plugin `trace`, review aid, not original): every period logs `aitrace` lines: race order,
  per car state/pos/speed/race distance/lap/position/raceline lateral/section/failure/stuck time/offset
  (active/current/target)/rival side/slipstream target/damage/human, traffic active count + red-blocker bit string,
  police state/target/wanted and each cop's mode/state/siren. Logs + traces of the review in `research/ai_review/`.
- **London Circuit Race09 crash (core sweep) fixed**: section 1 branches at section 0's first raceline point, which
  the start snap (SetStartPoint, finish line) moves along the line, so the branch lies just before segment 0 (t<0)
  and ProcessRaceSections 0x31390 finds no section. The original asserts "pClosestSection != NULL" (line 1043) and then
  reads through NULL; the port warns and takes the nearest line ignoring t. Test
  `every_race_processes_with_its_finish_line` (eventdef finish line, as the plugin) — the only such race.
- Seen, all consistent with the binary (re-checked 0x1F6D0 follow/overtake + constants 0x6D2CAC.. set by 0x250A0,
  0x19370 slowdown 75..200 m ×0.5, 0x19480 catch-up >25 m behind, 0x22970 rival, 0x208D0 respawn, 0x295A0 lights,
  0x39D80/0x39E70 police alerts, 0x3FD40 free vehicle incl. its position-dot quirk):
  LADown circuit 120 s: raceline |lateral| avg 1.3-1.8 m (max 6.4), 17 position changes, overtake offsets active
  5-11% of samples, 14 traffic cars, lights cycling; a traffic cop alerted at 20 m (player 41 m/s), chased 40 s at
  up to 44 m/s within 20-130 m (never within 5 m, so wanted stays 1.05, no Chase), gave up at ~190 m → Patrol.
  London sprint: |lat| avg 2.7-3.4, offsets 25-49 of 68 samples, slipstream, rival weaving 27 samples. LADown drag:
  traffic on, no police (as Init), lights cycle, 2 ai finish. Wet London circuit Race01: one car takes race section 1 (shortcut); run ends in the physics crash below.
  No arrest flow exists (no strings/code in the XBE): a chase ends only by lost target. Respawns: see below.
- Cruise has no AISpline (only EventDef.txt in every Cruise dir): no traffic sections, so no traffic and no police in
  cruise in the original data either; the "cruise with police" case is only reachable on races with a spline.
- **"AI-driven player stops at ~240 s" (frontend report, London sprint, 2026-11-06): not reproducible.** Three
  `SMOKE_REVIEW=1 SMOKE_AREA=2 SMOKE_RACE_TYPE=2` runs (`SMOKE_RACE=1` → LONDON2 Sprint Race01, none → LONDON Sprint
  Race02; logs `research/ai_review/sprint240{,b,c}.log`, trace 10 s) all finish at race time 152-159 s, hc false all
  race, no stalls. The game runs at ~0.63x real time beside the core sweep, so the finish lands at smoke wall time
  295-306 s, right at the smoke's 300 s limit (no `SMOKE_DRIVE`): run c timed out 5 s into PostRace. After the finish
  the car stands by design (Finished state, speed match to 0). Use `SMOKE_DRIVE=400` for review runs. In review mode
  the smoke's quit path (dt > SMOKE_DRIVE) presses START, which the zeroed review keys swallow ("not paused" spam).
- Physics crash (0xCCCCCCCC contact point → NaN camera, wet London) filed in docs/physics.md. Clip
  `research/video/ai_review.mp4` (London Sprint Race02, player behind an opponent).

### Split screen (two human players) — checked against the XBE
- `byte_6F70D8` (VsAIWorld::Init arg 2 → `+0x4767`, getter VsRaceManager 0x3DD30) is **not "split screen"**: writers
  are GamePlayer 0xC1930 (=1), main menu quick race 0x23CE80 (=1), debug menu 0xACFA0 (=1), the online match
  screen 0x217D70, network 0xCF520/0xD73D0/0xD7510 and CGameJump::Execute 0xC29D4 (= 0 only with the jump's
  split screen +0x204, `dword_6F5C00` = 2). The front-end split screen (MultiplayerScreen entry 2) leaves it 1, so a
  front-end two-player race seats **competitors − 2 ai cars**; only a split-screen *jump* has none (core's
  `RaceSelection.splitscreen` = that jump flag, which 0xC2EC0 clears anyway). The plugin's
  `if race.splitscreen { 0 }` therefore matches. Other readers of 0x3DD30 are render code (0x5C220, 0x110220,
  0x1326A0, 0x152860).
- Already handled for n players: race manager players/`player_positions`/player drivers sized by `num_players`, ids
  players first (physics id = player index), `bind_race_cars` binds every player (set_player, human control,
  control mode 0), score managers per car, traffic spawn/despawn on the closest player (GetPlayerProximity 0x344E0
  per player, 0x34600/0x34690 loop), TFT `play_tft(name, player)`.
- **Player 0 only in the original** (`VsAIDriver__15DB0(rm, 0)`, kept): rubber-banding slowdown 0x19370 and catch-up
  0x19480, overtake speech 0x14F90 (player 0's position), the NOS flash distance 0x19740, the pursuit PlayerAssist hand-over (players[0]), police targeting/alerts 0x38DC0 (`+0x50` = player 0) / 0x39D80 / 0x39E70, traffic light 0x29EA0, the
  respawn's crash camera reset + point-of-interest clear 0x2BB30 (player 1's respawn only cuts its own camera, +0x78),
  one global respawn fade. So with two humans the ai rubber-bands against player 0 and the police chase only player 0.
- Test `lap_test::split_screen_two_players_and_nos_flash` (London Circuit Race01, 2 players under PlayerAIControl +
  2 ai, 40 s): both players progress (~1940 m) and hold distinct `player_positions` ([1, 2]).
- Depends on core (filed in core.md): `RaceProgress` carries only player 0's place/state; player 1's HUD needs
  per-player `player_positions` / `player_states` (the ai fills them once the fields exist). Everything else for a
  split-screen race (cameras, pad 1 input, second view) is core/graphics/frontend.

### Restarts and drag rounds (2026-11-05, `research/ai_dump/drag_rounds.c`, `_bc5f0.c`, `_bf270.c`)
- **RestartRace 0x000BE0F0 calls VsAIWorld::Reset (0x2CC90)**, which the plugin never did: after a restart (pause
  menu, next drag round) `race_finished`, laps, distances and failures carried over, so a drag round 2 ended at once.
  `session_messages` now reads `GameModeMessage::RaceRestart` (written by GameManager::Update in the Input set of the
  same tick, before MonitorSessions' ResetToGrid) and resets the world; before the cars are bound (the race's first
  RestartRace from InGameInit) it resets with a null physics context.
- Drag elimination: MonitorSessions state 5 (drag) sets GameManager **+0x14C = VsRaceManager 0x3DB70** (last active
  car in the order) when the player qualified; a human there is the ELIMINATED branch (+0x14C = 0). RestartRace then
  deactivates it (VsAIVehicle 0x2C0F0) after the reset. Port: on `RaceSessionMessage::DragResult(WINNER|QUALIFIED)`
  `AiState.drag_eliminated = AiWorld::drag_eliminated_car()`; on the next RaceRestart with `RaceSession.drag_round > 0`
  (a continued round keeps it) → `drag_deactivate`. Core's `RaceSession.drag_winner` ("+0x14C round winner") is that
  car and is never set by core; the ai keeps it itself.
- GameManager 0x000BF270 (drag restart from the menus): every inactive ai racer → VsAIVehicle 0x2BDA0(VS_ZERO,
  identity 0x2B6E50, VS_ZERO), +0x14C = 0, then RestartRace. Port: RaceRestart in a drag race with `drag_round == 0`
  → `drag_reactivate` before the reset (unbound cars skipped).
- Other drag-only ai code (verified): 0x1E400 stop at `laps * lap length + 3` (`time_trial_stop`), 0x22FE0 players
  take damage failures only in drag, 0x3CBC0 finish check ignores a failed player in drag, 0x192F0 is an empty getter
  in this build, Init: no police in drag. PostRaceSequence 0xC3DE0 also reactivates inactive ai cars (core's
  `PostRaceEvent::ParkOthers`, which nobody consumes; unreachable: no shipped POST_RACE_STUNT_AREA marker).
- Smoke (`SMOKE_DIRECT=1 SMOKE_RACE_TYPE=5 SMOKE_REVIEW=1`, `research/ai_review/drag_rounds.log`): round 1 finishes
  at 42.2 s with the player last (ELIMINATED, no car put out); core's round 2 then reloads the race and the ai world
  (filed in core.md), so the in-place elimination is only covered by the test below so far.
- Test `lap_test::drag_rounds_put_the_last_car_out` (LADown Drag Race01, 3 opponents): round 1 finishes, the last car
  is put out, stays inactive and last in the order through round 2, `drag_reactivate` puts it back on its grid slot.
- Opponent database for events re-checked: competitor +4 = OpponentDatabase 0xE7AA0 by crc (EventDatabaseEvents
  0x92070), PhysicsManager 0x19E080 gives the race's non-human slots to the ai cars in order (as ported); the list is
  append-order (0xE81C0), count/get 0xE7B50/0xE7C20 filter group (0 = any) and flags (0 = any) as ported. The temp
  event builder 0x93160 also sets opponent flag 4 on a random group-0..9 pick per slot; nothing in the ai reads it.

### Respawn flow (re-checked against the XBE, `research/ai_dump/respawn_dis.txt`, `respawn_208D0_code.c`)
- Upside-down 0x00020700 (every update): exempt when VE+0x1294 == 1 and float VE+0x11F4 < 10.0 (flag cleared);
  bad = base velocity.y (+0x80) < -20.0 (0x625754) or body up (body +0x80, 0x248C0(1)) · +y <= 0.3 (0x625750).
  First bad frame: flag +0x324, deadline +0x328 = AI time + 3.0 (0x62550C); good again (not falling and upright)
  clears it; past the deadline → 0x208D0(speed 0). No fade on this path (fades only from 0x22C90 / the respawn
  trigger branch of 0x18A40, see "Respawn fade").
- Stuck: 0x20490 |speed| < 1.5 → stuck time, Recover after 2 s, 0x206A0 respawn 9 s after the stuck time.
- 0x208D0: clear failure (0x19D40(1)); with a raceline: heading = pi/2 - angle(+Z, flat raceline segment dir),
  stuck flag off, grace = time + 2, raceline point (0x21430) probed k = 1..5 from ±3k m (line check top→bottom),
  y -= VE+0x810 - (body 0x35FB40(0) - 0.1 (0x684E78)); else grid slot 0 with angle **90.0 radians** (original
  bug, kept). Then +952/+956/+960 = 0. 0x2BB30: reset dynamics, camera cut 0x2C620 (`AiCameraCut` +
  `GameMessage::CameraCut`), teleport, car 0: crash camera reset + point of interest cleared (`AiHost`), speed →
  velocity + commit (car vtbl +0x20), 0x1D1810 history scale, section 0 and UpdateState(1).
- Tests (`tests/ai_lap.rs`, London Circuit Race01, one opponent): `stuck_opponent_respawns_on_raceline` (velocities
  zeroed from t0: respawn 9.0-9.5 s later on the raceline, upright, drives on > 10 m/s) and
  `upside_down_opponent_respawns_on_raceline` (flipped at t0: respawn 3.0-3.5 s later). Measured 9.17 s / 3.12 s,
  0.0 m off the raceline, up.y 1.0, 33 m/s afterwards. The respawn is detected by the grace time (+0x320) changing:
  the raceline point is the car's own projection, so the teleport can be < 1 m. Integration tests need
  `VELOCITY_GAME_DIR` (cargo test runs from the deps dir); without a game dir they return early, with one they fail
  when no respawn happens.
- Core sweep (`research/sweep/logs`): player respawns on LACoast placeholder races happen before the start (car at
  the origin grid, falls: unreachable in the original, which int3s on those eventdefs); LADown Circuit Race09 wet
  player fell through the world (y -90, falling branch, correct); LADown Race04 opponent 1 stuck → respawned.

### Explosions and the ai (checked 2026-11-06)
- TimelineExplosion 0x0017ACD0 → PhysicsManager 0x0019FD80 reaches **no ai function** (car velocity/angular kick,
  message 2 to objects, VE+4876 = 96 via 0x001D1170, which the ai never reads). A pushed ai car is handled only by
  the existing recover paths. Callers of respawn 0x208D0: 0x1AFE0 (stuck state), 0x20700 (upside down), TimelineAI
  0x176D90, **VsCrashCamera 0x00062D90**; of 0x2BB30: 0x18A40, 0x20DF0, traffic 0x2BAA0, TriggerUpdate 0x180310.
- Test `ai_lap::exploded_opponent_recovers` (London C/R01, real physics, `LAPDBG=1` traces): side blast (force 40)
  → 33.7 m/s kick, the car drives on (2.7 m off the raceline, 51 m/s 30 s later, no respawn). Blast under the car
  (force 60, angular 20) → 55 m/s, thrown ~50 m up onto a ledge 7 m above the road: upside-down (3 s) clears each
  time the body is briefly upright mid-air (as 0x20700), and on the ledge the recover loop (stuck → Recover → leave
  at 5 m/s → stuck) keeps resetting the stuck time, so no respawn in 30 s (original logic, "Recover loop" above).
- **Crash camera respawn was dropped**: 0x62D90 calls 0x208D0(crashed car, `flt_6850F0`) when the crash cam
  failed or `respawn_on_raceline && max speed >= speed (0x15C60)`; core's `vs_crash_camera::stop` emits
  `CrashCameraEvent::Respawn` and `apply_crash_events` ignores it. Ai side ready: message
  `plugin::AiRespawnOnRaceline { vehicle, speed }` → `VsRaceManager::crash_camera_respawn` (find by +0x78 id, 0x208D0),
  applied at the start of the Ai set (crash cam runs in CrashCam, before it). Request filed in core.md.

### Timeline AI events and race progress (core's requests, done)
- `timeline_objects` (GameTick, after `InGameUpdateSet::Timeline`) applies `TimelineObjectMessage`:
  EnableRaceSection → `AiWorld::enable_race_section` (race section `+0x25` = `flag_25`, 0x00177B20; the sub-route
  test 0x1A930 reads it), RespawnRacersOnRaceline → `respawn_players_on_raceline` (0x00176D90: **player** cars only,
  0x208D0(value)), PlayerHumanControl → `player_human_control` (0x00176F00, one player only, 0x18920),
  Respawn → `timeline_respawn` (0x00177150: active players then active ai cars, inside bit `1 << id` or human,
  respawn points used in turn even when missing; ground probe ±10 m, +0.75 up, heading = euler y of the trigger
  rotation (0x2B7A30: `-(acos(-2(xz - yw)) - pi/2)`) + pi (0x625544), speed = RespawnSpeed, then section 0 +
  UpdateState(1)).
- `RaceProgress` (filled after every Ai-set update, `fill_progress`): `order` = event competitor index of the car
  in each place (race vehicle +0xF0 = competitor, 0x19350 from 0x19E080: human race slots to the players in order,
  the others to the ai cars; GetVehicleInPosition 0x34770 = race_order[place-1]), `order_ids`, `player_position`
  (dPlayerPos[0]), `player_state` = player 0 failure +0x260 (0x23CE0, 6 none), `racers` (+0x40), `active_racers`
  (0x3DCA0), `leader_human` (0x3DB70 last active car in the order, +0x34C), `racer_records` = players' stats
  [11]/[20] (0xBE430). `results_busy` / `car_flags` are not the ai's and are left alone.

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

### Requests from physics — status

Done (physics' answer below is wired: launch/shift results, clear, traffic index, police handling): adapter moved (`src/physics_ai.rs`), plugin wiring, collisions, lap test moved (`tests/ai_lap.rs`); corner
braking and the recover loop verified as original (above); barrier/Armco walls sent, control modes (0/1/2/4) set
on bind, score gains fed each update (`AiPhysicsVehicle::set_score_gains`, `wheel_contact_normal`). The physics
copies (`velocity_physics/tests/ai_support`, `tests/ai_lap.rs`) can be deleted by their owner. Wanted from
physics: the gear-switch results and the police handling scaling above.
**Physics' answer** (`velocity_physics::vehicle::vehicle_entity::sim_ai`, `PhysicsWorld`): `launch_result(id)` =
VE+0x1338 (0x001D1270, ScoreManagerChecks 0x0016CD50; only reset by PhysicsCarInput 0x001B9940), `shift_result(id)` =
VE+0x133C (0x001D1480: 1 early/late, 2 good, 3 perfect; 0x0016CD70) and `clear_shift_result(id)` = ScoreManager
0x0016AFE0 (called from 0x00169E30 after scoring it); `set_traffic_index(id, i)` = 0x0019E080's VE+0x13D8;
`police_handling(id)` = 0x0019E080's police scaling (VE+3220 ×2, LC +264/+92 ×2, +80/+76 ×1.5), call once after
binding the police car (control mode 1). The trailblazer's physics events are forwarded by core (docs/core.md).

### Real-physics lap test (`tests/ai_lap.rs`)

London Circuit Race01, eventdef grid, 350Z configs, player under PlayerAIControl + opponents AL, BUD, JULIO
(quick-race temp event names), 400 s at 24 Hz (~8 s release). Player alone: no lap, stuck at the first hairpin
(1147 m), 342 m driven off-road, worst 17.5 m past the edge. Pack: player lap 230 s, JULIO 219 s, BUD 232 s, AL
stuck at the hairpin and wrecked (failure 5); 140-770 m driven off-road per car, worst 22-55 m. With the barrier
walls (1140 faces, default; `LAPWALLS=0` off) the results are identical and no car touches a wall: the barriers
close side streets 5-20 m off the raceline, while the cars stick against track geometry at the hairpin
((-1541, -30), race distance ~1150) and recover/drive back in (player 965 recover frames). `LAPDBG=1`
(`LAPALL=1`) prints per-car state, `LAPDIFF=x` overrides Difficulty Scale. Plugin test
`jumped_race_binds_physics_cars`: a London jump with the physics plugin binds car 0 as human at update 4.

### Headless lap test (`src/ai/lap_test.rs`)

Kinematic bicycle stand-in (no slip, 200 m/s^2 lateral cap) implementing `AiPhysicsWorld`; one player with
`player_ai_control` plus 0/3 opponents. London Circuit Race01 laps in ~85 s, LADown Circuit Race01 in ~52 s
with 14 traffic cars (90 spawns, lights cycling, no red-blocker runs, traffic within 2.6 m of its lane, a police
car joins the traffic); race cars stay within 6 m of the edge lines (corner apex cutting). Pursuit test: both
police cars leave the grid chasing a scripted player and stay within 11 m of it over 60 s.
`plugin::tests`: a `jumptogame` boot builds the AI world from the core's load steps.

### TrailBlazer runner (core's `game_mode::trail_blazer::TrailBlazer`)
- `VsAIRaceVehicle.trail_blazer` (+0x3C8) is built by 0x00023250 for players only, from `set_player` (0x00023110)
  and `reset` (0x00018190, after SetupMaxDamage); the old one is dropped (0x000235E0, empty dtor 0x000C4850).
- Start (TriggerUpdate 0x00180310, type 0x9C6D28D5, player cars only): `trigger_hints` takes
  `TriggerMessage::Trailblazer` → `start_trail_blazer`: VsAIWorld 0x00034F70/0x00034E80 finds the trailblazer section
  (`+0x478C`, 12 B) having **any lane** with id = the trigger's target crc and follows that section's **first** lane
  line (`section+8`); "Trail blazer spline %x not found" otherwise. `saved` = `AiPhysicsWorld::trail_blazer_globals`
  (PhysicsAi reads GC +0x84/+0x90/+0xD4/+0xD0/+0xCC and glob 0x686B58 through physics' `Cx`).
- Update: `VsAIRaceVehicle::update` (0x00018A40) runs it right after the post-race switch, before the startline revs,
  with the update's dt (now a parameter of `update`). Car = base +0x98 position, +0x90 speed, +0xC8 forward.
  Failure (0x000C4C90): delay = respawn trigger's **params[2]** (+0x48, 0x000C6220) / 1000 → 0x00022C90
  `schedule_respawn` inside the ai (the `TrailEvent::Respawn` is still forwarded; core must not respawn again).
- Events collect in `VsAIRaceVehicle.trail_events` and go out as `TrailBlazerMessage { vehicle: id }` from the Ai-set
  `update` and from `trigger_hints` (after Physics), both before core's Effects set. Handlers for CarFlag/CarReset/
  PhysicsGlobals (physics), TimeStretch/Timeline*/Message/Audio*/Vibrate (core) are not the ai's.
- `RespawnSource` had no implementation: the plugin now falls back to `TriggerRespawns` over core's `CTriggers`
  (0x00181AB0 by name crc; pos/rot, params[0] `delay_ms`, [1] `speed`, [2] `param_2`) when `AiRespawns` is absent.
- Shipped trailblazer triggers: only LADown Circuit Race02 (2), Race04 (2), Race09 (1) (`triggers.trg`).
  Test `lap_test::trail_blazer_follows_ladown_spline`: Race02 spline 0x3E7ED980 found and followed to success.
  The smoke race (LADown Race01) has none, so smoke runs only show that nothing regresses.

### Respawn fade (core's request, done)
- `AiHost::fade_start(t)` at both DangerZoneRender 0x00123070 (`flt_7190FC`) calls: `schedule_respawn` 0x22C90
  (`t = delay`, after +952, before the failure) and `update` 0x18A40's respawn-trigger branch (`t = 0 - delay_ms/1000`
  when that delay >= 1e-6, after SetHumanControl(0) and +960). The plugin wraps the host (`FadeHost`), collects the
  calls in `AiState.fades` and the Ai-set update writes them as `FadeStart(t)` the same tick. One global fade for
  whichever vehicle respawns (quirk kept).

### Career teams (`TeamDatabase`, plugin `AiTeams`)
- `Team` +0x1C = `vehicle_crc` (the team car, FEHangoutProgression 0x00290CD0); +0x3C flags: bit0 **unlocked**
  (0x00093D30 `is_locked` = bit0 clear), bit1 offering invites, 0x20 newly unlocked. `find_team` = VsCommonMain
  0x0016E610. `RaceCarSource::team_car_by_team` = StartEvent 0x0028F430: `SetupDefaultVehicle(team+0x1C)`; every
  shipped team resolves (test `every_team_has_a_car`). `team_car` (opponent's team) sets up the vehicle named by the
  team crc, not +0x1C (verified against PhysicsManager 0x0019E45C, see `race_car_source` above).
- `AiTeams` (resource = VsCommonMain's `CTeamDatabase`): `career_teams` in `Last` applies
  `RaceSessionMessage::EventOver` (success + unlock team, 0x00091890: if locked |1, |0x20, |2; unknown crc warns)
  and on `CareerMessage::Created` / `Loaded { ok: true }` runs RefreshAllTeamsLockStatus 0x0016E790 (callers
  SetInitialVariables 0x000C1930, SaveFileData 0x001050C0): clear 2 and 1 on every team, then |1 |2 for the
  `unlock_team` of each event with `EVENT_FLAG_COMPLETED`. Career smoke loads PLAYER1 without warnings.
- Team layout for the front end (0x290CB0..0x290DC0): +0x08 name string crc (`unk_08`), +0x1C car, +0x20 invite
  STATUS value (`unk_20`), +0x28 cash bonus (`unk_28`), flags 0x10 joined once, 0x20 newly unlocked; `Team::name` ..
  `clear_newly_unlocked`, `count_teams_offering_invites` (0x16E8B0). The original 0x16E980 keeps writing past a full
  cache (asserts); the port stops.
- Front-end view (done): `career_teams` mirrors `AiTeams` into `velocity_game::career_teams::CareerTeams` (original
  list order = reverse of load order) on its first run and after every change (event unlock, lock refresh,
  request), and applies `TeamRequest::{ClearNewlyUnlocked, SetJoinedOnce}` (0x00290D60 / 0x00290DC0) on the team
  `find_team` returns (unknown crc warns). Test `plugin::tests::team_requests_reach_the_view`.

### Requests from core — done
- Update dt: the plugin passes `GameClock.timer.scaled_dt()` (InGameUpdate 0x000B7C15 → GameManager 0x000BF670,
  dt × stretch) to `AiWorld::update`, `TICK` only without a `GameClock`. `timer_time` (startline revs) stays the
  unstretched tick count. LADown Race02 smoke: stretch 0.1 at 177.6 s → 1 at 179.9 s (~2.3 s, was 0.3 s); one of
  two runs hit an intermittent physics panic (filed in docs/physics.md).
- `SMOKE_AI_DRIVE` / "PlayerAIControl" is original behaviour: 0x006D3698 is read only by AddTweaks, ApplyDebugHandling
  and reset 0x00018190 (`if !PlayerAIControl: human = 1`; it never clears it), while the bind (PhysicsManager
  0x0019E080) calls SetHumanControl(1) for players. So the tweak hands nothing over in the original either; a smoke
  that wants the ai to drive must call `set_human_control(false)` on the player race vehicle after the bind.
- Wet London lens splash: the ai inputs are correct (`get_dist_along` = +0x2A8, set every update for players and
  opponents). Cause is graphics' `puddle_test`: 0x00132A69 reads `this + 48k + 44` = record +24 (`distance`), not a
  zero record +44, so the port always tests only the last and first puddle. Filed in docs/graphics.md.

### Requests from core (name tag colour) — done (`name_colour`, wired in core's `hud.rs`) — done
- `VsAIDriver::name_colour(&self) -> u32` (D3DCOLOR ARGB, 0x00015F80): gradient green 0xFF00FF00 → yellow
  0xFFFFFF00 → red 0xFFFF0000 at positions 0 / 0.5 / 1 over aggression (+0x40) * 0.01; values outside the range
  (and exactly 1) give red (the loop's last-entry early-out). Statics: VISOS_ColourXBOX ctors 0x431AD0..0x431BD0
  (`min(2x, 255)` per byte). Test `vs_ai_driver::tests::name_colour_runs_green_yellow_red`.
- Original request:
- The binary's `hud.rs::name_tags` fills the front end's name tags (COSDInGameScreen 0x000F9340) from
  `race.ai` / `race.drivers[DriverRef::Ai(i)].get_name`; the colour is white for now. Please port
  `VsAIDriver::0x00015F80` (= 0x000156C0 on the driver's +68 colour gradient at `+64 * 0.01`, with 0x00015ED0 and the
  gradient's set-up 0x00015470) as e.g. `VsAIDriver::name_colour(&self) -> u32` (D3DCOLOR); core will pass it through.

### Requests from graphics (police flash for the 3d hud) — done
- `velocity_ai::plugin::PoliceFlash([f32; 4])` (resource, rgb + w weight): the last `police_flash` colour of each
  Ai-set update (the plugin's host wrapper records it), zero by default and from each race's AI init. The frontend
  reads it for the hud tint.
- Original request:
- `VsPoliceManager::flash` (0x0003A1C0) calls `ctx.host.police_flash(&c)`, which every host drops. In the original it
  is OSD3dInterface 0x000EAE90 = the 3d hud's +2912 tint (graphics.md "Light sets": hud ambient = lerp(white,
  c.rgb, c.w)). Please keep the last value in a resource (e.g. `velocity_ai::PoliceFlash(Vec4)`, zero by default and
  on reset) for the frontend's hud lighting (request in frontend.md).
- Frontend: `PoliceFlash` is now read by the 3d hud (`hud::run_osd_3d`), thanks.

### Requests from frontend (cinematic bars off after a respawn) — done
- 0x00018BB6..0x00018BE4 (`respawn_time` +0x3C0 reached, 0x23CC0 not human): SetHumanControl(1) 0x18920, OSDManager
  0x000ED6E0(0) on `dword_6FD200` (bars off), then 0x00023D00(1) on the same OSD manager (`+0x24 &= ~1`, hud flag).
  Host calls `AiHost::cinematic_bars(on)` / `osd_clear_flags(mask)`, recorded by `FadeHost` in `AiState.osd` and
  written by the Ai-set update the same tick as `plugin::AiCinematicBars(bool)` (velocity_ai cannot name the
  frontend's `HudCommand`: frontend depends on velocity_ai) and `RaceSessionMessage::OsdFlags { set: 0, clear: 1 }`
  (read by the frontend's `osd_flags_in`). Core forwards `AiCinematicBars` as `HudCommand::CinematicBars` (core.md).

### Requests from core (point_ahead endless walk, LACOAST Circuit Race05) — answered: original behaviour
- Re-checked 0x000214B0 against the dump: same loop (raceline length test, last sub-route point of the section, +4
  target section, **+16 = distance along the target's raceline** + remainder, wrap only on the first section); the
  "doesn't join" assert (+4 == NULL) cannot fire in the port (`to` is always set). The sub-route points come from
  ProcessRaceSections 0x00031390 (same `best > d`, t in [-1e-6, 1+1e-6], section order as the port). LACoast circuits
  duplicate race sections, so section 1's end lies on section 3's end (distance 0) and vice versa: each join leads
  to the other's end at 255.1, and any d beyond that walks forever **in the original too**. Unreachable there:
  every LACoast EventDef is a placeholder that int3s at load (see "Start line"), and the only original caller of
  0x214B0 is CompartmentManager 0x000897C0. Keep core's `point_ahead_ends` guard (a hang counts as a crash for
  the port); the ai itself never calls `point_ahead`.
- Original request:
- `VsAIRaceVehicle::point_ahead` (0x000214B0) never returns for the player on LACOAST Circuit Race05 (the sweep's old
  "killed" hang; core's stream raceline calls it 9x per update): player race section 1 (raceline 255.1), d = 287.1;
  section 1's last sub-route point 1 (from 1 to 3, join, dist 255.1) -> section 3 (raceline 255.1); section 3's last
  sub-route point 5 (from 3 to 1, dist 255.1) -> section 1, and so on (d stays 287.1 > 255.1, the first race section
  is never reached). Either the original loops too (then nothing in F3_D asks this far ahead from section 1/3), or
  the port's data / section assignment differs: please check sections 1 / 3 and sub-route points 1 / 5 of that race
  (is +16 the distance on the target section? is the player meant to be on section 1 at (47, -35, -146)?).
  Core's `stream_raceline.rs` now simulates the walk first (`point_ahead_ends`, 64 steps) and stops the raceline
  points there; drop it once point_ahead ends on this data.

### Requests from graphics (flash records, dummy police car) — done
- `trigger_failure` (0x00019740, players never: +844) failure 1 → `nos_explosion_flash` before the velocity kick
  (asm 0x19907..0x19A35, SSE f32: 0.05 = 0x3D4CCCCD `dword_625184`, 0.5 `625180`, 0.666 = 0x3F2A7EFA `62517C`):
  k = clamp(0.5 / (|player0 − this| · 0.05)); d = dot(camera 0 **dir +280** (0x23EB0; not +120), normalise(this −
  camera 0 pos +212 (0x23E00))); d > 0 → a = clamp(k · ((d + 0.5) · 0.666)); a > 0 → record {0, 0.05, a, world,
  pos, k · 0.5}. pos = D3DXVec3Transform(car data +64 translation (0x23F40 = CarRender +304 + 2352·i, 0x23DD0 = +48),
  rows VE 0x1C65A0 right / 0x1C6630 up / 0x1C66C0 forward / 0x1BA270 position). Car data +64 = the render matrix +0
  (already world) plus the engine shake (CarRender 0x11ABB0), so the point is transformed **twice** (original bug,
  kept: the flash lands at R·p + p, usually off screen).
- Host API (`AiHost`): `camera(player) -> (pos, dir)`, `car_render_point(physics)`, `flare_flash(&AiFlashRecord)`.
  The plugin fills them itself: `view_inputs` (GameTick, Ai set, before `update`) reads `GameCameras.manager.view(p)`
  (position, normalised target − position: the render view, not the tick +212/+280) and every
  `visos_render::car_render::CarRender` root translation (request = physics id; engine shake not modelled, the 24 Hz
  body position without a render car); `update` / `collisions` write the records as `visos_render::flare_render::
  FlareFlash` the same tick. velocity_ai now depends on visos_render (already below velocity_game).
- Dummy police car: needs nothing from the ai (CarRender copies the police car's render car; core sets `CarSpec.police`
  and spawns the police physics cars, the ai binds them as before).
- Test `lap_test::split_screen_two_players_and_nos_flash` (forced failure 1 with a recording host, camera 20 m behind
  the last car): 2 flashes, intensity 0.57 / 0.45, size 0.29 / 0.22, positions far from the cars (double transform).
- Original request:
- VsAIRaceVehicle 0x00019740 event 1 (`+608 = a3 == 1`; dump `research/gfx/s9_dc.c` lines 544..720): k = clamp(0.5 /
  (|this − player| · 0.05), 0, 1), d = dot(camera dir (camera 0 +120), normalise(this − camera)); when d > 0: a =
  clamp(k · (d + 0.5) · 0.666, 0, 1); if a > 0 write `visos_render::flare_render::FlareFlash(FlashRecord { rise: 0.0,
  fall: 0.05, intensity: a, screen: false, pos, size: k · 0.5 })`, pos = the vehicle matrix (rows VE 0x1C65A0 /
  0x1C6630 / 0x1C66C0, translation PhysicsCarInput 0x1BA270) applied to the car render data +64 point of this driver's
  car (VsAIRaceVehicle 0x23F40 / 0x23DD0). Check the exact operands in the dump; graphics draws the flash.

### Requests from core (race record block shared with the score manager, 2026-11-06) — done
- Checked: ScoreManager 0x00169350 adds dt to `0x244E0(+188) + 68` / `+12`, 0x0016BE50 writes +36/+92 (gap to the car
  ahead / that car's race vehicle pointer) and +40/+96 (behind); 0x000244E0 = `this + 460` (+0x1CC). The port keeps
  both copies and makes them one: `VsAIRaceVehicle::stats_to_score` before and `stats_from_score` after
  ScoreManager::update (in `update_score`), `stats_to_score` at the end of `reset` (after the score reset). Indices
  = offset / 4: lap time [3], gap ahead [9], gap behind [10], fastest lap [11], lead time [17], overtakes [18], lap
  score [19], best lap score [20], score [21], car ahead/behind [23]/[24] (pointers in the original; id + 1, 0 =
  none). So `stats[21]` (prestige), `stats[11]` (`racer_records`) and `score.stats` agree after every update; the race
  manager's lap-line writes ([3] = 0, [11], [20]) reach `score.stats` at the next update.
- Original request:
- In the original the score manager writes the race vehicle's record block 0x000244E0 itself (ScoreManager +188 =
  its vehicle; COSDInGameScreen 0x000F99E0 reads `244E0(score +188)[21]`): `VsAIRaceVehicle.stats` ([u32; 25],
  +0x1CC) and `ScoreManager.stats` (`RaceStats`: lap_time +0x0C, gaps +0x24/+0x28, fastest_lap +0x2C, lead_time
  +0x44, overtakes +0x48, lap_score +0x4C, best_lap_score +0x50, score +0x54, cars +0x5C/+0x60) are one block. In the
  port they are two copies and never synced, so `stats[21]` (prestige reads) and `stats[11]` (`racer_records`) miss
  the score manager's writes. Please make them one (e.g. copy the score manager's fields into `stats` after each
  `update` / `on_collision`, or have it write `stats` directly). Core reads points from `score.stats.score` for the
  HUD / `RacerPoints` meanwhile (core.md "Requests from frontend (race points ...)").

### Requests from physics (friction spots, 2026-11-06) — done
- `update_oil_spill` (0x00019DD0, already ported: +608 == 0 → throttle 0, brake 0.2, steer ±0.2 to the farther edge;
  every `VsRandRange(lo 0x6D3770, hi 0x6D3774)` = globals +0xD8/+0xDC s) now calls
  `AiPhysicsWorld::friction_spot(pos +152 (0x16690), friction +0xE8 (0x6D3780), radius VsRandRange(+0xE0, +0xE4)
  (0x6D3778/7C), race distance +0x2A8 (0x15CA0), ignore car +0x78 (0x15B50), expiry time + lifetime +0xEC
  (0x6D3784, 0 in the ctor), rand)`; draw order radius → physics' 2 puddle draws → next delay, as the original.
  `PhysicsAi` runs `PhysicsWorld::friction_spot` with the ai's rand state handed in and taken back. The old no-op
  `AiHost::oil_spill` is gone.
- **Shared rand**: VsRandRange 0x134F0 → VsRandUnit 0x13510 → CRT `rand` 0x3DE030 is the game's one stream. The plugin
  now runs `AiState.rand` on core's `GameManager.rand.state` (`rand_in`/`rand_out` around load_steps, session
  messages, timeline objects, collisions and update); core hands the same state to physics around its tick.
- **Race distance hook** (hydrant, 0x001E1F80 case 0x17 → GetFirstRaceSection 0x00024300 (+0x477C) → GetRaceline →
  FindClosestSegment 0x262D0 → GetDistanceAlongLine 0x25E00): the ai installs it itself. At AiWorldData the first
  race section's raceline is cloned into `AiState.pending_distance_line`; `send_walls` installs
  `PhysicsWorld::set_race_distance_hook` once the physics world exists (a closure over the clone, no locks). No
  race section (cruise: the original reads through NULL) → `None` (distance 0). Core need not install it.
- Original request:
- VsAIRaceVehicle 0x00019DD0 (from 0x00018A40, with +608 == 0, every `VsRandRange(flt_6D3778, flt_6D377C)` s of AI
  time) calls PhysicsObjectManager 0x001E18E0(driver position 0x00016690, `dword_6D3780`, `VsRandRange(flt_6D3778,
  flt_6D377C)`, race distance 0x00015CA0, vehicle id 0x00015B50, time + `dword_6D3784`). If you port it, call
  `PhysicsWorld::friction_spot(&sim::FrictionSpot { position, friction_high, radius, race_distance, ignore_car,
  expiry })` (inside your physics access, rand state shared) or write `plugin::PhysicsFrictionSpot`.
- The hydrant hit needs the race distance of a point on the first race section's raceline (FindClosestSegment +
  GetDistanceAlongLine): core installs it as `PhysicsWorld::set_race_distance_hook` (core.md); please expose that
  query on your world if it is not public.

