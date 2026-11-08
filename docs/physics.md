# Physics - INDEX

_owner: physics agent. Crate `velocity_physics`. Details live in `docs/physics/details.md` (the full former physics.md, ~93 KB): NEVER read it whole; grep it by heading (`grep -n '^###' docs/physics/details.md`, then `Read` with offset/limit)._

## APIs other crates use (details: "Bevy API for core", "AI bridge", then the named section)
- Wiring: `plugin::run_physics_tick` in `InGameUpdateSet::Physics`; `PhysicsTimestep` = `clock.timer.scaled_dt()` (clamped 0.041841); schedule `PhysicsTick` = `PhysicsSet::{Setup, Step, Output}`.
- Track: messages `LoadPhysicsTrack { track }`, `UnloadPhysicsTrack`, `LoadPhysicsRaceObjects { race }`, `SetPhysicsWalls { armco, vertices, faces }`, `RestartPhysicsWorld`.
- Cars: entity `Transform` + `PhysicsCarSpawn { inputs: LocalConfigInputs, pad_slot }` (+ `PhysicsCarInput` for players); out `PhysicsCar { id, pad_slot }` + `PhysicsCarState`. `LocalConfigInputs::{from_setup, from_traffic, default_spans}`.
- `PhysicsWorldRes` (`SharedPhysicsWorld` = `Arc<Mutex<Option<sim::PhysicsWorld>>>`): `ray_cast(from,to,flags,ignore)`, `ground_height`; `PhysicsWorld::{spawn, step, remove, body_matrix, wheel, car_gear, engine_speed, engine_rpm, set_audio_max_rpm}`.
- Messages out: `PhysicsCollision(CarCollision)`, `AiCollisionMessage`, `PhysicsSlipstream`, `PhysicsObjectMessage(PhysicsObjectEvent)`, `PhysicsObjectNoticeMessage`, `PhysicsPuddleMessage`, `PhysicsCarDamage(CarDamageEvent)`.
- Messages in: `PhysicsInstanceMove`, `ResetPhysicsObject(name|None)`, `ResetAllPhysicsObjects`, `PhysicsExplosion`, `PhysicsFrictionSpot`, `SetupCarDamage`, `RepairCarDamage`. Resources: `PhysicsObjects` (`by_name()`), `CarDamageManager`.
- AI bridge (`sim_ai`): `controls/set_controls`, `set_human_control`, `set_control_mode`, `teleport`, `activate/deactivate`, `set_steer_angle`, `steer_angle_to`, `line_check`, `take_collisions`, `set_score_gains`, `set_traffic_index`, `police_handling`, trailblazer (`set_trail_blazer_globals/_flag`, `trail_blazer_reset`), `shift_gear`, `launch_result/shift_result/clear_shift_result`.
- Gauges/camera: `nos_ratio`, `zone_ratio`, `turbo_ratio`, `has_turbo`; `camera_up_offset`, `camera_shake`, `set_camera_shake`, `graphics_pose`, `sync_model`; `handling_camera_overrides()`.
- Jump cam: `set_jump_cam_hook(Arc<Mutex<dyn FnMut(JumpCamCall, &mut dyn JumpCamWorld)->bool>>)`, `jump_cam_rate/set_jump_cam_rate`, `set_jump_cam_option`, `review_cameras`, `review_timer`.
- Shared rand: `rand_state()` / `set_rand_state(u32)` (core hands `GameManager.rand` in before and out after the tick). `set_zone_fx_level(pad, level)`, `set_race_distance_hook`.
- Vehicle database: `CVehicleDatabase::{count_traffic, get_nth_traffic, randomise_setup}`, `VehicleSetup::{add_mileage, race_mileage}`; `PhysicsTweaks` call lists (`physics_tweaks`, 37 systems / 113 tweaks, replayed by core).
- Env: `PHYSNANTRAP=1`, `VELOCITY_EVOLVE_DIFF=1`, `VELOCITY_PHYS_TRACE=1`, `VELOCITY_PHYS_NO_RACE_OBJECTS=1`, `VELOCITY_PHYS_GEAR_TRACE=1`.
- Workflow: never edit `tr*.rs`; `sh tools/phys_regen.sh` (translator `tools/phys_c2rs4.py`, patches `tools/phys_c2rs4_patches.txt`, overrides `tools/phys_c2rs4_ov.txt`); see "Translation pipeline".

## OPEN requests
- To physics: none. Done 2026-11-07: start boost 0x001D1270 translated (added to `_tr_addrs.txt`) and exposed as `PhysicsWorld::launch_boost(id, gauge: f32)` (writes the OSD gauge float into the `unk_6FC098+2936` global, runs it; `launch_result(id)` = 1..4: gauge <=1.0 -> 1, 0.85..0.92 -> 2, 0.92..1.0 -> 3, >1.0 -> 4 + puncture params). Test `tests/launch_boost.rs`. Core calls it per player car on ReleaseCars.
- From physics: none blocking. Open items: "Reload Handling.cfg" tweak not pushed (CConfig 0x00199DC0 is `unmodelled_i`); CarDamage hinged bonnet/boot buckle, FX and animated repair not ported; `dword_686B58` non-race zone slow-down branch not modelled; CarDamage 0x00070F40 keeps a body-matrix stand-in (`graphics_pose` could replace it); `__PAIR64__` is `ov_zero_i!` at 11 sites (checked ones harmless). Core-side wiring requests are filed in core.md "Requests from physics".

## Next candidates / coverage
- Candidates: timeline-driven object commands (`TimelineObjectMessage::PhysicsObject*` -> 0x1E1C00 lookup + 0x17CDA0/0x17CD20/0x1E2760); CarDamage remaining items above; Reload Handling.cfg.
- Coverage: `tools/coverage/cov.py physics`. Status paragraph at the top of details.md lists everything done per stage.

## Section index of details.md (heading, line)
- L5: Status (staged work)
- L34: Debug-menu tweaks — `physics_tweaks`
- L50: Bevy API for core (`velocity_physics::plugin`)
- L86: AI bridge (stage 3) — `vehicle_entity::sim_ai`
- L109: Acceleration check (240SX)
- L118: Architecture (what the binary contains)
- L140: Numeric model
- L151: Config (`Vehicles\cfgs`)
- L169: Translation pipeline
- L223: Ported so far
- L236: Stage 4 part 1 — handling check, multi-car, traffic, walls, score gains, mileage
- L285: Stage 4 part 2 — physics objects
- L341: CarDamage
- L379: Requests (physics -> others)
- L390: Requests from graphics (race effects) — answered
- L411: Requests from core (cross-crate glue) — answered
- L428: Requests from core (audio gear) — answered
- L435: Trailblazer, gear step, police
- L463: Requests from core (object wake panic, restore point messages)
- L484: Requests from core (timeline object move, vehicle reset)
- L500: Requests from ai (trailblazer panic) — not reproducible
- L515: Requests from core (LADown Race02 road height) — answered
- L527: Requests from core (LADown Race02 road height) — original request
- L536: Requests from core (null pointer panic on the Race02 E-W deck) — fixed
- L556: Requests from core (chase camera writes VE memory)
- L583: Requests from core (jump cam, attract RandomiseSetup)
- L617: Gearbox: automatic / manual, HUD and audio rpm
- L640: Requests from frontend (LACoast Sprint 01 panic) — fixed
- L664: Requests from ai (AI review, 2026-11-05)
- L672: Sweep crashes
- L698: Handling review (2026-11-05)
- L751: Requests from core (jump cam hook needs a ray test)
- L775: Requests from core (jump cam end: `dword_68580C`)
- L789: Requests from graphics (dummy police car tag)
- L800: Split screen (two local players)
- L824: Requests from core (jump cam `blocked` panics)
- L850: Requests from core (jump cam roll uses a private rand stream)
- L877: Requests from core (speedo gauges: zone, turbo; nos names)
- L896: Explosions and friction spots
- L933: Requests from core (Handling.cfg camera overrides)
