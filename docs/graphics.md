# Graphics - INDEX

_owner: graphics agent. Details live in `docs/graphics/details.md` (the full former graphics.md, ~240 KB): NEVER read it whole; grep it
by heading (`grep -n '^###' docs/graphics/details.md`, then `Read` with offset/limit; the list below gives line numbers, they drift as details grows)._

**Coordinate convention (all agents): game coordinates are used in Bevy as-is, no axis negated, no winding flip.** Right-handed Y-up, front faces CCW (cull Back). D3D row-vector matrices map to `Mat4::from_cols_array(&m)`. Viewer yaw 0 looks down -Z.

## APIs other crates use (details: "Render API for game flow", lines 13-58)
- Track: `LoadTrackRender::new("LACoast")` / `UnloadTrackRender`; tag the 3D camera `TrackCamera` (order -40, fov `sky_render::BACKGROUND_SPAN`); `TrackRoot`, `TrackLights`.
- Car: `SpawnCarRender { request, spec: CarSpec::new(dir, colour).with_colour(&CarColour::from_record(..)), transform }` / `DespawnCarRender`; root gets `CarRender` + `CarRenderParts { body, wheels, bounds, wheel_radius }`; set root and wheel `Transform` each frame.
- `CarLightInput { braking, reversing, damage }` on the car root each frame; `CarFlareMessage`, `FlareFlash`, `PoliceFlash` for flare/light hooks.
- Skinned characters: `SpawnSkinnedModel { request, pack: CharPack, model, anim, transform }`, `SkinnedModelAnim`, `DespawnSkinnedModel`, `SpecCharIndex`, `SkinnedModelBlend`.
- Front end: `FrontEndCamera`, `FrontEndRender`, `FeReflection` (reflect-flagged elements on `FE_REFL_ENV_LAYER`), `car_render::fe_closest_lights`, `set_lit` + `hud_light_set`, `with_setup_underbody`.
- Effects: `PuddleAdd`/`PuddleRestart`, `MotionBlurSecondInput`, `FuryFx::amount` + `FuryNosAudio`, `CameraRain*`, `DangerParticles`, barrier hit/wobble, `MtvEffect`, `view_render::MirrorView`, split screen (two `TrackCamera`s with `PlayerView(p)`).
- FE car parts: `CarRenderParts { decal_slots[6], doors[8] (FE_DOOR_NAMES), door_locators[8], fe_wheels[4] (f_l,r_l,f_r,r_r), wheel_pivots, wheel_base_rot, discs }`; `CarPrimLists` (carbon/glass prim lookups).
- Viewer: `trackview --track ExecTest --car LA/350z --shot out.png [--cam x,y,z,yaw,pitch]`; screenshots in `research/screens/` (list in "Render API").

## OPEN requests
- From frontend (2026-11-07, front-end tyre change): ANSWERED, `CarSpec.tyre: Option<String>` (= `Tyre.name` +0x14, a `\Vehicles\TyresT.vcd` stem such as "VS_soft") rebinds the stage 0 texture of the wheel shaders named "*_tyre" on all 4 wheels (`car_render::apply_tyre_texture`); frontend sets it from `setup.tyre` and adds it to its respawn key. Wired 2026-11-07 (`foyer.rs`, in the respawn key; shot `research/screens/fe_tyre_texture.png`).
- From frontend (2026-11-07, customisation decals): ANSWERED, `CarRenderParts.decal_slots: [u8; 6]` (CarRender 0x00116BC0 = `car_render::decal_slot_count`, kinds 0 chassis, 1 f_bumper, 2 r_bumper, 3 trim, 4 front, 5 rear screen; 0 for traffic) filled at spawn; frontend copies it to `fe.foyer_vehicles.decal_slots`. Decals are drawn on the body already (`CarSpec.decals`, details "Licence plates and decals").
- From frontend (front-end door parts): ANSWERED, see APIs (`CarRenderParts.doors/door_locators/fe_wheels/wheel_pivots/wheel_base_rot`; `discs` / `disc_pivots` filled in stage 12 when the spec has `brakes`, which `with_setup_underbody` sets from the setup's brake parts; the frontend poses them like the wheels, RotY about the pivot).
- From core (2026-11-08, perf stage 7 answers): fly-camera ground cast fixed (no 27 s `update_cameras`), InGameInit timer reset + in-race shots at an exact update (smoke "shot at update N lerp L" log lines; `--fps 0` vs `--fps 20` london_wet_cams 13/13 pass), `effect_clock`/`mode_messages` ordered. Every race golden image changed (intro camera at street level, shot timing): please recapture `research/golden/ref` race scenes with the new build (see core.md).
- From core (2026-11-07, "Requests from core (GameDataLoading lighting loaders...)"): optional `LoadedLights` input in visos_render filled from `gdl_lights::load_lights`, so `track_render`/`track_fx` can drop their own parsers.
- (done stage 7, `EffectOrder` + `RaceEffectsMessages`) From core (2026-11-08, golden determinism): `race_effect_messages` (race_effects.rs, Update, `ResMut<Particles>`: spark / collision debris spawn) and `update_particles` (stateless_particles/mod.rs, Update, `ResMut<Particles>`) are unordered, so adding any unrelated system can flip their order and change spark pixels (london_wet_cams shot 06 "6b_sparks": mean 0.37, 1.2% > 8 between two builds differing only by inert systems; every other shot is bit-exact / noise-level). Please order them (e.g. `race_effect_messages` before `particle_messages`/`update_particles`, and the other `ResMut<Particles>` writers: fury_fx, car_damage_fx, danger_zone_render) so the gate is order-independent. Core side is done (smoke shots keyed to game ticks).
- To others (open, see "Stubbed / not yet" ~line 1692): frontend 3D HUD/OSD per view, name tags per view; core `DummyPoliceCars`/`CarSpec.police`, `FlareFlash` from TimelineCameraShaker, `CarFlareMessage` from ai hazard/police hooks, `SpecCharIndex` on flag girls, `PuddleAdd`/`PuddleRestart` wiring, FuryFX wiring, `MotionBlurSecondInput`; physics object type tag +56 in `PhysicsObjectInfo`; ai `FlareFlash` from VsAIRaceVehicle 0x19740 event 1; frontend `SpecCharIndex` from FE character element +764.

## Performance gate (details "Performance"; user: optimisations must not regress)
- Harness `visos_render::golden` (`VELOCITY_GOLDEN=<dir>`, frame-locked time, captures after each smoke shot once pipelines are compiled; `VELOCITY_GOLDEN_PERF=n` timings + GPU passes; `VELOCITY_PERF_OFF=envmap,road,mirror,stencil`), `tools/golden.py capture|compare|perf|list` (11 scenes: menus, garage, London wet + car views/mirror, Tokyo, split, particles, night lights, FE car), `tools/perf_trace.py` (trace_chrome build). Stage 1: tools built, numbers/refs not taken yet (exact commands in details).
- Every optimisation stage: one optimisation per commit; each gated by `golden.py compare research/golden/ref <new>` within the recorded noise floor + `cargo test -p velocity_game camera::vs_vehicle_follow_camera_golden`; fps before/after recorded in details. Never re-capture refs to make a compare pass.
- Stage 2 done (2026-11-08): refs in `research/golden/ref/` (deterministic, noise <= 0.0006 mean), baseline `research/perf/baseline.md`: London wet 17 fps, envmap off +49 %, mirror +16 %, road +6 %; ~25 Core3d views per frame is the main cost; top-3 optimisations in details "Performance".
- Stage 3 done (2026-11-08): envmap source = world camera's main texture copied after its opaque pass (cars in the transmissive phase via `StageState.car_pass`); 5 views/frame removed; london run 12.6 -> 16.5 fps (+31 %, same session). Race shots are not comparable to `ref` across builds (core.md request 2026-11-08: paused-frame RNG + smoke shot order); gate = A/B vs a HEAD worktree build with `VELOCITY_GOLDEN_SETTLE=60`. Next: clustering off on all cameras (no bevy lights exist), mirror camera cost.
- Stage 4 done (2026-11-08): `race_effect_messages` / `car_damage_fx` ordered before `particle_messages` (core's determinism request answered); refs re-captured (`research/golden/ref`, two captures bit-identical); clustering `Single` on all Camera3d (`None` breaks bevy 0.19): london +34 %, tokyo +13 %, split +10 %, golden unchanged. Next: mirror camera cost (details "Performance stage 4").
- Stage 5 done (2026-11-08, details "Performance stage 5"): camera census (`VELOCITY_GOLDEN_CAMS=1`), one sky camera per view (`skyopt`), post camera only with flash batches (`postopt`; POST_VIEW_LAYER 23 -> 11, it collided with frontend HUD_3D_LAYER_P2), unchanged FSE/MTV/paste meshes not re-uploaded (`meshopt`). Next: FSE passes are 27 of ~35 graph runs (one render node for consecutive passes).
- Stage 6 done (2026-11-08, details "Performance stage 6"): consecutive FSE passes on one camera (`fse_runs`, `fseopt`; london 35 -> 28 graph runs, +10 %), MTV_LAYER 17 -> 10 (collided with VIEW_CAR_LAYERS[0]), golden.py pins the profile (`research/golden/profile_seed`, Rumble=1); refs re-baselined (split icon fix, hangout uv8 fix). Next: bloom ping-pong (10 cameras) needs a custom multi-pass render node.
- Stage 7 done (2026-11-08, details "Performance stage 7"): `effect_draw::EffectOrder` fixes the sparks flip (EffectDraw producers ran in scheduler order; `VELOCITY_AMBIGUITY=1` = bevy ambiguity report); trace: render-thread bound, 23 graph runs/frame; GPU clustering off (`gpucluster`; london +22 %, tokyo +36 %, split +21 %); `ProgramMaterial::rebind` (`bindopt`; london +6 %). Next: bloom in one render node.
- Stage 8 done (2026-11-08, details "Performance stage 8"): refs re-captured (core timer / fly-camera changes; 2 captures bit-identical); `fse_chain` (`FseChainGraph` camera, `fsechain`) draws consecutive off-screen FSE passes (bloom ping-pong) in one graph run, chain cameras pre-spawned (entity order changes race debris); Core3d runs 23 -> 9/frame; london +59 %, tokyo +52 %, split +38 %. Next: car ProgramMaterial per-object constants, check_visibility.
- Stage 9 done (2026-11-08, details "Performance stage 9"): car program materials written after visibility, only when seen (`carvis`, `PendingCarPrograms`; london +1..+10 %, split +2..+4 %, tokyo noise); `chainvis` tried, no gain, reverted. london_wet_cams_06 differs from ref with all stage 9 switches off (pre-existing, game side). Next: render-thread costs (effect batch mesh uploads, Core3d runs).
- Bug 2026-11-08 (FE hangout flat vs real Xbox): packed uv (NORMSHORT2) × 8 = DefaultPacked vp (`model_builder::packed_uv`), details "Requests from frontend (black FE hangout ...)".
- (done, core landed it) `GameClock::now` must return `golden::clock_ns()` when Some (race scenes deterministic).

## Next candidates (see "Coverage audit")
- 0x1181C0 / 0x10F9C0 checked, not ported (see details "Coverage audit" → stage 10): light-set binding only; the brake/reverse/head prims are already drawn by `update_car_lights`.
- Skipped: 0x13AB80 / 0x13A810 (SkinnedObjectRender load of `\Scenery\SknObj_*.VCD`, files absent from the install), 0x5C220 debug camera. Done in stage 11: 0x1174F0 / 0x117BB0 (`CarPrimLists`); 0x11B1E0 (ctor), 0x10FD20 (SetPart), 0x12B2D0 / 0x12C450 (GFXHelper draw/camera-matrix helpers) are platform/struct plumbing, not ported (details "Stage 11"). Stage 12 triaged the rest of the visible-rendering list (details "Stage 12": mostly sort/unload/debug/flag helpers). Only open item: 0x11ABB0 engine shake of the followed car (needs rpm + camera link, core to apply). Next: StatelessParticleType getters.

## Coverage
475 fn / 306004 B (47.0 %) (was 472 / 304491 B) of 3289 fn / 651558 B in the graphics regions (`tools/coverage_gfx/cov.py` -> `research/graphics_coverage.txt`, 2026-11-07).

## Headings in details.md (line numbers)
- 13 Render API for game flow (`visos_render::RenderPlugin` adds both plugins)
- 60 Formats
- 98 Ported
- 235 Tweaks (`gfx_tweaks`, the 21 graphics AddTweaks; registered tree done, menu hook-up = core request)
- 269 Character specular (`specular_chars`, SpecularChars.cpp 0x143140..0x146D50; done)
- 311 Vertex animation (`vertex_anim`, VertexAnim.cpp / VertexModel.cpp / VertexAnimBlendTree*.cpp; data + m
- 328 Beam lights / "Graphics.3DLightBeams" (`projected_light_beam_pc_xbox`, `model_builder::{Builder::beam,
- 394 Sector streaming, sun/moon, flag girl height
- 421 Race effects and particles (stage: effects)
- 496 Track flares and light streaks (`track_fx`, `light_streak`; `TrackFxPlugin`, automatic with the track)
- 604 Full-screen effects (`full_screen_effects_manager_pc_xbox`, `motion_blur_pc_xbox`; added by `RenderPlu
- 780 Fury FX — nitrous (`fury_fx`, `fury_instance`; `FuryFxPlugin` in RenderPlugin)
- 835 Driver, fade, lens spray, slipstream distortion (stage 3)
- 944 elParticle and tyre spray (stage: elParticle)
- 1043 Puddle mist (`puddle_render::{PuddleMist, MistHit, MistCar}`, in `PuddleRenderPlugin`; done)
- 1065 Wheel blur (`car_render::{CarWheelBlur, CarWheelSpin, wheel_blur_levels}`; done)
- 1093 Camera rain (`camera_rain::{CameraRain, CameraRainInput, CameraRainInit/Reset/Shutdown}`, `CameraRain
- 1129 TrailBlazerRender — zone trails (`trail_blazer_render`, `TrailBlazerRenderPlugin` in RenderPlugin)
- 1168 Danger particles (`danger_zone_render::{DangerParticles, DangerParticleCall}`, in `DangerZonePlugin`)
- 1183 Dynamic envmap (`danger_zone_render::dynamic_envmap`, `car_shader_manager_pc_xbox`; `DynamicEnvmapPlu
- 1223 Fixed-function car sets (`car_palette_pc_xbox::{fixed_car_set, FixedCarSet, car_colour_gradient}`, do
- 1282 RainSurface road layer + wet road reflection (`rain_surface`, `projected_light_beam_pc_xbox`; done)
- 1345 Car light flares (`flare_render`, FlareRender.cpp 0x123A20..0x1273B0; `FlareRenderPlugin` via TrackFx
- 1423 Front-end floor reflection "FErefl" (`rain_surface::{FeReflection, FrontEndCamera, FrontEndRender, Fe
- 1460 White car surfaces in races (fixed; user priority)
- 1483 Split screen — two players (`view_render`; stage "split screen")
- 1579 Rear-view mirror (`view_render::{MirrorView, setup_mirror_camera, mirror_scene, update_mirror_orders}
- 1637 Multi-layer blend trees (checked; the one shipped user ported)
- 1649 Swapchain guard (`swapchain_guard::SwapchainGuardPlugin`, in RenderPlugin; platform layer, no origina
- 1675 Stubbed / not yet
- 1714 Light sets, 3d hud lighting, front-end car shadows (`vs_render`, `car_render`, `model_builder`)
- 1770 Car damage rendering (`car_damage_render`; done — was physics' request below)
- 1830 Licence plates and decals (`licence_plate_render`, `car_render` decals)
- 1875 Requests from physics (CarDamage, see physics.md "CarDamage") — done (see "Car damage rendering")
- 1890 Requests from core (collision listener) — done, see "Barrier render"; core's wiring requested in co
- 1892 Barrier render (`barrier_render`, BarrierRender.cpp 0x00106B30..0x0010B740; `BarrierRenderPlugin` in 
- 1925 Requests from core (trailblazer render calls) — done: see "TrailBlazerRender — zone trails" and "
- 1936 Requests from ai (puddle test never splashes — `puddle_render::puddle_test`) — done (fixed, seen 
- 1943 Coverage audit (`tools/coverage_gfx/cov.py` -> `research/graphics_coverage.txt`; 2026-11-07, reconcil
- 2026 MTV glitch overlay (`mtv_effect::{MtvEffect, MtvEffectPlugin}`, LightStreak.cpp 0x1300D0..0x1310E3; i
- 2053 Light sets: ClosestLights (VsRender.cpp 0x5A710; `VsLights::closest_lights`, `car_render::fe_closest_
- 2065 Cat's eyes (`cats_eye_render`, CatsEyeRender.cpp 0x11CDB0..0x11E080; `CatsEyeRenderPlugin` in RenderP
- 2091 Requests from frontend — done (second race load crash fixed)
- 2117 Requests from audio (zone heartbeat never starts) — answered: original behaviour, port unchanged
- 2128 Requests from core (split screen world cameras, 2026-11-05)
- 2145 Requests from physics (friction-spot puddles, 2026-11-06) — done (see "Runtime puddles"; core wirin
- 2155 Requests from frontend (black FE hangout background: D3DVSDT_NORMSHORT3 positions, 2026-11-06)
- 2190 Requests from core (GameDataLoading lighting loaders, 2026-11-07)
- 2200 Requests from frontend (front-end door parts: per-part transforms, 2026-11-07)
