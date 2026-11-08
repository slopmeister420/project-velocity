# Physics

_owner: physics agent_

**Status (staged work).** Stages 1-3 done; stage 4 part 1 done (handling check, multi-car fixes, traffic cars,
barrier/Armco walls, score gains, mileage — below); FastCar Evolve now translated and the hand ports verified
against it (Translation pipeline). Stage 4 part 2: physics objects, active-object ("particle tracker") update,
object positions and the object particle/trail messages done (below). CarDamage done (body dents, lights, window
cracks; hinge panels / animated repair / FX left, see "CarDamage"). Open requests done (trailblazer events, gear
step, gear-check results, police/traffic bind parts — "Trailblazer, gear step, police"). Object wake panic fixed,
object messages 2/4 and object reset done ("Requests from core (object wake panic ...)"). Instance move + reset of
all objects done; ai's trailblazer panic not reproducible (27 runs); Race02 road height answered (two levels).
Next: timeline-driven object commands (`TimelineObjectMessage::PhysicsObject*` -> 0x1E1C00
lookup + 0x17CDA0/0x17CD20/0x1E2760), kind-23 explosions (0x1E18E0 -> 0x1B3500). Race02 deck panic and the
0x398230 pool leak fixed. Chase-camera VE fields done (+4392 via SyncModels 0x001C2FF0, player camera shake
0x001D1610; "Requests from core (chase camera writes VE memory)"). Gearbox automatic/manual flag from the setup and
the HUD/audio rpm source checked ("Gearbox"). Sweep crashes (London2 `nClosestBone`, 0xCC contact / NaN sky) fixed
("Sweep crashes"). Handling review done: per-car table, model wheel spans, tyre-solver NaN, C2 config fallback
("Handling review"). Jump cam hook, `RandomiseSetup`, zone FX boost block and skid wheel check done ("Requests from
core (jump cam, attract RandomiseSetup)", "Handling review" stubs); re-verified after the WIP snapshots: regen
reproduces tr.rs unchanged, RandomiseSetup re-read against 0x00182720, all crate tests green. 0x00154FC0 is
ZoneRender's level getter (graphics data, physics reads it via `set_zone_fx_level`, wiring filed in core.md);
0x00133970 is physics' (VehicleEntity inline), ported. Jump cam hook now gets a `JumpCamWorld` (car position
0x001BA270, camera ray 0x00167680) from inside the step ("Requests from core (jump cam hook needs a ray test)").
Jump cam rate `dword_68580C` getter/setter and the dummy police car object tag done; split screen checked (already
1:1 per pad slot, "Split screen"). Jump cam `blocked` panic fixed (one-box buses, "Requests from core (jump cam
`blocked` panics)"). Physics' rand draws use the game's shared CRT stream via `rand_state`/`set_rand_state`
("Requests from core (jump cam roll uses a private rand stream)"). Explosions (0x0019FD80) and friction spots /
hydrants (0x001E18E0 -> 0x001E1A20 -> 0x001B3500, race restart 0x0019C390) ported ("Explosions and friction spots").
Speedo getters (nos / zone / turbo) done and the swapped nos getter names fixed ("Requests from core (speedo gauges
..."). Debug-menu tweaks (3 AddTweaks) recorded for core's replay ("Debug-menu tweaks"). No open requests to
physics left.

### Debug-menu tweaks — `physics_tweaks` (hand-written; replay request in core.md "Requests from physics")
- Registrations (InGameInit only): CConfig::AddMaterialTweaks 0x0019BEC0 (at 0x000B6251: "Materials", then
  "Materials.<eSURF 1..25>" x Friction multiplier / Drag / Bump map height on `g_materialProperties`), PhysicsCarInput
  0x001B8A00 and VehicleManager 0x001DC650 ("Debug Cameras", Follow vehicle `dword_688B80` -1..6) via PhysicsManager::Init
  0x000B679E -> 0x0019E080 -> 0x001DC650. 37 systems, 113 tweaks, no enums. Recorded as `PhysicsTweaks` call lists
  (graphics' format): velocity_game depends on this crate, so core replays them.
- Live values = the executable globals: `sync_physics_tweaks` (First) writes edited slots into the current world's
  `PhysGlobals` and the process-wide `physics_tweaks::TWEAKED` (seeds every new `PhysicsWorld::new`, .data survives
  races), pulls game writes back. `SVehicleSetup` modifiers 0x00189640/0x00189750 read 0x688A74/78 via `global_f32`.
- Readers not ported (slot only, harmless): Network (net code, 0x001C1FF0/0x001C2EF0/0x001C51D0), Use Axis Sort List
  (PhysicsObjectManager render 0x001E0710/0x001E0EE0), Follow vehicle (SyncModels 0x001DDAE0, core's camera).
- **Not pushed** (`NOT_PUSHED`): "Reload Handling.cfg" 0x75B288 / "Config version" 0x75B28C — VehicleEntity 0x001CB530
  then reloads via CConfig 0x00199DC0, which is `unmodelled_i` (would panic). Porting that reload is open.
- Test `physics_tweaks::tests::registrations_match_the_binary` against `research/phys_tweak_regs.tsv`
  (`tools/ida/phys_tweak_tsv.py`: pushed immediates, var addresses, .data defaults), plus `sync_pushes_and_pulls_globals`.

### Bevy API for core (`velocity_physics::plugin`, `PhysicsPlugin` is already in the binary)

Wiring (core, once): `app.add_systems(GameTick, velocity_physics::plugin::run_physics_tick.in_set(InGameUpdateSet::Physics))`
and each tick `world.resource_mut::<PhysicsTimestep>().0 = clock.timer.scaled_dt()` (default 1/24; values above
0.041841 are clamped). `run_physics_tick` runs the `PhysicsTick` schedule (`PhysicsSet::Setup` -> `Step` -> `Output`).
- Track: write `LoadPhysicsTrack { track: "LACoast" }` (dir under `\Tracks`; loads `gfx\arena.msh` + `arena.qdt` and
  `::GC`, drops any previous world and detaches its cars); `UnloadPhysicsTrack` on mode shutdown. Processed in the
  next physics tick.
- Car: spawn an entity with `Transform` (game coordinates; the car's forward is its model +X) + `PhysicsCarSpawn {
  inputs, pad_slot }` (+ `PhysicsCarInput` for players). `inputs` = `LocalConfigInputs::from_setup(&db, &setup)` for a
  database vehicle + `VehicleSetup` (config by name hash, performance modifiers incl. NOS), or
  `LocalConfigInputs::default_spans("240SX")`; set `is_player` for human cars. `from_setup` reads the spans from the
  chassis model's `f_l_wheel`/`r_l_wheel` locators itself (`spans_from_vehicle_model`, as 0x001CABF0).
  Next tick the entity gets `PhysicsCar { id, pad_slot }` + `PhysicsCarState`; removing `PhysicsCar` (or despawning)
  removes the car.
- Input: `PhysicsCarInput { axis, accelerator, brake, steer, handbrake, gear_up, gear_down, boost }` = the pad record
  `VehicleEntity__1CB530` reads (players; AI cars will take the AI's controls in stage 3).
- Output each tick: `PhysicsCarState { body_matrix, position, rotation, velocity, angular_velocity, gear, engine_speed,
  wheels: [WheelState { matrix (render matrix), contacts, contact_point, contact_normal, surface (eSURF_*),
  suspension, omega, sliding }; 4] }`; the entity's `Transform` is set from the body matrix. Matrices are row-vector
  4x4: `Mat4::from_cols_array`. Wheel order is FastCar's: 0/3 front, 1/2 back.
- World queries: `ResMut<PhysicsWorldRes>::ray_cast(from, to, flags, ignore_car)` = `PhysicsManager::RayCast`
  0x19D3E0 (`RAY_TRACK|RAY_OBJECTS|RAY_CARS`, `RAY_FIRST_HIT`, `RAY_NORMAL`; hit `{ t, point, normal, kind 1 track
  2 object 3 car }`; flag 2 / kind 2 are really the **barrier walls** (BarrierRender 0x001B3F40/20), the name
  `RAY_OBJECTS` is kept for callers), `ground_height(x, z, top)`. `PhysicsWorldRes.0` is a `SharedPhysicsWorld`
  (`Arc<Mutex<Option<sim::PhysicsWorld>>>`, `.lock()`), shared so the AI's car proxies reach the same world; the
  plain-Rust `PhysicsWorld` (spawn/step/remove/position/velocity/orientation/body_matrix/wheel/car_gear/
  engine_speed) is what the tests use.
- `PhysicsCollision(CarCollision { a, b (-1 scenery, -2 barrier/Armco wall), point, normal, impact, scale, material })` message each
  tick = message 0 of `MessageManager__CA2C0` (car-car 0x1A3A30 / car-track 0x1A0570 / walls 0x1A7060); `AiCollisionMessage { a, b, impact }` maps 1:1.
- `LocalConfigInputs::from_setup(&db, &setup)` (0x1CABF0 setup branch): config by the vehicle entry name,
  setup performance modifiers incl. NOS; spans from the model locators ("Handling review").
- Pad slot -1 (AI / traffic cars) is fine: every car points at pad record 0 (VehicleManager single player);
  a car reads its pad only when `is_player && human_control`, otherwise the AI's controls block (VE+0x9C,
  `VehicleEntity__1CB470` + the AI drive/gear code `1CF0D0`, now translated).

### AI bridge (stage 3) — `vehicle_entity::sim_ai`

- `PhysicsWorld` methods for everything `velocity_ai`'s `AiPhysicsWorld`/`AiPhysicsVehicle`/`AiCollisionQuery`
  need, each citing its original: controls block (`controls`/`set_controls`, `VehicleControlsRaw` = VE+0x9C
  layout), `set_human_control` (VehicleManager__1DD9F0's per-player flag), `vs_orientation` (0x1BB230, conjugate
  of the body quat), `body_row` (0x1BB200: 0 forward, 1 up, 2 right), `teleport` (VsAIVehicle 0x2BDA0 body part:
  zeroed state via car vtbl +0x14, 0x36A370 conj quat, 0x36B130/0x36B530/0x36B730, commit, `activate` 0x1C5710
  minus the render `SyncModels`), `deactivate` 0x1C5750, `set_steer_angle` 0x361FF0, `steer_angle_to` 0x3717D0,
  `reset_dynamics` (VsAITrafficVehicle 0x2B800 physics part), `set_wheel_lock` 0x35DCC0 (the AI's
  "set_direct_steer"), gear/rpm/NOS/launch/shift/lights/ride height, `line_check` 0x19D350, `take_collisions`.
- The trait impls live in `velocity_ai/src/physics_ai.rs` (this crate cannot name the AI traits); the lap test
  is `velocity_ai/tests/ai_lap.rs`. This crate has no dev-dependencies any more (grid_drop reads eventdef itself).
- Bugs found on the way: **0x3717D0 and 0x3987C0 return floats in st0** but were typed void/`dx:ax`, so the
  LCP (0x3BE9A0), 0x39AE70 and the AI steering (0x1CBDE0) lost the values → NaN on any hard landing or chassis
  contact (core's "7000 m/s launch"); retyped in the idb and redumped. The PhysicsManager ctor (0x19C230)
  sets +0x10 = 1.0 and **+0x5F08 = 1.0** (player mass scale in car-car contacts); 0 gave 1/0 → NaN in pileups.
  `tests/grid_drop.rs` drops cars from the eventdef grid height (LADown/London) and checks they settle.
- `PHYSNANTRAP=1` panics at the first NaN float store (Cx and Mem); with
  `CARGO_PROFILE_RELEASE_DEBUG=line-tables-only RUST_BACKTRACE=1` it names the generated line. This is how the
  three NaN sources were found in minutes.
- Tests: `tests/plugin.rs` (ExecTest: spawn from a Transform, heading kept, drives along +X, ray cast, removal),
  `tests/vehicle_track.rs` (London, LACoast, ExecTest 0-100 km/h).

**Acceleration check (240SX).** 0-100 km/h takes 12.1 s of game time at 24 Hz, as the config predicts:
engine_power 107.4 kW, mass 1721 kg, ~83% at the wheels, traction-limited ~4.6 m/s^2 up to ~11 m/s, and physics
advancing `dt * 24 * LC->timestep` = 0.03 s per 1/24 s frame (`PhysicsManager::Evolve` 0x1ADF70; `GameManager__BF670`
= dt * time stretch 1.0) - i.e. the physics runs at 0.72x game time by design. 8.7 s of physics time to 100 km/h
matches the estimate exactly; top speed approaches `speed_limit` 46.5 m/s (167 km/h). Found on the way: `VsAbs` is a
float abs (was translated as an integer abs of the bits, breaking the steering-input path 0x1B82A0), `LOBYTE(v) =`
on an integer local (bool returns such as `RayCast`'s) was dropped, a pointer parked in a 64-bit local's high dword
was lost, and meshes without stored normals (ExecTest) need them appended to the image.

### Architecture (what the binary contains)

- The car model is Eugene Laptev's **FastCar** library, linked without asserts/strings into the unattributed range
  **0x355B60-0x3C63D0** (auto-named `_AUDIO_Resample__*`; region attribution there is meaningless). Decompile dump:
  `research/phys_dump/FastCarLib.c`. Objects found so far: `CRigidBody` 0x369640-0x36E7B0 (vtable 0x675894),
  `CFastCar` 0x35DD40-0x369640 + 0x36E7B0-0x372B70 (vtable 0x675854), tyre solver 0x372B70, Maple-generated
  closed-form torque solvers 0x3B3B70/0x3B9D90, expression parser 0x3A9000-0x3AC2A0, `FCMesh`/`FCQuadTree`,
  constraint handler 0x389630 + LCP 0x3BE9A0, contact records (1560 bytes, vtable 0x675BE0, 6 virtuals
  0x39C440..0x39CAE0, built by 0x398D80), scratch arena `CGrowingMemoryPool` (0x7B5710 buffer, 0x7B570C size =
  0x4000 from its static ctor 0x398760, 0x7B5708 bump; overflow doubles and retries the whole pass).
- Physics frame (`PhysicsManager__19F920`): objects update, `VehicleManager__1DD9F0` (every car's
  `ComputeForcesAndTorques` 0x1C5800; players get their pad slot, others -1), `ResolveCollisions` 0x1B32E0
  (`do_CD`: particle trackers 0x1ADF00, box prep 0x1B11B0, car-vs-track 0x1A0570, car-vs-car 0x1A3A30, barriers
  0x1A7060, static objects 0x1A89F0, car-vs-objects 0x1AAED0, debug draw 0x1B1300, resolve 0x1AC680; then
  `Evolve` 0x1ADF70 for all cars), `SyncModels` 0x1DDAE0.
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

### Translation pipeline (the bulk of the port)

- `tools/phys_c2rs4.py` translates Hex-Rays text (`research/phys_dump/*.c`, `Redump.c` wins) into Rust over a
  memory model; `sh tools/phys_regen.sh` regenerates every `tr.rs` (+ `physics_manager/vdispatch.rs`) from the
  address list `research/_tr_addrs.txt` (560 functions). Never edit `tr*.rs`: fix the translator, add a source
  patch (`tools/phys_c2rs4_patches.txt`, matched with `vN` renumbering tolerated) or an override
  (`tools/phys_c2rs4_ov.txt` -> `vehicle_entity/ov.rs`). Add functions with
  `python tools/phys_c2rs4.py <scratch.rs> $(cat research/_tr_addrs.txt) <new> --closure` and keep its
  "translated:" list as the new address list. Re-dump after IDA type fixes with `tools/ida/phys_dump_addrs.py`
  (addresses in `research/phys_addrs.txt`, decompiles without cache); keep big functions (0x1A0570, 0x19FD80,
  0x386450) out of `Redump.c`: a fresh decompile of them comes out with `__asm` x87 blocks.
- Memory model (`vehicle_entity/cx.rs`): `Ptr { sp, off }`; per-vehicle spaces (`Ve` incl. the car at +240,
  `Hist`, `Lc`) carry the vehicle id in `off >> 16`; `Heap` = `operator new` blocks (`cx::Heap`, freed addresses
  reused); `World` = PhysicsManager image (+0 vehicle manager, +28 mesh, +32 quadtree); `Vlist` = vehicle array
  (overrides for `GameManager__C0470`, `OSDInGameScreen__FAF10`, `VsArray::operator[]`, `19E060`); `Glob` =
  executable globals (initial values from `default.xbe`). Pointers stored as integers are `Ptr::bits` handles and
  decode back with `Ptr::raw`, so int/pointer punning in the decompilation survives.
- Virtual calls: `ov::vcall` - cars to `car_vcall` (slots 0 Evolve, 16/20 save/restore state, 24 matrix, 28
  position, 32 update), anything else through `*this` (vtable address) to `vdispatch` (every translated method).
  `CFastCar`'s wheel-collision callback re-enters the running `Cx` through a thread-local (`ov::evolve_car`).
- Translator rules learned the hard way: `**(float **)` keeps the float pointee; `HIDWORD(v64) = x` and
  `*((float *)&v64 + 1)` are the high dword (incl. increments); `NAN` is the -1 sentinel bits 0xFFFFFFFF;
  `(int)dword_X` passed as an argument is the array's address; `*this = off_X` stores the vtable address;
  gotos into nested blocks lower the whole function to a block-dispatch loop (`emit_cfg`); parameters
  annotated `@<st0>`/`@<efl>` are fake (dropped, callers' extra argument removed); vcall casts keep their return
  type (`vfn_float`), and a void vcall followed by a read of an x87 `// st7` local returns into that local.
- IDA type fixes made for this (all in the shared IDB): float returns of 0x35C460/0x35C500 (engine torque, was
  read as garbage), 0x1A0530, 0x1B8260, 0x1CF090, 0x1D9CB0, 0x1DFBB0, 0x369860, 0x189AB0 (`GetNosModifier`;
  0x1CABF0 had stored the top-speed modifier at VE+100); thiscall prototypes of 0x35D4B0, 0x389630, 0x3997E0,
  0x399CB0, 0x39A7D0, 0x39AC60, 0x39AE70, 0x39BC00, cdecl 0x3A8F30/0x3BB2B0, 0x1B32E0 (`this` = PhysicsManager).
- Not modelled yet (overrides that do nothing): `ParticleManager__1005C0` (sparks; graphics runs it from the
  collision messages), debug draw 0x1B1300 (the object passes and trackers are translated since stage 4 part 2); `MessageManager__CA2C0` posts are queued in `PhysGlobals::messages` (stage 3: collision
  messages for AI/audio). `qsort` (0x3E2B80) is an insertion sort with the original comparator 0x384190.
- **`VE+240..VE+3540` is the CFastCar** (`cx` routes it to car memory): hand-written accessors must read car
  offsets there, never `ve.m` (a dead copy). Fixed in sim/sim_ai: gear `VE+3216` = car+2976 (`gear`, AI `shift`
  had been writing the dead copy), shift-allowed flags `VE+0xDC9/A` = car+3289/3290, ride height `VE+0x810` =
  car+1824, and `VE+0x5C4` = car+1236 WHEEL_LOCKED (renamed `lock_wheels`; it was mislabelled "lights override",
  VsAIRaceVehicle 0x192C0 post-race stop).
- Translator: `LODWORD(float local) = int` is a bit copy (`f32::from_bits`), not a cast. 0x35C980 returned
  `(float)(gears-1)` bits as the gear (0x40800001), which crashed 0x35C500's ratio lookup on reverse-with-throttle.
- The translated chains need a deep stack: `PhysicsWorld::spawn`/`step` run on a 64 MB physics thread.
- **`CFastCar::Evolve` (0x3652D0) and everything it calls now run translated** (car vtable slot 0 ->
  `f_3652d0`; Ackermann 0x361FF0 too). IDA fixes for it: thiscall prototypes of 0x3652D0/365250/3651F0/362A30/
  363810/36D740/36D9E0/36DA90/372B70, float returns of 0x3724B0/0x3AEDD0, and **`_chkstk` (sub_3E2900, size in
  eax) typed `__usercall __spoils<eax>` with user stack points after each FastCar call** — without that every
  big-frame function decompiles with "bad sp value" and a clobbered `this` (`tools/ida/phys_evolve_types.py`).
  0x3651F0 (calls the car+1104 function pointer) is `ov_detect_wheel_contacts`.
- The hand ports (`car_evolve`, `car_contacts`, `car_tyre`, `rigid_body`, `solver`, `car_wheel`, `set_steering`)
  are kept as a reference: `VELOCITY_EVOLVE_DIFF=1` runs them on a copy of every evolve and prints the first
  differing car words (`ov::evolve_car`). Result over the London lap and all physics tests: dynamics identical,
  only 1-ulp x87 rounding in output-only fields (slip +2528, wheel spin angle +1416 and render matrices, susp
  offset +2252); the hand slip ratio also skips the 2.0/-2.0/0 branches. Lap test results are unchanged
  bit-for-bit, so the London off-road/spin behaviour is not a hand-port error.

### Ported so far

- `fastcar::rigid_body` (Euler 0x36D9E0, RK4 0x36DA90, derivatives, quaternions), `fastcar::car` (ctor, parameters
  0x35E4D0, Ackermann steering 0x361FF0, wheel frames), `car_evolve` (Evolve 0x3652D0 incl. two-wheel balancing and
  physics-vs-play), `car_contacts` (0x362A30, 0x363810), `car_tyre` (0x372B70), `car_wheel` (render matrices
  0x36E7B0), `solver` (0x3B3B70/0x3B9D90). `tests/fastcar_flat.rs`: 240SX settles and drives on a flat plane.
  These hand ports run the car's own `Evolve`; everything around them is translated (feature `tr`, on by
  default): VehicleEntity 96 functions, PhysicsManager 56, FastCar ~360 (constraint handler, LCP, contact
  generation), PhysicsCarInput, VsConfigFileRead.
- Helper agents ported `vehicle_database/*` (all bins, `string_hash` = case-sensitive CRC of the upper-cased name)
  and `fastcar::{mesh,quadtree}`, `material_properties`, `c_bump_map` (arena.msh/qdt formats documented in
  those modules).

### Stage 4 part 1 — handling check, multi-car, traffic, walls, score gains, mileage

- **Handling check (no physics bug found).** `tests/handling.rs` on `TrackWorld::flat` (a 2-face tarmac plane,
  one compact quadtree leaf; test ground): 350Z skidpad at 10 m/s, steer 0.3 rad gives an 8.3 m radius (kinematic
  L/tan = 8.6 m), so `set_steer_angle` (0x361FF0) reaches the wheels 1:1; friction-limited lateral acceleration
  ~1.9 g per physics second at 20-40 m/s; full brake 30 m/s -> 0 in 2.8 game s (~1.5 g physics). Signs: the
  spawn quaternion of the old tests leaves cars facing **-z** (`body_row(0)`); a positive steer angle turns the car
  negatively about +y, and `steer_angle_to` (0x3717D0) returns the matching sign (positive control over 5
  bearings; |angle| asymmetric ±10% from the Ackermann wheel it measures from). So the London run-offs are the AI
  driving 55 m/s into hairpins: `get_steer_towards` clamps to π/8, tightness ≤ (2π/8)^3 = 0.4845 never reaches
  the braking threshold (≥0.6), and no physics term limits speed in corners (the kinematic stand-in's 200 m/s²
  lateral cap hid it).
- **Collision message rate is original.** The track and car passes post message 0 for *every* contact record with a
  non-zero normal impulse (+236), every physics frame (PhysicsManager.c ~2400/3560); the wall pass posts one per
  car per frame (largest impulse, b = -2). ~14/s while scraping is what the original does; filtering belongs in
  the AI's handler (0x160B0).
- **Multi-car panic fixed** (core): `PhysicsCarInput__1BA270` = `VehicleEntity::GetPosition` (car vtbl +0x1C into
  a VsVector4, w = 1) -> `ov_ve_get_position`; the slipstream search (0x1BA8A0) also needed
  `VsAITools::IsLeftOfLine` 0x28090 and the cross product 0x293D0 (`ov_is_left_of_line`, `ov_ai_cross`, x87
  differences rounded where stored). `tests/handling.rs::slipstream_pair` finds the leader 12 m ahead. The
  search skips cars whose control mode (VE+0x13DC) is 2, 3 or 6 — **the ctor leaves 6**, so cars must get
  `set_control_mode` (0 player, 1 race AI, 4, as PhysicsManager 0x19E080 does) or nobody slipstreams them.
- Remaining unmodelled callees: see "Requests from core" below (none reachable with shipped data).
- **Traffic cars**: `CVehicleDatabase::count_traffic(area, kind_mask)` / `get_nth_traffic(...)` = 0x18AEF0 /
  0x18AE10 (TRAFFIC.BIN list, area = +0x10, kind bits +0x0C; 0 = any), `LocalConfigInputs::from_traffic(&db, i)` =
  the traffic branch of 0x1CABF0 (spans 2×half spans, axle distances half the front-back span, wheel radius
  +0x44, no modifiers, VE+0xFEA = 1). `tests/traffic_cars.rs`: all 18 setups (name hash == +0 hash) settle and drive.
  `spawn` now sets VE+5072 = vehicle id (VehicleManager.cpp; the race manager's id).
- **Barrier / Armco walls**: the wall pass 0x1A7060 runs over two meshes — `BarrierRender` (`dword_705F48`
  +4 mesh / +8 quadtree, getters 0x1B3F20/0x1B3F40, non-professional) and Armco (`dword_6D2458` / `dword_6D245C`,
  professional). Both are built at load by `Armco::BuildMesh` 0x12440 / `BarrierRender__108030`: a 4 m strip of
  material 21 over each line (`AddToFaceList` 0x7F8C0 alternates winding, starting from the side vote),
  `FCMesh::ComputeNormals`, `FCQuadTree::Build(8.0, 2.375, 0, 1)` + `Compact`. **Faces are one-sided**: a wall
  whose normals look away from the car is driven through. Ported `FCQuadTree::Build`/`BuildNode`/`Compact`
  (0x3593D0/0x359870/0x35A3A0) and the 2D triangle/box test 0x380EA0 in `fastcar::quadtree`
  (`build`, `compact`, `to_image`); `TrackWorld::from_walls(verts, faces)`; `PhysicsWorld::set_barrier_walls` /
  `set_armco_walls`; Bevy message `SetPhysicsWalls { armco, vertices, faces }`. In `Cx` the collision sets share
  `Sp::Mesh`/`Sp::Qt`, offset bits 22-23 = set (0 track, 1 barrier, 2 armco, `COLL_SHIFT`). Positive control
  `tests/quadtree_build.rs`: rebuilding every shipped `arena.qdt` from its mesh gives identical trees; ExecTest,
  London2, Tokyo2 word for word, the other four differ by ≤3 borderline face assignments (the exporter's
  arithmetic; the XBE path keeps x87/f32 as ported). `tests/walls.rs`: a car stops at a barrier and an Armco wall.
- **Score gains**: `PhysicsWorld::set_score_gains(id, Some((zone, nos)))` each update (None = no race record:
  traffic, police). Read by `ComputeForcesAndTorques` 0x1C5800 through a source patch of its `VsRaceManager__3D640`
  call (`ov_score_record`, keyed by VE+5072) and 0x3DF30 (`ov_score_manager`); the record is a heap stand-in with
  +0xA0 zone, +0xA4 nitrous. Other 3D640 callers (1CC3F0) still get null.
- **Mileage**: `VehicleSetup::mileage` (+0x190, was `unk_190`), `add_mileage(m)` = 0x1850B0, and
  `VehicleSetup::race_mileage(raceline_len, laps)` = EventDatabase 0x91620's `(int)length * laps` (first race
  section's raceline), added to `GamePlayer__C1E70(0)`'s current car.

### Stage 4 part 2 — physics objects

- **Data path** (what the game really runs): `CCompartment::RegisterInstances` 0x2E7B50 -> per instance list
  `CompartmentManager__86F70` (= `PhysicsObjectManager__1DE730`, a typed preallocated list) -> per record
  `F3CompartmentManager::AddInstance` 0x8AEA0 -> `PhysicsObjectManager__1DFF60` (AddCompartmentObject).
  `CompartmentManager__87D00` is only the re-relocation path of a streamed compartment (not used here).
  Ported: `physics_object_manager::register_instances` (hand port of 2E7B50 + AddInstance cases 1 and 3,
  incl. the streetlamp / dummy cop car / hydrant type tags by visual-resource CCRC) over translated 1DE730,
  1DFF60, 1E3CB0 (movable setup, renamed `PhysicsObjectParticleTracker_SetupMovable` in the idb), 1E32B0
  (immovable), 1E5270 (place), 1E5740 (matrix). The cpt image is copied to one heap block and stays resident:
  the relocated resource table is `{ptr, kind, hash}`; AddInstance params are the 8DDE0 layout (+0 type, +4/+8/+12
  vis/phys/extra table entry, +16 pos, +32 conj quat, +48 mass, +52 name hash, +56 u16 flags). Only v5 cpts
  (stale v3 LADown CPTS is skipped, as misparsed by the engine); lists stop at a type-5 list (padded records).
- Resource kinds: 11 = `\x01OCF` compound object (header 16 B, then +16 inertia, +52/53/54 sphere/box/capsule
  counts, +60/+64/+68 relative offsets of 116/36/64-byte primitives, relocated once), 12 = FCMesh immovable
  (`+16 = +32`, faces/normals pointers rebuilt). Shipped: London 3866 objects (1982 movable), London2 3015, LACoast
  2478 (its 1500 movable records have no physical resource -> visual only), LADown races 442, Tokyo 0.
- **Object** (160 B): +0 matrix (row 3 = position), +100 movable (40 B: +0 mass, +4 COM offset, +16 inertia,
  +28 spherical, +29 awake request, +32 compound object, +36 rigid body), +104 immovable (24 B), +108 self,
  +136 instance record (+0x24 timeline id, +0x30 old sound id, +0x34 type, +0x38/+0x3A type/index u16, +0x4C name),
  +140 inverse draw distance, +144/+148 visual/extra resource, +152 flags (1 collides, 2 visible, 4 hit, 8 hidden,
  0x10/0x2000 from instance flags, 0x20 broken, 0x40 exploded, 0x1000 hit before).
- **PhysicsManager** (= `VsAIRaceVehicle__24F90`, the World image): +24 object manager, +2836/+2840 axis-sorted
  lists (movables / immovables; C2AxisSortedList 0x383FC0, insert 0x3847C0, update 0x384C70, query 0x385920, sort
  0x385B80; entries 16 B {obj, r, x, z}, the object keeps its slot index at obj+96), **+2844 + 712·i: 30 rigid
  bodies** (ctor 0x1E3200, vtable 0x64B774 = CRigidBody's), +24204 free list, +24324 free count (29). The
  bodies' +540 is the owning object. Object manager: +0 count/list, +512 capacity, +1152 object array, +1664/+2176
  movable/immovable arrays, +2704 list type, +2692 30-entry array, +3216 m_CurrentList (starts -1 here).
- **Per frame**: 0x1ADF00 updates every owned body (0x1E4C30: evolve via vtable 0, trail particles for types 24/26,
  damping decay, sleep test -> 0x1E4AE0 returns the body); 0x1A89F0 (car vs immovables, posts message 2, plays the
  instance timeline when the car's impact exceeds W of the record) and 0x1AAED0 (car vs movables: wakes 0x1E4710,
  breaks 0x1DEBB0) run inside ResolveCollisions, 0x1AC680 resolves the object contacts. Message 2 goes straight to
  the listener `PhysicsObjectManager__1E1F80` (hand port in `physics_object_manager`, called from `ov_message`).
  After the frame `sync_matrices` runs the moving part of the render pass 0x1E0EE0 (`PhysicsObjects__1E58C0` with
  interp 1.0) so awake objects' matrices follow their bodies.
- API: `PhysicsWorld::register_compartment(bytes)`, `objects() -> Vec<PhysicsObjectInfo {name, kind, list_type,
  matrix, awake, flags, collides}>`, `take_object_events() -> Vec<PhysicsObjectEvent>` (`CollisionParticles {kind,
  object, point, impulse}` = FFC30, `Trail {from, to, event}` = 1000E0, `Impact {sound, mapped, point, impulse,
  again}` = VsAudio::PlayPhysicsImpact, `PlayTimeline {timeline}`, `Explosion {point}`). Bevy: `LoadPhysicsTrack`
  registers `CPTS.VCD:instance.cpt`; `LoadPhysicsRaceObjects { race }` registers the race's `RACE.VCD`
  instance/uinstance; `PhysicsObjectMessage(event)` each frame; resource `PhysicsObjects` (`by_name()` = the
  TimelineWorld map).
- Deviations: the render-only parts are ignored (1E3A60 object lighting, 8DF40, GFXHelper, StartLineEffects —
  streetlamps get index 0), no preallocated lists for visual-only instance types. Kind 23 runs the real friction spot
  since the explosions stage.
- Bugs fixed on the way (all in the idb / translator): thiscall prototypes of 0x1A89F0, 0x1AAED0, 0x1E4C30, 0x1DEBB0,
  0x1E4710, 0x1E3A60, 0x1E3CB0, 0x1DFF60, 0x384C70, 0x1E5270, 0x379660/0x379050, 0x3847C0 (edx unused, retn sizes);
  0x37EF70 takes two pointers; 0x378BD0's last args are floats (1AC680 patch passes 0.1f); 0x1ADF00 passes dt as
  float; `***(float ***)` reads now keep the float pointee (was an int->float conversion in 1A89F0/1AAED0's car
  position: no object was ever found); `VsVector4::VsVector4_XYZ` sets x y z and w = 1 (was a no-op override —
  also used by the car passes; all physics/AI tests unchanged); 16-bit `++/--*(_WORD *)` memory ops; `~1u` was
  rewritten to a destructor name; locals retyped to `int[]` in 0x3847C0/0x384C70 (phys_lvars); functions named
  `X__1E...` demangle as `E::X` in Hex-Rays text (two collided) — rename to single-underscore names.
- Test `tests/physics_objects.rs`: every track's CPTS + race compartments register (objects == listed, finite
  matrices); a 350Z driven into a London movable object reports impacts, wakes it and knocks it ~12 m.

### CarDamage (src/CarDamage, 0x0006AFA0-0x00079C90) — `velocity_physics::car_damage`

- **Purely visual: no effect on handling or engine.** The only callers are CarRender (0x00119900 lights, 0x001154E0),
  the AI vehicle (VsAIVehicle +0x37C owns the 0x1EF4-byte object: ctor `VsAIRaceVehicle__23620` + 0x0006B8B0, dtor
  0x0006BB20), the manager update 0x0006ADC0 and `GameManager::RestartRace` (0x0006B090 repair). The gameplay damage
  that fails AI cars (VsAIVehicle +0xE4, "damage >= 0.0f") is the AI's, already ported there.
- Dumps: `research/damage_dump/CarDamage.c` (all 113 functions), `callers.txt`, `consts.txt`; scripts
  `tools/ida/phys_damage_*.py`, `phys_disasm.py`, `phys_read_f32.py`.
- **Model**: 9 bones (+140, 40 B: rest, cur, shell radius, hinge type, linked bone, dent count) from the static table
  0x00421930 (0 front -x / 7 front +x corners, 1/2 and 6/5 sides, 3/4 rear corners, 8 roof), moved to the `l_head` /
  `l_brake` locators (car data +768/+896, z negated) by 0x0006CE00, every vertex weighted to its 2 nearest bones
  (0x0006DA40, weight info 16 B: u16 bones[2], original position), then corners moved to the body bounds (0x0006EDC0).
  Model space: x lateral, y up, z front. `deform` (bit 8 of +7640 clear) = VsAIVehicle +0x34C != 0; police never deform.
- **Intake** 0x00070F40 (message-0 listener 0x0006B820): impulse halved, CollisionFx event, point/normal to model space
  (normal negated when the car is `b`), closest bone by rest position; facing test 0x00070DB0 (>65° off the bone
  axis: corners fall to the side bone, others are dropped); per bone the strongest hit is kept with world->model.
- **Update** 0x0006BBA0 -> 0x00072580 per pending bone: mag = impulse*7 (>=10), push = local normal*mag*0.001, dent
  centre snapped to the nearest weighted vertex of a probe slid along the edge (0x00071680/0x00071B90), bone pushed up
  to its shell radius, lights 0x00070950 (2/3 radius: car flags +2256 bit 1 front / 2 rear cleared, hinge side set;
  full radius: light broken, bone 0 -> light 1, 7 -> 0, 3 -> 3, 4 -> 2), window cracks 0x00070B30
  (push accumulated per window, stages at radius x1/6/12, max 12), vertex dent 0x0006FF80 (reach 0.75 L, radius 6 L,
  cos falloff; element 1 moved by -1.5x the dent), impact record (92 B, max 56; nBuckles < 54, bone count < 6).
- Repair 0x00075780 (full): bones/vertices back (0x00075470), counters, windows/lights repaired, flags |= 0x17.
- **Not ported**: hinged bonnet/boot buckle (0x0006E7F0/0x0006E9E0/0x0006EBD0 setup, 0x00071E50 -> 0x0006F5E0; the
  stub returns "no buckle" = a car without hinge records), animated debug repair 0x00075B00 (debug menu only),
  CarDamageFX 0x00078880 particles / 0x000781C0 glass (graphics, from the events), debug draw 0x00074390, light
  sub-model matrices 0x00073D60, shader swaps 0x00073700 / 0x000730C0 (graphics, from the events).
- Bevy: `SetupCarDamage { vehicle, model: DamageModel { head, brake, meshes: [DamageMesh { matrix, positions,
  attr1 }] }, model_to_root, deform }`, `RepairCarDamage(Option<usize>)`, out `PhysicsCarDamage(CarDamageEvent)`
  (CollisionFx / LightBroken / LightRepaired / WindowCrack / WindowRepaired / MeshChanged); resource
  `CarDamageManager` holds the deformed vertices (`get(id).model.meshes[i]`). System `car_damage` runs after
  `write_state`; model->world = body matrix x `model_to_root` (stand-in for VehicleEntity 0x001C2FF0, which adds the
  graphics pivot / lean). `CarCollision` gained `impulse` (message +56, contact-frame impulse 0x001B39B0).
- Test `tests/car_damage.rs`: two head-on hits on a box body (push 0.14 then clamped 0.25, headlight 0 breaks,
  windscreen crack stage 0, side normal goes to bone 6, repair restores), and a 350Z into a barrier dents bone 7.
- Approximations: Dot3/Length via x87 `dot87`, inverses via glam (Renderer 0x002AB330/0x002ABD40), impact time =
  the manager's accumulated dt (original VsAIRaceVehicle__24390 timer).

### Requests (physics -> others)

- AI: build the barrier wall mesh as `BarrierRender__108030` does (per barrier line of kind != 1: points at y and
  y + `dword_68656C`, start side from kind 2 = +1, 3 = -1, kind 0 = vote over points via the closest raceline
  segment's midpoint and `IsLeftOfLine`; faces via `AddToFaceList` alternating, material 21) and send both meshes as
  `SetPhysicsWalls` at race load (Armco from `AiWorld.armco.mesh` when professional, barrier otherwise, none with
  `nobarriers`); call `set_control_mode` on bound cars; feed `set_score_gains` from the ScoreManager each update.
- Core: on `RaceSessionMessage::Mileage { laps }` call `VehicleSetup::add_mileage(race_mileage(len, laps))` on the
  player's current car; traffic cars: `get_nth_traffic` + `LocalConfigInputs::from_traffic`.


### Requests from graphics (race effects) — answered (core's `EffectCars` reads the physics images, core.md 446)
- Expose per car in `PhysicsCarState` (or a `sim::PhysicsWorld` getter) what `visos_render::race_effects::EffectCar`
  needs: FastCar velocity (+32), VehicleEntity +20 speed, +4788 heading (0x001BB200(0)), +3691 smoke-off byte, the
  ground plane of 0x001BAF60 (three wheel contacts, normal up), and per wheel: centre (0x00361400: +1968 + 16i when
  +2240 is set, else the wheel matrix origin), radius (+1804 + 4i), contacts, `sliding` over **all** contact records
  (0x00364A90 == 1; `WheelState.sliding` only reads the first record), surface (VE+4868+i), VE+24+i no-marks byte.
- Collision messages (`MessageManager__CA2C0` posts, queued in `PhysGlobals::messages`): forward each as
  `visos_render::race_effects::SparkCollisionMessage(SparkCollision { car (+0), other (+4, -1 scenery), point (+8),
  normal (+24), impulse (+40), scale (+72), material (+76) })` (consumer SparkRender 0x00141980).
- Slipstream: at VehicleEntity 0x001C5B38 and PhysicsCarInput 0x001BAD9D the original calls
  `SlipstreamRender::SetStrength(vehicle index (VE+0x13E0 / +0x54), strength)`; write `SlipstreamStrength` there.
- Collision debris rides on the same `SparkCollisionMessage` (graphics runs ParticleManager 0x001005C0 from it).
  Physics objects (PhysicsObjectManager 0x001E1F80 -> ParticleManager 0x000FFC30): write
  `visos_render::stateless_particles::ObjectCollisionParticles { kind: object type +52 (0x00101300), object: its
  matrix, point: record +8, impulse: record +40 }`; particle trackers (0x001E4C30 -> 0x001000E0, events 24 / 26):
  `ParticleTrail { from, to, event }`.
  **Answered (stage 4 part 2):** this crate cannot depend on `visos_render`, so both come out as
  `velocity_physics::plugin::PhysicsObjectMessage(PhysicsObjectEvent::CollisionParticles { kind, object (row-vector
  4x4, `Mat4::from_cols_array`), point, impulse } / Trail { from, to, event })` from the original call sites; the
  1:1 forwarding is requested from core (docs/core.md "Requests from physics").

### Requests from core (stage: cross-crate glue) — answered
- **London first-tick panic fixed**: 0x0005EC70 (`VsBaseCamera__5EC70`, matrix row `this + 16*k`) is now
  `ov_mat_row` (phys_c2rs4_ov.txt had an explicit `ov_unmodelled!` entry for it, which wins over earlier lines —
  remove such an entry when modelling a callee). London circuit smoke test (`SMOKE_AREA=2 SMOKE_TRACK=London`;
  without SMOKE_TRACK the smoke test looks for LADOWN in London and times out on the quick-race screen) exits 0.
- Remaining `unmodelled_*` stubs in the generated code, none reachable with shipped data: 1BB320's
  `FCTriangleBoxOverlap` / `FCWheelTriangleSoupContact` (only when `GC wheel_CD_with_raycasts` = 0; Handling.cfg
  ships 1), 1CB530's network branch (CConfig 199DC0, VsAIDriver 15DB0, gated on `byte_6F7158`), 1CC3F0's raceline
  respawn (VsAIRoadLine, 69A20, 1CC5D0; behind `ov_null_i` of the race record), 1B59C0's `sub_3DF020` (tyre debug
  print). 1A89F0 / 1AAED0 / 0x1E* object code has none.
- Sparks: `CarCollision` now carries `scale` (message +72: 5.0 car-car, 8.0 track, 30.0 walls) and `material`
  (+76 byte = contact record +1556; -1 car-car / walls). `ov_message` keeps 20 message words.
- Slipstream: `PhysicsSlipstream { vehicle: i32, strength }` Bevy message (= `SlipstreamRender::SetStrength`
  0x0013FB30 via `ov_slipstream`; `PhysicsWorld::take_slipstream`). VehicleEntity 0x001C5B38 passes VE+0x13E0 (the
  player index, set by VehicleManager 0x001DC650; spawn now sets it to the pad slot for players), PhysicsCarInput
  0x001BAD9D the slipstreamed car's index (search record +84). Core forwards it 1:1 as `SlipstreamStrength`.

### Requests from core (audio gear) — answered (see "Trailblazer, gear step, police" below)
- `PhysicsWorld::shift_gear(&mut self, id) -> i32` = one 0x001B74A0 call on vehicle `id`'s VE through a Cx (it
  writes +4048..+4064), for VsAudioEvents::UpdatePlayerEngine 0x0004C44E (core calls it once per GameTick in
  `race_cars::car_outputs` for the player). Core reads only the keep branch today (`race_cars::held_gear`); in the
  smoke run VE+3216 (request) is 0 while +4052 is 1/2 on ~280 updates, so the original's call would step the gear.
  Also worth checking that VE+3216 being 0 there matches the original (only the throttle paths write it).

### Trailblazer, gear step, police (requests from core / ai, ai.md TrailBlazer events) — `sim_ai`
- All plain `PhysicsWorld` methods over translated code; this crate cannot name `TrailBlazerMessage` (velocity_game
  depends on it), so core forwards the events (docs/core.md "Requests from physics (trailblazer, audio gear)").
- Trailblazer (TrailBlazer.cpp 0x000C4870 start / 0x000C4E50 end / 0x000C4FF0 update): `set_trail_blazer_globals(
  &TrailBlazerGlobals { zone_velocity, zone_sensitivities, friction_sa, steer_bias_steering, steer_bias, g_686b58 })`
  writes `::GC` +0x84/+0x90/+0xD4/+0xD0/+0xCC into **both** `gc` (struct, read by translated code as `cx.gc.X`) and
  `gcm` (image) and `dword_686B58` into `glob.w`; `trail_blazer_globals()` reads them back (what 0x000C4870 saves).
  `set_trail_blazer_flag(id, b)` = VE+0x10 byte via `VsAIPoliceVehicle__17DE0` (= VsAIVehicle +0x24 -> VE); read by
  0x001D1850 and VehicleManager 0x001DE050. `trail_blazer_reset(id)` = 0x001CC2E0 (`C::VehicleEntity`: VE+3720 = 1,
  LC+148 = LC+272, `dword_688B70` = 1.0, LC+8 = GC+172, hist entry +144 = 0).
- Time stretch: 0x000C61D0 writes the WMM `dword_6FA8D0`+0x10 (core's; also set by PhysicsManager 0x0019F920 in WMM
  modes 3/1). Physics' own slow-down is `dword_688B70` (Evolve's h multiplier), driven by the translated 0x001D1850
  zone code from `GC->visual_zone_velocity` / car speed. `dword_686B58` only acts in 0x0019F920's non-race branch
  when `dword_6F5B70` (zone FX amount) > 0: dt × ((1−a) + a·min(1, 686B58/(0.625·3.6·speed))) — not modelled (our
  frame is the race branch).
- Gear step: `shift_gear(id)` = VsConfigFileRead 0x001B74A0 (held gear +4052 steps one gear towards the request
  +3216 = car+2976 once timer +4048 > GC+372 + GC+344/348 (0x00364B20 == 1); +4056/+4060 shift state from GC
  +460/+464/+468, +4064 previous; returns +100 while +4048 < GC+372). 0x001CF0D0 (AI) calls it itself and stores the
  result to +3216 (−100 later); for players VE+3216 is car+2976, written by the translated FastCar gear up/down
  code and 0x001D1850, so a request of 0 = neutral at standstill is normal.
- Score gear checks: `launch_result(id)` VE+0x1338 (0x001D1270, 1..4), `shift_result(id)` VE+0x133C (0x001D1480: 1
  early/late, 2 good, 3 perfect), `clear_shift_result(id)` = ScoreManager 0x0016AFE0 (after scoring, from
  0x00169E30); both zeroed by PhysicsCarInput 0x001B9940.
- PhysicsManager 0x0019E080 bind parts: `set_traffic_index(id, i)` (VE+0x13D8 = traffic manager index, control
  mode 2), `police_handling(id)` (control mode 1, then VE+3220 ×2, LC +264 ×2, +92 ×2, +80 ×1.5, +76 ×1.5).
- Test `tests/trail_blazer.rs`: globals save/set/restore round trip + flag + reset (688B70 back to 1.0) while
  driving; held gear converges on the request one step at a time; police scaling doubles car+2980.

### Requests from core (object wake panic, restore point messages) — answered
- **Object wake panic (LADown Circuit Race02) fixed.** Root cause: the sleep / give-back 0x001E4AE0 never returned
  its body to the free list, two translator bugs: (1) `LOBYTE(v1) = PhysicsManager__1B41F0(...); if ((_BYTE)v1)` on a
  pointer-typed local was dropped (`{ let _ = ..}`) and the test read the whole null pointer -> always false, so no
  body ever came back; after 30 wakes the count went to -1, the "Too many awake objects" path (an `ov_ignore`d log,
  hence never seen) re-ran the no-op give-back and popped entry -1 = null. (2) `(*(this + 25))[9]` with
  `_DWORD **this` indexed bytes (+9) instead of dwords (+36, the body). Translator now: byte writes into pointer
  locals rebuild the bits (`Ptr::raw`), `(_BYTE)ptrlocal` is the low byte, and a subscript of a deref of a
  generic `pp` is 4-byte elements (also corrected 0x001CABF0's `(*a6)[12]`/`[99]`, a6 null in our spawn path).
  0x001E4C30's sleep gate (`LOBYTE(v2)`) is now honoured too. Verified: Race02 smoke with race objects
  (`SMOKE_RACE=2 SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=170`) drives 176 s and exits 0; `tests/physics_objects.rs::
  many_wakes_reuse_bodies` wakes 80 London movables (never > 30 awake).
- Channels 2 / 4: `ov_message` queues `PhysicsObjectNotice::Hit { object }` (msg +4) / `Woken { object }` (msg +0),
  object = instance name hash (`*(obj+136)+76`, `RaceTimelineSequencer__172C70`); `PhysicsWorld::take_object_notices`,
  Bevy `PhysicsObjectNoticeMessage` (separate from `PhysicsObjectEvent`, whose exhaustive match lives in core).
- Reset: `PhysicsWorld::reset_object(name)` / Bevy `ResetPhysicsObject(name)` = lookup 0x001E1C00 (hand port,
  lists 127..0, objects last-first) + translated 0x001E37D0 (flags 4/0x40/0x80/0x1000 cleared, visible unless
  instance flag 1, movable: matrix from the instance placement, 0x001E5270, awake request off, body given back;
  immovable: matrix + axis-list update). `wake_object(name)` = 0x001E1C00 + 0x001E4710. Request to core filed
  (core.md "Requests from physics").

### Requests from core (timeline object move, vehicle reset) — answered
- F3EventInstanceMove (TimelinePhysics 0x0017D540): Bevy `PhysicsInstanceMove { object, location (z already
  negated), velocity, angular_velocity }` / `PhysicsWorld::instance_move` (hand port in `physics_object_manager`:
  lookup 0x001E1C00, matrix row 3 = location (0x00101320/0x000611B0/0x0017DCF0), immovable +104 vector saved,
  overwritten with the location and restored around the call, wake 0x001E4710 when movable, 0x001E5270(obj, vel =
  event +16, ang = event +32; the angular one is scaled by 0.017453292 = degrees). The velocities only land when
  the object has a body (+100 +36), i.e. movables after the wake.
- **0x001DE400 is not a vehicle reset**: it is attributed to VehicleManager.cpp but runs on PhysicsManager+24 (the
  object manager): every object of the active (+1024+i byte == 1) type-1/3 lists through 0x001E37D0, then every
  flagged (+152 & 1) entry of the 30-object array +2692 through 0x001E5670 (body given back, axis lists). Port:
  Bevy `ResetAllPhysicsObjects` / `PhysicsWorld::reset_all_objects` (for `ResetPhysicsObject { object: None }`).
  Applied in `PhysicsSet::Setup` after the single resets, then the moves. Request filed in core.md.
- `PhysicsWorld::step` runs no physics frame at all while the world has no car (objects stay frozen); tests that
  move objects spawn a car.
- Test `physics_objects.rs::instance_move_and_reset_all` (London movable moved 5 m up, launched along +x, reset).

### Requests from ai (trailblazer panic) — not reproducible on the current tree
- 27 headless Race02 trailblazer runs (RUST_BACKTRACE=1, through the slow motion and its end; 10 of them with
  `VELOCITY_PHYS_NO_RACE_OBJECTS=1` = the reporter's object-free setup, 5 consecutive of those exit 0, the rest with
  the race objects, several under parallel load) never panicked. The `cx.rs:709` site is `Cx::i` on a null pointer
  at offset 0 (e.g. a vcall through a null body/object); no root cause found, so nothing was changed for it. If it
  recurs, the backtrace names the generated line. Diagnostics added: `VELOCITY_PHYS_TRACE=1` logs every object
  notice (Hit / Woken), `VELOCITY_PHYS_NO_RACE_OBJECTS=1` skips `LoadPhysicsRaceObjects`. Note for the object
  counts: Race02 wakes ~150 distinct objects (120 in one frame at ~165 s), far beyond the 30 bodies, all fine.
- 0x00398230 pool leak: fixed (see "null pointer panic on the Race02 E-W deck").
- Original report: Intermittent panic (1 of 2 runs) on the LADown Race02 trailblazer smoke (`SMOKE_RACE=2 VELOCITY_NO_RACE_OBJECTS=1
  "SMOKE_VIA=340.5,-53.3;349.6,-71.1" SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=128`): thread 'physics' at
  `vehicle_entity/cx.rs:709` "load from Ptr { sp: Null, off: 0 }", 2.3 s into the trailblazer just as the time
  stretch returned to 1 (ai now runs on the stretched dt). No backtrace yet (rerun with RUST_BACKTRACE=1 passed).
  Graphics' new `trail_blazer_render` ray cast through `PhysicsWorldRes` was being added at the same time.

### Requests from core (LADown Race02 road height) — answered: no offset, two road levels
- `tests/road_layers.rs` (repeated downward ray casts, every layer): from x ≈ 350 the trailblazer runs on an
  **elevated deck** (top y −9.5..−9.7, underside −13.9, normal down) above the street (−14.8 at x 503, −17.9 from
  x 520). The deck has a **hole at x ≈ 578..595, z ≈ −56..−58** (no top face, only the one-sided underside), deck again
  at (597.6, −63.6). The ai raceline 25 m ahead of the car is on the street (−17.9) from x ≈ 536, and the trailblazer
  spline rises to −7.3 at (515, −67) (jump arc). So the triggers at −15.5..−18 are on the street the route drops to
  through the hole; the car height on the deck is right (body origin −9.35..−9.44 over a −9.58 surface, same on the
  whole elevated run from x 350 where the raceline itself is at −9.5). In `core_trails.log` the car stops at
  (575.4, −9.35), 3 m short of the hole (speed 16 → 4 → 0 m/s from 200 to 204 s: worth checking whether the smoke's
  throttle ended there or the car hit something). Whether a race object covers or opens the hole in the original
  (e.g. moved by a timeline F3EventInstanceMove) is not checked.

### Requests from core (LADown Race02 road height) — original request
- Race02 trailblazer end (x 519..600, z −50..−115): its TIMELINE triggers (Triggers.trg rows 17..39, incl.
  "TriggerBigParticles" at (547.4, −17.8, −52.5), box 10×10×40) and the ai raceline sit at y ≈ −15.5..−18, but the
  physics car drives that stretch at y −9.3 (smoke `SMOKE_RACE=2 SMOKE_FOLLOW_LINE=1`, log
  `research/video/core_trails.log`), so none of those triggers fire. Please check whether the track collision
  there (a lower road / ramp, or race objects that should open it) matches the original.
- Closed by core: the street is reached by the down-ramp at x 465..510, z −50..−55; the car stopped at the deck's
  end wall (x 577.5). Details in core.md "Race02 street route".

### Requests from core (null pointer panic on the Race02 E-W deck) — fixed
- **Root cause**: `VISOS_ColourXBOX__2B11C0` (ColourXBOX ctor: bytes +3/+2/+1/+0 = min(2·r/g/b/a, 255),
  returns `this`) was overridden as `ov_null!`. 0x1A3A30 (car-vs-car) loops over mesh faces and, when a face's
  cross product has length 0 (a degenerate triangle, present on the E-W deck), draws three debug lines
  unconditionally: `v152 = *(_DWORD *)ColourXBOX(&local, 0,0,0,0)` dereferenced the null. Now
  `ov_colour_xbox` (ov.rs); no other `cx.i(ov_null!..)` deref remains in the generated code. Verified against
  the XBE (0x2B11C0 = four `FontManager__2B1190(2*x, 255)` min clamps, returns `this`); regen reproduces the
  committed tr.rs unchanged; **8 consecutive repro runs exit 0** (car crosses the deck in each). This is also the
  most likely cause of ai's old trailblazer panic. Note: a run under parallel cargo load died once with a wgpu
  DeviceLost (driver), not physics — run repro loops without concurrent builds.
- **0x398230 pool leak fixed**: source patch `operator_delete((void *)(dword_7B5710 + 0))` so the old scratch
  block is freed (the `(T)dword_X` = address rule otherwise deletes the global's address).
- Original report: intermittent (3 of ~6 headless runs) on LADown Circuit Race02 while the player drives the E-W deck east from
  (340, −9.4, −54) (x 340..370, z −52..−60; traffic car 14 scraping material 5 nearby in one run):
  `thread 'physics' panicked at vehicle_entity\cx.rs:709:9: load from Ptr { sp: Null, off: 0 }`, stack `Cx::i` ←
  `physics_manager::tr::f_1a3a30_body` ← `f_1b32e0` ← `PhysicsWorld::step_inner` ← `step` closure. Repro: smoke
  `SMOKE_RACE=2 SMOKE_FOLLOW_LINE=1 "SMOKE_VIA=340,-54;372,-65;420,-66;455,-53;475,-52.5;500,-52.5;547,-52.5;580,-52.5"
  SMOKE_DRIVE=218 SMOKE_MAX_SPEED=22 RUST_BACKTRACE=1`; logs `research/core_danger_run2.log`,
  `research/core_danger_panic.log` (earlier run, same panic at ~305 s).

### Requests from core (chase camera writes VE memory) — answered (core.md "Requests from physics (chase camera VE fields")
- Displacement scan (`tools/ida/phys_camscan.py` -> `research/phys_camscan.txt`, game code 0x11000..0x3DDD20):
  **+4436/+4440/+4444** (CVI rate/height/frame) only 0x001C5CF0 (core's port) and the zeroing in PhysicsCarInput
  0x001B9940; **+4392** written only by `VehicleEntity__1C2FF0` (graphics position/orientation, called from SyncModels
  0x001C1FF0 <- VehicleManager 0x001DDAE0 / Activate 0x001C5710, and from CarDamage 0x00070F40), read by 0x001C2FF0
  (graphics pivot y) and 0x001C5CF0; **+5008..+5020** = the *timeline player camera shake*: set by 0x001D1610 (only
  caller TimelineCameraShaker 0x0017A5F0, `F3EventPlayerCameraShake`, vehicle array [0]), zeroed by 0x001CABF0 /
  0x001B9940, decayed by the camera 0x00066940.
- Ported (translated): 0x001C2FF0 (+0x001BB020/0x001BB060), 0x001D1610; override `15D50 ov_race_time` (race manager
  +520 = `*(this+130)`, glob 0x6F5DD0, seeded by `set_camera_shake`). `PhysicsWorld::sync_model(id)` = 0x001C1FF0's
  physics part (non-network: 0x001C2FF0(ve, 0, pos, quat); model/wheel 0x001C4910 and net sync are render's), run
  for every active car (VE+28) at the end of `step` (SyncModels order) and from `activate` on 0 -> 1. Side effects
  as the original: lean state VE+36/+40/+44 and faked skid angle +4576..+4588 (skid exaggeration is off in the shipped
  Handling.cfg). API: `camera_up_offset(id)`, `camera_shake(id)`, `set_camera_shake(id, intensity, length,
  race_time)`, `graphics_pose(id)` (pos, quat). Test `tests/camera_fields.rs` (AI car cornering: 0 < |+4392| < 0.15,
  player 0; setter round trip). Not done: CarDamage 0x00070F40's own 0x001C2FF0 call (car_damage keeps its
  body-matrix stand-in; `graphics_pose` could replace it).
- Original note from core:
- The camera half of VehicleEntity 0x001C5CF0 (the point the chase camera follows) is ported in core
  (`velocity_game::camera::vs_vehicle_follow_camera::camera_point`, glue `game_cameras::follow_inputs`). Its
  vertical inertia spring (CVI_*) keeps its state in **VE+4436 (rate), +4440 (height), +4444 (frame)**, and core
  writes them back into the VE image every update (Cameras set, right after Physics, as the original's SyncModels
  0x001DDAE0 does inside the physics frame). It also writes **VE+5008** (impact shake amount, 0x00066940). These
  are physics-owned bytes: please keep them free for this (don't reuse or reset them elsewhere), and tell core if
  you ever model their physics-side writers (0x001C5CF0 is the only one found for +4436..+4444; VE+4392 and
  VE+5008..+5020 still have no physics writer in the port, so the up offset and the impact shake stay 0).

### Requests from core (jump cam, attract RandomiseSetup) — answered
- **Jump cam done.** `ov.txt`: 0x001665E0 -> `ov_jump_cam_can_see`, 0x001665A0 -> `ov_jump_cam_start`, both call
  `PhysicsWorld::set_jump_cam_hook(Some(Arc<Mutex<dyn FnMut(JumpCamCall) -> bool + Send>>))` (`cx::JumpCamCall::
  CanSee { choose }` = 0x001665E0(1): 0x00166B70 choose then 0x001649A0 "sees the car"; `Start` = 0x001665A0: +17 = 1,
  0x00166B70). The hook runs inside `step` on the physics thread (must not lock the world); no hook = no camera.
  Globals: `set_jump_cam_option` (`byte_75B278`), `review_cameras`/`set_review_cameras` (`dword_749110`),
  `review_timer() -> (749118, 74911C)` / `set_review_timer` (InGameUpdate's countdown). Flight time +4592 is the
  landing predictor's (set at take-off only with horizontal speed > `PL_min_horisontal_speed` 30 mph);
  thresholds `dword_6889F4` 0.65, `dword_6889FC` 0.1, probability `PL_jump_cam_probability` 1/3 vs 0x001CF070
  (VsRandUnit). 0x001CF0D0 (AI drive) has the same block without the option / probability checks. Regen changed
  exactly the four call sites. Test `tests/jump_cam.rs` (launches at 25 m/s from 8 m: CanSee then Start, review 2,
  countdown 0.764 = start; no camera / option off / too slow keep review 0). Wiring request: core.md.
- **RandomiseSetup done** (`CVehicleDatabase::randomise_setup(&mut setup, vehicle_colours, &mut CrtRand) -> cost`,
  helpers `random_decal_allowed` 0x001826D0, `get_setup_cost` 0x0018A980 without the team discount branch). Checked
  against the decompile: 16 licensed CCRCs, decal slots +0x194..+0x228 (the two kind-2 *flags* slots +0x220/+0x228
  get a decal pointer: original bug, kept as a non-zero stand-in), brand colours kind 3 -> +0xC8, kind 5 -> +0x148,
  vehicle colour `(int)(f32(n) * f32 r)`. The `vehicle_colours == 0` restore branch's
  0x00187BA0 (mix-your-own hue/shade -> setup +0xBC/+0xC0, `update_mix_colour`, ported 2026-11-07 for the frontend sliders; hue wheel 0x00187880 `hue_wheel_colour`) is now called there too. The original shares one
  rand state; pass the game's. Test `vehicle_database_all.rs::randomise_setup_every_vehicle`.
- Original request:
- **Jump cam** (VehicleEntity 0x001CCAD0 at 0x001CD0B0): after a flight longer than `dword_6889F4` on a player car
  (+4072, +4876 cooldown 0, not network) it asks the replay camera manager `0x001665E0(1)` (= a fixed camera can see
  the car), needs the video option `byte_75B278` (Jump Cam) and `GC+0x47C` (PL_jump_cam_probability) >
  0x001CF070, then sets `dword_749110` = 2, calls `0x001665A0` and writes `dword_749118` / `dword_74911C` = flight
  time − `dword_6889FC`. In the port the manager call is `ov_zero_i!` (always 0) and the three globals live in your Cx
  memory, so the jump cam never fires and core never sees it. Core owns the replay manager (`GameCameras.replay`:
  `review_cameras` = `dword_749110`). Please expose the decision as a callback or a message (e.g. a
  `JumpCamQuery`/`JumpCamStart { flight_time }`) so core can answer 0x001665E0(1) (`fixed.choose(review, w)`) and run
  0x001665A0 / the countdown (InGameUpdate 0x000B8329: −1/24 per update, then `dword_68580C` = 24, review 0,
  render cut); VehicleEntity 0x001C5800 / 0x001CF0D0 read `dword_749110` / `dword_749118` back (slow motion).
- **RandomiseSetup** (`CVehicleDatabase::RandomiseSetup(db, setup, setup, 1)`, PhysicsManager 0x0019E30E /
  0x0019E53A): called for every race car while the attract mode plays. Not ported anywhere; if you port it on the
  vehicle database, core calls it from `race_cars::plan_race_cars` when `CAttractMode.playing`.

### Gearbox: automatic / manual, HUD and audio rpm
- **Measured** (`VELOCITY_PHYS_GEAR_TRACE=1`, quick race smoke, `research/phys_gears_before.log`): the player car
  already shifted automatically (0..4 and back down); the "redline" is the readout: core shows FastCar engine speed
  x 9.549 (rad/s as rpm) = 5500..7700 at full throttle against the 6000 gauge. FastCar's gear choice picks the gear
  of maximum power, so engine speed sits near `ideal_RPM_to_change_gear_up` 740 rad/s by design.
- **Original automatic logic**: throttle 0x0035D6F0 (reverse 0x0035D940) sets car+2976 = 0x0035D1E0, which keeps
  the gear while car+3296 (VE+3536 = VE+48 hold timer > 0, set to 1.0 s by a manual gear up) or both car+3289/3290
  are set, else picks the max-power gear via 0x0035C980 (hysteresis car+3028 `relative_delta_RPM_to_change_gear`,
  car+3032 `min_power_advantage`), then refuses an up-shift when car+3289 and a down-shift when car+3290 is set.
  Pad (0x001CB530, record +16): bit 4 rising edge with +3289 set and gear < top -> 0x001D1210 (VE+3216 += 1, VE+48 =
  1.0); bit 8 with +3290 set and gear > 1 -> 0x001D1240.
- **Where the setting is read**: only `VehicleEntity::Create` 0x001CABF0: `0x001DA8E0(VE, setup +0x18C == 0, 0)`
  (VE+0xDC9 = car+3289, VE+0xDCA = car+3290 = 0, so a manual car still shifts down by itself). Setup +0x18C is the
  car select's `dword_768978+2220` (the FE's working setup at +0x720); no game option / profile field is involved.
  Ported: `LocalConfigInputs.automatic` (`from_setup` = `unk_18c != 0`, default true = the ctor's zeros), applied in
  `spawn` right after 0x001CABF0. Requests: frontend.md / core.md "Requests from physics".
- **rpm readers**: HUD (COSDInGameScreen::Update 0x000F9D72 -> OSDBridge +0), audio (VsVehicleAudio::UpdateRpm
  0x00052401, UpdateAiCars/UpdateTraffic), CarRender 0x0011ABD5 and PhysicsCarInput 0x001BA7A3 all call 0x001B6F10
  (`engine_rpm`, smoothing VE+4068 per call, output 1000 + w*RPM_multiplier/1000*(dword_6D4D98 - 1000)). dword_6D4D98
  is the audio's (StartPlayerEngine 0x0004C3A0 sets 6000; 0 in the image): `PhysicsWorld::set_audio_max_rpm`.
- Test `tests/gearbox.rs` (350Z player on the flat plane, full throttle): automatic 0->5, rpm 2351..6037; manual stays
  in 1st, gear-up taps every 2 s reach 5 (4->3 auto down-shifts still happen). Video `research/video/phys_gears.mp4`.

### Requests from frontend (LACoast Sprint 01 panic) — fixed
- **Root cause**: LACoast's `arena.qdt` is the only shipped **non-compact** quadtree (+0x5C = 0; every other track
  and the built walls are compact). `PhysicsManager::RayCast` 0x0019D3E0 -> 0x0019DB00 steps a ray through the
  leaves; on a non-compact tree it reads the node bounds through `*(qt + 84)` (FCQuadTree nodes pointer, 40 B
  nodes, cf. the grow function 0x00359330), and `Cx::i` handed out the stale file pointer (0x0C300020) -> `Sp::Raw`
  panic (`cx.rs:699`). The trigger was the chase camera's `line_check` (core `update_follow`). Fix (memory model,
  not tr.rs): `Cx::i` / `ptr_at` give handles for `Sp::Qt` +84 (nodes) / +88 (compact words) / +12 / +16 (indices)
  into the image (96-byte header, nodes, indices). Test `road_layers.rs::lacoast_long_rays_cross_leaves`
  reproduces the panic without the fix. Verified: 5 consecutive repro runs (`SMOKE_REVIEW=1 SMOKE_RACE_TYPE=2`)
  exit 0 (logs `research/phys_sprint_fix1..5.log`).
- `ov_null` audit (all call sites of the four `ov_null` overrides): `B::PhysicsManager(dword_705F48)` is two
  colliding demangled names = BarrierRender 0x001B3F40 (+8 quadtree) / 0x001B3F20 (+4 mesh) (checked call targets
  in the XBE): RayCast's flag-2 pass (barrier walls) never ran; now source patches like 1A7060 (also 1BB320's
  debug draw). `CBumpMap::CBumpMap` 0x001B3BD0 returned null, so `g_pBumpMap` stayed 0 and the player-car bump map
  (1BB320: `player && g_pBumpMap && +12`, height x material +8) was off; now `ov_bump_map_ctor` (+12 = 1).
  Left as is: `VsRaceManager::FindVehicleById` (1CB530, 1D1850; only feeds `VsAIRaceVehicle__23310` =
  `*(rv+0x3C8) && 0x24F10(..)`, an AI-side state this crate cannot see, stubbed 0 by `ov_ai_vehicle_flag`), and
  `699D0 ov_null_i` (1CC3F0 raceline respawn, see above). Not fixed, worth a look: `__PAIR64__` is `ov_zero_i!`
  (11 sites, e.g. 0x0019DB00's `v8` and FastCar 0x39xxxx `v2 = __PAIR64__(a1, HIDWORD(a1))`); the ones checked only
  feed values that are overwritten or a bool low byte.
- Original report: panic loading LACoast Sprint RACE01 (`SMOKE_REVIEW=1 SMOKE_RACE_TYPE=2`, quick race defaults): `vehicle_entity/cx.rs:699`
  "load from Ptr { sp: Raw, off: 204723764 }" on a compute task, before the session starts (log
  `research/fe_smoke_sprint.log` of that run was overwritten; reproduce with the env above).

### Requests from ai (AI review, 2026-11-05)
- Wet London Circuit Race01 (`SMOKE_RACE_PATH=LONDON\CIRCUIT\RACE01 SMOKE_WET=1 SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=120`,
  log `research/ai_review/wet.log`): the player hits something at (-1620, -47) and physics posts `Impact` /
  `CollisionParticles` with point `[-107374180; 3]` (= 0xCCCCCCCC as f32, an unset /RTC fill) and impulse 2.3e10;
  the next frame the camera transform is NaN and `sky_render::update_sky` panics (`Dir3::new_unchecked`). Same panic
  in core's sweep `LONDON2_CIRCUIT_RACE03_wet`. The ai inputs are finite up to the crash (trace in the same log).
- **Fixed (translator)**, see "Sweep crashes" below.

### Sweep crashes (core's `research/sweep/results.tsv`, 2026-11-05)
- **Root cause (one bug, both panics)**: Hex-Rays types FastCar's bool-returning contact tests as `__int64`
  (edx = register garbage, e.g. `HIDWORD(v5) = a4`). MSVC tests **eax only** (`test eax, eax` after every such
  call, `tools/ida/phys_callsite_test.py` -> `research/phys_dump/callsite_test.txt`), but the translator compared
  the whole i64. In compound-object contacts 0x00378750 the sphere-vs-car-box test 0x0037B860 then "hit" with
  eax = 0 and the record count was bumped without a contact: the record kept 0x001B32E0's /RTC stack fill
  (point 0xCCCCCCCC = -1.07e8, impulse ~1e10). Via 0x001AAED0 that became Impact / CollisionParticles events (NaN
  camera -> sky panic) and, as a message-0 car collision, a car-damage point no bone is within `BIG` of
  (`nClosestBone != -1`). Same misuse fixed at 0x001A7060's 0x00394EA0 test (barrier walls).
- Fix: `phys_c2rs4.py` `is_call64`: a direct call of an `__int64` function used as a truth value or compared to an
  integer literal with `==`/`!=` uses the low dword. Regen changed exactly those two sites. Test
  `tests/object_contact_fill.rs` (London2 object 0xBD0C8C40 at (415.7, 6.8, -202.2) reproduced it in 0.4 s; plus
  150 movables each of London2 / London).

| key | cause | status |
|---|---|---|
| LONDON2_CIRCUIT_RACE03_wet (NaN sky) | 0x0037B860 i64 test, 0xCC contact point via 0x001AAED0 | fixed, 2/2 exit 0 |
| LONDON2 CIRCUIT RACE01/02/03/04/07/08 dry+wet, SPRINT RACE05 dry+wet (`nClosestBone`) | same contact record -> car damage | fixed, each 2/2 exit 0 |
| LONDON_CIRCUIT_RACE09 (`pClosestSection`) | velocity_ai, not physics | not mine |
| timeouts / killed | no physics panic in the logs | not mine |

Reruns: sweep command line with `target/phys/debug/velocity.exe`, logs `research/sweep/phys_logs/<key>_<n>.log`
(no 0xCC value in any of them, no panic). 7 of the 26 runs (RACE04_wet both, RACE02/08_dry, RACE07_wet,
SPRINT05_wet one each) end in the smoke's own "timed out" after "still paused after the quit confirm, pressing A
again" — exit 0, race reached, the sweep's `timeout` class; a pause-menu quit issue (core / frontend), not physics.

### Handling review (2026-11-05) — `tests/handling_review.rs`
- Method: flat tarmac plane, stock setups (`setup_default_vehicle`, `from_setup`), player config (`bPLAYER` 1). Straight
  line and brakes on the pad (automatic), skidpad on the AI controls block + `set_steer_angle` at 25 m/s (mean
  speed × yaw rate over the last 8 s). Times are **game seconds** (physics advances 0.72 s per game s, see
  "Acceleration check"); speeds are FastCar m/s ×3.6; lateral g in physics time (× 0.518 for game time).
  `cargo test --release --test handling_review handling_review_table -- --nocapture`; `HRCAR=` one car,
  `HRDEFSPANS=1` the old default spans. `300ZX+` = every performance part at its top level, best brakes, softest tyre.

| car | 0-100 km/h | top speed per gear (km/h) | top | 100-0 km/h | max lateral |
|---|---|---|---|---|---|
| 300ZX | 5.96 s | 1:75 2:120 3:168 4:197 | 196.5 (= speed_limit 155 mph × 0.8) | 36.6 m, 3.75 s | 4.62 g |
| 300ZX+ | 4.83 s | 1:75 2:120 3:168 4:215 | 215.0 | 15.7 m, 1.71 s | 5.38 g |
| 240SX | 12.08 s | 1:53 2:83 3:112 4:140 5:151 | 151.3 | 29.7 m, 3.38 s | 3.27 g |
| 350Z | 5.79 s | 1:63 2:98 3:133 4:170 5:197 | 196.8 | 36.7 m, 3.75 s | 4.45 g |
| SKYLINE | 6.29 s | 1:75 2:114 3:154 4:195 | 194.7 | 35.7 m, 3.67 s | 4.36 g |
| VIPER | 4.00 s | 1:88 2:125 3:168 4:222 5:236 | 235.6 | 53.2 m, 5.12 s | 4.70 g |
| MINI | 6.33 s | 1:48 2:79 3:114 4:153 5:177 | 176.7 | 34.5 m, 3.58 s | 4.44 g |

- Every number follows the shipped data: top speed = `TOP_SPEED*MPH*TOP_SPEED_MOD` (stock PM 0 -> 0.8, full 1.0 ->
  1.0), power `BHP*...*ACCELERATION_MOD`, brake torque `MASS*5*BRAKE_MOD` (0.5 stock). The Viper's long stop is the
  data too: friction ramps up only above `the_low_speed_max = (0.25*TOP_SPEED+10) mph` (25.7 m/s for 190 mph).
  Lateral grip is arcade by design (kinematic path up to ~0.2 rad at 25 m/s; above that the cars slide or spin, hence
  the drop at 0.3/0.5 rad).
- Inputs checked against the XBE: performance modifiers 0x001895C0..0x00189AB0 (part types 6/7/9/10/11/12/8, tyre
  grip classes, brake part levels) and `setup_default_vehicle` 0x0018A210 (BRAKE_DISC/PAD_DEFAULT, TYRE_VS_MEDIUM)
  match; the gearbox is set by 0x0035DB90 -> 0x0035D000 (`GC level` 0.8 ≠ 0, so the 0x0035CD40 fit with its
  `__PAIR64__`/lost-st0 gaps never runs), then VehicleEntity overwrites car+3044.. with `mult / gear_ratios[i]`.
- **Fixed: wheel spans.** 0x001CABF0 takes the spans from the chassis model's `f_l_wheel`/`r_l_wheel` locators
  (0x001C0D60 loads `<name>_base` from `<path>\vehicleM.vcd` into race vehicle +1092); the port used the 2.58/1.4
  defaults for every car. `LocalConfigInputs::from_setup` now calls `spans_from_vehicle_model` (reader
  `xmf_locator` = GameDataLoading 0x000A3CB0): 31 of 33 cars have locators (300ZX 2.427 m wheelbase, 1.521 track);
  COPTO and HIN01 have none and get the original's "ill-chosen defaults" 1.0/2.0/2.0/0.5/0.5 (vehicle +0x38 is
  always set). Traffic cars keep `from_traffic` (their TrafficM.vcd models are not checked for locators yet).
- **Fixed: NaN in hard cornering** (300ZX 0.3 rad / Viper 0.5 rad at 25 m/s went NaN). Tyre solver 0x00372B70
  computes `sqrt(Fmax² - Fy²/k²)` (and `sqrt(Fmax² - Fx²)`) right at the friction ellipse, where f32 rounding
  makes the argument negative; the original's `comiss` + `jbe` sends the NaN ratio into the "<=" branch
  (0x00375297, 0x00374DD3) so it never reaches the force; Rust's `NaN <= x` is false. Source patches write both as
  `!(a > b)` (regen changed exactly those two lines; hand port `car_tyre` likewise). **General hazard**: every
  translated float compare has Rust NaN semantics; it only matters where the original produces a NaN on purpose.
- **Fixed: C2 config fallback.** The original keeps the one CLocalConfig object (VE+3552) across the Main.cfg and
  Mainsafe.cfg attempts (Mainsafe.cfg is not shipped), so C2 keeps its partial load; the port restarted from zeros
  (mass 0 -> panic in mem.rs). Now `load_local_config_with_fallback` loads Mainsafe on top. C2 itself is still broken
  data (engine power 0, physics NaN) as in the original, so it is left out of the table.
- Drive-path stubs now modelled: `tyrespray__154FC0` -> `ov_zone_fx_amount` = the **Zone FX** level of
  `flt_737AB0` (+24 + 4·pad, ZoneRender, graphics' `ZoneFx`; not Fury) × 0.5, read from Glob; feed it with
  `PhysicsWorld::set_zone_fx_level(pad, level)` before each step (while non-zero 0x001B82A0 clears the boost flag
  +20; test `jump_cam.rs::zone_fx_blocks_boost`). `LightStreak__133970` (a VehicleEntity inline linked into
  LightStreak.cpp) -> `ov_wheel_on_ground` = FastCar 0x00364BB0 (car+1088+4i > 0), for the skid check 0x001D1770
  of the rumble 0x001D1680. `__PAIR64__` sites returning
  edx:eax are read by low dword only where checked (0x001C9070's is patched out of the config load).
- Note: the hand ports no longer match the translation bit-for-bit for a player car (VELOCITY_EVOLVE_DIFF shows
  velocity differences ~1e-6 from the first frames; probably the player bump map added since); reference only.

### Requests from core (jump cam hook needs a ray test) — answered
- **What the original reads** (core_replay*.c dumps): 0x001665E0(this, choose) = `if choose: 0x00166B70(this+260)`
  (only sets fixed camera byte +13 = 1, "re-pick") then `return 0x001649A0(this+260)`; 0x001665A0 = +17 = 1, 0x00166B70.
  0x001649A0 gets the car as vehicle array [fixed +0] (PhysicsManager 0x0019E060 + VsArray) -> 0x001BA270 (body
  position) into fixed +52, and when +4 >= 1.5 s, +13 or 0x00166490(byte_749128) re-picks via 0x00164EC0, whose
  per-camera test is ReplayCamera_Orbit 0x00167680(cam, fixed +52, fixed +0) = MaterialProperties 0x0019D350(world,
  t, from, to, 1, vehicle) = RayCast 0x0019D3E0 with **flags 23** (track | barrier walls | cars | normal), the
  followed car ignored, any hit = blocked (`(_BYTE)r != 0`). Nothing else of physics is read.
- **Port**: hook type `cx::JumpCamFn = dyn FnMut(JumpCamCall, &mut dyn JumpCamWorld) -> bool + Send`; trait
  `cx::JumpCamWorld { car_position(&mut self, vehicle) -> [f32;3]; blocked(&mut self, from, to, vehicle) -> bool }`,
  served by `ov::CxJumpCamWorld` over the running step's Cx (vehicle array slot -> `ov_ve_get_position`; translated
  `f_19d3e0` flags 23, t preset 0.0 as 0x00167680). Vehicle = array index (live ids ascending = the physics id
  while no car was removed); out of range gives [0;3]. Test `jump_cam.rs::hook_reads_car_and_ray` (in flight: the
  car position matches, camera->car clear, ray into the plane blocked). Wiring: core.md "Requests from physics
  (jump cam ...)". Note: core's `CamWorld::blocked` uses `RAY_TRACK|RAY_FIRST_HIT`, the original flags 23.
- Original request:
- RandomiseSetup and the zone FX level are wired (core.md). The jump cam hook is not: 0x001665E0's answer is the
  fixed camera's `choose` 0x00166B70 / `sees` 0x001649A0, which test the line camera -> car against the track
  (ReplayCamera_Orbit 0x00167680, core's `ReplayWorld::blocked` = a ray cast on `PhysicsWorld`). Inside `step` the
  hook cannot lock the world, so it has no ray test and no current car position. Please pass them in, e.g.
  `FnMut(JumpCamCall, &dyn JumpCamWorld) -> bool` with `car_position(id)` (0x001BA270) and `blocked(from, to)` (the
  same ray the camera code uses), or call the hook with the Cx's world view. Core then installs it on race load
  (shared `Arc<Mutex<CReplayCameraFixed>>`, copied back after the step) and runs InGameUpdate 0x000B8329's countdown.

### Requests from core (jump cam end: `dword_68580C`) — answered
- `PhysicsWorld::jump_cam_rate()` / `set_jump_cam_rate(f32)` read / write `dword_68580C` in the Cx globals (image
  value 24.0). Writers in the XBE: FrontEndInit 0x000B5627, InGameInit 0x000B6099, InGameUpdate 0x000B836B (24.0),
  VehicleEntity 0x001C5800 (0x001C5AED/0x001C5AFD); readers VsCommonMain 0x0016E17C, FontHelper 0x000B38AC (core's
  clock). The 0x001C5800 block runs only for **control mode 0** (VE+0x13DC, `set_control_mode(id, 0)`) while
  `dword_749110` == 2: rate = `dword_6889F8` 18.0 once countdown < start − `dword_688A00` 0.7, else
  sin(0.5π·clamp((start − countdown)/0.7)) · (18.0 − 0.005) + 0.005 (`dword_688A10`), i.e. ~frozen at take-off,
  easing to 18 within 0.7 s. Core should read it after each step (it lives only in the physics Cx) and write 24.0
  at the countdown end. Test `jump_cam.rs::long_flight_starts_jump_cam` (0.005 at the start, setter round trip).
- Original request: Jump cam wired in core (`velocity_game::game_mode::jump_cam`, core.md). InGameUpdate 0x000B8329's countdown end
  writes `dword_68580C = 24.0` (0x41C00000) in your Cx globals (the value 0x001C5800 / tr.rs:1709 lerps during the
  jump cam). Please add a setter (e.g. `PhysicsWorld::set_jump_cam_rate(f32)` writing 0x0068580C); core calls it
  when the countdown reaches 0.

### Requests from graphics (dummy police car tag) — answered
- `PhysicsObjectInfo { object_type, object_index }` = instance record +56 / +58 u16 (0x001E5E80, set in
  `register_instances`; 0xFFFF for untagged objects) and `PhysicsObjectInfo::is_dummy_cop_car()` = 0x001E5F30(obj, 1).
  Available per frame in the Bevy resource `PhysicsObjects` (with `matrix`, `flags`). Shipped counts
  (`physics_objects.rs::register_all_compartments`): LADown 16 dummy cop cars + 15 hydrants, London 451 / London2 211
  streetlamps, none elsewhere.
- Original request: Add the object's type tag to `PhysicsObjectInfo` (instance record +56 u16, written by `f_1e5e80` in
  `register_instances`: 0 streetlamp, **1 `propld_dummycopcar`**, 2 hydrant, 0xFFFF none) — PhysicsObjectParticleTracker
  0x001E3B70 tests `*(u16*)(obj+136 → +56) == 1` (`PhysicsObjects__1E5F30`) to draw the dummy police car instead of
  the object (core fills `visos_render::car_render::DummyPoliceCars`).

### Split screen (two local players) — physics side complete, wiring is core's
- **Original** (VehicleManager.cpp): `m_nNumPlayers` (VehicleManager +20) = `dword_6F5C00` at load (assert
  `m_nNumPlayers <= m_pVehicleRefs.GetCount()`, line 347); the players are **vehicle array indices 0..N-1**.
  0x001DD9F0 calls 0x001C5800(car i, dt, pad i) for i < N when `VsAIRaceVehicle__23CC0` (human control) holds, else
  pad -1; every later car gets -1. 0x001CB530 reads pad record `VsTrafficManager +24 + 36·pad`. SyncModels 0x001DDAE0
  runs the camera point 0x001C5CF0 for every active car with control mode 0 (VE+0x13DC), so both players' CVI fields.
  No physics function reads the player count otherwise; nothing in VehicleEntity/PhysicsManager/PhysicsCarInput
  gates on split screen.
- **Port (already 1:1, verified by `tests/split_screen.rs`: two 350Z players, each moves only on its own pad)**:
  `step(dt, pads)` indexes pads by slot, pad record at traffic +24 + 36·slot (16 slots), `compute_forces` passes the
  slot when `is_player && human`; Bevy: spawn player 2 with `PhysicsCarSpawn { pad_slot: 1, inputs.is_player }` +
  `PhysicsCarInput` (the plugin builds the pad list from `PhysicsCar.pad_slot`). Spawn sets VE+0x13E0 = slot
  (0x001DC650; slipstream render index, `PhysicsSlipstream.vehicle`).
- **Per player state, all per vehicle id**: `camera_up_offset`, `camera_shake`, `graphics_pose`, CVI VE+4436..4444
  (core), `engine_rpm` (smoothing per VE), `shift_gear`, `launch_result`/`shift_result`, slipstream index.
  Per pad: `set_zone_fx_level(pad, level)` (ZoneRender has pads 0 and 1 only).
- **Shared / player-1-only in the original (keep)**: timeline camera shake 0x001D1610 writes vehicle array [0] only;
  `dword_6D4D98` audio max rpm (one value); jump cam globals (`dword_749110`/`749118`/`74911C`, `dword_68580C`) are
  single — any player car (+4072, control mode 0) can start it, the fixed camera follows vehicle array
  [fixed +0]; whether the game disables Jump Cam in split screen is InGameUpdate's / the option's (core).
- **Core must**: spawn the human cars first (physics id = vehicle array index while no car was removed), pad slot =
  player index, `set_control_mode(id, 0)` and `set_human_control(id, true)` for each, feed pad 1 from the second
  SDL pad, run 0x001C5CF0 per player camera.

### Requests from core (jump cam `blocked` panics, 2026-11-05) — fixed
- **Root cause: the original reads past a one-box car's box array** (not the translator, not `CxJumpCamWorld`).
  VE+3556 = `CBoxTree` {+0 boxes, +4 count} (0x001B4730, `operator new[](132·n)`, n = LC+64 `number_of_boxes`; box
  132 B: +12 bounding radius (0x001DA870), +52 half size, +116 offset, +64 parent). RayCast 0x0019D3E0's car pass
  (disasm 0x0019D6C2..0x0019D74A, translation is exact) reads box 1 unconditionally: radius = box0+12 + **box1 half
  y (+0xBC)**, and box0's half y is raised to box1 offset y + half y (+0xFC/+0xBC) for the test. Handling.cfg gives
  `BUS number_of_boxes 1` (LON_BUS01, LON_FIRE01, LA_FIRE01, SCHOOL_BUS), so on a bus the original reads heap
  garbage 56..124 bytes past the block; the port panicked in `Mem::f`. Any RAY_CARS cast (jump cam flags 23, AI
  `line_check`) near a bus could hit it.
- Fix: two source patches of 0x0019D3E0 (`phys_c2rs4_patches.txt`): with count < 2 the cabin box adds nothing
  (radius = box0+12, half y unchanged). Regen changed exactly those two statements. Test
  `traffic_cars.rs::every_traffic_setup_drives` now casts a flags-23 ray across every traffic car (4 buses incl.).
- Verified: the Race09 repro (env below, `target/phys/debug/velocity.exe`, logs `research/phys_jump_r09_1..5.log`)
  exits 0 in 5 of 5 runs; runs 2..5 take off at (-492, -9.8, -490), 53.7..54.1 m/s and drive on, run 1 was slow
  (machine under build load) and reached only (-364, -588) by 150 s. The jump cam itself never started (review
  cameras stayed 0, no `9_jump_cam` shot), so `research/video/core_jumpcam.mp4` is not recorded; the take-off is
  a 3.1 m/s hop; not checked whether flight time (< 0.65 s), the 1/3 probability or core's CanSee answer stops it.
- Original report: First real jump cam CanSee in London: **London2 Circuit Race09** (`SMOKE_DIRECT=1 SMOKE_AREA=2 SMOKE_TRACK=LONDON2
  SMOKE_RACE_TYPE=3 SMOKE_RACE=9 SMOKE_FOLLOW_LINE=1 SMOKE_JUMP_CAM=1 SMOKE_DRIVE=150`, take-off ~94..142 s after Go at
  (-493, -9.8, -489), 53 m/s; 3 of 3 runs) panics on the physics thread: `fastcar/mem.rs:33 index out of bounds: the
  len is 33 but the index is 47` in `f_19d3e0_body` (physics_manager/tr.rs:112, the car pass: `*(*(VE+3556)) + 188`
  read from a 132-byte block) <- `CxJumpCamWorld::blocked` (ov.rs:810, flags 23) <- core's
  `CReplayCameraFixed::nearest` (0x001649A0 line test) <- `ov_jump_cam_can_see` <- `f_1ccad0_body` (tr.rs:2794).
  Log with backtrace `research/core_jump_london2_r09bt.log`. The VE+3556 object of some car (traffic / police?) is
  smaller than the ray test expects; please size it / port it so `blocked` can run against every car.

### Requests from core (jump cam roll uses a private rand stream, 2026-11-06) — answered
- **All rand use in the port, checked against the XBE** (`tools/ida/phys_rand_xrefs.py` -> `research/phys_rand_xrefs.txt`,
  every caller of CRT `rand` 0x003DE030 / VsRandUnit 0x00013510 / VsRandRange 0x000134F0 plus one level of
  wrappers; `research/phys_rand_xrefs_phys.txt` = the physics ones). Live in the translated code, all inside `step`,
  in the original's order because the call sites are translated 1:1, all through `ov_rand` (`cx.glob.rand_state`):
  0x001A0530 (signed unit, from 0x001AAED0 x3), 0x001CF070 (VsRandUnit: 0x001CCAD0 jump cam roll, 0x001D1850
  vibration x2), 0x001CF090 (0x001CCAD0 x2, 0x001CF0D0 x2, 0x001D1850 x2), 0x001DFBB0 (0x001DEBB0 object break x9).
  RandomiseSetup 0x00182720 takes the game's `CrtRand` already. Dead in the original (not translated, correct):
  FastCar 0x00369860 from 0x003682B0 / 0x00372B70 (behind `xor reg,reg; jz`, Hex-Rays drops them), the unreferenced
  VsConfigFileRead 0x001B7690 -> 0x001B77C0. PhysicsManager 0x0019FD80 (0x001A0530 x3 per object and per car; the
  0x001A0510 cone branch is unreachable) and 0x001E1A20 (VsRandRange x2) are ported since and draw from the same
  `ov_rand` stream. Out of physics but called inside the original's physics frame (so their draws interleave
  there, while the port runs them from the events afterwards): `VsAudio::PlayPhysicsImpact` 0x00052E90 ->
  AUDIO_SelectSound -> `_AUDIO_RandomFloat` 0x002BB910 (audio, from `PhysicsObjectEvent::Impact`), the AI collision
  listener 0x000160B0 (ai), CarDamageFX 0x00078320 (not ported). The jump cam hook entries 0x001665E0 / 0x001665A0
  reach no rand (checked transitively), so the hook does not need the stream.
- **API**: `PhysicsWorld::rand_state() -> u32` / `set_rand_state(u32)` (`ov_rand` now runs `visos::handy::crt_rand::
  CrtRand` on that state). Core: `set_rand_state(gm.rand.state)` right before the physics tick, `gm.rand.state =
  rand_state()` right after (request in core.md). Test `jump_cam.rs::roll_uses_the_handed_in_rand_stream` (8 seeds
  handed in per step: the roll step advances the handed-in state, both outcomes occur). Regen: tr.rs unchanged.
- Original request: The jump cam now fires in game (core.md "Why Race09 did not fire"). Its 1/3 roll (tr.rs:2794, `f_1cf070` ->
  `vs_rand_unit` -> `ov_rand`) draws from `cx.glob.rand_state` (seeded 1 per world), while the original's VsRandUnit
  0x00013510 / CRT `rand` 0x003DE030 is the one stream the whole game shares (core's `GameManager.rand` = `CrtRand`,
  ai, particles ...), so the Race09 crest's roll fails on most runs. If you want it shared: expose
  `PhysicsWorld::{rand_state, set_rand_state}` and core will hand `gm.rand`'s state in before each step and take it
  back after.

### Requests from core (speedo gauges: zone, turbo; nos names, 2026-11-06) — answered
- **Names were swapped (fixed)**: disasm of 0x00024720 (`comiss [+64h]` vs `[+60h]`, assert when +100 < +96, returns
  +96/+100) and 0x000248F0 (`fld [+60h]`, the AI's "NOS left" in 0x0001D160/0x0001D690, also 0x001C5800) =>
  `nos_remaining` = VE +96, `nos_capacity` = +100. The setters agree: 0x001DA6E0 writes +96 (×24), refill 0x001CB300
  `+96 = +100`, `+104 = +108`. **This also fixes the AI**: `want_nitrous` had tested the capacity and
  `should_conserve_nos` used the inverted ratio. New `nos_ratio` (0x00024720), `zone_remaining`/`zone_capacity`/
  `zone_ratio` (0x000F78D0, +104/+108; Hex-Rays had lost its return), `has_turbo` (0x000F7970, +144 byte as f64, set by
  0x001CABF0 from setup +48), `turbo_ratio` (0x000F79A0, +148, PhysicsCarInput 0x001BA770 / 0x001D1850 writers, scan
  `tools/ida/phys_turbo_scan.py` -> `research/phys_dump/turbo_scan.txt`). Ratios return f64 (x87 fdiv). Dump
  `research/phys_dump/speedo.txt` (`tools/ida/phys_speedo.py`). Test `tests/speedo_gauges.rs` (boost drains
  remaining; `default_spans` has a 1-frame tank, zone 2400/2400). Core told to use `nos_ratio` (core.md).
- Original request: Speedo::GetRatioForGauge 0x000F4F40 reads physics vehicle [p] (VehicleEntity.h inlines): nos 0x00024720 = VE +96 /
  +100 (assert "m_NosRemaining<=m_NosCapacity" fires when +100 < +96, so **+96 = remaining, +100 = capacity**: your
  `nos_remaining()` (+100) / `nos_capacity()` (+96) are swapped by name; core uses `nos_capacity / nos_remaining` =
  +96 / +100 for now), zone 0x000F78D0 = VE +104 (m_ZoneRemaining) / +108 (m_ZoneCapacity), turbo 0x000F7970 = VE
  +144 byte (has turbo) and 0x000F79A0 = VE +148 (float ratio; 0x001D1850 reads both too). Please add
  `PhysicsWorld::{zone_remaining, zone_capacity, has_turbo, turbo_ratio}(id)` (and fix or document the nos names);
  core then fills `RaceStatus.zone` / `turbo` (now 0 / 1). Dump `research/core_speedo_ratios.c`.

### Explosions and friction spots (2026-11-06) — translated, `tests/explosions.rs`
- **PhysicsManager 0x0019FD80** (only caller TimelineExplosion 0x0017ACD0, args = the `F3EventExplosion` attributes
  +0..+24, `a9` cone null, `a11` = Ignore_Player_Car ? 1 : -1 compared with the **vehicle array index**: original quirk,
  index 1 is skipped). Movables from the axis list query 0x00385920 within the radius: wake 0x001E4710, body velocity
  (+32) += (obj − centre)·(r − d)/r·force/d, angular (+48/+52/+56) += 0x001A0530()·(r − d)/r·angular per axis (x87),
  message 2 (→ 0x001E1F80, so hydrants in range drop spots), body +536 = 1. Cars (VE+28 active, not `a11`): VE+272 /
  +288 the same with the vehicle forces (d < 0.01 → (0, 0.01, 0)); then every car with VE+5084 == 0 gets VE+4876 = 96
  (0x001D1170). Returns the count. Source: the old `PhysicsManager.c` text plus `phys_c2rs4_patches.txt` (the array
  `_DWORD *v28[46]` retyped to `int`, `v28[38][25]` = +100, the 0x001A0530 st0 results put back where Hex-Rays had
  lost them; a fresh decompile still comes out with `__asm`).
- **Friction spots**: 0x001E18E0(pos, a3, a4, distance, car, a7): ray pos ± `unk_7805B0` = (0, 1, 0, 0) (static ctor
  0x00431EF0; `ov_up_vector`) through 0x0019D350 flags 17 ignoring `car`; on a hit, lerp 0x000762F0 → 0x001E1A20:
  debug cross 0x002B0C20 (ignored), 0x001B3500 (thiscall; zone i of 100 at PhysicsManager +36 + 28·i: +0 active, +4
  pos, +16 radius = a4, +20 = a3, +24 = a7; faces of the quadtree leaf containing pos that 0x003A8F30 finds within the
  sphere get material bits |= (i + 1) << 8 — read by the wheel code 0x001BB320 (GC `friction_spot_*` 592..608)), then
  the 48-byte puddle record (2 × VsRandRange) to graphics' `unk_720840` (`PhysicsPuddle`). Hydrant (0x001E1F80 case
  0x17): a3 = GC `friction_spot_friction_multiplier_high_speed` (+592), a4 = `friction_spot_radius` (+608, 24 m),
  distance = AI raceline (`set_race_distance_hook`), car = msg +0, a7 = FLT_MAX; 0x40 only on a hit; then
  `Explosion` (= its AddEnvSound) as before. 0x001B3850 clears a zone; **0x0019C390** (GameManager::RestartRace) resets
  objects (0x001DE400 hand port via `ov_reset_all_objects`), +0x10 / +0x5F08 = 1.0, clears every zone and tracker owner.
- Memory model: the track mesh is now writable — the first `Sp::Mesh` set-0 store copies the image into
  `PhysGlobals::track_mesh`, which reads then prefer. IDA: 0x001E1A20 / 0x001B3500 typed thiscall (retn 14h / 10h,
  edx was never an argument), redumped with 0x0019D310/0x0019D350/0x001E18E0/0x000762F0/0x0019C390 into Redump.c.
- **Bug fixed on the way**: `VsAdd` 0x00024960 is thiscall (`a`, out, b) but `ov_vec_add` wrote into its first
  argument: 0x001D1480's perfect-shift boost (VsAdd(velocity ptr, &out, forward·k) then 0x0036B530(out)) overwrote the
  car's velocity source and then set the velocity from an uninitialised frame. Now (a, out, b), w = 1; `VsSub` 0x00016450
  likewise (`ov_vec_sub`). New overrides: `VsRandRange` (`vs_rand_range`), 0x00133A80 (VsVector4 ctor), 0x00133860
  (SetW), 0x00133750 (`ov_puddle_add`). Translator: stem `CarDamage` maps to tr_ext.rs.
- API: `PhysicsWorld::explosion(&sim::Explosion) -> i32`, `friction_spot(&sim::FrictionSpot) -> bool`, `restart_race()`,
  `friction_spots()`, `take_puddles()`, `set_race_distance_hook(..)`; Bevy `PhysicsExplosion`, `PhysicsFrictionSpot`,
  `RestartPhysicsWorld` (Setup, before / after the resets), out `PhysicsPuddleMessage`. Requests: core.md, graphics.md,
  ai.md "Requests from physics (explosions / friction spots ...)".
- Test `tests/explosions.rs`: London explosion wakes 7 objects with exactly 21 draws from the handed-in stream, a car
  6.02 m away gets 3.592 m/s = (r − d)/r·force away from the centre, ignored index untouched; a spot 0.5 m above
  ground registers (2 draws, record fields as above), 5 m up misses, restart clears it; LADown Race02 uinstance
  hydrant (7, kind 23) hit through an explosion drops a 24 m spot with the hooked race distance, sets 0x40, once.

### Requests from core (Handling.cfg camera overrides, 2026-11-06) — answered (core.md "Requests from physics (Handling.cfg camera overrides")
- `PhysicsWorld::handling_camera_overrides() -> sim::HandlingCameraOverrides`: one `Option<f32>` per global
  0x001C9070 writes (far clip 0x6850A0, yaw 0x68519C / 0x6851A4 / 0x6851B4, wheel cam FOV / X / Z 0x68768C /
  0x687690 / 0x687698); Some = written by this world (`PhysGlobals::w` entry; none of them is in `GLOBAL_INIT`), so
  None = the .data value stands. Written at spawn (`spawn` runs 0x001C9070). Test `camera_fields::handling_camera_overrides`.
- VehicleEntity 0x001C9070 (`tr.rs` ~1981) writes `g_ftFarClipDistance` / `g_yawRatePerDegree` / `g_yawRelaxAngle` /
  `g_yawDirectionBlend` (when != 0) into `Ptr::glob(0x6850A0 / 0x68519C / 0x6851A4 / 0x6851B4)`, i.e. your emulated
  memory; the live globals are now core's `GameCameras.manager.clip.far` / `.globals.{yaw_gain, yaw_full,
  heading_weight}`. Please expose those writes (e.g. an `Option<f32>` per value on a resource, or a message) so core
  applies them. Shipped Handling.cfg: 0 / 200 / 15 / 0.6 = the .data defaults, so nothing changes today.

